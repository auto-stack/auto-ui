//! Reproducible public project derivation through the normal import/save path.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::Api,
    behavior_project::{Actors, Binding, CodePayload},
    document::{Document, Kind, Object, SpriteBinding},
    public_audio::{self, bank, CueRef},
    session::Editor,
    storage::{Fault, Saved, Storage},
};
use std::{path::Path, sync::Arc};
fn main() {
    let output = std::env::args_os()
        .nth(1)
        .expect("destination directory required");
    let output = Path::new(&output);
    assert!(!output.exists(), "refuse existing destination");
    std::fs::create_dir_all(output).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inputs: Vec<_> = ["jump.ogg", "coin.ogg", "land.ogg"]
        .into_iter()
        .map(|n| {
            (
                n,
                Arc::<[u8]>::from(
                    std::fs::read(root.join("samples/public_audio/sounds").join(n)).unwrap(),
                ),
            )
        })
        .collect();
    let bank = bank::derive(&inputs, 64 * 1024 * 1024).unwrap();
    std::fs::write(
        output.join("bank-audit.json"),
        serde_json::to_vec_pretty(&bank.audit).unwrap(),
    )
    .unwrap();
    for (folder, api) in [("project2d", Api::Audio2V1), ("project3d", Api::Audio3V1)] {
        let dest = output.join(folder);
        std::fs::create_dir(&dest).unwrap();
        let mut registry = Registry::new();
        let mut doc = Document::empty(format!("plan012-{folder}"));
        doc.schema = "autographics.scene/7".into();
        let mut actor = Object::new(
            1,
            if api == Api::Audio3V1 {
                Kind::Model
            } else {
                Kind::Quad
            },
        );
        if api == Api::Audio3V1 {
            for (i, name) in ["character.glb", "coin.glb", "flag.glb", "platform.glb"]
                .into_iter()
                .enumerate()
            {
                let path = root.join("samples/textured_models/models").join(name);
                let asset = if i == 0 {
                    assets::import_animated(&path, &registry)
                } else {
                    assets::import(&path, &registry)
                }
                .unwrap();
                if i == 0 {
                    let (lo, hi) =
                        autographics_auto_driver::authoring::interaction::bounds(&asset.model)
                            .unwrap();
                    let e = hi - lo;
                    actor.scale = [0.8 / e.x, 1.0 / e.y, 0.8 / e.z];
                    actor.resource = Some(asset.identity.key());
                } else {
                    let mut o = Object::new(i as u64 + 3, Kind::Model);
                    o.translation = [20.0 + i as f32 * 3.0, 0.0, 0.0];
                    o.scale = [0.3, 0.3, 0.3];
                    o.resource = Some(asset.identity.key());
                    doc.objects.push(o);
                }
                assets::insert(&mut registry, asset).unwrap();
            }
        } else {
            let mut pixels = vec![];
            for _ in 0..8 {
                for x in 0..8 {
                    pixels.extend_from_slice(if x < 4 {
                        &[255, 0, 0, 255]
                    } else {
                        &[0, 0, 255, 255]
                    });
                }
            }
            let png = agp::assets::encode_png_rgba(8, 8, &pixels).unwrap();
            let asset =
                assets::decode_png_in(png, Some("generated-two-colors.png".into()), &registry)
                    .unwrap();
            actor.scale = [0.8, 1.0, 1.0];
            actor.sprite = Some(SpriteBinding {
                asset: asset.identity.key(),
                frame: [0, 0, 4, 8],
                flip: false,
            });
            assets::insert(&mut registry, asset).unwrap();
        }
        doc.objects.insert(0, actor);
        let mut template = Object::new(
            3,
            if api == Api::Audio3V1 {
                Kind::Cylinder
            } else {
                Kind::Circle
            },
        );
        template.translation = [20.0, 0.0, 0.0];
        template.scale = [0.3, 0.5, if api == Api::Audio3V1 { 0.3 } else { 1.0 }];
        doc.objects.push(template);
        let sound = public_audio::import_raw(
            bank.bytes.clone().into(),
            &registry,
            Some("three-cue-bank.wav".into()),
        )
        .unwrap();
        doc.cues = Some(
            bank.spans
                .iter()
                .enumerate()
                .map(|(i, (start, end))| CueRef {
                    name: ["jump", "coin", "land"][i].into(),
                    asset: sound.identity.key(),
                    start_frame: *start,
                    end_frame: *end,
                    gain: 1.0,
                })
                .collect(),
        );
        assets::insert(&mut registry, sound).unwrap();
        doc.assets = registry.values().map(|a| a.identity.clone()).collect();
        let source = root.join(format!(
            "samples/public_audio/{}.at",
            if api == Api::Audio3V1 {
                "audio3d"
            } else {
                "audio2d"
            }
        ));
        let code = CodePayload::read_for(&source, api).unwrap();
        doc.behavior = Some(Binding {
            api: api.version().into(),
            source: code.identity().clone(),
            actors: Actors {
                two_d: if api == Api::Audio2V1 { Some(1) } else { None },
                three_d: if api == Api::Audio3V1 { Some(1) } else { None },
            },
        });
        let mut editor = Editor::with_project(doc, registry, Some(code)).unwrap();
        assert_eq!(
            Storage::default()
                .save(&mut editor, &dest.join("scene.agscene"), Fault::None)
                .unwrap(),
            Saved::Confirmed
        );
        for name in ["SOURCE-README.md", "SOURCE-LICENSE.md"] {
            std::fs::copy(
                root.join("samples/public_audio").join(name),
                dest.join(name),
            )
            .unwrap();
        }
        std::fs::write(dest.join("README.md"),format!("PLAN-012 finite {folder} engine fixture. scene.agscene binds immutable Auto, visuals and one three-cue PCM16 bank. Sources and CC0 art/sound provenance: SOURCE-README.md; upstream code license separately in SOURCE-LICENSE.md. Generated two-color image (2D) is original test art, CC0. No font/HUD/full-game delivery.\n")).unwrap();
        println!("saved {}", dest.display());
    }
}
