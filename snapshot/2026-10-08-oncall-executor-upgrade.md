# Snapshot — 2026-10-08 · 值守身份修复部署到实际执行器

> 职责边界：记录 Dartify 新会话仍被误拦的回流、本机执行器升级和验证证据。不修改 Dartify 值守登记，不代替原窗口的真实合并验收，不发布其他未完成功能。

## 原因与范围

来源：[Dartify 10 月 8 日回流](../intake/2026-10-08-dartify.md)第一张卡。新会话也能触发旧缺陷：个人 PreToolUse 没有插件标记或 Bash 的适配器环境时，旧程序忽略原生 `CLAUDE_CODE_SESSION_ID`，不能识别 Claude 宿主。修复不只适用于 resume。

源码修复已在 v0.28.1 发布，但本机仍执行 10 月 4 日构建的 0.28.0。全局、仓内 release、Claude 0.28.0 缓存和 Codex 0.28.1 缓存四份旧二进制均为 commit `102b477`、指纹 `fnv1a64:9fa66d9b5bc58c27`。Codex 缓存的源码已经更新，二进制仍旧，目录名不能作为执行版本证据。个人 hook 和 Claude 插件 hook 各自选二进制，更新一份不够。

只读核对原值守会话记录，最后可见的相关真实合并调用是 2026-10-07T04:37:38.417Z 的 #404，仍被旧文案拦截，没有身份核对行。这是旧现场证据，不是部署后的验证。

## 实际执行

用户要求尽快解决值守不能合并，沿用已发布修复和既有安装。更新前逐文件备份并给出回退命令：

```bash
bash /private/tmp/agent-on-oncall-host-20261008/rollback.sh
```

备份目录 0700，包含四份旧执行器、Claude 旧缓存的六个对应 CLI 源文件和原插件登记条目。回退脚本恢复精确文件和本插件登记，遇到后来改动拒绝覆盖，保留新缓存，不覆盖个人配置或信任记录。脚本通过语法检查，未在活环境执行回退。

隔离构建当前本机源码，保留 10 月 4 日已批准安装的活动等功能：

```text
cargo build --manifest-path cli/Cargo.toml --release --offline --locked \
  --target-dir /private/tmp/agent-on-oncall-host-20261008/target
Finished release profile; exit 0
```

先原子替换四个已存在的执行器，并同步 Claude 旧缓存的六个差异文件：`session.rs`、`oncall.rs`、`coordination.rs`、`paths.rs`、`Cargo.toml`、`Cargo.lock`。实际差异只有已发布身份修复与版本元数据；其余 CLI 源码与原已安装功能一致。

随后实际运行 `claude plugin update agent-on@agent-on --scope user`，返回 `updated from 0.28.0 to 0.28.1`，登记指向新缓存。当前会话已加载的 0.28.0 路径也已补修，下一次调用不用等待重启才获得修复。没有修改 hook 定义；新登记按宿主原生重载流程应用。

最终五个路径是全局 CLI、仓内 release、Claude 旧/新缓存、Codex 0.28.1 缓存；二进制 SHA256 相同，均返回：

```json
{"version":"0.28.1","commit":"5f48584b722e941a8b19143aa95a49a2e107ab4a","release_tag":null,"source_dirty":true,"source_fingerprint":"fnv1a64:e964ada2c77a43d3"}
```

这是含既有本机未发布功能的构建，不能冒称纯 tag 构建。没有新增 Rust 源码或新发布。个人 settings、hook 定义和 shim 的升级前后 SHA256 相同；三个相关缓存的全部 CLI 源码与本仓一致。未删除残留缓存或嵌套 worktree，未释放、认领或改写 Dartify 登记。

## 验证与未完成边界

```text
cargo test --manifest-path cli/Cargo.toml --offline --locked \
  --test session_continuity --test oncall_liveness --test oncall_routing --test coordination
23 + 7 + 7 + 25 = 62 passed; 0 failed

隔离临时 Git 仓，同一登记与原生 Claude 环境：旧个人 guard exit 2；新候选 exit 0。
实际个人/Claude 旧缓存/Claude 新缓存/Codex 缓存 shim：32 checks; 32 passed。
agent-on doctor --cwd /Users/chao/Projects/Dartify
结论：执行面与 READ_ROOT 一致。
```

32 次入口检查覆盖 fresh/resume 后重新 claim、merge/review、值守放行、同目录另一窗口拒绝且不续值守心跳。只运行 guard 判定，未执行 payload 中的 gh 命令；临时仓的构造环境不作为真实宿主 payload 证据。入口检查明细与前后核对 JSON 在本机 `/private/tmp/agent-on-oncall-host-20261008/`。

已请用户在原 Dartify 值守窗口重试原本获准的动作。尚无部署后的真实回执，第一张卡记为半落；其余两张卡仍 pending。若值守自己也被拦，停止转投循环，按身份故障处理：保留新的「身份核对」JSON，核 mismatches 和实际 executor；不向自己转投，不释放登记消除闸，不注入其他窗口 ID。
