# Snapshot — 2026-08-03 · deep-research 余量审视（搭 / 废 / 留）

> **职责边界**：对照 deep-research（`wf_019fc301…` Partial）与 [2026-08-02-light-hard-premium-mrd.md](2026-08-02-light-hard-premium-mrd.md)，勾选 **已落地 / 本批补齐 / 仍 deferred**。  
> **不是** PRD 全量实现清单；本批只 ship 最小可执行余量（降档协议 + 可见降权 + 检查扩展）。

## 状态一句话

B1/C1 与轻主路径文档已在 `e4bebc6` 等落地。本批补 research 明确缺口：**误播高档后的降档协议**（对等升档）+ **空转件对读者再钉死** + **开箱断言扩展**。三不变量与结账闭环 **不删**。

## 一、主干保留（research：仍在扛）

| 件 | 落点 | 状态 |
|---|---|---|
| 结账→intake→消化 + annotated tag | `boot/settlement.md`、`playbook/iteration-loop.md` | **保留** |
| 跨仓 git 机械闸 | `kit/guard/`、`hooks/` | **保留** |
| 三不变量：契约即文件 / 单一状态写者 / 完成=证据 | `playbook/orchestration-future.md`、BOOTSTRAP L2–L4 | **保留**（资产，不可当保费删） |
| 负空间、破坏性物理闸 | anti-hallucination、bench 14/19 | **保留** |
| pin 一行升级 | settlement 升级口令、lock 模板 | **保留** |

## 二、废除或可见降权（research：绑手/空转）

| 件 | research 判定 | 产品动作 | 状态 |
|---|---|---|---|
| L 档 jsonl 四卡 + audit-lint + schemas | 旁路、零真实项目跑通 | 开箱勿启用；README/ledger/schemas 横幅 | **降权已做**；schemas 本批再钉 |
| 二代「每轮 Global/In-Scope/Next 锁口令复述」 | 过时形式 | 死亡名单只留 fail-closed；**禁止复活为默认** | `phase-gates` §五；本批交叉链 |
| 「下次顺手」无闸推迟 | 流程税且无效 | 禁顺手；绑机械信号或日历 | truth-hierarchy 已有；**保留禁令** |
| Superpowers 默认执行栈 | 太重 | 退出默认推荐 | B1/C1 **已落地** |

## 三、仍要搭 / 可调保费（research + MRD 余量）

| 件 | 动作 | 状态 |
|---|---|---|
| **误播高档后的降档协议** | 与升档对等：用户显式批准、只删不用件、不重播、记 lock | **本批落地** → `boot/adopt.md` §三 |
| 开箱断言扩展 | 扫降档协议 / schemas 旁路 / 锁口令非默认 | **本批** → `scripts/check-skill-routing.py` |
| 保费旋钮（phase 粒度/审查轮次/双轨/TDD） | 探针调，不整包删 | 文档已有；**不做**假实测分 |
| capability-probe 案例→新题转化 | 转化数 0 | **deferred**（诚实不扩题） |
| >95% ↔ probe 三档映射表 | 未写死 | **deferred / 未确认** |
| L dogfood jsonl | 未验证≠废弃 | **deferred**（A1 旁路） |
| 冻结令是否仍全局 | v0.3 绑定文 | **deferred** 诚实标注，本批不续期也不假废止 |

## 四、本批不碰（non-goal 再确认）

- 不跑 capability-probe 真人分；不打 release tag；不卸 Superpowers 插件；不改 agent-memory；不删 audit-lint 源码。

## 证据链

- research 报告会话路径：deep-research `wf_019fc30106f67180b98e252744004c3f`
- 上轮轻主路径 commit：`e4bebc6`（agent-on）/ `e399df5`（agent-memory C1）
