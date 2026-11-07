//! Plan 500 步骤 5 —— broker 表面两态合成（宿主渲染臂）。
//!
//! 虚拟窗客户区内容源：**broker client 的帧**（进程外 App 经协议上报）
//! 取代宿主本地重渲染（v1.2 的临时形态——host 侧 dynamic_view 同源重画，
//! child 帧仅消息级断言；本模块兑现"live-iced 消费"，host.rs 头注遗留点）。
//!
//! 两态（`Welcome.frame_mode` 协商）：
//! - **Commands（queue 臂）**：`DrawList` → canvas Program 降级——Quad =
//!   抗锯齿 fill、Text = `fill_text` 宿主侧 shaping（D1 定案 A：宿主
//!   iced 文本栈，零新依赖）、clear = 底色。damage v1.3 作重绘提示
//!   （每帧全量重建几何，正确性不受损；Cache 局部化归 Stage 5）。
//! - **Pixels（independent 臂）**：shm RGBA → `image::Handle::from_rgba`
//!   上传（497 快照同通道口径）→ Image 挂客户区。

use crate::ui::desktop_protocol::message::{DrawList, DrawOp, Rgba8};
use crate::ui::desktop_protocol::stage3::PixelsSurface;
use crate::ui::iced::renderer::INTER_FONT;
use crate::ui::session::{DesktopMessage, DesktopSession, Wid};

pub(crate) fn to_color(c: Rgba8) -> iced::Color {
    iced::Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// CSS 字重刻度（100..900）→ iced `Weight` 档（Plan 515 G2）。
fn css_weight_to_iced(w: u16) -> iced::font::Weight {
    match w {
        100 => iced::font::Weight::Thin,
        200 => iced::font::Weight::ExtraLight,
        300 => iced::font::Weight::Light,
        500 => iced::font::Weight::Medium,
        600 => iced::font::Weight::Semibold,
        700 => iced::font::Weight::Bold,
        800 => iced::font::Weight::ExtraBold,
        900 => iced::font::Weight::Black,
        _ => iced::font::Weight::Normal,
    }
}

/// TextStyled/Text 线载荷 → 宿主重放字体（PLAN-102 F-RF-3 修复）。
///
/// 必须钉 app 字体栈 family（Inter——Plan 411 内嵌三字重）：iced 默认
/// generic family + 非常规字重（如按钮 Medium=500）在 cosmic 系统匹配下
/// 会乱选字面（实机 = 重放文本碎片化乱码）。w=400 恰整值等于
/// `Font::DEFAULT` 被 `default_font` 替换兜住，其余字重不会——与
/// `renderer::font_weight_to_iced` 钉 Inter 同律（app 内带字重文本先例）。
fn wire_font(weight: u16, italic: bool) -> iced::Font {
    iced::Font {
        family: INTER_FONT.family,
        weight: css_weight_to_iced(weight),
        style: if italic {
            iced::font::Style::Italic
        } else {
            iced::font::Style::Normal
        },
        ..Default::default()
    }
}

/// DrawList → canvas 绘制程序（queue 臂栅格化：宿主 GPU 抗锯齿）。
/// PLAN-031：消息类型泛型化 `<M>`（rqhost daemon 复用接驳——现绑死
/// DesktopMessage 的唯一消费面改由类型推断承接，零行为差）。
struct DrawListPainter<M> {
    list: DrawList,
    _message: std::marker::PhantomData<fn() -> M>,
}

// ---------------------------------------------------------------------------
// PLAN-028 图像通道 —— DrawOp::Image 宿主侧解析/缓存/降级（D1–D4 定案）。
// ---------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

type ImageHandle = iced::widget::image::Handle;

/// src → Handle 进程级缓存（解码一次防每帧重上传；key = src——v1 无
/// radius/filter 维度，D1）。`None` = 已定失败负缓存（降级占位 + 观测
/// 去重的依据；http 负缓存与 `load_image_bytes` 双层独立）。
fn handle_cache() -> &'static Mutex<HashMap<String, Option<ImageHandle>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<ImageHandle>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 句柄缓存的插入序（FIFO 上限用；PLAN-062 复审 B5：Host 侧必须有界）。
fn handle_order() -> &'static Mutex<std::collections::VecDeque<String>> {
    static ORDER: OnceLock<Mutex<std::collections::VecDeque<String>>> = OnceLock::new();
    ORDER.get_or_init(|| Mutex::new(std::collections::VecDeque::new()))
}

/// 句柄缓存上限（超过即按插入序逐出；不承诺 LRU）。
/// 例外：`bitmap://` 由 producer 拥有，丢了无法自愈（Host 不会收到重传），
/// 因此不参与本地淘汰——它们只在 producer 侧驱逐时消失。
const HANDLE_CACHE_CAP: usize = 512;

/// 入句柄缓存并维持上限（FIFO；同 src 重入不重复排队）。
fn handle_cache_insert(src: &str, handle: Option<ImageHandle>) -> Option<Option<ImageHandle>> {
    let previous = handle_cache()
        .lock()
        .unwrap()
        .insert(src.to_string(), handle);
    {
        let mut order = handle_order().lock().unwrap();
        if previous.is_none() && !order.iter().any(|k| k == src) {
            order.push_back(src.to_string());
        }
        while order.len() > HANDLE_CACHE_CAP {
            let Some(oldest) = order.pop_front() else {
                break;
            };
            // producer 拥有的位图句柄不可自愈：保留（容量由 producer 的
            // 256 MiB 驻留预算约束，Host 侧只做上限提示）。
            if oldest.starts_with("bitmap://") {
                continue;
            }
            handle_cache().lock().unwrap().remove(&oldest);
        }
    }
    previous
}

/// http(s) 在途集：后台解码去重（同 src 至多一线程；失败由负缓存止位，
/// 不重试风暴）。
fn http_inflight() -> &'static Mutex<HashSet<String>> {
    static INFLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 未解析观测去重集（D4：同 src 首败打一行，后续静默——防每帧刷行）。
fn observed_unresolved() -> &'static Mutex<HashSet<String>> {
    static OBSERVED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    OBSERVED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 观测双落（remote.rs enable_remote_ws 先例同款：stderr + ui_console）。
fn observe(line: &str) {
    let full = format!("[drawlist-image] {line}");
    eprintln!("{full}");
    auto_lang_core::vm::ui_console::ui_console_push(&full);
}

fn observe_unresolved(src: &str) {
    let fresh = observed_unresolved()
        .lock()
        .unwrap()
        .insert(src.to_string());
    if fresh {
        observe(&format!("unresolved src (placeholder fallback): {src}"));
    }
}

/// 编码字节 → Handle（`from_bytes`：解码/上传在 iced 渲染器侧按 Handle
/// 身份去重——child 免解码的"轻 child"哲学在宿主侧的镜像）。
fn decode_handle(bytes: &[u8]) -> Option<ImageHandle> {
    if bytes.is_empty() {
        return None;
    }
    Some(ImageHandle::from_bytes(bytes.to_vec()))
}

/// PLAN-034 T-05（D3）：位图入缓存（宿主三消费面共用——rqhost
/// apply_actions / session broker_apply_actions / host ProtocolHost 的
/// BitmapReady 处理点）。同 id 重上传 = 即时翻新（insert 覆盖——无版本
/// 号，单写者时序由 BitmapAck 槽纪律钉死）。stride 宽于 w×4 时行重排
/// （`Handle::from_rgba` 要紧排像素）。
pub(crate) fn bitmap_cache_put(src: &str, w: u32, h: u32, stride: u32, rgba: Vec<u8>) {
    // 注：`src` 已含内容哈希与单调序号，同 src 必然同内容同代号，本地做不了
    // 有意义的 generation 比较；跨进程 generation 校验需要 wire 上的显式
    // 字段（Spec §6 具名遗留）。
    let tight = (w as usize) * 4;
    let pixels = if stride as usize == tight {
        // 紧排直用（stride == w×4 主路径）。
        rgba
    } else {
        // 行重排：每行取前 w×4，落紧排缓冲（短行按现状截断——欠长
        // 由下方尺寸失配守卫兜底）。
        let mut packed = Vec::with_capacity(tight * h as usize);
        for row in 0..h as usize {
            let base = row * stride as usize;
            if base >= rgba.len() {
                break;
            }
            let end = (base + tight).min(rgba.len());
            packed.extend_from_slice(&rgba[base..end]);
        }
        packed
    };
    let handle = if pixels.len() == tight * h as usize && w > 0 && h > 0 {
        Some(ImageHandle::from_rgba(w, h, pixels))
    } else {
        observe(&format!(
            "bitmap 尺寸失配（弃置）: {src} w={w} h={h} stride={stride} len={}",
            pixels.len()
        ));
        None
    };
    handle_cache_insert(src, handle);
}

/// 位图键逐出（宿主 ReclaimWindow/断连清 client 位图——防 pid 复用串扰）。
pub(crate) fn bitmap_cache_evict(src: &str) {
    handle_cache().lock().unwrap().remove(src);
}

/// PLAN-062（AC-03）：每个资源 id 最后已应用的 generation（只有协商
/// [`CAP_RESOURCE_GENERATION`] 的连线才会写入——旧端零成本）。
fn bitmap_generations() -> &'static std::sync::Mutex<std::collections::HashMap<String, u64>> {
    static GENS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, u64>>> =
        std::sync::OnceLock::new();
    GENS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// 带显式 generation 的位图入缓存（`BitmapReadyGen` 三消费面共用）。
///
/// 语义：同一 `src` 只接受**严格更新**的序号；相等（重复投递）或更小
/// （迟到/乱序）一律丢弃并保留已应用内容，同时留具名观测。返回是否应用。
/// 槽归还由调用方无条件执行（丢弃也要 `BitmapAck`，否则产者槽永久占用）。
pub(crate) fn bitmap_cache_put_gen(
    src: &str,
    generation: u64,
    w: u32,
    h: u32,
    stride: u32,
    rgba: Vec<u8>,
) -> bool {
    let applied = {
        let mut gens = bitmap_generations().lock().unwrap();
        match gens.get(src) {
            Some(last) if generation <= *last => {
                let last = *last;
                observe(&format!(
                    "位图迟到上传丢弃: {src} gen={generation} ≤ 已应用 {last}（保旧内容）"
                ));
                return false;
            }
            _ => {
                gens.insert(src.to_string(), generation);
                true
            }
        }
    };
    if applied {
        bitmap_cache_put(src, w, h, stride, rgba);
    }
    applied
}

/// DrawOp::Image src 解析总入口（D2 词汇表；paint 路径调用——禁阻塞）。
/// pub(crate)：p028_image_arm e2e 宿主侧解析/降级腿与度量直接驱动。
/// PLAN-029 T-07（D5）：签名扩 `(w, h)`——lucide: 栅格化目标尺寸/缓存
/// 键维度（`{src}@{w}x{h}`）；thumbnail/本地/http 词汇忽略尺寸（快照
/// 原始尺寸/内在尺寸语义不变）。
pub(crate) fn resolve_drawlist_image(src: &str, w: u32, h: u32) -> Option<ImageHandle> {
    // thumbnail://{wid}[!{fallback}] 虚拟引用（D3 + 029 T-05 fallback
    // 语法）：快照缓存直查，不进永久 Handle 缓存——SWR 刷新语义，冻结
    // 句柄会锁死旧图（WindowThumbnail 消费臂 from_rgba 每帧重建同律）；
    // miss → fallback lucide 占位图标真渲（灰 quad 降级升级）。
    if let Some(rest) = src.strip_prefix("thumbnail://") {
        return resolve_thumbnail(src, rest, w, h);
    }
    // workspace://{ws}[!{fallback}]（029 T-05 D4-A）：宿主合成虚拟引用
    //——壁纸基色 + 分区窗 tile 等比 Contain（012 W3 数据面直用）。
    if let Some(rest) = src.strip_prefix("workspace://") {
        return resolve_workspace(src, rest, w, h);
    }
    // lucide:{name}[#{rrggbb}]（029 T-07 D5）：字形栅格化真渲（ink 缺省
    // #FFFFFF——深色壳面约定；tint 可选）。缓存键含尺寸。
    if let Some(rest) = src.strip_prefix("lucide:") {
        return resolve_lucide(src, rest, w, h);
    }
    // bitmap://{id}（PLAN-034 D3：P028-D1 兑现）：app 上传位图直查——
    // 宿主 BitmapReady 处理点翻新（`bitmap_cache_put`）；miss 不落负
    // 缓存（位图可能后于首帧到达，负缓存会 pin 死后到位图——与本地族
    // 语义的差异点）+ 观测去重。
    if src.strip_prefix("bitmap://").is_some() {
        let hit = handle_cache().lock().unwrap().get(src).cloned().flatten();
        if hit.is_none() {
            observe_unresolved(src);
        }
        return hit;
    }
    if let Some(entry) = handle_cache().lock().unwrap().get(src) {
        return entry.clone();
    }
    if src.starts_with("http://") || src.starts_with("https://") {
        // D1：占位先行——paint 路径禁阻塞（reqwest 3s 同步会把渲染线程
        // 挂满 3s，os-007 P534-D4 教训）；后台线程复用 load_image_bytes
        //（自带进程缓存含负缓存）→ 落缓存 → 翻真由宿主任一后续重绘兑现
        //（鼠标/toast tick/MCP 心跳/帧泵驱动，零新增触发器）。
        if http_inflight().lock().unwrap().insert(src.to_string()) {
            observe(&format!("http fetch started: {src}"));
            let owned = src.to_string();
            std::thread::spawn(move || {
                let handle = crate::ui::iced::renderer::load_image_bytes(&owned)
                    .and_then(|bytes| decode_handle(&bytes));
                let hit = handle.is_some();
                handle_cache_insert(&owned, handle);
                http_inflight().lock().unwrap().remove(&owned);
                if hit {
                    observe(&format!("http image ready: {owned}"));
                }
            });
        }
        return None;
    }
    // 本地词汇（文件 / builtin: / data: / media 票据）：同步快路径
    //（本地字节，无网络等待）。
    let handle =
        crate::ui::iced::renderer::load_image_bytes(src).and_then(|bytes| decode_handle(&bytes));
    if handle.is_none() {
        observe_unresolved(src);
    }
    handle_cache_insert(src, handle.clone());
    handle
}

/// `thumbnail://{wid}[!{fallback}]` 解析（D3 + 029 T-05）：命中（含
/// 过期）→ `from_rgba` 直绘；过期 → request_capture 静默重抓（SWR）；
/// 真 miss → request_capture + fallback lucide 占位图标（029 升级：灰
/// quad → 真图标）。wid 非法 = 未解析降级。
fn resolve_thumbnail(src: &str, rest: &str, w: u32, h: u32) -> Option<ImageHandle> {
    use crate::ui::iced::snapshot;
    use crate::ui::session::Wid;
    // fallback 语法拆分（`!` 后为 lucide 名）。
    let (wid_part, fallback) = match rest.split_once('!') {
        Some((wid, fb)) => (wid, Some(fb.to_string())),
        None => (rest, None),
    };
    let Some(wid) = wid_part.parse::<u64>().ok().map(Wid) else {
        observe_unresolved(src);
        return None;
    };
    match snapshot::snapshot_window_stale(wid) {
        Some((snap, fresh)) => {
            if !fresh {
                snapshot::request_capture(wid);
            }
            Some(ImageHandle::from_rgba(snap.w, snap.h, snap.rgba))
        }
        None => {
            snapshot::request_capture(wid);
            // 029 T-05：miss → fallback 图标真渲（I3 降级升级；中性灰
            // tint——缩略图占位与前景皆宜）。
            if let Some(fb) = fallback {
                return resolve_lucide(src, &format!("{fb}#9aa0a6"), w, h);
            }
            observe_unresolved(&format!("{src} (capture requested)"));
            None
        }
    }
}

/// `workspace://{ws}[!{fallback}]` 解析（029 T-05 D4-A）：
/// `workspace_preview::current()` 数据面合成——壁纸基色铺底 + 分区 tile
/// 等比 Contain（`tile_rect` 纯函数复用）+ tile 内 snapshot 命中真缩略/
/// miss 灰块；Published 缺席/分区空 → fallback 图标。逐帧合成不进永久
/// 缓存（thumbnail 同纪律——SWR 活语义）。
fn resolve_workspace(src: &str, rest: &str, w: u32, h: u32) -> Option<ImageHandle> {
    let (ws, fallback) = match rest.split_once('!') {
        Some((ws, fb)) => (ws, Some(fb.to_string())),
        None => (rest, None),
    };
    let fallback_or_none = |why: &str| -> Option<ImageHandle> {
        if let Some(fb) = &fallback {
            return resolve_lucide(src, &format!("{fb}#9aa0a6"), w, h);
        }
        observe_unresolved(&format!("{src} ({why})"));
        None
    };
    let Some(published) = crate::ui::iced::workspace_preview::current() else {
        return fallback_or_none("no preview published");
    };
    let Some(tiles) = published.workspaces.get(ws) else {
        return fallback_or_none("workspace absent");
    };
    let w = w.max(1);
    let h = h.max(1);
    let mut pm = tiny_skia::Pixmap::new(w, h)?;
    // 壁纸基色铺底（None = 中性深底占位——012 W3 口径）。
    let (br, bg, bb) = published.wallpaper.unwrap_or((24, 28, 38));
    pm.fill(tiny_skia::Color::from_rgba8(br, bg, bb, 255));
    for tile in tiles {
        let (tx, ty, tw, th) = crate::ui::iced::workspace_preview::tile_rect(
            tile,
            published.usable,
            w as f32,
            h as f32,
        );
        if tw <= 0.0 || th <= 0.0 {
            continue;
        }
        let snap =
            crate::ui::iced::snapshot::snapshot_window_stale(crate::ui::session::Wid(tile.wid))
                .map(|(s, _)| s);
        blit_rgba_rect(
            pm.data_mut(),
            snap.as_ref().map(|s| (s.rgba.as_slice(), s.w, s.h)),
            tx,
            ty,
            tw,
            th,
        );
    }
    Some(ImageHandle::from_rgba(w, h, pm.take()))
}

/// RGBA 区块近邻缩放平贴进 Pixmap 矩形（workspace tile；无快照 = 灰块）。
fn blit_rgba_rect(pm: &mut [u8], snap: Option<(&[u8], u32, u32)>, x: f32, y: f32, w: f32, h: f32) {
    for row in 0..h.max(1.0) as u32 {
        for col in 0..w.max(1.0) as u32 {
            let px = x as u32 + col;
            let py = y as u32 + row;
            if px as f32 >= w + x || py as f32 >= h + y {
                continue;
            }
            let idx = ((py * w.max(1.0) as u32 + px) * 4) as usize;
            if idx + 3 >= pm.len() {
                return;
            }
            let (r, g, b) = match snap {
                Some((rgba, sw, sh)) if sw > 0 && sh > 0 => {
                    let sx = (((col as f32 + 0.5) / w) * (sw as f32)) as u32 % sw;
                    let sy = (((row as f32 + 0.5) / h) * (sh as f32)) as u32 % sh;
                    let sidx = ((sy * sw + sx) * 4) as usize;
                    if sidx + 2 >= rgba.len() {
                        (70, 74, 84)
                    } else {
                        (rgba[sidx], rgba[sidx + 1], rgba[sidx + 2])
                    }
                }
                _ => (70, 74, 84),
            };
            pm[idx] = r;
            pm[idx + 1] = g;
            pm[idx + 2] = b;
            pm[idx + 3] = 255;
        }
    }
}

/// `lucide:{name}[#{rrggbb}]` 解析（029 T-07 D5）：`lucide_svg_doc_with`
///（stroke 按 size 推导——直挂臂先例 ≥48px→1.5 否则 2.0）→ currentColor
/// 文档内替换 tint（ink 缺省 #FFFFFF）→ resvg/tiny-skia 栅格化目标尺寸
///（Contain 居中）→ RGBA Handle。缓存键 `{src}@{w}x{h}`（进程级含负缓
/// 存）；未知名/栅格化失败 → observe_unresolved + None（占位 + I3）。
fn resolve_lucide(src: &str, rest: &str, w: u32, h: u32) -> Option<ImageHandle> {
    let (name, tint) = match rest.split_once('#') {
        Some((n, c)) => (n, format!("#{c}")),
        None => (rest, "#ffffff".to_string()),
    };
    let w = w.max(1);
    let h = h.max(1);
    let key = format!("{src}@{w}x{h}");
    if let Some(entry) = handle_cache().lock().unwrap().get(&key) {
        return entry.clone();
    }
    let stroke = if w.max(h) >= 48 { 1.5 } else { 2.0 };
    let built = crate::ui::iced::renderer::lucide_svg_doc_with(name, stroke)
        .map(|doc| doc.replace("currentColor", &tint));
    let handle = built.and_then(|doc| rasterize_svg_contain(&doc, w, h));
    if handle.is_none() {
        observe_unresolved(src);
    }
    handle_cache_insert(&key, handle.clone());
    handle
}

/// SVG 文档 → Contain 居中栅格化（resvg 0.45 + tiny-skia 0.11——
/// plan619 测试先例同机具）。
fn rasterize_svg_contain(doc: &str, w: u32, h: u32) -> Option<ImageHandle> {
    let tree = resvg::usvg::Tree::from_str(doc, &resvg::usvg::Options::default()).ok()?;
    let mut pm = tiny_skia::Pixmap::new(w, h)?;
    let ts = tree.size();
    if ts.width() <= 0.0 || ts.height() <= 0.0 {
        return None;
    }
    let scale = (w as f32 / ts.width()).min(h as f32 / ts.height());
    let tx = (w as f32 - ts.width() * scale) / 2.0;
    let ty = (h as f32 - ts.height() * scale) / 2.0;
    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);
    resvg::render(&tree, transform, &mut pm.as_mut());
    Some(ImageHandle::from_rgba(w, h, pm.take()))
}

impl<M> iced::widget::canvas::Program<M> for DrawListPainter<M> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        paint_draw_list(renderer, &self.list, bounds.size())
    }
}

/// Plan 515 G1 —— scissor 栈栅格化：`Scissor` 起一段 `with_clip`（匹配
/// pop 之间的 op 裁剪到矩形内；嵌套 push 自然取交——draft/paste 的组合
/// 裁剪语义）。空栈 pop / 未闭合 push（编码端违约）宽容不炸：pop =
/// no-op，未闭合 = 裁到序列尾。
/// PLAN-683 T-00：泛型化 canvas 后端（实机 = `iced::Renderer`；headless
/// 单测 = `()` debug 断言后端——同一降格路径的零 GPU 回放验证）+ pub(crate)。
pub(crate) fn paint_ops<R>(frame: &mut iced::widget::canvas::Frame<R>, ops: &[DrawOp])
where
    R: iced::advanced::graphics::geometry::Renderer,
{
    use iced::widget::canvas::Text;
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DrawOp::Scissor { rect } => {
                // 深度扫描找配对 pop（含嵌套层）。
                let mut depth = 1usize;
                let mut end = ops.len();
                for (j, op) in ops.iter().enumerate().take(ops.len()).skip(i + 1) {
                    match op {
                        DrawOp::Scissor { .. } => depth += 1,
                        DrawOp::ScissorPop => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let region = iced::Rectangle::new(
                    iced::Point::new(rect.x, rect.y),
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                );
                frame.with_clip(region, |f| paint_ops(f, &ops[i + 1..end]));
                // 跳过配对 pop（未闭合时 end = ops.len()，循环自然收）。
                i = end + 1;
            }
            // 本层游离 pop（编码端违约）= no-op。
            DrawOp::ScissorPop => i += 1,
            DrawOp::Quad { rect, color } => {
                // widget 本地坐标 → canvas 原点平移（越界面出 canvas
                // 自动裁剪）。
                let at = iced::Point::new(rect.x, rect.y);
                frame.fill_rectangle(at, iced::Size::new(rect.w, rect.h), to_color(*color));
                i += 1;
            }
            DrawOp::QuadR {
                rect,
                color,
                radius,
            } => {
                // PLAN-679 Phase 2：圆角矩形（radius 编码端解析具值；
                // rounded-full = min(w,h)/2 亦在编码端解析；此处再钳半边
                // 防御）。r ≤ 0.5 退化直角。
                let radius = (*radius).min(rect.w.min(rect.h) / 2.0).max(0.0);
                if radius <= 0.5 {
                    let at = iced::Point::new(rect.x, rect.y);
                    frame.fill_rectangle(at, iced::Size::new(rect.w, rect.h), to_color(*color));
                } else {
                    let path = iced::widget::canvas::Path::rounded_rectangle(
                        iced::Point::new(rect.x, rect.y),
                        iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                        iced::border::Radius::from(radius),
                    );
                    frame.fill(&path, to_color(*color));
                }
                i += 1;
            }
            DrawOp::Text {
                x,
                y,
                size,
                line_height,
                color,
                text,
            } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute((*line_height).into()),
                    ..Default::default()
                });
                i += 1;
            }
            // Plan 515 G2 —— typography 差分：weight/style 映射 iced Font
            //（宿主字体栈按 face 选择——cosmic-text 家族回退取最接近档）。
            DrawOp::TextStyled {
                x,
                y,
                size,
                line_height,
                color,
                weight,
                italic,
                text,
            } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute((*line_height).into()),
                    font: wire_font(*weight, *italic),
                    ..Default::default()
                });
                i += 1;
            }
            // PLAN-028 图像通道（D1–D4）：src 引用 → 宿主侧解析（词汇 +
            // 进程级缓存 + thumbnail:// 虚拟引用）→ `Frame::draw_image`
            // 仓内首用；未解析降级 = 占位 Quad + 观测行（I3 禁静默错绘）。
            DrawOp::Image { rect, src, .. } => {
                let at = iced::Point::new(rect.x, rect.y);
                let size = iced::Size::new(rect.w.max(0.0), rect.h.max(0.0));
                match resolve_drawlist_image(src, rect.w.max(1.0) as u32, rect.h.max(1.0) as u32) {
                    Some(handle) => frame.draw_image(iced::Rectangle::new(at, size), &handle),
                    None => frame.fill_rectangle(
                        at,
                        size,
                        to_color(crate::ui::desktop_protocol::client_runtime::image_placeholder()),
                    ),
                }
                i += 1;
            }
        }
    }
}

/// queue 臂内容：DrawList → canvas 元素（Fill×Fill 客户区）。
pub fn drawlist_element<'a, M: 'a>(list: &DrawList) -> iced::Element<'a, M> {
    iced::widget::canvas(DrawListPainter::<M> {
        list: list.clone(),
        _message: std::marker::PhantomData,
    })
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// PLAN-683（remote 模式）：DisplayList v2 画师——canvas 原生原语重放
//（渐变 stop/四角半径/border/transform 原生；shadow 无 blur 原语 =
// 偏移半透明垫层近似 P683-D4）。
// ---------------------------------------------------------------------------

use crate::ui::desktop_protocol::message::{
    BorderSpec, DisplayList, DisplayOp, Fill as WireFill, ShadowSpec,
};

struct DisplayListPainter<M> {
    list: DisplayList,
    _message: std::marker::PhantomData<fn() -> M>,
}

/// remote 臂内容：DisplayList v2 → canvas 元素（Fill×Fill 客户区）。
pub fn displaylist_element<'a, M: 'a>(list: &DisplayList) -> iced::Element<'a, M> {
    iced::widget::canvas(DisplayListPainter::<M> {
        list: list.clone(),
        _message: std::marker::PhantomData,
    })
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

impl<M> iced::widget::canvas::Program<M> for DisplayListPainter<M> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        paint_display_list(_renderer, &self.list, bounds.size())
    }
}

// WGPU Frame::paste appends clipped meshes before the parent's unflushed
// buffers. Keep each clip run in its own geometry so a later base fill cannot
// cover clipped content. Adjacent primitives still share a batch.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PaintScope { Payload, ClipPush, ClipPop, TransformPush, TransformPop }

fn clipped_paint_runs<T: Clone>(
    ops: &[T], kind: impl Fn(&T) -> PaintScope, close: impl Fn(&T) -> T,
) -> Vec<Vec<T>> {
    let mut scopes: Vec<T> = Vec::new();
    let mut prefix = Vec::new();
    let mut run = Vec::new();
    let mut runs = Vec::new();
    let flush = |prefix: &[T], run: &mut Vec<T>, scopes: &[T], runs: &mut Vec<Vec<T>>| {
        if run.iter().any(|op| kind(op) == PaintScope::Payload) {
            let mut batch = prefix.to_vec();
            batch.append(run);
            batch.extend(scopes.iter().rev().map(&close));
            runs.push(batch);
        } else { run.clear(); }
    };
    for op in ops {
        match kind(op) {
            PaintScope::ClipPush | PaintScope::ClipPop => {
                flush(&prefix, &mut run, &scopes, &mut runs);
                if kind(op) == PaintScope::ClipPush { scopes.push(op.clone()); }
                else if let Some(i) = scopes.iter().rposition(|s| kind(s) == PaintScope::ClipPush) { scopes.remove(i); }
                prefix = scopes.clone();
            }
            PaintScope::TransformPush => { scopes.push(op.clone()); run.push(op.clone()); }
            PaintScope::TransformPop => {
                if let Some(i) = scopes.iter().rposition(|s| kind(s) == PaintScope::TransformPush) { scopes.remove(i); }
                run.push(op.clone());
            }
            PaintScope::Payload => run.push(op.clone()),
        }
    }
    flush(&prefix, &mut run, &scopes, &mut runs);
    runs
}

fn ordered_geometries<R, T>(
    renderer: &R, clear: Option<Rgba8>, size: iced::Size, runs: Vec<Vec<T>>,
    paint: impl Fn(&mut iced::widget::canvas::Frame<R>, &[T]),
) -> Vec<R::Geometry>
where R: iced::advanced::graphics::geometry::Renderer {
    let mut geometries = Vec::new();
    if let Some(clear) = clear {
        let mut frame = iced::widget::canvas::Frame::<R>::new(renderer, size);
        frame.fill_rectangle(iced::Point::ORIGIN, size, to_color(clear));
        geometries.push(frame.into_geometry());
    }
    for run in runs {
        let mut frame = iced::widget::canvas::Frame::<R>::new(renderer, size);
        paint(&mut frame, &run);
        geometries.push(frame.into_geometry());
    }
    geometries
}

fn paint_draw_list<R>(renderer: &R, list: &DrawList, size: iced::Size) -> Vec<R::Geometry>
where R: iced::advanced::graphics::geometry::Renderer {
    let runs = clipped_paint_runs(&list.ops, |op| match op {
        DrawOp::Scissor { .. } => PaintScope::ClipPush,
        DrawOp::ScissorPop => PaintScope::ClipPop,
        _ => PaintScope::Payload,
    }, |_| DrawOp::ScissorPop);
    ordered_geometries(renderer, list.clear, size, runs, paint_ops::<R>)
}

fn paint_display_list<R>(renderer: &R, list: &DisplayList, size: iced::Size) -> Vec<R::Geometry>
where R: iced::advanced::graphics::geometry::Renderer {
    let runs = clipped_paint_runs(&list.ops, |op| match op {
        DisplayOp::Scissor { .. } => PaintScope::ClipPush,
        DisplayOp::ScissorPop => PaintScope::ClipPop,
        DisplayOp::Transform { .. } => PaintScope::TransformPush,
        DisplayOp::TransformPop => PaintScope::TransformPop,
        _ => PaintScope::Payload,
    }, |op| match op {
        DisplayOp::Transform { .. } => DisplayOp::TransformPop,
        _ => DisplayOp::ScissorPop,
    });
    ordered_geometries(renderer, list.clear, size, runs, paint_ops_v2::<R>)
}

/// v2 原语 canvas 重放（paint_ops 同构 + v2 原生面）：Quad 全参（渐变
/// angle→to_distance 绝对几何重建 + 四角半径 + border 内描边 + shadow
/// 垫层近似）；Transform 压出栈（轴对齐矩阵直落 translate+scale，剪切
/// 面观测省略——降格端只产轴对齐形态）；Scissor/文本/图像与 v1 同路。
pub(crate) fn paint_ops_v2<R>(frame: &mut iced::widget::canvas::Frame<R>, ops: &[DisplayOp])
where
    R: iced::advanced::graphics::geometry::Renderer,
{
    use iced::widget::canvas::{Fill, Path, Stroke, Text};
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DisplayOp::Scissor { rect } => {
                let mut depth = 1usize;
                let mut end = ops.len();
                for (j, op) in ops.iter().enumerate().take(ops.len()).skip(i + 1) {
                    match op {
                        DisplayOp::Scissor { .. } => depth += 1,
                        DisplayOp::ScissorPop => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let region = iced::Rectangle::new(
                    iced::Point::new(rect.x, rect.y),
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                );
                frame.with_clip(region, |f| paint_ops_v2(f, &ops[i + 1..end]));
                i = end + 1;
            }
            DisplayOp::ScissorPop => i += 1,
            DisplayOp::Transform { matrix } => {
                // 配对 pop 扫描（Scissor 同法；Transform 栈内可含 Scissor
                //——按种类独立计数）。
                let mut depth = 1usize;
                let mut end = ops.len();
                for (j, op) in ops.iter().enumerate().take(ops.len()).skip(i + 1) {
                    match op {
                        DisplayOp::Transform { .. } => depth += 1,
                        DisplayOp::TransformPop => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let [m00, m01, m10, m11, tx, ty] = *matrix;
                // PLAN-062 T-02：仿射分解为 translate ∘ rotate ∘ scale（canvas
                // 变换栈可精确表达该族）。剪切（含非正交 m01/m10）不可达，
                // 具名降级为纯平移并在走样前保留诊断。
                let scale_x = (m00 * m00 + m10 * m10).sqrt();
                let theta = m10.atan2(m00);
                let det = m00 * m11 - m01 * m10;
                let scale_y = if scale_x.abs() > f32::EPSILON {
                    det / scale_x
                } else {
                    1.0
                };
                let shear = (m01 + theta.sin() * scale_y).abs()
                    + (m10 - theta.sin() * scale_x).abs();
                frame.push_transform();
                frame.translate(iced::Vector::new(tx, ty));
                if theta.abs() > f32::EPSILON {
                    frame.rotate(iced::Radians(theta));
                }
                if (scale_x - 1.0).abs() > f32::EPSILON
                    || (scale_y - 1.0).abs() > f32::EPSILON
                {
                    frame.scale_nonuniform(iced::Vector::new(scale_x, scale_y));
                }
                if shear > 1e-3 {
                    eprintln!(
                        "[rq-warn] transform 含剪切不可达（residual={shear:.4}）——按旋转+缩放降级"
                    );
                }
                paint_ops_v2(frame, &ops[i + 1..end]);
                frame.pop_transform();
                i = end + 1;
            }
            DisplayOp::TransformPop => i += 1,
            DisplayOp::Quad {
                rect,
                fill,
                radius,
                border,
                shadow,
            } => {
                let region = iced::Rectangle::new(
                    iced::Point::new(rect.x, rect.y),
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                );
                // shadow 垫层近似（canvas 无 blur 原语——偏移半透明圆角
                // 垫层承载投影主体，blur 缺席=登记债 P683-D4）。
                if let Some(shadow) = shadow {
                    paint_shadow_scrim(frame, region, *radius, shadow);
                }
                let radius = [
                    radius[0].min(region.width / 2.0).min(region.height / 2.0),
                    radius[1].min(region.width / 2.0).min(region.height / 2.0),
                    radius[2].min(region.width / 2.0).min(region.height / 2.0),
                    radius[3].min(region.width / 2.0).min(region.height / 2.0),
                ];
                let path = Path::rounded_rectangle(
                    region.position(),
                    region.size(),
                    iced::border::Radius {
                        top_left: radius[0],
                        top_right: radius[1],
                        bottom_right: radius[2],
                        bottom_left: radius[3],
                    },
                );
                let fill_style: Fill = match fill {
                    WireFill::Color(c) => to_color(*c).into(),
                    WireFill::LinearGradient { angle, stops } => {
                        let (start, end) = iced::Radians(*angle).to_distance(&region);
                        let mut linear = iced::widget::canvas::gradient::Linear::new(start, end);
                        for (offset, color) in stops {
                            linear = linear.add_stop(*offset, to_color(*color));
                        }
                        linear.into()
                    }
                };
                frame.fill(&path, fill_style);
                // border 内描边（inset w/2——iced Border 语义近似）。
                if let Some(b) = border {
                    if b.width > 0.0 {
                        let inset = b.width / 2.0;
                        let border_region = iced::Rectangle::new(
                            iced::Point::new(region.x + inset, region.y + inset),
                            iced::Size::new(
                                (region.width - b.width).max(0.0),
                                (region.height - b.width).max(0.0),
                            ),
                        );
                        let br = [
                            (radius[0] - inset).max(0.0),
                            (radius[1] - inset).max(0.0),
                            (radius[2] - inset).max(0.0),
                            (radius[3] - inset).max(0.0),
                        ];
                        let border_path = Path::rounded_rectangle(
                            border_region.position(),
                            border_region.size(),
                            iced::border::Radius {
                                top_left: br[0],
                                top_right: br[1],
                                bottom_right: br[2],
                                bottom_left: br[3],
                            },
                        );
                        frame.stroke(
                            &border_path,
                            Stroke::default()
                                .with_color(to_color(b.color))
                                .with_width(b.width),
                        );
                    }
                }
                i += 1;
            }
            DisplayOp::TextStyled {
                x,
                y,
                size,
                line_height,
                color,
                weight,
                italic,
                text,
            } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute((*line_height).into()),
                    font: wire_font(*weight, *italic),
                    ..Default::default()
                });
                i += 1;
            }
            DisplayOp::Path {
                path,
                fill,
                even_odd,
                stroke,
            } => {
                if let Some(built) = canvas_path(path) {
                    let bounds = wire_path_bounds(path);
                    if let Some(fill) = fill {
                        let style: iced::widget::canvas::Style = match fill {
                            WireFill::Color(c) => to_color(*c).into(),
                            WireFill::LinearGradient { angle, stops } => {
                                let (start, end) = iced::Radians(*angle).to_distance(&bounds);
                                let mut linear =
                                    iced::widget::canvas::gradient::Linear::new(start, end);
                                for (offset, color) in stops {
                                    linear = linear.add_stop(*offset, to_color(*color));
                                }
                                iced::widget::canvas::Style::Gradient(
                                    iced::widget::canvas::Gradient::Linear(linear),
                                )
                            }
                        };
                        frame.fill(
                            &built,
                            iced::widget::canvas::Fill {
                                style,
                                rule: if *even_odd {
                                    iced::widget::canvas::fill::Rule::EvenOdd
                                } else {
                                    iced::widget::canvas::fill::Rule::NonZero
                                },
                            },
                        );
                    }
                    if let Some(s) = stroke {
                        let mut spec = iced::widget::canvas::Stroke::default()
                            .with_color(to_color(s.color))
                            .with_width(s.width);
                        spec.line_cap = match s.cap {
                            1 => iced::widget::canvas::LineCap::Round,
                            2 => iced::widget::canvas::LineCap::Square,
                            _ => iced::widget::canvas::LineCap::Butt,
                        };
                        spec.line_join = match s.join {
                            1 => iced::widget::canvas::LineJoin::Round,
                            2 => iced::widget::canvas::LineJoin::Bevel,
                            _ => iced::widget::canvas::LineJoin::Miter,
                        };
                        frame.stroke(&built, spec);
                    }
                }
                i += 1;
            }
            DisplayOp::GlyphRun {
                font_hash,
                font_size,
                weight,
                flags,
                color,
                line_offset,
                glyphs,
            } => {
                paint_glyph_run(
                    frame,
                    font_hash,
                    *font_size,
                    *weight,
                    *flags,
                    *color,
                    *line_offset,
                    glyphs,
                );
                i += 1;
            }
            DisplayOp::Image { rect, src, .. } => {
                let at = iced::Point::new(rect.x, rect.y);
                let size = iced::Size::new(rect.w.max(0.0), rect.h.max(0.0));
                match resolve_drawlist_image(src, rect.w.max(1.0) as u32, rect.h.max(1.0) as u32) {
                    Some(handle) => {
                        frame.draw_image(iced::Rectangle::new(at, size), &handle);
                    }
                    None => {
                        frame.fill_rectangle(
                            at,
                            size,
                            to_color(
                                crate::ui::desktop_protocol::client_runtime::image_placeholder(),
                            ),
                        );
                    }
                }
                i += 1;
            }
        }
    }
}

/// v3 路径 → canvas 路径（verb 直落；坐标已是记录端烘焙后的设备坐标）。
/// 首个 `MoveTo` 之前的绘制段一律忽略（canvas Builder 要求先定位）。
fn canvas_path(
    path: &crate::ui::desktop_protocol::message::WirePath,
) -> Option<iced::widget::canvas::Path> {
    use crate::ui::desktop_protocol::message::PathSeg;
    use iced::widget::canvas::path::Builder;

    if path.segs.is_empty() {
        return None;
    }
    let mut builder = Builder::new();
    let mut positioned = false;
    for seg in &path.segs {
        match seg {
            PathSeg::MoveTo(x, y) => {
                builder.move_to(iced::Point::new(*x, *y));
                positioned = true;
            }
            PathSeg::LineTo(x, y) if positioned => {
                builder.line_to(iced::Point::new(*x, *y));
            }
            PathSeg::QuadTo(x1, y1, x, y) if positioned => {
                builder.quadratic_curve_to(
                    iced::Point::new(*x1, *y1),
                    iced::Point::new(*x, *y),
                );
            }
            PathSeg::CubicTo(x1, y1, x2, y2, x, y) if positioned => {
                builder.bezier_curve_to(
                    iced::Point::new(*x1, *y1),
                    iced::Point::new(*x2, *y2),
                    iced::Point::new(*x, *y),
                );
            }
            PathSeg::Close if positioned => builder.close(),
            _ => {}
        }
    }
    if !positioned {
        return None;
    }
    Some(builder.build())
}

/// 路径包围盒（渐变 fill 的 angle→绝对几何还原所需）。
fn wire_path_bounds(path: &crate::ui::desktop_protocol::message::WirePath) -> iced::Rectangle {
    use crate::ui::desktop_protocol::message::PathSeg;

    let (mut min_x, mut min_y) = (f32::INFINITY, f32::INFINITY);
    let (mut max_x, mut max_y) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut add = |x: f32, y: f32| {
        if x.is_finite() && y.is_finite() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    };
    for seg in &path.segs {
        match seg {
            PathSeg::MoveTo(x, y) | PathSeg::LineTo(x, y) => add(*x, *y),
            PathSeg::QuadTo(x1, y1, x, y) => {
                add(*x1, *y1);
                add(*x, *y);
            }
            PathSeg::CubicTo(x1, y1, x2, y2, x, y) => {
                add(*x1, *y1);
                add(*x2, *y2);
                add(*x, *y);
            }
            PathSeg::Close => {}
        }
    }
    if !min_x.is_finite() || !min_y.is_finite() {
        return iced::Rectangle::default();
    }
    iced::Rectangle::new(
        iced::Point::new(min_x, min_y),
        iced::Size::new((max_x - min_x).max(0.0), (max_y - min_y).max(0.0)),
    )
}

/// v3 字形 run 重放（PLAN-062 T-02）：按 wire 的字体内容身份与 CacheKey
/// 参数取回**与生产端同一张**字形位图，按 wire 位置合成——不做整形/布局。
///
/// 字形位图按 `CacheKey` 全局缓存（跨帧/跨 op 复用），着色以 RGBA 位图承载
/// （`Mask` 位图 = 颜色 × 覆盖率；emoji 的 `Color` 位图原样）。
#[allow(clippy::too_many_arguments)]
fn paint_glyph_run<R>(
    frame: &mut iced::widget::canvas::Frame<R>,
    font_hash: &[u8; 16],
    font_size: f32,
    weight: u16,
    flags: u32,
    color: Rgba8,
    line_offset: f32,
    glyphs: &[crate::ui::desktop_protocol::message::GlyphInstance],
) where
    R: iced::advanced::graphics::geometry::Renderer,
{
    use cosmic_text::CacheKey;

    let Some(font_id) = crate::ui::desktop_protocol::glyph_fonts::font_id_for_hash(font_hash) else {
        eprintln!(
            "[rq-warn] glyph run 的字体内容身份未装载（{:02x?}）——该 run 不呈现并须上报",
            &font_hash[..4]
        );
        return;
    };
    let system = iced::advanced::graphics::text::font_system();
    let mut guard = system.write().expect("Write font system");
    let mut cache = glyph_cache().lock().unwrap();
    for glyph in glyphs {
        let (key, ix, iy) =
            CacheKey::new(
            font_id,
            glyph.glyph_id,
            font_size,
            (glyph.x, glyph.y),
            cosmic_text::fontdb::Weight(weight),
            cosmic_text::CacheKeyFlags::from_bits_truncate(flags),
        );
        let rgba = match cache.get(&key) {
            Some(cached) => cached.clone(),
            None => {
                let mut swash = cosmic_text::SwashCache::new();
                let Some(image) = swash.get_image_uncached(guard.raw(), key) else {
                    continue;
                };
                let Some(rgba) = glyph_rgba(&image, color) else {
                    continue;
                };
                let rgba = std::sync::Arc::new(rgba);
                glyph_cache_insert(&mut cache, key, rgba.clone());
                rgba
            }
        };
        let (w, h) = (rgba.width, rgba.height);
        if w == 0 || h == 0 {
            continue;
        }
        let x = (ix + rgba.left) as f32;
        let y = (iy - rgba.top) as f32 + line_offset;
        frame.draw_image(
            iced::Rectangle::new(iced::Point::new(x, y), iced::Size::new(w as f32, h as f32)),
            &rgba.handle,
        );
    }
}

/// 字形位图 + 放置（以 `CacheKey` 为键缓存）。
struct CachedGlyph {
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    pixels: Vec<u8>,
    /// iced 图像句柄按 CacheKey 复用（每次 draw 新建句柄会每字形每帧重建
    /// GPU 纹理，P-02 违约 + 8–13ms 级重放开销）。
    handle: iced::widget::image::Handle,
}

type GlyphCacheMap = std::collections::HashMap<cosmic_text::CacheKey, std::sync::Arc<CachedGlyph>>;

fn glyph_cache() -> &'static Mutex<GlyphCacheMap> {
    static CACHE: OnceLock<Mutex<GlyphCacheMap>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(GlyphCacheMap::new()))
}

/// 字形位图缓存上限（复审 B5：Host 侧缓存不得无界；超过即整体裁剪到
/// 上限的一半——字形重算比无界增长安全）。
const GLYPH_CACHE_CAP: usize = 4096;

fn glyph_cache_insert(
    cache: &mut GlyphCacheMap,
    key: cosmic_text::CacheKey,
    glyph: std::sync::Arc<CachedGlyph>,
) {
    if cache.len() >= GLYPH_CACHE_CAP {
        cache.clear();
    }
    cache.insert(key, glyph);
}

/// swash 位图 → straight RGBA（Mask = 颜色 × 覆盖率；Color = 原位图）。
fn glyph_rgba(image: &cosmic_text::SwashImage, color: Rgba8) -> Option<CachedGlyph> {
    let width = image.placement.width;
    let height = image.placement.height;
    if width == 0 || height == 0 {
        return None;
    }
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    match image.content {
        cosmic_text::SwashContent::Mask => {
            // 画布图像管线按**预乘**语义合成（与 tiny-skia 的 PixmapPaint 同
            // 律）：覆盖率乘进 RGB，否则半透明边缘会被当成不透明色加亮，
            // 表现为整段文字偏粗。
            for alpha in &image.data {
                let a = ((*alpha as u16 * color.a as u16) / 255) as u8;
                let premul = |c: u8| ((c as u16 * a as u16) / 255) as u8;
                pixels.extend_from_slice(&[premul(color.r), premul(color.g), premul(color.b), a]);
            }
        }
        cosmic_text::SwashContent::Color => {
            pixels.extend_from_slice(&image.data);
        }
        cosmic_text::SwashContent::SubpixelMask => return None,
    }
    let handle = iced::widget::image::Handle::from_rgba(width, height, pixels.clone());
    Some(CachedGlyph {
        width,
        height,
        left: image.placement.left,
        top: image.placement.top,
        pixels,
        handle,
    })
}

/// shadow 垫层（P683-D4 近似）：偏移圆角矩形，alpha 按模糊半径弱化
///（blur 越大越淡——单层垫层的光学折衷）。
fn paint_shadow_scrim<R>(
    frame: &mut iced::widget::canvas::Frame<R>,
    region: iced::Rectangle,
    radius: [f32; 4],
    shadow: &ShadowSpec,
) where
    R: iced::advanced::graphics::geometry::Renderer,
{
    use iced::widget::canvas::{Fill, Path};
    let grow = shadow.blur.min(8.0);
    let scrim = iced::Rectangle::new(
        iced::Point::new(
            region.x + shadow.offset.0 - grow,
            region.y + shadow.offset.1 - grow,
        ),
        iced::Size::new(region.width + grow * 2.0, region.height + grow * 2.0),
    );
    let path = Path::rounded_rectangle(
        scrim.position(),
        scrim.size(),
        iced::border::Radius {
            top_left: radius[0] + grow,
            top_right: radius[1] + grow,
            bottom_right: radius[2] + grow,
            bottom_left: radius[3] + grow,
        },
    );
    // 单层垫层：alpha 随 blur 稀释（无 blur = 原样 alpha）。
    let dilution = if shadow.blur > 0.0 { 0.55 } else { 1.0 };
    let mut color = to_color(shadow.color);
    color.a *= dilution;
    let _ = Fill::default();
    frame.fill(&path, color);
}

/// independent 臂内容：RGBA 前缓冲 → Image（`from_rgba` 直接纳 straight
/// 非预乘；预乘换算在 iced 渲染器内部，协议层不感知）。
pub fn pixels_element<'a, M: 'a>(surface: &PixelsSurface) -> iced::Element<'a, M> {
    let handle = iced::widget::image::Handle::from_rgba(surface.w, surface.h, surface.rgba.clone());
    iced::widget::image(handle)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// 虚拟窗 wid → broker 表面内容（非 broker 窗 = None，调用方走本地
/// dynamic_view 既有路径）。
pub fn broker_client_content(
    state: &DesktopSession,
    wid: Wid,
) -> Option<iced::Element<'_, DesktopMessage>> {
    let client = state.broker_clients.values().find(|c| c.wid == Some(wid))?;
    // 像素前缓冲优先（independent 臂），回退命令帧（queue 臂）。
    if let Some(px) = client.composed_pixels() {
        return Some(pixels_element(px));
    }
    // PLAN-683：v2 合成面优先（remote 臂——DisplayListPainter）。
    if let Some(v2) = client.composed_v2() {
        return Some(displaylist_element(v2));
    }
    client.composed().map(drawlist_element)
}

#[cfg(test)]
mod tests {
    #[test]
    fn p060_native_clip_runs_preserve_scope_and_batch_adjacent_primitives() {
        let quad = DisplayOp::Quad { rect: crate::ui::desktop_protocol::message::WRect::new(1.0,1.0,2.0,2.0), fill: WireFill::Color(Rgba8::new(255,0,0,255)), radius:[0.0;4], border:None, shadow:None };
        let clip = DisplayOp::Scissor { rect: crate::ui::desktop_protocol::message::WRect::new(0.0,0.0,10.0,10.0) };
        let ops = vec![DisplayOp::Transform { matrix:[1.0,0.0,0.0,1.0,5.0,7.0] }, quad.clone(), clip, quad.clone(),quad.clone(),DisplayOp::ScissorPop,quad,DisplayOp::TransformPop];
        let runs = clipped_paint_runs(&ops, |op| match op {
            DisplayOp::Scissor {..} => PaintScope::ClipPush, DisplayOp::ScissorPop => PaintScope::ClipPop,
            DisplayOp::Transform {..} => PaintScope::TransformPush, DisplayOp::TransformPop => PaintScope::TransformPop,
            _ => PaintScope::Payload,
        }, |op| if matches!(op,DisplayOp::Transform{..}) { DisplayOp::TransformPop } else { DisplayOp::ScissorPop });
        assert_eq!(runs.len(),3,"split only at clip boundaries, not each primitive");
        assert_eq!(runs[1].iter().filter(|op| matches!(op,DisplayOp::Quad{..})).count(),2);
        assert!(matches!(runs[1][0],DisplayOp::Transform{..}));
        assert!(matches!(runs[1][1],DisplayOp::Scissor{..}));
        assert!(matches!(runs[1][runs[1].len()-2],DisplayOp::ScissorPop));
        assert!(matches!(runs[1].last(),Some(DisplayOp::TransformPop)));
    }

    #[test]
    fn p060_native_clipped_quad_order_matches_display_list() {
        struct Recorder;
        impl iced::advanced::Renderer for Recorder {
            fn start_layer(&mut self, _: iced::Rectangle) {}
            fn end_layer(&mut self) {}
            fn start_transformation(&mut self, _: iced::Transformation) {}
            fn end_transformation(&mut self) {}
            fn fill_quad(&mut self, _: iced::advanced::renderer::Quad, _: impl Into<iced::Background>) {}
            fn reset(&mut self, _: iced::Rectangle) {}
            fn allocate_image(&mut self, _: &iced::advanced::image::Handle,
                _: impl FnOnce(Result<iced::advanced::image::Allocation, iced::advanced::image::Error>) + Send + 'static) {
                panic!("image allocation outside this geometry case");
            }
        }
        impl iced::advanced::graphics::geometry::Renderer for Recorder {
            type Geometry = iced_wgpu::geometry::Geometry;
            type Frame = iced_wgpu::geometry::Frame;
            fn new_frame(&self, bounds: iced::Rectangle) -> Self::Frame { iced_wgpu::geometry::Frame::new(bounds) }
            fn draw_geometry(&mut self, _: Self::Geometry) {}
        }
        use crate::ui::desktop_protocol::message::{DisplayList, DisplayOp, Fill, Rgba8, WRect};
        let list = DisplayList { clear: Some(Rgba8::new(0,0,255,255)), ops: vec![
            DisplayOp::Scissor { rect: WRect::new(10.0,10.0,50.0,50.0) },
            DisplayOp::Quad { rect: WRect::new(20.0,20.0,10.0,10.0), fill: Fill::Color(Rgba8::new(255,0,0,255)), radius:[0.0;4], border:None, shadow:None },
            DisplayOp::ScissorPop,
        ] };
        let geometries = super::paint_display_list(&Recorder, &list, iced::Size::new(100.0,100.0));
        let mut clips=Vec::new();
        for geometry in geometries {
            let iced_wgpu::geometry::Geometry::Live { meshes, .. } = geometry else { panic!("expected live geometry"); };
            clips.extend(meshes.iter().map(|m| m.clip_bounds().width));
        }
        assert_eq!(clips, vec![100.0,50.0], "canvas must paint base before clipped bubble; later base currently covers bubble");
        let legacy = DrawList { clear: list.clear, ops: vec![
            DrawOp::Scissor { rect: WRect::new(10.0,10.0,50.0,50.0) },
            DrawOp::Quad { rect: WRect::new(20.0,20.0,10.0,10.0), color: Rgba8::new(255,0,0,255) },
            DrawOp::ScissorPop,
        ] };
        let geometries = super::paint_draw_list(&Recorder, &legacy, iced::Size::new(100.0,100.0));
        let mut legacy_clips = Vec::new();
        for geometry in geometries {
            let iced_wgpu::geometry::Geometry::Live { meshes, .. } = geometry else { panic!("expected live geometry"); };
            legacy_clips.extend(meshes.iter().map(|m| m.clip_bounds().width));
        }
        assert_eq!(legacy_clips, clips, "v1/v2 clip paint order differs");

    }

    use super::*;

    /// PLAN-102 F-RF-3：重放字体钉 Inter family——generic family + Medium
    /// 实机乱选字面（碎片化乱码）的回归护栏；字重/斜体映射同刻度。
    #[test]
    fn frf3_wire_font_pins_inter_family() {
        for w in [400u16, 500, 600, 700] {
            let f = wire_font(w, false);
            assert_eq!(
                f.family, INTER_FONT.family,
                "w={w} 须钉 app 字体栈 family（Inter）"
            );
        }
        assert_eq!(wire_font(500, false).weight, iced::font::Weight::Medium);
        assert_eq!(wire_font(700, false).weight, iced::font::Weight::Bold);
        assert_eq!(wire_font(400, true).style, iced::font::Style::Italic);
        assert_eq!(wire_font(400, false).style, iced::font::Style::Normal);
    }

    /// 1×1 PNG（合法编码字节——from_bytes 不即解码，宿主渲染器按 Handle
    /// 身份懒解码；单测断言解析/缓存/降级分派，真栅格归 e2e 截图腿）。
    const TINY_PNG_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    /// T-03：data: 词汇解析 → Handle；二次调用缓存命中（同 Handle 实例）。
    #[test]
    fn t028_data_uri_resolves_and_caches() {
        let h1 = resolve_drawlist_image(TINY_PNG_DATA_URI, 24, 24);
        assert!(h1.is_some(), "data: 解码命中");
        let h2 = resolve_drawlist_image(TINY_PNG_DATA_URI, 24, 24);
        assert_eq!(h1, h2, "二次调用 = 缓存命中（同 Handle）");
    }

    /// T-03：builtin: 内嵌壁纸词汇（与 518 壁纸线同源）。
    #[test]
    fn t028_builtin_wallpaper_resolves() {
        assert!(resolve_drawlist_image("builtin:ricepaper", 24, 24).is_some());
    }

    /// T-03/D4：未解析降级——缺文件/未知 scheme/字形词汇（lucide not-yet，
    /// P026-D1 后半维持）/空 src 全部 None + 负缓存落位（防每帧重读与
    /// 观测重刷）。
    #[test]
    fn t028_unresolved_negative_cache_and_notyet_lexicon() {
        let missing = "Z:/definitely/missing-028.png";
        assert!(
            resolve_drawlist_image(missing, 24, 24).is_none(),
            "缺文件 None"
        );
        assert!(
            handle_cache().lock().unwrap().contains_key(missing),
            "负缓存落位（占位 + 观测去重依据）"
        );
        assert!(
            resolve_drawlist_image("foo://bar", 24, 24).is_none(),
            "未知 scheme"
        );
        assert!(
            resolve_drawlist_image("lucide:definitely-not-a-real-icon-name", 24, 24).is_none(),
            "未知名降级 None"
        );
        assert!(resolve_drawlist_image("", 24, 24).is_none(), "空 src 容错");
    }

    /// T-03/D1：http miss 占位先行（不阻塞 paint）→ 后台解码落缓存。
    /// 载体 = 回环拒绝端口（快速失败，无网络依赖）。
    #[test]
    fn t028_http_placeholder_first_then_background_fill() {
        let src = "http://127.0.0.1:1/zero28.png";
        assert!(
            resolve_drawlist_image(src, 24, 24).is_none(),
            "首帧占位（无阻塞等待）"
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if handle_cache().lock().unwrap().contains_key(src) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "后台解码未落缓存");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(
            handle_cache().lock().unwrap().get(src).unwrap().is_none(),
            "连接拒绝 = 负缓存（不重试风暴）"
        );
    }

    /// T-04/D3：thumbnail:// 三路径——真 miss（占位 + request_capture
    /// 入队）/ 新鲜命中（直绘零重抓）/ SWR 过期（续绘 + 静默重抓）。
    #[test]
    fn t028_thumbnail_miss_hit_and_swr() {
        use crate::ui::iced::snapshot::{self, WindowSnapshot};
        use crate::ui::session::Wid;

        // 真 miss：当帧占位 + 抓取请求入队（宿主 update 排空）。
        let miss_wid = Wid(428_001);
        let _ = snapshot::take_capture_requests(); // 清场
        let src = format!("thumbnail://{}", miss_wid.0);
        assert!(
            resolve_drawlist_image(&src, 96, 64).is_none(),
            "miss 当帧占位"
        );
        assert!(
            snapshot::take_capture_requests()
                .iter()
                .any(|w| *w == miss_wid),
            "miss 触发 request_capture"
        );

        // 新鲜命中：缓存注入替身 → Handle 直出，零新增抓取请求。
        let hit_wid = Wid(428_002);
        let src = format!("thumbnail://{}", hit_wid.0);
        snapshot::cache_put(
            hit_wid,
            WindowSnapshot {
                rgba: vec![1, 2, 3, 255],
                w: 1,
                h: 1,
            },
        );
        let _ = snapshot::take_capture_requests(); // 清场
        assert!(resolve_drawlist_image(&src, 96, 64).is_some(), "命中直绘");
        assert!(
            snapshot::take_capture_requests().is_empty(),
            "新鲜命中零重抓"
        );

        // SWR：过期条目续绘（不跌占位）+ request_capture 静默重抓。
        let swr_wid = Wid(428_003);
        let src = format!("thumbnail://{}", swr_wid.0);
        snapshot::cache_put(
            swr_wid,
            WindowSnapshot {
                rgba: vec![9, 9, 9, 255],
                w: 1,
                h: 1,
            },
        );
        snapshot::__test_backdate(swr_wid);
        let _ = snapshot::take_capture_requests(); // 清场
        assert!(
            resolve_drawlist_image(&src, 96, 64).is_some(),
            "过期条目续绘"
        );
        assert!(
            snapshot::take_capture_requests()
                .iter()
                .any(|w| *w == swr_wid),
            "过期触发静默重抓"
        );

        // wid 非法 = 未解析降级。
        assert!(resolve_drawlist_image("thumbnail://not-a-wid", 24, 24).is_none());
    }

    /// PLAN-029 T-07（D5）：lucide: 词汇真渲——代表图标 ink 非零 +
    /// tint 着色 + 缓存命中零重栅格化 + 未知名降级。
    #[test]
    fn t029_lucide_vocabulary_rasterizes_with_tint_and_cache() {
        // ink 非零（缺省 #FFFFFF——全通道命中即非零）。
        let h = resolve_drawlist_image("lucide:search", 24, 24).expect("search 真渲");
        let (hw, hh, data) = handle_rgba(&h);
        assert_eq!((hw, hh), (24, 24));
        let ink = data.chunks(4).filter(|px| px[0] > 0).count();
        assert!(ink > 0, "ink 非零: {ink}");
        // tint：#ff0000 红——ink 像素 R 通道显著高于 B。
        let hr = resolve_drawlist_image("lucide:search#ff0000", 32, 32).expect("tint 真渲");
        let (_, _, rdata) = handle_rgba(&hr);
        assert!(
            rdata.chunks(4).any(|px| px[0] > 100 && px[0] > px[2]),
            "红 tint 命中"
        );
        // 缓存命中：同键二次解析（值等即可——内部零重栅格化由缓存键直证）。
        assert!(resolve_drawlist_image("lucide:search#ff0000", 32, 32).is_some());
        // 尺寸维度：不同尺寸 = 不同缓存键 = 各自可解析。
        assert!(resolve_drawlist_image("lucide:search", 64, 64).is_some());
        // 未知名 → None（负缓存 + 观测去重——I3）。
        assert!(
            resolve_drawlist_image("lucide:not-a-lucide-icon-xyz", 24, 24).is_none(),
            "未知名降级"
        );
        assert!(
            resolve_drawlist_image("lucide:not-a-lucide-icon-xyz", 24, 24).is_none(),
            "负缓存续 None"
        );
    }

    /// PLAN-029 T-05：thumbnail miss → fallback 图标真渲（灰 quad 升级）。
    #[test]
    fn t029_thumbnail_miss_falls_back_to_lucide_icon() {
        assert!(
            resolve_drawlist_image("thumbnail://999777!app-window", 48, 48).is_some(),
            "miss → lucide:app-window 占位图标"
        );
        // 无 fallback 的 miss 维持 None（028 口径不回归）。
        assert!(resolve_drawlist_image("thumbnail://999778", 48, 48).is_none());
    }

    /// PLAN-029 T-05（D4-A）：workspace:// 合成——Published 数据面铺底 +
    /// tile 灰块（无快照）+ 缺席 → fallback。
    #[test]
    fn t029_workspace_preview_composition() {
        use crate::ui::iced::workspace_preview::{publish, PreviewTile, Published};
        let mut p = Published::default();
        p.usable = (1920.0, 1040.0);
        p.wallpaper = Some((10, 20, 30));
        p.workspaces.insert(
            "0".into(),
            vec![PreviewTile {
                wid: 777001,
                x: 0.0,
                y: 0.0,
                w: 960.0,
                h: 520.0,
            }],
        );
        publish(p);
        let h =
            resolve_drawlist_image("workspace://0!app-window", 176, 64).expect("workspace 合成");
        let (w, hh, data) = handle_rgba(&h);
        assert_eq!((w, hh), (176, 64));
        // 壁纸基色铺底在场（非 tile 区像素 = 10,20,30）。
        assert!(
            data.chunks(4)
                .any(|px| px[0] == 10 && px[1] == 20 && px[2] == 30),
            "壁纸基色铺底"
        );
        // tile 灰块在场（无快照 → 70,74,84）。
        assert!(
            data.chunks(4)
                .any(|px| px[0] == 70 && px[1] == 74 && px[2] == 84),
            "tile 灰块占位"
        );
        // 分区缺席 → fallback 图标。
        assert!(
            resolve_drawlist_image("workspace://9!app-window", 64, 64).is_some(),
            "缺席分区 → fallback 图标"
        );
        // 清场（ Published 全局静态——防污染他测）。
        publish(Published::default());
    }

    /// PLAN-034 T-05（D3）：bitmap:// 前缀臂——入缓存真渲 / miss 占位
    /// 且**不落负缓存**（位图可后到）/ 同 id 重上传翻新 / evict 逐出。
    #[test]
    fn p034_bitmap_prefix_arm_put_miss_and_refresh() {
        let src = "bitmap://test-p034-anim";
        // miss：None 占位 + 无负缓存（后到语义）。
        assert!(resolve_drawlist_image(src, 48, 24).is_none(), "未上传 miss");
        assert!(
            !handle_cache().lock().unwrap().contains_key(src),
            "miss 不落负缓存（位图可后到）"
        );
        // 入缓存：紧排主路径。
        let (w, h) = (4u32, 2u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i % 251) as u8).collect();
        bitmap_cache_put(src, w, h, w * 4, rgba.clone());
        let hit1 = resolve_drawlist_image(src, 48, 24).expect("上传后命中");
        let (hw, hh, data1) = handle_rgba(&hit1);
        assert_eq!((hw, hh), (w, h), "Handle 尺寸");
        assert_eq!(data1, rgba, "像素直传（紧排）");
        // 同 id 重上传 = 即时翻新。
        let rgba2: Vec<u8> = (0..w * h * 4).map(|i| ((i + 7) % 251) as u8).collect();
        bitmap_cache_put(src, w, h, w * 4, rgba2.clone());
        let (_, _, data2) = handle_rgba(&resolve_drawlist_image(src, 48, 24).expect("翻新后命中"));
        assert_ne!(data1, data2, "重上传翻新");
        assert_eq!(data2, rgba2);
        // stride 宽于紧排：行重排（首行取前 w×4）。
        let src_wide = "bitmap://test-p034-wide";
        let stride = w * 4 + 16;
        let mut wide = vec![0u8; (stride * h) as usize];
        wide[..(w * 4) as usize].copy_from_slice(&rgba[..(w * 4) as usize]);
        wide[stride as usize..stride as usize + (w * 4) as usize]
            .copy_from_slice(&rgba[(w * 4) as usize..]);
        bitmap_cache_put(src_wide, w, h, stride, wide);
        let (_, _, data_w) =
            handle_rgba(&resolve_drawlist_image(src_wide, 48, 24).expect("宽行重排"));
        assert_eq!(data_w, rgba, "宽 stride 行重排为紧排");
        // evict：逐出后回 miss（仍无负缓存）。
        bitmap_cache_evict(src);
        assert!(resolve_drawlist_image(src, 48, 24).is_none(), "逐出后 miss");
        assert!(
            !handle_cache().lock().unwrap().contains_key(src),
            "逐出即清键"
        );
        // 尺寸失配守卫：载荷短于 h×w×4 → None Handle 入缓存（占位 +
        // 观测），不 panic。
        bitmap_cache_put("bitmap://test-p034-short", w, h, w * 4, vec![1u8; 4]);
        assert!(
            resolve_drawlist_image("bitmap://test-p034-short", 48, 24).is_none(),
            "失配守卫 = 占位"
        );
    }

    /// PLAN-062（AC-03）：显式 generation 的位图上传只接受**严格更新**的序号；
    /// 迟到/重复上传被丢弃并保旧内容；旧 tag（无序号）保持即时翻新语义。
    #[test]
    fn p062_bitmap_generation_rejects_late_and_duplicate_uploads() {
        let src = "bitmap://test-p062-gen";
        let (w, h) = (2u32, 2u32);
        let px = |seed: u8| vec![seed; (w * h * 4) as usize];

        assert!(
            bitmap_cache_put_gen(src, 5, w, h, w * 4, px(5)),
            "首次上传（gen=5）接受"
        );
        let (_, _, applied) = handle_rgba(&resolve_drawlist_image(src, 8, 8).expect("命中"));
        assert_eq!(applied, px(5));

        // 迟到（更小序号）：丢弃，内容保持 gen=5。
        assert!(
            !bitmap_cache_put_gen(src, 3, w, h, w * 4, px(3)),
            "迟到上传（gen=3）必须被拒"
        );
        let (_, _, kept) = handle_rgba(&resolve_drawlist_image(src, 8, 8).expect("仍命中"));
        assert_eq!(kept, px(5), "拒绝不得覆盖已应用内容");

        // 重复投递（相等序号）：同样丢弃（幂等重放不翻新）。
        assert!(
            !bitmap_cache_put_gen(src, 5, w, h, w * 4, px(9)),
            "重复上传（gen=5）必须被拒"
        );
        let (_, _, still) = handle_rgba(&resolve_drawlist_image(src, 8, 8).expect("仍命中"));
        assert_eq!(still, px(5));

        // 更新序号：接受并翻新。
        assert!(
            bitmap_cache_put_gen(src, 6, w, h, w * 4, px(6)),
            "更新序号（gen=6）接受"
        );
        let (_, _, fresh) = handle_rgba(&resolve_drawlist_image(src, 8, 8).expect("命中"));
        assert_eq!(fresh, px(6), "更新序号必须翻新内容");

        // 旧 tag（无序号）语义不变：直接覆盖（旧端/未协商连线）。
        bitmap_cache_put(src, w, h, w * 4, px(7));
        let (_, _, legacy) = handle_rgba(&resolve_drawlist_image(src, 8, 8).expect("命中"));
        assert_eq!(legacy, px(7), "旧 tag 保持即时翻新");
    }

    /// Handle → (w, h, rgba)（iced Handle 数据面读取——ink/tint 断言口）。
    fn handle_rgba(h: &ImageHandle) -> (u32, u32, Vec<u8>) {
        use iced::widget::image::Handle;
        match h {
            Handle::Rgba {
                width,
                height,
                pixels,
                ..
            } => (*width, *height, pixels.to_vec()),
            _ => panic!("期望 RGBA Handle"),
        }
    }
}
