# 决策快照：多会话 Worktree 控制面

> 职责边界：记录 2026-08-16 用户提出的多 Claude 对话 / 多 worktree 长期失控问题、根因判断、产品裁决与本批落点。它解释为什么这样设计；可执行规则以 `kit/worktree-control-plane.md`、BOOTSTRAP、AGENTS 骨架和 CLI 为准。

## 用户问题

用户已经在实践“一上下文一 worktree 一功能”，但长时间后仍出现：

- 前后功能衍生，执行会话顺手扩 scope；
- 多个 worktree 的依赖与合流顺序只活在聊天脑内；
- 到收尾时不知道哪些已合、哪些有孤本、哪些可以回收；
- 不同会话建议继续做相邻功能，最终撞同一批文件；
- Superpowers 默认链路过重、耗时过长，本轮明确禁用。

## 根因裁决

问题不是“worktree 数量多”本身，而是 **worktree 只有物理隔离，没有控制面**。现有 Agent-On 已有五块正确零件：目录轨道、单一状态写者、契约先行、合流 checklist、保守 GC；但缺少贯穿生命周期的同一份本机事实：

1. 这棵树只交付什么；
2. 它唯一能改哪些路径；
3. 它依赖哪条轨先落地；
4. 它当前处于 active / ready / landed 的哪一站；
5. 它含不含越界、陈旧或唯一副本。

因此，仅坚持“一个上下文一个 worktree”不够；必须升级为“一个写会话一个 worktree 一个轨道合同”。

## 产品裁决

采用 **轻量本地控制面**，不做编排运行时：

- 轨道合同写在 git common dir 的 `agent-on/lanes/*.json`，所有本机 worktree 共见，不进 commit；
- CLI 提供 `worktree claim / set-status / status / check`；
- `claim` 在开工前阻止文件域重叠；`check` 在提交/合流前阻止实际越界；
- `status` 同屏显示依赖、base 落差、独有 commit 与回收分类；
- 不自动 merge、不自动删 worktree；squash / 无 PR / detached 的不确定性一律保守归 `review` 或 `rescue`；
- 长期项目真相仍由 phase / progress / git 承担，本地控制面丢失时可重建，不升级成数据库或第二套 canonical 状态。

## 为什么不用单一共享 YAML

把活跃轨道全写进项目的 `progress.yaml` 会让每个执行分支争抢同一状态文件，正好破坏单一写者；每个分支各带一份 manifest 又无法让其他 worktree 实时看到。git common dir 天然是“一仓多 worktree 共见、但不参与合并”的本地文件系统，正合适承载交通状态。

## 为什么不自动回收

Agent-On 已有 Dartify 实证：无 PR worktree 可能含 +1358 行唯一副本；squash 后 commit 祖先关系又会误判“未合”。所以本批只做可验证分类，不提供 delete 子命令。删除仍由控制轨基于 `safe` + 远端 read-back 人工执行。

## 本批落点

- `cli/src/worktree.rs`：合同注册、生命周期、全量扫描、边界/重叠闸、回收分类；
- `kit/worktree-control-plane.md`：自包含使用模式；
- `BOOTSTRAP.md` / `kit/AGENTS-skeleton.md`：并行一启用就生效；
- `boot/session-handshake.md`：新会话先确认自己在哪棵树、有没有合同；
- `kit/track-prompt-template.md` / `kit/merge-checklist.md` / `kit/worktree-gc-pattern.md`：派工、合流、回收闭环；
- `playbook/multi-contributor-protocol.md`：方法论解释；
- `README.md` / `kit/README.md` / `CHANGELOG.md`：用户入口和版本账本。

## 边界

- 本轮明确不使用任何 Superpowers skill；其他环节 harness 仍可按项目选择。
- 不改变 CHARTER：这是“启动和推进项目”的制度与本地审计器，不执行任务调度，不成为运行时框架。
- 不替 GitHub / GitLab 做 PR 状态权威；远端合并仍 fresh read-back。
