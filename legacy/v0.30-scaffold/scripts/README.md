# scripts/ — 装机与校验入口（Rust CLI）

> 职责边界：可执行物在 **`cli/`**（Rust）。本目录不再放 Python。装机后：`cargo install --path cli` 得到 `agent-on` 二进制。

| 命令 | 作用 |
|---|---|
| `agent-on setup` | 一键：默认目录 clone/checkout pin + 写 `~/.config/agent-on/config.json` + 可选 plugin/symlink + doctor + intake-lint |
| `agent-on tag-release --level patch\|minor\|major --title "…" [--push]` | **本仓对话 commit 发版硬门**：annotated tag 钉 HEAD |
| `agent-on check routing [--with-agent-memory]` | 开箱 skill 路由 / 降档协议 / jsonl 旁路断言 |
| `agent-on doctor` | read_root / work_root 登记报告 |
| `agent-on intake-lint [files…]` | Promotion Card 六项 + evidence 硬门 |
| `agent-on audit-lint <run>.jsonl` | L 档 jsonl 状态机（旁路机件） |
| `agent-on guard` | PreToolUse 跨仓 git 闸（stdin JSON） |

### 构建 / 安装

```bash
# 依赖：rustup (https://rustup.rs) + git
cd /path/to/agent-on
cargo install --path cli --force
# 或仅构建插件内二进制（hooks 会优先用这个）：
cargo build --release --manifest-path cli/Cargo.toml
```

### 消化后发版（必做）

```bash
# 先封 CHANGELOG、更新推荐 pin，并 commit
agent-on tag-release --level minor --title "一句话" --push
# 仅本地 tag：
agent-on tag-release --level patch --title "措辞修正"
```

## setup 默认目录

| OS | `work_root` |
|---|---|
| macOS / Linux | `~/.local/share/agent-on` |
| Windows | `%LOCALAPPDATA%\agent-on` |

```bash
agent-on setup
agent-on setup --with-plugins --with-symlinks
agent-on setup --pin v0.6.3
agent-on setup --work-root /custom/path
```

决策全文：[snapshot/2026-07-16-v10-and-setup.md](../snapshot/2026-07-16-v10-and-setup.md)。源码：`cli/`。
