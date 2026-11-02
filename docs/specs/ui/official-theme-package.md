# 正式主题包与外部消费（official theme package）

> PLAN-031 交付合同（SD-03）。官方主题包从教学目录提升为仓库正式资产后的
> **单源、版本、校验、获取与生成准入**规则：包在哪里、如何被校验、外部消费者
> 如何取得固定快照、内容变化如何触发重新生成。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/031-theme-scope-publication-and-package-closure.md

## 1. 单源位置与身份（C-01）

- 唯一可编辑源：本仓 `packages/themes/`（不设第二份可编辑副本）。
- 包名 `themes`，正式版本 `0.2.0`（自试点 `0.1.0` 迁移），导入写法保持
  `use themes.themes: Auto, Solarized`。
- 内含族：`Auto`（默认设计）与 `Solarized`（风格示意），均声明 `light`/`dark`
  两种模式与 31 个公共色键 + `tokens.radius` 尺寸槽。
- 旧 `examples/ui/themes/` 只保留迁移说明，不再承载可编辑 `.at`；指向旧路径的
  引用会以缺失/校验失败的形式暴露（尚无专门的迁移提示诊断，见 §7 边界）。
- app 命名空间扩展色（`tokens.colors` 之外的 ns 子树）随包数据进入 CSS 变量面，
  但**不扩**公共 31 键；某个 App 的成功色等语义应使用包内既有的
  `success`/`warning`/`info`/`error` 槽。

## 2. 受管清单（C-02）

`packages/themes/theme-package.json` 记录：

| 字段 | 含义 |
|---|---|
| `owner` | 固定 `auto-ui` |
| `package` / `version` / `schema` | `themes` / `0.2.0` / `1` |
| `files[]` | `path` + `sha256`（逐文件） |
| `families` | 每族支持的 modes 与默认 mode |

清单是**来源/能力清单**，不是第二份颜色数据；颜色只在 `themes.at`。

## 3. 校验与拒绝（C-03）

- 校验器（`design_tokens::theme_package::verify_theme_package`）检查：owner/
  包名/版本/schema 相符、清单路径不逃逸、逐文件 sha256 一致、`themes.at` 能被
  真实解析、族声明的模式可用。
- 失败为**具名错误**：`MissingManifest` / `BadManifest` / `FieldMismatch` /
  `PathEscape` / `MissingFile` / `HashMismatch` / `BadThemeData` / `ModeMismatch`。
  任一失败即拒绝该包，不静默回落到另一个同名来源。
- 生成准入（`auto_ui_tools::theme_package::verify_app_theme_inputs`）：
  - App 未声明 `dep themes` → `Ok(None)`（零变化）；
  - 声明但校验失败 → 具名 `Err`，调用方**中止生成**，不产半成品。

## 4. 外部获取与内容摘要（C-04）

- 发行快照（`publish_theme_package_snapshot(src, dest)`）：先按官方身份校验源包，
  失败零写入；只复制清单 + 清单列出的文件到独立根（如 `vendor/themes`），再对
  快照做同标准校验——快照可脱离源根独立使用，不含 README 等非发行件。
- 内容摘要（`theme_package_content_digest`）：清单列出文件按路径排序后聚合
  sha256。**与绝对根无关**（同包搬根不变）、**与 mtime 无关**（内容不变即不变）。
- 开发期用显式 dep path 指向本仓包；外部消费者**规划**经受管 source/SDK/
  delivery 渠道取得固定版本快照——目前只交付了可调用的快照发布/校验/摘要
  原语与第二根真实构建验证，渠道接线本身尚未落地（见 §7）。该快照不是第二套可编辑权威源，也不需要新 registry 或网络
  发布。

## 5. 生成准入与新鲜度（C-05）

- 声明 `dep themes` 的 App，每次 Vue 生成/运行都会在 `<gen>/.theme-inputs.json`
  写生成输入 receipt：`{schema, package:{owner,name,version,digest}}`。它回答
  "这次产物消费了哪个包内容"，是审计凭证，不是版本化清单。
- 新鲜度按**内容摘要**判定，不按 mtime、不按主题名：
  - 内容变更 + 清单 hash 同步更新（合法发布，`mtime` 可保持不变）→ 摘要变化，
    生成取得新值；
  - 内容变更但清单未更新（篡改）→ 准入具名拒绝；
  - 内容未变 → 摘要稳定，不做无谓重写。
- 双端消费同一源：VM 由主题树解析语义色；Vue 由 `index.css` 的 `:root/.dark`
  与运行时 `THEME_PALETTES`/`__AUTO_TREE_THEMES__` 消费，值均为
  `H S% L%` 三段式，且首帧应用声明的默认族。

## 6. 已迁移消费者（C-06）

- 主题试点与首批：Theme Lab、002-counter、005-login（pac `dep themes` +
  design 绑定）。
- 第二批：003-converter、010-contact-form（本包交付；003 保留其
  `desktop_render: "remote"` 声明与既有消费方向，010 保留显式 `theme: "dark"`）。
- 音/视频等其余示例与产品仓仍以各自计划迁移，不在本文件承诺范围内。

## 7. 边界（不冒充完成）

- 不做远端 registry 发布、不做包签名体系；`registry`/`network` 不在本合同。
- 语言侧 910 的 receipt/target 结构未改：本包把"包身份 + 内容摘要"做成 UI tools
  的生成输入 receipt；是否并入语言生成缓存键属跨仓最小扩展，另立决策。
- VM/Rust 生成轨（`auto build`）尚未接同一 receipt，仍按源码解析包内容。
- 受管 source/SDK/delivery **发行渠道尚未接线**：当前交付的是快照发布
  （`publish_theme_package_snapshot`）、校验与内容摘要原语，以及第二绝对根的
  真实构建验证；由 SDK/安装器自动取得快照的通道属后继接线。
- 包内 `success/warning/info/error` 等语义槽的颜色仍由包数据决定；下游 App
  不得复制调色板，也不得把语义槽改成错误色。

## 8. 验收锚点

- `theme_package_manifest`（compiler）、`theme_package_verify`（compiler）、
  `theme_package_input`（tools，含 receipt/篡改/内容变更用例）。
- 真实生成：010 的 `index.css` 与运行时变量逐值为 Auto/Solarized 包值；
  `v09-e2e` 脚本演示"合法改色→摘要变化+产物更新""篡改→准入拒绝"。
