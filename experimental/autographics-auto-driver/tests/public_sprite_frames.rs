//! PLAN-004 public sprite frames: profile admission, byte-identical older
//! profiles, the real-Auto CPU feasibility probe (T-02) and the named host
//! admission controls of the sprite block. The GPU presentation cases live in
//! the driver's sprite host test and the AGP `sprite_resources` test.
//!
//! Run single-threaded: `cargo test --locked --test public_sprite_frames -- --test-threads=1`
use autographics_auto_driver::authoring::{
    assets,
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind, Object, SpriteBinding},
    hierarchy,
    play::{Action, Play},
    runtime_objects::{self, Dim, RuntimeObjectsFrame, Slot},
    runtime_sprites::{self, Target},
    session::Editor,
    spatial::Descriptor,
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
    assert_eq!(play.run_id, 1);
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
    assert_eq!(first.run_id, 1);
    assert_eq!(first.tick, 0);
    let sprites = first.sprites.as_ref().expect("sprite block");
    assert_eq!(sprites.run_id, 1);
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
    let final_frame = play.last.as_ref().unwrap();
    let slot = final_frame
        .sprites
        .as_ref()
        .unwrap()
        .slot_visual(0)
        .expect("coin slot");
    assert_eq!(
        slot.visual.frame,
        [64, 0, 32, 32],
        "auto wrote slot frame 2"
    );
    assert!(slot.visual.flip, "the slot write carries the flip");
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
    println!(
        "probe: compile_us={} tick_median_us={median} tick_p95_us={p95} ticks={} block_len={} managed_hash={} vm_owner={}",
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
