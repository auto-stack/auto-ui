use autogui::app::Viewable;
use crate::spec::Spec;
use autolang::ast::{Expr, Key, Name};
use crate::dyna::state::{State, Dot};
use autoval::value::Value;
use autogui::widget::button::Button;
use autogui::widget::table::{ColConfig, Row, Table};
use gpui::*;

pub struct DynaView {
    spec: Spec,
    // state: State,
    builder: Option<Box<dyn Fn(Div, &mut Spec, &mut ViewContext<Self>) -> Div + 'static>>,
}

impl Viewable for DynaView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        // let mut state = State::new();
        // state.set_int("count", 0);
        Self {
            spec: Spec::new(),
            // state,
            builder: None,
        }
    }
}

impl DynaView {

    pub fn reload(&mut self) {
        self.spec.reload();
        self.update_spec();
    }

    pub fn set_path(&mut self, path: &str) {
        self.spec.read_file(path);
    }

    pub fn update_spec(&mut self) {
        // self.spec.set_state(&mut self.state);

        self.builder = Some(Box::new(move |div, spec, cx| {
            let mut div = div;

            let spec_view = spec.get_view();
            for (name, node) in spec_view.nodes.iter() {
                let name = name.text.clone();
                let node = node.clone();
                match name.as_str() {
                    "button" => {
                        let text_arg = node.args.get(0);
                        if let Some(Expr::Str(text)) = text_arg {
                            let mut button = Button::primary(text.as_str());

                            let onclick = node.props.get(&Key::NamedKey(Name::new("onclick".to_string())));
                            match onclick {
                                Some(Expr::Lambda(lambda)) => {
                                    let lambda = lambda.clone();
                                    button = button.on_click_mut(cx, move |this, _ev, cx| {
                                        this.spec.run_lambda(lambda.clone());
                                        cx.notify();
                                    });
                                }
                                _ => (),
                            }
                            div = div.child(button);
                        }
                    },
                    "text" => {
                        let text_arg = node.args.get(0);
                        if let Some(str) = text_arg {
                            match str {
                                Expr::Str(text) => {
                                    div = div.child(format!("{}", text));
                                }
                                Expr::FStr(fstr) => {
                                    let mut sb = String::new();
                                    for p in fstr.parts.iter() {
                                        match p {
                                            Expr::Str(text) => sb.push_str(&format!("{}", text)),
                                            Expr::Ident(_) => {
                                                let val = spec.eval_ident(p);
                                                sb.push_str(&format!("{}", val));
                                            }
                                            _ => (),
                                        }
                                    }
                                    div = div.child(sb);
                                }
                                _ => (),
                            }
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
        builder: impl Fn(Div, &mut Spec, &mut ViewContext<Self>) -> Div + 'static,
    ) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }
}

impl Render for DynaView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = div().flex().flex_col()
            .child(Button::primary("Refresh").on_click_mut(cx, |this, _ev, cx| {
                println!("reload");
                this.reload();
                cx.notify();
            })).gap_2();

        if let Some(builder) = self.builder.as_ref() {
            builder(div, &mut self.spec, cx)
        } else {
            div
        }
    }
}

