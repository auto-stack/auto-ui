use gpui::SharedString;

pub struct TableColumn {
    pub id: SharedString,
    pub name: SharedString,
}

impl TableColumn {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

