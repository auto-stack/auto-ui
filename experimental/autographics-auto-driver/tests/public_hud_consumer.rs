//! Public, source-bound projects: identical entrypoint in an exported consumer.
use autographics_auto_driver::authoring::{
    assets::{self, Registry},
    behavior::{Api, SourceSnapshot},
    host::Host,
    play::{Action, State},
    public_audio::{
        self,
        mixer::{Mixer, Packet},
    },
    storage::{Fault, Saved, Storage},
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/public_hud")
}
fn open(path: &Path) -> Host {
    let (d, r, c, _) = Storage::load_project(path, &Registry::new()).unwrap();
    assert!(c.is_some());
    Host::with_project(d, r, c).unwrap()
}
fn pump(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(20);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "pump deadline");
        h.frame(None).unwrap();
    }
    assert_ne!(
        h.play.as_ref().unwrap().state,
        State::Faulted,
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}
fn start(h: &mut Host) {
    h.prepare_measurement.enable(true);
    h.publish_measurement.enable(true);
    h.start_selected_play().unwrap();
    h.pause_play("");
    pump(h);
}
fn input(h: &mut Host) {
    let p = h.play.as_mut().unwrap();
    p.resume().unwrap();
    p.char_key(1, true, false);
    p.dispatch(Action::default()).unwrap();
    h.pause_play("");
    pump(h);
}
fn hud_text(h: &Host, slot: usize) -> String {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .unwrap()
        .hud
        .as_ref()
        .unwrap()
        .slots[slot]
        .text
        .clone()
}

fn report(name: &str, v: &Value) {
    println!("UI013 {name} {v}");
    if let Some(dir) = std::env::var_os("UI013_REPORT_DIR") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.json")),
            serde_json::to_vec_pretty(v).unwrap(),
        )
        .unwrap();
    }
}
fn summary(v: &[u64]) -> Value {
    assert!(!v.is_empty());
    let mut v = v.to_vec();
    v.sort();
    json!({"count":v.len(),"median_us":v[v.len()/2] as f64/1000.0,"max_us":v.last().unwrap().to_owned() as f64/1000.0})
}
// Independent PCM16 RIFF reader for this canonical derived bank: does not use
// production codec, cue resolver or mixer interpolation to compute the oracle.
fn pcm_oracle(h: &Host, packet: Packet, first: usize, gain: f32) -> Vec<[f32; 2]> {
    assert_eq!(packet.requests.len(), 2);
    assert_eq!(
        (packet.requests[0].cue, packet.requests[0].gain),
        (first, gain)
    );
    assert_eq!((packet.requests[1].cue, packet.requests[1].gain), (2, 0.5));
    let sound = h
        .editor
        .assets
        .values()
        .find(|a| a.audio.is_some())
        .unwrap();
    assert_eq!(
        sound.identity.sha256,
        "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d"
    );
    let bytes = &sound.bytes;
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[36..40], b"data");
    let source: Vec<_> = bytes[44..]
        .chunks_exact(4)
        .map(|f| {
            [
                f32::from(i16::from_le_bytes([f[0], f[1]])) / 32768.0,
                f32::from(i16::from_le_bytes([f[2], f[3]])) / 32768.0,
            ]
        })
        .collect();
    let spans = [(0, 7403), (7403, 20046), (20046, 22504)];
    let mut expected = vec![[0.0; 2]; 12645];
    for (cue, g) in [(first, gain), (2, 0.5)] {
        let (a, b) = spans[cue];
        for (dst, sample) in expected.iter_mut().zip(&source[a..b]) {
            for ch in 0..2 {
                dst[ch] += sample[ch] * g;
            }
        }
    }
    let cues = public_audio::resolve_cues(&h.editor.document, &h.editor.assets).unwrap();
    assert!(std::sync::Arc::ptr_eq(&cues[0].pcm, &cues[2].pcm));
    let mut mixer = Mixer::default();
    mixer.begin(packet.run).unwrap();
    mixer.accept(packet, &cues, false, || Ok(())).unwrap();
    let mut actual = vec![[0.0; 2]; expected.len()];
    mixer.render(&mut actual, 44100);
    assert_eq!(
        actual, expected,
        "actual chosen span and gain match raw PCM16 independent oracle"
    );
    assert_eq!(mixer.voices(), 0);
    assert!(actual.iter().any(|f| f[0] != 0.0));
    actual
}
#[test]
fn bound_two_dimensional_projects_move_cold_reopen_and_source_changes_cue_gain_and_motion() {
    let mut results = vec![];
    for (name, api) in [("project2d", Api::Hud2V1), ("project3d", Api::Hud3V1)] {
        let entry = root().join(name).join("scene.agscene");
        let mut h = open(&entry);
        start(&mut h);
        let initial = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation;
        input(&mut h);
        let packet = h
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .audio
            .clone()
            .unwrap();
        let normal_pcm = pcm_oracle(&h, packet, 0, 0.25);
        let normal = h.play.as_ref().unwrap().last.as_ref().unwrap().nodes[&1].translation;
        assert_ne!(normal, initial);
        assert_eq!(hud_text(&h, 0), "1");
        assert_eq!(hud_text(&h, 1), "Playing");
        input(&mut h);
        let packet = h
            .play
            .as_ref()
            .unwrap()
            .last
            .as_ref()
            .unwrap()
            .audio
            .clone()
            .unwrap();
        assert_eq!(packet.requests[0].ordinal, 3);
        assert_eq!(hud_text(&h, 0), "2");
        assert_eq!(hud_text(&h, 1), "Won");
        pcm_oracle(&h, packet, 1, 0.25); // real trigger event101 from previous tick
        let source = h.editor.code.as_ref().unwrap().bytes().to_vec();
        let accepted = h.editor.document.clone();
        h.stop_play();
        let dest = std::env::temp_dir().join(format!(
            "ui012-project-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let a = dest.join("a");
        std::fs::create_dir_all(&a).unwrap();
        assert_eq!(
            h.storage
                .save(&mut h.editor, &a.join("scene.agscene"), Fault::None)
                .unwrap(),
            Saved::Confirmed
        );
        for license in [
            "README.md",
            "SOURCE-README.md",
            "SOURCE-LICENSE.md",
            "FONT-LICENSE.txt",
        ] {
            std::fs::copy(root().join(name).join(license), a.join(license)).unwrap();
        }
        drop(h);
        let b = dest.join("b");
        std::fs::rename(&a, &b).unwrap();
        assert!(!a.exists());
        let mut h = open(&b.join("scene.agscene"));
        assert_eq!(h.editor.document, accepted);
        start(&mut h);
        input(&mut h);
        let reopened = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert_eq!(reopened.nodes[&1].translation, normal);
        assert_eq!(hud_text(&h, 0), "1");
        assert_eq!(hud_text(&h, 1), "Playing");
        assert_eq!(
            pcm_oracle(&h, reopened.audio.clone().unwrap(), 0, 0.25),
            normal_pcm
        );
        h.stop_play();
        let raw = String::from_utf8(source).unwrap();
        let changed = raw
            .replace("var choice=0", "var choice=2")
            .replace("0.25", "0.75")
            .replace("var speed=1.0", "var speed=0.25");
        assert_ne!(changed, raw);
        let snapshot = SourceSnapshot::from_bytes_for(
            Path::new("edited-public-audio.at"),
            changed.into_bytes(),
            api,
        )
        .unwrap();
        h.start_user_snapshot(snapshot).unwrap();
        h.pause_play("");
        pump(&mut h);
        input(&mut h);
        let frame = h.play.as_ref().unwrap().last.as_ref().unwrap();
        let changed_pcm = pcm_oracle(&h, frame.audio.clone().unwrap(), 2, 0.75);
        assert_ne!(normal_pcm, changed_pcm, "source must control real waveform");
        let changed_x = (frame.nodes[&1].translation[0] - initial[0]).abs();
        assert!((normal[0] - initial[0]).abs() > changed_x * 3.9);
        assert_eq!(hud_text(&h, 0), "1");
        h.stop_play();
        results.push(json!({"profile":api.version(),"moved_project":true,"cold_reopen":true,"user_mutation_sensitive":true,"cue_gain_waveform_sensitive":true,"event_selects_cue1":true,"pcm_frames_compared":12645,"source_sha256":accepted.behavior.unwrap().source.sha256,"licenses_moved":true,"hud_dynamic":true}));
    }
    report(
        "consumer",
        &json!({"manifest_dir":env!("CARGO_MANIFEST_DIR"),"projects":results}),
    );
}
#[test]
fn complete_hud_profiles_keep_cpu_source_resource_guards_and_report_native_cost_separately() {
    let mut results = vec![];
    for name in ["project2d", "project3d"] {
        let mut h = open(&root().join(name).join("scene.agscene"));
        start(&mut h);
        let decodes = assets::decode_attempts();
        let atlas_builds = autographics_auto_driver::authoring::public_hud::atlas_builds();
        let resources = h.resource_snapshot();
        let uploads = resources["mesh_uploads"].clone();
        let texture = resources["model_texture_uploads"].clone();
        h.play_pump_us.clear();
        h.prepare_measurement.enable(true);
        h.publish_measurement.enable(true);
        let p = h.play.as_mut().unwrap();
        p.vm_output_measurement.enable(true);
        p.admission_measurement.enable(true);
        for _ in 0..12 {
            input(&mut h);
        }
        let p = h.play.as_ref().unwrap();
        assert_eq!(p.last.as_ref().unwrap().tick, 12);
        assert_eq!(hud_text(&h, 0), "12");
        assert_eq!(hud_text(&h, 1), "Failed");
        assert_eq!(
            autographics_auto_driver::authoring::public_hud::atlas_builds(),
            atlas_builds
        );
        let (steps, slices) = p.callback_high_water().unwrap();
        assert!(steps > 0 && steps <= 100000 && slices <= 121);
        let source = p.user_source.as_ref().unwrap();
        let managed = source.source.len() - source.bytes.len();
        assert!(
            managed <= 96 * 1024
                && source.bytes.len() <= 256 * 1024
                && source.source.len() <= 352 * 1024
        );
        assert_eq!(assets::decode_attempts(), decodes);
        let resources = h.resource_snapshot();
        assert_eq!(resources["mesh_uploads"], uploads);
        assert_eq!(resources["model_texture_uploads"], texture);
        let u = &resources["asset_union"];
        assert!(
            u["count"].as_u64().unwrap() <= 8
                && u["raw_bytes"].as_u64().unwrap() <= 32 * 1024 * 1024
        );
        assert!(
            u["cpu_total_with_pose_bytes"].as_u64().unwrap() <= 64 * 1024 * 1024
                && u["gpu_expected_bytes"].as_u64().unwrap() <= 64 * 1024 * 1024
        );
        let vm = summary(&p.vm_output_measurement.samples_ns);
        let admission = summary(&p.admission_measurement.samples_ns);
        let ticks: Vec<_> = p
            .reports
            .iter()
            .filter(|r| r.tick > 0)
            .map(|r| r.elapsed_us as u64 * 1000)
            .collect();
        assert!(ticks.iter().all(|v| *v < 2_000_000_000));
        let pump = summary(
            &h.play_pump_us
                .iter()
                .map(|v| *v as u64 * 1000)
                .collect::<Vec<_>>(),
        );
        let prepare = summary(&h.prepare_measurement.samples_ns);
        let publish = summary(&h.publish_measurement.samples_ns);
        let run = p.run_id();
        let user_hash = source.user_hash.clone();
        let combined_hash = p.source_hash.clone();
        let tick = summary(&ticks);
        // Ordinary Host Resume reaches the already-open service stream; output
        // polling bounds software service latency, not physical speaker latency.
        let begin = Instant::now();
        let token = h.ui().play_token;
        h.dispatch(&json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":token,"action":"play-resume"}).to_string()).unwrap();
        h.frame(None).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while h
            .audio
            .as_ref()
            .unwrap()
            .observation
            .lock()
            .unwrap()
            .nonzero
            == 0
        {
            assert!(Instant::now() < deadline, "real native output required");
            std::thread::sleep(Duration::from_millis(1));
        }
        let latency_upper = begin.elapsed().as_micros();
        while h.audio.as_ref().unwrap().mixer.lock().unwrap().voices() != 0 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        h.pause_play("");
        let native = h.audio_snapshot();
        assert!(native["nonzero"].as_u64().unwrap() > 0);
        assert_eq!(native["run"], run);
        results.push(json!({"project":name,"ticks":12,"user_sha256":user_hash,"combined_sha256":combined_hash,"managed_bytes":managed,"steps":steps,"slices":slices,"tick":tick,"vm_output":vm,"admission":admission,"host_prepare":prepare,"host_publish":publish,"host_pump":pump,"pump_target_us":8000,"pump_target":if pump["max_us"].as_f64().unwrap()<=8000.0 {"met"}else{"missed"},"resources":u,"warm_decode_delta":assets::decode_attempts()-decodes,"warm_mesh_uploads":0,"warm_texture_uploads":0,"warm_atlas_builds_delta":autographics_auto_driver::authoring::public_hud::atlas_builds()-atlas_builds,"native":native,"resume_to_first_pcm_upper_us":latency_upper,"observation_poll_us":1000,"hardware_latency":"not measured; already buffered samples cannot be withdrawn","underrun_counter":"not exposed by this CPAL backend"}));
        h.stop_play();
    }
    report(
        "performance",
        &json!({"manifest_dir":env!("CARGO_MANIFEST_DIR"),"guards":{"credits":32,"slice_steps":1000,"callback_steps":100000,"resumptions":120,"callback_ms":250,"init_ms":30000,"tick_ms":2000,"pump_target_us":8000},"projects":results}),
    );
}
