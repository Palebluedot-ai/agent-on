# intake/ — 承接层(项目回流的落点)

> 职责边界:本目录只收「agent-on 结账」写入的 Promotion Card 文件。它是真相源三层里的**承接层**——素材不是规则,未经消化不得影响 canonical(playbook / kit / bench 正文)。机制全文见 [../playbook/iteration-loop.md](../playbook/iteration-loop.md)。

## 规则(五条)

1. **一次结账 = 一个新文件**:`<YYYY-MM-DD>-<项目名>.md`,append-only,不动他人文件——多项目并发在 git 层物理不撞
2. 每张卡按 [../kit/promotion-card-template.md](../kit/promotion-card-template.md) 六项齐,缺一拒收
3. 消化后**原地**标注去向:`landed@<commit>` / `rejected(原因)` / `deferred`——不设 archive,标注即收口
4. **目录即仪表盘**:`ls` 一眼数出未消化文件数;≥3 份 = 该开消化会话了(结账收尾自动播报)
5. 本目录只有两类写者:结账会话(新建自己的文件)、消化会话(标注去向)
