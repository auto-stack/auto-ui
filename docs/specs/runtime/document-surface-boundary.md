# document-surface-boundary — 文档表面实验边界（PLAN-026 SD-01）

状态：实验边界记录（探针交付）｜来源：PLAN-026 探针（2026-11-02，报告见
`docs/research/reader-engine-probe.md`）｜规则：本文件只记录实验 profile 的
实际承载与边界，**不承诺**通用浏览器/全格式产品能力；正式能力由后续产品计
划另立规格。

## 1. 已证承载（实验 profile 内）

1. **Vue 表面文档接缝**：正常生成/装配的 AutoUI 消费页中，`remote-frame`
   透传 div（PLAN-760 宿主接管契约：`data-remote-frame` + 宿主 querySelector
   接管）可作为第三方文档引擎（foliate-js fork@3d79419）的挂载点；页面
   fetch 通道可达 `.at` 后端 handler（AutoVM HTTP）。
2. **原生表面嵌入接缝**：iced 0.14 应用内，`window::run(id, |win| …)` 暴露
   的 `&dyn Window`（HasWindowHandle）可直接 `wry WebViewBuilder::build_as_child`
   创建 WebView2 子表面；子表面缺省 bounds 为 200×200，必须按宿主客户区
   `set_bounds`（GetClientRect 物理尺寸）；引擎页面在同子表面内装载并经
   `window.ipc` 回传（仅主 frame 注入，iframe 内不可用）。
3. **定位回传链（vue 表面全链已证）**：真实鼠标拖选 → 引擎 CFI（epubcfi）
   + 逐字 quote + 全书出现序 → `.at` handler 七级校验（schema/会话/文档/
   序号/预算/kind/quote）→ 受管原文滑窗 substr 出现计数核验 → sidecar 落盘
   （backup/tmp/commit 协议）→ 冷重启后 `goTo(cfi)` 恢复原句。

## 2. 边界与不变量（实现者必须遵守）

1. **身份链**：文档身份（book_id+source_hash+受管副本路径）只来自真实导入
   契约；消费端核验 quote/occurrence 必须对受管原文重算，不信任引擎自述。
2. **偏移纪律**：跨端只传 CFI+quote+occurrence，不裸传 UTF-16/字节/字符偏移。
   多字节出现计数用滑窗 substr 逐位比较（find 结果 +1 回喂 find 会触发 VM
   字符边界 panic——strings_json.rs 实证）。
3. **拒绝语义**：七级校验任何失败零落盘；受管内容改写后旧 quote 必须显式
   `rejected_bad_locator`（quote_not_found），禁止静默换锚或删除原记录。
4. **资源边界**：文档代码零本机 fs/命令能力；资源服务 root 由注册记录/env
   派生 + 名称形状白名单（`..`/绝对路径不可达），未注册文档返回 404。
5. **生命周期**：会话序号单调（stale 拒绝）；实例清理 kill 树并复核端口
   释放；冷重启复用同 sidecar。

## 3. 已知缺口（不冒充能力）

1. **原生真实输入回传未闭合**：注入输入（SendInput/Ctrl+A）在 WebView2 内
   不产生 DOM 选区（自动化环境侧边界；vue 同管线真实输入全绿，人工拖选待
   验证）。原生选区回传**不得**声称已支持。
2. 焦点/IME/系统无障碍在原生臂未取证。
3. 探针 profile 仅两本自制 ≤1MiB EPUB；大书/真实出版物/其他格式未验证。
4. 引擎资产需同源扁平化副本（vendor 目录需重写引用），`makeBook` 需 File
   包装；这些是实验胶水的一部分，不是公共 API。

## 4. 验收入口

- `auto-reader tests/probe/engine_surface/verify_engine_surface.py
  --profile vue|vm --out-dir <隔离>`（vue 30 检查全绿基线；vm 臂含具名 GAP）。
- 探针宿主：`experiments/reader-engine-probe`（独立 workspace，非生产 member）。
