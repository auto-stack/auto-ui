// TodoMVC example using auto-ui abstraction layer with GPUI adapter
//
// This demonstrates a more complex application with state management,
// list operations, and conditional rendering.

use auto_ui::{Component, View};
use gpui::*;
use gpui_component::{button::Button, button::ButtonVariants, *};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Debug, Clone)]
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
                    .child(View::text(format!(
                        "{} {}",
                        if todo.completed { "✓" } else { "○" },
                        todo.text
                    )))
                    .child(View::button("Remove", Message::RemoveTodo(todo.id)))
                    .build(),
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
                format!(
                    "Active ({})",
                    self.todos.iter().filter(|t| !t.completed).count()
                ),
                Message::SetFilter(Filter::Active),
            ))
            .child(View::button(
                format!(
                    "Completed ({})",
                    self.todos.iter().filter(|t| t.completed).count()
                ),
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

// GPUI Renderer for TodoApp
struct TodoRenderer {
    app: TodoApp,
}

impl TodoRenderer {
    fn new() -> Self {
        Self {
            app: TodoApp::default(),
        }
    }
}

impl Render for TodoRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filter = self.app.filter;
        let todos = &self.app.todos;

        let filtered_todos: Vec<_> = match filter {
            Filter::All => todos.clone(),
            Filter::Active => todos.iter().filter(|t| !t.completed).cloned().collect(),
            Filter::Completed => todos.iter().filter(|t| t.completed).cloned().collect(),
        };

        div()
            .v_flex()
            .gap_3()
            .p_4()
            .size_full()
            .child(div().text_xl().child("TodoMVC"))
            .child(
                Button::new("add-todo")
                    .primary()
                    .label("Add Todo")
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.app.on(Message::AddTodo);
                    })),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        Button::new("filter-all")
                            .label(format!("All ({})", todos.len()))
                            .selected(filter == Filter::All)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::SetFilter(Filter::All));
                            })),
                    )
                    .child(
                        Button::new("filter-active")
                            .label(format!(
                                "Active ({})",
                                todos.iter().filter(|t| !t.completed).count()
                            ))
                            .selected(filter == Filter::Active)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::SetFilter(Filter::Active));
                            })),
                    )
                    .child(
                        Button::new("filter-completed")
                            .label(format!(
                                "Completed ({})",
                                todos.iter().filter(|t| t.completed).count()
                            ))
                            .selected(filter == Filter::Completed)
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::SetFilter(Filter::Completed));
                            })),
                    )
                    .child(
                        Button::new("clear-completed")
                            .small()
                            .label("Clear Completed")
                            .on_click(cx.listener(|view, _, _, _cx| {
                                view.app.on(Message::ClearCompleted);
                            })),
                    ),
            )
            .children(filtered_todos.into_iter().enumerate().map(|(idx, todo)| {
                let todo_id = todo.id;
                div()
                    .h_flex()
                    .gap_2()
                    .p_2()
                    .child(div().child(format!(
                        "{} {}",
                        if todo.completed { "✓" } else { "○" },
                        todo.text
                    )))
                    .child(
                        Button::new(("remove", idx))
                            .small()
                            .label("Remove")
                            .on_click(cx.listener(move |view, _, _, _cx| {
                                view.app.on(Message::RemoveTodo(todo_id));
                            })),
                    )
            }))
    }
}

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point { x: px(100.0), y: px(100.0) },
                        size: gpui::Size {
                            width: px(800.0),
                            height: px(600.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("TodoMVC - AutoUI GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| TodoRenderer::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
