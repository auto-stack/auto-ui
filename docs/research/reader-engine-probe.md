# PLAN-026 研究报告：阅读引擎嵌入与定位回传双端可行性探针

日期：2026-11-02 ｜ 计划：auto-ui/PLAN-026 r1 ｜ 状态：研究交付（随计划归档）

## 1. Cohort（真实执行面）

| 项 | 身份 |
|---|---|
| auto-ui 实施树 | `.wt/ui-026/auto-ui`@codex/plan-026-dev（base 8c1b916；T-00 缝隙 85d1b2a→ca45246） |
| auto-reader | main，探针工程 18af8f1→ac6a01e（仅新增 `tests/probe/engine_surface/**`，业务 src 零改动） |
| 引擎 | foliate-js fork@3d794195（readest v0.12.16@3cc39ad7 子模块，只读副本扁平化） |
| 工具 | auto-man debug auto.exe（plan-890 面，SHA256 390e664e…）；WebView2 156.0.4314.50；iced 0.14.0（探针 crate 独立 workspace） |
| 隔离 | 全部数据面走 `AUTO_READER_DATA` / `PROBE_DATA_DIR`（plan-artifacts），用户库零写入 |

## 2. 结论（先答问题）

**有条件 go——不是 no_go_current_profile。**

1. **Vue 表面：全链 go。** 真实 EPUB 经 reader 真实导入契约建立身份 → 正常生成/装配的 AutoUI 消费页 remote-frame 接缝内挂载 foliate-js 引擎 → 真实鼠标拖选产生引擎 CFI → `.at` 消费 handler 原文核验（逐字+出现位次与独立 oracle 吻合）→ sidecar 落盘 → 冷重启同 CFI 恢复原句。30/30 检查全绿（`vue-final4/results-vue.json`）。定位回传「选择→保存→重启→恢复」在 vue 表面**端到端成立**。
2. **原生表面：嵌入可行已证，真实输入回传具名 GAP。** iced 0.14 进程内嵌 WebView2 子表面成立（`window::run`+HasWindowHandle+wry `build_as_child`，IPC 回传 exit=0）；真实引擎在同一子表面装载（spine=3，~200ms）；父子 HWND/尺寸/前台关系实证。**未闭合**：注入输入（SendInput 拖选/Ctrl+A）不被 WebView2 内容消费——自动化环境侧边界（同一管线在 vue 表面用真实输入全绿；人工拖选未验证），非引擎或桥的技术失败。
3. **无任何降低「覆盖 Readest」长期目标的暗示**：探针 profile 仅两本自制 ≤1MiB EPUB；不宣称产品支持。

## 3. 每端逐项结果

### Vue（verify_engine_surface.py --profile vue，30/30 PASS）

| 域 | 结果 | 证据 |
|---|---|---|
| V-01 装载/图/脚注 | PASS | 真实导入 sha256 身份；foliate-view 挂在 `[data-remote-frame]` 内（176-461ms）；PNG 96×64 naturalWidth 解码；章内注可见 |
| V-02 真实选区→CFI→.at→sidecar | PASS | Playwright 真实 mouse down/move/up；CFI `epubcfi(/6/2!/4/18,/1:34,/1:61)`；`.at` accepted；oracle 出现位次吻合（occ=2 = ch1 para7，全书 3 处）；HTTP+磁盘双读回 |
| V-03 冷重启恢复 | PASS | 杀实例→重启→`view.goTo(cfi)`→目标原句可见（30s 预算内） |
| V-06 身份/序号负控 | PASS | bad_session/bad_document/stale_seq/bad_schema 全拒；拒绝零落盘（前后记录数不变） |
| V-07 资源边界 | PASS | 未注册文档 404；`..%2F` 路径逃逸 404（修复前 500 粗拒绝已修为 404） |
| V-09 内容改写错锚 | PASS | 受管内容改写后原句 quote → `rejected_bad_locator/quote_not_found`，零落盘；改写句对照接受（不静默换锚） |

### 原生（driver_vm.py，vm profile）

| 域 | 结果 | 证据 |
|---|---|---|
| 嵌入缝隙 | PASS(T-03) | iced 0.14 主窗 → wry WebView2 子表面（`WRY_WEBVIEW` 子 HWND 客户区撑满）→ IPC 回传 exit=0 |
| NV-01 引擎装载 | PASS | t03-engine IPC：engine=foliate-js@3d794195、spine=3、loadMs≈200 |
| 父子/几何 | PASS | 子 HWND=宿主客户区 1024×768；`WindowFromPoint` 命中 Chrome_RenderWidgetHostHWND |
| NV-02 真实拖选回传 | **GAP（具名）** | 注入输入不被 WebView2 内容消费（详见 §4）；无 JS 代选 |
| NV-05 resize | 未完成（同输入边界） | MoveWindow 已执行；resize 后重拖选因输入边界无法取证 |
| V-04 焦点/IME | 未试（同输入边界） | 键盘注入同样不达内容 |

### 工具链真实发现（owner=auto-lang/VM 工具链，探针内以既有纪律绕开）

1. `#[api]` 形参名 `file_size` 与 `fs.file_size` native 冲突 → POST 绑定整体移位一位（echo5/echo7 最小对照）。
2. fn 调用实参含内联方法调用（如 `time.now_sec()`）→ 参数栈移位。
3. 3 操作数拼接链静默截断（epub spine 只提出第一章；逐步拼接修复）。
4. 多字节文本 `find` 结果 +1 回喂 `find` → VM panic（strings_json.rs:199，字符边界）； reader 既有「滑窗 substr」纪律绕开。
5. 普通 `.at` 的 `null` 字面量 E5501 门；`try/catch` 需catch 内提前 return。
6. `json.parse` 后访问缺失字段抛错 → 消费面 JSON 全形态化。

### 旧 Reader 守旧门（cohort 真实，历史 GAP 保持可见）

- `verify_standalone --profile vue`：**28 检查 27 PASS**。唯一 FAIL=R-05（verify_epub 全量）——其自管 8080 直连 `auto src/back/api.at` 入口在当前工具 cohort 下**宣告 21 路由但全部 404**（transpile 正常、监听正常；owner=工具链直接入口路由接线）。reader src 与门脚本在本计划零改动（9b0d513 后仅新增 tests/probe/engine_surface/**）。
- `verify_standalone --profile vm`：**27 检查 25 PASS**。除同一 R-05 外，R-06(vm)「毁损图必使原生像素 oracle 失败」rc=0（预期非零）——reader 自身负控判别在当前环境的缺口（owner=reader 后续计划）。原生 UI oracle（R-02/03/04 driver）0 FAIL。
- 本计划未修复这两个历史缺口（§10.3 纪律：他仓缺口另计划），只如实登记。

## 4. 原生输入边界（GAP 的完整口径）

- 现象：`SetForegroundWindow` 成功（fg==hwnd 经 ALT 解锁+AttachThreadInput+SwitchToThisWindow+最小化恢复三重强化）、`WindowFromPoint` 命中 `Chrome_RenderWidgetHostHWND`、SendInput 绝对坐标慢速拖选（12 段、0.12s/段）与 keybd_event Ctrl+A 均不产生 DOM 选区（页内 3s 周期 sel 探针穿透 shadow root 实证恒 `(c)`）。
- 已排除：前台切换失败、坐标错位、webview 尺寸缺省 200×200（已修复为客户区撑满）、iframe 内 `window.ipc` 缺失（探针改走 fetch/页面周期回报）。
- 定性：注入输入→WebView2 内容的路径在该环境断开；**不是引擎、桥或 .at 链的技术失败**（vue 表面同引擎同 adapter 同消费链在真实输入下全绿）。剩余工作=在真实桌面会话人工拖选验证（预计成立）或换 UA 层驱动（如 CDP 直连 WebView2 远程调试口）。
- 诚实边界：本探针不把该 GAP 记为原生 no_go，也不声称原生选区回传已证。

## 5. 迁移契约建议（给 Locator v2 / 后续产品计划）

1. **身份链**：reader 真实导入（book_id+source_hash+managed_path）→ 消费面注册（尺寸复核）→ 引擎仅从受管原件装载。身份永远由消费端对受管原文核验，不信任引擎自述。
2. **定位表示**：跨端只传「引擎 CFI + 逐字 quote + 全书出现序」；消费端以「滑窗 substr 出现计数」对受管全文重算（多字节安全的唯一已证口径）。偏移单位（UTF-16/字节/字符）不跨端裸传。
3. **拒绝面**：schema/会话/文档/序号/预算/kind/quote 七级校验，拒绝零落盘；改写内容 → `rejected_bad_locator`，保留原记录，绝不静默换锚。
4. **表面接缝**：vue 走 `remote-frame` 透传 div（PLAN-760 宿主接管契约已验证）；原生走 iced `window::run`+wry 子表面（已验证），输入路径待真实桌面会话验证。
5. **引擎资产**：foliate-js fork 钉 3d79419；服务同源扁平化副本（vendor 需扁平重写）；`makeBook` 需 File 包装（带 .name）。

## 6. 局限

- 探针 profile 两本自制书；未覆盖大型/真实出版物、PDF/TTS、云同步。
- V-08（双实例互扰/关闭断链）与 V-10（探针驱动坏 tool/占口负控）未单列执行：V-10 的坏 tool 路径语义由旧门 R-01 的同构负控覆盖（PASS），V-08 的实例隔离由全流程独立数据目录+端口约定承载但未做专门双实例用例（review F-1 补记，2026-11-02）。
- 原生拖选回传 GAP 未在真实桌面人工复核。
- R-05/R-06(vm) 历史缺口未修（owner 另计划）。
- 工具链发现 1–4 的上游修复不在本计划（绕开已证可行）。
