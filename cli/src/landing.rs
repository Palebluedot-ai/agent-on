//! Landing control plane: merge-queue coordination + lifecycle classification.
//!
//! Design contract lives in `kit/landing-control-plane.md`. Invariants:
//! - the management unit is the feature track / PR, never the worktree path;
//! - every check result is bound to `(PR head SHA, base SHA)`; if neither
//!   moved the cached evidence is reused verbatim (SKIP);
//! - when base moves, only tracks with a dependency edge or file overlap are
//!   re-checked;
//! - v1 is strictly read-only + on-demand: no merges, no deletions, no daemon.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SNAPSHOT_VERSION: u8 = 1;

// ---------------------------------------------------------------------------
// Evidence model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CiState {
    Green,
    Red,
    Pending,
    None,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Required,
    None,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeState {
    Clean,
    Conflicting,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceState {
    Fresh,
    ReusedSame,
    ReusedValid,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Category {
    Now,
    Next,
    Parallel,
    Fix,
    Stale,
    Skip,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Now => "NOW",
            Category::Next => "NEXT",
            Category::Parallel => "PARALLEL",
            Category::Fix => "FIX",
            Category::Stale => "STALE",
            Category::Skip => "SKIP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Lifecycle {
    Active,
    Waiting,
    Parked,
    Rescue,
    Reapable,
}

impl Lifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lifecycle::Active => "ACTIVE",
            Lifecycle::Waiting => "WAITING",
            Lifecycle::Parked => "PARKED",
            Lifecycle::Rescue => "RESCUE",
            Lifecycle::Reapable => "REAPABLE",
        }
    }
}

// ---------------------------------------------------------------------------
// Incremental evidence decision (the SHA-bound cache contract)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDecision {
    /// No cached record: full check.
    CheckNew,
    /// PR head SHA moved: full check.
    CheckHeadMoved,
    /// A dependency landed into base since the last check: full check.
    CheckDepLanded,
    /// Base moved and its file movement could not be computed: full check.
    CheckConservative,
    /// Neither SHA moved: reuse the cached evidence verbatim.
    ReuseSame,
    /// Base moved without overlap: evidence stays valid, key advances.
    ReuseValid,
    /// Base moved over this track's files: needs rebase, no re-check yet.
    Invalidated,
}

#[derive(Debug, Clone)]
pub struct CachedEvidence {
    pub head_sha: String,
    pub base_sha: String,
    pub files: BTreeSet<String>,
    pub files_truncated: bool,
}

pub fn decide_evidence(
    cached: Option<&CachedEvidence>,
    probe_head: &str,
    current_base_sha: &str,
    movement_files: Option<&BTreeSet<String>>,
    dep_landed: bool,
) -> EvidenceDecision {
    let Some(cached) = cached else {
        return EvidenceDecision::CheckNew;
    };
    if cached.head_sha != probe_head {
        return EvidenceDecision::CheckHeadMoved;
    }
    if cached.base_sha == current_base_sha {
        return EvidenceDecision::ReuseSame;
    }
    if dep_landed {
        return EvidenceDecision::CheckDepLanded;
    }
    let Some(movement) = movement_files else {
        return EvidenceDecision::CheckConservative;
    };
    if cached.files_truncated || cached.files.iter().any(|f| movement.contains(f)) {
        return EvidenceDecision::Invalidated;
    }
    EvidenceDecision::ReuseValid
}

// ---------------------------------------------------------------------------
// Landing categories
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TrackFacts {
    /// Session owner label for FIX assignment: lane id, or `#<pr>` fallback.
    pub owner: String,
    pub pr_number: u64,
    pub ci: CiState,
    pub review: ReviewState,
    pub mergeable: MergeState,
    pub draft: bool,
    pub evidence: EvidenceState,
    /// Display labels of unmet dependencies, e.g. `#182` or a lane id.
    pub deps_unmet: Vec<String>,
    pub files: BTreeSet<String>,
    pub files_truncated: bool,
    /// Transitive dependent count inside the open track set (queue ranking).
    pub dependents: usize,
    pub prev_category: Option<Category>,
}

/// Categorized outcome for one track: display category, queue category
/// (identical except that SKIP keeps the underlying NEXT/PARALLEL for
/// planning), and the reason line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryOutcome {
    pub display: Category,
    pub queue: Category,
    pub reason: String,
}

/// Pre-queue verdict for one track, before NOW election.
enum BaseVerdict {
    Fix(String),
    Stale(String),
    Next(String),
    /// Fully green and dependency-satisfied: eligible to be elected NOW.
    NowCandidate,
    /// Healthy but not mergeable yet (pending CI/review, draft, unknown).
    ParallelCandidate(String),
}

fn base_verdict(t: &TrackFacts) -> BaseVerdict {
    if t.ci == CiState::Red {
        return BaseVerdict::Fix(format!("CI 红，分配给 {} 会话", t.owner));
    }
    if t.review == ReviewState::ChangesRequested {
        return BaseVerdict::Fix(format!("评审打回，分配给 {} 会话", t.owner));
    }
    if t.evidence == EvidenceState::Invalidated {
        return BaseVerdict::Stale("main 更新且文件重叠，需要 rebase".to_string());
    }
    if t.mergeable == MergeState::Conflicting {
        return BaseVerdict::Stale("与 main 冲突，需要 rebase".to_string());
    }
    if !t.deps_unmet.is_empty() {
        return BaseVerdict::Next(format!("等 {}", t.deps_unmet.join("、")));
    }
    if t.draft {
        return BaseVerdict::ParallelCandidate("draft，可并行开发".to_string());
    }
    let ci_ok = matches!(t.ci, CiState::Green | CiState::None);
    let review_ok = matches!(t.review, ReviewState::Approved | ReviewState::None);
    if ci_ok && review_ok && t.mergeable == MergeState::Clean {
        return BaseVerdict::NowCandidate;
    }
    let reason = match (t.ci, t.review, t.mergeable) {
        (CiState::Pending, _, _) => "CI 运行中，可并行验证",
        (CiState::Unknown, _, _) => "CI 状态未知，可并行准备",
        (_, ReviewState::Required, _) | (_, ReviewState::Unknown, _) => "评审未到，可并行准备",
        (_, _, MergeState::Unknown) => "mergeable 未知，等平台计算",
        _ => "与当前变更无重叠，证据仍有效",
    };
    BaseVerdict::ParallelCandidate(reason.to_string())
}

fn files_overlap(a: &TrackFacts, b: &TrackFacts) -> bool {
    a.files_truncated || b.files_truncated || a.files.iter().any(|f| b.files.contains(f))
}

pub fn categorize(tracks: &[TrackFacts]) -> Vec<CategoryOutcome> {
    let verdicts: Vec<BaseVerdict> = tracks.iter().map(base_verdict).collect();
    // Elect NOW: merges are strictly serial, so exactly one candidate wins.
    // Ranking: more transitive dependents first, then the lower PR number.
    let now_index = verdicts
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(v, BaseVerdict::NowCandidate))
        .map(|(i, _)| i)
        .min_by_key(|&i| (std::cmp::Reverse(tracks[i].dependents), tracks[i].pr_number));
    let mut out = Vec::with_capacity(tracks.len());
    for (i, verdict) in verdicts.iter().enumerate() {
        let (queue, reason) = match verdict {
            BaseVerdict::Fix(r) => (Category::Fix, r.clone()),
            BaseVerdict::Stale(r) => (Category::Stale, r.clone()),
            BaseVerdict::Next(r) => (Category::Next, r.clone()),
            BaseVerdict::NowCandidate => {
                if now_index == Some(i) {
                    (Category::Now, "全绿，依赖根节点，可合".to_string())
                } else {
                    let now = now_index.expect("a NOW candidate exists");
                    if files_overlap(&tracks[i], &tracks[now]) {
                        (
                            Category::Next,
                            format!(
                                "全绿，与 #{} 重叠，等 #{} 落地",
                                tracks[now].pr_number, tracks[now].pr_number
                            ),
                        )
                    } else {
                        (
                            Category::Parallel,
                            "与当前变更无重叠，证据仍有效".to_string(),
                        )
                    }
                }
            }
            BaseVerdict::ParallelCandidate(r) => match now_index {
                Some(now) if files_overlap(&tracks[i], &tracks[now]) => (
                    Category::Next,
                    format!("与 #{} 文件重叠，等其落地", tracks[now].pr_number),
                ),
                _ => (Category::Parallel, r.clone()),
            },
        };
        // SKIP only replaces non-actionable rows whose evidence and category
        // are both unchanged; NOW/FIX/STALE always surface.
        let display = if tracks[i].evidence == EvidenceState::ReusedSame
            && matches!(queue, Category::Next | Category::Parallel)
            && tracks[i].prev_category == Some(queue)
        {
            Category::Skip
        } else {
            queue
        };
        let reason = if display == Category::Skip {
            "SHA 未变化，不重复检查".to_string()
        } else {
            reason
        };
        out.push(CategoryOutcome {
            display,
            queue,
            reason,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Lifecycle classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct LifecycleFacts {
    /// active | blocked | ready | landed | parked, when a lane is registered.
    pub lane_status: Option<String>,
    pub primary: bool,
    /// Worktree directory exists on disk. Tracks without a worktree use false.
    pub present: bool,
    pub dirty: Option<bool>,
    /// Commits ahead of upstream; None when there is no upstream.
    pub unpushed: Option<u64>,
    /// Commits not reachable from base.
    pub unique: Option<u64>,
    /// Merged-PR headRefOid covers HEAD, or HEAD is an ancestor of base.
    pub integrated: Option<bool>,
    pub pr_open: bool,
    pub quiet_passed: Option<bool>,
}

pub fn classify_lifecycle(facts: &LifecycleFacts) -> (Lifecycle, String) {
    let lane = facts.lane_status.as_deref();
    let dirty = facts.dirty == Some(true);
    let unpushed = facts.unpushed.unwrap_or(0) > 0;
    // Orphan: unique commits with no upstream to preserve them and no proof
    // that base or a merged PR already covers them.
    let orphan =
        facts.unpushed.is_none() && facts.unique.unwrap_or(0) > 0 && facts.integrated != Some(true);

    if facts.integrated == Some(true) {
        if facts.present && (dirty || unpushed) {
            return (
                Lifecycle::Rescue,
                "已合流但有未提交/未推送残留，先抢救再回收".to_string(),
            );
        }
        if facts.primary {
            return (Lifecycle::Waiting, "控制轨，永不回收".to_string());
        }
        if !facts.present {
            return (
                Lifecycle::Waiting,
                "已合流，worktree 已移除；用 worktree forget 清理 lane 记录".to_string(),
            );
        }
        return match facts.quiet_passed {
            Some(true) => (
                Lifecycle::Reapable,
                "已合流、干净、静默期已过；仅报告，删除走 gc + 人工".to_string(),
            ),
            Some(false) => (Lifecycle::Waiting, "已合流，静默期未满".to_string()),
            None => (
                Lifecycle::Waiting,
                "已合流，静默期未知，保守等待".to_string(),
            ),
        };
    }

    if lane == Some("active") {
        return (Lifecycle::Active, "开发中，计入活跃上限".to_string());
    }

    if facts.present && (dirty || unpushed || orphan) {
        let mut what = Vec::new();
        if dirty {
            what.push("未提交改动");
        }
        if unpushed {
            what.push("未推送 commit");
        }
        if orphan {
            what.push("无远端保底的独有 commit");
        }
        return (
            Lifecycle::Rescue,
            format!("{}，绝不能回收", what.join(" + ")),
        );
    }

    if lane == Some("parked") {
        return (Lifecycle::Parked, "上下文已保存，可安全暂停".to_string());
    }

    let reason = if facts.primary {
        "控制轨待命".to_string()
    } else {
        match lane {
            Some("blocked") => "等依赖 / 解阻".to_string(),
            Some("ready") => "等控制轨合流".to_string(),
            Some("landed") => "已标 landed，等权威合流证据".to_string(),
            _ if facts.pr_open => "等 CI / 评审 / 合流".to_string(),
            _ => "无待办证据，等待盘点".to_string(),
        }
    };
    (Lifecycle::Waiting, reason)
}

// ---------------------------------------------------------------------------
// Rendering + planning
// ---------------------------------------------------------------------------

pub fn render_row(category: Category, pr_number: u64, reason: &str) -> String {
    format!(
        "{:<10}{:<6}{}",
        category.as_str(),
        format!("#{pr_number}"),
        reason
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCounts {
    pub active: usize,
    pub parked: usize,
    pub waiting: usize,
    pub rescue: usize,
    pub reapable: usize,
}

pub fn render_summary(counts: &SummaryCounts) -> String {
    format!(
        "现在做：{} 条\n下一批：{} 条\n等待中：{} 条\n需抢救：{} 条\n可回收：{} 条\n",
        counts.active, counts.parked, counts.waiting, counts.rescue, counts.reapable
    )
}

/// Minimal per-track view used by wave planning.
#[derive(Debug, Clone)]
pub struct PlanTrack {
    pub pr_number: u64,
    pub owner: String,
    pub queue: Category,
    /// PR numbers (within the open set) this track waits for.
    pub deps: Vec<u64>,
}

pub fn plan_waves(tracks: &[PlanTrack]) -> Vec<String> {
    let mut lines = Vec::new();
    let now: Vec<&PlanTrack> = tracks.iter().filter(|t| t.queue == Category::Now).collect();
    let parallel: Vec<&PlanTrack> = tracks
        .iter()
        .filter(|t| t.queue == Category::Parallel)
        .collect();
    let fmt_list = |items: &[&PlanTrack]| {
        items
            .iter()
            .map(|t| format!("#{}", t.pr_number))
            .collect::<Vec<_>>()
            .join("、")
    };
    let mut wave1 = String::from("WAVE 1  ");
    if now.is_empty() {
        wave1.push_str("合流：无可合轨道");
    } else {
        wave1.push_str(&format!("合流：{}", fmt_list(&now)));
    }
    if !parallel.is_empty() {
        wave1.push_str(&format!("；并行准备：{}", fmt_list(&parallel)));
    }
    lines.push(wave1);

    // Layer NEXT tracks topologically: a track enters wave k+1 once all its
    // in-set dependencies sit in earlier waves (merged deps are outside the
    // open set and count as satisfied).
    let open: BTreeSet<u64> = tracks.iter().map(|t| t.pr_number).collect();
    let mut placed: BTreeSet<u64> = tracks
        .iter()
        .filter(|t| t.queue != Category::Next)
        .map(|t| t.pr_number)
        .collect();
    let mut pending: Vec<&PlanTrack> = tracks
        .iter()
        .filter(|t| t.queue == Category::Next)
        .collect();
    let mut wave = 2usize;
    while !pending.is_empty() {
        let (layer, rest): (Vec<&PlanTrack>, Vec<&PlanTrack>) = pending.iter().partition(|t| {
            t.deps
                .iter()
                .all(|d| !open.contains(d) || placed.contains(d))
        });
        if layer.is_empty() {
            lines.push(format!(
                "待定    {}（依赖成环或不在开放集合内）",
                fmt_list(&rest)
            ));
            break;
        }
        let described = layer
            .iter()
            .map(|t| {
                let deps: Vec<String> = t
                    .deps
                    .iter()
                    .filter(|d| open.contains(d))
                    .map(|d| format!("#{d}"))
                    .collect();
                if deps.is_empty() {
                    format!("#{}", t.pr_number)
                } else {
                    format!("#{}（等 {}）", t.pr_number, deps.join("、"))
                }
            })
            .collect::<Vec<_>>()
            .join("；");
        lines.push(format!("WAVE {wave}  {described}"));
        for t in &layer {
            placed.insert(t.pr_number);
        }
        pending = rest;
        wave += 1;
    }

    let fixes: Vec<String> = tracks
        .iter()
        .filter_map(|t| match t.queue {
            Category::Fix => Some(format!("#{} → {} 会话修复", t.pr_number, t.owner)),
            Category::Stale => Some(format!("#{} → rebase 后重新取证", t.pr_number)),
            _ => None,
        })
        .collect();
    if !fixes.is_empty() {
        lines.push(format!("前置修复  {}", fixes.join("；")));
    }
    lines
}

// ---------------------------------------------------------------------------
// Snapshot persistence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub id: String,
    pub lane_id: Option<String>,
    pub pr_number: Option<u64>,
    pub branch: Option<String>,
    pub url: Option<String>,
    pub checked_head_sha: Option<String>,
    pub checked_base_sha: Option<String>,
    pub checked_at: Option<String>,
    pub evidence: Option<EvidenceState>,
    pub ci: Option<CiState>,
    pub review: Option<ReviewState>,
    pub mergeable: Option<MergeState>,
    pub draft: Option<bool>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub files_truncated: bool,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub owns: Vec<String>,
    pub category: Option<Category>,
    pub queue_category: Option<Category>,
    pub reason: Option<String>,
    /// open | merged | closed, when a PR is known for this track.
    pub pr_state: Option<String>,
    pub integrated: Option<bool>,
    /// PR numbers of in-flight dependencies (edges for wave planning).
    #[serde(default)]
    pub deps_open_prs: Vec<u64>,
    #[serde(default)]
    pub primary: bool,
    pub worktree: Option<String>,
    pub lifecycle: Lifecycle,
    pub lifecycle_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u8,
    pub generated_at: String,
    pub base_branch: String,
    pub base_sha: String,
    pub base_sha_source: String,
    pub tracks: Vec<TrackState>,
    #[serde(default)]
    pub notes: Vec<String>,
}

pub fn snapshot_path(common_git_dir: &std::path::Path) -> std::path::PathBuf {
    common_git_dir
        .join("agent-on")
        .join("landing")
        .join("snapshot.json")
}

pub fn load_snapshot(common_git_dir: &std::path::Path) -> Result<Option<Snapshot>, String> {
    let path = snapshot_path(common_git_dir);
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let snapshot: Snapshot =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if snapshot.version != SNAPSHOT_VERSION {
        return Err(format!(
            "unsupported landing snapshot version {} in {} (expected {}); delete it and refresh",
            snapshot.version,
            path.display(),
            SNAPSHOT_VERSION
        ));
    }
    Ok(Some(snapshot))
}

pub fn save_snapshot(common_git_dir: &std::path::Path, snapshot: &Snapshot) -> Result<(), String> {
    let path = snapshot_path(common_git_dir);
    let parent = path
        .parent()
        .ok_or_else(|| "invalid snapshot path".to_string())?;
    std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    let raw = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("serialize landing snapshot: {e}"))?;
    // Atomic replace so a concurrent reader never sees a torn file.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{raw}\n"))
        .map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Evidence sources (network side, injectable for tests)
// ---------------------------------------------------------------------------

use crate::worktree as wt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PrProbe {
    pub number: u64,
    pub branch: String,
    pub head_sha: String,
    pub base_ref: String,
    pub draft: bool,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct PrDetail {
    pub ci: CiState,
    pub review: ReviewState,
    pub mergeable: MergeState,
    pub files: Vec<String>,
    pub files_truncated: bool,
}

pub trait GhClient {
    /// One cheap batched read of every open PR: number, branch, head SHA.
    fn probe_open_prs(&self, root: &Path) -> Result<Vec<PrProbe>, String>;
    /// Expensive per-PR evidence; only called for invalidated tracks.
    fn pr_detail(&self, root: &Path, number: u64) -> Result<PrDetail, String>;
    /// Terminal state for a PR that left the open set:
    /// ("merged"|"closed"|"open", headRefOid).
    fn pr_state(&self, root: &Path, number: u64) -> Result<(String, Option<String>), String>;
    /// Changed file paths between two base commits, when computable remotely.
    fn compare_files(&self, root: &Path, old: &str, new: &str) -> Option<Vec<String>>;
}

pub struct RealGh;

fn run_gh(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("gh")
        .current_dir(root)
        .env("GH_PROMPT_DISABLED", "1")
        .args(args)
        .output()
        .map_err(|e| format!("cannot run gh: {e}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("gh {} failed", args.join(" "))
        } else {
            stderr
        })
    }
}

/// Fold a GitHub statusCheckRollup array into one CI verdict.
pub fn ci_from_rollup(rollup: &serde_json::Value) -> CiState {
    let Some(items) = rollup.as_array() else {
        return CiState::None;
    };
    if items.is_empty() {
        return CiState::None;
    }
    let mut pending = false;
    for item in items {
        let conclusion = item
            .get("conclusion")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let state = item.get("state").and_then(|v| v.as_str()).unwrap_or("");
        if matches!(
            conclusion,
            "FAILURE" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED" | "STARTUP_FAILURE"
        ) || matches!(state, "FAILURE" | "ERROR")
        {
            return CiState::Red;
        }
        let check_pending = !status.is_empty() && status != "COMPLETED";
        if check_pending || matches!(state, "PENDING" | "EXPECTED") {
            pending = true;
        }
    }
    if pending {
        CiState::Pending
    } else {
        CiState::Green
    }
}

fn review_from_str(value: Option<&str>) -> ReviewState {
    match value.unwrap_or("") {
        "APPROVED" => ReviewState::Approved,
        "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
        "REVIEW_REQUIRED" => ReviewState::Required,
        "" => ReviewState::None,
        _ => ReviewState::Unknown,
    }
}

fn merge_from_str(value: Option<&str>) -> MergeState {
    match value.unwrap_or("") {
        "MERGEABLE" => MergeState::Clean,
        "CONFLICTING" => MergeState::Conflicting,
        _ => MergeState::Unknown,
    }
}

impl GhClient for RealGh {
    fn probe_open_prs(&self, root: &Path) -> Result<Vec<PrProbe>, String> {
        let raw = run_gh(
            root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--limit",
                "200",
                "--json",
                "number,headRefName,baseRefName,headRefOid,isDraft,url",
            ],
        )?;
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(&raw).map_err(|e| format!("cannot parse gh pr list: {e}"))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(PrProbe {
                number: row
                    .get("number")
                    .and_then(|v| v.as_u64())
                    .ok_or("gh pr list row without number")?,
                branch: row
                    .get("headRefName")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                head_sha: row
                    .get("headRefOid")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                base_ref: row
                    .get("baseRefName")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                draft: row
                    .get("isDraft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                url: row
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            });
        }
        Ok(out)
    }

    fn pr_detail(&self, root: &Path, number: u64) -> Result<PrDetail, String> {
        let raw = run_gh(
            root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "statusCheckRollup,reviewDecision,mergeable,files",
            ],
        )?;
        let value: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|e| format!("cannot parse gh pr view: {e}"))?;
        let files: Vec<String> = value
            .get("files")
            .and_then(|v| v.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|r| r.get("path").and_then(|p| p.as_str()))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        // gh caps the files list at 100 entries; treat a full page as truncated.
        let files_truncated = files.len() >= 100;
        Ok(PrDetail {
            ci: ci_from_rollup(
                value
                    .get("statusCheckRollup")
                    .unwrap_or(&serde_json::Value::Null),
            ),
            review: review_from_str(value.get("reviewDecision").and_then(|v| v.as_str())),
            mergeable: merge_from_str(value.get("mergeable").and_then(|v| v.as_str())),
            files,
            files_truncated,
        })
    }

    fn pr_state(&self, root: &Path, number: u64) -> Result<(String, Option<String>), String> {
        let raw = run_gh(
            root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "state,mergedAt,headRefOid",
            ],
        )?;
        let value: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|e| format!("cannot parse gh pr view: {e}"))?;
        let oid = value
            .get("headRefOid")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let merged = value.get("mergedAt").map(|v| !v.is_null()).unwrap_or(false)
            || value.get("state").and_then(|v| v.as_str()) == Some("MERGED");
        let state = if merged {
            "merged"
        } else {
            match value.get("state").and_then(|v| v.as_str()) {
                Some("OPEN") => "open",
                Some("CLOSED") => "closed",
                _ => "unknown",
            }
        };
        Ok((state.to_string(), oid))
    }

    fn compare_files(&self, root: &Path, old: &str, new: &str) -> Option<Vec<String>> {
        let raw = run_gh(root, &["repo", "view", "--json", "nameWithOwner"]).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&raw).ok()?;
        let nwo = value.get("nameWithOwner").and_then(|v| v.as_str())?;
        let raw = run_gh(
            root,
            &["api", &format!("repos/{nwo}/compare/{old}...{new}")],
        )
        .ok()?;
        let value: serde_json::Value = serde_json::from_slice(&raw).ok()?;
        let files: Vec<String> = value
            .get("files")?
            .as_array()?
            .iter()
            .filter_map(|f| f.get("filename").and_then(|v| v.as_str()))
            .map(str::to_string)
            .collect();
        // The compare API lists at most 300 files; a full page may be truncated.
        if files.len() >= 300 {
            return None;
        }
        Some(files)
    }
}

// ---------------------------------------------------------------------------
// Commands: refresh / status / plan
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LandingOpts {
    pub json: bool,
    pub base: Option<String>,
    pub quiet_hours: u64,
}

impl Default for LandingOpts {
    fn default() -> Self {
        LandingOpts {
            json: false,
            base: None,
            quiet_hours: 24,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
struct RefreshStats {
    checked: usize,
    reused_same: usize,
    reused_valid: usize,
    invalidated: usize,
    newly_merged: usize,
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

fn canonical_string(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn resolve_base(root: &Path, base_override: Option<&str>) -> Result<(String, String), String> {
    let base_ref = match base_override {
        Some(v) => v.to_string(),
        None => wt::default_base(root)?,
    };
    let base_branch = wt::hosted_base_name(root, &base_ref)
        .or_else(|| base_ref.rsplit('/').next().map(str::to_string))
        .ok_or_else(|| format!("cannot determine hosted branch for base {base_ref}"))?;
    Ok((base_ref, base_branch))
}

fn resolve_base_sha(
    root: &Path,
    base_ref: &str,
    base_branch: &str,
) -> Result<(String, String), String> {
    if let Ok(out) = wt::git(
        root,
        &["ls-remote", "origin", &format!("refs/heads/{base_branch}")],
    ) {
        if let Some(sha) = out.split_whitespace().next() {
            if !sha.is_empty() {
                return Ok((sha.to_string(), "ls-remote".to_string()));
            }
        }
    }
    let sha = wt::git(root, &["rev-parse", "--verify", base_ref])
        .map_err(|e| format!("cannot resolve base {base_ref}: {e}"))?;
    Ok((sha, "local".to_string()))
}

/// Files touched by base moving from `old` to `new`; None when uncomputable.
fn movement_between(
    root: &Path,
    gh: &dyn GhClient,
    memo: &mut BTreeMap<String, Option<BTreeSet<String>>>,
    old: &str,
    new: &str,
) -> Option<BTreeSet<String>> {
    if old == new {
        return Some(BTreeSet::new());
    }
    if let Some(cached) = memo.get(old) {
        return cached.clone();
    }
    let have = |sha: &str| {
        wt::git(
            root,
            &["rev-parse", "--verify", &format!("{sha}^{{commit}}")],
        )
        .is_ok()
    };
    let result = if have(old) && have(new) {
        wt::git(root, &["diff", "--name-only", &format!("{old}..{new}")])
            .ok()
            .map(|out| out.lines().map(str::to_string).collect::<BTreeSet<_>>())
    } else {
        gh.compare_files(root, old, new)
            .map(|v| v.into_iter().collect())
    };
    memo.insert(old.to_string(), result.clone());
    result
}

/// Local (non-network) git facts + lifecycle verdict for one track.
#[allow(clippy::too_many_arguments)]
fn local_lifecycle(
    base_ref: &str,
    lane_status: Option<&str>,
    wt_path: Option<&Path>,
    primary: bool,
    merged_oid: Option<&str>,
    pr_open: bool,
    quiet_hours: u64,
) -> (Lifecycle, String, Option<bool>) {
    let present = wt_path.map(|p| p.exists()).unwrap_or(false);
    let (dirty, unpushed, unique, anc) = if present {
        let p = wt_path.expect("present implies path");
        let dirty = wt::git(p, &["status", "--porcelain"])
            .ok()
            .map(|v| !v.is_empty());
        let has_upstream = wt::git(
            p,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        )
        .is_ok();
        let unpushed = if has_upstream {
            wt::count_revs(p, "@{u}..HEAD")
        } else {
            None
        };
        let unique = wt::count_revs(p, &format!("{base_ref}..HEAD"));
        let anc = wt::ancestor_result(p, "HEAD", base_ref).ok();
        (dirty, unpushed, unique, anc)
    } else {
        (None, None, None, None)
    };
    let mut integrated = anc;
    if integrated != Some(true) {
        if let Some(oid) = merged_oid {
            let covered = present && {
                let p = wt_path.expect("present implies path");
                wt::git(p, &["rev-parse", "HEAD"]).ok().as_deref() == Some(oid)
                    || wt::ancestor_result(p, "HEAD", oid) == Ok(true)
            };
            // A merged PR whose coverage cannot be proven is unknown, not "no":
            // squash merges break ancestry, so never report false here.
            integrated = if covered { Some(true) } else { None };
        }
    }
    let quiet_passed = if integrated == Some(true) && present && dirty == Some(false) && !primary {
        if quiet_hours == 0 {
            Some(true)
        } else {
            match wt::scan_worktree(wt_path.expect("present implies path"), true) {
                Ok((_, latest)) => {
                    let hours = std::time::SystemTime::now()
                        .duration_since(latest)
                        .unwrap_or_default()
                        .as_secs_f64()
                        / 3600.0;
                    Some(hours >= quiet_hours as f64)
                }
                Err(_) => None,
            }
        }
    } else {
        None
    };
    let facts = LifecycleFacts {
        lane_status: lane_status.map(str::to_string),
        primary,
        present,
        dirty,
        unpushed,
        unique,
        integrated,
        pr_open,
        quiet_passed,
    };
    let (class, reason) = classify_lifecycle(&facts);
    (class, reason, integrated)
}

fn summary_counts(tracks: &[TrackState]) -> SummaryCounts {
    let count = |c: Lifecycle| tracks.iter().filter(|t| t.lifecycle == c).count();
    SummaryCounts {
        active: count(Lifecycle::Active),
        parked: count(Lifecycle::Parked),
        waiting: count(Lifecycle::Waiting),
        rescue: count(Lifecycle::Rescue),
        reapable: count(Lifecycle::Reapable),
    }
}

fn category_rank(category: Category) -> usize {
    match category {
        Category::Now => 0,
        Category::Next => 1,
        Category::Parallel => 2,
        Category::Fix => 3,
        Category::Stale => 4,
        Category::Skip => 5,
    }
}

fn render_table(tracks: &[TrackState]) -> String {
    let mut rows: Vec<&TrackState> = tracks
        .iter()
        .filter(|t| t.category.is_some() && t.pr_number.is_some())
        .collect();
    if rows.is_empty() {
        return "（合流表为空：没有开放 PR 轨道）\n".to_string();
    }
    rows.sort_by_key(|t| {
        (
            category_rank(t.category.expect("filtered")),
            t.pr_number.expect("filtered"),
        )
    });
    let mut out = String::new();
    for row in rows {
        out.push_str(&render_row(
            row.category.expect("filtered"),
            row.pr_number.expect("filtered"),
            row.reason.as_deref().unwrap_or(""),
        ));
        out.push('\n');
    }
    out
}

/// Pad to `width`, but always keep at least two spaces before the next column
/// even when the content overflows the column.
fn col(text: &str, width: usize) -> String {
    if text.chars().count() >= width.saturating_sub(2) {
        format!("{text}  ")
    } else {
        format!("{text:<width$}")
    }
}

fn render_lifecycle(tracks: &[TrackState]) -> String {
    let mut out = String::from("WORKTREES / TRACKS:\n");
    let mut rows: Vec<&TrackState> = tracks.iter().collect();
    rows.sort_by_key(|t| (t.lifecycle, t.id.clone()));
    for t in rows {
        let mut reason = t.lifecycle_reason.clone();
        if t.primary {
            reason = format!("控制轨 primary；{reason}");
        }
        out.push_str(&format!(
            "{}{}{}{}\n",
            col(t.lifecycle.as_str(), 10),
            col(&t.id, 18),
            col(t.branch.as_deref().unwrap_or("-"), 26),
            reason
        ));
    }
    out
}

fn render_body(snapshot: &Snapshot, active_lanes: usize, cap: u64) -> String {
    let counts = summary_counts(&snapshot.tracks);
    let mut out = String::new();
    for note in &snapshot.notes {
        out.push_str(&format!("NOTE: {note}\n"));
    }
    out.push('\n');
    out.push_str(&render_summary(&counts));
    out.push('\n');
    out.push_str(&render_table(&snapshot.tracks));
    out.push('\n');
    out.push_str(&render_lifecycle(&snapshot.tracks));
    out.push_str(&format!("active lanes: {active_lanes}/{cap}"));
    if active_lanes as u64 > cap {
        out.push_str("  ⚠ 超过活跃轨上限，先 park 一部分再开新轨");
    }
    out.push('\n');
    out.push_str(
        "READ-ONLY: 只写本机快照缓存（common git dir 的 agent-on/landing/snapshot.json），未改任何 worktree / 分支 / PR。\n",
    );
    out
}

fn snapshot_json(snapshot: &Snapshot, active_lanes: usize, cap: u64) -> String {
    let counts = summary_counts(&snapshot.tracks);
    let value = serde_json::json!({
        "snapshot": snapshot,
        "summary": counts,
        "active_lanes": active_lanes,
        "active_cap": cap,
    });
    serde_json::to_string_pretty(&value)
        .map(|v| format!("{v}\n"))
        .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}\n"))
}

struct RefreshBuild {
    common: PathBuf,
    root: PathBuf,
    snapshot: Snapshot,
    stats: RefreshStats,
    active_lanes: usize,
    cap: u64,
}

fn build_refresh(
    repo: &Path,
    gh: &dyn GhClient,
    opts: &LandingOpts,
) -> Result<RefreshBuild, String> {
    let root = wt::repo_root(repo)?;
    let common = wt::common_git_dir(&root)?;
    let (base_ref, base_branch) = resolve_base(&root, opts.base.as_deref())?;
    let (base_sha, base_sha_source) = resolve_base_sha(&root, &base_ref, &base_branch)?;
    let prev = load_snapshot(&common)?;
    let lanes = wt::load_records(&root)?;
    let worktrees = wt::parse_worktrees(&wt::git(&root, &["worktree", "list", "--porcelain"])?);
    let now_iso = chrono::Utc::now().to_rfc3339();

    let primary_path = worktrees.first().map(|w| canonical_string(&w.path));
    let wt_by_path: BTreeMap<String, &wt::WorktreeInfo> = worktrees
        .iter()
        .map(|w| (canonical_string(&w.path), w))
        .collect();
    let wt_by_branch: BTreeMap<String, String> = worktrees
        .iter()
        .filter_map(|w| w.branch.clone().map(|b| (b, canonical_string(&w.path))))
        .collect();
    let lane_by_branch: BTreeMap<String, &wt::LaneRecord> =
        lanes.iter().map(|l| (l.branch.clone(), l)).collect();
    let lane_by_id: BTreeMap<String, &wt::LaneRecord> =
        lanes.iter().map(|l| (l.id.clone(), l)).collect();

    let mut notes = Vec::new();
    let mut stats = RefreshStats::default();

    // 1) One cheap batched probe of every open PR.
    let all_probes = gh.probe_open_prs(&root)?;
    let mut probes = Vec::new();
    for p in all_probes {
        if p.base_ref == base_branch {
            probes.push(p);
        } else {
            notes.push(format!(
                "PR #{} 目标分支是 {}（非 {}），不进合流表",
                p.number, p.base_ref, base_branch
            ));
        }
    }
    let open_numbers: BTreeSet<u64> = probes.iter().map(|p| p.number).collect();

    // 2) PRs that left the open set: resolve their terminal state once.
    let empty_prev = Vec::new();
    let prev_tracks: &Vec<TrackState> = prev.as_ref().map(|s| &s.tracks).unwrap_or(&empty_prev);
    let prev_by_pr: BTreeMap<u64, &TrackState> = prev_tracks
        .iter()
        .filter_map(|t| t.pr_number.map(|n| (n, t)))
        .collect();
    // pr -> (headRefOid, branch)
    let mut merged_info: BTreeMap<u64, (Option<String>, String)> = BTreeMap::new();
    let mut newly_merged: BTreeSet<u64> = BTreeSet::new();
    for track in prev_tracks {
        let Some(number) = track.pr_number else {
            continue;
        };
        if open_numbers.contains(&number) {
            continue;
        }
        let branch = track.branch.clone().unwrap_or_default();
        match track.pr_state.as_deref() {
            Some("merged") => {
                merged_info.insert(number, (track.checked_head_sha.clone(), branch));
            }
            Some("closed") => {}
            _ => match gh.pr_state(&root, number) {
                Ok((state, oid)) => match state.as_str() {
                    "merged" => {
                        merged_info.insert(number, (oid, branch));
                        newly_merged.insert(number);
                    }
                    "closed" => notes.push(format!("PR #{number} 已关闭未合，需人工处置")),
                    other => notes.push(format!("PR #{number} 状态 {other}，暂不判定")),
                },
                Err(e) => notes.push(format!("PR #{number} 终态查询失败：{e}")),
            },
        }
    }
    stats.newly_merged = newly_merged.len();
    let merged_by_branch: BTreeMap<String, (u64, Option<String>)> = merged_info
        .iter()
        .map(|(n, (oid, branch))| (branch.clone(), (*n, oid.clone())))
        .collect();

    // lane id -> open PR / merged PR numbers, for dependency resolution.
    let lane_pr_open: BTreeMap<String, u64> = probes
        .iter()
        .filter_map(|p| {
            lane_by_branch
                .get(&p.branch)
                .map(|l| (l.id.clone(), p.number))
        })
        .collect();
    let lane_pr_merged: BTreeMap<String, u64> = merged_by_branch
        .iter()
        .filter_map(|(branch, (n, _))| lane_by_branch.get(branch).map(|l| (l.id.clone(), *n)))
        .collect();

    // 3) Per-track evidence decision; expensive detail fetch only when needed.
    let mut movement_memo: BTreeMap<String, Option<BTreeSet<String>>> = BTreeMap::new();
    struct OpenBuild {
        id: String,
        lane_id: Option<String>,
        probe: PrProbe,
        detail: PrDetail,
        evidence: EvidenceState,
        checked_base_sha: String,
        checked_at: String,
        deps_unmet: Vec<String>,
        deps_open_prs: Vec<u64>,
        depends_on: Vec<String>,
        owns: Vec<String>,
        prev_queue: Option<Category>,
        worktree: Option<String>,
    }
    let mut open_builds: Vec<OpenBuild> = Vec::new();
    for probe in &probes {
        let lane = lane_by_branch.get(&probe.branch).copied();
        let id = lane
            .map(|l| l.id.clone())
            .unwrap_or_else(|| format!("pr-{}", probe.number));
        let prev_track = prev_by_pr.get(&probe.number).copied();
        let cached = prev_track.and_then(|t| match (&t.checked_head_sha, &t.checked_base_sha) {
            (Some(h), Some(b)) => Some(CachedEvidence {
                head_sha: h.clone(),
                base_sha: b.clone(),
                files: t.files.iter().cloned().collect(),
                files_truncated: t.files_truncated,
            }),
            _ => None,
        });
        let dep_landed = lane
            .map(|l| {
                l.depends_on.iter().any(|d| {
                    lane_pr_merged
                        .get(d)
                        .map(|n| newly_merged.contains(n))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        let movement = cached.as_ref().and_then(|c| {
            if c.base_sha != base_sha {
                movement_between(&root, gh, &mut movement_memo, &c.base_sha, &base_sha)
            } else {
                None
            }
        });
        let decision = decide_evidence(
            cached.as_ref(),
            &probe.head_sha,
            &base_sha,
            movement.as_ref(),
            dep_landed,
        );
        let cached_detail = |t: &TrackState| PrDetail {
            ci: t.ci.unwrap_or(CiState::Unknown),
            review: t.review.unwrap_or(ReviewState::Unknown),
            mergeable: t.mergeable.unwrap_or(MergeState::Unknown),
            files: t.files.clone(),
            files_truncated: t.files_truncated,
        };
        let prev_checked_at = prev_track
            .and_then(|t| t.checked_at.clone())
            .unwrap_or_else(|| now_iso.clone());
        let (detail, evidence, checked_base_sha, checked_at) = match decision {
            EvidenceDecision::ReuseSame => {
                stats.reused_same += 1;
                (
                    cached_detail(prev_track.expect("cache implies prev")),
                    EvidenceState::ReusedSame,
                    cached.as_ref().expect("cache").base_sha.clone(),
                    prev_checked_at,
                )
            }
            EvidenceDecision::ReuseValid => {
                stats.reused_valid += 1;
                // Disjointness proved validity against the new base: the key's
                // base half advances without a re-check.
                (
                    cached_detail(prev_track.expect("cache implies prev")),
                    EvidenceState::ReusedValid,
                    base_sha.clone(),
                    prev_checked_at,
                )
            }
            EvidenceDecision::Invalidated => {
                stats.invalidated += 1;
                (
                    cached_detail(prev_track.expect("cache implies prev")),
                    EvidenceState::Invalidated,
                    cached.as_ref().expect("cache").base_sha.clone(),
                    prev_checked_at,
                )
            }
            EvidenceDecision::CheckNew
            | EvidenceDecision::CheckHeadMoved
            | EvidenceDecision::CheckDepLanded
            | EvidenceDecision::CheckConservative => {
                stats.checked += 1;
                let detail = match gh.pr_detail(&root, probe.number) {
                    Ok(d) => d,
                    Err(e) => {
                        notes.push(format!("PR #{} 取证失败：{e}", probe.number));
                        PrDetail {
                            ci: CiState::Unknown,
                            review: ReviewState::Unknown,
                            mergeable: MergeState::Unknown,
                            files: Vec::new(),
                            files_truncated: false,
                        }
                    }
                };
                (
                    detail,
                    EvidenceState::Fresh,
                    base_sha.clone(),
                    now_iso.clone(),
                )
            }
        };
        let (deps_unmet, deps_open_prs, depends_on, owns) = if let Some(l) = lane {
            let mut unmet = Vec::new();
            let mut open_prs = Vec::new();
            for dep in &l.depends_on {
                let landed = lane_by_id
                    .get(dep)
                    .map(|dl| dl.status == "landed")
                    .unwrap_or(false);
                let merged = lane_pr_merged.contains_key(dep);
                if let Some(n) = lane_pr_open.get(dep) {
                    open_prs.push(*n);
                }
                if !(landed || merged) {
                    let label = lane_pr_open
                        .get(dep)
                        .map(|n| format!("#{n}"))
                        .unwrap_or_else(|| dep.clone());
                    unmet.push(label);
                }
            }
            (unmet, open_prs, l.depends_on.clone(), l.owns.clone())
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };
        let worktree = lane
            .map(|l| l.worktree.clone())
            .filter(|p| wt_by_path.contains_key(p))
            .or_else(|| wt_by_branch.get(&probe.branch).cloned());
        open_builds.push(OpenBuild {
            id,
            lane_id: lane.map(|l| l.id.clone()),
            probe: probe.clone(),
            detail,
            evidence,
            checked_base_sha,
            checked_at,
            deps_unmet,
            deps_open_prs,
            depends_on,
            owns,
            prev_queue: prev_track.and_then(|t| t.queue_category),
            worktree,
        });
    }

    // 4) Transitive dependent counts inside the open set (NOW ranking).
    let index_by_lane: BTreeMap<String, usize> = open_builds
        .iter()
        .enumerate()
        .filter_map(|(i, b)| b.lane_id.clone().map(|id| (id, i)))
        .collect();
    let mut forward_edges: Vec<Vec<usize>> = vec![Vec::new(); open_builds.len()];
    for (i, build) in open_builds.iter().enumerate() {
        for dep in &build.depends_on {
            if let Some(&j) = index_by_lane.get(dep) {
                forward_edges[j].push(i);
            }
        }
    }
    let mut dependents = vec![0usize; open_builds.len()];
    for (i, count) in dependents.iter_mut().enumerate() {
        let mut seen = BTreeSet::new();
        let mut stack = forward_edges[i].clone();
        while let Some(j) = stack.pop() {
            if seen.insert(j) {
                stack.extend(forward_edges[j].iter().copied());
            }
        }
        *count = seen.len();
    }

    // 5) Categorize the open set.
    let facts: Vec<TrackFacts> = open_builds
        .iter()
        .enumerate()
        .map(|(i, b)| TrackFacts {
            owner: b
                .lane_id
                .clone()
                .unwrap_or_else(|| format!("#{}", b.probe.number)),
            pr_number: b.probe.number,
            ci: b.detail.ci,
            review: b.detail.review,
            mergeable: b.detail.mergeable,
            draft: b.probe.draft,
            evidence: b.evidence,
            deps_unmet: b.deps_unmet.clone(),
            files: b.detail.files.iter().cloned().collect(),
            files_truncated: b.detail.files_truncated,
            dependents: dependents[i],
            prev_category: b.prev_queue,
        })
        .collect();
    let outcomes = categorize(&facts);

    // 6) Assemble the track table + lifecycle for every worktree / track.
    let mut tracks: Vec<TrackState> = Vec::new();
    let mut used_paths: BTreeSet<String> = BTreeSet::new();
    let mut used_lane_ids: BTreeSet<String> = BTreeSet::new();
    for (build, outcome) in open_builds.iter().zip(outcomes.iter()) {
        let lane_status = build
            .lane_id
            .as_ref()
            .and_then(|id| lane_by_id.get(id))
            .map(|l| l.status.clone());
        let primary =
            build.worktree.as_deref() == primary_path.as_deref() && build.worktree.is_some();
        if let Some(p) = &build.worktree {
            used_paths.insert(p.clone());
        }
        if let Some(id) = &build.lane_id {
            used_lane_ids.insert(id.clone());
        }
        let (lifecycle, lifecycle_reason, integrated) = local_lifecycle(
            &base_ref,
            lane_status.as_deref(),
            build.worktree.as_deref().map(Path::new),
            primary,
            None,
            true,
            opts.quiet_hours,
        );
        tracks.push(TrackState {
            id: build.id.clone(),
            lane_id: build.lane_id.clone(),
            pr_number: Some(build.probe.number),
            branch: Some(build.probe.branch.clone()),
            url: Some(build.probe.url.clone()),
            checked_head_sha: Some(build.probe.head_sha.clone()),
            checked_base_sha: Some(build.checked_base_sha.clone()),
            checked_at: Some(build.checked_at.clone()),
            evidence: Some(build.evidence),
            ci: Some(build.detail.ci),
            review: Some(build.detail.review),
            mergeable: Some(build.detail.mergeable),
            draft: Some(build.probe.draft),
            files: build.detail.files.clone(),
            files_truncated: build.detail.files_truncated,
            depends_on: build.depends_on.clone(),
            owns: build.owns.clone(),
            category: Some(outcome.display),
            queue_category: Some(outcome.queue),
            reason: Some(outcome.reason.clone()),
            pr_state: Some("open".to_string()),
            integrated,
            deps_open_prs: build.deps_open_prs.clone(),
            primary,
            worktree: build.worktree.clone(),
            lifecycle,
            lifecycle_reason,
        });
    }

    // Merged PRs whose lane or worktree still exists locally: REAPABLE input.
    for (number, (oid, branch)) in &merged_info {
        let lane = lane_by_branch.get(branch).copied();
        let path = lane
            .map(|l| l.worktree.clone())
            .filter(|p| wt_by_path.contains_key(p))
            .or_else(|| wt_by_branch.get(branch).cloned());
        if lane.is_none() && path.is_none() {
            continue; // nothing local left to manage
        }
        let id = lane
            .map(|l| l.id.clone())
            .unwrap_or_else(|| format!("pr-{number}"));
        let primary = path.as_deref() == primary_path.as_deref() && path.is_some();
        if let Some(p) = &path {
            used_paths.insert(p.clone());
        }
        used_lane_ids.extend(lane.map(|l| l.id.clone()));
        let lane_status = lane.map(|l| l.status.clone());
        let (lifecycle, lifecycle_reason, integrated) = local_lifecycle(
            &base_ref,
            lane_status.as_deref(),
            path.as_deref().map(Path::new),
            primary,
            oid.as_deref(),
            false,
            opts.quiet_hours,
        );
        if let Some(l) = lane {
            if wt::ownership_live(&l.status) {
                notes.push(format!(
                    "lane {} 的 PR #{number} 已合流，建议 `agent-on worktree set-status landed --id {}`",
                    l.id, l.id
                ));
            }
        }
        tracks.push(TrackState {
            id,
            lane_id: lane.map(|l| l.id.clone()),
            pr_number: Some(*number),
            branch: Some(branch.clone()),
            url: None,
            checked_head_sha: oid.clone(),
            checked_base_sha: Some(base_sha.clone()),
            checked_at: Some(now_iso.clone()),
            evidence: None,
            ci: None,
            review: None,
            mergeable: None,
            draft: None,
            files: Vec::new(),
            files_truncated: false,
            depends_on: lane.map(|l| l.depends_on.clone()).unwrap_or_default(),
            owns: lane.map(|l| l.owns.clone()).unwrap_or_default(),
            category: None,
            queue_category: None,
            reason: None,
            pr_state: Some("merged".to_string()),
            integrated,
            deps_open_prs: Vec::new(),
            primary,
            worktree: path,
            lifecycle,
            lifecycle_reason,
        });
    }

    // Lanes with no PR at all: lifecycle only.
    for lane in &lanes {
        if used_lane_ids.contains(&lane.id) {
            continue;
        }
        let path = Some(lane.worktree.clone()).filter(|p| wt_by_path.contains_key(p));
        let primary = path.as_deref() == primary_path.as_deref() && path.is_some();
        if let Some(p) = &path {
            used_paths.insert(p.clone());
        }
        let (lifecycle, lifecycle_reason, integrated) = local_lifecycle(
            &base_ref,
            Some(lane.status.as_str()),
            path.as_deref().map(Path::new),
            primary,
            None,
            false,
            opts.quiet_hours,
        );
        tracks.push(TrackState {
            id: lane.id.clone(),
            lane_id: Some(lane.id.clone()),
            pr_number: None,
            branch: Some(lane.branch.clone()),
            url: None,
            checked_head_sha: None,
            checked_base_sha: None,
            checked_at: None,
            evidence: None,
            ci: None,
            review: None,
            mergeable: None,
            draft: None,
            files: Vec::new(),
            files_truncated: false,
            depends_on: lane.depends_on.clone(),
            owns: lane.owns.clone(),
            category: None,
            queue_category: None,
            reason: None,
            pr_state: None,
            integrated,
            deps_open_prs: Vec::new(),
            primary,
            worktree: path,
            lifecycle,
            lifecycle_reason,
        });
    }

    // Remaining worktrees (unregistered + the primary tree): lifecycle only.
    for info in &worktrees {
        let path = canonical_string(&info.path);
        if used_paths.contains(&path) {
            continue;
        }
        let primary = Some(&path) == primary_path.as_ref();
        let branch = info.branch.clone();
        let merged = branch
            .as_ref()
            .and_then(|b| merged_by_branch.get(b))
            .cloned();
        let mut id = branch
            .clone()
            .unwrap_or_else(|| format!("wt-{}", short(&info.head)));
        if tracks.iter().any(|t| t.id == id) {
            id = format!("{id}-wt");
        }
        let (lifecycle, lifecycle_reason, integrated) = local_lifecycle(
            &base_ref,
            None,
            Some(Path::new(&path)),
            primary,
            merged.as_ref().and_then(|(_, oid)| oid.as_deref()),
            false,
            opts.quiet_hours,
        );
        tracks.push(TrackState {
            id,
            lane_id: None,
            pr_number: merged.as_ref().map(|(n, _)| *n),
            branch,
            url: None,
            checked_head_sha: None,
            checked_base_sha: None,
            checked_at: None,
            evidence: None,
            ci: None,
            review: None,
            mergeable: None,
            draft: None,
            files: Vec::new(),
            files_truncated: false,
            depends_on: Vec::new(),
            owns: Vec::new(),
            category: None,
            queue_category: None,
            reason: None,
            pr_state: merged.as_ref().map(|_| "merged".to_string()),
            integrated,
            deps_open_prs: Vec::new(),
            primary,
            worktree: Some(path),
            lifecycle,
            lifecycle_reason,
        });
    }

    let active_lanes = lanes.iter().filter(|l| l.status == "active").count();
    let cap = wt::active_cap(&root);
    let snapshot = Snapshot {
        version: SNAPSHOT_VERSION,
        generated_at: now_iso,
        base_branch,
        base_sha,
        base_sha_source,
        tracks,
        notes,
    };
    Ok(RefreshBuild {
        common,
        root,
        snapshot,
        stats,
        active_lanes,
        cap,
    })
}

pub fn run_refresh(repo: &Path, gh: &dyn GhClient, opts: &LandingOpts) -> (i32, String) {
    let build = match build_refresh(repo, gh, opts) {
        Ok(v) => v,
        Err(e) => return (1, format!("ERROR: {e}\n")),
    };
    if let Err(e) = save_snapshot(&build.common, &build.snapshot) {
        return (1, format!("ERROR: {e}\n"));
    }
    if opts.json {
        return (
            0,
            snapshot_json(&build.snapshot, build.active_lanes, build.cap),
        );
    }
    let mut out = format!(
        "LANDING REFRESH: {}\nbase: {} @ {} ({})\n取证 {} 条 | SKIP 复用 {} 条 | 证据仍有效 {} 条 | 失效待 rebase {} 条 | 新合流 {} 条\n",
        build.root.display(),
        build.snapshot.base_branch,
        short(&build.snapshot.base_sha),
        build.snapshot.base_sha_source,
        build.stats.checked,
        build.stats.reused_same,
        build.stats.reused_valid,
        build.stats.invalidated,
        build.stats.newly_merged,
    );
    out.push_str(&render_body(&build.snapshot, build.active_lanes, build.cap));
    (0, out)
}

/// Re-derive lifecycle from live local git state while keeping the cached PR
/// evidence; status/plan never touch the network.
fn reload_with_live_lifecycle(
    repo: &Path,
    quiet_hours: u64,
) -> Result<(PathBuf, Snapshot, usize, u64), String> {
    let root = wt::repo_root(repo)?;
    let common = wt::common_git_dir(&root)?;
    let Some(mut snapshot) = load_snapshot(&common)? else {
        return Err("还没有 landing 快照；先跑 `agent-on landing refresh` 取证一次".to_string());
    };
    let lanes = wt::load_records(&root)?;
    let lane_by_id: BTreeMap<String, &wt::LaneRecord> =
        lanes.iter().map(|l| (l.id.clone(), l)).collect();
    let worktrees = wt::parse_worktrees(&wt::git(&root, &["worktree", "list", "--porcelain"])?);
    let primary_path = worktrees.first().map(|w| canonical_string(&w.path));
    let wt_by_path: BTreeSet<String> = worktrees
        .iter()
        .map(|w| canonical_string(&w.path))
        .collect();
    let wt_by_branch: BTreeMap<String, String> = worktrees
        .iter()
        .filter_map(|w| w.branch.clone().map(|b| (b, canonical_string(&w.path))))
        .collect();
    let base_ref = format!("origin/{}", snapshot.base_branch);
    let base_ref = if wt::git(&root, &["rev-parse", "--verify", &base_ref]).is_ok() {
        base_ref
    } else {
        snapshot.base_branch.clone()
    };
    let mut seen_lanes: BTreeSet<String> = BTreeSet::new();
    let mut seen_paths: BTreeSet<String> = BTreeSet::new();
    for track in &mut snapshot.tracks {
        let lane = track
            .lane_id
            .as_ref()
            .and_then(|id| lane_by_id.get(id))
            .copied();
        if let Some(id) = &track.lane_id {
            seen_lanes.insert(id.clone());
        }
        let path = track
            .worktree
            .clone()
            .filter(|p| wt_by_path.contains(p))
            .or_else(|| {
                track
                    .branch
                    .as_ref()
                    .and_then(|b| wt_by_branch.get(b).cloned())
            });
        if let Some(p) = &path {
            seen_paths.insert(p.clone());
        }
        track.worktree = path.clone();
        let primary = path.as_deref() == primary_path.as_deref() && path.is_some();
        track.primary = primary;
        let merged_oid = if track.pr_state.as_deref() == Some("merged") {
            track.checked_head_sha.as_deref()
        } else {
            None
        };
        let (lifecycle, reason, integrated) = local_lifecycle(
            &base_ref,
            lane.map(|l| l.status.as_str()),
            path.as_deref().map(Path::new),
            primary,
            merged_oid,
            track.pr_state.as_deref() == Some("open"),
            quiet_hours,
        );
        track.lifecycle = lifecycle;
        track.lifecycle_reason = reason;
        track.integrated = integrated;
    }
    // Lanes/worktrees claimed after the last refresh: lifecycle-only rows.
    for lane in &lanes {
        if seen_lanes.contains(&lane.id) {
            continue;
        }
        let path = Some(lane.worktree.clone()).filter(|p| wt_by_path.contains(p));
        let primary = path.as_deref() == primary_path.as_deref() && path.is_some();
        if let Some(p) = &path {
            seen_paths.insert(p.clone());
        }
        let (lifecycle, reason, integrated) = local_lifecycle(
            &base_ref,
            Some(lane.status.as_str()),
            path.as_deref().map(Path::new),
            primary,
            None,
            false,
            quiet_hours,
        );
        snapshot.notes.push(format!(
            "lane {} 在快照之后登记，未取证；跑 refresh 更新合流表",
            lane.id
        ));
        snapshot.tracks.push(TrackState {
            id: lane.id.clone(),
            lane_id: Some(lane.id.clone()),
            pr_number: None,
            branch: Some(lane.branch.clone()),
            url: None,
            checked_head_sha: None,
            checked_base_sha: None,
            checked_at: None,
            evidence: None,
            ci: None,
            review: None,
            mergeable: None,
            draft: None,
            files: Vec::new(),
            files_truncated: false,
            depends_on: lane.depends_on.clone(),
            owns: lane.owns.clone(),
            category: None,
            queue_category: None,
            reason: None,
            pr_state: None,
            integrated,
            deps_open_prs: Vec::new(),
            primary,
            worktree: path,
            lifecycle,
            lifecycle_reason: reason,
        });
    }
    for info in &worktrees {
        let path = canonical_string(&info.path);
        if seen_paths.contains(&path) {
            continue;
        }
        let primary = Some(&path) == primary_path.as_ref();
        let mut id = info
            .branch
            .clone()
            .unwrap_or_else(|| format!("wt-{}", short(&info.head)));
        if snapshot.tracks.iter().any(|t| t.id == id) {
            id = format!("{id}-wt");
        }
        let (lifecycle, reason, integrated) = local_lifecycle(
            &base_ref,
            None,
            Some(Path::new(&path)),
            primary,
            None,
            false,
            quiet_hours,
        );
        snapshot.tracks.push(TrackState {
            id,
            lane_id: None,
            pr_number: None,
            branch: info.branch.clone(),
            url: None,
            checked_head_sha: None,
            checked_base_sha: None,
            checked_at: None,
            evidence: None,
            ci: None,
            review: None,
            mergeable: None,
            draft: None,
            files: Vec::new(),
            files_truncated: false,
            depends_on: Vec::new(),
            owns: Vec::new(),
            category: None,
            queue_category: None,
            reason: None,
            pr_state: None,
            integrated,
            deps_open_prs: Vec::new(),
            primary,
            worktree: Some(path),
            lifecycle,
            lifecycle_reason: reason,
        });
    }
    let active_lanes = lanes.iter().filter(|l| l.status == "active").count();
    let cap = wt::active_cap(&root);
    Ok((root, snapshot, active_lanes, cap))
}

fn snapshot_age(generated_at: &str) -> String {
    let Ok(then) = chrono::DateTime::parse_from_rfc3339(generated_at) else {
        return "未知".to_string();
    };
    let minutes = (chrono::Utc::now() - then.with_timezone(&chrono::Utc)).num_minutes();
    if minutes < 1 {
        "刚刚".to_string()
    } else if minutes < 60 {
        format!("{minutes} 分钟前")
    } else {
        format!("{} 小时前", minutes / 60)
    }
}

pub fn run_status(repo: &Path, opts: &LandingOpts) -> (i32, String) {
    let (root, snapshot, active_lanes, cap) =
        match reload_with_live_lifecycle(repo, opts.quiet_hours) {
            Ok(v) => v,
            Err(e) => return (1, format!("ERROR: {e}\n")),
        };
    if opts.json {
        return (0, snapshot_json(&snapshot, active_lanes, cap));
    }
    let mut out = format!(
        "LANDING CONTROL PLANE: {}\nbase: {} @ {} ({}) | snapshot {}（{}）\n",
        root.display(),
        snapshot.base_branch,
        short(&snapshot.base_sha),
        snapshot.base_sha_source,
        snapshot.generated_at,
        snapshot_age(&snapshot.generated_at),
    );
    out.push_str(&render_body(&snapshot, active_lanes, cap));
    (0, out)
}

pub fn run_plan(repo: &Path, opts: &LandingOpts) -> (i32, String) {
    let (root, snapshot, active_lanes, cap) =
        match reload_with_live_lifecycle(repo, opts.quiet_hours) {
            Ok(v) => v,
            Err(e) => return (1, format!("ERROR: {e}\n")),
        };
    let plan_tracks: Vec<PlanTrack> = snapshot
        .tracks
        .iter()
        .filter_map(|t| {
            let (Some(number), Some(queue)) = (t.pr_number, t.queue_category) else {
                return None;
            };
            Some(PlanTrack {
                pr_number: number,
                owner: t.lane_id.clone().unwrap_or_else(|| t.id.clone()),
                queue,
                deps: t.deps_open_prs.clone(),
            })
        })
        .collect();
    if opts.json {
        let value = serde_json::json!({
            "snapshot": snapshot,
            "waves": plan_waves(&plan_tracks),
            "active_lanes": active_lanes,
            "active_cap": cap,
        });
        return (
            0,
            serde_json::to_string_pretty(&value)
                .map(|v| format!("{v}\n"))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}\n")),
        );
    }
    let mut out = format!(
        "LANDING PLAN: {}\nbase: {} @ {} ({}) | snapshot {}（{}）\n\n",
        root.display(),
        snapshot.base_branch,
        short(&snapshot.base_sha),
        snapshot.base_sha_source,
        snapshot.generated_at,
        snapshot_age(&snapshot.generated_at),
    );
    out.push_str(&render_table(&snapshot.tracks));
    out.push('\n');
    for line in plan_waves(&plan_tracks) {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(
        "\n合并严格串行；波次只是建议，实际合流仍走控制轨合流清单 + 远端 read-back。\nREAD-ONLY: 未联网、未改任何 worktree / 分支 / PR。\n",
    );
    (0, out)
}

// ---------------------------------------------------------------------------
// Tests (contract from kit/landing-control-plane.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|p| p.to_string()).collect()
    }

    fn cached(head: &str, base: &str, files: &[&str]) -> CachedEvidence {
        CachedEvidence {
            head_sha: head.to_string(),
            base_sha: base.to_string(),
            files: set(files),
            files_truncated: false,
        }
    }

    // ---- decide_evidence ----

    #[test]
    fn evidence_first_check_when_no_cache() {
        assert_eq!(
            decide_evidence(None, "h1", "b1", None, false),
            EvidenceDecision::CheckNew
        );
    }

    #[test]
    fn evidence_head_moved_forces_recheck() {
        let c = cached("h1", "b1", &["a.rs"]);
        assert_eq!(
            decide_evidence(Some(&c), "h2", "b1", None, false),
            EvidenceDecision::CheckHeadMoved
        );
    }

    #[test]
    fn evidence_unchanged_shas_reuse_verbatim() {
        let c = cached("h1", "b1", &["a.rs"]);
        // Even a claimed dep landing cannot matter if base did not move.
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b1", None, true),
            EvidenceDecision::ReuseSame
        );
    }

    #[test]
    fn evidence_dep_landed_forces_recheck_when_base_moved() {
        let c = cached("h1", "b1", &["a.rs"]);
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b2", Some(&set(&["x.rs"])), true),
            EvidenceDecision::CheckDepLanded
        );
    }

    #[test]
    fn evidence_unknown_movement_is_conservative() {
        let c = cached("h1", "b1", &["a.rs"]);
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b2", None, false),
            EvidenceDecision::CheckConservative
        );
    }

    #[test]
    fn evidence_overlapping_movement_invalidates() {
        let c = cached("h1", "b1", &["a.rs", "b.rs"]);
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b2", Some(&set(&["b.rs", "z.rs"])), false),
            EvidenceDecision::Invalidated
        );
    }

    #[test]
    fn evidence_truncated_files_invalidate_on_base_move() {
        let mut c = cached("h1", "b1", &["a.rs"]);
        c.files_truncated = true;
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b2", Some(&set(&["z.rs"])), false),
            EvidenceDecision::Invalidated
        );
    }

    #[test]
    fn evidence_disjoint_movement_stays_valid() {
        let c = cached("h1", "b1", &["a.rs"]);
        assert_eq!(
            decide_evidence(Some(&c), "h1", "b2", Some(&set(&["z.rs"])), false),
            EvidenceDecision::ReuseValid
        );
    }

    // ---- categorize ----

    fn green(owner: &str, pr: u64, files: &[&str]) -> TrackFacts {
        TrackFacts {
            owner: owner.to_string(),
            pr_number: pr,
            ci: CiState::Green,
            review: ReviewState::Approved,
            mergeable: MergeState::Clean,
            draft: false,
            evidence: EvidenceState::Fresh,
            deps_unmet: Vec::new(),
            files: set(files),
            files_truncated: false,
            dependents: 0,
            prev_category: None,
        }
    }

    #[test]
    fn ci_red_is_fix_with_session_assignment() {
        let mut t = green("auth-api", 186, &["api/auth.rs"]);
        t.ci = CiState::Red;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Fix);
        assert_eq!(out[0].reason, "CI 红，分配给 auth-api 会话");
    }

    #[test]
    fn changes_requested_is_fix() {
        let mut t = green("ui-web", 190, &["ui/a.tsx"]);
        t.review = ReviewState::ChangesRequested;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Fix);
        assert_eq!(out[0].reason, "评审打回，分配给 ui-web 会话");
    }

    #[test]
    fn invalidated_evidence_is_stale() {
        let mut t = green("auth-api", 189, &["api/auth.rs"]);
        t.evidence = EvidenceState::Invalidated;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Stale);
        assert_eq!(out[0].reason, "main 更新且文件重叠，需要 rebase");
    }

    #[test]
    fn conflicting_merge_state_is_stale() {
        let mut t = green("auth-api", 189, &["api/auth.rs"]);
        t.mergeable = MergeState::Conflicting;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Stale);
        assert_eq!(out[0].reason, "与 main 冲突，需要 rebase");
    }

    #[test]
    fn fix_takes_precedence_over_stale() {
        let mut t = green("auth-api", 186, &["api/auth.rs"]);
        t.ci = CiState::Red;
        t.evidence = EvidenceState::Invalidated;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Fix);
    }

    #[test]
    fn unmet_dependency_is_next() {
        let mut t = green("ui-web", 184, &["ui/a.tsx"]);
        t.deps_unmet = vec!["#182".to_string()];
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Next);
        assert_eq!(out[0].reason, "等 #182");
    }

    #[test]
    fn single_green_root_is_now() {
        let out = categorize(&[green("auth-api", 182, &["api/auth.rs"])]);
        assert_eq!(out[0].display, Category::Now);
        assert_eq!(out[0].reason, "全绿，依赖根节点，可合");
    }

    #[test]
    fn second_green_root_disjoint_is_parallel() {
        let a = green("auth-api", 182, &["api/auth.rs"]);
        let b = green("ui-web", 187, &["ui/a.tsx"]);
        let out = categorize(&[a, b]);
        assert_eq!(out[0].display, Category::Now);
        assert_eq!(out[1].display, Category::Parallel);
        assert_eq!(out[1].reason, "与当前变更无重叠，证据仍有效");
    }

    #[test]
    fn second_green_root_overlapping_queues_behind_now() {
        let a = green("auth-api", 182, &["api/auth.rs"]);
        let b = green("auth-extra", 185, &["api/auth.rs", "api/token.rs"]);
        let out = categorize(&[a, b]);
        assert_eq!(out[0].display, Category::Now);
        assert_eq!(out[1].display, Category::Next);
        assert!(out[1].reason.contains("等 #182"), "{}", out[1].reason);
    }

    #[test]
    fn now_ranking_prefers_more_dependents_then_lower_pr() {
        let mut a = green("leaf", 100, &["a.rs"]);
        a.dependents = 0;
        let mut b = green("root", 200, &["b.rs"]);
        b.dependents = 2;
        let out = categorize(&[a, b]);
        assert_eq!(out[1].display, Category::Now, "root with dependents wins");
        assert_eq!(out[0].display, Category::Parallel);
    }

    #[test]
    fn pending_ci_disjoint_is_parallel_with_reason() {
        let a = green("auth-api", 182, &["api/auth.rs"]);
        let mut b = green("ui-web", 187, &["ui/a.tsx"]);
        b.ci = CiState::Pending;
        let out = categorize(&[a, b]);
        assert_eq!(out[1].display, Category::Parallel);
        assert_eq!(out[1].reason, "CI 运行中，可并行验证");
    }

    #[test]
    fn draft_never_becomes_now() {
        let mut t = green("auth-api", 182, &["api/auth.rs"]);
        t.draft = true;
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Parallel);
        assert_eq!(out[0].reason, "draft，可并行开发");
    }

    #[test]
    fn reused_same_unchanged_next_collapses_to_skip() {
        let mut t = green("ui-web", 191, &["ui/a.tsx"]);
        t.deps_unmet = vec!["#182".to_string()];
        t.evidence = EvidenceState::ReusedSame;
        t.prev_category = Some(Category::Next);
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Skip);
        assert_eq!(out[0].queue, Category::Next, "plan keeps the queue meaning");
        assert_eq!(out[0].reason, "SHA 未变化，不重复检查");
    }

    #[test]
    fn reused_same_now_is_still_shown_as_now() {
        let mut t = green("auth-api", 182, &["api/auth.rs"]);
        t.evidence = EvidenceState::ReusedSame;
        t.prev_category = Some(Category::Now);
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Now, "actionable rows never skip");
    }

    #[test]
    fn reused_same_with_changed_category_does_not_skip() {
        let mut t = green("ui-web", 191, &["ui/a.tsx"]);
        t.deps_unmet = vec!["#182".to_string()];
        t.evidence = EvidenceState::ReusedSame;
        t.prev_category = Some(Category::Parallel);
        let out = categorize(&[t]);
        assert_eq!(out[0].display, Category::Next);
    }

    // ---- classify_lifecycle ----

    fn base_facts() -> LifecycleFacts {
        LifecycleFacts {
            lane_status: None,
            primary: false,
            present: true,
            dirty: Some(false),
            unpushed: Some(0),
            unique: Some(0),
            integrated: Some(false),
            pr_open: false,
            quiet_passed: Some(true),
        }
    }

    #[test]
    fn merged_clean_quiet_is_reapable() {
        let mut f = base_facts();
        f.integrated = Some(true);
        f.lane_status = Some("landed".to_string());
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Reapable);
    }

    #[test]
    fn merged_but_dirty_is_rescue() {
        let mut f = base_facts();
        f.integrated = Some(true);
        f.dirty = Some(true);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Rescue);
    }

    #[test]
    fn merged_clean_inside_quiet_window_waits() {
        let mut f = base_facts();
        f.integrated = Some(true);
        f.quiet_passed = Some(false);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Waiting);
    }

    #[test]
    fn active_lane_dirty_is_active_not_rescue() {
        let mut f = base_facts();
        f.lane_status = Some("active".to_string());
        f.dirty = Some(true);
        f.unpushed = None;
        f.unique = Some(3);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Active);
    }

    #[test]
    fn non_active_dirty_is_rescue() {
        let mut f = base_facts();
        f.lane_status = Some("blocked".to_string());
        f.dirty = Some(true);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Rescue);
    }

    #[test]
    fn unpushed_commits_are_rescue() {
        let mut f = base_facts();
        f.lane_status = Some("ready".to_string());
        f.unpushed = Some(2);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Rescue);
    }

    #[test]
    fn orphan_unique_commits_without_upstream_are_rescue() {
        let mut f = base_facts();
        f.unpushed = None;
        f.unique = Some(5);
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Rescue);
    }

    #[test]
    fn parked_clean_pushed_is_parked() {
        let mut f = base_facts();
        f.lane_status = Some("parked".to_string());
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Parked);
    }

    #[test]
    fn ready_clean_pushed_waits_for_control_lane() {
        let mut f = base_facts();
        f.lane_status = Some("ready".to_string());
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Waiting);
    }

    #[test]
    fn open_pr_clean_pushed_waits() {
        let mut f = base_facts();
        f.pr_open = true;
        assert_eq!(classify_lifecycle(&f).0, Lifecycle::Waiting);
    }

    #[test]
    fn merged_track_with_removed_worktree_waits_for_forget() {
        let mut f = base_facts();
        f.integrated = Some(true);
        f.present = false;
        f.dirty = None;
        f.unpushed = None;
        f.unique = None;
        let (class, reason) = classify_lifecycle(&f);
        assert_eq!(class, Lifecycle::Waiting);
        assert!(reason.contains("forget"), "{reason}");
    }

    #[test]
    fn primary_is_never_reapable() {
        let mut f = base_facts();
        f.primary = true;
        f.integrated = Some(true);
        let (class, reason) = classify_lifecycle(&f);
        assert_eq!(class, Lifecycle::Waiting);
        assert!(reason.contains("控制轨"), "{reason}");
    }

    #[test]
    fn idle_clean_primary_waits_as_control_lane() {
        let mut f = base_facts();
        f.primary = true;
        let (class, reason) = classify_lifecycle(&f);
        assert_eq!(class, Lifecycle::Waiting);
        assert!(reason.contains("控制轨"), "{reason}");
    }

    // ---- rendering ----

    #[test]
    fn row_format_matches_contract_exactly() {
        assert_eq!(
            render_row(Category::Now, 182, "全绿，依赖根节点，可合"),
            "NOW       #182  全绿，依赖根节点，可合"
        );
        assert_eq!(
            render_row(Category::Parallel, 187, "与当前变更无重叠，证据仍有效"),
            "PARALLEL  #187  与当前变更无重叠，证据仍有效"
        );
        assert_eq!(
            render_row(Category::Skip, 191, "SHA 未变化，不重复检查"),
            "SKIP      #191  SHA 未变化，不重复检查"
        );
    }

    #[test]
    fn summary_lists_five_fixed_lines() {
        let text = render_summary(&SummaryCounts {
            active: 2,
            parked: 3,
            waiting: 4,
            rescue: 1,
            reapable: 5,
        });
        assert_eq!(
            text,
            "现在做：2 条\n下一批：3 条\n等待中：4 条\n需抢救：1 条\n可回收：5 条\n"
        );
    }

    // ---- planning ----

    #[test]
    fn waves_are_serial_with_parallel_prep_and_fix_queue() {
        let tracks = vec![
            PlanTrack {
                pr_number: 182,
                owner: "auth-api".to_string(),
                queue: Category::Now,
                deps: vec![],
            },
            PlanTrack {
                pr_number: 184,
                owner: "ui-web".to_string(),
                queue: Category::Next,
                deps: vec![182],
            },
            PlanTrack {
                pr_number: 187,
                owner: "docs".to_string(),
                queue: Category::Parallel,
                deps: vec![],
            },
            PlanTrack {
                pr_number: 186,
                owner: "auth-fix".to_string(),
                queue: Category::Fix,
                deps: vec![],
            },
            PlanTrack {
                pr_number: 189,
                owner: "search".to_string(),
                queue: Category::Stale,
                deps: vec![],
            },
        ];
        let lines = plan_waves(&tracks);
        let joined = lines.join("\n");
        assert!(
            joined.contains("WAVE 1") && joined.contains("#182") && joined.contains("#187"),
            "{joined}"
        );
        assert!(
            joined.contains("WAVE 2") && joined.contains("#184"),
            "{joined}"
        );
        assert!(
            joined.contains("#186") && joined.contains("auth-fix"),
            "{joined}"
        );
        assert!(
            joined.contains("#189") && joined.contains("rebase"),
            "{joined}"
        );
    }

    #[test]
    fn chained_next_tracks_layer_into_later_waves() {
        let tracks = vec![
            PlanTrack {
                pr_number: 1,
                owner: "a".to_string(),
                queue: Category::Now,
                deps: vec![],
            },
            PlanTrack {
                pr_number: 2,
                owner: "b".to_string(),
                queue: Category::Next,
                deps: vec![1],
            },
            PlanTrack {
                pr_number: 3,
                owner: "c".to_string(),
                queue: Category::Next,
                deps: vec![2],
            },
        ];
        let lines = plan_waves(&tracks);
        let wave2 = lines.iter().find(|l| l.starts_with("WAVE 2")).unwrap();
        let wave3 = lines.iter().find(|l| l.starts_with("WAVE 3")).unwrap();
        assert!(wave2.contains("#2"), "{wave2}");
        assert!(wave3.contains("#3"), "{wave3}");
    }

    // ---- snapshot persistence ----

    #[test]
    fn snapshot_round_trips_and_rejects_bad_version() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        assert_eq!(load_snapshot(dir).unwrap(), None);
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            generated_at: "2026-08-16T08:00:00Z".to_string(),
            base_branch: "main".to_string(),
            base_sha: "b1".to_string(),
            base_sha_source: "ls-remote".to_string(),
            tracks: vec![TrackState {
                id: "auth-api".to_string(),
                lane_id: Some("auth-api".to_string()),
                pr_number: Some(182),
                branch: Some("feat/auth".to_string()),
                url: None,
                checked_head_sha: Some("h1".to_string()),
                checked_base_sha: Some("b1".to_string()),
                checked_at: Some("2026-08-16T08:00:00Z".to_string()),
                evidence: Some(EvidenceState::Fresh),
                ci: Some(CiState::Green),
                review: Some(ReviewState::Approved),
                mergeable: Some(MergeState::Clean),
                draft: Some(false),
                files: vec!["api/auth.rs".to_string()],
                files_truncated: false,
                depends_on: vec![],
                owns: vec!["api/auth".to_string()],
                category: Some(Category::Now),
                queue_category: Some(Category::Now),
                reason: Some("全绿，依赖根节点，可合".to_string()),
                pr_state: Some("open".to_string()),
                integrated: Some(false),
                deps_open_prs: vec![],
                primary: false,
                worktree: None,
                lifecycle: Lifecycle::Waiting,
                lifecycle_reason: "等控制轨合流".to_string(),
            }],
            notes: vec![],
        };
        save_snapshot(dir, &snapshot).unwrap();
        let loaded = load_snapshot(dir).unwrap().unwrap();
        assert_eq!(loaded.base_sha, "b1");
        assert_eq!(loaded.tracks.len(), 1);
        assert_eq!(loaded.tracks[0].category, Some(Category::Now));

        let path = snapshot_path(dir);
        let mut raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        raw["version"] = serde_json::json!(99);
        std::fs::write(&path, serde_json::to_string(&raw).unwrap()).unwrap();
        assert!(load_snapshot(dir).is_err());
    }
}

// PartialEq for Snapshot test helpers.
impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.generated_at == other.generated_at
    }
}

#[cfg(test)]
mod io_tests {
    use super::*;
    use crate::worktree::{claim_lane, common_git_dir, ClaimOpts};
    use std::cell::RefCell;
    use std::fs;
    use tempfile::TempDir;

    fn run(cwd: &Path, args: &[&str]) {
        let output = Command::new(args[0])
            .current_dir(cwd)
            .args(&args[1..])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn rev(cwd: &Path, what: &str) -> String {
        crate::worktree::git(cwd, &["rev-parse", what]).unwrap()
    }

    struct FakeGh {
        probes: RefCell<Vec<PrProbe>>,
        details: RefCell<std::collections::BTreeMap<u64, PrDetail>>,
        states: RefCell<std::collections::BTreeMap<u64, (String, Option<String>)>>,
        detail_calls: RefCell<Vec<u64>>,
    }

    impl FakeGh {
        fn new() -> Self {
            FakeGh {
                probes: RefCell::new(Vec::new()),
                details: RefCell::new(Default::default()),
                states: RefCell::new(Default::default()),
                detail_calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl GhClient for FakeGh {
        fn probe_open_prs(&self, _root: &Path) -> Result<Vec<PrProbe>, String> {
            Ok(self.probes.borrow().clone())
        }
        fn pr_detail(&self, _root: &Path, number: u64) -> Result<PrDetail, String> {
            self.detail_calls.borrow_mut().push(number);
            self.details
                .borrow()
                .get(&number)
                .cloned()
                .ok_or_else(|| format!("no detail fixture for PR {number}"))
        }
        fn pr_state(&self, _root: &Path, number: u64) -> Result<(String, Option<String>), String> {
            self.states
                .borrow()
                .get(&number)
                .cloned()
                .ok_or_else(|| format!("no state fixture for PR {number}"))
        }
        fn compare_files(&self, _root: &Path, _old: &str, _new: &str) -> Option<Vec<String>> {
            None
        }
    }

    fn green_detail(files: &[&str]) -> PrDetail {
        PrDetail {
            ci: CiState::Green,
            review: ReviewState::Approved,
            mergeable: MergeState::Clean,
            files: files.iter().map(|f| f.to_string()).collect(),
            files_truncated: false,
        }
    }

    /// Local repo + local bare origin + one claimed lane worktree on feat/a.
    fn fixture() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        run(
            tmp.path(),
            &["git", "init", "--bare", "-b", "main", "origin.git"],
        );
        let root = tmp.path().join("repo");
        fs::create_dir_all(&root).unwrap();
        run(&root, &["git", "init", "-b", "main"]);
        run(&root, &["git", "config", "user.email", "test@example.com"]);
        run(&root, &["git", "config", "user.name", "Test"]);
        fs::create_dir_all(root.join("api")).unwrap();
        fs::create_dir_all(root.join("web")).unwrap();
        fs::write(root.join("README.md"), "root\n").unwrap();
        fs::write(root.join("api/auth.rs"), "base\n").unwrap();
        fs::write(root.join("web/ui.txt"), "base\n").unwrap();
        run(&root, &["git", "add", "."]);
        run(&root, &["git", "commit", "-m", "init"]);
        run(&root, &["git", "remote", "add", "origin", "../origin.git"]);
        run(&root, &["git", "push", "-u", "origin", "main"]);
        run(&root, &["git", "remote", "set-head", "origin", "main"]);
        let wt_a = tmp.path().join("lane-a");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "feat/a",
                wt_a.to_str().unwrap(),
                "origin/main",
            ],
        );
        fs::write(wt_a.join("api/auth.rs"), "changed\n").unwrap();
        run(&wt_a, &["git", "commit", "-am", "auth change"]);
        run(&wt_a, &["git", "push", "-u", "origin", "feat/a"]);
        let (code, out) = claim_lane(
            &wt_a,
            &ClaimOpts {
                parked: false,
                id: "auth-api".to_string(),
                goal: "auth api".to_string(),
                base: Some("origin/main".to_string()),
                owns: vec!["api".to_string()],
                depends_on: Vec::new(),
            },
        );
        assert_eq!(code, 0, "{out}");
        (tmp, root, wt_a)
    }

    fn opts() -> LandingOpts {
        LandingOpts {
            json: false,
            base: None,
            quiet_hours: 0,
        }
    }

    fn probe_a(root: &Path) -> PrProbe {
        PrProbe {
            number: 182,
            branch: "feat/a".to_string(),
            head_sha: rev(root, "feat/a"),
            base_ref: "main".to_string(),
            draft: false,
            url: "https://example.test/pr/182".to_string(),
        }
    }

    fn commit_on_main(root: &Path, file: &str, content: &str) {
        fs::write(root.join(file), content).unwrap();
        run(root, &["git", "add", file]);
        run(root, &["git", "commit", "-m", "main moves"]);
        run(root, &["git", "push", "origin", "main"]);
    }

    #[test]
    fn refresh_green_pr_is_now_and_persists_snapshot() {
        let (_tmp, root, _wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        let (code, out) = run_refresh(&root, &fake, &opts());
        assert_eq!(code, 0, "{out}");
        assert!(
            out.contains("NOW       #182  全绿，依赖根节点，可合"),
            "{out}"
        );
        assert!(out.contains("现在做：1 条"), "{out}");
        let common = common_git_dir(&root).unwrap();
        let snapshot = load_snapshot(&common).unwrap().unwrap();
        let track = snapshot
            .tracks
            .iter()
            .find(|t| t.pr_number == Some(182))
            .unwrap();
        assert_eq!(track.evidence, Some(EvidenceState::Fresh));
        assert_eq!(track.category, Some(Category::Now));
        assert_eq!(track.lane_id.as_deref(), Some("auth-api"));
    }

    #[test]
    fn second_refresh_with_unchanged_shas_reuses_evidence() {
        let (_tmp, root, _wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        let (code, out) = run_refresh(&root, &fake, &opts());
        assert_eq!(code, 0, "{out}");
        assert_eq!(
            fake.detail_calls.borrow().len(),
            1,
            "unchanged SHAs must not re-fetch evidence"
        );
        let common = common_git_dir(&root).unwrap();
        let snapshot = load_snapshot(&common).unwrap().unwrap();
        let track = snapshot
            .tracks
            .iter()
            .find(|t| t.pr_number == Some(182))
            .unwrap();
        assert_eq!(track.evidence, Some(EvidenceState::ReusedSame));
        assert_eq!(
            track.category,
            Some(Category::Now),
            "actionable rows stay visible"
        );
    }

    #[test]
    fn base_move_with_file_overlap_goes_stale_without_refetch() {
        let (_tmp, root, _wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        commit_on_main(&root, "api/auth.rs", "conflicting main change\n");
        let (code, out) = run_refresh(&root, &fake, &opts());
        assert_eq!(code, 0, "{out}");
        assert!(
            out.contains("STALE     #182  main 更新且文件重叠，需要 rebase"),
            "{out}"
        );
        assert_eq!(fake.detail_calls.borrow().len(), 1, "no re-fetch for STALE");
    }

    #[test]
    fn base_move_without_overlap_keeps_evidence_valid() {
        let (_tmp, root, _wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        commit_on_main(&root, "web/ui.txt", "disjoint main change\n");
        let (code, out) = run_refresh(&root, &fake, &opts());
        assert_eq!(code, 0, "{out}");
        assert_eq!(
            fake.detail_calls.borrow().len(),
            1,
            "no re-fetch when disjoint"
        );
        let common = common_git_dir(&root).unwrap();
        let snapshot = load_snapshot(&common).unwrap().unwrap();
        let track = snapshot
            .tracks
            .iter()
            .find(|t| t.pr_number == Some(182))
            .unwrap();
        assert_eq!(track.evidence, Some(EvidenceState::ReusedValid));
        assert_eq!(track.category, Some(Category::Now));
    }

    #[test]
    fn merged_pr_reclassifies_worktree_as_reapable() {
        let (_tmp, root, wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        let head = rev(&wt, "HEAD");
        run(&root, &["git", "merge", "--ff-only", "feat/a"]);
        run(&root, &["git", "push", "origin", "main"]);
        fake.probes.borrow_mut().clear();
        fake.states
            .borrow_mut()
            .insert(182, ("merged".to_string(), Some(head)));
        let (code, out) = run_refresh(&root, &fake, &opts());
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("可回收：1 条"), "{out}");
        assert!(out.contains("REAPABLE"), "{out}");
        let common = common_git_dir(&root).unwrap();
        let snapshot = load_snapshot(&common).unwrap().unwrap();
        let track = snapshot
            .tracks
            .iter()
            .find(|t| t.lane_id.as_deref() == Some("auth-api"))
            .unwrap();
        assert_eq!(track.lifecycle, Lifecycle::Reapable);
        assert_eq!(track.pr_state.as_deref(), Some("merged"));
    }

    #[test]
    fn status_requires_a_snapshot_first() {
        let (_tmp, root, _wt) = fixture();
        let (code, out) = run_status(&root, &opts());
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("refresh"), "{out}");
    }

    #[test]
    fn status_renders_summary_and_table_offline() {
        let (_tmp, root, _wt) = fixture();
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        let (code, out) = run_status(&root, &opts());
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("现在做：1 条"), "{out}");
        assert!(out.contains("NOW       #182"), "{out}");
        assert!(out.contains("READ-ONLY"), "{out}");
    }

    #[test]
    fn plan_orders_dependency_chain_into_waves() {
        let (tmp, root, _wt) = fixture();
        let wt_b = tmp.path().join("lane-b");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "feat/b",
                wt_b.to_str().unwrap(),
                "origin/main",
            ],
        );
        fs::write(wt_b.join("web/ui.txt"), "changed\n").unwrap();
        run(&wt_b, &["git", "commit", "-am", "ui change"]);
        run(&wt_b, &["git", "push", "-u", "origin", "feat/b"]);
        let (code, out) = claim_lane(
            &wt_b,
            &ClaimOpts {
                parked: false,
                id: "ui-web".to_string(),
                goal: "ui".to_string(),
                base: Some("origin/main".to_string()),
                owns: vec!["web".to_string()],
                depends_on: vec!["auth-api".to_string()],
            },
        );
        assert_eq!(code, 0, "{out}");
        let fake = FakeGh::new();
        fake.probes.borrow_mut().push(probe_a(&root));
        fake.probes.borrow_mut().push(PrProbe {
            number: 184,
            branch: "feat/b".to_string(),
            head_sha: rev(&root, "feat/b"),
            base_ref: "main".to_string(),
            draft: false,
            url: "https://example.test/pr/184".to_string(),
        });
        fake.details
            .borrow_mut()
            .insert(182, green_detail(&["api/auth.rs"]));
        fake.details
            .borrow_mut()
            .insert(184, green_detail(&["web/ui.txt"]));
        assert_eq!(run_refresh(&root, &fake, &opts()).0, 0);
        let (code, out) = run_plan(&root, &opts());
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("NEXT      #184  等 #182"), "{out}");
        assert!(out.contains("WAVE 1") && out.contains("#182"), "{out}");
        assert!(out.contains("WAVE 2") && out.contains("#184"), "{out}");
    }
}
