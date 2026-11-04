//! auto-ui PLAN-048 node_ui_storage: cold A→B consumption and the storage
//! identity chain (V-S05/AC-06). A cold consumer B (fresh NodeUiApp, no
//! producer state) reads the SAME frozen products and observes the same
//! chain; the identity chain is enforced at the consumption entry — a
//! damaged world.at (sha drift), a drifted closure module (module sha
//! mismatch), a drifted ui.json schema string and a bad generator identity
//! are each named-rejected BEFORE any Run; the exact repair (restored
//! bytes) recovers on the NEXT actual Run without producer help.
//! Run: cargo test --locked --test node_ui_storage -- --test-threads=1
use autographics_auto_driver::node_ui::NodeUiApp;

const ARTIFACT: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/ui.json");
const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/world.at");
const IDENTITY: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/identity.json");
const CLOSURE_MAIN: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/closure/main.at");
const CLOSURE_COUNTER: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/closure/counter.at");
const CLOSURE_PANEL: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-ui/ui-ab/closure/panel.at");

#[test]
fn cold_consumer_reads_same_artifact_and_identity_chain_holds() {
    // A (implicit: the generator) -> B (this consumer): fresh app from the
    // frozen products; the same observable chain as node_ui_flow's V-S01.
    let mut b = NodeUiApp::new(ARTIFACT, WORLD, "plan048-storage").expect("cold consumer");
    b.start_run().expect("run");
    let a = b.instance_index("Main/a").unwrap();
    b.enqueue_click(a, "counter_ui_click_inc".into(), Some(0), "cold-b".into());
    let r = b.drain();
    assert_eq!(r.dispatched, 1);
    assert_eq!(b.texts(a)[1], "1");
    assert!(b.error.is_none());

    // identity.json cross-checks (frozen provenance): world sha, api identity,
    // closure module shas all bind the same bytes we consumed.
    assert!(
        IDENTITY.contains(&format!("\"world_sha256\": \"{}\"", artifact_world_sha())),
        "identity.json binds the consumed world sha"
    );
    assert!(IDENTITY.contains("\"api\": \"autographics.node-ui/1\""));
    assert!(IDENTITY.contains("plan048 node-ui"));
    for (name, src) in [("main", CLOSURE_MAIN), ("counter", CLOSURE_COUNTER), ("panel", CLOSURE_PANEL)] {
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
    let damaged = WORLD.replace("var counter_count=[0,10,100]", "var counter_count=[0,10,101]");
    assert_ne!(damaged, WORLD);
    let err = match NodeUiApp::new(ARTIFACT, &damaged, "plan048-storage") {
        Err(e) => e,
        Ok(_) => panic!("damaged world copy must be rejected"),
    };
    assert!(
        err.contains("behavior source identity"),
        "named rejection: {err}"
    );
    // exact repair: restored bytes -> the NEXT actual Run recovers.
    let mut repaired = NodeUiApp::new(ARTIFACT, WORLD, "plan048-storage").expect("repair");
    repaired.start_run().expect("next run after repair");
    assert!(repaired.error.is_none());
}

#[test]
fn drifted_closure_module_and_schema_are_named_rejected() {
    // closure module drift (source-level) no longer matches identity.json.
    let drifted_counter = CLOSURE_COUNTER.replace(".count = .count + .step", ".count = .count + 1");
    assert_ne!(drifted_counter, CLOSURE_COUNTER);
    let sha = agp::renderer::capture::sha256_hex(drifted_counter.as_bytes());
    assert!(
        !IDENTITY.contains(&format!("\"module:counter\": \"{sha}\"")),
        "drifted module sha must not match the frozen identity"
    );

    // schema-string drift in ui.json: named reject (two-graph spelling is
    // frozen on both sides).
    let bad_schema = ARTIFACT.replace("autographics.node-ui/1", "autographics.node-ui/2");
    let err = match NodeUiApp::new(&bad_schema, WORLD, "plan048-storage") {
        Err(e) => e,
        Ok(_) => panic!("schema drift must be rejected"),
    };
    assert!(err.contains("schema mismatch"), "schema drift rejected: {err}");

    // generator identity is preserved in the artifact (public consumption
    // identity for 049/AG021).
    assert!(ARTIFACT.contains("plan048 node-ui"));
}
