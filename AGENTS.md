# AGENTS.md — agent-on 仓开发纪律（v2）

> 职责边界：本文件规定 **agent-on 仓库自身** 的开发、验证与发布规则。产品定位与资产边界以 [CHARTER.md](CHARTER.md) 为唯一权威；对外安装与用法见 [README.md](README.md)。

## 当前状态

**当前推荐版本：`v2.0.0`**（版本真相 = git annotated tag；变更账本见 [CHANGELOG.md](CHANGELOG.md)，v0.x 升级指引见 [docs/migration-v2.0.md](docs/migration-v2.0.md)）。

仓库由四部分组成：
- `skills/`：4 个独立原子技能（`prove-it`、`thin-review`、`wt-doctor`、`merge-check`）
- `rules/`：一页核心铁律 [rules/core-guardrails.md](rules/core-guardrails.md)（及插件加载软链 `rules/AGENTS.md`）
- `cli/`：零运行时外部工具依赖的 Rust 单二进制 `agent-on`（`wt`、`merge-check`、`install`、`doctor`、`build-info`，最低支持 Rust 1.89+）
- `bench/cases/`：50 个真实项目翻车案例索引；历史重型脚手架归档于 `legacy/v0.30-scaffold/`

## 自举开发纪律

1. **遵守自家铁律与验证协议**：本仓开发直接适用 [rules/core-guardrails.md](rules/core-guardrails.md)。修改 `cli/` 行为必须通过对应集成测试并按 [skills/prove-it/SKILL.md](skills/prove-it/SKILL.md) 完成错误对照验证；非平凡改动合并前可执行 [skills/thin-review/SKILL.md](skills/thin-review/SKILL.md) 空白上下文复核。
2. **决策入快照，宪章需拍板**：架构与产品边界变更写入 `snapshot/YYYY-MM-DD-*.md`；修改 [CHARTER.md](CHARTER.md) 须用户明确批准。
3. **单一权威，拒绝膨胀**：每项规则只在一处权威文件维护；`skills/*/SKILL.md` 与 `rules/core-guardrails.md` 保持精悍高信噪比，不为大模型默认已会做的常识堆砌废话。历史退役内容放入 `legacy/`，不与现行文件混放。
4. **本地门禁全绿后交付**：
   - Rust 代码变更：`cargo fmt --manifest-path cli/Cargo.toml --check`、`cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings`、`cargo test --manifest-path cli/Cargo.toml --locked`
   - CI 范围脚本变更：`python3 -m unittest discover -s .github/scripts -p 'test_*.py'`
   - 文档与版本一致性：`python3 .github/scripts/check_docs.py`
5. **版本与 Tag 对齐**：在本仓直接交付发版时，同步更新 `CHANGELOG.md`、`cli/Cargo.toml`、`plugin.json`、`.claude-plugin/plugin.json`、`.claude-plugin/marketplace.json`、`.codex-plugin/plugin.json` 以及 `AGENTS.md` / `README.md` 推荐版本号，并打 annotated git tag 推送远端，保持 `check_docs.py` 全绿。

## 红线（不做的事）

- 不恢复业务仓侵入文件（`agent-on.lock.md`、`BOOTSTRAP.md`、跨仓结账台账）
- 不恢复常驻后台服务、网页面板、自建 Agent 派工器或粗粒度 `PreToolUse` 命令拦截钩子
- 不在 `wt --prune` 或 `install` 中添加未获明确确认的强制覆盖（`--force`）或分支删除逻辑
