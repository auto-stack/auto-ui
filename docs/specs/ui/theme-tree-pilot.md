# 主题树试点（theme tree pilot）

> PLAN-007 交付合同（SD-01/SD-02）。树状主题数据、extend 合成、模式选择与
> widget 局部设计的**受限实验合同**：仅试点支持面（Theme Lab、002-counter、
> 005-login）；不宣称其余 app/全部原生部件已主题化。分类：feature ｜ 状态：
> active ｜ 来源计划：docs/plans/007-autoui-tree-theme-and-examples.md

## 1. 主题数据声明（C-01/02/03）

- 主题/token 是 UI 域的树状数据声明：`pub theme <Name> = <对象字面量或
  extend(...) 调用>`（`auto-ui-dialect` `UiStmt::ThemeDataDecl`）。普通
  `use` 导入（`use themes.themes: Auto, Solarized`），无路径型加载 DSL，
  无 Core 语句种类新增。
- 骨架键 `tokens / styles / components / modes`；其余顶层键为 app/Feature
  命名空间（`my_app: { panel: "#eee8d5" }`）。`modes` 只接受 `light|dark`
  （system 是选择策略，不是第三棵树）。
- `tokens.colors` 公共键对账既有 31 键封闭词表（`TokenName`）；31 键外的
  扩展色必须放在 app 命名空间下。token 叶值须为 `#rrggbb` 颜色或长度
  （`8dp`/`12px`）。违例为 build 期可定位错误（路径形如
  `tokens.colors.<key>`）。

## 2. extend 合成与模式（C-04/06/07）

- `extend(base, patch, ...)`：左到右；唯一命名字段递归合并、标量后胜、
  数组整体替换、输入只读。base 引用为**字符串名**（`extend("Solarized",
  "app_defaults", {...})`）——Core 解析器对对象值中的未声明标识符报
  `undefined variable`，放行裸名/点链会改通用 Core 语义（T-01 探针实证，
  超出本合同边界）；字符串引用仍是纯数据，键表由真实 use/dep 解析供给。
- 每次 `compose_mode(theme, dark)` 从原始公共数据重建（叠加
  `modes.<mode>` 差异、剔除 modes 键），不继承上次已解析值。
- 模式可用性：声明了 `modes` 却缺请求模式 → 拒绝换肤（保旧 + 诊断，
  C-07）；未声明 modes 的单模式主题对任意模式落到基座。
- **模式差异按族自足**：extend 继承的基族 modes 差异会随 merge 进入新
  族；新族改动了某模式的基值时必须显式重声明该模式差异（Solarized
  light 即此例）——否则基族 light 值会盖掉新族 base patch。

## 3. 槽引用与局部设计（C-05/08/09）

- `tokens.<ns>.<key>` 点链在 design 对象中保存为类型化槽引用，消费点
  （每帧）按当前主题+模式解析；不先复制值。
- 元素级 `design: { … }` 局部差异：槽引用随主题联动、直接字面量不受同槽变值
  影响（V-02 双腿）。局部差异只影响声明它的元素及其**声明了 design 的后代**
  （后代 frame = 最近祖先 frame + 本元素同路径覆盖）；未声明 design 的元素不
  应用任何局部值，同型两个实例互不影响。
- `design` 冻结支持面（PLAN-031 扩展）：`radius`（长度）与颜色槽
  `background` / `foreground` / `border` / `ring`（焦点环，宽 2px）/
  `placeholder`；面外槽跳过并给可定位诊断。控件（input/textarea）与容器
  （panel/card）都消费同一条合并路径。
- widget 级 `design { theme: .绑定, mode: .绑定 }`：theme/mode 是普通数据绑定，
  每帧在视图构建期求值；同帧校验后**一次**发布（事务），非法组合/未知主题名
  保留旧有效设计并给诊断；不写业务状态、不做 I/O（C-08）。选择与目录的 owner
  是 App scope，不再是进程级唯一状态（见
  [主题作用域与原子发布](theme-scope-and-publication.md)）。

## 4. 双端投影（复用既有消费面）

- **VM/Iced**：主题树进入 `style::theme::ActiveSpec::Tree` 槽；
  `resolve_semantic_rgb` 对活动树缺席的公共键返回 None，消费方沿既有
  字面量/内置主题路径取值（活动树不做逐键回退；官方族声明全部 31 键，
  实践中即树值优先、树外键保持 legacy 既有外观，C-12）。局部差异经
  `RoundedT/B(Px)`、`BackgroundColor`、`TextColor`、`BorderColor`、
  `RingColor`、`PlaceholderColor` 结构类注入。一帧构建期内读到的是安装中的
  App scope 快照（见 [主题作用域与原子发布](theme-scope-and-publication.md)）；
  `panel`/`card` 是结构容器，携带 class/style 与局部 design。
- **Vue**：树族经 `window.__AUTO_TREE_THEMES__` 并入 `THEME_PALETTES`；
  `theme_name` watch → `applyTheme` 全量 CSS 变量写（含 app 命名空间色与
  `radius-*` 尺寸槽），写入目标是**本 App 容器子树**（widget 模板根 ref），
  多 App 同文档互不覆盖；无容器时回落 `documentElement` 的 legacy 行为。
  生成期 `index.css` 的 `:root/.dark` 亦由 App 导入的官方包驱动（见
  [正式主题包与外部消费](official-theme-package.md)），值必须是
  `H S% L%` 三段式；首帧应用 **design 投影**声明的默认族
  （`__designTheme.value || theme_name.value || getActiveTheme()`；未声明
  design 时退回 `theme_name.value || getActiveTheme()`）。design 对象 →
  `:style` 绑定，槽引用为 `var(--<css-var>)`
  （如 `tokens.colors.card` → `var(--card)`、`tokens.radius.control` →
  `var(--radius-control)`）。
- 组件字段直接字面值解除该槽 token 引用（C-10）；不存在 token/theme 两套
  整体优先级。

## 5. 已知边界（不冒充完成面）

- 试点支持面：Button（native 状态面）/Input/Textarea/Panel 最小表面；
  `design` 对象支持 `radius` 与五个颜色槽（见 §3）。`style` 字段仍为类串
  （槽引用经 design 对象消费）。
- 主题槽为进程级状态（沿 DARK_MODE/ACTIVE_THEME 既有模式）；桌面宿主
  多应用并存时的 per-app 主题隔离未在试点内。（PLAN-031 起 App/组件级
  scope 已交付，per-app 隔离见
  [主题作用域与原子发布](theme-scope-and-publication.md)；本试点页只保留
  其原始范围说明。）
- Vue 轨 `theme_name`/`dark_mode` 为保留状态名约定（与 design 块绑定同
  源）；原生部件全量接入与旧 class 优先级清理属后继批次。
- 平台边界：VM 原生无键盘焦点模型（button focus 环仅 Vue 端原生
  `focus-visible`）；input/textarea 的 disabled 为真实通道（`View` 字段 +
  构造期不挂 handler/textarea 不挂 keydown），Input 另有降透明表面、Textarea 暂无。
