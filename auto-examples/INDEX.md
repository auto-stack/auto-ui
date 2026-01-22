# Auto-UI Examples Index

A comprehensive index of all example code in the Auto-UI project, organized by purpose and complexity.

## 📁 Auto Language Examples (Progressive Learning)

**Location**: [auto-examples/](auto-examples/)

Progressive learning path from beginner to advanced. Each level builds upon the previous one.

| Level | Example | File | Concepts Introduced |
|-------|---------|------|---------------------|
| 1 | Hello World | [level-1-hello/hello.at](auto-examples/level-1-hello/hello.at) | Basic structure, text display |
| 2 | Counter | [level-2-counter/counter.at](auto-examples/level-2-counter/counter.at) | State, events, buttons |
| 3 | Layout | [level-3-layout/layout.at](auto-examples/level-3-layout/layout.at) | col/row layouts, nesting |
| 4 | Input Form | [level-4-inputs/input_form.at](auto-examples/level-4-inputs/input_form.at) | Text input, forms |
| 5 | Sliders | [level-5-sliders/slider_demo.at](auto-examples/level-5-sliders/slider_demo.at) | Numeric input, progress |
| 6 | Todo List | [level-6-todos/todo_list.at](auto-examples/level-6-todos/todo_list.at) | Full CRUD app |

## 🎨 Unified Examples (Backend-Agnostic)

**Location**: [examples/unified-*](examples/)

Same code works with both Iced and GPUI backends via feature flags.

| Example | Location | Description |
|---------|----------|-------------|
| Counter | [unified-counter](examples/unified-counter/) | Simple counter with increment/decrement |
| Input | [unified-input](examples/unified-input/) | Text input form with validation |
| Layout | [unified-layout](examples/unified-layout/) | Column, row, nested layouts |
| Slider | [unified-slider](examples/unified-slider/) | Slider widgets with value binding |
| Progress | [unified-progress](examples/unified-progress/) | Progress bar with slider control |
| Todos | [unified-todos](examples/unified-todos/) | CRUD operations with state management |

## 🔧 Platform-Specific Examples

### Iced Examples
**Location**: [crates/iced-examples/src/bin/](crates/iced-examples/src/bin/)

Direct implementations using the Iced framework.

| Example | File | Description |
|---------|------|-------------|
| Hello | [hello.rs](crates/iced-examples/src/bin/hello.rs) | Basic hello world |
| Counter | [counter.rs](crates/iced-examples/src/bin/counter.rs) | Counter with state |
| Button | [button.rs](crates/iced-examples/src/bin/button.rs) | Button interactions |
| Checkbox | [checkbox.rs](crates/iced-examples/src/bin/checkbox.rs) | Checkbox widget |
| Slider | [slider.rs](crates/iced-examples/src/bin/slider.rs) | Slider widgets |
| Layout | [layout.rs](crates/iced-examples/src/bin/layout.rs) | Layout demonstrations |
| Progress | [progress.rs](crates/iced-examples/src/bin/progress.rs) | Progress bar |
| Todos | [todos.rs](crates/iced-examples/src/bin/todos.rs) | Todo list application |
| Dropdown | [dropdown.rs](crates/iced-examples/src/bin/dropdown.rs) | Dropdown selection |
| Circle | [circle.rs](crates/iced-examples/src/bin/circle.rs) | Custom drawing |

### GPUI Examples
**Location**: [crates/gpui-examples/src/bin/](crates/gpui-examples/src/bin/)

Reference implementations using the GPUI framework.

| Example | File | Description |
|---------|------|-------------|
| Hello | [hello.rs](crates/gpui-examples/src/bin/hello.rs) | Basic hello world |
| Counter | [counter.rs](crates/gpui-examples/src/bin/counter.rs) | Counter with state |
| Button | [button.rs](crates/gpui-examples/src/bin/button.rs) | Button interactions |

## 🗂️ Experimental Scratch Examples

**Location**: [scratch/](scratch/)

Experimental and test examples for Auto language development.

| Example | File | Description |
|---------|------|-------------|
| Simple Hello | [hello_basic.at](scratch/hello_basic.at) | Minimal hello world |
| Simple Counter | [simple_counter.at](scratch/simple_counter.at) | Minimal counter |
| Working Counter | [working_counter.at](scratch/working_counter.at) | Full counter implementation |
| Layout Test | [col_test.at](scratch/col_test.at) | Column layout test |
| UI Components | [ui_components.at](scratch/ui_components.at) | Various UI widgets |
| Layout Showcase | [layout_showcase.at](scratch/layout_showcase.at) | Layout demonstrations |

## 📚 Component Library Examples

### Auto-UI Components
**Location**: [crates/auto-ui/examples/](crates/auto-ui/examples/)

Examples demonstrating the abstract component system.

| Example | File | Description |
|---------|------|-------------|
| Style Demo | [style_demo.rs](crates/auto-ui/examples/style_demo.rs) | Styling system |
| Styled Counter | [styled_counter.rs](crates/auto-ui/examples/styled_counter.rs) | Styled components |
| All Components | [all_components.rs](crates/auto-ui/examples/all_components.rs) | Component showcase |

### GPUI-Component Examples
**Location**: [crates/gpui-components-examples/](crates/gpui-components-examples/)

Official examples from the gpui-component library.

## 🚀 How to Use Examples

### Auto Language Examples

1. Navigate to the example directory
2. Open the `.at` file to view the Auto source
3. Generate Rust code:
   ```bash
   cargo run --package auto-ui --example gen
   ```
4. Run the generated example:
   ```bash
   cargo run --package auto-ui --example <name>
   ```

### Unified Examples

Run with backend selection:

```bash
# With Iced backend (default)
cargo run --package unified-counter --features iced

# With GPUI backend
cargo run --package unified-counter --features gpui
```

### Platform-Specific Examples

```bash
# Iced examples
cargo run --package iced-examples --bin <name>

# GPUI examples
cargo run --package gpui-examples --bin <name>
```

## 📖 Learning Path

### For Beginners
1. Start with [Level 1: Hello World](auto-examples/level-1-hello/)
2. Progress through [Level 2: Counter](auto-examples/level-2-counter/)
3. Learn layouts with [Level 3: Layout](auto-examples/level-3-layout/)

### For Intermediate Users
4. Handle input with [Level 4: Input Form](auto-examples/level-4-inputs/)
5. Work with numbers using [Level 5: Sliders](auto-examples/level-5-sliders/)

### For Advanced Users
6. Build complete apps with [Level 6: Todo List](auto-examples/level-6-todos/)
7. Study [unified-todos](examples/unified-todos/) for backend abstraction
8. Examine [platform-specific examples](crates/iced-examples/) for framework details

## 🎯 Example Categories

### By Feature

**Widgets**:
- Text display
- Buttons
- Input fields
- Checkboxes
- Sliders
- Progress bars
- Dropdowns

**Layouts**:
- Column (vertical)
- Row (horizontal)
- Nested layouts
- Containers
- Scrollable areas

**State Management**:
- Simple state
- Form state
- List state
- Complex application state

**Patterns**:
- CRUD operations
- Event handling
- Form validation
- Dynamic UI generation

### By Complexity

**Beginner** (1-5 concepts):
- Hello world
- Simple counter
- Basic layouts

**Intermediate** (6-10 concepts):
- Forms
- Sliders
- Event routing
- List operations

**Advanced** (11+ concepts):
- Full applications
- Complex state
- Multiple components
- Backend abstraction

## 🔍 Finding Examples

### By Purpose
- **Learning**: Start with [auto-examples/](auto-examples/)
- **Reference**: Check [unified-*](examples/) for backend-agnostic code
- **Implementation**: See [platform-specific examples](crates/) for framework details

### By Component
- **Buttons**: [unified-counter](examples/unified-counter/), [counter.at](auto-examples/level-2-counter/counter.at)
- **Inputs**: [unified-input](examples/unified-input/), [input_form.at](auto-examples/level-4-inputs/input_form.at)
- **Layouts**: [unified-layout](examples/unified-layout/), [layout.at](auto-examples/level-3-layout/layout.at)
- **Sliders**: [unified-slider](examples/unified-slider/), [slider_demo.at](auto-examples/level-5-sliders/slider_demo.at)

## 📝 Contributing

When adding new examples:

1. **Choose appropriate level**: Match complexity to existing examples
2. **Follow conventions**: Use established patterns and naming
3. **Document thoroughly**: Include README with concepts covered
4. **Test thoroughly**: Ensure code compiles and runs correctly
5. **Update index**: Add entry to this file

## 📚 Additional Resources

- [Auto Language Reference](../docs/auto-language-reference.md)
- [Component Guide](../docs/component-guide.md)
- [Layout Guide](../docs/layout-guide.md)
- [State Management](../docs/state-management.md)
- [Styling System](../docs/styling-system.md)
