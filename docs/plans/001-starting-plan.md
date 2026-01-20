# AutoUI 项目初步实施计划

## 项目愿景

AutoUI 是一个基于 Auto 语言的跨平台 UI 描述框架，目标是实现"一次编写，多处运行"的 UI 开发体验。

### 长期目标
支持多种 UI 框架后端：
- PC 端：gpui, iced
- Web 端：vue.js
- 移动端：Jetpack Compose, 鸿蒙 UI
- 嵌入式：LVGL

### 短期目标（第一阶段）
实现以 Auto 语言为描述层，支持 iced 和 gpui 为实现层的桌面端跨平台 UI 库。

---

## 技术策略

### 设计原则
1. **描述层与实现层分离**：Auto 语言作为独立于底层的 UI 描述语言
2. **ELM 架构**：采用类似 ELM 的消息通讯机制实现 UI 行为
3. **多后端支持**：设计抽象接口，支持切换不同底层实现
4. **优先 iced**：先确保 iced 基底的稳定实现，再扩展到 gpui

### 为什么选择 iced 优先
- 设计简洁，API 相对稳定
- 消息机制与 Auto 语言设计理念相容
- 社区活跃，文档完善
- 跨平台支持良好（Windows, macOS, Linux）

---

## 架构设计

### 三层架构

```
┌─────────────────────────────────────────┐
│          Auto Language Layer            │
│  (声明式 UI 描述 + 状态 + 消息处理)       │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│         AutoUI Core Layer               │
│  (组件抽象 + 虚拟 DOM + 渲染管线)        │
└─────────────────────────────────────────┘
                    ↓
┌──────────────┬──────────────┬──────────┐
│   Iced Backend │  GPUI Backend │  Others  │
└──────────────┴──────────────┴──────────┘
```

### 核心模块

1. **auto-lang**：Auto 语言核心（已有）
   - Parser / AST / Type System
   - Code Generation

2. **auto-ui-core**：UI 框架核心（新建）
   - Component Abstraction
   - Virtual DOM / Widget Tree
   - Event System
   - State Management

3. **auto-ui-iced**：iced 后端实现
   - Adapter for iced widgets
   - Message bridging
   - Rendering pipeline

4. **auto-ui-gpui**：gpui 后端实现（第二阶段）
   - Adapter for gpui widgets
   - Event handling

---

## 实施阶段

### Phase 1: 基础设施 ✅ **已完成**（2025-01-19）

#### 1.1 项目结构搭建 ✅
- [x] 创建 Cargo workspace
- [x] 设置基础目录结构：
  ```
  auto-ui/
  ├── crates/
  │   ├── auto-ui/            # 核心抽象层
  │   ├── iced-examples/      # iced 示例
  │   └── gpui-examples/      # gpui 示例
  ├── scratch/                # Auto 语言原型
  └── docs/                   # 文档
  ```

#### 1.2 依赖配置 ✅
- [x] 添加 iced 0.14.0 依赖
- [x] 添加 gpui-component 0.5.0 依赖
- [x] 配置 workspace dependencies
- [x] 设置 .gitignore

#### 1.3 改进的抽象层设计 ✅
基于设计评估（[abstraction-evaluation.md](../design/abstraction-evaluation.md)），实现了与 Auto 语言高度对齐的抽象层：
- [x] Component trait: `update()` → `on()`
- [x] 泛型化 View: `View<M: Clone + Debug>`
- [x] ViewBuilder 链式 API
- [x] 直接消息存储（无 Option 包装）

**核心抽象**：
```rust
pub trait Component: Sized + Debug {
    type Msg: Clone + Debug + 'static;
    fn on(&mut self, msg: Self::Msg);
    fn view(&self) -> View<Self::Msg>;
}

pub enum View<M: Clone + Debug> {
    Empty,
    Text(String),
    Button { label: String, onclick: M },
    Row { children, spacing, padding },
    Column { children, spacing, padding },
    Input { placeholder, value, on_change: Option<M> },
    Checkbox { is_checked, label, on_toggle: Option<M> },
}
```

#### 1.4 示例实现 ✅
- [x] auto-ui 示例：counter_component.rs, all_components.rs
- [x] iced-examples：hello, counter, button, checkbox, circle, dropdown
- [x] gpui-examples：counter, layout, button（基本可用）

#### 1.5 文档完成 ✅
- [x] [phase1-summary.md](../phase1-summary.md) - Phase 1 总结
- [x] [phase1-abstraction-implementation.md](../phase1-abstraction-implementation.md) - 抽象层实现总结
- [x] [ui-frameworks-comparison.md](../analysis/ui-frameworks-comparison.md) - 框架对比分析
- [x] [abstraction-evaluation.md](../design/abstraction-evaluation.md) - 设计评估
- [x] [execution-mode-analysis.md](../design/execution-mode-analysis.md) - 执行模式分析

---

### Phase 2: 核心抽象层（2-3 周）

#### 2.1 定义核心 Trait

```rust
// 抽象组件接口
pub trait Component {
    type Message;
    type Props;

    fn view(&self) -> View;
    fn update(&mut self, msg: Self::Message) -> Command<Self::Message>;
}

// 抽象视图节点
pub enum View {
    Empty,
    Text(String),
    Container(Box<dyn Component>),
    Row(Vec<View>),
    Col(Vec<View>),
    // ...
}

// 后端抽象
pub trait Backend {
    type Renderer;
    fn run(app: impl Application<Self>);
}
```

#### 2.2 状态管理
- [ ] 实现 ELM 风格的 Model-Update-View 循环
- [ ] 消息传递机制
- [ ] 命令模式（Command）处理副作用

#### 2.3 布局系统
- [ ] 抽象布局接口
- [ ] Flex 布局（row/col）
- [ ] 绝对定位支持
- [ ] 响应式尺寸计算

---

### Phase 3: Iced 后端实现（3-4 周）

#### 3.1 基础组件适配
- [ ] Text / Label
- [ ] Button
- [ ] Input / TextBox
- [ ] Container

#### 3.2 布局组件
- [ ] Row / Column
- [ ] Center / Align
- [ ] Padding / Margin
- [ ] Scroll

#### 3.3 表单组件
- [ ] TextInput
- [ ] PasswordInput
- [ ] CheckBox
- [ ] Radio
- [ ] Select

#### 3.4 高级组件
- [ ] List / Table
- [ ] Dialog / Modal
- [ ] Menu
- [ ] Tabs

#### 3.5 样式系统
- [ ] 主题定义
- [ ] 样式继承
- [ ] 动态样式绑定

---

### Phase 4: Auto 语言集成（2-3 周）

#### 4.1 代码生成
- [ ] 编写 Transpiler（Auto → Rust）
- [ ] 生成 iced 应用代码
- [ ] 模板系统

#### 4.2 编译流程
```
.at 文件 → Parser → AST → Transpiler → Rust 代码 → 编译运行
```

#### 4.3 开发工具
- [ ] 热重载（file watcher）
- [ ] 错误提示
- [ ] 调试支持

---

### Phase 5: 示例与测试（2 周）

#### 5.1 核心示例
基于 [scratch/](scratch/) 的原型实现：
- [ ] Counter（计数器）
- [ ] Button（按钮）
- [ ] Login（登录表单）
- [ ] Layouts（布局展示）
- [ ] TodoMVC（完整应用）

#### 5.2 测试
- [ ] 单元测试（核心逻辑）
- [ ] 集成测试（组件渲染）
- [ ] 跨平台测试（Win/Mac/Linux）

---

### Phase 6: GPUI 后端（第二阶段，未来）

当 iced 后端稳定后，添加 gpui 支持：
- [ ] GPUI widget 适配
- [ ] 事件系统桥接
- [ ] 渲染管线
- [ ] 性能优化

---

## 里程碑

| 里程碑 | 目标 | 预计时间 | 状态 |
|--------|------|----------|------|
| M1 | 项目结构搭建完成 | Week 1 | ✅ 完成 |
| M2 | 核心抽象层定义完成 | Week 3 | ✅ 完成 |
| M3 | Iced 基础组件可用 | Week 6 | ⏳ 进行中 |
| M4 | Auto 语言可运行简单示例 | Week 9 | 📅 待开始 |
| M5 | Counter/Login 示例完成 | Week 11 | 📅 待开始 |
| M6 | 文档和测试完善 | Week 12 | 📅 待开始 |
| M7 | GPUI 后端（可选） | 未来 | 📅 待开始 |

---

## 风险与挑战

### 技术风险
1. **iced API 变化**：选择稳定版本，锁定依赖
2. **性能问题**：虚拟 DOM 的 diff 算法需要优化
3. **跨平台兼容性**：需要多平台测试

### 设计挑战
1. **抽象层设计**：如何在抽象和性能之间平衡
2. **Auto 语言表达力**：确保语法足够简洁强大
3. **消息传递**：复杂场景下的消息流管理

### 缓解措施
- 增量迭代，先实现简单场景
- 充分的单元测试和集成测试
- 参考 ELM, React 等成熟框架的设计

---

## 下一步行动

### 立即开始
1. ✅ 创建项目规划文档（本文档）
2. ✅ 初始化 Cargo workspace
3. ✅ 搭建基础目录结构
4. ✅ 实现第一个 "Hello World" 示例（纯 iced）
5. ✅ 定义核心 Trait（改进版）
6. ⏳ 开始 Phase 2：Iced 适配器实现

### 本周目标（Phase 2）
- [ ] 实现 Iced 适配器（auto-ui-iced crate）
- [ ] 创建使用抽象层的 Counter 示例
- [ ] 验证抽象层到 Iced 的转换
- [ ] 完善文档和测试

---

## Phase 1 完成总结 ✅

### 完成日期
2025-01-19

### 主要成果

#### 1. 项目基础设施 ✅
- ✅ Cargo workspace 配置完成
- ✅ 三个 crates 创建：auto-ui, iced-examples, gpui-examples
- ✅ 依赖配置：iced 0.14.0, gpui-component 0.5.0
- ✅ .gitignore 配置

#### 2. 核心抽象层设计 ✅
**Component Trait 改进**：
- `update()` → `on()` (对齐 Auto 的 `fn on(ev Msg)`)
- 移除 `Command` 返回值（简化）
- 泛型化 `View<Self::Msg>`（类型安全）

**View 枚举改进**：
- 泛型化 `View<M: Clone + Debug>`
- 直接消息存储：`onclick: M`（非 `Option<String>`）
- 支持 Text, Button, Row, Column, Input, Checkbox

**ViewBuilder 链式 API**：
```rust
View::col()
    .spacing(10)
    .padding(20)
    .child(View::button("+", Msg::Inc))
    .child(View::text("Hello"))
    .build()
```

#### 3. 示例实现 ✅
**auto-ui 示例**（2 个）：
- [counter_component.rs](../../crates/auto-ui/examples/counter_component.rs) - 基础 Counter
- [all_components.rs](../../crates/auto-ui/examples/all_components.rs) - 所有组件展示

**iced-examples**（6 个）：
- hello, counter, button, checkbox, circle, dropdown
- 全部可编译运行

**gpui-examples**（3 个）：
- counter, layout, button
- 基本可运行

#### 4. 文档完成 ✅
- [phase1-summary.md](../phase1-summary.md) - 原始总结
- [phase1-abstraction-implementation.md](../phase1-abstraction-implementation.md) - 抽象层实现
- [ui-frameworks-comparison.md](../analysis/ui-frameworks-comparison.md) - 框架对比
- [abstraction-evaluation.md](../design/abstraction-evaluation.md) - 设计评估
- [execution-mode-analysis.md](../design/execution-mode-analysis.md) - 执行模式分析
- [unified-abstraction.md](../design/unified-abstraction.md) - 统一抽象设计

### 技术亮点

1. **类型安全**：编译时消息类型检查，无运行时字符串匹配
2. **零成本抽象**：View enum 纯数据结构，编译期优化
3. **简洁 API**：链式调用流畅自然，Builder 模式简化布局
4. **Auto 对齐**：语法映射清晰，易于理解和实现

### 与 Auto 语言的映射

| Auto | 抽象层 | 说明 |
|------|-------|------|
| `widget` | `impl Component` | 组件定义 |
| `fn on(ev Msg)` | `fn on(&mut self, msg: Self::Msg)` | 消息处理 |
| `fn view() View` | `fn view(&self) -> View<Self::Msg>` | 视图渲染 |
| `col { }` | `View::col().child(...).build()` | 垂直布局 |
| `onclick: Msg.Inc` | `View::button("label", Msg::Inc)` | 事件绑定 |

### 关键经验

1. **iced 0.14 API 变化**：
   - `iced::run()` 只需 2 个参数（update + view）
   - `update()` 无返回值
   - `view()` 返回 `Element<'_, Message>`

2. **抽象层设计原则**：
   - 简单优于复杂（KISS）
   - 贴近 Auto 语言语法
   - 易于理解和实现
   - 支持两个框架

### Git 提交记录

```
3eb203b chore: add .gitignore for Rust project
3be3e99 feat(phase1): implement improved abstraction layer aligned with Auto language
de5a085 feat: complete Phase 1 - setup workspace and examples
```

### 下一步：Phase 2 - Iced 适配器

**目标**：
1. 实现 `auto-ui-iced` crate
2. 将 `View<M>` 转换为 `iced::Element<'_, M>`
3. 创建使用抽象层的 Counter 示例
4. 验证类型安全和事件处理

**技术要点**：
- trait `IntoIcedElement<M>`
- 消息桥接和事件绑定
- 渲染管线设计
- 性能优化考虑

---

## 参考资料

- [Iced 官方文档](https://docs.iced.rs/)
- [Iced GitHub](https://github.com/iced-rs/iced)
- [ELM 架构](https://guide.elm-lang.org/architecture/)
- [React 架构](https://react.dev/learn/understanding-your-ui-as-a-tree)
- [GPUI](https://github.com/zed-industries/zed)
- [GPUI-Component](https://github.com/longbridgeapp/gpui-component)
