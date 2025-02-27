use crate::Widget;
use crate::Text;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Default, Clone)]
pub enum Pane {
    #[default]
    Empty,
    // Raw(Kids),
    Center(Kids),
    // Left(Kids),
    // Right(Kids),
    // Bottom(Kids),
    // Top(Kids),
    // Row(Kids),
    // Col(Kids),
    // WorkSpace(WorkSpace),
}

pub struct WorkSpace {
    pub left: Kids,
    pub right: Kids,
    pub bottom: Kids,
    pub top: Kids,
}

pub type Kids = Vec<Kid>;

#[derive(Clone)]
pub enum Kid{
    // Pane(Pane),
    Widget(Widget),
}

