# Landing 控制面：合流协调 + 生命周期分类

> 职责边界：本模式管多 PR / 多 worktree 并行时的「能不能合、按什么顺序合、每棵树处于哪一站」。它是 [worktree 控制面](worktree-control-plane.md)（lane 合同 + 边界闸）之上的第二层：lane 管"别互相踩"，landing 管"排队与调配"。v1 严格只读 + 按需运行：不驻后台、不自动 merge、不自动删 worktree。

## 一句话

**管理单元是功能轨 / PR，不是 worktree 路径；所有检查结果绑定 `(PR head SHA, base SHA)`，两者未变绝不重查。**

层级固定为：

```text
功能轨 / PR
  └─ 分支
      └─ worktree（可丢弃的执行容器）
          └─ Claude / Codex / agent 会话
```

## 命令面

```bash
agent-on landing refresh   # 联网批量取证，增量重算，写快照（唯一联网命令）
agent-on landing status    # 读快照 + 本地 git 事实 → 首页汇总 + 状态表
agent-on landing plan      # 读快照 → 状态表 + 合流波次
```

三条命令都支持 `--json` 与 `--repo <path>`。`status` / `plan` 不联网；没有快照时报错并提示先 `refresh`。快照有年龄标注，过旧时 `status` 会提醒但不擅自联网。

## 数据模型与缓存

快照写在仓库 common git dir 的 `agent-on/landing/snapshot.json`——与 lane 合同同级，所有本机 worktree 共见、不进 commit、丢失可重建。它是缓存，不是第二套 canonical 真相；PR 权威永远在托管平台。

```jsonc
{
  "version": 1,
  "generated_at": "2026-08-16T08:00:00Z",
  "base_branch": "main",            // 托管平台上的合流线名字
  "base_sha": "…",                  // refresh 时的 base 头
  "base_sha_source": "ls-remote",   // ls-remote | local（离线降级，可能过期）
  "tracks": [
    {
      "id": "auth-api",             // lane id；无 lane 的 PR 用 "pr-182"
      "lane_id": "auth-api",        // 可空：PR 可以没有本机 lane / worktree
      "pr_number": 182,             // 可空：lane 可以还没开 PR（只进生命周期，不进合流表）
      "branch": "feat/142-auth-api",
      "url": "…",
      "checked_head_sha": "…",      // ← 证据键的一半
      "checked_base_sha": "…",      // ← 证据键的另一半
      "checked_at": "…",
      "evidence": "fresh",          // fresh | reused-same | reused-valid | invalidated
      "ci": "green",                // green | red | pending | none | unknown
      "review": "approved",         // approved | changes-requested | required | none | unknown
      "mergeable": "clean",         // clean | conflicting | unknown
      "draft": false,
      "files": ["api/auth/…"],      // 取证时的 PR 变更文件（重叠图的输入）
      "files_truncated": false,     // gh 每 PR 最多返回 100 个文件，超出必须如实标注
      "depends_on": ["contract-v2"],
      "owns": ["api/auth"],
      "category": "NOW",            // NOW | NEXT | PARALLEL | FIX | STALE | SKIP
      "reason": "全绿，依赖根节点，可合",
      "worktree": "/path/…",        // 可空
      "lifecycle": "ACTIVE",        // ACTIVE | WAITING | PARKED | RESCUE | REAPABLE
      "lifecycle_reason": "…"
    }
  ]
}
```

### 缓存键与增量失效（核心不变量）

每条轨的证据绑定 `(checked_head_sha, checked_base_sha)`。`refresh` 先跑一次便宜探针（`gh pr list` 只取 number/branch/headRefOid + `git ls-remote` 取 base 头），再对每条轨做如下判定；**只有判定为「重查」的轨才发昂贵的逐 PR 取证**（CI rollup、mergeable、reviewDecision、files）：

| 探针事实 | 判定 | 说明 |
|---|---|---|
| 无缓存记录 | 重查（首查） | |
| head SHA 变了 | 重查（head 移动） | 作者推了新代码 |
| head 未变，base 未变 | **SKIP，复用缓存** | 绝不重复检查 |
| head 未变，base 变了，且某依赖轨的 PR 在此期间合入 | 重查（依赖落地） | 依赖图边触发 |
| head 未变，base 变了，base 移动的文件 ∩ 本轨 files ≠ ∅ | **invalidated → STALE** | 不重查；等 rebase 后 head 移动再查 |
| head 未变，base 变了，文件无重叠 | **reused-valid**，键的 base 半边推进到新 base | 证据仍有效（PARALLEL 的依据） |
| head 未变，base 变了，但 base 移动的文件列表算不出来 | 重查（保守） | 见下 |

base 移动的文件列表优先用本地 `git diff --name-only <旧base>..<新base>`；本地对象缺失时降级 `gh api compare`；都拿不到就对 base 变动涉及的轨保守重查，不假装知道无重叠。

「无重叠 ⇒ 证据仍有效」是文件粒度的近似（改名/删除等罕见情形可能漏判）；换来的是 main 每合入一条只重查受影响的 PR，不让整队重跑。要求更严时删掉快照全量重查即可。

已从 open 列表消失的缓存 PR 逐个查终态：MERGED → 对应轨标记合流完成，其 worktree 进入 REAPABLE 候选评估，并提示 `set-status landed`（v1 不代写 lane 状态）；CLOSED → 提示人工处置。

## 合流表：六种状态，每轨恰好一种

输出格式固定（类别左对齐 10 列，PR 号左对齐 6 列）：

```text
NOW       #182  全绿，依赖根节点，可合
NEXT      #184  等 #182
PARALLEL  #187  与当前变更无重叠，证据仍有效
FIX       #186  CI 红，分配给 auth-api 会话
STALE     #189  main 更新且文件重叠，需要 rebase
SKIP      #191  SHA 未变化，不重复检查
```

判定优先级（自上而下，命中即停）：

1. **FIX** —— CI 红或评审打回：需要改代码。理由标注分配给哪个 lane 会话（无 lane 时留给控制轨派工）。
2. **STALE** —— 证据被 base 移动 + 文件重叠打穿，或托管平台报 CONFLICTING：需要 rebase，不需要新代码。
3. **NOW** —— 全绿（CI green 或无 CI、评审 approved 或无要求、mergeable clean、非 draft）、依赖全部落地、证据有效。多条候选时按「下游依赖数多者优先，再按 PR 号小者优先」排序，**只取第一条**——合并写入严格串行。
4. **NEXT** —— 仅差一条未合入的依赖（`等 #<dep>`），或全绿但与 NOW 的文件重叠、必须排在其后（`等 #<now>`）。
5. **PARALLEL** —— 与当前变更（NOW 的文件域）无重叠、证据有效，但还不能合：CI 运行中、评审未到、draft、或 mergeable 未知。准备与验证可以并行推进。
6. **SKIP** —— 计算出的类别是 NEXT / PARALLEL、证据为 reused-same（两个 SHA 都未变）、且类别与上次一致：本轮完全没碰它。NOW / FIX / STALE 是可执行动作，即使证据复用也照常显示，不降级成 SKIP。

没开 PR 的 lane 不进合流表（没有可合对象），只进生命周期段。

## 生命周期：五类，每棵树恰好一类

`status` 把全部 worktree（含未登记的）+ 全部功能轨分类。判定优先级：

1. **REAPABLE** —— 已合流有权威证据（MERGED PR 的 headRefOid 覆盖 HEAD，或 HEAD 是 base 祖先）、clean、无合后残留 commit、静默期已过。**只报告，绝不代删**；删除仍走 `worktree gc --dry-run` 的 safe 分类 + 人工执行。主 worktree 永不 REAPABLE。
2. **RESCUE** —— 已合流但有脏文件 / 未推送 / 合后残留 commit，或未合流且（脏 ∨ 未推送 ∨ 有未被 base / MERGED PR 覆盖且无远端保底的独有 commit）。绝不能回收；例外：lane 仍是 active 的开发中脏树不算 RESCUE（见下条）。
3. **ACTIVE** —— lane 状态 active：正在开发，脏是常态，计入活跃上限。
4. **PARKED** —— lane 状态 parked 且已通过 RESCUE 检查（干净、无孤本）：上下文已保存，可以安全暂停。
5. **WAITING** —— 其余一切：blocked / ready 的 lane、开着 PR 等 CI / 评审 / 依赖、已合流但静默期未满、干净的未登记树、控制轨待命。不需要 Agent 反复检查。

主 worktree（primary）按同样规则分类，但永不进入 REAPABLE；干净无 lane 时归 WAITING（控制轨待命）。

首页汇总固定五行，对应关系：现在做=ACTIVE、下一批=PARKED、等待中=WAITING、需抢救=RESCUE、可回收=REAPABLE：

```text
现在做：N 条
下一批：N 条
等待中：N 条
需抢救：N 条
可回收：N 条
```

## 活跃轨上限

配置在 common git dir 的 `agent-on/config.json`（没有就用默认值 3）：

```json
{ "active_cap": 3 }
```

执行点有三处：

- `worktree claim` 在已有 `active` lane 数达到上限时拒绝新开 active 轨；想先排队用 `--parked`，lane 直接以 parked 落账，不占上限；
- `worktree set-status active`（从 blocked / parked 拉回）同样过上限闸；
- `landing status` 显示 `active n/cap`，超限时标红提醒（存量超限不阻塞报告，只阻塞新增）。

## 波次规划（plan）

`plan` 在状态表之下给出合流波次：

```text
WAVE 1  合流：#182；并行准备：#187
WAVE 2  #184（等 #182）
前置修复  #186 → auth-api 会话修 CI；#189 → rebase 后重新取证
```

- 合并严格串行：每个波次只有一条真正写入 base；
- PARALLEL 轨在波次内并行准备 / 验证；
- 依赖用 lane 的 `depends_on` 拓扑排层，文件重叠图决定谁必须错峰；
- 波次是建议，不是执行：v1 不发 merge，合流仍由控制轨按 [worktree 控制面](worktree-control-plane.md) 的合流清单逐条执行 + 远端 read-back。

## 职责分层（三层严格分开）

| 层 | 负责 | 不负责 |
|---|---|---|
| Git hooks / PreToolUse guard | 机械边界：别互相踩、越界即拦 | 排队、判断 |
| Landing Coordinator | PR 排队、依赖 / 冲突图、波次、下一动作分配 | 改代码、merge、删树 |
| Lifecycle Manager | ACTIVE/WAITING/PARKED/RESCUE/REAPABLE 分类、活跃上限、回收候选 | 实际删除（仍走 gc dry-run + 人工） |

执行会话只负责自己轨道的代码与修复；只有控制轨读这三层的输出做判断、排队和合流决策。

## 未来加 auto-merge 的挂点（v1 不做）

- 快照里的 `category == NOW` + `evidence != invalidated` 是唯一合法的自动合流入口条件；
- 合流后必须触发一次 `refresh`，让 base 移动走增量失效，而不是清空全表；
- 自动化仍须遵守 [worktree 控制面](worktree-control-plane.md) 的权限边界：删树、删分支、`--force` 永远人工。
