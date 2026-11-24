# 虚拟列表渲染（VList / VirtualList）

> 状态：现行合同，由 `auto-ui/PLAN-126` 引入（2026-11-24），实施 revision
> `ce6d6234`（`plan-126-dev`，工作树 `D:/autostack/.wt/ui-126/auto-ui`）。本规范
> 只记录 Windows VM/Iced 路径上已测得的行为，推断或未取证处标 `[未取证]`；原始
> 证据见 `D:/autostack/plan-artifacts/plan-126/{t03,t03-scale,t04,t05}/README.md`。

## 1. 范围与命名

`VList` 为 AutoUI 组件名，Rust 类型名 `VirtualList`。首期范围：Windows VM/Iced
路径、固定行高、纵向虚拟化。几何台账、纯窗口计算与 iced probe 在
`ui/virtual_list.rs`，构建侧在 `ui/aura_view_builder/virtual_rows.rs`。

适用范围是已排序或过滤完成、可按索引随机读取的有序集合。实现接受四种列表表示
（`Value::Array`、堆 `ListData<Value>`、堆 `ListData<i32>`、`Value::VmRef`），经
`vm_bridge::list_len_of` / `list_get` 做 O(1) 长度与单元素读取，并沿用容器读的
粗粒度依赖记账（`record_dep_any`）。

明确排除：变高行与测量锚点、行列双向虚拟化、任意 `for` 的自动虚拟化、跨帧整行
`View` 缓存、远程按需取数、排序搬离 UI 线程（Q-01，另行立项）。

两个消费入口共用同一窗口计算、占位与行构建实现：

- **表格体**（025）：行循环位于 `overflow-y-auto` 列生成的隐式 `View::Scrollable`
  之内，`ForLoop` 臂自动认领。025 源不需要修改（T-02 / Q-02 结论）。
- **普通列表**：显式 `scroll (row-height: N[, list-top: M])`（别名 `scrollable`；
  `row_height` / `list_top` 拼写同样接受）。声明 `row-height` 是普通列表窗口化的
  opt-in 条件；未声明时 tracked 轨的显式 `scroll` 行为逐字节不变。

以下情形返回 `None`，调用方保留既有完整展开：检索过滤态（模型 `search` 非空）、
嵌套循环变量绑定、dotted 属性路径且非 store 字段、数据源不可按索引读取、索引读
失败（`virtual_rows.rs` 的 `vlist_search_active` / `vlist_list_value`）。

## 2. 几何合同

- 列表自身逻辑高度 = 行数 × 固定行高。pane 内容高另含列表起点之前的内容（025
  的表头与摘要），因此 pane 内容高 ≠ 列表逻辑高度。
- 屏外部分不产生行控件，由首尾两段 `View::SpacePx` 表达：首段 `start × row_h`、
  尾段 `(n − end) × row_h`；宽度缺省 `f32::NAN`（不指定），高度经 f32 承载，
  投影为 `Length::Fixed`，不受 `Container.height: Option<u16>` 的 65535px 上限。
- 构建带用 `View::VirtualMark { key, start, end }` 包一层零布局 container，挂合成
  iced id `vlist_mark:{key}:{start}:{end}`。行区间随 id 上报，**不**依赖台账「最近
  一次构建」——测量回执可能晚于下一次构建（实测错配会让行高自激振荡）。
- 窗口 = 可见行 + 两侧各 2 行 overscan（`OVERSCAN_ROWS = 2`）。可见区间是半开
  区间：上界 floor、下界 ceil，加 overscan 后夹紧到 `[0, n)`；列表整体在视口
  上/下方时为空区间（仍保留总高度）；`n = 0`、`row_h ≤ 0`、视口高 ≤ 0 或非有限
  输入一律空区间（`visible_range`，`virtual_list.rs:117`）。
- `row_h` / `list_top` 有两个来源：
  1. **显式 `row-height`**（创作侧声明，1–400px，非法值忽略）。声明后该 pane 不再
     标定、不做周期复核，视口宽变化也不作废行高；`list-top` 可选，缺省 0，首次
     构建带测量到达后只精修 `list_top`。
  2. **标定反解**。无几何时先做一次顶部 `min(n, 256)` 行的标定构建
     （`CALIBRATION_ROWS = 256`），`row_h = 构建带高 / 行数`；须两次间隔 ≥500ms 的
     样本一致（相对差 ≤0.5%）才采信，采信后每 3s 做一次复核构建纠偏。字型/世界
     稳定前的早期样本偏高（025 实测 3456 → 3477），连续帧采样会锁死早期值。
- `list_top = band_top − start · row_h − viewport_y`（`virtual_list.rs:472`）。
  `band_top` 是 iced 布局给出的窗口绝对坐标（布局不按滚动偏移平移内容，滚动只在
  绘制/命中阶段换算），故扣除视口原点 `viewport_y`。窗口带只用于重算 `list_top`；
  其平均行高不得覆盖标定或显式行高——窗口带均值可偏 ~0.5%（025 实测全局
  28.975、尾部窗口 28.82），反覆覆盖会让内容高随滚动位置漂移。行高是单值合同，
  不是逐行测量（换行漂移见 §8）。

## 3. 视口与测量

- `View::Scrollable.measure_key: Option<String>`：滚动容器内含可窗口化循环时置为
  本帧 pane 作用域 key `vlist_{seq}`。渲染器 `build_scrollable` 把该 scrollable
  **既有** iced id（`vnode_*` / `scroll_ctl_*`）登记进几何台账
  （`style_widgets.rs:2092`），不另挂 id，滚动目标、边界、快照 id 全部不变名。
- `VListProbe` 一次树遍历同时收两类证据：`scrollable` 回调收 offset / 视口高 /
  内容高 / 视口原点 / 视口宽；`container` 回调按 `vlist_mark:` 前缀解析出 scope
  key 与行区间，收构建带 rect。`vlist_pane:` 前缀仅作历史 id 的回退解析，当前没有
  控件使用该前缀。
- 回执走宿主自派发消息 `__vlist_measure`（`MEASURE_EVENT`）：host 在 update 里
  解析 JSON 落台账，不派发 VM、不改路由（`host.rs:2042`）。probe 由构建期
  `request_measure()` 触发；进入窗口化后每拍轻量 probe，以感知滚动位置变化
  （`host.rs:4611`）。
- 首帧只有 pane 几何、没有构建带时不做真实裁行；显式 `row-height` 例外——只要
  视口/偏移到手即可窗口化（首帧先做顶部 `min(n, 256)` 行 + 尾部占位，总逻辑高度
  从首帧起正确）。标定路径要等行高被两次间隔样本确认。
- 偏移变化 >0.5px 置脏重建窗口行集合（`host.rs:2047`）；不重建会沿用旧窗口，滚动
  后屏内可能整段空白（已实测定罪）。视口宽变化 >1px 作废已解出的行高、下一帧重新
  标定；显式行高不受影响。视口高变化直接改变窗口行数；尾部、缩短列表与越界请求
  按 `visible_range` 夹紧。
- 空列表不产生窗口（`plan(n=0) == None`，调用方保留完整构建的空结果）；短于视口
  的列表整表落在窗口内且无占位；列表完全在视口外时窗口为空、内容高保持。

## 4. 两轨一致性

同一窗口语义接在四处：生产轨 `convert_node_with` 的 `ForLoop` 臂
（`tracked.rs:52`）、调试/MCP 轨 `convert_for_unkeyed`（`tracked.rs:437`）与
`convert_for_keyed`（`route_memo.rs:575`）；列双胎（untracked
`tags_container.rs:38`、tracked `tracked.rs:2283`）与显式 `scroll` 双胎
（untracked `tags_container.rs:176`、tracked `tracked.rs:2596`）都压 pane 作用域，
两轨在同一窗口、同一行区间内认领同一 scope key。

窗口行由 `try_virtual_for` 经 `convert_node_with`（untracked 转换）构建，因此
**没有**逐行 `BuildProbe` / debug-id 条目：窗口化时 `for_context` 条目为 0，完整
构建为每行 1 条。该差异被用例
`plan126_vlist_windowed_rows_have_no_for_probe_entries` 钉住。不补造条目是刻意
选择：窗口行在 VTree 里的路径是 `path + [带索引, i − start]`，不是 `path + [i]`，
在构建器自身路径记账会得到指向错行的条目。

025 不受该缺口影响：`table-row` 在两模式下都委托 untracked `convert_element`，行内
probe/debug-id 本为空；差异只有「可见行的 `for` 迭代上下文」一项（inspector 面）。

MCP / 快照面：`VirtualMark` 在 vnode 转换与快照构建中透明（`vnode_converter.rs:132`、
`snapshot_builder.rs`），窗口行落在标记所在路径；`SpacePx` 在检视层是空 `Text`
叶子，快照里以 `Space` 节点带高度呈现。因此 `autoui_action press` 的
`vnode.path → find_view_by_path` 仍精确落回该行
（`plan126_vlist_windowed_row_vnode_paths_resolve_to_the_row`）。025 的表格行在开/
关两模式都没有 `[for:]` 上下文行。

## 5. 行身份与事件

- 行身份是**绝对数组索引**：行绑定取 `list_get(i)`，`index` 绑定变量为 `i`，行内
  `onclick` / `oncontextmenu` 参数在构建时按当帧绑定烘焙（布局件
  `set_layout_events`，`tracked.rs:936–947`；`table-row` 同款于
  `convert_element.rs:1264–1276`）。窗口起点不前移、不重编号。
- 数据源换序后再构建，同一窗口位置携带**新顺序**的行。025 按名重排后点击
  `WmiPrvSE.exe` → 8548、`WindowsTerminal.exe` → 31472，PID 三路互证（handler
  载荷 / `selected_pid` / `tasklist`）。
- 滚出窗口的行不可命中：滚动后用旧 vnode id 报 `VNode not found`，选中态保持。
  窗口内的 overscan 行仍可按 id 寻址（索引合同一致），但被视口裁剪、用户点不到。
- 同一 pane 的多循环隔离：`[未取证]`。源码没有「已认领即拒绝」的门
  （`try_virtual_for` 不检查 `claimed`，`plan()` 也不检查），同一 pane 内两个循环
  会共享同一 `vlist_{seq}` 台账，且 `note_plan` 会让后者覆盖前者的 total/band。
  首期按「每 pane 一个虚拟化循环」使用。
- 兄弟 pane 按帧级编号隔离：每帧最外层视图构建把 pane 序号归零
  （`enter_build_frame`），`push_scope` 只递增、`pop_scope` 不清零。同帧两个 pane
  得 `vlist_0` / `vlist_1`，各自台账与窗口，行事件不串线
  （`plan126_vlist_two_panes_do_not_cross_wire`）。修复前两者同为 `vlist_0`，实测
  120/60 行的第二个 pane 覆盖了第一个的 total。
- 选中态、菜单与后续动作仍由应用持有；VList 不改变事件载荷形状，也不介入
  `SelectProc` / `RowCtx` / `RequestKill` 等既有 handler。

## 6. 开关与回退

- `AUTOUI_VIRTUAL_LIST`：缺省启用；设为 `0` 关闭窗口化（进程内 `OnceLock`，运行
  期不可翻转）。关闭时窗口路径首判返回 `None`，不产生标记与占位；025 关闭对照为
  120 行、内容高 3689.8、clamp 2950，完整构建行为不变（数值差源于换行与实时进程名）。
- `AUTOUI_VLIST_DIAG=1`：诊断输出（`[VLIST]` / `[VLIST-PROBE]` / `[VLIST-MARK]`）。
- `AUTOUI_VLIST_ROW_H` 与 `AUTOUI_VLIST_LIST_TOP` 必须同时给出，
  `AUTOUI_VLIST_VIEWPORT_H` 必需、`AUTOUI_VLIST_OFFSET` 缺省 0：仅用于无 iced
  布局的纯构建测试与故障定位，生产缺省不使用。
- 回退规则：开关关闭、无 pane 作用域（无滚动祖先）、§1 列出的不适用数据源/绑定、
  索引读失败 → 完整构建；`n = 0` → 完整构建的空结果。
- 有 pane 作用域但尚未测得几何（或无有界视口）时不做真实窗口裁行，而是有界顶窗
  （≤256 行）+ 尾部占位；该子情形与「完整构建」不等价，其逐字节等价性 `[未取证]`。

## 7. 实测结论

- **025 实窗**（GL，`t01/probe11`，偏移 0/600/2400/7200）：`measure panes=1
  bands=1`；`row_h=28.97`、`list_top=176.20`；窗口 `[0, 22)`（顶）与 `[93, 120)`
  （尾，27 行）；pane 内容高 3663.6–3685.1（T-01 基线 3669，差值等于换行成两行的
  进程名各 +21px）；clamp 7200→2946；114 次窗口构建 / 29 次标定构建。关闭对照
  （`t01/off10`）：120 行、内容 3689.8、clamp 2950。
- **V-01 规模**（`row_h=28`、`list_top=0`、viewport 740、offset 700、overscan 2 ⇒
  窗口 `[23, 54)`）：1,000 与 10,000 行逐维相同——读取 31 行 / `list_get` 31 次 /
  构建 31 行 / vnode 37 / layout 节点 3；关闭开关为 1,003 / 10,003 vnode
  （14.9 / 123.3ms）。占位合计 = `(n − 31) × 28`，10,000 行的 279,132px 总高由 f32
  占位承载。
- **V-04 纯滚动**（同机 GL、同窗、两模式同一滚动序列 0/600/1200/1800/2400/2800、
  settle 1.2s；暂停生产者以分离刷新开销，样本 = 暂停相位全部绘制帧，窗口化 94 /
  关闭 41 帧，中位/P95/最大）：整帧 129.0 / 172.0 / 194.0ms vs 464.0 / 496.0 /
  517.0ms（**3.60×**）；`s4` iced Element 118.5 / 148.3 / 168.4ms vs 408.5 / 430.5 /
  454.6ms（3.45×）；`s3b` 构建 2.86 vs 10.03ms（3.51×）；MCP scroll 往返 80.7 /
  107.6 / 117.3ms vs 307.3 / 332.9 / 436.0ms（3.81×）。
- **V-04 刷新帧**（`s2_vm ≥ 20ms`，窗口化 50 / 关闭 82 帧，其中绘制 20 / 12）：
  `s2_vm` 100.8 vs 100.9ms（1.00×，未变）、整帧 228.0 vs 586.0ms（**2.57×**）。
  2× 目标在纯滚动与刷新两个维度达成；仅对 120 行上限内的逐帧计时成立，不外推到
  感知平滑或 >120 行。
- **残余主导**：整页 Element 树每帧全量重建，`s4` = 118.5 / 129.0ms（≈92%）；可见
  带仍占 313 节点中的 242（22 行 × 11，77.3%）。节点比天花板 1390 / 313 = 4.44×，
  实测 3.60× 为天花板的 81%。刷新残余为 VM `Refresh` + O(120²) `.ApplySort`
  ≈100ms（占窗口化刷新帧 44%），属 Q-01 范围。
- **回归**：HEAD lib `2702 run / 2689 pass / 12 fail / 1 timeout`，与干净
  `fbabf41d` 的 13 项同名同耗时（0 回归）；`--test desktop_behavior` 11/11；定向
  `plan126_vlist` 14/14、`virtual_list` 33/33、`scrollable` 9/9、`vnode` 47/47。
- **025 双模式走查 12/12**：启动（22 vs 120 行）、进程↔性能切换、表头排序 asc 与
  翻转（19ms vs 1,049ms）、结束任务对话框开/取消、滚动到底/顶/中（clamp = 内容高 − 视口高）。

## 8. 已知边界与债务

- **换行导致内容高漂移**：固定行高是均值，换行行各 +21px，内容高与 clamp 随数据
  变化（025 实测 3663.6–3685.1 / clamp 2946–2962）；这不是 clamp 缺陷，也不做逐行
  测量修正。
- **`table-row` 丢弃行样式**：`convert_element.rs:1269–1276` 构建
  `View::Row { style: None, … }`，025 的行高亮样式在 VM 轨两模式下都不生效
  （`selected_pid` / `kill_target` 正确）。先于本计划的缺陷；本计划对该文件的唯一
  改动是 `measure_key: None`（`:709`）。
- **tracked 轨显式 `scroll` 的窗口行无 probe 条目**（§4）。补保真须把
  `path / id_map / probe` 穿进 `try_virtual_for` 并记在窗口带的 VTree 路径上。
- **Vue 生成器无窗口化与 `row-height` 提示**：Vue 轨不消费本组件的窗口化。
- **右键菜单在窗口内不可编程驱动**：MCP 无 context-menu 动作，OS 合成输入不能
  进窗；菜单路径以标注的 `autoui_fixture` 注入取证（`ctx_open=true`、`CloseCtx`
  关闭），`ConfirmKill` 未按。
- **显式 `row-height` 普通列表的实窗（iced/GL）验证未做**：现有证据是 headless
  真实构建 + 025 已验的同一 probe 机制；**同 pane 多循环未隔离**（§5）。两项均
  `[未取证]`。
- **快照 `@rect` 触发面有限**：只在最后脏帧 + MCP 采集帧出现；行级判据以 vnode id、窗口行序与 `autoui_scroll_state` 为准。
