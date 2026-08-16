//! Git boundary guard for Claude/Codex PreToolUse. Exit 0 allow, 2 block.

use crate::paths::resolve_work_root;
use crate::worktree;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Default)]
struct ParsedGitCommand {
    write_hit: bool,
    targets: Vec<PathBuf>,
    commit_dirs: BTreeSet<PathBuf>,
    commit_push_dirs: BTreeSet<PathBuf>,
}

fn write_ops() -> HashSet<&'static str> {
    [
        "add",
        "commit",
        "push",
        "reset",
        "revert",
        "rebase",
        "merge",
        "cherry-pick",
        "stash",
        "tag",
        "rm",
        "mv",
        "restore",
        "checkout",
        "switch",
        "clean",
        "am",
        "apply",
        "update-ref",
    ]
    .into_iter()
    .collect()
}

fn flags_take_arg() -> HashSet<&'static str> {
    [
        "-m",
        "--message",
        "-F",
        "--file",
        "--author",
        "--date",
        "--fixup",
        "--onto",
        "--strategy",
        "-s",
        "-X",
        "--reuse-message",
        "--reedit-message",
        "-C",
        "--template",
        "-t",
        "--exec",
    ]
    .into_iter()
    .collect()
}

fn norm(p: &str, base: &Path) -> PathBuf {
    let expanded = if p.starts_with("~/") || p == "~" {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        if p == "~" {
            home
        } else {
            home.join(&p[2..])
        }
    } else {
        PathBuf::from(p)
    };
    let joined = if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    };
    std::fs::canonicalize(&joined).unwrap_or(joined)
}

fn is_git_token(token: &str) -> bool {
    Path::new(token)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("git") || name.eq_ignore_ascii_case("git.exe"))
        .unwrap_or(false)
}

fn is_separator(token: &str) -> bool {
    matches!(token, "&&" | "||" | ";" | "|")
}

fn tool_command(data: &Value) -> &str {
    let input = data.get("tool_input").unwrap_or(&Value::Null);
    input
        .get("command")
        .or_else(|| input.get("cmd"))
        .or_else(|| data.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn tool_cwd(data: &Value) -> PathBuf {
    let input = data.get("tool_input").unwrap_or(&Value::Null);
    let raw = data
        .get("cwd")
        .or_else(|| input.get("workdir"))
        .or_else(|| input.get("cwd"))
        .and_then(Value::as_str)
        .unwrap_or(".");
    let process_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    norm(raw, &process_cwd)
}

fn parse_git_command(cmd: &str, cwd: &Path) -> ParsedGitCommand {
    let toks: Vec<String> = match shlex::split(cmd) {
        Some(tokens) => tokens,
        None => cmd.split_whitespace().map(str::to_string).collect(),
    };
    let write_ops = write_ops();
    let flags_take = flags_take_arg();
    let mut parsed = ParsedGitCommand::default();
    let mut cd_base = cwd.to_path_buf();
    let mut i = 0usize;

    while i < toks.len() {
        let token = &toks[i];
        if token == "cd" && i + 1 < toks.len() {
            cd_base = norm(&toks[i + 1], &cd_base);
            i += 2;
            continue;
        }
        if !is_git_token(token) {
            i += 1;
            continue;
        }

        let mut git_dir = cd_base.clone();
        let mut global_targets = Vec::new();
        let mut j = i + 1;
        while j < toks.len() && !is_separator(&toks[j]) {
            let arg = &toks[j];
            if arg == "-C" && j + 1 < toks.len() {
                git_dir = norm(&toks[j + 1], &cd_base);
                global_targets.push(git_dir.clone());
                j += 2;
                continue;
            }
            if let Some(path) = arg.strip_prefix("-C") {
                if !path.is_empty() {
                    git_dir = norm(path, &cd_base);
                    global_targets.push(git_dir.clone());
                    j += 1;
                    continue;
                }
            }
            if matches!(arg.as_str(), "-c" | "--config-env" | "--namespace") && j + 1 < toks.len() {
                j += 2;
                continue;
            }
            if matches!(arg.as_str(), "--git-dir" | "--work-tree") && j + 1 < toks.len() {
                let path = norm(&toks[j + 1], &cd_base);
                global_targets.push(path.clone());
                if arg == "--work-tree" {
                    git_dir = path;
                } else if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                    if let Some(parent) = path.parent() {
                        git_dir = parent.to_path_buf();
                    }
                }
                j += 2;
                continue;
            }
            if let Some(path) = arg.strip_prefix("--work-tree=") {
                let path = norm(path, &cd_base);
                global_targets.push(path.clone());
                git_dir = path;
                j += 1;
                continue;
            }
            if let Some(path) = arg.strip_prefix("--git-dir=") {
                let path = norm(path, &cd_base);
                global_targets.push(path.clone());
                if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                    if let Some(parent) = path.parent() {
                        git_dir = parent.to_path_buf();
                    }
                }
                j += 1;
                continue;
            }
            if arg.starts_with('-') {
                j += 1;
                continue;
            }

            let op = arg.as_str();
            if !write_ops.contains(op) {
                break;
            }

            let mut end = j + 1;
            while end < toks.len() && !is_separator(&toks[end]) {
                end += 1;
            }
            let help_only = toks[j + 1..end]
                .iter()
                .any(|value| matches!(value.as_str(), "-h" | "--help"));
            let read_only_tag = if op == "tag" {
                let first = toks.get(j + 1).map(String::as_str);
                let read_flags = [
                    "-l",
                    "--list",
                    "-n",
                    "--contains",
                    "--points-at",
                    "--sort",
                    "--format",
                    "--merged",
                    "--no-merged",
                    "--column",
                ];
                first.is_none_or(|value| {
                    is_separator(value)
                        || read_flags.contains(&value.split('=').next().unwrap_or(value))
                })
            } else {
                false
            };
            if !help_only && !read_only_tag {
                parsed.write_hit = true;
                parsed.targets.push(git_dir.clone());
                parsed.targets.extend(global_targets.iter().cloned());
                if matches!(op, "commit" | "push") {
                    parsed.commit_push_dirs.insert(git_dir.clone());
                }
                if op == "commit" {
                    parsed.commit_dirs.insert(git_dir.clone());
                }

                let mut k = j + 1;
                while k < end {
                    let value = &toks[k];
                    if value.starts_with('-') {
                        let flag = value.split('=').next().unwrap_or(value);
                        if flags_take.contains(flag) && !value.contains('=') && k + 1 < end {
                            k += 2;
                            continue;
                        }
                        k += 1;
                        continue;
                    }
                    parsed.targets.push(norm(value, &git_dir));
                    k += 1;
                }
            }
            j = end;
            break;
        }
        i = j.saturating_add(1);
    }

    parsed
}

fn existing_repo_root(cwd: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    if !output.status.success() {
        // The requested git commit/push will fail on its own. There is no
        // worktree mutation for this guard to protect, so avoid replacing the
        // native git diagnostic with a misleading lane error.
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!raw.is_empty()).then(|| {
        let root = PathBuf::from(raw);
        std::fs::canonicalize(&root).unwrap_or(root)
    })
}

/// True iff `p` is the agent-on root or a path *inside* it (component boundary).
/// Rejects sibling prefixes like `/tmp/B` vs `/tmp/B-evil`.
pub(crate) fn inside_agent_on(p: &Path, agent_on: &Path) -> bool {
    // macOS: /var vs /private/var — canonicalize when possible
    let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let a = std::fs::canonicalize(agent_on).unwrap_or_else(|_| agent_on.to_path_buf());
    let pl = p.to_string_lossy().to_lowercase();
    let al = a.to_string_lossy().to_lowercase();
    if pl == al {
        return true;
    }
    // Require path separator after root — bare starts_with would match B-evil for B
    let prefix = format!("{}{}", al, std::path::MAIN_SEPARATOR);
    pl.starts_with(&prefix)
}

/// Core decision given parsed tool payload JSON.
pub fn guard_decision(data: &Value) -> i32 {
    let cmd = tool_command(data);
    if !cmd.contains("git") {
        return 0;
    }

    let cwd = tool_cwd(data);
    let parsed = parse_git_command(cmd, &cwd);
    if !parsed.write_hit {
        return 0;
    }

    // Preserve the original cross-repo boundary. This lookup is deliberately
    // after parsing so non-git/read-only commands do no filesystem work.
    if let (Some(agent_on), _) = resolve_work_root(Some(&cwd)) {
        if parsed
            .targets
            .iter()
            .any(|target| inside_agent_on(target, &agent_on))
        {
            let sess = env::var("CLAUDE_PROJECT_DIR")
                .or_else(|_| env::var("CODEX_PROJECT_DIR"))
                .unwrap_or_else(|_| cwd.display().to_string());
            let sess_path = norm(&sess, Path::new("."));
            if !inside_agent_on(&sess_path, &agent_on) {
                eprintln!(
                    "⛔ 跨仓 git 边界拦截(agent-on-git-guard):项目端会话对 agent-on 工作仓只写 intake/ 素材文件,\
不 add / 不 commit / 不 push(AGENT.md 硬规矩 2026-07-13)。\n\
工作仓(B) {}\n\
会话根 {} 不在 B 内,却试图对 B 执行 git 写操作。\n\
正确动作:落盘 intake 文件即止;git 收件/消化由用户切到 agent-on 工作仓会话执行。\n\
被拦命令:{cmd}\n",
                    agent_on.display(),
                    sess
                );
                return 2;
            }
        }
    }

    // Commit/push are the only PreToolUse points that pay for a full lane
    // audit. Other git writes keep the existing cross-repo check only.
    let mut audit_repos = BTreeSet::new();
    let mut commit_repos = BTreeSet::new();
    for dir in &parsed.commit_push_dirs {
        if let Some(root) = existing_repo_root(dir) {
            audit_repos.insert(root.clone());
            if parsed.commit_dirs.contains(dir) {
                commit_repos.insert(root);
            }
        }
    }

    for repo in &commit_repos {
        let (code, reason) = crate::worktree_hooks::primary_control_guard(repo);
        if code != 0 {
            eprintln!(
                "⛔ 主树控制轨检查未通过，已拦截 git commit。\n\
{}\n\
下一步:把业务改动移到已 claim 的 worktree；合流操作会由 Agent-On 自动识别并放行。\n\
自查:`agent-on worktree status --repo {}`\n\
被拦命令: {cmd}\n",
                reason.trim_end(),
                repo.display()
            );
            return 2;
        }
    }

    for repo in &audit_repos {
        let (code, report) = worktree::run_audit(repo, false, true);
        if code != 0 {
            eprintln!(
                "⛔ Worktree 边界检查未通过，已拦截 git commit/push。\n\
检查目标: {}\n\
{}\n\
下一步:\n\
  1. 运行 `agent-on worktree status --repo {}` 查看 lane / owns。\n\
  2. 把 OUT-OF-BOUNDS 文件移回所属 lane，或由控制轨重新划分 owns。\n\
  3. 若检查器本身报错，先修复报错；不要用跳过 hook 掩盖 unknown。\n\
被拦命令: {cmd}\n",
                repo.display(),
                report.trim_end(),
                repo.display()
            );
            return 2;
        }
    }

    0
}

pub fn run_from_stdin() -> i32 {
    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        return 0;
    }
    let data: Value = match serde_json::from_str(if buf.trim().is_empty() { "{}" } else { &buf }) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    guard_decision(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::process::{Command, Stdio};
    use tempfile::{tempdir, TempDir};

    // env-based tests must not run in parallel
    fn with_b_env<F: FnOnce(PathBuf)>(f: F) {
        let _g = crate::TEST_ENV_LOCK.lock().unwrap();
        let d = tempdir().unwrap();
        fs::write(d.path().join("CHARTER.md"), "x").unwrap();
        fs::write(d.path().join("BOOTSTRAP.md"), "x").unwrap();
        let p = d.path().to_path_buf();
        env::set_var("AGENT_ON_ROOT", &p);
        f(p);
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CLAUDE_PROJECT_DIR");
        env::remove_var("CODEX_PROJECT_DIR");
    }

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

    fn lane_fixture() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        fs::create_dir_all(root.join("app")).unwrap();
        run(&root, &["git", "init", "-b", "main"]);
        run(&root, &["git", "config", "user.email", "test@example.com"]);
        run(&root, &["git", "config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "root\n").unwrap();
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
        let (code, out) = crate::worktree::claim_lane(
            &wt,
            &crate::worktree::ClaimOpts {
                parked: false,
                id: "lane-a".to_string(),
                goal: "change app".to_string(),
                base: Some("main".to_string()),
                owns: vec!["app".to_string()],
                depends_on: Vec::new(),
            },
        );
        assert_eq!(code, 0, "{out}");
        (tmp, root, wt)
    }

    #[test]
    fn blocks_cross_repo_commit() {
        with_b_env(|b| {
            env::set_var("CLAUDE_PROJECT_DIR", "/tmp");
            let data = json!({
                "tool_input": {"command": format!("git -C {} commit -m x", b.display())},
                "cwd": "/tmp"
            });
            assert_eq!(guard_decision(&data), 2);
        });
    }

    #[test]
    fn allows_self_session() {
        with_b_env(|b| {
            env::set_var("CLAUDE_PROJECT_DIR", &b);
            let data = json!({
                "tool_input": {"command": "git commit -m x"},
                "cwd": b
            });
            assert_eq!(guard_decision(&data), 0);
        });
    }

    #[test]
    fn allows_status_read() {
        with_b_env(|b| {
            env::set_var("CLAUDE_PROJECT_DIR", "/tmp");
            let data = json!({
                "tool_input": {"command": format!("git -C {} status", b.display())},
                "cwd": "/tmp"
            });
            assert_eq!(guard_decision(&data), 0);
        });
    }

    #[test]
    fn parser_keeps_read_only_git_paths_free() {
        let cwd = Path::new("/tmp");
        for command in [
            "git status",
            "git tag",
            "git tag --list 'v*'",
            "git commit --help",
            "/usr/bin/git push --help",
        ] {
            let parsed = parse_git_command(command, cwd);
            assert!(!parsed.write_hit, "{command} must remain read-only");
            assert!(parsed.commit_push_dirs.is_empty(), "{command}");
        }
    }

    #[test]
    fn parses_codex_cmd_and_workdir_shape() {
        let dir = tempdir().unwrap();
        let data = json!({
            "tool_name": "exec_command",
            "tool_input": {
                "cmd": "git push",
                "workdir": dir.path()
            }
        });
        assert_eq!(tool_command(&data), "git push");
        assert_eq!(tool_cwd(&data), fs::canonicalize(dir.path()).unwrap());
        let parsed = parse_git_command(tool_command(&data), &tool_cwd(&data));
        assert_eq!(parsed.commit_push_dirs.len(), 1);
    }

    #[test]
    fn claude_and_codex_share_one_plugin_hook_manifest() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let codex: Value = serde_json::from_str(
            &fs::read_to_string(repo.join(".codex-plugin/plugin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            codex.get("hooks").and_then(Value::as_str),
            Some("./hooks/hooks.json")
        );
        let hooks = fs::read_to_string(repo.join("hooks/hooks.json")).unwrap();
        assert!(hooks.contains("agent-on-git-guard"), "{hooks}");
        assert!(hooks.contains("CLAUDE_PLUGIN_ROOT"), "{hooks}");
        assert!(
            !repo.join("hooks/hooks-codex.json").exists(),
            "Codex must not fork a second hook policy"
        );
    }

    #[test]
    fn legacy_wrapper_accepts_shell_and_python_interpreters() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let wrapper = repo.join("kit/guard/agent-on-git-guard.sh");
        for interpreter in ["sh", "bash", "python3"] {
            if Command::new(interpreter).arg("--version").output().is_err() {
                continue;
            }
            let output = Command::new(interpreter)
                .arg(&wrapper)
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{interpreter} compatibility failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                !String::from_utf8_lossy(&output.stderr).contains("SyntaxError"),
                "{interpreter} must not parse the compatibility wrapper as the wrong language"
            );
        }
    }

    #[test]
    fn blocks_codex_commit_when_lane_owns_is_violated() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let (_tmp, _root, wt) = lane_fixture();
        fs::write(wt.join("README.md"), "escaped\n").unwrap();
        env::set_var("AGENT_ON_ROOT", "/nonexistent/agent-on-guard-test");
        env::set_var("CODEX_PROJECT_DIR", &wt);
        env::remove_var("CLAUDE_PROJECT_DIR");
        let data = json!({
            "tool_name": "exec_command",
            "tool_input": {
                "cmd": "git commit -m escaped",
                "workdir": wt
            }
        });
        assert_eq!(guard_decision(&data), 2);
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CODEX_PROJECT_DIR");
    }

    #[test]
    fn allows_claude_commit_when_change_stays_inside_lane_owns() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let (_tmp, _root, wt) = lane_fixture();
        fs::write(wt.join("app/inside.txt"), "allowed\n").unwrap();
        env::set_var("AGENT_ON_ROOT", "/nonexistent/agent-on-guard-test");
        env::set_var("CLAUDE_PROJECT_DIR", &wt);
        env::remove_var("CODEX_PROJECT_DIR");
        let data = json!({
            "tool_name": "Bash",
            "tool_input": {"command": "git commit -m allowed"},
            "cwd": wt
        });
        assert_eq!(guard_decision(&data), 0);
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CLAUDE_PROJECT_DIR");
    }

    #[test]
    fn fail_open_without_b() {
        let _g = crate::TEST_ENV_LOCK.lock().unwrap();
        // Force resolve_work_root → None without relying on host default_B/config
        env::set_var("AGENT_ON_ROOT", "/nonexistent/agent-on-xyz-no-such");
        env::remove_var("CLAUDE_PROJECT_DIR");
        let data = json!({
            "tool_input": {"command": "git -C /somewhere/agent-on commit -m x"},
            "cwd": "/tmp"
        });
        assert_eq!(
            guard_decision(&data),
            0,
            "B unregistered/invalid must fail-open (allow)"
        );
        env::remove_var("AGENT_ON_ROOT");
    }

    #[test]
    fn sibling_prefix_not_inside_b() {
        let base = tempdir().unwrap();
        let b = base.path().join("B");
        let evil = base.path().join("B-evil");
        fs::create_dir_all(&b).unwrap();
        fs::create_dir_all(&evil).unwrap();
        fs::write(b.join("CHARTER.md"), "x").unwrap();
        fs::write(b.join("BOOTSTRAP.md"), "x").unwrap();
        // no need markers on evil
        assert!(
            !inside_agent_on(&evil, &b),
            "B-evil must not count as inside B"
        );
        assert!(inside_agent_on(&b, &b));
        let nested = b.join("intake");
        fs::create_dir_all(&nested).unwrap();
        assert!(inside_agent_on(&nested, &b));
    }

    #[test]
    fn sibling_session_does_not_bypass_guard() {
        // CLAUDE_PROJECT_DIR=B-evil must NOT allow git -C B write
        let base = tempdir().unwrap();
        let b = base.path().join("B");
        let evil = base.path().join("B-evil");
        fs::create_dir_all(&b).unwrap();
        fs::create_dir_all(&evil).unwrap();
        fs::write(b.join("CHARTER.md"), "x").unwrap();
        fs::write(b.join("BOOTSTRAP.md"), "x").unwrap();

        let _g = crate::TEST_ENV_LOCK.lock().unwrap();
        env::set_var("AGENT_ON_ROOT", &b);
        env::set_var("CLAUDE_PROJECT_DIR", &evil);
        let data = json!({
            "tool_input": {"command": format!("git -C {} commit -m x", b.display())},
            "cwd": evil
        });
        assert_eq!(
            guard_decision(&data),
            2,
            "session under B-evil must not authorize writes into B"
        );
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CLAUDE_PROJECT_DIR");
    }

    #[test]
    fn write_to_sibling_not_blocked_as_b() {
        // git -C B-evil from outside: target is not B → allow (not our boundary)
        let base = tempdir().unwrap();
        let b = base.path().join("B");
        let evil = base.path().join("B-evil");
        fs::create_dir_all(&b).unwrap();
        fs::create_dir_all(&evil).unwrap();
        fs::write(b.join("CHARTER.md"), "x").unwrap();
        fs::write(b.join("BOOTSTRAP.md"), "x").unwrap();

        let _g = crate::TEST_ENV_LOCK.lock().unwrap();
        env::set_var("AGENT_ON_ROOT", &b);
        env::set_var("CLAUDE_PROJECT_DIR", "/tmp");
        let data = json!({
            "tool_input": {"command": format!("git -C {} commit -m x", evil.display())},
            "cwd": "/tmp"
        });
        assert_eq!(
            guard_decision(&data),
            0,
            "writes to B-evil are not B — guard should not false-block"
        );
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CLAUDE_PROJECT_DIR");
    }
}
