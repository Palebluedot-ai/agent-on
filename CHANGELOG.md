# CHANGELOG — agent-on 发布账本

> 职责边界:人读的版本账本;版本真相 = git annotated tag(不设 VERSION 文件)。semver 判据:**major = 不动手会坏 / minor = 不动手不坏 / patch = 不用知道**;major 条目必附迁移注记,否则不许打 tag。L3 规则改动必须成对列出 playbook + kit 双落点。

## [未发布]

### 第十二次消化（2026-07-19，Euan 结账 2 卡全 landed；定级 minor 攒批）
- **privacy-feature-tension-hybrid-optin**：elicitation-protocol 新增 §五 隐私×能力冲突（结构化默认 + 证据 opt-in + 禁双全叙事）+ prd-template 第 10 条
- **partial-constitution-unpause-dtable**：AGENTS-skeleton 暂停项局部解禁（D 表划界 + 多面同步）+ §9 一行 + truth-hierarchy §六
- 附记：收件 e5878c1；一卡一 commit（83e8dec 隐私 / 7add1e7 局部解禁）；产品口径 D18 留项目端不出仓（域判据）

### 第十一次消化（2026-07-18，IPONews 四结+五结 5 卡全 landed；定级 minor 攒批）
- **long-task-loop-ledger-on-phase**：kit/phase-card-template §2b 可选 Loop 台账 + anti-hallucination 机制5 半句
- **human-async-not-global-bottleneck**：kit/progress-template current_bottleneck 旁注 + mechanisms/three-stabilizers TOC 口径
- **hook-dev-floor-api-prod-gate**：anti-hallucination 附节 dev floor vs prod API gate + claude-hooks-template 文首
- **session-receipt-before-gated-actions**：kit/guard README L-进场会话回执模式 + AGENTS-skeleton 可选 preflight 行
- **modular-prompt-orphan-must-fail**：sop 外部服务集成清单第 7 条 + anti-hallucination 映射行
- 附记：开场收件 970ccf1/573346f；一卡一 commit（ae9bec8/e0cef89/af03b2f/69c7cc6/0c7b2e7）；来源 intake/2026-07-16-IPONews.md + 2026-07-18-IPONews.md

（自 v0.5.1 起攒）

## v0.5.1（2026-07-16）——默认目录 setup + 三工具装机文档

> **patch**：不用知道也能用；推荐新装走 setup 默认目录。存量 pin v0.5.0 可直升。无 major、无迁移注记需求。

### feat：默认工作目录 + setup 一键装机 + v1.0 定义入 snapshot
- **默认 B**：mac/linux `~/.local/share/agent-on`；Windows `%LOCALAPPDATA%\agent-on`；`agent_on_paths` 解析序末档认默认目录
- **scripts/setup.py**：clone/checkout pin、写 config、可选 plugin/symlink、doctor、intake-lint
- **snapshot/2026-07-16-v10-and-setup.md**：v1.0 四条诚实验收 + 目录/setup 拍板
- **AGENTS.md**：当前阶段改为 v0.5.x / 下一里程碑 v1.0（去掉陈旧 v0.2 表述）
- README 五分钟装机改以 setup 为推荐入口；settlement 结账前提示 intake-lint
- **第十次消化开场（2026-07-16）**：intake 零 pending（60 landed / 4 deferred / 2 rejected）；无收件；本批无卡落地。附带：封本 patch + 清理孤儿 worktree

### docs：三工具装机入口写清下载源
- README「给朋友的 5 分钟装机」扩为 Claude Code / Codex / Grok 分节：唯一源 GitHub + 各工具从哪来/怎么挂
- codex/README 远端 marketplace 与离线 clone 路径对齐 public 仓

## v0.5.0（2026-07-16）——Plugin 装机 + 可移植路径 + 上游贡献

> 封版依据：路线 A 分期发；Claude marketplace install + hook 注册本机实测通过；路径废除 Chao 默认 `~/Projects/Agent-On`；上游贡献 L0–L3 协议 + GitHub 模板。**minor**：存量项目 pin 直升即可，实例化文件不动。装机见 README「给朋友的 5 分钟装机」。
>
> 诚实边界（不挡 minor 封版）：Codex 侧 plugin hook 仍 `hooks:{}`（#16430 未接线）；Claude 交互会话 PreToolUse 真拦未在登录 TUI 补跑（合成拦测 + debug 注册已有证据）。

### 主能力（本版用户可见）
- **Plugin 分发**：`.claude-plugin/*` + `.codex-plugin/*` + `skills/agent-on` 别名 + Claude `hooks/hooks.json`（guard 经 `${CLAUDE_PLUGIN_ROOT}`）
- **路径 A/B**：运行包 = plugin；可写工作仓 B = `AGENT_ON_ROOT` / `~/.config/agent-on/config.json` / lock「本地路径」；`doctor`；无 B 时 settle 拒、guard fail-open
- **上游贡献**：默认只「用」；自愿 intake-only PR / Issue；禁止社区直改 playbook/kit
- **第九次消化**：结账回执 default-branch 硬门 + 待消化 N 读时对账

### 上游贡献形态：intake-only PR / Issue，禁止社区直改 canonical（2026-07-16）
- **协议**：`boot/settlement.md` 新增「上游贡献形态」——L0 私货 / L1 intake / L2 运输(PR|Issue) / L3 仅维护者消化；默认不强制 PR；PR diff 白名单 `intake/**`
- **模板**：`.github/ISSUE_TEMPLATE/intake-card.md`、`.github/PULL_REQUEST_TEMPLATE/intake.md`、默认 `pull_request_template.md`
- **双落点轻量**：intake/README 第六条、iteration-loop §六½、promotion-card 纪律、skill 贡献指引、README FAQ
- 与可移植装机正交：plugin(A)+任意 B；回馈≠人人改 main

### 可移植路径：废除 Chao 默认 `~/Projects/Agent-On`（2026-07-16）
- **协议**：装机面 A（plugin / `CLAUDE_PLUGIN_ROOT`）与可写工作仓 B 分离；B 须显式登记（`AGENT_ON_ROOT` → `~/.config/agent-on/config.json` 的 `work_root` → lock「本地路径」）；任意 OS/任意文件夹名
- **实现**：新增 `kit/guard/agent_on_paths.py`（doctor 报告）；guard 改用其 `resolve_work_root`，**未登记 B 时 fail-open**；示例 `kit/agent-on-user-config.example.json`
- **skill**：`$READ_ROOT` / `$WRITE_ROOT` 双解析；子命令 `doctor`；settle/digest 无 B 拒绝；禁止猜 `Projects`
- **文档**：README 远程装机 + Windows；lock 模板本地路径说明；settlement 消化粘贴令用 `$WRITE_ROOT`；codex 全局片段去硬编码
- **验证**：config 拦/放、无 B fail-open、`AGENT_ON_ROOT` 覆盖、lock 解析

### 第九次消化（2026-07-16，IPONews 三结 2 卡全 landed；定级 minor 攒批）
- **settlement-receipt-on-default-branch**：`boot/settlement.md` 上半场 step5 加 **default-branch 硬门**（回执 commit 须为项目 main 祖先，`git merge-base --is-ancestor` 验收；worktree/feature 上只写 intake，回执须 checkout main 或 cherry-pick 后再报完成）+ `boot/session-handshake.md` 读取表「工作树蔓延」行扩 **结账回执困死枝巡检**（与 worktree-sprawl 同族）。双落点均在 boot/ 执行面（结账写点 + 握手读点）
- **pending-digest-reconcile-on-read**：`boot/session-handshake.md` 读取表「待消化」行扩为 **读时对账、不盲信 N** + `boot/settlement.md` 上半场 step0 顺手对账清已收口积压、step6 写明 **N 生命周期**（结账写/抬高；清 0 在下次打开项目的会话，不在消化端——跨仓边界禁止回写项目）。双落点：handshake 读 + settlement 写两侧交叉引用
- 附记：开场工作区有 v0.5 阶段 2–4 WIP，digest 前 stash 隔离、收尾 pop 还原；一卡一 commit（4dac4c2 / 423645f）；用户原话「用户拍板两道防呆入协议」直落未再出选择题
- 来源：`intake/2026-07-15-IPONews.md`（IPONews pin v0.3.0；实证 acf6e4a 困旁支 + loop-notes 粘「待消化 3」）

### 阶段 2–4：Claude guard 入 plugin + 换机文档 + Codex 备件（2026-07-15）
- **阶段 2**：新增 `hooks/hooks.json`——Claude PreToolUse 调 `python3 "${CLAUDE_PLUGIN_ROOT}/kit/guard/agent-on-git-guard.sh"`（随 plugin 启用自动挂载）
- **阶段 3 备件未接线**：`hooks/hooks-codex.json` 已写；`.codex-plugin/plugin.json` **仍** `hooks:{}`——#16430 未实测前不接线
- **阶段 4 文档**：README 换机 A/B；codex/README；hooks/README + kit/guard 两路注册
- **Claude 闸门实测（2026-07-16）**：marketplace add 本仓 → install agent-on@agent-on；details Hooks(1) PreToolUse；debug `Loading hooks from plugin: agent-on`；cache 路径合成 PreToolUse exit 2

### 阶段 1：Plugin 骨架落地（2026-07-15）
- `.claude-plugin/plugin.json` + `marketplace.json`（自营单仓 `source:"./"`）+ `.codex-plugin/plugin.json` + `skills/agent-on → ../skill`
- 验证：JSON 合法；Codex marketplace add 实测通过

## v0.4.0（2026-07-15）——从文档约定到机械强制

> 封版依据：2026-07-11 拍板「攒批至新项目 dogfood 后随 v0.4.0 一并封版」；AInvestment 完成 BOOTSTRAP 全流程 dogfood（init→规划→结账→消化）+ 两默认件实测。本版主线：规划链 §1.5、强制层 guard（PreToolUse 双工具）、项目端零 git 边界、消化协议三缝、第二~八次消化全部内容。minor 档：存量项目 pin 直升即可，实例化文件不动。

### 第八次消化（2026-07-15，AInvestment 首结 3 卡 + Euan 两日 2 卡：4 landed + 1 rejected；定级 minor 攒批）
- **裁决先行（卡A rejected 立边界）**：delayed-data-as-product-truth 判 rejected——**agent-on 只收 AI 协作过程的教训，项目域知识归项目端**（用户裁决；过程内核「外部数据对账」已被 sop 集成清单第 3 条覆盖，再加=指令膨胀）。此判例是出仓判据的第一次显式适用
- **freeze-deferred-channels-as-ban**：AGENTS-skeleton §1 暂停项行补渠道/触点类举例 + BOOTSTRAP 需求六问 Q6 补半句——MVP 后置渠道必须入禁令表，防实现会话当 soft backlog 偷做
- **interaction-reference-not-asset-clone**：BOOTSTRAP 六问 Q2 参照物扩注「两栏拆法：学什么/不复制什么+能力对等边界」+ elicitation-protocol 参照物回显条同补
- **worktree-when-and-collision-guard**：multi-contributor-protocol §二.2 补「何时开 worktree 判据（单轨不开/≥2 轨且文件域可切才开）+ 三共享面归口（design/·contracts/·progress.yaml）」+ merge-checklist 第 1 步合流前文件域对照
- **tool-detail-confabulation-guard**：anti-hallucination 增补**第六型「取证幻觉」**（完成幻觉的姊妹型：把没看见的细节补成看见了）三条机械防线（复跑取证开 -v / 片段≠日志 / 锚点出口前存在性检查）+ 映射表行 + review-prompt 禁令段补取证条款
- 附记：本批为「项目端零 git」新协议 + guard 上线后的首次消化——AInvestment 结账正确地只留 untracked 文件，消化开场收件 commit（bd75914），跨仓边界闭环首次完整走通；一卡一 commit（cb25d9d/4105d0f/2562bba/4f67722/901536c）
- **追加（用户拍板「项目域都归项目」）**：卡A 判例入协议——settlement 上半场 step 2 证据硬门扩「**域判据**」（出仓候选必须是 AI 协作过程教训，域知识不出仓）+ promotion-card 纪律行同落

### 强制约束层 L-动作（2026-07-13，deep-research wf_9c47f385-3e2 + 用户拍板「按建议进行」；定级 minor 攒批）
- **enforcement-layer-design 最小件**：新增 `kit/guard/agent-on-git-guard.sh`（PreToolUse hook，python3 轻量校验脚本）——「会话根不在 agent-on 仓却对 agent-on 执行 git 写操作」即 exit 2 拦截并回灌原因；读操作与 agent-on 自会话放行。实测矩阵 14/14（事故原型/cd 链/相对路径/-C/--no-verify/tag 创建读取边界/四类合法动作零误伤），kit/guard/README 记录注册片段、失效面与回滚
- 注册现状：Codex `~/.codex/hooks.json` 已挂（两家 hooks schema 相同）；Claude 侧写入被用户 autoMode soft_deny 正确拦截（改 ~/.claude/ 需确认）——**机械护栏不认 agent 身份的活例**，片段待用户确认
- 附带修复：Codex Stop hook 的 auto-sync 路径仍指已改名的 `~/claude-memory` → 改指 `~/agent-memory`（死链导致 Codex 侧会话结束不同步）
- 研究结论要点入卡（intake/2026-07-13-agent-on-self.md）：advisory 天花板官方盖章、去矛盾救不了合规率（p=0.912 null）、目录级 git 禁令必须 hook（deny glob 无 cwd）、Codex rules 不认 cwd 故跨仓模式必须 hook 或仓侧兜底；L-进场/L-收尾/git 原生兜底三件 deferred

### 消化协议第四缝（2026-07-13，Euan 越界事故取证 + 跨仓边界硬规矩执行；定级 minor 攒批）
- **settle-no-git-boundary-align**：settle 上半场 step4 从「commit 后立即 push」改为「**只写文件，不碰 git**」——项目端会话对 agent-on 仓不 add/commit/push，git 动作全归 agent-on 仓会话；消化开场三检放行未跟踪 intake/*.md 并新增「**收件 commit**」步（boot/settlement.md 两处 + intake/README 规则 5）
- 根因是**规则冲突不是（只是）没读**：step4 旧文（07-12 修并发缝）与 AGENT.md 跨仓边界（07-13 用户立）矛盾并存一天——Euan 会话依旧协议 commit 被判越界（5b4ecdd 已撤为 dangling，本批收件 e82f0d1），同日上午 IPONews 同样 commit+push 还被当「新协议实证」表扬。同一行为两份文档下分别「守规/越界」，advisory 层连自洽都难保证——此实证直接输入「强制约束层」研究（进行中）
- 卡在 intake/2026-07-13-agent-on-self.md，直落即标 landed（执行用户既有硬规矩，非新决策）

### 第七次消化（2026-07-13，IPONews 二结 4 卡全 landed，一卡一 commit 首跑；定级 minor 攒批）
- **fanout-probe-gate**：workflow-orchestration §〇 探针闸门（大扇出前 1 探针子代理验工具通路，不过不扇出）+ checklist 探针行 + bench 案例 24（同机正反对照：22 代理 66 万 token 零产出 vs 105 代理 0 错）；README 案例计数 23→24 并修陈旧拆分
- **dom-render-verification + pricing-freshness-gate（语义归并同节落地）**：workflow-orchestration 新增 **§三½ 外部事实直核纪律**两条（价格/榜单证据必须浏览器渲染 DOM、表格核总行数防静默截断；型号与价格当日官方页直核+逐项标查证日期，prompt 预填型号只当检索线索）+ checklist「调研型扇出附加条款」两行
- **quota-fallback-subagent-downgrade**：§一第 6 件断点续跑变体（已完成成果照用/缺口降档模型续跑/合成判断留主会话）+ checklist 撞配额预案行——标「单会话实证」
- 附记：一卡一 commit 新规首次实跑（4 卡 4 commit：102b87f/1205a6a/197b20b/eccb2f3），canonical 中间态窗口缩到单卡分钟级；卡2 落地时曾误吞 §四标题，同 commit 内 grep 结构核验抓回——锚点替换必须把锚补回替换文本

### 消化协议第三缝（2026-07-13，Euan 撞窗实证 + 用户拍板选项 1 直落；定级 minor 攒批）
- **digest-per-card-commit**：消化落地改「一卡一 commit」禁攒批（boot/settlement.md 下半场 step 3）——agent-on 工作区是所有项目会话的服务面，攒批拉长 canonical 中间态窗口，并发读者会读到半截规则。实证：第六次消化 12 文件一批收口，窗口期被 Euan 会话撞见未提交 BOOTSTRAP.md。来源：Euan 反馈（其观察行待其下次结账回流，届时按语义归并标已落）；候选项 2「读者按 pin 读」维持 deferred（触发 = 首个 major）

### 第六次消化（2026-07-13，框架自研究 5 卡 + Euan 尾单 1 卡，5 landed + 1 半落半缓；定级 minor 攒批）
- 频次扫描：「未 commit 的工作对其他会话不存在」坑家族跨项目双实证（AINVESTMENT 零落盘零 commit + Euan worktree-sprawl）→ 置顶升 L3 双落点
- **bootstrap-verifiable-landing（L3）**：BOOTSTRAP §2 S 捷径与 M/L 新增第 9 步 initial commit（落盘未 commit = 初始化未完成，禁报完成）+ §7 验收第 7 条 `git log` 证据——boot 侧；kit/commit-layering「环节收口 = 一个 commit」节——kit 侧
- **gstack-artifact-transcription（L3，与上同族成对）**：BOOTSTRAP §4 新增 **L8 产物入仓 + 收口 commit**（外部 skill 产物转录进 docs/ 才算环节完成；orchestrator 是规划链落盘与 commit 唯一责任人，与 L3 同构）+ §2 步 1 补 docs/{product,requirements,plans}/
- **planning-chain-routing**：BOOTSTRAP 新增 **§1.5 规划链**（调研→MRD→澄清→PRD→方案→审查→拆解路由表，M/L 分档，S 跳过；/office-hours 强制 Startup 当 MRD、/spec 下沉单卡精修、/autoplan per-milestone）+ 模板问卷化协议（AI 草拟+用户勘误、答不上落 99_待确认、禁编填）落 prd/requirement-pack 模板头注 + kit/README 启动步骤第 0 步
- **cross-tool-skill-routing-parity**：BOOTSTRAP §4 尾注「路由含压制」+ AGENTS-skeleton §skill 路由压制条款 + AGENTS-lite §0 一行——压制必须写双工具共读层（实证：AINVESTMENT superpowers 抢跑）；机器侧 AGENT.md 属 agent-memory 仓，提醒用户自补
- **novice-checkpoint-ux（半落半缓）**：最小件「口令动作收口即 commit、commit 时间线即回退时间线」并入 L8 与 commit-layering；「回退口令」deferred（触发 = dogfood 中用户真喊回退）
- **single-human-facing-list（Euan 尾单）**：playbook/truth-hierarchy 新增 §五「单读面：人读状态面一个受众只养一份，第二份直接删」+ kit/dashboard-template 铁律第 ⑤ 条「只此一份」（四条铁律 → 五条）
- 附记：本批 5 卡的拍板依据 = 全部内容已在来源会话与用户逐条对过 + 消化口令（与第三次消化同例）；intake 未收口清零

### 消化协议两缝（2026-07-12，框架自研究会话直落，2 卡 landed；定级 minor 攒批）
- **digest-semantic-grouping**：开场频次扫描分组判据从「slug 字面」扩为「slug 或 claim 语义同类」——slug 是 AI 起的会漂，同坑不同名照样置顶升 L3（boot/settlement.md 下半场 step 1）
- **digest-batch-budget**：三态分诊加预算线（硬）——选择题一场一组 ≤10 题，超线按 intake 文件先旧后新处理到线即收口，剩余留承接层播报「剩 N 份下批」（boot/settlement.md 下半场 step 2）
- 附记：两缝均为机制推演的预防性修补（无实体撞例，confidence=medium 如实标注），卡在 intake/2026-07-12-framework-research.md 追加、直落即标 landed

### 第五次消化（2026-07-12，用户三问：保鲜与升级提示，3 卡全 landed）
- 裁决先行：「定期更新」改造为**事件绑定 + 读时痛感**（时钟触发=快照定稿时已钉死的负空间）
- **readme-freshness**：消化收尾加第四件「README 对表」（agent-on 侧，双实证：两次消化都在 README 抓到过期）+ sop Phase 7 里程碑项目 README 对表（项目侧）
- **dashboard-staleness**：session-handshake 读取表加「仪表盘还新鲜吗」行 + dashboard 模板更新时机第三条「握手新鲜度提醒」
- **upgrade-nudge**：session-handshake 读取表加「pin 落后了吗」行——每次握手顺带对表 agent-on 最新 tag，落后就播报「可说『agent-on 升级』」；只提示不动手，未发布 commit 不算版本不催

### 第四次消化（2026-07-12，首次跨项目：Euan 二结 7 卡 + IPONews 首结 3 卡 + 并发缝自源 1 卡，全 landed）
- **bench 案例 21-23**（Euan 高置信三张）：双假阴性（测试锚真相源不锚生成物）/ 编辑器类 MCP 三层真相（commit 前查 mtime）/ sed 字符类事故（结构化文件禁 shell 字面量替换）+ 索引与使用时机表
- **worktree-sprawl-patrol（L3 双落点，Euan+agent-on 双处实证）**：kit/merge-checklist 第 7 步全量巡检（拆前三查/孤儿先收编）+ boot/session-handshake 读取表「工作树蔓延」行
- **穷尽式提取模式**：playbook/workflow-orchestration.md 新增 §三（分片提取→双路对抗核对，核对者必须独立生成机械对表清单）
- **design-handoff-as-file**：sop Phase 1「设计稿交接=收文件落仓，链接不是交接物」
- **hard-axis-soft-tags（判据先行）**：playbook/architect-lens.md 尾注同族补条（先给一句话判据再端 schema）
- **IPONews 三条 adopt 软化条款**（均标「单项目实证」）：低摩擦合并变体 / probe 延后条款 / 双真相源按维度分工（boot/adopt.md 三处 + kit/dashboard-template 数据源多源注记）
- **并发缝三件（当日双实体证据）**：settle step4 补「commit 后立即 push+被拒 rebase 重推+同日同项目追加同名文件」；消化下半场新增 step0「开场三检」单写者安全门
- 附记：本批为**首次真实多项目并发**——两项目同日结账零 git 冲突（intake 命名空间设计经受实战），暴露的三条协议缝当场修复

### 第三次消化（2026-07-12，1 卡：Codex 一核两适配提案经对抗裁决）
- **single-skill-kernel（landed）**：`skill/SKILL.md` 确立为唯一内核（头注声明 Claude `/agent-on` 与 Codex `$agent-on` 同源，symlink 各挂 `~/.claude/skills` 与 `~/.agents/skills`）+ 补两条路由规则（无 lock 自动判 init/adopt；HEAD 领先 tag 时诚实播报「未发布变化」不伪装成版本，同落 settlement step 0）；`codex/prompts/agent-on.md` 降为迁移壳（v0.4 dogfood 后删）；codex/README 主路改 `$agent-on`。对抗保留：每 pin 路由（触发=首个 major）、能力探测/自动路由（dogfood 前不设计）、Plugin 打包（维持先自用拍板）。附记：审查者初判 `$agent-on`/`~/.agents` 为幻觉，被实证纠正——对抗双向生效

### 第二次消化（2026-07-11，7 张卡：外部记忆系统输入 5 + 用户功能两问 2）

去向：6 landed / 1 半落半缓。定级 minor；按用户拍板**攒批至新项目 dogfood 后随 v0.4.0 一并封版**。

- **no-database-stance（入宪）**：CHARTER 边界节「记忆不建数据库」（markdown 即记忆，DB 永远可选旁挂、坏退 grep、不进必需件）+ 拍板链登记
- **auto-snapshot-triggers（L3 双落点；设计经对抗评审击穿四处后定稿）**：playbook/sop.md Phase 0「拍板即快照」（D 表搭车）+ Phase 7「快照三写点」（决策边界/险段前/交接收口，时钟回合永不触发）——playbook 侧；boot/session-handshake.md staleness 标红 + 消费记录、kit/claude-hooks-template.md 机械地板（PreCompact agent 型 hook，Claude 专属可选）——boot/kit 侧
- **conversation-idea-capture（L3 双落点）**：BOOTSTRAP §6 + kit/AGENTS-skeleton §9 想法类捷径（boot/playbook 侧）+ kit/thoughts-and-ideas-template 头部「AI 也会代笔」（kit 侧）——三保险：只进速记区/保守偏置/升级需求归用户
- **distill-merge-abstraction**：settlement 下半场「同类多条先合并抽象成一条 L2」
- **memory-health-visibility（半落）**：settlement 收尾「顺口报成长」播报（数字从文件数出禁手编）；元仪表盘维持 deferred（触发=服务 2-3 项目）
- **deferred 两张入 Backlog**：cases-retirement-tiering、semantic-retrieval-adapter（见下）

### 新增（v0.4 功能）
- **Codex 适配层**（2026-07-11 用户提——兑现宪章承诺 3「工具无关」还停在纸面的机器侧半边）：`codex/` 三件——AGENTS-global-snippet（并入 `~/.codex/AGENTS.md` 的口令路由，Codex 不读 CLAUDE.md）+ prompts/agent-on.md（Codex 自定义 prompt 约定的斜杠命令，与 skill/SKILL.md 互为镜像、arg 无关设计）+ README（接入两行/谁写谁读/卸载/诚实边界：中文口令永远主路）。项目侧零适配（AGENTS.md 双工具原生）。README FAQ「Codex 能用吗」与换机步骤已更

### 新增（v0.4 功能，2026-07-10 用户提；待新项目 dogfood 验证后随 v0.4 发版）
- **项目仪表盘**（源自 Euan `docs/dashboard.html` 已验证原型）：`kit/dashboard-template.html`——纯静态单文件零依赖，五块（当前阶段 / To-do 分「🫵等你·🤖等我」/ 执行计划 / 决策台账 / 里程碑），内容集中在 DATA 对象、从真相源重绘。**接线**：BOOTSTRAP §2 第 8 步播种（M/L）+ merge-checklist 第 7 步合流必更 + 口令「更新仪表盘」。四铁律：从真相源读了重绘 / 禁手填 / 单写者 / 忠于真相不定义真相
- **想法收集箱**（源自 Euan `docs/idea.md`）：`kit/thoughts-and-ideas-template.md`——两区（📥速记区你随手写 + 🗂已整理 AI 维护带日期），整理时归类成文标去向（待评估 / 升级需求→requirements / 结账→agent-on / 搁置 / 弃）。**接线**：BOOTSTRAP 播种（全档）+ session-handshake 握手提醒速记区非空 + 口令「整理想法」
- 两个项目内口令：BOOTSTRAP §6 + 全局 CLAUDE.md 路由——「整理想法」（全档）、「更新仪表盘」（M/L）

## [v0.3.0] - 2026-07-09 · 首次真实闭环转完一圈

> **里程碑**：CHARTER 定义的 v0.3 门槛达成——Euan 仓内 `agent-on/` 倒仓首次结账（@345bad6）+ 首次消化（本次会话）跑通。semver：minor（新增内容/模板行/案例/机件，无 breaking，存量项目可不动手）。这是「项目 → 结账 → 消化 → 发布」闭环的**第一次真实运转**（v0.2 那次是自建过程内部验证，非下游结账口令回流）。

### 首次消化（2026-07-09，23 张 Promotion Card 分诊落地）

来源：[intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md)（17 张四镜头评审卡）+ [intake/2026-07-09-euan-flutter.md](intake/2026-07-09-euan-flutter.md)（6 张 Euan 倒仓 delta）。去向：**21 landed / 1 deferred / 1 rejected**，零残余 pending。

- **低风险直落 7 卡**（@72185bb）：README 诚实化（删「已转过一圈」幻觉、加术语表 5 行、换机三步 FAQ、斜杠 quirk 说明）、settlement 首结账空锚分支 + 幂等注、BOOTSTRAP 档播错不重播、kit/README 限定 M/L 启动步骤 + 删倒仓前旧句
- **中高风险用户拍板 12 卡**（@d42344a）：
  - `digest-friction-paste`：settlement 收尾从「一句问句」改为默认动作（可粘贴消化开场 + 项目 loop-notes 待办位）+ session-handshake 读取表加「待消化」行
  - `prose-first-settle-path`：settlement step1 主路径显式定为「loop-notes 散文 → Promotion Card」，jsonl/audit-lint 降为 L 档旁路并标「尚未实战验证」（settlement + README 结构表 + ledger/run-card-logging 头部）
  - **`cases-delivery-channel`（L3 双落点）**：BOOTSTRAP §4 扫坑指针（playbook 侧）+ kit/phase-card-template 内联要点行（kit 侧）——bench/cases 对下游的送达通道接通
  - **`skill-routing-slot`（L3 双落点）**：kit/AGENTS-skeleton 新增 §skill 路由槽（kit 侧）+ BOOTSTRAP §1 第 5 问采集本机 skill 体系（playbook 侧）
  - `lock-model-premium-field`：kit/agent-on-lock-template 加 model + 保费档位行 + session-handshake 握手对表核模型档位
  - `intake-lint-timing`（争议→现在就做）：新增 `ledger/intake-lint.py`（Promotion 六项完整性校验器，反例测试验证硬门有效）
  - `parallel-live-as-discovery`（Euan 3x 复现 → 升 L3）：kit/merge-checklist step5 加「并行轨各跑 LIVE 当发现器」
  - `readonly-guardrails` / `destructive-api-protection` / `eval-goldset-honesty`（Euan 单次 → bench 案例）：新增 bench/cases/18、19、20 + 索引
  - `architecture-radar`（冻结令期不新开篇）：并入 playbook/architect-lens.md 附节（信号→动作→量级），修复原悬空链
  - `semver-clash`（争议→本次即打 v0.3.0）：里程碑语义与版本号同指 minor bump，累积 minor 工作骑进 v0.3.0
- **deferred 1 卡**：`probe-from-cases-zero`（冻结令期不加探针题，转化时机顺延，Backlog 保留）
- **rejected 1 卡**：`scaffold-not-design`（已在 bench/cases/02 + freedom-vs-discipline §三，结账对照清单漏扫，无新增量）

### 拍板（2026-07-09，四镜头评审后用户裁决）
- **v0.3 门槛砍半**：v0.3 = Euan 倒仓首次结账+消化跑通（单件）；新项目 BOOTSTRAP dogfood 顺延为 v0.4 —— CHARTER 成功标准 + README 路线已更
- **冻结令入宪**：首次真实结账前不新增 playbook 篇目 / kit 模板，修订必须走 intake 消化 —— CHARTER 边界节
- 对外节奏：先自用磨 2-3 个项目再谈开源；首批 16 张评审卡入 [intake/2026-07-09-agent-on-self-review.md](intake/2026-07-09-agent-on-self-review.md) 等首次消化

### 新增
- **迭代闭环六站机制**(2026-07-08,三镜头提案 + 对抗评审合成,ABDC 自举):playbook/iteration-loop.md + boot/settlement.md(结账/消化/升级三口令执行书)+ intake/ 承接层 + kit/promotion-card-template.md + kit/agent-on-lock-template.md
- **Bench 案例集**(批三):bench/cases/ 17 张翻车案例卡(Euan 实战 15 + 一代二代标本 2)
- 前身三仓归档标头(批四):agent-orchestration-playbook / communication-governance-playbook / non-drift-communication-protocol 各自 README 指回本仓

### 变更(L3 双落点成对列出)
- 六类触发采集纪律:BOOTSTRAP §6(playbook 侧)+ kit/merge-checklist.md 第 7 步扫尾行(kit 侧)
- lock 播种步骤:BOOTSTRAP §2 第 7 步(playbook 侧)+ kit/AGENTS-skeleton.md §0 指针(kit 侧)
- CHARTER 新增承诺 4「越用越强」(2026-07-07 用户拍板)
- **外部服务集成清单**:sop.md 新增六条实证清单(playbook 侧)+ BOOTSTRAP L7 精简铁律与 phase-card 集成探针既有(boot/kit 侧)——由批三案例审查倒逼产出:5 张卡引用一个从未存在的清单,消化时真的把它建出来。**这是「案例 → 协议升级」回路的第一次真实运转**(2026-07-08)
- **S/M/L 档位路由 + 存量项目接入**(2026-07-08,用户「高射炮打蚊子?已开工的项目怎么用?」两问触发——「脚手架不合身」信号的即时消化):BOOTSTRAP §1 定档三问 + §2 S 档三件套捷径(boot 侧)+ kit/AGENTS-lite.md 轻装宪法(kit 侧)+ boot/adopt.md 存量接入书(考古→定档→增量补件,含升档协议);README 全面重写为「三入口两口令」产品自述
- **`/agent-on` 斜杠命令**(2026-07-08,用户「怎么更丝滑调用」触发):skill/SKILL.md 六子命令(init/adopt/handshake/settle/digest/upgrade)→ 按表导到 boot/ 对应执行书;skill 源随仓版本化,symlink 挂 ~/.claude/skills/agent-on(不占 claude-memory 同步,换机器一行 symlink);disable-model-invocation(与中文口令 CLAUDE.md 路由分工:斜杠管确定性,口令管自然语言)
- **工作流编排防幻觉七件**(2026-07-08,agent-on 融合工程自身实战的即时消化——脚本控制流/schema 强制/证据派工/对抗 stage/断点续跑/单一合流权):playbook/workflow-orchestration.md(playbook 侧)+ kit/workflow-orchestration-checklist.md 任务书七要素+编排七件(kit 侧);与六步协议按「会不会撞文件」分工,补齐七层防幻觉栈的第 2 层 schema 件与第 5 层整层

## [v0.2.0] - 2026-07-07

- 批一:五块骨架(Boot/Kit/Playbook/Bench/Ledger)+ 三代资产迁入 + CHARTER / README / AGENTS / BOOTSTRAP 四门面
- 批二:前身仓精选移植 21 文件——五篇方法论(真相源治理/阶段闸门/元原则/ABDC/沉淀分层)、四张卡 JSON Schema、audit-lint、ABDC 四模板、DoD 门禁、会话握手、深挖问卷、能力真相表、修正闭环
- 融合裁决全记录:snapshot/2026-07-07-fusion-map.md

## [v0.1.0] - 2026-07-01

- 工具定义 snapshot(agent-on 边界拍板)+ Loop Engineering 机制七篇导入(project-kickoff-os 时期)

## Backlog(deferred,等 dogfood 数据说话)

- ~~intake-lint:Promotion 六项机器校验~~ **✓ v0.3.0 落地**(ledger/intake-lint.py,首结账消化时人眼核卡繁琐 = 工程镜头胜出的实证)
- ~~架构雷达机制移植~~ **✓ v0.3.0 落地**(并入 playbook/architect-lens.md 附节,冻结令期不新开篇)
- changelog-lint:「major 无迁移注记不许打 tag」的机器门
- probe-from-cases-zero:案例 08/09/10/14 转化为新探针题(deferred,冻结令期不加;转化时机顺延——冻结令已解除,下次消化可捡)
- 多协作者结账对接(multi-contributor-protocol 条款)
- cases-retirement-tiering:案例集「活跃扫坑清单 vs 归档库」两级可见性——升成 L3 门禁的案例退出默认扫描(触发信号:案例超 ~40-50 张,或 loop-notes 首现「扫坑捞不到/扫一堆无关」)
- semantic-retrieval-adapter:语义检索可选旁挂(如 gbrain 索引本仓)——三到顶信号(README 漏登记≥2 次 / 同坑异措辞漏合≥3 次 / 时机表单场景 >10 卡退化全扫)任一触发才动;三设计闸:永远 optional / 坏退 grep / 不进 BOOTSTRAP 必需件
- memory-health 元仪表盘(触发=服务 2-3 个项目;结账播报半句已于 2026-07-11 先行落地)
- **v0.4 门槛**:一个新项目 BOOTSTRAP dogfood 全流程(v0.3 已达,门槛顺延)
