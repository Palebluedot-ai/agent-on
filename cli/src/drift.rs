//! Projection drift: which stored copies no longer match the source they came from.
//!
//! Every durable record in this repo is one of two things: an original claim,
//! or a **projection** — a copy of a fact whose real home is somewhere else.
//! A lane record copies the worktree path, branch and base that git owns. A doc
//! sentence about what the guard blocks copies behaviour that `cli/src/` owns.
//!
//! Originals do not rot. Projections do, silently, because nothing links a copy
//! back to the thing it copied. This module is the reconciliation loop that was
//! missing: one command, one question — *which projections no longer match?*
//!
//! It is deliberately **report-only**. Bookkeeping drift is somebody else's
//! ledger; blocking this session's commit on it is the coupling that produced
//! the deadlocks recorded in `bench/cases/40-gate-exit-unreachable.md`. Every
//! row therefore carries an owner and a non-destructive exit command instead.

use crate::worktree::{self, LaneRecord};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// One projection that stopped matching its source.
#[derive(Debug, Clone, Serialize)]
pub struct DriftRow {
    /// Machine-readable defect class.
    pub kind: &'static str,
    /// What drifted: a lane id, or a repo-relative file path.
    pub subject: String,
    /// The measured mismatch, in the form "recorded X, git says Y".
    pub detail: String,
    /// Who may repair it. A projection nobody owns is a projection nobody feeds.
    pub owner: String,
    /// A non-destructive command that closes this row. Never a delete.
    pub exit: String,
}

#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub repo: String,
    pub lanes_scanned: usize,
    pub docs_scanned: usize,
    pub ledger: Vec<DriftRow>,
    pub docs: Vec<DriftRow>,
}

impl DriftReport {
    pub fn total(&self) -> usize {
        self.ledger.len() + self.docs.len()
    }
}

// ---------------------------------------------------------------------------
// Ledger side: lane records vs git
// ---------------------------------------------------------------------------

/// How stable is the ref a lane recorded as its `base`?
///
/// `base` is not decoration: the boundary audit diffs `base...HEAD` against it,
/// so an unstable base turns every file in the tree into an OUT-OF-BOUNDS row.
/// A remote-tracking ref moves only when someone fetches; a local branch moves
/// every time a merge lands under you.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseVerdict {
    /// Remote-tracking ref, tag, or raw sha — safe to diff against.
    Stable,
    /// Local branch: it moves under the lane whenever a merge lands.
    LocalMoving,
    /// Does not resolve at all; the boundary audit cannot compare.
    Unresolvable,
}

/// Judge a base from what `git rev-parse --symbolic-full-name` said about it.
/// `None` means the ref did not resolve.
pub fn base_verdict(full_name: Option<&str>) -> BaseVerdict {
    match full_name {
        None => BaseVerdict::Unresolvable,
        Some(name) if name.starts_with("refs/heads/") => BaseVerdict::LocalMoving,
        Some(_) => BaseVerdict::Stable,
    }
}

/// The non-destructive way out of a ghost lane, given its recorded status.
///
/// `forget` only accepts `landed|parked`, and `set-status` has no edge out of
/// `landed`, so a live ghost needs the `edit --status` repair door first. Both
/// steps are metadata-only: neither touches a worktree, branch or commit.
pub fn ghost_exit(id: &str, status: &str) -> String {
    if matches!(status, "landed" | "parked") {
        format!("agent-on worktree forget --id {id}")
    } else {
        format!(
            "agent-on worktree edit --id {id} --status parked && agent-on worktree forget --id {id}"
        )
    }
}

fn scan_ledger(cwd: &Path) -> Result<(usize, Vec<DriftRow>), String> {
    let records = worktree::load_records(cwd)?;
    let live_trees = worktree::parse_worktrees(&worktree::git(
        cwd,
        &["worktree", "list", "--porcelain"],
    )?);
    let mut rows = Vec::new();

    for record in &records {
        let path = PathBuf::from(&record.worktree);
        if !path.is_dir() {
            rows.push(DriftRow {
                kind: "GHOST-LANE",
                subject: record.id.clone(),
                detail: format!(
                    "recorded worktree is gone: {} (status {})",
                    record.worktree, record.status
                ),
                owner: format!("lane {} (metadata only; no files at risk)", record.id),
                exit: ghost_exit(&record.id, &record.status),
            });
            // Every remaining field describes a tree that no longer exists.
            // Reporting them would be noise about a row that is about to go.
            continue;
        }

        if let Some(info) = live_trees.iter().find(|w| w.path == path) {
            match &info.branch {
                Some(actual) if actual != &record.branch => rows.push(DriftRow {
                    kind: "BRANCH-DRIFT",
                    subject: record.id.clone(),
                    detail: format!(
                        "recorded branch {}, worktree is on {}",
                        record.branch, actual
                    ),
                    owner: format!("lane {}", record.id),
                    exit: format!(
                        "agent-on worktree edit --id {} --branch {}",
                        record.id, actual
                    ),
                }),
                None => rows.push(DriftRow {
                    kind: "BRANCH-DRIFT",
                    subject: record.id.clone(),
                    detail: format!(
                        "recorded branch {}, worktree is detached at {}",
                        record.branch,
                        &info.head[..info.head.len().min(12)]
                    ),
                    owner: format!("lane {}", record.id),
                    exit: format!(
                        "git -C {} switch {} , then re-run drift",
                        record.worktree, record.branch
                    ),
                }),
                _ => {}
            }
        }

        let full = worktree::git(
            &path,
            &["rev-parse", "--symbolic-full-name", "--verify", "--quiet", &record.base],
        )
        .ok()
        .filter(|s| !s.is_empty());
        // `--symbolic-full-name` prints nothing for a raw sha, so fall back to a
        // plain verify before calling it unresolvable.
        let resolves_as_sha = full.is_none()
            && worktree::git(&path, &["rev-parse", "--verify", "--quiet", &record.base]).is_ok();
        let verdict = if resolves_as_sha {
            BaseVerdict::Stable
        } else {
            base_verdict(full.as_deref())
        };
        match verdict {
            BaseVerdict::Stable => {}
            BaseVerdict::LocalMoving => rows.push(DriftRow {
                kind: "UNSTABLE-BASE",
                subject: record.id.clone(),
                detail: format!(
                    "base {} is a local branch; it moves under this lane every time a merge lands, \
and the boundary audit diffs against it",
                    record.base
                ),
                owner: format!("lane {}", record.id),
                exit: format!(
                    "agent-on worktree edit --id {} --base origin/{}",
                    record.id, record.base
                ),
            }),
            BaseVerdict::Unresolvable => rows.push(DriftRow {
                kind: "UNRESOLVABLE-BASE",
                subject: record.id.clone(),
                detail: format!(
                    "base {} does not resolve in {}; the boundary audit cannot compare",
                    record.base, record.worktree
                ),
                owner: format!("lane {}", record.id),
                exit: format!(
                    "agent-on worktree edit --id {} --base origin/<default>",
                    record.id
                ),
            }),
        }
    }

    Ok((records.len(), rows))
}

// ---------------------------------------------------------------------------
// Doc side: prose about machine behaviour vs the implementation
// ---------------------------------------------------------------------------

/// Directories whose markdown is exempt from the anchor requirement.
///
/// `intake/` is raw material carried in from other projects, `legacy/` is
/// archived evidence, and `CHANGELOG.md` is a dated record of what changed —
/// none of them are claims this repo makes about how the tool behaves today.
fn anchor_exempt(rel: &str) -> bool {
    rel.starts_with("intake/") || rel.starts_with("legacy/") || rel == "CHANGELOG.md"
}

fn anchor_re() -> Regex {
    Regex::new(r"<!--\s*src:\s*([^\s#>]+)(?:#([^\s>]+))?\s*-->").expect("static anchor regex")
}

fn impl_ref_re() -> Regex {
    Regex::new(r"cli/(?:src|tests)/[A-Za-z0-9_]+\.rs").expect("static impl-path regex")
}

/// Every `<!-- src: path[#needle] -->` in the text, in order.
pub fn parse_anchors(text: &str) -> Vec<(String, Option<String>)> {
    anchor_re()
        .captures_iter(text)
        .map(|c| {
            (
                c[1].to_string(),
                c.get(2).map(|m| m.as_str().to_string()),
            )
        })
        .collect()
}

/// Does `body` contain `needle` as a whole symbol rather than as a fragment?
///
/// Substring containment is not enough: renaming `foo` to `foo_v2` leaves every
/// anchor to `foo` green while the symbol it named is gone. That failure was
/// found by mutating the symbol and watching the anchor stay silent — the
/// repo's own rule that an anchor which never goes red was never tested.
pub fn contains_symbol(body: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let bytes = body.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = body[from..].find(needle) {
        let start = from + rel;
        let end = start + needle.len();
        let before_ok = start == 0 || !body[..start].chars().next_back().is_some_and(ident);
        let after_ok = end >= bytes.len() || !body[end..].chars().next().is_some_and(ident);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Every implementation file the text names in prose.
pub fn named_impl_files(text: &str) -> BTreeSet<String> {
    impl_ref_re()
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Which lane, if any, holds this path — the answer to "who may fix this row".
fn owner_of(rel: &str, records: &[LaneRecord]) -> String {
    let holders: Vec<&str> = records
        .iter()
        .filter(|r| worktree::owns_path(&r.owns, rel))
        .map(|r| r.id.as_str())
        .collect();
    if holders.is_empty() {
        "unclaimed (any session may fix it, then say so)".to_string()
    } else {
        format!("lane {}", holders.join(" / "))
    }
}

fn scan_docs(root: &Path, records: &[LaneRecord]) -> (usize, Vec<DriftRow>) {
    let files = markdown_files(root);
    let mut rows = Vec::new();

    for file in &files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        let rel = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");

        let anchors = parse_anchors(&text);

        // 1. A declared anchor that no longer resolves is always wrong, in every
        //    directory: a pointer into nothing sends the next reader nowhere.
        for (target, needle) in &anchors {
            let full = root.join(target);
            if !full.exists() {
                rows.push(DriftRow {
                    kind: "ANCHOR-BROKEN",
                    subject: rel.clone(),
                    detail: format!("anchor points at {target}, which no longer exists"),
                    owner: owner_of(&rel, records),
                    exit: format!("repoint the anchor in {rel}, or delete the claim it backs"),
                });
                continue;
            }
            if let Some(needle) = needle {
                let hit = fs::read_to_string(&full)
                    .map(|body| contains_symbol(&body, needle))
                    .unwrap_or(false);
                if !hit {
                    rows.push(DriftRow {
                        kind: "ANCHOR-BROKEN",
                        subject: rel.clone(),
                        detail: format!(
                            "anchor names {needle} in {target}, which no longer contains it"
                        ),
                        owner: owner_of(&rel, records),
                        exit: format!(
                            "re-read {target}, then correct the claim in {rel} and re-point the anchor"
                        ),
                    });
                }
            }
        }

        // 2. Naming an implementation file without anchoring it is the shape
        //    that produced the misjudgement: a file path stays valid while the
        //    behaviour inside it changes completely.
        if anchor_exempt(&rel) {
            continue;
        }
        let anchored: BTreeSet<&str> = anchors.iter().map(|(t, _)| t.as_str()).collect();
        for named in named_impl_files(&text) {
            if !anchored.contains(named.as_str()) {
                rows.push(DriftRow {
                    kind: "ANCHOR-MISSING",
                    subject: rel.clone(),
                    detail: format!(
                        "names {named} in prose with no src anchor; a bare file path cannot go stale, \
so it proves nothing about the behaviour claimed here"
                    ),
                    owner: owner_of(&rel, records),
                    exit: format!("add `<!-- src: {named}#<symbol> -->` beside the claim in {rel}"),
                });
            }
        }
    }

    (files.len(), rows)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn build_report(cwd: &Path) -> Result<DriftReport, String> {
    let root = worktree::repo_root(cwd)?;
    let records = worktree::load_records(cwd).unwrap_or_default();
    let (lanes_scanned, ledger) = scan_ledger(cwd)?;
    let (docs_scanned, docs) = scan_docs(&root, &records);
    Ok(DriftReport {
        repo: root.display().to_string(),
        lanes_scanned,
        docs_scanned,
        ledger,
        docs,
    })
}

fn render_section(title: &str, rows: &[DriftRow], out: &mut String) {
    out.push_str(&format!("\n{title}\n"));
    if rows.is_empty() {
        out.push_str("  (none)\n");
        return;
    }
    for row in rows {
        out.push_str(&format!("- {} {}\n", row.kind, row.subject));
        out.push_str(&format!("  drift: {}\n", row.detail));
        out.push_str(&format!("  owner: {}\n", row.owner));
        out.push_str(&format!("  exit:  {}\n", row.exit));
    }
}

pub fn render_text(report: &DriftReport) -> String {
    let mut out = format!("PROJECTION DRIFT: {}\n", report.repo);
    out.push_str(&format!(
        "scanned: {} lane record(s), {} markdown file(s)\n",
        report.lanes_scanned, report.docs_scanned
    ));
    render_section("LEDGER (lane records vs git)", &report.ledger, &mut out);
    render_section("DOCS (prose vs implementation)", &report.docs, &mut out);
    if report.total() == 0 {
        out.push_str("\nRESULT: CLEAN\n");
    } else {
        out.push_str(&format!(
            "\nRESULT: {} drift row(s). Report only — none of this blocks a commit.\n",
            report.total()
        ));
    }
    out
}

/// `agent-on drift`. Exit 0 unless `--strict` is set, so a stale ledger never
/// holds another session's work hostage.
pub fn run(cwd: &Path, json: bool, strict: bool) -> (i32, String) {
    match build_report(cwd) {
        Ok(report) => {
            let code = if strict && report.total() > 0 { 1 } else { 0 };
            let text = if json {
                match serde_json::to_string_pretty(&report) {
                    Ok(s) => format!("{s}\n"),
                    Err(e) => return (1, format!("ERROR: serialize report: {e}\n")),
                }
            } else {
                render_text(&report)
            };
            (code, text)
        }
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_branch_base_is_flagged_as_moving() {
        assert_eq!(
            base_verdict(Some("refs/heads/main")),
            BaseVerdict::LocalMoving
        );
    }

    #[test]
    fn remote_tracking_base_is_stable() {
        assert_eq!(
            base_verdict(Some("refs/remotes/origin/main")),
            BaseVerdict::Stable
        );
    }

    #[test]
    fn unresolvable_base_is_its_own_class() {
        assert_eq!(base_verdict(None), BaseVerdict::Unresolvable);
    }

    #[test]
    fn ghost_exit_for_finished_lane_is_a_single_forget() {
        assert_eq!(
            ghost_exit("auth-api", "landed"),
            "agent-on worktree forget --id auth-api"
        );
    }

    #[test]
    fn ghost_exit_for_live_lane_opens_the_repair_door_first() {
        // `forget` refuses a live lane and `set-status` has no edge out of
        // `landed`, so the exit has to go through `edit --status`.
        let exit = ghost_exit("auth-api", "active");
        assert!(exit.contains("edit --id auth-api --status parked"), "{exit}");
        assert!(exit.contains("forget --id auth-api"), "{exit}");
    }

    #[test]
    fn anchors_parse_with_and_without_a_needle() {
        let text = "see <!-- src: cli/src/guard.rs#commit_push_dirs --> and <!-- src: hooks/hooks.json -->";
        let got = parse_anchors(text);
        assert_eq!(
            got,
            vec![
                (
                    "cli/src/guard.rs".to_string(),
                    Some("commit_push_dirs".to_string())
                ),
                ("hooks/hooks.json".to_string(), None),
            ]
        );
    }

    #[test]
    fn prose_mentions_of_implementation_files_are_collected() {
        let text = "实现在 cli/src/oncall.rs + cli/src/guard.rs，回归在 cli/tests/oncall_routing.rs";
        let got = named_impl_files(text);
        assert!(got.contains("cli/src/oncall.rs"), "{got:?}");
        assert!(got.contains("cli/src/guard.rs"), "{got:?}");
        assert!(got.contains("cli/tests/oncall_routing.rs"), "{got:?}");
    }

    #[test]
    fn a_symbol_anchor_goes_red_when_the_symbol_is_renamed() {
        let before = "parsed.commit_push_dirs.insert(git_dir.clone());";
        let after = "parsed.commit_push_dirs_RENAMED.insert(git_dir.clone());";
        assert!(contains_symbol(before, "commit_push_dirs"));
        // The rename only extends the name, so plain substring containment
        // would still say yes. Whole-symbol matching is what makes the anchor
        // able to discriminate.
        assert!(after.contains("commit_push_dirs"));
        assert!(!contains_symbol(after, "commit_push_dirs"));
    }

    #[test]
    fn a_symbol_is_matched_across_punctuation_but_not_inside_a_longer_word() {
        assert!(contains_symbol("fn base_verdict(full: Option<&str>)", "base_verdict"));
        assert!(contains_symbol("crate::drift::base_verdict;", "base_verdict"));
        assert!(!contains_symbol("fn xbase_verdict()", "base_verdict"));
    }

    #[test]
    fn carried_in_material_and_the_changelog_are_exempt() {
        assert!(anchor_exempt("intake/2026-08-16-dartify.md"));
        assert!(anchor_exempt("legacy/gen1-role-model/README.md"));
        assert!(anchor_exempt("CHANGELOG.md"));
        assert!(!anchor_exempt("kit/worktree-gc-pattern.md"));
        assert!(!anchor_exempt("snapshot/2026-08-20-gate-exit-reachability.md"));
    }
}
