# AutoUI Examples Guide

This guide shows you how to run AutoUI examples using both **runtime interpretation** and **transpilation** modes.

## Quick Start

### Option 1: Runtime Interpretation (Easiest)

Run the counter example directly in Rust - no .at files needed:

```bash
cargo run --example runtime_counter
```

This demonstrates the **runtime mode** where you build UI programmatically using the View API.

### Option 2: Transpilation (Auto Language Support)

Transpile an `.at` file to Rust code:

```bash
# Requires transpiler feature
cargo run --example transpile_counter --features transpiler
```

This will:
1. Read `scratch/counter.at`
2. Generate Rust code
3. Save it to `counter_generated.rs`
4. Display the generated code

## Available Examples

### Runtime Examples (No Transpilation)

| Example | Description | Command |
|---------|-------------|---------|
| `runtime_counter` | Simple counter UI | `cargo run --example runtime_counter` |
| `counter_component` | Counter with View builder | `cargo run --example counter_component` |
| `all_components` | Demonstrates all 12 UI components | `cargo run --example all_components` |
| `styled_counter` | Counter with styling | `cargo run --example styled_counter` |
| `style_demo` | Style system basics (L1) | `cargo run --example style_demo` |
| `style_demo_l2` | Style system important features (L2) | `cargo run --example style_demo_l2` |
| `style_demo_l3` | Style system advanced features (L3) | `cargo run --example style_demo_l3` |

### Transpilation Examples

| Example | Description | Command |
|---------|-------------|---------|
| `transpile_counter` | Transpile counter.at to Rust | `cargo run --example transpile_counter --features transpiler` |

## Auto Language Files

The following `.at` files are available in `scratch/`:

### counter.at
A simple counter widget with increment/decrement buttons:

```auto
widget Counter {
    count int

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

app CounterExample {
    center {
        Counter(count: 0)
    }
}
```

### hello.at
Simple "Hello World" widget:

```auto
widget Hello {
    msg str = "Hello, Auto!"

    fn view() View {
        text(self.msg) {}
    }
}
```

### login.at
Login form with input fields:

```auto
widget LoginForm {
    username str
    password str

    fn view() View {
        col {
            text("Login") {}
            input(placeholder: "Username") {}
            input(placeholder: "Password", password: true) {}
            button("Login") {}
        }
    }
}
```

## How to Use AutoUI

### Mode 1: Pure Rust (Runtime)

Write your UI directly in Rust using the View builder:

```rust
use auto_ui::{Component, View};

struct MyWidget {
    count: i32,
}

impl Component for MyWidget {
    type Msg = Msg;

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .child(View::button("Click me").on_click(Msg::Click).build())
            .child(View::text(&format!("Count: {}", self.count)).build())
            .build()
    }
}
```

**Pros**:
- Full type safety
- IDE support
- No build step
- Best for production apps

**Cons**:
- More verbose
- No hot-reload

### Mode 2: Auto Language (Transpilation)

Write UI in `.at` files, then transpile to Rust:

1. Create `my_widget.at`:
```auto
widget MyWidget {
    count int = 0

    fn view() View {
        col {
            button("Click me") { onclick: Msg.Click }
            text(count)
        }
    }
}
```

2. Transpile:
```bash
cargo run --example transpile_my_widget --features transpiler
```

3. Copy generated code into your project

**Pros**:
- Less verbose
- Declarative syntax
- Auto-generates Component impl
- Good for rapid development

**Cons**:
- Requires build step
- Less mature (development phase)

### Mode 3: Auto Language (Hot-Reload - Future)

Write UI in `.at` files with live reloading:

```rust
use auto_ui::hot_reload::{HotReloadComponent, UIWatcher};

let mut counter = HotReloadComponent::load("ui/counter.at")?;
let view = counter.view()?;

// Watch for changes
let mut watcher = UIWatcher::new()?;
watcher.watch("ui/")?;

// File changes automatically reload the component
```

**Pros**:
- Fastest iteration
- Live preview
- Best for development

**Cons**:
- Not fully implemented yet
- Requires file watcher

## Testing the Transpiler

Run the test suite to see transpilation in action:

```bash
# Run all tests
cargo test --package auto-ui --features transpiler

# Run transpiler-specific tests
cargo test --package auto-ui --features transpiler --test transpiler_test

# Run integration tests
cargo test --package auto-ui --features transpiler --test integration_test
```

## Current Status

✅ **Implemented**:
- Phase 0: auto.ui module (all 12 UI components)
- Phase 1: Widget macro system
- Phase 2: Node → View conversion (runtime mode)
- Phase 3: Rust transpiler (transpile .at → .rs)
- Phase 4: Hot-reload infrastructure
- Phase 5: Testing and validation (99% pass rate)

⚠️ **Limitations**:
- Style parsing not complete (1 failing test)
- Hot-reload uses placeholder parser
- Integration with backends (GPUI/Iced) needs examples

🔜 **Next Steps**:
- Create complete GPUI backend example
- Create complete Iced backend example
- Fix style parsing
- Implement full AutoLang parser in hot-reload

## Getting Help

- **Plan 006**: [Auto Language Integration](../docs/plans/006-auto-language-integration.md)
- **Test Report**: [Phase 5 Testing Report](../docs/reports/phase5-testing-report.md)
- **Examples**: See `examples/` directory

## Contributing

To add a new example:

1. Create `examples/your_example.rs`
2. Implement Component trait
3. Add it to this README with description
4. Test it: `cargo run --example your_example`
