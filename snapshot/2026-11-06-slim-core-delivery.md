# 2026-11-06 · 精简核心的实施与交付证据

> 职责边界：记录本轮用户授权、实现、验证、发布和安装。路线与删除范围见同日路线图；未取得实际回执的步骤不宣称完成。

## 用户批准

用户原文：「可以，按这个路线图开始实施，把这些冗余功能彻底删掉，验证通过后按仓库规则提交、打 tag 并推送。」

已确认删除自建派工、巡逻、活动采集和网页面板，保留普通对话、文字状态、Git 冲突保护和合入审计。包含路线图列出的共享基础拆分、专属测试／安装接线／配套清道夫退出、CI 与正文更新。此范围包含 `cli/src/**`、hooks、插件、CI 和配套规则；按现有硬停政策引用本次用户批准，不扩大到无关权限。

版本问题用户明确答复：「保持 v0.x，本次明确作为 breaking 特例」。因此本批为 **v0.30.0**，附迁移注记；不声称满足 v1.0 外部用户验收，不改变后续版本规则。

## 实施

- 基于远端 `5e4142c`（#44）继续；保留已有按范围 CI，固定工具链并分开下载／构建缓存。构建 key 包含实际编译器指纹、平台、lock、profile 和内容标识，兼容前缀可恢复更新后的构建；原 OS+lock 命中后不保存问题消除。
- 删除 activity、coordination、codex_model、dashboard、system_dashboard、janitor 和仅派工使用的 integration 模块；删面板 HTML、浏览器用例和专属集成套件；kit 静态面板模板也退出供货。
- 文件锁／原子写入抽到 state_io；status v2 读取当前 checkout 的真实 Git 状态、现有项目记录位置和合入缓存，不推断任务完成或窗口活动。
- SessionStart 只保留身份接续；逐工具 capture 与结束事件注册移除，旧非启动事件入口静默无操作。历史绑定仅保留路由限制，不当实时状态。
- retire-legacy 默认预览，apply 关闭项目旧采集／janitor 策略、备份配置并校验卸载旧 janitor 调度；不删除 checkout、分支、恢复 ref 或历史记录。原 report-only GC 保留。
- 方法／接入双落点：playbook/delivery.md + kit/delivery-workflow.md；boot、skill、Codex、身份接续、README/AGENTS/CHARTER 与迁移说明同步。历史执行书进 legacy/retired-v0.30。

## 本地验证（发布前）

- `cargo test --manifest-path cli/Cargo.toml --locked`：16 套，**345 passed / 0 failed**。
- `cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings`：exit 0；fmt 通过。
- Python 独立合入审计：**101 tests OK**；CI 选择器：**9 tests OK**。
- 文档闸：`DOC-GATE: PASS（250 份 markdown：职责边界棘轮 / 相对链接 / 推荐 pin 三处一致）`（实施版本更新前）。
- 新回归覆盖退场命令拒绝执行、旧非启动 hook 不采集、坏历史任务不阻塞 status、当前 worktree 语义、迁移可预览／重复且保全历史、坏配置与软链拒绝；身份与合入保护回归保留。
- 本仓 `retire-legacy --apply`：关闭 1 份旧活动配置，janitor timer absent；历史保留。只读进程核对未发现 agent-on 面板／巡逻运行进程。已有 worktree-gc 报告任务不属于退场清道夫，保留。

## 待补实际回执

线上候选 CI、最低 Rust 版本、缓存复用实验、合入与审计、annotated tag 推送、实际全局／插件执行器升级分别补记。夜间触发延迟不因删功能宣称已解决。

## 历史 #44 的本次追认

合入前 report 实际发现 #44 为 UNVERIFIED_HARDSTOP：10 月 29 日已合入，但缺可核验批准指针。已向用户说明该 PR 的文档免编译、测试范围精简内容及链接，并询问“确认保留并追加本次追认，保留原历史，不伪造当时批准”；用户答复：「按你建议」。据此于 2026-11-06 追加追认记录，不改写此前账本，也不声称 10 月 29 日已有批准。

最低支持版本验证：`cargo +1.89.0 check --manifest-path cli/Cargo.toml --locked --all-targets` exit 0；默认工具链未更改。

## 批次审查补修

需求与约定两轴审查定位并修复：迁移锁文件软链可能截断目标（先红后绿回归）；首次备份改为完整写入／同步后原子发布，已有不完整备份拒绝覆盖原配置；历史绑定扫描取消 256 条截断，保留超上限的已知窗口路由限制；无旧调度器支持的平台跳过 Unix HOME 探测。修复保留在相关保护回归中。

首个线上 PR #45 run 42178335421 全部必需检查 success；此为修补前候选，最终候选需另核，不以旧绿代替新提交验证。
