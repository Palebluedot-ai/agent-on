//! agent-on CLI — replaces former Python scripts.

mod audit_lint;
mod guard;
mod intake_lint;
mod paths;
mod routing;
mod setup;
mod tag_release;

use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process;

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
    IntakeLint {
        files: Vec<PathBuf>,
    },
    /// Lint audit_event jsonl state machine
    #[command(name = "audit-lint")]
    AuditLint {
        file: PathBuf,
    },
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
    };
    process::exit(code);
}
