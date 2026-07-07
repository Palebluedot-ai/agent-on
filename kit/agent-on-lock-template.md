# agent-on.lock.md 模板(项目侧唯一新增文件)

> 职责边界:BOOTSTRAP 第七步实例化到项目根 `agent-on.lock.md`。三段:pin(用哪版)、结账回执(增量锚点)、偏离登记(哪里不合身)。项目 AGENTS.md 首节放一行指针「agent-on 映射见 agent-on.lock.md」——读者(握手 / 结账 / 升级会话)本来必经 AGENTS.md,指针即可达。

```md
# agent-on.lock

## pin
agent-on @ v0.2.0 (<commit-hash>)   <!-- tag+commit 双记;只有口令「agent-on 升级」允许改这一行 -->
本地路径:~/Projects/Agent-On

## last_settlement
<!-- 每次结账追加一行:<YYYY-MM-DD> → intake/<文件名>;首次为空 -->

## local_deviations(脚手架不合身登记簿)
| 日期 | 偏离了哪条规则/模板 | 为什么 | 状态 |
|---|---|---|---|
<!-- 「照 agent-on 的规则做很别扭」就是最值钱的回流信号,随下次结账自动出仓 -->
```

## 纪律

- 主规则正文一个字不抄(truth-hierarchy §四:下游只映射不复制)
- 从 kit 实例化任何模板时,目标文件头部加一行 `<!-- instantiated-from kit/<文件> @ vX.Y.Z -->`——升级 diff 的基准随文件走,不设集中映射表
- 实例化文件是项目自己的 canonical,上游永不自动覆盖
