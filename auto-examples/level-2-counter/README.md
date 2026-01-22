# Level 2: Counter

Building on Level 1, this example adds:
- Integer state
- Event handling with onclick
- Multiple widgets (buttons, text)
- Layout with col
- State updates in on()

## New Concepts

1. **Integer State**: `count int = 0` - Number field
2. **Layout**: `col { }` - Vertical container
3. **Buttons**: `button("Label") { onclick: "event" }`
4. **Self Reference**: `text(self.count)` - Access component state
5. **Event Handling**: `fn on(ev str)` - Process user interactions
6. **Pattern Matching**: `is ev { "inc" => { ... } }` - Conditional logic

## Run

```bash
# Generate and run
cargo run --package auto-ui --example gen
cargo run --package auto-ui --example counter
```
