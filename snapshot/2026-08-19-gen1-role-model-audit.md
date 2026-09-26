# Snapshot — 2026-08-19 · Gen-1 角色体系空转审视（Reviewer / Checker / Curator / Worker / state-manager）

> **职责边界**：本篇裁决「第一代抽象角色」这批资产的去留，并记录改法选型。**不是**角色体系的重建方案（那是 C 档，本篇明确否掉）。现役角色（maintainer / 值守 oncall / 独立对抗审查员）的正文仍在 `playbook/multi-contributor-protocol.md`、`kit/babysit/`、`kit/review-prompt-template.md`，本篇不改它们的权威地位。

## 状态一句话

仓里有**两代角色**：Gen-1（Reviewer / Checker / Curator / Worker / Implementer / state-manager，2026-07-07 随 v0.2 导入）**七条证据全部指向空转**；Gen-2（maintainer / 值守 / 独立审查员）才有机械闸在扛。裁决：**走 B 档——收编三条真活的纪律进现役文档，其余整体归档进 `legacy/`，并删掉唯一还在对外推销的入口**。

## 一、Key Decision（用户 2026-08-19 拍板）

| 选项 | 内容 | 裁决 |
|---|---|---|
| A | 只加「可见降权」横幅，不删不动 | 否 —— 空转件再挂横幅=第二层装样子 |
| **B** | **收编活纪律 → 6 篇进 `legacy/` → 删问卷 `required_roles` 行** | **采纳** |
| C | 补闸复活（progress-template 加 `review_status`、写 `kit/agents/state-manager.md`、状态机接 `audit-lint`） | 否 —— 与 bench/cases/16「能写成一条约束就别写成一个系统」直接冲突；且现役 oncall + worktree 闸已覆盖其目标（单写者 / 串行合并 / 独立审查） |

## 二、Discoveries：Gen-1 空转的七条证据（全部可复跑）

1. **导入后零改动**。`playbook/mechanisms/` 7 篇中 6 篇最后一次 commit 即导入那次（`27f3714`，2026-07-07）；唯一例外 `three-stabilizers.md`（`e0cef89`，2026-07-18）改的是 bottleneck 口径，与角色无关。至今 43 天静止。
2. **角色定义文件从来不存在**。`self-contained-file-system.md:171`、`three-stabilizers.md:69`、`monitoring-and-error-signal.md:81` 三处「相关文件」都指 `agents/state-manager.md`——全仓无 `agents/` 目录、无该文件、kit 无其模板。角色落点是空指针。
3. **状态机没有承载字段**。`reviewer-trigger-and-state-flow.md` 定义 `pending → in_review → passed / failed / checker_review`，`checker-review-rules.md:40` 要求「必须更新 `review_status`」——但 `kit/progress-template.yaml` **不含 `review_status`，不含 `reviewer_invoked_at`**。发给项目的模板没有承载字段，无处可更。
4. **日志落点从没写过东西**。`monitoring-and-error-signal.md:28` 要求结构化 JSON 写 `docs/state/logs/`。全仓提到该路径的第二处正是反证：`intake/2026-07-30-euan-flutter.md:7` —— 「`docs/state/logs/` 建目录 24 天零文件」。
5. **Worker / Curator 只活在 enum 里**。仅存在于 `kit/schemas/{task,result,memory,audit-event}` 的 `executor_role` / `actor_role` / `curator.role` 枚举，而 `kit/schemas/README.md` 自挂横幅「至今未在任何真实项目跑通，开箱、S/M 主路径勿启用」；`ledger/runs/` 不存在。
6. **所有机械闸零涉及**。`grep -rniE "reviewer|checker|curator|state.manager|worker" cli/src/ hooks/ kit/guard/` → 零命中。oncall / worktree / guard / audit-lint / landing 全部不认这套角色。
7. **唯一出口没有下游**。`boot/new-project-questionnaire.md:37` 的 `required_roles:（worker/reviewer/curator）` 是全仓唯一还在向新项目推销这套角色的地方；`grep -rn required_roles` 全仓只此一行——填了没人读。同页 Step 2 硬规则第 1 条还写着「默认新增 **capability**，**不新增永久 role**」，自相矛盾。

**附标本**：`playbook/mechanisms/file-system-archive.md` 结尾至今挂着当年会话的回话「所有四个文档的 Header 已经真正写入并更新完成。」——原样导入、从未复读。该文件是 euan-flutter 项目的存档清单，对 agent-on 无信息量。

## 三、Discoveries：活下来的三样，都不再叫「角色」

| 现役 | 落点 | 机械闸 |
|---|---|---|
| **maintainer** | `playbook/multi-contributor-protocol.md` §三 | CODEOWNERS + CI test gate + merge 记账棘轮 + 分支保护 up-to-date |
| **值守 / oncall** | `agent-on oncall claim/release/status/route/whoami` + `kit/babysit/` + AGENTS §8/§9 | PreToolUse 路由闸，登记落 common git dir；**角色第一次真有代码** |
| **独立对抗审查员** | `kit/review-prompt-template.md` | 非常驻角色，是**一段派工词**：换实例 + 假定有缺陷 + 复算证据 |

关键观察：**Reviewer 不是被执行，是被改写了形态**——reviewer-rules 里活下来的东西（结构化输出 / 禁模糊反馈 / 严重度分级）全部转生进 review-prompt-template，但载体从「一个要维护的角色」变成「一次性任务 + prompt」。

**顺带发现的现役漂移**：`current_bottleneck` 字段，`three-stabilizers.md:39` 说「由 state-manager 维护」，`kit/progress-template.yaml:1` 说「只有 orchestrator 写」——同一字段两份现役文档两个主人名。B 档一并统一。

## 四、根因：仓里自己早写过诊断，只是没扫自己

- `bench/cases/16-gen1-engine-vs-discipline.md`：「**引擎会死，纪律会活**……能写成一条要遵守的约束的，就不要写成一个要维护的系统。」
- `playbook/multi-contributor-protocol.md` §三½：「**纸面规则无闸 = 装样子**……被审计证实空转两周以上的纸面机制：机械化或删除，不许留着装样子。」

Gen-1 角色恰好是这条规则的未执行对象，空转 43 天 = 自设阈值的 3 倍。而该规则写在协作篇里，**从未回头扫过自己的 playbook**——正是 §三½.1 亲手写下的盲区族：**审计不审计审计者**。本篇即该盲区的首次执行。

## 五、Next Steps（B 档执行清单，按序）

1. **收编活纪律进现役文档**（只补出处，不新增概念）
   - Error Signal 四要素 + 严重度分级 + 禁模糊反馈 → `kit/review-prompt-template.md` 补出处一行
   - 单写者不变量的世代演化（state-manager → orchestrator / maintainer）→ `playbook/multi-contributor-protocol.md` §二.1 补一句
   - `current_bottleneck` / `monitoring_summary` 写者名统一为 orchestrator → `kit/progress-template.yaml` 与 `three-stabilizers.md` 对齐
2. **归档**：`playbook/mechanisms/` **7 篇整体**移入 `legacy/gen1-role-model/`。~~`file-system-archive.md` 直接删~~ → **执行时改判随批归档**：它是别的项目的存档清单没错，但结尾那句会话残留是「批量迁入的资产没人真读过」的干净物证，legacy 本就是考古层，留证据比删干净重要（改判记于此，不另开快照）。
3. **断推销入口**：删 `boot/new-project-questionnaire.md:37` 的 `required_roles` 行（与同页 Step 2 硬规则第 1 条冲突且无下游消费者）
4. **索引对齐**：`README.md:192` 与 `playbook/README.md` 的「机制七篇」表述随之更新
5. **收尾**：按 AGENTS §6 硬门，commit 后 `agent-on tag-release --push`；值守在班则交单不自合（AGENTS §8/§9）

**保留不动**：`kit/schemas/` 的 role enum（已有横幅，enum 值本身不构成推销，且 L 档旁路件不在本次射程）；`ledger/` 横幅照旧；三不变量与结账闭环照旧。

## 五½、执行中收到的反例：「Checker 轮」名字撞车（2026-08-19，转述情报）

执行 B 档途中，用户从另一窗口转述了一段正在跑的「Checker 轮」实况：*5 个视角跑完 3 个、起了 4 个对抗核验 agent、出结果 8 条 confirmed（0 条被证伪）+ 17 条 minor、拿真生成器端到端跑出来、结论是开修复 PR*。这是对「Checker 从未触发」的直接反例，**先核再落盘**。

**核验结论：名字撞车，机制不同源。**

| | Gen-1 `checker-review-rules.md` | 转述中的「Checker 轮」 |
|---|---|---|
| 关注面 | 长期影响 / 合规性 / 记忆污染 / 是否需 Curator 固化 | 闸有没有真洞（端到端实跑） |
| 触发 | Reviewer 输出 Critical 或主动标记 | 审计任务本身派工 |
| 裁决词 | `passed` / `failed` | `confirmed` / 被证伪（= CONFIRMED/REFUTED） |
| 收尾义务 | **必须更新 `review_status`** | 开修复 PR，不碰任何状态字段 |
| 对应现役件 | —— | `bench/cases/28` + `kit/review-prompt-template.md` §合规变体（源流 Euan 2026-07-30 `wf_2a0aa710`：5 维取证 + 对抗复核三态） |

**这个反例让结论更强而不是更弱**：Gen-1 三个角色是同一种下场——**定位活下来、载体被换掉**（Reviewer → 派工词；Checker → 对抗核验 stage；state-manager → orchestrator/maintainer）。不是一堆孤立死亡，是一条一致的规律：*角色会死，它守的那条判据会换个载体活下来*——与 `bench/cases/16`「引擎会死，纪律会活」同构。

**取证诚实标注**：该实况为**跨窗口转述**，本会话未直接读到那个 run 的产物（按协作篇 §三½.6 第 1 条，转述是情报不是事实）。上表右列据转述特征与仓内现役件比对得出，**未向该窗口本人复核**。若后续确认它跑的确是 review-prompt 合规变体，可把它升格为「Gen-2 承接方正在真实履职」的活证据补进 `legacy/gen1-role-model/README.md`。

## 六、Open Questions

- `three-stabilizers.md`（Lean / TOC / XP）与 `self-contained-file-system.md`（自包含 + 控制论映射）**去掉角色段之后剩下的部分仍有现役价值**（bottleneck 口径、自包含 phase 卡原则已被 anti-hallucination 机制 5 与 phase-card 模板承接）。是整篇进 `legacy/`，还是先摘出仍活的段落再归档？—— 执行时逐篇判，判据=「这段是否已被现役文档承接」。
- `boot/new-project-questionnaire.md` Step 3 的 `mode:` 与 `review_gate:` 两行是否同属无下游字段？本次只确认了 `required_roles`，另两行待扫。

## 证据链

- 审视会话：worktree `babysit-merge-dispatcher-6aeff4`，分支 `claude/agent-on-roles-review-3e65fb`，基线 `adee119`
- 关键复跑命令：
  - `for f in playbook/mechanisms/*.md; do git log -1 --pretty="%ad %h" --date=short -- "$f"; done`
  - `grep -rn "review_status\|reviewer_invoked_at" kit/`（零命中 = 证据 3）
  - `grep -rniE "reviewer|checker|curator|state.manager" cli/src/ hooks/ kit/guard/`（零命中 = 证据 6）
  - `grep -rn required_roles .`（单行 = 证据 7）
