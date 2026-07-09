# intake — 2026-07-09 · agent-on 自评审（v0.3 规划前四镜头评审会 + 用户实测摩擦）

> 来源会话：v0.2 收官后的全局评审（四镜头：CEO/工程/DX/反脆弱 + 补漏批评家，全部实读仓库取证）。本文件是首批真实 Promotion Card——按补漏批评家裁决：**评审建议不许绕过闭环直落正文**，全部入承接层，与 Euan 倒仓素材一起走首次消化。消化时中高风险卡打包成选择题给用户。

### dogfood-freeze（冻结令入宪）
- source:agent-on 自评审 @ 045ce31 | v0.2.0
- evidence:全仓 82 份非 legacy canonical md，下游真实消费项目数=0；intake/ 除 README 外空；二代死法 14 篇:0 实现，本仓比例更极端
- confidence:high（四镜头 + 批评家五方共识）
- claim:跑通首次真实结账前，playbook/kit **只删不增**；文档洁癖就是本产品版本的「造引擎」
- suggested_landing:CHARTER 边界节加一条 + CHANGELOG 记
- rollback:revert 落地 commit
- trace:CEO gap#2 / 风险 bet#2
- 状态:landed@a854ce3（2026-07-09 用户拍板入宪）

### v03-gate-halve（v0.3 门槛砍半）
- source:同上
- evidence:门槛现为「倒仓+新项目 dogfood 两件」；风险镜头：门槛越大越不会发生，先证明闭环能转一圈比证明能转两种场景重要十倍
- confidence:high
- claim:v0.3 门槛砍为「Euan 倒仓首结账跑通」单件；新项目 BOOTSTRAP dogfood 顺延为 v0.4 门槛
- suggested_landing:CHARTER 成功标准 + README 路线
- rollback:revert
- trace:风险 bet#1
- 状态:landed@a854ce3（2026-07-09 用户拍板，CHARTER+README 已更）

### readme-loop-claim（README 自我完成幻觉）
- source:同上
- evidence:README 称「回路已经真实转过一圈」——实为 v0.2 自建过程的内部审查，非下游项目经结账口令回流，两者不是一回事
- confidence:high
- claim:改为「v0.2 自建过程内部验证过一次，首个下游真实结账待 v0.3」
- suggested_landing:README 迭代闭环节
- rollback:revert
- trace:CEO quickWin#4
- 状态:pending（低风险，消化直落）

### settle-first-run-fallback（首结账鸡生蛋）
- source:同上
- evidence:lock 模板/adopt 都写 last_settlement 首次留空，settlement step1 却说「以 last_settlement 为锚」——首跑会话会卡在锚在哪
- confidence:high
- claim:settlement step1 加显式分支「锚为空=首次结账，收全部未标 synced 条目」
- suggested_landing:boot/settlement.md step1
- rollback:revert
- trace:工程 gap#4
- 状态:pending（低风险，消化直落）

### settle-idempotent-note（结账断点恢复）
- source:同上
- evidence:结账六步线性流程，无「中断了下次从哪续」说明；新手最可能卡的错误恢复路径为零
- confidence:medium
- claim:settlement 加一句「中断即安全：intake 文件在就算数，重跑口令幂等」；BOOTSTRAP §7 后加「档播错了：改 AGENTS 档位标记+lock 记一行，不重播」
- suggested_landing:boot/settlement.md + BOOTSTRAP §7
- rollback:revert
- trace:DX gap#2
- 状态:pending（低风险，消化直落）

### digest-friction-paste（消化换会话摩擦→粘贴级）
- source:同上（用户已实测表达硌手）
- evidence:结账收尾只有一句问句「现在开消化会话吗」；用户当时没开，intake 即进入无人复诊状态，仅靠 ls 自觉巡检
- confidence:high
- claim:收尾问句改为默认动作——①生成一行可粘贴的消化开场（cd ~/Projects/Agent-On + /agent-on digest + 积压数）②把「待消化 N 张」写进项目 loop-notes 固定待办位，session-handshake 读取表加一行，下次任意握手必带出
- suggested_landing:boot/settlement.md 收尾 + boot/session-handshake.md §三读取表
- rollback:revert
- trace:CEO gap#4 / 风险 gap#3 / DX gap#6 三方独立命中
- 状态:pending（中风险，选择题）

### prose-first-settle-path（散文主路径，jsonl 降为 L 档件）
- source:同上
- evidence:全仓 find 无任何 .jsonl；ledger/runs/ 不存在；audit-lint 校验的是从未产生过的数据；S/M 档只产 loop-notes 散文，settlement step1 却把 memory_card jsonl 列为主来源——采集与消费数据形态对不上
- confidence:high
- claim:结账主路径显式定为「loop-notes 散文条目 → Promotion Card」；jsonl/四卡/audit-lint 标注「L 档多 agent 编排时启用，尚未实战验证」；README Ledger 行分清「Euan 散文数据（真）」与「jsonl 机件（未验）」
- suggested_landing:boot/settlement.md step1 + README 结构表 + ledger/run-card-logging.md 头部
- rollback:revert
- trace:工程 gap#1/#2
- 状态:pending（中风险，选择题）

### lock-model-premium-field（保费调档埋探针）
- source:同上
- evidence:orchestration-future 的保费/资产之分是最有护城河的洞见，但纯叙述零触发机制——模型换代没人会想起跑 capability-probe
- confidence:medium
- claim:lock 模板加一行「model + 保费档位」；握手对表顺带核：模型变了就提示跑 bench/capability-probe.md 调档
- suggested_landing:kit/agent-on-lock-template.md + boot/session-handshake.md
- rollback:revert
- trace:风险 gap#4 / bet#3
- 状态:pending（中风险，选择题）

### cases-delivery-channel（案例集出矿通道断头）
- source:同上
- evidence:bench/cases/README 自称两类读者，第一类（新项目 AI 动手前扫同类坑）送达通道不存在——播进项目的所有文件（skeleton/lite/phase-card/merge-checklist/track/review prompt）零指向 bench/cases/；17 张卡对下游是不可达资产
- confidence:high
- claim:在播种文件加最小指针（如 BOOTSTRAP §4 铁律区或 phase-card 模板一行：「接外部服务/并行/交付前，扫 agent-on bench/cases 使用时机表」）
- suggested_landing:BOOTSTRAP 或 kit/phase-card-template.md（一行，不加新文件）
- rollback:revert
- trace:批评家漏项①A
- 状态:pending（中风险，选择题）

### probe-from-cases-zero（探针题库零转化）
- source:同上
- evidence:capability-probe 至今只有固定四题；「案例持续转化为新探针题」承诺的转化数=0
- confidence:medium
- claim:首结账后按案例 08/09/10/14 转化 4 道新探针题（时机绑首结账，不提前）
- suggested_landing:bench/capability-probe.md
- rollback:revert
- trace:批评家漏项①B
- 状态:pending（deferred 候选，选择题裁时机）

### skill-routing-slot（GStack 边界从声明变能力）
- source:同上
- evidence:kit/review-prompt-template 是完整对抗审查协议，与用户全局 Skill Routing（审查=GStack /review+Codex）在新项目会撞出双审查制度；Euan 靠手写「Skill 路线」节人肉调和，skeleton 无此槽位，BOOTSTRAP 六问不问机器上已有哪些 skill 体系
- confidence:high
- claim:AGENTS-skeleton 加「本机 skill 路由」槽位（填空：审查/发布/调试各用哪套，agent-on 模板为 fallback）；BOOTSTRAP 需求问句加半句
- suggested_landing:kit/AGENTS-skeleton.md + BOOTSTRAP §1
- rollback:revert
- trace:批评家漏项②
- 状态:pending（中风险，选择题）

### glossary-5-lines（术语黑话）
- source:同上
- evidence:全仓无术语表；pin/fixture/worktree/L0-L4 对非工程师零解释，违反项目自己 CLAUDE.md「术语首现给人话」铁律
- confidence:medium
- claim:README 末尾加 5 行「一句话术语」
- suggested_landing:README
- rollback:revert
- trace:DX gap#3
- 状态:pending（低风险，消化直落）

### kit-readme-scope（kit/README 陈旧误导）
- source:同上
- evidence:kit/README「启动步骤(新项目第一小时)」实为 M/L 七步却挂通用标题，与 S 档三件套冲突；仍标 v0.1、含「agent-on/(或指回本仓)」倒仓前旧写法
- confidence:high
- claim:标题限定「M/L 启动步骤」+ 加一行「S 档看 BOOTSTRAP §2 捷径」+ 删旧句
- suggested_landing:kit/README.md
- rollback:revert
- trace:DX gap#4
- 状态:pending（低风险，消化直落）

### remote-backup-note（单机单点）
- source:同上
- evidence:真相源全在本地 git，异地副本=GitHub 远端（已 push），但 README 无「换机三步」，恢复路径靠人肉记忆
- confidence:medium
- claim:README FAQ 补「换机三步」（clone 仓 / symlink skill / CLAUDE.md 路由随 claude-memory 自带）
- suggested_landing:README FAQ
- rollback:revert
- trace:风险 gap#5
- 状态:pending（低风险，消化直落）

### semver-clash（争议卡：v0.2.0 后 10 commit 未打 tag）
- source:同上
- evidence:CHANGELOG 未发布段积压（斜杠命令/档位路由/编排七件均属 minor=新增篇目），按账本法应打 minor tag；但 v0.3 已被预定为「dogfood 通过」的里程碑语义——直接打 v0.3.0 撞语义，降级 patch 违账本头
- confidence:high（事实确凿，裁决开放）
- claim:消化选择题裁决：A. 打 v0.2.5 之类 minor（里程碑语义与版本号解耦，dogfood 通过改为「标志性事件」而非版本号）B. 维持攒批，首结账消化时一并打
- suggested_landing:CHANGELOG 头部 semver 注 + tag
- rollback:tag 可删
- trace:风险 quickWin#1 + 批评家冲突#2
- 状态:pending（中风险，选择题）

### intake-lint-timing（争议卡：唯一机件的时机对撞）
- source:同上
- evidence:工程镜头主张「首结账后即做 20 行脚本（唯一过得了谁写谁读的机件）」vs CEO 镜头主张「继续 deferred，等 3+ 卡堆积且人眼实证看漏再介入」；Backlog 原文「首次真实结账验收协议后再定」贴近后者
- confidence:medium
- claim:消化选择题二选一；本卡本身就是首批 16 张卡——若消化时人眼核六项确实吃力，即是工程镜头胜出的实证
- suggested_landing:CHANGELOG Backlog 条目改写
- rollback:—
- trace:批评家冲突#1
- 状态:pending（争议，选择题）

### slash-command-ui-quirk（用户实测：斜杠命令部分界面不识别）
- source:用户实测 2026-07-09 @ 当前会话
- evidence:用户在 Claude Code 某界面输入 /agent-on 得「isn't a recognized command here」，同时 skill 实际已加载（正文被注入）——识别依赖界面与输入方式（须补全菜单选中）
- confidence:high
- claim:中文口令保持主路（已通）；README FAQ 或 skill description 加一句「斜杠不识别时直接说中文口令」
- suggested_landing:README FAQ 一句
- rollback:revert
- trace:用户原话「显示有这个啊但没反应」
- 状态:pending（低风险，消化直落）
