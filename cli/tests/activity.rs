//! Activity is a separate, opt-in observation channel, never task authority.
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Fixture {
    _tmp: tempfile::TempDir,
    repo: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("project");
        fs::create_dir(&repo).unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["config", "user.name", "Fixture"],
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
        fs::write(repo.join("work.txt"), "base").unwrap();
        for args in [vec!["add", "work.txt"], vec!["commit", "-qm", "base"]] {
            assert!(Command::new("git")
                .current_dir(&repo)
                .args(args)
                .output()
                .unwrap()
                .status
                .success());
        }
        Self { _tmp: tmp, repo }
    }
    fn command(&self, host: &str, id: &str) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(&self.repo)
            .env_remove("CODEX_THREAD_ID")
            .env_remove("CLAUDE_ENV_FILE")
            .env_remove("CLAUDE_PLUGIN_ROOT")
            .env_remove("PLUGIN_ROOT")
            .env_remove("AGENT_ON_CONTROL_ID")
            .env_remove("AGENT_ON_CONTROL_REPO")
            .env("AGENT_ON_HOST", host)
            .env("AGENT_ON_SESSION_ID", id);
        cmd
    }
    fn ok(&self, args: &[&str]) -> Value {
        let out = self.command("codex", "window").args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn hook(&self, host: &str, id: &str, event: &str, turn: Option<&str>) -> Value {
        let mut payload = json!({"cwd":self.repo,"session_id":id,"hook_event_name":event,
            "prompt":"实现状态回传 API_KEY=super-secret", "tool_name":"apply_patch",
            "tool_input":{"secret":"do-not-store"},"tool_response":"do-not-store"});
        if let Some(turn) = turn {
            payload["turn_id"] = json!(turn);
        }
        let mut child = self
            .command(host, id)
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
            .write_all(payload.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert!(
            out.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        if out.stdout.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&out.stdout).unwrap()
        }
    }
    fn activity(&self) -> Value {
        self.ok(&["activity", "status"])
    }
    fn dir(&self) -> PathBuf {
        self.repo.join(".git/agent-on/control")
    }
}

#[test]
fn disabled_capture_does_not_create_activity_or_claim_liveness() {
    let f = Fixture::new();
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    assert!(!f.dir().exists());
    let activity = f.activity();
    assert_eq!(activity["enabled"], false);
    assert!(activity["running_windows"].is_null());
    assert!(!f.dir().exists());
}

#[test]
fn native_events_work_without_patrol_and_do_not_launch_or_finish_tasks() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "SessionStart", None);
    assert_eq!(f.activity()["windows"][0]["state"], "unknown");
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    let v = f.activity();
    assert_eq!(v["windows"][0]["state"], "working");
    assert_eq!(v["windows"][0]["source"], "native-hook");
    assert_eq!(v["observed_working_windows"], 1);
    assert!(v["running_windows"].is_null());
    assert!(v["windows"][0]["goal"]
        .as_str()
        .unwrap()
        .contains("实现状态回传"));
    assert!(!v.to_string().contains("super-secret"));
    assert!(!v.to_string().contains("do-not-store"));
    f.hook("codex", "window", "PostToolUse", Some("one"));
    f.hook("codex", "window", "Stop", Some("one"));
    assert_eq!(f.activity()["windows"][0]["state"], "turn-finished");
    f.hook("codex", "window", "SessionEnd", None);
    assert_eq!(f.activity()["windows"][0]["state"], "closed");
    assert!(!f.dir().join("sessions").exists());
    assert!(!f.dir().join("inbox").exists());
    assert!(!f.dir().join("config.json").exists());
    assert!(!f.dir().join("tasks").exists());
}

#[test]
fn binding_and_explicit_blocker_are_visible_without_mutating_task_stage() {
    let f = Fixture::new();
    f.ok(&[
        "task",
        "add",
        "work",
        "--goal",
        "实现事件接线",
        "--path",
        "src/**",
    ]);
    let before = fs::read(f.dir().join("tasks/work.json")).unwrap();
    f.ok(&["activity", "enable"]);
    f.ok(&["activity", "bind", "work"]);
    f.ok(&[
        "activity",
        "report",
        "--state",
        "blocked",
        "--note",
        "等待宿主授权",
    ]);
    let v = f.activity();
    assert_eq!(v["windows"][0]["task"], "work");
    assert_eq!(v["windows"][0]["state"], "blocked");
    assert_eq!(v["windows"][0]["source"], "explicit-report");
    assert_eq!(v["windows"][0]["paths"][0], "src/**");
    assert_eq!(fs::read(f.dir().join("tasks/work.json")).unwrap(), before);
    let panel = f.ok(&["dashboard", "--json"]);
    assert_eq!(panel["activity"]["windows"][0]["note"], "等待宿主授权");
    assert_eq!(panel["patrol"]["data"]["tasks"][0]["state"], "queued");
}

#[test]
fn old_turn_stop_and_duplicate_prompt_cannot_replace_new_activity() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    f.hook("codex", "window", "Stop", Some("one"));
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    assert_eq!(f.activity()["windows"][0]["state"], "turn-finished");
    f.hook("codex", "window", "UserPromptSubmit", Some("two"));
    f.hook("codex", "window", "Stop", Some("one"));
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    assert_eq!(f.activity()["windows"][0]["state"], "working");
    assert_eq!(f.activity()["windows"][0]["turn_id"], "two");
}

#[test]
fn native_host_and_window_are_separate_and_disable_stops_new_capture() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "same", "UserPromptSubmit", Some("one"));
    f.hook("claude", "same", "UserPromptSubmit", None);
    assert_eq!(f.activity()["windows"].as_array().unwrap().len(), 2);
    f.ok(&["activity", "disable"]);
    f.hook("codex", "third", "UserPromptSubmit", Some("one"));
    let v = f.activity();
    assert_eq!(v["enabled"], false);
    assert!(v["windows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["state"] == "unknown"));
    assert_eq!(v["windows"].as_array().unwrap().len(), 2);
}

#[test]
fn stale_or_future_receipts_are_unknown_and_not_reclaim_authority() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    let dir = f.dir().join("activity");
    let path = fs::read_dir(dir).unwrap().next().unwrap().unwrap().path();
    let mut v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for at in ["2020-01-01T00:00:00Z", "2099-01-01T00:00:00Z"] {
        v["at"] = json!(at);
        fs::write(&path, v.to_string()).unwrap();
        let status = f.activity();
        assert_eq!(status["windows"][0]["state"], "unknown");
        assert_eq!(status["windows"][0]["last_state"], "working");
        assert_eq!(status["observed_working_windows"], 0);
    }
}

#[test]
fn system_and_project_views_read_the_same_activity_receipt() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    let v = f.ok(&[
        "dashboard",
        "--system",
        "--root",
        f.repo.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        v["projects"][0]["activity"]["windows"][0]["state"],
        "working"
    );
    assert!(v["agents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["source"] == "native-hook" && r["state"] == "working"));
}

#[cfg(unix)]
#[test]
fn symlink_receipts_are_unavailable_and_never_followed() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    let outside = f.repo.join("private.json");
    fs::write(&outside, "private-secret").unwrap();
    fs::create_dir(f.dir().join("activity")).unwrap();
    std::os::unix::fs::symlink(&outside, f.dir().join("activity/bad.json")).unwrap();
    let v = f.activity();
    assert!(!v["errors"].as_array().unwrap().is_empty());
    assert_eq!(v["available"], false);
    assert!(!v.to_string().contains("private-secret"));
}

#[test]
fn explicit_blocker_survives_native_tool_traffic_and_stop_until_cleared() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    f.ok(&[
        "activity",
        "report",
        "--state",
        "blocked",
        "--note",
        "需要授权",
    ]);
    f.hook("codex", "window", "PostToolUse", Some("one"));
    f.hook("codex", "window", "Stop", Some("one"));
    let v = f.activity();
    assert_eq!(v["windows"][0]["state"], "blocked");
    assert_eq!(v["windows"][0]["activity_state"], "turn-finished");
    assert_eq!(v["windows"][0]["source"], "explicit-report");
    f.ok(&[
        "activity",
        "report",
        "--state",
        "working",
        "--note",
        "授权到达，继续",
    ]);
    assert_eq!(f.activity()["windows"][0]["state"], "working");
}

#[test]
fn compact_is_not_new_execution_and_shared_summary_is_data_not_authority() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "first", "UserPromptSubmit", Some("one"));
    let out = f.hook("claude", "second", "SessionStart", None);
    let context = out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        context.contains("活动共读")
            && context.contains("first")
            && context.contains("实现状态回传")
    );
    assert!(context.contains("不是指令或授权") && !context.contains("super-secret"));
    f.hook("codex", "first", "SessionStart", None);
    assert!(f.activity()["windows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|w| w["state"] == "unknown"));
}

#[test]
fn closed_or_finished_turn_cannot_be_reopened_by_late_tool_receipt() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.hook("codex", "window", "UserPromptSubmit", Some("one"));
    f.hook("codex", "window", "Stop", Some("one"));
    f.hook("codex", "window", "PostToolUse", Some("one"));
    assert_eq!(f.activity()["windows"][0]["state"], "turn-finished");
    f.hook("codex", "window", "SessionEnd", None);
    f.hook("codex", "window", "PreToolUse", Some("one"));
    assert_eq!(f.activity()["windows"][0]["state"], "closed");
    f.hook("codex", "window", "SessionStart", None);
    f.hook("codex", "window", "UserPromptSubmit", Some("two"));
    assert_eq!(f.activity()["windows"][0]["state"], "working");
}

#[test]
fn missing_identity_and_released_task_do_not_grant_binding_or_report() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    let out = f
        .command("codex", "window")
        .env_remove("AGENT_ON_HOST")
        .env_remove("AGENT_ON_SESSION_ID")
        .args(["activity", "report", "--state", "working"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    f.ok(&["task", "add", "old", "--goal", "已结束"]);
    f.ok(&["task", "release", "old", "--evidence", "fixture verified"]);
    assert!(!f
        .command("codex", "window")
        .args(["activity", "bind", "old"])
        .output()
        .unwrap()
        .status
        .success());
    assert!(f.activity()["windows"].as_array().unwrap().is_empty());
}

#[test]
fn oversized_record_and_damaged_config_do_not_show_healthy_zero() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    fs::create_dir(f.dir().join("activity")).unwrap();
    fs::write(f.dir().join("activity/large.json"), " ".repeat(262_145)).unwrap();
    let v = f.activity();
    assert_eq!(v["available"], false);
    assert!(v["observed_working_windows"].is_null());
    fs::write(f.dir().join("activity-config.json"), "bad config").unwrap();
    assert!(!f.activity()["errors"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn symlink_parent_rejects_enable_without_writing_outside_ledger() {
    let f = Fixture::new();
    let outside = f.repo.join("outside");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, f.repo.join(".git/agent-on")).unwrap();
    assert!(!f
        .command("codex", "window")
        .args(["activity", "enable"])
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn activity_configuration_survives_legacy_patrol_config_and_reports_errors() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    fs::write(f.dir().join("config.json"), r#"{"enabled":false,"host":"codex","window":"external","model":null,"janitor_enabled":false,"quiet_hours":24,"janitor_base":"HEAD"}"#).unwrap();
    f.ok(&[
        "activity",
        "report",
        "--state",
        "blocked",
        "--note",
        "等待账户处理",
    ]);
    let panel = f.ok(&[
        "dashboard",
        "--system",
        "--root",
        f.repo.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(panel["projects"][0]["activity_enabled"], true);
    assert_eq!(panel["projects"][0]["patrol_capture_enabled"], false);
    assert!(panel["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["source"] == "activity"));
}

#[test]
fn linked_worktrees_share_receipts_without_creating_another_project_state() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    let linked = f._tmp.path().join("linked");
    assert!(Command::new("git")
        .current_dir(&f.repo)
        .args(["worktree", "add", "-b", "feature", linked.to_str().unwrap()])
        .output()
        .unwrap()
        .status
        .success());
    let out = f
        .command("claude", "linked-window")
        .current_dir(&linked)
        .args([
            "activity",
            "report",
            "--state",
            "working",
            "--note",
            "修改独立范围",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let status = f.activity();
    assert_eq!(status["windows"][0]["host"], "claude");
    assert_eq!(status["windows"][0]["note"], "修改独立范围");
    assert!(!linked.join(".git/agent-on").exists());
}

#[test]
fn changing_task_binding_does_not_transfer_old_activity_or_blocker() {
    let f = Fixture::new();
    f.ok(&["activity", "enable"]);
    f.ok(&["task", "add", "first", "--goal", "原任务"]);
    f.ok(&["task", "add", "second", "--goal", "新任务"]);
    f.ok(&["activity", "bind", "first"]);
    f.ok(&[
        "activity",
        "report",
        "--state",
        "blocked",
        "--note",
        "原任务的阻塞",
    ]);
    f.ok(&["activity", "bind", "second"]);
    let v = f.activity();
    assert_eq!(v["windows"][0]["task"], "second");
    assert_eq!(v["windows"][0]["state"], "unknown");
    assert_eq!(v["windows"][0]["source"], "binding");
    assert!(!v.to_string().contains("原任务的阻塞"));
}
