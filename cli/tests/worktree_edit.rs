use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

/// One repo with one claimed lane worktree, ready for `worktree edit` calls.
fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "test@example.com"]);
    must_run(&root, "git", &["config", "user.name", "Test"]);
    fs::write(root.join("README.md"), "root\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let lane = tmp.path().join("lane-a");
    must_run(
        &root,
        "git",
        &["worktree", "add", "-b", "lane/a", lane.to_str().unwrap(), "main"],
    );
    let claimed = agent_on(
        &lane,
        &[
            "worktree", "claim", "--id", "lane-a", "--goal", "initial goal", "--base", "main",
            "--owns", "app",
        ],
    );
    assert!(claimed.status.success(), "{}", combined(&claimed));
    (tmp, root, lane)
}

#[test]
fn edit_cli_splits_comma_owns_and_keeps_quoted_octal_comma_path() {
    let (_tmp, root, lane) = fixture();
    // No --id: the current worktree's lane is the target.
    let edited = agent_on(
        &lane,
        &[
            "worktree",
            "edit",
            "--owns",
            "docs/a.md,docs/b.md",
            "--owns",
            r#""x\054y.md""#,
        ],
    );
    let text = combined(&edited);
    assert!(edited.status.success(), "{text}");
    assert!(text.contains("EDITED lane-a"), "{text}");
    assert!(text.contains("owns: docs/a.md, docs/b.md, x,y.md"), "{text}");
    let record = fs::read_to_string(root.join(".git/agent-on/lanes/lane-a.json")).unwrap();
    for boundary in ["\"docs/a.md\"", "\"docs/b.md\"", "\"x,y.md\""] {
        assert!(record.contains(boundary), "{record}");
    }
    assert!(!record.contains("docs/a.md,docs/b.md"), "{record}");
}

#[test]
fn edit_cli_updates_goal_and_base_by_id() {
    let (_tmp, root, lane) = fixture();
    let edited = agent_on(
        &lane,
        &[
            "worktree",
            "edit",
            "--id",
            "lane-a",
            "--goal",
            "redivided from cli",
            "--base",
            "main",
        ],
    );
    let text = combined(&edited);
    assert!(edited.status.success(), "{text}");
    assert!(text.contains("goal: redivided from cli"), "{text}");
    assert!(text.contains("base: main @ "), "{text}");
    let record = fs::read_to_string(root.join(".git/agent-on/lanes/lane-a.json")).unwrap();
    assert!(record.contains("redivided from cli"), "{record}");
}

#[test]
fn edit_cli_with_no_fields_fails_with_guidance() {
    let (_tmp, _root, lane) = fixture();
    let edited = agent_on(&lane, &["worktree", "edit"]);
    let text = combined(&edited);
    assert!(!edited.status.success(), "{text}");
    assert!(text.contains("nothing to edit"), "{text}");
}
