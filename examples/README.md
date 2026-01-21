# AutoUI Examples Guide

This guide shows you how to run AutoUI examples and understand the different modes of operation.

## Quick Start

### ✅ Run Working Examples Now

These examples work right now - no special features needed:

```bash
# 1. Simple counter (Best starting point!)
cargo run --example counter_component

# 2. All 12 UI components demonstration
cargo run --example all_components

# 3. Styled counter with unified styling
cargo run --example styled_counter

# 4. Style system demos
cargo run --example style_demo          # L1: Core features
cargo run --example style_demo_l2       # L2: Important features
cargo run --example style_demo_l3       # L3: Advanced features
cargo run --example styling_showcase    # Complete styling demo
```

### 🔬 Test the Transpiler

The transpiler is tested and now available as a CLI tool!

```bash
# Use the transpiler CLI tool
cargo run --package auto-ui-transpiler -- --help
cargo run --package auto-ui-transpiler -- transpile input.at --stdout
cargo run --package auto-ui-transpiler -- info input.at

# Run transpiler tests
cargo test --package auto-ui --features transpiler --test transpiler_test

# Run integration tests (includes transpilation)
cargo test --package auto-ui --features transpiler --test integration_test
```

See [crates/auto-ui-transpiler-cli/README.md](../crates/auto-ui-transpiler-cli/README.md) for detailed CLI usage.

## Available Examples

### Core Examples

| Example | Description | Features Demonstrated |
|---------|-------------|----------------------|
| `counter_component` | Simple counter UI | Component trait, messages, View API, styling |
| `all_components` | All 12 UI components | col, row, center, container, scrollable, text, button, input, checkbox, radio, select, list, table |
| `styled_counter` | Counter with advanced styling | Unified styling system, L1/L2 features |

### Unified Examples (Cross-Backend)

| Example | Description | Features |
|---------|-------------|----------|
| `unified-hello-loader` | Loads scratch/hello.at with UI | Auto language file loading, transpiler integration |
| `unified-container` | Container layout examples | Padding, sizing, centering, nesting |
| `unified-counter` | Counter component | State management, messages |
| `unified-input` | Text input handling | User input, text display |
| `unified-list` | List components | Dynamic lists, scrolling |
| `unified-todo` | Todo application | Complete app example |
| `unified-table` | Table components | Data tables, multiple columns |
| `unified-*` | Other component examples | Various UI patterns |

### Style System Examples

| Example | Description | Plan Reference |
|---------|-------------|----------------|
| `style_demo` | Core styling features (L1) | Plan 004 |
| `style_demo_l2` | Important features (L2) | Plan 004 |
| `style_demo_l3` | Advanced features (L3) | Plan 004 + Plan 005 |
| `styling_showcase` | Complete styling system | Plan 004 + Plan 005 |

## How AutoUI Works

### Mode 1: Pure Rust (Runtime) ✅ Working Now!

Write your UI directly in Rust using the View builder API. This is what all current examples demonstrate.

```rust
use auto_ui::{Component, View};

struct Counter {
    count: i32,
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
            .child(View::button("+").on_click(Msg::Inc).build())
            .child(View::text(&format!("Count: {}", self.count)).build())
            .child(View::button("-").on_click(Msg::Dec).build())
            .build()
    }
}
```

**Run it**: `cargo run --example counter_component`

**Pros**:
- ✅ Full type safety
- ✅ IDE support (autocomplete, refactoring)
- ✅ No build step
- ✅ Production ready
- ✅ Works with GPUI and Iced backends

**Cons**:
- More verbose than Auto language
- No hot-reload

### Mode 2: Auto Language (Transpilation) 🔧 In Development

Write UI in `.at` files, then transpile to Rust code.

**Status**: Transpiler implemented and tested, but not yet integrated into examples workflow.

**Auto Language File** (`scratch/counter.at`):
```auto
widget Counter {
    count int = 0

    fn view() View {
        col {
            button "+" { onclick: Msg.Inc }
            text(count)
            button "-" { onclick: Msg.Dec }
        }
    }

    fn on(ev Msg) {
        is ev {
            Msg.Inc => .count += 1
            Msg.Dec => .count -= 1
        }
    }
}
```

**Transpile** (currently via tests):
```bash
cargo test --package auto-ui --features transpiler --test integration_test
```

**Generated Rust Code** (what transpiler creates):
```rust
#[derive(Debug)]
pub struct Counter {
    pub count: i32,
}

impl Component for Counter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Inc => self.count += 1,
            Dec => self.count -= 1,
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .child(View::button("+").on_click("Inc").build())
            .child(View::text(&format!("{}", self.count)).build())
            .child(View::button("-").on_click("Dec").build())
            .build()
    }
}
```

**Pros**:
- ✅ Less verbose (5-10x less code)
- ✅ Declarative syntax
- ✅ Auto-generates Component impl
- ✅ Good for rapid development

**Cons**:
- ⚠️ Requires build step
- ⚠️ Less mature (development phase)
- ⚠️ Not integrated into examples yet

### Mode 3: Hot-Reload (Future) 🔮 Planned

Live reloading of `.at` files during development.

**Status**: Infrastructure complete, needs full parser integration.

```rust
use auto_ui::hot_reload::{HotReloadComponent, UIWatcher};

// Load component from .at file
let mut counter = HotReloadComponent::load("ui/counter.at")?;
let view = counter.view()?;

// Watch for file changes
let mut watcher = UIWatcher::new()?;
watcher.watch("ui/")?;

// File changes automatically trigger reload
```

**Pros**:
- 🔜 Fastest iteration
- 🔜 Live preview
- 🔜 Best for development

**Cons**:
- Not fully implemented yet
- Requires file watcher

## Auto Language Files Available

The following `.at` files are available in `scratch/` for reference:

### counter.at
A counter widget with increment/decrement buttons.

### hello.at
Simple "Hello World" widget.

### login.at
Login form with input fields.

**Note**: These are reference implementations showing the Auto language syntax. They are not yet integrated into the build system.

## Running the Examples

### Prerequisites

```bash
# From the auto-ui directory
cd d:/autostack/auto-ui

# Ensure workspace is built
cargo build
```

### Example 1: Counter Component

```bash
cargo run --example counter_component
```

**What you'll see**:
- A counter widget with +/- buttons
- Styled using the unified styling system
- Demonstrates message handling
- Shows View builder API

### Example 2: All Components

```bash
cargo run --example all_components
```

**What you'll see**:
- All 12 UI components demonstrated
- Layout examples (col, row, center)
- Container examples (container, scrollable)
- Element examples (text, button, input, checkbox, radio, select, list, table)

### Example 3: Hello Auto Language Loader

```bash
cargo run --package unified-hello-loader --features iced
```

**What you'll see**:
- A UI that loads and displays [scratch/hello.at](scratch/hello.at)
- Code preview showing the Auto language syntax
- Information about how transpilation works
- Cross-backend support (Iced and GPUI)

**This demonstrates**:
- Loading Auto language (.at) files
- Transpiler integration (Plan 006)
- Unified styling system
- Cross-backend compatibility

### Example 4: Styling System

```bash
cargo run --example styling_showcase
```

**What you'll see**:
- Complete styling system demo
- L1 (Core): Basic styling
- L2 (Important): Common patterns
- L3 (Advanced): Complex layouts

## Understanding the Examples

### Component Structure

All examples follow this pattern:

```rust
// 1. Define your struct
struct MyWidget {
    field: Type,
}

// 2. Define messages
#[derive(Clone, Copy)]
enum Msg {
    Message1,
    Message2,
}

// 3. Implement Component trait
impl Component for MyWidget {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        // Handle messages
    }

    fn view(&self) -> View<Self::Msg> {
        // Build UI
    }
}
```

### View Builder API

The View builder uses a fluent API:

```rust
View::col()                    // Create column
    .spacing(16)                // Set spacing
    .padding(8)                 // Add padding
    .child(widget1)              // Add child
    .child(widget2)              // Add child
    .build()                     // Finish building
```

### Styling

Two ways to apply styles:

**Method 1: Style strings** (Plan 004)
```rust
View::text("Hello").style("text-xl font-bold")
```

**Method 2: Styled helpers**
```rust
View::text_styled("Hello", "text-xl font-bold")
```

## Current Status

✅ **Fully Working**:
- Runtime mode (all examples)
- View builder API
- Styling system (Plan 004 + Plan 005)
- 12 UI components
- Component trait
- Message handling

⚠️ **In Development**:
- Transpiler (implemented, tested)
- Hot-reload (infrastructure ready)
- Auto language integration

🔜 **Next Steps**:
- Create transpiler CLI tool
- Build GPUI backend example
- Build Iced backend example
- Implement full AutoLang parser in hot-reload

## Testing

```bash
# Run all tests
cargo test --package auto-ui

# Run tests with transpiler feature
cargo test --package auto-ui --features transpiler

# Test specific component
cargo test --package auto-ui test_node_converter
```

## Getting Help

- **Plan 006**: [Auto Language Integration](../docs/plans/006-auto-language-integration.md)
- **Test Report**: [Phase 5 Testing Report](../docs/reports/phase5-testing-report.md)
- **API Docs**: Check inline documentation in `src/lib.rs`

## Contributing

To add a new example:

1. Create `examples/your_example.rs`
2. Implement Component trait
3. Add entry to this README
4. Test it: `cargo run --example your_example`
5. Ensure it compiles without special features

**Important**: Examples should work with `cargo run --example <name>` without requiring additional features or setup.
