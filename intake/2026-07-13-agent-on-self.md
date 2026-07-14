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

### enforcement-layer-design(强制约束层:三层机械护栏,不再赌 agent 读没读)
- source:agent-on 自源 2026-07-13 | @ 066aa6b
- evidence:deep-research run wf_9c47f385-3e2(106 代理,25 主张对抗核验 22 存活 3 否决),关键存活结论:①Anthropic 官方明文「permission rules 由 Claude Code 强制执行,CLAUDE.md 只塑形不改变允许什么」,真护栏=hooks+permissions(3-0×3);②实证:指令越多遵循越差、冲突时尤甚,但**只去矛盾救不了**——1650 会话析因研究里「相邻文件放直接矛盾指令」对合规率无可测影响(C0=63.7% vs C1=64.1%,p=0.912),会话内每多生成一个函数合规率降 5.6%(单 preprint,medium);③Claude Code PreToolUse hook 收到含 cwd 的完整 JSON,exit 2 拦截先于 permission 求值、压过 allow——目录级 git 禁令的官方指定机制,静态 deny glob 做不到(cd 后命令文本里没有目录信息)(3-0×4);④Codex 2026 已有等价 hooks(PreToolUse deny/exit-2)+ rules/execpolicy(forbidden 免提示硬拦),但 rules 只匹配 argv 前缀不认 cwd——跨仓写入模式 Codex 原生 rules 防不住,必须 hook 或仓侧兜底(3-0×4);⑤SessionStart hook 可机械注入规则(约 10k 字符上限,注入≠遵循仍需行为层兜底)(2-1)
- confidence:high(官方文档为主,单 preprint 结论已降级标注)
- claim:三层设计——**L-进场**:SessionStart hook 双工具注入跨仓边界等硬规矩摘要(治「没读」);**L-动作**(核心):一份共享 guard 脚本挂双工具 PreToolUse——解析命令与 cwd/-C 目标,「git add/commit/push 目标是 agent-on 仓 && 会话项目根≠agent-on」即 deny 并回灌原因(治「越界动作」,不依赖读没读、读哪份);**L-收尾**:Stop hook 结账校验(agent-on 仓有 intake/ 之外改动则不许收工)。分发走 agent-memory setup.sh(hooks 配置不随仓 clone,该仓已是跨机配置分发通道);脚本本体住 agent-on kit/(宪章「轻量校验脚本」额度内)
- suggested_landing:新 kit/guard/agent-on-git-guard.sh + kit/claude-hooks-template.md 扩条 + codex 侧注记;agent-memory settings.json 注册(跨仓,用户操作)
- rollback:摘除 hook 注册即回advisory 现状,脚本可留
- trace:run wf_9c47f385-3e2 output(完整 findings/caveats/openQuestions);未验证项显式列后
- 状态:L-动作层 landed@同批(用户拍板「按建议进行」:kit/guard/agent-on-git-guard.sh + README,实测矩阵 14/14;Codex ~/.codex/hooks.json 已注册,Claude 侧注册被用户 soft_deny 正确拦下、片段备于 guard/README 待用户确认——机械护栏对 agent 自己也生效的活例);L-进场/L-收尾 deferred(L-动作跑两周再议);git 原生 hook 兜底 deferred(本轮零验证证据)
