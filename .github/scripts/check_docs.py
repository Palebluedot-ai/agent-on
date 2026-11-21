#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Agent-On v2 文档与包元数据不变量闸。

职责边界：校验四项可机械判定的不变量——
1. `docs/` 现行文档开头的职责边界声明（棘轮）；
2. `skills/*/SKILL.md` 的 YAML frontmatter（`name` 与目录同名、`description` 非空）；
3. 非冻结层 Markdown（排除 `snapshot/`、`legacy/`）的仓内相对链接可解析；
4. 八处版本声明（AGENTS / README / CHANGELOG / Cargo.toml / 四份 plugin JSON）与 git tag 一致。

用法：
  python3 .github/scripts/check_docs.py [--allow-untagged] [--fix-baseline]
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASELINE = Path(__file__).resolve().parent / "doc-boundary-baseline.txt"

BOUNDARY_DIRS = ("docs/",)
BOUNDARY_MARKER = "职责边界"
BOUNDARY_HEAD_LINES = 12

FROZEN_DIRS = ("snapshot/", "legacy/")
LINK_RE = re.compile(r"(?<!\!)\[[^\]]*\]\(([^)\s]+)\)")
PATH_LIKE = re.compile(r"/|\.(md|html|py|rs|toml|json|yml|yaml|sh|txt)$", re.I)

VERSION_RE = re.compile(r"当前推荐版本：`v(\d+\.\d+\.\d+)`")
CARGO_VERSION_RE = re.compile(r'^version\s*=\s*"(\d+\.\d+\.\d+)"', re.M)


class Evidence(Exception):
    """取证失败：闸自己无法完成读取，退出码 2。"""


def tracked_markdown() -> list[Path]:
    try:
        out = subprocess.run(
            ["git", "ls-files", "*.md"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError) as exc:
        raise Evidence(f"git ls-files 失败：{exc}") from exc
    return [ROOT / line for line in out.splitlines() if line and (ROOT / line).exists()]


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        raise Evidence(f"无法读取 {path.relative_to(ROOT)}：{exc}") from exc


def load_baseline() -> set[str]:
    if not BASELINE.exists():
        return set()
    return {
        line.strip()
        for line in read(BASELINE).splitlines()
        if line.strip() and not line.startswith("#")
    }


def boundary_violators(files: list[Path]) -> set[str]:
    missing = set()
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        if not rel.startswith(BOUNDARY_DIRS):
            continue
        head = "\n".join(read(path).splitlines()[:BOUNDARY_HEAD_LINES])
        if BOUNDARY_MARKER not in head:
            missing.add(rel)
    return missing


def check_boundary(files: list[Path]) -> list[str]:
    baseline = load_baseline()
    missing = boundary_violators(files)
    problems = []
    for rel in sorted(missing - baseline):
        problems.append(
            f"职责边界缺失：{rel} 前 {BOUNDARY_HEAD_LINES} 行缺少「{BOUNDARY_MARKER}」声明"
        )
    for rel in sorted(baseline - missing):
        problems.append(f"棘轮基线未收紧：{rel} 已补职责边界，请从 {BASELINE.relative_to(ROOT)} 移除")
    for rel in sorted(baseline):
        if not (ROOT / rel).exists():
            problems.append(f"棘轮基线指向不存在的文件：{rel}")
    return problems


def check_skills() -> list[str]:
    skills_root = ROOT / "skills"
    if not skills_root.is_dir():
        return ["缺少 skills/ 目录"]
    problems = []
    skill_dirs = sorted(p for p in skills_root.iterdir() if p.is_dir())
    if not skill_dirs:
        return ["skills/ 下没有任何技能目录"]
    for sdir in skill_dirs:
        md = sdir / "SKILL.md"
        if not md.is_file():
            problems.append(f"技能缺少入口文件：skills/{sdir.name}/SKILL.md")
            continue
        text = read(md)
        if not text.startswith("---\n"):
            problems.append(f"skills/{sdir.name}/SKILL.md 缺少开头 YAML frontmatter (`---`)")
            continue
        end = text.find("\n---\n", 4)
        if end == -1:
            problems.append(f"skills/{sdir.name}/SKILL.md 的 YAML frontmatter 未闭合")
            continue
        fm = text[4:end]
        name_match = re.search(r"^name:\s*(\S+)\s*$", fm, re.M)
        desc_match = re.search(r"^description:\s*(.+)", fm, re.M)
        if not name_match:
            problems.append(f"skills/{sdir.name}/SKILL.md frontmatter 缺少 `name`")
        elif name_match.group(1) != sdir.name:
            problems.append(
                f"skills/{sdir.name}/SKILL.md 的 name=`{name_match.group(1)}` 与目录名 `{sdir.name}` 不一致"
            )
        if not desc_match:
            problems.append(f"skills/{sdir.name}/SKILL.md frontmatter 缺少 `description`")
    return problems


def check_links(files: list[Path]) -> list[str]:
    problems = []
    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        if rel.startswith(FROZEN_DIRS):
            continue
        in_fence = False
        for lineno, line in enumerate(read(path).splitlines(), 1):
            stripped = line.lstrip()
            if stripped.startswith("```"):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            for target in LINK_RE.findall(line):
                if re.match(r"^[a-z][a-z0-9+.-]*:", target) or target.startswith(("#", "~")):
                    continue
                if not PATH_LIKE.search(target):
                    continue
                bare = target.split("#", 1)[0]
                if not bare:
                    continue
                resolved = (path.parent / bare).resolve()
                if not resolved.exists():
                    problems.append(
                        f"死链：{rel}:{lineno} 指向 {target}（解析路径 {os.path.relpath(resolved, ROOT)} 不存在）"
                    )
    return problems


def check_versions(allow_untagged: bool) -> list[str]:
    problems = []
    versions: dict[str, str] = {}

    for doc_name in ("AGENTS.md", "README.md"):
        found = VERSION_RE.findall(read(ROOT / doc_name))
        if len(found) != 1:
            problems.append(
                f"{doc_name} 应包含且仅包含 1 处 `当前推荐版本：`vX.Y.Z``，实际匹配 {len(found)} 处"
            )
        else:
            versions[doc_name] = found[0]

    cargo_m = CARGO_VERSION_RE.search(read(ROOT / "cli/Cargo.toml"))
    if not cargo_m:
        problems.append("cli/Cargo.toml 未找到 package version")
    else:
        versions["cli/Cargo.toml"] = cargo_m.group(1)

    for manifest_rel in (
        "plugin.json",
        ".claude-plugin/plugin.json",
        ".codex-plugin/plugin.json",
    ):
        data = json.loads(read(ROOT / manifest_rel))
        ver = data.get("version")
        if not isinstance(ver, str):
            problems.append(f"{manifest_rel} 缺少字符串 version 字段")
        else:
            versions[manifest_rel] = ver

    mp = json.loads(read(ROOT / ".claude-plugin/marketplace.json"))
    mp_plugins = mp.get("plugins") or []
    if not mp_plugins or not isinstance(mp_plugins[0].get("version"), str):
        problems.append(".claude-plugin/marketplace.json plugins[0] 缺少 version 字段")
    else:
        versions[".claude-plugin/marketplace.json"] = mp_plugins[0]["version"]

    distinct = sorted(set(versions.values()))
    if len(distinct) > 1:
        detail = ", ".join(f"{k}={v}" for k, v in sorted(versions.items()))
        problems.append(f"跨文件版本号不一致：{detail}")
        return problems

    if distinct:
        ver = distinct[0]
        tag = f"v{ver}"
        changelog = read(ROOT / "CHANGELOG.md")
        if not re.search(rf"^## {re.escape(tag)}[（(]", changelog, re.M):
            problems.append(f"CHANGELOG.md 缺少 `## {tag}（…）` 章节")

        if not allow_untagged:
            try:
                tags = subprocess.run(
                    ["git", "tag", "--list", tag],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    check=True,
                ).stdout.split()
            except (OSError, subprocess.CalledProcessError) as exc:
                raise Evidence(f"git tag 执行失败：{exc}") from exc
            if tag not in tags:
                problems.append(f"Git 仓库尚未创建对应的 `{tag}` tag（发版前自检可加 `--allow-untagged`）")

    return problems


def fix_baseline(files: list[Path]) -> int:
    missing = sorted(boundary_violators(files))
    header = "# 职责边界欠账基线（棘轮，只许变短）\n"
    body = ("\n".join(missing) + "\n") if missing else ""
    BASELINE.write_text(header + body, encoding="utf-8")
    print(f"基线已更新：{len(missing)} 份欠账 -> {BASELINE.relative_to(ROOT)}")
    return 0


def main() -> int:
    if os.environ.get("AGENT_ON_DOC_GATE_OFF", "").lower() in ("1", "true", "yes"):
        print("DOC-GATE: OFF（AGENT_ON_DOC_GATE_OFF 已设）")
        return 0
    allow_untagged = "--allow-untagged" in sys.argv
    try:
        files = tracked_markdown()
        if "--fix-baseline" in sys.argv:
            return fix_baseline(files)
        problems = (
            check_boundary(files)
            + check_skills()
            + check_links(files)
            + check_versions(allow_untagged)
        )
    except Evidence as exc:
        print(f"DOC-GATE: 取证失败 —— {exc}")
        return 2

    if problems:
        print(f"DOC-GATE: FAIL（{len(problems)} 条）")
        for item in problems:
            print(f"  - {item}")
        return 1

    tag_note = "（allow-untagged）" if allow_untagged else ""
    print(
        f"DOC-GATE: PASS{tag_note}（{len(files)} 份 markdown · 4 项 Skills · 版本与链接一致）"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
