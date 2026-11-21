---
name: merge-check
description: >-
  Rehearse merging multiple parallel branches or open PRs in memory using git merge-tree
  and file-overlap matrices without touching any working tree. Use during high-concurrency
  multi-agent sprints to plan conflict-free merge waves or verify whether a branch can
  skip redundant main-branch sync reruns.
---

# Merge Check — 高并发冲刺期按需合并预演

单线开发无需值守调度；但当多个 Agent 同时推进数条分支或开放 PR 时，“单看每条分支都绿、合完第一条就把其余全撞红”是并发开发的常态。本技能在不改动任何工作区和分支的前提下，在 Git 对象内存中预演合并冲突与重叠波次。

## 执行步骤

### 1. 运行无副作用合并预演

根据待合流对象，在仓库内执行只读预演：

```bash
# 针对当前仓库所有开放 GitHub PR 做重叠与合并树预演
agent-on merge-check --prs

# 针对指定 PR 编号集合预演
agent-on merge-check --pr 41,42,45

# 针对本地或远端多个候选分支离线预演（无需联网）
agent-on merge-check --branches feat/auth,feat/billing,fix/nav

# 输出 JSON 供脚本或后续步骤消费
agent-on merge-check --branches feat/auth,feat/billing --json
```

> **未安装 `agent-on` CLI 时的原生 Git 退化方案**：
> 1. 用 `git diff --name-only $(git merge-base origin/main <branch>)..<branch>` 取各分支改动文件集合，求交集找热点文件；
> 2. 用 `git merge-tree --write-tree --name-only --no-messages origin/main <branch>` 在内存预演与主线的三方合并（退出码 `0` 首行返回结果树 OID `T1`，非零则后续行即为冲突文件）；
> 3. 串联验证第二条分支能否叠在 `T1` 之上：`git merge-tree --write-tree --merge-base=$(git merge-base origin/main <branch2>) <T1> <branch2>`。

**完成判据**：取得每条候选轨道相对基线的同步状态、两两文件重叠矩阵、热点文件及累计虚拟合并波次。

### 2. 按三类预演结论调度交付

1. **无重叠免冗余同步（`SYNC_FREE`）**：
   - 若主线自该分支切出后有推进，但 `<merge_base>..<base>` 改动的文件集合与本分支改动集合**交集为空**，且未改动共享契约/锁文件，则无需为了追平主线而反复 rebase 重跑整套慢速 CI。
2. **并发热点收敛（`HOTSPOTS`）**：
   - 同一文件若已被 ≥ 2 条在途分支/PR 修改，新任务不再新开第 3 条改同一文件的分支：同一事项直接叠到现有分支上，不同事项等前序波次合入后再动工。
3. **按零冲突波次合入（`Wave 1` → `Wave 2` → `Conflicts`）**：
   - **`Wave 1`**：彼此无文件重叠、且在内存中可与当前主线连续干净合成一棵虚拟树的轨道集合，可按任意顺序安全合入，一条合入不会把同波次其余轨道撞红。
   - **`Wave 2+` / `CONFLICT`**：与主线已存在真实冲突（`BASE_CONFLICT`），或单独看与主线干净、但与 `Wave 1` 某条轨道合并后会产生三方冲突（`PAIR_CONFLICT`）。先合完 `Wave 1`，再让对应作者基于新主线解冲突并做局部增量复验。

**完成判据**：给出明确的波次合入顺序、可免同步直合的分支清单，以及存在真实冲突的具体分支对与冲突文件路径。
