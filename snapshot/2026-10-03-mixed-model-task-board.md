# 2026-10-03 · 混合模型任务看板本地预览

> 职责边界：记录本轮同项目 Codex/Claude 任务展示的本地改进与实际证据；不是发布、全局升级或全平台自动捕获验收。

用户要求同项目不同工具的任务统一登记，像 dashboard 大屏展示，少读长文。复用 common-dir 台账与原 localhost 服务，不创建第二套任务数据库、模型运行时或写操作 API。

## 本轮实现

- 任务按待安排、进行中、卡住、待验收分栏；已结项、工作区与长证据折叠。可按工具和关键词筛选，并切换大屏布局。5 秒观察与断开保留最后数据的机制不变。
- `task add/progress --title` 设置短标题，不改完整目标、路径或授权。显式登记/进度/未释放任务结果写入实际宿主 `last_reporter`；它仅是最近登记/回传来源，不是独占负责、当前在线或值守权限证明。派工任务仍使用实际关联会话的工具标签。
- 旧记录缺新字段可读，未知来源不从编号猜。未接入的 Claude 任务不会造假卡片；混合来源验证用临时仓和浏览器响应替身，不写入实际项目台账。

## 已取得证据

- `cargo test --offline --locked --manifest-path cli/Cargo.toml --test coordination`：25 passed / 0 failed。新增两宿主手动任务同账本/linked worktree 共读、旧记录兼容与未知来源、非法标题无状态副作用三项回归。
- 完整 `cargo test --offline --locked --manifest-path cli/Cargo.toml` exit 0；Clippy `--all-targets -- -D warnings`、fmt 与 diff check exit 0。
- `python3 .github/scripts/check_docs.py`：229 份 Markdown PASS（新增本页前）。
- 真实 Chrome 开发预览测试：`{"browser":"PASS","worktrees":6,"tasks":3,"mixed_host_fixture":"PASS","dynamic_lanes":"PASS","console_errors":[]}`。覆盖工具筛选、卡片状态移动、刷新保留展开、未知来源、损坏台账、HTML 字面渲染、手机无溢出、断开留数据、同源与只读 API。

## 现场与未完成边界

原地址 `http://127.0.0.1:8765/` 使用本仓开发二进制预览，仍保留 PR 每 60 秒联网刷新。正式全局 CLI 和 Claude/Codex 插件没有升级，本轮无 commit、tag 或 push。browse 自动添加 `.gstack/` 忽略项用于本机检查状态，不纳入任务数据。

真实台账只有两项未结任务：本轮看板改进，以及仍因默认模型接口拒绝而阻塞的 Codex 接续测试。旧源码交付仍保持 released。尚无真实登记的 Claude 任务，不把测试替身说成 Claude 模型运行成功。巡逻与清道夫仍关闭，没有新建/删除项目 worktree、启动其他模型或改变信任权限。

旧执行器读取新字段没有问题，但回写旧结构可能丢掉短标题/来源。跨工具稳定使用本轮字段仍需后续发布并升级各自实际执行器；本地 UI 验证不等于这一步完成。真实 Claude 任务接入需其窗口登记/回传，不能只打开面板就获得全部任务。完整自动捕获、跨机与云端接续仍不在本轮完成声明内。
