//! Case `surface-lifecycle` (PLAN-830 T-08 / V-07-V-09).
//!
//! - V-07: 20 rounds of resize / view close+reopen / detach+reattach /
//!   rejected structural changes on a composed graph. Gates use REAL
//!   evidence: `OffscreenTarget::build_seq` and the runtime's
//!   `target_creations`/`target_drops` counters (not a loop-shaped guess),
//!   content that survives its consumer view closing, unrelated surfaces
//!   skipped, and a rejected change that leaves the last good graph intact.
//! - V-08: negative controls on the REAL composed path. Each fault is
//!   injected into the production path (scissor off, opaque portal blend,
//!   duplicate events, second render pass, composite-before-render, a second
//!   device, per-frame readback) and the corresponding gate MUST observe it.
//! - V-09: composition metrics (CPU frame p50/p95, target creations/drops,
//!   uploads, readbacks, dirty skips) + raw 20-round facts.

use crate::core::composition::{
    CompositionChange, CompositionError, MountKind, PortalId, Rect, SurfaceId, ViewId,
};
use crate::core::scene::Color;
use crate::host::cases::metrics::sample_rss_mb;
use crate::host::iced_surface::{SurfaceElement, SurfaceUi};
use crate::host::report::*;
use crate::host::surface_runtime::{Game2dContent, IcedContent, SurfaceRuntime};
use crate::renderer::pipeline::DrawCmd;
use crate::renderer::GpuDevice;
use iced::widget::{button, column, text, text_input};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

const ROUNDS: u32 = 20;
/// Even rounds resize BOTH the top and the UI surface to a DIFFERENT size.
const EXPLICIT_RESIZE_CREATIONS: u64 = (ROUNDS / 2) as u64 * 2;

#[derive(Default)]
pub struct LcUi {
    pub presses: u32,
}

impl SurfaceUi for LcUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![button(text(format!("lc {}", self.presses)))
            .width(iced::Length::Fill)
            .on_press(format!("{}", self.presses + 1)),]
        .spacing(6)
        .padding(8)
        .width(iced::Length::Fixed(200.0))
        .into()
    }
    fn update(&mut self, m: String) {
        if let Ok(n) = m.parse::<u32>() {
            self.presses = n;
        }
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.95, 0.95, 0.98)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Text input used by the duplicate-event control (each KeyPressed inserts a
/// character, so a duplicated event list is directly observable).
#[derive(Default)]
pub struct DupUi {
    pub text: String,
}

impl SurfaceUi for DupUi {
    type Message = String;
    fn view(&self) -> SurfaceElement<'_, String> {
        column![text_input("dup", &self.text)
            .id(iced_wgpu::core::widget::Id::new("dup-input"))
            .on_input(|s| s)
            .padding(8)]
        .padding(8)
        .width(iced::Length::Fixed(200.0))
        .into()
    }
    fn update(&mut self, m: String) {
        self.text = m;
    }
    fn background(&self) -> iced_wgpu::core::Color {
        iced_wgpu::core::Color::from_rgb(0.96, 0.96, 0.99)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct LifecycleReport {
    adapter: crate::renderer::AdapterInfoReport,
    device_token: String,
    rounds: u32,
    lifecycle: serde_json::Value,
    metrics: serde_json::Value,
    gates: Vec<GateResult>,
    all_gates_behaved: bool,
    keyframes: Vec<Keyframe>,
    failures: Vec<String>,
    gpu_timing_available: bool,
    notes: Vec<String>,
}

fn gate(name: &str, ok: bool, reason: String) -> GateResult {
    GateResult {
        gate: name.into(),
        positive_passed: ok,
        negative_fired: false,
        reason,
    }
}

fn frame(rt: &mut SurfaceRuntime) {
    let _ = rt.frame(None, &HashMap::new());
}

fn canvas(rt: &mut SurfaceRuntime, size: [f32; 2], color: Color) -> SurfaceId {
    rt.add_surface(
        size,
        crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(Game2dContent::simple(
            rt.gpu(),
            color,
        ))),
    )
    .expect("canvas surface")
}

fn flat(x: f32, y: f32, w: f32, h: f32) -> MountKind {
    MountKind::Flat2d {
        offset: [x, y],
        scale: [1.0, 1.0],
        clip: Rect::at(x, y, w, h),
    }
}

fn is_magenta(p: [u8; 4]) -> bool {
    p[0] > 200 && p[2] > 200 && p[1] < 80
}

fn is_gray(p: [u8; 4]) -> bool {
    p[0].abs_diff(188) <= 10 && p[1].abs_diff(188) <= 10 && p[2].abs_diff(188) <= 10
}

fn near(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

fn dup_text(rt: &SurfaceRuntime, id: SurfaceId) -> String {
    match rt.content(id) {
        Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) => {
            c.ui.as_any()
                .downcast_ref::<DupUi>()
                .map(|u| u.text.clone())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    if mode == "window" {
        eprintln!("surface-lifecycle: lifecycle/metrics gates run headless (deterministic)");
    }
    let gpu = match GpuDevice::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("surface-lifecycle: no GPU: {e}");
            return 3;
        }
    };
    let mut rt = match SurfaceRuntime::new(gpu) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("surface-lifecycle: runtime: {e}");
            return 3;
        }
    };
    let adapter = rt.gpu().adapter_info().clone();
    let token = rt.gpu().token().to_report();
    let mut gates: Vec<GateResult> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    // ---- graph: top 2D + UI child + canvas grandchild ---------------------
    let top = canvas(&mut rt, [640.0, 480.0], Color::rgb(0.12, 0.13, 0.16));
    let top_view = rt.add_view(top, true).expect("top view");
    let ui = rt
        .add_surface(
            [200.0, 120.0],
            crate::host::surface_runtime::SurfaceContent::Iced(Box::new(IcedContent::borrowed(
                rt.gpu(),
                [200.0, 120.0],
                Box::new(LcUi::default()),
            ))),
        )
        .expect("ui");
    let mut ui_view = rt.add_view(ui, false).expect("ui view");
    let mut ui_portal = rt
        .add_portal(top_view, ui, flat(20.0, 20.0, 200.0, 120.0), true)
        .expect("ui mount");
    let mut canvas_cmds = Game2dContent::simple(rt.gpu(), Color::rgb(0.9, 0.85, 0.2));
    canvas_cmds.commands = vec![DrawCmd {
        center: [50.0, 40.0],
        size: [30.0, 20.0],
        z: 0.0,
        color: Color::rgb(0.2, 0.9, 0.3),
        shape: crate::renderer::pipeline::Shape::Rect,
    }];
    let canvas_id = rt
        .add_surface(
            [120.0, 80.0],
            crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(canvas_cmds)),
        )
        .expect("canvas");
    rt.add_view(canvas_id, false).expect("canvas view");
    rt.add_portal(ui_view, canvas_id, flat(40.0, 60.0, 120.0, 60.0), true)
        .expect("canvas mount");

    // Warm: one frame builds pipelines/targets.
    frame(&mut rt);
    let surfaces_baseline = rt.graph.live_surfaces();
    let creations_baseline = rt.stats.target_creations;
    let drops_baseline = rt.stats.target_drops;
    let canvas_build_seq = rt.target(canvas_id).expect("canvas target").build_seq;
    let mut top_build_seq = rt.target(top).expect("top target").build_seq;
    let mut ui_build_seq = rt.target(ui).expect("ui target").build_seq;
    let mut top_rebuilds = 0u32;
    let mut ui_rebuilds = 0u32;

    // ---- 20 lifecycle rounds ---------------------------------------------
    let mut round_facts = Vec::new();
    let mut frames = 0u32;
    let mut content_survived_close = 0u32;
    for round in 0..ROUNDS {
        // (a) resize BOTH surfaces every other round; the sizes actually
        //     change, so each call is one REAL target rebuild.
        if round % 2 == 0 {
            let w = 640.0 + (round as f32 + 1.0) * 4.0;
            rt.resize_surface(top, [w, 480.0]).expect("resize top");
            let h = if (round / 2) % 2 == 0 { 110.0 } else { 120.0 };
            rt.resize_surface(ui, [200.0, h]).expect("resize ui");
            let new_top = rt.target(top).expect("top target").build_seq;
            let new_ui = rt.target(ui).expect("ui target").build_seq;
            if new_top != top_build_seq {
                top_rebuilds += 1;
            }
            if new_ui != ui_build_seq {
                ui_rebuilds += 1;
            }
            top_build_seq = new_top;
            ui_build_seq = new_ui;
        }
        // (b) close + reopen the UI's view (its portals cascade away; the
        //     child content and its GPU slot must survive).
        rt.graph
            .submit_one(CompositionChange::RemoveView(ui_view))
            .expect("close view");
        frame(&mut rt);
        frames += 1;
        let content_alive = rt.graph.surface(ui).is_ok()
            && rt.content(ui).is_some()
            && rt.slot(canvas_id).is_some()
            && rt.content(canvas_id).is_some();
        if content_alive {
            content_survived_close += 1;
        }
        ui_view = rt.add_view(ui, false).expect("reopen view");
        rt.add_portal(ui_view, canvas_id, flat(40.0, 60.0, 120.0, 60.0), true)
            .expect("reopen canvas mount");
        frame(&mut rt);
        frames += 1;

        // (c) hide + restore the UI mount (unmount must not destroy content).
        rt.remove_portal(ui_portal).expect("hide ui");
        frame(&mut rt);
        frames += 1;
        let hidden_alive = rt.content(ui).is_some() && rt.slot(ui).is_some();
        ui_portal = rt
            .add_portal(top_view, ui, flat(20.0, 20.0, 200.0, 120.0), true)
            .expect("restore ui");
        frame(&mut rt);
        frames += 1;

        // (d) a rejected structural change must keep the usable graph.
        let snapshot_before = rt.graph.snapshot();
        let bad = rt.graph.submit_one(CompositionChange::AddPortal {
            desc: crate::core::composition::PortalDesc {
                parent: ui_view,
                child: top, // top -> ui -> top: indirect cycle
                mount: flat(0.0, 0.0, 10.0, 10.0),
                interactive: true,
            },
        });
        let rejected = bad.is_err();
        let graph_intact = snapshot_before == rt.graph.snapshot();
        frame(&mut rt);
        frames += 1;
        round_facts.push(serde_json::json!({
            "round": round,
            "child_alive_while_unmounted": content_alive && hidden_alive,
            "cycle_rejected": rejected,
            "graph_intact_after_rejection": graph_intact,
            "surfaces": rt.graph.live_surfaces(),
            "portals": rt.graph.portals().len(),
            "target_creations": rt.stats.target_creations,
            "ui_build_seq": rt.target(ui).map(|t| t.build_seq),
            "canvas_build_seq": rt.target(canvas_id).map(|t| t.build_seq),
        }));
        if !(rejected && graph_intact) {
            failures.push(format!("round {round}: rejected change altered the graph"));
        }
    }
    let surfaces_end = rt.graph.live_surfaces();
    let creations_delta = rt.stats.target_creations - creations_baseline;
    let canvas_build_seq_end = rt.target(canvas_id).expect("canvas target").build_seq;

    // ---- explicit destroy/recreate of a slot ------------------------------
    let scratch = canvas(&mut rt, [32.0, 32.0], Color::rgb(0.0, 0.0, 1.0));
    frame(&mut rt);
    let drops_before_destroy = rt.stats.target_drops;
    rt.drop_surface(scratch).expect("drop scratch");
    let destroyed = rt.slot(scratch).is_none() && rt.stats.target_drops == drops_before_destroy + 1;
    let scratch2 = canvas(&mut rt, [24.0, 24.0], Color::rgb(0.0, 0.0, 1.0));
    frame(&mut rt);
    let recreated = rt.slot(scratch2).is_some() && rt.graph.surface(scratch2).is_ok();
    gates.push(gate(
        "slot_destroy_and_recreate",
        destroyed && recreated,
        format!(
            "destroyed={destroyed} recreated={recreated} drops={}",
            rt.stats.target_drops
        ),
    ));

    // ---- target accounting (V-07/V-09) -----------------------------------
    // A resize REPLACES the slot's target; the replaced target is really
    // dropped and counted, an identical resize is a true no-op, and a slot
    // removal drops exactly one. Review r2 F05 (the old code only counted
    // explicit slot removals).
    {
        let acct = canvas(&mut rt, [20.0, 20.0], Color::rgb(0.0, 0.0, 1.0));
        let c0 = rt.stats.target_creations;
        let d0 = rt.stats.target_drops;
        let resized = rt.resize_surface(acct, [30.0, 20.0]).is_ok();
        let created_one = rt.stats.target_creations == c0 + 1;
        let replaced_dropped = rt.stats.target_drops == d0 + 1;
        let _ = rt.resize_surface(acct, [30.0, 20.0]);
        let noop_stable = rt.stats.target_creations == c0 + 1 && rt.stats.target_drops == d0 + 1;
        let dropped = rt.drop_surface(acct).is_ok() && rt.stats.target_drops == d0 + 2;
        let balanced = rt.stats.live_targets() as usize == rt.live_target_slots()
            && rt.stats.live_targets() as usize == rt.graph.live_surfaces();
        gates.push(gate(
            "resize_replacement_is_counted_as_a_drop",
            resized && created_one && replaced_dropped && noop_stable && dropped && balanced,
            format!(
                "resize ok={resized} created+1={created_one} replaced-drop+1={replaced_dropped} \
                 noop-stable={noop_stable} final-drop={dropped} \
                 live_targets={} live_slots={} graph_surfaces={}",
                rt.stats.live_targets(),
                rt.live_target_slots(),
                rt.graph.live_surfaces()
            ),
        ));
    }

    // ---- metrics (V-09) --------------------------------------------------
    // Sampling points for every metric below: `*_start` immediately before
    // the 60-frame loop, `*_end` immediately after it. Upload counters come
    // from the REAL write paths (`SurfaceRuntime::upload_stats` merges the
    // composition pipelines and the content adapters); the process working
    // set is the OS residency of THIS process and is NOT owned GPU memory.
    let metric_frames = 60u32;
    let mut cpu_ms: Vec<f64> = Vec::with_capacity(metric_frames as usize);
    let stats_before = rt.stats.clone();
    let uploads_before = rt.upload_stats();
    let plane_up_before = rt.plane_uploads();
    let quad_up_before = rt.portal_quad_uploads();
    let content_up_before = rt.content_uploads();
    let rss_start_mb = sample_rss_mb();
    for i in 0..metric_frames {
        if i % 10 == 0 {
            rt.graph.mark_content_dirty(canvas_id);
        }
        let t0 = Instant::now();
        frame(&mut rt);
        cpu_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    let mut sorted = cpu_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p50 = sorted[sorted.len() / 2];
    let p95 = sorted[(sorted.len() * 95 / 100).min(sorted.len() - 1)];
    let stats_after = rt.stats.clone();
    let uploads_delta = rt.upload_stats().delta_since(&uploads_before);
    let plane_up_delta = rt.plane_uploads().delta_since(&plane_up_before);
    let quad_up_delta = rt.portal_quad_uploads().delta_since(&quad_up_before);
    let content_up_delta = rt.content_uploads().delta_since(&content_up_before);
    let rss_end_mb = sample_rss_mb();
    let quad_delta = stats_after.quad_composites - stats_before.quad_composites;
    let plane_delta = stats_after.plane_composites - stats_before.plane_composites;
    let owned_target_bytes = rt.owned_target_bytes();
    let live_targets_now = rt.stats.live_targets();
    let live_slots_now = rt.live_target_slots() as u64;
    let metrics = serde_json::json!({
        "frames": metric_frames,
        "cpu_frame_ms_p50": p50,
        "cpu_frame_ms_p95": p95,
        "cpu_frame_ms_max": sorted.last().copied().unwrap_or(0.0),
        "surfaces_rendered_delta": stats_after.surfaces_rendered - stats_before.surfaces_rendered,
        "portal_composites_delta": stats_after.portal_composites - stats_before.portal_composites,
        "quad_composites_delta": quad_delta,
        "plane_composites_delta": plane_delta,
        "dirty_skips_delta": stats_after.dirty_skips - stats_before.dirty_skips,
        "readbacks_delta": stats_after.readbacks - stats_before.readbacks,
        "target_creations_delta": stats_after.target_creations - stats_before.target_creations,
        "target_drops_delta": stats_after.target_drops - stats_before.target_drops,
        "uploads": {
            "sampling": "delta over the 60-frame loop (start/end snapshots); the plane liveness control below samples its own window",
            "unit": "buffer_rewrites = queue.write_buffer calls; bytes_rewritten = bytes written",
            "plane": plane_up_delta,
            "portal_quad": quad_up_delta,
            "content": content_up_delta,
            "composition_path": {
                "buffer_rewrites": plane_up_delta.buffer_rewrites + quad_up_delta.buffer_rewrites,
                "bytes_rewritten": plane_up_delta.bytes_rewritten + quad_up_delta.bytes_rewritten,
            },
            "path_total": uploads_delta,
            "expected_quad_bytes": quad_delta * 160,
            "expected_plane_bytes_per_composite": 128,
            "expected_quad_bytes_per_composite": 160,
            "excluded": "iced-internal uploads (inside iced_wgpu::Renderer::present), texture/bind-group creation, and driver-side copies are not counted",
        },
        "memory": {
            "sampling": "start/end immediately before/after the 60-frame loop",
            "process_working_set_mb_start": rss_start_mb,
            "process_working_set_mb_end": rss_end_mb,
            "process_working_set_note": "OS residency of this process (includes wgpu driver allocations, iced, assets); not owned-only and not the driver's own report",
            "owned_target_count": live_targets_now,
            "owned_target_slots": live_slots_now,
            "owned_target_bytes": owned_target_bytes,
            "owned_target_bytes_formula": "sum(width*height*4) over live Rgba8UnormSrgb sampled targets (driver alignment/overhead excluded)",
            "gpu_memory_available": false,
            "gpu_memory_unavailable_reason": "wgpu 27 exposes no portable per-process GPU memory query; DXGI budget/usage would be adapter-specific and is not implemented",
            "driver_residency": "not measured (see gpu_memory_unavailable_reason); the process working set above must not be read as owned GPU memory",
        },
        "warm_target_build_seqs": rt
            .graph
            .surfaces()
            .iter()
            .filter_map(|s| rt.target(s.id).map(|t| (s.id.token(), t.build_seq)))
            .collect::<HashMap<_, _>>(),
        "gpu_timing_available": false,
    });

    gates.push(gate(
        "round_unmount_keeps_content",
        content_survived_close == ROUNDS,
        format!("{content_survived_close}/{ROUNDS} rounds kept the unmounted child content"),
    ));
    gates.push(gate(
        "round_count_and_graph_stable",
        surfaces_end == surfaces_baseline && rt.graph.portals().len() == 2,
        format!(
            "surfaces {surfaces_baseline}->{surfaces_end}, portals={}",
            rt.graph.portals().len()
        ),
    ));
    gates.push(gate(
        "lifecycle_had_no_failures",
        failures.is_empty(),
        format!("{} failures", failures.len()),
    ));
    // REAL rebuild accounting: only the 20 changing resizes may create a
    // target; the canvas (never resized) keeps the same build identity.
    gates.push(gate(
        "warm_targets_not_rebuilt_every_round",
        creations_delta == EXPLICIT_RESIZE_CREATIONS
            && top_rebuilds == ROUNDS / 2
            && ui_rebuilds == ROUNDS / 2
            && canvas_build_seq_end == canvas_build_seq,
        format!(
            "target_creations delta={creations_delta} (expected {EXPLICIT_RESIZE_CREATIONS}); \
             top rebuilds={top_rebuilds}, ui rebuilds={ui_rebuilds}, canvas seq {canvas_build_seq}->{canvas_build_seq_end}"
        ),
    ));
    gates.push(gate(
        "steady_frame_readback_zero",
        metrics["readbacks_delta"].as_u64().unwrap_or(1) == 0,
        format!("readbacks delta={}", metrics["readbacks_delta"]),
    ));
    let skips = metrics["dirty_skips_delta"].as_u64().unwrap_or(0);
    gates.push(gate(
        "dirty_skips_unrelated_surfaces",
        skips >= metric_frames as u64,
        format!("dirty_skips={skips} over {metric_frames} frames (top always clean)"),
    ));
    // V-09 upload oracle (review r4 F-830-R4-02). Each pipeline is checked
    // against ITS OWN counter and the composite count it serves, so a missing
    // record site cannot be masked by another pipeline's bytes; the total is
    // checked against the sum of its three independently read parts, so a
    // missing aggregation branch is observable; and the content adapter gets
    // an exact-bytes liveness control below.
    gates.push(gate(
        "quad_upload_counter_matches_composites",
        quad_upload_predicate(quad_delta, &quad_up_delta),
        format!(
            "quad uploads rewrites={} bytes={} vs composites={quad_delta} (expected {} B)",
            quad_up_delta.buffer_rewrites,
            quad_up_delta.bytes_rewritten,
            quad_delta * 160
        ),
    ));
    gates.push(gate(
        "upload_total_is_the_sum_of_its_sources",
        content_up_delta.bytes_rewritten > 0
            && total_upload_predicate(
                &uploads_delta,
                &plane_up_delta,
                &quad_up_delta,
                &content_up_delta,
            ),
        format!(
            "total rewrites={} bytes={} vs plane+quad+content={} / {} (`content`={} B)",
            uploads_delta.buffer_rewrites,
            uploads_delta.bytes_rewritten,
            plane_up_delta.buffer_rewrites
                + quad_up_delta.buffer_rewrites
                + content_up_delta.buffer_rewrites,
            plane_up_delta.bytes_rewritten
                + quad_up_delta.bytes_rewritten
                + content_up_delta.bytes_rewritten,
            content_up_delta.bytes_rewritten
        ),
    ));
    // Content-adapter liveness with EXACT bytes: one fresh 2D canvas with no
    // draw commands performs exactly one camera write of 64B per content pass.
    // Dropping the adapter counter (or the aggregation that feeds it) turns
    // this red even if every other adapter still reports.
    {
        let probe = canvas(&mut rt, [32.0, 32.0], Color::rgb(0.1, 0.2, 0.3));
        let before = rt
            .content_uploads_of(probe)
            .expect("2D adapter has an upload counter");
        frame(&mut rt);
        let delta = rt
            .content_uploads_of(probe)
            .expect("2D adapter has an upload counter")
            .delta_since(&before);
        gates.push(gate(
            "content_upload_counter_is_exact",
            delta.buffer_rewrites == 1 && delta.bytes_rewritten == 64,
            format!(
                "fresh empty canvas content pass: rewrites={} bytes={} (expected 1 / 64)",
                delta.buffer_rewrites, delta.bytes_rewritten
            ),
        ));
    }
    // Plane-path upload oracle: the 3D plane pipeline has its OWN counter
    // (128B uniforms per composite). The lifecycle topology above is 2D-only,
    // so exercise real plane composites and compare against the plane
    // composite count — a quad's 160B can never stand in for it.
    {
        let comps_before = rt.stats.plane_composites;
        let quad_comps_before = rt.stats.quad_composites;
        let plane_before = rt.plane_uploads();
        let quad_before = rt.portal_quad_uploads();
        let total_before = rt.upload_stats();
        let content_before = rt.content_uploads();
        let scene = crate::host::surface_runtime::Scene3dContent {
            screen_overlay: None,
            text_overlay: None,
            hud_overlay: None,
            mesh: crate::render3d::MeshPipeline::new(
                rt.gpu(),
                crate::renderer::target::OffscreenTarget::FORMAT,
            )
            .expect("metric plane mesh"),
            store: Default::default(),
            instances: vec![],
            sky: [0.64625, 0.65575, 0.67075, 1.0],
            view_proj: crate::host::iced_surface_probe::probe_camera(1.6),
        };
        let parent = rt
            .add_surface(
                [320.0, 200.0],
                crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(scene)),
            )
            .expect("metric plane parent");
        let pv = rt.add_view(parent, false).expect("metric plane view");
        let child = {
            let ui = IcedContent::borrowed(
                rt.gpu(),
                [64.0, 64.0],
                Box::new(crate::host::cases::surface_scenarios::CounterUi::default()),
            );
            rt.add_surface(
                [64.0, 64.0],
                crate::host::surface_runtime::SurfaceContent::Iced(Box::new(ui)),
            )
            .expect("metric plane child")
        };
        rt.add_portal(
            pv,
            child,
            MountKind::Plane3d {
                center: [0.0, 0.0, 0.0],
                half: [0.5, 0.5],
            },
            true,
        )
        .expect("metric plane mount");
        frame(&mut rt);
        let plane_n = rt.stats.plane_composites - comps_before;
        let quad_n = rt.stats.quad_composites - quad_comps_before;
        let plane_d = rt.plane_uploads().delta_since(&plane_before);
        let quad_d = rt.portal_quad_uploads().delta_since(&quad_before);
        let content_d = rt.content_uploads().delta_since(&content_before);
        let total_d = rt.upload_stats().delta_since(&total_before);
        gates.push(gate(
            "plane_upload_counter_matches_composites",
            plane_upload_predicate(plane_n, &plane_d),
            format!(
                "plane uploads rewrites={} bytes={} vs plane composites={plane_n} (quad composites={quad_n})",
                plane_d.buffer_rewrites, plane_d.bytes_rewritten
            ),
        ));
        gates.push(gate(
            "plane_control_total_is_the_sum_of_its_sources",
            total_upload_predicate(&total_d, &plane_d, &quad_d, &content_d),
            format!(
                "plane block total={}/{} B vs plane+quad+content={}/{} B",
                total_d.buffer_rewrites,
                total_d.bytes_rewritten,
                plane_d.buffer_rewrites + quad_d.buffer_rewrites + content_d.buffer_rewrites,
                plane_d.bytes_rewritten + quad_d.bytes_rewritten + content_d.bytes_rewritten
            ),
        ));
    }

    // V-08-style ORACLE SENSITIVITY for the measurement gates (review r4
    // F-830-R4-02): the SAME predicates that pass on real data must FAIL when
    // the plane accounting or the content aggregation is actually omitted.
    // Both hooks keep the real GPU writes; only the accounting is skipped.
    {
        // (a) plane omission
        let comps0 = rt.stats.plane_composites;
        let plane0 = rt.plane_uploads();
        rt.set_plane_upload_accounting(false);
        frame(&mut rt);
        rt.set_plane_upload_accounting(true);
        let n = rt.stats.plane_composites - comps0;
        let omitted = rt.plane_uploads().delta_since(&plane0);
        let omission_caught = n >= 1 && !plane_upload_predicate(n, &omitted);
        // Legit recovery: the next frame records again.
        let comps1 = rt.stats.plane_composites;
        let plane1 = rt.plane_uploads();
        frame(&mut rt);
        let n_legit = rt.stats.plane_composites - comps1;
        let legit = plane_upload_predicate(n_legit, &rt.plane_uploads().delta_since(&plane1));
        gates.push(gate(
            "neg_plane_upload_omission_caught",
            omission_caught && legit,
            format!(
                "omitted plane accounting: composites={n} recorded={}/{} B (must fail); \
                 restored: composites={n_legit} legit={legit}",
                omitted.buffer_rewrites, omitted.bytes_rewritten
            ),
        ));

        // (b) content aggregation omission
        let total0 = rt.upload_stats();
        let plane0 = rt.plane_uploads();
        let quad0 = rt.portal_quad_uploads();
        let content0 = rt.content_uploads();
        rt.set_content_upload_aggregation(false);
        frame(&mut rt);
        // Read the TOTAL while the aggregation is still omitted: the hook
        // changes the REPORTED total, not the real writes.
        let total = rt.upload_stats().delta_since(&total0);
        rt.set_content_upload_aggregation(true);
        let plane_d = rt.plane_uploads().delta_since(&plane0);
        let quad_d = rt.portal_quad_uploads().delta_since(&quad0);
        let content_d = rt.content_uploads().delta_since(&content0);
        let omission_caught = content_d.bytes_rewritten > 0
            && !total_upload_predicate(&total, &plane_d, &quad_d, &content_d);
        // Legit recovery.
        let total1 = rt.upload_stats();
        let plane1 = rt.plane_uploads();
        let quad1 = rt.portal_quad_uploads();
        let content1 = rt.content_uploads();
        frame(&mut rt);
        let legit = total_upload_predicate(
            &rt.upload_stats().delta_since(&total1),
            &rt.plane_uploads().delta_since(&plane1),
            &rt.portal_quad_uploads().delta_since(&quad1),
            &rt.content_uploads().delta_since(&content1),
        );
        gates.push(gate(
            "neg_content_upload_omission_caught",
            omission_caught && legit,
            format!(
                "omitted content aggregation: total={}/{} B parts={}/{} B (must fail); restored legit={legit}",
                total.buffer_rewrites,
                total.bytes_rewritten,
                plane_d.buffer_rewrites + quad_d.buffer_rewrites + content_d.buffer_rewrites,
                plane_d.bytes_rewritten + quad_d.bytes_rewritten + content_d.bytes_rewritten
            ),
        ));
    }

    // V-07/V-09: memory facts are reported with units/sampling/boundaries and
    // keep OS residency, owned targets and the unavailable driver residency
    // strictly separate.
    let memory = &metrics["memory"];
    gates.push(gate(
        "memory_facts_reported_and_separated",
        rss_start_mb.is_some()
            && rss_end_mb.is_some()
            && owned_target_bytes > 0
            && live_targets_now == live_slots_now
            && memory["gpu_memory_available"].as_bool() == Some(false)
            && memory["gpu_memory_unavailable_reason"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
            && memory["process_working_set_note"].is_string()
            && memory["driver_residency"].is_string(),
        format!(
            "rss {rss_start_mb:?}->{rss_end_mb:?} MB (process), owned targets {live_targets_now} \
             × slots {live_slots_now} = {owned_target_bytes} B, GPU memory unavailable (named)"
        ),
    ));

    // ===== V-08 negative controls (real faults in the real path) ==========
    let mut negative_controls: Vec<serde_json::Value> = Vec::new();

    // (1) clip: the child quad overflows its clip; disabling the scissor
    //     must make the overflow pixel visible. Both the legit and the
    //     sabotaged pixel come from the SAME production draw path.
    {
        let parent = canvas(&mut rt, [200.0, 200.0], Color::rgb(0.5, 0.5, 0.5));
        let parent_view = rt.add_view(parent, false).expect("clip parent view");
        let child = canvas(&mut rt, [100.0, 100.0], Color::rgb(1.0, 0.0, 1.0));
        rt.add_view(child, false).expect("clip child view");
        rt.add_portal(
            parent_view,
            child,
            // quad spans 20..120, clip only 20..70: (90,40) is quad∩¬clip.
            MountKind::Flat2d {
                offset: [20.0, 20.0],
                scale: [1.0, 1.0],
                clip: Rect::at(20.0, 20.0, 50.0, 50.0),
            },
            true,
        )
        .expect("clip mount");
        frame(&mut rt);
        let inside = rt.probe_pixel(parent, [35.0, 35.0]).unwrap_or([0, 0, 0, 0]);
        let outside = rt.probe_pixel(parent, [90.0, 40.0]).unwrap_or([0, 0, 0, 0]);
        let legit_ok = is_magenta(inside) && is_gray(outside);
        // Inject the fault: same draw call, no scissor.
        rt.set_scissor_enabled(false);
        rt.graph.mark_content_dirty(child);
        frame(&mut rt);
        let leak = rt.probe_pixel(parent, [90.0, 40.0]).unwrap_or([0, 0, 0, 0]);
        rt.set_scissor_enabled(true);
        rt.graph.mark_content_dirty(child);
        frame(&mut rt);
        let restored = rt.probe_pixel(parent, [90.0, 40.0]).unwrap_or([0, 0, 0, 0]);
        let caught = is_magenta(leak) && is_gray(restored);
        negative_controls.push(serde_json::json!({
            "control": "clip_scissor",
            "inside_clip": inside,
            "outside_clip": outside,
            "sabotage_leak": leak,
            "restored": restored,
            "legit_ok": legit_ok,
            "sabotage_caught": caught,
        }));
        gates.push(gate(
            "neg_clip_caught",
            legit_ok && caught,
            format!("inside={inside:?} outside={outside:?} leak={leak:?} restored={restored:?}"),
        ));
    }

    // (2) alpha: the formal premultiplied oracle is re-run against a REAL
    //     opaque-blend pipeline; the same gate must reject it.
    {
        let parent = canvas(&mut rt, [300.0, 200.0], Color::rgb(0.5, 0.5, 0.5));
        let parent_view = rt.add_view(parent, false).expect("alpha parent view");
        let child = canvas(&mut rt, [80.0, 80.0], Color::rgba(0.25, 0.0, 0.0, 0.5));
        rt.add_view(child, false).expect("alpha child view");
        rt.add_portal(parent_view, child, flat(40.0, 40.0, 80.0, 80.0), true)
            .expect("alpha mount");
        frame(&mut rt);
        let legit = rt.probe_pixel(parent, [80.0, 80.0]).unwrap_or([0, 0, 0, 0]);
        let expected = [188u8, 138, 138, 255];
        let legit_ok = near(legit, expected, 12);
        rt.set_premultiplied_portal_blend(false);
        rt.graph.mark_content_dirty(child);
        frame(&mut rt);
        let opaque = rt.probe_pixel(parent, [80.0, 80.0]).unwrap_or([0, 0, 0, 0]);
        rt.set_premultiplied_portal_blend(true);
        rt.graph.mark_content_dirty(child);
        frame(&mut rt);
        let restored = rt.probe_pixel(parent, [80.0, 80.0]).unwrap_or([0, 0, 0, 0]);
        let caught = (opaque[0].abs_diff(137) <= 8 && opaque[1] <= 8 && opaque[2] <= 8)
            && near(restored, expected, 12);
        negative_controls.push(serde_json::json!({
            "control": "alpha_premultiplied",
            "legit_pixel": legit,
            "expected": expected,
            "sabotage_pixel": opaque,
            "restored_pixel": restored,
            "legit_ok": legit_ok,
            "sabotage_caught": caught,
        }));
        gates.push(gate(
            "neg_opaque_alpha_caught",
            legit_ok && caught,
            format!(
                "legit={legit:?} expected≈{expected:?} opaque={opaque:?} restored={restored:?}"
            ),
        ));
    }

    // (3) double content pass: the runtime counter observes a REAL second
    //     `render_content` call (not a fabricated +2).
    {
        rt.graph.mark_content_dirty(canvas_id);
        rt.graph.mark_content_dirty(ui);
        frame(&mut rt);
        let single = rt
            .stats
            .last_frame_renders
            .get(&canvas_id)
            .copied()
            .unwrap_or(0);
        let doubled = rt.force_double_render(canvas_id);
        negative_controls.push(serde_json::json!({
            "control": "single_content_pass",
            "legit_renders": single,
            "sabotage_renders": doubled,
            "legit_ok": single == 1,
            "sabotage_caught": doubled == 2,
        }));
        gates.push(gate(
            "neg_double_render_caught",
            single == 1 && doubled == 2,
            format!("legit renders={single}, second real pass count={doubled}"),
        ));
    }

    // (4) duplicate events: the runtime duplicates the per-surface list in
    //     the real frame path; a real key event must insert ONE character,
    //     and the sabotaged frame must insert two.
    {
        let dup = rt
            .add_surface(
                [200.0, 120.0],
                crate::host::surface_runtime::SurfaceContent::Iced(Box::new(
                    IcedContent::borrowed(rt.gpu(), [200.0, 120.0], Box::new(DupUi::default())),
                )),
            )
            .expect("dup ui");
        rt.add_view(dup, false).expect("dup view");
        frame(&mut rt);
        let _ = rt.focus_iced_text_input(dup, iced_wgpu::core::widget::Id::new("dup-input"));
        let key = iced_wgpu::core::keyboard::Key::Character(iced_wgpu::core::SmolStr::new("a"));
        let key_event =
            iced_wgpu::core::Event::Keyboard(iced_wgpu::core::keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced_wgpu::core::keyboard::key::Physical::Unidentified(
                    iced_wgpu::core::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced_wgpu::core::keyboard::Location::Standard,
                modifiers: iced_wgpu::core::keyboard::Modifiers::default(),
                text: Some(iced_wgpu::core::SmolStr::new("a")),
                repeat: false,
            });
        let cursors = HashMap::from([(dup, [40.0f32, 20.0])]);
        let mut ev = HashMap::new();
        ev.insert(dup, vec![key_event.clone()]);
        let _ = rt.frame_with_cursors(None, &ev, &cursors);
        let legit = dup_text(&rt, dup);
        // Reset the control, then inject the fault.
        if let Some(crate::host::surface_runtime::SurfaceContent::Iced(c)) = rt.content_mut(dup) {
            if let Some(ui) = c.ui.as_any_mut().downcast_mut::<DupUi>() {
                ui.text.clear();
            }
        }
        rt.set_duplicate_events(true);
        let mut ev = HashMap::new();
        ev.insert(dup, vec![key_event]);
        let _ = rt.frame_with_cursors(None, &ev, &cursors);
        let sabotage = dup_text(&rt, dup);
        rt.set_duplicate_events(false);
        negative_controls.push(serde_json::json!({
            "control": "duplicate_events",
            "legit_text": legit,
            "sabotage_text": sabotage,
            "legit_ok": legit == "a",
            "sabotage_caught": sabotage == "aa",
        }));
        gates.push(gate(
            "neg_duplicate_events_caught",
            legit == "a" && sabotage == "aa",
            format!("legit text={legit:?}, duplicated text={sabotage:?}"),
        ));
    }

    // (5) per-frame readback: `probe_pixel` is a REAL readback; the metric
    //     counter must observe each one.
    {
        let before = rt.stats.readbacks;
        for _ in 0..5 {
            frame(&mut rt);
            let _ = rt.probe_pixel(canvas_id, [10.0, 10.0]);
        }
        let delta = rt.stats.readbacks - before;
        negative_controls.push(serde_json::json!({
            "control": "per_frame_readback",
            "readbacks_observed": delta,
            "legit_ok": true,
            "sabotage_caught": delta == 5,
        }));
        gates.push(gate(
            "neg_readback_counter_caught",
            delta == 5,
            format!("readbacks observed={delta} for 5 real probes"),
        ));
    }

    // (6) second device: create a REAL second device and content on it; the
    //     runtime must refuse EVERY content adapter (2D canvas, iced UI and
    //     3D scene) at the entry with a classified error, keep the graph
    //     untouched, and still accept same-device content of all three kinds.
    {
        match GpuDevice::new() {
            Ok(second) => {
                let snapshot = rt.graph.snapshot();
                let creations_before = rt.stats.target_creations;
                let foreign_canvas = Game2dContent::simple(&second, Color::rgb(0.0, 0.0, 0.0));
                let canvas_err = rt.add_surface(
                    [16.0, 16.0],
                    crate::host::surface_runtime::SurfaceContent::Game2d(Box::new(foreign_canvas)),
                );
                let foreign_ui = crate::host::surface_runtime::IcedContent::borrowed(
                    &second,
                    [16.0, 16.0],
                    Box::new(crate::host::cases::surface_scenarios::CounterUi::default()),
                );
                let ui_err = rt.add_surface(
                    [16.0, 16.0],
                    crate::host::surface_runtime::SurfaceContent::Iced(Box::new(foreign_ui)),
                );
                // The 3D branch used to return `None` (no identity) and was
                // silently admitted (review r2 F04).
                let foreign_scene = crate::host::surface_runtime::Scene3dContent {
                    screen_overlay: None,
                    text_overlay: None,
                    hud_overlay: None,
                    mesh: crate::render3d::MeshPipeline::new(
                        &second,
                        crate::renderer::target::OffscreenTarget::FORMAT,
                    )
                    .expect("foreign mesh"),
                    store: Default::default(),
                    instances: vec![],
                    sky: [0.0, 0.0, 0.0, 1.0],
                    view_proj: crate::host::iced_surface_probe::probe_camera(1.0),
                };
                let scene_err = rt.add_surface(
                    [16.0, 16.0],
                    crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(foreign_scene)),
                );
                let caught = matches!(canvas_err, Err(CompositionError::DeviceMismatch { .. }))
                    && matches!(ui_err, Err(CompositionError::DeviceMismatch { .. }))
                    && matches!(scene_err, Err(CompositionError::DeviceMismatch { .. }));
                let graph_intact = snapshot == rt.graph.snapshot();
                let no_targets = rt.stats.target_creations == creations_before;
                // Unverifiable raw-device content is refused, not admitted.
                let raw_mesh = crate::render3d::MeshPipeline::new_with(
                    rt.gpu().device(),
                    rt.gpu().queue(),
                    crate::renderer::target::OffscreenTarget::FORMAT,
                )
                .expect("raw mesh");
                let unverifiable = rt.add_surface(
                    [16.0, 16.0],
                    crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(
                        crate::host::surface_runtime::Scene3dContent {
                            screen_overlay: None,
                            text_overlay: None,
                            hud_overlay: None,
                            mesh: raw_mesh,
                            store: Default::default(),
                            instances: vec![],
                            sky: [0.0, 0.0, 0.0, 1.0],
                            view_proj: crate::host::iced_surface_probe::probe_camera(1.0),
                        },
                    )),
                );
                let unverifiable_refused = matches!(
                    unverifiable,
                    Err(CompositionError::DeviceUnverifiable { .. })
                );
                // Legit controls: same device accepted for all three kinds.
                let legit = canvas(&mut rt, [16.0, 16.0], Color::rgb(0.0, 0.0, 0.0));
                let legit_ok = rt.slot(legit).is_some();
                let legit_scene = rt.add_surface(
                    [16.0, 16.0],
                    crate::host::surface_runtime::SurfaceContent::Scene3d(Box::new(
                        crate::host::surface_runtime::Scene3dContent {
                            screen_overlay: None,
                            text_overlay: None,
                            hud_overlay: None,
                            mesh: crate::render3d::MeshPipeline::new(
                                rt.gpu(),
                                crate::renderer::target::OffscreenTarget::FORMAT,
                            )
                            .expect("own mesh"),
                            store: Default::default(),
                            instances: vec![],
                            sky: [0.0, 0.0, 0.0, 1.0],
                            view_proj: crate::host::iced_surface_probe::probe_camera(1.0),
                        },
                    )),
                );
                let legit_scene_ok = legit_scene.is_ok();
                negative_controls.push(serde_json::json!({
                    "control": "single_device",
                    "second_device_created": true,
                    "foreign_canvas_refused": matches!(canvas_err, Err(CompositionError::DeviceMismatch { .. })),
                    "foreign_ui_refused": matches!(ui_err, Err(CompositionError::DeviceMismatch { .. })),
                    "foreign_scene_refused": matches!(scene_err, Err(CompositionError::DeviceMismatch { .. })),
                    "unverifiable_refused": unverifiable_refused,
                    "graph_intact": graph_intact,
                    "no_target_created": no_targets,
                    "legit_ok": legit_ok && legit_scene_ok,
                    "sabotage_caught": caught,
                }));
                gates.push(gate(
                    "neg_second_device_caught",
                    caught
                        && unverifiable_refused
                        && graph_intact
                        && no_targets
                        && legit_ok
                        && legit_scene_ok,
                    format!(
                        "foreign 2d/iced/3d refused={caught}, unverifiable refused={unverifiable_refused}, \
                         graph_intact={graph_intact}, no_target_created={no_targets}, same-device control={} ",
                        legit_ok && legit_scene_ok
                    ),
                ));
            }
            Err(e) => {
                failures.push(format!("second device unavailable for the control: {e}"));
                negative_controls.push(serde_json::json!({
                    "control": "single_device",
                    "second_device_created": false,
                    "reason": format!("{e}"),
                }));
                gates.push(gate(
                    "neg_second_device_caught",
                    false,
                    format!("second device unavailable: {e}"),
                ));
            }
        }
    }

    // (7) dependency order: composite BEFORE the dirty child renders. The
    //     runtime must classify the submit-order violation.
    {
        rt.graph.mark_content_dirty(canvas_id);
        rt.set_composite_before_render(true);
        let err = rt.frame(None, &HashMap::new()).err();
        rt.set_composite_before_render(false);
        let caught = matches!(err, Some(CompositionError::DependencyOrder { .. }));
        rt.graph.mark_content_dirty(canvas_id);
        let recovered = rt.frame(None, &HashMap::new()).is_ok();
        negative_controls.push(serde_json::json!({
            "control": "dependency_order",
            "classified_error": matches!(err, Some(CompositionError::DependencyOrder { .. })),
            "recovered_frame": recovered,
            "legit_ok": recovered,
            "sabotage_caught": caught,
        }));
        gates.push(gate(
            "neg_dependency_order_caught",
            caught && recovered,
            format!("classified={caught}, recovered={recovered}"),
        ));
    }

    let all_ok = gates.iter().all(|g| g.positive_passed) && failures.is_empty();
    let report = LifecycleReport {
        adapter,
        device_token: token,
        rounds: ROUNDS,
        lifecycle: serde_json::json!({
            "surfaces_baseline": surfaces_baseline,
            "surfaces_end": surfaces_end,
            "rounds": round_facts,
            "frames": frames,
            "target_creations_total": rt.stats.target_creations,
            "target_creations_baseline": creations_baseline,
            "target_drops_total": rt.stats.target_drops,
            "target_drops_baseline": drops_baseline,
        }),
        metrics: serde_json::json!({
            "composition": metrics,
            "negative_controls": negative_controls,
        }),
        gates,
        all_gates_behaved: all_ok,
        keyframes: Vec::new(),
        failures,
        gpu_timing_available: false,
        notes: vec![
            "GPU timestamps are not wired; CPU frame times only (never fabricated)".into(),
            "target rebuilds are counted from real OffscreenTarget::build_seq values".into(),
            "negative controls inject faults into the production frame path (scissor, blend, events, order, device)".into(),
            format!("replay={replay}"),
        ],
    };
    let exit = if all_ok { "pass" } else { "fail" };
    let _ = write_report(report_dir, "surface-lifecycle", replay, exit, &report);
    if !all_ok {
        eprintln!("surface-lifecycle: gates failed (see report.json)");
    }
    rt.gpu().wait_idle();
    if all_ok {
        0
    } else {
        1
    }
}

// Keep `PortalId`/`ViewId` reachable for readers of the round facts.
#[allow(unused)]
fn _ids(_: PortalId, _: ViewId) {}

// ---------------------------------------------------------------------------
// Upload-measurement predicates (single source of truth for the V-09 gates and
// for the V-08 omission controls — review r4 F-830-R4-02).
// ---------------------------------------------------------------------------

/// One plane composite ⇒ exactly one 128B uniform rewrite.
fn plane_upload_predicate(composites: u64, delta: &crate::renderer::UploadStats) -> bool {
    composites >= 1
        && delta.buffer_rewrites == composites
        && delta.bytes_rewritten == composites * 128
}

/// One drawn 2D portal ⇒ exactly one 160B uniform rewrite.
fn quad_upload_predicate(composites: u64, delta: &crate::renderer::UploadStats) -> bool {
    composites >= 1
        && delta.buffer_rewrites == composites
        && delta.bytes_rewritten == composites * 160
}

/// The reported total must be EXACTLY the sum of its independently measured
/// parts, so an omitted aggregation branch is observable.
fn total_upload_predicate(
    total: &crate::renderer::UploadStats,
    plane: &crate::renderer::UploadStats,
    quad: &crate::renderer::UploadStats,
    content: &crate::renderer::UploadStats,
) -> bool {
    total.buffer_rewrites == plane.buffer_rewrites + quad.buffer_rewrites + content.buffer_rewrites
        && total.bytes_rewritten
            == plane.bytes_rewritten + quad.bytes_rewritten + content.bytes_rewritten
}
