# 案例 25:Claude Design `.dc.html`——file:// 静默断依赖 + fiber 无头驱动截基线

> 层级:L2 教训 | 来源:Euan-Flutter 2026-07-21(design/demo-v9 @8374abc) | 入册:2026-07-26 第十三次消化

## 症状
用 `file://` 直接打开 Claude Design 导出的 `.dc.html`:页面不白屏,但排版崩、字缺位;console 报 `[dc-runtime] x-import: FAILED to load ./ios-frame.jsx … URL scheme "file" is not supported`。另:逐屏点点截基线不可脚本化,demo 微调后重截成本爆炸。

## 根因
1. **运行面**:dc-runtime 的 `x-import` / fetch 依赖本地 HTTP;file 协议不支持,失败静默到 console,主壳仍渲染 → 极具迷惑性。
2. **取证面**:交互原型的屏态藏在 React fiber 的 logic 实例里,靠人手点导航拿不到认证态/子页,也无法一键再生。

## 修法
1. **必须**起本地 HTTP 服务跑 demo(`python3 -m http.server` 或等价);排查首看 console 的 `x-import FAILED`。
2. 无头驱动:从任意 DOM 节点沿 fiber `return` 链找到含 `.logic` 的 `stateNode`,调 `go(screen,extra)` / `setState` 直切屏;对固定尺寸设备框做 element screenshot。方法入 demo 旁 README,微调后一键再生。

## 可复用规则
- `.dc.html`(及同类「本地模块 import 的静态导出」)禁止 file:// 验收;HTTP 服务 + console 绿才算跑通。
- Demo 基线截图要可脚本化再生,别只靠人工点击路径。

## 已固化到哪
本案例卡;phase 卡 §0 Demo 锚点四件套(基线截图+更新流程 README 必入仓)。
