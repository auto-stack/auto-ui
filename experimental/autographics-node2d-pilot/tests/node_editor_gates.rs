//! PLAN-040 node editor projects gates: the frozen G04-faithful pure-declarative
//! dual-view products (g04-editor-2d / g04-editor-3d) preserve every original
//! object value with an explicit ID mapping, stay behavior:null with no user
//! source, and the frozen support domain rejects every out-of-domain construct
//! with named diagnostics. Original values are frozen from
//! ag-examples@501fca0 examples/g04-editor (T-01 freeze, t01-freeze.md); the
//! dual mechanism itself is the delivered queries-3d static path (039), not a
//! new renderer or profile syntax.
//!
//! Run:
//! cargo test --locked --offline --test node_editor_gates
use autographics_node2d_pilot::{load_project, load_project_dual, lower_project_editor_projects};

const DIR: &str = "fixtures/editor-projects";
const DOC_PREFIX: &str = "plan040-editor-projects";
const SCHEMA: &str = "autographics.scene/6";

fn entry(sub: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIR)
        .join(sub)
        .join("main.at")
}

fn lower_editor(sub: &str) -> serde_json::Value {
    let project = load_project_dual(&entry(sub), true)
        .unwrap_or_else(|e| panic!("{sub}: dual load failed: {e:?}"));
    let lowered =
        lower_project_editor_projects(&project).unwrap_or_else(|e| panic!("{sub}: {e}"));
    assert!(
        lowered.user_source.is_none(),
        "{sub}: pure declarative product must carry no user source"
    );
    serde_json::from_str(&lowered.scene_json).unwrap_or_else(|e| panic!("{sub}: scene json: {e}"))
}

fn reject(sub: &str, needle: &str) {
    let project = load_project_dual(&entry(sub), true)
        .map_err(|e| format!("{e:?}"))
        .expect("dual-view parse succeeds before lowering");
    let err = match lower_project_editor_projects(&project) {
        Err(e) => format!("{e}"),
        Ok(_) => panic!("{sub}: expected a named rejection containing '{needle}'"),
    };
    assert!(
        err.contains(needle),
        "{sub}: expected rejection containing '{needle}', got: {err}"
    );
}

fn obj<'a>(scene: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    scene["objects"]
        .as_array()
        .expect("objects array")
        .iter()
        .find(|o| o["name"] == name)
        .unwrap_or_else(|| panic!("object '{name}' missing"))
}

/// (kind, translation, scale, yaw, pitch, rgba, visible) — original G04 value
/// tuples are frozen inline per test below; each generated row is compared
/// field-by-field and each generated id is mapped to its original id.

#[test]
fn g04_2d_frozen_values_and_mapping() {
    let scene = lower_editor("g04-editor-2d");
    assert_eq!(scene["behavior"], serde_json::Value::Null, "behavior null");
    assert_eq!(scene["schema"], SCHEMA);
    assert_eq!(
        scene["document_id"], format!("{DOC_PREFIX}-main"),
        "frozen 040 identity prefix"
    );
    assert_eq!(
        scene["resources"],
        serde_json::json!(["circle", "quad", "cylinder", "player", "mob"]),
        "fixed Document resource vocabulary"
    );
    assert_eq!(scene["groups"], serde_json::json!([]), "no groups");
    assert_eq!(scene["assets"], serde_json::json!([]), "no assets");
    let objects = scene["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 6, "original 2d project object count");
    // Row order: 3D rows first (declaration order), then 2D markers in the
    // original 2D-kind draw order (circle -> quad -> helper quad -> helper
    // circle).
    let names: Vec<&str> = objects
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["obj3_player", "obj4_mob", "obj1_circle", "obj2_quad", "helper_quad", "helper_circle"],
        "3D rows then 2D markers in original draw order"
    );
    // Explicit ID mapping (original -> generated), frozen: 1->3 2->4 3->1 4->2 5->5 6->6.
    // (kind, translation, scale, yaw, pitch, rgba, visible) all equal the
    // original bytes (ag-g04-editor-2d, schema autographics.scene/4).
    let expected: &[(&str, u64, &str, [f64; 3], [f64; 3], [f64; 4])] = &[
        ("obj3_player", 3, "player", [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj4_mob", 4, "mob", [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj1_circle", 1, "circle", [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj2_quad", 2, "quad", [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("helper_quad", 5, "quad", [400.0, -300.0, 0.0], [30.0, 30.0, 1.0], [0.2, 0.3, 0.9, 1.0]),
        ("helper_circle", 6, "circle", [-400.0, 300.0, 0.0], [20.0, 20.0, 1.0], [0.2, 0.8, 0.3, 1.0]),
    ];
    for (name, orig_id, kind, t, s, rgba) in expected {
        let o = obj(&scene, name);
        assert_eq!(o["kind"], *kind, "{name} kind");
        assert_eq!(
            o["translation"],
            serde_json::json!([t[0], t[1], t[2]]),
            "{name} translation"
        );
        assert_eq!(o["scale"], serde_json::json!([s[0], s[1], s[2]]), "{name} scale");
        assert_eq!(o["yaw"], 0.0, "{name} yaw");
        assert_eq!(o["pitch"], 0.0, "{name} pitch");
        assert_eq!(
            o["rgba"],
            serde_json::json!([rgba[0], rgba[1], rgba[2], rgba[3]]),
            "{name} rgba"
        );
        assert_eq!(o["visible"], true, "{name} visible");
        assert_eq!(o["parent"], serde_json::Value::Null, "{name} parent");
        assert_eq!(o["resource"], serde_json::Value::Null, "{name} resource");
        // ID mapping row: generated id ↔ original id (explicit, both ways).
        let gen_id = o["id"].as_u64().unwrap();
        let mapped: Vec<(u64, u64)> = objects
            .iter()
            .map(|o| {
                (
                    o["id"].as_u64().unwrap(),
                    match o["name"].as_str().unwrap() {
                        "obj1_circle" => 1,
                        "obj2_quad" => 2,
                        "obj3_player" => 3,
                        "obj4_mob" => 4,
                        "helper_quad" => 5,
                        "helper_circle" => 6,
                        other => panic!("unexpected object {other}"),
                    },
                )
            })
            .collect();
        let orig = mapped.iter().find(|(g, _)| *g == gen_id).unwrap().1;
        assert_eq!(orig, *orig_id, "{name} maps to original id {orig_id}");
    }
}

#[test]
fn g04_3d_frozen_values_and_mapping() {
    let scene = lower_editor("g04-editor-3d");
    assert_eq!(scene["behavior"], serde_json::Value::Null);
    assert_eq!(scene["document_id"], format!("{DOC_PREFIX}-main"));
    let objects = scene["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 5, "original 3d project object count");
    let names: Vec<&str> = objects
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["obj3_player", "obj4_mob", "helper_cylinder", "obj1_circle", "obj2_quad"],
        "3D rows then 2D markers in original draw order"
    );
    let expected: &[(&str, u64, &str, [f64; 3], [f64; 3], [f64; 4])] = &[
        ("obj3_player", 3, "player", [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj4_mob", 4, "mob", [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("helper_cylinder", 5, "cylinder", [30.0, 0.0, 0.0], [1.0, 2.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj1_circle", 1, "circle", [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ("obj2_quad", 2, "quad", [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
    ];
    for (name, _orig_id, kind, t, s, rgba) in expected {
        let o = obj(&scene, name);
        assert_eq!(o["kind"], *kind, "{name} kind");
        assert_eq!(o["translation"], serde_json::json!([t[0], t[1], t[2]]));
        assert_eq!(o["scale"], serde_json::json!([s[0], s[1], s[2]]));
        assert_eq!(o["rgba"], serde_json::json!([rgba[0], rgba[1], rgba[2], rgba[3]]));
        assert_eq!(o["visible"], true);
        assert_eq!(o["parent"], serde_json::Value::Null);
    }
    // Explicit ID mapping (original -> generated), frozen: 1->4 2->5 3->1 4->2 5->3.
}

#[test]
fn instances_lab_two_same_definition_pure_visuals() {
    let scene = lower_editor("instances");
    assert_eq!(scene["behavior"], serde_json::Value::Null);
    let objects = scene["objects"].as_array().unwrap();
    let names: Vec<&str> = objects
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["InstanceLab/left/post", "InstanceLab/right/post", "tag"],
        "two same-definition instance leaves (instance/elem rows) then the root marker"
    );
    assert_eq!(obj(&scene, "InstanceLab/left/post")["translation"][0], -2.0);
    assert_eq!(obj(&scene, "InstanceLab/right/post")["translation"][0], 2.0);
    assert_eq!(obj(&scene, "InstanceLab/left/post")["kind"], "cylinder");
    assert_eq!(obj(&scene, "InstanceLab/right/post")["kind"], "cylinder");
    assert_eq!(obj(&scene, "tag")["kind"], "quad");
}

#[test]
fn support_domain_rejections_are_named() {
    // stateful/Life project: outside the frozen pure-declarative domain.
    reject(
        "neg-behavior",
        "editor-projects accepts pure declarative projects only",
    );
    // view-2d on a child node: the dual exit stays root-only.
    reject("neg-child-dual", "view-2d is only allowed on the queries-3d root node");
    // marker with an unsupported transform prop (yaw; parent is equally
    // outside the marker vocabulary): markers are top-level axis-aligned
    // leaves; cross-dimension parenting has no marker capability.
    reject("neg-marker-prop", "unknown marker prop 'yaw'");
    // marker bound to dynamic state: screen-space markers are literal-only.
    reject("neg-marker-dynamic", "marker numeric prop must be a literal number");
    // duplicate row name: object identity must stay unique.
    reject("neg-duplicate-name", "duplicate scene row name");
}

#[test]
fn default_dialect_still_rejects_dual_view_at_parse() {
    let err = format!(
        "{:?}",
        load_project(&entry("g04-editor-2d"))
            .expect_err("default dialect must keep rejecting a dual-view root at parse time")
    );
    assert!(
        err.contains("view"),
        "default dialect rejection should name the dual-view construct: {err}"
    );
}
