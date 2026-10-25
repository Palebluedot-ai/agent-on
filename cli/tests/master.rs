//! Exercise the public delivery command with real Git and the real independent
//! auditor. Only the hosted GitHub boundary is replaced; no network or user repo.
#![cfg(unix)]
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Field {
    _tmp: TempDir,
    root: PathBuf,
    bin: PathBuf,
}
fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
impl Field {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        let bin = tmp.path().join("bin");
        fs::create_dir_all(root.join("tools/merge-audit")).unwrap();
        fs::create_dir_all(&bin).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.name", "Master test"]);
        git(&root, &["config", "user.email", "master@example.com"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&root, &["add", "README.md"]);
        git(&root, &["commit", "-m", "base"]);
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            source.join("tools/merge-audit/merge_audit.py"),
            root.join("tools/merge-audit/merge_audit.py"),
        )
        .unwrap();
        let mut policy: Value = serde_json::from_slice(
            &fs::read(source.join("tools/merge-audit/policy.json")).unwrap(),
        )
        .unwrap();
        policy["repo"] = "owner/repo".into();
        policy["trusted_authors"] = json!(["owner"]);
        policy["merge_health"]["required_checks"] = json!(["ci"]);
        fs::write(
            root.join("tools/merge-audit/policy.json"),
            policy.to_string(),
        )
        .unwrap();
        let gh = bin.join("gh");
        fs::write(&gh, GH).unwrap();
        fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
        let python = bin.join("python3");
        fs::write(
            &python,
            r#"#!/usr/bin/python3
import os, sys
from pathlib import Path
flag = Path(os.environ['MASTER_FIXTURE']) / 'fail-record-once'
if 'record' in sys.argv and flag.exists():
    flag.unlink()
    print('injected record interruption', file=sys.stderr)
    sys.exit(2)
os.execv(sys.executable, [sys.executable] + sys.argv[1:])
"#,
        )
        .unwrap();
        fs::set_permissions(&python, fs::Permissions::from_mode(0o755)).unwrap();
        let f = Self {
            _tmp: tmp,
            root,
            bin,
        };
        f.write_pr(1, json!({}));
        f
    }
    fn write_pr(&self, number: u64, update: Value) {
        let mut pr = json!({"number":number,"title":"deliver notes","body":"","author":{"login":"owner"},
            "files":[{"path":"README.md"}],"changedFiles":1,"labels":[],"headRefName":format!("feature-{number}"),
            "baseRefName":"main","headRefOid":format!("{number:040x}"),"isDraft":false,"state":"OPEN",
            "mergeable":"MERGEABLE","mergeStateStatus":"CLEAN","reviewDecision":"",
            "statusCheckRollup":[{"name":"ci","conclusion":"SUCCESS","status":"COMPLETED"}],
            "url":format!("https://github.com/owner/repo/pull/{number}"),"mergedAt":null,"mergeCommit":null});
        pr.as_object_mut()
            .unwrap()
            .extend(update.as_object().unwrap().clone());
        fs::write(self.root.join(format!("pr-{number}.json")), pr.to_string()).unwrap();
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(&self.root)
            .env("MASTER_FIXTURE", &self.root)
            .env(
                "PATH",
                format!("{}:{}", self.bin.display(), std::env::var("PATH").unwrap()),
            )
            .env_remove("CODEX_THREAD_ID")
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .env_remove("CLAUDE_PROJECT_DIR")
            .env_remove("CODEX_PROJECT_DIR")
            .env("AGENT_ON_HOST", "codex")
            .env("AGENT_ON_SESSION_ID", "master-test");
        cmd
    }
    fn run(&self, apply: bool) -> (Output, Value) {
        let mut cmd = self.command();
        cmd.args(["master", "--json"]);
        if apply {
            cmd.arg("--apply");
        }
        let out = cmd.output().unwrap();
        let data = serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
            panic!(
                "{} {}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out, data)
    }
    fn calls(&self) -> String {
        fs::read_to_string(self.root.join("calls.jsonl")).unwrap_or_default()
    }
    fn claims(&self) -> Vec<Value> {
        fs::read_to_string(self.root.join("ledger/merge-audit.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
}

#[test]
fn preview_and_apply_reuse_queue_and_do_not_need_valid_lane_registration() {
    let f = Field::new();
    fs::create_dir_all(f.root.join(".git/agent-on/lanes")).unwrap();
    fs::write(f.root.join(".git/agent-on/lanes/bad.json"), "{").unwrap();
    let (out, data) = f.run(false);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["items"][0]["state"], "READY");
    assert!(!f.calls().contains("\"merge\""));
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 1);
    assert_eq!(f.claims().len(), 1);
    let calls = f.calls();
    assert!(calls.contains("--match-head-commit"));
    assert!(
        !calls.contains("--admin")
            && !calls.contains("--delete-branch")
            && !calls.contains("--auto")
    );
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 0);
    assert_eq!(f.claims().len(), 1);
}

#[test]
fn a_waiting_pr_does_not_stop_another_ready_pr() {
    let f = Field::new();
    f.write_pr(
        1,
        json!({"statusCheckRollup":[{"name":"ci","status":"IN_PROGRESS"}]}),
    );
    f.write_pr(2, json!({}));
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 1);
    assert!(data["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["pr"] == 1 && r["state"] == "WAIT"));
    assert_eq!(f.claims()[0]["pr"], 2);
}

#[test]
fn head_changes_after_precheck_never_merge_the_unchecked_commit() {
    let f = Field::new();
    fs::write(f.root.join("race-head"), "").unwrap();
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 0);
    assert!(data["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["state"] == "WAIT"));
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn confirmed_merge_recovers_a_failed_record_without_merging_again() {
    let f = Field::new();
    fs::write(f.root.join("fail-record-once"), "").unwrap();
    let (out, _) = f.run(true);
    assert!(!out.status.success());
    assert!(f
        .root
        .join(".git/agent-on/landing/merge-pending.json")
        .exists());
    assert!(f.claims().is_empty());
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(f.claims().len(), 1);
    assert_eq!(
        f.calls()
            .lines()
            .filter(|s| s.contains("\"merge\""))
            .count(),
        1
    );
    assert!(!f
        .root
        .join(".git/agent-on/landing/merge-pending.json")
        .exists());
}

#[test]
fn hard_stop_remains_separate_from_ci_repair_and_waiting() {
    let f = Field::new();
    f.write_pr(1, json!({"files":[{"path":"hooks/guard.sh"}]}));
    f.write_pr(
        2,
        json!({"statusCheckRollup":[{"name":"ci","conclusion":"FAILURE"}]}),
    );
    f.write_pr(3, json!({"reviewDecision":"REVIEW_REQUIRED"}));
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    let rows = data["items"].as_array().unwrap();
    for (pr, state) in [(1, "HARD_STOP"), (2, "FIX"), (3, "WAIT")] {
        assert!(
            rows.iter().any(|r| r["pr"] == pr && r["state"] == state),
            "{data}"
        );
    }
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn another_oncall_owner_blocks_apply_but_leaves_preview_available() {
    let f = Field::new();
    let out = f
        .command()
        .env("AGENT_ON_SESSION_ID", "another-window")
        .args(["oncall", "claim", "--session", "other-owner"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let (out, data) = f.run(true);
    assert!(!out.status.success(), "{data}");
    assert!(data["error"].as_str().unwrap().contains("other-owner"));
    assert!(f.run(false).0.status.success());
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn behind_pr_updates_with_expected_head_and_waits_for_new_checks() {
    let f = Field::new();
    f.write_pr(
        1,
        json!({"mergeStateStatus":"BEHIND", "statusCheckRollup":[]}),
    );
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 0);
    assert!(f.calls().contains("expected_head_sha="));
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn a_check_turning_red_on_the_same_head_prevents_merge() {
    let f = Field::new();
    fs::write(f.root.join("flip-ci"), "").unwrap();
    let (out, data) = f.run(true);
    assert!(out.status.success(), "{data}");
    assert_eq!(data["merged"], 0);
    assert!(data["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["state"] == "FIX"));
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn repo_argument_does_not_replace_the_actual_oncall_workspace() {
    let f = Field::new();
    assert!(f
        .command()
        .args(["oncall", "claim", "--session", "master"])
        .output()
        .unwrap()
        .status
        .success());
    let other = f.root.parent().unwrap().join("other");
    git(
        &f.root,
        &["worktree", "add", "-b", "other", other.to_str().unwrap()],
    );
    let out = f
        .command()
        .current_dir(&other)
        .args([
            "master",
            "--apply",
            "--json",
            "--repo",
            f.root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(!f.calls().contains("\"merge\""));
}

#[test]
fn approval_is_committed_and_recorded_as_a_portable_project_path() {
    let f = Field::new();
    f.write_pr(1, json!({"files":[{"path":"hooks/guard.sh"}]}));
    let external = f.root.parent().unwrap().join("approval.md");
    fs::write(&external, "User approved this batch").unwrap();
    let invoke = |path: &Path| {
        f.command()
            .args([
                "master",
                "--apply",
                "--json",
                "--pr",
                "1",
                "--head",
                &format!("{:040x}", 1),
                "--approval",
                path.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    assert!(!invoke(&external).status.success());
    let path = f.root.join("approval.md");
    fs::write(&path, "User approved this batch").unwrap();
    assert!(!invoke(&path).status.success());
    assert!(!f.calls().contains("\"merge\""));
    git(&f.root, &["add", "approval.md"]);
    git(&f.root, &["commit", "-m", "record user approval"]);
    fs::write(
        &path,
        "User approved this batch\nUncommitted extension of approval",
    )
    .unwrap();
    assert!(!invoke(&path).status.success());
    assert!(!f.calls().contains("\"merge\""));
    fs::write(&path, "User approved this batch").unwrap();
    let out = invoke(&path);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(f.claims()[0]["pointer"], "approval.md");
}

#[test]
fn approved_deep_scan_evidence_survives_the_live_state_refresh() {
    let f = Field::new();
    let policy_path = f.root.join("tools/merge-audit/policy.json");
    let mut policy: Value = serde_json::from_slice(&fs::read(&policy_path).unwrap()).unwrap();
    policy["hard_stop_rules"][1]["diff_patterns"] = json!(["synthetic-sensitive-example"]);
    fs::write(policy_path, policy.to_string()).unwrap();
    fs::write(f.root.join("diff-example"), "synthetic-sensitive-example").unwrap();
    fs::write(
        f.root.join("approval.md"),
        "User approved the synthetic example",
    )
    .unwrap();
    git(&f.root, &["add", "approval.md"]);
    git(&f.root, &["commit", "-m", "record approval"]);
    let (_, preview) = f.run(false);
    assert_eq!(preview["items"][0]["state"], "HARD_STOP");
    let out = f
        .command()
        .args([
            "master",
            "--apply",
            "--json",
            "--pr",
            "1",
            "--head",
            &format!("{:040x}", 1),
            "--approval",
            "approval.md",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let data: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(data["merged"], 1, "{data}");
    assert_eq!(f.claims()[0]["claimed"], "HARD_STOP");
}

#[test]
fn an_unwritable_ledger_stops_before_the_remote_write() {
    let f = Field::new();
    fs::create_dir(f.root.join("ledger")).unwrap();
    let ledger = f.root.join("ledger/merge-audit.jsonl");
    fs::write(&ledger, "").unwrap();
    fs::set_permissions(&ledger, fs::Permissions::from_mode(0o444)).unwrap();
    let (out, data) = f.run(true);
    fs::set_permissions(&ledger, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(!out.status.success(), "{data}");
    assert!(!f.calls().contains("\"merge\""));
}

const GH: &str = r#"#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
root = Path(os.environ['MASTER_FIXTURE'])
args = sys.argv[1:]
with (root / 'calls.jsonl').open('a') as f: f.write(json.dumps(args) + '\n')
def emit(value): print(json.dumps(value))
def rows(): return [json.loads(p.read_text()) for p in sorted(root.glob('pr-*.json'))]
if args[:2] == ['repo', 'view']: emit({'nameWithOwner':'owner/repo'})
elif args[:2] == ['pr', 'list']:
    state = args[args.index('--state')+1]
    emit([p for p in rows() if p['state'].lower() == state or state == 'all'])
elif args[:2] == ['pr', 'view']:
    path = root / ('pr-' + args[2] + '.json')
    pr = json.loads(path.read_text())
    if (root / 'race-armed').exists() and 'files' not in args[-3:][0] and 'author' not in ' '.join(args):
        pr['headRefOid'] = 'b' * 40
        path.write_text(json.dumps(pr))
        (root / 'race-armed').unlink()
    emit(pr)
elif args[:2] == ['pr', 'diff']:
    if (root / 'race-head').exists(): (root / 'race-armed').touch()
    if (root / 'flip-ci').exists():
        path = root / ('pr-' + args[2] + '.json')
        pr = json.loads(path.read_text())
        pr['statusCheckRollup'] = [{'name':'ci','conclusion':'FAILURE','status':'COMPLETED'}]
        path.write_text(json.dumps(pr))
    example = (root / 'diff-example').read_text() if (root / 'diff-example').exists() else 'useful change'
    print('diff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n+' + example)
elif args[:2] == ['pr', 'merge']:
    path = root / ('pr-' + args[2] + '.json')
    pr = json.loads(path.read_text())
    assert args[args.index('--match-head-commit')+1] == pr['headRefOid']
    pr.update(state='MERGED', mergedAt='2026-10-25T12:00:00Z', mergeCommit={'oid':'c'*40})
    path.write_text(json.dumps(pr))
    print('merged')
elif args[0] == 'api' and 'update-branch' in ' '.join(args): emit({'message':'Updating pull request branch'})
else:
    print('Unhandled mock gh: '+repr(args), file=sys.stderr)
    sys.exit(2)
"#;
