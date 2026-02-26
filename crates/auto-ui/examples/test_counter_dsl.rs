// Test counter_new.at DSL preprocessing and transpilation
// Run with: cargo run -p auto-ui --features transpiler --example test_counter_dsl

fn main() {
    let source = include_str!("../../../scratch/counter_new.at");

    // First, show preprocessed output
    let preprocessed = auto_ui::trans::dsl_preprocess::dsl_preprocess(source);

    println!("=== Preprocessed Auto Code ===\n");
    println!("{}", preprocessed);
    println!("\n=== End Preprocessed ===\n");

    // Then try to transpile
    match auto_ui::trans::transpile_dsl(source, None) {
        Ok(code) => {
            println!("=== Generated Rust Code ===\n");
            println!("{}", code);
        }
        Err(e) => {
            eprintln!("Transpilation failed: {}", e);
            std::process::exit(1);
        }
    }
}
