//! auto-ui PLAN-032 node2d_full_game_gates (V-S01): the components-2d-runtime
//! lowering surface. The committed G2 fixture lowers byte-identically to the
//! committed product (regeneration determinism), the generated skeleton
//! carries exactly the frozen T-02 contract (snapshot store + snapshot poll
//! dispatch, batch despawn, patrol, end-state Enter, kind=0 four-frame
//! closeout — and NO idle-turn gate, no cursor reads, no business names), the
//! scene declares autographics.components2d/1 with the atlas sprite binding
//! and the typed png/wav/ttf assets, and the named rejections fire (missing
//! root `state`, spawn rows, reserved author fn, character without pose/
//! facing, character without atlas). Run:
//! cargo test --locked --offline --test node2d_full_game_gates
use autographics_node2d_pilot::{load_project, lower_project_components2d_runtime};
use std::path::PathBuf;

const MAIN: &str = include_str!("../fixtures/components2d-runtime/g2/main.at");
const PLAYER: &str = include_str!("../fixtures/components2d-runtime/g2/player.at");
const COIN: &str = include_str!("../fixtures/components2d-runtime/g2/coin.at");
const ENEMY: &str = include_str!("../fixtures/components2d-runtime/g2/enemy.at");
const GOAL: &str = include_str!("../fixtures/components2d-runtime/g2/goal.at");
const PIT: &str = include_str!("../fixtures/components2d-runtime/g2/pit.at");
const RESOURCES: &str = include_str!("../fixtures/components2d-runtime/g2/resources.at");
const HUDMAP: &str = include_str!("../fixtures/components2d-runtime/g2/hudmap.at");
const STATUSMAP: &str = include_str!("../fixtures/components2d-runtime/g2/statusmap.at");
const COMMITTED_SHA: &str = "d0c6c196ea73d45bb7d66198aa1b614b09da1145a91608ef63919ceaea44a0ac";

fn fixture_files() -> Vec<(&'static str, String)> {
    vec![
        ("main.at", MAIN.to_string()),
        ("player.at", PLAYER.to_string()),
        ("coin.at", COIN.to_string()),
        ("enemy.at", ENEMY.to_string()),
        ("goal.at", GOAL.to_string()),
        ("pit.at", PIT.to_string()),
        ("resources.at", RESOURCES.to_string()),
        ("hudmap.at", HUDMAP.to_string()),
        ("statusmap.at", STATUSMAP.to_string()),
    ]
}

/// Lower an inline project in a fresh scratch dir with the committed fixture
/// assets (wav/bank, font, license, atlas png).
fn lower_temp(files: &[(&str, String)]) -> Result<autographics_node2d_pilot::LowerOutputComponents2dRuntime, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan032-gates")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        ));
    let assets = dir.join("assets");
    std::fs::create_dir_all(&assets).expect("scratch dir");
    for (name, body) in files {
        let lf = body.replace('\r', "");
        std::fs::write(dir.join(name), lf).expect("write module");
    }
    let src_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/components2d-runtime/g2/assets");
    for blob in [
        "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
        "lilita_one_regular.ttf",
        "license.txt",
        "8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.png",
    ] {
        std::fs::copy(src_assets.join(blob), assets.join(blob)).expect("copy asset");
    }
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components2d_runtime(&project, &dir).map_err(|e| e.to_string())
}

#[test]
fn g2_fixture_lowers_byte_identically_to_the_committed_product() {
    let out = lower_temp(&fixture_files()).expect("lower fixture");
    let src = out.base.user_source.clone().expect("source");
    assert_eq!(
        autographics_node2d_pilot::scene::sha256_hex(src.as_bytes()),
        COMMITTED_SHA,
        "regeneration must be byte-identical to the committed generated world"
    );
    // frozen scene identity: components2d/1 + atlas sprite binding + assets
    assert!(
        out.base.scene_json.contains("\"api\": \"autographics.components2d/1\""),
        "scene api identity"
    );
    assert!(
        out.base
            .scene_json
            .contains("\"sprite\": {\"asset\": \"sha256:8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df\", \"frame\": [0, 0, 32, 32], \"flip\": false}"),
        "atlas sprite binding on the character row"
    );
    assert!(out.base.scene_json.contains("\"format\": \"png\""), "png asset typed entry");
    assert_eq!(out.assets.len(), 4, "wav + ttf + license + png");
}

#[test]
fn generated_skeleton_carries_only_the_frozen_contract() {
    let out = lower_temp(&fixture_files()).expect("lower fixture");
    let src = out.base.user_source.clone().expect("source");
    // the frozen prelude: one stored snapshot then the snapshot poll
    assert!(src.contains(" r_snap = ag_actions_snapshot()\n cmp_events()\n"), "main_lifetick prelude");
    assert!(src.contains("let n = ag_events_snapshot()"), "whole-tick snapshot poll");
    assert!(src.contains("ag_event_snap_pair(i)"), "snapshot field reads");
    assert!(!src.contains("ag_event_next("), "no cursor reads in the components2d skeleton");
    // the author destruction lowers to the single-element batch
    assert!(
        src.contains("ag_object_despawn_batch(1, coin_handle[j], 0, 0, 0, 0, 0, 0, 0)"),
        "obj.despawn lowers to the batch call"
    );
    assert!(!src.contains("ag_object_despawn("), "no single despawn anywhere");
    // patrol + restart + kind=0 closeout
    assert!(src.contains("fn cmp_patrol(dt float)"), "generated patrol");
    assert!(src.contains("ag_object_move(enemy_handle[0]"), "patrol moves the enemy instance");
    assert!(src.contains("fn cmp_restart()"), "generated end-state Enter");
    assert!(src.contains("ag_round_request()"), "round request wired");
    assert!(src.contains("ag_present2d(0, "), "kind=0 bundle with the pose frame");
    assert!(src.contains("cmp_frame_x(player_pose[0])"), "frame from the frozen pose field");
    assert!(src.contains("cmp_flip()"), "flip from the frozen facing field");
    // the banned idle-turn gate: no halt/busy services, no held reads
    assert!(!src.contains("ag_action_held("), "no per-direction held reads");
    assert!(!src.contains("ag_action_pressed("), "no separate pressed reads");
    assert!(!src.contains("halt"), "no idle-turn gate fields");
    // the bounded four-row buffer with the fifth-row fault count
    assert!(src.contains("if (r_np < 5)"), "5-row buffer shape (fault 51 at the closeout bound)");
    // every-tick duty: the walk tick runs snapshot+move+grounded+patrol+events+bundle
}

#[test]
fn named_rejections_fire_before_any_generation() {
    // missing root `state`
    let mut files = fixture_files();
    files[0] = ("main.at", MAIN.replace("var state = 0", "var status = 0").replace("text_field: \"state\"", "text_field: \"status\""));
    let e = match lower_temp(&files) { Ok(_) => panic!("expected rejection"), Err(e) => e };
    assert!(e.contains("state"), "missing root state named reject: {e}");
    // spawn rows are outside the frozen surface
    let mut files = fixture_files();
    files[0] = (
        "main.at",
        MAIN.replace("        patrol {", "        spawn ghost { template: \"enemy\", slot: 6, at: (1.0, 1.0) }\n        patrol {"),
    );
    let e = match lower_temp(&files) { Ok(_) => panic!("expected rejection"), Err(e) => e };
    assert!(e.contains("spawn"), "spawn row named reject: {e}");
    // reserved author fn
    let mut files = fixture_files();
    files[7] = ("hudmap.at", HUDMAP.to_string() + "fn cmp_patrol(x int) int {return x}");
    let e = match lower_temp(&files) { Ok(_) => panic!("expected rejection"), Err(e) => e };
    assert!(e.contains("reserved"), "reserved author fn named reject: {e}");
    // character without pose/facing
    let mut files = fixture_files();
    files[1] = ("player.at", PLAYER.replace("        var pose = 0\n", ""));
    let e = match lower_temp(&files) { Ok(_) => panic!("expected rejection"), Err(e) => e };
    assert!(e.contains("pose"), "missing pose named reject: {e}");
    // character without atlas
    let mut files = fixture_files();
    let atlas_start = MAIN.find("        atlas {").unwrap();
    let atlas_end = MAIN[atlas_start..].find("}\n").unwrap() + atlas_start + 2;
    files[0] = ("main.at", format!("{}{}", &MAIN[..atlas_start], &MAIN[atlas_end..]));
    let e = match lower_temp(&files) { Ok(_) => panic!("expected rejection"), Err(e) => e };
    assert!(e.contains("atlas"), "missing atlas named reject: {e}");
}
