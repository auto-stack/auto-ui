use gpui_component::{
    StyleSized as _,
    popup_menu::PopupMenu,
};
use std::ops::Range;

use auto_ui::TableColumn;

use serde::Deserialize;
use gpui::impl_internal_actions;
use gpui_component::Size;
use gpui_component::table::{Table, TableDelegate, ColFixed, ColSort};

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct ChangeSize(Size);

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct OpenDetail(usize);

impl_internal_actions!(table, [ChangeSize, OpenDetail]);

struct MyTableRow {
$ for c in cols {
    ${c.id}: ${c.typ},
$ }
}

impl MyTableRow {
    fn new(
$ for c in cols {
    ${c.id}: ${c.arg_type},
$ }
    ) -> Self {
        Self {
        $ for c in cols {
            ${c.id}: ${c.arg_value},
        $ }
        }
    }
}

struct MyTableDelegate {
    rows: Vec<MyTableRow>,
    columns: Vec<TableColumn>,
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
        $ for c in cols {
            "${c.id}" => row.${c.id}.${c.to_str}.into_any_element(),
        $ }
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

impl MyTableDelegate {
    fn new(rows: Vec<MyTableRow>) -> Self {
        Self {
            size: Size::default(),
            rows,
            columns: vec![
            $ for c in cols {
                TableColumn::new("${c.id}", "${c.name}"),
            $ }
            ],
        }
    }
}