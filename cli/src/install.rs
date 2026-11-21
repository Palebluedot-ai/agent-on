//! Zero-intrusion cross-host installer (`agent-on install`).

use crate::git_util::{canonical_or_raw, git_raw, git_stdout, list_worktrees, resolve_repo};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const SKILLS: &[&str] = &["prove-it", "thin-review", "wt-doctor", "merge-check"];
pub const GUARDRAILS_REL: &str = "rules/core-guardrails.md";
pub const RULES_START: &str = "<!-- agent-on:core-guardrails:start -->";
pub const RULES_END: &str = "<!-- agent-on:core-guardrails:end -->";

#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    /// Path to the Agent-On v2 repository (auto-detected if omitted)
    #[arg(long)]
    pub source: Option<PathBuf>,
    /// Override user home directory (defaults to $HOME / $USERPROFILE)
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Host targets to configure: claude, codex, antigravity, or all
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "claude,codex,antigravity"
    )]
    pub targets: Vec<String>,
    /// Also sync the 1-page core-guardrails block into ~/.claude/CLAUDE.md and ~/.codex/AGENTS.md
    #[arg(long)]
    pub with_rules: bool,
    /// Clean up v0.x legacy skill symlinks, host PreToolUse hooks, and repo `.git/agent-on/hooks`
    #[arg(long)]
    pub clean_legacy: bool,
    /// Repository path when cleaning legacy Git hooks (defaults to current directory if in a Git repo)
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Remove Agent-On symlinks and managed guardrail blocks
    #[arg(long)]
    pub uninstall: bool,
    /// Preview actions without modifying any files
    #[arg(long)]
    pub dry_run: bool,
    /// Output machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallAction {
    pub target: String,
    pub path: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallReport {
    pub mode: String,
    pub dry_run: bool,
    pub source_root: Option<String>,
    pub home: String,
    pub actions: Vec<InstallAction>,
    pub conflicts: Vec<String>,
}

#[derive(Deserialize)]
struct ConfigFile {
    work_root: Option<String>,
}

#[derive(Deserialize)]
struct LegacyHooksJson {
    #[serde(default)]
    previous_hooks_path: Option<String>,
}

pub fn default_home() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn looks_like_v2_repo(path: &Path) -> bool {
    path.join("CHARTER.md").is_file()
        && path.join(GUARDRAILS_REL).is_file()
        && SKILLS
            .iter()
            .all(|s| path.join("skills").join(s).join("SKILL.md").is_file())
}

/// If `candidate` is a linked worktree of Agent-On, prefer the primary non-bare worktree
/// so symlinks survive pruning ephemeral worktrees.
fn normalize_v2_repo(candidate: &Path) -> Option<PathBuf> {
    let canon = canonical_or_raw(candidate);
    if !looks_like_v2_repo(&canon) {
        return None;
    }
    if let Ok(wts) = list_worktrees(&canon) {
        if let Some(primary) = wts.into_iter().find(|w| !w.bare && !w.missing) {
            if looks_like_v2_repo(&primary.path) {
                return Some(primary.path);
            }
        }
    }
    Some(canon)
}

pub fn resolve_source_root(
    explicit: Option<&Path>,
    home: &Path,
) -> Result<Option<PathBuf>, String> {
    if let Some(p) = explicit {
        return normalize_v2_repo(p).map(Some).ok_or_else(|| {
            format!(
                "指定的 `--source` 路径 `{}` 不是有效的 Agent-On v2 仓库",
                p.display()
            )
        });
    }

    if let Some(env_root) = env::var_os("AGENT_ON_ROOT") {
        if let Some(found) = normalize_v2_repo(Path::new(&env_root)) {
            return Ok(Some(found));
        }
    }

    let cfg_path = home.join(".config/agent-on/config.json");
    if let Ok(raw) = fs::read_to_string(&cfg_path) {
        if let Ok(cfg) = serde_json::from_str::<ConfigFile>(&raw) {
            if let Some(wr) = cfg.work_root {
                if let Some(found) = normalize_v2_repo(Path::new(&wr)) {
                    return Ok(Some(found));
                }
            }
        }
    }

    let built = Path::new(env!("AGENT_ON_SOURCE_ROOT"));
    if let Some(found) = normalize_v2_repo(built) {
        return Ok(Some(found));
    }

    let default_local = home.join(".local/share/agent-on");
    if let Some(found) = normalize_v2_repo(&default_local) {
        return Ok(Some(found));
    }

    if let Ok(cwd) = env::current_dir() {
        for anc in canonical_or_raw(&cwd).ancestors() {
            if let Some(found) = normalize_v2_repo(anc) {
                return Ok(Some(found));
            }
        }
    }

    Ok(None)
}

fn create_dir_link(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(src, dst)
    }
}

fn remove_symlink(dst: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::remove_file(dst)
    }
    #[cfg(windows)]
    {
        fs::remove_file(dst).or_else(|_| fs::remove_dir(dst))
    }
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn has_agent_on_structural_suffix(p: &Path) -> bool {
    let slash = p
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase();
    if slash.ends_with("/agent-on")
        || slash.ends_with("/agent-on/skill")
        || slash.ends_with("/legacy/v0.30-scaffold/skill")
    {
        return true;
    }
    SKILLS
        .iter()
        .any(|s| slash.ends_with(&format!("/agent-on/skills/{s}")))
}

pub fn symlink_points_to_agent_on(link_path: &Path, expected_src: Option<&Path>) -> bool {
    let Ok(raw_target) = fs::read_link(link_path) else {
        return false;
    };
    let joined = if raw_target.is_absolute() {
        raw_target.clone()
    } else {
        link_path
            .parent()
            .map(|p| p.join(&raw_target))
            .unwrap_or_else(|| raw_target.clone())
    };
    let resolved = canonical_or_raw(&joined);

    if let Some(exp) = expected_src {
        if resolved == canonical_or_raw(exp) {
            return true;
        }
    }
    if looks_like_v2_repo(&resolved) {
        return true;
    }
    if resolved
        .parent()
        .and_then(|p| p.parent())
        .is_some_and(looks_like_v2_repo)
    {
        return true;
    }
    has_agent_on_structural_suffix(&raw_target) || has_agent_on_structural_suffix(&resolved)
}

fn atomic_write(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let file_name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("target");
    let tmp = parent.join(format!(".{file_name}.agent-on-tmp-{}", std::process::id()));
    fs::write(&tmp, bytes)?;
    if let Ok(meta) = fs::metadata(dest) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    if let Err(e) = fs::rename(&tmp, dest) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

fn ensure_symlink(
    target_label: &str,
    src: &Path,
    dst: &Path,
    dry_run: bool,
    actions: &mut Vec<InstallAction>,
    conflicts: &mut Vec<String>,
) {
    let src_canon = canonical_or_raw(src);
    if is_symlink(dst) {
        if let Ok(existing_raw) = fs::read_link(dst) {
            let existing_resolved = if existing_raw.is_absolute() {
                canonical_or_raw(&existing_raw)
            } else {
                dst.parent()
                    .map(|p| canonical_or_raw(&p.join(&existing_raw)))
                    .unwrap_or(existing_raw)
            };
            if existing_resolved == src_canon && dst.exists() {
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dst.display().to_string(),
                    status: "unchanged".to_string(),
                    detail: format!("-> {}", src_canon.display()),
                });
                return;
            }
        }
        // Protect ALL non-Agent-On symlinks, including dangling ones (`!dst.exists()`)
        if !symlink_points_to_agent_on(dst, Some(&src_canon)) {
            let msg = format!("{} 已指向非 Agent-On 软链目标，拒绝覆盖", dst.display());
            conflicts.push(msg.clone());
            actions.push(InstallAction {
                target: target_label.to_string(),
                path: dst.display().to_string(),
                status: "conflict".to_string(),
                detail: msg,
            });
            return;
        }
        if !dry_run {
            if let Err(e) = remove_symlink(dst) {
                let msg = format!("移除旧软链失败：{e}");
                conflicts.push(format!("{}: {msg}", dst.display()));
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dst.display().to_string(),
                    status: "error".to_string(),
                    detail: msg,
                });
                return;
            }
            if let Err(e) = create_dir_link(&src_canon, dst) {
                let msg = format!("更新软链失败：{e}");
                conflicts.push(format!("{}: {msg}", dst.display()));
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dst.display().to_string(),
                    status: "error".to_string(),
                    detail: msg,
                });
                return;
            }
        }
        actions.push(InstallAction {
            target: target_label.to_string(),
            path: dst.display().to_string(),
            status: "updated".to_string(),
            detail: format!("-> {}", src_canon.display()),
        });
        return;
    }

    if dst.exists() {
        let msg = format!("{} 已存在非软链目录或文件，拒绝覆盖", dst.display());
        conflicts.push(msg.clone());
        actions.push(InstallAction {
            target: target_label.to_string(),
            path: dst.display().to_string(),
            status: "conflict".to_string(),
            detail: msg,
        });
        return;
    }

    if !dry_run {
        if let Some(parent) = dst.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                let msg = format!("创建父目录失败：{e}");
                conflicts.push(format!("{}: {msg}", dst.display()));
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dst.display().to_string(),
                    status: "error".to_string(),
                    detail: msg,
                });
                return;
            }
        }
        if let Err(e) = create_dir_link(&src_canon, dst) {
            let msg = format!("创建软链失败：{e}");
            conflicts.push(format!("{}: {msg}", dst.display()));
            actions.push(InstallAction {
                target: target_label.to_string(),
                path: dst.display().to_string(),
                status: "error".to_string(),
                detail: msg,
            });
            return;
        }
    }

    actions.push(InstallAction {
        target: target_label.to_string(),
        path: dst.display().to_string(),
        status: "linked".to_string(),
        detail: format!("-> {}", src_canon.display()),
    });
}

fn remove_owned_symlink(
    target_label: &str,
    dst: &Path,
    expected_src: Option<&Path>,
    dry_run: bool,
    actions: &mut Vec<InstallAction>,
    conflicts: &mut Vec<String>,
) {
    if !is_symlink(dst) {
        return;
    }
    if !symlink_points_to_agent_on(dst, expected_src) {
        actions.push(InstallAction {
            target: target_label.to_string(),
            path: dst.display().to_string(),
            status: "skipped".to_string(),
            detail: "软链不指向 Agent-On，保持不动".to_string(),
        });
        return;
    }
    if !dry_run {
        if let Err(e) = remove_symlink(dst) {
            let msg = format!("移除软链失败：{e}");
            conflicts.push(format!("{}: {msg}", dst.display()));
            actions.push(InstallAction {
                target: target_label.to_string(),
                path: dst.display().to_string(),
                status: "error".to_string(),
                detail: msg,
            });
            return;
        }
    }
    actions.push(InstallAction {
        target: target_label.to_string(),
        path: dst.display().to_string(),
        status: "removed".to_string(),
        detail: "已移除软链".to_string(),
    });
}

pub fn upsert_rules_block(existing: &str, guardrails_body: &str) -> String {
    let block = format!("{RULES_START}\n{}\n{RULES_END}", guardrails_body.trim_end());
    if let (Some(start), Some(end_rel)) = (existing.find(RULES_START), existing.find(RULES_END)) {
        if start < end_rel {
            let after_end = end_rel + RULES_END.len();
            let mut out = String::new();
            out.push_str(&existing[..start]);
            out.push_str(&block);
            out.push_str(&existing[after_end..]);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            return out;
        }
    }
    let trimmed = existing.trim_end();
    if trimmed.is_empty() {
        format!("{block}\n")
    } else {
        format!("{trimmed}\n\n{block}\n")
    }
}

pub fn strip_rules_block(existing: &str) -> Option<String> {
    let start = existing.find(RULES_START)?;
    let end_rel = existing.find(RULES_END)?;
    if start >= end_rel {
        return None;
    }
    let after_end = end_rel + RULES_END.len();
    let before = existing[..start].trim_end();
    let after = existing[after_end..].trim_start_matches('\n');
    let combined = match (before.is_empty(), after.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("{before}\n"),
        (true, false) => after.to_string(),
        (false, false) => format!("{before}\n\n{after}"),
    };
    Some(combined)
}

struct RulesSyncSpec<'a> {
    target_label: &'a str,
    dest: &'a Path,
    guardrails_src_canon: Option<&'a Path>,
    guardrails_body: &'a str,
    uninstall: bool,
    dry_run: bool,
}

fn sync_rules_file(
    spec: &RulesSyncSpec<'_>,
    actions: &mut Vec<InstallAction>,
    conflicts: &mut Vec<String>,
) {
    let RulesSyncSpec {
        target_label,
        dest,
        guardrails_src_canon,
        guardrails_body,
        uninstall,
        dry_run,
    } = *spec;
    let (write_target, existing) = match fs::symlink_metadata(dest) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            if uninstall {
                return;
            }
            (dest.to_path_buf(), String::new())
        }
        Err(e) => {
            let msg = format!("读取 {} 元数据失败：{e}", dest.display());
            conflicts.push(msg.clone());
            actions.push(InstallAction {
                target: target_label.to_string(),
                path: dest.display().to_string(),
                status: "error".to_string(),
                detail: msg,
            });
            return;
        }
        Ok(meta) => {
            if meta.file_type().is_symlink() && !dest.exists() {
                let msg = format!("{} 是悬空软链，拒绝覆写以免丢失目标", dest.display());
                conflicts.push(msg.clone());
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dest.display().to_string(),
                    status: "conflict".to_string(),
                    detail: msg,
                });
                return;
            }
            let real_dest = canonical_or_raw(dest);
            if guardrails_src_canon.is_some_and(|src| src == real_dest) {
                actions.push(InstallAction {
                    target: target_label.to_string(),
                    path: dest.display().to_string(),
                    status: "unchanged".to_string(),
                    detail: "已直接软链至 Agent-On 自身规则源文件，无需注入标记".to_string(),
                });
                return;
            }
            if uninstall {
                match fs::read(&real_dest) {
                    Ok(raw) => {
                        if !raw
                            .windows(RULES_START.len())
                            .any(|w| w == RULES_START.as_bytes())
                        {
                            return;
                        }
                    }
                    Err(e) => {
                        let msg = format!("读取 {} 失败：{e}", dest.display());
                        conflicts.push(msg.clone());
                        actions.push(InstallAction {
                            target: target_label.to_string(),
                            path: dest.display().to_string(),
                            status: "error".to_string(),
                            detail: msg,
                        });
                        return;
                    }
                }
            }
            match fs::read_to_string(&real_dest) {
                Ok(content) => (real_dest, content),
                Err(e) => {
                    let msg = format!(
                        "无法安全读取 {}（拒绝以空内容覆盖现有文件）：{e}",
                        dest.display()
                    );
                    conflicts.push(msg.clone());
                    actions.push(InstallAction {
                        target: target_label.to_string(),
                        path: dest.display().to_string(),
                        status: "error".to_string(),
                        detail: msg,
                    });
                    return;
                }
            }
        }
    };

    if uninstall {
        if let Some(updated) = strip_rules_block(&existing) {
            if !dry_run {
                if let Err(e) = atomic_write(&write_target, updated.as_bytes()) {
                    conflicts.push(format!("{}: 写入失败：{e}", dest.display()));
                    return;
                }
            }
            actions.push(InstallAction {
                target: target_label.to_string(),
                path: dest.display().to_string(),
                status: "removed-block".to_string(),
                detail: "已移除核心铁律受管段落（保留其余用户内容）".to_string(),
            });
        }
        return;
    }

    let next = upsert_rules_block(&existing, guardrails_body);
    if next == existing {
        actions.push(InstallAction {
            target: target_label.to_string(),
            path: dest.display().to_string(),
            status: "unchanged".to_string(),
            detail: "核心铁律受管段落已是最新".to_string(),
        });
        return;
    }
    if !dry_run {
        if let Err(e) = atomic_write(&write_target, next.as_bytes()) {
            conflicts.push(format!("{}: 写入失败：{e}", dest.display()));
            return;
        }
    }
    actions.push(InstallAction {
        target: target_label.to_string(),
        path: dest.display().to_string(),
        status: "synced-rules".to_string(),
        detail: "已写入/更新核心铁律受管段落".to_string(),
    });
}

pub fn is_legacy_agent_on_hook_command(cmd: &str) -> bool {
    cmd.contains("agent-on-git-guard")
        || cmd.contains("agent-on guard")
        || cmd.contains("agent-on capture")
}

/// Remove only legacy `agent-on` hook entries from a Claude `settings.json` or Codex `hooks.json`
/// value, preserving all sibling hooks (e.g. `Stop -> auto-sync.sh`) and unrelated settings keys.
pub fn strip_legacy_hooks_from_json(doc: &mut Value, remove_empty_hooks_root: bool) -> usize {
    let Some(root_obj) = doc.as_object_mut() else {
        return 0;
    };
    let mut removed = 0;
    let empty_hooks = if let Some(hooks_val) = root_obj.get_mut("hooks") {
        if let Some(events_obj) = hooks_val.as_object_mut() {
            let event_names: Vec<String> = events_obj.keys().cloned().collect();
            for event in event_names {
                let mut keep_groups = Vec::new();
                if let Some(groups) = events_obj.get_mut(&event).and_then(Value::as_array_mut) {
                    for mut group in std::mem::take(groups) {
                        if let Some(inner_hooks) = group
                            .as_object_mut()
                            .and_then(|g| g.get_mut("hooks"))
                            .and_then(Value::as_array_mut)
                        {
                            let before = inner_hooks.len();
                            inner_hooks.retain(|h| {
                                !h.get("command")
                                    .and_then(Value::as_str)
                                    .is_some_and(is_legacy_agent_on_hook_command)
                            });
                            removed += before - inner_hooks.len();
                            if !inner_hooks.is_empty() {
                                keep_groups.push(group);
                            }
                        } else {
                            keep_groups.push(group);
                        }
                    }
                }
                if keep_groups.is_empty() {
                    events_obj.remove(&event);
                } else {
                    events_obj.insert(event, Value::Array(keep_groups));
                }
            }
            events_obj.is_empty()
        } else {
            false
        }
    } else {
        false
    };

    if removed > 0 && empty_hooks && remove_empty_hooks_root {
        root_obj.remove("hooks");
    }
    removed
}

fn clean_legacy_host_hook_file(
    cfg_path: &Path,
    remove_empty_hooks_root: bool,
    dry_run: bool,
    actions: &mut Vec<InstallAction>,
    conflicts: &mut Vec<String>,
) {
    let meta = match fs::symlink_metadata(cfg_path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return,
        Err(e) => {
            conflicts.push(format!("读取 {} 失败：{e}", cfg_path.display()));
            return;
        }
        Ok(m) => m,
    };
    if meta.file_type().is_symlink() && !cfg_path.exists() {
        conflicts.push(format!(
            "{} 是悬空软链，跳过旧 hook 清理",
            cfg_path.display()
        ));
        return;
    }
    let real_path = canonical_or_raw(cfg_path);
    let raw = match fs::read_to_string(&real_path) {
        Ok(s) => s,
        Err(e) => {
            conflicts.push(format!("读取 {} 失败：{e}", cfg_path.display()));
            return;
        }
    };
    if !is_legacy_agent_on_hook_command(&raw) {
        return;
    }
    let mut doc: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            conflicts.push(format!(
                "解析 {} JSON 失败，未修改文件：{e}",
                cfg_path.display()
            ));
            return;
        }
    };
    let removed = strip_legacy_hooks_from_json(&mut doc, remove_empty_hooks_root);
    if removed == 0 {
        return;
    }
    if !dry_run {
        let mut rendered = serde_json::to_string_pretty(&doc).unwrap();
        rendered.push('\n');
        if let Err(e) = atomic_write(&real_path, rendered.as_bytes()) {
            conflicts.push(format!("写回 {} 失败：{e}", cfg_path.display()));
            return;
        }
    }
    actions.push(InstallAction {
        target: "legacy-host-hooks".to_string(),
        path: cfg_path.display().to_string(),
        status: "cleaned".to_string(),
        detail: format!("已移除 {removed} 条旧版 PreToolUse / SessionStart 钩子（保留其余配置）"),
    });
}

fn clean_legacy_repo_hooks(
    repo_candidate: &Path,
    dry_run: bool,
    actions: &mut Vec<InstallAction>,
    conflicts: &mut Vec<String>,
) {
    let Ok(ctx) = resolve_repo(repo_candidate) else {
        return;
    };

    let local_has_legacy = git_stdout(
        &ctx.current_worktree,
        &["config", "--local", "--get-all", "core.hooksPath"],
    )
    .map(|s| s.lines().any(|l| l.contains("agent-on/hooks")))
    .unwrap_or(false);

    let worktree_cfg_enabled = git_stdout(
        &ctx.current_worktree,
        &["config", "--bool", "--get", "extensions.worktreeConfig"],
    )
    .map(|v| v.trim() == "true")
    .unwrap_or(false);

    let mut wt_legacy_paths = Vec::new();
    if worktree_cfg_enabled {
        let wts = list_worktrees(&ctx.current_worktree).unwrap_or_default();
        for wt in wts.iter().filter(|w| !w.bare && !w.missing) {
            if git_stdout(
                &wt.path,
                &["config", "--worktree", "--get-all", "core.hooksPath"],
            )
            .map(|s| s.lines().any(|l| l.contains("agent-on/hooks")))
            .unwrap_or(false)
            {
                wt_legacy_paths.push(wt.path.clone());
            }
        }
    }

    if !local_has_legacy && wt_legacy_paths.is_empty() {
        return;
    }

    let meta_path = ctx.common_git_dir.join("agent-on/worktree-hooks.json");
    let previous = fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str::<LegacyHooksJson>(&s).ok())
        .and_then(|m| m.previous_hooks_path)
        .filter(|s| !s.trim().is_empty());

    if !dry_run {
        if local_has_legacy {
            let out = if let Some(ref prev) = previous {
                git_raw(
                    &ctx.current_worktree,
                    &["config", "--local", "--replace-all", "core.hooksPath", prev],
                )
            } else {
                git_raw(
                    &ctx.current_worktree,
                    &["config", "--local", "--unset-all", "core.hooksPath"],
                )
            };
            match out {
                Ok(o)
                    if o.status.success() || (previous.is_none() && o.status.code() == Some(5)) => {
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    conflicts.push(format!("恢复/清除 local core.hooksPath 失败：{stderr}"));
                    return;
                }
                Err(e) => {
                    conflicts.push(format!("恢复/清除 local core.hooksPath 失败：{e}"));
                    return;
                }
            }
        }

        for wt_path in &wt_legacy_paths {
            match git_raw(
                wt_path,
                &["config", "--worktree", "--unset-all", "core.hooksPath"],
            ) {
                Ok(o) if o.status.success() || o.status.code() == Some(5) => {}
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    conflicts.push(format!(
                        "清除 {} 的 worktree core.hooksPath 失败：{stderr}",
                        wt_path.display()
                    ));
                    return;
                }
                Err(e) => {
                    conflicts.push(format!(
                        "清除 {} 的 worktree core.hooksPath 失败：{e}",
                        wt_path.display()
                    ));
                    return;
                }
            }
        }
    }

    let detail = match previous {
        Some(prev) => format!("已将旧版 agent-on Git hooks 恢复为原 core.hooksPath={prev}"),
        None => "已移除旧版 core.hooksPath=.git/agent-on/hooks 配置".to_string(),
    };
    actions.push(InstallAction {
        target: "legacy-git-hooks".to_string(),
        path: ctx.common_git_dir.display().to_string(),
        status: "cleaned".to_string(),
        detail,
    });
}

fn parse_targets(raw: &[String]) -> Result<BTreeSet<String>, String> {
    let mut set = BTreeSet::new();
    for item in raw {
        match item.trim().to_lowercase().as_str() {
            "" => {}
            "all" => {
                set.insert("claude".to_string());
                set.insert("codex".to_string());
                set.insert("antigravity".to_string());
            }
            "claude" | "claude-code" => {
                set.insert("claude".to_string());
            }
            "codex" | "agents" => {
                set.insert("codex".to_string());
            }
            "antigravity" | "gemini" | "agy" => {
                set.insert("antigravity".to_string());
            }
            other => {
                return Err(format!(
                    "未知目标 `{other}`（可选：claude, codex, antigravity, all）"
                ));
            }
        }
    }
    Ok(set)
}

pub fn execute(args: &InstallArgs) -> Result<(InstallReport, i32), String> {
    let home = args.home.clone().unwrap_or_else(default_home);
    let targets = parse_targets(&args.targets)?;
    let source = resolve_source_root(args.source.as_deref(), &home)?;

    if !args.uninstall && source.is_none() {
        return Err(
            "未能定位 Agent-On v2 仓库根目录；请通过 `--source <路径>` 或环境变量 `AGENT_ON_ROOT` 指定"
                .to_string(),
        );
    }

    let guardrails_src_canon = source
        .as_ref()
        .map(|s| canonical_or_raw(&s.join(GUARDRAILS_REL)));

    let mut actions = Vec::new();
    let mut conflicts = Vec::new();

    if args.uninstall {
        if targets.contains("claude") {
            for skill in SKILLS {
                let dst = home.join(".claude/skills").join(skill);
                let exp = source.as_ref().map(|s| s.join("skills").join(skill));
                remove_owned_symlink(
                    "claude",
                    &dst,
                    exp.as_deref(),
                    args.dry_run,
                    &mut actions,
                    &mut conflicts,
                );
            }
            sync_rules_file(
                &RulesSyncSpec {
                    target_label: "claude-rules",
                    dest: &home.join(".claude/CLAUDE.md"),
                    guardrails_src_canon: guardrails_src_canon.as_deref(),
                    guardrails_body: "",
                    uninstall: true,
                    dry_run: args.dry_run,
                },
                &mut actions,
                &mut conflicts,
            );
        }
        if targets.contains("codex") {
            for skill in SKILLS {
                let dst = home.join(".agents/skills").join(skill);
                let exp = source.as_ref().map(|s| s.join("skills").join(skill));
                remove_owned_symlink(
                    "codex",
                    &dst,
                    exp.as_deref(),
                    args.dry_run,
                    &mut actions,
                    &mut conflicts,
                );
            }
            sync_rules_file(
                &RulesSyncSpec {
                    target_label: "codex-rules",
                    dest: &home.join(".codex/AGENTS.md"),
                    guardrails_src_canon: guardrails_src_canon.as_deref(),
                    guardrails_body: "",
                    uninstall: true,
                    dry_run: args.dry_run,
                },
                &mut actions,
                &mut conflicts,
            );
        }
        if targets.contains("antigravity") {
            let dst = home.join(".gemini/config/plugins/agent-on");
            remove_owned_symlink(
                "antigravity",
                &dst,
                source.as_deref(),
                args.dry_run,
                &mut actions,
                &mut conflicts,
            );
        }
    } else if let Some(ref src_root) = source {
        let guardrails_body = fs::read_to_string(src_root.join(GUARDRAILS_REL))
            .map_err(|e| format!("读取 {GUARDRAILS_REL} 失败：{e}"))?;

        if targets.contains("claude") {
            for skill in SKILLS {
                let src = src_root.join("skills").join(skill);
                let dst = home.join(".claude/skills").join(skill);
                ensure_symlink(
                    "claude",
                    &src,
                    &dst,
                    args.dry_run,
                    &mut actions,
                    &mut conflicts,
                );
            }
            if args.with_rules {
                sync_rules_file(
                    &RulesSyncSpec {
                        target_label: "claude-rules",
                        dest: &home.join(".claude/CLAUDE.md"),
                        guardrails_src_canon: guardrails_src_canon.as_deref(),
                        guardrails_body: &guardrails_body,
                        uninstall: false,
                        dry_run: args.dry_run,
                    },
                    &mut actions,
                    &mut conflicts,
                );
            }
        }

        if targets.contains("codex") {
            for skill in SKILLS {
                let src = src_root.join("skills").join(skill);
                let dst = home.join(".agents/skills").join(skill);
                ensure_symlink(
                    "codex",
                    &src,
                    &dst,
                    args.dry_run,
                    &mut actions,
                    &mut conflicts,
                );
            }
            if args.with_rules {
                sync_rules_file(
                    &RulesSyncSpec {
                        target_label: "codex-rules",
                        dest: &home.join(".codex/AGENTS.md"),
                        guardrails_src_canon: guardrails_src_canon.as_deref(),
                        guardrails_body: &guardrails_body,
                        uninstall: false,
                        dry_run: args.dry_run,
                    },
                    &mut actions,
                    &mut conflicts,
                );
            }
        }

        if targets.contains("antigravity") {
            let dst = home.join(".gemini/config/plugins/agent-on");
            ensure_symlink(
                "antigravity",
                src_root,
                &dst,
                args.dry_run,
                &mut actions,
                &mut conflicts,
            );
        }
    }

    if args.clean_legacy {
        for legacy_link in [
            home.join(".claude/skills/agent-on"),
            home.join(".agents/skills/agent-on"),
        ] {
            remove_owned_symlink(
                "legacy-skill-router",
                &legacy_link,
                None,
                args.dry_run,
                &mut actions,
                &mut conflicts,
            );
        }
        clean_legacy_host_hook_file(
            &home.join(".claude/settings.json"),
            true,
            args.dry_run,
            &mut actions,
            &mut conflicts,
        );
        clean_legacy_host_hook_file(
            &home.join(".codex/hooks.json"),
            false,
            args.dry_run,
            &mut actions,
            &mut conflicts,
        );
        let repo_target = args.repo.clone().or_else(|| env::current_dir().ok());
        if let Some(repo_dir) = repo_target {
            clean_legacy_repo_hooks(&repo_dir, args.dry_run, &mut actions, &mut conflicts);
        }
    }

    let code = if conflicts.is_empty() { 0 } else { 1 };
    let report = InstallReport {
        mode: if args.uninstall {
            "uninstall".to_string()
        } else {
            "install".to_string()
        },
        dry_run: args.dry_run,
        source_root: source.map(|p| p.display().to_string()),
        home: home.display().to_string(),
        actions,
        conflicts,
    };
    Ok((report, code))
}

pub fn format_human(r: &InstallReport) -> String {
    let mut out = String::new();
    let dry = if r.dry_run {
        "（预览 dry-run）"
    } else {
        ""
    };
    out.push_str(&format!("Agent-On {} {dry}\n", r.mode));
    if let Some(ref src) = r.source_root {
        out.push_str(&format!("源仓库：{src}\n"));
    }
    out.push_str(&format!("用户目录：{}\n\n", r.home));

    if r.actions.is_empty() {
        out.push_str("无需变更。\n");
    } else {
        for a in &r.actions {
            out.push_str(&format!(
                "  [{:<18}] {:<14} {} ({})\n",
                a.target, a.status, a.path, a.detail
            ));
        }
    }

    if !r.conflicts.is_empty() {
        out.push_str("\n存在未覆盖冲突或错误：\n");
        for c in &r.conflicts {
            out.push_str(&format!("  ! {c}\n"));
        }
    }
    out
}

pub fn run(args: &InstallArgs) -> i32 {
    match execute(args) {
        Ok((report, code)) => {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print!("{}", format_human(&report));
            }
            code
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            2
        }
    }
}
