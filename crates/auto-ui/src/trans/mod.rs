// Auto UI Transpiler
//
// This module provides transpilation from Auto language .at files to auto-ui Rust code.
// It parses Auto UI widget definitions and generates Component implementations.

use auto_lang::ast::Code;
use std::collections::HashSet;
use std::io::Write;

pub mod auto_ui_trans;
pub mod rust_gen;
pub mod api;

pub use auto_ui_trans::AutoUITrans;
pub use rust_gen::RustCodeGenerator;

/// High-level transpilation API
#[cfg(feature = "transpiler")]
pub use api::{transpile_file, transpile_ast};

/// Sink for collecting generated Rust code
pub struct CodeSink {
    pub imports: HashSet<String>,
    pub body: Vec<u8>,
    pub indent: usize,
}

impl CodeSink {
    pub fn new() -> Self {
        Self {
            imports: HashSet::new(),
            body: Vec::new(),
            indent: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    pub fn print_indent(&mut self) {
        for _ in 0..self.indent {
            self.body.write(b"    ").unwrap();
        }
    }

    pub fn write(&mut self, s: &str) {
        self.body.write(s.as_bytes()).unwrap();
    }

    pub fn writeln(&mut self, s: &str) {
        self.print_indent();
        self.write(s);
        self.body.write(b"\n").unwrap();
    }

    pub fn add_import(&mut self, import: &str) {
        self.imports.insert(import.to_string());
    }

    pub fn done(&mut self) -> String {
        let mut result = String::new();

        // Add imports
        if !self.imports.is_empty() {
            let mut imports: Vec<_> = self.imports.iter().cloned().collect();
            imports.sort();
            for import in imports {
                result.push_str(&format!("use {};\n", import));
            }
            result.push('\n');
        }

        // Add body
        result.push_str(&String::from_utf8_lossy(&self.body));
        result
    }
}

/// Trans trait for auto-ui specific transpilation
pub trait Trans {
    fn trans(&mut self, ast: Code, sink: &mut CodeSink) -> Result<(), String>;
}
