//! PLAN-061 T-04 repair probes (F-R1/F-R2/F-R3)，test-only。
//!
//! - F-R1：两表面混合负载（轻文本 App + 重上传/重绘 App）的公平/隔离原型，
//!   含“故意独占”负控（饥饿必须可检出）。
//! - F-R2：M-05 内存原型——同进程 direct 与 RQ 重放路径的 private/working-set。
//! - F-R3：P-01 槽账/背压模型——冻结的 credit 策略不覆盖未 ACK 槽；当前
//!   endpoint 回落 `1-last_slot` 作为负控（覆盖必须可检出）。
//!
//! 边界：全部 inprocx 代理，不是两 Host 实测；不宣称生产公平/内存已通过。

use std::path::PathBuf;
use std::time::Instant;

use super::rq_probe::{direct_frames, ms, out_dir, stats, theme};
use super::*;
use crate::ui::component::Component;
use crate::ui::view::View;

const W: f32 = 640.0;
const H: f32 = 400.0;

#[derive(Debug, Clone)]
struct RepairComponent {
    view: View<()>,
}

impl Component for RepairComponent {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

fn produce(view: View<()>) -> (super::super::message::DisplayList, f64) {
    let t = Instant::now();
    let mut surface = HeadlessSurface::new(RepairComponent { view }, W, H);
    let layers = surface.drive_and_draw();
    let mut obs = Vec::new();
    let mut bms = Vec::new();
    let mut cache = ResourceCache::default();
    let list = lower_layers_v2(&layers, &mut obs, &mut cache, &mut bms);
    (list, ms(t.elapsed()))
}

/// 每 App 一个重放设备（warm）：首个帧付设备初始化，之后只测 draw+读回。
struct ReplayCell {
    sim: Option<iced_test::simulator::Simulator<'static, (), iced::Theme, iced::Renderer>>,
    startup_ms: f64,
}

impl ReplayCell {
    fn new() -> Self {
        Self {
            sim: None,
            startup_ms: 0.0,
        }
    }

    fn cost(&mut self, list: &super::super::message::DisplayList) -> f64 {
        if self.sim.is_none() {
            let mut neutral = list.clone();
            neutral.clear = None;
            let leaked: &'static super::super::message::DisplayList =
                Box::leak(Box::new(neutral));
            let t = Instant::now();
            self.sim = Some(iced_test::simulator::Simulator::with_size(
                iced::Settings::default(),
                iced::Size::new(W, H),
                crate::ui::iced::broker_surface::displaylist_element::<()>(leaked),
            ));
            self.startup_ms = ms(t.elapsed());
        }
        let t = Instant::now();
        let _ = self.sim.as_mut().unwrap().snapshot(&theme());
        ms(t.elapsed())
    }
}

// ---------------------------------------------------------------------------
// F-R1：公平/隔离（轻 App vs 重 App）
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Policy {
    /// 两个 App 交替服务（冻结的交互优先近似）。
    RoundRobin,
    /// 故意独占：连续服务重 App 5 拍再服务轻 App（饥饿负控）。
    HeavyHog,
}

struct FairResult {
    light_produce_ms: f64,
    heavy_produce_ms: f64,
    light_replay_ms: f64,
    heavy_replay_ms: f64,
    light_max_gap: usize,
    heavy_max_gap: usize,
    light_max_pending: usize,
    heavy_max_pending: usize,
    light_share: f64,
}

fn run_policy(policy: Policy, iterations: usize) -> FairResult {
    let light: View<()> = View::text("轻 App：中文状态文本");
    // 重 App：代码编辑器（每帧位图上传 + 多文本 run）
    let heavy: View<()> = View::code_editor("fr1-heavy")
        .value("fn main() {\n    println!(\"你好\");\n}")
        .lang("rust")
        .build();

    let mut light_replay = ReplayCell::new();
    let mut heavy_replay = ReplayCell::new();

    let mut light_pending = 0usize;
    let mut heavy_pending = 0usize;
    let mut since_light = 0usize; // 距上次服务轻 App 的迭代数
    let mut since_heavy = 0usize;
    let mut light_max_gap = 0usize;
    let mut heavy_max_gap = 0usize;
    let mut light_max_pending = 0usize;
    let mut heavy_max_pending = 0usize;
    let mut lp = 0.0;
    let mut hp = 0.0;
    let mut lr = 0.0;
    let mut hr = 0.0;

    for i in 0..iterations {
        let serve_light = match policy {
            Policy::RoundRobin => i % 2 == 0,
            Policy::HeavyHog => i % 6 == 0,
        };
        if !serve_light {
            light_pending += 1;
            since_light += 1;
        }
        if serve_light {
            heavy_pending += 1;
            since_heavy += 1;
        }
        light_max_pending = light_max_pending.max(light_pending);
        heavy_max_pending = heavy_max_pending.max(heavy_pending);

        if serve_light {
            light_max_gap = light_max_gap.max(since_light);
            since_light = 0;
            let (list, p) = produce(light.clone());
            lp += p;
            lr += light_replay.cost(&list);
            light_pending = 0;
        } else {
            heavy_max_gap = heavy_max_gap.max(since_heavy);
            since_heavy = 0;
            let (list, p) = produce(heavy.clone());
            hp += p;
            hr += heavy_replay.cost(&list);
            heavy_pending = 0;
        }
    }

    let total = lp + hp + lr + hr;
    FairResult {
        light_produce_ms: lp,
        heavy_produce_ms: hp,
        light_replay_ms: lr,
        heavy_replay_ms: hr,
        light_max_gap,
        heavy_max_gap,
        light_max_pending,
        heavy_max_pending,
        light_share: if total > 0.0 {
            (lp + lr) / total
        } else {
            0.0
        },
    }
}

#[test]
fn rq_repair_fr1_multi_app_fairness_and_starvation_negative_control() {
    let iterations = 12usize;
    let rr = run_policy(Policy::RoundRobin, iterations);
    let hog = run_policy(Policy::HeavyHog, iterations);
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr1_fairness",
            "iterations": iterations,
            "round_robin": {
                "light_max_gap": rr.light_max_gap, "heavy_max_gap": rr.heavy_max_gap,
                "light_max_pending": rr.light_max_pending, "heavy_max_pending": rr.heavy_max_pending,
                "light_share": rr.light_share,
                "light_produce_ms": rr.light_produce_ms, "heavy_produce_ms": rr.heavy_produce_ms,
                "light_replay_ms": rr.light_replay_ms, "heavy_replay_ms": rr.heavy_replay_ms,
            },
            "heavy_hog": {
                "light_max_gap": hog.light_max_gap, "heavy_max_gap": hog.heavy_max_gap,
                "light_max_pending": hog.light_max_pending, "heavy_max_pending": hog.heavy_max_pending,
                "light_share": hog.light_share,
                "light_produce_ms": hog.light_produce_ms, "heavy_produce_ms": hog.heavy_produce_ms,
            },
            "note": "inproc 代理；不是两 Host 实测，不宣称生产公平已通过",
        })
    );
    // 正常策略：轻 App 不被饿（gap ≤1 拍）。
    assert!(
        rr.light_max_gap <= 1,
        "round-robin 下轻 App 不应被连续饿 >1 拍（实际 {}）",
        rr.light_max_gap
    );
    // 负控：独占策略必须能检出轻 App 饥饿（≥5 拍）。
    assert!(
        hog.light_max_gap >= 5,
        "独占负控必须检出轻 App 饥饿（实际 {}）",
        hog.light_max_gap
    );
    // 队列积压（pending）应区分：负控下轻 App 在队帧数显著更高。
    assert!(
        hog.light_max_pending > rr.light_max_pending,
        "独占负控下轻 App 在队帧应更高（rr={} hog={}）",
        rr.light_max_pending,
        hog.light_max_pending
    );
}

// ---------------------------------------------------------------------------
// F-R2：M-05 内存原型（direct vs RQ 重放，同进程采样）
// ---------------------------------------------------------------------------

fn sample_mem(tag: &str) -> Option<(u64, u64)> {
    match super::super::stage3::sample_process_memory(std::process::id()) {
        Ok(s) => {
            println!(
                "[RQREPAIR] {}",
                serde_json::json!({ "tag": "fr2_mem", "point": tag,
                    "private_bytes": s.private_bytes, "working_set": s.working_set })
            );
            Some((s.private_bytes, s.working_set))
        }
        Err(e) => {
            println!(
                "[RQREPAIR] {}",
                serde_json::json!({ "tag": "fr2_mem_unavailable", "point": tag, "error": e })
            );
            None
        }
    }
}

#[test]
fn rq_repair_fr2_memory_prototype_direct_vs_rq() {
    let baseline = sample_mem("baseline");
    assert!(baseline.is_some(), "本机应可采样进程内存（windows）");
    let (p0, w0) = baseline.unwrap();

    // direct 路径：持有一个 Simulator 连续渲染 20 帧
    let direct_view: View<()> = View::text("内存原型 direct 文本");
    let (startup, frames) = {
        let mut sim = iced_test::simulator::Simulator::with_size(
            iced::Settings::default(),
            iced::Size::new(W, H),
            direct_view.clone().into_iced(),
        );
        let _ = sim.snapshot(&theme());
        let t = Instant::now();
        for _ in 0..10 {
            let _ = sim.snapshot(&theme());
        }
        let _frames = ms(t.elapsed());
        let _startup = 0.0;
        std::hint::black_box(&sim);
        (_startup, _frames)
    };
    let after_direct = sample_mem("after_direct_10_frames");

    // RQ 路径：产帧+重放 20 次
    let rq_view: View<()> = View::text("内存原型 RQ 文本");
    let t = Instant::now();
    for _ in 0..10 {
        let (list, _) = produce(rq_view.clone());
        let mut cell = ReplayCell::new();
        let _ = cell.cost(&list);
    }
    let rq_ms = ms(t.elapsed());
    let after_rq = sample_mem("after_rq_10_frames");

    let (p1, w1) = after_direct.unwrap_or((p0, w0));
    let (p2, w2) = after_rq.unwrap_or((p1, w1));
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr2_summary",
            "direct_frame_ms": frames,
            "rq_10_frames_ms": rq_ms,
            "private_delta_direct": p1.saturating_sub(p0),
            "private_delta_rq": p2.saturating_sub(p1),
            "working_set_delta_direct": w1.saturating_sub(w0),
            "working_set_delta_rq": w2.saturating_sub(w1),
            "note": "进程级采样含分配器噪声与共享页；不代表 GPU 驻留或物理上屏",
        })
    );
    // 原型存在性：两路径都应有可测结果；不做阈值断言（研究期）。
    assert!(rq_ms > 0.0);
    let _ = startup;
}

// ---------------------------------------------------------------------------
// F-R3：P-01 槽账/背压模型（credit 不覆盖未 ACK 槽；当前回落为负控）
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct SlotModel {
    slots: [Option<u32>; 2],
    free: Vec<u8>,
    last: u8,
    produced: u32,
    overwrites: u32,
    deferred: u32,
}

impl SlotModel {
    fn new() -> Self {
        Self {
            slots: [None, None],
            free: vec![1], // 与 endpoint.rs:510 同：槽 0 视为在写首帧
            last: 0,
            produced: 0,
            overwrites: 0,
            deferred: 0,
        }
    }

    /// 当前生产路径（endpoint.rs:323）：无 free 时回落 `1-last_slot`。
    fn produce_current_policy(&mut self) -> u32 {
        let slot = match self.free.pop() {
            Some(s) => s,
            None => {
                let s = 1 - self.last;
                if self.slots[s as usize].is_some() {
                    self.overwrites += 1;
                }
                s
            }
        };
        self.last = slot;
        self.produced += 1;
        self.slots[slot as usize] = Some(self.produced);
        self.produced
    }

    /// T-03 冻结策略：只有 free 槽才产帧，否则延后/合并（不覆盖未 ACK）。
    fn produce_credit_policy(&mut self) -> Option<u32> {
        let slot = self.free.pop()?;
        self.last = slot;
        self.produced += 1;
        self.slots[slot as usize] = Some(self.produced);
        Some(self.produced)
    }

    /// Host 归还槽（ACK 传输槽）。
    fn ack(&mut self, slot: u8) {
        self.slots[slot as usize] = None;
        self.free.push(slot);
    }
}

#[test]
fn rq_repair_fr3_slot_credit_model_and_overwrite_negative_control() {
    // 1) 当前策略：延迟 ACK 下会覆盖未 ACK 槽（负控必须检出）。
    let mut current = SlotModel::new();
    for _ in 0..5 {
        current.produce_current_policy();
    }
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr3_current_policy", "produced": current.produced,
            "overwrites": current.overwrites, "slots": current.slots,
            "note": "endpoint.rs 无 free 槽回落 1-last_slot → 覆盖未 ACK 槽可检出",
        })
    );
    assert!(
        current.overwrites >= 1,
        "当前回落策略在无 ACK 时必须产生覆盖（负控可检出）"
    );

    // 2) 冻结 credit 策略：无 ACK 时不覆盖，只延后；ACK 后恢复。
    let mut credit = SlotModel::new();
    let f0 = credit.produce_credit_policy();
    assert!(f0.is_some(), "首帧应占 free 槽");
    let f1 = credit.produce_credit_policy();
    assert!(f1.is_none(), "无 free 槽时必须延后（不覆盖）");
    credit.deferred += 1;
    let f2 = credit.produce_credit_policy();
    assert!(f2.is_none(), "持续无 ACK 必须持续延后");
    credit.deferred += 1;
    assert_eq!(credit.overwrites, 0, "credit 策略不得覆盖未 ACK 槽");
    // 模拟 Host 归还旧 front（compose_v2 语义）
    credit.ack(0);
    let f3 = credit.produce_credit_policy();
    assert!(f3.is_some(), "ACK 归还后应恢复产帧");
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr3_credit_policy", "produced": credit.produced,
            "deferred": credit.deferred, "overwrites": credit.overwrites,
            "note": "冻结 credit 策略：不覆盖未 ACK 槽，ACK 后恢复；实现归 UI062/UI063",
        })
    );

    // 3) 场景二：延迟 ACK 下 credit 有界（在途 ≤2 槽），ACK 后继续。
    let mut pipeline = SlotModel::new();
    let mut in_flight = 0usize;
    let mut sent = 0usize;
    let mut max_in_flight = 0usize;
    for i in 0..10 {
        if pipeline.produce_credit_policy().is_some() {
            sent += 1;
            in_flight += 1;
        } else {
            pipeline.deferred += 1;
        }
        max_in_flight = max_in_flight.max(in_flight);
        assert!(in_flight <= 2, "在途帧超过 2 槽上限");
        // 模拟 Host 在延迟后归还一个旧槽（每 2 拍归还一次）
        if i % 2 == 1 && in_flight > 0 {
            let slot = 1 - pipeline.last;
            pipeline.ack(slot);
            in_flight = in_flight.saturating_sub(1);
        }
    }
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr3_bounded_pipeline", "sent": sent, "deferred": pipeline.deferred,
            "max_in_flight": max_in_flight,
            "note": "延迟 ACK 下在途占用 ≤2 槽（与 endpoint slot_count=2 一致）",
        })
    );
}

// ---------------------------------------------------------------------------
// F-R4：资源身份/generation（V-06/P-02）——稳定帧零重传模型 + 当前路径负控
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ResourceKind {
    Bitmap,
    Font,
    Svg,
    Mesh,
    Media,
}

/// 内容身份 + generation 注册表（T-03 §3 冻结方案的探针模型）。
struct ResourceRegistry {
    entries: std::collections::HashMap<(ResourceKind, [u8; 32]), u64>,
    next_gen: u64,
}

impl ResourceRegistry {
    fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            next_gen: 0,
        }
    }

    /// Some(generation) = 需要上传（首次/内容变更/驱逐后）；None = 稳定命中（零上传）。
    fn observe(&mut self, kind: ResourceKind, key: [u8; 32]) -> Option<u64> {
        if self.entries.contains_key(&(kind, key)) {
            return None;
        }
        self.next_gen += 1;
        self.entries.insert((kind, key), self.next_gen);
        Some(self.next_gen)
    }

    fn evict(&mut self, kind: ResourceKind, key: &[u8; 32]) {
        self.entries.remove(&(kind, *key));
    }

    fn new_session(&mut self) {
        self.entries.clear();
    }
}

fn bitmap_key(u: &super::super::endpoint::BitmapUpload) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(&u.w.to_le_bytes());
    h.update(&u.h.to_le_bytes());
    h.update(&u.stride.to_le_bytes());
    h.update(&u.rgba);
    *h.finalize().as_bytes()
}

fn produce_uploads(view: View<()>) -> (usize, Vec<[u8; 32]>, usize) {
    let mut surface = HeadlessSurface::new(RepairComponent { view }, W, H);
    let mut uploads = Vec::new();
    let mut bytes = 0usize;
    let _ = surface.render_frame_v2();
    for u in surface.drain_bitmap_uploads() {
        bytes += u.rgba.len();
        uploads.push(bitmap_key(&u));
    }
    (uploads.len(), uploads, bytes)
}

#[test]
fn rq_repair_fr4_resource_identity_zero_reupload() {
    // 1) 负控（无驻留缓存 = 062 前行为）：每帧新建 surface → 每帧都重传。
    let stable: View<()> = View::code_editor("fr4-stable")
        .value("fn main() {\n    println!(\"稳定\");\n}")
        .lang("rust")
        .build();
    let mut raw: Vec<Vec<[u8; 32]>> = Vec::new();
    let mut counts = Vec::new();
    let mut bytes = Vec::new();
    for _ in 0..6 {
        let mut surface = HeadlessSurface::new(RepairComponent { view: stable.clone() }, W, H);
        let _ = surface.render_frame_v2();
        let uploads: Vec<_> = surface.drain_bitmap_uploads();
        counts.push(uploads.len());
        bytes.push(uploads.iter().map(|u| u.rgba.len()).sum::<usize>());
        raw.push(uploads.iter().map(bitmap_key).collect());
    }
    let prefixes: Vec<Vec<String>> = raw
        .iter()
        .map(|ks| {
            ks.iter()
                .map(|k| format!("{:02x}{:02x}{:02x}{:02x}", k[0], k[1], k[2], k[3]))
                .collect()
        })
        .collect();
    // 稳定起始帧：从第 i 帧起与上一帧键集相同。
    let mut stable_from = None;
    for i in 1..raw.len() {
        if raw[i] == raw[i - 1] {
            stable_from = Some(i);
            break;
        }
    }
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr4_nocache_control", "uploads_per_frame": counts,
            "bytes_per_frame": bytes, "key_prefixes": prefixes,
            "stable_from_frame": stable_from,
            "note": "无驻留缓存（每帧新 surface）= 062 前行为：稳定内容仍每帧上传（负控）",
        })
    );
    assert!(
        counts.iter().all(|n| *n > 0),
        "无缓存负控：稳定帧仍逐帧上传（否则对照不成立）"
    );
    let stable_from = stable_from.expect("编辑器应出现内容稳定的连续帧");
    let key = raw[stable_from][0];

    // 2) 正控（062 生产路径：内容身份 + 驻留缓存）：同一 surface 连帧，
    //    首帧上传、稳定帧零上传、字节级变更才重传、改回命中驻留。
    let mut surface = HeadlessSurface::new(RepairComponent { view: stable.clone() }, W, H);
    let mut per_frame = Vec::new();
    for _ in 0..5 {
        let _ = surface.render_frame_v2();
        per_frame.push(surface.drain_bitmap_uploads().len());
    }
    let (uploads, _uploaded_bytes, hits, _evictions, resident) = surface.resource_stats();
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr4_production_cache", "uploads_per_frame": per_frame,
            "total_uploads": uploads, "hits": hits, "resident_bytes": resident,
        })
    );
    assert!(per_frame[0] > 0, "首帧必须上传资源");
    assert_eq!(
        per_frame[1..].iter().sum::<usize>(),
        0,
        "稳定帧必须零重传（P-02/V-06）"
    );
    assert!(hits > 0, "稳定帧必须命中驻留缓存");
    assert_eq!(uploads as usize, per_frame[0], "稳定段不得产生新上传");

    // 内容变更 → 新身份 → 一次重传；改回 → 命中驻留（零重传）。
    // 用矢量图标（生产端栅格化资源）做内容变更面：改文本不必然改变位图。
    let mut icon_surface = HeadlessSurface::new(
        RepairComponent {
            view: View::image("lucide:bell"),
        },
        W,
        H,
    );
    let _ = icon_surface.render_frame_v2();
    let first_icon_uploads = icon_surface.drain_bitmap_uploads().len();
    assert!(first_icon_uploads > 0, "矢量图标必须栅格化为位图资源");
    let _ = icon_surface.render_frame_v2();
    assert_eq!(
        icon_surface.drain_bitmap_uploads().len(),
        0,
        "同图标稳定帧必须零重传"
    );
    icon_surface.component.view = View::image("lucide:check");
    let _ = icon_surface.render_frame_v2();
    let changed_upload_count = icon_surface.drain_bitmap_uploads().len();
    assert!(changed_upload_count > 0, "内容变更必须以新身份重传一次");
    icon_surface.component.view = View::image("lucide:bell");
    let _ = icon_surface.render_frame_v2();
    assert_eq!(
        icon_surface.drain_bitmap_uploads().len(),
        0,
        "改回原内容必须命中驻留缓存"
    );

    // 驱逐 → 同内容重传一次（小上限迫使最早键出列）。
    icon_surface.set_resource_limit_bytes(1);
    let _ = icon_surface.render_frame_v2();
    assert!(
        !icon_surface.drain_bitmap_uploads().is_empty(),
        "驱逐后同内容必须重传（可解释的重传）"
    );
    let (_, _, _, evictions, _) = icon_surface.resource_stats();
    assert!(evictions > 0, "小上限必须发生驱逐");
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr4_change_evict", "changed_uploads": changed_upload_count,
            "evictions": evictions,
        })
    );

    // 3) 冻结方案模型：稳定帧零重传；变更/驱逐/新 session 才重传。
    let mut registry = ResourceRegistry::new();
    let g1 = registry.observe(ResourceKind::Bitmap, key);
    assert!(g1.is_some(), "首次出现必须上传");
    let stable_uploads = (0..4)
        .filter(|_| registry.observe(ResourceKind::Bitmap, key).is_some())
        .count();
    assert_eq!(stable_uploads, 0, "稳定帧（第 2 帧起）必须零上传");

    let mut changed_bytes = raw[stable_from][0];
    changed_bytes[0] ^= 0xFF;
    let changed_key = bitmap_key(&super::super::endpoint::BitmapUpload {
        id: String::new(),
        w: 1,
        h: 1,
        stride: 4,
        rgba: changed_bytes.to_vec(),
    });
    assert_ne!(changed_key, key, "字节级内容变更必须产生不同内容键");
    let g2 = registry.observe(ResourceKind::Bitmap, changed_key);
    assert!(g2.is_some(), "内容变更必须重传");
    assert!(g2.unwrap() > g1.unwrap(), "generation 必须单调递增");
    // 驱逐 → 同键重新上传；新 session → 清空后重传。
    registry.evict(ResourceKind::Bitmap, &key);
    assert!(
        registry.observe(ResourceKind::Bitmap, key).is_some(),
        "驱逐后同键必须重传"
    );
    registry.new_session();
    assert!(
        registry.observe(ResourceKind::Bitmap, key).is_some(),
        "新 session 必须全量重传"
    );
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({
            "tag": "fr4_identity_model",
            "first_gen": g1, "changed_gen": g2,
            "stable_frame_uploads": stable_uploads,
            "evict_reupload": true, "new_session_reupload": true,
            "note": "内容身份+generation 模型与 062 生产缓存同律（V-06 关闭口径）",
        })
    );

    // 4) 资源种类通用性：同一注册表可承载 bitmap/font/svg/mesh/media 身份。
    let mut multi = ResourceRegistry::new();
    let mut kinds = Vec::new();
    for kind in [
        ResourceKind::Bitmap,
        ResourceKind::Font,
        ResourceKind::Svg,
        ResourceKind::Mesh,
        ResourceKind::Media,
    ] {
        kinds.push((format!("{kind:?}"), multi.observe(kind, key).is_some()));
        assert!(
            multi.observe(kind, key).is_none(),
            "{kind:?} 稳定命中必须零上传"
        );
    }
    println!(
        "[RQREPAIR] {}",
        serde_json::json!({ "tag": "fr4_kinds", "first_upload": kinds })
    );
}
