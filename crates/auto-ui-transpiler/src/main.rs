// Auto UI Transpiler CLI
//
// Transpiles Auto language .at files to auto-ui Rust code
// Supports three modes:
// - Single file transpilation
// - Batch directory processing
// - Watch mode for auto-recompilation

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Color};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::fs;

#[derive(Subcommand, Debug)]
enum Command {
    /// Transpile a single .at file
    File {
        /// Input .at file path
        #[arg(short, long)]
        input: String,

        /// Output .rs file path (optional, prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,

        /// Check syntax without writing output
        #[arg(long)]
        check: bool,
    },

    /// Transpile all .at files in a directory
    Batch {
        /// Input directory containing .at files
        #[arg(short, long)]
        input: String,

        /// Output directory for .rs files
        #[arg(short, long)]
        output: String,

        /// Number of parallel jobs (default: number of CPU cores)
        #[arg(short, long, default_value_t = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4))]
        jobs: usize,

        /// Preserve directory structure
        #[arg(long)]
        preserve_structure: bool,

        /// Watch for changes and auto-recompile
        #[arg(long, short)]
        watch: bool,
    },

    /// Watch a directory and transpile on file changes
    Watch {
        /// Directory to watch
        #[arg(short, long)]
        input: String,

        /// Output directory for .rs files
        #[arg(short, long)]
        output: String,

        /// Verbose output (print all events)
        #[arg(long, short)]
        verbose: bool,
    },
}

/// Auto UI Transpiler - Convert .at files to auto-ui Rust code
#[derive(Parser, Debug)]
#[command(name = "auto-ui-transpile")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::File { input, output, check } => {
            run_single_file(&input, output.as_deref(), check)?;
        }
        Command::Batch { input, output, jobs, preserve_structure, watch } => {
            if watch {
                run_watch(&input, &output, true)?;
            } else {
                run_batch(&input, &output, jobs, preserve_structure)?;
            }
        }
        Command::Watch { input, output, verbose } => {
            run_watch(&input, &output, verbose)?;
        }
    }

    Ok(())
}

/// Transpile a single .at file
fn run_single_file(input: &str, output: Option<&str>, check: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler").fg(Color::Cyan).bold());
    println!("{}", style("==================").fg(Color::Cyan).bold());
    println!("Input: {}", style(input).fg(Color::White));
    if let Some(out) = output {
        println!("Output: {}", style(out).fg(Color::White));
    }
    println!();

    // Validate input file exists
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input file not found").fg(Color::Red), input));
    }

    // Start timing
    let start = Instant::now();

    // Transpile the file
    let rust_code = auto_ui::trans::transpile_file(input, None)
        .map_err(|e| anyhow::anyhow!("{}: {}", style("Transpilation failed").fg(Color::Red), e))?;

    let elapsed = start.elapsed();

    if check {
        println!("{} {} {:?}", style("✓").fg(Color::Green), style("Syntax check passed").fg(Color::Green), elapsed);
        return Ok(());
    }

    // Output result
    if let Some(output_path) = output {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to file
        fs::write(output_path, rust_code)
            .map_err(|e| anyhow::anyhow!("{}: {}", style("Failed to write output file").fg(Color::Red), e))?;
        println!("{} {} {}", style("✓").fg(Color::Green), style("Generated").fg(Color::Green), style(output_path).fg(Color::White));
        println!("  {} {:?}", style("Time:").fg(Color::Dim), elapsed);
    } else {
        // Print to stdout
        println!("{}", style("// Generated Rust Code").fg(Color::Dim));
        println!("{}", style("=====================").fg(Color::Dim));
        println!();
        println!("{}", rust_code);
    }

    Ok(())
}

/// Transpile all .at files in a directory
fn run_batch(input: &str, output: &str, jobs: usize, preserve_structure: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler - Batch Mode").fg(Color::Cyan).bold());
    println!("{}", style("===============================").fg(Color::Cyan).bold());
    println!("Input: {}", style(input).fg(Color::White));
    println!("Output: {}", style(output).fg(Color::White));
    println!("Jobs: {}", style(jobs).fg(Color::White));
    println!();

    // Validate input directory exists
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input directory not found").fg(Color::Red), input));
    }

    // Collect all .at files
    let at_files = collect_at_files(input_path)?;
    let total_files = at_files.len();

    if total_files == 0 {
        println!("{} No .at files found in {}", style("ℹ").fg(Color::Yellow), input);
        return Ok(());
    }

    println!("{} Found {} .at file(s)", style("✓").fg(Color::Green), style(total_files).fg(Color::White));
    println!();

    // Create output directory
    fs::create_dir_all(output)?;

    // Process files
    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for (index, input_file) in at_files.iter().enumerate() {
        let relative_path = input_file.strip_prefix(input_path)?;
        let output_file = if preserve_structure {
            PathBuf::from(output).join(relative_path).with_extension("rs")
        } else {
            PathBuf::from(output).join(input_file.file_name().unwrap()).with_extension("rs")
        };

        print!("[{}/{}] ", style(index + 1).fg(Color::Dim), style(total_files).fg(Color::Dim));
        print!("{}", style(input_file.display()).fg(Color::White));

        // Transpile the file
        match auto_ui::trans::transpile_file(input_file, None) {
            Ok(rust_code) => {
                // Ensure parent directory exists
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Write output
                match fs::write(&output_file, rust_code) {
                    Ok(_) => {
                        println!(" {}", style("✓").fg(Color::Green));
                        success_count += 1;
                    }
                    Err(e) => {
                        println!(" {}", style("✗").fg(Color::Red));
                        println!("  {} Failed to write: {}", style("Error:").fg(Color::Red), e);
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                println!(" {}", style("✗").fg(Color::Red));
                println!("  {} {}", style("Error:").fg(Color::Red), e);
                error_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    println!();
    println!("{}", style("─").fg(Color::Dim).repeat(40));
    println!("{} {}", style("Summary:").fg(Color::Cyan).bold(), style(format!("{} succeeded, {} failed", success_count, error_count)).fg(Color::White));
    println!("{} {:?}", style("Time:").fg(Color::Cyan).bold(), elapsed);

    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Watch directory and transpile on changes
fn run_watch(input: &str, output: &str, verbose: bool) -> Result<()> {
    println!("{}", style("Auto UI Transpiler - Watch Mode").fg(Color::Cyan).bold());
    println!("{}", style("==============================").fg(Color::Cyan).bold());
    println!("Watching: {}", style(input).fg(Color::White));
    println!("Output: {}", style(output).fg(Color::White));
    println!();
    println!("{}", style("Press Ctrl+C to stop...").fg(Color::Dim));
    println!();

    // Validate input directory exists
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("{}: {}", style("Input directory not found").fg(Color::Red), input));
    }

    // Create output directory
    fs::create_dir_all(output)?;

    // Initial transpilation
    println!("{}", style("Running initial transpilation...").fg(Color::Dim));
    run_batch(input, output, 1, true)?;
    println!();

    // Setup file watcher
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    use notify_debouncer_mini::new_debouncer;

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = new_debouncer(std::time::Duration::from_millis(250), None, move |res| {
        if let Ok(events) = res {
            let _ = tx.send(events);
        }
    })?;

    watcher.watch(input_path, RecursiveMode::Recursive)?;

    println!("{} Watching for changes...", style("✓").fg(Color::Green));
    println!();

    // Process events
    for events in rx {
        for event in events {
            if verbose {
                println!("{:?}", event);
            }

            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("at") {
                            let relative_path = path.strip_prefix(input_path).unwrap_or(&path);
                            let output_file = PathBuf::from(output).join(relative_path).with_extension("rs");

                            println!("{} {}", style("Change detected:").fg(Color::Yellow), style(path.display()).fg(Color::White));

                            // Transpile the file
                            match auto_ui::trans::transpile_file(&path, None) {
                                Ok(rust_code) => {
                                    // Ensure parent directory exists
                                    if let Some(parent) = output_file.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }

                                    // Write output
                                    match fs::write(&output_file, rust_code) {
                                        Ok(_) => {
                                            println!("{} {}", style("✓ Transpiled:").fg(Color::Green), style(output_file.display()).fg(Color::White));
                                        }
                                        Err(e) => {
                                            println!("{} Failed to write: {}", style("Error:").fg(Color::Red), e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("{} {}", style("Error:").fg(Color::Red), e);
                                }
                            }
                            println!();
                        }
                    }
                }
                _ => {}
            }
        }
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
