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

pub type State = HashMap<String, Dot>;

pub trait StateExt {
    fn get_int(&self, key: &str) -> i32;
    fn get_str(&self, key: &str) -> String;
    fn get_usize(&self, key: &str) -> usize;
    fn get_bool(&self, key: &str) -> bool;

    fn set_int(&mut self, key: &str, value: i32);
    fn set_str(&mut self, key: &str, value: String);
    fn set_usize(&mut self, key: &str, value: usize);
    fn set_bool(&mut self, key: &str, value: bool);

    fn inc_int(&mut self, key: &str);
}

impl StateExt for State {
    fn get_int(&self, key: &str) -> i32 {
        match self.get(key) {
            Some(Dot::Int(v)) => *v,
            _ => 0,
        }
    }

    fn get_str(&self, key: &str) -> String {
        match self.get(key) {
            Some(Dot::Str(v)) => v.clone(),
            _ => String::default(),
        }
    }

    fn get_usize(&self, key: &str) -> usize {
        match self.get(key) {
            Some(Dot::Usize(v)) => *v,
            _ => 0,
        }
    }

    fn get_bool(&self, key: &str) -> bool {
        match self.get(key) {
            Some(Dot::Bool(v)) => *v,
            _ => false,
        }
    }

    fn set_int(&mut self, key: &str, value: i32) {
        self.insert(key.to_string(), Dot::Int(value));
    }

    fn set_str(&mut self, key: &str, value: String) {
        self.insert(key.to_string(), Dot::Str(value));
    }

    fn set_usize(&mut self, key: &str, value: usize) {
        self.insert(key.to_string(), Dot::Usize(value));
    }

    fn inc_int(&mut self, key: &str) {
        self.set_int(key, self.get_int(key) + 1);
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        self.insert(key.to_string(), Dot::Bool(value));
    }
}
