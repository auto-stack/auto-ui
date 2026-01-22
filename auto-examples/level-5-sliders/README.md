# Level 5: Slider and Progress Bar

Demonstrates numeric input with sliders and progress display.

## New Concepts

1. **Float Type**: `float` - Floating-point numbers
2. **Slider Widget**: `slider(min, max, value) { ... }` - Numeric input control
3. **Range Definition**: `slider(0.0, 100.0, value)` - Min, max, current value
4. **Change Events**: `onchange: "event"` - Triggered on value change
5. **Progress Bar**: `progress_bar(min, max, value)` - Visual progress indicator
6. **Arithmetic**: `self.progress * 100.0` - Mathematical operations

## How Sliders Work

In Auto, when you use `slider(min, max, self.field)`, the slider **automatically updates** `self.field` when the user moves it. The `onchange` event is optional and used for notifications.

## Slider Use Cases

- **Volume Control**: `slider(0.0, 1.0, volume)` - 0% to 100%
- **Temperature**: `slider(-20.0, 50.0, temp)` - Celsius range
- **Percentage**: `slider(0.0, 100.0, percent)` - 0 to 100
- **Brightness**: `slider(0.0, 1.0, brightness)` - 0 to 1

## Run

```bash
cargo run --package auto-ui --example gen
cargo run --package auto-ui --example slider_demo
```

## UI Structure

```
col (main)
├── text (title)
├── col (value section)
│   ├── text (label with value)
│   └── slider (0-100 range)
├── col (volume section)
│   ├── text (label with value)
│   └── slider (0-1 range)
└── col (progress section)
    ├── text (percentage)
    └── progress_bar (visual indicator)
```
