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
        on_change: Option<M>,
        width: Option<u16>,
        password: bool,
    },

    /// Checkbox
    Checkbox {
        is_checked: bool,
        label: String,
        on_toggle: Option<M>,
    },

    /// Container wrapper for styling and layout
    Container {
        child: Box<View<M>>,
        padding: u16,
        width: Option<u16>,
        height: Option<u16>,
        center_x: bool,
        center_y: bool,
    },

    /// Scrollable container for content overflow
    Scrollable {
        child: Box<View<M>>,
        width: Option<u16>,
        height: Option<u16>,
    },

    /// Radio button for single selection from multiple options
    Radio {
        label: String,
        is_selected: bool,
        on_select: Option<M>,
    },

    /// Select dropdown for choosing from multiple options
    Select {
        options: Vec<String>,
        selected_index: Option<usize>,
        on_select: Option<M>,
    },

    /// List for displaying items in a vertical sequence
    List {
        items: Vec<View<M>>,
        spacing: u16,
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

    /// Create input field with placeholder
    pub fn input(placeholder: impl Into<String>) -> ViewInputBuilder<M> {
        ViewInputBuilder {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: None,
            width: None,
            password: false,
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

    /// Create radio button
    pub fn radio(is_selected: bool, label: impl Into<String>) -> Self {
        View::Radio {
            label: label.into(),
            is_selected,
            on_select: None,
        }
    }

    /// Create select dropdown with options
    pub fn select(options: Vec<String>) -> Self {
        View::Select {
            options,
            selected_index: None,
            on_select: None,
        }
    }

    /// Create a list with items
    pub fn list(items: Vec<View<M>>) -> ViewListBuilder<M> {
        ViewListBuilder {
            items,
            spacing: 0,
        }
    }

    /// Create a container with a child
    pub fn container(child: View<M>) -> ViewContainerBuilder<M> {
        ViewContainerBuilder {
            child,
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
        }
    }

    /// Create a scrollable container
    pub fn scrollable(child: View<M>) -> ViewScrollableBuilder<M> {
        ViewScrollableBuilder {
            child,
            width: None,
            height: None,
        }
    }
}

/// Builder for Scrollable with fluent API
pub struct ViewScrollableBuilder<M: Clone + Debug> {
    child: View<M>,
    width: Option<u16>,
    height: Option<u16>,
}

impl<M: Clone + Debug> ViewScrollableBuilder<M> {
    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Build the scrollable view
    pub fn build(self) -> View<M> {
        View::Scrollable {
            child: Box::new(self.child),
            width: self.width,
            height: self.height,
        }
    }
}

/// Builder for List with fluent API
pub struct ViewListBuilder<M: Clone + Debug> {
    items: Vec<View<M>>,
    spacing: u16,
}

impl<M: Clone + Debug> ViewListBuilder<M> {
    /// Set spacing between items
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Build the list view
    pub fn build(self) -> View<M> {
        View::List {
            items: self.items,
            spacing: self.spacing,
        }
    }
}

/// Builder for Input with fluent API
pub struct ViewInputBuilder<M: Clone + Debug> {
    placeholder: String,
    value: String,
    on_change: Option<M>,
    width: Option<u16>,
    password: bool,
}

impl<M: Clone + Debug> ViewInputBuilder<M> {
    /// Set input value
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self
    }

    /// Set input change handler
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }

    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Enable password mode (hides input)
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Build the input view
    pub fn build(self) -> View<M> {
        View::Input {
            placeholder: self.placeholder,
            value: self.value,
            on_change: self.on_change,
            width: self.width,
            password: self.password,
        }
    }
}

/// Builder for Container with fluent API
pub struct ViewContainerBuilder<M: Clone + Debug> {
    child: View<M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
}

impl<M: Clone + Debug> ViewContainerBuilder<M> {
    /// Set padding
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Center horizontally
    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
    }

    /// Center vertically
    pub fn center_y(mut self) -> Self {
        self.center_y = true;
        self
    }

    /// Center both horizontally and vertically
    pub fn center(mut self) -> Self {
        self.center_x = true;
        self.center_y = true;
        self
    }

    /// Build the container view
    pub fn build(self) -> View<M> {
        View::Container {
            child: Box::new(self.child),
            padding: self.padding,
            width: self.width,
            height: self.height,
            center_x: self.center_x,
            center_y: self.center_y,
        }
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

// Chaining methods for Radio
impl<M: Clone + Debug> View<M> {
    /// Set radio select handler
    pub fn on_select(mut self, msg: M) -> Self {
        if let View::Radio { on_select, .. } = &mut self {
            *on_select = Some(msg);
        }
        self
    }
}

// Chaining methods for Select
impl<M: Clone + Debug> View<M> {
    /// Set selected option by index
    pub fn selected(mut self, index: usize) -> Self {
        if let View::Select { selected_index, .. } = &mut self {
            *selected_index = Some(index);
        }
        self
    }

    /// Set select change handler
    pub fn on_choose(mut self, msg: M) -> Self {
        if let View::Select { on_select, .. } = &mut self {
            *on_select = Some(msg);
        }
        self
    }
}
