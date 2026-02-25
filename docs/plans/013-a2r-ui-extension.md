# Plan 013: Extend a2r Transpiler for AutoUI

**Status**: 📋 Planned
**Created**: 2025-02-25
**Priority**: High
**Complexity**: Medium-High
**Estimated Timeline**: 20-28 days (MVP: 3-5 days)
**Last Updated**: 2025-02-25

## Overview

Extend the a2r (Auto-to-Rust) transpiler in `auto-lang` to generate `Component` trait implementations for Auto widgets. This plan includes:
1. **Parser Extension Plugin System** - Support UI DSL syntax (`widget`, `msg`, `model`)
2. **Hierarchical Message System** - Auto-generated `AppMsg` with `From` trait upcasting
3. **Default Style Support** - Component-level Tailwind style defaults
4. **a2r UI Extension** - Code generation for Component trait

## Motivation

### Problem 1: Transpiler Duplication
Currently, AutoUI has its own transpiler (`crates/auto-ui/src/trans/`) that is:
- Largely incomplete (returns placeholder comments)
- Duplicates logic from a2r
- Generates non-idiomatic Rust code

### Problem 2: Syntax Not Friendly to Frontend Developers
Current Auto syntax is too "systems programming" style:
```auto
type Counter is Widget {
    count int
    fn view() View { ... }
    fn on(ev CounterMsg) { ... }
}
```

Frontend developers prefer DSL-style syntax:
```auto
widget Counter {
    model { count int }
    view { ... }
    on { CounterMsg.Inc => { .count += 1 } }
    style { px-8 py-8 }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Auto Language Parser                      │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                  Extension Plugin System                 ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      ││
│  │  │ UI Dialect  │  │  Future     │  │  Custom     │      ││
│  │  │ (widget,msg)│  │  Dialects   │  │  Dialects   │      ││
│  │  └─────────────┘  └─────────────┘  └─────────────┘      ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Standard Auto AST                         │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    a2r Transpiler                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              RustUI Extension                            ││
│  │  - Generate AppMsg enum (hierarchical)                   ││
│  │  - Generate From<T> for Msg types (upcasting)            ││
│  │  - Generate Component with default_style()               ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Extension location | `auto-lang` | Single source of truth for all Auto→Rust transpilation |
| Generated trait | `Component` | Matches existing auto-ui API |
| Message enum naming | `TypeNameMsg` | Clear separation of message types |
| Message hierarchy | Nested enum + `From` trait | Type-safe upcasting, parent can listen to children |
| Style support | `default_style()` in Component | Component-level defaults, Tailwind syntax |
| DSL syntax | Parser plugin system | Extensible, future dialects can be added |
| Initial widget scope | Basic (col, row, text, button, input) | Focus on core functionality first |

---

## 🎯 Phase 0.5: MVP - counter_new.at 端到端运行 (3-5 days)

### 目标

将 `scratch/counter_new.at` 成功转换为 Rust 代码，并在 GPUI 和 Iced 两个后端都能渲染运行。

**这是第一个里程碑**，后续所有 Phase 都建立在此 MVP 之上。

### MVP 范围界定

| 功能 | MVP 策略 | 完整实现 (后续 Phase) |
|------|----------|----------------------|
| DSL 语法转换 | 硬编码预处理函数 | Parser 插件系统 |
| 层级消息 (AppMsg) | ❌ 跳过（单组件不需要） | Phase 2 |
| style 块 | 硬编码到 view 中 | Phase 3 |
| 组件数量 | 仅支持 Counter | 所有基础组件 |

### MVP 架构

```
counter_new.at (DSL)
       ↓
┌─────────────────────────────────┐
│  dsl_preprocess(source)         │  ← 硬编码的 DSL → Auto 转换
│  - msg → enum                   │
│  - widget → type is Widget      │
│  - model → 字段                 │
│  - view → fn view()             │
│  - on → fn on(ev Msg)           │
│  - button + → button "+"        │
│  - h2 > ${x} → h2 `${x}`        │
└─────────────────────────────────┘
       ↓
Standard Auto AST
       ↓
┌─────────────────────────────────┐
│  RustUITrans (简化版)           │  ← 最小化代码生成
│  - 生成 CounterMsg enum         │
│  - 生成 Counter struct          │
│  - 生成 impl Component          │
│  - 生成 view() 返回 View::col() │
└─────────────────────────────────┘
       ↓
counter.rs (Rust)
       ↓
┌─────────────────────────────────┐
│  GPUI / Iced 后端渲染           │
└─────────────────────────────────┘
```

### MVP 任务清单

#### 0.5.1 DSL 预处理器（硬编码）- 1 day

**文件**: `auto-ui/src/trans/dsl_preprocess.rs`

```rust
/// MVP: 硬编码的 DSL → Auto 转换
pub fn dsl_preprocess(source: &str) -> String {
    let mut result = source.to_string();
    
    // 1. 添加 prelude
    if !result.contains("use auto.ui:") {
        result = format!("use auto.ui: View, widget, app, center, col, button, h2\n\n{}", result);
    }
    
    // 2. msg → enum
    result = transform_msg_to_enum(&result);
    
    // 3. widget → type is Widget
    result = transform_widget_to_type(&result);
    
    // 4. button + → button "+"
    result = transform_inline_buttons(&result);
    
    // 5. h2 > ${x} → h2 `${x}`
    result = transform_content_shorthand(&result);
    
    result
}

fn transform_msg_to_enum(source: &str) -> String {
    // 正则替换: msg X { } → enum X { }
    // 实现...
}

fn transform_widget_to_type(source: &str) -> String {
    // 解析 widget X { model { } view { } on { } style { } }
    // 转换为 type X is Widget { fields fn view() fn on() }
    // 实现...
}
```

**任务**:
- [ ] 实现 `transform_msg_to_enum()`
- [ ] 实现 `transform_widget_to_type()` 
  - [ ] 提取 `model { }` 内容作为字段
  - [ ] 转换 `view { }` → `fn view() View { }`
  - [ ] 转换 `on { }` → `fn on(ev Msg) { is ev { } }`
  - [ ] 处理 `style { }` (暂时忽略或硬编码)
- [ ] 实现 `transform_inline_buttons()` - `button +` → `button "+"`
- [ ] 实现 `transform_content_shorthand()` - `h2 > ${x}` → `h2 \`${x}\``
- [ ] 单元测试：输入 `counter_new.at`，输出标准 Auto 语法

#### 0.5.2 代码生成器（最小化）- 1-2 days

**文件**: `auto-ui/src/trans/mvp_gen.rs`

```rust
/// MVP: 为 Counter 生成 Rust 代码
pub fn generate_counter_widget(ast: &Code) -> Result<String, String> {
    let mut code = String::new();
    
    // 找到 enum CounterMsg
    // 找到 type Counter is Widget
    
    // 生成:
    // 1. #[derive(Clone, Debug, PartialEq)]
    //    pub enum CounterMsg { Inc, Dec }
    //    
    // 2. #[derive(Debug)]
    //    pub struct Counter { count: i32 }
    //    
    // 3. impl Counter { pub fn new(count: i32) -> Self { ... } }
    //    
    // 4. impl Component for Counter {
    //      type Msg = CounterMsg;
    //      fn on(&mut self, msg: Self::Msg) { ... }
    //      fn view(&self) -> View<Self::Msg> { ... }
    //    }
    
    Ok(code)
}
```

**任务**:
- [ ] 提取 enum 定义并生成 Rust enum
- [ ] 提取 struct 字段并生成 Rust struct
- [ ] 生成 `impl Counter { new() }` 构造函数
- [ ] 生成 `impl Component for Counter`
  - [ ] `type Msg = CounterMsg`
  - [ ] `fn on()` - 转换 `is ev { }` 为 `match msg { }`
  - [ ] `fn view()` - 转换 `col { button... }` 为 `View::col().child(...).build()`

#### 0.5.3 GPUI 后端集成 - 0.5 day

**文件**: `auto-ui-gpui/examples/counter_dsl.rs`

```rust
// 手动粘贴生成的代码，验证可以编译和运行
use auto_ui::prelude::*;
use auto_ui_gpui::GpuiBackend;

// ... 生成的 Counter 代码 ...

fn main() {
    GpuiBackend::run(Counter::new(0));
}
```

**任务**:
- [ ] 创建 GPUI 示例
- [ ] 手动测试生成的 Counter 代码
- [ ] 验证按钮点击可以改变 count

#### 0.5.4 Iced 后端集成 - 0.5 day

**文件**: `auto-ui-iced/examples/counter_dsl.rs`

**任务**:
- [ ] 创建 Iced 示例
- [ ] 验证同样代码在 Iced 后端也能运行

### MVP 验收标准

```
✅ counter_new.at 经过 DSL 预处理 → 标准 Auto 语法
✅ 标准 Auto AST → Rust 代码（Counter struct + Component impl）
✅ 生成的 Rust 代码编译通过
✅ GPUI 后端：显示 Counter，点击 +/- 按钮 count 会变化
✅ Iced 后端：同上
```

### MVP 完成后

MVP 完成后，可以选择：
1. **继续完善 DSL 语法**：添加更多组件支持
2. **重构为插件系统**：将硬编码转换为 Phase 0 的插件架构
3. **添加层级消息**：支持多组件场景

---

## Phase 0: Parser Extension Plugin System (4-5 days)

### 0.1 Plugin System Design

**Goal**: Create an extensible plugin system for Parser that allows different dialects to transform syntax before standard AST generation.

**Location**: `auto-lang/crates/auto-lang/src/parser/plugin.rs`

```rust
/// Parser plugin trait for syntax extensions
pub trait ParserPlugin {
    /// Plugin name for identification
    fn name(&self) -> &str;
    
    /// Keywords introduced by this plugin
    fn keywords(&self) -> &[&str];
    
    /// Transform DSL syntax to standard Auto syntax
    /// Returns transformed source code
    fn transform(&self, source: &str) -> Result<String, PluginError>;
    
    /// Post-process AST after parsing (optional)
    fn post_process(&self, ast: &mut Code) -> Result<(), PluginError> {
        Ok(()) // Default: no-op
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn ParserPlugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
    
    pub fn register(&mut self, plugin: Box<dyn ParserPlugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn transform_all(&self, source: &str) -> Result<String, PluginError> {
        let mut result = source.to_string();
        for plugin in &self.plugins {
            result = plugin.transform(&result)?;
        }
        Ok(result)
    }
}
```

### 0.2 UI Dialect Plugin

**Location**: `auto-lang/crates/auto-lang/src/parser/plugins/ui_dialect.rs`

```rust
use super::{ParserPlugin, PluginError};

/// UI Dialect plugin - transforms widget/model/view/on/style syntax
pub struct UIDialectPlugin;

impl ParserPlugin for UIDialectPlugin {
    fn name(&self) -> &str { "ui_dialect" }
    
    fn keywords(&self) -> &[&str] {
        &["widget", "msg", "model", "view", "on", "style"]
    }
    
    fn transform(&self, source: &str) -> Result<String, PluginError> {
        let mut result = source.to_string();
        
        // 1. Add prelude if not present
        result = self.add_prelude(result);
        
        // 2. Transform `msg X { }` → `enum X { }`
        result = self.transform_msg_to_enum(result);
        
        // 3. Transform `widget X { model { } view { } on { } style { } }`
        //    → `type X is Widget { fields fn view() { } fn on() { } }`
        result = self.transform_widget_to_type(result);
        
        // 4. Transform inline node syntax: `button + { }` → `button "+" { }`
        result = self.transform_inline_nodes(result);
        
        // 5. Transform content shorthand: `h2 > text` → `h2 \`text\``
        result = self.transform_content_shorthand(result);
        
        Ok(result)
    }
}
```

### 0.3 DSL → Standard Auto Transformations

| DSL Syntax | Standard Auto | Notes |
|------------|---------------|-------|
| `msg X { A, B }` | `enum X { A, B }` | Direct replacement |
| `widget X { model { f T } }` | `type X is Widget { f T }` | Extract model fields |
| `view { col { } }` | `fn view() View { col { } }` | Wrap in function |
| `on { A => { } }` | `fn on(ev Msg) { is ev { A => { } } }` | Wrap in function + is |
| `style { px-8 }` | `_style str = "px-8"` | Add field + apply in view |
| `button + { }` | `button "+" { }` | Inline string arg |
| `h2 > Text` | `h2 \`Text\`` | Content shorthand |

### 0.4 Example Transformation

**Input (DSL):**
```auto
msg CounterMsg { Inc, Dec }

widget Counter {
    model { count int }
    
    view {
        col {
            button + { onclick: CounterMsg.Inc }
            h2 > Current Count: ${count}
            button - { onclick: CounterMsg.Dec }
        }
    }
    
    on {
        CounterMsg.Inc => { .count += 1 }
        CounterMsg.Dec => { .count -= 1 }
    }
    
    style {
        px-8 py-8
    }
}

app CounterApp {
    center { Counter(count: 0) }
}
```

**Output (Standard Auto):**
```auto
use auto.ui: View, widget, app, center, col, button, h2

enum CounterMsg { Inc, Dec }

type Counter is Widget {
    count int
    _style str = "px-8 py-8"
    
    fn view() View {
        col {
            button "+" { onclick: CounterMsg.Inc }
            h2 `Current Count: ${count}`
            button "-" { onclick: CounterMsg.Dec }
        }
    }
    
    fn on(ev CounterMsg) {
        is ev {
            CounterMsg.Inc => { .count += 1 }
            CounterMsg.Dec => { .count -= 1 }
        }
    }
}

app CounterApp {
    center { Counter(count: 0) }
}
```

### 0.5 Tasks

- [ ] Design `ParserPlugin` trait
- [ ] Implement `PluginManager`
- [ ] Integrate plugins into `Parser::parse()`
- [ ] Implement `UIDialectPlugin`
  - [ ] `add_prelude()` - Add default imports
  - [ ] `transform_msg_to_enum()` - msg → enum
  - [ ] `transform_widget_to_type()` - widget → type is Widget
  - [ ] `transform_inline_nodes()` - `button +` → `button "+"`
  - [ ] `transform_content_shorthand()` - `h2 > text` → `h2 \`text\``
- [ ] Unit tests for each transformation
- [ ] Integration test with full DSL example

---

## Phase 1: Foundation (3-4 days)

### 1.1 Create UI Extension Module

**File**: `auto-lang/crates/auto-lang/src/trans/rust_ui.rs`

```rust
//! Auto-to-Rust UI Transpiler Extension
//! 
//! Generates Component trait implementations with:
//! - Hierarchical message types (AppMsg)
//! - Default style support
//! - View builder chains

use super::{Sink, Trans};
use super::rust::RustTrans;
use crate::ast::*;
use crate::AutoResult;
use std::collections::HashMap;

pub struct RustUITrans {
    base: RustTrans,
    /// Current widget being processed
    current_widget: Option<AutoStr>,
    /// Message variants for current widget
    messages: Vec<MessageVariant>,
    /// All widgets in current file (for hierarchical messages)
    all_widgets: Vec<WidgetInfo>,
    /// App name (for AppMsg generation)
    app_name: Option<AutoStr>,
}

pub struct MessageVariant {
    name: AutoStr,
    has_data: bool,
    data_type: Option<Type>,
}

pub struct WidgetInfo {
    name: AutoStr,
    msg_type: AutoStr,
    has_default_style: bool,
}

pub struct UITransConfig {
    pub generate_constructors: bool,
    pub derive_debug: bool,
    pub generate_app_msg: bool,  // Generate hierarchical AppMsg
}

impl RustUITrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            base: RustTrans::new(name),
            current_widget: None,
            messages: Vec::new(),
            all_widgets: Vec::new(),
            app_name: None,
        }
    }
}
```

### 1.2 Widget Detection & Classification

```rust
pub enum WidgetKind {
    Full,       // Has Widget spec + view + on
    ViewOnly,   // Has view() only
    None,       // Regular type
}

impl RustUITrans {
    fn is_widget(&self, type_decl: &TypeDecl) -> bool {
        type_decl.specs.iter().any(|s| s.as_str() == "Widget")
            || type_decl.methods.iter().any(|m| m.name == "view")
    }
}
```

### 1.3 Tasks

- [ ] Create `rust_ui.rs` module scaffold
- [ ] Define `RustUITrans` struct with widget tracking
- [ ] Implement widget detection logic
- [ ] Collect all widgets for hierarchical message generation

---

## Phase 2: Hierarchical Message System (3-4 days)

### 2.1 Design

**Problem**: How can parent components listen to child component messages?

**Solution**: Generate a top-level `AppMsg` enum that wraps all widget messages, with `From` trait implementations for upcasting.

### 2.2 Generated Code Structure

```rust
// Generated for each widget
#[derive(Clone, Debug, PartialEq)]
pub enum CounterMsg {
    Inc,
    Dec,
}

// Generated once per file (if multiple widgets exist)
#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Counter(CounterMsg),
    Form(FormMsg),
    // ... all widget messages
}

// Upcasting: CounterMsg → AppMsg
impl From<CounterMsg> for AppMsg {
    fn from(msg: CounterMsg) -> Self {
        AppMsg::Counter(msg)
    }
}

// Component uses its own Msg type
impl Component for Counter {
    type Msg = CounterMsg;
    
    fn on(&mut self, msg: Self::Msg) { ... }
}

// Parent component can use AppMsg to receive all messages
impl Component for App {
    type Msg = AppMsg;
    
    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::Counter(m) => { /* handle counter message */ }
            AppMsg::Form(m) => { /* handle form message */ }
        }
    }
}
```

### 2.3 Implementation

```rust
impl RustUITrans {
    /// Generate AppMsg enum that wraps all widget messages
    fn generate_app_msg(&self, sink: &mut Sink) -> AutoResult<()> {
        if self.all_widgets.len() <= 1 {
            // Single widget, no need for AppMsg
            return Ok(());
        }
        
        writeln!(sink.body, "/// Top-level message type for the application")?;
        writeln!(sink.body, "#[derive(Clone, Debug, PartialEq)]")?;
        writeln!(sink.body, "pub enum AppMsg {{")?;
        
        for widget in &self.all_widgets {
            writeln!(sink.body, "    {}({}Msg),", widget.name, widget.name)?;
        }
        
        writeln!(sink.body, "}}")?;
        writeln!(sink.body)?;
        
        // Generate From traits for upcasting
        for widget in &self.all_widgets {
            writeln!(sink.body, "impl From<{}Msg> for AppMsg {{", widget.name)?;
            writeln!(sink.body, "    fn from(msg: {}Msg) -> Self {{", widget.name)?;
            writeln!(sink.body, "        AppMsg::{}(msg)", widget.name)?;
            writeln!(sink.body, "    }}")?;
            writeln!(sink.body, "}}")?;
            writeln!(sink.body)?;
        }
        
        Ok(())
    }
}
```

### 2.4 Tasks

- [ ] Collect all widget message types during parsing
- [ ] Generate `AppMsg` enum with all widget variants
- [ ] Generate `From<WidgetMsg>` implementations for `AppMsg`
- [ ] Handle case of single widget (skip AppMsg)
- [ ] Test with multi-widget file

---

## Phase 3: Default Style Support (2-3 days)

### 3.1 Component Trait Extension

Add `default_style()` method to Component trait:

**File**: `auto-ui/src/component.rs`

```rust
pub trait Component: Sized + Debug {
    type Msg: Clone + Debug + 'static;
    
    fn on(&mut self, msg: Self::Msg);
    
    fn view(&self) -> View<Self::Msg>;
    
    /// Default style for this component (Tailwind CSS syntax)
    /// Override this to provide component-level default styles
    fn default_style(&self) -> Option<&str> {
        None
    }
    
    /// Helper to apply default style to a view builder
    fn with_default_style<M: Clone + Debug>(&self, view: View<M>) -> View<M> {
        if let Some(style) = self.default_style() {
            // Apply style to view (implementation depends on View API)
            view.with_style_string(style)
        } else {
            view
        }
    }
}
```

### 3.2 Generated Code

**Auto (DSL):**
```auto
widget Counter {
    model { count int }
    view { ... }
    style { px-8 py-8 bg-white rounded-lg }
}
```

**Generated Rust:**
```rust
impl Counter {
    const DEFAULT_STYLE: &str = "px-8 py-8 bg-white rounded-lg";
}

impl Component for Counter {
    type Msg = CounterMsg;
    
    fn default_style(&self) -> Option<&str> {
        Some(Self::DEFAULT_STYLE)
    }
    
    fn view(&self) -> View<Self::Msg> {
        View::col()
            .style(Self::DEFAULT_STYLE)  // Apply default style
            .child(...)
            .build()
    }
}
```

### 3.3 Tasks

- [ ] Add `default_style()` method to `Component` trait
- [ ] Add `with_style_string()` helper to View types
- [ ] Generate `DEFAULT_STYLE` const for widgets with style block
- [ ] Apply default style in generated `view()` method
- [ ] Test style inheritance and override

---

## Phase 4: View Expression Transpilation (3-4 days)

### 4.1 View Expression Mapping

| Auto Syntax | Generated Rust |
|-------------|----------------|
| `col(spacing: 10) { ... }` | `View::col().spacing(10).child(...).build()` |
| `row(padding: 5) { ... }` | `View::row().padding(5).child(...).build()` |
| `center { text(x) }` | `View::container(View::text(&self.x)).center().build()` |
| `text(self.count)` | `View::text(&self.count.to_string())` |
| `button("+", onclick: Inc)` | `View::button("+", CounterMsg::Inc)` |
| `input("placeholder", value: name)` | `View::input("placeholder").value(&self.name).build()` |

### 4.2 Layout Widget Implementations

```rust
impl RustUITrans {
    fn trans_col_node(&mut self, node: &Node, sink: &mut Sink) -> AutoResult<()> {
        let spacing = self.get_prop_u16(node, "spacing").unwrap_or(0);
        let padding = self.get_prop_u16(node, "padding").unwrap_or(0);
        
        write!(sink.body, "View::col()")?;
        if spacing > 0 { write!(sink.body, ".spacing({})", spacing)?; }
        if padding > 0 { write!(sink.body, ".padding({})", padding)?; }
        
        for child in &node.body.stmts {
            write!(sink.body, ".child(")?;
            self.trans_view_stmt(child, sink)?;
            write!(sink.body, ")")?;
        }
        write!(sink.body, ".build()")?;
        Ok(())
    }
}
```

### 4.3 Tasks

- [ ] Implement `trans_view_method()` with dispatch logic
- [ ] Implement layout widgets: `col`, `row`, `center`, `container`, `scrollable`
- [ ] Implement basic widgets: `text`, `button`, `input`
- [ ] Add helper methods for property extraction

---

## Phase 5: Integration (2-3 days)

### 5.1 Parser Integration

```rust
// auto-lang/src/parser.rs
impl Parser {
    pub fn parse_with_dialect(&mut self, dialect: &str) -> Result<Code, ParseError> {
        // Register appropriate plugin
        match dialect {
            "ui" => self.plugin_manager.register(Box::new(UIDialectPlugin)),
            _ => {}
        }
        
        // Transform source with plugins
        let transformed = self.plugin_manager.transform_all(&self.source)?;
        self.source = transformed;
        
        self.parse()
    }
}
```

### 5.2 Module Exports

```rust
// auto-lang/src/lib.rs
pub use parser::{Parser, PluginManager, ParserPlugin};
pub use parser::plugins::UIDialectPlugin;
pub use trans::rust_ui::{RustUITrans, transpile_ui, UITransConfig};
```

### 5.3 Tasks

- [ ] Integrate plugin system into Parser
- [ ] Export all new types from lib.rs
- [ ] Update auto-ui to use extended a2r
- [ ] Test end-to-end transpilation

---

## Phase 6: Testing (2-3 days)

### 6.1 Test Structure

```
auto-lang/test/
├── a2r/
│   ├── 200_widget_text/
│   ├── 201_widget_button/
│   ├── 203_widget_counter/
│   └── 206_widget_multi/      # Test hierarchical messages
└── dsl/
    ├── 100_basic_widget/      # Basic DSL → Auto transformation
    ├── 101_model_view_on/     # model/view/on blocks
    ├── 102_inline_syntax/     # `button +` syntax
    └── 103_style_block/       # style { } block
```

### 6.2 Tasks

- [ ] Create test directory structure
- [ ] Write DSL transformation tests
- [ ] Write hierarchical message tests
- [ ] Write default style tests
- [ ] All tests passing

---

## Phase 7: Cleanup & Documentation (1-2 days)

### 7.1 Deprecate Old Transpiler

- Mark `crates/auto-ui/src/trans/rust_gen.rs` as deprecated
- Remove `crates/auto-ui/src/trans/auto_ui_trans.rs`
- Simplify `crates/auto-ui/src/trans/api.rs`

### 7.2 Update Documentation

- Document DSL syntax in CLAUDE.md
- Document plugin system API
- Document generated code patterns

---

## Timeline

| Phase | Days | Milestone | 优先级 |
|-------|------|-----------|--------|
| **0.5 MVP** | **3-5** | **counter_new.at 端到端运行** | ⭐⭐⭐ |
| 0 | 4-5 | Parser plugin system + UI dialect | ⭐⭐ |
| 1 | 3-4 | UI extension module scaffold | ⭐⭐ |
| 2 | 3-4 | Hierarchical message generation | ⭐ |
| 3 | 2-3 | Default style support | ⭐ |
| 4 | 3-4 | View expression transpilation | ⭐⭐ |
| 5 | 2-3 | Integration | ⭐⭐ |
| 6 | 2-3 | Testing | ⭐⭐ |
| 7 | 1-2 | Cleanup & docs | ⭐ |
| **Total** | **23-33** | **Production ready** | |

### MVP 优先 (3-5 days)

**第一个里程碑**: 将 `counter_new.at` 在 GPUI 和 Iced 上跑起来

```
MVP 验收标准:
✅ DSL → Auto 转换正确
✅ 生成的 Rust 代码编译通过
✅ GPUI 后端可运行
✅ Iced 后端可运行
```

### 后续迭代 (20-28 days)

MVP 完成后，根据实际需求选择后续 Phase 的优先级。

---

## Success Criteria

### MVP 成功标准 (Phase 0.5)
- [ ] `counter_new.at` DSL 语法正确转换为标准 Auto
- [ ] 生成的 `Counter` struct 和 `Component` impl 编译通过
- [ ] GPUI 后端显示 Counter，按钮点击正常
- [ ] Iced 后端同上

### 完整版成功标准 (Phase 0-7)
- [ ] DSL syntax (`widget`, `msg`, `model`, `view`, `on`, `style`) transforms correctly
- [ ] Parser plugin system is extensible
- [ ] Hierarchical `AppMsg` generated with `From` traits
- [ ] Default style applied to components
- [ ] Basic widgets (col, row, text, button, input) transpile correctly
- [ ] Generated code compiles without errors
- [ ] All tests passing

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| DSL transformation edge cases | Medium | Medium | Comprehensive test cases |
| Plugin system complexity | Medium | Medium | Keep plugin API minimal |
| Hierarchical message overhead | Low | Low | Benchmark, optimize if needed |
| Style parsing errors | Low | Low | Graceful fallback |

---

## Future Enhancements

- Additional dialect plugins (e.g., SwiftUI-style, React-style)
- Style inheritance from parent components
- Generic widget support
- Widget composition patterns
- Hot-reload integration
