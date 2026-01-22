# Auto-UI Examples - Progressive Learning Path

This directory contains Auto language examples organized by complexity level, from beginner to advanced. Each level builds upon the previous ones, introducing new concepts incrementally.

## Learning Path

### Level 1: Hello World 🌟
**File**: [hello.at](level-1-hello/hello.at)

The simplest Auto program - just displaying text.

**Concepts**:
- Type declarations
- String fields
- The view() function
- Text display
- Main app entry point

**Perfect for**: Absolute beginners to Auto language

---

### Level 2: Counter 📊
**File**: [counter.at](level-2-counter/counter.at)

Adds state management and event handling.

**New Concepts**:
- Integer state (`int`)
- Event handling with `onclick`
- Multiple widgets (buttons, text)
- Layout with `col`
- State updates in `on()`

**Perfect for**: Learning interactivity and state

---

### Level 3: Layout Showcase 🎨
**File**: [layout.at](level-3-layout/layout.at)

Demonstrates various layout patterns and nesting.

**New Concepts**:
- Column layout (`col`)
- Row layout (`row`)
- Layout nesting
- Multiple children
- Layout composition

**Perfect for**: Understanding layout system

---

### Level 4: Input Form 📝
**File**: [input_form.at](level-4-inputs/input_form.at)

Demonstrates user input with text fields.

**New Concepts**:
- Input widget
- Placeholder text
- Input events (`oninput`)
- String concatenation
- Automatic state updates

**Perfect for**: Learning data entry and forms

---

### Level 5: Slider & Progress 📊
**File**: [slider_demo.at](level-5-sliders/slider_demo.at)

Numeric input with sliders and progress display.

**New Concepts**:
- Float type (`float`)
- Slider widget with ranges
- Change events (`onchange`)
- Progress bar widget
- Arithmetic operations

**Perfect for**: Working with numeric data

---

### Level 6: Todo List ✅
**File**: [todo_list.at](level-6-todos/todo_list.at)

A complete CRUD application with advanced state management.

**New Concepts**:
- Custom types (`type Todo`)
- Generic lists (`list<Todo>`)
- For loops and iteration
- String operations (`len`, `starts_with`, `substring`)
- Conditional logic in event handlers
- List manipulation (add, remove)
- Dynamic event naming with IDs

**Perfect for**: Building real applications

---

## How to Use These Examples

### 1. Generate Rust Code

First, generate Rust code from the Auto script:

```bash
# From the auto-ui directory
cargo run --package auto-ui --example gen
```

This compiles all `.at` files in the scratch/ directory.

### 2. Run the Examples

```bash
# Run the generated Rust code
cargo run --package auto-ui --example hello
cargo run --package auto-ui --example counter
cargo run --package auto-ui --example layout
cargo run --package auto-ui --example input_form
cargo run --package auto-ui --example slider_demo
cargo run --package auto-ui --example todo_list
```

### 3. Modify and Experiment

Each `.at` file is standalone. Feel free to:
- Modify values and see the result
- Add new widgets
- Combine concepts from different levels
- Create your own variations

## Progressive Learning Approach

We recommend going through the levels in order:

1. **Level 1**: Understand the basic structure
2. **Level 2**: Add interactivity
3. **Level 3**: Master layouts
4. **Level 4**: Handle user input
5. **Level 5**: Work with numbers
6. **Level 6**: Build complete applications

Each level introduces 3-6 new concepts, making learning manageable.

## File Organization

```
auto-examples/
├── README.md (this file)
├── level-1-hello/
│   ├── hello.at
│   └── README.md
├── level-2-counter/
│   ├── counter.at
│   └── README.md
├── level-3-layout/
│   ├── layout.at
│   └── README.md
├── level-4-inputs/
│   ├── input_form.at
│   └── README.md
├── level-5-sliders/
│   ├── slider_demo.at
│   └── README.md
└── level-6-todos/
    ├── todo_list.at
    └── README.md
```

## Next Steps

After completing all levels, you'll be ready to:
- Build complex UI applications in Auto
- Understand the Auto-to-Rust transpilation process
- Create your own widgets and components
- Contribute to the Auto-UI framework

## Additional Resources

- [Auto Language Reference](../../docs/auto-language-reference.md)
- [Component Guide](../../docs/component-guide.md)
- [Layout System](../../docs/layout-guide.md)
- [State Management](../../docs/state-management.md)

## Tips for Learning

1. **Run each example** before modifying it
2. **Read the error messages** - they're helpful
3. **Experiment** - change values, add widgets
4. **Compare levels** - see how complexity increases
5. **Build incrementally** - start simple, add features

Happy coding! 🚀
