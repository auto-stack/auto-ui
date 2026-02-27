//! AURA Counter Example - Demonstrates Plan 096 AURA pipeline
//!
//! Run with: cargo run --example aura_counter --features transpiler

use auto_ui::trans::{transpile_aura, transpile_vue_aura};

fn main() {
    // Read the Counter widget source
    let source = r#"
widget Counter {
    msg Msg { Inc, Dec, Reset }

    model {
        count int = 0
    }

    view {
        col {
            h2 > "Counter: ${.count}"
            row {
                button "-" { onclick: .Dec }
                button "+" { onclick: .Inc }
            }
            button "Reset" { onclick: .Reset }
        }
    }

    on {
        .Inc => {
            .count += 1
        }
        .Dec => {
            .count -= 1
        }
        .Reset => {
            .count = 0
        }
    }
}
"#;

    println!("=== AURA Pipeline Demo ===\n");

    // Generate Rust code
    println!("--- Generated Rust Code ---\n");
    match transpile_aura(source, None) {
        Ok(rust_code) => {
            println!("{}", rust_code);
        }
        Err(e) => {
            eprintln!("Error generating Rust: {}", e);
        }
    }

    println!("\n--- Generated Vue3 SFC ---\n");
    match transpile_vue_aura(source, None) {
        Ok(vue_code) => {
            println!("{}", vue_code);
        }
        Err(e) => {
            eprintln!("Error generating Vue: {}", e);
        }
    }
}
