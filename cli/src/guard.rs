//! Cross-repo git guard (PreToolUse). Exit 0 allow, 2 block.

use crate::paths::resolve_work_root;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn write_ops() -> HashSet<&'static str> {
    [
        "add", "commit", "push", "reset", "revert", "rebase", "merge", "cherry-pick", "stash",
        "tag", "rm", "mv", "restore", "checkout", "switch", "clean", "am", "apply", "update-ref",
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

fn inside_agent_on(p: &Path, agent_on: &Path) -> bool {
    // macOS: /var vs /private/var — always canonicalize before prefix compare
    let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let a = std::fs::canonicalize(agent_on).unwrap_or_else(|_| agent_on.to_path_buf());
    let pl = p.to_string_lossy().to_lowercase();
    let al = a.to_string_lossy().to_lowercase();
    pl == al || pl.starts_with(&(al.clone() + std::path::MAIN_SEPARATOR_STR)) || pl.starts_with(&al)
}

/// Core decision given parsed tool payload JSON.
pub fn guard_decision(data: &Value) -> i32 {
    let ti = data.get("tool_input").cloned().unwrap_or(Value::Null);
    let cmd = ti
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or("");
    if !cmd.contains("git") {
        return 0;
    }

    let cwd_str = data
        .get("cwd")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".into())
        });
    let cwd = PathBuf::from(&cwd_str);

    let (wr, _) = resolve_work_root(Some(&cwd));
    let agent_on = match wr {
        Some(p) => p,
        None => return 0, // fail-open when B unset
    };

    let toks: Vec<String> = match shlex::split(cmd) {
        Some(t) => t,
        None => cmd.split_whitespace().map(|s| s.to_string()).collect(),
    };

    let write_ops = write_ops();
    let flags_take = flags_take_arg();
    let mut write_hit = false;
    let mut targets: Vec<PathBuf> = Vec::new();
    let mut cd_base = cwd.clone();
    let mut i = 0usize;
    while i < toks.len() {
        let t = &toks[i];
        if t == "cd" && i + 1 < toks.len() {
            cd_base = norm(&toks[i + 1], &cd_base);
            i += 2;
            continue;
        }
        if t == "git" {
            let mut git_dir = cd_base.clone();
            let mut j = i + 1;
            while j < toks.len() {
                let tj = &toks[j];
                if tj == "-C" && j + 1 < toks.len() {
                    git_dir = norm(&toks[j + 1], &cd_base);
                    j += 2;
                    continue;
                }
                if tj.starts_with('-') {
                    j += 1;
                    continue;
                }
                if write_ops.contains(tj.as_str()) {
                    if tj == "tag" {
                        let nxt = toks.get(j + 1).map(|s| s.as_str());
                        let read_tag_flags = [
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
                        let is_sep = matches!(nxt, None | Some("&&") | Some("||") | Some(";") | Some("|"));
                        let is_read = nxt
                            .map(|n| {
                                let head = n.split('=').next().unwrap_or(n);
                                read_tag_flags.contains(&head)
                            })
                            .unwrap_or(false);
                        if is_sep || is_read {
                            break;
                        }
                    }
                    write_hit = true;
                    targets.push(git_dir.clone());
                    let mut k = j + 1;
                    while k < toks.len() {
                        let tk = &toks[k];
                        if matches!(tk.as_str(), "&&" | "||" | ";" | "|") {
                            break;
                        }
                        if tk.starts_with('-') {
                            let flag = tk.split('=').next().unwrap_or(tk);
                            if flags_take.contains(flag) && !tk.contains('=') && k + 1 < toks.len()
                            {
                                k += 2;
                                continue;
                            }
                            k += 1;
                            continue;
                        }
                        targets.push(norm(tk, &git_dir));
                        k += 1;
                    }
                }
                break;
            }
            i = j + 1;
            continue;
        }
        i += 1;
    }

    if !write_hit {
        return 0;
    }
    if !targets.iter().any(|t| inside_agent_on(t, &agent_on)) {
        return 0;
    }

    let sess = env::var("CLAUDE_PROJECT_DIR")
        .or_else(|_| env::var("CODEX_PROJECT_DIR"))
        .unwrap_or(cwd_str);
    let sess_path = norm(&sess, Path::new("."));
    if inside_agent_on(&sess_path, &agent_on) {
        return 0;
    }

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
    2
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    // env-based tests must not run in parallel
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_b_env<F: FnOnce(PathBuf) -> ()>(f: F) {
        let _g = ENV_LOCK.lock().unwrap();
        let d = tempdir().unwrap();
        fs::write(d.path().join("CHARTER.md"), "x").unwrap();
        fs::write(d.path().join("BOOTSTRAP.md"), "x").unwrap();
        let p = d.path().to_path_buf();
        env::set_var("AGENT_ON_ROOT", &p);
        f(p);
        env::remove_var("AGENT_ON_ROOT");
        env::remove_var("CLAUDE_PROJECT_DIR");
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
    fn fail_open_without_b() {
        let _g = ENV_LOCK.lock().unwrap();
        env::remove_var("AGENT_ON_ROOT");
        let data = json!({
            "tool_input": {"command": "git -C /somewhere/agent-on commit -m x"},
            "cwd": "/tmp"
        });
        // may still resolve default_work_root if real machine has it — only assert non-panic
        let _ = guard_decision(&data);
    }
}
