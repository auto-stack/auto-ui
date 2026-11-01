//! PLAN-014 T-06：订阅/任务责任（AppTickRecipe/frame pump、panic probe/exit generation、toast/MCP/shell 桥/http/keyboard/hotkey）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;

/// 每 `interval_ms` 产一条 `DM::App(app, msg)` 的订阅。
enum AppTickKind {
    /// 固定事件名 tick（hot reload / widget tick / timer / toast / heartbeat）。
    Event(&'static str, u64),
    /// 每拍轮询一次进程级通道（fn 指针读全局静态，不捕获）。
    Poll(fn() -> Option<IcedMessage>, u64),
    /// PLAN-765 T-02：HTTP 消息桥回填泵——Poll 同族，但产出映射为
    /// `DM::HttpBackfill`（update 侧 seq 门控派发，见 DesktopMessage 文档）。
    PollBackfill(fn() -> Option<IcedMessage>, u64),
    /// Plan 051 C7: timer 块条目——按 (widget, event) 命名空间派发。
    WidgetEvent(String, String, u64),
}

struct AppTickRecipe {
    app: crate::ui::session::AppId,
    kind: AppTickKind,
}

impl std::hash::Hash for AppTickRecipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // 订阅身份：AppId + 事件名/函数指针 + 间隔（453 T4 教训：同间隔
        // 订阅在 iced 订阅表按 hash 去重，后注册者静默丢失 —— 事件名/函数
        // 指针参与 hash 保证互不去重）。
        std::mem::discriminant(&self.kind).hash(state);
        self.app.hash(state);
        match &self.kind {
            AppTickKind::Event(ev, ms) => (*ev, *ms).hash(state),
            AppTickKind::Poll(f, ms) => (*f as usize, *ms).hash(state),
            AppTickKind::PollBackfill(f, ms) => (*f as usize, *ms).hash(state),
            AppTickKind::WidgetEvent(w, ev, ms) => (w, ev, *ms).hash(state),
        }
    }
}

impl iced_futures::subscription::Recipe for AppTickRecipe {
    type Output = crate::ui::session::DesktopMessage;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        std::hash::Hash::hash(self, state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        use crate::ui::session::DesktopMessage as DM;
        use iced_futures::futures::stream::StreamExt;

        let (event, poll, backfill, ms, widget_event) = match self.kind {
            AppTickKind::Event(ev, ms) => (Some(ev), None, false, ms, None),
            AppTickKind::Poll(f, ms) => (None, Some(f), false, ms, None),
            AppTickKind::PollBackfill(f, ms) => (None, Some(f), true, ms, None),
            AppTickKind::WidgetEvent(w, ev, ms) => (None, None, false, ms, Some((w, ev))),
        };
        let app = self.app;
        let start = tokio::time::Instant::now() + std::time::Duration::from_millis(ms);
        let mut interval = tokio::time::interval_at(start, std::time::Duration::from_millis(ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        iced_futures::futures::stream::unfold(
            (interval, app, event, poll, backfill, widget_event),
            move |(mut iv, app, event, poll, backfill, widget_event)| async move {
                let _ = iv.tick().await;
                let msg = if let Some(ev) = event {
                    Some(IcedMessage {
                        widget: String::new(),
                        event: ev.to_string(),
                        input_value: None,
                    })
                } else if let Some((w, ev)) = &widget_event {
                    Some(IcedMessage {
                        widget: w.clone(),
                        event: ev.clone(),
                        input_value: None,
                    })
                } else {
                    poll.expect("AppTickRecipe: poll or event must be set")()
                };
                // 空拍（无事件）以 `Some(None)` 项表示、由流级 filter_map 剔除
                // —— 若直接 yield `None`，unfold 语义是**流终止**（Poll 变体
                // 首个空轮询即死，459 实测 MCP 动作全丢）。
                // PLAN-765：backfill 档产 `DM::HttpBackfill`（seq 门控派发）。
                Some((
                    msg.map(|m| {
                        if backfill {
                            DM::HttpBackfill(app, m)
                        } else {
                            DM::App(app, m)
                        }
                    }),
                    (iv, app, event, poll, backfill, widget_event),
                ))
            },
        )
        .filter_map(|msg| async move { msg })
        .boxed()
    }
}

pub(super) fn app_tick(
    app: crate::ui::session::AppId,
    event: &'static str,
    interval_ms: u64,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::from_recipe(AppTickRecipe {
        app,
        kind: AppTickKind::Event(event, interval_ms),
    })
}

/// PLAN-711 T-04 (D-1): 帧通知泵订阅——`listen_raw` 是唯一不过滤
/// RedrawRequested 的公开 API（`listen()` 显式丢弃 redraw；无 post-present
/// 回执，硬屏障需 fork iced_winit——计划约束禁止）。id 携带 AppId（订阅
/// 表按 hash 去重——453 T4 教训：同 id 后注册者静默丢失，per-app 身份
/// 必须进 id）。跨窗帧（多窗会话）会唤醒他窗 App 的泵——有界泵零任务时
/// 廉价 no-op，可接受；per-window 路由精化归 T-06余 观测面复核。
/// AUTO_SCHED_DIAG=1 时输出帧消息到达时间戳（R-1 序障实证：消费时刻
/// 必须 ≥ 该帧 present 返回时刻，与构建时间戳/截图三轴对读）。
pub(super) fn frame_pump_sub(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    use crate::ui::session::DesktopMessage as DM;
    iced_futures::subscription::filter_map((app, "plan711_frame_pump"), move |event| match event {
        iced_futures::subscription::Event::Interaction {
            event: iced::event::Event::Window(iced::window::Event::RedrawRequested(_)),
            ..
        } => {
            if std::env::var("AUTO_SCHED_DIAG").ok().as_deref() == Some("1") {
                let t0 = crate::ui::dynamic::sched_diag_t0();
                eprintln!(
                    "[SCHED-DIAG] frame_msg enqueue t={}ms app={:?}",
                    t0.elapsed().as_millis(),
                    app
                );
            }
            Some(DM::App(
                app,
                crate::ui::iced::IcedMessage {
                    widget: String::new(),
                    event: "__frame_pump".to_string(),
                    input_value: None,
                },
            ))
        }
        _ => None,
    })
}

/// PLAN-735 T-02（轴①——发放/送达节奏观测）：RedrawRequested 送达
/// 旁路观测订阅。**恒 None 不产消息**（filter_map 产出过滤——不进
/// runtime 通道，零自续帧面）；仅 AUTO_SCHED_DIAG=1 装配（订阅集
/// OnceLock 稳定）。与 `frame_pump_sub`（711 D-2 门控——仅 Init/CPU
/// 工作窗口在册）互补：本观测不受泵门控，RedrawRequested **送达**
/// 节奏轴（winit 合并后、iced present 臂内 broadcast）在全程可见
/// ——与 frame_msg enqueue（门控窗口内到达）分离对读。
pub(super) fn redraw_diag_sub(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::filter_map((app, "plan735_redraw_diag"), move |event| match event {
        iced_futures::subscription::Event::Interaction {
            event: iced::event::Event::Window(iced::window::Event::RedrawRequested(_)),
            ..
        } => {
            let t0 = crate::ui::dynamic::sched_diag_t0();
            eprintln!(
                "[SCHED-DIAG] redraw_deliver t={}ms app={:?}",
                t0.elapsed().as_millis(),
                app
            );
            None
        }
        _ => None,
    })
}

/// Plan 051 C7: timer 块条目订阅——每 `interval_ms` 产一条
/// `DM::App(app, IcedMessage{widget, event})`（update 侧 fire_timer 门控）。
pub(super) fn widget_event_tick(
    app: crate::ui::session::AppId,
    widget: &str,
    event: &str,
    interval_ms: u64,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::from_recipe(AppTickRecipe {
        app,
        kind: AppTickKind::WidgetEvent(widget.to_string(), event.to_string(), interval_ms),
    })
}

/// PLAN-650 E-3：hot_reload 轮询间隔。
/// `AUTOUI_HOT_RELOAD=0` → None（不订阅）；`=1` → 500ms；缺省 debug 500ms /
/// 非 debug 2000ms——浏览/静止场景显著降低 update→view 泵频率，开发热重载仍可用。
pub(super) fn hot_reload_interval_ms(debug_mode: bool) -> Option<u64> {
    match std::env::var("AUTOUI_HOT_RELOAD").as_deref() {
        Ok("0") => None,
        Ok("1") => Some(500),
        _ => Some(if debug_mode { 500 } else { 2000 }),
    }
}

/// Periodic tick subscription for widget .Tick handlers (per-App).
fn widget_tick(
    app: crate::ui::session::AppId,
    interval_ms: u32,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    app_tick(app, TICK_EVENT, interval_ms as u64)
}

/// Plan 459 T5：panic 注入探针（验收 §4-2 的"示例内置开关"）。
/// `AUTOUI_PANIC_PROBE=1` 时，demo 的 `.panic_probe` 事件在 update 边界
/// panic（落 catch_unwind），并记录肇事 App —— 其后的 view 出口仅对该
/// App 持续 panic（落崩溃页元素），另一窗口不受影响。非 demo 运行零影响。
pub(super) static PANIC_PROBE_CRASHED_APP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(super) fn panic_probe_enabled() -> bool {
    std::env::var("AUTOUI_PANIC_PROBE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// 探针事件匹配：DSL handler 名风格不一（`PanicProbe` / `panic_probe`），
/// 规范化为去下划线小写后比较。
pub(super) fn is_panic_probe_event(event: &str) -> bool {
    event.to_ascii_lowercase().replace('_', "") == "panicprobe"
}

/// PLAN-815 T-05（F03，R2 根修）：宿主退出请求代数（进程级单调计数）。
/// 嵌入式宿主（如 vm 会话子进程的 stdin 退出通道）调用
/// [`request_host_exit`] 递增；`run_session` 启动时快照当前代数，其订阅
/// 只在代数 **大于快照** 时收口事件循环（`iced::exit`）。代数单调递增、
/// 永不清零：会话启动前/初始化期间已到达的请求（代数 > 快照）不会丢，
/// 上一会话的陈旧请求（代数 ≤ 快照）不会误杀下一会话——R2 实测的
/// “启动清零吞掉本次请求”竞态在结构上不存在。CLI 直跑形态无人递增，
/// 零行为变化。
static HOST_EXIT_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 请求当前 iced 会话退出（递增退出请求代数；返回新代数）。
pub fn request_host_exit() -> u64 {
    HOST_EXIT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}

/// 当前退出请求代数（`run_session` 启动快照用）。
pub fn host_exit_generation() -> u64 {
    HOST_EXIT_GENERATION.load(std::sync::atomic::Ordering::SeqCst)
}

/// 当前退出窗口的启动快照（R3-02：窗口生命周期）。`u64::MAX` = 无窗口。
/// [`host_exit_window_begin`] 在 VM 执行的**最早边界**（内部执行臂启动/
/// `run_file_dynamic_ui_inner` 入口——组件构建之前）开启窗口；`run_session`
/// 消费快照，会话结束后关闭窗口（store MAX）。开启与消费之间的任何
/// 退出请求（含初始化/构建期间的 q）都 > 快照，不被吸收；上一会话的
/// 陈旧请求在新窗口开启时 ≤ 新快照，不误杀下一会话。
static CURRENT_SESSION_SINCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(u64::MAX);

pub(super) fn current_host_exit_since() -> u64 {
    CURRENT_SESSION_SINCE.load(std::sync::atomic::Ordering::SeqCst)
}

/// 开启本进程的退出请求窗口（PLAN-815 T-05 R3-02）：以当前代数为快照。
/// 幂等且**不抬高**已开启窗口的快照（窗口已开则保持原值）。在 VM 执行
/// 的最早边界调用——内部执行臂进程启动处、或 `run_file_dynamic_ui_inner`
/// 入口（组件构建前）。
pub fn host_exit_window_begin() {
    let now = HOST_EXIT_GENERATION.load(std::sync::atomic::Ordering::SeqCst);
    CURRENT_SESSION_SINCE
        .compare_exchange(
            u64::MAX,
            now,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .ok();
}

/// 会话结束：关闭退出请求窗口（下一会话开启新窗口时以彼时快照吸收
/// 陈旧请求）。
pub(super) fn host_exit_window_end() {
    CURRENT_SESSION_SINCE.store(u64::MAX, std::sync::atomic::Ordering::SeqCst);
}

/// Global MCP action channel receiver (Plan 278).
/// Set once at startup by `run_dynamic_iced`, polled by `mcp_action_subscription`.
pub(super) static MCP_ACTION_RX: std::sync::OnceLock<
    std::sync::Mutex<Option<std::sync::mpsc::Receiver<crate::ui::mcp_server::ActionMessage>>>,
> = std::sync::OnceLock::new();

/// Plan 412 续(toast 堆叠视觉):kind → (边框色, 背景色, 标题)。success 绿 /
/// error 红 / warning 琥珀 / default zinc 边框;info 用主题 accent(与页面
/// text-primary 一致,蓝色只是解析失败的退回值)。default 边框用 zinc-500:
/// zinc-700 在暗色主题下几乎与背景融为一体。
fn toast_palette(kind: &str) -> ((u8, u8, u8), (u8, u8, u8), &'static str) {
    match kind {
        "success" => ((34, 197, 94), (34, 197, 94), "Success"), // green-500
        "error" => ((239, 68, 68), (239, 68, 68), "Error"),     // red-500
        "warning" => ((245, 158, 11), (245, 158, 11), "Warning"), // amber-500
        "info" => {
            let accent = crate::ui::style_iced_adapter::resolve_semantic_rgb(
                &crate::ui::style::Color::Primary,
            )
            .unwrap_or((59, 130, 246)); // blue-500(仅解析失败时退回)
            (accent, accent, "Info")
        }
        _ => ((113, 113, 122), (24, 24, 27), "Notification"), // zinc-500 / zinc-900
    }
}

/// Plan 412 续(toast VM 化):toast 卡片 — kind 决定配色(对齐 toast.at 的
/// 静态 demo:success 绿 / error 红 / warning 琥珀 / info 蓝 / default 主题
/// 边框),深色主题,max-width 360。宽度 Fill(封顶 360)—— 与探出条构成
/// 等宽牌堆,不随文字长度伸缩(vue-sonner 同为定宽卡)。
fn build_toast_card(t: &ToastReq) -> iced::Element<'static, IcedMessage> {
    let (border_rgb, bg_rgb, title) = toast_palette(&t.kind);
    let title_color = if t.kind == "default" {
        iced::Color::WHITE
    } else {
        iced::Color::from_rgb8(border_rgb.0, border_rgb.1, border_rgb.2)
    };
    let msg_color = iced::Color::from_rgb8(161, 161, 170); // zinc-400

    let card_body = iced::widget::column![
        iced::widget::text(title.to_string())
            .size(14)
            .style(move |_| {
                iced::widget::text::Style {
                    color: Some(title_color),
                }
            }),
        iced::widget::text(t.msg.clone()).size(14).style(move |_| {
            iced::widget::text::Style {
                color: Some(msg_color),
            }
        }),
    ]
    .spacing(4);

    let (br, bgc, bg_a, bd_a) = (
        iced::Color::from_rgb8(border_rgb.0, border_rgb.1, border_rgb.2),
        iced::Color::from_rgb8(bg_rgb.0, bg_rgb.1, bg_rgb.2),
        if t.kind == "default" { 1.0 } else { 0.10 },
        if t.kind == "default" { 1.0 } else { 0.50 },
    );
    iced::widget::container(card_body)
        .padding(16)
        .width(iced::Length::Fill)
        .max_width(360)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color { a: bg_a, ..bgc })),
            border: iced::Border {
                color: iced::Color { a: bd_a, ..br },
                width: 1.0,
                radius: iced::border::Radius::new(8.0),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        })
        .into()
}

/// Plan 563: CSS 色串 → iced Color(场景契约的色值形态;坏值回退黑)。
fn canvas_css_color(s: &str) -> iced::Color {
    crate::ui::style::Color::from_hex(s)
        .map(|c| {
            let (r, g, b) = c.to_rgb8();
            iced::Color::from_rgb8(r, g, b)
        })
        .unwrap_or(iced::Color::BLACK)
}

/// Plan 563: 画布渲染程序 —— 场景数据契约的 iced 侧独立映射(与 vue 2D
/// 端不共享代码,共享映射规约):strokes → Path stroke;逻辑坐标 → px 按
/// extent 线性缩放(coords "WxH" 值域,bounds 尺寸对齐);线帽/拐角 round;
/// 单点笔画 = 直径线宽圆点(零长线 round cap 不可见,双端同规约);eraser
/// v1 = clear 色笔画(缺省白,不真挖除)。View 每帧重建 → Program 随之
/// 重建,draw 每帧重放(DrawListPainter 同形态,无缓存债)。
#[derive(Clone)]
pub(super) struct CanvasPainter {
    pub(super) scene: crate::ui::view::CanvasScene,
    pub(super) clear: Option<String>,
    pub(super) extent: Option<(f32, f32)>,
}

impl<M: Clone + 'static> iced::widget::canvas::Program<M> for CanvasPainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, LineCap, LineJoin, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        if let Some(clear) = &self.clear {
            frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), canvas_css_color(clear));
        }
        let (sx, sy) = match self.extent {
            Some((w, h)) if bounds.width > 0.0 && bounds.height > 0.0 => {
                (bounds.width / w, bounds.height / h)
            }
            _ => (1.0, 1.0),
        };
        let eraser_color = self.clear.as_deref().unwrap_or("#ffffff");
        for stroke in &self.scene.strokes {
            if stroke.points.is_empty() {
                continue;
            }
            let pts: Vec<iced::Point> = stroke
                .points
                .iter()
                .map(|(x, y)| iced::Point::new(x * sx, y * sy))
                .collect();
            let color = if stroke.eraser {
                canvas_css_color(eraser_color)
            } else {
                canvas_css_color(&stroke.color)
            };
            if pts.len() == 1 {
                frame.fill(&Path::circle(pts[0], stroke.width / 2.0), color);
                continue;
            }
            let path = Path::new(|b| {
                b.move_to(pts[0]);
                for p in &pts[1..] {
                    b.line_to(*p);
                }
            });
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(color)
                    .with_width(stroke.width)
                    .with_line_cap(LineCap::Round)
                    .with_line_join(LineJoin::Round),
            );
        }
        // PLAN-661 T-05: 图元三表绘制——edges（线段）→ nodes（circle/rect）
        // → labels（文本），声明序；extent 缩放复用 strokes 现行换算。
        for edge in &self.scene.edges {
            let path = Path::new(|b| {
                b.move_to(iced::Point::new(edge.x1 * sx, edge.y1 * sy));
                b.line_to(iced::Point::new(edge.x2 * sx, edge.y2 * sy));
            });
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(canvas_css_color(&edge.color))
                    .with_width(edge.width)
                    .with_line_cap(LineCap::Round),
            );
        }
        for node in &self.scene.nodes {
            let center = iced::Point::new(node.x * sx, node.y * sy);
            let color = canvas_css_color(&node.color);
            if node.shape == "rect" {
                let (w, h) = (node.w.unwrap_or(40.0) * sx, node.h.unwrap_or(40.0) * sy);
                frame.fill_rectangle(
                    iced::Point::new(center.x - w / 2.0, center.y - h / 2.0),
                    iced::Size::new(w, h),
                    color,
                );
            } else {
                frame.fill(&Path::circle(center, node.r.unwrap_or(16.0) * sx), color);
            }
        }
        for label in &self.scene.labels {
            frame.fill_text(iced::widget::canvas::Text {
                content: label.text.clone(),
                position: iced::Point::new(label.x * sx, label.y * sy),
                color: canvas_css_color(&label.color),
                size: iced::Pixels(label.size.clamp(1.0, 96.0)),
                ..Default::default()
            });
        }
        vec![frame.into_geometry()]
    }
}

/// Plan 412 续(toast 堆叠视觉):探出条的自绘 painter —— 三边描边 + 露出侧
/// 圆角 + 半透明填充,贴卡侧直角无边线(canvas 实现,iced Border 无法表达
/// 单边无边框,clip 也只是矩形裁剪)。
#[derive(Clone)]
struct ToastPeekPainter {
    line: iced::Color,
    body: iced::Color,
    /// 露出侧(圆角所在)在顶部?
    round_top: bool,
}

impl iced::widget::canvas::Program<IcedMessage> for ToastPeekPainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Fill, Frame, Path, Stroke};
        let w = bounds.width;
        let h = bounds.height;
        let r = (h / 2.0).min(7.0);
        let path = Path::new(|b| {
            if self.round_top {
                b.move_to(iced::Point::new(0.0, h));
                b.line_to(iced::Point::new(0.0, r));
                b.quadratic_curve_to(iced::Point::new(0.0, 0.0), iced::Point::new(r, 0.0));
                b.line_to(iced::Point::new(w - r, 0.0));
                b.quadratic_curve_to(iced::Point::new(w, 0.0), iced::Point::new(w, r));
                b.line_to(iced::Point::new(w, h));
            } else {
                b.move_to(iced::Point::new(0.0, 0.0));
                b.line_to(iced::Point::new(0.0, h - r));
                b.quadratic_curve_to(iced::Point::new(0.0, h), iced::Point::new(r, h));
                b.line_to(iced::Point::new(w - r, h));
                b.quadratic_curve_to(iced::Point::new(w, h), iced::Point::new(w, h - r));
                b.line_to(iced::Point::new(w, 0.0));
            }
        });
        let mut frame = Frame::new(renderer, bounds.size());
        // 开放路径的 fill 隐式闭合(贴卡侧直线),描边只画三边。
        frame.fill(&path, self.body);
        frame.stroke(
            &path,
            Stroke::default().with_color(self.line).with_width(1.0),
        );
        vec![frame.into_geometry()]
    }
}

/// Plan 412 续(toast 堆叠视觉):被压在后面的 toast 的"探出条"—— 只露出
/// 卡片顶缘 14px 的一条,宽度随深度收窄(360→342→324)、边框/背景透明度
/// 递减,叠在前卡后方,复刻 vue-sonner 折叠堆叠的观感。iced widget 不支持
/// 旋转,以宽度收窄 + 透明度阶梯近似 sonner 的缩放/旋转纵深。纯装饰 ——
/// 完整内容只有最前(最新)一张;某条到期出队后,下一张自然顶上成为前卡。
///
/// 配色与 build_toast_card 完全同源(default 不透明 zinc 深底 / 彩色 kind
/// 为 zinc 底 + 10% 主色、边框 50% 主色),仅按深度轻微衰减 —— 视觉上是
/// "真卡片露出的顶缘",而不是色条。贴住前卡的那条边不画边线(避免两条
/// kind 色线贴在一起混色),其余三边保留。
fn build_toast_peek(
    t: &ToastReq,
    depth: usize,
    top_anchor: bool,
    h_anchor: usize,
) -> iced::Element<'static, IcedMessage> {
    let (border_rgb, bg_rgb, _) = toast_palette(&t.kind);
    // 透明度与 build_toast_card 完全一致(内部颜色 = 真卡顶缘,不是色条):
    // default 不透明 zinc 深底;彩色 kind 为 10% 主色底 + 50% 主色边框。
    // 背景不随深度衰减(10% 底再打折会融进页面背景变幽灵条);深度感靠
    // 双侧缩进 + 边框阶梯。
    let is_default = t.kind == "default";
    let bd_a = if is_default {
        [0.0f32, 1.0, 0.85, 0.75, 0.65][depth.min(4)]
    } else {
        [0.0f32, 0.50, 0.38, 0.30, 0.25][depth.min(4)]
    };
    let bg_a = if is_default { 1.0f32 } else { 0.10 };
    let (br, bgc) = (
        iced::Color::from_rgb8(border_rgb.0, border_rgb.1, border_rgb.2),
        iced::Color::from_rgb8(bg_rgb.0, bg_rgb.1, bg_rgb.2),
    );
    // 双侧缩进(sonner 的后卡是整体 scale,左右同时收):锚侧每层 +8px,
    // 对侧由 max_width 收 18px/层;center 锚两侧各 4px。外层 wrapper 用
    // padding + align_x 把条带从槽边缘推离,右缘不再与前卡贴齐。
    let (pad_anchor, pad_off) = if h_anchor == 1 {
        (4.0f32, 4.0)
    } else {
        (8.0f32, 0.0)
    };
    let (pad_left, pad_right) = if h_anchor == 0 {
        (pad_anchor, pad_off)
    } else {
        (pad_off, pad_anchor)
    };
    // 三边框 + 圆角用 canvas 自绘(iced Border 只支持四边等宽、clip 只是
    // 矩形裁剪):路径 = 左边线 + 露出侧圆角 + 顶线 + 右圆角 + 右边线,
    // 贴住前卡的那条边不画线(避免本条底线与下一层顶线两条 kind 色线贴
    // 在一起混色);填充即半透明主体。top 锚自动翻转(圆角与露出的底线
    // 在下侧)。
    let painter = ToastPeekPainter {
        line: iced::Color { a: bd_a, ..br },
        body: iced::Color { a: bg_a, ..bgc },
        round_top: !top_anchor,
    };
    let canvas_el = iced::widget::canvas(painter)
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(14.0));
    let styled = iced::widget::container(canvas_el)
        .height(iced::Length::Fixed(14.0))
        .width(iced::Length::Fill)
        .max_width(360.0 - depth as f32 * 18.0);
    iced::widget::container(styled)
        .width(iced::Length::Fill)
        .align_x(match h_anchor {
            0 => iced::Alignment::Start,
            1 => iced::Alignment::Center,
            _ => iced::Alignment::End,
        })
        .padding(iced::Padding {
            top: 0.0,
            right: pad_right * depth as f32,
            bottom: 0.0,
            left: pad_left * depth as f32,
        })
        .into()
}

/// Plan 412 续(toast 修正 3):toast 悬浮层 — 多卡片堆叠。按 position 分组到
/// 九宫格槽位(top-/bottom-/center × left/center/right,加正中 "center"),
/// 同锚点纵向堆叠:**从上到下 = 最旧到最新**(新 toast 总在最下方 —— top 锚
/// 从窗口顶向下生长,bottom 锚贴窗口底、旧的被新来的挤到上方),某条到期
/// 移除后其余上移补位(标准 toast 库行为)。边距 16px、卡片间距 8px。
pub(super) fn build_toast_layer(toasts: &[ToastReq]) -> iced::Element<'static, IcedMessage> {
    let parse_pos = |pos: &str| -> (usize, usize) {
        let v = if pos.starts_with("top") {
            0
        } else if pos.starts_with("bottom") {
            2
        } else {
            1
        };
        let h = if pos.ends_with("left") {
            0
        } else if pos.ends_with("right") {
            2
        } else {
            1
        };
        (v, h)
    };
    // 3×3 槽位分组(保序:同槽按加入顺序 = 最旧到最新)。
    let mut slots: [Vec<usize>; 9] = Default::default();
    for (i, t) in toasts.iter().enumerate() {
        let (v, h) = parse_pos(t.position.as_str());
        slots[v * 3 + h].push(i);
    }

    // 单个槽:该锚点的折叠牌堆(deck,对齐 vue-sonner)—— 只有最前(最新)
    // 一张渲染完整内容卡,后面至多 4 张以"探出条"露出顶缘(双侧缩进、
    // 边框透明度阶梯),更旧的完全被盖住。空槽 = Fill 宽占位,保持三列等宽。
    // 前卡贴锚点边:bottom/center 锚前卡在堆底(探出条在上方),top 锚前卡
    // 在堆顶(探出条在下方,旧的被新的向下挤)。
    let slot = |idxs: &[usize]| -> iced::Element<'static, IcedMessage> {
        if idxs.is_empty() {
            return iced::widget::container(iced::widget::Space::new())
                .width(iced::Length::Fill)
                .into();
        }
        let h_anchor = parse_pos(&toasts[idxs[0]].position).1;
        let h_align = match h_anchor {
            0 => iced::Alignment::Start,
            1 => iced::Alignment::Center,
            _ => iced::Alignment::End,
        };
        let top_anchor = parse_pos(&toasts[idxs[0]].position).0 == 0;
        let n = idxs.len();
        let mut c = iced::widget::column![]
            .width(iced::Length::Fill)
            .align_x(h_align);
        // depth = 距前卡的层数(0 = 前卡);探出条画 depth 1..=4(前卡 +
        // 4 层 = 最多 5 张可见),更旧的完全被盖住不渲染。
        let ranks: Box<dyn Iterator<Item = usize>> = if top_anchor {
            // 前卡(rank = n-1)贴顶:列自上而下 = 最新 → 最旧。
            Box::new((0..n).rev())
        } else {
            // 前卡贴堆底:列自上而下 = 最旧 → 最新,探出条压在前卡上方。
            Box::new(0..n)
        };
        for rank in ranks {
            let depth = n - 1 - rank;
            if depth == 0 {
                c = c.push(build_toast_card(&toasts[idxs[rank]]));
            } else if depth <= 4 {
                c = c.push(build_toast_peek(
                    &toasts[idxs[rank]],
                    depth,
                    top_anchor,
                    h_anchor,
                ));
            }
        }
        c.into()
    };

    let row_for = |v: usize| -> iced::Element<'static, IcedMessage> {
        let mut r = iced::widget::row![];
        for h in 0..3 {
            r = r.push(slot(&slots[v * 3 + h]));
        }
        r.width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .into()
    };

    let fill_h = || iced::widget::Space::new().height(iced::Length::Fill);
    // 3×3 行列骨架。列必须显式 Fill 尺寸:column/row 默认 Shrink,若列收缩,
    // 内部所有 Fill 宽的槽列与 fill_h 都会因无可用空间而塌为 0(堆叠版首个
    // 实现的"完全不显示"bug 就是这条 Shrink 链)。
    let mut col = iced::widget::column![]
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);
    col = col.push(row_for(0));
    col = col.push(fill_h());
    col = col.push(row_for(1));
    col = col.push(fill_h());
    col = col.push(row_for(2));

    iced::widget::container(col)
        .padding(16)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// MCP action channel 轮询（AppTickRecipe::Poll 用；读进程级静态通道）。
/// PLAN-095 T-02: pipeline → event-loop wake. Subscription-time gates evaluate
/// per update, and a millisecond decode window falls between evaluations — an
/// otherwise-idle app would never wake to run the notify sweep. This
/// desktop-level recipe polls the pipeline generation at 50ms and yields
/// `__media_tick` only when it advanced (idle = zero messages, no update
/// storm); the per-app awaiting gate tick continues from there.
static MEDIA_WAKE_SEEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(super) static MEDIA_SWEEP_SEEN_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn poll_media_wake() -> Option<IcedMessage> {
    let gen = auto_media_core::image_pipeline::media_change_generation();
    let last = MEDIA_WAKE_SEEN.swap(gen, std::sync::atomic::Ordering::AcqRel);
    if gen != last {
        Some(IcedMessage {
            widget: String::new(),
            event: "__media_tick".to_string(),
            input_value: None,
        })
    } else {
        None
    }
}

pub(super) fn media_wake_subscription(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::from_recipe(AppTickRecipe {
        app,
        kind: AppTickKind::Poll(poll_media_wake, 50),
    })
}

/// PLAN-737：合成动作臂的 noop 回发（ack id 随行——noop 排在臂内 Task
/// 队列尾，其消费即「动作已应用」的回执点；link 消息等 Task 先于 noop
/// 同队列派发）。
pub(super) fn mcp_noop_with_ack(ack: Option<u64>) -> IcedMessage {
    let event = match ack {
        Some(id) => format!("__noop|ack={id}"),
        None => "__noop".to_string(),
    };
    IcedMessage::from_dynamic(&DynamicMessage::String(event))
}

/// PLAN-737：合成动作 ack 回执（无 noop 臂的臂尾直执回执点；noop 臂由
/// noop 消费臂回执——见 `__noop|ack=` 消费处）。复用 fixture 回执表
///（同 id 空间、同容量纪律；Applied 空载荷 = 纯栅栏语义）。
pub(super) fn mcp_ack_apply(mcp_shared: &Option<crate::ui::mcp_server::SharedStateHandle>, ack: Option<u64>) {
    if let (Some(id), Some(mcp)) = (ack, mcp_shared) {
        mcp.lock().unwrap().finish_fixture(
            id,
            crate::ui::mcp_server::FixtureAck::Applied {
                changed: vec![],
                trigger: None,
            },
        );
    }
}

/// PLAN-740 T-02：MCP action 消息映射（原 `poll_mcp_actions` 的
/// target→IcedMessage 臂原样保留）。VM mode uses Event addressing,
/// which maps onto IcedMessage. Path mode is a no-op here (rust mode
/// uses devtools_subscription/devtools_update).
fn map_mcp_action(action: crate::ui::mcp_server::ActionMessage) -> Option<IcedMessage> {
    match action.target {
        crate::ui::mcp_server::ActionTarget::Event { widget, event } => Some(IcedMessage {
            widget,
            event,
            input_value: action.value,
        }),
        crate::ui::mcp_server::ActionTarget::Path { .. } => None,
        crate::ui::mcp_server::ActionTarget::Fixture { request_id } => {
            let payload = action.value.unwrap_or_default();
            Some(IcedMessage {
                widget: String::new(),
                event: format!("__mcp_fixture|{}|{}", request_id, payload),
                input_value: None,
            })
        }
    }
}

/// PLAN-740 T-02（交付节奏修复——T-01 定谳的消息间隔结构杠杆）：
/// MCP action **push 通道订阅**——替代 16ms AppTick 轮询。
///
/// T-00/T-01 定责链：54Hz 驱动的原生动作间隔（18.5ms）被 16ms tick
/// 消费网格重排出亚 16ms 消息对（got 间隔 8ms bin×49/127 直证），
/// 配对落入 16.7ms 呈现槽被 iced_winit AboutToWait 批处理合并为单帧
/// ——交付 0.79×驱动（<AC-02 ±10% 带）。本订阅经转发线程
/// `recv()` 阻塞等待（**到达即投递**，间隔恢复原生 ±转发抖动 >
/// 槽宽 → 无配对），唤醒后 `try_recv` 循环取空（**drain-to-empty
/// 语义内含**——突发吸收；Disconnected 断流终止，Path 畸形臂原样）。
/// 不 fork iced_winit（纯 app 侧订阅重排）；712 r2 泵域零触碰；
/// 16ms 轮询的 tick 轴随之退役（mcp_poll 行由 mcp_recv 行接替——
/// 转发线程逐动作打点，AUTO_SCHED_DIAG 门控零开销）。
#[derive(Debug, Clone, Hash)]
struct McpPushRecipe {
    app: crate::ui::session::AppId,
}

impl iced_futures::subscription::Recipe for McpPushRecipe {
    type Output = crate::ui::session::DesktopMessage;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        "plan740_mcp_push".hash(state);
        self.app.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        use crate::ui::session::DesktopMessage as DM;
        use iced_futures::futures::StreamExt;

        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let rx = guard.lock().unwrap().take();
        let app = self.app;
        let Some(rx) = rx else {
            // 接收器不在位（rust 轨/重复装配）——空流兜底（订阅表
            // 身份含本字符串，与 poll 轨互不去重冲突）。
            eprintln!("[PLAN740] mcp push stream: receiver not present");
            return iced_futures::futures::stream::empty().boxed();
        };
        let (tx, rx_msg) = iced_futures::futures::channel::mpsc::unbounded::<IcedMessage>();
        let _ = std::thread::Builder::new()
            .name("mcp-action-push".to_string())
            .spawn(move || {
                loop {
                    let Ok(action) = rx.recv() else { break };
                    // drain-to-empty：唤醒后循环取空再逐条投递（突发
                    // 吸收；顺序保持 recv 序）。
                    let mut batch = vec![action];
                    while let Ok(more) = rx.try_recv() {
                        batch.push(more);
                    }
                    for a in batch {
                        if crate::ui::sched_diag::enabled() {
                            let t0 = crate::ui::dynamic::sched_diag_t0();
                            eprintln!(
                                "[SCHED-DIAG] mcp_recv t={}ms got=1",
                                t0.elapsed().as_millis()
                            );
                        }
                        if let Some(msg) = map_mcp_action(a) {
                            if tx.unbounded_send(msg).is_err() {
                                return; // 订阅端已断（进程退出路径）
                            }
                        }
                    }
                }
            });
        rx_msg.map(move |msg| DM::App(app, msg)).boxed()
    }
}

/// PLAN-740 T-00：batch-drive 探针线程体（test-only，AUTOUI_DRIVE_PROBE
/// 门——spawn 点注释为完整协议）。调度=绝对截止时轴（防 Windows sleep
/// 过冲累积漂移，末 2ms 自旋兜精度）；动作经 `SharedState::send_action`
/// 异步直推（与 MCP 工具同一 action 通道；**无 applied 栅栏**——栅栏会
/// 把驱动闭成「应用率」回路，破坏吞吐勘定语义）。
pub(super) fn spawn_drive_probe(mcp_shared: crate::ui::mcp_server::SharedStateHandle, trigger_file: String) {
    std::thread::spawn(move || {
        let mut consumed_offset: u64 = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(5));
            let Ok(meta) = std::fs::metadata(&trigger_file) else {
                continue;
            };
            let len = meta.len();
            if len <= consumed_offset {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&trigger_file) else {
                continue;
            };
            // 只消费到末个换行为止（半行留待下一轮——防读到部分写入）。
            let consume_to = match content.rfind('\n') {
                Some(pos) => pos + 1,
                None => continue,
            };
            let new_text = &content[consumed_offset as usize..consume_to];
            consumed_offset = consume_to as u64;
            for line in new_text.lines() {
                // 行格式：scroll <hz> <count> <element_id> <ystep>
                // element_id 接受 `vnode_N` 全串或裸 N（统一归一）。
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() != 5 || parts[0] != "scroll" {
                    continue;
                }
                let (Ok(hz), Ok(count)) = (parts[1].parse::<f64>(), parts[2].parse::<usize>())
                else {
                    continue;
                };
                let (Ok(eid_num), Ok(ystep)) = (
                    parts[3].trim_start_matches("vnode_").parse::<u64>(),
                    parts[4].parse::<f32>(),
                ) else {
                    continue;
                };
                if hz <= 0.0 || count == 0 {
                    continue;
                }
                let period = std::time::Duration::from_secs_f64(1.0 / hz);
                let start = std::time::Instant::now();
                for i in 0..count {
                    let deadline = start + period.mul_f64(i as f64);
                    // 绝对截止时轴（防 sleep 过冲累积漂移）；末 2ms 自旋。
                    // checked_duration_since——截止已过返回 None，不做
                    // Duration 减法（朴素 `deadline - now - 2ms` 在检查与
                    // 减法之间时钟推进时会下溢 panic——本件 T-00 实录）。
                    loop {
                        match deadline.checked_duration_since(std::time::Instant::now()) {
                            None => break,
                            Some(remaining) if remaining <= std::time::Duration::from_millis(2) => {
                                std::hint::spin_loop();
                            }
                            Some(remaining) => {
                                std::thread::sleep(remaining - std::time::Duration::from_millis(2));
                            }
                        }
                    }
                    let msg = crate::ui::mcp_server::ActionMessage {
                        target: crate::ui::mcp_server::ActionTarget::Event {
                            widget: String::new(),
                            event: "__mcp_scroll".to_string(),
                        },
                        action: crate::ui::mcp_types::UiActionType::Scroll,
                        value: Some(format!(
                            "vnode_{eid_num}{}{}",
                            PAYLOAD_SEP,
                            i as f32 * ystep
                        )),
                    };
                    if let Err(e) = mcp_shared.lock().unwrap().send_action(msg) {
                        eprintln!("[DRIVE-PROBE] send failed: {e}");
                        break;
                    }
                }
                if crate::ui::sched_diag::enabled() {
                    eprintln!(
                        "[SCHED-DIAG] drive_probe done hz={hz} count={count} elapsed={}ms",
                        start.elapsed().as_millis()
                    );
                }
            }
        }
    });
}

/// Apply one test-only state fixture on the VM/iced thread. The MCP server
/// performs the cheap schema checks before enqueueing; this second check keeps
/// the renderer safe if a queued payload is malformed or stale.
pub(super) fn apply_mcp_fixture(
    component: &mut crate::ui::dynamic::DynamicComponent,
    payload: &str,
) -> crate::ui::mcp_server::FixtureAck {
    let root: serde_json::Value = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(error) => {
            return crate::ui::mcp_server::FixtureAck::Error {
                code: "invalid_schema".to_string(),
                message: format!("fixture payload is not valid JSON: {error}"),
            }
        }
    };
    let state_obj = match root.get("state").and_then(|value| value.as_object()) {
        Some(map) if !map.is_empty() => map,
        _ => {
            return crate::ui::mcp_server::FixtureAck::Error {
                code: "invalid_schema".to_string(),
                message: "fixture state must be a non-empty object".to_string(),
            }
        }
    };

    let mut prepared = Vec::with_capacity(state_obj.len());
    let mut array_items = 0usize;
    for (field, raw) in state_obj {
        let existing = match component.read_state(field) {
            Ok(value) => value,
            Err(error) => {
                return crate::ui::mcp_server::FixtureAck::Error {
                    code: "unknown_field".to_string(),
                    message: format!("state field '{field}' is not available: {error}"),
                }
            }
        };
        let incoming = match crate::ui::mcp_server::fixture_json_to_value(raw, 0, &mut array_items)
        {
            Ok(value) => value,
            Err(error) => {
                return crate::ui::mcp_server::FixtureAck::Error {
                    code: "invalid_schema".to_string(),
                    message: format!("field '{field}': {error}"),
                }
            }
        };
        if !crate::ui::mcp_server::fixture_value_compatible(&existing, &incoming) {
            return crate::ui::mcp_server::FixtureAck::Error {
                code: "type_mismatch".to_string(),
                message: format!(
                    "field '{field}' is {}, request is {}",
                    crate::ui::mcp_server::fixture_value_kind(&existing),
                    crate::ui::mcp_server::fixture_value_kind(&incoming)
                ),
            };
        }
        prepared.push((field.clone(), incoming));
    }

    let mut changed = Vec::with_capacity(prepared.len());
    for (field, value) in prepared {
        let result = match value {
            auto_val::Value::Array(array) => component.write_state_vec(&field, array.values),
            value => component.write_state(&field, value),
        };
        if let Err(error) = result {
            return crate::ui::mcp_server::FixtureAck::Error {
                code: "write_failed".to_string(),
                message: format!("field '{field}': {error}"),
            };
        }
        changed.push(field);
    }
    changed.sort();

    // PLAN-659 T-05：trigger 双形态——{handler}（裸名直查注册表）或
    // {widget, event, input?}（namespaced 键派发）。
    enum FixtureTrigger {
        Handler(String),
        WidgetEvent {
            widget: String,
            event: String,
            input: Option<String>,
        },
    }
    let trigger = root
        .get("trigger")
        .and_then(|value| value.as_object())
        .and_then(|trigger| {
            let handler = trigger
                .get("handler")
                .and_then(|value| value.as_str())
                .filter(|s| !s.is_empty());
            if let Some(handler) = handler {
                return Some(FixtureTrigger::Handler(handler.to_string()));
            }
            let widget = trigger
                .get("widget")
                .and_then(|value| value.as_str())
                .filter(|s| !s.is_empty())?;
            let event = trigger
                .get("event")
                .and_then(|value| value.as_str())
                .filter(|s| !s.is_empty())?;
            let input = trigger
                .get("input")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            Some(FixtureTrigger::WidgetEvent {
                widget: widget.to_string(),
                event: event.to_string(),
                input,
            })
        });
    let trigger_label = trigger.as_ref().map(|t| match t {
        FixtureTrigger::Handler(name) => format!("handler:{name}"),
        FixtureTrigger::WidgetEvent { widget, event, .. } => format!("{widget}.{event}"),
    });
    if let Some(trigger) = trigger {
        // PLAN-659 T-05：handler 裸名形态——跨名空间扫描 exports 解析
        // widget 归属（嵌入画廊形态根组件是宿主 App，目标 handler 在子
        // demo 名空间），解析后与 widget 形态同路派发；未命中/歧义响亮
        // 报错（此前静默无操作）。
        let trigger = match trigger {
            FixtureTrigger::Handler(name) => {
                let mut widgets = component.widgets_declaring_handler(&name);
                match widgets.len() {
                    1 => FixtureTrigger::WidgetEvent {
                        widget: widgets.remove(0),
                        event: name,
                        input: None,
                    },
                    0 => {
                        let sample: Vec<String> =
                            component.msg_handler_names().into_iter().take(12).collect();
                        return crate::ui::mcp_server::FixtureAck::Error {
                            code: "handler_not_found".to_string(),
                            message: format!(
                                "msg handler '{name}' not in registry; available (first 12): {sample:?}"
                            ),
                        };
                    }
                    _ => {
                        return crate::ui::mcp_server::FixtureAck::Error {
                            code: "ambiguous_handler".to_string(),
                            message: format!(
                                "msg handler '{name}' declared by multiple widgets {widgets:?}; use the {{widget, event}} trigger form"
                            ),
                        };
                    }
                }
            }
            widget_event => widget_event,
        };
        // 正规化后必为 WidgetEvent（handler 形态已解析归属或提前回错）。
        let (widget, event, input) = match trigger {
            FixtureTrigger::WidgetEvent {
                widget,
                event,
                input,
            } => (widget, event, input),
            FixtureTrigger::Handler(_) => {
                unreachable!("handler form normalized to widget form above")
            }
        };
        if component.is_timer_entry(&widget, &event) {
            component.fire_timer(&widget, &event);
        } else if !component.has_handler_for(&widget, &event) {
            // PLAN-659 T-05：namespaced 键注册表直查（与 call_handler_for
            // 同键）——未命中响亮报错而非静默无操作（AddrGo 病灶）。
            return crate::ui::mcp_server::FixtureAck::Error {
                code: "handler_not_found".to_string(),
                message: format!(
                    "no handler '{event}' in widget '{widget}' (namespaced registry miss)"
                ),
            };
        } else {
            component.on_with_input_for(&widget, &event, input);
        }
    }
    crate::ui::mcp_server::FixtureAck::Applied {
        changed,
        trigger: trigger_label,
    }
}

/// Subscription that polls the MCP action channel and injects messages into
/// the event loop. This allows MCP actions to truly simulate user operations
/// (with animations, state updates, and full UI refresh). 459：打标归 primary
/// App（T8 单 App 语义）。
/// PLAN-740 T-02：16ms 轮询 → push 通道（McpPushRecipe——到达即投递 +
/// drain-to-empty；poll 轨退役理由与定责链见该类型文档）。
pub(super) fn mcp_action_subscription(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::from_recipe(McpPushRecipe { app })
}

/// Plan 314: MCP heartbeat. iced only calls `update()` in response to a
/// message, so a freshly-launched, untouched app would otherwise never cycle
/// `update()` → bounds collection and the styled VTree snapshot pushed by
/// `__bounds_collected` (Task 5) would stay `None` — the `autoui_vtree` tool
/// would degrade to "UI not yet rendered" until the user interacted. This emits
/// a `__mcp_heartbeat` message on a fixed cadence so the snapshot stays fresh
/// on its own. The message matches no `update()` arm; it simply reaches the
/// `needs_bounds` block at the end of `update()` (which `view()` set true under
/// the same `capture_debug` gate) and launches the next bounds round-trip.
/// Gated on MCP being active — see the subscription closure — so a non-MCP run
/// stays fully idle.
/// Plan 423 P1: update 闭包的热重载传播状态 —— 已见过的配置 generation
/// (swap 比较)与上次 mtime 轮询时刻(500ms 节流)。
pub(super) static CONFIG_GEN_SEEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(super) static CONFIG_MTIME_POLL: std::sync::Mutex<Option<std::time::Instant>> =
    std::sync::Mutex::new(None);

pub(super) fn mcp_heartbeat_subscription(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    // 2026-08-22:200ms → 2s。心跳消息会触发一次完整 update→view 重建
    // (ash-gui 基线视图 ~80ms/次,大 Code 块时 >200ms)—— 200ms 节拍令
    // 消息队列常态积压,事件饿死(命令提交丢失、CPU 满转;实测空闲就
    // ~40% CPU)。快照 2s 内新鲜即可,真实交互路径本就会触发重建。
    app_tick(app, "__mcp_heartbeat", 2000)
}

// =====================================================================
// Shell SSE → iced subscription bridge (ash-gui-native M1)
// =====================================================================
// 让 iced 原生路径消费后端 `~Stream<ShellEvent>` 契约。两条路径共用本桥:
//
//   - **merged (in-process)** 模式(ash-gui vm 默认):UI VM 无法执行系统进程,
//     本桥在 renderer 侧起一个 Rust 执行器线程跑 std::process::Command,通过
//     `tokio::sync::mpsc` 把 command_output / command_result 事件回流。
//   - **HTTP** 模式(`AUTO_BACKEND=http://host:port`):执行器线程改为拉起一个
//     reqwest 异步客户端连后端 `/api/stream`,逐帧 SSE 解析后推同一 mpsc。
//
// VM 限制:`push_value` 对 struct 参数只推占位 0(vm_bridge.rs:929),store 的
// `.RunResult(result)` / `.RunOutput(output)` 无法直接收 struct 参数。故 update
// 闭包先把事件字段 write_state 到 store 的「预置字段」(`__sse_*`),再以**无参**
// 触发对应 handler —— handler body 读预置字段(见 ash-gui shell_store.at)。
//
// 事件契约对齐 ash-server 的 ShellEvent(`types.rs:115-122`):
//   {"event":"command_output","block_id":N,"chunk":"..."}
//   {"event":"command_result","CommandResult":{block_id,cwd,status,output,duration_ms}}

/// 一条回流到 iced 的事件(执行器线程 / HTTP 客户端 → subscription)。
/// `event` 取 "command_output" | "command_result"。`payload` 是完整 JSON
/// (update 闭包按字段解析后写预置字段)。
pub(super) struct ShellStreamEvent {
    pub(super) event: String,
    pub(super) payload_json: String,
}

/// 执行器线程的共享句柄:取消标志 + HTTP 后端地址(若为 HTTP 模式)。
/// Plan 060:命令队列已迁至 vm::shell_bridge(shell.at 经 native 生产,
/// PendingShellCommand 退役,由 ShellExecRequest 顶替)。
pub(super) struct ShellExecutorHandle {
    /// block_id → cancel flag。Cancel 时插入 true;执行器轮询后清除。
    pub(super) cancel_flags: std::collections::HashMap<i64, bool>,
    /// HTTP 模式后端 base URL(如 "http://127.0.0.1:3000");None = merged 模式。
    pub(super) http_backend: Option<String>,
}

/// 执行器线程的事件回流 receiver(全局,subscription poll)。仿 MCP_ACTION_RX。
pub(super) static SHELL_EVENT_RX: std::sync::OnceLock<
    std::sync::Mutex<Option<std::sync::mpsc::Receiver<ShellStreamEvent>>>,
> = std::sync::OnceLock::new();

/// Plan 060 M3:事件注入 sender(全局)。ash-runner(auto-shell 仓)把进程内
/// ash_server 的 ShellEvent 广播转成 SSE 同格式 JSON 后,经
/// `inject_shell_event` 推入本通道 —— 与 merged_exec_loop / http_sse_loop
/// 的回流同源,前端事件泵与 blocks 回写零改动。
static SHELL_EVENT_TX: std::sync::OnceLock<
    std::sync::Mutex<Option<std::sync::mpsc::Sender<ShellStreamEvent>>>,
> = std::sync::OnceLock::new();

/// Plan 060 M3:向 shell 事件泵注入一条事件(pub,供 ash-runner 调用)。
/// `payload_json` 为 ash-server ShellEvent 的完整 JSON(与 http_sse_loop
/// 转发格式一致,含 event 判别键)。返回是否成功入队。
pub fn inject_shell_event(event: &str, payload_json: &str) -> bool {
    let guard = SHELL_EVENT_TX.get_or_init(|| std::sync::Mutex::new(None));
    let mut lock = match guard.lock() {
        Ok(l) => l,
        Err(_) => return false,
    };
    match lock.as_ref() {
        Some(tx) => tx
            .send(ShellStreamEvent {
                event: event.to_string(),
                payload_json: payload_json.to_string(),
            })
            .is_ok(),
        None => false,
    }
}

/// 执行器线程的共享句柄(全局,update 闭包提交命令 / 标记取消)。仿 MCP_ACTION_RX。
pub(super) static SHELL_EXEC_HANDLE: std::sync::OnceLock<
    std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
> = std::sync::OnceLock::new();

/// Plan 057 (ash-gui): shell 桥是否运行在 HTTP 模式(AUTO_BACKEND 非空)。
/// HTTP 模式下 store 的 `run_command()`/`cancel()` 经 api_over_http 改写直连
/// 后端,本进程执行器只做纯 SSE 泵 —— update 闭包不应再向队列提交命令 /
/// 置本地取消标志(否则双提交)。
pub(super) fn shell_bridge_is_http() -> bool {
    SHELL_EXEC_HANDLE
        .get()
        .and_then(|h| h.lock().ok())
        .map(|h| h.http_backend.is_some())
        .unwrap_or(false)
}

/// 启动 shell 执行器线程(merged + HTTP 双模式)。在 `run_dynamic_iced` 里调用一次。
/// 返回事件 receiver,由调用方塞进 `SHELL_EVENT_RX` 全局量供 subscription poll。
pub(super) fn start_shell_executor() -> std::sync::mpsc::Receiver<ShellStreamEvent> {
    let (tx, rx) = std::sync::mpsc::channel::<ShellStreamEvent>();
    let handle = std::sync::Arc::new(std::sync::Mutex::new(ShellExecutorHandle {
        cancel_flags: std::collections::HashMap::new(),
        http_backend: std::env::var("AUTO_BACKEND").ok().filter(|s| !s.is_empty()),
    }));
    // 注册全局句柄,供 update 闭包提交命令 / 标记取消。
    {
        let guard = SHELL_EXEC_HANDLE.get_or_init(|| handle.clone());
        let _ = guard; // 已初始化则保留旧值(启动只调一次,正常路径是新初始化)
    }
    // Plan 060 M3:事件 sender 存全局,ash-runner 经 inject_shell_event 注入。
    {
        let guard = SHELL_EVENT_TX.get_or_init(|| std::sync::Mutex::new(None));
        *guard.lock().unwrap() = Some(tx.clone());
    }

    let exec_handle = handle.clone();
    let event_tx = tx.clone();
    std::thread::spawn(move || {
        // 执行器线程内建 current_thread tokio runtime(reqwest 异步 + 阻塞 IO 复用)。
        // 仿 mcp_server.rs:387 的建法。
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => return,
        };
        let is_http = exec_handle.lock().unwrap().http_backend.is_some();
        if is_http {
            // HTTP 模式:连后端 /api/stream,逐帧推事件。
            rt.block_on(http_sse_loop(exec_handle.clone(), event_tx));
        } else {
            // merged 模式:轮询本地命令队列,用 std::process 执行。
            rt.block_on(merged_exec_loop(exec_handle.clone(), event_tx));
        }
    });

    rx
}

/// Plan 044 M1: 把 serde_json::Value 转成 auto_val::Value(递归)。
/// 用于 command_result 的 output 字段(RenderedOutput tagged union)。
fn json_to_auto_val(v: &serde_json::Value) -> auto_val::Value {
    match v {
        serde_json::Value::Null => auto_val::Value::Nil,
        serde_json::Value::Bool(b) => auto_val::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                auto_val::Value::Int(i as i32)
            } else {
                auto_val::Value::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => auto_val::Value::str(s),
        serde_json::Value::Array(arr) => auto_val::Value::Array(auto_val::Array {
            values: arr.iter().map(json_to_auto_val).collect(),
        }),
        serde_json::Value::Object(map) => {
            let mut obj = auto_val::Obj::new();
            for (k, val) in map {
                obj.set(k.clone(), json_to_auto_val(val));
            }
            auto_val::Value::Obj(Box::new(obj))
        }
    }
}

/// merged 模式执行循环:从队列取命令 → std::process::Command 执行 → 推流式 + 结果事件。
async fn merged_exec_loop(
    handle: std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
    tx: std::sync::mpsc::Sender<ShellStreamEvent>,
) {
    use std::io::Read;
    loop {
        // 取一条命令(队列空则短歇,避免忙等)。
        // Plan 060(M1-T1):队列源切换为 vm::shell_bridge(shell.at 经
        // auto.shell.exec_submit native 生产;旧的 handle.queue 已退役)。
        let pending = auto_lang_core::vm::shell_bridge::pop();
        let pending = match pending {
            Some(p) => p,
            None => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                continue;
            }
        };
        let block_id = pending.block_id;
        let cmd = pending.cmd;
        let cwd = pending.cwd;

        // Plan 060(M2):Result 变体 —— shell.at 对 builtin(ls/cd/pwd)已在
        // .at 侧算好完整 command_result payload,直接回流(不 spawn)。
        // 语义在提交侧(api.at 契约内),传输层零语义。
        if matches!(
            pending.kind,
            auto_lang_core::vm::shell_bridge::ShellExecKind::Result
        ) {
            let _ = tx.send(ShellStreamEvent {
                event: "command_result".to_string(),
                payload_json: pending.result_json,
            });
            continue;
        }

        // Plan 060(M2-T5):拦截全数退役 —— show/ls/dir/cd/pwd 的 ash 语义
        // 已下沉 .at(shell.at run_command 提交侧分派)。执行线程只剩
        // 传输:cmd /C | sh -c + 逐块流式 + 取消 + 退出码。stdout 以
        // {"Text": ...} 直通。
        // 跨平台:Windows 用 cmd /C,Unix 用 sh -c。对齐 ash-server 的执行语义
        // (外部命令经 shell 解析,支持管道/重定向)。
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), cmd.clone()])
        } else {
            ("sh", vec!["-c".to_string(), cmd.clone()])
        };
        let mut child = match std::process::Command::new(program)
            .args(&args)
            .current_dir(if cwd.is_empty() { "." } else { &cwd })
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                // spawn 失败 → Failed。
                let _ = tx.send(ShellStreamEvent {
                    event: "command_result".to_string(),
                    payload_json: serde_json::json!({
                        "block_id": block_id,
                        "cwd": cwd,
                        "status": {"Failed": format!("spawn failed: {}", e)},
                        "output": null,
                        "duration_ms": 0,
                    })
                    .to_string(),
                });
                continue;
            }
        };

        // 逐块读 stdout 推 command_output。为支持取消,每读一块检查 cancel flag。
        let t0 = std::time::Instant::now();
        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();
        let mut cancelled = false;
        let mut buf = [0u8; 4096];
        let mut full_stdout = String::new();
        loop {
            // 取消检查。
            if let Some(true) = handle.lock().unwrap().cancel_flags.get(&block_id).copied() {
                cancelled = true;
                let _ = child.kill();
                break;
            }
            let read_guard = stdout.as_mut();
            let n = match read_guard.and_then(|r| r.read(&mut buf).ok()) {
                Some(0) => break, // EOF
                Some(n) => n,
                None => break,
            };
            if n > 0 {
                let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                full_stdout.push_str(&chunk);
                let _ = tx.send(ShellStreamEvent {
                    event: "command_output".to_string(),
                    payload_json: serde_json::json!({
                        "block_id": block_id,
                        "chunk": chunk,
                    })
                    .to_string(),
                });
            }
        }
        // 收 stderr(失败时作为 Failed 消息)。
        let mut err_text = String::new();
        if let Some(mut e) = stderr.take() {
            let _ = e.read_to_string(&mut err_text);
        }
        let status_code = child.wait().ok().and_then(|s| s.code());
        let duration_ms = t0.elapsed().as_millis() as u64;

        // 清除取消标志(已处理)。
        handle.lock().unwrap().cancel_flags.remove(&block_id);

        if cancelled {
            let _ = tx.send(ShellStreamEvent {
                event: "command_result".to_string(),
                payload_json: serde_json::json!({
                    "block_id": block_id,
                    "cwd": cwd,
                    "status": "Cancelled",
                    "output": null,
                    "duration_ms": duration_ms,
                })
                .to_string(),
            });
            continue;
        }

        // Plan 060(M2-T5):renderer 不再做任何输出结构化(旧 ls→Table 拦截
        // 与 parse_output_to_structured 已退役 —— 曾为规避 auto-shell →
        // auto-lang 的反向依赖而在 renderer 侧重实现 ash 语义,现语义归位
        // .at 层,见 docs/plans/060)。
        let success = matches!(status_code, Some(0));
        let (status_val, output_val) = if success {
            // Plan 060(M2-T5):结构化下沉前端 —— stdout 以 Text 直通,
            // ls/cd/pwd 语义由 store .RunResult 幂等修复补齐。
            (
                serde_json::Value::String("Success".to_string()),
                serde_json::json!({ "Text": full_stdout }),
            )
        } else {
            let msg = if err_text.is_empty() {
                format!("exit code {:?}", status_code)
            } else {
                err_text
            };
            (serde_json::json!({"Failed": msg}), serde_json::Value::Null)
        };
        let _ = tx.send(ShellStreamEvent {
            event: "command_result".to_string(),
            payload_json: serde_json::json!({
                "block_id": block_id,
                "cwd": cwd,
                "status": status_val,
                "output": output_val,
                "duration_ms": duration_ms,
            })
            .to_string(),
        });
    }
}

/// HTTP 模式:连后端 `/api/stream` SSE,逐帧 JSON 解析后推 ShellStreamEvent。
/// 命令提交 / 取消走 HTTP POST。本函数假设后端是 ash-server(契约见 ash-server/types.rs)。
async fn http_sse_loop(
    handle: std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
    tx: std::sync::mpsc::Sender<ShellStreamEvent>,
) {
    let base = match handle.lock().unwrap().http_backend.clone() {
        Some(b) => b,
        None => return,
    };
    let client = reqwest::Client::new();
    let stream_url = format!("{}/api/stream", base.trim_end_matches('/'));

    // Plan 057 (ash-gui): pure SSE pump — no poster task. When AUTO_BACKEND is
    // set, api_over_http rewrites the store's `run_command()`/`cancel()` calls
    // into direct HTTP POSTs to the backend, so forwarding the local queue
    // here would submit every command twice. The queue/cancel_flags stay
    // unused in HTTP mode.

    // 主:循环连 SSE(断线重连)。
    loop {
        match client.get(&stream_url).send().await {
            Ok(resp) => {
                use futures::StreamExt;
                let mut byte_stream = resp.bytes_stream();
                let mut acc = String::new();
                while let Some(chunk_res) = byte_stream.next().await {
                    match chunk_res {
                        Ok(bytes) => {
                            acc.push_str(&String::from_utf8_lossy(&bytes));
                            // 按 SSE 帧边界 "\n\n" 切分。
                            while let Some(idx) = acc.find("\n\n") {
                                let frame = acc[..idx].to_string();
                                acc = acc[idx + 2..].to_string();
                                // 提取 `data:` 行。
                                let data: String = frame
                                    .lines()
                                    .filter_map(|l| {
                                        l.strip_prefix("data:").map(|s| s.trim_start().to_string())
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                if data.is_empty() {
                                    continue;
                                }
                                // 解析 JSON,取 event 字段判别。
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                                    let event = v
                                        .get("event")
                                        .and_then(|e| e.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    // Plan 057: forward job events too (Plan 055
                                    // Phase A previously dropped them in HTTP mode).
                                    if event == "command_output"
                                        || event == "command_result"
                                        || event == "job_started"
                                        || event == "job_done"
                                        || event == "job_list"
                                    {
                                        let _ = tx.send(ShellStreamEvent {
                                            event,
                                            payload_json: v.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(_) => {}
        }
        // 断线后等一会再重连。
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

/// Update a block in store.blocks by block_id from a shell event (ash-gui M1).
/// Called from the update closure's command_output/command_result branch.
/// VM RunOutput/RunResult handlers can't read renderer-written Value::Array
/// blocks (type mismatch), so we update directly in Rust. Returns true if a
/// block was found and modified.
/// Plan 059(表格增强):对 block 的 output.Table.rows 做过滤 + 排序并
/// 原地写回(VM 端 BlockItem.Sort/Filter 特例调用;与 .at 侧 Vue 实现
/// 语义一致)。过滤词取 block.table_filter_q(小写),排序按 sort_col 列
/// 单元格文本,数字感知("1.2 KB"/"900 B"/"42" 按数值含 K/M/G 缩放,
/// 否则字节序)。原始行序无法恢复 —— 过滤词非空时从"过滤后的原序"重排,
/// 故过滤变化时先按原序重建:这里简单重跑"过滤 + 稳定选择排序"。
fn table_cell_text(cell: &auto_val::Value) -> String {
    if let auto_val::Value::Obj(o) = cell {
        if let Some(t) = o.get("Text") {
            return t.as_str().to_string();
        }
        if let Some(t) = o.get("Tagged") {
            if let auto_val::Value::Obj(tg) = t {
                return tg
                    .get("text")
                    .map(|v| v.as_str().to_string())
                    .unwrap_or_default();
            }
        }
    }
    String::new()
}

/// Plan 059:TableOutput → CSV 文本(复制/导出)。转义与 .at 侧一致:
/// 引号加倍;含逗号/引号/换行的字段整体加引号。
pub(super) fn table_to_csv(tbl: &auto_val::Obj) -> Option<String> {
    let cols: Vec<String> = match tbl.get("columns") {
        Some(auto_val::Value::Array(a)) => a
            .values
            .iter()
            .map(|v| csv_escape(&table_cell_text(v)))
            .collect(),
        _ => return None,
    };
    let mut out = cols.join(",");
    if let Some(auto_val::Value::Array(rows)) = tbl.get("rows") {
        for r in rows.values.iter() {
            if let auto_val::Value::Array(cells) = r {
                let line: Vec<String> = cells
                    .values
                    .iter()
                    .map(|c| csv_escape(&table_cell_text(c)))
                    .collect();
                out.push('\n');
                out.push_str(&line.join(","));
            }
        }
    }
    Some(out)
}

fn csv_escape(t: &str) -> String {
    if t.contains(',') || t.contains('"') || t.contains('\n') || t.contains('\n') {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t.to_string()
    }
}

/// 2026-08-22:结果底栏 copy(Table)→ TSV(制表符直连,不做转义 —— 直贴
/// Excel/Sheets 自动分列)。CSV 导出归 ExportCsv(table 图标)。
pub(super) fn table_to_tsv(tbl: &auto_val::Obj) -> Option<String> {
    let cols: Vec<String> = match tbl.get("columns") {
        Some(auto_val::Value::Array(a)) => a.values.iter().map(|v| table_cell_text(v)).collect(),
        _ => return None,
    };
    let mut out = cols.join("\t");
    if let Some(auto_val::Value::Array(rows)) = tbl.get("rows") {
        for r in rows.values.iter() {
            if let auto_val::Value::Array(cells) = r {
                let line: Vec<String> = cells.values.iter().map(|c| table_cell_text(c)).collect();
                out.push('\n');
                out.push_str(&line.join("\t"));
            }
        }
    }
    Some(out)
}

/// 数字前缀解析:返回 (值, 是否有效)。单位 K/M/G 缩放(B=1)。
fn table_numeric_prefix(t: &str) -> (f64, bool) {
    let mut num = String::new();
    let mut seen = false;
    for ch in t.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
            seen = true;
        } else if seen {
            break;
        }
    }
    if !seen {
        return (0.0, false);
    }
    let mut v: f64 = num.parse().unwrap_or(0.0);
    let rest = t[num.len()..].trim_start();
    let u = rest.chars().next().unwrap_or(' ').to_ascii_uppercase();
    v *= match u {
        'K' => 1024.0,
        'M' => 1048576.0,
        'G' => 1073741824.0,
        _ => 1.0,
    };
    (v, true)
}

fn table_row_text(row: &auto_val::Value, col: i64) -> String {
    if col < 0 {
        return String::new();
    }
    if let auto_val::Value::Array(cells) = row {
        if let Some(c) = cells.values.get(col as usize) {
            return table_cell_text(c);
        }
    }
    String::new()
}

pub(super) fn sort_table_rows(block: &mut auto_val::Obj, sort_col: i32, sort_dir: i32) {
    use auto_val::Value;
    // 取 output.Table.rows(原序由服务端/SSE 写入 —— 注意:一旦原地重排,
    // "原序"丢失;接受此限制:清空过滤/切换排序均基于当前序的稳定重排,
    // bash 也无"恢复原序"键;列指示 ● 未排序态在首次点击后不再出现)。
    let query = block
        .get("table_filter_q")
        .map(|v| v.as_str().to_lowercase())
        .unwrap_or_default();
    let mut rows: Vec<Value> = {
        let out = match block.get("output") {
            Some(Value::Obj(o)) => o.clone(),
            _ => return,
        };
        let tbl = match out.get("Table") {
            Some(Value::Obj(t)) => t.clone(),
            _ => return,
        };
        match tbl.get("rows") {
            Some(Value::Array(a)) => a.values.clone(),
            _ => return,
        }
    };
    // 过滤(大小写不敏感,任一单元格命中)
    if !query.is_empty() {
        rows.retain(|r| {
            let cells = match r {
                Value::Array(a) => &a.values,
                _ => return false,
            };
            cells
                .iter()
                .any(|c| table_cell_text(c).to_lowercase().contains(&query))
        });
    }
    // 排序(稳定:选择排序取"应排最前"者,按方向)
    if sort_col >= 0 {
        let sc = sort_col as usize;
        let mut out: Vec<Value> = Vec::with_capacity(rows.len());
        let mut remaining = rows;
        while !remaining.is_empty() {
            let mut best = 0usize;
            for i in 1..remaining.len() {
                let ta = table_row_text(&remaining[i], sc as i64);
                let tb = table_row_text(&remaining[best], sc as i64);
                let (na, va) = table_numeric_prefix(&ta);
                let (nb, vb) = table_numeric_prefix(&tb);
                let a_first = if va && vb { na < nb } else { ta < tb };
                let take = if sort_dir > 0 {
                    a_first
                } else {
                    !a_first && ta != tb
                };
                if take {
                    best = i;
                }
            }
            out.push(remaining.remove(best));
        }
        rows = out;
    }
    // 写回 output.Table.rows
    if let Some(Value::Obj(out)) = block.get_mut("output") {
        if let Some(Value::Obj(tbl)) = out.get_mut("Table") {
            tbl.set("rows", Value::Array(auto_val::Array { values: rows }));
        }
    }
}

pub(super) fn update_block_in_state(
    component: &mut DynamicComponent,
    block_id: i64,
    event: &str,
    payload: &serde_json::Value,
) -> bool {
    // Read blocks as Value::Array (renderer wrote it that way). Fall back to
    // read_state_as_vec for VM-native List, then write back the same way.
    let raw = match component.read_state("blocks") {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut blocks_vec: Vec<auto_val::Value> = match &raw {
        auto_val::Value::Array(arr) => arr.values.clone(),
        auto_val::Value::Nil => Vec::new(),
        _ => match component.read_state_as_vec("blocks") {
            Ok(v) => v,
            Err(_) => return false,
        },
    };
    // Plan 064: store-internal RunCommand submits (e.g. the boot-script
    // dispatch inside Init) push VM-heap blocks — Value::VmRef elements the
    // Obj-only match below can't finalize (block stuck Running). When no Obj
    // matches, materialize VmRef elements ONE target at a time (same shape
    // swap the renderer's own RunCommand interception performs on the tail
    // block) and retry the match; only the matched element is replaced in the
    // vec, other VmRef blocks keep their heap identity.
    let mut found = false;
    for b in &mut blocks_vec {
        if !matches!(b, auto_val::Value::Obj(_)) {
            let m = component.materialize_obj_value(b);
            match m {
                auto_val::Value::Obj(obj) => {
                    let id_matches = obj
                        .get("id")
                        .map(|v| v.as_int() as i64 == block_id)
                        .unwrap_or(false);
                    if !id_matches {
                        continue;
                    }
                    *b = auto_val::Value::Obj(obj);
                }
                _ => continue,
            }
        }
        if let auto_val::Value::Obj(obj) = b {
            let id_matches = obj
                .get("id")
                .map(|v| v.as_int() as i64 == block_id)
                .unwrap_or(false);
            if !id_matches {
                continue;
            }
            found = true;
            if event == "command_output" {
                // Append chunk to streamed_text.
                let chunk = payload.get("chunk").and_then(|x| x.as_str()).unwrap_or("");
                let cur = obj
                    .get("streamed_text")
                    .map(|v| v.as_str().to_string())
                    .unwrap_or_default();
                obj.set(
                    "streamed_text",
                    auto_val::Value::str(&format!("{}{}", cur, chunk)),
                );
            } else {
                // command_result: set status / output / duration_ms / clear streamed_text.
                let status_str = match payload.get("status") {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(obj @ serde_json::Value::Object(_)) => {
                        // {"Failed": msg} → 取 Failed 消息
                        obj.get("Failed")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Failed")
                            .to_string()
                    }
                    _ => "Failed".to_string(),
                };
                let (kind, message) = if status_str == "Success" {
                    ("Success".to_string(), String::new())
                } else if status_str == "Cancelled" {
                    ("Cancelled".to_string(), String::new())
                } else {
                    ("Failed".to_string(), status_str)
                };
                let dur = payload
                    .get("duration_ms")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as i32;
                // Plan 044 M1: output 按 payload 变体分发(Table/Text)。
                let output_obj = if let Some(output) = payload.get("output") {
                    if output.is_null() {
                        let text = obj
                            .get("streamed_text")
                            .map(|v| v.as_str().to_string())
                            .unwrap_or_default();
                        let mut o = auto_val::Obj::new();
                        o.set("Text", auto_val::Value::str(&text));
                        auto_val::Value::Obj(Box::new(o))
                    } else {
                        json_to_auto_val(output)
                    }
                } else {
                    let text = obj
                        .get("streamed_text")
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_default();
                    let mut o = auto_val::Obj::new();
                    o.set("Text", auto_val::Value::str(&text));
                    auto_val::Value::Obj(Box::new(o))
                };
                let mut status = auto_val::Obj::new();
                status.set("kind", auto_val::Value::str(&kind));
                status.set("message", auto_val::Value::str(&message));
                obj.set("status", auto_val::Value::Obj(Box::new(status)));
                obj.set("output", output_obj);
                // 2026-08-22(show 渲染成本):Code 变体把拼好的全文写入
                // streamed_text(复用 Block 已有 str 字段,renderer↔.at 的
                // VM-only 契约)——block_item.at 用单个 font-mono text 渲染
                // 整块(本文件 highlight_code 一次扫描),免去 .at 逐行逐
                // span 拼接。此前 660 行代码一次 view 重建 >200ms,MCP 心跳
                // (默认开启,200ms 一拍)令消息队列积压 → 事件饿死。
                let code_plain = payload
                    .get("output")
                    .and_then(|o| o.get("Code"))
                    .and_then(|c| c.get("lines"))
                    .and_then(|l| l.as_array())
                    .map(|lines| {
                        lines
                            .iter()
                            .map(|line| {
                                line.as_array()
                                    .map(|spans| {
                                        spans
                                            .iter()
                                            .map(|s| {
                                                s.get("text").and_then(|t| t.as_str()).unwrap_or("")
                                            })
                                            .collect::<String>()
                                    })
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    });
                match code_plain {
                    Some(s) => obj.set("streamed_text", auto_val::Value::str(&s)),
                    None => obj.set("streamed_text", auto_val::Value::str("")),
                }
                obj.set("duration_ms", auto_val::Value::Int(dur));
                // Plan 053 后续:补 exit_code,否则 exit_label computed 读不到字段
                // → "${exit_label}" 字面量。Plan 057:优先用后端透传的真实退出码
                // (ash-server CommandResult.exit_code,Plan 054 M2);merged 执行器
                // payload 不带,按 status 推导兜底。
                let exit_code = payload
                    .get("exit_code")
                    .and_then(|x| x.as_i64())
                    .map(|n| n as i32)
                    .unwrap_or(if kind == "Success" { 0 } else { 1 });
                obj.set("exit_code", auto_val::Value::Int(exit_code));
            }
            break;
        }
    }
    if found {
        // Write back the same type we read.
        let _ = match &raw {
            auto_val::Value::Array(_) | auto_val::Value::Nil => component.write_state(
                "blocks",
                auto_val::Value::Array(auto_val::Array { values: blocks_vec }),
            ),
            _ => component.write_state_vec("blocks", blocks_vec),
        };
    }
    found
}

/// Plan 063 T4: append chat-drawer lines to `chat_events`. Same ownership
/// model as job_list: the store-declared `List<ChatEvent>` is a VM-heap VmRef
/// that `write_state_vec` can't write back, so the renderer owns the field as
/// a `Value::Array` unconditionally (read-as-vec → extend → write_state).
pub(super) fn append_chat_events(component: &mut DynamicComponent, lines: Vec<auto_val::Value>) {
    let mut cur: Vec<auto_val::Value> = match component.read_state("chat_events") {
        Ok(auto_val::Value::Array(arr)) => arr.values.clone(),
        Ok(auto_val::Value::Nil) | Err(_) => Vec::new(),
        Ok(_) => component
            .read_state_as_vec("chat_events")
            .unwrap_or_default(),
    };
    cur.extend(lines);
    let _ = component.write_state(
        "chat_events",
        auto_val::Value::Array(auto_val::Array { values: cur }),
    );
}

/// Plan 063 T5: write the chat turn number onto the matching block's `turn`
/// field (block header banner "· 第 N 轮"). Mirrors update_block_in_state's
/// Obj-first / materialize-VmRef-on-miss match; only the matched element is
/// replaced. Non-chat blocks keep turn = 0.
pub(super) fn set_block_turn(component: &mut DynamicComponent, block_id: i64, turn: i64) {
    let raw = match component.read_state("blocks") {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut blocks_vec: Vec<auto_val::Value> = match &raw {
        auto_val::Value::Array(arr) => arr.values.clone(),
        auto_val::Value::Nil => Vec::new(),
        _ => match component.read_state_as_vec("blocks") {
            Ok(v) => v,
            Err(_) => return,
        },
    };
    let mut found = false;
    for b in &mut blocks_vec {
        if !matches!(b, auto_val::Value::Obj(_)) {
            let m = component.materialize_obj_value(b);
            match m {
                auto_val::Value::Obj(obj) => {
                    let id_matches = obj
                        .get("id")
                        .map(|v| v.as_int() as i64 == block_id)
                        .unwrap_or(false);
                    if !id_matches {
                        continue;
                    }
                    *b = auto_val::Value::Obj(obj);
                }
                _ => continue,
            }
        }
        if let auto_val::Value::Obj(obj) = b {
            let id_matches = obj
                .get("id")
                .map(|v| v.as_int() as i64 == block_id)
                .unwrap_or(false);
            if !id_matches {
                continue;
            }
            obj.set("turn", auto_val::Value::Int(turn as i32));
            found = true;
            break;
        }
    }
    if found {
        let _ = match &raw {
            auto_val::Value::Array(_) | auto_val::Value::Nil => component.write_state(
                "blocks",
                auto_val::Value::Array(auto_val::Array { values: blocks_vec }),
            ),
            _ => component.write_state_vec("blocks", blocks_vec),
        };
    }
}

/// Subscription:poll `SHELL_EVENT_RX`,把每条 shell 事件转成 IcedMessage,
/// 由 update 闭包派发到 store 的 RunOutput/RunResult handler(无参,读预置字段)。
/// Shell SSE 事件通道轮询（AppTickRecipe::Poll 用；读进程级静态通道）。
fn poll_shell_events() -> Option<IcedMessage> {
    let guard = SHELL_EVENT_RX.get_or_init(|| std::sync::Mutex::new(None));
    let mut lock = guard.lock().unwrap();
    let Some(rx) = lock.as_mut() else {
        return None;
    };
    // 取一条(非阻塞)。一次 update 处理一条事件,避免 handler 重入。
    match rx.try_recv() {
        Ok(ev) => Some(IcedMessage {
            widget: "ShellStore".to_string(),
            event: ev.event,
            input_value: Some(ev.payload_json),
        }),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
    }
}

pub(super) fn shell_event_subscription(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    // Plan 062(2026-08-23):间隔必须 ≠ mcp_action_subscription 的 16ms ——
    // 两个 `time::every` 同 duration 的订阅在 iced 的订阅表里 hash 相同,
    // 后注册者被去重、消息静默丢失(merged 模式 job_started/job_done 不到
    // 达 update 的根因;实测 16→17ms 后事件恢复投递)。459：Recipe 身份含
    // 轮询函数指针，天然互不去重，间隔差异不再是正确性前提（保留 17ms
    // 减少同拍抖动）。
    iced_futures::subscription::from_recipe(AppTickRecipe {
        app,
        kind: AppTickKind::Poll(poll_shell_events, 17),
    })
}

// ─── PLAN-083 T-01: 异步 HTTP 消息桥泵（C2 回填段） ─────────────────────────
//
// stdlib 侧 Http.get_msg(url, event) 入队的完成项在此回流事件循环：一次
// update 取一条（防 handler 重入，对齐 poll_shell_events），产
// IcedMessage{widget, "Event␟s␟<payload>"} 走 update 通用派发
// （on_with_input_for → decode_payload）——回填 handler 以字符串实参收
// {"ok","status","body"} JSON。发起段无 task Waiting（无忙等），UI 在
// 请求全程保持响应（对照：#[api] 走 get_json re-entry yield，call_fn_
// by_name 忙等 5ms×N 冻结 UI——082 §10-7 实测 20.9s 的根因）。
fn poll_http_msgs() -> Option<IcedMessage> {
    let done = auto_lang_core::vm::ffi::stdlib::http_msg_poll_one()?;
    // PLAN-084 诊断打点（临时）→ PLAN-765 F-6：改 env 门控（缺省关）。
    if std::env::var("AUTOUI_HTTP_BRIDGE_DIAG").is_ok() {
        let (name, _) = done.split_event();
        eprintln!("[msg-pump] dispatch widget={} event={}", done.widget, name);
    }
    // PLAN-765 T-02（纪要二 F-3）：入队侧已预编码 `handler␟s␟{json}`——
    // 此处零拷贝移交（原 format! 再拼一份全文拷贝）。
    Some(IcedMessage {
        widget: done.widget,
        event: done.event,
        input_value: None,
    })
}

pub(super) fn http_msg_subscription(
    app: crate::ui::session::AppId,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    // 19ms（≠ shell 17 / mcp 16，见 Plan 062 同拍抖动注；Recipe 身份含
    // 函数指针，去重安全）≈52fps——数据回填延迟人眼无感。
    // PLAN-765 T-02：PollBackfill 档——产出 DM::HttpBackfill（seq 门控派发）。
    iced_futures::subscription::from_recipe(AppTickRecipe {
        app,
        kind: AppTickKind::PollBackfill(poll_http_msgs, 19),
    })
}

/// Keyboard subscription: F12 devtools toggle + widget key bindings (Plan 275).
///
/// Uses `listen_with` (fn pointer) with a global `Arc<Mutex<HashMap>>` for bindings.
/// The subscription closure updates the global ref each time it's evaluated,
/// and the fn pointer reads from it.
/// Plan 459：per-App 键盘订阅 —— bindings 随闭包按 App 捕获（`listen_with`
/// 只收 fn 指针不可捕获，故走 `iced_futures::subscription::filter_map`），
/// `my_window` 过滤保证按键只路由到发生窗口的 App（多窗口互不串扰；
/// 单窗口行为与原实现等价）。订阅身份含 (AppId, window)，多 App 互不去重。
pub(super) fn keyboard_subscription(
    app: crate::ui::session::AppId,
    my_window: iced::window::Id,
    key_bindings: HashMap<String, String>,
    focused: bool,
    desktop_mode: bool,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    keyboard_subscription_ext(app, my_window, key_bindings, focused, desktop_mode, false)
}

/// Plan 464 T5：`escape_forward` —— Esc 被焦点输入框/IME 捕获（Captured）时
/// bind 路径收不到按键；launcher 的 Esc 逐层退出依赖宿主转发同一 .Escape
/// （幂等：handler 以 visible=="1" 门控，与 bind 双派发亦安全）。仅 launcher
/// overlay 的订阅传 true。
pub(super) fn keyboard_subscription_ext(
    app: crate::ui::session::AppId,
    my_window: iced::window::Id,
    key_bindings: HashMap<String, String>,
    focused: bool,
    desktop_mode: bool,
    escape_forward: bool,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    use crate::ui::session::DesktopMessage as DM;
    // Plan 462：identity 加入 focused —— desktop 模式焦点翻转即重订阅，
    // 闭包内按焦点门控（R12：键盘只进焦点虚拟窗口的 App）。
    iced_futures::subscription::filter_map(
        ("autoui-keyboard", app, my_window, focused),
        move |event: iced_futures::subscription::Event| {
            let iced_futures::subscription::Event::Interaction {
                window,
                event,
                status,
            } = event
            else {
                return None;
            };
            if window != my_window || !focused {
                return None;
            }
            if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = &event
                {
                    eprintln!("[464-KEY] app={app:?} key={key:?} status={status:?}");
                }
            }
            let is_escape = matches!(
                &event,
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                })
            );
            match keyboard_event_message(event, status, &key_bindings, !desktop_mode) {
                Some(m) => Some(DM::App(app, m)),
                // Captured 捕获的 Esc：bind 路径不可达 —— launcher 场景转发
                None if escape_forward && is_escape => Some(DM::App(
                    app,
                    IcedMessage {
                        widget: String::new(),
                        event: "Escape".to_string(),
                        input_value: None,
                    },
                )),
                None => None,
            }
        },
    )
}

/// PLAN-019 v1.7：picker 键盘门控（订阅闭包无会话状态访问——原子量桥接：
/// execute_wallpaper_pick 置位、close/return 复位。picker 开时 ←/→/Esc 消费
/// 为 picker 导航；负一屏无文本焦点窗，吞键无副作用。关态不拦截，App 文本
/// 编辑不受影响）。
pub(super) static PICKER_KEYS_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Plan 463 T3/T6：桌面级热键订阅（单份，按宿主窗过滤；App 焦点无关，
/// R12 桌面层路由的键盘半边）。
/// Plan 490 T2：逐臂硬编码布尔式退役，改查 `HotkeyTable`（键位数据级
/// 可配置；内置新默认 = G1 Alt+Tab 退役 + G2 分区切换迁 Ctrl+Alt+[ / ]；
/// launcher 主键 Ctrl+Space + 别名 Ctrl+Alt+Space IME 兜底双收显性化）。
pub(super) fn desktop_hotkey_subscription(
    my_window: iced::window::Id,
    hotkeys: crate::ui::session::HotkeyTable,
) -> iced::Subscription<crate::ui::session::DesktopMessage> {
    iced_futures::subscription::filter_map(
        ("autoui-desktop-hotkeys", my_window),
        move |event: iced_futures::subscription::Event| {
            let iced_futures::subscription::Event::Interaction { window, event, .. } = event else {
                return None;
            };
            if window != my_window {
                return None;
            }
            let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
                event
            else {
                return None;
            };
            // PLAN-019 v1.7：picker 键臂前置（门控见 PICKER_KEYS_OPEN）——
            // 优先于热键表，关态完全穿透。
            if PICKER_KEYS_OPEN.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(msg) = picker_key_message(&key) {
                    return Some(msg);
                }
            }
            // Plan 490 T2：逐臂硬编码退役，改查 HotkeyTable（含 488 T7
            // Ctrl+V→NativePaste 臂——热键域协调条款收编入 shell.keys.paste）。
            desktop_hotkey_message(&hotkeys, &modifiers, &key)
        },
    )
}

/// PLAN-019 v1.7：picker 键 → 消息的纯函数（订阅闭包可测化内核，
/// desktop_hotkey_message 同型）。←/→ = 导航，Esc = 逐级退回链；其余
/// 键 None（穿透热键表）。
pub(super) fn picker_key_message(key: &iced::keyboard::Key) -> Option<crate::ui::session::DesktopMessage> {
    use crate::ui::session::DesktopEvent as DE;
    use crate::ui::session::DesktopMessage as DM;
    use iced::keyboard::{key::Named, Key};
    match key {
        Key::Named(Named::ArrowLeft) => Some(DM::Desktop(DE::WallpaperKeyNav("prev"))),
        Key::Named(Named::ArrowRight) => Some(DM::Desktop(DE::WallpaperKeyNav("next"))),
        Key::Named(Named::Escape) => Some(DM::Desktop(DE::WallpaperKeyEscape)),
        _ => None,
    }
}

/// Plan 490 T2：桌面热键 → DesktopMessage 的表驱动纯函数（订阅闭包的
/// 可测化内核）。臂序 = 旧行为优先级保持（Esc → switcher → send_to →
/// 布局 → 分区 → launcher；CycleWindow 内置无臂，storage 复活即生效）。
fn desktop_hotkey_message(
    hotkeys: &crate::ui::session::HotkeyTable,
    modifiers: &iced::keyboard::Modifiers,
    key: &iced::keyboard::Key,
) -> Option<crate::ui::session::DesktopMessage> {
    use crate::ui::session::HotkeyAction as HA;
    use crate::ui::session::{DesktopMessage as DM, WmCommand, WorkspaceStep};

    if hotkeys.matches(HA::ExitDesktop, modifiers, key) {
        return Some(DM::Wm(WmCommand::ExitDesktop));
    }
    if hotkeys.matches(HA::CycleSwitcher, modifiers, key) {
        return Some(DM::Desktop(
            crate::ui::session::DesktopEvent::SummonSwitcher,
        ));
    }
    if hotkeys.matches(HA::CycleWindow, modifiers, key) {
        return Some(DM::Wm(WmCommand::CycleWindow));
    }
    if hotkeys.matches(HA::SendToNext, modifiers, key) {
        return Some(DM::Wm(WmCommand::SendFocusedTo(WorkspaceStep::Next)));
    }
    if hotkeys.matches(HA::SendToPrev, modifiers, key) {
        return Some(DM::Wm(WmCommand::SendFocusedTo(WorkspaceStep::Prev)));
    }
    if hotkeys.matches(HA::SetLayoutGrid, modifiers, key) {
        return Some(DM::Wm(WmCommand::SetLayout(
            crate::ui::layout::LayoutMode::Grid,
        )));
    }
    if hotkeys.matches(HA::SetLayoutStack, modifiers, key) {
        return Some(DM::Wm(WmCommand::SetLayout(
            crate::ui::layout::LayoutMode::MasterStack,
        )));
    }
    if hotkeys.matches(HA::SetLayoutFree, modifiers, key) {
        return Some(DM::Wm(WmCommand::SetLayout(
            crate::ui::layout::LayoutMode::Free,
        )));
    }
    if hotkeys.matches(HA::WorkspaceNext, modifiers, key) {
        return Some(DM::Wm(WmCommand::NextWorkspace));
    }
    if hotkeys.matches(HA::WorkspacePrev, modifiers, key) {
        return Some(DM::Wm(WmCommand::PrevWorkspace));
    }
    if hotkeys.matches(HA::SummonLauncher, modifiers, key) {
        return Some(DM::Desktop(
            crate::ui::session::DesktopEvent::SummonLauncher,
        ));
    }
    // Plan 488 T7（490 收编）：Ctrl+V 粘贴路由（表驱动化——shell.keys.paste
    // 可覆盖；update 臂语义未动）。
    if hotkeys.matches(HA::Paste, modifiers, key) {
        return Some(DM::Desktop(crate::ui::session::DesktopEvent::NativePaste));
    }
    None
}

#[cfg(test)]
mod hotkey_subscription_tests {
    use super::*;
    use crate::ui::session::{DesktopMessage as DM, HotkeyTable, WmCommand};
    use iced::keyboard::{key::Named, Key, Modifiers};

    fn named(n: Named) -> Key {
        Key::Named(n)
    }

    fn mods(ctrl: bool, alt: bool, shift: bool) -> Modifiers {
        let mut m = Modifiers::default();
        if ctrl {
            m |= Modifiers::CTRL;
        }
        if alt {
            m |= Modifiers::ALT;
        }
        if shift {
            m |= Modifiers::SHIFT;
        }
        m
    }

    /// Plan 490 T2 订阅行为测：新默认表逐臂过纯函数内核
    /// `desktop_hotkey_message`（Alt+Tab 无臂 / bracket 分区 / 覆盖恢复 /
    /// 双收 / 既有臂不回归）。
    #[test]
    fn hotkey_sub_builtin_arms() {
        let t = HotkeyTable::builtin();

        // G1：Alt+Tab → None（退役；switcher 承担窗口循环）
        assert!(
            desktop_hotkey_message(&t, &mods(false, true, false), &named(Named::Tab)).is_none()
        );

        // G2：Ctrl+Alt+] / [ → 分区环切
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, false), &Key::Character("]".into())),
            Some(DM::Wm(WmCommand::NextWorkspace))
        ));
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, false), &Key::Character("[".into())),
            Some(DM::Wm(WmCommand::PrevWorkspace))
        ));
        // 旧默认方向键 → None（可覆盖恢复）
        assert!(
            desktop_hotkey_message(&t, &mods(true, true, false), &named(Named::ArrowRight))
                .is_none()
        );

        // 既有臂不回归（PLAN-526 T13：Escape→ExitDesktop 已退役——
        // 裸 Esc 不再出任何 WM 命令）
        assert!(
            desktop_hotkey_message(&t, &mods(false, false, false), &named(Named::Escape)).is_none()
        );
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, false, false), &named(Named::Tab)),
            Some(DM::Desktop(
                crate::ui::session::DesktopEvent::SummonSwitcher
            ))
        ));
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, true), &named(Named::ArrowRight)),
            Some(DM::Wm(WmCommand::SendFocusedTo(_)))
        ));
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, false), &Key::Character("g".into())),
            Some(DM::Wm(WmCommand::SetLayout(_)))
        ));

        // Plan 488 T7（490 收编）：Ctrl+V 粘贴臂表驱动（Ctrl+Shift+V 不收）
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, false, false), &Key::Character("v".into())),
            Some(DM::Desktop(crate::ui::session::DesktopEvent::NativePaste))
        ));
        assert!(
            desktop_hotkey_message(&t, &mods(true, false, true), &Key::Character("v".into()))
                .is_none(),
            "Ctrl+Shift+V 留给 App 层"
        );

        // launcher 双收（主键 + IME 兜底别名）
        let space = named(Named::Space);
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, false, false), &space),
            Some(DM::Desktop(
                crate::ui::session::DesktopEvent::SummonLauncher
            ))
        ));
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, false), &space),
            Some(DM::Desktop(
                crate::ui::session::DesktopEvent::SummonLauncher
            ))
        ));
    }

    /// Plan 490 T2/T3：storage 覆盖后的表驱动行为——方向键恢复生效、
    /// CycleWindow 显式复活（G1 逃生舱）。
    #[test]
    fn hotkey_sub_storage_override_arms() {
        let mut t = HotkeyTable::builtin();
        assert!(t.apply_override("workspace_next", "ctrl+alt+right"));
        assert!(t.apply_override("workspace_prev", "ctrl+alt+left"));
        assert!(t.apply_override("cycle_window", "alt+tab"));

        assert!(matches!(
            desktop_hotkey_message(&t, &mods(true, true, false), &named(Named::ArrowRight)),
            Some(DM::Wm(WmCommand::NextWorkspace))
        ));
        assert!(matches!(
            desktop_hotkey_message(&t, &mods(false, true, false), &named(Named::Tab)),
            Some(DM::Wm(WmCommand::CycleWindow))
        ));
    }

    /// Plan 490 T3 storage 往返：publish 覆盖键 → load_hotkey_overrides 读
    /// 回 → apply_override 表生效；坏值/未知键静默跳过不 panic；缺席 =
    /// 空（默认表）。storage 隔离沿 479/489 铁律（AUTO_VM_STORAGE_FILE
    /// 独立临时文件——进程级全局 + cwd 哈希落盘双重污染面）。
    #[test]
    fn hotkey_storage_boot_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "auto-490-hotkeys-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);

        // 空库 → 无覆盖
        assert!(load_hotkey_overrides().is_empty(), "缺席键 = 不覆盖");

        // 合法覆盖 + 坏值 + 未知动作键共存
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.keys.workspace_next",
            "ctrl+alt+right".to_string(),
        );
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.keys.workspace_prev",
            "ctrl+alt+!!!bad".to_string(),
        );
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.keys.cycle_window",
            "alt+tab".to_string(),
        );

        let overrides = load_hotkey_overrides();
        assert_eq!(overrides.len(), 3, "三键均读回（好坏由 apply 判）");

        let mut t = HotkeyTable::builtin();
        for (k, v) in &overrides {
            t.apply_override(k, v); // 坏值静默拒绝——不 panic
        }
        assert!(
            matches!(
                desktop_hotkey_message(&t, &mods(true, true, false), &named(Named::ArrowRight)),
                Some(DM::Wm(WmCommand::NextWorkspace))
            ),
            "合法覆盖生效（方向键恢复=Intel 冲突机逃生舱）"
        );
        assert!(
            matches!(
                desktop_hotkey_message(&t, &mods(true, true, false), &Key::Character("[".into())),
                Some(DM::Wm(WmCommand::PrevWorkspace))
            ),
            "坏值保留缺省（bracket 仍生效）"
        );
        assert!(
            matches!(
                desktop_hotkey_message(&t, &mods(false, true, false), &named(Named::Tab)),
                Some(DM::Wm(WmCommand::CycleWindow))
            ),
            "G1 逃生舱：storage 显式复活 Alt+Tab"
        );

        let _ = std::fs::remove_file(&path);
    }
}

/// 键盘/窗口事件 → IcedMessage（原 listen_with 闭包体的纯函数化；
/// F12 toggle、Captured 过滤、bindings 查找、shifted 兜底、Tab 聚焦）。
/// `allow_devtools=false`（Plan 462 desktop 模式）时 F12 不展开 DevTools
/// 面板（固定 widget id 在同窗多 App 下会互相撞车，见订阅处注记）。
pub(super) fn keyboard_event_message(
    event: iced::Event,
    status: iced::event::Status,
    key_bindings: &HashMap<String, String>,
    allow_devtools: bool,
) -> Option<IcedMessage> {
    // F12 → DevTools toggle (always active, even when a widget has focus)
    // Must be checked BEFORE the Captured guard, otherwise F12 is swallowed
    // when a text input is focused (Plan 371: F12 reliability fix).
    if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = &event {
        if allow_devtools
            && matches!(
                key,
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)
            )
        {
            return Some(IcedMessage {
                widget: String::new(),
                event: DEBUG_TOGGLE_EVENT.to_string(),
                input_value: None,
            });
        }
    }

    // Skip events already consumed by a focused widget (e.g., text input)
    if matches!(status, iced::event::Status::Captured) {
        return None;
    }

    match event {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            // Build key string for lookup
            let key_str = match &key {
                // Named keys
                iced::keyboard::Key::Named(named) => {
                    let name = match named {
                        iced::keyboard::key::Named::Enter => "Enter",
                        iced::keyboard::key::Named::Escape => "Escape",
                        iced::keyboard::key::Named::Backspace => "Backspace",
                        iced::keyboard::key::Named::Tab => "Tab",
                        iced::keyboard::key::Named::Space => " ",
                        iced::keyboard::key::Named::ArrowUp => "ArrowUp",
                        iced::keyboard::key::Named::ArrowDown => "ArrowDown",
                        iced::keyboard::key::Named::ArrowLeft => "ArrowLeft",
                        iced::keyboard::key::Named::ArrowRight => "ArrowRight",
                        iced::keyboard::key::Named::Delete => "Delete",
                        iced::keyboard::key::Named::Home => "Home",
                        iced::keyboard::key::Named::End => "End",
                        _ => return None,
                    };
                    name.to_string()
                }
                // Character keys — raw character from OS, no case normalization.
                // "s" and "S" are different keys. "S" = Shift+s. "Ctrl+S" = Ctrl+Shift+s.
                // With Ctrl/Alt held, prepend modifier prefix to the raw character.
                iced::keyboard::Key::Character(c) => {
                    if modifiers.control() || modifiers.alt() {
                        let mut prefix = String::new();
                        if modifiers.control() {
                            prefix.push_str("Ctrl+");
                        }
                        if modifiers.alt() {
                            prefix.push_str("Alt+");
                        }
                        format!("{}{}", prefix, c)
                    } else {
                        c.to_string()
                    }
                }
                _ => return None,
            };

            // Look up handler from this App's captured bindings
            // Platform compatibility: on Windows, Shift+= returns Character("=") with
            // SHIFT modifier (NOT Character("+")). This fallback maps the base key to its
            // shifted symbol so bind { "+" -> ... } works on all platforms.
            // Only applies when no Ctrl/Alt modifier is held.
            let handler: Option<String> = key_bindings
                .get(&key_str)
                .cloned()
                .or_else(|| {
                    if modifiers.shift() && !modifiers.control() && !modifiers.alt() {
                        let shifted_map: &[(&str, &str)] =
                            &[("=", "+"), ("8", "*"), ("-", "_"), ("/", "?")];
                        shifted_map
                            .iter()
                            .find(|(from, _)| *from == key_str.as_str())
                            .and_then(|(_, to)| key_bindings.get(*to))
                            .cloned()
                    } else {
                        None
                    }
                })
                // Plan 418 P2-4: config-declared shortcuts (auto-edit.at via
                // pac.at `ui_config:`) are a fallback layer UNDER the
                // DSL-declared bindings — same key lookup form ("Ctrl+n").
                .or_else(|| {
                    crate::ui::action_config::action_config()
                        .and_then(|cfg| cfg.handler_for_key(&key_str).map(str::to_owned))
                });
            if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
                eprintln!("[464-BIND] key_str={key_str:?} handler={handler:?} status={status:?}");
            }
            if let Some(handler) = handler {
                // Strip the leading dot from ".Digit1" → "Digit1"
                let event_name = if let Some(stripped) = handler.strip_prefix('.') {
                    stripped.to_string()
                } else {
                    handler.clone()
                };
                Some(IcedMessage {
                    widget: String::new(),
                    event: event_name,
                    input_value: None,
                })
            } else if key_str == "Escape" {
                // PLAN-646: Esc → 采集面板关闭/清 marquee。app 级 Escape
                // binding 优先（上方 handler 命中即短路）；无 binding 时
                // 发 __select_esc，update 臂在无采集态时忽略（no-op）。
                Some(IcedMessage {
                    widget: String::new(),
                    event: "__select_esc".to_string(),
                    input_value: None,
                })
            } else if key_str == "Tab" {
                // Plan 491（057 语义收窄为「无 input 聚焦时」）：事件到达
                // 这里 = 未被任何聚焦件捕获（iced text_input 无 Tab 臂；
                // prompt 编辑器持焦时其 Tab 补全为 Captured，不会到达此
                // 臂）。按 shift 分派焦点环遍历：Tab → 前进（下一项，尾
                // 回环），Shift+Tab → 后退（上一项，首回环）；当前无
                // input 聚焦/无 input 视图时 update 臂回落登记表首个或
                // 057 prompt 链（等价旧 __focus_prompt 语义）。
                // Named 臂不加修饰前缀——Shift+Tab 与 Tab 同命中 "Tab"，
                // 须在此按 modifiers 分流（旧状：Shift+Tab 被当前进处理）。
                let event = if modifiers.shift() {
                    FOCUS_PREV_INPUT_EVENT
                } else {
                    FOCUS_NEXT_INPUT_EVENT
                };
                Some(IcedMessage {
                    widget: String::new(),
                    event: event.to_string(),
                    input_value: None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) const DEBUG_TOGGLE_EVENT: &str = "__toggle_debug";
/// Plan 057 (ash-gui)：无 input 视图的未捕获 Tab → prompt 编辑器聚焦。
/// Plan 491 后无生产派发点（Tab 分派改 next/prev 两事件）；消费臂保留
/// 供无 input 视图回落与外部注入兼容。
pub(super) const FOCUS_PROMPT_EVENT: &str = "__focus_prompt";
/// Plan 491: 未捕获 Tab → input 焦点环前进（下一项，尾回环；无聚焦/不在
/// 登记表 → 首项）。
pub(super) const FOCUS_NEXT_INPUT_EVENT: &str = "__focus_next_input";
/// Plan 491: 未捕获 Shift+Tab → input 焦点环后退（上一项，首回环；无聚焦/
/// 不在登记表 → 首项，与 next 同臂）。
pub(super) const FOCUS_PREV_INPUT_EVENT: &str = "__focus_prev_input";
pub(super) const DEBUG_HOVER_MOVE: &str = "__hover_";
pub(super) const DEBUG_HOVER_EXIT: &str = "__hover_exit_";
pub(super) const DEBUG_SELECT_PREFIX: &str = "__select_";
pub(super) const DEBUG_EDIT_PREFIX: &str = "__edit_";
pub(super) const DEBUG_EDIT_APPLY: &str = "__edit_apply";
pub(super) const DEBUG_EDIT_CANCEL: &str = "__edit_cancel";
pub(super) const SRC_CLICK_PREFIX: &str = "__src_click_";
/// Select a VNode by its u64 id (Plan 307 Task 14): `__vnode_select_<id>`.
///
/// NOTE (Plan 309 续篇): this prefix MUST NOT share a leading segment with
/// `DEBUG_SELECT_PREFIX` (`"__select_"`). The select-widget handler does
/// `event.strip_prefix("__select_")`, and `"__select_vnode_"` is prefixed by
/// it — so a tree-node click message was hijacked by the widget handler (id
/// misread as `"vnode_42"`) and the VNode block never ran, making tree nodes
/// un-selectable. Renaming the prefix breaks the overlap.
pub(super) const DEBUG_SELECT_VNODE_PREFIX: &str = "__vnode_select_";
/// Switch the inspector right-panel inner sub-tab (Plan 307 Task 15):
/// `__inspector_subtab_<Variant>`.
pub(super) const DEBUG_INSPECTOR_SUBTAB_PREFIX: &str = "__inspector_subtab_";
/// Toggle a collapsible section inside the 检视 sub-tab (Plan 307 续篇 IV):
/// `__inspector_section_<box|computed|props>`.
pub(super) const DEBUG_INSPECTOR_SECTION_PREFIX: &str = "__inspector_section_";
/// PLAN-646 Select Anything: switch the Select tab view — `__select_view_<Auto|Json|Atom>`.
pub(super) const SELECT_VIEW_PREFIX: &str = "__select_view_";

/// DevTools panel top-level mode (Plan 309 续篇: 元素树与检视已统一为同屏
/// 分屏，不再是互斥 tab；控制台仍为独立整宽模式).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DevToolsTab {
    /// 同屏分屏：左元素树 (VTree) | 右检视 (面包屑 + 子标签).
    Inspect,
    /// 控制台占满整宽.
    Console,
    /// PLAN-646 Select Anything：框选结果信封（Auto/JSON/Atom 三视图 + 复制）。
    Select,
}

/// Inspector right-panel inner sub-tab (Plan 307 Task 15; 续篇 IV collapsed
/// Box/Computed/Properties into the single 检视 tab). AutoUI and Source remain
/// standalone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InspectorSubTab {
    /// Combined: a single scrollable column of collapsible Box / Computed /
    /// Properties sections (Chrome-DevTools style).
    Inspect,
    AutoUI,
    Source,
}

