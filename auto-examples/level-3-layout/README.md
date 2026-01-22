# Level 3: Layout Showcase

Demonstrates various layout patterns and nesting.

## New Concepts

1. **Column Layout**: `col { }` - Vertical stacking of children
2. **Row Layout**: `row { }` - Horizontal arrangement of children
3. **Nesting**: Combining layouts inside each other
4. **Multiple Children**: Widgets can have multiple child elements
5. **Layout Composition**: Building complex UIs from simple layouts

## Layout Hierarchy

```
col (main)
├── text (title)
├── col (column demo)
│   ├── text
│   ├── text
│   └── text
├── row (row demo)
│   ├── text
│   ├── text
│   └── text
└── col (nested demo)
    └── row
        ├── col
        └── col
```

## Run

```bash
cargo run --package auto-ui --example gen
cargo run --package auto-ui --example layout
```
