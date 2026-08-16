//! One-shot install: clone/update B, write config, optional plugins/symlinks.

use crate::intake_lint::{default_intake_paths, lint_paths};
use crate::paths::{
    config_path, default_work_root, doctor_report, looks_like_agent_on, write_config_work_root,
    write_config_work_root_to, DEFAULT_PIN, OFFICIAL_HTTPS,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn which(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|p| {
            let cand = p.join(name);
            if cand.is_file() {
                Some(cand)
            } else {
                // Windows .exe
                let cand_exe = p.join(format!("{name}.exe"));
                if cand_exe.is_file() {
                    Some(cand_exe)
                } else {
                    None
                }
            }
        })
    })
}

fn run_cmd(cmd: &[&str], cwd: Option<&Path>, check: bool) -> Result<(), String> {
    eprintln!("+ {}", cmd.join(" "));
    let mut c = Command::new(cmd[0]);
    c.args(&cmd[1..]);
    if let Some(cwd) = cwd {
        c.current_dir(cwd);
    }
    let st = c.status().map_err(|e| e.to_string())?;
    if check && !st.success() {
        return Err(format!("command failed: {}", cmd.join(" ")));
    }
    Ok(())
}

pub struct SetupOpts {
    pub work_root: Option<PathBuf>,
    pub pin: String,
    pub remote: String,
    pub with_plugins: bool,
    pub with_symlinks: bool,
    pub config_only: bool,
    /// Override config.json path (tests); default `~/.config/agent-on/config.json`
    pub config_path_override: Option<PathBuf>,
}

pub fn run_setup(opts: &SetupOpts) -> i32 {
    if which("git").is_none() {
        eprintln!("ERROR: 需要 git 在 PATH 中。");
        return 2;
    }

    let work_root = opts.work_root.clone().unwrap_or_else(default_work_root);
    let work_root = fs::canonicalize(&work_root).unwrap_or(work_root);

    println!("platform     = {}", env::consts::OS);
    println!("work_root    = {}", work_root.display());
    println!("pin          = {}", opts.pin);
    println!("remote       = {}", opts.remote);

    if opts.config_only {
        if !looks_like_agent_on(&work_root) {
            eprintln!(
                "ERROR: --config-only 但 {} 不是合法 agent-on 仓",
                work_root.display()
            );
            return 1;
        }
    } else if let Err(e) = clone_or_update(&work_root, &opts.pin, &opts.remote) {
        eprintln!("ERROR: {e}");
        return 1;
    }

    let write_cfg = |wr: &Path| -> std::io::Result<PathBuf> {
        if let Some(ref c) = opts.config_path_override {
            write_config_work_root_to(c, wr)
        } else {
            write_config_work_root(wr)
        }
    };
    match write_cfg(&work_root) {
        Ok(cfg) => println!("wrote config = {}", cfg.display()),
        Err(e) => {
            eprintln!("ERROR writing config: {e}");
            return 1;
        }
    }

    if opts.with_plugins {
        try_plugin_claude(&work_root);
        try_plugin_codex(&work_root);
    }
    if opts.with_symlinks {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        link_skill(&work_root, &home.join(".claude/skills/agent-on"));
        link_skill(&work_root, &home.join(".agents/skills/agent-on"));
    }

    // Install this CLI into cargo bin if possible (best-effort)
    if let Some(cargo) = which("cargo") {
        let manifest = work_root.join("cli/Cargo.toml");
        if manifest.is_file() {
            let _ = Command::new(cargo)
                .args([
                    "install",
                    "--path",
                    work_root.join("cli").to_str().unwrap_or("cli"),
                    "--force",
                ])
                .status();
        }
    }

    println!();
    println!("--- doctor ---");
    print!("{}", doctor_report(None));

    run_intake_lint(&work_root);
    print_next_steps(&work_root);
    0
}

fn clone_or_update(work_root: &Path, pin: &str, remote: &str) -> Result<(), String> {
    let git_dir = work_root.join(".git");
    if !git_dir.exists() {
        if let Some(parent) = work_root.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if work_root.exists()
            && fs::read_dir(work_root)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
        {
            return Err(format!(
                "{} 非空且不是 git 仓。换 --work-root 或清空后重试。",
                work_root.display()
            ));
        }
        run_cmd(
            &["git", "clone", remote, &work_root.display().to_string()],
            None,
            true,
        )?;
    } else {
        let _ = run_cmd(
            &["git", "fetch", "--tags", "--force", "origin"],
            Some(work_root),
            false,
        );
    }

    let out = Command::new("git")
        .args(["checkout", "--detach", pin])
        .current_dir(work_root)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        eprintln!("WARN: checkout {pin} 失败，尝试 origin/main …");
        let _ = run_cmd(&["git", "checkout", "main"], Some(work_root), false);
        let _ = run_cmd(
            &["git", "pull", "--ff-only", "origin", "main"],
            Some(work_root),
            false,
        );
    } else {
        println!("checked out {pin}");
    }

    if !looks_like_agent_on(work_root) {
        return Err(format!(
            "{} 不像 agent-on 仓（缺 CHARTER/BOOTSTRAP）。",
            work_root.display()
        ));
    }
    Ok(())
}

fn try_plugin_claude(work_root: &Path) {
    let Some(claude) = which("claude") else {
        println!("skip claude plugin: claude 不在 PATH");
        return;
    };
    let r1 = Command::new(&claude)
        .args(["plugin", "marketplace", "add", "Palebluedot-ai/agent-on"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if r1.map(|s| !s.success()).unwrap_or(true) {
        let _ = Command::new(&claude)
            .args([
                "plugin",
                "marketplace",
                "add",
                &work_root.display().to_string(),
            ])
            .status();
    }
    let _ = Command::new(&claude)
        .args(["plugin", "install", "agent-on@agent-on", "-s", "user"])
        .status();
    println!("claude plugin: 已尝试 install agent-on@agent-on（失败可手动重跑）");
}

fn try_plugin_codex(work_root: &Path) {
    let Some(codex) = which("codex") else {
        println!("skip codex plugin: codex 不在 PATH");
        return;
    };
    let r1 = Command::new(&codex)
        .args(["plugin", "marketplace", "add", "Palebluedot-ai/agent-on"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if r1.map(|s| !s.success()).unwrap_or(true) {
        let _ = Command::new(&codex)
            .args([
                "plugin",
                "marketplace",
                "add",
                &work_root.display().to_string(),
            ])
            .status();
    }
    let _ = Command::new(&codex)
        .args(["plugin", "install", "agent-on@agent-on"])
        .status();
    println!("codex plugin: 已尝试 install agent-on@agent-on");
}

fn link_skill(work_root: &Path, dest: &Path) {
    let src = work_root.join("skill");
    if !src.is_dir() {
        println!("skip symlink: 无 {}", src.display());
        return;
    }
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if dest.exists() || dest.is_symlink() {
        if dest.is_symlink() {
            if let (Ok(a), Ok(b)) = (fs::canonicalize(dest), fs::canonicalize(&src)) {
                if a == b {
                    println!("symlink ok: {}", dest.display());
                    return;
                }
            }
        }
        println!("skip symlink: 已存在 {}（不覆盖）", dest.display());
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if symlink(&src, dest).is_ok() {
            println!("symlink: {} -> {}", dest.display(), src.display());
        }
    }
    #[cfg(not(unix))]
    {
        println!("skip symlink on non-unix: {}", dest.display());
    }
}

fn run_intake_lint(work_root: &Path) {
    let cards = default_intake_paths(work_root);
    if cards.is_empty() {
        println!("intake-lint: 无卡文件，跳过");
        return;
    }
    let (code, out) = lint_paths(&cards);
    print!("{out}");
    if code == 0 {
        println!("intake-lint: 通过");
    } else {
        println!("intake-lint: 有问题（exit {code}）——贡献前请修好");
    }
}

fn print_next_steps(work_root: &Path) {
    println!();
    println!("{}", "=".repeat(60));
    println!("agent-on setup 完成");
    println!("  work_root (B) = {}", work_root.display());
    println!("  config        = {}", config_path().display());
    println!();
    println!("开工：");
    println!("  Claude Code  →  /agent-on init   或「初始化本项目」");
    println!("  Codex        →  $agent-on init  或「初始化本项目」");
    println!("  Grok         →  「初始化本项目」（全局 AGENT.md 需有 Agent-On 路由）");
    println!();
    println!("自检：");
    println!("  agent-on doctor");
    println!("  agent-on intake-lint");
    println!();
    println!("文档：README「给朋友的 5 分钟装机」");
    println!("{}", "=".repeat(60));
}

pub fn default_pin() -> &'static str {
    DEFAULT_PIN
}

pub fn default_remote() -> &'static str {
    OFFICIAL_HTTPS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn markers(dir: &Path) {
        fs::write(dir.join("CHARTER.md"), "x").unwrap();
        fs::write(dir.join("BOOTSTRAP.md"), "x").unwrap();
    }

    #[test]
    fn config_only_rejects_invalid_tree() {
        let d = tempdir().unwrap();
        // no markers
        let code = run_setup(&SetupOpts {
            work_root: Some(d.path().to_path_buf()),
            pin: DEFAULT_PIN.into(),
            remote: OFFICIAL_HTTPS.into(),
            with_plugins: false,
            with_symlinks: false,
            config_only: true,
            config_path_override: Some(d.path().join("cfg.json")),
        });
        assert_eq!(code, 1);
        assert!(!d.path().join("cfg.json").exists());
    }

    #[test]
    fn config_only_writes_config_for_valid_tree() {
        let d = tempdir().unwrap();
        markers(d.path());
        let cfg = d.path().join("nested").join("config.json");
        let code = run_setup(&SetupOpts {
            work_root: Some(d.path().to_path_buf()),
            pin: DEFAULT_PIN.into(),
            remote: OFFICIAL_HTTPS.into(),
            with_plugins: false,
            with_symlinks: false,
            config_only: true,
            config_path_override: Some(cfg.clone()),
        });
        assert_eq!(code, 0, "config-only on valid tree must succeed");
        assert!(cfg.is_file());
        let text = fs::read_to_string(&cfg).unwrap();
        assert!(text.contains("work_root"), "{text}");
        // work_root value is absolute path to d
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let wr = v["work_root"].as_str().unwrap();
        let got = PathBuf::from(wr);
        let expect = fs::canonicalize(d.path()).unwrap();
        assert_eq!(got, expect);
    }
}
