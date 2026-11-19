# Button 主题：Auto 树状数据与 widget 内的局部设计

> 状态：设计草案 v0.2，2026-10-29；树状数据优先方向已确认，本轮模块引用、主题族/模式、局部设计事件均为待讨论提案，未实施。
> 关联：[数据优先建议](design-data-model.md)、[高阶抽象议题](design-system-abstractions.md)、[历史 Button 语言方案](button-design-syntax.md)。
> 后续：[试用约定与小样](theme-conventions-and-pilot.md)明确公共/Feature/app 命名空间、extend 数据扩展及 v0.6 候选边界；auto-ui/PLAN-007 尚待确认，未实施。此前本稿“不创建执行 Plan”描述 v0.2 讨论轮，不禁止后续有授权的规划。
> 本稿取代 v0.1 的文件路径 source、字符串引用、design-scope/overrides 公共入口。只改设计文档，不改源码/Schema/正式 Spec，不创建执行 Plan，不运行构建或测试。

## 1. 采用 Auto 模块与树状数据，不另建主题加载语言

建议普通 `use` 导入主题与 token；包、版本、别名、可见性复用 Auto 模块机制。暂不新增 `use.theme`：只有存在不同于普通模块的实际解析/装配需求，才讨论专用 import kind。

Atom 的树形外观与静态 Atom 解析入口必须区分：现有 auto-atom parser 不执行 use、变量和字段表达式；下面是含树状数据的 Auto 模块草案，不能交给纯静态 AtomParser 直接运行。工具交换仍可导出规范化静态数据及引用元信息。

以下 `TokenTree`、`Theme` 是拟议的框架数据类型/Schema，采用普通数据声明，不是要求作者定义 trait/class。示例的 pub let、模块数据导出接入、widget theme/mode 字段与事件尚未联合验证，不声称现有版本直接可运行。

### 1.1 token 可独立导出

示意模块 `design/tokens.at`：

```auto
pub let tokens TokenTree = {
    radius: { control: "6dp" }
    color: {
        action: {
            background: "#eee8d5"
            foreground: "#586e75"
            hover: "#e5ddc6"
            pressed: "#d8cfb8"
        }
        disabled: { background: "#eee8d5", foreground: "#93a1a1" }
        focus: "#268bd2"
    }
}
```

TokenTree 的叶子具有可定位的逻辑槽身份与值类型。长度仍先写明确单位的字符串，由 Schema 解释；不新增 lexer 单位语法。不同类型的 token 树可以分别导出、按名导入；合并到主题的槽路径必须显式且经过 Schema 检查，不依据 import 顺序自动混合。

### 1.2 theme 代表主题族，模式放在族内部

示意模块 `themes.at`：

```auto
use design.tokens: tokens

pub let Solarized Theme = {
    default_mode: "light"
    tokens: tokens
    modes: {
        light: {}
        dark: {
            tokens: {
                color: {
                    action: {
                        background: "#073642"
                        foreground: "#93a1a1"
                        hover: "#0b4350"
                        pressed: "#125461"
                    }
                    disabled: { background: "#073642", foreground: "#586e75" }
                    focus: "#268bd2"
                }
            }
        }
    }
    components: {
        button: {
            base: {
                radius: tokens.radius.control
                background: tokens.color.action.background
                foreground: tokens.color.action.foreground
            }
            states: {
                hover: { background: tokens.color.action.hover }
                pressed: { background: tokens.color.action.pressed }
                disabled: {
                    background: tokens.color.disabled.background
                    foreground: tokens.color.disabled.foreground
                }
                focus_visible: {
                    focus_ring: { color: tokens.color.focus, width: "2dp" }
                }
            }
        }
    }
}
```

这里 Solarized 表示一个含 light/dark 的主题族，不把 Solarized-Light 再与一个独立 dark 开关拼接。配色只演示数据组织，不声明是完整 Solarized 规范、全组件包或已通过可访问性验证。

`modes.light` 是空补丁，使用根 token；dark 仅替换对应值。根值加所选模式必须组成可校验完整数据。单模式主题可以只提供 light；不能把其 light 数据谎称为 dark。

为缩短本稿，组件样式直接写在 components 中。需要复用时仍可导出语义样式树：样式组作为数据被显式组合；模块 use 只负责导入，不兼任样式合并。v0.1 的数据字段 use 不再作为本稿必需入口；语义组的最简组合字段另行讨论，不预先增加语法。

## 2. 点访问必须保留 token 关联

`radius: tokens.radius.control` 在主题样式字段里保存对 token 槽的类型化引用，不提前复制成一个永久的 "6dp"。渲染时从当前 widget 的有效 token 树解析，因此 dark、局部替换和热更新都能触发相关样式重算。

建议把该能力放在 TokenTree/Theme 的数据协议中：TokenTree 叶子读出可引用的 Token 值，字段 Schema 接受字面量或相容类型的 token 引用；普通 Auto 对象、数值和点运算不因此全局改成懒求值。作者不用写字符串路径，但实现仍须保存槽身份、来源与依赖。

导入绑定按词法命名空间解析；设计上下文只决定同一逻辑槽当前的值，不重新搜索同名变量。主题把导入树挂入 tokens 时建立槽对应关系；同一源树多次挂入产生歧义时必须明确选择挂载路径，不猜测。完全替换主题后，引用按新主题自身声明重新建立，不能套用旧主题的私有路径。

未知槽、循环、Color/Length 错型、不同树引用未挂载均给诊断。静态导出保存 token 引用身份/路径元数据，不能只导出最终颜色而声称仍保有可替换关系。Schema 识别、Token 值表示与模块数据导出是后续实现工作，本轮不假设已有。

## 3. 应用只选择主题族与模式策略

```auto
use themes

widget App {
    theme: themes.Solarized
    mode: "system"

    view {
        button "保存" { onclick: .Save }
        SettingsPanel {}
    }
    on {
        .ToggleTheme(theme) -> { .theme = theme }
        .ToggleMode(mode) -> { .mode = mode }
    }
}
```

这两个消息由宿主的设置/菜单操作发送，例中不另展开控件接线。只有一个主题时只需 theme 一行；省略 mode 时根 widget 默认 system，子 widget 继承父级的模式策略。

mode 表示请求策略（system/light/dark），resolved_mode 表示当前真实采用的 light/dark，后者只读。system 尽量匹配系统；单模式主题缺目标模式时允许明确回到其 default_mode，并暴露实际模式。显式 dark 配上仅 light 主题则拒绝这次更新；不静默冒充 dark。其他产品可选策略需另行明示，不藏在名称里。

同时切换 theme 与 mode 时，同一个同步事件内写入组成一次提交；先收集最终组合，校验后才对外发布，避免先换主题造成中间不合法。一次事件之后的独立异步更新不自动视作同一事务。失败保留此前有效组合，带出错误。

## 4. 局部修改属于 widget，不增加包装节点

拟议设计生命周期事件暂名 `.Design`：

```auto
widget CapsulePanel {
    view {
        button "当前面板按钮" { onclick: .Confirm }
        ChildPanel {}
    }
    on {
        .Design -> {
            .tokens.radius.control = "999dp"
            .theme.components.button.base.min_height = "36dp"
        }
    }
}
```

`.tokens` 是当前 widget 有效主题 tokens 的便捷入口；`.theme.tokens` 指向同一局部树。`.theme = themes.Solarized` 选择主题族，`.theme.components... = ...` 修改本实例的设计字段；导入 themes.Solarized 与导入 TokenTree 始终只读，不被这些写入污染。

这里的局部写入在内部形成本 widget 的差异层，未写字段继续跟随父级。实现可用持久化结构或写时复制，不要求为每个 widget 深拷贝整包。对作者不暴露 overrides 关键字。

`.Design` 是新事件提案，不把现有 `.Init` 改成多次执行：现有 .Init 位于首渲染后，不适合避免首帧样式闪动。通用 setup 也不能独自解决后续主题切换时重算的问题。

建议时序：实例/model/props 与继承设计可用 → 合成候选 theme/mode → .Design → 解析引用/校验 → 首帧；主题、模式或已跟踪的外部输入变化时重新构造候选并再执行 .Design。此处时序是设计需求，不宣称当前 setup/model 顺序已有该支持。

每次 .Design 从新基准计算，不在上一次结果上累积。它可以读取 props/model、做条件和局部设计赋值；不允许更换 theme/mode 选择器、不写业务 model、不发消息或做 I/O，避免重入与主题重算触发业务副作用。静态依赖无法确定时可保守重算。普通事件修改其依赖的业务状态即可动态调节局部设计；不用在 .Init 中写一次后失去关联。

若允许任意事件直接写局部 token，它将引入“这次写入保留到何时”的第二种状态。第一版建议把设计字段写入集中于 .Design；普通事件仅更换 theme/mode 或更新被读取的设置状态，不先扩展持久临时写入机制。

## 5. 覆盖顺序与继承边界

有效设计计算分成数据合成与引用解析两个阶段：

```text
主题族根数据
→ 选中模式的补丁
→ 从祖先继承的局部差异
→ 当前 widget 的 .Design 局部差异
→ 解析 token/语义样式引用与类型校验
→ 按组件状态生成最终样式
```

在候选树中运行 .Design 时，语句仍按普通 Auto 的源序执行：后写同一路径覆盖先写，读取可见此前的候选写入；不承诺任意读写代码顺序无关。命名树/对象按路径深合并，后层标量替换，数组整体替换；缺省继承，null 仅限 Schema 明确可空字段。声明数据同层重复字段应报错，不借此模拟有序赋值。

**token 与组件字段没有全局“谁优先”的竞争**：改 token 影响所有仍引用它的字段；把某组件字段设为字面量，只解除该字段对 token 的引用。

```auto
.Design -> {
    .tokens.radius.control = "10dp"
    .theme.components.button.base.radius = "20dp"
}
```

本 widget 下 Button 为 20dp，其他引用 control 的部件为 10dp。调换这两个独立赋值的次序仍是这个结果；若后续代码明确把 radius 改回 token 引用，则以最后一次赋值为准。

- 当前 widget 的修改作用于自身与其默认继承的后代；父级、兄弟、同一 widget 的另一个实例不变。
- 子 widget 无声明时继承父级的有效设计与模式策略；父级变化继续传播。
- 子 widget 的 .Design 对同路径进一步修改，只影响该子树；销毁子 widget 后自然失效。
- 子 widget 显式选择不同 theme 时建立新的设计基准，不自动携带父主题私有差异；mode 策略仍可继承，再按新主题校验。跨包品牌参数用普通 props 显式传递。
- 子 widget 只改 mode 时从同一主题族原始数据重建所选模式，再重放继承差异与自身差异，不把旧 dark 的已解析结果当 light 基准。
- 祖先显式写入的固定颜色会继续覆盖子树的新模式；框架不自动把它变成深浅自适应。祖先 hook 已求出的值按祖先上下文产生，子 mode 不会倒过来重新执行祖先生命周期；需要按子模式变化的颜色用模式 token 引用，或由子级 .Design 决定。

主题切换后，各保留实例的 .Design 从新主题重新计算；非法局部路径使受影响的整次更新失败，不能留下半棵子树的新旧主题混合。需要预备受影响子树并原子发布，业务 model/文档/undo 不重建。

## 6. Button 状态组合继续由组件规则负责

完成 token 与样式合成后再处理交互状态。disabled 抑制 hover/pressed；否则 pressed 优先 hover；focus_visible 的焦点环独立。loading 若组件支持，应独立表示忙碌部件并抑制激活；不能通过改颜色恢复行为。

mode/局部重算不会清空组件的当前 hover/focus 等行为状态，而是用新样式重新解释当前状态。多个独立状态写同一属性且无标准优先级时报告冲突，不让节点声明顺序代替状态语义。

## 7. 现有依据与后续验证边界

读取依据：`crates/auto-atom/src/parser.rs`（静态入口边界）、[模块解析](../../../../auto-lang/docs/specs/auto-lang/frontend/design/module-resolution.md)（普通 use、命名空间与别名）、[UI 生命周期](../../../../auto-lang/docs/specs/auto-lang/ui/design/scenario-dialect.md)（setup/.Init/.Destroy）。这些依据不证明新主题数据类型、叶子引用、widget 设计上下文与 .Design 已存在。

本轮只形成语法/语义讨论，不冻结事件名、类型名、mode fallback 产品策略或依赖追踪实现。未来应验证：模块导出和别名、树挂载歧义、token 换值联动、双实例隔离、父子覆盖、子级独立模式、换主题与本地路径不兼容、事件内原子切换、首次挂载失败诊断、明暗反复切换、HMR 后局部差异重算、Vue/native 同义。首次挂载无旧上下文可保留时应拒绝发布该子树并给诊断；换肤失败则保留上一个有效上下文。
