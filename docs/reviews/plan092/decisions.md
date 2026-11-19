# PLAN-092 T-01 决策记录

日期：2026-11-19。执行树 `D:/autostack/.wt/ui-092/auto-ui`，分支
`codex/ui-092-property-inspector`，基线 master `481a3cb5`。探针工件（仓库外
scratch，合并时按证据政策折叠/删除）：`D:/autostack/plan-artifacts/plan-092/probe/`。

## 1. baseline

- 仓库/树/分支/HEAD：见上。087（numeric-fields）/088（color-picker）均已交付
  归档，包在 `packages/{numeric-fields,color-picker}/`，规范在
  `docs/specs/{numeric-fields,color-picker}.md`（工作树内可直接消费）。
- 依赖组：`.wt/ui-092/{auto-man@4a30483, auto-lang@670c720a8, auto-web@26d085f,
  auto-down@72b0970}`（全 detached master、零源码改动；sibling 相对 path 依赖
  原样解析）。debug 二进制：`auto-man/target/debug/auto.exe`（内嵌本树
  auto-ui 编译器）、`auto-ui/target/debug/auto-ui-tool.exe`。
- 观察路径：`packages/numeric-fields/*.at`（Tick/受控回路/十进制引擎）、
  `packages/color-picker/*.at`（canvas 手势/Dialog 组合/记录载荷）、
  `packages/file-dialog/file_dialog.at`（记录列表渲染/逐项动作）、
  `crates/auto-ui-runtime/src/ui/dynamic.rs`（Tick 订阅、input 事件实参、
  handler 分发）、`crates/auto-ui-compiler/src/ui_gen/`（ts/shadcn 发射）。
- 新增路径：`packages/property-inspector/`（T-02/03）、
  `examples/settings-inspector/`（T-04）、
  `crates/auto-ui-tools/tests/property_inspector_component.rs` +
  `tests/product/verify_property_inspector.py`（T-05）。

## 2. public interface（对照 Plan §5 r2 表）

r2 Props（`fields/values/groups/fieldErrors:List=[]`、`targetId:str=""`、
`revision/schemaRevision:int=0`、`disabled/readOnly/showSearch:bool`）与事件
（`changeRequested/previewRequested/previewCancelled/error`，请求含
`seq/baseRevision/schemaRevision/targetId/fieldId/kind/value/source`）已按
探针包 `probe-pkg/probe_inspector.at` 全量以真实 `.at` 表达并双轨生成通过。
逐项结论：

- kind→控件映射：string→`input`（点击进入编辑、Enter 提交、Escape 取消）、
  integer/number→087 `NumericField`（integer 强制 precision=0）、
  unit→087 `UnitField`、bool→`checkbox`、enum→**chip 按钮组**（见 §3 D-2）、
  color→088 `ColorPicker`。全部为真实子控件导入，无自有数值/选色实现。
- value 类型：`{"id":"showHints","value":false}` 与
  `{"id":"fontSize","value":"16"}` 同表异型——**values 记录的 value 成员保持
  JSON 原生类型**（bool→bool，其余→十进制/HEX/枚举 id 字符串），请求载荷经
  `json.stringify` 实测输出 JSON 布尔 `true`（native MCP 会话实证：
  `接受 showHints = true edit`）。不做全字符串化。
- unit kind 的值记录带可选 `unit` 成员：`{"id":"width","value":"210.00",
  "unit":"mm"}`（依据 Plan §5「通道可读：…当前值/单位」）。单位切换发
  `changeRequested{kind:"unit", value:<canonical 不变>, unit:<新单位>,
  source:"edit"}`——不扩 r2 的 source 枚举（edit/reset），单位状态作为字段
  值状态的一部分由应用原子接受（决策 U-1）。该解释如复审不认，走 needs_fix
  修订而不是静默换语义。
- record 成员缺失不猜：字段记录要求调用方提供全成员（公共核 + kind 扩展位
  置空值）；组件只按 kind 读取对应扩展成员，**不读未声明的动态成员**（避免
  缺成员语义风险）。bool 判断用 `hasDefault` 显式位，不做 truthy 推断。

## 3. mechanism probe（整链样例 + 关键失败控制）

探针链：`probe-app`（消费包）→ `probe-pkg`（r2 形状的 inspector 骨架）→
087/088 包 → 应用 handler（接受/拒绝/Storage）→ `storage.get` 读回。

**通过项（native MCP 真窗口 20/20 + vue 浏览器实点，报告
`plan-artifacts/plan-092/probe/native-run/report.json`）**：

- P1 全部 7 kind 由 record 数据渲染（昵称/字号/行距/页宽/显示辅助提示/对齐/
  文本色/只读备注 + 087/088 子控件特征：−/+ 步进、HEX/RGB/HSV 通道、单位
  mm/cm/m 切换钮）。
- P3 bool 请求为 JSON 布尔且被接受（`showHints = true`）。
- P4 enum chip → `changeRequested(center)` → 应用接受 → 受控回投；string
  点击进入编辑 → 草稿 → 提交一次（`name = 张三`）→ 正式值回投。
- P9 业务拒绝：strict 模式下 fieldErrors 反馈、正式值不变（`拒绝 name` 且
  无李四）。非法 schema：重复 id/未知 kind 具名面板错误 + `error` 事件一次，
  编辑器整体禁用（不渲染歧义可编辑字段）。
- P-reset 恢复默认走 `changeRequested(source=reset)`（`fontSize = 16 reset`）。
- P10 换目标：targetId/revision 换代后值集重置，旧目标编辑不落入新对象。
- P8 应用 Storage 写/读回一致（`storage 读回 8 项`）。
- 搜索过滤仅改变展示，不产生业务事件。
- 双轨生成：`auto build`（vue，vue-tsc 全量类型检查）Completed；native
  `auto run -r vm` 真窗口渲染 + MCP 全流程。

**机制结论（写进 T-02/03 的实现约束）**：

- **D-1 事件绑定语言边界**：view 内 `onenter: .Handler(表达式)`（带循环变量
  实参）在 native 解释器**不派发**（实证：`StrChange(f.id)` 静默无调用）。
  采用 013-todo 生产验证的形态：`onenter: .StrChange`（裸名、无载荷），提交
  处理器从组件状态 `.edit_id` 读字段身份；oninput 处理器接收输入文本
  （Plan 370 B12b）写 `.edit_draft`。带载荷子控件事件（087 committed 等）
  用 `oncommitted: .NumChange(f.id, $event)` 形态——实测派发正常（native
  MCP + vue 双轨）。
- **D-2 enum 编辑器用 chip 按钮组**，不用 `select`：schema 的
  select+selectitem 形状在 native 无实现（node_converter 的 convert_select
  消费 `options/selected_index` 另一形状；014-weather 同因改用 chip）。
  chip 组为普通 button，键盘/MCP 可寻址，选中态用主题浅底+边框结构表达
  （指南 §3），候选项 id 作为请求 value。
- **D-3 包内跨包依赖**：包 widget 内 `use { package: numeric_fields from
  "../numeric_fields" }`（**蛇形物化名**）。native 由 widget 级 use 传递装载
  （消费方 pac.at 须声明全部 dep）；vue 由 deps/ 物化同级解析。包自身
  pac.at 必须无 dep 块（VM 模块解析器不认 dep DSL，fatal at boot）。
- **D-4 Tick 投影**：主 inspector 以 `var interval int = 100` + `.Tick`
  比较 `targetId/revision/schemaRevision/fieldErrors.len()` 快照，变化时
  重建投影行并失效活动草稿（P10 实证）；Init 不投影（首个 Tick 落地，
  087 同型）。handler 间不互调（087 T-01 P8），重建逻辑内联于 Tick。
- **D-5 子控件修订透传**：NumericField/ColorPicker 的 `revision` 接收该行
  的受控修订（应用 revision 变化 → 行投影重建 → 行内 `edited` 递增），
  草稿撤销依赖 087/088 既有 Tick 语义，不在 inspector 内重写。

**基线缺陷修复（本树内最小适配，087 先例：消费 Plan 落地编译器最小适配）**：

| ID | 缺陷 | 修复 | 回归 |
|---|---|---|---|
| B-1 | vue 轨 `onenter` 裸名绑定 + 载荷 msg：模板发射 `Name()` 与载荷签名 `Name(arg0)` 失配（TS2554），且丢实参与 native（Enter 传输入文本）分叉——088 通道框全部中招 | `ui_gen/vue/{template_media,shadcn_attrs}.rs`：input/textarea 的 keyup.enter 裸名绑定在 msg 载荷 arity>0 时前向 `(($event.target as HTMLInputElement).value)` | `plan092_onenter_payload_binding_forwards_input_value`（vue::tests） |
| B-2 | `packages/color-picker/color_picker.at` 7 处死代码 `if "preview" == "committed" {…} else {…}`（088 r3 重构残留；vue-tsc TS2367 硬失败） | 删除死条件、保留恒取的 preview 分支（行为逐字节等价，native 原本恒走 else） | 088 示例 `auto build` Completed；native 探针全绿 |
| B-3 | vue 轨 `ui.close()` 无桥接臂：裸引用发射 → TS2304 + 运行期 ReferenceError（087/088 演示页 Quit 均 vue 轨点击即抛） | `ui_gen/ts_adapter.rs` ui 命名空间补 `close → window.close()` | `plan092_ui_close_bridges_to_window_close`（ts_adapter::tests） |

- 回归基线：`cargo test -p auto-ui-compiler --lib` 1170/1170（含 2 个新增
  永久用例）；修复前失败证据：`auto build` TS2554×5/TS2367×7/TS2304（
  examples/color-picker 与 examples/numeric-fields 在基线 481a3cb5 即红，
  系交付后编译器漂移 + 产品门只跑 dev server 未跑 vue-tsc 的组合盲区）。
- 087/088 归档收据影响：两者 vue 产品门（dev server/playwright 实点）结论
  不受影响；`auto build` 生产类型检查为其共同新增缺口，B-1/B-2/B-3 修复后
  两示例均 Completed。已在本记录留痕，供 087/088 owner 与 merge 阶段对账。

## 4. dependencies

| 依赖 | 交付身份 | 探针结论 |
|---|---|---|
| PLAN-087 numeric-fields | 归档 r3 6/8，交付 45a80058；包 `packages/numeric-fields/` | 直接消费：NumericField/UnitField 在循环中多实例化正常，committed/preview/error 载荷含 baseRevision/value/source；MCP 非首字段提交用 `autoui_action action='submit'` 按元素路由（README 集成约束实证有效） |
| PLAN-088 color-picker | 归档（交付见归档收据）；包 `packages/color-picker/` | 直接消费：ColorPicker 在循环内实例 + B-2 修复后双轨生成/运行正常；committed 载荷 8 位 HEX |
| 编译器/运行时 | 本树 `481a3cb5` + B-1/B-3 最小适配 | 记录循环（`for f in .fields`）+ kind 分支 + 动态记录成员绑定（`f.vs/f.vb/entry.*` 形，file-dialog 先例）双轨可用 |

无 stub：T-03 起全部子控件用上述正式包；T-01 探针包为 scratch，不进主树。

## 5. runners

| 轨 | 状态 | 命令/工作目录 |
|---|---|---|
| Win native | ✅ 实跑（本机真窗口 + MCP，20/20） | `auto.exe run -r vm`（cwd=probe-app）；驱动 `probe_run.py`（独立 Session：ready=`PropertyInspector T-01 探针`、exit=退出，不复制 FileDialog session） |
| Win vue | ✅ 实跑（vite + 浏览器实点：渲染/枚举 chip/受控回推） | 生成物 `gen/front/vue`；`pnpm exec vite --port 4392`。注：`auto run` 的 vue 封装本组未拉起 vite（进程驻留无监听），T-05 门按 087/088 形态直接驱动 vite+playwright，并单列该封装观察 |
| Linux native / vue | not_run（无 Linux runner，按 Q-04 不冒 pass） | T-05/06 沿组内 Linux（VMware）路线补 |
| tools 轨 | ✅ `cargo test -p auto-ui-compiler --lib` 1170/1170 | T-05 落 `property_inspector_component.rs` |

Q-01（DEBT-006 文本 shaping）：字段 label/值文本在 native 截图/MCP 文本中
可读（本探针 snapshot 全字段可寻址），受影响面与 087 同口径——不做逐字形
workaround，T-05 实拍归档时按 AC-06 记录。

## 6. next

- 允许进入 **T-02**：按 §2/§3 结论实现 `packages/property-inspector/`
  （schema 验证/投影/错误状态），命名与文件布局沿探针骨架
  （probe_inspector.at → property_inspector.at，蛇形包名
  `property_inspector`）。
- 未决缺口（不阻塞 T-02，T-03/05 闭环）：折叠分组交互（D-4 投影扩展）、
  `auto run` vue 封装观察项、Linux 双轨、错误/预览窄窗实拍（指南 §3.2）。
- 合同层留痕（供复审确认，非阻塞）：U-1（unit 值记录带 unit 成员 + 单位
  切换走 source=edit）为 §5 r2 的解释性落地，如需改 r2 文本则增 revision。


## 7. T-02…T-05 增补决议（2026-11-20）

| ID | 决议 | 实证 |
|---|---|---|
| D-7 | 示例双实例按路由分离（`/` 基础 + `/advanced` 高级/边界）：native 同页同类型双实例的面板行重投影不稳定（B-6），路由分离保持单实例挂载，双面板语义经导航呈现 | 单实例 probe 20/20 vs 双实例复现陈旧按钮；路由化后 gate 全绿 |
| D-8 | 行构造 handler 内联（B-6）：模块 fn 返回的记录在 native 视图绑定不重投影；纯计算（schema 扫描/值错误）保留模块 fn（pi_schema_error/pi_value_error），结果经模型变量/内联字段落视图 | 内联后单实例重投影恢复、双实例路由化门全绿 |
| D-9 | 子控件事件一律 `$event` 单参绑定，行身份经 revision 槽位编码（`(rev+1)*1000+行序`，slot_ids 反解）：native 子件事件带 view 实参时静默不派发（D-1 扩展） | 087 错误事件（$event-only）路由成功 vs `.NumChange(f.id,$event)` 静默；槽位化后 gate 通过 |
| B-4 | shadcn checkbox 资产：文本槽移出 CheckboxIndicator（原顶替勾选图标、标签不可见）；外层 div 而非 label（label 转发点击造成双派发，一次点击两条请求） | vue 实测 label 渲染 + 单发；编译套件 1170/1170 |
| B-5 | VM runtime 债（不修，留痕）：native 包父层子件注入不派发——±按钮 delivered=0、嵌套 input Enter 无 committed；错误事件路由正常。088 decisions §7 depth-3 债扩展 | route 形态对照 smoke10/11；gate 单列 not_run，vue 轨承担该行为矩阵 |
| B-6 | native 同类型多实例视图重投影不稳定 + 动态记录成员视图条件不重渲染：交互折叠 deferred（v1 静态分组标题）、双实例路由分离、行构造内联 | dbg 标记：状态/重建正确而视图成员滞后；单实例无此象 |
| U-1 落地 | unit 值记录带 `unit` 成员；单位切换发 `changeRequested{kind:unit, value 不变, unit, source:edit}` | gate（native 高级页 + vue）通过 |

## 8. 最终门禁（T-05 收口）

- tools：`property_inspector_component` 2/2；compiler lib 1170/1170（含
  B-1/B-3 永久回归）。
- native 产品门：23 pass + 1 not_run（B5-native-nested-write，Q-04 单列）。
- vue 产品门：18/18（含 numeric 步进、unit、坏 schema、420 窄窗、无页面错误）。
- 报告/截图：plan-artifacts/plan-092/gate/{natf6,vuef}/；计时与矩阵见
  evidence.md §2/§3。
- V-04 读回口径：同进程显式「读回」按钮（set→get→回投）；应用级退出重开
  验证二次渲染（V05）。native storage 跨进程生命周期属 VM 存储层行为，
  不作本包合同断言。


## 9. 复审修复轮（F-1/F-2 收口，2026-11-20）

**F-1（gate 计时分相）— 已修复**：`verify_property_inspector.py` 增
`Case.reset_stamp()`（准备段锚重置）+ 快速清理轮询；会话启动/页面水合计入
准备段不归属功能用例。复跑：native 23 pass + 1 not_run、vue 18/18，全部用例
≤10s（native 最长 6.57s=V05-cleanup（清理相）、vue 最长 4.46s）。

**F-2（交互折叠）— 带证据改道（B-6 深化）**：复审指出原 B-6 诊断与
fn-record 缺陷混淆。修复轮在**内联行+编译器 B-7 修复后的干净基线**上系统
复测六种实现形态：

| # | 形态 | 结果 |
|---|---|---|
| 1 | 视图 `List.contains(循环成员)` 条件 | **UI 校验器拒绝**（受控 bisect A/B：同文件去/留对照） |
| 2 | 投影期过滤 + 组记录成员条件（`g.collapsed`） | Tick 重建执行但组级结构不重渲染（snap_fold2） |
| 3 | 列表变异 unfold（整体替换） | 不传播 |
| 4 | 列表变异 unfold（pop+回填） | 不传播 |
| 5 | toggle_seq 奇偶 + 行 hidden 成员（行级条件） | fold（隐藏）传播、unfold（恢复）不传播 |
| 6 | 标量 `.folded_group` + 组级兄弟互换（edit_id 同型） | 不传播 |

结论（D-10）：native 视图差分对「组级结构变化/被移除子树的再插入」不传播；
仅「现存子树内的标量驱动互换」（edit_id 型）与「push 驱动的 contains 首折」
可见。交互折叠在本机 native 不可靠交付，**v1 维持静态分组标题**（9d112489
语义）；折叠能力依赖 VM 视图差分修复（与 B-5 同族的 runtime 债），归档时
入 debts 登记由 runtime owner 排期。vue 轨视图响应正常，折叠可在 VM 修复后
双轨补齐（合同文本不变）。

**B-7（新基线适配）**：视图条件串后处理补 `.contains(x)` → `.includes(x)`
（空格/紧凑双形态；此前 is_empty/length/find 有映射而 contains 缺失——
PropertyInspector 折叠条件 vue 轨 TS2339）。永久回归
`plan092_view_condition_contains_maps_to_includes`；编译套件 1171/1171。

F-2 收口口径：AC-03 的「折叠」子项以 B-6/D-10 边界 + 六形态证据改道为
runtime 债（非本包可修）；恢复默认/搜索/草稿/只读/无隐式写入等其余子项
双轨门全绿。是否接受该改道由复审裁定；不接受则 needs_replan 修订 AC-03。
