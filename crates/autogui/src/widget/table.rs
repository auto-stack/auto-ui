use core::num;

use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::widget::util::*;
use crate::widget::checkbox::Checkbox;
use crate::widget::dropdown::Dropdown;
use crate::widget::input::TextInput;
use crate::style::theme::ActiveTheme;
use autolang::value::Value;

pub enum WidthMode {
    Pixels(f32),
    Percent(f32),
    Stretch,
}

pub enum Align {
    Start,
    Center,
    End,
}

pub enum ShowAs {
    Hex,
    Text,
    Checkbox,
    Dropdown,
    Input,
}

pub struct ColConfig {
    pub idx: usize,
    pub title: String,
    pub width: WidthMode,
    pub align: Align,
    pub showas: ShowAs,
}

pub struct Row {
    pub cells: Vec<Value>,
}

pub struct Table {
    focus_handle: FocusHandle,
    num_rows: usize,
    num_cols: usize,
    row_height: f32,
    config: Vec<ColConfig>,
    data: Vec<Row>,
}

impl Table {
    pub fn new(cx: &mut ViewContext<Self>, col_config: Vec<ColConfig>, data: Vec<Row>) -> Self {
        let num_cols = col_config.len();
        let num_rows = data.len();
        Self { focus_handle: cx.focus_handle(), num_rows: num_rows, num_cols: num_cols, row_height: 50.0, config: col_config, data }
    }
}

impl Render for Table {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let view = cx.view().clone();
        div()
            .size_full()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .bg(theme.table)
            .child(col()
                .id("table")
                .track_focus(&self.focus_handle)
                .size_full()
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
                .size_full()
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
                row().h_full().overflow_hidden().justify_center().items_center()
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
            .h(px(self.row_height))
            .border_b_1()
            .border_color(theme.border)
            .when(is_even, |e| e.bg(theme.table_even))
            .hover(|this| this.bg(theme.table_hover))
            .child(
                row().h_full().overflow_hidden().justify_center().items_center()
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
                                        Value::Int(i) => div.child(format!("0x{:x}", i)),
                                        _ => div.child(cell.to_string()),
                                    }
                                }
                                ShowAs::Checkbox => {
                                    div.child(Checkbox::new("cb").checked(cell.to_bool())
                                        .on_click_mut(cx, move |this, b, cx| {
                                            println!("checkbox clicked");
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