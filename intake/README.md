# intake/ — 承接层(项目回流的落点)

> 职责边界:本目录只收「agent-on 结账」写入的 Promotion Card 文件。它是真相源三层里的**承接层**——素材不是规则,未经消化不得影响 canonical(playbook / kit / bench 正文)。机制全文见 [../playbook/iteration-loop.md](../playbook/iteration-loop.md)。

## 规则(五条)

1. **一次结账 = 一个新文件**:`<YYYY-MM-DD>-<项目名>.md`,append-only,不动他人文件——多项目并发在 git 层物理不撞
2. 每张卡按 [../kit/promotion-card-template.md](../kit/promotion-card-template.md) 六项齐,缺一拒收
3. 消化后**原地**标注去向:`landed@<commit>` / `rejected(原因)` / `deferred`——不设 archive,标注即收口
4. **目录即仪表盘**:`ls` 一眼数出未消化文件数;≥3 份 = 该开消化会话了(结账收尾自动播报)
5. 本目录只有两类写者:结账会话(新建自己的文件)、消化会话(标注去向);**git 动作(add/commit/push)只归 agent-on 仓会话**——结账会话落盘即止(跨仓边界硬规矩 2026-07-13,消化开场负责收件 commit)

## 上游 / 多人(第六条)

6. **社区贡献只进本目录**:intake-only PR 或 [Issue 模板](../.github/ISSUE_TEMPLATE/intake-card.md)。**禁止**贡献 PR 直接改 `playbook/` `kit/` `boot/` 等 canonical——那些只能由维护者消化会话写入。协议全文 [../boot/settlement.md](../boot/settlement.md)「上游贡献形态」。默认用户**不必**提 PR;自愿「贡献上游」才走远程通道。
