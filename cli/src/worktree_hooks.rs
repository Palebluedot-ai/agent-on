use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const STATE_VERSION: u8 = 1;
const MANAGED_MARKER: &str = "# agent-on managed worktree hook v1";
const HOOK_NAMES: &[&str] = &["pre-commit", "pre-push"];

#[derive(Debug, Serialize, Deserialize)]
struct HookInstallState {
    version: u8,
    hooks_dir: String,
    executable: String,
    pre_commit: String,
    pre_push: String,
}

#[derive(Debug, Deserialize)]
struct LaneSummary {
    id: String,
    worktree: String,
    status: String,
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|e| format!("cannot run git: {e}"))
}

fn git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = git_output(cwd, args)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        })
    }
}

fn repo_root(cwd: &Path) -> Result<PathBuf, String> {
    git(cwd, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

fn common_git_dir(cwd: &Path) -> Result<PathBuf, String> {
    if let Ok(raw) = git(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ) {
        return Ok(PathBuf::from(raw));
    }
    let raw = PathBuf::from(git(cwd, &["rev-parse", "--git-common-dir"])?);
    let path = if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    };
    Ok(fs::canonicalize(&path).unwrap_or(path))
}

fn managed_root(cwd: &Path) -> Result<PathBuf, String> {
    Ok(common_git_dir(cwd)?.join("agent-on"))
}

fn hooks_dir(cwd: &Path) -> Result<PathBuf, String> {
    Ok(managed_root(cwd)?.join("hooks"))
}

fn state_path(cwd: &Path) -> Result<PathBuf, String> {
    Ok(managed_root(cwd)?.join("worktree-hooks.json"))
}

fn worktree_paths(cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let raw = git(cwd, &["worktree", "list", "--porcelain"])?;
    let paths = raw
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        Err("git worktree list returned no worktrees".to_string())
    } else {
        Ok(paths)
    }
}

fn config_values(cwd: &Path, local_only: bool) -> Result<Vec<String>, String> {
    let args = if local_only {
        ["config", "--local", "--get-all", "core.hooksPath"].as_slice()
    } else {
        ["config", "--get-all", "core.hooksPath"].as_slice()
    };
    let output = git_output(cwd, args)?;
    match output.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&output.stdout)
            .split_terminator('\n')
            .map(|value| value.trim_end_matches('\r').to_string())
            .collect()),
        Some(1) if output.stderr.is_empty() => Ok(Vec::new()),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                "cannot read core.hooksPath".to_string()
            } else {
                stderr
            })
        }
    }
}

fn worktree_config_enabled(cwd: &Path) -> Result<bool, String> {
    let output = git_output(
        cwd,
        &["config", "--bool", "--get", "extensions.worktreeConfig"],
    )?;
    match output.status.code() {
        Some(0) => match String::from_utf8_lossy(&output.stdout).trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            value => Err(format!("invalid extensions.worktreeConfig value: {value}")),
        },
        Some(1) if output.stderr.is_empty() => Ok(false),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                "cannot read extensions.worktreeConfig".to_string()
            } else {
                stderr
            })
        }
    }
}

fn worktree_scope_values(cwd: &Path, enabled: bool) -> Result<Vec<String>, String> {
    if !enabled {
        return Ok(Vec::new());
    }
    let output = git_output(
        cwd,
        &["config", "--worktree", "--get-all", "core.hooksPath"],
    )?;
    match output.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&output.stdout)
            .split_terminator('\n')
            .map(|value| value.trim_end_matches('\r').to_string())
            .collect()),
        Some(1) if output.stderr.is_empty() => Ok(Vec::new()),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!(
                    "cannot read worktree-scope core.hooksPath in {}",
                    cwd.display()
                )
            } else {
                stderr
            })
        }
    }
}

#[derive(Debug)]
struct WorktreeHookConfig {
    path: PathBuf,
    effective: Vec<String>,
    worktree_scope: Vec<String>,
}

fn all_worktree_hook_configs(cwd: &Path) -> Result<Vec<WorktreeHookConfig>, String> {
    let enabled = worktree_config_enabled(cwd)?;
    worktree_paths(cwd)?
        .into_iter()
        .map(|path| {
            if !path.exists() {
                return Err(format!(
                    "cannot verify hooks config for missing worktree {}",
                    path.display()
                ));
            }
            Ok(WorktreeHookConfig {
                effective: config_values(&path, false)?,
                worktree_scope: worktree_scope_values(&path, enabled)?,
                path,
            })
        })
        .collect()
}

fn set_hooks_path(cwd: &Path, value: &Path) -> Result<(), String> {
    let value = value
        .to_str()
        .ok_or_else(|| format!("hooks path is not valid UTF-8: {}", value.display()))?;
    git(
        cwd,
        &[
            "config",
            "--local",
            "--replace-all",
            "core.hooksPath",
            value,
        ],
    )
    .map(|_| ())
}

fn unset_hooks_path(cwd: &Path) -> Result<(), String> {
    let output = git_output(cwd, &["config", "--local", "--unset-all", "core.hooksPath"])?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "cannot unset repository-local core.hooksPath".to_string()
        } else {
            stderr
        })
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn hook_script(executable: &Path, hook: &str) -> Result<String, String> {
    let executable = executable.to_str().ok_or_else(|| {
        format!(
            "executable path is not valid UTF-8: {}",
            executable.display()
        )
    })?;
    Ok(format!(
        "#!/bin/sh\n{MANAGED_MARKER}\nexec {} worktree hooks run --hook {} --repo \"$PWD\"\n",
        shell_quote(executable),
        shell_quote(hook)
    ))
}

fn write_new(path: &Path, contents: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("refuse to overwrite {}: {e}", path.display()))?;
    file.write_all(contents.as_bytes())
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    file.sync_all()
        .map_err(|e| format!("sync {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|e| format!("stat {}: {e}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|e| format!("chmod {}: {e}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn read_state(path: &Path) -> Result<HookInstallState, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let state: HookInstallState =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if state.version != STATE_VERSION {
        return Err(format!(
            "unsupported hook state version {} in {} (expected {})",
            state.version,
            path.display(),
            STATE_VERSION
        ));
    }
    Ok(state)
}

fn script_for<'a>(state: &'a HookInstallState, name: &str) -> &'a str {
    match name {
        "pre-commit" => &state.pre_commit,
        "pre-push" => &state.pre_push,
        _ => unreachable!("fixed managed hook name"),
    }
}

fn installation_problems(cwd: &Path, state: &HookInstallState) -> Result<Vec<String>, String> {
    let expected_dir = hooks_dir(cwd)?;
    let mut problems = Vec::new();
    if Path::new(&state.hooks_dir) != expected_dir {
        problems.push(format!(
            "state points to unexpected hooks directory {} (expected {})",
            state.hooks_dir,
            expected_dir.display()
        ));
    }

    let local_values = config_values(cwd, true)?;
    let expected = expected_dir.display().to_string();
    if local_values != [expected.clone()] {
        problems.push(format!(
            "repository-local core.hooksPath is {} (expected {})",
            if local_values.is_empty() {
                "unset".to_string()
            } else {
                local_values.join(", ")
            },
            expected
        ));
    }
    for config in all_worktree_hook_configs(cwd)? {
        if !config.worktree_scope.is_empty() {
            problems.push(format!(
                "{} has worktree-scope core.hooksPath={} which can bypass the shared guard",
                config.path.display(),
                config.worktree_scope.join(", ")
            ));
        }
        if config.effective != [expected.clone()] {
            problems.push(format!(
                "{} resolves effective core.hooksPath={} (expected {})",
                config.path.display(),
                if config.effective.is_empty() {
                    "unset".to_string()
                } else {
                    config.effective.join(", ")
                },
                expected
            ));
        }
    }

    for name in HOOK_NAMES {
        let path = expected_dir.join(name);
        match fs::read_to_string(&path) {
            Ok(contents) if contents == script_for(state, name) => {
                if !is_executable(&path) {
                    problems.push(format!("{} is not executable", path.display()));
                }
            }
            Ok(_) => problems.push(format!(
                "{} was changed after installation; Agent-On will not overwrite it",
                path.display()
            )),
            Err(e) => problems.push(format!("cannot read {}: {e}", path.display())),
        }
    }
    if !Path::new(&state.executable).is_file() {
        problems.push(format!(
            "installed agent-on executable is missing: {}",
            state.executable
        ));
    } else if !is_executable(Path::new(&state.executable)) {
        problems.push(format!(
            "installed agent-on executable is not executable: {}",
            state.executable
        ));
    }
    Ok(problems)
}

fn unmanaged_default_hooks(cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let default_dir = common_git_dir(cwd)?.join("hooks");
    if !default_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&default_dir)
        .map_err(|e| format!("read existing hooks in {}: {e}", default_dir.display()))?
    {
        let path = entry
            .map_err(|e| format!("read existing hook entry: {e}"))?
            .path();
        let sample = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.ends_with(".sample"))
            .unwrap_or(false);
        if !sample {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn remove_if_exact(path: &Path, expected: &str) {
    if fs::read_to_string(path).ok().as_deref() == Some(expected) {
        let _ = fs::remove_file(path);
    }
}

fn uninstall_preflight(cwd: &Path) -> Result<Option<HookInstallState>, String> {
    let state_file = state_path(cwd)?;
    let directory = hooks_dir(cwd)?;
    if !state_file.exists() {
        let local_values = config_values(cwd, true)?;
        let points_to_managed = local_values == [directory.display().to_string()];
        let orphaned_files = HOOK_NAMES
            .iter()
            .map(|name| directory.join(name))
            .filter(|path| path.exists())
            .collect::<Vec<_>>();
        if points_to_managed || !orphaned_files.is_empty() {
            return Err(format!(
                "managed state is missing, but Agent-On hook assets remain (core.hooksPath managed={} files={}). Review them manually; nothing was removed",
                points_to_managed,
                if orphaned_files.is_empty() {
                    "none".to_string()
                } else {
                    orphaned_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ));
        }
        return Ok(None);
    }

    let state = read_state(&state_file)?;
    // hooks_dir is the value installed into core.hooksPath. It can differ from
    // the current common-dir location after the whole repository is moved; the
    // state file and exact hook contents move with that Git store and remain
    // sufficient ownership evidence for safe uninstall.
    let configured_expected = state.hooks_dir.clone();
    let local_values = config_values(cwd, true)?;
    if local_values != [configured_expected.clone()] {
        return Err(format!(
            "repository-local core.hooksPath changed after install (current: {})",
            if local_values.is_empty() {
                "unset".to_string()
            } else {
                local_values.join(", ")
            }
        ));
    }
    for config in all_worktree_hook_configs(cwd)? {
        if !config.worktree_scope.is_empty() {
            return Err(format!(
                "{} has worktree-scope core.hooksPath={} which can bypass the shared guard",
                config.path.display(),
                config.worktree_scope.join(", ")
            ));
        }
        if config.effective != [configured_expected.clone()] {
            return Err(format!(
                "{} resolves effective core.hooksPath={} (expected {})",
                config.path.display(),
                if config.effective.is_empty() {
                    "unset".to_string()
                } else {
                    config.effective.join(", ")
                },
                configured_expected
            ));
        }
    }
    for name in HOOK_NAMES {
        let path = directory.join(name);
        let actual = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read managed {}: {error}", path.display()))?;
        if actual != script_for(&state, name) {
            return Err(format!(
                "{} changed after installation; Agent-On will not remove it",
                path.display()
            ));
        }
    }
    Ok(Some(state))
}

pub fn install(repo: &Path) -> (i32, String) {
    match install_inner(repo) {
        Ok(message) => (0, message),
        Err(error) => (
            1,
            format!(
                "ERROR: cannot install Agent-On worktree hooks: {error}\nnext: fix the reported conflict, then rerun `agent-on worktree hooks install`\n"
            ),
        ),
    }
}

pub fn install_with_options(repo: &Path, daily_gc: bool) -> (i32, String) {
    let (hook_code, mut output) = install(repo);
    if hook_code != 0 || !daily_gc {
        return (hook_code, output);
    }
    let root = match repo_root(repo) {
        Ok(root) => root,
        Err(error) => {
            output.push_str(&format!(
                "ERROR: Git hooks were installed, but the daily GC repository could not be resolved: {error}\n"
            ));
            return (2, output);
        }
    };
    let schedule_root = match shared_repo_root(&root) {
        Ok(root) => root,
        Err(error) => {
            output.push_str(&format!(
                "ERROR: Git hooks were installed, but the shared repository root for daily GC could not be resolved: {error}\n"
            ));
            return (2, output);
        }
    };
    let (schedule_code, schedule_output) = crate::worktree_schedule::run_install(&schedule_root);
    output.push_str(&schedule_output);
    if schedule_code != 0 {
        output.push_str(
            "Git pre-commit/pre-push protection remains installed and healthy. Fix the scheduler error, then rerun `agent-on worktree hooks install --daily-gc`.\n",
        );
    }
    (schedule_code, output)
}

fn install_inner(repo: &Path) -> Result<String, String> {
    let root = repo_root(repo)?;
    let state_file = state_path(&root)?;
    if state_file.exists() {
        let state = read_state(&state_file)?;
        let problems = installation_problems(&root, &state)?;
        if problems.is_empty() {
            return Ok(format!(
                "WORKTREE HOOKS: already installed and healthy\nrepo: {}\nhooks: {}\n",
                root.display(),
                state.hooks_dir
            ));
        }
        return Err(format!(
            "an incomplete or changed installation already exists:\n- {}\nrun `agent-on worktree hooks status` for the same repository; no files were changed",
            problems.join("\n- ")
        ));
    }

    let configured = all_worktree_hook_configs(&root)?;
    let conflicts = configured
        .iter()
        .filter(|config| !config.effective.is_empty() || !config.worktree_scope.is_empty())
        .map(|config| {
            format!(
                "{} effective={} worktree-scope={}",
                config.path.display(),
                if config.effective.is_empty() {
                    "unset".to_string()
                } else {
                    config.effective.join(", ")
                },
                if config.worktree_scope.is_empty() {
                    "unset".to_string()
                } else {
                    config.worktree_scope.join(", ")
                }
            )
        })
        .collect::<Vec<_>>();
    if !conflicts.is_empty() {
        return Err(format!(
            "core.hooksPath is already configured in one or more worktrees: {}. Agent-On will not replace or bypass existing hooks; compose them explicitly, then retry",
            conflicts.join("; ")
        ));
    }
    let existing = unmanaged_default_hooks(&root)?;
    if !existing.is_empty() {
        return Err(format!(
            "existing user hook files would be bypassed: {}. Agent-On left them untouched; move/compose them explicitly before retrying",
            existing
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let executable = std::env::current_exe()
        .map_err(|e| format!("cannot locate current agent-on executable: {e}"))?;
    let executable = fs::canonicalize(&executable).unwrap_or(executable);
    let directory = hooks_dir(&root)?;
    fs::create_dir_all(&directory).map_err(|e| format!("create {}: {e}", directory.display()))?;
    let pre_commit = hook_script(&executable, "pre-commit")?;
    let pre_push = hook_script(&executable, "pre-push")?;
    let state = HookInstallState {
        version: STATE_VERSION,
        hooks_dir: directory.display().to_string(),
        executable: executable.display().to_string(),
        pre_commit,
        pre_push,
    };
    let pre_commit_path = directory.join("pre-commit");
    let pre_push_path = directory.join("pre-push");

    let result = (|| -> Result<(), String> {
        write_new(&pre_commit_path, &state.pre_commit)?;
        make_executable(&pre_commit_path)?;
        write_new(&pre_push_path, &state.pre_push)?;
        make_executable(&pre_push_path)?;
        let raw = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("serialize hook install state: {e}"))?;
        write_new(&state_file, &format!("{raw}\n"))?;
        set_hooks_path(&root, &directory)?;
        let problems = installation_problems(&root, &state)?;
        if !problems.is_empty() {
            return Err(format!(
                "post-install verification failed:\n- {}",
                problems.join("\n- ")
            ));
        }
        Ok(())
    })();

    if let Err(error) = result {
        if config_values(&root, true).unwrap_or_default() == [directory.display().to_string()] {
            let _ = unset_hooks_path(&root);
        }
        remove_if_exact(&pre_commit_path, &state.pre_commit);
        remove_if_exact(&pre_push_path, &state.pre_push);
        let _ = fs::remove_file(&state_file);
        let _ = fs::remove_dir(&directory);
        return Err(error);
    }

    Ok(format!(
        "WORKTREE HOOKS: installed\nrepo: {}\nGit hooks: managed here for all worktrees via shared core.hooksPath\npre-commit: worktree check + primary control-track guard\npre-push: worktree check\nPreToolUse: plugin-managed; this command did not edit ~/.claude or ~/.codex (verify host trust once with `/hooks`)\nstatus: `agent-on worktree hooks status`\nrollback: `agent-on worktree hooks uninstall`\n",
        root.display()
    ))
}

pub fn status(repo: &Path) -> (i32, String) {
    match status_inner(repo) {
        Ok((healthy, message)) => (if healthy { 0 } else { 1 }, message),
        Err(error) => (
            1,
            format!("ERROR: cannot inspect Agent-On worktree hooks: {error}\n"),
        ),
    }
}

pub fn status_with_schedule(repo: &Path) -> (i32, String) {
    let (hook_code, mut output) = status(repo);
    // Scheduler state lives outside the repository and remains discoverable by
    // its install manifest after a move/delete. Always inspect it even when the
    // Git store is no longer reachable.
    let schedule = crate::worktree_schedule::inspect(repo);
    output.push_str(&crate::worktree_schedule::render_status(&schedule));
    output.push('\n');
    if let Some(warning) = legacy_codex_hook_warning(std::env::var_os("HOME").as_deref()) {
        output.push_str(&warning);
        output.push('\n');
    }
    let unhealthy_installed_schedule = schedule.installed && !schedule.healthy();
    (
        if hook_code != 0 || unhealthy_installed_schedule {
            1
        } else {
            0
        },
        output,
    )
}

fn legacy_codex_hook_warning(home: Option<&std::ffi::OsStr>) -> Option<String> {
    let path = PathBuf::from(home?).join(".codex").join("hooks.json");
    let raw = fs::read_to_string(&path).ok()?;
    if raw.contains("python3") && raw.contains("agent-on-git-guard.sh") {
        Some(format!(
            "WARN PreToolUse: {} still uses the legacy python3 wrapper (it fails on older Bash-only checkouts and is redundant on v0.12.1+). After verifying the plugin-managed hook once, remove that user-level entry manually; Git-hook health above is unaffected",
            path.display()
        ))
    } else {
        None
    }
}

fn status_inner(repo: &Path) -> Result<(bool, String), String> {
    let root = repo_root(repo)?;
    let state_file = state_path(&root)?;
    if !state_file.exists() {
        let configured = config_values(&root, false)?;
        return Ok((
            false,
            format!(
                "WORKTREE HOOKS: not installed\nrepo: {}\nGit hooks: not managed here\ncore.hooksPath: {}\nPreToolUse: plugin-managed and not part of this result\nnext: `agent-on worktree hooks install`\n",
                root.display(),
                if configured.is_empty() {
                    "unset".to_string()
                } else {
                    configured.join(", ")
                }
            ),
        ));
    }
    let state = read_state(&state_file)?;
    let problems = installation_problems(&root, &state)?;
    if problems.is_empty() {
        Ok((
            true,
            format!(
                "WORKTREE HOOKS: healthy\nrepo: {}\nGit hooks: managed here for all worktrees\ncore.hooksPath: {}\npre-commit: active\npre-push: active\nPreToolUse: plugin-managed; not part of this repo-local health result (verify host trust once with `/hooks`)\n",
                root.display(), state.hooks_dir
            ),
        ))
    } else {
        Ok((
            false,
            format!(
                "WORKTREE HOOKS: unhealthy\nrepo: {}\n- {}\nnext: restore the reported managed files/config, or remove them manually after review; Agent-On will not overwrite drift\n",
                root.display(),
                problems.join("\n- ")
            ),
        ))
    }
}

pub fn uninstall(repo: &Path) -> (i32, String) {
    match uninstall_inner(repo) {
        Ok(message) => (0, message),
        Err(error) => (
            1,
            format!(
                "ERROR: cannot uninstall Agent-On worktree hooks safely: {error}\nno changed or unknown hook file was removed\n"
            ),
        ),
    }
}

pub fn uninstall_with_schedule(repo: &Path) -> (i32, String) {
    let root = match repo_root(repo) {
        Ok(root) => root,
        Err(error) => {
            let (schedule_code, schedule_output) = crate::worktree_schedule::run_uninstall(repo);
            let result = if schedule_code == 0 {
                "RESULT: PARTIAL (scheduler handled; Git store unknown)"
            } else {
                "RESULT: FAIL (scheduler cleanup failed; Git store unknown)"
            };
            return (
                if schedule_code == 0 { 1 } else { schedule_code },
                format!(
                    "Git hooks: unavailable — {error}\nGit hook store was not changed; it may have been deleted or moved and Agent-On will not guess.\n{schedule_output}{result}\n"
                ),
            );
        }
    };
    if let Err(error) = uninstall_preflight(&root) {
        return (
            1,
            format!(
                "ERROR: uninstall preflight failed for Git hooks: {error}\nGit hooks and daily GC schedule were left unchanged.\n"
            ),
        );
    }
    // A scheduler cannot join a filesystem transaction. Remove it first only
    // after both sides pass drift checks; if it fails, Git hooks remain active.
    // The Git phase repeats its own preflight immediately before removal.
    let (schedule_code, mut output) = crate::worktree_schedule::run_uninstall(&root);
    if schedule_code != 0 {
        output.push_str("Git hooks were left installed because daily GC uninstall failed.\n");
        return (schedule_code, output);
    }
    let (hook_code, hook_output) = uninstall(&root);
    output.push_str(&hook_output);
    if hook_code != 0 {
        output.push_str(
            "Daily GC was removed, but a concurrent Git-hook change prevented the second phase; review the exact hook diagnostic above.\n",
        );
    }
    (hook_code, output)
}

fn uninstall_inner(repo: &Path) -> Result<String, String> {
    let root = repo_root(repo)?;
    let state_file = state_path(&root)?;
    let Some(_state) = uninstall_preflight(&root)? else {
        return Ok(format!(
            "WORKTREE HOOKS: already uninstalled\nrepo: {}\n",
            root.display()
        ));
    };

    let directory = hooks_dir(&root)?;
    unset_hooks_path(&root)?;
    for name in HOOK_NAMES {
        fs::remove_file(directory.join(name))
            .map_err(|e| format!("remove managed {name} hook: {e}"))?;
    }
    fs::remove_file(&state_file).map_err(|e| format!("remove {}: {e}", state_file.display()))?;
    let _ = fs::remove_dir(&directory);
    Ok(format!(
        "WORKTREE HOOKS: uninstalled\nrepo: {}\nuser hook files: untouched\nPreToolUse: plugin-managed and unchanged\n",
        root.display()
    ))
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn primary_worktree(cwd: &Path) -> Result<PathBuf, String> {
    let raw = git(cwd, &["worktree", "list", "--porcelain"])?;
    raw.lines()
        .find_map(|line| line.strip_prefix("worktree "))
        .map(PathBuf::from)
        .ok_or_else(|| "git worktree list returned no primary worktree".to_string())
}

fn shared_repo_root(cwd: &Path) -> Result<PathBuf, String> {
    primary_worktree(cwd).map(|path| canonical(&path))
}

fn active_lanes(cwd: &Path, primary: &Path) -> Result<Vec<LaneSummary>, String> {
    let directory = managed_root(cwd)?.join("lanes");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut lanes = Vec::new();
    for entry in fs::read_dir(&directory)
        .map_err(|e| format!("read lane registry {}: {e}", directory.display()))?
    {
        let path = entry
            .map_err(|e| format!("read lane registry entry: {e}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("read lane record {}: {e}", path.display()))?;
        let lane: LaneSummary = serde_json::from_str(&raw)
            .map_err(|e| format!("parse lane record {}: {e}", path.display()))?;
        if matches!(lane.status.as_str(), "active" | "blocked" | "ready")
            && canonical(Path::new(&lane.worktree)) != canonical(primary)
        {
            lanes.push(lane);
        }
    }
    lanes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(lanes)
}

fn git_path(cwd: &Path, name: &str) -> Result<PathBuf, String> {
    let raw = PathBuf::from(git(cwd, &["rev-parse", "--git-path", name])?);
    Ok(if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    })
}

fn control_operation_in_progress(cwd: &Path) -> Result<Option<&'static str>, String> {
    for (name, label) in [
        ("MERGE_HEAD", "merge"),
        ("SQUASH_MSG", "squash merge"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("REVERT_HEAD", "revert"),
        ("rebase-merge", "rebase"),
        ("rebase-apply", "rebase/am"),
        ("sequencer", "sequencer"),
    ] {
        if git_path(cwd, name)?.exists() {
            return Ok(Some(label));
        }
    }
    Ok(None)
}

fn primary_control_guard_inner(repo: &Path) -> Result<Option<String>, String> {
    let current = canonical(&repo_root(repo)?);
    let primary = canonical(&primary_worktree(&current)?);
    if current != primary {
        return Ok(None);
    }
    let lanes = active_lanes(&current, &primary)?;
    if lanes.is_empty() || control_operation_in_progress(&current)?.is_some() {
        return Ok(None);
    }
    let summary = lanes
        .iter()
        .map(|lane| format!("{} [{}]", lane.id, lane.status))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Some(format!(
        "primary worktree is a control track while live execution lanes exist: {summary}\nmove ordinary changes to a claimed worktree, or finish/park the live lanes first. Merge, squash-merge, cherry-pick, revert, and rebase control commits are detected and allowed automatically"
    )))
}

fn clear_inherited_git_local_env() {
    // Git exports repository-local variables to hooks. They are valid for the
    // invoking worktree only and can poison the all-worktree audit (notably a
    // linked worktree's relative GIT_INDEX_FILE). Git itself recommends
    // clearing `rev-parse --local-env-vars` before operating on another tree.
    for key in [
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
        "GIT_OBJECT_DIRECTORY",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_IMPLICIT_WORK_TREE",
        "GIT_GRAFT_FILE",
        "GIT_INDEX_FILE",
        "GIT_NO_REPLACE_OBJECTS",
        "GIT_REPLACE_REF_BASE",
        "GIT_PREFIX",
        "GIT_SHALLOW_FILE",
        "GIT_COMMON_DIR",
    ] {
        std::env::remove_var(key);
    }
}

/// Shared fail-closed gate for Git hooks and AI-tool PreToolUse guards.
///
/// Exit 0 allows the operation. A non-zero result covers both a policy block
/// and an inability to prove safety; the returned detail is ready for a hook
/// protocol to wrap and report.
pub(crate) fn primary_control_guard(repo: &Path) -> (i32, String) {
    match primary_control_guard_inner(repo) {
        Ok(Some(reason)) => (1, reason),
        Ok(None) => (0, String::new()),
        Err(error) => (
            1,
            format!(
                "cannot verify the primary control-track boundary: {error}\nnext: run `agent-on worktree hooks status` and `agent-on worktree status`"
            ),
        ),
    }
}

pub fn run_hook(repo: &Path, hook: &str) -> (i32, String) {
    if !HOOK_NAMES.contains(&hook) {
        return (
            2,
            format!(
                "ERROR: unknown managed hook {hook}; expected {}\n",
                HOOK_NAMES.join("|")
            ),
        );
    }

    clear_inherited_git_local_env();

    let (check_code, check_output) = crate::worktree::run_audit(repo, false, true);
    if check_code != 0 {
        return (
            1,
            format!(
                "BLOCKED by Agent-On {hook}: worktree check failed\n{check_output}next: run `agent-on worktree status` and fix the reported lane registration/boundary problem\n"
            ),
        );
    }
    if hook == "pre-commit" {
        let (guard_code, reason) = primary_control_guard(repo);
        if guard_code != 0 {
            return (1, format!("BLOCKED by Agent-On pre-commit: {reason}\n"));
        }
    }
    (0, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn run(cwd: &Path, args: &[&str]) {
        let output = Command::new(args[0])
            .current_dir(cwd)
            .args(&args[1..])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn fixture() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        fs::create_dir_all(root.join("app")).unwrap();
        run(&root, &["git", "init", "-b", "main"]);
        run(&root, &["git", "config", "user.email", "test@example.com"]);
        run(&root, &["git", "config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "root\n").unwrap();
        fs::write(root.join("app/base.txt"), "base\n").unwrap();
        run(&root, &["git", "add", "."]);
        run(&root, &["git", "commit", "-m", "init"]);
        (tmp, root)
    }

    #[test]
    fn shell_quote_handles_single_quotes() {
        assert_eq!(shell_quote("a'b"), "'a'\"'\"'b'");
    }

    #[test]
    fn legacy_codex_python_wrapper_is_advisory_only() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".codex")).unwrap();
        fs::write(
            tmp.path().join(".codex/hooks.json"),
            r#"{"command":"python3 /tmp/agent-on-git-guard.sh"}"#,
        )
        .unwrap();
        let warning = legacy_codex_hook_warning(Some(tmp.path().as_os_str())).unwrap();
        assert!(warning.contains("WARN PreToolUse"), "{warning}");
        assert!(
            warning.contains("Git-hook health above is unaffected"),
            "{warning}"
        );
    }

    #[test]
    fn install_is_idempotent_and_uninstall_is_scoped() {
        let (_tmp, root) = fixture();
        let (code, out) = install(&root);
        assert_eq!(code, 0, "{out}");
        let (code, out) = install(&root);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("already installed and healthy"), "{out}");
        assert_eq!(status(&root).0, 0);

        let directory = hooks_dir(&root).unwrap();
        fs::write(directory.join("user-note"), "keep\n").unwrap();
        let (code, out) = uninstall(&root);
        assert_eq!(code, 0, "{out}");
        assert!(directory.join("user-note").exists());
        assert!(config_values(&root, true).unwrap().is_empty());
        assert_eq!(uninstall(&root).0, 0);
    }

    #[test]
    fn install_never_bypasses_existing_user_hook() {
        let (_tmp, root) = fixture();
        let hook = common_git_dir(&root).unwrap().join("hooks/pre-commit");
        fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
        let before = fs::read_to_string(&hook).unwrap();

        let (code, out) = install(&root);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("would be bypassed"), "{out}");
        assert_eq!(fs::read_to_string(&hook).unwrap(), before);
        assert!(config_values(&root, true).unwrap().is_empty());
    }

    #[test]
    fn install_never_replaces_existing_hooks_path() {
        let (tmp, root) = fixture();
        let existing = tmp.path().join("user-hooks");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("pre-push"), "#!/bin/sh\nexit 0\n").unwrap();
        run(
            &root,
            &[
                "git",
                "config",
                "--local",
                "core.hooksPath",
                existing.to_str().unwrap(),
            ],
        );

        let (code, out) = install(&root);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("already configured"), "{out}");
        assert_eq!(
            config_values(&root, true).unwrap(),
            [existing.display().to_string()]
        );
        assert_eq!(
            fs::read_to_string(existing.join("pre-push")).unwrap(),
            "#!/bin/sh\nexit 0\n"
        );
    }

    #[test]
    fn install_detects_linked_worktree_scope_hooks_path() {
        let (tmp, root) = fixture();
        let linked = tmp.path().join("linked");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "linked",
                linked.to_str().unwrap(),
                "main",
            ],
        );
        run(
            &root,
            &["git", "config", "extensions.worktreeConfig", "true"],
        );
        let bypass = tmp.path().join("bypass-hooks");
        fs::create_dir_all(&bypass).unwrap();
        run(
            &linked,
            &[
                "git",
                "config",
                "--worktree",
                "core.hooksPath",
                bypass.to_str().unwrap(),
            ],
        );

        let (code, out) = install(&root);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("worktree-scope="), "{out}");
        assert!(out.contains(linked.to_str().unwrap()), "{out}");
        assert_eq!(
            worktree_scope_values(&linked, true).unwrap(),
            [bypass.display().to_string()]
        );
        assert!(config_values(&root, true).unwrap().is_empty());
    }

    #[test]
    fn status_and_uninstall_fail_closed_after_linked_override() {
        let (tmp, root) = fixture();
        let linked = tmp.path().join("linked");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "linked",
                linked.to_str().unwrap(),
                "main",
            ],
        );
        run(
            &root,
            &["git", "config", "extensions.worktreeConfig", "true"],
        );
        assert_eq!(install(&root).0, 0);
        let bypass = tmp.path().join("bypass-hooks");
        fs::create_dir_all(&bypass).unwrap();
        run(
            &linked,
            &[
                "git",
                "config",
                "--worktree",
                "core.hooksPath",
                bypass.to_str().unwrap(),
            ],
        );

        let (status_code, status_out) = status(&root);
        assert_eq!(status_code, 1, "{status_out}");
        assert!(
            status_out.contains("can bypass the shared guard"),
            "{status_out}"
        );
        let (uninstall_code, uninstall_out) = uninstall(&root);
        assert_eq!(uninstall_code, 1, "{uninstall_out}");
        assert!(
            uninstall_out.contains("can bypass the shared guard"),
            "{uninstall_out}"
        );
        assert!(state_path(&root).unwrap().exists());
        assert_eq!(
            config_values(&root, true).unwrap(),
            [hooks_dir(&root).unwrap().display().to_string()]
        );
    }

    #[test]
    fn uninstall_refuses_changed_managed_hook() {
        let (_tmp, root) = fixture();
        assert_eq!(install(&root).0, 0);
        let hook = hooks_dir(&root).unwrap().join("pre-commit");
        fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();

        let (code, out) = uninstall(&root);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("changed after installation"), "{out}");
        assert!(hook.exists());
        assert!(!config_values(&root, true).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn status_requires_installed_cli_to_remain_executable() {
        let (tmp, root) = fixture();
        assert_eq!(install(&root).0, 0);
        let fake_executable = tmp.path().join("agent-on-not-executable");
        fs::write(&fake_executable, "binary placeholder\n").unwrap();
        let path = state_path(&root).unwrap();
        let mut state = read_state(&path).unwrap();
        state.executable = fake_executable.display().to_string();
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&state).unwrap()),
        )
        .unwrap();

        let (code, out) = status(&root);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("is not executable"), "{out}");
    }

    #[test]
    fn primary_guard_blocks_business_commit_but_allows_squash_marker() {
        let (tmp, root) = fixture();
        let lane = tmp.path().join("lane");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "lane/a",
                lane.to_str().unwrap(),
                "main",
            ],
        );
        let claim = crate::worktree::ClaimOpts {
            parked: false,
            id: "lane-a".to_string(),
            goal: "change app".to_string(),
            base: Some("main".to_string()),
            owns: vec!["app".to_string()],
            depends_on: Vec::new(),
        };
        assert_eq!(crate::worktree::claim_lane(&lane, &claim).0, 0);

        let blocked = primary_control_guard_inner(&root).unwrap().unwrap();
        assert!(blocked.contains("lane-a [active]"), "{blocked}");
        fs::write(git_path(&root, "SQUASH_MSG").unwrap(), "squash\n").unwrap();
        assert!(primary_control_guard_inner(&root).unwrap().is_none());
    }
}
