# legacy/gen1-role-model — 考古层(Gen-1 抽象角色体系)

> 职责边界：2026-06-28 在 euan-flutter 项目内写成、2026-07-07 随 v0.2 五块骨架原样迁入 `playbook/mechanisms/` 的七篇规则文档，**2026-08-19 整体归档，只读**。它们定义的角色（Reviewer / Checker / Curator / Worker / Implementer / state-manager）**在 agent-on 的真实运行中从未生效**——审视全文与七条证据见 [../../snapshot/2026-08-19-gen1-role-model-audit.md](../../snapshot/2026-08-19-gen1-role-model-audit.md)。
>
> 归档不是否定：这批文档是「引擎会死、纪律会活」（[bench/cases/16](../../bench/cases/16-gen1-engine-vs-discipline.md)）在本仓的第二个标本，也是「纸面规则无闸 = 装样子」（[协作篇 §三½](../../playbook/multi-contributor-protocol.md)）第一次被回头执行在自己身上的对象。留着，是为了下次想造角色体系时有个可查的前车。

## 归档理由（一句话版）

七篇里定义的六个角色，**零机械闸、零下游消费者、零状态承载字段**，导入后 43 天无人改动；而它们想解决的三件事（单写者、串行合并、独立审查）已被现役的 maintainer / `agent-on oncall` 值守 / 对抗审查派工词以**有闸**的形式覆盖。

## 活下来的部分去了哪（这是本篇最有用的一段）

| Gen-1 里的东西 | 现役承接方 | 形态变化 |
|---|---|---|
| Error Signal 四要素 + 严重度分级、禁模糊反馈 | [kit/review-prompt-template.md](../../kit/review-prompt-template.md) | 四级收敛为三级；**从「角色规则」变成「一次性派工词」** |
| Reviewer 的结构化输出、禁自评、独立实例 | 同上 | 同上——Reviewer 不是被执行，是被改写了载体 |
| Checker 的**定位**（第二轮把关 / 必须独立上下文 / 可推翻第一轮的 `passed`） | [bench/cases/28](../../bench/cases/28-audit-adversarial-verifiers.md) 取证与定罪分离 + [kit/review-prompt-template.md](../../kit/review-prompt-template.md) §合规变体（三态 CONFIRMED/PARTIAL/REFUTED） | 从「常驻角色 + `review_status` 状态机」变成 **workflow 里的对抗核验 stage**：每条 Critical/High 另派独立代理、以推翻为目标 |
| 单一状态写者（`state-manager` 唯一写 progress） | [playbook/multi-contributor-protocol.md](../../playbook/multi-contributor-protocol.md) §二.1 | 写者名三代演化 `state-manager` → orchestrator → maintainer，**不变量一次没动** |
| TOC 瓶颈管理 + human-async 口径 | [kit/progress-template.yaml](../../kit/progress-template.yaml) `current_bottleneck` | 已内联全文旁注，自包含 |
| Monitoring 摘要 | 同上 `monitoring_summary` | 同上 |
| Lean 短反馈时间盒 | [playbook/sop.md](../../playbook/sop.md) `max_feedback_loop_min` + [kit/phase-card-template.md](../../kit/phase-card-template.md) | 从「Agent 约束」变成 phase 卡字段 |
| XP 简单设计 / 反过度设计 | [playbook/architect-lens.md](../../playbook/architect-lens.md) 触发式重构预案 + 协作篇 YAGNI | —— |
| 自包含 phase 卡 + 精确指针 | [playbook/anti-hallucination.md](../../playbook/anti-hallucination.md) 机制 5 + [BOOTSTRAP.md](../../BOOTSTRAP.md) §6 | —— |

## 没有承接方、就此作废的部分

- **Checker Review 这个角色载体**——本篇定义的形态（常驻角色 + 靠 Reviewer 标记触发 + 收尾必须更新 `review_status` + 挂钩 Curator 固化）从未触发过一次。
  > ⚠️ **名字撞车警告（2026-08-19 补记）**：现役实践中口语说的「**Checker 轮**」**不是**本篇这个 Checker，而是上表那个对抗核验 stage（5 视角取证 + 独立代理以推翻为目标 + confirmed/refuted 裁决 + 产出直接是修复 PR）。看到别处在跑「Checker 轮」，那是**活的 Gen-2 机制在履职**，不是归档件复活——别据此以为本次归档判错，更别据此把 `review_status` 状态机捞回来。
- **`review_status` 状态机**（`pending → in_review → passed/failed/checker_review`）+ `reviewer_invoked_at`——`kit/progress-template.yaml` 从来没有这两个字段，无处承载。
- **`docs/state/logs/` 结构化 JSON 日志**——Euan 实测建目录 24 天零文件（[intake/2026-07-30-euan-flutter.md](../../intake/2026-07-30-euan-flutter.md)）。
- **`agents/state-manager.md`**——三篇文档的「相关文件」都指向它，**它从未存在过**，kit 里也没有它的模板。角色定义是个空指针。
- **Curator**——只活在 `kit/schemas/` 的 role enum 里，而那套四卡流自挂横幅「至今未在任何真实项目跑通」。

## 标本注记

`file-system-archive.md` 是 euan-flutter 项目的存档清单，对 agent-on 零信息量，本可直接删。留下的唯一理由：它结尾至今挂着当年会话的一句回话——「所有四个文档的 Header 已经真正写入并更新完成。」**原样导入、一年多没人复读第二遍**，是「批量迁入的资产没人真的读过」最干净的物证。

## 复跑本次审视的命令

```
for f in legacy/gen1-role-model/*.md; do git log -1 --pretty="%ad %h" --date=short -- "$f"; done   # 导入后零改动
grep -rn "review_status\|reviewer_invoked_at" kit/                                                  # 零命中 = 无承载字段
grep -rniE "reviewer|checker|curator|state.manager" cli/src/ hooks/ kit/guard/                      # 零命中 = 无机械闸
grep -rn required_roles .                                                                           # 归档前仅问卷一行，无下游
```
