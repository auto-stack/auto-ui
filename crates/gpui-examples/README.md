# GPUI Examples

This crate contains examples using the **gpui-component** UI framework.

## Running Examples

### Hello World
```bash
cargo run --bin hello
```

### Counter
```bash
cargo run --bin counter
```

### Button
```bash
cargo run --bin button
```

## About GPUI-Component

[GPUI-Component](https://github.com/longbridgeapp/gpui-component) is a Rust UI library built on top of GPUI (the rendering engine behind [Zed Editor](https://zed.dev/)).

Features:
- GPU-accelerated rendering
- Modern, declarative API
- Built-in styling system with theme support
- High performance for complex UIs
- Cross-platform support

## Architecture

GPUI-Component uses a simple state management system:

```rust
pub struct Counter {
    count: i64,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Build UI using fluent API
        div()
            .v_flex()
            .gap_3()
            .items_center()
            .child(/* ... */)
    }
}
```

## Event Handling

Events are handled using the `cx.listener()` pattern:

```rust
Button::new("inc")
    .primary()
    .label("+")
    .on_click(cx.listener(|view, _, _, _cx| {
        view.count += 1;  // Directly mutate state
    }))
```

## App Structure

```rust
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // Must initialize gpui-component first!
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions { /* ... */ },
                |window, cx| {
                    let view = cx.new(|_| Example);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
```

## Key Points

1. **gpui dependency**: Must be explicitly declared in Cargo.toml when using `gpui::*`
2. **Initialization**: Always call `gpui_component::init(cx)` before using any components
3. **Root component**: Window content must be wrapped in a `Root` component
4. **State mutation**: Use `cx.listener()` for event handlers that need to mutate state

## Dependencies

```toml
[dependencies]
gpui = "0.2.2"
gpui-component = "0.5.0"
anyhow = "1.0"
```

## Common Components

- **Button** - Interactive button with variants (primary, secondary, etc.)
- **Label** - Text display
- **Div** - Layout container
- **Input** - Text input field
- **Checkbox** - Checkbox input
- **And more...** - See [gpui-component docs](https://longbridge.github.io/gpui-component/)

## Layout

```rust
div()
    .v_flex()              // Vertical flex layout
    .h_flex()              // Horizontal flex layout
    .gap_3()               // Spacing between children
    .items_center()        // Center items horizontally
    .justify_center()      // Center items vertically
    .size_full()           // Fill available space
    .padding_4()           // Add padding
    .child(/* widget */)   // Add child widget
```

## References

- [GPUI-Component GitHub](https://github.com/longbridgeapp/gpui-component)
- [GPUI-Component Documentation](https://longbridge.github.io/gpui-component/)
- [GPUI (Zed Engine)](https://github.com/zed-industries/zed)
