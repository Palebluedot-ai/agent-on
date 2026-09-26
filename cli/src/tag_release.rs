//! Annotated tag release helper. Git ops via external `git`.

use regex::Regex;
use std::path::Path;
use std::process::Command;

fn run_git(repo: &Path, args: &[&str], check: bool) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git spawn: {e}"))?;
    if check && !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn parse_ver(tag: &str) -> Result<(u32, u32, u32), String> {
    let re = Regex::new(r"^v(\d+)\.(\d+)\.(\d+)$").unwrap();
    let c = re
        .captures(tag)
        .ok_or_else(|| format!("无法解析 tag: {tag}"))?;
    Ok((
        c[1].parse().unwrap(),
        c[2].parse().unwrap(),
        c[3].parse().unwrap(),
    ))
}

pub fn bump(level: &str, major: u32, minor: u32, patch: u32) -> Result<String, String> {
    match level {
        "major" => Ok(format!("v{}.0.0", major + 1)),
        "minor" => Ok(format!("v{}.{}.0", major, minor + 1)),
        "patch" => Ok(format!("v{}.{}.{}", major, minor, patch + 1)),
        _ => Err("--level 须为 major|minor|patch".into()),
    }
}

pub fn latest_tag(repo: &Path) -> Result<String, String> {
    let out = run_git(repo, &["tag", "--sort=-v:refname"], false)?;
    let re = Regex::new(r"^v\d+\.\d+\.\d+$").unwrap();
    for line in out.lines() {
        let t = line.trim();
        if re.is_match(t) {
            return Ok(t.to_string());
        }
    }
    Err("找不到 vX.Y.Z 形态 tag".into())
}

/// Intake marks `landed@vX.Y.Z` whose tag does not exist yet. A mark may only
/// name a commit or tag that already carries the landing (Dartify 2026-09-26:
/// cards marked `landed@v0.22.0` before the tag existed; another commit then
/// took the number). Returns `file:line  mark` rows.
pub fn unminted_tag_marks(repo: &Path) -> Vec<String> {
    let re = Regex::new(r"landed@(v\d+\.\d+\.\d+)").unwrap();
    let mut rows = Vec::new();
    for path in crate::intake_lint::default_intake_paths(repo) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for cap in re.captures_iter(line) {
                let tag = &cap[1];
                let exists = run_git(
                    repo,
                    &["rev-parse", "-q", "--verify", &format!("refs/tags/{tag}")],
                    true,
                )
                .is_ok();
                if !exists {
                    rows.push(format!(
                        "{}:{}  landed@{tag}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        i + 1
                    ));
                }
            }
        }
    }
    rows
}

pub struct TagOpts {
    pub level: String,
    pub title: String,
    pub push: bool,
    pub allow_dirty: bool,
}

/// Returns exit code and message. Creates annotated tag on repo HEAD when appropriate.
pub fn run_tag_release(repo: &Path, opts: &TagOpts) -> (i32, String) {
    let status = match run_git(repo, &["status", "--porcelain"], false) {
        Ok(s) => s,
        Err(e) => return (1, e),
    };
    if !status.is_empty() && !opts.allow_dirty {
        return (
            1,
            format!("工作区不干净,先 commit 再发版(或 --allow-dirty):\n{status}\n"),
        );
    }

    let tag = match latest_tag(repo) {
        Ok(t) => t,
        Err(e) => return (1, format!("{e}\n")),
    };
    let ahead = match run_git(
        repo,
        &["rev-list", "--count", &format!("{tag}..HEAD")],
        true,
    ) {
        Ok(a) => a,
        Err(e) => return (1, e),
    };
    if ahead == "0" {
        return (2, format!("HEAD 已与 {tag} 齐平,无需发版\n"));
    }

    let (maj, mino, pat) = match parse_ver(&tag) {
        Ok(v) => v,
        Err(e) => return (1, format!("{e}\n")),
    };
    let new_tag = match bump(&opts.level, maj, mino, pat) {
        Ok(t) => t,
        Err(e) => return (1, format!("{e}\n")),
    };
    let premarked = unminted_tag_marks(repo);
    if !premarked.is_empty() {
        return (
            1,
            format!(
                "intake 里有去向标注指向还不存在的 tag,拒绝打 {new_tag}:\n  {}\n去向只能引用已经存在、并且包含落点的 commit 或 tag;写不出 hash 就写 `landed@同批（落点）`。\n",
                premarked.join("\n  ")
            ),
        );
    }
    let head = run_git(repo, &["rev-parse", "--short", "HEAD"], true).unwrap_or_default();

    let mut msg_extra = String::new();
    if opts.level == "major" {
        msg_extra
            .push_str("WARNING: major 必须在 CHANGELOG 写清迁移注记;无注记禁止打 tag(人工自检)\n");
    }

    let msg = format!(
        "{new_tag} — {}\n\n{}\n\n基于 {tag} + {ahead} commit (HEAD {head})",
        opts.level, opts.title
    );
    if let Err(e) = run_git(repo, &["tag", "-a", &new_tag, "-m", &msg], true) {
        return (1, e);
    }
    let full = run_git(repo, &["rev-parse", "HEAD"], true).unwrap_or_default();
    let mut out = format!(
        "{msg_extra}created annotated tag {new_tag} (was {tag}, +{ahead} commits)\n  HEAD: {full}\n"
    );

    // Branch and tag go up in one atomic push: pushed separately, CI checked
    // out the branch before the tag existed and the pin gate went red (v0.23.1).
    if opts.push {
        if let Err(e) = run_git(
            repo,
            &["push", "--atomic", "origin", "HEAD", &new_tag],
            true,
        ) {
            return (1, e);
        }
        out.push_str(&format!("pushed origin HEAD and {new_tag} (atomic)\n"));
    } else {
        out.push_str("下一步(须执行,否则下游仍升不了):\n");
        out.push_str(&format!("  git push --atomic origin HEAD {new_tag}\n"));
        out.push_str(&format!("并确认 README/AGENTS 推荐 pin 已改为 {new_tag}\n"));
    }
    (0, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(repo: &Path, args: &[&str]) {
        let st = Command::new("git")
            .args(args)
            .current_dir(repo)
            .status()
            .unwrap();
        assert!(st.success(), "git {args:?}");
    }

    #[test]
    fn bump_patch() {
        assert_eq!(bump("patch", 0, 6, 3).unwrap(), "v0.6.4");
        assert_eq!(bump("minor", 0, 6, 3).unwrap(), "v0.7.0");
        assert_eq!(bump("major", 0, 6, 3).unwrap(), "v1.0.0");
    }

    #[test]
    fn creates_tag_in_temp_repo() {
        let d = tempdir().unwrap();
        let repo = d.path();
        git(repo, &["init"]);
        git(repo, &["config", "user.email", "t@t.com"]);
        git(repo, &["config", "user.name", "t"]);
        fs::write(repo.join("f"), "1").unwrap();
        git(repo, &["add", "f"]);
        git(repo, &["commit", "-m", "c1"]);
        git(repo, &["tag", "-a", "v0.1.0", "-m", "v0.1.0"]);
        fs::write(repo.join("f"), "2").unwrap();
        git(repo, &["add", "f"]);
        git(repo, &["commit", "-m", "c2"]);

        let (code, msg) = run_tag_release(
            repo,
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: false,
                allow_dirty: false,
            },
        );
        assert_eq!(code, 0, "{msg}");
        assert!(msg.contains("v0.1.1"), "{msg}");
        let tags = run_git(repo, &["tag", "-l"], true).unwrap();
        assert!(tags.contains("v0.1.1"));
    }

    fn repo_with_intake_mark(mark: &str) -> tempfile::TempDir {
        let d = tempdir().unwrap();
        let repo = d.path();
        git(repo, &["init"]);
        git(repo, &["config", "user.email", "t@t.com"]);
        git(repo, &["config", "user.name", "t"]);
        fs::write(repo.join("f"), "1").unwrap();
        git(repo, &["add", "f"]);
        git(repo, &["commit", "-m", "c1"]);
        git(repo, &["tag", "-a", "v0.1.0", "-m", "v0.1.0"]);
        fs::create_dir_all(repo.join("intake")).unwrap();
        fs::write(
            repo.join("intake/2026-01-01-x.md"),
            format!("### slug\n- 状态: {mark}\n"),
        )
        .unwrap();
        git(repo, &["add", "intake"]);
        git(repo, &["commit", "-m", "c2"]);
        d
    }

    /// Dartify 2026-09-26: intake cards were marked `landed@v0.22.0` before
    /// that tag existed; another commit then took the number.
    #[test]
    fn refuses_when_intake_marks_a_tag_that_does_not_exist_yet() {
        let d = repo_with_intake_mark("landed@v0.1.1（kit/x.md）");
        let repo = d.path();
        let (code, msg) = run_tag_release(
            repo,
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: false,
                allow_dirty: false,
            },
        );
        assert_eq!(code, 1, "{msg}");
        assert!(msg.contains("landed@v0.1.1"), "{msg}");
        let tags = run_git(repo, &["tag", "-l"], true).unwrap();
        assert!(!tags.contains("v0.1.1"), "tag must not be created");
    }

    /// A work repo with `v0.1.0` and one commit on top, tracking a bare remote
    /// that already has `v0.1.0` and the first commit on `main`.
    fn repo_with_remote() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let d = tempdir().unwrap();
        let remote = d.path().join("remote.git");
        let repo = d.path().join("work");
        fs::create_dir_all(&repo).unwrap();
        git(
            d.path(),
            &["init", "--bare", "-b", "main", remote.to_str().unwrap()],
        );
        git(&repo, &["init", "-b", "main"]);
        git(&repo, &["config", "user.email", "t@t.com"]);
        git(&repo, &["config", "user.name", "t"]);
        fs::write(repo.join("f"), "1").unwrap();
        git(&repo, &["add", "f"]);
        git(&repo, &["commit", "-m", "c1"]);
        git(&repo, &["tag", "-a", "v0.1.0", "-m", "v0.1.0"]);
        git(
            &repo,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git(&repo, &["push", "origin", "main", "v0.1.0"]);
        fs::write(repo.join("f"), "2").unwrap();
        git(&repo, &["add", "f"]);
        git(&repo, &["commit", "-m", "c2"]);
        (d, repo, remote)
    }

    fn remote_rev(remote: &Path, rev: &str) -> Option<String> {
        let out = Command::new("git")
            .args([
                "--git-dir",
                remote.to_str().unwrap(),
                "rev-parse",
                "-q",
                "--verify",
            ])
            .arg(rev)
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    #[test]
    fn push_lands_branch_and_tag_together() {
        let (_d, repo, remote) = repo_with_remote();
        let (code, msg) = run_tag_release(
            &repo,
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: true,
                allow_dirty: false,
            },
        );
        assert_eq!(code, 0, "{msg}");
        let head = run_git(&repo, &["rev-parse", "HEAD"], true).unwrap();
        assert_eq!(
            remote_rev(&remote, "refs/heads/main").as_deref(),
            Some(head.as_str())
        );
        assert_eq!(
            remote_rev(&remote, "refs/tags/v0.1.1^{commit}").as_deref(),
            Some(head.as_str())
        );
    }

    /// v0.23.1: pushing the branch and the tag separately let CI's pin gate
    /// check out the branch before the tag existed. One atomic push means a
    /// rejected tag keeps the branch back too, and the other way round.
    #[test]
    fn push_is_atomic_so_a_rejected_tag_keeps_the_branch_back() {
        let (d, repo, remote) = repo_with_remote();
        // Someone else already published v0.1.1 on another commit.
        let other = d.path().join("other");
        git(
            d.path(),
            &[
                "clone",
                "-q",
                remote.to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        git(&other, &["config", "user.email", "o@o.com"]);
        git(&other, &["config", "user.name", "o"]);
        git(
            &other,
            &["tag", "-a", "v0.1.1", "-m", "not ours", "origin/main"],
        );
        git(&other, &["push", "origin", "v0.1.1"]);
        let before = remote_rev(&remote, "refs/heads/main");

        let (code, msg) = run_tag_release(
            &repo,
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: true,
                allow_dirty: false,
            },
        );
        assert_ne!(code, 0, "the tag push must fail: {msg}");
        assert_eq!(
            remote_rev(&remote, "refs/heads/main"),
            before,
            "branch reached the remote without its tag"
        );
    }

    #[test]
    fn next_step_hint_is_one_atomic_push() {
        let (_d, repo, _remote) = repo_with_remote();
        let (code, msg) = run_tag_release(
            &repo,
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: false,
                allow_dirty: false,
            },
        );
        assert_eq!(code, 0, "{msg}");
        assert!(
            msg.contains("git push --atomic origin HEAD v0.1.1"),
            "{msg}"
        );
        assert!(!msg.contains("&& git push"), "{msg}");
    }

    #[test]
    fn marks_pointing_at_existing_tags_pass() {
        let d = repo_with_intake_mark("landed@v0.1.0（kit/x.md）");
        let (code, msg) = run_tag_release(
            d.path(),
            &TagOpts {
                level: "patch".into(),
                title: "test".into(),
                push: false,
                allow_dirty: false,
            },
        );
        assert_eq!(code, 0, "{msg}");
    }
}
