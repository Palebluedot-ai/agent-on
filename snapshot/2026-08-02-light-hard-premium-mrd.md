# Snapshot — 2026-08-02 · 轻主路径 + 硬闸门 + 可降保费（MRD）

> **职责边界**：本文件是 **MRD 级**需求澄清与建设范围拍板用文档——回答「大模型已很强，agent-on 还做什么 / 砍什么 / 怎么更好发挥」。  
> **不是** PRD、不是技术方案、不是 phase 卡、不是实现清单。  
> 体例参照 [2026-07-15-v05-plugin-scoping.md](2026-07-15-v05-plugin-scoping.md)（调研/MRD 自举）。  
> 证据输入：本会话 deep-research `wf_019fc30106f67180b98e252744004c3f`（Partial）；[playbook/orchestration-future.md](../playbook/orchestration-future.md)；[CHARTER.md](../CHARTER.md)；[BOOTSTRAP.md](../BOOTSTRAP.md) §1 / §1.5 / skill 尾注；[playbook/model-playbook.md](../playbook/model-playbook.md)。  
> 用户口语「支付这一块」在本 MRD 中按已对齐方向理解：**强化「轻主路径 + 硬闸门 + 可降保费」的产品交付效果**，与支付业务无关。

## 状态一句话

**岔路 B1 + C1 已拍板并落地（2026-08-02 用户）**：制度优先、Superpowers 退出默认；agent-on 开箱模板 + 本机 `agent-memory` Claude 专属路由一并收。  
产品价值 = **制度层**（证据 / 边界 / 回流 / 不可逆闸）；空转件可见降权；默认心态偏 S；保费用探针调。  
**本文件仍是 MRD 权威取舍记录**——探针刻度统一、L dogfood、打 tag 等其余分期仍属后续；阶段 1 文档/模板与 C1 本机路由已执行，不等于 MRD 全量 P1–P7 发版完成。

---

## 一、做什么 / 给谁 / 为什么现在

### 做什么（产品层升级方向）

在 **不重写编排运行时、不推翻迭代闭环** 的前提下，做一次「别迷恋自己」的产品再定档：

1. **主路径压轻**：默认体验对齐 S 轻装（三件套 + thoughts）；M/L 是显式升档，不是默认堆全套。
2. **闸门留硬**：完成=贴证据、跨仓 git 机械闸、负空间、单一写者、破坏性操作物理闸——与模型档位无关，不可当「仪式」砍掉。
3. **保费可降**：phase 颗粒度 / 审查轮次 / 双轨 / TDD 严格度等按 capability-probe 调，不整包删除七条补齐。
4. **空转件对用户可见降权**：L 档 jsonl 四卡、案例→探针转化空转、二代锁口令复述、「下次顺手」无闸推迟——标旁路/过时/禁用默认，避免新项目踩税。
5. **skill 路由再收敛**：agent-on 侧推荐路径大量压制 Superpowers 式环节 skill；规划/审查/发布优先 GStack（或本机已装的对等强 skill）；agent-on 只做 fallback + 压制清单，**不**与环节 skill 抢活（宪章边界）。

### 给谁

- 第一用户：Chao 自己的后续新项目（dogfood）。
- 产品化后：用 AI 协作开发、非全职工程师背景的 builder——开箱要轻，翻车要有闸，教训要回流。

### 为什么现在

- deep-research（2026-08-02）把「还在扛 / 空转 / 可降保费」拆开了：结账—消化与机械闸是真实闭环；jsonl L 旁路与锁口令复述是税。
- 2025–2026 主流模型 agentic 能力显著加强 → 执行侧「教练型」脚手架边际下降，但**完成幻觉、越界写、状态撞车、禁区蒸发、CLI 自动确认**仍系统性出现（bench 案例与 dogfood）。
- 仓内文档体量大、部分机件零 dogfood → 不先写 MRD 就改 canonical，容易「再堆一层」而非变轻。

---

## 二、产品层取舍表（固化 research，指名资产）

### 2.1 保留 / 加固（资产 + 硬闸；模型越强越值钱）

| 件 / 机制 | 仓内落点 | 产品立场 |
|---|---|---|
| 结账 → intake → 消化闭环 | `boot/settlement.md`、`playbook/iteration-loop.md`、`kit/promotion-card-template.md` | **主干**。S/M 主路径：loop-notes 散文 → Promotion Card 六项齐 → intake；消化单写者改 canonical；收尾必打 annotated tag。 |
| 跨仓 git 机械闸 | `kit/guard/agent-on-git-guard.sh`、`hooks/hooks.json` | **必须保留**。只靠文档自觉已否证；pathspec 误拦已修，继续当物理边界。 |
| 完成 = 贴验证命令实际输出 | BOOTSTRAP L2、`playbook/anti-hallucination.md` | **与模型档位无关的硬门**。防完成幻觉 / 取证幻觉，必须外置。 |
| 单一状态写者 | BOOTSTRAP L3、`kit/progress-template.yaml`、`bench/cases/03-parallel-collision-single-writer.md` | **并发数学问题**，不是弱模型问题。 |
| 契约即文件 | BOOTSTRAP L4、`contracts/` 惯例、`playbook/orchestration-future.md` 三不变量 | 多智能体物理接口；强模型只降低冻结成本，不取消契约。 |
| 负空间（未写明允许即禁止） | BOOTSTRAP L5、phase 卡 disturbance / 暂停项 | 禁区只活在对话里会被自作主张补上。 |
| 破坏性 / 不可逆写的物理闸 | `bench/cases/14-cli-agent-autoconfirm.md`、`19-destructive-api-protection.md`；guard / permissions 类 | 不能赌模型小心或 `echo n`——agent 非交互会系统性自动确认。 |
| 机制须带机械闸 + 接线锚 | multi-contributor / anti-hallucination 相关落地 | 无闸纸面机制静默空转；有闸才能抓完成幻觉与 integration-gap。 |
| pin / 选择性升级 | `kit/agent-on-lock-template.md`、`boot/settlement.md` 升级口令 | patch/minor **只改 lock 一行 pin**；永不从 kit 重拷覆盖存量实例化文件；major 才出迁移 diff 逐条批。 |

### 2.2 降权 / 旁路 / 过时（流程税或零 dogfood）

| 件 / 机制 | 仓内落点 | 产品立场 |
|---|---|---|
| L 档 jsonl 四卡 + audit-lint + 四 schema | `ledger/run-card-logging.md`、`ledger/audit-lint.py`、`kit/schemas/*` | **正式旁路、至今零真实项目跑通**（文内已注明）。S/M 不用 jsonl **不构成退步**。MRD 建议：README/五块资产表对用户 **可见降权**（旁路 / 未验证）；是否「标死废弃」见 §七岔路 A。 |
| 案例 → 探针题库持续转化 | `bench/capability-probe.md`、`intake/2026-07-09-agent-on-self-review.md` | 转化数仍为 **0**（固定四题未扩）。**deferred / 空转**。不阻塞主路径；升 L 或发「探针产品」前须先补转化或诚实写「暂不扩题」。 |
| 二代「每轮 Global/In-Scope/Next 锁口令复述」 | `playbook/phase-gates.md` 历史教训 | **过时形式**。死亡名单只留 fail-closed 思想；主线写进 AGENTS/progress，不靠逐轮复述。推荐路径 **禁止** 复活为默认仪式。 |
| 「下次顺手」式无闸推迟 | `playbook/truth-hierarchy.md`（Dartify 实证） | **绑手又无效**。触发须绑机械信号或日历死线；产品文案默认禁用「顺手」话术。 |
| 默认全开 M/L 重脚手架 | BOOTSTRAP 定档三问 | 拿不准取 **低档**；误播高档后降档只删不用的件、不重播。升级方向 = **默认心态更偏 S**，不是再堆模板。 |

### 2.3 可调保费（旋钮，非整包删）

| 旋钮 | 相关落点 | 调法（产品承诺） |
|---|---|---|
| phase 卡颗粒度 | `kit/phase-card-template.md`、`playbook/model-playbook.md` | 模型更强 → 可放宽（半天→一天→一片）；**探针该环节裸奔一次通过率 >95% 才降一档**（orchestration-future）。 |
| 审查轮次 / 对抗强度 | `kit/review-prompt-template.md`；有 GStack 则 `/review` | 低危抽检、高危必审；**未确认**：无对照实验证明删对抗审查后翻车率不升——故本 MRD **禁止**把「可降」写成「已可删审查」。 |
| 双轨 vs 单轨 | `kit/track-prompt-template.md`、BOOTSTRAP §5 | 单 agent 能干完别上多 agent；强模型可单轨全栈。 |
| TDD 严格度 | BOOTSTRAP L1 | 可按爆炸半径分级（钱/数据全量，UI 抽查）——须写在项目 AGENTS，非全局静默放松。 |
| 悬点裁决上移程度 | model-playbook 七条 | 甲档可放宽颗粒度与抽查；乙档保留编号铁律全文、负空间、裁决上移、每卡独立审查。 |
| 七条补齐整体 | `playbook/model-playbook.md` §二 | **产品化为按执行模型切换的派工旋钮**，不是删除清单。项目 lock 记 model + 保费档位；握手发现模型与 pin 不符 → 提示重跑 probe（`boot/session-handshake.md` 方向）。 |

**刻度映射未确认**：orchestration-future 写「>95% 降一档」，capability-probe 用 4/4 · P1–P4 三档——两套刻度如何一一映射 **未写死**（research 缺口）。PRD 前须二选一或做映射表，本 MRD 不假装已统一。

---

## 三、Superpowers 与 skill 路由（agent-on 产品立场）

> **本 goal / 本 MRD 边界**：只定 **agent-on 仓内推荐与模板侧建议**。  
> **不得**在本 MRD 实现阶段声称已卸载本机 Superpowers；**不得**改 `~/agent-memory` 全局路由——那是用户机器配置，另会话另拍板。

### 3.1 宪章边界（不变）

[CHARTER.md](../CHARTER.md)：**不与 GStack / Superpowers 等强 skill 冲突**——agent-on 管「项目怎么启动和推进」，**不管**「某个环节怎么做」。  
BOOTSTRAP skill 尾注：有强 skill 则审查/发布/调试走强 skill；agent-on 模板 **fallback**；抢跑型 skill 须在 **双工具共读层**（项目 AGENTS.md / 机器侧 AGENT.md）**点名禁用**（AINVESTMENT：superpowers brainstorming 抢跑 init → 骨架零落盘）。

### 3.2 用户方向（本 MRD 采纳 · 2026-08-02 补充）

用户明确两层意思：

1. **大量砍掉 Superpowers 式环节，未必还需要。**  
2. **补充（同日）**：Superpowers **已经比较重、太费劲**，**不倾向再用**；agent-on 这边是按 **支持制度** 来的——要的是轻闸门与闭环，不是再扛一套重环节 skill 栈。

产品解读（对齐上述）：

- 不是「立刻全机卸载插件」，而是 **agent-on 推荐路径默认不再依赖 Superpowers**。  
- 新项目 AGENTS §skill 路由 = **agent-on 制度层（主）+ GStack 环节能力（若已装）**；Superpowers **退出默认推荐**，最多保留「未装任何强 skill 时的可选局部兜底」，且 **不** 写进开箱主路径。  
- 轻重分工一句话：**agent-on = 制度（证据/边界/回流/单写者）**；环节 skill 只在真需要时点名调用；**不再默认叠 Superpowers 全流程税**。

### 3.3 推荐路由矩阵（实现前的立场表）

| 环节 | agent-on 推荐（有 GStack 时） | Superpowers 类 | agent-on 自身 |
|---|---|---|---|
| 冷启动 / 定档 / 骨架 | agent-on BOOTSTRAP | **压制** brainstorming 抢跑 init | **主责** |
| 调研 / MRD / office-hours | GStack `/office-hours`、deep-research 等 | 不作为默认入口 | 只收口产物进 `docs/`（L8） |
| 规划 / autoplan | GStack autoplan 链 | **点名禁用** writing-plans / brainstorming 抢规划 | 不自研提问流 |
| 实现执行 | 主会话/子代理按需；TDD+证据仍是 agent-on 铁律 | subagent-driven-development **不默认**（重、费劲） | 契约 / 单写者 / 完成贴证据 |
| 代码审查（PR 级） | GStack `/review` | **不**进推荐路径 | `kit/review-prompt-template.md` = **无强 skill 时 fallback** |
| 发布 / ship | GStack `/ship` 等（若装） | finishing-a-development-branch **不**进推荐路径 | pin 升级、结账闭环仍是 agent-on |
| 调试 | GStack `/investigate` 等 | systematic-debugging **不默认** | bench 案例扫坑 + L2 证据 |
| QA | GStack `/qa` 等 | 无对等则 fallback 自测+证据 | 完成=输出 |

### 3.4 压制策略（模板侧将要写清的内容——**尚未改文件**）

后续 PRD/实现应保证：

1. **新项目 AGENTS 默认 skill 路由块**：**不写 Superpowers 为推荐**；若装了 GStack → 环节走 GStack；Superpowers 规划/brainstorming/默认执行链 **点名禁用或「不加载」**；审查只保留一套。  
2. **kit/AGENTS-lite 与 skeleton**：路由段默认内嵌「制度在 agent-on；环节 skill 可选；禁止 Superpowers 抢跑」——用户忘写也不会被重栈劫持。  
3. **agent-on 本仓开发纪律**（`AGENTS.md`）：继续「不引入与 GStack/Superpowers 重叠的环节型功能」。  
4. **机器全局**（`~/agent-memory`）：用户已表达 **不倾向再用 Superpowers** → 建议另开短 goal 把 Claude 专属层从「GStack + Superpowers 执行引擎」收到 **GStack（可选）+ agent-on 制度**；**本 MRD 不自动改本机**，避免越权。

### 3.5 明确不做什么（skill 维度）

- 不在 agent-on 内再造一套 brainstorming / writing-plans / ship 运行时。  
- 不把「砍 Superpowers」实现为删除用户插件目录。  
- 不强迫未装 GStack 的机器装 GStack——无强 skill 时走 kit 模板 fallback，开箱仍可用。

---

## 四、可验收产品承诺（拍板后实现阶段的验收靶）

下列为 **目标态承诺**（用户拍板 MRD + 后续实现发版后应可对表）；**不是**「本文件落盘即已达成」。

| # | 承诺 | 可观察判据（实现后） |
|---|---|---|
| P1 | **开箱更轻** | 定档三问拿不准 → 文档与 skill 引导取 S；S 只强制三件套 + thoughts，不诱导播 phase/progress/dashboard。 |
| P2 | **主路径默认轻装** | README / BOOTSTRAP 用户可见路径以 S→升档为主叙事；L 全装与 jsonl 不出现在「五分钟装机」主路径。 |
| P3 | **闸门留硬** | 完成无证据不可报完成；跨仓 commit/push 仍被 guard 拦；暂停项/负空间仍为铁律；破坏性场景仍要求物理闸说明。 |
| P4 | **pin 升级一行** | patch/minor：改 lock pin 行即可；禁止文档教「从 kit 重拷覆盖」。 |
| P5 | **空转件可见降权** | jsonl 四卡 / 锁口令复述 / 「下次顺手」在 README 或资产表有「旁路/过时/禁用默认」标注，新读者 1 分钟内能看到。 |
| P6 | **skill 默认不抢跑** | 新实例化 AGENTS 含 GStack-first + Superpowers 规划类点名禁用（或等价压制句）；fallback 审查仅一套。 |
| P7 | **保费可调且有门槛** | 文档写清：降保费条件（探针/档位）与「三不变量不可降」；禁止「模型新了就全砍审查」。 |

---

## 五、明确不做（非目标）

| 非目标 | 理由 |
|---|---|
| 重写编排运行时 / 新平台 | 宪章：不做引擎；一代教训。 |
| 整包重拷 kit 覆盖下游实例化文件 | 升级语义 = pin；覆盖会毁掉项目本地偏差。 |
| 删除三不变量（契约文件 / 单一写者 / 完成=证据） | 资产不是保费；防并发与自利偏差。 |
| 本 MRD 阶段改 playbook/kit/BOOTSTRAP 行为 | 实现属拍板后；本文件只澄清需求。 |
| 卸载本机 Superpowers / 改 agent-memory | 机器配置另案；agent-on 只定推荐路径。 |
| 把 L 档 jsonl 在无 dogfood 前做成默认必装 | 与「轻」直接冲突。 |
| 伪造 capability-probe 新模型实测分数 | research 标明无 2025–2026 新模型完整跑分回执 → **未确认**。 |
| 消化 intake 打 release tag 作为本 MRD 交付 | 属后续发版流程。 |

---

## 六、建议分期（仅方向，非 PRD 任务拆解）

> 分期供拍板节奏参考；**未授权即不得当实现清单执行**。

| 阶段 | 内容 | 风险 |
|---|---|---|
| **0 · 本文件** | MRD 落盘、用户拍板 §七岔路 | 无实现风险 |
| **1 · 文档可见瘦身** | README 资产表 / BOOTSTRAP 主叙事 / ledger 启用范围横幅对齐「降权」；skill 路由默认块写入 kit 模板；phase-gates/死亡名单交叉链 | 纯文档 + 模板，minor/patch 级 |
| **2 · 定档与升级 DX** | S 默认话术加强；误播高档降档协议补全；lock 记保费档位字段（若缺） | 小改 kit/boot |
| **3 · 探针与保费刻度** | 统一 >95% 与 probe 三档映射；可选：对当前主力模型跑 probe 落档 | 需实测会话；**未确认**前不写死档位数字 |
| **4 · L 机件命运** | 按岔路 A：继续旁路并强化警告 / 或标死 / 或找一个 L 项目 dogfood | 取决于拍板 |

---

## 七、待拍板岔路（≥2）

### 岔路 A · L 档 jsonl 四卡命运

- **A1 · 继续旁路 + 可见降权（倾向）**  
  保持「L 且上 run 台账才启用、未验证」；README/五块表醒目标「旁路」。零 dogfood 不删代码，避免误删后有人真要 L 时无件。  
- **A2 · 标死废弃**  
  文案改为 deprecated，schema/lint 移 `legacy/` 或顶栏大废弃条。更轻、更绝；若未来要机读台账需重建。  
- **A3 · 强制找一个 L dogfood 再定**  
  推迟产品结论，成本高，与「先轻」张力大。

### 岔路 B · 默认 skill 栈

- **B1 · 制度优先 + GStack 可选；Superpowers 退出默认（用户方向已强化 · 倾向）**  
  agent-on = 制度主责；有 GStack 则环节点名走 GStack；**Superpowers 不进开箱推荐、默认点名禁用抢跑类**；未装 GStack → kit fallback。对齐「Superpowers 太重、不倾向再用」。  
- **B2 · 维持现状双栈文档**  
  继续写「GStack 规划审查 + Superpowers 执行」为推荐。与用户最新口语 **冲突**，仅当明确收回时选。  
- **B3 · 极简：两家都可选，agent-on 只写压制抢跑 + 铁律**  
  最少意见；新用户仍可能被本机已装的 Superpowers 抢跑。

### 岔路 C · 是否在实现阶段动机器全局（可选，可延后）

- **C0 · 不动 agent-memory（本 MRD 阶段默认，避免越权）**  
- **C1 · 另开短 goal：全局收掉 Superpowers 依赖**（与用户「不倾向再用」一致时优先）  
  改 Claude 专属 skill 路由 / 文档表述；与 agent-on 仓文档升级可并行、可解耦。

**已拍板（2026-08-02 用户口令「1.2 + C1 一起干」）**：**B1**（制度优先、Superpowers 退出默认）+ **C1**（本机 Claude 专属层路由一并收）。A 未另点名 → 落地时按 **A1**（jsonl 继续旁路 + 可见降权）执行。插件二进制未卸——路由退出 ≠ 卸载。

---

## 八、未确认事项（禁止当成已定事实）

以下直接来自 research **Coverage and uncertainty**，实现前须保持诚实：

1. **非每个 S 档项目**都有公开稳定结账数字；闭环证据集中在 Euan / IPONews / AInvestment / Dartify 等已接入项目。  
2. L 档 jsonl：**未验证 ≠ 永久废弃**——源文未宣判永远无用。  
3. **无** Claude 4.x / GPT‑5 / o3 等对 capability-probe 四题完整落档（甲/乙/丙）的回执 → 不能把「日历上的新模型」等同「可降保费」。  
4. **无对照实验**证明删除独立对抗审查或合流 checklist 后多轨翻车率不升。  
5. merge-checklist 大量条目来自并行撞车/平台行为 → **协作与环境风险**，不是弱模型记忆问题，模型变强不自动可删。  
6. 厂商 SWE-bench 等分数 **不能外推** 到双轨契约冻结、单一写者、生产 canary 失败率。  
7. 冻结令原文绑定「首次真实结账（v0.3）前」；当前 pin v0.6.x，该令是否仍为全局纪律 **仓内未在同一处显式废止或续期**。  
8. 误播高档后的降档「只删不用的件」是否需用户显式批准，**未与升档协议同等细化**。  
9. L3 双落点对账主要靠 CHANGELOG 成对列名软门；漏写 kit 侧的检出强度 **未产品化**。  
10. 本调查未独立打开 Anthropic/Codex 官方权限文档复核「真护栏=hooks+permissions」外引主张。

---

## 九、与「大模型很强，agent-on 还有用吗」的产品答案

| 问 | 答（本 MRD 立场） |
|---|---|
| 还能发挥作用吗？ | **能。** 价值不在教练写代码，在 **制度**：证据链、边界闸、状态单写者、结账回流。能力越强，翻车越快越大，越需要外置闸。 |
| 怎么更好发挥？ | **主路径压到 S/轻装 + 闸门留硬 + 保费用探针降 + 空转件可见降权 + skill 默认 GStack-first 并压制抢跑。** |
| 什么必须留下？ | 结账—消化、跨仓机械闸、完成=证据、单一写者、契约文件、负空间、破坏性物理闸。 |
| 什么该收？ | jsonl L 空转默认、锁口令复述、下次顺手、默认全开 M/L、Superpowers 规划抢跑。 |

---

## 诚实边界

- **本文件仅 MRD**（需求澄清 / 建设范围拍板用）。  
- **PRD / 需求澄清包全卷 / 技术方案 / phase 卡 / 改 playbook·kit·BOOTSTRAP·hooks 等 canonical / 打 annotated tag / 跑 capability-probe 实测** —— 均属后续，**不在本文件交付范围内**。  
- deep-research 状态为 **Partial**；上列未确认项不得在下游文档改写成「已验证」。  
- 用户拍板 §七之前，任何「已经完成大升级」的说法均为虚假完成。

---

## 附录 · 主题清单（写作用，实现勿直接当 backlog）

**保留/加固**：settlement 闭环、git-guard、L2 证据、L3 单写者、L4 契约、L5 负空间、破坏性闸、Promotion 六项、pin 一行升级、机制带闸。  
**降权**：jsonl 四卡、probe 转化 0、锁口令复述、下次顺手、默认重装。  
**保费**：phase 粒度、审查轮次、双/单轨、TDD 分级、七条旋钮、probe 门槛。  
**Superpowers**：推荐路径砍规划/默认审查/默认 ship 依赖；GStack-first；fallback kit；不改本机卸载。
