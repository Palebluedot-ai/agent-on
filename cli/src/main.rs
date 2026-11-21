//! Agent-On v2 CLI — zero-intrusion cross-client worktree radar, merge-tree rehearsal & skill installer.

mod build_info;
mod doctor;
mod git_util;
mod install;
mod merge_check;
mod source_identity;
mod wt;

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "agent-on",
    version,
    about = "Zero-intrusion cross-client AI toolkit (wt-doctor, merge-check, install, doctor)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan all Git worktrees across Claude/Codex/Antigravity for collisions, base lag, and safe pruning
    #[command(visible_alias = "wt-doctor")]
    Wt(wt::WtArgs),
    /// Rehearse merging parallel branches or open PRs in memory via git merge-tree
    MergeCheck(merge_check::MergeCheckArgs),
    /// Symlink atomic skills and optional core-guardrails into Claude Code, Codex, and Antigravity
    Install(install::InstallArgs),
    /// Inspect CLI build identity, host skill mounts, and any v0.x legacy residues
    Doctor(doctor::DoctorArgs),
    /// Print compiled source fingerprint, commit, and verified release tag
    BuildInfo {
        #[arg(long)]
        json: bool,
    },
    /// Legacy v0.x PreToolUse hook compatibility no-op (never blocks)
    #[command(hide = true)]
    Guard,
    /// Legacy v0.x SessionStart hook compatibility no-op
    #[command(hide = true)]
    Capture,
    /// Legacy v0.x `agent-on worktree ...` compatibility shim so uncleaned git hooks/timers never fail
    #[command(hide = true)]
    Worktree {
        #[command(subcommand)]
        action: LegacyWorktreeCmd,
    },
}

#[derive(Subcommand)]
enum LegacyWorktreeCmd {
    Hooks {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    Gc {
        #[arg(long)]
        repo: Option<PathBuf>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        _ignored: Vec<String>,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        Commands::Wt(args) => wt::run(&args),
        Commands::MergeCheck(args) => merge_check::run(&args),
        Commands::Install(args) => install::run(&args),
        Commands::Doctor(args) => doctor::run(&args),
        Commands::BuildInfo { json } => {
            print!("{}", build_info::report(json));
            0
        }
        Commands::Guard => {
            // Drain stdin briefly so callers expecting EOF do not get EPIPE
            let mut sink = Vec::new();
            let _ = std::io::stdin().take(65_536).read_to_end(&mut sink);
            0
        }
        Commands::Capture => 0,
        Commands::Worktree { action } => match action {
            LegacyWorktreeCmd::Hooks { rest } => {
                if rest.iter().any(|a| a == "run") {
                    eprintln!(
                        "[agent-on v2] 旧版受管 Git hook 已退役（本次直接放行）；运行 `agent-on install --clean-legacy` 可清除残留配置。"
                    );
                }
                0
            }
            LegacyWorktreeCmd::Gc {
                repo,
                base,
                json,
                _ignored: _,
            } => wt::run(&wt::WtArgs {
                repo,
                base,
                with_prs: false,
                prune: false,
                json,
            }),
            LegacyWorktreeCmd::Other(sub) => {
                let name = sub.first().map(String::as_str).unwrap_or("worktree");
                eprintln!(
                    "[agent-on v2] 旧版 `worktree {name}` 台账命令已由无状态 `agent-on wt` 取代。"
                );
                0
            }
        },
    };
    process::exit(code);
}
