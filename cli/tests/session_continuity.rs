//! Real CLI/host-wire regression: window identity must survive shared cwd and compact.
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    repo: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("project");
        fs::create_dir(&repo).unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["config", "user.name", "Test"],
            vec!["config", "user.email", "test@example.com"],
        ] {
            assert!(Command::new("git")
                .current_dir(&repo)
                .args(args)
                .output()
                .unwrap()
                .status
                .success());
        }
        fs::write(repo.join("agent-on.lock.md"), "pin: v0.27.0\n").unwrap();
        Self { _tmp: tmp, repo }
    }
    fn cmd(&self, host: Option<&str>, id: &str) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(&self.repo);
        for key in [
            "CODEX_THREAD_ID",
            "AGENT_ON_HOST",
            "AGENT_ON_SESSION_ID",
            "AGENT_ON_PARENT_CODEX_THREAD_ID",
            "CLAUDE_ENV_FILE",
            "PLUGIN_ROOT",
            "CLAUDE_PLUGIN_ROOT",
            "AGENT_ON_CONTROL_ID",
            "CLAUDE_PROJECT_DIR",
            "CODEX_PROJECT_DIR",
        ] {
            cmd.env_remove(key);
        }
        if let Some(host) = host {
            cmd.env("AGENT_ON_HOST", host)
                .env("AGENT_ON_SESSION_ID", id);
        }
        cmd
    }
    fn cli(&self, host: Option<&str>, id: &str, args: &[&str]) -> Output {
        self.cmd(host, id).args(args).output().unwrap()
    }
    fn hook(&self, host: &str, id: &str, command: &str, event: &str) -> Output {
        let mut child = self
            .cmd(Some(host), id)
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let payload = json!({"cwd":self.repo,"session_id":id,"hook_event_name":event,"source":"compact","tool_name":"Bash","tool_input":{"command":"gh pr merge 17 --merge"}});
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.to_string().as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
    fn record(&self) -> Value {
        serde_json::from_slice(&fs::read(self.repo.join(".git/agent-on/oncall.json")).unwrap())
            .unwrap()
    }
    fn age(&self) {
        let mut record = self.record();
        record["heartbeat_at"] = (chrono::Utc::now() - chrono::Duration::minutes(5))
            .to_rfc3339()
            .into();
        fs::write(
            self.repo.join(".git/agent-on/oncall.json"),
            record.to_string(),
        )
        .unwrap();
    }
}
fn ok(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn same_directory_windows_cannot_impersonate_owner_or_renew_it() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    f.age();
    let before = f.record();
    let other = f.cli(Some("codex"), "other", &["oncall", "status", "--json"]);
    ok(&other);
    let status: Value = serde_json::from_slice(&other.stdout).unwrap();
    assert_eq!(status["self_is_oncall"], false);
    assert!(!f
        .cli(
            Some("codex"),
            "other",
            &["oncall", "claim", "--session", "other"]
        )
        .status
        .success());
    assert!(!f
        .cli(Some("codex"), "other", &["oncall", "heartbeat"])
        .status
        .success());
    assert!(!f
        .cli(Some("codex"), "other", &["oncall", "release"])
        .status
        .success());
    assert_eq!(
        f.hook("codex", "other", "guard", "PreToolUse")
            .status
            .code(),
        Some(2)
    );
    assert_eq!(f.record(), before);
    assert_eq!(
        f.hook("codex", "owner", "guard", "PreToolUse")
            .status
            .code(),
        Some(0)
    );
    assert_ne!(f.record()["heartbeat_at"], before["heartbeat_at"]);
}

#[test]
fn host_is_part_of_identity_even_when_ids_are_equal() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "same-id",
        &["oncall", "claim", "--session", "main"],
    ));
    assert_eq!(
        f.hook("claude", "same-id", "guard", "PreToolUse")
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        f.hook("codex", "same-id", "guard", "PreToolUse")
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn unknown_and_legacy_identity_do_not_grant_the_directory_owner_rights() {
    let f = Fixture::new();
    assert!(!f
        .cli(None, "", &["oncall", "claim", "--session", "unknown"])
        .status
        .success());
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    let mut record = f.record();
    record.as_object_mut().unwrap().remove("identity");
    fs::write(f.repo.join(".git/agent-on/oncall.json"), record.to_string()).unwrap();
    let status = f.cli(Some("codex"), "owner", &["oncall", "status", "--json"]);
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["present"], true);
    assert_eq!(status["self_is_oncall"], false);
    assert_eq!(status["identity_status"], "legacy-unverified");
    assert_eq!(
        f.hook("codex", "owner", "guard", "PreToolUse")
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn concurrent_same_directory_claims_choose_exactly_one_owner() {
    let f = Fixture::new();
    let mut a = f
        .cmd(Some("codex"), "a")
        .args(["oncall", "claim", "--session", "a"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut b = f
        .cmd(Some("codex"), "b")
        .args(["oncall", "claim", "--session", "b"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    assert_ne!(a.wait().unwrap().success(), b.wait().unwrap().success());
    let record = f.record();
    assert_eq!(record["session"], record["identity"]["session_id"]);
}

#[test]
fn delayed_old_owner_event_cannot_refresh_a_new_owner() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "old",
        &["oncall", "claim", "--session", "old"],
    ));
    ok(&f.cli(
        Some("codex"),
        "new",
        &["oncall", "claim", "--session", "new", "--force"],
    ));
    f.age();
    let before = f.record();
    assert_eq!(
        f.hook("codex", "old", "guard", "PreToolUse").status.code(),
        Some(2)
    );
    assert_eq!(f.record(), before);
}

#[test]
fn compact_reads_shared_project_state_without_enabling_capture_or_launching() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    let out = f.hook("claude", "next-window", "capture", "SessionStart");
    ok(&out);
    let context: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        context["hookSpecificOutput"]["hookEventName"],
        "SessionStart"
    );
    let text = context["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(text.contains("main"), "{text}");
    assert!(text.contains("未启用"), "{text}");
    assert!(!f.repo.join(".git/agent-on/control").exists());
}

#[test]
fn same_directory_feature_cannot_dispatch_using_owner_workdir() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    let out = f.cli(
        Some("codex"),
        "other",
        &[
            "dispatch",
            "--id",
            "worker",
            "--goal",
            "test",
            "--host",
            "codex",
            "--window",
            "external",
            "--read-only",
        ],
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("值守在班"));
    assert!(!f
        .repo
        .join(".git/agent-on/control/tasks/worker.json")
        .exists());
}

#[test]
fn hook_session_id_overrides_inherited_owner_identity() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    f.age();
    let before = f.record();
    let mut child = f
        .cmd(Some("codex"), "owner")
        .arg("guard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let payload = json!({"cwd":f.repo,"session_id":"other","turn_id":"1","tool_name":"Bash","tool_input":{"command":"gh pr merge 1 --merge","workdir":f.repo}});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    assert_eq!(child.wait_with_output().unwrap().status.code(), Some(2));
    assert_eq!(f.record(), before);
}

#[test]
fn claude_session_env_is_quoted_and_nested_codex_uses_its_own_native_id() {
    let f = Fixture::new();
    let env_file = f.repo.join("host-env");
    fs::write(&env_file, "export OTHER_HOOK='preserved'\n").unwrap();
    let id = "claude-'quoted'-$literal";
    let mut child = f
        .cmd(None, "")
        .env("CLAUDE_ENV_FILE", &env_file)
        .env("CODEX_THREAD_ID", "parent-codex")
        .arg("capture")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            json!({"cwd":f.repo,"session_id":id,"hook_event_name":"SessionStart"})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    ok(&child.wait_with_output().unwrap());
    assert!(fs::read_to_string(&env_file)
        .unwrap()
        .starts_with("export OTHER_HOOK='preserved'\n"));
    let out = Command::new("bash").current_dir(&f.repo).env_remove("AGENT_ON_HOST").env_remove("AGENT_ON_SESSION_ID").env("CODEX_THREAD_ID", "parent-codex").args(["-c", "source \"$1\"; test \"$OTHER_HOOK\" = preserved || exit 8; exec \"$2\" oncall claim --session claude-owner", "test"]).arg(&env_file).arg(env!("CARGO_BIN_EXE_agent-on")).output().unwrap();
    ok(&out);
    assert_eq!(
        f.record()["identity"],
        json!({"host":"claude","session_id":id})
    );
    let out = f
        .cmd(Some("claude"), id)
        .env("CODEX_THREAD_ID", "child-codex")
        .env("AGENT_ON_PARENT_CODEX_THREAD_ID", "parent-codex")
        .args(["oncall", "claim", "--session", "child", "--force"])
        .output()
        .unwrap();
    ok(&out);
    assert_eq!(
        f.record()["identity"],
        json!({"host":"codex","session_id":"child-codex"})
    );
}

#[test]
fn unmanaged_repo_startup_is_silent_and_does_not_persist_identity() {
    let f = Fixture::new();
    fs::remove_file(f.repo.join("agent-on.lock.md")).unwrap();
    let env_file = f.repo.join("host-env");
    fs::write(&env_file, "original\n").unwrap();
    let mut child = f
        .cmd(Some("claude"), "caller")
        .env("CLAUDE_ENV_FILE", &env_file)
        .arg("capture")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            json!({"cwd":f.repo,"session_id":"caller","hook_event_name":"SessionStart"})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let out = child.wait_with_output().unwrap();
    ok(&out);
    assert!(out.stdout.is_empty());
    assert_eq!(fs::read_to_string(env_file).unwrap(), "original\n");
    assert!(!f.repo.join(".git/agent-on").exists());
}

#[test]
fn corrupt_owner_record_is_unavailable_and_does_not_authorize_merge() {
    let f = Fixture::new();
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    fs::write(f.repo.join(".git/agent-on/oncall.json"), "{broken").unwrap();
    let out = f.hook("codex", "owner", "capture", "SessionStart");
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let context = value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("不可用"), "{context}");
    assert!(!context.contains("无人在班"), "{context}");
    assert_eq!(
        f.hook("codex", "owner", "guard", "PreToolUse")
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn corrupt_owner_record_whoami_reports_unavailable_not_nobody() {
    let f = Fixture::new();
    let registry = f.repo.join(".git/agent-on");
    fs::create_dir_all(&registry).unwrap();
    fs::write(registry.join("oncall.json"), "{broken").unwrap();
    let out = f.cli(Some("codex"), "caller", &["oncall", "whoami", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(value["role"], "unavailable");
    assert_eq!(value["is_oncall"], false);
    assert!(value["error"].as_str().unwrap().contains("parse"));
    let text = f.cli(Some("codex"), "caller", &["oncall", "whoami"]);
    assert_eq!(text.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&text.stderr).contains("不可用"));
    assert!(!String::from_utf8_lossy(&text.stderr).contains("无人在班"));
}

#[test]
fn invalid_host_or_session_id_cannot_claim() {
    let f = Fixture::new();
    for (host, id) in [
        ("unknown", "session"),
        ("codex", ""),
        ("claude", "bad\nidentity"),
        ("codex", &"x".repeat(257)),
    ] {
        assert!(!f
            .cli(Some(host), id, &["oncall", "claim", "--session", "address"])
            .status
            .success());
    }
    assert!(!f.repo.join(".git/agent-on/oncall.json").exists());
}

#[test]
fn plugin_matchers_cover_native_codex_commands_and_messages() {
    let config: Value = serde_json::from_str(include_str!("../../hooks/hooks.json")).unwrap();
    let groups = config["hooks"]["PreToolUse"].as_array().unwrap();
    for tool in [
        "Bash",
        "exec_command",
        "mcp__host__exec_command",
        "SendMessage",
        "mcp__codex_app__send_message_to_thread",
    ] {
        assert!(
            groups.iter().any(
                |group| regex::Regex::new(group["matcher"].as_str().unwrap())
                    .unwrap()
                    .is_match(tool)
            ),
            "no hook matches {tool}"
        );
    }
    assert!(config["hooks"]["SessionStart"]
        .as_array()
        .is_some_and(|v| !v.is_empty()));
}

#[test]
fn startup_task_summary_is_bounded_and_excludes_released_tasks() {
    let f = Fixture::new();
    for args in [vec!["add", "."], vec!["commit", "-qm", "base"]] {
        assert!(Command::new("git")
            .current_dir(&f.repo)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    for n in 0..7 {
        ok(&f.cli(
            Some("codex"),
            "caller",
            &["task", "add", &format!("task-{n}"), "--goal", "test"],
        ));
    }
    let released_path = f.repo.join(".git/agent-on/control/tasks/task-6.json");
    let mut released: Value = serde_json::from_slice(&fs::read(&released_path).unwrap()).unwrap();
    released["released_at"] = "2026-10-03T00:00:00Z".into();
    fs::write(released_path, released.to_string()).unwrap();
    let out = f.hook("codex", "caller", "capture", "SessionStart");
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let context = value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("未释放任务 6 项"), "{context}");
    for n in 0..5 {
        assert!(context.contains(&format!("task-{n}")), "{context}");
    }
    assert!(
        !context.contains("task-5") && !context.contains("task-6"),
        "{context}"
    );
}

#[test]
fn broken_tasks_and_unwritable_claude_environment_report_unavailable() {
    let f = Fixture::new();
    let tasks = f.repo.join(".git/agent-on/control/tasks");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(tasks.join("broken.json"), "{broken").unwrap();
    let out = f.hook("codex", "caller", "capture", "SessionStart");
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("不可用"));
    let mut child = f
        .cmd(Some("claude"), "caller")
        .env("CLAUDE_ENV_FILE", &f.repo)
        .arg("capture")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            json!({"cwd":f.repo,"session_id":"caller","hook_event_name":"SessionStart"})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let out = child.wait_with_output().unwrap();
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let context = value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("不可用") && context.contains("session environment unavailable"));
    assert!(!f.repo.join(".git/agent-on/control/sessions").exists());
}

#[test]
fn linked_worktree_compact_reads_primary_tasks_and_owner() {
    let f = Fixture::new();
    for args in [vec!["add", "."], vec!["commit", "-qm", "base"]] {
        assert!(Command::new("git")
            .current_dir(&f.repo)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    let linked = f.repo.parent().unwrap().join("linked");
    assert!(Command::new("git")
        .current_dir(&f.repo)
        .args(["worktree", "add", "-b", "linked"])
        .arg(&linked)
        .output()
        .unwrap()
        .status
        .success());
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "main"],
    ));
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["task", "add", "current-task", "--goal", "test"],
    ));
    let mut child = f
        .cmd(Some("codex"), "next")
        .current_dir(&linked)
        .arg("capture")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(json!({"cwd":linked,"session_id":"next","hook_event_name":"SessionStart","source":"compact"}).to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let context = value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        context.contains("current-task") && context.contains("main"),
        "{context}"
    );
    assert!(!linked.join(".git/agent-on").exists());
}

#[test]
fn damaged_capture_config_still_injects_unavailable_startup_context() {
    let f = Fixture::new();
    let control = f.repo.join(".git/agent-on/control");
    fs::create_dir_all(&control).unwrap();
    fs::write(control.join("config.json"), "{broken").unwrap();
    let out = f.hook("codex", "caller", "capture", "SessionStart");
    ok(&out);
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .contains("不可用"));
}

#[test]
fn same_directory_native_peer_messages_route_through_oncall() {
    let f = Fixture::new();
    for args in [vec!["add", "."], vec!["commit", "-qm", "base"]] {
        assert!(Command::new("git")
            .current_dir(&f.repo)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    ok(&f.cli(
        Some("codex"),
        "owner",
        &["oncall", "claim", "--session", "coordinator"],
    ));
    ok(&f.cli(
        Some("codex"),
        "owner",
        &[
            "patrol",
            "start",
            "--host",
            "codex-app",
            "--window",
            "external",
        ],
    ));
    ok(&f.hook("codex", "peer-b", "capture", "SessionStart"));
    ok(&f.hook("codex", "owner", "capture", "SessionStart"));
    let message = |recipient: &str| {
        let mut child = f
            .cmd(Some("codex"), "peer-a")
            .arg("guard")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(json!({"cwd":f.repo,"session_id":"peer-a","turn_id":"1","tool_name":"mcp__codex_app__send_message_to_thread","tool_input":{"threadId":recipient,"prompt":"test"}}).to_string().as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    };
    assert_eq!(message("peer-b").status.code(), Some(2));
    assert_eq!(message("coordinator").status.code(), Some(0));
    assert_eq!(message("owner").status.code(), Some(0));
    assert_eq!(message("main").status.code(), Some(0));
    assert_eq!(message("researcher").status.code(), Some(0));
    fs::write(f.repo.join(".git/agent-on/oncall.json"), "{broken").unwrap();
    assert_eq!(message("peer-b").status.code(), Some(2));
    // A damaged registry must not block communication inside this session.
    assert_eq!(message("main").status.code(), Some(0));
    assert_eq!(message("researcher").status.code(), Some(0));
}
