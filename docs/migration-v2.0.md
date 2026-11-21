# 从 Agent-On v0.x 迁移到 v2.0.0

> 职责边界：说明从 `v0.x`（重型跨仓方法论脚手架）升级到 `v2.0.0`（零侵入跨客户端 Skills + 薄 CLI）的操作步骤、兼容兜底与业务仓清理方式。

## 一、核心变化概览

`v2.0.0` 将 Agent-On 从“需要在每个业务仓放锁文件和台账的脚手架”重构为**零业务仓侵入的本机跨工具增强包**：

| 旧版（`v0.x`） | 新版（`v2.0.0`） |
|---|---|
| 单一禁自动触发的大路由 `/agent-on`（11 个子命令） | 4 个按场景可独立触发的原子技能：`prove-it`、`thin-review`、`wt-doctor`、`merge-check` |
| 长篇方法论与模板库（`BOOTSTRAP` / `playbook` / `kit`） | 一页高信噪比全局铁律 [core-guardrails.md](../rules/core-guardrails.md) + [50 个翻车案例集](../bench/cases/README.md) |
| 手工登记 `worktree claim --owns` + 受管 Git hooks | 无状态跨客户端扫描 `agent-on wt`（自动识别 `.claude` / `.codex` / `.gemini/antigravity` 全部 worktree） |
| 常驻值守 `oncall` / `master` / `landing` / `merge-audit` | 高并发冲刺期按需内存预演 `agent-on merge-check`（基于 `git merge-tree --write-tree`） |
| 业务仓放置 `agent-on.lock.md`、`agent-on/` 并跑 `settle/digest` | **业务仓零文件侵入**；旧材料完整归档在 `legacy/v0.30-scaffold/` |

## 二、本机升级三步走

在 Agent-On 仓库根目录执行：

```bash
# 1. 拉取 v2.0.0 并重编安装 CLI
git pull --tags origin main
cargo install --path cli --force

# 2. 一键挂载四项新 Skills 到 Claude Code / Codex / Antigravity，并自动清理旧软链与宿主旧 PreToolUse 钩子
agent-on install --with-rules --clean-legacy

# 3. 核对三端挂载状态与旧版残留
agent-on doctor
```

## 三、旧版钩子与业务仓外壳的平滑退场

### 1. 宿主旧钩子自动清理与老仓库 Git 钩子静默兼容

- **宿主 `PreToolUse / SessionStart` 旧钩子清理**：若 `~/.codex/hooks.json` 或 `~/.claude/settings.json` 中曾配置指向旧版 `kit/guard/agent-on-git-guard` 脚本的条目（该脚本已归档至 `legacy/`，未清理前宿主调用会报 `exit 127`），运行上面的 `agent-on install --clean-legacy` 会**自动删除旧版 `agent-on` 钩子条目并完整保留你的其他钩子与设置**。
- **老仓库 `.git/agent-on/hooks` 内置放行兜底**：如果你电脑上的其他老项目曾运行过 `v0.x` 的 `agent-on worktree hooks install`（`.git/config` 留有 `core.hooksPath=.../.git/agent-on/hooks`），`v2.0.0` 二进制内置了隐藏兼容桩（`guard`、`capture`、`worktree hooks run`、`worktree gc`），触发时**直接返回退出码 `0` 放行**，不会把老仓库卡死在 `git commit` 或 `git push`。
- 想彻底移除某个老仓库的旧版 Git hook 配置时，运行：
  ```bash
  agent-on install --clean-legacy --repo /path/to/your-project
  ```
  该命令会自动恢复安装前的 `previous_hooks_path`（若无则清除 `--local` 与 `--worktree` 下的残留 `core.hooksPath`）。

### 2. 清理业务仓库里的 `agent-on.lock.md` 与 `agent-on/`

`v2.0.0` 不再读取业务仓库根目录下的 `agent-on.lock.md` 或 `agent-on/{README,loop-notes,run-log}.md`：

- 普通仓库可直接 `git rm -r agent-on agent-on.lock.md`（历史记录仍在 Git 提交历史中可查）；
- 若项目自身设有历史文档保护检查（如迁移清单或目录索引校验），将上述文件移入项目的历史归档目录（如 `docs/evidence/`）并同步更新目录索引即可。
