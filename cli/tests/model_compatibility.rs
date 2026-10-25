//! Model discovery is diagnostic-only; only the user's explicit choice may override a host.
#![cfg(unix)]
use serde_json::{json, Value};
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    tmp: TempDir,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" != "app-server" ]; then
  printf '%s\n' "$@" > "$TEST_ARGV"
  exit "${TEST_EXIT:-0}"
fi
if [ -n "$TEST_METADATA_CWD" ]; then pwd > "$TEST_METADATA_CWD"; fi
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"method":"initialize"'*) result='{}' ;;
    *'"method":"account/read"'*) result="$TEST_ACCOUNT" ;;
    *'"method":"config/read"'*) result="$TEST_CONFIG" ;;
    *'"method":"configRequirements/read"'*) result="$TEST_REQUIREMENTS" ;;
    *'"method":"model/list"'*)
      case "$line" in *'"cursor":"page-2"'*) result="$TEST_CATALOG_SECOND" ;; *) result="$TEST_CATALOG" ;; esac ;;
    *) continue ;;
  esac
  printf '{"id":%s,"result":%s}\n' "$id" "$result"
done
"#;
        let exe = tmp.path().join("codex");
        fs::write(&exe, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self { tmp }
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.args(["model-check", "--cwd"])
            .arg(self.tmp.path())
            .env("PATH", format!("{}:/usr/bin:/bin", self.tmp.path().display()))
            .env("TEST_ACCOUNT", json!({"account":{"type":"chatgpt"},"requiresOpenaiAuth":true}).to_string())
            .env("TEST_CONFIG", json!({"config":{"model":"desktop-only-model","model_provider":null,"model_reasoning_effort":"xhigh"}}).to_string())
            .env("TEST_REQUIREMENTS", json!({"requirements":null}).to_string())
            .env("TEST_CATALOG", catalog().to_string());
        cmd
    }
}

fn catalog() -> Value {
    json!({"data":[{"model":"native-recommended-model","isDefault":true,"hidden":false,
        "defaultReasoningEffort":"low","supportedReasoningEfforts":[{"reasoningEffort":"low"},{"reasoningEffort":"xhigh"}]}],"nextCursor":null})
}
fn parsed(out: Output) -> Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn incompatible_default_is_diagnosed_without_selecting_a_replacement() {
    let f = Fixture::new();
    let config = f.tmp.path().join("config.toml");
    fs::write(&config, "model = \"desktop-only-model\"\n").unwrap();
    let result = f.command().output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no model substituted"));
    assert!(!String::from_utf8_lossy(&result.stdout).contains("native-recommended-model"));
    assert_eq!(
        fs::read_to_string(config).unwrap(),
        "model = \"desktop-only-model\"\n"
    );
}

#[test]
fn explicit_choice_never_requires_catalog_or_silent_substitution() {
    let f = Fixture::new();
    fs::remove_file(f.tmp.path().join("codex")).unwrap();
    let result = parsed(
        f.command()
            .args(["--model", "my-exact-model"])
            .output()
            .unwrap(),
    );
    assert_eq!(result["selected_model"], "my-exact-model");
    assert_eq!(result["source"], "explicit");
    assert_eq!(result["verification"], "not-probed");
}

#[test]
fn compatible_default_is_preserved() {
    let f = Fixture::new();
    let result = parsed(f.command().env("TEST_CONFIG", json!({"config":{"model":"native-recommended-model","model_reasoning_effort":"xhigh"}}).to_string()).output().unwrap());
    assert_eq!(result["source"], "configured");
    assert_eq!(result["selected_model"], "native-recommended-model");
    assert_eq!(result["verification"], "catalog-only");
    assert_eq!(result["reasoning_effort_override"], Value::Null);
}

#[test]
fn standalone_check_defaults_to_actual_cwd_not_agent_on_source_tree() {
    let f = Fixture::new();
    let observed = f.tmp.path().join("metadata-cwd.txt");
    parsed(
        f.host_command()
            .env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"))
            .env("TEST_METADATA_CWD", &observed)
            .env("TEST_CONFIG", json!({"config":{"model":null}}).to_string())
            .arg("model-check")
            .output()
            .unwrap(),
    );
    assert_eq!(
        fs::canonicalize(fs::read_to_string(observed).unwrap().trim()).unwrap(),
        fs::canonicalize(f.tmp.path()).unwrap()
    );
}

#[test]
fn api_key_and_custom_provider_are_not_switched_to_chatgpt_models() {
    let f = Fixture::new();
    let result = parsed(
        f.command()
            .env(
                "TEST_ACCOUNT",
                json!({"account":{"type":"apiKey"},"requiresOpenaiAuth":true}).to_string(),
            )
            .output()
            .unwrap(),
    );
    assert_eq!(result["selected_model"], Value::Null);
    assert_eq!(result["source"], "host-config");
    let result = parsed(
        f.command()
            .env(
                "TEST_CONFIG",
                json!({"config":{"model":"private-model","model_provider":"private"}}).to_string(),
            )
            .output()
            .unwrap(),
    );
    assert_eq!(result["selected_model"], Value::Null);
    assert_eq!(result["source"], "host-config");
}

#[test]
fn missing_or_ambiguous_default_and_managed_mismatch_do_not_guess() {
    let f = Fixture::new();
    for models in [
        json!([]),
        json!([{"model":"a","isDefault":true},{"model":"b","isDefault":true}]),
    ] {
        let out = f
            .command()
            .env("TEST_CONFIG", json!({"config":{"model":null}}).to_string())
            .env(
                "TEST_CATALOG",
                json!({"data":models,"nextCursor":null}).to_string(),
            )
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("no unique"));
    }
    let out = f
        .command()
        .env(
            "TEST_REQUIREMENTS",
            json!({"requirements":{"models":{"newThread":{"model":"managed-unavailable"}}}})
                .to_string(),
        )
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("managed model"));
}

#[test]
fn incompatible_reasoning_effort_is_diagnosed_without_substitution() {
    let f = Fixture::new();
    let config =
        json!({"config":{"model":"native-recommended-model","model_reasoning_effort":"ultra"}});
    let result = f
        .command()
        .env("TEST_CONFIG", config.to_string())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no effort substituted"));
    let out = f
        .command()
        .env("TEST_CONFIG", config.to_string())
        .env(
            "TEST_REQUIREMENTS",
            json!({"requirements":{"models":{"newThread":{"modelReasoningEffort":"ultra"}}}})
                .to_string(),
        )
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("managed reasoning effort"));
}

#[test]
fn logged_out_account_does_not_mask_authentication_failure() {
    let f = Fixture::new();
    let out = f
        .command()
        .env(
            "TEST_ACCOUNT",
            json!({"account":null,"requiresOpenaiAuth":true}).to_string(),
        )
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("login"));
}

#[test]
fn catalogue_pagination_is_followed_before_default_observation() {
    let f = Fixture::new();
    let result = parsed(
        f.command()
            .env("TEST_CONFIG", json!({"config":{"model":null}}).to_string())
            .env(
                "TEST_CATALOG",
                json!({"data":[],"nextCursor":"page-2"}).to_string(),
            )
            .env("TEST_CATALOG_SECOND", catalog().to_string())
            .output()
            .unwrap(),
    );
    assert_eq!(result["selected_model"], "native-recommended-model");
    assert_eq!(result["source"], "native-default");
    assert_eq!(result["reasoning_effort_override"], Value::Null);
}

#[test]
fn metadata_timeout_is_bounded_and_rpc_secrets_are_not_printed() {
    let f = Fixture::new();
    fs::write(f.tmp.path().join("codex"), "#!/bin/sh\nexec sleep 20\n").unwrap();
    let start = std::time::Instant::now();
    let out = f.command().output().unwrap();
    assert!(!out.status.success());
    assert!(start.elapsed() < std::time::Duration::from_secs(17));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no model override guessed"));
    fs::write(f.tmp.path().join("codex"), "#!/bin/sh\nread line\nprintf '%s\\n' '{\"id\":1,\"error\":{\"code\":400,\"message\":\"private-secret-value\"}}'\n").unwrap();
    let out = f.command().output().unwrap();
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("private-secret-value"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("private-secret-value"));
}

impl Fixture {
    fn host_command(&self) -> Command {
        let mut cmd = self.command();
        // Use a fresh command to drop model-check's argv but preserve the fake metadata env.
        let env: Vec<_> = cmd
            .get_envs()
            .map(|(k, v)| (k.to_owned(), v.map(|v| v.to_owned())))
            .collect();
        cmd = Command::new(env!("CARGO_BIN_EXE_agent-on"));
        cmd.current_dir(self.tmp.path())
            .env_remove("CODEX_THREAD_ID")
            .env("AGENT_ON_HOST", "codex")
            .env("AGENT_ON_SESSION_ID", "compat-fixture")
            .env_remove("AGENT_ON_CONTROL_ID")
            .env_remove("AGENT_ON_CONTROL_REPO");
        for (key, value) in env {
            if let Some(value) = value {
                cmd.env(key, value);
            }
        }
        cmd.env("TEST_ARGV", self.tmp.path().join("argv.txt"));
        cmd
    }
    fn prepare_receipt(&self) -> String {
        for args in [
            &["init", "-b", "main"][..],
            &["config", "user.name", "Test"],
            &["config", "user.email", "test@example.com"],
            &["commit", "--allow-empty", "-qm", "base"],
        ] {
            assert!(Command::new("git")
                .current_dir(self.tmp.path())
                .args(args)
                .output()
                .unwrap()
                .status
                .success());
        }
        let value = parsed(
            self.host_command()
                .args([
                    "dispatch",
                    "--id",
                    "model-test",
                    "--goal",
                    "read-only fixture",
                    "--host",
                    "codex",
                    "--window",
                    "external",
                    "--read-only",
                ])
                .output()
                .unwrap(),
        );
        let id = value["session"]["id"].as_str().unwrap().to_owned();
        let path = self
            .tmp
            .path()
            .join(format!(".git/agent-on/control/sessions/{id}.json"));
        let mut session = value["session"].clone();
        session["state"] = "starting".into();
        fs::write(path, session.to_string()).unwrap();
        id
    }
}

#[test]
fn legacy_automatic_choices_never_override_user_or_host_settings() {
    for requested in [Value::Null, json!("user-selected-model")] {
        let f = Fixture::new();
        let id = f.prepare_receipt();
        let path = f
            .tmp
            .path()
            .join(format!(".git/agent-on/control/sessions/{id}.json"));
        let mut session: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        session["model"] = requested.clone();
        session["model_resolution"] = json!({
            "requested_model":null,"configured_model":"old-configured-model",
            "selected_model":"old-automatic-model","reasoning_effort_override":"low",
            "source":"native-default-fallback","account_type":"chatgpt",
            "verification":"catalog-only","checked_at":"2026-10-03T00:00:00Z"
        });
        fs::write(path, session.to_string()).unwrap();
        let metadata = f.tmp.path().join("metadata-was-started.txt");
        let result = parsed(
            f.host_command()
                .env("TEST_METADATA_CWD", &metadata)
                .args(["patrol", "serve", &id])
                .output()
                .unwrap(),
        );
        let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
        assert!(!args.contains("old-automatic-model"), "{args}");
        assert!(!args.contains("model_reasoning_effort="), "{args}");
        if let Some(requested) = requested.as_str() {
            assert!(args.contains(&format!("--model\n{requested}\n")), "{args}");
            assert_eq!(args.lines().filter(|arg| *arg == "--model").count(), 1);
            assert_eq!(result["session"]["model_resolution"]["source"], "explicit");
        } else {
            assert!(!args.lines().any(|arg| arg == "--model"), "{args}");
            assert_eq!(
                result["session"]["model_resolution"]["source"],
                "host-settings"
            );
        }
        assert_eq!(
            result["session"]["model_resolution"]["selected_model"],
            requested
        );
        assert_eq!(
            result["session"]["model_resolution"]["reasoning_effort_override"],
            Value::Null
        );
        assert_eq!(
            result["session"]["model_resolution"]["verification"],
            "not-probed"
        );
        assert!(
            !metadata.exists(),
            "launch must not query a model catalogue"
        );
    }
}

#[test]
fn runner_records_host_settings_without_probing_or_adding_overrides() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    let metadata = f.tmp.path().join("metadata-was-started.txt");
    let result = parsed(
        f.host_command()
            .env("TEST_METADATA_CWD", &metadata)
            .args(["patrol", "serve", &id])
            .output()
            .unwrap(),
    );
    assert_eq!(result["session"]["model"], Value::Null);
    assert_eq!(
        result["session"]["model_resolution"]["selected_model"],
        Value::Null
    );
    assert_eq!(
        result["session"]["model_resolution"]["source"],
        "host-settings"
    );
    assert_eq!(
        result["session"]["model_resolution"]["verification"],
        "not-probed"
    );
    let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
    assert!(!args.lines().any(|arg| arg == "--model"), "{args}");
    assert!(!args.contains("model_reasoning_effort="), "{args}");
    assert!(!args.contains("desktop-only-model"));
    assert!(!metadata.exists());
    assert_eq!(result["session"]["state"], "exited");
}

#[test]
fn missing_catalogue_does_not_block_native_launch_or_require_a_model() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    let out = f
        .host_command()
        .env(
            "TEST_CATALOG",
            json!({"data":[],"nextCursor":null}).to_string(),
        )
        .args(["patrol", "serve", &id])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
    assert!(!args.lines().any(|arg| arg == "--model"));
    let result = parsed(
        f.host_command()
            .args(["patrol", "status"])
            .output()
            .unwrap(),
    );
    let session = result["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == id)
        .unwrap();
    assert_eq!(session["state"], "exited");
    assert_eq!(session["pid"], Value::Null);
    assert_eq!(session["model_resolution"]["source"], "host-settings");
    assert_eq!(session["model_resolution"]["selected_model"], Value::Null);
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_ne!(tasks["tasks"][0]["state"], "blocked");
}

#[test]
fn compatible_or_unconfigured_default_is_not_converted_to_a_model_flag() {
    for configured in [Value::Null, json!("native-recommended-model")] {
        let f = Fixture::new();
        let id = f.prepare_receipt();
        let result = parsed(
            f.host_command()
                .env(
                    "TEST_CONFIG",
                    json!({"config":{"model":configured}}).to_string(),
                )
                .args(["patrol", "serve", &id])
                .output()
                .unwrap(),
        );
        assert_eq!(result["session"]["model"], Value::Null);
        let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
        assert!(!args.lines().any(|arg| arg == "--model"), "{args}");
    }
}

#[test]
fn retry_and_exact_resume_follow_host_settings_without_selecting_new_defaults() {
    for action in ["retry", "resume"] {
        let f = Fixture::new();
        let id = f.prepare_receipt();
        let first = parsed(
            f.host_command()
                .args(["patrol", "serve", &id])
                .output()
                .unwrap(),
        );
        let path = f
            .tmp
            .path()
            .join(format!(".git/agent-on/control/sessions/{id}.json"));
        let mut session = first["session"].clone();
        session["window"] = "tmux".into();
        session["model_resolution"]["selected_model"] = "obsolete-automatic-model".into();
        session["model_resolution"]["source"] = "native-default-fallback".into();
        session["model_resolution"]["reasoning_effort_override"] = "low".into();
        if action == "resume" {
            session["host_session_id"] = "exact-native-id".into();
        }
        fs::write(&path, session.to_string()).unwrap();
        let tmux = f.tmp.path().join("tmux");
        fs::write(&tmux, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(tmux, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut updated = catalog();
        updated["data"][0]["model"] = "new-native-default".into();
        let result = parsed(
            f.host_command()
                .env("TEST_CATALOG", updated.to_string())
                .args(["patrol", action, &id])
                .output()
                .unwrap(),
        );
        assert_eq!(result["session"]["model"], Value::Null);
        assert_eq!(
            result["session"]["model_resolution"]["selected_model"],
            Value::Null
        );
        assert_eq!(
            result["session"]["model_resolution"]["source"],
            "host-settings"
        );
        assert_eq!(result["session"]["id"], id);
        assert_eq!(result["ready"], false);
        parsed(
            f.host_command()
                .args(["patrol", "serve", &id])
                .output()
                .unwrap(),
        );
        let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
        assert!(!args.lines().any(|arg| arg == "--model"), "{args}");
        assert!(!args.contains("model_reasoning_effort="), "{args}");
        if action == "resume" {
            assert!(args.contains("resume\nexact-native-id\n"));
        }
    }
}

#[test]
fn retry_clears_only_its_own_launch_failure_and_does_not_clear_author_blocking_note() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    let executable = f.tmp.path().join("codex");
    let script = fs::read_to_string(&executable).unwrap();
    // A real local launch failure, not an optional metadata warning.
    fs::write(&executable, "#!/agent-on-fixture-nonexistent-interpreter\n").unwrap();
    let out = f
        .host_command()
        .args(["patrol", "serve", &id])
        .output()
        .unwrap();
    assert!(!out.status.success());
    fs::write(&executable, script).unwrap();
    let path = f
        .tmp
        .path()
        .join(format!(".git/agent-on/control/sessions/{id}.json"));
    let mut session: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    session["state"] = "starting".into();
    session["model_resolution"] = Value::Null;
    fs::write(&path, session.to_string()).unwrap();
    parsed(
        f.host_command()
            .args(["patrol", "serve", &id])
            .output()
            .unwrap(),
    );
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_eq!(tasks["tasks"][0]["state"], "dispatched");
    parsed(
        f.host_command()
            .args([
                "task",
                "progress",
                "model-test",
                "--state",
                "blocked",
                "--note",
                "waiting for user design decision",
            ])
            .output()
            .unwrap(),
    );
    session["state"] = "starting".into();
    session["model_resolution"] = Value::Null;
    fs::write(&path, session.to_string()).unwrap();
    parsed(
        f.host_command()
            .args(["patrol", "serve", &id])
            .output()
            .unwrap(),
    );
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_eq!(tasks["tasks"][0]["state"], "blocked");
    assert_eq!(
        tasks["tasks"][0]["progress"]["note"],
        "waiting for user design decision"
    );
}

#[test]
fn unavailable_metadata_still_reports_actual_host_rejection_without_a_model_flag() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    let result = parsed(
        f.host_command()
            .env(
                "TEST_CATALOG",
                json!({"data":[],"nextCursor":null}).to_string(),
            )
            .env("TEST_EXIT", "1")
            .args(["patrol", "serve", &id])
            .output()
            .unwrap(),
    );
    assert_eq!(result["host_success"], false);
    assert_eq!(result["session"]["exit_code"], 1);
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_eq!(tasks["tasks"][0]["state"], "blocked");
    assert_eq!(tasks["tasks"][0]["progress"]["source"], "host-exit");
    let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
    assert!(!args.lines().any(|arg| arg == "--model"));
}

#[test]
fn unsuccessful_model_exit_blocks_task_without_automatic_replay() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    let result = parsed(
        f.host_command()
            .env("TEST_EXIT", "1")
            .args(["patrol", "serve", &id])
            .output()
            .unwrap(),
    );
    assert_eq!(result["host_success"], false);
    assert_eq!(result["session"]["state"], "exited");
    assert_eq!(result["session"]["exit_code"], 1);
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_eq!(tasks["tasks"][0]["state"], "blocked");
    assert_eq!(tasks["tasks"][0]["progress"]["source"], "host-exit");
    let args = fs::read_to_string(f.tmp.path().join("argv.txt")).unwrap();
    assert_eq!(args.lines().filter(|line| *line == "--model").count(), 0);
}

#[test]
fn native_launch_does_not_erase_authors_existing_block() {
    let f = Fixture::new();
    let id = f.prepare_receipt();
    parsed(
        f.host_command()
            .args([
                "task",
                "progress",
                "model-test",
                "--state",
                "blocked",
                "--note",
                "author requires a design decision",
            ])
            .output()
            .unwrap(),
    );
    assert!(f
        .host_command()
        .env(
            "TEST_CATALOG",
            json!({"data":[],"nextCursor":null}).to_string()
        )
        .args(["patrol", "serve", &id])
        .output()
        .unwrap()
        .status
        .success());
    let tasks = parsed(f.host_command().args(["task", "list"]).output().unwrap());
    assert_eq!(tasks["tasks"][0]["progress"]["source"], "executor-update");
    assert_eq!(
        tasks["tasks"][0]["progress"]["note"],
        "author requires a design decision"
    );
}
