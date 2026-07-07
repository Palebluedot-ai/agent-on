# kit/schemas — 四张编排契约卡

> 职责边界:本目录只放「一次编排 run 里流转的四张卡」的 JSON Schema 与说明;不放实例数据、不放校验逻辑(校验在 `ledger/audit-lint.py`)。
> 源流:一代 agent-orchestration-playbook/04_contracts/,memory_card 部分并入 evolving-memory-system/04_protocols,2026-07-07 批二移植去平台化。

## 四张卡各是什么、何时产生

| 卡 | 文件 | 何时产生 | 谁产生 |
|---|---|---|---|
| task_card | `task-card.schema.json` | **派工时** — orchestrator 主会话把一张 phase 卡切成任务、派子代理下轨 | orchestrator |
| result_card | `result-card.schema.json` | **交活时** — 子代理在本轨 worktree 干完,把结果+证据交回 | worker |
| memory_card | `memory-card.schema.json` | **沉淀时** — run 收官,把可复用的决策/模式/教训固化(仅 reviewer=pass 且过门禁) | curator |
| audit_event | `audit-event.schema.json` | **状态每次变更时** — 每条合法迁移(queued→…→done\|failed)追加一行 jsonl | 谁触发谁记 |

一句话流水:phase 卡 → orchestrator 拆出 **task_card** 派工 → 子代理干活、状态迁移逐条落 **audit_event** → 交回 **result_card** → reviewer 裁决 → 过门禁后 curator 沉淀 **memory_card**。

## 与 phase 卡、run 台账的关系

- **phase 卡**(`docs/phases/`)是 task 的上游:每张 task_card 的 `run_context.phase_card` 指回它,自包含验收条目就是 task 的 `acceptance_criteria` 来源。
- **run 台账**(`ledger/run-ledger-template.md`)是四张卡的下游汇总:一次 run 合流时台账记一行(时长/冲突/悬点/返工/成本),而四张卡是这行背后的可回放明细。audit_event 的 jsonl 就是台账那行的「过程录像」。
- 三处 `run_id` 串起一切:task→result→memory→每条 audit_event 都带同一个 `run_id`,回放时按它捞全链路。

## 设计要点(从一代保留)

- **draft/2020-12 + `required` + `enum` + `additionalProperties`**:结构强约束,但顶层 `additionalProperties:true` 留扩展余地。
- **`allOf` / `if-then` 条件校验**:`scope_level=shared`(要动共享状态,如 progress.yaml、main 契约)时,强制附带 `shared_lock` 与 review/promotion trace——门禁写进 schema 而非靠人自觉。
- **fencing 锁语义**:`shared_lock` 的 `fence_token`(单调递增,防陈旧写覆盖新写)+ `lease_until`(租约到期自动失效)——多轨并行写共享状态的安全底座。

## 去平台化改编(批二)

- 删掉 `channel_id`/`thread_id`/`discord`/`workspace_id` 等一代平台字段。
- 上下文定位换成本环境真实坐标:`worktree_path`(哪条轨)、`run_id`(哪次 run)、`phase_card`(指向 `docs/phases/` 卡片路径)。
- executor 枚举换成 `claude_code`/`codex`/`human`;actor_role 折叠掉 `router`/`hitl`/`memory_keeper`(本环境无独立 Router 进程,派工归 orchestrator,人工介入记 human)。
- memory_card 并入 evolving-memory-system 的更丰富字段:`memory_type`(decision/pattern/anti_pattern/lesson/reference)、`evidence[{kind,value}]`、`confidence`。

## 校验

```
python3 -c "import json;json.load(open('task-card.schema.json'))"   # 逐个验证 JSON 合法
```

状态机迁移合法性(audit_event 的 jsonl)由 `ledger/audit-lint.py` 校验,字段名与本目录 `audit-event.schema.json` 对齐。
