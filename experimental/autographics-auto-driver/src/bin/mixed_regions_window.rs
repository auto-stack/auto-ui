//! PLAN-054 混合区域公开原生窗口入口（v2 装配/挂卸/路由全部经
//! `MixedRegionsHost` 单一生产链——本 bin 只持窗口/探针相位/验证报告）。
//! 一个真实 OS 窗口 = MixedRegionsHost 组合：UI（Iced 控件）为窗口面
//! view；组件/root 实例区域按 metadata 目录＋真实布局激活（应用活区域
//! ≤4）；条件隐藏＝撤呈现；输入经 CompositionInput 门户优先命中。
//!
//! --probe   正控相位（全部真实 OS 通道 posted 注入）：
//!           bump 按钮×1 → p1 区域×1 → strip 区域×1（布局 A）→
//!           Tab+Enter（真实 OS 键盘；toggle→隐藏 p1/p2，撤呈现）→
//!           strip 于移动后布局×1 → Tab+Enter（恢复；新挂载代次）→
//!           p1 重挂后×1 → resize（真实窗 resize；区域保持活跃）→
//!           空闲零 Tick。验证 Auto 状态文字、逐实例区域状态、挂卸代次、
//!           布局跟随路由；写 JSON 报告；pass exit 0。
//!           真实 native SendInput/焦点/DPI 切换通道沿 048/049 手工取证
//!           先例（不在本自动链；scale_factor 与 posted 坐标=逻辑×scale
//!           在报告记录）。
//! (默认)    素窗（无自动化、无报告）
//! --manual  可执行手工门（PLAN-066 T-02）：人真实操作本窗（Tab/Enter、点击、
//!           失焦、移出、resize、第二 DPI scale）；关窗时写
//!           `manual-native-gate.json` 记录真实 OS 观察，auto_pass=false
//!           （手工证据，不计自动 pass）。
//! GPU/窗口/焦点单实例运行。
use agp::host::iced_surface_probe as probe;
use agp::renderer::{GpuDevice, WindowTarget};
use autographics_auto_driver::m1_evidence::MIXED_MANUAL_CHECKLIST;
use autographics_auto_driver::node_mixed_regions::MixedRegionsUiApp;
use autographics_auto_driver::node_mixed_regions_host::MixedRegionsHost;
use autographics_auto_driver::node_ui::PlatformKey;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key as WKey;
use winit::keyboard::NamedKey as WNamed;
use winit::window::{Window, WindowId};

const TITLE: &str = "AGE mixed-regions (PLAN-054)";
const LOGICAL: [f32; 2] = [480.0, 560.0];
const RESIZED: [f32; 2] = [400.0, 420.0];
const P1: &str = "Dash/p1#arc";
const STRIP: &str = "Dash#strip";
const BUMP: &str = "Dash#bump";
const TOGGLE: &str = "Dash#toggle";
const VK_TAB: u32 = 0x09;
const VK_RETURN: u32 = 0x0D;

struct App {
    probe: bool,
    /// PLAN-066 T-02 manual gate: record real OS events (human-run) without
    /// claiming an automatic pass.
    manual: bool,
    report: PathBuf,
    ui_path: String,
    world_path: String,
    window: Option<Arc<Window>>,
    target: Option<WindowTarget>,
    host: Option<MixedRegionsHost>,
    cursor_px: [f32; 2],
    frames: u64,
    started: Instant,
    phase: usize,
    /// PLAN-066 T-03: per-phase clock; state gates decide transitions instead of
    /// fixed global timestamps (a real Focused(false)/0×0 event must not shift
    /// the whole schedule into a race).
    phase_started: Instant,
    passed: bool,
    posted_clicks: u32,
    posted_keys: u32,
    scale_observed: f32,
    logical_sizes: Vec<[f32; 2]>,
    physical_sizes: Vec<[u32; 2]>,
    epochs_before_toggle: [Option<u64>; 2],
    epochs_after_remount: [Option<u64>; 2],
    marks: Vec<(String, u64)>,
    manual_seen: ManualSeen,
    /// PLAN-071 T-04: read-only actual-read checkpoints around each manual OS
    /// event (scale/size/counters/epochs at that instant; no business access).
    manual_checkpoints: Vec<serde_json::Value>,
    /// Keyboard target of the current adaptive Tab job (None = no job).
    kb_target: Option<String>,
    kb_posted: u32,
    /// Zero-size Resized events (minimize) are ignored, not applied.
    zero_resizes: u64,
    /// Strip bounds before the first toggle (layout-follow gate).
    strip_before: Option<[f32; 4]>,
    /// Named failure reason; makes `passed` false and is reported.
    failure: Option<String>,
}

/// Real OS-channel observations for the manual gate (never auto-passed).
#[derive(Default)]
struct ManualSeen {
    tabs: u32,
    enters: u32,
    clicks: u32,
    focus_lost: u32,
    cursor_left: u32,
    resizes: u32,
    scales: Vec<f32>,
    /// F-SCALE-02（PLAN-076）：开窗时播种的起始 scale 档；每次真实
    /// ScaleFactorChanged 后跟新——关窗写报告用它确保起始档在
    /// `scales_observed` 里（合并幂等）。
    last_scale: Option<f32>,
    /// PLAN-071 T-04: real ScaleFactorChanged events (before, after) — the
    /// window's actual scale readback on both sides of the OS change.
    scale_factor_changed: Vec<(f32, f32)>,
}

impl App {
    fn new(
        probe: bool,
        manual: bool,
        ui_path: String,
        world_path: String,
        report: PathBuf,
    ) -> Self {
        Self {
            probe,
            manual,
            report,
            ui_path,
            world_path,
            window: None,
            target: None,
            host: None,
            cursor_px: [-1.0, -1.0],
            frames: 0,
            started: Instant::now(),
            phase: 0,
            phase_started: Instant::now(),
            passed: false,
            posted_clicks: 0,
            posted_keys: 0,
            scale_observed: 1.0,
            logical_sizes: Vec::new(),
            physical_sizes: Vec::new(),
            epochs_before_toggle: [None; 2],
            epochs_after_remount: [None; 2],
            marks: Vec::new(),
            manual_seen: ManualSeen::default(),
            manual_checkpoints: Vec::new(),
            kb_target: None,
            kb_posted: 0,
            zero_resizes: 0,
            strip_before: None,
            failure: None,
        }
    }

    /// 逻辑控件 bounds → 物理客户区 px（posted 点击通道＝逻辑×scale）。
    fn bounds_px(&mut self, key: &str) -> Option<(f32, f32)> {
        let window = self.window.as_ref()?;
        let r = self.host.as_mut()?.widget_bounds(key)?;
        let scale = window.scale_factor() as f32;
        Some(((r.x + r.w / 2.0) * scale, (r.y + r.h / 2.0) * scale))
    }

    fn click(&mut self, key: &str) {
        if let Some(px) = self.bounds_px(key) {
            probe::os_click_client_once(TITLE, px, true);
            self.posted_clicks += 1;
        }
    }

    fn scene_values(&mut self) -> (f32, f32, f32) {
        let h = self.host.as_mut().expect("host");
        let st = h.app().scene_state();
        let qx = |path: &str| -> f32 {
            match &st.iter().find(|s| s.path == path).expect(path).elems[0] {
                autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => *x,
                autographics_auto_driver::node_mixed::SceneElemState::Cylinder { tx, .. } => *tx,
                _ => 0.0,
            }
        };
        (qx("Dash/p1"), qx("Dash/p2"), qx("Dash/m3"))
    }

    fn goto(&mut self, phase: usize) {
        self.phase = phase;
        self.phase_started = Instant::now();
    }

    fn held(&self, ms: u128) -> bool {
        self.phase_started.elapsed().as_millis() >= ms
    }

    fn active(&mut self) -> usize {
        self.host.as_mut().map_or(0, |h| h.active_count())
    }

    fn fail(&mut self, why: impl Into<String>) {
        if self.failure.is_none() {
            self.failure = Some(why.into());
        }
    }

    /// PLAN-071 T-04: one read-only actual-read snapshot around a manual OS
    /// event — scale/size from the real window, counters/epochs from the host
    /// public faces. Recording never touches Tick/dispatch/routing.
    fn manual_checkpoint(&mut self, label: &str) {
        if !self.manual {
            return;
        }
        let mut entry = serde_json::json!({ "label": label });
        if let Some(w) = &self.window {
            entry["scale"] = serde_json::json!(w.scale_factor());
            let size = w.inner_size();
            entry["physical"] = serde_json::json!([size.width, size.height]);
        }
        if let Some(h) = self.host.as_mut() {
            entry["logical"] = serde_json::json!(h.logical());
            entry["count_text"] = serde_json::json!(h.app().ui.texts(0).first().cloned());
            entry["callbacks"] = serde_json::json!(h.app().ui.callbacks);
            entry["active_regions"] = serde_json::json!(h.active_keys());
            entry["epochs"] = serde_json::json!([
                h.region_epoch(P1),
                h.region_epoch(STRIP),
            ]);
        }
        self.manual_checkpoints.push(entry);
    }

    fn bounds4(&mut self, key: &str) -> Option<[f32; 4]> {
        self.host
            .as_mut()
            .and_then(|h| h.widget_bounds(key))
            .map(|r| [r.x, r.y, r.w, r.h])
    }

    /// Adaptive real-keyboard job: post Tab until the requested ring entry is
    /// focused (a real Focused(false) clears the focus, so the first Tab after
    /// it re-acquires), then Enter. One posted key per frame, bounded attempts;
    /// a timeout becomes a named failure instead of a silent count mismatch.
    fn kb_start(&mut self, target: &str) {
        self.kb_target = Some(target.to_string());
        self.kb_posted = 0;
    }

    fn kb_step(&mut self) -> bool {
        let Some(target) = self.kb_target.clone() else {
            return true;
        };
        if self.kb_posted > 12 {
            self.fail(format!("keyboard focus timeout for '{target}'"));
            self.kb_target = None;
            return true;
        }
        let active = self
            .host
            .as_ref()
            .is_some_and(|h| h.keyboard_focus_active());
        let focused = self
            .host
            .as_ref()
            .and_then(|h| h.focused_key().map(str::to_string));
        // Always route at least one Tab per round so the real Tab/Enter channel
        // is exercised even when the ring already sits on the target.
        if self.kb_posted > 0 && active && focused.as_deref() == Some(target.as_str()) {
            if probe::os_key_client_once(TITLE, VK_RETURN) {
                self.posted_keys += 1;
            }
            self.kb_target = None;
            self.kb_posted = 0;
            return true;
        }
        if probe::os_key_client_once(TITLE, VK_TAB) {
            self.posted_keys += 1;
        }
        self.kb_posted += 1;
        false
    }

    fn verify_and_finish(&mut self, e: &ActiveEventLoop) {
        if self.manual {
            let seen = &self.manual_seen;
            // PLAN-071 T-04: the same real-OS observation now lands in the
            // versioned evidence schema (autographics.m1-native-evidence/1).
            // N1..N6 carry the observed counters each; the human conclusions
            // and observer name stay null until a person fills them in, and
            // the validator keeps the record pending until then.
            let mut record = autographics_auto_driver::m1_evidence::EvidenceRecord::new(
                "mixed-regions",
                "native-manual",
            );
            let ui_sha = std::fs::read(&self.ui_path)
                .ok()
                .map(|b| agp::renderer::capture::sha256_hex(&b));
            let world_sha = std::fs::read(&self.world_path)
                .ok()
                .map(|b| agp::renderer::capture::sha256_hex(&b));
            let mut bindings = serde_json::json!({
                "ui_json_path": self.ui_path,
                "world_at_path": self.world_path,
            });
            if let Some(sha) = &ui_sha {
                bindings["ui.json#sha256"] = serde_json::json!(sha);
            }
            if let Some(sha) = &world_sha {
                bindings["world.at#sha256"] = serde_json::json!(sha);
            }
            record.bind_project("mixed-artifact", bindings);
            let runner = std::env::current_exe().ok().map(|p| {
                let bytes = std::fs::read(&p).unwrap_or_default();
                serde_json::json!({
                    "window_runner_path": p,
                    "window_runner_sha256": agp::renderer::capture::sha256_hex(&bytes),
                })
            });
            record.bind_candidate(serde_json::json!({
                "window": TITLE,
                "runner": runner,
            }));
            if let Some(w) = &self.window {
                let size = w.inner_size();
                let scale = w.scale_factor();
                // F-SCALE-02（PLAN-076）：关窗读回前先以跟踪的起始档播种
                // （observe_window 合并幂等——单次切档会话保住开窗档）；
                // 随后的最终读回覆盖 window.* 实测字段。
                if let Some(start) = self.manual_seen.last_scale {
                    record.observe_window(
                        start as f64,
                        [
                            size.width.max(1) as f64 / scale,
                            size.height.max(1) as f64 / scale,
                        ],
                        [size.width, size.height],
                    );
                }
                record.observe_window(
                    scale,
                    [
                        size.width.max(1) as f64 / scale,
                        size.height.max(1) as f64 / scale,
                    ],
                    [size.width, size.height],
                );
            }
            for (before, after) in &seen.scale_factor_changed {
                record.record_scale_change(*before as f64, *after as f64, "manual");
            }
            for cp in &self.manual_checkpoints {
                let label = cp["label"].as_str().unwrap_or("checkpoint").to_string();
                record.checkpoint(&label, cp.clone());
            }
            for (kind, count) in [
                ("tabs", seen.tabs),
                ("enters", seen.enters),
                ("clicks", seen.clicks),
                ("focus_lost", seen.focus_lost),
                ("cursor_left", seen.cursor_left),
                ("resizes", seen.resizes),
            ] {
                for _ in 0..count {
                    record.count_os_event(kind);
                }
            }
            record.observe_checklist(
                "N1",
                serde_json::json!({ "tabs": seen.tabs, "enters": seen.enters }),
            );
            record.observe_checklist(
                "N2",
                serde_json::json!({ "clicks": seen.clicks }),
            );
            record.observe_checklist(
                "N3",
                serde_json::json!({ "focus_lost": seen.focus_lost, "enters": seen.enters }),
            );
            record.observe_checklist(
                "N4",
                serde_json::json!({ "cursor_left": seen.cursor_left }),
            );
            record.observe_checklist(
                "N5",
                serde_json::json!({
                    "resizes": seen.resizes,
                    "logical_sizes": self.logical_sizes,
                    "physical_sizes": self.physical_sizes,
                }),
            );
            record.observe_checklist(
                "N6",
                serde_json::json!({
                    "scale_factor_changed": seen.scale_factor_changed,
                    "scales": seen.scales,
                }),
            );
            record.set_tier_channel("native");
            let mut report = record.to_value();
            // Legacy face (PLAN-066 T-02 consumers) keeps its field names.
            report["title"] = serde_json::json!(TITLE);
            report["mode_note"] = serde_json::json!("manual-gate (PLAN-066 T-02; PLAN-071 T-04 evidence schema)");
            report["observed"] = serde_json::json!({
                "tabs": seen.tabs,
                "enters": seen.enters,
                "clicks": seen.clicks,
                "focus_lost": seen.focus_lost,
                "cursor_left": seen.cursor_left,
                "resizes": seen.resizes,
                "scales": seen.scales,
            });
            report["checklist"] = serde_json::json!(MIXED_MANUAL_CHECKLIST
                .iter()
                .map(|(_, item)| item.to_string())
                .collect::<Vec<_>>());
            report["note"] = serde_json::json!(
                "real OS input evidence only; never an automatic pass; N1..N6 conclusions and observer stay null until a human fills them"
            );
            std::fs::create_dir_all(&self.report).ok();
            std::fs::write(
                self.report.join("manual-native-gate.json"),
                serde_json::to_string_pretty(&report).unwrap(),
            )
            .expect("manual report");
            eprintln!("MIXED-REGIONS MANUAL GATE report: {report}");
            e.exit();
            return;
        }
        let state = self.host.as_mut().map(|h| {
            (
                h.app().ui.texts(0).to_vec(),
                h.active_keys(),
                h.app().ui.callbacks,
            )
        });
        let report = match state {
            Some((texts, active, callbacks)) => {
                let count = texts.get(0).cloned().unwrap_or_default();
                let (p1x, p2x, m3tx) = self.scene_values();
                let mark = |k: &str| self.marks.iter().find(|(n, _)| n == k).map(|(_, v)| *v);
                let idle_ok =
                    mark("idle_start").is_some() && mark("idle_start") == mark("idle_end");
                let toggled = self.epochs_before_toggle != self.epochs_after_remount
                    && self.epochs_after_remount.iter().all(|e| e.is_some());
                self.passed = !self.probe
                    || (self.failure.is_none()
                        && count == "202"
                        && p1x == 24.0
                        && p2x == 0.0
                        && m3tx >= 0.0
                        && active.len() == 4
                        && active.iter().any(|k| k == P1)
                        && active.iter().any(|k| k == STRIP)
                        && toggled
                        && idle_ok);
                serde_json::json!({
                    "title": TITLE,
                    "host_chain": "MixedRegionsHost (PLAN-054 metadata assembly)",
                    "count": count,
                    "post_resize_hit": count == "202",
                    "p1_x": p1x,
                    "p2_x": p2x,
                    "m3_tx": m3tx,
                    "active_regions": active,
                    "epochs_before_toggle": self.epochs_before_toggle,
                    "epochs_after_remount": self.epochs_after_remount,
                    "os_channel": "posted (WM clicks + WM keys direct injection; real native SendInput/focus/DPI-switch run is the manual gate per 048/049 precedent)",
                    "posted_clicks": self.posted_clicks,
                    "posted_keys": self.posted_keys,
                    "scale_factor_observed": self.scale_observed,
                    "logical_sizes": self.logical_sizes,
                    "physical_sizes": self.physical_sizes,
                    "frames": self.frames,
                    "callback_marks": self.marks,
                    "callbacks_total": callbacks,
                    "idle_no_tick": idle_ok,
                    "focus_lost": self.manual_seen.focus_lost,
                    "cursor_left": self.manual_seen.cursor_left,
                    "zero_resizes": self.zero_resizes,
                    "failure": self.failure,
                    "passed": self.passed,
                })
            }
            _ => serde_json::json!({"passed": false, "reason": "missing app"}),
        };
        std::fs::create_dir_all(&self.report).ok();
        std::fs::write(
            self.report.join("mixed-regions-native.json"),
            serde_json::to_string_pretty(&report).unwrap(),
        )
        .expect("report");
        eprintln!("MIXED-REGIONS NATIVE report: {report}");
        e.exit();
    }

    fn run_phase(&mut self, e: &ActiveEventLoop) {
        if !self.probe {
            return;
        }
        if let Some(why) = self.failure.clone() {
            self.marks.push(("failed".into(), 0));
            eprintln!("PROBE FAILURE: {why}");
            self.verify_and_finish(e);
            return;
        }
        if self.kb_target.is_some() && !self.kb_step() {
            return;
        }
        match self.phase {
            0 if self.held(1500) => {
                if let Some(w) = &self.window {
                    self.scale_observed = w.scale_factor() as f32;
                }
                if let Some(h) = self.host.as_mut() {
                    let cb = h.app().ui.callbacks;
                    self.marks.push(("start".into(), cb));
                    self.epochs_before_toggle = [h.region_epoch(P1), h.region_epoch(STRIP)];
                    self.logical_sizes.push(h.logical());
                }
                self.goto(1);
            }
            1 if self.held(700) => {
                self.click(BUMP);
                self.goto(2);
            }
            2 if self.held(700) => {
                self.click(P1);
                self.goto(3);
            }
            3 if self.held(700) => {
                self.strip_before = self.bounds4(STRIP);
                self.click(STRIP); // count 1 → 101
                self.goto(4);
            }
            4 => {
                self.kb_start(TOGGLE);
                self.goto(5);
            }
            5 => {
                // Deterministic gate (F-066-R2-01): the p1 region must really be
                // unmounted (epoch None) AND the strip must have followed the
                // moved layout before the strip click is placed; a blind 700 ms
                // click raced the Enter. (The dash fixture keeps p3/strip/m3
                // mounted, so the visible count is 4 → 3, not 2.)
                let hidden = self
                    .host
                    .as_ref()
                    .is_some_and(|h| h.region_epoch(P1).is_none());
                let followed = match (self.bounds4(STRIP), self.strip_before) {
                    (Some(now), Some(before)) => {
                        (now[1] - before[1]).abs() > 0.5 || (now[3] - before[3]).abs() > 0.5
                    }
                    _ => false,
                };
                if hidden && followed {
                    let cb = self
                        .host
                        .as_mut()
                        .map(|h| h.app().ui.callbacks)
                        .unwrap_or(0);
                    self.marks.push(("hidden".into(), cb));
                    let moved_y = self.bounds4(STRIP).unwrap_or([0.0; 4])[1];
                    self.marks.push(("strip_moved".into(), moved_y as u64));
                    self.click(STRIP); // count 101 → 201（真实布局跟随路由）
                    self.goto(6);
                } else if self.held(4000) {
                    let active = self.active();
                    self.fail(format!(
                        "hide/layout not observed (hidden={hidden}, active={active}, followed={followed})"
                    ));
                }
            }
            6 => {
                self.kb_start(TOGGLE);
                self.goto(7);
            }
            7 => {
                let shown = self
                    .host
                    .as_ref()
                    .is_some_and(|h| h.region_epoch(P1).is_some() && h.active_count() == 4);
                if shown {
                    if let Some(h) = self.host.as_mut() {
                        self.epochs_after_remount = [h.region_epoch(P1), h.region_epoch(STRIP)];
                    }
                    self.click(P1); // 新代次：x 12 → 24
                    self.goto(8);
                } else if self.held(4000) {
                    self.fail("show not observed after the second toggle");
                }
            }
            8 if self.held(700) => {
                // 真实窗 resize：Windows 同步路径 request_inner_size 直接
                // 返回新物理尺寸（无后续 Resized 事件）；异步平台由
                // Resized 事件驱动 host.resize。两条路径同一 host 入口。
                if let (Some(w), Some(h)) = (self.window.as_ref(), self.host.as_mut()) {
                    let req =
                        w.request_inner_size(winit::dpi::LogicalSize::new(RESIZED[0], RESIZED[1]));
                    self.marks
                        .push(("request_returned_sync".into(), req.is_some() as u64));
                    if let Some(phys) = req {
                        let scale = w.scale_factor() as f32;
                        let logical = [
                            phys.width.max(1) as f32 / scale,
                            phys.height.max(1) as f32 / scale,
                        ];
                        {
                            let gpu = h.rt.gpu();
                            if let Some(target) = self.target.as_mut() {
                                let _ = target.handle_resize(
                                    gpu,
                                    phys.width.max(1),
                                    phys.height.max(1),
                                );
                            }
                        }
                        let _ = h.resize(logical);
                    }
                }
                self.goto(9);
            }
            9 => {
                // Gate the post-resize hit on the applied logical size (F-066-
                // R2-01: the 0×0/0.5 event must be refused, not clicked into).
                let applied = self.host.as_ref().is_some_and(|h| {
                    let l = h.logical();
                    (l[0] - RESIZED[0]).abs() < 0.5 && (l[1] - RESIZED[1]).abs() < 0.5
                });
                if applied {
                    if let (Some(w), Some(h)) = (self.window.as_ref(), self.host.as_mut()) {
                        let phys = w.inner_size();
                        self.physical_sizes.push([phys.width, phys.height]);
                        self.logical_sizes.push(h.logical());
                    }
                    // PLAN-066 T-03: the post-resize hit uses the same posted
                    // physical channel (logical layout × scale); bump +1
                    // (201→202) proves the hit follows the new layout.
                    self.click(BUMP);
                    self.goto(10);
                } else if self.held(4000) {
                    self.fail("resize not applied (post-resize hit deferred)");
                }
            }
            10 if self.held(900) => {
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_start".into(), cb));
                self.goto(11);
            }
            11 if self.held(900) => {
                let cb = self
                    .host
                    .as_mut()
                    .map(|h| h.app().ui.callbacks)
                    .unwrap_or(0);
                self.marks.push(("idle_end".into(), cb));
                self.goto(12);
            }
            12 if self.held(300) => {
                self.verify_and_finish(e);
            }
            _ => {}
        }
    }

    fn redraw(&mut self, e: &ActiveEventLoop) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        if let Some(target) = self.target.as_mut() {
            if let Err(err) = host.frame(Some(target)) {
                eprintln!("FRAME ERROR: {err}");
            }
        } else if let Err(err) = host.frame(None) {
            eprintln!("FRAME ERROR: {err}");
        }
        self.frames += 1;
        self.run_phase(e);
    }
}

fn window_scale_of(window: Option<&Arc<Window>>) -> f32 {
    window.map(|w| w.scale_factor() as f32).unwrap_or(1.0)
}

/// F-SCALE-02（PLAN-076）：真实 (before, after) 对取自开窗起跟踪的
/// `last_scale`，并跟新之。无播种时的退化形状（before==after）保持可见，
/// 不冒称真实转换。
fn push_scale_pair(seen: &mut ManualSeen, new_scale: f32) -> (f32, f32) {
    let before = seen.last_scale.replace(new_scale).unwrap_or(new_scale);
    seen.scale_factor_changed.push((before, new_scale));
    (before, new_scale)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let probe_flag = args.iter().any(|a| a == "--probe");
    let manual = args.iter().any(|a| a == "--manual");
    let ui = args
        .iter()
        .position(|a| a == "--ui")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .expect("--ui <ui.json>");
    let world = args
        .iter()
        .position(|a| a == "--world")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .expect("--world <world.at>");
    let report = args
        .iter()
        .position(|a| a == "--report")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("D:/autostack/plan-artifacts/plan-054/auto-ui/t04/reports")
        });
    let mut app = App::new(probe_flag, manual, ui, world, report);
    EventLoop::new().unwrap().run_app(&mut app).unwrap();
    if probe_flag && !app.passed {
        std::process::exit(1);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, e: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            e.create_window(
                Window::default_attributes()
                    .with_title(TITLE)
                    .with_inner_size(winit::dpi::LogicalSize::new(LOGICAL[0], LOGICAL[1])),
            )
            .unwrap(),
        );
        let gpu = GpuDevice::new().unwrap();
        let size = window.inner_size();
        let target =
            WindowTarget::new(&gpu, window.clone(), size.width.max(1), size.height.max(1)).unwrap();
        let _ = &gpu;
        let ui_json = std::fs::read_to_string(&self.ui_path).expect("ui.json");
        let world_at = std::fs::read_to_string(&self.world_path).expect("world.at");
        let app = MixedRegionsUiApp::new(&ui_json, &world_at, "plan054-window").expect("v2 app");
        let fmt = target.format();
        let host = MixedRegionsHost::new(gpu, app, LOGICAL, Some(fmt)).expect("v2 host");
        // F-SCALE-02（PLAN-076）：开窗即播种起始 scale 档（真实 winit 读回）。
        self.manual_seen.last_scale = Some(window.scale_factor() as f32);
        self.window = Some(window);
        self.target = Some(target);
        self.host = Some(host);
    }

    fn window_event(&mut self, e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.verify_and_finish(e),
            WindowEvent::Resized(size) => {
                // PLAN-066 T-03 (F-066-R2-01): a minimized window delivers
                // Resized(0×0). Never turn that into a 0.5×0.5 logical surface;
                // ignore it and let the next real resize re-apply the layout.
                if size.width == 0 || size.height == 0 {
                    self.zero_resizes += 1;
                    self.marks.push(("resize_zero_ignored".into(), 0));
                    return;
                }
                self.marks.push(("resized_event".into(), size.width as u64));
                self.manual_seen.resizes += 1;
                if self.manual {
                    if let Some(w) = &self.window {
                        self.manual_seen.scales.push(w.scale_factor() as f32);
                    }
                    self.manual_checkpoint("resize");
                }
                let (Some(target), Some(host)) = (self.target.as_mut(), self.host.as_mut()) else {
                    return;
                };
                let scale = window_scale_of(self.window.as_ref());
                let logical = [size.width as f32 / scale, size.height as f32 / scale];
                {
                    // 单设备合同：GPU 由 host 独占持有（rt.gpu() 借用面）。
                    let gpu = host.rt.gpu();
                    let _ = target.handle_resize(gpu, size.width.max(1), size.height.max(1));
                }
                match host.resize(logical) {
                    Ok(_) => self
                        .marks
                        .push(("resized_applied".into(), logical[0] as u64)),
                    Err(_) => self.marks.push(("resize_err".into(), 0)),
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_px = [position.x as f32, position.y as f32];
                let scale = window_scale_of(self.window.as_ref());
                let logical = [position.x as f32 / scale, position.y as f32 / scale];
                if let Some(h) = self.host.as_mut() {
                    h.pointer_moved(logical);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if self.cursor_px[0] < 0.0 {
                    return;
                }
                if state == ElementState::Pressed {
                    self.manual_seen.clicks += u32::from(self.manual);
                    self.manual_checkpoint("click");
                }
                let scale = window_scale_of(self.window.as_ref());
                let logical = [self.cursor_px[0] / scale, self.cursor_px[1] / scale];
                if let Some(h) = self.host.as_mut() {
                    if state == ElementState::Pressed {
                        h.pointer_pressed(logical);
                    } else {
                        h.pointer_released();
                    }
                }
            }
            WindowEvent::Focused(false) => {
                // PLAN-066 T-03: a real OS focus loss cancels in-flight capture
                // and invalidates the keyboard focus (no stale Enter target).
                // A probe continues via the adaptive Tab re-acquisition.
                self.manual_seen.focus_lost += 1;
                self.marks.push(("focus_lost".into(), 1));
                self.manual_checkpoint("focus_lost");
                if let Some(h) = self.host.as_mut() {
                    h.window_focus_lost();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                // Pointer leaving cancels the in-flight press/capture only; the
                // keyboard focus is owned by the focus channel above.
                self.manual_seen.cursor_left += 1;
                self.manual_checkpoint("cursor_left");
                if let Some(h) = self.host.as_mut() {
                    h.cancel_all(agp::core::composition::input::CancelReason::PointerLeftWindow);
                    self.marks.push(("cursor_left".into(), 1));
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // PLAN-071 T-04: record the REAL OS scale change (winit actual
                // before/after). F-SCALE-02（PLAN-076）: `before` 取自开窗起
                // 跟踪的 last_scale——事件到达时 winit 已换新值，晚读
                // `scale_factor()` 必产 (X, X)。Read-only: the following
                // Resized event applies the geometry through the existing
                // host path.
                if self.manual {
                    push_scale_pair(&mut self.manual_seen, scale_factor as f32);
                    self.manual_checkpoint("scale_factor_changed");
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let key = match &event.logical_key {
                        WKey::Named(WNamed::Tab) => Some(PlatformKey::Tab),
                        WKey::Named(WNamed::Enter) => Some(PlatformKey::Enter),
                        _ => None,
                    };
                    if let Some(k) = key {
                        if self.manual {
                            match k {
                                PlatformKey::Tab => self.manual_seen.tabs += 1,
                                PlatformKey::Enter => self.manual_seen.enters += 1,
                            }
                            let label = match k {
                                PlatformKey::Tab => "tab",
                                PlatformKey::Enter => "enter",
                            };
                            self.manual_checkpoint(label);
                        }
                        if let Some(h) = self.host.as_mut() {
                            h.key(k);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => self.redraw(e),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        // Poll：探针窗需要确定性泵送（沿 049 实测——WaitUntil 在节流桌面
        // 上休眠丢相位）。
        e.set_control_flow(ControlFlow::Poll);
    }
}

/// F-SCALE-02（PLAN-076）scale 会话跟踪单元：真窗事件在 CI 不可重放，
/// 跟踪逻辑以纯函数面单测钉住（开窗播种 → 真实对 → 关窗读回由
/// `m1_native_evidence_support` 会话形测试覆盖）。
#[cfg(test)]
mod scale_track_tests {
    use super::{push_scale_pair, ManualSeen};

    #[test]
    fn scale_pairs_come_from_tracked_open_scale_not_late_readback() {
        // 会话：开窗 2.0 → 切 2.5 → 切 1.25。旧实现（事件内晚读
        // `scale_factor()`）产 (2.5, 2.5)/(1.25, 1.25)——本用例对旧实现必红。
        let mut seen = ManualSeen::default();
        seen.last_scale = Some(2.0); // resumed() 播种
        assert_eq!(push_scale_pair(&mut seen, 2.5), (2.0, 2.5));
        assert_eq!(push_scale_pair(&mut seen, 1.25), (2.5, 1.25));
        assert_eq!(seen.scale_factor_changed, vec![(2.0, 2.5), (2.5, 1.25)]);
        assert_eq!(seen.last_scale, Some(1.25), "tracker follows the last real factor");
    }

    #[test]
    fn unseeded_tracker_degenerates_visibly_to_same_factor_pair() {
        // 无播种（理论上不可达：resumed() 先于事件）；退化形状保持可见，
        // 不冒称真实转换。
        let mut seen = ManualSeen::default();
        assert_eq!(push_scale_pair(&mut seen, 2.0), (2.0, 2.0));
        assert_eq!(seen.last_scale, Some(2.0));
    }
}
