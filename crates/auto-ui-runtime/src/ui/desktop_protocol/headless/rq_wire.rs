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
}

macro_rules! wire_delegate {
    ($this:ident, $method:ident $(, $arg:expr)*) => {
        match $this {
            WireSource::Rust(s) => s.$method($($arg),*),
            WireSource::Vm(s) => s.$method($($arg),*),
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
    // AC-02 默认 producer 门（阈值 2/255）：两种产者 × 两 Host 全部 ≤0.1%。
    for row in cells {
        let diff = row["pixel_diff_vs_direct"].as_f64().unwrap_or(1.0);
        assert!(
            diff <= 0.001,
            "wire 格 {} / {} 像素差 {:.5} 超 0.1% 门（AC-02 r4）",
            row["cell"],
            row["case"],
            diff
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
