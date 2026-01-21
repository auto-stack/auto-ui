# Unified Counter Example

This example demonstrates how to write a single `Component` that works with multiple backends through feature flags.

## Running

### With Iced backend
```bash
cargo run --package unified-counter --features iced
```

### With GPUI backend
```bash
cargo run --package unified-counter --features gpui
```

## Architecture

The `Counter` component is written once using the `auto_ui` abstractions:

- `Component` trait for state management
- `View` builder for UI declaration
- Message enum for events

The `main()` function uses conditional compilation to select the backend:

```rust
#[cfg(feature = "iced")]
return auto_ui_iced::run_app::<Counter>();

#[cfg(feature = "gpui")]
return auto_ui_gpui::run_app::<Counter>();
```

## Key Points

1. **Single Component Code** - Write your UI logic once
2. **Backend Selection** - Choose backend via feature flags
3. **Type Safety** - Compile-time checks ensure backend is enabled
4. **Zero Runtime Cost** - Conditional compilation has no runtime overhead

## Note on GPUI

GPUI backend requires manual `Render` trait implementation due to its different architecture (closures vs message enums). See `auto-ui-gpui-examples/src/bin/counter.rs` for the full GPUI pattern.
