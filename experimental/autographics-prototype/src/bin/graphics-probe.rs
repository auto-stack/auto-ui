//! graphics-probe: AGP verification runner.
//!
//! Usage:
//!   graphics-probe --list
//!   graphics-probe --case <case_id> --replay canonical --report <dir> [--mode auto|window|headless]
//!
//! Exit codes: 0 pass, 1 case assertions failed, 2 usage/unknown case,
//! 3 environment/runner failure.

use std::path::PathBuf;

fn usage() -> ! {
    eprintln!(
        "usage: graphics-probe --list | --case <case_id> --replay <name> --report <dir> [--mode auto|window|headless]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--list" || a == "-l") {
        for case in agp::host::cases::CASES {
            println!("{case}");
        }
        return;
    }

    let mut case: Option<String> = None;
    let mut replay: Option<String> = None;
    let mut report: Option<PathBuf> = None;
    let mut mode = "auto".to_string();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--case" => case = it.next().cloned(),
            "--replay" => replay = it.next().cloned(),
            "--report" => report = it.next().map(PathBuf::from),
            "--mode" => mode = it.next().cloned().unwrap_or_else(|| "auto".into()),
            other => {
                eprintln!("unknown argument: {other}");
                usage();
            }
        }
    }

    let (case, replay, report) = match (case, replay, report) {
        (Some(c), Some(r), p) => (
            c,
            r,
            p.unwrap_or_else(|| PathBuf::from("scratch/829/live-mode")),
        ),
        _ => usage(),
    };

    if !agp::host::cases::has_case(&case) {
        eprintln!("unknown case: {case} (try --list)");
        std::process::exit(2);
    }
    let code = agp::host::cases::run(&case, &replay, &report, &mode);
    if code != 0 {
        eprintln!(
            "case {case}: exit {code} (see {}/report.json)",
            report.display()
        );
    } else {
        println!(
            "case {case}: pass (report: {}/report.json)",
            report.display()
        );
    }
    std::process::exit(code);
}
