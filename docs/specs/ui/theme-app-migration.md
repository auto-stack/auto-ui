# 主题应用迁移（theme app migration）

> PLAN-007 交付合同（SD-03）。官方主题单源、应用默认跟随/独立回退、首批
> 两 demo 迁移与全量后继清单。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/007-autoui-tree-theme-and-examples.md

## 1. 官方主题单源（C-13）

- 官方候选族维护在 `examples/ui/themes/`（`dep themes { path: "../themes" }`
  + `use themes.themes: <Name>`）。首批两族，各含 light/dark 完整 31 公共键：
  - **Auto** —— 默认设计（中性紫罗兰基调）；
  - **Solarized** —— 风格示意（经典 Solarized 基色；非完整标准实现）。
- 消费方经 `extend("Auto", ...)` 合成自己的主题；不得把整套颜色复制进
  每个 app。名称与视觉值经首批（V-07）冻结。
- 跨包数据模块（纯 `pub theme` 声明、无 widget/store）在 vue 轨 dep 预编
  译扫描会告警 "No widget or store declarations"——非致命，属已登记边界。

## 2. 应用接入形态（C-14）

- 已迁移 app 的主题/模式选择 = design 块数据绑定（`theme: .theme_name`、
  `mode: if .dark_mode {...}`）；切换 = 普通事件改模型字段。不复制设置
  菜单、不固定 Auto/light 阻断统一切换。
- 独立运行默认官方 Auto/light；宿主跟随与跨进程通知/持久化为 AutoOS
  承接批次的显式任务（本计划未交付）。
- Vue 轨保留状态名约定：`theme_name`（str）与 `dark_mode`（bool）声明即
  接通主题热切换（与 design 块绑定同源）。

## 3. 首批与后继清单（不降级、不冒称全量）

- 首批已交付：`002-counter`（增/减/重置保持）、`005-login`（输入/必填
  校验保持）——双端（Vue + VM/Iced）换肤验收（V-08）。
- 其余 33 个编号目录 + 公共库/夹具的主题接入 = **已确认后继批次**，逐
  app 独立状态；不能以框架完成或首批完成冒称 examples 全量接入。
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
