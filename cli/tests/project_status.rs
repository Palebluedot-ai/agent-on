//! Project delivery brief through the public CLI, using real Git and local receipts.
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cli(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(repo)
        .env_remove("CODEX_THREAD_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("CODEX_PROJECT_DIR")
        .env_remove("AGENT_ON_CONTROL_ID")
        .env_remove("AGENT_ON_CONTROL_REPO")
        .env("AGENT_ON_HOST", "codex")
        .env("AGENT_ON_SESSION_ID", "status-fixture")
        .args(args)
        .output()
        .unwrap()
}

fn ok(repo: &Path, args: &[&str]) -> String {
    let output = cli(repo, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-b", "main"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(temp.path(), &["config", "user.email", "test@example.com"]);
    fs::write(temp.path().join("base.txt"), "base\n").unwrap();
    git(temp.path(), &["add", "base.txt"]);
    git(temp.path(), &["commit", "-qm", "base"]);
    temp
}

#[test]
fn current_checkout_facts_do_not_invent_goals_or_read_retired_tasks() {
    let temp = fixture();
    let repo = temp.path();
    fs::write(repo.join("project-state.md"), "Existing project goal\n").unwrap();
    let old = repo.join(".git/agent-on/control/tasks");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("broken.json"), "{broken").unwrap();
    let before = fs::read(old.join("broken.json")).unwrap();
    let value: Value = serde_json::from_str(&ok(repo, &["status", "--json"])).unwrap();
    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["dirty"], true);
    assert_eq!(value["goal"], Value::Null);
    assert!(value["project_records"]
        .as_array()
        .unwrap()
        .contains(&Value::from("project-state.md")));
    assert!(value.get("patrol").is_none());
    assert!(value.get("activity").is_none());
    assert_eq!(fs::read(old.join("broken.json")).unwrap(), before);
    assert!(!repo.join(".git/agent-on/control/config.json").exists());
    let brief = ok(repo, &["status"]);
    assert!(brief.contains("下一步"));
    assert!(!brief.contains("打开"));
}
#[test]
fn linked_status_describes_the_callers_checkout_and_shares_only_delivery_evidence() {
    let temp = fixture();
    let repo = temp.path();
    let linked = temp.path().join("linked");
    git(
        repo,
        &["worktree", "add", "-b", "feature", linked.to_str().unwrap()],
    );
    fs::write(linked.join("new.txt"), "dirty").unwrap();
    let v: Value = serde_json::from_str(&ok(&linked, &["status", "--json"])).unwrap();
    assert_eq!(v["branch"], "feature");
    assert_eq!(v["dirty"], true);
    assert_eq!(
        Path::new(v["repo"].as_str().unwrap()),
        fs::canonicalize(linked).unwrap()
    );
}
#[test]
fn removed_commands_and_panel_flags_are_not_executable() {
    let temp = fixture();
    for args in [
        vec!["dashboard"],
        vec!["dispatch"],
        vec!["agent-os"],
        vec!["patrol"],
        vec!["activity"],
        vec!["agent"],
        vec!["task"],
        vec!["janitor"],
        vec!["model-check"],
        vec!["status", "--serve"],
        vec!["status", "--system"],
    ] {
        assert!(!cli(temp.path(), &args).status.success(), "{args:?}");
    }
    assert!(!temp.path().join(".git/agent-on/control").exists());
}
#[test]
fn retirement_is_previewable_repeatable_and_preserves_records_and_checkouts() {
    let temp = fixture();
    let repo = temp.path();
    let control = repo.join(".git/agent-on/control");
    fs::create_dir_all(control.join("tasks")).unwrap();
    fs::write(
        control.join("config.json"),
        r#"{"enabled":true,"janitor_enabled":true,"keep":"value"}"#,
    )
    .unwrap();
    fs::write(control.join("activity-config.json"), r#"{"enabled":true}"#).unwrap();
    fs::write(control.join("tasks/historical.json"), "history").unwrap();
    let original = fs::read(control.join("config.json")).unwrap();
    ok(repo, &["retire-legacy"]);
    assert_eq!(fs::read(control.join("config.json")).unwrap(), original);
    ok(repo, &["retire-legacy", "--apply"]);
    ok(repo, &["retire-legacy", "--apply"]);
    let v: Value = serde_json::from_slice(&fs::read(control.join("config.json")).unwrap()).unwrap();
    assert_eq!(v["enabled"], false);
    assert_eq!(v["janitor_enabled"], false);
    assert_eq!(v["keep"], "value");
    assert_eq!(
        fs::read(control.join("config.json.pre-v030")).unwrap(),
        original
    );
    assert_eq!(
        fs::read_to_string(control.join("tasks/historical.json")).unwrap(),
        "history"
    );
    assert!(repo.join("base.txt").is_file());
}
#[test]
fn retirement_refuses_bad_policy_without_discarding_it() {
    let temp = fixture();
    let control = temp.path().join(".git/agent-on/control");
    fs::create_dir_all(&control).unwrap();
    fs::write(control.join("config.json"), "{broken").unwrap();
    assert!(!cli(temp.path(), &["retire-legacy", "--apply"])
        .status
        .success());
    assert_eq!(
        fs::read_to_string(control.join("config.json")).unwrap(),
        "{broken"
    );
}
#[cfg(unix)]
#[test]
fn retirement_never_follows_config_symlinks() {
    let temp = fixture();
    let control = temp.path().join(".git/agent-on/control");
    fs::create_dir_all(&control).unwrap();
    let external = temp.path().join("external.json");
    fs::write(&external, r#"{"enabled":true}"#).unwrap();
    std::os::unix::fs::symlink(&external, control.join("config.json")).unwrap();
    assert!(!cli(temp.path(), &["retire-legacy", "--apply"])
        .status
        .success());
    assert_eq!(fs::read_to_string(external).unwrap(), r#"{"enabled":true}"#);
}
#[test]
fn old_non_startup_hooks_cannot_record_activity_even_when_old_policy_is_enabled() {
    use std::io::Write;
    use std::process::Stdio;
    let temp = fixture();
    let control = temp.path().join(".git/agent-on/control");
    fs::create_dir_all(&control).unwrap();
    fs::write(control.join("config.json"), r#"{"enabled":true}"#).unwrap();
    for event in [
        "PreToolUse",
        "PostToolUse",
        "UserPromptSubmit",
        "Stop",
        "SessionEnd",
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
            .current_dir(temp.path())
            .arg("capture")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(serde_json::json!({"cwd":temp.path(),"hook_event_name":event,"session_id":"old","tool_name":"Bash"}).to_string().as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert!(out.stdout.is_empty());
    }
    assert_eq!(fs::read_dir(control).unwrap().count(), 1);
}
