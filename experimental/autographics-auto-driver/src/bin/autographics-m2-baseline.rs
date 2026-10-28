//! Research runner: real Host/Auto execution, no game logic replacement.
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Dimension, Document, Group, Kind, Object},
    host::Host,
    play::{Action, State},
    storage::Storage,
};
use serde_json::{json, Value};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn settle(h: &mut Host) -> Result<Vec<Value>, String> {
    let mut frames = vec![];
    let end = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().ok_or("missing Play")?.pending() {
        if Instant::now() >= end {
            return Err("baseline initialization timeout".into());
        }
        let before = h.rt.upload_stats();
        let start = Instant::now();
        h.frame(None)?;
        let frame_ns = start.elapsed().as_nanos();
        let after = h.rt.upload_stats();
        frames.push(json!({"frame_ns":frame_ns,"pending_after":h.play.as_ref().unwrap().pending(),"buffer_rewrites":after.buffer_rewrites-before.buffer_rewrites,"bytes_rewritten":after.bytes_rewritten-before.bytes_rewritten}));
    }
    let p = h.play.as_ref().unwrap();
    if p.state != State::Paused {
        return Err(format!("baseline rejected: {:?}", p.error));
    }
    Ok(frames)
}
fn pixels(h: &mut Host) -> Result<Value, String> {
    Ok(json!([
        h.rt.capture(h.two, "m2-2d").ok_or("2d capture")?.0,
        h.rt.capture(h.three, "m2-3d").ok_or("3d capture")?.0
    ]))
}
fn input(
    case: &str,
) -> Result<
    (
        Document,
        autographics_auto_driver::authoring::assets::Registry,
        Option<autographics_auto_driver::authoring::behavior_project::Code>,
        Vec<u8>,
        Api,
    ),
    String,
> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if matches!(case, "wide128" | "deep128") {
        let deep = case == "deep128";
        let mut d = Document::empty(case.into());
        let groups = if deep { 126 } else { 3 };
        for i in 0..groups {
            let mut g = Group::new(i + 1, Dimension::Two);
            if i > 0 {
                g.parent = Some(if deep { i } else { 1 });
            }
            d.groups.push(g);
        }
        for i in groups..128 {
            let mut o = Object::new(i + 1, Kind::Circle);
            o.parent = Some(if deep {
                groups
            } else if i % 2 == 0 {
                2
            } else {
                3
            });
            d.objects.push(o);
        }
        // Original motion_host 128-node initialization/query oracle, unchanged math.
        let raw=b"fn app_start(){}\nfn app_update(dt float){ag_set_local(0,0,2.0);let q=ag_move_and_slide(126,0,8.0,0.0,0.0,0.25);let s=ag_motion_status(q);let c=ag_motion_contacts(q);let x=ag_motion_delta(q,0);let y=ag_motion_delta(q,1);let z=ag_motion_delta(q,2);let nx=ag_motion_normal(q,0);let ny=ag_motion_normal(q,1);let nz=ag_motion_normal(q,2);ag_set_local(0,0,4.0);if s!=3 || c!=0 {ag_set_visible(127,0)}}\n";
        return Ok((d, Default::default(), None, raw.to_vec(), Api::MotionV5));
    }
    let folder = match case {
        "move" => "behavior_projects/move",
        "maze2d" => "motion_projects/maze2d",
        "corridor3d" => "motion_projects/corridor3d",
        _ => return Err("unknown baseline case".into()),
    };
    let (d, a, c, _) = Storage::load_project(
        &root.join("samples").join(folder).join("scene.agscene"),
        &Default::default(),
    )?;
    let code = c.as_ref().ok_or("fixture has no code")?;
    let api = Api::parse(&d.behavior.as_ref().ok_or("fixture unbound")?.api)?;
    let raw = code.bytes().to_vec();
    Ok((d, a, c, raw, api))
}
fn generated(out: &Path) -> Result<(), String> {
    std::fs::create_dir(out).map_err(|e| e.to_string())?;
    let mut records = vec![];
    for api in [
        Api::V1,
        Api::SpatialV2,
        Api::SegmentV3,
        Api::SweepV4,
        Api::MotionV5,
    ] {
        for crlf in [false, true] {
            // Local reuse is accepted; no new public or reserved names.
            let raw="// Unicode 基线\nfn app_start(){let reused=1}\nfn app_update(dt float){let reused=2}\n";
            let raw = if crlf {
                raw.replace('\n', "\r\n")
            } else {
                raw.into()
            };
            let s = SourceSnapshot::from_bytes_for(
                Path::new("fixture.at"),
                raw.as_bytes().to_vec(),
                api,
            )?;
            let name = format!("v{}-{}", api.number(), if crlf { "crlf" } else { "lf" });
            std::fs::write(out.join(format!("{name}.combined.at")), s.source.as_bytes())
                .map_err(|e| e.to_string())?;
            std::fs::write(out.join(format!("{name}.user.at")), s.bytes.as_slice())
                .map_err(|e| e.to_string())?;
            records.push(json!({"name":name,"snapshot":s,"profile":api.profile(),"managed_bytes":s.source.len()-s.bytes.len(),"combined_bytes":s.source.len()}));
        }
    }
    std::fs::write(
        out.join("generated.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .map_err(|e| e.to_string())
}
fn run(case: &str, enabled: bool) -> Result<Value, String> {
    let process_start = Instant::now();
    let read_start = Instant::now();
    let (d, a, c, raw, api) = input(case)?;
    let read_ns = read_start.elapsed().as_nanos();
    let generate_start = Instant::now();
    let snapshot = SourceSnapshot::from_bytes_for(Path::new("baseline.at"), raw, api)?;
    let generate_ns = generate_start.elapsed().as_nanos();
    let identity = json!({"source":snapshot.user_hash,"api":snapshot.api_hash,"bridge":snapshot.bridge_hash,"combined":snapshot.combined_hash,"version":snapshot.version,"profile":api.profile(),"user_bytes":snapshot.bytes.len(),"managed_bytes":snapshot.source.len()-snapshot.bytes.len(),"combined_bytes":snapshot.source.len()});
    let host_start = Instant::now();
    let mut h = Host::with_project(d, a, c)?;
    let host_init_ns = host_start.elapsed().as_nanos();
    h.prepare_measurement.enable(enabled);
    let first = Instant::now();
    h.frame(None)?;
    let first_render_ns = first.elapsed().as_nanos();
    let process_to_first_render_ns = process_start.elapsed().as_nanos();
    let editing_pixels = pixels(&mut h)?;
    let before = h.resource_snapshot();
    let start = Instant::now();
    h.start_user_snapshot(snapshot)?;
    h.pause_play("");
    let synchronous_run_ns = start.elapsed().as_nanos();
    let vm_owner = h
        .play
        .as_ref()
        .unwrap()
        .owner_probe()
        .ok_or("no VM owner")?;
    let spatial_owner = h.play.as_ref().unwrap().spatial_owner_probe();
    let init_frames = settle(&mut h)?;
    let init_ns = start.elapsed().as_nanos();
    let compile_us = h.play.as_ref().unwrap().compile_us;
    let idle_before = h.rt.upload_stats();
    h.frame(None)?;
    let idle_after = h.rt.upload_stats();
    let idle_upload = json!({"buffer_rewrites":idle_after.buffer_rewrites-idle_before.buffer_rewrites,"bytes_rewritten":idle_after.bytes_rewritten-idle_before.bytes_rewritten});
    let init_resources = h.resource_snapshot();
    let initial_pixels = pixels(&mut h)?;
    let mut ticks = vec![];
    let mut warmup = vec![];
    for tick in 1..=35 {
        let start = Instant::now();
        h.play.as_mut().unwrap().dispatch(Action::default())?;
        let frames = settle(&mut h)?;
        let elapsed = start.elapsed().as_nanos();
        let p = h.play.as_ref().unwrap();
        if p.tick() != tick {
            return Err("accepted tick identity".into());
        }
        if case.ends_with("128")
            && (!p.last.as_ref().unwrap().objects.iter().all(|o| o.visible)
                || p.last.as_ref().unwrap().objects[0].translation[0] != 4.)
        {
            return Err("128 original motion oracle".into());
        }
        let row = json!({"tick":tick,"elapsed_ns":elapsed,"frames":frames,"document":p.last.as_ref().unwrap().document});
        if tick <= 5 {
            warmup.push(row)
        } else {
            ticks.push(row)
        }
    }
    let final_pixels = pixels(&mut h)?;
    let warm_resources = h.resource_snapshot();
    let p = h.play.as_ref().unwrap();
    let observations = json!({"vm_output_admission_pump":p.vm_output_measurement,"output_admission":p.admission_measurement,"renderer_prepare":h.prepare_measurement});
    let configuration =
        json!({"calls":p.spatial_configuration_calls,"us":p.spatial_configuration_us});
    let owner_live = vm_owner() && spatial_owner.as_ref().is_none_or(|w| w.upgrade().is_some());
    let stop = Instant::now();
    h.stop_play();
    let stop_ns = stop.elapsed().as_nanos();
    h.frame(None)?;
    let released = !vm_owner() && spatial_owner.as_ref().is_none_or(|w| w.upgrade().is_none());
    let restored = pixels(&mut h)?;
    if !owner_live || !released || restored != editing_pixels {
        return Err("owner/content recovery".into());
    }
    Ok(
        json!({"schema":"autographics.m2-baseline/1","case":case,"observer":enabled,"identity":identity,
        "adapter":h.rt.gpu().adapter_info(),"read_ns":read_ns,"generate_ns":generate_ns,"host_init_ns":host_init_ns,
        "first_render_ns":first_render_ns,"process_to_first_render_ns":process_to_first_render_ns,
        "run_sync_ns":synchronous_run_ns,"initialization_ns":init_ns,"initialization_frames":init_frames,"compile_us":compile_us,"stop_ns":stop_ns,
        "warmup":warmup,"ticks":ticks,"observations":observations,"configuration":configuration,
        "resources":{"before":before,"init":init_resources,"warm":warm_resources,"stop":h.resource_snapshot()},
        "pixels":{"editing":editing_pixels,"initial":initial_pixels,"final":final_pixels,"stop":restored},
        "idle_upload":idle_upload,"owners":{"alive_during_run":owner_live,"released_after_stop":released},"completed":true}),
    )
}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let result = if a.get(1).map(String::as_str) == Some("--generated") {
        a.get(2)
            .ok_or("missing output".into())
            .and_then(|p| generated(Path::new(p)))
    } else if a.len() == 4 && matches!(a[2].as_str(), "off" | "on") {
        run(&a[1], a[2] == "on").and_then(|v| {
            std::fs::write(&a[3], serde_json::to_vec_pretty(&v).unwrap()).map_err(|e| e.to_string())
        })
    } else {
        Err("usage: autographics-m2-baseline CASE off|on OUTPUT | --generated NEW_FOLDER".into())
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
