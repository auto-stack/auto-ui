# Level 1: Hello World

The simplest Auto program. This example demonstrates:
- Basic type declaration
- String fields
- The view() function
- Text display
- Main app entry point

## Run

```bash
# Generate Rust code
cargo run --package auto-ui --example gen

# Run the generated code
cargo run --package auto-ui --example hello
```

## Concepts Learned

1. **Type Declaration**: `type Name is Widget` - Defines a UI component
2. **Fields**: `msg str = "value"` - Component state
3. **View Function**: `fn view()` - Describes the UI layout
4. **Text Widget**: `text(msg) {}` - Displays text content
5. **Main Function**: `fn main()` with `app()` - Entry point
