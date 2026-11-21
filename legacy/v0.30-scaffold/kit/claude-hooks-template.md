# Claude Code hook 模板（快照机械地板，可选）

> 职责边界：给 M/L 档项目的**可选**增强——用 Claude Code 的 hook 在「上下文将满 / 会话结束」时机械兜底快照。**Claude Code 专属**（Codex/Grok 无 hook，它们的地板 = 快照三写点第 3 条的手动门控，见 playbook/sop.md Phase 7）。BOOTSTRAP 播种时问一句要不要挂；不挂全流程照跑。
> 源流：2026-07-11 第二次消化落地；hook 能力经官方文档核实（PreCompact/SessionEnd 存在、agent 型 hook 可派子代理写文件、项目级 .claude/settings.json 可提交进仓）。
>
> **Hook 不是生产护栏**：IDE SessionStart/PreToolUse 等只是**开发期地板**（防 coding agent 未读规则就 push/批跑）。产品若是「LLM → 你的 API」部署在 Vercel/Cloudflare 等无 agent 运行时的云上，防幻觉必须落在**查询/API 程序层**（无 Evidence 则 refused、结构化字段、禁止裸 chat 当事实源）。见 playbook/anti-hallucination「dev floor vs prod API gate」。

## 挂法

把下面片段并入项目的 `.claude/settings.json`（没有就新建；此文件可 commit 进仓，全组共享）：

```json
{
  "hooks": {
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "agent",
            "prompt": "上下文即将压缩。读 docs/state/progress.yaml 与 docs/snapshots/ 最新一份,若最新快照落后于当前工作(有未记录的拍板/转折/进行中手术),往 docs/snapshots/ 写一份增量快照(命名 YYYY-MM-DD-HHMM-precompact.md,只写「为什么链」delta 与下一步锁,不抄账本);否则不写。",
            "timeout": 60
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "ls docs/snapshots/*.md 2>/dev/null | tail -1 | xargs -I{} echo '最后快照: {}' >> .claude-session-end.log"
          }
        ]
      }
    ]
  }
}
```

## 三句诚实话（读懂再挂）

1. **这是地板不是主力**：PreCompact 触发时会话已上下文枯竭，是全场写作质量最低的时刻——九成价值在三写点（决策边界 / 险段前 / 交接收口），hook 只兜「都漏了」的最后一层。
2. **别挂 Stop 每回合跑快照**：Stop 每个回合都触发，挂快照 = 机械定时的变种，产无人读的日志。
3. **崩溃场景 hook 也救不了**（会话异常死不触发任何 hook）——那时救你的是最后一次 commit 和 progress.yaml，这正是「碎片化持续落盘为主、快照为叙事层」的分层理由。
