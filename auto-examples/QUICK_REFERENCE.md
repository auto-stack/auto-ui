# Auto Language Quick Reference

This is a quick reference guide for the Auto language syntax. For complete examples, see the [auto-examples](../auto-examples/) directory.

## Basic Structure

```auto
type ComponentName is Widget {
    field_name Type = default_value

    fn view() {
        // UI description
    }

    fn on(event Type) {
        // Event handling
    }
}

fn main() {
    app("Title") {
        component() {}
    }
}
```

## Types

### Primitive Types
- `str` - String text
- `int` - Integer number
- `float` - Floating-point number
- `bool` - Boolean (true/false)

### Complex Types
- `list<T>` - List of type T
- `CustomType` - User-defined structure

## Widgets

### Display Widgets
```auto
text("Hello")              // Simple text
text(variable)             // Text from variable
text("Value: " + value)   // Concatenation
```

### Button
```auto
button("Label") {
    onclick: "event_name"
}
```

### Input
```auto
input(value) {
    placeholder: "Hint text"
    oninput: "event_name"
}
```

### Checkbox
```auto
checkbox(is_checked) {
    oncheck: "event_name"
}
```

### Slider
```auto
slider(min, max, value) {
    onchange: "event_name"
}
```

### Progress Bar
```auto
progress_bar(min, max, value)
```

### Layout Widgets
```auto
col {
    // Vertical stacking
    child1
    child2
}

row {
    // Horizontal arrangement
    child1
    child2
}

container {
    // Wrapper with styling
    child
}
```

## Event Handling

### Simple Events
```auto
fn on(ev str) {
    is ev {
        "event_name" => {
            // Handle event
        }
    }
}
```

### Pattern Matching
```auto
fn on(ev str) {
    is ev {
        "add" => {
            // Add item
        }
        ev if starts_with(ev, "delete_") => {
            // Extract ID from event
            let id = int(substring(ev, 7))
        }
        _ => {
            // Default case
        }
    }
}
```

## State Management

### Fields
```auto
type Counter is Widget {
    count int = 0
}
```

### Updating State
```auto
fn on(ev str) {
    is ev {
        "increment" => {
            self.count += 1
        }
    }
}
```

### Lists
```auto
// Add to list
self.todos += Todo { ... }

// Iterate over list
for todo in self.todos {
    text(todo.description)
}

// Filter list
let filtered = []
for item in self.list {
    is item.condition {
        filtered += item
    }
}
```

## String Operations

```auto
len(str)           // String length
str1 + str2        // Concatenation
substring(s, 7)    // Get substring from index 7
starts_with(s, "prefix")  // Check prefix
```

## Arithmetic

```auto
// Addition
count + 1

// Subtraction
count - 1

// Multiplication
value * 100.0

// Division
total / count
```

## Common Patterns

### Conditional Display
```auto
// Using ternary (not directly supported, use event-driven approach)
col {
    button("Show") {
        onclick: "show"
    }
    // Conditionally show content via state
}
```

### Dynamic Lists
```auto
col {
    for todo in self.todos {
        row {
            checkbox(todo.completed) {
                oncheck: "toggle_" + todo.id
            }
            text(todo.description)
        }
    }
}
```

### Form Handling
```auto
type Form is Widget {
    input_value str = ""

    fn view() {
        input(self.input_value) {
            placeholder: "Type here..."
            oninput: "input_changed"
        }
    }

    fn on(ev str) {
        // Input value is automatically updated
        is ev {
            "input_changed" => {
                // self.input_value already has new value
            }
        }
    }
}
```

## Tips and Best Practices

1. **Event Naming**: Use descriptive event names like `add_todo`, `delete_item`
2. **State Updates**: Always update state in the `on()` function
3. **View Purity**: The `view()` function should only describe UI, not modify state
4. **Type Inference**: Auto can infer types in many cases
5. **Incremental Development**: Start simple, add complexity gradually

## Converting to Rust

```bash
# Generate Rust code
cargo run --package auto-ui --example gen

# Run the application
cargo run --package auto-ui --example <example_name>
```

## Examples by Complexity

1. **Beginner**: [hello.at](auto-examples/level-1-hello/hello.at) - Display text
2. **Basic**: [counter.at](auto-examples/level-2-counter/counter.at) - Add interactivity
3. **Intermediate**: [layout.at](auto-examples/level-3-layout/layout.at) - Layout systems
4. **Advanced**: [input_form.at](auto-examples/level-4-inputs/input_form.at) - User input
5. **Expert**: [todo_list.at](auto-examples/level-6-todos/todo_list.at) - Full CRUD

## See Also

- [Auto Examples](../scratch/) - Experimental examples
- [Component Guide](../../docs/component-guide.md) - Component architecture
- [Layout Guide](../../docs/layout-guide.md) - Layout system details
