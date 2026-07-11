# intake — 2026-07-11 · 用户两问：自动快照 + 对话想法捕获

> 来源：用户在 agent-memory 改制收尾对话中的两个功能请教（原话见 trace）。按闭环入承接层，下次消化拍板。

### auto-snapshot-triggers（快照从口令驱动升级为事件驱动）
- source:用户提问 2026-07-11 @ 本会话 | agent-on @ cb5face
- evidence:现状=用户说「固化」才写快照（sop Phase 7 已绑「阶段性 snapshot」但靠会话自觉）；用户问「能否对话几个回合后自动 snapshot、用 hook 实现」。技术可行性已核：Claude Code hook 机制真实存在（PreCompact 事件=上下文将满被压缩前触发，快照黄金时机；Stop 事件亦可）
- confidence:medium
- claim:①协议层（工具无关）：sop Phase 7 / merge-checklist 把快照触发点显式化为事件清单（合流时/结账时/上下文将满时/用户口令），明确「回合计数是坏触发器——机械定时产无人读的日志，二代教训」②机制层（Claude 专属）：kit 出一份可选 hook 配置示例（PreCompact→提醒或自动写 snapshot），标注「Claude Code 专属增强，Codex/Grok 靠协议层事件清单」
- suggested_landing:playbook/sop.md Phase 7 一句 + kit 可选 hook 示例文件（消化时裁要不要新文件——冻结令已解除但仍过「谁写谁读」拷问）
- rollback:revert
- trace:用户原话「我能否在对话几个回合之后让系统自动进行 snapshot？可以通过编写一些 hook 代码来实现吗」
- 状态:pending（中风险，选择题）

### conversation-idea-capture（对话中想法自动代笔进速记区）
- source:同上
- evidence:用户原话「平时会在对话框里写一些新想法，系统能不能自动识别出这些属于新功能想法（而不是 debug/『请继续』），连同日期自动同步到指定 md」。骨架已存在：AGENTS-skeleton §9 动态需求协议（中途新想法→复述定位不打断主线）+ v0.4 thoughts-and-ideas.md 两区结构——缺的只是「AI 代笔」这半步
- confidence:high（用户明确要 + 机制延伸自然）
- claim:AGENTS-skeleton §9 与 thoughts-and-ideas 模板头部各加一句：会话中用户冒出新想法/新待办（非 debug、非状态询问、非日常应答）时，AI **当场代笔**写进项目 `thoughts-and-ideas.md` 速记区（带日期 + 「对话捕获」标记），一句话向用户确认后继续主线。三条保险：①只进速记区不进「已整理」（质量闸门保留，整理仍走口令/握手提醒）②保守偏置：拿不准就不记，宁漏勿噪③绝不自动升级为需求——做不做归用户在 requirements 拍板
- suggested_landing:kit/AGENTS-skeleton.md §9 一句 + kit/thoughts-and-ideas-template.md 头部一句 + BOOTSTRAP §6 半句（L3 双落点齐）
- rollback:revert
- trace:用户原话（识别「新功能想法」vs「普通 debug/请继续」；连同日期同步进 md）
- 状态:pending（中风险，选择题）
