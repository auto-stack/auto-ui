use autogui::app::Viewable;
use crate::spec::Spec;
use crate::dyna::state::{State, Dot};
use autolang::ast::Expr;
use autogui::widget::button::Button;
use gpui::*;

pub struct DynaView {
    state: State,
    builder: Option<Box<dyn Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static>>,
}

impl Viewable for DynaView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        let mut state = State::new();
        state.set_int("count", 0);
        Self {
            state,
            builder: None,
        }
    }
}

impl DynaView {
    pub fn update_spec(&mut self, spec: Spec) {
        spec.set_state(&mut self.state);

        self.builder = Some(Box::new(move |div, state, cx| {
            let spec_view = spec.get_view();
            let mut div = div;
            for (name, node) in spec_view.nodes.iter() {
                match name.text.as_str() {
                    "button" => {
                        let text_arg = &node.args[0];
                        if let Expr::Str(text) = text_arg {
                            div = div.child(Button::primary(text.as_str()));
                        }
                    },
                    "text" => {
                        let text_arg = &node.args[0];   
                        if let Expr::Str(text) = text_arg {
                            div = div.child(format!("{}", text.as_str()));
                        }
                    },
                    _ => ()
                };
            }
            div
        }));
    }

    pub fn contents(
        mut self,
        builder: impl Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static,
    ) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }
}

impl Render for DynaView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = div().flex().flex_col()
            .child(Button::primary("Refresh").on_click_mut(cx, |this, _ev, cx| {
                let mut spec = Spec::new();
                spec.read_file("counter.au");
                this.update_spec(spec);
                cx.notify();
            })).gap_2();

        if let Some(builder) = self.builder.as_ref() {
            builder(div, &mut self.state, cx)
        } else {
            div
        }
    }
}

