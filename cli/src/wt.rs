//! Cross-client Git worktree radar & safe cleanup (`agent-on wt`).

use crate::git_util::{
    canonical_or_raw, classify_client, commit_exists, diff_name_only_z, dirty_paths, git_raw,
    git_stdout, list_worktrees, query_open_prs, resolve_base_ref, resolve_repo, short_oid,
};
use clap::Args;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct WtArgs {
    /// Target repository or worktree path (defaults to current directory)
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Base branch/ref to compare against (auto-detects origin/main, main, etc.)
    #[arg(long)]
    pub base: Option<String>,
    /// Cross-check modified files against open GitHub PRs via `gh`
    #[arg(long)]
    pub with_prs: bool,
    /// Safely remove clean, unlocked, already-merged linked worktrees (never uses --force)
    #[arg(long)]
    pub prune: bool,
    /// Output machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WtState {
    Main,
    CurrentClean,
    ActiveDirty,
    AheadClean,
    LikelySquashed,
    Reapable,
    Locked,
    Missing,
}

impl WtState {
    pub fn label(self) -> &'static str {
        match self {
            WtState::Main => "MAIN",
            WtState::CurrentClean => "CURRENT_CLEAN",
            WtState::ActiveDirty => "ACTIVE_DIRTY",
            WtState::AheadClean => "AHEAD_CLEAN",
            WtState::LikelySquashed => "LIKELY_SQUASHED",
            WtState::Reapable => "REAPABLE",
            WtState::Locked => "LOCKED",
            WtState::Missing => "MISSING",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorktreeEntry {
    pub path: String,
    pub client: String,
    pub branch: Option<String>,
    pub head: String,
    pub primary: bool,
    pub current: bool,
    pub locked: bool,
    pub lock_reason: Option<String>,
    pub state: WtState,
    pub ahead: u64,
    pub behind: u64,
    pub dirty_files: Vec<String>,
    pub committed_files: Vec<String>,
    pub last_commit_date: Option<String>,
    pub last_commit_subject: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileCollision {
    pub file: String,
    pub worktrees: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrOverlap {
    pub pr_number: u64,
    pub pr_title: String,
    pub pr_head_branch: String,
    pub pr_url: String,
    pub local_worktree: String,
    pub local_branch: Option<String>,
    pub overlapping_files: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WtSummary {
    pub total: usize,
    pub active_dirty: usize,
    pub ahead_clean: usize,
    pub likely_squashed: usize,
    pub reapable: usize,
    pub locked: usize,
    pub missing: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct WtReport {
    pub primary_worktree: String,
    pub current_worktree: String,
    pub base_ref: String,
    pub base_oid: String,
    pub current_behind_base: Option<u64>,
    pub current_ahead_base: Option<u64>,
    pub summary: WtSummary,
    pub uncommitted_collisions: Vec<FileCollision>,
    pub branch_overlaps: Vec<FileCollision>,
    pub pr_overlaps: Vec<PrOverlap>,
    pub pr_query_error: Option<String>,
    pub worktrees: Vec<WorktreeEntry>,
    pub pruned_worktrees: Vec<String>,
    pub prune_errors: Vec<String>,
}

fn ahead_behind(wt_path: &Path, base_oid: &str) -> (u64, u64) {
    let range = format!("{base_oid}...HEAD");
    let Ok(raw) = git_stdout(wt_path, &["rev-list", "--left-right", "--count", &range]) else {
        return (0, 0);
    };
    let mut parts = raw.split_whitespace();
    let behind = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let ahead = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (ahead, behind)
}

fn is_ancestor(wt_path: &Path, ancestor: &str, descendant: &str) -> bool {
    git_raw(
        wt_path,
        &["merge-base", "--is-ancestor", ancestor, descendant],
    )
    .map(|o| o.status.success())
    .unwrap_or(false)
}

fn all_commits_patch_equivalent(wt_path: &Path, base_oid: &str) -> bool {
    let Ok(out) = git_stdout(wt_path, &["cherry", base_oid, "HEAD"]) else {
        return false;
    };
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    !lines.is_empty() && lines.iter().all(|l| l.starts_with("- "))
}

fn last_commit_meta(wt_path: &Path) -> (Option<String>, Option<String>) {
    let Ok(raw) = git_stdout(wt_path, &["show", "-s", "--format=%cs%x09%s", "HEAD"]) else {
        return (None, None);
    };
    match raw.split_once('\t') {
        Some((date, subj)) => (Some(date.to_string()), Some(subj.to_string())),
        None => (None, None),
    }
}

fn wt_tag(entry: &WorktreeEntry) -> String {
    let name = entry
        .branch
        .as_deref()
        .unwrap_or_else(|| short_oid(&entry.head));
    format!("[{}] {} ({})", entry.client, name, entry.path)
}

pub fn collect_and_run(args: &WtArgs) -> Result<(WtReport, i32), String> {
    let caller_cwd = env::current_dir().ok().map(|p| canonical_or_raw(&p));
    let start_dir = match &args.repo {
        Some(p) => canonical_or_raw(p),
        None => caller_cwd
            .clone()
            .ok_or_else(|| "无法获取当前工作目录".to_string())?,
    };
    let ctx = resolve_repo(&start_dir)?;
    let raw_wts = list_worktrees(&ctx.current_worktree)?;
    let non_bare: Vec<_> = raw_wts.into_iter().filter(|w| !w.bare).collect();

    let primary_path = non_bare
        .iter()
        .find(|w| !w.missing)
        .map(|w| w.path.clone())
        .unwrap_or_else(|| ctx.current_worktree.clone());

    let (base_ref, base_oid) = resolve_base_ref(&primary_path, args.base.as_deref())?;

    let mut entries = Vec::new();
    let mut dirty_file_owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut touched_file_owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut summary = WtSummary::default();

    let mut current_behind_base = None;
    let mut current_ahead_base = None;

    for raw in &non_bare {
        // Protect both the target `--repo` worktree AND the caller's actual shell working directory
        let is_caller_cwd = caller_cwd
            .as_ref()
            .is_some_and(|c| c == &raw.path || c.starts_with(&raw.path));
        let is_current = !raw.missing && (raw.path == ctx.current_worktree || is_caller_cwd);
        let client = classify_client(&raw.path, raw.primary).to_string();

        if raw.missing || raw.prunable {
            summary.missing += 1;
            entries.push(WorktreeEntry {
                path: raw.path.display().to_string(),
                client,
                branch: raw.branch.clone(),
                head: raw.head.clone(),
                primary: raw.primary,
                current: false,
                locked: raw.locked,
                lock_reason: raw.lock_reason.clone(),
                state: WtState::Missing,
                ahead: 0,
                behind: 0,
                dirty_files: Vec::new(),
                committed_files: Vec::new(),
                last_commit_date: None,
                last_commit_subject: None,
            });
            continue;
        }

        // Always compare against immutable `base_oid` so `HEAD` fallback never collapses to `HEAD...HEAD`
        let dirty_set = dirty_paths(&raw.path)?;
        let (ahead, behind) = ahead_behind(&raw.path, &base_oid);
        let committed_set = if ahead > 0 {
            let range = format!("{base_oid}...HEAD");
            diff_name_only_z(&raw.path, &range).unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        let (last_date, last_subj) = last_commit_meta(&raw.path);

        if raw.path == ctx.current_worktree {
            current_ahead_base = Some(ahead);
            current_behind_base = Some(behind);
        }

        let state = if raw.primary {
            if !dirty_set.is_empty() {
                summary.active_dirty += 1;
            }
            WtState::Main
        } else if raw.locked {
            summary.locked += 1;
            WtState::Locked
        } else if !dirty_set.is_empty() {
            summary.active_dirty += 1;
            WtState::ActiveDirty
        } else if ahead == 0 && is_ancestor(&raw.path, "HEAD", &base_oid) {
            if is_current {
                WtState::CurrentClean
            } else {
                summary.reapable += 1;
                WtState::Reapable
            }
        } else if ahead > 0 && all_commits_patch_equivalent(&raw.path, &base_oid) {
            summary.likely_squashed += 1;
            WtState::LikelySquashed
        } else {
            summary.ahead_clean += 1;
            WtState::AheadClean
        };

        let entry = WorktreeEntry {
            path: raw.path.display().to_string(),
            client,
            branch: raw.branch.clone(),
            head: raw.head.clone(),
            primary: raw.primary,
            current: is_current,
            locked: raw.locked,
            lock_reason: raw.lock_reason.clone(),
            state,
            ahead,
            behind,
            dirty_files: dirty_set.iter().cloned().collect(),
            committed_files: committed_set.iter().cloned().collect(),
            last_commit_date: last_date,
            last_commit_subject: last_subj,
        };

        let tag = wt_tag(&entry);
        for f in &dirty_set {
            dirty_file_owners
                .entry(f.clone())
                .or_default()
                .push(tag.clone());
        }
        if !matches!(
            entry.state,
            WtState::Reapable | WtState::LikelySquashed | WtState::Missing
        ) {
            for f in dirty_set.union(&committed_set) {
                touched_file_owners
                    .entry(f.clone())
                    .or_default()
                    .push(tag.clone());
            }
        }

        entries.push(entry);
    }
    summary.total = entries.len();

    let uncommitted_collisions: Vec<FileCollision> = dirty_file_owners
        .into_iter()
        .filter(|(_, owners)| owners.len() >= 2)
        .map(|(file, worktrees)| FileCollision { file, worktrees })
        .collect();

    let uncommitted_files: BTreeSet<&str> = uncommitted_collisions
        .iter()
        .map(|c| c.file.as_str())
        .collect();

    let branch_overlaps: Vec<FileCollision> = touched_file_owners
        .into_iter()
        .filter(|(file, owners)| owners.len() >= 2 && !uncommitted_files.contains(file.as_str()))
        .map(|(file, worktrees)| FileCollision { file, worktrees })
        .collect();

    let mut pr_overlaps = Vec::new();
    let mut pr_query_error = None;
    if args.with_prs {
        match query_open_prs(&ctx.current_worktree) {
            Ok(prs) => {
                for pr in prs {
                    let pr_files: BTreeSet<&str> =
                        pr.files.iter().map(|f| f.path.as_str()).collect();
                    if pr_files.is_empty() {
                        continue;
                    }
                    for entry in &entries {
                        if matches!(
                            entry.state,
                            WtState::Reapable | WtState::LikelySquashed | WtState::Missing
                        ) {
                            continue;
                        }
                        if entry.branch.as_deref() == Some(pr.head_ref_name.as_str()) {
                            continue;
                        }
                        let mut overlap: Vec<String> = entry
                            .dirty_files
                            .iter()
                            .chain(entry.committed_files.iter())
                            .filter(|f| pr_files.contains(f.as_str()))
                            .cloned()
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect();
                        overlap.sort();
                        if !overlap.is_empty() {
                            pr_overlaps.push(PrOverlap {
                                pr_number: pr.number,
                                pr_title: pr.title.clone(),
                                pr_head_branch: pr.head_ref_name.clone(),
                                pr_url: pr.url.clone(),
                                local_worktree: entry.path.clone(),
                                local_branch: entry.branch.clone(),
                                overlapping_files: overlap,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                pr_query_error = Some(e);
            }
        }
    }

    let mut pruned_worktrees = Vec::new();
    let mut prune_errors = Vec::new();
    if args.prune {
        // Execute worktree operations from `ctx.current_worktree` (guaranteed alive & never Reapable)
        if summary.missing > 0 {
            if let Err(e) = git_stdout(&ctx.current_worktree, &["worktree", "prune"]) {
                prune_errors.push(format!("git worktree prune 失败：{e}"));
            }
        }
        for entry in &entries {
            if entry.state != WtState::Reapable {
                continue;
            }
            let wt_path = Path::new(&entry.path);
            let still_clean = dirty_paths(wt_path).map(|d| d.is_empty()).unwrap_or(false);
            let still_merged = commit_exists(wt_path, "HEAD")
                .is_some_and(|head| is_ancestor(wt_path, &head, &base_oid));
            if !still_clean || !still_merged {
                prune_errors.push(format!(
                    "跳过 {}：移除前复核状态已变化（不再是干净已合并状态）",
                    entry.path
                ));
                continue;
            }
            match git_raw(&ctx.current_worktree, &["worktree", "remove", &entry.path]) {
                Ok(out) if out.status.success() => {
                    pruned_worktrees.push(entry.path.clone());
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    prune_errors.push(format!("移除 {} 失败：{stderr}", entry.path));
                }
                Err(e) => {
                    prune_errors.push(format!("移除 {} 失败：{e}", entry.path));
                }
            }
        }
    }

    let exit_code = if !prune_errors.is_empty() { 1 } else { 0 };
    let report = WtReport {
        primary_worktree: primary_path.display().to_string(),
        current_worktree: ctx.current_worktree.display().to_string(),
        base_ref,
        base_oid,
        current_behind_base,
        current_ahead_base,
        summary,
        uncommitted_collisions,
        branch_overlaps,
        pr_overlaps,
        pr_query_error,
        worktrees: entries,
        pruned_worktrees,
        prune_errors,
    };
    Ok((report, exit_code))
}

pub fn format_human(r: &WtReport, prune_requested: bool) -> String {
    let mut out = String::new();
    let short_base = short_oid(&r.base_oid);
    out.push_str(&format!(
        "Agent-On WT 雷达 · 基线 {} ({})\n",
        r.base_ref, short_base
    ));
    out.push_str(&format!("当前检出：{}\n", r.current_worktree));
    if let (Some(ahead), Some(behind)) = (r.current_ahead_base, r.current_behind_base) {
        if behind > 0 {
            out.push_str(&format!(
                "  ⚠ 主线落差预警：当前分支落后 {} 共 {} 个提交（领先 {} 个）——打包/装机/提 PR 前请核对是否需同步主线\n",
                r.base_ref, behind, ahead
            ));
        } else {
            out.push_str(&format!(
                "  ✓ 主线对齐：落后 0 个提交，领先 {} 个提交\n",
                ahead
            ));
        }
    }

    out.push_str(&format!(
        "\n全景统计：共 {} 棵检出（活跃脏工作区 {} · 独有提交 {} · 疑似已 Squash {} · 可安全回收 {} · 加锁 {} · 缺失 {}）\n",
        r.summary.total,
        r.summary.active_dirty,
        r.summary.ahead_clean,
        r.summary.likely_squashed,
        r.summary.reapable,
        r.summary.locked,
        r.summary.missing,
    ));

    if !r.uncommitted_collisions.is_empty() {
        out.push_str("\n[高危 · 跨 Worktree 未提交文件撞车]\n");
        for c in &r.uncommitted_collisions {
            out.push_str(&format!("  ! {} -> {}\n", c.file, c.worktrees.join(" ↔ ")));
        }
    }
    if !r.branch_overlaps.is_empty() {
        out.push_str("\n[注意 · 在途分支改动文件重叠]\n");
        for c in &r.branch_overlaps {
            out.push_str(&format!("  ~ {} -> {}\n", c.file, c.worktrees.join(" ↔ ")));
        }
    }
    if let Some(ref err) = r.pr_query_error {
        out.push_str(&format!("\n[开放 PR 检查未完成] {err}\n"));
    } else if !r.pr_overlaps.is_empty() {
        out.push_str("\n[注意 · 本地工作树与开放 PR 文件重叠]\n");
        for p in &r.pr_overlaps {
            out.push_str(&format!(
                "  ~ PR #{} ({}) ↔ {}：{} 个文件（{}）\n",
                p.pr_number,
                p.pr_head_branch,
                p.local_branch.as_deref().unwrap_or(&p.local_worktree),
                p.overlapping_files.len(),
                p.overlapping_files.join(", ")
            ));
        }
    }

    out.push_str("\n检出清单：\n");
    for w in &r.worktrees {
        let cur = if w.current { "*" } else { " " };
        let br = w.branch.as_deref().unwrap_or_else(|| short_oid(&w.head));
        let mut detail = format!("+{}/-{}", w.ahead, w.behind);
        if !w.dirty_files.is_empty() {
            detail.push_str(&format!(" · 脏文件 {}", w.dirty_files.len()));
        }
        if matches!(w.state, WtState::AheadClean | WtState::LikelySquashed) {
            if let Some(ref subj) = w.last_commit_subject {
                let d = w.last_commit_date.as_deref().unwrap_or("");
                detail.push_str(&format!(" · {d} {subj}"));
            }
        }
        out.push_str(&format!(
            " {cur} [{:<11}] {:<15} {:<28} ({})  {}\n",
            w.client,
            w.state.label(),
            br,
            detail,
            w.path
        ));
    }

    if prune_requested {
        out.push('\n');
        if !r.pruned_worktrees.is_empty() {
            out.push_str(&format!(
                "已安全回收 {} 棵 REAPABLE 工作树（分支引用已保留）：\n",
                r.pruned_worktrees.len()
            ));
            for p in &r.pruned_worktrees {
                out.push_str(&format!("  - {p}\n"));
            }
        } else {
            out.push_str("没有需要自动回收的 REAPABLE 工作树。\n");
        }
        for err in &r.prune_errors {
            out.push_str(&format!("  ! {err}\n"));
        }
    } else if r.summary.reapable > 0 {
        out.push_str(&format!(
            "\n提示：有 {} 棵已合入主线且干净的工作树可一键安全回收：`agent-on wt --prune`\n",
            r.summary.reapable
        ));
    }

    out
}

pub fn run(args: &WtArgs) -> i32 {
    match collect_and_run(args) {
        Ok((report, code)) => {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print!("{}", format_human(&report, args.prune));
            }
            code
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            2
        }
    }
}
