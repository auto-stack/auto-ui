# 示例归属（examples ownership）

> PLAN-007 交付合同（SD-04）。教学/框架示例归 auto-ui 单源、来源映射、
> 依赖定位与测试义务保留。分类：structure ｜ 状态：active ｜ 来源计划：
> docs/plans/007-autoui-tree-theme-and-examples.md

## 1. 单源归属

- AutoUI 教学/框架示例单源在本仓 **`examples/ui/`**（37 目录 + 顶层
  README/clean.ps1；含共享库 `stylekit`、验证夹具 `remote-frame-e2e`、
  两个不同身份的 `031-image-viewer`/`031-paint`、编号验证例
  `032-handler-async-probe`）。不新建 ui-examples 仓。
- `examples/theme-lab/`（与 ui/ 平级）为主题试点应用，同属本仓示例面。
- 语言仓 `auto-lang/examples/ui` 仅保留定位 README；旧位置无活体 `.at`
  副本、无链接、无目录搜索顺序回退。

## 2. 来源映射与身份（2026-11-02 T-02 迁移）

- 源：auto-lang `examples/ui` @ `5923d4dee`（381 tracked 文件，
  3,263,775 字节；冻结清单哈希 `979b55ad9532f1f7…`
  记录于归档 Plan-007 §9.1；清单内容即 T-02 落地树本身）。目标：本仓
  T-02 提交 `1478193`，**逐文件字节等价**
  （hash 对账 381/381）。
- 跨仓历史以「源删除提交（auto-lang `4061af61c`）+ 目标新增提交」对读；
  不重写两仓历史。旧路径 → 新路径为同相对结构直映射
  `auto-lang/examples/ui/<rest>` → `auto-ui/examples/ui/<rest>`。
- 本仓 `.gitignore` 增补：`examples/**/{gen,rust-workspace,.auto,.am}/`
  新生成产物不入库（证据入 plan-artifacts，merge 时折入 Plan）。源侧
  既有 tracked 的 `015-notes/rust-workspace/*.toml` 等按来源守恒保留
  （T-02 字节等价迁移的一部分），不因该规则追溯去跟踪。

## 3. 依赖定位（读者解析序）

- 示例内跨包依赖以 pac `dep { path }` 声明为唯一通道：
  `stylekit`（同目录相对）、`themes`（`../themes`）、041 的 `bps`
  （**语言仓资产**：`../../../../auto-lang/packages/blueprints`，组内
  兄弟/平铺主检出双形态可解析，L1 零副本）。
- 本仓读者的解析序：`AUTO_UI_EXAMPLES_ROOT` env（第二绝对根验证）→
  本仓根 `examples/ui`。跨仓残余读者（capability-tests、`auto` CLI 构建
  产物）仍走语言仓定位（`AUTO_LANG_ROOT` → 组内兄弟 → 主检出）。
- `auto run` 对**无 back 契约**的纯前端 app 跳过 Rust 后端服务
  （`start_api_server` 前置 `resolve_back_api` 门）——此前靠旧主检出的
  预生成工作区成员兜底，新检出全数误败（T-05 修复，具名）。

### 3.1 TreeView 消费者（PLAN-085）

- **单源**：`packages/tree-view` 是 TreeView 的唯一实现。`examples/ui/026-database`
  与 `examples/ui/041-auto-edit` 改为 `use { package: tree_view … }`，各自
  `components/treeview.at` **已退役**；全仓不再存在第二份 `treeview.at`。
- **示例保留自身业务代码**：026 的 `components/tree_util.at`
  （`filter_tree`/`collect_ids`/`toggle_id`）与 `filetree.at`、041 的 `editor_store`
  及 `use bps.navigation.filetree.tree_util: toggle_id` 均不变。`tree_icon.at`
  在两者中仍在被 `filetree.at` 使用，故保留。
- **通用树工具的 owner 不变**：仍是语言 blueprint
  `bps.navigation.filetree.tree_util`；包内 `tree_view_util.at` 只持 UI 交互投影，
  不构成第二真源。
- **接入形态**：两消费者与自托管演示 `examples/tree-view` 均把树托管在 **App 根
  widget**。包路径深度与同文件已工作的 `pagination`/`file_dialog` 一致
  （`src/front` 起五级到仓根）。
- **已知形态差异（未修复，具名）**：树托管在**路由页**时，真实窗口里行可渲染但按下
  不写页面状态（库路径正常、无诊断）。该差异由
  `crates/auto-ui-runtime/tests/plan085_routed_callbacks.rs`（2/2，正反两面）守护，
  不混入本包产品门；两个真实消费者均不受影响。相悖的旧禁令已在
  `docs/reviews/plan085/decisions.md` 的「决策反转」条目中正式撤销。
- **018/027 不构成消费者证据**：二者无任何 `TreeView (` 调用位，仅含从未实例化的
  副本，已随 PLAN-085 清理；不得据此声称"四示例接入"。

## 4. 测试义务（迁移不消灭断言）

- 已回接新根的分类读者：UI registry 扫描/策展集（28 测试）、desktop
  protocol 覆盖门（coverage 21）、vm_bridge/layout/renderer、
  ui_snapshots（workspace_path）、媒体夹具（auto-media-core 改用仓内
  `crates/auto-media-core/tests/fixtures/` 固定输入 + PROVENANCE 标注，
  非活体镜像）、auto-man autodesk（组内兄弟回退）。
- 语言侧：a2ts 探针的 tsc.js 绝对候选随语料改指 auto-ui（gen 产物在
  新根再生，缺席时探针照旧 skip）；AP-16/ACE 语料义务不因迁移消失
  （对应职责组 owner 不变）。
- 已知边界（具名，非本计划红）：`ui_snapshots` insta 基线在迁移前后
  同样红（主检出旧语料可复现，属快照漂移既有债，owner=UI 测试基线维护
  者）；`013-todo` Rust 生成问题按 905 交接维持具名未闭，本迁移不对其
  归因（源文件字节等价迁移，身份中性）。
