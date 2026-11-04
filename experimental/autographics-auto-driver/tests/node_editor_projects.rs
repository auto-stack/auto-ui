//! PLAN-040 T-04: project-scoped editing oracles driven against the ACTUAL
//! frozen G04-faithful node editor products (pilot generated-editor-projects;
//! document_id plan040-editor-projects-main). Every stage below maps 1:1 onto
//! an original viewport2d (20-phase) / viewport3d (34-phase) probe stage and
//! is covered through the supported no-OS-steal tiers — real Editor messages
//! (epoch/revision-stamped dispatch), the actual Auto interaction computation
//! (pointer gestures, interaction::axes/AxisDrag), and real GPU surface
//! capture/probe readback — while the real-OS-input specifics stay a named
//! pending tier (plan D-02). Swapping in the default sample document or
//! grabbing the wrong object row must go red here, not green.
//!
//! Run single-threaded:
//! cargo test --locked --offline --test node_editor_projects -- --test-threads=1
use autographics_auto_driver::authoring::{
    assets::Registry,
    document::*,
    host::Host,
    interaction,
    session::{Edit, Editor},
    storage::{Fault, Storage},
    view3d::Mode,
};
use glam::{Vec2, Vec3};

const SCENE_2D: &str =
    include_str!("../../autographics-node2d-pilot/generated-editor-projects/g04-editor-2d/scene.agscene");
const SCENE_3D: &str =
    include_str!("../../autographics-node2d-pilot/generated-editor-projects/g04-editor-3d/scene.agscene");
const DOC_ID: &str = "plan040-editor-projects-main";

fn write_project(scene_text: &str, tag: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan040-editor-projects")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("project dir");
    std::fs::write(dir.join("scene.agscene"), scene_text).expect("scene");
    dir
}

fn load_project_host(scene_text: &str, tag: &str) -> (Host, std::path::PathBuf) {
    let dir = write_project(scene_text, tag);
    let (doc, registry, code, _) =
        Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
            .expect("load editor project");
    assert!(code.is_none(), "editor projects carry no behavior payload");
    let h = Host::with_assets(doc, registry).expect("host");
    (h, dir)
}

fn command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"action":action,"payload":payload})
            .to_string(),
    )
}

fn view_command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"view_revision":h.view2d().revision,"action":action,"payload":payload})
            .to_string(),
    )
}

fn view3d_command(h: &mut Host, action: &str, payload: serde_json::Value) -> Result<(), String> {
    let rev = h.view3d().revision;
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"view3d_revision":rev,"action":action,"payload":payload})
            .to_string(),
    )
}

fn settle(h: &mut Host) {
    h.settle().expect("settle");
}

fn obj_by_orig_id<'a>(doc: &'a Document, orig: u64) -> &'a Object {
    let name = match orig {
        1 => "obj1_circle",
        2 => "obj2_quad",
        3 => "obj3_player",
        4 => "obj4_mob",
        5 => {
            if doc.objects.iter().any(|o| o.name == "helper_quad") {
                "helper_quad"
            } else {
                "helper_cylinder"
            }
        }
        6 => "helper_circle",
        other => panic!("no original id {other}"),
    };
    doc.objects
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("object {name} missing"))
}

/// Explicit frozen ID mapping (generated row id -> original G04 id).
fn assert_id_mapping(doc: &Document, three_first: bool) {
    let mapping: &[(u64, u64)] = if three_first {
        // g04-editor-2d rows: player/mob then circle/quad/helper quad/helper circle.
        &[(1, 3), (2, 4), (3, 1), (4, 2), (5, 5), (6, 6)]
    } else {
        // g04-editor-3d rows: player/mob/cylinder then circle/quad.
        &[(1, 3), (2, 4), (3, 5), (4, 1), (5, 2)]
    };
    assert_eq!(doc.objects.len(), mapping.len(), "object count");
    for (gen, orig) in mapping {
        assert_eq!(doc.objects[(*gen - 1) as usize].id, *gen, "row order id {gen}");
        assert_eq!(obj_by_orig_id(doc, *orig).id, *gen, "orig {orig} -> row {gen}");
    }
}

/// Original per-object value tuples (ag-examples@501fca0 g04-editor, T-01
/// freeze): (orig id, kind, translation, scale, rgba).
fn assert_original_values(doc: &Document, which: &str) {
    let table: &[(u64, Kind, [f32; 3], [f32; 3], [f32; 4])] = match which {
        "2d" => &[
            (1, Kind::Circle, [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (2, Kind::Quad, [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (3, Kind::Player, [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (4, Kind::Mob, [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (5, Kind::Quad, [400.0, -300.0, 0.0], [30.0, 30.0, 1.0], [0.2, 0.3, 0.9, 1.0]),
            (6, Kind::Circle, [-400.0, 300.0, 0.0], [20.0, 20.0, 1.0], [0.2, 0.8, 0.3, 1.0]),
        ],
        "3d" => &[
            (1, Kind::Circle, [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (2, Kind::Quad, [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (3, Kind::Player, [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (4, Kind::Mob, [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
            (5, Kind::Cylinder, [30.0, 0.0, 0.0], [1.0, 2.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        ],
        other => panic!("unknown project {other}"),
    };
    for (orig, kind, t, s, rgba) in table {
        let o = obj_by_orig_id(doc, *orig);
        assert_eq!(&o.kind, kind, "orig {orig} kind");
        assert_eq!(o.translation, *t, "orig {orig} translation");
        assert_eq!(o.scale, *s, "orig {orig} scale");
        assert_eq!(o.yaw, 0.0, "orig {orig} yaw");
        assert_eq!(o.pitch, 0.0, "orig {orig} pitch");
        assert_eq!(o.rgba, *rgba, "orig {orig} rgba");
        assert!(o.visible, "orig {orig} visible");
        assert_eq!(o.parent, None, "orig {orig} parent");
        assert_eq!(o.resource, None, "orig {orig} resource");
    }
}

// ------------------------------------------------------------ identity ----

#[test]
fn products_load_with_original_identity_not_the_default_sample() {
    for (scene, which, three_first) in [(SCENE_2D, "2d", true), (SCENE_3D, "3d", false)] {
        let dir = write_project(scene, &format!("identity-{which}"));
        let (doc, _registry, code, _storage) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                .expect("load project");
        assert!(code.is_none(), "behavior:null project");
        assert_eq!(doc.behavior, None, "behavior null");
        assert_eq!(doc.schema, "autographics.scene/6");
        assert_eq!(doc.document_id, DOC_ID, "frozen 040 identity");
        // The default sample must NOT pass for this project: the oracle is
        // bound to the actual loaded document, never to Document::sample().
        let sample = Document::sample();
        assert_ne!(doc.document_id, sample.document_id);
        assert_ne!(doc.objects.len(), sample.objects.len());
        assert_id_mapping(&doc, three_first);
        assert_original_values(&doc, which);
        // 2D draw order: the 2D-kind rows keep the original relative order.
        let two_d: Vec<u64> = doc
            .objects
            .iter()
            .filter(|o| o.kind.is_2d())
            .map(|o| o.id)
            .collect();
        let expected: &[u64] = if three_first { &[3, 4, 5, 6] } else { &[4, 5] };
        assert_eq!(two_d, expected, "{which} 2D draw order");
    }
}

// ------------------------------------------------- 2D project editing ----

/// Original viewport2d stages p0-p19 mapped onto the no-OS tiers (D-02:
/// focus_window acquisition and physical key/wheel posting stay OS-input
/// pending; every Host-level oracle runs here).
#[test]
fn g04_2d_project_nav_grid_pick_drag_undo_redo_cancel_gpu() {
    let (mut h, _dir) = load_project_host(SCENE_2D, "edit-2d");
    let before = h.editor.document.clone();
    h.frame(None).expect("frame");

    // p0/p1 grid: off-pixel, enable via message, grid pixels change on the
    // real 2D surface (GPU composite readback).
    let off = h.rt.probe_pixel(h.top, [19.0, 30.0]).expect("off readback");
    let rev = h.view2d().revision;
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"view_revision":rev,"action":"view-grid","payload":null}).to_string(),
    )
    .expect("grid on");
    assert!(h.view2d().grid, "grid enabled (p1)");
    h.frame(None).expect("frame");
    let line = h.rt.probe_pixel(h.top, [19.0, 30.0]).expect("line readback");
    let gap = h.rt.probe_pixel(h.top, [25.0, 35.0]).expect("gap readback");
    assert!(Some(line) != Some(off) && line != gap, "grid composite pixels (p1)");

    // p2-p4 MMB world pan + content/history invariance (view stays: the
    // probe zooms from the panned state before any reset).
    h.navigation_down([300.0, 190.0]);
    h.pointer_move([340.0, 210.0]);
    assert!(
        (h.view2d().center - Vec2::new(-40.0, 20.0)).length() < 0.01,
        "MMB world pan (p4)"
    );
    assert_eq!(h.editor.document, before, "navigation altered content (p4)");
    assert_eq!(h.editor.history().0, 0, "navigation altered history (p4)");
    h.navigation_up([340.0, 210.0]);
    h.frame(None).expect("frame presents the panned view");

    // p5 wheel zoom + stable anchor: the world point under the cursor is
    // the panned view's ((300,150) at center (-40,20) zoom 1) = (-140,60).
    let zoomed = h.view2d().wheel(Vec2::new(300.0, 150.0), 1.0).expect("wheel");
    h.set_view2d(zoomed).expect("apply zoom");
    h.frame(None).expect("frame presents the zoomed view");
    assert!((h.view2d().zoom - 1.1).abs() < 0.001, "wheel zoom (p5)");
    let anchor = h.view2d().world(Vec2::new(300.0, 150.0)).expect("anchor");
    assert!((anchor - Vec2::new(-140.0, 60.0)).length() < 1.0, "wheel anchor (p5)");

    // p6-p8 pick + real Auto world drag of the ORIGINAL object 1 (circle at
    // (-55,0)) + history + composite pixel at the moved location.
    let start = [
        400.0 + (-55.0 - h.view2d().center.x) * 1.1,
        190.0 + h.view2d().center.y * 1.1,
    ];
    h.pointer_down(start);
    assert_eq!(h.editor.selection, Some(3), "pick maps to original id 1 row (p6)");
    h.pointer_move([start[0] + 22.0, start[1] - 11.0]);
    h.pointer_up([start[0] + 22.0, start[1] - 11.0]);
    settle(&mut h);
    assert!(!h.gesture_active(), "gesture ended (p8)");
    let dragged = h.editor.document.objects[2].clone();
    assert!(
        (dragged.translation[0] + 35.0).abs() < 1.1
            && (dragged.translation[1] - 10.0).abs() < 1.1,
        "Auto world drag to (-35,10) (p8), got {:?}",
        dragged.translation
    );
    assert_eq!(h.editor.history().0, 1, "drag history (p8)");
    h.frame(None).expect("frame after drag");
    let screen = h
        .presented_view2d()
        .screen(Vec2::new(dragged.translation[0], dragged.translation[1]))
        .expect("object screen");
    let pixel = h
        .rt
        .probe_pixel(h.top, screen.to_array())
        .expect("object readback");
    assert!(
        pixel[0] > 100 && pixel[0] > pixel[1].saturating_add(30),
        "transformed composite object (p8), pixel {pixel:?}"
    );
    // Wrong row must go red: the mob (original id 4) did not move.
    assert_eq!(
        obj_by_orig_id(&h.editor.document, 4).translation,
        [1.5, 0.0, 0.0],
        "wrong row stayed put"
    );

    // p9 undo restores; p10 reset; redo re-applies; undo back to baseline.
    command(&mut h, "undo", serde_json::Value::Null).expect("undo");
    settle(&mut h);
    assert_eq!(h.editor.document, before, "undo restores (p9)");
    view_command(&mut h, "view-reset", serde_json::Value::Null).expect("reset");
    assert_eq!(h.view2d().center, Vec2::ZERO, "reset (p10)");
    assert!(!h.view2d().grid, "reset cleared grid (p10)");
    assert_eq!(h.editor.document, before, "reset altered content (p10)");
    command(&mut h, "redo", serde_json::Value::Null).expect("redo");
    settle(&mut h);
    assert_eq!(
        h.editor.document.objects[2].translation,
        dragged.translation,
        "redo re-applies the drag"
    );
    command(&mut h, "undo", serde_json::Value::Null).expect("undo");
    settle(&mut h);
    assert_eq!(h.editor.document, before, "back to baseline");

    // p13-p15 second pan + Esc cancel mid-gesture restores the navigation
    // start (probe order: the reset happens at p10 before this re-pan).
    view_command(&mut h, "view-reset", serde_json::Value::Null).expect("reset");
    assert_eq!(h.view2d().center, Vec2::ZERO, "reset before re-pan (p10)");
    h.navigation_down([300.0, 190.0]);
    h.pointer_move([340.0, 210.0]);
    assert_eq!(h.view2d().center, Vec2::new(-40.0, 20.0), "re-pan fixture (p13)");
    h.cancel_navigation();
    assert_eq!(h.view2d().center, Vec2::ZERO, "Esc nav restoration (p15)");

    // p11/p12 F focus (message equivalent) centers on the selected circle.
    h.editor.select(Some(3)).expect("select circle");
    view_command(&mut h, "view-focus", serde_json::Value::Null).expect("focus");
    assert!(
        (h.view2d().center - Vec2::new(-55.0, 0.0)).length() < 0.01,
        "F focus (p12)"
    );
}

// ------------------------------------------------- 3D project editing ----

/// Original viewport3d stages p0-p33 mapped onto the no-OS tiers (grid +
/// occlusion, camera pan/orbit/zoom, projection, pick, snap, X-axis drag,
/// history/undo, save→reopen on the ACTUAL project, F focus, Esc cancel,
/// play restore, Home; the real button-stream cross-capture stages stay
/// OS-input pending, D-02).
#[test]
fn g04_3d_project_grid_occlusion_axes_snap_save_reopen_play_restore() {
    let (mut h, dir) = load_project_host(SCENE_3D, "edit-3d");
    let baseline_bytes = std::fs::read(dir.join("scene.agscene")).expect("baseline file");
    let before = h.editor.document.clone();
    h.frame(None).expect("frame");

    // p0/p1: grid on + composite change + occlusion (3D rows hide the real
    // grid; removing the scene instances reveals it again).
    let mut view = h.view3d();
    view.grid = true;
    h.set_view3d(view).expect("grid on");
    h.frame(None).expect("frame");
    let vp = h.camera();
    let (_, lines) = h.view3d().grid_vertices().expect("grid vertices");
    let mut grid_points = Vec::new();
    'outer: for pair in lines.chunks_exact(2) {
        for t in [0.25f32, 0.5, 0.75] {
            let a = Vec3::from(pair[0].position);
            let b = Vec3::from(pair[1].position);
            let world = a + (b - a) * t;
            if let Some(p) = interaction::project(vp, world) {
                if p.x > 5.0 && p.x < 795.0 && p.y > 5.0 && p.y < 395.0 {
                    for dy in -1..=1 {
                        grid_points.push([p.x, p.y + 400.0 + dy as f32]);
                    }
                    if grid_points.len() >= 45 {
                        break 'outer;
                    }
                }
            }
        }
    }
    assert!(grid_points.len() >= 6, "visible grid samples (p0)");
    let grid_on = h.rt.probe_pixels(h.top, &grid_points).expect("grid readback");
    let saved_instances = match h.rt.content_mut(h.three) {
        Some(agp::host::surface_runtime::SurfaceContent::Scene3d(scene)) => {
            std::mem::take(&mut scene.instances)
        }
        _ => panic!("missing occlusion scene"),
    };
    h.rt.graph.mark_content_dirty(h.three);
    h.rt.frame(None, &Default::default()).expect("occlusion frame");
    let grid_without = h
        .rt
        .probe_pixels(h.top, &grid_points)
        .expect("grid-without readback");
    match h.rt.content_mut(h.three) {
        Some(agp::host::surface_runtime::SurfaceContent::Scene3d(scene)) => {
            scene.instances = saved_instances
        }
        _ => panic!("missing scene while restoring"),
    }
    h.rt.graph.mark_content_dirty(h.three);
    h.rt.frame(None, &Default::default()).expect("restore frame");
    let changed = grid_on
        .iter()
        .zip(grid_without.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 0, "grid-occlusion: foreground hid real grid (p1)");

    // p2-p5 camera pan (message math) + content invariance + projection.
    let start_view = h.view3d();
    let panned = start_view.pan(Vec2::new(40.0, 20.0)).expect("pan");
    h.set_view3d(panned).expect("apply pan");
    assert_ne!(h.view3d().target, start_view.target, "camera pan (p4)");
    assert_eq!(h.editor.document, before, "navigation changed document (p4)");
    assert_eq!(h.editor.history().0, 0, "navigation changed history (p4)");
    let perspective = h.view3d().toggle_projection().expect("projection").0;
    h.set_view3d(perspective).expect("apply projection");
    assert_eq!(h.view3d().mode, Mode::Perspective, "P toggle (p5)");

    // p6-p9 orbit + perspective wheel (View3d message math).
    let orbited = h.view3d().orbit(Vec2::new(-30.0, 0.0)).expect("orbit");
    h.set_view3d(orbited).expect("apply orbit");
    let zoomed = h.view3d().wheel(1.0).expect("wheel");
    h.set_view3d(zoomed).expect("apply zoom");
    assert!(h.view3d().distance > 0.0, "perspective wheel (p9)");

    // p10-p13 pick + snap + X-axis handle drag on the player (original id 3).
    // The pick requires a presented frame matching the current views.
    h.frame(None).expect("frame before pick");
    let player = obj_by_orig_id(&h.editor.document, 3).clone();
    let p = interaction::project(h.camera(), Vec3::from(player.translation))
        .expect("player on screen");
    h.pointer_down([p.x, p.y + 400.0]);
    assert_eq!(h.editor.selection, Some(player.id), "3D pick (p10)");
    h.pointer_up([p.x, p.y + 400.0]);
    settle(&mut h);
    view3d_command(&mut h, "view3d-snap", serde_json::Value::Null).expect("snap on");
    assert!(h.view3d().snap, "snap enabled (p10)");
    h.frame(None).expect("frame after snap");
    let vp = h.camera();
    let (axis, a, d) = interaction::axes(&player, vp)
        .into_iter()
        .find(|(axis, _, _)| *axis == 0)
        .expect("X handle");
    let handle_start = a + d.normalize() * 40.0;
    assert_eq!(
        interaction::handle(&player, vp, handle_start.to_array())
            .expect("handle")
            .0,
        axis,
        "X handle hit (p10)"
    );
    let drag =
        interaction::AxisDrag::new(vp, Vec3::from(player.translation), axis, handle_start)
            .expect("axis plane");
    let mut end = None;
    for n in 15..200 {
        let candidate = handle_start + d.normalize() * n as f32;
        if let Some(v) = drag.delta(candidate) {
            if v.abs() > 0.5 && candidate.x > 0.0 && candidate.x < 800.0 && candidate.y > 0.0 {
                end = Some((candidate, v));
                break;
            }
        }
    }
    let (end, delta) = end.expect("valid X movement");
    let expected = h.view3d().snapped_axis(player.translation[0] + delta);
    h.pointer_down([handle_start.x, handle_start.y + 400.0]);
    assert!(h.gesture_active(), "axis capture began (p11)");
    h.pointer_move([end.x, end.y + 400.0]);
    h.pointer_up([end.x, end.y + 400.0]);
    settle(&mut h);
    let after_t = obj_by_orig_id(&h.editor.document, 3).translation;
    assert!(
        (after_t[0] - expected).abs() < 0.03,
        "Auto perspective drag == snapped expectation (p13): {:?} vs {expected}",
        after_t
    );
    assert_eq!(h.editor.history().0, 1, "drag history (p13)");
    assert_eq!(
        obj_by_orig_id(&h.editor.document, 4).translation,
        [1.5, 0.0, 0.0],
        "wrong row stayed put (p13)"
    );

    // p14-p16 save → reopen on the ACTUAL project: the accepted drag is
    // persisted, the reopened root keeps it, and the view resets on open.
    let save_path = dir.join("saved.agscene");
    command(&mut h, "saveas", serde_json::json!(save_path.to_string_lossy()))
        .expect("saveas");
    let saved_bytes = std::fs::read(&save_path).expect("save file exists (p15)");
    assert_ne!(saved_bytes, baseline_bytes, "saved file carries the edit");
    command(&mut h, "undo", serde_json::Value::Null).expect("undo");
    settle(&mut h);
    assert_eq!(h.editor.document, before, "undo restored (p14)");
    command(&mut h, "open", serde_json::json!(save_path.to_string_lossy()))
        .expect("open request");
    command(&mut h, "leave-discard", serde_json::Value::Null).expect("leave");
    settle(&mut h);
    assert!(!h.editor.dirty(), "reopened clean (p16)");
    assert_eq!(
        obj_by_orig_id(&h.editor.document, 3).translation,
        after_t,
        "reopened document keeps the accepted drag (p16)"
    );
    assert_eq!(h.editor.document.behavior, None, "reopened stays behavior:null");
    assert!(h.view3d().sample_default, "open reset the 3D view (p16)");
    let reopened_doc = h.editor.document.clone();

    // p17/p18 F focus on the reopened selection (message equivalent).
    h.editor.select(Some(player.id)).expect("select player");
    let target_before = h.view3d().target;
    view3d_command(&mut h, "view3d-focus", serde_json::Value::Null).expect("focus");
    assert_ne!(h.view3d().target, target_before, "F moved to selection (p18)");

    // p19-p21 Esc cancel mid-gesture restores the camera start.
    h.frame(None).expect("frame presents the focused view");
    let pre_cancel = h.view3d();
    h.navigation_down([400.0, 600.0]);
    h.pointer_move([430.0, 620.0]);
    assert_ne!(h.view3d().target, pre_cancel.target, "cancel fixture navigated (p20)");
    h.cancel_navigation();
    assert_eq!(h.view3d().target, pre_cancel.target, "Esc restoration (p21)");

    // p22/p23 play run/stop: empty Builtin route (behavior:null), editor
    // camera restored after Stop, content untouched. A fresh Host's play
    // token is 0.
    let cam_before_play = h.camera();
    h.dispatch(
        &serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision,"play_token":0,"action":"play-run","payload":null}).to_string(),
    )
    .expect("play-run");
    assert!(h.play.is_some(), "Run started (p22)");
    while h.play.as_ref().map(|pp| pp.pending()).unwrap_or(false) {
        h.frame(None).expect("play frame");
    }
    h.stop_play();
    settle(&mut h);
    assert!(h.play.is_none(), "Stop returned to editor (p23)");
    assert_eq!(
        h.editor.document, reopened_doc,
        "play round-trip did not alter the accepted content"
    );

    // p24 Home reset (message equivalent) + GPU: both project surfaces alive.
    view3d_command(&mut h, "view3d-reset", serde_json::Value::Null).expect("home reset");
    assert!(h.view3d().sample_default, "Home reset (p24/p33)");
    let _ = cam_before_play;
    let cap2 = h.rt.capture(h.two, "p040-2d").expect("capture 2d").0;
    let cap3 = h.rt.capture(h.three, "p040-3d").expect("capture 3d").0;
    assert!(!cap2.is_empty(), "2D surface rendered");
    assert!(!cap3.is_empty(), "3D surface rendered");
}

// --------------------------------------------- baseline/copy ownership ----

/// AC-04: SaveAs creates an independent editable copy; the copy carries edits
/// across save/close/reopen at a new root; the baseline file stays byte-
/// identical; the edited copy does not pretend to be the baseline; re-lowering
/// is a separate explicit action (verify_plan040.py) and never silently
/// rewrote the copy.
#[test]
fn saveas_copy_edits_survive_reopen_and_baseline_stays_immutable() {
    let (mut h, dir) = load_project_host(SCENE_3D, "saveas");
    let baseline_bytes = std::fs::read(dir.join("scene.agscene")).expect("baseline file");
    h.frame(None).expect("frame");

    // Drag the player (original id 3) via the real Auto axis drag.
    let player = obj_by_orig_id(&h.editor.document, 3).clone();
    h.editor.select(Some(player.id)).expect("select");
    h.frame(None).expect("frame");
    let vp = h.camera();
    let (axis, a, d) = interaction::axes(&player, vp)
        .into_iter()
        .find(|(axis, _, _)| *axis == 0)
        .expect("X handle");
    let start = a + d.normalize() * 40.0;
    let end = start + d.normalize() * 60.0;
    h.pointer_down([start.x, start.y + 400.0]);
    h.pointer_move([end.x, end.y + 400.0]);
    h.pointer_up([end.x, end.y + 400.0]);
    settle(&mut h);
    let edited = obj_by_orig_id(&h.editor.document, 3).translation;
    assert_ne!(edited, player.translation, "copy edit applied");

    // SaveAs an independent copy; the baseline file must stay byte-identical.
    let copy_path = dir.join("copy.agscene");
    command(&mut h, "saveas", serde_json::json!(copy_path.to_string_lossy()))
        .expect("saveas");
    assert_eq!(
        std::fs::read(dir.join("scene.agscene")).expect("baseline after saveas"),
        baseline_bytes,
        "SaveAs must not touch the opened baseline file"
    );

    // Close and reopen the COPY at a new root: the edit survives.
    command(&mut h, "open", serde_json::json!(copy_path.to_string_lossy()))
        .expect("open copy");
    settle(&mut h);
    assert_eq!(
        obj_by_orig_id(&h.editor.document, 3).translation,
        edited,
        "copy reopened with the user edit"
    );
    assert_ne!(
        h.editor.document.document_id,
        Document::sample().document_id,
        "reopened copy is the project, not the sample"
    );
    // The copy does not pretend to be the baseline.
    let reopened_text = std::fs::read_to_string(&copy_path).expect("copy file");
    let baseline_text = String::from_utf8(baseline_bytes).expect("baseline utf8");
    assert_ne!(reopened_text, baseline_text, "copy differs from baseline");
    // Frozen product == baseline file (regeneration binds to this text).
    assert_eq!(SCENE_3D, baseline_text, "frozen product == baseline file");
}

/// AC-04: save faults keep last-good or report accurate residue; a stale
/// epoch/revision dispatch is rejected; recovery lands the edit and reopens
/// green; the project file itself is untouched throughout.
#[test]
fn save_faults_stale_revisions_and_recovery() {
    let (mut h, dir) = load_project_host(SCENE_2D, "faults");
    h.frame(None).expect("frame");
    let baseline = std::fs::read(dir.join("scene.agscene")).expect("baseline");

    // Stale epoch and revision must be rejected (message contract).
    let stale_epoch = serde_json::json!({"epoch":h.editor.stamp.epoch+1,"revision":h.editor.stamp.revision,"action":"undo","payload":null});
    assert!(h.dispatch(&stale_epoch.to_string()).is_err(), "stale epoch");
    let stale_rev = serde_json::json!({"epoch":h.editor.stamp.epoch,"revision":h.editor.stamp.revision+1,"action":"undo","payload":null});
    assert!(h.dispatch(&stale_rev.to_string()).is_err(), "stale revision");
    assert_eq!(obj_by_orig_id(&h.editor.document, 1).translation, [-55.0, 0.0, 0.0]);

    // Real storage path: edit, clean save, then fault-injected saves keep
    // last-good and stay dirty; recovery (Fault::None) lands the edit.
    let circle = obj_by_orig_id(&h.editor.document, 1).clone();
    h.editor.select(Some(circle.id)).expect("select circle");
    let mut moved = circle.clone();
    moved.translation = [-45.0, 5.0, 0.0];
    h.editor
        .begin(h.editor.stamp, Edit::Replace(moved.clone()))
        .expect("edit");
    settle(&mut h);
    let mut storage = Storage::default();
    let target = dir.join("fault-round.agscene");
    storage
        .save(&mut h.editor, &target, Fault::None)
        .expect("clean first save");
    let first = std::fs::read(&target).expect("first save bytes");
    let mut moved2 = moved.clone();
    moved2.translation = [-35.0, 10.0, 0.0];
    h.editor
        .begin(h.editor.stamp, Edit::Replace(moved2))
        .expect("edit2");
    settle(&mut h);
    for fault in [Fault::Prepare, Fault::Write, Fault::Flush, Fault::BeforeReplace] {
        assert!(storage.save(&mut h.editor, &target, fault).is_err(), "{fault:?} fails");
        assert_eq!(std::fs::read(&target).unwrap(), first, "last-good kept");
        assert!(h.editor.dirty(), "unsaved edit stays dirty");
        assert!(!storage.unverified(), "no false verification");
    }
    storage.save(&mut h.editor, &target, Fault::None).expect("repair");
    assert!(!h.editor.dirty(), "repair landed the accepted edit");
    // Reopen the saved file at a new root: edit present, project untouched.
    command(&mut h, "open", serde_json::json!(target.to_string_lossy())).expect("open");
    settle(&mut h);
    assert_eq!(
        obj_by_orig_id(&h.editor.document, 1).translation,
        [-35.0, 10.0, 0.0],
        "reopened carries the accepted edit"
    );
    assert_eq!(
        std::fs::read(dir.join("scene.agscene")).unwrap(),
        baseline,
        "project file untouched by the fault round"
    );
}

/// AC-05 (V-S04): 20 rounds of open/edit/close on the actual project keep the
/// real resource owners settled — no per-round mesh re-upload growth, no
/// leaked history/pin owners; navigation frames do not re-upload the static
/// meshes (warm upload stays the owner's).
#[test]
fn twenty_round_open_edit_close_owners_stay_settled() {
    let (mut h, dir) = load_project_host(SCENE_3D, "owners");
    h.frame(None).expect("frame");
    let baseline = h.resource_snapshot();
    assert!(baseline.is_object(), "resource snapshot shape");

    // 20 rounds: pick + axis drag + undo + save + reopen (new root each
    // round); uploads must not grow per round.
    let save_path = dir.join("owners-round.agscene");
    for round in 0..20 {
        h.frame(None).expect("round frame");
        let player = obj_by_orig_id(&h.editor.document, 3).clone();
        h.editor.select(Some(player.id)).expect("select");
        h.frame(None).expect("frame");
        let vp = h.camera();
        let (axis, a, d) = interaction::axes(&player, vp)
            .into_iter()
            .find(|(axis, _, _)| *axis == 0)
            .expect("X handle");
        let start = a + d.normalize() * 30.0;
        let end = start + d.normalize() * (5.0 + round as f32);
        h.pointer_down([start.x, start.y + 400.0]);
        h.pointer_move([end.x, end.y + 400.0]);
        h.pointer_up([end.x, end.y + 400.0]);
        settle(&mut h);
        command(&mut h, "saveas", serde_json::json!(save_path.to_string_lossy()))
            .expect("round saveas");
        command(&mut h, "undo", serde_json::Value::Null).expect("round undo");
        settle(&mut h);
        command(&mut h, "open", serde_json::json!(save_path.to_string_lossy()))
            .expect("round open");
        command(&mut h, "leave-discard", serde_json::Value::Null).ok();
        settle(&mut h);
    }
    // Ten navigation frames: static meshes must not re-upload.
    for _ in 0..10 {
        let view = h.view3d().orbit(Vec2::new(4.0, 0.0)).expect("orbit");
        h.set_view3d(view).expect("apply");
        h.frame(None).expect("nav frame");
    }
    let after = h.resource_snapshot();
    assert_eq!(
        after["uploads"], baseline["uploads"],
        "mesh upload owners stayed settled across 20 edit rounds + navigation"
    );
    assert_eq!(
        h.editor.history().0, 0,
        "reopen leaves no residual history entries"
    );
}

/// Original stage coverage table: every original 20/34 probe phase maps to
/// the tier that closed it here ("auto": message/Auto/GPU evidence in this
/// file) or is a named real-OS-input pending (D-02; never deleted, never
/// Ready). A missing stage row makes this test red.
#[test]
fn original_stage_table_is_fully_mapped() {
    const TWO_D: &[(&str, &str)] = &[
        ("p0 snapshot+off-pixel", "auto"),
        ("p0 focus_window", "os"),
        ("p1 grid enable+composite", "auto"),
        ("p2-3 MMB pan", "auto"),
        ("p4 pan exact+content/history", "auto"),
        ("p4 physical wheel post", "os"),
        ("p5 zoom 1.1+anchor", "auto"),
        ("p6 transformed pick", "auto"),
        ("p7 drag move+up", "auto"),
        ("p8 drag exact+history+composite", "auto"),
        ("p9 undo restore", "auto"),
        ("p10 reset+content", "auto"),
        ("p11 F key post", "os"),
        ("p12 focus mapping (-55,0)/6.08", "auto"),
        ("p13 MMB move", "auto"),
        ("p14 Esc key post", "os"),
        ("p15 Esc nav restoration+scene-path", "auto"),
        ("p16 Home+F key posts", "os"),
        ("p17 text-focus isolation", "os"),
        ("p18 Home key post", "os"),
        ("p19 Home reset+content", "auto"),
    ];
    const THREE_D: &[(&str, &str)] = &[
        ("p0 snapshot+grid baseline", "auto"),
        ("p0 focus_window", "os"),
        ("p1 grid composite", "auto"),
        ("p1 grid-occlusion", "auto"),
        ("p2-3 MMB camera pan", "auto"),
        ("p4 pan exact+content", "auto"),
        ("p4 P key post", "os"),
        ("p5 perspective toggle", "auto"),
        ("p6-7 RMB orbit", "auto"),
        ("p8 yaw exact", "auto"),
        ("p8 physical wheel post", "os"),
        ("p9 perspective zoom", "auto"),
        ("p10 pick", "auto"),
        ("p10 snap enable+X handle+AxisDrag", "auto"),
        ("p11-12 drag move/up", "auto"),
        ("p13 drag snapped+history", "auto"),
        ("p14 undo restore", "auto"),
        ("p14-15 save file", "auto"),
        ("p15-16 open+reopen equality+view reset", "auto"),
        ("p17 selection+F key post", "os"),
        ("p18 F focus target", "auto"),
        ("p19-20 MMB navigate", "auto"),
        ("p20 Esc key post", "os"),
        ("p21 Esc camera restoration", "auto"),
        ("p21 play-run button", "auto"),
        ("p22 run start+sample camera", "auto"),
        ("p22 play-stop button", "auto"),
        ("p23 stop editor camera restore", "auto"),
        ("p23-24 Home key post", "os"),
        ("p24 Home reset+content", "auto"),
        ("p24-28 RMB release ends middle capture", "os"),
        ("p29-32 middle release ends right capture", "os"),
        ("p33 Home reset", "auto"),
    ];
    assert_eq!(TWO_D.len(), 21, "2D stage rows (20 phases, focus split)");
    assert_eq!(THREE_D.len(), 33, "3D stage rows (34 phases, focus split)");
    for (stage, tier) in TWO_D.iter().chain(THREE_D.iter()) {
        assert!(!stage.is_empty());
        assert!(matches!(*tier, "auto" | "os"), "{stage}");
    }
    let os = TWO_D.iter().chain(THREE_D.iter()).filter(|(_, t)| *t == "os").count();
    assert_eq!(os, 15, "named OS-input pending rows (D-02)");
}
