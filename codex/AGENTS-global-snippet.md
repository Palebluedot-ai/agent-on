<!-- 本片段并入 ~/.codex/AGENTS.md（建议放在 Skill Routing 之前）；源流：agent-on 仓 codex/AGENTS-global-snippet.md，升级 pin 时对照更新 -->

## Agent-On（新项目脚手架 + 方法论迭代闭环）
- **装机（每台机器）**：Claude/Codex `plugin marketplace add Palebluedot-ai/agent-on` → `plugin install agent-on@agent-on`；或 symlink 指向本机 clone 的 `skill/`。**路径不绑 `Projects`**：见 skill 的 A/B 解析。
- **可写工作仓 B（结账/消化才需要）**：`git clone` 到**任意路径**后登记其一——`export AGENT_ON_ROOT=<路径>` / `~/.config/agent-on/config.json` 的 `work_root` / 项目 `agent-on.lock.md`「本地路径」。自检：`python3 <read_root>/kit/guard/agent_on_paths.py` 或 `$agent-on doctor`。
- **新项目初始化**：`/agent-on init` 或中文口令；读的是 plugin/`$READ_ROOT` 的 BOOTSTRAP，**不要求**固定文件夹名。**已开工未接入**→ adopt；换会话续跑→ handshake。
- **口令**：「agent-on 结账 / 升级」→ settlement 执行书；intake 只写 **B**；消化必须在 B 会话。未登记 B 时 settle/digest 应拒绝并提示登记，禁止写 plugin cache。
- **项目内口令**：「整理想法」「更新仪表盘」（M/L）。
- **斜杠 / 美元命令**：`/agent-on` 与 `$agent-on` 同一内核（`skill/SKILL.md`）；子命令含 `doctor`。
- 方法论权威 = agent-on 仓（remote + 本机 B）；前身仓已归档为证据层
