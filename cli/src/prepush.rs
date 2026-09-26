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
//! open PR. Two exits stay open on purpose:
//! - a merge that conflicts: update-branch cannot resolve conflicts (GitHub
//!   answers 422), so a local merge is the only way through (case 40: a gate
//!   with no reachable exit is a deadlock);
//! - a branch with no open PR: there is no server-side update-branch to race.

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
    pub owner: String,
    pub name: String,
}

impl GithubRepo {
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// `owner/repo` from a GitHub remote URL (scp-like, ssh://, https://). The host
/// only has to mention github, so ssh aliases like `github-work` count.
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
    if !host.to_ascii_lowercase().contains("github") {
        return None;
    }
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.rsplit('/').filter(|part| !part.is_empty());
    let name = parts.next()?.to_string();
    let owner = parts.next()?.to_string();
    Some(GithubRepo { owner, name })
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

fn default_branch(repo: &Path, remote: &str) -> Option<String> {
    if let Ok(head) = git(
        repo,
        &[
            "symbolic-ref",
            "--short",
            &format!("refs/remotes/{remote}/HEAD"),
        ],
    ) {
        if let Some(branch) = head.strip_prefix(&format!("{remote}/")) {
            return Some(branch.to_string());
        }
    }
    ["main", "master"]
        .into_iter()
        .map(str::to_string)
        .find(|branch| {
            git_ok(
                repo,
                &[
                    "rev-parse",
                    "-q",
                    "--verify",
                    &format!("refs/remotes/{remote}/{branch}"),
                ],
            )
        })
}

/// A merge the server could have made itself: in the pushed range, committed
/// locally, a non-first parent already on the default branch, and clean.
#[derive(Debug)]
struct LocalBaseMerge {
    sha: String,
    first_parent: String,
    subject: String,
    committer: String,
}

/// `git merge-tree --write-tree` (git 2.38+) exits 1 on conflicts. Anything
/// else it cannot answer counts as clean: the gate stays shut and
/// `--no-verify` remains. No `--quiet`: it is newer than `--write-tree`, and a
/// usage error must not be read as "clean" on an older git.
fn merges_cleanly(repo: &Path, ours: &str, theirs: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-tree", "--write-tree", ours, theirs])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.code() != Some(1))
        .unwrap_or(true)
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
        if !merges_cleanly(repo, first, from_base) {
            continue;
        }
        found.push(LocalBaseMerge {
            sha: sha.to_string(),
            first_parent: first.to_string(),
            subject: subject.to_string(),
            committer: committer.to_string(),
        });
    }
    Ok(found)
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

/// `remote` and `url` come from the hook's `$1 $2`; scripts installed before
/// this check pass neither, which means origin. The owner/repo is read from
/// the configured URL, not the one `insteadOf` rewrote.
pub fn check(
    repo: &Path,
    hook_args: &[String],
    updates: &[RefUpdate],
    lookup: &dyn Fn(&GithubRepo, &str) -> PrLookup,
) -> Option<String> {
    let named = hook_args
        .first()
        .filter(|name| git_ok(repo, &["config", "--get", &format!("remote.{name}.url")]));
    let remote = named.map(String::as_str).unwrap_or("origin");
    let url = git(repo, &["config", "--get", &format!("remote.{remote}.url")])
        .ok()
        .or_else(|| hook_args.get(1).cloned())?;
    // No server-side update-branch off GitHub: nothing to race, no exit to give.
    let gh_repo = github_repo(&url)?;
    let default = default_branch(repo, remote)?;
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
        let merges = match local_base_merges(repo, remote, &base_ref, update) {
            Ok(merges) if merges.is_empty() => continue,
            Ok(merges) => merges,
            Err(_) => continue,
        };
        let pr = match lookup(&gh_repo, branch) {
            PrLookup::NoneOpen => continue,
            other => other,
        };
        blocks.push(block_message(
            &gh_repo, remote, &default, branch, update, &merges, &pr,
        ));
    }
    (!blocks.is_empty()).then(|| blocks.join("\n"))
}

fn block_message(
    gh_repo: &GithubRepo,
    remote: &str,
    default: &str,
    branch: &str,
    update: &RefUpdate,
    merges: &[LocalBaseMerge],
    pr: &PrLookup,
) -> String {
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
                format!("   # N: gh pr view {branch} --json number -q .number"),
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
    match merges {
        [only] if only.sha == update.local_sha => lines.push(format!(
            "  git reset --keep {}   # drop the local merge; nothing of yours is in it",
            short(&only.first_parent)
        )),
        _ => lines.push(
            "  take the local merge commit(s) above out of the branch, keeping your own commits"
                .to_string(),
        ),
    }
    lines.push(format!(
        "  git push {remote} {branch}   # your own commits first, if any"
    ));
    lines.push(format!(
        "  gh api -X PUT repos/{}/pulls/{number}/update-branch{find_n}",
        gh_repo.slug()
    ));
    lines.push("  git pull --ff-only   # once the server's merge commit is there".to_string());
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
pub fn gh_open_pr(gh_repo: &GithubRepo, branch: &str) -> PrLookup {
    let mut cmd = Command::new("gh");
    cmd.args([
        "pr",
        "list",
        "-R",
        &gh_repo.slug(),
        "--head",
        branch,
        "--state",
        "open",
        "--json",
        "number",
        "--limit",
        "5",
    ])
    .env("GH_PROMPT_DISABLED", "1")
    .env("GH_NO_UPDATE_NOTIFIER", "1");
    match run_with_timeout(cmd, GH_TIMEOUT) {
        Ok((status, stdout)) if status.success() => {
            match serde_json::from_slice::<Vec<Value>>(&stdout) {
                Ok(prs) => prs
                    .iter()
                    .find_map(|pr| pr.get("number").and_then(Value::as_u64))
                    .map(PrLookup::Open)
                    .unwrap_or(PrLookup::NoneOpen),
                Err(e) => PrLookup::Unknown(format!("gh output: {e}")),
            }
        }
        Ok(_) => PrLookup::Unknown("gh pr list failed".to_string()),
        Err(e) => PrLookup::Unknown(e),
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

        let block = check(&work, &["origin".into()], &[update], &|repo, branch| {
            assert_eq!((repo.slug().as_str(), branch), ("acme/widget", "fix/x"));
            PrLookup::Open(42)
        })
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
        assert_eq!(check(&work, &[], &update, &|_, _| PrLookup::NoneOpen), None);
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
