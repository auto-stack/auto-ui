# document-surface-boundary — 文档表面实验边界（PLAN-026 SD-01；PLAN-029 SD-01 修订）

状态：实验边界记录（探针交付 + PLAN-029 原生收口）｜来源：PLAN-026 探针
（2026-11-02，报告见 `docs/research/reader-engine-probe.md`）+ PLAN-029 收口
（2026-11-03，报告见 `docs/research/reader-native-surface-closure.md`）｜规则：
本文件只记录实验 profile 的实际承载与边界，**不承诺**通用浏览器/全格式产品
能力；正式能力由后续产品计划另立规格。

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
3. **定位回传链（双表面全链已证，PLAN-029）**：真实鼠标拖选 → 引擎 CFI
   （epubcfi）+ 逐字 quote + 全书出现序 → `.at` handler 七级校验（schema/
   会话/文档/序号/预算/kind/quote）→ 受管原文滑窗 substr 出现计数核验 →
   sidecar 落盘（backup/tmp/commit 协议）→ 冷重启后 `goTo(cfi)` 恢复原句。
   vue 与原生 VM 消费者两轨均以真实输入全链取证（N-02，双 profile exit0）。

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

1. ~~原生真实输入回传未闭合~~ **已闭合（PLAN-029）**：真实 SendInput 拖选在
   原生 VM 消费者内产出 DOM 选区 → CFI → `.at` 校验 → sidecar 全链取证
   （026 的驱动坐标索引/DPI 感知根因已修）。原生选区回传现可在实验 profile
   内声称已支持（边界仍见 §2 与 native-input spec）。
2. ~~焦点/IME/系统无障碍在原生臂未取证~~ **已取证（PLAN-029）**：真实
   MS Pinyin 组合/提交/取消/组合期快捷键、焦点往返（合成注入盲区按配对协议
   补证）、系统 UIA 树定位 + UIA 焦点 + 真实键盘翻页（N-04/06）。合成焦点
   往返的盲区（Chromium 焦点信念不随外部 SetFocus 迁移）为已记录边界。
3. 探针 profile 仅两本自制 ≤1MiB EPUB；大书/真实出版物/其他格式未验证。
4. 引擎资产需同源扁平化副本（vendor 目录需重写引用），`makeBook` 需 File
   包装；这些是实验胶水的一部分，不是公共 API。
5. **MCP oninput 阻塞（PLAN-029 具名）**：输入框键入期间 MCP 服务线程挂起
   （实验 profile）——门读取已容错；产品化需修复。

## 4. 验收入口

- `auto-reader tests/probe/engine_surface/verify_engine_surface.py
  --profile vue|vm --auto-bin <auto> --out-dir <隔离>`（PLAN-029 必需集
  N-01..12；双 profile exit0 冻结基线：vm 92 记录、vue 34 记录，见
  `docs/research/reader-native-surface-closure.md` §3）。
- 两端旧门内嵌于 N-12：`tests/basic/verify_standalone.py`（vue 28/28、
  vm 27/27 冻结基线）+ `tests/epub/verify_epub.py --auto`（22/22）。
- 探针宿主：`experiments/reader-engine-probe`（独立 workspace，非生产
  member；仅诊断对照，非 AC 入口）。

## 5. 严格构建声明（PLAN-956）

既有 `doc-surface` leaf 登记到 AURA schema（url、height、class），供严格
前端校验使用；Vue 映射为带 `data-auto-tag="doc-surface"` 的 div，应用自身
adapter 挂载引擎。此登记不增加通用文档引擎，也不开放未登记标签。
Windows native 表面仍由 `doc-surface` feature 门控；iced 标记 partial，
queue 不消费，其他后端不据此声称支持。Reader 的默认严格 Vue 构建无需
以 lenient 绕过该组件。该 schema 与 Rust 元件表、Vue 映射/责任表共同维护。
