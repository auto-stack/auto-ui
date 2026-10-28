//! Probe report model. Plain serde data — no wgpu types.

use crate::renderer::error::RendererError;
use serde::Serialize;
use std::path::Path;

/// One captured keyframe (PNG file + its sha256).
#[derive(Debug, Clone, Serialize)]
pub struct Keyframe {
    pub name: String,
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
}

/// One failure-injection outcome: the failure MUST be rejected with the
/// expected classification (and, where applicable, last-good state kept).
#[derive(Debug, Clone, Serialize)]
pub struct FailureInjection {
    pub name: String,
    pub expected_classification: String,
    pub observed_classification: String,
    /// True when the operation returned the classified error (not a panic,
    /// not silent success, not a wrong classification).
    pub rejected_as_expected: bool,
    pub last_good_preserved: bool,
}

/// gpu-foundation (V-01) report body.
#[derive(Debug, Clone, Serialize)]
pub struct GpuFoundationReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub backend: String,
    pub device_token: String,
    /// Token observed from EACH target — V-01 requires them identical.
    pub target_tokens: Vec<String>,
    pub window_created: bool,
    pub window_mode: String,
    pub window: Option<WindowFacts>,
    pub frames: FrameFacts,
    pub keyframes: Vec<Keyframe>,
    pub lifecycle: LifecycleFacts,
    pub resources: ResourceFacts,
    pub failure_injection: Vec<FailureInjection>,
    /// GPU timestamp queries are not wired in T-02 — never fabricate timing.
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowFacts {
    pub initial_inner_size: [u32; 2],
    pub scale_factor: f64,
    pub dpi_observations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameFacts {
    pub window_frames_presented: u64,
    pub offscreen_frames_rendered: u64,
    pub offscreen_bytes_copied_for_keyframes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleFacts {
    pub resize_events_observed: Vec<[u32; 2]>,
    pub minimized_size_observed: Option<[u32; 2]>,
    pub restore_size_observed: Option<[u32; 2]>,
    pub zero_size_presentation_suspended: bool,
    pub device_lost_seen: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceFacts {
    pub cache_capacity: usize,
    pub stats_before: crate::renderer::CacheStats,
    pub stats_after: crate::renderer::CacheStats,
    pub live_entries: usize,
    /// True when untouched cache entries survived the whole run without
    /// recreation (no per-frame texture rebuild).
    pub unmodified_survived: bool,
}

/// Top-level probe report written to `<report_dir>/report.json`.
/// `body` is case-specific; every case owns its typed body and serializes it
/// here (cases never share a body schema).
#[derive(Debug, Clone, Serialize)]
pub struct ProbeReport {
    pub case: String,
    pub replay: String,
    pub utc: String,
    pub platform: PlatformFacts,
    pub agp_version: String,
    /// Evidence identity (review r1 F08): commit, lock hash, asset-set hash.
    pub fingerprints: ReportFingerprints,
    pub exit: String,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportFingerprints {
    pub agp_commit: String,
    pub cargo_lock_sha256: String,
    pub asset_set_sha256: String,
}

fn sha256_file(path: &std::path::Path) -> String {
    match std::fs::read(path) {
        Ok(bytes) => {
            use sha2::Digest;
            let mut h = sha2::Sha256::new();
            h.update(&bytes);
            format!("{:x}", h.finalize())
        }
        Err(_) => "unavailable".to_string(),
    }
}

impl ReportFingerprints {
    pub fn collect() -> Self {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // asset-set hash: the recorded sha256 manifests themselves (stable,
        // cheap, and exactly what T-01 froze).
        let mut asset_input = Vec::new();
        for f in [
            "assets/SHA256SUMS",
            "testdata/SHA256SUMS",
            "testdata/breakout/FONTS.sha256",
        ] {
            asset_input.extend_from_slice(&std::fs::read(manifest.join(f)).unwrap_or_default());
        }
        use sha2::Digest;
        let mut h = sha2::Sha256::new();
        h.update(&asset_input);
        ReportFingerprints {
            agp_commit: env!("AGP_GIT_COMMIT").to_string(),
            cargo_lock_sha256: sha256_file(&manifest.join("Cargo.lock")),
            asset_set_sha256: format!("{:x}", h.finalize()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformFacts {
    pub os: String,
    pub arch: String,
    pub profile: String,
}

pub fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix-{secs}")
}

pub fn write_report(
    report_dir: &Path,
    case: &str,
    replay: &str,
    exit: &str,
    body: impl serde::Serialize,
) -> Result<(), RendererError> {
    std::fs::create_dir_all(report_dir)
        .map_err(|e| RendererError::ReportIo(format!("mkdir {}: {e}", report_dir.display())))?;
    let body = serde_json::to_value(&body)
        .map_err(|e| RendererError::ReportIo(format!("json body: {e}")))?;
    let report = ProbeReport {
        case: case.to_string(),
        replay: replay.to_string(),
        utc: utc_now(),
        platform: PlatformFacts {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            profile: if cfg!(debug_assertions) {
                "debug".into()
            } else {
                "release".into()
            },
        },
        agp_version: env!("CARGO_PKG_VERSION").to_string(),
        fingerprints: ReportFingerprints::collect(),
        exit: exit.to_string(),
        body,
    };
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| RendererError::ReportIo(format!("json: {e}")))?;
    let path = report_dir.join("report.json");
    std::fs::write(&path, json)
        .map_err(|e| RendererError::ReportIo(format!("write {}: {e}", path.display())))?;
    Ok(())
}

/// shared-runtime (V-02) report body.
#[derive(Debug, Clone, Serialize)]
pub struct SharedRuntimeReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    pub ticks: u64,
    pub snapshot_hash_a: String,
    pub snapshot_hash_b: String,
    pub snapshots_agree: bool,
    pub events_equal: bool,
    pub event_count: usize,
    pub changelog_replay_matches: bool,
    pub changelog_replay_hash: String,
    pub reset_replay_matches: bool,
    pub stale_id_rejections: u32,
    pub late_event_rejected: bool,
    pub cache_created: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_stale_rejected: u64,
    pub untouched_resource_rebuilds: bool,
    pub rng_reproducible: bool,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

/// One host-extension reset round (V-03): rebuild + replay prefix must match.
#[derive(Debug, Clone, Serialize)]
pub struct BreakoutResetEvidence {
    pub index: u32,
    pub ticks: u64,
    pub snapshot_hash: String,
    pub objects: usize,
    pub trace_matches_prefix: bool,
    pub events_all_new_session: bool,
    pub cache_created: u64,
}

/// ag2d-breakout (V-03) report body.
#[derive(Debug, Clone, Serialize)]
pub struct BreakoutReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    pub ticks: u64,
    pub score_final: u32,
    pub bricks_destroyed: u32,
    pub trace_match: bool,
    pub max_pos_diff: f32,
    pub pos_mismatch_ticks: u64,
    pub score_mismatch_ticks: u64,
    pub contact_mismatch_ticks: u64,
    pub contact_sequence_match: bool,
    pub contacts_total: u64,
    pub sounds_triggered: u64,
    pub kira_plays_ok: u64,
    pub kira_play_errors: u64,
    pub sound_count_equals_contacts: bool,
    pub audio_backend: String,
    pub audio_asset: String,
    pub audio_asset_sha256: String,
    pub hud_text_instances: usize,
    pub hud_font: String,
    pub hud_font_size: f32,
    pub keyframes: Vec<Keyframe>,
    pub resets: Vec<BreakoutResetEvidence>,
    pub cache_stats: crate::renderer::CacheStats,
    pub live_mode_available: bool,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

/// One loaded-model fact sheet (ag3d-risk-slice).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssetFact {
    pub file: String,
    pub sha256: String,
    pub meshes: Vec<AssetMeshFact>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssetMeshFact {
    pub name: String,
    pub vertices: usize,
    pub indices: usize,
    pub material: String,
    pub base_color: [f32; 4],
}

/// ag3d-risk-slice report body (T-05).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RiskSliceReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    pub assets: Vec<AssetFact>,
    pub uploaded_meshes: usize,
    pub draw_calls_per_frame: u32,
    pub instances_per_frame: u32,
    pub camera: String,
    pub environment_clear: [f32; 4],
    pub animation: serde_json::Value,
    pub keyframes: Vec<Keyframe>,
    pub shadow_on_off_hashes_distinct: bool,
    pub collision_queries: Vec<CollisionQueryResult>,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CollisionQueryResult {
    pub name: String,
    pub outcome: String,
}

/// ag3d-squash (V-04) report body (T-06).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SquashReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    pub trace: SquashTraceFacts,
    pub spawn_stream_matches_golden: bool,
    pub music: SquashMusicFacts,
    pub keyframes: Vec<Keyframe>,
    pub retry: SquashRetryFacts,
    pub cache_uploaded_meshes: usize,
    pub gpu_timing_available: bool,
    pub live_mode_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SquashTraceFacts {
    pub ticks: u64,
    pub max_pos_diff: f32,
    pub pos_mismatch_ticks: u64,
    pub score_mismatch_ticks: u64,
    pub mobs_alive_mismatch_ticks: u64,
    pub alive_mismatch_ticks: u64,
    pub event_count: usize,
    pub golden_event_count: usize,
    pub event_sequence_exact: bool,
    pub squashes: u32,
    pub death_tick: Option<u64>,
    pub spawns: u32,
    pub score_final: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SquashMusicFacts {
    pub asset: String,
    pub asset_sha256: String,
    pub backend: String,
    pub autoplay_on_start: bool,
    pub playback_state: String,
    pub looped: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SquashRetryFacts {
    pub reset_rounds: u32,
    pub prefix_ticks: u64,
    pub snapshots_equal: bool,
    pub new_session_each_round: bool,
}

/// lifecycle (V-05) report body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LifecycleReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub mode: String,
    pub restarts: Vec<RestartFacts>,
    pub view_add_remove: ViewFacts,
    pub failure_injection: Vec<FailureInjectionFacts>,
    pub close_during_load: CloseDuringLoadFacts,
    pub thread_observation: ThreadFacts,
    pub teardown: TeardownFacts,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RestartFacts {
    pub game: String,
    pub rounds: u32,
    pub sim_ticks_per_round: u64,
    pub scene_objects_baseline: usize,
    pub scene_objects_after_each_round_equal: bool,
    pub event_session_violations: u64,
    /// GPU-side facts with the REAL game resources integrated (r1 F06):
    /// mesh uploads over all rounds (must be flat after scene build),
    /// per-session cache creations within rounds (must be 0 after warm),
    /// and the real audio stream state across restarts.
    pub mesh_uploads_total: u64,
    pub mesh_uploads_flat_after_build: bool,
    pub cache_creations_within_rounds: u64,
    pub audio_playing_after_rounds: bool,
    pub wall_ms_per_round: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewFacts {
    pub cycles: u32,
    /// Derived from owned lifetimes via DropWitness (r1 F06): creation and
    /// drop are observed, not hand-counted — a leaked (forgotten) target
    /// keeps the witness alive and fails the gate.
    pub targets_created: u32,
    pub witness_alive_during_cycles: usize,
    pub witness_alive_end: usize,
    pub persistent_targets: usize,
    pub baseline_restored: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FailureInjectionFacts {
    pub name: String,
    pub classification: String,
    pub rejected: bool,
    pub last_good_frame_preserved: bool,
    pub last_good_hash: String,
    pub after_failure_hash: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CloseDuringLoadFacts {
    /// A map was issued, the buffer dropped in flight, and the callback
    /// fired with an ABORT acknowledgement after a poll (r1 F06: the
    /// outcome is awaited and observed, not assumed).
    pub cancellation_acknowledged: bool,
    pub callback_outcome: String,
    pub success_path_callback_ok: bool,
    pub method: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreadFacts {
    pub method: String,
    /// Samples taken around the ACTUAL workload and teardown (r1 F06):
    /// [before, after-workload, after-teardown].
    pub samples: Vec<i64>,
    pub rss_samples_mb: Vec<f64>,
    pub owned_worker_threads_created_by_agp: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TeardownFacts {
    /// Set only after device/pipeline/target/audio destruction completed
    /// without panic (r1 F06); part of the pass predicate.
    pub clean_exit_after_teardown: bool,
    pub gpu_device_destroyed: bool,
}

/// metrics (V-07) report body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub mode: String,
    pub runs: Vec<MetricsRun>,
    pub gpu_timing_available: bool,
    pub methodology: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsRun {
    pub game: String,
    /// cold = first pass in this process; warm = second pass, everything
    /// already resident (pipelines/views are built ONCE, before the passes).
    pub pass: String,
    pub frames: u32,
    pub cpu_frame_ms_p50: f64,
    pub cpu_frame_ms_p95: f64,
    pub cpu_frame_ms_max: f64,
    /// New GPU resource OBJECT creations per frame (instrumented deltas).
    pub creations_per_frame_p50: f64,
    pub creations_per_frame_max: u64,
    /// Bytes REWRITTEN into existing buffers per frame (instrumented at the
    /// actual write paths — camera/light/instance/instance-quad uploads).
    pub rewrite_bytes_per_frame_p50: f64,
    pub rewrite_bytes_per_frame_max: u64,
    /// Instrumented totals for this run (cross-checked against the summary;
    /// a sabotaged counter makes the totals disagree → the run fails).
    pub instrumented_total_rewrites: u64,
    pub instrumented_total_bytes: u64,
    /// Raw per-frame series persisted next to the report (recomputable
    /// quantiles).
    pub raw_series_file: String,
    pub rss_start_mb: f64,
    pub rss_end_mb: f64,
    pub startup_wall_ms: f64,
    pub exit_wall_ms: f64,
    pub views: u32,
    pub observations_complete: bool,
}

/// verifier-negative-controls (V-08) report body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NegativeReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub mode: String,
    pub gates: Vec<GateResult>,
    pub all_gates_behaved: bool,
    pub any_gate_skipped_or_unexecuted: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GateResult {
    pub gate: String,
    /// true = the legitimate positive control passed.
    pub positive_passed: bool,
    /// true = the injected counterexample was CAUGHT (its gate fired).
    pub negative_fired: bool,
    pub reason: String,
}

/// iced-shared-device (V-06) report body (T-08).
#[derive(Debug, Clone, serde::Serialize)]
pub struct IcedReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    pub dpi_scale_factor: Option<f32>,
    pub window_size_initial: Option<[f32; 2]>,
    pub resized_sizes: Vec<[f32; 2]>,
    pub click: IcedClickFacts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub views_on_iced_device: Option<bool>,
    pub view2d_rendered_on_iced_device: bool,
    pub view3d_rendered_on_iced_device: bool,
    pub keyframes: Vec<Keyframe>,
    pub gates: Vec<crate::host::report::GateResult>,
    /// Final-composite evidence (r1 F05): assertions against the real window
    /// output (both views + controls + clip + resize).
    pub composite: serde_json::Value,
    pub all_gates_behaved: bool,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IcedClickFacts {
    pub dispatched: bool,
    pub observed_via_real_event_path: bool,
    pub presses: u32,
    /// "os_focus" = real foreground OS click; "posted_queue" = WM_LBUTTON
    /// posted to the window's message queue (synthetic delivery through
    /// winit's real input pipeline; used when the session denies focus).
    pub click_path: String,
}

/// independent-consumer (V-09) report body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsumerReport {
    pub closure: ConsumerClosureFacts,
    pub locked_build: ConsumerCommandFacts,
    pub relocation: ConsumerRelocationFacts,
    pub collateral: ConsumerCollateralFacts,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

/// surface-offscreen (PLAN-830 V-02) report body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SurfaceReport {
    pub adapter: crate::renderer::AdapterInfoReport,
    pub device_token: String,
    pub window_created: bool,
    pub window_mode: String,
    /// "os_window_events" (real window input path) or "synthetic_headless"
    /// (fallback: events fed at mapped coords, no OS input claimed).
    pub input_evidence: String,
    pub inner_ui_pressed: u32,
    pub inner_input_text: String,
    pub clicks_mapped_to_plane: u32,
    pub keyboard_events_delivered: u32,
    pub plane_passes_drawn: u64,
    pub ui_frames_presented: u64,
    /// GPU→CPU readbacks (acceptance captures ONLY; steady state has none).
    pub readbacks: u64,
    pub keyframes: Vec<Keyframe>,
    pub gates: Vec<GateResult>,
    pub all_gates_behaved: bool,
    pub button_bounds: Option<[f32; 4]>,
    pub input_bounds: Option<[f32; 4]>,
    pub failures: Vec<String>,
    pub gpu_timing_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsumerClosureFacts {
    pub path_dependencies: Vec<String>,
    pub violations: Vec<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsumerCommandFacts {
    pub command: String,
    pub cwd: String,
    pub success: bool,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsumerRelocationFacts {
    pub src: String,
    pub dst: String,
    pub build: ConsumerCommandFacts,
    pub case_command: String,
    pub case_success: bool,
    pub machine_specific_fields: Vec<String>,
    pub compared_fields: Vec<String>,
    pub mismatches: Vec<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsumerCollateralFacts {
    pub base_commit: String,
    pub root_diff_outside_agp: Vec<String>,
    /// Plan-owned provisional Spec drafts (declared delta, not production
    /// code). Recorded explicitly instead of silently dropping them.
    pub provisional_spec_drafts: Vec<String>,
    pub reparse_points_in_agp: Vec<String>,
    pub core_layering_violations: Vec<String>,
    pub ok: bool,
}
