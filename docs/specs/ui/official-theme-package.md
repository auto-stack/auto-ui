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
  **目标目录非空即具名拒绝**（未知内容零合并、零覆盖；换装由调用方以独立
  staging+swap 完成）。
- 内容摘要（`theme_package_content_digest`）：清单列出文件按路径排序后聚合
  sha256。**与绝对根无关**（同包搬根不变）、**与 mtime 无关**（内容不变即不变）。
  **算法冻结**：OS053 固定快照（auto-calc `vendor/themes`，digest
  `9fa07742…bef838e`）与宿主 `theme_family_digest` 策略绑定现值。
- 语义身份（`theme_package_identity`，PLAN-036）：在内容摘要之上纳入清单身份
  与能力元数据（owner/package/version/schema/module/families 含 default）。
  字段顺序/空白/families 排列规范化后不变；能力、身份或文件集合变化必使身份
  变化，即使同版本同内容摘要。
- 发行渠道（PLAN-036 接线）：Man Full 安装 SDK 携带 `sdk/packages/themes/`
  （安装事务 staging 内先按自描述清单核验身份与逐文件 sha256，坏包保旧回滚；
  Core 恒零主题且 verify 具名拒绝）；`auto gen --delivery-root` 交付根携带
  `packages/themes/` 快照并在 `delivery-manifest.json` 的 `theme_package` 节
  记录身份/双摘要，随包 `verify.py` 重算校验。外部消费者以 dep path 指向
  **已安装/已交付**的受管快照即可，不手工拷贝、不依赖兄弟检出或绝对开发根。
  该快照不是第二套可编辑权威源，也不需要新 registry 或网络发布。

## 5. 生成准入与新鲜度（C-05）

- 声明 `dep themes` 的 App，每次 Vue 生成/运行都会在 `<gen>/.theme-inputs.json`
  写生成输入 receipt：`{schema, package:{owner,name,version,digest,identity}}`
  （PLAN-036 起含语义身份；旧 receipt 缺 `identity` 字段一律按不新鲜处理，
  首次安全重验/重建，绝不当 fresh）。它回答
  "这次产物消费了哪个包内容"，是审计凭证，不是版本化清单。
  **receipt 写失败 = 消费失败**（生成具名中止，不警告后成功）；未声明官方包
  的 App 保持旧行为（零 receipt 文件）。
- 生成准入覆盖**全部生成入口**（PLAN-036）：Vue run/build/gen-only/
  generate_ui 工具链、VM 源码运行、Rust 生成/构建/复用——声明官方包就必须
  先校验通过（错版/缺件/篡改具名中止，不发半成品）；dep 名必须恰为
  `themes`（近似名/前缀词/其他 dep 块内的 `themes` 字样不误命中）。
- 新鲜度按**内容摘要 + 语义身份**合判，不按 mtime、不按主题名：
  - 内容变更 + 清单 hash 同步更新（合法发布，`mtime` 可保持不变）→ 内容与
    语义身份变化，生成取得新值；
  - 能力/身份元数据变化（同版本同内容）→ 语义身份变化 → 复用失效；
  - 内容变更但清单未更新（篡改）→ 准入具名拒绝；
  - 内容未变且身份未变 → 摘要稳定，不做无谓重写。
- Rust 生成轨（PLAN-036）：统一输入身份作为**必需 front input** 键入
  `FrontReceipt.inputs`（`theme/content` + `theme/identity`）；读/验失败 =
  具名错误（不静默跳过后签发 reduced 身份）；生成期间输入漂移 → receipt
  置扣（frozen-vs-current 规则沿用 910）；旧 receipt 缺键 = 不新鲜 = 安全重建。
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
- 语言侧 910 的 receipt **schema 未改**（PLAN-036 落地形态）：`FrontReceipt`
  仍为 `generated-front-receipt/v1`，主题输入以**加法键**（`theme/content` +
  `theme/identity`）进入既有 `inputs` 映射；旧 receipt 容错语义（缺键 = 不
  新鲜）由语言侧既有防读规则承载，无迁移门。
- VM/Rust 生成轨已接统一准入与输入身份（PLAN-036）；VM 源码运行不强造
  Cargo 产物 receipt，以运行前准入校验证明本次所读包与输入一致。
- 受管 source/SDK/delivery 发行渠道已接线（PLAN-036）：Full SDK
  `sdk/packages/themes/` + delivery `packages/themes/` + 清单身份节。已具名
  边界：**已安装 Full SDK 当前不含 `crates/auto-web`**（安装闭包按
  workspace metadata 决定），从已安装工具链直接 `--delivery-root` 生成会被
  producer 预检具名拒绝——发行交付的生成入口仍为工作树/driver 路径（792
  验收面）；SDK 闭包扩展归 Man 发行 owner（816/832 谱系）后继决策。
- 包内 `success/warning/info/error` 等语义槽的颜色仍由包数据决定；下游 App
  不得复制调色板，也不得把语义槽改成错误色。

## 8. 验收锚点

- `theme_package_manifest`（compiler）、`theme_package_verify`（compiler）、
  `theme_package_input`（tools，含 receipt/篡改/内容变更/语义身份/解析精度/
  非空拒绝/旧 receipt 容错用例，PLAN-036 扩至 12 测）。
- 真实生成：010 的 `index.css` 与运行时变量逐值为 Auto/Solarized 包值；
  `v09-e2e` 脚本演示"合法改色→摘要变化+产物更新""篡改→准入拒绝"。
- 渠道与输入收口（PLAN-036）：Full/real 安装 `man-install.py install|verify
  --profile full`（含 `sdk/packages/themes` 必需件与 Core 具名反控，
  `test_install_transaction.py` 9 测）；交付驱动 `generated-ui-consumer/
  driver.py` V06 腿（vendor 快照→交付生成→清单身份节→verify.py→B 根
  locked 构建→篡改具名红）；OS053 固定快照 digest 与 canonical 逐位一致
  （`9fa07742…`）。
