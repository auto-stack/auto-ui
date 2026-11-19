# 设计体系的数据优先组织建议

> 状态：讨论稿 v0.3，2026-10-29；用户已确认树状数据优先方向与可用 Atom，并建议采用普通 use、数据变量引用和 widget 内的局部设计；具体协议未冻结、未实施。
> 当前具体样本：[Button 的 Auto 树状主题](button-theme-atom.md) v0.2：模块导入、类型化 token 引用、主题族/模式与拟议 .Design 事件。本文旧 JSON/字符串引用形状仅保留为历史数据模型对照，以该样本的后续讨论为准。
> 来源：用户认为 Button 草案中的视觉接口、recipe 实现与派生写法过重，希望比较树状配置与类编程语言表达。
> 关联：[高阶抽象议题](design-system-abstractions.md)、[Button 显式语言方案](button-design-syntax.md)、[Design 47](../../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md)。
> 范围：仅调整推荐设计方向，不修改生产源码、既有组件 Schema 或已确认 Spec，不创建执行 Plan，不执行构建或测试。

## 1. 建议：数据为主，代码为可选生成工具

用户已确认采用**树状、有 Schema 校验的设计数据**作为继续设计的方向，并允许使用 Auto/Atom 节点组织；具体协议仍待讨论。上一版 `design contract` / `theme schema` / `style ... for ... extends ...` 是候选表达，不再作为本讨论的首选公开写法；保留原稿作为语义问题与对照，不把它当成用户已同意的执行合同。

保留的目标是类型正确、部件公开、状态合成确定、跨端同义；需要收敛的是给主题作者增加接口、实现、类继承等维护负担。普通作者应能打开一份配置，找到按钮/颜色/尺寸，改少数项并看到效果。

作者可通过普通 Auto 模块导出树状数据，使用既有 use 与点访问，不另建主题文件路径加载语言。高级作者可用普通 Auto 代码组织数据；交付模型仍是可校验数据与引用关系。静态 AtomParser 不负责执行 use/表达式。局部设计计算拟议由 widget 的 .Design 事件承担，其时序、可写范围与副作用边界见当前样本；不把任意运行期 I/O 放入设计求值。

## 2. VS Code 的借鉴范围

VS Code 以配置和主题文件组织命名的工作台颜色，并区分工作台颜色与编辑器语法/语义着色。[Color Theme](https://code.visualstudio.com/api/extension-guides/color-theme)、[Theme Color Reference](https://code.visualstudio.com/api/references/theme-color)

它的许多颜色 ID 使用点分名称，配置文件不必物理嵌套成完整树。AutoUI 可以选树状书写、内部规范化为稳定路径，借鉴“程序消费语义键，主题提供数据”的方式。

AutoUI 还需表达尺寸、排版、部件和状态样式，不能把 VS Code 的颜色表等同于完整跨端 styling 引擎。新增这些数据域即可继续讨论，不必因此向作者暴露新的类体系。

## 3. 一份树状配置可包含什么

| 数据域 | 职责 |
|---|---|
| tokens | 有单位的基础值或公共角色值：颜色、尺寸、字体、圆角等 |
| styles | 可命名引用的语义样式组：action.primary、field.invalid、surface.panel 等 |
| components | 内置组件默认、变体、公开部件与允许的状态覆盖 |
| modes | 明暗、density 等预先定义轴的值差异 |
| assets / metadata | 字体图标等资产引用、版本、来源和支持范围 |

这些是同一数据模型中的分区，普通用户不必逐层创建文件或对象。完整设计包可以提供默认值，应用只保存自己的小补丁；规模大时才拆文件。

下面是 **JSON 形状示意与局部配置摘录**，不是新标准、DTCG 完整文件或当前可加载配置；未列出的默认值/状态需要由明确基准提供。

```json
{
  "tokens": {
    "color": {
      "primary": "#2563eb",
      "onPrimary": "#ffffff",
      "primaryHover": "#1d4ed8"
    },
    "radius": { "control": "6dp" }
  },
  "styles": {
    "action": {
      "primary": {
        "base": {
          "background": "{tokens.color.primary}",
          "foreground": "{tokens.color.onPrimary}"
        },
        "states": {
          "hover": { "background": "{tokens.color.primaryHover}" }
        }
      }
    }
  },
  "components": {
    "button": {
      "base": { "radius": "{tokens.radius.control}" },
      "variants": { "primary": "{styles.action.primary}" }
    }
  }
}
```

这里 primary 样式连同自己的 hover 数据一起复用，不是把所有按钮的 hover 硬编码成主色。此历史示意的 `{...}` 已由新样本中的类型化 token 点引用取代；类型、路径和循环检查目标保留。原始值复制与保留关联须区分，普通对象访问不因此全局改变求值语义。

应用仍可以使用熟悉的组件入口，例如 `button "保存" { variant: "primary" }`。此处只演示单轴变体；并不将上一稿 intent/emphasis 自动改判为现行 API，也不要求第一版就支持任意维度矩阵。

## 4. Schema 留在组件/引擎侧

已有组件声明源应继续负责说明允许的属性、变体、公开部件和行为状态。需要新增的设计元信息也优先补在同一声明源，避免另维护一份视觉 trait 与一份 widget Schema。

- 主题作者不重复声明 Button 有哪些 parts/states；配置编辑器从组件元信息提供补全。
- 颜色引用不能填进尺寸；未知路径、变体、部件或单位给可定位诊断。
- 状态名由行为合同提供，主题不能自造一个状态后期待组件自动执行它。
- 必需角色/状态数据缺失时，有明确基准或报错，不能静默回落到另一个平台皮肤。
- 数据驱动允许严格校验；严格校验不要求作者编写接口/继承声明。

## 5. 状态规则由引擎定义，主题只填值

Button 的 disabled/loading/pressed/hover 着色选择、focus ring 与 busy 指示独立叠加等语义仍需定义，但应是组件系统的标准规则，不由每个主题重写 `button_phase()`。

配置数据只描述各阶段、公开部件的样式。基准→包→应用→局部补丁的覆盖顺序、同层冲突、shorthand 和状态重叠规则统一处理，不暴露一套任意 selector 程序。改圆角不应抹掉 disabled 反馈；按压底色不能掩盖键盘焦点。

第一版优先支持命名值、引用、有限变体/状态表、主题族/模式和 widget 内局部替换；允许普通 Auto 控制逻辑，但不另造 selector/类继承语言。数据合成与引用解析分开，局部替换不用额外 design-scope/overrides 公共入口。新的数据模型不自动修改旧 chromeless/default 或 legacy class 语义。

## 6. Theme 与 Design Pack 可以共用数据模型

Theme 可以主要是值/模式的覆盖；Design Pack 则包含更完整的语义样式、组件样式与资产。二者是内容范围和分发范围的区别，不需要两套语言。

小设计包可一个模块；较大包通过 use 组织 token/style/component 模块。依赖、版本、资产解析复用原有包机制。允许类型化引用与 widget 局部替换，但不先实现深层 base class、多重继承或方法覆写。Theme 在当前样本中代表可含多个明暗模式的主题族，mode 单独选择，避免把 Light 名称与 dark 状态错误拼接。

物理格式与数据模型分开：可以先用 Auto 现有的类 JSON 数据写法保存，工具读写规范化 JSON；也可以允许 JSON 输入。选择一个权威源，其他文件是导出产物，不要求双份手工维护。最终文件后缀、字段名、引用语法和 mode 结构仍待讨论。

Figma/DTCG 先映射 token/模式及来源 ID；组件样式和设计属性映射另外记录。数据模型更便于 diff、局部 patch、编辑器表单和导入导出，但不能因此声称任意 Figma 文件都可无损转换。

## 7. 与上一方案的比较与保留边界

| 维度 | 数据优先 | 公开的样式语言/接口体系 |
|---|---|---|
| 默认编写和阅读 | 找路径、改值、引用样式组 | 需理解更多声明/实现/派生规则 |
| 工具交换 | 可直接校验、diff、patch，映射边界清楚 | 必须再定义可交换子集与求值边界 |
| 复杂计算 | 通过可选工具生成数据；表达力受控 | 表达更灵活，语言/调试/资源边界也更复杂 |
| 类型和跨端正确性 | 依赖共享 Schema 与解释规则 | 同样仍需要共享 Schema/运行语义，语法本身不保证 |

推荐公开模型采用数据优先；原 Button 草案中“状态要分通道”“字段要校验”“业务与视觉分离”“局部设计不串线”等约束继续作为引擎内部设计输入。尚未由用户确认的语法选择不沉淀为 canonical Spec，不启动实现。
