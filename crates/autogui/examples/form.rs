use autogui::app::SimpleApp;
use autogui::style::theme::ActiveTheme;
use autogui::widget::button::Button;
use autogui::widget::checkbox::Checkbox;
use autogui::widget::dropdown::Dropdown;
use autogui::widget::input::TextInput;
use autogui::widget::radio::{Radio, RadioGroup};
use autogui::widget::toolbar::*;
use autogui::widget::util::*;
use autogui::widget::workspace::Workspace;
use gpui::*;
use std::fmt::Display;

struct RootView {
    workspace: View<Workspace>,
}

#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    Motorola = 0,
    Intel = 1,
}

impl From<usize> for ByteOrder {
    fn from(value: usize) -> Self {
        match value {
            0 => ByteOrder::Motorola,
            1 => ByteOrder::Intel,
            _ => panic!("Invalid byte order value"),
        }
    }
}

impl Display for ByteOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct CenterContent {
    name: SharedString,
    message: SharedString,
    is_signed: bool,
    fullcan: usize,
    byte_order: ByteOrder,
    default_value: SharedString,

    byte_order_dropdown: View<Dropdown>,
    input: View<TextInput>,
    default_value_input: View<TextInput>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_1_2()
            .items_center()
            .gap_4()
            .child(
                card("Signal Properties", cx)
                    .child(field("Name: ", self.name.clone()))
                    .child(field("Message: ", self.message.clone()))
                    .child(field("Is Signed: ", Checkbox::new("is_signed")
                        .checked(self.is_signed)
                        .on_click_mut(cx, |this, checked, _cx| {
                            this.is_signed = *checked;
                        }),
                    ))
                    .child(field("FullCAN: ", RadioGroup::new("fullcan")
                        .add(Radio::new("normal").label("Normal"))
                        .add(Radio::new("fullcan").label("FullCAN"))
                        .select(self.fullcan)
                        .on_click(cx.listener(|this, v: &usize, _cx| {
                            this.fullcan = *v;
                        })),
                    ))
                    .child(field("Byte Order: ", self.byte_order_dropdown.clone()))
                    .child(field("Init Value: ", self.default_value_input.clone()))
            )
    }
}

impl RootView {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        let center = cx.new_view(|cx| CenterContent {
            name: "ACC_Kilometre_Mile".into(),
            message: "ICM_5".into(),
            is_signed: false,
            fullcan: 0,
            byte_order: ByteOrder::Motorola,
            default_value: "0".into(),
            byte_order_dropdown: cx.new_view(|cx| Dropdown::new(
                SharedString::from("Byte Order"),
                vec!["Motorola".into(), "Intel".into()],
                Some(1),
                cx,
            )),
            input: cx.new_view(|cx| TextInput::new(cx)),
            default_value_input: cx.new_view(|cx| TextInput::new(cx)),
        });
        let workspace = cx.new_view(|cx| Workspace::new(cx).child(center));

        Self {
            workspace,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.active_theme();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.workspace.clone())
    }
}

fn main() {
    SimpleApp::new().run(false, |cx| cx.new_view(|cx| RootView::new(cx)));
}
