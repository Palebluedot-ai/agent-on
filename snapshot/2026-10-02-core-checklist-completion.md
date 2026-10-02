# 2026-10-02 · 核心与规则精简清单收口

> 职责边界：逐项对照 10-01 原清单，记录本批手工删改、行为验证与生效范围；当前执行规则在 BOOTSTRAP / boot / playbook / kit，不从本快照另生一套规则。发布真相以同批 HEAD 的 annotated tag 与 origin/main 为准。
> 基线：`v0.25.2` / `124303d`。本批包含先前巡逻实施和后续核心清单补齐；旧快照保留当时尚未完成的状态。

## 授权与边界

用户先明确「直接开工」，后续明确要求「把按照核心与规则精简清单 完成和修改 请按照这个 继续递交提交」。该指令覆盖原清单涉及的 CLI / hook 行为修正、产品方向文字及本批交付；不改自动合并的硬停清单，不把项目指令扩大成全局配置写入、真实模型调用或回收现有未知树。

值守当前 `oncall status --json` 返回 `{"present":false,"self_is_oncall":false,"stale":false}`。按维护者交付规则分层提交，封 CHANGELOG、更新推荐 pin，同批 annotated tag 原子推送默认分支。用户独立 intake 不收件、不改、不提交。

## 按优先级逐项收口

| 优先级 / 原项 | 结果 | 实际落点与证据 |
|---|---|---|
| P0 登记/owns 前置 | 退出开工与提交前提，lane 仅记账 | 握手、AGENTS-skeleton、merge-checklist；原有未登记树测试保留 |
| P0 值守审批与寻址漂移 | 入口与现行机器政策一致，交单只认当前登记 | docs/babysit、CONTRIBUTING-CLAUSE；policy.json 未改 |
| P1 续接强制三选一 | 明确指令直接续跑；默认原窗口 native compact | boot/session-handshake、AGENTS-skeleton、README |
| P1 S/M/L 整包升档 | 任务规模、风险、并行分别决定补件与验证 | BOOTSTRAP、adopt、两种 AGENTS、README、kit 目录 |
| P1 成套 MRD/PRD/卡/仪表盘 | 复用已有目标、待办和状态，模板分别按需 | BOOTSTRAP 规划工具箱、adopt、kit/README、merge-checklist |
| P1 全量 TDD / 原型强制重写 | 按风险验证；合格部分可复用 | freedom-vs-discipline、anti-hallucination / AGENTS-skeleton、merge-checklist |
| P1 每轮固定六段报告 | 普通回复简短，交付/失败/决策补相应证据 | output-contract 正文及复制条款、AGENTS-skeleton、派工模板 |
| P1 旧品牌保费 / 换模型必考 | 默认退出；实际反复失败时可选诊断 | model-playbook / track-prompt、capability-probe、初始化与握手；旧实验原文存 legacy，字节校验相同 |
| P1 两项目自动 L3 / 消化必须改正文 | 重复信号只提高归因优先级；修正、合并、删除、退场、保留案例或有理由拒绝均可 | iteration-loop、meta-principles / promotion-card、AGENTS-skeleton；settlement 与 README 同步 |
| P2 整工作区挡无关 commit/push | Git hook 检查有效 index / 待推历史，PreToolUse 保留路由与跨仓授权 | worktree、worktree_hooks、guard / worktree-control-plane、guard/README；11 个真实临时 Git 行为测试 |
| P2 审计账本随 worktree 漂移 | 默认同一主 worktree 账本；显式覆盖只接受绝对路径 | merge_audit.py / docs/babysit、审计 README；真实 linked worktree 追加后主树读到完整哈希链 |
| P2 构建身份 / 升级只改 pin | build-info 编入源码指纹、commit、版本和可核 release tag；doctor 检查宿主与 Git hook 实际二进制 | build.rs、build_info、source_identity、doctor / guard/README、settlement 的版本/模板/执行面三栏回执 |

保留：完成证据、暂停禁令、实际授权边界、值守三权、required CI、独立审计与孤本保护。没有通过删整个历史仓来减规则，也没有新增通用编排运行时。源码指纹只核内容是否一致，不是发布签名或真实性证明。

## 实际验证

- `cargo test --manifest-path cli/Cargo.toml --quiet`：`212 + 1 + 20 + 10 + 7 + 7 + 11 + 11 + 9 + 3 + 3 + 11 + 8 = 313` 项通过，0 failed，exit 0。完整本地输出：`/private/tmp/agent-on-core-complete-rust.log`。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、`cargo fmt --manifest-path cli/Cargo.toml --check`：exit 0。源码 release 构建 exit 0。
- `python -m unittest discover -s tools/merge-audit`（uv、系统 Python、离线）：`Ran 98 tests` / `OK`，exit 0；输出 `/private/tmp/agent-on-core-complete-audit.log`。
- `agent-on intake-lint`：`承接层校验通过:208 张卡,六项齐全。`，exit 0。文档闸在加入本收口快照前：`DOC-GATE: PASS（222 份 markdown）`，exit 0；封版后再核推荐 pin 与真实 tag。
- `git diff --check`：exit 0。两份 legacy 归档与原 HEAD 文档 SHA256 一致。
- 操作范围实测：无关暂存文件 / `--only` / 空提交 / 无关推送 / tag 推送可通过；`-a`、指定共享路径、rename 源路径和后来被 revert 的待推共享历史仍拦真实冲突；非法 pre-push 输入拒绝，不变成空范围。
- 构建身份实测：真实已编译 CLI 与相同源码副本一致；只改源码、不改版本且把二进制 mtime 调到未来，宿主与 Git executor 均标 STALE；旧程序无 build-info 标 UNVERIFIED，不凭时间给绿。

## 已改源码与现场生效分别验收

本机新版 release CLI 与 READ_ROOT 内容身份相同。当前 native Git hooks 仍调用 `/Users/chao/.cargo/bin/agent-on`，旧程序无 build-info，doctor 实报 UNVERIFIED；Claude 已启用插件缓存仍为 0.12.1，配置/脚本与新源码不同，实报 STALE。未改全局安装或插件缓存，不能把源码完成说成所有窗口已经用上新规则。

独立巡逻、Agent OS 和清道夫源码与临时仓/替身集成测试已落地。真实 Terminal 弹窗、厂商认证调用、桌面 create→wait→hook→bind 与系统定时激活未现场验证；没有启动本项目巡逻或清道夫，没有回收任何现有树。原窗口 compact 是流程约定，未接管宿主压缩阈值或增加自动摘要注入。

用户已有 `intake/2026-09-29-dartify.md` 保持未跟踪，SHA256：`85273f542f18f54cfee06cecd836b5d971011a20966f46da8fb4ffd46955321d`。
