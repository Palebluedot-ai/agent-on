//! Every gate refusal must carry an exit the blocked party can actually reach.
//!
//! Source: 2026-08-20 field report. A session was refused with `overlaps
//! still-writing lane <id>` and stopped to ask its human "who is supposed to
//! run this?" — while the answer was written in `kit/worktree-control-plane.md`
//! the whole time. A refusal that names the blocker but not the exit turns
//! every agent into a human-powered permanently-red gate, which is the exact
//! failure the boundary-gate liveness work set out to remove.
//!
//! The two exits are asymmetric on purpose, so that reading one does not teach
//! the other: a live contract must NOT be edited by the party it blocks, a
//! stale registration MUST be repairable by it.

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

/// Drive lane-a to `landed` and redivide the reused worktree onto `shared`,
/// i.e. the stale registration a returning session leaves behind.
fn landed_lane_redivided_onto_shared(a: &Path) {
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

/// A stale registration is metadata. The session it blocks may repair it, and
/// the refusal has to say so — with the command, and with the fact that this is
/// repair rather than deletion.
#[test]
fn still_writing_refusal_names_the_repair_the_blocked_lane_can_run() {
    let (_tmp, a, b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    write_in(&a, "shared/s.md", "lane-a writing");

    let claimed = agent_on(
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
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("still-writing lane lane-a"), "{text}");
    // Who: the party reading this refusal, not some absent owner.
    assert!(text.contains("exit:"), "no exit offered: {text}");
    // What: the exact non-destructive command, aimed at the blocking lane.
    assert!(
        text.contains("agent-on worktree edit --id lane-a --base"),
        "exit does not name the repair command: {text}"
    );
    // Which mistake not to make: deleting the boundary is the wrong operation.
    assert!(
        text.contains("not deletion"),
        "exit does not fence off the destructive lookalike: {text}"
    );
}

/// The mirror image. A live contract has somebody at home; the party it blocks
/// must be told to stay out, so that reading the repair exit above cannot be
/// generalised into "I may edit anyone's lane".
#[test]
fn live_contract_refusal_tells_the_blocked_lane_to_stay_out() {
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

    let claimed = agent_on(
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
    let text = combined(&claimed);
    assert!(!claimed.status.success(), "{text}");
    assert!(text.contains("live lane lane-a"), "{text}");
    assert!(text.contains("exit:"), "no exit offered: {text}");
    assert!(
        text.contains("do not edit its registration"),
        "live contract exit must fence off registration edits: {text}"
    );
    // The blocked party's own move, in its own hands.
    assert!(
        text.contains("narrow your own"),
        "live contract exit must name a move the blocked lane can make: {text}"
    );
}

/// `STATUS-DRIFT` is a warning nobody owns unless the line says who may clear
/// it. It also has to carry the re-pin exit: `--status active` widens the
/// boundary to full `owns`, which is the wrong direction for a lane whose work
/// already landed under a squashed commit.
#[test]
fn status_drift_line_names_its_owner_and_the_re_pin_exit() {
    let (_tmp, a, _b) = fixture();
    landed_lane_redivided_onto_shared(&a);
    write_in(&a, "shared/s.md", "lane-a writing");

    let status = ok(&a, &["worktree", "status"]);
    assert!(status.contains("STATUS-DRIFT"), "{status}");
    assert!(
        status.contains("any lane this blocks may repair"),
        "drift line does not name who may clear it: {status}"
    );
    assert!(
        status.contains("--base"),
        "drift line does not offer the re-pin exit: {status}"
    );
    assert!(
        status.contains("not deletion"),
        "drift line does not fence off the destructive lookalike: {status}"
    );
}
