//! An on-call registration is only as alive as the window behind it. The
//! window proves it is there by heartbeat: every guarded tool call from the
//! on-call worktree refreshes it, and `agent-on oncall heartbeat` refreshes it
//! explicitly. Past the configured silence the registration expires and the
//! routing gate fails open — a closed window must never keep the whole repo's
//! merges locked.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn must_run(cwd: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Identity and host markers a live Claude/Codex session exports. Inherited by
/// `cargo test` run inside one, they would outrank the per-window identity.
const HOST_SESSION_ENV: &[&str] = &[
    "CODEX_THREAD_ID",
    "AGENT_ON_HOST",
    "AGENT_ON_SESSION_ID",
    "AGENT_ON_PARENT_CODEX_THREAD_ID",
    "CLAUDE_ENV_FILE",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_HOST_SESSION_ID",
    "CLAUDE_CODE_CHILD_SESSION",
    "PLUGIN_ROOT",
    "CLAUDE_PLUGIN_ROOT",
    "AGENT_ON_CONTROL_ID",
    "AGENT_ON_CONTROL_REPO",
    "CLAUDE_PROJECT_DIR",
    "CODEX_PROJECT_DIR",
];

/// The real binary as the window at `cwd`, never as the session running tests.
fn window(cwd: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
    cmd.current_dir(cwd);
    for key in HOST_SESSION_ENV {
        cmd.env_remove(key);
    }
    cmd.env("AGENT_ON_HOST", "codex")
        .env("AGENT_ON_SESSION_ID", cwd)
        .env("AGENT_ON_ROOT", "/nonexistent/agent-on-liveness-test");
    cmd
}

fn agent_on(cwd: &Path, args: &[&str]) -> Output {
    window(cwd).args(args).output().unwrap()
}

fn guard(cwd: &Path, payload: &str) -> Output {
    let mut child = window(cwd)
        .arg("guard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn bash_payload(cwd: &Path, cmd: &str) -> String {
    format!(
        r#"{{"tool_name":"Bash","cwd":"{}","tool_input":{{"command":"{cmd}"}}}}"#,
        cwd.display()
    )
}

fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "t@example.com"]);
    must_run(&root, "git", &["config", "user.name", "T"]);
    fs::write(root.join("README.md"), "x\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let feature = tmp.path().join("feature");
    must_run(
        &root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "feature",
            feature.to_str().unwrap(),
            "main",
        ],
    );
    (tmp, root, feature)
}

fn oncall_json(root: &Path) -> PathBuf {
    root.join(".git/agent-on/oncall.json")
}

fn set_stale_after_minutes(root: &Path, minutes: u64) {
    let dir = root.join(".git/agent-on");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("config.json"),
        format!("{{\"oncall_stale_after_minutes\": {minutes}}}\n"),
    )
    .unwrap();
}

/// Rewrite the registration's timestamps to `minutes_ago`, the way a closed
/// window's record looks after the fact.
fn backdate(root: &Path, minutes_ago: i64) {
    let path = oncall_json(root);
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let stamp = (chrono::Utc::now() - chrono::Duration::minutes(minutes_ago)).to_rfc3339();
    value["started_at"] = serde_json::Value::String(stamp.clone());
    value["heartbeat_at"] = serde_json::Value::String(stamp);
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn heartbeat_of(root: &Path) -> String {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(oncall_json(root)).unwrap()).unwrap();
    value["heartbeat_at"].as_str().unwrap_or("").to_string()
}

#[test]
fn claim_records_a_heartbeat() {
    let (_tmp, root, _feature) = fixture();
    let out = agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    assert!(out.status.success(), "{}", combined(&out));
    assert!(
        !heartbeat_of(&root).is_empty(),
        "claim must stamp heartbeat_at"
    );
}

#[test]
fn expired_registration_fails_open_and_can_be_reclaimed_without_force() {
    let (_tmp, root, feature) = fixture();
    set_stale_after_minutes(&root, 30);
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    backdate(&root, 120);

    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));

    let status = combined(&agent_on(&feature, &["oncall", "status"]));
    assert!(
        status.contains("过期") || status.contains("失效"),
        "{status}"
    );
    assert!(status.contains("oncall-a"), "{status}");

    let out = agent_on(&feature, &["oncall", "claim", "--session", "feature-b"]);
    assert!(out.status.success(), "{}", combined(&out));
}

#[test]
fn any_guarded_call_from_the_oncall_window_refreshes_the_heartbeat() {
    let (_tmp, root, feature) = fixture();
    set_stale_after_minutes(&root, 30);
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    backdate(&root, 120);
    let before = heartbeat_of(&root);

    // A plain read-only command from the on-call window is enough.
    let out = guard(&root, &bash_payload(&root, "ls -la"));
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
    let after = heartbeat_of(&root);
    assert_ne!(before, after, "heartbeat must move on on-call activity");

    // Alive again: the feature window is routed once more.
    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(2), "{}", combined(&out));
}

#[test]
fn feature_window_activity_does_not_refresh_someone_elses_heartbeat() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    backdate(&root, 5);
    let before = heartbeat_of(&root);
    guard(&feature, &bash_payload(&feature, "ls -la"));
    assert_eq!(before, heartbeat_of(&root));
}

#[test]
fn explicit_heartbeat_command_is_oncall_only() {
    let (_tmp, root, feature) = fixture();
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    backdate(&root, 5);
    let before = heartbeat_of(&root);

    let out = agent_on(&feature, &["oncall", "heartbeat"]);
    assert!(!out.status.success(), "{}", combined(&out));
    assert_eq!(before, heartbeat_of(&root));

    let out = agent_on(&root, &["oncall", "heartbeat"]);
    assert!(out.status.success(), "{}", combined(&out));
    assert_ne!(before, heartbeat_of(&root));
}

#[test]
fn block_message_tells_the_feature_window_when_the_lock_expires_on_its_own() {
    let (_tmp, root, feature) = fixture();
    set_stale_after_minutes(&root, 45);
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    let text = combined(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("心跳"), "{text}");
    assert!(text.contains("45"), "{text}");
}

#[test]
fn zero_disables_expiry() {
    let (_tmp, root, feature) = fixture();
    set_stale_after_minutes(&root, 0);
    agent_on(&root, &["oncall", "claim", "--session", "oncall-a"]);
    backdate(&root, 100_000);
    let out = guard(&feature, &bash_payload(&feature, "gh pr merge 17 --merge"));
    assert_eq!(out.status.code(), Some(2), "{}", combined(&out));
}
