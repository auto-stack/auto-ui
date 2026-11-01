# UI 结构收口（structure closure）

> PLAN-014 交付合同（r2）。四面内部职责/状态owner/公开兼容的有限结构合同；
> 不新增语义、不改公开行为。分类：structure ｜ 状态：active ｜ 来源计划：
> docs/plans/014-ui-man-structure-closure.md

## 1. 四面模块布局（公开路径零改）

| 面 | 拆分后布局 | mod保留 | 行为合同 |
|---|---|---|---|
| U1 Vue | `auto-ui-compiler/src/ui_gen/vue/`：mod + nav_sidebar/script/template_media/expr/tags/style_helpers/shadcn_attrs/project_files/store/library | mod.rs=类型(VueGenerator/VueMode/payload/WidgetTemplate)+自由助手+BackendGenerator/Default+generate_sfc编排+测试 | 单 `VueGenerator` state（~75字段）唯一registry；`ui_gen::vue::VueGenerator` 路径不变；golden字节/模式/reset/010路由/fingerprint语义不变 |
| U2 Aura | `auto-ui-runtime/src/ui/aura_view_builder/`：mod + slots/eval/tracked/convert_element/nav_sidebar/child_widgets/tags_container/tags_panels/tags_controls/events/props/route_memo | mod.rs=类型(AuraViewBuilder/SlotFills/OutletInitIdentityGuard)+thread-locals+自由助手+build/build_with_debug入口+测试 | `&VmBridge` 仅借用（owner=dynamic.rs `DynamicComponent.bridge`）；求值次序/惰性/键/路由/挂载与调试身份/guard恢复零改 |
| U3 Iced renderer | `auto-ui-runtime/src/ui/iced/renderer/`：mod + style_widgets/media/projection/host/subscriptions/shell_state/devtools | mod.rs=文件头助手与thread-locals/statics+IntoIcedElement trait+IcedMessage+convert_view_messages+测试 | 消息序/active判据（primary_app/app_of_window/focused_window）/last-owner exit（WindowClosed空→shutdown_broker+exit）/纹理与external pin寿命零改 |
| M1 Man CLI | auto-ui仓不含；见 auto-man `docs/specs/cli/structure-closure.md` | — | — |

拆分为纯 impl/条目移动：切点在方法文档/属性边界逐一经字节映射核验；测试随文件保留
（aura plan045/046/047族迁至 route_memo.rs，#[test] 守恒 172=172）。

## 2. 状态与资源owner（唯一）

- `VueGenerator` 字段即全部生成态；`reset()` 保 config 字段不清（既有语义）。
- `AuraViewBuilder` 借用 `&VmBridge`；Bindings/tracked三元组（path/id_map/probe）
  由 `build_with_debug_gated` 唯一创建、显式穿线（未收敛为struct——29签名散布
  7子文件，收敛登记为UI-007接续候选）。
- `DesktopSession`（session.rs）拥有 windows/apps/host/Drop收子进程；renderer
  子模块只经 `&mut DesktopSession` 变更。纹理句柄缓存三处 OnceLock 语义原样。

## 3. 公开面与禁止边

- `pub mod vue` / `pub mod aura_view_builder`（模块路径）与 iced/renderer 的全部
  外部消费路径（`renderer::{X}`）逐名恢复（mod.rs 重导出面）；`pub(crate)` 面
  同步重导出（session.rs/stage3/broker_surface/headless 消费者零改）。
- compiler 零 runtime 反依赖（cargo tree 证）；Man Core 零 UI/VM 边；语言仓零
  auto-ui 边。
- 禁止：跨家族复制状态/算法；以 include! 平替拆分；改 help/flag/默认值/机器JSON/
  退出码；为结构改动伪造旧 fingerprint（ui_gen 布局变化经 build.rs 指纹如实失效重建）。

## 4. 验证锚点

V-01 四面盘点；V-02/03 CLI（真实进程）；V-04 327+守恒329；V-05 ui_cache_schema；
V-06 日常2196/2196+builder族164；V-07/08 renderer族170+lucide coverage；V-09/10
真实VM窗口输入/读回/GPU帧屏证据；V-11 闭包核对；V-12 守恒账；V-13 fresh/locked/
A→B/中断恢复。运行选择遵 build/ui-test-selection.md（ui_gen 族走 bare `cargo
test` 单进程）。
