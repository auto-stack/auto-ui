use iced::Size;

/// 页面枚举
#[derive(Clone, Debug)]
pub enum Page {
    Home,
    Button,
    Checkbox,
    Counter,
    Select,
    Dropdown,
    Slider,
    Progress,
    Todos,
    Layout,
    Circle,
    Hello,
}

impl Default for Page {
    fn default() -> Self {
        Page::Home
    }
}

/// 页面项
#[derive(Clone, Debug)]
pub struct PageItem {
    pub label: String,
    pub page: Page,
}

impl PageItem {
    pub fn new(label: impl Into<String>, page: Page) -> Self {
        Self {
            label: label.into(),
            page,
        }
    }
}

/// 页面分组
#[derive(Clone, Debug)]
pub struct PageGroup {
    pub icon: char,
    pub label: String,
    pub page_items: Vec<PageItem>,
    pub expanded: bool,
}

impl PageGroup {
    pub fn new(icon: char, label: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            page_items: Vec::new(),
            expanded: false,
        }
    }

    pub fn with_items(mut self, items: Vec<PageItem>) -> Self {
        self.page_items = items;
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// 显示模式
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayMode {
    Compact,  // < 1000px 宽度，仅显示图标
    Full,     // >= 1000px 宽度，显示图标+文本
}

/// Gallery 应用状态
#[derive(Debug)]
pub struct Gallery {
    pub current_page: Page,
    pub side_nav_display_mode: DisplayMode,
    pub window_size: Size,
    pub page_groups: Vec<PageGroup>,
}

impl Gallery {
    pub fn new() -> Self {
        let page_groups = Self::create_page_groups();

        Self {
            current_page: Page::default(),
            side_nav_display_mode: DisplayMode::Full,
            window_size: Size::new(1200.0, 800.0),
            page_groups,
        }
    }

    fn create_page_groups() -> Vec<PageGroup> {
        vec![
            PageGroup::new('🏠', "Getting Started")
                .with_items(vec![
                    PageItem::new("Home", Page::Home),
                    PageItem::new("Hello", Page::Hello),
                ])
                .with_expanded(true),
            PageGroup::new('📦', "Basic Widgets")
                .with_items(vec![
                    PageItem::new("Button", Page::Button),
                    PageItem::new("Checkbox", Page::Checkbox),
                    PageItem::new("Counter", Page::Counter),
                    PageItem::new("Slider", Page::Slider),
                    PageItem::new("Progress", Page::Progress),
                ])
                .with_expanded(false),
            PageGroup::new('📝', "Forms & Input")
                .with_items(vec![
                    PageItem::new("Select", Page::Select),
                    PageItem::new("Dropdown", Page::Dropdown),
                    PageItem::new("Todos", Page::Todos),
                ])
                .with_expanded(false),
            PageGroup::new('🎨', "Layout & Style")
                .with_items(vec![
                    PageItem::new("Layout", Page::Layout),
                    PageItem::new("Circle", Page::Circle),
                ])
                .with_expanded(false),
        ]
    }

    pub fn update_display_mode(&mut self) {
        self.side_nav_display_mode = if self.window_size.width < 1000.0 {
            DisplayMode::Compact
        } else {
            DisplayMode::Full
        };
    }

    pub fn navigate_to(&mut self, page: Page) {
        self.current_page = page;
    }

    pub fn toggle_group(&mut self, group_label: &str) {
        if let Some(group) = self.page_groups.iter_mut().find(|g| g.label == group_label) {
            group.expanded = !group.expanded;
        }
    }
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}
