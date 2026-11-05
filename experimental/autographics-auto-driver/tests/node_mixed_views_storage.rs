//! auto-ui PLAN-049 node_mixed_views_storage: cold A→B consumption and the
//! storage identity chain for the mixed profile (V-S05/AC-05; T-06). A cold
//! consumer B (fresh MixedUiApp, no producer state) reads the SAME frozen
//! products and observes the same chain; the identity chain is enforced at
//! the consumption entry — a damaged world.at (sha drift), a drifted closure
//! module (module sha mismatch), a drifted schema string and a bad generator
//! identity are each named-rejected BEFORE any Run; the exact repair
//! (restored bytes) recovers on the NEXT actual Run without producer help.
//! Run: cargo test --locked --test node_mixed_views_storage -- --test-threads=1
use autographics_auto_driver::node_mixed::{parse_mixed_artifact, MixedUiApp};

const ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/world.at");
const IDENTITY: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/identity.json");
const CLOSURE_MAIN: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/closure/main.at"
);
const CLOSURE_COUNTER: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed/ui-ab-mixed/closure/counter.at"
);

#[test]
fn cold_consumer_reads_same_artifact_and_identity_chain_holds() {
    // A (implicit: the generator) -> B (this consumer): fresh mixed app from
    // the frozen products; both dimensions drive through the SAME chain.
    let mut b = MixedUiApp::new(ARTIFACT, WORLD, "plan049-storage").expect("cold consumer");
    b.start_run().expect("run");
    // root button (UI dimension).
    b.ui.enqueue_click(0, "main_ui_click_inc".into(), None, "cold-b".into());
    let r = b.drain();
    assert_eq!(r.dispatched, 1);
    assert_eq!(b.ui.texts(0)[0], "1");
    assert!(b.ui.error.is_none());
    // region click (graphics dimension back to Auto).
    assert!(b.enqueue_region_click("reg2"));
    let r = b.drain();
    assert_eq!(r.dispatched, 1);
    assert_eq!(b.ui.texts(0)[2], "1");
    // scene state observable off the same accepted cache (2D quad moved).
    let reg2 = b
        .scene_state()
        .into_iter()
        .find(|s| s.name == "reg2")
        .expect("reg2 state");
    let quad_x = reg2
        .elems
        .iter()
        .find_map(|e| match e {
            autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => Some(*x),
            _ => None,
        })
        .expect("quad");
    assert_eq!(quad_x, 8.0, "scene binding follows the same accepted state");

    // identity.json cross-checks (frozen provenance): world sha, api
    // identity, closure module shas all bind the same bytes we consumed.
    assert!(
        IDENTITY.contains(&format!("\"world_sha256\": \"{}\"", artifact_world_sha())),
        "identity.json binds the consumed world sha"
    );
    assert!(IDENTITY.contains("\"api\": \"autographics.node-ui-mixed/1\""));
    assert!(IDENTITY.contains("plan049 node-ui-mixed"));
    for (name, src) in [("main", CLOSURE_MAIN), ("counter", CLOSURE_COUNTER)] {
        let sha = agp::renderer::capture::sha256_hex(src.as_bytes());
        assert!(
            IDENTITY.contains(&format!("\"module:{name}\": \"{sha}\"")),
            "closure/{name}.at matches identity.json"
        );
    }
}

fn artifact_world_sha() -> String {
    ARTIFACT
        .split("\"world_sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("artifact binds a world sha")
        .to_string()
}

#[test]
fn damaged_world_copy_is_named_rejected_and_repairs_next_run() {
    // damaged world copy: one byte of business state changed -> sha drift
    // -> named rejection at the consumption entry (no Run happens).
    let damaged = WORLD.replace("var r_count=0", "var r_count=1");
    assert_ne!(damaged, WORLD);
    let err = match MixedUiApp::new(ARTIFACT, &damaged, "plan049-storage") {
        Err(e) => e,
        Ok(_) => panic!("damaged world copy must be rejected"),
    };
    assert!(
        err.contains("behavior source identity"),
        "named rejection: {err}"
    );
    // exact repair: restored bytes -> the NEXT actual Run recovers.
    let mut repaired = MixedUiApp::new(ARTIFACT, WORLD, "plan049-storage").expect("repair");
    repaired.start_run().expect("next run after repair");
    assert!(repaired.ui.error.is_none());
    assert_eq!(repaired.ui.texts(0)[0], "0");
}

#[test]
fn drifted_closure_module_and_schema_are_named_rejected() {
    // closure module drift (source-level) no longer matches identity.json.
    let drifted_main = CLOSURE_MAIN.replace(".count = .count + 1", ".count = .count + 2");
    assert_ne!(drifted_main, CLOSURE_MAIN);
    let sha = agp::renderer::capture::sha256_hex(drifted_main.as_bytes());
    assert!(
        !IDENTITY.contains(&format!("\"module:main\": \"{sha}\"")),
        "drifted module sha must not match the frozen identity"
    );

    // schema-string drift in ui.json: named reject (both spellings frozen).
    let bad_schema = ARTIFACT.replace(
        "autographics.node-ui-mixed/1",
        "autographics.node-ui-mixed/2",
    );
    let err = match parse_mixed_artifact(&bad_schema) {
        Err(e) => e,
        Ok(_) => panic!("schema drift must be rejected"),
    };
    assert!(
        err.contains("schema mismatch"),
        "schema drift rejected: {err}"
    );

    // 048 schema is a DIFFERENT identity: mixed parse rejects it, and an
    // embed artifact through the 048 parse is rejected too.
    let as_048 = ARTIFACT.replace("autographics.node-ui-mixed/1", "autographics.node-ui/1");
    assert!(
        parse_mixed_artifact(&as_048).is_err(),
        "048 schema must not parse as mixed"
    );

    // generator identity is preserved in the artifact (public consumption
    // identity for AG021).
    assert!(ARTIFACT.contains("plan049 node-ui-mixed"));
}
