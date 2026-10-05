# Claude 恢复后的值守身份分歧排错

> 职责边界：记录 Dartify 值守窗口报告的 claim/status/whoami 与 guard 分歧、已复现的源码缺陷、修复及验证边界。不是值守交接或发布授权，不修改 Dartify 登记，不以构造 payload 代替原窗口验收。

## 来源与现场只读核对

2026-10-05 用户从 Dartify「30分钟值守循环」带回报告：2026-10-03 resume 后重新 claim 成功，status/whoami 在班，但真实 PreToolUse 拦 gh pr merge/review。用户提供的旧执行器为 v0.28.0、commit 102b477、dirty；九条 PR 手动合入的数量来自报告，本轮没有逐单核查。

本轮只读确认：

- `~/.claude/sessions/99870.json` 的 sessionId/name/cwd 与报告一致。
- Dartify common-dir `agent-on/oncall.json` 的 identity 为 claude＋报告 UUID，worktree 为指定值守树；没有改写或 release 这份登记。
- 本机个人 `~/.claude/settings.json` 仍注册 PreToolUse → 本仓 `kit/guard/agent-on-git-guard`，该入口不是插件 hook，不保证有 CLAUDE_PLUGIN_ROOT。未修改个人 hook 配置。
- 源码的 `Identity::from_hook` 原来忽略 Claude 原生 `CLAUDE_CODE_SESSION_ID`，缺 CLAUDE_ENV_FILE/插件/turn 标记时依赖 `Identity::current()`；后者也只认 CODEX_THREAD_ID 或 AGENT_ON_HOST＋AGENT_ON_SESSION_ID。

Claude 官方文档说明：SessionStart 的环境文件供后续 Bash 使用，PreToolUse 没有 CLAUDE_ENV_FILE；CLAUDE_CODE_SESSION_ID 在 Bash 和 hook 子进程存在，匹配 hook JSON 的 session_id。[hooks](https://code.claude.com/docs/en/hooks#persist-environment-variables)、[环境变量](https://code.claude.com/docs/en/env-vars)。这支持一条可复现的缺陷：Bash 有适配器 export，个人 PreToolUse 只有原生 ID，旧代码无法解析 hook 宿主，故同 worktree 仍被判 feature。

**尚未抓到那次真实 PreToolUse 的原始 payload/环境。** 不把以上源码复现称作已证明现场只存在这一条原因；恢复前后 payload ID 是否变化、是否混用了旧插件 executor 仍未验证。

## 实际比较字段与来源

所有路径最终共用 `oncall::matches_owner`，必须同时匹配：

| 字段 | 登记 | CLI claim/status/whoami | guard |
|---|---|---|---|
| host | identity.host | 独立 CODEX_THREAD_ID 优先；其次 Claude 原生 ID；再适配器 AGENT_ON_HOST | CLAUDE_ENV_FILE → PLUGIN_ROOT → CLAUDE_PLUGIN_ROOT → payload.turn_id → Claude 原生 ID 标记 → current 环境宿主 |
| session_id | identity.session_id | 同上对应的实际 ID；Claude 原生 ID 覆盖残留适配器 ID | hook stdin.session_id 优先；只有缺字段才取 current 环境；非法字段不退回环境冒认 |
| worktree | worktree | 命令 cwd 对应 Git root，经 canonicalize | hook 顶层 cwd 对应 Git root，经 canonicalize；tool_input.workdir 不替换顶层 cwd |

没有把 `--session` 交单地址、ListAgents 短编号、CLAUDE_CODE_HOST_SESSION_ID 的 local_ 编号或 CHILD_SESSION 标记作为授权字段。旧无 identity 登记仍为 legacy-unverified，不按目录授值守权。

## 修复

1. `cli/src/session.rs`：current 优先读取 Claude 原生 ID；个人 hook 无插件环境时用该原生变量识别 Claude 宿主，实际 ID 仍严格用 stdin。
2. `cli/src/coordination.rs`：启动独立 worker 清掉父 CLAUDE_CODE_SESSION_ID，防新增识别来源泄漏值守身份。
3. `cli/src/oncall.rs`：拦截 stderr 增「身份核对」JSON，列 mismatches、actor/registered、worktree、hook 原始 session_id、身份环境和宿主标记的有无；不采集任意环境或全量 payload 日志。明确同目录仍核 host＋ID，改交单名不修身份。
4. `kit/session-control-plane.md` 与 `kit/babysit/ROUTING.md` 对齐接线与只判定探针；没有改变值守三权、expiry、合并审批或旧登记政策，属于实现修复，不新增 L3 规则。

## 验证证据

先新增真实 CLI/临时 Git 仓回归，再修源码：

```text
cargo test --manifest-path cli/Cargo.toml --test session_continuity claude_ -- --nocapture
修复前：2 passed; 2 failed
claude_resume_claim_and_personal_guard_agree_without_plugin_environment FAILED
claude_native_session_overrides_stale_bash_adapter_exports FAILED
```

恢复测试先登记、老化 36 小时，输入 SessionStart:resume 写宿主环境文件，实际 Bash source 后重新 claim/status/whoami，再用不含插件/适配器环境的 PreToolUse 检查 merge/review 放行与心跳。resume 自身不认领或续心跳。

新增负向测试：另一 ID、local_ ID、空/控制字符 ID 不放行、不续心跳；stdin 不被继承 owner 环境覆盖；分别核 host/worktree 差异、未知宿主拒绝；嵌套 Codex 保留实际 ID 优先级；非法原生 ID 不退回适配器；已有 runner 测试增加父 Claude ID 清除断言。只有临时仓里的 classifier 判定，没有执行真实 merge、评论或跨窗口发送。

```text
cargo test --manifest-path cli/Cargo.toml --test session_continuity --test oncall_routing --test oncall_liveness --test coordination
23 + 7 + 7 + 25 = 62 passed; 0 failed

cargo test --manifest-path cli/Cargo.toml
19 suites; 422 passed; 0 failed

cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
Finished dev profile; exit 0

cargo fmt --manifest-path cli/Cargo.toml --check
git diff --check
exit 0
```

首次全量运行的两个已有 system_overview HTTP 测试因沙箱禁止 localhost bind 报 Operation not permitted；自动审批允许沙箱外重跑后全绿。最终测试完整输出在本机 `/tmp/agent-on-oncall-20261005-tests.log`，临时日志不是持久发布资产。

## v0.28.1 独立发布验证

用户要求验证后 commit＋push。本次从 tag v0.28.0 的干净源码独立组装候选，仅包含上述身份修复、测试、接入文档和 patch 版本元数据。其他本地未发布源码不入本批。版本一致性测试先指出 setup 默认 pin 与 Claude marketplace 两处遗漏，已对齐 v0.28.1；最终候选输出：

```text
cargo test --manifest-path cli/Cargo.toml
15 suites; 349 passed; 0 failed

cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path cli/Cargo.toml --check
cargo build --release --manifest-path cli/Cargo.toml
exit 0

cli/target/debug/agent-on intake-lint
承接层校验通过:207 张卡,六项齐全。
```

349 是将要发布补丁的独立回归数；上面的 422 是原工作区含其他未发布代码的回归数，两者不能互相替代。候选日志保存在本机 `/private/tmp/agent-on-oncall-release-20261005-01a10c12/`，release 构建只在隔离目录，没有安装到任何真实宿主。文档三闸中的 tag 存在性在实际 annotated tag 创建后再核，不提前造占位 tag。

## 未完成边界与续接

- 用户随后在本仓明确要求「验证，如果没问题 commit + push」。本轮只发布值守身份修复 v0.28.1：从 v0.28.0 导出隔离候选树，局部加入修复、测试、接入文档及版本元数据，独立验证后分层提交并原子推 main＋annotated tag。既有系统总览/模型/活动改动仍保留在原工作区，不混入本批。
- 构建/验证本仓及隔离发布候选二进制；未更新全局 CLI、原工作仓 release executor 或两边插件缓存，没有改宿主信任、个人配置或 Dartify 文件。
- 下一步核实际个人 hook 与插件 hook 选择的 executor，统一升级后，在原 Dartify 值守窗口按现有宿主流程验证真实 PreToolUse；未拿到该现场回执前，不声明原窗口已恢复。
- 如果仍拦，保留新文案里的身份核对 JSON（只含身份字段），可直接判是 host、session_id、worktree 还是旧/未知身份；不要 release --force 来消除闸，也不要把另一窗口 ID 注入本窗口。
