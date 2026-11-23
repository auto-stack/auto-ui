//! ImageSurface native input adapter (PLAN-089 T-01/T-02 修正适配)。
//!
//! Plan 547 Task 25 的缺口：native 渲染路径此前没有任何输入接线——
//! `View::ImageSurface` 的 `on_wheel/on_pan/on_double_click` 被转换成
//! 无参 `IcedMessage` 后由 paint pass 显式忽略（projection "consumed by
//! the input adapter (Task 25)" 注释），wheel/pan/dblclick 事件从未到达
//! `.at` handler。本模块按 PointerArea（Plan 499）/`PointerMoveHandler`
//! 先例补上这段适配：
//!
//! 1. [`ImageSurfaceCallback`]：视图节点携带的载荷回调（泛型 M），builder
//!    期由「基消息 + 绑定位参数」构造——绑定位声明了参数（`$0` 形态）则
//!    事件期追加该 kind 的固定载荷，空参绑定保持无参派发（与 Vue 生成器
//!    「空声明 → 无参调用」逐字节同语义，031 等旧绑定零变化）。
//! 2. [`ImageSurfaceInput`]：iced 输入包装 widget——wheel 滚轮、左键拖拽
//!    三相 pan、双击检测，经 `normalize_image_surface_event` +
//!    `ImageSurfaceInputState`（src revision 门/单次语义与底层合同一致）
//!    后调用回调发布消息。
//!
//! 载荷形状（跨后端冻结，包 README 同步）：
//! - wheel → `(delta_y)`（f32；锚点沿 image_surface_geometry 的视口中心，
//!   不用光标坐标另起第二套变换算法）
//! - pan → `(dx, dy, phase)`，phase 为字符串 `"start"|"move"|"end"`（与
//!   Vue 生成器 surface_pans 包装的 `(dx, dy, 'start'|'move'|'end')` 对齐）
//! - dblclick → v1 无载荷（toggle 1:1 语义，同 031）
//!
//! Vue 侧 wheel 参数由生成器 `$event.deltaY` 替换承担（见
//! ui_gen/vue/shadcn_attrs.rs vue_event_param）。
use super::image_surface::{
    normalize_image_surface_event, ImageSurfaceEvent, ImageSurfaceInputEvent,
    ImageSurfaceInputState, ImageSurfacePanPhase,
};
use crate::ui::interpreter::DynamicMessage;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{Element, Event, Point, Rectangle, Size, Vector};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 双击判定窗口：两次左键按下间隔与位移阈值（iced mouse_area 双击语义的
/// 自持实现，后端版本无关）。
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
const DOUBLE_CLICK_SLOP: f32 = 6.0;

/// 图像面输入载荷（规范化后的最小携带；phase 用事件期字符串）。
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSurfaceCarry {
    Wheel { delta_y: f32, x: f32, y: f32 },
    Pan { dx: f32, dy: f32, phase: ImageSurfacePanPhase },
    DoubleClick { x: f32, y: f32 },
}

impl ImageSurfaceCarry {
    /// 跨后端冻结的载荷投影（包 README 同步）：wheel=1 参 delta_y；pan=3 参
    /// (dx, dy, phase 字符串)；dblclick=0 参。
    pub fn payload_args(&self) -> Vec<auto_val::Value> {
        match self {
            ImageSurfaceCarry::Wheel { delta_y, .. } => {
                vec![auto_val::Value::Double(*delta_y as f64)]
            }
            ImageSurfaceCarry::Pan { dx, dy, phase } => vec![
                auto_val::Value::Double(*dx as f64),
                auto_val::Value::Double(*dy as f64),
                auto_val::Value::Str(phase.as_str().into()),
            ],
            ImageSurfaceCarry::DoubleClick { .. } => Vec::new(),
        }
    }
}

/// 图像面事件回调（`PointerMoveHandler` 同款视图层回调：事件期把规范化
/// 载荷并入 handler 消息）。`declared = false`（绑定位无参）时恒发基
/// 消息、零载荷——与 Vue「空声明 → 无参调用」对齐，旧绑定零变化。
pub struct ImageSurfaceCallback<M> {
    callback: Arc<dyn Fn(ImageSurfaceCarry) -> M + Send + Sync>,
}

impl<M> std::fmt::Debug for ImageSurfaceCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageSurfaceCallback").finish()
    }
}

impl<M> Clone for ImageSurfaceCallback<M> {
    fn clone(&self) -> Self {
        Self {
            callback: Arc::clone(&self.callback),
        }
    }
}

impl ImageSurfaceCallback<DynamicMessage> {
    /// 从基消息构造：`declared` 为假 → 事件期回基消息（无载荷）；为真 →
    /// 规范化载荷按 kind 固定形状追加。基消息携带的绑定位字面量参数保留
    /// 在前（`onclick: .SelectDay(cell.date)` 同语义）。
    pub fn from_base(base: DynamicMessage, declared: bool) -> Self {
        let callback = Arc::new(move |carry: ImageSurfaceCarry| match &base {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                args,
            } => {
                let mut out = args.clone();
                if declared {
                    out.extend(carry.payload_args());
                }
                DynamicMessage::Typed {
                    widget_name: widget_name.clone(),
                    event_name: event_name.clone(),
                    args: out,
                }
            }
            DynamicMessage::TypedInst {
                widget_name,
                event_name,
                args,
                instance,
            } => {
                let mut out = args.clone();
                if declared {
                    out.extend(carry.payload_args());
                }
                DynamicMessage::TypedInst {
                    widget_name: widget_name.clone(),
                    event_name: event_name.clone(),
                    args: out,
                    instance: instance.clone(),
                }
            }
            DynamicMessage::String(name) => DynamicMessage::String(name.clone()),
        });
        Self { callback }
    }
}

impl<M: 'static> ImageSurfaceCallback<M> {
    pub fn call(&self, carry: ImageSurfaceCarry) -> M {
        (self.callback)(carry)
    }

    /// 消息类型重映射（convert_view_messages 的 DynamicMessage → IcedMessage
    /// 腿；MediaEventHandler::new(move |p| f(cb.call(p))) 同款）。
    pub fn map_message<N: Clone + 'static>(
        &self,
        f: Arc<dyn Fn(M) -> N + Send + Sync>,
    ) -> ImageSurfaceCallback<N> {
        let inner = Arc::clone(&self.callback);
        ImageSurfaceCallback {
            callback: Arc::new(move |carry| f(inner(carry))),
        }
    }

    /// 转换腿的原始闭包构造（生成 Rust 轨恒发纯消息；VM 轨用 from_base）。
    pub fn from_closure(
        callback: Arc<dyn Fn(ImageSurfaceCarry) -> M + Send + Sync>,
    ) -> Self {
        Self { callback }
    }
}

/// 每实例输入簿记（iced Tree 状态；src revision 变更即重置拖拽/门）。
#[derive(Debug, Default)]
pub struct InputStateHandle {
    surface: Option<ImageSurfaceInputState>,
    last_press: Option<(Instant, Point)>,
}

/// 纯函数：wheel 滚动增量（行型按滚轮原始行计；像素型直接取 y——与
/// session live_input_from_wheel 的语义分层一致，缩放步长由消费方定标）。
pub fn wheel_delta_y(delta: &iced::mouse::ScrollDelta) -> f32 {
    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => *y,
        iced::mouse::ScrollDelta::Pixels { y, .. } => *y,
    }
}

/// 纯函数：双击判定（时间窗 + 位移 slop）；每次按下都刷新 last_press。
pub fn is_double_click(
    last_press: &mut Option<(Instant, Point)>,
    now: Instant,
    position: Point,
) -> bool {
    let double = match last_press {
        Some((at, at_point)) => {
            now.duration_since(*at) <= DOUBLE_CLICK_WINDOW
                && (position.x - at_point.x).abs() <= DOUBLE_CLICK_SLOP
                && (position.y - at_point.y).abs() <= DOUBLE_CLICK_SLOP
        }
        None => false,
    };
    *last_press = Some((now, position));
    double
}

/// ImageSurface 输入包装 widget（PointerArea 先例：bounds 现场持有、纯
/// 委托布局/绘制，只在目标事件上叠加发布，不捕获事件）。
pub struct ImageSurfaceInput<'a, M>
where
    M: Clone + 'static,
{
    content: Element<'a, M>,
    src_revision: u64,
    on_wheel: Option<ImageSurfaceCallback<M>>,
    on_pan: Option<ImageSurfaceCallback<M>>,
    on_double_click: Option<ImageSurfaceCallback<M>>,
}

impl<'a, M> ImageSurfaceInput<'a, M>
where
    M: Clone + 'static,
{
    pub fn new(
        content: Element<'a, M>,
        src_revision: u64,
        on_wheel: Option<ImageSurfaceCallback<M>>,
        on_pan: Option<ImageSurfaceCallback<M>>,
        on_double_click: Option<ImageSurfaceCallback<M>>,
    ) -> Self {
        Self {
            content,
            src_revision,
            on_wheel,
            on_pan,
            on_double_click,
        }
    }

    fn any_callback(&self) -> bool {
        self.on_wheel.is_some() || self.on_pan.is_some() || self.on_double_click.is_some()
    }
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for ImageSurfaceInput<'_, M>
where
    M: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<InputStateHandle>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(InputStateHandle::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        // src revision 变化 → 重置门/拖拽（底层 set_source_revision 同语义）。
        let handle = tree.state.downcast_mut::<InputStateHandle>();
        match &mut handle.surface {
            Some(state) => state.set_source_revision(self.src_revision),
            spot => *spot = Some(ImageSurfaceInputState::new(self.src_revision)),
        }
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if self.any_callback() {
            let bounds = layout.bounds();
            // 同一 handle 的两字段不相交借用（pan 门 vs 双击计时）。
            let InputStateHandle { surface, last_press } =
                tree.state.downcast_mut::<InputStateHandle>();
            let surface_state =
                surface.get_or_insert_with(|| ImageSurfaceInputState::new(self.src_revision));
            // normalize 的 viewport 参数取原点尺寸——cursor.position_in 已
            // 给出 surface-local 坐标，无需再做一次平移换算。
            let local_viewport = Rectangle::new(Point::ORIGIN, bounds.size());
            let revision = self.src_revision;
            let mut publishes: Vec<(usize, ImageSurfaceCarry)> = Vec::new();
            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    if self.on_wheel.is_some() {
                        if let Some(local) = cursor.position_in(bounds) {
                            if let Some(event) = normalize_image_surface_event(
                                surface_state,
                                local_viewport,
                                revision,
                                ImageSurfaceInputEvent::Wheel {
                                    delta_y: wheel_delta_y(delta),
                                    cursor: local,
                                },
                            ) {
                                publishes.push((0, carry_into(event)));
                            }
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if self.on_pan.is_some() {
                        if let Some(local) = cursor.position_in(bounds) {
                            if let Some(event) = normalize_image_surface_event(
                                surface_state,
                                local_viewport,
                                revision,
                                ImageSurfaceInputEvent::PanStart { cursor: local },
                            ) {
                                publishes.push((1, carry_into(event)));
                            }
                        }
                    }
                    if self.on_double_click.is_some() {
                        if let Some(local) = cursor.position_in(bounds) {
                            if is_double_click(last_press, Instant::now(), local) {
                                publishes.push((2, ImageSurfaceCarry::DoubleClick {
                                    x: local.x,
                                    y: local.y,
                                }));
                            }
                        }
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if self.on_pan.is_some() {
                        if let Some(local) = cursor.position_in(bounds) {
                            if let Some(event) = normalize_image_surface_event(
                                surface_state,
                                local_viewport,
                                revision,
                                ImageSurfaceInputEvent::PanMove { cursor: local },
                            ) {
                                publishes.push((1, carry_into(event)));
                            }
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if self.on_pan.is_some() {
                        if let Some(event) = normalize_image_surface_event(
                            surface_state,
                            local_viewport,
                            revision,
                            ImageSurfaceInputEvent::PanEnd,
                        ) {
                            publishes.push((1, carry_into(event)));
                        }
                    }
                }
                _ => {}
            }
            for (kind, carry) in publishes {
                let message = match kind {
                    0 => self.on_wheel.as_ref().map(|cb| cb.call(carry)),
                    1 => self.on_pan.as_ref().map(|cb| cb.call(carry)),
                    _ => self.on_double_click.as_ref().map(|cb| cb.call(carry)),
                };
                if let Some(message) = message {
                    shell.publish(message);
                }
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

fn carry_into(event: ImageSurfaceEvent) -> ImageSurfaceCarry {
    match event {
        ImageSurfaceEvent::Wheel { delta_y, x, y } => ImageSurfaceCarry::Wheel { delta_y, x, y },
        ImageSurfaceEvent::Pan { dx, dy, phase } => ImageSurfaceCarry::Pan { dx, dy, phase },
        ImageSurfaceEvent::DoubleClick { x, y } => ImageSurfaceCarry::DoubleClick { x, y },
        ImageSurfaceEvent::Loaded { .. } | ImageSurfaceEvent::Error { .. } => {
            unreachable!("loaded/error ride the session media notify sweep, not input")
        }
    }
}

impl<'a, M> From<ImageSurfaceInput<'a, M>> for Element<'a, M>
where
    M: Clone + 'static,
{
    fn from(widget: ImageSurfaceInput<'a, M>) -> Self {
        Element::new(widget)
    }
}
