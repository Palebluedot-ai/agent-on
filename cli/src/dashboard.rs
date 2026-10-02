//! A read-only projection of existing ledgers, not another task store or model runtime.
//! One collector serves all browser clients; opening a page never launches a model.

use crate::{coordination as c, landing, oncall, worktree};
use clap::Args;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Args)]
pub(crate) struct DashboardArgs {
    #[arg(long)]
    repo: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    /// Serve an optional local browser panel until Ctrl-C
    #[arg(long, conflicts_with = "json")]
    serve: bool,
    #[arg(long, default_value_t = 8765)]
    port: u16,
    /// Opt in to one shared network refresh every 60 seconds (no model calls)
    #[arg(long, requires = "serve")]
    refresh_landing: bool,
}

fn section(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => json!({"available":true,"data":value}),
        Err(error) => json!({"available":false,"error":error}),
    }
}

fn parsed(code: i32, text: String) -> Result<Value, String> {
    if code != 0 {
        return Err(text.trim().to_string());
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Reads only the primary/common-dir facts, so linked worktrees see the same project.
pub(crate) fn collect(repo: &Path) -> Result<Value, String> {
    let store = c::Store::open(repo)?;
    let common = worktree::common_git_dir(&store.repo)?;
    let config = section(
        store
            .config()
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
    );
    let mut patrol = section(c::scan(&store, None));
    if let Some(sessions) = patrol["data"]["sessions"].as_array_mut() {
        for session in sessions {
            let age = session["heartbeat_at"]
                .as_str()
                .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
                .map(|at| {
                    (chrono::Utc::now() - at.with_timezone(&chrono::Utc))
                        .num_seconds()
                        .max(0)
                });
            let pids: [Option<u32>; 2] = [
                session["pid"].as_u64().map(|p| p as u32),
                session["child_pid"].as_u64().map(|p| p as u32),
            ];
            session["heartbeat_age_seconds"] = json!(age);
            session["process_alive"] = json!(if pids.iter().any(Option::is_some) {
                Some(pids.into_iter().flatten().any(c::pid_alive))
            } else {
                None
            });
        }
    }
    let raw = c::git(&store.repo, &["worktree", "list", "--porcelain"])?;
    let mut trees = Vec::new();
    let tasks = store.tasks();
    for (i, tree) in worktree::parse_worktrees(&raw).into_iter().enumerate() {
        let facts = (|| -> Result<Value, String> {
            let status = c::git(&tree.path, &["status", "--porcelain=v1", "-z"])?;
            let unique = c::git(
                &tree.path,
                &["rev-list", "--count", "HEAD", "--not", "--remotes"],
            )?;
            Ok(json!({"clean":status.is_empty(),"local_only_commits":unique.parse::<u64>().ok()}))
        })();
        trees.push(json!({"path":tree.path,"head":tree.head,"branch":tree.branch,
            "primary":i==0,"locked":tree.locked,"prunable":tree.prunable,"facts":section(facts),
            "managed":tasks.as_ref().ok().map(|tasks|tasks.iter().any(|t|t.managed && t.worktree==tree.path))}));
    }
    let (code, output) = oncall::status(&store.repo, true);
    let oncall = section(parsed(code, output));
    let opts = landing::LandingOpts {
        json: true,
        ..Default::default()
    };
    let (code, output) = landing::run_status(&store.repo, &opts);
    let landing = section(parsed(code, output));
    for tree in &mut trees {
        if let Some(track) = landing["data"]["snapshot"]["tracks"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|track| track["worktree"] == tree["path"])
        {
            tree["lifecycle"] = track["lifecycle"].clone();
            tree["lifecycle_reason"] = track["lifecycle_reason"].clone();
        }
    }
    let snapshot = landing::load_snapshot(&common).ok().flatten();
    let age = snapshot
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s.generated_at).ok())
        .map(|at| {
            (chrono::Utc::now() - at.with_timezone(&chrono::Utc))
                .num_seconds()
                .max(0)
        });
    let repo_head = c::git(&store.repo, &["rev-parse", "HEAD"])?;
    let mut attention = Vec::new();
    if config["data"]["enabled"] != true {
        attention.push("巡逻未启用：现存 Git 树可见，宿主任务与会话不保证自动捕获。".to_string());
    }
    if !landing["available"].as_bool().unwrap_or(false) {
        attention.push(
            "尚无可用 PR 快照；运行 agent-on landing refresh，或启用面板的 --refresh-landing。"
                .into(),
        );
    } else if age.is_none_or(|n| n > 120) {
        attention.push("PR 取证已超过两分钟；这是上次观察，不能据此直接决定合并。".into());
    }
    let hooks = worktree::git(&store.repo, &["config", "--get", "core.hooksPath"]).ok();
    if hooks.is_none() {
        attention.push("未设置原生 Git hooks；需要 agent-on worktree hooks install。".into());
    }
    let daily_gc = crate::worktree_schedule::inspect(&store.repo);
    if daily_gc.installed && !daily_gc.healthy() {
        attention.push(crate::worktree_schedule::render_status(&daily_gc));
    }
    let daily_gc = section(serde_json::to_value(daily_gc).map_err(|error| error.to_string()));
    Ok(
        json!({"schema_version":1,"observed_at":c::now(),"repo":store.repo,"head":repo_head,
        "config":config,"patrol":patrol,"oncall":oncall,"landing":landing,
        "landing_age_seconds":age,"worktrees":trees,"attention":attention,
        "hooks_path":hooks,"daily_gc":daily_gc,
        "coverage":"本机 Git 事实与已有回执；Stop/退出/结果自述不等于验证通过或任务释放"}),
    )
}

pub(crate) fn run(args: DashboardArgs) -> i32 {
    let repo = args
        .repo
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let result = if args.serve {
        serve(&repo, args.port, args.refresh_landing)
    } else {
        collect(&repo).map(|value| {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                );
            } else {
                println!(
                    "Agent-On · {}\n观察时间：{}",
                    value["repo"].as_str().unwrap_or("?"),
                    value["observed_at"].as_str().unwrap_or("?")
                );
                let report = &value["patrol"]["data"];
                println!(
                    "任务 {} · 会话 {} · worktree {} · 重叠 {}",
                    count(&report["tasks"]),
                    count(&report["sessions"]),
                    count(&value["worktrees"]),
                    count(&report["conflicts"])
                );
                for note in value["attention"].as_array().into_iter().flatten() {
                    println!("{}", note.as_str().unwrap_or("?"));
                }
                println!("打开实时面板：agent-on dashboard --serve --refresh-landing");
            }
        })
    };
    if let Err(error) = result {
        eprintln!("agent-on dashboard: {error}");
        1
    } else {
        0
    }
}
fn count(value: &Value) -> usize {
    value.as_array().map_or(0, Vec::len)
}

fn serve(repo: &Path, port: u16, refresh: bool) -> Result<(), String> {
    let store = c::Store::open(repo)?;
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let address = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .to_string();
    let state = Arc::new(Mutex::new(collect(&store.repo)?));
    let shared = state.clone();
    let project = store.repo.clone();
    std::thread::spawn(move || loop {
        match collect(&project) {
            Ok(mut value) => {
                value["remote_refresh_enabled"] = json!(refresh);
                let mut previous = shared.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(remote) = previous.get("remote_refresh") {
                    value["remote_refresh"] = remote.clone();
                }
                *previous = value;
            }
            Err(error) => {
                let mut value = shared.lock().unwrap_or_else(|e| e.into_inner());
                value["collection_error"] = json!(error);
            }
        }
        std::thread::sleep(Duration::from_secs(5));
    });
    if refresh {
        let project = store.repo.clone();
        let shared = state.clone();
        std::thread::spawn(move || loop {
            let started = Instant::now();
            let outcome = refresh_once(&project);
            let mut value = shared.lock().unwrap_or_else(|e| e.into_inner());
            value["remote_refresh"] = json!({"at":c::now(),"error":outcome.err()});
            drop(value);
            // Exactly one refresher per server, irrespective of browser client count.
            std::thread::sleep(Duration::from_secs(60).saturating_sub(started.elapsed()));
        });
    }
    println!(
        "Agent-On 面板 http://{address}/\n本机观察每 5 秒；PR 联网刷新 {}；Ctrl-C 关闭。",
        if refresh {
            "每 60 秒"
        } else {
            "未启用（读缓存）"
        }
    );
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    for stream in listener.incoming() {
        let mut stream = stream.map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        // Small bounded GET requests; no request can invoke shell, merge or deletion.
        let _ = respond(&mut stream, &address, &state);
    }
    Ok(())
}

fn refresh_once(repo: &Path) -> Result<(), String> {
    use std::process::{Command, Stdio};
    let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .args(["landing", "refresh", "--repo"])
        .arg(repo)
        .arg("--json")
        .env("GH_HTTP_TIMEOUT", "10")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return if status.success() {
                Ok(())
            } else {
                Err("PR 联网取证失败；保留上次快照，请运行 landing refresh 查看原因。".into())
            };
        }
        if started.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("PR 取证超时；本机状态仍继续更新。".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn allowed<'a>(request: &'a str, address: &str) -> Result<&'a str, u16> {
    let mut lines = request.lines();
    let mut first = lines.next().unwrap_or("").split_whitespace();
    if first.next() != Some("GET") {
        return Err(405);
    }
    let path = first.next().ok_or(400_u16)?;
    let headers: Vec<_> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim()))
        .collect();
    if !headers.iter().any(|(k, v)| k == "host" && *v == address) {
        return Err(403);
    }
    if headers
        .iter()
        .any(|(k, v)| k == "origin" && *v != format!("http://{address}"))
    {
        return Err(403);
    }
    if path == "/api/status"
        && !headers
            .iter()
            .any(|(k, v)| k == "x-agent-on" && *v == "dashboard")
    {
        return Err(403);
    }
    if path != "/" && path != "/api/status" {
        return Err(404);
    }
    Ok(path)
}

fn respond(
    stream: &mut TcpStream,
    address: &str,
    state: &Arc<Mutex<Value>>,
) -> std::io::Result<()> {
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while request.len() < 8192 {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..n]);
        if request.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let text = String::from_utf8_lossy(&request);
    let (code, kind, body) = match allowed(&text, address) {
        Ok("/") => (
            200,
            "text/html; charset=utf-8",
            include_str!("dashboard.html").to_string(),
        ),
        Ok(_) => (
            200,
            "application/json",
            state.lock().unwrap_or_else(|e| e.into_inner()).to_string(),
        ),
        Err(code) => (code, "text/plain", format!("request refused ({code})")),
    };
    write!(stream,"HTTP/1.1 {code} Response\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'\r\nConnection: close\r\n\r\n{body}",body.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_api_requires_same_host_origin_and_custom_header() {
        let addr = "127.0.0.1:8765";
        assert_eq!(
            allowed(
                "GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:8765\r\nX-Agent-On: dashboard\r\n",
                addr
            ),
            Ok("/api/status")
        );
        for request in ["GET /api/status HTTP/1.1\r\nHost: evil.test\r\nX-Agent-On: dashboard\r\n","GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:8765\r\nOrigin: https://evil.test\r\nX-Agent-On: dashboard\r\n","GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:8765\r\n"] {assert_eq!(allowed(request,addr),Err(403))}
        assert_eq!(allowed("POST /api/status HTTP/1.1", addr), Err(405));
    }
}
