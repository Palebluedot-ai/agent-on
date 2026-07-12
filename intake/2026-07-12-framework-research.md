# intake — 2026-07-12 · 框架自研究：deep-research 对抗验证 + 规划链对表 + AINVESTMENT 事故取证

> 来源：用户 /deep-research「框架还有哪些可提高——目标=更好陪伴 Claude Code 与 Codex，最终用户=编程小白」。三条证据线：① AINVESTMENT 冷启动事故现场取证；② deep-research 工作流（107 agent，5 角度 25 源 124 主张，抽 25 条对抗核验：22 存活 / 3 否决）；③ 本机 skill 对表工作流（office-hours / design-consultation / autoplan / spec 正文精读 + 缺口裁决）。研究摘要见文末附录。用户已拍板的方向：规划链**调用现成 skill 而非自研提问流**，bootstrap 只负责把调用启动起来。

### bootstrap-verifiable-landing（bootstrap 落盘与 initial commit 硬验收）
- source:agent-on 自源 2026-07-12 | @ eff1cef
- evidence:① AINVESTMENT 事故：`ls -la ~/Projects/AINVESTMENT` 仅 .git（`git log --all` 为空 = 零 commit）+ .superpowers/，BOOTSTRAP 骨架文件零落盘——advisory 指令典型失效样本；② Anthropic 官方裁决："Unlike CLAUDE.md instructions which are advisory, hooks are deterministic"（code.claude.com/docs/en/best-practices，2026-07-12 抓取，对抗核验 3-0）；③ 对标：spec-kit 用 specify-cli 脚本负责落盘骨架与前置校验，不靠 AI 自觉（github.com/github/spec-kit，3-0）
- confidence:high（本地事故 + 官方文档 + 对标框架三重）
- claim:BOOTSTRAP §2 播种收尾必须 initial commit（骨架文件全部入 git）；§7 验收补一条「`git log --oneline` 非空且含骨架文件」——落盘或 commit 未达成 = 初始化未完成，禁止向用户报完成。远期（dogfood 后）：落盘步骤脚本化（参照 specify-cli），文档只承载判断性方法论
- suggested_landing:BOOTSTRAP §2 七步后新增收尾步 + §7 验收清单；playbook/anti-hallucination.md 可引此案
- rollback:revert 落地 commit
- trace:本会话 AINVESTMENT 取证命令实录；deep-research run wf_97aa9203-085
- 状态:pending

### cross-tool-skill-routing-parity（skill 路由压制必须落双工具共读层）
- source:agent-on 自源 2026-07-12 | @ eff1cef
- evidence:① AINVESTMENT 现场：.superpowers/brainstorm/ 六份 HTML = superpowers brainstorming 在 Codex 会话抢跑，agent-on init 流程被完全覆盖；② 用户的 Skill Routing 压制规则（规划用 GStack、不用 superpowers brainstorming）只写在 ~/.claude/CLAUDE.md（Claude 专属层），Codex 读 ~/.codex/AGENTS.md + AGENT.md，两处均无此节；③ superpowers 自述 "user instructions always take precedence"——压制写了就生效，没写就抢跑
- confidence:high（事故 + 配置文件双实证）
- claim:凡「多 skill 体系冲突时谁优先」的路由裁决，必须写在双工具共读层（机器侧 = AGENT.md，项目侧 = AGENTS.md §skill 路由），不许只活在单工具专属文件；BOOTSTRAP §4 skill 分工尾注补一句「路由含压制：被路由掉的 skill 要点名禁用，不点名 = 可被抢跑」
- suggested_landing:kit/AGENTS-skeleton.md §skill 路由 + kit/AGENTS-lite.md 对应行 + BOOTSTRAP §4 尾注；机器侧 AGENT.md 属 agent-memory 仓（跨仓，消化时提醒用户去补一节）
- rollback:revert
- trace:AINVESTMENT .superpowers/ 目录列表实录 + ~/.claude/CLAUDE.md Skill Routing 节
- 状态:pending

### planning-chain-routing（规划链：调研→MRD→澄清→PRD→方案→审查→拆解 的路由表）
- source:agent-on 自源 2026-07-12 | @ eff1cef
- evidence:① 对表工作流精读四个 skill 正文（wf_f7ec7a5c-dc0）：office-hours Startup mode 六逼问 = 完整 MRD 引导流（demand reality / status quo / target user / wedge，逐题问 + push-twice + 反谄媚）；/spec 五阶段全 issue 粒度（产单张 backlog issue，放产品层 = 层级错位）；autoplan 自述 per-feature 且输入假设 = 已有 plan 文件；需求澄清与 PRD 填写两段无任何 skill 对位 = 全链仅有的真空白；② Kiro 三段闸门（requirements(EARS)→design→tasks，段间显式审批，跳闸 = opt-in，3-0）与 spec-kit 八命令序列（3-0）佐证「一段一动词一文档一闸门」范式；③ 用户拍板：调用不自研
- confidence:high（skill 正文精读 + 双外部对标）
- claim:BOOTSTRAP 定档后接规划链路由表（S 档整链跳过坍缩回现有捷径 / M 选走 / L 全走）：调研 = office-hours Phase 2.75 搭车 + 深调研点名 deep-research；MRD = /office-hours **强制 Startup mode**（禁 Builder 模式路由，忽略其 Phase 6 营销收尾）；澄清 + PRD = **模板问卷化协议**——实例化 prd-template / requirement-pack-template 时每个空节 = 一轮「AI 从上游文档草拟 + 用户勘误」，答不上落 99_待确认，禁止 AI 编内容填空（PRD 大半可从 office-hours design doc 机械转录）；技术方案 = plan mode 按选定 approach 写 rough plan（正是 autoplan 的输入假设，不算缺口）；审查 = /autoplan（per-milestone 喂，禁整本 PRD）；拆解 = agent-on 自持 phase 卡；/spec 下沉为单张 phase 卡的可选精修器。无 GStack 的机器：模板问卷化协议全链兜底
- suggested_landing:BOOTSTRAP §1 与 §2 之间新节（或 §4 尾注扩展成路由表）+ kit/prd-template.md 与 kit/requirement-pack-template.md 头部加问卷化协议注 + kit/README 链条落盘路径表
- rollback:revert（新节独立成段，可整段撤）
- trace:wf_f7ec7a5c-dc0 journal（S/M/L 裁剪表全文在该 run 的 gap 产出）
- 状态:pending

### gstack-artifact-transcription（外部 skill 产物转录铁则与 commit 责任人）
- source:agent-on 自源 2026-07-12 | @ eff1cef
- evidence:① 对表实证：office-hours design doc、autoplan test plan / review logs 全落 `~/.gstack/projects/{slug}/`——项目 repo 之外，不随 clone 走、换机即失，违反「文件系统持久」的实际意图（持久 = 进 repo）；② BOOTSTRAP §2 只建 docs/{state,phases,snapshots}/，产品级文档无 canonical 路径；③ kit/commit-layering.md 只管粒度与前缀，无「环节收口 = commit」约定；L3 只管 progress.yaml，规划文档 commit 责任人未定义
- confidence:high（路径实证）
- claim:走外部 skill 的规划环节，收口动作 = orchestrator 把产物转录进项目 docs/（MRD→docs/product/mrd.md、PRD→docs/product/prd.md、澄清包→docs/requirements/、plan→docs/plans/）+ 一个 commit，否则该环节不算完成（挂 L2「完成 = 贴实际输出」）；orchestrator 主会话 = 规划链落盘与 commit 的唯一责任人（与 L3 单一状态写者同构）；BOOTSTRAP §2 M/L 目录步补 docs/{product,requirements,plans}/
- suggested_landing:BOOTSTRAP §2 步 1（目录）+ §4 L2 扩写（或新增 L8）+ kit/commit-layering.md 加环节收口行
- rollback:revert
- trace:wf_f7ec7a5c-dc0 gap 产出 §4（五条落盘约定缺口全文）
- 状态:pending

### novice-checkpoint-ux（小白回退体验：自动 commit 时间线，git 藏在口令后——方向卡）
- source:agent-on 自源 2026-07-12 | @ eff1cef
- evidence:① Replit 官方：checkpoint = 开发里程碑自动 git commit + 对话上下文 / Agent memory / 环境配置同捆快照，时间线一键 rollback，git 完全藏在抽象后（docs.replit.com checkpoints + snapshot engine 工程博客，3-0×4）；② Claude Code checkpoint 官方边界：不追 bash 改动、不捕跨会话/跨工具改动（3-0）——语义层是框架侧空间；③ Codex 无 checkpoint 等价物（openai/codex#12558 feature request 佐证）；④ 用户痛点原话「没 commit 的情况下（worktree）非常容易乱」
- confidence:medium（外部范式已验证，本框架未实测）
- claim:方向卡——面向小白的 git 卫生正确形态不是教 git，而是「每个口令动作收口自动 commit（中文语义化 message）构成时间线 + 回退口令」。最小件可先落：所有口令动作（整理想法/更新仪表盘/结账/规划链各环节）收口即 commit（与 gstack-artifact-transcription 卡合流落地）；worktree 遵循 Claude Code 官方分诊语义（干净自动清 / 脏必人裁），Codex 侧以文档复刻同一 checklist。完整「回退口令」等 dogfood 信号再设计
- suggested_landing:最小件并入 gstack-artifact-transcription 落地；「回退口令」deferred
- rollback:revert
- trace:deep-research wf_97aa9203-085 findings（checkpoint/worktree/Replit 三组）
- 状态:pending

## 附录：deep-research 摘要（供消化会话引用，非卡）

**存活结论精要（22/25 条，全部 2026-07-12 对抗核验 3-0 除注明外）：**
- Claude Code checkpoint：每 prompt 自动建、跨会话可访问、/rewind 可分开恢复代码与对话；硬边界 = 不追 bash 改动、不捕外部编辑与并行会话 → **跨工具状态共享两家官方均无解，git commit 是唯一可靠载体**
- Claude Code worktree 官方分诊：干净自动删 / 脏弹 keep-or-remove / 子代理未推送豁免（注：「默认从 origin/HEAD 分支」一条被 0-3 否决，勿依赖）
- Kiro：requirements(EARS 固定句式，官方理由 = 无歧义 + 可直译测试)→design→tasks 默认闸门，跳闸 = opt-in；单句 prompt 起步，拆解责任在框架不在小白
- spec-kit：~11 万 star；形态 = md 模板 + bash 脚本 + Python CLI 混合体；`--integration` 一源多渲染（claude/codex 各装各目录）已大规模验证；注意「implement 有程序化前置校验」被 1-2 否决——其 gate 硬度低于观感，真硬 gate 要自己加脚本
- Replit：防翻车 = 基础设施硬隔离（dev/prod 库物理分离，agent 只授开发库），prompt 级护栏实战失效（SaaStr 删库事件，Fortune/The Register/AIID 三方交叉验证）
- Anthropic 官方：CLAUDE.md = advisory，hooks = deterministic；文档越肥遵循度越差，处方 = ruthlessly prune + 反复被违反的规则转 hooks
- jj 反证：agentjj 实测失败已归档（diff/orient/log 全迁回原生 git），git 卫生该在 git 之上加纪律层，不换 VCS（medium，单团队自报）

**开放问题（4，本轮无存活证据，勿当结论用）：** Codex 侧 checkpoint/hooks/worktree 对等机制全缺位（只有 feature request），决定双工具层要自建多少；AGENTS.md 标准业界采纳现状未被存活主张覆盖（基座假设，需补一轮针对性检索）；跨项目 registry / 多项目 dashboard 业界方案无存活证据（可能是空白市场）——且本仓刚裁决 single-human-facing-list（eff1cef），加注册表清单须过那道裁决，列 deferred；Kiro/spec-kit 的问题脚本级 elicitation 需实跑产品逆向。

**来源：** 25 源（primary 9：code.claude.com×3、kiro.dev×3、github/spec-kit、replit×2、claude.com blog、arxiv 等）；全清单与逐条 evidence 在 deep-research run wf_97aa9203-085 output。
