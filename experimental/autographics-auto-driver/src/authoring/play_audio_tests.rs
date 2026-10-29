//! Corrupt actual real-VM output, then run the ordinary Play/mixer admission.
use super::super::{
    behavior::{Api, SourceSnapshot},
    document::Kind,
    public_audio::{self, mixer::Mixer, CueRef},
    spatial::Descriptor,
};
use super::*;
use std::path::Path;
fn candidate(api: Api) -> Play {
    let mut raw = b"RIFF".to_vec();
    raw.extend_from_slice(&44u32.to_le_bytes());
    raw.extend_from_slice(b"WAVEfmt ");
    raw.extend_from_slice(&16u32.to_le_bytes());
    raw.extend_from_slice(&1u16.to_le_bytes());
    raw.extend_from_slice(&1u16.to_le_bytes());
    raw.extend_from_slice(&44100u32.to_le_bytes());
    raw.extend_from_slice(&88200u32.to_le_bytes());
    raw.extend_from_slice(&2u16.to_le_bytes());
    raw.extend_from_slice(&16u16.to_le_bytes());
    raw.extend_from_slice(b"data");
    raw.extend_from_slice(&8u32.to_le_bytes());
    raw.extend_from_slice(&[0, 64].repeat(4));
    let mut registry = assets::Registry::new();
    let audio = public_audio::import_raw(raw.into(), &registry, None).unwrap();
    assets::insert(&mut registry, audio.clone()).unwrap();
    let mut d = Document::empty("audio-wire".into());
    d.schema = "autographics.scene/7".into();
    d.assets.push(audio.identity.clone());
    d.cues = Some(vec![CueRef {
        name: "one".into(),
        asset: audio.identity.key(),
        start_frame: 0,
        end_frame: 4,
        gain: 1.0,
    }]);
    let mut actor = Object::new(
        1,
        if api == Api::Audio3V1 {
            Kind::Model
        } else {
            Kind::Quad
        },
    );
    let model = if api == Api::Audio3V1 {
        let asset = assets::import_animated(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/textured_models/models/character.glb"),
            &registry,
        )
        .unwrap();
        let (lo, hi) = super::super::interaction::bounds(&asset.model).unwrap();
        let size = hi - lo;
        actor.scale = [0.8 / size.x, 1.0 / size.y, 0.8 / size.z];
        actor.resource = Some(asset.identity.key());
        d.assets.push(asset.identity.clone());
        assets::insert(&mut registry, asset.clone()).unwrap();
        Some(asset)
    } else {
        actor.scale = [0.8, 1.0, 1.0];
        None
    };
    d.objects.push(actor);
    let e = Editor::with_assets(d, registry).unwrap();
    let start = if api == Api::Audio3V1 {
        "ok=ag_char3_bind(1);ok=ag_camera_orbit_3d(1,8.0,45.0,0,0.0)"
    } else {
        "ok=ag_char_bind(1);ok=ag_camera_follow_2d(1,-10.0,-10.0,10.0,10.0,4.0,4.0)"
    };
    let source=format!("var ok=0\nfn app_start(){{{start}}}\nfn app_update(dt float){{ok=ag_audio_emit(1,0,0.5)}}\n");
    let snapshot =
        SourceSnapshot::from_bytes_for(Path::new("audio-wire.at"), source.into_bytes(), api)
            .unwrap();
    let descriptor = Descriptor::build(&e.document, |_| model.as_ref().map(|m| &m.model)).unwrap();
    let mut p = Play::new_user_spatial(&e, [None, None], snapshot, descriptor).unwrap();
    let ready = |p: &mut Play| {
        let end = Instant::now() + Duration::from_secs(10);
        while p.candidate().is_none() {
            assert!(Instant::now() < end);
            p.pump_until(Instant::now() + Duration::from_millis(8))
                .unwrap();
        }
    };
    ready(&mut p);
    p.accept().unwrap();
    p.dispatch(Action::default()).unwrap();
    ready(&mut p);
    p
}
#[test]
fn real_audio_wire_mutations_cannot_accept_partial_frame_or_insert_pcm() {
    use Scalar::{Float, Int};
    for api in [Api::Audio2V1, Api::Audio3V1] {
        let mut p = candidate(api);
        let original = p.read.clone();
        let base = original.len() - 16;
        let cues = p.audio_cues.clone().unwrap();
        let mut mixer = Mixer::default();
        mixer.begin(p.run_id).unwrap();
        let mut controls: Vec<(&str, Vec<Scalar>)> = vec![];
        for (name, offset, value) in [
            ("floatRun", 0, Float(p.run_id as f32)),
            ("oldRun", 0, Int(p.run_id + 1)),
            ("oldTick", 1, Int(0)),
            ("floatTick", 1, Float(1.0)),
            ("floatBundle", 2, Float(1.0)),
            ("badBundle", 2, Int(2)),
            ("floatCount", 3, Float(1.0)),
            ("largeCount", 3, Int(5)),
            ("floatOrdinal", 4, Float(16777216.0)),
            ("zeroOrdinal", 4, Int(0)),
            ("exhaustedOrdinal", 4, Int(i32::MAX)),
            ("floatCue", 5, Float(0.0)),
            ("missingCue", 5, Int(1)),
            ("negativeCue", 5, Int(-1)),
            ("integerGain", 6, Int(1)),
            ("nanGain", 6, Float(f32::NAN)),
            ("infiniteGain", 6, Float(f32::INFINITY)),
            ("largeGain", 6, Float(1.01)),
            ("negativeGain", 6, Float(-0.1)),
            ("unusedOrdinal", 7, Int(1)),
            ("unusedGain", 9, Float(f32::NAN)),
        ] {
            let mut input = original.clone();
            input[base + offset] = value;
            controls.push((name, input));
        }
        let mut short = original.clone();
        short.pop();
        controls.push(("short", short));
        let mut duplicate = original.clone();
        duplicate.push(Int(0));
        controls.push(("extra", duplicate));
        let mut duplicate_ordinal = original.clone();
        duplicate_ordinal[base + 3] = Int(2);
        duplicate_ordinal[base + 7] = Int(1);
        duplicate_ordinal[base + 8] = Int(0);
        duplicate_ordinal[base + 9] = Float(0.5);
        controls.push(("duplicate ordinal", duplicate_ordinal));
        for (name, input) in controls {
            p.candidate = None;
            p.read = input;
            match p.admit() {
                Err(_) => {}
                Ok(frame) => {
                    let packet = frame.audio.clone().unwrap();
                    p.candidate = Some(frame);
                    assert!(
                        mixer.accept(packet, &cues, false, || p.accept()).is_err(),
                        "{api:?}: {name}"
                    );
                }
            }
            assert_eq!(
                p.last.as_ref().unwrap().tick,
                0,
                "{name}: old logical frame preserved"
            );
            assert_eq!(mixer.voices(), 0, "{name}: no partial sound");
            let mut pcm = [[9.0; 2]; 8];
            mixer.render(&mut pcm, 44100);
            assert_eq!(pcm, [[0.0; 2]; 8]);
        }
        // A value above f32's exact integer domain remains exact through VM
        // integer wire, independent Host validation and accepted packet.
        p.read = original;
        p.read[base + 4] = Int(16777217);
        p.candidate = Some(p.admit().unwrap());
        let packet = p.candidate.as_ref().unwrap().audio.clone().unwrap();
        assert_eq!(packet.requests[0].ordinal, 16777217);
        mixer.accept(packet, &cues, false, || p.accept()).unwrap();
        assert_eq!(p.last.as_ref().unwrap().tick, 1);
        let mut pcm = [[0.0; 2]; 8];
        mixer.render(&mut pcm, 44100);
        assert_eq!(&pcm[..4], &[[0.25; 2]; 4]);
        assert_eq!(&pcm[4..], &[[0.0; 2]; 4]);
    }
}
