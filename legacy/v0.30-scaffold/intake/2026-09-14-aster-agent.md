# intake — aster-agent 2026-09-14 结账（首次）

> 项目：`~/Projects/aster-agent`（GitHub Palebluedot-ai/aster-agent，private）。pin v0.19.0 (b9d380c)。档位 S→L（2026-09-06 升档，碰真钱）。
> 本批 5 卡。项目域知识（风控上限 ≥ 最小手数名义值、.env.example 正则误报、Aster 签名细节）留项目端，不装卡。

### tier-probe-test-environment-availability（定档三问漏了「测试环境本身拿得到吗」）
- source:aster-agent @ 7f496bc | pin v0.19.0
- evidence:项目 loop-notes 2026-09-06 第 2 行；浏览器实测 asterdex-testnet.com/en/faucet 页面文案「The faucet is limited to whitelisted users」+ 错误码 099050；搜索证实白名单 2025-12-22 截止（36 万选 1000）；升档 commit 7f496bc「测试网水龙头白名单制不可用，改为主网小额」
- confidence:low（本项目一次；但机制普适：任何依赖第三方测试网/沙箱的项目都可能撞）
- claim:BOOTSTRAP §1 定档三问之外，凡项目依赖外部测试环境（testnet / sandbox / staging 由第三方控制），初始化当场用只读方式验证「拿得到测试资源」，拿不到就按真实环境定档；不许以「有测试网」为由默认 S 档。
- suggested_landing:BOOTSTRAP.md §1 定档三问后加一句「第四问（条件触发）：依赖外部测试环境的，先验证能拿到，否则按真实环境定档」；bench/cases 新增一例（测试网白名单致 S 档假设当日作废）
- rollback:revert 落地 commit
- trace:aster-agent loop-notes.md 2026-09-06|Error Signal 高 行；agent-on.lock.md local_deviations 第 2 行
- 状态: landed@d83fd19（BOOTSTRAP.md §1 定档新增第四问（条件触发：外部测试环境））。随 v0.22.0 的闸改动 commit 一起发出，CHANGELOG 当时漏记，本批补记

### upgrade-backfill-deferred-to-first-milestone（升档补件可延后到首个真实里程碑通过后，但必须登记且同会话补齐）
- source:aster-agent @ 2f1d4af | pin v0.19.0
- evidence:agent-on.lock.md local_deviations 第 3 行「补件延后…不是静默降档」→ 同日更新为「已补件」；commit 7f496bc（升档改禁令）→ 9e86903（冒烟通过）→ 2f1d4af（补件：AGENTS-skeleton / progress.yaml / phase 卡 ×2 / requirements / run-ledger / dashboard）；间隔约 50 分钟
- confidence:low（一次；但 adopt.md §二 对「补件时机」目前无规定，实操必然遇到）
- claim:升档信号触发时，若用户当下要的是一个具体里程碑（如首次真实冒烟），允许先改禁令与风控、把补件排到该里程碑通过之后——条件是：① 当场在 lock local_deviations 登记「补件延后」，② 同一会话内里程碑通过即补齐并把登记改为已补件；跨会话仍未补 = 静默降档。
- suggested_landing:boot/adopt.md §二 升档表加「补件时机」一行；kit/agent-on-lock-template.md local_deviations 注释加示例行
- rollback:revert 落地 commit
- trace:aster-agent agent-on.lock.md local_deviations 第 3 行
- 状态: landed@同批（boot/adopt.md §二 升档表新增「补件时机」行）

### human-executes-money-commands-ai-transcribes（碰钱项目：真钱/签名命令由人执行，AI 只写码、转录、关卡）
- source:aster-agent @ d7af0cd | pin v0.19.0
- evidence:docs/evidence/2026-09-06-mainnet-smoke.md、2026-09-06-strategy-live-3cycles.md、2026-09-06-s0.3-live.md 三份均为用户贴回终端输出、AI 转录判读；phase 卡 S0.2/S0.3 live 条目先标 ⏸「只能由用户执行」后按输出关闭；AGENTS.md §1 高风险域 preflight 行「AI 不代替用户执行真钱命令」；commits 9e86903 / a056041 / d7af0cd
- confidence:medium（同一项目三次里程碑全部按此模式闭环，零伪造、零事故）
- claim:碰钱或持凭据的 L 档项目，把「执行签名/真钱命令」固定为人的动作：AI 写代码与命令、人执行并贴回原始输出、AI 逐条对照验收标准转录进 docs/evidence/ 并关卡；phase 卡的 live 验收条目在输出回来前一律标 ⏸，禁止 AI 以 dry-run 或推断冒充。
- suggested_landing:kit/AGENTS-skeleton.md §1「高风险域 preflight」行加一句；kit/phase-card-template.md 验收旁注「本项验证的真实依赖是什么」处加「真钱/凭据 → 人执行 + AI 转录」；playbook/anti-hallucination 证据篇加一条
- rollback:revert 落地 commit
- trace:aster-agent docs/ledger/run-ledger.md Run #1/#2 轨道行
- 状态: landed@8631471 + 同批（anti-hallucination C 附6 第 10 条 + phase-card 验收区「真钱/凭据」条目 @8631471；kit/AGENTS-skeleton.md §1 高风险域 preflight @同批）

### link-proof-vs-behavior-proof（验收要分「链路证据」与「行为证据」，结构上触发不了的测试只证明链路）
- source:aster-agent @ a056041 | pin v0.19.0
- evidence:loop-notes 2026-09-06|观测 行：两次三轮实跑 12 张 post-only 全 NEW、拒单 0、成交 0；报价离盘口 25 tick 而盘口价差 1 tick，被打中概率≈0；docs/evidence/2026-09-06-strategy-live-3cycles.md「判读」节明写「这一轮验证的是链路不是策略收益」；run-ledger Run #2 教训行
- confidence:low（一次；与 anti-hallucination 第六型「判别式」同源但角度不同：不是约束是否生效，而是行为是否被激发）
- claim:写验收条目时，对「行为类」条目（策略被触发、告警被激发、降级路径被走到）先问「本测试的参数下该行为在结构上可能发生吗」；不可能则该条只能记作链路证据，行为证据另立条目并写明触发方式（调参 / 注入 / 等待），禁止用链路通过冒充行为验证。
- suggested_landing:kit/phase-card-template.md 验收标准区加一条旁注「链路证据 ≠ 行为证据」；playbook/anti-hallucination 判别式条附例
- rollback:revert 落地 commit
- trace:aster-agent loop-notes.md 2026-09-06|观测 行
- 状态: landed@同批（anti-hallucination C 附6 第 8 条 + phase-card 验收区「行为类条目」）

### cross-repo-git-guard-false-positive-on-cp（跨仓 git 守卫按命令文本匹配路径，把「拷 agent-on 模板 + 对项目仓 git」误判为对 agent-on 仓写）
- source:aster-agent @ 2f1d4af | pin v0.19.0（守卫来自 plugin cache agent-on 0.5.0 `kit/guard/agent-on-git-guard.sh`）
- evidence:PreToolUse hook 原文「⛔ 跨仓 git 边界拦截(agent-on-git-guard):项目端会话对 agent-on 仓只写 intake/ 素材文件…被拦命令:cp ~/Projects/Agent-On/kit/dashboard-template.html dashboard.html && python3 … && git add -A && git commit …」——命令里对 agent-on 仓的唯一操作是 `cp` 读；git 操作对象全是项目仓（cwd=aster-agent）。拆成两条命令（先 cp，再 git）即通过，说明判据是「命令文本同时含 agent-on 路径与 git 写动词」
- confidence:medium（确定性复现；任何按 BOOTSTRAP「从 kit 实例化」再同命令 commit 的会话都会撞）
- claim:跨仓 git 守卫的判据应落在 git 命令的实际作用仓（`git -C <path>` / 命令内 `cd` 后的 cwd / 参数中的仓路径）而非「整条命令文本包含 agent-on 路径」；至少放行「只读引用 agent-on 路径（cp/cat/sed 读）+ 对当前项目仓 git」的组合，并在拦截文案里给出出口（「拆成两条命令」）。
- suggested_landing:kit/guard/agent-on-git-guard.sh 判据修正 + 拦截文案加出口；bench/cases 新增一例；v0.20.0「闸必须自带出口」条目的同类
- rollback:revert 守卫脚本改动（拦截面回到更宽即安全侧）
- trace:aster-agent loop-notes.md 2026-09-14|脚手架不合身 行；agent-on.lock.md local_deviations 第 4 行
- 状态: landed@同批（与 CryptoQuant guard 卡合并入册 bench/cases/46；根因改判同该卡：判据早已在 Rust 闸里修好，误拦出自插件缓存 0.5.0 的老闸，执行面见案 47）
