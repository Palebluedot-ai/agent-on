//! The mutual-exclusion gate only means something while two sessions are in
//! fact writing. These tests pin the two facts that keep it from going
//! permanently red: a non-live lane holds only what it actually has in hand
//! (not everything it once declared), and it stops holding anything at all once
//! its unlanded work has gone cold.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

/// Repo with two lane worktrees and two directories either of them could reach.
fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(root.join("shared")).unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    must_run(&root, "git", &["init", "-b", "main"]);
    must_run(&root, "git", &["config", "user.email", "test@example.com"]);
    must_run(&root, "git", &["config", "user.name", "Test"]);
    fs::write(root.join("shared/s.md"), "shared\n").unwrap();
    fs::write(root.join("app/a.md"), "app\n").unwrap();
    fs::write(root.join("docs/d.md"), "docs\n").unwrap();
    must_run(&root, "git", &["add", "."]);
    must_run(&root, "git", &["commit", "-m", "init"]);
    let a = tmp.path().join("lane-a");
    let b = tmp.path().join("lane-b");
    for (path, branch) in [(&a, "lane/a"), (&b, "lane/b")] {
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
    (tmp, a, b)
}

fn write_in(worktree: &Path, rel: &str, line: &str) {
    let path = worktree.join(rel);
    let mut body = fs::read_to_string(&path).unwrap_or_default();
    body.push_str(line);
    body.push('\n');
    fs::write(&path, body).unwrap();
}

/// Push a file's mtime back so its edit reads as work nobody has touched in
/// `days` — the same fact a real abandoned worktree presents.
fn age_file(worktree: &Path, rel: &str, days: u64) {
    let file = fs::File::options()
        .write(true)
        .open(worktree.join(rel))
        .unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(days * 86_400))
        .unwrap();
}

/// Take lane-a to `landed` and redivide the reused worktree onto both
/// directories, the way a returning session would.
fn landed_lane_holding(a: &Path, owns: &[&str]) {
    ok(
        a,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-a",
            "--goal",
            "first goal",
            "--base",
            "main",
            "--owns",
            "app",
        ],
    );
    ok(a, &["worktree", "set-status", "ready"]);
    ok(a, &["worktree", "set-status", "landed"]);
    let mut args = vec!["worktree", "edit", "--goal", "reused tree, new work"];
    for path in owns {
        args.push("--owns");
        args.push(path);
    }
    ok(a, &args);
}

fn claim_b(b: &Path, owns: &str) -> Output {
    agent_on(
        b,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-b",
            "--goal",
            "parallel work",
            "--base",
            "main",
            "--owns",
            owns,
        ],
    )
}

/// A non-live lane reserves nothing for the future: it holds the paths it
/// actually has work in, and its other declared boundaries are free.
#[test]
fn writing_landed_lane_holds_only_what_it_actually_changed() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    // lane-a is writing, but only inside `app`.
    write_in(&a, "app/a.md", "lane-a writing");

    let claimed = claim_b(&b, "shared");
    let text = combined(&claimed);
    assert!(
        claimed.status.success(),
        "a paper-only boundary must not block a claim: {text}"
    );
}

/// The narrowing must not reach the path it really is writing.
#[test]
fn writing_landed_lane_still_holds_the_path_it_changed() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    write_in(&a, "shared/s.md", "lane-a writing");

    let claimed = claim_b(&b, "shared");
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("lane-a"), "{text}");
}

/// Unlanded work nobody has touched for longer than the dormancy window is not
/// a second writer; it is rescue debt. It must stop holding boundaries.
#[test]
fn dormant_lane_stops_holding_a_boundary() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    write_in(&a, "shared/s.md", "lane-a writing");
    age_file(&a, "shared/s.md", 30);

    let claimed = claim_b(&b, "shared");
    let text = combined(&claimed);
    assert!(
        claimed.status.success(),
        "cold work must not hold a boundary: {text}"
    );
}

/// Same fact through the redivision gate.
#[test]
fn edit_can_take_a_boundary_a_dormant_lane_declared() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    write_in(&a, "shared/s.md", "lane-a writing");
    age_file(&a, "shared/s.md", 30);
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "own work", "--base", "main",
            "--owns", "app/a.md",
        ],
    );

    let edited = agent_on(&b, &["worktree", "edit", "--owns", "shared"]);
    let text = combined(&edited);
    assert!(edited.status.success(), "{text}");
}

/// A dormant lane is reported, not silently dropped: the audit still says its
/// work is unrescued, it just stops being a red light.
#[test]
fn check_reports_dormant_work_as_debt_and_still_passes() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    write_in(&a, "shared/s.md", "lane-a writing");
    age_file(&a, "shared/s.md", 30);
    ok(
        &b,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-b",
            "--goal",
            "parallel work",
            "--base",
            "main",
            "--owns",
            "shared",
        ],
    );
    write_in(&b, "shared/s.md", "lane-b writing");

    let checked = agent_on(&b, &["worktree", "check"]);
    let text = combined(&checked);
    assert!(checked.status.success(), "{text}");
    assert!(!text.contains("OVERLAP"), "{text}");
    assert!(
        text.contains("RESCUE-DEBT"),
        "dormant work must stay visible: {text}"
    );
    assert!(text.contains("lane-a"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");
}

/// Two lanes writing the same path right now is the case the gate exists for.
/// Narrowing and dormancy must both leave it red.
#[test]
fn two_live_writers_on_one_path_still_fail() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    ok(
        &b,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-b",
            "--goal",
            "parallel work",
            "--base",
            "main",
            "--owns",
            "shared",
        ],
    );
    write_in(&a, "shared/s.md", "lane-a writing");
    write_in(&b, "shared/s.md", "lane-b writing");

    let checked = agent_on(&b, &["worktree", "check"]);
    let text = combined(&checked);
    assert!(!checked.status.success(), "{text}");
    assert!(text.contains("OVERLAP"), "{text}");
    assert!(text.contains("RESULT: FAIL"), "{text}");
}

/// A dormant lane's out-of-bounds changes are part of the same debt. They must
/// not keep the gate red either — nobody can fix them by editing the registry.
#[test]
fn dormant_out_of_bounds_is_debt_not_failure() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app"]);
    ok(
        &b,
        &[
            "worktree", "claim", "--id", "lane-b", "--goal", "own work", "--base", "main",
            "--owns", "docs",
        ],
    );
    write_in(&a, "shared/s.md", "outside its owns");
    age_file(&a, "shared/s.md", 30);

    let checked = agent_on(&a, &["worktree", "check"]);
    let text = combined(&checked);
    assert!(checked.status.success(), "{text}");
    assert!(text.contains("RESCUE-DEBT"), "{text}");
    assert!(text.contains("RESULT: PASS"), "{text}");
}

/// A live lane never goes dormant: `active` means a session says it is coming
/// back, and the contract holds ground it has not written yet.
#[test]
fn active_lane_never_goes_dormant() {
    let (_tmp, a, b) = fixture();
    ok(
        &a,
        &[
            "worktree",
            "claim",
            "--id",
            "lane-a",
            "--goal",
            "live work",
            "--base",
            "main",
            "--owns",
            "shared",
        ],
    );
    write_in(&a, "shared/s.md", "lane-a writing");
    age_file(&a, "shared/s.md", 30);

    let claimed = claim_b(&b, "shared");
    let text = combined(&claimed);
    assert!(
        !claimed.status.success(),
        "an active lane's contract does not expire: {text}"
    );
}

/// The window is a project knob, not a constant baked into the binary.
#[test]
fn dormancy_window_is_configurable() {
    let (_tmp, a, b) = fixture();
    landed_lane_holding(&a, &["app", "shared"]);
    write_in(&a, "shared/s.md", "lane-a writing");
    age_file(&a, "shared/s.md", 10);

    // Ten days is cold under the 7-day default...
    let claimed = claim_b(&b, "shared");
    assert!(claimed.status.success(), "{}", combined(&claimed));

    // ...and warm again once the project widens the window.
    let config = a.join("../repo/.git/agent-on/config.json");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, "{\"dormant_after_days\": 30}\n").unwrap();
    ok(&b, &["worktree", "edit", "--owns", "app/a.md"]);

    let edited = agent_on(&b, &["worktree", "edit", "--owns", "shared"]);
    let text = combined(&edited);
    assert!(
        !edited.status.success(),
        "a widened window must re-arm the gate: {text}"
    );
}
