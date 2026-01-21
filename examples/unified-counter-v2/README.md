# Unified Counter v2 - True Backend Abstraction

This example demonstrates **TRUE unification** between Iced and GPUI backends using automatic message conversion.

## 🎯 Key Innovation

**Automatic conversion from enum-based messages to GPUI closures** - You write your Component once with enum messages, and the system automatically handles the backend differences!

## 🚀 Running

### With Iced backend
```bash
cargo run --package unified-counter-v2 --features iced
```

### With GPUI backend (NOW WORKS!)
```bash
cargo run --package unified-counter-v2 --features gpui
```

## ✨ How It Works

### Your Code (Same for Both Backends)

```rust
#[derive(Clone, Copy, Debug)]
enum Message {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .child(View::button("Increment", Message::Increment))
            .child(View::text(format!("Count: {}", self.count)))
            .child(View::button("Decrement", Message::Decrement))
            .build()
    }
}
```

### Behind the Scenes

#### Iced Backend
```
View<M> → iced::Element
```
Messages are passed directly as enum values.

#### GPUI Backend (NEW!)
```
View<M> → auto_render → GPUI Elements with closures
```

The `auto_render` module automatically:
1. Wraps your Component in `GpuiComponentState`
2. Converts `View<M>` tree to GPUI elements
3. Generates closures that call `state.handle(msg)`
4. Triggers re-render via `cx.notify()`

## 🎨 Benefits

1. **Single Codebase** - Write once, run on both backends
2. **Enum Messages** - Use familiar enum-based patterns
3. **Type Safe** - Compile-time message checking
4. **Zero Boilerplate** - No manual GPUI Render implementation needed

## 📊 Comparison

| Feature | v1 (unified-counter) | v2 (unified-counter-v2) |
|----------|----------------------|---------------------------|
| Iced Support | ✅ Works | ✅ Works |
| GPUI Support | ❌ Error | ✅ Works! |
| Auto-Conversion | ❌ | ✅ Yes |
| Enum Messages | ✅ | ✅ |

## 🔧 Technical Details

The magic happens in `auto-ui-gpui/src/auto_render.rs`:

```rust
pub trait ViewExt<M: Clone + Debug + 'static> {
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M>;
}
```

This trait automatically:
- Creates message handlers that call `state.handle(msg)`
- Triggers re-renders with `cx.notify()`
- Converts the entire View tree recursively

## 🎉 Result

**True backend abstraction achieved!** You can now write auto-ui Components with enum messages, and they work on both Iced and GPUI without any manual GPUI-specific code!
