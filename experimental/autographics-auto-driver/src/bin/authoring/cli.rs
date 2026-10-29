//! Command line selection, final report and process exit.
use super::*;

pub(super) fn run() {
    let args: Vec<_> = std::env::args().collect();
    let value = |n: &str| {
        args.iter()
            .position(|v| v == n)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    // A supplied option without its value must not become the default API.
    if args.iter().any(|v| v == "--behavior-api")
        && value("--behavior-api").is_none_or(|v| v.starts_with("--"))
    {
        eprintln!("--behavior-api requires v1 or v2 or v3 or v4 or v5 or character or character3d or camera2d or camera3d or objects2d or objects3d or triggers2d or triggers3d");
        std::process::exit(2)
    }
    let behavior_api = match value("--behavior-api") {
        None => None,
        Some(v) => {
            if value("--behavior").is_none() {
                eprintln!("--behavior-api requires temporary --behavior");
                std::process::exit(2)
            }
            Some(match v.as_str() {
                "v1" => autographics_auto_driver::authoring::behavior::Api::V1,
                "v2" => autographics_auto_driver::authoring::behavior::Api::SpatialV2,
                "v5" => autographics_auto_driver::authoring::behavior::Api::MotionV5,
                "v4" => autographics_auto_driver::authoring::behavior::Api::SweepV4,
                "v3" => autographics_auto_driver::authoring::behavior::Api::SegmentV3,
                "character" => autographics_auto_driver::authoring::behavior::Api::CharacterV1,
                "character3d" => autographics_auto_driver::authoring::behavior::Api::Character3V1,
                "camera2d" => autographics_auto_driver::authoring::behavior::Api::Camera2V1,
                "camera3d" => autographics_auto_driver::authoring::behavior::Api::Camera3V1,
                "objects2d" => autographics_auto_driver::authoring::behavior::Api::Objects2V1,
                "objects3d" => autographics_auto_driver::authoring::behavior::Api::Objects3V1,
                "triggers2d" => autographics_auto_driver::authoring::behavior::Api::Triggers2V1,
                "triggers3d" => autographics_auto_driver::authoring::behavior::Api::Triggers3V1,
                _ => {
                    eprintln!(
                        "--behavior-api expects v1 or v2 or v3 or v4 or v5 or character or character3d or camera2d or camera3d or objects2d or objects3d or triggers2d or triggers3d"
                    );
                    std::process::exit(2)
                }
            })
        }
    };
    if args.iter().any(|v| {
        v == "--spatial-check"
            || v == "--segment-check"
            || v == "--sweep-check"
            || v == "--motion-check"
    }) && (value("--open").is_none()
        || !matches!(value("--spatial-space").as_deref(), Some("2") | Some("3")))
    {
        eprintln!("--spatial-check requires --open and --spatial-space 2 or 3");
        std::process::exit(2)
    }
    if let Some(path) = value("--inspect") {
        let result = (|| {
            let (doc, assets, code, _) =
                autographics_auto_driver::authoring::storage::Storage::load_project(
                    std::path::Path::new(&path),
                    &Default::default(),
                )?;
            let mut h = Host::with_project(doc, assets, code)?;
            h.frame(None)?;
            let two = h.rt.capture(h.two, "reopen-2d").ok_or("2d capture")?.0;
            let three = h.rt.capture(h.three, "reopen-3d").ok_or("3d capture")?.0;
            println!(
                "{}",
                serde_json::json!({"closure_verified":true,"document":h.editor.document,"hash":h.editor.document.hash()?,"two":two,"three":three,"history":h.editor.history(),"selection":h.editor.selection,"dirty":h.editor.dirty(),"world":h.editor.preview.objects,"interaction":h.interaction_snapshot()})
            );
            Ok::<_, String>(())
        })();
        if let Err(e) = result {
            eprintln!("{e}");
            std::process::exit(1)
        }
        return;
    }
    if args.iter().any(|v| v == "--behavior-check")
        && ((value("--behavior").is_none() && value("--open").is_none())
            || !matches!(
                value("--behavior-reference").as_deref().unwrap_or("move"),
                "move" | "move-fast" | "patrol"
            ))
    {
        let error = "behavior-check requires --behavior and a known independent behavior-reference";
        if let Some(path) = value("--report") {
            let _ = std::fs::write(
                path,
                serde_json::to_vec_pretty(&serde_json::json!({"passed":false,"error":error}))
                    .unwrap(),
            );
        }
        eprintln!("{error}");
        std::process::exit(2);
    }
    let mut app = App {
        viewport3d_probe: args
            .iter()
            .any(|v| v == "--viewport3d-check")
            .then(Default::default),
        viewport2d_probe: args
            .iter()
            .any(|v| v == "--viewport2d-check")
            .then(Default::default),
        character_probe: args
            .iter()
            .any(|v| v == "--character-check")
            .then(native_character2d::CharacterProbe::new),
        character3d_probe: args
            .iter()
            .any(|v| v == "--character3d-check")
            .then(native_character3d::Character3dProbe::new),
        runtime_camera_probe: args
            .iter()
            .any(|v| v == "--runtime-camera-check")
            .then(native_runtime_camera::RuntimeCameraProbe::new),
        play_probe: args
            .iter()
            .any(|v| v == "--play-check" || v == "--behavior-check")
            .then(|| {
                let mut probe = native_play::Probe::new(
                    value("--play-asset")
                        .unwrap_or_else(|| "research/asset-probe/two-parts.glb".into()),
                    args.iter().any(|v| v == "--wrong-play-focus"),
                    value("--play-gpu-fault"),
                    args.iter().any(|v| v == "--drop-play-tick"),
                );
                probe.step_delay_frames = value("--play-step-delay-frames")
                    .map(|v| v.parse::<u64>().expect("invalid native step delay"))
                    .unwrap_or(0);
                probe.drop_step = args.iter().any(|v| v == "--drop-play-step");
                probe.double_step = args.iter().any(|v| v == "--double-play-step");
                if args.iter().any(|v| v == "--behavior-check") {
                    probe.project_bind = args.iter().any(|v| v == "--project-bind-check");
                    probe.behavior_reference =
                        Some(value("--behavior-reference").unwrap_or_else(|| "move".into()));
                }
                probe
            }),
        asset_probe: args.iter().any(|v| v == "--asset-check").then(|| {
            let mut probe = native_assets::Probe::new(
                value("--asset-file").unwrap_or_default(),
                args.iter().any(|v| v == "--drop-release"),
                value("--asset-gpu-fault"),
            );
            probe.preparation_delay_frames = value("--probe-prepare-delay-frames")
                .map(|v| v.parse().expect("invalid probe preparation delay"))
                .unwrap_or(0);
            probe
        }),
        asset_save: value("--asset-save").map(PathBuf::from),
        wrong_asset: args.iter().any(|v| v == "--wrong-asset"),
        probe: args.iter().any(|v| v == "--window-probe"),
        interaction_probe: args
            .iter()
            .any(|v| v == "--interaction-probe" || v == "--interaction-check"),
        interaction_fault: value("--interaction-fault"),
        hierarchy_probe: args.iter().any(|v| v == "--hierarchy-check").then(|| {
            let mut probe = native_hierarchy::Probe::new(
                args.iter().any(|v| v == "--wrong-parent"),
                args.iter().any(|v| v == "--drop-release"),
            );
            probe.preparation_delay_frames = value("--probe-prepare-delay-frames")
                .map(|v| v.parse().expect("invalid probe preparation delay"))
                .unwrap_or(0);
            probe
        }),
        native_probe: args.iter().any(|v| v == "--interaction-check").then(|| {
            native_interaction::Probe::new(
                args.iter().any(|v| v == "--sabotage-target"),
                args.iter().any(|v| v == "--drop-release"),
            )
        }),
        limit: value("--frames").map(|v| v.parse().expect("frames")),
        open: value("--open").map(PathBuf::from),
        behavior: value("--behavior"),
        behavior_api,
        spatial_probe: args
            .iter()
            .any(|v| {
                v == "--spatial-check"
                    || v == "--segment-check"
                    || v == "--sweep-check"
                    || v == "--motion-check"
            })
            .then(|| {
                let mut probe = native_spatial::Probe::new(
                    value("--spatial-space").unwrap().parse().unwrap(),
                    args.iter().any(|v| v == "--spatial-narrow"),
                    value("--play-gpu-fault"),
                );
                probe.update = args.iter().any(|v| {
                    v == "--spatial-update"
                        || v == "--segment-update"
                        || v == "--sweep-update"
                        || v == "--motion-update"
                });
                probe.sweep = args.iter().any(|v| v == "--sweep-check");
                probe.motion = args.iter().any(|v| v == "--motion-check");
                probe.segment = args.iter().any(|v| v == "--segment-check");
                probe
            }),
        report: value("--report").map(PathBuf::from),
        ..Default::default()
    };
    if let Err(error) = EventLoop::new().unwrap().run_app(&mut app) {
        app.error = Some(format!("native event loop failed: {error}"));
    }
    // Every event-loop return (limit, close, initialization failure, or oracle) shares this gate.
    app.finalize_checks();
    if let Some(path) = &app.report {
        if let Some(h) = app.host.as_mut() {
            let mut times: Vec<_> = h.frame_us.iter().skip(1).copied().collect();
            times.sort();
            let quantile = |p: usize| {
                times
                    .get(times.len().saturating_sub(1) * p / 100)
                    .copied()
                    .unwrap_or(0)
            };
            let two = h.rt.capture(h.two, "native-2d").map(|v| v.0);
            let three = h.rt.capture(h.three, "native-3d").map(|v| v.0);
            if let Some((_, png, _, _)) = h.rt.capture(h.top, "native-top") {
                std::fs::write(path.with_extension("png"), png).expect("capture");
            }
            let mut data = serde_json::Map::new();
            data.insert(
                "schema".into(),
                serde_json::json!("autographics-authoring-report/1"),
            );
            data.insert(
                "spatial_check".into(),
                serde_json::json!(app.spatial_probe.is_some()),
            );
            data.insert(
                "spatial_phase".into(),
                serde_json::json!(app.spatial_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "spatial_records".into(),
                serde_json::json!(app.spatial_probe.as_ref().map(|p| &p.records)),
            );
            data.insert("play_runtime".into(), serde_json::json!(h.play.as_ref().map(|p|serde_json::json!({"state":format!("{:?}",p.state),"error":p.error,"trace":p.reports,"frame":p.last.as_ref().map(|f|&f.document)}))));
            data.insert(
                "character_check".into(),
                serde_json::json!(app.character_probe.is_some()),
            );
            data.insert(
                "character_phase".into(),
                serde_json::json!(app.character_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "character_records".into(),
                serde_json::json!(app.character_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "character3d_check".into(),
                serde_json::json!(app.character3d_probe.is_some()),
            );
            data.insert(
                "character3d_phase".into(),
                serde_json::json!(app.character3d_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "character3d_records".into(),
                serde_json::json!(app.character3d_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "runtime_camera_check".into(),
                serde_json::json!(app.runtime_camera_probe.is_some()),
            );
            data.insert(
                "runtime_camera_phase".into(),
                serde_json::json!(app.runtime_camera_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "runtime_camera_records".into(),
                serde_json::json!(app.runtime_camera_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "play_check".into(),
                serde_json::json!(app.play_probe.is_some()),
            );
            data.insert(
                "play_phase".into(),
                serde_json::json!(app.play_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "play_records".into(),
                serde_json::json!(app.play_probe.as_ref().map(|p| &p.records)),
            );
            data.insert("frames".into(), serde_json::json!(app.frames));
            data.insert(
                "interaction_check".into(),
                serde_json::json!(app.native_probe.is_some()),
            );
            data.insert(
                "interaction_phase".into(),
                serde_json::json!(app.native_probe.as_ref().map(|p| p.phase())),
            );
            data.insert(
                "interaction_records".into(),
                serde_json::json!(app.native_probe.as_ref().map(|p| &p.records)),
            );
            data.insert("inputs".into(), serde_json::json!(app.native_inputs));
            data.insert("probe".into(), serde_json::json!(app.probe));
            data.insert("phase".into(), serde_json::json!(app.phase));
            data.insert("passed".into(), serde_json::json!(app.passed));
            data.insert("error".into(), serde_json::json!(app.error));
            data.insert("document".into(), serde_json::json!(h.editor.document));
            data.insert("two".into(), serde_json::json!(two));
            data.insert("three".into(), serde_json::json!(three));
            data.insert(
                "cold_frame_us".into(),
                serde_json::json!(h.frame_us.first()),
            );
            data.insert("warm_p50_us".into(), serde_json::json!(quantile(50)));
            data.insert("warm_p95_us".into(), serde_json::json!(quantile(95)));
            data.insert("p50_us".into(), serde_json::json!(quantile(50)));
            data.insert("p95_us".into(), serde_json::json!(quantile(95)));
            data.insert(
                "compile_us".into(),
                serde_json::json!(h.editor.preview.compile_us),
            );
            data.insert("accepted".into(), serde_json::json!(h.accepted));
            data.insert("resources".into(), serde_json::json!(h.resource_snapshot()));
            data.insert(
                "window_scale".into(),
                serde_json::json!(app.window.as_ref().map(|w| w.scale_factor())),
            );
            data.insert(
                "window_px".into(),
                serde_json::json!(app.window.as_ref().map(|w| {
                    let s = w.inner_size();
                    [s.width, s.height]
                })),
            );
            data.insert("working_set_bytes".into(), serde_json::json!(working_set()));
            data.insert(
                "target_bytes".into(),
                serde_json::json!(h.rt.owned_target_bytes()),
            );
            data.insert(
                "uploads".into(),
                serde_json::json!(format!("{:?}", h.rt.upload_stats())),
            );
            data.insert(
                "asset_check".into(),
                serde_json::json!(app.asset_probe.is_some()),
            );
            data.insert(
                "asset_phase".into(),
                serde_json::json!(app.asset_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "asset_records".into(),
                serde_json::json!(app.asset_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "hierarchy_check".into(),
                serde_json::json!(app.hierarchy_probe.is_some()),
            );
            data.insert(
                "hierarchy_phase".into(),
                serde_json::json!(app.hierarchy_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "hierarchy_records".into(),
                serde_json::json!(app.hierarchy_probe.as_ref().map(|p| &p.records)),
            );
            let mut data = data;
            data.insert(
                "viewport3d_check".into(),
                serde_json::json!(app.viewport3d_probe.is_some()),
            );
            data.insert(
                "viewport3d_phase".into(),
                serde_json::json!(app.viewport3d_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "viewport3d_records".into(),
                serde_json::json!(app.viewport3d_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "viewport2d_check".into(),
                serde_json::json!(app.viewport2d_probe.is_some()),
            );
            data.insert(
                "viewport2d_phase".into(),
                serde_json::json!(app.viewport2d_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "viewport2d_records".into(),
                serde_json::json!(app.viewport2d_probe.as_ref().map(|p| &p.records)),
            );
            let runner = std::env::current_exe().expect("runner path");
            data.insert(
                "runner_sha256".into(),
                serde_json::json!(agp::renderer::capture::sha256_hex(
                    &std::fs::read(&runner).expect("runner bytes")
                )),
            );
            data.insert("runner_path".into(), serde_json::json!(runner));
            std::fs::write(path, serde_json::to_vec_pretty(&data).unwrap()).expect("report");
        } else {
            let mut data = serde_json::Map::new();
            data.insert(
                "schema".into(),
                serde_json::json!("autographics-authoring-report/1"),
            );
            data.insert(
                "spatial_check".into(),
                serde_json::json!(app.spatial_probe.is_some()),
            );
            data.insert(
                "spatial_phase".into(),
                serde_json::json!(app.spatial_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "spatial_records".into(),
                serde_json::json!(app.spatial_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "character_check".into(),
                serde_json::json!(app.character_probe.is_some()),
            );
            data.insert(
                "character_phase".into(),
                serde_json::json!(app.character_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "character_records".into(),
                serde_json::json!(app.character_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "character3d_check".into(),
                serde_json::json!(app.character3d_probe.is_some()),
            );
            data.insert(
                "character3d_phase".into(),
                serde_json::json!(app.character3d_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "character3d_records".into(),
                serde_json::json!(app.character3d_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "runtime_camera_check".into(),
                serde_json::json!(app.runtime_camera_probe.is_some()),
            );
            data.insert(
                "runtime_camera_phase".into(),
                serde_json::json!(app.runtime_camera_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "runtime_camera_records".into(),
                serde_json::json!(app.runtime_camera_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "play_check".into(),
                serde_json::json!(app.play_probe.is_some()),
            );
            data.insert(
                "play_phase".into(),
                serde_json::json!(app.play_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "play_records".into(),
                serde_json::json!(app.play_probe.as_ref().map(|p| &p.records)),
            );
            data.insert("frames".into(), serde_json::json!(app.frames));
            data.insert("probe".into(), serde_json::json!(app.probe));
            data.insert(
                "interaction_check".into(),
                serde_json::json!(app.native_probe.is_some()),
            );
            data.insert("passed".into(), serde_json::json!(app.passed));
            data.insert("error".into(), serde_json::json!(app.error));
            data.insert(
                "interaction_records".into(),
                serde_json::json!(app.native_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "asset_check".into(),
                serde_json::json!(app.asset_probe.is_some()),
            );
            data.insert(
                "asset_phase".into(),
                serde_json::json!(app.asset_probe.as_ref().map(|p| p.phase)),
            );
            data.insert(
                "asset_records".into(),
                serde_json::json!(app.asset_probe.as_ref().map(|p| &p.records)),
            );
            data.insert(
                "hierarchy_check".into(),
                serde_json::json!(app.hierarchy_probe.is_some()),
            );
            data.insert(
                "hierarchy_phase".into(),
                serde_json::json!(app.hierarchy_probe.as_ref().map(|p| p.phase)),
            );
            let mut data = data;
            let runner = std::env::current_exe().expect("runner path");
            data.insert(
                "runner_sha256".into(),
                serde_json::json!(agp::renderer::capture::sha256_hex(
                    &std::fs::read(&runner).expect("runner bytes")
                )),
            );
            data.insert("runner_path".into(), serde_json::json!(runner));
            std::fs::write(path, serde_json::to_vec_pretty(&data).unwrap()).expect("report");
        }
    }
    if let Some(error) = app.error {
        eprintln!("{error}");
        std::process::exit(1)
    }
}
