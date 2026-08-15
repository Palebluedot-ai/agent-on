//! Optional, report-only daily worktree GC scheduling.
//!
//! This module deliberately owns no reclaim logic.  It only installs a user-level
//! scheduler entry which invokes the existing fail-closed command:
//! `agent-on worktree gc --dry-run --json --repo <repo>`.

use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DAILY_HOUR: u8 = 3;
const DAILY_MINUTE: u8 = 30;
const MANAGED_MARKER: &str = "agent-on managed worktree GC schedule v1";
const INSTALL_STATE_VERSION: u8 = 1;
const INSTALL_STATE_NAME: &str = "install-state.json";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SchedulerKind {
    Launchd,
    SystemdUser,
    Unsupported,
}

#[derive(Clone, Debug)]
struct ScheduleSpec {
    kind: SchedulerKind,
    repo: PathBuf,
    executable: PathBuf,
    home: PathBuf,
    path_env: String,
    key: String,
    state_dir: PathBuf,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
    launchd_label: String,
    launchd_plist: PathBuf,
    systemd_service_name: String,
    systemd_timer_name: String,
    systemd_service: PathBuf,
    systemd_timer: PathBuf,
}

#[derive(Clone, Debug)]
struct RenderedFiles {
    files: Vec<(PathBuf, String)>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct GitIdentity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct InstalledFile {
    path: String,
    contents: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct InstallState {
    version: u8,
    kind: SchedulerKind,
    repo_at_install: String,
    git_identity: Option<GitIdentity>,
    key: String,
    state_dir: String,
    stdout_log: String,
    stderr_log: String,
    launchd_label: String,
    launchd_plist: String,
    systemd_timer_name: String,
    command: Vec<String>,
    files: Vec<InstalledFile>,
}

impl InstallState {
    fn from_spec(spec: &ScheduleSpec) -> Result<Self, String> {
        let rendered = spec.rendered_files();
        Ok(Self {
            version: INSTALL_STATE_VERSION,
            kind: spec.kind,
            repo_at_install: path_text(&spec.repo).to_string(),
            git_identity: git_identity(&spec.repo)?,
            key: spec.key.clone(),
            state_dir: path_text(&spec.state_dir).to_string(),
            stdout_log: path_text(&spec.stdout_log).to_string(),
            stderr_log: path_text(&spec.stderr_log).to_string(),
            launchd_label: spec.launchd_label.clone(),
            launchd_plist: path_text(&spec.launchd_plist).to_string(),
            systemd_timer_name: spec.systemd_timer_name.clone(),
            command: gc_args(spec),
            files: rendered
                .files
                .into_iter()
                .map(|(path, contents)| InstalledFile {
                    path: path_text(&path).to_string(),
                    contents,
                })
                .collect(),
        })
    }

    fn state_dir(&self) -> PathBuf {
        PathBuf::from(&self.state_dir)
    }

    fn state_file(&self) -> PathBuf {
        self.state_dir().join(INSTALL_STATE_NAME)
    }

    fn stdout_log(&self) -> PathBuf {
        PathBuf::from(&self.stdout_log)
    }

    fn stderr_log(&self) -> PathBuf {
        PathBuf::from(&self.stderr_log)
    }

    fn rendered_files(&self) -> Vec<(PathBuf, String)> {
        self.files
            .iter()
            .map(|file| (PathBuf::from(&file.path), file.contents.clone()))
            .collect()
    }
}

/// Machine-readable state used by `worktree hooks status`.
///
/// `Absent` is intentionally distinct from a failure because daily GC is opt-in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleState {
    Active,
    Absent,
    Drifted,
    Inactive,
    Unsupported,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleStatus {
    pub state: ScheduleState,
    pub installed: bool,
    pub active: bool,
    pub detail: String,
    pub reports_path: Option<PathBuf>,
    pub errors_path: Option<PathBuf>,
}

impl ScheduleStatus {
    pub fn healthy(&self) -> bool {
        self.state == ScheduleState::Active
    }
}

/// Install or idempotently refresh the daily report-only audit for `repo`.
///
/// Returns a process-style exit code and a complete user-facing message.  The
/// caller should print successful output to stdout and failures to stderr.
pub fn run_install(repo: &Path) -> (i32, String) {
    let result = match find_install_state(repo) {
        Ok(Some(state)) => reinstall_existing(&state),
        Ok(None) => ScheduleSpec::from_runtime(repo, true).and_then(|spec| install_new(&spec)),
        Err(error) => Err(error),
    };
    result_to_output(result)
}

/// Disable the exact per-repository scheduler entry and remove only its scheduler
/// files.  Historical reports are intentionally retained in the state directory.
pub fn run_uninstall(repo: &Path) -> (i32, String) {
    let result = match find_install_state(repo) {
        Ok(Some(state)) => uninstall_installed(&state),
        Ok(None) => Ok(format!(
            "worktree GC schedule: already absent\nrepo: {}\nNo scheduler, worktree, branch, or report was changed.\n",
            absolute_normalized(repo)
                .unwrap_or_else(|_| repo.to_path_buf())
                .display()
        )),
        Err(error) => Err(error),
    };
    result_to_output(result)
}

fn result_to_output(result: Result<String, String>) -> (i32, String) {
    match result {
        Ok(message) => (0, message),
        Err(error) => (2, format!("ERROR: {error}\n")),
    }
}

impl ScheduleSpec {
    fn from_runtime(repo: &Path, require_repo: bool) -> Result<Self, String> {
        let kind = current_scheduler();
        if kind == SchedulerKind::Unsupported {
            return Err(format!(
                "daily worktree GC scheduling is unsupported on {}; supported: macOS launchd and Linux systemd --user. No files were changed.",
                env::consts::OS
            ));
        }
        let home = env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                "HOME is unset; cannot resolve a user-level scheduler directory. No files were changed."
                    .to_string()
            })?;
        let state_base = env::var_os("XDG_STATE_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("state"));
        let config_base = env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let executable = env::current_exe()
            .map_err(|error| format!("cannot resolve the current agent-on executable: {error}"))?;
        let path_env = env::var("PATH").unwrap_or_else(|_| {
            "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".to_string()
        });
        let repo = resolve_primary_worktree(repo, require_repo)?;
        Self::new(
            kind,
            &repo,
            require_repo,
            home,
            state_base,
            config_base,
            executable,
            path_env,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        kind: SchedulerKind,
        repo: &Path,
        require_repo: bool,
        home: PathBuf,
        state_base: PathBuf,
        config_base: PathBuf,
        executable: PathBuf,
        path_env: String,
    ) -> Result<Self, String> {
        if kind == SchedulerKind::Unsupported {
            return Err("unsupported scheduler".to_string());
        }
        let repo = absolute_normalized(repo)?;
        let home = absolute_normalized(&home)?;
        let state_base = absolute_normalized(&state_base)?;
        let config_base = absolute_normalized(&config_base)?;
        if require_repo && !repo.join(".git").exists() {
            return Err(format!(
                "{} is not a Git worktree root (.git is missing). No scheduler was installed.",
                repo.display()
            ));
        }
        let executable = absolute_normalized(&executable)?;
        if require_repo && !executable.is_file() {
            return Err(format!(
                "agent-on executable does not exist at {}. Re-run install from the installed CLI.",
                executable.display()
            ));
        }
        for (label, value) in [
            ("repo", repo.as_path()),
            ("executable", executable.as_path()),
            ("HOME", home.as_path()),
            ("state directory", state_base.as_path()),
            ("config directory", config_base.as_path()),
        ] {
            require_safe_utf8_path(label, value)?;
        }
        if path_env.chars().any(char::is_control) {
            return Err(
                "PATH contains a control character; refusing to write scheduler configuration"
                    .to_string(),
            );
        }

        let key = repo_key(&repo);
        let state_dir = state_base.join("agent-on").join("worktree-gc").join(&key);
        let stdout_log = state_dir.join("reports.log");
        let stderr_log = state_dir.join("errors.log");
        let launchd_label = format!("io.agent-on.worktree-gc.{key}");
        let launchd_plist = home
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{launchd_label}.plist"));
        let systemd_service_name = format!("agent-on-worktree-gc-{key}.service");
        let systemd_timer_name = format!("agent-on-worktree-gc-{key}.timer");
        let systemd_user_dir = config_base.join("systemd").join("user");
        let systemd_service = systemd_user_dir.join(&systemd_service_name);
        let systemd_timer = systemd_user_dir.join(&systemd_timer_name);

        let scheduler_files = [&launchd_plist, &systemd_service, &systemd_timer];
        if state_dir.starts_with(&repo)
            || scheduler_files.iter().any(|path| path.starts_with(&repo))
        {
            return Err(format!(
                "local scheduler state/config resolves inside repository {}; set HOME/XDG_STATE_HOME/XDG_CONFIG_HOME to user-local directories outside the repo. No files were changed.",
                repo.display()
            ));
        }

        Ok(Self {
            kind,
            repo,
            executable,
            home,
            path_env,
            key,
            state_dir,
            stdout_log,
            stderr_log,
            launchd_label,
            launchd_plist,
            systemd_service_name,
            systemd_timer_name,
            systemd_service,
            systemd_timer,
        })
    }

    fn rendered_files(&self) -> RenderedFiles {
        match self.kind {
            SchedulerKind::Launchd => RenderedFiles {
                files: vec![(self.launchd_plist.clone(), render_launchd(self))],
            },
            SchedulerKind::SystemdUser => RenderedFiles {
                files: vec![
                    (self.systemd_service.clone(), render_systemd_service(self)),
                    (self.systemd_timer.clone(), render_systemd_timer(self)),
                ],
            },
            SchedulerKind::Unsupported => RenderedFiles { files: Vec::new() },
        }
    }
}

fn current_scheduler() -> SchedulerKind {
    if cfg!(target_os = "macos") {
        SchedulerKind::Launchd
    } else if cfg!(target_os = "linux") {
        SchedulerKind::SystemdUser
    } else {
        SchedulerKind::Unsupported
    }
}

fn runtime_state_root() -> Result<PathBuf, String> {
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            "HOME is unset; cannot resolve persisted daily GC install state".to_string()
        })?;
    let state_base = env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("state"));
    absolute_normalized(&state_base.join("agent-on").join("worktree-gc"))
}

fn find_install_state(repo: &Path) -> Result<Option<InstallState>, String> {
    let state_root = runtime_state_root()?;
    find_install_state_in_root(repo, &state_root)
}

fn find_install_state_in_root(
    repo: &Path,
    state_root: &Path,
) -> Result<Option<InstallState>, String> {
    if !state_root.is_dir() {
        return Ok(None);
    }
    let raw_repo = absolute_normalized(repo)?;
    let mut lookup_errors = Vec::new();
    let live_identity = if raw_repo.exists() {
        match git_identity(&raw_repo) {
            Ok(identity) => identity,
            Err(error) => {
                lookup_errors.push(error);
                None
            }
        }
    } else {
        None
    };
    let primary = if raw_repo.exists() {
        match resolve_primary_worktree(&raw_repo, false) {
            Ok(primary) => Some(primary),
            Err(error) => {
                lookup_errors.push(error);
                None
            }
        }
    } else {
        None
    };

    let mut lookup_paths = vec![raw_repo.clone()];
    if let Some(primary) = primary {
        if !lookup_paths.contains(&primary) {
            lookup_paths.push(primary);
        }
    }
    for path in &lookup_paths {
        let candidate = state_root.join(repo_key(path)).join(INSTALL_STATE_NAME);
        if candidate.is_file() {
            return read_install_state_in_root(&candidate, state_root).map(Some);
        }
    }

    let mut manifests = fs::read_dir(state_root)
        .map_err(|error| format!("cannot scan {}: {error}", state_root.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(INSTALL_STATE_NAME))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    manifests.sort();
    let mut matches = Vec::new();
    for manifest in manifests {
        let Ok(raw) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(peek) = serde_json::from_str::<InstallState>(&raw) else {
            continue;
        };
        let path_match = lookup_paths
            .iter()
            .any(|path| path_text(path) == peek.repo_at_install);
        let identity_match = live_identity
            .as_ref()
            .zip(peek.git_identity.as_ref())
            .is_some_and(|(live, installed)| live == installed);
        if path_match || identity_match {
            matches.push(read_install_state_in_root(&manifest, state_root)?);
        }
    }
    match matches.len() {
        0 if lookup_errors.is_empty() => Ok(None),
        0 => Err(format!(
            "cannot safely match {} to persisted daily GC state: {}",
            raw_repo.display(),
            lookup_errors.join("; ")
        )),
        1 => Ok(matches.pop()),
        _ => Err(format!(
            "multiple persisted daily GC installations match {}: {}. Inspect the exact state directories before continuing.",
            raw_repo.display(),
            matches
                .iter()
                .map(|state| state.state_file().display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn write_install_state(state: &InstallState) -> Result<(), String> {
    let path = state.state_file();
    if path.exists() {
        return Err(format!(
            "persisted install state already exists at {}; inspect it before retrying",
            path.display()
        ));
    }
    let raw = serialize_install_state(state)?;
    atomic_write_private(&path, raw.as_bytes()).map_err(|error| {
        format!(
            "cannot write persisted install state {}: {error}",
            path.display()
        )
    })
}

fn serialize_install_state(state: &InstallState) -> Result<String, String> {
    serde_json::to_string_pretty(state)
        .map(|raw| format!("{raw}\n"))
        .map_err(|error| format!("cannot serialize daily GC install state: {error}"))
}

fn read_install_state_in_root(path: &Path, state_root: &Path) -> Result<InstallState, String> {
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read persisted install state {}: {error}",
            path.display()
        )
    })?;
    let state: InstallState = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "cannot parse persisted install state {}: {error}",
            path.display()
        )
    })?;
    validate_install_state(path, &state, state_root)?;
    let canonical = serialize_install_state(&state)?;
    if raw != canonical {
        return Err(format!(
            "persisted install state formatting/content drifted at {}; inspect it manually. No scheduler file was changed.",
            path.display()
        ));
    }
    Ok(state)
}

fn validate_install_state(
    path: &Path,
    state: &InstallState,
    state_root: &Path,
) -> Result<(), String> {
    if state.version != INSTALL_STATE_VERSION {
        return Err(format!(
            "unsupported daily GC install-state version {} at {} (expected {})",
            state.version,
            path.display(),
            INSTALL_STATE_VERSION
        ));
    }
    if state.kind != current_scheduler() {
        return Err(format!(
            "persisted scheduler platform {:?} does not match current platform {:?} at {}",
            state.kind,
            current_scheduler(),
            path.display()
        ));
    }
    let expected_dir = state_root.join(&state.key);
    if Path::new(&state.state_dir) != expected_dir || path != expected_dir.join(INSTALL_STATE_NAME)
    {
        return Err(format!(
            "persisted install state points outside its expected directory {}",
            expected_dir.display()
        ));
    }
    if repo_key(Path::new(&state.repo_at_install)) != state.key {
        return Err(format!(
            "persisted install key does not match repository path in {}",
            path.display()
        ));
    }
    if Path::new(&state.stdout_log) != expected_dir.join("reports.log")
        || Path::new(&state.stderr_log) != expected_dir.join("errors.log")
    {
        return Err(format!(
            "persisted log paths are invalid in {}",
            path.display()
        ));
    }
    if state.command.len() != 7
        || !state.command[1..6].iter().map(String::as_str).eq([
            "worktree",
            "gc",
            "--dry-run",
            "--json",
            "--repo",
        ])
        || state.command.last().map(String::as_str) != Some(state.repo_at_install.as_str())
        || !Path::new(&state.command[0]).is_absolute()
    {
        return Err(format!(
            "persisted command is not the fixed report-only GC command in {}",
            path.display()
        ));
    }
    if state.files.is_empty()
        || state.files.iter().any(|file| {
            !Path::new(&file.path).is_absolute() || !file.contents.contains(MANAGED_MARKER)
        })
    {
        return Err(format!(
            "persisted scheduler file inventory is invalid in {}",
            path.display()
        ));
    }
    let expected_label = format!("io.agent-on.worktree-gc.{}", state.key);
    let expected_timer = format!("agent-on-worktree-gc-{}.timer", state.key);
    if state.launchd_label != expected_label || state.systemd_timer_name != expected_timer {
        return Err(format!(
            "persisted scheduler activation identifiers are invalid in {}",
            path.display()
        ));
    }
    Ok(())
}

fn remove_install_state_if_exact(state: &InstallState) -> Result<(), String> {
    let path = state.state_file();
    let actual = fs::read_to_string(&path)
        .map_err(|error| format!("cannot re-read persisted state {}: {error}", path.display()))?;
    let expected = serialize_install_state(state)?;
    if actual != expected {
        return Err(format!(
            "scheduler files were disabled, but persisted state changed during uninstall and was retained at {}",
            path.display()
        ));
    }
    fs::remove_file(&path).map_err(|error| {
        format!(
            "cannot remove exact persisted state {}: {error}",
            path.display()
        )
    })
}

fn install_new(spec: &ScheduleSpec) -> Result<String, String> {
    let state = InstallState::from_spec(spec)?;
    let rendered = state.rendered_files();
    let foreign = rendered
        .iter()
        .filter_map(|(path, expected)| match fs::read_to_string(path) {
            Ok(actual) if actual == *expected || actual.contains(MANAGED_MARKER) => None,
            Ok(_) => Some(format!("{} (not Agent-On managed)", path.display())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => Some(format!("{} ({error})", path.display())),
        })
        .collect::<Vec<_>>();
    if !foreign.is_empty() {
        return Err(format!(
            "refusing to overwrite existing scheduler assets: {}. Inspect those exact paths and compose/remove them explicitly, then retry. No scheduler file was changed.",
            foreign.join(", ")
        ));
    }

    fs::create_dir_all(state.state_dir()).map_err(|error| {
        format!(
            "cannot create local state directory {}: {error}",
            state.state_dir().display()
        )
    })?;
    set_private_dir_permissions(&state.state_dir()).map_err(|error| {
        format!(
            "cannot secure local state directory {}: {error}",
            state.state_dir().display()
        )
    })?;
    write_install_state(&state)?;
    for (path, contents) in &rendered {
        atomic_write(path, contents.as_bytes()).map_err(|error| {
            format!(
                "cannot write scheduler file {}: {error}. Fix permissions, then run install again.",
                path.display()
            )
        })?;
    }
    activate_installed(&state)?;

    Ok(installed_message(&state, false))
}

fn reinstall_existing(state: &InstallState) -> Result<String, String> {
    let report = inspect_installed(state);
    match report.state {
        ScheduleState::Active => Ok(installed_message(state, true)),
        ScheduleState::Inactive => {
            activate_installed(state)?;
            Ok(installed_message(state, true))
        }
        _ => Err(format!(
            "existing daily GC installation is not safe to refresh: {}. Resolve the exact persisted-state/config issue, then retry; current PATH and executable were not substituted.",
            report.detail
        )),
    }
}

fn installed_message(state: &InstallState, existing: bool) -> String {
    format!(
        "worktree GC schedule: {}\nrepo: {}\nschedule: daily {:02}:{:02}\ncommand: {}\nreports: {}\nerrors: {}\nsafety: report-only; this scheduler has no delete mode\n",
        if existing {
            "already installed and active"
        } else {
            "installed and active"
        },
        state.repo_at_install,
        DAILY_HOUR,
        DAILY_MINUTE,
        display_string_command(&state.command),
        state.stdout_log,
        state.stderr_log
    )
}

/// Inspect the optional scheduler without changing its configuration.
///
/// An absent schedule is returned as `ScheduleState::Absent`, not as an error,
/// so the enclosing hooks status can remain healthy when the user did not opt in.
pub fn inspect(repo: &Path) -> ScheduleStatus {
    if current_scheduler() == SchedulerKind::Unsupported {
        return ScheduleStatus {
            state: ScheduleState::Unsupported,
            installed: false,
            active: false,
            detail: format!(
                "unsupported on {}; supported: macOS launchd and Linux systemd --user",
                env::consts::OS
            ),
            reports_path: None,
            errors_path: None,
        };
    }
    match find_install_state(repo) {
        Ok(Some(state)) => inspect_installed(&state),
        Ok(None) => {
            let repo = resolve_primary_worktree(repo, false)
                .or_else(|_| absolute_normalized(repo))
                .unwrap_or_else(|_| repo.to_path_buf());
            ScheduleStatus {
                state: ScheduleState::Absent,
                installed: false,
                active: false,
                detail: format!(
                    "enable with `agent-on worktree hooks install --daily-gc --repo {}`",
                    shell_hint(&repo)
                ),
                reports_path: None,
                errors_path: None,
            }
        }
        Err(error) => ScheduleStatus {
            state: ScheduleState::Error,
            installed: true,
            active: false,
            detail: error,
            reports_path: None,
            errors_path: None,
        },
    }
}

/// Render a compact line suitable for inclusion in `worktree hooks status`.
pub fn render_status(report: &ScheduleStatus) -> String {
    let state = match report.state {
        ScheduleState::Active => "active",
        ScheduleState::Absent => "not installed (optional)",
        ScheduleState::Drifted => "configuration drifted",
        ScheduleState::Inactive => "installed but inactive",
        ScheduleState::Unsupported => "unsupported",
        ScheduleState::Error => "error",
    };
    format!("daily report-only GC: {state} — {}", report.detail)
}

fn inspect_installed(state: &InstallState) -> ScheduleStatus {
    let installed_repo = Path::new(&state.repo_at_install);
    let repo_problem = if !installed_repo.exists() {
        Some("repository moved or was deleted".to_string())
    } else if let Some(expected) = state.git_identity.as_ref() {
        match git_identity(installed_repo) {
            Ok(Some(actual)) if &actual == expected => None,
            Ok(_) => {
                Some("repository path now identifies a different Git common directory".to_string())
            }
            Err(error) => Some(format!(
                "repository identity is no longer readable: {error}"
            )),
        }
    } else {
        None
    };
    if let Some(problem) = repo_problem {
        return ScheduleStatus {
            state: ScheduleState::Drifted,
            installed: true,
            active: false,
            detail: format!(
                "{problem}; persisted scheduler remains at {} and can be removed with hooks uninstall using the current/moved repo or old path {}",
                state.state_file().display(),
                shell_hint(installed_repo)
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        };
    }
    let executable = Path::new(&state.command[0]);
    if !is_executable_file(executable) {
        return ScheduleStatus {
            state: ScheduleState::Drifted,
            installed: true,
            active: false,
            detail: format!(
                "persisted scheduler executable is missing or not executable at {}; uninstall remains available from {}",
                executable.display(),
                state.state_file().display()
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        };
    }
    let rendered = state.rendered_files();
    let missing: Vec<_> = rendered
        .iter()
        .filter(|(path, _)| !path.is_file())
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !missing.is_empty() {
        return ScheduleStatus {
            state: ScheduleState::Drifted,
            installed: true,
            active: false,
            detail: format!(
                "persisted installation is missing {}; uninstall is still available from state {}",
                missing.join(", "),
                state.state_file().display()
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        };
    }
    let drifted: Vec<_> = rendered
        .iter()
        .filter(|(path, contents)| fs::read_to_string(path).ok().as_deref() != Some(contents))
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !drifted.is_empty() {
        return ScheduleStatus {
            state: ScheduleState::Drifted,
            installed: true,
            active: false,
            detail: format!(
                "content differs from persisted install state: {}; inspect exact files and {}",
                drifted.join(", "),
                state.state_file().display()
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        };
    }
    match scheduler_active(state) {
        Ok(true) => ScheduleStatus {
            state: ScheduleState::Active,
            installed: true,
            active: true,
            detail: format!(
                "daily {:02}:{:02}; reports at {}",
                DAILY_HOUR, DAILY_MINUTE, state.stdout_log
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        },
        Ok(false) => ScheduleStatus {
            state: ScheduleState::Inactive,
            installed: true,
            active: false,
            detail: format!(
                "reactivate with `agent-on worktree hooks install --daily-gc --repo {}`",
                shell_hint(Path::new(&state.repo_at_install))
            ),
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        },
        Err(error) => ScheduleStatus {
            state: ScheduleState::Error,
            installed: true,
            active: false,
            detail: error,
            reports_path: Some(state.stdout_log()),
            errors_path: Some(state.stderr_log()),
        },
    }
}

fn uninstall_installed(state: &InstallState) -> Result<String, String> {
    let rendered = state.rendered_files();
    let drifted = rendered
        .iter()
        .filter_map(|(path, expected)| {
            if !path.exists() {
                return None;
            }
            match fs::read_to_string(path) {
                Ok(actual) if &actual == expected => None,
                Ok(_) => Some(format!("{} (content changed)", path.display())),
                Err(error) => Some(format!("{} ({error})", path.display())),
            }
        })
        .collect::<Vec<_>>();
    if !drifted.is_empty() {
        return Err(format!(
            "refusing to disable or remove changed scheduler assets: {}. Review those exact paths against persisted state {}; restore exact contents or remove them manually after inspection. No scheduler file was deleted.",
            drifted.join(", "),
            state.state_file().display()
        ));
    }

    deactivate_installed(state)?;
    for (path, expected) in rendered {
        match fs::read_to_string(&path) {
            Ok(actual) if actual == expected => fs::remove_file(&path).map_err(|error| {
                format!(
                    "scheduler was disabled, but exact managed file {} could not be removed: {error}. Remove that exact file manually.",
                    path.display()
                )
            })?,
            Ok(_) => {
                return Err(format!(
                    "scheduler was disabled, but {} changed during uninstall and was not removed. Inspect that exact file manually.",
                    path.display()
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "scheduler was disabled, but exact managed file {} could not be read: {error}. Inspect that exact file manually.",
                    path.display()
                ));
            }
        }
    }
    if state.kind == SchedulerKind::SystemdUser {
        run_checked(
            "systemctl",
            &[OsString::from("--user"), OsString::from("daemon-reload")],
            "reload the user systemd manager after uninstall",
        )?;
    }
    remove_install_state_if_exact(state)?;
    Ok(format!(
        "worktree GC schedule: uninstalled\nrepo: {}\nretained reports: {}\nretained errors: {}\nNo worktree, branch, or report was deleted.\n",
        state.repo_at_install,
        state.stdout_log,
        state.stderr_log
    ))
}

fn scheduler_active(state: &InstallState) -> Result<bool, String> {
    match state.kind {
        SchedulerKind::Launchd => {
            let uid = current_uid()?;
            let target = format!("gui/{uid}/{}", state.launchd_label);
            let output = run_output("launchctl", &[OsString::from("print"), target.into()])?;
            Ok(output.status.success())
        }
        SchedulerKind::SystemdUser => {
            let (enabled, active) = systemd_timer_state(state)?;
            Ok(enabled && active)
        }
        SchedulerKind::Unsupported => Ok(false),
    }
}

fn activate_installed(state: &InstallState) -> Result<(), String> {
    match state.kind {
        SchedulerKind::Launchd => {
            let uid = current_uid()?;
            let domain = format!("gui/{uid}");
            let target = format!("{domain}/{}", state.launchd_label);
            // A stale loaded job blocks bootstrap.  bootout is intentionally best-effort:
            // its common non-zero result means simply "not loaded".
            let _ = run_output("launchctl", &[OsString::from("bootout"), target.into()]);
            run_checked(
                "launchctl",
                &[
                    OsString::from("bootstrap"),
                    domain.into(),
                    OsString::from(&state.launchd_plist),
                ],
                &format!(
                    "activate the LaunchAgent; inspect `{}` and retry install",
                    state.launchd_plist
                ),
            )
        }
        SchedulerKind::SystemdUser => {
            run_checked(
                "systemctl",
                &[OsString::from("--user"), OsString::from("daemon-reload")],
                "reload the user systemd manager",
            )?;
            run_checked(
                "systemctl",
                &[
                    OsString::from("--user"),
                    OsString::from("enable"),
                    OsString::from("--now"),
                    OsString::from(&state.systemd_timer_name),
                ],
                &format!(
                    "enable {}; try `systemctl --user status {}`",
                    state.systemd_timer_name, state.systemd_timer_name
                ),
            )
        }
        SchedulerKind::Unsupported => Err("unsupported scheduler".to_string()),
    }
}

fn deactivate_installed(state: &InstallState) -> Result<(), String> {
    match state.kind {
        SchedulerKind::Launchd => {
            if scheduler_active(state)? {
                let uid = current_uid()?;
                let target = format!("gui/{uid}/{}", state.launchd_label);
                run_checked(
                    "launchctl",
                    &[OsString::from("bootout"), target.into()],
                    "disable the LaunchAgent before uninstall; retry uninstall",
                )?;
            }
            Ok(())
        }
        SchedulerKind::SystemdUser => {
            let (enabled, active) = systemd_timer_state(state)?;
            if enabled || active {
                run_checked(
                    "systemctl",
                    &[
                        OsString::from("--user"),
                        OsString::from("disable"),
                        OsString::from("--now"),
                        OsString::from(&state.systemd_timer_name),
                    ],
                    &format!(
                        "disable {}; try `systemctl --user status {}`",
                        state.systemd_timer_name, state.systemd_timer_name
                    ),
                )?;
            }
            Ok(())
        }
        SchedulerKind::Unsupported => Err("unsupported scheduler".to_string()),
    }
}

fn systemd_timer_state(state: &InstallState) -> Result<(bool, bool), String> {
    let enabled = run_output(
        "systemctl",
        &[
            OsString::from("--user"),
            OsString::from("is-enabled"),
            OsString::from(&state.systemd_timer_name),
        ],
    )?;
    let active = run_output(
        "systemctl",
        &[
            OsString::from("--user"),
            OsString::from("is-active"),
            OsString::from(&state.systemd_timer_name),
        ],
    )?;
    Ok((enabled.status.success(), active.status.success()))
}

fn run_output(program: &str, args: &[OsString]) -> Result<Output, String> {
    Command::new(program).args(args).output().map_err(|error| {
        format!(
            "cannot run `{}`: {error}. Ensure the platform user scheduler is available; no worktree was changed.",
            display_os_command(program, args)
        )
    })
}

fn run_checked(program: &str, args: &[OsString], remediation: &str) -> Result<(), String> {
    let output = run_output(program, args)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(format!(
        "`{}` failed (exit {}). {}. {}",
        display_os_command(program, args),
        output.status.code().unwrap_or(-1),
        if detail.is_empty() {
            "No diagnostic output was returned".to_string()
        } else {
            detail
        },
        remediation
    ))
}

fn current_uid() -> Result<String, String> {
    let output = run_output("id", &[OsString::from("-u")])?;
    if !output.status.success() {
        return Err("`id -u` failed; cannot address the per-user launchd domain".to_string());
    }
    let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uid.is_empty() || !uid.chars().all(|value| value.is_ascii_digit()) {
        return Err(format!("`id -u` returned an invalid user id: {uid:?}"));
    }
    Ok(uid)
}

fn render_launchd(spec: &ScheduleSpec) -> String {
    let args = gc_args(spec);
    let mut program_args = String::new();
    for arg in args {
        program_args.push_str("        <string>");
        program_args.push_str(&xml_escape(&arg));
        program_args.push_str("</string>\n");
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<!-- agent-on managed worktree GC schedule v1 -->
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
{program_args}    </array>
    <key>WorkingDirectory</key>
    <string>{repo}</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>GIT_OPTIONAL_LOCKS</key>
        <string>0</string>
        <key>HOME</key>
        <string>{home}</string>
        <key>PATH</key>
        <string>{path_env}</string>
    </dict>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>{hour}</integer>
        <key>Minute</key>
        <integer>{minute}</integer>
    </dict>
    <key>ProcessType</key>
    <string>Background</string>
    <key>LowPriorityIO</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
</dict>
</plist>
"#,
        label = xml_escape(&spec.launchd_label),
        repo = xml_escape(path_text(&spec.repo)),
        home = xml_escape(path_text(&spec.home)),
        path_env = xml_escape(&spec.path_env),
        hour = DAILY_HOUR,
        minute = DAILY_MINUTE,
        stdout = xml_escape(path_text(&spec.stdout_log)),
        stderr = xml_escape(path_text(&spec.stderr_log)),
    )
}

fn render_systemd_service(spec: &ScheduleSpec) -> String {
    let command = gc_args(spec)
        .iter()
        .map(|value| systemd_quote(value))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "# {marker}\n[Unit]\nDescription=Agent-On report-only worktree GC ({key})\n\n[Service]\nType=oneshot\nWorkingDirectory={repo}\nExecStart={command}\nEnvironment={git_locks}\nEnvironment={home}\nEnvironment={path_env}\nStandardOutput={stdout}\nStandardError={stderr}\n",
        marker = MANAGED_MARKER,
        key = spec.key,
        repo = systemd_quote(path_text(&spec.repo)),
        git_locks = systemd_quote("GIT_OPTIONAL_LOCKS=0"),
        home = systemd_quote(&format!("HOME={}", path_text(&spec.home))),
        path_env = systemd_quote(&format!("PATH={}", spec.path_env)),
        stdout = systemd_quote(&format!("append:{}", path_text(&spec.stdout_log))),
        stderr = systemd_quote(&format!("append:{}", path_text(&spec.stderr_log))),
    )
}

fn render_systemd_timer(spec: &ScheduleSpec) -> String {
    format!(
        "# {marker}\n[Unit]\nDescription=Daily Agent-On report-only worktree GC ({key})\n\n[Timer]\nOnCalendar=*-*-* {hour:02}:{minute:02}:00\nPersistent=true\nAccuracySec=5m\nUnit={service}\n\n[Install]\nWantedBy=timers.target\n",
        marker = MANAGED_MARKER,
        key = spec.key,
        hour = DAILY_HOUR,
        minute = DAILY_MINUTE,
        service = spec.systemd_service_name
    )
}

fn gc_args(spec: &ScheduleSpec) -> Vec<String> {
    vec![
        path_text(&spec.executable).to_string(),
        "worktree".to_string(),
        "gc".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
        "--repo".to_string(),
        path_text(&spec.repo).to_string(),
    ]
}

fn display_string_command(args: &[String]) -> String {
    args.iter()
        .map(|value| shell_quote(value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_os_command(program: &str, args: &[OsString]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| arg.to_string_lossy().to_string()))
        .map(|arg| shell_quote(&arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_hint(path: &Path) -> String {
    shell_quote(path_text(path))
}

fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "/._:@%+=,-".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn systemd_quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}

fn path_text(path: &Path) -> &str {
    path.to_str()
        .expect("ScheduleSpec validates every rendered path as UTF-8")
}

fn absolute_normalized(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| format!("cannot resolve {}: {error}", path.display()))?
    };
    let absolute = lexical_normalize(&absolute);

    // Canonicalize the deepest existing ancestor, then append the not-yet-created
    // suffix.  This keeps `/var` and `/private/var` (or a symlinked HOME) in the
    // same namespace without requiring state/config files to exist already.
    let mut cursor = absolute.clone();
    let mut suffix = Vec::<OsString>::new();
    loop {
        if let Ok(mut canonical) = fs::canonicalize(&cursor) {
            for component in suffix.iter().rev() {
                canonical.push(component);
            }
            return Ok(canonical);
        }
        let Some(name) = cursor.file_name() else {
            break;
        };
        suffix.push(name.to_os_string());
        if !cursor.pop() {
            break;
        }
    }
    Ok(absolute)
}

fn resolve_primary_worktree(repo: &Path, require_repo: bool) -> Result<PathBuf, String> {
    let repo = absolute_normalized(repo)?;
    if !repo.exists() {
        return if require_repo {
            Err(format!(
                "repository path does not exist: {}. No scheduler was installed.",
                repo.display()
            ))
        } else {
            // Allows cleanup/status computation for a repository removed after a
            // scheduler was installed with this already-primary path.
            Ok(repo)
        };
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["worktree", "list", "--porcelain"])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| format!("cannot resolve primary worktree with git: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "cannot resolve the primary worktree for {}: {}. No scheduler file was changed.",
            repo.display(),
            if detail.is_empty() {
                "`git worktree list --porcelain` failed"
            } else {
                &detail
            }
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|_| {
        "git returned a non-UTF-8 primary worktree path; refusing scheduler configuration"
            .to_string()
    })?;
    let primary = stdout
        .lines()
        .find_map(|line| line.strip_prefix("worktree "))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "git returned no primary worktree for {}; no scheduler file was changed",
                repo.display()
            )
        })?;
    absolute_normalized(Path::new(primary))
}

fn git_identity(repo: &Path) -> Result<Option<GitIdentity>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| format!("cannot resolve Git common directory identity: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "cannot resolve Git common directory identity for {}: {}",
            repo.display(),
            if detail.is_empty() {
                "git rev-parse failed"
            } else {
                &detail
            }
        ));
    }
    let raw = String::from_utf8(output.stdout)
        .map_err(|_| "Git common directory path is not UTF-8".to_string())?;
    let common = absolute_normalized(Path::new(raw.trim()))?;
    let metadata = fs::metadata(&common).map_err(|error| {
        format!(
            "cannot stat Git common directory {}: {error}",
            common.display()
        )
    })?;
    Ok(platform_file_identity(&metadata))
}

#[cfg(unix)]
fn platform_file_identity(metadata: &fs::Metadata) -> Option<GitIdentity> {
    use std::os::unix::fs::MetadataExt;
    Some(GitIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(not(unix))]
fn platform_file_identity(_metadata: &fs::Metadata) -> Option<GitIdentity> {
    None
}

fn lexical_normalize(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn require_safe_utf8_path(label: &str, path: &Path) -> Result<(), String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("{label} is not valid UTF-8: {}", path.display()))?;
    if text.chars().any(char::is_control) {
        return Err(format!(
            "{label} contains a control character; refusing scheduler configuration"
        ));
    }
    Ok(())
}

fn repo_key(repo: &Path) -> String {
    let basename = repo.file_name().and_then(OsStr::to_str).unwrap_or("repo");
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in basename.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            previous_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if !previous_dash {
            previous_dash = true;
            Some('-')
        } else {
            None
        };
        if let Some(ch) = mapped {
            slug.push(ch);
        }
        if slug.len() >= 32 {
            break;
        }
    }
    let slug = slug.trim_matches('-');
    let slug = if slug.is_empty() { "repo" } else { slug };
    format!("{slug}-{:016x}", fnv1a64(path_text(repo).as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with_permissions(path, bytes, false)
}

fn atomic_write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_write_with_permissions(path, bytes, true)
}

fn atomic_write_with_permissions(path: &Path, bytes: &[u8], private: bool) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} has no parent directory", path.display()),
        )
    })?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".agent-on-{}.{}.tmp",
        path.file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("schedule"),
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp)?;
    if private {
        set_private_file_permissions(&temp)?;
    } else {
        set_public_config_permissions(&temp)?;
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temp, path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_public_config_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644))
}

#[cfg(not(unix))]
fn set_public_config_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn git(cwd: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn fixture(kind: SchedulerKind) -> (TempDir, ScheduleSpec) {
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home & person");
        let repo = temp.path().join("Repos").join("Demo Repo");
        let executable = temp.path().join("bin").join("agent-on");
        fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-b", "main"]);
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let spec = ScheduleSpec::new(
            kind,
            &repo,
            true,
            home.clone(),
            home.join("state"),
            home.join("config"),
            executable,
            "/opt/homebrew/bin:/usr/bin".to_string(),
        )
        .unwrap();
        (temp, spec)
    }

    fn persist_without_activation(spec: &ScheduleSpec) -> (PathBuf, InstallState) {
        let state = InstallState::from_spec(spec).unwrap();
        let state_root = state.state_dir().parent().unwrap().to_path_buf();
        fs::create_dir_all(state.state_dir()).unwrap();
        set_private_dir_permissions(&state.state_dir()).unwrap();
        write_install_state(&state).unwrap();
        for (path, contents) in state.rendered_files() {
            atomic_write(&path, contents.as_bytes()).unwrap();
        }
        (state_root, state)
    }

    #[test]
    fn launchd_render_is_report_only_and_escapes_paths() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let plist = render_launchd(&spec);
        assert!(plist.contains("<string>worktree</string>"));
        assert!(plist.contains("<string>gc</string>"));
        assert!(plist.contains("<string>--dry-run</string>"));
        assert!(plist.contains("<string>--json</string>"));
        assert!(plist.contains("<string>--repo</string>"));
        assert!(!plist.contains("--force"));
        assert!(!plist.contains(" remove"));
        assert!(plist.contains("home &amp; person"));
        assert!(plist.contains("<integer>3</integer>"));
        assert!(plist.contains("<integer>30</integer>"));
        assert!(plist.contains("GIT_OPTIONAL_LOCKS"));
        assert!(plist.contains("reports.log"));
    }

    #[test]
    fn scheduled_command_is_exactly_the_report_only_gc_surface() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        assert_eq!(
            gc_args(&spec),
            vec![
                path_text(&spec.executable),
                "worktree",
                "gc",
                "--dry-run",
                "--json",
                "--repo",
                path_text(&spec.repo),
            ]
        );
    }

    #[test]
    fn systemd_render_is_oneshot_persistent_and_report_only() {
        let (_temp, spec) = fixture(SchedulerKind::SystemdUser);
        let service = render_systemd_service(&spec);
        let timer = render_systemd_timer(&spec);
        assert!(service.contains("Type=oneshot"));
        assert!(service.contains("\"worktree\" \"gc\" \"--dry-run\" \"--json\" \"--repo\""));
        assert!(service.contains("StandardOutput=\"append:"));
        assert!(!service.contains("--force"));
        assert!(timer.contains("OnCalendar=*-*-* 03:30:00"));
        assert!(timer.contains("Persistent=true"));
        assert!(timer.contains(&format!("Unit={}", spec.systemd_service_name)));
    }

    #[test]
    fn repository_keys_are_stable_distinct_and_scheduler_safe() {
        let one = Path::new("/tmp/Team Repo");
        let two = Path::new("/other/Team Repo");
        let key_one = repo_key(one);
        assert_eq!(key_one, repo_key(one));
        assert_ne!(key_one, repo_key(two));
        assert!(key_one.starts_with("team-repo-"));
        assert!(key_one
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-'));
    }

    #[test]
    fn linked_and_primary_worktrees_share_one_stable_schedule_identity() {
        let temp = TempDir::new().unwrap();
        let primary = temp.path().join("primary");
        let linked = temp.path().join("linked");
        fs::create_dir_all(&primary).unwrap();
        git(&primary, &["init", "-b", "main"]);
        git(&primary, &["config", "user.email", "test@example.com"]);
        git(&primary, &["config", "user.name", "Test"]);
        fs::write(primary.join("README.md"), "root\n").unwrap();
        git(&primary, &["add", "README.md"]);
        git(&primary, &["commit", "-m", "init"]);
        git(
            &primary,
            &[
                "worktree",
                "add",
                "-b",
                "feat/lane",
                linked.to_str().unwrap(),
                "main",
            ],
        );

        let from_linked = resolve_primary_worktree(&linked, true).unwrap();
        let from_primary = resolve_primary_worktree(&primary, true).unwrap();
        assert_eq!(from_linked, from_primary);
        assert_eq!(from_primary, fs::canonicalize(&primary).unwrap());

        let home = temp.path().join("home");
        let executable = home.join("bin").join("agent-on");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        let linked_install = ScheduleSpec::new(
            SchedulerKind::Launchd,
            &from_linked,
            true,
            home.clone(),
            home.join("state"),
            home.join("config"),
            executable.clone(),
            "/usr/bin".to_string(),
        )
        .unwrap();
        let main_status_or_uninstall = ScheduleSpec::new(
            SchedulerKind::Launchd,
            &from_primary,
            true,
            home.clone(),
            home.join("state"),
            home.join("config"),
            executable,
            "/usr/bin".to_string(),
        )
        .unwrap();
        assert_eq!(linked_install.key, main_status_or_uninstall.key);
        assert_eq!(
            linked_install.launchd_plist,
            main_status_or_uninstall.launchd_plist
        );
        assert_eq!(linked_install.repo, from_primary);
        assert_eq!(
            gc_args(&linked_install).last(),
            Some(&path_text(&from_primary).to_string())
        );
    }

    #[test]
    fn persisted_state_survives_path_and_executable_environment_changes() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let (state_root, installed) = persist_without_activation(&spec);

        let replacement_executable = spec.home.join("new bin").join("agent-on-v2");
        fs::create_dir_all(replacement_executable.parent().unwrap()).unwrap();
        fs::write(&replacement_executable, b"new binary").unwrap();
        let current_runtime = ScheduleSpec::new(
            SchedulerKind::Launchd,
            &spec.repo,
            true,
            spec.home.clone(),
            spec.home.join("state"),
            spec.home.join("config"),
            replacement_executable,
            "/different/path:/usr/bin".to_string(),
        )
        .unwrap();
        assert_ne!(installed.command.first(), gc_args(&current_runtime).first());
        assert_ne!(
            installed.files[0].contents,
            current_runtime.rendered_files().files[0].1
        );

        let found = find_install_state_in_root(&spec.repo, &state_root)
            .unwrap()
            .unwrap();
        assert_eq!(found, installed);
        assert_eq!(found.command[0], path_text(&spec.executable));
        assert!(found.files.iter().all(|file| {
            fs::read_to_string(&file.path).ok().as_deref() == Some(&file.contents)
        }));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(found.state_file())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn persisted_state_finds_moved_repo_by_identity_and_deleted_repo_by_old_path() {
        let (temp, spec) = fixture(SchedulerKind::Launchd);
        let old_repo = spec.repo.clone();
        let (state_root, installed) = persist_without_activation(&spec);
        let moved_repo = temp.path().join("Moved Demo Repo");
        fs::rename(&old_repo, &moved_repo).unwrap();

        let found_after_move = find_install_state_in_root(&moved_repo, &state_root)
            .unwrap()
            .unwrap();
        assert_eq!(found_after_move, installed);
        assert_eq!(found_after_move.repo_at_install, path_text(&old_repo));

        fs::remove_dir_all(&moved_repo).unwrap();
        let found_after_delete = find_install_state_in_root(&old_repo, &state_root)
            .unwrap()
            .unwrap();
        assert_eq!(found_after_delete, installed);
        let report = inspect_installed(&found_after_delete);
        assert_eq!(report.state, ScheduleState::Drifted);
        assert!(report.detail.contains("moved or was deleted"));
    }

    #[test]
    fn persisted_state_drift_fails_closed_before_scheduler_access() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let (state_root, installed) = persist_without_activation(&spec);
        let mut raw = fs::read_to_string(installed.state_file()).unwrap();
        raw.push(' ');
        fs::write(installed.state_file(), raw).unwrap();
        let error = find_install_state_in_root(&spec.repo, &state_root).unwrap_err();
        assert!(error.contains("formatting/content drifted"));
        assert!(installed
            .files
            .iter()
            .all(|file| Path::new(&file.path).is_file()));
    }

    #[test]
    fn missing_persisted_executable_is_reported_but_state_stays_uninstallable() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let (_state_root, installed) = persist_without_activation(&spec);
        fs::remove_file(&spec.executable).unwrap();
        let report = inspect_installed(&installed);
        assert_eq!(report.state, ScheduleState::Drifted);
        assert!(report.detail.contains("missing or not executable"));
        assert!(installed.state_file().is_file());
        assert!(installed
            .files
            .iter()
            .all(|file| Path::new(&file.path).is_file()));
    }

    #[test]
    fn rendered_files_write_only_to_temp_home_and_state() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let files = spec.rendered_files();
        fs::create_dir_all(&spec.state_dir).unwrap();
        for (path, contents) in &files.files {
            atomic_write(path, contents.as_bytes()).unwrap();
        }
        assert!(files.files.iter().all(|(path, expected)| {
            fs::read_to_string(path).ok().as_deref() == Some(expected)
        }));
        assert!(spec.launchd_plist.starts_with(&spec.home));
        assert!(spec.state_dir.starts_with(&spec.home));
        assert!(!spec.launchd_plist.starts_with(&spec.repo));
        assert!(!spec.state_dir.starts_with(&spec.repo));
        assert!(!spec.repo.join(".agent-on").exists());
    }

    #[test]
    fn optional_absence_is_structured_not_an_error() {
        let report = ScheduleStatus {
            state: ScheduleState::Absent,
            installed: false,
            active: false,
            detail: "enable with `agent-on worktree hooks install --daily-gc`".to_string(),
            reports_path: None,
            errors_path: None,
        };
        assert_eq!(report.state, ScheduleState::Absent);
        assert!(!report.installed);
        assert!(!report.active);
        assert!(render_status(&report).contains("not installed (optional)"));
        assert!(report.detail.contains("hooks install --daily-gc"));
    }

    #[test]
    fn partial_systemd_install_is_reported_as_drift() {
        let (_temp, spec) = fixture(SchedulerKind::SystemdUser);
        let state = InstallState::from_spec(&spec).unwrap();
        let rendered = state.rendered_files();
        let (service_path, service) = &rendered[0];
        atomic_write(service_path, service.as_bytes()).unwrap();
        let report = inspect_installed(&state);
        assert_eq!(report.state, ScheduleState::Drifted);
        assert!(report.installed);
        assert!(!report.active);
        assert!(report.detail.contains(".timer"));
    }

    #[test]
    fn uninstall_refuses_changed_scheduler_file_before_deactivation() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let state = InstallState::from_spec(&spec).unwrap();
        atomic_write(&spec.launchd_plist, b"user-owned change\n").unwrap();
        let error = uninstall_installed(&state).unwrap_err();
        assert!(error.contains("refusing to disable or remove changed scheduler assets"));
        assert!(error.contains(&spec.launchd_plist.display().to_string()));
        assert_eq!(
            fs::read_to_string(&spec.launchd_plist).unwrap(),
            "user-owned change\n"
        );
    }

    #[test]
    fn install_refuses_to_overwrite_foreign_scheduler_file() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        atomic_write(&spec.launchd_plist, b"user-owned plist\n").unwrap();
        let error = install_new(&spec).unwrap_err();
        assert!(error.contains("refusing to overwrite existing scheduler assets"));
        assert_eq!(
            fs::read_to_string(&spec.launchd_plist).unwrap(),
            "user-owned plist\n"
        );
        assert!(!spec.state_dir.exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rendered_launchd_plist_passes_native_plutil_validation() {
        let (_temp, spec) = fixture(SchedulerKind::Launchd);
        let plist = render_launchd(&spec);
        atomic_write(&spec.launchd_plist, plist.as_bytes()).unwrap();
        let output = Command::new("plutil")
            .args(["-lint", "--"])
            .arg(&spec.launchd_plist)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "plutil rejected {}: {}{}",
            spec.launchd_plist.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn uninstall_spec_survives_a_missing_repo() {
        let (temp, installed) = fixture(SchedulerKind::SystemdUser);
        fs::remove_dir_all(&installed.repo).unwrap();
        let rebuilt = ScheduleSpec::new(
            SchedulerKind::SystemdUser,
            &installed.repo,
            false,
            installed.home.clone(),
            temp.path().join("home & person").join("state"),
            temp.path().join("home & person").join("config"),
            installed.executable.clone(),
            installed.path_env.clone(),
        )
        .unwrap();
        assert_eq!(rebuilt.key, installed.key);
        assert_eq!(rebuilt.systemd_timer, installed.systemd_timer);
    }

    #[test]
    fn unsafe_control_characters_fail_closed() {
        let temp = TempDir::new().unwrap();
        let error = ScheduleSpec::new(
            SchedulerKind::Launchd,
            Path::new("/tmp/repo\nattack"),
            false,
            temp.path().join("home"),
            temp.path().join("state"),
            temp.path().join("config"),
            temp.path().join("agent-on"),
            "/usr/bin".to_string(),
        )
        .unwrap_err();
        assert!(error.contains("control character"));
    }

    #[test]
    fn state_or_scheduler_files_inside_repo_fail_closed() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path().join("repo");
        let home = temp.path().join("home");
        let executable = home.join("bin").join("agent-on");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        let error = ScheduleSpec::new(
            SchedulerKind::Launchd,
            &repo,
            true,
            home.clone(),
            repo.join("local-state"),
            home.join("config"),
            executable,
            "/usr/bin".to_string(),
        )
        .unwrap_err();
        assert!(error.contains("inside repository"));
        assert!(!repo.join("local-state").exists());
    }

    #[test]
    fn shell_and_systemd_escaping_are_explicit() {
        assert_eq!(shell_quote("plain/path"), "plain/path");
        assert_eq!(shell_quote("has space"), "'has space'");
        assert_eq!(shell_quote("has'quote"), "'has'\\''quote'");
        assert_eq!(systemd_quote("a%b$c\\d\"e"), "\"a%%b$$c\\\\d\\\"e\"");
    }
}
