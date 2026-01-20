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
  │   ├── auto-ui/                    # 核心抽象层
  │   ├── auto-ui-iced/               # Iced 适配器
  │   ├── auto-ui-iced-examples/      # 抽象层 + Iced 示例
  │   ├── iced-examples/              # 纯 Iced 框架示例
  │   └── gpui-examples/              # 纯 GPUI 框架示例
  ├── scratch/                        # Auto 语言原型
  └── docs/                           # 文档
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

### Phase 2: Iced 适配器实现 ✅ **核心完成**（2025-01-19）

**状态**: 核心功能已完成，验证受阻于第三方依赖问题（naga 编译错误）

#### 2.1 auto-ui-iced Crate 创建 ✅

**文件结构**：
```
crates/auto-ui-iced/
├── Cargo.toml          # 依赖配置
└── src/
    └── lib.rs          # 核心适配器实现（167行）
```

**依赖配置**：
```toml
[dependencies]
auto-ui = { path = "../auto-ui" }
iced = { workspace = true }
```

#### 2.2 IntoIcedElement Trait 实现 ✅

**核心 Trait**：
```rust
pub trait IntoIcedElement<M: Clone + Debug + 'static> {
    fn into_iced(self) -> iced::Element<'static, M>;
}
```

**实现的组件转换**：
- ✅ `View::Empty` → `text("")`
- ✅ `View::Text(content)` → `text(content)`
- ✅ `View::Button { label, onclick }` → `button(text(label)).on_press(onclick)`
- ✅ `View::Row { children, spacing, padding }` → `row([...]).spacing(...).padding(...)`
- ✅ `View::Column { children, spacing, padding }` → `column([...]).spacing(...).padding(...)`
- ✅ `View::Input { placeholder, value, on_change }` → `text_input(&placeholder, &value).on_input(...)`
- ✅ `View::Checkbox { is_checked, label, on_toggle }` → `row![checkbox(is_checked), text(label)]`

#### 2.3 ComponentIced 扩展 Trait ✅

**为所有 Component 类型自动实现**：
```rust
pub trait ComponentIced: Component {
    fn view_iced(&self) -> iced::Element<'static, Self::Msg>;
    fn update(&mut self, msg: Self::Msg);
}

impl<T: Component> ComponentIced for T
where
    T::Msg: Clone + Debug + 'static,
{
    fn view_iced(&self) -> iced::Element<'static, Self::Msg> {
        self.view().into_iced()
    }
}
```

#### 2.4 Counter Abstract 示例 ✅

**文件**: `crates/iced-examples/src/bin/counter_abstract.rs`

**代码示例**：
```rust
#[derive(Default)]
struct Counter { count: i64 }

#[derive(Clone, Copy, Debug)]
enum Message { Increment, Decrement }

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(10)
            .padding(20)
            .child(View::button("+", Message::Increment))
            .child(View::text(self.count.to_string()))
            .child(View::button("-", Message::Decrement))
            .build()
    }
}

fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view_iced)
}
```

#### 2.5 技术亮点 ✅

1. **类型安全的消息传递**: 编译时类型检查，无运行时字符串匹配
2. **零成本抽象**: `View<M>` 纯数据结构，`into_iced()` 简单模式匹配，编译期优化
3. **无缝集成**: Component 类型自动获得 Iced 支持
4. **递归转换**: 支持任意深度的组件嵌套

#### 2.6 Naga 编译错误解决方案 ✅

**问题描述**：
- **错误**: `error[E0277]: the trait bound 'std::string::String: WriteColor' is not satisfied`
  ```
  error[E0277]: the trait bound `std::string::String: WriteColor' is not satisfied
    --> naga-27.0.3\src\error.rs:50:17
     |
  50 |                 writer.inner_mut(),
     |                 ^^^^^^^^^^^^^^^^^^ the trait `WriteColor` is not implemented for `std::string::String`
  ```
- **原因**: naga 27.0.3（iced 的 GPU 着色器编译依赖）在 Windows 平台的已知 bug
  1. naga 27.0.3 是 iced 0.14.0 的传递依赖（用于 GPU 着色器编译）
  2. naga 默认配置使用 `String` 作为诊断输出缓冲区
  3. `String` 没有实现 `termcolor` 库的 `WriteColor` trait
- **影响**: 无法在 Windows 上编译任何使用 iced 的应用

**解决方案**：
1. ✅ **启用 naga 的 termcolor feature**：在项目依赖中添加 `naga = { version = "27.0.3", features = ["termcolor"] }`
2. ✅ **termcolor feature 修复**：启用后，naga 使用 `NoColor<Vec<u8>>` 而不是 `String`，实现了 `WriteColor` trait

**实施步骤**：

在 `crates/iced-examples/Cargo.toml` 中添加：
```toml
[dependencies]
iced = { workspace = true }
auto-ui = { workspace = true }
auto-ui-iced = { path = "../auto-ui-iced" }

# 强制启用 naga 的 termcolor feature 以避免 Windows WriteColor trait 错误
naga = { version = "27.0.3", features = ["termcolor"] }
```

在 `crates/auto-ui-iced/Cargo.toml` 中添加：
```toml
[dependencies]
auto-ui = { path = "../auto-ui" }
iced = { workspace = true }

# 强制启用 naga 的 termcolor feature 以避免 Windows WriteColor trait 错误
naga = { version = "27.0.3", features = ["termcolor"] }
```

**原理说明**：

naga 的 `error.rs` 中有以下条件编译：
```rust
cfg_if::cfg_if! {
    if #[cfg(feature = "termcolor")] {
        // ✅ 使用 NoColor<Vec<u8>>，实现了 WriteColor
        type DiagnosticBufferInner = codespan_reporting::term::termcolor::NoColor<alloc::vec::Vec<u8>>;
    } else if #[cfg(feature = "stderr")] {
        type DiagnosticBufferInner = alloc::vec::Vec<u8>;
    } else {
        // ❌ 使用 String，没有实现 WriteColor（Windows 上会失败）
        type DiagnosticBufferInner = String;
    }
}
```

通过启用 `termcolor` feature，naga 会使用第一个分支，从而避免错误。

**其他尝试的方法（未成功）**：

1. **锁定 naga 版本为 25.0.1**
   ```toml
   [workspace.dependencies]
   naga = "=25.0.1"
   ```
   问题：iced 内部依赖 wgpu 27.x，而 wgpu 27.x 依赖 naga 27.x，workspace.lock 对传递依赖不起作用。

2. **使用 `[patch.crates-io]` 指向 git 仓库**
   ```toml
   [patch.crates-io]
   naga = { git = "https://github.com/gfx-rs/naga", tag = "25.0.1" }
   ```
   问题：git tag 格式不正确，无法找到对应的引用。

3. **设置环境变量 `NO_COLOR=1`**
   ```bash
   set NO_COLOR=1 && cargo build
   ```
   问题：这是运行时配置，不影响编译时期的 trait 检查。

4. **降级 wgpu 到 22.x**
   问题：iced 0.14.0 强制依赖 wgpu 27.x，无法降级。

**关键经验**：

1. **Feature 优先于版本锁定**：当遇到依赖 bug 时，优先检查是否有 feature 可以解决，而不是尝试降级版本
2. **传递依赖的控制**：workspace.dependencies 只影响直接依赖，对传递依赖的控制有限
3. **查看源代码**：直接查看依赖库的源代码（如 `error.rs`）比猜测更有效
4. **Windows 特定问题**：某些 trait 实现问题只在特定平台出现，需要跨平台测试

#### 2.7 验证方法 ✅

1. **编译验证** ✅
   ```bash
   $ cargo build --bin counter_abstract
   Finished `dev` profile in 12.41s
   ```

2. **运行验证** ✅
   ```bash
   $ cargo run --bin counter_abstract
   # GUI 窗口成功打开，显示计数器应用
   ```

3. **代码审查** ✅
   - Trait 定义正确
   - 所有 View 变体都有对应的转换
   - 递归转换逻辑正确
   - 消息类型传递正确

4. **API 设计验证** ✅
   ```rust
   // 简洁的 API
   let view = View::button("Click", Msg::Click);
   let element = view.into_iced();
   ```

5. **生命周期处理** ✅
   - 使用包装函数 `fn view(counter: &Counter) -> Element<'_, Message>` 来桥接生命周期
   - `Element<'static, Message>` → `Element<'_, Message>` 转换

#### 2.8 完成度评估

| 任务 | 状态 | 完成度 |
|------|------|--------|
| 创建 auto-ui-iced crate | ✅ 完成 | 100% |
| 实现 IntoIcedElement trait | ✅ 完成 | 100% |
| 实现所有组件转换 | ✅ 完成 | 100% |
| 创建 Counter 示例 | ✅ 完成 | 100% |
| 解决 Naga 编译错误 | ✅ 完成 | 100% |
| 运行验证 | ✅ 完成 | 100% |
| **Phase 2 总体** | **✅ 完全完成** | **100%** |

---

### Phase 3: Iced 后端实现（3-4 周）

#### 3.1 基础组件适配
- [x] Text / Label ✅
- [x] Button ✅
- [ ] Input / TextBox（需要改进设计以支持值获取）
- [x] Container ✅

#### 3.2 布局组件
- [x] Row / Column ✅
- [x] Center / Align ✅（Container 支持 center_x/center_y）
- [x] Padding / Margin ✅
- [ ] Scroll

#### 3.3 表单组件
- [ ] TextInput
- [ ] PasswordInput
- [x] CheckBox ✅
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

#### 3.6 示例应用 ✅

**TodoMVC 示例** (`todo.rs`) ✅
- **文件**: `crates/auto-ui-iced-examples/src/bin/todo.rs`
- **功能**:
  - 添加/删除待办事项
  - 标记完成状态
  - 过滤显示（All/Active/Completed）
  - 清除已完成项目
- **验证的组件**: Text, Button, Row, Column, 条件渲染
- **状态管理**: 复杂的列表状态和过滤逻辑
- **运行**: `cargo run --package auto-ui-iced-examples --bin todo`

**温度转换器示例** (`temp_converter.rs`) ✅
- **文件**: `crates/auto-ui-iced-examples/src/bin/temp_converter.rs`
- **功能**:
  - 摄氏度和华氏度双向转换
  - 增量调整温度
  - 重置功能
- **验证的组件**: Text, Button, 嵌套布局
- **数据流**: 双向数据绑定和计算值
- **运行**: `cargo run --package auto-ui-iced-examples --bin temp_converter`

**计数器示例** (`counter.rs`) ✅
- **文件**: `crates/auto-ui-iced-examples/src/bin/counter.rs`
- **功能**:
  - 基础计数器
  - 增量/减量操作
- **运行**: `cargo run --package auto-ui-iced-examples --bin counter`

**增强的 ViewBuilder API** ✅
- 添加了 `children()` 方法支持批量添加子组件
- 示例: `.children(vec![...])` 简化列表构建

**项目结构重构** ✅
- 将 `auto-ui-examples` 重命名为 `auto-ui-iced-examples`
- 明确区分：
  - `auto-ui-iced-examples/` - 抽象层 + Iced 后端示例
  - `iced-examples/` - 纯 Iced 框架示例（学习参考）
- 为未来添加 GPUI 后端建立清晰的命名模式

**Container 组件** (`container_demo.rs`) ✅
- **文件**: `crates/auto-ui-iced-examples/src/bin/container_demo.rs`
- **功能**:
  - 内边距（padding）控制
  - 固定宽高（width/height）
  - 水平/垂直居中（center_x/center_y）
  - 嵌套容器支持
- **API 设计**:
  ```rust
  View::container(child)
      .padding(20)
      .width(300)
      .height(100)
      .center_x()
      .center_y()
      .build()
  ```
- **实现要点**:
  - 使用 Builder 模式（`ViewContainerBuilder`）提供链式 API
  - 支持可选的宽高设置
  - 通过 `iced::widget::container` 映射到 Iced
- **运行**: `cargo run --package auto-ui-iced-examples --bin container_demo`

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
| M3 | Iced 基础组件可用 | Week 6 | ✅ 完成 |
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
6. ✅ Phase 2：Iced 适配器实现（核心完成）

### 下一步目标（Phase 3）
- [x] ✅ 创建 TodoMVC 示例
- [x] ✅ 创建温度转换器示例
- [x] ✅ 增强 ViewBuilder API（添加 children 方法）
- [ ] 改进 Input 组件设计（支持值获取）
- [ ] 添加 Container 组件
- [ ] 添加更多布局组件（Center/Align/Scroll）
- [ ] 实现样式系统
- [ ] 性能测试和优化

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

### 框架与工具
- [Iced 官方文档](https://docs.iced.rs/)
- [Iced GitHub](https://github.com/iced-rs/iced)
- [naga GitHub Issues](https://github.com/gfx-rs/naga/issues)
- [termcolor crate](https://docs.rs/termcolor/)
- [ELM 架构](https://guide.elm-lang.org/architecture/)
- [React 架构](https://react.dev/learn/understanding-your-ui-as-a-tree)
- [GPUI](https://github.com/zed-industries/zed)
- [GPUI-Component](https://github.com/longbridgeapp/gpui-component)
