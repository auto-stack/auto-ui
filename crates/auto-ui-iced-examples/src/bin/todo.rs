// TodoMVC example using auto-ui abstraction layer with Iced adapter
//
// This demonstrates a more complex application with state management,
// list operations, and conditional rendering.

use auto_ui::{Component, View};
use auto_ui_iced::ComponentIced;

#[derive(Debug, Clone)]
struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Debug)]
struct TodoApp {
    todos: Vec<TodoItem>,
    next_id: usize,
    filter: Filter,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self {
            todos: Vec::new(),
            next_id: 0,
            filter: Filter::All,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug)]
enum Message {
    AddTodo,
    RemoveTodo(usize),
    ToggleTodo(usize),
    SetFilter(Filter),
    ClearCompleted,
}

impl Component for TodoApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::AddTodo => {
                self.todos.push(TodoItem {
                    id: self.next_id,
                    text: format!("Todo {}", self.next_id + 1),
                    completed: false,
                });
                self.next_id += 1;
            }
            Message::RemoveTodo(id) => {
                self.todos.retain(|todo| todo.id != id);
            }
            Message::ToggleTodo(id) => {
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                }
            }
            Message::SetFilter(filter) => {
                self.filter = filter;
            }
            Message::ClearCompleted => {
                self.todos.retain(|todo| !todo.completed);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        let filtered_todos: Vec<_> = match self.filter {
            Filter::All => self.todos.clone(),
            Filter::Active => self.todos.iter().filter(|t| !t.completed).cloned().collect(),
            Filter::Completed => self.todos.iter().filter(|t| t.completed).cloned().collect(),
        };

        // Build todo list items
        let mut todo_views = Vec::new();
        for todo in &filtered_todos {
            todo_views.push(
                View::row()
                    .spacing(8)
                    .padding(4)
                    .child(View::text(format!("{} {}", if todo.completed { "✓" } else { "○" }, todo.text)))
                    .child(View::button("Remove", Message::RemoveTodo(todo.id)))
                    .build()
            );
        }

        // Filter buttons
        let filter_buttons = View::row()
            .spacing(8)
            .padding(8)
            .child(View::button(
                format!("All ({})", self.todos.len()),
                Message::SetFilter(Filter::All),
            ))
            .child(View::button(
                format!("Active ({})", self.todos.iter().filter(|t| !t.completed).count()),
                Message::SetFilter(Filter::Active),
            ))
            .child(View::button(
                format!("Completed ({})", self.todos.iter().filter(|t| t.completed).count()),
                Message::SetFilter(Filter::Completed),
            ))
            .child(View::button("Clear Completed", Message::ClearCompleted))
            .build();

        View::col()
            .spacing(16)
            .padding(20)
            .child(View::text("TodoMVC".to_string()))
            .child(View::button("Add Todo", Message::AddTodo))
            .child(filter_buttons)
            .children(todo_views)
            .build()
    }
}

fn main() -> iced::Result {
    iced::run(TodoApp::update, view)
}

fn view(app: &TodoApp) -> iced::Element<'_, Message> {
    app.view_iced()
}
