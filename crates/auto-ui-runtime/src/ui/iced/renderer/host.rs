//! PLAN-014 T-06：宿主装配责任（startup 窗口配置、run_dynamic_*、run_session/update 闭包/view_desktop_fn/subscription 扇出/exit 机）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::devtools::*;
use super::media::*;
use super::projection::*;
use super::shell_state::*;
use super::style_widgets::*;
use super::subscriptions::*;

/// Plan 411: startup window size (logical px). Sources, in priority order:
/// 1. pac.at `window: "WxH"` — injected by `auto run` as AUTO_VM_WINDOW
/// 2. 1280x800 desktop default
/// Invalid env values fall back to the default rather than failing the app.
pub fn startup_window_size() -> iced::Size {
    let mut resolved = iced::Size::new(1280.0, 800.0);
    if let Ok(spec) = std::env::var("AUTO_VM_WINDOW") {
        if let Some((w, h)) = spec.trim().split_once(['x', 'X']) {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                if w >= 200.0 && h >= 200.0 && w <= 7680.0 && h <= 4320.0 {
                    resolved = iced::Size::new(w, h);
                }
            }
        } else if spec.trim().eq_ignore_ascii_case("fit") {
            // Plan 644 follow-up: fit mode defaults to compact card dimensions
            // before content measurement completes, avoiding large desktop window flash.
            resolved = iced::Size::new(480.0, 680.0);
        }
    }
    // PLAN-046-B (auto-musk T7): baseline height lands in session KV so .at
    // code (`platformViewportHeight`) reads a real value from frame one;
    // OS-actual corrections flow through both __window_resized funnels.
    auto_lang_core::vm::ffi::stdlib::storage_host_publish(
        "vm.window_inner_height",
        format!("{}", resolved.height),
    );
    resolved
}

/// Extract initial content size from root view's style when `window: "fit"` is declared.
pub fn extract_root_view_fit_size<M: Clone + std::fmt::Debug>(
    view: &AbstractView<M>,
) -> Option<iced::Size> {
    let style = match view {
        AbstractView::Column { style, .. }
        | AbstractView::Row { style, .. }
        | AbstractView::Container { style, .. } => style.as_ref(),
        _ => None,
    }?;
    let layout = crate::ui::style::BoxLayout::from_style(style);
    let w = match layout.width {
        Some(SizeValue::Pixels(px)) if px > 0.0 => Some(px),
        Some(SizeValue::Fixed(units)) if units > 0 => Some((units as f32) * 4.0),
        _ => None,
    }?;
    let h = match layout.height {
        Some(SizeValue::Pixels(px)) if px > 0.0 => px,
        Some(SizeValue::Fixed(units)) if units > 0 => (units as f32) * 4.0,
        _ => (w * 1.4).clamp(320.0, 720.0),
    };
    Some(iced::Size::new(w.max(200.0), h.max(200.0)))
}

/// Plan 504: startup window fit-to-content mode. Source:
/// pac.at `window: "fit"` — injected by `auto run` as AUTO_VM_WINDOW=fit.
/// When true, the first rendered frame is measured and the OS window is
/// resized to the content's natural size (see `__fit_measured` routing).
pub fn startup_window_fit() -> bool {
    std::env::var("AUTO_VM_WINDOW")
        .map(|v| v.trim().eq_ignore_ascii_case("fit"))
        .unwrap_or(false)
}

/// PLAN-097 解阻批次（auto-musk 演示 UX）:窗口 z-level 环境契约。canvas
/// 会话 spawn 的 AutoVM 原生窗是帧截图的渲染源(面板镜像 = 内嵌呈现面)，
/// 默认 Normal 会盖住宿主 Musk 窗——`AUTO_VM_WINDOW_LEVEL=bottom` 置
/// AlwaysOnBottom(永不遮宿主;iced::window::screenshot 走窗口自有
/// surface,遮挡不影响帧管线,仅最小化受制)。默认 Normal(独立运行形态
/// 不变)。
pub fn startup_window_level() -> iced::window::Level {
    match std::env::var("AUTO_VM_WINDOW_LEVEL").as_deref() {
        Ok("bottom") | Ok("Bottom") | Ok("BOTTOM") => iced::window::Level::AlwaysOnBottom,
        _ => iced::window::Level::Normal,
    }
}

/// PLAN-102（auto-musk canvas 内嵌）：boot 期窗口隐藏位。`AUTO_VM_HIDDEN=1`
/// = 应用窗不进 OS 桌面（Plan 500 像素臂 `Settings{visible:false}` 同语义
/// ——渲染/截图照常，MCP `autoui_screenshot` 即帧源），且不要求像素桥协议
/// 宿主（消费走既有 MCP 通道，零管线新增）。缺省零变化（独立运行形态
/// 不变）。运行期显隐切换（内嵌↔独立窗不重启）由宿主侧对目标窗
/// ShowWindow 承载（canvas-embed-probe P4 验证口径）。
pub fn startup_window_hidden() -> bool {
    std::env::var("AUTO_VM_HIDDEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "True"))
        .unwrap_or(false)
}

/// PLAN-097 演示 UX:窗口初始位置环境契约。`AUTO_VM_POSITION=x,y`(逻辑
/// 像素)→ `Position::Specific`——canvas 会话按宿主窗右侧确定性停靠,
/// 不遮宿主(缺省 = OS 选择,独立运行形态不变)。
pub fn startup_window_position() -> Option<iced::window::Position> {
    let spec = std::env::var("AUTO_VM_POSITION").ok()?;
    let (x, y) = spec.trim().split_once(',')?;
    let x: f32 = x.trim().parse().ok()?;
    let y: f32 = y.trim().parse().ok()?;
    Some(iced::window::Position::Specific(iced::Point::new(x, y)))
}

/// VM window title. Sources, in priority order:
/// 1. pac.at `title: "..."` — injected by `auto run` as AUTO_VM_TITLE
/// 2. fallback — "Auto - {root widget name}"
/// Blank env values fall back rather than producing an empty title.
pub fn window_title(fallback: String) -> String {
    match std::env::var("AUTO_VM_TITLE") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => fallback,
    }
}

/// Convert IcedFontSize to f32 pixel value
pub(super) fn font_size_to_f32(font_size: &crate::ui::style_iced_adapter::IcedFontSize) -> f32 {
    use crate::ui::style_iced_adapter::IcedFontSize;
    match font_size {
        IcedFontSize::Xs => 12.0,
        IcedFontSize::Sm => 14.0,
        IcedFontSize::Base => 16.0,
        IcedFontSize::Lg => 18.0,
        IcedFontSize::Xl => 20.0,
        IcedFontSize::Xxl => 24.0,
        IcedFontSize::X3xl => 30.0,
        IcedFontSize::X4xl => 36.0,
        IcedFontSize::X5xl => 48.0,
        IcedFontSize::X6xl => 60.0,
        IcedFontSize::X7xl => 72.0,
        IcedFontSize::X8xl => 96.0,
        IcedFontSize::X9xl => 128.0,
    }
}

/// Get effective font size in pixels, preferring arbitrary pixel value over named size.
pub(super) fn effective_font_size(iced_style: &IcedStyle) -> Option<f32> {
    iced_style
        .font_size_arbitrary
        .or_else(|| iced_style.font_size.as_ref().map(font_size_to_f32))
}

// ============================================================================
// Plan 227: Send-safe IcedMessage wrapper for DynamicComponent
// ============================================================================


/// Run a `DynamicComponent` in an iced window.
///
/// This is the main entry point for running AURA widgets with iced. It:
/// 1. Wraps the `DynamicComponent` in a `DesktopSession`（T4c 会话翻转，
///    R3 退化桌面；平铺读写经 split_mut/split_ref 拆借视图承接）
/// 2. Uses `iced::daemon()` — one OS window per App, per-window view routing
/// 3. Converts `View<DynamicMessage>` to `View<IcedMessage>` before rendering
/// 4. Maps iced messages back to `DynamicMessage` on update
///
/// # Arguments
///
/// * `component` - A ready-to-use `DynamicComponent`
///
/// # Returns
///
/// `AppResult<String>` - Ok("UI closed") on normal exit, Err on failure.
pub fn run_dynamic_iced(component: DynamicComponent) -> AppResult<String> {
    run_dynamic_iced_multi(vec![component])
}

/// Plan 459：多 App 多窗口入口 —— 一个 `DesktopSession` 服务 N 个 OS 窗口，
/// 每窗口渲染各自的 AppSession（iced::daemon，boot 期 `window::open` 逐 App
/// 开窗并同步登记）。全窗口关闭后进程退出（daemon 语义，`iced::exit`）。
pub fn run_dynamic_iced_multi(components: Vec<DynamicComponent>) -> AppResult<String> {
    run_session(components, RunMode::Standalone, DesktopOptions::default())
}

/// Plan 463：desktop 模式运行选项（I3 配置位族；独立模式忽略）。
#[derive(Debug, Clone, Default)]
pub struct DesktopOptions {
    /// T3：全屏无框桌面（`Settings{fullscreen, decorations:false}`）。
    pub fullscreen: bool,
    /// T7：应用注册表目录（扫描 `*/pac.at` → LaunchApp 目标）。None =
    /// 不装载注册表（LaunchApp 回 toast "registry unavailable"）。
    pub apps_dir: Option<std::path::PathBuf>,
    /// Stage B P-7：shell pack 目录显式覆盖（沿 `apps_dir` 先例；Design 01
    /// §4-P7）。None = 缺省解析序（env → 兄弟 → 主检出 → 内嵌 pin 快照）。
    pub shell_pack: Option<std::path::PathBuf>,
    /// Stage B P-3：extra 根显式覆盖（沿 `apps_dir` 先例；用户裁定
    /// 2026-09-07 仅 API 层不加 CLI flag）。None = 缺省探测
    /// （`app_registry::host_extra_roots`：storage + `../auto-os-config/auto`
    /// 单根 + `../auto-os/apps` 容器 + apps.manifest repo 条目）；Some =
    /// 全额替换（id 取路径末段，vue 轨 `AUTO_DESKTOP_APPS_EXTRA` 同语义）。
    pub extra_app_roots: Option<Vec<std::path::PathBuf>>,
    /// Plan 494：真洞模式（docked 原生窗口垫桌面下方 + SetWindowRgn 洞
    /// 排除）。boot 期与 `shell.native.hole` storage 键取或（472 dock 配置
    /// 同型）；运行时失败自动回退 off。
    pub hole_mode: bool,
    /// Plan 500 步骤 4：independent 臂像素桥——Standalone 单窗**隐藏化**
    /// （`Settings{visible:false}`）+ boot 取像素桥入 `session.pixels` +
    /// 协议轮询/截图回调订阅。false（缺省）= 既有 Standalone 行为零变化。
    pub pixels: bool,
    /// PLAN-698 T-04b/SD-04：桌面侧 rqhost 合成器 daemon 孵化（子进程
    /// 形态——`auto rqhost --pipe` 专进程，winit Windows 单事件循环约束
    /// 否决同进程库形态，勘定留痕见计划）。`Some(name)` = 管道名显式；
    /// `None` + env `AUTO_DESKTOP_RQHOST=1` = pid 派生名（多桌面实例
    /// 隔离）；两者缺席 = 不孵化（行为零变化）。
    pub rqhost_endpoint: Option<String>,
}

/// Plan 462 T5：desktop 模式入口 —— 单宿主 OS 窗口承载 N 个虚拟窗口
/// （Design 23 R2 拓扑；chrome/拖拽/缩放/关闭/聚焦见 `virtual_window.rs`）。
pub fn run_dynamic_desktop(components: Vec<DynamicComponent>) -> AppResult<String> {
    run_session(components, RunMode::Desktop, DesktopOptions::default())
}

/// Plan 472 T5：窗口模式带选项入口（注册表/dock 配置装配；与全屏入口
/// 同落到 `run_session` 单管线）。
pub fn run_dynamic_desktop_with_options(
    components: Vec<DynamicComponent>,
    opts: DesktopOptions,
) -> AppResult<String> {
    run_session(components, RunMode::Desktop, opts)
}

/// Plan 463 T3/T7：全屏桌面入口 —— borderless + Fullscreen + 可选注册表
/// （Esc 保留调试退出；`DesktopOptions.apps_dir` 装配 LaunchApp 注册表）。
pub fn run_dynamic_desktop_fullscreen(
    components: Vec<DynamicComponent>,
    opts: DesktopOptions,
) -> AppResult<String> {
    run_session(components, RunMode::Desktop, opts)
}

/// Plan 500 步骤 4：independent 臂 child 渲染宿主入口——Standalone 单窗
/// **隐藏**（`Settings{visible:false}`）+ 像素桥（协议轮询订阅 + 截图
/// 回调 → shm/FrameReadyPixels）。零新管线：复用 `run_session` 唯一
/// update/view/订阅管线（I3），仅 boot 开窗可见位与像素桥配置位分叉；
/// 既有 Standalone/desktop 行为不变（`opts.pixels=false` 缺省）。
pub fn run_dynamic_iced_pixels(component: DynamicComponent) -> AppResult<String> {
    run_session(
        vec![component],
        RunMode::Standalone,
        DesktopOptions {
            pixels: true,
            ..Default::default()
        },
    )
}

/// 会话运行形态（Plan 462，I3：唯一 update/view/订阅管线，仅此配置位与
/// boot 开窗方式分叉；禁止再增第二条 standalone/desktop 代码路径）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// R3 退化桌面：每 App 一个 OS 窗口（459 语义，`auto run` 现状）。
    Standalone,
    /// R2 虚拟桌面：单宿主 OS 窗口 + N 个 `VirtualWindow`（462）。
    Desktop,
}

fn run_session(
    mut components: Vec<DynamicComponent>,
    mode: RunMode,
    opts: DesktopOptions,
) -> AppResult<String> {
    // PLAN-815 T-05（F03，R2 根修）：新会话退出请求代数——本会话只响应
    // 快照之后的新请求（初始化期间到达的取消不丢、上一会话的陈旧请求
    // 不误杀；订阅经 host_exit_poll_subscription(since) 消费——since 在
    // iced 构建订阅时读取当前代数）。
    // R3-02：优先沿用执行边界已开启的窗口（初始化/构建期间的退出请求
    // 因此 > 会话起始代数而不被吸收）；无窗口（桌面直入口）则在此开启。
    host_exit_window_begin();
    // Stage B P-7：宿主显式 shell pack 注入（OnceLock 首值胜；后续幂等）。
    if let Some(pack_dir) = &opts.shell_pack {
        crate::ui::shell::set_shell_pack_override(pack_dir.clone());
    }
    // PLAN-698 T-04b/SD-04：桌面侧 rqhost 合成器 daemon 孵化（仅 Desktop
    // 模式；P693-D2 配对面）。形态勘定（2026-09-23 实机）：**子进程**——
    // 库形态（同进程专线程 run_daemon）被 winit/iced Windows 单事件循环
    // 约束否决（第二 iced 循环 = `RecreationAttempt` panic，实测留痕
    // scratch-w1/t04_desktop.log）；子进程复用既有孵化机制
    // （`auto rqhost --pipe <name>`，spawn_rqhost_default 同型）。管道
    // 命名裁定（T-04a，待澄清2 默认案）：**注册表发布 + pid 派生隔离**
    // ——发布=AUTO_DESKTOP_ENDPOINT env（桌面进程设置，子进程启动即继承，
    // 注册表 launch 面零改动）；pid 派生名=多桌面实例隔离；wellknown
    // 兜底=桌面外 `-q` 轨既有语义不变。生命周期注记：daemon 子进程按
    // 末窗自退语义存活（零窗常驻与 `-q` 轨一致）；宿主强杀不级联。
    if mode == RunMode::Desktop {
        let endpoint = opts.rqhost_endpoint.clone().or_else(|| {
            (std::env::var("AUTO_DESKTOP_RQHOST").as_deref() == Ok("1")).then(|| {
                format!(
                    "{}-desktop-{}",
                    crate::ui::desktop_protocol::rqhost::RQHOST_PIPE,
                    std::process::id()
                )
            })
        });
        if let Some(pipe) = endpoint {
            match crate::ui::desktop_protocol::rqhost::spawn_desktop_daemon(&pipe) {
                Ok(()) => {
                    std::env::set_var("AUTO_DESKTOP_ENDPOINT", &pipe);
                    eprintln!(
                        "[ui-desktop] rqhost daemon spawned at {pipe} (AUTO_DESKTOP_ENDPOINT published)"
                    );
                }
                Err(e) => eprintln!("[ui-desktop] rqhost daemon spawn failed: {e}"),
            }
        }
    }
    // PLAN-575 D1 挂点②：main 装配处（run_session 唯一管线，I3）装全局
    // panic 审计 hook——只追加日志，不改 panic 语义（G3 零行为变更）。
    auto_lang_core::vm::ffi::stdlib::install_exit_audit_panic_hook();
    // PLAN-765 T-03（纪要三 F-9）：Windows 原生崩溃审计钩（未处理 SEH
    // 异常兜底落笔，落笔后 CONTINUE_SEARCH 保 WER 语义——覆盖边界见
    // stdlib 文档）。
    #[cfg(windows)]
    auto_lang_core::vm::ffi::stdlib::install_native_crash_audit();
    // PLAN-765 r2 T-05 诊断：ExitProcess IAT 追踪钩（env 门控）——四审计面
    // +WER 全零痕的静默码 1 死亡复现后，唯一未覆盖进程内路径=依赖直调
    // ExitProcess。见 stdlib::install_exit_process_trace 文档。
    #[cfg(windows)]
    if std::env::var("P765_EXIT_TRACE").map_or(false, |v| v == "1") {
        auto_lang_core::vm::ffi::stdlib::install_exit_process_trace();
    }
    // PLAN-708 T-02（S-02）：生成器默认注册表后台预热——首帧带 preview-card
    // 的页面渲染原本内联承担 OnceLock 首建全量（register_defaults + schema
    // 解析折叠，实测 ~1.27s 单块，T-00 baseline §3.2），违反加载期连续占用
    // 门禁。run_session 启动即后台触碰 `WidgetRegistry::with_defaults()`：
    // UI 线程到达首个 preview-card 时 OnceLock 已热（clone ~µs）；竞争态下
    // 最坏退化为现状一次性内联（OnceLock 单胜利者，语义等价）。纯预热无
    // 共享写面——线程安全由 OnceLock + 纯数据 Clone 保证。
    std::thread::Builder::new()
        .name("autoui-registry-warm".to_string())
        .spawn(|| {
            let _ = auto_ui_compiler::ui_gen::widget::WidgetRegistry::with_defaults();
        })
        .ok();
    if components.is_empty() {
        // PLAN-043 补刀：桌面宿主 boot 直挂退役后 ui_desktop 恒传空表——
        // 桌面生命周期由 shell/desktop-face 伪窗撑住（dock 电源键退出），
        // 空组件表合法。纯多 app runner（ui_dual_app 族）恒非空，不受
        // 影响；此处降级为提示不再拒启（此前拒启会把无直挂的桌面一并
        // 拒之门外）。
        eprintln!("[ui-desktop] boot components empty — shell-only desktop");
    }

    // Plan 458: seed DECLARED dark_mode / accent_color state vars once from
    // the `auto run` env defaults (AUTO_UI_THEME / AUTO_UI_ACCENT, resolved
    // by the CLI/pac.at layer), so the CLI value stays the INITIAL value even
    // for apps that declare the vars (their model init would otherwise win
    // over the thread-local default from frame one). Only vars that already
    // exist are seeded (read_state Ok = declared); runtime clicks mutate the
    // vars afterwards and take precedence as usual. Plan 459：逐 App 播种。
    for component in &mut components {
        if let Ok(t) = std::env::var("AUTO_UI_THEME") {
            if crate::ui::style::theme::THEME_PREFS.contains(&t.as_str())
                && component.read_state("dark_mode").is_ok()
            {
                let _ = component.write_state("dark_mode", auto_val::Value::Bool(t == "dark"));
            }
        } else {
            // PLAN-615 T-04：env 链（CLI > os-config > pac.at）未解析任何
            // theme 值 → OS 系统主题回退（读取失败 None 则维持 App 内置
            // 缺省）。桌面轨的对应链路在 desktop_config::load 的
            // theme_source=system 派生（T-06），两轨语义对齐。
            if let Some(dark) = crate::ui::system_theme::system_prefers_dark() {
                if component.read_state("dark_mode").is_ok() {
                    let _ = component.write_state("dark_mode", auto_val::Value::Bool(dark));
                }
            }
        }
        if let Ok(a) = std::env::var("AUTO_UI_ACCENT") {
            if crate::ui::style::theme::ACCENT_PRESETS.contains(&a.as_str())
                && component.read_state("accent_color").is_ok()
            {
                let _ = component.write_state("accent_color", auto_val::Value::str(&a));
            }
        }
    }

    /// Save an iced Screenshot as a PNG file in the tmp/ directory (Plan 285).
    /// Plan 371 Task 20: process a captured screenshot according to the requested
    /// mode. Returns a human-readable result string.
    ///
    /// - Default (no name): save a timestamped PNG to `tmp/`, return its path.
    /// - `baseline=true`: save to `tests/screenshots/<name>.png` (overwrite).
    /// - `diff=true`: compare against `tests/screenshots/<name>.png`; return a
    ///   `matches`/`DIFFERS` verdict with the diff percentage, and save a
    ///   highlighted diff image to `tmp/<name>-diff.png` when they differ.
    fn process_screenshot(
        screenshot: &iced::window::Screenshot,
        name: &str,
        baseline: bool,
        diff: bool,
        threshold: f64,
    ) -> Result<String, String> {
        let width = screenshot.size.width;
        let height = screenshot.size.height;
        let img = image::RgbaImage::from_raw(width, height, screenshot.rgba.as_ref().to_vec())
            .ok_or_else(|| "Failed to create RGBA image from screenshot bytes".to_string())?;

        if baseline {
            let dir = std::path::Path::new("tests/screenshots");
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create tests/screenshots: {}", e))?;
            let path = dir.join(format!("{}.png", name));
            img.save(&path)
                .map_err(|e| format!("Failed to save baseline PNG: {}", e))?;
            return Ok(format!("Baseline saved: {}", path.display()));
        }

        if diff {
            let baseline_path =
                std::path::Path::new("tests/screenshots").join(format!("{}.png", name));
            let baseline_img = image::open(&baseline_path)
                .map_err(|e| {
                    format!(
                        "Failed to load baseline '{}': {}",
                        baseline_path.display(),
                        e
                    )
                })?
                .to_rgba8();
            return compare_pngs(&baseline_img, &img, name, threshold);
        }

        // Default: legacy timestamped capture.
        let tmp_dir = std::path::Path::new("tmp");
        std::fs::create_dir_all(tmp_dir)
            .map_err(|e| format!("Failed to create tmp/ directory: {}", e))?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let path = tmp_dir.join(format!("autoui-screenshot-{}.png", timestamp));
        img.save(&path)
            .map_err(|e| format!("Failed to save PNG: {}", e))?;
        let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        Ok(format!(
            "Screenshot saved to: {}",
            abs_path.to_string_lossy()
        ))
    }

    /// Plan 371 Task 20: per-pixel comparison of two RGBA images. Returns a
    /// verdict string and, when they differ beyond `threshold`, writes a
    /// highlighted diff image (changed pixels → red) to `tmp/<name>-diff.png`.
    fn compare_pngs(
        baseline: &image::RgbaImage,
        current: &image::RgbaImage,
        name: &str,
        threshold: f64,
    ) -> Result<String, String> {
        let (bw, bh) = baseline.dimensions();
        let (cw, ch) = current.dimensions();
        if (bw, bh) != (cw, ch) {
            let pct = 100.0;
            return Ok(format!(
            "Screenshot DIFFERS from baseline '{}': size mismatch ({}x{} vs {}x{}, {:.1}%) — threshold {:.1}%",
            name, bw, bh, cw, ch, pct, threshold * 100.0
        ));
        }
        let total = (bw as usize) * (bh as usize);
        let mut differing: usize = 0;
        let mut diff_img = image::RgbaImage::new(bw, bh);
        for y in 0..bh {
            for x in 0..bw {
                let b = baseline.get_pixel(x, y);
                let c = current.get_pixel(x, y);
                if b.0 != c.0 {
                    differing += 1;
                    // Mark changed pixel red, keep alpha.
                    diff_img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                } else {
                    diff_img.put_pixel(x, y, *b);
                }
            }
        }
        let diff_frac = if total == 0 {
            0.0
        } else {
            differing as f64 / total as f64
        };
        let diff_pct = diff_frac * 100.0;
        if diff_frac > threshold {
            // Save the highlighted diff image.
            let tmp_dir = std::path::Path::new("tmp");
            std::fs::create_dir_all(tmp_dir)
                .map_err(|e| format!("Failed to create tmp/ directory: {}", e))?;
            let diff_path = tmp_dir.join(format!("{}-diff.png", name));
            diff_img
                .save(&diff_path)
                .map_err(|e| format!("Failed to save diff PNG: {}", e))?;
            Ok(format!(
            "Screenshot DIFFERS from baseline '{}': {:.2}% pixels changed (threshold {:.2}%) | diff: {}",
            name, diff_pct, threshold * 100.0, diff_path.display()
        ))
        } else {
            Ok(format!(
                "Screenshot matches baseline '{}' ({:.2}% pixels changed, threshold {:.2}%)",
                name,
                diff_pct,
                threshold * 100.0
            ))
        }
    }

    // Plan 459：MCP 以首个（primary）App 命名——MCP 寻址维持 single-app 语义
    // （453 T8 冻结），快照/操作永远指向 primary App。
    let widget_name = components
        .first()
        .map(|c| c.widget_name().to_string())
        .unwrap_or_default();

    // Start MCP UI server in background thread (Plan 278)
    let (mcp_shared, mcp_action_rx) = crate::ui::mcp_server::start_mcp_server(
        widget_name.clone(),
        crate::ui::mcp_server::mcp_port(),
    );
    mcp_shared
        .lock()
        .unwrap()
        .set_backend_kind(crate::ui::mcp_server::BackendKind::Vm);
    // Store the action receiver in a global for the subscription to poll
    {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        *lock = Some(mcp_action_rx);
    }

    // PLAN-740 T-00（batch-drive 勘定通道——test-only）：AUTOUI_DRIVE_PROBE
    // =触发文件路径时 spawn 常驻探针线程；触发文件每追加一行
    // `scroll <hz> <count> <element_id> <ystep>` 即以该速率向 action 通道
    // 直推 <count> 条裸 `__mcp_scroll` 合成动作（异步 send_action，无
    // applied 栅栏；y=三角形波 i*ystep 防钳制）。用途=绕开 MCP HTTP 单发
    // 时延地板（p50 ~15-37ms）的精确速率驱动，供 740 阶梯 ≥54Hz 档勘定
    // ——下游 auto-edit 027 基准的 in-process 驱动形态同构。非该环境零
    // 开销（未设=不 spawn）；语义=test-only 注入，与 MCP 工具同一动作
    // 通道（消费/交付管线全真）。
    if let Ok(probe_file) = std::env::var("AUTOUI_DRIVE_PROBE") {
        spawn_drive_probe(mcp_shared.clone(), probe_file);
    }

    // Plan 414 §5.4: seed the in-app console so the panel has content the
    // moment it opens (print() output joins these lines live).
    auto_lang_core::vm::ui_console::ui_console_push(&format!(
        "[boot] AutoUI VM app \"{}\" started ({} app window{})",
        widget_name,
        components.len(),
        if components.len() > 1 { "s" } else { "" }
    ));
    auto_lang_core::vm::ui_console::ui_console_push(&format!(
        "[boot] MCP automation on 127.0.0.1:{}",
        crate::ui::mcp_server::mcp_port()
    ));

    // Start shell executor (merged in-process / HTTP SSE bridge, ash-gui M1).
    // Returns the event receiver; stash it in SHELL_EVENT_RX for the subscription.
    {
        let shell_rx = start_shell_executor();
        let guard = SHELL_EVENT_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        *lock = Some(shell_rx);
    }

    // BootFn requires Fn (not FnOnce), so we use RefCell<Option<...>> to
    // allow the boot closure to extract the components on the first (and only)
    // call while still satisfying the Fn bound.
    let init = std::cell::RefCell::new(Some(components));

    let boot = move || -> (crate::ui::session::DesktopSession,
                           iced::Task<crate::ui::session::DesktopMessage>) {
        let comps = init.borrow_mut().take()
            .expect("boot should only be called once");
        // R3 退化桌面 + 459 多窗口：逐 App 分配 AppId、开 OS 窗口并同步登记
        // （`window::open` 同步返回 window::Id，无需等 Opened 事件反推归属）。
        let mut session = crate::ui::session::DesktopSession::empty(Some(mcp_shared.clone()));
        match mode {
            RunMode::Desktop => {
                // Plan 462：单宿主 OS 窗口 + N 个虚拟窗口（级联偏移 48n，
                // 初始尺寸 = 宿主的 60%，与独立模式级联错开视觉重叠）。
                // Plan 463 T3：全屏桌面 = borderless + Fullscreen（无关闭
                // 按钮，Esc 退出见桌面级热键臂）。
                let host_size = startup_window_size();
                let mut settings = iced::window::Settings {
                    size: host_size,
                    position: iced::window::Position::Specific(iced::Point::new(80.0, 80.0)),
                    ..Default::default()
                };
                if opts.fullscreen {
                    settings.fullscreen = true;
                    settings.decorations = false;
                }
                let (win_id, open_task) = iced::window::open(settings);
                session.open_desktop(win_id);
                // PLAN-030 T-04/T-07：壳进程模型分叉——boot 读一次（读取点
                // 前移到壳装载之前，§5.1 D7）；outproc = 跳过 in-proc 装载
                // （spawn 挪到 enable_broker 之后——broker 倒挂实锤），失败
                // 回退 in-proc（降级链）。
                // PLAN-030 T-04/T-07：壳进程模型分叉——boot 读一次（读取点
                // 前移到壳装载之前，§5.1 D7）；读序 = env AUTO_SHELL_MODEL
                // （e2e/smoke 便捷注入）> storage `shell.apps.shell_model`；
                // outproc = 跳过 in-proc 装载（spawn 挪到 enable_broker 之后
                // ——broker 倒挂实锤），失败回退 in-proc（降级链）。
                session.desktop.shell_model =
                    crate::ui::session::ShellModel::from_storage(std::env::var(
                        "AUTO_SHELL_MODEL"
                    )
                        .ok()
                        .as_deref()
                        .or(auto_lang_core::vm::ffi::stdlib::storage_host_read(
                            "shell.apps.shell_model",
                        )
                        .as_deref()));
                if session.desktop.shell_model
                    == crate::ui::session::ShellModel::Outproc
                {
                    // 几何缓存（boot 定档：viewport × dock 预留带高）。
                    let reserved =
                        desktop_dock_edges(&session.desktop.config);
                    let geometry = crate::ui::desktop_protocol::shell_client::ShellGeometry {
                        viewport_w: host_size.width,
                        viewport_h: host_size.height,
                        band_h: reserved.bottom.max(36.0),
                    };
                    session.desktop.shell_geometry = Some(geometry);
                } else {
                    // PLAN-012 O1：特权 App 先于直挂 comps 分配——desktop 模式的
                    // "单 App 语义"锚点（primary = BTreeMap 首 App，app_of_window /
                    // MCP 快照同步 / toast 等）此前落在首个直挂窗（如 459 探针、
                    // calculator），MCP 验收通道（autoui_state/autoui_vtree）因此
                    // 只见直挂 App，shell 投影面（__wm_wins 等）与任务栏几何不可
                    // 观测——O1"真实链路 bounds 探针"被此阻断。shell 装载失败时
                    // 依 桌面面 → 首个直挂 comp 顺次回退（语义不变，锚点仍唯一）。
                    // Plan 463 T5：shell 特权 App —— 进程内编译装载（R1/R8
                    // 首落）。装载失败不阻断桌面（无任务栏的退化桌面，stderr 可见）。
                    match crate::ui::shell::build_shell_component() {
                        Ok(shell_comp) => {
                            session.desktop.shell_app = Some(session.allocate_app(shell_comp));
                        }
                        Err(err) => {
                            crate::syslog!(SyslogLevel::Error, "host", "[session] shell load failed (desktop continues): {err}")
                        }
                    }
                    // Plan 496 M5：桌面本体面（第五面，常驻不召唤——与 overlay
                    // 懒挂载不同，boot 期装载）。装载失败不阻断桌面（无图标
                    // 桌面退化，shell 同型降级）。
                    match crate::ui::shell::build_desktop_surface_component() {
                        Ok(surface_comp) => {
                            session.desktop.desktop_app =
                                Some(session.allocate_app(surface_comp));
                        }
                        Err(err) => {
                            crate::syslog!(SyslogLevel::Error, "host", "[session] desktop surface load failed (desktop continues): {err}")
                        }
                    }
                }
                let mut entries: Vec<(crate::ui::session::AppId, String)> = Vec::new();
                for comp in comps {
                    let app_id = session.allocate_app(comp);
                    let title = session
                        .apps
                        .get(&app_id)
                        .map(|a| a.component.widget_name().to_string())
                        .unwrap_or_default();
                    entries.push((app_id, title));
                }
                // 注册表宿主条目：app = primary（app_of_window / MCP / toast
                // 等"单 App 语义"锚点在 desktop 模式仍指向 primary，T8 冻结；
                // PLAN-012 O1 起 primary = 特权 shell——见上方分配序注记）。
                let primary = session.primary_app().expect("desktop boot has apps");
                session.register_window(win_id, primary, host_size);
                for (i, (app_id, title)) in entries.into_iter().enumerate() {
                    let rect = iced::Rectangle::new(
                        iced::Point::new(80.0 + 48.0 * i as f32, 80.0 + 48.0 * i as f32),
                        iced::Size::new(
                            (host_size.width * 0.6).max(360.0),
                            (host_size.height * 0.6).max(280.0),
                        ),
                    );
                    session.wm_add_win(app_id, title, rect);
                }
                // Plan 472 T4：dock 数据级配置（shell.dock.* storage 键）→
                // 布局预留边；缺席回退 pack 默认 bottom/48。v1 boot 读一次。
                session.desktop.dock_edges = desktop_dock_edges(&session.desktop.config);
                // Plan 494：真洞模式位（`shell.native.hole` storage >
                // DesktopOptions 程序位取或；缺席 = off）。
                session.desktop.hole_mode = opts.hole_mode || load_native_hole_mode();
                // PLAN-044 T-04：桌面快捷方式缺键自愈（`shell.desktop.icons`
                // 缺/空 → 播种 DEFAULT_DESKTOP_ICONS 预置集并落盘；有键
                // no-op，幂等）。
                crate::ui::desktop_config::ensure_desktop_icons_seeded();
                // Plan 479 T5：通知历史 boot 恢复（storage 定长槽
                // shell.notes.0..9 读回会话域——I9 单一事实，桌面模式限定）。
                restore_notifications(&mut session);
                // Plan 463 T7：应用注册表 —— 扫描 apps_dir 装配 LaunchApp
                // 解析器。boot 不过滤 render：声明 `render:"vue"` 的 App 多数
                // vm 兼容（011-calculator 即桌面 demo 常客），声明的 render
                // 是前端目标不是 vm 兼容性；真不兼容的由 panic 边界 + 占位页
                // 兜底（T7）。`ScanOptions.render` 过滤开关留给 464 launcher。
                if let Some(apps_dir) = &opts.apps_dir {
                    // Plan 501 + Stage B P-3：多扫描根聚合——examples 主根 +
                    // 外部仓自含根（storage `shell.apps.extra_dirs` + 相邻仓
                    // 探测缺省 `../auto-os-config/auto` 单根 + `../auto-os/apps`
                    // 容器 + apps.manifest repo 条目；去重主根优先）。
                    // `DesktopOptions.extra_app_roots` Some = 全额替换探测
                    //（vue 轨 AUTO_DESKTOP_APPS_EXTRA 同语义）。
                    let extra_roots = match &opts.extra_app_roots {
                        Some(roots) => roots
                            .iter()
                            .filter_map(|p| {
                                p.is_dir().then(|| {
                                    let id = p
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("extra")
                                        .to_string();
                                    (id, p.clone())
                                })
                            })
                            .collect(),
                        None => crate::ui::app_registry::host_extra_roots(),
                    };
                    let full = crate::ui::app_registry::aggregate_scan(
                        apps_dir,
                        &extra_roots,
                        &crate::ui::app_registry::ScanOptions::default(),
                    );
                    // PLAN-552：boot 两分——启动解析器（app_resolver）保留
                    // 全量（按名启动/自定义图标按 id 经全量 resolver 解析，
                    // 不受策展限制）；展示清单（registry_entries：launcher/
                    // 图标格/dock 消费）过滤为策展集（pac `desktop:` 字段；
                    // 主根缺省 false=opt-in，外部自含根缺省 true=opt-out）。
                    let mut curated: Vec<_> = full
                        .iter()
                        .filter(|e| e.desktop_visible)
                        .cloned()
                        .collect();
                    crate::syslog!(
                        SyslogLevel::Info, "host",
                        "[session] app registry: {} entries ({} desktop-visible) from {}",
                        full.len(),
                        curated.len(),
                        apps_dir.display()
                    );
                    let resolver_entries = full;
                    session.desktop.app_resolver =
                        Some(std::sync::Arc::new(move |name: &str| {
                            resolver_entries.iter()
                                .find(|e| e.id == name)
                                // Plan 508 G1：目录名兜底——outproc 子进程按
                                // App 目录名握手（--app386=<dirname>），外部根
                                // 条目 id≠目录名（如 os-config/auto）。
                                .or_else(|| {
                                    resolver_entries.iter().find(|e| {
                                        std::path::Path::new(&e.entry)
                                            .ancestors()
                                            .nth(3)
                                            .and_then(|d| d.file_name())
                                            .is_some_and(|n| n == name)
                                    })
                                })
                                .and_then(|e| {
                                    let code = std::fs::read_to_string(&e.entry).ok()?;
                                    // Plan 501：外部后端根（注册表扫描期已解析为
                                    // 绝对路径；坏路径 launch 臂 is_dir 兜底跳过）。
                                    // Plan 020 T-06：pac `desktop_exe:` 相对 App 根
                                    // 解析（`<dir>/src/front/app.at` 剥三层得 App 根）；
                                    // 缺席 = None（launch 期约定路径兜底扫描）。
                                    let app_dir = std::path::Path::new(&e.entry)
                                        .ancestors()
                                        .nth(3)
                                        .map(|d| d.to_path_buf());
                                    let exe = e
                                        .desktop_exe
                                        .as_deref()
                                        .zip(app_dir.as_ref())
                                        .map(|(rel, dir)| dir.join(rel));
                                    Some(crate::ui::session::LaunchSpec {
                                        code,
                                        source_path: Some(e.entry.to_string_lossy().to_string()),
                                        title: Some(e.display_title().to_string()),
                                        name: e.name.clone(),
                                        daemon: e.daemon.clone(),
                                        back_root: e.back_root.clone(),
                                        opens: e.opens.clone(),
                                        fit: e.fit,
                                        exe,
                                        render_decl: e.desktop_render.clone(),
                                        // PLAN-037：供给决策树透传（capability/
                                        // session 臂的读源）。
                                        media_root: e.media_root.clone(),
                                        // PLAN-043 Part 1：photo capability 臂透传。
                                        photo_root: e.photo_root.clone(),
                                        back_entry: e.back_entry.clone(),
                                    })
                                })
                        }));
                    // Plan 464 T4：launcher 入口 + 注册表快照（召唤注入用）。
                    // 入口匹配：id "launcher" 或 "-launcher" 结尾（441 预订
                    // 028-launcher；459 回退形态同名规则）。
                    // PLAN-552：快照换 curated（028 属 C 档，字段加齐后无回归）。
                    // PLAN-018：桌面图标位图接线——读 auto-os assets/icons/
                    // mapping.json 把命中 id 的 icon 改写 iconfile:<stem>，
                    // 并注入渲染臂资产根（缺席 = 原样 lucide，零回归）。
                    if let Some(root) = crate::ui::iced::icon_file::icon_root() {
                        crate::ui::iced::icon_file::apply_icon_mapping(&mut curated, &root);
                        std::env::set_var("AUTO_OS_ICON_ROOT", &root);
                    }
                    session.desktop.registry_entries = curated;
                    // PLAN-526 T6：boot 直挂窗注册表对齐——回填 registry_id
                    // 并 armed `window: "fit"`（计算器等直挂 App 窗随内容
                    // 收缩；462 固定 60% 初值 blanks 的实测反馈根治）。
                    arm_boot_fit_windows(&mut session);
                    // PLAN-041 T-11：boot 直挂/恢复窗标题注册表对齐——建窗
                    // 期 title = widget 根名（15942，通用 "App"），须在
                    // arm_boot_fit_windows 回填 registry_id 之后覆盖展示名
                    // （守卫 = 仅当 title 仍等于建窗期 widget 名，自定义
                    // window 声明者不误伤；chrome/任务栏/投影单源 v.title）。
                    {
                        let reg_titles: std::collections::HashMap<String, String> = session
                            .desktop
                            .registry_entries
                            .iter()
                            .map(|e| (e.id.clone(), e.display_title().to_string()))
                            .collect();
                        if let Some(host) = session.host.as_mut() {
                            for (_, v) in host.wm.wins.iter_mut() {
                                let Some(rid) = v.registry_id.clone() else { continue };
                                let Some(t) = reg_titles.get(&rid) else { continue };
                                let widget_default = session
                                    .apps
                                    .get(&v.app)
                                    .map(|a| a.component.widget_name().to_string());
                                if Some(v.title.as_str()) == widget_default.as_deref()
                                    && v.title != *t
                                {
                                    v.title = t.clone();
                                }
                            }
                        }
                    }
                    if let Some(e) = session
                        .desktop
                        .registry_entries
                        .iter()
                        .find(|e| e.id == "launcher" || e.id.ends_with("-launcher"))
                    {
                        session.desktop.launcher_entry = Some(e.entry.clone());
                    }
                    // Plan 540 T2：pinned 单源（config.dock_pinned——构造期
                    // DesktopConfig::load 已含旧键迁移；pack 默认 = 缺省链）。
                    session.desktop.dock_pinned = session.desktop.config.dock_pinned.clone();
                    // Plan 490 T3：热键表覆盖（storage `shell.keys.*` >
                    // 内置默认；坏值静默回退——订阅唯一消费面为
                    // desktop_hotkey_subscription 的会话表快照）。
                    for (k, v) in load_hotkey_overrides() {
                        session.desktop.hotkeys.apply_override(&k, &v);
                    }
                    // PLAN-033 T-04（D6）：`shell.apps.process_model` 配置位
                    // 退役（解释态两合法形态 = inproc 直挂 / `-q` 经 native
                    // 臂）。读到 outproc 时忽略留痕（I3 退役非静默）——不
                    // 再影响 launch 路由。
                    if auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.apps.process_model")
                        .as_deref()
                        .map(str::trim)
                        == Some("outproc")
                    {
                        crate::syslog!(
                            SyslogLevel::Info, "host",
                            "[session] shell.apps.process_model=outproc 已退役（PLAN-033）——
                             解释态两合法形态 = inproc 直挂 / -q 经 native 臂；键值忽略"
                        );
                    }
                    // Plan 508 G4：远程 WS 面（storage `shell.remote.token`
                    // 有值才监听 :17800——回环 + token，缺省拒绝=零远程面）。
                    if let Some(token) = load_remote_token() {
                        session.enable_remote_ws(
                            &token,
                            crate::ui::desktop_protocol::remote::REMOTE_WS_PORT,
                        );
                    }
                }
                // Plan 472 T5：{id,icon} 解析注入 shell（apps_dir 缺席也注入
                // ——pinned 常驻，图标回退 "app-window"）。
                inject_dock_pinned(&mut session);
                // Plan 496 M5：桌面本体数据链——壁纸解析（键缺席/坏值回退
                // pack 默认色）+ 图标投影注入（pinned ∪ 自定义合并去重、
                // hidden 排除）。settings/右键移除写键，boot 重读生效
                // （487 pinned 同语义）。
                session.desktop.desktop_wallpaper = load_desktop_wallpaper(&session.desktop.config);
                inject_desktop_surface(&mut session);
                // Plan 518 G1 + Plan 540 T2：boot 主题读回（单源
                // `config.dark_theme` 覆盖 thread-local 默认——设置窗
                // set_theme 持久端,后于 458 env 播种 = 配置文件优先于 CLI
                // 初值）。已声明 dark_mode 的 App 变量须同步——dynamic_view
                // 每帧读该变量回写全局（如 011-calculator）,不同步则首帧
                // 被翻回。
                {
                    let dark = session.desktop.config.dark_theme;
                    crate::ui::style_iced_adapter::set_dark_mode(dark);
                    for app in session.apps.values_mut() {
                        if app.component.read_state("dark_mode").is_ok() {
                            let _ = app
                                .component
                                .write_state("dark_mode", auto_val::Value::Bool(dark));
                        }
                    }
                }
                // Plan 480 S3：真桌面壳孵化通道——broker 常驻受理 spawn 孵化
                // （`auto --autodesk-incubate` 经 `request_incubation` 连入，
                // ServiceTick 帧泵周期 attach 落 462 会话）。
                // PLAN-055 T-01（auto-os 055）：pipe 名 env 覆盖——探测/隔离
                // 环境独占井盖（同名多实例服务端使客户端连入任意桌面，
                // probe 污染实证株）；无 env = 常量名，零行为差。镜像
                // rqhost 的 AUTO_RQHOST_WELLKNOWN 先例。
                let broker_pipe = std::env::var("AUTOOS_BROKER_PIPE")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| crate::ui::desktop_protocol::broker::BROKER_PIPE.to_string());
                session.enable_broker(
                    &broker_pipe,
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                );
                // PLAN-030 T-04：outproc 壳孵化——broker 就位后 spawn（boot
                // 序倒挂清偿）；attach 走 tick 泵。spawn 失败 = 回退 in-proc
                // 装载（降级链 D7——开发友好）+ stderr 留痕，桌面不炸（I5）。
                if session.desktop.shell_model
                    == crate::ui::session::ShellModel::Outproc
                {
                    if let Err(err) = session.launch_shell_outproc() {
                        crate::syslog!(
                            SyslogLevel::Error, "host",
                            "[session] shell outproc spawn failed (fallback inproc): {err}"
                        );
                        match crate::ui::shell::build_shell_component() {
                            Ok(shell_comp) => {
                                session.desktop.shell_app =
                                    Some(session.allocate_app(shell_comp));
                            }
                            Err(err2) => crate::syslog!(
                                SyslogLevel::Error, "host",
                                "[session] fallback shell load failed (desktop continues): {err2}"
                            ),
                        }
                        match crate::ui::shell::build_desktop_surface_component() {
                            Ok(surface_comp) => {
                                session.desktop.desktop_app =
                                    Some(session.allocate_app(surface_comp));
                            }
                            Err(err2) => crate::syslog!(
                                SyslogLevel::Error, "host",
                                "[session] fallback desktop surface load failed: {err2}"
                            ),
                        }
                    }
                }
                // PLAN-024：常驻小组件层——boot 即挂载并显示（用户裁定
                // 2026-09-17：桌面常驻，× = 隐藏 / dock ▦ 切换恢复；
                // toggle 在未挂载态首次调用即挂载+显示，幂等）。
                let dash_boot_task = toggle_dashboard(&mut session);
                (session, open_task.discard().chain(dash_boot_task))
            }
            RunMode::Standalone => {
                let mut open_tasks = Vec::new();
                // Plan 500 步骤 8：像素臂先取桥——窗尺寸 = 表面逻辑尺寸
                //（截图降采样目标与 shm 槽口径同源；非像素臂 startup 尺寸不变）。
                let pixels_child = opts
                    .pixels
                    .then(crate::ui::desktop_protocol::pixels::take_launch)
                    .flatten();
                let pixels_size = pixels_child
                    .as_ref()
                    .map(|c| iced::Size::new(c.size().0, c.size().1));
                for (i, comp) in comps.into_iter().enumerate() {
                    let app_id = session.allocate_app(comp);
                    let size = pixels_size.unwrap_or_else(startup_window_size);
                    // 级联偏移：第 n 窗 +48n px，避免多窗完全重叠遮挡。
                    // PLAN-097：AUTO_VM_POSITION=x,y 覆盖（canvas 会话确定性
                    // 停靠宿主右侧——原生窗为帧渲染源，不遮宿主 Musk 窗）。
                    let position = startup_window_position().unwrap_or_else(|| {
                        iced::window::Position::Specific(iced::Point::new(
                            80.0 + 48.0 * i as f32,
                            80.0 + 48.0 * i as f32,
                        ))
                    });
                    let (win_id, open_task) = iced::window::open(iced::window::Settings {
                        size,
                        position,
                        // Plan 500 步骤 4：像素臂 child 窗口隐藏——渲染/
                        // 截图照常，仅不进 OS 桌面 z 序（合成归宿主虚拟窗）。
                        // PLAN-102：AUTO_VM_HIDDEN=1 同语义 boot 隐藏位（无像素
                        // 桥——canvas 内嵌消费走 MCP 截图/动作通道）。
                        #[cfg(windows)]
                        visible: false,
                        #[cfg(not(windows))]
                        visible: !opts.pixels && !startup_window_hidden(),
                        ..Default::default()
                    });
                    session.register_window(win_id, app_id, size);
                    #[cfg(windows)]
                    super::native_accessibility::prepare(win_id);
                    #[cfg(windows)]
                    let open_task = open_task.then(move |window| {
                        super::native_accessibility::install_task(window, !opts.pixels && !startup_window_hidden())
                    });
                    // Plan 504：pac `window: "fit"` —— 标记待测量；主触发是
                    // 首个 __window_resized（初始 Resized 到达即界面已建，
                    // 测量锚点必在树中），见 DM::Window 臂。
                    // Plan 512：同步置 fit_enabled 持久标记（动态重测识别用）。
                    if startup_window_fit() {
                        if let Some(entry) = session.windows.get_mut(&win_id) {
                            entry.fit_pending.set(true);
                            entry.fit_enabled.set(true);
                        }
                    }
                    open_tasks.push(open_task);
                }
                // Plan 500 步骤 4/8：像素桥装配（boot 顶部已取走——窗尺寸
                // 对齐用），发起 Hello（握手续走协议轮询订阅）。
                if let Some(mut child) = pixels_child {
                    if child.start() {
                        session.pixels = Some(child);
                    }
                }
                // 开窗 Task 的完成通知（window::Id）无需回消息——登记已同步完成，
                // Opened 事件臂作幂等兜底。
                (session, iced::Task::batch(open_tasks).discard())
            }
        }
    };

    let update_inner = |state: &mut crate::ui::session::DesktopSession,
                        app_id: crate::ui::session::AppId,
                        msg: IcedMessage,
                        // PLAN-765 T-02（纪要一 F-4）：true = HTTP 消息桥回填
                        // 消息（DM::HttpBackfill 归一化而来）——派发走
                        // on_http_backfill_for 状态突变序号门控。
                        http_backfill: bool|
     -> iced::Task<IcedMessage> {
        // PLAN-716 组B（供②）: 帧开始时间戳=update 入口到达时刻。门控
        // AUTO_FRAME_BENCH——未设零开销（OnceLock 布尔读+分支），不染
        // 调度语义（712 r2 帧泵域边界）。
        crate::ui::frame_bench::note_frame_begin();
        // PLAN-725 T-00：分段探针帧开始（同门同点——孤儿帧在此落账）。
        crate::ui::frame_segments::frame_begin();
        // PLAN-735 T-02（轴②③④——消息到达/消费时序轴）：update 入口
        // 逐消息 trace（AUTO_SCHED_DIAG 门）——MCP 动作到达 vs 重绘送达
        // （redraw_deliver）vs 泵消费（arm_frame）三轴同一 t0 对读。
        if crate::ui::sched_diag::enabled() {
            let t0 = crate::ui::dynamic::sched_diag_t0();
            let event_disp: String = if msg.event.chars().count() > 48 {
                msg.event.chars().take(48).collect()
            } else {
                msg.event.clone()
            };
            eprintln!(
                "[SCHED-DIAG] update t={}ms app={:?} widget={} event={}",
                t0.elapsed().as_millis(),
                app_id,
                msg.widget,
                event_disp
            );
        }
        // T4c：拆借视图承接旧 DynamicState 平铺命名（施工图 §2 路线甲）。
        // Plan 459：按消息归属 App 拆借（窗口级字段随该 App 的窗口条目）；
        // 缺 App/窗口仅在会话被外部破坏时发生，空转返回。
        let mut state = match state.split_mut(app_id) {
            Some(v) => v,
            None => return iced::Task::none(),
        };
        #[cfg(windows)]
        let msg = {
            if super::native_accessibility::is_tick(&msg) {
                return super::native_accessibility::tick(state.window);
            }
            match super::native_accessibility::resolve(state.window, state.component, &msg) {
                Some(super::native_accessibility::Dispatch::Message(message)) => message,
                Some(super::native_accessibility::Dispatch::Focus(id)) => {
                    return iced::Task::batch([
                        iced::window::gain_focus(state.window),
                        iced::widget::operation::focus(id),
                    ]);
                }
                Some(super::native_accessibility::Dispatch::Rejected) => return iced::Task::none(),
                None => msg,
            }
        };
        if !msg.event.starts_with("__") {
            if auto_lang_core::is_vm_hot_trace() {
                eprintln!(
                    "[UI_EVENT] widget={:?} event={:?} input_val={:?}",
                    msg.widget, msg.event, msg.input_value
                );
            }
        }
        // PLAN-623: test-only VM fixture channel. It is deliberately handled
        // before normal event dispatch so a fixture can seed state and then
        // trigger an existing handler in the same update turn. The MCP tool
        // gates this path with AUTOUI_TEST_FIXTURES=1 and the VM capability;
        // malformed direct messages are acknowledged as errors and never fall
        // through to an app handler.
        if let Some(raw) = msg.event.strip_prefix("__mcp_fixture|") {
            let Some((id_raw, payload)) = raw.split_once('|') else {
                return iced::Task::none();
            };
            let Ok(request_id) = id_raw.parse::<u64>() else {
                return iced::Task::none();
            };
            let mcp_shared = state.desktop.mcp_shared.clone();
            let ack = apply_mcp_fixture(&mut state.component, payload);
            *state.app.view_dirty.borrow_mut() = true;
            if let Some(mcp_shared) = mcp_shared {
                mcp_shared.lock().unwrap().finish_fixture(request_id, ack);
            }
            return iced::Task::none();
        }
        // PLAN-058（auto-down）：043 T6 滚动同步 rust 直写快道退役——
        // VM handler 对 float 实参绑定与算术写入的引擎腐坏（nanbox 整值
        // float 丢标签）已由彼侧过渡计划清偿（回归锁 =
        // plan058_engine_gap_tests），OnLeftScroll/OnRightScroll/
        // SetScrollTop 现走解释器 handler 真消费（app.at 三 handler 补
        // 齐快道同款对栏比例级联），不再在此拦截。

        // PLAN-044 T5: ghost 占位 rust 直写快道——onfocusblock 的 Typed 消息
        //（args = [Int block_index, Float height]，失焦 (Int -1, 0.0)；见
        // aura_view_builder autodown_on_focus_binding）在此拦截直写
        // placeholder state（043 T6 同信任路径：VM handler 对 float 实参/
        // 算术写入腐坏，.at 的 OnEditorFocus handler 保留为 vue 契约面，
        // VM 轨不再向下分派）。state 口径：ghost_id = "block-N" 字符
        // 串（vue data-block-id 同源；空串 = 无 ghost）+ ghost_height
        //（+1e-3 防 nanbox 整值丢标签，0.001px 误差不可见）。
        if msg.event.starts_with("OnEditorFocus") {
            let (clean, args) = crate::ui::dynamic::decode_payload(&msg.event);
            let geti = || {
                args.first().and_then(|v| match v {
                    auto_val::Value::Int(i) => Some(*i),
                    auto_val::Value::Uint(u) => Some(*u as i32),
                    _ => None,
                })
            };
            let getf = || {
                args.get(1).and_then(|v| match v {
                    auto_val::Value::Float(f) => Some(*f as f32),
                    auto_val::Value::Double(d) => Some(*d as f32),
                    auto_val::Value::Int(i) => Some(*i as f32),
                    _ => None,
                })
            };
            if clean == "OnEditorFocus" {
                if let (Some(id), Some(h)) = (geti(), getf()) {
                    write_ghost_state(&mut state.component, id, h);
                    return iced::Task::none();
                }
            }
        }

        // PLAN-045 T5: 表格列宽落定 rust 直写快道——oncolresize 的 Typed
        // 消息（args = [Str "t{表键}", Int col, Float width]；见
        // aura_view_builder autodown_on_col_resize_binding）在此拦截直写
        // table_widths state（043 scroll/044 ghost 同信任路径：VM handler
        // 对 float 实参/map 索引写入腐坏，.at 的 OnColResize handler 保留
        // 为 vue 契约面，VM 轨不再向下分派）。读-改-写保留其余表键；宽
        // +1e-3 防 nanbox 整值丢标签（044 frac 同口径）。view_dirty 单独
        // 不驱动重绘——回发 __noop 走一轮 update→view（__mcp_click 同款）。
        if msg.event.starts_with("OnColResize") {
            let (clean, args) = crate::ui::dynamic::decode_payload(&msg.event);
            if clean == "OnColResize" {
                let key = args.first().and_then(|v| match v {
                    auto_val::Value::Str(s) => Some(s.to_string()),
                    _ => None,
                });
                let col = args.get(1).and_then(|v| match v {
                    auto_val::Value::Int(i) => Some(*i as usize),
                    auto_val::Value::Uint(u) => Some(*u as usize),
                    _ => None,
                });
                let w = args.get(2).and_then(|v| match v {
                    auto_val::Value::Float(f) => Some(*f as f32),
                    auto_val::Value::Double(d) => Some(*d as f32),
                    auto_val::Value::Int(i) => Some(*i as f32),
                    _ => None,
                });
                if let (Some(k), Some(c), Some(w)) = (key, col, w) {
                    write_table_width_state(&mut state.component, &k, c, w);
                    *state.app.view_dirty.borrow_mut() = true;
                    return iced::Task::done(IcedMessage::from_dynamic(&DynamicMessage::String(
                        "__noop".to_string(),
                    )));
                }
            }
        }

        // PLAN-045 T7: MCP resize_col 直落——合成事件 __mcp_resize_col
        //（input_value = "t{表键}␟col␟width"；mcp_server 侧已从 Table
        // vnode props 解析表键）与 OnColResize 拦截同一 write_table_width_state
        // 落 table_widths state；__noop 回发走一轮 update→view 让新列宽进当帧。
        // PLAN-737：合成动作 ack——工具侧在 `__mcp_*` event 尾附 `|ack=<id>`
        // 发送；此处统一剥离还原（下方各臂匹配零改动），ack 随臂内回发
        // noop（noop 排在臂内 Task 队列尾，其后处理即「动作已应用」）或
        // 臂尾直执回执。MCP 工具线程限时轮询回执，把合成动作从「入队」
        // 收紧为「已应用」——探针 press→立即读状态与 60fps 泵竞速、块带
        // 映射系统性偏移（jade N 臂 mapBlocks 伪影）的根修。
        let mut msg = msg;
        let mut mcp_ack: Option<u64> = None;
        if msg.event.starts_with("__mcp_") {
            if let Some((base, id)) = msg.event.rsplit_once("|ack=") {
                if let Ok(id) = id.parse::<u64>() {
                    msg.event = base.to_string();
                    mcp_ack = Some(id);
                }
            }
        }
        if msg.event == "__mcp_resize_col" {
            let mut parts = msg.input_value.as_deref().unwrap_or("").split(PAYLOAD_SEP);
            if let (Some(k), Some(c), Some(w)) = (
                parts.next(),
                parts.next().and_then(|s| s.parse::<usize>().ok()),
                parts.next().and_then(|s| s.parse::<f32>().ok()),
            ) {
                write_table_width_state(&mut state.component, k, c, w);
                *state.app.view_dirty.borrow_mut() = true;
                return iced::Task::done(mcp_noop_with_ack(mcp_ack));
            }
            return iced::Task::none();
        }

        // PLAN-044 T6: MCP click action——autodown 编辑壳块内坐标点击合成
        //（input_value = "storage_key␟x,y"；mcp_server 侧已从 vnode path
        // 解析出编辑壳 storage key）。直调 core.handle_input（真实
        // hit_test/焦点写入路径），focus_changed 后与 widget publish →
        // OnEditorFocus 拦截同一 write_ghost_state 落 ghost state；唯一
        // 不覆盖的是 iced Shell 消息分发本身（T2 单测在册）。
        if msg.event == "__mcp_click" {
            let mut parts = msg.input_value.as_deref().unwrap_or("").split(PAYLOAD_SEP);
            if let (Some(sk), Some(xy)) = (parts.next(), parts.next()) {
                let mut it = xy.split(',');
                if let (Some(x), Some(y)) = (
                    it.next().and_then(|s| s.parse::<f32>().ok()),
                    it.next().and_then(|s| s.parse::<f32>().ok()),
                ) {
                    #[cfg(feature = "autodown")]
                    {
                        use crate::ui::autodown_editor as ade;
                        use crate::ui::autodown_editor::DocInput;
                        let core = ade::autodown_editor(sk);
                        let out = crate::ui::code_editor::core::with_font_system(|fs| {
                            core.handle_input(
                                fs,
                                DocInput::MousePressed {
                                    button: crate::ui::code_editor::core::EditorButton::Left,
                                    x,
                                    y,
                                },
                                &mut crate::ui::code_editor::core::NullClipboard,
                            )
                        });
                        if out.focus_changed {
                            let block = core.focused_block();
                            let h = block
                                .and_then(|i| core.block_rects().get(i).map(|r| r.h))
                                .unwrap_or(0.0);
                            write_ghost_state(
                                &mut state.component,
                                block.map(|b| b as i32).unwrap_or(-1),
                                h,
                            );
                            *state.app.view_dirty.borrow_mut() = true;
                            // view_dirty 单独不驱动 iced 重绘（Task::none 无
                            // 重排）——回发 __noop 消息走一轮 update→view，
                            // 让 ghost 包装进当帧（Plan 482 no-op 通道）。
                            // PLAN-737：ack 随 noop 回执（core 焦点态已落）。
                            return iced::Task::done(mcp_noop_with_ack(mcp_ack));
                        }
                    }
                    #[cfg(not(feature = "autodown"))]
                    let _ = (sk, x, y);
                    mcp_ack_apply(&state.desktop.mcp_shared, mcp_ack);
                    return iced::Task::none();
                }
            }
            return iced::Task::none();
        }

        // PLAN-057（055 D2）: MCP key_press——编辑壳逐键合成（input_value =
        // "storage_key␟widget␟event␟keyspec"；mcp_server 侧已从 vnode path
        // 解析 storage key 并取编辑壳 on_change 名——text_changed 发布步
        // 需要）。keyspec → EditorKey（13 命名键 + c:X）后
        // core.handle_input(KeyPressed) 直调——__mcp_click 同信任路径（真实
        // 按键的同构 core 处理，物理键同路径代证口径）；text_changed 时构造
        // on_change 同形态消息（input_value: Some(全文)——面 A 修复后的
        // 发布形态）驱动 .Edit(str) → state.content；focus_changed 走
        // write_ghost_state；表格读数落 editor_table_geom/editor_col_widths；
        // __noop 回发驱动重绘（Plan 482 通道，click 臂同款）。
        if msg.event == "__mcp_key" {
            let mut parts = msg.input_value.as_deref().unwrap_or("").split(PAYLOAD_SEP);
            if let (Some(sk), Some(widget), Some(event), Some(keyspec)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            {
                #[cfg(feature = "autodown")]
                {
                    use crate::ui::autodown_editor as ade;
                    use crate::ui::autodown_editor::DocInput;
                    if let Some(key) = crate::ui::mcp_server::parse_editor_key_spec(keyspec) {
                        let core = ade::autodown_editor(sk);
                        let out = crate::ui::code_editor::core::with_font_system(|fs| {
                            core.handle_input(
                                fs,
                                DocInput::KeyPressed {
                                    key,
                                    text: None,
                                    modifiers: crate::ui::code_editor::core::EditorModifiers::none(
                                    ),
                                },
                                &mut crate::ui::code_editor::core::NullClipboard,
                            )
                        });
                        let mut tasks = Vec::new();
                        if out.focus_changed {
                            let block = core.focused_block();
                            let h = block
                                .and_then(|i| core.block_rects().get(i).map(|r| r.h))
                                .unwrap_or(0.0);
                            write_ghost_state(
                                &mut state.component,
                                block.map(|b| b as i32).unwrap_or(-1),
                                h,
                            );
                            *state.app.view_dirty.borrow_mut() = true;
                        }
                        if out.text_changed && !event.is_empty() {
                            let text = ade::autodown_editor_text(sk).unwrap_or_default();
                            tasks.push(iced::Task::done(IcedMessage {
                                widget: widget.to_string(),
                                event: event.to_string(),
                                input_value: Some(text),
                            }));
                        }
                        tasks.push(iced::Task::done(mcp_noop_with_ack(mcp_ack)));
                        return iced::Task::batch(tasks);
                    }
                }
                #[cfg(not(feature = "autodown"))]
                let _ = (sk, widget, event, keyspec);
                mcp_ack_apply(&state.desktop.mcp_shared, mcp_ack);
                return iced::Task::none();
            }
            return iced::Task::none();
        }

        // PLAN-057（055 D2）: MCP editor_drag——编辑壳拖拽序列合成（input_value
        // = "storage_key␟widget␟event␟x0,y0;x1,y1;..."）。首点 MousePressed、
        // 中段逐点 MouseDragged、末点 MouseReleased 各一次 core.handle_input
        // 直调——055 列宽拖拽（边界命中建 col_drag → apply_col_drag 写
        // table_widths → 松手落定）与 048 T3 拖选面同覆盖；输出位累积
        // text_changed/focus_changed 处理同 __mcp_key；__noop 回发重绘。
        if msg.event == "__mcp_drag_ade" {
            let mut parts = msg.input_value.as_deref().unwrap_or("").split(PAYLOAD_SEP);
            if let (Some(sk), Some(widget), Some(event), Some(pts)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            {
                #[cfg(feature = "autodown")]
                {
                    use crate::ui::autodown_editor as ade;
                    use crate::ui::autodown_editor::DocInput;
                    let seq = parse_drag_points(pts);
                    if !seq.is_empty() {
                        let core = ade::autodown_editor(sk);
                        let mut focus_changed = false;
                        let mut text_changed = false;
                        for (i, (x, y)) in seq.iter().enumerate() {
                            let (px, py) = (*x, *y);
                            let out = crate::ui::code_editor::core::with_font_system(|fs| {
                                let input = if i == 0 {
                                    DocInput::MousePressed {
                                        button: crate::ui::code_editor::core::EditorButton::Left,
                                        x: px,
                                        y: py,
                                    }
                                } else {
                                    DocInput::MouseDragged { x: px, y: py }
                                };
                                core.handle_input(
                                    fs,
                                    input,
                                    &mut crate::ui::code_editor::core::NullClipboard,
                                )
                            });
                            focus_changed |= out.focus_changed;
                            text_changed |= out.text_changed;
                        }
                        let (last_x, last_y) = seq.last().copied().unwrap_or((0.0, 0.0));
                        let out = crate::ui::code_editor::core::with_font_system(|fs| {
                            core.handle_input(
                                fs,
                                DocInput::MouseReleased {
                                    button: crate::ui::code_editor::core::EditorButton::Left,
                                    x: last_x,
                                    y: last_y,
                                },
                                &mut crate::ui::code_editor::core::NullClipboard,
                            )
                        });
                        focus_changed |= out.focus_changed;
                        text_changed |= out.text_changed;
                        // PLAN-737：完整点击门放行的 wikilink 激活——合成通道
                        // 派发腿。此前 link_activated 在本通道被丢弃（真实
                        // merged 窗完整点击零激活的根因）；现经装配期注册的
                        // 同款 LinkCallback 构造消息（与 widget publish 单源
                        //），无注册（未挂载/泛型轨）零派发语义不变。
                        let mut link_msg: Option<IcedMessage> = None;
                        if let Some(act) = &out.link_activated {
                            link_msg = ade_link_dispatch_message(sk, act);
                            ade::core::ade_trace(&format!(
                                "mcp_drag link_activated target={:?} anchor={:?} dispatched={}",
                                act.target,
                                act.anchor,
                                link_msg.is_some()
                            ));
                        }
                        let mut tasks = Vec::new();
                        if let Some(m) = link_msg {
                            tasks.push(iced::Task::done(m));
                        }
                        if focus_changed {
                            let block = core.focused_block();
                            let h = block
                                .and_then(|i| core.block_rects().get(i).map(|r| r.h))
                                .unwrap_or(0.0);
                            write_ghost_state(
                                &mut state.component,
                                block.map(|b| b as i32).unwrap_or(-1),
                                h,
                            );
                            *state.app.view_dirty.borrow_mut() = true;
                        }
                        if text_changed && !event.is_empty() {
                            let text = ade::autodown_editor_text(sk).unwrap_or_default();
                            tasks.push(iced::Task::done(IcedMessage {
                                widget: widget.to_string(),
                                event: event.to_string(),
                                input_value: Some(text),
                            }));
                        }
                        // PLAN-737：ack 随 noop 回执——link 激活消息等 Task
                        // 先于 noop 同队列派发，noop 消费即门级联全落。
                        tasks.push(iced::Task::done(mcp_noop_with_ack(mcp_ack)));
                        return iced::Task::batch(tasks);
                    }
                }
                #[cfg(not(feature = "autodown"))]
                let _ = (sk, widget, event, pts);
                return iced::Task::none();
            }
            return iced::Task::none();
        }

        // PLAN-043 T6: MCP scroll action 直落——scrollable 无 VM handler，
        // 合成事件 __mcp_scroll 在此拦截（input_value = "element_id␟y"），
        // 直接发 iced scroll_to 写目标滚动位。
        // PLAN-629 T-03: 光标跟随——键盘/IME 把光标移出可见带时，编辑器发
        // 本命令；会话读 caret 内容 y 换算目标偏移（caret 置视口上沿下一行）。
        if msg.event == "__editor_scroll_to_caret" {
            let key = &msg.widget;
            if let Some(caret_y) = crate::ui::code_editor::code_editor_caret_offset_y(key) {
                let target = (caret_y - 24.0).max(0.0);
                return iced::widget::operation::scroll_to(
                    iced::widget::Id::from(format!("editor-scroll-{key}")),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: target },
                );
            }
            return iced::Task::none();
        }
        if msg.event == "__mcp_scroll" {
            let mut parts = msg.input_value.as_deref().unwrap_or("").split(PAYLOAD_SEP);
            // PLAN-656 F-4 末环：可选第三段 x（controller 双轴回环复用本消费者；
            // 既有 MCP 调用 "id␟y" 两段形态不变，x 缺省 0）。
            if let (Some(id), Some(y)) = (
                parts.next(),
                parts.next().and_then(|s| s.parse::<f32>().ok()),
            ) {
                let x = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                if std::env::var("P656_DEBUG").is_ok() {
                    eprintln!("[P656-EXEC] mcp_scroll id={id} x={x} y={y}");
                }
                mcp_ack_apply(&state.desktop.mcp_shared, mcp_ack);
                return iced::Task::batch([iced::widget::operation::scroll_to(
                    id.to_string(),
                    iced::widget::scrollable::AbsoluteOffset { x, y },
                )]);
            }
            return iced::Task::none();
        }
        // PLAN-043 T10: MCP drag action——mouse-area 拖拽序列合成（input_value
        // = "W␟Down␟Move␟Up␟x0,y0;x1,y1;..."）。逐段 on_with_input_for，与
        // PointerArea 闭包产出的消息同构：down 不带实参（VM 按压无坐标原语，
        // e 未绑定 → TrackDown 的 DOM 几何守卫为假，与真实按压一致）；
        // move 附加 (x, y) 尾参（+1e-3 防 nanbox 整值丢标签，见上 frac）；
        // up 收尾。.at 拖拽数学 → emit → SetScrollTop fast-path → 写臂
        // scroll_to 全走真实机制；唯一不覆盖的是 iced 按钮分发本身（库层）。
        if msg.event == "__mcp_drag" {
            if let Some(spec) = msg.input_value.as_deref() {
                let parts: Vec<&str> = spec.split(PAYLOAD_SEP).collect();
                if parts.len() == 5 {
                    let (w, down, mv, up, pts) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
                    // down 带 $event 冻结标记实参（与真实转换器的
                    // onmousedown DynamicMessage 同构——handler 声明 (e) 时
                    // 0 参派发会错帧，函数体静默不执行）。
                    // PLAN-095 T-05: move 坐标改 Float 编码——真实指针通道
                    // (mouse_area_move_arm) 推 Float(x+1e-3)，合成通道此前
                    // 推 Double：同一 float 形参双通道位型分叉（nanbox
                    // fallback 位型错读风险，AC-06 ≤0.5px 合同）。
                    let down_ev = encode_payload(down, &[auto_val::Value::str("$event")]);
                    state.component.on_with_input_for(w, &down_ev, None);
                    for pair in pts.split(';').filter(|s| !s.is_empty()) {
                        let mut it = pair.split(',');
                        if let (Some(x), Some(y)) = (
                            it.next().and_then(|s| s.parse::<f64>().ok()),
                            it.next().and_then(|s| s.parse::<f64>().ok()),
                        ) {
                            let ev = encode_payload(
                                mv,
                                &[
                                    auto_val::Value::Float(x + 0.001),
                                    auto_val::Value::Float(y + 0.001),
                                ],
                            );
                            state.component.on_with_input_for(w, &ev, None);
                        }
                    }
                    state.component.on_with_input_for(w, up, None);
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
            mcp_ack_apply(&state.desktop.mcp_shared, mcp_ack);
            return iced::Task::none();
        }
        // Plan 563 T5: MCP pen action——canvas 笔画序列合成(input_value =
        // "W␟Start␟Move␟End␟x0,y0;x1,y1;...")。penstart 带首点 (x,y) 尾参
        // (PenArea onpenstart 消息同构,+1e-3 分数化同 drag);中继点逐段
        // penmove;penend 带尾点坐标。widget 层门控/限频/出界收笔是真实
        // 事件语义(pen_area.rs),不由本合成通道覆盖。
        if msg.event == "__mcp_pen" {
            if let Some(spec) = msg.input_value.as_deref() {
                let parts: Vec<&str> = spec.split(PAYLOAD_SEP).collect();
                if parts.len() == 5 {
                    let (w, start, mv, end, pts) =
                        (parts[0], parts[1], parts[2], parts[3], parts[4]);
                    let pairs: Vec<(f64, f64)> = pts
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .filter_map(|pair| {
                            let mut it = pair.split(',');
                            let x = it.next().and_then(|s| s.parse::<f64>().ok())?;
                            let y = it.next().and_then(|s| s.parse::<f64>().ok())?;
                            Some((x, y))
                        })
                        .collect();
                    if let Some(&(x, y)) = pairs.first() {
                        let ev = encode_payload(
                            start,
                            &[
                                auto_val::Value::Float(x + 0.001),
                                auto_val::Value::Float(y + 0.001),
                            ],
                        );
                        state.component.on_with_input_for(w, &ev, None);
                    }
                    for &(x, y) in pairs.iter().skip(1) {
                        let ev = encode_payload(
                            mv,
                            &[
                                auto_val::Value::Float(x + 0.001),
                                auto_val::Value::Float(y + 0.001),
                            ],
                        );
                        state.component.on_with_input_for(w, &ev, None);
                    }
                    if let Some(&(x, y)) = pairs.last() {
                        let ev = encode_payload(
                            end,
                            &[
                                auto_val::Value::Float(x + 0.001),
                                auto_val::Value::Float(y + 0.001),
                            ],
                        );
                        state.component.on_with_input_for(w, &ev, None);
                    }
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
            mcp_ack_apply(&state.desktop.mcp_shared, mcp_ack);
            return iced::Task::none();
        }
        // Plan 412 续(toast 修正 3/6):在 update 最前消费 handler 写入的
        // __toast state —— push 进堆叠并立即清空(无去重:同一条消息可反复
        // 触发,每次都是新 toast)。必须在所有事件分支**之前**:按钮点击等
        // 用户事件路径在中途 return,放 update 尾部时这些路径写入的 payload
        // 永远等不到消费。到期由 __toast_tick 订阅按 shown_at + duration_ms
        // 驱动(见下方分支),不再从 update 返回 Task —— 本应用 update 返回
        // 的 Task::perform 从未被执行。handler_codegen 侧是追加式赋值
        // (__toast += "\x1E" + 记录),同一 handler 连发多条不覆盖 —— 这里按
        // \x1E 拆记录逐条入队(空记录跳过)。堆叠上限 8,超出丢弃最旧。
        if let Ok(auto_val::Value::Str(payload)) = state.component.read_state("__toast") {
            if !payload.is_empty() {
                let _ = state
                    .component
                    .write_state("__toast", auto_val::Value::str(""));
                let mut pushed_any = false;
                for rec in payload.split('\u{1e}') {
                    if rec.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = rec.split('\u{1f}').collect();
                    if parts.len() == 4 && !parts[1].is_empty() {
                        let duration = parts[3].parse::<u64>().unwrap_or(4000).max(200);
                        let id = state.desktop.toast_next_id.get();
                        state.desktop.toast_next_id.set(id + 1);
                        {
                            let mut toasts = state.desktop.toasts.borrow_mut();
                            if toasts.len() >= 8 {
                                toasts.remove(0);
                            }
                            toasts.push(ToastReq {
                                id,
                                kind: parts[0].to_string(),
                                msg: parts[1].to_string(),
                                position: parts[2].to_string(),
                                shown_at: std::time::Instant::now(),
                                duration_ms: duration,
                            });
                        }
                        pushed_any = true;
                    }
                }
                // toast 层是 dynamic_view 结尾 Stack 包装的一部分,而 view() 对
                // 非 dirty 帧直接返回 cached_rendered(跳过包装)—— 入队后必须
                // 置 view_dirty,否则 toast 永远不显示;到期路径在 __toast_tick
                // 里置。
                if pushed_any {
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
        }
        // Plan 402: on first update, resize window to model's window_width/window_height
        // if declared (lets each example specify its own initial window size via Auto).
        if !state.initial_resize_done.get() {
            state.initial_resize_done.set(true);
            let w = state
                .component
                .read_state("window_width")
                .map(|v| v.as_int())
                .unwrap_or(0);
            let h = state
                .component
                .read_state("window_height")
                .map(|v| v.as_int())
                .unwrap_or(0);
            if w > 0 && h > 0 {
                *state.pending_window_resize.borrow_mut() =
                    Some(iced::Size::new(w as f32, h as f32));
            }
        }

        // Clear component dirty at start of each update cycle.
        // It will be re-set by on_with_input/write_state/reload if state actually changes.
        state.component.clear_dirty();

        // Plan 401/VM-routing: a `link` click arrives as a __navigate message
        // whose event string embeds the target path (encode_payload format:
        // "__navigate\u{1F}s\u{1F}/book/1"). Intercept it, set the route, and
        // skip normal handler dispatch (it's a synthetic internal event).
        if msg.event.starts_with("__navigate") {
            // Plan 482: "__navigate_back" (router.back() / nav back button)
            // pops the history stack — must not fall into the path-parsing
            // branch above (its payload carries no path).
            if msg.event.starts_with("__navigate_back") {
                if state.component.navigate_back() {
                    *state.app.view_dirty.borrow_mut() = true;
                }
                return iced::Task::none();
            }
            let path = msg
                .event
                .split(PAYLOAD_SEP)
                .nth(2)
                .unwrap_or("/")
                .to_string();
            state.component.set_route(&path);
            *state.app.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }

        // Plan 482: nav-group built-in fold toggle ("__nav_toggle\u{1F}s\u{1F}<key>")
        // — flips the per-group open state (DynamicComponent), view rebuilds.
        if msg.event.starts_with("__nav_toggle") {
            let (name, args) = crate::ui::dynamic::decode_payload(&msg.event);
            debug_assert_eq!(name, "__nav_toggle");
            if let Some(auto_val::Value::Str(key)) = args.first() {
                state.component.toggle_nav_group(key.as_str());
                *state.app.view_dirty.borrow_mut() = true;
            }
            return iced::Task::none();
        }

        // Plan 482: inert nav-item click (no to:/onclick) — graceful no-op.
        // PLAN-737：ack 随行 noop 的回执点——合成动作臂回发的 noop 消费即
        // 该动作（含其 Task 级联：link 消息派发/VM handler）已全部应用。
        if msg.event.starts_with("__noop") {
            if let Some(id) = msg
                .event
                .strip_prefix("__noop|ack=")
                .and_then(|s| s.parse::<u64>().ok())
            {
                mcp_ack_apply(&state.desktop.mcp_shared, Some(id));
            }
            return iced::Task::none();
        }

        // Plan 412 续(toast 修正 3):到期消息由 update 结尾发放的一次性
        // Task(sleep duration)发出 —— toast 显示期间零消息零重建,不干扰
        // 滚动/焦点/交互。按 id 移除,其余 toast 上移补位。
        if msg.event == "__toast_tick" {
            // Plan 412 续(toast 修正 5):到期改由 subscription tick 驱动 ——
            // 本应用从 update 返回的 Task::perform(含 tokio sleep)从未被
            // 执行(消息零到达),而 iced::time::every 订阅可靠(hot_reload/
            // heartbeat 均在跳)。250ms tick 仅在有 toast 时订阅;每次 tick
            // 只有真的过期了才置 view_dirty,配合 dynamic_view 的 Element
            // 缓存,稳态零重建、不扰动滚动/焦点。
            let now = std::time::Instant::now();
            let mut toasts = state.desktop.toasts.borrow_mut();
            let before = toasts.len();
            toasts.retain(|t| now.duration_since(t.shown_at).as_millis() < t.duration_ms as u128);
            let removed = toasts.len() != before;
            drop(toasts);
            if removed {
                *state.app.view_dirty.borrow_mut() = true;
            }
            return iced::Task::none();
        }

        // Plan 442 A5: one-shot timer tick — fire every due set_timeout
        // callback (event form → handler dispatch; closure form →
        // call_closure). Marks the view dirty only when something fired;
        // the subscription itself only exists while timers are pending
        // (16ms cadence, matching the auto-down SchedulerTimer batch rate).
        if msg.event == "__timer_tick" {
            let fired = state.component.poll_timers();
            if fired > 0 {
                *state.app.view_dirty.borrow_mut() = true;
            }
            return iced::Task::none();
        }

        // PLAN-702 T-03: parked handler 段恢复泵 — resume every parked
        // segment whose wait (async HTTP / external future) is ready, then
        // re-render the mid-await state writes. Same conditional-subscription
        // family as `__timer_tick`: the subscription only exists while
        // `has_parked_tasks()` (16ms cadence; the iced update that parked the
        // handler re-evaluates subscriptions, first tick lands ≤16ms after a
        // result becomes ready). 恢复触发待澄清项①的落地形态：AppTick 轮询
        // （`__timer_tick` 同族条件订阅——免静态通道、免跨 App 路由歧义，
        // 恢复内联在 update 内天然满足单 VM 串行裁断）。
        if msg.event == "__parked_resume_tick" {
            // PLAN-712 r2 T-16 诊断：恢复泵到达/结果计数（stderr 对账）。
            // PLAN-721 T-1：到达 + 泵后置脏传播 trace（桌面轨断链定案轴）。
            let parked_before = state.component.has_parked_io_tasks();
            crate::ui::sched_diag::trace(
                Some(app_id),
                "arm_parked",
                &format!(
                    "arrive parked_io_before={} pending_init={} cpu={}",
                    parked_before,
                    state.component.has_pending_init_work(),
                    state.component.has_cpu_continuations(),
                ),
            );
            state.component.poll_parked_resumes();
            // PLAN-712 r2 T-16：恢复完成可能写状态（Init 挂起 fetch 回来
            // 填数据）——不标脏则视图停在「Loading...」直到下一外部事件
            // 强迫重渲染（018 书架/详情实机实证）。与 __frame_pump 臂同款
            // is_dirty→view_dirty 回填。
            let dirty = state.component.is_dirty();
            if dirty {
                *state.app.view_dirty.borrow_mut() = true;
            }
            crate::ui::sched_diag::trace(
                Some(app_id),
                "arm_parked",
                &format!(
                    "done parked_io_after={} dirty={} view_dirty_set={}",
                    state.component.has_parked_io_tasks(),
                    dirty,
                    dirty,
                ),
            );
            return iced::Task::none();
        }

        // PLAN-711 T-04 (D-1): 帧通知泵消费臂——listen_raw 收到的
        // RedrawRequested 经 runtime 通道异步回环，**消费该消息时本帧
        // present 已同步返回**（D-1 序障）——此处运行有界泵（Init demand
        // 派发 + CPU continuation 续跑，共享 8ms 轮次预算），骨架先呈现。
        // dirty 直置 view_dirty（早退臂不达 update 尾回填；与热重载臂
        // ：18457 同款模式）——完成帧即重建，不等 Element 缓存 take 空
        // 后的 fall-through 帧。
        if msg.event == "__frame_pump" {
            // PLAN-716 组B（供②）: 呈现完成时间戳=帧泵消息消费时刻（711
            // D-1 序障：消费时本帧 present 已同步返回——合法代理）。
            crate::ui::frame_bench::note_frame_present();
            // PLAN-725 T-00：分段探针帧发布（present 行；孤儿帧纪律见模块头）。
            crate::ui::frame_segments::frame_present();
            state.component.poll_frame_pump();
            let dirty = state.component.is_dirty();
            if dirty {
                *state.app.view_dirty.borrow_mut() = true;
            }
            // PLAN-721 T-1：泵后置脏传播 trace（桌面轨断链定案轴——到达面
            // 由 poll_frame_pump 内既有 frame_pump enter 行承担）。
            crate::ui::sched_diag::trace(
                Some(app_id),
                "arm_frame",
                &format!(
                    "done pending_init={} cpu={} parked={} dirty={} view_dirty_set={}",
                    state.component.has_pending_init_work(),
                    state.component.has_cpu_continuations(),
                    state.component.has_parked_io_tasks(),
                    dirty,
                    dirty,
                ),
            );
            return iced::Task::none();
        }

        // Plan 409 §10 续 19: preview-card 的 toggle/tab(局部 UI state,存 DynamicComponent)。
        if msg.event.starts_with("__preview_toggle")
            || msg.event.starts_with("__preview_tab")
            || msg.event.starts_with("__preview_copy")
        {
            let (name, args) = crate::ui::dynamic::decode_payload(&msg.event);
            match name.as_str() {
                "__preview_toggle" => {
                    if let Some(auto_val::Value::Str(id)) = args.get(0) {
                        let st = state
                            .component
                            .preview_states
                            .entry(id.to_string())
                            .or_default();
                        st.show = !st.show;
                        st.copied = false;
                        // PLAN-708 T-01：组件局部 UI 态不 bump VM seq——
                        // epoch 通道废止在册 memo 产物（否则 outlet 命中回放
                        // 陈旧 tab/show 面）。
                        crate::ui::memo_deps::bump_ui_epoch();
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                "__preview_tab" => {
                    if let (Some(auto_val::Value::Str(id)), Some(auto_val::Value::Str(tab))) =
                        (args.get(0), args.get(1))
                    {
                        let st = state
                            .component
                            .preview_states
                            .entry(id.to_string())
                            .or_default();
                        st.tab = if tab.as_str() == "vue" {
                            crate::ui::dynamic::PreviewTab::Vue
                        } else {
                            crate::ui::dynamic::PreviewTab::Auto
                        };
                        st.copied = false;
                        // PLAN-708 T-01：组件局部 UI 态——epoch 通道废止 memo。
                        crate::ui::memo_deps::bump_ui_epoch();
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                // Plan 411: preview-card copy icon — args [id, code]. Writes the
                // current tab's code to the system clipboard (arboard) and flips
                // the copied flag so the icon swaps to a check until the next
                // tab/toggle interaction (vue parity: instant copy feedback).
                "__preview_copy" => {
                    if let (Some(auto_val::Value::Str(id)), Some(auto_val::Value::Str(code))) =
                        (args.get(0), args.get(1))
                    {
                        match arboard::Clipboard::new() {
                            Ok(mut cb) => {
                                if let Err(e) = cb.set_text(code.as_str().to_string()) {
                                    eprintln!("preview copy failed: {}", e);
                                }
                            }
                            Err(e) => eprintln!("clipboard unavailable: {}", e),
                        }
                        let st = state
                            .component
                            .preview_states
                            .entry(id.to_string())
                            .or_default();
                        st.copied = true;
                        // PLAN-708 T-01：组件局部 UI 态——epoch 通道废止 memo。
                        crate::ui::memo_deps::bump_ui_epoch();
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                _ => {}
            }
        }

        // Plan 423 P1: 配置热替换传播 —— 每条消息核对 generation(手动 reload/
        // MCP 工具/轮询任一途径 bump),变了强制重建;mtime 轮询挂 heartbeat
        // 节拍(500ms 节流,变更才真正重读)。
        {
            let gen = crate::ui::action_config::config_generation();
            let last_seen = CONFIG_GEN_SEEN.swap(gen, std::sync::atomic::Ordering::SeqCst);
            if gen != last_seen {
                *state.app.view_dirty.borrow_mut() = true;
            }
            if msg.event == "__mcp_heartbeat" {
                let now = std::time::Instant::now();
                let due = CONFIG_MTIME_POLL.lock().unwrap().map_or(true, |t| {
                    now.duration_since(t) >= std::time::Duration::from_millis(500)
                });
                if due {
                    *CONFIG_MTIME_POLL.lock().unwrap() = Some(now);
                    if crate::ui::action_config::check_action_config_changed() {
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
            }
            // PLAN-654 阶段 B: 框架墙钟 1Hz 刷新（仅消费方订阅时到达）。
            if msg.event == crate::ui::dynamic::DynamicComponent::CLOCK_TICK_EVENT {
                if state.component.handle_clock_tick() {
                    *state.app.view_dirty.borrow_mut() = true;
                }
                return iced::Task::none();
            }
        }

        // Plan 418 P2-3: synthesized menubar (config-driven via auto-edit.at)
        // local UI state — same internal-message pattern as preview-card
        // states. Open state lives in action_config; any non-internal message
        // closes the menu (internal ticks/heartbeats excluded so MCP runs
        // don't auto-close menus).
        if msg.event.starts_with("__menubar") {
            let (name, args) = crate::ui::dynamic::decode_payload(&msg.event);
            match name.as_str() {
                "__menubar_toggle" => {
                    if let Some(auto_val::Value::Str(id)) = args.get(0) {
                        let next = if crate::ui::action_config::menubar_open().as_deref()
                            == Some(id.as_str())
                        {
                            None
                        } else {
                            Some(id.to_string())
                        };
                        crate::ui::action_config::set_menubar_open(next);
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                "__menubar_close" => {
                    crate::ui::action_config::set_menubar_open(None);
                    *state.app.view_dirty.borrow_mut() = true;
                    return iced::Task::none();
                }
                _ => {}
            }
        } else if msg.event.starts_with("__popover") {
            // Plan 422 P4: DSL popover 自管开合(无 open 属性的 widget 锚
            // 形态,gallery 的 popover/popover-trigger/popover-content 页)。
            // 与 menubar 同型:slot id 为构建路径键,dismiss 走 popover 自身
            // 的 on_dismiss 内部消息。
            let (name, args) = crate::ui::dynamic::decode_payload(&msg.event);
            match name.as_str() {
                "__popover_toggle" => {
                    if let Some(auto_val::Value::Str(slot)) = args.get(0) {
                        let next = if crate::ui::action_config::popover_open().as_deref()
                            == Some(slot.as_str())
                        {
                            None
                        } else {
                            Some(slot.to_string())
                        };
                        crate::ui::action_config::set_popover_open(next);
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                "__popover_close" => {
                    crate::ui::action_config::set_popover_open(None);
                    *state.app.view_dirty.borrow_mut() = true;
                    return iced::Task::none();
                }
                _ => {}
            }
        } else if crate::ui::action_config::menubar_open().is_some()
            && !msg.event.starts_with("__")
            // PLAN-664 U-1：TimeSource 泵事件（DSL Tick/timer{} 处理器名，
            // 无 `__` 前缀）是框架周期消息而非用户交互——不参与"任意
            // 非内部消息关菜单"。041 状态栏时钟每秒的 `Tick` 曾把刚打开
            // 的 menubar 在下一次快照前关掉（041 矩阵六失败根因）。
            && !state.component.is_timesource_event(&msg.event)
        {
            crate::ui::action_config::set_menubar_open(None);
            *state.app.view_dirty.borrow_mut() = true;
        }

        // Pick up pending screenshot request from MCP thread at every update (Plan 285).
        if state.app.devtools.screenshot_request.borrow().is_none() {
            if let Some(ref mcp_handle) = state.desktop.mcp_shared {
                if let Some(req) = mcp_handle.lock().unwrap().take_screenshot_request() {
                    *state.app.devtools.screenshot_request.borrow_mut() = Some(req);
                }
            }
        }

        // PLAN-656 review F-4: scroll 状态读回落库——ScrollStateReader 的
        // 六测量按 widget id 反查 handle 写入 controller 注册表（预热/校正）。
        if msg.event == "__scroll_state_read" {
            if let Some(ref json) = msg.input_value {
                if let Ok(map) = serde_json::from_str::<
                    crate::ui::iced::scroll_state_reader::ScrollStateMap,
                >(json)
                {
                    // PLAN-781 T-03: pane 原点旁路 stash（锚点解析第二路输入；
                    // 与锚点 rect 到达顺序不定，齐备即解析）。
                    let origins: std::collections::HashMap<String, (f64, f64)> = map
                        .iter()
                        .map(|(id, e)| (id.clone(), (e.viewport[0] as f64, e.viewport[1] as f64)))
                        .collect();
                    crate::ui::iced::scroll_anchor::note_pane_origins(origins);
                    crate::ui::iced::scroll_anchor::stash_resolved_finals(
                        crate::ui::iced::scroll_anchor::try_resolve_pending(),
                    );
                    for (id, entry) in map {
                        let (ox, oy) = (entry.offset[0], entry.offset[1]);
                        let (vw, vh) = (entry.viewport[2], entry.viewport[3]);
                        let (cw, ch) = (entry.content[0], entry.content[1]);
                        if std::env::var("P656_DEBUG").is_ok() {
                            eprintln!("[P656-READ] id={id} vals={:?}", (ox, oy, vw, vh, cw, ch));
                        }
                        // PLAN-701 供②b: editor 句柄（editor-scroll-*）放开
                        // offset 回写——读端点 `code_editor_scroll_offset_*`
                        // 的注册表投影由此收敛（用户滚动无 on_scroll 回声臂，
                        // M 泛型臂；读回 offset 与 Viewport::absolute_offset
                        // 同源同号已证可信[F-4]）。controller pane 维持
                        // extent-only 不变（排空同帧 pre-scroll 读值竞态的
                        // 原 F-4 裁定不受扰）；editor 句柄同帧竞态由心跳
                        // 节拍收敛（写臂 drain 投影先行，读回校正随后）。
                        if id.starts_with("editor-scroll-") {
                            crate::ui::scroll::controller::note_controller_offset(
                                &id, ox as f64, oy as f64,
                            );
                        }
                        for handle in crate::ui::scroll::controller::handles_for_widget(&id) {
                            // F-4 终段：extent-only——offset 语义单源在注册表
                            //（写臂回填/on_scroll 回声/用户滚动）；读回 offset
                            // 同号修正后虽已可信，v1 不回写（排空同帧的
                            // pre-scroll 读值会短暂覆盖命令值投影）。
                            crate::ui::scroll::controller::note_controller_extents(
                                &handle,
                                (vw as f64, vh as f64),
                                (cw as f64, ch as f64),
                            );
                        }
                    }
                }
            }
        }

        // PLAN-781 T-11：MCP 观测面的 pane 几何发布（与 __bounds_collected
        // 同一批 operate 推送，相邻 tick 到达——布局不变，同帧语义）。只进
        // SharedState，不触 controller 注册表（那是 __scroll_state_read 的
        // 职责，F-4/PLAN-701 裁定不受扰）。
        if msg.event == "__mcp_scroll_states" {
            if let Some(ref json) = msg.input_value {
                if let Ok(map) = serde_json::from_str::<
                    crate::ui::iced::scroll_state_reader::ScrollStateMap,
                >(json)
                {
                    if let Some(ref mcp) = state.desktop.mcp_shared {
                        mcp.lock().unwrap().set_scroll_states(map);
                    }
                }
            }
            return iced::Task::none();
        }

        // PLAN-781 T-03: 锚点 rect 采集回执——stash 后尝试解析（pane 原点
        // 已在 __scroll_state_read 侧 stash；两路齐备即产出 finals，暂存
        // 待 drain 块统一落盘）。
        if msg.event == "__anchor_bounds_collected" {
            if let Some(ref json) = msg.input_value {
                if let Ok(rects) =
                    serde_json::from_str::<crate::ui::iced::scroll_anchor::AnchorRectMap>(json)
                {
                    if std::env::var("P781_DEBUG").is_ok() {
                        eprintln!("[P781] anchor rects collected: {} entries", rects.len());
                    }
                    crate::ui::iced::scroll_anchor::note_anchor_rects(rects);
                    crate::ui::iced::scroll_anchor::stash_resolved_finals(
                        crate::ui::iced::scroll_anchor::try_resolve_pending(),
                    );
                }
            }
            return iced::Task::none();
        }

        // Layout bounds collection: store result from previous operation (Plan 282)
        if msg.event == "__bounds_collected" {
            if let Some(ref json) = msg.input_value {
                if let Ok(bounds_map) = serde_json::from_str::<
                    std::collections::HashMap<String, (f32, f32, f32, f32)>,
                >(json)
                {
                    // Backfill layout bounds into the debug InspectorCache first
                    // (Plan 307, Task 13) — borrows `bounds_map` by ref.
                    // `live_cache` is `None` outside debug mode (Task 12 clears
                    // it), so this borrow is the debug gate. Padding/margin
                    // refinement is deferred until `raw_class` is populated by a
                    // later task.
                    if let Some(cache) = state.app.live_cache.borrow_mut().as_mut() {
                        crate::ui::debug::backfill_bounds(cache, &bounds_map);
                    }
                    // PLAN-646: 框选定稿后的延迟计算——此刻 bounds 刚采集
                    // （新鲜），执行 select→trim→envelope 并打开 Select 标签页
                    //（__toggle_inspect 同款联动：debug_mode + devtools_open）。
                    // 先于 mcp 块消费（下方 set_layout_bounds 会 move bounds_map）。
                    let sel_rect = state.app.devtools.pending_selection.borrow_mut().take();
                    if let Some(sel_rect) = sel_rect {
                        let vnode_bounds: std::collections::HashMap<
                            crate::ui::vnode::VNodeId,
                            crate::ui::debug::Rect,
                        > = bounds_map
                            .iter()
                            .filter_map(|(k, (x, y, w, h))| {
                                let vid = state
                                    .app
                                    .live_cache
                                    .borrow()
                                    .as_ref()
                                    .and_then(|c| c.iced_to_vnode(k))?;
                                Some((vid, crate::ui::debug::Rect::new(*x, *y, *w, *h)))
                            })
                            .collect();
                        // Plan 483 D4 同源裁定：vtree 取 mcp_sync_vtree（裸
                        // View 快照），缺缓存退 live_vtree（id 为 path 派生，
                        // 结构稳定时两树 id 一致）。
                        let vtree_src = state
                            .app
                            .mcp_sync_vtree
                            .borrow()
                            .clone()
                            .or_else(|| state.app.live_vtree.borrow().clone());
                        if let Some(vtree) = vtree_src {
                            ensure_source_loaded(state.as_ref_view());
                            let source = state.app.source_code.borrow().clone();
                            let app_name = state.component.widget_name().to_string();
                            let result = crate::ui::selection::select_envelope(
                                "vm",
                                &app_name,
                                sel_rect,
                                &vnode_bounds,
                                &vtree,
                                source.as_deref(),
                            );
                            if std::env::var("AUTO_SELECT_TRACE").is_ok() {
                                eprintln!(
                                    "[PLAN-646] select rect=({:.0},{:.0},{:.0},{:.0}) bounds={} nodes={}",
                                    sel_rect.x, sel_rect.y, sel_rect.width, sel_rect.height,
                                    vnode_bounds.len(),
                                    result.nodes.len()
                                );
                            }
                            *state.app.devtools.selection_result.borrow_mut() = Some(result);
                            *state.app.devtools.select_copy_feedback.borrow_mut() = None;
                            *state.app.devtools.select_view.borrow_mut() =
                                crate::ui::selection::SelectionFormat::Auto;
                            state.app.devtools.debug_mode = true;
                            *state.app.devtools.devtools_open.borrow_mut() = true;
                            *state.app.devtools.devtools_tab.borrow_mut() = DevToolsTab::Select;
                            *state.app.view_dirty.borrow_mut() = true;
                        }
                    }
                    if let Some(ref mcp) = state.desktop.mcp_shared {
                        let mut handle = mcp.lock().unwrap();
                        handle.set_layout_bounds(bounds_map);
                        // Plan 314 Task 5: push a serializable snapshot of the
                        // live VTree + computed cache (bounds just backfilled
                        // above) into SharedState, so the `autoui_vtree` MCP tool
                        // can return the runtime VTree as Atom WITHOUT opening
                        // F12. Done here — not in view() — so the freshly-measured
                        // `bounds` are included. `live_vtree`/`live_cache` are
                        // populated by view() under the same `capture_debug` gate
                        // (Task 4); if the panel/MCP just started they may still be
                        // None on the very first round-trip, in which case we skip
                        // (the tool degrades to "UI not yet rendered").
                        // Plan 483 D4：vtree 源取 mcp_sync_vtree（与
                        // shared.view 同一份裸 View 建的快照）——此前取
                        // live_vtree（convert_view_messages 加工树，Tabs 族
                        // 折 Empty）会造成 styled_vtree 与 shared.view 的
                        // path 空间错位，MCP vnode 派发落错节点。缓存缺失
                        // （同步块未跑过）时退回 live_vtree（旧行为）。
                        let vtree_borrow = state.app.mcp_sync_vtree.borrow();
                        let live_borrow = state.app.live_vtree.borrow();
                        let cache_borrow = state.app.live_cache.borrow();
                        let vtree_src = vtree_borrow.as_ref().or(live_borrow.as_ref());
                        if let (Some(vtree), Some(cache)) = (vtree_src, cache_borrow.as_ref()) {
                            let snap = crate::ui::mcp_server::StyledNodeSnapshot::from_live(
                                state.component.widget_name(),
                                vtree,
                                cache,
                            );
                            handle.set_styled_vtree(snap);
                        }
                    }
                }
            }
            return iced::Task::none();
        }

        // Handle screenshot request from MCP thread (Plan 285 / Task 20)
        if let Some(req) = state.app.devtools.screenshot_request.borrow_mut().take() {
            // Desktop virtual-app entries may have no physical window size.
            // The shared screenshot request can be drained by any such entry;
            // validate the actual iced window in the size() continuation below
            // instead of rejecting a healthy host from this app's bookkeeping.
            let reply_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(req.reply_tx)));
            // Clone the optionals into the outer move closure; `name` is cloned
            // again into the inner `then` (the outer closure is FnMut).
            let name = req.name.clone();
            let baseline = req.baseline;
            let diff = req.diff;
            let threshold = req.threshold;
            return iced::window::oldest()
                .then(move |maybe_id: Option<iced::window::Id>| {
                    match maybe_id {
                        Some(id) => {
                            let reply_tx = reply_tx.clone();
                            let name = name.clone();
                            // PLAN-659 T-03：size 二段跳守卫——前置
                            // window_size 借记是快照值，跳变（最小化/关窗
                            // 竞态）后 iced_wgpu offscreen 仍会以零维
                            // create_texture panic（audit 在案 Dimension X
                            // is zero ×4）。截图 Task 派发前以窗口实时
                            // 尺寸复核，零维降级为回错。
                            iced::window::size(id).then(move |size| {
                                if size.width <= 0.0 || size.height <= 0.0 {
                                    let tx = reply_tx.lock().unwrap().take().unwrap();
                                    let _ = tx.send(Err(
                                        "Screenshot skipped: window size is zero (minimized or not yet laid out)".to_string(),
                                    ));
                                    return iced::Task::none();
                                }
                                // 内层 then 再各取一份所有权（FnMut 闭包捕获
                                // 变量不能跨两级 move）。
                                let reply_tx = reply_tx.clone();
                                let name = name.clone();
                                iced::window::screenshot(id)
                                    .then(move |ss: iced::window::Screenshot| {
                                        let result = process_screenshot(
                                            &ss, &name, baseline, diff, threshold,
                                        );
                                        let tx = reply_tx.lock().unwrap().take().unwrap();
                                        let _ = tx.send(result);
                                        iced::Task::none()
                                    })
                            })
                        }
                        None => {
                            let tx = reply_tx.lock().unwrap().take().unwrap();
                            let _ = tx.send(Err("No window found".to_string()));
                            iced::Task::none()
                        }
                    }
                });
        }

        // NOTE: A `ui_changed` flag once lived here to mark UI-only state changes
        // (hover/select/debug) as needing a view rebuild. Every assignment sat on
        // an early-return path, so the flag was always `false` by the time it was
        // read below — i.e. dead. It has been removed; view rebuilds now hinge
        // solely on `component.is_dirty()`. If a UI-only change ever needs to
        // force a rebuild, set `*state.app.view_dirty.borrow_mut() = true;` directly
        // (the field that flag used to feed) on the relevant path.

        // Handle debug mode messages
        if msg.event == DEBUG_TOGGLE_EVENT {
            state.app.devtools.debug_mode = !state.app.devtools.debug_mode;
            if state.app.devtools.debug_mode {
                // Opening: show DevTools panel
                *state.app.devtools.devtools_open.borrow_mut() = true;
            } else {
                // Closing: clear all debug state
                *state.app.devtools.hovered_widget.borrow_mut() = None;
                *state.app.devtools.selected_widget.borrow_mut() = None;
                *state.app.devtools.selected_vnode.borrow_mut() = None;
                *state.app.devtools.hovered_vnode.borrow_mut() = None;
                // Plan 309 Phase 5: inspect cursor mode is a sub-state of debug
                // mode — reset it whenever F12 turns debug off.
                *state.app.devtools.inspect_mode.borrow_mut() = false;
                *state.app.devtools.devtools_open.borrow_mut() = false;
                state.app.devtools.pending_hovers.borrow_mut().clear();
                // PLAN-646: F12 关面板顺带清 marquee 在途态（结果保留）。
                *state.app.devtools.marquee.borrow_mut() = None;
                *state.app.devtools.pending_selection.borrow_mut() = None;
            }
            // Plan 371: force view rebuild so the DevTools panel appears/disappears.
            *state.app.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }

        // Plan 057 (ash-gui): global uncaptured Tab → focus the prompt editor
        // (terminal-style). Resolve the focus Id the same way the update-tail
        // refocus does (last-edited textarea's stable Id, falling back to
        // prompt_input_id) — must return the Task directly; an early
        // Task::none() would skip the tail and never focus.
        //
        // Plan 491（语义收窄）:未捕获 Tab/Shift+Tab 的生产派发已改
        // `__focus_next_input`/`__focus_prev_input`(焦点环遍历,见下臂);
        // 本臂保留两职——旧事件名外部注入兼容,及**无 input 视图**(纯
        // textarea 世界,ash-gui/028)的 next/prev 回落:登记表空时遍历
        // 无对象,保持 057 的「Tab 聚焦 prompt 编辑器」原语义不回归。
        if msg.event == FOCUS_PROMPT_EVENT
            || ((msg.event == FOCUS_NEXT_INPUT_EVENT || msg.event == FOCUS_PREV_INPUT_EVENT)
                && state.app.devtools.input_ids.borrow().is_empty())
        {
            // Last-edited textarea → any rendered textarea (first launch,
            // before any edit) → single-line input fallback.
            let id = state
                .app
                .devtools
                .last_textarea_key
                .borrow()
                .as_ref()
                .map(|k| iced::widget::Id::from(format!("textarea_{}", k)))
                .or_else(|| {
                    TEXTAREA_CONTENTS
                        .lock()
                        .unwrap()
                        .keys()
                        .next()
                        .map(|k| iced::widget::Id::from(format!("textarea_{}", k)))
                })
                .or_else(|| state.app.devtools.input_ids.borrow().first().cloned())
                .unwrap_or_else(|| state.app.devtools.prompt_input_id.clone());
            return iced::widget::operation::focus(id);
        }
        // Plan 491: 未捕获 Tab/Shift+Tab → input 焦点环遍历(483 登记表
        // DFS 序=视觉树序)。两段链:FindFocusedInput 探针读**实际**持焦者
        // (含点击直聚——iced 置焦都在 widget Tree 状态里,operate 可问),
        // focus_traverse 按方向求址(回环;不在表内/无聚焦 → 首项),再以
        // 内建 focus operation 置焦(unfocus 其余)。登记表空已在上臂回落,
        // 此处必非空 → focus_traverse 恒 Some。
        if msg.event == FOCUS_NEXT_INPUT_EVENT || msg.event == FOCUS_PREV_INPUT_EVENT {
            let forward = msg.event == FOCUS_NEXT_INPUT_EVENT;
            let registry: Vec<iced::widget::Id> = state.app.devtools.input_ids.borrow().clone();
            return iced::advanced::widget::operate(FindFocusedInput::new()).then(
                move |current: Option<iced::widget::Id>| match focus_traverse(
                    &registry,
                    current.as_ref(),
                    forward,
                ) {
                    Some(id) => iced::widget::operation::focus(id),
                    None => iced::Task::none(),
                },
            );
        }
        // Handle click-to-select: set selected element and open DevTools panel
        if let Some(id) = msg.event.strip_prefix(DEBUG_SELECT_PREFIX) {
            let id = id.to_string();
            // Toggle: if clicking the same element, deselect
            if state.app.devtools.selected_widget.borrow().as_deref() == Some(id.as_str()) {
                *state.app.devtools.selected_widget.borrow_mut() = None;
                // Plan 307 Task 17: keep selected_vnode in sync with selected_widget.
                // The live_cache holds the last frame's VNodeId <-> iced id map,
                // which is valid for selection (selection persists across frames).
                *state.app.devtools.selected_vnode.borrow_mut() = None;
                // Don't close panel on deselect — user may want to inspect other tabs
            } else {
                *state.app.devtools.selected_widget.borrow_mut() = Some(id.clone());
                // Plan 307 Task 17: derive selected_vnode from the aura_N string
                // via the last frame's live_cache so the left-tree highlight and
                // inspector panels (keyed on VNodeId) follow the click.
                let derived_vnode = state
                    .app
                    .live_cache
                    .borrow()
                    .as_ref()
                    .and_then(|c| c.iced_to_vnode(&id));
                *state.app.devtools.selected_vnode.borrow_mut() = derived_vnode;
                *state.app.devtools.devtools_open.borrow_mut() = true;
                *state.app.devtools.devtools_tab.borrow_mut() = DevToolsTab::Inspect;
                // Cache source code (shared loader; Plan 309 Phase 4.1).
                ensure_source_loaded(state.as_ref_view());
                // Plan 309 续篇: 检视光标改为常驻 —— 点击后不再自动退出，便于连点
                // 多个画布元素；由 🔍 按钮手动关闭。
            }
            // Try to set pending scroll from element's span
            if let Some(ref sel_id) = *state.app.devtools.selected_widget.borrow() {
                let styles = state.app.devtools.debug_element_styles.borrow();
                if let Some(elem_info) = styles.get(sel_id) {
                    if let Some((offset, _len)) = elem_info.span {
                        let line_offsets = state.app.source_line_offsets.borrow();
                        let line_idx = line_offsets
                            .partition_point(|&pos| pos <= offset)
                            .saturating_sub(1);
                        *state.app.devtools.pending_scroll_to_center.borrow_mut() = Some(line_idx);
                    }
                }
            }
            return iced::Task::none();
        }
        // Handle VNode selection from the live VTree (Plan 307 Task 14).
        if let Some(id_str) = msg.event.strip_prefix(DEBUG_SELECT_VNODE_PREFIX) {
            if let Ok(raw) = id_str.parse::<u64>() {
                let vnode_id = crate::ui::vnode::VNodeId::new(raw);
                if *state.app.devtools.selected_vnode.borrow() == Some(vnode_id) {
                    // Toggle off on re-click (matches the old id-string behavior).
                    *state.app.devtools.selected_vnode.borrow_mut() = None;
                    // Plan 307 Task 17: keep selected_widget in sync so the
                    // wrap_debug overlay (keyed on the aura_N string) clears too.
                    *state.app.devtools.selected_widget.borrow_mut() = None;
                } else {
                    *state.app.devtools.selected_vnode.borrow_mut() = Some(vnode_id);
                    // Plan 307 Task 17: mirror selected_widget from the live_cache
                    // reverse map (VNodeId -> aura_N) so the wrap_debug orange
                    // overlay and source-click paths stay consistent with the
                    // tree selection. If no mapping exists yet (e.g. first frame),
                    // leave selected_widget as-is — the overlay simply won't draw
                    // until the next frame builds the map.
                    let mirrored_widget = state
                        .app
                        .live_cache
                        .borrow()
                        .as_ref()
                        .and_then(|c| c.vnode_to_iced(vnode_id))
                        .cloned();
                    if let Some(aura) = mirrored_widget {
                        *state.app.devtools.selected_widget.borrow_mut() = Some(aura);
                    }
                    *state.app.devtools.devtools_open.borrow_mut() = true;
                    *state.app.devtools.devtools_tab.borrow_mut() = DevToolsTab::Inspect;
                    // Plan 309 Phase 4: load source so the Source sub-tab can
                    // render the listing on a tree-click (no element click yet).
                    ensure_source_loaded(state.as_ref_view());
                    // Plan 309 Phase 4.3: auto-scroll the Source tab to the
                    // selected node's line (the deferred-scroll path at the
                    // bottom of update() only covers selected_widget spans).
                    let scroll_line = state.app.live_vtree.borrow().as_ref().and_then(|tree| {
                        tree.get(vnode_id).and_then(|node| {
                            node.source_span.map(|span| {
                                state
                                    .app
                                    .source_line_offsets
                                    .borrow()
                                    .partition_point(|&p| p <= span.offset)
                                    .saturating_sub(1)
                            })
                        })
                    });
                    if let Some(line) = scroll_line {
                        *state.app.devtools.pending_scroll_to_center.borrow_mut() = Some(line);
                    }
                }
            }
            return iced::Task::none();
        }
        // Switch the inspector right-panel inner sub-tab (Plan 307 Task 15).
        if let Some(tail) = msg.event.strip_prefix(DEBUG_INSPECTOR_SUBTAB_PREFIX) {
            if let Some(sub) = InspectorSubTab::from_message_tail(tail) {
                *state.app.devtools.inspector_subtab.borrow_mut() = sub;
            }
            return iced::Task::none();
        }
        // Toggle a collapsible section inside the 检视 sub-tab (Plan 307 续篇 IV).
        if let Some(tail) = msg.event.strip_prefix(DEBUG_INSPECTOR_SECTION_PREFIX) {
            {
                let mut s = state.app.devtools.inspector_sections.borrow_mut();
                match tail {
                    "box" => s.box_collapsed = !s.box_collapsed,
                    "computed" => s.computed_collapsed = !s.computed_collapsed,
                    "props" => s.props_collapsed = !s.props_collapsed,
                    _ => {}
                }
            }
            return iced::Task::none();
        }
        match msg.event.as_str() {
            // Plan 309 续篇: 元素树与检视已合并为同屏分屏 (Inspect 模式)，
            // 不再有独立的元素/检视 tab；__tab_console 在控制台与分屏间切换。
            "__tab_console" => {
                let cur = *state.app.devtools.devtools_tab.borrow();
                *state.app.devtools.devtools_tab.borrow_mut() = if cur == DevToolsTab::Console {
                    DevToolsTab::Inspect
                } else {
                    DevToolsTab::Console
                };
                return iced::Task::none();
            }
            // PLAN-646: 切到 Select（采集）标签页。
            "__tab_select" => {
                *state.app.devtools.devtools_tab.borrow_mut() = DevToolsTab::Select;
                return iced::Task::none();
            }
            // PLAN-646: Select 标签页三视图切换（`__select_view_<label>`）。
            e if e.starts_with(SELECT_VIEW_PREFIX) => {
                match &e[SELECT_VIEW_PREFIX.len()..] {
                    "Auto" => {
                        *state.app.devtools.select_view.borrow_mut() =
                            crate::ui::selection::SelectionFormat::Auto
                    }
                    "Json" | "JSON" => {
                        *state.app.devtools.select_view.borrow_mut() =
                            crate::ui::selection::SelectionFormat::Json
                    }
                    "Atom" => {
                        *state.app.devtools.select_view.borrow_mut() =
                            crate::ui::selection::SelectionFormat::Atom
                    }
                    _ => {}
                }
                *state.app.view_dirty.borrow_mut() = true;
                return iced::Task::none();
            }
            // PLAN-646: 复制当前视图全文到剪贴板（按钮态反馈）。
            "__select_copy" => {
                let fmt = *state.app.devtools.select_view.borrow();
                let text = state
                    .app
                    .devtools
                    .selection_result
                    .borrow()
                    .as_ref()
                    .map(|r| r.render(fmt));
                let ok = match text {
                    Some(t) => {
                        {
                            crate::ui::clipboard::clipboard_set(&t)
                        }
                    }
                    None => false,
                };
                *state.app.devtools.select_copy_feedback.borrow_mut() = Some(ok);
                *state.app.view_dirty.borrow_mut() = true;
                return iced::Task::none();
            }
            // PLAN-646: Esc = 关闭采集面板并清 marquee（结果保留，重开可看）。
            "__select_esc" => {
                let selecting = state.app.devtools.selection_result.borrow().is_some()
                    || state.app.devtools.marquee.borrow().is_some();
                if selecting {
                    *state.app.devtools.marquee.borrow_mut() = None;
                    *state.app.devtools.pending_selection.borrow_mut() = None;
                    *state.app.devtools.devtools_open.borrow_mut() = false;
                    *state.app.view_dirty.borrow_mut() = true;
                }
                return iced::Task::none();
            }
            "__close_devtools" => {
                *state.app.devtools.devtools_open.borrow_mut() = false;
                // Plan 309 Phase 5: closing the panel also exits the picker so
                // no always-on overlay renders behind a closed panel.
                *state.app.devtools.inspect_mode.borrow_mut() = false;
                // PLAN-646: 关面板顺带清 marquee 在途态（结果保留）。
                *state.app.devtools.marquee.borrow_mut() = None;
                *state.app.devtools.pending_selection.borrow_mut() = None;
                return iced::Task::none();
            }
            // Plan 309 Phase 5.1: Chrome-style inspect-element cursor toggle.
            // Turning it on also forces debug mode + opens the panel so the
            // picker is usable from a single click; turning off just clears it.
            "__toggle_inspect" => {
                let new_mode = !*state.app.devtools.inspect_mode.borrow();
                *state.app.devtools.inspect_mode.borrow_mut() = new_mode;
                if new_mode {
                    state.app.devtools.debug_mode = true;
                    *state.app.devtools.devtools_open.borrow_mut() = true;
                }
                return iced::Task::none();
            }
            // Plan 309 续篇: 内层 Tree|Inspector 分隔栏按下 → 进入拖拽。实际
            // 位移由窗口级 `__mouse_moved` 订阅用绝对坐标计算（同外层分隔栏）。
            "__inner_divider_press" => {
                *state.app.devtools.dragging_inner_divider.borrow_mut() = true;
                return iced::Task::none();
            }
            // Source line click in Inspector: reverse-lookup AuraNodeId → debug element ID
            e if e.starts_with(SRC_CLICK_PREFIX) => {
                if let Ok(line) = e[SRC_CLICK_PREFIX.len()..].parse::<usize>() {
                    let line_map = state.app.line_to_aura_ids.borrow();
                    if let Some(aura_ids) = line_map.get(&line) {
                        // Pick the last (innermost) AuraNodeId for this line
                        if let Some(&aura_id) = aura_ids.last() {
                            let cache = state.app.aura_to_id_cache.borrow();
                            if let Some(debug_id) = cache.get(&aura_id).cloned() {
                                drop(cache);
                                drop(line_map);
                                *state.app.devtools.selected_widget.borrow_mut() =
                                    Some(debug_id.clone());
                                // Plan 309 Phase 4.2: derive selected_vnode from
                                // the aura_N id so the right panel (keyed on
                                // VNodeId) shows the clicked line's full data —
                                // without this the panel stayed empty after a
                                // source-line click.
                                let derived_vnode = state
                                    .app
                                    .live_cache
                                    .borrow()
                                    .as_ref()
                                    .and_then(|c| c.iced_to_vnode(&debug_id));
                                *state.app.devtools.selected_vnode.borrow_mut() = derived_vnode;
                                *state.app.devtools.devtools_open.borrow_mut() = true;
                                *state.app.devtools.devtools_tab.borrow_mut() =
                                    DevToolsTab::Inspect;
                                // Scroll source to the selected element's span
                                let styles = state.app.devtools.debug_element_styles.borrow();
                                if let Some(elem_info) = styles.get(&debug_id) {
                                    if let Some((offset, _len)) = elem_info.span {
                                        let line_offsets = state.app.source_line_offsets.borrow();
                                        let line_idx = line_offsets
                                            .partition_point(|&pos| pos <= offset)
                                            .saturating_sub(1);
                                        *state.app.devtools.pending_scroll_to_center.borrow_mut() =
                                            Some(line_idx);
                                    }
                                }
                            }
                        }
                    }
                }
                return iced::Task::none();
            }
            // Window resize: track current window size for panel width clamping.
            // Only trigger view rebuild when devtools is visible (panel width clamping matters).
            // For normal apps without devtools, Iced handles layout recalculation internally
            // and we don't need to rebuild the entire AbstractView + Element tree.
            "__window_resized" => {
                if let Some(ref val) = msg.input_value {
                    if let Some((w, h)) = val.split_once('x') {
                        let w: f32 = w.parse().unwrap_or(1600.0);
                        let h: f32 = h.parse().unwrap_or(900.0);
                        // PLAN-530 步骤2 表面追踪：resize 事件轨迹。
                        if std::env::var("P530_TRACE").as_deref() == Ok("1") {
                            eprintln!(
                                "[P530-TRACE] __window_resized: {w}x{h} (widget={})",
                                msg.widget
                            );
                        }
                        *state.window_size.borrow_mut() = iced::Size::new(w, h);
                        // PLAN-046-B: keep .at-visible viewport height fresh.
                        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                            "vm.window_inner_height",
                            format!("{}", h),
                        );
                        // Clamp panel width to not exceed 80% of window
                        let max_pw = w * 0.8;
                        let pw = *state.app.devtools.devtools_panel_width.borrow();
                        if pw > max_pw {
                            *state.app.devtools.devtools_panel_width.borrow_mut() = max_pw;
                        }
                        // Plan 409 §10 续 11: resize 时重建 view,让响应式布局
                        // (如 category grid 列数)随窗口宽度更新。
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
                return iced::Task::none();
            }
            // Divider drag: press
            "__divider_press" => {
                *state.app.devtools.dragging_divider.borrow_mut() = true;
                return iced::Task::none();
            }
            // Mouse move: update panel width when dragging the OUTER divider, or
            // the inner Tree|Inspector split ratio when dragging the INNER divider.
            "__mouse_moved" => {
                if let Some(ref val) = msg.input_value {
                    let (mx, my) = {
                        let mut it = val.split(',');
                        let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                        let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                        (x, y)
                    };
                    // PLAN-646: cursor 追踪（起笔锚点源）+ marquee 拖拽跟随。
                    // 拖拽中置 view_dirty——下一帧根 Stack 注入 marquee 蒙层。
                    state.app.devtools.last_cursor.set((mx, my));
                    let marquee_active = state.app.devtools.marquee.borrow().is_some();
                    if marquee_active {
                        if let Some(m) = state.app.devtools.marquee.borrow_mut().as_mut() {
                            m.update(mx, my);
                        }
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                    if *state.app.devtools.dragging_divider.borrow() {
                        let win_w = state.window_size.borrow().width;
                        let new_width = (win_w - mx).max(200.0).min(win_w - 200.0);
                        *state.app.devtools.devtools_panel_width.borrow_mut() = new_width;
                    }
                    // Plan 309 续篇: inner Tree|Inspector divider. The panel's
                    // left edge sits at win_w - panel_width; the divider's share
                    // of the panel is (mx - panel_left) / panel_width.
                    if *state.app.devtools.dragging_inner_divider.borrow() {
                        let win_w = state.window_size.borrow().width;
                        let panel_w = (*state.app.devtools.devtools_panel_width.borrow()).max(1.0);
                        let panel_left = win_w - panel_w;
                        let ratio = ((mx - panel_left) / panel_w).clamp(0.1, 0.9);
                        *state.app.devtools.inspector_split_ratio.borrow_mut() = ratio;
                    }
                }
                return iced::Task::none();
            }
            // Mouse release: stop dragging either divider
            "__mouse_released" => {
                if *state.app.devtools.dragging_divider.borrow() {
                    *state.app.devtools.dragging_divider.borrow_mut() = false;
                }
                if *state.app.devtools.dragging_inner_divider.borrow() {
                    *state.app.devtools.dragging_inner_divider.borrow_mut() = false;
                }
                // PLAN-646: marquee 定稿——死区内 = 点击清态；实拖 = 暂存矩形
                // 并请求一轮 bounds 采集（__bounds_collected 臂延迟计算信封
                // + 打开 Select 标签页）。
                if let Some(m) = state.app.devtools.marquee.borrow_mut().take() {
                    if std::env::var("AUTO_SELECT_TRACE").is_ok() {
                        eprintln!("[PLAN-646] released, drag={}", m.is_drag());
                    }
                    if m.is_drag() {
                        *state.app.devtools.pending_selection.borrow_mut() = Some(m.rect());
                        *state.app.devtools.needs_bounds.borrow_mut() = true;
                    }
                    *state.app.view_dirty.borrow_mut() = true;
                }
                return iced::Task::none();
            }
            // Plan 309 续篇 II: keyboard modifiers changed (e.g. Alt press/
            // release). T4c 裁定 M1：载荷携带 modifiers.bits()，此处写回
            // DesktopState.current_modifiers（唯一事实源，view 直读）；
            // 返回 Task::none() 仅强制重建，让 widget 翻转交互/非交互态。
            "__modifiers_changed" => {
                if let Some(bits) = msg
                    .input_value
                    .as_deref()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    *state.desktop.current_modifiers.borrow_mut() =
                        iced::keyboard::Modifiers::from_bits_truncate(bits);
                }
                return iced::Task::none();
            }
            // --- Edit mode messages (E4) ---
            e if e == DEBUG_EDIT_CANCEL => {
                *state.app.devtools.editing_element.borrow_mut() = None;
                *state.app.devtools.edit_textarea_key.borrow_mut() = None;
                *state.app.devtools.edit_span.borrow_mut() = None;
                *state.app.devtools.edit_error.borrow_mut() = None;
                return iced::Task::none();
            }
            e if e == DEBUG_EDIT_APPLY => {
                apply_edit(state);
                return iced::Task::none();
            }
            _ => {}
        }
        // Enter edit mode: __edit_{id}
        if let Some(id) = msg.event.strip_prefix(DEBUG_EDIT_PREFIX) {
            let id = id.to_string();
            let styles = state.app.devtools.debug_element_styles.borrow();
            if let Some(info) = styles.get(&id) {
                if let Some(span) = info.span {
                    *state.app.devtools.editing_element.borrow_mut() = Some(id.clone());
                    *state.app.devtools.edit_span.borrow_mut() = Some(span);
                    *state.app.devtools.edit_error.borrow_mut() = None;
                    // Initialize textarea with source code fragment
                    if let Some(ref code) = *state.app.source_code.borrow() {
                        let (offset, len) = span;
                        if offset + len <= code.len() {
                            let fragment = &code[offset..offset + len];
                            let key = format!("__edit_{}", id);
                            get_textarea_content(&key, fragment);
                            *state.app.devtools.edit_textarea_key.borrow_mut() = Some(key);
                        }
                    }
                }
            }
            return iced::Task::none();
        }
        // Accumulate hover move messages — resolved in view() by picking smallest counter
        if let Some(payload) = msg.event.strip_prefix(DEBUG_HOVER_MOVE) {
            if let Some((counter_str, id)) = payload.split_once(':') {
                if let Ok(counter) = counter_str.parse::<usize>() {
                    state
                        .app
                        .devtools
                        .pending_hovers
                        .borrow_mut()
                        .push((counter, id.to_string()));
                }
            }
            return iced::Task::none();
        }
        // Exit: no longer used for hover tracking (kept for compatibility)
        if msg.event.starts_with(DEBUG_HOVER_EXIT) {
            return iced::Task::none();
        }

        if msg.event == HOT_RELOAD_EVENT {
            // PLAN-646/Plan 282 补线：needs_bounds 的消费点原本只在 app 消息
            // 尾部批处理臂——静默会话（无交互、MCP 心跳在问）只有 500ms 泵
            // 消息到达，走早退分支，采集器永不分发，MCP layout_bounds 常驻
            // 空（实机 trace：armed 4 次 / consumed 0 次）。泵臂代为消费，
            // 让无交互会话的 layout_bounds/styled_vtree 回路照常闭环。
            if *state.app.devtools.needs_bounds.borrow()
                && state.app.devtools.screenshot_request.borrow().is_none()
            {
                *state.app.devtools.needs_bounds.borrow_mut() = false;
                use crate::ui::iced::LayoutCollector;
                // PLAN-781 T-11：泵臂与尾部批处理臂同构——bounds 与 pane 几何
                // 成对推送（Task::batch 保持一次 return 的早退形态）。
                return iced::Task::batch(vec![
                    iced::advanced::widget::operate(LayoutCollector::new()).map(|bounds_map| {
                        IcedMessage {
                            widget: String::new(),
                            event: "__bounds_collected".to_string(),
                            input_value: Some(
                                serde_json::to_string(&bounds_map).unwrap_or_default(),
                            ),
                        }
                    }),
                    iced::advanced::widget::operate(
                        crate::ui::iced::scroll_state_reader::ScrollStateReader::new(),
                    )
                    .map(|state_map| IcedMessage {
                        widget: String::new(),
                        event: "__mcp_scroll_states".to_string(),
                        input_value: Some(serde_json::to_string(&state_map).unwrap_or_default()),
                    }),
                ]);
            }
            // PLAN-056: 真实候选 bundle 原子 reload 泵——检测 + 独立候选
            // 构建 + typed 迁移 + 原子提交走 component.try_reload 单一入口；
            // 繁忙保旧下拍重试；失败具名拒绝保旧（绝不部分替换）。源码
            // 缓存/高亮/span 索引在"确有变化"时刷新。
            let reload_outcome = state.component.try_reload();
            let source_touched = !matches!(
                reload_outcome,
                crate::ui::dynamic::ReloadOutcome::NoChange
                    | crate::ui::dynamic::ReloadOutcome::Busy
            );
            if source_touched {
                if let Some(path) = state.component.source_path() {
                    if let Ok(code) = std::fs::read_to_string(path) {
                        // Refresh cached source code and line offsets for DevTools
                        let mut offsets = vec![0usize];
                        for (i, ch) in code.char_indices() {
                            if ch == '\n' {
                                offsets.push(i + 1);
                            }
                        }
                        *state.app.source_line_offsets.borrow_mut() = offsets;
                        *state.app.source_code.borrow_mut() = Some(code.clone());
                        // Rebuild syntax highlight cache after hot-reload
                        if let Some(ref c) = *state.app.source_code.borrow() {
                            *state.app.devtools.cached_highlighted.borrow_mut() =
                                Some(build_highlight_cache(c));
                        }
                        // Invalidate caches since view_template changed
                        *state.app.cached_converted_view.borrow_mut() = None;
                        *state.app.devtools.cached_debug_id_map.borrow_mut() = None;
                        // Rebuild line → AuraNodeId index after hot-reload
                        {
                            let span_map = state.component.span_map().clone();
                            if let Some(ref src) = *state.app.source_code.borrow() {
                                *state.app.line_to_aura_ids.borrow_mut() =
                                    build_line_to_aura_ids(&span_map, src);
                            }
                        }
                    }
                }
            }
            if let crate::ui::dynamic::ReloadOutcome::Rejected(ref why) = reload_outcome {
                eprintln!(
                    "[VM-RELOAD] {} rejected (old bundle keeps serving): {}",
                    state.component.widget_name(),
                    why
                );
            }
            if let crate::ui::dynamic::ReloadOutcome::Error(ref why) = reload_outcome {
                eprintln!(
                    "[VM-RELOAD] {} error (old bundle keeps serving): {}",
                    state.component.widget_name(),
                    why
                );
            }
            // PLAN-536 T2(题1 兜底桥): 热重载拍是 source_path 在案时的常驻
            // 500ms update 泵。update 间隙落到 state 的脏写(timer handler
            // 异步回填/native 完成回调等,不经过任何 update 派发)由此桥接进
            // view 失效通路——否则 update 尾部的 is_dirty→view_dirty 回填
            // 永远看不到它们,画布最坏冻在最后一帧。干净拍零写入零 dirty。
            if state.component.is_dirty() {
                *state.app.view_dirty.borrow_mut() = true;
            }
            return iced::Task::none();
        }
        // Handle periodic tick events (stopwatch, timers)
        if msg.event == TICK_EVENT {
            // Plan 402: dispatch Tick to the handler unconditionally — the
            // store/handler itself decides whether to act (e.g. minesweeper
            // only increments elapsed when game_state == "playing"). The old
            // code gated on a stopwatch-specific `running == "true"` field
            // which minesweeper doesn't have, so Tick never fired.
            // Stopwatch compatibility: still do the running check + elapsed
            // formatting for widgets that DO have a `running` field.
            state.component.on_with_input("Tick", None);
            let running = state
                .component
                .read_state("running")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if running == "true" {
                if let Ok(elapsed) = state.component.read_state("elapsed").map(|v| v.as_int()) {
                    let total_cs = elapsed / 10; // centiseconds
                    let cs = total_cs % 100;
                    let total_secs = total_cs / 100;
                    let secs = total_secs % 60;
                    let mins = total_secs / 60;
                    let time_display = format!("{:02}:{:02}", mins, secs);
                    let ms_display = format!(".{:02}", cs);
                    let _ = state
                        .component
                        .write_state("time_display", auto_val::Value::str(&time_display));
                    let _ = state
                        .component
                        .write_state("ms_display", auto_val::Value::str(&ms_display));
                }
            }
            *state.app.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }

        // ── Shell SSE bridge (ash-gui M1) ──────────────────────────────
        // subscription 把 command_output/command_result 事件送成 IcedMessage
        // (widget="ShellStore",input_value=完整 JSON)。VM 无法收 struct 参
        // (push_value 对 Obj 推占位 0),故此处先 write_state 预置字段,
        // 再以**无参**触发 ShellStore 的 RunOutput/RunResult handler —— handler
        // body 读预置字段(shell_store.at 的 `__sse_*`)。
        // Plan 057:job_started/job_done 也走此通道(预置 __sse_job_* 后触发
        // 无参 JobStarted/JobDone handler,对齐 vue codegen 的 SSE dispatch)。
        if msg.event == "command_output"
            || msg.event == "command_result"
            || msg.event == "job_started"
            || msg.event == "job_done"
            || msg.event == "job_list"
            || msg.event == "ai_turn"
            || msg.event == "ai_chunk"
            || msg.event == "ai_tool_call"
            || msg.event == "ai_tool_result"
            || msg.event == "chat_cleared"
        {
            if let Some(json) = &msg.input_value {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                    match msg.event.as_str() {
                        "job_started" | "job_done" => {
                            // Plan 057:job_list 由 renderer 直接用 Rust 维护
                            // (镜像 blocks 的 update_block_in_state 模式)—— VM 的
                            // JobStarted/JobDone handler 需迭代读 .job_list 字段,
                            // 受 handler 内字段读(GET_FIELD 编码)问题影响不可靠。
                            // vue 端仍走 .at handler(增量 push/remove)。
                            let job_id = v.get("job_id").and_then(|x| x.as_i64()).unwrap_or(0);
                            let job_cmd = v
                                .get("cmd")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let raw = state.component.read_state("job_list").ok();
                            let mut jobs_vec: Vec<auto_val::Value> = match &raw {
                                Some(auto_val::Value::Array(arr)) => arr.values.clone(),
                                Some(auto_val::Value::Nil) | None => Vec::new(),
                                _ => state
                                    .component
                                    .read_state_as_vec("job_list")
                                    .unwrap_or_default(),
                            };
                            if msg.event == "job_started" {
                                let mut ji = auto_val::Obj::new();
                                ji.set("id", auto_val::Value::Int(job_id as i32));
                                ji.set("command", auto_val::Value::str(&job_cmd));
                                ji.set("state", auto_val::Value::str("Running"));
                                ji.set("exit_code", auto_val::Value::Int(0));
                                jobs_vec.push(auto_val::Value::Obj(Box::new(ji)));
                            } else {
                                // job_done:按 id 移除(对齐 .at JobDone handler 语义)。
                                jobs_vec.retain(|j| {
                                    if let auto_val::Value::Obj(obj) = j {
                                        obj.get("id")
                                            .map(|x| x.as_int() as i64 != job_id)
                                            .unwrap_or(true)
                                    } else {
                                        true
                                    }
                                });
                            }
                            // Plan 062(2026-08-23):无条件写 renderer-owned
                            // Value::Array —— store 声明的 List<JobInfo> 是 VM
                            // 原生列表(VmRef),write_state_vec 写不回 VM 堆对象
                            // (读回恒为同一 VmRef、条目不可见)。blocks 之所以
                            // 正常,正因 renderer 从第一块起就以 Value::Array
                            // 形态持有;job_list 对齐同一所有权模型。
                            let _ = state.component.write_state(
                                "job_list",
                                auto_val::Value::Array(auto_val::Array { values: jobs_vec }),
                            );
                            *state.app.view_dirty.borrow_mut() = true;
                        }
                        "ai_turn" | "ai_chunk" | "ai_tool_call" | "ai_tool_result"
                        | "chat_cleared" => {
                            // Plan 063 T4/T5:AI chat 抽屉事件族。chat_events 由
                            // renderer 直接用 Rust 维护(job_list 同款所有权模型:
                            // store 声明的 List 是 VM 堆 VmRef,write_state_vec
                            // 写不回;无条件写 renderer-owned Value::Array)。
                            // vue 端走 .at 无参 handler 读 __ai_* 预置字段。
                            // ai_turn 另把回合号写进块头横幅字段 block.turn。
                            let bid = v.get("block_id").and_then(|x| x.as_i64()).unwrap_or(-1);
                            if msg.event == "chat_cleared" {
                                let _ = state.component.write_state(
                                    "chat_events",
                                    auto_val::Value::Array(auto_val::Array { values: Vec::new() }),
                                );
                                *state.app.view_dirty.borrow_mut() = true;
                                return iced::Task::none();
                            }
                            let turn = v.get("turn").and_then(|x| x.as_i64()).unwrap_or(0);
                            let (kind, text) = match msg.event.as_str() {
                                "ai_turn" => {
                                    let q = v
                                        .get("question")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let mut lines: Vec<auto_val::Value> = Vec::new();
                                    let mut sep = auto_val::Obj::new();
                                    sep.set("kind", auto_val::Value::str("turn"));
                                    sep.set(
                                        "text",
                                        auto_val::Value::str(&format!("── 第 {turn} 轮 ──")),
                                    );
                                    lines.push(auto_val::Value::Obj(Box::new(sep)));
                                    let mut u = auto_val::Obj::new();
                                    u.set("kind", auto_val::Value::str("user"));
                                    u.set("text", auto_val::Value::str(&q));
                                    lines.push(auto_val::Value::Obj(Box::new(u)));
                                    append_chat_events(&mut *state.component, lines);
                                    set_block_turn(&mut *state.component, bid, turn);
                                    *state.app.view_dirty.borrow_mut() = true;
                                    return iced::Task::none();
                                }
                                "ai_chunk" => {
                                    let t = v
                                        .get("text")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    ("text".to_string(), t)
                                }
                                "ai_tool_call" => {
                                    let tool = v
                                        .get("tool")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let args = v
                                        .get("args")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    ("tool".to_string(), format!("⚙ {tool} {args}"))
                                }
                                _ => {
                                    // ai_tool_result
                                    let tool = v
                                        .get("tool")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let res = v
                                        .get("result")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    ("result".to_string(), format!("← {tool}: {res}"))
                                }
                            };
                            let mut ev = auto_val::Obj::new();
                            ev.set("kind", auto_val::Value::str(&kind));
                            ev.set("text", auto_val::Value::str(&text));
                            append_chat_events(
                                &mut *state.component,
                                vec![auto_val::Value::Obj(Box::new(ev))],
                            );
                            *state.app.view_dirty.borrow_mut() = true;
                        }
                        _ => {
                            // command_output:{block_id,chunk};command_result:{block_id,cwd,status,output,duration_ms,exit_code}
                            // M1:VM 的 RunOutput/RunResult handler 读 .blocks 读不到 renderer 写的
                            // Value::Array(renderer↔vm state 类型不同步),故在此直接用 Rust 更新
                            // store.blocks 里匹配 block_id 的 block(streamed_text / status / output)。
                            let bid = v.get("block_id").and_then(|x| x.as_i64()).unwrap_or(-1);
                            let updated =
                                update_block_in_state(&mut *state.component, bid, &msg.event, &v);
                            if msg.event == "command_result" {
                                // Plan 057:cwd 回写(cd 后标题栏/新块 cwd 立即反映)
                                // + 触发 RefreshContext 刷 git 标签(HTTP 模式下
                                // prompt_context() 走真后端;merged 模式为 mock,无害)。
                                // Windows 扩展路径前缀(\\?\D:\...)对展示无意义,剥掉。
                                if let Some(cwd) = v.get("cwd").and_then(|x| x.as_str()) {
                                    let cwd = cwd.trim_start_matches(r"\\?\");
                                    if !cwd.is_empty() {
                                        let _ = state
                                            .component
                                            .write_state("cwd", auto_val::Value::str(cwd));
                                    }
                                }
                                state.component.on_with_input_for(
                                    "ShellStore",
                                    "RefreshContext",
                                    None,
                                );
                            }
                            if updated {
                                *state.app.view_dirty.borrow_mut() = true;
                            }
                        }
                    }
                }
            }
            return iced::Task::none();
        }

        let event_name = derive_event_name(&msg.event);
        let input_cache_key = input_cache_key(state.component, &msg.widget, &event_name);

        // Save input text BEFORE on_with_input runs .at handler (which clears it for AddTodo)
        let saved_input = state
            .component
            .read_state("input")
            .map(|v| v.as_str().to_string())
            .unwrap_or_default();

        // If this message carries input text, track it and update state
        if let Some(text) = &msg.input_value {
            if let Some(key) = &input_cache_key {
                track_input_text(&mut state.app.input_values, key, text);
            }
        }

        // Plan 057 (ash-gui 输入焦点): an edit message that originates from a
        // text_editor's on_action (identified by the TEXTAREA_CONTENTS action
        // key). The handler it dispatches updates state (suggestions/highlight
        // spans), and the resulting view rebuild can shift the editor's tree
        // position — iced's positional state diff then drops editor focus.
        // Remember the source editor; the tail of update() re-focuses it by
        // its stable Id after the rebuild.
        if msg.input_value.is_some() {
            let ta_key = format!("{}_{}", msg.widget, msg.event);
            if TEXTAREA_CONTENTS.lock().unwrap().contains_key(&ta_key) {
                if std::env::var("ASH_DEBUG_FOCUS").is_ok() {
                    eprintln!("[FOCUS-DBG] edit detected, key={}", ta_key);
                }
                state.app.devtools.needs_prompt_refocus.set(true);
                *state.app.devtools.last_textarea_key.borrow_mut() = Some(ta_key.clone());
                // Plan 057 续(行内编辑):编辑器自身消息 → echo:下一次该 key
                // 的 content 重建保持光标偏移;顺带把光标字节偏移写入组件
                // cursor_pos(仅当 .at 声明了该字段),供补全按真实光标计算。
                TEXTAREA_ECHO_KEYS.lock().unwrap().insert(ta_key.clone());
                {
                    let cur_off = TEXTAREA_CONTENTS
                        .lock()
                        .unwrap()
                        .get(&ta_key)
                        .map(|c| le_cursor_offset(c) as i64)
                        .unwrap_or(0);
                    if state.component.read_state("cursor_pos").is_ok() {
                        let _ = state
                            .component
                            .write_state("cursor_pos", auto_val::Value::Int(cur_off as i32));
                    }
                }
            }
        } else {
            // Plan 057 续(keydown 失焦修复):onkeydown 派发的 handler(AcceptGhost/
            // OnRight/OnTab/历史导航…)同样会改 input/ghost/suggestions,候选行
            // 显隐造成树位移丢焦点。这些消息无 input_value,按登记映射找到所属
            // 编辑器,同样触发重建后重聚焦(光标停在 input 末尾,由内容重建负责)。
            let ta_key = format!("{}_{}", msg.widget, msg.event);
            if let Some(editor_key) = TEXTAREA_KEYDOWN_TO_EDITOR
                .lock()
                .unwrap()
                .get(&ta_key)
                .cloned()
            {
                if std::env::var("ASH_DEBUG_FOCUS").is_ok() {
                    eprintln!(
                        "[FOCUS-DBG] keydown handler {}, refocus editor {}",
                        ta_key, editor_key
                    );
                }
                state.app.devtools.needs_prompt_refocus.set(true);
                *state.app.devtools.last_textarea_key.borrow_mut() = Some(editor_key.clone());
                // 外部变更(历史/补全/ghost 接受/模式切换):清 echo,
                // 重建时光标回到 value 末尾。
                TEXTAREA_ECHO_KEYS.lock().unwrap().remove(&editor_key);
            }
        }

        // Plan 320: route event to the correct widget's handler (single VM).
        let widget_name = &msg.widget;
        // Save input_value before it's moved into on_with_input_for (ash-gui M1
        // emit simulation below reads it).
        // EDGE-15: on_submit(真实 Enter)的 msg 不带 input_value,而 PromptBar.Run
        // handler 会清空 .input。故在 handler 执行前,若 msg 无 input_value 则从
        // state.input 预先抢救当前值,供下方 emit 模拟使用(等价 mcp_server.rs:1966)。
        let saved_input_value = msg.input_value.clone().or_else(|| {
            if widget_name == "PromptBar" && event_name == "Run" {
                state
                    .component
                    .read_state("input")
                    .ok()
                    .map(|v| v.as_str().to_string())
            } else {
                None
            }
        });
        // Plan 402: track difficulty before handler (for dynamic window resize)
        let diff_before = state
            .component
            .read_state("difficulty")
            .map(|v| v.as_str().to_string())
            .unwrap_or_default();

        let msg_input_snapshot = msg.input_value.clone();
        if msg.input_value.is_some() {}
        // Plan 051 C7: timer 拍走 fire_timer（`when` 门控在派发前对根态
        // 求值，假丢弃本拍）；非 timer 事件走通用路径不变。
        // PLAN-725 T-00：S2 VM 段打点（通用派发→handler 解释）。
        let p725_t_s2 = std::time::Instant::now();
        if state.component.is_timer_entry(widget_name, &event_name) {
            state.component.fire_timer(widget_name, &event_name);
        } else if http_backfill {
            // PLAN-765 T-02（纪要一 F-4）：HTTP 消息桥回填——seq 门控派发，
            // handler 零状态写的空回填不置脏（F1 fire_timer 语义延伸）。
            state
                .component
                .on_http_backfill_for(widget_name, &event_name);
        } else {
            state
                .component
                .on_with_input_for(widget_name, &event_name, msg.input_value);
        }
        crate::ui::frame_segments::note_s2_vm(p725_t_s2.elapsed());

        // Plan 055 D5(补实现,此前缺失):BlockItem.ToggleCollapse(id) 的 .at handler
        // 是空体,设计上由 renderer emit 给父级翻转 blocks[id].collapsed —— 该桥
        // 一直没实现,VM 端点击折叠无效果。这里解码 payload 里的 id 直接翻转。
        // ── Plan 059(表格增强):BlockItem 的表格交互特例(同 ToggleCollapse
        // 模式 —— VM 剥离 child callback emit,.at handler 空体,renderer
        // 直改 blocks 状态)。Sort/Filter 的 payload 是 "block_id" /
        // "block_id:col" 编码串;Filter 的查询词在 msg.input_value。
        if widget_name == "BlockItem" && event_name.starts_with("Sort") {
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            let id = args.first().map(|v| v.as_int() as i64).unwrap_or_else(|| {
                args.first()
                    .map(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1)
            });
            let col = args.get(1).map(|v| v.as_int() as i64).unwrap_or(-1);
            if true {
                if id >= 0 {
                    if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                        for b in blocks.iter_mut() {
                            if let auto_val::Value::Obj(obj) = b {
                                if obj
                                    .get("id")
                                    .map(|v| v.as_int() as i64 == id)
                                    .unwrap_or(false)
                                {
                                    // 同列翻转 / 异列重置升序
                                    let cur_col =
                                        obj.get("table_sort_col").map(|v| v.as_int()).unwrap_or(-1);
                                    let (nc, nd) = if cur_col == col as i32 {
                                        let d = obj
                                            .get("table_sort_dir")
                                            .map(|v| v.as_int())
                                            .unwrap_or(1);
                                        (cur_col, -d)
                                    } else {
                                        (col as i32, 1)
                                    };
                                    obj.set("table_sort_col", auto_val::Value::Int(nc));
                                    obj.set("table_sort_dir", auto_val::Value::Int(nd));
                                    sort_table_rows(obj, nc, nd);
                                    break;
                                }
                            }
                        }
                        let _ = state.component.write_state_vec("blocks", blocks);
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
            }
        }
        if widget_name == "BlockItem" && event_name.starts_with("Filter") {
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            // Plan 062 T9:对齐 Sort 的 id 解析(int 优先,str 回退)——此前只走
            // str parse,int 参数解析成 -1,过滤恒不生效(059 §4.3 疑点的真身)。
            let id = args.first().map(|v| v.as_int() as i64).unwrap_or_else(|| {
                args.first()
                    .map(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1)
            });
            let query = msg_input_snapshot
                .clone()
                .unwrap_or_default()
                .to_lowercase();
            if id >= 0 {
                if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                    for b in blocks.iter_mut() {
                        if let auto_val::Value::Obj(obj) = b {
                            if obj
                                .get("id")
                                .map(|v| v.as_int() as i64 == id)
                                .unwrap_or(false)
                            {
                                obj.set("table_filter_q", auto_val::Value::str(&query));
                                let sc =
                                    obj.get("table_sort_col").map(|v| v.as_int()).unwrap_or(-1);
                                let sd = obj.get("table_sort_dir").map(|v| v.as_int()).unwrap_or(1);
                                sort_table_rows(obj, sc, sd);
                                break;
                            }
                        }
                    }
                    let _ = state.component.write_state_vec("blocks", blocks);
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
        }
        // Plan 059 → 2026-08-22 结果底栏版:复制按钮 VM 桥 —— navigator 是
        // 浏览器全局,VM 运行时不存在(.at handler 在 guard 静默中止),由
        // arboard 写系统剪贴板。CopyCommand → 命令文本;CopyOutput → Text
        // 原文 / Code 全文(streamed_text,apply 时写入)/ Table TSV(直贴
        // 表格软件);ExportCsv → Table CSV(引号转义,与 .at 侧 Vue 实现
        // 同语义)。
        if widget_name == "BlockItem"
            && (event_name.starts_with("CopyCommand")
                || event_name.starts_with("CopyOutput")
                || event_name.starts_with("ExportCsv"))
        {
            let (_, cargs) = crate::ui::dynamic::decode_payload(&msg.event);
            let cid = cargs.first().map(|v| v.as_int() as i64).unwrap_or(-1);
            if cid >= 0 {
                if let Ok(blocks) = state.component.read_state_as_vec("blocks") {
                    let mut text: Option<String> = None;
                    for b in blocks.iter() {
                        if let auto_val::Value::Obj(obj) = b {
                            if !obj
                                .get("id")
                                .map(|v| v.as_int() as i64 == cid)
                                .unwrap_or(false)
                            {
                                continue;
                            }
                            if event_name.starts_with("CopyCommand") {
                                text = obj.get("command").map(|v| v.as_str().to_string());
                            } else if let Some(auto_val::Value::Obj(out)) = obj.get("output") {
                                if let Some(auto_val::Value::Str(t)) = out.get("Text") {
                                    text = Some(t.as_str().to_string());
                                } else if let Some(auto_val::Value::Obj(tbl)) = out.get("Table") {
                                    text = if event_name.starts_with("ExportCsv") {
                                        table_to_csv(&tbl)
                                    } else {
                                        table_to_tsv(&tbl)
                                    };
                                } else if out.get("Code").is_some() {
                                    // 2026-08-22(show 底栏):Code → 复制全文
                                    // (streamed_text,update_block_in_state 在
                                    // apply Code 结果时写入;空串防御)。
                                    if let Some(auto_val::Value::Str(s)) = obj.get("streamed_text")
                                    {
                                        if !s.as_str().is_empty() {
                                            text = Some(s.as_str().to_string());
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                    if let Some(t) = text {
                        match arboard::Clipboard::new() {
                            Ok(mut cb) => {
                                if let Err(e) = cb.set_text(t) {
                                    eprintln!("[copy] set_text failed: {}", e);
                                }
                            }
                            Err(e) => eprintln!("[copy] clipboard unavailable: {}", e),
                        }
                    }
                }
            }
        }
        // Plan 059:表格单元格点击打开 → store.OpenPath(payload 携带路径)。
        if widget_name == "BlockItem" && event_name.starts_with("OpenPath") {
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            if let Some(p) = args.first().map(|v| v.as_str()).filter(|p| !p.is_empty()) {
                state
                    .component
                    .on_with_input_for("ShellStore", "OpenPath", Some(p.to_string()));
                *state.app.view_dirty.borrow_mut() = true;
            }
        }
        if widget_name == "BlockItem" && event_name.starts_with("ToggleCollapse") {
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            let id = args.first().map(|v| v.as_int() as i64).unwrap_or(-1);
            if id >= 0 {
                if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                    let mut flipped = false;
                    for b in blocks.iter_mut() {
                        if let auto_val::Value::Obj(obj) = b {
                            let matches = obj
                                .get("id")
                                .map(|v| v.as_int() as i64 == id)
                                .unwrap_or(false);
                            if matches {
                                let cur = obj
                                    .get("collapsed")
                                    .map(|v| matches!(v, auto_val::Value::Bool(true)))
                                    .unwrap_or(false);
                                obj.set("collapsed", auto_val::Value::Bool(!cur));
                                flipped = true;
                                break;
                            }
                        }
                    }
                    if flipped {
                        let _ = state.component.write_state_vec("blocks", blocks);
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
            }
        }
        // ── Plan 060 R16:child-callback 缺失桥补齐 ──────────────────────
        // vm 剥离 callback emit(handler_codegen D-GAP-4),这些按钮的 .at
        // handler 为空体、父级 handler 不发生 —— 点击无任何效果(真实鼠标
        // 与 MCP 模拟同症,2026-08-23 实证:Stop 连点 7 次 ping -n 30 跑满、
        // DeleteBlock 块数不变、ToolSidebar.Pick input 不填)。按既有惯例
        // (OpenPath/ToggleCollapse/RunCommand 后备)在 renderer 侧补桥:
        if widget_name == "BlockItem" && event_name == "Stop" {
            // Stop → ShellStore.Cancel(cancel() #[api] → host → worker kill)。
            // .at 侧 .Cancel 的 for 循环读 .blocks 为 nil(B 系列债)不会跑,
            // 这里 Rust 直改:Running → Cancelled(终态由 command_result 事件
            // 的 Cancelled 状态确认,worker 侧配套映射)。
            let _ = state
                .component
                .on_with_input_for("ShellStore", "Cancel", None);
            if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                let mut flipped = false;
                for b in blocks.iter_mut() {
                    if let auto_val::Value::Obj(obj) = b {
                        let running = obj
                            .get("status")
                            .and_then(|s| {
                                if let auto_val::Value::Obj(so) = &s {
                                    so.get("kind").map(|v| v.as_str() == "Running")
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);
                        if running {
                            let mut status = auto_val::Obj::new();
                            status.set("kind", auto_val::Value::str("Cancelled"));
                            status.set("message", auto_val::Value::str(""));
                            obj.set("status", auto_val::Value::Obj(Box::new(status)));
                            flipped = true;
                        }
                    }
                }
                if flipped {
                    let _ = state.component.write_state_vec("blocks", blocks);
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
        }
        if widget_name == "BlockItem" && event_name.starts_with("DeleteBlock") {
            // DeleteBlock(id) → Rust 直改 blocks(镜像 ToggleCollapse 模式;
            // store 的 .at 实现同样读 .blocks 为 nil,不可走 handler)。
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            let id = args.first().map(|v| v.as_int() as i64).unwrap_or(-1);
            if id >= 0 {
                if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                    let before = blocks.len();
                    blocks.retain(|b| match b {
                        auto_val::Value::Obj(obj) => {
                            obj.get("id").map(|v| v.as_int() as i64) != Some(id)
                        }
                        _ => true,
                    });
                    if blocks.len() != before {
                        let _ = state.component.write_state_vec("blocks", blocks);
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
            }
        }
        if widget_name == "BlockItem" && event_name.starts_with("Rerun") {
            // Rerun(cmd) → 复用 store.RunCommand 主路径(与 PromptBar.Run 的
            // emit 模拟同构:dispatch 后读 __pending_command_* 置换/追加块)。
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            if let Some(cmd) = args
                .first()
                .map(|v| v.as_str())
                .filter(|c| !c.trim().is_empty())
            {
                let blocks_before = state
                    .component
                    .read_state_as_vec("blocks")
                    .map(|v| v.len())
                    .unwrap_or(0);
                state.component.on_with_input_for(
                    "ShellStore",
                    "RunCommand",
                    Some(cmd.to_string()),
                );
                let bid = state
                    .component
                    .read_state("__pending_command_id")
                    .map(|v| v.as_int() as i64)
                    .unwrap_or(0);
                let cwd = state
                    .component
                    .read_state("cwd")
                    .map(|v| v.as_str().to_string())
                    .unwrap_or_default();
                if bid >= 0 {
                    let mut status = auto_val::Obj::new();
                    status.set("kind", auto_val::Value::str("Running"));
                    status.set("message", auto_val::Value::str(""));
                    let mut block = auto_val::Obj::new();
                    block.set("id", auto_val::Value::Int(bid as i32));
                    block.set("command", auto_val::Value::str(cmd));
                    block.set("cwd", auto_val::Value::str(&cwd));
                    block.set("status", auto_val::Value::Obj(Box::new(status)));
                    block.set("streamed_text", auto_val::Value::str(""));
                    block.set("duration_ms", auto_val::Value::Int(0));
                    block.set("output", auto_val::Value::Nil);
                    block.set("exit_code", auto_val::Value::Int(0));
                    block.set("collapsed", auto_val::Value::Bool(false));
                    block.set("table_sort_col", auto_val::Value::Int(-1));
                    block.set("table_sort_dir", auto_val::Value::Int(1));
                    block.set("table_filter_q", auto_val::Value::str(""));
                    // Plan 063 T5: chat 回合号(非 chat 块恒 0)。
                    block.set("turn", auto_val::Value::Int(0));
                    if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                        let store_pushed_this = blocks.len() > blocks_before;
                        if store_pushed_this {
                            *blocks.last_mut().unwrap() = auto_val::Value::Obj(Box::new(block));
                        } else {
                            blocks.push(auto_val::Value::Obj(Box::new(block)));
                        }
                        let _ = state.component.write_state_vec("blocks", blocks);
                    }
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
        }
        if widget_name == "ToolSidebar" && event_name.starts_with("Pick") {
            // Pick(name) → App.Pick 语义:injected_command + PromptBar 填充。
            // PromptBar 的 prop watcher(vm 端不可靠)跳过,直接写 root input
            // —— 与 autoui_type / Run 清空同机制,强制重建 textarea 实例。
            let (_, args) = crate::ui::dynamic::decode_payload(&msg.event);
            if let Some(name) = args.first().map(|v| v.as_str()).filter(|n| !n.is_empty()) {
                let _ = state
                    .component
                    .write_state("injected_command", auto_val::Value::str(name));
                let _ = state
                    .component
                    .write_state("input", auto_val::Value::str(name));
                *state.app.cached_rendered.borrow_mut() = None;
                *state.app.cached_converted_view.borrow_mut() = None;
                *state.app.view_dirty.borrow_mut() = true;
            }
        }

        // Plan 402: if difficulty changed (SetDifficulty/Init), resize window
        // to fit the board snugly.
        let diff_after = state
            .component
            .read_state("difficulty")
            .map(|v| v.as_str().to_string())
            .unwrap_or_default();
        if diff_before != diff_after {
            let cols = state
                .component
                .read_state("cols")
                .map(|v| v.as_int())
                .unwrap_or(9);
            let rows = state
                .component
                .read_state("rows")
                .map(|v| v.as_int())
                .unwrap_or(9);
            // Plan 402: snug window fit. cell=32px(w-8 h-8) + 2px border.
            // Width: grid + p-6 padding (48px) + a little slack.
            // Height: info-bar(~60) + difficulty row(~40) + grid + p-6(48) + mt-8(32) + spacing(~20).
            let cell = 34.0; // 32px button + 2px border
            let w = ((cols as f32) * cell + 64.0).max(320.0);
            let h = (rows as f32) * cell + 200.0;
            *state.pending_window_resize.borrow_mut() = Some(iced::Size::new(w, h));
        }

        // After handler runs, clear input_values for OTHER inputs whose state
        // fields may have been modified by the handler. For example, the
        // CelsiusChanged handler writes fahrenheit — the fahrenheit input should
        // now show the computed value, not stale user-typed text.
        // Keep only the triggering event's entry (the user just typed it).
        retain_input_values_after_handler(
            &mut state.app.input_values,
            &state.component,
            input_cache_key.as_deref().unwrap_or(&event_name),
        );

        // ── Shell bridge:emit 模拟(ash-gui M1) ──────────────────────────
        // vm 模式 handler_codegen 剥离子组件的 callback prop 调用(handler_codegen.rs
        // :996 Plan 370 D-GAP-4),故 PromptBar.Run 清 input 后的自动 emit('Run',cmd)
        // → App.RunCommand 在 vm 不发生。这里模拟该 emit:PromptBar.Run 执行后,若它
        // 带了 cmd 值(submit 传入的 input 当前值),直接触发 store.RunCommand(cmd)。
        // 这是 ash-gui 特定知识(widget=PromptBar,event=Run),而非通用 emit 修复。
        // Plan 053 M4:OnEnter(submit 目标)内部调用 Run 执行命令 — Run 清空
        // .input。若 OnEnter 消息处理后 .input 为空且携带了命令,视为命令已执行,
        // 走同一 bridge(Run 的 cmd 从 handler 参数传入,不经 IcedMessage)。
        let cmd_ran = if widget_name == "PromptBar" && event_name == "Run" {
            true
        } else if widget_name == "PromptBar" && event_name == "OnEnter" {
            state
                .component
                .read_state("input")
                .map(|v| v.as_str().is_empty())
                .unwrap_or(false)
        } else {
            false
        };
        if cmd_ran {
            // saved_input_value 在 handler 执行前已补值(见上方 EDGE-15 注释):
            // on_submit 无 input_value 时从 state.input 抢救,故此处直接用。
            if let Some(cmd) = saved_input_value.as_deref() {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    // 触发 store 级 RunCommand(cmd)。on_with_input_for 把 input_value
                    // 作为字符串参数传给 handler(cmd 参数)。
                    // store 调用前记录块数:store 的 RunCommand 会自己 push 一个
                    // Running 块(元素是 VM 堆引用,update_block_in_state 按
                    // Value::Obj 匹配会失败)。下方 Rust 用完整块**替换**这个刚推入
                    // 的尾块 —— 既消灭双块,又保证结果回写能匹配。
                    let blocks_before = state
                        .component
                        .read_state_as_vec("blocks")
                        .map(|v| v.len())
                        .unwrap_or(0);
                    state.component.on_with_input_for(
                        "ShellStore",
                        "RunCommand",
                        Some(cmd.to_string()),
                    );
                    // store.RunCommand 已写 __pending_command_{id,str}。
                    let bid = state
                        .component
                        .read_state("__pending_command_id")
                        .map(|v| v.as_int() as i64)
                        .unwrap_or(0);
                    let cwd = state
                        .component
                        .read_state("cwd")
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_default();
                    if bid >= 0 {
                        let mut status = auto_val::Obj::new();
                        status.set("kind", auto_val::Value::str("Running"));
                        status.set("message", auto_val::Value::str(""));
                        let mut block = auto_val::Obj::new();
                        block.set("id", auto_val::Value::Int(bid as i32));
                        block.set("command", auto_val::Value::str(cmd));
                        block.set("cwd", auto_val::Value::str(&cwd));
                        block.set("status", auto_val::Value::Obj(Box::new(status)));
                        block.set("streamed_text", auto_val::Value::str(""));
                        block.set("duration_ms", auto_val::Value::Int(0));
                        // Plan 053 后续:补齐 Block 其余字段 —— 缺 output 时
                        // BlockBody 渲染空;缺 exit_code/collapsed 时
                        // collapse_glyph/exit_label computed 读取失败,块头回退成
                        // "${collapse_glyph}" 字面量。
                        block.set("output", auto_val::Value::Nil);
                        block.set("exit_code", auto_val::Value::Int(0));
                        block.set("collapsed", auto_val::Value::Bool(false));
                        // Plan 059(表格增强):表格排序/过滤状态(前端字段,
                        // 服务端数据无此概念;缺失会导致 .at 读取静默中止)。
                        block.set("table_sort_col", auto_val::Value::Int(-1));
                        block.set("table_sort_dir", auto_val::Value::Int(1));
                        block.set("table_filter_q", auto_val::Value::str(""));
                        // Plan 063 T5: chat 回合号(非 chat 块恒 0)。
                        block.set("turn", auto_val::Value::Int(0));
                        // store 的 RunCommand 已 push 同 id 的块,但 VM List 里的
                        // 元素是堆引用(非 Value::Obj),update_block_in_state 按
                        // Obj 匹配会失败 → 块永远停在 Running。这里用完整 Rust 块
                        // **替换**同 id 尾块(幂等,无双块),仅当无同 id 块时追加。
                        if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                            // store 刚为本次命令 push 了一个块(计数 +1)→ 原位替换;
                            // 否则(历史路径/store 未推)追加。
                            let store_pushed_this = blocks.len() > blocks_before;
                            if store_pushed_this {
                                *blocks.last_mut().unwrap() = auto_val::Value::Obj(Box::new(block));
                            } else {
                                blocks.push(auto_val::Value::Obj(Box::new(block)));
                            }
                            let _ = state.component.write_state_vec("blocks", blocks);
                        } else {
                            let _ = state.component.write_state(
                                "blocks",
                                auto_val::Value::Array(auto_val::Array {
                                    values: vec![auto_val::Value::Obj(Box::new(block))],
                                }),
                            );
                        }
                        // Plan 060(M1-T3):执行提交已退役 —— shell.at 的
                        // run_command(经 api.at,上方 on_with_input_for 内同步
                        // 调用)已通过 auto.shell.exec_submit native 把请求压入
                        // vm::shell_bridge 队列,merged_exec_loop 从那里取。
                        // renderer 不再读 __pending_command 提交执行(旧链绕过
                        // api.at 契约)。此处置换块后仅需标脏。
                        *state.app.view_dirty.borrow_mut() = true;
                    }
                }
            }
        }

        // Plan 049:PromptBar.Run handler 清空 widget 本地 .input="",但 iced 0.14
        // 的 text_input 是单向 value 语义(widget.value 不同步回 state),固定 Id
        // (Plan 047)让 tree state 跨帧复用,导致 value="" 不生效。
        // 修复:Run 后显式清空 component state 的 input 字段 + 清 cached_rendered
        // 强制下一帧完全重建 text_input widget(新实例 value="")。
        // Plan 053 M4:OnEnter 内部 Run 后同样需要强制重建(cmd_ran 已判)。
        if cmd_ran {
            let _ = state
                .component
                .write_state("input", auto_val::Value::str(""));
            *state.app.cached_rendered.borrow_mut() = None; // 强制重建(丢弃旧 widget)
            *state.app.cached_converted_view.borrow_mut() = None;
            *state.app.view_dirty.borrow_mut() = true;
            if let Some(key) = &input_cache_key { state.app.input_values.remove(key); }
            // Plan 047:view 重建后恢复 input 焦点(否则 on_submit 第二次不触发)。
            state.app.devtools.needs_prompt_refocus.set(true);
        }

        // Plan 049:检测 blocks 数量增加 → 自动滚到底部。
        let cur_block_count = state
            .component
            .read_state_as_vec("blocks")
            .map(|v| v.len())
            .unwrap_or(0);
        if cur_block_count > state.app.devtools.last_block_count.get() {
            state.app.devtools.last_block_count.set(cur_block_count);
            state.app.devtools.needs_scroll_to_bottom.set(true);
        }

        // PB-11:Ctrl+L 清屏 emit 模拟(ash-gui M2)。PromptBar.OnCtrlL 应 emit
        // 'clear' 给 App → App.ClearScreen → store.ClearScreen,但 vm 剥离 callback
        // emit(同 Run)。这里直接触发 store.ClearScreen(归档所有 blocks)。
        if widget_name == "PromptBar" && event_name == "OnCtrlL" {
            let _ = state
                .component
                .on_with_input_for("ShellStore", "ClearScreen", None);
            *state.app.view_dirty.borrow_mut() = true;
        }

        // ── Shell bridge:RunCommand 后备路径(ash-gui M1) ──
        // 当 store.RunCommand 从 App.RunCommand 直接触发(非 emit 模拟路径,
        // 如 Rerun/侧栏),event_name=="RunCommand"。此路径下 store 也只记 pending,
        // block 构造 + 执行器提交同上。emit 模拟路径(PromptBar.Run)已在上方处理。
        if event_name == "RunCommand" {
            let bid = state
                .component
                .read_state("__pending_command_id")
                .map(|v| v.as_int() as i64)
                .unwrap_or(0);
            let cmd = state
                .component
                .read_state("__pending_command_str")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            let cwd = state
                .component
                .read_state("cwd")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if !cmd.is_empty() && bid >= 0 {
                let mut status = auto_val::Obj::new();
                status.set("kind", auto_val::Value::str("Running"));
                status.set("message", auto_val::Value::str(""));
                let mut block = auto_val::Obj::new();
                block.set("id", auto_val::Value::Int(bid as i32));
                block.set("command", auto_val::Value::str(&cmd));
                block.set("cwd", auto_val::Value::str(&cwd));
                block.set("status", auto_val::Value::Obj(Box::new(status)));
                block.set("streamed_text", auto_val::Value::str(""));
                block.set("duration_ms", auto_val::Value::Int(0));
                block.set("output", auto_val::Value::Nil);
                block.set("exit_code", auto_val::Value::Int(0));
                block.set("collapsed", auto_val::Value::Bool(false));
                block.set("table_sort_col", auto_val::Value::Int(-1));
                block.set("table_sort_dir", auto_val::Value::Int(1));
                block.set("table_filter_q", auto_val::Value::str(""));
                // Plan 063 T5: chat 回合号(非 chat 块恒 0)。
                block.set("turn", auto_val::Value::Int(0));
                if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                    blocks.push(auto_val::Value::Obj(Box::new(block)));
                    let _ = state.component.write_state_vec("blocks", blocks);
                }
                // Plan 060(M1-T3):同上 —— 执行提交由 shell.at 的
                // run_command → auto.shell.exec_submit 完成,renderer 不再插手。
                *state.app.view_dirty.borrow_mut() = true;
            }
        }
        if event_name == "Cancel" {
            // CMD-06:只停首个 Running block(对齐 Vue useShellTauri.ts:112 .find,
            // 非 filter)。blocks 由 renderer 管(Value::Array),在此用 Rust 读 blocks
            // 找首个 Running:标 cancel_flags(执行器进程 kill)+ 更新其 status=Cancelled。
            let raw = state.component.read_state("blocks").ok();
            let blocks_vec: Vec<auto_val::Value> = match &raw {
                Some(auto_val::Value::Array(arr)) => arr.values.clone(),
                Some(auto_val::Value::Nil) | None => Vec::new(),
                _ => state
                    .component
                    .read_state_as_vec("blocks")
                    .unwrap_or_default(),
            };
            let mut first_running_id: Option<i64> = None;
            let mut blocks_vec = blocks_vec;
            for b in &blocks_vec {
                if let auto_val::Value::Obj(obj) = b {
                    let kind = obj
                        .get("status")
                        .and_then(|s| {
                            if let auto_val::Value::Obj(so) = s {
                                so.get("kind").map(|k| k.as_str().to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    if kind == "Running" {
                        first_running_id =
                            Some(obj.get("id").map(|v| v.as_int() as i64).unwrap_or(0));
                        break;
                    }
                }
            }
            if let Some(bid) = first_running_id {
                // 通知执行器 kill 进程。
                // Plan 057:HTTP 模式跳过 —— store 的 cancel() 已被 api_over_http
                // 改写为 POST /api/cancel(后端 kill 子进程,结果经 SSE 回流)。
                if !shell_bridge_is_http() {
                    if let Some(handle) = SHELL_EXEC_HANDLE.get() {
                        if let Ok(mut h) = handle.lock() {
                            h.cancel_flags.insert(bid, true);
                        }
                    }
                }
                // 更新该 block status → Cancelled(CMD-06 只首个)。
                let mut changed = false;
                for b in blocks_vec.iter_mut() {
                    if let auto_val::Value::Obj(obj) = b {
                        let id_match = obj
                            .get("id")
                            .map(|v| v.as_int() as i64 == bid)
                            .unwrap_or(false);
                        if id_match {
                            let mut status = auto_val::Obj::new();
                            status.set("kind", auto_val::Value::str("Cancelled"));
                            status.set("message", auto_val::Value::str(""));
                            obj.set("status", auto_val::Value::Obj(Box::new(status)));
                            changed = true;
                            break;
                        }
                    }
                }
                if changed {
                    let _ = state.component.write_state(
                        "blocks",
                        auto_val::Value::Array(auto_val::Array { values: blocks_vec }),
                    );
                    *state.app.view_dirty.borrow_mut() = true;
                }
            }
        }

        // Post-process Lap: format lap entries with "Lap N: time" prefix.
        // The bytecode handler already shifts lap3=lap2, lap2=lap1, lap1=time.
        // We just re-format lap1 to include the lap count prefix.
        if event_name == "Lap" {
            let lap_count = state
                .component
                .read_state("lap_count")
                .map(|v| {
                    // Handle both int (after numeric += fix) and string types
                    match v {
                        auto_val::Value::Int(n) => format!("{}", n),
                        _ => v.as_str().to_string(),
                    }
                })
                .unwrap_or_else(|_| "0".to_string());
            let lap1 = state
                .component
                .read_state("lap1")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if !lap1.is_empty() {
                let _ = state.component.write_state(
                    "lap1",
                    auto_val::Value::str(&format!("Lap {}: {}", lap_count, lap1)),
                );
            }
        }

        // Dynamic todo list: handle indexed Toggle:N / Delete:N / AddTodo
        // Audit B12(ii): legacy demo fallback — run ONLY when the VM has no
        // real handler for this event. 013-todo's App handlers (AddTodo/
        // ToggleAll/ClearCompleted) used to run BOTH the VM handler and this
        // hardcoded Rust-side todo list; its sync_todos_to_vm then clobbered
        // the store's List<Todo> handle with a fabricated
        // Array([Obj{id: enumerate-index}]) — losing the seed list and nextid
        // ids (todos collapsed to [new item with id 0]).
        {
            let (base, idx) = parse_indexed_event(&event_name);
            if state.component.bridge().has_handler(base) {
                // Real VM handler already ran via on_with_input_for above —
                // the hardcoded fallback must not double-apply.
            } else {
                match base {
                    "Toggle" | "ToggleTodo" => {
                        if let Some(i) = idx {
                            if i < state.app.todos.len() {
                                state.app.todos[i].done = !state.app.todos[i].done;
                                let active =
                                    state.app.todos.iter().filter(|t| !t.done).count() as i32;
                                let _ = state
                                    .component
                                    .write_state("active_count", auto_val::Value::Int(active));
                                sync_todos_to_vm(&state.app.todos, &mut *state.component);
                            }
                        }
                    }
                    "Delete" | "DeleteTodo" => {
                        if let Some(i) = idx {
                            // Indexed Delete:N — todo item deletion
                            if i < state.app.todos.len() {
                                state.app.todos.remove(i);
                                let active =
                                    state.app.todos.iter().filter(|t| !t.done).count() as i32;
                                let _ = state
                                    .component
                                    .write_state("active_count", auto_val::Value::Int(active));
                                let _ = state.component.write_state(
                                    "todo_count",
                                    auto_val::Value::Int(state.app.todos.len() as i32),
                                );
                                sync_todos_to_vm(&state.app.todos, &mut *state.component);
                            }
                        } else {
                            // Bare Delete (no index) — notes deletion from EditorPanel
                            if let (Ok(mut notes), Ok(active_val)) = (
                                state.component.read_state_as_vec("notes"),
                                state.component.read_state("active_id"),
                            ) {
                                let active = active_val.as_int() as usize;
                                if !notes.is_empty() {
                                    let del_idx = if active < notes.len() { active } else { 0 };
                                    notes.remove(del_idx);
                                    let new_active = if notes.is_empty() {
                                        0
                                    } else {
                                        del_idx.min(notes.len() - 1)
                                    };
                                    let _ = state.component.write_state_vec("notes", notes);
                                    let _ = state.component.write_state(
                                        "active_id",
                                        auto_val::Value::Int(new_active as i32),
                                    );
                                }
                            }
                            let _ = state
                                .component
                                .write_state("editing", auto_val::Value::Bool(false));
                        }
                    }
                    "AddTodo" => {
                        let from_input_values =
                            state.app.input_values.get("EditInputChanged").cloned();
                        if !saved_input.is_empty() {
                            state.app.todos.push(TodoItem {
                                text: saved_input,
                                done: false,
                            });
                            let active = state.app.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state
                                .component
                                .write_state("active_count", auto_val::Value::Int(active));
                            let _ = state.component.write_state(
                                "todo_count",
                                auto_val::Value::Int(state.app.todos.len() as i32),
                            );
                            let _ = state
                                .component
                                .write_state("input", auto_val::Value::str(""));
                            sync_todos_to_vm(&state.app.todos, &mut *state.component);
                            state.app.input_values.remove("EditInputChanged");
                            state.app.input_values.remove("InputChanged");
                        } else if let Some(text) = from_input_values {
                            // Fallback: use the last tracked input value
                            state.app.todos.push(TodoItem { text, done: false });
                            let active = state.app.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state
                                .component
                                .write_state("active_count", auto_val::Value::Int(active));
                            let _ = state.component.write_state(
                                "todo_count",
                                auto_val::Value::Int(state.app.todos.len() as i32),
                            );
                            let _ = state
                                .component
                                .write_state("input", auto_val::Value::str(""));
                            sync_todos_to_vm(&state.app.todos, &mut *state.component);
                            state.app.input_values.remove("EditInputChanged");
                            state.app.input_values.remove("InputChanged");
                        }
                    }
                    "ClearCompleted" => {
                        state.app.todos.retain(|t| !t.done);
                        let active = state.app.todos.iter().filter(|t| !t.done).count() as i32;
                        let _ = state
                            .component
                            .write_state("active_count", auto_val::Value::Int(active));
                        let _ = state.component.write_state(
                            "todo_count",
                            auto_val::Value::Int(state.app.todos.len() as i32),
                        );
                        sync_todos_to_vm(&state.app.todos, &mut *state.component);
                    }
                    "ToggleAll" => {
                        let any_active = state.app.todos.iter().any(|t| !t.done);
                        for todo in &mut state.app.todos {
                            todo.done = any_active; // if any active → mark all done; else → mark all undone
                        }
                        let active = state.app.todos.iter().filter(|t| !t.done).count() as i32;
                        let _ = state
                            .component
                            .write_state("active_count", auto_val::Value::Int(active));
                        sync_todos_to_vm(&state.app.todos, &mut *state.component);
                    }
                    // Notes app: VM handlers now manage all state correctly.
                    // The previous hardcoded state-sync (read notes as Value::Obj,
                    // write edit_title/edit_body) is removed because notes elements
                    // are raw Int(heap_id) in VM mode, not Value::Obj — the if-let
                    // always failed. The VM handler_EditorPanel_Edit etc. handle
                    // everything via GET_FIELD/SET_FIELD on the unified state.
                    // We only clear stale input_values caches here so the next
                    // render reflects handler-set state, not old typed text.
                    "Edit" => {
                        state.app.input_values.remove("EditTitle");
                        state.app.input_values.remove("EditBody");
                    }
                    "Save" | "Cancel" => {
                        state.app.input_values.remove("EditTitle");
                        state.app.input_values.remove("EditBody");
                    }
                    _ => {}
                }
            }
        }

        // Deferred scroll: if selected_widget is set but pending_scroll not yet computed,
        // try to compute from element styles (which are populated during rendering).
        if state.app.devtools.selected_widget.borrow().is_some()
            && state
                .app
                .devtools
                .pending_scroll_to_center
                .borrow()
                .is_none()
        {
            if let Some(ref sel_id) = *state.app.devtools.selected_widget.borrow() {
                let styles = state.app.devtools.debug_element_styles.borrow();
                if let Some(elem_info) = styles.get(sel_id) {
                    if let Some((offset, _len)) = elem_info.span {
                        let offsets = state.app.source_line_offsets.borrow();
                        let line = offsets.partition_point(|&p| p <= offset).saturating_sub(1);
                        *state.app.devtools.pending_scroll_to_center.borrow_mut() = Some(line);
                    }
                }
            }
        }

        // Mark view dirty if component state changed.
        // Component dirty: set by on_with_input, write_state, reload.
        // (UI-only changes — hover, select, tab, debug — previously fed a
        // `ui_changed` flag here, but that flag was always false; see the note
        // above where it was declared. Set view_dirty directly if needed.)
        if state.component.is_dirty() {
            *state.app.view_dirty.borrow_mut() = true;
        }

        // Emit scroll_to Task if pending scroll is set
        let scroll_task: Option<iced::Task<IcedMessage>> = state
            .app
            .devtools
            .pending_scroll_to_center
            .borrow_mut()
            .take()
            .map(|line_idx| {
                let line_height = 14.0; // font_size(10) + spacing(4)
                let viewport_height = 500.0; // estimated panel content area height
                let target_y = (line_idx as f32 * line_height) - (viewport_height / 3.0);
                let y = target_y.max(0.0);
                iced::widget::operation::scroll_to(
                    state.app.devtools.inspector_scroll_id.clone(),
                    iced::widget::scrollable::AbsoluteOffset {
                        x: Some(0.0),
                        y: Some(y),
                    },
                )
            });

        // ── Plan 057 续(尾部任务批处理)────────────────────────────────
        // 原实现是一串互斥的提前 return(refocus/初始聚焦/resize/bounds/滚底),
        // 同一 update 里多个任务并存时后者被前者吞掉 —— 典型回归:Enter 执行
        // 的那帧同时置 refocus 与 scroll_to_bottom,refocus 先返回,自动滚底
        // 永不执行(输入多条命令后列表不再钉在最新块)。改为收集后 batch。
        // 原「refocus 必须先于 needs_bounds(否则被 bounds 收集链饿死)」的
        // 约束在 batch 模式下天然满足(两者同帧都执行)。
        let mut tail_tasks: Vec<iced::Task<IcedMessage>> = Vec::new();

        // Plan 047/057: 恢复输入焦点(最后被编辑的 textarea,按稳定 Id)。
        if state.app.devtools.needs_prompt_refocus.get() {
            state.app.devtools.needs_prompt_refocus.set(false);
            let id = state
                .app
                .devtools
                .last_textarea_key
                .borrow()
                .as_ref()
                .map(|k| iced::widget::Id::from(format!("textarea_{}", k)))
                .or_else(|| state.app.devtools.input_ids.borrow().first().cloned())
                .unwrap_or_else(|| state.app.devtools.prompt_input_id.clone());
            if std::env::var("ASH_DEBUG_FOCUS").is_ok() {
                eprintln!("[FOCUS-DBG] tail refocus, id={:?}", id);
            }
            tail_tasks.push(iced::widget::operation::focus(id));
        }

        // Plan 057: 启动默认聚焦(首个 update,textarea 已注册时)。
        if !state.initial_focus_done.get() && !TEXTAREA_CONTENTS.lock().unwrap().is_empty() {
            state.initial_focus_done.set(true);
            let id = TEXTAREA_CONTENTS
                .lock()
                .unwrap()
                .keys()
                .next()
                .map(|k| iced::widget::Id::from(format!("textarea_{}", k)))
                .or_else(|| state.app.devtools.input_ids.borrow().first().cloned())
                .unwrap_or_else(|| state.app.devtools.prompt_input_id.clone());
            if std::env::var("ASH_DEBUG_FOCUS").is_ok() {
                eprintln!("[FOCUS-DBG] initial focus, id={:?}", id);
            }
            tail_tasks.push(iced::widget::operation::focus(id));
        }

        // Plan 464 T3：`__focus_input` 状态约定 —— app 侧把该状态写 "1" 请求
        // 聚焦其输入框（palette「打开即聚焦」P3；desktop 召唤路径 T4 宿主同款
        // 写法）。消费即清零（防重复聚焦）；Id 用 Plan 047 的 vm input 统一
        // 稳定 Id prompt_input。desktop 多 App 同 Id 冲突为 v1 已知边界
        // （launcher 层最上、召唤场景无竞争输入框；462 焦点分区命名空间跟进）。
        if matches!(
            state.component.read_state("__focus_input"),
            Ok(auto_val::Value::Str(ref s)) if s.to_string() == "1"
        ) {
            // PLAN-013 W2：登记表对 overlay 挂载面恒空（dynamic_view 组装
            // 路径不产 input Id，16032 的登记写入会被空集合覆盖）——改为
            // 消费时即时派生：同步构建当前 app 视图收集 input Id（与
            // summon 预登记同一 derive_input_id 稳定通道），登记表只作
            // 回退。
            let derived = {
                let (view, _, _) = state.component.view_with_debug_gated(false);
                let converted = convert_view_messages(view);
                let mut ids = Vec::new();
                collect_input_ids(&converted, &mut ids);
                ids.first().cloned()
            };
            let focus_target = derived.unwrap_or_else(|| {
                state
                    .app
                    .devtools
                    .input_ids
                    .borrow()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| state.app.devtools.prompt_input_id.clone())
            });
            if std::env::var("AUTO_DEBUG_FOCUS").is_ok() {
                eprintln!("[464-FOCUS] __focus_input consumed: derived target={focus_target:?}");
            }
            tail_tasks.push(iced::widget::operation::focus(focus_target));
            // PLAN-013 W2：重试上限——focus 任务与 overlay 入树存在时序
            // 竞态（首拍可能落空），借 launcher 消息泵周期重试；5 轮后
            // 放弃清位（期间每次 launcher 消息周期补一次聚焦）。
            let tries = match state.component.read_state("__focus_input_tries") {
                Ok(auto_val::Value::Int(n)) => n,
                _ => 0,
            };
            if tries >= 5 {
                let _ = state
                    .component
                    .write_state("__focus_input", auto_val::Value::str(""));
                let _ = state
                    .component
                    .write_state("__focus_input_tries", auto_val::Value::Int(0));
            } else {
                let _ = state
                    .component
                    .write_state("__focus_input_tries", auto_val::Value::Int(tries + 1));
            }
        }

        // PLAN-095 T-02: ImageSurface loaded/error 通知 sweep —— 解码完成
        // 且下一拍（其间已有一帧消费 rendition）单次派发 on_loaded；终态
        // 失败单次派发 on_error（fn(str) 收原因串）。表项随视图存在性生死
        // （卸载即退订）；不在 view 构建/paint 期重入 handler。
        media_notify_sweep(&mut state.component, &state.app);

        // PLAN-095 T-04: ui.focus(target_key) 消费——native 请求槽 → 当前
        // 视图 Input/Textarea 稳定 Id 解析 → iced focus 任务；结果落
        // __focus_result（"ok" / "miss:<key>"）可观察，不做 no-op 降级。
        // 挂载时序竞态沿用 __focus_input 的 5 轮重试口径（有更新周期才推进）。
        if let Some(key) = auto_lang_core::vm::native::take_ui_focus_request() {
            *state.app.focus_pending.borrow_mut() = Some((key, 0));
        }
        // PLAN-747 根修：克隆提为独立语句——edition 2021 下 if-let 判别式临时
        // （Ref 守卫）活整个块，块内 borrow_mut 必 panic 且 pending 永不清除
        // （每条 update 消息重入 panic，update 后半段随 unwind 全丢 = UI 冻结；
        // 2026-10-09 退出审计 4611 例 "RefCell already borrowed" 实证）。
        let focus_pending = state.app.focus_pending.borrow().clone();
        if let Some((key, tries)) = focus_pending {
            let (view, _, _) = state.component.view_with_debug_gated(false);
            let converted = convert_view_messages(view);
            let mut inputs: Vec<FocusableInput> = Vec::new();
            collect_focusable_inputs(&converted, &mut inputs);
            let (focus_id, done, result) = focus_pending_step(&inputs, &key, tries);
            if let Some(id) = focus_id {
                tail_tasks.push(iced::widget::operation::focus(id));
            }
            if done {
                // 命中=ok；稳定 miss（视图已含输入而目标缺席，或 5 轮重试耗尽）。
                let _ = state
                    .component
                    .write_state("__focus_result", auto_val::Value::str(&result));
                *state.app.focus_pending.borrow_mut() = None;
            } else {
                // 挂载竞态窗口：视图尚无任何输入，重试计数 +1。
                *state.app.focus_pending.borrow_mut() = Some((key, tries + 1));
            }
        }

        // Plan 402: pending window resize。
        if let Some(size) = state.pending_window_resize.borrow_mut().take() {
            *state.window_size.borrow_mut() = size;
            if let Some(rect) = state.vwin_rect {
                // Plan 462：desktop 模式 —— 模型 window_* 变量落虚拟窗口矩形，
                // 宿主 OS 窗保持不变（R2/R9）。
                let mut r = rect.borrow_mut();
                r.width = size.width;
                r.height = size.height;
            } else {
                // Plan 459 遗留 bug 修复（462 T5）：resize 直接指向本视图
                // 归属窗口，退役 `window::oldest()` 单窗时代猜测（多窗下会
                // 改错窗）。
                let win = state.window;
                tail_tasks.push(iced::window::resize::<IcedMessage>(win, size));
            }
        }

        // Plan 282: 布局 bounds 收集(截图/检视数据;截屏挂起时跳过)。
        // PLAN-781 T-11：同一批并推 ScrollStateReader——pane 几何与 bounds
        // 同一布局遍、相邻 tick 发布进 SharedState（见 __mcp_scroll_states）。
        if *state.app.devtools.needs_bounds.borrow()
            && state.app.devtools.screenshot_request.borrow().is_none()
        {
            *state.app.devtools.needs_bounds.borrow_mut() = false;
            use crate::ui::iced::LayoutCollector;
            tail_tasks.push(iced::advanced::widget::operate(LayoutCollector::new()).map(
                |bounds_map| IcedMessage {
                    widget: String::new(),
                    event: "__bounds_collected".to_string(),
                    input_value: Some(serde_json::to_string(&bounds_map).unwrap_or_default()),
                },
            ));
            tail_tasks.push(
                iced::advanced::widget::operate(
                    crate::ui::iced::scroll_state_reader::ScrollStateReader::new(),
                )
                .map(|state_map| IcedMessage {
                    widget: String::new(),
                    event: "__mcp_scroll_states".to_string(),
                    input_value: Some(serde_json::to_string(&state_map).unwrap_or_default()),
                }),
            );
        }

        // Plan 049: blocks 增加后自动滚到底部(最新 block 可见)。
        if state.app.devtools.needs_scroll_to_bottom.get() {
            state.app.devtools.needs_scroll_to_bottom.set(false);
            tail_tasks.push(iced::widget::operation::snap_to_end(
                state.app.devtools.blocklist_scroll_id.clone(),
            ));
        }

        // Plan 043 T1: 排空 Scrollable offset 绑定写入队列（build 面
        // note_scroll_offset 登记，此处发 scroll_to Task——build 面拿不到
        // Task 返回槽，devtools pending_scroll_to_center 同款接力）。
        for (id, (x, y)) in drain_pending_scroll_offsets() {
            if std::env::var("P043_DEBUG").is_ok() {
                eprintln!("[P043-DRAIN] scroll_to id={id} y={y}");
            }
            tail_tasks.push(iced::widget::operation::scroll_to(
                id,
                iced::widget::scrollable::AbsoluteOffset { x, y },
            ));
        }

        // PLAN-656 T-05 + review F-4: controller intent 队列排空与测量回读。
        // ① 新绑定 handle 的预热请求 → 读回操作补首用基线；
        // ② 未预热 handle 的 intents 由 drain 留队（返回 prime 名单）——
        //    先读回、下一 tick 重解析（首用动作晚一拍而非解析为 0）；
        // ③ 已解析终态 → scroll_to 落盘 + 投影写回 + 读回校正（iced 的
        //    实际 clamp 结果覆盖 F-2 的命令值投影）。
        let scroll_read_task = || {
            iced::advanced::widget::operate(
                crate::ui::iced::scroll_state_reader::ScrollStateReader::new(),
            )
            .map(|state_map| IcedMessage {
                widget: String::new(),
                event: "__scroll_state_read".to_string(),
                input_value: Some(serde_json::to_string(&state_map).unwrap_or_default()),
            })
        };
        // PLAN-781 T-11：同一读回节拍并行发布 MCP 观测面（滚动不改脏帧，
        // bounds 批不会重跑——offset 新鲜度由本通道维持，≤2s 节拍与
        // __scroll_state_read 同源）。
        let mcp_scroll_task = || {
            iced::advanced::widget::operate(
                crate::ui::iced::scroll_state_reader::ScrollStateReader::new(),
            )
            .map(|state_map| IcedMessage {
                widget: String::new(),
                event: "__mcp_scroll_states".to_string(),
                input_value: Some(serde_json::to_string(&state_map).unwrap_or_default()),
            })
        };
        let has_prime_requests = !crate::ui::scroll::controller::take_prime_requests().is_empty();
        if has_prime_requests {
            tail_tasks.push(scroll_read_task());
            tail_tasks.push(mcp_scroll_task());
        }
        // PLAN-781 T-03: pending 锚点请求 → 推 AnchorBoundsCollector + 读回
        //（两路消息下一拍到达、齐备即解析；未预热 pane 的请求自补 prime）。
        if crate::ui::scroll::controller::has_pending_anchor_requests() {
            if std::env::var("P781_DEBUG").is_ok() {
                eprintln!("[P781] drain: pending anchors -> pushing collector");
            }
            tail_tasks.push(scroll_read_task());
            tail_tasks.push(
                iced::advanced::widget::operate(
                    crate::ui::iced::scroll_anchor::AnchorBoundsCollector::new(),
                )
                .map(|rects| IcedMessage {
                    widget: String::new(),
                    event: "__anchor_bounds_collected".to_string(),
                    input_value: Some(serde_json::to_string(&rects).unwrap_or_default()),
                }),
            );
        }
        let (finals, prime) = crate::ui::scroll::drain_resolved_intents();
        let finals_done = !finals.is_empty();
        for (handle, id, x, y) in finals {
            if std::env::var("P656_DEBUG").is_ok() {
                eprintln!("[P656-DRAIN] scroll_to id={id} x={x} y={y}");
            }
            // review F-2：命令值先行投影（读回一 tick 后以真实 clamp 校正）。
            crate::ui::scroll::controller::note_controller_offset(&handle, x, y);
            // F-4 末环 diff 实验：复用 __mcp_scroll 消费者（载荷 "id␟y␟x"，
            // 第三段 x 为本计划扩展）——该头部直返路径对 MCP 实证有效。
            tail_tasks.push(iced::Task::done(IcedMessage {
                widget: String::new(),
                event: "__mcp_scroll".to_string(),
                input_value: Some(format!("{id}{sep}{y}{sep}{x}", sep = PAYLOAD_SEP)),
            }));
        }
        // 读回节拍：预热/排空后立即读 + 心跳持续读（程序化 scroll_to 的
        // 落盘 offset 在下一帧才可见，单次读会取到旧值——心跳读收敛投影；
        // 读消息自身不再触发读，防自激环）。
        if msg.event != "__scroll_state_read"
            && msg.event != "__mcp_scroll_states"
            && (!prime.is_empty() || finals_done || msg.event == "__mcp_heartbeat")
        {
            tail_tasks.push(scroll_read_task());
            tail_tasks.push(mcp_scroll_task());
        }

        // PLAN-781 T-03: 锚点解析 finals 落盘（消息处理器暂存产物；同款
        // 写臂投影 + __mcp_scroll 头部直返路径——与 controller finals 一致）。
        for (a_handle, a_id, ax, ay) in crate::ui::iced::scroll_anchor::take_resolved_finals() {
            crate::ui::scroll::controller::note_controller_offset(&a_handle, ax, ay);
            tail_tasks.push(iced::Task::done(IcedMessage {
                widget: String::new(),
                event: "__mcp_scroll".to_string(),
                input_value: Some(format!("{a_id}{sep}{ay}{sep}{ax}", sep = PAYLOAD_SEP)),
            }));
        }

        // PLAN-063 T-04d-2: 块锚定同步目标消费——锚块变化时经注册表把
        // 「锚块内容 y」写进 sync_anchor_target 指向的状态字段（右栏
        // offset 绑定写臂既有通路滚动；iced 原生钳制内容边界）。
        if let Some((field, idx)) = crate::ui::anchor_slot::drain_pending_anchor() {
            let y = crate::ui::anchor_slot::content_y(idx);
            eprintln!("[P063-C] field={field:?} idx={idx} y={y:?}");
            // 锚块索引直写（.at 声明 sync_anchor_block，-1 = 未锚定；
            // ghost_id/ghost_height 同款固定名直写先例）——vm-smoke 组 4
            // AC-06 断言可观测面。y 未就绪（块未布局）时索引先行登记。
            let _ = state
                .component
                .write_state("sync_anchor_block", auto_val::Value::Int(idx as i32));
            if let (Some(field), Some(y)) = (field, y) {
                let _ = state
                    .component
                    .write_state(&field, auto_val::Value::Float(y as f64));
            }
        }
        if !tail_tasks.is_empty() {
            return iced::Task::batch(tail_tasks);
        }
        scroll_task.unwrap_or_else(iced::Task::none)
    };

    // Plan 453 T4 / Plan 459 C2：外壳消息 DesktopMessage 三分派 ——
    // DM::App 按归属 App 分派（update_inner 经 split_mut(app) 取窗口级字段）、
    // DM::Window 先经 app_of_window 现场解析归属（活注册表，无陈旧性）、
    // DM::Desktop 维护窗口生命周期。全窗口关闭 → `iced::exit()`（daemon
    // 语义：不退则进程静默长存，见施工图 §1-2）。T6 的 panic 边界保留在
    // App/Window 分支内的 catch_unwind；RefCell 无中毒语义、RAII Guard 随
    // unwind 释放。
    let update = move |state: &mut crate::ui::session::DesktopSession,
                       msg: crate::ui::session::DesktopMessage|
          -> iced::Task<crate::ui::session::DesktopMessage> {
        use crate::ui::session::{DesktopEvent, DesktopMessage as DM, WmCommand};
        if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
            match &msg {
                DM::App(ref app_id, ref m) => {
                    eprintln!("[464-UPD] DM::App({app_id:?}, {})", m.event);
                }
                DM::Desktop(ref ev) => {
                    eprintln!("[464-UPD] DM::Desktop({ev:?})");
                }
                _ => {}
            }
        }
        // Plan 463 T4：DesktopBus 排空原在此处（任意消息周期开头）。Plan 464
        // T4：summon 命令产生 iced 任务，需在能返回 Task 的排空点执行——排空
        // 移至 DM::App 臂尾（按钮同周期即达，463 T4 语义）+ ServiceTick 臂
        // （400ms 兜底，覆盖空闲期）。本点仅保留注入。
        if state.desktop.shell_app.is_some() {
            // Plan 463 T5：窗口列表/布局注入（指纹门控，每周期一次兜底；
            // DM::Wm 臂尾有同周期即时版）。
            sync_shell_windows(state);
        }
        // 消息归属 → update 分派（App/Window 两臂共用；console 打标 + T5
        // 探针 + panic 边界 + Task 回标 DM::App(app)）。闭包捕获 update_inner
        //（fn 条目无法捕获闭包环境），state 经参数传入不与之冲突。
        let dispatch_app = |state: &mut crate::ui::session::DesktopSession,
                            app_id: crate::ui::session::AppId,
                            m: IcedMessage,
                            http_backfill: bool|
         -> iced::Task<crate::ui::session::DesktopMessage> {
            // console 打标：update 期间 print/console_log 归属本 App。
            auto_lang_core::libs::builtin::set_console_current_app(app_id.0);
            // PLAN-721 T-1：泵族消息到达 trace（`__` 前缀 = 框架内务事件；
            // 桌面轨断链定案轴——r2「桌面内嵌轨 0 条 T16-DIAG」的复测面）。
            if m.event.starts_with("__") {
                crate::ui::sched_diag::trace(
                    Some(app_id),
                    "msg",
                    &format!("widget={:?} event={}", m.widget, m.event),
                );
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Plan 459 T5 探针：验收用 panic 注入（见 PANIC_PROBE_CRASHED_APP）。
                // 必须在 catch_unwind **内部** panic（与 update_inner 同一边界），
                // 否则 panic 直穿 winit 事件循环杀死进程（实测）。
                if panic_probe_enabled() && is_panic_probe_event(&m.event) {
                    PANIC_PROBE_CRASHED_APP.store(app_id.0, std::sync::atomic::Ordering::SeqCst);
                    panic!("plan-459 T5 panic probe injected for App {app_id:?}");
                }
                update_inner(state, app_id, m, http_backfill)
            })) {
                Ok(task) => {
                    // PLAN-740 T-00（交付节奏三轴之二——request_redraw 发出
                    // 代理轴）：update_inner 返回时刻。iced_winit AboutToWait
                    // 轮内逐消息 update 后对全窗无条件 request_redraw（735
                    // 轴②实勘）——本点即每消息 update 轮完成/发出节拍，
                    // 与 redraw_deliver（到达轴）对读定责 emit→deliver 时延。
                    // AUTO_SCHED_DIAG 门控零开销。
                    if crate::ui::sched_diag::enabled() {
                        let t0 = crate::ui::dynamic::sched_diag_t0();
                        eprintln!(
                            "[SCHED-DIAG] update_end t={}ms app={:?}",
                            t0.elapsed().as_millis(),
                            app_id
                        );
                    }
                    // Plan 512：fit 动态重测——app view 重建（update 置位
                    // view_dirty）落到其 fit 窗条目打标；ServiceTick 节拍
                    // 消费发起测量，测量时对最新布局树取末值（帧合并）。
                    if state
                        .apps
                        .get(&app_id)
                        .map(|a| *a.state.view_dirty.borrow())
                        .unwrap_or(false)
                    {
                        state.mark_fit_dirty(app_id);
                    }
                    // PLAN-711 T-04 (D-1) "view 登记需求仅发一次调度信号"
                    // 的 update 侧实现：Init demand 在**渲染期**登记，而订阅
                    // 重估在 update() 内部、view() **之前**（iced_winit
                    // lib.rs :1337 in update/:1300——本周期 update 后重估
                    // 看不到本周期 view 将登记的 demand）。凡本 update 致脏
                    // （重建将发生、可能登记 demand）即链一个 ready 唤醒：
                    // 下一轮 update 的订阅重估必然看到 demand → 帧泵订阅
                    // 装配 → 帧消息接管驱动。收斂性：唤醒链以 is_dirty 为
                    // 燃料——泵有推进才置脏，I/O 在途（泵零推进）时不再
                    // 链（tick 泵接管其 I/O 就绪），demand 排空后自然熄火，
                    // 无消息自旋。
                    let needs_wake = state
                        .apps
                        .get(&app_id)
                        .map(|a| a.component.is_dirty())
                        .unwrap_or(false);
                    let task = task.map(move |m| DM::App(app_id, m));
                    if needs_wake {
                        task.chain(iced::Task::perform(std::future::ready(()), move |()| {
                            DM::App(
                                app_id,
                                crate::ui::iced::IcedMessage {
                                    widget: String::new(),
                                    event: "__frame_pump".to_string(),
                                    input_value: None,
                                },
                            )
                        }))
                    } else {
                        task
                    }
                }
                Err(payload) => {
                    eprintln!("[session] app update panicked (plan-453 T6 boundary): {payload:?}");
                    iced::Task::<IcedMessage>::none().map(move |m| DM::App(app_id, m))
                }
            }
        };
        match msg {
            DM::Desktop(ev) => {
                // 桌面事件分支：窗口生命周期落注册表 / 焦点记录（T4c）。
                match ev {
                    DesktopEvent::WindowOpened(id, size) => {
                        // boot 期已按 `window::open` 同步登记；本臂作 Opened
                        // 兜底/幂等刷新。daemon 下窗口必有 boot 归属——
                        // 未知窗口（非本会话开窗）忽略。
                        let app = state.app_of_window(&id).or_else(|| state.primary_app());
                        if let Some(app) = app {
                            state.register_window(id, app, size);
                        }
                        // Plan 488 步骤 5：宿主窗打开即挂拖入目标（幂等，HWND
                        // 身份键控；winit 目标位移见 native_dnd spike 结论）。
                        #[cfg(all(windows, feature = "native-dnd"))]
                        if state.is_desktop() {
                            crate::ui::native_dnd::win32::ensure_host_drop_target();
                        }
                    }
                    DesktopEvent::WindowClosed(id) => {
                        #[cfg(windows)]
                        super::native_accessibility::remove(id);
                        // 459 不变式：一窗一 App，App 生命周期随窗（454 引入
                        // VirtualWindow 后由 WM 接管重挂载/续命）。
                        if let Some(entry) = state.windows.remove(&id) {
                            state.apps.remove(&entry.app);
                        }
                        // daemon 语义：全窗口关闭才退出进程（施工图 §1-2）。
                        if state.windows.is_empty() {
                            // Plan 505 B5（债 P480-R1）：退出前显式
                            // 停机 broker serve 线程。
                            state.shutdown_broker();
                            return iced::exit();
                        }
                    }
                    DesktopEvent::HostExitRequested => {
                        // PLAN-815 T-05（F03）：嵌入式宿主外部收口——镜像
                        // 全窗关闭的退出收尾（broker 停机 + daemon 退出），
                        // run_dynamic_iced 正常返回。
                        eprintln!(
                            "[session] host exit requested — closing the session (PLAN-815 T-05)"
                        );
                        state.shutdown_broker();
                        return iced::exit();
                    }
                    DesktopEvent::WindowFocused(id) => {
                        *state.focused_window.borrow_mut() = Some(id);
                    }
                    DesktopEvent::WindowUnfocused(id) => {
                        // 仅当失焦者正是当前焦点时清空（防多窗口事件乱序误清）。
                        let was_focused = {
                            let mut f = state.focused_window.borrow_mut();
                            if *f == Some(id) {
                                *f = None;
                                true
                            } else {
                                false
                            }
                        };
                        // App-level blur is opt-in: only dispatch when the
                        // component actually declares a `Blur` handler. This
                        // keeps the desktop shell and existing apps unchanged
                        // while giving native games a real focus-loss hook.
                        if was_focused {
                            if let Some(app_id) = state.app_of_window(&id) {
                                let handles_blur = state
                                    .apps
                                    .get(&app_id)
                                    .map(|app| app.component.bridge().has_handler("Blur"))
                                    .unwrap_or(false);
                                if handles_blur {
                                    let blur = IcedMessage {
                                        widget: String::new(),
                                        event: "Blur".to_string(),
                                        input_value: None,
                                    };
                                    return dispatch_app(state, app_id, blur, false);
                                }
                            }
                        }
                    }
                    // PLAN-029 T-02（D1）：live 输入泵入——桌面窗过滤（防独立
                    // app 窗未捕获键漏路由）+ picker 模态避让（负一屏开时键
                    // 归 picker 导航，不透传子窗）→ 生产路由（键盘/IME = 焦点
                    // 窗、滚轮 = last_cursor 命中窗，broker_* 零改动直用）。
                    DesktopEvent::LiveInput { window, input } => {
                        let desktop_window = state.host.as_ref().map(|h| h.window);
                        if desktop_window == Some(window)
                            && !PICKER_KEYS_OPEN.load(std::sync::atomic::Ordering::Relaxed)
                        {
                            state.route_live_input(&input);
                        }
                    }
                    // PLAN-063（OS T-03，P-03/P-04 接线）：broker 活跃档
                    // 专泵——仅泵（预算切片防重窗垄断），壳层职责仍归
                    // 400ms ServiceTick（其泵调用保留为待命档兜底）。
                    DesktopEvent::BrokerPump => {
                        state.pump_broker_clients();
                    }
                    // Plan 462：desktop 帧泵 —— 空更新，仅驱动 view 重算
                    //（MCP 截图请求在 view 投递 / update 消费）。
                    // Plan 464 T4：帧泵期排空（463 排空 #1 上移后的空闲期
                    // 兜底；shell/launcher 命令至多 400ms 达）。
                    DesktopEvent::ServiceTick => {
                        // Plan 488 步骤 5：宿主窗重建自愈——HWND 变化时重挂
                        // 拖入目标（幂等直返，400ms 节拍）。
                        #[cfg(all(windows, feature = "native-dnd"))]
                        if state.is_desktop() {
                            crate::ui::native_dnd::win32::ensure_host_drop_target();
                        }
                        // Plan 497 G1：dock 时钟——分钟变化才注入（400ms
                        // 帧泵粒度检查，稳态零重建；本地 tick 非投影流量）。
                        update_shell_clock(state);
                        // PLAN-041 T-15：面板暗色注入位单点同步（值变才写）。
                        sync_dash_dark_bit(state);
                        // PLAN-053 T-03：主题策略状态面刷新脏位消费（outproc
                        // 结果帧/策略变化置位；同值零写——变更检测在 sync 内）。
                        if crate::ui::theme_policy::take_shell_dirty() {
                            crate::ui::iced::renderer::shell_state::sync_theme_shell_state(
                                state, None,
                            );
                        }
                        // PLAN-042 T-06：syslog 查看窗注入泵（窗在才扫描；
                        // seq 门 + 500ms 攒批；全量快照下行）。
                        syslog_inject_tick(state);
                        // PLAN-041 T-14：空闲呈现保活（1s 节流，Win32 门控）。
                        #[cfg(windows)]
                        presentation_keepalive_tick();
                        // PLAN-036 T-07（B3）：launcher attach 追赶——
                        /// spawn→attach 异步落差（launcher_open 挂起 +
                        /// pipe 落地 + 指纹未建）时补推召唤快照。
                        if state.desktop.launcher_open
                            && state.desktop.launcher_pipe.is_some()
                            && state.desktop.launcher_fp.is_none()
                        {
                            if let Some(lwid) = state.desktop.launcher_wid {
                                state.wm_focus(lwid);
                            }
                            push_launcher_snapshot(
                                state,
                                &[crate::ui::shell_projection::ShellEvent::ApplyFilter],
                            );
                        }
                        // 2026-09-15：切换预览面板开着时——逐可见窗补抓快照
                        // + 置 shell 重建。快照异步入缓存后原本无人触发重建，
                        // 预览恒为空壁纸底（用户实测截图）；400ms 节拍内小
                        // 视图重建成本可控，面板收起（switcher_open="0"）即停。
                        if let Some(shell) = state.desktop.shell_app {
                            let open = state
                                .apps
                                .get(&shell)
                                .and_then(|a| a.component.read_state("switcher_open").ok())
                                .map(|v| v.to_string().contains('1'))
                                .unwrap_or(false);
                            if open {
                                if let Some(host) = state.host.as_ref() {
                                    for &wid in &host.wm.z_order {
                                        let visible = host
                                            .wm
                                            .wins
                                            .get(&wid)
                                            .map(|v| !v.hidden.get())
                                            .unwrap_or(false);
                                        if visible {
                                            crate::ui::iced::snapshot::request_capture(wid);
                                        }
                                    }
                                }
                                publish_workspace_previews(state);
                                if let Some(app) = state.apps.get_mut(&shell) {
                                    *app.state.view_dirty.borrow_mut() = true;
                                }
                            }
                        }
                        // Plan 551 T6:外写热应用轮询(os-config 经 daemon 改
                        // config.at → 宿主 400ms 节拍感知 → 差异应用)。
                        // PLAN-053 T-04：standalone armed 时同拍生效——
                        // 策略组差异臂（apply_external_config_diff）与宿主
                        // bus 动词同一 stage/commit 入口喂 031 端口；订阅门
                        // 在 armed 位拉起（unarmed 零拍）。
                        poll_external_config(state);
                        // PLAN-526 T18：热键切换的切换预览面板倒计时收起
                        // （~1.6s；触发 icon 手动开合不经此——无倒计时）。
                        if let Some(until) = state.desktop.switcher_until.get() {
                            if std::time::Instant::now() >= until {
                                state.desktop.switcher_until.set(None);
                                if let Some(shell) = state.desktop.shell_app {
                                    if let Some(app) = state.apps.get_mut(&shell) {
                                        let _ = app
                                            .component
                                            .write_state("switcher_open", auto_val::Value::str(""));
                                        *app.state.view_dirty.borrow_mut() = true;
                                    }
                                }
                            }
                        }
                        // PLAN-002 N2：fit 窗隐藏时长计数——测量常态首帧内
                        // 回执，此前窗体以越界偏移隐藏（vwin_fit_hidden）；
                        // 本计数超 FIT_HIDE_MAX_TICKS 即强制显形（退化为旧
                        // 观感），防 app 异常时测量永不回执的死窗。
                        if state.has_fit_pending() {
                            if let Some(host) = state.host.as_ref() {
                                for v in host.wm.wins.values() {
                                    if v.fit_pending.get() {
                                        v.fit_hidden_ticks
                                            .set(v.fit_hidden_ticks.get().saturating_add(1));
                                    }
                                }
                            }
                        }
                        // PLAN-012 F2 走查：命令通道前置——下方 fit/快照
                        // 臂按单任务语义 `return task`，持续命中（fit 反复
                        // 待测/快照队列连续）时原本会把臂尾的注入排空 +
                        // DesktopBus drain 无限饿死（实机确定性复现：首个
                        // launch 之后的全部 activate 记录静默丢弃 = dock
                        // 图标点击既不聚焦也不切高亮）。排空先于一切早退；
                        // 其任务与 fit/快照任务并存时合批（测量/抓取可重
                        // 入，下一拍重试；命令不可丢）。
                        apply_desktop_injects(state);
                        let (drain_exit, mut drain_tasks) =
                            drain_and_execute_desktop_commands(state);
                        // Retire observation ownership after close commands,
                        // before fit/screenshot early returns (also minimized).
                        state.prune_mcp_app_surfaces();
                        if drain_exit {
                            // Plan 505 B5（债 P480-R1）：退出前显式
                            // 停机 broker serve 线程。
                            state.shutdown_broker();
                            return iced::exit();
                        }
                        // Plan 504：fit 虚拟窗触发——节拍上若有待测量窗且
                        // 单程闸空闲，发起内容测量（宿主树恒在；vwin 首帧
                        // 渲染后锚点即在，回执 __fit_measured 收缩矩形）。
                        // Plan 512：动态重测并入同闸——fit_dirty（view 重建
                        // 打标）同样驱动一次测量，回执按滞回决策跟随。
                        if (state.has_fit_pending() || state.has_fit_remeasure_pending())
                            && !state.desktop.fit_measure_in_flight.get()
                        {
                            // Plan 512：standalone 无宿主窗——测量目标取待测
                            // fit 窗自身（锚点在该窗树中；desktop 恒为宿主窗，
                            // vwin 锚点在宿主树）。
                            let fit_target = state.host.as_ref().map(|h| h.window).or_else(|| {
                                state
                                    .windows
                                    .iter()
                                    .find(|(_, e)| e.fit_pending.get() || e.fit_dirty.get())
                                    .map(|(id, _)| *id)
                            });
                            if let Some(hw) = fit_target {
                                state.desktop.fit_measure_in_flight.set(true);
                                state.desktop.fit_measure_retries.set(0);
                                let fit = fit_measure_task(hw);
                                return if drain_tasks.is_empty() {
                                    fit
                                } else {
                                    let mut all = std::mem::take(&mut drain_tasks);
                                    all.push(fit);
                                    iced::Task::batch(all)
                                };
                            }
                        }
                        // Plan 497 G3：快照抓取编排——排空渲染臂/召唤面
                        // 攒下的 request_capture 队列，一次整窗 screenshot
                        // 服务全部请求（回调 SnapshotShot 裁剪入缓存）。
                        if let Some(task) = service_snapshot_requests(state) {
                            return if drain_tasks.is_empty() {
                                task
                            } else {
                                let mut all = std::mem::take(&mut drain_tasks);
                                all.push(task);
                                iced::Task::batch(all)
                            };
                        }
                        // Plan 480 S3/S4：broker 孵化落地 + 多 client 帧泵
                        // （有在册/排队连接才有成本，零排队两调用皆空转）。
                        if state.pending_incubations() > 0 {
                            state.attach_pending_incubations(5000);
                        }
                        state.pump_broker_clients();
                        // PLAN-030 T-05/T-06：泵后三件——①壳看门兵（到期
                        // respawn，D5 分拍）；②respawn 预算耗尽降级 = 一次性
                        // 回退 in-proc 装载（shell_degraded 位置位防重入）；
                        // ③DesktopBus inbox 排空直调执行（D4——泵后同拍，
                        // 归因 notify_source 随段）。
                        if state.desktop.shell_model == crate::ui::session::ShellModel::Outproc {
                            state.shell_watchdog_step();
                            if state.desktop.shell_degraded && state.desktop.shell_app.is_none() {
                                state.desktop.shell_degraded = false;
                                eprintln!(
                                    "[autodesk-shell] 降级回退 in-proc 装载（respawn 预算耗尽）"
                                );
                                if let Ok(shell_comp) = crate::ui::shell::build_shell_component() {
                                    state.desktop.shell_app = Some(state.allocate_app(shell_comp));
                                }
                                if let Ok(surface_comp) =
                                    crate::ui::shell::build_desktop_surface_component()
                                {
                                    state.desktop.desktop_app =
                                        Some(state.allocate_app(surface_comp));
                                }
                            }
                        }
                        {
                            let tasks = drain_desktop_bus_inbox(state);
                            drain_tasks.extend(tasks);
                        }
                        // Plan 508 G4：远程 WS 镜像泵（帧源=上方合成产物；
                        // 无监听/无在册镜像两调用皆空转）。
                        if state.pending_remote_connections() > 0 {
                            state.attach_pending_remotes();
                        }
                        state.pump_remote_mirrors();
                        // PLAN-760: remote-frame 宿主客户端泵（url 租约清理 /
                        // connect 收编 / Welcome/FrameReadyV2 帧槽 / EOF 重连
                        // ——空注册表零成本；帧推进节拍 = 本 400ms ServiceTick
                        // + presentation_keepalive_tick 重绘拍）。
                        crate::ui::iced::remote_frame_registry::pump();
                        if !drain_tasks.is_empty() {
                            return iced::Task::batch(drain_tasks);
                        }
                    }
                    // Plan 473 T6 / 505 A 族：原生槽位 WinEventHook 事件批
                    // （C4 拖走 / B7 回收；一拍 drain 排空、START/END 批内
                    // 优先——[`crate::ui::native_dock::drain_slot_events`]）。
                    DesktopEvent::NativeSlotEvents(events) => {
                        for evt in events {
                            handle_native_slot_event(state, evt.hwnd.0, evt.kind);
                        }
                    }
                    // Plan 500 步骤 4：像素桥协议入站——child 端点状态机
                    // （握手段落 / BufferAlloc 开段 / Close 生命周期）；
                    // 需要截图时对隐藏窗发起 window::screenshot（回调走
                    // PixelsShot 臂）。
                    DesktopEvent::PixelsProtocol(msg) => {
                        let Some(mut bridge) = state.pixels.take() else {
                            return iced::Task::none();
                        };
                        let component = state
                            .primary_app()
                            .and_then(|id| state.app_mut(id))
                            .map(|a| &mut a.component);
                        let (replies, want_capture) = bridge.on_protocol(*msg, component);
                        for reply in replies {
                            if !bridge.send(&reply) {
                                return iced::exit();
                            }
                        }
                        // 状态写回（快照注入）后重渲染再截图。
                        if want_capture {
                            if let Some(app) = state.primary_app() {
                                if let Some(a) = state.apps.get_mut(&app) {
                                    *a.state.view_dirty.borrow_mut() = true;
                                }
                            }
                        }
                        let mut tasks: Vec<iced::Task<crate::ui::session::DesktopMessage>> =
                            Vec::new();
                        if want_capture && bridge.request_capture() {
                            if let Some(win) =
                                state.primary_app().and_then(|app| state.window_of_app(app))
                            {
                                // PLAN-659 T-03：零维守卫（同 9652 host_ok
                                // 形态）——pixels 桥截图此前无守卫，最小化/
                                // 未布局窗直达 iced_wgpu offscreen 零维
                                // create_texture panic。
                                let win_ok = state
                                    .windows
                                    .get(&win)
                                    .map(|w| {
                                        let s = w.window_size.borrow();
                                        s.width > 0.0 && s.height > 0.0
                                    })
                                    .unwrap_or(false);
                                if win_ok {
                                    tasks.push(iced::window::screenshot(win).map(|ss| {
                                        crate::ui::session::DesktopMessage::Desktop(
                                            crate::ui::session::DesktopEvent::PixelsShot(ss),
                                        )
                                    }));
                                }
                            }
                        }
                        let detached = bridge.is_detached();
                        state.pixels = Some(bridge);
                        if detached {
                            return iced::exit();
                        }
                        return iced::Task::batch(tasks);
                    }
                    // Plan 500 步骤 4：像素桥截图回调——RGBA 写 shm 槽 +
                    // FrameReadyPixels 回发宿主（宿主合成接步骤 5）。
                    DesktopEvent::PixelsShot(ss) => {
                        let Some(bridge) = state.pixels.as_mut() else {
                            return iced::Task::none();
                        };
                        // 物理像素 → 表面逻辑尺寸（盒降采样）：shm 槽按
                        // 逻辑尺寸定档，HiDPI scale 经此对齐（497 快照同型）。
                        let (lw, lh) = bridge.size();
                        let lw = lw.max(1.0).ceil() as u32;
                        let lh = lh.max(1.0).ceil() as u32;
                        let rgba = crate::ui::desktop_protocol::pixels::downsample_to_logical(
                            ss.rgba.as_ref(),
                            ss.size.width,
                            ss.size.height,
                            lw,
                            lh,
                        );
                        let frame = crate::ui::desktop_protocol::pixels::PixelsFrame {
                            rgba,
                            w: lw,
                            h: lh,
                            stride: lw * 4,
                        };
                        if let Some(msg) = bridge.capture(frame) {
                            if !bridge.send(
                                &crate::ui::desktop_protocol::message::ProtocolMsg::Frame(msg),
                            ) {
                                return iced::exit();
                            }
                        }
                        if bridge.is_detached() {
                            return iced::exit();
                        }
                    }
                    // Plan 497：整窗截图回调——按 pending wid 集各窗 rect
                    // 裁剪入快照缓存（T1 定案裁剪式）；switcher 可见则置
                    // dirty（行 fallback → 真缩略升级；dock/pager hover 面
                    // 同随 dirty 重建）。
                    DesktopEvent::SnapshotShot(ss) => {
                        // PLAN-040 F3：裁剪判定前现算冻结态——入队到回调
                        // 之间窗可能刚最小化/被遮挡（400ms sync 未及），
                        // 用最新可见性裁决，杜绝污染帧入库。
                        sync_snapshot_frozen(state);
                        let wids =
                            std::mem::take(&mut *state.desktop.snapshot_pending_wids.borrow_mut());
                        if let Some(host) = state.host.as_ref() {
                            for wid in wids {
                                let Some(v) = host.wm.wins.get(&wid) else {
                                    continue;
                                };
                                // PLAN-040 F3：入队到回调之间窗可能已最小化/
                                // 被遮挡（冻结）——裁剪像素已非该窗内容，跳过
                                // 不入缓存（末帧保留语义）。
                                if crate::ui::iced::snapshot::is_frozen(wid) {
                                    continue;
                                }
                                let rect = *v.rect.borrow();
                                if let Some(snap) =
                                    crate::ui::iced::snapshot::thumbnail_from_screenshot(
                                        ss.rgba.as_ref(),
                                        ss.size.width,
                                        ss.size.height,
                                        rect,
                                        ss.scale_factor,
                                    )
                                {
                                    crate::ui::iced::snapshot::cache_put(wid, snap);
                                }
                            }
                        }
                        // 消费者 dirty：switcher（行 fallback→真缩略升级）
                        // + shell（dock/pager hover 预览面——miss 时渲染臂
                        // 已 request_capture，快照入库后需一次 view 重建
                        // 才升级；shell 置 dirty 幂等低频，400ms 抓取节拍
                        // 下可接受）。
                        if let Some(sw) = state.desktop.switcher_app {
                            if state.switcher_visible() {
                                if let Some(app) = state.apps.get_mut(&sw) {
                                    *app.state.view_dirty.borrow_mut() = true;
                                }
                            }
                        }
                        if let Some(shell) = state.desktop.shell_app {
                            if let Some(app) = state.apps.get_mut(&shell) {
                                *app.state.view_dirty.borrow_mut() = true;
                            }
                        }
                    }
                    // Plan 486：拖入高亮注入面（正常流由事件驱动直写；本臂供
                    // E2E/headless 验证 overlay 渲染）。
                    DesktopEvent::NativeDragOver(rect) => {
                        set_native_drag_over(state, rect);
                    }
                    // Plan 464 T4：launcher 召唤消费 —— 懒挂载 + 下行注入 +
                    // 打开即聚焦（返回聚焦任务）。
                    DesktopEvent::SummonLauncher => {
                        return summon_launcher(state);
                    }
                    // Plan 478 T4：Ctrl+Tab 改道——switcher 可见 → .Advance
                    // 直投（选中环走，仅 sel 变更不触发 drain）；否则召唤。
                    // PLAN-526 T32：召唤后紧接一次 .Advance——首按即预选
                    // 下一个窗口（Windows Alt-Tab 语义；RebuildMru 复位
                    // sel=0=当前窗，预选顺延一位），松开 Ctrl 提交（见
                    // __modifiers_changed 臂），按住期间再按继续推进。
                    // PLAN-013 W1 根修：直投消息必须带 widget 名——空名
                    // 解析 `handler__Advance` 不在 exports（实机调试日志
                    // CALL_HANDLER_FOR_NOT_FOUND 钉死），按住 Ctrl 连按
                    // Tab 的推进全部静默失败；箭头走 switcher 自 bind
                    // （handler_Switcher_Advance）故有效。
                    // PLAN-019 v1.7：picker 键盘导航（订阅层门控产出；同落
                    // wallpaper_nav 执行体——栅格态 flip 应用/预览态游标）。
                    DesktopEvent::WallpaperKeyNav(dir) => {
                        execute_wallpaper_nav(state, dir);
                    }
                    // PLAN-019 v1.7：picker Esc 链——预览态回栅格；栅格态
                    // 关闭（close 臂按归属簿记决定是否自动返回）。
                    DesktopEvent::WallpaperKeyEscape => execute_wallpaper_escape(state),
                    DesktopEvent::SummonSwitcher => {
                        // PLAN-036 T-04（D6）：outproc 轨——可见时 Advance
                        /// 事件快照（child dispatch → SwitcherMsg::Advance）；
                        /// 未可见 = 首召唤（switcher_open 置位 + RebuildMru +
                        /// Advance 首步——in-proc 首按推进语义同册）。
                        if state.desktop.shell_pipe.is_some() {
                            if state.desktop.switcher_open {
                                push_switcher_snapshot(
                                    state,
                                    &[crate::ui::shell_projection::ShellEvent::Advance],
                                );
                            } else {
                                state.desktop.switcher_open = true;
                                push_switcher_snapshot(
                                    state,
                                    &[
                                        crate::ui::shell_projection::ShellEvent::RebuildMru,
                                        crate::ui::shell_projection::ShellEvent::Advance,
                                    ],
                                );
                            }
                            return iced::Task::none();
                        }
                        // PLAN-013 W1 诊断:每按记录 visible/sel 前值——区分
                        // 「重召唤复位」与「推进失败」两种病灶。
                        if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
                            let dbg = state
                                .desktop
                                .switcher_app
                                .and_then(|sw| state.apps.get(&sw))
                                .and_then(|a| {
                                    a.component
                                        .read_state("sel")
                                        .ok()
                                        .zip(a.component.read_state("visible").ok())
                                })
                                .map(|(sel, vis)| (sel.to_string(), vis.to_string()))
                                .unwrap_or_else(|| ("?".into(), "?".into()));
                            eprintln!(
                                "[P013-W1] SummonSwitcher: visible={} sel={} (app mounted={})",
                                dbg.1,
                                dbg.0,
                                state.desktop.switcher_app.is_some()
                            );
                        }
                        if state.switcher_visible() {
                            if let Some(sw) = state.desktop.switcher_app {
                                let msg = IcedMessage {
                                    widget: "Switcher".to_string(),
                                    event: "Advance".to_string(),
                                    input_value: None,
                                };
                                return update_inner(state, sw, msg, false)
                                    .map(move |m| DM::App(sw, m));
                            }
                        }
                        let task = summon_switcher(state);
                        if let Some(sw) = state.desktop.switcher_app {
                            let msg = IcedMessage {
                                widget: "Switcher".to_string(),
                                event: "Advance".to_string(),
                                input_value: None,
                            };
                            let advance =
                                update_inner(state, sw, msg, false).map(move |m| DM::App(sw, m));
                            return iced::Task::batch([task, advance]);
                        }
                        return task;
                    }
                    // Plan 488 步骤 4/6 → 505 D（债 P488-D4 清偿）：拖出完成 →
                    // on_dnd_finished 交付**发起时锚定 App**（dispatch 环置位；
                    // DoDragDrop 在发起方 handler 内联阻塞，完成时焦点通常
                    // 即发起方，锚定消除偏差场景）；无锚回退完成时焦点
                    // （v1 语义保持）。
                    DesktopEvent::DndFinished { effect } => {
                        if let Some(app) = dnd_finished_target(state) {
                            inject_native_event(
                                state,
                                app,
                                "on_dnd_finished",
                                &[auto_val::Value::str(effect)],
                            );
                        }
                    }
                    // Plan 488 步骤 5/6：拖入落点——屏幕物理坐标 → 宿主逻辑域
                    // （drag_mapper 同源）→ z 序命中 → AppId → on_native_drop
                    // 注入（payload Record：text/files/image whichever）。
                    // headless/无宿主窗时坐标按恒等退化。
                    DesktopEvent::NativeDrop(data) => {
                        if std::env::var("AUTO_DND_TRACE").map_or(false, |v| v == "1") {
                            eprintln!("[dnd-hop3] update arm NativeDrop enter");
                        }
                        let local = (data.screen_x as f32, data.screen_y as f32);
                        #[cfg(windows)]
                        let local = if let Some((mapper, _)) = drag_mapper() {
                            let l = mapper.screen_to_local(crate::ui::native_dock::Rect::new(
                                data.screen_x,
                                data.screen_y,
                                0,
                                0,
                            ));
                            (l.x as f32, l.y as f32)
                        } else {
                            local
                        };
                        let hit = state.drop_hit_app_at_local(local.0, local.1);
                        if std::env::var("AUTO_DND_TRACE").map_or(false, |v| v == "1") {
                            eprintln!(
                                "[dnd-hit] local=({:.0},{:.0}) app={:?} widget={:?}",
                                local.0,
                                local.1,
                                hit,
                                hit.and_then(|a| state
                                    .apps
                                    .get(&a)
                                    .map(|s| s.component.widget_name().to_string()))
                            );
                        }
                        if let Some(app) = hit {
                            if let Some(sess) = state.app_mut(app) {
                                let res = sess.component.bridge_mut().call_handler_with_record(
                                    "on_native_drop",
                                    native_drop_record_fields(&data),
                                );
                                if std::env::var("AUTO_DND_TRACE").map_or(false, |v| v == "1") {
                                    eprintln!("[dnd-hop4] handler result: {res:?}");
                                }
                            }
                        }
                    }
                    // Plan 488 T7：桌面级 Ctrl+V → 读 OS 剪贴板（418 文本 →
                    // 485 文件/图片）→ on_native_paste 注入焦点 App。
                    DesktopEvent::NativePaste => {
                        if let Some(app) = state.wm_focused_app().or_else(|| state.primary_app()) {
                            if let Some(sess) = state.app_mut(app) {
                                let _ = sess.component.bridge_mut().call_handler_with_record(
                                    "on_native_paste",
                                    clipboard_paste_record_fields(),
                                );
                            }
                        }
                    }
                }
                iced::Task::none()
            }
            // PLAN-765 T-02（纪要一 F-4）：HTTP 消息桥回填臂——经
            // dispatch_app(.., true) 走 on_http_backfill_for seq 门控派发。
            // dashboard 拦截器/DnD 锚定等 App 臂内务与回填无关，直取通用
            // 尾（dispatch + bus 排空 + exit/sync + task 并批，与 App 臂同形）。
            DM::HttpBackfill(app_id, m) => {
                let task = dispatch_app(state, app_id, m, true);
                let (exit, mut tasks) = drain_and_execute_desktop_commands(state);
                if exit {
                    state.shutdown_broker();
                    return iced::exit();
                }
                if state.desktop.shell_app.is_some() {
                    sync_shell_windows(state);
                }
                tasks.push(task);
                iced::Task::batch(tasks)
            }
            DM::App(app_id, m) => {
                // PLAN-024：dashboard 占位卡一键 launch 拦截臂——face 叠合层
                // 宿主合成消息（`__dashboard_launch:<registry-id>`）直投面板
                // App；写入面板 `__dashboard_cmd` 后走同周期 bus 排空统一
                // 执行（面板 .at 零感知、无 handler 不产生派发噪音）。尾与
                // 常规臂同形（exit/sync/batch）。
                if let Some(launch_id) = m
                    .event
                    .strip_prefix("__dashboard_launch:")
                    .map(str::to_string)
                {
                    // PLAN-036 T-06（B2）：outproc 卡叠层 = 宿主合成面，
                    // 目标 AppId(0)（面板 App 不在场）——命令直入
                    // desktop_bus_inbox（panel 写点旁路，drain 统一执行）。
                    if state.desktop.shell_pipe.is_some() && app_id.0 == 0 {
                        state.desktop.desktop_bus_inbox.push((
                            "dashboard-face".into(),
                            crate::ui::session::DesktopCommand::LaunchApp(launch_id.clone()),
                        ));
                        let (exit, mut tasks) = drain_and_execute_desktop_commands(state);
                        if exit {
                            state.shutdown_broker();
                            return iced::exit();
                        }
                        if state.desktop.shell_app.is_some() {
                            sync_shell_windows(state);
                        }
                        return iced::Task::batch(tasks);
                    }
                    if state.desktop.dashboard_app == Some(app_id) {
                        if let Some(panel) = state.desktop.dashboard_app {
                            if let Some(app) = state.apps.get_mut(&panel) {
                                let _ = app.component.write_state(
                                    "__dashboard_cmd",
                                    auto_val::Value::str(&format!("dashboard_launch\t{launch_id}")),
                                );
                                *app.state.view_dirty.borrow_mut() = true;
                            }
                        }
                        let (exit, mut tasks) = drain_and_execute_desktop_commands(state);
                        if exit {
                            state.shutdown_broker();
                            return iced::exit();
                        }
                        if state.desktop.shell_app.is_some() {
                            sync_shell_windows(state);
                        }
                        return iced::Task::batch(tasks);
                    }
                }
                // PLAN-024 R20：卡片点击打开 app——合成消息
                // `__dashboard_open:<registry-id>`（face 层 mouse-area 发
                // 出，带 face app 标签）。三态：孵化会话 → 升格开窗
                // （face/窗同会话零分家）；已有窗 → activate 聚焦（跨分区
                // 语义复用）；无会话 → launch。尾与常规臂同形。
                if let Some(open_id) = m
                    .event
                    .strip_prefix("__dashboard_open:")
                    .map(str::to_string)
                {
                    // 判定顺序 = 去重语义（R21）：已有窗 → 聚焦（多次打开
                    // 不重复开窗）；其次孵化会话 → 升格开窗；最后 → launch。
                    if dashboard_running_app(state, &open_id).is_some() {
                        // PLAN-036 T-06（B2）：outproc 直入 inbox（launch 臂
                        // 同册——running 分支的 panel 写点旁路）。
                        if state.desktop.shell_pipe.is_some() && app_id.0 == 0 {
                            state.desktop.desktop_bus_inbox.push((
                                "dashboard-face".into(),
                                crate::ui::session::DesktopCommand::ActivateApp(open_id.clone()),
                            ));
                        } else {
                            if let Some(panel) = state.desktop.dashboard_app {
                                if let Some(app) = state.apps.get_mut(&panel) {
                                    let _ = app.component.write_state(
                                        "__dashboard_cmd",
                                        auto_val::Value::str(&format!("activate	{open_id}")),
                                    );
                                    *app.state.view_dirty.borrow_mut() = true;
                                }
                            }
                        }
                    } else if let Some(hatched) = state.hatched_mini_of(&open_id) {
                        if let Err(err) = state.open_window_for_session(&open_id, hatched) {
                            eprintln!("[session] dashboard promote failed: {err}");
                        }
                    } else {
                        execute_launch_app(state, &open_id);
                    }
                    let (exit, mut tasks) = drain_and_execute_desktop_commands(state);
                    if exit {
                        state.shutdown_broker();
                        return iced::exit();
                    }
                    if state.desktop.shell_app.is_some() {
                        sync_shell_windows(state);
                    }
                    return iced::Task::batch(tasks);
                }
                // PLAN-002 N6b 取证探针（AUTO_POPOVER_DEBUG=1；定案后移除）。
                if std::env::var("AUTO_POPOVER_DEBUG").as_deref() == Ok("1")
                    && (m.event == "MenuClose"
                        || m.event == "MenuOpen"
                        || m.event.starts_with("IconMenu"))
                {
                    eprintln!(
                        "[pv-update] DM::App reached daemon update: app={app_id:?} event={}",
                        m.event
                    );
                }
                // Plan 505 D（债 P488-D4）：拖出发起方锚定——DoDragDrop 在
                // App handler dispatch 内联阻塞至完成（488 步骤 9 定案），
                // 代际变化即本次 dispatch 发起并完成了一次拖出。
                let dnd_gen_before = dnd_drag_generation();
                let task = dispatch_app(state, app_id, m, false);
                maybe_anchor_dnd_initiator(state, app_id, dnd_gen_before, dnd_drag_generation());
                // Plan 463 T4：DesktopBus 排空 #2 —— handler 本周期写入的
                // 命令（按钮 onclick → __desktop_cmd）同周期尾即达，
                // 按钮路径不受帧泵节拍限制（T1 报告 §2.3）。
                // Plan 464 T4：shell + launcher 联合排空；summon 任务与
                // 本消息任务 batch 回 iced。
                let (exit, mut tasks) = drain_and_execute_desktop_commands(state);
                if exit {
                    // Plan 505 B5（债 P480-R1）：退出前显式
                    // 停机 broker serve 线程。
                    state.shutdown_broker();
                    return iced::exit();
                }
                if state.desktop.shell_app.is_some() {
                    // shell 自身按钮路径：本周期命令已执行，列表即时同步。
                    sync_shell_windows(state);
                }
                tasks.push(task);
                iced::Task::batch(tasks)
            }
            DM::Wm(cmd) => {
                // Plan 462：WM 命令臂。独立模式不产生该变体（防御性忽略，
                // I3：不设第二管线，仅配置位门控）。
                // PLAN-646：Alt+左键按下 = Select Anything 框选起笔（双模式
                // 生效，先于 is_desktop 门）。命中即吞掉本次 GlobalPress——
                // 抑制 WM 焦点抢占/命中聚焦，避免起笔瞬间 z 序重排。
                if matches!(cmd, crate::ui::session::WmCommand::GlobalPress)
                    && selection_press_anchor(state)
                {
                    return iced::Task::none();
                }
                if !state.is_desktop() {
                    return iced::Task::none();
                }
                match cmd {
                    // Plan 463 T3：Esc = 桌面调试退出（全屏无框无关闭按钮）。
                    // Plan 464 T4：launcher 可见时 Esc 归 launcher 逐层退出
                    // （P3：清词→退网格→关闭；app 内 bind 路径处理），不退桌面。
                    WmCommand::ExitDesktop => {
                        // Plan 464 T4：launcher 可见时 Esc 归 launcher 逐层
                        // 退出（P3：清词→退网格→关闭）。Plan 478 T4：switcher
                        // 可见时 Esc 归 switcher 自隐（app 内 bind 路径处理），
                        // 不退桌面。Plan 479 T3：通知中心可见时同理自隐。
                        // Plan 540 T9：设置 overlay 退役——Esc 不再与设置
                        // 相干（设置 = 普通窗，×关闭）。
                        if state.launcher_visible()
                            || state.switcher_visible()
                            || state.notification_visible()
                            || state.dashboard_visible()
                        {
                            // PLAN-036 T-04（D6）：outproc 轨 overlay 自隐
                            /// ——Escape 事件快照（面板 .at Escape handler 置
                            /// visible=0 + 镜像位复位；in-proc 走 app 内 bind
                            /// 路径不动）。
                            if state.desktop.shell_pipe.is_some() {
                                if state.desktop.switcher_open {
                                    state.desktop.switcher_open = false;
                                    push_switcher_snapshot(
                                        state,
                                        &[crate::ui::shell_projection::ShellEvent::Escape],
                                    );
                                }
                                if state.desktop.notes_open {
                                    state.desktop.notes_open = false;
                                    push_notes_snapshot(
                                        state,
                                        &[crate::ui::shell_projection::ShellEvent::Escape],
                                    );
                                }
                                // PLAN-036 T-05（B2）：dashboard 同册。
                                if state.desktop.dashboard_open {
                                    state.desktop.dashboard_open = false;
                                    push_dashboard_snapshot(
                                        state,
                                        &[crate::ui::shell_projection::ShellEvent::Escape],
                                    );
                                }
                                // PLAN-036 T-07（B3）：launcher 同册（.at
                                // Escape handler 清词→退格→自隐逐级）。
                                if state.desktop.launcher_open {
                                    state.desktop.launcher_open = false;
                                    push_launcher_snapshot(
                                        state,
                                        &[crate::ui::shell_projection::ShellEvent::Escape],
                                    );
                                }
                            }
                            return iced::Task::none();
                        }
                        // Plan 473 T6 / B8：退出不吞窗口——全部 docked 槽位
                        // 恢复 pre-dock bounds/样式后再退出。
                        restore_all_native_slots(state);
                        // Plan 505 B5（债 P480-R1）：退出前显式
                        // 停机 broker serve 线程。
                        state.shutdown_broker();
                        return iced::exit();
                    }
                    // 全局左键按下：按最近光标位置做 z 序命中测试 → 软聚焦
                    //（last_cursor 由全局 CursorMoved 持续回写；点击桌面空白
                    // 不改焦点，其语义归 463 桌面层）。
                    // PLAN-526 T20：点击聚焦改 `focus_soft`——只记焦点/
                    // 还原最小化，置顶推迟到 `__mouse_released` 偿还。根因：
                    // press→release 之间 z_order 重排会改变桌面 Stack 层序，
                    // `Tree::diff_children` 按位 diff（iced 0.14 无 Id 匹配）
                    // 弄丢 button 持有的 is_pressed 态——跨窗首击只切焦点
                    // 不触发按钮。标题栏拖拽/缩放的即时置顶由 StartDrag/
                    // StartResize 臂自理（彼时命中的是 chrome，无按钮在途）。
                    WmCommand::GlobalPress => {
                        // PLAN-526 T37 合同补线：任意全局左键按下先撤标题
                        // 右键菜单（点外关闭——icon/任务栏/blank 菜单
                        // popover ondismiss 的 chrome 同语义面）。落在菜单
                        // 项上的按压同样经此关菜单，项动作消息同帧照常
                        // 执行（订阅面原始事件与 widget 消息同帧全送达）。
                        // 此前只有 open/TitleMenuClose 两个写点，合同注释
                        // 在案但从未接线——菜单一旦打开仅显式命令可撤。
                        if let Some(host) = state.host.as_mut() {
                            host.wm.title_menu = None;
                        }
                        let cursor = state
                            .host
                            .as_ref()
                            .map(|h| h.wm.last_cursor.get())
                            .unwrap_or_default();
                        if let Some(wid) = state
                            .host
                            .as_ref()
                            .and_then(|h| h.wm.hit_test(cursor.x, cursor.y))
                        {
                            state.wm_focus_soft(wid);
                            // PLAN-030 T-04：pointer 生产接线（D3 缺口清偿）——
                            // 命中窗属 broker 客户端（含壳伪窗）时投递
                            // PointerPressed（坐标平移窗局部系；软聚焦不动
                            // z——与 in-proc 按钮在途语义一致）。
                            let is_broker = state.broker_clients.values().any(|c| c.owns_wid(wid));
                            if is_broker {
                                state.broker_pointer_down(
                                    cursor.x,
                                    cursor.y,
                                    crate::ui::desktop_protocol::message::MouseButton::Left,
                                );
                            }
                        }
                    }
                    // PLAN-526 T37：标题栏右键菜单开/关（chrome 自绘浮层，
                    // 见 virtual_window.rs 菜单层；任意 GlobalPress 已清）。
                    WmCommand::TitleMenuOpen(wid) => {
                        // PLAN-012：菜单锚点随右键落点——last_cursor（host
                        // 逻辑坐标）减窗 rect 原点得窗内坐标，经钳制纯函数
                        // 定面板左上角（窗缘不越界，y 不高于标题条底）。
                        if let Some(host) = state.host.as_mut() {
                            let cursor = host.wm.last_cursor.get();
                            let rect = host.wm.wins.get(&wid).map(|v| *v.rect.borrow()).unwrap_or(
                                iced::Rectangle {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 0.0,
                                    height: 0.0,
                                },
                            );
                            let (x, y) = crate::ui::iced::virtual_window::title_menu_spot(
                                cursor.x,
                                cursor.y,
                                rect.x,
                                rect.y,
                                rect.width,
                                rect.height,
                            );
                            host.wm.title_menu =
                                Some(crate::ui::session::TitleMenuSpot { wid, x, y });
                        }
                        // PLAN-012：右键标题栏先聚焦置顶（`focus` = 立即
                        // z_order 刷新）——非最前窗的菜单层在自身 vwin
                        // Stack 内，不置顶会被前窗遮挡；右键无 in-flight
                        // 按钮态，无 T20 press→release diff 风险，故用即时
                        // focus 而非软聚焦（左键路径仍走 GlobalPress 软
                        // 聚焦+release 偿还，双击首击即聚焦）。
                        state.wm_focus(wid);
                    }
                    WmCommand::TitleMenuClose => {
                        if let Some(host) = state.host.as_mut() {
                            host.wm.title_menu = None;
                        }
                    }
                    WmCommand::Focus(wid) => state.wm_focus(wid),
                    // Plan 463 T6：窗口循环（Alt+Tab/Ctrl+Tab 热键臂）。
                    WmCommand::CycleWindow => {
                        state.wm_cycle_focus();
                    }
                    // Plan 463 T6：布局切换热键臂（与 shell 总线
                    // DesktopCommand::SetLayout 同落 wm_set_layout；
                    // 臂尾 sync_shell_windows 即时刷新任务栏按钮态）。
                    WmCommand::SetLayout(mode) => {
                        state.wm_set_layout(mode);
                        // Plan 473 T6：热键 relayout 即刻排水槽位几何。
                        sync_native_geometry(state);
                    }
                    // Plan 473 T6：槽位框 chrome 按钮（最小化/关闭——
                    // 关闭走 WM_CLOSE 给目标 app 正常关闭机会）。
                    WmCommand::NativeSlotMin(id) => {
                        #[cfg(windows)]
                        if let Some(slot) =
                            state.host.as_ref().and_then(|h| h.wm.native_slots.get(&id))
                        {
                            use crate::ui::native_dock::win32 as ndw;
                            let _ = ndw::show_window(slot.hwnd, ndw::ShowMode::Minimize);
                        }
                        #[cfg(not(windows))]
                        let _ = id;
                    }
                    WmCommand::NativeSlotClose(id) => {
                        #[cfg(windows)]
                        if let Some(slot) =
                            state.host.as_ref().and_then(|h| h.wm.native_slots.get(&id))
                        {
                            let _ = crate::ui::native_dock::win32::request_close(slot.hwnd);
                        }
                        #[cfg(not(windows))]
                        let _ = id;
                    }
                    // PLAN-709：槽位标题栏按下——聚焦置顶（伪 Wid 域，带内
                    // OS 置顶在 restack 排水拍生效）+ 进槽位拖拽（grab 按
                    // last_cursor 现算，StartDrag 同型 renderer.rs:20441）。
                    WmCommand::NativeSlotStartDrag { slot_id } => {
                        state.wm_focus_native_slot(slot_id);
                        if let Some(host) = state.host.as_mut() {
                            if let Some(origin) = host
                                .wm
                                .native_slot_local_rects
                                .get(&slot_id)
                                .map(|r| r.position())
                            {
                                let grab = host.wm.last_cursor.get();
                                host.wm.interaction =
                                    Some(crate::ui::session::WmInteraction::NativeDrag {
                                        slot_id,
                                        grab: iced::Point::new(
                                            grab.x - origin.x,
                                            grab.y - origin.y,
                                        ),
                                    });
                            }
                        }
                    }
                    // PLAN-709：槽位把手按下——八向缩放；min_size 起点
                    // 折算（min_size_est 物理域 ÷ DPI + 160/120 地板）。
                    WmCommand::NativeSlotStartResize { slot_id, edge } => {
                        state.wm_focus_native_slot(slot_id);
                        if let Some(host) = state.host.as_mut() {
                            let scale = native_slot_dpi_scale();
                            let est = host
                                .wm
                                .native_slots
                                .get(&slot_id)
                                .and_then(|s| s.min_size_est)
                                .map(|s| iced::Size::new(s.w as f32 / scale, s.h as f32 / scale))
                                .unwrap_or_default();
                            if let Some(start_rect) =
                                host.wm.native_slot_local_rects.get(&slot_id).copied()
                            {
                                host.wm.interaction =
                                    Some(crate::ui::session::WmInteraction::NativeResize {
                                        slot_id,
                                        edge,
                                        start_rect,
                                        start_cursor: host.wm.last_cursor.get(),
                                        min_size: iced::Size::new(
                                            160.0_f32.max(est.width),
                                            120.0_f32.max(est.height),
                                        ),
                                    });
                            }
                        }
                    }
                    // PLAN-709：槽位聚焦——WM 侧置顶 + OS 前台（486
                    // focus_native 执行臂同体；SW_RESTORE 先行）。
                    WmCommand::NativeSlotFocus(id) => {
                        state.wm_focus_native_slot(id);
                        #[cfg(windows)]
                        {
                            use crate::ui::native_dock::win32 as ndw;
                            if let Some(slot) =
                                state.host.as_ref().and_then(|h| h.wm.native_slots.get(&id))
                            {
                                if ndw::is_minimized(slot.hwnd) {
                                    let _ = ndw::show_window(slot.hwnd, ndw::ShowMode::Restore);
                                }
                                let _ = ndw::focus_window(slot.hwnd);
                            }
                        }
                        #[cfg(not(windows))]
                        let _ = id;
                    }
                    // Plan 472 T2：分区切换热键臂（同落 WmState 分区方法；
                    // 臂尾 sync_shell_windows 即时刷新投影）。
                    // Plan 472 T2：分区切换热键臂（同落 WmState 分区方法；
                    // 臂尾 sync_shell_windows 即时刷新投影）。
                    // PLAN-526 T18：热键切换即 transient 显示切换预览面板
                    // （~1.6s 自动收起，ServiceTick 到期臂）。
                    WmCommand::NextWorkspace => {
                        state.wm_next_workspace();
                        show_workspace_switcher(state);
                    }
                    WmCommand::PrevWorkspace => {
                        state.wm_prev_workspace();
                        show_workspace_switcher(state);
                    }
                    // Plan 478 T2：发送聚焦窗到相邻分区（Ctrl+Alt+Shift+←/→；
                    // 目标 = 环切对称，与 next/prev_workspace 同肌肉记忆）。
                    WmCommand::SendFocusedTo(step) => {
                        use crate::ui::session::WorkspaceStep;
                        let target = state.host.as_ref().and_then(|h| {
                            let len = h.wm.workspaces.len();
                            if len == 0 {
                                return None;
                            }
                            let cur = h.wm.current_workspace;
                            let t = match step {
                                WorkspaceStep::Prev => (cur + len - 1) % len,
                                WorkspaceStep::Next => (cur + 1) % len,
                            };
                            h.wm.focused.map(|w| (w, t))
                        });
                        if let Some((wid, t)) = target {
                            // PLAN-709：焦点域含槽位（伪 Wid）——统一成员臂。
                            state.wm_move_member_to_workspace(wid, t);
                            sync_native_geometry(state);
                        }
                    }
                    // 标题栏按下 = 聚焦置顶 + 进入拖拽（grab 偏移按
                    // last_cursor 现算；后续 move/release 走 DM::Window 拦截）。
                    // PLAN-526 T2：拖动最大化窗即还原（回 restore_rect 跟手）。
                    WmCommand::StartDrag { wid } => {
                        state.wm_focus(wid);
                        if let Some(host) = state.host.as_mut() {
                            host.wm.unmaximize_for_interaction(wid, true);
                            let grab = host.wm.last_cursor.get();
                            let origin = host
                                .wm
                                .wins
                                .get(&wid)
                                .map(|v| v.rect.borrow().position())
                                .unwrap_or_default();
                            host.wm.interaction = Some(crate::ui::session::WmInteraction::Drag {
                                wid,
                                grab: iced::Point::new(grab.x - origin.x, grab.y - origin.y),
                            });
                        }
                    }
                    // PLAN-526 T2：缩放启动即退出最大化（仅清标志，新 rect
                    // 以缩放结果为准不回跳）。
                    WmCommand::StartResize { wid, edge } => {
                        state.wm_focus(wid);
                        if let Some(host) = state.host.as_mut() {
                            host.wm.unmaximize_for_interaction(wid, false);
                            host.wm.interaction = Some(crate::ui::session::WmInteraction::Resize {
                                wid,
                                edge,
                                start_rect: host
                                    .wm
                                    .wins
                                    .get(&wid)
                                    .map(|v| *v.rect.borrow())
                                    .unwrap_or_default(),
                                start_cursor: host.wm.last_cursor.get(),
                            });
                        }
                    }
                    // PLAN-526 T2：最小化到任务栏（标题栏 `–`/右键菜单/总线
                    // win_min）；任务栏 icon 点击 = WinFocus → focus 还原。
                    WmCommand::Minimize(wid) => state.wm_minimize(wid),
                    // PLAN-526 T2：最大化/还原切换（标题栏 `▢`/右键菜单；
                    // 可用区 = viewport 扣任务栏，见 wm_toggle_maximize）。
                    WmCommand::ToggleMaximize(wid) => state.wm_toggle_maximize(wid),
                    // 关闭虚拟窗口；App 随窗移除（459 一窗一 App 不变式的
                    // WM 化，见 459 update 注释预告）。全部虚拟窗关闭：
                    // 无 shell = daemon 语义退出进程（462 行为不变）；有
                    // shell = 空桌面合法态（任务栏存活，等待再启动；463）。
                    WmCommand::Close(wid) => {
                        if let Some(app) = state.wm_remove_win(wid) {
                            // 2026-10-10 用户裁定（小组件独立于 app 窗配套）：
                            // 孵化 mini 会话的升格窗关闭 = **降格**不杀——窗
                            // 摘除即降格回 face 垫片渲染（split_ref_face 无窗
                            // 臂自动回落），会话与后端供给原样常驻（小组件
                            // 继续活：音乐不因关窗停播、Todo 计数照旧）。
                            // hatched_minis 反查命中才降格（孵化会话无窗时
                            // 不可能进本臂；普通 launch 窗照旧随窗移除 +
                            // 释放后端）。
                            let demote = state.desktop.hatched_minis.values().any(|a| *a == app);
                            if demote {
                                // 仅摘窗（wm_remove_win 已做），会话保留。
                            } else {
                                state.apps.remove(&app);
                                // PLAN-037 T-04：后端供给随窗释放（计数归零即
                                // 卸载；标题栏 × 的 WM 主关闭路径）。
                                state.release_backend(app);
                            }
                        }
                        if state.desktop.shell_app.is_none()
                            && state
                                .host
                                .as_ref()
                                .map(|h| h.wm.wins.is_empty())
                                .unwrap_or(false)
                        {
                            // Plan 505 B5（债 P480-R1）：退出前显式
                            // 停机 broker serve 线程。
                            state.shutdown_broker();
                            return iced::exit();
                        }
                    }
                }
                // Plan 463 T5：WM 状态变化（聚焦/拖拽落位/关闭/布局）当周期
                // 即时同步任务栏（指纹门控，无变化时零写）。
                sync_shell_windows(state);
                // PLAN-709：槽位聚焦置顶的带内 OS 重申随批收口
                // （pending_native_restack 旗标；无旗标无 pending = 零开销）。
                sync_native_geometry(state);
                iced::Task::none()
            }
            DM::Window(win, m) => {
                // PLAN-002 N6b 取证探针（AUTO_POPOVER_DEBUG=1；定案后移除）。
                if std::env::var("AUTO_POPOVER_DEBUG").as_deref() == Ok("1")
                    && (m.event == "MenuClose"
                        || m.event == "MenuOpen"
                        || m.event.starts_with("IconMenu"))
                {
                    eprintln!(
                        "[pv-window] DM::Window reached: win={win:?} event={}",
                        m.event
                    );
                }
                // Plan 462 desktop：全局光标事件优先驱动 WM 拖拽/缩放状态机。
                // 交互进行中消费事件（不下发 App——app 内 divider 拖拽与 WM
                // 拖拽互斥，桌面语义下 WM 优先）。
                if state.is_desktop()
                    && matches!(
                        m.event.as_str(),
                        "__mouse_moved" | "__mouse_released" | "__modifiers_changed"
                    )
                {
                    match m.event.as_str() {
                        "__mouse_moved" => {
                            if let Some(ref val) = m.input_value {
                                if let Some((xs, ys)) = val.split_once(',') {
                                    let x: f32 = xs.parse().unwrap_or(0.0);
                                    let y: f32 = ys.parse().unwrap_or(0.0);
                                    // PLAN-012：光标坐标喂桌面面 VM（空白菜
                                    // 单坐标锚数据源）——只写状态不置
                                    // view_dirty，blank_menu open 翻转才重
                                    // 建视图读取最新值，零逐帧重建成本。
                                    // 2026-09-15：拖拽进行中——超阈值即置
                                    // moved 旗标（两路落格动词的门），并把
                                    // 光标所在格回写 drop_c/drop_r（落点格
                                    // 高亮数据面）；置 view_dirty 让坐标锚
                                    // 幽灵 popover 跟随（拖拽结束即恢复零
                                    // 逐帧重建）。
                                    let mut drag_active = false;
                                    if let Some((_, (ox, oy))) = &state.desktop.icon_drag {
                                        let dist = ((x - ox).powi(2) + (y - oy).powi(2)).sqrt();
                                        if dist >= 6.0 {
                                            state.desktop.icon_drag_moved = true;
                                            drag_active = true;
                                        }
                                    }
                                    // PLAN-030 T-06：outproc 壳 cursor 接泵
                                    // （app 借用外执行——消费门 = 命中
                                    // background 伪窗[桌面空白区 = 空白菜单/
                                    // 拖拽作用域；菜单开态在 child 面内 host
                                    // 不可见，空白区语义精确覆盖]或拖拽进行中，
                                    // D3 节流定案）。
                                    let mut shell_cursor_push = false;
                                    if state.desktop.shell_pipe.is_some() {
                                        let bg_hit = state
                                            .desktop
                                            .shell_pseudo_wids
                                            .first()
                                            .copied()
                                            .is_some_and(|w| {
                                                state
                                                    .host
                                                    .as_ref()
                                                    .and_then(|h| h.wm.hit_test(x, y))
                                                    == Some(w)
                                            });
                                        shell_cursor_push = drag_active || bg_hit;
                                    }
                                    if let Some(surface) = state.desktop.desktop_app {
                                        if let Some(app) = state.apps.get_mut(&surface) {
                                            let _ = app.component.write_state(
                                                "__desktop_cursor_x",
                                                auto_val::Value::Float(x as f64),
                                            );
                                            let _ = app.component.write_state(
                                                "__desktop_cursor_y",
                                                auto_val::Value::Float(y as f64),
                                            );
                                            if drag_active {
                                                // 与 execute_desktop_icon_drop_at
                                                // 同一套栅格换算（12px 面内边距 +
                                                // 88/80px 栅距）。
                                                let col = (((x - 12.0) / 88.0).floor() as i32)
                                                    .clamp(0, 7);
                                                let row = (((y - 12.0) / 80.0).floor() as i32)
                                                    .clamp(0, 96);
                                                let _ = app.component.write_state(
                                                    "drop_c",
                                                    auto_val::Value::str(col.to_string()),
                                                );
                                                let _ = app.component.write_state(
                                                    "drop_r",
                                                    auto_val::Value::Str(row.to_string().into()),
                                                );
                                                // 2026-09-15：幽灵 popover 开门条件
                                                // （drag_id 按下即置位，点击会闪副本
                                                // ——改用 drag_moved 门，真拖起来才显示）。
                                                let _ = app.component.write_state(
                                                    "drag_moved",
                                                    auto_val::Value::str("1"),
                                                );
                                                *app.state.view_dirty.borrow_mut() = true;
                                            }
                                        }
                                    }
                                    if shell_cursor_push {
                                        use crate::ui::desktop_protocol::message::{
                                            shell_face, ControlMsg, ProtocolMsg,
                                        };
                                        let msg =
                                            ProtocolMsg::Control(ControlMsg::ShellCursorMove {
                                                face: shell_face::DESKTOP_SURFACE,
                                                x,
                                                y,
                                            });
                                        let _ = state.push_shell_control(&msg);
                                    }
                                    let host_size = state
                                        .host
                                        .as_ref()
                                        .and_then(|h| state.windows.get(&h.window))
                                        .map(|e| *e.window_size.borrow())
                                        .unwrap_or(iced::Size::new(1280.0, 800.0));
                                    if state
                                        .host
                                        .as_mut()
                                        .map(|h| h.wm.apply_cursor(x, y, host_size))
                                        .unwrap_or(false)
                                    {
                                        // PLAN-709 D3：交互消费即尾随排水——
                                        // 槽位拖拽/缩放的 SetWindowPos 跟手
                                        // （pending 空时零开销幂等；vwin 拖拽
                                        // 路径 pending 恒空零新增成本）。
                                        sync_native_geometry(state);
                                        // PLAN-712 r3 T-18（方案 A）：八向缩放
                                        // 的每一拍标记被缩放 app 视图脏——与
                                        // WinRect 臂同理（烤定像素 + 缓存快门
                                        // = 缩放全程不重排）；拖拽（Drag）不改
                                        // 内容尺寸，不标。逐拍重建即活布局的
                                        // 标准成本（独立窗 OS resize 同价）。
                                        if let Some(host) = state.host.as_ref() {
                                            if let Some(
                                                crate::ui::session::WmInteraction::Resize {
                                                    wid,
                                                    ..
                                                },
                                            ) = host.wm.interaction
                                            {
                                                state.mark_vwin_resized_dirty(wid);
                                            }
                                        }
                                        return iced::Task::none();
                                    }
                                    // PLAN-063 T-03（G-04）：hover 下行——WM
                                    // 未消费（无拖拽/缩放交互进行）时，指针
                                    // 移动投递命中 broker 客户端（rqhost
                                    // daemon CursorMoved 臂同语义；此前桌面
                                    // 会话无 PointerMoved 生产，outproc 子面
                                    // hover 态缺席）。
                                    {
                                        let hit = state
                                            .host
                                            .as_ref()
                                            .and_then(|h| h.wm.hit_test(x, y));
                                        if let Some(wid) = hit {
                                            if state
                                                .broker_clients
                                                .values()
                                                .any(|c| c.owns_wid(wid))
                                            {
                                                state.broker_pointer_move(x, y);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "__modifiers_changed" => {
                            // PLAN-526 T32：Ctrl 松开且 switcher 可见 → 注入
                            // .Pick 提交选中（Focus(wid) → __desktop_cmd →
                            // 排水聚焦）——Windows/Linux Alt-Tab 语义的
                            // "松手即切换"半边；Esc 取消语义不变（overlay
                            // bind 自管）。修饰键事实源照旧回写。
                            if let Some(ref val) = m.input_value {
                                if let Ok(bits) = val.parse::<u32>() {
                                    let prev = *state.desktop.current_modifiers.borrow();
                                    let new_mods =
                                        iced::keyboard::Modifiers::from_bits_truncate(bits);
                                    *state.desktop.current_modifiers.borrow_mut() = new_mods;
                                    let ctrl_released = prev.control() && !new_mods.control();
                                    if ctrl_released && state.switcher_visible() {
                                        // PLAN-036 T-04（D6）：outproc 轨——
                                        /// Pick 事件快照（child dispatch →
                                        /// SwitcherMsg::Pick 提交选中 →
                                        /// SendCmd focus 上行 DesktopBus）+
                                        /// 面板自隐（switcher_open 复位）。
                                        if state.desktop.shell_pipe.is_some() {
                                            state.desktop.switcher_open = false;
                                            push_switcher_snapshot(
                                                state,
                                                &[crate::ui::shell_projection::ShellEvent::Pick],
                                            );
                                            return iced::Task::none();
                                        }
                                        if let Some(sw) = state.desktop.switcher_app {
                                            let pick = IcedMessage {
                                                widget: String::new(),
                                                event: "Pick".to_string(),
                                                input_value: None,
                                            };
                                            return update_inner(state, sw, pick, false)
                                                .map(move |m| DM::App(sw, m));
                                        }
                                    }
                                }
                            }
                        }
                        "__mouse_released" => {
                            // PLAN-526 T20：先偿还软聚焦挂起的置顶（此刻无
                            // 在途点击，重排无害），再判定拖拽/缩放交互收尾
                            //——两分支语义互不干扰（pending 过期 = no-op）。
                            state.wm_apply_pending_raise();
                            // PLAN-030 T-04：pointer release 接线——命中窗属
                            // broker 客户端（含壳伪窗）时投递 PointerReleased
                            //（press→release 对成 click）。
                            {
                                let cursor = state
                                    .host
                                    .as_ref()
                                    .map(|h| h.wm.last_cursor.get())
                                    .unwrap_or_default();
                                let hit = state
                                    .host
                                    .as_ref()
                                    .and_then(|h| h.wm.hit_test(cursor.x, cursor.y));
                                if let Some(wid) = hit {
                                    if state.broker_clients.values().any(|c| c.owns_wid(wid)) {
                                        state.broker_pointer_up(
                                            cursor.x,
                                            cursor.y,
                                            crate::ui::desktop_protocol::message::MouseButton::Left,
                                        );
                                    }
                                }
                            }
                            if state
                                .host
                                .as_mut()
                                .map(|h| h.wm.end_interaction())
                                .unwrap_or(false)
                            {
                                return iced::Task::none();
                            }
                            // PLAN-012 F2 走查：桌面图标拖拽松手落格——
                            // 宿主全局臂收尾（mouse_area 离开即失
                            // is_pressed，跨件 release 不可达）。光标 =
                            // last_cursor（全局 CursorMoved 持续回写，
                            // desktop 本地坐标）。
                            if let Some((dragged, _origin)) = state.desktop.icon_drag.take() {
                                // 2026-09-15：未超位移阈值 = 点击——原样落回
                                // （只清视觉态），不再误落光标格。
                                if state.desktop.icon_drag_moved {
                                    execute_desktop_icon_drop_at_cursor(state, &dragged);
                                } else {
                                    clear_desktop_drag_visual(state);
                                }
                                return iced::Task::none();
                            }
                        }
                        _ => {}
                    }
                }
                // Plan 504：fit 窗口内容测量回执（measure → shrink 路由；
                // 独立/desktop 双模式统一在此消费，见 apply_fit_measured）。
                if m.event == "__fit_measured" {
                    return apply_fit_measured(state, win, m.input_value.as_deref());
                }
                // Plan 504：fit 主触发——__window_resized（含开窗后的初始
                // Resized 事件）到达即该窗界面已建，测量锚点必在树中。
                // 不消费事件：测量任务与常管 resize 处理并联下发。
                // Plan 512：动态重测（fit_dirty）同闸并入。
                let mut fit_task = if m.event == "__window_resized"
                    && (state.has_fit_pending() || state.has_fit_remeasure_pending())
                    && !state.desktop.fit_measure_in_flight.replace(true)
                {
                    state.desktop.fit_measure_retries.set(0);
                    Some(fit_measure_task(win))
                } else {
                    None
                };
                // Plan 512：fit 用户锁定（独立模式）——程序化 resize 的 OS
                // 回波按 fit_last_applied 豁免（±2px 容 DPI 取整）；其余
                // 到达尺寸视为用户手动 resize，一次性锁定不再跟随。
                if !state.is_desktop() && m.event == "__window_resized" {
                    if let Some(entry) = state.windows.get(&win) {
                        if entry.fit_enabled.get() && !entry.fit_pending.get() {
                            if let Some((ws, hs)) =
                                m.input_value.as_deref().and_then(|v| v.split_once('x'))
                            {
                                let w: f32 = ws.parse().unwrap_or(0.0);
                                let h: f32 = hs.parse().unwrap_or(0.0);
                                let mut last = entry.fit_last_applied.borrow_mut();
                                if let Some(applied) = *last {
                                    *last = None;
                                    if (w - applied.width).abs() > 2.0
                                        || (h - applied.height).abs() > 2.0
                                    {
                                        entry.fit_user_locked.set(true);
                                    }
                                } else {
                                    entry.fit_user_locked.set(true);
                                }
                            }
                        }
                    }
                }
                // Plan 462 desktop：宿主窗 resize → 全体虚拟窗口 window_size
                // 同步 + 全场重排（逻辑在 [`sync_host_resized`]，测试共用）。
                if state.is_desktop() && m.event == "__window_resized" {
                    if let Some(ref val) = m.input_value {
                        if let Some((ws, hs)) = val.split_once('x') {
                            let w: f32 = ws.parse().unwrap_or(1280.0);
                            let h: f32 = hs.parse().unwrap_or(800.0);
                            sync_host_resized(state, iced::Size::new(w, h));
                            auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                                "vm.window_inner_height",
                                format!("{}", h),
                            );
                            return fit_task.take().unwrap_or_else(iced::Task::none);
                        }
                    }
                }
                // 关窗请求：产 window::close（Closed 事件随后走注册表清理 +
                // 空则退出）。PLAN-626 T-03: 声明 CloseRequest 生命周期
                // handler 的应用可拦截（fire 语义同 Init 直调先例）——有
                // 未保存状态的编辑器先弹确认层，由 handler 决定后续；未
                // 声明行为不变（向后兼容）。
                if m.event == "__window_close_request" {
                    let close_declared = state.app_of_window(&win).and_then(|app_id| {
                        state
                            .apps
                            .get(&app_id)
                            .map(|a| (app_id, a.component.has_lifecycle_handler("CloseRequest")))
                    });
                    if let Some((app_id, true)) = close_declared {
                        if let Some(app) = state.apps.get_mut(&app_id) {
                            if let Err(e) = app.component.fire_close_request() {
                                eprintln!(
                                    "[VM-HANDLER] {}.CloseRequest failed: {e}",
                                    app.component.widget_name()
                                );
                            }
                            *app.state.view_dirty.borrow_mut() = true;
                        }
                        return iced::Task::none();
                    }
                    return iced::window::close::<crate::ui::session::DesktopMessage>(win);
                }
                match state.app_of_window(&win) {
                    Some(app_id) => {
                        let t = dispatch_app(state, app_id, m, false);
                        match fit_task.take() {
                            Some(f) => iced::Task::batch([t, f]),
                            None => t,
                        }
                    }
                    None => {
                        eprintln!(
                        "[session] window event for unregistered window {win:?} dropped (plan-459)"
                    );
                        fit_task.take().unwrap_or_else(iced::Task::none)
                    }
                }
            }
        }
    };

    // 视图出口打标：Element<IcedMessage> → Element<DesktopMessage>（仅顶层一次，
    // widget 回调仍全程产 IcedMessage，view 管线零改动）。闭包对借用生命周期的
    // HKT 局限（同 theme_fn 教训）要求以 fn 条目承载。
    // Plan 459：daemon 的 view 按窗口调用 —— 先反查归属 App，未登记窗口
    // （Opened 前的瞬态/外部窗口）回退占位元素；出口按窗口打标 DM::App。
    fn view_desktop_fn(
        state: &crate::ui::session::DesktopSession,
        window: iced::window::Id,
    ) -> iced::Element<'_, crate::ui::session::DesktopMessage> {
        use crate::ui::session::DesktopMessage as DM;
        // Plan 462：desktop 模式 —— 宿主窗渲染 WM z-stack（背景 + N 个
        // VirtualWindow 层），每层复用同一 per-App 视图管线（dynamic_view +
        // panic 边界 + DM::App 打标），仅外层组装按配置分叉（I3）。
        if state.is_desktop() {
            let Some(host) = state.host.as_ref() else {
                return iced::widget::text("[AutoUI 会话] 桌面宿主未初始化")
                    .size(14)
                    .into();
            };
            if host.window != window {
                return iced::widget::container(
                    iced::widget::text("[AutoUI 会话] 窗口未登记").size(14),
                )
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center(iced::Length::Fill)
                .into();
            }
            let mut layers: Vec<iced::Element<'_, DM>> = Vec::new();
            // Plan 496 M5：壁纸层（图片分支——solid 分支由 desktop.at 根 bg
            // 承接，不推本层）。Stack 最底之底：桌面图标面之下、虚拟窗之下。
            if !state.desktop.desktop_wallpaper.starts_with('#') {
                layers.push(desktop_wallpaper_element(&state.desktop.desktop_wallpaper));
                // Plan 503 M3：壁纸罩层（可读性 scrim，紧贴壁纸之上）。
                layers.push(desktop_wallpaper_scrim());
            }
            if state.desktop.shell_pipe.is_some() {
                // PLAN-030 T-04：background 表面帧（壳 outproc 双表面之一——
                // 本槽位 = 壁纸之上/dashboard 之下，替代 in-proc desktop 面）。
                let bg_wid = state.desktop.shell_pseudo_wids.first().copied();
                if let Some(el) = bg_wid.and_then(|w| shell_surface_element(state, w)) {
                    layers.push(el);
                }
            } else if state.desktop.desktop_app.is_some() {
                let surface_app = state.desktop.desktop_app.expect("surface checked");
                let build = || state.split_ref_desktop().map(|v| dynamic_view(v, false));
                let surface_client: iced::Element<'_, IcedMessage> = match std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(build),
                ) {
                    Ok(Some(el)) => el,
                    Ok(None) => iced::widget::text("[AutoUI 会话] 桌面本体缺失")
                        .size(14)
                        .into(),
                    Err(payload) => {
                        eprintln!(
                            "[session] desktop surface view panicked (plan-453 T6 boundary): {payload:?}"
                        );
                        desktop_crash_element()
                    }
                };
                layers.push(surface_client.map(move |m| DM::App(surface_app, m)));
            }
            // PLAN-024：dashboard 常驻小组件层（用户裁定 2026-09-17）——
            // z 高于桌面图标层、低于全部 app 窗（R12：图标层全屏
            // BlankPress mouse-area 会吞面板 click，必须在图标层之上才可
            // 交互；视觉右上与图标网格不重叠，app 窗照常遮挡——R3 保持）；
            // 常驻非召唤：× = 隐藏 / dock ▦ 切换。
            // chrome wrapper 与 face 卡共用 dashboard_layout 算式定位
            // 定尺寸（px spacer 链，像素一致——.at 侧尺寸类不参与的根修）。
            if state.dashboard_visible() {
                let faces_view = dashboard_faces_for_view(state);
                let viewport = state.host_viewport();
                // PLAN-035 T-04 v2：dashboard_layout 直出视口绝对矩形
                // （外框 = 右上 8×2 网格块 720×176 @12px；face 卡 88/80
                // 网格算术 2×2/3×2）——R10 十列外框 + 三等分内格 sx/sy
                // 缩放路径退役。
                let (panel, cells) = dashboard_layout(viewport, &faces_view);
                // R4：卡面 glass 底（stella dash-card 语言——主题感知半透
                // 明填充，dark=轻提亮/light=白玻璃）。
                // PLAN-041 追随三轮（用户裁定）：dark = 任务栏同色系
                // #272e48 @95%（卡自身实底；外层面板框已退役），light =
                // 白 45% 浅卡沿旧。
                let card_fill = if crate::ui::style_iced_adapter::dark_mode() {
                    iced::Color::from_rgba(0.153, 0.180, 0.282, 0.90)
                } else {
                    iced::Color::from_rgba(1.0, 1.0, 1.0, 0.45)
                };
                // PLAN-036 T-05/T-06（B2）：chrome 二轨——outproc = child
                /// 表面帧贴层（伪窗[4]，表面 = dashboard_layout 外框
                /// 720×176 spacer 链右上贴放）；in-proc = 面板 App 拆借
                /// 视图 wrapper（原路径零变化）。face 卡两轨共用（宿主
                /// 叠层——D1 修订：卡内容真渲保留宿主，面板 chrome/清单
                /// /grid = child）。
                let msg_target = state
                    .desktop
                    .dashboard_app
                    .unwrap_or(crate::ui::session::AppId(0));
                let chrome = if state.desktop.shell_pipe.is_some() {
                    let d_wid = state.desktop.shell_pseudo_wids.get(4).copied();
                    match d_wid.and_then(|w| shell_surface_element(state, w)) {
                        Some(el) => iced::widget::container(iced::widget::row![
                            iced::widget::Space::new()
                                .width(iced::Length::Fixed(panel.x))
                                .height(iced::Length::Shrink),
                            iced::widget::column![
                                iced::widget::Space::new()
                                    .width(iced::Length::Shrink)
                                    .height(iced::Length::Fixed(panel.y)),
                                iced::widget::container(el)
                                    .width(iced::Length::Fixed(panel.width))
                                    .height(iced::Length::Fixed(panel.height + DASH_TAB_STRIP_H,))
                                    .clip(true),
                            ],
                        ])
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .into(),
                        None => iced::widget::text("").size(1).into(),
                    }
                } else {
                    let dash_app = state.desktop.dashboard_app.expect("dashboard checked");
                    // PLAN-041 T-15：面板底色宿主侧实铺（面板 .at 的 chrome
                    // bg 类在 VM 视图链不落漆，根因另卡 P041-D3）——定位
                    // 复用 face 卡同款 spare_position（已被证明正确落位，
                    // 弃自研 spacer 链 Fill 容器被中央化错位面）：
                    // dark = 任务栏同色系 #272e48 @95%（用户二轮裁定），
                    // light = 白 10% 浅玻璃。
                    let build = || state.split_ref_dashboard().map(|v| dynamic_view(v, false));
                    let dash_client: iced::Element<'_, IcedMessage> = match std::panic::catch_unwind(
                        std::panic::AssertUnwindSafe(build),
                    ) {
                        Ok(Some(el)) => el,
                        Ok(None) => iced::widget::text("[AutoUI 会话] dashboard 缺失")
                            .size(14)
                            .into(),
                        Err(payload) => {
                            eprintln!(
                                    "[session] dashboard view panicked (plan-453 T6 boundary): {payload:?}"
                                );
                            desktop_crash_element()
                        }
                    };
                    // PLAN-041 追随三轮（用户裁定）：外层大框退役——
                    // 底色改铺 face 卡自身（card_fill 深色支），面板层
                    // 只承担 .at chrome 定位（透明）。
                    let chrome_card =
                        iced::widget::container(dash_client.map(move |m| DM::App(dash_app, m)))
                            .width(iced::Length::Fixed(panel.width))
                            .height(iced::Length::Fixed(panel.height + DASH_TAB_STRIP_H));
                    spare_position(
                        chrome_card.into(),
                        iced::Rectangle {
                            x: panel.x,
                            y: panel.y,
                            width: panel.width,
                            height: panel.height + DASH_TAB_STRIP_H,
                        },
                    )
                };
                layers.push(chrome);
                // face 卡叠合（viewport 绝对格位；占位卡 = 宿主合成面）。
                for (f, rect) in faces_view.iter().zip(cells.iter()) {
                    if f.status != "running" && f.status != "hatched" {
                        let launch_msg = IcedMessage {
                            widget: String::new(),
                            event: format!("__dashboard_launch:{}", f.id),
                            input_value: None,
                        };
                        let hint: iced::Element<'_, IcedMessage> = iced::widget::mouse_area(
                            iced::widget::column![
                                iced::widget::text("▸").size(22),
                                iced::widget::text(f.title.clone()).size(12),
                                iced::widget::text("未运行 — 点击启动").size(11),
                            ]
                            .align_x(iced::alignment::Horizontal::Center)
                            .spacing(4),
                        )
                        .on_press(launch_msg)
                        .into();
                        let placeholder_client = hint.map(move |m| DM::App(msg_target, m));
                        let card = iced::widget::container(placeholder_client)
                            .width(iced::Length::Fixed(rect.width))
                            .height(iced::Length::Fixed(rect.height))
                            .align_x(iced::alignment::Horizontal::Center)
                            .align_y(iced::alignment::Vertical::Center)
                            .style(move |_t| iced::widget::container::Style {
                                background: Some(card_fill.into()),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.25),
                                    width: 1.0,
                                    radius: 12.0.into(),
                                },
                                ..Default::default()
                            });
                        layers.push(spare_position(card.into(), *rect));
                        continue;
                    }
                    let Some(app_id) = (match f.status {
                        "running" => dashboard_running_app(state, &f.id),
                        "hatched" => state.hatched_mini_of(&f.id),
                        _ => None,
                    }) else {
                        continue;
                    };
                    let build_face = || {
                        state
                            .split_ref_face(app_id, "mini")
                            .map(|v| dynamic_view(v, false))
                    };
                    let face_el: iced::Element<'_, IcedMessage> =
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build_face)) {
                            Ok(Some(el)) => el,
                            Ok(None) => iced::widget::text("").size(1).into(),
                            Err(payload) => {
                                eprintln!(
                                "[session] face view panicked (plan-453 T6 boundary): {payload:?}"
                            );
                                desktop_crash_element()
                            }
                        };
                    let face_client = face_el.map(move |m| DM::App(app_id, m));
                    // R20：卡体点击 → 打开对应 app（内层按钮/交互优先命中，
                    // 空白区落到本 mouse_area——N6d 内外层同款机制）。
                    // R21：双击打开（桌面图标同款交互；单击留给卡片内部
                    // 交互/无动作）。
                    let face_wrapped =
                        iced::widget::mouse_area(face_client).on_double_click(DM::App(
                            app_id,
                            IcedMessage {
                                widget: String::new(),
                                event: format!("__dashboard_open:{}", f.id),
                                input_value: None,
                            },
                        ));
                    let card = iced::widget::container(face_wrapped)
                        .width(iced::Length::Fixed(rect.width))
                        .height(iced::Length::Fixed(rect.height))
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center)
                        .style(move |_t| iced::widget::container::Style {
                            background: Some(card_fill.into()),
                            border: iced::Border {
                                color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.35),
                                width: 1.0,
                                radius: 12.0.into(),
                            },
                            ..Default::default()
                        });
                    layers.push(spare_position(card.into(), *rect));
                }
            }
            // Plan 496 M5：桌面本体层（463 预留桌面层 z 槽消费）——Stack
            // 最底：先于虚拟窗推层 = 桌面图标在壁纸层之上、App 虚拟窗口
            // 之下（G3 层级：窗口拖过时图标自然被覆盖）。shell 层同型
            // catch_unwind 视图边界（453 T6）。
            // PLAN-709：统一 z 序循环——伪 Wid 段 = 槽位 chrome 插序臂
            // （与虚拟窗同一绘制序；假洞视觉边界注：虚拟窗层恒在原生窗
            // 之下，插序改变的是 chrome 框序，真洞模式语义自然正确）。
            for &wid in &host.wm.z_order {
                if let Some(slot_id) = crate::ui::session::WmState::native_slot_id_of_pseudo(wid) {
                    let Some(slot) = host.wm.native_slots.get(&slot_id) else {
                        continue;
                    };
                    let Some(local) = host.wm.native_slot_local_rects.get(&slot_id) else {
                        continue;
                    };
                    // PLAN-709：非当前分区槽位 chrome 缺席（AC-05；OS 级
                    // 隐现随 sync_native_workspace_visibility 排水拍）。
                    if slot.workspace != host.wm.current_workspace {
                        continue;
                    }
                    let focused = host.wm.focused == Some(wid);
                    let interacting = host.wm.interaction.is_some_and(|i| match i {
                        crate::ui::session::WmInteraction::NativeDrag { slot_id: s, .. }
                        | crate::ui::session::WmInteraction::NativeResize { slot_id: s, .. } => {
                            s == slot_id
                        }
                        _ => false,
                    });
                    layers.push(crate::ui::iced::virtual_window::native_slot_element(
                        slot_id,
                        &slot.title_cache,
                        *local,
                        focused,
                        interacting,
                    ));
                    continue;
                }
                let Some(vwin) = host.wm.wins.get(&wid) else {
                    continue;
                };
                // Plan 472 T2：只绘制当前分区（换分区=窗口随分区隐现）。
                if vwin.workspace != host.wm.current_workspace {
                    // PLAN-041 T-04 诊断（AUTO_DEBUG_KEYS 门控）：实证跨分区
                    // 残影时核对本臂是否随帧泵执行（打印 = 过滤在、层已跳）。
                    if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
                        eprintln!(
                            "[472-WS] view skip wid={} ws={} cur={}",
                            wid.0, vwin.workspace, host.wm.current_workspace
                        );
                    }
                    continue;
                }
                // PLAN-526 T2：最小化窗不推层（隐藏；任务栏 icon 保留，
                // WinFocus → focus 即还原）。
                // PLAN-012 W1：常驻隐藏窗不推层（close→hide；__wm_wins 已
                // 投影排除——任务栏无条目）。
                // PLAN-030：壳伪窗不进 vwin 循环——background 走桌面面槽、
                // chrome 走任务栏槽（层位 §5.1 D1；WM 命中承接保留）。
                if state.desktop.shell_pseudo_wids.contains(&wid) {
                    continue;
                }
                if vwin.minimized.get() || vwin.hidden.get() {
                    continue;
                }
                let app_id = vwin.app;
                // Plan 500 步骤 5：broker 表面窗（进程外 App）——客户区内容
                // 取 child 上报帧的两态合成（queue=DrawList canvas 降级 /
                // independent=RGBA Image），不再本地重渲染。非 broker 窗走
                // 既有 dynamic_view 路径零变化。
                if let Some(broker_client) =
                    crate::ui::iced::broker_surface::broker_client_content(state, wid)
                {
                    let focused = host.wm.focused == Some(wid);
                    let title_menu_pos = host
                        .wm
                        .title_menu
                        .filter(|t| t.wid == wid)
                        .map(|t| (t.x, t.y));
                    layers.push(crate::ui::iced::virtual_window::virtual_window_element(
                        vwin,
                        focused,
                        title_menu_pos,
                        crate::ui::iced::virtual_window::transparency_alpha_for(
                            &state.desktop.config.transparency,
                        ),
                        broker_client,
                    ));
                    continue;
                }
                let probe_hit = panic_probe_enabled()
                    && PANIC_PROBE_CRASHED_APP.load(std::sync::atomic::Ordering::SeqCst)
                        == app_id.0;
                let is_primary = state.primary_app() == Some(app_id);
                let build = || {
                    if probe_hit {
                        panic!("plan-459 T5 view panic probe for App {app_id:?}");
                    }
                    // MCP 快照同步仅 primary（T8 冻结）；desktop 下其余 App
                    // 走同一视图管线但 sync_mcp=false。
                    let Some(view) = state.split_ref_at(app_id, host.window) else {
                        return None;
                    };
                    Some(dynamic_view_vwin(view, is_primary))
                };
                let client: iced::Element<'_, IcedMessage> =
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
                        Ok(Some(el)) => el,
                        Ok(None) => iced::widget::text("[AutoUI 会话] 缺少 App 会话")
                            .size(14)
                            .into(),
                        Err(payload) => {
                            eprintln!(
                            "[session] app view panicked (plan-453 T6 view boundary): {payload:?}"
                        );
                            desktop_crash_element()
                        }
                    };
                let client = client.map(move |m| DM::App(app_id, m));
                let focused = host.wm.focused == Some(wid);
                let title_menu_pos = host
                    .wm
                    .title_menu
                    .filter(|t| t.wid == wid)
                    .map(|t| (t.x, t.y));
                layers.push(crate::ui::iced::virtual_window::virtual_window_element(
                    vwin,
                    focused,
                    title_menu_pos,
                    crate::ui::iced::virtual_window::transparency_alpha_for(
                        &state.desktop.config.transparency,
                    ),
                    client,
                ));
            }
            // Plan 473 T6 槽位框 chrome 层——PLAN-709 起并入统一 z 序循环
            // （上方 vwin 循环内伪 Wid 臂），固定垫全部虚拟窗之后的装配序
            // 退役（AC-04 chrome 绘制序置顶语义）。
            // Plan 486：拖入手势落点高亮层（DragWatch::Over 时；槽位 chrome
            // 之上、shell 之下——半透明不遮挡既有 chrome 标题条）。
            if let Some(drag_rect) = state.native_drag_over {
                layers.push(crate::ui::iced::virtual_window::native_drag_over_element(
                    drag_rect,
                ));
            }
            // Plan 463 T5：shell 层（任务栏；z-stack 之上常驻）。底部锚定
            // 由 shell.at 根 col（h-full + 纵向 spacer）落实——dynamic_view
            // 根是 Fill×Fill toast-Stack，装配层 align 无从发力（实测）。
            // Plan 464 overlay 槽：launcher/通知层追加在本层之后（Stack 顶层，
            // 消费 DM::Desktop(SummonLauncher)）；v1 无消费者，不推空层。
            if state.desktop.shell_pipe.is_some() {
                // PLAN-030 T-04：chrome 表面帧（壳 outproc 双表面之二——
                // 本槽位 = 虚拟窗之上/launcher overlay 之下，替代 in-proc
                // shell 任务栏层；表面 = 任务栏带矩形，坐标带局部）。
                let band = state
                    .desktop
                    .shell_geometry
                    .map(|g| g.band_h)
                    .unwrap_or(48.0);
                let chrome_wid = state.desktop.shell_pseudo_wids.get(1).copied();
                if let Some(el) = chrome_wid.and_then(|w| shell_surface_element(state, w)) {
                    let band_el = iced::widget::container(el)
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fixed(band))
                        .clip(true);
                    layers.push(
                        iced::widget::container(band_el)
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill)
                            .align_y(iced::alignment::Vertical::Bottom)
                            .into(),
                    );
                }
            }
            if state.desktop.shell_app.is_some() {
                let shell_app = state.desktop.shell_app.expect("shell checked");
                // PLAN-012 O1：primary = 特权 shell（boot 分配序，见 boot 注
                // 记）——MCP 快照同步（autoui_state/autoui_vtree/bounds）随
                // shell 层开启；旧序 primary=首个直挂窗，shell 面不可观测。
                let shell_sync = state.primary_app() == Some(shell_app);
                let build = || state.split_ref_shell().map(|v| dynamic_view(v, shell_sync));
                let shell_client: iced::Element<'_, IcedMessage> =
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
                        Ok(Some(el)) => el,
                        Ok(None) => iced::widget::text("[AutoUI 会话] shell 缺失")
                            .size(14)
                            .into(),
                        Err(payload) => {
                            eprintln!(
                                "[session] shell view panicked (plan-453 T6 boundary): {payload:?}"
                            );
                            desktop_crash_element()
                        }
                    };
                layers.push(shell_client.map(move |m| DM::App(shell_app, m)));
            }
            // Plan 464 T4：launcher overlay 层（Stack 顶层；隐藏态渲染透明
            // 空层不挡桌面点击——app.at else 分支）。垫片视图同 shell。
            // PLAN-036 T-07（B3）：outproc 轨 = launcher exe 表面帧贴层
            ///（伪窗全屏——launcher 可见时置顶；child else 分支同语义
            /// 渲透明空层）。
            if state.desktop.launcher_pipe.is_some() && state.launcher_visible() {
                if let Some(lwid) = state.desktop.launcher_wid {
                    if let Some(el) =
                        shell_surface_element_pipe(state, state.desktop.launcher_pipe.clone(), lwid)
                    {
                        let full = iced::widget::container(el)
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill);
                        layers.push(full.into());
                    }
                }
            } else if state.desktop.launcher_app.is_some() {
                let launcher_app = state.desktop.launcher_app.expect("launcher checked");
                let build = || state.split_ref_launcher().map(|v| dynamic_view(v, false));
                let launcher_client: iced::Element<'_, IcedMessage> =
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
                        Ok(Some(el)) => el,
                        Ok(None) => iced::widget::text("[AutoUI 会话] launcher 缺失")
                            .size(14)
                            .into(),
                        Err(payload) => {
                            eprintln!(
                            "[session] launcher view panicked (plan-453 T6 boundary): {payload:?}"
                        );
                            desktop_crash_element()
                        }
                    };
                layers.push(launcher_client.map(move |m| DM::App(launcher_app, m)));
            }
            // Plan 478 T4：switcher overlay 层（launcher 层邻位顶层；仅
            // visible 时推层——T1 施工图 §1.5，隐匿由 visible 翻转后本周期
            // 视图重建生效，召唤/隐匿两臂显式置 view_dirty 保险）。
            if state.switcher_visible() {
                // PLAN-036 T-04（B1）：outproc 轨——switcher 表面帧全屏
                /// 贴层（伪窗 [2]；面板内容/scrim 由 child DrawList 自绘）。
                if state.desktop.shell_pipe.is_some() {
                    let overlay_wid = state.desktop.shell_pseudo_wids.get(2).copied();
                    if let Some(el) = overlay_wid.and_then(|w| shell_surface_element(state, w)) {
                        let full = iced::widget::container(el)
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill);
                        layers.push(full.into());
                    }
                    // in-proc 分支不落（overlay App 不在场）。
                } else {
                    let switcher_app = state.desktop.switcher_app.expect("switcher checked");
                    let build = || state.split_ref_switcher().map(|v| dynamic_view(v, false));
                    let switcher_client: iced::Element<'_, IcedMessage> =
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
                            Ok(Some(el)) => el,
                            Ok(None) => iced::widget::text("[AutoUI 会话] switcher 缺失")
                                .size(14)
                                .into(),
                            Err(payload) => {
                                eprintln!("[session] switcher view panicked (plan-453 T6 boundary): {payload:?}");
                                desktop_crash_element()
                            }
                        };
                    layers.push(switcher_client.map(move |m| DM::App(switcher_app, m)));
                }
            }
            // Plan 479 T3：通知中心 overlay 层（switcher 层邻位顶层；仅
            // visible 时推层——第三枚 overlay 槽，switcher 同款语义）。
            // PLAN-012 O3 终解（复验三连反馈）：锚定与 scrim 回归 .at 内
            // N6d 模式（scrim 外点关闭 + 卡片守卫，用户走查已验证）+
            // justify-start 列顶对齐（真实链验证机制——mt-auto 填充条/
            // col-reverse 在真实 Stack 子层不可依赖，复验三连贴顶 0 实
            // 证；justify-end 贴 dock 底已实测有效 = 同机制可信）。紧凑
            // max-h 滚动清偿"满高贴顶"。
            if state.notification_visible() {
                // PLAN-036 T-04（B1）：outproc 轨——通知表面帧全屏贴层
                ///（伪窗 [3]；对齐/scrim 由 child 自绘）。
                if state.desktop.shell_pipe.is_some() {
                    let overlay_wid = state.desktop.shell_pseudo_wids.get(3).copied();
                    if let Some(el) = overlay_wid.and_then(|w| shell_surface_element(state, w)) {
                        let full = iced::widget::container(el)
                            .width(iced::Length::Fill)
                            .height(iced::Length::Fill);
                        layers.push(full.into());
                    }
                } else {
                    let panel_app = state.desktop.notification_app.expect("panel checked");
                    let build = || {
                        state
                            .split_ref_notification()
                            .map(|v| dynamic_view(v, false))
                    };
                    let panel_client: iced::Element<'_, IcedMessage> =
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
                            Ok(Some(el)) => el,
                            Ok(None) => iced::widget::text("[AutoUI 会话] 通知面板缺失")
                                .size(14)
                                .into(),
                            Err(payload) => {
                                eprintln!("[session] notification view panicked (plan-453 T6 boundary): {payload:?}");
                                desktop_crash_element()
                            }
                        };
                    layers.push(panel_client.map(move |m| DM::App(panel_app, m)));
                }
            }
            return crate::ui::iced::virtual_window::desktop_root(layers);
        }
        let Some(app_id) = state.app_of_window(&window) else {
            return iced::widget::container(
                iced::widget::text("[AutoUI 会话] 窗口未登记").size(14),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center(iced::Length::Fill)
            .into();
        };
        let Some(view) = state.split_ref_at(app_id, window) else {
            return iced::widget::text("[AutoUI 会话] 缺少 App 会话")
                .size(14)
                .into();
        };
        // Plan 459 T5 view 探针：被注入 panic 的 App 其后每帧在 view 边界
        // panic → 持续显示崩溃页（另一窗口不受影响）。
        let probe_hit = panic_probe_enabled()
            && PANIC_PROBE_CRASHED_APP.load(std::sync::atomic::Ordering::SeqCst) == app_id.0;
        // T6 视图侧边界：view panic 同样不落进程，降级为全屏提示元素。
        // MCP 快照同步只在 primary App 的视图执行（T8 单 App 语义冻结）。
        let is_primary = state.primary_app() == Some(app_id);
        let build = || {
            if probe_hit {
                panic!("plan-459 T5 view panic probe for App {app_id:?}");
            }
            dynamic_view(view, is_primary)
        };
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build)) {
            Ok(el) => el.map(move |m| crate::ui::session::DesktopMessage::App(app_id, m)),
            Err(payload) => {
                eprintln!("[session] app view panicked (plan-453 T6 view boundary): {payload:?}");
                iced::widget::container(
                    iced::widget::text("[AutoUI 会话] 视图构建异常（plan-453 边界兜底）").size(14),
                )
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center(iced::Length::Fill)
                .style(|_t| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.10, 0.05, 0.05).into()),
                    ..Default::default()
                })
                .into()
            }
        }
    }

    // Plan 462：desktop 分支的视图崩溃页（plan-453 T6 边界兜底的
    // VirtualWindow 版；消息类型无关，独立模式的内联版保持原样不动）。
    fn desktop_crash_element() -> iced::Element<'static, IcedMessage> {
        iced::widget::container(
            iced::widget::text("[AutoUI 会话] 视图构建异常（plan-453 边界兜底）").size(14),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .center(iced::Length::Fill)
        .into()
    }

    let title_fn =
        move |state: &crate::ui::session::DesktopSession, window: iced::window::Id| -> String {
            // 459：per-window 标题 = 归属 App 的 widget 名；未登记窗口退回壳名。
            let name = state
                .app_of_window(&window)
                .and_then(|id| state.apps.get(&id))
                .map(|a| a.component.widget_name().to_string())
                .unwrap_or_else(|| widget_name.clone());
            window_title(format!("Auto - {}", name))
        };

    // Plan 047:深色主题(对齐 ash-gui vue dark mode)。
    // Plan 448 对齐批:内置 Theme::Dark 的窗口底色是 #2B2D31,与 vue 产物
    // shadcn --background(hsl 222.2 47.4% 7% = #090E1A)不一致 —— 换成
    // shadcn 令牌基的自定义调色板,明暗随 iced_adapter 的 dark_mode 走
    //(默认暗色,与既有 DARK_MODE 初值一致)。
    let theme_fn =
        move |_state: &crate::ui::session::DesktopSession,
              _window: iced::window::Id|
              -> iced::Theme { shadcn_theme(crate::ui::style_iced_adapter::dark_mode()) };

    let session_result = iced::daemon(boot, update, view_desktop_fn)
        .title(title_fn)
        // Plan 411 P1-C: 内嵌 Inter 三字重 + 默认 family(中文字形回退系统)。
        .font(INTER_FONT_REGULAR)
        .font(INTER_FONT_MEDIUM)
        .font(INTER_FONT_SEMIBOLD)
        .default_font(INTER_FONT)
        // Plan 047:深色主题(对齐 ash-gui vue dark mode)。之前无 theme,窗口默认白色。
        .theme(theme_fn)
        .subscription(|state: &crate::ui::session::DesktopSession| {
            // Plan 459：订阅按 App 扇出（453 T5 原案落地）——
            //   App 级（hot reload / widget tick / timer tick / keyboard）
            //   按 AppId 打标；桌面级服务（toast 到期 / MCP / shell /
            //   heartbeat）只订一份、归 primary_app（T8 单 App 语义）；
            //   listen_with 事件带窗口上下文走 DM::Window（update 现场解析）。
            use crate::ui::session::DesktopMessage as DM;
            let mut subs: Vec<iced::Subscription<DM>> = Vec::new();
            // PLAN-815 T-05（F03）：宿主退出请求轮询——无条件订阅（CLI 形态
            // 无人递增，空拍零消息；不依赖 primary app 存在）。`since` =
            // run_session 启动快照（见 F03 R2 根修注记）。
            subs.push(crate::ui::session::host_exit_poll_subscription(
                current_host_exit_since(),
            ));
            #[cfg(windows)]
            if !state.is_desktop() {
                for window in state.windows.keys() {
                    subs.push(super::native_accessibility::subscription(*window));
                }
            }
            for (app_id, app) in state.apps.iter() {
                let app_id = *app_id;
                // PLAN-650 E-3：hot_reload 降频/门控——AUTOUI_HOT_RELOAD=0 全关；
                // =1 强制 500ms；缺省 debug 500ms / 非 debug 2000ms（静止泵 ÷4）。
                if app.component.source_path().is_some() {
                    if let Some(interval_ms) = hot_reload_interval_ms(app.state.devtools.debug_mode)
                    {
                        subs.push(app_tick(app_id, HOT_RELOAD_EVENT, interval_ms));
                    }
                }
                // PLAN-654 阶段 B C2: 框架墙钟——仅当视图/computed 引用
                // `__clock_` 时订 1Hz `__clock_tick`（静止无引用零额外泵）。
                if app.component.wants_framework_clock() {
                    subs.push(app_tick(
                        app_id,
                        crate::ui::dynamic::DynamicComponent::CLOCK_TICK_EVENT,
                        1000,
                    ));
                }
                // PLAN-652: Tick/timer 统一走 timesources + mounted/when 过滤。
                // 根 `tick_interval()` 不再单独订阅（避免与 root Tick 源双订）。
                // PLAN-654 T-A04: path 级订阅——identity 含 InstancePath；
                // 同 widget 不同 path = 不同 iced Subscription（hash 区分）。
                // 派发时 IcedMessage.widget = path 串；update 侧解析类型名。
                // AUTOUI_TIMESOURCE_DEBUG=1 → 打印 path 级订阅表（AC-A6）。
                let ts_debug = std::env::var("AUTOUI_TIMESOURCE_DEBUG").is_ok();
                for src in app.component.subscribable_instance_timesources() {
                    if ts_debug {
                        eprintln!(
                            "[TS_PATH] app={:?} path={} widget={} event={} kind={:?} ms={}",
                            app_id,
                            src.path.as_str(),
                            src.widget,
                            src.event,
                            src.kind,
                            src.every_ms
                        );
                    }
                    match src.kind {
                        crate::ui::dynamic::TimeSourceKind::Tick => {
                            // R5：孵化 mini 会话 Tick 门控仍适用于 Tick 源。
                            if dashboard_hatched_tick_allowed(state, app_id) {
                                subs.push(widget_event_tick(
                                    app_id,
                                    src.path.as_str(),
                                    &src.event,
                                    src.every_ms,
                                ));
                            }
                        }
                        crate::ui::dynamic::TimeSourceKind::Timer => {
                            subs.push(widget_event_tick(
                                app_id,
                                src.path.as_str(),
                                &src.event,
                                src.every_ms,
                            ));
                        }
                    }
                }
                // Plan 442 A5: one-shot timer tick — only while set_timeout timers
                // are pending; due callbacks fire in update's __timer_tick arm.
                if app.component.has_pending_timers() {
                    subs.push(app_tick(app_id, "__timer_tick", 16));
                }
                // PLAN-702 T-03: parked handler 段恢复泵 — only while segments
                // are parked; ready resumes fire in update's
                // __parked_resume_tick arm. 16ms ≠ 17/19ms 既有泵（Recipe
                // 身份含事件名，去重安全）。
                // PLAN-711 T-04 (D-2): 订阅门收窄为**纯 I/O 凭据**——CPU
                // continuation 对 tick 恒不就绪，CPU-only 注册表不吊 tick
                //（帧通知泵接管其驱动，见下）。
                if app.component.has_parked_io_tasks() {
                    // PLAN-721 T-1：恢复泵订阅装配 trace（桌面轨断链定案轴）。
                    crate::ui::sched_diag::trace(
                        Some(app_id),
                        "sub_parked",
                        &format!("parked_total={}", app.component.bridge().parked_count()),
                    );
                    subs.push(app_tick(app_id, "__parked_resume_tick", 16));
                }
                // PLAN-095 T-02: 媒体就绪泵 —— 仅在存在未终态媒体资产或有
                // 待派发的 loaded/error 通知时吊 tick（__media_tick 不匹配
                // 任何 update 臂，仅驱动 notify sweep 心跳）。解码完成/失败
                // 都是终态：pending 归零 + 通知派发完 → 订阅自动消失（有界，
                // 无重绘死循环）。
                {
                    let media_awaiting = app
                        .state
                        .media_notify
                        .borrow()
                        .iter()
                        .any(crate::ui::session::MediaNotifyEntry::awaiting_notify);
                    if auto_media_core::image_pipeline::pending_media_count() > 0 || media_awaiting
                    {
                        subs.push(app_tick(app_id, "__media_tick", 50));
                    }
                }
                // PLAN-711 T-04 (D-1): 帧通知泵——listen_raw 收 RedrawRequested
                // （唯一不过滤 redraw 的公开通道），经 runtime 通道异步回环
                // 构成"消费帧消息在该帧 present 返回之后"的应用层序障（无
                // post-present 回执 API，不 fork iced）。**仅在有未完成
                // Init demand / CPU continuation 时激活**（零 demand → 无
                // 订阅，防 listen_raw 自我续帧；无任务不跑周期泵）。消费臂
                // 运行有界泵（8ms 轮次预算，非全量 drain）。
                if app.component.has_pending_init_work() || app.component.has_cpu_continuations() {
                    // PLAN-721 T-1：帧泵订阅装配 trace（原 inline 打印归一）。
                    crate::ui::sched_diag::trace(
                        Some(app_id),
                        "sub_frame",
                        &format!(
                            "init_work={} cpu={}",
                            app.component.has_pending_init_work(),
                            app.component.has_cpu_continuations(),
                        ),
                    );
                    subs.push(frame_pump_sub(app_id));
                }
                // PLAN-735 T-02 轴①：RedrawRequested 送达观测（恒 None
                // 零消息面）——AUTO_SCHED_DIAG=1 时全程装配。
                if crate::ui::sched_diag::enabled() {
                    subs.push(redraw_diag_sub(app_id));
                }
                // F12 DevTools + key bindings（per-App bindings + 本窗过滤）。
                if let Some(win) = state.window_of_app(app_id) {
                    let bindings = app.component.key_bindings().clone();
                    // Plan 462（R12 订阅半边）：desktop 模式键盘只投递焦点
                    // 虚拟窗口的 App（identity 含 focused，焦点翻转即重订阅）；
                    // 独立模式恒 true —— 每窗过滤照旧，行为不变。F12 DevTools
                    // 面板在 desktop 模式禁用（面板固定 widget id 会同窗撞车；
                    // MCP/DevTools 单 App 语义见 453 T8 冻结，463+ 再解）。
                    // Plan 464 T4：launcher overlay 可见时独占键盘（P3 全键盘
                    // 流——输入/方向键进 palette，不漏进底层虚拟窗）。
                    // Plan 478 T4：switcher overlay 可见时同样独占（Tab/←→/
                    // Enter/Esc 进面板，不漏进底层虚拟窗）。
                    // Plan 479 T3：通知中心可见时同样独占（Esc 进面板）。
                    let focused = if state.is_desktop() {
                        state.wm_focused_app() == Some(app_id)
                            && !state.launcher_visible()
                            && !state.switcher_visible()
                            && !state.notification_visible()
                    } else {
                        true
                    };
                    subs.push(keyboard_subscription(
                        app_id,
                        win,
                        bindings,
                        focused,
                        state.is_desktop(),
                    ));
                }
            }
            // Plan 464 T4：launcher overlay 的键盘订阅（可见时独占；
            // 无 OS 窗——挂宿主窗，focused=true 门控在闭包内过滤）。
            if state.is_desktop() && state.launcher_visible() {
                if let (Some(la), Some(host)) = (state.desktop.launcher_app, state.host.as_ref()) {
                    if let Some(app) = state.apps.get(&la) {
                        let bindings = app.component.key_bindings().clone();
                        subs.push(keyboard_subscription_ext(
                            la,
                            host.window,
                            bindings,
                            true,
                            true,
                            true,
                        ));
                    }
                }
            }
            // Plan 478 T4：switcher overlay 的键盘订阅（launcher 块同型；
            // escape_forward 统一传——Esc 被 Captured 时宿主转发同一事件）。
            if state.is_desktop() && state.switcher_visible() {
                if let (Some(sw), Some(host)) = (state.desktop.switcher_app, state.host.as_ref()) {
                    if let Some(app) = state.apps.get(&sw) {
                        let bindings = app.component.key_bindings().clone();
                        subs.push(keyboard_subscription_ext(
                            sw,
                            host.window,
                            bindings,
                            true,
                            true,
                            true,
                        ));
                    }
                }
            }
            // Plan 479 T3：通知中心的键盘订阅（switcher 块同型第四块；
            // escape_forward 统一传——Esc Captured 转发，幂等由 handler
            // visible 门控保证）。
            if state.is_desktop() && state.notification_visible() {
                if let (Some(panel), Some(host)) =
                    (state.desktop.notification_app, state.host.as_ref())
                {
                    if let Some(app) = state.apps.get(&panel) {
                        let bindings = app.component.key_bindings().clone();
                        subs.push(keyboard_subscription_ext(
                            panel,
                            host.window,
                            bindings,
                            true,
                            true,
                            true,
                        ));
                    }
                }
            }
            if let Some(primary) = state.primary_app() {
                // Plan 463 T3：桌面级热键（Esc 调试退出；T6 追加布局/循环/
                // 召唤）。仅 desktop 模式订一份；独立模式行为不变。
                // Plan 490 T3：表取自会话（boot 期 shell.keys.* 覆盖已并入）。
                if state.is_desktop() {
                    if let Some(host) = &state.host {
                        subs.push(desktop_hotkey_subscription(host.window, state.hotkeys()));
                    }
                }
                // Plan 462：desktop 模式帧泵（空闲时也要有机会消费 view 侧
                // 投递的 MCP 截图请求；AUTOUI_MCP_DISABLE=1 一并关闭）。
                // Plan 512：standalone fit 窗动态重测复用同节拍——脏标只在
                // view 重建后打（dispatch_app），订阅在下轮 diff 即拉起；
                // 测量回执消费脏标后订阅自动回落（稳态零成本）。MCP 开关
                // 不管控此路（fit 是窗口语义而非验收通道）。
                let fit_remeasure_tick = !state.is_desktop() && state.has_fit_remeasure_pending();
                // PLAN-760: standalone（`auto run -r vm`）含 remote-frame 时
                // 也要 ServiceTick——注册表泵（连接收编/帧槽/重连）与帧推进
                // 都挂在 400ms 拍；url 退场租约清空 → 订阅自动回落。
                let remote_frame_tick =
                    !state.is_desktop() && crate::ui::iced::remote_frame_registry::has_live_urls();
                // PLAN-053 T-04：standalone 跟随腿——`AUTOOS_DESKTOP_CONFIG`
                // 武装（run_file_dynamic_ui_inner 置位）时拉 400ms 拍，独立
                // 进程经外写轮询差异臂（与宿主同入口）跟随宿主策略。
                let standalone_follow_tick =
                    !state.is_desktop() && crate::ui::theme_policy::standalone_follow_armed();
                if fit_remeasure_tick
                    || remote_frame_tick
                    || standalone_follow_tick
                    || (state.is_desktop()
                        && std::env::var("AUTOUI_MCP_DISABLE").map_or(true, |v| v != "1"))
                {
                    subs.push(crate::ui::session::desktop_service_tick(400));
                }
                // PLAN-063（OS T-03，P-03/P-04 接线）：broker 客户端在场 =
                // 活跃档 15ms 专泵（daemon `rq_subscription` 同律同值；
                // 零客户端不订——空闲零新增唤醒，400ms ServiceTick 兜底）。
                if state.is_desktop() && !state.broker_clients.is_empty() {
                    subs.push(crate::ui::session::broker_pump_tick(15));
                }
                // Plan 412 续(toast 修正 5):toast 到期 tick —— 桌面共享堆叠
                // 只订一份;到期的移除逻辑在 update 的 __toast_tick 分支。
                if !state.desktop.toasts.borrow().is_empty() {
                    subs.push(app_tick(primary, "__toast_tick", 250));
                }
                // Plan 473 T6：原生槽位 WinEventHook 事件泵（windows 实现；
                // 非 Windows 平台为空订阅）。
                if state.is_desktop() {
                    subs.push(crate::ui::session::native_dock_event_subscription());
                    // Plan 488：拖出完成事件泵（windows × native-dnd 双门控，
                    // 其余档空订阅——DoDragDrop STA 线程 → on_dnd_finished 面）。
                    subs.push(crate::ui::session::dnd_finished_subscription());
                }
                // MCP action channel — polls for injected actions from AI agent (Plan 278)
                subs.push(mcp_action_subscription(primary));
                // PLAN-095 T-02: 管线代次唤醒（桌面级常驻；空拍零消息）。
                subs.push(media_wake_subscription(primary));
                // Shell SSE → store bridge (ash-gui M1). Polls SHELL_EVENT_RX and
                // dispatches command_output/command_result to ShellStore handlers.
                subs.push(shell_event_subscription(primary));
                // PLAN-083 T-01: 异步 HTTP 消息桥泵（Http.get_msg 完成项 →
                // store 回填 handler）。常驻订阅（空队列空转，一次 poll 一次
                // VecDeque pop，开销可忽略）。
                subs.push(http_msg_subscription(primary));
                // Plan 500 步骤 4：像素桥协议轮询（independent 臂 child 进程
                // 才在册——宿主消息 → PixelsProtocol 事件面）。
                if state.pixels.is_some() {
                    subs.push(crate::ui::desktop_protocol::pixels::pixels_protocol_subscription());
                }
                // Plan 314: keep a styled VTree snapshot fresh on an otherwise-idle
                // app while an agent is connected. Only ticks when MCP is active.
                // 2026-08-22(活联门控):心跳改为"最近 30s 内有 MCP 请求"才开 ——
                // 周期性 view 重建在大 Code 块下会触发静默退出(实测 ~10s 内进程
                // 消失;关掉心跳后 30s+ 存活),普通运行(无 agent 连接)不应
                // 付出该代价。AUTOUI_MCP_DISABLE=1 可彻底关闭(诊断用)。
                let mcp_recent = state
                    .desktop
                    .mcp_shared
                    .as_ref()
                    .map(|s| s.lock().unwrap().mcp_active_recently(30))
                    .unwrap_or(false);
                if mcp_recent && std::env::var("AUTOUI_MCP_DISABLE").map_or(true, |v| v != "1") {
                    subs.push(mcp_heartbeat_subscription(primary));
                }
            }
            // Window resize + mouse move/release events for DevTools panel drag.
            // Plan 459：消息带窗口上下文走 DM::Window —— update 侧按发生窗口
            // 归位窗口级字段（`__window_resized` 写对窗口的尺寸；多窗互不串扰）。
            subs.push(iced::event::listen_with(|e, _status, window_id| match e {
                // Plan 453 T4c-4：Opened/Focused 捕获已统一改道
                // desktop_window_events()（DM::Desktop 通路），业务侧不再处理
                // 窗口生命周期事件。
                // Plan 065:自退排查打点 —— 会话中途 VM 干净退出 = iced 窗口被关
                // 闭(run() 返回 Ok("UI closed") → exit 0;auto-shell 065 死亡现场
                // 已钉死)。谁发的关闭请求未知(OS 级/焦点/事件误投递),打点让下
                // 一次死亡现场直接看到 CloseRequested 的到达时刻。不改变行为
                // (返回 None,与之前落入 `_ => None` 相同)。
                iced::Event::Window(iced::window::Event::CloseRequested) => {
                    eprintln!(
                        "[window] CloseRequested at {:?} (instrumentation: plan 065)",
                        std::time::SystemTime::now()
                    );
                    // Plan 459：daemon 语义下应用必须自答关窗（window::close）
                    // —— Windows 实测壳层先行代关（本臂未见触发），但其他
                    // 平台的 CloseRequested 会浮出到应用，此臂保证跨平台
                    // 关窗语义完整。
                    Some(DM::Window(
                        window_id,
                        IcedMessage {
                            widget: String::new(),
                            event: "__window_close_request".to_string(),
                            input_value: None,
                        },
                    ))
                }
                iced::Event::Window(iced::window::Event::Resized(size)) => Some(DM::Window(
                    window_id,
                    IcedMessage {
                        widget: String::new(),
                        event: "__window_resized".to_string(),
                        input_value: Some(format!("{}x{}", size.width, size.height)),
                    },
                )),
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Some(DM::Window(
                        window_id,
                        IcedMessage {
                            widget: String::new(),
                            event: "__mouse_moved".to_string(),
                            input_value: Some(format!("{},{}", position.x, position.y)),
                        },
                    ))
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => Some(DM::Window(
                    window_id,
                    IcedMessage {
                        widget: String::new(),
                        event: "__mouse_released".to_string(),
                        input_value: None,
                    },
                )),
                // Plan 462：全局左键按下 → WM 命中测试/聚焦置顶（R12 桌面层
                // 路由的事件半边；独立模式 update 臂配置位门控忽略）。
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Left,
                )) => Some(DM::Wm(crate::ui::session::WmCommand::GlobalPress)),
                // Plan 309 续篇 II: track keyboard modifiers so the inspect
                // picker can switch plain-click (inspect) ↔ Alt-click (native).
                // The subscription callback can't touch session state (other
                // thread), so the value rides the message payload (M1, T4c):
                // `__modifiers_changed` carries `modifiers.bits()`, the update
                // arm writes `desktop.current_modifiers` — the single source.
                //
                // We read modifiers from BOTH `ModifiersChanged` AND every
                // `KeyPressed`/`KeyReleased` (which carry their own `modifiers`
                // field). On Windows, pressing Alt ALONE frequently does not
                // emit `ModifiersChanged` (the key is eaten by the window system
                // menu), so the per-key-event fallback is what actually catches
                // Alt-hold during an Alt+click.
                iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m))
                | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    modifiers: m, ..
                })
                | iced::Event::Keyboard(iced::keyboard::Event::KeyReleased {
                    modifiers: m, ..
                }) => Some(DM::Window(
                    window_id,
                    IcedMessage {
                        widget: String::new(),
                        event: "__modifiers_changed".to_string(),
                        input_value: Some(m.bits().to_string()),
                    },
                )),
                _ => None,
            }));
            iced::Subscription::batch(vec![
                iced::Subscription::batch(subs),
                crate::ui::session::desktop_window_events(),
            ])
        })
        .run();
    host_exit_window_end();
    session_result?;

    // PLAN-575 D1 挂点③：正常返回路径（iced::exit() 关窗/电源键确认退出
    // 都落到这里）——区分"走到正常退出"与"被外界终止"（外界 TerminateProcess
    // 三挂点均不落笔 = 审计零记录 ⇒ 外部击杀实锤）。
    auto_lang_core::vm::ffi::stdlib::exit_audit(0, "main_return");
    Ok("UI closed".to_string())
}

