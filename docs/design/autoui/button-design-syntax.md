# Button：视觉接口、Recipe、Theme 与 Design Pack 语法草案

> 状态：Draft v0.1，2026-10-29；全部新语法、类型、属性与 API 均是设计提案，未实施、未编译验证。
> 关联：[样式高阶抽象](design-system-abstractions.md)、[Design 47 §7](../../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md#7-设计体系从样式复用到设计编程)、[Design 29](../../../../auto-lang/docs/design/29-autoui-style-theme-system.md)。
> 范围：仅以 Button 给出可讨论的端到端合同；不创建执行 Plan，不修改应用/DSL/compiler/runtime，不执行构建或测试，不改变既有默认外观。
> 2026-10-29 讨论调整：用户质疑接口/实现/派生语法的复杂度，当前推荐转为[树状数据优先模型](design-data-model.md)。本文保留为显式语言候选与语义问题对照，**不是首选公开语法或已确认方案**；内部状态/校验边界仍可复用。

## 1. 应用作者先看到什么

应用显式选择一个设计包，普通按钮不必声明 recipe、Theme、parts 或 target。

```auto
use auto.designs: Studio

widget App {
    design: Studio

    view {
        row {
            button "取消" { onclick: .Cancel }
            button "保存" { intent: "primary", onclick: .Save }
            button "删除" { intent: "danger", emphasis: "soft", onclick: .Delete }
        }
    }
}
```

本草案选定默认 `intent=normal`、`emphasis=solid`、`size=medium`，mode 默认跟随系统、density 默认 comfortable。App 只表达少量意图，由设计包协调文字、图标、背景、尺寸和状态。

`design: Studio` 是 opt-in 的新设计 profile 入口。没有选入该 profile 的现有应用/legacy 组件保持原合同，尤其是当前裸 button 的 chromeless 默认。新项目是否由脚手架写入该选择另行裁定，不在本稿中全局更改默认。旧模块的接入/映射也需要显式兼容策略，不能只因被新 App 导入就重写其源语义。

只需调整环境时可展开为：

```auto
design: Studio(mode: "dark", density: "compact")
```

这里 `Studio(...)` 创建/选择 DesignContext，不创建另一份业务 model；选项 schema 由包与框架合同声明，未知选项报错。

## 2. 四者的关系

| 对象 | 回答的问题 | Button 示例 |
|---|---|---|
| 视觉接口 | 组件允许设计改变什么？提供什么状态？ | ButtonVisual 的 parts、variants、状态与主题 schema |
| Recipe | 怎样把这些输入变成各部件样式？ | StudioButton 的结构、paint、focus、busy 规则 |
| Theme | 规则所需的语义值在当前模式下是多少？ | primary/solid/hover 色组、medium 尺寸、焦点环 |
| Design Pack | 应用选哪一套默认组合？ | StudioTheme＋StudioButton＋字体/图标/默认环境 |

```text
组件行为 ──提供──> variants / 实际交互状态 / 公开 parts
                         │
ButtonVisual ──约束──> StudioButton Recipe <──读取── StudioTheme 有效值
                         │                          ↑
                         └────规范化 parts 样式      │ mode/density/scope
                                                    │
Studio Design Pack ──绑定── ButtonVisual→StudioButton、Theme→StudioTheme
                         ↑
                      App 选择
```

Theme 只换值；Recipe 定义这些值如何用于组件；Design Pack 可以一起更换配方、值、资产及默认政策。切换任一层都不应更换组件业务身份、事件接收者或已有输入状态。

## 3. 组件库作者：声明视觉接口

拟议 UI 方言的库级声明形状：

```auto
design contract ButtonVisual {
    parts {
        root: BoxPart
        label: TextPart
        icon: Optional[IconPart]
        spinner: Optional[IndicatorPart]
    }

    variants {
        intent: ["normal", "primary", "danger"] = "normal"
        emphasis: ["solid", "soft", "outline"] = "solid"
        size: ["small", "medium", "large"] = "medium"
    }

    states {
        hovered: bool
        pressed: bool
        focus_visible: bool
        disabled: bool
        loading: bool
    }

    derived {
        phase: ButtonPhase = button_phase(states)
    }

    theme: ControlsTheme
}
```

`variants` 中的数组表示有限可选值域，不是每个调用点传入一个数组；具体复用核心 enum 还是 schema enum 待 parser/类型设计决定。`Optional[T]` 也只是合同形状，不在这里修改核心 May/Option 写法。

现有组件定义/Schema 通过拟议的 `visual: ButtonVisual` 元数据绑定它，并将真实节点映射到公开 parts。业务组件仍维护 disabled/loading、焦点、键盘与 activation；不要求每个设计包复制 Button 的事件实现。

- root/label 是必需部件。icon 在存在图标时提供；默认 Button 对 loading 提供可见忙碌反馈。spinner 是否预挂载或按需创建是行为/结构实现细节，配方不能凭空创建 native 节点。
- 公开部件的类型决定可写属性，如 label 接受 typography，root 接受框、尺寸和 focus ring；未知 part/属性类型错误应定位到 recipe。
- loading 默认阻止重复 activation，但保留当前焦点和可访问名称；disabled 走组件自身的禁用合同。两者不合并成一个 Boolean。
- focus_visible 来自实际行为/平台适配；是否允许 disabled 状态保留或程序化取得焦点，由行为合同决定。Recipe 只响应真实状态，不凭空制造焦点。
- 设计可以改变允许的样式与度量，不改 onclick、权限或 disabled 的执行含义。HTML submit/type 等行为参数与 intent 独立。

## 4. Theme：先声明值接口，再提供实现

以下类型和表形状均是拟议标准库合同，不是当前已有类型。

```auto
theme schema ControlsTheme {
    action: Table[Intent, Emphasis, ButtonPhase, ActionPaint]
    control: Table[ControlSize, ControlMetrics]
    label: Typography
    focus: FocusRing
    busy_motion: MotionSpec
}
```

其中：

| 类型 | 字段示例 |
|---|---|
| ActionPaint | background、foreground、border，均为 Color |
| ControlMetrics | height、padding_inline、radius、border_width、icon_size，均为明确单位的 Length |
| Typography | family、size、weight、line_height 等组合值 |
| FocusRing | color、width、offset 等绘制/几何合同 |
| MotionSpec | duration、easing、reduced-motion 下的静态替代策略 |

button_phase 的 ButtonPhase、variants 所需 Intent/Emphasis/ControlSize 由公共合同统一定义；不是不相关字符串键。最终主题必须覆盖全部合法 key；不能把缺少 disabled/danger 值藏成“恰好回落到 primary”。

设计作者可以继承一份完整基准值，只写差异：

```auto
theme StudioTheme for ControlsTheme extends CoreControlsTheme {
    mode light {
        action.primary.solid.rest: paint("#2563eb", "#ffffff", "#2563eb")
        action.primary.solid.hover: paint("#1d4ed8", "#ffffff", "#1d4ed8")
        action.primary.solid.pressed: paint("#1e40af", "#ffffff", "#1e40af")
    }
    mode dark {
        action.primary.solid.rest: paint("#93c5fd", "#172554", "#93c5fd")
        action.primary.solid.hover: paint("#bfdbfe", "#172554", "#bfdbfe")
        action.primary.solid.pressed: paint("#60a5fa", "#172554", "#60a5fa")
    }
    density compact {
        control.medium.height: 28dp
        control.medium.padding_inline: 10dp
    }
    control.medium.radius: 6dp
}
```

`paint(bg, fg, border)` 是概念性纯构造函数，将色值解析为 ActionPaint；`dp` 表示拟议的逻辑长度单位，单位语法未冻结。数字仅用于演示接口，不是默认设计的验收值。

CoreControlsTheme 在该例中代表已提供完整值表的公共基准；其他 intent/emphasis/phase、small/large metrics、字体/focus/motion 都继承它。基准实现与色彩/可读性验收需要后续真实设计，不能仅凭此摘录称整包已完整。

继承后按环境解析 mode/density，再应用合法局部主题补丁，最后解析 alias 并校验类型、缺项与循环。同一层同条件重复赋同一键报错；有意覆盖须来自明确的继承/补丁层。mode 默认 system 只负责选择 light/dark；high contrast、字号缩放和 reduced motion 不被普通 density 覆写。

## 5. Recipe：把值应用到部件，而非维护第二套行为

拟议的 typed style 扩展沿用 `style` 名称；`for ButtonVisual` 表示实现该视觉接口。

```auto
style StudioButton for ButtonVisual {
    base {
        let m = theme.control[size]

        root {
            min_height: m.height
            padding_inline: m.padding_inline
            radius: m.radius
            border_width: m.border_width
        }
        label { typography: theme.label, opacity: 1 }
        icon { size: m.icon_size, opacity: 1 }
        spinner { visible: false }
    }

    paint {
        let p = theme.action[intent, emphasis, phase]

        root {
            background: p.background
            foreground: p.foreground
            border_color: p.border
        }
        label { color: p.foreground }
        icon { color: p.foreground }
        spinner { color: p.foreground }
    }

    when focus_visible {
        root { focus_ring: theme.focus }
    }

    when loading {
        label { opacity: 0 }
        icon { opacity: 0 }
        spinner { visible: true, motion: theme.busy_motion }
    }
}
```

`base` 声明常态几何与排版，`paint` 是依赖已选 phase 的着色输出，`when` 声明独立状态补丁。它们都是纯样式输出，可静态分析；没有创建文档、执行 I/O 或修改业务 model 的权限。

代码是设计摘录，标准组件还负责内容排列、spinner 定位与 busy 时保留度量/可访问名称；opacity 变更不会删除 label 内容。每次求值都由 base＋当前激活规则生成完整有效结果，loading 退出后恢复 label/icon opacity 与 spinner 的基值，不残留上一次补丁。禁用反馈由 disabled 色组体现，避免用 root 的统一低透明度连焦点环一起压暗。

如果 icon/spinner 在当前合法结构中不存在，相应可选 part 输出跳过；必需 part 缺失是组件/adapter 合同错误。可选 part 不应让 loading 的必需反馈消失：没有 spinner 的实现须由组件提供合同认可的替代。

### 5.1 合成规则不能依赖源码先后

候选规则为：

1. 先选择兼容的 recipe、主题有效值和 variant 参数。
2. 合成 recipe base，再应用被允许的 App/scope/instance base 补丁。
3. 根据实际状态解析 phase paint 和激活的状态补丁；状态输出覆盖相应 base 属性。
4. paint、focus、busy 等独立输出只要写不同属性就可相加。同级激活规则写同一属性且没有明确互斥/优先声明时，报冲突，不用文件顺序决定。
5. 更上层有意替换一个状态规则，必须指明对应状态/输出位置；普通 base 补丁不会自动抹掉 disabled/focus/loading 规则。
6. 最终再校验行为相关限制、有效设计政策及目标后端支持，生成有来源信息的规范化样式。

这不是 CSS specificity 模拟器。旧 utility/class 继续作为兼容输入；新 profile 的实例布局 utility 可正常使用，但不能用未声明的 class 串绕开受管状态属性。需要强定制状态颜色时改语义 role 或显式状态 recipe 补丁，并给出来源诊断。

## 6. Design Pack：给应用一个默认入口

```auto
design pack Studio {
    theme: StudioTheme
    recipes {
        ButtonVisual: StudioButton
    }
    defaults {
        mode: "system"
        density: "comfortable"
    }
}
```

`design: Studio` 据此完成两件事：选择 StudioTheme 的有效值解析，以及为当前 profile 的 ButtonVisual 绑定 StudioButton。应用不再逐按钮写 `style: StudioButton(...)`。

例中 Studio 只展开 Button 切片；真正应用级设计包还要提供其声明支持的其他组件合同和字体/图标资产等。缺少实际使用组件的 recipe 或必需角色时明确报错/要求声明兼容基准，不能随意漏回某个平台的默认皮肤。

另一个兼容包 Paper 可以绑定 PaperTheme 和 PaperButton。它们可以采用不同字体/形状/阴影与部件配方；只要仍实现 ButtonVisual 和所需主题合同，应用的按钮 markup 和事件无须改变。包/角色 schema/组件视觉合同分别有版本，依赖与资产复用既有包机制，不另建包解析器。

## 7. 切换：值变化与设计实现变化分开表达

### 7.1 换模式或整套设计包

静态选择：

```auto
design: Studio(mode: "dark", density: "compact")
// 或独立选择另一个包：
design: Paper
```

运行期切包的拟议写法：

```auto
widget App {
    model { var appearance DesignPack = Studio }
    design: .appearance

    view {
        button "切换设计" { onclick: .SwitchDesign }
    }
    on {
        .SwitchDesign -> { .appearance = Paper }
    }
}
```

上述每个 design 声明是不同示例，不能在同一 owner 中重复声明。切包先准备并校验 recipe/schema/资源组合，再发布新的 DesignContext；失败保留旧有效设计并报告原因。它不是再次挂载整个 App 的信号。

默认在切包时采用新包 defaults；需要保持用户的 mode/density 选择时应把这些选项作为独立稳定偏好传入，而非存进旧包私有配置。切换主题值与更换整个设计上下文都保留业务状态/事件身份，按依赖使绘制或 measure/layout 失效。

### 7.2 局部 mode/角色覆盖

```auto
design-scope(mode: "dark") {
    button "深色区域按钮" { onclick: .Confirm }
}

design-scope(theme: {
    control.medium.radius: 999dp
}) {
    button "局部胶囊按钮" { onclick: .Confirm }
    button "局部危险按钮" { intent: "danger", onclick: .Delete }
}
```

未写的 mode/density/recipe/theme 继承外层；局部 theme 是对当前有效 schema 的有类型补丁，不是另建一份不完整 Theme。medium 半径只影响当前子树中消费该公共角色的控件；small/large 不被无意改动。配方若声称支持该公共定制角色，就需真实消费并接受验证。

改变危险按钮底色应覆盖相应 action role，而不是给所有状态写一个 background：

```auto
design-scope(theme: {
    action.danger.soft.rest.background: "#fff1f2"
    action.danger.soft.hover.background: "#ffe4e6"
}) {
    button "删除" { intent: "danger", emphasis: "soft", onclick: .Delete }
}
```

此处只覆写 rest/hover，其余 phase 和 foreground/border 继承；示意颜色需结合所选模式和配色验证，不能当成通用跨模式安全色。需要同时适配明暗时，提供分 mode 的主题补丁或一个派生 Theme，不把 light 的 hex 值无条件带入 dark。

### 7.3 换局部配方或改单个按钮

```auto
style CapsuleButton for ButtonVisual extends StudioButton {
    base {
        root { radius: 999dp }
    }
}

design-scope(recipes: {
    ButtonVisual: CapsuleButton
}) {
    button "局部配方" { onclick: .Confirm }
}

button "只有这个不同" {
    recipe: CapsuleButton
    onclick: .Confirm
}
```

派生继承未改的 paint/focus/busy；实例级 `recipe:` 是选一个兼容实现，不是丢弃组件合同。它只影响这一组件，自身弹层等是否继承配方仍按各自组件合同决定，不能把 ButtonVisual 的 recipe 传播给不相关部件。

CapsuleButton 明确继承 StudioButton，切换外层 Paper 时仍是显式局部绑定；不会自动改为“Paper 的胶囊派生”。希望定制跟随任何包时，应优先覆写公共 radius role，或另外设计针对有效 recipe 的受限 patch 机制，后者本稿不假称已解决。

局部切整个包可用 `design-scope(pack: Paper) { ... }`：换值源与 recipe 集，同时继承显式用户环境选择；新包私有默认只补未指定项。旧包私有 role/recipe 补丁若在新 schema 中无效，准备阶段诊断，不能静默忽略。App 根切包与局部切包的默认继承差异应在 IDE/Inspector 中可见。

## 8. 多状态组合的确定结果

ButtonVisual 的只读 phase 使用下列默认选择：

```text
disabled → disabled
否则 loading → loading
否则 pressed → pressed
否则 hovered → hover
否则 → rest
```

这只选择底色/前景/边框用的 ActionPaint，**不是所有样式的全局状态优先级**。Theme 可以显式将 loading paint alias 到 rest，或提供自己的 loading 色组；不能遗漏该 key。

focus_visible 独立控制 focus ring；loading 独立控制 busy 指示；pressed 的位移等反馈只能在 `!disabled && !loading` 时输出。若某个包没有位移反馈，无须添加一个假 transform 状态。

| 实际状态 | paint phase | 额外组合 |
|---|---|---|
| hovered | hover | 无焦点环，除非真实 focus_visible 同时成立 |
| hovered + pressed | pressed | 允许 pressed 反馈 |
| hovered + focus_visible | hover | 保留 focus ring |
| pressed + focus_visible | pressed | pressed 反馈与 focus ring 同时存在 |
| loading + hovered | loading | busy 指示，抑制 hover/pressed 反馈 |
| loading + focus_visible | loading | busy 指示＋focus ring；焦点仍归原组件 |
| disabled + hovered + pressed | disabled | 不产生 hover/pressed 激活反馈 |
| disabled + loading | disabled | loading 的忙碌指示仍可见，不能丢掉在途操作信息 |
| disabled + focus_visible（行为层确实保留该状态时） | disabled | 真实焦点仍可被标识，不因改底色而隐去 |

disabled/loading 的实际 activation 阻断由行为层执行，包括程序化/快捷键触发该 Button 的入口；配方只承担视觉输出。这不意味着某个按钮 disabled 就禁止同一业务命令从其他合法入口执行，领域权限与 command enablement 仍有自己的合同。focus ring 不随 root 低透明度一起消失，reduced-motion 下仍有静态 busy 信息。这些都是候选验收合同，尚未做浏览器/native 交互证明。

## 9. 候选解析与运行边界

建议有效组合分别解析：

- recipe：Design Pack 默认 → App 允许的替换 → 最近局部替换 → 实例显式 recipe。
- Theme：所选包值源 → mode/density 等合法轴 → App/scope/实例的有类型角色补丁 → alias/type/完整性校验。
- Button 参数：组件合同 defaults → App 声明允许的默认参数策略 → 调用点显式 variants。
- 输出：base → 已选 phase 的 paint＋独立状态补丁 → 明确的状态覆写 → 政策/目标能力检查。

不要把上述四条不同问题合成一个“最后写的对象获胜”。Recipe 与 Theme 的 schema 必须兼容；传另一个 Theme 不自动获得另一套 recipe。

theme-scope/portal/overlay 沿逻辑 owner 保留 DesignContext；native thread-local、DOM 物理父节点、焦点或 GPU surface 不能单独决定主题。动效、字体和纯色更新分别触发适当的 paint/measure/layout 失效，不清空 scope、文档/undo 或 model。

编译器应检查未知 part/variant/role、错型单位、缺失 recipe/schema、alias 环与状态冲突；运行期动态切换也重复必要校验。CSS 与 native 消费同一规范化条件/属性语义，unsupported 属性明确报错或经声明降级，不让 Web 的默认 cascade 成为隐式规范。

## 10. 与现有写法的边界及待讨论项

本稿沿用 button、style、on、model 的大体形态，但新增 design contract、theme schema、typed recipe、design pack、design-scope、设计属性与状态合成合同。**所有示例都不能直接作为当前 AutoUI 的可运行代码。** `ButtonVisual`、ControlsTheme、Studio/Paper、字段/函数名和单位只是当前讨论用命名。

正式实现前优先裁定：

1. 是否接受新 profile 的简洁默认（选一个包即可使用设计化 Button），以及旧应用/模块的兼容门。
2. `base`、phase paint、独立状态补丁的组合语义与冲突诊断，尤其是设计 override 不绕过行为。
3. Theme 公共角色与 Recipe 的真实消费合同、局部切包的继承与无效补丁处理。
4. 模块化语法是否可以复用现有类型/Schema/纯函数，减少新的 parser 特例；此稿先固定语义，拼写允许调整。

未来验证至少覆盖完整 variants×可达状态组合、模式/density、库公开 part 与实际 renderer 映射、Theme/Pack 切换失败保旧、两个局部设计范围不串色、busy 焦点/activation、未知角色/冲突反例，以及 legacy chromeless 不变。本轮只做设计文档检查，未执行这些案例。
