//! Real CLI/host-wire regression: window identity must survive shared cwd and compact.
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

/// Identity and host markers a live Claude/Codex session exports. Inherited by
/// `cargo test` run inside one, they would outrank the identity a case sets.
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

/// `program` without the identity of whichever session runs the tests.
fn isolated(program: &str) -> Command {
    let mut cmd = Command::new(program);
    for key in HOST_SESSION_ENV {
        cmd.env_remove(key);
    }
    cmd
}

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
        let mut cmd = isolated(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(&self.repo);
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

fn wire(cmd: &mut Command, action: &str, payload: Value) -> Output {
    let mut child = cmd
        .arg(action)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn claude_resume_claim_and_personal_guard_agree_without_plugin_environment() {
    let f = Fixture::new();
    let id = "3f2b762b-2ca2-4386-93bc-df153176a88b";
    let address = "30分钟值守循环";
    ok(&f.cli(
        Some("claude"),
        id,
        &["oncall", "claim", "--session", address],
    ));
    let mut record = f.record();
    record["heartbeat_at"] = (chrono::Utc::now() - chrono::Duration::hours(36))
        .to_rfc3339()
        .into();
    fs::write(f.repo.join(".git/agent-on/oncall.json"), record.to_string()).unwrap();

    // SessionStart persists adapter exports for Bash. Personal PreToolUse
    // hooks do not inherit that file or the plugin's environment markers.
    let env_file = f.repo.join("host-env");
    let resumed = wire(
        f.cmd(None, "")
            .env("CLAUDE_ENV_FILE", &env_file)
            .env("CLAUDE_CODE_SESSION_ID", id),
        "capture",
        json!({"cwd":f.repo,"session_id":id,"hook_event_name":"SessionStart","source":"resume"}),
    );
    ok(&resumed);
    assert_eq!(
        f.record(),
        record,
        "resume must not silently claim or refresh"
    );
    for args in [
        vec!["oncall", "claim", "--session", address],
        vec!["oncall", "status", "--json"],
        vec!["oncall", "whoami", "--json"],
    ] {
        let out = isolated("bash")
            .current_dir(&f.repo)
            .env("CLAUDE_CODE_SESSION_ID", id)
            .args(["-c", "source \"$1\"; shift; exec \"$@\"", "test"])
            .arg(&env_file)
            .arg(env!("CARGO_BIN_EXE_agent-on"))
            .args(&args)
            .output()
            .unwrap();
        ok(&out);
        if args.contains(&"--json") {
            let result: Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(
                result[if args[1] == "status" {
                    "self_is_oncall"
                } else {
                    "is_oncall"
                }],
                true
            );
        }
    }
    for command in [
        "gh pr merge 17 --merge",
        "gh pr review 17 --comment --body-file reply.md",
    ] {
        f.age();
        let before = f.record();
        let out = wire(
            f.cmd(None, "")
                .env("CLAUDE_CODE_SESSION_ID", id)
                .env("CLAUDE_CODE_HOST_SESSION_ID", "local_13a95139")
                .env("CLAUDE_CODE_CHILD_SESSION", "1"),
            "guard",
            json!({"cwd":f.repo,"session_id":id,"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":command}}),
        );
        ok(&out);
        assert_ne!(f.record()["heartbeat_at"], before["heartbeat_at"]);
    }
}

#[test]
fn claude_native_session_overrides_stale_bash_adapter_exports() {
    let f = Fixture::new();
    let out = f
        .cmd(Some("claude"), "old-id")
        .env("CLAUDE_CODE_SESSION_ID", "resumed-id")
        .args(["oncall", "claim", "--session", "main"])
        .output()
        .unwrap();
    ok(&out);
    assert_eq!(
        f.record()["identity"],
        json!({"host":"claude","session_id":"resumed-id"})
    );
}

fn diagnostic(out: &Output) -> Value {
    let text = String::from_utf8_lossy(&out.stderr);
    let raw = text
        .lines()
        .find_map(|line| line.strip_prefix("身份核对（guard 本次输入）："))
        .expect(&text);
    serde_json::from_str(raw).unwrap()
}

#[test]
fn claude_hook_payload_remains_authoritative_and_explains_each_identity_mismatch() {
    let f = Fixture::new();
    let owner = "owner-session";
    ok(&f.cli(
        Some("claude"),
        owner,
        &["oncall", "claim", "--session", "main"],
    ));
    f.age();
    let before = f.record();
    // Neither the UUID in Bash's environment nor the Desktop host's local_
    // identifier can override a different actual session_id in hook stdin.
    for id in ["other-session", "local_13a95139", "", "bad\nidentity"] {
        let out = wire(
            f.cmd(Some("claude"), owner)
                .env("CLAUDE_CODE_SESSION_ID", owner),
            "guard",
            json!({"cwd":f.repo,"session_id":id,"tool_name":"Bash","tool_input":{"command":"gh pr review 17 --comment --body-file reply.md"}}),
        );
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(
            f.record(),
            before,
            "a rejected window must not renew the owner"
        );
        let trace = diagnostic(&out);
        assert_eq!(trace["hook_session_id"], id);
        assert_eq!(trace["registered"]["session_id"], owner);
        assert_eq!(trace["session_id_source"], "hook stdin.session_id");
        if !id.is_empty() && !id.contains('\n') {
            assert_eq!(trace["mismatches"], json!(["session_id"]));
            assert_eq!(trace["actor"]["host"], "claude");
        } else {
            assert_eq!(trace["actor"], Value::Null);
        }
    }
    let out = wire(
        &mut f.cmd(Some("codex"), owner),
        "guard",
        json!({"cwd":f.repo,"session_id":owner,"turn_id":"codex-turn","tool_name":"exec_command","tool_input":{"cmd":"gh pr merge 17 --merge"}}),
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(diagnostic(&out)["mismatches"], json!(["host"]));
    assert_eq!(f.record(), before);
    let other_tree = f._tmp.path().join("other-tree");
    for args in [
        vec!["add", "."],
        vec!["commit", "-qm", "base"],
        vec![
            "worktree",
            "add",
            "-b",
            "other",
            other_tree.to_str().unwrap(),
        ],
    ] {
        assert!(Command::new("git")
            .current_dir(&f.repo)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    let out = wire(
        f.cmd(None, "").env("CLAUDE_CODE_SESSION_ID", owner),
        "guard",
        json!({"cwd":other_tree,"session_id":owner,"tool_name":"Bash","tool_input":{"command":"gh pr merge 17 --merge"}}),
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(diagnostic(&out)["mismatches"], json!(["worktree"]));
    assert_eq!(f.record(), before);
    // Unknown wiring does not acquire authority from cwd or a matching ID.
    let out = wire(
        &mut f.cmd(None, ""),
        "guard",
        json!({"cwd":f.repo,"session_id":owner,"tool_name":"Bash","tool_input":{"command":"gh pr merge 17 --merge"}}),
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(diagnostic(&out)["actor"], Value::Null);
    assert_eq!(diagnostic(&out)["hook_session_id"], owner);
    assert_eq!(f.record(), before);
}

#[test]
fn claude_native_identity_keeps_nested_codex_precedence_and_rejects_invalid_ids() {
    let f = Fixture::new();
    let mut cmd = f.cmd(Some("claude"), "adapter-parent");
    cmd.env("CLAUDE_CODE_SESSION_ID", "claude-parent")
        .env("CODEX_THREAD_ID", "native-codex-child");
    ok(&cmd
        .args(["oncall", "claim", "--session", "child"])
        .output()
        .unwrap());
    assert_eq!(
        f.record()["identity"],
        json!({"host":"codex","session_id":"native-codex-child"})
    );
    // A Claude launched from Codex has the parent thread explicitly marked.
    let out = f
        .cmd(Some("claude"), "stale-adapter")
        .env("CLAUDE_CODE_SESSION_ID", "claude-child")
        .env("CODEX_THREAD_ID", "parent-codex")
        .env("AGENT_ON_PARENT_CODEX_THREAD_ID", "parent-codex")
        .args(["oncall", "claim", "--session", "child", "--force"])
        .output()
        .unwrap();
    ok(&out);
    assert_eq!(
        f.record()["identity"],
        json!({"host":"claude","session_id":"claude-child"})
    );
    for id in ["", "bad\nidentity", &"x".repeat(257)] {
        let out = f
            .cmd(Some("claude"), "claude-child")
            .env("CLAUDE_CODE_SESSION_ID", id)
            .args(["oncall", "claim", "--session", "invalid", "--force"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(f.record()["identity"]["session_id"], "claude-child");
    }
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
    let out = isolated("bash").current_dir(&f.repo).env("CODEX_THREAD_ID", "parent-codex").args(["-c", "source \"$1\"; test \"$OTHER_HOOK\" = preserved || exit 8; exec \"$2\" oncall claim --session claude-owner", "test"]).arg(&env_file).arg(env!("CARGO_BIN_EXE_agent-on")).output().unwrap();
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
