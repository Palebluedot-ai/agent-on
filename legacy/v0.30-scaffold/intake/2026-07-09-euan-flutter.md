# Intake — Euan-Flutter 首次结账(2026-07-09)

> 倒仓首结账(agent-on v0.3 门槛件)。项目 `agent-on/` 采集端无 lock(project-kickoff-os 期播种,早于 v0.2 产品化),按全量扫描处理,以本文件为未来 last_settlement 锚点。
> source 仓:Euan-Flutter @ e863967 | pin:无(倒仓,pre-v0.2 seed)
> 素材面:agent-on/loop-notes.md(13 条)+ agent-on/run-log.md(Run #1–9)。

## 已消化免出仓(消化会话不必重判,仅备查)

结账时逐条对照 agent-on canonical,以下已在正文,**不再出卡**:

- 六步协议(契约先行双轨并行)= sop.md Phase 5 全文 ← loop-notes 契约先行条 + Run #1–9 数据
- fixture 冻结写死语义 / 进壳测试钉全量 Fake = sop Phase 5 合流 checklist ← Run #3
- 本地模拟器≠生产 / 分层冒烟 / GET+POST 各打一发 / 适配器=分叉源 = sop 外部服务清单 #1 ← vercel dev 三连坑 + 适配器 504 + @vercel/node ESM
- .env 行内注释 / 故障进程自曝 env = 清单 #2 ← env 行内注释条
- LIVE 先 dump 真实载荷对账 / 时间戳透传 DB 原生串 = 清单 #3 ← basil 字段搬家 + PostgREST 微秒窗
- 函数区=数据区(地域对齐)= 清单 #4 ← 性能日三连①
- 缓存链逐环查稳定性 = 清单 #5 ← 性能日三连③(B5×B6 咬合)
- CLI 假定自动 yes / 声明式全量声明 / 配置后跑行为探测 = 清单 #6 ← echo n 不是护栏
- 偏好缺口=幻觉第五类 / 前置追问协议 = elicitation-protocol.md ← 官网返工复盘
- 挂账文化(⏸+事后验证,严禁伪造)= sop Phase 6 ← Run #5 断点续跑诚实度

---

## 出仓卡(6 张,genuine delta)

### architecture-radar(架构雷达:非技术 owner 的 unknown-unknowns 解法)
- source:Euan-Flutter @ e863967 | pin 无(倒仓)
- evidence:`docs/architecture-radar.md`(110 行,实体已在项目内运行);loop-notes.md:64–68
- confidence:medium(单项目,但 agent-on CHANGELOG Backlog 已独立标记「架构雷达机制移植」= 第二次背书)
- claim:AI + 非技术决策者的组合,必须制度化一台「架构雷达」——用「信号→动作→量级」代替「建议堆」:每维度记(我们在哪/主流在哪/什么信号该动/动的成本)+ 地雷清单(踩了才响的非代码雷)+ 触发式重构预案 + 里程碑必扫节律。非技术 owner 不需理解方案,只需认得信号、批预算。
- suggested_landing:playbook 新篇 `architecture-radar.md`(正文)+ kit 里加一份 `architecture-radar-template.md` 或并入 milestone 模板(L3 双落点);BOOTSTRAP M/L 档种入
- rollback:revert 落地 commit;模板未被任何项目引用即可删
- trace:loop-notes.md:64–68(2026-07-04 条)
- 状态:landed@d42344a（并入 playbook/architect-lens.md 附节，冻结令期不新开篇；悬空链顺手修复）  <!-- 消化后原地改 landed@<commit> / rejected / deferred -->

### scaffold-not-design(骨架期给用户打「这是线框不是设计」预防针)
- source:Euan-Flutter @ e863967 | pin 无
- evidence:loop-notes.md:7–9(S1 真机首见「UI 和想象完全不一样,不好看」,根因=S0.3 跳过 Pencil 稿先行)
- confidence:low(单项目一次)
- claim:骨架/线框期首次给用户看之前,显式打预防针「这是结构线框,不是视觉设计,丑是预期」;否则第一眼失望是必然,且易被误读为「做砸了」。核心屏进入视觉工序前先出设计稿(Pencil/mockup)再写码。
- suggested_landing:sop.md Phase 3 或 Phase 6 加一行「首屏展示前的期望管理」;或 elicitation-protocol.md 邻近(都是「预期对齐」类)
- rollback:revert 落地 commit(仅一行/一段)
- trace:loop-notes.md:7–9(2026-07-03 用户设计反馈条)
- 状态:rejected（已在 canonical：bench/cases/02-wireframe-expectation.md 同规则，「设计稿先行」半句已在 kit/README 品味前置步 + playbook/freedom-vs-discipline.md §三——结账对照清单漏扫本条；无新增量）

### readonly-guardrails(「绝不能写」场景的三层护栏模式)
- source:Euan-Flutter @ e863967 | pin 无
- evidence:run-log.md:57(Run #6 教训②);迁移轨=只读事务+SQL 白名单+静态扫描断言,零误写
- confidence:low(单项目一次,但模式清晰可移植)
- claim:任何「绝对不能写/不能删」的操作(数据迁移读侧、审计、只读分析),用三层护栏而非单点信任:①运行时(只读事务/只读连接)②输入面(SQL/操作白名单)③静态面(扫描断言禁写关键字)。三层任一失守另两层兜底。
- suggested_landing:sop.md Phase 6 邻近,或 anti-hallucination.md(「危险操作的纵深防御」);可做 bench 案例卡
- rollback:revert 落地 commit
- trace:run-log.md:50–58(Run #6)
- 状态:landed@d42344a（bench/cases/18；单次教训落案例集而非 sop 正文，符合 memory-layering）

### destructive-api-protection(危险端点:先定「什么绝不能删」+ 终态核查脚本)
- source:Euan-Flutter @ e863967 | pin 无
- evidence:`api/src/routes/me.ts`(DEV 账号 403 绝对保护);`api/test/account-delete.test.ts` + `account-delete.live.test.ts`;run-log.md:77(Run #8 教训)
- confidence:low(单项目一次)
- claim:设计删除/清号类 API,先想「什么绝不能删」(护栏端点,如 DEV/系统账号一律 403)再想「删什么」;验收靠**终态核查脚本**(残留 NONE + 受保护对象 intact),不信过程 console——审计操作的日志会被输出链吞掉。
- suggested_landing:sop.md 外部服务清单邻近加一条「危险端点设计」,或 bench 案例卡;与 careful/guard 类安全 skill 分工说明
- rollback:revert 落地 commit
- trace:run-log.md:70–77(Run #8)
- 状态:landed@d42344a（bench/cases/19）

### parallel-live-as-discovery(并行 LIVE 是发现器,不只是验证器)
- source:Euan-Flutter @ e863967 | pin 无
- evidence:run-log.md:39(Run #4 GoTrue global-logout 401 真分支)、:57(Run #6 微秒窗)、:76(Run #8 清号回放)——**项目内 3 次复现**
- confidence:medium(单项目但 3 次独立复现,频次信号强)
- claim:双轨并行时让每轨各自跑 LIVE,不要只当「验证已知」跑——并行 LIVE 本身会撞出串行/单测永远遇不到的真分支(权限边界、精度窗、回放态)。把「并行轨各跑 LIVE」写进合流前置,当发现手段而非事后确认。
- suggested_landing:sop.md Phase 5 合流 checklist 加一行,或 playbook/workflow-orchestration.md 对抗 stage 邻近
- rollback:revert 落地 commit(一行 checklist)
- trace:run-log.md Run #4/#6/#8
- 状态:landed@d42344a（3x复现→升 L3：kit/merge-checklist step5）

### eval-goldset-honesty(评测集:瓶颈是数据不是工具,金标 n=1 要写明不支撑调优)
- source:Euan-Flutter @ e863967 | pin 无
- evidence:`api/eval/BASELINE.md`、`api/eval/golden/`、`docs/phases/phase-s12.1-eval-and-fixes.md`;run-log.md:82–83(Run #9)
- confidence:low(单项目一次)
- claim:AI 产品的评测集,先建 harness 立口径是对的,但金标数据的瓶颈要诚实标注——n=1 的基线只能立口径、**不支撑调优**,告警里写死这句;真正的金标池要等产品被真实使用(「确认闭环白送评测资产」)才兑现,别假装数据够了。适用「验证后才说完成」到评测域。
- suggested_landing:sop.md Phase 6 QA 邻近加一段「评测集的诚实」,或 anti-hallucination.md(数据不足=幻觉温床)
- rollback:revert 落地 commit
- trace:run-log.md:79–83(Run #9)
- 状态:landed@d42344a（bench/cases/20）
