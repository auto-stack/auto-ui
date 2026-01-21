# AutoUI Standard Library Module

This directory contains the `auto.ui` module for the Auto language, providing UI types and components for AutoUI integration.

## Files

- **ui.at** - Complete UI module definition (~200 lines)
  - Core types: Widget, App, View
  - Layout components: Center, Col, Row, Container, Scrollable
  - Element components: Text, Button, Input, Checkbox, Radio, Select, List, Table

## Installation

This module needs to be installed in AutoLang's stdlib path. There are three installation options:

### Option A: Symlink (Recommended for Development)

```bash
# In auto-lang repository
cd stdlib/auto
ln -s ../../../auto-ui/stdlib/auto/ui.at ui.at
```

### Option B: Copy

```bash
# In auto-lang repository
cd stdlib/auto
cp ../../../auto-ui/stdlib/auto/ui.at .
```

### Option C: Build Script (For Automation)

Add to `auto-lang/Cargo.toml`:
```toml
[features]
default = []
ui = []

[build-dependencies]
auto-ui = { path = "../../../auto-ui/crates/auto-ui", optional = true }
```

Add to `auto-lang/build.rs`:
```rust
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

## Module Structure

### Core Types

1. **Widget** - Base trait for all UI components
   - `style: str` - Style string for Plan 005 integration
   - `view() -> View` - Returns UI structure
   - `on(ev: Msg)` - Event handler

2. **App** - Application entry point
   - `title: str` - Window title
   - `theme: str` - Theme name
   - Inherits from Widget

3. **View** - Type marker for UI element trees
   - Maps to `auto_ui::View<Msg>` in Rust

### Layout Components

- **Center** - Centers content horizontally and vertically
- **Col** - Vertical layout with spacing/padding
- **Row** - Horizontal layout with spacing/padding
- **Container** - Generic container with sizing
- **Scrollable** - Container with scrollbars

### Element Components

- **Text** - Display text
- **Button** - Clickable with message
- **Input** - Text input field
- **Checkbox** - Checkbox with toggle
- **Radio** - Radio button
- **Select** - Dropdown selector
- **List** - List of widgets
- **Table** - Table with headers and rows

## Usage in Auto Files

```auto
use auto.ui: View, Widget, App, Center, Text, Button

widget Hello {
    msg str

    fn view() View {
        center {
            Text(msg)
        }
    }
}

app MyApp {
    title: "Hello World"
    center {
        Hello("Hello from Auto!")
    }
}
```

## Transpilation

The `auto.ui` module definitions are **stubs** for type checking. The actual implementation is in Rust via AutoUI.

The transpiler maps:
- `text(...)` → `auto_ui::View::text(...)`
- `button(...)` → `auto_ui::View::button(...)`
- `col(spacing: X) { ... }` → `auto_ui::View::col().spacing(X)`

## Note on Module Structure

AutoLang doesn't support folder-based modules yet, so all UI types are defined in a single `ui.at` file. Future versions may split this into multiple files when AutoLang adds folder module support.

## Related Plans

- **Plan 004** - Unified Styling System
- **Plan 005** - Style System Integration
- **Plan 006** - Auto Language Integration
