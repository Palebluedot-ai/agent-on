# CHARTER — Agent-On 项目宪章（v2 新时代零侵入架构）

> 职责边界：本文件是 Agent-On 总目标、资产结构与产品边界的唯一权威。其他文档与本文件冲突时，以本文件为准；修改本文件须经用户拍板。
> 演进链：2026-07-07 v0.1 方法论脚手架立项 → 2026-07~09 多项目实战沉淀 50 个翻车案例与并行控制面 → 2026-11-06 `v0.30.0` 删除自建派工与网页面板 → **2026-11-22 `v2.0.0` 新时代零侵入重构（用户拍板，见 [重构快照](snapshot/2026-11-22-v2-zero-intrusion-refactor.md)）**。

## 总目标（一句话）

**Agent-On 是一套零业务仓侵入的跨客户端 AI 协作工具包（薄 Skills + 薄 CLI + 一页核心铁律），专门补齐大模型与宿主工具至今未能内化的“单会话视角盲区、自我辩护偏见与本地-真实环境断层”。**

## 时代判断：什么留给宿主，什么由 Agent-On 守住

判断标准只有一条：**凡是靠“单窗口模型变聪明”或宿主原生功能就能做好的，全部交还宿主；凡是受限于跨会话盲区、同上下文圆谎偏见、假绿测试与交付链错位的，浓缩成即插即用的轻量组件。**

### 已交还宿主、不再维护的旧机制（归档于 `legacy/v0.30-scaffold/`）

- **业务仓侵入壳与跨仓结账仪式**：不再往业务仓放 `agent-on.lock.md`、`BOOTSTRAP.md`、`loop-notes.md`、`progress.yaml`，也不再维护 `settle / digest / upgrade` 双边对账流程。
- **基础子代理分派与手工模型档位表**：Claude Code、Codex 与 Antigravity 已原生内化子代理调度与自动开 worktree；固定模型档位表随模型换代迅速过期。
- **常驻值守（`oncall / babysit`）与粗粒度命令正则拦截（`PreToolUse` 钩子）**：宿主自带权限沙箱已足够稳健，日常单线开发零钩子延迟、零字面误拦。

### Agent-On v2 核心五件套

| 组件 | 形态 | 解决什么不可内化的问题 |
|---|---|---|
| **`prove-it`** | [skills/prove-it/SKILL.md](skills/prove-it/SKILL.md) | **防假绿测试**：锁死真实业务入口、独立错误对照（故意改坏逻辑看测试红不红）、判别式探针、写后独立回读与交付链核验 |
| **`thin-review`** | [skills/thin-review/SKILL.md](skills/thin-review/SKILL.md) | **防同窗口自我圆谎 + 省掉 80% 空转复核耗时**：空白上下文独立子代理审查、作者已贴当前 HEAD 测试输出则免重跑全套、二轮起只审增量 diff |
| **`wt-doctor`** | [skills/wt-doctor/SKILL.md](skills/wt-doctor/SKILL.md) + `agent-on wt` | **跨客户端 Worktree 孤岛打通**：统一扫描 `.claude`、`.codex`、`.gemini/antigravity` 等全部 worktree 的未提交文件撞车、主线落差预警与已合入死目录安全回收 |
| **`merge-check`** | [skills/merge-check/SKILL.md](skills/merge-check/SKILL.md) + `agent-on merge-check` | **高并发冲刺期合并防撞**：不碰工作区，用 `git merge-tree --write-tree` 在内存中预演多分支/多 PR 两两冲突、免冗余同步判据与零冲突合入波次 |
| **核心铁律 + 案例库** | [rules/core-guardrails.md](rules/core-guardrails.md) + [bench/cases/](bench/cases/README.md) | **反直觉排障与追问判据**：交付链先于环境归因、事实自查 vs 偏好 2×2、门禁退出码保真、外部副作用先回读；配 50 个真实翻车案例按需检索 |

## 产品承诺（可验收）

1. **业务仓零侵入**：接入任何新老项目无需往项目仓库提交任何配置文件、锁文件或目录；本机装一次，跨 Claude Code、Codex、Antigravity 全项目生效。
2. **默认零驻留、零拦截**：无后台守护进程、无端口占用、无常驻 `PreToolUse` 命令拦截钩子；CLI 默认只读，仅在显式传入 `--prune` 或安装参数时执行受控变更。
3. **跨工具同源**：同一套 `skills/`、`rules/core-guardrails.md` 与 `agent-on` 单二进制同时支持原生插件挂载与用户目录软链安装。
4. **安全底线硬约束**：`agent-on wt --prune` 永不传 `--force`、永不删除含未提交改动或独有提交的工作树、永不删除疑似 Squash 合并待人工确认的工作树、永不删除本地或远端分支引用。
