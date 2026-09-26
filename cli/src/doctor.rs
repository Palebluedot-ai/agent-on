//! `agent-on doctor` beyond path resolution: which agent-on hooks the host
//! actually runs (the execution surface), and whether this session sits in a
//! linked worktree instead of WRITE_ROOT's main tree.
//!
//! Read-only by construction. Nothing here writes, least of all under
//! `~/.claude`: replacing a stale hook is the user's action.

use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn git_out(cwd: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn absolute_git_dir(cwd: &Path, flag: &str) -> Option<PathBuf> {
    git_out(cwd, &["rev-parse", "--path-format=absolute", flag])
        .map(|raw| canonical(Path::new(&raw)))
}

/// One line when `cwd` is inside a linked worktree, `None` otherwise.
///
/// The intake queue and any unfinished canonical edits live only in the main
/// tree, so a digest run from a linked worktree sees an empty queue and a clean
/// tree and concludes "no backlog" (settlement lower half, step 0, check 4).
pub fn worktree_line(cwd: &Path, work_root: Option<&Path>) -> Option<String> {
    let git_dir = absolute_git_dir(cwd, "--git-dir")?;
    let common = absolute_git_dir(cwd, "--git-common-dir")?;
    if git_dir == common {
        return None;
    }
    let top = git_out(cwd, &["rev-parse", "--show-toplevel"])
        .map(|raw| canonical(Path::new(&raw)).display().to_string())
        .unwrap_or_else(|| cwd.display().to_string());
    let main = git_out(cwd, &["worktree", "list", "--porcelain"])
        .and_then(|raw| {
            raw.lines()
                .find_map(|line| line.strip_prefix("worktree "))
                .map(|path| canonical(Path::new(path)).display().to_string())
        })
        .unwrap_or_else(|| "?".to_string());
    let mut line = format!("worktree      = linked {top}  [主树 {main}]");
    let write_roots_repo = work_root
        .and_then(|root| absolute_git_dir(root, "--git-common-dir"))
        .is_some_and(|root_common| root_common == common);
    if write_roots_repo {
        line.push_str("  不是 WRITE_ROOT 主树：收件 / 消化在这里看不见主树的承接队列（settlement 下半场第 0 步第四检）");
    }
    Some(line)
}

/// The "hook 执行面" section: every agent-on hook the host runs, compared
/// with the READ_ROOT copy it should match.
pub fn hook_surface(home: &Path, read_root: Option<&Path>) -> Vec<String> {
    surface(home, read_root, env::var_os("PATH"))
}

/// The shim's own forwarding order (kit/guard/agent-on-git-guard), mirrored so
/// doctor can name the binary a hook really runs. A test pins the two together.
const RELEASE_BINARY: &str = "cli/target/release/agent-on";
const SHIMS: &[&str] = &[
    "kit/guard/agent-on-git-guard",
    "kit/guard/agent-on-git-guard.sh",
];

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn on_path(name: &str, path_var: Option<&OsString>) -> Option<PathBuf> {
    env::split_paths(path_var?)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

fn blob(path: &Path) -> String {
    Command::new("git")
        .args(["hash-object", "--no-filters", "--"])
        .arg(path)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .chars()
                .take(12)
                .collect()
        })
        .unwrap_or_else(|| "?".to_string())
}

fn epoch(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|age| age.as_secs())
        .unwrap_or(0)
}

fn local_time(secs: u64) -> String {
    let time = UNIX_EPOCH + std::time::Duration::from_secs(secs);
    chrono::DateTime::<chrono::Local>::from(time)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn plugin_version(root: &Path) -> Option<String> {
    read_json(&root.join(".claude-plugin/plugin.json"))?
        .get("version")?
        .as_str()
        .map(str::to_string)
}

/// The last commit that touched the guard's source in READ_ROOT. A binary
/// built before it cannot contain it.
struct SourceChange {
    epoch: u64,
    sha: String,
}

fn last_source_change(read_root: &Path) -> Option<SourceChange> {
    let raw = git_out(
        read_root,
        &["log", "-1", "--format=%ct%x09%h", "--", "cli/src"],
    )?;
    let (secs, sha) = raw.split_once('\t')?;
    Some(SourceChange {
        epoch: secs.parse().ok()?,
        sha: sha.to_string(),
    })
}

fn find_agent_on_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|dir| crate::paths::looks_like_agent_on(dir))
        .map(Path::to_path_buf)
}

/// Commands grouped by text: `PreToolUse[Bash]` and `PreToolUse[SendMessage]`
/// running the same command are one executor, judged once.
struct HookCommand {
    labels: Vec<String>,
    command: String,
}

fn hook_commands(hooks: Option<&Value>, only_agent_on: bool) -> Vec<HookCommand> {
    let mut out: Vec<HookCommand> = Vec::new();
    let Some(events) = hooks.and_then(Value::as_object) else {
        return out;
    };
    for (event, groups) in events {
        for group in groups.as_array().into_iter().flatten() {
            let matcher = group.get("matcher").and_then(Value::as_str).unwrap_or("");
            for hook in group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(command) = hook.get("command").and_then(Value::as_str) else {
                    continue;
                };
                if only_agent_on && !command.to_lowercase().contains("agent-on") {
                    continue;
                }
                let label = if matcher.is_empty() {
                    event.clone()
                } else {
                    format!("{event}[{matcher}]")
                };
                match out.iter_mut().find(|known| known.command == command) {
                    Some(known) => known.labels.push(label),
                    None => out.push(HookCommand {
                        labels: vec![label],
                        command: command.to_string(),
                    }),
                }
            }
        }
    }
    out
}

fn expand(token: &str, home: &Path, plugin_root: Option<&Path>) -> String {
    let mut text = token.to_string();
    if let Some(root) = plugin_root {
        let root = root.display().to_string();
        text = text
            .replace("${CLAUDE_PLUGIN_ROOT}", &root)
            .replace("$CLAUDE_PLUGIN_ROOT", &root);
    }
    let home = home.display().to_string();
    text = text.replace("${HOME}", &home).replace("$HOME", &home);
    match text.strip_prefix("~/") {
        Some(rest) => format!("{home}/{rest}"),
        None => text,
    }
}

struct Surface<'a> {
    home: &'a Path,
    read_root: Option<&'a Path>,
    source: Option<SourceChange>,
    path_var: Option<OsString>,
    lines: Vec<String>,
    /// Kind-prefixed problems: the executing copy is behind READ_ROOT
    /// (`STALE`), or nothing executes at all (`GUARD OFF`).
    problems: Vec<String>,
    fixes: BTreeSet<String>,
}

impl Surface<'_> {
    /// `fixes` are commands; sorted, `cargo …` rebuilds print before
    /// `claude plugin update …`, which is also the order they must run in.
    fn stale(&mut self, what: String, fixes: Vec<String>) {
        self.problems.push(format!("STALE      {what}"));
        self.fixes.extend(fixes);
    }

    fn plugin_fix(key: &str) -> String {
        format!("claude plugin update {key}（重启 Claude 生效；改 ~/.claude 是用户动作）")
    }

    fn rebuild_fix(root: &Path) -> String {
        format!(
            "cargo build --release --manifest-path {}",
            root.join("cli/Cargo.toml").display()
        )
    }

    /// One hook command: the scripts it runs, each against READ_ROOT, then
    /// the binary a forwarding shim ends up in.
    fn command(&mut self, hook: &HookCommand, plugin: Option<(&Path, &str)>, indent: &str) {
        self.lines.push(format!(
            "{indent}{}  {}",
            hook.labels.join(", "),
            hook.command
        ));
        let plugin_root = plugin.map(|(root, _)| root);
        let Some(tokens) = shlex::split(&hook.command) else {
            self.lines
                .push(format!("{indent}  命令解析不了（引号不成对），没法核脚本"));
            return;
        };
        if tokens.first().map(String::as_str) == Some("agent-on") {
            self.binary(None, plugin, indent);
            return;
        }
        for token in &tokens {
            let expanded = expand(token, self.home, plugin_root);
            if !expanded.contains('/') {
                continue;
            }
            let path = PathBuf::from(&expanded);
            let root = match plugin_root {
                Some(root) if path.starts_with(root) => Some(root.to_path_buf()),
                _ => find_agent_on_root(&path),
            };
            let Some(root) = root else {
                if !path.exists() && expanded.to_lowercase().contains("agent-on") {
                    self.lines.push(format!(
                        "{indent}  {expanded}  不存在：这条 hook 会报错或空转"
                    ));
                    self.problems.push(format!("GUARD OFF  {expanded} 不存在"));
                }
                continue;
            };
            let rel = path
                .strip_prefix(&root)
                .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| expanded.clone());
            if !path.is_file() {
                self.lines
                    .push(format!("{indent}  {rel}  不存在：这条 hook 会报错或空转"));
                self.problems
                    .push(format!("GUARD OFF  {} 不存在", path.display()));
                continue;
            }
            if self.script(&path, &rel, plugin, indent) && SHIMS.contains(&rel.as_str()) {
                self.binary(Some(&root), plugin, indent);
            }
        }
    }

    /// Compare one executing script with READ_ROOT's copy. Returns whether it
    /// matches, i.e. whether it forwards the way READ_ROOT's shim does.
    fn script(
        &mut self,
        path: &Path,
        rel: &str,
        plugin: Option<(&Path, &str)>,
        indent: &str,
    ) -> bool {
        let Some(read_root) = self.read_root else {
            self.lines
                .push(format!("{indent}  {rel}  ({})", blob(path)));
            return true;
        };
        let reference = read_root.join(rel);
        let fix = match plugin {
            Some((_, key)) => Self::plugin_fix(key),
            None => format!("让这条 hook 指向 READ_ROOT 的 {rel}"),
        };
        if canonical(path) == canonical(&reference) {
            self.lines
                .push(format!("{indent}  {rel}  = READ_ROOT 本份"));
            return true;
        }
        let (Ok(here), Ok(there)) = (fs::read(path), fs::read(&reference)) else {
            self.lines
                .push(format!("{indent}  {rel}  READ_ROOT 已没有这个文件"));
            self.stale(
                format!("{} 在 READ_ROOT 已没有对应文件", path.display()),
                vec![fix],
            );
            return false;
        };
        if here == there {
            self.lines.push(format!(
                "{indent}  {rel}  与 READ_ROOT 一致（{}）",
                blob(path)
            ));
            true
        } else {
            self.lines.push(format!(
                "{indent}  {rel}  与 READ_ROOT 不同（这份 {}，READ_ROOT {}）",
                blob(path),
                blob(&reference)
            ));
            self.stale(format!("{} 与 READ_ROOT 不同", path.display()), vec![fix]);
            false
        }
    }

    /// Which binary the shim forwards to, in the shim's own order: the
    /// plugin-local build, the build of the checkout the script sits in, then
    /// PATH. None of them → the shim exits 0 and nothing is guarded.
    fn binary(&mut self, script_root: Option<&Path>, plugin: Option<(&Path, &str)>, indent: &str) {
        let mut candidates = Vec::new();
        if let Some((root, _)) = plugin {
            candidates.push(root.join(RELEASE_BINARY));
        }
        if let Some(root) = script_root {
            candidates.push(root.join(RELEASE_BINARY));
        }
        let found = candidates
            .into_iter()
            .find(|candidate| is_executable(candidate))
            .or_else(|| on_path("agent-on", self.path_var.as_ref()));
        let Some(binary) = found else {
            self.lines.push(format!(
                "{indent}  → 找不到 agent-on 二进制：shim 直接放行（fail-open），闸不生效"
            ));
            self.problems
                .push("GUARD OFF  shim 找不到 agent-on 二进制，闸不生效（fail-open）".to_string());
            return;
        };
        let built = fs::metadata(&binary)
            .and_then(|meta| meta.modified())
            .map(epoch)
            .ok();
        self.lines.push(format!(
            "{indent}  → 二进制 {}{}",
            binary.display(),
            built
                .map(|secs| format!("  编于 {}", local_time(secs)))
                .unwrap_or_default()
        ));
        let (Some(built), Some(source)) = (built, self.source.as_ref()) else {
            return;
        };
        if built >= source.epoch {
            return;
        }
        let mut what = format!(
            "二进制 {} 编于 {}，早于 READ_ROOT 最近一次 cli/src 提交 {}（{}）——不含之后的闸修复",
            binary.display(),
            local_time(built),
            source.sha,
            local_time(source.epoch)
        );
        let read_root = self.read_root.map(Path::to_path_buf);
        let fixes = match (plugin, read_root) {
            // A directory marketplace copies target/ into the cache: rebuild
            // at the source first, then refresh the cache from it.
            (Some((root, key)), Some(read_root)) if binary.starts_with(root) => {
                what.push_str("（目录型 marketplace 把 target/ 一起拷进了缓存）");
                vec![Self::rebuild_fix(&read_root), Self::plugin_fix(key)]
            }
            _ => match script_root.filter(|root| binary.starts_with(root)) {
                Some(root) => vec![Self::rebuild_fix(root)],
                None => vec![format!(
                    "cargo install --path {}",
                    self.read_root
                        .map(|root| root.join("cli").display().to_string())
                        .unwrap_or_else(|| "<READ_ROOT>/cli".to_string())
                )],
            },
        };
        self.stale(what, fixes);
    }
}

fn surface(home: &Path, read_root: Option<&Path>, path_var: Option<OsString>) -> Vec<String> {
    let mut s = Surface {
        home,
        read_root,
        source: read_root.and_then(last_source_change),
        path_var,
        lines: vec!["hook 执行面（只读；改 ~/.claude 是用户动作）".to_string()],
        problems: Vec::new(),
        fixes: BTreeSet::new(),
    };
    let read_root_version = read_root.and_then(plugin_version);
    match read_root {
        Some(root) => {
            s.lines.push(format!(
                "  READ_ROOT   {}  plugin.json {}",
                root.display(),
                read_root_version.as_deref().unwrap_or("?")
            ));
            let cache = canonical(&home.join(".claude/plugins/cache"));
            if canonical(root).starts_with(&cache) {
                s.lines.push(
                    "  注意：READ_ROOT 就是插件缓存本身（CLAUDE_PLUGIN_ROOT），和自己比永远一致；到终端跑或设 AGENT_ON_ROOT 指向工作仓"
                        .to_string(),
                );
            }
        }
        None => s
            .lines
            .push("  READ_ROOT 未解析：只列条目，不做比对".to_string()),
    }

    let settings_path = home.join(".claude/settings.json");
    let settings = read_json(&settings_path);
    let settings_hooks = hook_commands(settings.as_ref().and_then(|v| v.get("hooks")), true);
    let mut executors = settings_hooks.len();
    if !settings_hooks.is_empty() {
        s.lines
            .push(format!("  settings    {}", settings_path.display()));
        for hook in &settings_hooks {
            s.command(hook, None, "    ");
        }
    }

    let enabled: Vec<(String, bool)> = settings
        .as_ref()
        .and_then(|v| v.get("enabledPlugins"))
        .and_then(Value::as_object)
        .map(|plugins| {
            plugins
                .iter()
                .filter(|(key, _)| key.split('@').next() == Some("agent-on"))
                .map(|(key, on)| (key.clone(), on.as_bool() == Some(true)))
                .collect()
        })
        .unwrap_or_default();
    let installed = read_json(&home.join(".claude/plugins/installed_plugins.json"));
    for (key, on) in &enabled {
        if !on {
            s.lines
                .push(format!("  plugin      {key}  未启用：插件 hook 不执行"));
            continue;
        }
        let entries: Vec<&Value> = match installed
            .as_ref()
            .and_then(|v| v.get("plugins"))
            .and_then(|v| v.get(key.as_str()))
        {
            Some(Value::Array(items)) => items.iter().collect(),
            Some(item @ Value::Object(_)) => vec![item],
            _ => Vec::new(),
        };
        if entries.is_empty() {
            s.lines.push(format!(
                "  plugin      {key}  已启用，但 installed_plugins.json 里没有安装记录：插件 hook 不执行"
            ));
        }
        let mut install_paths = Vec::new();
        for entry in entries {
            let Some(install) = entry.get("installPath").and_then(Value::as_str) else {
                continue;
            };
            let install = PathBuf::from(install);
            install_paths.push(canonical(&install));
            let version = entry
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| plugin_version(&install))
                .unwrap_or_else(|| "?".to_string());
            s.lines.push(format!(
                "  plugin      {key}  {version}  {}",
                install.display()
            ));
            if let Some(expected) = &read_root_version {
                if &version != expected {
                    s.lines.push(format!(
                        "    版本 {version} ≠ READ_ROOT plugin.json {expected}"
                    ));
                    s.stale(
                        format!("插件缓存 {key} 是 {version}，READ_ROOT 是 {expected}"),
                        vec![Surface::plugin_fix(key)],
                    );
                }
            }
            let manifest = install.join("hooks/hooks.json");
            if let Some(root) = read_root {
                match (fs::read(&manifest), fs::read(root.join("hooks/hooks.json"))) {
                    (Ok(here), Ok(there)) if here == there => s
                        .lines
                        .push("    hooks.json  与 READ_ROOT 一致".to_string()),
                    (Ok(_), Ok(_)) => {
                        s.lines.push(
                            "    hooks.json  与 READ_ROOT 不同：在跑的那组 hook 不是仓里这组"
                                .to_string(),
                        );
                        s.stale(
                            format!("{} 与 READ_ROOT 不同", manifest.display()),
                            vec![Surface::plugin_fix(key)],
                        );
                    }
                    (Err(_), _) => s
                        .lines
                        .push("    hooks.json  缓存里没有：插件不挂 hook".to_string()),
                    (Ok(_), Err(_)) => s
                        .lines
                        .push("    hooks.json  READ_ROOT 里没有 hooks/hooks.json".to_string()),
                }
            }
            let plugin_hooks = hook_commands(
                read_json(&manifest).as_ref().and_then(|v| v.get("hooks")),
                false,
            );
            executors += plugin_hooks.len();
            for hook in &plugin_hooks {
                s.command(hook, Some((&install, key)), "    ");
            }
        }
        let marketplace = key.split_once('@').map(|(_, m)| m).unwrap_or("agent-on");
        let cache = home
            .join(".claude/plugins/cache")
            .join(marketplace)
            .join("agent-on");
        let mut leftovers: Vec<String> = fs::read_dir(&cache)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && !install_paths.contains(&canonical(path)))
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .collect();
        leftovers.sort();
        if !leftovers.is_empty() {
            s.lines.push(format!(
                "  残留缓存    {}（不在 installed_plugins.json 里，不执行）",
                leftovers.join(", ")
            ));
        }
    }

    if executors == 0 {
        s.lines.push(
            "  宿主上没有 agent-on hook：PreToolUse 闸不生效（装插件，或 `agent-on setup --with-plugins`）"
                .to_string(),
        );
    }
    if s.problems.is_empty() {
        if executors > 0 && read_root.is_some() {
            s.lines
                .push("  结论        执行面与 READ_ROOT 一致".to_string());
        }
    } else {
        s.lines
            .push(format!("  结论        {} 处要处理：", s.problems.len()));
        for problem in &s.problems {
            s.lines.push(format!("    {problem}"));
        }
        for fix in &s.fixes {
            s.lines.push(format!("  修          {fix}"));
        }
    }
    s.lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn git(cwd: &Path, args: &[&str]) {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    fn set_mtime(path: &Path, when: SystemTime) {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(when)
            .unwrap();
    }

    const SHIM: &str = "#!/usr/bin/env bash\n# shim: forwards to agent-on guard\n";
    const POLYGLOT: &str = "#!/bin/sh\n# polyglot: forwards to the shim\n";
    const OLD_PYTHON_GUARD: &str =
        "#!/usr/bin/env python3\n# 0.5.0 guard: judges by command text\n";

    fn plugin_hooks(command: &str) -> String {
        json!({"hooks": {"PreToolUse": [
            {"matcher": "Bash", "hooks": [{"type": "command", "command": command, "timeout": 15}]},
            {"matcher": "SendMessage", "hooks": [{"type": "command", "command": command, "timeout": 15}]}
        ]}})
        .to_string()
    }

    /// An agent-on checkout (READ_ROOT, or a cached copy of one).
    fn agent_on_tree(root: &Path, version: &str, polyglot: &str) {
        write(&root.join("CHARTER.md"), "charter\n");
        write(&root.join("BOOTSTRAP.md"), "bootstrap\n");
        write(
            &root.join(".claude-plugin/plugin.json"),
            &json!({"name": "agent-on", "version": version}).to_string(),
        );
        write(
            &root.join("hooks/hooks.json"),
            &plugin_hooks("bash \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard\""),
        );
        write(&root.join("kit/guard/agent-on-git-guard"), SHIM);
        write(&root.join("kit/guard/agent-on-git-guard.sh"), polyglot);
        let binary = root.join("cli/target/release/agent-on");
        write(&binary, "#!/bin/sh\n");
        make_executable(&binary);
    }

    struct Host {
        _tmp: TempDir,
        home: PathBuf,
        read_root: PathBuf,
    }

    /// A fake HOME whose settings.json carries one agent-on hook pointing at
    /// READ_ROOT itself (and one unrelated hook), with the plugin installed
    /// from `cache_version`.
    fn host(cache_version: &str) -> Host {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let read_root = tmp.path().join("Agent-On");
        agent_on_tree(&read_root, "0.12.1", POLYGLOT);
        let install = home
            .join(".claude/plugins/cache/agent-on/agent-on")
            .join(cache_version);
        write(
            &home.join(".claude/settings.json"),
            &json!({
                "enabledPlugins": {"agent-on@agent-on": true, "other@x": true},
                "hooks": {
                    "PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command",
                        "command": format!("bash \"{}/kit/guard/agent-on-git-guard\"", read_root.display())}]}],
                    "Stop": [{"hooks": [{"type": "command", "command": "bash \"$HOME/x/auto-sync.sh\""}]}]
                }
            })
            .to_string(),
        );
        write(
            &home.join(".claude/plugins/installed_plugins.json"),
            &json!({"version": 2, "plugins": {"agent-on@agent-on": [{
                "scope": "user",
                "installPath": install.display().to_string(),
                "version": cache_version
            }]}})
            .to_string(),
        );
        Host {
            _tmp: tmp,
            home,
            read_root,
        }
    }

    fn cache_dir(host: &Host, version: &str) -> PathBuf {
        host.home
            .join(".claude/plugins/cache/agent-on/agent-on")
            .join(version)
    }

    /// Case 47: the enabled plugin still runs 0.5.0's Python guard.
    fn stale_cache_host() -> Host {
        let host = host("0.5.0");
        let cache = cache_dir(&host, "0.5.0");
        agent_on_tree(&cache, "0.5.0", OLD_PYTHON_GUARD);
        write(
            &cache.join("hooks/hooks.json"),
            &json!({"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command",
                "command": "python3 \"${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard.sh\"", "timeout": 15}]}]}})
            .to_string(),
        );
        host
    }

    /// The plugin runs a cache identical to READ_ROOT; an old 0.5.0 copy is
    /// still on disk but no longer installed.
    fn fresh_cache_host() -> Host {
        let host = host("0.12.1");
        agent_on_tree(&cache_dir(&host, "0.12.1"), "0.12.1", POLYGLOT);
        agent_on_tree(&cache_dir(&host, "0.5.0"), "0.5.0", OLD_PYTHON_GUARD);
        host
    }

    #[test]
    fn stale_plugin_cache_is_flagged_with_the_fix() {
        let host = stale_cache_host();
        let out = surface(&host.home, Some(&host.read_root), None).join("\n");
        assert!(out.contains("hook 执行面"), "{out}");
        // settings.json: the agent-on entry is listed, the unrelated one is not.
        assert!(out.contains("kit/guard/agent-on-git-guard\""), "{out}");
        assert!(!out.contains("auto-sync"), "{out}");
        // The enabled plugin, its version, and the one READ_ROOT carries.
        assert!(out.contains("agent-on@agent-on"), "{out}");
        assert!(out.contains("0.5.0"), "{out}");
        assert!(out.contains("0.12.1"), "{out}");
        // The script the cached hook runs differs from READ_ROOT's copy.
        assert!(out.contains("kit/guard/agent-on-git-guard.sh"), "{out}");
        assert!(out.contains("与 READ_ROOT 不同"), "{out}");
        assert!(out.contains("STALE"), "{out}");
        assert!(
            out.contains("claude plugin update agent-on@agent-on"),
            "{out}"
        );
    }

    #[test]
    fn fresh_plugin_cache_is_consistent_and_leftovers_are_named() {
        let host = fresh_cache_host();
        let out = surface(&host.home, Some(&host.read_root), None).join("\n");
        assert!(!out.contains("STALE"), "{out}");
        assert!(out.contains("与 READ_ROOT 一致"), "{out}");
        assert!(out.contains("= READ_ROOT 本份"), "{out}");
        // Both matchers run the same command; it is judged once.
        assert!(out.contains("PreToolUse[Bash]"), "{out}");
        assert!(out.contains("PreToolUse[SendMessage]"), "{out}");
        // The 0.5.0 copy is on disk but is not what runs.
        let leftover = out
            .lines()
            .find(|line| line.contains("残留"))
            .unwrap_or_else(|| panic!("no leftover line:\n{out}"));
        assert!(leftover.contains("0.5.0"), "{out}");
    }

    /// The cached plugin carries its own `cli/target/release/agent-on`, and
    /// the shim prefers it over PATH. Scripts matching READ_ROOT say nothing
    /// about that binary: it can predate every guard fix since.
    #[test]
    fn a_forwarded_binary_older_than_read_roots_guard_source_is_stale() {
        let host = fresh_cache_host();
        write(&host.read_root.join("cli/src/guard.rs"), "// guard\n");
        git(&host.read_root, &["init", "-q", "-b", "main"]);
        git(&host.read_root, &["config", "user.email", "t@t.com"]);
        git(&host.read_root, &["config", "user.name", "t"]);
        git(&host.read_root, &["add", "."]);
        git(&host.read_root, &["commit", "-qm", "guard fix"]);
        // READ_ROOT's own build is fresh; the cached copy is from 2020.
        set_mtime(
            &host.read_root.join("cli/target/release/agent-on"),
            SystemTime::now() + Duration::from_secs(60),
        );
        let binary = cache_dir(&host, "0.12.1").join("cli/target/release/agent-on");
        set_mtime(
            &binary,
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_577_836_800),
        );

        let out = surface(&host.home, Some(&host.read_root), None).join("\n");
        assert!(out.contains(&binary.display().to_string()), "{out}");
        assert!(out.contains("STALE"), "{out}");
        assert!(out.contains("cli/src"), "{out}");
        assert!(out.contains("cargo build --release"), "{out}");

        // Rebuilt after the fix: nothing to flag.
        set_mtime(&binary, SystemTime::now() + Duration::from_secs(60));
        let out = surface(&host.home, Some(&host.read_root), None).join("\n");
        assert!(!out.contains("STALE"), "{out}");
    }

    /// No build next to the shim and none on PATH: the shim exits 0, so every
    /// hook "runs" and nothing is guarded.
    #[test]
    fn a_shim_with_no_binary_to_forward_to_is_guard_off() {
        let host = fresh_cache_host();
        fs::remove_file(host.read_root.join("cli/target/release/agent-on")).unwrap();
        fs::remove_file(cache_dir(&host, "0.12.1").join("cli/target/release/agent-on")).unwrap();
        let out = surface(&host.home, Some(&host.read_root), None).join("\n");
        assert!(out.contains("GUARD OFF"), "{out}");
        assert!(out.contains("fail-open"), "{out}");
        assert!(!out.contains("执行面与 READ_ROOT 一致"), "{out}");
    }

    #[test]
    fn no_agent_on_hook_on_the_host_says_the_guard_is_off() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let read_root = tmp.path().join("Agent-On");
        agent_on_tree(&read_root, "0.12.1", POLYGLOT);
        write(&home.join(".claude/settings.json"), "{}");
        let out = surface(&home, Some(&read_root), None).join("\n");
        assert!(out.contains("hook 执行面"), "{out}");
        assert!(out.contains("没有 agent-on hook"), "{out}");
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u64, SystemTime)> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                let meta = fs::symlink_metadata(&path).unwrap();
                if meta.is_dir() {
                    stack.push(path.clone());
                }
                out.insert(path, (meta.len(), meta.modified().unwrap()));
            }
        }
        out
    }

    #[test]
    fn the_report_never_writes_under_the_home_claude_dir() {
        let host = stale_cache_host();
        let before = snapshot(&host.home);
        let _ = surface(&host.home, Some(&host.read_root), None);
        assert_eq!(snapshot(&host.home), before);
    }

    /// doctor mirrors the shim's forwarding order to name the binary a hook
    /// really runs. That mirror is a projection of kit/guard/agent-on-git-guard:
    /// if the shim's order changes, this test goes red before doctor lies.
    #[test]
    fn shim_forwarding_order_matches_what_doctor_assumes() {
        let shim = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("kit/guard/agent-on-git-guard");
        let text = fs::read_to_string(&shim).unwrap();
        let plugin = text
            .find("\"${ROOT}/cli/target/release/agent-on\" guard")
            .expect("shim prefers the plugin-local binary");
        let repo = text
            .find("\"${REPO}/cli/target/release/agent-on\" guard")
            .expect("then the binary of the checkout the shim sits in");
        let path = text
            .find("exec agent-on guard")
            .expect("then agent-on on PATH");
        assert!(plugin < repo && repo < path, "{text}");
        assert!(text.contains("exit 0"), "no binary → fail-open: {text}");
    }

    fn repo_with_linked_worktree() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let main = tmp.path().join("main");
        fs::create_dir_all(&main).unwrap();
        git(&main, &["init", "-q", "-b", "main"]);
        git(&main, &["config", "user.email", "t@t.com"]);
        git(&main, &["config", "user.name", "t"]);
        write(&main.join("README.md"), "x\n");
        git(&main, &["add", "."]);
        git(&main, &["commit", "-qm", "init"]);
        let linked = tmp.path().join("linked");
        git(
            &main,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "lane",
                linked.to_str().unwrap(),
            ],
        );
        (tmp, main, linked)
    }

    #[test]
    fn main_tree_prints_no_worktree_line() {
        let (_tmp, main, _linked) = repo_with_linked_worktree();
        assert_eq!(worktree_line(&main, Some(&main)), None);
    }

    #[test]
    fn linked_worktree_of_write_root_gets_one_line_naming_the_main_tree() {
        let (_tmp, main, linked) = repo_with_linked_worktree();
        let line = worktree_line(&linked, Some(&main)).expect("linked worktree must be reported");
        assert!(!line.contains('\n'), "one line: {line}");
        assert!(line.contains("linked"), "{line}");
        let main_shown = fs::canonicalize(&main).unwrap().display().to_string();
        assert!(line.contains(&main_shown), "{line}");
        assert!(line.contains("第四检"), "{line}");
    }

    #[test]
    fn linked_worktree_of_another_repo_is_named_without_the_digest_hint() {
        let (_tmp, _main, linked) = repo_with_linked_worktree();
        let (_other_tmp, other_main, _) = repo_with_linked_worktree();
        let line = worktree_line(&linked, Some(&other_main)).expect("still a linked worktree");
        assert!(line.contains("linked"), "{line}");
        assert!(!line.contains("第四检"), "{line}");
    }

    #[test]
    fn outside_git_prints_no_worktree_line() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(worktree_line(tmp.path(), None), None);
    }
}
