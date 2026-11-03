# 正式主题包发行渠道与生成输入收口（official theme delivery and inputs）

> PLAN-036 交付合同（SD-02）。官方主题包进入 Full SDK 安装、交付根发行，以及
> 全部真实生成/运行/复用入口的**统一准入、输入身份与失败语义**：谁在哪条渠道
> 拿到哪个快照、生成前必须校验什么、输入变化如何失效复用、失败时保什么旧状态。
> 分类：feature ｜ 状态：active ｜ 来源计划：docs/plans/036-official-theme-distribution-and-input-closure.md

## 1. 渠道与来源（channel）

| 渠道 | 落点 | 装配方 | 校验 |
|---|---|---|---|
| 唯一编辑源 | auto-ui `packages/themes/` | UI（人工/计划） | compiler `verify_theme_package`（8 具名错误） |
| Full 安装 SDK | `<install>/sdk/packages/themes/` | Man 安装事务（full profile 专属） | staging 内 python 自描述清单核验（身份常量 + 逐文件 sha256 + 路径逃逸）；坏包 = 事务保旧回滚；安装后多件/篡改由 `verify()` 全树 hash 对账具名拒绝；Core 恒零主题（`sdk/packages/themes` 进 Core 具名 forbidden） |
| 交付根 | `<delivery>/packages/themes/` | UI delivery（`auto gen --delivery-root`） | 发布原语复制+复验；`delivery-manifest.json` 新增 `theme_package` 节（owner/name/version/schema + content/identity 双摘要）；随包 `verify.py` 重算逐文件 sha256 + content digest + 身份对账 + 未登记文件 |
| App 消费 | pac `dep themes { path: <受管快照> }` | App（外部 fixture/产品） | dep 名恰为 `themes`；指向已安装/已交付快照即可，不手工拷贝、不依赖兄弟检出或绝对开发根 |

- 内容摘要算法**冻结**（OS053 固定快照与宿主策略绑定 `9fa07742…`）；身份扩展
  走加法字段（receipt `identity`、front inputs `theme/content`+`theme/identity`），
  旧 receipt/旧快照继续可读，缺新字段一律按不新鲜处理（安全重验），绝不当 fresh。
- 快照不可二次编辑、非第二权威源；升级 = UI 发新快照替换 + 消费方重验
  （与 OS053 宿主交接语义一致）。

## 2. 生成准入（admission，全部入口同律）

- **准入面**：Vue run/build/gen-only/`generate_ui` 工具链、VM 源码运行、
  Rust 生成/构建/复用。声明官方包就必须校验通过；非法（错版/缺件/篡改/解析
  失败）= 具名 `主题包生成准入失败（…）`，中止且不发半成品。
- **声明判定 = 权威解析 dep 集**（R-01/R-02 修正）：任何 dep（名无关，
  Bare/Quoted、单行/多行同权）其解析目标目录含 `theme-package.json` 即为
  官方包声明——校验通过→消费，非法→具名拒绝；声明判定不依赖 dep 名。
  行扫描的「恰名 `themes`」语法窄于模块系统的名无关消费语法，已废弃
  （曾使 `dep themesX`/`dep "themes"` 声明的非法包被静默消费、产出包值
  CSS 而零 receipt）。多个 dep 同时指向主题包目录 = 具名歧义错误。
- **未声明 = 旧行为**：无任何 dep 指向主题包目录时零主题资产、零 receipt、
  零 front-input 键。
- **生成期换包（R-03）**：Vue run/build/gen 轨在准入时冻结统一输入身份，
  receipt 写点与当前重算值比对——漂移 = `receipt 置扣`（具名失败，不签
  新收据给旧产物），与 Rust 轨 frozen-vs-current 语义同律。

## 3. 生成输入与复用（inputs & reuse）

- Vue：`<gen>/.theme-inputs.json` = `{schema:1, package:{owner,name,version,
  digest,identity}}`；**写失败 = 消费失败**（具名中止，不警告后成功）。
- Rust：统一输入身份进 `FrontReceipt.inputs`（键 `theme/content`、
  `theme/identity`）；必需输入读/验失败 = 具名错误（不签 reduced 身份）；
  生成期漂移 → receipt 置扣（沿用 910 frozen-vs-current）；910 其余规则
  （项目锚、同 basename id8 隔离、ctx/flags/target、lease/锁）不退化。
- VM：源码运行不强造产物 receipt；运行前准入证明本次所读包与输入一致。
- 失效判定：内容变化（mtime 可不变）→ `theme/content` 变化；能力/身份/集合
  语义变化（同版本）→ `theme/identity` 变化；两者任一变化即真实重新生成，
  未变则不无谓改写。旧 receipt 缺键 = 不新鲜 = 安全重建。

## 4. 失败与恢复（failure & recovery）

- 篡改/读失败/receipt 目标不可写：各入口退出码非 0 + 具名诊断；无新成功
  receipt、无旧 exe 冒新结果。
- 安装/再发行失败：Man 事务（staging→verify→swap，fail-point
  prepare/verify/swap）任何失败保旧可用集合并清理本次 staging；恢复合法后
  可再次真实消费（重装/重生成成功为证）。
- 未知非空目录不误删：UI 发布原语拒绝非空目标；Man staging/backup 以唯一名
  + 属主标记认领，外来 staging/活动目录保留并报告。

## 5. 验收锚点

- UI：`cargo test -p auto-ui-tools --test theme_package_input`（12 测）、
  `cargo test -p auto-ui-compiler --test theme_package_verify --test
  theme_package_manifest`；`cargo test -p auto-ui-tools --lib`（主题闭包/fresh
  receipt 单测；6 个既有红与本合同无关，见 Plan §9 归因）。
- Man：`python scripts/test_install_transaction.py`（9 测）；实际
  `man-install.py install|verify --profile full|core`。
- 端到端：`auto-lang/test/fixtures/modularity/generated-ui-consumer/driver.py`
  V06 腿（交付生成含 `theme_package` 节 → verify.py → B 根 locked 构建 →
  篡改具名红）；OS053 固定快照 digest 逐位一致探针。
