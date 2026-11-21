---
name: wt-doctor
description: >-
  Audit all Git worktrees across Claude Code (.claude/worktrees), Codex (.codex/worktrees),
  Antigravity (.gemini/antigravity/worktrees), and custom directories to catch uncommitted
  file collisions, stale-branch shipping lag behind origin/main, and merged worktrees safe
  to prune. Use before parallel edits, before building/shipping from a worktree, or when
  cleaning up accumulated worktrees.
---

# WT Doctor — 跨客户端 Worktree 防撞雷达与安全回收

Claude Code、Codex 与 Antigravity 都会各自自动创建 Git worktree，但彼此隔离：A 工具不知道 B 工具正在改同一份文件，发包时容易拿落后主线十几个提交的旧 worktree 当最新版装机，磁盘上也会积攒大量已合并的死目录。本技能做跨工具统一体检与安全回收。

## 执行步骤

### 1. 运行只读全景扫描（Read-Only Scan）

在仓库任意目录或 worktree 下优先调用 `agent-on` CLI：

```bash
# 毫秒级本地扫描：跨客户端 worktree 盘点 + 未提交/分支撞车 + 主线落差 + 可回收分类
agent-on wt

# 连同 GitHub 开放 PR 改动文件一起比对重叠（需要联网与 gh 登录）
agent-on wt --with-prs

# 机器可读结构化输出
agent-on wt --json
```

> **未安装 `agent-on` CLI 时的原生 Git 退化方案**：
> 先运行 `git fetch --prune origin`（需最新远端基线时），再用 `git worktree list --porcelain` 枚举全部检出目录；对每棵树执行 `git -C <wt> status --porcelain`（查未提交改动）与 `git -C <wt> rev-list --left-right --count origin/main...HEAD`（查主线落差与独有提交），并用 `git -C <wt> cherry origin/main HEAD` 区分已等价合入（全 `-`）与真未合入提交（含 `+`）。

**完成判据**：拿到全部检出目录的客户端归属（`claude-code` / `codex` / `antigravity` / `primary` / `custom`）、脏文件清单、相对主线的落后/领先数及回收分类。

### 2. 按三类信号决策

1. **跨树/跨 PR 撞车信号（`UNCOMMITTED_COLLISION` / `BRANCH_OR_PR_OVERLAP`）**：
   - 两棵活跃 worktree 同时存在同一文件的未提交修改时，立即停止双写，先归并到同一棵树处理或等对方提交后再继续。
   - 当前分支修改范围与已开 PR 重叠时，评估叠在同一分支推进或运行 `merge-check` 预演合并顺序。
2. **交付前主线落差核对（`BEHIND_BASE`）**：
   - 打包客户端、部署测试环境、录屏验收或提 PR 前，核对当前树落后 `origin/main` 的提交数。若落后且上游涉及相关模块或构建脚本，先同步主线再验收，杜绝把旧版当新版发出去。
3. **死 Worktree 分级安全盘点（绝不一刀切强删）**：
   - `REAPABLE`（干净无未跟踪文件、未加锁、非主目录、非当前目录，且 `HEAD` 已是主线祖先）：**唯一可自动安全清理的类别**。
   - `LIKELY_SQUASHED`（工作区干净，按提交 SHA 看领先主线，但 `git cherry` 显示补丁内容已等价进入主线）：通常是 Squash Merge 留下的旧分支，**列给用户确认后再清理**，不自动删。
   - `AHEAD_CLEAN`（工作区干净，含尚未进入主线的独有提交）与 `ACTIVE_DIRTY`（含未提交或未跟踪改动）：**严格保护**，原样保留并提示分支名与提交摘要。

### 3. 执行安全回收（仅在用户要求清理时）

确认需要清理死目录时，运行不带强制覆盖的安全回收：

```bash
agent-on wt --prune
```

`--prune` 只移除 `REAPABLE` 类别的关联 worktree 目录（内部调用不带 `--force` 的 `git worktree remove` 并清理失效元数据），**保留本地与远端 Git 分支引用**。对 `LIKELY_SQUASHED` 或 `AHEAD_CLEAN` 目录，先向用户展示分支名、领先提交数与最后提交说明，由用户逐项拍板。

**完成判据**：清理前后均无未提交文件或独有提交丢失，输出已回收目录数量与保留原因清单。
