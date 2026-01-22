// Todo List example
//
// Demonstrates state management with a practical CRUD application
//
// Run with:
//   cargo run --package todos-example --features iced

use auto_ui::{Component, View, App};

#[derive(Debug, Default)]
struct TodoApp {
    todos: Vec<Todo>,
    next_id: usize,
}

#[derive(Debug, Clone)]
struct Todo {
    id: usize,
    description: String,
    completed: bool,
}

#[derive(Debug, Clone)]
enum Message {
    AddTodo,
    ToggleTodo(usize),
    DeleteTodo(usize),
}

impl Component for TodoApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::AddTodo => {
                self.todos.push(Todo {
                    id: self.next_id,
                    description: format!("Task {}", self.next_id + 1),
                    completed: false,
                });
                self.next_id += 1;
            }
            Message::ToggleTodo(id) => {
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                }
            }
            Message::DeleteTodo(id) => {
                self.todos.retain(|t| t.id != id);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(40)
            .child(View::text("Todo List"))
            .child(View::button("Add Task", Message::AddTodo))
            .child(View::text(format!("Total: {} tasks", self.todos.len())))
            .child(View::text("Click Add Task to create new todos"))
            .child(View::text("Click on todo items to toggle completion"))
            .build()
    }
}

fn main() -> auto_ui::AppResult<()> {
    #[cfg(feature = "iced")]
    {
        println!("🎨 Running todo list example with Iced backend");
        return auto_ui_iced::run_app::<TodoApp>();
    }

    #[cfg(feature = "gpui")]
    {
        println!("🎨 Running todo list example with GPUI backend");
        return auto_ui_gpui::run_app::<TodoApp>("Todos - AutoUI");
    }

    #[cfg(not(any(feature = "iced", feature = "gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --features iced\n\
             • cargo run --features gpui"
                .into(),
        )
    }
}
