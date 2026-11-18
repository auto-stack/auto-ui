//! PLAN-058 T-04: the UI-side segment/sweep/image2d arms against the landed
//! record ABI (`D-981-GFX-PORT-ABI`; auto-lang `5ee4d23c0`, receipt `381d7a364`).
//!
//! Values are the fixture's own numbers (`fixtures/plan058/topdown-node/main.at`
//! rows: ground 0 render-only, player 1, walls 2/3, targets 4/5), checked
//! element-by-element against `AGE_PORT_FIELDS` order. Charging lives in the 981
//! `AgeGfxSession`; these arms only execute and are therefore never metered.
use autographics_auto_driver::authoring::{
    document::Layer,
    gfx_host_arms::{
        GfxArg, GfxFault, GfxReturn, GfxValue, HostRow, NeutralGfxHost, IMAGE_SYMBOL, SEGMENT_SYMBOL,
        SWEEP_SYMBOL,
    },
};
use sha2::{Digest, Sha256};
use std::time::Instant;

const ASSET: &str = "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png";

fn rows() -> Vec<HostRow> {
    vec![
        HostRow {
            ordinal: 0,
            world: [8.0, 0.0],
            half: [6.0, 4.0],
            layer: Layer::RenderOnly,
            visible: true,
        },
        HostRow {
            ordinal: 1,
            world: [4.0, 0.0],
            half: [0.4, 0.5],
            layer: Layer::Collision,
            visible: true,
        },
        HostRow {
            ordinal: 2,
            world: [10.0, 0.0],
            half: [1.0, 3.0],
            layer: Layer::Collision,
            visible: true,
        },
        HostRow {
            ordinal: 3,
            world: [4.0, -2.5],
            half: [1.0, 1.0],
            layer: Layer::Collision,
            visible: true,
        },
        HostRow {
            ordinal: 4,
            world: [4.0, 2.0],
            half: [0.5, 0.5],
            layer: Layer::Collision,
            visible: true,
        },
        HostRow {
            ordinal: 5,
            world: [4.0, 3.25],
            half: [0.5, 0.5],
            layer: Layer::Collision,
            visible: true,
        },
    ]
}

fn assets_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../autographics-node2d-pilot/fixtures/sprites2d/assets")
}

fn fixture_host() -> NeutralGfxHost {
    NeutralGfxHost::new(rows(), assets_root())
}

#[test]
fn segment_matches_the_abi_record_and_the_fixture_geometry() {
    let t0 = Instant::now();
    let mut host = fixture_host();
    let c = host
        .segment(1, true, 4.0, 0.0, 4.0, 6.0)
        .expect("segment executes");
    assert!(c.hit);
    assert_eq!(c.node, 4, "target_a (ordinal 4) is nearest");
    assert_eq!(c.t, 0.25, "bare box, no Minkowski: t=0.25 exactly");
    assert_eq!((c.point_x, c.point_y), (4.0, 1.5));
    assert_eq!((c.normal_x, c.normal_y), (0.0, -1.0), "bottom face normal");
    // Record order: hit, node, t, point_x, point_y, normal_x, normal_y.
    let record = host
        .dispatch(
            SEGMENT_SYMBOL,
            &[
                GfxArg::I32(1),
                GfxArg::Bool(true),
                GfxArg::F32(4.0),
                GfxArg::F32(0.0),
                GfxArg::F32(4.0),
                GfxArg::F32(6.0),
            ],
        )
        .expect("dispatch");
    assert_eq!(
        record,
        GfxReturn::Record(vec![
            GfxValue::Bool(true),
            GfxValue::I32(4),
            GfxValue::F32(0.25),
            GfxValue::F32(4.0),
            GfxValue::F32(1.5),
            GfxValue::F32(0.0),
            GfxValue::F32(-1.0),
        ]),
        "the record is AGE_PORT_FIELDS-ordered and exact"
    );
    match record {
        GfxReturn::Record(fields) => assert_eq!(fields.len(), 7),
        GfxReturn::Scalar(_) => panic!("segment returns a record"),
    }
    // Miss: node -1, t 1, point/normal 0.
    let miss = host.segment(1, true, 4.0, 0.0, 4.0, 1.0).expect("miss");
    assert!(!miss.hit);
    assert_eq!((miss.node, miss.t), (-1, 1.0));
    assert_eq!((miss.point_x, miss.point_y, miss.normal_x, miss.normal_y), (0.0, 0.0, 0.0, 0.0));
    println!(
        "PLAN058_TIMING test=segment_matches_the_abi_record_and_the_fixture_geometry ms={} node=4 t=0.25 record=7",
        t0.elapsed().as_millis()
    );
}

#[test]
fn sweep_matches_the_abi_record_minkowski_and_start_overlap() {
    let t0 = Instant::now();
    let mut moved = rows();
    moved[1].world = [8.533333, 0.0];
    let mut host = NeutralGfxHost::new(moved, assets_root());
    let s = host
        .sweep(1, true, 0.13333334, 0.0)
        .expect("sweep executes");
    assert!(s.hit);
    assert_eq!(s.node, 2, "the wall in the corridor");
    assert!(
        (s.t - 0.5).abs() <= 1e-4,
        "Minkowski entry t≈0.5, got {}",
        s.t
    );
    assert_eq!((s.origin_x, s.origin_y), (8.533333, 0.0), "origin = mover world");
    assert_eq!((s.normal_x, s.normal_y), (-1.0, 0.0), "left face normal");
    assert!(!s.start_overlap);
    // Record order: hit, node, t, origin_x, origin_y, normal_x, normal_y,
    // start_overlap.
    let record = host
        .dispatch(
            SWEEP_SYMBOL,
            &[
                GfxArg::I32(1),
                GfxArg::Bool(true),
                GfxArg::F32(0.13333334),
                GfxArg::F32(0.0),
            ],
        )
        .expect("dispatch");
    match &record {
        GfxReturn::Record(fields) => {
            assert_eq!(fields.len(), 8, "Sweep2D has 8 port fields");
            assert_eq!(fields[0], GfxValue::Bool(true));
            assert_eq!(fields[1], GfxValue::I32(2));
            match fields[2] {
                GfxValue::F32(t) => assert!((t - 0.5).abs() <= 1e-4),
                ref other => panic!("t is f32: {other:?}"),
            }
            assert_eq!(fields[3], GfxValue::F32(8.533333));
            assert_eq!(fields[4], GfxValue::F32(0.0));
            assert_eq!(fields[5], GfxValue::F32(-1.0));
            assert_eq!(fields[6], GfxValue::F32(0.0));
            assert_eq!(fields[7], GfxValue::Bool(false));
        }
        GfxReturn::Scalar(_) => panic!("sweep returns a record"),
    }
    // Inside the expanded wall box: t=0 and start_overlap, lowest ordinal.
    let mut inside = rows();
    inside[1].world = [9.5, 0.0];
    let mut host = NeutralGfxHost::new(inside, assets_root());
    let s = host
        .sweep(1, true, 1.0, 0.0)
        .expect("inside sweep executes");
    assert!(s.hit && s.node == 2, "the containing wall, not a farther row");
    assert_eq!(s.t, 0.0, "start overlap is t=0");
    assert!(s.start_overlap);
    assert_eq!((s.normal_x, s.normal_y), (0.0, 0.0), "no hit face");
    // Miss keeps the origin and t=1.
    let miss = host.sweep(1, true, -1.0, 0.0).expect("away sweep");
    assert!(
        !miss.hit || miss.node != 2,
        "moving away cannot enter the wall behind the origin"
    );
    assert_eq!((miss.origin_x, miss.origin_y), (9.5, 0.0));
    println!(
        "PLAN058_TIMING test=sweep_matches_the_abi_record_minkowski_and_start_overlap ms={} node=2 t≈0.5 inside=(t0,overlap) record=8",
        t0.elapsed().as_millis()
    );
}

#[test]
fn ground_layer_decides_candidacy_and_render_only_mover_is_named() {
    let t0 = Instant::now();
    let mut host = fixture_host();
    // Positive: the render-only ground is never hit or selected.
    let c = host.segment(1, true, 4.0, 0.0, 4.0, 6.0).expect("segment");
    assert_ne!(c.node, 0, "the ground is not a candidate");
    // Admit the ground as a collision row: both arms hit it at t=0/overlap.
    let mut admitted = rows();
    admitted[0].layer = Layer::Collision;
    let mut host = NeutralGfxHost::new(admitted, assets_root());
    let c = host.segment(1, true, 4.0, 0.0, 4.0, 6.0).expect("segment");
    assert!(c.hit);
    assert_eq!((c.node, c.t), (0, 0.0), "the ground wins with t=0 (055)");
    let s = host.sweep(1, true, 0.1, 0.0).expect("sweep");
    assert_eq!((s.node, s.t), (0, 0.0), "sweep start overlap on the ground");
    assert!(s.start_overlap);
    // A render-only mover is a named refusal.
    let mut host = fixture_host();
    let err = host.sweep(0, true, 0.1, 0.0).expect_err("render-only mover");
    assert_fault(&err, "age-gfx.render-only-mover");
    let err = host
        .segment(0, true, 4.0, 0.0, 4.0, 6.0)
        .expect_err("render-only mover");
    assert_fault(&err, "age-gfx.render-only-mover");
    // Out-of-range rows are named too.
    assert_fault(&host.sweep(9, true, 1.0, 0.0).expect_err("row 9"), "age-gfx.bad-row");
    assert_fault(
        &host.segment(-1, true, 0.0, 0.0, 1.0, 1.0).expect_err("row -1"),
        "age-gfx.bad-row",
    );
    println!(
        "PLAN058_TIMING test=ground_layer_decides_candidacy_and_render_only_mover_is_named ms={} positive=(node4) admitted=(node0,t0,overlap) faults=render-only-mover/bad-row",
        t0.elapsed().as_millis()
    );
}

fn assert_fault(fault: &GfxFault, code: &str) {
    assert_eq!(fault.code, code, "fault code: {}", fault.message);
}

#[test]
fn image2d_record_is_content_identity_and_image_faults_are_named() {
    let t0 = Instant::now();
    let mut host = fixture_host();
    let image = host.image_png(ASSET).expect("image2d.png");
    let bytes = std::fs::read(assets_root().join(ASSET)).expect("fixture png");
    let want_sha = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(image.sha256, want_sha, "sha256 is the full content identity");
    assert_eq!(image.byte_length, bytes.len() as i32);
    // Independent IHDR read via the png dev-dependency.
    let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    let reader = decoder.read_info().expect("png info");
    let info = reader.info();
    assert_eq!((image.width, image.height), (96, 32));
    assert_eq!((image.width as u32, info.width), (96, 96), "both widths come from IHDR");
    assert_eq!(image.height as u32, info.height);
    // Record order: sha256 Str, byte_length i32, width i32, height i32.
    let record = host
        .dispatch(IMAGE_SYMBOL, &[GfxArg::Str(ASSET.into())])
        .expect("dispatch");
    assert_eq!(
        record,
        GfxReturn::Record(vec![
            GfxValue::Str(want_sha.clone()),
            GfxValue::I32(bytes.len() as i32),
            GfxValue::I32(96),
            GfxValue::I32(32),
        ])
    );
    // Faults: missing, path escape (relative and absolute), bad magic.
    let missing = host.image_png("nope.png").expect_err("missing");
    assert_fault(&missing, "age-gfx.image-missing");
    for escape in ["../assets/x.png", "..\\x.png", "C:\\windows\\x.png", "/etc/x.png"] {
        let err = host.image_png(escape).expect_err("escape");
        assert_fault(&err, "age-gfx.image-path");
    }
    let junk_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan058-t04-arms")
        .join(format!("{}", std::process::id()));
    std::fs::create_dir_all(&junk_dir).expect("junk dir");
    std::fs::write(junk_dir.join("junk.png"), b"not a png at all").expect("junk png");
    let mut junk = NeutralGfxHost::new(rows(), &junk_dir);
    let err = junk.image_png("junk.png").expect_err("bad magic");
    assert_fault(&err, "age-gfx.image-bad-png");
    println!(
        "PLAN058_TIMING test=image2d_record_is_content_identity_and_image_faults_are_named ms={} sha={} len={} 96x32 faults=missing/path/bad-png",
        t0.elapsed().as_millis(),
        &want_sha[..16],
        bytes.len()
    );
}

#[test]
fn dispatch_refuses_unknown_symbols_and_bad_arguments() {
    let t0 = Instant::now();
    let mut host = fixture_host();
    assert_fault(
        &host.dispatch("age.row.place.v1", &[GfxArg::I32(1)]).expect_err("scalar op"),
        "age-gfx.unknown-op",
    );
    assert_fault(
        &host.dispatch("age.nope.v1", &[]).expect_err("unknown"),
        "age-gfx.unknown-op",
    );
    // Wrong arity / wrong variant / non-finite endpoint.
    assert_fault(
        &host
            .dispatch(
                SEGMENT_SYMBOL,
                &[GfxArg::I32(1), GfxArg::Bool(true), GfxArg::F32(4.0), GfxArg::F32(0.0), GfxArg::F32(4.0)]
                    .to_vec(),
            )
            .expect_err("5 args"),
        "age-gfx.bad-args",
    );
    assert_fault(
        &host
            .dispatch(
                SEGMENT_SYMBOL,
                &[
                    GfxArg::I32(1),
                    GfxArg::I32(1),
                    GfxArg::F32(4.0),
                    GfxArg::F32(0.0),
                    GfxArg::F32(4.0),
                    GfxArg::F32(6.0),
                ],
            )
            .expect_err("visible_only as i32"),
        "age-gfx.bad-args",
    );
    assert_fault(
        &host
            .dispatch(SWEEP_SYMBOL, &[GfxArg::I32(1), GfxArg::Bool(true), GfxArg::F32(f32::NAN), GfxArg::F32(0.0)])
            .expect_err("NaN"),
        "age-gfx.bad-args",
    );
    assert_fault(
        &host
            .dispatch(IMAGE_SYMBOL, &[GfxArg::I32(1)])
            .expect_err("image arg as i32"),
        "age-gfx.bad-args",
    );
    println!(
        "PLAN058_TIMING test=dispatch_refuses_unknown_symbols_and_bad_arguments ms={} faults=unknown-op/bad-args",
        t0.elapsed().as_millis()
    );
}

#[test]
fn arms_do_not_charge_and_visible_only_filters_rows() {
    let t0 = Instant::now();
    let mut host = fixture_host();
    // The 981 AgeGfxSession owns the 32-credit pool; these arms never meter, so
    // repeating a paid op many times cannot fault here.
    for _ in 0..10 {
        assert!(host.segment(1, true, 4.0, 0.0, 4.0, 6.0).is_ok());
        assert!(host.sweep(1, true, 0.1, 0.0).is_ok());
    }
    // visible_only skips an invisible nearest target: target_a hidden -> node 5.
    let mut hidden = rows();
    hidden[4].visible = false;
    let host = NeutralGfxHost::new(hidden, assets_root());
    let c = host.segment(1, true, 4.0, 0.0, 4.0, 6.0).expect("segment");
    assert_eq!(c.node, 5, "the far target, because the near one is invisible");
    // Without the filter the invisible row is a legal candidate again.
    let c = host.segment(1, false, 4.0, 0.0, 4.0, 6.0).expect("segment");
    assert_eq!(c.node, 4);
    println!(
        "PLAN058_TIMING test=arms_do_not_charge_and_visible_only_filters_rows ms={} repeats=10 visible_only=5/no-filter=4",
        t0.elapsed().as_millis()
    );
}
