#[path = "src/source_identity.rs"]
mod source_identity;
use std::{env, path::PathBuf, process::Command};

fn git(root: &std::path::Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    for file in source_identity::inputs(&root).expect("CLI build inputs") {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    println!("cargo:rerun-if-changed=src");
    for name in ["HEAD", "packed-refs", "refs/heads", "refs/tags"] {
        if let Some(path) = git(
            &root,
            &["rev-parse", "--path-format=absolute", "--git-path", name],
        ) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    println!(
        "cargo:rustc-env=AGENT_ON_SOURCE_ID={}",
        source_identity::fingerprint(&root).expect("source identity")
    );
    println!(
        "cargo:rustc-env=AGENT_ON_BUILD_COMMIT={}",
        git(&root, &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into())
    );
    println!(
        "cargo:rustc-env=AGENT_ON_BUILD_TAG={}",
        git(&root, &["describe", "--exact-match", "--tags", "HEAD"]).unwrap_or_default()
    );
    println!(
        "cargo:rustc-env=AGENT_ON_SOURCE_DIRTY={}",
        git(
            &root,
            &[
                "status",
                "--porcelain",
                "--untracked-files=normal",
                "--",
                "."
            ]
        )
        .map(|s| (!s.is_empty()).to_string())
        .unwrap_or_else(|| "unknown".into())
    );
}
