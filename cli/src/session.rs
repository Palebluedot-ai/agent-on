//! Host session identity and a small, read-only startup/compact projection.
//! No automatic role claim, recorder enablement, new window, or remote ownership.
use crate::coordination::Store;
use crate::oncall::{self, Role};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Identity {
    pub(crate) host: String,
    pub(crate) session_id: String,
}
impl Identity {
    fn new(host: &str, id: &str) -> Option<Self> {
        let host = match host {
            "codex" | "codex-app" => "codex",
            "claude" | "grok" | "hermes" => host,
            _ => return None,
        };
        if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
            return None;
        }
        Some(Self {
            host: host.into(),
            session_id: id.into(),
        })
    }
    pub(crate) fn current() -> Option<Self> {
        // Claude launched from Codex can inherit its parent's thread id. Its
        // SessionStart adapter marks that value; a newly launched Codex child
        // gets a different native thread id and must supersede the parent.
        if let Ok(id) = std::env::var("CODEX_THREAD_ID") {
            if !id.is_empty()
                && std::env::var("AGENT_ON_PARENT_CODEX_THREAD_ID")
                    .ok()
                    .as_deref()
                    != Some(&id)
            {
                return Self::new("codex", &id);
            }
        }
        Self::new(
            &std::env::var("AGENT_ON_HOST").ok()?,
            &std::env::var("AGENT_ON_SESSION_ID").ok()?,
        )
    }
    pub(crate) fn from_hook(data: &Value) -> Option<Self> {
        let Some(id) = data.get("session_id") else {
            // Older hook wiring may omit the field; only a native environment
            // identity can fill it, never the workdir or a routing address.
            return Self::current();
        };
        let id = id.as_str()?;
        let host = if std::env::var_os("CLAUDE_ENV_FILE").is_some() {
            "claude".into()
        } else if std::env::var_os("PLUGIN_ROOT").is_some() {
            "codex".into()
        } else if std::env::var_os("CLAUDE_PLUGIN_ROOT").is_some() {
            "claude".into()
        } else if data.get("turn_id").is_some() {
            "codex".into()
        } else {
            Self::current()?.host
        };
        Self::new(&host, id)
    }
}

fn managed(store: &Store) -> bool {
    store.repo.join("agent-on.lock.md").is_file()
        || (store.repo.join("CHARTER.md").is_file() && store.repo.join("BOOTSTRAP.md").is_file())
        || store.dir.is_dir()
}

pub(crate) fn startup_context(data: &Value) -> Result<Option<String>, String> {
    if data.get("hook_event_name").and_then(Value::as_str) != Some("SessionStart") {
        return Ok(None);
    }
    let cwd = Path::new(
        data.get("cwd")
            .and_then(Value::as_str)
            .ok_or("missing hook cwd")?,
    );
    let store = Store::open(cwd)?;
    if !managed(&store) {
        return Ok(None);
    }
    let identity = Identity::from_hook(data);
    if let Err(error) = persist_claude_identity(identity.as_ref()) {
        return Ok(Some(unavailable(&error)));
    }
    let config = match store.config() {
        Ok(value) => value,
        Err(error) => return Ok(Some(unavailable(&error))),
    };
    let tasks = match store.tasks() {
        Ok(value) => value,
        Err(error) => return Ok(Some(unavailable(&error))),
    };
    // A damaged registry is reported as unavailable, never as nobody on call.
    let record = match oncall::load(cwd) {
        Ok(value) => value,
        Err(error) => return Ok(Some(unavailable(&error))),
    };
    let oncall = match oncall::role_for(cwd, identity.as_ref()) {
        Role::Oncall(r) => format!("值守：{}（本窗口）", r.session),
        Role::Feature(r) => format!(
            "值守：{}（另一个窗口；合入和跨窗口派工经它）{}",
            r.session,
            if r.identity.is_none() {
                "；旧登记缺实际会话身份，需原值守明确重新接班"
            } else {
                ""
            }
        ),
        Role::Nobody if record.is_some() => "值守：登记已失效，当前无人在班".into(),
        Role::Nobody => "值守：无人在班".into(),
    };
    let active: Vec<_> = tasks.iter().filter(|t| t.released_at.is_none()).collect();
    let brief = active
        .iter()
        .take(5)
        .map(|t| format!("{} ({})", t.id, t.state))
        .collect::<Vec<_>>()
        .join("、");
    Ok(Some(format!("Agent-On 项目状态已接续。{}。巡逻/任务捕获：{}。未释放任务 {} 项{}。共读台账：{}；详细状态用 agent-on dashboard --json。按项目规则和真实文件继续；结果回传不等于验证完成。原窗口继续，不重复初始化或自动新开窗口。", oncall, if config.enabled { "已启用" } else { "未启用" }, active.len(), if brief.is_empty() { String::new() } else { format!("：{brief}") }, store.dir.display())))
}

fn unavailable(error: &str) -> String {
    format!("Agent-On 项目状态不可用，不能确认负责人或任务。请修复台账/会话接线并运行 agent-on dashboard --json 核对。取证错误：{error}")
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn persist_claude_identity(identity: Option<&Identity>) -> Result<(), String> {
    let Some(identity) = identity.filter(|i| i.host == "claude") else {
        return Ok(());
    };
    let Some(path) = std::env::var_os("CLAUDE_ENV_FILE") else {
        return Ok(());
    };
    // Only the host-provided per-session environment file; append and preserve
    // other hooks. Never edit a shell profile or a user-level configuration.
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|e| format!("session environment unavailable: {e}"))?;
    writeln!(file, "\nexport AGENT_ON_HOST='claude'\nexport AGENT_ON_SESSION_ID={}\nexport AGENT_ON_PARENT_CODEX_THREAD_ID={}", quote(&identity.session_id), quote(&std::env::var("CODEX_THREAD_ID").unwrap_or_default())).map_err(|e| e.to_string())
}
