# scripts/ — 轻量可执行物

> 职责边界：跨平台装机与校验入口。方法论正文不在这里。

| 脚本 | 作用 |
|---|---|
| [setup.py](setup.py) | 一键：默认目录 clone/checkout pin + 写 `~/.config/agent-on/config.json` + 可选 plugin/symlink + doctor + intake-lint |
| 路径解析 | `kit/guard/agent_on_paths.py` |
| 卡片校验 | `ledger/intake-lint.py` |

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
