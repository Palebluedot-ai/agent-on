//! One resumable delivery pass over the existing landing queue and audit ledger.
//! The host owns scheduling and conversation; this module never starts a model,
//! checks out another branch, deletes a branch, or bypasses GitHub requirements.
use crate::{coordination, landing, oncall, paths, session::Identity, worktree};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Args)]
pub struct MasterArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    /// Execute authorized ready merges; default only reports the queue.
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    json: bool,
    /// Limit this pass to one PR.
    #[arg(long)]
    pr: Option<u64>,
    /// Maximum merges per invocation; the host schedules subsequent passes.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=100))]
    limit: u32,
    /// Existing project policy; defaults to tools/merge-audit/policy.json.
    #[arg(long)]
    policy: Option<PathBuf>,
    /// Existing user approval evidence, scoped to --pr and --head.
    #[arg(long, requires_all = ["pr", "head"])]
    approval: Option<PathBuf>,
    #[arg(long, requires = "approval")]
    head: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pending {
    pr: u64,
    head: String,
    claimed: String,
    pointer: String,
    session: String,
}

struct Delivery {
    root: PathBuf,
    repo: String,
    policy: PathBuf,
    auditor: PathBuf,
    ledger: PathBuf,
    pending: PathBuf,
    method: String,
}

fn execute(command: &mut Command) -> Result<Output, String> {
    command
        .env("GH_PROMPT_DISABLED", "1")
        .output()
        .map_err(|e| e.to_string())
}

fn parsed(out: &Output) -> Result<Value, String> {
    serde_json::from_slice(&out.stdout).map_err(|_| {
        let error = String::from_utf8_lossy(&out.stderr);
        format!(
            "无法读取执行回执（exit {:?}）：{}",
            out.status.code(),
            error.trim()
        )
    })
}

impl Delivery {
    fn gh(&self, args: &[&str]) -> Result<Output, String> {
        execute(
            Command::new("gh")
                .current_dir(&self.root)
                .args(args)
                .args(["--repo", &self.repo]),
        )
    }

    fn audit(&self, args: &[&str]) -> Result<Output, String> {
        execute(
            Command::new("python3")
                .current_dir(&self.root)
                .arg(&self.auditor)
                .arg("--policy")
                .arg(&self.policy)
                .args(["--repo", &self.repo])
                .arg("--ledger")
                .arg(&self.ledger)
                .args(args),
        )
    }

    fn view(&self, pr: u64) -> Result<Value, String> {
        let out = self.gh(&[
            "pr",
            "view",
            &pr.to_string(),
            "--json",
            "number,state,headRefOid,mergeStateStatus,mergeable,isDraft,reviewDecision,mergedAt",
        ])?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().into());
        }
        parsed(&out)
    }

    fn record(&self, pending: &Pending) -> Result<(), String> {
        // A process may die after record succeeded but before the receipt was
        // removed. Repeating a pass must not create another claim.
        let raw = match fs::read_to_string(&self.ledger) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.to_string()),
        };
        let already = raw
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .any(|row| {
                row["kind"] == "claim" && row["action"] == "merged" && row["pr"] == pending.pr
            });
        if !already {
            let note = format!(
                "master: checked head {}; confirmed merged by GitHub",
                pending.head
            );
            let out = self.audit(&[
                "record",
                "--pr",
                &pending.pr.to_string(),
                "--action",
                "merged",
                "--claimed",
                &pending.claimed,
                "--pointer",
                &pending.pointer,
                "--session",
                &pending.session,
                "--note",
                &note,
            ])?;
            if !out.status.success() {
                return Err(format!(
                    "合入已确认，记账待重试：{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }
        fs::remove_file(&self.pending).map_err(|e| e.to_string())
    }

    fn recover(&self) -> Result<Option<Pending>, String> {
        if !self.pending.exists() {
            return Ok(None);
        }
        let pending: Pending =
            serde_json::from_slice(&fs::read(&self.pending).map_err(|e| e.to_string())?)
                .map_err(|e| format!("未完成合入回执不可读，保留原件：{e}"))?;
        let current = self.view(pending.pr)?;
        if current["state"] == "MERGED" {
            if current["headRefOid"] != pending.head {
                return Err(format!(
                    "#{} 已合入，但 head 与上次检查不符，需核对真实合入",
                    pending.pr
                ));
            }
            self.record(&pending)?;
            return Ok(None);
        }
        if current["state"] == "CLOSED" || current["headRefOid"] != pending.head {
            // No merge of this candidate happened; new code needs a fresh pass.
            fs::remove_file(&self.pending).map_err(|e| e.to_string())?;
            return Ok(None);
        }
        Ok(Some(pending))
    }
}

fn authority(root: &Path) -> Result<(), String> {
    oncall::load(root)?;
    let actor = std::env::var_os("CLAUDE_PROJECT_DIR")
        .or_else(|| std::env::var_os("CODEX_PROJECT_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let identity = Identity::current();
    match oncall::role_at(root) {
        oncall::Role::Feature(owner) | oncall::Role::Oncall(owner)
            if !oncall::matches_owner(&actor, &owner, identity.as_ref()) =>
        {
            Err(format!(
                "合入由在班值守 {} 处理；本窗口可运行 agent-on master 查看队列",
                owner.session
            ))
        }
        oncall::Role::Oncall(_) | oncall::Role::Feature(_) => {
            let (code, message) = oncall::heartbeat(root);
            if code == 0 {
                Ok(())
            } else {
                Err(message)
            }
        }
        oncall::Role::Nobody => Ok(()),
    }
}

fn approval_pointer(root: &Path, path: &Path) -> Result<String, String> {
    let absolute = fs::canonicalize(if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    })
    .map_err(|_| "用户批准证据文件不存在")?;
    let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let relative = absolute
        .strip_prefix(canonical_root)
        .map_err(|_| "批准证据须在当前仓内，不能使用临时目录中的文件")?;
    let pointer = relative
        .to_str()
        .ok_or("批准证据路径不是 UTF-8")?
        .replace('\\', "/");
    let recorded = execute(
        Command::new("git")
            .current_dir(root)
            .args(["show", &format!("HEAD:{pointer}")]),
    )?;
    if !recorded.status.success()
        || recorded.stdout != fs::read(&absolute).map_err(|e| e.to_string())?
    {
        return Err(
            "批准记录当前内容尚未进入 Git；先随授权批次提交，不能只凭一个临时或已改写的文件".into(),
        );
    }
    Ok(pointer)
}

fn setup(args: &MasterArgs) -> Result<Delivery, String> {
    let cwd = args
        .repo
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let root = worktree::repo_root(&cwd)?;
    let policy = args
        .policy
        .clone()
        .unwrap_or_else(|| root.join("tools/merge-audit/policy.json"));
    let policy = if policy.is_absolute() {
        policy
    } else {
        root.join(policy)
    };
    let settings: Value = serde_json::from_slice(&fs::read(&policy).map_err(|e| {
        format!(
            "项目合入政策不可读：{}：{e}；按 kit/babysit/SETUP.md 复用项目已有授权",
            policy.display()
        )
    })?)
    .map_err(|e| e.to_string())?;
    let repo = settings["repo"]
        .as_str()
        .ok_or("policy 缺 repo")?
        .to_string();
    let out = execute(Command::new("gh").current_dir(&root).args([
        "repo",
        "view",
        "--json",
        "nameWithOwner",
    ]))?;
    if !out.status.success() || parsed(&out)?["nameWithOwner"] != repo {
        return Err("政策里的仓库与当前 GitHub 仓库不同，不能把别的项目的授权用于此处".into());
    }
    let method = settings["merge_method"]
        .as_str()
        .unwrap_or("squash")
        .to_string();
    if !["squash", "merge", "rebase"].contains(&method.as_str()) {
        return Err("merge_method 只支持 squash / merge / rebase".into());
    }
    let local = root.join("tools/merge-audit/merge_audit.py");
    let auditor = if local.is_file() {
        local
    } else {
        paths::resolve_read_root(Some(&root))
            .0
            .ok_or("找不到独立合并审计工具")?
            .join("tools/merge-audit/merge_audit.py")
    };
    let common = worktree::common_git_dir(&root)?;
    let trees =
        worktree::parse_worktrees(&worktree::git(&root, &["worktree", "list", "--porcelain"])?);
    let primary = trees.first().ok_or("找不到主 worktree")?;
    Ok(Delivery {
        root,
        repo,
        policy,
        auditor,
        method,
        ledger: primary.path.join("ledger/merge-audit.jsonl"),
        pending: common.join("agent-on/landing/merge-pending.json"),
    })
}

fn pass(args: &MasterArgs) -> Result<Value, String> {
    let d = setup(args)?;
    let common = worktree::common_git_dir(&d.root)?;
    let _lock = if args.apply {
        authority(&d.root)?;
        Some(coordination::lock_path(
            &common.join("agent-on/landing/master.lock"),
        )?)
    } else {
        None
    };
    let mut pending = if args.apply { d.recover()? } else { None };
    if args.apply {
        let report = d.audit(&["report", "--json"])?;
        if !report.status.success() {
            return Err(format!(
                "独立审计未通过，暂不追加合入；报告：{}",
                String::from_utf8_lossy(&report.stdout)
            ));
        }
        let parent = d.ledger.parent().ok_or("账本缺父目录")?;
        fs::create_dir_all(parent).map_err(|e| format!("账本目录不可写，尚未合入：{e}"))?;
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&d.ledger)
            .map_err(|e| format!("账本不可追加，尚未合入：{e}"))?;
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let mut merged = 0;
    loop {
        let (code, output) = landing::run_refresh(
            &d.root,
            &landing::RealGh,
            &landing::LandingOpts {
                json: true,
                ..Default::default()
            },
        );
        if code != 0 {
            return Err(output);
        }
        let data: Value = serde_json::from_str(&output).map_err(|e| e.to_string())?;
        let mut tracks: Vec<landing::TrackState> =
            serde_json::from_value(data["snapshot"]["tracks"].clone())
                .map_err(|e| e.to_string())?;
        tracks.sort_by_key(|t| t.pr_number);
        let mut advanced = false;
        for track in tracks {
            let Some(pr) = track
                .pr_number
                .filter(|_| track.pr_state.as_deref() == Some("open"))
            else {
                continue;
            };
            if args.pr.is_some_and(|n| n != pr) || !seen.insert(pr) {
                continue;
            }
            if !track.deps_open_prs.is_empty() || track.category == Some(landing::Category::Next) {
                rows.push(json!({"pr":pr,"state":"WAIT","reason":track.reason}));
                continue;
            }
            if pending.as_ref().is_some_and(|p| p.pr != pr) {
                rows.push(json!({"pr":pr,"state":"WAIT","reason":"上次合入尚未确认"}));
                continue;
            }
            let out = d.audit(&["precheck", "--pr", &pr.to_string(), "--json"])?;
            if !matches!(out.status.code(), Some(0 | 10 | 20 | 30 | 40)) {
                return Err(format!(
                    "#{} 无法完成合入检查：{}",
                    pr,
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            let verdict = parsed(&out)?;
            let state = verdict["execution"]["state"]
                .as_str()
                .ok_or("审计工具版本过旧，缺执行状态；更新项目的 merge-audit 副本")?;
            let head = verdict["head_sha"]
                .as_str()
                .filter(|s| s.len() >= 40)
                .unwrap_or("");
            let mut pointer = String::new();
            let decision = verdict["decision"].as_str().ok_or("缺政策结论")?;
            if decision == "HARD_STOP" {
                if args.pr == Some(pr) && args.head.as_deref() == Some(head) {
                    let path = args.approval.as_ref().ok_or("缺用户批准证据")?;
                    pointer = approval_pointer(&d.root, path)?;
                } else {
                    rows.push(json!({"pr":pr,"state":if state == "READY" {"HARD_STOP"} else {state},
                        "reason":if state == "READY" {&verdict["hard_stop"]} else {&verdict["execution"]["reasons"]},
                        "policy":verdict["hard_stop"],"head":head}));
                    continue;
                }
            }
            // Updating a lagging PR may be necessary for its required workflow to exist.
            if args.apply
                && !head.is_empty()
                && verdict["merge_state"] == "BEHIND"
                && verdict["paused"] != true
                && matches!(state, "READY" | "WAIT")
            {
                authority(&d.root)?;
                let out = execute(Command::new("gh").current_dir(&d.root).args([
                    "api",
                    "--method",
                    "PUT",
                    &format!("repos/{}/pulls/{pr}/update-branch", d.repo),
                    "-f",
                    &format!("expected_head_sha={head}"),
                ]))?;
                rows.push(
                    json!({"pr":pr,"state":"WAIT","reason":if out.status.success() {
                    "已追平目标分支；等待新提交的相关检查".to_string()
                } else {String::from_utf8_lossy(&out.stderr).trim().to_string()}}),
                );
                continue;
            }
            if state != "READY" || head.is_empty() {
                rows.push(json!({"pr":pr,"state":state,"reason":verdict["execution"]["reasons"]}));
                continue;
            }
            if !args.apply {
                rows.push(json!({"pr":pr,"state":"READY","decision":decision,"head":head}));
                continue;
            }
            authority(&d.root)?;
            let current = d.view(pr)?;
            if current["headRefOid"] != head || current["state"] != "OPEN" {
                rows.push(
                    json!({"pr":pr,"state":"WAIT","reason":"候选提交或 PR 状态已变化，下轮重读"}),
                );
                continue;
            }
            if current["mergeable"] != "MERGEABLE"
                || current["isDraft"] == true
                || matches!(
                    current["reviewDecision"].as_str(),
                    Some("REVIEW_REQUIRED" | "CHANGES_REQUESTED")
                )
            {
                rows.push(
                    json!({"pr":pr,"state":"WAIT","reason":"合入条件刚发生变化，下轮重新分诊"}),
                );
                continue;
            }
            // Same SHA does not freeze CI or reviews. Refresh only live facts;
            // the unchanged head retains the earlier deep diff scan.
            let fresh = d.audit(&["precheck", "--pr", &pr.to_string(), "--json", "--no-deep"])?;
            let live = parsed(&fresh)?;
            let previous_rules = verdict["hard_stop"].as_array().ok_or("缺首次政策证据")?;
            let live_rules = live["hard_stop"].as_array().ok_or("缺最新政策证据")?;
            let new_policy_evidence = live_rules.iter().any(|rule| {
                !previous_rules.iter().any(|previous| {
                    previous["id"] == rule["id"]
                        && rule["evidence"].as_array().is_some_and(|evidence| {
                            previous["evidence"]
                                .as_array()
                                .is_some_and(|old| evidence.iter().all(|item| old.contains(item)))
                        })
                })
            });
            if !matches!(fresh.status.code(), Some(0 | 10 | 20))
                || live["head_sha"] != head
                || live["execution"]["state"] != "READY"
                || new_policy_evidence
                || live["merge_state"] == "BEHIND"
            {
                rows.push(json!({"pr":pr,"state":if live["execution"]["state"] == "FIX" {"FIX"} else {"WAIT"},
                    "reason":if new_policy_evidence {json!(["政策证据已变化，下轮重新判定批准范围"])}
                    else if live["execution"]["reasons"].as_array().is_some_and(|r| !r.is_empty()) {live["execution"]["reasons"].clone()}
                    else {json!(["候选提交或合入条件已变化，下轮重读"])} }));
                continue;
            }
            let receipt = Pending {
                pr,
                head: head.into(),
                claimed: decision.into(),
                pointer,
                session: Identity::current()
                    .map(|i| format!("{}:{}", i.host, i.session_id))
                    .unwrap_or_else(|| "maintainer".into()),
            };
            coordination::write_json(&d.pending, &receipt)?;
            let out = d.gh(&[
                "pr",
                "merge",
                &pr.to_string(),
                &format!("--{}", d.method),
                "--match-head-commit",
                head,
            ])?;
            let confirmation = d.view(pr)?;
            if confirmation["state"] == "MERGED" && confirmation["headRefOid"] == head {
                d.record(&receipt)?;
                pending = None;
                rows.push(json!({"pr":pr,"state":"MERGED","head":head,"decision":decision}));
                merged += 1;
                advanced = true;
                break; // Base changed; refresh dependencies before selecting the next PR.
            }
            rows.push(
                json!({"pr":pr,"state":"WAIT","reason":if out.status.success() {
                "GitHub 尚未确认合入；保留回执，下轮续接".to_string()
            } else { String::from_utf8_lossy(&out.stderr).trim().to_string() }}),
            );
            return Ok(json!({"apply":args.apply,"merged":merged,"items":rows}));
        }
        if !advanced || merged >= args.limit {
            break;
        }
        // Waiting dependencies may now be ready; don't reprocess unrelated waits.
        seen.retain(|pr| {
            rows.iter()
                .any(|r| r["pr"] == *pr && r["state"] == "MERGED")
        });
    }
    Ok(json!({"apply":args.apply,"merged":merged,"items":rows}))
}

pub fn run(args: MasterArgs) -> (i32, String) {
    match pass(&args) {
        Ok(value) if args.json => (0, format!("{value}\n")),
        Ok(value) => {
            let mut out = format!(
                "交付统筹：本轮合入 {} 项{}\n",
                value["merged"],
                if args.apply { "" } else { "（预览）" }
            );
            for row in value["items"].as_array().into_iter().flatten() {
                out.push_str(&format!(
                    "#{} {} {}\n",
                    row["pr"],
                    row["state"].as_str().unwrap_or("UNKNOWN"),
                    row.get("reason").map(Value::to_string).unwrap_or_default()
                ));
            }
            (0, out)
        }
        Err(error) => (
            1,
            if args.json {
                format!("{}\n", json!({"state":"UNAVAILABLE","error":error}))
            } else {
                format!("交付统筹暂未完成：{error}\n")
            },
        ),
    }
}
