use gpui::*;
use std::collections::HashMap;
use autogui::app::Viewable;

pub enum SimpleState {
    Int(i32),
    Usize(usize),
    Str(SharedString),
}

impl SimpleState {
    pub fn dft_int() -> Self {
        SimpleState::Int(0)
    }

    pub fn dft_usize() -> Self {
        SimpleState::Usize(0)
    }

    pub fn dft_str() -> Self {
        SimpleState::Str(SharedString::default())
    }   
}

pub type State = HashMap<String, SimpleState>;

pub trait StateExt {
    fn get_int(&self, key: &str) -> i32;
    fn get_str(&self, key: &str) -> SharedString;   
    fn get_usize(&self, key: &str) -> usize;

    fn set_int(&mut self, key: &str, value: i32);
    fn set_str(&mut self, key: &str, value: SharedString);
    fn set_usize(&mut self, key: &str, value: usize);

    fn inc_int(&mut self, key: &str);
}

impl StateExt for State {
    fn get_int(&self, key: &str) -> i32 {
        match self.get(key) {
            Some(SimpleState::Int(v)) => *v,
            _ => 0
        }
    }

    fn get_str(&self, key: &str) -> SharedString {
        match self.get(key) {
            Some(SimpleState::Str(v)) => v.clone(),
            _ => SharedString::default()
        }
    }

    fn get_usize(&self, key: &str) -> usize {
        match self.get(key) {
            Some(SimpleState::Usize(v)) => *v,
            _ => 0
        }
    }

    fn set_int(&mut self, key: &str, value: i32) {
        self.insert(key.to_string(), SimpleState::Int(value));
    }

    fn set_str(&mut self, key: &str, value: SharedString) {
        self.insert(key.to_string(), SimpleState::Str(value));
    }

    fn set_usize(&mut self, key: &str, value: usize) {
        self.insert(key.to_string(), SimpleState::Usize(value));
    }

    fn inc_int(&mut self, key: &str) {
        self.set_int(key, self.get_int(key) + 1);
    }
}

pub struct AutoView {
    state: State,
    builder: Option<Box<dyn Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static>>,
}

impl Viewable for AutoView {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        let mut state = State::new();
        state.insert("count".into(), SimpleState::dft_int());
        Self {
            state,
            builder: None,
        }
    }
}

impl AutoView {
    pub fn contents(
        mut self,
        builder: impl Fn(Div, &mut State, &mut ViewContext<Self>) -> Div + 'static,
    ) -> Self {
        self.builder = Some(Box::new(builder));
        self
    }
}

impl Render for AutoView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let div = div().flex().flex_col();
        if let Some(builder) = self.builder.as_ref() {
            builder(div, &mut self.state, cx)
        } else {
            div
        }
    }
}
