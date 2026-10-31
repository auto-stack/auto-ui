# UI 测试选择（UI test selection）

> PLAN-018 新增合同（Design 52 计划D）。三个 UI 测试选择 Cargo feature
> （`test-http-e2e` / `iced-layout-tests` / `test-native-dock`）退役：
> 族常驻 test 编译可见，运行由 nextest 静态边界与显式名过滤决定。
> 分类：build ｜ 状态：active ｜ 来源计划：docs/plans/018-ui-build-configuration-closure.md

## 1. 族与边界

| 族 | 规模 | 原门 | 现选择 |
|---|---|---|---|
| http e2e（`ui_tests::back_proxy_tests::http_e2e_*`） | 19 | `test-http-e2e`（空feature） | 常驻 `cfg(test)`；日常排除（default-filter `/http_e2e/`）；显式 `-E` + `threads-required=1` 串行 |
| iced GPU/布局（layout_tests/p022/p483/p490/p491/plan080/selectable simulator/terminal_pixel/terminal_input/pointer_placement/p025/virtual_scroll 门内对） | 117 | `iced-layout-tests`（+optional iced_test） | iced_test 转 dev-dependency；常驻 `cfg(test)`；日常排除（default-filter 按模块/名前缀）；串行同上 |
| native dock/真窗口（hole_region 等） | 1+2目标 | `test-native-dock` | lib 内真窗口测试 `#[ignore]`（`--ignored` 手动）；`native_dnd_e2e` 门收敛 `native-dnd`（`cargo tn`）；`native_dock_e2e`/`t5_smoke` 目标门 `cfg(all(windows,test))`，显式 `--test` 运行 |

`iced-layout-tests` 族在拍平前基线编译红（14 错误，841 迁移漂移被默认关闭
的选择器隐藏）——本合同落地时修复（UiCode/UiStmt 导入 + 两处引用形）。

## 2. 日常边界（零漂移口径）

- `.config/nextest.toml` `default-filter`：`ui_gen::`（既有）+ 上表各族。
- 日常选择实测：runtime lib **2217 条与基线逐位一致**（对照基线
  `--run-ignored all` 口径）；全可见集 2374 = 2217 + 新编译 157（136 族 +
  21 win32 dock 单元），原 2353 并集零丢失。
- 族内串行/资源条件（真 TCP、GPU 模拟器）经 `threads-required = 1`
  override 保持；平台门（windows）、原 `#[ignore]`、`--ignored` 手动入口
  全部保留；零选择不 PASS（显式过滤运行须以枚举对账，见
  plan-artifacts/plan-018/t03_*）。

## 3. 已知边界

- plan732 six_ring 的 corpus 夹具不在仓（locate_corpus 三路径皆空）：
  按作者原 skip 意图显式 `SKIPPED`，不冒 PASS。
- workspace 日常（`t`，--workspace --lib）在本计划起点编译红
  （autodown 组合 20 错误，831/900 迁移漂移）——本计划修复后
  2960/2960 绿（autodown_engine 六辅助重建、AnchorSlot 解包、
  six_ring skip）。
