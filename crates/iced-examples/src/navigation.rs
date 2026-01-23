use iced::{Element};
use iced::widget::{container, column, text, button, Button};

/// 导航消息
#[derive(Clone, Debug)]
pub enum NavigationMessage {
    PageSelected(crate::gallery::Page),
    GroupToggled(String),
}

/// 侧边栏
pub struct Sidebar {
    pub page_groups: Vec<crate::gallery::PageGroup>,
    pub display_mode: crate::gallery::DisplayMode,
}

impl Sidebar {
    pub fn new(
        page_groups: Vec<crate::gallery::PageGroup>,
        display_mode: crate::gallery::DisplayMode,
    ) -> Self {
        Self {
            page_groups,
            display_mode,
        }
    }

    pub fn view(&self) -> Element<'static, NavigationMessage> {
        // 标题
        let title = if self.display_mode == crate::gallery::DisplayMode::Full {
            text("Iced Gallery").size(24)
        } else {
            text("📱").size(24)
        };

        // Group 0: Getting Started
        let g0_label = if self.display_mode == crate::gallery::DisplayMode::Full {
            format!("{} {}", self.page_groups[0].icon, self.page_groups[0].label)
        } else {
            self.page_groups[0].icon.to_string()
        };
        let g0_toggle = button(text(g0_label.clone()))
            .on_press(NavigationMessage::GroupToggled(self.page_groups[0].label.clone()));

        // Group 1: Basic Widgets
        let g1_label = if self.display_mode == crate::gallery::DisplayMode::Full {
            format!("{} {}", self.page_groups[1].icon, self.page_groups[1].label)
        } else {
            self.page_groups[1].icon.to_string()
        };
        let g1_toggle = button(text(g1_label.clone()))
            .on_press(NavigationMessage::GroupToggled(self.page_groups[1].label.clone()));

        // Group 2: Forms & Input
        let g2_label = if self.display_mode == crate::gallery::DisplayMode::Full {
            format!("{} {}", self.page_groups[2].icon, self.page_groups[2].label)
        } else {
            self.page_groups[2].icon.to_string()
        };
        let g2_toggle = button(text(g2_label.clone()))
            .on_press(NavigationMessage::GroupToggled(self.page_groups[2].label.clone()));

        // Group 3: Layout & Style
        let g3_label = if self.display_mode == crate::gallery::DisplayMode::Full {
            format!("{} {}", self.page_groups[3].icon, self.page_groups[3].label)
        } else {
            self.page_groups[3].icon.to_string()
        };
        let g3_toggle = button(text(g3_label.clone()))
            .on_press(NavigationMessage::GroupToggled(self.page_groups[3].label.clone()));

        container(
            column!(
                title,
                text(""),
                g0_toggle,
                self.build_page_items(&self.page_groups[0]),
                g1_toggle,
                self.build_page_items(&self.page_groups[1]),
                g2_toggle,
                self.build_page_items(&self.page_groups[2]),
                g3_toggle,
                self.build_page_items(&self.page_groups[3]),
            )
            .spacing(4)
            .padding(10)
        )
        .into()
    }

    fn build_page_items(&self, group: &crate::gallery::PageGroup) -> Element<'static, NavigationMessage> {
        if !group.expanded {
            return text("").into();
        }

        match group.page_items.len() {
            0 => text("").into(),
            1 => {
                let btn = self.build_item_button(&group.page_items[0]);
                column!(btn).spacing(2).into()
            }
            2 => {
                let btn1 = self.build_item_button(&group.page_items[0]);
                let btn2 = self.build_item_button(&group.page_items[1]);
                column!(btn1, btn2).spacing(2).into()
            }
            3 => {
                let btn1 = self.build_item_button(&group.page_items[0]);
                let btn2 = self.build_item_button(&group.page_items[1]);
                let btn3 = self.build_item_button(&group.page_items[2]);
                column!(btn1, btn2, btn3).spacing(2).into()
            }
            4 => {
                let btn1 = self.build_item_button(&group.page_items[0]);
                let btn2 = self.build_item_button(&group.page_items[1]);
                let btn3 = self.build_item_button(&group.page_items[2]);
                let btn4 = self.build_item_button(&group.page_items[3]);
                column!(btn1, btn2, btn3, btn4).spacing(2).into()
            }
            5 => {
                let btn1 = self.build_item_button(&group.page_items[0]);
                let btn2 = self.build_item_button(&group.page_items[1]);
                let btn3 = self.build_item_button(&group.page_items[2]);
                let btn4 = self.build_item_button(&group.page_items[3]);
                let btn5 = self.build_item_button(&group.page_items[4]);
                column!(btn1, btn2, btn3, btn4, btn5).spacing(2).into()
            }
            _ => text("").into()
        }
    }

    fn build_item_button(&self, item: &crate::gallery::PageItem) -> Button<'static, NavigationMessage> {
        let item_label = if self.display_mode == crate::gallery::DisplayMode::Full {
            format!("  → {}", item.label)
        } else {
            "  •".to_string()
        };

        button(text(item_label))
            .on_press(NavigationMessage::PageSelected(item.page.clone()))
    }
}
