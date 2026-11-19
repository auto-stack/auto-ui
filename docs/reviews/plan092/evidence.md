# PLAN-092 交付证据（版本绑定）

日期：2026-11-20。工作树 `D:/autostack/.wt/ui-092/auto-ui`，分支
`codex/ui-092-property-inspector`，基线 master `481a3cb5`。
依赖组：`.wt/ui-092/{auto-man@4a30483, auto-lang@670c720a8, auto-web@26d085f,
auto-down@72b0970}`（detached master、零源码改动）。二进制：组内 debug 构建
`auto.exe` + `auto-ui-tool.exe`（含本树编译器与 B-1/B-3 适配）。

## 1. 门禁结果（逐例计时入 report JSON）

| 轨 | 命令 | 结果 | 用例 |
|---|---|---|---|
| tools | `cargo test -p auto-ui-tools --test property_inspector_component` | **ok 2/2**（4.7s…5.6s） | 双生成器准入 + vendor 闭包字节等价 + 缺依赖具名失败 + r2 载荷面断言 |
| compiler 回归 | `cargo test -p auto-ui-compiler --lib` | **ok 1170/1170**（1.2s） | 含 plan092 两个永久用例（B-1/B-3） |
| native 产品门 | `python tests/product/verify_property_inspector.py --auto-bin <auto.exe> --ui-tool-bin <tool.exe> --out-dir <isolated> --backend native` | **pass：23 pass + 1 not_run（B-5 具名边界）** | V-01…V-06 见下 |
| vue 产品门 | 同参数 `--backend vue` | **pass：18/18** | 见下 |

报告：`D:/autostack/plan-artifacts/plan-092/gate/{natf6,vuef}/report-*.json`
（每例 entry/expected/observed/seconds + source/tools SHA + 截图 sha256）。
视觉实拍：`gate/natf6/v05-native-{default-light,dark}.png`、
`gate/vuef/v05-vue-{light,dark,narrow-420}.png`。

## 2. 产品门用例矩阵（native 23+1）

V-01：六 kind 由 record 渲染、087/088 子控件标记、unit kind（高级页
UnitField+mm/cm）、双目标页（路由分离）、坏 schema 具名拒绝
（unknown_kind + 面板错误 + 编辑器禁用）。
V-02：enum chip→接受、bool JSON 布尔→预览提示实际出现、string 草稿提交→
预览回投、业务拒绝（fieldErrors 反馈、预览正式值不变）、搜索只过滤展示
（无业务事件）。
V-03：高级页独立目标 editor、同 label 不同面板互不串写（B接受 align=right
不落 A）。
V-04：Storage set→get 显式读回一致（预览文本保留）；退出重开二次渲染。
V-05：owned 进程回收、明暗实拍、（vue 轨）420×560 窄窗可读、无页面错误。
V-06：数量默认 0 恢复（精确整数字符串"0"）、hasDefault=false 无恢复入口、
只读字段可读。
not_run：B5-native-nested-write（见 decisions §3 B-5）。

vue 18 例：kind 渲染、子控件标记、enum、bool 单发、numeric 步进
（B-5 的 vue 承担面）、双目标、unit、边界、恢复 0、坏 schema、窄窗、
无页面错误、会话回收。

## 3. 用例耗时（test-discipline 口径）

单例耗时均在 report JSON `seconds` 字段（native 各例 ≤ 数秒；整个 native 门
含 3 次 app 启动与清理约 3 分钟，vue 门含 vite 启动约 2 分钟）。新增门脚本
本身为本 Plan 交付物（T-05），非既有用例提速对象。每次新用例的被测行为见
§2 矩阵。

## 4. 本 Plan 落地的基线适配（B-1…B-4，均已附永久回归）

| ID | 面 | 内容 | 回归 |
|---|---|---|---|
| B-1 | 编译器 vue 轨 | input/textarea `onenter` 裸名+载荷 msg 前向输入文本（native Plan 370 同义；088 通道框 TS2554 收口） | `plan092_onenter_payload_binding_forwards_input_value`（vue::tests） |
| B-2 | 088 包源 | 7 处恒假死条件删除（行为等价，TS2367 收口） | 088 示例 auto build Completed |
| B-3 | 编译器 vue 轨 | `ui.close()` → `window.close()`（TS2304/运行期 ReferenceError 收口） | `plan092_ui_close_bridges_to_window_close`（ts_adapter::tests） |
| B-4 | shadcn 资产 | checkbox 文本槽移出指示器（标签可见、勾选图标恢复；div 包裹避免 label 双派发——vue 实证单发） | vue 轨产品门 bool 用例 + 1170 编译套件 |

087/088 归档收据影响：两者 vue 产品门（dev server 实点）结论不变；
`auto build` 生产类型检查为共同新增缺口，B-1/B-2/B-3 修复后两示例均
Completed（本轮实证）。留 merge 对账。

## 5. 运行时债务留痕（本 Plan 不修，具名待偿）

| ID | 内容 | 实证 |
|---|---|---|
| B-5 | native：父件为包 widget 时子件注入不派发（±按钮 delivered=0、嵌套 input Enter 不派发 committed；错误事件路由正常）。088 decisions §7 depth-3 债的扩展 | route 形态对照：`.NumericField\x01demo@//…`（local 父，通）vs `.NumericField/PropertyInspector#0/…`（包父，delivered=0）；smoke10/smoke11 日志 |
| B-6 | native：同页同类型双实例的面板行重投影不稳定（单实例 20/20 正常）；交互折叠同源 deferred | 单实例 probe_run 20/20 vs 双实例复现；dbg 标记显示状态/重建正确而视图成员滞后 |

应对：示例按路由分离实例（单实例挂载）；行构造 handler 内联；
gate 单列 not_run；vue 轨承担子件写入行为矩阵。

## 6. 探针与门禁工件（仓库外 scratch）

`D:/autostack/plan-artifacts/plan-092/`：`probe/`（probe-pkg、probe-app、
probe_run.py、native-run/ 20/20 报告）、`gate/`（native/vue 报告+截图）。
合并闭包时按证据政策折叠入本文件并删除。
