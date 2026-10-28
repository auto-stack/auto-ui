//! Probe case registry. T-03..T-08 add cases here.

pub mod breakout;
pub mod consumer;
pub mod frame_oracle;
pub mod gpu_foundation;
pub mod iced_shared_device;
pub mod lifecycle;
pub mod metrics;
pub mod negative;
pub mod risk_slice;
pub mod shared_runtime;
pub mod squash_game;
pub mod surface_compose;
pub mod surface_ime_os;
pub mod surface_input;
pub mod surface_lifecycle;
pub mod surface_offscreen;
pub mod surface_scenarios;

/// All known case ids (also printed by `--list`).
pub const CASES: &[&str] = &[
    "gpu-foundation",
    "shared-runtime",
    "ag2d-breakout",
    "ag3d-risk-slice",
    "ag3d-squash",
    "lifecycle",
    "metrics",
    "verifier-negative-controls",
    "iced-shared-device",
    "independent-consumer",
    "surface-offscreen",
    "surface-compose",
    "surface-input",
    "surface-scenarios",
    "surface-lifecycle",
    "surface-ime-os",
];

pub fn has_case(case: &str) -> bool {
    CASES.contains(&case)
}

/// Run one case; returns the process exit code.
pub fn run(case: &str, replay: &str, report_dir: &std::path::Path, mode: &str) -> i32 {
    match case {
        "gpu-foundation" => gpu_foundation::run(replay, report_dir, mode),
        "shared-runtime" => shared_runtime::run(replay, report_dir, mode),
        "ag2d-breakout" => breakout::run(replay, report_dir, mode),
        "ag3d-risk-slice" => risk_slice::run(replay, report_dir, mode),
        "ag3d-squash" => squash_game::run(replay, report_dir, mode),
        "lifecycle" => lifecycle::run(replay, report_dir, mode),
        "metrics" => metrics::run(replay, report_dir, mode),
        "verifier-negative-controls" => negative::run(replay, report_dir, mode),
        "iced-shared-device" => iced_shared_device::run(replay, report_dir, mode),
        "independent-consumer" => consumer::run(replay, report_dir, mode),
        "surface-offscreen" => surface_offscreen::run(replay, report_dir, mode),
        "surface-compose" => surface_compose::run(replay, report_dir, mode),
        "surface-input" => surface_input::run(replay, report_dir, mode),
        "surface-scenarios" => surface_scenarios::run(replay, report_dir, mode),
        "surface-lifecycle" => surface_lifecycle::run(replay, report_dir, mode),
        "surface-ime-os" => surface_ime_os::run(replay, report_dir, mode),
        other => {
            eprintln!("unknown case: {other} (try --list)");
            2
        }
    }
}
