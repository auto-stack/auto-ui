use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::widget::util::*;
use crate::widget::checkbox::Checkbox;
use crate::widget::dropdown::Dropdown;
use crate::style::theme::ActiveTheme;

pub struct Table {
    focus_handle: FocusHandle,
    num_rows: usize,
    num_cols: usize,
    row_height: f32,
    dropdown: View<Dropdown>,
}

impl Table {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self { focus_handle: cx.focus_handle(), num_rows: 10, num_cols: 6, row_height: 50.0, dropdown: cx.new_view(|cx| Dropdown::new("dd", vec!["Intel".into(), "Motorola".into()], Some(0), cx)) }
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
        div()
            .w_full()
            .h(px(self.row_height))
            .border_b_1()
            .border_color(theme.border)
            .hover(|this| this.bg(theme.table_hover))
            .child(
                row().h_full().overflow_hidden().justify_center().items_center()
                    .children((0..self.num_cols).map(|colid| {
                        // cell
                        div().h_full().items_center()
                        .flex_shrink_0()
                        .flex()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .p(px(6.0))
                        .map(|this| {
                            match colid {
                                0 => this.w(px(100.0)).child("ID"),
                                1 => this.w(px(100.0)).child("Name"),
                                2 => this.w(px(100.0)).child("Age"),
                                3 => this.w(px(100.0)).child("Active"),
                                4 => this.w(px(150.0)).child("Order"),
                                _ => this.flex_grow().child("Misc")
                            }
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
                            match colid {
                                0 => this.w(px(100.0)).child(format!("{}", rowid)),
                                1 => this.w(px(100.0)).child(format!("Name")),
                                2 => this.w(px(100.0)).child(format!("12")),
                                3 => this.w(px(100.0)).child(Checkbox::new("cb").checked(true)),
                                4 => this.w(px(150.0)).child(self.dropdown.clone()),
                                _ => this.flex_grow().child(format!("--"))
                            }
                        })

                }))
            )
    }
}