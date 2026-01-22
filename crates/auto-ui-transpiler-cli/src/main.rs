// AutoUI Transpiler CLI
//
// Command-line tool to transpile Auto language (.at) files to AutoUI Rust code.
// Supports three modes:
// - Single file transpilation
// - Batch directory processing
// - Watch mode for auto-recompilation
//
// Usage:
//   auto-ui-transpile file input.at [output.rs]
//   auto-ui-transpile batch --input ./src --output ./gen
//   auto-ui-transpile watch --input ./src --output ./gen

use anyhow::Result;
use auto_ui::trans::transpile_file;
use clap::{Parser, Subcommand};
use console::style;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::fs;

#[derive(Parser)]
#[command(name = "auto-ui-transpile")]
#[command(about = "Transpile Auto language (.at) files to AutoUI Rust code", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile a single .at file to Rust code
    File {
        /// Input .at file to transpile
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output Rust file (writes to stdout if not specified)
        #[arg(value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Check syntax without writing output
        #[arg(long)]
        check: bool,
    },

    /// Transpile all .at files in a directory
    Batch {
        /// Input directory containing .at files
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for .rs files
        #[arg(short, long)]
        output: PathBuf,

        /// Number of parallel jobs (default: number of CPU cores)
        #[arg(short, long, default_value_t = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4))]
        jobs: usize,

        /// Preserve directory structure
        #[arg(long)]
        preserve_structure: bool,
    },

    /// Watch a directory and transpile on file changes
    Watch {
        /// Directory to watch
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for .rs files
        #[arg(short, long)]
        output: PathBuf,

        /// Verbose output (print all events)
        #[arg(long, short)]
        verbose: bool,
    },

    /// Show information about the .at file without transpiling
    Info {
        /// Input .at file to analyze
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// Run an .at file directly (transpile, build, and run)
    Run {
        /// Input .at file to run
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Backend to use (gpui or iced)
        #[arg(short, long, default_value = "gpui")]
        backend: String,

        /// Keep temporary files after running
        #[arg(long)]
        keep_temp: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::File { input, output, check } => {
            run_file(&input, output.as_ref(), check)
        }
        Commands::Batch { input, output, jobs, preserve_structure } => {
            run_batch(&input, &output, jobs, preserve_structure)
        }
        Commands::Watch { input, output, verbose } => {
            run_watch(&input, &output, verbose)
        }
        Commands::Info { input } => {
            show_info(&input)
        }
        Commands::Run { input, backend, keep_temp } => {
            run_run(&input, &backend, keep_temp)
        }
    }
}

/// Transpile a single .at file
fn run_file(input: &PathBuf, output: Option<&PathBuf>, check: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler").cyan().bold());
    println!("{}", style("==================").cyan().bold());
    println!("Input: {}", style(input.display()));
    if let Some(out) = output {
        println!("Output: {}", style(out.display()));
    }
    println!();

    // Check input file exists
    if !input.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input file not found").red(), input.display()));
    }

    // Start timing
    let start = Instant::now();

    // Transpile the file
    let rust_code = transpile_file(input, None)
        .map_err(|e| anyhow::anyhow!("{}: {}", style("Transpilation failed").red(), e))?;

    let elapsed = start.elapsed();

    if check {
        println!("{} {} {:?}", style("✓").green(), style("Syntax check passed").green(), elapsed);
        return Ok(());
    }

    // Output the result
    if let Some(output_path) = output {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to output file
        fs::write(output_path, &rust_code)
            .map_err(|e| anyhow::anyhow!("{}: {}", style("Failed to write output file").red(), e))?;
        println!("{} {} {}", style("✓").green(), style("Generated").green(), style(output_path.display()));
        println!("  Time: {:?}", elapsed);
    } else {
        // Print to stdout
        println!("{}", style("// Generated Rust Code").dim());
        println!("{}", style("=====================").dim());
        println!();
        println!("{}", rust_code);
    }

    Ok(())
}

/// Transpile all .at files in a directory
fn run_batch(input: &PathBuf, output: &PathBuf, jobs: usize, preserve_structure: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler - Batch Mode").cyan().bold());
    println!("{}", style("===============================").cyan().bold());
    println!("Input: {}", style(input.display()));
    println!("Output: {}", style(output.display()));
    println!("Jobs: {}", style(jobs));
    println!();

    // Check input directory exists
    if !input.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input directory not found").red(), input.display()));
    }

    // Collect all .at files
    let at_files = collect_at_files(input)?;
    let total_files = at_files.len();

    if total_files == 0 {
        println!("{} No .at files found in {}", style("ℹ").yellow(), input.display());
        return Ok(());
    }

    println!("{} Found {} .at file(s)", style("✓").green(), style(total_files));
    println!();

    // Create output directory
    fs::create_dir_all(output)?;

    // Process files
    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for (index, input_file) in at_files.iter().enumerate() {
        let relative_path = input_file.strip_prefix(input)?;
        let output_file = if preserve_structure {
            output.join(relative_path).with_extension("rs")
        } else {
            output.join(input_file.file_name().unwrap()).with_extension("rs")
        };

        print!("[{}/{}] ", style(index + 1).dim(), style(total_files).dim());
        print!("{}", style(input_file.display()));

        // Transpile the file
        match transpile_file(input_file, None) {
            Ok(rust_code) => {
                // Ensure parent directory exists
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Write output
                match fs::write(&output_file, rust_code) {
                    Ok(_) => {
                        println!(" {}", style("✓").green());
                        success_count += 1;
                    }
                    Err(e) => {
                        println!(" {}", style("✗").red());
                        println!("  {} Failed to write: {}", style("Error:").red(), e);
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                println!(" {}", style("✗").red());
                println!("  {} {}", style("Error:").red(), e);
                error_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    println!();
    println!("{}", style("─".repeat(40)).dim());
    println!("{} {}", style("Summary:").cyan().bold(), style(format!("{} succeeded, {} failed", success_count, error_count)));
    println!("{} {:?}", style("Time:").cyan().bold(), elapsed);

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Watch directory and transpile on changes
fn run_watch(input: &PathBuf, output: &PathBuf, verbose: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler - Watch Mode").cyan().bold());
    println!("{}", style("==============================").cyan().bold());
    println!("Watching: {}", style(input.display()));
    println!("Output: {}", style(output.display()));
    println!();
    println!("{}", style("Press Ctrl+C to stop...").dim());
    println!();

    // Check input directory exists
    if !input.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input directory not found").red(), input.display()));
    }

    // Create output directory
    fs::create_dir_all(output)?;

    // Initial transpilation
    println!("{}", style("Running initial transpilation...").dim());
    run_batch(input, output, 1, true)?;
    println!();

    // Setup file watcher
    use notify::RecursiveMode;
    use notify_debouncer_mini::new_debouncer;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(std::time::Duration::from_millis(250), tx)?;

    // Access the underlying watcher and configure it
    debouncer.watcher().watch(input, RecursiveMode::Recursive)?;

    println!("{} Watching for changes...", style("✓").green());
    println!();

    // Process events
    loop {
        match rx.recv() {
            Ok(result) => {
                if let Ok(events) = result {
                    for event in events {
                        if verbose {
                            println!("{:?}", event);
                        }

                        let path = &event.path;
                        if path.extension().and_then(|s| s.to_str()) == Some("at") {
                            let relative_path = path.strip_prefix(input).unwrap_or(path);
                            let output_file = output.join(relative_path).with_extension("rs");

                            println!("{} {}", style("Change detected:").yellow(), style(path.display()));

                            // Transpile the file
                            match transpile_file(path, None) {
                                Ok(rust_code) => {
                                    // Ensure parent directory exists
                                    if let Some(parent) = output_file.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }

                                    // Write output
                                    match fs::write(&output_file, rust_code) {
                                        Ok(_) => {
                                            println!("{} {}", style("✓ Transpiled:").green(), style(output_file.display()));
                                        }
                                        Err(e) => {
                                            println!("{} Failed to write: {}", style("Error:").red(), e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{} {}", style("Error:").red(), e);
                                }
                            }
                            println!();
                        }
                    }
                }
            }
            Err(e) => {
                println!("{} Watch error: {:?}", style("Error:").red(), e);
                break;
            }
        }
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

/// Recursively collect all .at files in a directory
fn collect_at_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            files.extend(collect_at_files(&path)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("at") {
            files.push(path);
        }
    }

    files.sort();

    Ok(files)
}

/// Run an .at file directly (transpile, build, and run)
fn run_run(input: &PathBuf, backend: &str, keep_temp: bool) -> Result<()> {
    println!("{}", style("AutoUI Run - One Command to Run").cyan().bold());
    println!("{}", style("===============================").cyan().bold());
    println!("Input: {}", style(input.display()));
    println!("Backend: {}", style(backend));
    println!();

    // Validate backend
    if backend != "gpui" && backend != "iced" {
        return Err(anyhow::anyhow!(
            "{}: {}",
            style("Invalid backend").red(),
            "supported backends: gpui, iced"
        ));
    }

    // Check input file exists
    if !input.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input file not found").red(), input.display()));
    }

    // Extract component name from filename
    let component_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Component");

    println!("{} Transpiling {}...", style("→").dim(), style(component_name).cyan());

    // Step 1: Transpile the .at file
    let rust_code = transpile_file(input, None)
        .map_err(|e| anyhow::anyhow!("{}: {}", style("Transpilation failed").red(), e))?;

    println!("{} Generating runnable example...", style("→").dim());

    // Step 2: Generate a complete runnable example
    let example_code = generate_runnable_example(&rust_code, backend)?;

    // Step 3: Create/update the example in the correct crate
    let examples_dir = if backend == "gpui" {
        PathBuf::from("crates/auto-ui-gpui/examples")
    } else {
        PathBuf::from("crates/auto-ui-iced/examples")
    };

    fs::create_dir_all(&examples_dir)?;

    let example_file = examples_dir.join(format!("auto_ui_run_{}.rs", component_name.to_lowercase()));
    fs::write(&example_file, example_code)?;

    println!("{} Running with {} backend...", style("→").dim(), style(backend).cyan());
    println!();
    println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
    println!("{}", style("Press Ctrl+C to stop the application").dim());
    println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
    println!();

    // Step 4: Run the example
    use std::process::Command;
    let example_name = format!("auto_ui_run_{}", component_name.to_lowercase());

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--package"])
        .arg(if backend == "gpui" { "auto-ui-gpui" } else { "auto-ui-iced" })
        .arg("--example")
        .arg(&example_name);

    let status = cmd.status()?;

    // Clean up unless keep_temp is set
    if !keep_temp {
        let _ = fs::remove_file(&example_file);
    }

    if status.success() {
        println!();
        println!("{} {}", style("✓").green(), style("Application exited successfully").green());
    } else {
        println!();
        println!("{} {}", style("✗").red(), style("Application exited with error").red());
        if !keep_temp {
            println!("{} Example file kept at: {}", style("ℹ").yellow(), example_file.display());
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Generate a complete runnable example from transpiled code
fn generate_runnable_example(transpiled_code: &str, backend: &str) -> Result<String> {
    // Extract the actual component name from the generated code
    // The code contains: pub struct Name { ... }
    let actual_component_name = extract_component_name(transpiled_code)?;

    // Remove all use statements from transpiled code since we'll add our own
    let code_without_use = transpiled_code
        .lines()
        .filter(|line| !line.trim().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n");

    // Determine which backend to use and generate appropriate main function
    let (_backend_module, main_function) = if backend == "gpui" {
        ("auto_ui_gpui", format!(
            "{backend_module}::run_app::<{actual_component_name}>(\"AutoUI - {actual_component_name}\")",
            backend_module = "auto_ui_gpui",
            actual_component_name = actual_component_name
        ))
    } else {
        ("auto_ui_iced", format!(
            "{backend_module}::run_app::<{actual_component_name}>()",
            backend_module = "auto_ui_iced",
            actual_component_name = actual_component_name
        ))
    };

    // We need to add Default impl and main function
    let wrapper = format!(
        r#"
// Auto-generated runnable example
// Generated by: auto-ui-transpile run
// This file can be deleted after the application exits

use auto_ui::{{Component, View}};

// ===== Generated Component Code =====
{code_without_use}

// ===== Default Implementation =====
impl Default for {actual_component_name} {{
    fn default() -> Self {{
        // Create component with default values
        Self::new(
            "Hello from Auto!".to_string(),
        )
    }}
}}

// ===== Main Function =====
fn main() -> auto_ui::AppResult<()> {{
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        AutoUI Quick Run                              ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("📝 Component: {actual_component_name}");
    println!("🚀 Running AutoUI application with {backend} backend...");
    println!();

    {main_function}
}}
"#,
        code_without_use = code_without_use,
        actual_component_name = actual_component_name,
        backend = backend,
        main_function = main_function
    );

    Ok(wrapper)
}

/// Extract component name from generated code
/// Looks for: pub struct Name { ... }
fn extract_component_name(code: &str) -> Result<String> {
    use regex::Regex;

    // Match: pub struct Name {
    let re = Regex::new(r"pub struct (\w+)\s*\{")?;
    if let Some(caps) = re.captures(code) {
        Ok(caps[1].to_string())
    } else {
        // Fallback: try to find first struct
        for line in code.lines() {
            if line.contains("pub struct") {
                if let Some(start) = line.find("pub struct") {
                    let after = &line[start + "pub struct".len()..];
                    let name: String = after
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !name.is_empty() {
                        return Ok(name);
                    }
                }
            }
        }
        Err(anyhow::anyhow!("Could not extract component name from generated code"))
    }
}


