//! Compare an actual compiled CLI with a copied checkout, then change source
//! without changing the package version or binary mtime.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, SystemTime},
};

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let dest = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &dest);
        } else {
            fs::copy(path, dest).unwrap();
        }
    }
}

#[test]
fn doctor_distinguishes_actual_source_identity_from_version_and_mtime() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("checkout");
    let home = tmp.path().join("home");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    copy_tree(&manifest.join("src"), &root.join("cli/src"));
    for name in ["Cargo.toml", "Cargo.lock", "build.rs"] {
        fs::copy(manifest.join(name), root.join("cli").join(name)).unwrap();
    }
    for name in ["CHARTER.md", "BOOTSTRAP.md"] {
        fs::write(root.join(name), "fixture\n").unwrap();
    }
    let shim = root.join("kit/guard/agent-on-git-guard");
    fs::create_dir_all(shim.parent().unwrap()).unwrap();
    fs::copy(
        manifest
            .parent()
            .unwrap()
            .join("kit/guard/agent-on-git-guard"),
        &shim,
    )
    .unwrap();
    let binary = root.join("cli/target/release/agent-on");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_agent-on"), &binary).unwrap();
    fs::create_dir_all(home.join(".claude")).unwrap();
    let init = Command::new("git")
        .current_dir(&root)
        .args(["init", "-b", "main"])
        .output()
        .unwrap();
    assert!(init.status.success());
    let hooks = Command::new(env!("CARGO_BIN_EXE_agent-on"))
        .current_dir(&root)
        .args(["worktree", "hooks", "install"])
        .output()
        .unwrap();
    assert!(
        hooks.status.success(),
        "{}",
        String::from_utf8_lossy(&hooks.stderr)
    );
    fs::write(home.join(".claude/settings.json"), serde_json::json!({"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":format!("bash \"{}\"", shim.display())}]}]}}).to_string()).unwrap();
    let doctor = || {
        let out = Command::new(env!("CARGO_BIN_EXE_agent-on"))
            .current_dir(&root)
            .arg("doctor")
            .env("HOME", &home)
            .env("AGENT_ON_ROOT", &root)
            .env_remove("CLAUDE_PLUGIN_ROOT")
            .env_remove("PLUGIN_ROOT")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    let before = doctor();
    assert!(before.contains("source identity = READ_ROOT"), "{before}");
    assert!(
        before.contains("Git executor source identity = READ_ROOT"),
        "{before}"
    );
    assert!(!before.contains("STALE"), "{before}");
    fs::write(
        root.join("cli/src/main.rs"),
        "// different source, unchanged package version\n",
    )
    .unwrap();
    fs::File::options()
        .write(true)
        .open(&binary)
        .unwrap()
        .set_modified(SystemTime::now() + Duration::from_secs(3600))
        .unwrap();
    let after = doctor();
    assert!(after.contains("STALE Git executor source"), "{after}");
    assert!(
        after.contains("STALE") && after.contains("源内容标识"),
        "{after}"
    );
    // An old executable does not become verified merely because mtime is new.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(&binary, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let legacy = doctor();
        assert!(
            legacy.contains("UNVERIFIED") && legacy.contains("legacy: build-info"),
            "{legacy}"
        );
    }
}
