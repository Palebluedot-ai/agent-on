//! Local system projection: real Git fixtures, no private platform databases or model calls.
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

fn command(repo: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_agent-on"));
    command
        .current_dir(repo)
        .env_remove("CODEX_THREAD_ID")
        .env("AGENT_ON_HOST", "codex")
        .env("AGENT_ON_SESSION_ID", "system-test")
        .env_remove("AGENT_ON_CONTROL_ID")
        .env_remove("AGENT_ON_CONTROL_REPO")
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("CODEX_PROJECT_DIR");
    command
}
fn ok(repo: &Path, args: &[&str]) -> Value {
    let out = command(repo).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn repo(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "-b", "main"]);
    git(&path, &["config", "user.name", "Fixture"]);
    git(&path, &["config", "user.email", "test@example.com"]);
    fs::write(path.join("work.txt"), "base").unwrap();
    git(&path, &["add", "work.txt"]);
    git(&path, &["commit", "-qm", "base"]);
    path
}
fn overview(root: &Path) -> Value {
    ok(
        root,
        &[
            "dashboard",
            "--system",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ],
    )
}
fn record(repo: &Path, category: &str, id: &str, value: Value) {
    let dir = repo.join(".git/agent-on/control").join(category);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{id}.json")), value.to_string()).unwrap();
}

#[test]
fn discovers_unconnected_projects_without_creating_ledgers() {
    let tmp = TempDir::new().unwrap();
    let a = repo(tmp.path(), "a");
    let b = repo(tmp.path(), "b");
    let non_git = tmp.path().join("design");
    fs::create_dir(&non_git).unwrap();
    fs::write(non_git.join("AGENTS.md"), "scope").unwrap();
    let v = overview(tmp.path());
    assert_eq!(v["scope"], "system");
    assert_eq!(v["projects"].as_array().unwrap().len(), 3);
    assert!(v["projects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|p| p["connected"] == false));
    assert!(!a.join(".git/agent-on").exists());
    assert!(!b.join(".git/agent-on").exists());
    assert!(!non_git.join(".git").exists());
}
#[test]
fn task_stage_and_fresh_receipts_do_not_claim_native_window_activity() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    record(
        &p,
        "tasks",
        "review",
        json!({"id":"review","state":"reported"}),
    );
    record(&p, "tasks", "work", json!({"id":"work","state":"working"}));
    record(
        &p,
        "sessions",
        "window",
        json!({"id":"window","host":"codex","state":"ready","heartbeat_at":chrono::Utc::now().to_rfc3339()}),
    );
    let v = overview(tmp.path());
    assert_eq!(v["projects"][0]["activity"]["available"], false);
    assert_eq!(v["projects"][0]["activity"]["state"], "unknown");
    assert!(v["projects"][0]["activity"]["running_windows"].is_null());
    assert_eq!(v["tasks"][0]["source"], "task-ledger");
    assert!(v["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["state"] == "reported"));
    assert!(v["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["state"] == "working"));
    let detail = ok(&p, &["dashboard", "--json"]);
    assert_eq!(detail["activity"]["available"], false);
    assert_eq!(detail["activity"]["state"], "unknown");
}
#[test]
fn namespaces_identical_task_ids_and_keeps_damaged_project_visible() {
    let tmp = TempDir::new().unwrap();
    let a = repo(tmp.path(), "a");
    let b = repo(tmp.path(), "b");
    let damaged = repo(tmp.path(), "damaged");
    for path in [&a, &b] {
        record(
            path,
            "tasks",
            "same",
            json!({"id":"same","goal":"scope","state":"working","last_reporter":{"host":"codex"}}),
        );
    }
    record(
        &a,
        "sessions",
        "claude",
        json!({"id":"claude","host":"claude","task":"same","state":"failed","exit_code":1,"heartbeat_at":"2000-01-01T00:00:00Z"}),
    );
    record(&damaged, "tasks", "broken", json!([]));
    let v = overview(tmp.path());
    let tasks = v["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_ne!(tasks[0]["key"], tasks[1]["key"]);
    let projects = v["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 3);
    assert_eq!(
        projects.iter().find(|p| p["name"] == "damaged").unwrap()["available"],
        false
    );
    assert_eq!(
        v["agents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == "claude")
            .unwrap()["stale"],
        true
    );
    assert!(v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["source"] == "session"));
}
#[test]
fn linked_worktrees_are_one_project_and_explicit_paths_work() {
    let tmp = TempDir::new().unwrap();
    let primary = repo(tmp.path(), "primary");
    let linked = tmp.path().join("linked");
    git(
        &primary,
        &["worktree", "add", "--detach", linked.to_str().unwrap()],
    );
    let v = ok(
        tmp.path(),
        &[
            "dashboard",
            "--system",
            "--root",
            tmp.path().to_str().unwrap(),
            "--project",
            linked.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(v["projects"].as_array().unwrap().len(), 1);
    assert_eq!(v["projects"][0]["worktrees"], 2);
    assert_eq!(
        v["projects"][0]["repo"],
        fs::canonicalize(primary)
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
}
#[test]
fn skips_archive_generated_and_soft_links_but_explicit_can_include_archive() {
    let tmp = TempDir::new().unwrap();
    repo(tmp.path(), "normal");
    let archive = tmp.path().join("_archive");
    fs::create_dir(&archive).unwrap();
    let old = repo(&archive, "old");
    let vendor = tmp.path().join("node_modules");
    fs::create_dir(&vendor).unwrap();
    repo(&vendor, "dependency");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&archive, tmp.path().join("shortcut")).unwrap();
    assert_eq!(
        overview(tmp.path())["projects"].as_array().unwrap().len(),
        1
    );
    let v = ok(
        tmp.path(),
        &[
            "dashboard",
            "--system",
            "--root",
            tmp.path().to_str().unwrap(),
            "--project",
            old.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(v["projects"].as_array().unwrap().len(), 2);
}
#[test]
fn missing_root_and_broken_git_metadata_are_explicit_not_healthy_empty() {
    let tmp = TempDir::new().unwrap();
    repo(tmp.path(), "good");
    let broken = tmp.path().join("broken");
    fs::create_dir(&broken).unwrap();
    fs::write(broken.join(".git"), "not git").unwrap();
    let missing = tmp.path().join("missing");
    let v = ok(
        tmp.path(),
        &[
            "dashboard",
            "--system",
            "--root",
            tmp.path().to_str().unwrap(),
            "--root",
            missing.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(v["projects"].as_array().unwrap().len(), 2);
    assert!(!v["discovery"]["warnings"].as_array().unwrap().is_empty());
    assert!(v["projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["available"] == false));
}
#[test]
fn all_five_platforms_report_without_task_oncall_or_dispatch_authority() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    for host in ["codex", "claude", "hermes", "grok", "antigravity"] {
        let receipt = ok(
            &p,
            &[
                "agent",
                "report",
                "--host",
                host,
                "--id",
                "native-window",
                "--state",
                "blocked",
                "--note",
                "api_key=PRIVATE permission denied",
            ],
        );
        assert_eq!(receipt["verified"], false);
        assert!(!receipt["note"].as_str().unwrap().contains("PRIVATE"));
        let file = p.join(format!(
            ".git/agent-on/control/agents/{host}-native-window.json"
        ));
        assert!(file.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    let v = overview(tmp.path());
    let reports = v["agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["source"] == "self-report")
        .count();
    assert_eq!(reports, 5);
    assert_eq!(v["errors"].as_array().unwrap().len(), 5);
    assert_eq!(v["tasks"], json!([]));
    assert!(!p.join(".git/agent-on/control/tasks").exists());
    assert!(!p.join(".git/agent-on/control/sessions").exists());
    assert_eq!(ok(&p, &["oncall", "status", "--json"])["present"], false);
}
#[test]
fn report_updates_one_observation_but_never_releases_or_changes_task() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    ok(&p, &["task", "add", "work", "--goal", "real scope"]);
    let path = p.join(".git/agent-on/control/tasks/work.json");
    let before = fs::read(&path).unwrap();
    for state in ["working", "waiting", "blocked", "done"] {
        ok(
            &p,
            &[
                "agent", "report", "--host", "hermes", "--id", "one", "--state", state, "--task",
                "work",
            ],
        );
    }
    assert_eq!(fs::read(path).unwrap(), before);
    let v = overview(tmp.path());
    let reports = v["agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["source"] == "self-report")
        .collect::<Vec<_>>();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["state"], "done");
    assert!(v["errors"].as_array().unwrap().is_empty());
    assert!(v["tasks"][0]["hosts"]
        .as_array()
        .unwrap()
        .contains(&json!("hermes")));
}
#[test]
fn invalid_reports_and_cli_scope_conflicts_fail_without_writes() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    for args in [
        vec![
            "agent",
            "report",
            "--host",
            "hermes",
            "--id",
            "../escape",
            "--state",
            "working",
        ],
        vec![
            "agent",
            "report",
            "--host",
            "antigravity",
            "--id",
            "ok",
            "--state",
            "working",
            "--task",
            "missing",
        ],
        vec!["dashboard", "--system", "--repo", "."],
        vec!["dashboard", "--root", "."],
        vec!["dashboard", "--system", "--serve", "--refresh-landing"],
    ] {
        let out = command(&p).args(args).output().unwrap();
        assert!(!out.status.success());
    }
    assert!(!p.join(".git/agent-on/control/agents").exists());
}
#[test]
fn record_limits_and_secret_fields_do_not_leak_or_hide_damage() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    record(
        &p,
        "tasks",
        "work",
        json!({"id":"work","goal":"password=PRIVATE","state":"blocked","prompt":"TRANSCRIPT_PRIVATE","env":{"API_KEY":"PRIVATE"},"progress":{"note":"token=PRIVATE"}}),
    );
    fs::write(
        p.join(".git/agent-on/control/tasks/oversized.json"),
        vec![b'X'; 262145],
    )
    .unwrap();
    let v = overview(tmp.path());
    let text = v.to_string();
    assert!(!text.contains("TRANSCRIPT_PRIVATE"));
    assert!(!text.contains("PRIVATE"));
    assert_eq!(v["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(v["projects"][0]["available"], false);
}

struct Server {
    child: Child,
    address: String,
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Server {
    fn new(root: &Path) -> Self {
        let mut child = command(root)
            .args([
                "dashboard",
                "--system",
                "--root",
                root.to_str().unwrap(),
                "--serve",
                "--port",
                "0",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send(line);
        });
        let line = receiver
            .recv_timeout(Duration::from_secs(20))
            .expect("server startup timeout");
        if !line.contains("http://") {
            let _ = child.kill();
            let mut error = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut error);
            }
            let _ = child.wait();
            panic!("server failed to start: {error}");
        }
        let address = line
            .split("http://")
            .nth(1)
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .to_string();
        Self { child, address }
    }
    fn request(
        &self,
        method: &str,
        path: &str,
        header: bool,
        origin: Option<&str>,
    ) -> (u16, String) {
        let mut stream = TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {}\r\n",
            self.address
        )
        .unwrap();
        if header {
            write!(stream, "X-Agent-On: dashboard\r\n").unwrap();
        }
        if let Some(origin) = origin {
            write!(stream, "Origin: {origin}\r\n").unwrap();
        }
        write!(stream, "\r\n").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let code = response.split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = response.split_once("\r\n\r\n").unwrap().1.to_string();
        (code, body)
    }
}
#[test]
fn plain_project_directory_is_visible_even_without_a_manifest() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("unregistered");
    fs::create_dir(&path).unwrap();
    let v = overview(tmp.path());
    assert_eq!(v["projects"].as_array().unwrap().len(), 1);
    assert_eq!(v["projects"][0]["connected"], false);
    assert_eq!(v["projects"][0]["detail_available"], false);
}
#[cfg(unix)]
#[test]
fn in_root_git_alias_is_deduplicated_and_named() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "canonical");
    std::os::unix::fs::symlink(&p, tmp.path().join("alias")).unwrap();
    let v = overview(tmp.path());
    assert_eq!(v["projects"].as_array().unwrap().len(), 1);
    assert!(v["projects"][0]["aliases"]
        .as_array()
        .unwrap()
        .contains(&json!("alias")));
}
#[cfg(unix)]
#[test]
fn receipt_directory_symlink_is_not_read_or_written() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    let outside = TempDir::new().unwrap();
    fs::create_dir_all(p.join(".git/agent-on/control")).unwrap();
    std::os::unix::fs::symlink(outside.path(), p.join(".git/agent-on/control/agents")).unwrap();
    let out = command(&p)
        .args([
            "agent", "report", "--host", "hermes", "--id", "no", "--state", "working",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    let v = overview(tmp.path());
    assert_eq!(v["projects"][0]["available"], false);
}
#[test]
fn system_http_preserves_security_and_project_details_without_writes() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    let server = Server::new(tmp.path());
    let (code, body) = server.request("GET", "/", false, None);
    assert_eq!(code, 200);
    assert!(body.contains("系统总览"));
    let (code, body) = server.request("GET", "/api/status", true, None);
    assert_eq!(code, 200);
    let v: Value = serde_json::from_str(&body).unwrap();
    let id = v["projects"][0]["id"].as_str().unwrap();
    let api = format!("/api/projects/{id}");
    let page = format!("/projects/{id}");
    assert_eq!(server.request("GET", &api, false, None).0, 403);
    assert_eq!(
        server
            .request("GET", &api, true, Some("https://evil.test"))
            .0,
        403
    );
    assert_eq!(server.request("POST", &api, true, None).0, 405);
    assert_eq!(
        server
            .request("GET", "/api/projects/p0000000000000000", true, None)
            .0,
        404
    );
    assert_eq!(
        server
            .request("GET", "/projects/../../etc/passwd", true, None)
            .0,
        404
    );
    assert_eq!(server.request("GET", &page, false, None).0, 200);
    let (code, body) = server.request("GET", &api, true, None);
    assert_eq!(code, 200);
    let detail: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(detail["schema_version"], 1);
    assert!(!p.join(".git/agent-on/control").exists());
}
#[test]
fn fresh_receipts_update_live_without_restarting_the_server() {
    let tmp = TempDir::new().unwrap();
    let p = repo(tmp.path(), "project");
    let server = Server::new(tmp.path());
    ok(
        &p,
        &[
            "agent",
            "report",
            "--host",
            "antigravity",
            "--id",
            "live",
            "--state",
            "working",
        ],
    );
    let started = std::time::Instant::now();
    loop {
        let (_, body) = server.request("GET", "/api/status", true, None);
        let value: Value = serde_json::from_str(&body).unwrap();
        if value["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == "live" && a["state"] == "working")
        {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(12),
            "receipt did not refresh"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}
