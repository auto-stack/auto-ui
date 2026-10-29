//! PLAN-004 public sprite frames: profile admission, byte-identical older
//! profiles, the real-Auto CPU feasibility probe (T-02) and the named host
//! admission controls of the sprite block. The GPU presentation cases live in
//! the driver's sprite host test and the AGP `sprite_resources` test.
//!
//! Run single-threaded: `cargo test --locked --test public_sprite_frames -- --test-threads=1`
use agp::assets::encode_png_rgba;
use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::authoring::{
    assets,
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object, SpriteBinding},
    hierarchy,
    host::Host,
    play::{Action, Play, State},
    runtime_objects::{self, Dim, RuntimeObjectsFrame, Slot},
    runtime_sprites::{self, Target},
    session::{Edit, Editor},
    spatial::Descriptor,
    storage::{Fault, Saved, Storage},
};
use autographics_auto_driver::Scalar;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const USER: &[u8] = b"fn app_start() {\n}\nfn app_update(dt float) {\n}\n";

/// The assembled managed bytes of every pre-PLAN-004 profile, recorded on the
/// pristine `codex/plan-004-dev` base (d3bd6b4) before any source edit. The
/// new profile must not move a single byte of these.
const BASELINE: [(&str, Api, &str, &str, &str); 13] = [
    (
        "autographics.behavior/1",
        Api::V1,
        "75127ed7cf1502a72be5ae4621f431d90c98236690b8f76d254349512650233f",
        "343f34a27c62f58980ef9df34a009341b20ba47d5a1b38cbbde7a1c3d499939f",
        "c301a80f047d78866415147e5251af22018e1510548126a5f0352841d46ebc1d",
    ),
    (
        "autographics.behavior/2",
        Api::SpatialV2,
        "fe88c042cbb4dccd6b1ba60a7ac9b0eae392b72635d109433eeaafda179f68c6",
        "32c0412a2a662a920f3394ac5f176bfa66ac9a5b487b77c60014b9659f9ba63c",
        "79161a8e705cfd87abc0f45c158bc64ce070b1ffec63d5c266760aaac055801d",
    ),
    (
        "autographics.behavior/3",
        Api::SegmentV3,
        "c5307731cc1c0c659f20ea0012223088e68d58c1d7fd2adbdf5069c62e32c2f4",
        "560e22eaa3da6998d747f6ec0372a40f975d9dd75fdaf24c9f24de59a1623c37",
        "a175f670601cb0feadfa9e252ae4f77010bc0f08c93155502397abae7b925e27",
    ),
    (
        "autographics.behavior/4",
        Api::SweepV4,
        "f4e96c8e01edaecc0d53aee591d56ec4f0036dd3893036611d4fd855b8d5499e",
        "0e1e554f87302d15fb0295bed30f3a99dea4595edb1d67860703a4021d2fad7e",
        "b0b5e49fd8234506f119687535b98bcd7c4e687194b1104d9bdd9664aa8bdb3d",
    ),
    (
        "autographics.behavior/5",
        Api::MotionV5,
        "82e744dbee5899615db18d33b5479fdd6458ccc172abaee43ddcce9e7dbf8b5c",
        "a0c4fce2e235cc6484914b25634e9c0b7d636a5a6991019262a812564de05b0d",
        "9c7c68e60ae815432ed14c85e0568fd977ad26f6faf57b46b5221e288ec6eaf6",
    ),
    (
        "autographics.character/1",
        Api::CharacterV1,
        "52ca5439f480b23f158015f04973f2a1bb9ee4f52757aeb3bebc9683c83fd647",
        "2b4631e371623dc51a0b9977cf0c7f59deb944408e4acde051599c36cfcc84ac",
        "320a9b90b24fad77b71720f88fc134624557ff26cfca96ff9ee58e340dc1cd97",
    ),
    (
        "autographics.character3d/1",
        Api::Character3V1,
        "eefad1ca50c7917a800cafecdec354862e7372447eb5e7756664a9213d683fbc",
        "3ae0b1bad472bf0845bb167476f43adb7dfc77ac499f27b12939d76db5152a39",
        "09b9d215b237e1f77dc5c4a1628ffbd40995d39d0c6765602b6b303857900f22",
    ),
    (
        "autographics.runtime_camera2d/1",
        Api::Camera2V1,
        "545434f2b334202360e728c2b5ac0402b08c2764773271602d94f13b05d6bf67",
        "07765d0bd6ec533da2d4f4469112a767957f0bd4ca8174716fd8950c4f8e588c",
        "71dc7ab14f581ffa5f1b299728b8b9b8fa0484025f6a27225e54aa4a92843eba",
    ),
    (
        "autographics.runtime_camera3d/1",
        Api::Camera3V1,
        "b9763f94d830746db9304ef20f0e816eb67ee93e31406e05a9ecc8fc6f8e14ec",
        "2d92247f9146ee41188b8756d22d747fd8b51e698becb34a4afaab986c59d38f",
        "3a51fb037fb267c56808315645f6f5d7f59c928b2d62fc8e03c1c89a85869557",
    ),
    (
        "autographics.objects2d/1",
        Api::Objects2V1,
        "08d83c162b20f3a96a49f0a1ffd78f18cfbd1210cd103d30230fa6b7f19459e2",
        "2846d0f3284e0fdb32b57309350eb47d2a3f64260f99b6a7bc89f228fd94b8fd",
        "18a0a3028e36d2f1f3862268c15a606cc42f044c19c651a7351ff7074cbacec5",
    ),
    (
        "autographics.objects3d/1",
        Api::Objects3V1,
        "28bc3ad12746072ff79c697186040df2cc2349f50f71ddd9eebc667d4b6c169b",
        "920f0e9f0513e31db27bd1486e1aea6582dc3ea1c08f9f5b138e51bbf8d31148",
        "b5b55d72667a04ebc6bc0f0393093c12c76831467bd4d80e16d89d1e2dbb7261",
    ),
    (
        "autographics.triggers2d/1",
        Api::Triggers2V1,
        "c51700b106466b43dfc757fa5b1fb6c3ca1d6ac75ac2cd56e0fcf12b8a2652aa",
        "4df4e49f2b11ee6f0ee71869e63954ad19664265284ff900a598f70904a1db46",
        "8441914f01cbc7fb3bdb1bfafe712dfdbb843863e05a33a0bf35af9cf205f6a4",
    ),
    (
        "autographics.triggers3d/1",
        Api::Triggers3V1,
        "7673e061cabb4e4c82d0a5c871931f6ee1155230bd90bcec74b4711c82c9c077",
        "b0efdd00fb0803eecbbb35680feaf7be78a87fdc65f89dce4bc8c71c8028c757",
        "584713670eeeddcc767ac2b3550c7ab54e5705081d159b3ad6119b5cbc00204a",
    ),
];

fn sample_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/sprite_frames")
}

#[test]
fn older_profiles_keep_their_exact_assembled_bytes_and_the_new_one_fits() {
    for (version, api, combined, api_hash, bridge_hash) in BASELINE {
        let s = SourceSnapshot::from_bytes_for(Path::new("user.at"), USER.to_vec(), api)
            .unwrap_or_else(|e| panic!("{version}: {e}"));
        assert_eq!(s.version, version);
        assert_eq!(s.combined_hash, combined, "{version} combined bytes moved");
        assert_eq!(s.api_hash, api_hash, "{version} api bytes moved");
        assert_eq!(s.bridge_hash, bridge_hash, "{version} bridge bytes moved");
    }
    let sprite =
        SourceSnapshot::from_bytes_for(Path::new("user.at"), USER.to_vec(), Api::Sprites2V1)
            .expect("sprites2d admits");
    assert_eq!(sprite.version, "autographics.sprites2d/1");
    assert_eq!(Api::Sprites2V1.number(), 109);
    assert_eq!(Api::Sprites2V1.profile(), 20);
    assert_eq!(sprite.regions[1].file, "<runtime_sprites_v1.at>");
    // The composed managed text (pre-namespace assembly + entry) must stay
    // inside the frozen 96KiB envelope; the plan records the measured number.
    let managed: usize = sprite
        .regions
        .iter()
        .take(2)
        .map(|r| r.end - r.start)
        .sum::<usize>()
        + sprite.regions.last().unwrap().end
        - sprite.regions.last().unwrap().start;
    println!(
        "sprite profile: managed={managed} api={} user={} combined={}",
        sprite.regions[1].end - sprite.regions[1].start,
        sprite.regions[2].end - sprite.regions[2].start,
        sprite.source.len()
    );
    assert!(
        managed < 96 * 1024,
        "sprites2d managed source {managed} exceeds the 96KiB envelope"
    );
    // The sprite profile is the triggers2d composition plus one unit: the whole
    // existing public surface stays addressable.
    for name in [
        "fn ag_char_move(",
        "fn ag_camera_follow_2d(",
        "fn ag_object_spawn(",
        "fn ag_trigger_bind(",
        "fn ag_sprite_frame(",
        "fn ag_sprite_slot(",
        "fn ag_sprite_bound(",
    ] {
        assert!(
            sprite.source.contains(name),
            "{name} missing from profile 20"
        );
    }
}

fn sprite_document(dir: &Path) -> (Document, assets::Registry) {
    let registry: assets::Registry = Default::default();
    let sheet = assets::import_png(&dir.join("sheet.png"), &registry).expect("sheet import");
    let mut registry = registry;
    assets::insert(&mut registry, sheet.clone()).expect("sheet registry");
    let mut doc = Document::empty("sprite-frames".into());
    doc.assets.push(sheet.identity.clone());
    let hero = {
        let mut o = Object::new(1, Kind::Quad);
        o.name = "hero".into();
        o.scale = [0.8, 1.0, 1.0];
        o.translation = [2.0, 1.0, 0.0];
        o.sprite = Some(SpriteBinding {
            asset: sheet.identity.key(),
            frame: [0, 0, 32, 32],
            flip: false,
        });
        o
    };
    let coin = {
        let mut o = Object::new(2, Kind::Circle);
        o.name = "coin".into();
        o.scale = [0.6, 0.6, 1.0];
        o.translation = [0.0, -20.0, 0.0];
        o.sprite = Some(SpriteBinding {
            asset: sheet.identity.key(),
            frame: [32, 0, 32, 32],
            flip: false,
        });
        o
    };
    let mut goal = Object::new(3, Kind::Quad);
    goal.name = "goal".into();
    goal.scale = [1.2, 1.2, 1.0];
    goal.translation = [0.0, -22.0, 0.0];
    let mut hazard = Object::new(4, Kind::Circle);
    hazard.name = "hazard".into();
    hazard.scale = [0.5, 0.5, 1.0];
    hazard.translation = [0.0, -24.0, 0.0];
    let mut floor = Object::new(5, Kind::Quad);
    floor.name = "floor".into();
    floor.scale = [40.0, 1.0, 1.0];
    floor.translation = [15.0, -1.5, 0.0];
    doc.objects = vec![hero, coin, goal, hazard, floor];
    doc.validate().expect("sprite document");
    (doc, registry)
}

#[test]
fn real_auto_drives_frames_flip_and_the_whole_2d_composition_in_one_tick() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let editor = Editor::with_project(doc.clone(), registry.clone(), None).expect("editor");
    let source = std::fs::read_to_string(dir.join("main.at")).expect("sample source");
    let snapshot =
        SourceSnapshot::from_bytes_for(&dir.join("main.at"), source.into_bytes(), Api::Sprites2V1)
            .expect("sample admits on profile 20");
    let descriptor = Descriptor::build(&doc, |_| None).expect("descriptor");
    let mut play =
        Play::new_user_spatial(&editor, [Some(1), None], snapshot, descriptor).expect("play");
    assert!(play.run_id >= 1, "a real Run was admitted");
    let order = hierarchy::ordered(&doc).expect("order");
    let targets = runtime_sprites::targets(&doc, &order).expect("targets");
    assert_eq!(targets.len(), 2, "hero and the coin template are bound");

    let settle = |play: &mut Play| {
        let end = Instant::now() + Duration::from_secs(30);
        while play.pending() {
            assert!(Instant::now() < end, "play deadline");
            if play.candidate().is_some() {
                break;
            }
            play.pump_until(end).expect("pump");
        }
        play.accept().expect("accept");
    };
    settle(&mut play);
    let first = play.last.clone().expect("tick 0");
    assert_eq!(first.run_id, play.run_id);
    assert_eq!(first.tick, 0);
    let sprites = first.sprites.as_ref().expect("sprite block");
    assert_eq!(sprites.run_id, play.run_id);
    assert_eq!(sprites.targets.len(), 2);
    assert_eq!(sprites.targets[0].frame, [0, 0, 32, 32]);
    assert!(!sprites.targets[0].flip);
    // The coin instance inherits the template default until Auto writes it.
    let slot0 = sprites.slot_visual(0).expect("coin slot visual");
    assert_eq!(slot0.visual.frame, [32, 0, 32, 32]);
    assert_eq!(slot0.generation, 1);
    assert_eq!(first.instances.as_ref().unwrap().live_count(), 3);
    assert!(first.camera.is_some(), "camera frame rides the same tick");

    // Hold right: the character moves, the camera follows, Auto walks the
    // frame sequence 0 -> 1 -> 2 and flips while moving left afterwards.
    play.char_key(1, true, false);
    let mut frames = vec![];
    let mut positions = vec![];
    let mut slots: Vec<Option<(i32, [u32; 4], bool)>> = vec![];
    let mut ticks_us: Vec<u128> = vec![];
    for _ in 0..24 {
        play.dispatch(Action::default()).expect("dispatch");
        let t = Instant::now();
        settle(&mut play);
        ticks_us.push(t.elapsed().as_micros());
        let f = play.last.as_ref().expect("frame");
        let sprite = f.sprites.as_ref().expect("sprite block");
        frames.push(sprite.targets[0].frame[0]);
        positions.push(f.nodes[&1].translation[0]);
        slots.push(
            sprite
                .slot_visual(0)
                .map(|s| (s.generation, s.visual.frame, s.visual.flip)),
        );
    }

    assert!(
        positions.last().unwrap() > positions.first().unwrap(),
        "the real Auto moved the character: {positions:?}"
    );
    let distinct: std::collections::BTreeSet<u32> = frames.iter().copied().collect();
    assert!(
        distinct.contains(&0) && distinct.contains(&32) && distinct.contains(&64),
        "the frame sequence must follow the real phase: {frames:?}"
    );
    assert_eq!(
        *frames.first().unwrap(),
        0,
        "the visual starts on the accepted default frame"
    );
    // The frame only changes when the real Auto phase boundary changes: the
    // sequence is not a host timer and repeats the same 0/1/2 walk.
    let changes = frames.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(changes >= 3, "the frame walk must advance: {frames:?}");
    // Left input flips the same frame rectangle; the geometry does not change.
    play.char_key(1, false, false);
    play.char_key(0, true, false);
    let before = play.last.as_ref().unwrap().nodes[&1].translation[0];
    let mut flipped = false;
    for _ in 0..6 {
        play.dispatch(Action::default()).expect("dispatch");
        settle(&mut play);
        let f = play.last.as_ref().unwrap();
        if f.sprites.as_ref().unwrap().targets[0].flip {
            flipped = true;
        }
    }
    assert!(flipped, "left input must flip the visual");
    let after = play.last.as_ref().unwrap().nodes[&1].translation[0];
    assert!(after < before, "flip never replaces the real movement");
    // One instance frame write is admitted from the real event read: the coin
    // pair is entered on the walk and Auto advances that slot to frame 2 with
    // the flipped sample, which is the slot default only until Auto writes it.
    // The unit keeps the auto-written frame for its own generation, then a new
    // generation of the same slot falls back to the template default.
    let written = slots
        .iter()
        .flatten()
        .find(|(gen, frame, flip)| *gen == 1 && *frame == [64, 0, 32, 32] && *flip)
        .copied();
    assert!(written.is_some(), "auto wrote slot frame 2: {slots:?}");
    let respawned = slots
        .iter()
        .flatten()
        .find(|(gen, _, _)| *gen == 2)
        .copied();
    assert!(
        respawned.is_some_and(|(_, frame, flip)| frame == [32, 0, 32, 32] && !flip),
        "a respawned slot uses the template default: {slots:?}"
    );
    let final_frame = play.last.as_ref().unwrap();
    let slot = final_frame
        .sprites
        .as_ref()
        .unwrap()
        .slot_visual(0)
        .expect("coin slot");
    // The admitted instance table and the sprite table agree on generation.
    assert_eq!(
        slot.generation,
        final_frame.instances.as_ref().unwrap().slots[0].generation
    );
    // T-02 measured feasibility: the whole composed profile runs one real tick
    // with every public surface active, and the blocked cost is bounded.
    let vm = play.owner_probe().expect("vm owner probe");
    ticks_us.sort_unstable();
    let median = ticks_us[ticks_us.len() / 2];
    let p95 = ticks_us[(ticks_us.len() * 95) / 100];
    let min = *ticks_us.first().unwrap();
    let max = *ticks_us.last().unwrap();
    println!(
        "probe: compile_us={} tick_min_us={min} tick_median_us={median} tick_p95_us={p95} tick_max_us={max} ticks={} block_len={} managed_hash={} vm_owner={}",
        play.compile_us,
        ticks_us.len(),
        runtime_sprites::block_len(targets.len()),
        play.source_hash,
        vm(),
    );
}

fn block_for(
    targets: &[Target],
    run: i32,
    tick: i32,
    instances: &RuntimeObjectsFrame,
) -> Vec<Scalar> {
    let mut data = vec![
        Scalar::Int(1),
        Scalar::Int(targets.len() as i32),
        Scalar::Int(run),
        Scalar::Int(tick),
    ];
    for t in targets {
        match Some(t) {
            Some(t) => {
                data.extend([
                    Scalar::Int(t.node_index),
                    Scalar::Int(t.asset_index),
                    Scalar::Int(t.frame[0] as i32),
                    Scalar::Int(t.frame[1] as i32),
                    Scalar::Int(t.frame[2] as i32),
                    Scalar::Int(t.frame[3] as i32),
                    Scalar::Int(i32::from(t.flip)),
                ]);
            }
            None => data.extend([Scalar::Int(0); runtime_sprites::ROW_FIELDS]),
        }
    }
    for s in 0..runtime_objects::SLOTS {
        match instances.get(s) {
            Some(slot) => {
                let template = targets
                    .iter()
                    .find(|t| t.node_id == slot.template_id)
                    .expect("bound template");
                data.extend([
                    Scalar::Int(slot.generation),
                    Scalar::Int(template.asset_index),
                    Scalar::Int(template.frame[0] as i32),
                    Scalar::Int(template.frame[1] as i32),
                    Scalar::Int(template.frame[2] as i32),
                    Scalar::Int(template.frame[3] as i32),
                    Scalar::Int(i32::from(template.flip)),
                ]);
            }
            None => data.extend([Scalar::Int(0); runtime_sprites::ROW_FIELDS]),
        }
    }
    data
}

fn dims(key: &str) -> Option<(u32, u32)> {
    if key.starts_with("sha256:") {
        Some((96, 32))
    } else {
        None
    }
}

#[test]
fn sprite_block_rejections_name_their_stage_and_the_control_still_admits() {
    let dir = sample_dir();
    let (doc, _) = sprite_document(&dir);
    let order = hierarchy::ordered(&doc).expect("order");
    let targets = runtime_sprites::targets(&doc, &order).expect("targets");
    let mut slots = [Slot::FREE; runtime_objects::SLOTS];
    slots[0] = Slot {
        generation: 1,
        template_index: 1,
        template_id: 2,
        translation: [6.0, 1.0, 0.0],
        visible: true,
    };
    let instances = RuntimeObjectsFrame {
        dim: Dim::TwoD,
        run_id: 7,
        tick: 0,
        slots,
    };
    let control = block_for(&targets, 7, 3, &instances);
    let frame = runtime_sprites::admit_block(&control, 7, 3, &targets, &dims, Some(&instances))
        .expect("accepted control");
    assert_eq!(frame.targets[0].frame, [0, 0, 32, 32]);
    assert_eq!(frame.slot_visual(0).unwrap().visual.frame, [32, 0, 32, 32]);

    let row = runtime_sprites::HEADER;
    let slot_row = runtime_sprites::HEADER + targets.len() * runtime_sprites::ROW_FIELDS;
    let cases: Vec<(&str, Vec<Scalar>, &str)> = vec![
        (
            "status",
            {
                let mut d = control.clone();
                d[0] = Scalar::Int(0);
                d
            },
            "sprite status",
        ),
        (
            "count",
            {
                let mut d = control.clone();
                d[1] = Scalar::Int(1);
                d
            },
            "sprite bound count",
        ),
        (
            "run",
            {
                let mut d = control.clone();
                d[2] = Scalar::Int(8);
                d
            },
            "sprite run/tick",
        ),
        (
            "tick",
            {
                let mut d = control.clone();
                d[3] = Scalar::Int(4);
                d
            },
            "sprite run/tick",
        ),
        (
            "row-node",
            {
                let mut d = control.clone();
                d[row] = Scalar::Int(4);
                d
            },
            "sprite row 0 node",
        ),
        (
            "row-asset",
            {
                let mut d = control.clone();
                d[row + 1] = Scalar::Int(3);
                d
            },
            "sprite row 0 asset",
        ),
        (
            "row-bounds",
            {
                let mut d = control.clone();
                d[row + 4] = Scalar::Int(128);
                d
            },
            "sprite row 0 frame bounds",
        ),
        (
            "row-overflow",
            {
                let mut d = control.clone();
                d[row + 2] = Scalar::Int(i32::MAX);
                d
            },
            "sprite row 0 frame bounds",
        ),
        (
            "row-flip",
            {
                let mut d = control.clone();
                d[row + 6] = Scalar::Int(2);
                d
            },
            "sprite row 0 flip",
        ),
        (
            "slot-stale",
            {
                let mut d = control.clone();
                d[slot_row] = Scalar::Int(2);
                d
            },
            "sprite slot 0 stale generation",
        ),
        (
            "slot-asset",
            {
                let mut d = control.clone();
                d[slot_row + 1] = Scalar::Int(3);
                d
            },
            "sprite slot 0 asset",
        ),
        (
            "slot-bounds",
            {
                let mut d = control.clone();
                d[slot_row + 2] = Scalar::Int(70);
                d
            },
            "sprite slot 0 frame bounds",
        ),
        (
            "slot-flip",
            {
                let mut d = control.clone();
                d[slot_row + 6] = Scalar::Int(-1);
                d
            },
            "sprite slot 0 flip",
        ),
        (
            "slot-free",
            {
                let mut d = control.clone();
                d[slot_row] = Scalar::Int(0);
                d
            },
            "sprite slot 0 free row",
        ),
    ];
    for (name, data, expected) in cases {
        let error = runtime_sprites::admit_block(&data, 7, 3, &targets, &dims, Some(&instances))
            .expect_err(name);
        assert!(
            error.contains(expected),
            "case {name} must fail at {expected:?}: {error}"
        );
    }
    // A payload whose registry dimensions drifted is refused before any frame
    // is considered: identity is the content, not the binding name.
    let drifted = |_: &str| Some((96u32, 33u32));
    let error = runtime_sprites::admit_block(&control, 7, 3, &targets, &drifted, Some(&instances))
        .expect_err("identity");
    assert!(error.contains("identity/registry"), "{error}");
}

#[test]
fn png_admission_rejects_out_of_domain_inputs_before_decoding() {
    let dir = sample_dir();
    let base: assets::Registry = Default::default();
    let sheet = assets::import_png(&dir.join("sheet.png"), &base).expect("valid sheet");
    assert_eq!(sheet.identity.format, "png");
    assert_eq!(
        (sheet.identity.width, sheet.identity.height),
        (Some(96), Some(32))
    );
    assert_eq!(sheet.cpu_bytes, 96 * 32 * 4);
    assert_eq!(sheet.gpu_bytes, 96 * 32 * 4);
    assert!(sheet.model.meshes.is_empty());
    assert!(sheet.model_data().is_err(), "an image is never a model");
    assert_eq!(
        sheet.image_data().unwrap().pixel(0, 1),
        Some([255, 0, 0, 255]),
        "the sheet keeps its real left-red half"
    );
    assert_eq!(sheet.image_data().unwrap().pixel(1, 0), Some([0, 0, 0, 0]));

    let raw = std::fs::read(dir.join("sheet.png")).expect("fixture bytes");
    let attempts = assets::decode_attempts();
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("signature", {
            let mut b = raw.clone();
            b[0] = 0;
            b
        }),
        ("truncated", raw[..40].to_vec()),
        ("apng", {
            let mut b = raw.clone();
            // Insert one acTL chunk after IHDR; the preflight rejects animation
            // before any decoder sees the file.
            let at = 8 + 12 + 13;
            let mut chunk = 4u32.to_be_bytes().to_vec();
            chunk.extend_from_slice(b"acTL");
            chunk.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1]);
            chunk.extend_from_slice(&[0, 0, 0, 0]);
            b.splice(at..at, chunk);
            b
        }),
        ("interlaced", {
            let mut b = raw.clone();
            b[28] = 1;
            b
        }),
        ("bit-depth", {
            let mut b = raw.clone();
            b[24] = 16;
            b
        }),
        ("palette", {
            let mut b = raw.clone();
            b[25] = 3;
            b
        }),
        ("oversize", {
            let mut b = raw.clone();
            b[16..20].copy_from_slice(&5000u32.to_be_bytes());
            b
        }),
        ("zero-side", {
            let mut b = raw.clone();
            b[20..24].copy_from_slice(&0u32.to_be_bytes());
            b
        }),
        ("trailing", {
            let mut b = raw.clone();
            b.push(0);
            b
        }),
    ];
    for (name, bytes) in cases {
        let error = assets::decode_png_in(bytes, None, &base).expect_err(name);
        assert!(
            error.starts_with("invalid")
                || error.starts_with("unsupported")
                || error.starts_with("resource limit"),
            "case {name} must be a named rejection: {error}"
        );
    }
    assert_eq!(
        assets::decode_attempts(),
        attempts,
        "no rejected input may reach the decoder"
    );
    // The accepted payload is cached by content identity and reused without a
    // second decode; the same bytes under a drifted declared identity reject.
    let again = assets::import_png(&dir.join("sheet.png"), &base).expect("cached import");
    assert_eq!(again.identity, sheet.identity);
    let mut declared = sheet.identity.clone();
    declared.width = Some(96);
    declared.height = Some(31);
    let mut registry: assets::Registry = Default::default();
    assets::insert(&mut registry, sheet.clone()).expect("registry");
    let error = assets::load_declared(
        &dir.join("sheet.png"),
        &declared,
        &assets::Registry::default(),
    )
    .expect_err("dimension drift");
    assert!(
        error.contains("decoded image") || error.contains("dimensions"),
        "{error}"
    );
}

#[test]
fn scene_v5_persists_the_binding_and_legacy_schemas_never_accept_it_silently() {
    let dir = sample_dir();
    let (doc, _) = sprite_document(&dir);
    let bytes = doc.encode().expect("encode");
    let text = String::from_utf8(bytes.clone()).expect("utf8");
    assert!(text.contains("\"sprite\":{"), "{text}");
    let decoded = Document::decode(&bytes).expect("v5 round trip");
    assert_eq!(decoded, doc);

    // A real v4 file keeps its exact meaning: it has no image asset and no
    // binding, and migrating it leaves every leaf without a visual. A v4 file
    // that carries the new field or a PNG resource is rejected instead of being
    // silently upgraded.
    let mut older = doc.clone();
    older.assets.clear();
    for o in &mut older.objects {
        o.sprite = None;
    }
    let older_text = String::from_utf8(older.encode().expect("legacy encode")).expect("utf8");
    let v4 = older_text
        .replace("autographics.scene/5", "autographics.scene/4")
        .replace(",\"sprite\":null", "");
    assert!(v4.contains("autographics.scene/4") && !v4.contains("\"sprite\""));
    let migrated = Document::decode(v4.as_bytes()).expect("v4 migrates in memory");
    assert_eq!(migrated.schema, "autographics.scene/5");
    assert!(migrated.objects.iter().all(|o| o.sprite.is_none()));
    assert_eq!(migrated.objects.len(), doc.objects.len());
    let sneaky = text.replace("autographics.scene/5", "autographics.scene/4");
    let error = Document::decode(sneaky.as_bytes()).expect_err("legacy sprite field");
    assert!(
        error.contains("legacy unknown asset format") || error.contains("legacy unknown sprite"),
        "{error}"
    );

    let mut broken: Document = doc.clone();
    broken.objects[0].sprite = Some(SpriteBinding {
        asset: doc.assets[0].key(),
        frame: [80, 0, 32, 32],
        flip: false,
    });
    assert!(broken.validate().is_err(), "out-of-image frame must reject");

    let mut kind: Document = doc.clone();
    kind.objects[1].sprite = Some(SpriteBinding {
        asset: doc.assets[0].key(),
        frame: [0, 0, 32, 32],
        flip: false,
    });
    kind.objects[1].kind = Kind::Cylinder;
    assert!(kind.validate().is_err(), "3D leaves cannot carry a sprite");
}

// ---------------------------------------------------------------------------
// V-01: the finite PNG domain, the project closure and the editor binding.
// ---------------------------------------------------------------------------

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-sprite-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp root");
    dir
}

fn settle_editor(e: &mut Editor) {
    while !e.pump().expect("pump") {}
    e.commit().expect("commit");
}

#[test]
fn png_closure_saves_reopens_and_moves_with_the_project() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let mut editor = Editor::with_project(doc.clone(), registry, None).expect("editor");
    let root = temp_root("closure");
    let entry = root.join("scene.agscene");
    let mut storage = Storage::default();
    assert_eq!(
        storage
            .save(&mut editor, &entry, Fault::None)
            .expect("save"),
        Saved::Confirmed
    );
    let sha = doc.assets[0].sha256.clone();
    let blob = root.join("assets").join(format!("{sha}.png"));
    assert!(blob.is_file(), "the accepted PNG joins the project closure");
    assert_eq!(
        std::fs::read(&blob).unwrap(),
        std::fs::read(dir.join("sheet.png")).unwrap(),
        "the closure blob is the accepted original bytes, not a re-encode"
    );
    // Reopen: document, frame domain and decoded pixels all come from the
    // project directory, never from the original import path.
    let (reopened, assets_in, code, reopened_storage) =
        Storage::load_project(&entry, &Default::default()).expect("reopen");
    assert!(code.is_none());
    assert!(!reopened_storage.unverified());
    assert_eq!(reopened, editor.document);
    let key = doc.assets[0].key();
    let asset = assets_in.get(&key).expect("image payload");
    let image = asset.image_data().expect("decoded image");
    assert_eq!((image.width, image.height), (96, 32));
    assert_eq!(
        image.pixel(0, 1),
        Some([255, 0, 0, 255]),
        "real pixels survived"
    );
    assert_eq!(image.pixel(20, 1), Some([0, 0, 255, 255]));
    // Move the whole directory A -> B and open from the new place.
    let moved = temp_root("closure-moved");
    std::fs::rename(&root, &moved).expect("move project");
    let moved_entry = moved.join("scene.agscene");
    let (again, assets_moved, _, _) =
        Storage::load_project(&moved_entry, &Default::default()).expect("reopen moved");
    assert_eq!(again, reopened);
    assert_eq!(assets_moved.get(&key).unwrap().identity, doc.assets[0]);
    // The moved project is still editable and saveable with the same payload.
    let mut editor2 = Editor::with_project(again, assets_moved, None).expect("editor2");
    let mut leaf = Object::new(editor2.next_id().unwrap(), Kind::Quad);
    leaf.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [32, 0, 32, 32],
        flip: true,
    });
    assert!(editor2
        .begin(
            editor2.stamp,
            Edit::SpriteObject {
                asset: None,
                object: leaf,
                create: true,
            },
        )
        .unwrap());
    settle_editor(&mut editor2);
    assert_eq!(
        Storage::default()
            .save(&mut editor2, &moved_entry, Fault::None)
            .expect("save moved"),
        Saved::Confirmed
    );
    let _ = std::fs::remove_dir_all(&moved);
}

#[test]
fn png_closure_rejects_missing_tampered_foreign_and_over_budget_blobs() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let mut editor = Editor::with_project(doc.clone(), registry, None).expect("editor");
    let root = temp_root("closure-neg");
    let entry = root.join("scene.agscene");
    let mut storage = Storage::default();
    storage
        .save(&mut editor, &entry, Fault::None)
        .expect("save");
    let sha = doc.assets[0].sha256.clone();
    let blob = root.join("assets").join(format!("{sha}.png"));
    let original = std::fs::read(&blob).unwrap();
    // Missing blob: the reopen rejects instead of fabricating the payload.
    std::fs::remove_file(&blob).unwrap();
    let error = match Storage::load_project(&entry, &Default::default()) {
        Ok(_) => panic!("missing blob must reject"),
        Err(e) => e,
    };
    assert!(error.contains("io") || error.contains("missing"), "{error}");
    // Restoring the exact bytes recovers the project (repair, not rollback).
    std::fs::write(&blob, &original).unwrap();
    assert!(Storage::load_project(&entry, &Default::default()).is_ok());
    // Tampered bytes under the accepted name are refused and preserved.
    let mut tampered = original.clone();
    tampered[40] ^= 0xff;
    std::fs::write(&blob, &tampered).unwrap();
    let error = match Storage::load_project(&entry, &Default::default()) {
        Ok(_) => panic!("missing blob must reject"),
        Err(e) => e,
    };
    assert!(error.contains("conflict"), "{error}");
    assert_eq!(
        std::fs::read(&blob).unwrap(),
        tampered,
        "a bad blob is never overwritten by the reader"
    );
    std::fs::write(&blob, &original).unwrap();
    // A foreign file in assets/ is reported and left untouched by a save.
    let foreign = root.join("assets").join("notes.txt");
    std::fs::write(&foreign, b"keep").unwrap();
    let error = Storage::default()
        .save(&mut editor, &entry, Fault::None)
        .unwrap_err();
    assert!(
        error.contains("foreign or incomplete asset path"),
        "{error}"
    );
    assert_eq!(std::fs::read(&foreign).unwrap(), b"keep");
    std::fs::remove_file(&foreign).unwrap();
    // The shared 8-file budget counts every blob in the directory, not only
    // the declared ones.
    let mut extras = vec![];
    for i in 0..8u8 {
        let name = format!("{:02x}{}.png", i, "0".repeat(62));
        let path = root.join("assets").join(name);
        std::fs::write(&path, [i]).unwrap();
        extras.push(path);
    }
    let error = Storage::default()
        .save(&mut editor, &entry, Fault::None)
        .unwrap_err();
    assert!(error.contains("8"), "{error}");
    for path in extras {
        std::fs::remove_file(path).unwrap();
    }
    assert_eq!(
        Storage::default()
            .save(&mut editor, &entry, Fault::None)
            .unwrap(),
        Saved::Confirmed,
        "removing the residuals restores a clean closure"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn editor_sprite_commands_create_bind_detach_and_reject_wrong_targets() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let key = doc.assets[0].key();
    let mut editor = Editor::with_project(doc.clone(), registry.clone(), None).expect("editor");
    assert_eq!(editor.document.assets.len(), 1);
    // Create a new sprite leaf from the accepted payload.
    let id = editor.next_id().unwrap();
    let mut leaf = Object::new(id, Kind::Quad);
    leaf.scale = [3.0, 1.0, 1.0];
    leaf.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [0, 0, 96, 32],
        flip: false,
    });
    assert!(editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: Some(registry.get(&key).unwrap().clone()),
                object: leaf,
                create: true,
            },
        )
        .unwrap());
    settle_editor(&mut editor);
    let stored = editor
        .document
        .objects
        .iter()
        .find(|o| o.id == id)
        .expect("created leaf");
    assert_eq!(stored.sprite.as_ref().unwrap().frame, [0, 0, 96, 32]);
    // Rebind the selected 2D leaf while keeping its local/parent/properties.
    editor.select(Some(5)).unwrap();
    let mut target = editor.document.local_target(5).unwrap();
    let before = target.clone();
    target.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [32, 0, 32, 32],
        flip: true,
    });
    assert!(editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: target,
                create: false,
            },
        )
        .unwrap());
    settle_editor(&mut editor);
    let stored = editor
        .document
        .objects
        .iter()
        .find(|o| o.id == 5)
        .unwrap()
        .clone();
    assert_eq!(stored.sprite.as_ref().unwrap().frame, [32, 0, 32, 32]);
    assert!(stored.sprite.as_ref().unwrap().flip);
    assert_eq!(stored.translation, before.translation);
    // Detach clears only the binding.
    let mut detached = stored.clone();
    detached.sprite = None;
    assert!(editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: detached,
                create: false,
            },
        )
        .unwrap());
    settle_editor(&mut editor);
    assert!(editor
        .document
        .objects
        .iter()
        .find(|o| o.id == 5)
        .unwrap()
        .sprite
        .is_none());
    // Stale selection, wrong payload identity, an out-of-image frame and a 3D
    // target are named rejections that leave the document unchanged.
    let mut stale = editor.document.local_target(5).unwrap();
    stale.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [0, 0, 32, 32],
        flip: false,
    });
    editor.select(Some(1)).unwrap();
    assert!(editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: stale,
                create: false,
            },
        )
        .is_err());
    editor.select(Some(5)).unwrap();
    let kept = editor.document.clone();
    let mut wrong = editor.document.local_target(5).unwrap();
    wrong.sprite = Some(SpriteBinding {
        asset: format!("sha256:{}", "0".repeat(64)),
        frame: [0, 0, 32, 32],
        flip: false,
    });
    let error = editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: wrong,
                create: false,
            },
        )
        .unwrap_err();
    assert!(error.contains("sprite asset reference"), "{error}");
    let mut out = editor.document.local_target(5).unwrap();
    out.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [80, 0, 32, 32],
        flip: false,
    });
    let error = editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: out,
                create: false,
            },
        )
        .unwrap_err();
    assert!(error.contains("sprite frame bounds"), "{error}");
    let mut three = Object::new(9, Kind::Cylinder);
    three.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [0, 0, 32, 32],
        flip: false,
    });
    let error = editor
        .begin(
            editor.stamp,
            Edit::SpriteObject {
                asset: None,
                object: three,
                create: true,
            },
        )
        .unwrap_err();
    assert!(error.contains("2D leaf"), "{error}");
    assert_eq!(editor.document, kept);
    // Nine bindings exceed the frozen target table.
    let mut nine = Document::empty("nine".into());
    nine.assets.push(doc.assets[0].clone());
    for id in 1..=9u64 {
        let mut o = Object::new(id, Kind::Quad);
        o.sprite = Some(SpriteBinding {
            asset: key.clone(),
            frame: [0, 0, 1, 1],
            flip: false,
        });
        nine.objects.push(o);
    }
    let error = nine.validate().unwrap_err();
    assert!(error.contains("sprite target budget"), "{error}");
}

#[test]
fn png_side_and_decoded_budget_are_checked_before_decoding() {
    let base: assets::Registry = Default::default();
    // A full 2048x2048 RGBA image is exactly the 16MiB decoded ceiling.
    let bytes = encode_png_rgba(2048, 2048, &vec![0x7f; 2048 * 2048 * 4]).expect("encode");
    let attempts = assets::decode_attempts();
    let asset = assets::decode_png_in(bytes, None, &base).expect("2048 accepted");
    assert_eq!(asset.cpu_bytes, 16 * 1024 * 1024);
    assert_eq!(asset.gpu_bytes, 16 * 1024 * 1024);
    assert_eq!(assets::decode_attempts(), attempts + 1);
    // One pixel more per side is outside the finite domain and never decodes.
    let wide = encode_png_rgba(2049, 1, &vec![0u8; 2049 * 4]).expect("encode");
    let error = assets::decode_png_in(wide, None, &base).unwrap_err();
    assert!(error.contains("resource limit: PNG side"), "{error}");
    let tall = encode_png_rgba(1, 2049, &vec![0u8; 2049 * 4]).expect("encode");
    assert!(assets::decode_png_in(tall, None, &base)
        .unwrap_err()
        .contains("PNG side"));
    assert_eq!(
        assets::decode_attempts(),
        attempts + 1,
        "rejected sides never reach the decoder"
    );
    // The union budget is shared with models: four 16MiB image payloads fit at
    // the documented 64MiB ceiling, a fifth is refused before decoding, and no
    // per-format quota is added.
    let mut union = base.clone();
    assets::insert(&mut union, asset.clone()).expect("first image");
    for i in 1..4u8 {
        let payload =
            encode_png_rgba(2048, 2048, &vec![0x40 + i; 2048 * 2048 * 4]).expect("encode");
        let image = assets::decode_png_in(payload, None, &union).expect("within 64MiB");
        assets::insert(&mut union, image).expect("registry insert");
    }
    assert_eq!(union.len(), 4);
    assert_eq!(
        union.values().map(|a| a.cpu_bytes).sum::<usize>(),
        64 * 1024 * 1024
    );
    let attempts = assets::decode_attempts();
    let payload = encode_png_rgba(2048, 2048, &vec![0x55; 2048 * 2048 * 4]).expect("encode");
    let error = assets::decode_png_in(payload, None, &union).unwrap_err();
    assert!(
        error.contains("candidate/resident union before decode"),
        "{error}"
    );
    assert_eq!(
        assets::decode_attempts(),
        attempts,
        "an over-budget candidate never allocates pixels"
    );
    assert_eq!(assets::MAX_GEOMETRY, 64 * 1024 * 1024);
    assert_eq!(assets::MAX_ASSETS, 8);
}

#[test]
fn host_imports_binds_and_detaches_a_png_visual() {
    use autographics_auto_driver::authoring::host::Host;
    use serde_json::json;
    let dir = sample_dir();
    let path = dir.join("sheet.png");
    let mut host = Host::new(Document::empty("sprite-host".into())).expect("host");
    let send = |host: &mut Host, action: &str, payload: serde_json::Value| {
        let message = json!({
            "epoch": host.editor.stamp.epoch,
            "revision": host.editor.stamp.revision,
            "action": action,
            "payload": payload,
            "selection": host.editor.selection,
            "asset_generation": host.ui().asset_token,
        })
        .to_string();
        host.dispatch(&message)?;
        host.settle()?;
        if !host.ui().error.is_empty() {
            return Err(host.ui().error.clone());
        }
        Ok(())
    };
    // One public command imports the accepted PNG and creates a bound 2D leaf.
    send(&mut host, "import-sprite", json!(path.to_str().unwrap())).expect("import");
    let doc = host.editor.document.clone();
    assert_eq!(doc.assets.len(), 1);
    assert!(doc.assets[0].is_image());
    assert_eq!(
        (doc.assets[0].width, doc.assets[0].height),
        (Some(96), Some(32))
    );
    let leaf = doc
        .objects
        .iter()
        .find(|o| o.sprite.is_some())
        .expect("bound leaf")
        .clone();
    assert!(leaf.kind.is_2d());
    assert_eq!(leaf.sprite.as_ref().unwrap().frame, [0, 0, 96, 32]);
    assert!(
        (leaf.scale[0] / leaf.scale[1] - 3.0).abs() < 1e-6,
        "aspect kept"
    );
    // The accepted document renders through the existing 2D canvas.
    host.frame(None).expect("frame after import");
    // Detach the binding from the selected leaf.
    host.editor.select(Some(leaf.id)).expect("select");
    send(&mut host, "sprite-detach", json!(doc.assets[0].key())).expect("detach");
    assert!(host
        .editor
        .document
        .objects
        .iter()
        .find(|o| o.id == leaf.id)
        .unwrap()
        .sprite
        .is_none());
    // Re-importing the same bytes reuses the single content identity.
    send(&mut host, "import-sprite", json!(path.to_str().unwrap())).expect("reimport");
    assert_eq!(host.editor.document.assets.len(), 1);
    assert_eq!(
        host.editor
            .document
            .objects
            .iter()
            .filter(|o| o.sprite.is_some())
            .count(),
        1
    );
    // An invalid PNG path is a named failure that leaves the document alone.
    let before = host.editor.document.clone();
    let bad = dir.join("not-a-png.png");
    std::fs::write(&bad, b"not a png").unwrap();
    assert!(send(&mut host, "import-sprite", json!(bad.to_str().unwrap())).is_err());
    assert_eq!(host.editor.document, before);
    std::fs::remove_file(bad).unwrap();
}

// ---------------------------------------------------------------------------
// V-02: real pixels. The 2D surface is probed at the exact points the
// presentation code projects, so the oracle covers the whole path from the
// accepted binding/Auto output through UV construction to the drawn texel.
// ---------------------------------------------------------------------------

fn sample_registry(dir: &Path, name: &str) -> (assets::Registry, String) {
    let empty: assets::Registry = Default::default();
    let asset = assets::import_png(&dir.join(name), &empty).expect("png import");
    let key = asset.identity.key();
    let mut registry = empty;
    assets::insert(&mut registry, asset).expect("registry");
    (registry, key)
}

fn settle_host(h: &mut autographics_auto_driver::authoring::host::Host) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

/// Dispatch exactly one tick and pump it to completion. The clock may leave a
/// new tick in flight after a frame, so a dispatch is only legal when idle.
fn step_once(h: &mut autographics_auto_driver::authoring::host::Host) {
    let end = Instant::now() + Duration::from_secs(60);
    while h.play.as_ref().is_some_and(|p| p.pending()) {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
    h.play
        .as_mut()
        .expect("play")
        .dispatch(Action::default())
        .expect("dispatch");
    loop {
        h.frame(None).expect("frame");
        if !h.play.as_ref().is_some_and(|p| p.pending()) {
            break;
        }
        assert!(Instant::now() < end, "host deadline");
    }
}

#[test]
fn editor_preview_draws_the_bound_image_and_flips_only_the_uv() {
    let dir = sample_dir();
    let (registry, key) = sample_registry(&dir, "padded.png");
    let mut doc = Document::empty("sprite-pixels".into());
    doc.assets
        .push(registry.get(&key).unwrap().identity.clone());
    let mut leaf = Object::new(1, Kind::Quad);
    leaf.scale = [64.0, 64.0, 1.0];
    leaf.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [0, 0, 32, 32],
        flip: false,
    });
    doc.objects.push(leaf.clone());
    doc.validate().expect("document");
    let geometry = Descriptor::build(&doc, |_| None)
        .expect("descriptor")
        .rows()
        .to_vec();
    let mut host = Host::with_project(doc, registry, None).expect("host");
    host.frame(None).expect("frame");
    // view2d default: world (0,0) is screen (400,190) and zoom 1, so the 64x64
    // leaf spans 368..432 x 158..222 with an 8px transparent border.
    let probe = |host: &mut Host, dx: f32| {
        host.rt
            .probe_pixel(host.two, [400.0 + dx, 190.0])
            .expect("pixel")
    };
    let left = probe(&mut host, -16.0);
    let right = probe(&mut host, 16.0);
    let border = probe(&mut host, -30.0);
    assert!(left[0] > 180 && left[2] < 60, "left red {left:?}");
    assert!(right[2] > 180 && right[0] < 60, "right blue {right:?}");
    assert!(
        border[0] < 70 && border[2] < 70 && border[1] < 70,
        "a transparent border shows the canvas instead of a tint quad {border:?}"
    );
    // The flip is a document edit of the same binding and mirrors U only.
    host.editor.select(Some(1)).expect("select");
    let mut flipped = leaf.clone();
    flipped.sprite = Some(SpriteBinding {
        asset: key.clone(),
        frame: [0, 0, 32, 32],
        flip: true,
    });
    host.editor
        .begin(host.editor.stamp, Edit::Replace(flipped))
        .expect("begin");
    host.settle().expect("settle");
    host.frame(None).expect("frame");
    let left = probe(&mut host, -16.0);
    let right = probe(&mut host, 16.0);
    assert!(left[2] > 180 && left[0] < 60, "flipped left blue {left:?}");
    assert!(
        right[0] > 180 && right[2] < 60,
        "flipped right red {right:?}"
    );
    // A frame or flip change is a UV fact only: the entity's spatial boxes and
    // therefore every collision rule stay exactly the same.
    let after_geometry = Descriptor::build(&host.editor.document, |_| None)
        .expect("descriptor")
        .rows()
        .to_vec();
    assert_eq!(
        after_geometry, geometry,
        "flip must not change the entity bounds"
    );
}

fn hero_probe(host: &mut Host, sign: f32) -> [u8; 4] {
    let (center, scale) = {
        let play = host.play.as_ref().expect("play");
        let frame = play.last.as_ref().expect("frame");
        let cam = frame.camera.as_ref().expect("camera");
        let t = frame.nodes[&1].translation;
        let follow = cam
            .follow_center_2d(glam::Vec2::new(t[0], t[1]))
            .expect("camera centre");
        let screen = cam
            .screen_2d(glam::Vec2::new(t[0], t[1]), follow)
            .expect("camera projection");
        (screen, cam.letterbox_2d().expect("letterbox").0.x)
    };
    let half = 0.8 * scale * 0.5;
    host.rt
        .probe_pixel(host.two, [center.x + sign * half * 0.55, center.y])
        .expect("hero pixel")
}

#[test]
fn real_auto_frame_and_flip_change_the_drawn_pixels() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let mut host = Host::with_project(doc, registry, None).expect("host");
    let source = std::fs::read_to_string(dir.join("main.at")).expect("source");
    let snapshot =
        SourceSnapshot::from_bytes_for(&dir.join("main.at"), source.into_bytes(), Api::Sprites2V1)
            .expect("snapshot");
    host.start_user_snapshot(snapshot).expect("start");
    host.pause_play("");
    settle_host(&mut host);
    // One neutral tick: frame 0 with no flip.
    host.play.as_mut().unwrap().resume().expect("resume");
    host.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    host.frame(None).expect("frame");
    let left = hero_probe(&mut host, -1.0);
    let right = hero_probe(&mut host, 1.0);
    assert!(left[0] > 150 && left[2] < 70, "frame 0 left red {left:?}");
    assert!(
        right[2] > 150 && right[0] < 70,
        "frame 0 right blue {right:?}"
    );
    // Hold left until the admitted flip reaches the pixels.
    host.play.as_mut().unwrap().char_key(0, true, false);
    let mut flipped = false;
    for _ in 0..10 {
        host.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .expect("dispatch");
        host.frame(None).expect("frame");
        let admitted = host
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .and_then(|f| f.sprites.as_ref())
            .is_some_and(|s| s.targets[0].flip);
        if admitted {
            flipped = true;
            break;
        }
    }
    assert!(flipped, "the real Auto admitted a flip");
    let left = hero_probe(&mut host, -1.0);
    let right = hero_probe(&mut host, 1.0);
    assert!(left[2] > 150 && left[0] < 70, "flipped left blue {left:?}");
    assert!(
        right[0] > 150 && right[2] < 70,
        "flipped right red {right:?}"
    );
    host.play.as_mut().unwrap().char_key(0, false, false);
    host.stop_play();
}

// ---------------------------------------------------------------------------
// V-04: Pause/Step/Stop/new Run, whole-group last-good on a failed prepare and
// the texture owner lifetime across repeated runs.
// ---------------------------------------------------------------------------

fn play_snapshot(dir: &Path) -> SourceSnapshot {
    let source = std::fs::read_to_string(dir.join("main.at")).expect("source");
    SourceSnapshot::from_bytes_for(&dir.join("main.at"), source.into_bytes(), Api::Sprites2V1)
        .expect("snapshot")
}

fn sprite_stats(host: &Host) -> (u64, u64, usize) {
    match host.rt.content(host.two) {
        Some(SurfaceContent::Game2d(g)) => (
            g.sprite_store.created,
            g.sprite_store.bytes_uploaded,
            g.sprite_store.len(),
        ),
        _ => panic!("2d content"),
    }
}

fn sprite_weak(host: &Host, key: &str) -> std::sync::Weak<agp::renderer::sprite::SpriteTexture> {
    match host.rt.content(host.two) {
        Some(SurfaceContent::Game2d(g)) => {
            std::sync::Arc::downgrade(g.sprite_store.get(key).expect("texture"))
        }
        _ => panic!("2d content"),
    }
}

/// The UV of the hero's published command, matched to the exact screen point the
/// presentation code projects for its admitted world position.
fn hero_uv(host: &Host) -> [f32; 4] {
    let play = host.play.as_ref().expect("play");
    let frame = play.last.as_ref().expect("frame");
    let cam = frame.camera.as_ref().expect("camera");
    let t = frame.nodes[&1].translation;
    let follow = cam
        .follow_center_2d(glam::Vec2::new(t[0], t[1]))
        .expect("camera centre");
    let p = cam
        .screen_2d(glam::Vec2::new(t[0], t[1]), follow)
        .expect("projection");
    match host.rt.content(host.two) {
        Some(SurfaceContent::Game2d(g)) => {
            g.sprites
                .iter()
                .find(|c| (c.center[0] - p.x).abs() < 0.5 && (c.center[1] - p.y).abs() < 0.5)
                .expect("hero command")
                .uv
        }
        _ => panic!("2d content"),
    }
}

fn published_uvs(host: &Host) -> Vec<[f32; 4]> {
    match host.rt.content(host.two) {
        Some(SurfaceContent::Game2d(g)) => g.sprites.iter().map(|c| c.uv).collect(),
        _ => panic!("2d content"),
    }
}

#[test]
fn pause_step_stop_and_a_new_run_bound_the_visual_lifecycle() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let mut host = Host::with_project(doc, registry, None).expect("host");
    let default_uv = {
        let key = host.editor.document.assets[0].key();
        let _ = key;
        [0.0, 0.0, 32.0 / 96.0, 1.0]
    };
    host.start_user_snapshot(play_snapshot(&dir))
        .expect("start");
    host.pause_play("");
    settle_host(&mut host);
    assert_eq!(
        hero_uv(&host),
        default_uv,
        "tick 0 uses the accepted default"
    );
    // Resume and advance past the first phase boundary: the admitted frame
    // really comes from the Auto phase, not from a host timer.
    host.play.as_mut().unwrap().resume().expect("resume");
    // The wall clock may dispatch extra ticks between a dispatch and its frame,
    // so assert what the phase actually walked instead of a fixed tick count.
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..8 {
        step_once(&mut host);
        seen.insert(format!("{:?}", hero_uv(&host)));
    }
    assert!(
        seen.len() > 1,
        "the published frame follows the real phase: {seen:?}"
    );
    // Pause: no tick, no visual change, no re-upload.
    host.pause_play("test");
    let paused_tick = host.play.as_ref().unwrap().tick();
    let paused_uv = published_uvs(&host);
    host.frame(None).expect("frame while paused");
    assert_eq!(host.play.as_ref().unwrap().tick(), paused_tick);
    assert_eq!(published_uvs(&host), paused_uv);
    // Step: exactly one neutral tick, prepared and published like any other.
    host.play.as_mut().unwrap().single_step().expect("step");
    host.frame(None).expect("frame while stepping");
    assert_eq!(host.play.as_ref().unwrap().tick(), paused_tick + 1);
    // Stop: the runtime owner is revoked and the editing preview (document
    // default frame) is restored.
    host.stop_play();
    host.frame(None).expect("frame after stop");
    assert!(host.play.is_none());
    assert_eq!(
        published_uvs(&host),
        vec![default_uv, [32.0 / 96.0, 0.0, 64.0 / 96.0, 1.0]],
        "Stop restores the editing preview with the document default frames"
    );
    // A new Run starts a fresh Auto session on the initialization frame.
    host.start_user_snapshot(play_snapshot(&dir))
        .expect("restart");
    host.pause_play("");
    settle_host(&mut host);
    assert_eq!(host.play.as_ref().unwrap().tick(), 0);
    assert_eq!(hero_uv(&host), default_uv);
    host.stop_play();
}

#[test]
fn failed_prepare_keeps_the_whole_previous_group_and_recovers() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let mut host = Host::with_project(doc, registry, None).expect("host");
    host.start_user_snapshot(play_snapshot(&dir))
        .expect("start");
    host.pause_play("");
    settle_host(&mut host);
    host.play.as_mut().unwrap().resume().expect("resume");
    for _ in 0..4 {
        step_once(&mut host);
    }
    let before_tick = host.play.as_ref().unwrap().tick();
    let before_uv = published_uvs(&host);
    let before_x = host.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation[0];
    let before_slot = host
        .play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .sprites
        .as_ref()
        .unwrap()
        .slot_visual(0)
        .map(|s| (s.generation, s.visual.frame));
    // A preparation failure rejects the whole candidate: nothing publishes.
    host.fail_resource = true;
    while host.play.as_ref().is_some_and(|p| p.pending()) {
        host.frame(None).expect("frame");
    }
    host.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    // A tick may span several frames under the 8ms pump budget, so keep the
    // injection until the play actually reports the failed preparation.
    let end = Instant::now() + Duration::from_secs(30);
    loop {
        host.frame(None).expect("frame reports through the play");
        let play = host.play.as_ref().unwrap();
        if play.error.is_some() || play.state == State::Faulted {
            break;
        }
        assert!(Instant::now() < end, "injected failure never surfaced");
    }
    host.fail_resource = false;
    assert!(
        host.play.as_ref().unwrap().error.is_some(),
        "the failure is named"
    );
    assert_eq!(
        host.play.as_ref().unwrap().tick(),
        before_tick,
        "tick did not advance"
    );
    assert_eq!(
        host.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation[0],
        before_x,
        "position is the old one"
    );
    assert_eq!(
        published_uvs(&host),
        before_uv,
        "the sprite group is the old one"
    );
    assert_eq!(
        host.play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .sprites
            .as_ref()
            .unwrap()
            .slot_visual(0)
            .map(|s| (s.generation, s.visual.frame)),
        before_slot
    );
    // Stop recovers the editing preview and a fresh Run is green again.
    host.stop_play();
    host.start_user_snapshot(play_snapshot(&dir))
        .expect("restart");
    host.pause_play("");
    settle_host(&mut host);
    assert!(host.play.as_ref().unwrap().error.is_none());
    assert_eq!(host.play.as_ref().unwrap().tick(), 0);
    host.stop_play();
}

#[test]
fn twenty_play_stop_rounds_reuse_one_texture_and_release_the_last_owner() {
    let dir = sample_dir();
    let (doc, registry) = sprite_document(&dir);
    let sheet = doc.assets[0].key();
    let mut host = Host::with_project(doc, registry, None).expect("host");
    let mut weak = None;
    for round in 0..20 {
        host.start_user_snapshot(play_snapshot(&dir))
            .expect("start");
        host.pause_play("");
        settle_host(&mut host);
        host.play.as_mut().unwrap().resume().expect("resume");
        for _ in 0..5 {
            step_once(&mut host);
        }
        let (created, bytes, textures) = sprite_stats(&host);
        assert_eq!(created, 1, "round {round} reused one content texture");
        assert_eq!(bytes, 96 * 32 * 4, "round {round} uploaded the pixels once");
        assert_eq!(textures, 1);
        if weak.is_none() {
            weak = Some(sprite_weak(&host, &sheet));
        }
        host.stop_play();
    }
    let weak = weak.expect("witness");
    assert!(
        weak.upgrade().is_some(),
        "the accepted project still legitimately owns the texture"
    );
    // Closing the project drops the accepted payload; the next frame prunes the
    // cache and the last owner releases the texture.
    host.editor.close();
    host.frame(None).expect("frame after close");
    assert_eq!(sprite_stats(&host).2, 0, "no texture outlives its payload");
    assert!(
        weak.upgrade().is_none(),
        "the last owner released the texture"
    );
}
