#!/usr/bin/env python3
"""本仓 CI 范围与执行入口：PR 验改动，夜间/手动验全量；不调用模型或修改仓库。"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
# Shared infrastructure and unknown Rust paths deliberately fall back to all tests.
GROUPS = {
    "project_status.rs": ["project_status", "session_continuity", "system_overview"],
    "dashboard.rs": ["project_status", "system_overview", "coordination"],
    "dashboard.html": ["project_status", "system_overview", "coordination"],
    "system_dashboard.rs": ["project_status", "system_overview"],
    "system_dashboard.html": ["project_status", "system_overview"],
    "codex_model.rs": ["model_compatibility", "stage_handoff", "coordination"],
}


def plan(paths: list[str], full: bool = False) -> dict:
    rust = full
    intake = full or any(p.startswith("intake/") for p in paths)
    audit = full or any(p.startswith("tools/merge-audit/") for p in paths)
    tests = set()
    for path in paths:
        if path.startswith((".github/", "hooks/", "kit/guard/", "tools/merge-audit/")):
            full = rust = True
        elif path.startswith("cli/"):
            rust = True
            if path.startswith("cli/tests/") and path.endswith(".rs"):
                tests.add(Path(path).stem)
            elif path.startswith("cli/src/") and Path(path).name in GROUPS:
                tests.update(GROUPS[Path(path).name])
            else:
                full = True
    # Changes to the runner itself need its tests and the independent auditor too.
    audit |= full
    return {"full": full, "rust": rust, "intake": intake or rust,
            "audit": audit, "tests": sorted(tests), "changed_files": len(paths)}


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
    args = parser.parse_args()
    if args.run:
        data = json.loads(args.output.read_text())
        if not data["rust"]:
            print("No Rust behavior changed; CLI tests deferred to nightly quality.")
            return
        cmd = ["cargo", "test", "--manifest-path", "cli/Cargo.toml", "--locked"]
        if not data["full"]:
            cmd += ["--bin", "agent-on"]
            for name in data["tests"]:
                # Deleted/renamed tests no longer exist; full suite verifies their consumers.
                if not (ROOT / "cli/tests" / f"{name}.rs").is_file():
                    cmd = cmd[:5]
                    break
                cmd += ["--test", name]
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
