use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const RECORD_VERSION: u8 = 1;
const VALID_STATUSES: &[&str] = &["active", "blocked", "ready", "landed", "parked"];

#[derive(Debug, Clone)]
pub struct ClaimOpts {
    pub id: String,
    pub goal: String,
    pub base: Option<String>,
    pub owns: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GcOpts {
    pub dry_run: bool,
    pub json: bool,
    pub base: Option<String>,
    pub quiet_hours: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckState {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
struct GcCriterion {
    result: CheckState,
    evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrKind {
    None,
    Open,
    Merged,
    Closed,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
struct GcPrAudit {
    status: String,
    number: Option<u64>,
    url: Option<String>,
    base_ref_name: Option<String>,
    head_ref_oid: Option<String>,
    covers_head: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct GcBaseAudit {
    reference: String,
    head_ancestor_of_base: Option<bool>,
    unique_commits: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct GcCriteria {
    integrated: GcCriterion,
    preserved: GcCriterion,
    clean: GcCriterion,
}

#[derive(Debug, Clone, Serialize)]
struct GcQuietAudit {
    required_hours: u64,
    pass: Option<bool>,
    last_activity_at: Option<String>,
    hours_since_activity: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct GcWorktreeAudit {
    path: String,
    branch: Option<String>,
    head: String,
    primary: bool,
    locked: bool,
    prunable: bool,
    lane_status: Option<String>,
    du_bytes: Option<u64>,
    dirty_entries: Vec<String>,
    base: GcBaseAudit,
    upstream: Option<String>,
    unpushed_commits: Option<u64>,
    pr: GcPrAudit,
    criteria: GcCriteria,
    quiet: GcQuietAudit,
    decision: String,
    reasons: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GcPrQueryAudit {
    status: String,
    command: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GcReport {
    repo: String,
    mode: String,
    base: String,
    quiet_hours: u64,
    github_pr_query: GcPrQueryAudit,
    worktrees: Vec<GcWorktreeAudit>,
    candidates: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GhPrRow {
    number: u64,
    state: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    #[serde(rename = "headRefOid", default)]
    head_ref_oid: Option<String>,
    url: String,
    #[serde(rename = "mergedAt", default)]
    merged_at: Option<String>,
}

#[derive(Debug)]
struct GhInventory {
    audit: GcPrQueryAudit,
    rows: Option<Vec<GhPrRow>>,
}

#[derive(Debug, Clone, Copy)]
struct DecisionFacts {
    primary: bool,
    locked: bool,
    prunable: bool,
    live_lane: bool,
    pr: PrKind,
    post_merge_local: bool,
    integrated: CheckState,
    preserved: CheckState,
    clean: CheckState,
    quiet: Option<bool>,
    unique_commits: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LaneRecord {
    version: u8,
    id: String,
    goal: String,
    worktree: String,
    branch: String,
    base: String,
    base_sha_at_claim: String,
    owns: Vec<String>,
    depends_on: Vec<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct WorktreeInfo {
    path: PathBuf,
    head: String,
    branch: Option<String>,
    detached: bool,
    locked: bool,
    prunable: bool,
}

#[derive(Debug, Serialize)]
struct LaneAudit {
    id: String,
    goal: String,
    worktree: String,
    branch: String,
    base: String,
    status: String,
    owns: Vec<String>,
    depends_on: Vec<String>,
    changed_files: Vec<String>,
    out_of_bounds: Vec<String>,
    base_ahead: Option<u64>,
    unique_commits: Option<u64>,
    clean: bool,
    locked: bool,
    prunable: bool,
    reclaim: String,
    present: bool,
}

#[derive(Debug, Serialize)]
struct AuditReport {
    repo: String,
    lanes: Vec<LaneAudit>,
    unregistered_worktrees: Vec<String>,
    overlaps: Vec<String>,
    dependency_blocks: Vec<String>,
    errors: Vec<String>,
}

fn git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        })
    }
}

fn repo_root(cwd: &Path) -> Result<PathBuf, String> {
    git(cwd, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

fn common_git_dir(cwd: &Path) -> Result<PathBuf, String> {
    if let Ok(raw) = git(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ) {
        return Ok(PathBuf::from(raw));
    }
    let raw = PathBuf::from(git(cwd, &["rev-parse", "--git-common-dir"])?);
    let path = if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    };
    Ok(fs::canonicalize(&path).unwrap_or(path))
}

fn registry_dir(cwd: &Path) -> Result<PathBuf, String> {
    Ok(common_git_dir(cwd)?.join("agent-on").join("lanes"))
}

fn record_path(cwd: &Path, id: &str) -> Result<PathBuf, String> {
    Ok(registry_dir(cwd)?.join(format!("{id}.json")))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn normalize_owns(values: &[String]) -> Result<Vec<String>, String> {
    let mut out = BTreeSet::new();
    for raw in values {
        let value = raw.trim().trim_start_matches("./").trim_end_matches('/');
        if value.is_empty()
            || value == "."
            || value.starts_with('/')
            || value.split('/').any(|part| part == "..")
        {
            return Err(format!("invalid owned path boundary: {raw}"));
        }
        out.insert(value.to_string());
    }
    if out.is_empty() {
        return Err("at least one --owns path is required".to_string());
    }
    Ok(out.into_iter().collect())
}

fn boundary_contains(boundary: &str, path: &str) -> bool {
    path == boundary || path.starts_with(&format!("{boundary}/"))
}

fn boundaries_overlap(a: &str, b: &str) -> bool {
    boundary_contains(a, b) || boundary_contains(b, a)
}

fn owns_path(boundaries: &[String], path: &str) -> bool {
    boundaries.iter().any(|b| boundary_contains(b, path))
}

fn ownership_live(status: &str) -> bool {
    matches!(status, "active" | "blocked" | "ready")
}

fn load_records(cwd: &Path) -> Result<Vec<LaneRecord>, String> {
    let dir = registry_dir(cwd)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let path = entry
            .map_err(|e| format!("read {} entry: {e}", dir.display()))?
            .path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let record: LaneRecord =
            serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
        if record.version != RECORD_VERSION {
            return Err(format!(
                "unsupported lane record version {} in {} (expected {})",
                record.version,
                path.display(),
                RECORD_VERSION
            ));
        }
        if !valid_id(&record.id) || !VALID_STATUSES.contains(&record.status.as_str()) {
            return Err(format!("invalid lane record: {}", path.display()));
        }
        normalize_owns(&record.owns)
            .map_err(|e| format!("invalid lane record {}: {e}", path.display()))?;
        records.push(record);
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(records)
}

fn write_record(cwd: &Path, record: &LaneRecord) -> Result<(), String> {
    let path = record_path(cwd, &record.id)?;
    let parent = path
        .parent()
        .ok_or_else(|| "invalid registry path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    let raw = serde_json::to_string_pretty(record)
        .map_err(|e| format!("serialize lane {}: {e}", record.id))?;
    fs::write(&path, format!("{raw}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}

fn default_base(cwd: &Path) -> Result<String, String> {
    if let Ok(v) = git(
        cwd,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    for candidate in ["origin/main", "main", "origin/master", "master"] {
        if git(cwd, &["rev-parse", "--verify", candidate]).is_ok() {
            return Ok(candidate.to_string());
        }
    }
    Err("cannot infer base; pass --base <ref>".to_string())
}

pub fn claim_lane(cwd: &Path, opts: &ClaimOpts) -> (i32, String) {
    match claim_lane_inner(cwd, opts) {
        Ok(record) => (
            0,
            format!(
                "CLAIMED {} on {}\nbase: {} @ {}\nowns: {}\nnext: run `agent-on worktree check` before commit and merge\n",
                record.id,
                record.branch,
                record.base,
                &record.base_sha_at_claim[..record.base_sha_at_claim.len().min(12)],
                record.owns.join(", ")
            ),
        ),
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

fn claim_lane_inner(cwd: &Path, opts: &ClaimOpts) -> Result<LaneRecord, String> {
    if !valid_id(&opts.id) {
        return Err(
            "lane id may contain only letters, digits, dot, underscore, and dash".to_string(),
        );
    }
    if opts.goal.trim().is_empty() {
        return Err("lane goal cannot be empty".to_string());
    }
    let root = repo_root(cwd)?;
    let worktree = fs::canonicalize(&root).unwrap_or(root);
    let branch = git(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .map_err(|_| "detached HEAD cannot claim a lane; create a branch first".to_string())?;
    let base = match &opts.base {
        Some(v) => v.clone(),
        None => default_base(cwd)?,
    };
    let base_sha = git(cwd, &["rev-parse", "--verify", &base])?;
    let owns = normalize_owns(&opts.owns)?;
    let records = load_records(cwd)?;
    if records.iter().any(|r| r.id == opts.id) {
        return Err(format!(
            "lane {} already exists; change its state instead of overwriting it",
            opts.id
        ));
    }
    if let Some(existing) = records.iter().find(|r| Path::new(&r.worktree) == worktree) {
        return Err(format!(
            "worktree already has lane record {} ({}); finish/remove/forget it before reusing the path",
            existing.id, existing.status
        ));
    }
    for existing in records.iter().filter(|r| ownership_live(&r.status)) {
        for new_path in &owns {
            for old_path in &existing.owns {
                if boundaries_overlap(new_path, old_path) {
                    return Err(format!(
                        "owned path {} overlaps live lane {} boundary {}",
                        new_path, existing.id, old_path
                    ));
                }
            }
        }
    }
    for dep in &opts.depends_on {
        if !valid_id(dep) {
            return Err(format!("invalid dependency lane id: {dep}"));
        }
        if !records.iter().any(|r| &r.id == dep) {
            return Err(format!("dependency lane is not registered: {dep}"));
        }
    }
    let now = Utc::now().to_rfc3339();
    let record = LaneRecord {
        version: RECORD_VERSION,
        id: opts.id.clone(),
        goal: opts.goal.trim().to_string(),
        worktree: worktree.display().to_string(),
        branch,
        base,
        base_sha_at_claim: base_sha,
        owns,
        depends_on: opts.depends_on.clone(),
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    write_record(cwd, &record)?;
    Ok(record)
}

pub fn set_lane_status(cwd: &Path, id: Option<&str>, status: &str) -> (i32, String) {
    if !VALID_STATUSES.contains(&status) {
        return (
            1,
            format!(
                "ERROR: invalid status {status}; expected {}\n",
                VALID_STATUSES.join("|")
            ),
        );
    }
    let result = (|| -> Result<LaneRecord, String> {
        let mut records = load_records(cwd)?;
        let target_index = if let Some(id) = id {
            records.iter().position(|r| r.id == id)
        } else {
            let root = repo_root(cwd)?;
            let canonical = fs::canonicalize(&root).unwrap_or(root);
            records
                .iter()
                .position(|r| Path::new(&r.worktree) == canonical)
        }
        .ok_or_else(|| "no matching lane record".to_string())?;
        let current = records[target_index].status.as_str();
        if !transition_allowed(current, status) {
            return Err(format!("invalid lane transition: {current} -> {status}"));
        }
        if status == "ready" {
            let target = &records[target_index];
            for dep in &target.depends_on {
                let dep_status = records
                    .iter()
                    .find(|r| &r.id == dep)
                    .map(|r| r.status.as_str())
                    .unwrap_or("missing");
                if dep_status != "landed" {
                    return Err(format!(
                        "lane {} waits for {} ({})",
                        target.id, dep, dep_status
                    ));
                }
            }
            let path = Path::new(&target.worktree);
            if !path.exists() {
                return Err(format!("worktree is missing: {}", target.worktree));
            }
            if !git(path, &["status", "--porcelain"]).map(|v| v.is_empty())? {
                return Err(
                    "ready requires a clean worktree; commit or park the lane first".to_string(),
                );
            }
            let files = changed_files(path, &target.base)?;
            let escaped: Vec<String> = files
                .into_iter()
                .filter(|f| !owns_path(&target.owns, f))
                .collect();
            if !escaped.is_empty() {
                return Err(format!(
                    "ready blocked by out-of-bound changes: {}",
                    escaped.join(", ")
                ));
            }
        }
        let target = &mut records[target_index];
        target.status = status.to_string();
        target.updated_at = Utc::now().to_rfc3339();
        let result = target.clone();
        write_record(cwd, &result)?;
        Ok(result)
    })();
    match result {
        Ok(r) => (0, format!("UPDATED {}: {}\n", r.id, r.status)),
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

fn transition_allowed(from: &str, to: &str) -> bool {
    from == to
        || matches!(
            (from, to),
            ("active", "blocked")
                | ("active", "ready")
                | ("active", "parked")
                | ("blocked", "active")
                | ("blocked", "parked")
                | ("ready", "active")
                | ("ready", "blocked")
                | ("ready", "landed")
                | ("parked", "active")
        )
}

pub fn forget_lane(cwd: &Path, id: &str) -> (i32, String) {
    let result = (|| -> Result<(), String> {
        let records = load_records(cwd)?;
        let record = records
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| format!("lane does not exist: {id}"))?;
        if !matches!(record.status.as_str(), "landed" | "parked") {
            return Err(format!(
                "refuse to forget {} while status is {}; mark landed or parked first",
                id, record.status
            ));
        }
        if Path::new(&record.worktree).exists() {
            return Err(format!(
                "refuse to forget {} while worktree still exists: {}",
                id, record.worktree
            ));
        }
        let path = record_path(cwd, id)?;
        fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))
    })();
    match result {
        Ok(()) => (0, format!("FORGOT lane metadata: {id}\n")),
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

fn parse_worktrees(raw: &str) -> Vec<WorktreeInfo> {
    let mut out = Vec::new();
    let mut current: Option<WorktreeInfo> = None;
    for line in raw.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if let Some(item) = current.take() {
                out.push(item);
            }
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(item) = current.take() {
                out.push(item);
            }
            current = Some(WorktreeInfo {
                path: PathBuf::from(path),
                head: String::new(),
                branch: None,
                detached: false,
                locked: false,
                prunable: false,
            });
        } else if let Some(item) = current.as_mut() {
            if let Some(head) = line.strip_prefix("HEAD ") {
                item.head = head.to_string();
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                item.branch = Some(branch.to_string());
            } else if line == "detached" {
                item.detached = true;
            } else if line.starts_with("locked") {
                item.locked = true;
            } else if line.starts_with("prunable") {
                item.prunable = true;
            }
        }
    }
    out
}

fn nul_paths(cwd: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(output
        .stdout
        .split(|b| *b == 0)
        .filter(|v| !v.is_empty())
        .map(|v| String::from_utf8_lossy(v).to_string())
        .collect())
}

fn changed_files(path: &Path, base: &str) -> Result<Vec<String>, String> {
    let mut files = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", "-z", &format!("{base}...HEAD")],
        vec!["diff", "--name-only", "-z"],
        vec!["diff", "--cached", "--name-only", "-z"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        for file in nul_paths(path, &args)? {
            files.insert(file);
        }
    }
    Ok(files.into_iter().collect())
}

fn count_revs(path: &Path, range: &str) -> Option<u64> {
    git(path, &["rev-list", "--count", range])
        .ok()
        .and_then(|v| v.parse().ok())
}

fn is_ancestor(path: &Path, ancestor: &str, descendant: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn build_report(repo: &Path) -> Result<AuditReport, String> {
    let root = repo_root(repo)?;
    let raw = git(&root, &["worktree", "list", "--porcelain"])?;
    let worktrees = parse_worktrees(&raw);
    let records = load_records(&root)?;
    let record_by_path: BTreeMap<String, &LaneRecord> =
        records.iter().map(|r| (r.worktree.clone(), r)).collect();
    let present_paths: BTreeSet<String> = worktrees
        .iter()
        .map(|w| {
            fs::canonicalize(&w.path)
                .unwrap_or_else(|_| w.path.clone())
                .display()
                .to_string()
        })
        .collect();
    let worktree_by_path: BTreeMap<String, &WorktreeInfo> = worktrees
        .iter()
        .map(|worktree| {
            (
                fs::canonicalize(&worktree.path)
                    .unwrap_or_else(|_| worktree.path.clone())
                    .display()
                    .to_string(),
                worktree,
            )
        })
        .collect();
    let primary_path = worktrees.first().map(|w| {
        fs::canonicalize(&w.path)
            .unwrap_or_else(|_| w.path.clone())
            .display()
            .to_string()
    });
    let mut unregistered = Vec::new();
    let inferred_base = default_base(&root).ok();
    for wt in worktrees.iter().skip(1) {
        let path = fs::canonicalize(&wt.path)
            .unwrap_or_else(|_| wt.path.clone())
            .display()
            .to_string();
        if !record_by_path.contains_key(&path) {
            let dirty = git(&wt.path, &["status", "--porcelain"])
                .map(|v| !v.is_empty())
                .unwrap_or(true);
            let (behind, unique) = inferred_base
                .as_deref()
                .map(|base| {
                    (
                        count_revs(&wt.path, &format!("HEAD..{base}")),
                        count_revs(&wt.path, &format!("{base}..HEAD")),
                    )
                })
                .unwrap_or((None, None));
            let reclaim = if dirty || unique.unwrap_or(0) > 0 {
                "rescue"
            } else {
                "review"
            };
            let mut flags = vec![
                format!(
                    "branch={}",
                    wt.branch
                        .as_deref()
                        .unwrap_or(if wt.detached { "DETACHED" } else { "?" })
                ),
                format!("head={}", &wt.head[..wt.head.len().min(12)]),
                format!("dirty={dirty}"),
                format!("base={}", inferred_base.as_deref().unwrap_or("?")),
                format!(
                    "behind={}",
                    behind
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "?".to_string())
                ),
                format!(
                    "unique={}",
                    unique
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "?".to_string())
                ),
                format!("reclaim={reclaim}"),
            ];
            flags.extend(
                [
                    wt.detached.then_some("detached"),
                    wt.locked.then_some("locked"),
                    wt.prunable.then_some("prunable"),
                ]
                .into_iter()
                .flatten()
                .map(str::to_string),
            );
            unregistered.push(format!("{path} ({})", flags.join(",")));
        }
    }

    let live: Vec<&LaneRecord> = records
        .iter()
        .filter(|r| ownership_live(&r.status))
        .collect();
    let mut overlaps = Vec::new();
    for (i, a) in live.iter().enumerate() {
        for b in live.iter().skip(i + 1) {
            for aa in &a.owns {
                for bb in &b.owns {
                    if boundaries_overlap(aa, bb) {
                        overlaps.push(format!("{}:{} <-> {}:{}", a.id, aa, b.id, bb));
                    }
                }
            }
        }
    }

    let statuses: BTreeMap<&str, &str> = records
        .iter()
        .map(|r| (r.id.as_str(), r.status.as_str()))
        .collect();
    let mut dependency_blocks = Vec::new();
    for lane in &records {
        if !ownership_live(&lane.status) {
            continue;
        }
        for dep in &lane.depends_on {
            match statuses.get(dep.as_str()) {
                Some(status) if *status == "landed" => {}
                Some(status) => {
                    dependency_blocks.push(format!("{} waits for {} ({})", lane.id, dep, status))
                }
                None => {
                    dependency_blocks.push(format!("{} references missing lane {}", lane.id, dep))
                }
            }
        }
    }

    let mut audits = Vec::new();
    let mut errors = Vec::new();
    for record in &records {
        let path = PathBuf::from(&record.worktree);
        let present = present_paths.contains(&record.worktree);
        let worktree_info = worktree_by_path.get(&record.worktree).copied();
        let locked = worktree_info
            .map(|worktree| worktree.locked)
            .unwrap_or(false);
        let prunable = worktree_info
            .map(|worktree| worktree.prunable)
            .unwrap_or(false);
        let files = if present {
            match changed_files(&path, &record.base) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(format!(
                        "{}: cannot compare with {}: {}",
                        record.id, record.base, e
                    ));
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        let out_of_bounds = files
            .iter()
            .filter(|f| !owns_path(&record.owns, f))
            .cloned()
            .collect::<Vec<_>>();
        let base_ahead = present
            .then(|| count_revs(&path, &format!("HEAD..{}", record.base)))
            .flatten();
        let unique_commits = present
            .then(|| count_revs(&path, &format!("{}..HEAD", record.base)))
            .flatten();
        let clean = present
            && git(&path, &["status", "--porcelain"])
                .map(|v| v.is_empty())
                .unwrap_or(false);
        let merged = present && is_ancestor(&path, "HEAD", &record.base);
        let reclaim = if primary_path.as_deref() == Some(record.worktree.as_str()) {
            "primary".to_string()
        } else if !present {
            if matches!(record.status.as_str(), "landed" | "parked") {
                "metadata".to_string()
            } else {
                "review-missing".to_string()
            }
        } else if locked || prunable {
            "review".to_string()
        } else if record.status == "landed" && clean && merged {
            "safe".to_string()
        } else if !clean
            || !out_of_bounds.is_empty()
            || (!merged && unique_commits.unwrap_or(0) > 0)
        {
            "rescue".to_string()
        } else {
            "review".to_string()
        };
        if !present && ownership_live(&record.status) {
            errors.push(format!(
                "{} is {} but its worktree is missing",
                record.id, record.status
            ));
        }
        audits.push(LaneAudit {
            id: record.id.clone(),
            goal: record.goal.clone(),
            worktree: record.worktree.clone(),
            branch: record.branch.clone(),
            base: record.base.clone(),
            status: record.status.clone(),
            owns: record.owns.clone(),
            depends_on: record.depends_on.clone(),
            changed_files: files,
            out_of_bounds,
            base_ahead,
            unique_commits,
            clean,
            locked,
            prunable,
            reclaim,
            present,
        });
    }
    Ok(AuditReport {
        repo: root.display().to_string(),
        lanes: audits,
        unregistered_worktrees: unregistered,
        overlaps,
        dependency_blocks,
        errors,
    })
}

fn report_has_failures(report: &AuditReport) -> bool {
    !report.unregistered_worktrees.is_empty()
        || !report.overlaps.is_empty()
        || !report.errors.is_empty()
        || report
            .lanes
            .iter()
            .any(|lane| !lane.out_of_bounds.is_empty())
}

fn render_text(report: &AuditReport) -> String {
    let mut out = format!("WORKTREE CONTROL PLANE: {}\n", report.repo);
    if report.lanes.is_empty() {
        out.push_str("lanes: none\n");
    }
    for lane in &report.lanes {
        out.push_str(&format!(
            "- {} [{}] {} | base {} behind {} | changed {} | locked {} | reclaim {}\n  goal: {}\n  owns: {}\n",
            lane.id,
            lane.status,
            lane.branch,
            lane.base,
            lane.base_ahead
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string()),
            lane.changed_files.len(),
            lane.locked,
            lane.reclaim,
            lane.goal,
            lane.owns.join(", ")
        ));
        if !lane.out_of_bounds.is_empty() {
            out.push_str(&format!(
                "  OUT-OF-BOUNDS: {}\n",
                lane.out_of_bounds.join(", ")
            ));
        }
    }
    for path in &report.unregistered_worktrees {
        out.push_str(&format!("UNREGISTERED: {path}\n"));
    }
    for item in &report.overlaps {
        out.push_str(&format!("OVERLAP: {item}\n"));
    }
    for item in &report.dependency_blocks {
        out.push_str(&format!("WAIT: {item}\n"));
    }
    for item in &report.errors {
        out.push_str(&format!("ERROR: {item}\n"));
    }
    out.push_str(if report_has_failures(report) {
        "RESULT: FAIL\n"
    } else {
        "RESULT: PASS\n"
    });
    out
}

pub fn run_audit(repo: &Path, json: bool, strict: bool) -> (i32, String) {
    match build_report(repo) {
        Ok(report) => {
            let failed = report_has_failures(&report);
            let output = if json {
                serde_json::to_string_pretty(&report)
                    .map(|v| format!("{v}\n"))
                    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}\n"))
            } else {
                render_text(&report)
            };
            (if strict && failed { 1 } else { 0 }, output)
        }
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

fn git_text(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end_matches(['\r', '\n'])
            .to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        })
    }
}

fn count_revs_result(path: &Path, range: &str) -> Result<u64, String> {
    git(path, &["rev-list", "--count", range])?
        .parse::<u64>()
        .map_err(|e| format!("invalid rev-list count for {range}: {e}"))
}

fn ancestor_result(path: &Path, ancestor: &str, descendant: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) if output.stderr.is_empty() => Ok(false),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("cannot compare {ancestor} to {descendant}")
            } else {
                stderr
            })
        }
    }
}

fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.blocks().saturating_mul(512)
    }
    #[cfg(not(unix))]
    {
        metadata.len()
    }
}

#[cfg(unix)]
fn file_identity(metadata: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn file_identity(_metadata: &fs::Metadata) -> Option<(u64, u64)> {
    None
}

fn scan_tree(path: &Path) -> Result<(u64, SystemTime), String> {
    let mut total = 0_u64;
    let mut latest = UNIX_EPOCH;
    let mut seen = BTreeSet::new();
    let mut stack = vec![path.to_path_buf()];
    while let Some(item) = stack.pop() {
        let metadata =
            fs::symlink_metadata(&item).map_err(|e| format!("scan {}: {e}", item.display()))?;
        if let Some(identity) = file_identity(&metadata) {
            if !seen.insert(identity) {
                continue;
            }
        }
        total = total.saturating_add(allocated_bytes(&metadata));
        if let Ok(modified) = metadata.modified() {
            latest = latest.max(modified);
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&item).map_err(|e| format!("scan {}: {e}", item.display()))? {
                stack.push(
                    entry
                        .map_err(|e| format!("scan {} entry: {e}", item.display()))?
                        .path(),
                );
            }
        }
    }
    Ok((total, latest))
}

fn worktree_git_dir(path: &Path) -> Result<PathBuf, String> {
    if let Ok(raw) = git(path, &["rev-parse", "--path-format=absolute", "--git-dir"]) {
        return Ok(PathBuf::from(raw));
    }
    let raw = PathBuf::from(git(path, &["rev-parse", "--git-dir"])?);
    let resolved = if raw.is_absolute() {
        raw
    } else {
        path.join(raw)
    };
    Ok(fs::canonicalize(&resolved).unwrap_or(resolved))
}

fn scan_worktree(path: &Path, include_git_activity: bool) -> Result<(u64, SystemTime), String> {
    let (bytes, mut latest) = scan_tree(path)?;
    if include_git_activity {
        let git_dir = worktree_git_dir(path)?;
        let (_, git_latest) = scan_tree(&git_dir)?;
        latest = latest.max(git_latest);
    }
    Ok((bytes, latest))
}

fn query_all_prs(root: &Path) -> GhInventory {
    const FIELDS: &str = "number,state,headRefName,baseRefName,headRefOid,url,mergedAt";
    let command = format!("gh pr list --state all --limit 1000 --json {FIELDS}");
    let output = Command::new("gh")
        .current_dir(root)
        .env("GH_PROMPT_DISABLED", "1")
        .args([
            "pr", "list", "--state", "all", "--limit", "1000", "--json", FIELDS,
        ])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            match serde_json::from_slice::<Vec<GhPrRow>>(&output.stdout) {
                Ok(rows) => GhInventory {
                    audit: GcPrQueryAudit {
                        status: "ok".to_string(),
                        command,
                        error: None,
                    },
                    rows: Some(rows),
                },
                Err(e) => GhInventory {
                    audit: GcPrQueryAudit {
                        status: "unknown".to_string(),
                        command,
                        error: Some(format!("cannot parse gh output: {e}")),
                    },
                    rows: None,
                },
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            GhInventory {
                audit: GcPrQueryAudit {
                    status: "unknown".to_string(),
                    command,
                    error: Some(if stderr.is_empty() {
                        format!("gh exited with {}", output.status)
                    } else {
                        stderr
                    }),
                },
                rows: None,
            }
        }
        Err(e) => GhInventory {
            audit: GcPrQueryAudit {
                status: "unknown".to_string(),
                command,
                error: Some(format!("cannot run gh: {e}")),
            },
            rows: None,
        },
    }
}

fn pr_for_worktree(
    path: &Path,
    branch: Option<&str>,
    head: &str,
    expected_base: Option<&str>,
    rows: Option<&[GhPrRow]>,
) -> (PrKind, GcPrAudit, bool) {
    let Some(rows) = rows else {
        return (
            PrKind::Unknown,
            GcPrAudit {
                status: "unknown".to_string(),
                number: None,
                url: None,
                base_ref_name: None,
                head_ref_oid: None,
                covers_head: None,
            },
            false,
        );
    };
    let Some(branch) = branch else {
        return (
            PrKind::Unknown,
            GcPrAudit {
                status: "unknown".to_string(),
                number: None,
                url: None,
                base_ref_name: None,
                head_ref_oid: None,
                covers_head: None,
            },
            false,
        );
    };
    let mut matched: Vec<&GhPrRow> = rows
        .iter()
        .filter(|row| row.head_ref_name == branch)
        .collect();
    matched.sort_by_key(|row| std::cmp::Reverse(row.number));
    if let Some(row) = matched
        .iter()
        .copied()
        .find(|row| row.state.eq_ignore_ascii_case("OPEN"))
    {
        return (
            PrKind::Open,
            GcPrAudit {
                status: "open".to_string(),
                number: Some(row.number),
                url: Some(row.url.clone()),
                base_ref_name: Some(row.base_ref_name.clone()),
                head_ref_oid: row.head_ref_oid.clone(),
                covers_head: None,
            },
            false,
        );
    }
    let target_matches: Vec<&GhPrRow> = matched
        .iter()
        .copied()
        .filter(|row| expected_base == Some(row.base_ref_name.as_str()))
        .collect();
    let merged: Vec<&GhPrRow> = target_matches
        .iter()
        .copied()
        .filter(|row| row.merged_at.is_some() || row.state.eq_ignore_ascii_case("MERGED"))
        .collect();
    if !merged.is_empty() {
        let mut unknown = None;
        let mut failed = None;
        for row in merged {
            let coverage = row.head_ref_oid.as_deref().and_then(|oid| {
                if head == oid {
                    Some(true)
                } else {
                    ancestor_result(path, "HEAD", oid).ok()
                }
            });
            let audit = GcPrAudit {
                status: "merged".to_string(),
                number: Some(row.number),
                url: Some(row.url.clone()),
                base_ref_name: Some(row.base_ref_name.clone()),
                head_ref_oid: row.head_ref_oid.clone(),
                covers_head: coverage,
            };
            match coverage {
                Some(true) => return (PrKind::Merged, audit, false),
                Some(false) => failed.get_or_insert(audit),
                None => unknown.get_or_insert(audit),
            };
        }
        if let Some(audit) = unknown {
            return (PrKind::Merged, audit, false);
        }
        return (PrKind::Merged, failed.expect("merged PR exists"), true);
    }
    if let Some(row) = target_matches.first() {
        return (
            PrKind::Closed,
            GcPrAudit {
                status: "closed".to_string(),
                number: Some(row.number),
                url: Some(row.url.clone()),
                base_ref_name: Some(row.base_ref_name.clone()),
                head_ref_oid: row.head_ref_oid.clone(),
                covers_head: None,
            },
            false,
        );
    }
    if let Some(row) = matched.first() {
        return (
            PrKind::Unknown,
            GcPrAudit {
                status: "other-base".to_string(),
                number: Some(row.number),
                url: Some(row.url.clone()),
                base_ref_name: Some(row.base_ref_name.clone()),
                head_ref_oid: row.head_ref_oid.clone(),
                covers_head: None,
            },
            false,
        );
    }
    (
        PrKind::None,
        GcPrAudit {
            status: "none".to_string(),
            number: None,
            url: None,
            base_ref_name: None,
            head_ref_oid: None,
            covers_head: None,
        },
        false,
    )
}

fn integrated_criterion(
    head_ancestor: Option<bool>,
    pr_kind: PrKind,
    pr: &GcPrAudit,
) -> GcCriterion {
    if head_ancestor == Some(true) {
        return GcCriterion {
            result: CheckState::Pass,
            evidence: "HEAD is an ancestor of base".to_string(),
        };
    }
    match (pr_kind, pr.covers_head) {
        (PrKind::Merged, Some(true)) => GcCriterion {
            result: CheckState::Pass,
            evidence: "MERGED PR headRefOid covers current HEAD (squash-safe)".to_string(),
        },
        (PrKind::Merged, Some(false)) => GcCriterion {
            result: CheckState::Fail,
            evidence: "MERGED PR headRefOid does not cover current HEAD".to_string(),
        },
        (PrKind::Merged, None) => GcCriterion {
            result: CheckState::Unknown,
            evidence: "MERGED PR found but headRefOid coverage is unknown".to_string(),
        },
        (PrKind::Open, _) => GcCriterion {
            result: CheckState::Fail,
            evidence: "PR is still open".to_string(),
        },
        (PrKind::Closed, _) => GcCriterion {
            result: CheckState::Fail,
            evidence: "PR was closed without merge".to_string(),
        },
        (PrKind::None, _) if head_ancestor == Some(false) => GcCriterion {
            result: CheckState::Fail,
            evidence: "no PR and HEAD is not in base".to_string(),
        },
        (PrKind::Unknown, _) | (PrKind::None, _) => GcCriterion {
            result: CheckState::Unknown,
            evidence: "integration cannot be proven".to_string(),
        },
    }
}

fn preserved_criterion(
    upstream: Option<&str>,
    unpushed: Option<u64>,
    head_ancestor: Option<bool>,
    unique: Option<u64>,
    pr: &GcPrAudit,
) -> GcCriterion {
    match unpushed {
        Some(0) => GcCriterion {
            result: CheckState::Pass,
            evidence: format!("no commits ahead of {}", upstream.unwrap_or("upstream")),
        },
        Some(n) => GcCriterion {
            result: CheckState::Fail,
            evidence: format!("{n} unpushed commit(s)"),
        },
        None if head_ancestor == Some(true) && unique == Some(0) => GcCriterion {
            result: CheckState::Pass,
            evidence: "no upstream, but HEAD is in base with no unique commits".to_string(),
        },
        None if pr.status == "merged" && pr.covers_head == Some(true) => GcCriterion {
            result: CheckState::Pass,
            evidence: "no upstream, but MERGED PR headRefOid preserves HEAD".to_string(),
        },
        None if unique.unwrap_or(0) > 0 => GcCriterion {
            result: CheckState::Fail,
            evidence: format!(
                "no usable upstream and {} unique commit(s)",
                unique.unwrap_or(0)
            ),
        },
        None => GcCriterion {
            result: CheckState::Unknown,
            evidence: "upstream/unpushed state cannot be proven safe".to_string(),
        },
    }
}

fn decide_gc(facts: DecisionFacts) -> (String, Vec<String>) {
    let mut reasons = Vec::new();
    if facts.primary {
        reasons.push("primary_worktree".to_string());
        return ("primary".to_string(), reasons);
    }
    if facts.prunable {
        reasons.push("prunable_metadata_requires_review".to_string());
        return ("review".to_string(), reasons);
    }
    if facts.clean == CheckState::Fail {
        reasons.push("dirty_worktree".to_string());
    }
    if facts.post_merge_local {
        reasons.push("post_merge_local_commits".to_string());
    }
    if facts.preserved == CheckState::Fail {
        reasons.push("unpreserved_or_unpushed_work".to_string());
    }
    if facts.pr == PrKind::None
        && facts.integrated == CheckState::Fail
        && facts.unique_commits.unwrap_or(0) > 0
    {
        reasons.push("no_pr_unique_work_not_in_base".to_string());
    }
    if !reasons.is_empty() {
        return ("rescue".to_string(), reasons);
    }
    if facts.locked {
        reasons.push("locked_worktree".to_string());
        return ("keep".to_string(), reasons);
    }
    if facts.live_lane {
        reasons.push("live_lane".to_string());
        return ("keep".to_string(), reasons);
    }
    if facts.pr == PrKind::Open {
        reasons.push("open_pr".to_string());
        return ("keep".to_string(), reasons);
    }
    if facts.quiet == Some(false) {
        reasons.push("inside_quiet_window".to_string());
        return ("keep".to_string(), reasons);
    }
    if facts.pr == PrKind::Closed {
        reasons.push("closed_without_merge".to_string());
        return ("review".to_string(), reasons);
    }
    if facts.pr == PrKind::Unknown {
        reasons.push("pr_query_unknown".to_string());
        return ("review".to_string(), reasons);
    }
    if facts.integrated == CheckState::Unknown
        || facts.preserved == CheckState::Unknown
        || facts.clean == CheckState::Unknown
        || facts.quiet.is_none()
    {
        reasons.push("required_evidence_unknown".to_string());
        return ("review".to_string(), reasons);
    }
    if facts.integrated == CheckState::Pass
        && facts.preserved == CheckState::Pass
        && facts.clean == CheckState::Pass
        && facts.quiet == Some(true)
        && matches!(facts.pr, PrKind::None | PrKind::Merged)
    {
        reasons.push("all_reclaim_gates_pass".to_string());
        return ("candidate".to_string(), reasons);
    }
    reasons.push("reclaim_gates_not_all_pass".to_string());
    ("review".to_string(), reasons)
}

fn hosted_base_name(root: &Path, base: &str) -> Option<String> {
    let full = git(root, &["rev-parse", "--symbolic-full-name", base]).ok()?;
    if let Some(name) = full.strip_prefix("refs/heads/") {
        return Some(name.to_string());
    }
    let remote = full.strip_prefix("refs/remotes/")?;
    remote.split_once('/').map(|(_, name)| name.to_string())
}

fn build_gc_report(repo: &Path, opts: &GcOpts) -> Result<GcReport, String> {
    let root = repo_root(repo)?;
    let base = match &opts.base {
        Some(base) => base.clone(),
        None => default_base(&root)?,
    };
    git(&root, &["rev-parse", "--verify", &base])?;
    let expected_pr_base = hosted_base_name(&root, &base);
    let worktrees = parse_worktrees(&git(&root, &["worktree", "list", "--porcelain"])?);
    let records = load_records(&root)?;
    let records_by_path: BTreeMap<String, &LaneRecord> = records
        .iter()
        .map(|record| (record.worktree.clone(), record))
        .collect();
    let gh = query_all_prs(&root);
    let mut audits = Vec::new();
    let mut candidates = Vec::new();
    for (index, worktree) in worktrees.iter().enumerate() {
        let canonical = fs::canonicalize(&worktree.path).unwrap_or_else(|_| worktree.path.clone());
        let path_string = canonical.display().to_string();
        let lane = records_by_path.get(&path_string).copied();
        let mut errors = Vec::new();

        let dirty = if worktree.path.exists() {
            match git_text(
                &worktree.path,
                &["status", "--porcelain=v1", "--untracked-files=all"],
            ) {
                Ok(raw) => raw.lines().map(str::to_string).collect::<Vec<_>>(),
                Err(e) => {
                    errors.push(format!("dirty status: {e}"));
                    Vec::new()
                }
            }
        } else {
            errors.push("worktree path is missing".to_string());
            Vec::new()
        };
        let clean = if !worktree.path.exists()
            || errors
                .iter()
                .any(|error| error.starts_with("dirty status:"))
        {
            GcCriterion {
                result: CheckState::Unknown,
                evidence: "git status unavailable".to_string(),
            }
        } else if dirty.is_empty() {
            GcCriterion {
                result: CheckState::Pass,
                evidence: "git status --porcelain is empty".to_string(),
            }
        } else {
            GcCriterion {
                result: CheckState::Fail,
                evidence: format!("{} raw dirty item(s)", dirty.len()),
            }
        };

        let (du_bytes, quiet) = if worktree.path.exists() {
            match scan_worktree(&worktree.path, index > 0) {
                Ok((bytes, latest)) => {
                    let elapsed = SystemTime::now().duration_since(latest).unwrap_or_default();
                    let hours = elapsed.as_secs_f64() / 3600.0;
                    (
                        Some(bytes),
                        GcQuietAudit {
                            required_hours: opts.quiet_hours,
                            pass: Some(opts.quiet_hours == 0 || hours >= opts.quiet_hours as f64),
                            last_activity_at: Some(
                                chrono::DateTime::<Utc>::from(latest).to_rfc3339(),
                            ),
                            hours_since_activity: Some(hours),
                        },
                    )
                }
                Err(e) => {
                    errors.push(format!("du/activity scan: {e}"));
                    (
                        None,
                        GcQuietAudit {
                            required_hours: opts.quiet_hours,
                            pass: None,
                            last_activity_at: None,
                            hours_since_activity: None,
                        },
                    )
                }
            }
        } else {
            (
                None,
                GcQuietAudit {
                    required_hours: opts.quiet_hours,
                    pass: None,
                    last_activity_at: None,
                    hours_since_activity: None,
                },
            )
        };

        let head_ancestor = if worktree.path.exists() {
            match ancestor_result(&worktree.path, "HEAD", &base) {
                Ok(value) => Some(value),
                Err(e) => {
                    errors.push(format!("base ancestor: {e}"));
                    None
                }
            }
        } else {
            None
        };
        let unique = if worktree.path.exists() {
            match count_revs_result(&worktree.path, &format!("{base}..HEAD")) {
                Ok(value) => Some(value),
                Err(e) => {
                    errors.push(format!("unique commits: {e}"));
                    None
                }
            }
        } else {
            None
        };
        let upstream = if worktree.path.exists() {
            git(
                &worktree.path,
                &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
            )
            .ok()
        } else {
            None
        };
        let unpushed = match upstream.as_deref() {
            Some(upstream) => {
                match count_revs_result(&worktree.path, &format!("{upstream}..HEAD")) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        errors.push(format!("unpushed commits: {e}"));
                        None
                    }
                }
            }
            None => None,
        };
        let (pr_kind, pr, post_merge_local) = pr_for_worktree(
            &worktree.path,
            worktree.branch.as_deref(),
            &worktree.head,
            expected_pr_base.as_deref(),
            gh.rows.as_deref(),
        );
        let integrated = integrated_criterion(head_ancestor, pr_kind, &pr);
        let preserved =
            preserved_criterion(upstream.as_deref(), unpushed, head_ancestor, unique, &pr);
        let (decision, reasons) = decide_gc(DecisionFacts {
            primary: index == 0,
            locked: worktree.locked,
            prunable: worktree.prunable,
            live_lane: lane
                .map(|record| ownership_live(&record.status))
                .unwrap_or(false),
            pr: pr_kind,
            post_merge_local,
            integrated: integrated.result,
            preserved: preserved.result,
            clean: clean.result,
            quiet: quiet.pass,
            unique_commits: unique,
        });
        if decision == "candidate" {
            candidates.push(path_string.clone());
        }
        audits.push(GcWorktreeAudit {
            path: path_string,
            branch: worktree.branch.clone(),
            head: worktree.head.clone(),
            primary: index == 0,
            locked: worktree.locked,
            prunable: worktree.prunable,
            lane_status: lane.map(|record| record.status.clone()),
            du_bytes,
            dirty_entries: dirty,
            base: GcBaseAudit {
                reference: base.clone(),
                head_ancestor_of_base: head_ancestor,
                unique_commits: unique,
            },
            upstream,
            unpushed_commits: unpushed,
            pr,
            criteria: GcCriteria {
                integrated,
                preserved,
                clean,
            },
            quiet,
            decision,
            reasons,
            errors,
        });
    }
    Ok(GcReport {
        repo: root.display().to_string(),
        mode: "dry-run".to_string(),
        base,
        quiet_hours: opts.quiet_hours,
        github_pr_query: gh.audit,
        worktrees: audits,
        candidates,
        errors: Vec::new(),
    })
}

fn render_gc_text(report: &GcReport) -> String {
    let mut out = format!(
        "WORKTREE GC DRY-RUN: {}\nbase: {} | quiet: {}h | gh: {}\n",
        report.repo, report.base, report.quiet_hours, report.github_pr_query.status
    );
    if let Some(error) = &report.github_pr_query.error {
        out.push_str(&format!("GH UNKNOWN: {error}\n"));
    }
    for worktree in &report.worktrees {
        out.push_str(&format!(
            "- {} {} | branch={} | head={}\n",
            worktree.decision.to_uppercase(),
            worktree.path,
            worktree.branch.as_deref().unwrap_or("DETACHED"),
            &worktree.head[..worktree.head.len().min(12)]
        ));
        out.push_str(&format!(
            "  flags: primary={} locked={} prunable={} lane={} du_bytes={}\n",
            worktree.primary,
            worktree.locked,
            worktree.prunable,
            worktree.lane_status.as_deref().unwrap_or("unregistered"),
            worktree
                .du_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "?".to_string())
        ));
        out.push_str(&format!(
            "  base: {} head_ancestor={} unique={} | upstream={} unpushed={}\n",
            worktree.base.reference,
            worktree
                .base
                .head_ancestor_of_base
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            worktree
                .base
                .unique_commits
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            worktree.upstream.as_deref().unwrap_or("none"),
            worktree
                .unpushed_commits
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
        out.push_str(&format!(
            "  pr: {} number={} base={} headRefOid={} covers_head={}\n",
            worktree.pr.status,
            worktree
                .pr
                .number
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            worktree.pr.base_ref_name.as_deref().unwrap_or("none"),
            worktree.pr.head_ref_oid.as_deref().unwrap_or("none"),
            worktree
                .pr
                .covers_head
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
        out.push_str(&format!(
            "  criteria: integrated={:?} preserved={:?} clean={:?} | quiet={}\n",
            worktree.criteria.integrated.result,
            worktree.criteria.preserved.result,
            worktree.criteria.clean.result,
            worktree
                .quiet
                .pass
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
        if worktree.dirty_entries.is_empty() {
            out.push_str("  dirty: none\n");
        } else {
            out.push_str("  dirty raw:\n");
            for entry in &worktree.dirty_entries {
                out.push_str(&format!("    {entry}\n"));
            }
        }
        out.push_str(&format!("  reasons: {}\n", worktree.reasons.join(", ")));
        for error in &worktree.errors {
            out.push_str(&format!("  ERROR: {error}\n"));
        }
    }
    out.push_str(&format!("CANDIDATES ({}):\n", report.candidates.len()));
    for path in &report.candidates {
        out.push_str(&format!("  {path}\n"));
    }
    out.push_str("READ-ONLY: no worktree, branch, lane, or report file was changed.\n");
    out
}

pub fn run_gc(repo: &Path, opts: &GcOpts) -> (i32, String) {
    if !opts.dry_run {
        return (
            2,
            "ERROR: `agent-on worktree gc` is read-only and requires --dry-run; no apply/delete mode exists\n"
                .to_string(),
        );
    }
    match build_gc_report(repo, opts) {
        Ok(report) => {
            let output = if opts.json {
                serde_json::to_string_pretty(&report)
                    .map(|value| format!("{value}\n"))
                    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}\n"))
            } else {
                render_gc_text(&report)
            };
            (0, output)
        }
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn fixture() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        fs::create_dir_all(&root).unwrap();
        run(&root, &["git", "init", "-b", "main"]);
        run(&root, &["git", "config", "user.email", "test@example.com"]);
        run(&root, &["git", "config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "root\n").unwrap();
        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(root.join("app/base.txt"), "base\n").unwrap();
        run(&root, &["git", "add", "."]);
        run(&root, &["git", "commit", "-m", "init"]);
        let wt = tmp.path().join("lane-a");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "lane/a",
                wt.to_str().unwrap(),
                "main",
            ],
        );
        (tmp, root, wt)
    }

    #[test]
    fn boundary_matching_uses_path_segments() {
        assert!(boundary_contains("app", "app/page.rs"));
        assert!(boundary_contains("app/page.rs", "app/page.rs"));
        assert!(!boundary_contains("app", "apple/page.rs"));
        assert!(boundaries_overlap("app", "app/pages"));
        assert!(!boundaries_overlap("api", "app"));
    }

    #[test]
    fn parses_porcelain_worktree_inventory() {
        let rows = parse_worktrees(
            "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\nworktree /tmp/lane\nHEAD def\ndetached\nlocked reason\n\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].branch.as_deref(), Some("main"));
        assert!(rows[1].detached);
        assert!(rows[1].locked);
    }

    #[test]
    fn claim_and_check_catch_out_of_boundary_change() {
        let (_tmp, root, wt) = fixture();
        let (code, out) = claim_lane(
            &wt,
            &ClaimOpts {
                id: "lane-a".to_string(),
                goal: "change app".to_string(),
                base: Some("main".to_string()),
                owns: vec!["app/".to_string()],
                depends_on: Vec::new(),
            },
        );
        assert_eq!(code, 0, "{out}");
        fs::write(wt.join("README.md"), "escaped\n").unwrap();
        let (code, out) = run_audit(&root, false, true);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("OUT-OF-BOUNDS: README.md"), "{out}");
    }

    #[test]
    fn claimed_clean_worktree_passes_strict_check() {
        let (_tmp, root, wt) = fixture();
        let opts = ClaimOpts {
            id: "lane-a".to_string(),
            goal: "change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(claim_lane(&wt, &opts).0, 0);
        let (code, out) = run_audit(&root, false, true);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("RESULT: PASS"), "{out}");
    }

    #[test]
    fn second_claim_cannot_overlap_live_lane() {
        let (tmp, root, wt) = fixture();
        let first = ClaimOpts {
            id: "lane-a".to_string(),
            goal: "change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(claim_lane(&wt, &first).0, 0);
        let wt_b = tmp.path().join("lane-b");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "lane/b",
                wt_b.to_str().unwrap(),
                "main",
            ],
        );
        let second = ClaimOpts {
            id: "lane-b".to_string(),
            goal: "also change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app/pages".to_string()],
            depends_on: Vec::new(),
        };
        let (code, out) = claim_lane(&wt_b, &second);
        assert_eq!(code, 1);
        assert!(out.contains("overlaps live lane lane-a"), "{out}");
    }

    #[test]
    fn ready_requires_clean_in_boundary_worktree() {
        let (_tmp, _root, wt) = fixture();
        let opts = ClaimOpts {
            id: "lane-a".to_string(),
            goal: "change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(claim_lane(&wt, &opts).0, 0);
        fs::write(wt.join("app/dirty.txt"), "dirty\n").unwrap();
        let (code, out) = set_lane_status(&wt, None, "ready");
        assert_eq!(code, 1);
        assert!(out.contains("ready requires a clean worktree"), "{out}");
    }

    #[test]
    fn primary_worktree_is_never_reclaimable() {
        let (_tmp, root, _wt) = fixture();
        let opts = ClaimOpts {
            id: "control".to_string(),
            goal: "coordinate".to_string(),
            base: Some("main".to_string()),
            owns: vec!["README.md".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(claim_lane(&root, &opts).0, 0);
        assert_eq!(set_lane_status(&root, None, "ready").0, 0);
        assert_eq!(set_lane_status(&root, None, "landed").0, 0);
        let report = build_report(&root).unwrap();
        let control = report
            .lanes
            .iter()
            .find(|lane| lane.id == "control")
            .unwrap();
        assert_eq!(control.reclaim, "primary");
    }

    #[test]
    fn locked_registered_worktree_is_never_safe() {
        let (_tmp, root, wt) = fixture();
        let opts = ClaimOpts {
            id: "lane-a".to_string(),
            goal: "change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(claim_lane(&wt, &opts).0, 0);
        assert_eq!(set_lane_status(&wt, None, "ready").0, 0);
        assert_eq!(set_lane_status(&wt, None, "landed").0, 0);
        run(&root, &["git", "worktree", "lock", wt.to_str().unwrap()]);

        let report = build_report(&root).unwrap();
        let lane = report
            .lanes
            .iter()
            .find(|lane| lane.id == "lane-a")
            .unwrap();
        assert!(lane.locked);
        assert_ne!(lane.reclaim, "safe");
        assert_eq!(lane.reclaim, "review");
    }

    fn safe_decision_facts() -> DecisionFacts {
        DecisionFacts {
            primary: false,
            locked: false,
            prunable: false,
            live_lane: false,
            pr: PrKind::None,
            post_merge_local: false,
            integrated: CheckState::Pass,
            preserved: CheckState::Pass,
            clean: CheckState::Pass,
            quiet: Some(true),
            unique_commits: Some(0),
        }
    }

    #[test]
    fn gc_requires_explicit_dry_run_before_touching_repo() {
        let opts = GcOpts {
            dry_run: false,
            json: true,
            base: None,
            quiet_hours: 24,
        };
        let (code, out) = run_gc(Path::new("/path/that/does/not/exist"), &opts);
        assert_eq!(code, 2, "{out}");
        assert!(out.contains("requires --dry-run"), "{out}");
        assert!(out.contains("no apply/delete mode exists"), "{out}");
    }

    #[test]
    fn gc_locked_worktree_is_kept() {
        let mut facts = safe_decision_facts();
        facts.locked = true;
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "keep");
        assert_eq!(reasons, ["locked_worktree"]);
    }

    #[test]
    fn gc_prunable_worktree_requires_review() {
        let mut facts = safe_decision_facts();
        facts.prunable = true;
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "review");
        assert_eq!(reasons, ["prunable_metadata_requires_review"]);
    }

    #[test]
    fn gc_dirty_worktree_is_rescue() {
        let mut facts = safe_decision_facts();
        facts.clean = CheckState::Fail;
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "rescue");
        assert!(reasons.iter().any(|reason| reason == "dirty_worktree"));
    }

    #[test]
    fn gc_squash_merged_pr_can_prove_integration() {
        let pr = GcPrAudit {
            status: "merged".to_string(),
            number: Some(42),
            url: Some("https://example.test/pr/42".to_string()),
            base_ref_name: Some("main".to_string()),
            head_ref_oid: Some("def".to_string()),
            covers_head: Some(true),
        };
        let criterion = integrated_criterion(Some(false), PrKind::Merged, &pr);
        assert_eq!(criterion.result, CheckState::Pass);
        assert!(criterion.evidence.contains("squash-safe"));

        let mut facts = safe_decision_facts();
        facts.pr = PrKind::Merged;
        let (decision, _) = decide_gc(facts);
        assert_eq!(decision, "candidate");
    }

    #[test]
    fn gc_merged_pr_for_another_base_cannot_prove_integration() {
        let (_tmp, _root, wt) = fixture();
        let head = git(&wt, &["rev-parse", "HEAD"]).unwrap();
        let rows = vec![GhPrRow {
            number: 42,
            state: "MERGED".to_string(),
            head_ref_name: "lane/a".to_string(),
            base_ref_name: "develop".to_string(),
            head_ref_oid: Some(head.clone()),
            url: "https://example.test/pr/42".to_string(),
            merged_at: Some("2026-08-16T00:00:00Z".to_string()),
        }];

        let (kind, audit, post_merge_local) =
            pr_for_worktree(&wt, Some("lane/a"), &head, Some("main"), Some(&rows));
        assert_eq!(kind, PrKind::Unknown);
        assert_eq!(audit.status, "other-base");
        assert_eq!(audit.base_ref_name.as_deref(), Some("develop"));
        assert_eq!(audit.covers_head, None);
        assert!(!post_merge_local);
    }

    #[test]
    fn gc_post_merge_local_commit_is_rescue() {
        let mut facts = safe_decision_facts();
        facts.pr = PrKind::Merged;
        facts.post_merge_local = true;
        facts.integrated = CheckState::Fail;
        facts.unique_commits = Some(1);
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "rescue");
        assert!(reasons
            .iter()
            .any(|reason| reason == "post_merge_local_commits"));
    }

    #[test]
    fn gc_no_pr_orphan_is_rescue() {
        let mut facts = safe_decision_facts();
        facts.pr = PrKind::None;
        facts.integrated = CheckState::Fail;
        facts.unique_commits = Some(3);
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "rescue");
        assert!(reasons
            .iter()
            .any(|reason| reason == "no_pr_unique_work_not_in_base"));
    }

    #[test]
    fn gc_closed_unmerged_pr_is_never_candidate() {
        let mut facts = safe_decision_facts();
        facts.pr = PrKind::Closed;
        let (decision, reasons) = decide_gc(facts);
        assert_eq!(decision, "review");
        assert_eq!(reasons, ["closed_without_merge"]);
    }
}
