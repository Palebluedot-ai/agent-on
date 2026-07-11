# intake — 2026-07-12 · 外部输入：Codex 的「一核两适配」提案（经对抗裁决）

> 来源：用户在 Codex 会话获得的架构提案，转来要求对抗审查。裁决过程两个关键事实核查：`$agent-on` 语法与 `~/.agents/skills` 目录**均为真**（Codex 状态文件实证 `[$snapshot](~/.agents/skills/snapshot/SKILL.md)`，格式=Claude 同款 SKILL.md）——审查者初判「疑似幻觉」被证据纠正，对抗双向生效。机器侧 symlink（`~/.agents/skills/agent-on → 仓 skill/`）已挂，`$agent-on` 即刻可用；本文件只收 canonical 改动。

### single-skill-kernel（统一 Skill 内核，消 prompt/skill 双头）
- source:Codex 提案 2026-07-12 | agent-on @ e4105cf
- evidence:①`skill/SKILL.md` 与 `codex/prompts/agent-on.md` 是 2026-07-08/11 先后建的镜像双头，措辞已有微漂——正是 agent-memory 刚治过的病②`~/.agents/skills/` 实证为跨工具共享目录（gstack 全套在此,Codex `$` 语法调用,Claude SKILL.md 同款格式）③机器侧一条 symlink 已让两家吃同一份源文件
- confidence:high（三处实证）
- claim:①`skill/SKILL.md` 确立为唯一内核，头注一行「Claude 经 /agent-on、Codex 经 $agent-on 调用同一份本文件」②内核补三条路由规则：无 lock → 判新项目/存量 → 导 init/adopt；digest 仅限 Agent-On 仓会话（已有,强化措辞）；对表时上游若只有未发布 commit → 播报「有未发布变化」不伪装成可升级版本（此条同落 boot/settlement.md step 0）③`codex/prompts/agent-on.md` 降为三行迁移壳（「Codex 请用 $agent-on,本文件 v0.4 dogfood 后删除」），codex/README 同步
- suggested_landing:skill/SKILL.md + boot/settlement.md step 0 + codex/prompts/agent-on.md + codex/README.md
- rollback:revert;symlink 卸载=rm ~/.agents/skills/agent-on
- trace:Codex 提案原文（用户转述）+ 本会话三步核实
- 状态:landed@c25a27f（2026-07-12 第三次消化——上轮对抗裁决用户无异议+消化口令即拍板）

### 对抗保留项（记录在案，不落地）
- **每 pin 路由**（skill 按项目 pin 读旧版执行书）：deferred——现全 minor（不动手不坏），复杂度>收益；**触发条件=第一个 major 版本出现**
- **能力探测/自动路由/版本状态机**（Codex 预告的「下一节」）：rejected（现阶段）——dogfood 前不设计探测系统，宪章「先固化规则再自动化」+ 一代 5200 行标本
- **Plugin 打包**：已有拍板（先自用磨 2-3 项目），维持
- Codex 两处事实误差已纠：「内核不感知工具」「项目只存 lock」均为现状非提案（它未读仓——装上 $agent-on 后自愈）
