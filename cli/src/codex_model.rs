//! Record the user's launch choice; optionally diagnose native metadata separately.
//! No automatic model/effort selection, config/auth writes, provider switching, or replay.
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, String>;
const TIMEOUT: Duration = Duration::from_secs(12);
const MAX_LINE: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Resolution {
    requested_model: Option<String>,
    configured_model: Option<String>,
    pub(crate) selected_model: Option<String>,
    pub(crate) reasoning_effort_override: Option<String>,
    pub(crate) source: String,
    account_type: Option<String>,
    verification: String,
    checked_at: String,
}

// All native response contents are kept in memory, not printed or persisted. In particular,
// account emails, config layers, credentials and arbitrary RPC error messages never leave here.
struct Metadata {
    child: Child,
    input: ChildStdin,
    output: Receiver<Result<Value>>,
    deadline: Instant,
    serial: u32,
}
impl Metadata {
    fn start(cwd: &Path) -> Result<Self> {
        let exe = crate::coordination::resolve_executable("codex")?;
        let mut child = Command::new(exe)
            .args(["app-server", "--stdio"])
            .current_dir(cwd)
            .env_remove("CODEX_THREAD_ID")
            .env_remove("AGENT_ON_CONTROL_ID")
            .env_remove("AGENT_ON_CONTROL_REPO")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "cannot start native Codex metadata reader".to_string())?;
        let input = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, output) = mpsc::sync_channel(16);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            for _ in 0..1024 {
                let mut line = Vec::new();
                let read = reader
                    .by_ref()
                    .take((MAX_LINE + 1) as u64)
                    .read_until(b'\n', &mut line);
                if !matches!(read, Ok(n) if n > 0 && n <= MAX_LINE) {
                    let _ = tx.send(Err(
                        "native Codex metadata stream closed or exceeded limit".into()
                    ));
                    break;
                }
                let result = serde_json::from_slice(&line)
                    .map_err(|_| "invalid native Codex metadata response".into());
                if tx.send(result).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            deadline: Instant::now() + TIMEOUT,
            serial: 0,
        })
    }
    fn send(&mut self, message: Value) -> Result<()> {
        // Requests are small and the native process reads stdin concurrently.
        writeln!(self.input, "{message}").map_err(|_| "native Codex metadata request failed".into())
    }
    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.serial += 1;
        let id = self.serial;
        self.send(json!({"id":id,"method":method,"params":params}))?;
        loop {
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            let message = self.output.recv_timeout(remaining).map_err(|_| {
                format!("native Codex metadata unavailable ({method}); check login/network/CLI version; no model override guessed")
            })??;
            if message["id"].as_u64() != Some(id as u64) {
                continue;
            }
            if message.get("error").is_some() {
                return Err(format!(
                    "native Codex metadata RPC failed ({method}); no model substituted"
                ));
            }
            return message
                .get("result")
                .cloned()
                .ok_or_else(|| "native Codex response missing result".into());
        }
    }
}
impl Drop for Metadata {
    fn drop(&mut self) {
        // Only the metadata child started above; never terminate an existing host session.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// No discovery or policy inference at launch: an omitted model belongs to the host.
/// This records the instruction, not proof of the model actually used or its entitlement.
pub(crate) fn launch_choice(requested: Option<&str>) -> Resolution {
    Resolution {
        requested_model: requested.map(str::to_owned),
        configured_model: None,
        selected_model: requested.map(str::to_owned),
        reasoning_effort_override: None,
        source: if requested.is_some() {
            "explicit"
        } else {
            "host-settings"
        }
        .into(),
        account_type: None,
        verification: "not-probed".into(),
        checked_at: Utc::now().to_rfc3339(),
    }
}

fn string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
}

pub(crate) fn resolve(cwd: &Path, requested: Option<&str>) -> Result<Resolution> {
    let mut resolution = launch_choice(requested);
    // An explicit choice is authoritative, including private/unlisted model identifiers.
    // The host still enforces its own permissions and account policy on the real launch.
    if requested.is_some() {
        return Ok(resolution);
    }
    let mut rpc = Metadata::start(cwd)?;
    rpc.request("initialize", json!({"clientInfo":{"name":"agent-on-model-check","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":false}}))?;
    rpc.send(json!({"method":"initialized","params":{}}))?;
    let config = rpc.request("config/read", json!({"cwd":cwd,"includeLayers":false}))?;
    let config = config
        .get("config")
        .ok_or("native Codex response missing config")?;
    resolution.configured_model = string(config, "model");
    // Custom providers keep their own settings. Do not query or reinterpret their account.
    if string(config, "model_provider").is_some_and(|provider| provider != "openai") {
        resolution.source = "host-config".into();
        return Ok(resolution);
    }
    let account = rpc.request("account/read", json!({"refreshToken":false}))?;
    let account_type = string(&account["account"], "type");
    if account["requiresOpenaiAuth"] == true && account["account"].is_null() {
        return Err("Codex login required; model selection cannot repair authentication".into());
    }
    resolution.account_type = account_type.clone();
    // The ChatGPT model catalogue is not proof of API-key entitlement. No provider/billing switch.
    if account_type.as_deref() != Some("chatgpt") {
        resolution.source = "host-config".into();
        return Ok(resolution);
    }
    let requirements = rpc.request("configRequirements/read", json!({}))?;
    let requirements = requirements
        .get("requirements")
        .ok_or("native Codex response missing requirements")?;
    let managed = &requirements["models"]["newThread"];
    let managed_model = string(managed, "model");
    let managed_effort = string(managed, "modelReasoningEffort");
    let mut models = Vec::new();
    let mut cursor = Value::Null;
    for page in 0..16 {
        let list = rpc.request(
            "model/list",
            json!({"includeHidden":false,"limit":100,"cursor":cursor}),
        )?;
        models.extend(
            list["data"]
                .as_array()
                .ok_or("native Codex response missing model catalogue")?
                .iter()
                .filter(|m| m["hidden"] != true)
                .cloned(),
        );
        cursor = list
            .get("nextCursor")
            .cloned()
            .ok_or("native Codex response missing catalogue cursor")?;
        if cursor.is_null() {
            break;
        }
        if !cursor.is_string() || page == 15 {
            return Err(
                "native Codex model catalogue pagination incomplete; no fallback guessed".into(),
            );
        }
    }
    resolution.verification = "catalog-only".into();
    let desired = managed_model
        .as_ref()
        .or(resolution.configured_model.as_ref());
    let configured = desired.and_then(|model| {
        models
            .iter()
            .find(|m| m["model"].as_str() == Some(model.as_str()))
    });
    let selected = if let Some(model) = configured {
        resolution.source = if managed_model.is_some() {
            "managed-default"
        } else {
            "configured"
        }
        .into();
        model
    } else if managed_model.is_some() {
        return Err("managed model is absent from this CLI catalogue; no model substituted; check account/client policy".into());
    } else if desired.is_some() {
        return Err("configured model is absent from this CLI catalogue; no model substituted; model choice belongs to the user".into());
    } else {
        let defaults: Vec<_> = models.iter().filter(|m| m["isDefault"] == true).collect();
        if defaults.len() != 1 {
            return Err(
                "native Codex catalogue has no unique visible default; no model override guessed"
                    .into(),
            );
        }
        // A read-only observation in model-check, never an execution instruction.
        resolution.source = "native-default".into();
        defaults[0]
    };
    resolution.selected_model =
        Some(string(selected, "model").ok_or("native default has no model id")?);
    let effort = managed_effort
        .as_ref()
        .cloned()
        .or_else(|| string(config, "model_reasoning_effort"));
    let supported = selected["supportedReasoningEfforts"]
        .as_array()
        .ok_or("native model missing reasoning capabilities")?;
    let supports = |effort: &str| {
        supported
            .iter()
            .any(|s| s["reasoningEffort"].as_str() == Some(effort))
    };
    if let Some(effort) = effort {
        if !supports(&effort) {
            if managed_effort.is_some() {
                return Err(
                    "managed reasoning effort is incompatible; no policy override attempted".into(),
                );
            }
            return Err("configured reasoning effort is incompatible; no effort substituted; choice belongs to the user".into());
        }
    }
    Ok(resolution)
}
