// PLAN-683（方案 2：远程 Renderer）—— headless iced 宿主。
//
// **架构**（T-00 spike 定案，替代原计划的"自研 RecordRenderer trait 实现"）：
// `iced::Renderer` 在本仓 feature 集下 = `iced_renderer::fallback::Renderer
// <iced_wgpu::Renderer, iced_tiny_skia::Renderer>` 公开枚举——
// `Renderer::Secondary(iced_tiny_skia::Renderer::new(..))` 纯 CPU 构造
//（零 wgpu/零窗口），与 `view.into_iced()` 产出的组件树（实例化于
// `iced::Renderer`）**类型同源**，`UserInterface::draw` 可直接驱动。
// tiny_skia 后端是延迟栅格化设计：draw 期原语只被记录进
// `layers()`（quads/text/images + 各自 clip 层），本模块把这些记录降格为
// 协议 DrawList 过线——iced 组件树的布局/命中/聚焦/IME 全原生，渲染语义
// 与独立轨同出一份 iced 实现（"函数调用变 RPC"的字面达成）。
//
// 与设计文档 rq-remote-renderer §4 的偏差登记：原设想的 RecordRenderer
// = 手写 `iced_core::Renderer` + 文本子 trait 全表面；实勘发现该路线须把
// 仓内 3.5 万行 `into_iced()` 适配层按 Renderer 泛型化（Element 树在构造
// 期钉死具体 Renderer 类型），成本不可行。tiny_skia Secondary 臂是同一
// 截获语义的现成载体（其 Layer 即"记录原语的 display list"），且官方
// CPU 后端长期维护。详见 T-00 决策档。

use crate::ui::component::Component;
use crate::ui::desktop_protocol::message::{
    BorderSpec, ControlMsg, DisplayList, DisplayOp, DrawList, DrawOp, Fill, ImeReq, PathSeg, Rgba8,
    ShadowSpec, WirePath, WireStroke, WRect,
};
use crate::ui::iced::renderer::IntoIcedElement;

use iced::advanced::graphics::text::font_system;
use iced::advanced::graphics::text::Text as GfxText;
use iced::advanced::input_method::{InputMethod, Purpose};
use iced::advanced::renderer as advanced_renderer;
use iced::mouse;
use iced::theme::Base;
use iced::Element;
use iced_runtime::user_interface::{Cache, State as UiState, UserInterface};
use iced_tiny_skia::Layer;

/// 字体装载（进程内一次）：独立轨 application 链同款 Inter 三字重
///（renderer.rs Plan 411 P1-C），中文等族外字形走系统回退（cosmic-text）。
/// daemon 侧重整行盒所需字体经 FontBlob 握手通道下发（413 §7.2）。
pub(crate) fn ensure_headless_fonts() {
    use std::sync::Once;
    static FONTS: Once = Once::new();
    FONTS.call_once(|| {
        // PLAN-062 T-02：装载即登记字体内容身份——glyph run 的 Host 侧按
        // 内容哈希解析字体，不依赖路径/进程内 ID。
        for bytes in [
            crate::ui::iced::renderer::INTER_FONT_REGULAR,
            crate::ui::iced::renderer::INTER_FONT_MEDIUM,
            crate::ui::iced::renderer::INTER_FONT_SEMIBOLD,
        ] {
            let _ = super::glyph_fonts::register_font_bytes(bytes.to_vec());
        }
    });
}

/// PLAN-063 T-03（rq-fidelity-contract §3 缺口"剪贴板 Null"关闭）：
/// headless/远程客户端的 iced widget 剪贴板面——OS 桥（arboard，与
/// direct 的 handler 面 [crate::ui::clipboard] 同源）。此前 `clipboard::Null`
/// 使 widget 内 Ctrl+C/V 复制粘贴在远程轨静默失效（direct winit 臂读
/// 同一 OS 剪贴板——同源 direct 是兼容基准）。Primary（X11/Wayland
/// 选择区）在 Windows 无对应：read None / write 忽略，与 direct winit
/// 行为同律；Clipboard 族往返与可用性由 V-02 真实负控核验。
pub struct OsClipboard;

impl iced::advanced::clipboard::Clipboard for OsClipboard {
    fn read(&self, kind: iced::advanced::clipboard::Kind) -> Option<String> {
        match kind {
            iced::advanced::clipboard::Kind::Standard => crate::ui::clipboard::clipboard_get(),
            iced::advanced::clipboard::Kind::Primary => None,
        }
    }

    fn write(&mut self, kind: iced::advanced::clipboard::Kind, contents: String) {
        if kind == iced::advanced::clipboard::Kind::Standard {
            crate::ui::clipboard::clipboard_set(&contents);
        }
    }
}

#[cfg(test)]
mod p063_clipboard_tests {
    use super::*;

    /// PLAN-063 T-03（G-01/V-02）：widget 剪贴板面 OS 往返——OsClipboard
    /// write→read 闭环（与 direct handler 面同源 arboard；Primary 无对应
    /// = None/忽略同 direct winit 律）。环境探针先行：provider 裸面
    /// set→get 失败 = 本机剪贴板被占（外部环境，2026-11-08 共享机实录）
    /// → 跳过不冒红；provider 通而 trait 面断 = 真缺陷必须红。
    #[test]
    fn p063_os_clipboard_roundtrip() {
        #[cfg(windows)]
        let _global = match crate::ui::clipboard_native::GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return, // 拿锁超时视同 headless 跳过
        };
        // provider 裸面探针（环境门）：失败 = 环境不可验，跳过。
        if !crate::ui::clipboard::clipboard_set("p063-probe")
            || crate::ui::clipboard::clipboard_get().as_deref() != Some("p063-probe")
        {
            eprintln!("[p063] provider 裸面 set→get 失败——本机剪贴板环境不可验，跳过");
            return;
        }
        let mut cb = OsClipboard;
        use iced::advanced::clipboard::Clipboard as _;
        use iced::advanced::clipboard::Kind;
        cb.write(Kind::Standard, "p063-os-clipboard-roundtrip".to_string());
        assert_eq!(
            cb.read(Kind::Standard).as_deref(),
            Some("p063-os-clipboard-roundtrip"),
            "widget 面 write→read OS 往返"
        );
        // Primary：Windows 无选择区——读 None 写忽略（不污染 Standard）。
        cb.write(Kind::Primary, "should-be-ignored".to_string());
        assert_eq!(
            cb.read(Kind::Primary),
            None,
            "Primary 读 None（同 direct winit 律）"
        );
        assert_eq!(
            cb.read(Kind::Standard).as_deref(),
            Some("p063-os-clipboard-roundtrip"),
            "Primary 写不污染 Standard"
        );
    }
}

/// headless 会话表面：组件 + 视口 + iced widget 树缓存。
/// 帧循环（T-02 扩展 revision 门控）：`render_frame` = view() →
/// UserInterface::build（复用 Cache 状态树 diff）→ draw（tiny_skia 记录）
/// → 降格 DrawList。
pub struct HeadlessSurface<C: Component> {
    pub component: C,
    width: f32,
    height: f32,
    cursor: mouse::Cursor,
    cache: Cache,
    /// 上帧观测行（未覆盖原语/降格近似——T-01 逐项消解台账，v2 后剩余项）。
    observations: Vec<String>,
    /// v2 帧待上传位图（Rgba 句柄 → bitmap:// 通道；v1 投影器
    /// drain_bitmap_uploads 同型缝）。
    pending_bitmaps: Vec<super::endpoint::BitmapUpload>,
    /// PLAN-690 T-02：上次下行的 IME 请求（去重基线——iced 每帧
    /// RedrawRequested 重发 `InputMethod::Enabled`，变化才下行）。
    last_ime: Option<ImeReq>,
    /// PLAN-690 T-05：上次下行的光标形状（0 default / 1 pointer / 2 text）。
    last_cursor_kind: u8,
    /// PLAN-690：待下行控制批（ImeRequest/SetCursor——drain_window_controls
    /// 消费，泵在产帧后上行）。
    pending_controls: Vec<ControlMsg>,
    /// PLAN-690 T-03：最近一帧 text_input 上报的组合串（Enabled.preedit
    /// 内容——上行 Preedit 事件 → App iced 态 → 原样上报的环测断言口；
    /// wire ImeRequest v1 不携带组合串本体）。
    preedit_reported: Option<String>,
    /// PLAN-062 T-03：位图/矢量资源身份与驻留缓存（跨帧存活，P-02）。
    resources: ResourceCache,
    /// PLAN-062 T-02：文本以 glyph run 过线（Host 不重排）。默认关 = 字符串
    /// `TextStyled` 兼容面；对端声明 `fidelity/glyphrun/1` 后再打开。
    glyph_runs: bool,
    /// PLAN-062 T-02：阴影以生产端 SDF 贴片过线。默认关——实测贴片经画布图像
    /// 合成后比单层 scrim 更远（见 Spec §2.3 的 A/B），需先解决合成对齐。
    shadow_patches: bool,
}

/// PLAN-062 T-03：位图资源身份与驻留缓存（P-02）。
///
/// 身份 = 内容键（blake3(宽,高,字节)），`local id` = `rq-img-<seq>-<hash8>`：
/// 内容变则 id 变（generation 语义由 id 单调序号承载），内容未变的后续帧
/// **不再上传 payload**，帧只引用同一 `bitmap://` src，Host 侧按 src 命中
/// 已驻留 GPU 纹理。驻留总量有上限（默认 256 MiB/App，按最早键驱逐），
/// 驱逐后的重引用会重传一次并在计数上可解释。
#[derive(Debug)]
pub struct ResourceCache {
    next_seq: u64,
    resident: std::collections::HashMap<([u8; 32], u32, u32), ResidentBitmap>,
    order: std::collections::VecDeque<([u8; 32], u32, u32)>,
    resident_bytes: usize,
    limit_bytes: usize,
    /// 累计上传次数与字节（M-04 资源面计数）。
    pub uploads: u64,
    pub uploaded_bytes: u64,
    /// 命中数（未重复上传的帧引用）。
    pub hits: u64,
    pub evictions: u64,
    /// 阴影贴片缓存：同参数（几何/半径/模糊/偏移/颜色）不重复逐像素求值。
    shadow_patches: std::collections::HashMap<[u8; 32], (WRect, u32, u32, Vec<u8>)>,
    pub shadow_patch_hits: u64,
    /// SVG 设备像素栅格缓存（内容+尺寸不变 → 不重复 resvg 栅格化，P-02）。
    rasters: std::collections::HashMap<([u8; 32], u32, u32), Vec<u8>>,
    pub raster_hits: u64,
    /// per-App 上传预算（PLAN-062 T-04 / 061 冻结 ≤8 MiB·s⁻¹）：滑动窗口内
    /// 的上传字节上限；超限则本帧资源登记回滚并让产帧背压（不丢内容）。
    rate_limit_bytes: u64,
    rate_window: std::time::Duration,
    window_start: Option<std::time::Instant>,
    window_bytes: u64,
    /// 本帧新增的资源键（回滚用）与本帧上传计数。
    frame_keys: Vec<([u8; 32], u32, u32)>,
    frame_uploads: u64,
    frame_bytes: u64,
    over_budget: bool,
    pub budget_deferrals: u64,
    /// PLAN-062（AC-03）：被驱逐（最后 owner 归零）的资源 src 队列——供
    /// `FrameSource::drain_resource_releases` 取出并通知宿主释放。
    released: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResidentBitmap {
    local: String,
    bytes: usize,
}

impl Default for ResourceCache {
    fn default() -> Self {
        Self {
            next_seq: 0,
            resident: std::collections::HashMap::new(),
            order: std::collections::VecDeque::new(),
            resident_bytes: 0,
            limit_bytes: 256 * 1024 * 1024,
            uploads: 0,
            uploaded_bytes: 0,
            hits: 0,
            evictions: 0,
            shadow_patches: std::collections::HashMap::new(),
            shadow_patch_hits: 0,
            rasters: std::collections::HashMap::new(),
            raster_hits: 0,
            rate_limit_bytes: 8 * 1024 * 1024,
            rate_window: std::time::Duration::from_secs(1),
            window_start: None,
            window_bytes: 0,
            frame_keys: Vec::new(),
            frame_uploads: 0,
            frame_bytes: 0,
            over_budget: false,
            budget_deferrals: 0,
            released: Vec::new(),
        }
    }
}

impl ResourceCache {
    pub fn with_limit_bytes(mut self, limit: usize) -> Self {
        self.limit_bytes = limit;
        self.enforce_limit();
        self
    }

    pub fn resident_bytes(&self) -> usize {
        self.resident_bytes
    }

    /// SVG 栅格缓存查询：同内容同尺寸直接复用（不重复跑 resvg）。
    fn raster_lookup(&mut self, key: [u8; 32], w: u32, h: u32) -> Option<Vec<u8>> {
        match self.rasters.get(&(key, w, h)) {
            Some(bytes) => {
                self.raster_hits += 1;
                Some(bytes.clone())
            }
            None => None,
        }
    }

    fn raster_store(&mut self, key: [u8; 32], w: u32, h: u32, bytes: Vec<u8>) {
        self.rasters.insert((key, w, h), bytes);
    }

    /// 按最早键驱逐直到驻留字节回落到上限内（预算收紧也立即生效）。
    fn enforce_limit(&mut self) {
        while self.resident_bytes > self.limit_bytes {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(dropped) = self.resident.remove(&oldest) {
                self.resident_bytes = self.resident_bytes.saturating_sub(dropped.bytes);
                self.evictions += 1;
                // PLAN-062（AC-03）：“最后 owner 归零”→ 记入待释放队列，由泵在
                // 排水时通知宿主（宿主侧按帧引用记账延后真正释放）。
                self.released.push(format!(
                    "bitmap://{}",
                    super::endpoint::bitmap_local_id(&dropped.local)
                ));
            }
        }
    }

    /// 开始一帧：清空本帧登记标记（预算回滚基线）。
    fn begin_frame(&mut self) {
        self.frame_keys.clear();
        self.frame_uploads = 0;
        self.frame_bytes = 0;
        self.over_budget = false;
    }

    /// 提交一帧：上传字节计入滑动窗口。
    fn commit_frame(&mut self) {
        let now = std::time::Instant::now();
        match self.window_start {
            Some(start) if now.duration_since(start) < self.rate_window => {}
            _ => {
                self.window_start = Some(now);
                self.window_bytes = 0;
            }
        }
        self.window_bytes += self.frame_bytes;
        self.frame_keys.clear();
        self.frame_uploads = 0;
        self.frame_bytes = 0;
    }

    /// 回滚一帧的资源登记：本帧新增的键全部撤销、上传计数复原（下一帧重试
    /// 时会重新上传，不会留下半登记的资源）。
    fn rollback_frame(&mut self) {
        for key in self.frame_keys.drain(..) {
            if let Some(dropped) = self.resident.remove(&key) {
                self.resident_bytes = self.resident_bytes.saturating_sub(dropped.bytes);
                self.order.retain(|k| *k != key);
            }
        }
        self.uploads = self.uploads.saturating_sub(self.frame_uploads);
        self.uploaded_bytes = self.uploaded_bytes.saturating_sub(self.frame_bytes);
        self.frame_uploads = 0;
        self.frame_bytes = 0;
        self.budget_deferrals += 1;
    }

    fn over_budget(&self) -> bool {
        self.over_budget
    }

    /// 滑动窗口内是否还有 `cost` 字节的预算。
    fn budget_allows(&mut self, cost: u64) -> bool {
        let now = std::time::Instant::now();
        match self.window_start {
            Some(start) if now.duration_since(start) < self.rate_window => {}
            _ => {
                self.window_start = Some(now);
                self.window_bytes = 0;
            }
        }
        self.window_bytes + self.frame_bytes + cost <= self.rate_limit_bytes
    }

    /// 驻留或上传：命中返回既有 src（零上传），新内容登记并推入本帧上传表。
    /// 返回 `None` = 本帧上传预算不足（帧将背压重试，内容不丢）。
    fn intern(
        &mut self,
        w: u32,
        h: u32,
        rgba: &[u8],
        bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
    ) -> Option<String> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&w.to_le_bytes());
        hasher.update(&h.to_le_bytes());
        hasher.update(rgba);
        let key = (*hasher.finalize().as_bytes(), w, h);
        if let Some(resident) = self.resident.get(&key) {
            self.hits += 1;
            return Some(super::endpoint::bitmap_src(&resident.local));
        }
        if !self.budget_allows(rgba.len() as u64) {
            self.over_budget = true;
            return None;
        }
        self.next_seq += 1;
        let hash8: String = key.0[..4].iter().map(|b| format!("{b:02x}")).collect();
        let local = format!("rq-img-{}-{hash8}", self.next_seq);
        self.uploads += 1;
        self.uploaded_bytes += rgba.len() as u64;
        self.frame_uploads += 1;
        self.frame_bytes += rgba.len() as u64;
        self.frame_keys.push(key);
        bitmaps.push(super::endpoint::BitmapUpload {
            id: super::endpoint::bitmap_local_id(&local),
            w,
            h,
            stride: w * 4,
            rgba: rgba.to_vec(),
        });
        self.resident.insert(
            key,
            ResidentBitmap {
                local: local.clone(),
                bytes: rgba.len(),
            },
        );
        self.order.push_back(key);
        self.resident_bytes += rgba.len();
        self.enforce_limit();
        Some(super::endpoint::bitmap_src(&local))
    }
}

impl<C: Component> HeadlessSurface<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    pub fn new(component: C, width: f32, height: f32) -> Self {
        ensure_headless_fonts();
        Self {
            component,
            width,
            height,
            cursor: mouse::Cursor::Unavailable,
            cache: Cache::default(),
            observations: Vec::new(),
            pending_bitmaps: Vec::new(),
            last_ime: None,
            last_cursor_kind: 0,
            pending_controls: Vec::new(),
            preedit_reported: None,
            resources: ResourceCache::default(),
            glyph_runs: false,
            shadow_patches: false,
        }
    }

    /// Rebuild layout constraints without replacing the component or widget cache.
    fn resize(&mut self, width: f32, height: f32) -> bool {
        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || (self.width, self.height) == (width, height)
        {
            return false;
        }
        self.width = width;
        self.height = height;
        true
    }

    /// 产一帧（v1 wire 降格——legacy/parity 探针面；产线帧走
    /// [`Self::render_frame_v2`]）。
    pub fn render_frame(&mut self) -> DrawList {
        let layers = self.drive_and_draw();
        self.observations.clear();
        lower_layers(&layers, &mut self.observations)
    }

    /// 产一帧 v2（PLAN-683 SD-01 remote 模式产线帧）：tiny_skia 记录层 →
    /// DisplayList v2 原生原语（渐变 stop/四角半径/border/shadow/
    /// transform/位图通道）。
    pub fn render_frame_v2(&mut self) -> DisplayList {
        let layers = self.drive_and_draw();
        self.observations.clear();
        self.pending_bitmaps.clear();
        self.resources.begin_frame();
        let bitmaps = &mut self.pending_bitmaps;
        // 设备像素/逻辑像素比：根层 bounds 已含 viewport scale。
        let scale = layers
            .first()
            .map(|layer| {
                if self.width > 0.0 && layer.bounds.width.is_finite() {
                    (layer.bounds.width / self.width).max(1.0)
                } else {
                    1.0
                }
            })
            .unwrap_or(1.0);
        lower_layers_v2(
            &layers,
            &mut self.observations,
            &mut self.resources,
            bitmaps,
            self.glyph_runs,
            self.shadow_patches,
            scale,
        )
    }

    /// 文本是否以 glyph run 过线（Host 不重排）。默认关：字符串 `TextStyled`
    /// 是兼容面；对端声明 `fidelity/glyphrun/1` 后再打开。
    pub fn set_glyph_runs(&mut self, on: bool) {
        self.glyph_runs = on;
    }

    /// 阴影是否以生产端 SDF 贴片过线（默认关 = 单层 scrim 近似）。
    pub fn set_shadow_patches(&mut self, on: bool) {
        self.shadow_patches = on;
    }

    /// 产帧后核对 per-App 上传预算：超限则回滚本帧资源登记并返回 true，
    /// 调用方以 `Backpressure{resource:"upload"}` 重试（P-04 预算、不丢内容）。
    pub fn take_upload_budget_overrun(&mut self) -> bool {
        if self.resources.over_budget() {
            self.pending_bitmaps.clear();
            self.resources.rollback_frame();
            true
        } else {
            self.resources.commit_frame();
            false
        }
    }

    /// 设置 per-App 上传预算（默认 8 MiB / 1s；探针用小窗口做负控）。
    pub fn set_upload_budget(&mut self, bytes_per_window: u64, window: std::time::Duration) {
        self.resources.rate_limit_bytes = bytes_per_window;
        self.resources.rate_window = window;
    }

    /// 上传预算延期次数（M-04/背压观测）。
    pub fn upload_budget_deferrals(&self) -> u64 {
        self.resources.budget_deferrals
    }

    /// 资源缓存读数（M-04/M-05 资源面：累计上传/命中/驱逐/驻留字节）。
    /// PLAN-062（AC-03）：取出待通知宿主的资源释放（最后 owner 归零的 src）。
    /// 泵在排水点调用并经 `ResourceRelease` 过线（宿主声明 `resource/release/1` 时）。
    pub fn drain_resource_releases(&mut self) -> Vec<String> {
        std::mem::take(&mut self.resources.released)
    }

    pub fn resource_stats(&self) -> (u64, u64, u64, u64, usize) {
        (
            self.resources.uploads,
            self.resources.uploaded_bytes,
            self.resources.hits,
            self.resources.evictions,
            self.resources.resident_bytes(),
        )
    }

    /// SVG 栅格缓存命中数（P-02 资源面观测）。
    pub fn raster_cache_hits(&self) -> u64 {
        self.resources.raster_hits
    }

    /// 调整资源驻留上限（预算/驱逐探针用；生产默认 256 MiB/App）。
    pub fn set_resource_limit_bytes(&mut self, limit: usize) {
        self.resources.limit_bytes = limit;
        self.resources.enforce_limit();
    }

    /// 驱动一轮 iced 帧循环：view → UI::build（Cache 复用）→ RedrawRequested
    /// → draw（tiny_skia 记录）→ 消息回灌组件。返回记录层快照（Clone）。
    fn drive_and_draw(&mut self) -> Vec<Layer> {
        let mut view = self.component.view();
        crate::ui::iced::renderer::rewrite_viewport_units(&mut view);
        let element: Element<'_, C::Msg> = view.into_iced();

        // tiny_skia Secondary 臂：纯 CPU 记录器（iced::Renderer 的合法值）。
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));

        let cache = std::mem::take(&mut self.cache);
        let mut ui = UserInterface::build(
            element,
            iced::Size::new(self.width, self.height),
            cache,
            &mut renderer,
        );

        // RedrawRequested 先行（iced_test snapshot 同款驱动）——部分 widget
        // 首帧状态推进依赖一次 redraw 事件。
        let mut messages = Vec::new();
        // PLAN-063 T-03：widget 剪贴板面接 OS 桥（此前 Null——远程轨
        // Ctrl+C/V 静默失效）。
        let mut clipboard = OsClipboard;
        let (state, _) = ui.update(
            &[iced::Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            self.cursor,
            &mut renderer,
            &mut clipboard,
            &mut messages,
        );
        // PLAN-690 T-01：State::Updated.input_method / mouse_interaction
        // 截获（IME 激活门控 + 光标形状的下行源头）。
        self.capture_ui_state(state);

        // 主题单源（PLAN-679）：与独立轨 run 链同款 shadcn 调色板。
        let theme =
            crate::ui::iced::renderer::shadcn_theme(crate::ui::style_iced_adapter::dark_mode());
        let style = advanced_renderer::Style {
            text_color: theme.base().text_color,
        };
        ui.draw(&mut renderer, &theme, &style, self.cursor);
        self.cache = ui.into_cache();
        for msg in messages {
            self.component.on(msg);
        }

        // Secondary 臂拆取 tiny_skia 记录层（fallback 枚举不转发 layers()）。
        match &mut renderer {
            iced::Renderer::Secondary(tiny) => tiny.layers().to_vec(),
            iced::Renderer::Primary(_) => unreachable!("headless 宿主恒构造 Secondary 臂"),
        }
    }

    /// 远程事件注入（T-02 事件回路径主入口；T-00 首帧 spike 仅渲染）。
    pub fn update(&mut self, events: &[iced::Event]) -> Vec<C::Msg> {
        let mut view = self.component.view();
        crate::ui::iced::renderer::rewrite_viewport_units(&mut view);
        let element: Element<'_, C::Msg> = view.into_iced();

        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));

        let cache = std::mem::take(&mut self.cache);
        let mut ui = UserInterface::build(
            element,
            iced::Size::new(self.width, self.height),
            cache,
            &mut renderer,
        );
        let mut messages = Vec::new();
        if !events.is_empty() {
            // PLAN-063 T-03：事件路径同接 OS 桥剪贴板（与 drive_and_draw
            // 同面——Ctrl+V 粘贴在 KeyPressed 事件轮内读 OS 剪贴板）。
            let mut clipboard = OsClipboard;
            let _ = ui.update(
                events,
                self.cursor,
                &mut renderer,
                &mut clipboard,
                &mut messages,
            );
            // 截获面不在事件路径：iced_winit 只在 RedrawRequested 处理臂
            // 消费 input_method/mouse_interaction（lib.rs:940-970）——事件
            // 轮的 shell 态是瞬态 Disabled（text_input 仅在 Redraw 臂请求
            // IME），照收会误发 Disable 下行。drive_and_draw 独占截获。
        }
        self.cache = ui.into_cache();
        // 事件消息回灌（drive_and_draw 同律——组件状态变更即刻生效，
        // 下一帧 view() 反映；INPUT_TEXT 通道同线程纪律：闭包写入 →
        // on() 读面在同一 update 调用内闭合）。返回值保留原消息集
        //（HeadlessFrameSource 的 revision 门控按非空计前进）。
        for msg in &messages {
            self.component.on(msg.clone());
        }
        messages
    }

    /// 指针位同步（daemon 上行 PointerMoved 后调用——hover 态渲染依赖）。
    pub fn point_at(&mut self, position: Option<iced::Point>) {
        self.cursor = match position {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
    }

    /// PLAN-690 T-01/T-05：`UserInterface::update` 返回态截获——
    /// `State::Updated { input_method, mouse_interaction }` 是 iced 0.14
    /// 公开的 IME 请求/光标形状出口（user_interface.rs:615-640；headless
    /// 旧代码以 `let _ =` 丢弃）。变化才入下行批（iced 每帧重发请求）。
    fn capture_ui_state(&mut self, state: UiState) {
        let UiState::Updated {
            input_method,
            mouse_interaction,
            ..
        } = state
        else {
            return;
        };
        let ime = match input_method {
            InputMethod::Disabled => {
                self.preedit_reported = None;
                None
            }
            InputMethod::Enabled {
                cursor,
                purpose,
                preedit,
            } => {
                // 空组合串归一 None（winit Windows commit 序 = Preedit("")
                // 先行清态——text_input 会存空串，上报面按"无组合态"口径）。
                self.preedit_reported =
                    preedit.filter(|p| !p.content.is_empty()).map(|p| p.content);
                Some(ImeReq {
                    cursor: WRect {
                        x: cursor.x,
                        y: cursor.y,
                        w: cursor.width,
                        h: cursor.height,
                    },
                    purpose: purpose_u8(purpose),
                })
            }
        };
        if ime != self.last_ime {
            self.last_ime = ime.clone();
            // wid = 0 哨兵（连接级寻址——headless 装配期不知真实 wid，
            // rqhost 客户端单 wid 连接按 client 记录路由，壳投影族同型）。
            let wid = 0;
            self.pending_controls
                .push(ControlMsg::ImeRequest { wid, enabled: ime });
        }
        let kind = cursor_kind_u8(mouse_interaction);
        if kind != self.last_cursor_kind {
            self.last_cursor_kind = kind;
            let wid = 0;
            self.pending_controls
                .push(ControlMsg::SetCursor { wid, kind });
        }
    }

    /// PLAN-690：App 派生窗口控制下行批读走（ImeRequest/SetCursor——
    /// 泵在产帧后消费并上行 `ProtocolMsg::Control`）。
    pub fn drain_window_controls(&mut self) -> Vec<ControlMsg> {
        std::mem::take(&mut self.pending_controls)
    }

    /// PLAN-690 T-03：最近一帧 text_input 上报的组合串（环测断言口，
    /// 见 `preedit_reported` 字段注）。
    pub fn preedit_reported(&self) -> Option<&str> {
        self.preedit_reported.as_deref()
    }

    /// 上帧降格观测（未入 wire 的原语——T-01 逐项消解的台账）。
    pub fn observations(&self) -> &[String] {
        &self.observations
    }

    /// v2 帧位图排水（Rgba/Bytes 句柄 → bitmap:// 上传；泵在
    /// `drain_bitmap_uploads` 缝消费——v1 投影器同型）。
    pub fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        std::mem::take(&mut self.pending_bitmaps)
    }
}

// ---------------------------------------------------------------------------
// Layer 记录 → DrawList v1 降格
// ---------------------------------------------------------------------------

/// tiny_skia 记录层 → v1 DrawList。层序 = tiny_skia 渲染序（base 层先画，
/// 后续 clip 层叠上）；层 0 不发 Scissor（全域），其余层以自身 bounds 裁剪
///（与 tiny_skia 光栅期"仅按层自身 bounds 裁剪"同口径）。
fn lower_layers(layers: &[Layer], observations: &mut Vec<String>) -> DrawList {
    let mut ops = Vec::new();
    for (index, layer) in layers.iter().enumerate() {
        let clipped = index > 0;
        if clipped {
            ops.push(DrawOp::Scissor {
                rect: to_wrect(layer.bounds),
            });
        }
        for (quad, background) in &layer.quads {
            lower_quad(quad, background, &mut ops, observations);
        }
        for item in &layer.text {
            for text in item.as_slice() {
                lower_text(text, item.transformation(), &mut ops, observations);
            }
        }
        if !layer.images.is_empty() {
            observations.push(format!(
                "image x{}（v1 词汇面外——T-01 v2 Image 原语承接）",
                layer.images.len()
            ));
        }
        if !layer.primitives.is_empty() {
            observations.push(format!(
                "mesh primitive x{}（canvas/vector 面——T-01 勘定：Path 序列化或位图降级）",
                layer.primitives.len()
            ));
        }
        if clipped {
            ops.push(DrawOp::ScissorPop);
        }
    }
    DrawList {
        clear: Some(super::client_runtime::bg()),
        ops,
    }
}

/// quad 降格：Background::Color → Quad/QuadR（圆角取四角 max——v1 单半径
/// 词汇，四角一致的常态零损）。border 以 inset 单像素描边近似（shadcn
/// 卡面观感主体）；gradient 取停点均值近似；shadow 省略——后三者均为
/// v1 词汇面外，T-01 v2 原生承接（观测行留痕）。
fn lower_quad(
    quad: &advanced_renderer::Quad,
    background: &iced::Background,
    ops: &mut Vec<DrawOp>,
    observations: &mut Vec<String>,
) {
    let rect = to_wrect(quad.bounds);
    let radius = quad
        .border
        .radius
        .top_left
        .max(quad.border.radius.top_right)
        .max(quad.border.radius.bottom_right)
        .max(quad.border.radius.bottom_left);
    match background {
        iced::Background::Color(color) => {
            let rgba = to_rgba8(*color);
            if radius > 0.5 {
                ops.push(DrawOp::QuadR {
                    rect,
                    color: rgba,
                    radius,
                });
            } else {
                ops.push(DrawOp::Quad { rect, color: rgba });
            }
        }
        iced::Background::Gradient(gradient) => {
            observations.push(format!(
                "gradient 降格均值近似（{rect:?}——T-01 v2 原生 stop 记录）"
            ));
            ops.push(DrawOp::QuadR {
                rect,
                color: gradient_average(gradient),
                radius: 0.0,
            });
        }
    }
    if quad.border.width > 0.0 {
        let border = to_rgba8(quad.border.color);
        let w = quad.border.width;
        let WRect { x, y, w: rw, h: rh } = rect;
        ops.push(DrawOp::Quad {
            rect: WRect { x, y, w: rw, h: w },
            color: border,
        });
        ops.push(DrawOp::Quad {
            rect: WRect {
                x,
                y: y + rh - w,
                w: rw,
                h: w,
            },
            color: border,
        });
        ops.push(DrawOp::Quad {
            rect: WRect { x, y, w, h: rh },
            color: border,
        });
        ops.push(DrawOp::Quad {
            rect: WRect {
                x: x + rw - w,
                y,
                w,
                h: rh,
            },
            color: border,
        });
    }
    if quad.shadow.color.a > 0.0
        && (quad.shadow.blur_radius > 0.0
            || quad.shadow.offset.x != 0.0
            || quad.shadow.offset.y != 0.0)
    {
        // v1 无阴影原语——T-01 v2 承接（004 卡面立体感主源）。
        observations.push(format!("shadow 降格省略（{rect:?}——T-01 v2）"));
    }
}

/// 文本记录降格：Paragraph/Editor（cosmic 整形 Buffer）→ 逐 layout run 的
/// TextStyled op（daemon 侧重整形同字体栈）；Cached → 同型直落。
/// 行盒定位 = run 顶（canvas fill_text 顶锚语义同构）；weight 取首字形
///（fontdb 直读）；italic 在 glyph 面缺失——v1 近似省略（观测行）。
fn lower_text(
    text: &GfxText,
    transformation: iced::Transformation,
    ops: &mut Vec<DrawOp>,
    observations: &mut Vec<String>,
) {
    let translation = (transformation * text_local_transformation(text)).translation();
    match text {
        GfxText::Paragraph {
            paragraph,
            position,
            color,
            ..
        } => {
            let Some(paragraph) = paragraph.upgrade() else {
                return;
            };
            buffer_runs(
                paragraph.buffer(),
                *position,
                *color,
                translation,
                &mut |x, y, size, lh, rgba, weight, text| {
                    ops.push(DrawOp::TextStyled {
                        x,
                        y,
                        size,
                        line_height: lh,
                        color: rgba,
                        weight,
                        italic: false,
                        text,
                    });
                },
            );
        }
        GfxText::Editor {
            editor,
            position,
            color,
            ..
        } => {
            let Some(editor) = editor.upgrade() else {
                return;
            };
            buffer_runs(
                editor.buffer(),
                *position,
                *color,
                translation,
                &mut |x, y, size, lh, rgba, weight, text| {
                    ops.push(DrawOp::TextStyled {
                        x,
                        y,
                        size,
                        line_height: lh,
                        color: rgba,
                        weight,
                        italic: false,
                        text,
                    });
                },
            );
        }
        GfxText::Cached {
            content,
            bounds,
            color,
            size,
            line_height,
            font,
            align_x,
            ..
        } => {
            let width = text_width_approx(content, size.0);
            let x = match align_x {
                iced::advanced::text::Alignment::Center => bounds.x - width / 2.0,
                iced::advanced::text::Alignment::Right => bounds.x - width,
                _ => bounds.x,
            };
            ops.push(DrawOp::TextStyled {
                x: x + translation.x,
                y: bounds.y + translation.y,
                size: size.0,
                line_height: line_height.0,
                color: to_rgba8(*color),
                weight: iced_weight_u16(font.weight),
                italic: matches!(
                    font.style,
                    iced::font::Style::Italic | iced::font::Style::Oblique
                ),
                text: content.clone(),
            });
        }
        GfxText::Raw { .. } => {
            observations.push("raw text buffer 降格省略（编辑器组合态——T-02 IME 面）".to_string());
        }
    }
}

/// cosmic Buffer 逐行提取（v1/v2 降格共用——run 顶锚 + 行首字形 x +
/// 首字形字号/字重；emit 回调落各自的 op 词汇）。
fn buffer_runs(
    buffer: &cosmic_text::Buffer,
    position: iced::Point,
    color: iced::Color,
    translation: iced::Vector,
    mut emit: impl FnMut(f32, f32, f32, f32, Rgba8, u16, String),
) {
    let rgba = to_rgba8(color);
    for run in buffer.layout_runs() {
        let Some(first) = run.glyphs.first() else {
            continue;
        };
        let line_h = if run.line_height > 0.0 {
            run.line_height
        } else {
            first.font_size * 1.3
        };
        // cosmic's run.text is the entire logical line, even after soft wrap.
        // Glyph cluster offsets select this visual run's UTF-8 source range.
        let start = run.glyphs.iter().map(|glyph| glyph.start).min().unwrap();
        let end = run.glyphs.iter().map(|glyph| glyph.end).max().unwrap();
        emit(
            position.x + translation.x + first.x,
            position.y + translation.y + run.line_top,
            first.font_size,
            line_h,
            rgba,
            first.font_weight.0,
            run.text[start..end].to_string(),
        );
    }
}

/// iced 字重枚举 → CSS 刻度 u16（v1 TextStyled 词汇）。
fn iced_weight_u16(weight: iced::font::Weight) -> u16 {
    match weight {
        iced::font::Weight::Thin => 100,
        iced::font::Weight::ExtraLight => 200,
        iced::font::Weight::Light => 300,
        iced::font::Weight::Normal => 400,
        iced::font::Weight::Medium => 500,
        iced::font::Weight::Semibold => 600,
        iced::font::Weight::Bold => 700,
        iced::font::Weight::ExtraBold => 800,
        iced::font::Weight::Black => 900,
    }
}

/// 渐变停点均值（v1 近似——T-01 二选一勘定后替换）。
fn gradient_average(gradient: &iced::gradient::Gradient) -> Rgba8 {
    let stops = match gradient {
        iced::gradient::Gradient::Linear(linear) => &linear.stops,
    };
    let mut count = 0usize;
    let (mut r, mut g, mut b, mut a) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for stop in stops.iter().flatten() {
        r += stop.color.r;
        g += stop.color.g;
        b += stop.color.b;
        a += stop.color.a;
        count += 1;
    }
    if count == 0 {
        return Rgba8::new(0, 0, 0, 255);
    }
    let n = count as f32;
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba8::new(to_u8(r / n), to_u8(g / n), to_u8(b / n), to_u8(a / n))
}

/// 宽度近似（Cached 文本对齐定位用——测量启发式，与 v1 投影器
/// measure_text 同刻度：0.6 × size × 字符数）。
fn text_width_approx(content: &str, size: f32) -> f32 {
    content.chars().count() as f32 * size * 0.6
}

fn to_wrect(rect: iced::Rectangle) -> WRect {
    WRect {
        x: rect.x,
        y: rect.y,
        w: rect.width,
        h: rect.height,
    }
}

fn to_rgba8(color: iced::Color) -> Rgba8 {
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba8::new(
        to_u8(color.r),
        to_u8(color.g),
        to_u8(color.b),
        to_u8(color.a),
    )
}

// ---------------------------------------------------------------------------
// 事件回路径（T-02）：wire InputMsg → iced::Event 注入
// ---------------------------------------------------------------------------

/// wire 修饰位（bit0 shift / bit1 ctrl / bit2 alt / bit3 logo——
/// session.rs `wire_mods` 同表反向）→ iced Modifiers。
fn wire_mods_to_iced(bits: u8) -> iced::keyboard::Modifiers {
    use iced::keyboard::Modifiers;
    let mut m = Modifiers::empty();
    if bits & 0b0001 != 0 {
        m |= Modifiers::SHIFT;
    }
    if bits & 0b0010 != 0 {
        m |= Modifiers::CTRL;
    }
    if bits & 0b0100 != 0 {
        m |= Modifiers::ALT;
    }
    if bits & 0b1000 != 0 {
        m |= Modifiers::LOGO;
    }
    m
}

/// Windows VK → iced Named/Code（宿主 LiveInput 语义：Named 键映射；字母
/// 数字走 Character 由 CharTyped 承担——此处只收控制面键）。
fn vk_to_named(vk: u32) -> Option<iced::keyboard::key::Named> {
    use iced::keyboard::key::Named;
    Some(match vk {
        8 => Named::Backspace,
        9 => Named::Tab,
        13 => Named::Enter,
        27 => Named::Escape,
        32 => Named::Space,
        33 => Named::PageUp,
        34 => Named::PageDown,
        35 => Named::End,
        36 => Named::Home,
        37 => Named::ArrowLeft,
        38 => Named::ArrowUp,
        39 => Named::ArrowRight,
        40 => Named::ArrowDown,
        45 => Named::Insert,
        46 => Named::Delete,
        _ => return None,
    })
}

/// Windows VK → iced Physical Code（字母/数字——物理位映射）。
fn vk_to_code(vk: u32) -> Option<iced::keyboard::key::Code> {
    use iced::keyboard::key::Code;
    Some(match vk {
        0x30..=0x39 => Code::Digit0,
        0x41 => Code::KeyA,
        0x42 => Code::KeyB,
        0x43 => Code::KeyC,
        0x44 => Code::KeyD,
        0x45 => Code::KeyE,
        0x46 => Code::KeyF,
        0x47 => Code::KeyG,
        0x48 => Code::KeyH,
        0x49 => Code::KeyI,
        0x4A => Code::KeyJ,
        0x4B => Code::KeyK,
        0x4C => Code::KeyL,
        0x4D => Code::KeyM,
        0x4E => Code::KeyN,
        0x4F => Code::KeyO,
        0x50 => Code::KeyP,
        0x51 => Code::KeyQ,
        0x52 => Code::KeyR,
        0x53 => Code::KeyS,
        0x54 => Code::KeyT,
        0x55 => Code::KeyU,
        0x56 => Code::KeyV,
        0x57 => Code::KeyW,
        0x58 => Code::KeyX,
        0x59 => Code::KeyY,
        0x5A => Code::KeyZ,
        _ => return None,
    })
}

/// wire InputMsg → iced::Event（远程事件回路径——命中/聚焦/IME 全由
/// iced widget 树原生处理）。IME 三态映射：Preedit/Commit →
/// `input_method::Event`（text_input 组合态原生消费）；Cancelled →
/// Closed（组合态清理）。Windows 中文 IME 实机完备性 = T-02 风险面
///（待澄清③），残缺则混合方案兜底登记。
fn vk_to_iced_key(vk: u32) -> Option<iced::keyboard::Key> {
    if let Some(named) = vk_to_named(vk) {
        return Some(iced::keyboard::Key::Named(named));
    }
    // Character input still arrives through CharTyped. Key events retain
    // letter/digit identity for shortcuts such as Ctrl+A, with no text payload.
    if (0x30..=0x39).contains(&vk) || (0x41..=0x5a).contains(&vk) {
        return Some(iced::keyboard::Key::Character(
            ((vk as u8).to_ascii_lowercase() as char).to_string().into(),
        ));
    }
    None
}

pub(crate) fn input_to_iced_events(
    input: &crate::ui::desktop_protocol::message::InputMsg,
) -> Vec<iced::Event> {
    use crate::ui::desktop_protocol::message::InputMsg;
    use iced::keyboard::{Event as KeyEvent, Key};
    use iced::mouse;
    match input {
        // PLAN-060：Action 由 surface.on_input 的 dispatch_named 分支消费。
        InputMsg::Action { .. } => Vec::new(),
        InputMsg::PointerMoved { x, y, .. } => {
            vec![iced::Event::Mouse(mouse::Event::CursorMoved {
                position: iced::Point::new(*x, *y),
            })]
        }
        InputMsg::PointerPressed {
            button,
            x,
            y,
            modifiers,
            ..
        } => {
            vec![
                iced::Event::Mouse(mouse::Event::CursorMoved {
                    position: iced::Point::new(*x, *y),
                }),
                iced::Event::Mouse(mouse::Event::ButtonPressed(wire_button(*button))),
                // 修饰键态同步（iced 内部跟踪依赖事件流——Ctrl+C 等组合
                // 键的正确派发需要按下时修饰在场）。
                iced::Event::Keyboard(KeyEvent::ModifiersChanged(wire_mods_to_iced(*modifiers))),
            ]
        }
        InputMsg::PointerReleased {
            button,
            x,
            y,
            modifiers,
            ..
        } => {
            vec![
                iced::Event::Mouse(mouse::Event::CursorMoved {
                    position: iced::Point::new(*x, *y),
                }),
                iced::Event::Mouse(mouse::Event::ButtonReleased(wire_button(*button))),
                iced::Event::Keyboard(KeyEvent::ModifiersChanged(wire_mods_to_iced(*modifiers))),
            ]
        }
        InputMsg::KeyPressed { key, modifiers, .. } => {
            let mods = wire_mods_to_iced(*modifiers);
            let Some(iced_key) = vk_to_iced_key(*key) else {
                return vec![iced::Event::Keyboard(KeyEvent::ModifiersChanged(mods))];
            };
            vec![iced::Event::Keyboard(KeyEvent::ModifiersChanged(mods)), iced::Event::Keyboard(KeyEvent::KeyPressed {
                key: iced_key.clone(),
                modified_key: iced_key,
                physical_key: vk_to_code(*key)
                    .map(iced::keyboard::key::Physical::Code)
                    .unwrap_or(iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    )),
                location: iced::keyboard::Location::Standard,
                modifiers: mods,
                text: None,
                repeat: false,
            })]
        }
        InputMsg::KeyReleased { key, modifiers, .. } => {
            let mods = wire_mods_to_iced(*modifiers);
            let Some(iced_key) = vk_to_iced_key(*key) else {
                return vec![iced::Event::Keyboard(KeyEvent::ModifiersChanged(mods))];
            };
            vec![iced::Event::Keyboard(KeyEvent::ModifiersChanged(mods)), iced::Event::Keyboard(KeyEvent::KeyReleased {
                key: iced_key.clone(),
                modified_key: iced_key,
                physical_key: vk_to_code(*key)
                    .map(iced::keyboard::key::Physical::Code)
                    .unwrap_or(iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    )),
                location: iced::keyboard::Location::Standard,
                modifiers: mods,
            })]
        }
        InputMsg::CharTyped { ch, .. } => {
            let text: String = ch.to_string();
            let key = Key::Character(iced_widget::core::SmolStr::new(&text));
            vec![iced::Event::Keyboard(KeyEvent::ModifiersChanged(iced::keyboard::Modifiers::empty())), iced::Event::Keyboard(KeyEvent::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::empty(),
                text: Some(iced_widget::core::SmolStr::new(&text)),
                repeat: false,
            })]
        }
        InputMsg::Scroll { dx, dy, .. } => vec![iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Pixels { x: *dx, y: *dy },
        })],
        InputMsg::ImePreedit {
            text, selection, ..
        } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Preedit(
                text.clone(),
                selection.map(|(s, e)| s as usize..e as usize),
            ),
        )],
        InputMsg::ImeCommit { text, .. } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Commit(text.clone()),
        )],
        InputMsg::ImeCancelled { .. } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Closed,
        )],
    }
}

/// iced IME purpose → wire u8（0 Normal / 1 Secure / 2 Terminal——
/// `ControlMsg::ImeRequest` 载荷语义，透传留协议面）。
fn purpose_u8(purpose: Purpose) -> u8 {
    match purpose {
        Purpose::Normal => 0,
        Purpose::Secure => 1,
        Purpose::Terminal => 2,
    }
}

/// iced 鼠标 Interaction → wire 光标形状 u8（PLAN-690 v1 两级：pointer/
/// text 显式，其余 default——iced ~27 变体不逐个过线，SD-02 契约注记）。
fn cursor_kind_u8(interaction: mouse::Interaction) -> u8 {
    match interaction {
        mouse::Interaction::Pointer => 1,
        mouse::Interaction::Text => 2,
        _ => 0,
    }
}

/// wire u8 → iced 光标图标（daemon view mouse_area 消费侧同表反向；
/// Idle 在 iced_winit conversion 映射 CursorIcon::Default——0 档即箭头）。
pub(crate) fn cursor_kind_icon(kind: u8) -> mouse::Interaction {
    match kind {
        1 => mouse::Interaction::Pointer,
        2 => mouse::Interaction::Text,
        _ => mouse::Interaction::Idle,
    }
}

fn wire_button(button: crate::ui::desktop_protocol::message::MouseButton) -> iced::mouse::Button {
    match button {
        crate::ui::desktop_protocol::message::MouseButton::Left => iced::mouse::Button::Left,
        crate::ui::desktop_protocol::message::MouseButton::Right => iced::mouse::Button::Right,
        crate::ui::desktop_protocol::message::MouseButton::Middle => iced::mouse::Button::Middle,
    }
}

// ---------------------------------------------------------------------------
// HeadlessFrameSource（T-02）：FrameSource 接入——ClientPump 直驱
// ---------------------------------------------------------------------------

/// remote 模式帧源（PLAN-683）：HeadlessSurface 的 FrameSource 适配——
/// 事件回路径（InputMsg → iced::Event → widget 树 → 消息 → 组件）+
/// revision 门控（消息产出/tick 到期 → 前进 → 泵侧对账产帧）+ v2 产线帧。
pub struct HeadlessFrameSource<C: Component> {
    surface: HeadlessSurface<C>,
    rev: u64,
    tick_last: Option<std::time::Instant>,
    observations: Vec<String>,
}

impl<C: Component> HeadlessFrameSource<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    pub fn new(component: C, width: f32, height: f32) -> Self {
        Self {
            surface: HeadlessSurface::new(component, width, height),
            rev: 0,
            tick_last: None,
            observations: Vec::new(),
        }
    }

    /// 组件只读访问（调用侧装载/观测）。
    pub fn component(&self) -> &C {
        &self.surface.component
    }

    /// PLAN-055 T-03：组件可变访问（调用侧装载/主题策略应用）。
    pub fn component_mut(&mut self) -> &mut C {
        &mut self.surface.component
    }
}

/// PLAN-060 诊断：帧产出计数（env AUTO_PLAN060_DIAG 同路径）。
#[cfg(debug_assertions)]
pub(crate) fn plan060_frame_diag(rev: u64) {
    let Ok(path) = std::env::var("AUTO_PLAN060_DIAG") else { return };
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "FRAME rev={rev}");
    }
}
/// PLAN-060 诊断：输入到 app 侧的坐标/事件/消息产出落文件（env
/// AUTO_PLAN060_DIAG=路径 时启用；验收排障用，生产缺省零开销一次
/// env 查找）。
#[cfg(debug_assertions)]
pub(crate) fn plan060_diag<T: std::fmt::Debug>(
    input: &crate::ui::desktop_protocol::message::InputMsg,
    messages: &[T],
) {
    let Ok(path) = std::env::var("AUTO_PLAN060_DIAG") else { return };
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{input:?} -> {messages:?}");
    }
}
impl<C: Component> super::endpoint::FrameSource for HeadlessFrameSource<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    fn revision(&self) -> u64 {
        self.rev
    }

    // PLAN-055 T-03（auto-os 055）：主题策略面 = 组件转发——remote/v2 臂
    // 是 a2r exe 桌面/rq 载体的主产线，缺省 false 会把 NativeThemeCarrier
    // 的 caps 门压掉（Hello theme_caps=[] 实证株），宿主永不推送。
    fn theme_capable(&self) -> bool {
        self.surface.component.theme_capable()
    }

    fn on_theme_policy(
        &mut self,
        snap: &crate::ui::theme_policy::PolicySnapshot,
    ) -> Result<(), String> {
        self.surface.component.on_theme_policy(snap)
    }

    fn theme_policy_status(&self) -> Option<(String, String)> {
        self.surface.component.theme_policy_status()
    }

    fn render_frame(&mut self) -> DrawList {
        // v1 面（legacy/parity）：泵 v2 位开着时本方法不为主产线。
        let list = self.surface.render_frame();
        self.observations = self.surface.observations().to_vec();
        list
    }

    fn render_frame_v2(&mut self) -> DisplayList {
        let list = self.surface.render_frame_v2();
        self.observations = self.surface.observations().to_vec();
        plan060_frame_diag(self.rev);
        list
    }

    fn take_upload_budget_overrun(&mut self) -> bool {
        self.surface.take_upload_budget_overrun()
    }

    fn drain_resource_releases(&mut self) -> Vec<String> {
        self.surface.drain_resource_releases()
    }

    fn on_input(&mut self, input: &crate::ui::desktop_protocol::message::InputMsg) {
        // Native remote clients use this generic source, while VM clients may
        // use McpHeadlessSource. Actions have no iced event representation.
        if let super::message::InputMsg::Action { name, arg, .. } = input {
            if self.surface.component.dispatch_named(name, arg.as_deref()) {
                self.rev += 1;
            }
            return;
        }
        // 光标位同步（hover 态渲染依赖——iced Cursor::Available）。
        // PLAN-694：press/release 同步（携带 x,y——任何位置性输入都蕴含
        // 光标在位；桌面会话注入面无 PointerMoved 生产（hover 不上线），
        // 协议级点击（p694 环测）据此可聚焦 iced text_input——物理鼠标
        // 场景 move 流恒先到，行为不变）。
        match input {
            crate::ui::desktop_protocol::message::InputMsg::PointerMoved { x, y, .. }
            | crate::ui::desktop_protocol::message::InputMsg::PointerPressed { x, y, .. }
            | crate::ui::desktop_protocol::message::InputMsg::PointerReleased { x, y, .. } => {
                self.surface.point_at(Some(iced::Point::new(*x, *y)));
            }
            _ => {}
        }
        let events = input_to_iced_events(input);
        if events.is_empty() {
            return;
        }
        let messages = self.surface.update(&events);
        plan060_diag(input, &messages);
        // 消息产出 = 组件状态可能变化 → revision 前进（泵对账产帧）。
        if !messages.is_empty() {
            self.rev += 1;
        }
    }

    fn on_control(&mut self, control: &crate::ui::desktop_protocol::message::ControlMsg) {
        if let ControlMsg::Resize { width, height, .. } = control {
            if self.surface.resize(*width, *height) {
                self.rev += 1;
            }
        }
        // PLAN-063 T-03（G-05）：焦点沿——ControlMsg::Focus → iced 窗
        // 焦点事件（direct winit 同律；客户端 focus/blur widget 态）。
        if let ControlMsg::Focus { focused, .. } = control {
            let ev = iced::window::Event::Focused;
            let ev = if *focused {
                ev
            } else {
                iced::window::Event::Unfocused
            };
            let msgs = self.surface.update(&[iced::Event::Window(ev)]);
            if !msgs.is_empty() {
                self.rev += 1;
                for m in msgs {
                    self.surface.component.on(m);
                }
            }
        }
    }

    fn poll_tick(&mut self) {
        // 双 tick 源（RqProjector 同构）：timesources 到期派发 + 单通道
        // interval tick。
        if self.surface.component.fire_due_timers() {
            self.rev += 1;
        }
        let Some(interval) = self.surface.component.tick_interval_ms() else {
            return;
        };
        let now = std::time::Instant::now();
        match self.tick_last {
            None => self.tick_last = Some(now),
            Some(last) => {
                if now.duration_since(last) >= std::time::Duration::from_millis(u64::from(interval))
                {
                    self.tick_last = Some(now);
                    if let Some(msg) = self.surface.component.tick_msg() {
                        self.surface.component.on(msg);
                        self.rev += 1;
                    }
                }
            }
        }
    }

    fn drain_desktop_commands(&mut self) -> Vec<String> {
        self.surface.component.drain_desktop_commands()
    }

    fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        self.surface.drain_bitmap_uploads()
    }

    fn drain_window_controls(&mut self) -> Vec<crate::ui::desktop_protocol::message::ControlMsg> {
        self.surface.drain_window_controls()
    }
}

// ---------------------------------------------------------------------------
// PLAN-102 D-2 —— rq 客户端 remote 臂 MCP 共位帧源
// ---------------------------------------------------------------------------

/// [`HeadlessFrameSource`] 的 MCP 增强形（`DynamicComponent` 具体面）。
///
/// 背景：rq 客户端臂（`run_vm_rqhost_client`）绕过 `run_session`，MCP
/// server（vtree/pick/state/act 面）不启动——musk canvas 的锚定/结构树/
/// agent 操作全依赖该面。本帧源在 headless 产线上共位 MCP：
///
/// - **发布**（每产线帧）：`view_with_debug_gated` 取 view+id_map+probe →
///   `view_to_vtree_with_paths`（span 伴跨）→ `StyledNodeSnapshot` +
///   `mcp.update(...)`（state/input_map/view_template/key_bindings——
///   run_session 帧同步块同源词汇，renderer.rs §S3a）。computed v1 取
///   probe 快照（raw_class/events/for_context）；bbox 未测界按 vtree 契约
///   省略（"Any field not measured yet is omitted, never an error"）。
///   注意：headless 产线自身仍用 `component.view()`（HeadlessSurface 内
///   部），MCP 发布侧的 debug-gated view 是第二次构建——仅随产线帧发生
///   （revision 前进才产帧），稳态零开销。
/// - **排水**（`poll_tick` 每泵轮）：MCP action 通道 →
///   `DynamicComponent::on_with_input_for`（renderer update 链的最小内联
///   同构——handler 直派 + input_value 按声明实参数解析）。`__mcp_*`
///   合成通道不在本臂派发（坐标输入走协议 InputMsg 真实路径），ack 位
///   以空回执解锁工具侧限时等待。
pub struct McpHeadlessSource {
    surface: HeadlessSurface<crate::ui::dynamic::DynamicComponent>,
    rev: u64,
    tick_last: Option<std::time::Instant>,
    width: f32,
    height: f32,
    mcp: crate::ui::mcp_server::SharedStateHandle,
    rx: std::sync::mpsc::Receiver<crate::ui::mcp_server::ActionMessage>,
}

impl McpHeadlessSource {
    pub fn new(
        component: crate::ui::dynamic::DynamicComponent,
        width: f32,
        height: f32,
        mcp: crate::ui::mcp_server::SharedStateHandle,
        rx: std::sync::mpsc::Receiver<crate::ui::mcp_server::ActionMessage>,
    ) -> Self {
        Self {
            surface: HeadlessSurface::new(component, width, height),
            rev: 0,
            tick_last: None,
            width,
            height,
            mcp,
            rx,
        }
    }

    /// MCP 帧发布（run_session §S3a 同源；见类型文档）。随产线帧调用。
    fn publish_mcp(&mut self) {
        use crate::ui::mcp_server::ComputedNodeLite;
        use std::collections::HashMap;

        let component = &self.surface.component;
        let (view, id_map, probe) = component.view_with_debug_gated(true);
        let span_map = component.span_map().clone();
        let vtree =
            crate::ui::vnode_converter::view_to_vtree_with_paths(view.clone(), |path: &[u16]| {
                let p: Vec<usize> = path.iter().map(|&x| x as usize).collect();
                id_map
                    .get(&p)
                    .and_then(|aura_id| span_map.get(&aura_id))
                    .and_then(|info| info.span)
                    .map(|(offset, len)| crate::ui::debug::SourceSpan { offset, len })
            });
        let mut computed: HashMap<crate::ui::vnode::VNodeId, ComputedNodeLite> = HashMap::new();
        for (path_u16, entry) in probe.snapshot() {
            let vid = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path_u16));
            let node = computed.entry(vid).or_default();
            if entry.raw_class.is_some() {
                node.raw_class = entry.raw_class.clone();
            }
            if !entry.events.is_empty() {
                node.events = entry
                    .events
                    .iter()
                    .map(|e| (e.event.clone(), e.handler.clone()))
                    .collect();
            }
            if let Some(f) = &entry.for_context {
                node.for_context = Some((f.var.clone(), f.index, f.value_repr.clone()));
            }
        }
        let state_vals = component.read_all_state_materialized();
        let input_map = component.input_state_map().clone();
        let view_template = Some(component.view_template().clone());
        let widget_name = component.widget_name().to_string();
        let mut mcp = self.mcp.lock().unwrap();
        mcp.set_styled_vtree(crate::ui::mcp_server::StyledNodeSnapshot {
            widget_name,
            vtree,
            computed,
        });
        mcp.update(
            view,
            id_map,
            state_vals,
            input_map,
            view_template,
            component.key_bindings().clone(),
        );
        mcp.set_window_size(self.width, self.height);
    }
}

impl super::endpoint::FrameSource for McpHeadlessSource {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        // v1 面（legacy/parity）：泵 v2 位开着时本方法不为主产线。
        self.surface.render_frame()
    }

    fn render_frame_v2(&mut self) -> DisplayList {
        let list = self.surface.render_frame_v2();
        self.publish_mcp();
        list
    }

    fn take_upload_budget_overrun(&mut self) -> bool {
        self.surface.take_upload_budget_overrun()
    }

    fn drain_resource_releases(&mut self) -> Vec<String> {
        self.surface.drain_resource_releases()
    }

    fn on_input(&mut self, input: &super::message::InputMsg) {
        // PLAN-060 扩展：组件消息下行。
        if let super::message::InputMsg::Action { name, arg, .. } = input {
            let ok = self.surface.component.dispatch_named(name, arg.as_deref());
            if std::env::var_os("AUTO_PLAN060_DIAG").is_some() {
                eprintln!("[plan060] child(headless) action {name} ok={ok}");
            }
            if ok {
                self.rev += 1;
            }
            return;
        }
        match input {
            super::message::InputMsg::PointerMoved { x, y, .. }
            | super::message::InputMsg::PointerPressed { x, y, .. }
            | super::message::InputMsg::PointerReleased { x, y, .. } => {
                self.surface.point_at(Some(iced::Point::new(*x, *y)));
            }
            _ => {}
        }
        let events = input_to_iced_events(input);
        if events.is_empty() {
            return;
        }
        let messages = self.surface.update(&events);
        plan060_diag(input, &messages);
        if !messages.is_empty() {
            self.rev += 1;
        }
    }

    fn on_control(&mut self, control: &super::message::ControlMsg) {
        if let ControlMsg::Resize { width, height, .. } = control {
            if self.surface.resize(*width, *height) {
                self.width = *width;
                self.height = *height;
                self.rev += 1;
            }
        }
    }

    fn poll_tick(&mut self) {
        // 双 tick 源 + MCP action 排水（HeadlessFrameSource 同构 + D-2 扩展）。
        if self.surface.component.fire_due_timers() {
            self.rev += 1;
        }
        if let Some(interval) = self.surface.component.tick_interval_ms() {
            let now = std::time::Instant::now();
            let due = match self.tick_last {
                None => {
                    self.tick_last = Some(now);
                    false
                }
                Some(last) => {
                    now.duration_since(last)
                        >= std::time::Duration::from_millis(u64::from(interval))
                }
            };
            if due {
                self.tick_last = Some(now);
                if let Some(msg) = self.surface.component.tick_msg() {
                    self.surface.component.on(msg);
                    self.rev += 1;
                }
            }
        }
        let mut acted = false;
        while let Ok(action) = self.rx.try_recv() {
            if let crate::ui::mcp_server::ActionTarget::Event { widget, event } = action.target {
                if event.starts_with("__mcp_") {
                    // 合成坐标通道不在 headless 臂派发（协议 InputMsg 为真实
                    // 路径）；ack 位空回执解锁工具侧 1s 限时等待。
                    if let Some((_, id)) = event.rsplit_once("|ack=") {
                        if let Ok(id) = id.parse::<u64>() {
                            self.mcp.lock().unwrap().finish_fixture(
                                id,
                                crate::ui::mcp_server::FixtureAck::Applied {
                                    changed: vec![],
                                    trigger: None,
                                },
                            );
                        }
                    }
                    continue;
                }
                self.surface
                    .component
                    .on_with_input_for(&widget, &event, action.value);
                acted = true;
            }
        }
        if acted {
            self.rev += 1;
        }
    }

    fn drain_desktop_commands(&mut self) -> Vec<String> {
        self.surface.component.drain_desktop_commands()
    }

    fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        self.surface.drain_bitmap_uploads()
    }

    fn drain_window_controls(&mut self) -> Vec<super::message::ControlMsg> {
        self.surface.drain_window_controls()
    }
}

// ---------------------------------------------------------------------------
// Layer 记录 → DisplayList v2（native 原语——remote 模式产线降格）
// ---------------------------------------------------------------------------

/// v2 降格（PLAN-683 SD-01）：quad 全参原生（渐变 stop/四角半径/border/
/// shadow）；文本 run 承接非恒等变换（Transform push/pop 包络——scale+
/// translate 近似，旋转不可达见观测）；图像句柄 → src 词汇或 bitmap://
/// 上传通道。mesh（canvas/vector）= v2 词汇面外（G2 词汇表未列），位图
/// 降级承接为登记债（试点三例无 canvas 面）。
fn lower_layers_v2(
    layers: &[Layer],
    observations: &mut Vec<String>,
    resources: &mut ResourceCache,
    bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
    glyph_runs: bool,
    shadow_patches: bool,
    scale: f32,
) -> DisplayList {
    let mut ops = Vec::new();
    for (index, layer) in layers.iter().enumerate() {
        let clipped = index > 0;
        if clipped {
            ops.push(DisplayOp::Scissor {
                rect: to_wrect(layer.bounds),
            });
        }
        for (quad, background) in &layer.quads {
            lower_quad_v2(
                quad,
                background,
                &mut ops,
                observations,
                resources,
                bitmaps,
                shadow_patches,
                scale,
            );
        }
        for item in &layer.text {
            lower_text_v2(
                item.as_slice(),
                item.transformation(),
                &mut ops,
                observations,
                glyph_runs,
            );
        }
        for image in &layer.images {
            lower_image_v2(image, &mut ops, observations, resources, bitmaps);
        }
        lower_primitives_v3(layer, &mut ops, observations);
        if clipped {
            ops.push(DisplayOp::ScissorPop);
        }
    }
    DisplayList {
        clear: Some(super::client_runtime::bg()),
        ops,
    }
}

/// v3 几何原语（mesh/canvas/path）：`Primitive::Fill/Stroke` 的 tiny_skia 路径
/// 与 paint 逐段过线（坐标已由 iced geometry 层烘焙变换——061 T-02 实测），
/// 记录端不再丢弃 mesh。组项携带的 clip_bounds 以 Scissor 包络保序。
fn lower_primitives_v3(
    layer: &Layer,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
) {
    for item in &layer.primitives {
        let clip = item.clip_bounds();
        let clip_set = clip.width.is_finite()
            && clip.height.is_finite()
            && clip.width > 0.0
            && clip.height > 0.0;
        if clip_set {
            ops.push(DisplayOp::Scissor {
                rect: to_wrect(clip),
            });
        }
        lower_paths_v3(item.as_slice(), ops, observations);
        if clip_set {
            ops.push(DisplayOp::ScissorPop);
        }
    }
}

/// 单个原语项内的 Fill/Stroke → `DisplayOp::Path`（顺序即记录顺序）。
fn lower_paths_v3(
    primitives: &[iced_tiny_skia::Primitive],
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
) {
    for primitive in primitives {
        match primitive {
            iced_tiny_skia::Primitive::Fill { path, paint, rule } => {
                let Some(fill) = paint_fill(paint) else {
                    observations.push(
                        "几何填充 paint 非纯色（v3 词表外——具名降级，非静默省略）".to_string(),
                    );
                    continue;
                };
                let wire = wire_path(path);
                if wire.is_empty() {
                    continue;
                }
                ops.push(DisplayOp::Path {
                    path: wire,
                    fill: Some(fill),
                    even_odd: *rule == tiny_skia::FillRule::EvenOdd,
                    stroke: None,
                });
            }
            iced_tiny_skia::Primitive::Stroke {
                path,
                paint,
                stroke,
            } => {
                let Some(color) = paint_solid_rgba(paint) else {
                    observations.push(
                        "几何描边 paint 非纯色（v3 词表外——具名降级，非静默省略）".to_string(),
                    );
                    continue;
                };
                let wire = wire_path(path);
                if wire.is_empty() {
                    continue;
                }
                ops.push(DisplayOp::Path {
                    path: wire,
                    fill: None,
                    even_odd: false,
                    stroke: Some(WireStroke {
                        width: stroke.width,
                        color,
                        cap: match stroke.line_cap {
                            tiny_skia::LineCap::Butt => 0,
                            tiny_skia::LineCap::Round => 1,
                            tiny_skia::LineCap::Square => 2,
                        },
                        join: match stroke.line_join {
                            tiny_skia::LineJoin::Miter => 0,
                            tiny_skia::LineJoin::Round => 1,
                            tiny_skia::LineJoin::Bevel => 2,
                            // canvas 无 miter-clip：按 miter 承载并在观测中具名
                            // （不静默改成圆角）。
                            tiny_skia::LineJoin::MiterClip => 0,
                        },
                    }),
                });
            }
        }
    }
}

/// tiny_skia 路径 → 线段序（verb 直落；坐标原样，不做变换）。
fn wire_path(path: &tiny_skia::Path) -> WirePath {
    let segs = path
        .segments()
        .map(|segment| match segment {
            tiny_skia::PathSegment::MoveTo(p) => PathSeg::MoveTo(p.x, p.y),
            tiny_skia::PathSegment::LineTo(p) => PathSeg::LineTo(p.x, p.y),
            tiny_skia::PathSegment::QuadTo(a, b) => PathSeg::QuadTo(a.x, a.y, b.x, b.y),
            tiny_skia::PathSegment::CubicTo(a, b, c) => {
                PathSeg::CubicTo(a.x, a.y, b.x, b.y, c.x, c.y)
            }
            tiny_skia::PathSegment::Close => PathSeg::Close,
        })
        .collect();
    WirePath { segs }
}

/// paint 纯色 → 逻辑 RGBA。iced `geometry::into_paint` 以 `(b, g, r, a)` 构造
/// tiny_skia 色（iced_tiny_skia 0.14.1 实勘）——此处换回逻辑 RGB，否则整帧
/// 红蓝互换（061 T-02 探针首版复现）。
fn paint_solid_rgba(paint: &tiny_skia::Paint<'_>) -> Option<Rgba8> {
    match &paint.shader {
        tiny_skia::Shader::SolidColor(color) => Some(Rgba8 {
            r: (color.blue() * 255.0).round() as u8,
            g: (color.green() * 255.0).round() as u8,
            b: (color.red() * 255.0).round() as u8,
            a: (color.alpha() * 255.0).round() as u8,
        }),
        _ => None,
    }
}

/// paint → v2/v3 `Fill`（纯色；tiny_skia 渐变 shader 无可读回访问器，
/// 明确降级并具名观测——不静默丢图）。
fn paint_fill(paint: &tiny_skia::Paint<'_>) -> Option<Fill> {
    match &paint.shader {
        tiny_skia::Shader::SolidColor(_) => paint_solid_rgba(paint).map(Fill::Color),
        _ => None,
    }
}

/// v2 quad：原生全参——Fill（色/渐变 angle+stops）、四角半径、border、
/// shadow 直落 wire（v1 近似面全部消除）。
fn lower_quad_v2(
    quad: &advanced_renderer::Quad,
    background: &iced::Background,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
    resources: &mut ResourceCache,
    bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
    shadow_patches: bool,
    scale: f32,
) {
    let rect = to_wrect(quad.bounds);
    // 半径是逻辑单位，而 rect/bounds 已是设备像素（记录层已乘 viewport
    // scale）——必须同尺度放大，否则重放的圆角只有一半大小。
    let radius = [
        quad.border.radius.top_left * scale,
        quad.border.radius.top_right * scale,
        quad.border.radius.bottom_right * scale,
        quad.border.radius.bottom_left * scale,
    ];
    let fill = match background {
        iced::Background::Color(color) => Fill::Color(to_rgba8(*color)),
        iced::Background::Gradient(gradient) => match gradient {
            iced::gradient::Gradient::Linear(linear) => Fill::LinearGradient {
                angle: linear.angle.0,
                stops: linear
                    .stops
                    .iter()
                    .flatten()
                    .map(|stop| (stop.offset, to_rgba8(stop.color)))
                    .collect(),
            },
        },
    };
    let border = (quad.border.width > 0.0).then(|| BorderSpec {
        color: to_rgba8(quad.border.color),
        width: quad.border.width,
    });
    // PLAN-062 T-02：shadow blur 不再靠 Host 垫层近似——生产端按 iced_wgpu
    // 阴影 shader 的同一 SDF 公式把阴影预栅格化成贴片资源（`scale` = 设备
    // 像素/逻辑像素），Host 只需合成位图，因此可以逐像素对齐。
    let has_shadow = quad.shadow.color.a > 0.0
        && (quad.shadow.blur_radius > 0.0
            || quad.shadow.offset.x != 0.0
            || quad.shadow.offset.y != 0.0);
    let mut shadow_spec = None;
    if has_shadow && shadow_patches {
        if let Some((patch_rect, w, h, rgba)) = resources.shadow_patch(quad, scale) {
            let src = resources.intern(w, h, &rgba, bitmaps);
            if let Some(src) = src {
                ops.push(DisplayOp::Image {
                    rect: patch_rect,
                    src,
                    fit: crate::ui::desktop_protocol::message::ImageFit::Stretch,
                });
            }
        } else {
            observations.push("shadow 贴片栅格化失败（该 quad 阴影不呈现，具名诊断）".to_string());
        }
    } else if has_shadow {
        // 默认面：Quad 携 ShadowSpec，Host 以单层垫层近似（具名降级）。
        shadow_spec = Some(ShadowSpec {
            color: to_rgba8(quad.shadow.color),
            offset: (quad.shadow.offset.x, quad.shadow.offset.y),
            blur: quad.shadow.blur_radius,
        });
    }
    ops.push(DisplayOp::Quad {
        rect,
        fill,
        radius,
        border,
        shadow: shadow_spec,
    });
}

impl ResourceCache {
    /// 阴影贴片：同参数命中缓存（逐像素 SDF 每帧重算会把 produce 推到 10ms+）。
    fn shadow_patch(
        &mut self,
        quad: &advanced_renderer::Quad,
        scale: f32,
    ) -> Option<(WRect, u32, u32, Vec<u8>)> {
        let b = quad.bounds;
        let mut hasher = blake3::Hasher::new();
        for v in [
            b.x, b.y, b.width, b.height, scale,
            quad.shadow.blur_radius, quad.shadow.offset.x, quad.shadow.offset.y,
            quad.border.radius.top_left, quad.border.radius.top_right,
            quad.border.radius.bottom_right, quad.border.radius.bottom_left,
        ] {
            hasher.update(&v.to_le_bytes());
        }
        let color = to_rgba8(quad.shadow.color);
        hasher.update(&[color.r, color.g, color.b, color.a]);
        let key = *hasher.finalize().as_bytes();
        if let Some(cached) = self.shadow_patches.get(&key) {
            self.shadow_patch_hits += 1;
            return Some(cached.clone());
        }
        let patch = rasterize_shadow(quad, scale)?;
        self.shadow_patches.insert(key, patch.clone());
        Some(patch)
    }
}

/// 阴影贴片：按 iced_wgpu `solid.wgsl` 的 SDF/平滑步进公式逐像素求值。
///
/// 与 shader 同口径：quad 在设备像素空间，`pos`/`scale` 为 quad 顶左与尺寸，
/// 半径取 `min(radius, 短边/2)`；阴影矩形 = quad + offset，扩边 =
/// `|offset| + 2·blur`（shader 的顶点扩边同式）。
fn rasterize_shadow(
    quad: &advanced_renderer::Quad,
    scale: f32,
) -> Option<(WRect, u32, u32, Vec<u8>)> {
    let b = quad.bounds;
    let blur = quad.shadow.blur_radius * scale;
    let ox = quad.shadow.offset.x * scale;
    let oy = quad.shadow.offset.y * scale;
    // 半径与几何同为设备像素后再 clamp（shader 的 clamp 也是设备尺度）。
    let half = b.width.min(b.height) / 2.0;
    let r = [
        (quad.border.radius.top_left * scale).min(half),
        (quad.border.radius.top_right * scale).min(half),
        (quad.border.radius.bottom_right * scale).min(half),
        (quad.border.radius.bottom_left * scale).min(half),
    ];
    // 贴片覆盖 shader 里被扩边的几何（否则模糊外沿会被裁掉）。
    let x0 = (b.x + ox.min(0.0) - blur).floor();
    let y0 = (b.y + oy.min(0.0) - blur).floor();
    let x1 = (b.x + ox.max(0.0) + b.width + blur).ceil();
    let y1 = (b.y + oy.max(0.0) + b.height + blur).ceil();
    let w = (x1 - x0).max(1.0) as u32;
    let h = (y1 - y0).max(1.0) as u32;
    if w > 4096 || h > 4096 {
        return None;
    }
    let color = to_rgba8(quad.shadow.color);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for py in 0..h {
        let fy = y0 + py as f32 + 0.5;
        for px in 0..w {
            let fx = x0 + px as f32 + 0.5;
            // shader: rounded_box_sdf(-(frag - pos - offset - size/2) * 2, size, radius*2)/2
            let sdf = rounded_box_sdf(
                -((fx - b.x - ox - b.width / 2.0) * 2.0),
                -((fy - b.y - oy - b.height / 2.0) * 2.0),
                b.width,
                b.height,
                // shader 传 `border_radius * 2.0`（与 p 的 2 倍坐标同尺度）。
                [r[0] * 2.0, r[1] * 2.0, r[2] * 2.0, r[3] * 2.0],
            ) / 2.0;
            let alpha = 1.0 - smoothstep(-blur, blur, sdf.max(0.0));
            // PLAN-062（AC-02 / 复审 F-01）：**只保留环带**。重放画布的图像通道
            // 在几何之后合成（iced `CanvasFrame::draw_image` 是延后批次），
            // DisplayList 里“图在 quad 之前”的顺序不保证视觉在下——整块贴片会
            // 把 quad 内部再压暗一层（实测 v3 富模式 2.56%）。把 quad 自身圆角
            // 形状内部的 alpha 清零后，贴片只画外环：顺序无关、视觉等价。
            let quad_sdf = rounded_box_sdf(
                -((fx - b.x - b.width / 2.0) * 2.0),
                -((fy - b.y - b.height / 2.0) * 2.0),
                b.width,
                b.height,
                [r[0] * 2.0, r[1] * 2.0, r[2] * 2.0, r[3] * 2.0],
            ) / 2.0;
            let alpha = if quad_sdf < -0.5 { 0.0 } else { alpha };
            let a = (alpha.clamp(0.0, 1.0) * color.a as f32).round() as u8;
            // 阴影为纯色（shader 的 premultiply 在合成时做）：这里按 straight
            // 输出，与其它位图资源同口径。
            rgba.extend_from_slice(&[color.r, color.g, color.b, a]);
        }
    }
    Some((
        WRect::new(x0, y0, w as f32, h as f32),
        w,
        h,
        rgba,
    ))
}

/// iced_wgpu `rounded_box_sdf`（shader 同式；`p` 已是相对中心的 2 倍坐标）。
fn rounded_box_sdf(px: f32, py: f32, w: f32, h: f32, radius: [f32; 4]) -> f32 {
    let box_half = if px > 0.0 {
        [radius[2], radius[3]]
    } else {
        [radius[0], radius[1]]
    };
    let corner = if py > 0.0 { box_half[1] } else { box_half[0] };
    let qx = px.abs() - w + corner;
    let qy = py.abs() - h + corner;
    qx.max(qy).min(0.0) + (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() - corner
}

/// GLSL/WGSL `smoothstep`。
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < f32::EPSILON {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 字符串承载（兼容面）：与 `lower_glyph_run` 同一整形结果按行拆 run 过线，
/// Host 用同一字体栈重排（具名降级，V-02 可观测）。
fn lower_text_runs(
    buffer: &cosmic_text::Buffer,
    position: iced::Point,
    color: iced::Color,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
) {
    // PLAN-062 r4 AC-02：字符串面 = Host 用同一字体栈重排，属**具名降级**；
    // 每次产出必须留可观测记录（不得静默）。
    let mut runs = 0usize;
    buffer_runs(
        buffer,
        position,
        color,
        iced::Vector::default(),
        &mut |x, y, size, lh, rgba, weight, text| {
            runs += 1;
            ops.push(DisplayOp::TextStyled {
                x,
                y,
                size,
                line_height: lh,
                color: rgba,
                weight,
                italic: false,
                text,
            });
        },
    );
    if runs > 0 {
        observations.push(format!(
            "文字降级：{runs} 个 run 以字符串 TextStyled 过线（Host 用同一字体栈重排，非 glyph run）"
        ));
    }
}

/// PLAN-062 T-02：cosmic Buffer → 逐行 `GlyphRun`（Host 不重排）。
///
/// 坐标与 tiny-skia 绘制同口径：`physical()` 的入参 =
/// `(glyph.x + glyph.x_offset·font_size) · scale + position·transform`；
/// 行基线取整偏移单独随 run 过线。字体按**内容身份**过线；字体未装载/无
/// 内容身份时整行退回字符串 `TextStyled` 并留具名观测（不静默换字体）。
fn lower_glyph_run(
    buffer: &cosmic_text::Buffer,
    position: iced::Point,
    color: iced::Color,
    transformation: iced::Transformation,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
) {
    let rgba = to_rgba8(color);
    let scale = transformation.scale_factor();
    let pos = position * transformation;
    let system = font_system();
    let mut font_system = system.write().expect("Write font system");

    let mut unshipped: Vec<u16> = Vec::new();
    for run in buffer.layout_runs() {
        let line_offset = (run.line_y * scale).round();
        let mut glyphs = Vec::new();
        let mut font_hash: Option<[u8; 16]> = None;
        let mut font_size = 0.0f32;
        let mut weight = 400u16;
        let mut flags = 0u32;
        for glyph in run.glyphs {
            let hash = super::glyph_fonts::hash_for_font_id(
                font_system.raw(),
                glyph.font_id,
                glyph.font_weight,
            );
            let Some(hash) = hash else {
                unshipped.push(glyph.glyph_id);
                continue;
            };
            if font_hash.is_none() {
                font_hash = Some(hash);
                font_size = glyph.font_size * scale;
                weight = glyph.font_weight.0;
                flags = glyph.cache_key_flags.bits();
            } else if font_hash != Some(hash) {
                // 同一行混排不同字面（例如中文回退）：按字形边界拆 run。
                let split = DisplayOp::GlyphRun {
                    font_hash: font_hash.expect("上方已设"),
                    font_size,
                    weight,
                    flags,
                    color: rgba,
                    line_offset,
                    glyphs: std::mem::take(&mut glyphs),
                };
                ops.push(split);
                font_hash = Some(hash);
                font_size = glyph.font_size * scale;
                weight = glyph.font_weight.0;
                flags = glyph.cache_key_flags.bits();
            }
            let x = (glyph.x + glyph.font_size * glyph.x_offset).mul_add(scale, pos.x);
            let y = (glyph.y - glyph.font_size * glyph.y_offset).mul_add(scale, pos.y);
            glyphs.push(crate::ui::desktop_protocol::message::GlyphInstance {
                glyph_id: glyph.glyph_id,
                x,
                y,
            });
        }
        if let Some(font_hash) = font_hash {
            if !glyphs.is_empty() {
                ops.push(DisplayOp::GlyphRun {
                    font_hash,
                    font_size,
                    weight,
                    flags,
                    color: rgba,
                    line_offset,
                    glyphs,
                });
            }
        }
        if !unshipped.is_empty() {
            observations.push(format!(
                "字体内容身份不可得（glyph {}…）——该行退回字符串承载（Host 重排，具名降级）",
                unshipped[0]
            ));
            unshipped.clear();
        }
        // 字符串回退：仅在没有任何 glyph 被承载时补一条 TextStyled，保证
        // 文字永不静默消失（V-02 负控口径）。
        if ops
            .last()
            .map(|op| !matches!(op, DisplayOp::GlyphRun { .. }))
            .unwrap_or(true)
        {
            if let Some(first) = run.glyphs.first() {
                let line_h = if run.line_height > 0.0 {
                    run.line_height
                } else {
                    first.font_size * 1.3
                };
                let start = run.glyphs.iter().map(|g| g.start).min().unwrap_or(0);
                let end = run.glyphs.iter().map(|g| g.end).max().unwrap_or(0);
                ops.push(DisplayOp::TextStyled {
                    x: pos.x + first.x,
                    y: pos.y + run.line_top,
                    size: first.font_size * scale,
                    line_height: line_h * scale,
                    color: rgba,
                    weight: first.font_weight.0,
                    italic: false,
                    text: run.text[start..end].to_string(),
                });
            }
        }
    }
}

/// v2 文本：项级非恒等变换 → Transform 包络（scale+translate 近似——
/// iced Transformation 未公开矩阵元，旋转/剪切不可达，观测留痕）；run
/// 几何在变换局部坐标直落。
fn text_local_transformation(text: &GfxText) -> iced::Transformation {
    match text {
        GfxText::Paragraph { transformation, .. }
        | GfxText::Editor { transformation, .. }
        | GfxText::Raw { transformation, .. } => *transformation,
        GfxText::Cached { .. } => iced::Transformation::IDENTITY,
    }
}

fn lower_text_v2(
    texts: &[GfxText],
    transformation: iced::Transformation,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
    glyph_runs: bool,
) {
    for text in texts {
        // Live paragraphs/editors carry their own transform; Item::Live reports
        // identity. Cached groups additionally carry a parent transform. Compose
        // both once, otherwise scrolling culls correctly but emits unscrolled text.
        let transformation = transformation * text_local_transformation(text);
        let wrapped = transformation != iced::Transformation::IDENTITY;
        if wrapped {
            let translation = transformation.translation();
            let scale = transformation.scale_factor();
            ops.push(DisplayOp::Transform {
                matrix: [scale, 0.0, 0.0, scale, translation.x, translation.y],
            });
            if (scale - 1.0).abs() > f32::EPSILON {
                observations.push(
                    "transform scale 近似（旋转/剪切矩阵元 iced 未公开——P683-D2）".to_string(),
                );
            }
        }
        match text {
            GfxText::Paragraph {
                paragraph,
                position,
                color,
                ..
            } => {
                if let Some(paragraph) = paragraph.upgrade() {
                    if glyph_runs {
                        lower_glyph_run(
                        paragraph.buffer(),
                        *position,
                        *color,
                            transformation,
                            ops,
                            observations,
                        );
                    } else {
                        lower_text_runs(paragraph.buffer(), *position, *color, ops, observations);
                    }
                }
            }
            GfxText::Editor {
                editor,
                position,
                color,
                ..
            } => {
                if let Some(editor) = editor.upgrade() {
                    if glyph_runs {
                        lower_glyph_run(
                        editor.buffer(),
                        *position,
                        *color,
                            transformation,
                            ops,
                            observations,
                        );
                    } else {
                        lower_text_runs(editor.buffer(), *position, *color, ops, observations);
                    }
                }
            }
            GfxText::Cached {
                content,
                bounds,
                color,
                size,
                line_height,
                font,
                align_x,
                ..
            } => {
                let width = text_width_approx(content, size.0);
                let x = match align_x {
                    iced::advanced::text::Alignment::Center => bounds.x - width / 2.0,
                    iced::advanced::text::Alignment::Right => bounds.x - width,
                    _ => bounds.x,
                };
                ops.push(DisplayOp::TextStyled {
                    x,
                    y: bounds.y,
                    size: size.0,
                    line_height: line_height.0,
                    color: to_rgba8(*color),
                    weight: iced_weight_u16(font.weight),
                    italic: matches!(
                        font.style,
                        iced::font::Style::Italic | iced::font::Style::Oblique
                    ),
                    text: content.clone(),
                });
            }
            GfxText::Raw { raw, .. } => {
                // PLAN-062 T-02：raw（编辑器组合态）不再静默省略——与
                // paragraph/editor 同源整形结果过线；weak 已失效或无行时具名
                // 诊断（Host 侧不猜内容）。
                if let Some(buffer) = raw.buffer.upgrade() {
                    let before = ops.len();
                    buffer_runs(
                        &buffer,
                        raw.position,
                        raw.color,
                        iced::Vector::default(),
                        &mut |x, y, size, lh, rgba, weight, text| {
                            ops.push(DisplayOp::TextStyled {
                                x,
                                y,
                                size,
                                line_height: lh,
                                color: rgba,
                                weight,
                                italic: false,
                                text,
                            });
                        },
                    );
                    if ops.len() == before {
                        observations.push(
                            "raw text 无整形行（buffer_lines=0——录制期须强制整形）".to_string(),
                        );
                    }
                } else {
                    observations
                        .push("raw text buffer 已释放（无内容可承载——具名诊断）".to_string());
                }
            }
        }
        if wrapped {
            ops.push(DisplayOp::TransformPop);
        }
    }
}

/// v2/v3 图像：句柄 → src 词汇（Path 直通）/ `bitmap://` 资源（Rgba 直取、
/// Bytes 解码、SVG 生产端设备像素栅格化）。
///
/// PLAN-062：呈现参数不再是“观测省略”——
/// - `opacity` 在生产端烘焙进字节（straight alpha 乘算，逐字节确定）；
/// - `rotation` 以 `Transform` 包络过线（绕 rect 中心），重放端按仿射
///   分解（translate/rotate/scale）执行，不再退化为仅平移；
/// - 位图按内容身份驻留（[`ResourceCache`]）：内容未变的帧零重传。
fn lower_image_v2(
    image: &iced::advanced::graphics::image::Image,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
    resources: &mut ResourceCache,
    bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
) {
    use iced::advanced::graphics::image::Image as GfxImage;

    let (bounds, rotation, pixels) = match image {
        GfxImage::Raster { image, bounds, .. } => {
            let Some((w, h, mut rgba)) = load_image_handle(&image.handle, observations) else {
                return;
            };
            if (image.opacity - 1.0).abs() > f32::EPSILON {
                apply_opacity(&mut rgba, image.opacity);
            }
            (*bounds, image.rotation.0, (w, h, rgba))
        }
        GfxImage::Vector { svg, bounds, .. } => {
            match rasterize_svg(svg, *bounds, observations, resources) {
                Some((w, h, rgba)) => (*bounds, 0.0, (w, h, rgba)),
                None => return,
            }
        }
    };

    let (w, h, rgba) = pixels;
    // 上传预算不足 → 该帧不产内容（调用方背压重试，资源不半登记）。
    let Some(src) = resources.intern(w, h, &rgba, bitmaps) else {
        return;
    };
    let rect = to_wrect(bounds);
    let rotated = rotation.abs() > f32::EPSILON;
    if rotated {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let (sin, cos) = rotation.sin_cos();
        ops.push(DisplayOp::Transform {
            matrix: [
                cos,
                -sin,
                sin,
                cos,
                cx - cos * cx + sin * cy,
                cy - sin * cx - cos * cy,
            ],
        });
    }
    ops.push(DisplayOp::Image {
        rect,
        src,
        fit: crate::ui::desktop_protocol::message::ImageFit::Stretch,
    });
    if rotated {
        ops.push(DisplayOp::TransformPop);
    }
}

/// 图像句柄 → RGBA（Path 读文件、Rgba 直取、Bytes 解码）。失败具名观测。
fn load_image_handle(
    handle: &iced::advanced::image::Handle,
    observations: &mut Vec<String>,
) -> Option<(u32, u32, Vec<u8>)> {
    use iced::advanced::image::Handle;
    match handle {
        Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => Some((*width, *height, pixels.to_vec())),
        Handle::Bytes(_, bytes) => match decode_image_rgba(&bytes[..]) {
            Some(decoded) => Some(decoded),
            None => {
                observations.push("Bytes 图像解码失败（该资源不呈现——具名诊断）".to_string());
                None
            }
        },
        Handle::Path(_, path) => match std::fs::read(path) {
            Ok(bytes) => match decode_image_rgba(&bytes) {
                Some(decoded) => Some(decoded),
                None => {
                    observations.push(format!(
                        "路径图像解码失败：{}（该资源不呈现）",
                        path.display()
                    ));
                    None
                }
            },
            Err(err) => {
                observations.push(format!(
                    "路径图像不可读：{}（{err}）",
                    path.display()
                ));
                None
            }
        },
    }
}

/// straight alpha 的 opacity 乘算（镜像 `Color::a` 语义；预乘输入先还原）。
fn apply_opacity(rgba: &mut [u8], opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    for px in rgba.chunks_exact_mut(4) {
        px[3] = (px[3] as f32 * opacity).round() as u8;
    }
}

/// SVG → 生产端设备像素栅格化（resvg 与 iced 内部同源）。
///
/// 目标尺寸 = 记录到的 `bounds`（已含 viewport scale 的设备像素），即 1:1
/// 像素映射；`Svg::color` 以 alpha 掩膜着色（与 iced 的 color filter 同义）。
fn rasterize_svg(
    svg: &iced::advanced::svg::Svg,
    bounds: iced::Rectangle,
    observations: &mut Vec<String>,
    resources: &mut ResourceCache,
) -> Option<(u32, u32, Vec<u8>)> {
    use iced::advanced::svg::Data;

    let bytes = match svg.handle.data() {
        Data::Bytes(bytes) => bytes.to_vec(),
        Data::Path(path) => match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                observations.push(format!("SVG 不可读：{}（{err}）", path.display()));
                return None;
            }
        },
    };
    let w = bounds.width.round().max(1.0) as u32;
    let h = bounds.height.round().max(1.0) as u32;
    // 内容身份：同 SVG 字节 + 同目标尺寸 → 直接复用上次栅格（内容未变的
    // 重复帧不再跑 resvg；T-05 实测未缓存时 20–33ms/帧）。
    let mut hasher = blake3::Hasher::new();
    hasher.update(&bytes);
    let tint = svg.color.map(to_rgba8);
    hasher.update(&[tint.map(|c| c.r).unwrap_or(0), tint.map(|c| c.g).unwrap_or(0),
        tint.map(|c| c.b).unwrap_or(0), tint.map(|c| c.a).unwrap_or(0)]);
    hasher.update(&w.to_le_bytes());
    hasher.update(&h.to_le_bytes());
    let key = *hasher.finalize().as_bytes();
    if let Some(cached) = resources.raster_lookup(key, w, h) {
        return Some((w, h, cached));
    }
    let tree = match resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default()) {
        Ok(tree) => tree,
        Err(err) => {
            observations.push(format!("SVG 解析失败（{err}）"));
            return None;
        }
    };
    const MAX_RASTER_EDGE: u32 = 4096;
    if w > MAX_RASTER_EDGE || h > MAX_RASTER_EDGE {
        observations.push(format!(
            "SVG 栅格尺寸 {w}×{h} 超单边上限 {MAX_RASTER_EDGE}（资源上限具名拒绝）"
        ));
        return None;
    }
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    let size = tree.size();
    let sx = w as f32 / size.width();
    let sy = h as f32 / size.height();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );
    let mut rgba = pixmap.data().to_vec();
    // tiny_skia 输出预乘 → 转 straight（wire/iced 均按 straight RGBA 解释）。
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a != 0 && a != 255 {
            for channel in &mut px[..3] {
                *channel = ((*channel as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            }
        }
    }
    if let Some(color) = svg.color {
        let tint = to_rgba8(color);
        for px in rgba.chunks_exact_mut(4) {
            let coverage = px[3] as f32 / 255.0;
            px[0] = (tint.r as f32 * coverage).round() as u8;
            px[1] = (tint.g as f32 * coverage).round() as u8;
            px[2] = (tint.b as f32 * coverage).round() as u8;
            px[3] = (px[3] as f32 * (tint.a as f32 / 255.0)).round() as u8;
        }
    }
    resources.raster_store(key, w, h, rgba.clone());
    Some((w, h, rgba))
}

/// 编码图像字节 → RGBA（Bytes 句柄解码——repo image 管线同源）。
fn decode_image_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Some((w, h, rgba.into_raw()))
}

// PLAN-061 T-01: direct 基线 + RQ 产帧计时/计数探针（test-only）。
#[cfg(test)]
mod rq_probe;
#[cfg(test)]
mod rq_seams;
#[cfg(test)]
mod rq_matrix;
#[cfg(test)]
mod rq_repair;
// PLAN-062 T-02: v3 无损原语线合同探针（test-only）。
#[cfg(test)]
mod rq_lossless;
// PLAN-062 T-05: 六格绘图/资源重放矩阵与指标重放（test-only）。
#[cfg(test)]
mod rq_cells;
// PLAN-062 T-05 补：真实 wire（管道/端点/shm）+ 两 Host 合成面重放（test-only）。
#[cfg(test)]
mod rq_wire;

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p060_wrapped_text_emits_only_each_visual_run() {
        let mut fonts = cosmic_text::FontSystem::new();
        for content in ["ABCDEFGHIJKLMNO", "中文路径客户数据库", "first\nsecond"] {
            let mut buffer = cosmic_text::Buffer::new(
                &mut fonts, cosmic_text::Metrics::new(16.0, 20.0),
            );
            buffer.set_size(&mut fonts, Some(45.0), Some(500.0));
            buffer.set_text(&mut fonts, content, &cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced, None);
            buffer.shape_until_scroll(&mut fonts, false);
            let mut runs = Vec::new();
            buffer_runs(&buffer, iced::Point::ORIGIN, iced::Color::BLACK, iced::Vector::ZERO,
                |_, y, _, _, _, _, text| runs.push((y, text)));
            assert!(runs.len() > 1, "fixture must wrap: {content:?}");
            assert!(runs.windows(2).all(|pair| pair[0].0 < pair[1].0));
            assert_eq!(runs.iter().map(|(_, text)| text.as_str()).collect::<String>(), content.replace('\n', ""),
                "visual runs must partition source rather than repeat the entire logical line");
        }
    }

    #[test]
    fn p060_native_scroll_reaches_bottom_and_replays_coordinates() {
        assert_native_scroll_reaches_bottom(false);
    }

    #[test]
    fn p060_native_style_overflow_reaches_bottom_and_replays_coordinates() {
        assert_native_scroll_reaches_bottom(true);
    }

    #[test]
    fn p060_native_scroll_retains_styled_bubble_paint() {
        #[derive(Debug)] struct Paint(bool);
        impl Component for Paint {
            type Msg = ();
            fn on(&mut self, _: ()) {}
            fn view(&self) -> crate::ui::View<()> {
                use crate::ui::View;
                let content = View::col()
                    .child(View::text_styled("BUBBLE", "bg-red-500 text-black rounded-xl px-4 py-2"))
                    .build();
                if self.0 { View::scrollable(content).style("h-full w-full").build() }
                else { content }
            }
        }
        for scrolling in [false, true] {
            let mut surface = HeadlessSurface::new(Paint(scrolling), 240.0, 160.0);
            let list = surface.render_frame_v2();
            let red = list.ops.iter().filter(|op| matches!(op,
                DisplayOp::Quad { fill: Fill::Color(c), .. } if c.r > 200 && c.g < 100 && c.b < 100
            )).count();
            println!("scrolling={scrolling} red_quads={red}");
            assert!(red > 0, "styled bubble background lost: scrolling={scrolling}; {:?}", list.ops);
        }
    }

    fn assert_native_scroll_reaches_bottom(styled: bool) {
        use crate::ui::View;
        #[derive(Debug)]
        struct ScrollPage(usize, bool);
        impl Component for ScrollPage {
            type Msg = ();
            fn on(&mut self, _: ()) { self.0 += 1; }
            fn view(&self) -> View<()> {
                let content = View::col()
                    .style(if self.1 { "h-full w-full overflow-y-auto" } else { "" })
                    .children((0..40).map(|index| View::text(format!("ROW{index}"))).collect::<Vec<_>>())
                    .child(View::button("BOTTOM").on_click(|_| ()).build())
                    .build();
                if self.1 { content.with_overflow_scroll() }
                else { View::scrollable(content).style("h-full w-full").build() }
            }
        }
        fn bottom(list: &DisplayList) -> Option<(f32, f32)> {
            let mut stack = vec![(0.0, 0.0)];
            for op in &list.ops {
                match op {
                    DisplayOp::Transform { matrix } => {
                        let (x, y) = *stack.last().unwrap();
                        stack.push((x + matrix[4], y + matrix[5]));
                    }
                    DisplayOp::TransformPop => {
                        stack.pop();
                    }
                    DisplayOp::TextStyled { x, y, text, .. } if text == "BOTTOM" => {
                        let (tx, ty) = *stack.last().unwrap();
                        return Some((x + tx, y + ty));
                    }
                    _ => {}
                }
            }
            None
        }
        // The same production endpoint must replace Hello's requested extent
        // with the actual WM client extent, preserving the component instance.
        use super::super::endpoint::{AppEndpoint, FrameSource, HostEndpoint};
        use super::super::message::{FrameMode, FrameMsg, ProtocolMsg, WRect};
        let source = HeadlessFrameSource::new(ScrollPage(7, styled), 1280.0, 800.0);
        let mut app = AppEndpoint::new(source, "scroll", "scroll", 1280.0, 800.0);
        let mut host = HostEndpoint::listen();
        host.on_message(app.connect().unwrap()).unwrap();
        app.on_message(
            host.activate(
                1,
                3,
                42,
                WRect::new(0.0, 0.0, 242.0, 197.0),
                FrameMode::Commands,
            )
            .unwrap(),
        )
        .unwrap();
        let before = app.session.revision();
        app.on_message(ProtocolMsg::Frame(FrameMsg::Resize {
            surface: 42,
            width: 240.0,
            height: 160.0,
        }))
        .unwrap();
        assert!(
            app.session.revision() > before,
            "resizing must invalidate the frame"
        );
        let resized = app.session.revision();
        for (surface, width, height) in [
            (42, 240.0, 160.0),
            (99, 20.0, 20.0),
            (42, 0.0, 0.0),
            (42, f32::NAN, 50.0),
        ] {
            app.on_message(ProtocolMsg::Frame(FrameMsg::Resize {
                surface,
                width,
                height,
            }))
            .unwrap();
        }
        assert_eq!(
            app.session.revision(),
            resized,
            "same, foreign and invalid extents are no-ops"
        );
        let surface = &mut app.session.surface;
        assert_eq!((surface.width, surface.height), (240.0, 160.0));
        assert_eq!(
            surface.component.0, 7,
            "resize must retain the component state"
        );
        surface.render_frame_v2();
        surface.point_at(Some(iced::Point::new(100.0, 80.0)));
        surface.update(&[iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Pixels {
                x: 0.0,
                y: -10000.0,
            },
        })]);
        let scrolled = surface.render_frame_v2();
        let (x, y) = bottom(&scrolled).expect("bottom text in scrolled frame");
        assert!(
            (0.0..160.0).contains(&y),
            "native bottom outside viewport: {x},{y}; {:?}",
            scrolled.ops
        );
        let legacy = surface.render_frame();
        let legacy_bottom = legacy
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::TextStyled { x, y, text, .. } if text == "BOTTOM" => Some((*x, *y)),
                _ => None,
            })
            .expect("legacy scrolled text");
        assert!((legacy_bottom.0 - x).abs() < 0.1 && (legacy_bottom.1 - y).abs() < 0.1);

        // Replay the actual wire list through the same canvas painter used by
        // the host, then record it again to catch coordinate loss at that seam.
        use iced::advanced::graphics::geometry::Renderer as _;
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));
        let mut frame = iced::widget::canvas::Frame::new(&renderer, iced::Size::new(240.0, 160.0));
        crate::ui::iced::broker_surface::paint_ops_v2(&mut frame, &scrolled.ops);
        iced::advanced::Renderer::start_transformation(
            &mut renderer,
            iced::Transformation::translate(5.0, 7.0),
        );
        renderer.draw_geometry(frame.into_geometry());
        iced::advanced::Renderer::end_transformation(&mut renderer);
        let layers = match &mut renderer {
            iced::Renderer::Secondary(tiny) => tiny.layers(),
            _ => unreachable!(),
        };
        let mut cache = ResourceCache::default();
        let mut bms = Vec::new();
        let replayed = lower_layers_v2(layers, &mut Vec::new(), &mut cache, &mut bms, false, false, 2.0);
        let replay = bottom(&replayed).expect("bottom text survives host replay");
        assert!(
            (replay.0 - x - 5.0).abs() < 0.1 && (replay.1 - y - 7.0).abs() < 0.1,
            "host replay lost/doubled parent transform: {:?} -> {replay:?}",
            (x, y)
        );
        surface.point_at(Some(iced::Point::new(x + 2.0, y + 8.0)));
        surface.update(&[
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )),
        ]);
        assert_eq!(
            surface.component.0, 8,
            "bottom button cannot be clicked after scroll"
        );
    }

    #[test]
    fn p060_native_headless_ctrl_a_replaces_existing_input() {
        use crate::ui::desktop_protocol::{endpoint::FrameSource, message::{InputMsg, MouseButton}};
        use crate::ui::View;
        #[derive(Debug)]
        struct NativeInput(String);
        impl Component for NativeInput {
            type Msg = ();
            fn on(&mut self, _: ()) { self.0 = crate::ui::iced::last_input_text(); }
            fn view(&self) -> View<()> {
                View::input("title").value(self.0.clone()).on_change(()).build()
            }
        }
        let mut source = HeadlessFrameSource::new(NativeInput("before".into()), 320.0, 100.0);
        source.render_frame();
        source.on_input(&InputMsg::PointerPressed { wid: 1, button: MouseButton::Left, x: 20.0, y: 15.0, modifiers: 0 });
        source.on_input(&InputMsg::PointerReleased { wid: 1, button: MouseButton::Left, x: 20.0, y: 15.0, modifiers: 0 });
        source.on_input(&InputMsg::KeyPressed { wid: 1, key: 0x41, modifiers: 2 });
        source.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
        assert_eq!(source.component().0, "x");
    }

    #[test]
    fn p060_native_headless_action_updates_component_and_frame_revision() {
        use crate::ui::desktop_protocol::{endpoint::FrameSource, message::InputMsg};
        use crate::ui::view::View;

        #[derive(Debug)]
        struct NativeLabel(String);
        impl Component for NativeLabel {
            type Msg = String;
            fn on(&mut self, msg: String) { self.0 = msg; }
            fn view(&self) -> View<String> { View::text(self.0.clone()) }
            fn dispatch_named(&mut self, name: &str, arg: Option<&str>) -> bool {
                match (name, arg) {
                    ("SetLabel", Some(value)) => { self.on(value.to_string()); true }
                    _ => false,
                }
            }
        }
        let mut source = HeadlessFrameSource::new(NativeLabel("before".into()), 320.0, 200.0);
        let revision = source.revision();
        for (name, arg) in [("Unknown", Some("wrong")), ("SetLabel", None)] {
            source.on_input(&InputMsg::Action { wid: 1, name: name.into(), arg: arg.map(str::to_string) });
            assert_eq!(source.revision(), revision);
            assert_eq!(source.surface.component.0, "before");
        }
        source.on_input(&InputMsg::Action { wid: 1, name: "SetLabel".into(), arg: Some("after".into()) });
        assert_eq!(source.revision(), revision + 1);
        assert_eq!(source.surface.component.0, "after");
        assert!(source.render_frame().ops.iter().any(|op| matches!(
            op, DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } if text == "after"
        )));
    }

    fn example_source(dir: &str) -> Option<String> {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
        let path = format!("{base}{dir}/src/front/app.at");
        std::fs::read_to_string(&path).ok()
    }

    /// PLAN-683 T-00 —— headless 首帧 spike：001 组件树经 tiny_skia 记录层
    /// 截获 → DrawList v1 → codec round-trip → paint_ops 回放（`()` canvas
    /// 后端，debug 断言档）。验收：非空帧 + 背景铺底 + 标题文本在档。
    #[test]
    fn p683_t00_helloworld_headless_first_frame() {
        let Some(src) = example_source("001-helloworld") else {
            eprintln!("[p683] skip: 001-helloworld 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 320.0);
        let list = surface.render_frame();

        assert!(!list.ops.is_empty(), "空帧——headless draw 未产原语");
        assert!(list.clear.is_some(), "缺省 clear（主题底）缺席");

        // 标题文本：TextStyled 含 "Hello, World!"（run 级提取）。
        let has_title = list.ops.iter().any(|op| match op {
            DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => {
                text.contains("Hello, World!")
            }
            _ => false,
        });
        assert!(has_title, "标题文本未入帧: {:?}", list.ops);

        // round-trip：encode → decode 恒等（v1 codec 兼容性）。
        let mut encoded = Vec::new();
        list.encode(&mut encoded);
        let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&encoded);
        let decoded = DrawList::decode(&mut reader).expect("decode");
        assert_eq!(list, decoded, "round-trip 漂移");

        // paint_ops 回放：`()` canvas 后端（debug 断言档 geometry::Renderer）
        // 驱动 daemon 同一降格路径——op 词汇面兼容性（越界值/路径构造炸点）。
        replay_with_paint_ops(&list);
    }

    /// 003-converter：交互组件树（button/input）首帧——组件覆盖 =
    /// iced 全集（coverage 门禁在 remote 模式失效的设计前验证）。
    #[test]
    fn p683_t00_converter_headless_first_frame() {
        let Some(src) = example_source("003-converter") else {
            eprintln!("[p683] skip: 003-converter 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 360.0);
        let list = surface.render_frame();
        assert!(!list.ops.is_empty(), "003 空帧");
        let has_text = list
            .ops
            .iter()
            .any(|op| matches!(op, DrawOp::Text { .. } | DrawOp::TextStyled { .. }));
        assert!(has_text, "003 无文本原语（标签/数值面缺席）");
        let mut encoded = Vec::new();
        list.encode(&mut encoded);
        let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&encoded);
        let decoded = DrawList::decode(&mut reader).expect("decode");
        assert_eq!(list, decoded, "003 round-trip 漂移");
        replay_with_paint_ops(&list);
    }

    /// PLAN-683 T-01（AC-04）—— v2 产线帧：headless 降格产 DisplayList v2
    /// 全原语（001/003 载体）→ codec round-trip 恒等；v1 降格面与 v2 直搬
    /// 面（from_v1）语义等价（quad/text 同几何同色）。
    #[test]
    fn p683_t01_v2_first_frame_round_trip() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p683] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut surface = HeadlessSurface::new(component, 480.0, 360.0);

            let v1 = surface.render_frame();
            let lifted = DisplayList::from_v1(&v1);
            let v2 = surface.render_frame_v2();

            // v2 非空 + 文本在档。
            assert!(!v2.ops.is_empty(), "{dir} v2 空帧");
            assert!(
                v2.ops
                    .iter()
                    .any(|op| matches!(op, DisplayOp::TextStyled { .. })),
                "{dir} v2 无文本"
            );

            // round-trip 恒等。
            let mut buf = Vec::new();
            v2.encode(&mut buf);
            assert_eq!(buf[0], 2, "{dir} v2 载荷 tag");
            let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&buf);
            assert_eq!(
                DisplayList::decode(&mut reader).unwrap(),
                v2,
                "{dir} v2 round-trip 漂移"
            );

            // v1 降格 vs v2 原生：文本 run 集合等价（同组件同驱动的
            // 两条降格路径文本面一致——run 提取单源）。
            let texts_of = |list: &DisplayList| -> Vec<(f32, f32, String)> {
                list.ops
                    .iter()
                    .filter_map(|op| match op {
                        DisplayOp::TextStyled { x, y, text, .. } => Some((*x, *y, text.clone())),
                        _ => None,
                    })
                    .collect()
            };
            assert_eq!(
                texts_of(&lifted),
                texts_of(&v2),
                "{dir} 文本面 v1/v2 不一致"
            );

            // 消息信封 round-trip（FrameReadyV2）。
            let msg = crate::ui::desktop_protocol::message::ProtocolMsg::Frame(
                crate::ui::desktop_protocol::message::FrameMsg::FrameReadyV2 {
                    wid: 1,
                    frame_id: 2,
                    slot: 0,
                    damage: None,
                    revision: 1,
                    payload: v2.clone(),
                },
            );
            let bytes = msg.encode();
            assert_eq!(
                crate::ui::desktop_protocol::message::ProtocolMsg::decode(&bytes).unwrap(),
                msg,
                "{dir} FrameReadyV2 信封 round-trip 漂移"
            );
        }
    }

    /// PLAN-690 T-02/T-03/T-05 —— IME 截获环（headless 单元级）：
    /// 点击聚焦 → `ImeRequest(Enabled{cursor})` + hover 光标 `SetCursor(2)`
    /// 下行批；Preedit（含字节选区）注入 → text_input 收态 → 下一帧经
    /// `Enabled.preedit` 原样上报（`preedit_reported` 断言口）→ Commit
    /// 入值出帧 → Esc 取消清组合态。
    #[test]
    fn p690_ime_capture_and_preedit_round_trip() {
        use crate::ui::desktop_protocol::message::{InputMsg, MouseButton};

        let Some(src) = example_source("003-converter") else {
            eprintln!("[p690] skip: 003-converter 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 360.0);

        // 首帧 + celsius 输入定位（p683 typing 环同锚：值文本 `0`）。
        let first = surface.render_frame();
        let (fx, fy) = first
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::TextStyled { x, y, text, .. } if text == "0" => Some((*x, *y)),
                _ => None,
            })
            .expect("celsius 值文本 `0` 缺席");
        let (cx, cy) = (fx + 4.0, fy + 8.0);

        // hover 同点（光标形状断言前置——text_input hover = Text）。
        surface.point_at(Some(iced::Point::new(cx, cy)));
        // 点击聚焦（Pressed + Released 完整对——iced text_input 点击聚焦）。
        let press = input_to_iced_events(&InputMsg::PointerPressed {
            wid: 0,
            button: MouseButton::Left,
            x: cx,
            y: cy,
            modifiers: 0,
        });
        let release = input_to_iced_events(&InputMsg::PointerReleased {
            wid: 0,
            button: MouseButton::Left,
            x: cx,
            y: cy,
            modifiers: 0,
        });
        surface.update(&press);
        surface.update(&release);

        // 下一帧：RedrawRequested 臂 → text_input 请求 IME（聚焦态）——
        // 截获出 ImeRequest(Enabled) + SetCursor(Text)。
        let _ = surface.render_frame();
        let controls = surface.drain_window_controls();
        assert!(
            controls.iter().any(|c| matches!(
                c,
                ControlMsg::ImeRequest { enabled: Some(req), .. }
                    if req.cursor.w > 0.0 && req.cursor.h > 0.0
            )),
            "聚焦后 IME enable 下行缺席: {controls:?}"
        );
        assert!(
            controls
                .iter()
                .any(|c| matches!(c, ControlMsg::SetCursor { kind: 2, .. })),
            "text_input hover 光标（Text）下行缺席: {controls:?}"
        );

        // Preedit 注入（组合串 + 字节选区）→ 下一帧 text_input 经
        // Enabled.preedit 原样上报。
        let preedit = input_to_iced_events(&InputMsg::ImePreedit {
            wid: 0,
            text: "nihao".into(),
            selection: Some((2, 4)),
        });
        surface.update(&preedit);
        let _ = surface.render_frame();
        assert_eq!(
            surface.preedit_reported(),
            Some("nihao"),
            "组合串未达 text_input 态（Enabled.preedit 上报缺席）"
        );
        let re_drain = surface.drain_window_controls();
        assert!(
            re_drain.is_empty(),
            "组合态帧不重发 IME 请求（去重基线失效）: {re_drain:?}"
        );

        // Commit 入值：winit Windows commit 序 = `Preedit("")` 清态先行 +
        // `Commit(text)`（event_loop.rs:1546-1556）——镜像真实事件序。
        let clear = input_to_iced_events(&InputMsg::ImePreedit {
            wid: 0,
            text: String::new(),
            selection: None,
        });
        surface.update(&clear);
        let commit = input_to_iced_events(&InputMsg::ImeCommit {
            wid: 0,
            text: "你好".into(),
        });
        surface.update(&commit);
        let committed = surface.render_frame();
        assert!(
            committed.ops.iter().any(|op| matches!(
                op,
                DrawOp::TextStyled { text, .. } | DrawOp::Text { text, .. } if text.contains("你好")
            )),
            "Commit 后帧文本不含上屏串"
        );
        assert_eq!(
            surface.preedit_reported(),
            None,
            "Commit 后组合态未清（preedit 上报残留）"
        );

        // Esc 取消（ImeCancelled → Closed 清组合态——G1 取消腿）。
        let cancelled = input_to_iced_events(&InputMsg::ImeCancelled { wid: 0 });
        surface.update(&cancelled);
        let _ = surface.render_frame();
        assert_eq!(surface.preedit_reported(), None, "取消后组合态残留");
    }

    /// daemon paint_ops 回放（`()` 后端）——与 broker_surface 的实机重放
    /// 同一降格函数。debug 断言档外该后端不存在，测试跳过（release 测档
    /// 由 T-03 实机窗承接）。
    /// PLAN-702 T-05 前置修复（预存 release 编译炸点，非本计划引入）：
    /// iced_graphics 的 `impl Renderer for ()` 挂 `#[cfg(debug_assertions)]`
    /// ——debug 档外该类型不存在；本函数原用**运行期** `cfg!(debug_
    /// assertions)` 门，release 测档（AC-03 `cargo test --release`）照样
    /// 编译即炸 E0277。改编译期门整段消除，语义与注释自述一致。
    fn replay_with_paint_ops(list: &DrawList) {
        #[cfg(debug_assertions)]
        replay_with_paint_ops_debug(list);
        #[cfg(not(debug_assertions))]
        let _ = list;
    }

    /// `()` 后端重放体——仅在 debug 断言档编译（与 iced `()` Renderer 的
    /// 存在档位对齐）。
    #[cfg(debug_assertions)]
    fn replay_with_paint_ops_debug(list: &DrawList) {
        let bounds = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(640.0, 480.0));
        let mut frame = iced::widget::canvas::Frame::with_bounds(&(), bounds);
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            iced::Color::from_rgba8(
                list.clear.map(|c| c.r).unwrap_or(0),
                list.clear.map(|c| c.g).unwrap_or(0),
                list.clear.map(|c| c.b).unwrap_or(0),
                1.0,
            ),
        );
        crate::ui::iced::broker_surface::paint_ops(&mut frame, &list.ops);
    }

    /// PLAN-683 T-03——v2 原语 canvas 重放：001/003 实帧（含渐变/圆角/
    /// border/shadow 全参 quad）经 `()` 后端完整走 paint_ops_v2 降格
    /// 路径（词汇面兼容/路径构造炸点/嵌套 scissor-transform 扫描）。
    /// 同 replay_with_paint_ops 的编译期门修复（`()` Renderer 存在档位）。
    #[test]
    fn p683_t03_v2_paint_replay() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p683] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut surface = HeadlessSurface::new(component, 480.0, 360.0);
            let v2 = surface.render_frame_v2();
            assert!(!v2.ops.is_empty(), "{dir} v2 空帧");
            #[cfg(debug_assertions)]
            {
                let bounds =
                    iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(640.0, 480.0));
                let mut frame = iced::widget::canvas::Frame::with_bounds(&(), bounds);
                if let Some(clear) = v2.clear {
                    frame.fill_rectangle(
                        iced::Point::ORIGIN,
                        bounds.size(),
                        iced::Color::from_rgba8(clear.r, clear.g, clear.b, 1.0),
                    );
                }
                crate::ui::iced::broker_surface::paint_ops_v2(&mut frame, &v2.ops);
            }
            #[cfg(not(debug_assertions))]
            let _ = v2;
        }
    }
}
