# intake · Euan-Flutter · 2026-07-21 结账

> 来源会话:前后端结合线开工(demo v9 锚点固化 + 契约冻结 + i3a 写路径三件套当日收口)。
> 项目分支 claude/frontend-backend-integration-plan-7c9f64(commit 2753267→2e93f8d 五连)。
> 域判据自检:以下全部为 AI 协作过程教训(编排/工具行为/纪律);契约内容、阶段词映射等项目域知识留项目端未装卡。

### demo-anchor-into-repo-before-build(demo 定稿先入仓固化再开工)
- source:Euan-Flutter @ 8374abc | pin v0.3.0
- evidence:commit 8374abc(design/demo-v9/:demo 本体+运行依赖 ios-frame.jsx+23 张逐屏基线截图+README 更新流程);此前 `git status` 显示整个 demo 文件夹 untracked——规格在仓外裸奔一周
- confidence:medium(demo-first 流程第二次验证,锚点固化机制首次)
- claim:Demo-as-spec 的稿子定稿当天必须入仓固化:demo 本体+全部运行依赖+逐屏基线截图+「微调后怎么同步」的更新流程 README,四件齐才算锚点;untracked 的规格 = 会蒸发的锚点,禁止对着仓外文件开工
- suggested_landing:playbook demo-first/设计管线篇加「锚点固化四件套」小节;或 kit phase-card-template 前置检查行
- rollback:revert 落地 commit
- trace:本会话(锚点固化批次);无 loop-notes(项目未启用)
- 状态:landed@同批(第十三次消化:kit/phase-card-template §0 Demo 锚点四件套前置检查)

### serve-dc-html-over-http(.dc.html 必须 HTTP 服务运行)
- source:Euan-Flutter @ 8374abc | pin v0.3.0
- evidence:console 报错原文 `[dc-runtime] x-import: FAILED to load ./ios-frame.jsx (jsx) TypeError: Failed to fetch` + `URL scheme "file" is not supported`;修复后 23 屏像素级完好(8374abc 截图为证);坑已档案化进 design/demo-v9/README.md
- confidence:high(可复现:file:// 必崩,http:// 必好)
- claim:Claude Design 导出的 .dc.html 用 file:// 直接打开会静默断掉 fetch 与 x-import 的本地依赖(如设备外框组件),排版崩但页面不白屏,极具迷惑性;必须起本地 HTTP 服务跑,排查首看 console 的 x-import FAILED
- suggested_landing:bench 案例一张(工具行为坑);或 sop 外部工具篇一行
- rollback:revert 落地 commit
- trace:本会话(基线截图批次,第一张截图缺字错位现场)
- 状态:landed@同批(第十三次消化:bench/cases/25 与 react-fiber 卡语义归并)

### disjoint-file-ownership-parallel-implementers(并行 implementer 文件域互斥 + 共享文件必须指派 owner)
- source:Euan-Flutter @ ca2e025 | pin v0.3.0
- evidence:workflow wf_72f612c3(三 implementer 同 worktree 并行 TDD,36 测试红→绿,722→792,零互踩);spec review Medium 发现:卡上「api-contract.md 各补一行」条款三人各自以避撞车为由回避 → 条款净未满足,收口人补行(ca2e025)
- confidence:medium(一次编排实证,教训清晰)
- claim:同 worktree 并行 implementer 必须在派工时显式划互斥文件域;跨切片共享文件(契约表/索引/清单类)必须指派唯一 owner 或明写「留收口人」,否则每个并行体都会理性回避共享文件,卡上条款集体落空——回避是对的,漏派才是错的
- suggested_landing:playbook 多 agent 编排篇;kit phase-card-template 加「共享文件 owner」一行
- rollback:revert 落地 commit
- trace:wf_72f612c3-21d(journal 三 agent 自述均含「未碰共享文件」段)
- 状态:pending

### react-fiber-logic-driving-for-dc-demo(无头驱动 .dc.html 逐屏截基线)
- source:Euan-Flutter @ 8374abc | pin v0.3.0
- evidence:23 张基线截图入仓(8374abc);方法档案化两处(design/demo-v9/README.md「截图怎么再生」+ gstack learnings dc-html-demo-driving)
- confidence:low(单项目单次,依赖 dc-runtime 内部结构)
- claim:.dc.html 交互原型逐屏截基线不靠逐屏点击:React fiber 从任意 DOM 节点走 return 链找到含 .logic 的 stateNode 拿逻辑实例,go(screen,extra)/setState 直切任意屏(含认证态/子页),对固定尺寸设备框元素做 element screenshot——可脚本化、可重跑、demo 微调后一键再生
- suggested_landing:bench 案例(与 serve-dc-html-over-http 同来源可合并成一张「dc.html 驱动」案例)
- rollback:revert 落地 commit
- trace:本会话(基线截图批次)
- 状态:landed@同批(第十三次消化:bench/cases/25 与 serve-dc-html 卡语义归并)

### chunked-readers-gap-matrix-workflow(大单文件规格的并行深读→差距矩阵编排)
- source:Euan-Flutter @ 2753267 | pin v0.3.0
- evidence:workflow wf_8f5a3d86(10 agent:474KB 单文件 demo 按行域切 5 段并行深读 + 现状代码盘点 2 路 + specs/版本考古 2 路 + 1 综合)产出 28 屏逐屏差距矩阵+后端缺口 20 条,入仓 docs/frontend-backend-integration-plan.md(2753267);后续契约(docs/backend-contract-v9.md)与 phase 卡直接引用矩阵为施工口径
- confidence:medium(一次编排,产物被下游持续消费即为验证)
- claim:数百 KB 的单文件规格(demo/大 spec)深读编排:按结构行域切段给并行 readers(每段结构化 schema 收口),另派现状盘点 agents(代码/API/schema 面),末位一个综合 agent 对撞出「逐屏(逐模块)差距矩阵」;矩阵先入仓,规划/契约/phase 卡全部引用矩阵而非重读原文——盘点一次,消费多次
- suggested_landing:playbook 多 agent 编排篇(与 review-army/深读模式并列)
- rollback:revert 落地 commit
- trace:wf_8f5a3d86-fab(journal 含各 reader 返回值)
- 状态:pending
