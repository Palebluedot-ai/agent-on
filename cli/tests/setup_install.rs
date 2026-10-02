//! Controlled installation workflow: real Git, isolated HOME, fake host/cargo commands.
//! Does not install tools or change the real user's plugin configuration.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn script(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
#[test]
fn executor_precedes_plugins_and_any_required_failure_reports_partial() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    fs::create_dir(&source).unwrap();
    git(&source, &["init", "-b", "main"]);
    git(&source, &["config", "user.email", "test@example.com"]);
    git(&source, &["config", "user.name", "Test"]);
    fs::write(source.join("CHARTER.md"), "fixture").unwrap();
    fs::write(source.join("BOOTSTRAP.md"), "fixture").unwrap();
    fs::create_dir(source.join("cli")).unwrap();
    fs::write(source.join("cli/Cargo.toml"), "fixture").unwrap();
    git(&source, &["add", "."]);
    git(&source, &["commit", "-qm", "init"]);
    git(&source, &["tag", "v0.1.0"]);
    let bin = tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    script(&bin.join("cargo"),"#!/bin/sh\nprintf 'cargo %s\\n' \"$*\" >> \"$TEST_SETUP_LOG\"\nexit \"${TEST_CARGO_EXIT:-0}\"\n");
    script(&bin.join("codex"),"#!/bin/sh\nprintf 'codex %s\\n' \"$*\" >> \"$TEST_SETUP_LOG\"\nif [ \"$2\" = add ] && [ \"$3\" != --help ]; then exit \"${TEST_PLUGIN_EXIT:-0}\"; fi\nexit 0\n");
    script(
        &bin.join("claude"),
        "#!/bin/sh\nprintf 'claude %s\\n' \"$*\" >> \"$TEST_SETUP_LOG\"\nexit 0\n",
    );
    let log = tmp.path().join("steps.log");
    let work = tmp.path().join("work");
    let home = tmp.path().join("home");
    fs::create_dir(&home).unwrap();
    let run = |pin: &str, cargo: &str, plugin: &str| {
        Command::new(env!("CARGO_BIN_EXE_agent-on"))
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    bin.display(),
                    std::env::var("PATH").unwrap_or_default()
                ),
            )
            .env("TEST_SETUP_LOG", &log)
            .env("TEST_CARGO_EXIT", cargo)
            .env("TEST_PLUGIN_EXIT", plugin)
            .args(["setup", "--work-root"])
            .arg(&work)
            .args(["--remote"])
            .arg(&source)
            .args(["--pin", pin, "--with-plugins"])
            .output()
            .unwrap()
    };
    let good = run("v0.1.0", "0", "0");
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stderr)
    );
    let steps = fs::read_to_string(&log).unwrap();
    assert!(steps.find("cargo install").unwrap() < steps.find("plugin marketplace").unwrap());
    assert!(steps.contains("codex plugin add agent-on@agent-on"));
    fs::write(&log, "").unwrap();
    let fail = run("v0.1.0", "7", "0");
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("SETUP PARTIAL"));
    assert!(!fs::read_to_string(&log).unwrap().contains("plugin"));
    fs::write(&log, "").unwrap();
    let fail = run("v0.1.0", "0", "7");
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("Codex plugin 安装失败"));
    fs::write(&log, "").unwrap();
    let fail = run("v9.9.9", "0", "0");
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("未换成另一版本"));
    assert!(fs::read_to_string(&log).unwrap().is_empty());
}
