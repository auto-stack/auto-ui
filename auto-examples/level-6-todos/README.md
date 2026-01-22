# Level 6: Todo List

A complete CRUD application demonstrating advanced state management.

## New Concepts

1. **Custom Types**: `type Todo { ... }` - Define custom data structures
2. **List Type**: `list<Todo>` - Generic list of Todo items
3. **Default Values**: `todos list<Todo> = []` - Empty list default
4. **For Loops**: `for todo in self.todos { ... }` - Iteration
5. **String Operations**: `len()`, `starts_with()`, `substring()`
6. **Conditional Updates**: `is todo.id == id { ... }`
7. **List Manipulation**: Adding and removing items from lists
8. **Dynamic Events**: Event names with IDs (`"toggle_" + todo.id`)

## CRUD Operations

- **Create**: Add new todo when button clicked
- **Read**: Display all todos in a list
- **Update**: Toggle completion status with checkbox
- **Delete**: Remove todo with delete button

## Event Naming Pattern

This example uses a pattern for handling dynamic events:
- `"add_todo"` - Simple event for adding todos
- `"toggle_" + id` - Event with embedded data (e.g., "toggle_1", "toggle_2")
- `"delete_" + id` - Event with embedded data

## State Flow

```
User Action → Event → on() → State Update → view() Re-render
    ↓              ↓           ↓              ↓
Click Add → "add_todo" → todos += Todo → Display new todo
Toggle → "toggle_3" → Find todo #3 → Toggle → Re-render
Delete → "delete_1" → Remove todo #1 → Update list → Re-render
```

## Advanced Features

### List Filtering (Future Enhancement)
```auto
// Filter completed todos
let active_todos = []
for todo in self.todos {
    is !todo.completed {
        active_todos += todo
    }
}
```

### List Operations
```auto
// Map - transform list
let descriptions = []
for todo in self.todos {
    descriptions += todo.description
}

// Filter - conditional inclusion
let completed = []
for todo in self.todos {
    is todo.completed {
        completed += todo
    }
}
```

## Run

```bash
cargo run --package auto-ui --example gen
cargo run --package auto-ui --example todo_list
```

## Complete UI Structure

```
col (main container)
├── text (title)
├── col (input section)
│   ├── input (text field)
│   └── button (add button)
├── col (todo list)
│   └── for each todo:
│       └── row
│           ├── checkbox (completion toggle)
│           ├── text (description)
│           └── button (delete)
└── col (footer)
    ├── text (total count)
    └── text (instructions)
```

## Key Takeaways

1. **State is King**: All UI is derived from `self.todos` state
2. **Events Drive Changes**: All state changes happen through `on()`
3. **Immutable Updates**: Create new lists rather than modifying in place
4. **Event Routing**: Use event naming patterns to pass data
5. **Dynamic UI**: The UI automatically updates when state changes
