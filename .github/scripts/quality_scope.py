#!/usr/bin/env python3
"""本仓 CI 范围与执行入口：PR 验改动，夜间/手动验全量；不调用模型或修改仓库。

文档、文档闸脚本和 PR 模板不编译 Rust。已知叶子模块跑对应集成测试，
再加整个二进制的单元测试。共享基础、闸、未知 Rust 路径和本调度器自身扩大到全量。
承接层 lint 只在 intake 卡或 linter 改动时跑，夜间全量仍跑。
必需检查的 job 名称不变；跳过的是步骤，不是整条 workflow。
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]

# Leaf modules: integration-test stems. The binary unit tests always run with them.
# Empty means unit tests only. Unknown cli sources fail closed to the full suite.
GROUPS = {
    "project_status.rs": ["project_status", "session_continuity", "system_overview"],
    "dashboard.rs": ["project_status", "system_overview", "coordination"],
    "dashboard.html": ["project_status", "system_overview", "coordination"],
    "system_dashboard.rs": ["project_status", "system_overview"],
    "system_dashboard.html": ["project_status", "system_overview"],
    "codex_model.rs": ["model_compatibility", "stage_handoff", "coordination"],
    "activity.rs": [
        "activity", "coordination", "project_status",
        "session_continuity", "stage_handoff", "system_overview",
    ],
    "audit_lint.rs": [],
    "build_info.rs": ["build_identity"],
    "doctor.rs": ["build_identity"],
    "drift.rs": [],
    "intake_lint.rs": ["setup_install"],
    "integration.rs": ["coordination", "master"],
    "janitor.rs": ["coordination"],
    "landing.rs": ["master", "project_status", "system_overview"],
    "master.rs": ["master"],
    "routing.rs": [],
    "setup.rs": ["setup_install"],
    "source_identity.rs": ["build_identity"],
    "tag_release.rs": [],
}

# Shared behavior or a safety gate. A change here runs the full Rust suite.
SHARED_EXACT = {
    "cli/Cargo.lock",
    "cli/Cargo.toml",
    "cli/build.rs",
    "cli/src/coordination.rs",
    "cli/src/guard.rs",
    "cli/src/main.rs",
    "cli/src/oncall.rs",
    "cli/src/paths.rs",
    "cli/src/prepush.rs",
    "cli/src/session.rs",
    "cli/src/worktree.rs",
    "cli/src/worktree_hooks.rs",
    "cli/src/worktree_schedule.rs",
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


def _intake_path(path: str) -> bool:
    return path.startswith("intake/") or path.endswith("/intake_lint.rs")


def _docs_only(path: str) -> bool:
    if path in DOCS_ONLY_EXACT:
        return True
    return path.startswith(".github/ISSUE_TEMPLATE/") or path.startswith(".github/PULL_REQUEST_TEMPLATE/")


def _non_rust_asset(path: str) -> bool:
    return path.startswith("cli/tests/") and not path.endswith(".rs")


def _scope_runner(path: str) -> bool:
    return path in SCOPE_RUNNER or path.startswith(".github/workflows/")


def _forces_full(path: str) -> bool:
    if path in SHARED_EXACT or _scope_runner(path):
        return True
    if path.startswith("hooks/") or path.startswith("kit/guard/"):
        return True
    # Unknown CI surface (new workflow script, CODEOWNERS, …) stays on the full suite.
    if path.startswith(".github/"):
        return True
    return False


def plan(paths: list[str], full: bool = False) -> dict:
    nightly = full
    rust = nightly
    intake = nightly or any(_intake_path(p) for p in paths)
    audit = nightly or any(
        p.startswith("tools/merge-audit/") or _scope_runner(p) for p in paths
    )
    tests: set[str] = set()
    changed_tests: set[str] = set()
    escalated = nightly
    for path in paths:
        if _docs_only(path) or _non_rust_asset(path):
            continue
        if _forces_full(path):
            escalated = rust = True
        elif path.startswith("tools/merge-audit/"):
            rust = True
            audit = True
            tests.add("master")
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
    # Directly edited integration tests stay in the clippy set even when the
    # suite itself is widened to everything.
    clippy_tests = set(changed_tests) if escalated else set(tests)
    if escalated:
        tests.clear()
    # Nightly and edits to this runner itself still lint every cargo target.
    # Ordinary source PRs lint the binary plus the integration tests they actually run.
    lint_all_targets = nightly or any(_scope_runner(p) for p in paths)
    return {
        "full": escalated,
        "rust": rust,
        "intake": intake,
        "audit": audit,
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
        # Deleted/renamed tests no longer exist; full suite verifies their consumers.
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
                cmd = ["cargo", "clippy", "--manifest-path", "cli/Cargo.toml", "--locked", "--all-targets"]
                break
            cmd += ["--test", name]
    cmd += ["--", "-D", "warnings"]
    return cmd


def changes(base: str, head: str) -> list[str]:
    if not base or not head or set(base) == {"0"}:
        raise ValueError("missing comparison commits")
    out = subprocess.run(["git", "diff", "--name-only", "--no-renames", "-z", base, head, "--"],
                         cwd=ROOT, check=True, capture_output=True).stdout
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
            print("No Rust behavior changed; CLI tests deferred to nightly quality.")
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
            for key in ("full", "rust", "intake", "audit"):
                stream.write(f"{key}={str(data[key]).lower()}\n")


if __name__ == "__main__":
    main()
