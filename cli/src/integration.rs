//! Proof that a released task's exact work reached the target branch.
//! Dispatch and optional cleanup share this proof; neither depends on a GC
//! report, directory inventory, file ages or cleanup being enabled.
use crate::coordination::{Store, Task};
use crate::worktree;

pub(crate) fn released_task(
    store: &Store,
    task: &Task,
    base_sha: &str,
    base: &str,
) -> Result<bool, String> {
    let head = if task.worktree.exists() {
        worktree::git(&task.worktree, &["rev-parse", "HEAD"])?
    } else {
        worktree::git(
            &store.repo,
            &["rev-parse", &format!("refs/agent-on/recovery/{}", task.id)],
        )?
    };
    if task.released_head.as_deref() != Some(head.as_str()) {
        return Ok(false);
    }
    worktree::head_integrated(&store.repo, &task.worktree, &head, base_sha, base)
}
