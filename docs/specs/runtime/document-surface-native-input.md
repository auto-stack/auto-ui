# document-surface-native-input — 原生输入链与聚合门契约（PLAN-029 SD-02）

状态：实验 profile 已证契约｜来源：PLAN-029（2026-11-03，报告见
`docs/research/reader-native-surface-closure.md`）｜边界：实验消费 profile
（探针语料 ≤1MiB EPUB×2）；通用浏览器/全格式能力不在本规格内。

## 1. 原生文档表面 API 与事件身份

1. **表面声明**：`.at` 页以 `doc-surface (url: .model_url, height: N)` 声明
   实验文档表面（compiler feature `doc-surface`，windows）；vue 轨同 tag
   退化为透传 div（`data-remote-frame` 宿主接管契约不变）。
2. **宿主形态**：弹出窗归属宿主（WS_POPUP+owner+TOOLWINDOW/NOACTIVATE，
   注册类 `AutoUIDocSurfacePopup`），逐帧同步组件物理矩形（移动/缩放/z
   跟随 owner）；WebView2 bounds 为弹出窗相对 (0,0,w,h)。
3. **事件身份**：选区经 adapter（assets/engine-adapter.js）→ 真实 DOM range
   → CFI + 逐字 quote + occurrence → 消息 v1（schema_version/kind/
   locator_select/session/book/hash/engine/href/cfi/quote/occ/seq/ts）→
   `.at` handler 七级校验。指针可达性探针（kind `input_probe`）为诊断面，
   不入正链。
4. **DPI 几何纪律**：弹出窗物理矩形 = 组件逻辑 bounds × 窗口 scale，逐帧
   跟随；DPI 变更时跟随**窗口比例**（popup_w/win_client_w 为缩放不变量——
   oracle 与实现同口径；物理保持/逻辑保持两种 winit 行为下均成立）。

## 2. 原文核验与拒绝语义

1. 身份链/偏移纪律/拒绝语义/资源边界/生命周期：继承
   [document-surface-boundary §2](document-surface-boundary.md)（不信任
   引擎自述；七级校验失败零落盘；stale 单调；kill 树复核端口）。
2. **零写字节级取证口径**：负控前后 sidecar 文件字节级快照比对（基线先经
   计数稳定等待——selectionchange 连发中间态在案）。
3. **变异判据**：恢复落点须异位于真 CFI 落点（位置指纹）——文本可见性
   口径会被邻 section 预渲染污染，不可用。

## 3. 布局 / 焦点 / IME / 无障碍

1. **重排**：vue 轨以**跌破列钳制的视口 resize** 触发（分页器 ResizeObserver
   一级通道；body 字号注入与 gap 属性不经分页器观察——在案）；vm 轨字号/
   resize/DPI 全通道。锚定纪律：forward-only 翻页前锚段首（目标 CFI 在重排
   后会落到段尾钳制）；拖选前片段级行盒精化（跨列 union 横跨列间隙落隙）。
2. **焦点**：表面→输入框方向的合成注入存在盲区（Chromium `hasFocus` 信念
   不随外部 SetFocus 迁移 + iced 组件焦点不随 OS 焦点恢复）——按配对协议
   补证（CDP hasFocus 断言 + 在案人工目击组合，门记录注明两半来源）。
   输入框→表面方向合成路径已证。
3. **IME**：真实 MS Pinyin 组合（Space 首选转换提交）/Esc 取消/组合期 Enter
   仅提交原文不触发页面命令（子序列+长度判据）。iced 主窗无经典 HIMC
   （TSF 路径）。
4. **无障碍**：`--force-renderer-accessibility` + 系统 UIA 树定位正文
   （ControlType.Text←Document 链）+ UIA SetFocus + 真实键盘翻页往返；
   非 DOM ARIA、非截图替代。
5. **已知阻塞（具名）**：输入框 oninput 键入期间 MCP 服务线程挂起（实验
   profile）——门读取有界重试容错；产品化需修复。

## 4. 失效 / 关闭 / 双实例

1. **生命周期**：WM_CLOSE 真实关闭路径 10s 内进程/端口/弹出窗释放（无
   taskkill 补偿）；取消腿弹窗隐藏（webview 对象进程内保留语义如实记录）；
   崩溃腿强杀后进程/端口 10s 内死亡、无 webview 孤儿增长。
2. **双实例**：`-B/-F` CLI 覆盖 + `AUTOUI_MCP_PORT` 钉端口（服务端 busy
   回退链下双实例必须钉）；A/B 独立书/会话/记录，A 关闭后旧事件
   `rejected_bad_document` 零写入，新 A 不接旧会话，B 跨 A 全生命周期扰动
   继续真实选择保存。
3. **坏 tool/占口**：`AUTO_UI_TOOL_BIN` 缺失路径 → 具名失败且 MCP 永不
   宣告（无静默回落）；占口（独占绑定）→ 应用不服务、占口者不被杀。

## 5. 严格聚合门

1. 入口：`python tests/probe/engine_surface/verify_engine_surface.py
   --profile vue|vm --auto-bin <auto> --out-dir <隔离>`；必需集 N-01..12
   （REQUIRED 清单，profile 适用性显式）；父用例=自身+子项最坏推导。
2. **非零语义**：缺项/重复/GAP/FAIL/not_run/未声明根/跨 profile 记录/坏
   cohort/ENV 不可用全部非零（exit1/2/3 区分）；exit0 当且仅当 findings=0
   且 not_run=0。
3. **out-dir 守卫**：拒用户根/Desktop/盘符/无标记非空目录/reparse 点；
   safe_rmtree 限 scope 拒 reparse 跟随。
4. **DPI 交互门双模式**：`PROBE_DPI_INTERACTIVE=1`（用户经系统设置切换）或
   `PROBE_DPI_AUTO=1`（驱动经 UIA 驱动真实设置应用切档——SelectionPattern
   消息级操作，不动物理鼠标）。oracle=窗口比例不变式（popup_w/win_client_w）。
5. **夹具纪律**：驱动可自管 out-dir 夹具；反例经夹具副本注入时必须
   `--no-regen`/`overwrite=False`（驱动默认重生成会静默还原变异）。
