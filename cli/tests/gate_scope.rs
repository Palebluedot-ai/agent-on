//! The boundary gate blocks one thing only: *this* worktree's uncommitted
//! change landing inside another live lane's `owns`. Everything else the audit
//! sees — an unregistered worktree, a lane whose worktree vanished, a change
//! outside your own owns that nobody else holds — is reported, never fatal, and
//! never charged to a different worktree's commit.
//!
//! Before 2026-09-14 a single unregistered worktree failed every commit in the
//! repo (连坐). Desktop hosts create worktrees per session without claiming
//! them, so in practice every multi-window repo was red most of the time.

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

fn agent_on(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(cwd)
        .env("AGENT_ON_ROOT", "/nonexistent/agent-on-gate-scope-test")
        .args(args)
        .output()
        .unwrap()
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn ok(cwd: &Path, args: &[&str]) -> String {
    let output = agent_on(cwd, args);
    let text = combined(&output);
    assert!(output.status.success(), "{} -> {text}", args.join(" "));
    text
}

/// `git commit` as the Claude hook sees it, from inside `cwd`.
fn commit_guard(cwd: &Path) -> Output {
    let payload = format!(
        r#"{{"tool_name":"Bash","cwd":"{}","tool_input":{{"command":"git commit -m probe"}}}}"#,
        cwd.display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(cwd)
        .env("AGENT_ON_ROOT", "/nonexistent/agent-on-gate-scope-test")
        .env("CLAUDE_PROJECT_DIR", cwd)
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

fn assert_allowed(cwd: &Path) {
    let out = commit_guard(cwd);
    assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
}

fn assert_blocked(cwd: &Path) -> String {
    let out = commit_guard(cwd);
    let text = combined(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    text
}

struct Field {
    _tmp: TempDir,
    root: PathBuf,
    lane_a: PathBuf,
    extra: PathBuf,
}

/// Primary + a claimed lane (owns `app`) + one worktree nobody registered,
/// exactly what a desktop host leaves behind after opening a second session.
fn field() -> Field {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(root.join("app")).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "test@example.com"]);
    must_run(&root, "git", &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "root\n").unwrap();
    fs::write(root.join("app/a.md"), "app\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let lane_a = tmp.path().join("lane-a");
    let extra = tmp.path().join("extra");
    for (path, branch) in [(&lane_a, "lane/a"), (&extra, "session/extra")] {
        must_run(
            &root,
            "git",
            &[
                "worktree",
                "add",
                "-b",
                branch,
                path.to_str().unwrap(),
                "main",
            ],
        );
    }
    ok(
        &lane_a,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-a",
            "--goal",
            "change app",
            "--base",
            "main",
            "--owns",
            "app",
        ],
    );
    Field {
        _tmp: tmp,
        root,
        lane_a,
        extra,
    }
}

fn write_in(worktree: &Path, rel: &str, line: &str) {
    let path = worktree.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("{line}\n")).unwrap();
}

#[test]
fn unregistered_worktree_is_reported_but_blocks_nobody() {
    let f = field();
    write_in(&f.extra, "notes.md", "scratch");
    write_in(&f.lane_a, "app/a.md", "lane-a work");

    assert_allowed(&f.lane_a);
    assert_allowed(&f.extra);
    assert_allowed(&f.root);

    let text = ok(&f.root, &["worktree", "check"]);
    assert!(text.contains("UNREGISTERED:"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");
}

#[test]
fn commit_is_blocked_only_when_it_writes_inside_another_live_lanes_owns() {
    let f = field();
    write_in(&f.extra, "app/a.md", "extra intrudes");

    let text = assert_blocked(&f.extra);
    assert!(text.contains("CONFLICT"), "{text}");
    assert!(text.contains("app/a.md"), "{text}");
    assert!(text.contains("lane-a"), "{text}");
    // The exits must be commands the blocked window can run, and none of
    // them may be "delete something".
    assert!(text.contains("set-status parked"), "{text}");
    assert!(text.contains("--owns"), "{text}");
    assert!(!text.contains("worktree remove"), "{text}");

    // The lane whose ground was entered is not punished for the intruder.
    write_in(&f.lane_a, "app/a.md", "lane-a own work");
    assert_allowed(&f.lane_a);
}

#[test]
fn change_outside_own_owns_is_advisory_when_nobody_holds_the_path() {
    let f = field();
    write_in(&f.lane_a, "README.md", "escaped but unclaimed");

    assert_allowed(&f.lane_a);
    let text = ok(&f.root, &["worktree", "check"]);
    assert!(text.contains("OUT-OF-BOUNDS: README.md"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");
}

#[test]
fn live_lane_whose_worktree_vanished_blocks_nobody_and_can_be_forgotten() {
    let f = field();
    let lane_b = f._tmp.path().join("lane-b");
    must_run(
        &f.root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "lane/b",
            lane_b.to_str().unwrap(),
            "main",
        ],
    );
    ok(
        &lane_b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "docs", "--base", "main", "--owns",
            "docs",
        ],
    );
    must_run(
        &f.root,
        "git",
        &["worktree", "remove", "--force", lane_b.to_str().unwrap()],
    );

    write_in(&f.lane_a, "app/a.md", "still working");
    assert_allowed(&f.lane_a);

    let text = ok(&f.root, &["worktree", "check"]);
    assert!(text.contains("MISSING: lane-b"), "{text}");
    assert!(text.contains("forget --id lane-b"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");

    // A lane whose worktree is gone can be forgotten whatever its status says.
    ok(&f.root, &["worktree", "forget", "--id", "lane-b"]);
    let text = ok(&f.root, &["worktree", "check"]);
    assert!(!text.contains("lane-b"), "{text}");
}

#[test]
fn primary_worktree_is_blocked_only_when_it_enters_a_live_lane() {
    let f = field();
    write_in(&f.root, "README.md", "control-track note");
    assert_allowed(&f.root);

    write_in(&f.root, "app/a.md", "primary intrudes");
    let text = assert_blocked(&f.root);
    assert!(text.contains("CONFLICT"), "{text}");
    assert!(text.contains("lane-a"), "{text}");
}

#[test]
fn parking_the_live_lane_is_a_reachable_exit() {
    let f = field();
    write_in(&f.extra, "app/a.md", "extra intrudes");
    assert_blocked(&f.extra);

    ok(
        &f.lane_a,
        &["worktree", "set-status", "parked", "--id", "lane-a"],
    );
    assert_allowed(&f.extra);
}

#[test]
fn two_unregistered_trees_on_one_file_are_not_blocked() {
    let f = field();
    let extra2 = f._tmp.path().join("extra2");
    must_run(
        &f.root,
        "git",
        &[
            "worktree",
            "add",
            "-b",
            "session/extra2",
            extra2.to_str().unwrap(),
            "main",
        ],
    );
    write_in(&f.extra, "README.md", "one");
    write_in(&extra2, "README.md", "two");
    assert_allowed(&f.extra);
    assert_allowed(&extra2);
}

#[test]
fn check_json_exposes_conflicts_and_missing_lanes() {
    let f = field();
    write_in(&f.extra, "app/a.md", "extra intrudes");
    let out = agent_on(&f.root, &["worktree", "check", "--json"]);
    let text = combined(&out);
    let value: serde_json::Value = serde_json::from_str(text.trim()).expect(&text);
    let conflicts = value["conflicts"].as_array().expect(&text);
    assert_eq!(conflicts.len(), 1, "{text}");
    assert_eq!(conflicts[0]["path"], "app/a.md");
    assert_eq!(conflicts[0]["lane"], "lane-a");
    assert!(value["missing"].is_array(), "{text}");
    assert!(!out.status.success(), "a conflict is the one real failure");
}
