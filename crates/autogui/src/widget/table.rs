use core::num;

use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::widget::util::*;
use crate::widget::checkbox::Checkbox;
use crate::widget::dropdown::Dropdown;
use crate::widget::input::TextInput;
use crate::style::theme::ActiveTheme;
use autoval::Value;

#[derive(Debug, Clone)]
pub enum WidthMode {
    Pixels(f32),
    Percent(f32),
    Stretch,
}

#[derive(Debug, Clone)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone)]
pub enum ShowAs {
    Hex,
    Text,
    Checkbox,
    Dropdown,
    Input,
}

#[derive(Debug, Clone)]
pub struct ColConfig {
    pub idx: usize,
    pub id: String,
    pub title: String,
    pub width: WidthMode,
    pub align: Align,
    pub showas: ShowAs,
}

impl Default for WidthMode {
    fn default() -> Self {
        WidthMode::Pixels(100.0)
    }
}

impl Into<WidthMode> for Value {
    fn into(self) -> WidthMode {
        match self {
            Value::Float(f) => WidthMode::Pixels(f as f32),
            Value::Str(s) => {
                if s == "stretch" {
                    WidthMode::Stretch
                } else {
                    WidthMode::default()
                }
            }
            _ => WidthMode::default(),
        }
    }
}

// TODO: add Value::Enum
impl Into<ShowAs> for String {
    fn into(self) -> ShowAs {
        match self.as_str() {
            "Hex" => ShowAs::Hex,
            "Checkbox" => ShowAs::Checkbox,
            "Dropdown" => ShowAs::Dropdown,
            "Input" => ShowAs::Input,
            _ => ShowAs::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub cells: Vec<Value>,
}

pub struct TableUpdate {
    pub row: usize,
    pub col: usize,
    pub old: Value,
    pub new: Value,
}

impl TableUpdate {
    pub fn new(row: usize, col: usize, old: Value, new: Value) -> Self {
        Self { row, col, old, new }
    }
}

pub struct Table {
    focus_handle: FocusHandle,
    num_rows: usize,
    num_cols: usize,
    row_height: f32,
    config: Vec<ColConfig>,
    data: Vec<Row>,
    update_history: Vec<TableUpdate>,
}

impl Table {
    pub fn new(cx: &mut ViewContext<Self>, col_config: Vec<ColConfig>, data: Vec<Row>) -> Self {
        let num_cols = col_config.len();
        let num_rows = data.len();
        Self { focus_handle: cx.focus_handle(), num_rows: num_rows, num_cols: num_cols, row_height: 50.0, config: col_config, data, update_history: vec![] }
    }

    pub fn record_update(&mut self, update: TableUpdate) {
        self.update_history.push(update);
    }

    pub fn get_update_history(&self) -> &Vec<TableUpdate> {
        &self.update_history
    }

    pub fn get_update_history_as_string(&self) -> String {
        self.update_history.iter().map(|u| {
            format!("[{},{}]: {} -> {}", u.row, u.col, u.old, u.new)
        }).collect::<Vec<_>>().join("\n")
    }
}

impl Render for Table {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let view = cx.view().clone();
        div()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .bg(theme.table)
            .child(col()
                .id("table")
                .track_focus(&self.focus_handle)
                .overflow_hidden()
                .child(self.render_header(cx))
                .child(uniform_list(
                    view,
                    "table_rows",
                    self.num_rows,
                    {
                        move |table, range, cx| {
                            range.map(|idx| {
                                table.render_row(idx, table.num_rows, table.num_cols, cx)
                            })
                        }.collect::<Vec<_>>()
                    }
                )
                .flex_grow()
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .into_any_element())
            )
        }
}


impl Table {
    fn render_header(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let theme = cx.active_theme();
        let num_cols = self.num_cols;
        div()
            .w_full()
            .h(px(self.row_height))
            .border_b_1()
            .border_color(theme.border)
            .hover(|this| this.bg(theme.table_hover))
            .child(
                row().h_full().px_5().overflow_hidden().justify_start().items_center()
                    .children((0..num_cols).map(|colid| {
                        // cell
                        div().h_full().items_center()
                        .flex_shrink_0()
                        .flex()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .p(px(6.0))
                        .map(|this| {
                            let config = &self.config[colid];
                            let mut div = this;
                            div = match config.width {
                                WidthMode::Pixels(w) => div.w(px(w)),
                                WidthMode::Percent(p) => div.w(DefiniteLength::Fraction(p)),
                                WidthMode::Stretch => div.flex_grow(),
                            };
                            div = match config.align {
                                Align::Start => div.justify_start(),
                                Align::Center => div.justify_center(),
                                Align::End => div.justify_end(),
                            };
                            div.child(config.title.clone())
                        })

                }))
            )
    }

    fn render_row(&mut self, rowid: usize, num_rows: usize, num_cols: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let theme = cx.active_theme();
        let is_even = rowid % 2 == 0;
        div()
            .w_full()
            .justify_start()
            .h(px(self.row_height))
            .border_b_1()
            .border_color(theme.border)
            .when(is_even, |e| e.bg(theme.table_even))
            .hover(|this| this.bg(theme.table_hover))
            .child(
                row().h_full().px_5().overflow_hidden().justify_start().items_center()
                    .children((0..num_cols).map(|colid| {
                        // cell
                        div().h_full().items_center()
                        .flex_shrink_0()
                        .flex()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .p(px(6.0))
                        .map(|this| {
                            let config = &self.config[colid];
                            let mut div = this;
                            div = match config.width {
                                WidthMode::Pixels(w) => div.w(px(w)),
                                WidthMode::Percent(p) => div.w(DefiniteLength::Fraction(p)),
                                WidthMode::Stretch => div.flex_grow(),
                            };
                            div = match config.align {
                                Align::Start => div.justify_start(),
                                Align::Center => div.justify_center(),
                                Align::End => div.justify_end(),
                            };
                            let cell = &self.data[rowid].cells[colid];
                            match config.showas {
                                ShowAs::Text => div.child(cell.to_string()),
                                ShowAs::Hex => {
                                    match cell {
                                        Value::Int(i) => div.child(format!("0x{:X}", i)),
                                        Value::Uint(u) => div.child(format!("0x{:X}", u)),
                                        _ => div.child(cell.to_string()),
                                    }
                                }
                                ShowAs::Checkbox => {
                                    div.child(Checkbox::new("cb").checked(cell.to_bool())
                                        .on_click_mut(cx, move |this, b, cx| {
                                            println!("check box clicked, {}", *b);
                                            // let old = Value::Bool(!*b);
                                            // let new = Value::Bool(*b);
                                            // this.data[rowid].cells[colid] = new.clone();
                                            // this.record_update(TableUpdate::new(rowid, colid, old, new));
                                            this.data[rowid].cells[colid] = Value::Bool(*b);
                                            cx.notify();
                                        })
                                    )
                                },
                                ShowAs::Dropdown => {
                                    div.child(cx.new_view(|cx| Dropdown::new("dd", vec!["Intel".into(), "Motorola".into()], Some(0), cx)))
                                },
                                ShowAs::Input => div.child(cx.new_view(|cx| {
                                    let mut input = TextInput::new(cx);
                                    input.set_text(cell.to_string(), cx);
                                    input
                                })),
                            }
                        })

                }))
            )
    }
}