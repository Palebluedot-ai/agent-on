# 融合地图 — agent-on v0.2（2026-07-07）

> 本文件职责：记录三个前身仓 + Euan-Flutter 实战沉淀合流进 agent-on 的完整决策——什么进、什么不进、为什么、按什么顺序执行。本文件自包含，不需要先读其他文档。
> 依据：两份深读评估（agent-orchestration-playbook / communication-governance-playbook 全仓通读）+ Euan-Flutter 仓内 agent-on/ 现状核对。

## 1. 为什么融合

agent-on 的定位（2026-07-01 tool-definition snapshot 已拍板）：新项目脚手架 + Loop Engineering 可复用方法论；交付物 = 可 clone 的 starter 包 + 协议文档；不做重编排框架；不与 GStack / Superpowers 冲突。

现状三个问题：
- 精华长在 Euan-Flutter 仓内 `agent-on/`（十篇方法论 + kit 十件套 + run 数据），独立仓断供（最后提交 07-01），要 clone 的仓里没有货
- 三个前身仓各有 15–30% 仍然成立的资产，散落未收编，且和本仓 5 月内容存在两代权威打架
- 用户即将开新项目，需要「零铺垫可用」的启动入口

## 2. 完整家谱（2026-07-07 全景扫描定稿）

```
2026-04-25  evolving-memory-system          支线·活仓（便携记忆系统，Hermes/OpenViking 时代）
2026-04-27  agent-orchestration-playbook    一代主线（Harness Engineering，自建编排运行时）
                 ↑ evolving-memory 的 ve-governance-adapter pin 它为治理上游（commit 8f175b4）
2026-04-30  skill-asset-governance          旁支（Hermes 技能资产治理：registry/冲突检测/生命周期）
2026-05-02  non-drift-communication-protocol  二代起点（锁协议）
     └─(同一条 git 历史直接长出;2026-07-08 push 时证实 GitHub 仓即同一个——
        non-drift 远程被改名为 communication-governance-playbook,旧 URL 重定向,
        本地 ~/Projects/non-drift-communication-protocol 只是改名前的旧 checkout)→
2026-05-24  communication-governance-playbook 二代主体（扩建 + 迁入 non-drift 与 evolving-memory 各两份协议）
2026-05-24  Agent-On 仓建立                  二代半（仓内自称 project-kickoff-os——它不是独立文件夹，
                                              就是本仓 ~/Projects/Agent-On 的 5 月内容）
2026-06~07  Euan-Flutter 仓内 agent-on/      三代（Loop Engineering 实战，9 次并行 run 验证）
外部参照     lipingtababa/agents-zone-skillset  Harness Engineering Playbook 的参照实现（他人资产，引用不融合）
```

| 代 | 仓 | 核心思想 | 可移植比例（深读裁决） |
|---|---|---|---|
| 一代 | agent-orchestration-playbook | Harness Engineering：自建编排运行时（Router / Lane / Policy 注入） | 25–30%（契约 schema、DoD、隔离原则活了；引擎蓝图死了，phase0 实现目录全空） |
| 二代 | non-drift → communication-governance-playbook（同一 git 线） | 沟通治理：用锁和状态机堵漂移 | 15–20%（真相源分层、三态转移活了；锁口令体系被「对齐」取代。non-drift 两份协议已被迁入承接，不需单独处理） |
| 二代半 | project-kickoff-os（= 本仓 Agent-On 的 5 月内容） | 文档驱动的项目启动 OS | ~70%（愿景、skill 拆解图、三模板直接收编） |
| 支线 | evolving-memory-system `04_protocols/` | 记忆系统的方法论协议层 | 只挖协议不动仓（活项目）：记忆分层 / 多轨决策 / 修正闭环三套思维模型 + run 台账落盘规范可移植；绑定 Hermes/OpenViking/自建 runner 的运维件不移植 |
| 旁支 | skill-asset-governance | Hermes 技能资产治理 | 暂不移植；若 v0.3 决定走 skill 形态，其 registry/生命周期/冲突策略再回来挖 |
| 三代 | Euan-Flutter `agent-on/` + `docs/snapshot/` | Loop Engineering 实战：契约先行并行 / 单写者 / 自包含卡 / 防幻觉 | 100%（就是主体） |

一条演化主线贯穿三代：**从「锁住 AI」到「和 AI 对齐」**——弱模型时代靠口令闸门和自建引擎兜底，强模型时代靠前置追问、契约冻结、单写者、对抗审查从源头消解。这条主线本身就是产品故事，写进 README。

## 3. 融合原则（五条）

1. **旧仓原地归档，不删不搬家**：三个前身仓保持只读，各加一段 README 标头指向本仓；agent-on 只吸收**提炼后的内容**，不堆原文（旧仓 = 证据层，本仓 = canonical——这正是二代仓 source-of-truth-map 自己的用法）。
2. **单一权威 = 本仓**：融合完成后，新项目只从 agent-on 取，不再翻任何前身仓或 Euan-Flutter。
3. **Euan-Flutter 仓内 `agent-on/` 继续当采集端**：Euan 还在开发，沉淀照旧写仓内，里程碑「结账」单向同步进本仓；不搞 submodule。
4. **先文档形态，后自动化**：skill 化 / 脚本化等 dogfood 一两个新项目后再定（本仓 AGENTS.md 原则：先固化规则，再抽象模板，再考虑自动化）。
5. **自举**：本仓自己的开发遵守自己的方法论（卡片化推进、完成贴证据、快照沉淀）。

## 4. 目标结构（v0.2）

```
agent-on/
├── README.md          # 产品自述：给谁用/解决什么/三代演化故事/快速开始
├── AGENTS.md          # 本仓开发纪律（自举，收编二代半文档纪律六条）
├── BOOTSTRAP.md       # ★Boot 入口：新项目第一条 prompt 只说「读这份并按它初始化」
├── boot/              # 冷启动配件
│   ├── new-project-questionnaire.md   # 新场景六步问卷（源：一代 onboarding template，去平台化）
│   └── session-handshake.md           # 会话续接握手（源：二代 handshake，防跨会话漂移）
├── kit/               # 模板层
│   ├── （三代 kit 十件原样迁入）
│   ├── prd-template.md / requirement-pack-template.md / milestone-template.md（二代半收编）
│   ├── merge-checklist.md             # 增强：并入一代 DoD 四条 + 最小卡集门禁
│   └── schemas/                       # task/result/audit-event JSON Schema（源：一代，去 Discord 字段，换 worktree/run_id）
├── playbook/          # 方法论层
│   ├── （三代十篇原样迁入：sop/防幻觉/二车道/模型无关化/混编经济学/多人协作/架构师透镜/前置追问 等）
│   ├── truth-hierarchy.md   # 新：真相源三层分层 + 冲突仲裁（源：二代 source-of-truth-map）
│   ├── phase-gates.md       # 新：阶段闸门三态语义 + block 理由码 + 冻结解锁协议（源：二代状态机，重写为 Claude Code 语境）
│   └── meta-principles.md   # 新：方法论元原则（self-dogfooding / 治理进 Git 不进聊天 / 初心引言）
├── bench/             # 能力探针 + 翻车案例集
│   ├── capability-probe.md            # 自 kit 迁入
│   ├── truth-table-template.md        # 动工前能力真相表（源：二代 truth table，防「假装已实现」）
│   └── cases/                         # 案例集：Euan loop-notes 结构化 + 两个前身仓弯路标本
├── ledger/            # run 数据层
│   ├── run-ledger-template.md         # 自 kit 迁入，加成本列
│   └── audit-lint.py                  # 状态机迁移校验（源：一代 execution_audit_lint_v1.py 改字段）
├── snapshot/          # 本仓快照（已有 8 篇 + 本文件）
└── legacy/            # 考古层：三代演化史笔记 + 前身仓指针（不搬全文）
```

## 5. 移植清单（精选件逐项）

### 来自一代（agent-orchestration-playbook）
| 移植物 | 源文件 | 去处 |
|---|---|---|
| task/result/memory/audit-event 四个 JSON Schema（含条件校验 + fencing 锁语义） | `04_contracts/*.json` | kit/schemas/ |
| DoD 四条（可编排/可审计/可追责/可回放）+「缺卡不得 done」 | `05_playbooks/execution-layer-dod-v1.md` | kit/merge-checklist.md |
| 「默认隔离、显式共享、单一状态写者」原则原文 | `05_playbooks/context-isolation-and-shared-state-architecture-v1.md` | playbook（给单写者补「为什么」） |
| 状态机迁移校验脚本 | `scripts/execution_audit_lint_v1.py` | ledger/audit-lint.py |
| 新场景 onboarding 六步问卷 | `05_playbooks/new-scenario-onboarding-template-v1.md` | boot/new-project-questionnaire.md |
| 失败降级 runbook 结构（触发信号+降级动作+恢复条件） | `05_playbooks/failure-drill-degrade-runbook-v1.md` | playbook + bench 案例模板 |
| commit 分层约定（thought/decision/policy/automation） | `05_playbooks/commit-layering-convention-v1.md` | kit |
| 「上下文边界优先；单 agent 能干完就别上多 agent」 | `02_decisions/ADR-003` + subagent 阅读笔记 | playbook（防过度编排） |
| 「不许用『感觉更好』替代不变量」灰度/回测门禁 | `05_playbooks/gray-release-checklist-v1.md` 等 | playbook（Ship 门禁） |
| 初心引言「我们的酷是想法的沉淀，不是 skills 的数量」 | `01_thoughts/2026-04-27-deep-dialogue-notes.md` | playbook 扉页 |

### 来自二代（communication-governance-playbook）
| 移植物 | 源文件 | 去处 |
|---|---|---|
| 真相源三层（canonical/承接/举证）+ 冲突仲裁优先级 | `docs/source-of-truth-map.md` | playbook/truth-hierarchy.md |
| 三态转移语义（申请切阶段 ≠ 切阶段）+ 6 个 block 理由码 | `docs/conversation-state-machine-v1.md` | playbook/phase-gates.md + kit 审查词表 |
| 冻结解锁两步协议（口令 + Next-ID，fail-closed） | `01_protocol/boundary-confirmation-v1.md` | playbook/phase-gates.md |
| Session Start Handshake（跨会话续接三步握手） | `imported/evolving-memory-system/04_protocols/global-inscope-lock-protocol-v1.md` §3 | boot/session-handshake.md |
| Truth Table（动工前能力真相表：已自动化/手动/未实现/依赖环境） | 同上 §4 | bench/truth-table-template.md |
| self-dogfooding 硬约束 + 「治理契约进 Git 不进聊天记忆」 | `docs/mrd.md` §13、`global-inscope-lock-protocol-v1.md` §7 | playbook/meta-principles.md |

### 来自二代半（project-kickoff-os，本仓现有）
| 移植物 | 去处 |
|---|---|
| 愿景 / skill 拆解图（project-bootstrap 等五个）/ 三阶段路线 | README 演化史 + legacy/（skill 拆解图在 dogfood 后决定是否落地为真 skill） |
| PRD / 需求澄清包 / milestone 三模板 | kit/ |
| 文档纪律六条 | AGENTS.md |

### 来自支线（evolving-memory-system `04_protocols/`，2026-07-07 补录）
| 移植物 | 源文件 | 去处 |
|---|---|---|
| Soul Memory L0–L4 五层记忆分层（Operational/Episodic/Semantic/Procedural/Identity + 「跨任务可复用才升层、高频失败优先沉淀为 anti-pattern」判据） | `04_protocols/agent-orchestration-classification-v1.md` §4 | playbook（给五块资产提供「什么沉淀到哪」的判据） |
| ABDC 多轨并行决策方法论（decision-brief → A/B/C 三轨论证 → 加权评审含**强制异议记录** → 单假设实验卡：≤5 天、Pass/Fail/Inconclusive 三态 + 熔断回滚） | `planning/abdc/` + `templates/` 四模板 | playbook（可审计选型子方法论）+ kit（四模板） |
| 修正闭环三态风险分级（低风险自动改 / 中高风险进人工队列，队列项带 risk/decision_reason/状态机 + operator 四动作） | `04_protocols/correction-automation-v1.8.md` + `memory-control-api-v1.7.md`（抽思想不抄脚本） | bench（翻车案例的下游：怎么系统性修掉） |
| run 台账 jsonl 落盘规范（`<run_id>.jsonl` 一行一卡、task→result→memory 强制顺序、durable 必写 memory_card）+ memory_card schema（evidence/confidence/memory_type 枚举） | `04_protocols/run-card-logging-v1.md` + `agent-cards-schema-v1.json` | ledger（落盘规范）+ kit/schemas/（memory_card 补一代三卡之缺） |
| Handoff 最小上下文包 6 字段（objective/scope_in/scope_out/constraints/acceptance_criteria/inputs+expected_output_format） | `agent-team-orchestration-v1.md` §4 | kit（phase 卡字段级 checklist，与现有卡模板合并） |
| 反思产出协议升级机制（每周反思必须产出 1 条策略升级，写入 protocol-changelog——方法论自身可版本化迭代） | `04_protocols/reflection-loop.md` §4 | playbook/meta-principles.md（Bench 案例回流 Playbook 的机制） |
| 「下游只映射不复制 + pin commit + Promotion Card」反双写漂移模式 | `04_protocols/ve-governance-adapter-v1.md` | playbook/truth-hierarchy.md（agent-on 被下游项目引用时的治理规范） |

不移植（支线判死部分）：Hermes 编排中枢二分架构（6 角色跨进程分工）、OpenViking 索引与同步补偿三件套、自建 runner 全家（entrypoint/healthcheck/continuous-runner/压测回归）、记忆库自身家务（capture/distill/inbox/weekly-audit）、repo-guard 桌面防误删、toolchain 版本基线。

### 来自三代（Euan-Flutter）
十篇方法论 + kit 十件套 + `docs/snapshot/` 7 篇机制文档，整体迁入；kit 的去 Euan 化（api/app 目录假设、Vercel/Supabase 语境）留到批五 dogfood 时做，避免闭门造车。

## 6. 死亡名单（明确不移植）

- **一代**：自建运行时全部蓝图（execution-governance-architecture / phase0-implementation-steps / Router 职责 / Lane 抽象）、多平台消息路由（Discord channel-repo / 跨平台 profile / thread 治理）、全部产品 roadmap（four-week / master-roadmap / screening-sheet 等）、自动生成 reports、genetic loop 脚本。
- **二代**：锁口令体系作为日常机制（被「前置追问 + 对齐」取代，仅在 phase-gates 保留 fail-closed 思想）、MRD/PRD 实例、技术选型记录、Hermes 会话 JSON（原地留作证据）。
- 一句话理由：它们描述的是**要造的引擎**或**已废的路线**，不是**要遵守的纪律**。

## 7. 前身仓的 Bench 标本价值（批三入册)

1. **一代标本**：「自建编排层」整仓 5,200 行文档 + 0 行落地代码——工具能力已覆盖时，造引擎不如立纪律。
2. **二代标本**：防漂移框架自己漂进文档洁癖——14 篇 canonical 文档、MRD 永远 READY_FOR_REVIEW、零实现；且 `01_protocol/` 与 `imported/` 逐字节重复，违反了自己定的「不许重复真相源」。教训：治理文档 ≠ 治理执行；过度前置治理会让主线空转。
3. **non-drift 微标本**：git 历史里「feat: add executable non-drift protocol baseline」提交后立即 Revert——可执行实现被回退、纯文档协议活下来，又一个「引擎死、纪律活」的数据点。
4. **共同正资产**：「漂移是核心敌人」的问题定义、三态转移语义、真相源分层——问题定义领先于工具能力，方向从来是对的。

## 8. 执行批次

| 批 | 内容 | 产出 |
|---|---|---|
| 批一 | 骨架 + 搬运：建 v0.2 目录，迁三代资产，收编二代半，写 README / AGENTS / BOOTSTRAP | 独立仓「今天可用」 |
| 批二 | 精选移植：§5 一代/二代/支线清单逐项重写为 agent-on 语言 | 三篇新 playbook + boot 两件 + schemas（含 memory_card）+ audit-lint + ABDC 四模板 + 记忆分层/修正闭环 |
| 批三 | Bench 起步：案例集结构化（§7 两个标本 + Euan loop-notes 案例） | bench/cases/ |
| 批四 | 收尾：用户全局 CLAUDE.md 加路由一行、三个旧仓加归档标头、push GitHub（需用户确认） | 对外可 clone |
| 批五 | 新项目 dogfood，摩擦记回 loop-notes → kit 去 Euan 化 | v0.3 |

每批完成停下来讲解（项目纪律）。

## 9. 待拍板（用户件）

1. **语言**：README / BOOTSTRAP 纯中文（自用优先）还是中英双份（对外产品化）？——建议先纯中文，dogfood 后再双语。
2. **命名收口**：GitHub 仓已叫 agent-on，但仓内 README 还自称 project-kickoff-os——批一统一为 agent-on（确认无异议即执行）。
3. **要不要先跑 /autoplan 过产品定位**？——建议不跑：定位 07-01 已拍板，融合是执行不是设计；dogfood 后做 v0.3 时再上四声音评审。
