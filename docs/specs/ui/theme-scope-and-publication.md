# 主题作用域与原子发布（theme scope and publication）

> PLAN-031 交付合同（SD-01）。App/session 与组件实例两侧的主题**归属、身份、
> 选择发布与失效**规则：一个进程内多个 App、同一 App 多窗、组件局部设计如何
> 各自持有并解析自己的有效主题，以及候选失败时如何保旧。分类：feature ｜
> 状态：active ｜ 来源计划：docs/plans/031-theme-scope-publication-and-package-closure.md

## 1. 身份与作用域（C-01）

| 对象 | owner / 寿命 | 约束 |
|---|---|---|
| Catalog（主题目录） | App/session 持有 | 按**包 owner/版本/内容摘要/主题名**索引；同名不同来源不得互相冲掉另一个 App 的目录 |
| App ThemeSelection | App session | 同一 App 多窗默认共享；另一 App 独立；显式 preview 建子 scope |
| DesignScope | 稳定组件实例 | 借父 scope 有效快照 + 本元素局部差异；同型两实例互不影响；根销毁即撤销 |
| ResolvedDesign / revision | scope 内 | 内容/模式/来源/局部差异共同决定；**名字不足以证明相等** |

- 选择与目录**不再**是线程局部唯一状态（`style::theme` 的 TLS 仅保留 legacy
  单应用路径）。运行时 `DynamicComponent` 持有 `ThemeScope`，在 `view*` 入口
  以 guard 安装，Drop 恢复；嵌套/重入/失败路径都按栈序恢复，未安装 scope 时
  读 legacy 槽（零变化）。
- scope 有全局单调分配的身份（`ThemeScope::id`）。内容寻址缓存（全局缓存、
  stream 缓存）的键必须并入该身份，否则换挂另一个 scope 但 revision 数值相同
  会命中旧样式。

## 2. 选择与原子发布（C-02）

- 候选必须**先完整校验，再一次提交**：family（树/内置/合成）、mode、来源
  revision 与局部 diff 全部合法后才写 family/mode 并推进 revision；同一帧内
  family+mode 变化只 +1 revision。
- 任一必要字段非法即**整候选拒绝**：不写 mode、不写 theme、不动注册目录、
  不覆盖旧快照（诊断给出主题名/槽路径）。渲染不出现"新 mode + 旧 family"的
  混合半发布。
- 首次挂载失败 = 明确无有效设计（不是"部分生效"）；失败后的合法候选可恢复。
- **同名内容变化必须重新发布**：主题身份按「形态 + 名 + 树内容」比较，而不是
  只比较名字。同名不同内容 → 视为新 revision；同名同内容 → 不推进 revision、
  不刷新 epoch（合法控制）。内置/树同名不同来源不算同一身份。
- design 选择器求值失败时**整帧拒绝**，不允许"先改 mode 再 continue 发布剩余
  半边"的顺序。

## 3. 宿主/受控输入与优先级（C-03）

优先级（高→低）：显式受控启动/用户 App override → App 声明 follow 的宿主
选择 → 新主题 App 声明独立默认 → legacy 回退。

- 输入值分三态：`Unset`（未设置，低优先来源可补）/ `Invalid`（非法：整帧
  拒绝，**不**静默降到另一来源）/ `Value`。
- `follow_host=false` 时宿主 family/mode 不生效；App override 恒生效，不受
  follow 门控。
- 该通道只承载 UI 可调用的选择输入与结果读回；不实现 AutoOS 消息、持久化或
  真实 OS 事件（见 §6）。

## 4. 局部设计（design 作用域）（C-04）

- 元素级 `design: { … }` 只为**声明它的元素**合成有效 frame（最近祖先 frame +
  本元素同路径覆盖）；未声明 design 的元素不应用任何局部值，也不把祖先的
  局部值绘制出来。
- 合成数值在消费时按当前 scope 的主题+模式解析（槽引用）或原样生效（字面量）；
  字面量不随同槽变值联动。
- 冻结支持面：`radius`（长度）+ 颜色槽 `background` / `foreground` /
  `border` / `ring`（焦点环，宽度按 2px）+ `placeholder`。面外槽只跳过该槽并
  给可定位诊断，父级/旧有效值保持。
- 同族后写胜：本元素 design 覆盖 class 中同类声明；radius 以 T/B 成对保留
  （四角同径），不得逐类删除只剩一半。

## 5. 双端投影（C-05）

- **VM/Iced**：一帧构建期内安装当前 scope；builder 的样式解析与语义色解析
  都读该快照。控件（button/input/textarea/panel 等）与容器消费同一快照；
  `panel`/`card` 是结构容器（携带 class/style 与局部 design），不再落
  unknown fallback 丢样式。
- **Vue**：vars 写到本 App 容器子树（widget 模板根的 `ref`），不是两个根共享
  `documentElement`；容器根自身在 dark 模式下也接收 dark 变量组，`__isDark`
  取 html 或容器任一为 dark。无容器/无主题 widget 保持 document 级 legacy。
- 主题族变量的值对 Tailwind 必须是 `H S% L%` 三段式（`hsl(var(--x))`）；原始
  `#rrggbb` 代入会让声明 invalid-at-computed-value 并静默回落静态表。
- App 声明了 `design { theme: .x }` 时，生成器把该表达式投影为组件内
  `__designTheme`（`computed`），首帧与自动应用点优先读它；`theme_name` 作为
  兼容别名在其后兜底，两者皆空才回落 localStorage/scaffold。投影支持面为字符串
  字面量与单字段引用（prop → `props.X`，state → `X.value`），其余形态回落别名。
  发射门：该投影随主题运行时一起注入，要求 widget 同时声明 `theme_name` 或
  `accent_color`；仅声明 design 的 widget 当前不发射运行时（见 §6 边界）。

## 6. 生命周期、释放与边界（C-06）

- 关闭一个 App/窗口只释放自己的选择与监听；另一 App 的 family/mode/目录与
  已发布快照不受影响；重开得到独立的新 scope（不继承前一次的临时选择）。
- preview 子 scope 继承父的选择/目录/设计，但发布不回流父；关闭 preview
  不影响父；父删除不悬挂子监听。
- 边界（不宣称完成）：
  - 真实 OS 主题事件、跨进程通知、配置持久化不在本包；系统模式只经受控
    provider 结果验证。
  - 非 widget（store composable）路径仍为 document 级变量，未接声明族首帧。
  - VM 原生无键盘焦点模型：button 的 focus 环只在 Vue 端由原生 `focus-visible`
    承担（`View` 无 button focus 状态）。
  - input/textarea 的 disabled 是真实通道：`View` 带 `disabled` 字段，构造期
    即不挂 `on_change`/`on_submit`（textarea 连 `onkeydown.*` 也不挂），
    **禁用不派发**。视觉上 Input 另做值/占位降透明；Textarea 目前只保证不派发，
    没有降透明表面（如实边界）。
  - 多 App/多实例同文档：每个带主题的 widget 模板根各自挂 `__autoThemeRoot` 与
    `data-auto-theme-root`，容器变量按实例隔离；子根的 vars 不受父容器清理影响。
    真实双实例 DOM 证据见 PLAN-031 的 V-03 腿（Auto/Solarized 各自持值、切换互不串写）。

## 7. 验收锚点

- 单测：`theme_scope_contract`（compiler）、`theme_scope_lifecycle`、
  `theme_scope_isolation`、`theme_host_inputs`、`design_scope_inheritance`、
  `control_surface_states`（runtime）。
- 真实腿：theme-lab 双端切换（Auto↔Solarized × light/dark，业务状态保持）、
  010 双端（VM MCP + Vue Playwright）、010 生成产物变量与官方包逐值对齐。
