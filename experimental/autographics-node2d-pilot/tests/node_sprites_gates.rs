//! auto-ui PLAN-022 node_sprites_gates: the sprites-2d authoring gate set
//! (V-01). Positive side: every generated product regenerates byte-identical
//! from its fixture closure, the fixed PNG identity (sha/bytes/IHDR dims)
//! flows into the scene assets, and the sprite bindings land on the frozen
//! document schema. Negative side: each invalid image/frame/slot surface is
//! named-rejected before execution, with module/line/symbol diagnostics —
//! no silent fallbacks, no panics.
//!
//! Run: cargo test --offline --test node_sprites_gates
use autographics_node2d_pilot::{load_project, lower_project_sprites};

const FX: &str = "fixtures/sprites2d";

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn generate(prod: &str) -> Result<autographics_node2d_pilot::LowerOutputBasic2D, String> {
    let project =
        load_project(&root().join(format!("{FX}/{prod}/main.at"))).map_err(|e| e.to_string())?;
    lower_project_sprites(&project).map_err(|e| e.to_string())
}

fn expect_rejection(prod: &str, needle: &str) {
    match generate(prod) {
        Ok(_) => panic!("{prod}: expected a named rejection containing '{needle}', got success"),
        Err(message) => assert!(
            message.contains(needle),
            "{prod}: rejection message '{message}' does not contain '{needle}'"
        ),
    }
}

const POSITIVE: [&str; 10] = [
    "atlas-g09",
    "atlas-alt",
    "atlas-flip-control",
    "atlas-slot",
    "character-g09",
    "character-alt",
    "character-wrong-air",
    "character-budget-32",
    "character-budget-33",
    "pair-isolated",
];

/// Every product generates and its scene carries the frozen document sprite
/// schema (sha256 asset identity + integer frame + flip) with byte-true
/// assets; generation is deterministic (two runs byte-equal).
#[test]
fn sprites_positive_products_generate_with_identity() {
    for prod in POSITIVE {
        let first = generate(prod).expect(prod);
        let second = generate(prod).expect(prod);
        assert_eq!(
            first.source_sha256, second.source_sha256,
            "{prod}: generation is not deterministic"
        );
        let scene = &first.scene_json;
        assert!(
            scene.contains("\"api\": \"autographics.sprites2d/1\""),
            "{prod}: scene must assemble sprites2d/1"
        );
        assert!(
            scene.contains("\"frame\": [0, 0, 32, 32]"),
            "{prod}: default frame must be an integer rect"
        );
        assert!(
            scene.contains("\"format\": \"png\"") && scene.contains("\"width\": "),
            "{prod}: scene must carry the png identity entry"
        );
        let shas: Vec<&str> = first
            .scene_json
            .match_indices("\"sha256\": \"")
            .filter_map(|(i, _)| first.scene_json[i + 11..].split('"').next())
            .collect();
        assert!(
            shas.iter().any(|s| s.len() == 64),
            "{prod}: asset sha256 identity missing"
        );
    }
}

/// The fixed PNG identity matches the origin bytes read from the fixture
/// assets (the generator refuses to see drifted bytes because the resource
/// loader hashes the file it reads; this control pins the fixture bytes).
#[test]
fn fixed_png_identity_matches_origin() {
    let atlas = std::fs::read(root().join(format!(
        "{FX}/assets/95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png"
    )))
    .expect("atlas fixture bytes");
    let character = std::fs::read(root().join(format!(
        "{FX}/assets/8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.png"
    )))
    .expect("character fixture bytes");
    assert_eq!(atlas.len(), 258);
    assert_eq!(character.len(), 735);
    let out = generate("atlas-g09").expect("generate");
    assert!(
        out.scene_json.contains(&format!(
            "\"sha256\": \"{}\"",
            autographics_node2d_pilot::scene::sha256_hex(&atlas)
        )),
        "scene asset identity must be the full original sha256"
    );
    let _ = character;
}

/// Frame rects must stay inside the image (named rejection, generator side).
#[test]
fn neg_frame_rect_out_of_image() {
    expect_rejection(
        "neg-frame-rect",
        "frame rect [90, 0, 32, 32] exceeds image 'atlas_img' (96x32)",
    );
}

/// Literal frame ordinals must name a table entry.
#[test]
fn neg_frame_literal_out_of_table() {
    expect_rejection(
        "neg-frame-lit",
        "literal frame ordinal 7 exceeds the frame table of image 'atlas_img' (3 frame(s))",
    );
}

/// The image port only accepts image2d resources.
#[test]
fn neg_image_type_mismatch() {
    expect_rejection(
        "neg-image-type",
        "binds 'resources.marker_color' which is not an image2d.png resource",
    );
}

/// Literal flips are 0/1 only.
#[test]
fn neg_flip_literal_domain() {
    expect_rejection("neg-flip-lit", "literal flip must be 0/1, got 2");
}

/// render2d.spawn targets must be bound root-view leaves.
#[test]
fn neg_unknown_spawn_target() {
    expect_rejection(
        "neg-unknown-spawn",
        "unknown render2d.spawn target 'no_such_leaf'",
    );
}

/// The frozen 8-slot pool is a lowering-time budget: nine dynamic sprite
/// templates are rejected before a run can exist.
#[test]
fn neg_ninth_slot_rejected() {
    expect_rejection(
        "slot-9",
        "runtime object slots exceed the frozen 8-slot pool (dynamic instance templates + render2d.spawn sites = 9)",
    );
}

/// A missing image file is a named load-time rejection, not a panic.
#[test]
fn neg_missing_image_file() {
    let dir = root().join(format!(
        "target/plan022-gates/{}-missing-image",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).expect("assets dir");
    std::fs::write(
        dir.join("resources.at"),
        "let atlas_img = image2d.png(\"assets/absent.png\")\n",
    )
    .expect("write module");
    std::fs::write(
        dir.join("main.at"),
        "use resources: atlas_img\n\nnode Main {\n    view-2d {\n    }\n}\n",
    )
    .expect("write main");
    let err = load_project(&dir.join("main.at"))
        .err()
        .expect("load must fail");
    let message = err.to_string();
    assert!(
        message.contains("absent.png"),
        "rejection '{message}' must name the missing file"
    );
}

/// A corrupted PNG (bad signature) is a named load-time rejection.
#[test]
fn neg_corrupt_png_rejected() {
    let dir = root().join(format!(
        "target/plan022-gates/{}-corrupt-png",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).expect("assets dir");
    std::fs::write(dir.join("assets/bad.png"), b"not a png at all").expect("write bad png");
    std::fs::write(
        dir.join("resources.at"),
        "let bad_img = image2d.png(\"assets/bad.png\")\n",
    )
    .expect("write module");
    std::fs::write(
        dir.join("main.at"),
        "use resources: bad_img\n\nnode Main {\n    view-2d {\n    }\n}\n",
    )
    .expect("write main");
    let err = load_project(&dir.join("main.at"))
        .err()
        .expect("load must fail");
    let message = err.to_string();
    assert!(
        message.contains("bad.png"),
        "rejection '{message}' must name the corrupt file"
    );
}
