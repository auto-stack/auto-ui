use autolang::ast::Lambda;
use gpui::*;
use std::collections::HashMap;
pub enum Dot {
    Int(i32),
    Usize(usize),
    Str(String),
    Bool(bool),
}

impl Dot {
    pub fn dft_int() -> Self {
        Dot::Int(0)
    }

    pub fn dft_usize() -> Self {
        Dot::Usize(0)
    }

    pub fn dft_str() -> Self {
        Dot::Str(String::default())
    }

    pub fn dft_bool() -> Self {
        Dot::Bool(false)
    }
}

pub struct State {
    dots: HashMap<String, Dot>
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            dots: HashMap::default()
        }
    }
}

impl State {
    pub fn get_int(&self, key: &str) -> i32 {
        match self.dots.get(key) {
            Some(Dot::Int(v)) => *v,
            _ => 0,
        }
    }

    pub fn get_str(&self, key: &str) -> String {
        match self.dots.get(key) {
            Some(Dot::Str(v)) => v.clone(),
            _ => String::default(),
        }
    }

    pub fn get_usize(&self, key: &str) -> usize {
        match self.dots.get(key) {
            Some(Dot::Usize(v)) => *v,
            _ => 0,
        }
    }

    pub fn get_bool(&self, key: &str) -> bool {
        match self.dots.get(key) {
            Some(Dot::Bool(v)) => *v,
            _ => false,
        }
    }

    pub fn set_int(&mut self, key: &str, value: i32) {
        self.dots.insert(key.to_string(), Dot::Int(value));
    }

    pub fn set_str(&mut self, key: &str, value: String) {
        self.dots.insert(key.to_string(), Dot::Str(value));
    }

    pub fn set_usize(&mut self, key: &str, value: usize) {
        self.dots.insert(key.to_string(), Dot::Usize(value));
    }

    pub fn inc_int(&mut self, key: &str) {
        self.set_int(key, self.get_int(key) + 1);
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.dots.insert(key.to_string(), Dot::Bool(value));
    }
}
