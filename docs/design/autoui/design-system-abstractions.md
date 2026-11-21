# AutoUI 设计体系：样式的高阶抽象与组织

> 状态：专题讨论稿 v0.1，2026-10-29；仅调研与设计，示意语法/API 未实施、未冻结。
> 关联：[Design 47 §7](../../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md#7-设计体系从样式复用到设计编程)、[Design 29](../../../../auto-lang/docs/design/29-autoui-style-theme-system.md)、[Editing Feature 默认用法](../../../../auto-lang/docs/design/autoui/editing-feature-syntax.md)、[基础样式与跨端规范](../../../../auto-lang/docs/design/autoui/base-styles-and-visual-parity.md)。
> 本轮不创建执行 Plan、不修改编译器/运行时/应用、不执行构建或测试；不启动现有 AutoGraphics/ACE/生产融合之外的新实施线。
> 具体样本：[Button 端到端语法草案](button-design-syntax.md)——视觉接口、recipe、Theme 与 Design Pack 的库作者/应用用法及状态合成；仅设计。
> 表达方式讨论更新：用户已确认以[树状设计数据](design-data-model.md)继续设计，具体 [Button Atom 样本](button-theme-atom.md)中的字段/求值协议尚未冻结。Schema/状态约束由组件与引擎提供；下文的接口/函数类比是语义分析工具，不要求全部成为新的用户语言声明。

> 2026-11-13 接续：通用组件扩展已另立[组件库扩展 strategy](component-library-extension-strategy.md)，第一批084/085/086仅建实施计划。上文“不创建执行Plan”描述2026-10-29专题讨论的边界，不限制这次已授权的规划；本页的未来样式语法仍未因组件立项而获实施。

## 1. 之前讨论到哪里，尚未解决什么

Design 29 已讨论并推动命名主题、颜色值单源与参数化 style recipe；Design 47 §7 给出 token → semantic roles → parts/recipe → Design Pack 的方向，§16 又登记了默认设计体系与组件接入的未来研究分母。它们没有完整回答以下问题：

- 一份可组合“样式”究竟是什么类型，组合后如何处理冲突？
- 组件的视觉接口怎样声明，怎样由不同设计包实现？
- 同一组件的多个部件、变体、交互状态如何协同，而非重复 class 串？
- 哪些决策属于应用、组件、主题、模式或设计工具，如何保持单一来源？
- 普通应用怎样使用默认设计，又能在需要时只覆写一小部分？

因此，前面的工作是方向与边界，尚不是成熟的设计表达体系。本篇展开这些问题，不把新增几种关键字当作答案。

## 2. 代码现状：已有函数式雏形，缺少设计合同

历史基线仍是 Design 47 的 v0.5.0。本轮补充只读检查迁移后的 auto-ui，2026-10-29 的 HEAD 背景为 `d3bd6b495dc8b690925e88deee9fbf2c42e7f991`；下面定位迁移后路径，不据此宣称整个独立仓迁移或跨端能力已完成。

| 入口 | 观察 | 含义 |
|---|---|---|
| `crates/auto-ui-compiler/src/design_tokens/recipe.rs:20` | StyleRecipe 有 params/body/pub/source_module；支持参数、引用、导入与展开 | 已不止常量变量，但表达载体主要仍是 class 串/表达式 |
| `design_tokens/registry.rs:26`、`:278` | 31 键 TokenName 与结构化 ThemeSpec | 已有颜色角色及多目标值源，不能从零描述主题能力 |
| `design_tokens/decl.rs:53` | ThemeDecl 包含 extends/mode/colors | 主题声明主要是颜色层，尚不能等同整个设计系统 |
| `crates/auto-ui-compiler/src/style/variants.rs` | button variant/size 是内建预设，裸 button 缺省 chromeless | 内建预设不等于应用可发布/替换的完整组件设计合同；旧默认不能静默改变 |

这里的“缺少”指尚未形成本文目标中的统一公开合同，不断言仓库完全不存在零散相似机制，也不把源码存在当作运行验收。

## 3. 外部机制的进一步比较

2026-10-29 查阅官方资料。下表右侧均是对 AutoUI 的建议，不是外部项目已为 AutoUI 提供支持。

| 体系 | 实际机制与来源 | 借鉴与边界 |
|---|---|---|
| Fluent 2 | global token 保存基础值，alias token 加入语义；复杂 alias 可组织 typography/shadow 等组合值。[官方说明](https://fluent2.microsoft.design/design-tokens) | 值层需从原始数值到用途角色；只有颜色别名仍不够形成组件组合层 |
| Panda CSS | slot recipe 为多个部件组织 base/variants/compoundVariants，返回各部件的 class。[Slot Recipe](https://panda-css.com/docs/recipes/slot-recipes) | 一个变体应协调 root/icon/label 等；不照搬任意 DOM selector 为跨端合同 |
| vanilla-extract Recipes | base、variants、compoundVariants、defaultVariants；类型化调用，样式在构建期生成。[Recipes](https://vanilla-extract.style/documentation/packages/recipes/) | 样式函数可以是受限、可静态分析的程序，不必靠字符串插值组织所有设计 |
| vanilla-extract contracts/Sprinkles | theme contract 声明变量形状；Sprinkles 组织允许的属性、值、conditions 与 shorthands。[Contract](https://vanilla-extract.style/documentation/api/create-theme-contract/)、[Sprinkles](https://vanilla-extract.style/documentation/packages/sprinkles/) | 角色接口与赋值分开；utility 可以是有类型的设计 API，而不必永远是自由字符串 |
| Tamagui | tokens 为 space/size/radius/color 等提供命名值；Theme 对子树换主题并允许局部更新。[Tokens](https://tamagui.dev/docs/core/tokens)、[Theme](https://tamagui.dev/docs/core/theme) | 研究跨 Web/native 的值解析与局部环境；不把某个版本的命名、降级或循环处理规则直接当 AutoUI 标准 |
| DTCG | Format 定义 token 交换；Resolver 支持 contexts/modifiers 与有序解析。[Format 2025.10](https://www.designtokens.org/tr/2025.10/format/)、[Resolver 2025.10](https://www.designtokens.org/tr/2025.10/resolver/) | 交换值/引用/模式是基础，部件合同、交互和完整皮肤仍需 AutoUI 定义；这是 Community Group 规范，不称 W3C 标准 |
| Figma | Variables API 交换变量；Code Connect 连接设计组件和代码组件，并有 framework-agnostic template 方式。[Variables](https://developers.figma.com/docs/rest-api/variables/)、[Code Connect](https://developers.figma.com/docs/code-connect/) | 必须区分 token 链与组件映射链；不由此宣称已有 AutoUI connector 或完整双向设计包同步 |

Material 3 token 页面本次工具只取得 JavaScript 占位，Radix tokens 页面未成功读取；不把未读到的细节列为已证实依据。本表不是全部 UI 库的穷尽调查，后续可以按具体争议定向补样本。

## 4. 更高阶的抽象对象是什么

建议把“设计编程”分成值、组合、组件接口、环境、发布五种职责。它们是语义层次，不要求一层一个新关键字或新 crate。

| 职责 | 可复用对象 | 示例 |
|---|---|---|
| 值与类型 | Token/Role schema | surface.canvas、text.secondary、control.height、focus.ring |
| 纯样式组合 | StylePatch、纯函数、具名组合 | focus_ring、interactive_row、panel_surface、text_role |
| 组件设计 | DesignContract 与 Recipe/实现 | Button 的部件、意图、形态和状态映射 |
| 环境解析 | Theme/DesignContext 与正交 axes | brand、light/dark、density、contrast、motion |
| 发布与治理 | Design Pack | schema、角色赋值、组件配方、资产、默认值与兼容说明 |

### 4.1 值层：语义不是换一种变量名

基础值如 blue.600、space.3 是调色/尺寸材料；语义角色如 action.danger.background、field.invalid.border 表达设计用途。应用主要选择用途，具体材料由当前设计实现决定。

类型至少区分 Color、Length、Duration、FontFamily、FontWeight 及 Typography/Shadow/Border 等组合结构。Length 的单位和基准必须明确，font scale、DPI、Portal 几何缩放分别处理，不能将所有 number 都自动解释为同一像素单位。颜色空间/alpha 与目标端支持也要成为值合同。

角色 schema 与值表分开：schema 声明名字、类型、默认/必需、可覆盖范围和来源；主题/设计包实现值。标准角色保持稳定，领域包显式扩展命名空间，解析后的可用词表仍可严格检查。

不建议为每个 widget 的每个属性都发布一个公共 token：公共角色应服务于有意义、可维护的定制面；一次性派生值可以留在配方内部。

### 4.2 组合层：样式应是有类型的值

长期目标可以用概念类型 `StylePatch` 表示受控属性集合，`Recipe<C>` 表示消费组件合同 C 的设计函数。它们不是当前已有的 Rust/Auto 类型，也未冻结此命名。

例如 focus_ring(role)、text_role(role)、interactive_surface(intent) 可以是普通纯函数/具名样式组合，返回有类型的 patch。多个组件可以复用这些视觉规则，不必为每份共用外观都包装一个 widget。

需支持的高阶操作包括：

- 组合：多个 patch 明确形成最终属性，不只是串接字符串。
- 参数化：参数为合法枚举/角色/数值类型，而非任意 class 片段。
- 条件：依赖公开 variant、交互状态、尺寸条件和 DesignContext。
- 派生：复用一个基配方，只覆写公开 part/规则，限制深继承和隐式覆盖。
- 检查：未知角色、类型不符、循环引用、未绑定必需状态/part 有来源诊断。

编译期可展开已知常量/组合；动态主题、运行状态和窗口条件保留为可解析依赖。设计函数不允许 I/O、时间/随机数或修改业务状态，不能把纯配方当作第二个事件处理器。受限计算需可终止、可缓存，具体预算留详细设计。

### 4.3 组件层：先有视觉接口，后有皮肤实现

以 Button 为例，其公开设计合同可以包含：

| 项 | 示例 |
|---|---|
| parts | root、label、icon、spinner |
| variants | intent=normal/primary/danger；emphasis=solid/soft/outline；size=small/medium/large |
| states | hovered、pressed、focus_visible、disabled、loading |
| 可用设计输入 | action roles、control metrics、label typography、focus/motion roles |
| 约束 | 合法变体组合、公开 part、不能改变的行为/可访问性语义 |

**DesignContract 是稳定接口，Recipe 是某套设计对接口的实现。** 同一份功能组件可以切换不同配方；组件内部 DOM/native 子树不成为应用的样式 API。

`intent` 表示用户意图，`emphasis` 表示视觉强调，两者分开。`size` 是局部选择，density 是环境政策；一个 compact App 内仍可有显式 large 控件，由配方定义二者的合成关系。

hover/disabled/loading 的真值来自组件/行为系统。配方可以据此改变外观，不能自行把 disabled 改成可点击，也不能用“颜色变灰”替代行为禁用。状态重叠时需要确定规则和组合断言，不能假设最后一个 class 恰好正确。

不是所有组件都必须暴露大量 variants；合同应选择有意义的轴，避免十几个互相矛盾的 Boolean 开关。

### 4.4 更高于组件的视觉复用

例如“可选择的列表行”“可折叠面板 chrome”“工具栏操作”“表单字段反馈”横跨多个组件。它们可以复用 typed patch/recipe 组合，声明需要的语义状态并显式绑定到组件合同，避免每个组件各自实现 focus、selected、invalid 的视觉规则。

如果变化需要增删节点、改变 slot 布局或事件流，应交给 Widget/Blueprint。样式配方可以改变允许的间距、几何和绘制属性；不把任意产品结构重建藏在 design pack 中。

## 5. Theme、Design Pack、模式分别是什么

建议术语：

- **Theme**：某份角色 schema 在一个设计上下文中的有效值选择；可有局部覆盖。
- **DesignContext**：当前设计包与 brand/mode/density/contrast/motion 等选择及继承范围。
- **Design Pack**：对角色与组件合同的一套完整可发布实现，包含值、配方、资产、默认值、支持和兼容清单。

换 accent 只改一部分值；换明暗模式改变角色赋值；换一整套 Design Pack 可以改变字体、形状、间距、组件强调方式及其协同关系。它们不应全部塞进一个名叫 theme 的无类型大对象。

各 axes 独立声明并定义解析顺序、默认值和非法组合，不为品牌×明暗×密度生成难维护的所有主题名称。density、字号缩放、高对比和 reduced motion 不得简单相互乘除；它们对可读性/操作区域有不同约束。

普通 App 只选设计包和少量模式；库作者负责复杂表达。可以暂用下述配置形状讨论体验，**它不是已决定的 pac.at Schema**：

```auto
design: {
    pack: "studio"
    mode: "system"
    density: "compact"
}
```

```auto
// 消费端语义示意；intent/emphasis 由设计合同校验，非现行通用字段。
button(text: "删除", intent: "danger", emphasis: "soft") {
    onclick: .Delete
}
```

应用不必在这个调用处计算背景、文字、hover、icon、disabled 与 ring。设计包负责协调，App 只表达意图。Studio 仅作示意名称，默认包命名和数值未裁定。

编辑 Feature 默认继承宿主 DesignContext，并声明 editor/diff 等领域角色及默认映射；它不携带一套强制覆盖宿主全局 CSS 的 auto-edit 产品皮肤。两份独立编辑器可以有不同局部 DesignContext，文档、undo、焦点语义不因此改变。

## 6. 一个需要讨论的库作者表达方式

目标语义可以先写成下面的设计示意，不先决定究竟用 `style` 的块扩展、schema 数据还是普通函数实现：

```text
recipe ToolbarAction implements Button.design
  inputs: intent, emphasis, size
  compose: control_metrics(size), label_text(), focus_ring()
  parts:
    root.background = action[intent, emphasis].background
    root.foreground = action[intent, emphasis].foreground
    icon.color       = inherit(root.foreground)
    label.typography = typography.control_label
  conditions:
    hovered         -> apply action[intent, emphasis].hover
    focus_visible   -> apply focus_ring to root
    disabled        -> apply disabled roles
```

关键是让“组件设计函数”有输入、输出部件、复用组合、状态条件与角色依赖。现有 `style pill(bg: str, fg: str) = ...` 可以继续作为兼容输入，但单靠字符串参数表达这些关系，会把约束隐藏在约定里。

应先用 Button、TreeRow/Tabs、Editor chrome 三类样本检验，再决定必要语法。简单组件检验默认体验，复杂组件检验多部件和状态组合，编辑功能检验跨应用的设计角色与包边界。

## 7. 最容易被省略的规则

### 7.1 合成与覆盖要可解释

不同层分别处理“选择设计实现”和“解析该实现所需的值”，不把它们都简化成任意字典 merge。

例如 framework baseline → 选定 Design Pack → App recipe 覆盖 → 局部允许覆盖 → 实例 override 可以作为候选层次；每层内部的 variant/state、shorthand/longhand、logical direction 等仍需明确优先级。两个同级规则写同一属性时，要求声明覆盖或确定顺序；公共库不能因 import/CSS 注入顺序变化而变样。

Inspector 应能追踪“此颜色来自 Studio 的 danger/soft，当前 dark 模式，局部 scope 覆写了某角色”。高对比/reduced motion 等用户约束在单独政策层校验，不是所有实例 override 都能解除。

### 7.2 设计范围沿逻辑归属解析

App/window/subtree 的设计范围要有 owner。菜单/弹层经 portal 移到宿主层后仍保留发起方的设计上下文；不能取当前焦点、线程全局主题或最后一个挂载者的值。

同一个 Surface 多处呈现时，“共享业务状态”与“是否共享外观/缓存”分别定义。不同外观可能需要不同 presentation/cache 实例，但不能暗中复制文档/VM/订阅。此项继续衔接 Design 47 §16 的图形边界，不扩展为自动给所有 3D 材质换肤。

### 7.3 跨端必须共享语义

utility、旧 recipe 和未来 typed recipe 应归一到可检查的样式语义，再分别生成 CSS/native 产物。不能只在 Web 中引入一个强大的新主题对象，随后手抄一份“近似值”到 native。

编译期做类型/引用/支持检查，运行期解析 scope/mode/state。颜色变化与字体、density、圆角/边框尺寸等影响不同，失效需要区分绘制和重新测量。未支持能力有明确诊断/显式降级；跨端相同语义不自动保证所有平台像素逐点相同。

### 7.4 默认包是产品选择，不是兼容修复

新项目是否默认采用某个完整设计包，需要独立比较和用户裁定；旧应用不自动换肤。当前裸 button 的 chromeless 行为不能因本篇的“设计包默认值”而被静默改掉。

默认设计的组件角色、状态、字体/图标回退、density 等应覆盖完整公共组件分母，不能只调好按钮、卡片便宣称设计体系完成。应用可以逐步选入新 profile，legacy class/主题/recipe 的映射与变化逐项记录。

## 8. Figma 接入需要两条链

### 8.1 值与模式交换

```text
Figma Variables / styles / 离线导出
  → adapter 与稳定 ID 映射
  → 版本化 token schema/value/mode 数据
  → AutoUI DesignContext 与跨端产物
```

DTCG 是候选交换边界；不要直接拿工具内部名称当代码 API。Typography、shadow、资产等组合值需要额外映射，不能假定每个组合都能从单个 Variable 无损取得。色彩空间、单位和字体资产也需转换/校验。

### 8.2 组件与设计变体映射

```text
Figma component / component properties / variants
  → AutoUI 组件设计合同、variant 参数、part 与资产映射
  → 可校验的代码使用方式与设计包实现
```

例如设计工具中 Danger + Soft + Compact 不能只导入几个颜色变量；还需知道对应哪个组件、哪几个参数，Compact 是局部 size 还是环境 density。Code Connect 是组件与代码映射的外部机制参考，不替代 AutoUI 自己的设计合同或生成完整实现。

每组资产指定权威源与版本，导入输出可审查差异；先单向流程，再讨论冲突处理后的反向同步。Figma Variables API 与 Code Connect 的计划/权限条件并不相同，应按实际账号和入口检查；离线导出也须可用。不能以“支持 Figma”掩盖只导入了 hex 色板的范围。

## 9. 这一方向真正要交付的设计问题

| 问题 | 需要的可审查结果 |
|---|---|
| 样式值与计算 | 类型/单位/纯度/求值时机，动态值与编译期值边界 |
| 组合与派生 | typed patch/recipe 的操作、冲突规则、公开/私有设计 API |
| 组件设计合同 | parts、variants、states、角色依赖与行为边界；替换实现的兼容条件 |
| 应用使用体验 | 选包、默认继承、局部定制和单个控件 override 的最少写法 |
| 设计包治理 | 发布单元、角色/部件版本、资产依赖与既有包机制衔接 |
| 设计工具交换 | token/mode 与组件映射分别覆盖什么，丢失/冲突如何报告 |
| 跨端与旧行为 | 规范化语义、支持分母、默认包 opt-in、legacy 映射 |

未来候选反例包括：unknown role/part/variant、Length 与 Duration 混用、alias 环、state 冲突、相同 logical 角色在两种模式下解析、portal 不串色、density 后重新测量、设计包切换不丢编辑状态、Figma rename 不破坏稳定映射，以及旧 chromeless button 保持原样。

这些是后续详细设计与验证输入，本轮未执行。最先值得收敛的是“组件视觉接口与配方实现”的分离，以及普通 App 如何只表达意图；否则扩大 token 数量仍然会把设计接线留给每个应用。

## 10. 组件库扩展的设计接入（2026-11-13）

[扩展 strategy](component-library-extension-strategy.md)管理新增/补齐组件和后续计划。首批为现有 FileDialog 增目录/保存模式、正式 TreeView 和 PaginationControl；后续含 PromptDialog、TaskProgressDialog、CommandPalette、TagInput、ColorPicker/ColorDialog、ImagePreviewDialog，以及 DataTable、日期时间和字体选择的能力研究。完整候选、状态和依赖只在 strategy 登记，不在本页复制执行进度。

组件行为合同先于皮肤：分别定义 parts、受控数据、焦点、禁用/加载/错误/取消、可访问名称及 MCP 行为；主题负责已有语义角色的视觉投影。首批直接复用当前主题和 `.at` 组合机制，不等待或新增本页尚未冻结的 Recipe/Design Pack 语言。

| 组件 | 需协调的部件和状态 | 应用/组件边界 |
|---|---|---|
| FileDialog | toolbar/path/filter/sidebar/list/selection/status/actions，新增filename/overwrite；modal焦点、空/加载/错误、选中和禁用 | 组件选择路径并验证模式；实际文件写入/导入由应用执行 |
| TreeView | root/row/indent/guide/chevron/icon/label/badge，focus/selected/expanded/disabled/loading/error | 组件渲染并请求展开/选择；节点、选择/展开数据和加载业务由应用持有 |
| PaginationControl | root/navigation/current/total/page-size，first/last/disabled/hidden/focus | 组件发受控页码请求；数据切片、SQL和请求时序归应用 |

部件名是设计沟通用语，不声明新的已实现公共 token/API。实际公开字段由对应Plan的T-01与最终Spec固定；保留现有组件解析和包单源。浅/深主题、窄窗、长中文、键盘和真实截图均为验收输入；strategy Q-01 的文字缺失不得以MCP文本完整代替视觉闭环。

### 10.1 通用创作组件接续（2026-11-14）

按用户要求先选择容易跨应用复用的7组：087数值/单位、088选色、089图片查看、090媒体播放、091资源选择、092属性面板、093停靠布局。它们覆盖普通设置、笔记/阅读、附件、素材和工作台；当前均仅立计划。完整候选/依赖/状态见[strategy](component-library-extension-strategy.md)，不在此重复制定执行进度。

新批继续复用当前主题与真实基础控件。颜色/图像像素属于内容值，不用主题色或CSS类串充当数据；数值单位、字段类型、资源ID、panel身份与皮肤分离。实例草稿/预览、最终提交、取消恢复和外部revision变化是组件行为合同，换皮不能改变这些语义。应用统一管理文档、持久化、撤销；组件通过同一普通事件链提供精确输入/按钮/菜单与拖动等价入口。

Vector/Transform、Gradient、Layer/SceneTree、Ruler/Guide/Grid、Gizmo、Timeline、Curve、NodeGraph及专业Canvas/SceneViewport仅保留在strategy储备。其坐标、色彩、时间、对象和连线规则等待游戏/图像/CAD等实际应用出现后再设计；不通过通用组件计划隐含批准新的专业内核或设计语言。

2026-11-14复核接续：[执行与视觉指南](common-components-execution-and-visual-guide.md)提供GPT生成的七组件参考图、逻辑尺寸、主题/状态与窄窗规则，087…093均细化到r2。图像不是交付截图，也不新增公共token或批量属性事务；实际公开接口/状态按各Plan r2，技术可行性从T-01验证。

### 10.2 轻量交互组件接续（2026-11-14）

PromptDialog、TaskProgressDialog、CommandPalette、TagInput已分别补齐[094](../../plans/archived/094-prompt-dialog.md)、[095](../../plans/095-task-progress-dialog.md)、[096](../../plans/096-command-palette.md)、[097](../../plans/archived/097-tag-input.md)实施合同。2026-11-14 立项复审时均为 drafting r2、0/8。直接引用[已生成四组件图及指南](file-navigation-and-interaction-visual-guide.md)。PLAN-094 后续已交付 Windows r3，PLAN-097 后续已交付 Windows r2（Linux 两轨按 084 §10 not_run，未宣称跨平台）。

| 组件 | 部件/状态 | 应用owner与可复用边界 |
|---|---|---|
| PromptDialog | title/description/label/input/error/actions，editing/invalid/submitting，模态焦点 | 复用027表单；应用负责唯一性、文件规则/写入，业务拒绝保留草稿 |
| TaskProgressDialog | step/progress/counts/results/cancel/close，running/requested/cancelling/terminal | 复用Reader状态映射和Progress；应用任务确认取消并产出结果，UI不管理worker |
| CommandPalette | search/groups/row/shortcut/help，active/disabled/pending/empty | 复用028搜索和键盘经验，provided模式保留原排序；应用执行命令/注册快捷键 |
| TagInput | chip/remove/input/suggestions/count/error，draft/readonly/limit/pending | 复用015控件/store；应用持标签存储及真实noteId，组件按受控值计数 |

四项均复用既有主题与普通控件；公共参数按各Plan r2，T-01验证机制，不从参考图再设计一套接口。稳定对象身份、会话/请求序号、重复操作锁与迟到反馈隔离属于行为合同，不能由皮肤改变；跨平台、普通MCP操作、真实业务读回、无人值守关闭重开和字体可读均有独立验收要求。更多执行状态和依赖只在strategy/主Plan维护。
