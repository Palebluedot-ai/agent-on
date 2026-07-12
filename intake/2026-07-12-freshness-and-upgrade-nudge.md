# intake — 2026-07-12 · 用户三问：README 保鲜 / 仪表盘保鲜 / 旧版升级提示

> 来源：用户原话「定期更新 readme 和真相之页设置成功能；其他库如果是 agent-on 旧款怎么提示用户升级」。裁决：「定期」改造为**事件绑定 + 读时痛感**（时钟触发是快照三写点定稿时已钉死的负空间——定时刷新产无人核的过期页）。与本批同场消化。

### readme-freshness-on-digest（README 对表挂消化/里程碑）
- source:用户提问 2026-07-12 | agent-on @ 133d772
- evidence:agent-on 自身 README 两次实证漂移——07-10 路线段停在「v0.3 未办」（消化会话看漏）、07-12 案例数 17→23/「两个口令」→四个（本次 README 升级时人肉扫出 5 处过期）;修复动作一直在做但未成文
- confidence:high（双实证）
- claim:①agent-on 侧:消化收尾三件加半句「README/CHANGELOG 的数字与状态截面对表实况,漂了顺手修」②项目侧:sop Phase 7 里程碑收口加半句「项目 README 数字/状态类内容一并对表」——绑必经事件,不设定时器
- suggested_landing:boot/settlement.md 下半场收尾 + playbook/sop.md Phase 7
- rollback:revert
- trace:本会话 README 修复实录（133d772 提交信息「过期修正」五处）
- 状态:pending（低风险，直落）

### dashboard-staleness-at-handshake（仪表盘新鲜度挂握手）
- source:同上
- evidence:dashboard 已有两触发（合流必更+口令），缺「读的时候发现陈旧」这道痛感门——与快照 staleness 标红同构（模板头部本有「最后更新:[日期]」字段可核）
- confidence:medium
- claim:session-handshake 读取表加一行:M/L 项目核 dashboard「最后更新」是否落后 progress.yaml 最近变更——落后就开场提一句「仪表盘陈旧,要更新吗」;dashboard 模板更新时机加第三条「握手新鲜度提醒」
- suggested_landing:boot/session-handshake.md 读取表 + kit/dashboard-template.html 更新时机注释
- rollback:revert
- trace:快照 staleness 痛感门同构（2026-07-11 定稿）
- 状态:pending（低风险，直落）

### upgrade-nudge-at-handshake（旧版 pin 的升级提示）
- source:同上
- evidence:升级提示现仅存在于结账 step 0 对表播报——不结账就永远不知道落后了;handshake 是每次会话必经且已有 model 对表行,加 pin 对表零新机件;lock 模板 pin 行注释本就预留「握手对表会核」语义
- confidence:high
- claim:session-handshake 读取表加一行:核 lock 的 pin vs agent-on 最新 tag——落后就开场播报「pin vX 落后 N 版(含/不含 major),可说『agent-on 升级』」;**只提示不动手**,升级永远显式口令（与「未发布 commit 不伪装成版本」同款诚实口径）
- suggested_landing:boot/session-handshake.md 读取表（与 model 对表行相邻）
- rollback:revert
- trace:settlement step 0 对表播报同构;用户原话「旧款怎么提示升级」
- 状态:pending（低风险，直落）
