# scripts/ — 轻量可执行物

> 职责边界：跨平台装机与校验入口。方法论正文不在这里。

| 脚本 | 作用 |
|---|---|
| [setup.py](setup.py) | 一键：默认目录 clone/checkout pin + 写 `~/.config/agent-on/config.json` + 可选 plugin/symlink + doctor + intake-lint |
| [tag-release.py](tag-release.py) | **消化收尾发版硬门**：按 patch/minor/major 打 annotated tag；`--push` 推远程。禁止「消化完不打 tag」 |
| [check-skill-routing.py](check-skill-routing.py) | 断言开箱 skill 路由：不默认 Superpowers 执行栈、S 轻心态、jsonl 旁路、MRD B1+C1；可选 `--with-agent-memory` |
| 路径解析 | `kit/guard/agent_on_paths.py` |
| 卡片校验 | `ledger/intake-lint.py` |

### 消化后发版（必做）

```bash
# 先把 CHANGELOG [未发布] 封进版本节、推荐 pin 写进 README/AGENTS，并 commit
python3 scripts/tag-release.py --level minor --title "一句话" --push
# 仅本地打 tag、稍后手推:
python3 scripts/tag-release.py --level patch --title "措辞修正"
```

## setup 默认目录

| OS | `work_root` |
|---|---|
| macOS / Linux | `~/.local/share/agent-on` |
| Windows | `%LOCALAPPDATA%\agent-on` |

```bash
# 在已 clone 的仓内，或任意目录先拿到脚本：
python3 scripts/setup.py
python3 scripts/setup.py --with-plugins --with-symlinks
python3 scripts/setup.py --pin v0.5.0
python3 scripts/setup.py --work-root /custom/path

# Windows
py -3 scripts\setup.py --with-plugins
```

决策全文：[snapshot/2026-07-16-v10-and-setup.md](../snapshot/2026-07-16-v10-and-setup.md)。
