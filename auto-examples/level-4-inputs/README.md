# Level 4: Input Form

Demonstrates user input handling with text fields.

## New Concepts

1. **Input Widget**: `input(value) { ... }` - Text input field
2. **Placeholder**: `placeholder: "text"` - Hint text
3. **Input Events**: `oninput: "event"` - Triggered on text change
4. **String Concatenation**: `text("Hello " + self.username)` - String operations
5. **Automatic State Updates**: Input values update the field automatically
6. **Empty String Default**: `str = ""` - Default value for strings

## Important Note

In Auto, when you use `input(self.field)`, the input field **automatically updates** `self.field` when the user types. The `oninput` event is just for notification.

## Run

```bash
cargo run --package auto-ui --example gen
cargo run --package auto-ui --example input_form
```

## UI Structure

```
col (main form)
├── text (title)
├── col (username field)
│   ├── text (label)
│   └── input
├── col (email field)
│   ├── text (label)
│   └── input
├── col (message field)
│   ├── text (label)
│   └── input
└── col (summary)
    ├── text (heading)
    ├── text (username display)
    ├── text (email display)
    └── text (message display)
```
