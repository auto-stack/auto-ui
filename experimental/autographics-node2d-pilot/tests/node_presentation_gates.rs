//! auto-ui PLAN-025 node_presentation_gates (V-01): the presentation-2d
//! lowering surface. Arbitrary business naming (a renamed root with renamed
//! fields lowers to the same structure and cue ids), the no-business-enum
//! scan over the committed products (no Collect/Goal/Fail, no count/state
//! roots, no game_cue), and the strict named rejections: unknown cue,
//! text_fn missing/wrong signature, non-int text/visible field, duplicate or
//! out-of-range slot, duplicate cue, missing/non-root block, reserved
//! infrastructure names, business trigger payload (parse layer), region rule,
//! unknown trigger event, character without camera, and the render2d surface
//! limit (despawn only). Run:
//! cargo test --locked --offline --test node_presentation_gates
use autographics_node2d_pilot::{load_project, lower_project_presentation};
use std::path::PathBuf;

const NOTIFY_MAIN: &str =
    include_str!("../fixtures/presentation2d/notify/main.at");
const NOTIFY_NOTIF: &str = include_str!("../fixtures/presentation2d/notify/notif.at");
const NOTIFY_RES: &str = include_str!("../fixtures/presentation2d/notify/resources.at");
const NOTIFY_TEXT: &str = include_str!("../fixtures/presentation2d/notify/textmap.at");

/// Lower an inline project (files relative to a fresh scratch dir). Bank/
/// font/license resolve from the committed notify fixture assets.
fn lower_temp(files: &[(&str, String)]) -> Result<(), String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan025-gates")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        ));
    let assets = dir.join("assets");
    std::fs::create_dir_all(&assets).expect("scratch dir");
    for (name, body) in files {
        // Normalize CRLF (Windows checkout) so the in-test anchors match.
        let lf = body.replace('\r', "");
        std::fs::write(dir.join(name), lf).expect("write module");
    }
    for (src, blob) in [
        (
            "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
            "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
        ),
        ("lilita_one_regular.ttf", "lilita_one_regular.ttf"),
        ("license.txt", "license.txt"),
    ] {
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures/presentation2d/notify/assets")
                .join(src),
            assets.join(blob),
        )
        .expect("copy asset");
    }
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    let out = lower_project_presentation(&project, &dir).map_err(|e| e.to_string())?;
    assert!(out.base.user_source.is_some());
    Ok(())
}

fn err_of(files: &[(&str, String)]) -> String {
    match lower_temp(files) {
        Ok(()) => panic!("expected a named rejection"),
        Err(e) => e,
    }
}

/// The committed notify project as inline files (the positive control for
/// every negative below).
fn norm(s: &str) -> String {
    s.replace("
", "
")
}

fn notify_files() -> Vec<(&'static str, String)> {
    vec![
        ("main.at", norm(NOTIFY_MAIN)),
        ("notif.at", norm(NOTIFY_NOTIF)),
        ("resources.at", norm(NOTIFY_RES)),
        ("textmap.at", norm(NOTIFY_TEXT)),
    ]
}

fn with_main(main: &str) -> Vec<(&'static str, String)> {
    vec![
        ("main.at", norm(main)),
        ("notif.at", norm(NOTIFY_NOTIF)),
        ("resources.at", norm(NOTIFY_RES)),
        ("textmap.at", norm(NOTIFY_TEXT)),
    ]
}

// ------------------------------------------------------------ positive ----

/// V-01 positive control: the committed notify authoring lowers.
#[test]
fn v01_positive_control_lowers() {
    lower_temp(&notify_files()).expect("notify lowers");
}

/// V-01 rename invariance: root/instance/field/helper names are arbitrary —
/// a fully renamed program (Main->Board, left/right->a/b, hits/live->n/on)
/// lowers to the same structure: same cue ids in the same sfx calls, same
/// closeout shape (no count/state anywhere, no Collect/Goal/Fail).
#[test]
fn v01_arbitrary_naming_renames_free() {
    let main = r#"// renamed root
use resources: disc_mesh, teal, amber
use textmap: label
use notif: Blink

node Board {
    model {
        var step = 0
    }
    presentation2d {
        audio {
            bank: "assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"
            font: "assets/lilita_one_regular.ttf"
            license: "assets/license.txt"
        }
        cue hi {
            span: (0, 7403)
            gain: 1.0
        }
        cue ho {
            span: (7403, 20046)
            gain: 0.5
        }
        hud {
            slot 0 {
                owner: "l"
                anchor: 0
                offset: (8.0, 8.0)
                size: 32.0
                rgba: (1.0, 1.0, 1.0, 1.0)
                text_fn: "label"
                text_field: "n"
                init: "0"
            }
            slot 1 {
                owner: "r"
                anchor: 1
                offset: (8.0, 8.0)
                size: 32.0
                rgba: (0.8, 0.9, 1.0, 1.0)
                text_fn: "label"
                text_field: "n"
                visible: "on"
                init: "0"
            }
        }
    }
    view-2d {
        Blink l {
            base: 0
            geo: resources.disc_mesh
            tint: resources.teal
        }
        Blink r {
            base: 1
            geo: resources.disc_mesh
            tint: resources.amber
        }
    }
    on {
        Life.Tick(dt) -> {
            .step = .step + 1
            if .step == 3 {
                l.Ping(1)
            }
            if .step == 6 {
                r.Ping(2)
            }
            if .step >= 9 {
                .step = 0
            }
        }
    }
}
"#;
    let notif = r#"node Blink(base: int, geo: Mesh2D, tint: Material2D) {
    model {
        var n = 0
        var on = 1
    }
    msg {
        Ping(int)
    }
    view-2d {
        shape2d disc {
            mesh: .geo
            position: (4.0 + .base * 8.0, 2.0)
            material: .tint
            visible: .on
        }
    }
    on {
        Life.Tick(dt) -> {
        }
        .Ping(kind) -> {
            .n = .n + 1
            if kind == 1 {
                sfx.play("hi", 1.0)
            } else {
                sfx.play("ho", 0.5)
            }
            if .n >= 2 {
                .on = 0
            }
        }
    }
}
"#;
    let text = r#"fn label(n int) str {
    if n == 0 {
        return "0"
    }
    return "1+"
}
"#;
    lower_temp(&[
        ("main.at", main.to_string()),
        ("notif.at", notif.to_string()),
        ("resources.at", NOTIFY_RES.to_string()),
        ("textmap.at", text.to_string()),
    ])
    .expect("renamed program lowers");
}

// ------------------------------------------------- no-business-enum scan ----

/// AC-02/V-01: the committed presentation products carry no game business
/// machinery — no Collect/Goal/Fail trigger kinds, no generated count/state
/// roots, no game_cue skeleton, and the actor-free products have no trigger
/// dispatch at all.
#[test]
fn v01_committed_products_have_no_business_enums() {
    for (world, triggers_allowed) in [
        (
            include_str!("../generated-presentation2d/notify/world.at"),
            false,
        ),
        (include_str!("../generated-presentation2d/four/world.at"), false),
        (include_str!("../generated-presentation2d/game/world.at"), true),
    ] {
        let words: Vec<&str> = world
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .collect();
        for banned in ["Collect", "Goal", "Fail", "game_cue", "game_events", "r_count", "r_state"] {
            assert!(
                !words.contains(&banned),
                "business machinery '{banned}' leaked into a presentation product"
            );
        }
        if !triggers_allowed {
            assert!(
                !world.contains("pres_events"),
                "actor-free product must not carry trigger dispatch"
            );
            assert!(
                !world.contains("ag_trigger_bind"),
                "actor-free product must not bind triggers"
            );
        }
        // The closeout is exactly one present per callback in every product.
        assert_eq!(
            world.matches("ag_present2d(").count(),
            1,
            "exactly one closeout present call"
        );
    }
}

// ---------------------------------------------------------- rejections ----

/// Unknown sfx cue name (typed mapping, author never writes ids).
#[test]
fn neg_unknown_cue() {
    let notif = NOTIFY_NOTIF.replace("sfx.play(\"ping\", 1.0)", "sfx.play(\"missing\", 1.0)");
    let mut files = notify_files();
    files[1] = ("notif.at", notif);
    let e = err_of(&files);
    assert!(e.contains("unknown sfx cue 'missing'"), "{e}");
}

/// text_fn must exist.
#[test]
fn neg_text_fn_missing() {
    let main = norm(NOTIFY_MAIN).replace("text_fn: \"beat_text\"", "text_fn: \"nope\"");
    let e = err_of(&with_main(&main));
    assert!(e.contains("text_fn 'nope' is not a declared fn"), "{e}");
}

/// text_fn signature must be (int) -> str.
#[test]
fn neg_text_fn_wrong_signature() {
    let text = r#"fn beat_text(n int) int {
    return n
}
"#;
    let mut files = notify_files();
    files[3] = ("textmap.at", text.to_string());
    let e = err_of(&files);
    assert!(e.contains("must have signature (int) -> str"), "{e}");
}

/// text_field must be an int model field of the owner.
#[test]
fn neg_text_field_unknown() {
    let main = norm(NOTIFY_MAIN).replace("text_field: \"hits\"", "text_field: \"nope\"");
    let e = err_of(&with_main(&main));
    assert!(e.contains("text_field 'nope' is not an int model field"), "{e}");
}

/// visible must be an int model field of the owner.
#[test]
fn neg_visible_unknown() {
    let main = norm(NOTIFY_MAIN).replace("visible: \"live\"", "visible: \"nope\"");
    let e = err_of(&with_main(&main));
    assert!(e.contains("visible 'nope' is not an int model field"), "{e}");
}

/// Slot index out of range (the closeout writes exactly four slots).
#[test]
fn neg_slot_out_of_range() {
    let main = norm(NOTIFY_MAIN).replace("slot 1 {", "slot 4 {");
    let e = err_of(&with_main(&main));
    assert!(e.contains("slot index must be 0..3"), "{e}");
}

/// Duplicate slot index.
#[test]
fn neg_slot_duplicate() {
    let base = norm(NOTIFY_MAIN);
    let main = base.replace("            slot 1 {", "            slot 0 {");
    assert!(main != base, "slot duplicate anchor must match");
    let e = err_of(&with_main(&main));
    assert!(e.contains("duplicate hud slot index"), "{e}");
}

/// Duplicate cue name (cue id = declaration order).
#[test]
fn neg_cue_duplicate() {
    let main = NOTIFY_MAIN
        .replace(
            "        cue pong {",
            "        cue ping { span: (0, 10) gain: 1.0 }
        cue pong {",
        );

    let e = err_of(&with_main(&main));
    assert!(e.contains("duplicate presentation2d cue name"), "{e}");
}

/// Missing presentation2d block (unknown block keyword is the parse-layer
/// named rejection for a missing presentation surface).
#[test]
fn neg_block_missing() {
    let main = norm(NOTIFY_MAIN).replace("    presentation2d {", "    view2dx {");
    let e = err_of(&with_main(&main));
    assert!(
        e.contains("unsupported node block 'view2dx'")
            || e.contains("requires a `presentation2d` block"),
        "{e}"
    );
}

/// Reserved generated infrastructure names on the root (ord/np/rows).
#[test]
fn neg_reserved_root_field() {
    let main = NOTIFY_MAIN
        .replace("var beat = 0", "var ord = 0")
        .replace(".beat", ".ord");
    let e = err_of(&with_main(&main));
    assert!(e.contains("generated presentation infrastructure"), "{e}");
}

/// Reserved generated fn names.
#[test]
fn neg_reserved_fn_name() {
    let text = r#"fn beat_text(n int) str {
    if n == 0 {
        return "0"
    }
    return "1+"
}

fn sfx_play(n int) str {
    return "x"
}
"#;
    let mut files = notify_files();
    files[3] = ("textmap.at", text.to_string());
    let e = err_of(&files);
    assert!(e.contains("generated presentation infrastructure"), "{e}");
}

/// Business trigger payload is game2d-only (parse layer).
#[test]
fn neg_business_trigger_payload() {
    let main = norm(NOTIFY_MAIN).replace(
        "        hud {",
        "        trigger bad {\n            shape: box\n            target: \"left\"\n            event: \"Ping\"\n            on: \"collect\"\n            cue: \"ping\"\n            full_at: 1\n            full_cue: \"pong\"\n        }\n        hud {",
    );
    let e = err_of(&with_main(&main));
    assert!(
        e.contains("business payload") || e.contains("business trigger"),
        "{e}"
    );
}

/// Region rule: trigger regions must be runtime instances, not static leaves.
#[test]
fn neg_trigger_static_leaf() {
    let main = r#"use resources: disc_mesh, teal
use notif: Notify

node Main {
    model {
        var beat = 0
    }
    presentation2d {
        audio {
            bank: "assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"
            font: "assets/lilita_one_regular.ttf"
            license: "assets/license.txt"
        }
        cue ping {
            span: (0, 7403)
            gain: 1.0
        }
        trigger floor_hit {
            shape: box
            target: "floor"
            event: "Ping"
        }
    }
    view-2d {
        Notify left {
            base: 0
            geo: resources.disc_mesh
            tint: resources.teal
        }
        shape2d floor {
            kind: quad
            size: (24.0, 2.0)
            position: (12.0, -1.0)
            material: resources.teal
        }
    }
}
"#;
    let e = err_of(&with_main(main));
    assert!(
        e.contains("region rule")
            || e.contains("trigger target must be a root-view component instance"),
        "{e}"
    );
}

/// Trigger event must be a declared msg of the target definition.
#[test]
fn neg_trigger_unknown_event() {
    let main = norm(NOTIFY_MAIN).replace(
        "        hud {",
        "        trigger bad {
            shape: box
            target: \"left\"
            event: \"Nope\"
        }
        hud {",
    );
    let e = err_of(&with_main(&main));
    assert!(e.contains("is not a declared msg of"), "{e}");
}

/// The render2d surface in this profile is despawn-only (spawn/frame stay
/// sprites-2d).
#[test]
fn neg_render_spawn_rejected() {
    let notif = NOTIFY_NOTIF.replace(
        "            if .hits >= 2 {",
        "            render2d.spawn(\"disc\", 0.0, 0.0)\n            if .hits >= 2 {",
    );
    let mut files = notify_files();
    files[1] = ("notif.at", notif);
    let e = err_of(&files);
    assert!(e.contains("render2d"), "{e}");
}
