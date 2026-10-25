# 2026-10-25 · 功能交付入口与项目简报首批实现

> 职责边界：记录用户要求“功能上怎么交付、按这个改进代码”后的本地实现与验证。它续接 [取舍审视](2026-10-25-agent-on-focus-and-matt-review.md)，不等于整库精简、下游 CI 迁移或发布/全局安装完成。

## 本批交付

1. **跨项目交付入口**：`skill/SKILL.md` 增加 deliver / 交付 / 统筹，自然语言有明确目标时走同一入口。`kit/delivery-workflow.md` 贯穿已有目标、分工、相关验证、批次审查与值守交单；复用现有 task/dispatch，单窗口可做就直接执行。没有 Matt 也能使用。
2. **检查与交付的统一规则**：正文 `playbook/delivery.md`，接入件 `kit/delivery-workflow.md`；骨架、派工、审查、合流、值守交单、BOOTSTRAP 和旧 SOP 同步。工具已覆盖的内容复用可信证据；风格/可选重构不自动阻塞，需求变化更新原验收，修复后局部复查。取消模板“默认至少有一个缺陷”和每切片强制五分钟多处记账。
3. **可执行的项目简报**：Rust `status` 成为独立命令，默认输出目标、窗口登记、最近回报、依赖、卡点、待验收和下一步；进展带来源时间。事实收集搬到 `cli/src/project_status.rs`，原网页消费同一份事实。JSON 结构及旧 status 的 serve/system 参数兼容。
4. **诚实状态**：回执不等于验收、释放或合入；回报者不等于负责人。没有任务绑定就明说；任务来源损坏时不报零任务，Git/worktree 仍可读。文本最多展示 20 个未释放任务，完整事实可读 JSON，不启动模型、服务器或联网刷新。

这是一份随 Agent-On 分发给项目的功能交付能力，不是仅调整本仓 CI。具体项目的检查命令、分支保护和部署链仍须由那个项目的授权会话读取并落实；本批未关闭任何现行检查、启用周计划或修改 Dartify。

## Matt 使用与范围

应用 codebase-design 的小接口与事实汇总集中、writing-for-agents 的单一入口与按条件读取，以及行为接口验证的方法。没有再叠加完整 Matt / GStack 审查流水线，也没有新建 tracker、通用执行引擎或审查角色。原版技能缓存未改；本机两家个人 Agent-On skill symlink 均指向本仓 `skill/`，该源码入口已更新，其他插件缓存是否重载需另核。

## 实际验证

- 新增公共 CLI 行为测试先失败：旧 status 只有计数、缺目标“统一导入接口”。实现后通过。
- `cargo test --manifest-path cli/Cargo.toml --test project_status --test coordination --test session_continuity --test system_overview`：新简报 2 项、协调 25 项、会话续接 23 项、系统总览 13 项通过；系统总览另两项因执行环境禁止本机监听而未能启动。
- 取得执行环境允许后，仅重跑 `system_http_preserves_security_and_project_details_without_writes` 与 `fresh_receipts_update_live_without_restarting_the_server`：各 1 passed / 0 failed。上述相关回归合计 **65 项通过**，没有重跑完整产品套件。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0。`cargo fmt --manifest-path cli/Cargo.toml --check`：exit 0。
- 文档闸初次发现两份旧文档已补职责边界、豁免基线仍留条目；只移除这两条旧豁免后：`DOC-GATE: PASS（230 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）`。这是缩小豁免，不是放宽检查。
- 用新构建的 `cli/target/debug/agent-on status` 读取真实本仓：看到 6 个既有未释放任务、10 项文件重叠、无人在班；旧进展和待验收如实展示，没有修改任务、认领值守或处理这些历史工作。此项证明本机可读，不证明一次真实多窗口任务已完成。
- 与开工备份对比：本批改 19 个既有文件并新增代码/测试/交付说明 4 个；先前 39 个工作文件中，本批范围外 31 个内容未变。涉及原有修改的文件使用局部补丁保留原内容，没有 reset 或整批纳入本批成果。

## 使用与未完成边界

- 源码试用：在项目目录运行本仓新构建的 `cli/target/debug/agent-on status --repo <项目绝对路径>`；全局旧二进制尚未替换。
- 技能入口：读取本仓 `skill/SKILL.md` 的 deliver 路由，或在已读取该源码入口的会话说“agent-on 交付：<目标>”。`deliver` 是 skill 路由，不是 Rust CLI 子命令。
- 当前保留 Dashboard 兼容路径；已经拆出事实汇总，尚未删除网页、清道夫或旧任务。此前未发布的活动、总览、阶段交接仍是既有工作。
- CI 周检自动接线、机器合并策略区分必需/建议检查、共享成果合入判定抽离，以及真实跨窗口交付试点仍未完成，不能把这批入口和模板当作这些运行功能已上线。
- 未执行 commit、push、tag、发布或全局安装；当前主树已有大量其他未发布工作，本批没有携带它们发版。推荐 pin 不变，未来发布仍遵本仓分层提交与 tag 纪律。
