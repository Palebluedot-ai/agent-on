//! A read-only projection of existing ledgers, not another task store or model runtime.
//! One collector serves all browser clients; opening a page never launches a model.

use crate::{coordination as c, project_status::collect, system_dashboard};
use clap::Args;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Args)]
pub(crate) struct DashboardArgs {
    #[arg(long, conflicts_with_all = ["system", "root", "project"])]
    pub(crate) repo: Option<PathBuf>,
    /// Aggregate local projects without dispatching or installing hooks
    #[arg(long, conflicts_with = "refresh_landing")]
    pub(crate) system: bool,
    /// Bounded project discovery root (repeatable; default: ~/Projects)
    #[arg(long, requires = "system")]
    root: Vec<PathBuf>,
    /// Explicit project outside discovery roots or below excluded/nested directories
    #[arg(long, requires = "system")]
    project: Vec<PathBuf>,
    #[arg(long)]
    pub(crate) json: bool,
    /// Serve an optional local browser panel until Ctrl-C
    #[arg(long, conflicts_with = "json")]
    pub(crate) serve: bool,
    #[arg(long, default_value_t = 8765)]
    port: u16,
    /// Opt in to one shared network refresh every 60 seconds (no model calls)
    #[arg(long, requires = "serve")]
    refresh_landing: bool,
}

pub(crate) fn run(args: DashboardArgs) -> i32 {
    let repo = args
        .repo
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let projection = if args.system {
        Projection::System(system_dashboard::Collector::new(args.root, args.project))
    } else {
        Projection::Project(repo)
    };
    let result = if args.serve {
        serve(projection, args.port, args.refresh_landing)
    } else {
        let mut projection = projection;
        projection.collect().map(|value| {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                );
            } else if value["scope"] == "system" {
                println!(
                    "Agent-On · 本机系统总览\n项目 {} · 任务 {} · 需关注 {}",
                    value["counts"]["projects"],
                    value["counts"]["active_tasks"],
                    value["counts"]["errors"]
                );
                for project in value["projects"].as_array().into_iter().flatten() {
                    println!(
                        "{} · 任务 {} · {}",
                        project["name"].as_str().unwrap_or("?"),
                        project["tasks"],
                        if project["available"] == false {
                            "记录不可读"
                        } else if project["connected"] == true {
                            "已有记录"
                        } else {
                            "未接入"
                        }
                    );
                }
                println!("打开系统面板：agent-on dashboard --system --serve");
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

enum Projection {
    Project(PathBuf),
    System(system_dashboard::Collector),
}
impl Projection {
    fn collect(&mut self) -> Result<Value, String> {
        match self {
            Self::Project(repo) => collect(repo),
            Self::System(collector) => Ok(collector.collect()),
        }
    }
}
fn serve(mut projection: Projection, port: u16, refresh: bool) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let address = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .to_string();
    let state = Arc::new(Mutex::new(projection.collect()?));
    let shared = state.clone();
    let refresh_repo = match &projection {
        Projection::Project(repo) => Some(repo.clone()),
        Projection::System(_) => None,
    };
    std::thread::spawn(move || loop {
        match projection.collect() {
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
        let project = refresh_repo.ok_or("系统总览只读本机缓存；联网刷新请在项目面板显式开启")?;
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
    let mut details = std::collections::BTreeMap::new();
    for stream in listener.incoming() {
        let mut stream = stream.map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        // Small bounded GET requests; no request can invoke shell, merge or deletion.
        let _ = respond(&mut stream, &address, &state, &mut details);
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
    if path.starts_with("/api/")
        && !headers
            .iter()
            .any(|(k, v)| k == "x-agent-on" && *v == "dashboard")
    {
        return Err(403);
    }
    if path != "/" && path != "/api/status" && project_route(path).is_none() {
        return Err(404);
    }
    Ok(path)
}

fn project_route(path: &str) -> Option<(&str, bool)> {
    let (id, api) = if let Some(id) = path.strip_prefix("/api/projects/") {
        (id, true)
    } else {
        (path.strip_prefix("/projects/")?, false)
    };
    // Opaque fixed-length identity, never a client-supplied filesystem path.
    (id.len() == 17 && id.starts_with('p') && id[1..].bytes().all(|b| b.is_ascii_hexdigit()))
        .then_some((id, api))
}

fn respond(
    stream: &mut TcpStream,
    address: &str,
    state: &Arc<Mutex<Value>>,
    details: &mut std::collections::BTreeMap<String, (Instant, Value)>,
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
        Ok("/") => {
            let system = state.lock().unwrap_or_else(|e| e.into_inner())["scope"] == "system";
            (
                200,
                "text/html; charset=utf-8",
                if system {
                    include_str!("system_dashboard.html")
                } else {
                    include_str!("dashboard.html")
                }
                .to_string(),
            )
        }
        Ok("/api/status") => (
            200,
            "application/json",
            state.lock().unwrap_or_else(|e| e.into_inner()).to_string(),
        ),
        Ok(path) => {
            let (id, api) = project_route(path).unwrap();
            let project = state.lock().unwrap_or_else(|e| e.into_inner())["projects"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|p| p["id"] == id)
                .cloned();
            match project {
                Some(project) if project["detail_available"] == true => {
                    if !api {
                        (
                            200,
                            "text/html; charset=utf-8",
                            include_str!("dashboard.html").to_string(),
                        )
                    } else {
                        let cached = details
                            .get(id)
                            .filter(|(at, _)| at.elapsed() < Duration::from_secs(5))
                            .map(|(_, v)| v.clone());
                        let value=cached.unwrap_or_else(|| {
                            let value=collect(Path::new(project["repo"].as_str().unwrap_or_default()))
                                .unwrap_or_else(|e|json!({"collection_error":e,"patrol":{"available":false,"error":e}}));
                            if details.len()>=128 { details.clear(); }
                            details.insert(id.to_string(),(Instant::now(),value.clone()));value
                        });
                        (200, "application/json", value.to_string())
                    }
                }
                _ => (
                    404,
                    "text/plain",
                    "项目未发现或尚无 Git 项目详情".to_string(),
                ),
            }
        }
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
    #[test]
    fn system_detail_routes_are_opaque_and_read_only() {
        let addr = "127.0.0.1:8765";
        for path in [
            "/projects/p0123456789abcdef",
            "/api/projects/p0123456789abcdef",
        ] {
            let request =
                format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nX-Agent-On: dashboard\r\n");
            assert_eq!(allowed(&request, addr), Ok(path));
        }
        for path in [
            "/projects/../../secret",
            "/projects/p0123456789abcdef/..",
            "/api/projects/%2Ftmp",
            "/projects/p0123456789abcdef?repo=/tmp",
            "/projects/pé123456789abcde",
        ] {
            assert!(project_route(path).is_none());
        }
        assert_eq!(
            allowed(
                &format!("GET /api/projects/p0123456789abcdef HTTP/1.1\r\nHost: {addr}\r\n"),
                addr
            ),
            Err(403)
        );
    }
}
