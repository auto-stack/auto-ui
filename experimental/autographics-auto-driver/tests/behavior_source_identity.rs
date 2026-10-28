//! Frozen actual outputs exported before PLAN-854 changed the assembler.
//! No golden regeneration path: expectations are delivered PLAN-853 bytes.
use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
use std::path::Path;

#[test]
fn all_versions_match_frozen_actual_bytes_regions_credentials_and_diagnostics() {
    let records: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/behavior-source-853.json")).unwrap();
    assert_eq!(records.as_array().unwrap().len(), 10);
    for record in records.as_array().unwrap() {
        let old = &record["snapshot"];
        let bytes: Vec<u8> = serde_json::from_value(old["bytes"].clone()).unwrap();
        let api = Api::parse(old["version"].as_str().unwrap()).unwrap();
        let current =
            SourceSnapshot::from_bytes_for(Path::new("fixture.at"), bytes.clone(), api).unwrap();
        assert_eq!(
            serde_json::to_value(&current).unwrap(),
            *old,
            "{}",
            record["name"]
        );
        assert_eq!(api.profile(), record["profile"].as_i64().unwrap() as i32);
        assert_eq!(
            current.source.len(),
            record["combined_bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            current.source.len() - bytes.len(),
            record["managed_bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            agp::renderer::capture::sha256_hex(current.source.as_bytes()),
            old["combined_hash"]
        );
        assert_eq!(agp::renderer::capture::sha256_hex(&bytes), old["user_hash"]);
        assert_eq!(
            &current.source.as_bytes()[current.regions[2].start..current.regions[2].end],
            bytes
        );
        assert!(current
            .diagnostic("callback", "app_update", "failure")
            .contains("SDK supplied no mapped line"));
        // Establish that byte-level drift cannot hide behind snapshot metadata.
        let changed = format!("{} ", current.source);
        assert_ne!(
            agp::renderer::capture::sha256_hex(changed.as_bytes()),
            old["combined_hash"]
        );
    }
    assert!(Api::parse("autographics.behavior/6").is_err());
}
