# UI 结构收口测试映射（structure test map）

> PLAN-014 交付合同（r2）。旧→新测试位置/ID映射与定向消费锚点；断言与原子
> 义务零弱化。分类：structure ｜ 状态：active ｜ 来源计划：
> docs/plans/014-ui-man-structure-closure.md

## 1. #[test] 守恒账（拆分前=拆分后）

| 面 | 原文件 | 拆分后 | 守恒 |
|---|---|---|---|
| U1 | ui_gen/vue.rs 329 | vue/mod.rs 329 | 329=329 |
| U2 | aura_view_builder.rs 172 | mod.rs 134 + route_memo.rs 38（plan045/046/047 随impl#2 owner迁入） | 172=172 |
| U3 | iced/renderer.rs 199 | renderer/mod.rs 199 | 199=199 |
| M1 | auto/src/main.rs 8 | main.rs 8（含双侧保留的双 `#[cfg(test)]` 怪癖） | 8=8 |

测试名/断言/mod名零改；唯一位移=U2尾族随owner。风格widget内嵌测试
（plan732/737/plan095）随 style_widgets.rs；热键测试随 subscriptions.rs；
plan024 随 shell_state.rs。

## 2. 定向执行锚点（bill-of-evidence）

- compiler lib 单进程全量 1127 passed/0 failed（`cargo test -p auto-ui-compiler
  --lib`）；`ui_gen::vue` 族 327/327。
- runtime 日常档 2196/2196（nextest 默认filter；178 契约skip）；builder 族
  164/164；renderer 族+lucide 170/170；plan811 源守卫 7/7（守卫扫描 8 个
  renderer 家族文件）。
- Man：unit 24/24；tcore 24/24；plan816+plan890+cmd_ui 真实进程 10/10。

## 3. 真实消费（V-09/10/13）

- 真实 VM 窗口：`auto run -r vm` + autoui-verifier MCP client；oracle=
  输入 structure14 视图可见且 store 读回一致、双击计数2、owned 关闭重开。
- 真实 GPU 帧：首帧/交互帧字节不同（sha be94d559/f3753f90），屏证据与 store
  状态交叉（计数 2→3）。
- 生成消费：fresh/locked（字节稳定 5b6a0cec）/A→B/生成中断恢复（精确恢复
  d7383278）；涂改产物不自愈=工具合同边界（源不变跳过写）。

## 4. 已知边界

- ugrep/脚本内联检查在 gen 退出后即刻读可能见部分内容——判定以静止后多重
  独立读回为准。
- 涂改产物不自愈（见上）；Linux cosmic 原生面未执行（沿 018 口径不冒充）。
