# Iced Examples

This crate contains examples using the **iced** UI framework.

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

## About Iced

[Iced](https://github.com/iced-rs/iced) is a cross-platform GUI library for Rust, inspired by Elm.
It features:
- A reactive, Elm-like architecture
- Simple, type-safe API
- Modern, GPU-accelerated rendering
- Cross-platform support (Windows, macOS, Linux)

## Architecture

These examples follow the Elm architecture:
- **Model**: Application state
- **Update**: State transitions based on messages
- **View**: UI representation

```rust
impl Sandbox for App {
    type Message = Message;

    fn new() -> Self { /* initial state */ }
    fn title(&self) -> String { /* window title */ }
    fn update(&mut self, message: Message) { /* handle message */ }
    fn view(&self) -> Element<Message> { /* render UI */ }
}
```
