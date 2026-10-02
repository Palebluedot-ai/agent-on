//! Opt-in reclaim policy for control-plane-owned checkouts only.
//! Existing `worktree gc` remains report-only. Branches and recovery refs survive removal.

use crate::coordination::{git, now, read_json, session_active, write_json, Config, Store, Task};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

type Result<T> = std::result::Result<T, String>;

#[derive(Args)]
pub(crate) struct JanitorArgs {
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
    #[command(subcommand)]
    action: JanitorCmd,
}

#[derive(Subcommand)]
enum JanitorCmd {
    /// Authorize bounded reclaim for this project's managed checkouts; no immediate deletion
    Enable {
        #[arg(long, default_value_t = 24)]
        quiet_hours: u64,
        /// Integration target in the PRIMARY worktree, never a worker's own HEAD
        #[arg(long, default_value = "HEAD")]
        base: String,
        /// Explicit regenerable cache directories; all other ignored files protect a checkout
        #[arg(long = "regenerable-ignored")]
        regenerable: Vec<String>,
        /// Policy only; use a native host heartbeat or the running patrol instead of an OS timer
        #[arg(long)]
        manual: bool,
    },
    Disable,
    /// Report by default. Apply requires the project's previously enabled policy
    Run {
        #[arg(long)]
        apply: bool,
        #[arg(long, hide = true)]
        scheduled: bool,
    },
    /// Restore a removed checkout from its retained recovery ref; never overwrites a path
    Restore {
        id: String,
    },
}

pub(crate) fn run(args: JanitorArgs) -> Result<Value> {
    let store = Store::open(
        &args
            .repo
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    )?;
    match args.action {
        JanitorCmd::Enable {
            quiet_hours,
            base,
            regenerable,
            manual,
        } => {
            crate::coordination::paths_valid(&regenerable)?;
            if regenerable.iter().any(|p| {
                p.contains(['*', '?'])
                    || !["node_modules", "target", ".next", "dist", "build", ".cache"]
                        .iter()
                        .any(|s| p.split('/').next_back() == Some(s))
            }) {
                return Err("regenerable ignores must name explicit cache directories (node_modules, target, .next, dist, build, .cache); no globs".into());
            }
            git(
                &store.repo,
                &["rev-parse", "--verify", &format!("{base}^{{commit}}")],
            )?;
            let schedule = if manual {
                None
            } else {
                let (code, output) = crate::worktree_schedule::run_install_janitor(&store.repo);
                if code != 0 {
                    return Err(format!("daily janitor not enabled: {output}; use --manual for a native heartbeat policy"));
                }
                Some(output)
            };
            let _lock = store.lock()?;
            let mut config = store.config()?;
            config.janitor_enabled = true;
            config.quiet_hours = quiet_hours;
            config.janitor_base = base;
            config.regenerable_ignored = regenerable;
            store.save_config(&config)?;
            Ok(
                json!({"policy":config,"scope":"only dispatch-created managed checkouts with explicit task release","schedule":schedule,"manual":manual}),
            )
        }
        JanitorCmd::Disable => {
            let _lock = store.lock()?;
            let mut config = store.config()?;
            config.janitor_enabled = false;
            store.save_config(&config)?;
            drop(_lock);
            let (code, output) = crate::worktree_schedule::run_uninstall_janitor(&store.repo);
            if code != 0 {
                return Err(format!(
                    "policy disabled, but scheduler retained for manual inspection: {output}"
                ));
            }
            Ok(json!({"janitor_enabled":false,"schedule":output}))
        }
        JanitorCmd::Run { apply, scheduled } => {
            if scheduled && !store.config()?.janitor_enabled {
                return Ok(json!({"janitor_enabled":false,"rows":[]}));
            }
            sweep(&store, apply)
        }
        JanitorCmd::Restore { id } => restore(&store, &id),
    }
}

fn descendant(root: &Path, scope: &Path) -> bool {
    match (fs::canonicalize(root), fs::canonicalize(scope)) {
        (Ok(root), Ok(scope)) => root != scope && root.starts_with(scope),
        _ => false,
    }
}
fn protected_ignored(path: &str, config: &Config) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    // Even inside an allowed cache, env/credentials/databases are protected.
    if name.starts_with(".env")
        || name == ".git"
        || name.ends_with(".db")
        || name.ends_with(".sqlite")
        || name.ends_with(".sqlite3")
        || name.ends_with(".pem")
        || name.ends_with(".key")
    {
        return true;
    }
    !config
        .regenerable_ignored
        .iter()
        .any(|cache| path.starts_with(&format!("{cache}/")))
}
fn latest_activity(task: &Task, root: &Path) -> Result<DateTime<Utc>> {
    let released = task
        .released_at
        .as_deref()
        .ok_or("release timestamp absent")?;
    let mut latest = DateTime::parse_from_rfc3339(released)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0_u64;
    // Detect ignored embedded repositories, local resources, symlinks, and recent activity
    // as well. A bounded unknown inventory is protected, never inferred clean.
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!("symlink requires review: {}", path.display()));
        }
        latest = latest.max(DateTime::<Utc>::from(
            metadata.modified().map_err(|e| e.to_string())?,
        ));
        if metadata.is_dir() {
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                count += 1;
                if count > 100_000 {
                    return Err("inventory exceeds 100000 entries; review required".into());
                }
                if entry.file_name() == ".git" {
                    if entry.path() != root.join(".git") {
                        return Err("embedded git repository".into());
                    }
                } else {
                    pending.push(entry.path());
                }
            }
        }
    }
    let admin = PathBuf::from(git(root, &["rev-parse", "--absolute-git-dir"])?);
    for name in ["HEAD", "index", "logs/HEAD"] {
        let path = admin.join(name);
        if path.exists() {
            latest = latest.max(DateTime::<Utc>::from(
                fs::metadata(path)
                    .map_err(|e| e.to_string())?
                    .modified()
                    .map_err(|e| e.to_string())?,
            ));
        }
    }
    Ok(latest)
}

fn merged_pr_covers(store: &Store, task: &Task, base: &str) -> Result<bool> {
    // Reuse the existing report's tested PR-head coverage predicate. A MERGED label alone is
    // insufficient: commits added after the PR head must remain protected.
    let (code, report) = crate::worktree::run_gc(
        &store.repo,
        &crate::worktree::GcOpts {
            dry_run: true,
            json: true,
            base: Some(base.into()),
            quiet_hours: 0,
        },
    );
    if code != 0 {
        return Ok(false);
    }
    let report: Value = serde_json::from_str(&report).map_err(|e| e.to_string())?;
    Ok(report["worktrees"].as_array().is_some_and(|rows| {
        rows.iter().any(|row| {
            row["path"].as_str() == task.worktree.to_str()
                && row["pr"]["status"] == "merged"
                && row["pr"]["covers_head"] == true
                && row["criteria"]["integrated"]["result"] == "pass"
        })
    }))
}
pub(crate) fn integrated(store: &Store, task: &Task, base_sha: &str, base: &str) -> Result<bool> {
    let head = if task.worktree.exists() {
        git(&task.worktree, &["rev-parse", "HEAD"])?
    } else {
        git(
            &store.repo,
            &["rev-parse", &format!("refs/agent-on/recovery/{}", task.id)],
        )?
    };
    if task.released_head.as_deref() != Some(head.as_str()) {
        return Ok(false);
    }
    Ok(git(
        &store.repo,
        &["merge-base", "--is-ancestor", &head, base_sha],
    )
    .is_ok()
        || merged_pr_covers(store, task, base)?)
}
fn no_live_cwd(root: &Path) -> Result<()> {
    let output = Command::new("lsof")
        .args(["+D"])
        .arg(root)
        .args(["-F", "p"])
        .output()
        .map_err(|e| format!("process/CWD inventory unavailable; checkout protected: {e}"))?;
    if !output.stderr.is_empty() || !matches!(output.status.code(), Some(0 | 1)) {
        return Err("process/CWD inventory incomplete; checkout protected".into());
    }
    if !output.stdout.is_empty() {
        return Err(
            "process still uses this checkout (worker/preview/other); close it first".into(),
        );
    }
    Ok(())
}
fn eligible(store: &Store, task: &Task, config: &Config, base_sha: &str) -> Result<Value> {
    let root = &task.worktree;
    if !task.managed || !descendant(root, &store.dir.join("checkouts")) {
        return Err("not an owned managed checkout".into());
    }
    if task.state != "released" || task.evidence.as_deref().is_none_or(|s| s.trim().is_empty()) {
        return Err("task has not been explicitly released with evidence".into());
    }
    if store
        .sessions()?
        .iter()
        .any(|s| s.cwd == *root && session_active(s))
    {
        return Err("live or unconfirmed session".into());
    }
    if store.tasks()?.iter().any(|t| {
        t.id != task.id
            && !matches!(t.state.as_str(), "released" | "reclaimed")
            && (t.worktree == *root || t.dependencies.contains(&task.id))
    }) {
        return Err("shared checkout or live dependent task".into());
    }
    let listing = git(&store.repo, &["worktree", "list", "--porcelain", "-z"])?;
    let block = listing
        .split("\0\0")
        .find(|block| {
            block.split('\0').any(|line| {
                line.strip_prefix("worktree ")
                    .is_some_and(|p| Path::new(p) == root)
            })
        })
        .ok_or("checkout no longer registered")?;
    if block
        .split('\0')
        .any(|line| line == "locked" || line.starts_with("locked ") || line.starts_with("prunable"))
    {
        return Err("locked or prunable checkout".into());
    }
    let admin = PathBuf::from(git(root, &["rev-parse", "--absolute-git-dir"])?);
    for name in [
        "MERGE_HEAD",
        "rebase-merge",
        "rebase-apply",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "BISECT_LOG",
    ] {
        if admin.join(name).exists() {
            return Err(format!("git operation in progress: {name}"));
        }
    }
    if !git(
        root,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?
    .is_empty()
    {
        return Err("staged, unstaged, or untracked work must be rescued".into());
    }
    if git(root, &["ls-files", "--stage"])?
        .lines()
        .any(|line| line.starts_with("160000 "))
    {
        return Err("initialized/declared submodule requires review".into());
    }
    let ignored = git(
        root,
        &[
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "-z",
        ],
    )?;
    let protected: Vec<_> = ignored
        .split('\0')
        .filter(|p| !p.is_empty() && protected_ignored(p, config))
        .collect();
    if !protected.is_empty() {
        return Err(format!(
            "ignored local resources require rescue: {protected:?}"
        ));
    }
    let head = git(root, &["rev-parse", "HEAD"])?;
    no_live_cwd(root)?;
    if task.released_head.as_deref() != Some(head.as_str()) {
        return Err("HEAD changed after release; new work requires a new explicit release".into());
    }
    if git(root, &["merge-base", "--is-ancestor", &head, base_sha]).is_err()
        && !merged_pr_covers(store, task, &config.janitor_base)?
    {
        return Err("HEAD not integrated; merged PR head coverage absent or unknown".into());
    }
    let activity = latest_activity(task, root)?;
    if (Utc::now() - activity).num_seconds()
        < config.quiet_hours.saturating_mul(3600).min(i64::MAX as u64) as i64
    {
        return Err("cooldown not complete".into());
    }
    Ok(
        json!({"id":task.id,"worktree":root,"head":head,"base_sha":base_sha,"last_activity":activity.to_rfc3339(),"decision":"reclaimable"}),
    )
}

pub(crate) fn sweep(store: &Store, apply: bool) -> Result<Value> {
    let _lock = store.lock()?;
    let config = store.config()?;
    if apply && !config.janitor_enabled {
        return Err("janitor is disabled; enable the bounded project policy first".into());
    }
    let base_sha = git(
        &store.repo,
        &[
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", config.janitor_base),
        ],
    )?;
    let mut rows = Vec::new();
    for task in store
        .tasks()?
        .into_iter()
        .filter(|t| t.managed && t.state != "reclaimed")
    {
        let evidence = match eligible(store, &task, &config, &base_sha) {
            Ok(value) => value,
            Err(reason) => {
                rows.push(json!({"id":task.id,"worktree":task.worktree,"decision":"protected","reason":reason}));
                continue;
            }
        };
        if !apply {
            rows.push(evidence);
            continue;
        }
        let recovery_ref = format!("refs/agent-on/recovery/{}", task.id);
        let head = evidence["head"].as_str().ok_or("head missing")?;
        git(&store.repo, &["update-ref", &recovery_ref, head])?;
        let receipt = json!({"id":task.id,"worktree":task.worktree,"head":head,"recovery_ref":recovery_ref,"at":now(),"state":"prepared","evidence":evidence});
        let receipt_path = store.dir.join("reclaim").join(format!("{}.json", task.id));
        write_json(&receipt_path, &receipt)?;
        // Re-read the task, local checkout, live session receipts, HEAD AND current base just
        // before removal. Git itself rejects concurrent dirty changes; no force is ever used.
        let current_base = git(&store.repo, &["rev-parse", &config.janitor_base])?;
        let check = eligible(store, &store.task(&task.id)?, &config, &current_base)?;
        if current_base != base_sha || check["head"] != head {
            return Err(
                "reclaim evidence changed; preserved receipt retained, removal aborted".into(),
            );
        }
        match git(&store.repo, &["worktree", "remove", task.worktree.to_str().ok_or("non-UTF8 worktree")?]) {
            Ok(_) => {
                let mut removed = task.clone();
                removed.state = "reclaimed".into();
                store.save_task(&removed)?;
                let mut receipt = receipt;
                receipt["state"] = "removed".into();
                write_json(&receipt_path, &receipt)?;
                rows.push(json!({"id":task.id,"decision":"reclaimed","recovery_ref":recovery_ref,"branch":"retained","receipt":receipt_path}));
            }
            Err(error) => rows.push(json!({"id":task.id,"decision":"protected","reason":error,"recovery_ref":recovery_ref})),
        }
    }
    let report =
        json!({"mode":if apply { "apply" } else { "report" },"base_sha":base_sha,"rows":rows});
    if apply {
        write_json(&store.dir.join("janitor-report.json"), &report)?;
    }
    Ok(report)
}

pub(crate) fn scheduled(store: &Store) -> Result<()> {
    if store.config()?.janitor_enabled {
        sweep(store, true)?;
    }
    Ok(())
}
fn restore(store: &Store, id: &str) -> Result<Value> {
    crate::coordination::validate_id(id)?;
    let _lock = store.lock()?;
    let receipt: Value = read_json(&store.dir.join("reclaim").join(format!("{id}.json")))?;
    let mut task = store.task(id)?;
    if receipt["state"] != "removed" || task.state != "reclaimed" {
        return Err("receipt is not a completed removal".into());
    }
    if task.worktree.exists() {
        return Err("restore path already exists; nothing overwritten".into());
    }
    let reference = receipt["recovery_ref"]
        .as_str()
        .ok_or("recovery ref absent")?;
    if reference != format!("refs/agent-on/recovery/{id}") {
        return Err("unexpected recovery ref".into());
    }
    git(
        &store.repo,
        &[
            "worktree",
            "add",
            "--detach",
            task.worktree.to_str().ok_or("non-UTF8 checkout")?,
            reference,
        ],
    )?;
    task.state = "restored".into();
    task.released_at = None;
    store.save_task(&task)?;
    Ok(
        json!({"restored":task.worktree,"head":git(&task.worktree, &["rev-parse", "HEAD"])?,"recovery_ref":reference,"release_required":true}),
    )
}
