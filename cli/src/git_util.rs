//! Shared read-only Git & GitHub CLI helpers.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct RepoContext {
    pub current_worktree: PathBuf,
    pub common_git_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RawWorktree {
    pub path: PathBuf,
    pub head: String,
    pub branch: Option<String>,
    pub locked: bool,
    pub lock_reason: Option<String>,
    pub prunable: bool,
    pub bare: bool,
    pub missing: bool,
    pub primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhPrItem {
    pub number: u64,
    pub title: String,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "headRefOid", default)]
    pub head_ref_oid: Option<String>,
    #[serde(rename = "isDraft", default)]
    pub is_draft: bool,
    pub url: String,
    #[serde(default)]
    pub files: Vec<GhPrFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhPrFile {
    pub path: String,
}

pub fn short_oid(s: &str) -> &str {
    s.char_indices().nth(7).map(|(i, _)| &s[..i]).unwrap_or(s)
}

pub fn canonical_or_raw(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn git_raw(cwd: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-c")
        .arg("core.quotepath=false")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|e| format!("无法执行 git：{e}"))
}

pub fn git_stdout(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let out = git_raw(cwd, args)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("git {} 退出码 {:?}", args.join(" "), out.status.code())
        } else {
            stderr
        })
    }
}

pub fn resolve_repo(cwd: &Path) -> Result<RepoContext, String> {
    let top = git_stdout(
        cwd,
        &["rev-parse", "--path-format=absolute", "--show-toplevel"],
    )
    .map_err(|_| format!("{} 不在 Git 工作树内", cwd.display()))?;
    let common = git_stdout(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    Ok(RepoContext {
        current_worktree: canonical_or_raw(Path::new(&top)),
        common_git_dir: canonical_or_raw(Path::new(&common)),
    })
}

pub fn commit_exists(cwd: &Path, rev: &str) -> Option<String> {
    let target = format!("{rev}^{{commit}}");
    git_stdout(cwd, &["rev-parse", "--verify", "--quiet", &target]).ok()
}

pub fn resolve_base_ref(cwd: &Path, explicit: Option<&str>) -> Result<(String, String), String> {
    if let Some(base) = explicit {
        let oid = commit_exists(cwd, base)
            .ok_or_else(|| format!("指定的基线引用 `{base}` 无法解析为提交"))?;
        return Ok((base.to_string(), oid));
    }

    if let Ok(origin_head) = git_stdout(
        cwd,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    ) {
        if let Some(oid) = commit_exists(cwd, &origin_head) {
            return Ok((origin_head, oid));
        }
    }

    for candidate in ["origin/main", "origin/master", "main", "master"] {
        if let Some(oid) = commit_exists(cwd, candidate) {
            return Ok((candidate.to_string(), oid));
        }
    }

    if let Ok(sym_head) = git_stdout(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"]) {
        if let Some(oid) = commit_exists(cwd, &sym_head) {
            return Ok((sym_head, oid));
        }
    }

    if let Some(oid) = commit_exists(cwd, "HEAD") {
        return Ok(("HEAD".to_string(), oid));
    }

    Err("仓库尚无任何可解析的基线提交".to_string())
}

pub fn list_worktrees(cwd: &Path) -> Result<Vec<RawWorktree>, String> {
    let raw = git_stdout(cwd, &["worktree", "list", "--porcelain"])?;
    let mut result = Vec::new();

    let mut path: Option<PathBuf> = None;
    let mut head = String::new();
    let mut branch: Option<String> = None;
    let mut locked = false;
    let mut lock_reason: Option<String> = None;
    let mut prunable = false;
    let mut bare = false;

    let flush = |result: &mut Vec<RawWorktree>,
                 path: &mut Option<PathBuf>,
                 head: &mut String,
                 branch: &mut Option<String>,
                 locked: &mut bool,
                 lock_reason: &mut Option<String>,
                 prunable: &mut bool,
                 bare: &mut bool| {
        if let Some(raw_path) = path.take() {
            let exists = raw_path.exists();
            let resolved = if exists {
                canonical_or_raw(&raw_path)
            } else {
                raw_path
            };
            let primary = result.is_empty() && !*bare;
            result.push(RawWorktree {
                path: resolved,
                head: std::mem::take(head),
                branch: branch.take(),
                locked: *locked,
                lock_reason: lock_reason.take(),
                prunable: *prunable,
                bare: *bare,
                missing: !exists,
                primary,
            });
        }
        *locked = false;
        *prunable = false;
        *bare = false;
    };

    for line in raw.lines() {
        if line.is_empty() {
            flush(
                &mut result,
                &mut path,
                &mut head,
                &mut branch,
                &mut locked,
                &mut lock_reason,
                &mut prunable,
                &mut bare,
            );
            continue;
        }
        if let Some(rest) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("HEAD ") {
            head = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("branch ") {
            branch = Some(rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string());
        } else if line == "locked" {
            locked = true;
        } else if let Some(reason) = line.strip_prefix("locked ") {
            locked = true;
            lock_reason = Some(reason.to_string());
        } else if line == "prunable" || line.starts_with("prunable ") {
            prunable = true;
        } else if line == "bare" {
            bare = true;
        }
    }
    flush(
        &mut result,
        &mut path,
        &mut head,
        &mut branch,
        &mut locked,
        &mut lock_reason,
        &mut prunable,
        &mut bare,
    );

    Ok(result)
}

pub fn classify_client(path: &Path, primary: bool) -> &'static str {
    if primary {
        return "primary";
    }
    let slash = path.to_string_lossy().replace('\\', "/");
    if slash.contains("/.claude/worktrees/") {
        "claude-code"
    } else if slash.contains("/.codex/worktrees/") {
        "codex"
    } else if slash.contains("/.gemini/antigravity/worktrees/") {
        "antigravity"
    } else {
        "custom"
    }
}

/// Parse `git status --porcelain=v1 -z --untracked-files=all --ignore-submodules=none`
/// so individual files in new directories, user config overrides, and dirty submodules are never hidden.
pub fn dirty_paths(wt_path: &Path) -> Result<BTreeSet<String>, String> {
    let out = git_raw(
        wt_path,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ],
    )?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let mut files = BTreeSet::new();
    let mut iter = out.stdout.split(|b| *b == 0).filter(|s| !s.is_empty());
    while let Some(entry) = iter.next() {
        if entry.len() < 4 {
            continue;
        }
        let status = &entry[..2];
        let path = String::from_utf8_lossy(&entry[3..]).to_string();
        if !path.is_empty() {
            files.insert(path);
        }
        if status.contains(&b'R') || status.contains(&b'C') {
            if let Some(orig) = iter.next() {
                let orig_path = String::from_utf8_lossy(orig).to_string();
                if !orig_path.is_empty() {
                    files.insert(orig_path);
                }
            }
        }
    }
    Ok(files)
}

pub fn diff_name_only_z(cwd: &Path, range: &str) -> Result<BTreeSet<String>, String> {
    let out = git_raw(cwd, &["diff", "--name-only", "-z", range, "--"])?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect())
}

pub fn query_open_prs(cwd: &Path) -> Result<Vec<GhPrItem>, String> {
    let out = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "100",
            "--json",
            "number,title,headRefName,headRefOid,isDraft,url,files",
        ])
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("无法执行 gh CLI：{e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("gh pr list 失败（退出码 {:?}）", out.status.code())
        } else {
            stderr
        });
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("解析 gh pr list JSON 失败：{e}"))
}
