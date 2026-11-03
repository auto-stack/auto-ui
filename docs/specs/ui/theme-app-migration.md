# 主题应用迁移（theme app migration）

> PLAN-007 交付合同（SD-03），PLAN-031 修订（SD-04）。官方主题单源、应用接入
> 形态、首批/第二批已迁 app 与全量后继清单。分类：feature ｜ 状态：active ｜
> 来源计划：docs/plans/007-autoui-tree-theme-and-examples.md、
> docs/plans/031-theme-scope-publication-and-package-closure.md

## 1. 官方主题单源（C-13）

- 官方族维护在**本仓 `packages/themes/`**（`dep themes { path:
  "../../../packages/themes" }` + `use themes.themes: <Name>`），首批两族各含
  light/dark 完整 31 公共键：**Auto**（默认设计，中性紫罗兰基调）与
  **Solarized**（风格示意，经典 Solarized 基色；非完整标准实现）。
- 旧 `examples/ui/themes/` 已退役为迁移说明，不再是可编辑源；包身份/版本/
  清单/校验/外部获取见
  [正式主题包与外部消费](official-theme-package.md)。
- 消费方经 `extend("Auto", ...)` 合成自己的主题；不得把整套颜色复制进每个
  app。名称与视觉值经首批（V-07）冻结。
- 跨包数据模块（纯 `pub theme` 声明、无 widget/store）在 vue 轨 dep 预编译
  扫描会告警 "No widget or store declarations"——非致命，属已登记边界。

## 2. 应用接入形态（C-14）

- 已迁移 app 的主题/模式选择 = design 块数据绑定（`theme: .theme_name`、
  `mode: if .dark_mode {...}`）；切换 = 普通事件改模型字段。不复制设置
  菜单、不固定 Auto/light 阻断统一切换。
- 独立运行默认官方 Auto/light；显式 pac/CLI 默认（如 010 的 `theme: "dark"`）
  按新契约保留，不暗改成未显式声明 App 的默认。
- Vue 轨保留状态名约定：`theme_name`（str）与 `dark_mode`（bool）声明即
  接通主题热切换（与 design 块绑定同源）；首帧应用声明的默认族。
- 宿主侧提供选择输入（follow/override）与结果读回端口；AutoOS 的通知/
  持久化/重连与真实 OS 事件由 AutoOS 独立合同承接（本目录不交付）。

## 3. 已交付批次与后继清单（不降级、不冒称全量）

- 首批（PLAN-007）：`002-counter`（增/减/重置保持）、`005-login`（输入/必填
  校验保持）+ 试点面 `theme-lab`（两族 × 明暗、局部设计隔离、业务状态保持）。
- 第二批（PLAN-031）：`003-converter`（两向换算；保留 `desktop_render:
  "remote"` 声明与既有消费方向）、`010-contact-form`（无必填/格式校验、
  Submit→submitted、FAQ 列表；回执走包内 `success` 语义槽）。
- 第三批（PLAN-033）：B1 余六全部接入——`001-helloworld`、
  `004-profile-card`、`006-hero-section`、`007-stats-board`、
  `008-pricing-table`、`009-article-feed`（逐字段映射/豁免、双端通道、
  隔离与包 receipt 见
  [基础示例主题接入](basic-examples-theme-adoption.md)）。**B1 收口**：
  002/005/003/010 + 本批六 = 10 个基础示例已迁。
- 其余 **24** 个编号目录（B2 十二 + B3 十二）的主题接入 = 已确认后继
  批次，逐 app 独立状态；不能以框架完成或前几批完成冒称 examples 全量
  接入。既有身份（002/005/003/010 与本批六）不并入后续批次重复迁移。
- **渠道与支持面（PLAN-036 交接，不扩历史交付）**：B1 十例的既有交付
  不变；新渠道（Full SDK `sdk/packages/themes/`、交付根 `packages/themes/`、
  全入口准入与统一输入身份）自本计划起对后继批次可用，采用新渠道的后继
  验收以后继计划 own 合同为准。交接对象与状态：**OS053** 已归档交付
  （calc `vendor/themes` 固定快照 digest `9fa07742…` 与本源逐位一致——
  本包冻结内容摘要算法，宿主策略值不受影响；升级 = UI 发新快照 +
  calc 重验）；**语言 918** 已归档（其 canonical
  `delivery-admission-and-boundary-policy.md` 生效：本包主题输入为 UI
  owner 加法字段、无新 cargo 依赖边；本包落地后 modularity audit baseline
  按其 §3 由变更 owner 重采样）；**B2** 尚未立约——可用入口/限制：单包
  声明（`dep themes` 恰名）、`vendor/themes` 或已安装快照 path、生成输入
  receipt 与移根/升级验收模板见
  [正式主题包发行渠道与生成输入收口](../build/official-theme-delivery-and-inputs.md)。
- 批次顺序按真实依赖：基础控件/表单 → 列表/卡片/导航 → 复杂业务/编辑/
  媒体；共享 `stylekit` 与验证夹具（`remote-frame-e2e`、032）按来源保持
  义务，不按业务 app 强套主题。
- AutoOS 产品 app 不在本目录、不受本合同治理；其接入以后继独立批次按
  现行注册表/来源仓逐个交付。

## 4. 验收口径

- 双端同源：同一 .at 源码在 Vue 与 VM/Iced 呈现同一官方族色板
  （V-03/V-08 像素/CSS-var 交叉证据；Solarized light `#fdf6e3`、dark
  `#002b36` 等基色逐值核对）。
- 换肤保持业务：计数/输入/校验状态与组件实例身份跨主题+模式往返保持
  （V-04/V-08）；非法主题/模式组合保旧有效设计（V-05 事务面）。
- 已知既有差异：003 的 VM 像素抓取只显示表面（迁移前基线同样如此，属
  既有抓取/布局缺陷），其业务正确性以 MCP 快照与换算 oracle 判定。
