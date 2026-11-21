//! In-memory high-concurrency merge rehearsal (`agent-on merge-check`).

use crate::git_util::{
    canonical_or_raw, commit_exists, diff_name_only_z, git_raw, git_stdout, list_worktrees,
    query_open_prs, resolve_base_ref, resolve_repo, short_oid,
};
use clap::Args;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct MergeCheckArgs {
    /// Target repository path (defaults to current directory)
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Base branch/ref to rehearse merging into (auto-detects origin/main, main, etc.)
    #[arg(long)]
    pub base: Option<String>,
    /// Candidate branches/refs to rehearse (comma-separated or repeated)
    #[arg(long = "branches", alias = "branch", value_delimiter = ',')]
    pub branches: Vec<String>,
    /// Rehearse all open GitHub PRs via `gh pr list`
    #[arg(long)]
    pub prs: bool,
    /// Rehearse specific GitHub PR numbers (comma-separated or repeated)
    #[arg(long = "pr", value_delimiter = ',')]
    pub pr_numbers: Vec<u64>,
    /// Output machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BaseSyncState {
    UpToDate,
    SyncFree,
    CleanWithOverlap,
    BaseConflict,
    UnfetchedRemote,
}

impl BaseSyncState {
    pub fn label(&self) -> &'static str {
        match self {
            BaseSyncState::UpToDate => "UP_TO_DATE",
            BaseSyncState::SyncFree => "SYNC_FREE",
            BaseSyncState::CleanWithOverlap => "CLEAN_WITH_OVERLAP",
            BaseSyncState::BaseConflict => "BASE_CONFLICT",
            BaseSyncState::UnfetchedRemote => "UNFETCHED_REMOTE",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateReport {
    pub id: String,
    pub label: String,
    pub pr_number: Option<u64>,
    pub is_draft: bool,
    pub head_oid: Option<String>,
    pub merge_base_oid: Option<String>,
    pub base_sync: BaseSyncState,
    pub changed_files: Vec<String>,
    pub base_overlap_files: Vec<String>,
    pub base_conflict_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    pub file: String,
    pub tracks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairConflict {
    pub track_a: String,
    pub track_b: String,
    pub overlapping_files: Vec<String>,
    pub merge_tree_conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeWave {
    pub wave: usize,
    pub tracks: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeCheckReport {
    pub repo: String,
    pub base_ref: String,
    pub base_oid: String,
    pub candidates: Vec<CandidateReport>,
    pub hotspots: Vec<Hotspot>,
    pub pair_conflicts: Vec<PairConflict>,
    pub waves: Vec<MergeWave>,
    pub blocked_tracks: Vec<String>,
}

struct InternalTrack {
    report: CandidateReport,
    files_set: BTreeSet<String>,
    base_merged_tree: Option<String>,
}

enum MergeTreeOutcome {
    Clean(String),
    Conflict(Vec<String>),
    Error(String),
}

fn run_merge_tree(
    cwd: &Path,
    merge_base: Option<&str>,
    tree_or_commit_a: &str,
    commit_b: &str,
) -> MergeTreeOutcome {
    let mut args = vec![
        "merge-tree",
        "--write-tree",
        "--name-only",
        "--no-messages",
        "-z",
    ];
    let mb_arg;
    if let Some(mb) = merge_base {
        mb_arg = format!("--merge-base={mb}");
        args.push(&mb_arg);
    }
    args.push(tree_or_commit_a);
    args.push(commit_b);

    let Ok(out) = git_raw(cwd, &args) else {
        return MergeTreeOutcome::Error("无法启动 git merge-tree".to_string());
    };
    let mut tokens = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).to_string());

    match out.status.code() {
        Some(0) => match tokens.next() {
            Some(tree) => MergeTreeOutcome::Clean(tree),
            None => MergeTreeOutcome::Error("git merge-tree 未输出树 OID".to_string()),
        },
        Some(1) => {
            let _conflicted_tree = tokens.next();
            let files: Vec<String> = tokens.collect();
            MergeTreeOutcome::Conflict(files)
        }
        _ => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            MergeTreeOutcome::Error(stderr)
        }
    }
}

fn evaluate_local_candidate(
    cwd: &Path,
    id: String,
    label: String,
    pr_number: Option<u64>,
    is_draft: bool,
    base_oid: &str,
    head_oid: String,
) -> Result<InternalTrack, String> {
    let mb_out = git_raw(cwd, &["merge-base", base_oid, &head_oid])?;
    if !mb_out.status.success() {
        // Exit code 1 means unrelated/orphan histories (e.g. gh-pages); mark as BaseConflict instead of crashing batch
        if mb_out.status.code() == Some(1) {
            return Ok(InternalTrack {
                report: CandidateReport {
                    id,
                    label,
                    pr_number,
                    is_draft,
                    head_oid: Some(head_oid),
                    merge_base_oid: None,
                    base_sync: BaseSyncState::BaseConflict,
                    changed_files: Vec::new(),
                    base_overlap_files: Vec::new(),
                    base_conflict_files: vec!["<unrelated-histories>".to_string()],
                },
                files_set: BTreeSet::new(),
                base_merged_tree: None,
            });
        }
        let stderr = String::from_utf8_lossy(&mb_out.stderr).trim().to_string();
        return Err(format!("{id}: git merge-base 失败：{stderr}"));
    }
    let mb = String::from_utf8_lossy(&mb_out.stdout).trim().to_string();

    let track_range = format!("{mb}..{head_oid}");
    let files_set = diff_name_only_z(cwd, &track_range)?;

    let base_moved_set = if mb == base_oid {
        BTreeSet::new()
    } else {
        let base_range = format!("{mb}..{base_oid}");
        diff_name_only_z(cwd, &base_range)?
    };

    let base_overlap: Vec<String> = files_set.intersection(&base_moved_set).cloned().collect();

    let (base_sync, base_merged_tree, base_conflict_files) =
        match run_merge_tree(cwd, Some(&mb), base_oid, &head_oid) {
            MergeTreeOutcome::Clean(tree) => {
                let sync = if mb == base_oid || base_moved_set.is_empty() {
                    BaseSyncState::UpToDate
                } else if base_overlap.is_empty() {
                    BaseSyncState::SyncFree
                } else {
                    BaseSyncState::CleanWithOverlap
                };
                (sync, Some(tree), Vec::new())
            }
            MergeTreeOutcome::Conflict(conflicts) => (BaseSyncState::BaseConflict, None, conflicts),
            MergeTreeOutcome::Error(err) => {
                return Err(format!("{id}: git merge-tree 失败：{err}"))
            }
        };

    Ok(InternalTrack {
        report: CandidateReport {
            id,
            label,
            pr_number,
            is_draft,
            head_oid: Some(head_oid),
            merge_base_oid: Some(mb),
            base_sync,
            changed_files: files_set.iter().cloned().collect(),
            base_overlap_files: base_overlap,
            base_conflict_files,
        },
        files_set,
        base_merged_tree,
    })
}

pub fn collect(args: &MergeCheckArgs) -> Result<MergeCheckReport, String> {
    let start_dir = match &args.repo {
        Some(p) => canonical_or_raw(p),
        None => canonical_or_raw(&env::current_dir().map_err(|e| e.to_string())?),
    };
    let ctx = resolve_repo(&start_dir)?;
    let wts = list_worktrees(&ctx.current_worktree)?;
    let primary_path = wts
        .iter()
        .find(|w| !w.bare && !w.missing)
        .map(|w| w.path.clone())
        .unwrap_or_else(|| ctx.current_worktree.clone());

    let (base_ref, base_oid) = resolve_base_ref(&primary_path, args.base.as_deref())?;

    let mut tracks: Vec<InternalTrack> = Vec::new();
    let mut seen_ids = BTreeSet::new();

    if args.prs || !args.pr_numbers.is_empty() {
        let prs = query_open_prs(&primary_path)?;
        let wanted: BTreeSet<u64> = args.pr_numbers.iter().copied().collect();
        let mut found_numbers: BTreeSet<u64> = BTreeSet::new();
        for pr in prs {
            if !wanted.is_empty() && !wanted.contains(&pr.number) {
                continue;
            }
            found_numbers.insert(pr.number);
            let id = format!("PR#{}", pr.number);
            if !seen_ids.insert(id.clone()) {
                continue;
            }
            let label = format!("PR #{} ({}) {}", pr.number, pr.head_ref_name, pr.title);
            let local_oid = pr
                .head_ref_oid
                .as_deref()
                .and_then(|oid| commit_exists(&primary_path, oid));

            if let Some(head_oid) = local_oid {
                tracks.push(evaluate_local_candidate(
                    &primary_path,
                    id,
                    label,
                    Some(pr.number),
                    pr.is_draft,
                    &base_oid,
                    head_oid,
                )?);
            } else {
                let files_set: BTreeSet<String> = pr.files.into_iter().map(|f| f.path).collect();
                tracks.push(InternalTrack {
                    report: CandidateReport {
                        id,
                        label,
                        pr_number: Some(pr.number),
                        is_draft: pr.is_draft,
                        head_oid: pr.head_ref_oid,
                        merge_base_oid: None,
                        base_sync: BaseSyncState::UnfetchedRemote,
                        changed_files: files_set.iter().cloned().collect(),
                        base_overlap_files: Vec::new(),
                        base_conflict_files: Vec::new(),
                    },
                    files_set,
                    base_merged_tree: None,
                });
            }
        }
        let missing: Vec<String> = wanted
            .difference(&found_numbers)
            .map(|n| format!("#{n}"))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "未在开放 PR 列表中找到指定 PR：{}（可能已合并/关闭或编号有误）",
                missing.join(", ")
            ));
        }
    }

    for br in &args.branches {
        let trimmed = br.trim();
        if trimmed.is_empty() || !seen_ids.insert(trimmed.to_string()) {
            continue;
        }
        let head_oid = commit_exists(&primary_path, trimmed)
            .ok_or_else(|| format!("无法解析候选分支或引用 `{trimmed}`"))?;
        tracks.push(evaluate_local_candidate(
            &primary_path,
            trimmed.to_string(),
            trimmed.to_string(),
            None,
            false,
            &base_oid,
            head_oid,
        )?);
    }

    // Default: inspect all local worktrees currently ahead of base
    if tracks.is_empty() && !args.prs && args.pr_numbers.is_empty() && args.branches.is_empty() {
        let mut seen_oids = BTreeSet::new();
        for wt in wts.iter().filter(|w| !w.bare && !w.missing) {
            let Ok(mb) = git_stdout(&wt.path, &["merge-base", &base_oid, &wt.head]) else {
                continue;
            };
            if mb == wt.head {
                continue;
            }
            if !seen_oids.insert(wt.head.clone()) {
                continue;
            }
            let id = wt
                .branch
                .clone()
                .unwrap_or_else(|| short_oid(&wt.head).to_string());
            if !seen_ids.insert(id.clone()) {
                continue;
            }
            tracks.push(evaluate_local_candidate(
                &primary_path,
                id.clone(),
                id,
                None,
                false,
                &base_oid,
                wt.head.clone(),
            )?);
        }
    }

    // Compute hotspots
    let mut file_to_tracks: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in &tracks {
        for f in &t.files_set {
            file_to_tracks
                .entry(f.clone())
                .or_default()
                .push(t.report.id.clone());
        }
    }
    let mut hotspots: Vec<Hotspot> = file_to_tracks
        .into_iter()
        .filter(|(_, owners)| owners.len() >= 2)
        .map(|(file, owners)| Hotspot {
            file,
            tracks: owners,
        })
        .collect();
    hotspots.sort_by(|a, b| {
        b.tracks
            .len()
            .cmp(&a.tracks.len())
            .then_with(|| a.file.cmp(&b.file))
    });

    // Pairwise overlap & virtual merge-tree check
    let mut pair_conflicts = Vec::new();
    for i in 0..tracks.len() {
        for j in (i + 1)..tracks.len() {
            let a = &tracks[i];
            let b = &tracks[j];
            let overlap: Vec<String> = a.files_set.intersection(&b.files_set).cloned().collect();
            let mut mt_conflicts = Vec::new();
            if let (Some(ref tree_a), Some(ref b_oid), Some(ref mb_b)) = (
                &a.base_merged_tree,
                &b.report.head_oid,
                &b.report.merge_base_oid,
            ) {
                if a.report.base_sync != BaseSyncState::BaseConflict
                    && b.report.base_sync != BaseSyncState::BaseConflict
                {
                    match run_merge_tree(&primary_path, Some(mb_b), tree_a, b_oid) {
                        MergeTreeOutcome::Clean(_) => {}
                        MergeTreeOutcome::Conflict(c) => mt_conflicts = c,
                        MergeTreeOutcome::Error(err) => {
                            return Err(format!(
                                "{} × {} 内存合并树预演失败：{err}",
                                a.report.id, b.report.id
                            ));
                        }
                    }
                }
            }
            if !overlap.is_empty() || !mt_conflicts.is_empty() {
                pair_conflicts.push(PairConflict {
                    track_a: a.report.id.clone(),
                    track_b: b.report.id.clone(),
                    overlapping_files: overlap,
                    merge_tree_conflicts: mt_conflicts,
                });
            }
        }
    }

    // Wave planning using cumulative virtual merge-tree
    let mut waves = Vec::new();
    let mut blocked_tracks = Vec::new();
    let mut remaining: Vec<usize> = Vec::new();

    for (idx, t) in tracks.iter().enumerate() {
        match t.report.base_sync {
            BaseSyncState::BaseConflict | BaseSyncState::UnfetchedRemote => {
                blocked_tracks.push(t.report.id.clone());
            }
            _ => remaining.push(idx),
        }
    }

    let mut wave_num = 1;
    while !remaining.is_empty() {
        let mut current_tree = base_oid.clone();
        let mut wave_files: BTreeSet<String> = BTreeSet::new();
        let mut wave_ids = Vec::new();
        let mut next_remaining = Vec::new();

        for idx in remaining {
            let t = &tracks[idx];
            let has_file_overlap = t.files_set.iter().any(|f| wave_files.contains(f));
            if has_file_overlap {
                next_remaining.push(idx);
                continue;
            }
            let Some(ref head_oid) = t.report.head_oid else {
                blocked_tracks.push(t.report.id.clone());
                continue;
            };
            let Some(ref mb) = t.report.merge_base_oid else {
                blocked_tracks.push(t.report.id.clone());
                continue;
            };
            match run_merge_tree(&primary_path, Some(mb), &current_tree, head_oid) {
                MergeTreeOutcome::Clean(new_tree) => {
                    current_tree = new_tree;
                    wave_files.extend(t.files_set.iter().cloned());
                    wave_ids.push(t.report.id.clone());
                }
                MergeTreeOutcome::Conflict(_) => {
                    next_remaining.push(idx);
                }
                MergeTreeOutcome::Error(err) => {
                    return Err(format!("Wave 规划预演 {} 失败：{err}", t.report.id));
                }
            }
        }

        if wave_ids.is_empty() {
            for idx in next_remaining {
                blocked_tracks.push(tracks[idx].report.id.clone());
            }
            break;
        }

        let note = if wave_num == 1 {
            "零文件重叠且内存累计 merge-tree 全绿，可按任意顺序安全合入".to_string()
        } else {
            format!(
                "与前序波次存在文件重叠或三方冲突，建议在 Wave {} 合入后同步增量再合",
                wave_num - 1
            )
        };
        waves.push(MergeWave {
            wave: wave_num,
            tracks: wave_ids,
            note,
        });
        wave_num += 1;
        remaining = next_remaining;
    }

    Ok(MergeCheckReport {
        repo: primary_path.display().to_string(),
        base_ref,
        base_oid,
        candidates: tracks.into_iter().map(|t| t.report).collect(),
        hotspots,
        pair_conflicts,
        waves,
        blocked_tracks,
    })
}

pub fn format_human(r: &MergeCheckReport) -> String {
    let mut out = String::new();
    let short_base = short_oid(&r.base_oid);
    out.push_str(&format!(
        "Agent-On 合并树预演 · 基线 {} ({})\n",
        r.base_ref, short_base
    ));

    if r.candidates.is_empty() {
        out.push_str("当前没有待预演的在途分支或开放 PR。\n");
        return out;
    }

    out.push_str(&format!("\n候选轨道（{} 条）：\n", r.candidates.len()));
    for c in &r.candidates {
        let draft_tag = if c.is_draft { " [DRAFT]" } else { "" };
        let extra = match c.base_sync {
            BaseSyncState::UpToDate => "已在最新基线上".to_string(),
            BaseSyncState::SyncFree => "主线推进无文件重叠（可免冗余同步重跑）".to_string(),
            BaseSyncState::CleanWithOverlap => format!(
                "与主线推进重叠 {} 个文件但可干净自动合并",
                c.base_overlap_files.len()
            ),
            BaseSyncState::BaseConflict => {
                format!("与当前基线冲突：{}", c.base_conflict_files.join(", "))
            }
            BaseSyncState::UnfetchedRemote => {
                "本地尚未 fetch 该 PR 提交（仅做文件列表重叠分析，运行 `git fetch origin` 可启用 merge-tree）"
                    .to_string()
            }
        };
        out.push_str(&format!(
            "  - {:<24} [{:<18}]{draft_tag} 改动 {:>2} 文件 · {}\n",
            c.id,
            c.base_sync.label(),
            c.changed_files.len(),
            extra
        ));
    }

    if !r.hotspots.is_empty() {
        out.push_str("\n并发热点文件（≥2 条轨道修改）：\n");
        for h in &r.hotspots {
            out.push_str(&format!(
                "  ~ {} ({} 条轨道：{})\n",
                h.file,
                h.tracks.len(),
                h.tracks.join(", ")
            ));
        }
    }

    let real_pair_conflicts: Vec<_> = r
        .pair_conflicts
        .iter()
        .filter(|p| !p.merge_tree_conflicts.is_empty())
        .collect();
    if !real_pair_conflicts.is_empty() {
        out.push_str("\n两两合并真实冲突预警（单看各自可合，合完一条另一条即红）：\n");
        for p in real_pair_conflicts {
            out.push_str(&format!(
                "  ! {} × {} -> 冲突文件：{}\n",
                p.track_a,
                p.track_b,
                p.merge_tree_conflicts.join(", ")
            ));
        }
    }

    if !r.waves.is_empty() {
        out.push_str("\n推荐合入波次：\n");
        for w in &r.waves {
            out.push_str(&format!(
                "  Wave {}：{}\n          （{}）\n",
                w.wave,
                w.tracks.join(" + "),
                w.note
            ));
        }
    }
    if !r.blocked_tracks.is_empty() {
        out.push_str(&format!(
            "\n需先处理冲突或拉取远端对象：{}\n",
            r.blocked_tracks.join(", ")
        ));
    }

    out
}

pub fn run(args: &MergeCheckArgs) -> i32 {
    match collect(args) {
        Ok(report) => {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print!("{}", format_human(&report));
            }
            0
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            2
        }
    }
}
