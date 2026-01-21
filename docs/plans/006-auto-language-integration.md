# Plan 006: Auto Language Integration as UI Scripting Language

## Implementation Progress Summary

**Updated**: 2025-01-22

| Phase | Status | Completion | Notes |
|-------|--------|------------|-------|
| Phase 0: auto.ui Module | ✅ Complete | 2025-01-22 | Full module with all 12 UI types |
| Phase 1: Widget Macro | ✅ Complete | 2025-01-22 | `widget` and `app` macro expansion |
| Phase 2: Node → View | ✅ Complete | 2025-01-22 | Runtime interpretation with `convert_node()` |
| Phase 3: Rust Transpiler | ⏳ In Progress | 2025-01-22 | Framework complete, needs AST fixes |
| Phase 4: Hot-Reload | ⏳ Pending | - | Not started |
| Phase 5: Testing | ⏳ Pending | - | Not started |

**Overall Progress**: 50% complete (3 phases fully complete, 1 phase in progress)

---

# Original Plan

## Overview

**Objective**: Integrate Auto language as the primary UI scripting language for AutoUI, enabling developers to write UI components in `.at` files that can be either interpreted at runtime (for development/hot-reload) or transpiled to Rust (for production).

**Status**: 📋 **Planning**

**Created**: 2025-01-21

**Estimated Duration**: 4-5 weeks (including Phase 0: auto.ui module definition)

**Complexity**: High

---

## Background and Motivation

### Current State

**AutoUI Strengths**:
- ✅ Unified Component trait abstraction
- ✅ Multi-backend support (GPUI, Iced)
- ✅ Comprehensive styling system (Plan 004 + Plan 005)
- ✅ Working Rust API with View builders

**AutoUI Limitations**:
- ❌ UI code written in Rust - verbose and repetitive
- ❌ No hot-reload support for rapid iteration
- ❌ High boilerplate for simple components
- ❌ Steep learning curve for UI developers

**AutoLang Strengths** (from `d:\autostack\auto-lang`):
- ✅ Working parser and AST
- ✅ Multiple transpilers (C, Rust, Python, JavaScript)
- ✅ Config evaluation mode (`EvalMode::CONFIG`)
- ✅ Type system with traits
- ✅ Expression evaluation

**Integration Gap**:
- No connection between AutoLang parser and AutoUI Component system
- `.at` UI files exist but cannot be parsed or executed
- Missing runtime interpretation mode
- Missing build-time transpilation

### Why Auto as UI Language?

1. **Declarative Syntax**: Natural fit for UI component definition
2. **Type Safety**: Catches errors before runtime
3. **Less Boilerplate**: 5-10x less code than Rust equivalent
4. **Hot-Reload**: Rapid development iteration
5. **Unified Ecosystem**: Single language for logic and UI

### Example Comparison

**Rust (Current)**:
```rust
#[derive(Debug)]
struct Counter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Inc,
    Dec,
}

impl Component for Counter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(20)
            .child(View::text(format!("Count: {}", self.count)))
            .child(View::row()
                .spacing(8)
                .child(View::button("+", Msg::Inc))
                .child(View::button("-", Msg::Dec))
                .build()
            )
            .build()
    }
}
```

**Auto (Target)**:
```auto
widget Counter {
    count i64 = 0

    fn view() View {
        col(spacing: 16, padding: 20) {
            text("Count: {count}")
            row(spacing: 8) {
                button("+", Inc)
                button("-", Dec)
            }
        }
    }

    fn on(ev Msg) {
        is ev {
            Msg.Inc => { count += 1 }
            Msg.Dec => { count -= 1 }
        }
    }
}
```

**Benefits**: 70% less code, more readable, declarative style, hot-reload capable.

---

## Technical Architecture

### Design Philosophy

**Dual-Mode Operation**:
1. **Runtime Interpretation** (Development): Parse `.at` files → AST → Evaluate → `View<M>`
2. **Transpilation** (Production): Parse `.at` files → AST → Generate Rust code → Compile

**No New Keywords**: Use existing AutoLang syntax with macro expansion
- `widget` → Macro expanding to `type ... is Widget`
- `app` → Regular type declaration
- `fn view() View` → Component trait method
- `fn on(ev Msg)` → Message handler

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    .at UI Source File                        │
│  (widget Counter { ... }, app MyApp { ... })                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              AutoLang Parser (auto-lang crate)               │
│  - Lexical analysis                                          │
│  - AST construction                                          │
│  - Type checking                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                ▼                           ▼
┌──────────────────────────┐    ┌───────────────────────────┐
│  Runtime Interpretation  │    │   Transpilation to Rust   │
│  (Development Mode)      │    │   (Production Mode)       │
└──────────────────────────┘    └───────────────────────────┘
                │                           │
                ▼                           ▼
┌──────────────────────────┐    ┌───────────────────────────┐
│  AST → Value::Node       │    │  AST → Rust Code Gen      │
│  (EvalMode::CONFIG)      │    │  (Transpiler)             │
└──────────────────────────┘    └───────────────────────────┘
                │                           │
                ▼                           ▼
┌──────────────────────────┐    ┌───────────────────────────┐
│  Node → View<M>          │    │  Component impl in Rust   │
│  (auto_ui::node_converter)│   │  (.rs file)               │
└──────────────────────────┘    └───────────────────────────┘
                │                           │
                ▼                           ▼
┌──────────────────────────┐    ┌───────────────────────────┐
│  Backend Rendering       │    │  Cargo Compile            │
│  (GPUI/Iced)             │    │  → Binary                 │
└──────────────────────────┘    └───────────────────────────┘
```

---

## Implementation Plan

### Phase 0: auto.ui Module Definition (Week 0 - Prerequisite)

**Goal**: Define the `auto.ui` module with base types and traits that `.at` UI scripts will depend on.

**Critical Prerequisite**: The `scratch/hello.at` file imports symbols from `auto.ui`:
```auto
use auto.ui: View, widget, app, center, text
```

These symbols must exist before any `.at` UI file can be parsed.

#### 0.1 Module Structure

**Location**: `stdlib/auto/ui.at` (single file - AutoLang doesn't support folder-based modules yet)

**Important**: AutoLang currently doesn't support folder-based module systems. All UI type definitions must be in a single `ui.at` file.

**File Structure**:
```
stdlib/
└── auto/
        └── ui.at         # All UI types in one file (~350 lines)
```

**Symbol Linking**: The `stdlib/auto/ui.at` file should be:
1. Created in the `auto-ui` crate repository at `stdlib/auto/ui.at`
2. Symlinked or copied to the global Auto language stdlib path
3. Installed with AutoLang as the official UI module

**Future Enhancement**: When AutoLang adds folder-based module support, this can be split into:
```
stdlib/auto/ui/
├── __init__.at
├── widget.at
├── app.at
├── view.at
├── layouts.at
└── elements.at
```

#### 0.2 Complete ui.at File

**File**: `stdlib/auto/ui.at` (single file containing all UI type definitions)

```auto
// ============================================================================
// auto.ui Module - Unified UI Framework for Auto Language
// ============================================================================
// This file defines all UI types and components for AutoUI integration.
// Note: AutoLang doesn't support folder-based modules yet, so all definitions
// are in this single file. Future versions may split this into multiple files.
// ============================================================================

// ----------------------------------------------------------------------------
// Core Types
// ----------------------------------------------------------------------------

// Widget trait - base trait for all UI components
// All custom widgets (Hello, Counter, etc.) must implement this
type Widget {
    // Style string for Plan 005 integration
    // Example: "p-4 bg-blue-500 text-white"
    style str = ""

    // View method - returns the UI structure
    #[vm]
    fn view() View

    // Event handler - processes messages
    // Default implementation: no-op
    #[vm]
    fn on(ev Msg) {
        // Default: do nothing
    }
}

// App type - application entry point
// Apps are top-level containers that hold widgets
type App is Widget {
    // Window title
    title str = "AutoUI App"

    // Theme name (for future theming support)
    theme str = "default"

    // App inherits view() and on() from Widget
}

// View type - represents UI element tree
// This is a type marker for the transpiler
// Actual implementation is in Rust (auto_ui::view::View)
type View {
    // Opaque type - represents View<M> in Rust
    // The transpiler converts this to auto_ui::View<Msg>
}

// ----------------------------------------------------------------------------
// Layout Components
// ----------------------------------------------------------------------------

// Center layout - centers content both horizontally and vertically
type Center is Widget {
    child Widget

    fn view() View {
        // Implemented by transpiler - converts to View::container with center_x/center_y
    }
}

// Column layout - arranges children vertically
type Col is Widget {
    spacing u16 = 0
    padding u16 = 0
    children [~Widget]

    fn view() View {
        // Implemented by transpiler - converts to View::col()
    }
}

// Row layout - arranges children horizontally
type Row is Widget {
    spacing u16 = 0
    padding u16 = 0
    children [~Widget]

    fn view() View {
        // Implemented by transpiler - converts to View::row()
    }
}

// Container layout - generic container with sizing
type Container is Widget {
    padding u16 = 0
    width u16
    height u16
    child Widget

    fn view() View {
        // Converts to View::container
    }
}

// Scrollable layout - container with scrollbars
type Scrollable is Widget {
    width u16
    height u16
    child Widget

    fn view() View {
        // Converts to View::scrollable
    }
}

// ----------------------------------------------------------------------------
// Element Components
// ----------------------------------------------------------------------------

// Text element - displays text
type Text is Widget {
    content str

    fn view() View {
        // Converts to View::text(content)
    }
}

// Button element - clickable with message
type Button is Widget {
    label str
    onclick Msg

    fn view() View {
        // Converts to View::button(label, onclick)
    }
}

// Input element - text input field
type Input is Widget {
    placeholder str
    value str
    on_change Msg

    fn view() View {
        // Converts to View::input(...)
    }
}

// Checkbox element
type Checkbox is Widget {
    is_checked bool
    label str
    on_toggle Msg

    fn view() View {
        // Converts to View::checkbox(...)
    }
}

// Radio button element
type Radio is Widget {
    label str
    is_selected bool
    on_select Msg

    fn view() View {
        // Converts to View::radio(...)
    }
}

// Select dropdown element
type Select is Widget {
    options [~str]
    selected_index u16
    on_select Msg

    fn view() View {
        // Converts to View::select(...)
    }
}

// List element
type List is Widget {
    items [~Widget]
    spacing u16 = 0

    fn view() View {
        // Converts to View::list(...)
    }
}

// Table element
type Table is Widget {
    headers [~Widget]
    rows [[~Widget]]
    spacing u16 = 0
    col_spacing u16 = 0

    fn view() View {
        // Converts to View::table(...)
    }
}
```

**Total Lines**: ~350 lines

**Organization**:
- Core Types: Widget, App, View (3 types)
- Layout Components: Center, Col, Row, Container, Scrollable (5 types)
- Element Components: Text, Button, Input, Checkbox, Radio, Select, List, Table (8 types)

**Usage in .at files** (note lowercase usage for functions):
```auto
center {
    Hello("Hello")
}

col(spacing: 16, padding: 20) {
    text("Hello")
    button("Click", Msg)
}

row(spacing: 8) {
    button("+", Inc)
    button("-", Dec)
}
```

**Transpilation Rules**:
- `center { ... }` → `View::container(...).center_x().center_y()`
- `col(spacing: X, padding: Y) { ... }` → `View::col().spacing(X).padding(Y)`
- `row(spacing: X, padding: Y) { ... }` → `View::row().spacing(X).padding(Y)`

#### 0.3 Integration with AutoLang

**Installation Options**:

**Option A: Symlink to Global Stdlib**
```bash
# In auto-lang repository
cd stdlib/auto
ln -s ../../../auto-ui/stdlib/auto/ui.at ui.at

# Or copy
cp ../../../auto-ui/stdlib/auto/ui.at .
```

**Option B: AutoLang Feature Flag**
```toml
# auto-lang/Cargo.toml
[features]
default = []
ui = []  # Enable UI module

[build-dependencies]
auto-ui = { path = "../../../auto-ui/crates/auto-ui", optional = true }

# build.rs
fn main() {
    #[cfg(feature = "ui")]
    {
        let ui_module = std::fs::read_to_string(
            "../../auto-ui/stdlib/auto/ui.at"
        ).unwrap();
        std::fs::write("stdlib/auto/ui.at", ui_module).unwrap();
    }
}
```

**Option C: Runtime Module Loading** (Recommended)
```rust
// In auto-lang evaluator
pub fn load_ui_module() {
    let ui_path = std::env::var("AUTO_UI_STDLIB")
        .unwrap_or_else(|_| "../../auto-ui/stdlib/auto/ui.at".to_string());
    load_module("auto.ui", ui_path).ok();
}
```

#### 0.4 Transpiler Handling

**Key Insight**: The `auto.ui` module definitions are **stubs** for type checking. The actual implementation is in Rust.

**Transpiler Strategy**:

1. **Type Declarations**: The transpiler knows about these types and maps them to Rust:
   ```auto
   // In .at file
   text("Hello")  // Type: Text

   // Generated Rust
   View::text("Hello")  // Calls auto_ui::View::text()
   ```

2. **Special Forms**: Layout components are special-cased:
   ```auto
   col(spacing: 16) { ... }

   // Generated Rust
   View::col()
       .spacing(16)
       .child(...)
       .child(...)
       .build()
   ```

3. **Widget Expansion**: User widgets expand to:
   ```auto
   widget Hello { ... }

   // Expands to
   type Hello is Widget { ... }

   // Generates Rust
   pub struct Hello { ... }
   impl Component for Hello { ... }
   ```

#### 0.5 Validation

**Test**: `scratch/hello.at` should parse without errors:

```bash
# Test parsing
auto-lang parse scratch/hello.at

# Expected: Success, no "unknown symbol" errors
```

**Validation Checklist**:
- ✅ `View` type exists
- ✅ `Widget` type exists with `view()` and `on()` methods
- ✅ `App` type exists
- ✅ `center` exists (as Center type, used as function)
- ✅ `text` exists (as Text type, used as function)
- ✅ Module can be imported: `use auto.ui: View, Widget, App, Center, Text`

**Deliverables**:
- ✅ `stdlib/auto/ui.at` - Complete UI module (~350 lines)
  - Core types: Widget, App, View
  - Layout components: Center, Col, Row, Container, Scrollable
  - Element components: Text, Button, Input, Checkbox, Radio, Select, List, Table
- ✅ Installation/symlink script or build.rs integration
- ✅ Documentation of module structure
- ✅ Test: `scratch/hello.at` parses successfully

**Acceptance Criteria**:
- ✅ All symbols in `scratch/hello.at` resolve correctly
- ✅ `use auto.ui: ...` works
- ✅ Type checking passes for widget definitions
- ✅ Module is accessible to both AutoLang parser and transpiler
- ✅ Documentation complete for all exported symbols

**Estimated Time**: 2-3 days

---

### Phase 1: AutoLang Extensions for UI (Week 1)

**Goal**: Extend AutoLang to understand UI-specific syntax without breaking existing functionality.

#### 1.1 Widget Macro System

**File**: `d:\autostack\auto-lang\crates\auto-lang\src\macro\ui.rs` (new)

**Approach**: Text-level preprocessing (simpler) → AST-level macros (future)

**Implementation**:
```rust
// crates/auto-lang/src/macro/ui.rs
use regex::Regex;

pub fn expand_widget_macros(code: &str) -> String {
    let mut result = code.to_string();

    // 1. widget Name { ... } → type Name is Widget { ... }
    let widget_regex = Regex::new(r"widget\s+(\w+)\s*\{").unwrap();
    result = widget_regex.replace_all(&result, "type $1 is Widget {").to_string();

    // 2. Handle style: "..." field (special syntax for widget-level styling)
    // Convert to: style_attr: "...", handled by trait

    result
}
```

**Integration**:
```rust
// crates/auto-lang/src/lib.rs
#[cfg(feature = "ui")]
pub mod macro {
    pub mod ui;

    pub use ui::expand_widget_macros;
}

// In parser entry point
pub fn parse_ui(code: &str) -> AutoResult<AST> {
    let expanded = expand_widget_macros(code)?;
    parse(&expanded)
}
```

#### 1.2 Widget and App Trait Definitions

**File**: `d:\autostack\auto-lang\stdlib\auto\widget.at` (new)

```auto
// Widget trait - base trait for all UI components
type Widget {
    #[vm]
    fn view() View

    #[vm]
    fn on(ev Msg) {
        // Default: no-op event handler
    }

    // Optional style string (Plan 005 integration)
    style str = ""
}

// App type - application entry point
type App is Widget {
    title str = "AutoUI App"
    theme str = "default"

    #[vm]
    fn run() {
        // Entry point - called by runtime
    }
}
```

**Rust Representation** (in `auto-ui` crate):
```rust
// Will be used by transpiler to generate Component impls
pub trait WidgetComponent {
    type Msg;
    fn view(&self) -> View<Self::Msg>;
    fn on(&mut self, msg: Self::Msg);
}
```

#### 1.3 Parser Extensions

**No parser changes needed initially** - using macro expansion approach.

**Future enhancement**: AST-level macros if text-level proves insufficient.

**Deliverables**:
- ✅ `crates/auto-lang/src/macro/ui.rs` - Widget macro expansion
- ✅ `stdlib/auto/widget.at` - Widget/App trait definitions
- ✅ Unit tests for macro expansion
- ✅ Updated auto-lang `Cargo.toml` with `ui` feature

**Acceptance Criteria**:
- ✅ `widget Hello { msg str }` expands to `type Hello is Widget { msg str }`
- ✅ Macro expansion handles nested braces correctly
- ✅ No breaking changes to existing AutoLang syntax

---

### Phase 2: Node to View Conversion (Week 1-2)

**Goal**: Convert AutoLang's `Value::Node` to AutoUI's `View<M>` for runtime interpretation.

#### 2.1 UINode Adapter Structure

**File**: `d:\autostack\auto-ui\crates\auto-ui\src\node_converter.rs` (new)

```rust
use auto_val::Value;
use crate::view::View;
use crate::style::Style;
use std::collections::HashMap;

/// Wrapper around AutoLang's Value::Node for UI components
pub struct UINode {
    pub kind: UINodeKind,
    pub props: HashMap<String, Value>,
    pub children: Vec<UINode>,
    pub style: Option<Style>,
}

pub enum UINodeKind {
    Widget(String),      // Custom widget: Hello, Counter, etc.
    Layout(LayoutKind),  // Layout containers: col, row, center
    Element(ElementKind), // Basic elements: text, button, input
}

pub enum LayoutKind {
    Col, Row, Center,
    Container, Scrollable,
}

pub enum ElementKind {
    Text, Button, Input,
    Checkbox, Radio, Select,
    List, Table,
}
```

#### 2.2 Value::Node to UINode Conversion

```rust
impl UINode {
    /// Convert AutoLang Value::Node to UINode
    pub fn from_value(value: &Value) -> AutoResult<Self> {
        match value {
            Value::Node(node) => {
                let kind = Self::detect_kind(&node.name)?;
                let props = Self::extract_props(node)?;
                let children = Self::extract_children(node)?;
                let style = Self::extract_style(&props)?;

                Ok(UINode { kind, props, children, style })
            }
            _ => Err("Expected Value::Node for UI conversion".into()),
        }
    }

    fn detect_kind(name: &str) -> AutoResult<UINodeKind> {
        match name {
            // Layouts
            "col" | "column" => Ok(UINodeKind::Layout(LayoutKind::Col)),
            "row" => Ok(UINodeKind::Layout(LayoutKind::Row)),
            "center" => Ok(UINodeKind::Layout(LayoutKind::Center)),
            "container" => Ok(UINodeKind::Layout(LayoutKind::Container)),
            "scrollable" => Ok(UINodeKind::Layout(LayoutKind::Scrollable)),

            // Elements
            "text" => Ok(UINodeKind::Element(ElementKind::Text)),
            "button" => Ok(UINodeKind::Element(ElementKind::Button)),
            "input" => Ok(UINodeKind::Element(ElementKind::Input)),
            "checkbox" => Ok(UINodeKind::Element(ElementKind::Checkbox)),
            "radio" => Ok(UINodeKind::Element(ElementKind::Radio)),
            "select" => Ok(UINodeKind::Element(ElementKind::Select)),
            "list" => Ok(UINodeKind::Element(ElementKind::List)),
            "table" => Ok(UINodeKind::Element(ElementKind::Table)),

            // Custom widgets (capitalized names)
            name if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                => Ok(UINodeKind::Widget(name.to_string())),

            _ => Err(format!("Unknown UI node kind: {}", name).into()),
        }
    }
}
```

#### 2.3 UINode to View<M> Conversion

```rust
impl<M: Clone + Debug + 'static> UINode {
    /// Convert UINode to AutoUI View<M>
    pub fn to_view(&self) -> AutoResult<View<M>> {
        match &self.kind {
            UINodeKind::Element(ElementKind::Text) => {
                let content = self.get_prop_str("content")?;
                let view = View::text(content);
                Ok(self.apply_style(view))
            }

            UINodeKind::Element(ElementKind::Button) => {
                let label = self.get_prop_str("label")?;
                let onclick = self.extract_message("onclick")?;
                let view = View::button(label, onclick);
                Ok(self.apply_style(view))
            }

            UINodeKind::Layout(LayoutKind::Col) => {
                let spacing = self.get_prop_u16("spacing").unwrap_or(0);
                let padding = self.get_prop_u16("padding").unwrap_or(0);
                let mut builder = View::col().spacing(spacing).padding(padding);

                for child in &self.children {
                    builder = builder.child(child.to_view()?);
                }

                Ok(self.apply_style(builder.build()))
            }

            UINodeKind::Layout(LayoutKind::Row) => {
                let spacing = self.get_prop_u16("spacing").unwrap_or(0);
                let padding = self.get_prop_u16("padding").unwrap_or(0);
                let mut builder = View::row().spacing(spacing).padding(padding);

                for child in &self.children {
                    builder = builder.child(child.to_view()?);
                }

                Ok(self.apply_style(builder.build()))
            }

            // ... other element types
            _ => Err(format!("Unsupported node kind: {:?}", self.kind).into()),
        }
    }

    fn apply_style(&self, mut view: View<M>) -> View<M> {
        if let Some(style) = &self.style {
            // Apply style to view
            // This relies on Plan 005's style field integration
        }
        view
    }
}
```

#### 2.4 Public API for Runtime Evaluation

**File**: `d:\autostack\auto-ui\crates\auto-ui\src\lib.rs`

```rust
// Re-export node_converter
#[cfg(feature = "transpiler")]
pub use node_converter::{UINode, UINodeKind, LayoutKind, ElementKind};

/// Load and evaluate .at file at runtime
#[cfg(feature = "transpiler")]
pub fn load_from_auto(code: &str) -> AutoResult<Box<dyn Component>> {
    use auto_lang::eval_ui;

    // Parse and evaluate .at code
    let node = eval_ui(code)?;

    // Convert Node → View<M>
    let ui_node = UINode::from_value(&node)?;

    // Create dynamic component
    Ok(Box::new(DynamicComponent::from_node(ui_node)))
}
```

**Deliverables**:
- ✅ `crates/auto-ui/src/node_converter.rs` - Node → View<M> conversion
- ✅ Support for all 12 View variants
- ✅ Style extraction from Node props
- ✅ Message type inference and extraction
- ✅ Public API in `auto_ui::load_from_auto()`

**Acceptance Criteria**:
- ✅ Simple `text("Hello")` node converts correctly
- ✅ Nested layouts (col, row, center) work
- ✅ Button onclick messages extracted properly
- ✅ Style strings applied correctly
- ✅ All unit tests passing

---

**Status**: ⏳ **IN PROGRESS** - Framework complete, needs AST compatibility fixes

**Completion Date**: 2025-01-22 (partial)

### Phase 3: Auto to Rust Transpiler (Week 2-3)

**Goal**: Create transpiler that converts `.at` widget definitions to Rust `Component` implementations.

#### 3.1 Transpiler Architecture

**File**: `d:\autostack\auto-ui\crates\auto-ui-transpiler\src\lib.rs` (new crate)

**Reference**: Study existing Rust transpiler at `d:\autostack\auto-lang\crates\auto-lang\src\trans\rust.rs`

**Key Design Decisions**:
- Reuse AutoLang's existing AST and parser
- Create AST visitor pattern for UI-specific nodes
- Generate idiomatic Rust code using templates
- Handle message enum derivation automatically

#### 3.2 Crate Structure

```toml
# Cargo.toml
[package]
name = "auto-ui-transpiler"
version = "0.1.0"

[dependencies]
auto-lang = { path = "../../../auto-lang/crates/auto-lang" }
auto-ui = { path = "../auto-ui" }
auto-val = { path = "../../../auto-lang/crates/auto-val" }
anyhow = "1.0"
thiserror = "1.0"
quote = "1.0"
proc-macro2 = "1.0"
```

```
crates/auto-ui-transpiler/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API
    ├── ast_visitor.rs      # AST traversal
    ├── widget_extractor.rs # Extract widget definitions
    ├── codegen.rs          # Rust code generation
    └── templates.rs        # Code templates
```

#### 3.3 Widget Extraction

**File**: `src/widget_extractor.rs`

```rust
use auto_lang::ast::*;
use auto_lang::AutoResult;

pub struct WidgetDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub view_method: Option<ViewMethod>,
    pub on_method: Option<OnMethod>,
    pub style: Option<String>,
}

pub struct FieldDefinition {
    pub name: String,
    pub type_name: String,
    pub default_value: Option<String>,
}

pub struct ViewMethod {
    pub body: Code, // AST of view() method
}

pub struct OnMethod {
    pub message_variants: Vec<MessageVariant>,
    pub body: Code,
}

pub struct MessageVariant {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

/// Extract widget definitions from AST
pub fn extract_widgets(ast: &Code) -> AutoResult<Vec<WidgetDefinition>> {
    let mut widgets = Vec::new();

    for stmt in &ast.stmts {
        if let Stmt::TypeDecl(type_decl) = stmt {
            // Check if this type "is Widget"
            if is_widget_trait(type_decl) {
                let widget = parse_widget_definition(type_decl)?;
                widgets.push(widget);
            }
        }
    }

    Ok(widgets)
}

fn is_widget_trait(type_decl: &TypeDecl) -> bool {
    type_decl.traits.iter().any(|t| t.name == "Widget")
}
```

#### 3.4 Message Enum Derivation

**File**: `src/codegen.rs`

```rust
impl WidgetDefinition {
    /// Generate message enum from on() method analysis
    pub fn derive_message_enum(&self) -> String {
        if let Some(on_method) = &self.on_method {
            let variants = &on_method.message_variants;

            let mut enum_def = String::from("#[derive(Clone, Copy, Debug)]\n");
            enum_def.push_str(&format!("enum Msg {{\n"));

            for variant in variants {
                if variant.fields.is_empty() {
                    enum_def.push_str(&format!("    {},\n", variant.name));
                } else {
                    let fields = variant.fields.iter()
                        .map(|f| format!("{}: {}", f.name, f.type_name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    enum_def.push_str(&format!("    {}({}),\n", variant.name, fields));
                }
            }

            enum_def.push_str("}\n");
            enum_def
        } else {
            // Empty message enum
            "#[derive(Clone, Copy, Debug)]\nenum Msg {}\n".to_string()
        }
    }
}
```

#### 3.5 Code Generation Templates

**File**: `src/templates.rs`

```rust
use super::WidgetDefinition;

pub fn generate_widget_impl(widget: &WidgetDefinition) -> String {
    format!(
        r#"// Auto-generated from {name}.at
// DO NOT EDIT - changes will be overwritten

use auto_ui::{{Component, View}};

#[derive(Debug)]
pub struct {name} {{
{fields}
}}

impl {name} {{
    pub fn new({field_params}) -> Self {{
        Self {{
{field_initializers}
        }}
    }}
}}

impl Component for {name} {{
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {{
{on_method_body}
    }}

    fn view(&self) -> View<Self::Msg> {{
{view_method_body}
    }}
}}
"#,
        name = widget.name,
        fields = generate_field_decls(&widget.fields),
        field_params = generate_field_params(&widget.fields),
        field_initializers = generate_field_inits(&widget.fields),
        on_method_body = generate_on_method(&widget.on_method),
        view_method_body = generate_view_method(&widget.view_method),
    )
}
```

#### 3.6 AST Visitor for View Body

**File**: `src/ast_visitor.rs`

```rust
use auto_lang::ast::*;
use super::WidgetDefinition;

/// Visit AST node and generate Rust View code
pub struct ViewCodeGenerator<'a> {
    indent: usize,
    widget: &'a WidgetDefinition,
}

impl<'a> ViewCodeGenerator<'a> {
    pub fn new(widget: &'a WidgetDefinition) -> Self {
        Self { indent: 0, widget }
    }

    pub fn generate_view(&mut self, code: &Code) -> String {
        let mut rust_code = String::new();

        for stmt in &code.stmts {
            rust_code.push_str(&self.generate_stmt(stmt));
        }

        rust_code
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Expr(expr) => self.generate_expr(expr),
            Stmt::Store(store) => self.generate_store(store),
            Stmt::Fn(fn_decl) => {
                // Skip function declarations in view body
                String::new()
            }
            // ... other statement types
            _ => format!("/* TODO: {:?} */", stmt),
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> String {
        match expr {
            // Node expression: col { ... } → View::col()...
            Expr::Node(node) => self.generate_node(node),

            // String interpolation: "Count: {count}" → format!("Count: {}", self.count)
            Expr::Str(s) if s.contains("{") => {
                let interpolated = self.interpolate_string(s);
                format!("{}.to_string()", interpolated)
            }

            // Simple string
            Expr::Str(s) => format!("\"{}\"", s),

            // Field access: count → self.count
            Expr::Ident(name) => format!("self.{}", name),

            // Enum variant: Msg.Inc → Msg::Inc
            Expr::Bina(lhs, Op::Dot, rhs) => {
                if let Expr::Ident(lhs_name) = lhs.as_ref() {
                    if lhs_name == "Msg" {
                        format!("{}::{}", lhs_name, rhs)
                    } else {
                        format!("{}.{}", lhs_name, rhs)
                    }
                } else {
                    // TODO: handle complex cases
                    format!("/* TODO: complex dot access */")
                }
            }

            _ => format!("/* TODO: expr {:?} */", expr),
        }
    }

    fn generate_node(&mut self, node: &NodeExpr) -> String {
        match node.name.as_str() {
            "col" | "column" => self.generate_col(node),
            "row" => self.generate_row(node),
            "center" => self.generate_center(node),
            "text" => self.generate_text(node),
            "button" => self.generate_button(node),
            _ => self.generate_custom_widget(node),
        }
    }

    fn generate_col(&mut self, node: &NodeExpr) -> String {
        let spacing = get_prop_u16(node, "spacing").unwrap_or(0);
        let padding = get_prop_u16(node, "padding").unwrap_or(0);
        let style = get_prop_str(node, "style");

        let mut builder = format!("View::col()\n{}    .spacing({})\n{}    .padding({})",
            "    ", spacing, "    ", padding);

        if let Some(style_str) = style {
            builder = format!("{}\n{}    .style(\"{}\")", builder, "    ", style_str);
        }

        for child in &node.body.stmts {
            if let Stmt::Expr(expr) = child {
                let child_code = self.generate_expr(expr);
                builder = format!("{}\n{}    .child({})", builder, "    ", child_code);
            }
        }

        format!("{}\n{}.build()", builder, "    ")
    }

    fn generate_text(&mut self, node: &NodeExpr) -> String {
        if let Some(Expr::Str(content)) = get_prop_expr(node, "content") {
            if content.contains("{") {
                let interpolated = self.interpolate_string(content);
                format!("View::text({})", interpolated)
            } else {
                format!("View::text(\"{}\")", content)
            }
        } else {
            format!("View::text(\"\")")
        }
    }

    fn generate_button(&mut self, node: &NodeExpr) -> String {
        let label = get_prop_str(node, "label").unwrap_or("".to_string());
        let onclick = get_prop_expr(node, "onclick");

        let msg = if let Some(expr) = onclick {
            self.generate_expr(expr)
        } else {
            "Msg::default()".to_string()
        };

        format!("View::button(\"{}\", {})", label, msg)
    }

    fn interpolate_string(&mut self, s: &str) -> String {
        // "Count: {count}" → format!("Count: {}", self.count)
        let mut result = s.to_string();
        let mut format_args = Vec::new();

        // Find all {expr} placeholders
        while let Some(start) = result.find('{') {
            let end = result[start..].find('}').unwrap_or(result.len());
            let placeholder = &result[start + 1..start + end];

            // Replace {} with {}
            result.replace_range(start..start + end + 1, "{}");

            // Convert placeholder to Rust: count → self.count
            format_args.push(format!("self.{}", placeholder));
        }

        format!("format!(\"{}\", {})", result, format_args.join(", "))
    }

    // ... other node generators
}
```

#### 3.7 Main Transpiler API

**File**: `src/lib.rs`

```rust
use auto_lang::{parse, AutoResult};
use auto_lang::ast::Code;

mod widget_extractor;
mod ast_visitor;
mod codegen;
mod templates;

pub use widget_extractor::{WidgetDefinition, extract_widgets};
pub use codegen::{generate_widget_impl, generate_message_enum};

/// Transpile .at file to Rust code
pub fn transpile(at_code: &str) -> AutoResult<String> {
    // 1. Expand widget macros
    let expanded = auto_lang::macro::expand_widget_macros(at_code)?;

    // 2. Parse to AST
    let ast = parse(&expanded)?;

    // 3. Extract widget definitions
    let widgets = extract_widgets(&ast)?;

    // 4. Generate Rust code
    let mut rust_code = String::new();

    for widget in &widgets {
        rust_code.push_str(&widget.derive_message_enum());
        rust_code.push_str("\n");

        rust_code.push_str(&generate_widget_impl(widget));
        rust_code.push_str("\n");
    }

    Ok(rust_code)
}

/// Transpile .at file and write to .rs file
pub fn transpile_file(at_path: &str) -> AutoResult<()> {
    let at_code = std::fs::read_to_string(at_path)?;
    let rust_code = transpile(&at_code)?;

    let rs_path = at_path.replace(".at", ".rs");
    std::fs::write(&rs_path, rust_code)?;

    Ok(())
}
```

**Deliverables**:
- ✅ `crates/auto-ui-transpiler/` - New transpiler crate
- ✅ Widget extraction from AST
- ✅ Message enum derivation
- ✅ View body code generation
- ✅ Support for all layout types
- ✅ Public API: `transpile()`, `transpile_file()`

**Acceptance Criteria**:
- ✅ Simple `widget Hello` transpiles correctly
- ✅ `widget Counter` with messages transpiles
- ✅ Generated Rust code compiles without errors
- ✅ Generated code implements `Component` trait
- ✅ Style strings preserved in generated code

---

### Phase 4: Hot-Reload Support (Week 3)

**Goal**: Enable live UI updates during development without recompilation.

#### 4.1 File Watcher Infrastructure

**File**: `d:\autostack\auto-ui\crates\auto-ui\src\hot_reload.rs` (new)

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::path::Path;
use crate::Component;

pub struct UIWatcher {
    watcher: RecommendedWatcher,
    tx: mpsc::Sender<UIUpdate>,
}

pub enum UIUpdate {
    Reload(String),      // Path to .at file that changed
    Error(String),       // Error message
}

impl UIWatcher {
    pub fn new() -> AutoResult<Self> {
        let (tx, rx) = mpsc::channel();

        let watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    if let Ok(path) = event.path.strip_prefix("src/ui") {
                        if path.extension().and_then(|s| s.to_str()) == Some("at") {
                            tx.send(UIUpdate::Reload(path.to_string_lossy().to_string()))
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tx.send(UIUpdate::Error(e.to_string())).ok();
                }
            }
        })?;

        Ok(Self { watcher, tx })
    }

    pub fn watch(&mut self, path: &Path) -> AutoResult<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }
}
```

#### 4.2 Hot-Reloadable Component Wrapper

**File**: `crates/auto-ui/src/hot_reload.rs` (continued)

```rust
pub struct HotReloadComponent<M: Clone + Debug> {
    current: Box<dyn Component<Msg = M>>,
    source_path: String,
}

impl<M: Clone + Debug + 'static> HotReloadComponent<M> {
    pub fn load(path: &str) -> AutoResult<Self> {
        let code = std::fs::read_to_string(path)?;
        let current = auto_ui::load_from_auto(&code)?;

        Ok(Self {
            current,
            source_path: path.to_string(),
        })
    }

    pub fn reload(&mut self) -> AutoResult<()> {
        let code = std::fs::read_to_string(&self.source_path)?;
        self.current = auto_ui::load_from_auto(&code)?;
        Ok(())
    }
}

impl<M: Clone + Debug + 'static> Component for HotReloadComponent<M> {
    type Msg = M;

    fn on(&mut self, msg: Self::Msg) {
        self.current.on(msg);
    }

    fn view(&self) -> View<Self::Msg> {
        self.current.view()
    }
}
```

#### 4.3 Integration with Backend Run Loops

**Example for Iced**:

**File**: `examples/hot_reload_counter/src/main.rs`

```rust
use auto_ui::{Component, View, HotReloadComponent};
use auto_ui_iced::ComponentIced;
use std::time::Duration;

#[derive(Debug)]
enum HotReloadMessage {
    UIReloaded,
    UIMessage(<Counter as Component>::Msg),
}

struct HotReloadApp {
    counter: HotReloadComponent<<Counter as Component>::Msg>,
    _watcher: UIWatcher,
}

impl HotReloadApp {
    fn new() -> AutoResult<Self> {
        let counter = HotReloadComponent::load("src/ui/counter.at")?;
        let mut watcher = UIWatcher::new()?;
        watcher.watch(Path::new("src/ui"))?;

        Ok(Self { counter, _watcher: watcher })
    }
}

impl Component for HotReloadApp {
    type Msg = HotReloadMessage;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            HotReloadMessage::UIReloaded => {
                self.counter.reload().ok();
            }
            HotReloadMessage::UIMessage(msg) => {
                self.counter.on(msg);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        self.counter.view()
            .map(HotReloadMessage::UIMessage)
    }
}

fn main() -> AutoResult<()> {
    auto_ui_iced::run_app::<HotReloadApp>()
}
```

**Deliverables**:
- ✅ File watcher for `.at` files
- ✅ `HotReloadComponent` wrapper
- ✅ Reload logic with error handling
- ✅ Integration examples for Iced and GPUI

**Acceptance Criteria**:
- ✅ File changes trigger reload
- ✅ UI updates live in running window
- ✅ Errors in .at files don't crash app
- ✅ No memory leaks on reload

---

### Phase 5: Validation & Testing (Week 4)

**Goal**: Comprehensive testing of both runtime interpretation and transpilation modes.

#### 5.1 Unit Tests

**File**: `crates/auto-ui-transpiler/tests/widget_extraction.rs`

```rust
#[test]
fn test_extract_simple_widget() {
    let at_code = r#"
        widget Hello {
            msg str

            fn view() View {
                text(msg) {}
            }
        }
    "#;

    let widgets = extract_widgets(&parse(at_code).unwrap()).unwrap();
    assert_eq!(widgets.len(), 1);
    assert_eq!(widgets[0].name, "Hello");
    assert_eq!(widgets[0].fields.len(), 1);
    assert_eq!(widgets[0].fields[0].name, "msg");
}

#[test]
fn test_counter_widget_extraction() {
    let at_code = include_str!("testdata/counter.at");
    let widgets = extract_widgets(&parse(at_code).unwrap()).unwrap();

    let counter = &widgets[0];
    assert_eq!(counter.name, "Counter");
    assert!(counter.on_method.is_some());

    let msg_variants = counter.on_method.as_ref().unwrap().message_variants.clone();
    assert_eq!(msg_variants.len(), 2);
    assert_eq!(msg_variants[0].name, "Inc");
    assert_eq!(msg_variants[1].name, "Dec");
}
```

**File**: `crates/auto-ui-transpiler/tests/codegen.rs`

```rust
#[test]
fn test_generate_hello_widget() {
    let widget = WidgetDefinition {
        name: "Hello".to_string(),
        fields: vec![FieldDefinition {
            name: "msg".to_string(),
            type_name: "str".to_string(),
            default_value: None,
        }],
        view_method: Some(/* ... */),
        on_method: None,
        style: None,
    };

    let rust_code = generate_widget_impl(&widget);

    assert!(rust_code.contains("struct Hello"));
    assert!(rust_code.contains("impl Component for Hello"));
    assert!(rust_code.contains("type Msg = Msg"));
    assert!(rust_code.contains("fn view(&self) -> View<Self::Msg>"));
}

#[test]
fn test_generate_counter_with_messages() {
    let widget = WidgetDefinition {
        name: "Counter".to_string(),
        fields: vec![FieldDefinition {
            name: "count".to_string(),
            type_name: "i64".to_string(),
            default_value: Some("0".to_string()),
        }],
        view_method: Some(/* ... */),
        on_method: Some(/* ... */),
        style: None,
    };

    let msg_enum = widget.derive_message_enum();

    assert!(msg_enum.contains("enum Msg"));
    assert!(msg_enum.contains("Inc,"));
    assert!(msg_enum.contains("Dec,"));
}
```

#### 5.2 Integration Tests

**Test: Runtime Interpretation**

```bash
# File: examples/runtime_hello/src/main.rs
fn main() -> AutoResult<()> {
    let at_code = r#"
        widget Hello {
            msg str

            fn view() View {
                text(msg) {}
            }
        }
    "#;

    let component = auto_ui::load_from_auto(at_code)?;
    auto_ui_iced::run_component(component)
}

# Test: cargo run --example runtime_hello
# Expected: Opens window showing "Hello"
```

**Test: Transpilation**

```bash
# File: examples/transpiled_counter/src/ui/counter.at
widget Counter {
    count i64 = 0

    fn view() View {
        col(spacing: 16) {
            text("Count: {count}")
            row(spacing: 8) {
                button("+", Inc)
                button("-", Dec)
            }
        }
    }

    fn on(ev Msg) {
        is ev {
            Msg.Inc => { count += 1 }
            Msg.Dec => { count -= 1 }
        }
    }
}

# Build script: build.rs
fn main() {
    auto_ui_transpiler::transpile_file("src/ui/counter.at").unwrap();
}

# Test: cargo build --example transpiled_counter
# Expected: Compiles successfully, generates counter.rs
```

**Test: Hot-Reload**

```bash
# File: examples/hot_reload_demo/src/main.rs
# (as shown in Phase 4.3)

# Test:
# 1. cargo run --example hot_reload_demo
# 2. Edit src/ui/counter.at (change text, add button)
# 3. Save file
# 4. Expected: Window updates live without restarting
```

#### 5.3 Test Data Files

**Directory**: `crates/auto-ui-transpiler/testdata/`

```
testdata/
├── hello.at          # Simple text widget
├── counter.at        # Counter with messages
├── todo.at           # TodoMVC with list operations
├── login.at          # Form with input fields
├── styles.at         # Style testing (Plan 005 integration)
└── nested.at         # Complex nested layouts
```

**Expected Outputs**:
```
testdata/expected/
├── hello.rs
├── counter.rs
├── todo.rs
├── login.rs
├── styles.rs
└── nested.rs
```

#### 5.4 End-to-End Test Suite

**File**: `tests/e2e_tests.rs`

```rust
#[test]
fn test_hello_world_transpile_and_run() {
    // 1. Transpile
    auto_ui_transpiler::transpile_file("testdata/hello.at").unwrap();

    // 2. Load generated component
    let hello = hello::Hello::new("Hello, World!".to_string());

    // 3. Verify view structure
    let view = hello.view();
    assert!(matches!(view, View::Text { .. }));

    // 4. Run with backend (optional, may require display)
}

#[test]
fn test_counter_transpile() {
    let rust_code = auto_ui_transpiler::transpile(
        include_str!("testdata/counter.at")
    ).unwrap();

    // Verify generated code structure
    assert!(rust_code.contains("struct Counter"));
    assert!(rust_code.contains("impl Component for Counter"));
    assert!(rust_code.contains("enum Msg"));
    assert!(rust_code.contains("Inc,"));
    assert!(rust_code.contains("Dec,"));
}
```

**Deliverables**:
- ✅ 20+ unit tests for transpiler
- ✅ 10+ integration tests for runtime mode
- ✅ 5+ end-to-end tests
- ✅ Test data files with expected outputs
- ✅ CI/CD integration

**Acceptance Criteria**:
- ✅ All unit tests passing
- ✅ All integration tests passing
- ✅ Runtime mode works for 5 example widgets
- ✅ Transpilation works for 5 example widgets
- ✅ Hot-reload example runs successfully
- ✅ Code coverage > 80%

---

## File Structure Summary

### New Files

**AutoUI Stdlib** (`d:\autostack\auto-ui\stdlib\auto\ui.at`):
```
stdlib/auto/ui.at        # Complete UI module in single file (~350 lines)
```

**Note**: AutoLang doesn't support folder-based modules yet, so all UI types are in a single `ui.at` file. Future enhancement may split this into multiple files when AutoLang adds folder module support.

**Installation**: Symlink or copy to AutoLang's stdlib path:
```bash
# In auto-lang repository
cd stdlib/auto
ln -s ../../../auto-ui/stdlib/auto/ui.at ui.at
```

**AutoLang** (`d:\autostack\auto-lang`):
```
crates/auto-lang/
├── src/macro/
│   └── ui.rs                 # Widget macro expansion (200 lines)
├── stdlib/auto/
│   ├── widget.at            # Widget trait definition (50 lines)
│   └── app.at               # App type definition (30 lines)
└── src/lib.rs               # Add ui feature export
```

**AutoUI** (`d:\autostack\auto-ui`):
```
crates/auto-ui/
├── src/
│   ├── node_converter.rs    # Node → View<M> conversion (600 lines)
│   └── hot_reload.rs        # File watcher + reload (300 lines)
└── Cargo.toml               # Add notify dependency
```

**AutoUI Transpiler** (new crate):
```
crates/auto-ui-transpiler/
├── Cargo.toml
├── build.rs                 # Build script for examples
└── src/
    ├── lib.rs               # Public API (200 lines)
    ├── widget_extractor.rs  # AST widget extraction (400 lines)
    ├── ast_visitor.rs       # View code generation (600 lines)
    ├── codegen.rs           # Code templates (300 lines)
    └── templates.rs         # Rust code templates (200 lines)
├── tests/
│   ├── widget_extraction.rs
│   ├── codegen.rs
│   └── e2e_tests.rs
└── testdata/
    ├── *.at                 # Test input files
    └── expected/*.rs        # Expected outputs
```

**Examples**:
```
examples/
├── runtime_hello/           # Runtime interpretation demo
├── transpiled_counter/      # Transpilation demo
├── hot_reload_demo/         # Hot-reload demo
└── ui_comparison/           # Side-by-side Rust vs Auto comparison
```

### Modified Files

**AutoLang**:
- `crates/auto-lang/src/lib.rs` - Add `ui` feature
- `crates/auto-lang/Cargo.toml` - Add `regex` dependency

**AutoUI**:
- `crates/auto-ui/Cargo.toml` - Add `auto-lang`, `notify` deps
- `crates/auto-ui/src/lib.rs` - Re-export node_converter

---

## Dependencies

### New Dependencies

```toml
# auto-lang/Cargo.toml
[dependencies]
regex = "1.10"  # For macro expansion
```

```toml
# auto-ui/Cargo.toml
[dependencies]
auto-lang = { path = "../../../auto-lang/crates/auto-lang", optional = true }
notify = "6.0"  # For file watching

[features]
transpiler = ["auto-lang"]
```

```toml
# auto-ui-transpiler/Cargo.toml
[dependencies]
auto-lang = { path = "../../../auto-lang/crates/auto-lang" }
auto-ui = { path = "../auto-ui" }
auto-val = { path = "../../../auto-lang/crates/auto-val" }
anyhow = "1.0"
thiserror = "1.0"
```

---

## Timeline and Milestones

| Week | Phase | Tasks | Deliverables |
|------|-------|-------|--------------|
| 0 | Phase 0 | auto.ui module definition | ✅ `scratch/hello.at` parses successfully |
| 1 | Phase 1 | AutoLang widget macros | ✅ Widget expansion working |
| 1-2 | Phase 2 | Node → View conversion | ✅ Runtime interpretation working |
| 2-3 | Phase 3 | Rust transpiler | ✅ Code generation working |
| 3 | Phase 4 | Hot-reload support | ✅ Live UI updates |
| 4 | Phase 5 | Testing & validation | ✅ All tests passing |

**Total**: 4-5 weeks (including Phase 0)

**Key Milestones**:
- **Week 0 End** (Phase 0): `scratch/hello.at` parses without errors
- **Week 1 End**: Simple `hello.at` works in runtime mode
- **Week 2 End**: `counter.at` transpiles to working Rust code
- **Week 3 End**: Hot-reload demo working
- **Week 4 End**: Production-ready, all tests passing

---

## Risk Assessment and Mitigation

### Risk 1: Macro Expansion Fragility

**Probability**: Medium
**Impact**: High

**Description**: Text-level preprocessing may break on edge cases (comments, strings, nested braces).

**Mitigation**:
- Start with simple regex-based approach
- Comprehensive test coverage for edge cases
- Fallback to AST-level macros if needed
- Document known limitations

### Risk 2: Message Type Inference

**Probability**: High
**Impact**: High

**Description**: Automatic message enum derivation may fail for complex patterns.

**Mitigation**:
- Require explicit `enum Msg` in .at files initially
- Implement pattern matching in `fn on(ev Msg)` body
- Fall back to manual message definition
- Clear error messages for inference failures

### Risk 3: Runtime Performance

**Probability**: Medium
**Impact**: Medium

**Description**: Runtime interpretation may be slower than compiled Rust.

**Mitigation**:
- Document performance trade-offs
- Recommend transpilation for production
- Optimize critical paths in node_converter
- Benchmark against pure Rust implementations

### Risk 4: Build Complexity

**Probability**: Low
**Impact**: Medium

**Description**: build.rs integration and transpilation may complicate builds.

**Mitigation**:
- Make transpilation opt-in (feature flag)
- Clear documentation for setup
- Provide cargo aliases for common workflows
- Fast incremental builds

### Risk 5: Hot-Reload Memory Leaks

**Probability**: Medium
**Impact**: High

**Description**: Repeated reloading may leak memory.

**Mitigation**:
- Use Rust's ownership system correctly
- Explicit cleanup in reload logic
- Memory profiling during development
- Leak detection in tests

---

## Success Criteria

### Functional Requirements

1. ✅ **Runtime Interpretation**: `.at` files can be loaded and rendered without compilation
2. ✅ **Transpilation**: `.at` files generate idiomatic Rust `Component` implementations
3. ✅ **Hot-Reload**: File changes trigger live UI updates
4. ✅ **Style Support**: Plan 005 style system works in `.at` files
5. ✅ **All View Variants**: All 12 View types supported in Auto syntax

### Quality Requirements

1. ✅ **Code Coverage**: > 80% test coverage
2. ✅ **Performance**: Runtime interpretation < 2x slower than Rust
3. ✅ **Documentation**: Complete API docs and user guide
4. ✅ **Examples**: 5+ working examples
5. ✅ **Error Messages**: Clear, actionable errors for common issues

### Developer Experience

1. ✅ **Setup Time**: < 15 minutes from zero to first .at file
2. ✅ **Learning Curve**: Familiar syntax for web developers
3. ✅ **Debugging**: Clear error messages pointing to .at file locations
4. ✅ **IDE Support**: Syntax highlighting (via VSCode extension)
5. ✅ **Iteration Speed**: < 1 second from save to UI update (hot-reload)

---

## Open Questions

1. **Message Type Derivation**: Should we require explicit `enum Msg` or attempt automatic derivation?
   - **Recommendation**: Start explicit, add automatic derivation later

2. **Component State**: How should state mutations be handled in `.at` files?
   - **Recommendation**: Use Rust-style mutable field access: `count += 1`

3. **Event Handler Syntax**: `fn on(ev Msg)` vs `fn handle(event: Event)`?
   - **Recommendation**: `fn on(ev Msg)` matches Component trait

4. **Build Integration**: Should transpilation be automatic or manual?
   - **Recommendation**: Manual with cargo alias: `cargo transpile`

5. **Style Integration**: How to reference Plan 005 style classes?
   - **Recommendation**: `style: "p-4 bg-blue-500"` field in widget

---

## Next Steps

### Immediate (This Week)

1. **Review and Approve Plan** ✅
   - Stakeholder feedback
   - Risk assessment
   - Resource allocation

2. **Setup Development Environment**
   - Create feature branch: `feature/auto-lang-integration`
   - Setup CI/CD pipeline
   - Create project tracking board

3. **Start Phase 0** - auto.ui Module Definition (CRITICAL PREREQUISITE)
   - Create `stdlib/auto/ui/` directory structure
   - Define `Widget` trait with `view()` and `on()` methods
   - Define `App` type with `title` and `theme` fields
   - Define `View` type marker
   - Create layout components (Center, Col, Row, etc.)
   - Create element components (Text, Button, Input, etc.)
   - Set up module export in `__init__.at`
   - Install/symlink to AutoLang stdlib
   - **Test**: Verify `scratch/hello.at` parses successfully

4. **Start Phase 1.1** - Widget Macro System
   - Implement `crates/auto-lang/src/macro/ui.rs`
   - Add unit tests
   - Document macro expansion rules

### Short-Term (Week 1-2)

1. **Complete Phase 1** - AutoLang Extensions
2. **Complete Phase 2** - Node → View Conversion
3. **First Working Demo**: `hello.at` in runtime mode

### Medium-Term (Week 3-4)

1. **Complete Phase 3** - Transpiler Implementation
2. **Complete Phase 4** - Hot-Reload Support
3. **Complete Phase 5** - Testing and Validation

### Long-Term (Post-MVP)

1. **IDE Support**: VSCode extension for .at files
2. **Auto-Completion**: IntelliSense for widget props
3. **Type Checking**: Enhanced type errors
4. **Optimization**: Performance improvements
5. **Advanced Features**: Animations, transitions, effects

---

## Related Plans

- **Plan 004**: Unified Styling System - Provides Tailwind CSS classes
- **Plan 005**: Style System Integration - Adds `style` field to View API
- **Plan 007**: Component Library (Future) - Pre-built widgets in Auto
- **Plan 008**: Documentation Site (Future) - Interactive examples

---

## References

### External Resources

- [Auto Language Parser](d:\autostack\auto-lang\crates\auto-lang\src)
- [Rust Transpiler](d:\autostack\auto-lang\crates\auto-lang\src\trans\rust.rs)
- [C Transpiler](d:\autack\auto-lang\crates\auto-lang\src\trans\c.rs)
- [Plan 045 - AutoUI Integration](d:\autostack\auto-lang\docs\plans\045-auto-ui-integration.md)

### Internal Examples

- `examples/unified-todo` - Cross-platform TodoMVC
- `crates/auto-ui/examples/counter_component.rs` - Rust Counter
- `scratch/hello.at` - Sample .at UI file

---

**Plan Created**: 2025-01-21
**Author**: Claude Code
**Status**: 📋 **Planning**
**Next Review**: After Phase 1 completion
