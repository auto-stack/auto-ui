//! PLAN-014 T-06：媒体/纹理责任（cached_media_handle/render_image_surface/render_video/svg-handle/图像句柄缓存/lucide 字形族）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::devtools::*;

/// Per-asset iced image::Handle cache (P547 flicker fix).
///
/// `Handle::from_rgba` mints a fresh `Id::unique()` on every call, so a new
/// Handle per frame makes iced treat the image as brand-new: texture cache
/// misses and re-uploads race the frame clock — the picture visibly
/// disappears and reappears (mean-luma flicker reproduced with sequential
/// screenshots). Keyed by media asset id; the pixel `Arc` identity gates
/// reuse, so a re-published rendition still swaps the texture. Bounded:
/// beyond 128 entries the table is dropped (viewer sessions hold a handful
/// of renditions; eviction semantics live in the pipeline caches above).
fn cached_media_handle(
    asset: auto_media_core::image_pipeline::MediaAssetId,
    width: u32,
    height: u32,
    rgba: std::sync::Arc<[u8]>,
) -> iced::widget::image::Handle {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    // Keyed by the opaque `MediaAssetId` itself (PLAN-811: the raw u128
    // field is private to auto-media-core; the id derives Hash/Eq).
    static CACHE: OnceLock<
        Mutex<
            HashMap<
                auto_media_core::image_pipeline::MediaAssetId,
                (std::sync::Arc<[u8]>, iced::widget::image::Handle),
            >,
        >,
    > = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("media handle cache poisoned");
    if let Some((prev, handle)) = guard.get(&asset) {
        if std::sync::Arc::ptr_eq(prev, &rgba) {
            return handle.clone();
        }
    }
    let handle = iced::widget::image::Handle::from_rgba(width, height, rgba.to_vec());
    if guard.len() >= 128 {
        guard.clear();
    }
    guard.insert(asset, (rgba, handle.clone()));
    handle
}

/// Render an ImageSurface without entering the legacy Image loader/cache.
/// Media tickets are resolved only through the process-local registry; a
/// pending/expired ticket therefore renders a stable placeholder instead of
/// performing synchronous file or HTTP I/O on the UI thread.
pub(super) fn render_image_surface<M: Clone + Debug + 'static>(
    src: String,
    alt: String,
    width: u32,
    height: u32,
    quality: u8,
    fit: String,
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    rotation: i32,
    _filter: String,
    style: Option<Style>,
) -> iced::Element<'static, M> {
    use crate::ui::iced::image_surface::ImageSurfaceFit;

    let is = style.as_ref().map(IcedStyle::from_style);
    let width_hint = if width > 0 {
        Some(iced::Length::Fixed(width as f32))
    } else {
        is.as_ref().and_then(|s| s.width.as_ref().map(iced_length))
    };
    let height_hint = if height > 0 {
        Some(iced::Length::Fixed(height as f32))
    } else {
        is.as_ref().and_then(|s| s.height.as_ref().map(iced_length))
    };
    let object_fit = match ImageSurfaceFit::parse(&fit) {
        ImageSurfaceFit::Contain => iced::ContentFit::Contain,
        ImageSurfaceFit::Width => iced::ContentFit::Contain,
        ImageSurfaceFit::OneToOne => iced::ContentFit::None,
        ImageSurfaceFit::Free => iced::ContentFit::None,
    };
    let filter_method = if quality < 80 {
        iced::widget::image::FilterMethod::Nearest
    } else {
        iced::widget::image::FilterMethod::Linear
    };

    // ImageSurface's contract is an opaque media URI. Consume only pixels
    // that a background resize lane has already decoded and cached; this path
    // never opens a file or decodes encoded bytes on the UI thread.
    let pixels = if src.starts_with("/api/__auto/media/") {
        auto_media_core::image_pipeline::resolve_media_render(&src)
    } else {
        None
    };
    let mut inner: iced::Element<'static, M> =
        if let Some((asset_id, source_width, source_height, rgba)) = pixels {
            let zoom = zoom.clamp(0.05, 64.0);
            let viewport = iced::Rectangle {
                x: 0.0,
                y: 0.0,
                width: width.max(source_width) as f32,
                height: height.max(source_height) as f32,
            };
            let geometry = crate::ui::iced::image_surface::image_surface_geometry(
                viewport,
                iced::Size::new(source_width as f32, source_height as f32),
                ImageSurfaceFit::parse(&fit),
                zoom,
                offset_x,
                offset_y,
                rotation as f32,
            );
            let display_width = geometry.rotated_bounds.width.max(1.0);
            let display_height = geometry.rotated_bounds.height.max(1.0);
            let radians = (rotation.rem_euclid(360) as f32).to_radians();
            let mut image = iced::widget::image(cached_media_handle(
                asset_id,
                source_width,
                source_height,
                rgba,
            ))
            .content_fit(object_fit)
            .filter_method(filter_method)
            .rotation(iced::Radians::from(radians));
            if width > 0 {
                image = image.width(iced::Length::Fixed(
                    display_width.max(width as f32 + offset_x.abs()),
                ));
            } else if let Some(w) = width_hint {
                image = image.width(w);
            }
            if height > 0 {
                image = image.height(iced::Length::Fixed(
                    display_height.max(height as f32 + offset_y.abs()),
                ));
            } else if let Some(h) = height_hint {
                image = image.height(h);
            }
            // Keep the translation in the clipping layout as well as in the
            // geometry calculation. This makes pan state observable without ever
            // moving decode or filesystem work onto the render thread.
            container(image)
                .padding(iced::Padding {
                    top: offset_y.max(0.0),
                    right: (-offset_x).max(0.0),
                    bottom: (-offset_y).max(0.0),
                    left: offset_x.max(0.0),
                })
                .into()
        } else {
            let label = if alt.is_empty() {
                "Image unavailable".to_string()
            } else {
                alt
            };
            let mut placeholder = container(text(label).size(14))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill);
            if let Some(w) = width_hint {
                placeholder = placeholder.width(w);
            }
            if let Some(h) = height_hint {
                placeholder = placeholder.height(h);
            }
            placeholder.into()
        };

    let mut surface = container(inner).clip(true);
    if let Some(w) = width_hint {
        surface = surface.width(w);
    }
    if let Some(h) = height_hint {
        surface = surface.height(h);
    }
    if let Some(ref style) = is {
        let background = style.background_color.map(iced::Background::Color);
        let border_color = style.border_color.unwrap_or(iced::Color::TRANSPARENT);
        let border_width = style.border_width.unwrap_or(0.0);
        let border_radius = style.border_radius.unwrap_or(0.0);
        surface = surface.style(move |_theme| container::Style {
            background,
            border: iced::Border::default()
                .rounded(border_radius.min(9999.0))
                .width(border_width)
                .color(border_color),
            ..Default::default()
        });
    }
    inner = surface.into();
    inner
}


/// `video` 的渲染面（PLAN-617 T-19）。
///
/// **有 `mpv-widget` 时**：建自定义 shader widget，把 T-16/T-17/T-18 的
/// 引擎 + 帧上屏通道 + 受控契约接上；帧由应用既有的 tick 驱动（见 `mpv::widget`
/// 模块文档）。
///
/// **没有该 feature 时**：保持**诚实降级**——渲染一个带说明的面板而不是黑屏
/// （AC-10/AC-11：静默黑屏是本计划要消灭的行为）。
#[allow(clippy::too_many_arguments)]
pub(super) fn render_video<M: Clone + Debug + 'static>(
    src: String,
    paused: bool,
    position: Option<f64>,
    volume: i32,
    muted: bool,
    rate: f64,
    label: String,
    on_time_update: Option<crate::ui::view::MediaEventHandler<M>>,
    on_loaded_metadata: Option<crate::ui::view::MediaEventHandler<M>>,
    on_play_state: Option<crate::ui::view::MediaEventHandler<M>>,
    on_ended: Option<M>,
    on_media_error: Option<crate::ui::view::MediaEventHandler<M>>,
    media_key: Option<String>,
    style: Option<Style>,
) -> iced::Element<'static, M> {
    use iced::Length;

    #[cfg(feature = "mpv-widget")]
    {
        use crate::ui::iced::video_uplink::VideoUplink;
        use crate::ui::mpv::widget::{VideoProgram, VideoWidgetProps};
        use crate::ui::mpv::VideoContractDown;

        // 稳定运行时键：`key:` 声明 + 当前会话域合成（域缺席 = ns 0——
        // 面板/桌面 chrome 等非 app 视图构建，键只落 key 串本身）。
        let runtime_key = media_key.map(|k| {
            let ns = MEDIA_VIEW_SCOPE.with(|c| c.get()).unwrap_or(0);
            crate::ui::mpv::widget::stable_runtime_key(ns, &k)
        });

        // 尺寸：交给父容器（布局说了算）；widget 自身撑满可用空间。
        // PLAN-721 T-19：下行世代打戳——每次视图构建（convert）取新 epoch，
        // 该代全部 primitive 同号；contract 的单调门据此拒绝旧代迟到 prepare。
        let program = VideoProgram::new(VideoWidgetProps {
            down: VideoContractDown {
                paused,
                position,
                volume,
                muted,
                rate,
                src: if src.is_empty() {
                    None
                } else {
                    Some(src.clone())
                },
                epoch: crate::ui::mpv::contract::next_down_epoch(),
            },
            width: 0,
            height: 0,
        })
        .with_runtime_key(runtime_key);
        let shader = iced::widget::shader::Shader::new(program)
            .width(Length::Fill)
            .height(Length::Fill);
        // PLAN-712 T-04: 事件泵 wrapper 直接包 shader（children[0] 即 shader
        // 的 Tree,泵靠它定位运行时 id）——黑底 letterbox 容器包在 wrapper 外。
        let inner: iced::Element<'static, M> = VideoUplink::new(shader)
            .with_runtime_key(runtime_key)
            .on_time_update(on_time_update)
            .on_loaded_metadata(on_loaded_metadata)
            .on_play_state(on_play_state)
            .on_ended(on_ended)
            .on_media_error(on_media_error)
            .into();
        let mut surface = iced::widget::container(inner)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                ..Default::default()
            });
        if let Some(style) = style.as_ref() {
            // 与其它容器臂同源：把 AURA 的 `Style` 翻成 iced 的 container 样式
            // （背景/边框/圆角），尺寸类由父布局决定（widget 自身 Fill）。
            let is = IcedStyle::from_style(style);
            let cs = build_container_style(&is);
            surface = surface.style(move |_theme| cs.clone());
            if let Some(ref w) = is.width {
                surface = surface.width(iced_length(w));
            }
            if let Some(ref h) = is.height {
                surface = surface.height(iced_length(h));
            }
        }
        return surface.into();
    }

    #[cfg(not(feature = "mpv-widget"))]
    {
        let _ = (
            src,
            paused,
            position,
            volume,
            muted,
            rate,
            style,
            media_key,
            on_time_update,
            on_loaded_metadata,
            on_play_state,
            on_ended,
            on_media_error,
        );
        // 诚实占位：说明本后端没接上原生播放，而不是留一块黑。
        let text = if label.is_empty() {
            "视频：本后端未启用原生播放（构建时未开 `mpv-widget`）".to_string()
        } else {
            format!(
                "{label}
本后端未启用原生播放（构建时未开 `mpv-widget`）"
            )
        };
        iced::widget::container(iced::widget::text(text).size(13))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.06, 0.06, 0.08,
                ))),
                text_color: Some(iced::Color::from_rgb(0.6, 0.6, 0.65)),
                ..Default::default()
            })
            .into()
    }
}

/// Download image bytes from a URL using blocking HTTP.
/// Results are cached in memory so each URL is only fetched once.
/// Returns None on failure.
/// PLAN-028：提权 pub(crate)——DrawList 图像算子（queue 臂 broker_surface）
/// 复用同一 src 词汇与负缓存（单源解析纪律）。
pub(crate) fn load_image_bytes(url: &str) -> Option<Vec<u8>> {
    // Process-local media tickets are resolved before the legacy URL/file
    // cache. A pending ticket must be retried on the next frame rather than
    // being cached as a permanent miss.
    if url.starts_with("/api/__auto/media/") {
        return auto_media_core::image_pipeline::resolve_media_uri(url);
    }
    // PLAN-099 T-05：相对 URL（`/` 开头）按 `AUTO_HTTP_BASE`/`AUTO_HTTP_PORT`
    // 展开为绝对地址（复用 stdlib 单源助手——与 VM 轨 `Http.get` 同一词汇）。
    // 此前相对路径落本地文件臂恒失败：VM 壳无页面源，musk 面板帧区
    // `/api/canvas/frame?...` 永远读不到（web 轨浏览器 origin 正常，实测恒 "?"）。
    let url: &str = &auto_lang_core::vm::ffi::stdlib::resolve_http_base_url(url);
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, Option<Vec<u8>>>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Check cache first
    {
        let lock = cache.lock().unwrap();
        if let Some(cached) = lock.get(url) {
            return cached.clone();
        }
    }

    // 2026-09-22 修：本地文件读失败不再永久负缓存——首读失败常见于
    // boot 竞态（杀软持句柄/休眠盘唤醒/写后即读），原实现 None 一旦
    // 入缓存整会话黑屏（壁纸重启丢失实录）。本地路径失败只挂 1s 负
    // 缓存，过期重读自愈；builtin:/data: 失败是确定性解析失败、http
    // 负缓存防打挂远端，三者维持原永久缓存语义。
    use std::time::{Duration, Instant};
    static NEGATIVE: std::sync::OnceLock<Mutex<HashMap<String, Instant>>> =
        std::sync::OnceLock::new();
    let negative = NEGATIVE.get_or_init(|| Mutex::new(HashMap::new()));
    const FILE_MISS_RETRY: Duration = Duration::from_secs(1);
    let is_local_file = !url.starts_with("builtin:")
        && !url.starts_with("data:")
        && !url.starts_with("http://")
        && !url.starts_with("https://")
        && !url.starts_with("/api/__auto/media/");
    if is_local_file {
        if let Some(t) = negative.lock().unwrap().get(url) {
            if t.elapsed() < FILE_MISS_RETRY {
                return None;
            }
        }
    }

    // Fetch and cache
    let result = if let Some(name) = url.strip_prefix("builtin:") {
        // Plan 518：内嵌壁纸虚拟方案(builtin:ricepaper / builtin:inkwash)。
        match name {
            "ricepaper" => Some(WALLPAPER_RICEPAPER.to_vec()),
            "inkwash" => Some(WALLPAPER_INKWASH.to_vec()),
            _ => None,
        }
    } else if let Some(rest) = url.strip_prefix("data:") {
        // Plan 606: Data URL support (SVG / PNG / JPEG / WEBP etc.)
        // Format: data:[<mediatype>][;base64],<data>
        if let Some(comma_pos) = rest.find(',') {
            let meta = &rest[..comma_pos];
            let payload = &rest[comma_pos + 1..];
            if meta.contains(";base64") {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(payload.trim())
                    .ok()
            } else {
                urlencoding::decode(payload)
                    .map(|s| s.into_owned().into_bytes())
                    .ok()
            }
        } else {
            None
        }
    } else if url.starts_with("http://") || url.starts_with("https://") {
        // os-007（P534-D4）：远程图源同步抓取加 3s 超时——此前裸
        // `reqwest::blocking::get` 在网络不可达时把渲染线程挂到 TCP
        // 超时（bounds 不更新/截图超时/导航无响应，avatar 家族接通
        // 网络图源后实机复现）。结果仍按 URL 缓存，阻塞每 URL 至多一次。
        static HTTP: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
        let client = HTTP.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .unwrap_or_default()
        });
        client
            .get(url)
            .send()
            .ok()?
            .bytes()
            .ok()
            .map(|b| b.to_vec())
    } else {
        // Try loading from local file path
        std::fs::read(url).ok()
    };

    if result.is_some() {
        cache
            .lock()
            .unwrap()
            .insert(url.to_string(), result.clone());
        negative.lock().unwrap().remove(url);
    } else if is_local_file {
        negative
            .lock()
            .unwrap()
            .insert(url.to_string(), Instant::now());
    } else {
        cache.lock().unwrap().insert(url.to_string(), None);
    }
    result
}

/// Cache image::Handle by URL and radius to avoid flickering.
/// Creating a new Handle each frame causes Iced to re-decode and re-upload the texture.
pub(super) fn get_or_create_image_handle(
    url: &str,
    data: Vec<u8>,
    border_radius: f32,
    layout_w: Option<f32>,
    layout_h: Option<f32>,
) -> iced::widget::image::Handle {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, iced::widget::image::Handle>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let cache_key = format!("{url}#r={border_radius}_{layout_w:?}_{layout_h:?}");
    let mut lock = cache.lock().unwrap();
    if let Some(handle) = lock.get(&cache_key) {
        return handle.clone();
    }

    let handle = if border_radius > 0.0 {
        if let Ok(dynamic_img) = image::load_from_memory(&data) {
            let mut rgba = dynamic_img.into_rgba8();
            let (w, h) = rgba.dimensions();
            if w > 0 && h > 0 {
                let r_px = if border_radius >= 9999.0 {
                    (w.min(h) as f32) / 2.0
                } else {
                    let scale_x =
                        layout_w.map_or(1.0, |lw| if lw > 0.0 { w as f32 / lw } else { 1.0 });
                    let scale_y =
                        layout_h.map_or(1.0, |lh| if lh > 0.0 { h as f32 / lh } else { 1.0 });
                    let scale = (scale_x + scale_y) / 2.0;
                    (border_radius * scale).min((w.min(h) as f32) / 2.0)
                };

                let cx = (w as f32) / 2.0;
                let cy = (h as f32) / 2.0;
                let half_w = cx;
                let half_h = cy;

                for y in 0..h {
                    for x in 0..w {
                        let px_val = ((x as f32 + 0.5) - cx).abs();
                        let py_val = ((y as f32 + 0.5) - cy).abs();
                        let qx = px_val - (half_w - r_px);
                        let qy = py_val - (half_h - r_px);
                        let dist = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt()
                            + qx.max(qy).min(0.0)
                            - r_px;
                        if dist > 0.5 {
                            rgba.get_pixel_mut(x, y).0[3] = 0;
                        } else if dist > -0.5 {
                            let factor = (-dist + 0.5).clamp(0.0, 1.0);
                            let pixel = rgba.get_pixel_mut(x, y);
                            pixel.0[3] = (pixel.0[3] as f32 * factor) as u8;
                        }
                    }
                }
                iced::widget::image::Handle::from_rgba(w, h, rgba.into_raw())
            } else {
                iced::widget::image::Handle::from_bytes(data)
            }
        } else {
            iced::widget::image::Handle::from_bytes(data)
        }
    } else {
        iced::widget::image::Handle::from_bytes(data)
    };

    lock.insert(cache_key, handle.clone());
    handle
}

/// Plan 409 §8: recursively apply an inherited text color to every Text node
/// in the view that has no explicit `text-{color}` class. This gives the vue
/// equivalent of CSS text-color inheritance — a `link (to:) { text "Docs" }`
/// child inherits the button's theme color (text-primary) instead of falling
/// back to the body default (text-foreground). Only leaves that carry no
/// explicit color are touched; everything else (layout containers, images,
/// buttons) is preserved.
pub(super) fn inherit_text_color<M: Clone + Debug>(view: &mut AbstractView<M>, color: Color) {
    match view {
        AbstractView::Text { style, .. } => {
            let has_explicit_color = style.as_ref().map_or(false, |s| {
                s.classes
                    .iter()
                    .any(|c| matches!(c, StyleClass::TextColor(_)))
            });
            if !has_explicit_color {
                let mut inherited = style.take().unwrap_or_default();
                inherited.classes.push(StyleClass::TextColor(color));
                *style = Some(inherited);
            }
        }
        AbstractView::Button { content, .. } => {
            if let Some(c) = content {
                inherit_text_color(c, color);
            }
        }
        // PLAN-054 T5 (A10): Image 同样继承按钮文本色——发送钮
        // `bg-primary text-primary-foreground { Send{} }` 的图标此前落
        // OnBackground 回退,亮色主题下白钮白标不可见。
        AbstractView::Image { style, .. } => {
            let has_explicit_color = style.as_ref().map_or(false, |s| {
                s.classes
                    .iter()
                    .any(|c| matches!(c, StyleClass::TextColor(_)))
            });
            if !has_explicit_color {
                let mut inherited = style.take().unwrap_or_default();
                inherited.classes.push(StyleClass::TextColor(color));
                *style = Some(inherited);
            }
        }
        AbstractView::ImageSurface { style, .. } => {
            let has_explicit_color = style.as_ref().map_or(false, |s| {
                s.classes
                    .iter()
                    .any(|c| matches!(c, StyleClass::TextColor(_)))
            });
            if !has_explicit_color {
                let mut inherited = style.take().unwrap_or_default();
                inherited.classes.push(StyleClass::TextColor(color));
                *style = Some(inherited);
            }
        }
        AbstractView::Row { children, .. }
        | AbstractView::Column { children, .. }
        | AbstractView::List {
            items: children, ..
        } => {
            for child in children {
                inherit_text_color(child, color);
            }
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells {
                inherit_text_color(cell, color);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            inherit_text_color(child, color);
        }
        AbstractView::Table { headers, rows, .. } => {
            for h in headers {
                inherit_text_color(h, color);
            }
            for row in rows {
                for cell in row {
                    inherit_text_color(cell, color);
                }
            }
        }
        _ => {}
    }
}

/// Plan 408: Return a complete SVG string for a lucide icon name.
/// The SVG uses 16x16 viewport (scaled from lucide's 24x24), stroke-based
/// rendering matching lucide's visual style.
/// Plan 482: lucide_svg returns bare shape fragments (no <svg> root) —
/// resvg refuses to parse them, so every `lucide:` icon rendered EMPTY
/// (pre-existing gap hit by the nav search icon; also affects the PUA
/// button-icon path). Wrap into a full stroke-based document here.
/// Plan 518 G4②: stroke-width 参数化——默认 2（lucide 标准）,大尺寸
/// （渲染 ≥48px）用 1.5 细线对齐 stella 线性观感。
pub(super) fn lucide_svg_doc(name: &str) -> Option<String> {
    lucide_svg_doc_with(name, 2.0)
}

pub(crate) fn lucide_svg_doc_with(name: &str, stroke_width: f32) -> Option<String> {
    // PLAN-619 §8.5（根因定案）+ PLAN-617 全量表：这里必须从**内部 markup**
    // （`lucide_fragment`，24×24 坐标系）按目标尺寸做**单层**包装。历史上这里
    // 曾把 `lucide_svg` 的完整 16×16 文档再套进一层 24×24 `<svg>` → **嵌套
    // viewport** 使 glyph 按 16/24 = 0.667 缩放，跨端图标 ink 系统性偏小
    // （声明 18px 实画 ≈12px，正是 PLAN-619 P2 的「≈12px 盒」；矢量与自栅格化
    // 两臂共用此文档，故差距从不在 iced 绘制路径）。回归锚：
    // `plan619_lucide_doc_renders_geometric_ink`（修前 ink 充满度 0.583 必红，
    // 几何值 ≈0.833）。
    let inner = lucide_fragment(name)?;
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"{stroke_width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{inner}</svg>"
    ))
}

pub(super) fn lucide_svg(name: &str) -> Option<&'static str> {
    // 字形来自 **lucide 官方数据生成的全量表**（`lucide_generated.rs`，由
    // `scripts/gen-lucide-table.mjs` 生成，覆盖 lucide-vue-next 的 1401 个图标）。
    //
    // 这里是 P537-D1 的根治点：在此之前 iced 端没有 lucide 数据源，表是手工
    // 抄的 85 条 —— 与 Vue 端的全量包天然分叉，且**缺名在 VM 端静默渲染成
    // 空盒**（`AbstractView::Image` 的 miss 分支返回 `container(text(""))`），
    // 表现为「Vue 有图标、VM 一片空」。全量表让「同名 ⇒ 同字形」由构造保证。
    let elements: &'static str = lucide_fragment(name)?;
    // Use a small static cache to avoid re-formatting.
    // The SVG uses width/height=16 for compact button rendering.
    // PLAN-530 步骤5：按 icon 名去重——旧实现每次调用 Box::leak 一份新串
    // （注释宣称缓存但实现是无界泄漏），每帧每图标 ~350B 随重建线性增长
    // = B 内存崩塌主源。key 用字形 'static 片段（名↔片段 1:1），免每调用
    // String 键分配。
    static DOC_CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<&'static str, &'static str>>,
    > = std::sync::OnceLock::new();
    let cache = DOC_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut lock = cache.lock().unwrap();
    Some(*lock.entry(elements).or_insert_with(|| {
        Box::leak(
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
                elements
            )
            .into_boxed_str(),
        )
    }))
}

/// 名字 → 字形内部 markup（24×24 坐标系）。全量表优先，然后是**遗留别名**。
///
/// 别名只在「上游改过名、而示例/资产里还用着旧名」时才有意义，且**必须逐条
/// 有据**：`sidebar` 在 lucide 上游已更名为 `panel-left`，`file-icon` 已并入
/// `file`——这两个名字在 lucide-vue-next 里已经不存在，而旧的手抄表里有它们，
/// 所以保留别名以免既有语料在 VM 端忽然变空盒（Vue 端用旧名本来就会 import
/// 失败，本来就不该再用）。
fn lucide_fragment(name: &str) -> Option<&'static str> {
    match crate::ui::iced::lucide_generated::lookup(name) {
        Some(frag) => Some(frag),
        None => match name {
            "sidebar" => crate::ui::iced::lucide_generated::lookup("panel-left"),
            "file-icon" => crate::ui::iced::lucide_generated::lookup("file"),
            // PLAN-012 F2：lucide 官方改名别名（v0.313+ 新名 → 本表旧名）——
            // circle-alert = alert-circle（错误通知 ❗ 用，os-012 F10）。
            "circle-alert" => crate::ui::iced::lucide_generated::lookup("alert-circle"),
            _ => None,
        },
    }
}

/// PLAN-530 步骤5：placeholder 'static 化的按文本去重缓存——text_editor
/// placeholder 契约要求 `&'static str`，旧实现每次重建 `Box::leak` 一份
/// 新串（无界泄漏，随页面重建频率线性增长）。同文本全局只 leak 一份。
pub(super) fn leaked_placeholder(placeholder: &str) -> &'static str {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, &'static str>>,
    > = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut lock = cache.lock().unwrap();
    if let Some(v) = lock.get(placeholder) {
        return v;
    }
    let leaked: &'static str = Box::leak(placeholder.to_string().into_boxed_str());
    lock.insert(placeholder.to_string(), leaked);
    leaked
}

/// Cache svg::Handle by URL to avoid flickering.
pub(super) fn get_or_create_svg_handle(url: &str, data: Vec<u8>) -> iced::widget::svg::Handle {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, iced::widget::svg::Handle>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let mut lock = cache.lock().unwrap();
    if let Some(handle) = lock.get(url) {
        return handle.clone();
    }
    let handle = iced::widget::svg::Handle::from_memory(data);
    lock.insert(url.to_string(), handle.clone());
    handle
}

/// Extract initials from a URL (e.g. seed name) for placeholder display.
pub(super) fn extract_initials(src: &str) -> String {
    if let Some(query) = src.split('?').nth(1) {
        for param in query.split('&') {
            if let Some(value) = param.strip_prefix("seed=") {
                let initials: String = value
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|s| !s.is_empty())
                    .filter_map(|p| p.chars().next())
                    .map(|c| c.to_ascii_uppercase())
                    .take(2)
                    .collect();
                if !initials.is_empty() {
                    return initials;
                }
            }
        }
    }
    "?".to_string()
}

/// Plan 448 对齐批: shadcn 令牌基的 iced 主题。窗口底色此前取内置
/// Theme::Dark(#2B2D31),与 vue 产物 shadcn --background 不一致;
/// 此处以生成端 index.css 的令牌为基(background/foreground 精确换算,
/// primary 用 indigo-500 基色),扩展色阶由 palette::Extended::generate
/// 派生(滚动条/选区等默认控件样式随之取色)。
/// PLAN-683：headless 宿主同源取用（remote 帧与独立轨同主题单源）。
pub(crate) fn shadcn_theme(dark: bool) -> iced::Theme {
    let (background, text) = if dark {
        // --background: hsl(222.2 47.4% 7%) / --foreground: hsl(210 40% 98%)
        // (Plan 518 stella 重校:dark #141a29 精修蓝黑)
        (
            iced::Color::from_rgb8(20, 26, 41),
            iced::Color::from_rgb8(248, 250, 252),
        )
    } else {
        // light 暖纸系 #f5f1e8 / 墨色 #2a2723(Plan 518 stella 对齐)
        (
            iced::Color::from_rgb8(245, 241, 232),
            iced::Color::from_rgb8(42, 39, 35),
        )
    };
    // Plan 458: primary follows the accent preset thread-local (default
    // indigo) instead of hardcoded indigo-500, so `auto run --accent` /
    // pac.at `accent:` drive window-level iced defaults too. Unknown names
    // keep the legacy indigo-500 fallback.
    let primary =
        crate::ui::style::theme::accent_primary_rgb(&crate::ui::style::theme::accent_name(), dark)
            .map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
            .unwrap_or_else(|| iced::Color::from_rgb8(99, 102, 241));
    iced::Theme::custom(
        if dark {
            "AutoShadcnDark"
        } else {
            "AutoShadcnLight"
        },
        iced::theme::Palette {
            background,
            text,
            primary,
            success: iced::Color::from_rgb8(34, 197, 94),
            warning: iced::Color::from_rgb8(234, 179, 8),
            danger: iced::Color::from_rgb8(239, 68, 68),
        },
    )
}

