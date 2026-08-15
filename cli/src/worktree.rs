use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
            "- {} [{}] {} | base {} behind {} | changed {} | reclaim {}\n  goal: {}\n  owns: {}\n",
            lane.id,
            lane.status,
            lane.branch,
            lane.base,
            lane.base_ahead
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string()),
            lane.changed_files.len(),
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
}
