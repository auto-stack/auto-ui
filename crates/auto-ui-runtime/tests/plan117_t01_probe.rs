//! PLAN-117 T-01 probe: A2Vue lane generation for the two original apps
//! (029-photo-gallery, 031-image-viewer) plus minimal probes isolating
//! record literals and multibyte strings. Sources are read from the repo
//! example trees (same commit as the worktree base). Outcome is recorded
//! by the caller from the printed status lines.

#![cfg(feature = "ace-bridge")]

use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane;

fn lane_for(root: &str, rel: &str) -> Result<(usize, usize, Vec<String>), String> {
    let path = format!("{root}/{rel}");
    let src = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let prepared = prepare_component_source(std::path::Path::new(root), &src)
        .map(|p| p.source)
        .unwrap_or_else(|e| {
            eprintln!("  prepare_component_source declined: {e}");
            src
        });
    if std::env::var("PLAN117_DUMP_PREP").is_ok() {
        let _ = std::fs::write(
            "D:/autostack/plan-artifacts/plan-117/probes/029-prepared.at",
            &prepared,
        );
    }
    match generate_sfc_with_lane(&prepared, rel) {
        Ok((sfc, bodies, absent)) => Ok((
            sfc.len(),
            bodies.len(),
            absent.iter().map(|n| n.to_string()).collect(),
        )),
        Err(e) => Err(format!("{rel}: {e}")),
    }
}

fn lane_raw(path: &str, label: &str) -> Result<(usize, usize, Vec<String>), String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    match generate_sfc_with_lane(&src, label) {
        Ok((sfc, bodies, absent)) => Ok((
            sfc.len(),
            bodies.len(),
            absent.iter().map(|n| n.to_string()).collect(),
        )),
        Err(e) => Err(format!("{label}: {e}")),
    }
}

#[test]
fn plan117_t01_029_lane_probe() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/029-photo-gallery");
    match lane_for(root, "src/front/app.at") {
        Ok((n, b, a)) => println!("PLAN117 PROBE 029 lane OK sfc_bytes={n} bodies={b} absent={a:?}"),
        Err(e) => println!("PLAN117 PROBE 029 lane FAIL {e}"),
    }
}

#[test]
fn plan117_t01_029_lane_raw_probe() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/029-photo-gallery/src/front/app.at");
    match lane_raw(p, "029-raw") {
        Ok((n, b, a)) => println!(
            "PLAN117 PROBE 029 lane-RAW OK sfc_bytes={n} bodies={b} absent={a:?}"
        ),
        Err(e) => println!("PLAN117 PROBE 029 lane-RAW FAIL {e}"),
    }
}

#[test]
fn plan117_t01_031_lane_probe() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/031-image-viewer");
    match lane_for(root, "src/front/app.at") {
        Ok((n, b, a)) => println!("PLAN117 PROBE 031 lane OK sfc_bytes={n} bodies={b} absent={a:?}"),
        Err(e) => println!("PLAN117 PROBE 031 lane FAIL {e}"),
    }
}

#[test]
fn plan117_t01_probes_lane() {
    let base = "D:/autostack/plan-artifacts/plan-117/probes";
    for (name, label) in [
        ("p-record.at", "p-record"),
        ("p-record2.at", "p-record2"),
        ("p-record3.at", "p-record3"),
        ("p-multibyte.at", "p-multibyte"),
    ] {
        match lane_raw(&format!("{base}/{name}"), label) {
            Ok((n, b, a)) => println!(
                "PLAN117 PROBE {label} lane OK sfc_bytes={n} bodies={b} absent={a:?}"
            ),
            Err(e) => println!("PLAN117 PROBE {label} lane FAIL {e}"),
        }
    }
}
