# 决策快照：Landing 控制面 v1（合流协调 + 生命周期）

> 职责边界：记录 2026-08-16 用户提出的多 PR「能不能合」重复检查问题、根因判断、产品裁决与本批落点。它解释为什么这样设计；可执行规则以 `kit/landing-control-plane.md` 与 CLI 实现为准。

## 用户问题

lane 控制面（v0.10–v0.12）解决了「别互相踩」，但 10–15 个 worktree 并行时仍然失控：

- 每个 PR 反复被问“能不能合”，每次都全量重查 CI / 冲突 / 评审；
- worktree 路径成了心智负担——真正要管的是功能轨 / PR，树只是执行容器；
- 合流顺序、依赖、谁在等谁只活在聊天记忆里；
- 没有活跃数量约束，新想法直接开新树，注意力被摊薄。

## 根因裁决

缺的不是又一个轮询 Agent，而是**证据驱动的唯一事实源**：所有 PR 状态集中检查一次，结果绑定 `(PR head SHA, base SHA)`；两者未变绝不重算，main 每合入一条只重查受影响的 PR（依赖边或文件重叠）。判断、排队、合流决策只属于控制轨；执行会话只管自己的代码和修复。

## 产品裁决

- **两个子系统一份快照**：Landing Coordinator（六类合流表 + 依赖/冲突图 + 波次）与 Lifecycle Manager（五类分类 + 活跃上限 + 回收候选），共用 common git dir 的 `agent-on/landing/snapshot.json`；
- **命令面极简**：`landing refresh|status|plan`，refresh 是唯一联网命令，先便宜探针（`gh pr list` + `ls-remote`）再对失效轨逐个取证；
- **六类每轨恰好一种**：FIX > STALE > NOW > NEXT > PARALLEL > SKIP；NOW 每轮只有一条（合并严格串行），SKIP 只替换证据与类别都未变的非行动行；
- **五类每树恰好一种**：REAPABLE（权威合流证据 + clean + 静默期）与 RESCUE（脏 / 未推送 / 无保底孤本）优先于 lane 标签；active lane 的脏是常态，不算 RESCUE；主树永不 REAPABLE；
- **上限在写入点执行**：`claim` 与 `set-status active` 过 `active_cap` 闸（默认 3），`--parked` 排队不占额——读命令只报告，不拦存量；
- **v1 严格只读**：不驻后台、不自动 merge、不自动删树、不代写 lane 状态。

## 为什么不做轮询 / 后台常驻

轮询把「证据没变」也变成成本，且和「按需运行、人是节拍器」的制度冲突。SHA 键控缓存让重复提问自然变成 SKIP；未来要 auto-merge，唯一合法入口就是快照里 `category == NOW`，合流后触发一次 refresh 走增量失效，而不是清表重跑。

## 为什么快照不进 commit

和 lane 合同同理：它是本机交通状态，随 git / PR 事实重算，丢失可重建。进 commit 会制造第二套 canonical 真相并引发跨 worktree 争抢。PR 权威永远在托管平台，合流确认仍要 fresh read-back。

## 已知近似（诚实边界）

- 「base 移动无文件重叠 ⇒ 证据仍有效」是文件粒度近似（改名/删除等罕见情形可能漏判）；要求更严时删快照全量重查；
- gh 的 files 列表每 PR 截断在 100、compare 截断在 300：截断一律按重叠处理（保守判 STALE / 排队），不假装知道无重叠；
- 离线时 base 降级本地 `origin/<default>` 并标注 `local`。

## 本批落点

- `cli/src/landing.rs`：证据决策、六类/五类判定、波次、快照持久化、GhClient（RealGh + 测试 FakeGh）、三条命令（134 测试、clippy 0 警告）；
- `cli/src/worktree.rs`：`active_cap` 配置读取、`claim --parked`、claim/set-status 上限闸；
- `kit/landing-control-plane.md`：数据模型、缓存键、失效规则、判定优先级、auto-merge 挂点；
- `kit/worktree-control-plane.md` / `README.md` / `CHANGELOG.md`：分层交叉引用与用户入口。

## 边界

- 不改 CHARTER：这是判断与排队的本地审计面，不是任务编排运行时；
- hooks 层职责不变（机械边界），landing 不重复它的闸；
- 删除 worktree 仍只走 `gc --dry-run` 的 safe 分类 + 人工执行。
