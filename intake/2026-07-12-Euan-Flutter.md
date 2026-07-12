# intake · 2026-07-12 · Euan-Flutter(第二次结账)

> 锚:上次回执 = loop-notes「2026-07-09 · ⇄ agent-on 结账回执」(intake 345bad6);本批 = 锚后全部新条目(07-11 两条 + 07-12 五条)。
> 项目无 agent-on.lock.md(倒仓项目,沿用首结账口径);pin 未锚。

### design-handoff-as-file(分享链接不是交接物)
- source:Euan-Flutter @ 72760b6 | pin 未锚(倒仓)
- evidence:DesignSync 404 / WebFetch 403 / Chrome 扩展未连三路全断的会话实录;文件到手后 72760b6 入仓
- confidence:medium(一次事故,但机理普适:链接依赖登录态与项目存续)
- claim:收设计稿=收文件落仓(design/incoming/),链接只当出处引用;交接清单写明「不收 URL」
- suggested_landing:playbook/sop.md 交接/集成清单一行;或 kit/AGENTS-skeleton 协作节
- rollback:revert 落地 commit
- trace:loop-notes「2026-07-11 · 设计交接的两条前端教训」第 1 条
- 状态:pending

### stale-codegen-double-false-negative(双假阴性:测试替过期生成物作伪证)
- source:Euan-Flutter @ b83be35 | pin 未锚
- evidence:design_tokens.json 已 #8248F3 而生成 dart 仍 #6366F1,且旧测试断言旧色=全绿;b83be35 修复(TDD 先红后绿)
- confidence:high(机理清晰:生成物+测试同锚旧值则双绿)
- claim:测试期望值锚真相源不锚生成物现状;「重跑生成无 diff」必须真进 CI 而非只写注释
- suggested_landing:playbook 里 codegen/生成物纪律段;kit CI checklist 一行
- rollback:revert
- trace:loop-notes「2026-07-11」第 2 条
- 状态:pending

### editor-memory-truth-layers(Pencil 三层真相:内存>磁盘>快照)
- source:Euan-Flutter @ 3fd26e8 | pin 未锚
- evidence:磁盘 pen 滞后编辑器内存 1.5h+(mtime 实测);渲染管线对新节点空白而 batch_get 数据一直正确;3fd26e8/6c70b9e 两次落盘收账
- confidence:high(同会话三次复现)
- claim:MCP 编辑器类工具改的是内存——commit 工件前查 mtime,过期先要一次用户保存;验证走数据模型,渲染空白≠稿坏
- suggested_landing:playbook 工具纪律篇(或 bench 案例);同类工具(Figma/Pencil/任何带编辑器的 MCP)通用
- rollback:revert
- trace:loop-notes「2026-07-12 · Pencil 三层真相纪律」
- 状态:pending

### sed-charclass-disaster(结构化文件禁用 shell 字面量替换)
- source:Euan-Flutter @ cc1b6d4 前后 | pin 未锚
- evidence:sed 's|[中文长字面量]|…|' 把 dashboard.html 每个匹配字符替换成整句,整页炸毁;git checkout 恢复+Edit 重做的会话实录
- confidence:high(机理确定:[] 是字符类)
- claim:改结构化文件一律用编辑工具精确锚点;shell 替换只用于自己刚生成的无特殊字符内容
- suggested_landing:kit 安全纪律 checklist(与 careful/guard 类规则同层);bench 案例
- rollback:revert
- trace:loop-notes「2026-07-12 · sed 字符类事故」
- 状态:pending

### worktree-sprawl-patrol(合流必带 worktree 巡检)
- source:Euan-Flutter @ 730a8e0 | pin 未锚
- evidence:730a8e0 一次清 7 个(6 死枝+1 孤儿收编+1 空壳);用户原话「很多窗口爬乱了」
- confidence:medium(一个项目实证,机理普适:会话生命周期与 worktree 生命周期脱钩)
- claim:任何合流/结账动作附带 git worktree list 巡检;拆前三查(脏/独有 commit/ancestor);孤儿 commit 先收编再拆
- suggested_landing:playbook/sop.md 合流节 + session-handshake 读取表加一行(与「待消化」同层的例行巡检)
- rollback:revert
- trace:loop-notes「2026-07-12 · worktree 蔓延」
- 状态:pending

### exhaustive-extraction-adversarial-verify(穷尽留存=分片提取+双路对抗核对)
- source:Euan-Flutter @ 713fef2 | pin 未锚
- evidence:run wf_e830365c(9 agent/62 万 token)产出 558 行状态清单+7 勘误(含渲染层覆盖死代码、有逻辑无 UI 隐藏能力);双核对=源码机械清单对表+用户点名逐项验
- confidence:medium(单次,但结构可复制)
- claim:「每个状态都要留存」类任务:分片并行提取→合并→双路对抗核对(机械清单对表,不是复读)→定稿;核对者必须独立生成对表清单
- suggested_landing:playbook 编排篇(与契约先行双轨并行同层的模式);kit 若有 workflow 模板则加一节
- rollback:revert
- trace:loop-notes「2026-07-12 · 穷尽式留存的多 agent 提取模式」
- 状态:pending

### hard-axis-soft-tags(时间判据拆本体论打架)
- source:Euan-Flutter @ 6c70b9e | pin 未锚
- evidence:两轴语义打架→判据「随时间走=阶段(硬轴),不随时间走=性质(标签)」→用户即拍板并自发修正一档;two-mirrors §10 定案,省一次 type 字段 migration
- confidence:low(单次,产品语境强)
- claim:AI+非技术 owner 的数据建模决策:先给一句话判据再给方案,别直接端 schema;类型类需求优先降级复用标签基建
- suggested_landing:playbook 决策辅助/architect-lens 篇(与「架构雷达」同族:信号→判据→动作)
- rollback:revert
- trace:loop-notes「2026-07-12 · 硬轴/软标签」
- 状态:pending
