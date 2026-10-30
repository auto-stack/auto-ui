use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::Api,
    behavior_project::{Actors, Binding, CodePayload},
    document::{Document, Kind, Object, SpriteBinding},
    public_audio::{self, CueRef},
    public_hud,
    session::Editor,
    storage::{Fault, Saved, Storage},
};
use std::path::Path;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .expect("destination directory required");
    let output = Path::new(&output);
    assert!(!output.exists(), "refuse existing destination");
    std::fs::create_dir_all(output).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (folder, api) in [("project2d", Api::Hud2V1), ("project3d", Api::Hud3V1)] {
        let dest = output.join(folder);
        std::fs::create_dir(&dest).unwrap();
        let mut registry = Registry::new();
        let mut doc = Document::empty(format!("plan013-{folder}"));
        doc.schema = "autographics.scene/8".into();
        let mut actor = Object::new(
            1,
            if api == Api::Hud3V1 {
                Kind::Model
            } else {
                Kind::Quad
            },
        );
        if api == Api::Hud3V1 {
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
                    let extent = hi - lo;
                    actor.scale = [0.8 / extent.x, 1.0 / extent.y, 0.8 / extent.z];
                    actor.resource = Some(asset.identity.key());
                } else {
                    let mut object = Object::new(i as u64 + 3, Kind::Model);
                    object.translation = [20.0 + i as f32 * 3.0, 0.0, 0.0];
                    object.scale = [0.3, 0.3, 0.3];
                    object.resource = Some(asset.identity.key());
                    doc.objects.push(object);
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
            let asset = assets::decode_png_in(
                agp::assets::encode_png_rgba(8, 8, &pixels).unwrap(),
                Some("generated-two-colors.png".into()),
                &registry,
            )
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
            if api == Api::Hud3V1 {
                Kind::Cylinder
            } else {
                Kind::Circle
            },
        );
        template.translation = [20.0, 0.0, 0.0];
        template.scale = [0.3, 0.5, if api == Api::Hud3V1 { 0.3 } else { 1.0 }];
        doc.objects.push(template);
        let sound = public_audio::import(
            &root.join("samples/public_audio/project3d/assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav"),
            &registry,
        )
        .unwrap();
        doc.cues = Some(
            [(0, 7403), (7403, 20046), (20046, 22504)]
                .into_iter()
                .enumerate()
                .map(|(i, (start_frame, end_frame))| CueRef {
                    name: ["jump", "coin", "land"][i].into(),
                    asset: sound.identity.key(),
                    start_frame,
                    end_frame,
                    gain: 1.0,
                })
                .collect(),
        );
        assets::insert(&mut registry, sound).unwrap();
        let font = public_hud::import(
            &root.join("samples/public_hud/fonts/lilita_one_regular.ttf"),
            &registry,
        )
        .unwrap();
        doc.hud = Some(public_hud::default_document(font.identity.key()));
        assets::insert(&mut registry, font).unwrap();
        doc.assets = registry
            .values()
            .map(|asset| asset.identity.clone())
            .collect();
        let source = root.join(format!(
            "samples/public_hud/{}.at",
            if api == Api::Hud3V1 { "hud3d" } else { "hud2d" }
        ));
        let code = CodePayload::read_for(&source, api).unwrap();
        doc.behavior = Some(Binding {
            api: api.version().into(),
            source: code.identity().clone(),
            actors: Actors {
                two_d: if api == Api::Hud2V1 { Some(1) } else { None },
                three_d: if api == Api::Hud3V1 { Some(1) } else { None },
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
        std::fs::copy(
            root.join("samples/public_hud/fonts/license.txt"),
            dest.join("FONT-LICENSE.txt"),
        )
        .unwrap();
        std::fs::write(
            dest.join("README.md"),
            format!("PLAN-013 finite {folder} public HUD fixture. scene.agscene binds Auto, visual, three-cue PCM bank and fixed Lilita One font. Font OFL/RFN is embedded in scene8 and retained separately in FONT-LICENSE.txt. Other art/sound provenance and upstream code license are separate. This is not a complete game or Gallery package.\n"),
        )
        .unwrap();
        println!("saved {}", dest.display());
    }
}
