#!/usr/bin/env python3
"""本地受影响验证与每日 CI 共用范围选择器（v2）。"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]

GROUPS = {
    "wt.rs": ["wt_radar"],
    "merge_check.rs": ["merge_rehearsal"],
    "install.rs": ["install_and_doctor"],
    "doctor.rs": ["install_and_doctor"],
    "build_info.rs": ["install_and_doctor"],
    "source_identity.rs": ["install_and_doctor"],
}

SHARED_EXACT = {
    "cli/Cargo.lock",
    "cli/Cargo.toml",
    "cli/build.rs",
    "cli/src/main.rs",
    "cli/src/git_util.rs",
}

DOCS_ONLY_EXACT = {
    ".github/pull_request_template.md",
    ".github/scripts/check_docs.py",
    ".github/scripts/doc-boundary-baseline.txt",
}

SCOPE_RUNNER = {
    ".github/scripts/quality_scope.py",
    ".github/scripts/test_quality_scope.py",
}


def _docs_only(path: str) -> bool:
    if path in DOCS_ONLY_EXACT:
        return True
    if path.startswith(("skills/", "rules/", "docs/", "snapshot/", "legacy/", "bench/")):
        return True
    if path.endswith(".md") and not path.startswith("cli/"):
        return True
    return path in {
        "plugin.json",
        ".claude-plugin/plugin.json",
        ".claude-plugin/marketplace.json",
        ".codex-plugin/plugin.json",
    }


def _non_rust_asset(path: str) -> bool:
    return path.startswith("cli/tests/") and not path.endswith(".rs")


def _scope_runner(path: str) -> bool:
    return path in SCOPE_RUNNER or path.startswith(".github/workflows/")


def _forces_full(path: str) -> bool:
    if path in SHARED_EXACT or _scope_runner(path):
        return True
    if path.startswith(".github/"):
        return True
    return False


def plan(paths: list[str], full: bool = False) -> dict:
    nightly = full
    rust = nightly
    tests: set[str] = set()
    changed_tests: set[str] = set()
    escalated = nightly

    for path in paths:
        if _docs_only(path) or _non_rust_asset(path):
            continue
        if _forces_full(path):
            escalated = rust = True
        elif path.startswith("cli/"):
            rust = True
            if path.startswith("cli/tests/") and path.endswith(".rs"):
                stem = Path(path).stem
                tests.add(stem)
                changed_tests.add(stem)
            elif path.startswith("cli/src/") and Path(path).name in GROUPS:
                tests.update(GROUPS[Path(path).name])
            else:
                escalated = True

    clippy_tests = set(changed_tests) if escalated else set(tests)
    if escalated:
        tests.clear()
    lint_all_targets = nightly or any(_scope_runner(p) for p in paths)
    return {
        "full": escalated,
        "rust": rust,
        "lint_all_targets": lint_all_targets,
        "tests": sorted(tests),
        "clippy_tests": sorted(clippy_tests),
        "changed_files": len(paths),
    }


def test_command(data: dict) -> list[str] | None:
    if not data["rust"]:
        return None
    cmd = ["cargo", "test", "--manifest-path", "cli/Cargo.toml", "--locked"]
    if data["full"]:
        return cmd
    cmd += ["--bin", "agent-on"]
    for name in data["tests"]:
        if not (ROOT / "cli/tests" / f"{name}.rs").is_file():
            return ["cargo", "test", "--manifest-path", "cli/Cargo.toml", "--locked"]
        cmd += ["--test", name]
    return cmd


def clippy_command(data: dict) -> list[str] | None:
    if not data["rust"]:
        return None
    cmd = ["cargo", "clippy", "--manifest-path", "cli/Cargo.toml", "--locked"]
    if data.get("lint_all_targets"):
        cmd.append("--all-targets")
    else:
        cmd += ["--bin", "agent-on"]
        for name in data.get("clippy_tests") or []:
            if not (ROOT / "cli/tests" / f"{name}.rs").is_file():
                cmd = [
                    "cargo",
                    "clippy",
                    "--manifest-path",
                    "cli/Cargo.toml",
                    "--locked",
                    "--all-targets",
                ]
                break
            cmd += ["--test", name]
    cmd += ["--", "-D", "warnings"]
    return cmd


def changes(base: str, head: str) -> list[str]:
    if not base or not head or set(base) == {"0"}:
        raise ValueError("missing comparison commits")
    out = subprocess.run(
        ["git", "diff", "--name-only", "--no-renames", "-z", base, head, "--"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout
    return [os.fsdecode(p) for p in out.split(b"\0") if p]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="")
    parser.add_argument("--head", default="")
    parser.add_argument("--full", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--run", action="store_true")
    parser.add_argument("--clippy", action="store_true")
    args = parser.parse_args()
    if args.run or args.clippy:
        data = json.loads(args.output.read_text())
        cmd = test_command(data) if args.run else clippy_command(data)
        if cmd is None:
            print("No Rust behavior changed; CLI tests skipped.")
            return
        subprocess.run(cmd, cwd=ROOT, check=True)
        return
    try:
        paths = [] if args.full else changes(args.base, args.head)
        data = plan(paths, args.full)
    except (ValueError, OSError, subprocess.CalledProcessError) as exc:
        data = plan([], full=True)
        data["reason"] = f"Cannot establish changed scope; run all: {exc}"
    args.output.write_text(json.dumps(data, indent=2) + "\n")
    print(json.dumps(data, indent=2))
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a") as stream:
            for key in ("full", "rust"):
                stream.write(f"{key}={str(data[key]).lower()}\n")


if __name__ == "__main__":
    main()
