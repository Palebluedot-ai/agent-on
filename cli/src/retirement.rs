//! One-shot retirement of old project collectors. Preserve files, refs and task history.
use crate::{state_io, worktree, worktree_schedule};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn inspect(repo: &Path, apply: bool) -> Result<String, String> {
    let common = worktree::common_git_dir(repo)?;
    let control = common.join("agent-on/control");
    for path in [common.join("agent-on"), control.clone()] {
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(format!("refusing symlink: {}", path.display()));
        }
    }
    let _lease = if apply && control.exists() {
        Some(state_io::lock_path(&control.join("write.lock"))?)
    } else {
        None
    };
    let mut changes = Vec::new();
    for (name, keys) in [
        ("config.json", &["enabled", "janitor_enabled"][..]),
        ("activity-config.json", &["enabled"][..]),
    ] {
        let path = control.join(name);
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        };
        if !metadata.is_file() || metadata.len() > 262_144 {
            return Err(format!("not a bounded regular config: {}", path.display()));
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let mut value: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let object = value
            .as_object_mut()
            .ok_or("legacy config is not an object")?;
        let changed = keys
            .iter()
            .any(|key| object.get(*key).is_some_and(|v| v != false));
        for key in keys {
            object.insert((*key).into(), Value::Bool(false));
        }
        if changed {
            changes.push((path, bytes, value));
        }
    }
    let timer = worktree_schedule::has_legacy_janitor(repo)?;
    let mut output = format!(
        "Legacy collectors: {} config(s) to disable; janitor timer: {}\n",
        changes.len(),
        if timer { "installed" } else { "absent" }
    );
    if apply {
        // Use the old writer lock so an old collector cannot race the policy update.
        for (path, bytes, value) in changes {
            let backup = path.with_extension("json.pre-v030");
            // create_new preserves the first backup and refuses symlink overwrite.
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&backup) {
                Ok(mut file) => {
                    use std::io::Write;
                    file.write_all(&bytes).map_err(|e| e.to_string())?;
                    file.sync_all().map_err(|e| e.to_string())?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !std::fs::symlink_metadata(&backup)
                        .map_err(|e| e.to_string())?
                        .is_file()
                    {
                        return Err("unsafe legacy backup".into());
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
            state_io::write_json(&path, &value)?;
        }
        if timer {
            let (code, message) = worktree_schedule::run_uninstall_janitor(repo);
            if code != 0 {
                return Err(format!(
                    "collectors disabled; timer requires inspection: {message}"
                ));
            }
            output.push_str(&message);
        }
        output.push_str("Retired policy disabled. Task history, worktrees, branches and recovery refs preserved.\n");
    } else {
        output.push_str("Preview only; use --apply to disable these project policies and remove the owned timer.\n");
    }
    output.push_str("Already-running old hosts and plugin copies need the upgrade steps in docs/migration-v0.30.md.\n");
    Ok(output)
}
pub(crate) fn run(repo: Option<PathBuf>, apply: bool) -> i32 {
    let repo =
        repo.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match inspect(&repo, apply) {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(error) => {
            eprintln!("agent-on retire-legacy: {error}");
            1
        }
    }
}
