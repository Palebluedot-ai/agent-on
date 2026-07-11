# intake — 2026-07-11 · 用户两问：自动快照 + 对话想法捕获

> 来源：用户在 agent-memory 改制收尾对话中的两个功能请教（原话见 trace）。按闭环入承接层，下次消化拍板。

### auto-snapshot-triggers（快照触发器定稿：三写点 + 机械地板 + 痛感门）
- source:用户提问 2026-07-11 @ 本会话 | agent-on @ 192ecf2 | **设计经完整对抗流程**：初稿「五触发」→ 对抗评审击穿四处（git 类比缺三支柱/会话中段快照无消费者[握手只读最新一份]/拍板已有 D 表强制落盘点勿造平行轮子/「语义为主机械兜底」层级说反）→ 修正定稿
- evidence:①hook 事实已核（官方文档 2026-07-11 版）：PreCompact/Stop/SessionEnd/SessionStart 全存在；hook 有 command/prompt/agent 四型（agent 型可派子代理代写快照初稿，60s）；Stop 可拦截；**项目级 .claude/settings.json 的 hook 可提交进仓**——kit 发模板可行 ②消费侧事实：session-handshake §三只读时间戳最新快照 → 高频中段快照结构性无读者 ③sop Phase 0 D 编号决策表=拍板已有的强制落盘点
- confidence:high（对抗后）
- claim:**三写点**（时钟/回合永不触发）：⑴决策边界=写 D 表同 commit 追一行「为什么链」delta（转折并入，搭已强制的车零新纪律）⑵险段-前=多步/不可逆手术开工前留计划+前态恢复点（git 回滚文件不回滚「我在第几步」，唯一独占；险段-后砍，commit 已捕获终态）⑶交接/收口=开轨/换机/切模型/合流/结账写完整叙事快照（读者[握手]保证出现且上下文归零，ROI 最高；亦是 Codex/Grok 无 hook 时的唯一地板）。**分层**：机械 hook=可靠性地板（kit 出可选 .claude/settings.json 模板：PreCompact→agent 型代写或提醒；标注 Claude 专属），三写点=价值层。**痛感门**：session-handshake 加 staleness 检查（最新快照落后最近 N commit→开场标红）+记录本次读了哪份快照（N 次冷启动后数据裁决哪类快照是死资产，照砍——meta-principle 5）。**负空间**：不快照 D 表/commit 已持有的内容，快照只写「为什么链」delta+下一步锁
- suggested_landing:playbook/sop.md Phase 0（D 表追行）与 Phase 7（三写点清单）+ boot/session-handshake.md（staleness 检查+消费记录）+ kit 可选 hook 模板文件（消化裁新文件与否）——L3 双落点齐
- rollback:revert
- trace:用户原话「只在上下文快耗尽时触发似乎不太好；但又不应该每隔几个回合驱动一次，怎么才能更好？」+ 对抗评审全文（本会话）+ hook 事实核查（guide）
- 状态:pending（中风险，选择题；用户已定「想清楚后一次性消化」）

### conversation-idea-capture（对话中想法自动代笔进速记区）
- source:同上
- evidence:用户原话「平时会在对话框里写一些新想法，系统能不能自动识别出这些属于新功能想法（而不是 debug/『请继续』），连同日期自动同步到指定 md」。骨架已存在：AGENTS-skeleton §9 动态需求协议（中途新想法→复述定位不打断主线）+ v0.4 thoughts-and-ideas.md 两区结构——缺的只是「AI 代笔」这半步
- confidence:high（用户明确要 + 机制延伸自然）
- claim:AGENTS-skeleton §9 与 thoughts-and-ideas 模板头部各加一句：会话中用户冒出新想法/新待办（非 debug、非状态询问、非日常应答）时，AI **当场代笔**写进项目 `thoughts-and-ideas.md` 速记区（带日期 + 「对话捕获」标记），一句话向用户确认后继续主线。三条保险：①只进速记区不进「已整理」（质量闸门保留，整理仍走口令/握手提醒）②保守偏置：拿不准就不记，宁漏勿噪③绝不自动升级为需求——做不做归用户在 requirements 拍板
- suggested_landing:kit/AGENTS-skeleton.md §9 一句 + kit/thoughts-and-ideas-template.md 头部一句 + BOOTSTRAP §6 半句（L3 双落点齐）
- rollback:revert
- trace:用户原话（识别「新功能想法」vs「普通 debug/请继续」；连同日期同步进 md）
- 状态:pending（中风险，选择题）
