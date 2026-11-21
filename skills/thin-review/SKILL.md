---
name: thin-review
description: >-
  Run a fast, blank-context adversarial code review via an isolated subagent—skipping
  redundant full-suite test reruns when fresh evidence exists and reviewing only incremental
  diffs on follow-up rounds. Use before merging non-trivial changes, auditing risky PRs,
  or when asked for an independent review / checker pass.
---

# Thin Review — 空白上下文薄对抗复核

同一对话窗口内的模型极易顺着自己刚写代码的思路自我辩护；而传统多轮独立审查又常把 80% 时间浪费在重跑全仓测试和重读未改文件上。本技能用**无聊天历史包袱的独立子代理 + 按证据选测 + 二轮只看增量**，在最小耗时下保留抓真缺陷的能力。

> 纯文案修正、单行注释或机械格式化无需启动本流程；涉及业务逻辑、权限安全、数据迁移、契约变更、并发状态或删改测试时使用。

## 执行步骤

### 1. 准备最小复核证据包（不含对话流水账）

在主会话收集以下客观材料，准备传给独立复核子代理：

- **首轮全量复核（Round 1）**：
  1. 当前需求目标与验收条件（若需求中途改过，只传最新生效口径）；
  2. 基线与当前提交范围（`git merge-base <base> HEAD` 与当前 `HEAD` SHA，附 `git diff --stat <base>...HEAD`）；
  3. 作者已在**当前 `HEAD` 提交**上跑过的测试命令与实际输出摘要。
- **二轮及后续增量复核（Round 2+）**：
  1. 上一轮复核过的提交 `prev_head` 与当前提交 `current_head`；
  2. 两者之间的精确增量差异：`git diff <prev_head> <current_head>`（若期间合入了主线，附 `git show --remerge-diff <merge_commit>` 检查冲突解决代码）；
  3. 上一轮的复核结论（列出的阻塞问题与“未覆盖范围”）；
  4. 本轮新增或重跑的局部测试输出。

**完成判据**：证据包中的所有 SHA、命令输出与当前工作树 `HEAD` 严格对应，不含旧提交的过期绿灯。

### 2. 派发空白上下文独立子代理（Blank-Context Subagent）

调用宿主原生子代理工具（如 Antigravity `invoke_subagent` 的 `research`/`self`、Claude Code `Agent` 或 Codex 子代理），**不注入前期实现过程的推理聊天记录**，要求子代理按以下三条纪律独立审查：

1. **两轴对抗审查**：
   - **需求轴**：目标行为是否完整闭环？边界输入（空值、越界、并发重复触发、中断恢复）是否漏判？
   - **契约与安全轴**：调用方/消费方是否兼容？权限、密钥、数据隔离是否守住？若有删除或改写旧测试，原测试保护的业务行为是否仍被有效覆盖？
2. **薄测试纪律（Thin Testing）**：
   - 若作者已提供匹配当前 `HEAD` 的可信测试输出，复核员**只运行与本次改动直接相关的测试文件，或为证实某个具体怀疑而构造的最小复现**，绝不从头重跑全仓测试套件。
   - **例外（必须亲跑）**：作者未贴输出、输出对应的提交落后于当前 `HEAD`、或本次修改的正是测试/门禁脚本本身。
3. **增量复核与异构搜索面**：
   - Round 2+ 默认只审 `prev_head..current_head` 增量并核销上轮问题，同时核对上轮登记的“未覆盖范围”是否依然成立。仅当增量触及上轮未审模块、新增产品行为或上轮记录缺失时，才退回全量审查。
   - 在断言“某处漏改 / 某函数未被调用 / 某配置缺失”前，必须更换搜索词在全仓（含其他分支/worktree 引用）核实，并用一个已知存在的正对照确认搜索命令本身未失效。

**完成判据**：独立子代理返回绑定具体提交范围与文件行号的核查报告。

### 3. 分离阻塞缺陷与非阻塞建议并收口

整理复核结果，第一行给出明确裁决：

- `PASSED`（`full: <base>..<head>` 或 `incremental: <prev>..<head>`）：未发现阻塞缺陷，可继续交付；非阻塞改进建议单列备查。
- `FAILED`：存在已证实的阻塞缺陷（安全/数据风险、契约破坏、验收未达、真实回归）。每条阻塞项必须包含：**位置（`file:line`）· 触发条件 · 实际影响 · 依据/复现证据 · 修正方向**。
- `UNVERIFIED`：代码逻辑未见硬伤，但缺少某项必需的真实环境/跨端证据；写明缺口与补验方式，不伪装成 `PASSED`，也不误判为代码缺陷。

> 风格偏好、顺手重构想法或超出本次需求范围的优化一律归入 **非阻塞建议（Suggestions）**，不得升级为 `FAILED` 阻塞交付。修复阻塞项后回到第 1 步走增量复核，直至无真实阻塞项即收口。
