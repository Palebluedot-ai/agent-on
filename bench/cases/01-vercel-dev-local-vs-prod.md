# 案例 01:vercel dev 三连坑:本地模拟器≠生产

> 层级:L2 教训 | 来源:Euan 实战 2026-07-03 S1 真机联调 | 入册:2026-07-07 批三

## 症状
App 报「网络不给力」。curl health 拿到 NOT_FOUND;换官方 Hono 模式后函数挂起 30s;再换适配器后 body 又是空的——一路排下去,坑是连着的三个。

## 根因
三坑同源:平台本地模拟器(vercel dev)的行为不等于生产运行时。①`[[...route]]` 通配路由本地能识别、生产 NOT_FOUND;②hono/vercel 是 fetch 风格,本地 Node 运行时却在等 (req,res),于是挂起;③vercel dev 无视 `bodyParser:false`,body 被吃空。反直觉处:同一份代码,本地跑通不等于生产跑通,模拟器的宽容会骗你。

## 修法
分层 curl 定位,每层只修一个变量:换官方 Hono 模式(api/index.ts + vercel.json rewrite,弃通配)→ 换 @hono/node-server@1 的 /vercel 适配器(v2 删了该出口)→ 网关双路兜底 readJsonBody,把兜底写在服务端而不是跟模拟器缠斗。

## 可复用规则
本地网关不通时先 curl 分层定位(health→带体端点→带鉴权端点),每层只改一个变量;平台本地模拟器行为≠生产,兜底写在服务端别跟模拟器缠斗。

## 已固化到哪
已固化:sop.md「外部服务集成清单」第 1 条(2026-07-08 消化本批案例时新增——本地模拟器≠生产、分层冒烟每层一变量、兜底写在服务端,全覆盖);BOOTSTRAP L7 为精简版铁律。同类分叉根另见案 07/09。
