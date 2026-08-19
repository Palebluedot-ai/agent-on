//! Single on-call registry + cross-window command routing.
//!
//! One window is on call at a time. Merging, outbound communication, and
//! window-to-window messaging belong to that window only. Every other window
//! reroutes such commands instead of running them.
//!
//! Storage lives beside the lane registry in the common git dir, so every
//! worktree of the repo reads the same record (a per-worktree file copy such
//! as `docs/babysit.md` cannot serve as the address book — each worktree
//! carries its own stale copy of it).

use crate::worktree;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const RECORD_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OncallRecord {
    version: u8,
    /// SendMessage address of the on-call window (session name or a prefix of it).
    pub(crate) session: String,
    /// Lane id of the on-call window, when it registered one.
    #[serde(default)]
    pub(crate) lane: String,
    /// Absolute worktree root of the on-call window — the identity key.
    pub(crate) worktree: String,
    pub(crate) started_at: String,
    #[serde(default)]
    pub(crate) note: String,
}

/// What the current window is, relative to the on-call registry.
#[derive(Debug, Clone)]
pub(crate) enum Role {
    /// Nobody registered, or the registered worktree is gone (stale) — the
    /// routing gate fails open and the repo's no-on-call rules apply.
    Nobody,
    /// This window is the on-call window.
    Oncall(OncallRecord),
    /// Someone else is on call; this window is a feature window.
    Feature(OncallRecord),
}

/// Class of command that belongs to the on-call window only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// Merge / remote public-state writes: merge, update-branch, close, tags, releases.
    Merge,
    /// Outbound communication to humans or other systems: PR/issue comments,
    /// chat webhooks, mail.
    Outbound,
    /// Window-to-window messaging that is not addressed to the on-call window.
    CrossWindow,
}

impl Action {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Action::Merge => "合并 / 远端公共态写入",
            Action::Outbound => "对外通信",
            Action::CrossWindow => "跨窗口沟通",
        }
    }
}

fn oncall_path(cwd: &Path) -> Result<PathBuf, String> {
    Ok(worktree::common_git_dir(cwd)?
        .join("agent-on")
        .join("oncall.json"))
}

fn canon(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

pub(crate) fn load(cwd: &Path) -> Result<Option<OncallRecord>, String> {
    let path = oncall_path(cwd)?;
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let record: OncallRecord =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if record.version != RECORD_VERSION {
        return Err(format!(
            "unsupported on-call record version {} in {} (expected {})",
            record.version,
            path.display(),
            RECORD_VERSION
        ));
    }
    Ok(Some(record))
}

fn save(cwd: &Path, record: &OncallRecord) -> Result<(), String> {
    let path = oncall_path(cwd)?;
    let parent = path
        .parent()
        .ok_or_else(|| "invalid on-call registry path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    let raw =
        serde_json::to_string_pretty(record).map_err(|e| format!("serialize on-call: {e}"))?;
    fs::write(&path, format!("{raw}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}

/// A registered worktree that no longer exists is not evidence of anyone being
/// on call — treat it as nobody so the gate cannot deadlock the whole repo.
fn is_stale(record: &OncallRecord) -> bool {
    !Path::new(&record.worktree).exists()
}

pub(crate) fn role_at(cwd: &Path) -> Role {
    let Ok(Some(record)) = load(cwd) else {
        return Role::Nobody;
    };
    if is_stale(&record) {
        return Role::Nobody;
    }
    let here = worktree::repo_root(cwd)
        .map(|p| canon(&p))
        .unwrap_or_else(|_| canon(cwd));
    if canon(Path::new(&record.worktree)) == here {
        Role::Oncall(record)
    } else {
        Role::Feature(record)
    }
}

// ---------------------------------------------------------------- classify

fn tokens(cmd: &str) -> Vec<String> {
    shlex::split(cmd).unwrap_or_else(|| cmd.split_whitespace().map(str::to_string).collect())
}

fn base_name(token: &str) -> String {
    Path::new(token)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(token)
        .to_ascii_lowercase()
}

/// Sub-commands after `gh <noun>` that belong to the on-call window.
fn gh_action(words: &[String]) -> Option<Action> {
    let noun = words.first()?.as_str();
    let verb = words.get(1).map(String::as_str).unwrap_or("");
    match (noun, verb) {
        ("pr", "merge") | ("pr", "close") | ("pr", "reopen") => Some(Action::Merge),
        ("release", "create") | ("release", "edit") | ("release", "delete") => Some(Action::Merge),
        ("pr", "comment") | ("pr", "review") => Some(Action::Outbound),
        ("issue", "comment") | ("issue", "create") | ("issue", "close") | ("issue", "reopen") => {
            Some(Action::Outbound)
        }
        ("api", _) => gh_api_action(&words[1..]),
        _ => None,
    }
}

fn is_write_method(method: &str) -> bool {
    matches!(
        method.to_ascii_uppercase().as_str(),
        "PUT" | "POST" | "PATCH" | "DELETE"
    )
}

/// Flags whose *next* token is a value, not the endpoint path.
const GH_API_VALUE_FLAGS: &[&str] = &[
    "-f",
    "--field",
    "-F",
    "--raw-field",
    "-H",
    "--header",
    "-q",
    "--jq",
    "-t",
    "--template",
    "--input",
    "--hostname",
    "-p",
    "--preview",
    "--cache",
];

/// `gh api` is only on-call territory when it writes remote public state.
/// Read calls and GraphQL queries (which also use POST) stay open.
fn gh_api_action(args: &[String]) -> Option<Action> {
    let mut writes = false;
    let mut endpoint: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "-X" || arg == "--method" {
            if let Some(method) = args.get(i + 1) {
                writes |= is_write_method(method);
            }
            i += 2;
            continue;
        }
        if let Some(method) = arg.strip_prefix("--method=") {
            writes |= is_write_method(method);
            i += 1;
            continue;
        }
        if GH_API_VALUE_FLAGS.contains(&arg) {
            i += 2;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        if endpoint.is_none() {
            endpoint = Some(arg);
        }
        i += 1;
    }
    if !writes {
        return None;
    }
    let path = endpoint?;
    if path.contains("/pulls/") || path.contains("/merges") || path.ends_with("/merge") {
        return Some(Action::Merge);
    }
    if path.contains("/comments") || path.contains("/reviews") || path.contains("/issues") {
        return Some(Action::Outbound);
    }
    None
}

fn git_push_action(args: &[String]) -> Option<Action> {
    let mut refs = Vec::new();
    for arg in args {
        if arg == "--tags" || arg == "--follow-tags" {
            return Some(Action::Merge);
        }
        if !arg.starts_with('-') {
            refs.push(arg.as_str());
        }
    }
    // refs[0] is the remote; the rest are refspecs.
    for spec in refs.iter().skip(1) {
        let dst = spec.rsplit(':').next().unwrap_or(spec);
        let name = dst.trim_start_matches('+');
        if name.starts_with("refs/tags/") {
            return Some(Action::Merge);
        }
        let short = name.trim_start_matches("refs/heads/");
        if matches!(short, "main" | "master") {
            return Some(Action::Merge);
        }
        // v1.2.3 style tag pushed by short name
        if name.starts_with('v') && name[1..].starts_with(|c: char| c.is_ascii_digit()) {
            return Some(Action::Merge);
        }
    }
    None
}

const CHAT_HOSTS: &[&str] = &[
    "hooks.slack.com",
    "slack.com/api",
    "webhook.office.com",
    "outlook.office.com",
    "office.com/webhook",
    "discord.com/api/webhooks",
    "discordapp.com/api/webhooks",
    "api.telegram.org",
    "chat.googleapis.com",
    "graph.microsoft.com/v1.0/teams",
    "graph.microsoft.com/v1.0/chats",
];

const MAIL_COMMANDS: &[&str] = &["sendmail", "mail", "mailx", "mutt", "msmtp"];
const CHAT_COMMANDS: &[&str] = &["slack", "teams", "msteams", "slack-cli"];

/// Which on-call-only action this shell command performs, if any.
///
/// This is a deny-list: it catches the known shapes, not every possible one.
/// The discipline layer (kit/babysit/ROUTING.md) covers the rest.
pub(crate) fn classify_bash(cmd: &str) -> Option<Action> {
    let toks = tokens(cmd);
    let mut i = 0;
    while i < toks.len() {
        let name = base_name(&toks[i]);
        let rest: Vec<String> = toks[i + 1..]
            .iter()
            .take_while(|t| !matches!(t.as_str(), "&&" | "||" | ";" | "|"))
            .cloned()
            .collect();
        match name.as_str() {
            "gh" => {
                let words: Vec<String> = rest
                    .iter()
                    .filter(|t| !t.starts_with('-'))
                    .cloned()
                    .collect();
                // `gh api` keeps its flags: the method matters.
                let hit = if words.first().map(String::as_str) == Some("api") {
                    let mut with_flags = vec!["api".to_string()];
                    with_flags.extend(
                        rest.iter()
                            .skip_while(|t| t.as_str() != "api")
                            .skip(1)
                            .cloned(),
                    );
                    gh_action(&with_flags)
                } else {
                    gh_action(&words)
                };
                if hit.is_some() {
                    return hit;
                }
            }
            "git" => {
                let sub = rest.iter().find(|t| !t.starts_with('-'));
                if sub.map(String::as_str) == Some("push") {
                    let args: Vec<String> = rest
                        .iter()
                        .skip_while(|t| t.as_str() != "push")
                        .skip(1)
                        .cloned()
                        .collect();
                    if let Some(hit) = git_push_action(&args) {
                        return Some(hit);
                    }
                }
            }
            "curl" | "wget" | "http" | "httpie" => {
                let joined = rest.join(" ").to_ascii_lowercase();
                if CHAT_HOSTS.iter().any(|host| joined.contains(host)) {
                    return Some(Action::Outbound);
                }
            }
            "osascript" => {
                let joined = rest.join(" ").to_ascii_lowercase();
                if joined.contains("messages") || joined.contains("mail") {
                    return Some(Action::Outbound);
                }
            }
            other => {
                if MAIL_COMMANDS.contains(&other) || CHAT_COMMANDS.contains(&other) {
                    return Some(Action::Outbound);
                }
            }
        }
        i += 1;
    }
    None
}

/// Session names carry a per-window suffix, so the registered address is often
/// a prefix of the real name (or the other way round).
pub(crate) fn addresses_match(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim(), b.trim());
    !a.is_empty() && !b.is_empty() && (a.starts_with(b) || b.starts_with(a))
}

// ------------------------------------------------------------------ commands

pub fn claim(
    cwd: &Path,
    session: &str,
    lane: Option<&str>,
    note: &str,
    force: bool,
) -> (i32, String) {
    let result = (|| -> Result<String, String> {
        if session.trim().is_empty() {
            return Err("--session cannot be empty (值守窗口的 SendMessage 地址)".to_string());
        }
        let here = worktree::repo_root(cwd)?;
        let here_canon = canon(&here);
        if let Some(existing) = load(cwd)? {
            let same = canon(Path::new(&existing.worktree)) == here_canon;
            if !same && !is_stale(&existing) && !force {
                return Err(format!(
                    "已有值守在班：{} (worktree {}，自 {})；同一时间至多一个值守。\n\
交接请用 --force，或让在班窗口先跑 `agent-on oncall release`",
                    existing.session, existing.worktree, existing.started_at
                ));
            }
        }
        let lane_id = match lane {
            Some(v) => v.to_string(),
            None => worktree::lane_id_for_worktree(cwd).unwrap_or_default(),
        };
        let record = OncallRecord {
            version: RECORD_VERSION,
            session: session.trim().to_string(),
            lane: lane_id,
            worktree: here_canon.display().to_string(),
            started_at: Utc::now().to_rfc3339(),
            note: note.to_string(),
        };
        save(cwd, &record)?;
        Ok(format!(
            "ONCALL CLAIMED\nsession: {}\nlane: {}\nworktree: {}\nsince: {}\n\
功能窗口从此可用 `agent-on oncall status` 读到交单地址；合并 / 对外通信 / 跨窗口消息归本窗口。\n",
            record.session,
            if record.lane.is_empty() {
                "-"
            } else {
                &record.lane
            },
            record.worktree,
            record.started_at
        ))
    })();
    match result {
        Ok(text) => (0, text),
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

pub fn release(cwd: &Path, force: bool) -> (i32, String) {
    let result = (|| -> Result<String, String> {
        let Some(existing) = load(cwd)? else {
            return Ok("ONCALL: 本来就无人在班，无需下班\n".to_string());
        };
        let here = canon(&worktree::repo_root(cwd)?);
        let same = canon(Path::new(&existing.worktree)) == here;
        if !same && !is_stale(&existing) && !force {
            return Err(format!(
                "在班值守是 {}（worktree {}），本窗口不是它。\n\
确实要替它下班（窗口已关 / 交接）请加 --force——这一步会留痕在班登记，不要静默绕过",
                existing.session, existing.worktree
            ));
        }
        let path = oncall_path(cwd)?;
        fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))?;
        Ok(format!(
            "ONCALL RELEASED: {}（worktree {}）\n合并 / 对外通信闸即刻 fail-open，回退本仓「值守不在班」规则。\n",
            existing.session, existing.worktree
        ))
    })();
    match result {
        Ok(text) => (0, text),
        Err(e) => (1, format!("ERROR: {e}\n")),
    }
}

pub fn status(cwd: &Path, json: bool) -> (i32, String) {
    let record = match load(cwd) {
        Ok(v) => v,
        Err(e) => return (1, format!("ERROR: {e}\n")),
    };
    let role = role_at(cwd);
    if json {
        let value = match (&record, &role) {
            (Some(r), _) => serde_json::json!({
                "present": !is_stale(r),
                "stale": is_stale(r),
                "session": r.session,
                "lane": r.lane,
                "worktree": r.worktree,
                "since": r.started_at,
                "note": r.note,
                "self_is_oncall": matches!(role, Role::Oncall(_)),
            }),
            (None, _) => serde_json::json!({
                "present": false,
                "stale": false,
                "self_is_oncall": false,
            }),
        };
        return (0, format!("{value}\n"));
    }
    match record {
        None => (
            0,
            "ONCALL: 无人在班——合并 / 对外通信闸 fail-open，按本仓「值守不在班」规则办。\n\
上岗：agent-on oncall claim --session <本窗口会话名>\n"
                .to_string(),
        ),
        Some(r) if is_stale(&r) => (
            0,
            format!(
                "ONCALL: 登记已失效（worktree 不存在）：{} → {}\n\
闸按无人在班处理；清理登记：agent-on oncall release --force\n",
                r.session, r.worktree
            ),
        ),
        Some(r) => {
            let mine = matches!(role, Role::Oncall(_));
            (
                0,
                format!(
                    "ONCALL: {}{}\nlane: {}\nworktree: {}\nsince: {}{}\n\
交单地址 = 上面的 session；合并 / 对外通信 / 跨窗口消息统一归它。\n",
                    r.session,
                    if mine { "（就是本窗口）" } else { "" },
                    if r.lane.is_empty() { "-" } else { &r.lane },
                    r.worktree,
                    r.started_at,
                    if r.note.is_empty() {
                        String::new()
                    } else {
                        format!("\nnote: {}", r.note)
                    }
                ),
            )
        }
    }
}

/// Which window a path belongs to — the second hop of a reroute.
///
/// The feature window only needs the on-call address; working out *which*
/// lane owns the file is the on-call window's job (ROUTING §5). This turns
/// that lookup from "read the lane table by eye" into one command.
pub fn route(cwd: &Path, path: &str, json: bool) -> (i32, String) {
    let rel = match relative_to_repo(cwd, path) {
        Ok(v) => v,
        Err(e) => return (1, format!("ERROR: {e}\n")),
    };
    let records = match worktree::load_records(cwd) {
        Ok(v) => v,
        Err(e) => return (1, format!("ERROR: {e}\n")),
    };
    let hits: Vec<_> = records
        .iter()
        .filter(|r| worktree::owns_path(&r.owns, &rel))
        .collect();
    let oncall = load(cwd).ok().flatten().filter(|r| !is_stale(r));

    if json {
        let value = serde_json::json!({
            "path": rel,
            "owners": hits.iter().map(|r| serde_json::json!({
                "lane": r.id,
                "worktree": r.worktree,
                "branch": r.branch,
                "status": r.status,
                "owns": r.owns,
                "live": worktree::ownership_live(&r.status),
            })).collect::<Vec<_>>(),
            "oncall_session": oncall.as_ref().map(|r| r.session.clone()),
        });
        return (0, format!("{value}\n"));
    }

    if hits.is_empty() {
        return (
            0,
            format!(
                "ROUTE {rel}\n无主：没有任何 lane 的 owns 覆盖它。\n\
值守动作：报用户（新开一条轨？并进某条现有轨？），别自己动手改。\n"
            ),
        );
    }

    // A path is routinely inside several lanes' owns — but only a live lane
    // (active/blocked/ready) has a window behind it worth messaging. Landed
    // and parked hits are history, and dispatching to them sends work into a
    // closed window.
    let (live, history): (Vec<_>, Vec<_>) = hits
        .iter()
        .partition(|r| worktree::ownership_live(&r.status));

    let line = |r: &&&worktree::LaneRecord| -> String {
        let mine = oncall
            .as_ref()
            .map(|o| canon(Path::new(&o.worktree)) == canon(Path::new(&r.worktree)))
            .unwrap_or(false);
        let boundary = r
            .owns
            .iter()
            .find(|b| worktree::owns_path(std::slice::from_ref(*b), &rel))
            .cloned()
            .unwrap_or_default();
        format!(
            "- lane {} [{}]{}\n  worktree: {}\n  branch: {}\n  命中边界: {}（该轨共 {} 条 owns）\n",
            r.id,
            r.status,
            if mine { "（值守自己的轨）" } else { "" },
            r.worktree,
            r.branch,
            boundary,
            r.owns.len()
        )
    };

    let mut out = format!("ROUTE {rel}\n");
    if live.is_empty() {
        out.push_str("活跃轨：无——命中的都是 landed / parked 的历史轨，它们背后多半已经没有窗口了。\n");
        for r in &history {
            out.push_str(&line(r));
        }
        out.push_str(
            "值守动作：**别直接派给上面任何一条**。报用户：新开一条轨，还是让某条历史轨 `worktree edit` 重划过来。\n",
        );
        return (0, out);
    }
    for r in &live {
        out.push_str(&line(r));
    }
    if !history.is_empty() {
        out.push_str(&format!(
            "（另有 {} 条 landed / parked 历史轨也覆盖此路径，已折叠——`agent-on worktree status` 看全量）\n",
            history.len()
        ));
    }
    out.push_str(
        "值守动作：SendMessage 派给上面活跃轨的会话，并给原窗口回一条「已派给 X」（ROUTING §5）。\n",
    );
    (0, out)
}

/// Normalise `path` to a repo-relative path, the form lane `owns` uses.
fn relative_to_repo(cwd: &Path, path: &str) -> Result<String, String> {
    let repo = worktree::repo_root(cwd)?;
    let repo = canon(&repo);
    let raw = Path::new(path);
    let abs = if raw.is_absolute() {
        canon(raw)
    } else {
        canon(&cwd.join(raw))
    };
    match abs.strip_prefix(&repo) {
        Ok(rel) => Ok(rel.to_string_lossy().replace('\\', "/")),
        // A path that does not exist on disk cannot be canonicalised; fall back
        // to treating it as already repo-relative.
        Err(_) => Ok(path.trim_start_matches("./").to_string()),
    }
}

pub fn whoami(cwd: &Path, json: bool) -> (i32, String) {
    let role = role_at(cwd);
    if json {
        let value = match &role {
            Role::Nobody => serde_json::json!({"role": "none", "is_oncall": false}),
            Role::Oncall(r) => {
                serde_json::json!({"role": "oncall", "is_oncall": true, "session": r.session})
            }
            Role::Feature(r) => {
                serde_json::json!({"role": "feature", "is_oncall": false, "oncall_session": r.session})
            }
        };
        return (0, format!("{value}\n"));
    }
    let text = match &role {
        Role::Nobody => "NONE: 无人在班；值守闸 fail-open\n".to_string(),
        Role::Oncall(r) => format!(
            "ONCALL: 本窗口是值守（{}）；合并 / 对外通信 / 跨窗口消息归你\n",
            r.session
        ),
        Role::Feature(r) => format!(
            "FEATURE: 本窗口不是值守；在班值守 = {}（{}）\n\
合并 / 对外通信 / 跨窗口消息一律转投它\n",
            r.session, r.worktree
        ),
    };
    (0, text)
}

// -------------------------------------------------------------- guard entry

fn tool_name(data: &Value) -> String {
    data.get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn tool_cwd(data: &Value) -> PathBuf {
    let input = data.get("tool_input").unwrap_or(&Value::Null);
    let raw = data
        .get("cwd")
        .or_else(|| input.get("workdir"))
        .or_else(|| input.get("cwd"))
        .and_then(Value::as_str)
        .map(str::to_string);
    match raw {
        Some(v) => PathBuf::from(v),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

fn message_recipient(data: &Value) -> String {
    let input = data.get("tool_input").unwrap_or(&Value::Null);
    input
        .get("to")
        .or_else(|| input.get("recipient"))
        .or_else(|| input.get("agent"))
        .or_else(|| input.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// The block message is the routing protocol in executable form: it names the
/// on-call address, hands over a fill-in reroute template, and lists the two
/// legitimate escape hatches (both of which change the registry, so they leave
/// a trace).
fn block_text(action: Action, record: &OncallRecord, what: &str) -> String {
    format!(
        "⛔ 跨窗口指令路由拦截（值守专属动作：{}）\n\
本窗口不是值守窗口。合并 / 对外通信 / 跨窗口消息在值守在班期间唯一归值守（kit/babysit/ROUTING.md）。\n\
在班值守：{}（worktree {}，自 {}）\n\
被拦内容：{}\n\
\n\
下一步三选一：\n\
  1）转投（默认）——不执行本条，改用 SendMessage 发给值守，模板：\n\
     to: \"{}\"\n\
     【转投】来源窗口 <本轨 lane>｜用户原话：<原样引用>｜请求动作：<一句话>｜回执给：<本窗口会话名>\n\
     然后给用户一行回执：这条归值守、已转投、球在值守那。\n\
  2）用户就是要在本窗口做 → 先让值守下班：agent-on oncall release --force\n\
  3）本窗口接班当值守 → agent-on oncall claim --session <本窗口会话名> --force\n\
绕闸（改权限 / 换等价命令偷跑）不在选项里。\n",
        action.label(),
        record.session,
        record.worktree,
        record.started_at,
        what,
        record.session
    )
}

/// Cross-window routing gate. Runs before the git boundary guard.
/// Returns 0 (allow) or 2 (block, reason on stderr).
pub(crate) fn route_decision(data: &Value) -> i32 {
    let name = tool_name(data);
    let cwd = tool_cwd(data);

    // SendMessage-style tools: only the recipient matters.
    if name.contains("SendMessage") || name.contains("send_message") {
        let to = message_recipient(data);
        if to.is_empty() {
            return 0;
        }
        let Role::Feature(record) = role_at(&cwd) else {
            return 0;
        };
        if addresses_match(&to, &record.session) {
            return 0; // the one allowed outbound channel: 交单 / 回执给值守
        }
        eprintln!(
            "{}",
            block_text(Action::CrossWindow, &record, &format!("SendMessage → {to}"))
        );
        return 2;
    }

    // Everything else is judged as a shell command.
    let input = data.get("tool_input").unwrap_or(&Value::Null);
    let cmd = input
        .get("command")
        .or_else(|| input.get("cmd"))
        .or_else(|| data.get("command"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if cmd.is_empty() {
        return 0;
    }
    // Cheap pattern match first: only a hit pays for reading the registry.
    let Some(action) = classify_bash(cmd) else {
        return 0;
    };
    let Role::Feature(record) = role_at(&cwd) else {
        return 0;
    };
    eprintln!("{}", block_text(action, &record, cmd));
    2
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::process::Command;
    use tempfile::TempDir;

    fn run(cwd: &Path, args: &[&str]) {
        let out = Command::new(args[0])
            .current_dir(cwd)
            .args(&args[1..])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// repo (main worktree) + one extra worktree, mirroring one on-call window
    /// and one feature window.
    fn fixture() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        fs::create_dir_all(&root).unwrap();
        run(&root, &["git", "init", "-b", "main"]);
        run(&root, &["git", "config", "user.email", "t@example.com"]);
        run(&root, &["git", "config", "user.name", "T"]);
        fs::write(root.join("README.md"), "x\n").unwrap();
        run(&root, &["git", "add", "."]);
        run(&root, &["git", "commit", "-m", "init"]);
        let wt = tmp.path().join("feature");
        run(
            &root,
            &[
                "git",
                "worktree",
                "add",
                "-b",
                "feature",
                wt.to_str().unwrap(),
                "main",
            ],
        );
        (tmp, root, wt)
    }

    #[test]
    fn merge_commands_are_oncall_only() {
        assert_eq!(classify_bash("gh pr merge 17 --merge"), Some(Action::Merge));
        assert_eq!(
            classify_bash("gh api -X PUT repos/o/r/pulls/17/update-branch"),
            Some(Action::Merge)
        );
        assert_eq!(
            classify_bash("git push origin v0.18.0"),
            Some(Action::Merge)
        );
        assert_eq!(classify_bash("git push --tags"), Some(Action::Merge));
        assert_eq!(classify_bash("git push origin main"), Some(Action::Merge));
        assert_eq!(classify_bash("gh pr close 3"), Some(Action::Merge));
    }

    #[test]
    fn outbound_commands_are_oncall_only() {
        assert_eq!(
            classify_bash("gh pr comment 17 --body hi"),
            Some(Action::Outbound)
        );
        assert_eq!(
            classify_bash("curl -X POST https://hooks.slack.com/services/xxx -d @-"),
            Some(Action::Outbound)
        );
        assert_eq!(
            classify_bash("curl -H 'Content-Type: application/json' https://webhook.office.com/webhookb2/abc -d '{}'"),
            Some(Action::Outbound)
        );
        assert_eq!(
            classify_bash("gh issue create --title x"),
            Some(Action::Outbound)
        );
    }

    #[test]
    fn feature_window_work_is_never_classified() {
        // Opening a PR is the feature window's delivery act, not the on-call's.
        assert_eq!(classify_bash("gh pr create --fill"), None);
        assert_eq!(classify_bash("gh pr list --state open"), None);
        assert_eq!(classify_bash("gh pr view 17 --json mergeable"), None);
        assert_eq!(classify_bash("gh pr checks 17"), None);
        assert_eq!(classify_bash("git push -u origin claude/my-lane"), None);
        assert_eq!(classify_bash("git commit -m 'x'"), None);
        assert_eq!(classify_bash("cargo test"), None);
        // GraphQL queries use POST but read nothing public-state-ish.
        assert_eq!(
            classify_bash("gh api graphql -f query='{viewer{login}}'"),
            None
        );
        // Reading via gh api stays open.
        assert_eq!(classify_bash("gh api repos/o/r/pulls/17"), None);
    }

    #[test]
    fn chained_commands_are_scanned_past_the_first_segment() {
        assert_eq!(
            classify_bash("git fetch origin -q && gh pr merge 17 --merge"),
            Some(Action::Merge)
        );
    }

    #[test]
    fn claim_registers_and_second_window_is_rejected_without_force() {
        let (_tmp, root, wt) = fixture();
        let (code, out) = claim(&root, "oncall-window-a", None, "", false);
        assert_eq!(code, 0, "{out}");
        let (code, out) = claim(&wt, "feature-window-b", None, "", false);
        assert_eq!(code, 1, "{out}");
        assert!(out.contains("已有值守在班"), "{out}");
        let (code, out) = claim(&wt, "feature-window-b", None, "", true);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("ONCALL CLAIMED"), "{out}");
    }

    #[test]
    fn whoami_separates_oncall_from_feature_window() {
        let (_tmp, root, wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        let (_, out) = whoami(&root, false);
        assert!(out.starts_with("ONCALL:"), "{out}");
        let (_, out) = whoami(&wt, false);
        assert!(out.starts_with("FEATURE:"), "{out}");
        release(&root, false);
        let (_, out) = whoami(&wt, false);
        assert!(out.starts_with("NONE:"), "{out}");
    }

    #[test]
    fn gate_blocks_merge_from_feature_window_only() {
        let (_tmp, root, wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        let payload = |cwd: &Path| {
            json!({
                "tool_name": "Bash",
                "cwd": cwd.display().to_string(),
                "tool_input": {"command": "gh pr merge 17 --merge"}
            })
        };
        assert_eq!(route_decision(&payload(&wt)), 2);
        assert_eq!(route_decision(&payload(&root)), 0);
    }

    #[test]
    fn gate_fails_open_when_nobody_is_on_call() {
        let (_tmp, _root, wt) = fixture();
        let payload = json!({
            "tool_name": "Bash",
            "cwd": wt.display().to_string(),
            "tool_input": {"command": "gh pr merge 17 --merge"}
        });
        assert_eq!(route_decision(&payload), 0);
    }

    #[test]
    fn gate_fails_open_when_registered_worktree_is_gone() {
        let (_tmp, root, wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        // simulate the on-call worktree disappearing without release
        let mut record = load(&wt).unwrap().unwrap();
        record.worktree = wt.join("gone-forever").display().to_string();
        save(&wt, &record).unwrap();
        let payload = json!({
            "tool_name": "Bash",
            "cwd": wt.display().to_string(),
            "tool_input": {"command": "gh pr merge 17 --merge"}
        });
        assert_eq!(route_decision(&payload), 0);
        let (_, out) = status(&wt, false);
        assert!(out.contains("登记已失效"), "{out}");
    }

    #[test]
    fn sendmessage_to_oncall_passes_and_sideways_is_blocked() {
        let (_tmp, root, wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        let msg = |to: &str| {
            json!({
                "tool_name": "SendMessage",
                "cwd": wt.display().to_string(),
                "tool_input": {"to": to, "message": "x"}
            })
        };
        // 交单通道：功能窗口 → 值守，放行（含带后缀的真实会话名）
        assert_eq!(route_decision(&msg("oncall-window-a")), 0);
        assert_eq!(route_decision(&msg("oncall-window-a-02")), 0);
        // 横向：功能窗口 → 另一个功能窗口，拦
        assert_eq!(route_decision(&msg("some-other-window-7f")), 2);
    }

    #[test]
    fn oncall_window_may_message_anyone() {
        let (_tmp, root, _wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        let msg = json!({
            "tool_name": "SendMessage",
            "cwd": root.display().to_string(),
            "tool_input": {"to": "some-other-window-7f", "message": "x"}
        });
        assert_eq!(route_decision(&msg), 0);
    }

    #[test]
    fn route_names_the_lane_that_owns_the_path() {
        let (_tmp, root, wt) = fixture();
        let (code, out) = worktree::claim_lane(
            &wt,
            &worktree::ClaimOpts {
                id: "lane-a".to_string(),
                goal: "g".to_string(),
                base: Some("main".to_string()),
                owns: vec!["app".to_string()],
                depends_on: Vec::new(),
                parked: false,
            },
        );
        assert_eq!(code, 0, "{out}");

        let (code, out) = route(&root, "app/page.rs", false);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("lane-a"), "{out}");
        assert!(out.contains(wt.display().to_string().as_str()), "{out}");

        // Unowned path: the on-call window must escalate, not improvise.
        let (code, out) = route(&root, "docs/orphan.md", false);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("无主"), "{out}");

        // Once the lane lands, the same path still matches — but dispatching
        // to it would send work into a window that is very likely closed.
        // active → ready → landed is the only legal path into the terminal state
        let (code, out) = worktree::set_lane_status(&wt, Some("lane-a"), "ready");
        assert_eq!(code, 0, "{out}");
        let (code, out) = worktree::set_lane_status(&wt, Some("lane-a"), "landed");
        assert_eq!(code, 0, "{out}");
        let (_, out) = route(&root, "app/page.rs", false);
        assert!(out.contains("活跃轨：无"), "{out}");
        assert!(out.contains("别直接派"), "{out}");
    }

    #[test]
    fn release_from_other_window_needs_force() {
        let (_tmp, root, wt) = fixture();
        claim(&root, "oncall-window-a", None, "", false);
        let (code, out) = release(&wt, false);
        assert_eq!(code, 1, "{out}");
        let (code, out) = release(&wt, true);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("ONCALL RELEASED"), "{out}");
    }
}
