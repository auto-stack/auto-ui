// View tree representation - improved version with generic messages

use std::fmt::Debug;

/// Abstract view node - generic over message type M
///
/// This enum represents the abstract UI tree that can be adapted to different backends.
/// Messages are stored directly (not as Option) for simpler mapping to Auto language.
#[derive(Debug, Clone)]
pub enum View<M: Clone + Debug> {
    /// Empty placeholder
    Empty,

    /// Text display
    Text(String),

    /// Button with label and click handler
    Button {
        label: String,
        onclick: M,  // Direct message storage (Auto: `onclick: Msg.Inc`)
    },

    /// Horizontal layout
    Row {
        children: Vec<View<M>>,
        spacing: u16,
        padding: u16,
    },

    /// Vertical layout
    Column {
        children: Vec<View<M>>,
        spacing: u16,
        padding: u16,
    },

    /// Text input field
    Input {
        placeholder: String,
        value: String,
        on_change: Option<M>,  // Optional because not all inputs need handlers
    },

    /// Checkbox
    Checkbox {
        is_checked: bool,
        label: String,
        on_toggle: Option<M>,
    },
}

/// View builder for fluent layout construction
///
/// Provides chainable API for building complex layouts.
///
/// # Example
/// ```rust
/// View::col()
///     .spacing(10)
///     .padding(20)
///     .child(View::text("Hello"))
///     .child(View::button("Click", Msg::Click))
/// ```
pub struct ViewBuilder<M: Clone + Debug> {
    kind: ViewBuilderKind,
    children: Vec<View<M>>,
    spacing: u16,
    padding: u16,
}

#[derive(Debug, Clone, Copy)]
enum ViewBuilderKind {
    Row,
    Column,
}

impl<M: Clone + Debug> ViewBuilder<M> {
    /// Create a new row builder
    pub fn row() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Row,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
        }
    }

    /// Create a new column builder
    pub fn col() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Column,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
        }
    }

    /// Add a child to the layout
    pub fn child(mut self, child: View<M>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children to the layout
    pub fn children(mut self, children: impl IntoIterator<Item = View<M>>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set spacing between children
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding for the layout
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Build the final View
    pub fn build(self) -> View<M> {
        match self.kind {
            ViewBuilderKind::Row => View::Row {
                children: self.children,
                spacing: self.spacing,
                padding: self.padding,
            },
            ViewBuilderKind::Column => View::Column {
                children: self.children,
                spacing: self.spacing,
                padding: self.padding,
            },
        }
    }
}

impl<M: Clone + Debug> View<M> {
    /// Create an empty view
    pub fn empty() -> Self {
        View::Empty
    }

    /// Create text view
    pub fn text(content: impl Into<String>) -> Self {
        View::Text(content.into())
    }

    /// Create button with click handler
    pub fn button(label: impl Into<String>, onclick: M) -> Self {
        View::Button {
            label: label.into(),
            onclick,
        }
    }

    /// Create a row builder
    pub fn row() -> ViewBuilder<M> {
        ViewBuilder::row()
    }

    /// Create a column builder
    pub fn col() -> ViewBuilder<M> {
        ViewBuilder::col()
    }

    /// Create input field
    pub fn input(placeholder: impl Into<String>) -> Self {
        View::Input {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: None,
        }
    }

    /// Create checkbox
    pub fn checkbox(is_checked: bool, label: impl Into<String>) -> Self {
        View::Checkbox {
            is_checked,
            label: label.into(),
            on_toggle: None,
        }
    }
}

// Chaining methods for Input
impl<M: Clone + Debug> View<M> {
    /// Set input value
    pub fn value(mut self, val: impl Into<String>) -> Self {
        if let View::Input { value, .. } = &mut self {
            *value = val.into();
        }
        self
    }

    /// Set input change handler
    pub fn on_change(mut self, msg: M) -> Self {
        if let View::Input { on_change, .. } = &mut self {
            *on_change = Some(msg);
        }
        self
    }
}

// Chaining methods for Checkbox
impl<M: Clone + Debug> View<M> {
    /// Set checkbox toggle handler
    pub fn on_toggle(mut self, msg: M) -> Self {
        if let View::Checkbox { on_toggle, .. } = &mut self {
            *on_toggle = Some(msg);
        }
        self
    }
}
