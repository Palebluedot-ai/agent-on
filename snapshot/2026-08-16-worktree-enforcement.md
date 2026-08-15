# Worktree 执行强制层（v0.12.1）

> 职责边界：记录从“人工运行 worktree check”升级为“关键写入点机械拦截”的产品裁决、安装面与验收证据。它不定义 lane/owns 语义，也不定义回收判据；两者仍分别以 `kit/worktree-control-plane.md` 与 `kit/worktree-gc-pattern.md` 为准。

## 问题

多 Claude / Codex 会话与多 worktree 同时存在时，`claim/status/check/gc` 已能表达轨道、文件边界和回收证据，但全部依赖人记得执行。时间一长仍会出现三类失效：

1. 执行轨未登记，或实际 diff 越出 `owns`，直到合流才发现；
2. 主 worktree 在并行期一边做控制轨、一边继续提交业务改动；
3. 回收盘点靠记忆触发，静态“可回收名单”很快过期。

只补文档不能解决这些失效。v0.12.1 的目标是把已有控制面接到少数高价值执行点，且不引入常驻 daemon、逐文件检查或自动删除。

## 裁决

### 1. Git 是仓库级最后一道闸

`agent-on worktree hooks install` 在仓库 common git dir 安装共享 `pre-commit` / `pre-push`，一次覆盖所有 linked worktree：

- 两个 hook 都运行严格 `worktree check`；
- `pre-commit` 额外阻断“存在活跃执行轨时，主 worktree 的普通提交”；
- Git 实际调用 `pre-commit` 时，merge / squash-merge / cherry-pick / revert / rebase 等控制态不被主树闸误伤；clean merge 使用 `pre-merge-commit`，本版本不安装该第三类 hook，仍由合流清单 + 后续 pre-push 负责；
- 已有 `core.hooksPath` 或真实用户 hook 时拒绝接管，并给出组合指引；不覆盖、不绕开；
- `status` 校验配置、脚本、执行文件与可选调度；`uninstall` 只移除内容指纹仍匹配的 Agent-On 资产。

Git 的 `--no-verify` 仍是 Git 自带的人工逃生口；产品不伪称无法绕过。Agent 发起同一命令时，PreToolUse 仍在 Git 之前检查。

### 2. PreToolUse 只拦高价值写入点

Claude 与 Codex 共用一份 plugin `hooks/hooks.json`。非 git、git 读命令和一般写命令保持轻量；只有 `git commit` / `git push` 才运行完整 lane/owns 审计。这样把阻断提前到 Agent 执行前，又不把每个 Bash 调用变成全仓扫描。

Codex plugin manifest 直接接同一 hook；旧 `hooks-codex.json` 删除，避免双头。非 managed Codex hook 的首次信任仍由 Codex `/hooks` 管理，Agent-On 不静默改写用户 home。

### 3. 每日 GC 是可选报告，不是回收器

`agent-on worktree hooks install --daily-gc` 可额外安装用户级定时任务：macOS 用 LaunchAgent，Linux 用 systemd user timer，每日 03:30 只执行：

```text
agent-on worktree gc --dry-run --json --repo <canonical-repo>
```

报告写入用户 state 目录；配置按 canonical repo 路径生成稳定且互不冲突的 key。没有 apply/delete 模式，不删 worktree、branch 或历史报告。未加 `--daily-gc` 时不安装任何调度。

## 明确不做

- 不做常驻 daemon 或会话编排器；
- 不在每次文件编辑时扫描仓库；
- 不自动 merge，不自动删除 worktree/branch；
- 不静默覆盖已有 Git hook、`core.hooksPath` 或用户 Claude/Codex 配置；
- 不把 Superpowers 引回默认工作流。

## 验收矩阵

发布前必须给出真实命令证据，而不只靠单元测试：

| 面 | 必须证明 |
|---|---|
| Git install/status | 一次安装覆盖 primary + linked worktree；重复安装幂等；漂移可见 |
| pre-commit | 未登记、越界、主树控制轨三类真实 commit 被拦；合法 lane commit 通过 |
| pre-push | 本地 bare remote 场景中，越界/未登记状态真实 push 被拦；修复后通过 |
| PreToolUse | Claude 与 Codex payload 的真实 hook 入口均 exit 2，并返回 lane/owns 修复命令；读操作保持 0 |
| daily GC | 配置只含 `gc --dry-run --json`；安装/status/触发/卸载闭环，无删除动作 |
| 回滚 | uninstall 后 `core.hooksPath` 与 managed hook 消失，用户资产和报告保留 |

最终命令输出、测试数和实盘结果在封版前回填到本节下方。

## 验收证据

### 宿主 PreToolUse

- Codex CLI `0.147.0` 在一次性双仓环境中真实加载项目 `.codex/hooks.json`；普通 `cat` 先通过，同一会话随后发出的跨仓 `git -C <B> commit -m host-probe` 在 Git 执行前被 Agent-On 以 exit 2 拒绝，宿主回执含 `跨仓 git 边界拦截`、工作仓、会话根与被拦命令。测试后两个临时仓和 payload 均已删除，不留 hook 配置。
- Claude Code 已验证 plugin 能发现同一 `hooks/hooks.json`，但本机该次非交互宿主启动停在未登录，未伪称完成工具调用；Claude 形状 payload 仍由 canonical shim 定向验证 commit/push exit 2、读操作 exit 0。Git `pre-commit/pre-push` 是与宿主无关的最终仓库闸。

### Rust / Git / scheduler

- `cargo test --no-fail-fast`：73 unit + 9 真实 Git integration 全过。真实用例覆盖未登记与越界 commit/push、primary 阻断、合法 linked lane 修复后 commit+push、squash 放行、既有 hook 保护、per-worktree `core.hooksPath` override、hook/scheduler 双向 drift、PATH/可执行文件变化、仓库 move 与旧路径不可达的分面卸载。
- `cargo clippy --all-targets -- -D warnings`、`git diff --check` 与本仓 `agent-on worktree check` 均为 0；严格检查回执 `RESULT: PASS`。
- macOS 真机用一次性仓库安装 `hooks install --daily-gc` 后，`launchctl print` 显示唯一 ProgramArguments 为 `worktree gc --dry-run --json --repo <temp-repo>`；`kickstart` 完成 1 次、last exit code 0，报告为合法 JSON、`mode: dry-run`、动态 `candidates: []`。随后 public `hooks uninstall` 成功；LaunchAgent、plist、install-state、`core.hooksPath` 与 managed hook 目录均确认不存在，报告在测试清理前保留并通过 `jq` 断言。全程未出现 delete/apply 命令，也未安装到 Agent-On 真仓。

最终 commit/tag/远端 read-back 贴在本轮交付回执；不把自引用 SHA 伪写进 tag 所指向的文档树。
