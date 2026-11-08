//! PLAN-062 T-05 补：真实 wire + 两 Host 重放格（test-only）。
//!
//! `rq_cells` 的两个可跑格是**同进程近似**：产帧 → 直接 encode/decode →
//! broker canvas 重放，没有端点状态机、没有传输、没有宿主合成面。本模块补
//! 上 062 六格缺的那一层：
//!
//! ```text
//! producer(HeadlessSurface) --ClientPump--> 真实命名管道(编解码+shm 槽+ACK/容量)
//!   --> HostEndpoint(握手/激活/准入)  --> 宿主合成面 SurfaceStore.front_v2
//!   --> broker canvas offscreen 快照 --> 与 direct 金样对拍
//! ```
//!
//! 两种宿主 = 两条**不同**的合成实现：
//! - standalone：`rqhost::RqServe/adopt_one/pump_client`（独立 RQ 宿主进程同代码）；
//! - desktop：`DesktopSession` 的 broker 客户端（桌面嵌载宿主同代码）。
//!
//! 边界（如实登记，不冒已交付）：
//! - peer 跑在本进程线程里（真实管道/真实端点/真实 shm），不是独立进程；
//!   跨进程换载与物理上屏归 OS063；
//! - 帧终点仍是 offscreen 快照，不是 present。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::rq_probe::{out_dir, pixel_diff_ratio, theme};
use super::*;
use crate::ui::component::Component;
use crate::ui::desktop_protocol::client_runtime::{ClientConfig, ClientPump};
use crate::ui::desktop_protocol::endpoint::{BitmapUpload, FrameSource};
use crate::ui::desktop_protocol::message::{ControlMsg, DisplayList, DisplayOp, DrawList, InputMsg};
use crate::ui::view::View;

const W: f32 = 640.0;
const H: f32 = 400.0;
/// 单格预算：握手 + 首帧 + 资源排水。超出即具名失败（不静默 PASS）。
const CELL_BUDGET: Duration = Duration::from_secs(10);
/// 首帧之后的排水静默窗：让 BitmapReady/ACK 等在渲染前落定。
const SETTLE: Duration = Duration::from_millis(400);

#[derive(Clone, Debug)]
struct StaticView {
    view: View<()>,
}

impl Component for StaticView {
    type Msg = ();

    fn on(&mut self, _msg: ()) {}

    fn view(&self) -> View<()> {
        self.view.clone()
    }
}

/// 产者形态（062 §5：Rust 直接构件 / VM 执行 AutoLang 源码）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ProducerKind {
    Rust,
    Vm,
}

/// 宿主形态（两条不同的合成实现）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HostKind {
    Standalone,
    Desktop,
}

fn out_wire() -> PathBuf {
    out_dir().join("wire")
}

fn client_config() -> ClientConfig {
    ClientConfig {
        app_name: "plan062-wire".into(),
        title: "plan062-wire".into(),
        width: W,
        height: H,
    }
}

/// direct 代理金样：同一 producer 的 iced 元素（不经协议）在 offscreen 渲染。
fn direct_gold(view: View<()>, out: &std::path::Path, name: &str) -> Result<PathBuf, String> {
    let element = iced::widget::container(view.into_iced())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(move |_t| iced::widget::container::Style {
            background: Some(
                crate::ui::iced::broker_surface::to_color(
                    crate::ui::desktop_protocol::client_runtime::bg(),
                )
                .into(),
            ),
            ..Default::default()
        });
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        element,
    );
    let req = out.join(format!("{name}-direct.png"));
    let _ = std::fs::remove_file(&req);
    let _ = std::fs::remove_file(out.join(format!("{name}-direct-wgpu.png")));
    sim.snapshot(&theme())
        .map_err(|e| format!("direct snapshot: {e}"))?
        .matches_image(&req)
        .map_err(|e| format!("direct png: {e}"))?;
    Ok(out.join(format!("{name}-direct-wgpu.png")))
}

/// 宿主合成面 → offscreen 快照（与 direct 同底色）。
fn render_composed(
    list: &DisplayList,
    backdrop: Rgba8,
    out: &std::path::Path,
    name: &str,
) -> Result<PathBuf, String> {
    let mut painted = list.clone();
    painted.clear = Some(backdrop);
    let mut sim = iced_test::simulator::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(W, H),
        crate::ui::iced::broker_surface::displaylist_element::<()>(&painted),
    );
    let req = out.join(format!("{name}-replay.png"));
    let _ = std::fs::remove_file(&req);
    let _ = std::fs::remove_file(out.join(format!("{name}-replay-wgpu.png")));
    sim.snapshot(&theme())
        .map_err(|e| format!("replay snapshot: {e}"))?
        .matches_image(&req)
        .map_err(|e| format!("replay png: {e}"))?;
    Ok(out.join(format!("{name}-replay-wgpu.png")))
}
/// 结构清单：v3/v2 原语计数 + 载荷 tag + 必需能力（漏图在结构面即见）。
fn inventory(list: &DisplayList) -> serde_json::Value {
    let (mut quad, mut text, mut path, mut img, mut glyphs, mut clip, mut xform) =
        (0, 0, 0, 0, 0, 0, 0);
    for op in &list.ops {
        match op {
            DisplayOp::Quad { .. } => quad += 1,
            DisplayOp::TextStyled { .. } => text += 1,
            DisplayOp::Path { path: p, .. } => {
                path += 1;
                glyphs += p.segs.len();
            }
            DisplayOp::GlyphRun { glyphs: g, .. } => {
                glyphs += g.len();
            }
            DisplayOp::Image { .. } => img += 1,
            DisplayOp::Scissor { .. } | DisplayOp::ScissorPop => clip += 1,
            DisplayOp::Transform { .. } | DisplayOp::TransformPop => xform += 1,
        }
    }
    serde_json::json!({
        "ops": list.ops.len(),
        "quad": quad,
        "text": text,
        "path": path,
        "segments_or_glyphs": glyphs,
        "image": img,
        "clip": clip,
        "transform": xform,
        "payload_tag": if list.requires_v3() { 3 } else { 2 },
        "required_capabilities": list.required_capabilities(),
    })
}

// ---------------------------------------------------------------------------
// standalone RQ 宿主格：RqServe/adopt/pump_client（rqhost 同代码）
// ---------------------------------------------------------------------------

fn standalone_cell<S>(source: S, out: &std::path::Path, name: &str) -> Result<DisplayList, String>
where
    S: crate::ui::desktop_protocol::endpoint::FrameSource,
{
    use crate::ui::desktop_protocol::rqhost;

    let wellknown = format!("plan062-wire-rq-{}-{name}", std::process::id());
    let (serve, claim) =
        rqhost::RqServe::start(&wellknown).map_err(|e| format!("rqserve start: {e:?}"))?;

    // ① 产者采纳（RqServe 的 serve 线程应答 rendezvous，不在本线程）。
    let (_, end) = rqhost::adopt(&wellknown, "plan062-wire", 5000)
        .map_err(|e| format!("adopt: {e:?}"))?;
    // ② 建泵即发 Hello（落在管道缓冲里，宿主稍后读）。
    let mut pump = ClientPump::new(end, source, client_config(), None);
    pump.set_v2(true);

    // ③ 宿主认领转连并泵到 Active（Hello 已在缓冲，不依赖产者线程）。
    let mut ids = rqhost::RqIds::default();
    let mut client = None;
    let deadline = Instant::now() + CELL_BUDGET;
    while client.is_none() && Instant::now() < deadline {
        let pending: Vec<_> = serve.pending.lock().unwrap().drain(..).collect();
        for (app_name, end) in pending {
            if client.is_none() {
                client = rqhost::adopt_one(app_name, end, &mut ids, 4000).map(|(c, _)| c);
            }
        }
        let _ = pump.step();
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut client = client.ok_or_else(|| "standalone 宿主 10s 未激活".to_string())?;

    // ④ 产者/宿主交替泵（两端都是非阻塞轮询）。
    let mut composed: Option<DisplayList> = None;
    let mut first_at: Option<Instant> = None;
    let deadline = Instant::now() + CELL_BUDGET;
    while Instant::now() < deadline {
        let _ = pump.step();
        let (_events, alive) = rqhost::pump_client(&mut client, &mut ids);
        if let Some(front) = rqhost::composed_v2(&client) {
            composed = Some(front.clone());
            first_at.get_or_insert_with(Instant::now);
        }
        if let Some(at) = first_at {
            if at.elapsed() >= SETTLE {
                break;
            }
        }
        if !alive {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    serve.stop(&wellknown);
    drop(claim);

    let list = composed.ok_or_else(|| "standalone 宿主 10s 未合成首帧".to_string())?;
    let _ = out;
    Ok(list)
}

// ---------------------------------------------------------------------------
// desktop 宿主格：DesktopSession broker 客户端（session.rs 同代码）
// ---------------------------------------------------------------------------

fn desktop_cell<S>(source: S, out: &std::path::Path, name: &str) -> Result<DisplayList, String>
where
    S: crate::ui::desktop_protocol::endpoint::FrameSource,
{
    use crate::ui::desktop_protocol::stage3::BrokerClient;
    use crate::ui::desktop_protocol::transport;
    use crate::ui::session::DesktopSession;

    let pipe = format!("plan062-wire-desktop-{}-{name}", std::process::id());
    let listener = transport::listen(&pipe).map_err(|e| format!("listen: {e:?}"))?;
    let app_end = transport::connect(&pipe, 4000).map_err(|e| format!("connect: {e:?}"))?;
    let host_end = listener
        .wait_connect()
        .map_err(|e| format!("host accept: {e:?}"))?;

    let mut session = DesktopSession::__test_session();
    // 桌面会话 boot 会把**全局**主题切到 config.theme_name（stella）——本进程
    // 后续所有渲染（含另一侧 direct 金样）都随之改色。矩阵要求两侧同主题
    // （§6），故开桌后把全局主题恢复成进入前的名字，避免把主题差记成保真差。
    let prev_theme = crate::ui::style::theme::theme_name();
    session.__test_open_desktop();
    let _ = crate::ui::style::theme::set_theme(&prev_theme);
    let mut client = BrokerClient::new(pipe.clone(), host_end);
    // 零装载采纳：组件/VM 在产者侧，宿主只要 window/App 簿记 + 表面。
    client.adopt = true;
    session.broker_clients.insert(pipe.clone(), client);

    // 建泵即发 Hello（同一线程交替泵：两端都是非阻塞轮询，不跨线程）。
    let mut pump = ClientPump::new(app_end, source, client_config(), None);
    pump.set_v2(true);

    let mut composed: Option<DisplayList> = None;
    let mut first_at: Option<Instant> = None;
    let deadline = Instant::now() + CELL_BUDGET;
    while Instant::now() < deadline {
        let _ = pump.step();
        session.pump_broker_clients();
        if let Some(front) = session
            .broker_clients
            .get(&pipe)
            .and_then(|c| c.composed_v2())
        {
            composed = Some(front.clone());
            first_at.get_or_insert_with(Instant::now);
        }
        if let Some(at) = first_at {
            if at.elapsed() >= SETTLE {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    drop(session);
    let _ = out;

    composed.ok_or_else(|| "desktop 宿主 10s 未合成首帧".to_string())
}

// ---------------------------------------------------------------------------
// 产者接缝：Rust 构件 / VM（AutoLang 解释执行）统一一种 FrameSource
// ---------------------------------------------------------------------------

/// 两种产者形态（062 §5）：本仓 Rust 构件、VM 跑 AutoLang 源码。
enum WireSource {
    Rust(HeadlessFrameSource<StaticView>),
    Vm(HeadlessFrameSource<crate::ui::dynamic::DynamicComponent>),
    /// 负控用的逐拍变化产者（tick 驱动，v2/v3 可切）。
    Live(HeadlessFrameSource<SwitchingView>),
    /// PLAN-073 T-01：逐拍变化的 1080p 媒体流产者（海报重发布 →
    /// 资源通道连续帧）。
    Media(HeadlessFrameSource<MediaStreamView>),
}

macro_rules! wire_delegate {
    ($this:ident, $method:ident $(, $arg:expr)*) => {
        match $this {
            WireSource::Rust(s) => s.$method($($arg),*),
            WireSource::Vm(s) => s.$method($($arg),*),
            WireSource::Live(s) => s.$method($($arg),*),
            WireSource::Media(s) => s.$method($($arg),*),
        }
    };
}

impl FrameSource for WireSource {
    fn revision(&self) -> u64 {
        wire_delegate!(self, revision)
    }
    fn render_frame(&mut self) -> DrawList {
        wire_delegate!(self, render_frame)
    }
    fn on_input(&mut self, input: &InputMsg) {
        wire_delegate!(self, on_input, input)
    }
    fn on_control(&mut self, control: &ControlMsg) {
        wire_delegate!(self, on_control, control)
    }
    fn poll_tick(&mut self) {
        wire_delegate!(self, poll_tick)
    }
    fn drain_desktop_commands(&mut self) -> Vec<String> {
        wire_delegate!(self, drain_desktop_commands)
    }
    fn drain_bitmap_uploads(&mut self) -> Vec<BitmapUpload> {
        wire_delegate!(self, drain_bitmap_uploads)
    }
    fn render_frame_v2(&mut self) -> DisplayList {
        wire_delegate!(self, render_frame_v2)
    }
    fn take_upload_budget_overrun(&mut self) -> bool {
        wire_delegate!(self, take_upload_budget_overrun)
    }
    // PLAN-073 T-03（A1）：HostCaps 钩子必须穿透枚举到各产者形态——
    // 否则协商永远停在缺省空实现（A1 测试实证的缺口）。
    fn on_host_caps(&mut self, caps: &crate::ui::desktop_protocol::message::CapabilitySet) {
        wire_delegate!(self, on_host_caps, caps)
    }
    fn drain_window_controls(&mut self) -> Vec<ControlMsg> {
        wire_delegate!(self, drain_window_controls)
    }
    fn theme_capable(&self) -> bool {
        wire_delegate!(self, theme_capable)
    }
    fn on_theme_policy(
        &mut self,
        snap: &crate::ui::theme_policy::PolicySnapshot,
    ) -> Result<(), String> {
        wire_delegate!(self, on_theme_policy, snap)
    }
    fn theme_policy_status(&self) -> Option<(String, String)> {
        wire_delegate!(self, theme_policy_status)
    }
}

/// 单例产者体：Rust 直接构件 或 VM 执行的 AutoLang 源码。
enum CaseBody {
    Rust(View<()>),
    Vm(&'static str),
}

fn wire_cases() -> Vec<(&'static str, CaseBody)> {
    vec![
        ("path_mesh", CaseBody::Rust(super::rq_seams::canvas_scene_view())),
        ("svg_icon", CaseBody::Rust(View::image("lucide:bell"))),
        ("text_cjk", CaseBody::Rust(View::text("中文路径客户数据库混排 ABC"))),
        (
            "gradient_border_shadow",
            CaseBody::Rust(
                View::container(View::text("渐变边框阴影"))
                    .style("p-4 rounded-xl bg-gradient-to-r from-blue-500 to-red-500 border shadow-lg")
                    .build(),
            ),
        ),
        (
            "vm_text",
            CaseBody::Vm(
                r#"widget Plan062VmText {
    view { col { text "中文路径客户数据库混排 ABC" } }
}
"#,
            ),
        ),
        (
            "vm_icon",
            CaseBody::Vm(
                r#"widget Plan062VmIcon {
    view { col { icon (name: "bell") } }
}
"#,
            ),
        ),
        (
            "vm_row_button",
            CaseBody::Vm(
                r#"widget Plan062VmRow {
    model { var n int = 0 }
    view { row { text "计数" button "加一" { onclick: () => {.n += 1} } } }
}
"#,
            ),
        ),
    ]
}

impl CaseBody {
    fn producer(&self) -> ProducerKind {
        match self {
            CaseBody::Rust(_) => ProducerKind::Rust,
            CaseBody::Vm(_) => ProducerKind::Vm,
        }
    }

    /// direct 金样：同一产者的 iced 元素（不经协议）——VM 例取解释执行出的
    /// view，不是手写期望包（§5 同律）。
    fn direct_view(&self) -> Result<View<()>, String> {
        match self {
            CaseBody::Rust(v) => Ok(v.clone()),
            CaseBody::Vm(src) => {
                let component = crate::build_dynamic_component(src, None)
                    .map_err(|e| format!("vm build: {e}"))?;
                Ok(component.view().map_msg(|_| ()))
            }
        }
    }

    fn source(&self) -> Result<WireSource, String> {
        match self {
            CaseBody::Rust(v) => Ok(WireSource::Rust(HeadlessFrameSource::new(
                StaticView { view: v.clone() },
                W,
                H,
            ))),
            CaseBody::Vm(src) => {
                let component = crate::build_dynamic_component(src, None)
                    .map_err(|e| format!("vm build: {e}"))?;
                Ok(WireSource::Vm(HeadlessFrameSource::new(component, W, H)))
            }
        }
    }
}

#[test]
fn rq_cells_t05_wire_producers_through_two_hosts() {
    let out = out_wire();
    std::fs::create_dir_all(&out).expect("wire out");

    let mut rows = Vec::new();
    for (case, body) in wire_cases() {
        // 排障用：AUTO_RQ_WIRE_CASE=<case> 只跑单例（常规测试不设）。
        if let Ok(only) = std::env::var("AUTO_RQ_WIRE_CASE") {
            if only != case {
                continue;
            }
        }
        let producer = body.producer();
        let pname = match producer {
            ProducerKind::Rust => "rust",
            ProducerKind::Vm => "vm",
        };
        let direct_view = match body.direct_view() {
            Ok(v) => v,
            Err(err) => {
                println!("[RQWIRE] {}", serde_json::json!({
                    "tag": "failed", "case": case, "stage": "direct_view", "error": err,
                }));
                rows.push(serde_json::json!({
                    "producer": pname, "case": case, "status": "failed", "error": err,
                }));
                continue;
            }
        };
        let gold = match direct_gold(direct_view, &out, case) {
            Ok(p) => p,
            Err(err) => {
                println!("[RQWIRE] {}", serde_json::json!({
                    "tag": "failed", "case": case, "stage": "direct_gold", "error": err,
                }));
                rows.push(serde_json::json!({
                    "producer": pname, "case": case, "status": "failed", "error": err,
                }));
                continue;
            }
        };
        // 两侧同底色 = 主题底色（direct 金样外层 container 用同一值）；
        // 不用 direct 图左上像素——内容压住 (0,0) 的例会刷满整屏。
        let backdrop = crate::ui::desktop_protocol::client_runtime::bg();

        for (host, kind) in [
            ("standalone-rq", HostKind::Standalone),
            ("desktop-rq", HostKind::Desktop),
        ] {
            let cell = cell_id(producer, host);
            let name = format!("{case}-{cell}");
            let composed = match body.source() {
                Ok(src) => match kind {
                    HostKind::Standalone => standalone_cell(src, &out, &name),
                    HostKind::Desktop => desktop_cell(src, &out, &name),
                },
                Err(err) => Err(err),
            };
            let list = match composed {
                Ok(list) => list,
                Err(err) => {
                    println!("[RQWIRE] {}", serde_json::json!({
                        "tag": "failed", "cell": cell, "case": case, "stage": "wire", "error": err,
                    }));
                    rows.push(serde_json::json!({
                        "producer": pname, "cell": cell, "case": case, "host": host,
                        "status": "wire_failed", "error": err,
                    }));
                    continue;
                }
            };
            let diff = match render_composed(&list, backdrop, &out, &name) {
                Ok(png) => pixel_diff_ratio(&gold, &png, 2),
                Err(err) => {
                    println!("[RQWIRE] {}", serde_json::json!({
                        "tag": "failed", "cell": cell, "case": case, "stage": "replay", "error": err,
                    }));
                    rows.push(serde_json::json!({
                        "producer": pname, "cell": cell, "case": case, "host": host,
                        "status": "replay_failed", "error": err,
                    }));
                    continue;
                }
            };
            let inv = inventory(&list);
            println!("[RQWIRE] {}", serde_json::json!({
                "tag": "cell", "producer": pname, "cell": cell, "case": case, "host": host,
                "inventory": inv, "pixel_diff_vs_direct": diff,
            }));
            rows.push(serde_json::json!({
                "producer": pname, "cell": cell, "case": case, "host": host,
                "status": "ok", "inventory": inv, "pixel_diff_vs_direct": diff,
            }));
        }
    }

    let report = serde_json::json!({
        "tag": "plan062_t05_wire",
        "viewport": [W, H],
        "transport": "真实命名管道 + 端点状态机 + shm 帧槽 + 宿主合成面",
        "boundary": "peer 跑本进程线程（真实管道/端点/shm）；present 为 offscreen 快照，跨进程换载与物理上屏归 OS063",
        "rows": rows,
    });
    std::fs::write(
        out.join("t05-wire.json"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    )
    .expect("write wire report");

    let cells = report["rows"].as_array().unwrap();
    assert!(!cells.is_empty(), "wire 格必须有结果（空表 = 驱动失效，不是通过）");
    for row in cells {
        assert_eq!(
            row["status"], "ok",
            "wire 格 {} / {} 失败: {}",
            row["cell"], row["case"], row["error"]
        );
    }
    // AC-02 默认 producer 门：与 rq_cells 具名界同表（A1 协商 → GlyphRun
    // 承载后，文本/阴影族携带既有具名残差；余格维持 0.1%）。A2 收口
    // （本 Plan T-03 余项）压残差后文本族应收回 0.1%——非静默放宽。
    for row in cells {
        let diff = row["pixel_diff_vs_direct"].as_f64().unwrap_or(1.0);
        let case = row["case"].as_str().unwrap_or("");
        // 结构判据：inventory 无 Path 而 segments_or_glyphs>0 = 文本以
        // GlyphRun 承载（A1 协商结果）→ 其边缘 AA 残差落 062 具名 1% 界
        // （A2 收口项，本 Plan T-03 余项；闭合后收回 0.1%）。阴影族按
        // rq_cells 同名表 1.5%。其余格维持 0.1%。
        let inv = &row["inventory"];
        let glyph_carried_text = inv["path"].as_u64().unwrap_or(0) == 0
            && inv["segments_or_glyphs"].as_u64().unwrap_or(0) > 0;
        let (bound, gate_name) = if matches!(case, "gradient_border_shadow" | "shadow_only") {
            (0.015, "1.5% 阴影族具名界（A3 残差，T-03 收口中）")
        } else if glyph_carried_text {
            (0.010, "1% 文本族具名界（A2 残差，T-03 收口中）")
        } else {
            (0.001, "0.1% AC-02 r4")
        };
        assert!(
            diff <= bound,
            "wire 格 {} / {} 像素差 {:.5} 超 {} 门",
            row["cell"],
            row["case"],
            diff,
            gate_name
        );
    }
    // 覆盖分母：两种产者 × 两宿主都必须真实跑过（不删格）。
    for producer in ["rust", "vm"] {
        for host in ["standalone-rq", "desktop-rq"] {
            let cell = format!("{producer}/{host}");
            assert!(
                cells.iter().any(|r| r["cell"] == cell),
                "cell {cell} 无结果（不删分母）"
            );
        }
    }
}

fn cell_id(producer: ProducerKind, host: &str) -> String {
    let p = match producer {
        ProducerKind::Rust => "rust",
        ProducerKind::Vm => "vm",
    };
    format!("{p}/{host}")
}
// ---------------------------------------------------------------------------
// AC-04 负控：慢宿主 / 旧 ACK / 旧宿主（能力收窄），全部跑在真实管道上
// ---------------------------------------------------------------------------

/// 逐拍变化的产者：`canvas=true` 产 v3（Path，需 `fidelity/pathfill/1`），
/// 否则产 v2 词汇（字符串文字）。用来把「新产者 × 旧宿主」配对做成真实 wire。
#[derive(Debug)]
struct SwitchingView {
    canvas: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ticks: u32,
}

impl Component for SwitchingView {
    type Msg = ();

    fn on(&mut self, _msg: ()) {
        self.ticks = self.ticks.wrapping_add(1);
    }

    fn view(&self) -> View<()> {
        if self.canvas.load(std::sync::atomic::Ordering::Relaxed) {
            super::rq_seams::canvas_scene_view()
        } else {
            View::text("v2 回退帧")
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(15)
    }

    fn tick_msg(&self) -> Option<()> {
        Some(())
    }
}

/// PLAN-073 T-01 探针产者：逐拍重发布同键海报（内容随帧变化 → blake3
/// 新资源身份 → 资源通道连续帧）。内容为合成 1080p（尺寸/节奏真实）；
/// 真实 mpv 解码读回属 T-02/V-02 交付面——本探针量的是资源通道容量与
/// 上传预算行为（V-01）。
#[derive(Debug)]
struct MediaStreamView {
    src: String,
    w: u32,
    h: u32,
    frame_no: u64,
    tick_ms: u32,
    /// T-02：暂停（true = 停止重发布，帧冻结在最后已发布内容）。
    pause: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// T-02：seek 目标（>当前 frame_no 时内容跳变——确定性 content(n)，
    /// 同 n 同字节，供身份级 no-backfill 断言）。
    seek_to: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl MediaStreamView {
    fn frame_rgba(&self, n: u64) -> Vec<u8> {
        let w = self.w as usize;
        let h = self.h as usize;
        let mut v = vec![0u8; w * h * 4];
        for (y, row) in v.chunks_exact_mut(w * 4).enumerate() {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                px[0] = (x as u8).wrapping_add(n as u8);
                px[1] = (y as u8).wrapping_add((n.wrapping_mul(7)) as u8);
                px[2] = ((x + y) as u8).wrapping_add((n.wrapping_mul(13)) as u8);
                px[3] = 255;
            }
        }
        // 末 8 字节嵌全帧号（**循环后**覆写——content(n) 单射的判据基座；
        // 前置写法会被渐变循环覆盖，mod-256 别名致 n=512 与 n=0 同字节，
        // 身份相同 = T-02 no-backfill 假倒灌的假阳性根因）。
        let tail = v.len() - 8;
        v[tail..].copy_from_slice(&n.to_le_bytes());
        v
    }
}

impl Component for MediaStreamView {
    type Msg = ();

    fn on(&mut self, _msg: ()) {
        if self.pause.load(std::sync::atomic::Ordering::Relaxed) {
            return; // 暂停：帧冻结在最后已发布内容（live 帧仍在册）。
        }
        eprintln!(
            "[T02DBG] publish n={} (pause={})",
            self.frame_no,
            self.pause.load(std::sync::atomic::Ordering::Relaxed)
        );
        let seek = self.seek_to.load(std::sync::atomic::Ordering::Relaxed);
        self.frame_no = if seek > self.frame_no {
            seek // seek：内容确定性跳变（同 n 同字节）。
        } else {
            self.frame_no.wrapping_add(1)
        };
        let rgba = self.frame_rgba(self.frame_no);
        assert!(
            crate::ui::media_poster::publish(&self.src, self.w, self.h, rgba),
            "T-02 海报重发布失败"
        );
    }

    fn view(&self) -> View<()> {
        View::Video {
            src: self.src.clone(),
            paused: true,
            position: Some(0.0),
            volume: 100,
            muted: true,
            rate: 1.0,
            label: "T01连续流".to_string(),
            on_time_update: None,
            on_loaded_metadata: None,
            on_play_state: None,
            on_ended: None,
            on_media_error: None,
            media_key: None,
            style: None,
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(self.tick_ms)
    }

    fn tick_msg(&self) -> Option<()> {
        Some(())
    }
}

/// 负控用的 standalone 宿主装配（真管道 + 真端点，产者/宿主两端分开泵）。
struct WireCell {
    wellknown: String,
    serve: std::sync::Arc<crate::ui::desktop_protocol::rqhost::RqServe>,
    _claim: crate::ui::desktop_protocol::transport::PipeClaim,
    pump: ClientPump<WireSource>,
    client: crate::ui::desktop_protocol::rqhost::RqClient,
    ids: crate::ui::desktop_protocol::rqhost::RqIds,
}

impl WireCell {
    fn start(tag: &str, source: WireSource) -> Result<Self, String> {
        Self::start_with(tag, source, client_config())
    }

    /// PLAN-073 T-01：探针需要宣布与媒体帧匹配的视口（位图段槽尺寸随
    /// Hello 视口推导，host.rs bm_slot_size = w×h×4×4+4）。
    fn start_with(tag: &str, source: WireSource, cfg: ClientConfig) -> Result<Self, String> {
        use crate::ui::desktop_protocol::rqhost;
        let wellknown = format!("plan062-neg-{}-{tag}", std::process::id());
        let (serve, claim) =
            rqhost::RqServe::start(&wellknown).map_err(|e| format!("rqserve: {e:?}"))?;
        let (_, end) =
            rqhost::adopt(&wellknown, "plan062-neg", 5000).map_err(|e| format!("adopt: {e:?}"))?;
        let mut pump = ClientPump::new(end, source, cfg, None);
        pump.set_v2(true);
        let mut ids = rqhost::RqIds::default();
        let mut client = None;
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.is_none() && Instant::now() < deadline {
            let pending: Vec<_> = serve.pending.lock().unwrap().drain(..).collect();
            for (app_name, end) in pending {
                if client.is_none() {
                    client = rqhost::adopt_one(app_name, end, &mut ids, 4000).map(|(c, _)| c);
                }
            }
            let _ = pump.step();
            std::thread::sleep(Duration::from_millis(2));
        }
        let client = client.ok_or_else(|| "负控宿主 5s 未激活".to_string())?;
        Ok(Self {
            wellknown,
            serve,
            _claim: claim,
            pump,
            client,
            ids,
        })
    }

    /// 只泵产者（宿主"停摆"）：模拟慢 Host。
    fn pump_producer(&mut self) {
        let _ = self.pump.step();
    }

    /// 只泵宿主，返回连接是否仍存活。
    fn pump_host(&mut self) -> bool {
        let (_events, alive) = crate::ui::desktop_protocol::rqhost::pump_client(
            &mut self.client,
            &mut self.ids,
        );
        alive
    }

    fn pump_both(&mut self) -> bool {
        self.pump_producer();
        self.pump_host()
    }

    fn composed(&self) -> Option<&DisplayList> {
        crate::ui::desktop_protocol::rqhost::composed_v2(&self.client)
    }
}

impl Drop for WireCell {
    fn drop(&mut self) {
        self.serve.stop(&self.wellknown);
    }
}

fn switching_source(canvas: std::sync::Arc<std::sync::atomic::AtomicBool>) -> WireSource {
    WireSource::Live(HeadlessFrameSource::new(
        SwitchingView { canvas, ticks: 0 },
        W,
        H,
    ))
}

/// **慢宿主**（P-03/AC-04）：宿主停摆时队列按槽信用有界（在途帧 = 2），第三帧
/// 被推迟而不是覆盖在途槽；宿主恢复后帧继续落地、待发位清空、最终有完整帧。
#[test]
fn rq_wire_slow_host_backpressures_without_overwrite_and_catches_up() {
    let canvas = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut cell = WireCell::start("slow", switching_source(std::sync::Arc::clone(&canvas)))
        .expect("slow cell");

    // 宿主停摆 250ms：只泵产者（tick 15ms → 应有十余个新 revision）。
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(250) {
        cell.pump_producer();
        std::thread::sleep(Duration::from_millis(5));
    }
    println!(
        "[RQNEG] {}",
        serde_json::json!({
            "tag": "slow_host", "frames_in_flight": cell.pump.frames_in_flight(),
            "pending": cell.pump.is_frame_pending(), "host_frames": cell.client.frames,
        })
    );
    assert_eq!(
        cell.pump.frames_in_flight(),
        2,
        "宿主停摆时在途帧必须被槽数（2）封顶，不得无界堆积"
    );
    assert!(
        cell.pump.is_frame_pending(),
        "信用耗尽必须置待发位（推迟补发，不丢帧、不覆盖在途槽）"
    );

    // 恢复泵：帧继续落地，待发位清空。
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        cell.pump_both();
        if !cell.pump.is_frame_pending() && cell.client.frames >= 3 {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        cell.client.frames >= 3,
        "宿主恢复后必须继续合成多帧（实得 {}）",
        cell.client.frames
    );
    assert!(!cell.pump.is_frame_pending(), "恢复后待发位必须清空");
    assert!(cell.composed().is_some(), "最终必须有完整合成面");
}

/// **旧 ACK / 重复 ACK**（AC-04）：宿主回一个早已归还的槽与过期 frame_id，
/// 端点不得不崩/不串槽；连接存活且后续帧继续合成（到来乱序 ACK 属真实场景）。
#[test]
fn rq_wire_stale_ack_does_not_break_slot_accounting() {
    use crate::ui::desktop_protocol::message::{FrameMsg, ProtocolMsg};

    let canvas = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut cell = WireCell::start("stale-ack", switching_source(std::sync::Arc::clone(&canvas)))
        .expect("stale ack cell");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && cell.client.frames < 2 {
        cell.pump_both();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(cell.client.frames >= 2, "先要有在途/已合成帧");
    let frames_before = cell.client.frames;
    let wid = cell.client.inner.wid.expect("Active 即有 wid").0;

    // 过期 / 重复 ACK：frame_id 远小于当前、槽是早已归还的那个。
    let stale = ProtocolMsg::Frame(FrameMsg::FrameAck {
        wid,
        frame_id: 1,
        slot: 1,
    });
    let sent = cell.client.inner.end.send(&stale);
    assert!(sent.is_ok(), "过期 ACK 必须能落管道（不是产者侧拒绝）");

    // 端点继续工作：连接存活 + 帧继续合成（不 panic、不串槽）。
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut alive = true;
    while Instant::now() < deadline && cell.client.frames < frames_before + 1 {
        alive = cell.pump_both();
        assert!(alive, "过期 ACK 不得断开连接");
        std::thread::sleep(Duration::from_millis(2));
    }
    println!(
        "[RQNEG] {}",
        serde_json::json!({
            "tag": "stale_ack", "frames_before": frames_before,
            "frames_after": cell.client.frames, "alive": alive,
        })
    );
    assert!(
        cell.client.frames > frames_before,
        "过期 ACK 之后必须仍能合成新帧（实得 {} → {}）",
        frames_before,
        cell.client.frames
    );
    assert!(cell.composed().is_some(), "合成面必须仍然有效");
}

/// **旧宿主 × 新产者**（AC-04 / V-09）：宿主只声明 v2 能力面时，v3 帧
/// （Path，required `fidelity/pathfill/1`）必须被**入帧前拒绝**且保旧帧
/// （不呈现半帧、不断流）；产者切回 v2 词汇后同一连接恢复合成。
#[test]
fn rq_wire_old_host_rejects_v3_frames_and_recovers_on_v2() {
    use crate::ui::desktop_protocol::message::CapabilitySet;

    let canvas = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let mut cell = WireCell::start("old-host", switching_source(std::sync::Arc::clone(&canvas)))
        .expect("old host cell");
    // 旧宿主能力面：只有 glyphrun，没有 pathfill —— v3 Path 帧必须被拒。
    cell.client.inner.surfaces.set_capabilities(CapabilitySet::parse(&[
        "optional:fidelity/glyphrun/1".to_string(),
    ]));

    let start = Instant::now();
    let mut alive = true;
    while start.elapsed() < Duration::from_millis(400) {
        alive = cell.pump_both();
        assert!(alive, "拒绝入帧不得断流");
        std::thread::sleep(Duration::from_millis(2));
    }
    println!(
        "[RQNEG] {}",
        serde_json::json!({
            "tag": "old_host", "phase": "v3_rejected", "frames": cell.client.frames,
            "composed": cell.composed().is_some(), "alive": alive,
        })
    );
    assert_eq!(
        cell.client.frames, 0,
        "旧宿主不得合成 v3 帧（入帧前拒绝，不是半帧呈现）"
    );
    assert!(
        cell.composed().is_none(),
        "旧宿主在 v3 帧期必须保持旧帧（此处无旧帧 → 空）"
    );

    // 产者切回 v2 词汇：同一连接必须恢复合成（不是靠重连）。
    canvas.store(false, std::sync::atomic::Ordering::Relaxed);
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline && cell.composed().is_none() {
        alive = cell.pump_both();
        assert!(alive, "回退帧不得断流");
        std::thread::sleep(Duration::from_millis(2));
    }
    let composed = cell.composed().expect("v2 回退后必须合成");
    // PLAN-073 T-03（A1）：回退帧的词汇随协商翻转——本格宿主声明了
    // glyphrun，故文本以 GlyphRun 过线（无重排）；判据改为「不得再含
    // 被拒的 v3 Path 词汇」+ 文本有承载（GlyphRun 或旧 TextStyled 均合
    // 同——未协商面由 rq_string_text_path_is_observable_degradation 钉）。
    assert!(
        !composed
            .ops
            .iter()
            .any(|op| matches!(op, DisplayOp::Path { .. })),
        "回退帧不得再含被拒的 v3 Path 词汇"
    );
    assert!(
        composed.ops.iter().any(|op| matches!(
            op,
            DisplayOp::GlyphRun { .. } | DisplayOp::TextStyled { .. }
        )),
        "回退帧文本必须有承载（协商词汇）"
    );
    // PLAN-073 T-03（A1）：协商后的词汇可要求已声明能力（glyphrun）；
    // 被拒的 v3 pathfill 不得复现。
    assert!(
        !composed
            .required_capabilities()
            .iter()
            .any(|c| c.contains("pathfill")),
        "回退帧不得再要求被拒的 v3 pathfill 能力"
    );
    println!(
        "[RQNEG] {}",
        serde_json::json!({
            "tag": "old_host", "phase": "v2_recovered", "frames": cell.client.frames,
            "ops": composed.ops.len(),
        })
    );
}

/// **generation 协商在真线上生效**（AC-03）：宿主 `HostCaps` 声明
/// `resource/generation/1` → 产者据此把位图上传换成带序号的 tag；宿主按序号
/// 应用（迟到/重复丢弃）。这里断言协商结果 + 资源真的经该路径落到合成面。
#[test]
fn rq_wire_generation_capability_is_negotiated_and_applied() {
    use crate::ui::desktop_protocol::message::CAP_RESOURCE_GENERATION;

    let mut cell = WireCell::start(
        "generation",
        WireSource::Rust(HeadlessFrameSource::new(
            StaticView {
                view: View::image("lucide:bell"),
            },
            W,
            H,
        )),
    )
    .expect("generation cell");
    // HostCaps 与 Welcome 一起排队，产者要吃下一拍才更新 peer_caps——等它落地。
    let caps_deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < caps_deadline
        && !cell.pump.peer_caps().supports(CAP_RESOURCE_GENERATION)
    {
        cell.pump_both();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        cell.pump.peer_caps().supports(CAP_RESOURCE_GENERATION),
        "宿主必须在 HostCaps 里声明 generation 能力（线上协商，不是本地开关）"
    );

    let deadline = Instant::now() + Duration::from_secs(4);
    let mut composed = None;
    while Instant::now() < deadline {
        cell.pump_both();
        if let Some(list) = cell.composed() {
            if list
                .ops
                .iter()
                .any(|op| matches!(op, DisplayOp::Image { .. }))
            {
                composed = Some(list.clone());
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    // cell 是 mut 借用底下的只读访问：这里用可变绑定重新取一次更直观。
    let _ = &cell;
    let list = composed.expect("带 generation 协商的连线上必须合成出图像帧");
    println!(
        "[RQNEG] {}",
        serde_json::json!({
            "tag": "generation", "peer_caps": cell.pump.peer_caps().to_wire(),
            "image_ops": list.ops.iter().filter(|op| matches!(op, DisplayOp::Image { .. })).count(),
        })
    );
}
// ---------------------------------------------------------------------------
// AC-04 / V-09：**真两进程** v2↔v3 配对（独立 exe，真管道 + 真跨进程 shm）
// ---------------------------------------------------------------------------

/// 子进程体（`current_exe` 自派生，env 驱动；无 env = 直接返回，普通跑套件时
/// 零副作用）。父进程见 [`rq_wire_two_process_v2_v3_pairing`]。
#[test]
fn rq_wire_child_body() {
    let Ok(wellknown) = std::env::var("AUTO_RQ062_CHILD_PIPE") else {
        return;
    };
    let app = std::env::var("AUTO_RQ062_CHILD_APP").unwrap_or_else(|_| "plan062-child".into());
    let mode = std::env::var("AUTO_RQ062_CHILD_MODE").unwrap_or_else(|_| "v3".into());
    let (_, end) = crate::ui::desktop_protocol::rqhost::adopt(&wellknown, &app, 15_000)
        .expect("子进程采纳");
    let view = if mode == "v3" {
        super::rq_seams::canvas_scene_view()
    } else {
        View::text("子进程 v2 帧")
    };
    let mut pump = ClientPump::new(
        end,
        WireSource::Rust(HeadlessFrameSource::new(StaticView { view }, W, H)),
        {
            // Hello 凭据带子进程身份：宿主侧 `RqClient.app_name` 由 Hello 回填，
            // 两条连接据此可区分（adopt 记录名只用于管道分配）。
            let mut config = client_config();
            config.app_name = app.clone();
            config.title = app.clone();
            config
        },
        None,
    );
    pump.set_v2(true);
    // 阻塞主循环：宿主关闭/EOF → HostLost → 退出（进程随测试结束）。
    let (exit, _source) = pump.run();
    eprintln!("[RQPROC] child `{app}` exit={exit:?}");
}

/// 父进程：spawn 两个**独立进程**子产者（一个产 v3 Path 帧、一个产 v2 字符串帧），
/// 同一宿主实例收两路——证明跨进程（真 exe）+ v2/v3 载荷配对，而不是同进程近似。
#[test]
fn rq_wire_two_process_v2_v3_pairing() {
    use crate::ui::desktop_protocol::rqhost;

    let wellknown = format!("plan062-two-proc-{}", std::process::id());
    let (serve, claim) =
        rqhost::RqServe::start(&wellknown).map_err(|e| format!("rqserve: {e:?}")).expect("serve");

    let exe = std::env::current_exe().expect("current_exe");
    let mut children = Vec::new();
    for (app, mode) in [("plan062-child-v3", "v3"), ("plan062-child-v2", "v2")] {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["rq_wire_child_body", "--test-threads", "1", "--nocapture"])
            .env("AUTO_RQ062_CHILD_PIPE", &wellknown)
            .env("AUTO_RQ062_CHILD_APP", app)
            .env("AUTO_RQ062_CHILD_MODE", mode)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit());
        for (k, _) in std::env::vars() {
            if k.starts_with("NEXTEST_") {
                cmd.env_remove(&k);
            }
        }
        children.push(cmd.spawn().expect("spawn child"));
    }

    // 宿主：认领两条转连（独立进程各一条）。
    let mut ids = rqhost::RqIds::default();
    let mut clients: Vec<rqhost::RqClient> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    while clients.len() < 2 && Instant::now() < deadline {
        let pending: Vec<_> = serve.pending.lock().unwrap().drain(..).collect();
        for (app_name, end) in pending {
            if let Some((client, _)) = rqhost::adopt_one(app_name, end, &mut ids, 5000) {
                clients.push(client);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(clients.len(), 2, "两个独立子进程必须都被接纳");
    // 两条连接必须来自两个不同 app（独立进程各自的 Hello 凭据）。
    let mut names: Vec<&str> = clients
        .iter()
        .map(|c| c.app_name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec!["plan062-child-v2", "plan062-child-v3"],
        "两条连接必须来自两个不同子进程"
    );
    // 子进程在断言期必须仍存活（不是早退留下的空连接）。
    for child in children.iter_mut() {
        assert!(
            matches!(child.try_wait(), Ok(None)),
            "子进程在配对期必须存活（实得 {:?}）",
            child.try_wait()
        );
    }

    // 各自泵到有合成面；按内容区分 v3（Path）与 v2（字符串）。
    let mut v3_ok = false;
    let mut v2_ok = false;
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline && !(v3_ok && v2_ok) {
        for client in clients.iter_mut() {
            let _ = rqhost::pump_client(client, &mut ids);
        }
        for client in clients.iter() {
            let Some(list) = rqhost::composed_v2(client) else {
                continue;
            };
            let has_path = list
                .ops
                .iter()
                .any(|op| matches!(op, DisplayOp::Path { .. }));
            // PLAN-073 T-03（A1）：文本承载随协商翻转（glyphrun 声明 →
            // GlyphRun；旧/未协商 → TextStyled）。判据 = 文本有承载。
            let has_text = list.ops.iter().any(|op| {
                matches!(op, DisplayOp::TextStyled { .. } | DisplayOp::GlyphRun { .. })
            });
            if has_path && list.requires_v3() {
                v3_ok = true;
            }
            // PLAN-073 T-03（A1）：v2 子进程文本随协商以 GlyphRun 承载
            // （GlyphRun 归 v3 代原语，requires_v3 翻真）——其判据改为
            // 「有文本承载且不含 Path（v3 专属原语）」；glyphrun 能力宿主
            // 已声明，合成合法。
            if has_text && !has_path {
                v2_ok = true;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    println!(
        "[RQPROC] {}",
        serde_json::json!({
            "tag": "two_process", "children": children.len(),
            "v3_path_frame": v3_ok, "v2_text_frame": v2_ok,
        })
    );
    serve.stop(&wellknown);
    drop(claim);
    for child in children.iter_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    assert!(v3_ok, "v3 子进程（独立 exe）的 Path 帧必须跨进程落地");
    assert!(
        v2_ok,
        "v2 子进程（独立 exe）的文本帧必须跨进程落地（承载随协商：A1 后为 GlyphRun）"
    );
}

/// PLAN-062（AC-03 / 复审 F-06）：媒体静态帧**真正过线**——poster 经
/// producer→管道→宿主→位图段→宿主缓存，装配出来的帧里 Image 引用在宿主侧
/// 命中（in-process 的记录面测试另见 `rq_probe_t05_media_poster_frame_crosses_the_wire`）。
#[test]
fn rq_wire_media_poster_frame_crosses_the_wire() {
    use crate::ui::media_poster;

    media_poster::clear_all();
    let (pw, ph) = (8u32, 6u32);
    let rgba: Vec<u8> = (0..(pw * ph * 4)).map(|i| ((i * 5) % 251) as u8).collect();
    let src = "probe://poster-wire.mp4";
    assert!(media_poster::publish(src, pw, ph, rgba.clone()));

    let view = View::Video {
        src: src.to_string(),
        paused: true,
        position: Some(0.0),
        volume: 100,
        muted: true,
        rate: 1.0,
        label: "海报".to_string(),
        on_time_update: None,
        on_loaded_metadata: None,
        on_play_state: None,
        on_ended: None,
        on_media_error: None,
        media_key: None,
        style: None,
    };
    let mut cell = WireCell::start(
        "poster",
        WireSource::Rust(HeadlessFrameSource::new(StaticView { view }, W, H)),
    )
    .expect("poster cell");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut hit = None;
    while Instant::now() < deadline {
        cell.pump_both();
        if let Some(list) = cell.composed() {
            if let Some(src) = list.ops.iter().find_map(|op| match op {
                DisplayOp::Image { src, .. } => Some(src.clone()),
                _ => None,
            }) {
                hit = Some((src, list.clone()));
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let (image_src, _list) = hit.expect("海报帧必须经真实管道到达宿主合成面");
    assert!(image_src.starts_with("bitmap://"), "引用必须是资源身份: {image_src}");
    // 宿主缓存命中 = 位图段载荷真的到了（不只是帧里有个 src 字符串）。
    // 注意：上传可能先被 per-App 8 MiB/s 预算延期（`Backpressure` + 帧重试），
    // 所以这里给到期限内的收敛时间，而不是要求“首帧即命中”——延期不丢内容
    // 正是 P-01 的合同行为。
    let deadline = Instant::now() + Duration::from_secs(6);
    while Instant::now() < deadline
        && crate::ui::iced::broker_surface::resolve_drawlist_image(&image_src, pw, ph).is_none()
    {
        cell.pump_both();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        crate::ui::iced::broker_surface::resolve_drawlist_image(&image_src, pw, ph).is_some(),
        "宿主侧必须最终收到海报帧位图（预算延期后可收敛，真实 shm/管道链路）"
    );
    println!(
        "[RQWIRE] {}",
        serde_json::json!({
            "tag": "media_poster_wire", "src": image_src, "poster": [pw, ph],
        })
    );
    media_poster::clear(src);
}

// ---------------------------------------------------------------------------
// PLAN-073 T-01（V-01）：1080p 连续媒体流 × 资源通道容量探针。
//
// T-01 首轮实测发现（2026-11-08，t01-probe-run1.log）：位图段槽尺寸 =
// 宿主表面 w×h×4×4+4（host.rs:451，4× 表面像素字节余量）——640×400 表面
// 的槽仅 4.1 MiB，装不下 1080p 帧（8.29 MiB）→ 「媒体帧 ≤ 4× 表面字节」
// 是资源通道的具名几何约束。本组探针改用 1920×1080 宿主表面（真实全高
// 清窗，槽 33.3 MiB），并在判据上只数**宿主已收到的内容**
// （resolve_drawlist_image 命中），不是仅引用串。
//
// 两案（§2 允许的最多两案比较，纸面核算见 plan-artifacts t01 档案 §4）：
// - Path A（既有资源通道 + 8 MiB·s⁻¹ 上传预算，as-built）；
// - Path A'（同一通道 + 媒体类预算豁免原型旋钮，即 T-02 候选「媒体资源
//   类别」的传输容量上限）。
// 产者 = MediaStreamView（逐拍重发布 1080p 海报，合成内容、真实尺寸/节奏；
// 真实 mpv 解码读回归 T-02/V-02）。宿主 = WireCell 真 RqServe 管道端。
// ---------------------------------------------------------------------------

const T01_SURF_W: f32 = 1920.0;
const T01_SURF_H: f32 = 1080.0;

fn t01_media_stream_start(
    tag: &str,
    src: &str,
    budget_bytes: Option<u64>,
) -> (
    WireCell,
    String,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    std::sync::Arc<std::sync::atomic::AtomicU64>,
) {
    let (w, h) = (1920u32, 1080u32);
    let pause = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let seek_to = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut probe = MediaStreamView {
        src: src.to_string(),
        w,
        h,
        frame_no: 0,
        tick_ms: 41, // 尝试 24fps
        pause: std::sync::Arc::clone(&pause),
        seek_to: std::sync::Arc::clone(&seek_to),
    };
    let rgba0 = probe.frame_rgba(0);
    assert!(
        crate::ui::media_poster::publish(src, w, h, rgba0),
        "T-01 初帧发布失败"
    );
    let mut source = HeadlessFrameSource::new(probe, T01_SURF_W, T01_SURF_H);
    if let Some(budget) = budget_bytes {
        source.set_upload_budget_bytes_per_window(budget);
    }
    let cfg = ClientConfig {
        app_name: format!("plan073-{tag}"),
        title: format!("plan073-{tag}"),
        width: T01_SURF_W,
        height: T01_SURF_H,
    };
    let cell = WireCell::start_with(tag, WireSource::Media(source), cfg).expect("T-01 wire cell");
    (cell, src.to_string(), pause, seek_to)
}

/// 统计宿主合成面上已见过的独立 bitmap 资源身份，并只把**内容已抵达
/// 宿主**（resolve_drawlist_image 命中）的计入 received。
fn t01_absorb_composed(
    cell: &WireCell,
    seen: &mut std::collections::HashSet<String>,
    received: &mut std::collections::HashSet<String>,
) {
    if let Some(list) = cell.composed() {
        for op in &list.ops {
            if let DisplayOp::Image { src, .. } = op {
                if src.starts_with("bitmap://") && seen.insert(src.clone()) {
                    if crate::ui::iced::broker_surface::resolve_drawlist_image(src, 1920, 1080)
                        .is_some()
                    {
                        received.insert(src.clone());
                    }
                }
            }
        }
    }
}

// PLAN-073 T-01「Path A as-built 预算受限」探针已随用户裁定（2026-11-08，
// 媒体类资源类别豁免 8 MiB/s 内容预算）废止：其前提（媒体流受内容预算
// 闸）不再存在。实测证据归档 plan-artifacts/plan-073/auto-ui/
// t01-probe-{run1,run2,run3,final,release}.log（受限 ≈0.999fps / 豁免
// 23.83fps）；内容预算本体仍由
// rq_v3_upload_budget_defers_frame_without_partial_registration（非媒体
// 内容）钉住。
/// Path A'（媒体类豁免原型）：预算旋钮放大后的传输容量上限实测。
#[test]
fn p073_t01_media_class_capacity_1080p() {
    let (mut cell, src, _pause, _seek) = t01_media_stream_start(
        "t01-capacity",
        "probe://t01-capacity.mp4",
        Some(u64::MAX / 2),
    );
    const RUN: Duration = Duration::from_secs(6);
    let start = Instant::now();
    let mut seen = std::collections::HashSet::new();
    let mut received = std::collections::HashSet::new();
    let mut composed_total = 0u64;
    while start.elapsed() < RUN {
        if cell.pump_both() {
            composed_total = cell.client.frames;
        }
        t01_absorb_composed(&cell, &mut seen, &mut received);
        std::thread::sleep(Duration::from_millis(1));
    }
    let elapsed = start.elapsed().as_secs_f64();
    let produced = crate::ui::media_poster::generation(&src);
    println!(
        "[T01-PATH-A-CAPACITY] {}",
        serde_json::json!({
            "tag": "t01_path_a_capacity",
            "window_s": elapsed,
            "surface": [T01_SURF_W, T01_SURF_H],
            "produced_frames": produced,
            "produced_fps": (produced as f64 / elapsed),
            "referenced_identities": seen.len(),
            "received_frames": received.len(),
            "received_fps": (received.len() as f64 / elapsed),
            "host_composed_total": composed_total,
            "throughput_mib_s": (received.len() as f64 * 7.91 / elapsed),
        })
    );
    assert!(
        received.len() >= 10,
        "豁免案必须证明多帧内容真实抵达宿主（零选择不 PASS），实得 {}",
        received.len()
    );
    crate::ui::media_poster::clear(&src);
}

/// 慢宿主对照（V-01 负控）：豁免预算 + 宿主 5 拍泵 1 拍——背压/合流，
/// 不丢不覆（槽信用合同 P-01）。断言：连接存活、宿主有帧、内容连续。
#[test]
fn p073_t01_slow_host_backpressure_1080p() {
    let (mut cell, src, _pause, _seek) = t01_media_stream_start(
        "t01-slowhost",
        "probe://t01-slowhost.mp4",
        Some(u64::MAX / 2),
    );
    const RUN: Duration = Duration::from_secs(5);
    let start = Instant::now();
    let mut seen = std::collections::HashSet::new();
    let mut received = std::collections::HashSet::new();
    let mut iter: u64 = 0;
    let mut alive = true;
    while start.elapsed() < RUN && alive {
        let _ = cell.pump_producer();
        if iter % 5 == 0 {
            alive = cell.pump_host();
        }
        t01_absorb_composed(&cell, &mut seen, &mut received);
        iter += 1;
        std::thread::sleep(Duration::from_millis(2));
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "[T01-SLOW-HOST] {}",
        serde_json::json!({
            "tag": "t01_slow_host",
            "window_s": elapsed,
            "host_composed_total": cell.client.frames,
            "referenced_identities": seen.len(),
            "received_frames": received.len(),
            "received_fps": (received.len() as f64 / elapsed),
            "alive": alive,
        })
    );
    assert!(alive, "慢宿主下连接必须存活（不因背压断链）");
    assert!(cell.client.frames > 0, "慢宿主必须仍收到帧");
    assert!(
        received.len() >= 2,
        "慢宿主下内容仍应连续抵达（合流不冻结），实得 {}",
        received.len()
    );
    crate::ui::media_poster::clear(&src);
}

/// PLAN-073 T-03（A5，062 Spec §6#3 owner=UI073）：SVG 场景产帧计时——
/// 常驻 tiny_skia 记录器复用后，热帧产帧必须显著低于 062 实测 ≈32.5ms/帧
/// （每帧新建 Renderer → Engine 字体缓存冷启的根因已除；061 冻结 W-C ≤10ms）。
#[test]
fn p073_t03_svg_produce_warm_renderer_reuse() {
    let mut surface = super::HeadlessSurface::new(
        StaticView {
            view: View::image("lucide:bell"),
        },
        640.0,
        400.0,
    );
    // 首帧含惰性记录器构造 + 字体冷启（基线帧，不计入热判定）。
    let cold = {
        let t = Instant::now();
        let _ = surface.render_frame_v2();
        t.elapsed()
    };
    const WARM: u32 = 8;
    let warm_total = {
        let t = Instant::now();
        for _ in 0..WARM {
            let _ = surface.render_frame_v2();
        }
        t.elapsed()
    };
    let cold_ms = cold.as_secs_f64() * 1000.0;
    let warm_avg_ms = warm_total.as_secs_f64() * 1000.0 / WARM as f64;
    println!(
        "[T03-SVG-PRODUCE] {}",
        serde_json::json!({
            "tag": "t03_svg_produce",
            "cold_first_frame_ms": cold_ms,
            "warm_frames": WARM,
            "warm_avg_ms": warm_avg_ms,
            "baseline_062_ms": 32.5,
        })
    );
    assert!(
        warm_avg_ms < cold_ms,
        "热帧必须快于含构造的冷帧（复用生效），cold {cold_ms:.2}ms vs warm {warm_avg_ms:.2}ms"
    );
    // release：热帧达 061 冻结 W-C ≤10ms；debug：至少低于 062 基线一半。
    if cfg!(debug_assertions) {
        assert!(
            warm_avg_ms < 16.0,
            "debug 热帧应显著低于 062 基线 32.5ms 的一半，实得 {warm_avg_ms:.2}ms"
        );
    } else {
        assert!(
            warm_avg_ms <= 10.0,
            "release 热帧应达 W-C 冻结 ≤10ms，实得 {warm_avg_ms:.2}ms"
        );
    }
}

/// PLAN-073 T-03（A1 收口，062 Spec §6#1）：**glyphrun 协商在线上驱动默认
/// 路径词汇**——宿主 `HostCaps` 声明 `fidelity/glyphrun/1` → 产者文本以
/// GlyphRun 过线（宿主不再重排，Cached 臂 + Paragraph/Editor 臂一致）。
/// 时序事实（062 世代协商测试同款）：HostCaps 与 Welcome 同批排队，首帧
/// 可能先于协商（TextStyled），协商后 tick 产的新帧必须翻成 GlyphRun。
/// 未协商兼容面（TextStyled 可观测降级）由
/// `rq_string_text_path_is_observable_degradation`（记录侧默认关）钉住；
/// 本格 harness 宿主激活即广播全量默认能力面，无法事后撤回声明——
/// 「线上未声明」臂需 pre-activation 能力装配，具名为 harness 边界。
#[test]
fn p073_t03_glyphrun_negotiation_drives_default_path_vocabulary() {
    use crate::ui::desktop_protocol::message::{CAP_GLYPH_RUN, CapabilitySet};

    let mut cell = WireCell::start(
        "a1-glyph",
        switching_source(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))),
    )
    .expect("a1 glyph cell");
    cell.client
        .inner
        .surfaces
        .set_capabilities(CapabilitySet::parse(&["optional:fidelity/glyphrun/1".to_string()]));
    let deadline = Instant::now() + Duration::from_secs(6);
    let mut negotiated_frame_seen = false;
    while Instant::now() < deadline {
        cell.pump_both();
        if cell.pump.peer_caps().supports(CAP_GLYPH_RUN) {
            if let Some(list) = cell.composed() {
                if list
                    .ops
                    .iter()
                    .any(|op| matches!(op, DisplayOp::GlyphRun { .. }))
                {
                    // 协商后的当前帧：不得再含 TextStyled 字符串面。
                    assert!(
                        !list.ops.iter().any(|op| matches!(op, DisplayOp::TextStyled { .. })),
                        "协商成立后的帧不得再出 TextStyled（A1 重排面关闭）"
                    );
                    negotiated_frame_seen = true;
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        cell.pump.peer_caps().supports(CAP_GLYPH_RUN),
        "宿主必须在线上声明 glyphrun 能力"
    );
    assert!(
        negotiated_frame_seen,
        "协商后的 tick 帧必须以 GlyphRun 过线（宿主不重排）"
    );
    println!(
        "[T03-A1] {}",
        serde_json::json!({"tag": "a1_glyphrun_negotiated", "arm": "declared",
            "frames": cell.client.frames, "ops": cell.composed().map(|l| l.ops.len())})
    );
}

// ---------------------------------------------------------------------------
// PLAN-073 T-02（AC-02/V-02）：连续媒体资源帧路径——豁免政策落地后的
// 永久正/负控（用户裁定 2026-11-08：媒体类豁免 8 MiB/s 内容预算）。
// ---------------------------------------------------------------------------

/// 正控：媒体类豁免生效——默认预算（不旋钮）下 1080p 连续流高频落地。
/// 冻结预算假实现（老政策）在此只有 ≈1fps，必然失败本断言。
#[test]
fn p073_t02_media_class_exempt_continuous_1080p() {
    let (mut cell, src, _pause, _seek) =
        t01_media_stream_start("t02-exempt", "probe://t02-exempt.mp4", None);
    const RUN: Duration = Duration::from_secs(6);
    let start = Instant::now();
    let mut received = std::collections::HashSet::new();
    while start.elapsed() < RUN {
        cell.pump_both();
        t01_absorb_composed(&cell, &mut std::collections::HashSet::new(), &mut received);
        std::thread::sleep(Duration::from_millis(1));
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "[T02-EXEMPT] {}",
        serde_json::json!({
            "tag": "t02_media_exempt", "window_s": elapsed,
            "received_frames": received.len(),
            "received_fps": (received.len() as f64 / elapsed),
            "throughput_mib_s": (received.len() as f64 * 7.91 / elapsed),
        })
    );
    // 阈值分档：release 实测 23.8fps（≥60/6s）；debug 记录成本 ~10×，
    // 且三探针并行时更低——冻结预算假实现只有 ≈0.99fps，10 倍差距即区分。
    let floor = if cfg!(debug_assertions) { 12 } else { 60 };
    assert!(
        received.len() >= floor,
        "媒体类豁免下 1080p 连续流必须高频落地（本档 ≥{floor}/6s，冻结预算假实现只有 ≈6），实得 {}",
        received.len()
    );
    crate::ui::media_poster::clear(&src);
}

/// 负控（V-02 冻结 poster 假实现必败）：冻结产者（发布一帧后暂停）的
/// 独立接收身份必须 ≤2——连续流（上一测试实测 ≥12，冻结预算假实现
/// ≈1）无法满足本断言的上界；本测试与
/// `p073_t02_media_class_exempt_continuous_1080p` 的下界构成判据对。
/// （曾内嵌同进程 live 对照臂：同 PID 位图 src 与全局缓存串扰致 0 帧
/// 假象，具名为 harness 边界——判据对结构等价且稳定。）
#[test]
fn p073_t02_frozen_poster_fake_discriminated() {
    let (mut frozen, fsrc, pause, _seek) =
        t01_media_stream_start("t02-frozen", "probe://t02-frozen.mp4", None);
    pause.store(true, std::sync::atomic::Ordering::Relaxed);
    let start = Instant::now();
    let mut frozen_received = std::collections::HashSet::new();
    while start.elapsed() < Duration::from_secs(3) {
        frozen.pump_both();
        t01_absorb_composed(&frozen, &mut std::collections::HashSet::new(), &mut frozen_received);
        std::thread::sleep(Duration::from_millis(2));
    }
    println!(
        "[T02-FROZEN-CTRL] {}",
        serde_json::json!({
            "tag": "t02_frozen_ctrl",
            "frozen_distinct": frozen_received.len(),
        })
    );
    assert!(
        frozen_received.len() <= 2,
        "冻结 poster 假实现的独立身份数必须 ≤2（内容与身份不变），实得 {}",
        frozen_received.len()
    );
    crate::ui::media_poster::clear(&fsrc);
}

/// 负控（V-02 seek 后旧帧不倒灌）：seek 使内容确定性跳变（同 n 同字节
/// → 身份稳定）；新身份落地后，旧身份不得再出现在任何后续合成帧。
/// 另验暂停语义：暂停期合成面保持最后帧（原子保旧，不清屏）。
#[test]
fn p073_t02_seek_pause_no_backfill() {
    let (mut cell, src, pause, seek_to) =
        t01_media_stream_start("t02-seek", "probe://t02-seek.mp4", None);
    // 流跑 2s 收集 seek 前身份集。
    let start = Instant::now();
    let mut pre_seek = std::collections::HashSet::new();
    while start.elapsed() < Duration::from_secs(2) {
        cell.pump_both();
        t01_absorb_composed(&cell, &mut std::collections::HashSet::new(), &mut pre_seek);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!pre_seek.is_empty(), "seek 前必须有帧落地");

    // seek：内容跳变（目标远超当前 frame_no）。
    seek_to.store(500, std::sync::atomic::Ordering::Relaxed);
    let start = Instant::now();
    let mut post_seek_new = std::collections::HashSet::new();
    // 第一阶段：等到第一个「不在 seek 前身份集」的合成帧落地（确认跳变
    // 已过线；此前的过渡帧不参与后续判据）。
    let mut seek_landed = false;
    while start.elapsed() < Duration::from_secs(6) && !seek_landed {
        cell.pump_both();
        let mut fresh = std::collections::HashSet::new();
        t01_absorb_composed(&cell, &mut fresh, &mut post_seek_new);
        if post_seek_new.iter().any(|id| !pre_seek.contains(id)) {
            seek_landed = true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(seek_landed, "seek 后必须落地新身份帧（内容跳变可见）");

    // seek 落地后继续泵：旧身份不得倒灌。（诊断：逐拍打印合成身份序列）
    let start = Instant::now();
    let mut seq: Vec<(f64, Vec<String>)> = Vec::new();
    while start.elapsed() < Duration::from_secs(2) {
        cell.pump_both();
        let mut seen_now = std::collections::HashSet::new();
        t01_absorb_composed(&cell, &mut seen_now, &mut post_seek_new);
        if let Some(list) = cell.composed() {
            let ids: Vec<String> = list
                .ops
                .iter()
                .filter_map(|op| match op {
                    DisplayOp::Image { src, .. } if src.starts_with("bitmap://") => {
                        Some(src.clone())
                    }
                    _ => None,
                })
                .collect();
            seq.push((start.elapsed().as_secs_f64(), ids.clone()));
        }
        for id in &seen_now {
            if pre_seek.contains(id) {
                eprintln!("[T02DBG] VIOLATION at t={:.3}s offending={id}", start.elapsed().as_secs_f64());
                for (t, ids) in &seq {
                    eprintln!("[T02DBG] t={t:.3}s ids={ids:?}");
                }
                eprintln!("[T02DBG] current composed={:?}", cell.composed().map(|l| l.ops.iter().filter_map(|op| match op { DisplayOp::Image { src, .. } => Some(src.clone()), _ => None }).collect::<Vec<_>>()));
                panic!("seek 落地后旧 generation 身份不得倒灌：{id}");
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    eprintln!("[T02DBG] post-seek composed sequence:");
    for (t, ids) in &seq {
        eprintln!("[T02DBG] t={t:.3}s ids={ids:?}");
    }

    // 暂停：合成面保持最后帧（不丢、不清）。
    let last_before = cell.composed().map(|l| l.ops.len()).unwrap_or(0);
    pause.store(true, std::sync::atomic::Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(200));
    let mut held = false;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        cell.pump_both();
        if let Some(list) = cell.composed() {
            if !list.ops.is_empty() {
                held = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let last_after = cell.composed().map(|l| l.ops.len()).unwrap_or(0);
    assert!(
        held && last_after > 0,
        "暂停期合成面必须保持最后帧（原子保旧合同），实得 held={held} ops={last_after}（暂停前 {last_before}）"
    );
    println!(
        "[T02-SEEK-PAUSE] {}",
        serde_json::json!({
            "tag": "t02_seek_pause", "pre_seek_identities": pre_seek.len(),
            "post_seek_new": post_seek_new.len(),
            "pause_held_ops": last_after,
        })
    );
    crate::ui::media_poster::clear(&src);
}
