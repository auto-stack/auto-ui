# AutoUI Crates

This document describes the workspace structure and purpose of each crate.

## Workspace Structure

```
crates/
├── auto-ui/                         # Core abstraction layer
├── auto-ui-iced/                    # Iced backend adapter
├── auto-ui-iced-examples/           # Examples using abstraction layer + Iced
├── iced-examples/                   # Pure Iced framework examples
└── gpui-examples/                   # Pure GPUI framework examples
```

## Crates

### Core Layer

#### `auto-ui/`
**Purpose**: Backend-agnostic UI framework core

**Provides**:
- `Component` trait - defines component interface with `on()` and `view()`
- `View<M>` enum - declarative UI description
- `ViewBuilder<M>` - fluent API for building layouts

**Example**:
```rust
use auto_ui::{Component, View};

struct Counter { count: i64 }

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) { /* handle messages */ }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(10)
            .child(View::text(self.count.to_string()))
            .child(View::button("+", Message::Increment))
            .build()
    }
}
```

### Backend Adapters

#### `auto-ui-iced/`
**Purpose**: Adapter for rendering `View<M>` with Iced framework

**Provides**:
- `IntoIcedElement<M>` trait - converts `View<M>` to `iced::Element`
- `ComponentIced` trait - adds `view_iced()` method to all `Component` types

**Example**:
```rust
use auto_ui_iced::ComponentIced;

fn main() -> iced::Result {
    iced::run(Counter::update, view)
}

fn view(counter: &Counter) -> iced::Element<'_, Message> {
    counter.view_iced()
}
```

### Examples

#### `auto-ui-iced-examples/`
**Purpose**: Demonstrates using auto-ui abstraction layer with Iced backend

**Examples**:
- `counter` - Simple counter with increment/decrement
- `todo` - TodoMVC with filtering and state management
- `temp_converter` - Temperature conversion with computed values

**Run**:
```bash
cargo run --package auto-ui-iced-examples --bin counter
cargo run --package auto-ui-iced-examples --bin todo
cargo run --package auto-ui-iced-examples --bin temp_converter
```

#### `iced-examples/`
**Purpose**: Pure Iced framework examples for learning and reference

**Examples**:
- `hello` - Basic "Hello, World!"
- `counter` - Counter using Iced directly
- `button` - Button click handling
- `checkbox` - Checkbox state
- `circle` - Custom drawing
- `dropdown` - Dropdown selection

**Run**:
```bash
cargo run --package iced-examples --bin hello
cargo run --package iced-examples --bin counter
# ... etc
```

#### `gpui-examples/`
**Purpose**: Pure GPUI framework examples for learning and reference

**Examples**:
- `counter` - Counter using GPUI directly
- `layout` - Layout demonstrations
- `button` - Button components

**Run**:
```bash
cargo run --package gpui-examples --bin counter
# ... etc
```

## Future Structure

When GPUI backend is implemented (Phase 6), the structure will expand to:

```
crates/
├── auto-ui/                         # Core abstraction layer
├── auto-ui-iced/                    # Iced backend adapter
├── auto-ui-gpui/                    # GPUI backend adapter (Phase 6)
├── auto-ui-iced-examples/           # Abstraction layer + Iced examples
├── auto-ui-gpui-examples/           # Abstraction layer + GPUI examples (Phase 6)
├── iced-examples/                   # Pure Iced framework examples
└── gpui-examples/                   # Pure GPUI framework examples
```

## Design Principles

1. **Separation of Concerns**: Core abstraction is independent of any UI framework
2. **Clear Naming**: Crate names explicitly indicate their purpose and backend
3. **Parallel Examples**: Each backend has both pure framework examples and abstraction layer examples
4. **Easy Comparison**: Side-by-side comparison of pure framework vs abstraction layer usage

## Dependencies

```
auto-ui-iced-examples/
  ├── depends on: auto-ui (core abstraction)
  ├── depends on: auto-ui-iced (Iced adapter)
  └── depends on: iced (Iced framework)

iced-examples/
  └── depends on: iced (Iced framework only)

gpui-examples/
  └── depends on: gpui (GPUI framework only)
```
