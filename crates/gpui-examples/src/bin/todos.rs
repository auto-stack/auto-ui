// Todo List example for gpui-component
use gpui::*;
use gpui_component::{button::*, Root, *};

#[derive(Debug, Clone)]
struct Todo {
    id: usize,
    description: String,
    completed: bool,
}

pub struct TodoApp {
    todos: Vec<Todo>,
    next_id: usize,
}

impl TodoApp {
    fn new() -> Self {
        Self {
            todos: Vec::new(),
            next_id: 0,
        }
    }

    fn add_todo(&mut self) {
        self.todos.push(Todo {
            id: self.next_id,
            description: format!("Task {}", self.next_id + 1),
            completed: false,
        });
        self.next_id += 1;
    }

    fn toggle_todo(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }

    fn delete_todo(&mut self, id: usize) {
        self.todos.retain(|t| t.id != id);
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_6()
            .p_6()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Todo List")
            )
            .child(
                Button::new("add")
                    .primary()
                    .label("Add Task")
                    .on_click(cx.listener(|view, _, _, _cx| {
                        view.add_todo();
                    }))
            )
            .child(
                div()
                    .text_lg()
                    .child(format!("Total: {} tasks", self.todos.len()))
            )
            .child(
                div()
                    .text_sm()
                    .child("Click Add Task to create new todos")
            )
            .child(
                div()
                    .text_sm()
                    .child("Toggle checkbox to complete, delete button to remove")
            )
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .w(px(400.0))
                    .children(self.todos.iter().map(|todo| {
                        let todo_id = todo.id;
                        let todo_desc = todo.description.clone();
                        let todo_completed = todo.completed;

                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .p_3()
                            .border_1()
                            .border_color(rgb(0x333333))
                            .rounded_md()
                            .child(
                                div()
                                    .text_sm()
                                    .child(if todo_completed { "✓" } else { " " })
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .child(todo_desc)
                            )
                            .child(
                                Button::new(("delete", todo_id))
                                    .ghost()
                                    .label("×")
                                    .on_click(cx.listener(move |view, _, _, _cx| {
                                        view.delete_todo(todo_id);
                                    }))
                            )
                    }))
            )
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
                        size: gpui::Size { width: px(600.0), height: px(700.0) },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Todo List - GPUI".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| TodoApp::new());
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
