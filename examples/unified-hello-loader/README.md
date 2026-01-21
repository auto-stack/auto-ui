# Unified Hello Loader Example

This example demonstrates how to load and display an Auto language (`.at`) file with a full UI interface.

## What It Does

1. **Loads `scratch/hello.at`** - Reads the Auto language widget definition
2. **Displays the content** - Shows the .at file content with syntax highlighting
3. **Explains the process** - Shows how .at files are transpiled to Rust and rendered
4. **Cross-backend support** - Works with both Iced and GPUI backends

## Running the Example

### With Iced Backend (Default)

```bash
cargo run --package unified-hello-loader --features iced
```

### With GPUI Backend

```bash
cargo run --package unified-hello-loader --features gpui
```

## Features Demonstrated

### Auto Language Integration

The example shows the complete workflow from Auto language to UI:

```
scratch/hello.at → Transpiler → Rust Code → Component → UI
```

**The `hello.at` file**:
```auto
use auto.ui: View, widget, app, center, text

widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }

    style: "p-1"
}

app CounterExample {
    center {
        Hello("Hello, World!")
    }
}
```

### Transpiler Status

**Current Implementation** (Plan 006 Phase 5):
- ✅ Transpiler implemented and tested
- ✅ Integration tests passing (99% pass rate)
- ✅ Can transpile `.at` → Rust code
- ⚠️ Not yet integrated into build workflow
- 🔜 Hot-reload infrastructure ready

### Testing the Transpiler

To test the actual transpilation:

```bash
# Run transpiler tests
cargo test --package auto-ui --features transpiler --test transpiler_test

# Run integration tests
cargo test --package auto-ui --features transpiler --test integration_test
```

## UI Components

This example demonstrates:

1. **Styled text** - Using the unified styling system
2. **Code display** - Syntax-highlighted .at file content
3. **Interactive buttons** - Load and show transpiled code actions
4. **Status indicators** - Visual feedback for loading state
5. **Information panels** - Explaining the transpilation process

## Architecture

The example uses the standard AutoUI pattern:

```rust
#[derive(Debug, Default)]
struct HelloLoaderApp {
    message: String,
    loaded: bool,
}

impl Component for HelloLoaderApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        // Handle button clicks
    }

    fn view(&self) -> View<Self::Msg> {
        // Build UI with View builder API
    }
}
```

## Styling

Uses the unified styling system (Plan 004):

- **Layout**: `spacing()`, `padding()`, `center_x()`
- **Colors**: Background colors, text colors
- **Typography**: Font sizes, weights, styles
- **Borders**: Rounded corners, borders
- **Spacing**: Gaps between elements

## Related Examples

- [`counter_component`](../../crates/auto-ui/examples/counter_component.rs) - Basic Component implementation
- [`all_components`](../../crates/auto-ui/examples/all_components.rs) - All 12 UI components
- [`unified-container`](../unified-container) - Container layouts with unified backend
- [`styling_showcase`](../../crates/auto-ui/examples/styling_showcase.rs) - Complete styling system demo

## Next Steps

After exploring this example:

1. **Modify `scratch/hello.at`** - Change the message or styling
2. **Run transpiler tests** - See how .at files become Rust code
3. **Build your own widget** - Create a new .at file
4. **Explore hot-reload** - Check out the hot_reload module

## Documentation

- **Plan 006**: [Auto Language Integration](../../docs/plans/006-auto-language-integration.md)
- **Phase 5 Report**: [Testing & Validation](../../docs/reports/phase5-testing-report.md)
- **Styling System**: [Plan 004 - Unified Styling](../../docs/plans/004-unified-styling-system.md)

## Status

✅ **Working** - Example compiles and runs with both backends
✅ **Transpiler** - Implemented and tested (99% pass rate)
⚠️ **Integration** - Transpiler not yet integrated into build workflow
🔜 **Hot-reload** - Infrastructure ready, needs full parser integration
