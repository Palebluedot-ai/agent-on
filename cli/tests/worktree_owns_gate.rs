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

fn ok(cwd: &Path, args: &[&str]) -> String {
    let output = agent_on(cwd, args);
    let text = combined(&output);
    assert!(output.status.success(), "{} -> {text}", args.join(" "));
    text
}

/// Repo with two lane worktrees, plus a `shared/` directory both can reach.
fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(root.join("shared")).unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "test@example.com"]);
    must_run(&root, "git", &["config", "user.name", "Test"]);
    fs::write(root.join("shared/s.md"), "shared\n").unwrap();
    fs::write(root.join("app/a.md"), "app\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let a = tmp.path().join("lane-a");
    let b = tmp.path().join("lane-b");
    for (path, branch) in [(&a, "lane/a"), (&b, "lane/b")] {
        must_run(
            &root,
            "git",
            &["worktree", "add", "-b", branch, path.to_str().unwrap(), "main"],
        );
    }
    (tmp, a, b)
}

/// Take a lane through its whole lifecycle to the terminal `landed` state, then
/// redivide the reused worktree onto `shared` the way a returning session would.
fn landed_lane_redivided_onto_shared(a: &Path) {
    ok(
        a,
        &[
            "worktree", "claim", "--id", "lane-a", "--goal", "first goal", "--base", "main",
            "--owns", "app",
        ],
    );
    ok(a, &["worktree", "set-status", "ready"]);
    ok(a, &["worktree", "set-status", "landed"]);
    ok(
        a,
        &[
            "worktree",
            "edit",
            "--goal",
            "reused tree, new work",
            "--owns",
            "shared",
        ],
    );
}

fn write_in(worktree: &Path, rel: &str, line: &str) {
    let path = worktree.join(rel);
    let mut body = fs::read_to_string(&path).unwrap_or_default();
    body.push_str(line);
    body.push('\n');
    fs::write(&path, body).unwrap();
}

/// The gate must key off what is actually being written, not off a `status`
/// field that a reused worktree left behind on `landed`.
#[test]
fn check_fails_when_two_writing_lanes_share_a_boundary() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    // At this moment lane-a is clean, so lane-b's claim is legitimately allowed.
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "parallel work", "--base", "main",
            "--owns", "shared",
        ],
    );
    // Now both sessions actually write inside the shared boundary.
    write_in(&a, "shared/s.md", "lane-a writing");
    write_in(&b, "shared/s.md", "lane-b writing");

    for lane in [&a, &b] {
        let checked = agent_on(lane, &["worktree", "check"]);
        let text = combined(&checked);
        assert!(
            !checked.status.success(),
            "check must fail while two lanes write the same boundary: {text}"
        );
        assert!(text.contains("OVERLAP"), "{text}");
        assert!(text.contains("lane-a"), "{text}");
        assert!(text.contains("lane-b"), "{text}");
        assert!(text.contains("RESULT: FAIL"), "{text}");
    }
}

/// A landed lane whose work reached its base is genuinely finished; it must not
/// start blocking the boundaries it used to hold.
#[test]
fn check_passes_when_the_landed_lane_is_clean_and_merged() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "parallel work", "--base", "main",
            "--owns", "shared",
        ],
    );
    write_in(&b, "shared/s.md", "lane-b writing");

    let checked = agent_on(&b, &["worktree", "check"]);
    let text = combined(&checked);
    assert!(checked.status.success(), "{text}");
    assert!(!text.contains("OVERLAP"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");
}

/// The entry gate has to see the same fact: a landed worktree that is still
/// being written to still owns its boundary.
#[test]
fn claim_refuses_a_boundary_a_landed_lane_is_still_writing() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    write_in(&a, "shared/s.md", "lane-a writing");

    let claimed = agent_on(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "parallel work", "--base", "main",
            "--owns", "shared",
        ],
    );
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("lane-a"), "{text}");
    assert!(text.contains("shared"), "{text}");
}

/// Same fact, reached through the redivision gate instead of the entry gate.
#[test]
fn edit_refuses_a_boundary_a_landed_lane_is_still_writing() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "own work", "--base", "main",
            "--owns", "app",
        ],
    );
    write_in(&a, "shared/s.md", "lane-a writing");

    let edited = agent_on(&b, &["worktree", "edit", "--owns", "shared"]);
    let text = combined(&edited);
    assert!(!edited.status.success(), "{text}");
    assert!(text.contains("lane-a"), "{text}");
}

/// An unmerged commit is writing too, not just an uncommitted file.
#[test]
fn claim_refuses_a_boundary_a_landed_lane_holds_unmerged_commits_for() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    write_in(&a, "shared/s.md", "lane-a writing");
    must_run(&a, "git", &["add", "shared/s.md"]);
    must_run(&a, "git", &["commit", "-m", "lane-a work"]);

    let claimed = agent_on(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "parallel work", "--base", "main",
            "--owns", "shared",
        ],
    );
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("lane-a"), "{text}");
}

/// `status` is a registration, so it has to be correctable. The lifecycle
/// transition graph stays closed; `edit --status` is the explicit repair door.
#[test]
fn edit_status_reopens_a_landed_lane_that_set_status_cannot() {
    let (_tmp, a, _b) = fixture();
    landed_lane_redivided_onto_shared(&a);

    let refused = agent_on(&a, &["worktree", "set-status", "active"]);
    let refused_text = combined(&refused);
    assert!(!refused.status.success(), "{refused_text}");
    assert!(
        refused_text.contains("invalid lane transition: landed -> active"),
        "{refused_text}"
    );

    let edited = agent_on(&a, &["worktree", "edit", "--status", "active"]);
    let text = combined(&edited);
    assert!(edited.status.success(), "{text}");
    assert!(text.contains("status: active"), "{text}");

    let status = ok(&a, &["worktree", "status"]);
    assert!(status.contains("lane-a [active]"), "{status}");
}

/// A repaired registration re-arms the plain live-vs-live gate.
#[test]
fn edit_status_active_restores_the_live_boundary_gate() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    ok(&a, &["worktree", "edit", "--status", "active"]);

    let claimed = agent_on(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "parallel work", "--base", "main",
            "--owns", "shared",
        ],
    );
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("overlaps live lane lane-a"), "{text}");
}

/// `edit --status` relaxes the transition graph, not the invariants behind it.
#[test]
fn edit_status_still_enforces_the_active_cap() {
    let (_tmp, a, b) = fixture();
    let root = a.parent().unwrap().join("repo");
    let config = root.join(".git/agent-on/config.json");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "{\"active_cap\":1}").unwrap();
    landed_lane_redivided_onto_shared(&a);
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "own work", "--base", "main",
            "--owns", "app",
        ],
    );

    let edited = agent_on(&a, &["worktree", "edit", "--status", "active"]);
    let text = combined(&edited);
    assert!(!edited.status.success(), "{text}");
    assert!(text.contains("活跃轨上限已满"), "{text}");
}
