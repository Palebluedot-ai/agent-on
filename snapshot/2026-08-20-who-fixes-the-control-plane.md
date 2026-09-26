# 2026-08-20 决策快照:闸必须自带出口 + 谁能修别人的 lane + 本仓首道 CI

> 职责边界:本文件记录 2026-08-20 这一轮「脚手架加固」的决策与证据,不是使用说明。
> 用法看 [kit/worktree-control-plane.md](../kit/worktree-control-plane.md) 与
> [playbook/elicitation-protocol.md](../playbook/elicitation-protocol.md);案例卡看
> [bench/cases/42](../bench/cases/42-lesson-misattribution-overtightening.md)。

## 源流

用户 2026-08-20 转述一段 Dartify 实战对话并定性:「**还是有很多漂移**」,要求**不解这个 case**(它有自己的功能窗口),而是补脚手架本身——文档 / 执行 / CI 三处。

对话原型:一条会话被 `overlaps still-writing lane <id>` 拦住,停下来问用户「不是我跑的话,应该谁来跑呢?」。用户反问之后它才去查仓,发现 `kit/worktree-control-plane.md` 早就写死了答案:**没有别人,该它自己跑**。它自己的复盘也说了两句关键的:「我把教训学歪了」「我那种『凡涉他人 lane 一律先问』的姿势,制造的就是人肉版的恒红闸」。

## 诊断:六处漂移,分三层

| # | 漂移 | 层 |
|---|---|---|
| 1 | `overlaps … lane` 报错只说「谁挡你」,零出口 | 执行 |
| 2 | `STATUS-DRIFT` 给的两条出口,一条错方向(`--status active` 放大边界)、一条要不存在的会话 | 执行 |
| 3 | 「改别的 lane 的登记」在权限表里**根本没有档位**——既不在自动化允许集,也不在人工保留集 | 文档 |
| 4 | 事故教训怎么写没有纪律,归因写歪会把一次事故固化成永久过度收紧 | 文档 |
| 5 | 追问判据(2×2)只管偏好问题,不管「这事有没有客观答案」 | 文档 |
| 6 | **本仓零 CI**:自举纪律六条、文档纪律六条全靠会话自觉——自己就是案 17「治理空转」的标本 | CI |

## 拍板与理由

### 一、判权限看**破坏性**,不看**所有权**

登记(`base` / `status` / 占位 claim)是**描述事实**的,幂等、可回滚、不碰内容一个字节——**谁被它挡住,谁就能改正**。内容(commit / push / 解冲突 / dirty)是干活的人的,一个字节都不许替别人动。删除(目录 / 分支 / `--force`)永远留给人。

**删别的轨的 `owns` 单独标成错误操作**:它不是「更彻底的修复」,只把「占着」变成「越界」,更糟。这正是原事故的真根因。

理由:等一个 `[landed]` 轨的会话回来批准,等的是一个不会来的人。把「不许删」推广成「不许碰」,就是把闸推成恒红——而且推的是人肉那一层,没有任何日志会记下来。

### 二、出口必须**成对**补

只补「你可以自己修停滞轨的登记」→ 会被读成「我可以改任何人的 lane」;只补「别动别人的 lane」→ 就是原来那个病。所以活轨(contract 档)的报错同时明说「不许改它的登记,缩自己的 owns 或交单」。回归测试把两条都钉死,防止后来的人只改一边。

### 三、每条诊断行三件套

谁能清 / 清的命令 / 清不掉时为什么。现状对照:`RESCUE-DEBT` 三件齐(它明说是债、改登记清不掉),`OUT-OF-BOUNDS` 有出口(死锁三解),`STATUS-DRIFT` 与 `overlaps` 两件全缺——已补。**新增任何 FAIL / WARN 文案时这是验收条件**,不是文风建议。

### 四、CI 只钉能机械判定的

不做「单一权威」「内容对错」这类要人读的判定。四件:`cargo fmt/clippy/test`、`intake-lint`、文档三闸(职责边界棘轮 / 死链 / 推荐 pin 三处一致 + tag 真存在)、外部贡献只许碰 `intake/`。

**职责边界用棘轮不用一刀切**:全仓 101 份 canonical 文档只有约一半写了职责边界声明,直接开闸 = 开局就红 = 训练「红了当没看见」。存量 28 份进基线,新增违规立即红,补齐后必须从基线划掉(不许倒退)。

**逃生门齐三条**(照 [kit/ledger-ratchet-pattern.md](../kit/ledger-ratchet-pattern.md)):仓库 variable `AGENT_ON_DOC_GATE_OFF=true` 一键关闸;基线补一行即绿;取证失败退 2、真违规退 1,分开。

## 证据

```
$ cargo test --manifest-path cli/Cargo.toml
155 passed / 7 passed / 9 passed / 3 passed / 3 passed / 11 passed / 8 passed;0 failed（含新增 worktree_gate_exits 3 条）
$ cargo fmt --check && cargo clippy --all-targets -- -D warnings
FMT: clean / CLIPPY: clean
$ python3 .github/scripts/check_docs.py
DOC-GATE: PASS（193 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）
```

判别式探针(案 35:证明约束生效靠暗号值,不靠恰好合规)——三闸逐一投毒,每闸只对自己那类信号变红,复原后回绿:

| 探针 | 结果 |
|---|---|
| 新建 `kit/zz-probe.md` 不写职责边界 | `FAIL：职责边界缺失：kit/zz-probe.md …` ✅ |
| 该文件加一条 `../nope/gone.md` 死链 | `FAIL：死链：kit/zz-probe.md:5 …` ✅ |
| 把 AGENTS.md 的 pin 改成 `v0.99.0` | `FAIL：推荐 pin 跨文件漂移：AGENTS.md=v0.99.0，README.md=v0.19.0` ✅ |
| 全部复原 | `DOC-GATE: PASS` ✅ |

顺带修掉两条真死链:`playbook/anti-hallucination.md:78` 与 `playbook/elicitation-protocol.md:58` 都把 `../kit/explore-prompt-template.md` 写成了 `kit/…`,从 `playbook/` 解析永远落空——存在很久,没有闸就没人发现。

## 悬点(交值守)

本轮有两处 canonical 被**活轨**占住,按制度不抢,走接线单:

1. `CHANGELOG.md` —— 值守轨 `worktree-output-clarity-placeholder` [active] 持有。本轮条目待值守写入。
2. `bench/cases/README.md` —— 轨 `dtcg-post-landing-revalidation` [ready] 持有。案 42 的索引行待补。

两处都不是本轮改动的必要条件;闸与文档已自洽,索引缺行只影响检索。

## 诚实边界

- CI 是**首道**,不是全套:单一权威、commit 分层、tag 纪律仍靠人。tag 那条只验「推荐 pin 有对应 tag」,验不了「这批 commit 有没有打 tag」。
- 文档闸的职责边界判据是**字符串匹配**,只验「声明了」,验不了「声明得对」。
- 死链闸不管 `snapshot/` `intake/` `legacy/` 三个冻结层——那里记的是「当时是这样」,改它们等于改历史。
- 本轮没碰 Dartify 那个具体 case,那是另一个功能窗口的事。
