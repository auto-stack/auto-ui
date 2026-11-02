# UI 编译配置（UI compilation configuration）

> PLAN-018 新增合同（Design 52 计划D）。取代 `ui-iced` 实现选择器：runtime
> 的稳定 UI 基线（ui/ui-interpreter/image-pipeline/code-editor/ui-clipboard/
> ui-dialog/native-dock/native-clipboard + iced 后端与 arboard/resvg/
> tiny-skia 依赖）恒在编译；`ui-iced` 保留为兼容别名。
> 分类：build ｜ 状态：active ｜ 来源计划：docs/plans/018-ui-build-configuration-closure.md

## 1. 稳定基线（恒开）

- `auto-ui-runtime` 的 `default = ["ui-iced"]`；`ui-iced` 为**仅 feature 别名**
  （转发基线子 feature，不挂任何 `dep:`、不被任何 cfg 消费）。生成模板
  （`ui-iced = ["auto-ui-runtime/ui-iced"]` / `default = ["ui-iced"]`）、
  26 个 example 的 required-features、Man 的默认链因此零改字面兼容。
- 基线依赖非 optional（iced/iced_widget/iced_futures/iced_runtime/
  iced_tiny_skia 目标域内 windows 随基线；arboard/rfd/raw-window-handle/
  image/resvg/tiny-skia/kamadak-exif/cosmic-text/syntect/ureq）。
  代码中基线九 feature 的 cfg 门已删除；`cfg(not(feature = "ui-iced"))`
  反向 stub 臂不存在（基线恒开使"无后端工程"不可构造）。
- 证据口径：拍平前后 `cargo build -p auto-ui-runtime --message-format=json`
  账本逐位一致——657 条 compiler-artifact 消息两形态相等，按
  （包, 目标组, features）三元组集合去重后 214 个不同单元相等
  （657==657 消息级、214==214 单元级零差）；检查警告 403==403 零新增。
  基线九 feature 的 cfg 门在全 crate（src/tests/examples）为零。

## 2. 真实可选产品能力（保 optional，语义不变）

`mpv-native/mpv-gpu/mpv-widget/mpv-spike`（mpv 族 + `iced_wgpu`）、
`highlight-treesitter`（tree-sitter 15+ 包）、`autodown`
（跨仓 autodown-core，不进 SDK）、`native-dnd`（windows-core）、
`ui-headless`（cosmic 消费面）。交付投影中 `autodown` 的 `dep:` 转发随
autodown-core 剥除同步摘除（feature 名保留）。

## 3. 历史形态接收与来源

- runtime 的 no-default 形态无生产读者（T-01 实证）；拍平后 no-default
  = 基线本体减别名（行为等同默认），不再承诺"精简无窗形态"。
- 生成工程 main 的 iced 臂无条件；原 no-backend stub 臂退役。
- 来源：ui-iced 门面注释随迁自 auto-lang（Plan 015/041/414/459/617/683）；
  本收口为 Design 52 计划D，B 面（语言测试 feature 拍平/常驻 TestCase）
  见 auto-test-case/test-feature-flattening。

## 4. 跨消费者

- Man `auto` crate：optional auto-ui-runtime + `ui-extra`/`autodown`
  转发链零改（别名兼容）；cosmic demo/host-libcosmic `ui-headless` 零改。
- 轻包（compiler/dialect/LSP）零 runtime 反向边不回归（拍平闭包零 iced
  单元）。

## 附：生成 Rust UI 工程的受管构建上下文（PLAN-910 SD-04）

- `auto build/run -r rust` 经 `auto-generation-core::generated_build` 单源解析
  producer/缓存根/flags（见语言仓 generated-cargo-build-lifecycle）；本仓
  `rust_ui.rs` 不再自推 target（`shared_cargo_target_dir` 仅作兼容导出）。
- 成员包名 `<snake(pac name)>-<id8>`，cargo `[[bin]] name = "<published>-<id8>"`
  （published = pac `exe_name` 否则 snake(pac name)）；发布件
  `<member>/bin/<profile>/<published>.exe` 恢复用户可见名；`auto run` 直接运行
  发布件。`default = ["ui-iced"]` 生成模板不变。
- 前端新鲜度为内容收据（generation.json v1），mtime 退役；`FRONT_LAYOUT_VERSION`
  随生成布局变更 bump。
- 发现：`app_registry::published_native_exe`（收据优先）→ 约定路径回退；
  runtime `launcher_compiled_exe` 同序。

## 附：主题包作为受管生成输入（PLAN-031 SD-05）

- 声明 `dep themes { path: … }` 的 App，其 UI 生成前必须通过主题包准入
  （owner/包名/版本/schema/逐文件 sha256/族模式）；失败**具名中止**，不产
  半成品、不静默回落别的同名来源。未声明该依赖的 App 零变化。
- 生成输入 receipt：`<gen>/.theme-inputs.json`
  （`{schema, package:{owner,name,version,digest}}`）。`digest` 是清单列出文件
  按路径排序的内容 sha256，**与绝对根、mtime 无关**；内容变更（即使 mtime
  不变）即视为输入变化，篡改（内容改而清单未同步）被准入拒绝。
- 与 §附「前端新鲜度为内容收据（generation.json v1），mtime 退役」同口径：
  主题包是**内容寻址的受管输入**，不是按路径/mtime 判断的附带资源。
- Vue 生成产物（`src/assets/index.css` 的 `:root/.dark`、
  `window.__AUTO_TREE_THEMES__`）由 App 导入的官方包驱动，值为 Tailwind
  消费的 `H S% L%` 三段式；VM 侧由同一主题树解析语义色。包的单源/版本/
  外部获取见 `docs/specs/ui/official-theme-package.md`。
- 边界：语言侧 `generation.json`/target 结构本包未改（主题包摘要目前由
  UI tools 记录）；若需并入语言生成缓存键，属跨仓最小通用受管输入扩展，
  另立决策，不在本文件预称已覆盖。