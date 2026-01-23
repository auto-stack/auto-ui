// Iced Gallery - 统一的 Iced 控件展示应用
//
// 这是 iced-examples 的默认入口，展示 Gallery 界面
// 独立的示例仍然可以通过 cargo run --bin <name> 运行

mod gallery;
mod navigation;
mod page;

use iced::widget::{center, container, column, text, row};
use gallery::Gallery;
use navigation::{Sidebar, NavigationMessage};

fn main() -> iced::Result {
    iced::run(GalleryApp::update, GalleryApp::view)
}

#[derive(Default)]
struct GalleryApp {
    gallery: Gallery,
}

impl GalleryApp {
    fn update(&mut self, message: Message) {
        match message {
            Message::Navigation(nav_msg) => {
                match nav_msg {
                    NavigationMessage::PageSelected(page) => {
                        self.gallery.navigate_to(page);
                    }
                    NavigationMessage::GroupToggled(label) => {
                        self.gallery.toggle_group(&label);
                    }
                }
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let sidebar = Sidebar::new(
            self.gallery.page_groups.clone(),
            self.gallery.side_nav_display_mode
        );

        let sidebar_element = sidebar.view()
            .map(Message::Navigation);

        let content = match &self.gallery.current_page {
            gallery::Page::Home => page::home::view(),
            gallery::Page::Button => page::button::view(),
            gallery::Page::Checkbox => page::checkbox::view(),
            gallery::Page::Counter => page::counter::view(),
            gallery::Page::Select => page::select::view(),
            gallery::Page::Dropdown => page::dropdown::view(),
            gallery::Page::Slider => page::slider::view(),
            gallery::Page::Progress => page::progress::view(),
            gallery::Page::Todos => page::todos::view(),
            gallery::Page::Layout => page::layout::view(),
            gallery::Page::Circle => page::circle::view(),
            gallery::Page::Hello => page::hello::view(),
        };

        container(
            row!(
                sidebar_element,
                text("  "),
                content
            )
        )
        .padding(10)
        .into()
    }
}

#[derive(Clone, Debug)]
enum Message {
    Navigation(NavigationMessage),
}
