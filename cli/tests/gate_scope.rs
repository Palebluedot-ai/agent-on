//! The commit gate blocks one thing only: this worktree has an uncommitted
//! path that another worktree also has uncommitted, and that other copy was
//! touched within 7 days. Lane registration is not an input. A clean pass is
//! the single line `ok`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime};
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

/// Rewrite fields of a lane record by hand, the way a stale or mistyped
/// registration arrives.
fn edit_record(root: &Path, id: &str, fields: &[(&str, &str)]) {
    let path = root.join(format!(".git/agent-on/lanes/{id}.json"));
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for (key, field) in fields {
        value[*key] = serde_json::json!(field);
    }
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn age_file(worktree: &Path, rel: &str, days: u64) {
    let file = fs::File::options()
        .write(true)
        .open(worktree.join(rel))
        .unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(days * 86_400))
        .unwrap();
}

#[test]
fn different_files_and_an_unregistered_tree_do_not_block() {
    let f = field();
    write_in(&f.extra, "notes.md", "scratch");
    write_in(&f.lane_a, "app/a.md", "lane-a work");

    assert_allowed(&f.lane_a);
    assert_allowed(&f.extra);
    assert_allowed(&f.root);

    let text = ok(&f.root, &["worktree", "check"]);
    assert_eq!(text, "ok\n");
}

#[test]
fn commit_is_blocked_only_when_another_tree_has_the_same_fresh_file() {
    let f = field();
    // The lane's owns cover `app`, but it has not touched this file.
    write_in(&f.extra, "app/a.md", "extra intrudes");
    assert_allowed(&f.extra);

    write_in(&f.lane_a, "app/a.md", "lane-a own work");
    let text = assert_blocked(&f.extra);
    assert!(
        text.contains("blocked: app/a.md is also uncommitted in"),
        "{text}"
    );
    assert!(!text.contains("worktree remove"), "{text}");
    assert!(!text.contains("set-status parked"), "{text}");
    assert_blocked(&f.lane_a);
}

#[test]
fn a_file_nobody_else_has_dirty_is_ok() {
    let f = field();
    write_in(&f.lane_a, "README.md", "escaped but unclaimed");

    assert_allowed(&f.lane_a);
    let text = ok(&f.lane_a, &["worktree", "check"]);
    assert_eq!(text, "ok\n");
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
    assert_eq!(ok(&f.lane_a, &["worktree", "check"]), "ok\n");

    // A lane whose worktree is gone can be forgotten whatever its status says.
    ok(&f.root, &["worktree", "forget", "--id", "lane-b"]);
    assert!(!f.root.join(".git/agent-on/lanes/lane-b.json").exists());
    assert_eq!(ok(&f.root, &["worktree", "check"]), "ok\n");
}

/// A lane whose `base` stops resolving (remote branch deleted, record edited
/// by hand) is that lane's own audit error. git's complaint about it must not
/// ride along on every other tree: they still read exactly `ok`, and their
/// hooks and guard stay silent.
#[test]
fn an_unresolvable_base_is_reported_only_on_its_own_tree() {
    let f = field();
    edit_record(&f.root, "lane-a", &[("base", "origin/nope")]);

    for tree in [&f.root, &f.extra] {
        for command in ["status", "check"] {
            let out = agent_on(tree, &["worktree", command]);
            assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
            assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
            assert_eq!(String::from_utf8_lossy(&out.stderr), "", "{command}");
        }
        for hook in ["pre-commit", "pre-push"] {
            let out = agent_on(tree, &["worktree", "hooks", "run", "--hook", hook]);
            assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
            assert_eq!(combined(&out), "", "{hook} passes silently");
        }
        let out = commit_guard(tree);
        assert_eq!(out.status.code(), Some(0), "{}", combined(&out));
        assert_eq!(combined(&out), "", "the guard passes silently");
    }

    // The lane itself still gets its error, and nothing ahead of it.
    let out = agent_on(&f.lane_a, &["worktree", "check"]);
    let text = combined(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(
        text.starts_with("error: lane-a: cannot compare with origin/nope: "),
        "{text}"
    );
}

/// `claim` measures an overlapping lane that is no longer live with the same
/// ancestry call, so a bad `base` there must not print either.
#[test]
fn claim_beside_an_unresolvable_base_prints_no_git_noise() {
    let f = field();
    edit_record(
        &f.root,
        "lane-a",
        &[("base", "origin/nope"), ("status", "parked")],
    );
    let out = agent_on(
        &f.extra,
        &[
            "worktree", "claim", "--id", "extra", "--goal", "app too", "--base", "main", "--owns",
            "app",
        ],
    );
    assert!(out.status.success(), "{}", combined(&out));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "");
}

#[test]
fn primary_is_blocked_only_when_another_tree_has_the_same_file() {
    let f = field();
    write_in(&f.root, "README.md", "control-track note");
    assert_allowed(&f.root);

    write_in(&f.root, "app/a.md", "primary intrudes");
    assert_allowed(&f.root);

    write_in(&f.lane_a, "app/a.md", "lane also");
    let text = assert_blocked(&f.root);
    assert!(
        text.contains("blocked: app/a.md is also uncommitted in"),
        "{text}"
    );
}

#[test]
fn a_copy_untouched_for_over_seven_days_does_not_block() {
    let f = field();
    write_in(&f.extra, "app/a.md", "extra intrudes");
    write_in(&f.lane_a, "app/a.md", "old lane work");
    age_file(&f.lane_a, "app/a.md", 30);
    assert_allowed(&f.extra);

    write_in(&f.lane_a, "app/a.md", "touched again");
    assert_blocked(&f.extra);
}

#[test]
fn two_unregistered_trees_on_one_fresh_file_block_each_other() {
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
    assert_blocked(&f.extra);
    assert_blocked(&extra2);
}

#[test]
fn check_json_lists_the_other_worktree_for_a_same_path_overlap() {
    let f = field();
    write_in(&f.extra, "app/a.md", "extra intrudes");
    write_in(&f.lane_a, "app/a.md", "lane also");
    let out = agent_on(&f.extra, &["worktree", "check", "--json"]);
    let text = combined(&out);
    let value: serde_json::Value = serde_json::from_str(text.trim()).expect(&text);
    let conflicts = value["conflicts"].as_array().expect(&text);
    let mine: Vec<_> = conflicts
        .iter()
        .filter(|c| c["path"] == "app/a.md" && c["other"].as_str().unwrap().contains("lane-a"))
        .collect();
    assert_eq!(mine.len(), 1, "{text}");
    assert!(value["missing"].is_array(), "{text}");
    assert!(
        !out.status.success(),
        "a same-path overlap is the one real failure"
    );
}
