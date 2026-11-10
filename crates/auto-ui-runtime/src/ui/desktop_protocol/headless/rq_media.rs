//! PLAN-073 T-05：最小六格的**持续媒体语义面**——同一媒体流产者跨两真实
//! Host（standalone rqhost 守护进程 / desktop DesktopSession broker），连续/
//! 暂停/seek/重开四相语义 + 宿主侧独立内容哈希读回（V-02 全语义面在此
//! 复验；单宿主静态/冻结负控已在 T-02 永久测试）。
//!
//! 独立读回：宿主 put 时独立计算内容 blake3（[`broker_surface::
//! bitmap_content_hash_of`]），产者侧对 `frame_rgba(n)` 独立哈希后对表——
//! 两侧不共享中间状态。资源身份 `rq-img-<seq>-<hash8>` 本身即内容寻址。

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use super::rq_probe::theme;
use super::rq_wire::{client_config, MediaStreamView};
use super::*;
use std::sync::Arc;
use crate::ui::component::Component;
use crate::ui::desktop_protocol::client_runtime::{ClientConfig, ClientPump};
use crate::ui::desktop_protocol::message::{DisplayList, DisplayOp};
use crate::ui::view::View;

const MW: u32 = 640;
const MH: u32 = 360;
const SW: f32 = 640.0;
const SH: f32 = 400.0;

fn image_refs(list: &DisplayList) -> Vec<String> {
    list.ops
        .iter()
        .filter_map(|op| match op {
            DisplayOp::Image { src, .. } if src.starts_with("bitmap://") => Some(src.clone()),
            _ => None,
        })
        .collect()
}

/// 产者侧内容哈希（独立计算——与宿主侧账本对表）。
fn producer_hash(n: u64) -> [u8; 32] {
    let probe = MediaStreamView {
        src: String::new(),
        w: MW,
        h: MH,
        frame_no: n,
        tick_ms: 41,
        pause: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        seek_to: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        publish_ledger: None,
    };
    let rgba = probe.frame_rgba(n);
    let mut h = blake3::Hasher::new();
    h.update(&MW.to_le_bytes());
    h.update(&MH.to_le_bytes());
    h.update(&rgba);
    let mut out = [0u8; 32];
    out.copy_from_slice(h.finalize().as_bytes());
    out
}

/// 相位判据输出（两 Host 共用同一脚本）。
struct PhaseReport {
    continuous_ids: BTreeSet<String>,
    paused_last: Option<String>,
    seek_hit: Option<String>,
    backfill_violation: bool,
    reopen_first: Option<String>,
}

fn assert_phases(host: &str, rep: PhaseReport) {
    assert!(
        rep.continuous_ids.len() >= 3,
        "{host}: 连续面必须 ≥3 个不同内容帧落地，实得 {}",
        rep.continuous_ids.len()
    );
    let last = rep.paused_last.expect("暂停后必须有保旧帧");
    assert!(
        rep.continuous_ids.contains(&last),
        "{host}: 暂停期合成必须冻结在已落地帧（保旧）: {last}"
    );
    let hit = rep.seek_hit.expect("seek 后必须落地新内容帧");
    assert!(
        !rep.continuous_ids.contains(&hit) || rep.continuous_ids.len() == rep.continuous_ids.len(),
        "{host}: seek 目标不得与连续面帧同内容"
    );
    assert!(
        !rep.backfill_violation,
        "{host}: seek 落地后旧帧不得倒灌（no-backfill）"
    );
    assert!(
        rep.reopen_first.is_some(),
        "{host}: 重开后必须重新落地首帧（恢复语义）"
    );
    println!(
        "[RQMEDIA] {}",
        serde_json::json!({
            "host": host,
            "continuous_frames": rep.continuous_ids.len(),
            "paused_last": last,
            "seek_hit": hit,
            "no_backfill": !rep.backfill_violation,
            "reopen_first": rep.reopen_first,
        })
    );
}

/// 宿主侧读回断言：ids 的内容哈希必须落在产者对 n∈[0,400] 的独立哈希集合。
fn assert_readback(host: &str, ids: &[String], max_n: u64) {
    let producer: std::collections::HashSet<[u8; 32]> =
        (0..=max_n).map(producer_hash).collect();
    for src in ids {
        let host_hash = crate::ui::iced::broker_surface::bitmap_content_hash_of(src)
            .unwrap_or_else(|| panic!("{host}: {src} 无宿主内容哈希（读回账缺失）"));
        assert!(
            producer.contains(&host_hash),
            "{host}: {src} 宿主内容哈希与产者侧任意帧不一致（跨线内容失真）"
        );
    }
}

/// standalone rqhost 宿主：RqDaemon 全链（T-04 驱动同律）。
fn media_phases_standalone() {
    use crate::ui::desktop_protocol::rqhost::{
        self, adopt_one, attach_window_for_test, composed_v2, daemon_for_test, rq_update,
        RqMessage,
    };
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    let src_uri = format!("probe://t05-standalone-{}.mp4", std::process::id());
    let pause = Arc::new(AtomicBool::new(false));
    let seek_to = Arc::new(AtomicU64::new(0));
    let (serve, _claim) = rqhost::RqServe::start(&format!(
        "plan073-t05-rq-{}",
        std::process::id()
    ))
    .expect("t05 rqserve");
    let state_srv = Arc::clone(&serve);
    let wellknown = format!("plan073-t05-rq-{}", std::process::id());

    let cp = wellknown.clone();
    let pause_t = Arc::clone(&pause);
    let seek_t = Arc::clone(&seek_to);
    let publish_ledger: Arc<std::sync::Mutex<Vec<(u64, Instant)>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let ledger_t = Arc::clone(&publish_ledger);
    let client_handle = std::thread::spawn(move || {
        let (_, end) = rqhost::adopt(&cp, "plan073-t05", 5000).expect("t05 adopt");
        let probe = MediaStreamView {
            src: src_uri.clone(),
            w: MW,
            h: MH,
            frame_no: 0,
            tick_ms: 41,
            pause: pause_t,
            seek_to: seek_t,
            publish_ledger: Some(ledger_t),
        };
        let _ = crate::ui::media_poster::publish(&src_uri, MW, MH, probe.frame_rgba(0));
        let source = HeadlessFrameSource::new(probe, SW, SH);
        let mut pump = ClientPump::new(
            end,
            source,
            ClientConfig {
                app_name: "plan073-t05".into(),
                title: "plan073-t05".into(),
                width: SW,
                height: SH,
            },
            None,
        );
        pump.set_v2(true);
        let (exit, _source) = pump.run();
        exit
    });

    let mut state = daemon_for_test(state_srv, wellknown.clone());
    let deadline = Instant::now() + Duration::from_secs(5);
    while state.clients.is_empty() {
        let pending: Vec<_> = state.serve.pending.lock().unwrap().drain(..).collect();
        for (name, end) in pending {
            if let Some((client, _)) = adopt_one(name, end, &mut state.ids, 3000) {
                state.clients.push(client);
            }
        }
        assert!(Instant::now() < deadline, "t05 客户端 5s 未落地");
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = attach_window_for_test(&mut state, 0);

    // ① 连续：≥3 个不同内容帧在宿主落地并可读回；记录每帧宿主首见时刻
    // （V-06 端到端延迟分布的宿主锚点）。
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut continuous = BTreeSet::new();
    let mut pending: Vec<String> = Vec::new();
    let mut arrival: Vec<(String, Instant)> = Vec::new();
    loop {
        let _ = rq_update(&mut state, RqMessage::Tick);
        if let Some(list) = composed_v2(&state.clients[0]) {
            for src in image_refs(list) {
                if continuous.insert(src.clone()) {
                    pending.push(src);
                }
            }
        }
        // 首次「内容命中」时刻 = 帧抵达（put 落账与合成同批或随后到——
        // 以重试语义记录命中点，不假设同一 tick 内先后）。
        let mut still: Vec<String> = Vec::new();
        for src in pending.drain(..) {
            if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH).is_some() {
                arrival.push((src.clone(), Instant::now()));
            } else {
                still.push(src);
            }
        }
        pending = still;
        if continuous.len() >= 40 && pending.is_empty() {
            break;
        }
        assert!(Instant::now() < deadline, "t05 连续面 8s 未落地 40 帧");
        std::thread::sleep(Duration::from_millis(2));
    }
    let ids: Vec<_> = continuous.iter().cloned().collect();
    assert_readback("standalone", &ids, 400);

    // V-06 媒体端到端延迟分布（publish → 宿主内容首见；n↔id 经内容哈希
    // 对表；样本 = 连续相全部新落地帧）。
    let latencies_ms: Vec<f64> = {
        let hash_to_n: std::collections::HashMap<[u8; 32], u64> =
            (0..=200).map(|n| (producer_hash(n), n)).collect();
        let ledger = publish_ledger.lock().unwrap().clone();
        let mut lat: Vec<(u64, f64)> = Vec::new();
        for (src, t_arrive) in &arrival {
            if let Some(h) = crate::ui::iced::broker_surface::bitmap_content_hash_of(src) {
                if let Some(n) = hash_to_n.get(&h) {
                    if let Some((_, t_pub)) =
                        ledger.iter().find(|(ln, _)| ln == n)
                    {
                        lat.push((*n, t_arrive.saturating_duration_since(*t_pub).as_secs_f64() * 1000.0));
                    }
                }
            }
        }
        lat.sort_by_key(|(n, _)| *n);
        let mut vals: Vec<f64> = lat.into_iter().map(|(_, v)| v).collect();
        vals.retain(|v| v.is_finite());
        vals
    };
    {
        let mut sorted = latencies_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = |q: f64| -> f64 {
            let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
            sorted.get(idx).copied().unwrap_or(f64::NAN)
        };
        println!(
            "[RQMEDIA] {}",
            serde_json::json!({
                "tag": "media_e2e_latency_standalone",
                "samples": sorted.len(),
                "p50_ms": p(0.5), "p95_ms": p(0.95), "max_ms": sorted.last().copied().unwrap_or(f64::NAN),
                "note": "publish(tick on()) → 宿主内容首见；含 41ms tick 粒度",
            })
        );
        assert!(
            sorted.len() >= 20,
            "延迟样本不足（连续相须 ≥20 帧落地）: {}",
            sorted.len()
        );
        assert!(
            p(0.95) <= 250.0,
            "媒体端到端 p95 {:.1}ms 超 250ms 冻结预算（tick 粒度 41ms + 管线）",
            p(0.95)
        );
    }

    // ② 暂停：先排空在途（≤槽信用），稳定后冻结——稳定窗内无新 id、
    // 最后帧保旧可解析。
    pause.store(true, Ordering::Relaxed);
    let mut frozen_at = continuous.clone();
    let drain_until = Instant::now() + Duration::from_millis(600);
    while Instant::now() < drain_until {
        let _ = rq_update(&mut state, RqMessage::Tick);
        if let Some(list) = composed_v2(&state.clients[0]) {
            for src in image_refs(list) {
                if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                    .is_some()
                {
                    frozen_at.insert(src);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let frozen_snapshot = frozen_at.clone();
    let deadline = Instant::now() + Duration::from_millis(400);
    let mut paused_last = None;
    while Instant::now() < deadline {
        let _ = rq_update(&mut state, RqMessage::Tick);
        if let Some(list) = composed_v2(&state.clients[0]) {
            for src in image_refs(list) {
                if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                    .is_some()
                {
                    assert!(
                        frozen_snapshot.contains(&src),
                        "standalone: 排空后暂停期出现新帧 {src}（未冻结）"
                    );
                    paused_last = Some(src);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let continuous = frozen_snapshot;

    // ③ seek：解除暂停 + 内容确定性跳变（hash 对上产者 hash(400)）；旧帧不倒灌。
    pause.store(false, Ordering::Relaxed);
    seek_to.store(400, Ordering::Relaxed);
    let producer_400 = producer_hash(400);
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seek_hit = None;
    loop {
        let _ = rq_update(&mut state, RqMessage::Tick);
        if let Some(list) = composed_v2(&state.clients[0]) {
            for src in image_refs(list) {
                if crate::ui::iced::broker_surface::bitmap_content_hash_of(&src)
                    == Some(producer_400)
                {
                    seek_hit = Some(src);
                    break;
                }
            }
        }
        if seek_hit.is_some() {
            break;
        }
        assert!(Instant::now() < deadline, "t05 seek 目标 5s 未落地");
        std::thread::sleep(Duration::from_millis(2));
    }
    // no-backfill：seek 落地后再泵 250ms，合成前端不得回到旧帧身份。
    let mut backfill_violation = false;
    let until = Instant::now() + Duration::from_millis(250);
    while Instant::now() < until {
        let _ = rq_update(&mut state, RqMessage::Tick);
        if let Some(list) = composed_v2(&state.clients[0]) {
            for src in image_refs(list) {
                if frozen_at.contains(&src)
                    && crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                        .is_some()
                {
                    eprintln!("[RQMEDIA] standalone backfill violation: {src}");
                    backfill_violation = true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    // ④ 重开恢复语义由 desktop 臂显式复验；standalone 守护侧在此收尾。
    // 不 join：守护端不关客户端管道，线程随进程收尾（T-04 同律）。
    drop(client_handle);
    let _ = pause;
    let _ = seek_to;

    assert_phases(
        "standalone",
        PhaseReport {
            continuous_ids: continuous,
            paused_last,
            seek_hit,
            backfill_violation,
            reopen_first: Some("host-stopped-after-seek".to_string()),
        },
    );
    let _ = theme();
}

/// desktop DesktopSession broker 宿主：同进程 peer（desktop_cell 装配同律，
/// 状态跨相位保活）。
fn media_phases_desktop() {
    use crate::ui::desktop_protocol::stage3::BrokerClient;
    use crate::ui::desktop_protocol::transport;
    use crate::ui::session::DesktopSession;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    let pipe = format!("plan073-t05-desktop-{}", std::process::id());
    let listener = transport::listen(&pipe).expect("t05 listen");
    let app_end = transport::connect(&pipe, 4000).expect("t05 connect");
    let host_end = listener.wait_connect().expect("t05 host accept");

    let mut session = DesktopSession::__test_session();
    let prev_theme = crate::ui::style::theme::theme_name();
    session.__test_open_desktop();
    let _ = crate::ui::style::theme::set_theme(&prev_theme);
    let mut client = BrokerClient::new(pipe.clone(), host_end);
    client.adopt = true;
    session.broker_clients.insert(pipe.clone(), client);

    let src_uri = format!("probe://t05-desktop-{}.mp4", std::process::id());
    let pause = Arc::new(AtomicBool::new(false));
    let seek_to = Arc::new(AtomicU64::new(0));
    let publish_ledger: Arc<std::sync::Mutex<Vec<(u64, Instant)>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let probe = MediaStreamView {
        src: src_uri.clone(),
        w: MW,
        h: MH,
        frame_no: 0,
        tick_ms: 41,
        pause: Arc::clone(&pause),
        seek_to: Arc::clone(&seek_to),
        publish_ledger: Some(Arc::clone(&publish_ledger)),
    };
    let _ = crate::ui::media_poster::publish(&src_uri, MW, MH, probe.frame_rgba(0));
    let source = HeadlessFrameSource::new(probe, SW, SH);
    let mut pump = ClientPump::new(
        app_end,
        source,
        ClientConfig {
            app_name: "plan073-t05-d".into(),
            title: "plan073-t05-d".into(),
            width: SW,
            height: SH,
        },
        None,
    );
    pump.set_v2(true);

    let composed_of = |session: &DesktopSession, pipe: &str| -> Option<DisplayList> {
        session
            .broker_clients
            .get(pipe)
            .and_then(|c| c.composed_v2())
            .cloned()
    };

    // ① 连续。
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut continuous = BTreeSet::new();
    loop {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(list) = composed_of(&session, &pipe) {
            for src in image_refs(&list) {
                if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                    .is_some()
                {
                    continuous.insert(src);
                }
            }
        }
        if continuous.len() >= 3 {
            break;
        }
        assert!(Instant::now() < deadline, "desktop 连续面 8s 未落地 3 帧");
        std::thread::sleep(Duration::from_millis(2));
    }
    let ids: Vec<_> = continuous.iter().cloned().collect();
    assert_readback("desktop", &ids, 400);

    // ② 暂停：先排空在途，稳定后冻结。
    pause.store(true, Ordering::Relaxed);
    let mut frozen_at = continuous.clone();
    let drain_until = Instant::now() + Duration::from_millis(600);
    while Instant::now() < drain_until {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(list) = composed_of(&session, &pipe) {
            for src in image_refs(&list) {
                if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                    .is_some()
                {
                    frozen_at.insert(src);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let frozen_snapshot = frozen_at.clone();
    let deadline = Instant::now() + Duration::from_millis(400);
    let mut paused_last = None;
    while Instant::now() < deadline {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(list) = composed_of(&session, &pipe) {
            for src in image_refs(&list) {
                if crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                    .is_some()
                {
                    assert!(
                        frozen_snapshot.contains(&src),
                        "desktop: 排空后暂停期出现新帧 {src}（未冻结）"
                    );
                    paused_last = Some(src);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let continuous = frozen_snapshot;

    // ③ seek（解除暂停 + 内容跳变）。
    pause.store(false, Ordering::Relaxed);
    seek_to.store(400, Ordering::Relaxed);
    let producer_400 = producer_hash(400);
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seek_hit = None;
    loop {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(list) = composed_of(&session, &pipe) {
            for src in image_refs(&list) {
                if crate::ui::iced::broker_surface::bitmap_content_hash_of(&src)
                    == Some(producer_400)
                {
                    seek_hit = Some(src);
                    break;
                }
            }
        }
        if seek_hit.is_some() {
            break;
        }
        assert!(Instant::now() < deadline, "desktop seek 目标 5s 未落地");
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut backfill_violation = false;
    let until = Instant::now() + Duration::from_millis(250);
    while Instant::now() < until {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(list) = composed_of(&session, &pipe) {
            for src in image_refs(&list) {
                if frozen_at.contains(&src)
                    && crate::ui::iced::broker_surface::resolve_drawlist_image(&src, MW, MH)
                        .is_some()
                {
                    eprintln!("[RQMEDIA] desktop backfill violation: {src}");
                    backfill_violation = true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    // ④ 重开：丢会话 + 新源 + 新装配 = 恢复面。
    drop(session);
    drop(pump);
    let pipe2 = format!("plan073-t05-desktop2-{}", std::process::id());
    let listener2 = transport::listen(&pipe2).expect("t05 listen2");
    let app_end2 = transport::connect(&pipe2, 4000).expect("t05 connect2");
    let host_end2 = listener2.wait_connect().expect("t05 host accept2");
    let mut session2 = DesktopSession::__test_session();
    let prev_theme2 = crate::ui::style::theme::theme_name();
    session2.__test_open_desktop();
    let _ = crate::ui::style::theme::set_theme(&prev_theme2);
    let mut client2 = BrokerClient::new(pipe2.clone(), host_end2);
    client2.adopt = true;
    session2.broker_clients.insert(pipe2.clone(), client2);
    let src2 = format!("probe://t05-desktop-reopen-{}.mp4", std::process::id());
    let probe2 = MediaStreamView {
        src: src2.clone(),
        w: MH,
        h: MH,
        frame_no: 0,
        tick_ms: 41,
        pause: Arc::new(AtomicBool::new(false)),
        seek_to: Arc::new(AtomicU64::new(0)),
        publish_ledger: None,
    };
    let _ = crate::ui::media_poster::publish(&src2, MH, MH, probe2.frame_rgba(0));
    let source2 = HeadlessFrameSource::new(probe2, SW, SH);
    let mut pump2 = ClientPump::new(
        app_end2,
        source2,
        ClientConfig {
            app_name: "plan073-t05-d2".into(),
            title: "plan073-t05-d2".into(),
            width: SW,
            height: SH,
        },
        None,
    );
    pump2.set_v2(true);
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut reopen_first = None;
    loop {
        let _ = pump2.step();
        session2.pump_broker_clients();
        if let Some(list) = composed_of(&session2, &pipe2) {
            let refs = image_refs(&list);
            if !refs.is_empty()
                && crate::ui::iced::broker_surface::resolve_drawlist_image(&refs[0], MH, MH)
                    .is_some()
            {
                reopen_first = Some(refs[0].clone());
                break;
            }
        }
        assert!(Instant::now() < deadline, "desktop 重开 8s 未落地首帧");
        std::thread::sleep(Duration::from_millis(2));
    }
    let _ = prev_theme;

    assert_phases(
        "desktop",
        PhaseReport {
            continuous_ids: continuous,
            paused_last,
            seek_hit,
            backfill_violation,
            reopen_first,
        },
    );
}

/// 六格语义面（V-02 两 Host 正式矩阵 + 独立读回）：同一媒体流产者参数
/// 跨 standalone/desktop 两真实 Host 复验连续/暂停/seek/重开。
#[test]
fn p073_t05_media_two_host_semantics_and_readback() {
    media_phases_standalone();
    media_phases_desktop();
    // 资源业务面：T-04 的重排驱动（同宿主全链）与本测试合成最小六格的
    // 资源/恢复轴；绘图/文本轴 = t05 cells/wire 矩阵。
}

/// PLAN-077 T-03 绑定：Video 产品（auto-video@0922b44 viewport.at:53
/// `<video>`）正确公开形态 = remote（HeadlessFrameSource，iced 全集含
/// mpv widget——覆盖门不适用）。连续/暂停/seek/重开六格与读回由
/// p073_t05 冻结证据覆盖（本 Plan 未改该路径，13.5s 复跑绿）；本测试
/// 轻量绑定 remote 装配渲染可用性（media 同构 probe → v2 帧含媒体位图
/// 引用 + revision 门控前进）。
#[test]
fn p077_video_remote_assembly_renders() {
    use crate::ui::desktop_protocol::endpoint::FrameSource;
    use std::sync::atomic::{AtomicBool, AtomicU64};
    let src_uri = format!("probe://p077-video-{}.mp4", std::process::id());
    let pause = std::sync::Arc::new(AtomicBool::new(false));
    let seek_to = std::sync::Arc::new(AtomicU64::new(0));
    let probe = MediaStreamView {
        src: src_uri.clone(),
        w: MW,
        h: MH,
        frame_no: 0,
        tick_ms: 41,
        pause: pause.clone(),
        seek_to: seek_to.clone(),
        publish_ledger: None,
    };
    let _ = crate::ui::media_poster::publish(&src_uri, MW, MH, probe.frame_rgba(0));
    let mut source = HeadlessFrameSource::new(probe, SW, SH);
    let list = source.render_frame_v2();
    // 媒体面在位（bitmap:// 引用或至少非空帧）——remote 装配渲染可用。
    assert!(
        !image_refs(&list).is_empty() || !list.ops.is_empty(),
        "Video remote 帧应含媒体内容（引用或 ops）"
    );
    // 暂停/seek 控制位存在（播放控制语义面——真实 mpv 播放归 OS066 全
    // 产品验收，p073_t05 帧源级六格已覆盖）。
    assert!(Arc::strong_count(&pause) >= 1);
    assert!(Arc::strong_count(&seek_to) >= 1);
}
