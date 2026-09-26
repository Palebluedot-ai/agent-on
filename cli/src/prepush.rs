//! pre-push: a branch with an open PR takes base only through the server.
//!
//! Dartify PR #204 (2026-08-19): a session merged origin/main into its PR
//! branch locally 34 seconds before the server-side update-branch did the
//! same; the push bounced and the cleanup merge made a second fork. The rule
//! (playbook/multi-contributor-protocol.md §三½.8) sat on paper for five
//! weeks. This is its mechanical half.
//!
//! Blocks one shape only: a merge commit in the pushed range, committed
//! locally (committer is not GitHub), whose non-first parent is already on
//! `<remote>/<default>`, pushed to an existing non-default branch that has an
//! open same-repo PR into `<default>`. Exits stay open on purpose:
//! - a merge that conflicts: update-branch cannot resolve conflicts (GitHub
//!   answers 422), so a local merge is the only way through (case 40: a gate
//!   with no reachable exit is a deadlock);
//! - a merge whose cleanness git cannot judge (no `merge-tree --write-tree`
//!   before git 2.38): not blocked, a note says so;
//! - a branch with no such PR (none open, a fork's PR that shares the branch
//!   name, a PR into another base): there is no update-branch to race.

use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const GITHUB_COMMITTER: &str = "noreply@github.com";
const GH_TIMEOUT: Duration = Duration::from_secs(10);

/// One line of the pre-push hook's stdin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefUpdate {
    pub local_ref: String,
    pub local_sha: String,
    pub remote_ref: String,
    pub remote_sha: String,
}

pub fn parse_updates(text: &str) -> Vec<RefUpdate> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some(RefUpdate {
                local_ref: fields.next()?.to_string(),
                local_sha: fields.next()?.to_string(),
                remote_ref: fields.next()?.to_string(),
                remote_sha: fields.next()?.to_string(),
            })
        })
        .collect()
}

fn is_zero(sha: &str) -> bool {
    !sha.is_empty() && sha.bytes().all(|b| b == b'0')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubRepo {
    /// `None` is github.com. A GitHub Enterprise host has to be named to gh.
    pub host: Option<String>,
    pub owner: String,
    pub name: String,
}

impl GithubRepo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// gh's `-R` form: `[HOST/]OWNER/REPO`.
    pub fn repo_arg(&self) -> String {
        match &self.host {
            Some(host) => format!("{host}/{}", self.slug()),
            None => self.slug(),
        }
    }

    fn api(&self) -> String {
        match &self.host {
            Some(host) => format!("gh api --hostname {host}"),
            None => "gh api".to_string(),
        }
    }
}

/// `owner/repo` from a GitHub remote URL (scp-like, ssh://, https://). The host
/// only has to mention github: an ssh alias without a dot (`github-work`)
/// stands for github.com, a dotted one (`github.corp.com`) is Enterprise.
pub fn github_repo(url: &str) -> Option<GithubRepo> {
    let url = url.trim();
    let (host, path) = match url.split_once("://") {
        Some((_, rest)) => {
            let (authority, path) = rest.split_once('/')?;
            let host = authority.rsplit('@').next()?;
            (host.split(':').next()?, path)
        }
        None => {
            let (authority, path) = url.split_once(':')?;
            (authority.rsplit('@').next()?, path)
        }
    };
    let host = host.to_ascii_lowercase();
    if !host.contains("github") {
        return None;
    }
    let host = match host.as_str() {
        "github.com" | "www.github.com" | "ssh.github.com" => None,
        alias if !alias.contains('.') => None,
        _ => Some(host),
    };
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.rsplit('/').filter(|part| !part.is_empty());
    let name = parts.next()?.to_string();
    let owner = parts.next()?.to_string();
    Some(GithubRepo { host, owner, name })
}

/// What `gh` says about open PRs whose head is the pushed branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrLookup {
    Open(u64),
    NoneOpen,
    Unknown(String),
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn git_ok(repo: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub(crate) fn default_branch(repo: &Path, remote: &str) -> Option<String> {
    let resolves = |branch: &str| {
        git_ok(
            repo,
            &[
                "rev-parse",
                "-q",
                "--verify",
                &format!("refs/remotes/{remote}/{branch}"),
            ],
        )
    };
    // `<remote>/HEAD` can outlive its target: a clone from before a
    // master→main rename, fetched with prune, still points at master.
    if let Ok(head) = git(
        repo,
        &[
            "symbolic-ref",
            "--short",
            &format!("refs/remotes/{remote}/HEAD"),
        ],
    ) {
        if let Some(branch) = head.strip_prefix(&format!("{remote}/")) {
            if resolves(branch) {
                return Some(branch.to_string());
            }
        }
    }
    ["main", "master"]
        .into_iter()
        .find(|branch| resolves(branch))
        .map(str::to_string)
}

/// Whether the server could have made a merge itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeVerdict {
    Clean,
    Conflict,
    /// git could not say (older than 2.38, or an error). Never blocked on.
    Unknown,
}

/// `git merge-tree --write-tree` exits 0 when the merge is clean and 1 when it
/// conflicts. Before git 2.38 `--write-tree` does not exist and the exit code
/// is an error; reading that as "clean" would block the one exit a conflicted
/// PR has, so anything but 0 or 1 is Unknown.
fn merge_verdict(code: Option<i32>) -> MergeVerdict {
    match code {
        Some(0) => MergeVerdict::Clean,
        Some(1) => MergeVerdict::Conflict,
        _ => MergeVerdict::Unknown,
    }
}

fn merge_verdict_of(repo: &Path, ours: &str, theirs: &str) -> MergeVerdict {
    merge_verdict(
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["merge-tree", "--write-tree", ours, theirs])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .and_then(|status| status.code()),
    )
}

/// A local merge of the default branch in the pushed range, with whether the
/// server could have made it. Conflicted ones are dropped before this.
#[derive(Debug)]
struct LocalBaseMerge {
    sha: String,
    first_parent: String,
    subject: String,
    committer: String,
    verdict: MergeVerdict,
}

fn local_base_merges(
    repo: &Path,
    remote: &str,
    base_ref: &str,
    update: &RefUpdate,
) -> Result<Vec<LocalBaseMerge>, String> {
    let exclude_remotes = format!("--remotes={remote}");
    let mut args = vec![
        "log",
        "--merges",
        "--format=%H%x09%P%x09%ce%x09%cn <%ce>%x09%s",
        update.local_sha.as_str(),
        "--not",
        exclude_remotes.as_str(),
    ];
    if git_ok(
        repo,
        &[
            "cat-file",
            "-e",
            &format!("{}^{{commit}}", update.remote_sha),
        ],
    ) {
        args.push(update.remote_sha.as_str());
    }
    let mut found = Vec::new();
    for line in git(repo, &args)?.lines() {
        let fields: Vec<&str> = line.splitn(5, '\t').collect();
        let [sha, parents, email, committer, subject] = fields[..] else {
            continue;
        };
        if email.eq_ignore_ascii_case(GITHUB_COMMITTER) {
            continue;
        }
        let parents: Vec<&str> = parents.split_whitespace().collect();
        let Some((first, others)) = parents.split_first() else {
            continue;
        };
        let from_base = others
            .iter()
            .find(|parent| git_ok(repo, &["merge-base", "--is-ancestor", parent, base_ref]));
        let Some(from_base) = from_base else {
            continue;
        };
        let verdict = merge_verdict_of(repo, first, from_base);
        if verdict == MergeVerdict::Conflict {
            continue;
        }
        found.push(LocalBaseMerge {
            sha: sha.to_string(),
            first_parent: first.to_string(),
            subject: subject.to_string(),
            committer: committer.to_string(),
            verdict,
        });
    }
    Ok(found)
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

/// What the hook decided: a block message, and notes that never block.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    pub block: Option<String>,
    pub notes: Vec<String>,
}

/// `hook_args` are the hook's `$1 $2` (remote name, URL); scripts installed
/// before this check pass neither, which means origin. The owner/repo is read
/// from the configured URL, not the one `insteadOf` rewrote. `lookup` gets the
/// repo, the pushed branch, and the default branch a qualifying PR must target.
pub fn check(
    repo: &Path,
    hook_args: &[String],
    updates: &[RefUpdate],
    lookup: &dyn Fn(&GithubRepo, &str, &str) -> PrLookup,
) -> Verdict {
    let mut verdict = Verdict::default();
    let named = hook_args.first().filter(|name| {
        !name.is_empty() && git_ok(repo, &["config", "--get", &format!("remote.{name}.url")])
    });
    let remote = named.map(String::as_str).unwrap_or("origin");
    let Some(url) = git(repo, &["config", "--get", &format!("remote.{remote}.url")])
        .ok()
        .or_else(|| hook_args.get(1).filter(|url| !url.is_empty()).cloned())
    else {
        return verdict;
    };
    // No server-side update-branch off GitHub: nothing to race, no exit to give.
    let Some(gh_repo) = github_repo(&url) else {
        return verdict;
    };
    let Some(default) = default_branch(repo, remote) else {
        return verdict;
    };
    let base_ref = format!("refs/remotes/{remote}/{default}");
    let mut blocks = Vec::new();
    for update in updates {
        let Some(branch) = update.remote_ref.strip_prefix("refs/heads/") else {
            continue;
        };
        // Deleting a branch, the default branch itself, or a branch the server
        // has never seen (no PR can be open on it yet): not this rule.
        if is_zero(&update.local_sha) || branch == default || is_zero(&update.remote_sha) {
            continue;
        }
        let Ok(merges) = local_base_merges(repo, remote, &base_ref, update) else {
            continue;
        };
        let (clean, unknown): (Vec<LocalBaseMerge>, Vec<LocalBaseMerge>) = merges
            .into_iter()
            .partition(|merge| merge.verdict == MergeVerdict::Clean);
        if !unknown.is_empty() {
            verdict.notes.push(format!(
                "note: agent-on pre-push: {branch} carries a local merge of {remote}/{default} ({}) that this git cannot judge for conflicts (`git merge-tree --write-tree` needs git 2.38+); not blocked. If the branch has an open PR and the merge was clean, the server-side update-branch was the way (playbook/multi-contributor-protocol.md §三½.8).",
                unknown
                    .iter()
                    .map(|merge| short(&merge.sha))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if clean.is_empty() {
            continue;
        }
        let pr = match lookup(&gh_repo, branch, &default) {
            PrLookup::NoneOpen => continue,
            other => other,
        };
        let pushed = Pushed::of(repo, update, remote, branch);
        blocks.push(block_message(&gh_repo, &default, &pushed, &clean, &pr));
    }
    if !blocks.is_empty() {
        verdict.block = Some(blocks.join("\n"));
    }
    verdict
}

/// Where the pushed commits live locally, so the work order moves that branch
/// and not whatever happens to be checked out.
struct Pushed<'a> {
    remote: &'a str,
    branch: &'a str,
    local_sha: &'a str,
    /// `refs/heads/<name>` when a local branch was pushed.
    local_ref: Option<String>,
    checked_out: bool,
}

impl<'a> Pushed<'a> {
    fn of(repo: &Path, update: &'a RefUpdate, remote: &'a str, branch: &'a str) -> Self {
        let current = git(repo, &["symbolic-ref", "-q", "HEAD"]).ok();
        // `git push origin HEAD:x` reports its local ref as `HEAD`.
        let (local_ref, checked_out) = if update.local_ref == "HEAD" {
            (current, true)
        } else if update.local_ref.starts_with("refs/heads/") {
            let checked_out = current.as_deref() == Some(update.local_ref.as_str());
            (Some(update.local_ref.clone()), checked_out)
        } else {
            (None, false)
        };
        Pushed {
            remote,
            branch,
            local_sha: &update.local_sha,
            local_ref,
            checked_out,
        }
    }

    fn local_name(&self) -> Option<&str> {
        self.local_ref.as_deref()?.strip_prefix("refs/heads/")
    }
}

fn block_message(
    gh_repo: &GithubRepo,
    default: &str,
    pushed: &Pushed,
    merges: &[LocalBaseMerge],
    pr: &PrLookup,
) -> String {
    let (remote, branch) = (pushed.remote, pushed.branch);
    let base = format!("{remote}/{default}");
    let (headline, number, find_n) = match pr {
        PrLookup::Open(n) => (
            format!("{branch} has open PR #{n}, and this push merges {base} into it locally:"),
            n.to_string(),
            String::new(),
        ),
        _ => {
            let reason = match pr {
                PrLookup::Unknown(reason) => reason.as_str(),
                _ => "",
            };
            (
                format!(
                    "{branch} may have an open PR (lookup failed: {reason}), and this push merges {base} into it locally:"
                ),
                "<N>".to_string(),
                format!(
                    "   # N: gh pr view {branch} --json number -q .number -R {}",
                    gh_repo.repo_arg()
                ),
            )
        }
    };
    let mut lines = vec![headline];
    for merge in merges {
        lines.push(format!(
            "  {}  {}  (committer {})",
            short(&merge.sha),
            merge.subject,
            merge.committer
        ));
    }
    lines.push(format!(
        "A PR branch takes {base} only through the server: a local merge races the server-side update-branch. This merge is clean, so the server can make it (playbook/multi-contributor-protocol.md §三½.8)."
    ));
    lines.push("next:".to_string());
    let target = pushed.local_name().unwrap_or("the pushed branch");
    match merges {
        [only] if only.sha == pushed.local_sha && pushed.checked_out => lines.push(format!(
            "  git reset --keep {}   # drop the local merge; nothing of yours is in it",
            short(&only.first_parent)
        )),
        [only] if only.sha == pushed.local_sha && pushed.local_ref.is_some() => {
            lines.push(format!(
                "  git update-ref {} {} {}   # drop the local merge from {target}; nothing of yours is in it",
                pushed.local_ref.as_deref().unwrap_or_default(),
                short(&only.first_parent),
                short(&only.sha)
            ))
        }
        _ => lines.push(format!(
            "  take the local merge commit(s) above out of {target}, keeping your own commits"
        )),
    }
    let source = match pushed.local_name() {
        Some(name) if name == branch => branch.to_string(),
        Some(name) => format!("{name}:{branch}"),
        None => format!("HEAD:{branch}"),
    };
    lines.push(format!(
        "  git push {remote} {source}   # your own commits first, if any"
    ));
    lines.push(format!(
        "  {} -X PUT repos/{}/pulls/{number}/update-branch{find_n}",
        gh_repo.api(),
        gh_repo.slug()
    ));
    lines.push(match (pushed.checked_out, pushed.local_name()) {
        (true, _) => format!(
            "  git pull --ff-only {remote} {branch}   # once the server's merge commit is there"
        ),
        (false, Some(name)) => format!(
            "  git fetch {remote} {branch}:{name}   # once the server's merge commit is there"
        ),
        (false, None) => {
            format!("  git fetch {remote} {branch}   # once the server's merge commit is there")
        }
    });
    format!("{}\n", lines.join("\n"))
}

/// Run `cmd`, give up after `timeout`. Only stdout is kept.
fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Result<(ExitStatus, Vec<u8>), String> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot run gh: {e}"))?;
    let mut stdout = child.stdout.take().ok_or("no stdout")?;
    let reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok((status, reader.join().unwrap_or_default())),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("gh timed out after {}s", timeout.as_secs()));
            }
            Err(e) => return Err(format!("gh: {e}")),
        }
    }
}

/// The real lookup: `gh pr list --head <branch> --state open`.
pub fn gh_open_pr(gh_repo: &GithubRepo, branch: &str, default: &str) -> PrLookup {
    let mut cmd = Command::new("gh");
    cmd.args([
        "pr",
        "list",
        "-R",
        &gh_repo.repo_arg(),
        "--head",
        branch,
        "--state",
        "open",
        "--json",
        "number,isCrossRepository,baseRefName",
        "--limit",
        "20",
    ])
    .env("GH_PROMPT_DISABLED", "1")
    .env("GH_NO_UPDATE_NOTIFIER", "1");
    match run_with_timeout(cmd, GH_TIMEOUT) {
        Ok((status, stdout)) if status.success() => open_pr_from_json(&stdout, default),
        Ok(_) => PrLookup::Unknown("gh pr list failed".to_string()),
        Err(e) => PrLookup::Unknown(e),
    }
}

/// `--head` matches by branch name alone, so a fork's PR that happens to use
/// the same name comes back too; and update-branch merges a PR's own base. So
/// only a same-repo PR into the default branch counts. A field gh did not send
/// is read permissively; that errs toward blocking, never toward a false exit.
fn open_pr_from_json(stdout: &[u8], default: &str) -> PrLookup {
    match serde_json::from_slice::<Vec<Value>>(stdout) {
        Ok(prs) => prs
            .iter()
            .filter(|pr| {
                !pr.get("isCrossRepository")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .filter(|pr| {
                pr.get("baseRefName")
                    .and_then(Value::as_str)
                    .is_none_or(|base| base == default)
            })
            .find_map(|pr| pr.get("number").and_then(Value::as_u64))
            .map(PrLookup::Open)
            .unwrap_or(PrLookup::NoneOpen),
        Err(e) => PrLookup::Unknown(format!("gh output: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hook_stdin() {
        let updates = parse_updates(
            "refs/heads/feat/x abc123 refs/heads/feat/x def456\n(delete) 0000 refs/heads/old 1111\n\n",
        );
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].remote_ref, "refs/heads/feat/x");
        assert_eq!(updates[1].local_sha, "0000");
        assert!(is_zero(&updates[1].local_sha));
        assert!(!is_zero("0a00"));
        assert!(!is_zero(""));
    }

    #[test]
    fn reads_owner_and_repo_from_github_urls() {
        let want = Some(GithubRepo {
            host: None,
            owner: "acme".into(),
            name: "widget".into(),
        });
        for url in [
            "git@github.com:acme/widget.git",
            "git@github.com:acme/widget",
            "https://github.com/acme/widget.git",
            "https://github.com/acme/widget/",
            "https://user:token@github.com/acme/widget.git",
            "ssh://git@github.com:22/acme/widget.git",
            "git@github-work:acme/widget.git",
        ] {
            assert_eq!(github_repo(url), want, "{url}");
        }
    }

    fn sh(cwd: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// The block is a work order: what happened, why, then the commands in
    /// the order they have to run (§三½.4 "报错文案即工单").
    #[test]
    fn block_message_is_a_work_order_in_run_order() {
        let tmp = tempfile::TempDir::new().unwrap();
        let remote = tmp.path().join("remote.git");
        let work = tmp.path().join("work");
        std::fs::create_dir_all(&work).unwrap();
        sh(
            tmp.path(),
            &[
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                remote.to_str().unwrap(),
            ],
        );
        sh(&work, &["init", "-q", "-b", "main"]);
        sh(&work, &["config", "user.email", "me@example.com"]);
        sh(&work, &["config", "user.name", "Me"]);
        sh(
            &work,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        std::fs::write(work.join("a"), "a\n").unwrap();
        sh(&work, &["add", "."]);
        sh(&work, &["commit", "-qm", "init"]);
        sh(&work, &["push", "-q", "-u", "origin", "main"]);
        sh(&work, &["checkout", "-q", "-b", "fix/x"]);
        std::fs::write(work.join("f"), "f\n").unwrap();
        sh(&work, &["add", "."]);
        sh(&work, &["commit", "-qm", "feature"]);
        sh(&work, &["push", "-q", "-u", "origin", "fix/x"]);
        sh(&work, &["checkout", "-q", "main"]);
        std::fs::write(work.join("m"), "m\n").unwrap();
        sh(&work, &["add", "."]);
        sh(&work, &["commit", "-qm", "main moves"]);
        sh(&work, &["push", "-q", "origin", "main"]);
        sh(&work, &["checkout", "-q", "fix/x"]);
        let pre_merge = sh(&work, &["rev-parse", "HEAD"]);
        sh(&work, &["merge", "-q", "--no-edit", "origin/main"]);
        // Only the configured URL has to read as GitHub; nothing is pushed.
        sh(
            &work,
            &[
                "config",
                "remote.origin.url",
                "git@github.com:acme/widget.git",
            ],
        );
        let update = RefUpdate {
            local_ref: "refs/heads/fix/x".into(),
            local_sha: sh(&work, &["rev-parse", "HEAD"]),
            remote_ref: "refs/heads/fix/x".into(),
            remote_sha: sh(&work, &["rev-parse", "origin/fix/x"]),
        };

        let verdict = check(
            &work,
            &["origin".into()],
            &[update],
            &|repo, branch, default| {
                assert_eq!(
                    (repo.slug().as_str(), branch, default),
                    ("acme/widget", "fix/x", "main")
                );
                PrLookup::Open(42)
            },
        );
        assert!(verdict.notes.is_empty(), "{verdict:?}");
        let block = verdict
            .block
            .expect("a clean local merge into an open PR branch is blocked");
        let lines: Vec<&str> = block.lines().collect();
        assert_eq!(
            lines[0],
            "fix/x has open PR #42, and this push merges origin/main into it locally:"
        );
        assert!(lines[1].contains("Merge remote-tracking branch 'origin/main' into fix/x"));
        assert!(lines[1].contains("(committer Me <me@example.com>)"));
        assert!(lines[2].contains("§三½.8"));
        assert_eq!(lines[3], "next:");
        assert!(lines[4].starts_with(&format!("  git reset --keep {}", &pre_merge[..12])));
        assert!(lines[5].starts_with("  git push origin fix/x"));
        assert_eq!(
            lines[6],
            "  gh api -X PUT repos/acme/widget/pulls/42/update-branch"
        );
        assert!(lines[7].starts_with("  git pull --ff-only"));
        assert_eq!(lines.len(), 8, "{block}");

        // No open PR on the branch: nothing to race, not blocked.
        let update = parse_updates(&format!(
            "refs/heads/fix/x {} refs/heads/fix/x {}",
            sh(&work, &["rev-parse", "HEAD"]),
            sh(&work, &["rev-parse", "origin/fix/x"])
        ));
        assert_eq!(
            check(&work, &[], &update, &|_, _, _| PrLookup::NoneOpen),
            Verdict::default()
        );
    }

    #[test]
    fn a_github_enterprise_host_is_carried_to_gh() {
        let ghe = github_repo("git@github.corp.example:acme/widget.git").unwrap();
        assert_eq!(ghe.host.as_deref(), Some("github.corp.example"));
        assert_eq!(ghe.repo_arg(), "github.corp.example/acme/widget");
        assert_eq!(ghe.api(), "gh api --hostname github.corp.example");
        let dotcom = github_repo("https://github.com/acme/widget.git").unwrap();
        assert_eq!(dotcom.repo_arg(), "acme/widget");
        assert_eq!(dotcom.api(), "gh api");
    }

    #[test]
    fn only_exit_zero_is_clean_and_an_undecidable_merge_is_never_clean() {
        assert_eq!(merge_verdict(Some(0)), MergeVerdict::Clean);
        assert_eq!(merge_verdict(Some(1)), MergeVerdict::Conflict);
        // git < 2.38 does not know --write-tree; that is not an answer.
        assert_eq!(merge_verdict(Some(128)), MergeVerdict::Unknown);
        assert_eq!(merge_verdict(Some(129)), MergeVerdict::Unknown);
        assert_eq!(merge_verdict(None), MergeVerdict::Unknown);
    }

    #[test]
    fn only_a_same_repo_pr_into_the_default_branch_counts() {
        let json = |text: &str| open_pr_from_json(text.as_bytes(), "main");
        assert_eq!(json("[]"), PrLookup::NoneOpen);
        assert_eq!(
            json(r#"[{"number":9,"isCrossRepository":true,"baseRefName":"main"}]"#),
            PrLookup::NoneOpen
        );
        assert_eq!(
            json(r#"[{"number":8,"isCrossRepository":false,"baseRefName":"release/1"}]"#),
            PrLookup::NoneOpen
        );
        assert_eq!(
            json(
                r#"[{"number":9,"isCrossRepository":true,"baseRefName":"main"},
                    {"number":7,"isCrossRepository":false,"baseRefName":"main"}]"#
            ),
            PrLookup::Open(7)
        );
        assert_eq!(json(r#"[{"number":5}]"#), PrLookup::Open(5));
        assert!(matches!(json("not json"), PrLookup::Unknown(_)));
    }

    #[test]
    fn non_github_remotes_are_not_this_rules_business() {
        for url in [
            "/tmp/remote.git",
            "file:///tmp/remote.git",
            "git@gitlab.com:acme/widget.git",
            "https://example.com/acme/widget.git",
        ] {
            assert_eq!(github_repo(url), None, "{url}");
        }
    }
}
