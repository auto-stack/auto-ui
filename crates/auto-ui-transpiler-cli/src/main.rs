// AutoUI Transpiler CLI
//
// Command-line tool to transpile Auto language (.at) files to AutoUI Rust code.
//
// Usage:
//   auto-ui-transpile input.at [output.rs]
//   auto-ui-transpile input.at --stdout
//   auto-ui-transpile --watch input.at

use anyhow::Result;
use auto_ui::trans::transpile_file;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "auto-ui-transpile")]
#[command(about = "Transpile Auto language (.at) files to AutoUI Rust code", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile a single .at file to Rust code
    Transpile {
        /// Input .at file to transpile
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output Rust file (writes to stdout if not specified)
        #[arg(value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Print generated code to stdout instead of writing to file
        #[arg(short, long)]
        stdout: bool,
    },
    /// Show information about the .at file without transpiling
    Info {
        /// Input .at file to analyze
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommand mode
    match cli.command {
        Some(Commands::Transpile { input, output, stdout }) => {
            transpile(&input, output.as_ref(), stdout)
        }
        Some(Commands::Info { input }) => {
            show_info(&input)
        }
        None => {
            // No subcommand, show help
            println!("{}", Cli::command().render_long_help());
            Ok(())
        }
    }
}

/// Transpile a single .at file
fn transpile(input: &PathBuf, output: Option<&PathBuf>, stdout: bool) -> Result<()> {
    // Check input file exists
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    println!("🔄 Transpiling {}...", input.display());

    // Transpile the file
    let rust_code = transpile_file(input, None)
        .map_err(|e| anyhow::anyhow!("Failed to transpile file: {}", e))?;

    // Output the result
    if stdout {
        println!("\n// Generated Rust Code:\n");
        println!("{}", rust_code);
    } else if let Some(output_path) = output {
        // Write to output file
        std::fs::write(output_path, &rust_code)
            .map_err(|e| anyhow::anyhow!("Failed to write output file: {}: {}", output_path.display(), e))?;
        println!("✅ Generated Rust code: {}", output_path.display());
    } else {
        // Default: print to stdout
        println!("{}", rust_code);
    }

    Ok(())
}

/// Show information about an .at file
fn show_info(input: &PathBuf) -> Result<()> {
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    println!("📄 File: {}", input.display());
    println!("   Size: {} bytes", std::fs::metadata(input)?.len());

    // Read and count lines
    let content = std::fs::read_to_string(input)?;
    let lines: Vec<&str> = content.lines().collect();
    println!("   Lines: {}", lines.len());

    // Basic analysis
    println!("\n🔍 Quick Analysis:");
    let has_widget = content.contains("widget ");
    let has_app = content.contains("app ");
    let has_view = content.contains("fn view()");

    if has_widget {
        println!("   ✅ Contains widget definition");
    } else {
        println!("   ⚠️  No widget definition found");
    }

    if has_app {
        println!("   ✅ Contains app definition");
    }

    if has_view {
        println!("   ✅ Contains view() method");
    }

    Ok(())
}
