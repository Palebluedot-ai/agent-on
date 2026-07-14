# intake — 2026-07-13 · agent-on 自源:跨仓边界与结账协议的冲突收敛

> 来源:用户转来 Euan-Flutter 会话截图(项目端 commit 了 agent-on 仓被叫停)+ 本会话取证。与 /deep-research「怎么强制约束不靠自觉」同场。

### settle-no-git-boundary-align(结账协议对齐跨仓 git 边界:项目端零 git)
- source:agent-on 自源 2026-07-13 | @ 0bac34b
- evidence:①Euan 会话依 settlement 上半场 step4 旧文「commit 后立即 push」在项目端 commit 了 5b4ecdd(取证:该 commit 现为 dangling 对象,已被撤销;文件以未跟踪状态留在工作区)——它执行的是旧协议,撞上 07-13 新立的 AGENT.md 跨仓边界硬规矩,被用户判越界;②两份 canonical(settlement step4 07-12 版 vs AGENT.md 跨仓边界 07-13 版)互相矛盾并存整一天;③同日上午 IPONews 结账同样 commit+push(6e3cf1f),当时还被消化会话当作「新协议生效实证」表扬——同一行为在两份文档下分别是「守规」和「越界」
- confidence:high(当日双实体 + 文档引文对照)
- claim:settle step4 改「只写文件,不碰 git」,git 动作全归 agent-on 仓会话;消化开场三检放行未跟踪 intake/*.md 并新增「收件 commit」步;intake/README 写者规则补 git 动作归属。元教训(供强制约束研究):advisory 文档层连自洽都难保证——规则冲突时 agent 读哪份走哪条路,都「守了规」;强制层(git hook/permissions 硬墙)不依赖读没读、也不依赖读的哪份
- suggested_landing:boot/settlement.md 上半场 step4 + 下半场 step0;intake/README 规则 5
- rollback:revert
- trace:用户提供的 Euan 会话截图;本会话取证命令实录(git cat-file -t 5b4ecdd / git status)
- 状态:landed@同批(用户 07-13 硬规矩已立,本修正为执行既有裁决,非新决策)
