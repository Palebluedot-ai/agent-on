# 模式:merge 记账棘轮(CI 比对 PR 号)

> 职责边界:把「merge 后更新 progress/dashboard」从 checklist 自觉变成 **CI 可红**。不绑定具体语言——项目可自写 `check_ledger` 脚本;本文件只钉接口与逃生门。
> 源流:Dartify 2026-08-06(78 merged PR / 44 记账 / 严格≤30min 仅 23%;高峰 26 条 0 记账)。digest ledger-ratchet-mechanism。

## 何时用

- 项目已要求:每合一个 PR,状态面(progress / dashboard / TODOS 记账区)出现 **PR 号或 merge commit**
- 依从率用 `gh`+`git` 审计过,确认「只靠自觉」已断档

## 三层(缺一不可)

| 层 | 做什么 |
|---|---|
| **闸** | CI job:列 `SINCE` 之后 merged PR → 状态文件是否含 `#N` / sha;超宽限 → fail |
| **触发器** | 制度上:merge 与记账同一会话完成(checklist 一行不够,要人会做) |
| **执行体** | 日历/巡检另见 worktree-gc-pattern;别用「某日回补」代替闸 |

## 逃生门(闸必带)

1. **关闸**:repo variable 如 `LEDGER_GATE_OFF=true`(事故时一键)
2. **宽限**:merge 后 N 分钟内允许尚未记账(warning 或黄)
3. **解锁**:状态文件补一行合法记账即绿——勿要求重开 PR

## 元动作自涵盖

记账动作自身必须进入被记账集合。`chore(state)` / 补账 PR 若只记别人、不记自己,欠账会自我繁殖。设计时问:「执行这条规则所产生的产物,受不受这条规则约束?」

补账时补**内容**不补号码——闸认的是「PR 号出现在台账原文」,只塞号码能骗过闸。

## 假红纪律

- `gh`/API 失败 → 输出 **「取证失败」**,exit 策略与「真未记账」区分(或 fail-open 并告警)
- 假红训练「红了当没看见」——比没有闸更糟

## 实现草图(伪代码)

```
SINCE = 上线日  # 历史死账不追
for pr in gh pr list --state merged --search "merged:>SINCE":
  if age(pr) < grace: continue  # 宽限
  if not ledger_mentions(pr.number, pr.merge_commit):
    fail(f"merged #{pr.number} not in progress/dashboard")
```

## 与 agent-on 其他纪律

- truth-hierarchy 喂养四元组「漏喂怎么发现」= 本闸
- merge-checklist §7b 状态面同批收口 = 人侧触发器
- 机制须带闸 §三½ = 总原则;本文件 = 实施续章
