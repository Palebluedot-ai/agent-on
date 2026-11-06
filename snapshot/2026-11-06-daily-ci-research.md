# 2026-11-06 · CI 每日一次的可行性研究

> 职责边界：回答“能否不在每次 PR／merge 跑 CI，改为每日一次”，比较实施代价与证据边界。本文件是研究与建议，不代表已修改工作流、合入政策、远端保护或通知设置。基线为 `d5e4cf618f4be8bf633c558f047bf2806b5d16d5`；采用 Matt Pocock research 的一手资料核对方法。

## 已核事实

1. **当前已经没有合入后重复跑 CI。** `gate.yml` 只有 `pull_request`、每日 `schedule` 和手动 `workflow_dispatch`；没有 `push`，也没有 PR `closed` 触发。主要等待来自 PR 创建／更新后的检查。按范围选择器对普通文档免 Rust 编译，但共享模块、闸、未知 Rust 路径与工作流改动仍跑全量。[基线工作流](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/.github/workflows/gate.yml)、[基线范围选择器](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/.github/scripts/quality_scope.py)
2. **能否合入还受本仓政策约束。** `merge_health.required_checks` 指定 CLI 与文档两个 job。`execution_state()` 会等待每个名字上报；只删除 PR 触发、不修改政策，会一直 WAIT。即使清空 required 列表，现有外部检查若正在运行或失败仍会被处理，除非另行明确配置为 advisory。因此“无本仓 Actions 等待”不等于关闭 GitGuardian 等外部服务。[基线政策](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/tools/merge-audit/policy.json)、[独立判定代码](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/tools/merge-audit/merge_audit.py)
3. **每日检查验证默认分支当时的最新提交，不逐条验证白天的 PR。** GitHub 官方说明 schedule 只在默认分支运行，高负载时可延迟，严重时排队任务可能被丢弃。现有 UTC 17:23 对应香港次日 01:23；实际近期曾延迟约五小时。每日模式不能承诺“任何回归 24 小时内必被发现”，也不能把昨夜通过当成今天新提交通过。[GitHub schedule 说明](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule)、[本仓实际时延](2026-11-06-ci-and-product-simplification-roadmap.md)
4. **跳过不等于测试通过。** 官方说明整条 required workflow 因过滤等原因不运行时可持续 Pending；条件跳过的 job 则可显示成功。正确做法是明确哪些证据不再要求，不保留名为“CLI 测试”的空 job 冒充已验证。官方还说明，手动 `workflow_dispatch` 的检查不能被假定满足分支规则的 PR required check。[GitHub required check 排障](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks)
5. **手动全量入口已经存在。** GitHub 支持手动指定分支运行；可用返回的 run、实际 head SHA 与结论作发布证据，但应核运行时该分支是否已前进，不能仅引用旧绿色图标。[GitHub 手动运行说明](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)、[本仓交付证据](2026-11-06-slim-core-delivery.md)
6. **合入审计与审计程序的测试是两件事。** 工作流里的 Python 单元测试检验审计代码；每次合入实际执行 `report`、合后 `record` 才是仓库的审计纪律。减少测试频率不意味着停止真实记账、身份验证、冲突检查或用户授权核对。[AGENTS 自举纪律 8–10](../AGENTS.md)、[master 接入约定](../kit/master-delivery.md)

## 三种选择

下表是基于上述事实的设计判断，耗时均为预期，不是实施后的测量。

| 方案 | PR／合入时 | 每日运行 | 收益 | 接受的代价 |
|---|---|---|---|---|
| A：本仓 Actions 每日一次 | 本地受影响验证、批次审查、真实合入审计；不启动本仓 PR Actions | 全 Rust、clippy/fmt、审计测试、文档、intake | 消除本仓 PR runner 排队及重复环境启动 | Linux／组合回归可能合入后才被发现；本地证据需要明确记录 |
| B：PR 仅轻量检查 | 文档／结构与外部贡献范围检查；不安装 Rust、不下载 Cargo 缓存 | 重检查全部每日运行 | 大部分编译等待消失，仍能早发现坏链接／pin 等问题 | 仍有每次 PR 的 runner 等待；不能称“每天才 CI” |
| C：维持当前按范围 PR 验证 | 相关 Rust 检查；共享基础扩大全量 | 全量补广度 | 当前提交的独立环境证据较强 | PR 更新会继续重复检查；全量候选三次样本 44／62／45 秒 |

本轮用户关注的是持续等待，**A 最直接满足“CI 每日一次”的意思**。B 是愿意保留很短 PR 等待时的折中；不应把 B 说成 A 已实现。C 已完成缓存与功能删除优化，继续压几秒编译时间不会消除每次进入队列的问题。[候选测量](2026-11-06-slim-core-delivery.md)

## 方案 A 的最小改动建议

1. `gate.yml` 保留每日与手动触发，移除 `pull_request`。每日直接全量，不保留不会执行的 PR diff 分支；范围选择器可以继续服务本地受影响验证，不为了每日 CI 重建调度框架。
2. 本仓 `policy.json` 的 `required_checks` 不再要求两个已退出 PR 的 Actions job；保留实际失败、评审要求、draft/hold、冲突、作者身份、硬停批准和审计约束。实施前重新核 GitHub 分支保护／rulesets；若远端另有 required names，同步明确迁移，不能绕过保护。[基线就绪判定](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/tools/merge-audit/merge_audit.py)
3. 删除 PR 专用 `intake-only` job 时，外部贡献“只收 intake”规则仍保留：外部作者在现行政策本来就硬停，审查其完整文件列表后才能合入。不能把该 job 直接搬到 schedule，夜间没有对应 PR 的作者和 diff 上下文。[基线外部贡献检查](https://github.com/Palebluedot-ai/agent-on/blob/d5e4cf618f4be8bf633c558f047bf2806b5d16d5/.github/workflows/gate.yml)、[上游贡献边界](../boot/settlement.md)
4. 正文写清：开发时跑本地受影响测试，一批一次审查；日常合入不等夜间，不声称当前 SHA 有远端测试通过。本地证据复用已有交付记录，写执行命令、相关提交、真实结论，不新造签名协议、常驻代理或第二套台账。[已有验证节奏](../playbook/delivery.md)
5. 夜间发现代码回归后，由当前维护者优先修复／回退，暂停受影响的后续发布；修复可正常合入。这是维护者处理约定，现有 master 没有“夜间红灯自动禁止后续合入”的联动，不把它说成机器已保证。基础设施失败单独诊断，不能把它说成代码通过。取不到本次运行或最近成功太旧时报告“未知／过期”，不要伪造成功；需要发布证据时用已有手动入口补跑。无需添加新后台服务。

## 发布与夜间失败的边界

本仓每轮交付都要求 tag。若又规定“每次 tag 前必须新跑一次远端全量”，就会把每日方案变回每轮等待。因此建议普通批次使用本地受影响验证与已有有效证据；破坏性接口、依赖／共享保护等高风险改变，或已有证据不能覆盖的发布，才手动全量。当前 v0.30 功能删除已有三次全量结果，可以复用；后续若修改了合入政策／工作流，则仍须验证新政策与触发语义，不能把旧绿当新改动已测。[本仓 tag 纪律](../AGENTS.md)、[本轮验证范围](2026-11-06-slim-core-delivery.md)

每日方案意味着一个真实取舍：白天部分平台／集成缺陷可能在 tag 发布后才被夜间发现。若未来 main 合入即生产部署，或外部用户要求每版独立环境验证，重新提高发布门槛；不要把工具仓当前的选择变成所有项目的默认。

GitHub 原生支持只接收失败运行通知，定时任务通知接收者与创建／修改 cron 的账号有关；这是可复用的入口，不保证当前用户已开启，也不能在未授权时替用户改通知设置。夜间红灯必须有维护者处理，不能只增加一条无人看的计划。[GitHub workflow 通知](https://docs.github.com/en/actions/concepts/workflows-and-actions/notifications-for-workflow-runs)

## 研究与实施的边界

研究者仅提交本文件，没有改变 CI、政策、通知、tag 或任何保护设置。主会话随后依据用户“每天 CI 一次”的最新要求推进方案 A；实施与验证结果见同日交付记录，不以研究建议充当已完成证据。本文件本身只核正文与链接，没有为研究文档重跑 Rust。
