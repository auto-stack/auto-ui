// Auto UI Transpiler CLI
//
// Transpiles Auto language .at files to auto-ui Rust code

use anyhow::Result;
use clap::Parser;
use std::path::Path;

/// Auto UI Transpiler - Convert .at files to auto-ui Rust code
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input .at file path
    #[arg(short, long)]
    input: String,

    /// Output .rs file path (optional, prints to stdout if not provided)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Auto UI Transpiler");
    println!("==================");
    println!("Input: {}", args.input);
    println!();

    // Validate input file exists
    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", args.input);
    }

    // Transpile the file
    let rust_code = auto_ui::trans::AutoUITrans::transpile_file(&args.input)
        .map_err(|e| anyhow::anyhow!("Transpilation failed: {}", e))?;

    // Output result
    if let Some(output_path) = args.output {
        // Write to file
        std::fs::write(&output_path, rust_code)
            .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;
        println!("✓ Generated: {}", output_path);
    } else {
        // Print to stdout
        println!("// Generated Rust Code");
        println!("=====================");
        println!();
        println!("{}", rust_code);
    }

    Ok(())
}
