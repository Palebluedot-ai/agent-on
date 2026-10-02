//! End-to-end receipts and reclaim safety, using real Git and fake provider executables.
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    repo: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo with 'quotes'");
        fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-b", "main"]);
        git(&repo, &["config", "user.name", "Test"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        fs::write(repo.join("shared.txt"), "base\n").unwrap();
        fs::write(repo.join(".gitignore"), ".env\nnode_modules/\n").unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-qm", "base"]);
        Self { _tmp: tmp, repo }
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(&self.repo)
            .env_remove("AGENT_ON_CONTROL_ID")
            .env_remove("AGENT_ON_CONTROL_REPO");
        cmd.env_remove("CLAUDE_PROJECT_DIR")
            .env_remove("CODEX_PROJECT_DIR");
        cmd
    }
    fn cli(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> Value {
        let out = self.cli(args);
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn dir(&self) -> PathBuf {
        self.repo.join(".git/agent-on/control")
    }
    fn dispatch(&self, id: &str, path: &str) -> Value {
        self.ok(&[
            "dispatch",
            "--id",
            id,
            "--goal",
            "test task",
            "--host",
            "claude",
            "--window",
            "external",
            "--worktree",
            "--path",
            path,
        ])
    }
    fn close_release(&self, value: &Value) {
        let session = value["session"]["id"].as_str().unwrap();
        let task = value["task"]["id"].as_str().unwrap();
        self.ok(&[
            "patrol",
            "close",
            session,
            "--evidence",
            "test host was never started",
        ]);
        self.ok(&[
            "task",
            "release",
            task,
            "--evidence",
            "verified fixture work",
        ]);
    }
    fn enable(&self) {
        self.ok(&["janitor", "enable", "--manual", "--quiet-hours", "0"]);
    }
    fn hook(&self, payload: Value) {
        let mut child = self
            .command()
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
        if !out.stdout.is_empty() {
            let context: Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(
                context["hookSpecificOutput"]["hookEventName"],
                "UserPromptSubmit"
            );
        }
        assert!(
            out.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(repo)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}

#[test]
fn simultaneous_patrol_launches_reuse_one_unready_receipt() {
    let f = Fixture::new();
    let mut a = f
        .command()
        .args([
            "patrol",
            "start",
            "--host",
            "codex-app",
            "--window",
            "external",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let b = f
        .command()
        .args([
            "patrol",
            "start",
            "--host",
            "codex-app",
            "--window",
            "external",
        ])
        .output()
        .unwrap();
    let mut bytes = Vec::new();
    use std::io::Read;
    a.stdout.take().unwrap().read_to_end(&mut bytes).unwrap();
    assert!(a.wait().unwrap().success());
    assert!(b.status.success());
    let a: Value = serde_json::from_slice(&bytes).unwrap();
    let b: Value = serde_json::from_slice(&b.stdout).unwrap();
    assert_eq!(a["session"]["id"], b["session"]["id"]);
    assert_eq!(
        f.ok(&["patrol", "status"])["sessions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(a["session"]["state"], "prepared");
    f.ok(&["patrol", "disable"]);
    assert_eq!(f.ok(&["patrol", "status"])["config"]["enabled"], false);
    let restarted = f.ok(&[
        "patrol",
        "start",
        "--host",
        "codex-app",
        "--window",
        "external",
    ]);
    assert_eq!(restarted["session"]["id"], a["session"]["id"]);
    assert_eq!(restarted["reused"], true);
    assert_eq!(f.ok(&["patrol", "status"])["config"]["enabled"], true);
}

#[test]
fn read_only_worker_does_not_block_a_writer_in_the_primary_checkout() {
    let f = Fixture::new();
    f.ok(&[
        "dispatch",
        "--id",
        "research",
        "--goal",
        "read only",
        "--host",
        "grok",
        "--window",
        "external",
        "--read-only",
    ]);
    f.ok(&[
        "dispatch",
        "--id",
        "writer",
        "--goal",
        "one writer",
        "--host",
        "codex",
        "--window",
        "external",
        "--path",
        "page",
    ]);
    assert!(!f
        .cli(&[
            "dispatch",
            "--id",
            "second",
            "--goal",
            "another writer",
            "--host",
            "claude",
            "--window",
            "external",
            "--path",
            "other"
        ])
        .status
        .success());
}

#[cfg(unix)]
#[test]
fn retry_preserves_failed_tasks_checkout_and_refuses_a_duplicate_launch() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let value = f.dispatch("retry", "page");
    let id = value["session"]["id"].as_str().unwrap();
    let receipt = f.dir().join("sessions").join(format!("{id}.json"));
    let mut session: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    session["state"] = "failed".into();
    session["window"] = "tmux".into();
    fs::write(receipt, session.to_string()).unwrap();
    let bin = f.repo.join("fake-bin");
    fs::create_dir(&bin).unwrap();
    for name in ["claude", "tmux"] {
        let path = bin.join(name);
        fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let retried = f
        .command()
        .env("PATH", &path)
        .args(["patrol", "retry", id])
        .output()
        .unwrap();
    assert!(
        retried.status.success(),
        "{}",
        String::from_utf8_lossy(&retried.stderr)
    );
    let retried: Value = serde_json::from_slice(&retried.stdout).unwrap();
    assert_eq!(retried["checkout"], "reused");
    assert_eq!(retried["session"]["id"], id);
    assert_eq!(retried["session"]["cwd"], value["task"]["worktree"]);
    assert_eq!(retried["ready"], false);
    assert_eq!(
        f.ok(&["task", "list"])["tasks"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        git(&f.repo, &["worktree", "list", "--porcelain"])
            .lines()
            .filter(|s| s.starts_with("worktree "))
            .count(),
        2
    );
    assert!(!f
        .command()
        .env("PATH", path)
        .args(["patrol", "retry", id])
        .output()
        .unwrap()
        .status
        .success());
}

#[cfg(unix)]
#[test]
fn codex_and_grok_adapters_preserve_explicit_model_and_exact_resume_id() {
    use std::os::unix::fs::PermissionsExt;
    for host in ["codex", "grok"] {
        let f = Fixture::new();
        let value = f.ok(&[
            if host == "codex" {
                "agent-os"
            } else {
                "dispatch"
            },
            "--id",
            "adapter",
            "--goal",
            "literal goal",
            "--host",
            host,
            "--window",
            "external",
            "--read-only",
            "--model",
            "explicit-model",
        ]);
        let id = value["session"]["id"].as_str().unwrap();
        let receipt = f.dir().join("sessions").join(format!("{id}.json"));
        let mut session: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
        session["state"] = "starting".into();
        session["resume"] = true.into();
        session["host_session_id"] = "exact-host-id".into();
        fs::write(receipt, session.to_string()).unwrap();
        let bin = f.repo.join("fake-bin");
        fs::create_dir(&bin).unwrap();
        let provider = bin.join(host);
        fs::write(
            &provider,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$TEST_ARGV\"\n",
        )
        .unwrap();
        fs::set_permissions(provider, fs::Permissions::from_mode(0o755)).unwrap();
        let argv = f.repo.join("argv.txt");
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = f
            .command()
            .env("PATH", path)
            .env("TEST_ARGV", &argv)
            .args(["patrol", "serve", id])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let args = fs::read_to_string(argv).unwrap();
        assert!(args.contains("--model\nexplicit-model\n"), "{args}");
        assert!(args.contains("exact-host-id\n"), "{args}");
        assert!(
            !args.contains("--last") && !args.contains("bypass") && !args.contains("dangerously")
        );
        if host == "codex" {
            assert!(args.contains("--sandbox\nread-only\n"));
            assert!(args.contains("resume\nexact-host-id\n"));
        } else {
            assert!(args.contains("--tools\nRead,Glob,Grep\n"));
            assert!(args.contains("--resume\nexact-host-id\n"));
        }
    }
}

#[test]
fn actual_scan_includes_already_committed_overlap() {
    let f = Fixture::new();
    let a = f.dispatch("a", "page-a");
    let b = f.dispatch("b", "page-b");
    for (value, text) in [(&a, "a\n"), (&b, "b\n")] {
        let path = Path::new(value["task"]["worktree"].as_str().unwrap());
        fs::write(path.join("shared.txt"), text).unwrap();
        git(path, &["add", "shared.txt"]);
        git(path, &["commit", "-qm", text]);
    }
    let scan = f.ok(&["patrol", "scan"]);
    assert!(scan["conflicts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["kind"] == "actual-path-overlap" && row["paths"] == json!(["shared.txt"])));
}

#[test]
fn dashboard_reads_the_same_tasks_and_results_from_primary_and_linked_trees() {
    let f = Fixture::new();
    let task = f.dispatch("panel", "page");
    let primary = f.ok(&["dashboard", "--json"]);
    let linked = f.ok(&[
        "dashboard",
        "--json",
        "--repo",
        task["task"]["worktree"].as_str().unwrap(),
    ]);
    assert_eq!(primary["repo"], linked["repo"]);
    assert_eq!(
        primary["patrol"]["data"]["tasks"],
        linked["patrol"]["data"]["tasks"]
    );
    assert_eq!(primary["worktrees"].as_array().unwrap().len(), 2);
    f.ok(&[
        "task",
        "progress",
        "panel",
        "--state",
        "working",
        "--note",
        "implementing the panel; next run browser checks",
    ]);
    let progress = f.ok(&["dashboard", "--json"]);
    assert_eq!(progress["patrol"]["data"]["tasks"][0]["state"], "working");
    let result = f._tmp.path().join("result.txt");
    fs::write(&result, "actual test result; source evidence in test log").unwrap();
    f.ok(&[
        "task",
        "result",
        "panel",
        "--file",
        result.to_str().unwrap(),
    ]);
    let next = f.ok(&["status", "--json"]);
    assert_eq!(next["patrol"]["data"]["results"][0]["verified"], false);
    assert_eq!(next["patrol"]["data"]["tasks"][0]["state"], "reported");
    assert!(next["patrol"]["data"]["tasks"][0]["released_head"].is_null());
    assert!(next["patrol"]["data"]["results"][0]["summary"]
        .as_str()
        .unwrap()
        .contains("actual test result"));
    fs::write(f.dir().join("tasks/panel.json"), "broken JSON").unwrap();
    let damaged = f.ok(&["dashboard", "--json"]);
    assert_eq!(damaged["patrol"]["available"], false);
    assert_eq!(damaged["worktrees"].as_array().unwrap().len(), 2);
    assert!(damaged["worktrees"][1]["managed"].is_null());
}

#[cfg(unix)]
#[test]
fn hermes_adapter_preserves_query_resume_and_exit_without_claiming_hook_readiness() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let value = f.ok(&[
        "dispatch",
        "--id",
        "hermes-task",
        "--goal",
        "literal $(touch SHOULD_NOT_EXIST) `echo nope`",
        "--host",
        "hermes",
        "--window",
        "external",
        "--worktree",
        "--path",
        "page",
        "--model",
        "explicit-model",
    ]);
    let id = value["session"]["id"].as_str().unwrap();
    let receipt = f.dir().join("sessions").join(format!("{id}.json"));
    let mut session: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    session["state"] = "starting".into();
    session["resume"] = true.into();
    session["host_session_id"] = "exact-hermes-session".into();
    fs::write(&receipt, session.to_string()).unwrap();
    let bin = f._tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let exe = bin.join("hermes");
    fs::write(
        &exe,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$TEST_ARGV\"\nexit 7\n",
    )
    .unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
    let argv = f._tmp.path().join("argv.txt");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = f
        .command()
        .env("PATH", path)
        .env("TEST_ARGV", &argv)
        .args(["patrol", "serve", id])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["session"]["exit_code"], 7);
    assert_eq!(result["host_success"], false);
    let args = fs::read_to_string(argv).unwrap();
    assert!(args.starts_with("chat\n--in\n"));
    assert!(args.contains("--resume\nexact-hermes-session\n--query\n"));
    assert!(args.contains("--model\nexplicit-model\n"));
    assert!(args.contains("$(touch SHOULD_NOT_EXIST)"));
    assert!(!args.contains("--yolo") && !args.contains("--accept-hooks"));
    assert!(!Path::new(value["task"]["worktree"].as_str().unwrap())
        .join("SHOULD_NOT_EXIST")
        .exists());
    assert!(!f
        .cli(&[
            "dispatch",
            "--id",
            "unsupported",
            "--host",
            "hermes",
            "--goal",
            "read",
            "--read-only",
            "--window",
            "external"
        ])
        .status
        .success());
    assert!(!f.dir().join("tasks/unsupported.json").exists());
}

#[test]
fn overlap_is_caught_before_checkout_or_model_creation() {
    let f = Fixture::new();
    f.dispatch("a", "src/shared/**");
    let out = f.cli(&[
        "dispatch",
        "--id",
        "b",
        "--goal",
        "b",
        "--host",
        "grok",
        "--window",
        "external",
        "--worktree",
        "--path",
        "src/shared/Card.tsx",
    ]);
    assert!(!out.status.success());
    assert!(!f.dir().join("checkouts/b").exists());
    assert!(!f.dir().join("tasks/b.json").exists());
}

#[test]
fn capture_is_opt_in_and_stop_does_not_release_host() {
    let f = Fixture::new();
    f.hook(json!({"cwd":f.repo,"hook_event_name":"UserPromptSubmit","session_id":"main","turn_id":"1","prompt":"hello"}));
    assert!(!f.dir().exists());
    let patrol = f.ok(&[
        "patrol",
        "start",
        "--host",
        "codex-app",
        "--window",
        "external",
    ]);
    f.hook(json!({"cwd":f.repo,"hook_event_name":"UserPromptSubmit","session_id":"main","turn_id":"1","prompt":"API_KEY=secret-value Authorization: Bearer auth-token password='private phrase' improve home"}));
    f.hook(json!({"cwd":f.repo,"hook_event_name":"UserPromptSubmit","session_id":"main","turn_id":"1","prompt":"API_KEY=secret-value Authorization: Bearer auth-token password='private phrase' improve home"}));
    f.hook(json!({"cwd":f.repo,"hook_event_name":"Stop","session_id":"main","last_assistant_message":"preview checked API_KEY=secret-value"}));
    let state = f.ok(&["patrol", "status"]);
    assert_eq!(state["report"]["inbox"].as_array().unwrap().len(), 1);
    assert!(!state.to_string().contains("secret-value"));
    assert!(!state.to_string().contains("auth-token"));
    assert!(!state.to_string().contains("private phrase"));
    assert_eq!(state["report"]["results"][0]["source"], "host-stop-hook");
    assert_eq!(state["report"]["results"][0]["verified"], false);
    assert!(state["report"]["results"][0]["summary"]
        .as_str()
        .unwrap()
        .contains("preview checked"));
    assert!(state["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["host_session_id"] == "main" && s["state"] == "ready"));
    let id = patrol["session"]["id"].as_str().unwrap();
    let bound = f.ok(&["patrol", "bind", id, "--thread-id", "native-chat"]);
    assert_eq!(bound["ready"], false);
    f.hook(json!({"cwd":f.repo,"hook_event_name":"SessionStart","session_id":"native-chat"}));
    assert!(f.ok(&["patrol", "status"])["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["id"] == id && s["state"] == "ready"));
}

#[test]
fn native_hooks_before_bind_are_reconciled_and_results_keep_the_managed_task() {
    let f = Fixture::new();
    f.ok(&[
        "patrol",
        "start",
        "--host",
        "codex-app",
        "--window",
        "external",
    ]);
    let worker = f.ok(&[
        "dispatch",
        "--id",
        "native",
        "--goal",
        "native task",
        "--host",
        "codex-app",
        "--window",
        "external",
        "--path",
        "page",
    ]);
    f.hook(json!({"cwd":f.repo,"hook_event_name":"SessionStart","session_id":"early-host"}));
    f.hook(json!({"cwd":f.repo,"hook_event_name":"Stop","session_id":"early-host","last_assistant_message":"initial result"}));
    let id = worker["session"]["id"].as_str().unwrap();
    let bound = f.ok(&["patrol", "bind", id, "--thread-id", "early-host"]);
    assert_eq!(bound["ready"], true);
    let report = f.ok(&["patrol", "status"]);
    assert_eq!(report["report"]["results"][0]["task"], "native");
    let other = f.ok(&[
        "dispatch",
        "--id",
        "other",
        "--goal",
        "read",
        "--host",
        "codex-app",
        "--window",
        "external",
        "--read-only",
    ]);
    assert!(!f
        .cli(&[
            "patrol",
            "bind",
            other["session"]["id"].as_str().unwrap(),
            "--thread-id",
            "early-host"
        ])
        .status
        .success());
    f.hook(json!({"cwd":f.repo,"hook_event_name":"Stop","session_id":"early-host","last_assistant_message":"final result"}));
    let results = f.ok(&["patrol", "status"])["report"]["results"]
        .as_array()
        .unwrap()
        .clone();
    assert!(results
        .iter()
        .any(|result| result["summary"] == "final result"
            && result["task"] == "native"
            && result["verified"] == false));
    f.hook(json!({"cwd":f.repo,"hook_event_name":"SessionEnd","session_id":"early-host"}));
    f.ok(&[
        "task",
        "release",
        "native",
        "--evidence",
        "actual native closure observed",
    ]);
    assert!(f.ok(&["patrol", "status"])["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|session| session["id"] == id && session["state"] == "closed"));
}

#[test]
fn disabled_scheduled_janitor_is_quiet_and_live_file_handles_protect_checkout() {
    let f = Fixture::new();
    let value = f.dispatch("held", "page");
    let path = PathBuf::from(value["task"]["worktree"].as_str().unwrap());
    f.close_release(&value);
    let disabled = f.ok(&["janitor", "run", "--apply", "--scheduled"]);
    assert_eq!(disabled["janitor_enabled"], false);
    assert!(path.exists());
    f.enable();
    let handle = fs::File::open(path.join("shared.txt")).unwrap();
    let held = f.ok(&["janitor", "run", "--apply"]);
    assert_eq!(held["rows"][0]["decision"], "protected", "{held}");
    assert!(
        held["rows"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("process"),
        "{held}"
    );
    assert!(path.exists());
    drop(handle);
    assert_eq!(
        f.ok(&["janitor", "run", "--apply"])["rows"][0]["decision"],
        "reclaimed"
    );
    // Historical reclaimed ownership no longer blocks an unrelated new writer.
    f.dispatch("new", "page");
}

#[test]
fn reclaim_requires_policy_release_and_preserves_branch_and_recovery() {
    let f = Fixture::new();
    let value = f.dispatch("unused", "page");
    let path = PathBuf::from(value["task"]["worktree"].as_str().unwrap());
    assert!(!f.cli(&["janitor", "run", "--apply"]).status.success());
    f.enable();
    assert_eq!(
        f.ok(&["janitor", "run", "--apply"])["rows"][0]["decision"],
        "protected"
    );
    f.close_release(&value);
    let out = f.ok(&["janitor", "run", "--apply"]);
    assert_eq!(out["rows"][0]["decision"], "reclaimed", "{out}");
    assert!(!path.exists());
    git(&f.repo, &["rev-parse", "refs/heads/codex/task-unused"]);
    git(&f.repo, &["rev-parse", "refs/agent-on/recovery/unused"]);
    assert!(f.ok(&["janitor", "run", "--apply"])["rows"]
        .as_array()
        .unwrap()
        .is_empty());
    f.ok(&["janitor", "restore", "unused"]);
    assert!(path.join("shared.txt").exists());
    assert_eq!(
        f.ok(&["janitor", "run", "--apply"])["rows"][0]["decision"],
        "protected"
    );
}

#[test]
fn ignored_resources_dirty_changes_and_locked_checkout_are_protected() {
    let f = Fixture::new();
    let value = f.dispatch("protect", "page");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    f.close_release(&value);
    f.enable();
    fs::write(path.join(".env"), "private\n").unwrap();
    let out = f.ok(&["janitor", "run", "--apply"]);
    assert!(out["rows"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("ignored local resources"));
    assert!(path.exists());
    fs::remove_file(path.join(".env")).unwrap();
    fs::write(path.join("shared.txt"), "unsaved\n").unwrap();
    assert!(f.ok(&["janitor", "run", "--apply"])["rows"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("unstaged"));
    git(path, &["restore", "shared.txt"]);
    git(&f.repo, &["worktree", "lock", path.to_str().unwrap()]);
    assert!(f.ok(&["janitor", "run", "--apply"])["rows"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("locked"));
}

#[test]
fn worker_head_never_counts_as_primary_integration() {
    let f = Fixture::new();
    let value = f.dispatch("unique", "page");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    fs::write(path.join("shared.txt"), "unique\n").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-qm", "unique"]);
    f.close_release(&value);
    f.enable();
    let out = f
        .command()
        .current_dir(path)
        .args(["janitor", "run", "--apply"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let out: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out["rows"][0]["decision"], "protected");
    assert_eq!(out["base_sha"], git(&f.repo, &["rev-parse", "HEAD"]));
    assert!(path.exists());
}

#[cfg(unix)]
fn fake_tool(dir: &Path, name: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).unwrap();
    let file = dir.join(name);
    fs::write(&file, script).unwrap();
    fs::set_permissions(file, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn launcher_runs_the_independent_host_and_keeps_stdout_machine_readable() {
    let f = Fixture::new();
    let bin = f.repo.join("launch-bin");
    fake_tool(
        &bin,
        "tmux",
        "#!/bin/sh\nprintf '%s\\n' 'launcher diagnostic'\n/bin/sh -c \"$4\"\n",
    );
    fake_tool(&bin, "claude", "#!/bin/sh\nexit 0\n");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = f
        .command()
        .env("PATH", path)
        .args([
            "dispatch",
            "--id",
            "launched",
            "--goal",
            "test",
            "--host",
            "claude",
            "--window",
            "tmux",
            "--worktree",
            "--path",
            "page",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let receipt: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(receipt["session"]["state"], "exited");
    assert_eq!(receipt["session"]["exit_code"], 0);
    assert_eq!(receipt["ready"], false); // No hook evidence; process exit is not fabricated readiness.
}

#[cfg(unix)]
#[test]
fn merged_pr_must_cover_released_head_after_squash() {
    let f = Fixture::new();
    let value = f.dispatch("squash", "page");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    fs::write(path.join("shared.txt"), "change\n").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-qm", "feature"]);
    let head = git(path, &["rev-parse", "HEAD"]);
    git(&f.repo, &["merge", "--squash", "codex/task-squash"]);
    git(&f.repo, &["commit", "-qm", "squashed result"]);
    assert_ne!(head, git(&f.repo, &["rev-parse", "HEAD"]));
    f.close_release(&value);
    f.enable();
    let bin = f.repo.join("gh-bin");
    let prs = json!([{"number":1,"state":"MERGED","headRefName":"codex/task-squash","baseRefName":"main","headRefOid":head,"url":"https://example.invalid/pr/1","mergedAt":"2026-10-01T00:00:00Z"}]);
    fake_tool(
        &bin,
        "gh",
        &format!("#!/bin/sh\ncat <<'JSON'\n{prs}\nJSON\n"),
    );
    let env_path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = f
        .command()
        .env("PATH", env_path)
        .args(["janitor", "run", "--apply"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["rows"][0]["decision"], "reclaimed", "{report}");
}

#[test]
fn new_commits_after_release_are_protected_even_when_later_integrated() {
    let f = Fixture::new();
    let value = f.dispatch("newwork", "page");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    f.close_release(&value);
    f.enable();
    fs::write(path.join("shared.txt"), "after release\n").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-qm", "more work"]);
    git(&f.repo, &["merge", "--ff-only", "codex/task-newwork"]);
    assert!(!f
        .cli(&[
            "dispatch",
            "--id",
            "consumer",
            "--goal",
            "use shared result",
            "--host",
            "codex",
            "--window",
            "external",
            "--worktree",
            "--path",
            "consumer",
            "--depends-on",
            "newwork"
        ])
        .status
        .success());
    assert!(f.ok(&["janitor", "run", "--apply"])["rows"][0]["reason"]
        .as_str()
        .unwrap()
        .contains("HEAD changed after release"));
}

#[test]
fn explicit_repo_argument_cannot_impersonate_the_oncall_worktree() {
    let f = Fixture::new();
    let value = f.dispatch("feature", "page");
    assert!(f
        .cli(&["oncall", "claim", "--session", "owner"])
        .status
        .success());
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    let out = f
        .command()
        .current_dir(path)
        .args([
            "dispatch",
            "--repo",
            f.repo.to_str().unwrap(),
            "--id",
            "nested",
            "--goal",
            "test",
            "--host",
            "codex",
            "--window",
            "external",
            "--read-only",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("值守在班"));
}

#[test]
fn native_worktree_bind_records_actual_host_checkout_without_claiming_janitor_ownership() {
    let f = Fixture::new();
    let value = f.ok(&[
        "dispatch",
        "--id",
        "native",
        "--goal",
        "test",
        "--host",
        "codex-app",
        "--window",
        "external",
        "--worktree",
        "--path",
        "page",
    ]);
    assert!(!f.dir().join("checkouts/native").exists());
    let root = f.repo.parent().unwrap().join("native-host-tree");
    git(
        &f.repo,
        &["worktree", "add", "--detach", root.to_str().unwrap()],
    );
    let id = value["session"]["id"].as_str().unwrap();
    assert!(!f
        .cli(&["patrol", "bind", id, "--thread-id", "native-id"])
        .status
        .success());
    f.ok(&[
        "patrol",
        "bind",
        id,
        "--thread-id",
        "native-id",
        "--worktree-path",
        root.to_str().unwrap(),
    ]);
    let tasks = f.ok(&["task", "list"]);
    assert_eq!(tasks["tasks"][0]["managed"], false);
    assert_eq!(
        Path::new(tasks["tasks"][0]["worktree"].as_str().unwrap()),
        fs::canonicalize(root).unwrap()
    );
}

#[test]
fn released_dependency_must_also_be_integrated() {
    let f = Fixture::new();
    let value = f.dispatch("shared", "shared.txt");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    fs::write(path.join("shared.txt"), "new api\n").unwrap();
    git(path, &["add", "."]);
    git(path, &["commit", "-qm", "api"]);
    f.close_release(&value);
    let out = f.cli(&[
        "dispatch",
        "--id",
        "consumer",
        "--goal",
        "consume api",
        "--host",
        "codex",
        "--window",
        "external",
        "--worktree",
        "--depends-on",
        "shared",
        "--path",
        "consumer.txt",
    ]);
    assert!(!out.status.success());
    assert!(!f.dir().join("checkouts/consumer").exists());
    git(&f.repo, &["merge", "--ff-only", "codex/task-shared"]);
    f.ok(&[
        "dispatch",
        "--id",
        "consumer",
        "--goal",
        "consume api",
        "--host",
        "codex",
        "--window",
        "external",
        "--worktree",
        "--depends-on",
        "shared",
        "--path",
        "consumer.txt",
    ]);
}

#[test]
fn ignored_cache_allowlist_does_not_allow_env_or_database_files() {
    let f = Fixture::new();
    let value = f.dispatch("cache", "page");
    let path = Path::new(value["task"]["worktree"].as_str().unwrap());
    f.close_release(&value);
    f.ok(&[
        "janitor",
        "enable",
        "--manual",
        "--quiet-hours",
        "0",
        "--regenerable-ignored",
        "node_modules",
    ]);
    fs::create_dir(path.join("node_modules")).unwrap();
    fs::write(path.join("node_modules/local.db"), "saved data").unwrap();
    assert_eq!(
        f.ok(&["janitor", "run", "--apply"])["rows"][0]["decision"],
        "protected"
    );
    fs::remove_file(path.join("node_modules/local.db")).unwrap();
    fs::write(path.join("node_modules/rebuildable.js"), "cache").unwrap();
    assert_eq!(
        f.ok(&["janitor", "run", "--apply"])["rows"][0]["decision"],
        "reclaimed"
    );
}

#[cfg(unix)]
#[test]
fn real_runner_keeps_goal_literal_and_reports_host_exit() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let value = f.dispatch("literal", "page");
    let id = value["session"]["id"].as_str().unwrap();
    let receipt = f.dir().join("sessions").join(format!("{id}.json"));
    let mut session: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    session["state"] = "starting".into();
    fs::write(&receipt, session.to_string()).unwrap();
    let tools = f.repo.join("fake-bin");
    fs::create_dir(&tools).unwrap();
    let provider = tools.join("claude");
    fs::write(
        &provider,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$TEST_ARGV\"\nprintf '%s\\n' \"$CLAUDE_PROJECT_DIR\" \"$CODEX_PROJECT_DIR\" > \"$TEST_CWD\"\nexit 7\n",
    )
    .unwrap();
    fs::set_permissions(&provider, fs::Permissions::from_mode(0o755)).unwrap();
    let task_path = f.dir().join("tasks/literal.json");
    let mut task: Value = serde_json::from_slice(&fs::read(&task_path).unwrap()).unwrap();
    task["goal"] = "literal `touch SHOULD_NOT_EXIST` $(touch ALSO_NOT_EXIST) ; quote '".into();
    fs::write(task_path, task.to_string()).unwrap();
    let argv = f.repo.join("argv.txt");
    let cwd_report = f.repo.join("child-cwd.txt");
    let path = format!(
        "{}:{}",
        tools.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = f
        .command()
        .env("PATH", path)
        .env("TEST_ARGV", &argv)
        .env("TEST_CWD", &cwd_report)
        .env("CLAUDE_PROJECT_DIR", &f.repo)
        .env("CODEX_PROJECT_DIR", &f.repo)
        .args(["patrol", "serve", id])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let output: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(output["session"]["exit_code"], 7);
    assert_eq!(output["host_success"], false);
    let actual_cwd = value["task"]["worktree"].as_str().unwrap();
    assert_eq!(
        fs::read_to_string(cwd_report).unwrap(),
        format!("{actual_cwd}\n{actual_cwd}\n")
    );
    assert!(fs::read_to_string(argv)
        .unwrap()
        .contains("$(touch ALSO_NOT_EXIST)"));
    assert!(!f.repo.join("SHOULD_NOT_EXIST").exists());
    assert!(!f.repo.join("ALSO_NOT_EXIST").exists());
}
