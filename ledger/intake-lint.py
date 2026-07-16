#!/usr/bin/env python3
"""intake-lint.py — 承接层 Promotion Card 六项完整性校验器。

职责:读 intake/*.md,逐张 Promotion Card 校验 ——
  1. 必填字段齐全(对齐 kit/promotion-card-template.md):
     source / evidence / confidence / claim / suggested_landing / rollback / trace / 状态;
  2. evidence 硬门:空 evidence = 这张卡不存在(「我记得」不是证据);
  3. confidence 取值合法(high|medium|low);
  4. 状态取值合法(pending | landed@<commit> | rejected(...) | deferred)。
识别「卡」的启发式:### 标题块内含 `- source:` 行才算卡,否则跳过
(自动放过「已消化免出仓」等叙述性 ### 段)。
纯标准库零依赖。报错中文。退出码:全绿 0,有错 1,用法错 2。

用法:  python3 intake-lint.py intake/*.md   (或不带参数 = 自动扫 ../intake/)

源流:2026-07-09 首结账消化落地(intake self-review:intake-lint-timing,用户拍板「现在就做」);
audit-lint.py 思路扩展 —— 从校验编排事件流扩到校验承接层卡片。
"""
import glob
import re
import sys
from pathlib import Path

REQUIRED = [
    "source", "evidence", "confidence", "claim",
    "suggested_landing", "rollback", "trace", "状态",
]
CONFIDENCE_OK = {"high", "medium", "low"}
# 状态合法: 文中出现 pending / landed@ / rejected / deferred / 半落@（允许「最小件 landed@同批」类前缀）
STATUS_OK_RE = re.compile(r"(pending|landed@|rejected|deferred|半落@)")


def split_cards(text):
    """按 ### 切块;只保留含 `- source:` 的块(= 真卡)。返回 (标题, 起始行号, 块文本) 列表。"""
    lines = text.splitlines()
    cards, cur, title, start = [], [], None, 0
    for i, line in enumerate(lines, start=1):
        if line.startswith("### "):
            if title is not None and any("- source" in l for l in cur):
                cards.append((title, start, "\n".join(cur)))
            title, start, cur = line[4:].strip(), i, []
        elif title is not None:
            cur.append(line)
    if title is not None and any("- source" in l for l in cur):
        cards.append((title, start, "\n".join(cur)))
    return cards


def field(block, name):
    """取 `- <name>:<值>` 的值(去掉行内 HTML 注释),缺字段返回 None。"""
    m = re.search(rf"^-\s*{re.escape(name)}\s*[:：](.*)$", block, re.M)
    if not m:
        return None
    return re.sub(r"<!--.*?-->", "", m.group(1)).strip()


def lint_card(title, block):
    errs = []
    for f in REQUIRED:
        v = field(block, f)
        if v is None:
            errs.append(f"缺必填字段:{f}")
        elif v == "":
            if f == "evidence":
                errs.append("evidence 空 = 这张卡不存在(「我记得」不是证据)")
            else:
                errs.append(f"字段 {f} 值为空")
    conf = field(block, "confidence")
    if conf is not None and conf != "":
        head = re.split(r"[（(]", conf)[0].strip()
        if head not in CONFIDENCE_OK:
            errs.append(f"confidence 取值非法:'{head}'(应为 high|medium|low)")
    status = field(block, "状态")
    if status is not None and status != "" and not STATUS_OK_RE.search(status):
        errs.append(
            f"状态取值非法:'{status[:30]}'(应含 pending | landed@… | rejected… | deferred | 半落@…)"
        )
    return errs


def main():
    args = sys.argv[1:]
    if not args:
        here = Path(__file__).resolve().parent
        args = sorted(glob.glob(str(here.parent / "intake" / "*.md")))
        args = [a for a in args if not a.endswith("README.md")]
    if not args:
        print("没有找到 intake 卡文件。用法:python3 intake-lint.py intake/*.md", file=sys.stderr)
        sys.exit(2)

    total_cards, total_errs = 0, 0
    for path in args:
        p = Path(path)
        if not p.exists():
            print(f"文件不存在:{p}", file=sys.stderr)
            sys.exit(2)
        cards = split_cards(p.read_text())
        for title, start, block in cards:
            total_cards += 1
            errs = lint_card(title, block)
            if errs:
                total_errs += len(errs)
                print(f"✗ {p.name}:{start}  {title}")
                for e in errs:
                    print(f"    - {e}")

    if total_errs == 0:
        print(f"承接层校验通过:{total_cards} 张卡,六项齐全。")
        sys.exit(0)
    print(f"承接层校验不通过:{total_cards} 张卡,发现 {total_errs} 处问题(见上)。")
    sys.exit(1)


if __name__ == "__main__":
    main()
