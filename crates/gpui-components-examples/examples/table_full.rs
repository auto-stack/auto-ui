use std::ops::Range;
use auto_ui::*;
use gpui::{
    div, px, Application, App, AppContext, Context, Entity, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, Pixels, Edges,
    impl_internal_actions,
};
use fake::{Fake, Faker};

use gpui_component::{
    v_flex,
    // red, green, yellow_500, yellow_800, ColorName, IconName, Kbd, Sizable, StyledExt,
    StyleSized as _,
    Size,
    popup_menu::PopupMenu,
    table::{Table, TableDelegate, ColFixed, ColSort},
};
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct ChangeSize(Size);

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct OpenDetail(usize);

impl_internal_actions!(table, [ChangeSize, OpenDetail]);

struct Row {
    id: usize,
    symbol: SharedString,
    name: SharedString,
}

struct Column {
    id: SharedString,
    name: SharedString,
    sort: Option<ColSort>,
}

impl Column {
    fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        sort: Option<ColSort>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sort,
        }
    }
}


struct MyTableDelegate {
    rows: Vec<Row>,
    columns: Vec<Column>,
    size: Size,
}

impl TableDelegate for MyTableDelegate {
    fn cols_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn col_name(&self, col_ix: usize, _: &App) -> SharedString {
        if let Some(col) = self.columns.get(col_ix) {
            col.name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize, _: &App) -> Pixels {
        if col_ix < 10 {
            150.0.into()
        } else if col_ix < 20 {
            150.0.into()
        } else {
            200.0.into()
        }
    }

    fn col_padding(&self, col_ix: usize, _: &App) -> Option<Edges<Pixels>> {
        if col_ix >= 3 && col_ix <= 10 {
            Some(Edges::all(px(0.)))
        } else {
            None
        }
    }

    fn col_fixed(&self, _col_ix: usize, _: &App) -> Option<ColFixed> {
        return None;
    }

    fn can_resize_col(&self, _col_ix: usize, _: &App) -> bool {
        return true;
    }

    fn can_select_col(&self, _: usize, _: &App) -> bool {
        return true;
    }

    fn render_th(
        &self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let th = div().child(self.col_name(col_ix, cx));

        if col_ix >= 3 && col_ix <= 10 {
            th.table_cell_size(self.size)
        } else {
            th
        }
    }

    fn context_menu(
        &self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &Window,
        _cx: &App,
    ) -> PopupMenu {
        menu.menu(
            format!("Selected Row: {}", row_ix),
            Box::new(OpenDetail(row_ix)),
        )
        .separator()
        .menu("Size Large", Box::new(ChangeSize(Size::Large)))
        .menu("Size Medium", Box::new(ChangeSize(Size::Medium)))
        .menu("Size Small", Box::new(ChangeSize(Size::Small)))
        .menu("Size XSmall", Box::new(ChangeSize(Size::XSmall)))
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let row = self.rows.get(row_ix).unwrap();
        let col = self.columns.get(col_ix).unwrap();

        match col.id.as_ref() {
            "id" => row.id.to_string().into_any_element(),
            "name" => row.name.clone().into_any_element(),
            "symbol" => row.symbol.clone().into_any_element(),
            _ => "--".to_string().into_any_element(),
        }
    }

    fn can_loop_select(&self, _: &App) -> bool {
        true
    }

    fn can_move_col(&self, _: usize, _: &App) -> bool {
        true
    }

    fn move_col(
        &mut self,
        col_ix: usize,
        to_ix: usize,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        let col = self.columns.remove(col_ix);
        self.columns.insert(to_ix, col);
    }

    fn col_sort(&self, _col_ix: usize, _: &App) -> Option<ColSort> {
        return None;
    }

    fn perform_sort(
        &mut self,
        _col_ix: usize,
        _sort: ColSort,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        return;
    }

    fn loading(&self, _: &App) -> bool {
        false
    }

    fn can_load_more(&self, _: &App) -> bool {
        return false;
    }

    fn load_more_threshold(&self) -> usize {
        150
    }

    fn load_more(&mut self, _: &mut Window, _cx: &mut Context<Table<Self>>) {

    }

    fn visible_rows_changed(
        &mut self,
        _visible_range: Range<usize>,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        // self.visible_rows = visible_range;
    }

    fn visible_cols_changed(
        &mut self,
        _visible_range: Range<usize>,
        _: &mut Window,
        _: &mut Context<Table<Self>>,
    ) {
        // self.visible_cols = visible_range;
    }
}

fn prepare_rows() -> Vec<Row> {
    vec![
        Row {
            id: 1,
            symbol: "AAPL".into(),
            name: "Apple Inc.".into(),
        },
        Row {
            id: 2,
            symbol: "GOOG".into(),
            name: "Google Inc.".into(),
        },
        Row {
            id: 3,
            symbol: "MSFT".into(),
            name: "Microsoft Corp.".into(),
        },
    ]
}

impl MyTableDelegate {
    fn new() -> Self {
        Self {
            size: Size::default(),
            rows: prepare_rows(),
            columns: vec![
                Column::new("id", "ID", None),
                Column::new("symbol", "Symbol", None),
                Column::new("name", "Name", None),
            ],
        }
    }
}

pub struct TableStory {
    table: Entity<Table<MyTableDelegate>>,
}

impl Story for TableStory {
    fn title() -> &'static str {
        "Table"
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }

    fn closable() -> bool {
        false
    }
}

impl Focusable for TableStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.table.focus_handle(cx)
    }
}

impl TableStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {

        let delegate = MyTableDelegate::new();
        let table = cx.new(|cx| Table::new(delegate, window, cx));

        Self {
            table,
        }
    }
}

impl Render for TableStory {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {

        v_flex()
            .size_full()
            .text_sm()
            .gap_2()
            .child(self.table.clone())
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);
        cx.activate(true);

        create_new_window_sized("Table Example", StoryView::view::<TableStory>, cx, 1024, 768);
    });
}
