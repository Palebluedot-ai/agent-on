//! Read-only environment, host mount, and legacy residue diagnostics (`agent-on doctor`).

use crate::build_info;
use crate::git_util::{canonical_or_raw, git_stdout, list_worktrees, resolve_repo};
use crate::install::{
    default_home, is_legacy_agent_on_hook_command, resolve_source_root, RULES_START, SKILLS,
};
use crate::source_identity;
use clap::Args;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Target repository to check for v0.x legacy residues (defaults to current directory)
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Override user home directory
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Override Agent-On source repository path
    #[arg(long)]
    pub source: Option<PathBuf>,
    /// Output machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostStatus {
    pub host: String,
    pub linked_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub rules_active: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyResidue {
    pub scope: String,
    pub path: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub version: &'static str,
    pub commit: &'static str,
    pub release_tag: Option<&'static str>,
    pub source_fingerprint: &'static str,
    pub source_root: Option<String>,
    pub binary_matches_source: Option<bool>,
    pub hosts: Vec<HostStatus>,
    pub legacy_residues: Vec<LegacyResidue>,
}

fn check_symlink_matches(link: &Path, expected: Option<&Path>) -> bool {
    let Ok(meta) = fs::symlink_metadata(link) else {
        return false;
    };
    if !meta.file_type().is_symlink() || !link.exists() {
        return false;
    }
    match expected {
        Some(exp) => canonical_or_raw(link) == canonical_or_raw(exp),
        None => true,
    }
}

fn file_has_rules_block(path: &Path, guardrails_src: Option<&Path>) -> bool {
    if !path.exists() {
        return false;
    }
    if let Some(src) = guardrails_src {
        if canonical_or_raw(path) == canonical_or_raw(src) {
            return true;
        }
    }
    fs::read_to_string(path)
        .map(|s| s.contains(RULES_START))
        .unwrap_or(false)
}

pub fn collect(args: &DoctorArgs) -> Result<DoctorReport, String> {
    let home = args.home.clone().unwrap_or_else(default_home);
    let source = resolve_source_root(args.source.as_deref(), &home)?;
    let info = build_info::current();
    let guardrails_src = source.as_ref().map(|s| s.join("rules/core-guardrails.md"));

    let binary_matches_source = source
        .as_ref()
        .and_then(|s| source_identity::fingerprint(&s.join("cli")).ok())
        .map(|fp| fp == info.source_fingerprint);

    // 1. Claude Code
    let mut claude_linked = Vec::new();
    let mut claude_missing = Vec::new();
    for skill in SKILLS {
        let link = home.join(".claude/skills").join(skill);
        let exp = source.as_ref().map(|s| s.join("skills").join(skill));
        if check_symlink_matches(&link, exp.as_deref()) {
            claude_linked.push((*skill).to_string());
        } else {
            claude_missing.push((*skill).to_string());
        }
    }
    let claude_rules =
        file_has_rules_block(&home.join(".claude/CLAUDE.md"), guardrails_src.as_deref());
    let claude_status = HostStatus {
        host: "claude-code".to_string(),
        detail: format!(
            "Skills {}/{} 已挂载 · CLAUDE.md 铁律段落：{}",
            claude_linked.len(),
            SKILLS.len(),
            if claude_rules {
                "已启用"
            } else {
                "未挂载（可选 --with-rules）"
            }
        ),
        linked_skills: claude_linked,
        missing_skills: claude_missing,
        rules_active: claude_rules,
    };

    // 2. Codex (.agents/skills)
    let mut codex_linked = Vec::new();
    let mut codex_missing = Vec::new();
    for skill in SKILLS {
        let link = home.join(".agents/skills").join(skill);
        let exp = source.as_ref().map(|s| s.join("skills").join(skill));
        if check_symlink_matches(&link, exp.as_deref()) {
            codex_linked.push((*skill).to_string());
        } else {
            codex_missing.push((*skill).to_string());
        }
    }
    let codex_rules =
        file_has_rules_block(&home.join(".codex/AGENTS.md"), guardrails_src.as_deref());
    let codex_status = HostStatus {
        host: "codex".to_string(),
        detail: format!(
            "Skills {}/{} 已挂载 · ~/.codex/AGENTS.md 铁律段落：{}",
            codex_linked.len(),
            SKILLS.len(),
            if codex_rules {
                "已启用"
            } else {
                "未挂载（可选 --with-rules）"
            }
        ),
        linked_skills: codex_linked,
        missing_skills: codex_missing,
        rules_active: codex_rules,
    };

    // 3. Antigravity (.gemini/config/plugins/agent-on)
    let agy_plugin = home.join(".gemini/config/plugins/agent-on");
    let agy_ok = check_symlink_matches(&agy_plugin, source.as_deref())
        || (agy_plugin.join("plugin.json").is_file()
            && SKILLS
                .iter()
                .all(|s| agy_plugin.join("skills").join(s).join("SKILL.md").is_file()));
    let (agy_linked, agy_missing) = if agy_ok {
        (
            SKILLS.iter().map(|s| (*s).to_string()).collect(),
            Vec::new(),
        )
    } else {
        (
            Vec::new(),
            SKILLS.iter().map(|s| (*s).to_string()).collect(),
        )
    };
    let agy_rules = agy_ok && agy_plugin.join("rules/AGENTS.md").is_file();
    let agy_status = HostStatus {
        host: "antigravity".to_string(),
        detail: if agy_ok {
            format!(
                "插件已挂载（Skills 4/4 + rules/AGENTS.md 自动生效）：{}",
                agy_plugin.display()
            )
        } else {
            "未挂载（运行 `agent-on install` 软链至 ~/.gemini/config/plugins/agent-on）".to_string()
        },
        linked_skills: agy_linked,
        missing_skills: agy_missing,
        rules_active: agy_rules,
    };

    // 4. Legacy v0.x residues
    let mut residues = Vec::new();
    for old_skill in [
        home.join(".claude/skills/agent-on"),
        home.join(".agents/skills/agent-on"),
    ] {
        if fs::symlink_metadata(&old_skill).is_ok() {
            residues.push(LegacyResidue {
                scope: "host-skill".to_string(),
                path: old_skill.display().to_string(),
                remediation: "运行 `agent-on install --clean-legacy` 自动移除旧版单路由软链"
                    .to_string(),
            });
        }
    }

    for host_hook_cfg in [
        home.join(".claude/settings.json"),
        home.join(".codex/hooks.json"),
    ] {
        if let Ok(text) = fs::read_to_string(&host_hook_cfg) {
            if is_legacy_agent_on_hook_command(&text) {
                let uses_removed_wrapper = text.contains("kit/guard/agent-on-git-guard");
                let remediation = if uses_removed_wrapper {
                    "旧版包装脚本路径已归档（未清理前宿主 PreToolUse 调用会报 exit 127）；请运行 `agent-on install --clean-legacy` 自动清理该钩子条目".to_string()
                } else {
                    "运行 `agent-on install --clean-legacy` 自动移除旧版 PreToolUse / SessionStart 钩子条目".to_string()
                };
                residues.push(LegacyResidue {
                    scope: "host-pretool-hook".to_string(),
                    path: host_hook_cfg.display().to_string(),
                    remediation,
                });
            }
        }
    }

    let repo_dir = args.repo.clone().or_else(|| env::current_dir().ok());
    if let Some(dir) = repo_dir {
        if let Ok(ctx) = resolve_repo(&dir) {
            let mut has_hook_residue = git_stdout(
                &ctx.current_worktree,
                &["config", "--get-all", "core.hooksPath"],
            )
            .map(|s| s.lines().any(|l| l.contains("agent-on/hooks")))
            .unwrap_or(false);

            if !has_hook_residue
                && git_stdout(
                    &ctx.current_worktree,
                    &["config", "--bool", "--get", "extensions.worktreeConfig"],
                )
                .map(|v| v.trim() == "true")
                .unwrap_or(false)
            {
                if let Ok(wts) = list_worktrees(&ctx.current_worktree) {
                    has_hook_residue = wts.iter().filter(|w| !w.bare && !w.missing).any(|w| {
                        git_stdout(
                            &w.path,
                            &["config", "--worktree", "--get-all", "core.hooksPath"],
                        )
                        .map(|s| s.lines().any(|l| l.contains("agent-on/hooks")))
                        .unwrap_or(false)
                    });
                }
            }

            if has_hook_residue {
                residues.push(LegacyResidue {
                    scope: "repo-git-hooks".to_string(),
                    path: ctx
                        .common_git_dir
                        .join("agent-on/hooks")
                        .display()
                        .to_string(),
                    remediation: format!(
                        "运行 `agent-on install --clean-legacy --repo {}` 自动移除旧版受管 core.hooksPath（当前由 CLI 兼容桩静默放行）",
                        ctx.current_worktree.display()
                    ),
                });
            }
            let lock_file = ctx.current_worktree.join("agent-on.lock.md");
            if lock_file.is_file() {
                residues.push(LegacyResidue {
                    scope: "repo-lockfile".to_string(),
                    path: lock_file.display().to_string(),
                    remediation: "业务仓不再需要 `agent-on.lock.md` 与 `agent-on/` 目录，可按项目规则归档或删除（见 docs/migration-v2.0.md）".to_string(),
                });
            }
        }
    }

    Ok(DoctorReport {
        version: info.version,
        commit: info.commit,
        release_tag: info.release_tag,
        source_fingerprint: info.source_fingerprint,
        source_root: source.map(|p| p.display().to_string()),
        binary_matches_source,
        hosts: vec![claude_status, codex_status, agy_status],
        legacy_residues: residues,
    })
}

pub fn format_human(r: &DoctorReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Agent-On Doctor · v{} ({})\n",
        r.version,
        r.release_tag.unwrap_or(r.commit)
    ));
    out.push_str(&format!("CLI 源码指纹：{}\n", r.source_fingerprint));
    match (&r.source_root, r.binary_matches_source) {
        (Some(src), Some(true)) => {
            out.push_str(&format!(
                "源仓库路径  ：{src}（与当前 CLI 二进制完全一致）\n"
            ));
        }
        (Some(src), Some(false)) => {
            out.push_str(&format!(
                "源仓库路径  ：{src}（⚠ 当前二进制与源仓库不一致，可运行 `cargo install --path {src}/cli --force` 更新）\n"
            ));
        }
        (Some(src), None) => {
            out.push_str(&format!("源仓库路径  ：{src}\n"));
        }
        (None, _) => {
            out.push_str("源仓库路径  ：未找到（通过 `--source` 或 `AGENT_ON_ROOT` 指定）\n");
        }
    }

    out.push_str("\n三端挂载状态：\n");
    for h in &r.hosts {
        out.push_str(&format!("  - {:<13} {}\n", h.host, h.detail));
    }

    if !r.legacy_residues.is_empty() {
        out.push_str(
            "\n检测到 v0.x 旧版残留（运行 `agent-on install --clean-legacy` 可一键清理）：\n",
        );
        for res in &r.legacy_residues {
            out.push_str(&format!(
                "  ~ [{}] {}\n    处理：{}\n",
                res.scope, res.path, res.remediation
            ));
        }
    } else {
        out.push_str("\n旧版残留检查：干净（无 v0.x 钩子或旧单路由残留）\n");
    }

    out
}

pub fn run(args: &DoctorArgs) -> i32 {
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
