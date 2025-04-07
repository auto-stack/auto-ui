use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::app::{GlobalDataStoreCollectAction, GlobalDataStore};
use crate::widget::util::*;
use crate::widget::checkbox::Checkbox;
use crate::widget::dropdown::{Dropdown, DropdownEvent};
use crate::widget::input::{TextInput, InputEvent};
use crate::widget::scroll::{ScrollbarState, Scrollbar};
use crate::style::theme::ActiveTheme;
use crate::widget::util::bool_icon;
use auto_val::{ValueKey, Value, Obj, Grid, AutoStr};
use std::rc::Rc;
use std::cell::Cell;

#[derive(Debug, Clone)]
pub enum WidthMode {
    Pixels(f32),
    Percent(f32),
    Auto,
}

#[derive(Debug, Clone)]
pub enum Align {
    Start,
    Center,
    End,
}

impl From<Value> for Align {
    fn from(v: Value) -> Self {
        match v {
            Value::Str(s) => match s.to_lowercase().as_str() {
                "start" => Align::Start,
                "center" => Align::Center,
                "end" => Align::End,
                _ => Align::Start,
            }
            _ => Align::Start,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Format {
    Hex,
    Text,
    Checkbox(CheckboxConfig),
    Bool,
    Dropdown,
    Input,
    NumberInput,
}

#[derive(Debug, Clone)]
pub struct CheckboxConfig {
    pub disabled: bool,
}

impl Default for CheckboxConfig {
    fn default() -> Self {
        Self { disabled: false }
    }
}

impl Format {
    pub fn checkbox() -> Self {
        Format::Checkbox(CheckboxConfig::default())
    }

    pub fn disabled_checkbox() -> Self {
        Format::Checkbox(CheckboxConfig { disabled: true })
    }
}


#[derive(Debug, Clone)]
pub struct ColConfig {
    pub idx: usize,
    pub id: String,
    pub title: String,
    pub width: WidthMode,
    pub align: Align,
    pub format: Format,
    pub options: Vec<String>,
}

impl Default for ColConfig {
    fn default() -> Self {
        Self { idx: 0, id: "".to_string(), title: "".to_string(), width: WidthMode::default(), align: Align::Start, format: Format::Text, options: vec![] }
    }
}

impl ColConfig {
    pub fn from_grid_head(values: &Vec<(ValueKey, Value)>) -> Vec<Self> {
        // default config 
        let mut configs = Vec::new();

        for (i, (key, val)) in values.iter().enumerate() {
            let mut config = ColConfig::default();
            config.idx = i;
            config.id = key.to_string();
            match val {
                Value::Str(s) => {
                    config.title = s.to_string();
                }
                Value::Obj(obj) => {
                    config.title = obj.get_str_or("title", "").to_string();
                    config.width = obj.get_or("width", Value::Float(0.0)).into();
                    config.align = obj.get_or("align", Value::from("start")).into();
                    config.format = obj.get_or("format", Value::from("Text")).into();
                    config.options = obj.get_array_of("options").iter().map(|s| s.repr().to_string()).collect::<Vec<String>>();
                }
                _ => (),
            }
            configs.push(config);
        }
        configs
    }
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
                println!("width mode: {}", s);
                if s == "Auto" || s == "auto" {
                    WidthMode::Auto
                } else {
                    WidthMode::default()
                }
            }
            _ => WidthMode::default(),
        }
    }
}

impl From<AutoStr> for Format {
    fn from(s: AutoStr) -> Self {
        match s.as_str() {
            "Hex" => Format::Hex,
            "Checkbox" => Format::Checkbox(CheckboxConfig::default()),
            "Dropdown" => Format::Dropdown,
            "Input" => Format::Input,
            "Bool" => Format::Bool,
            "NumberInput" => Format::NumberInput,
            _ => Format::Text,
        }
    }
}   

// TODO: add Value::Enum
impl From<Value> for Format {
    fn from(v: Value) -> Self {
        match v {
            Value::Str(s) => s.into(),
            Value::Obj(obj) => {
                let kind = obj.get_str_or("kind", "Text");
                match kind.as_str() {
                    "Checkbox" => {
                        let disabled = obj.get_bool_or("disabled", false);
                        Format::Checkbox(CheckboxConfig { disabled })
                    }
                    _ => kind.into(),
                }
            }
            _ => Format::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub cells: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct RowView {
    pub rowid: usize,
    pub cell_views: Vec<CellView>
}

#[derive(Debug, Clone)]
pub struct CellView {
    pub colid: usize,
    pub view: AnyView
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
    id: AutoStr,
    bounds: Bounds<Pixels>,
    focus_handle: FocusHandle,
    v_scroll: UniformListScrollHandle,
    scroll_state: Rc<Cell<ScrollbarState>>,
    num_rows: usize,
    num_cols: usize,
    row_height: f32,
    config: Vec<ColConfig>,
    data: Vec<Vec<Value>>,
    grid_head: Vec<(ValueKey, Value)>,
    row_views: Vec<RowView>,
    update_history: Vec<TableUpdate>,
}

impl Table {
    pub fn new(cx: &mut ViewContext<Self>, id: AutoStr, col_config: Vec<ColConfig>, data: Vec<Vec<Value>>, head: Vec<(ValueKey, Value)>) -> Self {
        let num_cols = col_config.len();
        let num_rows = data.len();
        let table_id = id.clone();
        cx.observe_global::<GlobalDataStoreCollectAction>(move |this, cx| {
            let grid = this.collect_data();
            let table_id = table_id.clone();
            GlobalDataStore::update_global(cx, move |g, cx| {
                g.table_grids.set(table_id, Value::Grid(grid));
            });
        }).detach();
        let row_views = Self::identify_row_views(cx, &col_config, &data);
        Self {
            id, 
            bounds: Bounds::default(),
            focus_handle: cx.focus_handle(), 
            v_scroll: UniformListScrollHandle::new(),
            scroll_state: Rc::new(Cell::new(ScrollbarState::new())),
            num_rows: num_rows, 
            num_cols: num_cols, 
            row_height: 42.0, 
            config: col_config, 
            data, 
            grid_head: head,
            row_views,
            update_history: vec![] 
        }
    }

    pub fn from_grid(cx: &mut ViewContext<Self>, id: AutoStr, grid: Grid) -> Self {
        let data = grid.data.clone();
        let col_config = ColConfig::from_grid_head(&grid.head);
        Self::new(cx, id, col_config, data, grid.head)
    }

    pub fn identify_row_views(cx: &mut ViewContext<Self>, col_config: &Vec<ColConfig>, data: &Vec<Vec<Value>>) -> Vec<RowView> {
        let mut row_views = Vec::new();
        for (rowid, row) in data.iter().enumerate() {
            let mut cell_views = Vec::new();
            for (colid, col) in col_config.iter().enumerate() {
                let format = &col.format;
                match format {
                    Format::Dropdown => {
                        let options = col.options.iter().map(|s| s.into()).collect::<Vec<SharedString>>();
                        let selected = row[colid].clone().as_uint() as usize;
                        let view = cx.new_view(|cx| {
                            Dropdown::new("dropdown", options, Some(selected), cx)
                        });
                        // &view.read(cx).on_selected(cx.listener(move |t, e, _cx| {
                        //     t.update_cell(rowid, colid, Value::Uint(*e as u32));
                        // }));
                        let r = rowid.clone();
                        let c = colid.clone();
                        cx.subscribe(&view, move |t, _v, e, _cx| {
                            let DropdownEvent::Selected(i) = e;
                            t.update_cell(r, c, Value::Uint(*i as u32));
                        }).detach();
                        cell_views.push(CellView { colid, 
                            view: view.into()
                        });
                    }
                    Format::Input => {
                        let cell = &row[colid];
                        let view = cx.new_view(|cx| {
                            let mut input = TextInput::new(cx);
                            input.set_text(cell.repr().to_string(), cx);
                            input
                        });
                        let r = rowid.clone();
                        let c = colid.clone();
                        cx.subscribe(&view, move |t, _v, e, _cx| {
                            match e {
                                InputEvent::Change(s) => t.update_cell(r, c, Value::Str(s.clone().to_string().into())),
                                _ => (),
                            }
                        }).detach();

                        cell_views.push(CellView { colid, 
                            view: view.into()
                        });
                    }
                    Format::NumberInput => {
                        let cell = &row[colid];
                        let view = cx.new_view(|cx| {
                            let mut input = TextInput::new(cx);
                            let text = cell.repr();
                            if text.is_empty() {
                                input.set_text("0", cx);
                            } else {
                                input.set_text(text.to_string(), cx);
                            }
                            input
                        });
                        let r = rowid.clone();
                        let c = colid.clone();
                        cx.subscribe(&view, move |t, _v, e, _cx| {
                            match e {
                                InputEvent::Change(s) => {
                                    let i = s.parse::<i32>();
                                    if let Ok(i) = i {
                                        t.update_cell(r, c, Value::Int(i));
                                    }
                                }
                                _ => (),
                            }
                        }).detach();

                        cell_views.push(CellView { colid, 
                            view: view.into()
                        });
                    }
                    _ => (),
                }
            }
            if cell_views.len() > 0 {
                row_views.push(RowView { rowid, cell_views });
            }
        }
        row_views
    }

    pub fn update_cell(&mut self, rowid: usize, colid: usize, new: Value) {
        self.data[rowid][colid] = new;
    }

    pub fn on_dropdown_event(&mut self, _dd: View<Dropdown>, ev: &DropdownEvent, _cx: &mut ViewContext<Self>) {
        match ev {
            DropdownEvent::Selected(i) => {
                println!("table: dropdown selected: {}", i);
            }
        }
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

    pub fn collect_data(&self) -> Grid {
        let mut rows = Vec::new();
        for row in self.data.iter() {
            rows.push(row.clone());
        }
        let grid = Grid {
            head: self.grid_head.clone(),
            data: rows,
        };
        grid
    }

    pub fn collect_data_obj(&self) -> Value {
        let mut rows = Vec::new();
        for row in self.data.iter() {
            let mut obj = Obj::new();
            for (idx, cell) in row.iter().enumerate() {
                match self.config.get(idx) {
                    Some(col) => obj.set(col.id.clone(), cell.clone()),
                    None => (),
                }
            }
            rows.push(Value::Obj(obj));
        }
        Value::Array(rows.into())
    }
}

impl Render for Table {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let view = cx.view().clone();
        let view1 = view.clone();
        let v_scroll = self.v_scroll.clone();
        let row_height = self.v_scroll.0.borrow().last_item_size.map(|size| size.item.height);
        let total_height = self.v_scroll.0.borrow().base_handle.bounds().size.height;


        div()
            .size_full()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .shadow_md()    
            .bg(theme.table)
            .child(col()
                .id("table")
                .track_focus(&self.focus_handle)
                .overflow_hidden()
                .size_full()
                .child(self.render_header(cx))
                .child(row().id("table_body").size_full().child(
                    uniform_list(
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
                    .track_scroll(v_scroll)
                    .into_any_element())
                )
            )
            .child(canvas(
                move |bounds, cx| view1.update(cx, |r, _| r.bounds = bounds),
                |_, _, _| {},
            ))
            .children(self.render_scrollbar(cx))
        }
}


impl Table {
    fn render_scrollbar(&mut self, cx: &mut ViewContext<Self>) -> Option<impl IntoElement> {
        let state = self.scroll_state.clone();
        let theme = cx.active_theme();

        Some(
            div()
                .absolute()
                .top(px(self.row_height))
                .left_0()
                .right_0()
                .bottom_0()
                .child(Scrollbar::uniform_scroll(
                    cx.view().entity_id(),
                    state,
                    self.v_scroll.clone(),
                )),
        )
    }

    fn render_header(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        let num_cols = self.num_cols;
        div()
            .w_full()
            .h(px(self.row_height))
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.table_head)
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
                                WidthMode::Auto => {
                                    div.flex_grow()
                                }
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
            // .when(rowid < num_rows - 1, |this| this.border_b_1().border_color(theme.border))
            .border_b_1().border_color(theme.border)
            .when(!is_even, |e| e.bg(theme.table_even))
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
                                WidthMode::Auto => div.flex_grow(),
                            };
                            div = match config.align {
                                Align::Start => div.justify_start(),
                                Align::Center => div.justify_center(),
                                Align::End => div.justify_end(),
                            };
                            let cell = &self.data[rowid][colid];
                            match &config.format {
                                Format::Text => div.child(cell.repr().to_string()),
                                Format::Hex => {
                                    match cell {
                                        Value::Int(i) => div.child(format!("0x{:04X}", i)),
                                        Value::Uint(u) => div.child(format!("0x{:04X}", u)),
                                        _ => div.child(cell.repr().to_string()),
                                    }
                                }
                                Format::Checkbox(config) => {
                                    div.child(Checkbox::new("cb")
                                        .disabled(config.disabled)
                                        .checked(cell.to_bool())
                                        .on_click_mut(cx, move |this, b, cx| {
                                            println!("check box clicked, {}", *b);
                                            // let old = Value::Bool(!*b);
                                            // let new = Value::Bool(*b);
                                            // this.data[rowid].cells[colid] = new.clone();
                                            // this.record_update(TableUpdate::new(rowid, colid, old, new));
                                            this.data[rowid][colid] = Value::Bool(*b);
                                            cx.notify();
                                        })
                                    )
                                },
                                Format::Dropdown => {
                                    // find the cell view
                                    let cell_views = &self.row_views.get(rowid).unwrap().cell_views;
                                    let cell_view = cell_views.iter().find(|cv| cv.colid == colid).unwrap();
                                    div.child(cell_view.view.clone())
                                },
                                Format::Input =>  {
                                    let cell_views = &self.row_views.get(rowid).unwrap().cell_views;
                                    let cell_view = cell_views.iter().find(|cv| cv.colid == colid).unwrap();
                                    div.child(cell_view.view.clone())
                                },
                                Format::NumberInput => {
                                    let cell_views = &self.row_views.get(rowid).unwrap().cell_views;
                                    let cell_view = cell_views.iter().find(|cv| cv.colid == colid).unwrap();
                                    div.child(cell_view.view.clone())
                                }
                                Format::Bool => {
                                    div.child(bool_icon(cell.to_bool()))
                                },
                            }
                        })

                }))
            )
    }
}