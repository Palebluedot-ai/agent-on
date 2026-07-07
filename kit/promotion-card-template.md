# Promotion Card 模板(沉淀出仓的登机牌)

> 职责边界:任何要从项目回流进 agent-on 的沉淀,一条 = 一张卡,六项缺一拒收(playbook/truth-hierarchy.md §四:跨仓写入要证据)。何时用:「agent-on 结账」上半场第 3 步。

```md
### <pattern-slug>(一句话标题)
- source:<项目名> @ <项目 commit> | pin vX.Y.Z
- evidence:<commit / 命令输出摘录 / run_id+行号——至少一个可回放锚点>
- confidence:<high|medium|low>(只在本项目见过一次 = low)
- claim:<可复用规则,一句祈使句>
- suggested_landing:<建议落点:bench 案例 / playbook 哪篇 / kit 哪个模板哪一行>
- rollback:<这条规则若错了怎么撤(通常 = revert 落地 commit)>
- trace:<来源 memory_card 的 run_id/task_id,或 loop-notes 行号>
- 状态:pending  <!-- 消化后原地改 landed@<commit> / rejected(原因) / deferred -->
```

## 纪律

- pattern-slug 用短横线小写英文(如 `local-prod-divergence`)——消化会话靠它 grep 跨项目重复,同 slug ≥2 项目 = 置顶信号
- **evidence 空着 = 这张卡不存在**;「我记得」不是证据
- 六项对应 memory-card schema 字段,结账时可从 jsonl 直接装配
