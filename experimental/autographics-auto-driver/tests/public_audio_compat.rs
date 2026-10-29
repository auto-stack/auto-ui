//! Actual assembled bytes frozen with pristine f7c83c6 assembler; see baseline receipt.
use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
use serde_json::{json, Value};
use std::path::Path;
#[test]
fn all_fifteen_delivered_profiles_keep_exact_assembled_bytes() {
    let mut profiles = serde_json::Map::new();
    for api in Api::all().into_iter().filter(|a| !a.audio()) {
        let s = SourceSnapshot::from_bytes_for(
            Path::new("legacy.at"),
            b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
            api,
        )
        .unwrap();
        profiles.insert(api.version().into(),json!({"combined":s.combined_hash,"api":s.api_hash,"bridge":s.bridge_hash,"version":api.number(),"profile":api.profile()}));
    }
    assert_eq!(profiles.len(), 15);
    if let Ok(path) = std::env::var("PLAN012_CAPTURE_LEGACY") {
        std::fs::write(path, serde_json::to_vec_pretty(&profiles).unwrap()).unwrap();
        println!("Captured15 legacy snapshots; capture mode is NOT verification");
        return;
    }
    let baseline: Value =
        serde_json::from_str(include_str!("../research/plan012-legacy-profiles.json")).unwrap();
    assert_eq!(Value::Object(profiles), baseline);
}
