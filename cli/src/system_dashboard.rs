//! Bounded local inventory and observational receipts, not a cross-project executor.
//! Existing task ledgers remain authoritative; process presence never proves task progress.

use crate::{coordination as c, worktree};
use clap::{Args, Subcommand, ValueEnum};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MAX_DIRS: usize = 512;
const MAX_PROJECTS: usize = 128;
const MAX_RECORDS: usize = 256;
const MAX_BYTES: u64 = 262_144;
const MAX_PROCESS_BYTES: u64 = 1_048_576;
const HOSTS: [&str; 5] = ["codex", "claude", "hermes", "grok", "antigravity"];

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum AgentHost {
    Codex,
    Claude,
    Hermes,
    Grok,
    Antigravity,
}
impl AgentHost {
    fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Hermes => "hermes",
            Self::Grok => "grok",
            Self::Antigravity => "antigravity",
        }
    }
}
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum AgentState {
    Working,
    Waiting,
    Blocked,
    Done,
}
impl AgentState {
    fn name(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Blocked => "blocked",
            Self::Done => "done",
        }
    }
}
#[derive(Args)]
pub(crate) struct AgentArgs {
    #[command(subcommand)]
    action: AgentCommand,
}
#[derive(Subcommand)]
enum AgentCommand {
    /// Self-reported observation only; does not dispatch, update a task, or register on-call
    Report {
        #[arg(long)]
        host: AgentHost,
        /// Stable id within this project and host; reuse it for heartbeat/status updates
        #[arg(long)]
        id: String,
        #[arg(long)]
        state: AgentState,
        #[arg(long, default_value = "")]
        note: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        repo: Option<PathBuf>,
    },
}

pub(crate) fn report(args: AgentArgs) -> Result<Value, String> {
    let AgentCommand::Report {
        host,
        id,
        state,
        note,
        task,
        repo,
    } = args.action;
    c::validate_id(&id)?;
    if let Some(task) = &task {
        c::validate_id(task)?;
    }
    let repo = repo.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let store = c::Store::open(&repo)?;
    if let Some(task) = &task {
        // Link only to an existing task, without modifying it or impersonating its owner.
        store.task(task)?;
    }
    let receipt = json!({"schema_version":1,"id":id,"host":host.name(),
        "state":state.name(),"note":c::prompt_summary(&note),"task":task,
        "at":c::now(),"source":"self-report","verified":false});
    let dir = store.dir.join("agents");
    if fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err("通用回执目录是软链，拒绝写入；请修复项目接线".into());
    }
    c::write_json(&dir.join(format!("{}-{id}.json", host.name())), &receipt)?;
    Ok(receipt)
}

#[derive(Clone)]
pub(crate) struct Project {
    pub(crate) id: String,
    pub(crate) repo: PathBuf,
    control: Option<PathBuf>,
    checkouts: Vec<PathBuf>,
    aliases: Vec<String>,
    discovery_error: Option<String>,
}
pub(crate) struct Collector {
    roots: Vec<PathBuf>,
    explicit: Vec<PathBuf>,
    pub(crate) projects: Vec<Project>,
    warnings: Vec<String>,
    discovered: Option<Instant>,
}
impl Collector {
    pub(crate) fn new(mut roots: Vec<PathBuf>, explicit: Vec<PathBuf>) -> Self {
        if roots.is_empty() && explicit.is_empty() {
            let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
            roots.push(home.map(PathBuf::from).unwrap_or_default().join("Projects"));
        }
        Self {
            roots,
            explicit,
            projects: Vec::new(),
            warnings: Vec::new(),
            discovered: None,
        }
    }
    fn discover(&mut self) {
        if self
            .discovered
            .is_some_and(|at| at.elapsed() < Duration::from_secs(60))
        {
            return;
        }
        self.projects.clear();
        self.warnings.clear();
        let mut seen = BTreeSet::new();
        let mut visited = 0;
        for path in self.explicit.clone() {
            self.add_project(&path, &mut seen);
        }
        for root in self.roots.clone() {
            if !root.is_dir() {
                self.warnings
                    .push(format!("扫描目录不可读：{}", root.display()));
                continue;
            }
            let resolved = fs::canonicalize(&root).unwrap_or(root);
            self.walk(&resolved, 0, &mut visited, &mut seen);
        }
        self.projects.sort_by(|a, b| a.repo.cmp(&b.repo));
        self.discovered = Some(Instant::now());
    }
    fn walk(
        &mut self,
        path: &Path,
        depth: usize,
        visited: &mut usize,
        seen: &mut BTreeSet<PathBuf>,
    ) {
        if *visited >= MAX_DIRS || self.projects.len() >= MAX_PROJECTS {
            if !self.warnings.iter().any(|w| w.contains("扫描上限")) {
                self.warnings
                    .push("达到扫描上限；可用 --project 显式补充，未扫描部分状态未知。".into());
            }
            return;
        }
        *visited += 1;
        if path.join(".git").exists() {
            self.add_project(path, seen);
            return;
        }
        let marker = [
            "AGENTS.md",
            "CLAUDE.md",
            "agent-on.lock.md",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "pubspec.yaml",
        ]
        .iter()
        .any(|name| path.join(name).is_file());
        if marker {
            self.add_project(path, seen);
            return;
        }
        if depth >= 3 {
            return;
        }
        match fs::read_dir(path) {
            Ok(entries) => {
                // Stable bounded traversal, ignoring generated/archive directories and symlinks.
                let mut paths = Vec::new();
                let mut total = 0;
                let before = self.projects.len();
                for entry in entries.take(MAX_DIRS + 1) {
                    total += 1;
                    match entry {
                        Ok(entry) if entry.file_type().is_ok_and(|t| t.is_dir()) => {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if !excluded(&name) {
                                paths.push(entry.path());
                            }
                        }
                        Ok(entry) if entry.file_type().is_ok_and(|t| t.is_symlink()) => {
                            // Only direct project aliases inside the chosen root; never recurse a symlink.
                            if depth == 0 {
                                if let Ok(target) = fs::canonicalize(entry.path()) {
                                    if target.strip_prefix(path).is_ok_and(|p| {
                                        !p.components().any(|part| {
                                            excluded(&part.as_os_str().to_string_lossy())
                                        })
                                    }) && target.join(".git").exists()
                                    {
                                        self.add_project(&entry.path(), seen);
                                    }
                                }
                            }
                        }
                        Ok(_) => (),
                        Err(error) => self.warnings.push(format!("{}：{error}", path.display())),
                    }
                }
                if total > MAX_DIRS {
                    self.warnings
                        .push(format!("目录条目超过扫描上限：{}", path.display()));
                }
                paths.sort();
                for child in paths {
                    self.walk(&child, depth + 1, visited, seen);
                }
                // A direct project directory can have no recognized manifest or Git yet.
                // Keep it visible as unconnected, rather than claiming its tasks are absent.
                if depth == 1 && self.projects.len() == before {
                    self.add_project(path, seen);
                }
            }
            Err(error) => self.warnings.push(format!("{}：{error}", path.display())),
        }
    }
    fn add_project(&mut self, path: &Path, seen: &mut BTreeSet<PathBuf>) {
        if self.projects.len() >= MAX_PROJECTS {
            self.warnings
                .push("项目数达到扫描上限，剩余项目状态未知。".into());
            return;
        }
        let canonical = match fs::canonicalize(path) {
            Ok(p) if p.is_dir() => p,
            _ => {
                self.warnings
                    .push(format!("项目不可读：{}", path.display()));
                return;
            }
        };
        let (repo, control, checkouts, error, key) = if canonical.join(".git").exists() {
            match c::Store::open(&canonical) {
                Ok(store) => {
                    let checkouts = c::git(&store.repo, &["worktree", "list", "--porcelain"])
                        .map(|raw| {
                            worktree::parse_worktrees(&raw)
                                .into_iter()
                                .map(|t| t.path)
                                .collect()
                        })
                        .unwrap_or_else(|_| vec![store.repo.clone()]);
                    let key = store.dir.clone();
                    (store.repo, Some(store.dir), checkouts, None, key)
                }
                Err(error) => (
                    canonical.clone(),
                    None,
                    vec![canonical.clone()],
                    Some(error),
                    canonical,
                ),
            }
        } else {
            (
                canonical.clone(),
                None,
                vec![canonical.clone()],
                None,
                canonical,
            )
        };
        let alias = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let aliases = if path.file_name() != repo.file_name() {
            vec![alias.clone()]
        } else {
            Vec::new()
        };
        if seen.insert(key.clone()) {
            self.projects.push(Project {
                id: project_id(&key),
                repo,
                control,
                checkouts,
                aliases,
                discovery_error: error,
            });
        } else if let Some(project) = self.projects.iter_mut().find(|p| p.id == project_id(&key)) {
            if path.file_name() != project.repo.file_name() && !project.aliases.contains(&alias) {
                project.aliases.push(alias);
            }
        }
    }
    pub(crate) fn collect(&mut self) -> Value {
        self.discover();
        let (mut processes, process_error) = process_inventory();
        self.project_value(&mut processes, process_error)
    }
    fn project_value(&self, processes: &mut [Value], process_error: Option<String>) -> Value {
        for process in processes.iter_mut() {
            if let Some(cwd) = process["cwd"].as_str() {
                let cwd = Path::new(cwd);
                let project = self
                    .projects
                    .iter()
                    .filter(|p| p.checkouts.iter().any(|t| cwd.starts_with(t)))
                    .max_by_key(|p| {
                        p.checkouts
                            .iter()
                            .filter(|t| cwd.starts_with(t))
                            .map(|t| t.components().count())
                            .max()
                    });
                if let Some(project) = project {
                    process["project_id"] = json!(project.id);
                }
            }
        }
        let mut projects = Vec::new();
        let mut tasks = Vec::new();
        let mut agents = processes.to_vec();
        let mut errors = Vec::new();
        for project in &self.projects {
            let mut problems = project.discovery_error.iter().cloned().collect::<Vec<_>>();
            let mut ptasks = Vec::new();
            let mut pagents = Vec::new();
            let mut enabled = false;
            let mut activity = crate::project_status::activity_coverage();
            let connected = project.control.as_ref().is_some_and(|d| d.exists());
            if let Some(control) = &project.control {
                activity = crate::activity::collect_dir(control);
                for error in activity["errors"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    problems.push(format!("活动回执：{error}"));
                }
                for window in activity["windows"].as_array().into_iter().flatten() {
                    if window["state"] == "blocked" {
                        errors.push(error_row(
                            project,
                            "activity",
                            window["id"].as_str().unwrap_or("?"),
                            &window["note"],
                        ));
                    }
                    let mut row = window.clone();
                    row["project_id"] = json!(project.id);
                    pagents.push(row);
                }
                let config = control.join("config.json");
                if config.exists() {
                    match bounded_json(&config) {
                        Ok(v) => enabled = v["enabled"] == true,
                        Err(error) => problems.push(error),
                    }
                }
                let (raw_tasks, issues) = records(&control.join("tasks"));
                problems.extend(issues);
                let (sessions, issues) = records(&control.join("sessions"));
                problems.extend(issues);
                let (reports, issues) = records(&control.join("agents"));
                problems.extend(issues);
                for raw in raw_tasks {
                    let Some(id) = raw["id"].as_str().filter(|id| c::validate_id(id).is_ok())
                    else {
                        problems.push("任务记录缺少有效编号".into());
                        continue;
                    };
                    let mut hosts = sessions
                        .iter()
                        .filter(|s| s["task"] == id)
                        .filter_map(|s| host(&s["host"]).map(str::to_string))
                        .collect::<BTreeSet<_>>();
                    for receipt in &reports {
                        if receipt["task"] == id {
                            if let Some(host) = host(&receipt["host"]) {
                                hosts.insert(host.into());
                            }
                        }
                    }
                    if hosts.is_empty() {
                        if let Some(host) = host(&raw["last_reporter"]["host"]) {
                            hosts.insert(host.into());
                        }
                    }
                    let task = json!({"id":id,"key":format!("{}:{id}",project.id),"project_id":project.id,
                        "title":safe(&raw["title"]),"goal":safe(&raw["goal"]),"state":safe(&raw["state"]),
                        "note":safe(&raw["progress"]["note"]),"at":timestamp(&raw["progress"]["at"]).or_else(||timestamp(&raw["created_at"])),
                        "hosts":hosts,"source":"task-ledger"});
                    if task["state"] == "blocked" {
                        errors.push(error_row(project, "task", id, &task["note"]));
                    }
                    ptasks.push(task);
                }
                for receipt in sessions {
                    let Some(host) = host(&receipt["host"]) else {
                        continue;
                    };
                    let age = age(&receipt["heartbeat_at"]);
                    let pid = receipt["child_pid"].as_u64().or(receipt["pid"].as_u64());
                    let alive = pid
                        .filter(|p| *p > 0 && *p <= u32::MAX as u64)
                        .map(|p| c::pid_alive(p as u32));
                    let failed = receipt["state"] == "failed"
                        || receipt["exit_code"].as_i64().is_some_and(|code| code != 0);
                    let row = json!({"id":safe(&receipt["id"]),"host":host,"project_id":project.id,
                        "state":safe(&receipt["state"]),"task":safe(&receipt["task"]),"source":"session-receipt",
                        "at":timestamp(&receipt["heartbeat_at"]),"age_seconds":age,"stale":age.is_none_or(|n|n>120),
                        "process_alive":alive,"pid":pid,"exit_code":receipt["exit_code"].as_i64()});
                    if failed {
                        errors.push(error_row(
                            project,
                            "session",
                            receipt["id"].as_str().unwrap_or("?"),
                            &json!(format!(
                                "{} · exit {}",
                                safe(&receipt["state"]),
                                receipt["exit_code"]
                                    .as_i64()
                                    .map(|code| code.to_string())
                                    .unwrap_or_else(|| "未知".into())
                            )),
                        ));
                    }
                    pagents.push(row);
                }
                for receipt in reports {
                    let Some(host) = host(&receipt["host"]) else {
                        continue;
                    };
                    let age = age(&receipt["at"]);
                    let row = json!({"id":safe(&receipt["id"]),"host":host,"project_id":project.id,
                        "state":safe(&receipt["state"]),"task":safe(&receipt["task"]),"note":safe(&receipt["note"]),
                        "source":"self-report","verified":false,"at":timestamp(&receipt["at"]),"age_seconds":age,
                        "stale":age.is_none_or(|n|n>120)});
                    if receipt["state"] == "blocked" {
                        errors.push(error_row(
                            project,
                            "agent",
                            receipt["id"].as_str().unwrap_or("?"),
                            &receipt["note"],
                        ));
                    }
                    pagents.push(row);
                }
            }
            for problem in &problems {
                errors.push(error_row(project, "source", "records", &json!(problem)));
            }
            let active = ptasks
                .iter()
                .filter(|t| !matches!(t["state"].as_str(), Some("released" | "reclaimed")))
                .count();
            let mut project_hosts = pagents
                .iter()
                .chain(processes.iter().filter(|a| a["project_id"] == project.id))
                .filter_map(|a| host(&a["host"]).map(str::to_string))
                .collect::<BTreeSet<_>>();
            for task in &ptasks {
                for h in task["hosts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    project_hosts.insert(h.into());
                }
            }
            projects.push(json!({"id":project.id,"name":project.repo.file_name().unwrap_or_default().to_string_lossy(),
                "repo":project.repo,"aliases":project.aliases,"connected":connected,
                "capture_enabled":enabled || activity["enabled"] == true,"patrol_capture_enabled":enabled,
                "activity_enabled":activity["enabled"],"available":problems.is_empty(),
                "activity":activity,
                "errors":problems,"tasks":active,"blocked":ptasks.iter().filter(|t|t["state"]=="blocked").count(),
                "agents":pagents.len(),"hosts":project_hosts,"worktrees":project.checkouts.len(),
                "detail_available":project.control.is_some()}));
            tasks.extend(ptasks);
            agents.extend(pagents);
        }
        let active = tasks
            .iter()
            .filter(|t| !matches!(t["state"].as_str(), Some("released" | "reclaimed")))
            .count();
        json!({"schema_version":2,"scope":"system","observed_at":c::now(),"roots":self.roots,
            "explicit_projects":self.explicit,"projects":projects,"tasks":tasks,"agents":agents,"errors":errors,
            "counts":{"projects":projects.len(),"active_tasks":active,"errors":errors.len(),"observed_agents":agents.len()},
            "discovery":{"interval_seconds":60,"max_depth":3,"max_directories":MAX_DIRS,"max_projects":MAX_PROJECTS,"warnings":self.warnings},
            "process_inventory":{"available":process_error.is_none(),"error":process_error},"hosts":HOSTS,
            "coverage":"仅本机选定目录；已有任务/回执 + 可识别进程。不读对话、参数或密钥，不连接云端；进程存在 ≠ 执行中；未登记的任务与错误未知。"})
    }
}

fn excluded(name: &str) -> bool {
    name.starts_with('.')
        || name.starts_with('_')
        || matches!(
            name,
            "node_modules" | "target" | "vendor" | "dist" | "build" | "venv" | "Pods"
        )
}
fn project_id(path: &Path) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path.as_os_str().to_string_lossy().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    format!("p{hash:016x}")
}
fn host(value: &Value) -> Option<&str> {
    let h = value.as_str()?;
    if h == "codex-app" {
        Some("codex")
    } else {
        HOSTS.contains(&h).then_some(h)
    }
}
fn safe(value: &Value) -> String {
    c::prompt_summary(value.as_str().unwrap_or_default())
}
fn age(value: &Value) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value.as_str()?)
        .ok()
        .map(|at| {
            (chrono::Utc::now() - at.with_timezone(&chrono::Utc))
                .num_seconds()
                .max(0)
        })
}
fn timestamp(value: &Value) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(value.as_str()?)
        .ok()
        .map(|at| at.to_rfc3339())
}
fn error_row(project: &Project, source: &str, id: &str, note: &Value) -> Value {
    json!({"project_id":project.id,"project":project.repo.file_name().unwrap_or_default().to_string_lossy(),"source":source,"id":c::prompt_summary(id),"note":safe(note)})
}
fn bounded_json(path: &Path) -> Result<Value, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}：{e}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err(format!(
            "记录不是普通文件或超过大小限制：{}",
            path.display()
        ));
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("记录超过大小限制".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| format!("记录损坏：{}", path.display()))
}
fn records(dir: &Path) -> (Vec<Value>, Vec<String>) {
    if !dir.exists() {
        return (Vec::new(), Vec::new());
    }
    if fs::symlink_metadata(dir).is_ok_and(|m| m.file_type().is_symlink()) {
        return (
            Vec::new(),
            vec![format!("不读取软链记录目录：{}", dir.display())],
        );
    }
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) => return (Vec::new(), vec![format!("{}：{e}", dir.display())]),
    };
    let mut paths = Vec::new();
    let mut errors = Vec::new();
    let mut total = 0;
    for entry in entries.take(MAX_RECORDS + 1) {
        total += 1;
        match entry {
            Ok(e) if e.path().extension().is_some_and(|x| x == "json") => paths.push(e.path()),
            Ok(_) => (),
            Err(e) => errors.push(e.to_string()),
        }
    }
    if total > MAX_RECORDS {
        errors.push(format!("记录达到读取上限：{}", dir.display()));
    }
    paths.sort();
    let mut values = Vec::new();
    for path in paths.into_iter().take(MAX_RECORDS) {
        match bounded_json(&path) {
            Ok(v) if v.is_object() => values.push(v),
            Ok(_) => errors.push(format!("记录不是对象：{}", path.display())),
            Err(e) => errors.push(e),
        }
    }
    (values, errors)
}

/// Executable names/known installation paths only. Never read command arguments or transcripts.
fn classify(executable: &str) -> Option<(&'static str, bool)> {
    let lower = executable.to_ascii_lowercase();
    let base = Path::new(&lower).file_name()?.to_str()?;
    if lower.contains(".hermes/hermes-agent/venv/bin/python")
        || matches!(base, "hermes" | "hermes-agent")
    {
        return Some(("hermes", false));
    }
    let family = if matches!(base, "codex" | "codex.exe")
        || lower.contains("/codex.app/")
        || lower.contains("/codex framework.framework/")
    {
        "codex"
    } else if matches!(base, "claude" | "claude.exe") || lower.contains("/claude.app/") {
        "claude"
    } else if matches!(base, "grok" | "grok.exe") || lower.contains("/grok bot.app/") {
        "grok"
    } else if matches!(base, "antigravity" | "antigravity.exe")
        || lower.contains("/antigravity.app/")
    {
        "antigravity"
    } else {
        return None;
    };
    // Application helpers are inventory only, not separate autonomous tasks.
    let helper = lower.contains(" helper")
        || lower.contains("(renderer)")
        || lower.contains("(gpu)")
        || lower.contains("(plugin)")
        || matches!(
            base,
            "shipit" | "language_server" | "chrome_crashpad_handler"
        );
    Some((family, helper))
}
fn command_output(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdout = child.stdout.take().ok_or("没有进程输出")?;
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(MAX_PROCESS_BYTES + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let started = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => {
                let bytes = receiver
                    .recv_timeout(Duration::from_secs(1))
                    .map_err(|_| "进程读取超时")?
                    .map_err(|e| e.to_string())?;
                if !status.success() {
                    return Err("系统拒绝读取进程元数据，相关状态未知".into());
                }
                if bytes.len() as u64 > MAX_PROCESS_BYTES {
                    return Err("进程元数据超过读取上限".into());
                }
                return Ok(String::from_utf8_lossy(&bytes).into());
            }
            None if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("进程元数据读取超时".into());
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}
fn process_inventory() -> (Vec<Value>, Option<String>) {
    if !cfg!(unix) {
        return (
            Vec::new(),
            Some("此平台暂未实现进程发现；已有任务与通用回执仍可读".into()),
        );
    }
    let raw = match command_output("ps", &["-axo", "pid=,comm="], Duration::from_secs(2)) {
        Ok(v) => v,
        Err(e) => return (Vec::new(), Some(e)),
    };
    let mut rows = Vec::new();
    let mut warning = None;
    for line in raw.lines() {
        let line = line.trim();
        let Some((pid, exe)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        let Some((host, helper)) = classify(exe.trim()) else {
            continue;
        };
        if rows.len() >= 128 {
            warning = Some("可识别 Agent 进程达到上限；其余状态未知".into());
            break;
        }
        rows.push(
            json!({"id":format!("pid-{pid}"),"pid":pid,"host":host,"source":"process",
            "state":"present","helper":helper,"project_id":null,"cwd":null,"task":null}),
        );
    }
    if cfg!(target_os = "macos") && !rows.is_empty() {
        let pids = rows
            .iter()
            .filter_map(|r| r["pid"].as_u64())
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        match command_output(
            "/usr/sbin/lsof",
            &["-a", "-p", &pids, "-d", "cwd", "-Fpn"],
            Duration::from_secs(2),
        ) {
            Ok(raw) => {
                let mut pid = None;
                let mut paths = BTreeMap::new();
                for line in raw.lines() {
                    if let Some(value) = line.strip_prefix('p') {
                        pid = value.parse::<u64>().ok();
                    }
                    if let Some(path) = line.strip_prefix('n') {
                        if let Some(pid) = pid {
                            paths.insert(pid, path.to_string());
                        }
                    }
                }
                for row in &mut rows {
                    if let Some(path) = row["pid"].as_u64().and_then(|p| paths.get(&p)) {
                        row["cwd"] = json!(path);
                    }
                }
            }
            Err(e) => warning = Some(format!("工作目录观察不可用：{e}；未关联进程保留未知")),
        }
    } else if cfg!(target_os = "linux") {
        for row in &mut rows {
            if let Some(pid) = row["pid"].as_u64() {
                if let Ok(path) = fs::read_link(format!("/proc/{pid}/cwd")) {
                    row["cwd"] = json!(path);
                }
            }
        }
    }
    (rows, warning)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_executables_not_prompt_arguments() {
        for (path,host,helper) in [
            ("/opt/bin/codex","codex",false),("/Applications/Claude.app/Contents/MacOS/Claude","claude",false),
            ("/Users/test/.hermes/hermes-agent/venv/bin/python","hermes",false),
            ("/Applications/Grok Bot.app/Contents/Frameworks/Grok Bot Helper","grok",true),
            ("/Applications/Antigravity.app/Contents/Resources/bin/language_server","antigravity",true),
            ("/Applications/ChatGPT.app/Contents/Frameworks/Codex Framework.framework/Helpers/Codex (Renderer).app/Contents/MacOS/Codex (Renderer)","codex",true),
        ] { assert_eq!(classify(path),Some((host,helper))); }
        for path in [
            "node",
            "python3",
            "/tmp/my-codex-wrapper",
            "/tmp/claude-plan.txt",
        ] {
            assert_eq!(classify(path), None);
        }
    }
    #[test]
    fn safe_record_size_and_json_damage_are_isolated() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("good.json"), r#"{"id":"ok"}"#).unwrap();
        fs::write(
            temp.path().join("broken.json"),
            "password=private-invalid-json",
        )
        .unwrap();
        fs::write(
            temp.path().join("big.json"),
            vec![b'a'; MAX_BYTES as usize + 1],
        )
        .unwrap();
        let (records, errors) = records(temp.path());
        assert_eq!(records.len(), 1);
        assert_eq!(errors.len(), 2);
        assert!(!errors.join("").contains("private-invalid-json"));
    }
    #[test]
    fn unknown_process_cannot_invent_a_task() {
        let collector = Collector::new(vec![PathBuf::from("/not-used")], Vec::new());
        let value = collector.project_value(
            &mut [json!({"host":"claude","source":"process","state":"present","cwd":"/unknown"})],
            Some("denied".into()),
        );
        assert_eq!(value["tasks"], json!([]));
        assert_eq!(value["agents"][0]["project_id"], Value::Null);
        assert_eq!(value["process_inventory"]["available"], false);
    }
}
