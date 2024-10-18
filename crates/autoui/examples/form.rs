use autoui::app::SimpleApp;
use autoui::style::theme::ActiveTheme;
use autoui::widget::button::Button;
use autoui::widget::checkbox::Checkbox;
use autoui::widget::dropdown::Dropdown;
use autoui::widget::input::TextInput;
use autoui::widget::radio::{Radio, RadioGroup};
use autoui::widget::toolbar::*;
use autoui::widget::util::*;
use autoui::widget::workspace::Workspace;
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
    byte_order: ByteOrder,
    default_value: SharedString,

    input: View<TextInput>,
}

impl Render for CenterContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let label_width = 100.;
        div()
            .flex()
            .flex_col()
            .w_1_3()
            .items_center()
            .gap_4()
            .child(
                card("Signal Properties", cx)
                    .child(
                        row()
                            .child(div().w(Pixels(label_width)).child("Is Signed: "))
                            .child(
                                Checkbox::new("is_signed")
                                    .checked(self.is_signed)
                                    .on_click_mut(cx, |this, checked, cx| {
                                        this.is_signed = *checked;
                                    }),
                            ),
                    )
                    .child(
                        row()
                            .child(div().w(Pixels(label_width)).child("Byte Order: "))
                            .child(
                                RadioGroup::new("byte_order")
                                    .add(Radio::new("motorola").label("Motorola"))
                                    .add(Radio::new("intel").label("Intel"))
                                    .select(self.byte_order as usize)
                                    .on_click(cx.listener(|this, v: &usize, _cx| {
                                        this.byte_order = ByteOrder::from(*v);
                                    })),
                            ),
                    )
                    .child(
                        row()
                            .child(div().w(Pixels(label_width)).child("Byte Order: "))
                            .child(cx.new_view(|cx| {
                                Dropdown::new(
                                    SharedString::from("dropdown"),
                                    vec!["Motorola".into(), "Intel".into()],
                                    cx,
                                )
                            })),
                    ),
            )
            .child(
                card("Section_2", cx)
                    .child("World")
                    .child(div().flex().flex_row().w_full().child(self.input.clone()))
                    .child(Button::button("Don't click me")),
            )
    }
}

impl RootView {
    fn new(cx: &mut WindowContext) -> Self {
        let toolbar = cx.new_view(|_cx| Toolbar {});
        let center = cx.new_view(|cx| CenterContent {
            name: "ACC_Kilometre_Mile".into(),
            message: "ICM_5".into(),
            is_signed: false,
            byte_order: ByteOrder::Motorola,
            default_value: "0".into(),
            input: cx.new_view(|cx| TextInput::new(cx)),
        });
        let workspace = Workspace::new().toolbar(toolbar).child(center);

        Self {
            workspace: cx.new_view(|_cx| workspace),
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
