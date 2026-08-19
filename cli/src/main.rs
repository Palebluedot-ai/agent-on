//! agent-on CLI — replaces former Python scripts.

mod audit_lint;
mod guard;
mod intake_lint;
mod landing;
mod oncall;
mod paths;
mod routing;
mod setup;
mod tag_release;
mod worktree;
mod worktree_hooks;
mod worktree_schedule;

use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process;

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Parser)]
#[command(name = "agent-on", version, about = "agent-on toolkit (Rust)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print read_root / work_root registration report
    Doctor {
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// PreToolUse git guard (stdin JSON → exit 0 allow / 2 block)
    Guard,
    /// Lint intake Promotion Cards
    #[command(name = "intake-lint")]
    IntakeLint { files: Vec<PathBuf> },
    /// Lint audit_event jsonl state machine
    #[command(name = "audit-lint")]
    AuditLint { file: PathBuf },
    /// Open-box skill routing / demotion checks
    Check {
        #[command(subcommand)]
        what: CheckCmd,
    },
    /// Create annotated release tag (semver)
    #[command(name = "tag-release")]
    TagRelease {
        #[arg(long)]
        level: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        push: bool,
        #[arg(long)]
        allow_dirty: bool,
        /// Repo root (default: cwd)
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// One-shot clone/config/plugin setup
    Setup {
        #[arg(long)]
        work_root: Option<PathBuf>,
        #[arg(long, default_value = setup::default_pin())]
        pin: String,
        #[arg(long, default_value = setup::default_remote())]
        remote: String,
        #[arg(long)]
        with_plugins: bool,
        #[arg(long)]
        with_symlinks: bool,
        #[arg(long)]
        config_only: bool,
    },
    /// Register and audit parallel worktree lanes
    Worktree {
        #[command(subcommand)]
        action: WorktreeCmd,
    },
    /// Landing coordinator + lifecycle manager (read-only control plane)
    Landing {
        #[command(subcommand)]
        action: LandingCmd,
    },
    /// Single on-call registry: who holds merge / outbound / cross-window rights
    Oncall {
        #[command(subcommand)]
        action: OncallCmd,
    },
}

#[derive(Subcommand)]
enum OncallCmd {
    /// Go on call from this worktree (at most one on-call window at a time)
    Claim {
        /// SendMessage address of this window (session name or a stable prefix)
        #[arg(long)]
        session: String,
        /// Lane id; defaults to the lane registered for this worktree
        #[arg(long)]
        lane: Option<String>,
        #[arg(long, default_value = "")]
        note: String,
        /// Take over from the window currently on call (handover, leaves a trace)
        #[arg(long)]
        force: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Who is on call, since when, and at which address (any window may read)
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Is *this* window the on-call one
    Whoami {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Which lane owns a path — the on-call window's "reroute to whom" lookup
    Route {
        /// Repo-relative or absolute path
        #[arg(long)]
        path: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Go off call; the routing gate fails open again
    Release {
        /// Release someone else's registration (closed window / handover)
        #[arg(long)]
        force: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum LandingCmd {
    /// Batched evidence refresh bound to (PR head SHA, base SHA); the only
    /// network command, writes the local snapshot cache
    Refresh {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long)]
        base: Option<String>,
        /// Hours of inactivity required before a merged worktree is REAPABLE
        #[arg(long, default_value_t = 24)]
        quiet_hours: u64,
    },
    /// Homepage summary + six-category merge table from the cached snapshot
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long, default_value_t = 24)]
        quiet_hours: u64,
    },
    /// Serial merge waves + parallel prep from the cached snapshot
    Plan {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long, default_value_t = 24)]
        quiet_hours: u64,
    },
}

#[derive(Subcommand)]
enum WorktreeCmd {
    /// Claim the current worktree with an explicit goal and non-overlapping file boundary
    Claim {
        #[arg(long)]
        id: String,
        #[arg(long)]
        goal: String,
        #[arg(long)]
        base: Option<String>,
        /// Owned path prefix; repeat the flag or pass a comma-separated list.
        /// A path containing a literal comma needs git quoted form, e.g. --owns '"a\054b.md"'
        #[arg(long = "owns", required = true, value_delimiter = ',')]
        owns: Vec<String>,
        /// Lane id this lane waits on; repeat the flag or pass a comma-separated list
        #[arg(long = "depends-on", value_delimiter = ',')]
        depends_on: Vec<String>,
        /// Queue the lane as parked (does not count toward the active-lane cap)
        #[arg(long)]
        parked: bool,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Change a lane lifecycle state: active, blocked, ready, landed, or parked
    #[command(name = "set-status")]
    SetStatus {
        status: String,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Redivide an existing lane in place: goal, owns, branch, base, or status
    Edit {
        /// Lane id; defaults to the lane registered for the current worktree
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        goal: Option<String>,
        /// Replacement boundary set; repeat the flag or pass a comma-separated list.
        /// A path containing a literal comma needs git quoted form, e.g. --owns '"a\054b.md"'
        #[arg(long = "owns", value_delimiter = ',')]
        owns: Vec<String>,
        /// New branch name; must resolve to an existing ref
        #[arg(long)]
        branch: Option<String>,
        /// New base ref; re-pins the recorded base sha
        #[arg(long)]
        base: Option<String>,
        /// Re-register the lifecycle state, bypassing the transition graph:
        /// active, blocked, ready, landed, or parked. Repair door for a stale
        /// registration, e.g. a reused worktree still booked as landed while a
        /// session writes in it. All other status guards still apply.
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        cwd: Option<PathBuf>,
    },
    /// Show all worktrees, boundaries, drift, dependencies, and reclaim class
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Exit non-zero on unregistered worktrees, boundary violations, or lane overlap
    Check {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Install, inspect, or remove shared Git commit/push guards
    Hooks {
        #[command(subcommand)]
        action: WorktreeHooksCmd,
    },
    /// Read-only, fail-closed reclaim audit; never removes worktrees or branches
    Gc {
        /// Required safety acknowledgement; this command has no apply mode
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long, default_value_t = 24)]
        quiet_hours: u64,
    },
    /// Remove a landed/parked lane record after its worktree is already gone
    Forget {
        #[arg(long)]
        id: String,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum WorktreeHooksCmd {
    /// Install repository-local shared pre-commit and pre-push guards
    Install {
        #[arg(long)]
        repo: Option<PathBuf>,
        /// Also install the optional daily report-only GC audit
        #[arg(long)]
        daily_gc: bool,
    },
    /// Show whether both shared guards are installed and healthy
    Status {
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Remove only the guards installed by Agent-On
    Uninstall {
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Internal entrypoint invoked by the managed Git hook scripts
    #[command(hide = true)]
    Run {
        #[arg(long)]
        hook: String,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CheckCmd {
    /// Skill routing + demotion protocol on agent-on tree
    Routing {
        #[arg(long)]
        with_agent_memory: bool,
        #[arg(long)]
        home: Option<PathBuf>,
        /// agent-on repo root (default: walk up from cwd or CARGO_MANIFEST_DIR parent)
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn find_repo_root() -> PathBuf {
    if let Ok(m) = env::var("CARGO_MANIFEST_DIR") {
        // cli/ -> repo
        let p = PathBuf::from(m);
        if let Some(parent) = p.parent() {
            if paths::looks_like_agent_on(parent) {
                return parent.to_path_buf();
            }
        }
    }
    let mut cur = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..12 {
        if paths::looks_like_agent_on(&cur) {
            return cur;
        }
        if !cur.pop() {
            break;
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Commands::Doctor { cwd } => {
            print!("{}", paths::doctor_report(cwd.as_deref()));
            0
        }
        Commands::Guard => guard::run_from_stdin(),
        Commands::IntakeLint { files } => {
            let paths_v = if files.is_empty() {
                intake_lint::default_intake_paths(&find_repo_root())
            } else {
                files
            };
            let (c, out) = intake_lint::lint_paths(&paths_v);
            print!("{out}");
            c
        }
        Commands::AuditLint { file } => {
            let (c, out) = audit_lint::lint_file(&file);
            print!("{out}");
            c
        }
        Commands::Check { what } => match what {
            CheckCmd::Routing {
                with_agent_memory,
                home,
                root,
            } => {
                let root = root.unwrap_or_else(find_repo_root);
                let (c, out) = routing::run_check(&root, with_agent_memory, home);
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
        },
        Commands::TagRelease {
            level,
            title,
            push,
            allow_dirty,
            repo,
        } => {
            let repo = repo.unwrap_or_else(find_repo_root);
            let (c, out) = tag_release::run_tag_release(
                &repo,
                &tag_release::TagOpts {
                    level,
                    title,
                    push,
                    allow_dirty,
                },
            );
            if c == 0 {
                print!("{out}");
            } else {
                eprint!("{out}");
            }
            c
        }
        Commands::Setup {
            work_root,
            pin,
            remote,
            with_plugins,
            with_symlinks,
            config_only,
        } => setup::run_setup(&setup::SetupOpts {
            work_root,
            pin,
            remote,
            with_plugins,
            with_symlinks,
            config_only,
            config_path_override: None,
        }),
        Commands::Worktree { action } => match action {
            WorktreeCmd::Claim {
                id,
                goal,
                base,
                owns,
                depends_on,
                parked,
                cwd,
            } => {
                let cwd = cwd
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::claim_lane(
                    &cwd,
                    &worktree::ClaimOpts {
                        id,
                        goal,
                        base,
                        owns,
                        depends_on,
                        parked,
                    },
                );
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::SetStatus { status, id, cwd } => {
                let cwd = cwd
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::set_lane_status(&cwd, id.as_deref(), &status);
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::Edit {
                id,
                goal,
                owns,
                branch,
                base,
                status,
                cwd,
            } => {
                let cwd = cwd
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::edit_lane(
                    &cwd,
                    &worktree::EditOpts {
                        id,
                        goal,
                        owns,
                        branch,
                        base,
                        status,
                    },
                );
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::Status { json, repo } => {
                let repo = repo
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::run_audit(&repo, json, false);
                print!("{out}");
                c
            }
            WorktreeCmd::Check { json, repo } => {
                let repo = repo
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::run_audit(&repo, json, true);
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::Hooks { action } => {
                let default_repo = || env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let (c, out) = match action {
                    WorktreeHooksCmd::Install { repo, daily_gc } => {
                        worktree_hooks::install_with_options(
                            &repo.unwrap_or_else(default_repo),
                            daily_gc,
                        )
                    }
                    WorktreeHooksCmd::Status { repo } => {
                        worktree_hooks::status_with_schedule(&repo.unwrap_or_else(default_repo))
                    }
                    WorktreeHooksCmd::Uninstall { repo } => {
                        worktree_hooks::uninstall_with_schedule(&repo.unwrap_or_else(default_repo))
                    }
                    WorktreeHooksCmd::Run { hook, repo } => {
                        worktree_hooks::run_hook(&repo.unwrap_or_else(default_repo), &hook)
                    }
                };
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::Gc {
                dry_run,
                json,
                repo,
                base,
                quiet_hours,
            } => {
                let repo = repo
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::run_gc(
                    &repo,
                    &worktree::GcOpts {
                        dry_run,
                        json,
                        base,
                        quiet_hours,
                    },
                );
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
            WorktreeCmd::Forget { id, repo } => {
                let repo = repo
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let (c, out) = worktree::forget_lane(&repo, &id);
                if c == 0 {
                    print!("{out}");
                } else {
                    eprint!("{out}");
                }
                c
            }
        },
        Commands::Landing { action } => {
            let default_repo = || env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (c, out) = match action {
                LandingCmd::Refresh {
                    json,
                    repo,
                    base,
                    quiet_hours,
                } => landing::run_refresh(
                    &repo.unwrap_or_else(default_repo),
                    &landing::RealGh,
                    &landing::LandingOpts {
                        json,
                        base,
                        quiet_hours,
                    },
                ),
                LandingCmd::Status {
                    json,
                    repo,
                    quiet_hours,
                } => landing::run_status(
                    &repo.unwrap_or_else(default_repo),
                    &landing::LandingOpts {
                        json,
                        base: None,
                        quiet_hours,
                    },
                ),
                LandingCmd::Plan {
                    json,
                    repo,
                    quiet_hours,
                } => landing::run_plan(
                    &repo.unwrap_or_else(default_repo),
                    &landing::LandingOpts {
                        json,
                        base: None,
                        quiet_hours,
                    },
                ),
            };
            if c == 0 {
                print!("{out}");
            } else {
                eprint!("{out}");
            }
            c
        }
        Commands::Oncall { action } => {
            let default_cwd = || env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (c, out) = match action {
                OncallCmd::Claim {
                    session,
                    lane,
                    note,
                    force,
                    cwd,
                } => oncall::claim(
                    &cwd.unwrap_or_else(default_cwd),
                    &session,
                    lane.as_deref(),
                    &note,
                    force,
                ),
                OncallCmd::Status { json, cwd } => {
                    oncall::status(&cwd.unwrap_or_else(default_cwd), json)
                }
                OncallCmd::Whoami { json, cwd } => {
                    oncall::whoami(&cwd.unwrap_or_else(default_cwd), json)
                }
                OncallCmd::Route { path, json, cwd } => {
                    oncall::route(&cwd.unwrap_or_else(default_cwd), &path, json)
                }
                OncallCmd::Release { force, cwd } => {
                    oncall::release(&cwd.unwrap_or_else(default_cwd), force)
                }
            };
            if c == 0 {
                print!("{out}");
            } else {
                eprint!("{out}");
            }
            c
        }
    };
    process::exit(code);
}
