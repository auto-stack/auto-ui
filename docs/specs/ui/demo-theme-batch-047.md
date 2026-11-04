# 教学示例主题迁移批次047（demo theme batch 047）

> PLAN-047 交付合同（SD-01）。B3 主题迁移第三分包（阅读/编辑/数据/专用
> 绘制，038 分包命名）且为示例生态第七批：随本批交付 **B3 十二例收口**
> （045=媒体分包、046=图像与文件分包先行）。六例——018-book-reader、
> 041-auto-edit、026-database、025-sys-monitor、024-charts、031-paint——
> 接入官方主题包的逐例规则：接入形态、逐例默认/映射与豁免、双端通道、
> 038 具名差额关闭、边界与后继。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/047-specialized-workbench-demo-theme-migration.md

## 1. 接入形态（C-01）

- 六例统一：pac `dep themes { path: "../../../packages/themes" }` +
  根 widget `use themes.themes: Auto, Solarized` + `theme_name`/`dark_mode`
  模型态 + `design { theme: .theme_name, mode: if .dark_mode … }`
  （B2 C-14 形态；不复制调色板，不新增业务菜单/handler；041 原无 model 块
  状态全在 store——主题魔法变量必须在根 widget，045/030 坑表口径）。
- 逐例 dark-first 保留迁移前有效默认：025（pac `theme: "dark"` 显式键 +
  App dark_mode=true 原已声明未读，本批接线生效）、031-paint（pac
  `theme: "dark"`）；018（无 pac 键，迁移前有效默认=生成器 html.dark +
  逃生舱 isDark=true，App 声明 dark-first 保留；host/CLI 链按 Plan-672
  继承链语义可覆盖）；041/024/026（无主题接线、无显式默认 → 官方
  Auto/light 独立默认）。
- 018 逃生舱退役：handmade `vue/src/components/ThemeToggle.vue` +
  `index.css` 覆盖随迁移删除（主题单源入 packages/themes；.gitignore
  `!vue/` 覆盖移除）；README 主题段落随实；侧栏官方切换按钮
  （`.ToggleTheme` 翻转根 dark_mode）承接 README:49 既有承诺挂载位，
  `theme-toggle-btn` 类保留原门 T9 oracle 选择器。

## 2. 逐例默认、映射与合法豁免（C-02）

| 例 | 默认 | 映射/豁免要点 |
|---|---|---|
| 018 | App dark-first（迁移前有效默认） | 全语义 token（637 映射面）；封面渐变 from-indigo-500… 与 bg-black/40 徽章为内容/叠字豁免（既有）；fmt 取整百分比恢复原门口径 |
| 041 | Auto/light | 激活 tab `bg-[#1C1D24]`、右键面板 `bg-[#16171B]` 为编辑器主题深色字面豁免（既有注释口径）；语法着色 owner=runtime code_editor（(theme, dark, accent) 键控主题集由运行时消费），不随本批迁移 |
| 024 | Auto/light | series 身份色 #2563eb/#16a34a/#d97706/#dc2626（图例点与四类图默认 colors）为数据内容色豁免——换肤不改 series→颜色映射与几何 |
| 025 | pac dark + App dark-first | KPI/图例 hex 点 #2563eb/#16a34a/#8b5cf6/#d97706 与 green/red 状态点为数据/状态身份色豁免；sysinfo 数据面不随肤变 |
| 026 | Auto/light | 全语义 token（038 实测零 hex）；green/red 状态点为状态身份色豁免 |
| 031-paint | pac dark + App dark-first | 16 色调色板/画布像素 hex 类串为文档内容色字面豁免（颜色词汇表口径）——换肤后文档像素/历史逐值不变（实测） |

## 3. 双端驱动通道（C-03）

- **VM**：`autoui_fixture`（AUTOUI_TEST_FIXTURES=1）写
  theme_name/dark_mode + 生效值经 autoui_state 断言（B2 C-03 同律）；
  024/025/026/031-paint/041 五例主题腿落在原门 desktop_mcp.py 内
  （041=T12 组，T8 退出前）。
- **Vue**：018 原门 smoke.spec.ts T9（官方切换按钮点击翻转 html.dark，
  运行时 `watch(dark_mode)` 同步 documentElement）；024 新建
  `tests/test_024_vue.mjs`（B2 016 首建入口先例）setupState 驱动 +
  `[data-auto-theme-root]` CSS-var 逐值（Solarized/light `44 87% 94%`、
  Solarized/dark `192 100% 11%` 实测上身）；025 smoke.spec.ts T7 同构。
- 渲染态快照名口径：VM 渲染后编辑器节点=textarea、首帧前 fallback 树
  =code_editor（041 T1 头注）；SVG 内图例/数据 VM 快照不可文本观测
  （024 实证）——数据值腿落 vue 轨。

## 4. 038 具名差额关闭（C-04；SD-03 接续）

| 038 具名 | 关闭结果 |
|---|---|
| F-10（018 五红） | 原门 10/10 ×2。根因四层：main.ts import 拼接（74d581d9 引入，esbuild 500 全挂）；json.from_value 悬挂（POST/PUT 静默吞）；store facade fire-and-forget（SubmitAdd 读 pre-POST state）；as int 不截断（进度 PUT 422）——均生成器缝，回归锚单测各一；pct 取整/Chapters 标题为 demo 侧漂移修定；T9 由官方机制闭 |
| F-9（024 七腿） | 按 484 后现 UI 修定：axisLabels/donutLegend/statsD 为退役 App 状态契约；VM 轨标量业务腿 23/23 + vue 轨数据腿 12/12（首建入口） |
| F-8（026 0/8） | 23/23：断言修定（Tab 短名/TreeView row/unicode 分页）+ MCP 首帧就绪等待；SQL/错误/分页/排序原语义全保持；T8 维持文档化未实现 |
| F-4（025 7/11） | VM 20/20：断言修定（2 Tab 退役负 pin/KPI 文案/图标按钮/已用 MB）+ taskkill /T 孤儿进程修复；vue 轨 smoke 修定随仓，执行被 vue 后端 a2r codegen 缺陷阻塞（见 C-05） |
| 四例主题声明补齐（024/026/031p/041） | 全部补齐并实际消费（fixture/CSS-var 实证），非仅声明 |

## 5. 边界与交接（C-05）

- **025 vue 轨**：生成后端 `sys_info.rs` 引 `types::system_snapshot/
  kill_process` 未生成（E0432）——base stash 对拍同败，存量 a2r codegen
  缺陷（038 具名 family，031-image-viewer 同族 blocked owner 口径）。
  smoke.spec.ts 修定随仓待执行；owner=a2r/vue 后端 codegen 谱系。
- 外部交付渠道（delivery-root/SDK 快照）最终消费 = Man 发行/U036 谱系
  （B2 交接不变）；本批 U036 面以工具已承诺测试面为界：官方包准入/身份
  /receipt（content-not-mtime）/篡改拒绝/近似名拒绝/snapshot 发布拒绝
  13/13（theme_package_input）。
- 非法主题状态写不崩渲染（VM 实证；admission 拒绝在 pac/生成层——
  PLAN-036 R-03 准入）；跨实例 scope 隔离 theme_scope_contract 6/6。
- 生成器随批修复（回归锚在 compiler/tools lib）：main.ts import 分行、
  json.from_value 恒等、store facade await、as int 截断、theme_tree
  dep 根感知 + 首脚手架 __AUTO_TREE_THEMES__ 注入（016 B2 vue 腿
  master 存量红实证后修复）。

## 6. 验收锚点

- 证据：PLAN-047 §8/§9 分例行（命令/退出码/计数）；worktree
  codex/plan-047-demo-themes 提交链。
- 双端对照：018 vue（html.dark 翻转 + 业务 10/10）× 041/024/025/026/
  031p VM（fixture 2×2 + 业务守恒）；024 双轨齐（VM 23 + vue 12）。
- 不变量：文档像素（031 px 哨兵跨肤逐值恒守）、编辑内容（041 tab/键）、
  数据/排序/分页（026）、流式（024 tick/seed 恢复）、真实刷新
  （025 sysinfo）跨主题输入恒守。
