#!/usr/bin/env python3
"""merge-audit —— 值守全自动合并的独立审计员。

职责边界：本工具**不合并任何东西，也不阻止任何东西**。它只做三件事：

  1. precheck —— 合之前，按穷举的硬停清单判一次（值守拿它当闸的输入）
  2. record   —— 值守把自己的判断与动作写进账本（它**说**自己干了什么）
  3. scan     —— 事后从 GitHub 真相里独立重判一次（它**实际**干了什么）

`report` 把 2 和 3 对起来。四种被点名的形状：

  VIOLATION   硬停清单命中，却已经合了            —— 越界，最重
  MERGED_RED  评审 changes-requested 或检查红着合  —— 越界
  UNRECORDED  合了但账本里没有对应记录            —— 悄悄合的，比合错更危险
  MISMATCH    值守自称的档位 ≠ 独立重判的档位      —— 自我合理化的指纹

**独立性是本工具唯一的价值来源**：scan 从不读 record 的结论去下判断。值守可以在
record 里写任何话，scan 照样从 PR 的真实文件清单 / 作者 / 检查结论重新算一遍。

账本是哈希链（每条记录带前一条的 sha256）。悄悄删掉或改写一条历史记录会让链断掉，
`report` 会报 LEDGER-BROKEN 并指出断在第几行。

依赖：python3 标准库 + `gh` CLI + `git`。无第三方包，不需要 venv。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
DEFAULT_POLICY = TOOL_DIR / "policy.json"

# 严重度：数字越大越严重。--fail-on 拿它做阈值。
LEVELS = {
    "OK": 0,
    "NOTABLE": 10,
    "MISMATCH": 20,
    "UNRECORDED": 30,
    "MERGED_RED": 35,
    "LEDGER_BROKEN": 38,
    "VIOLATION": 40,
}

# precheck 的退出码：值守脚本按它分流
EXIT_AUTO = 0        # 自动合，不用出声
EXIT_NOTABLE = 10    # 自动合，但当轮必须播报一行
EXIT_HARD_STOP = 20  # 不许自动合，必须问用户
EXIT_ERROR = 2


# ---------------------------------------------------------------- 基础设施

def repo_root() -> Path:
    """本仓根目录。跑在 worktree 里也对。"""
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=str(TOOL_DIR), capture_output=True, text=True, check=True,
        ).stdout.strip()
        return Path(out)
    except Exception:
        return TOOL_DIR.parent.parent


def default_ledger() -> Path:
    return repo_root() / "ledger" / "merge-audit.jsonl"


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def die(msg: str, code: int = EXIT_ERROR):
    print(f"merge-audit: {msg}", file=sys.stderr)
    sys.exit(code)


def gh_json(args: list[str]) -> object:
    """跑 gh 并解析 JSON。gh 不在 / 未登录 / 网络断都在这里变成清楚的错。"""
    try:
        proc = subprocess.run(["gh", *args], capture_output=True, text=True)
    except FileNotFoundError:
        die("找不到 gh CLI；本工具靠它读 GitHub 真相（brew install gh）")
    if proc.returncode != 0:
        die(f"gh {' '.join(args[:3])}… 失败：{proc.stderr.strip()[:400]}")
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        die(f"gh 返回的不是 JSON：{proc.stdout[:200]}")


# ---------------------------------------------------------------- 规则引擎

def glob_to_regex(pat: str) -> re.Pattern:
    """把 glob 编成 regex。约定与 shell 一致，别用 fnmatch 的松语义：

      **/x  → x 出现在任意深度（含仓根）
      **    → 跨目录任意
      *     → 单层内任意（不跨 /）
      ?     → 单层内一个字符
    """
    out = []
    if pat.startswith("**/"):
        out.append(r"(?:.*/)?")
        pat = pat[3:]
    i = 0
    while i < len(pat):
        c = pat[i]
        if c == "*":
            if pat[i:i + 2] == "**":
                out.append(r".*")
                i += 2
                continue
            out.append(r"[^/]*")
        elif c == "?":
            out.append(r"[^/]")
        else:
            out.append(re.escape(c))
        i += 1
    return re.compile("^" + "".join(out) + "$")


def paths_matching(paths: list[str], globs: list[str]) -> list[str]:
    pats = [glob_to_regex(g) for g in globs]
    return [p for p in paths if any(rx.match(p) for rx in pats)]


def text_markers_hit(text: str, markers: list[str]) -> list[str]:
    """自由文本包含匹配。**慎用**：在方法论仓里正文会讨论这些词，误报率极高。
    要判「作者喊了这单不普通」请用 body_line_patterns（整行前缀）。"""
    low = (text or "").lower()
    return [m for m in markers if m.lower() in low]


def split_diff_by_file(diff: str) -> dict[str, str]:
    """把 unified diff 拆成 {文件路径: 该文件的新增行}。

    按文件归属有两个好处：证据能说清「哪个文件里」，以及能按文件排除。
    """
    out: dict[str, list[str]] = {}
    cur = None
    for ln in (diff or "").splitlines():
        m = re.match(r"^diff --git a/(.+?) b/(.+)$", ln)
        if m:
            cur = m.group(2)
            out.setdefault(cur, [])
            continue
        if cur and ln.startswith("+") and not ln.startswith("+++"):
            out[cur].append(ln)
    return {k: "\n".join(v) for k, v in out.items()}


def diff_patterns_hit(diff: str, patterns: list[str],
                      exclude_globs: list[str] | None = None) -> list[str]:
    """只扫新增行——被删掉的密钥不是本 PR 引入的。

    exclude_globs 里的文件跳过：规则定义与它们的测试夹具**必然**包含模式本身
    （本工具的 policy.json 里写着 `AKIA[0-9A-Z]{16}`，测试里写着假的 PRIVATE KEY）。
    这些文件本来就被硬停第 1 类拦着，不需要再被密钥规则误报一次。
    """
    per_file = split_diff_by_file(diff)
    if not per_file and diff:
        # 没有 diff --git 头（如 --from-file 造的简化夹具）→ 退回整体扫描
        per_file = {"<diff>": "\n".join(
            ln for ln in diff.splitlines()
            if ln.startswith("+") and not ln.startswith("+++"))}

    excluded = set(paths_matching(list(per_file), exclude_globs or []))
    hits = []
    for path, added in per_file.items():
        if path in excluded:
            continue
        for pat in patterns:
            if re.search(pat, added):
                hits.append(f"{path} 新增行命中 /{pat}/")
    return hits


def pr_paths(pr: dict) -> list[str]:
    return [f["path"] for f in (pr.get("files") or [])]


def pr_labels(pr: dict) -> list[str]:
    return [(l.get("name") or "").lower() for l in (pr.get("labels") or [])]


def evaluate_rule(rule: dict, pr: dict, policy: dict, diff: str | None) -> list[str]:
    """返回这条规则的命中证据；空列表 = 没命中。"""
    ev: list[str] = []
    paths = pr_paths(pr)
    text = f"{pr.get('title', '')}\n{pr.get('body', '')}"

    hit_paths = paths_matching(paths, rule.get("path_globs", []))
    ev += [f"文件 {p}" for p in hit_paths]

    ev += [f"描述/标题命中「{m}」" for m in text_markers_hit(text, rule.get("body_markers", []))]

    # 结构化标记：整行以 `BREAKING:` 这种前缀开头才算。正文里出现这个词不算——
    # 方法论仓的正文天天在讨论 breaking / 迁移，自由搜索必然误报（2026-08-20 首跑实测）。
    for pat in rule.get("body_line_patterns", []):
        m = re.search(pat, text, re.MULTILINE)
        if m:
            ev.append(f"结构化标记行「{m.group(0).strip()}」")

    want_labels = [x.lower() for x in rule.get("labels", [])]
    ev += [f"标签 {l}" for l in pr_labels(pr) if l in want_labels]

    if rule.get("untrusted_author"):
        author = ((pr.get("author") or {}).get("login")) or "<unknown>"
        if author not in policy.get("trusted_authors", []):
            ev.append(f"作者 {author} 不在受信名单")

    if diff is not None and rule.get("diff_patterns"):
        ev += [f"密钥模式：{h}" for h in
               diff_patterns_hit(diff, rule["diff_patterns"], rule.get("diff_exclude_globs"))]

    return ev


def merge_health(pr: dict, policy: dict) -> dict:
    """合并当时的健康度。post-hoc 也核得到——评审结论与检查结论都留在 PR 上。"""
    cfg = policy.get("merge_health", {})
    red_set = set(cfg.get("check_conclusions_treated_as_red", []))
    reasons: list[str] = []

    if cfg.get("block_on_changes_requested", True):
        if (pr.get("reviewDecision") or "") == "CHANGES_REQUESTED":
            reasons.append("评审结论是 CHANGES_REQUESTED")

    if cfg.get("block_on_failing_checks", True):
        for c in (pr.get("statusCheckRollup") or []):
            concl = (c.get("conclusion") or c.get("state") or "").upper()
            if concl in red_set:
                reasons.append(f"检查「{c.get('name') or c.get('context') or '?'}」= {concl}")

    return {"red": bool(reasons), "reasons": reasons}


def classify(pr: dict, policy: dict, diff: str | None = None) -> dict:
    """独立重判。**只看 PR 的客观事实**，不看任何人对它的自述。"""
    hard, notable = [], []

    for rule in policy.get("hard_stop_rules", []):
        ev = evaluate_rule(rule, pr, policy, diff)
        if ev:
            hard.append({"id": rule["id"], "title": rule.get("title", rule["id"]),
                         "why": rule.get("why", ""), "evidence": ev})

    for rule in policy.get("notable_rules", []):
        ev = evaluate_rule(rule, pr, policy, diff)
        if ev:
            notable.append({"id": rule["id"], "title": rule.get("title", rule["id"]),
                            "why": rule.get("why", ""), "evidence": ev})

    decision = "HARD_STOP" if hard else ("NOTABLE" if notable else "AUTO")
    return {
        "pr": pr.get("number"),
        "title": pr.get("title", ""),
        "author": ((pr.get("author") or {}).get("login")) or "<unknown>",
        "decision": decision,
        "hard_stop": hard,
        "notable": notable,
        "health": merge_health(pr, policy),
        "files": pr_paths(pr),
        "merged_at": pr.get("mergedAt"),
        "merge_commit": ((pr.get("mergeCommit") or {}).get("oid")),
        "deep": diff is not None,
    }


# ---------------------------------------------------------------- 账本（哈希链）

GENESIS = "GENESIS"


def line_hash(line: str) -> str:
    return hashlib.sha256(line.encode("utf-8")).hexdigest()[:16]


def read_ledger(path: Path) -> tuple[list[dict], list[str]]:
    if not path.exists():
        return [], []
    raw = [ln for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()]
    recs = []
    for i, ln in enumerate(raw):
        try:
            recs.append(json.loads(ln))
        except json.JSONDecodeError:
            recs.append({"kind": "corrupt", "_line": i + 1})
    return recs, raw


def verify_chain(raw: list[str]) -> int | None:
    """返回第一处断链的行号（1-based）；None = 完整。"""
    prev = GENESIS
    for i, ln in enumerate(raw):
        try:
            rec = json.loads(ln)
        except json.JSONDecodeError:
            return i + 1
        if rec.get("prev") != prev:
            return i + 1
        prev = line_hash(ln)
    return None


def append_record(path: Path, rec: dict) -> dict:
    path.parent.mkdir(parents=True, exist_ok=True)
    _, raw = read_ledger(path)
    rec = dict(rec)
    rec["prev"] = line_hash(raw[-1]) if raw else GENESIS
    line = json.dumps(rec, ensure_ascii=False, sort_keys=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(line + "\n")
    return rec


# ---------------------------------------------------------------- 取数

PR_FIELDS = ("number,title,author,body,labels,files,mergedAt,mergeCommit,"
             "reviewDecision,statusCheckRollup,isDraft,state")


def fetch_pr(repo: str, number: int) -> dict:
    return gh_json(["pr", "view", str(number), "--repo", repo, "--json", PR_FIELDS])


def fetch_merged(repo: str, limit: int) -> list[dict]:
    return gh_json(["pr", "list", "--repo", repo, "--state", "merged",
                    "--limit", str(limit), "--json", PR_FIELDS])


def fetch_diff(repo: str, number: int) -> str:
    try:
        proc = subprocess.run(["gh", "pr", "diff", str(number), "--repo", repo],
                              capture_output=True, text=True)
    except FileNotFoundError:
        return ""
    return proc.stdout if proc.returncode == 0 else ""


def load_prs(args, policy: dict) -> list[dict]:
    """离线模式（--from-file）让测试与复盘不依赖网络。"""
    if getattr(args, "from_file", None):
        data = json.loads(Path(args.from_file).read_text(encoding="utf-8"))
        return data if isinstance(data, list) else [data]
    return fetch_merged(args.repo or policy["repo"], args.limit)


# ---------------------------------------------------------------- 渲染

def render_verdict(v: dict) -> str:
    icon = {"HARD_STOP": "■ 硬停", "NOTABLE": "▲ 需播报", "AUTO": "· 自动合"}[v["decision"]]
    lines = [f"{icon}  #{v['pr']} {v['title']}", f"        作者 {v['author']}"]
    for r in v["hard_stop"] + v["notable"]:
        lines.append(f"        [{r['id']}] {r['title']}")
        for e in r["evidence"][:6]:
            lines.append(f"            └ {e}")
        if len(r["evidence"]) > 6:
            lines.append(f"            └ …另有 {len(r['evidence']) - 6} 条证据")
    if v["health"]["red"]:
        for r in v["health"]["reasons"]:
            lines.append(f"        ⚠ 健康度：{r}")
    if not v["deep"]:
        lines.append("        （未跑 --deep：diff 里的密钥模式这一轮没扫）")
    return "\n".join(lines)


# ---------------------------------------------------------------- 子命令

def cmd_policy(args) -> int:
    policy = json.loads(Path(args.policy).read_text(encoding="utf-8"))
    print(f"merge-audit 生效规则（{args.policy}）")
    print(f"仓：{policy['repo']}   受信作者：{', '.join(policy['trusted_authors'])}")
    print("\n硬停（永不自动合，必须问用户）：")
    for r in policy["hard_stop_rules"]:
        print(f"  ■ {r['title']}（{r['id']}）")
        print(f"      为什么：{r.get('why','')}")
        for g in r.get("path_globs", []):
            print(f"      路径：{g}")
        for m in r.get("body_markers", []):
            print(f"      标记：{m}")
        if r.get("untrusted_author"):
            print("      作者：不在受信名单即命中")
    print("\n需播报（照常自动合，但值守当轮必须出声一行）：")
    for r in policy["notable_rules"]:
        print(f"  ▲ {r['title']}（{r['id']}）")
        print(f"      为什么：{r.get('why','')}")
    print("\n其余一切：自动合，只记一行账。")
    return 0


def cmd_precheck(args) -> int:
    policy = json.loads(Path(args.policy).read_text(encoding="utf-8"))
    repo = args.repo or policy["repo"]
    if args.from_file:
        prs = json.loads(Path(args.from_file).read_text(encoding="utf-8"))
        prs = prs if isinstance(prs, list) else [prs]
        pr = next((p for p in prs if p.get("number") == args.pr), None)
        if pr is None:
            die(f"--from-file 里没有 #{args.pr}")
        diff = None
    else:
        pr = fetch_pr(repo, args.pr)
        diff = fetch_diff(repo, args.pr) if not args.no_deep else None

    v = classify(pr, policy, diff)

    if args.json:
        print(json.dumps(v, ensure_ascii=False, indent=2))
    else:
        print(render_verdict(v))
        if v["decision"] == "HARD_STOP":
            print("\n→ 不许自动合。带上面的证据去问用户。")
        elif v["decision"] == "NOTABLE":
            print("\n→ 可以自动合，但本轮面板必须写一行让用户看见。")
        else:
            print("\n→ 自动合，合完 record 一行即可。")
        if v["health"]["red"]:
            print("→ 健康度红：合之前先分诊，别红着合。")

    if v["decision"] == "HARD_STOP" or v["health"]["red"]:
        return EXIT_HARD_STOP
    return EXIT_NOTABLE if v["decision"] == "NOTABLE" else EXIT_AUTO


def cmd_record(args) -> int:
    ledger = Path(args.ledger) if args.ledger else default_ledger()
    rec = {
        "kind": "claim",
        "ts": now_iso(),
        "pr": args.pr,
        "action": args.action,
        "claimed": args.claimed,
        "pointer": args.pointer or "",
        "note": args.note or "",
        "session": args.session or os.environ.get("CLAUDE_SESSION", "") or "unknown",
    }
    written = append_record(ledger, rec)
    print(f"已记账 → {ledger}")
    print(f"  #{args.pr} {args.action} / 自称 {args.claimed}"
          + (f" / 指针 {args.pointer}" if args.pointer else ""))
    print(f"  链：prev={written['prev']}")
    return 0


def cmd_scan(args) -> int:
    policy = json.loads(Path(args.policy).read_text(encoding="utf-8"))
    repo = args.repo or policy["repo"]
    ledger = Path(args.ledger) if args.ledger else default_ledger()

    prs = load_prs(args, policy)
    if args.since:
        prs = [p for p in prs if (p.get("mergedAt") or "") >= args.since]

    verdicts = []
    for pr in prs:
        diff = None
        if args.deep and not args.from_file:
            diff = fetch_diff(repo, pr["number"])
        verdicts.append(classify(pr, policy, diff))

    if args.write:
        for v in verdicts:
            append_record(ledger, {
                "kind": "verdict", "ts": now_iso(), "pr": v["pr"],
                "decision": v["decision"],
                "rules": [r["id"] for r in v["hard_stop"] + v["notable"]],
                "health_red": v["health"]["red"],
                "merge_commit": v["merge_commit"], "merged_at": v["merged_at"],
                "deep": v["deep"],
            })

    if args.json:
        print(json.dumps(verdicts, ensure_ascii=False, indent=2))
        return 0

    print(f"独立重判 {len(verdicts)} 条已合并 PR（仓 {repo}）\n")
    for v in verdicts:
        print(render_verdict(v))
        print()
    if args.write:
        print(f"判决已写进账本 → {ledger}")
    return 0


def collect_findings(verdicts: list[dict], claims: dict[int, dict],
                     chain_break: int | None, starts_at: str | None = None) -> list[dict]:
    """starts_at 之前合的 PR 不产生**记账类**发现（那时还没有账可对），
    但仍然产生**内容类**发现（合了不该合的，什么时候合的都算数）。"""
    findings: list[dict] = []

    if chain_break is not None:
        findings.append({
            "level": "LEDGER_BROKEN", "pr": None,
            "summary": f"账本哈希链在第 {chain_break} 行断了",
            "detail": "有人改写或删掉了历史记录。账本是追加写的，正常情况下链不会断。",
        })

    for v in verdicts:
        n = v["pr"]
        claim = claims.get(n)

        if v["decision"] == "HARD_STOP":
            ev = "；".join(e for r in v["hard_stop"] for e in r["evidence"][:3])
            findings.append({
                "level": "VIOLATION", "pr": n,
                "summary": f"#{n} 命中硬停清单却已经合了（{', '.join(r['id'] for r in v['hard_stop'])}）",
                "detail": ev,
            })

        if v["health"]["red"]:
            findings.append({
                "level": "MERGED_RED", "pr": n,
                "summary": f"#{n} 红着合了",
                "detail": "；".join(v["health"]["reasons"]),
            })

        in_scope = not starts_at or (v.get("merged_at") or "") >= starts_at

        if claim is None and in_scope:
            findings.append({
                "level": "UNRECORDED", "pr": n,
                "summary": f"#{n} 合了，但账本里没有值守的记录",
                "detail": "没记账的合并等于没发生过——事后谁都说不清当时是按什么判的。",
            })
        elif claim and claim.get("claimed") and claim["claimed"] != v["decision"]:
            findings.append({
                "level": "MISMATCH", "pr": n,
                "summary": f"#{n} 值守自称 {claim['claimed']}，独立重判是 {v['decision']}",
                "detail": f"值守备注：{claim.get('note') or '（无）'}",
            })

        if v["decision"] == "NOTABLE":
            findings.append({
                "level": "NOTABLE", "pr": n,
                "summary": f"#{n} 属于需播报类（{', '.join(r['id'] for r in v['notable'])}）",
                "detail": "自动合是对的，但用户该看见一行。",
            })

    findings.sort(key=lambda f: -LEVELS[f["level"]])
    return findings


def cmd_report(args) -> int:
    policy = json.loads(Path(args.policy).read_text(encoding="utf-8"))
    repo = args.repo or policy["repo"]
    ledger = Path(args.ledger) if args.ledger else default_ledger()

    prs = load_prs(args, policy)
    if args.since:
        prs = [p for p in prs if (p.get("mergedAt") or "") >= args.since]
    verdicts = [classify(p, policy, None) for p in prs]

    recs, raw = read_ledger(ledger)
    chain_break = verify_chain(raw) if raw else None
    claims = {r["pr"]: r for r in recs if r.get("kind") == "claim" and r.get("action") == "merged"}

    starts_at = args.since or policy.get("ledger_starts_at")
    findings = collect_findings(verdicts, claims, chain_break, starts_at)

    if args.json:
        print(json.dumps({"findings": findings, "scanned": len(verdicts)},
                         ensure_ascii=False, indent=2))
    else:
        print(f"merge-audit 报告 —— 仓 {repo}，覆盖 {len(verdicts)} 条已合并 PR"
              + (f"（{args.since} 之后）" if args.since else ""))
        print(f"账本：{ledger}（{len(raw)} 行，链{'断了' if chain_break else '完整'}）")
        if starts_at:
            print(f"记账类发现只覆盖 {starts_at} 之后合入的单（之前没有账可对）；"
                  f"越界类发现不受此限。\n")
        else:
            print()
        if not findings:
            print("没有发现。全部已合并 PR 都落在自动合入面里，且都有记账。")
        else:
            for f in findings:
                tag = f["level"].ljust(13)
                print(f"[{tag}] {f['summary']}")
                if f["detail"]:
                    print(f"                {f['detail']}")
        counts: dict[str, int] = {}
        for f in findings:
            counts[f["level"]] = counts.get(f["level"], 0) + 1
        if counts:
            print("\n小计：" + "  ".join(f"{k}×{v}" for k, v in counts.items()))

    threshold = LEVELS[args.fail_on.upper()]
    worst = max((LEVELS[f["level"]] for f in findings), default=0)
    return 1 if worst >= threshold else 0


# ---------------------------------------------------------------- 入口

def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="merge-audit",
        description="值守自动合并的独立审计员：合之前判一次，合之后独立重判一次，对不上就点名。",
    )
    p.add_argument("--policy", default=str(DEFAULT_POLICY), help="规则文件（默认 tools/merge-audit/policy.json）")
    p.add_argument("--repo", default=None, help="覆盖 policy.json 里的仓坐标")
    p.add_argument("--ledger", default=None, help="账本路径（默认 ledger/merge-audit.jsonl）")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("policy", help="打印生效规则（人读）")
    s.set_defaults(func=cmd_policy)

    s = sub.add_parser("precheck", help="合之前判一次：0=自动合 10=自动合但播报 20=必须问用户")
    s.add_argument("--pr", type=int, required=True)
    s.add_argument("--json", action="store_true")
    s.add_argument("--no-deep", action="store_true", help="跳过 diff 密钥扫描（快，但漏一层）")
    s.add_argument("--from-file", default=None, help="离线：从 JSON 文件读 PR，不连网")
    s.set_defaults(func=cmd_precheck)

    s = sub.add_parser("record", help="值守记一行：我对这单判了什么、干了什么")
    s.add_argument("--pr", type=int, required=True)
    s.add_argument("--action", choices=["merged", "held", "returned"], required=True)
    s.add_argument("--claimed", choices=["AUTO", "NOTABLE", "HARD_STOP"], required=True)
    s.add_argument("--pointer", default=None, help="拍板指针 / PR 链接 / 快照路径")
    s.add_argument("--note", default=None)
    s.add_argument("--session", default=None)
    s.set_defaults(func=cmd_record)

    s = sub.add_parser("scan", help="事后从 GitHub 真相独立重判已合并的 PR")
    s.add_argument("--limit", type=int, default=30)
    s.add_argument("--since", default=None, help="ISO 时间，只看这之后合的")
    s.add_argument("--deep", action="store_true", help="连 diff 一起扫密钥模式（慢）")
    s.add_argument("--write", action="store_true", help="把判决写进账本")
    s.add_argument("--json", action="store_true")
    s.add_argument("--from-file", default=None)
    s.set_defaults(func=cmd_scan)

    s = sub.add_parser("report", help="把值守的自述和独立重判对起来，点名对不上的")
    s.add_argument("--limit", type=int, default=30)
    s.add_argument("--since", default=None)
    s.add_argument("--json", action="store_true")
    s.add_argument("--from-file", default=None)
    s.add_argument("--fail-on", default="MISMATCH",
                   choices=[k.lower() for k in LEVELS] + list(LEVELS),
                   help="到这个严重度就退出码 1（默认 MISMATCH：NOTABLE 不算失败）")
    s.set_defaults(func=cmd_report)

    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
