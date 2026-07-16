---
name: Intake card (upstream contribution)
about: 把 AI 协作过程的教训贡献给 agent-on 承接层——不是直接改 playbook/kit
title: "intake: <pattern-slug 或项目名>"
labels: ["intake"]
---

<!--
规则(与 boot/settlement.md「上游贡献形态」一致):
- 只交 Promotion Card 素材;维护者消化后才会改 canonical
- 必须有 evidence;项目业务/域知识请留在你自己的项目里
- 想走 PR 时: diff 只许 intake/*.md,禁止改 playbook/kit/boot/CHARTER
-->

## Meta

- **source project**:
- **agent-on pin** (if any):
- **contributor**:

## Cards

复制下面块,每张卡一份;六项(+状态)缺一会被拒收。

### <pattern-slug>(一句话标题)

- source: <项目名> @ <项目 commit> | pin vX.Y.Z
- evidence: <commit / 命令输出摘录 / run_id——可回放锚点>
- confidence: <high|medium|low>
- claim: <可复用规则,一句祈使句>
- suggested_landing: <bench 案例 / playbook 哪篇 / kit 哪行>
- rollback: <若错了怎么撤>
- trace: <loop-notes 行号或 memory_card id>
- 状态: pending

<!-- 更多卡继续粘贴 ### 块 -->

## Checklist

- [ ] 每张卡都有 evidence(不是「我记得」)
- [ ] 都是 AI 协作过程教训,不是产品业务规则
- [ ] 我理解:这不会直接改官方 playbook;要等维护者消化 + tag
