//! PLAN-072: the standalone Windows x64 game player. Double-click entry:
//! `player.json` beside the exe names the game; the scene/resources/scripts
//! load under the exe root (never the working directory) through the shared
//! verified `Storage::load_project`, and the window opens straight into the
//! game's Playing state. No editor chrome, no sample fallback, no absolute
//! project paths. QA chains (`--qa`) drive real posted OS input for the
//! bounded proof and the integration test entry.
#![recursion_limit = "512"]

#[path = "player/config.rs"]
mod config;
#[path = "player/app.rs"]
mod app;
#[path = "player/native.rs"]
mod native;
#[path = "player/qa.rs"]
mod qa;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |n: &str| {
        args.iter()
            .position(|v| v == n)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    // The product entry takes no required argument: player.json beside the
    // exe is the launch description. QA flags exist for the bounded proof and
    // the integration test only; unknown flags are rejected, not ignored.
    let known = ["--qa", "--report", "--config"];
    let unknown = args.iter().skip(1).any(|v| {
        v.starts_with("--") && !known.iter().any(|k| v == *k)
    });
    if unknown {
        eprintln!("usage: age-game-player [--config <player.json>] [--qa platformer-2d|platformer-3d --report <path>]");
        std::process::exit(2);
    }
    let root = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or("exe path unavailable")
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
    let config_path = match value("--config") {
        Some(p) => PathBuf::from(p),
        None => root.join("player.json"),
    };
    let config = match config::PlayerConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let qa = match value("--qa") {
        Some(name) => match app::script_from(&name) {
            Some(script) => Some(qa::Qa::new(script, config.display.title.clone())),
            None => {
                eprintln!("--qa expects platformer-2d or platformer-3d");
                std::process::exit(2);
            }
        },
        None => None,
    };
    if qa.is_some() != value("--report").is_some() {
        eprintln!("--qa and --report must be given together");
        std::process::exit(2);
    }
    let launch = app::App::new(config, root, qa, value("--report").map(PathBuf::from));
    let code = app::run(launch);
    std::process::exit(code);
}
