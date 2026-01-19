// View tree representation

/// Abstract view node
#[derive(Debug, Clone)]
pub enum View {
    Empty,
    Text(String),
    Container {
        child: Box<View>,
        padding: Option<u16>,
        center_x: bool,
        center_y: bool,
    },
    Row {
        children: Vec<View>,
        spacing: u16,
        padding: Option<u16>,
    },
    Column {
        children: Vec<View>,
        spacing: u16,
        padding: Option<u16>,
    },
    Button {
        label: String,
        on_press: Option<String>, // message identifier
    },
    Input {
        placeholder: String,
        value: String,
        on_change: Option<String>,
    },
}

impl View {
    pub fn empty() -> Self {
        View::Empty
    }

    pub fn text(text: impl Into<String>) -> Self {
        View::Text(text.into())
    }

    pub fn container(child: View) -> Self {
        View::Container {
            child: Box::new(child),
            padding: None,
            center_x: false,
            center_y: false,
        }
    }

    pub fn row() -> Self {
        View::Row {
            children: Vec::new(),
            spacing: 0,
            padding: None,
        }
    }

    pub fn column() -> Self {
        View::Column {
            children: Vec::new(),
            spacing: 0,
            padding: None,
        }
    }

    pub fn button(label: impl Into<String>) -> Self {
        View::Button {
            label: label.into(),
            on_press: None,
        }
    }

    pub fn input(placeholder: impl Into<String>) -> Self {
        View::Input {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: None,
        }
    }
}
