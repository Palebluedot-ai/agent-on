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

    // Build the executor before a marketplace copies the package into its cache.
    // A failed required step cannot be reported as a complete installation.
    if !opts.config_only && work_root.join("cli/Cargo.toml").is_file() {
        if let Err(error) = install_cli(&work_root) {
            eprintln!("SETUP PARTIAL: config 已登记；executor 未就绪：{error}");
            return 1;
        }
    }
    let mut failures = Vec::new();
    if opts.with_plugins {
        if let Err(error) = try_plugin_claude(&work_root) {
            failures.push(error);
        }
        if let Err(error) = try_plugin_codex(&work_root) {
            failures.push(error);
        }
    }
    if opts.with_symlinks {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        link_skill(&work_root, &home.join(".claude/skills/agent-on"));
        link_skill(&work_root, &home.join(".agents/skills/agent-on"));
    }

    println!();
    println!("--- doctor ---");
    print!("{}", doctor_report(None));

    run_intake_lint(&work_root);
    if !failures.is_empty() {
        eprintln!("SETUP PARTIAL: {}", failures.join("\n"));
        eprintln!("已成功的步骤保留；修复失败项后重跑 setup，当前结果不能当完整就绪。");
        return 1;
    }
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
        run_cmd(&["git", "fetch", "--tags", "origin"], Some(work_root), true)?;
    }

    let out = Command::new("git")
        .args(["checkout", "--detach", pin])
        .current_dir(work_root)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "checkout {pin} 失败，未换成另一版本：{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
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

fn install_cli(work_root: &Path) -> Result<(), String> {
    let cargo = which("cargo").ok_or("cargo 不在 PATH；请安装 Rust 1.89+ 后重跑 setup")?;
    run_cmd(
        &[
            cargo.to_str().ok_or("cargo 路径不是 UTF-8")?,
            "install",
            "--path",
            work_root
                .join("cli")
                .to_str()
                .ok_or("工作仓路径不是 UTF-8")?,
            "--force",
        ],
        None,
        true,
    )
}

fn try_plugin_claude(work_root: &Path) -> Result<(), String> {
    let Some(claude) = which("claude") else {
        println!("skip claude plugin: claude 不在 PATH");
        return Ok(());
    };
    let r1 = Command::new(&claude)
        .args(["plugin", "marketplace", "add", "Palebluedot-ai/agent-on"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if r1.map(|s| !s.success()).unwrap_or(true) {
        let fallback = Command::new(&claude)
            .args([
                "plugin",
                "marketplace",
                "add",
                &work_root.display().to_string(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !fallback.success() {
            return Err(
                "Claude marketplace 注册失败；重跑 claude plugin marketplace add <工作仓>".into(),
            );
        }
    }
    let installed = Command::new(&claude)
        .args(["plugin", "install", "agent-on@agent-on", "-s", "user"])
        .status()
        .map_err(|e| e.to_string())?;
    if !installed.success() {
        return Err(
            "Claude plugin 安装失败；重跑 claude plugin install agent-on@agent-on -s user".into(),
        );
    }
    println!("claude plugin: install 成功（宿主重启和信任仍需自检）");
    Ok(())
}

fn codex_install_command(codex: &Path) -> Result<&'static str, String> {
    for command in ["add", "install"] {
        if Command::new(codex)
            .args(["plugin", command, "--help"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
        {
            return Ok(command);
        }
    }
    Err("Codex 不支持 plugin add/install；升级宿主，或用 --with-symlinks 接入共享 skill".into())
}

fn try_plugin_codex(work_root: &Path) -> Result<(), String> {
    let Some(codex) = which("codex") else {
        println!("skip codex plugin: codex 不在 PATH");
        return Ok(());
    };
    install_codex_plugin(&codex, work_root)
}

fn install_codex_plugin(codex: &Path, work_root: &Path) -> Result<(), String> {
    let install = codex_install_command(codex)?;
    let r1 = Command::new(codex)
        .args(["plugin", "marketplace", "add", "Palebluedot-ai/agent-on"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if r1.map(|s| !s.success()).unwrap_or(true) {
        let fallback = Command::new(codex)
            .args([
                "plugin",
                "marketplace",
                "add",
                &work_root.display().to_string(),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !fallback.success() {
            return Err(
                "Codex marketplace 注册失败；重跑 codex plugin marketplace add <工作仓>".into(),
            );
        }
    }
    let installed = Command::new(codex)
        .args(["plugin", install, "agent-on@agent-on"])
        .status()
        .map_err(|e| e.to_string())?;
    if !installed.success() {
        return Err(format!(
            "Codex plugin 安装失败；重跑 codex plugin {install} agent-on@agent-on"
        ));
    }
    println!("codex plugin: {install} 成功（宿主重启和信任仍需自检）");
    Ok(())
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

    #[cfg(unix)]
    #[test]
    fn codex_add_is_preferred_and_failed_install_is_not_success() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempdir().unwrap();
        let executable = d.path().join("codex");
        fs::write(&executable,"#!/bin/sh\nif [ \"$3\" = --help ]; then [ \"$2\" = add ]; exit $?; fi\nif [ \"$2\" = marketplace ]; then exit 0; fi\nexit 7\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(codex_install_command(&executable).unwrap(), "add");
        assert!(install_codex_plugin(&executable, d.path())
            .unwrap_err()
            .contains("plugin add"));
        fs::write(
            &executable,
            "#!/bin/sh\nif [ \"$3\" = --help ]; then [ \"$2\" = install ]; exit $?; fi\nexit 0\n",
        )
        .unwrap();
        assert_eq!(codex_install_command(&executable).unwrap(), "install");
        assert!(install_codex_plugin(&executable, d.path()).is_ok());
        fs::write(&executable, "#!/bin/sh\nexit 2\n").unwrap();
        assert!(codex_install_command(&executable).is_err());
    }
}
