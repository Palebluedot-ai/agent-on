# Snapshot — 2026-07-15 · v0.5 Plugin 打包调研与 MRD（研究已成，待拍板建设）

> 职责边界：固化 v0.5「symlink → 单源多工具 plugin」的调研结论与建设范围。这是 agent-on 吃自己狗粮——v0.5 走 BOOTSTRAP §1.5 规划链的「调研 + MRD」两段产物。上一段（v0.4 封版）见 [CHANGELOG](../CHANGELOG.md) v0.4.0 节。
> 证据源：本机侦察（superpowers 6.0.3 in-tree 参照）+ deep-research run wf_e4b00d7b-c62（103 代理，25 主张对抗核验 23 存活 2 否决）。区分「官方文档明确/已验证」与「需实测」如实标注。

## 状态一句话

v0.5 形态**可行且有活参照**（superpowers 就是单源多工具 plugin），但研究挖出**三个会卡实施的真实风险**（其中一个可能让 Codex 侧 hook 打包直接不成立）——故 v0.5 不是「照抄就发」，是「低风险件先建 + 高风险件设测试闸门」。**用户 2026-07-15 拍板路线 A（分期发）。阶段 1 已落地。**

## 阶段 1 已完成（2026-07-15，纯增量零风险，全 3-0 机制）

落地四样（commit 见 CHANGELOG v0.5 条）：
- `skills/agent-on` → `../skill` 符号链接（plugin 约定别名，canonical 仍是 `skill/SKILL.md` 不动；三条路径 `skills/agent-on` / `~/.claude/skills/agent-on` / `~/.agents/skills/agent-on` 经 readlink -f 全解析到同一文件，3 项目挂载零影响）
- `.claude-plugin/plugin.json`（Claude manifest，name/desc/version/author/homepage/keywords）
- `.claude-plugin/marketplace.json`（自营单仓 marketplace，`source:"./"`，Codex 经 legacy-compat 读同一份）
- `.codex-plugin/plugin.json`（Codex manifest，`skills:"./skills/"` + `hooks:{}` 显式空对象抑制 auto-discovery 误注册[F8] + interface 块）

**已验证**：三 JSON 全合法；**Codex 真实解析器实测通过**——`codex plugin marketplace add <本地仓>` 返回 `Added marketplace 'agent-on'`，证明其解析器读通 `.claude-plugin/marketplace.json` 并正确取出插件名（测试后 `remove` 还原，机器无残留）。
**未验证（留作闸门）**：Claude `/plugin marketplace add` 端到端 install（TUI 命令，需真会话；且本机 symlink 挂载在跑，install 会与之双挂 agent-on skill 冲突——install-test 必须在净机或临时摘挂载后做，不在本机与 symlink 并行）；skill 经 plugin 加载是否跟随符号链接别名（同属 install-test 闸门）。

## 一、v0.5 是什么（MRD 级）

**做什么**：把 agent-on 从 symlink 挂载（`setup.sh` 手工挂 5 处）升级为 plugin marketplace 安装——用户 `/plugin marketplace add` + install 一条命令装上，hook 随装自动挂、随卸自动摘。
**给谁**：编程小白（降低"换台机器要跑 setup.sh、手工注册 hook"的心智负担）+ 未来的外部用户（marketplace 是对外分发的标准形态）。
**为什么现在**：触发条件「先自用磨 2-3 项目」已由 Euan/IPONews/AInvestment 达标（2026-07-12 codex-kernel 卡记录）；且 v0.4 的 guard hook 现硬编码路径 + 手工注册（还被 soft_deny 拦过），plugin 化正好一并解掉它的分发短板。

## 二、已验证的机制（官方文档明确，可直接落地）

| 项 | 结论 | 证据 |
|---|---|---|
| 单源多工具骨架 | 一个 repo，顶层共享 `skills/` + `hooks/`，per-tool manifest `.claude-plugin/plugin.json` + `.codex-plugin/plugin.json` 各指同一份 | superpowers in-tree + F5/F7（3-0）|
| skill 内核复用 | 两家都认同一份 `skill/SKILL.md`；Claude 按约定 auto-discover 顶层 `skills/`（manifest 无 skills 字段），Codex manifest 显式 `"skills":"./skills/"` | F5（3-0）|
| 单 repo 免独立 marketplace repo | repo 内放 `.agents/plugins/marketplace.json`，plugin source `{"source":"url","url":"./"}` 指仓根，Codex 可直接从本仓装 | F7（3-0）|
| pin/lock 语义保得住 | Claude `plugin.json` 的 `"version":"x.y.z"` = **bump 才更新**（光 push commit 不推送更新）；marketplace 用 git-subdir 的 `sha` 钉死 commit | F1（3-0）|
| hook 随 plugin 生命周期 | Claude：plugin 启用即 auto-register hook（**不写进 settings.json**，只加 enabledPlugins 条目），卸载即摘——正是 guard 从"手工注册"升级的路径 | F3（3-0）|
| 可移植路径变量 | **`${CLAUDE_PLUGIN_ROOT}`**（不是我先前侦察说的 `${PLUGIN_ROOT}`）；Codex 同时设 `PLUGIN_ROOT` 与 `CLAUDE_PLUGIN_ROOT` 兼容——故 hook 命令用 `${CLAUDE_PLUGIN_ROOT}` 两家通吃 | F2/F6（纠正本地侦察）|
| Codex 是真 plugin target | `codex plugin marketplace add` 收 owner/repo/URL/本地目录；Codex 读 `.agents/plugins/marketplace.json`（repo+个人 scope）+ 兼容旧 `.claude-plugin/marketplace.json`（路径兼容非字节兼容，两份各留）| F4（3-0 核心）|
| Codex manifest hooks 字段不许缺 | 缺 hooks 字段 → Codex fallback auto-discover `hooks/hooks.json` 误注册（superpowers v6.1.0→6.1.1 真 bug 修的就是这个）；要么指真文件注册 guard，要么显式 `hooks:{}` 抑制 | F8（3-0）|

## 三、三个会卡实施的真实风险（研究标 implementation-blocking，发版前必实测）

1. **【最高危】Codex 可能还没真执行 plugin 内 hook**（openai/codex#16430）：manifest.rs 解析 skills/mcpServers/apps 但可能**不解析/不执行 hooks**。若属实，Codex 侧的 guard **打不进 plugin**——只能在 Codex 侧保留个人 scope 手工注册（现状），或 v0.5 Codex 侧先不带 guard。**这条直接决定 v0.5 Codex 侧范围，是下方岔路的核心。**
2. **`${CLAUDE_PLUGIN_ROOT}` 在部分 Claude 版本没可靠注入 hook 子进程**（#66557/#42564 等）：**不许删 guard 脚本现有的硬编码 fallback**，直到实机验证注入成立。
3. **git-subdir marketplace source 破旧 Claude 客户端**（#585）：发布用 git-subdir 需要用户客户端够新，或改用 url 型。

另有两个非阻断修正：marketplace.json 顶层 schema（$schema URL + 必填字段）我 brief 里的假设被**否决 0-3**，发布前必须抓官方实例核准确格式；OpenAI plugin 是单厂多面（不跑在 Claude Code），跨厂目标必须**两份 manifest 一个 repo**，不是一个 OpenAI bundle。

## 四、pin/lock 的关键澄清（别踩）

plugin 的 version/sha 门的是**维护者（agent-on 作者）的发版节奏**，不是**下游项目的 per-project 锁**。agent-on.lock.md 的项目级 pin **必须继续存在**，与 plugin 机制并行——而且若用户开了 Claude Code auto-update，bump 版本仍可能自动传播。所以：plugin 机制**补充**而非替代 lock。「小白项目锁定图纸、升级永远显式」这条靠 lock 守住，plugin 化不动它。

## 五、迁移安全（不破坏已接入 3 项目）

per-harness 安装（Claude 与 Codex 各一次 marketplace-add + install，**无统一安装**，superpowers README 明说"每家分别装"）。故 **plugin 与 symlink 并存过渡**：保留 setup.sh/symlink 供 Euan/IPONews/AInvestment 继续用，plugin 作为**新默认**提供，不强制切换。三项目零改动。

## 六、建议的分期建设（低风险先行，高风险设闸）

- **阶段 1（全 3-0 验证，可直接建）**：repo 内加 `.claude-plugin/plugin.json` + `.codex-plugin/plugin.json`（指顶层 skill/ 或规整出 skills/）+ `.agents/plugins/marketplace.json`（url:"./"）；**不含 hook**。产出 = 两家能 install 装上 skill 内核，中文口令可用。
- **阶段 2（闸门：先实测再决定）**：Claude 侧把 guard 打进 `hooks/hooks.json` 用 `${CLAUDE_PLUGIN_ROOT}`——实测风险 2（变量注入）过了才删硬编码 fallback。
- **阶段 3（闸门：#16430 实测）**：Codex 侧 guard 打包——**先做一次真 block 测试**验证 Codex 执行 plugin hook；不过就 Codex 侧维持手工注册，manifest 写 `hooks:{}` 防误注册。
- **阶段 4**：setup.sh 加 plugin 安装引导（并存过渡文案）；README/BOOTSTRAP 换机步骤更新；lock 与 plugin 关系写进文档。

## 七、待拍板岔路（v0.5 Codex 侧范围）

因风险 1（#16430），Codex 侧 guard 能否打进 plugin 未知。两条路：
- **A · 分期发**：阶段 1 先发（两家装 skill 内核，Claude 带 guard），Codex guard 作为阶段 3 待 #16430 实测——**v0.5 早交付、Codex 护栏暂维持手工**。
- **B · 齐了再发**：等 Codex hook 执行验证通过，两家 guard 都随 plugin 才发 v0.5——**交付晚、但双工具对等**。

（我的倾向：A。阶段 1 是纯增量零风险、立刻降低小白装机门槛；guard 的 Codex 侧本就已手工注册在跑，不因分期而倒退。但这是产品节奏取舍，归你。）

## 诚实边界

本份是**调研 + MRD**，非 PRD、非实现。三个 implementation-blocking 风险全部**未实测**（研究是文档层证据，真 block 行为要装上 plugin 才知道）。marketplace schema 待抓官方实例。两家 plugin 系统 2026 年很新（OpenAI plugin 2026-03 发布、Codex/ChatGPT 桌面 07-09 合并、superpowers 07-02 才踩 Codex hook bug），字段级细节易变，建设时对 live docs 复核。
