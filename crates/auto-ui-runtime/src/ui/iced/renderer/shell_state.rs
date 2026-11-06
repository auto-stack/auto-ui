//! PLAN-014 T-06：桌面 shell 状态变更责任（toast/notes/通知、switcher/launcher/dashboard、壁纸/dock/图标栅格/dnd/主题执行族）（自 iced/renderer.rs 原样收编）。
//! 消息序/active判据/last-owner exit/纹理与external pin寿命零改——纯移动。
use super::*;
use super::devtools::*;
use super::media::*;
use super::subscriptions::*;

impl Default for InspectorSubTab {
    fn default() -> Self {
        InspectorSubTab::Inspect
    }
}

impl InspectorSubTab {
    /// Display label for the sub-tab chip. Also used verbatim as the
    /// `__inspector_subtab_<label>` message-tail key (parsed below).
    pub(super) fn label(self) -> &'static str {
        match self {
            InspectorSubTab::Inspect => "检视",
            InspectorSubTab::AutoUI => "AutoUI",
            InspectorSubTab::Source => "源码",
        }
    }

    /// Parse a sub-tab name from a `__inspector_subtab_<name>` message tail.
    /// Returns `None` for unknown names so `update()` can ignore garbage.
    pub(super) fn from_message_tail(tail: &str) -> Option<Self> {
        Some(match tail {
            "检视" => InspectorSubTab::Inspect,
            "AutoUI" => InspectorSubTab::AutoUI,
            "源码" => InspectorSubTab::Source,
            _ => return None,
        })
    }
}

/// Collapsed state of the three sections inside the 检视 sub-tab (Plan 307
/// 续篇 IV). All default expanded (`false`).
#[derive(Default, Clone, Copy)]
pub(crate) struct InspectorSections {
    pub(super) box_collapsed: bool,
    pub(super) computed_collapsed: bool,
    pub(super) props_collapsed: bool,
}

/// Plan 453 T4c：`DynamicState` 已溶解——运行循环 State 即
/// `crate::ui::session::DesktopSession`（R3 退化桌面）。原平铺成员的读写经
/// `DesktopSession::split_mut/split_ref` 拆借视图承接（session.rs，字段名
/// 与旧 DynamicState 一一对应）；窗口登记由 `windows` 注册表接管。

/// Plan 412 续(toast VM 化):一条悬浮通知。kind(default/success/error/
/// warning/info)决定配色,position 支持 top-/bottom-/center × -left/-center/
/// -right(加正中 "center"),duration_ms 到期自动消失(默认 4000ms,
/// 与 vue-sonner 对齐)。
pub(crate) struct ToastReq {
    pub(super) id: u64,
    pub(super) kind: String,
    pub(super) msg: String,
    pub(super) position: String,
    pub(super) shown_at: std::time::Instant,
    pub(super) duration_ms: u64,
}

/// Plan 463 T4：桌面级 toast（LaunchApp 成败回执等）。入共享堆叠 +
/// 置 primary 的 view_dirty（toast 层随 primary 的 dynamic_view 包装渲染）。
fn push_desktop_toast(state: &mut crate::ui::session::DesktopSession, kind: &str, msg: &str) {
    let id = state.desktop.next_toast_id();
    state.desktop.toasts.borrow_mut().push(ToastReq {
        id,
        kind: kind.to_string(),
        msg: msg.to_string(),
        position: "bottom-center".to_string(),
        shown_at: std::time::Instant::now(),
        duration_ms: 3500,
    });
    if let Some(primary) = state.primary_app() {
        if let Some(v) = state.split_mut(primary) {
            *v.app.view_dirty.borrow_mut() = true;
        }
    }
}

/// PLAN-646: Alt+GlobalPress 框选起笔。返回 true = 已吞事件（抑制 WM 焦点
/// 抢占）。锚点取 `last_cursor`（`__mouse_moved` 臂持续回写）；编辑壳在途
/// 保守避让。无 Alt / 编辑中 = false，GlobalPress 照常走 WM 命中聚焦。
/// v1 落在 primary App（跨窗框选为非目标；单 App/独立模式即目标 App）。
pub(super) fn selection_press_anchor(state: &mut crate::ui::session::DesktopSession) -> bool {
    let alt = state.desktop.current_modifiers.borrow().alt();
    if std::env::var("AUTO_SELECT_TRACE").is_ok() {
        eprintln!("[PLAN-646] GlobalPress seen, alt={alt}");
    }
    if !alt {
        return false;
    }
    let Some(app_id) = state.primary_app() else {
        return false;
    };
    let Some(mut v) = state.split_mut(app_id) else {
        return false;
    };
    if v.app.devtools.editing_element.borrow().is_some() {
        return false;
    }
    let (x, y) = v.app.devtools.last_cursor.get();
    *v.app.devtools.marquee.borrow_mut() = Some(crate::ui::selection::Marquee::new(x, y));
    *v.app.view_dirty.borrow_mut() = true;
    true
}

/// Plan 479 T2：通知入史唯一入口（S6 双面一体——「史」入列 +「浮」toast）。
/// 顺序：入史（MRU front 插入，FIFO 容量 [`crate::ui::session::NOTES_CAP`]）
/// → 未读（面板不可见时 +1）→ 落盘（10 槽全量重写）→ toast 浮现（既有
/// 管线不变）→ 面板开着则重注入（打开期间列表活更新）。既有 8 处
/// push_desktop_toast 调用点（LaunchApp 成败/分区删除门/overlay 装载降级）
/// 已改道本入口——行为增量 = 多入史，浮现不变。
/// Plan 497 G3：快照抓取编排——排空 [`snapshot::take_capture_requests`]
/// 队列，非空则记录 pending wid 集 + 发起一次整窗 screenshot（回调
/// [`DesktopEvent::SnapshotShot`]）。411 零尺寸守卫同款（minimized/
/// pre-layout 窗口跳过本轮，下轮重试由渲染臂冷却队列自然再排）。
pub(super) fn service_snapshot_requests(
    state: &mut crate::ui::session::DesktopSession,
) -> Option<iced::Task<crate::ui::session::DesktopMessage>> {
    let wids = crate::ui::iced::snapshot::take_capture_requests();
    if wids.is_empty() {
        return None;
    }
    // 411 零尺寸守卫：快照目标 = **host 窗本体**（显式 id，不再用
    // `window::oldest()`——oldest 可能命中尚未完成尺寸初始化的特权层
    // 窗口，0×0 surface → `create_texture Dimension X is zero` 硬崩溃，
    // PLAN-019 走查启动竞态实测）。host 未就绪/尺寸 0 = 本轮不抓
    //（渲染臂冷却队列下轮自然再排）。
    let Some(host) = state.host.as_ref() else {
        return None;
    };
    let host_window = host.window;
    let host_ok = state
        .windows
        .get(&host_window)
        .map(|w| {
            let s = w.window_size.borrow();
            s.width > 0.0 && s.height > 0.0
        })
        .unwrap_or(false);
    if !host_ok {
        return None;
    }
    state.desktop.snapshot_pending_wids.replace(wids);
    Some(iced::window::screenshot(host_window).map(|ss| {
        crate::ui::session::DesktopMessage::Desktop(crate::ui::session::DesktopEvent::SnapshotShot(
            ss,
        ))
    }))
}

/// Plan 497 G1：dock 时钟注入——本地 HH:MM（chrono Local），分钟变化才
/// 写 shell `__wm_clock` + 置 view_dirty（同分钟重复调用零写入零 dirty；
/// toast tick 先例的"只有变化才 dirty"模式）。时钟不进投影指纹门控组
/// （每分钟翻指纹会引发 dock 全组换装抖动——计划"零投影流量"判定）。
/// **PLAN-014 W-06'（协议 v1.8）**：同泵扩注 `__wm_date`（"M月D日 周X"，
/// 中文周几宿主格式化——.at 无日期算术）——两字段独立脏帧（各自变化才
/// 写；跨天分钟/日期同翻无害幂等），稳态零重建口径不变。
pub(super) fn update_shell_clock(state: &mut crate::ui::session::DesktopSession) {
    use chrono::Datelike;
    let shell_id = state.desktop.shell_app;
    let now = chrono::Local::now();
    let hhmm = now.format("%H:%M").to_string();
    let weekday = match now.weekday() {
        chrono::Weekday::Mon => "周一",
        chrono::Weekday::Tue => "周二",
        chrono::Weekday::Wed => "周三",
        chrono::Weekday::Thu => "周四",
        chrono::Weekday::Fri => "周五",
        chrono::Weekday::Sat => "周六",
        chrono::Weekday::Sun => "周日",
    };
    let date = format!("{}月{}日 {}", now.format("%-m"), now.format("%-d"), weekday);
    let minute_changed = {
        let cur = state.desktop.clock_text.borrow();
        *cur != hhmm
    };
    let date_changed = {
        let cur = state.desktop.date_text.borrow();
        *cur != date
    };
    if !minute_changed && !date_changed {
        return;
    }
    if minute_changed {
        state.desktop.clock_text.replace(hhmm.clone());
    }
    if date_changed {
        state.desktop.date_text.replace(date.clone());
    }
    // PLAN-030 T-04：outproc 壳臂——ShellClockTick 独立变体（分钟门
    // 语义保持）；in-proc 轨零变化（I1）。
    if state.desktop.shell_pipe.is_some() {
        use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
        let msg = ProtocolMsg::Control(ControlMsg::ShellClockTick {
            face: shell_face::SHELL,
            time: hhmm,
            date,
        });
        let _ = state.push_shell_control(&msg);
        return;
    }
    let Some(shell_id) = shell_id else { return };
    if let Some(app) = state.apps.get_mut(&shell_id) {
        if minute_changed {
            let _ = app
                .component
                .write_state("__wm_clock", auto_val::Value::str(&hhmm));
        }
        if date_changed {
            let _ = app
                .component
                .write_state("__wm_date", auto_val::Value::str(&date));
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
}

/// PLAN-041 T-15：dashboard 面板暗色注入位单点同步——`dark_mode()` 翻转
/// （bus set_theme / config 外写 poll / boot 任一路径）经 400ms 帧泵在
/// 此收敛：值变才写面板 `__dash_dark` + 置脏（面板 .at 玻璃底双分支
/// 消费；boot 缺省 dark=true 与 DesktopConfig::default 对齐）。
pub(super) fn sync_dash_dark_bit(state: &mut crate::ui::session::DesktopSession) {
    let dark = crate::ui::style_iced_adapter::dark_mode();
    if state.desktop.dash_dark_cache.get() == dark {
        return;
    }
    state.desktop.dash_dark_cache.set(dark);
    let Some(panel) = state.desktop.dashboard_app else {
        return;
    };
    if let Some(app) = state.apps.get_mut(&panel) {
        let _ = app.component.write_state(
            "__dash_dark",
            auto_val::Value::str(if dark { "1" } else { "0" }),
        );
        *app.state.view_dirty.borrow_mut() = true;
    }
}

/// PLAN-041 T-14（呈现保活）：纯订阅 tick（ServiceTick）在 iced 0.14 不
/// 触发窗口 present——双实例实机实证：update 侧状态/投影每拍更新（
/// __wm_clock 与墙钟同步），OS 表面滞留旧帧（任务栏分钟字停旧、跨分区
/// 残影、通知面板延迟上屏、dashboard face 时钟"停走"四象同根因）。
/// Win32 InvalidateRect+UpdateWindow 强制 WM_PAINT → winit
/// RedrawRequested → 全管线重渲+present。1s 节流（分钟粒度状态一帧即
/// 足）；输入活跃期 winit 自身 present 照常，本臂纯兜底。
#[cfg(windows)]
pub(super) fn presentation_keepalive_tick() {
    static LAST: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    let now = std::time::Instant::now();
    let due = LAST.lock().map_or(true, |g| {
        g.map_or(true, |t| {
            now.duration_since(t) >= std::time::Duration::from_secs(1)
        })
    });
    if !due {
        return;
    }
    if let Ok(mut g) = LAST.lock() {
        *g = Some(now);
    }
    if let Some(h) = crate::ui::native_dock::win32::find_largest_own_window() {
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::Graphics::Gdi::InvalidateRect;
            let hwnd = HWND(h.0 as *mut _);
            // 仅异步失效（UpdateWindow 同步派发会在 update() 内重入
            // winit 事件环——017-chat fit collector unwrap 崩溃同族风险，
            // T-03 后摘除）；WM_PAINT 在消息队列排空后由系统投递。
            let _ = InvalidateRect(hwnd, None, false);
        }
    }
}

/// PLAN-042 T-06：syslog 查看窗注入泵（ServiceTick 段）。窗定位按
/// registry_id（目录名 `039-syslog`，OSCONFIG_APP_ID 先例 wm.wins 线性查）
/// ——窗不在 = 本段零额外工作（环照转，泵零扫描零注入，AC-06）；窗在：
/// `injection_due` 门（seq 无变化零注入 + 500ms 攒批 ≤2Hz）→ snapshot()
/// 全量克隆 → `__syslog_*` 平行字符串列表下行（B12 同族——宿主注入 Obj
/// 数组的 handler 字段读失效，launcher apps_* 先例；time 宿主预格式化
/// HH:MM:SS 本地时区，通知面板 at 串先例）→ 显式 `call_handler("Rebuild")`
/// （宿主写状态不触发 handler，RebuildNotes 同规）+ 置脏一拍重建。
/// v1 全量快照替换，不做增量协议（PLAN-042 §1.3 红线）。
pub(super) fn syslog_inject_tick(state: &mut crate::ui::session::DesktopSession) {
    const SYSLOG_APP_ID: &str = "039-syslog";
    // 顶层优先（z_order 逆序首个非隐藏匹配）——多窗同 app 时泵跟随用户
    // 正在看的窗；全隐藏回退首个在册窗（focus 追随语义）。
    let win_app = state.host.as_ref().and_then(|h| {
        h.wm.wins
            .iter()
            .filter(|(_, v)| v.registry_id.as_deref() == Some(SYSLOG_APP_ID))
            .map(|(k, v)| (k, v.app, v.hidden.get()))
            .collect::<Vec<_>>()
            .into_iter()
            .find(|(k, _, hidden)| !*hidden && h.wm.z_order.iter().rev().any(|z| z == *k))
            .map(|(_, app, _)| app)
            .or_else(|| {
                h.wm.wins
                    .values()
                    .find(|v| v.registry_id.as_deref() == Some(SYSLOG_APP_ID))
                    .map(|v| v.app)
            })
    });
    let Some(app_id) = win_app else { return };
    let dirty = crate::ui::syslog::dirty_seq();
    let now = std::time::Instant::now();
    let last_seq = state.desktop.syslog_inject_seq.get();
    let last_at = state.desktop.syslog_last_inject.get();
    if !crate::ui::syslog::injection_due(dirty, last_seq, last_at, now) {
        return;
    }
    let entries = crate::ui::syslog::snapshot();
    let mut seqs = Vec::with_capacity(entries.len());
    let mut times = Vec::with_capacity(entries.len());
    let mut levels = Vec::with_capacity(entries.len());
    let mut sources = Vec::with_capacity(entries.len());
    let mut msgs = Vec::with_capacity(entries.len());
    for e in &entries {
        seqs.push(auto_val::Value::Str(e.seq.to_string().into()));
        let time_s = chrono::DateTime::from_timestamp_millis(e.ts_ms as i64)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default();
        times.push(auto_val::Value::Str(time_s.into()));
        levels.push(auto_val::Value::Str(e.level.as_str().into()));
        sources.push(auto_val::Value::Str(e.source.clone().into()));
        msgs.push(auto_val::Value::Str(e.msg.clone().into()));
    }
    if let Some(app) = state.apps.get_mut(&app_id) {
        // hosted 旗标：泵首次命中即写 "1"（viewer 独立模式 mock 种子/脚注
        // 隐藏；launcher hosted 同语义）。
        let _ = app
            .component
            .write_state("hosted", auto_val::Value::str("1"));
        let _ = app.component.write_state_vec("__syslog_seq", seqs);
        let _ = app.component.write_state_vec("__syslog_time", times);
        let _ = app.component.write_state_vec("__syslog_level", levels);
        let _ = app.component.write_state_vec("__syslog_source", sources);
        let _ = app.component.write_state_vec("__syslog_msg", msgs);
        if let Err(err) = app.component.bridge_mut().call_handler("Rebuild", &[]) {
            crate::syslog!(
                SyslogLevel::Error,
                "host",
                "[session] syslog viewer Rebuild failed: {err}"
            );
        }
        *app.state.view_dirty.borrow_mut() = true;
        // 簿记后置（写失败下拍重试）。
        state.desktop.syslog_inject_seq.set(dirty);
        state.desktop.syslog_last_inject.set(Some(now));
    }
}

pub(super) fn push_notification(state: &mut crate::ui::session::DesktopSession, kind: &str, msg: &str) {
    // Plan 487 M4 + Plan 540 T2：通知持久化开关门控（479 消费链单点）——
    // 单源 `config.notes_enabled`（设置窗写经宿主臂收口落 config.at），
    // false = 关：notify 动词全链路（入史 + toast + 未读 + 落盘）短路。
    if !state.desktop.config.notes_enabled {
        return;
    }
    // PLAN-035 T-07（AC-09）：尾条去重——与史内最新条 kind+msg 全同即跳过
    // （跨 boot 的 hatch 重扫描不刷屏；进程内同源去重在发件方旗标臂）。
    if state
        .desktop
        .notifications
        .borrow()
        .first()
        .map(|n| n.kind == kind && n.msg == msg)
        .unwrap_or(false)
    {
        return;
    }
    let id = {
        let next = state.desktop.notes_next_id.get();
        state.desktop.notes_next_id.set(next.wrapping_add(1));
        next
    };
    // at = 入史时刻 HH:MM 本地时间（宿主侧格式化，T1 定案 3）。
    let at = chrono::Local::now().format("%H:%M").to_string();
    // PLAN-014 W-08：来源 app id（联合排空泵注册表分段执行期置位；
    // 特权面批量段/宿主内部通知 None → ""）。
    let src_app = state
        .desktop
        .notify_source
        .borrow()
        .clone()
        .unwrap_or_default();
    {
        let mut notes = state.desktop.notifications.borrow_mut();
        notes.insert(
            0,
            crate::ui::session::NotificationEntry {
                id,
                kind: kind.to_string(),
                msg: msg.to_string(),
                at,
                app: src_app,
            },
        );
        notes.truncate(crate::ui::session::NOTES_CAP);
    }
    if !state.notification_visible() {
        state
            .desktop
            .notes_unread
            .set(state.desktop.notes_unread.get() + 1);
    }
    persist_notes(state);
    push_desktop_toast(state, kind, msg);
    refresh_notification_panel(state);
}

/// Plan 479 T2：通知历史落盘——定长 10 槽 `shell.notes.0..9`（T1 定案 4；
/// slot0 = MRU front；每槽一条目 JSON，空槽 = ""，变更后全量重写）。
/// launcher.recent_apps 定长槽同型；`storage_host_publish` 写侧对偶。
fn persist_notes(state: &crate::ui::session::DesktopSession) {
    let notes = state.desktop.notifications.borrow();
    for slot in 0..10usize {
        let value = notes
            .get(slot)
            .map(|n| {
                serde_json::json!({
                    "id": n.id, "kind": n.kind, "msg": n.msg, "at": n.at, "app": n.app
                })
                .to_string()
            })
            .unwrap_or_default();
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            &format!("shell.notes.{slot}"),
            value,
        );
    }
}

/// Plan 479 T2：boot 期通知历史恢复（`storage_host_read` 读侧对偶）。
/// slot 序即 MRU 序；空/坏槽即止（写侧恒全量前缀）。未读不恢复（会话
/// 概念，恒 0 起步）；`notes_next_id` 推进到已恢复最大 id 之上（新条目
/// 不与历史撞 id）。desktop boot 接线见 T5（dock_edges 邻位）。
pub(crate) fn restore_notifications(state: &mut crate::ui::session::DesktopSession) {
    let mut restored: Vec<crate::ui::session::NotificationEntry> = Vec::new();
    for slot in 0..10usize {
        let Some(raw) =
            auto_lang_core::vm::ffi::stdlib::storage_host_read(&format!("shell.notes.{slot}"))
        else {
            break;
        };
        if raw.is_empty() {
            break;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            break;
        };
        let id = v.get("id").and_then(|x| x.as_u64());
        let kind = v.get("kind").and_then(|x| x.as_str());
        let msg = v.get("msg").and_then(|x| x.as_str());
        let at = v.get("at").and_then(|x| x.as_str());
        let (Some(id), Some(kind), Some(msg), Some(at)) = (id, kind, msg, at) else {
            break;
        };
        // PLAN-014 W-08：来源 app 恢复（历史槽无该键 = v1.7 旧库 → ""，
        // 面板行跳来源臂以空串判不可跳）。
        let app = v
            .get("app")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        restored.push(crate::ui::session::NotificationEntry {
            id,
            kind: kind.to_string(),
            msg: msg.to_string(),
            at: at.to_string(),
            app,
        });
    }
    if let Some(max_id) = restored.iter().map(|n| n.id).max() {
        state
            .desktop
            .notes_next_id
            .set(state.desktop.notes_next_id.get().max(max_id + 1));
    }
    *state.desktop.notifications.borrow_mut() = restored;
}

/// Plan 479 T2：面板开着时重注入通知快照（note_* 平行字符串串列 +
/// `RebuildNotes` handler 重建 rows——宿主写状态不触发 handler，switcher
/// 召唤注入同型）。面板未挂载/不可见时零操作。
fn refresh_notification_panel(state: &mut crate::ui::session::DesktopSession) {
    // PLAN-036 T-04（B1）：outproc 轨——快照重推（打开期间活更新；
    /// RebuildNotes 事件重建 rows——in-proc 写集 + handler 同册语义）。
    if state.desktop.shell_pipe.is_some() {
        if state.desktop.notes_open {
            push_notes_snapshot(
                state,
                &[crate::ui::shell_projection::ShellEvent::RebuildNotes],
            );
        }
        return;
    }
    let Some(panel) = state.desktop.notification_app else {
        return;
    };
    if !state.notification_visible() {
        return;
    }
    let mut ids: Vec<auto_val::Value> = Vec::new();
    let mut kinds: Vec<auto_val::Value> = Vec::new();
    let mut msgs: Vec<auto_val::Value> = Vec::new();
    let mut ats: Vec<auto_val::Value> = Vec::new();
    // PLAN-014 W-08：来源 app 平行列表（同 B12 规避族；W-14 落地后消参）。
    let mut apps: Vec<auto_val::Value> = Vec::new();
    {
        let notes = state.desktop.notifications.borrow();
        for n in notes.iter() {
            ids.push(auto_val::Value::Str(n.id.to_string().into()));
            kinds.push(auto_val::Value::Str(n.kind.clone().into()));
            msgs.push(auto_val::Value::Str(n.msg.clone().into()));
            ats.push(auto_val::Value::Str(n.at.clone().into()));
            apps.push(auto_val::Value::Str(n.app.clone().into()));
        }
    }
    let max_h = panel_max_h(state);
    let Some(app) = state.apps.get_mut(&panel) else {
        return;
    };
    let _ = app.component.write_state_vec("note_ids", ids);
    let _ = app.component.write_state_vec("note_kinds", kinds);
    let _ = app.component.write_state_vec("note_msgs", msgs);
    let _ = app.component.write_state_vec("note_ats", ats);
    let _ = app.component.write_state_vec("note_apps", apps);
    let _ = app
        .component
        .write_state("__panel_max_h", auto_val::Value::Int(max_h as i32));
    if let Err(err) = app.component.bridge_mut().call_handler("RebuildNotes", &[]) {
        crate::syslog!(
            SyslogLevel::Error,
            "host",
            "[session] notification RebuildNotes failed: {err}"
        );
    }
    *app.state.view_dirty.borrow_mut() = true;
}

/// PLAN-012 O3：通知面板几何注入（px）——`(panel_h, list_max_h)`。
/// panel_h = 面板根列显式高（视口 - dock 预留）：真实链中 Stack 子层的
/// `h-full`（Fill）约束传递失效（headless 复刻全链通过、实机 mt-auto 填
/// 充条塌缩——iced 0.14.2 真实进程布局差异，O1 同族），显式像素高不再
/// 依赖约束传递，mt-auto 贴底在任何环境下成立。list_max_h = panel_h -
/// 底垫(60) - 顶 gap(24，用户复验：首版 124 扣减顶 gap 仅 ~10px 感知
/// 贴顶) - 标题行(~52) - 余量(12)：条目多时列表滚动，卡片恒有界
///（实机 6 条 ~110px 条目 ≈ 790px > 744 可用 → 卡片贴顶，用户截图复现；
/// 首版仅扣 dock+gap 得 728 仍贴顶——扣减须覆盖卡片全部非列表部分）。
/// 注入点 = 召唤/活更新（resize 开着面板时留旧值，重开生效——v1 可接受）。
fn panel_max_h(state: &crate::ui::session::DesktopSession) -> f32 {
    let viewport = state.host_viewport();
    let reserved = desktop_dock_edges(&state.desktop.config);
    // 紧凑面板（用户复验 2026-09-13：满高卡+小顶缝感知仍为"贴顶"）——
    // 列表上限 = 可用高（视口 - dock）55%（≈4 条目 + 滚动），卡片恒为
    // 右下角紧凑卡；锚定在装配层 container 右下 align（本文件通知层推
    // 臂），.at 内 mt-auto/根高类在真实 Stack 子层不可依赖（复验三连
    // 贴顶 0 实证），全部退役。
    ((viewport.height - reserved.bottom) * 0.55).clamp(280.0, 560.0)
}

/// Plan 463 T4：执行 DesktopBus 命令序列（T1 报告 §5）。返回 true = 请求
/// 退出进程（shell 缺位时关掉最后一个虚拟窗沿用 462 daemon 语义；有 shell
/// 时空桌面是合法态，持续存活等待启动）。
/// Plan 464 T4：launcher overlay 召唤执行体（Ctrl+Space 热键事件与 shell
/// ⊞ 总线记录同落此处）。首次懒挂载（build_dynamic_component 自动触发
/// .Init：装载 storage recent + 网格行），随后下行注入：
/// `apps`（真注册表清单，R10——ln/lt 预小写由宿主供给，匹配零 .at 依赖）、
/// `hosted`（隐藏独立模式开关）、`visible`（overlay 置位）、
/// `__focus_input`（聚焦请求，T3 约定）。同步调 ApplyFilter 重算 ranked
/// （宿主写状态不触发 handler，须显式重算）；返回聚焦任务（打开即聚焦，
/// P3；update_inner 尾部的 __focus_input 消费在下一 launcher 消息周期
/// 再生效——双保险）。注册表无 launcher 时 toast 降级（441 fallback 边界）。
pub(crate) fn summon_launcher(
    state: &mut crate::ui::session::DesktopSession,
) -> iced::Task<crate::ui::session::DesktopMessage> {
    use crate::ui::session::DesktopMessage as DM;
    // PLAN-036 T-07（B3，D3-C）：outproc 轨（B 形态桌面 = shell_pipe 在
    /// 场）——launcher 独立 exe：已 attach 则快照推送（ApplyFilter 事件
    /// + child 自主聚焦[D5 等价]）；未 spawn 则一次性 spawn（attach 异步
    ///——ServiceTick 补推追赶）；伪窗聚焦承接键盘路由（D5 可聚焦语义）。
    /// in-proc 轨原路径零变化（I2）。
    if state.desktop.shell_pipe.is_some() {
        state.desktop.launcher_open = true;
        if state.desktop.launcher_pipe.is_some() {
            if let Some(lwid) = state.desktop.launcher_wid {
                state.wm_focus(lwid);
            }
            push_launcher_snapshot(
                state,
                &[crate::ui::shell_projection::ShellEvent::ApplyFilter],
            );
        } else if !state.desktop.launcher_spawned {
            if let Err(err) = state.spawn_launcher_outproc() {
                crate::syslog!(
                    SyslogLevel::Error,
                    "host",
                    "[p036] launcher exe spawn 失败: {err}"
                );
                state.desktop.launcher_open = false;
            }
        }
        return iced::Task::none();
    }
    if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
        eprintln!(
            "[464-SUMMON] summon_launcher entered, mounted={}",
            state.desktop.launcher_app.is_some()
        );
    }
    // 1. 懒挂载
    if state.desktop.launcher_app.is_none() {
        let Some(entry) = state.desktop.launcher_entry.clone() else {
            push_notification(state, "error", "launcher 不可用（注册表未含 028-launcher）");
            return iced::Task::none();
        };
        let Ok(code) = std::fs::read_to_string(&entry) else {
            push_notification(state, "error", "launcher 源读取失败");
            return iced::Task::none();
        };
        match crate::build_dynamic_component(&code, Some(entry.to_string_lossy().as_ref())) {
            Ok(comp) => {
                let app_id = state.allocate_app(comp);
                state.desktop.launcher_app = Some(app_id);
            }
            Err(err) => {
                push_notification(state, "error", &format!("launcher 装载失败: {err}"));
                return iced::Task::none();
            }
        }
    }
    let launcher = state.desktop.launcher_app.expect("launcher mounted");
    // 2. 下行注入（真注册表 + hosted/visible/聚焦请求）。
    // 注入形态 = 平行字符串列表（Obj 数组的 VM handler 字段读失效——
    // B12 同族，`injected_obj_array_for_field_read` 探针钉死 "0,0"；
    // 字符串列表下标读保真）。ln/lt 预小写由宿主供给。
    // PLAN-012 W6：launcher 自身不入列（入册规则同 :11609 发现链镜像），
    // `launcher_entry` 发现链不受影响（先取 entry 再过滤）。
    let entries = state
        .desktop
        .registry_entries
        .iter()
        .filter(|e| !(e.id == "launcher" || e.id.ends_with("-launcher")))
        .cloned()
        .collect::<Vec<_>>();
    let mut names: Vec<auto_val::Value> = Vec::new();
    let mut titles: Vec<auto_val::Value> = Vec::new();
    let mut icons: Vec<auto_val::Value> = Vec::new();
    let mut cats: Vec<auto_val::Value> = Vec::new();
    let mut lns: Vec<auto_val::Value> = Vec::new();
    let mut lts: Vec<auto_val::Value> = Vec::new();
    let mut colors: Vec<auto_val::Value> = Vec::new();
    for e in &entries {
        names.push(auto_val::Value::Str(e.id.clone().into()));
        // PLAN-015：展示名走 locale 解析链（zh=title_zh→title…）；搜索
        // 文本同源（display 后 lowercase；id 小写键 lns 仍在，中英两可搜）。
        titles.push(auto_val::Value::Str(e.display_title().to_string().into()));
        icons.push(auto_val::Value::Str(e.icon.clone().into()));
        cats.push(auto_val::Value::Str(e.category.clone().into()));
        lns.push(auto_val::Value::Str(e.id.to_lowercase().into()));
        lts.push(auto_val::Value::Str(
            e.display_title().to_lowercase().into(),
        ));
        colors.push(auto_val::Value::Str(launcher_brand_color(&e.id).into()));
    }
    if let Some(app) = state.apps.get_mut(&launcher) {
        let _ = app.component.write_state_vec("apps_names", names);
        let _ = app.component.write_state_vec("apps_titles", titles);
        let _ = app.component.write_state_vec("apps_icons", icons);
        let _ = app.component.write_state_vec("apps_cats", cats);
        let _ = app.component.write_state_vec("apps_lns", lns);
        let _ = app.component.write_state_vec("apps_lts", lts);
        let _ = app.component.write_state_vec("apps_colors", colors);
        let _ = app
            .component
            .write_state("hosted", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("visible", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("__focus_input", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("__focus_input_tries", auto_val::Value::Int(0));
        // 宿主写状态不触发 handler——显式重算 ranked/网格行 + 刷 view
        if let Err(err) = app.component.bridge_mut().call_handler("ApplyFilter", &[]) {
            crate::syslog!(
                SyslogLevel::Error,
                "host",
                "[session] launcher ApplyFilter failed: {err}"
            );
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
    // 3. 聚焦（打开即聚焦 P3；任务回 DM::App(launcher) 对齐消息形状）。
    // Plan 483: 目标 = launcher 视图登记的首个 input Id(focus Task 在
    // 下一帧 view 重建后执行,登记已就位);空表退旧字面量(无匹配即空聚焦)。
    // PLAN-526 T13：首次召唤登记表恒空（视图从未建→回退 prompt_input
    // 字面量盲聚焦，实测打字无反应）——同步构建一次 AbstractView 预登记
    // （derive_input_id 跨重建稳定，下一帧真视图同 Id 收敛）。
    let summon_target = {
        let mut app = state.apps.get_mut(&launcher).expect("launcher mounted");
        let (view, _, _) = app.component.view_with_debug_gated(false);
        let converted = convert_view_messages(view);
        let mut ids = Vec::new();
        collect_input_ids(&converted, &mut ids);
        // PLAN-013 W2：登记持久化——overlay 挂载面不走自身窗口渲染路径，
        // devtools.input_ids 恒空（registered=0）→ update_inner 尾部消费
        // 永远回退 prompt_input 死 Id → 不聚焦+无法输入。此处把同步构建
        // 收集到的真 Id 写入 launcher 自己的登记表，消费端即命中。
        *app.state.devtools.input_ids.borrow_mut() = ids.clone();
        ids.first().cloned()
    }
    .unwrap_or_else(|| iced::widget::Id::new("prompt_input"));
    iced::widget::operation::focus(summon_target).map(move |m| DM::App(launcher, m))
}

/// Plan 503 M4：launcher 品牌色（stella 柔粉彩系，6 位 hex）。注册表条目
/// 无 color 字段——已知 app 静态映射，未知 app 以 id 哈希散列到同系粉彩
/// 盘（稳定确定性：同名恒同色）。.at 侧消费为 `bg-[<color>21]`（13% alpha
/// 图标底块）+ `text-[<color>]`（品牌色字形）。
fn launcher_brand_color(id: &str) -> &'static str {
    match id {
        "011-calculator" => "#7c9a6d",
        "012-clock" | "012-stopwatch" => "#b88c61",
        "013-todo" => "#6a8bad",
        "014-weather" => "#7d9ec4",
        "015-notes" => "#c9a77e",
        "017-chat" => "#8d7ab5",
        "018-book-reader" => "#a47551",
        "020-music-player" => "#c4706a",
        "024-charts" => "#6fa3a7",
        "025-dashboard" => "#9a8fb8",
        "038-minesweeper" => "#8a9b6e",
        "041-auto-edit" => "#5f8d78",
        _ => {
            const PASTEL: [&str; 8] = [
                "#7c9a6d", "#b88c61", "#6a8bad", "#7d9ec4", "#c9a77e", "#8d7ab5", "#a47551",
                "#c4706a",
            ];
            let hash = id.bytes().map(|b| b as usize).sum::<usize>();
            PASTEL[hash % PASTEL.len()]
        }
    }
}

/// Plan 478 T4：switcher overlay 召唤执行体（Ctrl+Tab 热键；T1 施工图 §5.2，
/// 464 summon_launcher 同型）。懒挂载（`assets/switcher.at` 进程内嵌——无
/// 注册表依赖，构建失败 toast）→ 下行注入 MRU 快照：`mru_wids`/`mru_titles`/
/// `mru_icons` 平行字符串列表（B12 规避，launcher apps_* 同型）+ 合同面
/// `__wm_mru`（协议 v1.1 Obj 数组）+ `hosted`/`visible` 置位 → 显式
/// `call_handler("RebuildMru")`（宿主写状态不触发 handler）重建 rows + sel
/// 复位。快照语义：召唤时点定序，打开期间推进不重注。无输入控件——无
/// `__focus_input`/聚焦任务（键盘路由走订阅独占，R12）。
pub(super) fn summon_switcher(
    state: &mut crate::ui::session::DesktopSession,
) -> iced::Task<crate::ui::session::DesktopMessage> {
    // PLAN-036 T-04（B1）：outproc 轨推送臂——快照 + RebuildMru 事件下行
    ///（child 预装/懒装面 apply_writes + dispatch 重建 rows）；in-proc
    /// 原路径零变化（I2）。开关镜像位 = switcher_open（层序/仲裁消费）。
    if state.desktop.shell_pipe.is_some() {
        state.desktop.switcher_open = true;
        push_switcher_snapshot(
            state,
            &[crate::ui::shell_projection::ShellEvent::RebuildMru],
        );
        return iced::Task::none();
    }
    // 1. 懒挂载
    if state.desktop.switcher_app.is_none() {
        match crate::ui::shell::build_switcher_component() {
            Ok(comp) => {
                let app_id = state.allocate_app(comp);
                state.desktop.switcher_app = Some(app_id);
            }
            Err(err) => {
                push_notification(state, "error", &format!("switcher 装载失败: {err}"));
                return iced::Task::none();
            }
        }
    }
    let switcher = state.desktop.switcher_app.expect("switcher mounted");
    // 2. MRU 快照（当前分区；焦点环不跨分区——退役 Ctrl+Tab 语义延续）。
    let Some(host) = state.host.as_ref() else {
        return iced::Task::none();
    };
    let mut wids: Vec<auto_val::Value> = Vec::new();
    let mut titles: Vec<auto_val::Value> = Vec::new();
    let mut icons: Vec<auto_val::Value> = Vec::new();
    // Plan 497 G5：快照就绪标记（"1"/""，平行于 mru_icons 的合同面形态；
    // 行缩略渲染由 window_thumbnail 渲染臂自决——miss 走 fallback icon +
    // request_capture，本标记供测试/后续增强分支消费）。召唤同时把
    // miss 窗显式入队预抓（打开期间快照新鲜；渲染臂冷却队列同源）。
    let mut thumbs: Vec<auto_val::Value> = Vec::new();
    let mut mru_objs: Vec<auto_val::Value> = Vec::new();
    for wid in host.wm.mru_in_workspace(host.wm.current_workspace) {
        let Some(v) = host.wm.wins.get(&wid) else {
            continue;
        };
        let focused = host.wm.focused == Some(wid);
        wids.push(auto_val::Value::Str(v.wid.0.to_string().into()));
        titles.push(auto_val::Value::Str(v.title.clone().into()));
        let ready = crate::ui::iced::snapshot::snapshot_window(wid).is_some();
        if !ready {
            crate::ui::iced::snapshot::request_capture(wid);
        }
        thumbs.push(auto_val::Value::str(if ready { "1" } else { "" }));
        let icon = v
            .registry_id
            .as_ref()
            .and_then(|id| state.desktop.registry_entries.iter().find(|e| &e.id == id))
            .map(|e| e.icon.clone())
            .unwrap_or_else(|| "app-window".to_string());
        icons.push(auto_val::Value::Str(icon.into()));
        mru_objs.push(projection_win_entry(
            &state.desktop.registry_entries,
            v,
            focused,
            "",
            false,
            false,
        ));
    }
    if let Some(app) = state.apps.get_mut(&switcher) {
        let _ = app.component.write_state_vec("mru_wids", wids);
        let _ = app.component.write_state_vec("mru_titles", titles);
        let _ = app.component.write_state_vec("mru_icons", icons);
        let _ = app.component.write_state_vec("mru_thumbs", thumbs);
        let _ = app.component.write_state_vec("__wm_mru", mru_objs);
        let _ = app
            .component
            .write_state("hosted", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("visible", auto_val::Value::str("1"));
        // 宿主写状态不触发 handler——显式重建 rows（sel 复位在 handler 内）+ 刷 view
        if let Err(err) = app.component.bridge_mut().call_handler("RebuildMru", &[]) {
            crate::syslog!(
                SyslogLevel::Error,
                "host",
                "[session] switcher RebuildMru failed: {err}"
            );
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
    iced::Task::none()
}

/// Plan 479 T3：通知中心 overlay 召唤执行体（notes_toggle 总线动词；T1
/// 施工图定案 1，summon_switcher 同型第三枚）。懒挂载（assets/
/// notification_center.at 进程内嵌——无注册表依赖，装载失败走
/// push_notification 降级）→ 可见即自隐（toggle 语义），隐时打开：快照注入
/// note_ids/note_kinds/note_msgs/note_ats 平行字符串列表（B12 规避）+
/// `hosted`/`visible` 置位 → 显式 `call_handler("RebuildNotes")`（宿主写
/// 状态不触发 handler）重建 rows + **未读清零**（打开即读）。无输入控件
/// ——无 `__focus_input`/聚焦任务（Esc 关闭走订阅独占，R12）。
fn toggle_notification_center(
    state: &mut crate::ui::session::DesktopSession,
) -> iced::Task<crate::ui::session::DesktopMessage> {
    // PLAN-036 T-04（B1）：outproc 轨推送臂——toggle 语义镜像（开 =
    /// RebuildNotes 事件 + 未读清零；关 = visible=0 纯状态位）；in-proc
    /// 原路径零变化（I2）。
    if state.desktop.shell_pipe.is_some() {
        let open = !state.desktop.notes_open;
        state.desktop.notes_open = open;
        if open {
            state.desktop.notes_unread.set(0);
            push_notes_snapshot(
                state,
                &[crate::ui::shell_projection::ShellEvent::RebuildNotes],
            );
        } else {
            push_notes_snapshot(state, &[]);
        }
        return iced::Task::none();
    }
    // 1. 懒挂载
    if state.desktop.notification_app.is_none() {
        match crate::ui::shell::build_notification_center_component() {
            Ok(comp) => {
                let app_id = state.allocate_app(comp);
                state.desktop.notification_app = Some(app_id);
            }
            Err(err) => {
                push_notification(state, "error", &format!("通知中心装载失败: {err}"));
                return iced::Task::none();
            }
        }
    }
    // 2. toggle：可见 → 自隐（未读不动——打开时已清零）。
    if state.notification_visible() {
        let panel = state.desktop.notification_app.expect("panel checked");
        if let Some(app) = state.apps.get_mut(&panel) {
            let _ = app
                .component
                .write_state("visible", auto_val::Value::str("0"));
            *app.state.view_dirty.borrow_mut() = true;
        }
        return iced::Task::none();
    }
    // 3. 打开：快照注入 + 重建 rows + 未读清零 + 刷 view。
    let panel = state.desktop.notification_app.expect("panel mounted");
    let mut ids: Vec<auto_val::Value> = Vec::new();
    let mut kinds: Vec<auto_val::Value> = Vec::new();
    let mut msgs: Vec<auto_val::Value> = Vec::new();
    let mut ats: Vec<auto_val::Value> = Vec::new();
    // PLAN-014 W-08：来源 app 平行列表（与 refresh_notification_panel 同形）。
    let mut apps: Vec<auto_val::Value> = Vec::new();
    {
        let notes = state.desktop.notifications.borrow();
        for n in notes.iter() {
            ids.push(auto_val::Value::Str(n.id.to_string().into()));
            kinds.push(auto_val::Value::Str(n.kind.clone().into()));
            msgs.push(auto_val::Value::Str(n.msg.clone().into()));
            ats.push(auto_val::Value::Str(n.at.clone().into()));
            apps.push(auto_val::Value::Str(n.app.clone().into()));
        }
    }
    let max_h = panel_max_h(state);
    if let Some(app) = state.apps.get_mut(&panel) {
        let _ = app.component.write_state_vec("note_ids", ids);
        let _ = app.component.write_state_vec("note_kinds", kinds);
        let _ = app.component.write_state_vec("note_msgs", msgs);
        let _ = app.component.write_state_vec("note_ats", ats);
        let _ = app.component.write_state_vec("note_apps", apps);
        let _ = app
            .component
            .write_state("hosted", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("visible", auto_val::Value::str("1"));
        let _ = app
            .component
            .write_state("__panel_max_h", auto_val::Value::Int(max_h as i32));
        // 宿主写状态不触发 handler——显式重建 rows + 刷 view。
        if let Err(err) = app.component.bridge_mut().call_handler("RebuildNotes", &[]) {
            crate::syslog!(
                SyslogLevel::Error,
                "host",
                "[session] notification RebuildNotes failed: {err}"
            );
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
    state.desktop.notes_unread.set(0);
    iced::Task::none()
}

// ============================================================================
// PLAN-036 T-04（B1）：switcher/通知中心 outproc 推送泵——030 双面泵
// （push_shell_projection_outproc/push_desktop_surface_outproc）推广到
// overlay 两面。数据收集与 in-proc 召唤动词逐字段同源（summon_switcher/
// toggle_notification_center 上方写集的 typed 化）；指纹门放行 = fp 变化
// || events 非空（键盘动词是瞬态事件非状态——D6）。
// ============================================================================

/// switcher 快照构建（mru 平行列表 + `__wm_mru` 合同面——召唤时点定序）。
pub(crate) fn build_switcher_snapshot(
    state: &crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::SwitcherSnapshot {
    use crate::ui::shell_projection::{ShellWin, SwitcherSnapshot};
    let mut snap = SwitcherSnapshot {
        hosted: true,
        ..Default::default()
    };
    let Some(host) = state.host.as_ref() else {
        return snap;
    };
    for wid in host.wm.mru_in_workspace(host.wm.current_workspace) {
        // PLAN-709 v1.11：槽位条目并入 switcher（Docked 门控；wid="N<id>"
        // ——缩略 widget 对 native wid 天然 lucide/native-icon 回退（497 注），
        // thumbs 恒空无快照抓取）。
        if let Some(slot_id) = crate::ui::session::WmState::native_slot_id_of_pseudo(wid) {
            let Some(slot) = host.wm.native_slots.get(&slot_id) else {
                continue;
            };
            if slot.state != crate::ui::native_dock::SlotState::Docked {
                continue;
            }
            let focused = host.wm.focused == Some(wid);
            snap.mru_wids.push(format!("N{}", slot_id.0));
            snap.mru_titles.push(slot.title_cache.clone());
            snap.mru_thumbs.push(String::new());
            snap.mru_icons
                .push(crate::ui::iced::native_icon::icon_field(slot_id.0).to_string());
            snap.wm_mru.push(ShellWin {
                wid: format!("N{}", slot_id.0),
                title: slot.title_cache.clone(),
                focused,
                workspace: Some(slot.workspace),
                native: true,
                app: String::new(),
                icon: snap.mru_icons.last().cloned().unwrap_or_default(),
                pager: false,
                pinned: false,
                dup_app: false,
            });
            continue;
        }
        let Some(v) = host.wm.wins.get(&wid) else {
            continue;
        };
        // 壳伪窗不入切换列表（shell_pseudo_wids 过滤——build_shell_
        // projection 同册；overlay 伪窗加入后此过滤同样覆盖）。
        if state.desktop.shell_pseudo_wids.contains(&wid) {
            continue;
        }
        let focused = host.wm.focused == Some(wid);
        snap.mru_wids.push(v.wid.0.to_string());
        snap.mru_titles.push(v.title.clone());
        // Plan 497 G5：快照就绪标记 + miss 预抓（in-proc 同款）。
        let ready = crate::ui::iced::snapshot::snapshot_window(wid).is_some();
        if !ready {
            crate::ui::iced::snapshot::request_capture(wid);
        }
        snap.mru_thumbs
            .push(if ready { "1" } else { "" }.to_string());
        snap.mru_icons.push(
            v.registry_id
                .as_ref()
                .and_then(|id| state.desktop.registry_entries.iter().find(|e| &e.id == id))
                .map(|e| e.icon.clone())
                .unwrap_or_else(|| "app-window".to_string()),
        );
        snap.wm_mru.push(ShellWin {
            wid: v.wid.0.to_string(),
            title: v.title.clone(),
            focused,
            workspace: Some(v.workspace),
            native: false,
            app: v.registry_id.clone().unwrap_or_default(),
            icon: snap.mru_icons.last().cloned().unwrap_or_default(),
            pager: false,
            pinned: false,
            dup_app: false,
        });
    }
    snap
}

/// switcher 快照推送（face=SWITCHER；visible = switcher_open 镜像位）。
pub(crate) fn push_switcher_snapshot(
    state: &mut crate::ui::session::DesktopSession,
    events: &[crate::ui::shell_projection::ShellEvent],
) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let mut snap = build_switcher_snapshot(state);
    snap.visible = state.desktop.switcher_open;
    snap.events = events.to_vec();
    let fp = snap.fingerprint();
    let transient = !snap.events.is_empty();
    if !transient && state.desktop.switcher_fp.as_deref() == Some(fp.as_str()) {
        return;
    }
    let mut payload = Vec::new();
    snap.wire_encode(&mut payload);
    let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
        face: shell_face::SWITCHER,
        payload,
    });
    if state.push_shell_control(&msg) {
        state.desktop.switcher_fp = Some(fp);
    }
}

/// 通知快照构建（note_* 平行列表 + 合同面 + 面板几何）。
pub(crate) fn build_notes_snapshot(
    state: &crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::NotesSnapshot {
    use crate::ui::shell_projection::{NotesSnapshot, ShellNote};
    let mut snap = NotesSnapshot {
        hosted: true,
        ..Default::default()
    };
    {
        let notes = state.desktop.notifications.borrow();
        for n in notes.iter() {
            snap.note_ids.push(n.id.to_string());
            snap.note_kinds.push(n.kind.clone());
            snap.note_msgs.push(n.msg.clone());
            snap.note_ats.push(n.at.clone());
            snap.note_apps.push(n.app.clone());
            snap.wm_notes.push(ShellNote {
                id: n.id,
                kind: n.kind.clone(),
                msg: n.msg.clone(),
                at: n.at.clone(),
                app: n.app.clone(),
            });
        }
    }
    snap.wm_notes_unread = state.desktop.notes_unread.get() as u64;
    snap.panel_max_h = panel_max_h(state) as u32;
    snap
}

/// PLAN-036 T-07（B3）：launcher 快照构建 + 推送（face=LAUNCHER，定向
/// launcher exe 管线——独立于壳 pipe；七平行列表与 summon_launcher
/// in-proc 写点逐字段同源；visible = launcher_open 镜像位）。
pub(crate) fn build_launcher_snapshot(
    state: &crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::LauncherSnapshot {
    use crate::ui::shell_projection::LauncherSnapshot;
    let mut snap = LauncherSnapshot {
        hosted: true,
        ..Default::default()
    };
    for e in state.desktop.registry_entries.iter() {
        if e.id == "launcher" || e.id.ends_with("-launcher") {
            continue;
        }
        let title = e.display_title().to_string();
        snap.app_ids.push(e.id.clone());
        snap.app_titles.push(title.clone());
        snap.app_icons.push(e.icon.clone());
        snap.app_cats.push(e.category.clone());
        snap.app_lns.push(e.id.to_lowercase());
        snap.app_lts.push(title.to_lowercase());
        snap.app_colors
            .push(launcher_brand_color(&e.id).to_string());
    }
    snap
}

pub(crate) fn push_launcher_snapshot(
    state: &mut crate::ui::session::DesktopSession,
    events: &[crate::ui::shell_projection::ShellEvent],
) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let Some(pipe) = state.desktop.launcher_pipe.clone() else {
        return;
    };
    let mut snap = build_launcher_snapshot(state);
    snap.visible = state.desktop.launcher_open;
    snap.events = events.to_vec();
    let fp = snap.fingerprint();
    let transient = !snap.events.is_empty();
    if !transient && state.desktop.launcher_fp.as_deref() == Some(fp.as_str()) {
        return;
    }
    let mut payload = Vec::new();
    snap.wire_encode(&mut payload);
    let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
        face: shell_face::LAUNCHER,
        payload,
    });
    if state.push_control_to(&pipe, &msg) {
        state.desktop.launcher_fp = Some(fp);
    }
}

/// PLAN-036 T-05（B2）：dashboard 快照构建 + 推送（face=DASHBOARD；
/// visible = dashboard_open 镜像位；伪窗命中带随面板几何动态更新——
/// D2-A 中间 z 档的命中带化）。
pub(crate) fn build_dashboard_snapshot(
    state: &mut crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::DashboardSnapshot {
    use crate::ui::shell_projection::{DashboardFace, DashboardSnapshot};
    let (faces, panel_w, panel_h, panel_top) = collect_dashboard_faces(state);
    let mut snap = DashboardSnapshot {
        hosted: true,
        ..Default::default()
    };
    for f in faces {
        snap.faces.push(DashboardFace {
            id: f.id,
            title: f.title,
            icon: f.icon,
            status: f.status.to_string(),
            span: f.span.to_string(),
            tab: f.tab.to_string(),
        });
    }
    snap.panel_w = panel_w.max(0.0) as u32;
    snap.panel_h = panel_h.max(0.0) as u32;
    snap.panel_top = panel_top.max(0.0) as u32;
    snap
}

pub(crate) fn push_dashboard_snapshot(
    state: &mut crate::ui::session::DesktopSession,
    events: &[crate::ui::shell_projection::ShellEvent],
) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let mut snap = build_dashboard_snapshot(state);
    snap.visible = state.desktop.dashboard_open;
    snap.events = events.to_vec();
    // T-06：face 缓存刷新（宿主叠层消费——in-proc 读面板 state 的
    // outproc 等价）。
    *state.desktop.dashboard_face_cache.borrow_mut() = snap
        .faces
        .iter()
        .map(|f| {
            (
                f.id.clone(),
                f.title.clone(),
                f.icon.clone(),
                f.status.clone(),
                f.span.parse::<usize>().unwrap_or(2),
                f.tab.clone(),
            )
        })
        .collect();
    // 命中带动态更新：dashboard 伪窗（pseudo[4]）rect ← 面板几何
    ///（右上锚定：x = 视口 - w - 12，y = panel_top——035 固定外框契约）。
    if let (Some(dwid), Some(host)) = (
        state.desktop.shell_pseudo_wids.get(4).copied(),
        state.host.as_mut(),
    ) {
        let vw = state
            .desktop
            .shell_geometry
            .map(|g| g.viewport_w)
            .unwrap_or(snap.panel_w as f32);
        if let Some(v) = host.wm.wins.get_mut(&dwid) {
            *v.rect.borrow_mut() = iced::Rectangle::new(
                iced::Point::new(
                    (vw - snap.panel_w as f32 - 12.0).max(12.0),
                    snap.panel_top as f32,
                ),
                // 高度含下缘 tab 条预留带（与 .at 根块 h = 外框 + 28 同构，
                // 条内 tab 点击落入面板伪窗命中带）。
                iced::Size::new(snap.panel_w as f32, snap.panel_h as f32 + DASH_TAB_STRIP_H),
            );
        }
    }
    let fp = snap.fingerprint();
    let transient = !snap.events.is_empty();
    if !transient && state.desktop.dashboard_fp.as_deref() == Some(fp.as_str()) {
        return;
    }
    let mut payload = Vec::new();
    snap.wire_encode(&mut payload);
    let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
        face: shell_face::DASHBOARD,
        payload,
    });
    if state.push_shell_control(&msg) {
        state.desktop.dashboard_fp = Some(fp);
    }
}

/// 通知快照推送（face=NOTIFICATION_CENTER；visible = notes_open 镜像位）。
pub(crate) fn push_notes_snapshot(
    state: &mut crate::ui::session::DesktopSession,
    events: &[crate::ui::shell_projection::ShellEvent],
) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let mut snap = build_notes_snapshot(state);
    snap.visible = state.desktop.notes_open;
    snap.events = events.to_vec();
    let fp = snap.fingerprint();
    let transient = !snap.events.is_empty();
    if !transient && state.desktop.notes_fp.as_deref() == Some(fp.as_str()) {
        return;
    }
    let mut payload = Vec::new();
    snap.wire_encode(&mut payload);
    let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
        face: shell_face::NOTIFICATION_CENTER,
        payload,
    });
    if state.push_shell_control(&msg) {
        state.desktop.notes_fp = Some(fp);
    }
}

// ============================================================================
// PLAN-024：dashboard 面板（第四 overlay 槽——设置面板退役后继任；通知层
// 邻位顶层）。面板 chrome = dashboard.at 特权面（懒挂载）；face 卡 = 各
// App `view mini` 命名视图的宿主拆借渲染（活渲染面，DM::App 打标直达该
// App 会话）。配置 `shell.dashboard.*` storage（§5.6：面板 Init 读回 +
// 宿主注入两段式，单一事实在宿主侧读回）。
// ============================================================================

/// dashboard 面板布局常量（PLAN-035 T-04 v2：桌面图标网格吸附——外框 =
/// 屏幕右上 8 列×2 行网格块，widget 卡 = 网格单元整数倍（缺省 2×2、
/// 声明/存储 3×2），卡框线落 88/80 节距网格）。宿主计算单一事实，面板
/// .at 经 `__panel_*` 注入镜像；face 格位由同一算式产出（像素级一致，
/// 零漂移）。2026-09-22 用户裁定 8×3→8×2：T-18 满高 3 行格把 3x2/2x2
/// 声明卡拉伸成 3 行高，面板回 2 行后卡落自然 2 格高。
const DASH_GRID_COL: f32 = 88.0; // w-20 80 + gap 8（桌面图标列距）
const DASH_GRID_ROW: f32 = 80.0; // h-[72px] 72 + gap 8（桌面图标行距）
const DASH_MARGIN: f32 = 12.0; // 屏幕边距（桌面内容 p-3 同源）
const DASH_COLS: usize = 8; // 外框列数
const DASH_ROWS: usize = 2; // 外框行数

const DASH_FRAME_PAD: f32 = 12.0; // 外框四围 padding（PLAN-035 T-14：
                                  // 外框 = 8×2 网格块四周外扩 12px 做 chrome 留白——卡片不再贴框缘）

/// 面板下缘 tab 条预留带（2026-09-22 用户裁定：分页 pill 从右上悬浮
/// 改面板正下方 hover 显隐）。chrome wrapper 高度 = 面板外框 + 本带；
/// .at 内 tab 行常驻占位（h-7 = 28px），按钮 hover 显隐——单一
/// mouse-area 包面板+条全程，无跨元素 enter/leave 抖动。
pub(super) const DASH_TAB_STRIP_H: f32 = 28.0;

/// face 卡片（格位算式输入）：registry id + 列跨度 + 所属 tab。
/// R5：tab 由注册表 category 派生（"system" → 系统页，其余 → 小组件页）
/// ——实时轮询类组件住系统页，非活动页宿主不渲染且孵化 Tick 停订。
pub(super) struct DashFace {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) icon: String,
    pub(super) status: &'static str, // running | hatched | placeholder
    pub(super) span: usize,
    pub(super) tab: &'static str, // main | system
}

/// PLAN-024：face 卡 spacer 链定位（viewport 绝对格位 → 全幅层）——
/// row[h-space(x), col[v-space(y), card]]，家法（真实 Stack 子层
/// padding/align 不可依赖）。
pub(super) fn spare_position<M: Clone + std::fmt::Debug + 'static>(
    card: iced::Element<'_, M>,
    rect: iced::Rectangle,
) -> iced::Element<'_, M> {
    let placed = iced::widget::container(iced::widget::row![
        iced::widget::Space::new()
            .width(iced::Length::Fixed(rect.x))
            .height(iced::Length::Shrink),
        iced::widget::column![
            iced::widget::Space::new()
                .width(iced::Length::Shrink)
                .height(iced::Length::Fixed(rect.y)),
            card,
        ],
    ])
    .width(iced::Length::Fill)
    .height(iced::Length::Fill);
    placed.into()
}

/// 面板布局算式（宿主/面板几何单一事实）——返回 (panel_w, panel_h,
/// panel_top, 每张 face 的视口绝对格位矩形，行主序 next-fit：span 大于
/// 余量即换行)。面板 = 顶部居中（x 居中，top 注入）；高 = 标题行 + 行数
/// ×格高 + (行数+1)×gap + 2×pad。
#[cfg(test)]
mod plan024_dashboard_layout_tests {
    use super::*;

    fn face(id: &str, span: usize) -> DashFace {
        DashFace {
            id: id.to_string(),
            title: id.to_string(),
            icon: "app-window".into(),
            status: "hatched",
            span,
            tab: "main",
        }
    }

    const VP: iced::Rectangle = iced::Rectangle {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 800.0,
    };

    /// 空清单：无格位，外框 = 8×2 网格块四围外扩 PAD 的固定矩形。
    #[test]
    fn empty_faces_fixed_frame() {
        let (panel, cells) = dashboard_layout(VP, &[]);
        assert!(cells.is_empty());
        assert_eq!(
            panel.width,
            8.0 * DASH_GRID_COL - 8.0 + 2.0 * DASH_FRAME_PAD
        );
        assert_eq!(
            panel.height,
            2.0 * DASH_GRID_ROW - 8.0 + 2.0 * DASH_FRAME_PAD
        );
        assert_eq!(panel.x, VP.width - DASH_MARGIN - panel.width);
        assert_eq!(panel.y, DASH_MARGIN);
    }

    /// 网格算术：span2 卡宽 168、span3 卡宽 256；卡高 152（2 行格，
    /// 2026-09-22 裁定回自然高）；卡行 y = panel.y + PAD；x 落 88 节距。
    #[test]
    fn grid_unit_cell_geometry() {
        let faces = vec![face("a", 2), face("b", 3)];
        let (panel, cells) = dashboard_layout(VP, &faces);
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].width, 2.0 * DASH_GRID_COL - 8.0);
        assert_eq!(cells[1].width, 3.0 * DASH_GRID_COL - 8.0);
        // 卡高 = 满框 2 行格（152），原点 = 外框内缩 PAD。
        assert_eq!(cells[0].height, 2.0 * DASH_GRID_ROW - 8.0);
        assert_eq!(cells[0].y, panel.y + DASH_FRAME_PAD);
        assert_eq!(cells[0].x, panel.x + DASH_FRAME_PAD);
        assert_eq!(cells[1].x, panel.x + DASH_FRAME_PAD + 2.0 * DASH_GRID_COL);
    }

    /// 8 列单卡行容量：2+3+3 恰满；第 4 张裁剪（Q2 v1 策略）。
    #[test]
    fn single_widget_row_capacity_and_clip() {
        // PLAN-041 T-08（SD-01）：溢出 → span-3 依序收缩到 2，全容纳不再
        // 裁剪（2+3+3+2=10 → 2+2+2+2=8 恰满）。
        let faces = vec![face("a", 2), face("b", 3), face("c", 3), face("d", 2)];
        let (panel, cells) = dashboard_layout(VP, &faces);
        assert_eq!(cells.len(), 4, "收缩后 4 张全容纳（旧行为裁第 4 张）");
        for c in &cells {
            assert!(
                c.x + c.width <= panel.x + panel.width + 0.5,
                "卡不得越出外框右缘"
            );
        }
        // 收缩只落在 span-3 面（a/d 原宽保持 2 格 = 168px）。
        assert_eq!(cells[0].width as i32, (2 * DASH_GRID_COL as i32) - 8);
        assert_eq!(cells[3].width as i32, (2 * DASH_GRID_COL as i32) - 8);
        // 收缩数恰为溢出差额（2 张）：总宽 = 8 格 - 4×8 gap。
        let total_w: i32 = cells.iter().map(|c| c.width as i32).sum();
        assert_eq!(total_w, 8 * DASH_GRID_COL as i32 - 4 * 8);
    }

    /// span 越界 clamp（0/99 直传不 panic、不越界——存储坏值防御）。
    #[test]
    fn span_clamped_and_frame_intact() {
        let faces = vec![face("x", 99), face("y", 0)];
        let (panel, cells) = dashboard_layout(VP, &faces);
        assert_eq!(cells.len(), 2);
        for c in &cells {
            assert!(c.width <= panel.width);
            assert!(c.x + c.width <= panel.x + panel.width + 0.5);
        }
    }
}

/// 面板布局算式（PLAN-035 T-04 v2——宿主/面板几何单一事实）：外框 =
/// 屏幕右上 8×2 图标网格块（720×176 @ 12px 边距，节距 88/80，含四围
/// PAD）；face 卡 = 视口绝对格位，行主序单卡行 next-fit。卡高 = 满框
/// 2 行格（152）——2026-09-22 裁定退役 T-18 满高 3 行格。
/// PLAN-041 T-08（SD-01）：单行容量自适应——溢出时 span-3 卡依序收缩
/// 到 2（仅本布局生效，不回写声明/存储；2 格高卡折双行会半高/溢框故
/// 弃 wrap 臂），仍容不下才裁剪 + dev 日志。spans 2+3+3+2=9 → 2+2+2+2
/// =8 全容纳（1 face clipped 消失）。
pub(super) fn dashboard_layout(
    viewport: iced::Rectangle,
    faces: &[DashFace],
) -> (iced::Rectangle, Vec<iced::Rectangle>) {
    let grid_w = DASH_COLS as f32 * DASH_GRID_COL - 8.0;
    let grid_h = DASH_ROWS as f32 * DASH_GRID_ROW - 8.0;
    // PLAN-035 T-14：外框 = 网格块四围外扩 PAD（chrome 留白），格位原点
    // 随之内移 PAD——卡片不再贴框缘。
    let panel_w = grid_w + 2.0 * DASH_FRAME_PAD;
    let panel_h = grid_h + 2.0 * DASH_FRAME_PAD;
    let panel_x = (viewport.width - DASH_MARGIN - panel_w).max(DASH_MARGIN);
    let panel_y = DASH_MARGIN;
    let grid_x = panel_x + DASH_FRAME_PAD;
    let grid_y = panel_y + DASH_FRAME_PAD;
    // 容量预算：单行总跨度超 8 列的差额 = 需收缩的 span-3 卡数。
    let total: usize = faces.iter().map(|f| f.span.clamp(2, 3)).sum();
    let mut shrink_left = total.saturating_sub(DASH_COLS);
    let mut cells = Vec::with_capacity(faces.len());
    let mut col = 0usize;
    let mut clipped = 0usize;
    for f in faces {
        let mut want = f.span.clamp(2, 3);
        if shrink_left > 0 && want > 2 {
            want = 2;
            shrink_left -= 1;
        }
        let remain = DASH_COLS - col;
        if want > remain {
            clipped += 1;
            continue;
        }
        cells.push(iced::Rectangle {
            x: grid_x + col as f32 * DASH_GRID_COL,
            y: grid_y,
            width: want as f32 * DASH_GRID_COL - 8.0,
            height: grid_h,
        });
        col += want;
    }
    if clipped > 0 {
        crate::syslog!(
            SyslogLevel::Info, "host",
            "[dashboard] layout: {clipped} face(s) clipped — 单行 {DASH_COLS} 列 span 收缩后仍容不下"
        );
    }
    (
        iced::Rectangle {
            x: panel_x,
            y: panel_y,
            width: panel_w,
            height: panel_h,
        },
        cells,
    )
}

/// `shell.dashboard.enabled`（csv）读回——None = 未配置（首次召唤自动
/// 纳入全部候选，§5.6 默认策略）；Some = 用户显式清单（单一事实）。
fn dashboard_enabled_list() -> Option<Vec<String>> {
    auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.dashboard.enabled").map(|csv| {
        csv.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

/// `shell.dashboard.span.<id>` 读回（"1"|"2"，坏值/缺席 = 1）。
/// 存储覆写面（PLAN-035 SD-02：合法域 {2,3}——"3"→3；"1"/"2"→2 旧值
/// 迁移；坏值/缺席 = None，由消费臂回落声明与缺省）。
fn dashboard_span_from_storage(id: &str) -> Option<usize> {
    auto_lang_core::vm::ffi::stdlib::storage_host_read(&format!("shell.dashboard.span.{id}"))
        .and_then(|v| match v.trim() {
            "3" => Some(3usize),
            "1" | "2" => Some(2usize),
            _ => None,
        })
}

/// PLAN-035 SD-02：app 源 span 声明探测——源内唯一标记
/// `dashboard span: N`（N∈{2,3}；注释形态 `// dashboard span: 3`）。
/// 不锚定 `view mini` 位置：头注可能提前含该字样（012-clock 实测误锚，
/// 首个命中落在头注导致窗口扫不到标记）。文本级契约与 has_mini grep
/// 同族；存储覆写仍胜（refresh 臂消费序）。
fn dashboard_declared_span(state: &crate::ui::session::DesktopSession, id: &str) -> Option<usize> {
    let spec = state.desktop.app_resolver.as_ref().and_then(|r| r(id))?;
    let code = spec.code.as_str();
    const MARKER: &str = "dashboard span:";
    let p = code.find(MARKER)?;
    let tail = code[p + MARKER.len()..].trim_start();
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    match digits.as_str() {
        "3" => Some(3),
        "2" => Some(2),
        _ => None,
    }
}

/// face 候选清单推导（§5.3 两级）：①注册表全量扫 `view mini` 文本探测
/// （grep 级，含未运行——占位卡/孵化判定输入）；②会话确认——有会话者
/// 以 `has_named_view("mini")` 精确生效（文本误报兜底）。产出按注册表序。
fn dashboard_face_candidates(
    state: &mut crate::ui::session::DesktopSession,
) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    for entry in state.desktop.registry_entries.iter() {
        // ①文本探测：resolver 直读源（registry_entries 不携带源码）。
        let has_mini = state
            .desktop
            .app_resolver
            .as_ref()
            .and_then(|r| r(&entry.id))
            .map(|spec| spec.code.contains("view mini"))
            .unwrap_or(false);
        if has_mini {
            out.push((
                entry.id.clone(),
                entry.title.clone(),
                entry.icon.clone(),
                entry.category.clone(),
            ));
        }
    }
    out
}

/// R5：注册表 category → tab（"system" → 系统页，其余 → 小组件页）。
fn dashboard_tab_of_category(category: &str) -> &'static str {
    if category == "system" {
        "system"
    } else {
        "main"
    }
}

/// 注册表 id → 活会话 AppId（有窗运行中：vwin.registry_id 反查）。
pub(super) fn dashboard_running_app(
    state: &crate::ui::session::DesktopSession,
    id: &str,
) -> Option<crate::ui::session::AppId> {
    let host = state.host.as_ref()?;
    for v in host.wm.wins.values() {
        if v.registry_id.as_deref() == Some(id) {
            return Some(v.app);
        }
    }
    None
}

/// R5：孵化 mini 会话的 Tick 门控——face 当前不可见（面板隐藏或所在
/// tab 非活动）→ 停订 .Tick（实时轮询类组件常驻零开销；stella tab 语义
/// ——“平时不看，也就不影响 CPU”）。非孵化 app 恒允许（自己的窗自己
/// 驱动）。
pub(super) fn dashboard_hatched_tick_allowed(
    state: &crate::ui::session::DesktopSession,
    app_id: crate::ui::session::AppId,
) -> bool {
    let Some(host) = state.host.as_ref() else {
        return true;
    };
    if !host.face_fields.contains_key(&app_id.0) {
        return true; // 非孵化会话：正常 app，Tick 恒订。
    }
    // 升格窗在场 → 窗驱动的会话，Tick 恒订（face 垫片仍在——孵化出身
    // 标记不拆；2026-10-10 升格/降格语义配套，否则升格窗随面板隐藏冻结）。
    if host.wm.win_of_app(app_id).is_some() {
        return true;
    }
    // 面板隐藏 → face 不渲染 → 停订。
    if !state.dashboard_visible() {
        return false;
    }
    // 注册表 id 反查 → category → tab；面板活动 tab 匹配才订。
    let Some(panel) = state.desktop.dashboard_app else {
        return false;
    };
    let reg_id = state
        .desktop
        .hatched_minis
        .iter()
        .find(|(_, a)| **a == app_id)
        .map(|(name, _)| name.clone());
    let Some(reg_id) = reg_id else { return false };
    let tab = state
        .desktop
        .registry_entries
        .iter()
        .find(|e| e.id == reg_id)
        .map(|e| dashboard_tab_of_category(&e.category))
        .unwrap_or("main");
    let active_tab = state
        .apps
        .get(&panel)
        .and_then(|a| a.component.read_state("active_tab").ok())
        .and_then(|v| match v {
            auto_val::Value::Str(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "main".to_string());
    tab == active_tab
}

/// view 侧 faces 清单（装配层消费）——只读面板注入的平行快照（零源
/// 扫描/零文件 IO），重构 DashFace 供 [`dashboard_layout`] 同算式复算
/// 格位。面板未挂载/快照空 → 空。
pub(super) fn dashboard_faces_for_view(state: &crate::ui::session::DesktopSession) -> Vec<DashFace> {
    // PLAN-036 T-06（D1 修订）：outproc 轨读宿主缓存（push 时写入）。
    if state.desktop.shell_pipe.is_some() {
        let cache = state.desktop.dashboard_face_cache.borrow();
        return cache
            .iter()
            .map(|(id, title, icon, status, span, tab)| DashFace {
                id: id.clone(),
                title: title.clone(),
                icon: icon.clone(),
                status: match status.as_str() {
                    "running" => "running",
                    "hatched" => "hatched",
                    _ => "placeholder",
                },
                span: *span,
                tab: match tab.as_str() {
                    "system" => "system",
                    _ => "main",
                },
            })
            .collect();
    }
    let Some(panel) = state.desktop.dashboard_app else {
        return Vec::new();
    };
    let Some(app) = state.apps.get(&panel) else {
        return Vec::new();
    };
    // write_state_vec 落 VM 堆（读回 = VmRef 引用）——必须走物化读
    // （read_state_as_vec），Value::Array 直匹配恒空（实机诊断定位）。
    let read_vec = |key: &str| -> Vec<String> {
        app.component
            .read_state_as_vec(key)
            .unwrap_or_default()
            .into_iter()
            .map(|v| match v {
                auto_val::Value::Str(s) => s.to_string(),
                other => format!("{other:?}"),
            })
            .collect()
    };
    let ids = read_vec("face_ids");
    if ids.is_empty() {
        return Vec::new();
    }
    let titles = read_vec("face_titles");
    let icons = read_vec("face_icons");
    let statuses = read_vec("face_statuses");
    let spans = read_vec("face_spans");
    let tabs = read_vec("face_tabs");
    // R5：只渲染面板 .at 活动 tab 的 face（宿主 wrapper 尺寸随活动页
    // 行数变化——dashboard_layout 同算式重算）。
    let active_tab = app
        .component
        .read_state("active_tab")
        .ok()
        .and_then(|v| match v {
            auto_val::Value::Str(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "main".to_string());
    ids.iter()
        .enumerate()
        .filter(|(i, _)| tabs.get(*i).map(|t| t == &active_tab).unwrap_or(false))
        .map(|(i, id)| DashFace {
            id: id.clone(),
            title: titles.get(i).cloned().unwrap_or_else(|| id.clone()),
            icon: icons.get(i).cloned().unwrap_or_else(|| "app-window".into()),
            status: match statuses.get(i).map(|s| s.as_str()) {
                Some("running") => "running",
                Some("hatched") => "hatched",
                _ => "placeholder",
            },
            span: spans
                .get(i)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(2),
            tab: match tabs.get(i).map(|t| t.as_str()) {
                Some("system") => "system",
                _ => "main",
            },
        })
        .collect()
}

/// PLAN-036 T-05：faces 收集 + 面板几何（两轨同源——配置门/静默孵化
/// 副作用/存储跨度覆写/dashboard_layout 算式；返回 (faces, w, h, top)）。
fn collect_dashboard_faces(
    state: &mut crate::ui::session::DesktopSession,
) -> (Vec<DashFace>, f32, f32, f32) {
    let candidates = dashboard_face_candidates(state);
    let enabled_cfg = dashboard_enabled_list();
    let mut faces: Vec<DashFace> = Vec::new();
    for (id, title, icon, category) in candidates {
        // 配置门：显式清单单一事实；未配置 = 全纳入（默认策略）。
        if let Some(list) = &enabled_cfg {
            if !list.iter().any(|s| s == &id) {
                continue;
            }
        }
        let status = if dashboard_running_app(state, &id).is_some() {
            "running"
        } else if state.hatched_mini_of(&id).is_some() {
            "hatched"
        } else {
            // ②静默孵化臂（2026-10-10 用户裁定 F-R3 翻案）：后端 app 随孵化
            // 同链供给（hatch_mini_app 内 mirror launch 供给链）——组件与
            // 窗解耦独立工作；仅 exe 形态与孵化失败落占位卡。
            match state.hatch_mini_app(&id) {
                Ok(Some(app_id)) => {
                    state.register_hatched_mini(&id, app_id);
                    "hatched"
                }
                // PLAN-035：孵化失败不再静默——错误串落日志（可诊断性）。
                Err(err) => {
                    crate::syslog!(
                        SyslogLevel::Error,
                        "host",
                        "[dashboard] hatch {id} failed: {err}"
                    );
                    "placeholder"
                }
                _ => "placeholder",
            }
        };
        faces.push(DashFace {
            id,
            title,
            icon,
            status,
            span: 0,
            tab: dashboard_tab_of_category(&category),
        });
    }
    // PLAN-035 SD-02：存储覆写 → 源声明 → 缺省 2（旧 "1" 迁移在存储面）。
    for f in faces.iter_mut() {
        f.span = dashboard_span_from_storage(&f.id)
            .or_else(|| dashboard_declared_span(state, &f.id))
            .unwrap_or(2);
    }
    let viewport = state.host_viewport();
    let (panel_rect, _cells) = dashboard_layout(viewport, &faces);
    (faces, panel_rect.width, panel_rect.height, panel_rect.y)
}

/// dashboard 面板 faces 快照注入（召唤 + 打开期间活更新同体）：
/// 孵化臂（无会话 + 无后端 → windowless mini 会话）+ 平行列表注入 +
/// `__dashboard_faces` 合同面 + 几何 px 注入 + RebuildFaces。
fn refresh_dashboard_panel(state: &mut crate::ui::session::DesktopSession) {
    // PLAN-036 T-05（B2）：outproc 轨——faces 收集（含静默孵化副作用，
    /// 宿主行为两轨同源）+ 快照重推（RebuildFaces 事件重建 rows）。
    if state.desktop.shell_pipe.is_some() {
        if state.desktop.dashboard_open {
            push_dashboard_snapshot(
                state,
                &[crate::ui::shell_projection::ShellEvent::RebuildFaces],
            );
        }
        return;
    }
    let Some(panel) = state.desktop.dashboard_app else {
        return;
    };
    if !state.dashboard_visible() {
        return;
    }
    // PLAN-036 T-05：faces 收集 + 几何提炼为 collect_dashboard_faces
    ///（in-proc 注入与 outproc 快照两轨同源——孵化副作用/配置门/跨度/
    /// 布局算式单一事实）。
    let (faces, panel_w, panel_h, panel_top) = collect_dashboard_faces(state);

    // 3. 快照注入（平行列表 + 合同面 + 几何）。
    let mut ids: Vec<auto_val::Value> = Vec::new();
    let mut titles: Vec<auto_val::Value> = Vec::new();
    let mut icons: Vec<auto_val::Value> = Vec::new();
    let mut statuses: Vec<auto_val::Value> = Vec::new();
    let mut spans: Vec<auto_val::Value> = Vec::new();
    let mut tabs: Vec<auto_val::Value> = Vec::new();
    let mut objs: Vec<auto_val::Value> = Vec::new();
    for f in &faces {
        ids.push(auto_val::Value::Str(f.id.clone().into()));
        titles.push(auto_val::Value::Str(f.title.clone().into()));
        icons.push(auto_val::Value::Str(f.icon.clone().into()));
        statuses.push(auto_val::Value::Str(f.status.into()));
        spans.push(auto_val::Value::Str(f.span.to_string().into()));
        tabs.push(auto_val::Value::Str(f.tab.into()));
        objs.push(auto_val::Value::Str(f.id.clone().into()));
    }
    eprintln!(
        "[dashboard] refresh: faces={} statuses={:?} panel={panel_w}x{panel_h}@{panel_top}",
        faces.len(),
        faces
            .iter()
            .map(|f| (f.id.as_str(), f.status))
            .collect::<Vec<_>>(),
    );
    if let Some(app) = state.apps.get_mut(&panel) {
        // PLAN-041 T-15：挂载兜底——懒挂载可能晚于 dark_cache 收敛，
        // refresh（召唤后必经）无条件回写当前暗色位（值同写亦幂等）。
        let _ = app.component.write_state(
            "__dash_dark",
            auto_val::Value::str(if state.desktop.dash_dark_cache.get() {
                "1"
            } else {
                "0"
            }),
        );
        let _ = app.component.write_state_vec("face_ids", ids);
        let _ = app.component.write_state_vec("face_titles", titles);
        let _ = app.component.write_state_vec("face_icons", icons);
        let _ = app.component.write_state_vec("face_statuses", statuses);
        let _ = app.component.write_state_vec("face_spans", spans);
        let _ = app.component.write_state_vec("face_tabs", tabs);
        let _ = app.component.write_state(
            "__dashboard_faces",
            auto_val::Value::Array(auto_val::Array::from(objs)),
        );
        let _ = app
            .component
            .write_state("__panel_w", auto_val::Value::Int(panel_w as i32));
        let _ = app
            .component
            .write_state("__panel_h", auto_val::Value::Int(panel_h as i32));
        let _ = app
            .component
            .write_state("__panel_top", auto_val::Value::Int(panel_top as i32));
        if let Err(err) = app.component.bridge_mut().call_handler("RebuildFaces", &[]) {
            crate::syslog!(
                SyslogLevel::Error,
                "host",
                "[session] dashboard RebuildFaces failed: {err}"
            );
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
}

/// PLAN-024：dashboard 面板开合执行体（dock 钮 / Esc / × / scrim 同落；
/// 二态翻转——可见再召唤即关）。懒挂载：首次召唤编译装载 dashboard.at
/// （switcher/通知同型），召唤即 faces 快照注入（含静默孵化臂）。
pub(super) fn toggle_dashboard(
    state: &mut crate::ui::session::DesktopSession,
) -> iced::Task<crate::ui::session::DesktopMessage> {
    // PLAN-036 T-05（B2）：outproc 轨推送臂——toggle 语义镜像（开 =
    /// visible 置位 + RebuildFaces 事件快照；关 = visible=0 纯状态）；
    /// in-proc 原路径零变化（I2）。
    if state.desktop.shell_pipe.is_some() {
        if state.desktop.dashboard_open {
            state.desktop.dashboard_open = false;
            push_dashboard_snapshot(state, &[]);
        } else {
            state.desktop.dashboard_open = true;
            push_dashboard_snapshot(
                state,
                &[crate::ui::shell_projection::ShellEvent::RebuildFaces],
            );
        }
        return iced::Task::none();
    }
    // 1. 懒挂载
    if state.desktop.dashboard_app.is_none() {
        match crate::ui::shell::build_dashboard_component() {
            Ok(comp) => {
                let app_id = state.allocate_app(comp);
                state.desktop.dashboard_app = Some(app_id);
            }
            Err(err) => {
                push_notification(state, "error", &format!("dashboard 装载失败: {err}"));
                return iced::Task::none();
            }
        }
    }
    // 2. toggle：可见 → 自隐。
    if state.dashboard_visible() {
        close_dashboard(state);
        return iced::Task::none();
    }
    // 3. 打开：visible 置位 + faces 快照注入（孵化/几何/平行列表）。
    let panel = state.desktop.dashboard_app.expect("dashboard mounted");
    if let Some(app) = state.apps.get_mut(&panel) {
        let _ = app
            .component
            .write_state("visible", auto_val::Value::str("1"));
        *app.state.view_dirty.borrow_mut() = true;
    }
    refresh_dashboard_panel(state);
    iced::Task::none()
}

/// PLAN-024：dashboard 面板关闭执行体（× / scrim / Esc / 再点 dock 钮
/// 四路径同一入口；孵化会话常驻不回收——notification 槽先例）。
fn close_dashboard(state: &mut crate::ui::session::DesktopSession) {
    // PLAN-036 T-05（B2）：outproc 轨——镜像位复位 + visible=0 快照。
    if state.desktop.shell_pipe.is_some() {
        state.desktop.dashboard_open = false;
        push_dashboard_snapshot(state, &[]);
        return;
    }
    if let Some(panel) = state.desktop.dashboard_app {
        if let Some(app) = state.apps.get_mut(&panel) {
            let _ = app
                .component
                .write_state("visible", auto_val::Value::str("0"));
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
}

/// PLAN-024 T-05：面板编辑——纳入/移除（storage `shell.dashboard.enabled`
/// csv 直写 + 面板活刷新）。首次显式编辑把「未配置」升级为「显式清单」
/// （默认全纳入 → 以编辑后清单为单一事实）。
fn dashboard_set_pinned(state: &mut crate::ui::session::DesktopSession, id: &str, pin: bool) {
    let mut list = dashboard_enabled_list().unwrap_or_default();
    // 未配置态的首次编辑：以当前候选清单为底（全纳入），再做增量。
    if !dashboard_enabled_list_from_storage() {
        list = dashboard_face_candidates(state)
            .into_iter()
            .map(|(id, _, _, _)| id)
            .collect();
    }
    if pin {
        if !list.iter().any(|s| s == id) {
            list.push(id.to_string());
        }
    } else {
        list.retain(|s| s != id);
    }
    auto_lang_core::vm::ffi::stdlib::storage_host_publish(
        "shell.dashboard.enabled",
        list.join(","),
    );
    refresh_dashboard_panel(state);
}

/// `shell.dashboard.enabled` 是否已在 storage 显式配置。
fn dashboard_enabled_list_from_storage() -> bool {
    auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.dashboard.enabled").is_some()
}

/// PLAN-024 T-05：面板编辑——列跨度（storage `shell.dashboard.span.<id>`
/// 直写 + 面板活刷新）。
fn dashboard_set_span(state: &mut crate::ui::session::DesktopSession, id: &str, span: u8) {
    let span = span.clamp(1, 2);
    auto_lang_core::vm::ffi::stdlib::storage_host_publish(
        &format!("shell.dashboard.span.{id}"),
        span.to_string(),
    );
    refresh_dashboard_panel(state);
}

/// Plan 551：⚙️/open_settings 的 launch-or-focus 靶 = os-config 统一设置
/// 中心（501 相邻仓扫描 id `os-config`，pac `daemon: autoos` → launch 臂
/// ensure daemon）。540 的 045-desktop-settings 设置窗随方向纠正退役
/// （T7 删目录）；配置读写单源走 daemon（config.at），宿主经 T6 轮询热应用。
pub(crate) const OSCONFIG_APP_ID: &str = "os-config";

/// Plan 540 T7 → Plan 551 换向：设置召唤执行体——launch-or-focus
/// （`DC::ActivateApp` 同构：已开窗跨分区聚焦，未开 launch）。设置 =
/// os-config 普通 registry 窗（×关闭/拖拽/缩放/z 序全继承 WM）。
pub(super) fn execute_open_settings(state: &mut crate::ui::session::DesktopSession) {
    if let Some(host) = state.host.as_ref() {
        if let Some((wid, ws)) = host
            .wm
            .wins
            .iter()
            .find(|(_, v)| v.registry_id.as_deref() == Some(OSCONFIG_APP_ID))
            .map(|(wid, v)| (*wid, v.workspace))
        {
            // PLAN-012 F2（用户裁定）：二态语义补全——窗口在场且可见时
            // 再拨 = 关闭（走 W1 close→hide 拦截臂，重开瞬时）；隐藏/异
            // 分区才聚焦拉起。
            let visible_and_here = host.wm.current_workspace == ws
                && host
                    .wm
                    .wins
                    .get(&wid)
                    .map(|v| !v.hidden.get() && !v.minimized.get())
                    .unwrap_or(false);
            if visible_and_here {
                let _ = execute_desktop_commands(
                    state,
                    vec![crate::ui::session::DesktopCommand::CloseWindow(wid)],
                );
                return;
            }
            let current = state
                .host
                .as_ref()
                .map(|h| h.wm.current_workspace)
                .unwrap_or(0);
            if ws != current {
                state.wm_set_workspace(ws);
            }
            state.wm_focus(wid);
            return;
        }
    }
    execute_launch_app(state, OSCONFIG_APP_ID);
}
/// PLAN-526 T14：壁纸目录扫描（jpg/png 枚举 → {name,path,src} Obj 数组）。
/// 键缺席/非目录/空目录 = 空表（面板显示引导文案）。load_desktop_id_list
/// 同型的宿主派生面（I9）。注：.at 侧自 2026-08-22 起已有 fs.read_dir/fs.walk/fs.tree（2866/2860/2875），此处宿主侧枚举系历史实现，非能力缺失。
fn scan_wallpapers_dir(cfg: &crate::ui::desktop_config::DesktopConfig) -> Vec<auto_val::Value> {
    let Some(dir) = wallpapers_dir_or_default(cfg) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| {
                        let e = e.to_ascii_lowercase();
                        e == "jpg" || e == "jpeg" || e == "png"
                    })
                    .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let path = p.to_string_lossy().to_string();
            let name = p
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                ("name", auto_val::Value::Str(name.into())),
                ("path", auto_val::Value::Str(path.clone().into())),
                ("src", auto_val::Value::Str(path.into())),
            ])))
        })
        .collect()
}

/// Plan 487 M4：`set_dock_position` 执行臂（I7：几何是驱动事实）。写回
/// storage 键（boot 读路径同键，保证一致）→ 从键重推导 dock_edges（I9
/// 单一事实：boot `desktop_dock_edges` 同函数）→ apply_layout relayout +
/// 槽位几何排水 + shell.at 投影热同步。
fn execute_set_dock_position(state: &mut crate::ui::session::DesktopSession, top: bool) {
    // Plan 540 T3：写通道收口——单源 config 落盘（旧 storage 键写退役）。
    state.desktop.config.dock_position = if top { "top" } else { "bottom" }.to_string();
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    apply_dock_edges_now(state);
}

/// Plan 487 M4：`set_dock_enabled` 执行臂（同上；false = 零预留，位置键
/// 保留——重开时按原位置恢复）。
fn execute_set_dock_enabled(state: &mut crate::ui::session::DesktopSession, on: bool) {
    // Plan 540 T3：写通道收口——位置语义保留在 config.dock_position
    //（关再开按原位置恢复）。
    state.desktop.config.dock_enabled = on;
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    apply_dock_edges_now(state);
}

/// Plan 487 M4：dock 预留边热应用——键重推导 + relayout（wm_set_layout
/// 同型 apply_layout 全窗写回）+ 原生槽位几何排水（SetLayout 热键臂同款）
/// + shell.at 投影热同步（Init 只跑一次，`__dock_*` 状态变量直写 +
/// view_dirty）。
fn apply_dock_edges_now(state: &mut crate::ui::session::DesktopSession) {
    // Plan 497 G3：dock 位置/开关热切换 = 全场重排（487 I7 三联动之一），
    // 快照随撤（裁剪区域全部 stale）。
    crate::ui::iced::snapshot::invalidate_all();
    state.desktop.dock_edges = desktop_dock_edges(&state.desktop.config);
    let viewport = state.host_viewport();
    let edges = state.desktop.dock_edges;
    if let Some(host) = state.host.as_mut() {
        crate::ui::layout::apply_layout(&mut host.wm, viewport, edges);
    }
    sync_native_geometry(state);
    let pos = if edges.top > 0.0 { "top" } else { "bottom" };
    let en = if edges == crate::ui::layout::ReservedEdges::default() {
        "0"
    } else {
        "1"
    };
    if let Some(shell) = state.desktop.shell_app {
        if let Some(app) = state.apps.get_mut(&shell) {
            let _ = app
                .component
                .write_state("__dock_position", auto_val::Value::str(pos));
            // Plan 505 B1：边线类投影（top→border-b）——容器 if 样式在
            // 实机装配层不稳，边线数据驱动（与 __dock_position 同点热同步）。
            let border = if pos == "top" { "border-b" } else { "border-t" };
            let _ = app
                .component
                .write_state("__dock_border", auto_val::Value::str(border));
            let _ = app
                .component
                .write_state("__dock_enabled", auto_val::Value::str(en));
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
}

/// Plan 464 T4：shell + launcher 的 DesktopBus 联合排空与执行。
/// 返回 (是否退出进程, 召唤产生的任务)；调用方负责 batch 回 iced。
pub(super) fn drain_and_execute_desktop_commands(
    state: &mut crate::ui::session::DesktopSession,
) -> (bool, Vec<iced::Task<crate::ui::session::DesktopMessage>>) {
    let mut cmds = state.drain_desktop_commands();
    if let Some(la) = state.desktop.launcher_app {
        cmds.extend(state.drain_app_desktop_commands(la));
    }
    // Plan 478 T4：switcher 上行（focus 确认记录）同管线联合排空。
    if let Some(sw) = state.desktop.switcher_app {
        cmds.extend(state.drain_app_desktop_commands(sw));
    }
    // Plan 479 T3：通知中心上行（notes_dismiss/notes_clear）联合排空。
    if let Some(panel) = state.desktop.notification_app {
        cmds.extend(state.drain_app_desktop_commands(panel));
    }
    // Plan 540 T7：设置窗（registry App）上行联合排空——按 registry_id
    // 定位（普通窗可关可重开，不落 overlay 槽位）。
    let settings_window_app = state.host.as_ref().and_then(|h| {
        h.wm.wins
            .iter()
            .find(|(_, v)| v.registry_id.as_deref() == Some(OSCONFIG_APP_ID))
            .map(|(_, v)| v.app)
    });
    if let Some(app) = settings_window_app {
        cmds.extend(state.drain_app_desktop_commands(app));
    }
    // PLAN-016 T-07（协议 v1.7）：普通注册表窗上行联合排空——open_with
    // 动词面（027 文件管理器）。特权窗上方已排空（二次排空空读无害）；
    // 动词面仍受 DesktopCommand 解析白名单约束。
    // PLAN-014 W-08（协议 v1.8）：注册表窗段**按 app 分段收集**——段随
    // 携带发件方 registry_id，执行期置 `notify_source` 供 notify 落库来源
    // 归因。分段执行序与原扁平 concat 逐一相同（drain 序即执行序），
    // Shutdown 短路语义按段传递（特权段退出即跳过后续段）。
    let mut segments: Vec<(Option<String>, Vec<crate::ui::session::DesktopCommand>)> = Vec::new();
    {
        let app_ids: Vec<_> = state
            .host
            .as_ref()
            .map(|h| {
                h.wm.wins
                    .values()
                    .map(|v| (v.app, v.registry_id.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for (app_id, registry_id) in app_ids {
            let seg = state.drain_app_desktop_commands(app_id);
            if !seg.is_empty() {
                segments.push((registry_id, seg));
            }
        }
        // PLAN-035 T-07（SD-03）：dashboard 孵化会话段——windowless mini
        // 会话不在 wm.wins，其 __desktop_cmd（notify 词面）此前不可达；
        // 并入联合排空，registry_id = 孵化映射键（归因沿用）。词表白名单
        // 不变（DesktopCommand parse 门）。
        let hatch: Vec<_> = state
            .desktop
            .hatched_minis
            .iter()
            .map(|(rid, aid)| (rid.clone(), *aid))
            .collect();
        for (registry_id, app_id) in hatch {
            let seg = state.drain_app_desktop_commands(app_id);
            if !seg.is_empty() {
                segments.push((Some(registry_id), seg));
            }
        }
    }
    if cmds.is_empty() && segments.is_empty() {
        return (false, Vec::new());
    }
    if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
        let seg_n: usize = segments.iter().map(|(_, s)| s.len()).sum();
        eprintln!("[464-DRAIN] {} commands", cmds.len() + seg_n);
    }
    let (mut exit, mut tasks) = execute_desktop_commands(state, cmds);
    if !exit {
        for (registry_id, seg) in segments {
            *state.desktop.notify_source.borrow_mut() = registry_id;
            let (seg_exit, seg_tasks) = execute_desktop_commands(state, seg);
            *state.desktop.notify_source.borrow_mut() = None;
            tasks.extend(seg_tasks);
            if seg_exit {
                exit = true;
                break;
            }
        }
    }
    if !exit {
        // Plan 473 T6：命令驱动的 relayout（布局切换/关闭/launch）即刻
        // 排水槽位几何，不等 400ms 帧泵。
        sync_native_geometry(state);
    }
    (exit, tasks)
}

/// Plan 464 T4：执行桌面命令（返回 (是否退出进程, 产生的任务)）。
/// SummonLauncher 产生的任务（聚焦）由调用方 batch 回 iced。
pub(super) fn execute_desktop_commands(
    state: &mut crate::ui::session::DesktopSession,
    cmds: Vec<crate::ui::session::DesktopCommand>,
) -> (bool, Vec<iced::Task<crate::ui::session::DesktopMessage>>) {
    use crate::ui::session::DesktopCommand as DC;
    let mut tasks: Vec<iced::Task<crate::ui::session::DesktopMessage>> = Vec::new();
    for cmd in cmds {
        match cmd {
            DC::SummonLauncher => {
                if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
                    eprintln!("[464-SUMMON] bus summon command");
                }
                tasks.push(summon_launcher(state));
            }
            DC::LaunchApp(name) => execute_launch_app(state, &name),
            // PLAN-016 T-07（协议 v1.7）：open_with 执行臂。
            DC::OpenWith(app, path) => execute_open_with(state, &app, &path),
            DC::CloseWindow(wid) => {
                // PLAN-012 W1：os-config close→hide 拦截臂——设置窗"×"不改
                // 换成常驻隐藏（hidden 置位：投影/命中/推层全排除，"真关了"
                // 的感知），组件与编译产物保留 → 重开走 execute_open_settings
                // 聚焦臂（unhide，零编译零探活）。Q1 定案：方案 a（真 hidden
                // 字段）——win_min 形态任务栏保留条目且 __wm_running 持续含
                // os-config，W7 齿轮高亮语义断裂（AC-12）。
                let is_osconfig = state.host.as_ref().is_some_and(|h| {
                    h.wm.wins.get(&wid).and_then(|v| v.registry_id.as_deref())
                        == Some(OSCONFIG_APP_ID)
                });
                if is_osconfig {
                    if let Some(host) = state.host.as_mut() {
                        if let Some(v) = host.wm.wins.get(&wid) {
                            v.hidden.set(true);
                        }
                        if host.wm.focused == Some(wid) {
                            // 焦点让渡：分区栈顶下一窗（同 hit 语义找非隐藏窗）。
                            let next = host.wm.z_order.iter().rev().copied().find(|w| {
                                *w != wid
                                    && host
                                        .wm
                                        .wins
                                        .get(w)
                                        .is_some_and(|v| !v.minimized.get() && !v.hidden.get())
                            });
                            host.wm.focused = next;
                        }
                    }
                    crate::ui::iced::snapshot::invalidate(wid);
                    return (false, tasks);
                }
                // Plan 497 G3：关闭失效（窗口回收，缩略随撤）。
                crate::ui::iced::snapshot::invalidate(wid);
                state.broker_close_window(wid);
                if let Some(app) = state.wm_remove_win(wid) {
                    state.apps.remove(&app);
                    // PLAN-037 T-04：后端供给随窗释放（计数归零即卸载）。
                    state.release_backend(app);
                }
                if state.desktop.shell_app.is_none()
                    && state
                        .host
                        .as_ref()
                        .map(|h| h.wm.wins.is_empty())
                        .unwrap_or(false)
                {
                    return (true, Vec::new());
                }
            }
            DC::FocusWindow(wid) => state.wm_focus(wid),
            // PLAN-526 T8：dock 右键菜单最小化（win_min → 同款状态机）。
            DC::MinWindow(wid) => state.wm_minimize(wid),
            // PLAN-526 T11：电源键确认退出（Escape 热键已删后的唯一图形
            // 退出入口；ExitDesktop 同体——恢复槽位，退出由调用点收尾）。
            DC::Shutdown => {
                restore_all_native_slots(state);
                return (true, Vec::new());
            }
            // PLAN-526 T14：壁纸热切换（storage 持久 + 会话字段回读 +
            // 全场快照撤——壁纸层每帧重建读字段即生效，SetTheme 同链）。
            DC::SetWallpaper(path) => execute_set_wallpaper(state, &path),
            // Plan 540 T3：配置写动词族（单源 config.at 收口）。
            DC::SetTransparency(level) => execute_set_transparency(state, &level),
            DC::SetNotesEnabled(on) => execute_set_notes_enabled(state, on),
            DC::SetDockPinned(csv) => execute_set_dock_pinned(state, &csv),
            // PLAN-012 W4 协议 v1.6：单枚固定/取消固定。
            DC::DockPin(id) => execute_dock_pin(state, &id, true),
            DC::DockUnpin(id) => execute_dock_pin(state, &id, false),
            // PLAN-012 W5：拖拽落子后的格子重注入（storage 已由 shell 写）。
            DC::RefreshDesktopIcons => inject_desktop_surface(state),
            // PLAN-012 F2 走查：拖拽落格双动词。
            DC::DesktopIconDrop(dragged, target) => {
                execute_desktop_icon_drop(state, &dragged, target)
            }
            DC::DesktopIconDropAt(dragged, xy) => {
                execute_desktop_icon_drop_at(state, &dragged, &xy)
            }
            // PLAN-012 F2 走查：拖拽开始（宿主置位；__mouse_released 臂落格）。
            // 2026-09-15：附拾起起点光标 + drag_icon（拖拽幽灵坐标锚
            // popover 数据面）；移动超阈值前两路落格动词都拒落（点击语义）。
            DC::DesktopIconDragStart(id) => {
                let origin = state
                    .host
                    .as_ref()
                    .map(|h| {
                        let p = h.wm.last_cursor.get();
                        (p.x, p.y)
                    })
                    .unwrap_or((f32::NEG_INFINITY, f32::NEG_INFINITY));
                if let Some(surface) = state.desktop.desktop_app {
                    if let Some(app) = state.apps.get_mut(&surface) {
                        let icon = state
                            .desktop
                            .registry_entries
                            .iter()
                            .find(|e| e.id == *id)
                            .map(|e| e.icon.clone())
                            .unwrap_or_else(|| "app-window".to_string());
                        let _ = app
                            .component
                            .write_state("drag_icon", auto_val::Value::Str(icon.into()));
                        let _ = app
                            .component
                            .write_state("drop_c", auto_val::Value::str(""));
                        let _ = app
                            .component
                            .write_state("drop_r", auto_val::Value::str(""));
                        let _ = app
                            .component
                            .write_state("drag_moved", auto_val::Value::str(""));
                        *app.state.view_dirty.borrow_mut() = true;
                    }
                }
                state.desktop.icon_drag = Some((id, origin));
                state.desktop.icon_drag_moved = false;
            }
            DC::SetWallpapersDir(dir) => execute_set_wallpapers_dir(state, &dir),
            // PLAN-019 v1.7：显示桌面（负一屏）/ 返回 origin（sliver toggle
            // 双臂；wallpaper_pick/close/browse_dir 组合臂见 T-03 同族）。
            DC::ShowDesktop => execute_show_desktop(state),
            DC::ShowdeskReturn => execute_showdesk_return(state),
            // PLAN-019 v1.7：壁纸 picker 组合臂（更换壁纸 = 借负一屏 +
            // 开 picker + 归属规则单点；关闭按簿记决定是否自动返回）。
            DC::WallpaperPick => execute_wallpaper_pick(state),
            DC::WallpaperClose => execute_wallpaper_close(state),
            DC::WallpaperBrowseDir => execute_wallpaper_browse_dir(state),
            DC::WallpaperNav(dir) => execute_wallpaper_nav(state, &dir),
            DC::WallpaperPreview(path) => execute_wallpaper_preview(state, &path),
            // Plan 497 G3：重排失效（几何全变——裁剪区域随 VWinState.rect
            // stale）。
            DC::SetLayout(mode) => {
                crate::ui::iced::snapshot::invalidate_all();
                state.wm_set_layout(mode)
            }
            // PLAN-526 T33：预设 icon 切换键——同预设再按 = 恢复应用前的
            // 手动排布快照；否则按普通预设应用（快照在 Free→预设迁移时采集）。
            DC::TogglePresetLayout(mode) => {
                crate::ui::iced::snapshot::invalidate_all();
                if state.wm_layout_mode() == mode {
                    state.wm_restore_layout_snapshot();
                } else {
                    state.wm_set_layout(mode);
                }
            }
            // Plan 472 T2：分区切换（dock 切换条/workspace_next；调用臂尾
            // sync_shell_windows 刷新投影）。
            // PLAN-041 T-04：切完即推呈现三面（预览重发布 + shell 重建 +
            // 可见窗补抓）——状态/投影在 WmState 臂已切，呈现半边不再等
            // 帧泵兜底（2026-09-22 实机：Desktop 2 上残影上一分区窗）。
            DC::SetWorkspace(n) => {
                state.wm_set_workspace(n);
                after_workspace_switch(state);
            }
            DC::NextWorkspace => {
                state.wm_next_workspace();
                after_workspace_switch(state);
            }
            // Plan 472 T4：dock 固定图标点击——运行中 →（隐藏分区先切分区）
            // 聚焦其窗；未运行 → launch（与 LaunchApp 臂同执行体）。
            DC::ActivateApp(name) => {
                // PLAN-019 v1.7：负一屏上激活 = 先回 origin 再启动/聚焦
                // （负一屏保留空分区——新窗开在这里即穿帮；返回后既有
                // 「切到窗所在分区」语义照常工作）。
                if state.host.as_ref().is_some_and(|h| h.wm.on_showdesk()) {
                    execute_showdesk_return(state);
                }
                let target = state.host.as_ref().and_then(|h| {
                    // PLAN-040 F2a（2026-09-22）：可见窗优先——多实例同
                    // app 时把最小化/隐藏窗留在兜底位，单击优先聚焦在场
                    // 实例；全最小化/隐藏 = 首个命中（focus 自带取消最
                    // 小化/取消隐藏，单击恢复语义不变）。
                    let mut fallback: Option<(crate::ui::session::Wid, usize)> = None;
                    let mut visible: Option<(crate::ui::session::Wid, usize)> = None;
                    for (wid, v) in h.wm.wins.iter() {
                        if v.registry_id.as_deref() != Some(name.as_str()) {
                            continue;
                        }
                        let entry = (*wid, v.workspace);
                        if !v.minimized.get() && !v.hidden.get() && visible.is_none() {
                            visible = Some(entry);
                        }
                        if fallback.is_none() {
                            fallback = Some(entry);
                        }
                    }
                    visible.or(fallback)
                });
                match target {
                    Some((wid, ws)) => {
                        let current = state
                            .host
                            .as_ref()
                            .map(|h| h.wm.current_workspace)
                            .unwrap_or(0);
                        if ws != current {
                            state.wm_set_workspace(ws);
                        }
                        state.wm_focus(wid);
                    }
                    None => {
                        execute_launch_app(state, &name);
                    }
                }
            }
            // Plan 473 T4：原生窗口收编/解除（native dock Phase 1；执行体见下）。
            DC::DockNative(target) => execute_dock_native(state, target),
            DC::UndockNative(slot) => execute_undock_native(state, slot),
            // Plan 486 v1.3：任务栏 native 条目聚焦/关闭（执行体见下）。
            DC::FocusNative(slot) => execute_focus_native(state, slot),
            DC::CloseNative(slot) => execute_close_native(state, slot),
            // Plan 478 T2：pager `+` —— 新增分区并即入（验收②）。
            DC::WorkspaceAdd => {
                let id = state.host.as_mut().map(|h| h.wm.add_workspace());
                if let Some(id) = id {
                    state.wm_set_workspace(id);
                }
            }
            // Plan 478 T2：pager `×` —— 宿主策略门（T1 施工图 §3，待澄清①
            // 定案取 toast 最少意外）：非空分区不删仅提示；末分区 no-op 提示。
            DC::WorkspaceClose(n) => {
                // PLAN-019 v1.7：负一屏不可删（静默 no-op——pager 不投影
                // 该分区，正常路径到不了这里；守卫避免误报保底 toast）。
                let is_showdesk = state
                    .host
                    .as_ref()
                    .map(|h| h.wm.showdesk_ws == Some(n))
                    .unwrap_or(false);
                let has_wins = state
                    .host
                    .as_ref()
                    .map(|h| h.wm.wins.values().any(|v| v.workspace == n))
                    .unwrap_or(false);
                if is_showdesk {
                    // no-op
                } else if has_wins {
                    push_notification(
                        state,
                        "error",
                        &format!("分区 {} 含窗口，请先移动或关闭", n + 1),
                    );
                } else {
                    let before = state.host.as_ref().map(|h| h.wm.workspaces.len());
                    state.wm_remove_workspace(n);
                    let after = state.host.as_ref().map(|h| h.wm.workspaces.len());
                    if before.is_some() && before == after {
                        push_notification(state, "error", "至少保留一个分区");
                    }
                }
            }
            // Plan 478 T2：send_to 动词（跨分区发送；窗口随分区隐现）。
            // PLAN-709：槽位臂并入（伪 Wid 域统一入口；槽位隐现随
            // sync_native_workspace_visibility 排水拍）。
            DC::SendTo(wid, n) => {
                state.wm_move_member_to_workspace(wid, n);
                sync_native_geometry(state);
            }
            // PLAN-709 rev2：win_rect 执行臂——`VWinState.rect` 直写（移动+
            // 缩放一体），160/120 min 钳制（apply_cursor 同款），window_size
            // 同步；无 relayout/不动他窗/不夺焦点/不改 workspace。伪 Wid 段
            // 与未知 wid no-op（槽位几何归交互状态机）。本拍 update 返回即
            // 重绘 = 立即生效；铺排布局下下次 relayout 重铺（铺排主权），
            // Free = 即落定。
            DC::WinRect { wid, x, y, w, h } => {
                if let Some(host) = state.host.as_mut() {
                    if crate::ui::session::WmState::native_slot_id_of_pseudo(wid).is_none() {
                        if let Some(v) = host.wm.wins.get(&wid) {
                            let (old_w, old_h) = {
                                let r = v.rect.borrow();
                                (r.width, r.height)
                            };
                            let mut r = v.rect.borrow_mut();
                            r.x = x;
                            r.y = y;
                            r.width = w.max(160.0);
                            r.height = h.max(120.0);
                            let height = r.height;
                            let width = r.width;
                            drop(r);
                            if let Some(v) = host.wm.wins.get(&wid) {
                                *v.window_size.borrow_mut() = iced::Size::new(width, height);
                            }
                            // PLAN-712 r3 T-18（方案 A）：尺寸真变才标视图脏
                            // ——VM builder 按 window_size 构建期烤定响应式
                            // 取值 + dynamic_view 缓存快门，不标脏则 app 内容
                            // 恒停旧尺寸（表面 1:1 锚定 + 空白右/下）。纯移动
                            // 不标（零重排纪律）。
                            if (width - old_w).abs() >= 0.5 || (height - old_h).abs() >= 0.5 {
                                state.mark_vwin_resized_dirty(wid);
                            }
                        }
                    }
                }
            }
            // Plan 479 T2：notify 动词（App 主动请求通知；入史 + 未读 +
            // 浮现三联动，push_notification 单入口）。
            DC::Notify(kind, msg) => push_notification(state, &kind, &msg),
            // PLAN-042 T-04：log 动词执行臂——入宿主 syslog 环。source =
            // notify_source（联合排空泵注册表分段执行期置位的发件方
            // registry_id；特权面批量段 None → "privileged"）。
            DC::Syslog(level, text) => {
                let src = state
                    .desktop
                    .notify_source
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| "privileged".to_string());
                crate::ui::syslog::push(level, &src, text);
            }
            // Plan 479 T2：notes_clear / notes_dismiss（面板「全部清除」/
            // 「逐条 ×」）——历史变更 + 落盘 + 面板开着则重注入。未读不动
            // （两按钮只在面板内可达，此刻未读恒 0）。
            DC::NotesClear => {
                state.desktop.notifications.borrow_mut().clear();
                persist_notes(state);
                refresh_notification_panel(state);
            }
            DC::NotesDismiss(id) => {
                state
                    .desktop
                    .notifications
                    .borrow_mut()
                    .retain(|n| n.id != id);
                persist_notes(state);
                refresh_notification_panel(state);
            }
            // Plan 479 T3：notes_toggle（dock 铃铛钮；召唤执行体，T1 定案 1）。
            DC::NotesToggle => {
                tasks.push(toggle_notification_center(state));
            }
            // Plan 540 T7：open_settings（齿轮/菜单）——launch-or-focus
            // 设置窗（execute_open_settings）。
            DC::OpenSettings => execute_open_settings(state),
            // PLAN-024 v1.8：dashboard 面板开合/编辑/launch 执行臂。
            DC::DashboardToggle => {
                tasks.push(toggle_dashboard(state));
            }
            DC::DashboardClose => close_dashboard(state),
            DC::DashboardPin(app) => dashboard_set_pinned(state, &app, true),
            DC::DashboardUnpin(app) => dashboard_set_pinned(state, &app, false),
            DC::DashboardSpan(app, span) => dashboard_set_span(state, &app, span),
            DC::DashboardLaunch(app) => execute_launch_app(state, &app),
            // Plan 487 M4：dock 几何驱动动词（I7：热改 dock_edges + relayout
            // + storage 写回；执行体见下）。
            DC::SetDockPosition(top) => execute_set_dock_position(state, top),
            DC::SetDockEnabled(on) => execute_set_dock_enabled(state, on),
            // Plan 518 G1：主题热切换（Appearance 分区 Dark Mode 钮）。
            DC::SetTheme(dark) => execute_set_theme(state, dark),
            // PLAN-601 T-05：命名主题热切换（set_theme 未知名解析期已拒）。
            DC::SetThemeName(name) => execute_set_theme_name(state, &name.clone()),
            // PLAN-053 T-02：宿主主题策略选择（策略核心单点执行臂）。
            DC::SetThemePolicy(family, mode) => {
                execute_set_theme_policy(state, &family, &mode)
            }
        }
    }
    (false, tasks)
}

/// Plan 518 G1 + Plan 540 T3：`set_theme` 执行臂——set_dark_mode 即时生效
/// （语义 token + 窗口调色板同源跟帧）+ 单源 `config.dark_theme` 落盘
/// （boot 读回）+ 已声明 dark_mode 的 App 状态变量同步（458 语义：运行时
/// 变量每帧回写全局,不同步则被旧值翻回）+ 全 App view_dirty（构建期解析的
/// 颜色需重建换色）。
/// PLAN-601 T-05：命名主题执行体——style::theme::set_theme（THEME_EPOCH
/// 自增 → 既有失效回路：view 重建→语义 token 重解析）+ config.theme_name
/// 单源落盘 + 全场快照随撤 + 全 App 视图标脏（execute_set_theme 同款生效面）。
fn execute_set_theme_name(state: &mut crate::ui::session::DesktopSession, name: &str) {
    if !crate::ui::style::theme::set_theme(name) {
        return; // 未知名（解析期已拒，防御双门）
    }
    state.desktop.config.theme_name = Some(name.to_string());
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    crate::ui::iced::snapshot::invalidate_all();
    for app in state.apps.values_mut() {
        *app.state.view_dirty.borrow_mut() = true;
    }
}

/// PLAN-053 T-02：宿主主题策略选择执行臂——策略核心 stage（校验 + 候选
/// 改写）→ config 落盘（失败 = 零内存变更 + 具名状态面）→ publish_staged
/// （revision 发布）→ 全场生效（legacy 档位 + 全场快照撤 + 全 App
/// view_dirty + legacy dark_mode 回写 + 031 scoped 端口喂宿主输入）→
/// shell 状态面回写。校验拒绝：具名原因进 shell 状态面，policy/config
/// 零变更（保旧）；全等候选：Unchanged 零落盘（杀外写/重复写回环）。
fn execute_set_theme_policy(
    state: &mut crate::ui::session::DesktopSession,
    family: &str,
    mode: &str,
) {
    // 候选 = 活配置副本；shell 只给 family/mode，version/digest 由宿主目录补全
    // （shell 不持 digest——目录真源在宿主 boot 注入的 catalog）。
    let mut candidate = state.desktop.config.clone();
    match crate::ui::theme_policy::catalog() {
        Some(cat) => {
            candidate.theme_family = Some(family.to_string());
            candidate.theme_mode = Some(mode.to_string());
            candidate.theme_family_version = Some(cat.version.clone());
            candidate.theme_family_digest = Some(cat.digest.clone());
        }
        None => {
            // 目录缺席：不构造候选，直接具名拒绝（零落盘）。
            sync_theme_shell_state(
                state,
                Some(&format!(
                    "rejected: 主题包目录不可用（包缺席或校验失败）——family {family:?} 候选拒绝"
                )),
            );
            return;
        }
    }
    match crate::ui::theme_policy::stage_candidate(&mut candidate) {
        crate::ui::theme_policy::PolicyOutcome::Staged(staged) => {
            if crate::ui::desktop_config::save(&candidate).is_err() {
                // 写失败不发布内存 revision（§5.2 提交序）；stage 只动了
                // 候选副本，活配置未变 = 内存 policy 自动保旧。
                sync_theme_shell_state(state, Some("save-failed: 策略落盘失败——保旧 policy"));
                return;
            }
            let rev = crate::ui::theme_policy::publish_staged(staged);
            apply_theme_policy_effect(state, mode, rev);
            // outproc 能力会话广播（完整快照推送；pump 排水发出）。
            state.broadcast_theme_policy();
            sync_theme_shell_state(state, None);
        }
        crate::ui::theme_policy::PolicyOutcome::Rejected(reason) => {
            sync_theme_shell_state(state, Some(&format!("rejected: {reason}")));
        }
        crate::ui::theme_policy::PolicyOutcome::Unchanged => {
            // 与已提交策略全等：零落盘零重推，仅刷新状态面。
            sync_theme_shell_state(state, None);
        }
    }
}

/// PLAN-053 T-02：策略生效面（与 execute_set_theme 同款全场语义 + 031
/// scoped 端口差异面）：legacy 组件/shell chrome 走 adapter dark 档与
/// `dark_mode` 状态回写；有主题 scope 的 App（themes dep 采纳者）经
/// `set_host_theme_inputs` 喂宿主完整快照（失败保旧由 031 publish 层保证，
/// 本臂不逐 App 试错）。全场快照撤 + view_dirty 双路径共用（P041 等价）。
fn apply_theme_policy_effect(
    state: &mut crate::ui::session::DesktopSession,
    mode: &str,
    rev: u64,
) {
    let _ = rev;
    let resolved = crate::ui::theme_policy::resolved_dark(mode);
    if let Some(dark) = resolved {
        crate::ui::style_iced_adapter::set_dark_mode(dark);
        #[cfg(feature = "autodown")]
        crate::ui::autodown_editor::retheme_all_fence_buffers();
    }
    crate::ui::iced::snapshot::invalidate_all();
    let inputs = crate::ui::theme_policy::current().map(|p| p.snapshot().host_inputs());
    for app in state.apps.values_mut() {
        if let Some(inputs) = inputs.as_ref() {
            if app.component.theme_scope().is_some() {
                app.component.set_host_theme_inputs(inputs.clone());
                *app.state.view_dirty.borrow_mut() = true;
                continue;
            }
        }
        // legacy 链：adapter 档已全局生效，这里回写 App 级 dark_mode 状态变量
        // （458 语义：运行时变量每帧回写全局，不同步则被旧值翻回）。
        if let Some(dark) = resolved {
            if app.component.read_state("dark_mode").is_ok() {
                let _ = app
                    .component
                    .write_state("dark_mode", auto_val::Value::Bool(dark));
            }
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
}

/// PLAN-053 T-02：shell 主题状态面推送（`__theme_*` 模型变量，desktop.at
/// 消费）——family/mode/rev + 每应用状态行（in-proc 现算 + outproc 结果表）。
/// `error: Option<&str>` 非空 = 具名拒绝/落盘失败诊断（进 `__theme_error`）。
/// host.rs ServiceTick 经脏位消费同一入口（同值零写，变更检测内置）。
pub(crate) fn sync_theme_shell_state(
    state: &mut crate::ui::session::DesktopSession,
    error: Option<&str>,
) {
    let policy = crate::ui::theme_policy::current();
    let (family, mode, rev) = match &policy {
        Some(p) => (p.family.clone(), p.mode.clone(), p.rev),
        None => (String::new(), String::new(), 0),
    };
    // in-proc App 状态：scope 在场（采纳者）→ 与已提交策略比对 + eval 注册
    // 表读回；legacy → 随 adapter 档（applied 语义 = 全局档生效）。
    let mut rows: Vec<String> = Vec::new();
    let registry_names: std::collections::HashMap<u64, String> = state
        .host
        .as_ref()
        .map(|h| {
            h.wm
                .wins
                .iter()
                .filter_map(|(_wid, w)| w.registry_id.clone().map(|id| (w.app.0, id)))
                .collect()
        })
        .unwrap_or_default();
    for (id, app) in state.apps.iter() {
        let name = registry_names
            .get(&id.0)
            .cloned()
            .unwrap_or_else(|| format!("app-{}", id.0));
        match app.component.theme_scope() {
            Some(scope) => {
                let status = match (&policy, crate::ui::theme_policy::last_scope_eval(scope.id()))
                {
                    (Some(p), _) if scope.family_name() == p.family => "applied".to_string(),
                    (Some(_), Some((false, reason))) => format!("rejected: {reason}"),
                    (Some(_), _) => "pending".to_string(),
                    (None, _) => "idle".to_string(),
                };
                rows.push(format!("{name}={status}"));
            }
            None => rows.push(format!("{name}=legacy")),
        }
    }
    for (name, status, detail) in state.theme_outproc_status.borrow().iter() {
        if detail.is_empty() {
            rows.push(format!("{name}={status}"));
        } else {
            rows.push(format!("{name}={status}: {detail}"));
        }
    }
    rows.sort();
    rows.dedup();
    // 摘要头（family@mode rev=N）+ 每应用状态行；未激活 = 仅行集（恒空串）。
    let status = if policy.is_some() {
        format!(
            "{}@{} rev{}; {}",
            family,
            mode,
            rev,
            rows.join(", ")
        )
    } else {
        rows.join(", ")
    };
    if let Some(panel) = state.desktop.desktop_app {
        if let Some(app) = state.apps.get_mut(&panel) {
            // 变更检测写（ServiceTick 周期调用——同值零写零标脏，防逐帧重建）。
            let reads = (
                app.component.read_state("__theme_family").ok(),
                app.component.read_state("__theme_mode").ok(),
                app.component.read_state("__theme_rev").ok(),
                app.component.read_state("__theme_status").ok(),
                app.component.read_state("__theme_error").ok(),
            );
            let want = (
                Some(auto_val::Value::str(&family)),
                Some(auto_val::Value::str(&mode)),
                Some(auto_val::Value::Int(rev as i32)),
                Some(auto_val::Value::str(&status)),
                Some(auto_val::Value::str(error.unwrap_or(""))),
            );
            if reads == want {
                return;
            }
            let _ = app
                .component
                .write_state("__theme_family", auto_val::Value::str(&family));
            let _ = app
                .component
                .write_state("__theme_mode", auto_val::Value::str(&mode));
            let _ = app
                .component
                .write_state("__theme_rev", auto_val::Value::Int(rev as i32));
            let _ = app
                .component
                .write_state("__theme_status", auto_val::Value::str(&status));
            let _ = app.component.write_state(
                "__theme_error",
                auto_val::Value::str(error.unwrap_or("")),
            );
            *app.state.view_dirty.borrow_mut() = true;
            // PLAN-053 T-05：pending 行自续拍（eval 落定在帧构建路径——
            // 晚于本 sync；有待定行 = 下拍再查，落定即停，稳态零成本）。
            if status.contains("=pending") {
                crate::ui::theme_policy::mark_shell_dirty();
            }
        }
    }
}

fn execute_set_theme(state: &mut crate::ui::session::DesktopSession, dark: bool) {
    crate::ui::style_iced_adapter::set_dark_mode(dark);
    // PLAN-051 T10（DEBTS 050 处置）：桌面壳主题翻转臂与 D-GAP 值变化臂
    // 同款——既有 fence buffer 重着色到新档 hljs 主题。
    // PLAN-551 门控补缺（合并取 master 字面门）：随 autodown_editor 模块
    // 同门控 all(autodown, code-editor)；autodown 隐含 ui-iced→code-editor,
    // ui-iced 单开时跳过——fence 重着色本就是 autodown 编辑壳的附加面。
    #[cfg(feature = "autodown")]
    crate::ui::autodown_editor::retheme_all_fence_buffers();
    state.desktop.config.dark_theme = dark;
    // PLAN-615 T-06：用户显式切换即置 manual——终结 theme_source=system 的
    // OS 跟随派生（boot/外写热应用 load() 不再覆盖此选择）。
    state.desktop.config.theme_source = "manual".to_string();
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    // Plan 497 G3 同款：全场快照随撤（窗口缩略按旧主题渲染）。
    crate::ui::iced::snapshot::invalidate_all();
    // PLAN-053 T-02：family 链激活时采纳者的 dark_mode 由已提交策略派生
    // mode 驱动（同 apply_external_config_diff 差异臂；legacy 词不越权）。
    let policy_dark = crate::ui::theme_policy::current().and_then(|p| p.resolved_dark());
    for app in state.apps.values_mut() {
        let effective = if app.component.theme_scope().is_some() {
            policy_dark
        } else {
            Some(dark)
        };
        if let Some(d) = effective {
            if app.component.read_state("dark_mode").is_ok() {
                let _ = app
                    .component
                    .write_state("dark_mode", auto_val::Value::Bool(d));
            }
        }
        *app.state.view_dirty.borrow_mut() = true;
    }
    // PLAN-043 Phase 3：分主题壁纸——切主题即应用**目标主题**槽（槽空 =
    // 该主题未单独设过壁纸，维持当前壁纸不跟随）。与 SetWallpaper 同链：
    // 旧布局快照/单值+槽落盘/会话回读/快照全撤/格子重注入都在
    // execute_set_wallpaper 内（槽回写自身同值 = no-op）。
    let slot = if dark {
        state.desktop.config.wallpaper_path_dark.clone()
    } else {
        state.desktop.config.wallpaper_path_light.clone()
    };
    if !slot.is_empty() && slot != state.desktop.config.wallpaper_path {
        execute_set_wallpaper(state, &slot);
    }
}

/// PLAN-526 T14 + Plan 540 T3：壁纸热切换执行体——单源 `config.wallpaper_path`
/// 落盘 + 会话字段回读（load_desktop_wallpaper 含 is_file 验证与坏值
/// 回退）+ 快照全撤。壁纸层 view 每帧重建读该字段，下一帧即生效
/// （518 G1 set_theme 同链先例）。
fn execute_set_wallpaper(state: &mut crate::ui::session::DesktopSession, path: &str) {
    // PLAN-043 Phase 3 执行期修正：反斜杠路径经 `__desktop_cmd` 记录链
    // （VM 字符串管道）会丢 `\`（实机 picker 选 D:\Down\... 落盘成
    // "D:Downstella-os..."，靠目录首图兜底假活）——入口统一规整为
    // 正斜杠（Windows 文件 API 等价接受）。
    let path = path.trim().replace('\\', "/");
    // PLAN-019 T-06：布局随壁纸切换——先把**旧壁纸**的现行布局快照落其
    // 布局键（旧壁纸无键控语境 = 布局本就在缺省底稿，跳过），再切换。
    {
        let old_key = current_wallpaper_layout_key(state);
        if let Some(old_key) = old_key {
            let snapshot = load_desktop_positions_for(Some(&old_key));
            auto_lang_core::vm::ffi::stdlib::storage_host_publish(
                &old_key,
                serialize_positions(&snapshot),
            );
        }
    }
    state.desktop.config.wallpaper_path = path.to_string();
    // PLAN-043 Phase 3：分主题壁纸——写**当前主题**槽（记忆面；另一槽
    // 不动）。单值仍是活跃值，槽在本次 save 同步持久化。
    if state.desktop.config.dark_theme {
        state.desktop.config.wallpaper_path_dark = path.to_string();
    } else {
        state.desktop.config.wallpaper_path_light = path.to_string();
    }
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    state.desktop.desktop_wallpaper = load_desktop_wallpaper(&state.desktop.config);
    crate::ui::iced::snapshot::invalidate_all();
    // PLAN-019 v1.7：picker 开着时点选/轮换 = flip 游标同步（后续 ←/→
    // 从这张继续）；__wp_current 随写刷新（高亮判据面）。
    if state.host.as_ref().is_some_and(|h| h.wm.picker_open) {
        inject_wallpaper_picker(state);
    }
    // PLAN-019 T-06：格子表按新壁纸布局键重注入（缺席回退缺省底稿）——
    // 布局随壁纸即时切换（负一屏 flip 对比 = 所见即所得）。
    inject_desktop_surface(state);
}

/// PLAN-019 v1.7：显示桌面（负一屏）——簿记在 [`WmState::show_desktop`]
/// （懒建保留分区 + origin 记录 + 切入）。投影经指纹差分自动刷新：进入/
/// 返回必翻 current 位（分区段 `{id}:{current},{label}`），`__wm_showdesk`
/// 随写同步（sync 臂同一守卫块）。
fn execute_show_desktop(state: &mut crate::ui::session::DesktopSession) {
    if let Some(host) = state.host.as_mut() {
        host.wm.show_desktop();
    }
}

/// PLAN-019 v1.7：返回 origin——picker 开着先关（簿记幂等清零，sliver
/// toggle 不留悬浮 picker）再 [`WmState::showdesk_return`]。
fn execute_showdesk_return(state: &mut crate::ui::session::DesktopSession) {
    if let Some(host) = state.host.as_mut() {
        host.wm.picker_open = false;
        host.wm.picker_return_on_close = false;
        host.wm.showdesk_return();
    }
    PICKER_KEYS_OPEN.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// PLAN-019 v1.7：更换壁纸组合臂 = show_desktop 幂等 + picker 开 + 归属
/// 规则单点（**谁切屏谁负责切回**：到达前已在负一屏 = 用户自入，关闭时
/// 不代管返回）。注入面经 inject_wallpaper_picker 直写（picker 态变化
/// 不走指纹门控——桌面面字段直写 + view_dirty，inject_desktop_surface
/// 同模式）。
fn execute_wallpaper_pick(state: &mut crate::ui::session::DesktopSession) {
    let was_on_showdesk = state.host.as_ref().is_some_and(|h| h.wm.on_showdesk());
    if let Some(host) = state.host.as_mut() {
        host.wm.show_desktop();
        host.wm.picker_open = true;
        host.wm.picker_return_on_close = !was_on_showdesk;
    }
    PICKER_KEYS_OPEN.store(true, std::sync::atomic::Ordering::Relaxed);
    inject_wallpaper_picker(state);
    // FU7：滑窗对齐当前壁纸（可见 5 枚窗口容纳当前项居中偏左）。
    let cur_ix = state.host.as_ref().and_then(|h| {
        h.wm.picker_paths
            .iter()
            .position(|p| *p == state.desktop.desktop_wallpaper)
    });
    if let Some(ix) = cur_ix {
        if let Some(host) = state.host.as_mut() {
            let len = host.wm.picker_paths.len();
            host.wm.picker_win = ix.saturating_sub(2).min(len.saturating_sub(5.min(len)));
        }
    }
}

/// PLAN-019 v1.7：关闭 picker——簿记清零；return_on_close 分支走
/// showdesk_return（origin 消费）。负一屏自入开 picker 的关闭只关面板
/// 不切屏（AC-04）。
fn execute_wallpaper_close(state: &mut crate::ui::session::DesktopSession) {
    let should_return = state
        .host
        .as_ref()
        .map(|h| h.wm.picker_return_on_close)
        .unwrap_or(false);
    if let Some(host) = state.host.as_mut() {
        host.wm.picker_open = false;
        host.wm.picker_return_on_close = false;
        host.wm.picker_preview = None;
    }
    PICKER_KEYS_OPEN.store(false, std::sync::atomic::Ordering::Relaxed);
    inject_wallpaper_picker(state);
    if should_return {
        execute_showdesk_return(state);
    }
}

/// PLAN-019 v1.7：picker Esc 逐级退回链（update 臂薄壳直调；AC-05）——
/// 预览态回栅格态（picker 保持开）；栅格态关闭（[`execute_wallpaper_close`]
/// 按归属簿记决定是否自动返回 origin）。
pub(super) fn execute_wallpaper_escape(state: &mut crate::ui::session::DesktopSession) {
    let preview_open = state
        .host
        .as_ref()
        .is_some_and(|h| h.wm.picker_preview.is_some());
    if preview_open {
        if let Some(host) = state.host.as_mut() {
            host.wm.picker_preview = None;
        }
        inject_wallpaper_picker(state);
    } else {
        execute_wallpaper_close(state);
    }
}

/// PLAN-019 v1.7：预览进出臂——空参 = 退栅格态；带参 = 按 path 反查
/// 游标进入大图预览（缺席/关态 no-op）。`__wp_preview` 注入载荷即路径
/// （.at 零算术直渲染）。
fn execute_wallpaper_preview(state: &mut crate::ui::session::DesktopSession, path: &str) {
    if !state.host.as_ref().is_some_and(|h| h.wm.picker_open) {
        return;
    }
    if path.is_empty() {
        if let Some(host) = state.host.as_mut() {
            host.wm.picker_preview = None;
        }
    } else {
        let hit = state
            .host
            .as_ref()
            .and_then(|h| h.wm.picker_paths.iter().position(|p| p == path));
        match hit {
            Some(ix) => {
                if let Some(host) = state.host.as_mut() {
                    host.wm.picker_preview = Some(ix);
                }
            }
            None => return,
        }
    }
    inject_wallpaper_picker(state);
}

/// PLAN-019 v1.7：目录浏览臂——原生 pick_folder 对话框（rfd；PLAN-804
/// 起 dialog_parent 为 host_adapters::dialog 私有模块——pick_folder 族
/// 仍无复用面，v1 无父窗绑定，模态语义不受影响）→ 选定走
/// [`execute_set_wallpapers_dir`] 同一写臂
/// （config 单源 + mtime 热轮询），取消 = no-op。
fn execute_wallpaper_browse_dir(state: &mut crate::ui::session::DesktopSession) {
    let Some(dir) = rfd::FileDialog::new()
        .set_title("选择壁纸目录")
        .pick_folder()
    else {
        return;
    };
    execute_set_wallpapers_dir(state, &dir.to_string_lossy());
}

/// PLAN-019 v1.7：picker 导航臂（.at 无列表下标算术，导航数学宿主收口
/// ——B12 族）。预览态 = 大图游标环绕移动；栅格态 = flip 对比轮换并
/// 立即应用（cursor 同步走 [`execute_set_wallpaper`]）。
pub(super) fn execute_wallpaper_nav(state: &mut crate::ui::session::DesktopSession, dir: &str) {
    let open = state.host.as_ref().is_some_and(|h| h.wm.picker_open);
    let len = state
        .host
        .as_ref()
        .map(|h| h.wm.picker_paths.len())
        .unwrap_or(0);
    if !open || len == 0 {
        return;
    }
    let step: isize = if dir == "next" { 1 } else { -1 };
    {
        let host = state.host.as_mut().unwrap();
        if let Some(ix) = host.wm.picker_preview {
            // 预览态：大图游标环绕移动。
            host.wm.picker_preview = Some((ix as isize + step).rem_euclid(len as isize) as usize);
        } else {
            // FU7：栅格态 = carousel 滑窗（导航只滑窗不应用；点选才应用）。
            let max_win = len.saturating_sub(5) as isize;
            let w = (host.wm.picker_win as isize + step).clamp(0, max_win);
            host.wm.picker_win = w as usize;
        }
    }
    inject_wallpaper_picker(state);
}

/// PLAN-019 v1.7：picker 注入面——`__wp_picker`/`__wp_preview`/
/// `__wp_dir`/`__wp_current`/`__wp_items`（+ `wp_paths` 平行列表）。候选
/// 供源复用 [`scan_wallpapers_dir`]（jpg/jpeg/png 文件名升序）；paths
/// 缓存宿主侧（wallpaper_nav 数学区数据面，协议 §2.1）。
fn inject_wallpaper_picker(state: &mut crate::ui::session::DesktopSession) {
    let Some(surface) = state.desktop.desktop_app else {
        return;
    };
    let items = scan_wallpapers_dir(&state.desktop.config);
    let paths: Vec<String> = items
        .iter()
        .map(|v| match v {
            auto_val::Value::Obj(o) => match o.get("path") {
                Some(auto_val::Value::Str(s)) => s.to_string(),
                _ => String::new(),
            },
            _ => String::new(),
        })
        .collect();
    let dir = wallpapers_dir_or_default(&state.desktop.config)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let current = state.desktop.desktop_wallpaper.clone();
    // __wp_preview 载荷 = 预览图路径（"" = 栅格态）——.at 零算术直渲染
    // （无列表下标原语）；下标簿记仍宿主持有（nav 数学区）。
    let preview = state
        .host
        .as_ref()
        .and_then(|h| h.wm.picker_preview)
        .and_then(|ix| {
            state
                .host
                .as_ref()
                .map(|h| h.wm.picker_paths.get(ix).cloned())
                .unwrap_or(None)
        })
        .unwrap_or_default();
    let open = state.host.as_ref().is_some_and(|h| h.wm.picker_open);
    {
        let host = state.host.as_mut().unwrap();
        host.wm.picker_paths = paths.clone();
    }
    // FU7：carousel 滑窗切片（可见 5 枚）+ 面板锚点（底部居中、贴
    // 任务栏留 8px；宿主算好坐标注入——.at 无算术）。
    let win = state.host.as_ref().map(|h| h.wm.picker_win).unwrap_or(0);
    let visible: Vec<auto_val::Value> = items.iter().skip(win).take(5).cloned().collect();
    let viewport = state.host_viewport();
    let usable = crate::ui::layout::usable_rect(viewport, state.desktop.dock_edges);
    let panel_w = 720.0_f32;
    let panel_h = 190.0_f32;
    let wp_x = usable.x + ((usable.width - panel_w).max(0.0)) / 2.0;
    let wp_y = usable.y + usable.height - panel_h - 8.0;
    let Some(app) = state.apps.get_mut(&surface) else {
        return;
    };
    let _ = app.component.write_state(
        "__wp_picker",
        auto_val::Value::str(if open { "1" } else { "" }),
    );
    let _ = app
        .component
        .write_state("__wp_preview", auto_val::Value::str(&preview));
    let _ = app
        .component
        .write_state("__wp_x", auto_val::Value::Float(wp_x as f64));
    let _ = app
        .component
        .write_state("__wp_y", auto_val::Value::Float(wp_y as f64));
    let _ = app.component.write_state_vec("__wp_visible", visible);
    let _ = app
        .component
        .write_state("__wp_dir", auto_val::Value::str(&dir));
    let _ = app
        .component
        .write_state("__wp_current", auto_val::Value::str(&current));
    let _ = app.component.write_state_vec("__wp_items", items);
    let _ = app.component.write_state_vec(
        "wp_paths",
        paths.iter().map(|p| auto_val::Value::str(p)).collect(),
    );
    *app.state.view_dirty.borrow_mut() = true;
}

/// Plan 540 T3：透明度写臂——config 落盘（虚拟窗底色每帧自 config 映射，
/// 下一帧即时生效；518 G6 即时语义保持）。
fn execute_set_transparency(state: &mut crate::ui::session::DesktopSession, level: &str) {
    state.desktop.config.transparency = level.to_string();
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
}

/// Plan 540 T3：通知开关写臂——config 落盘（push_notification 门控直读，
/// 后续 notify 全链路即时短路/恢复）。
fn execute_set_notes_enabled(state: &mut crate::ui::session::DesktopSession, on: bool) {
    state.desktop.config.notes_enabled = on;
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
}

/// Plan 540 T3：pinned 写臂——config 落盘 + 会话域同步 + shell 投影热同步
/// （`inject_dock_pinned` 同格式 {id,icon} 注入。PLAN-012 W4：空表 = 空表
/// ——缺省三枚回退语义退役，显式空即空，parse/load 同步改）。
fn execute_set_dock_pinned(state: &mut crate::ui::session::DesktopSession, csv: &str) {
    let list: Vec<String> = csv
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    state.desktop.config.dock_pinned = list;
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    state.desktop.dock_pinned = state.desktop.config.dock_pinned.clone();
    inject_dock_pinned(state);
}

/// PLAN-012 W4 协议 v1.6：dock 单枚固定/取消固定执行臂——Vec 增删去重
/// （pin 已在表 = 幂等保持；unpin 不在表 = 幂等跳过）→ config 落盘 →
/// 会话域同步 + 投影热同步（Obj 数组与 csv 派生面都由同一会话域事实源
/// 刷新，sync_shell_windows 指纹差分拾取）。
fn execute_dock_pin(state: &mut crate::ui::session::DesktopSession, id: &str, pin: bool) {
    if id.is_empty() {
        return;
    }
    let mut list = state.desktop.config.dock_pinned.clone();
    if pin {
        if !list.iter().any(|e| e == id) {
            list.push(id.to_string());
        }
    } else if !list.iter().any(|e| e == id) {
        return;
    } else {
        list.retain(|e| e != id);
    }
    state.desktop.config.dock_pinned = list;
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    state.desktop.dock_pinned = state.desktop.config.dock_pinned.clone();
    inject_dock_pinned(state);
    inject_desktop_surface(state);
}

/// Plan 551 T6:外写热应用轮询——os-config(或任意写方)经 daemon 改
/// config.at 后,宿主 ServiceTick(400ms)stat mtime;变更即 load() 差异
/// 应用(400ms 节拍 ≪ 2s 生效门)。写竞争防回环:宿主自写臂 save() 后
/// mtime 变化,下次 poll 读回内容与内存态相等即早退(DesktopConfig:
/// PartialEq 全字段);应用臂不 save(宿主非本路径写方,避免 mtime 抖动)。
pub(super) fn poll_external_config(state: &mut crate::ui::session::DesktopSession) {
    if !state.desktop.config_poll_sampled {
        // boot 后首采样:boot load() 已是当前事实,只建锚不应用
        // (文件缺席 = None 锚,后续创建文件即 Some≠None → 应用)。
        state.desktop.config_poll_sampled = true;
        state.desktop.config_poll_mtime = crate::ui::desktop_config::config_mtime();
        return;
    }
    let mtime = crate::ui::desktop_config::config_mtime();
    if mtime == state.desktop.config_poll_mtime {
        return;
    }
    state.desktop.config_poll_mtime = mtime;
    let cfg = crate::ui::desktop_config::load();
    if cfg == state.desktop.config {
        return;
    }
    apply_external_config_diff(state, &cfg);
}

/// Plan 551 T6:差异应用臂——按字段复用 set_* 执行臂的生效语义(主题/
/// 壁纸/透明度/Dock/通知),但不落盘(宿主非本路径写方)。透明度与壁纸
/// 目录为逐帧/按需消费面,零额外动作。
pub(super) fn apply_external_config_diff(
    state: &mut crate::ui::session::DesktopSession,
    cfg: &crate::ui::desktop_config::DesktopConfig,
) {
    let old = state.desktop.config.clone();
    // PLAN-053 T-02：策略组差异 → 与 bus 动词**同一**策略核心入口（§5.2
    // 双路径等价：外写 = 候选来源不同，校验/提交/发布序完全一致）。外写方
    // 通常不持 version/digest——由宿主目录补全后参与校验；stage 全等判
    // Unchanged（杀外写回环）、拒绝具名状态面、提交改写候选字段并落盘
    // （补全值 + rev 是宿主权威事实；落盘后 mtime 变化一次，下轮 poll
    // 内容相等早退收敛）。
    let policy_diff = cfg.theme_family != old.theme_family
        || cfg.theme_mode != old.theme_mode
        || cfg.theme_family_version != old.theme_family_version
        || cfg.theme_family_digest != old.theme_family_digest;
    if policy_diff {
        let mut candidate = cfg.clone();
        match crate::ui::theme_policy::catalog() {
            Some(cat) => {
                candidate.theme_family_version = Some(cat.version.clone());
                candidate.theme_family_digest = Some(cat.digest.clone());
            }
            None => {
                candidate.theme_family_version = None;
                candidate.theme_family_digest = None;
            }
        }
    match crate::ui::theme_policy::stage_candidate(&mut candidate) {
        crate::ui::theme_policy::PolicyOutcome::Staged(staged) => {
            state.desktop.config = candidate.clone();
                if crate::ui::desktop_config::save(&state.desktop.config).is_ok() {
                    let mode = candidate.theme_mode.clone().unwrap_or_default();
                    let rev = crate::ui::theme_policy::publish_staged(staged);
                    apply_theme_policy_effect(state, &mode, rev);
                    state.broadcast_theme_policy();
                    sync_theme_shell_state(state, None);
                } else {
                    sync_theme_shell_state(
                        state,
                        Some("save-failed: 外写策略落盘失败——保旧 policy"),
                    );
                }
            }
            crate::ui::theme_policy::PolicyOutcome::Rejected(reason) => {
                sync_theme_shell_state(state, Some(&format!("rejected: {reason}")));
            }
            crate::ui::theme_policy::PolicyOutcome::Unchanged => {
                // 盘面与已提交策略语义全等：内存镜像保持权威，无需动作。
            }
        }
    }
    if cfg.dock_position != old.dock_position || cfg.dock_enabled != old.dock_enabled {
        state.desktop.config.dock_position = cfg.dock_position.clone();
        state.desktop.config.dock_enabled = cfg.dock_enabled;
        apply_dock_edges_now(state);
    }
    if cfg.dock_pinned != old.dock_pinned {
        state.desktop.config.dock_pinned = cfg.dock_pinned.clone();
        state.desktop.dock_pinned = state.desktop.config.dock_pinned.clone();
        inject_dock_pinned(state);
    }
    if cfg.theme_name != old.theme_name {
        // PLAN-601 T-05：boot 读回/热应用差分——THemE_EPOCH 失效回路接管换色。
        if let Some(name) = cfg.theme_name.as_deref() {
            crate::ui::style::theme::set_theme(name);
        }
        crate::ui::iced::snapshot::invalidate_all();
        for app in state.apps.values_mut() {
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
    if cfg.dark_theme != old.dark_theme {
        // execute_set_theme 同款生效面(减 save):adapter 切换 + fence
        // 重着色(autodown 门控)+ 全场快照随撤 + 全 App view_dirty/dark_mode 回写。
        // PLAN-041 T-05（F-R1 根修）：补 snapshot::invalidate_all——face
        // 内容/缩略/pager 走快照缓存，缺此则 config 外写热切后 face 渲染
        // 滞留旧主题（bus 臂 12345 同款；2026-09-22 实机复现 shot-07）。
        crate::ui::style_iced_adapter::set_dark_mode(cfg.dark_theme);
        #[cfg(feature = "autodown")]
        crate::ui::autodown_editor::retheme_all_fence_buffers();
        crate::ui::iced::snapshot::invalidate_all();
        // PLAN-053 T-04：family 链激活时，031 采纳者（有主题 scope）的
        // `dark_mode` 状态位由**已提交策略的派生 mode**驱动（视图类分支
        // 读它——rejected 候选的 legacy dark_theme 翻转不得越权改色，
        // §5.2 拒绝保旧）；legacy 组件沿 cfg.dark_theme（既有语义）。
        let policy_dark = crate::ui::theme_policy::current().and_then(|p| p.resolved_dark());
        let dark = cfg.dark_theme;
        for app in state.apps.values_mut() {
            let effective = if app.component.theme_scope().is_some() {
                policy_dark
            } else {
                Some(dark)
            };
            if let Some(d) = effective {
                if app.component.read_state("dark_mode").is_ok() {
                    let _ = app
                        .component
                        .write_state("dark_mode", auto_val::Value::Bool(d));
                }
            }
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
    if cfg.wallpaper_path != old.wallpaper_path {
        state.desktop.config.wallpaper_path = cfg.wallpaper_path.clone();
        state.desktop.desktop_wallpaper = load_desktop_wallpaper(&state.desktop.config);
        crate::ui::iced::snapshot::invalidate_all();
    }
    if cfg.wallpaper_request != old.wallpaper_request {
        // PLAN-019-FU8：os-config 设置面「切换壁纸」请求通道——config.at
        // wallpaper_request 值变化(非空)即触发一次 wallpaper_pick 组合臂
        // (与桌面右键「更换壁纸…」同链：show_desktop 幂等 + picker 开 +
        // return_on_close 归属单点)。值由设置面交替("1"/"2")，宿主只认
        // 变化、不清值(config.at 单写方仍是 daemon/设置面)。
        state.desktop.config.wallpaper_request = cfg.wallpaper_request.clone();
        if !cfg.wallpaper_request.is_empty() {
            execute_wallpaper_pick(state);
        }
    }
    state.desktop.config.wallpapers_dir = cfg.wallpapers_dir.clone();
    state.desktop.config.transparency = cfg.transparency.clone();
    state.desktop.config.notes_enabled = cfg.notes_enabled;
    state.desktop.config.dark_theme = cfg.dark_theme;
    state.desktop.config.theme_name = cfg.theme_name.clone();
}

/// Plan 540 T3：壁纸目录写臂——config 落盘（scan_wallpapers_dir 与缺省
/// 壁纸链共用解析；设置面重开即见新目录扫描）。
fn execute_set_wallpapers_dir(state: &mut crate::ui::session::DesktopSession, dir: &str) {
    state.desktop.config.wallpapers_dir = dir.to_string();
    let _ = crate::ui::desktop_config::save(&state.desktop.config);
    // PLAN-019 v1.7：picker 开着时换目录 = 候选重扫（__wp_items/__wp_dir
    // 重注入；游标/预览态复位——目录换了旧下标无意义）。
    if state.host.as_ref().is_some_and(|h| h.wm.picker_open) {
        if let Some(host) = state.host.as_mut() {
            host.wm.picker_preview = None;
            host.wm.picker_win = 0;
        }
        inject_wallpaper_picker(state);
    }
}

/// PLAN-041 T-04：分区切换后的呈现即时生效面。状态/投影由 WmState 臂
/// 与 sync_shell_windows 负责；此处补呈现半边——pager/切换预览数据重
/// 发布 + shell 视图重建（任务栏/预览即时随切，不等 400ms 帧泵兜底）。
fn after_workspace_switch(state: &mut crate::ui::session::DesktopSession) {
    if std::env::var("AUTO_DEBUG_KEYS").is_ok() {
        let cur = state
            .host
            .as_ref()
            .map(|h| h.wm.current_workspace)
            .unwrap_or(usize::MAX);
        eprintln!("[472-WS] switch done, current={cur}");
    }
    publish_workspace_previews(state);
    if let Some(shell) = state.desktop.shell_app {
        if let Some(app) = state.apps.get_mut(&shell) {
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
}

/// PLAN-526 T18：热键分区切换 → transient 显示切换预览面板（写 shell
/// `switcher_open="1"` + 记 ServiceTick 倒计时 ~1.6s；触发 icon 手动
/// 开合不走此——无倒计时，再点即收）。
pub(super) fn show_workspace_switcher(state: &mut crate::ui::session::DesktopSession) {
    state.desktop.switcher_until.set(Some(
        std::time::Instant::now() + std::time::Duration::from_millis(1600),
    ));
    if let Some(shell) = state.desktop.shell_app {
        if let Some(app) = state.apps.get_mut(&shell) {
            let _ = app
                .component
                .write_state("switcher_open", auto_val::Value::str("1"));
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
}

/// Plan 473 T5 / 486：下一个原生槽位的级联占位（桌面逻辑域）。dock 执行臂
/// 与 DragWatch 落点高亮同源——高亮矩形即松手落点（"所见即所得"不变量）。
fn native_candidate_logical(state: &crate::ui::session::DesktopSession) -> iced::Rectangle {
    let viewport = state.host_viewport();
    let usable = crate::ui::layout::usable_rect(viewport, state.desktop.dock_edges);
    let slot_count = state
        .host
        .as_ref()
        .map(|h| h.wm.native_slots.len())
        .unwrap_or(0);
    crate::ui::layout::cascade_rect(
        slot_count,
        iced::Size::new(usable.width * 0.35, usable.height * 0.55),
        usable,
    )
}

/// Plan 473 T4：原生窗口收编执行体（`dock_native` 命令路径）。Win32 发现/几何
/// 经 `native_dock::win32`；状态机经 `WmState::advance_native_slot` 推进。
/// 槽位矩形为可用区级联占位——布局参与由 T5 接管；局部→物理 DPI 换算（本步
/// 以 scale=1 近似，缩放屏几何暂偏）、z 序沉降与 chrome 装配见 T6（待澄清④）。
/// 拒绝/失败均 toast（C1/C3 提示语义），不留半收编窗口。
#[cfg(windows)]
fn execute_dock_native(
    state: &mut crate::ui::session::DesktopSession,
    target: crate::ui::session::NativeTarget,
) {
    use crate::ui::native_dock::{win32 as ndw, NativeHwnd, RejectReason, SlotEvent};

    if !state.is_desktop() {
        push_desktop_toast(state, "error", "原生窗口 dock 仅桌面模式可用");
        return;
    }
    // ① 定位：pid 枚举首个可见顶层窗 / hwnd 直传。
    let hwnd = match target {
        crate::ui::session::NativeTarget::ByPid(pid) => ndw::find_top_level_by_pid(pid),
        crate::ui::session::NativeTarget::ByHwnd(h) => {
            let h = NativeHwnd(h);
            ndw::is_alive(h).then_some(h)
        }
    };
    let Some(hwnd) = hwnd else {
        push_desktop_toast(state, "error", RejectReason::HwndNotFound.message());
        return;
    };
    // ② C2：独占全屏（覆盖桌面所在屏域）拒绝收编——收编会撕裂显示。
    if let Some(desktop) = ndw::find_largest_own_window() {
        if ndw::covers_rect(hwnd, desktop) {
            push_desktop_toast(state, "error", RejectReason::ExclusiveFullscreen.message());
            return;
        }
    }
    // ③ C5：已最大化窗口先 restore 再摆；记 pre-dock 基线。
    if ndw::is_maximized(hwnd) {
        let _ = ndw::show_window(hwnd, ndw::ShowMode::Restore);
    }
    // pre-dock 捕获走窗口矩形域（与恢复 set_bounds 同域，忠实还原）。
    let Some(pre_bounds) = ndw::get_bounds_window(hwnd) else {
        push_desktop_toast(state, "error", RejectReason::HwndNotFound.message());
        return;
    };
    let title = ndw::get_title(hwnd).unwrap_or_else(|| "native".into());
    let pid = ndw::pid_of(hwnd).unwrap_or(0);
    // ③ 槽位矩形：可用区 1/3 级联占位（T5 起由布局引擎分配）。
    let slot_logical = native_candidate_logical(state);
    let slot_rect = crate::ui::native_dock::Rect::new(
        slot_logical.x as i32,
        slot_logical.y as i32,
        slot_logical.width as i32,
        slot_logical.height as i32,
    );
    // C3 clamp：min-size 首装未知记 0，几何写读回后缓存估计。
    let (_, fitted_slot) = match crate::ui::native_dock::clamp_to_slot(
        pre_bounds.size(),
        slot_rect,
        crate::ui::native_dock::Size::new(0, 0),
        slot_rect,
    ) {
        Ok(v) => v,
        Err(reason) => {
            push_desktop_toast(state, "error", reason.message());
            return;
        }
    };
    // ④ 登记 Candidate → 剥样式 → DockRequested → 几何排水（DPI 换算 +
    // 写入客户区 + z 序沉降）→ min-size 探测 → DockConfirmed。
    let id = {
        let host = state.host.as_mut().expect("desktop checked");
        host.wm.add_native_slot(
            hwnd.0,
            pid,
            title.clone(),
            pre_bounds,
            fitted_slot,
            slot_logical,
        )
    };
    let saved_style = match ndw::strip_chrome(hwnd) {
        Ok(s) => s,
        Err(_) => {
            dock_fail(state, id, RejectReason::Elevated);
            return;
        }
    };
    if let Some(host) = state.host.as_mut() {
        if let Some(slot) = host.wm.native_slots.get_mut(&id) {
            slot.pre_dock_style = saved_style;
        }
    }
    {
        let host = state.host.as_mut().expect("desktop checked");
        host.wm.advance_native_slot(id, SlotEvent::DockRequested);
    }
    // T6：几何经排水管线落地（替代 T4 的 scale=1 直写近似，待澄清④偿还）。
    sync_native_geometry(state);
    // ⑤ min-size 写读回探测（不可信窗口防御）+ Win11 直角 + 确认入局。
    if let Some(host) = state.host.as_mut() {
        if let Some(slot) = host.wm.native_slots.get_mut(&id) {
            if let Some(actual) = ndw::get_bounds(hwnd) {
                slot.min_size_est = crate::ui::native_dock::observe_min_size_estimate(
                    slot.slot_rect.size(),
                    actual.size(),
                );
            }
        }
    }
    let _ = ndw::set_square_corners(hwnd);
    if let Some(host) = state.host.as_mut() {
        host.wm.advance_native_slot(id, SlotEvent::DockConfirmed);
    }
    push_desktop_toast(state, "success", &format!("已收编 {title}"));
}

/// dock 失败统一出口：状态机走 Rejected 终态（注册表自动移除）+ toast。
#[cfg(windows)]
fn dock_fail(
    state: &mut crate::ui::session::DesktopSession,
    id: crate::ui::native_dock::NativeSlotId,
    reason: crate::ui::native_dock::RejectReason,
) {
    use crate::ui::native_dock::SlotEvent;
    if let Some(host) = state.host.as_mut() {
        host.wm
            .advance_native_slot(id, SlotEvent::DockFailed(reason));
    }
    push_desktop_toast(state, "error", reason.message());
}

#[cfg(not(windows))]
fn execute_dock_native(
    state: &mut crate::ui::session::DesktopSession,
    _target: crate::ui::session::NativeTarget,
) {
    push_desktop_toast(state, "error", "原生窗口 dock 仅支持 Windows");
}

/// Plan 473 T4：解除收编执行体（`undock_native` 命令路径；B2 语义）——
/// 状态机 Docked→Undocking→恢复 pre-dock bounds/样式→RestoreCompleted
/// （终态自动出注册表）。
/// Plan 473 T6：解除收编核心（undock 命令路径与 C4 用户拖走共用）——
/// 状态机 Docked→Undocking→恢复 pre-dock bounds/样式/圆角→RestoreCompleted
/// （终态自动出注册表）+ 本地矩形缓存清理。返回槽位是否存在。
#[cfg(windows)]
fn undock_native_slot(
    state: &mut crate::ui::session::DesktopSession,
    id: crate::ui::native_dock::NativeSlotId,
) -> bool {
    use crate::ui::native_dock::{win32 as ndw, SlotEvent};
    let found = state
        .host
        .as_ref()
        .and_then(|h| h.wm.native_slots.get(&id))
        .map(|slot| {
            (
                slot.hwnd,
                slot.pre_dock_bounds,
                slot.pre_dock_style,
                slot.title_cache.clone(),
            )
        });
    let Some((hwnd, bounds, style, _title)) = found else {
        return false;
    };
    if let Some(host) = state.host.as_mut() {
        host.wm.advance_native_slot(id, SlotEvent::UndockRequested);
    }
    let _ = ndw::set_bounds(hwnd, bounds);
    if style != 0 {
        let _ = ndw::restore_chrome(hwnd, style);
    }
    let _ = ndw::restore_corner_preference(hwnd);
    if let Some(host) = state.host.as_mut() {
        host.wm.advance_native_slot(id, SlotEvent::RestoreCompleted);
        host.wm.native_slot_local_rects.remove(&id);
    }
    true
}

#[cfg(windows)]
fn execute_undock_native(state: &mut crate::ui::session::DesktopSession, slot_id: u64) {
    use crate::ui::native_dock::NativeSlotId;
    if undock_native_slot(state, NativeSlotId(slot_id)) {
        push_desktop_toast(state, "success", "已恢复原生窗口");
    } else {
        push_desktop_toast(state, "error", "未知原生槽位");
    }
}

#[cfg(not(windows))]
fn execute_undock_native(state: &mut crate::ui::session::DesktopSession, _slot_id: u64) {
    push_desktop_toast(state, "error", "原生窗口 dock 仅支持 Windows");
}

/// Plan 486 v1.3：任务栏 native 条目点击——聚焦槽位原生窗（最小化先
/// SW_RESTORE，再 SetForegroundWindow best-effort——前台锁拒绝不 toast，
/// 窗口仍可能经系统动画置前）。PLAN-709：并接 WM 侧槽位置顶（伪 Wid 进
/// focused/z_order/mru——chrome 序与投影 focused 位实时）。
#[cfg(windows)]
fn execute_focus_native(state: &mut crate::ui::session::DesktopSession, slot_id: u64) {
    use crate::ui::native_dock::{win32 as ndw, NativeSlotId};
    let id = NativeSlotId(slot_id);
    // PLAN-709：任务栏条目点击「切到窗所在分区」语义（ActivateApp 同款）
    // ——槽位在隐分区先切分区（隐现 tick 随后排水拍复显）。
    let slot_ws = state
        .host
        .as_ref()
        .and_then(|h| h.wm.native_slots.get(&id))
        .map(|s| s.workspace);
    if let Some(ws) = slot_ws {
        let current = state
            .host
            .as_ref()
            .map(|h| h.wm.current_workspace)
            .unwrap_or(0);
        if ws != current {
            state.wm_set_workspace(ws);
        }
    }
    state.wm_focus_native_slot(id);
    let Some(slot) = state.host.as_ref().and_then(|h| h.wm.native_slots.get(&id)) else {
        push_desktop_toast(state, "error", "未知原生槽位");
        return;
    };
    let hwnd = slot.hwnd;
    if ndw::is_minimized(hwnd) {
        let _ = ndw::show_window(hwnd, ndw::ShowMode::Restore);
    }
    let _ = ndw::focus_window(hwnd);
    // 隐现/带序重申（切分区复显 + 聚焦置顶）。
    sync_native_geometry(state);
}

#[cfg(not(windows))]
fn execute_focus_native(state: &mut crate::ui::session::DesktopSession, _slot_id: u64) {
    push_desktop_toast(state, "error", "原生窗口 dock 仅支持 Windows");
}

/// Plan 486 v1.3：任务栏 native 条目 ×——请求关闭（WM_CLOSE 正常关闭
/// 机会；槽位由 DESTROY 事件自然回收，B7 路径无需在此移除）。
#[cfg(windows)]
fn execute_close_native(state: &mut crate::ui::session::DesktopSession, slot_id: u64) {
    use crate::ui::native_dock::{win32 as ndw, NativeSlotId};
    let Some(slot) = state
        .host
        .as_ref()
        .and_then(|h| h.wm.native_slots.get(&NativeSlotId(slot_id)))
    else {
        push_desktop_toast(state, "error", "未知原生槽位");
        return;
    };
    if ndw::request_close(slot.hwnd).is_err() {
        push_desktop_toast(state, "error", "关闭请求失败（窗口可能已退出）");
    }
}

#[cfg(not(windows))]
fn execute_close_native(state: &mut crate::ui::session::DesktopSession, _slot_id: u64) {
    push_desktop_toast(state, "error", "原生窗口 dock 仅支持 Windows");
}

/// Plan 473 T6：槽位几何排水——本地逻辑矩形 → 屏幕物理坐标（桌面窗原点
/// + per-monitor DPI 缩放，§2 DPI 段），写入槽位客户区（标题条以下、边框
/// 环内缩，chrome 露出），回写 `slot_rect`。幂等：pending 为空且无带内
/// 重申旗标即零开销。
/// PLAN-709：z 序不变量改带内全量重申——几何写毕后按统一 z 序
/// （`native_slots_in_z_order`）`restack_slots` 一次成型（473 逐 pending
/// sink 的多槽勘误见 win32 restack 注）；`pending_native_restack` 旗标
/// （聚焦槽位置顶）单独驱动无几何重申。
/// Plan 494：hole_mode 下不变量翻转为桌面盖带顶（restack `desktop_above`）
/// + 桌面窗 Region 洞排除（`apply_hole_regions`，视觉+输入穿透）；
/// Region 失败自动回退 off（假洞 z 序重申 + 一行日志，storage 不回写）。
#[cfg(windows)]
pub(super) fn sync_native_geometry(state: &mut crate::ui::session::DesktopSession) {
    use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
    use crate::ui::native_dock::{win32 as ndw, CoordMapper, LogicalRect};

    let hole_mode = state.desktop.hole_mode;
    let restack_requested = state
        .host
        .as_ref()
        .map(|h| h.wm.pending_native_restack)
        .unwrap_or(false);
    // PLAN-709：workspace 隐现 tick（失配才写——稳态零 SetWindowPos）。
    // 返回 true = 可见集变化（随后 restack 重申带序）。
    if sync_native_workspace_visibility(state) {
        state
            .host
            .as_mut()
            .map(|h| h.wm.pending_native_restack = true);
    }
    let restack_requested = restack_requested
        || state
            .host
            .as_ref()
            .map(|h| h.wm.pending_native_restack)
            .unwrap_or(false);
    if state
        .host
        .as_ref()
        .map(|h| h.wm.pending_native_geometry.is_empty())
        .unwrap_or(true)
    {
        if restack_requested {
            // 无几何待排、仅带内 z 序重申（聚焦置顶臂）。
            if let Some(host) = state.host.as_mut() {
                host.wm.pending_native_restack = false;
            }
            if let Some(desktop_hwnd) = ndw::find_largest_own_window() {
                restack_band(state, desktop_hwnd, hole_mode);
            }
        }
        // 无几何待排也要维持 Region 洞（undock/回收后的洞收缩）。
        if hole_mode {
            refresh_hole_regions(state);
        }
        return;
    }
    let desktop_hwnd = match ndw::find_largest_own_window() {
        Some(h) => h,
        None => return,
    };
    let scale = ndw::dpi_scale_of(desktop_hwnd);
    let Some(frame) = ndw::get_bounds(desktop_hwnd) else {
        return;
    };
    let mapper = CoordMapper {
        origin_x: frame.x as f64,
        origin_y: frame.y as f64,
        scale,
    };
    let pending = {
        let host = state.host.as_mut().expect("desktop checked");
        host.wm.pending_native_restack = false;
        host.wm.drain_native_geometry()
    };
    for (id, local) in pending {
        // 客户区 = 槽位去掉顶部标题条与左右/底部边框环（chrome 露出）。
        let client = LogicalRect {
            x: local.x + BORDER,
            y: local.y + TITLEBAR_H,
            w: (local.width - 2.0 * BORDER).max(1.0),
            h: (local.height - TITLEBAR_H - BORDER).max(1.0),
        };
        let phys = mapper.local_to_screen(client);
        let Some(hwnd) = state
            .host
            .as_ref()
            .and_then(|h| h.wm.native_slots.get(&id))
            .map(|s| s.hwnd)
        else {
            continue;
        };
        let _ = ndw::set_bounds(hwnd, phys);
        if let Some(slot) = state
            .host
            .as_mut()
            .expect("desktop checked")
            .wm
            .native_slots
            .get_mut(&id)
        {
            slot.slot_rect = phys;
        }
    }
    // PLAN-709：带内 z 序全量重申（几何写毕一次成型；473 逐槽 sink 勘误）。
    restack_band(state, desktop_hwnd, hole_mode);
    if hole_mode {
        refresh_hole_regions(state);
    }
}

/// PLAN-709：按统一 z 序（`native_slots_in_z_order`，bottom→top）对全部
/// Docked 槽位做带内重申（真洞=桌面盖带顶，假洞=桌面垫带底）。
#[cfg(windows)]
fn restack_band(
    state: &crate::ui::session::DesktopSession,
    desktop_hwnd: crate::ui::native_dock::NativeHwnd,
    hole_mode: bool,
) {
    use crate::ui::native_dock::win32 as ndw;
    let Some(host) = state.host.as_ref() else {
        return;
    };
    let order: Vec<crate::ui::native_dock::NativeHwnd> = host
        .wm
        .native_slots_in_z_order()
        .into_iter()
        .filter_map(|id| host.wm.native_slots.get(&id).map(|s| s.hwnd))
        .collect();
    if order.is_empty() {
        return;
    }
    let _ = ndw::restack_slots(desktop_hwnd, &order, hole_mode);
}

/// PLAN-709：workspace 隐现排水——Docked 槽位按 `slot.workspace ==
/// current_workspace` 失配写 SW_HIDE/SW_SHOW（IsWindowVisible 探测，
/// 稳态零调用）；band restack 序只含当前分区槽位（隐分区 HWND 已藏，
/// 不入链）。返回可见集是否变化（调用方置 restack 旗标）。
#[cfg(windows)]
fn sync_native_workspace_visibility(state: &mut crate::ui::session::DesktopSession) -> bool {
    use crate::ui::native_dock::{win32 as ndw, SlotState};
    let Some(host) = state.host.as_ref() else {
        return false;
    };
    let current = host.wm.current_workspace;
    let targets: Vec<(crate::ui::native_dock::NativeHwnd, bool)> = host
        .wm
        .native_slots
        .values()
        .filter(|s| s.state == SlotState::Docked)
        .map(|s| (s.hwnd, s.workspace == current))
        .collect();
    drop(host);
    let mut changed = false;
    for (hwnd, want_visible) in targets {
        let visible = ndw::is_visible(hwnd);
        if want_visible && !visible {
            let _ = ndw::show_window(hwnd, ndw::ShowMode::Show);
            changed = true;
        } else if !want_visible && visible {
            let _ = ndw::show_window(hwnd, ndw::ShowMode::Hide);
            changed = true;
        }
    }
    changed
}

#[cfg(not(windows))]
fn sync_native_workspace_visibility(_state: &mut crate::ui::session::DesktopSession) -> bool {
    false
}

/// Plan 494：桌面窗 Region 洞排除重建——洞集 = 全部 docked 槽位客户区
/// （`slot_rect`，屏幕物理域）。失败自动回退：hole_mode 置 off + 473 假洞
/// z 序全量重申 + 一行日志（下次启动重试，storage 不回写）。
#[cfg(windows)]
fn refresh_hole_regions(state: &mut crate::ui::session::DesktopSession) {
    use crate::ui::native_dock::win32 as ndw;
    let Some(desktop_hwnd) = ndw::find_largest_own_window() else {
        return;
    };
    let Some(frame) = ndw::get_bounds(desktop_hwnd) else {
        return;
    };
    refresh_hole_regions_at(state, desktop_hwnd, frame);
}

/// Plan 494：Region 重建核心（hwnd/矩形注入可测形态——T3 档测试以
/// stale hwnd 驱动 Err 路径，兑现"回退路径实测一次"）。
#[cfg(windows)]
pub(super) fn refresh_hole_regions_at(
    state: &mut crate::ui::session::DesktopSession,
    desktop_hwnd: crate::ui::native_dock::NativeHwnd,
    frame: crate::ui::native_dock::Rect,
) {
    use crate::ui::native_dock::win32 as ndw;
    let holes: Vec<crate::ui::native_dock::Rect> = state
        .host
        .as_ref()
        .map(|h| {
            h.wm.native_slots
                .values()
                .filter(|s| s.state == crate::ui::native_dock::SlotState::Docked)
                .map(|s| s.slot_rect)
                .collect()
        })
        .unwrap_or_default();
    if ndw::apply_hole_regions(desktop_hwnd, frame, &holes).is_err() {
        crate::syslog!(
            SyslogLevel::Warn,
            "host",
            "[session] hole region apply failed (fallback to fake-hole, Plan 494)"
        );
        state.desktop.hole_mode = false;
        // 假洞 z 序全量重申（原生盖桌面；PLAN-709 统一 z 序 restack 一次成型）。
        let slots: Vec<crate::ui::native_dock::NativeHwnd> = state
            .host
            .as_ref()
            .map(|h| {
                h.wm.native_slots_in_z_order()
                    .into_iter()
                    .filter_map(|id| h.wm.native_slots.get(&id).map(|s| s.hwnd))
                    .collect()
            })
            .unwrap_or_default();
        let _ = ndw::restack_slots(desktop_hwnd, &slots, false);
        let _ = ndw::apply_hole_regions(desktop_hwnd, frame, &[]);
    }
}

#[cfg(not(windows))]
fn sync_native_geometry(_state: &mut crate::ui::session::DesktopSession) {}

/// PLAN-709：槽位交互的 DPI 折算系数（min_size_est 物理域 → 逻辑域；
/// 无桌面窗可查时回退 1.0）。
#[cfg(windows)]
pub(super) fn native_slot_dpi_scale() -> f32 {
    crate::ui::native_dock::win32::find_largest_own_window()
        .map(|h| crate::ui::native_dock::win32::dpi_scale_of(h) as f32)
        .unwrap_or(1.0)
}

#[cfg(not(windows))]
fn native_slot_dpi_scale() -> f32 {
    1.0
}

/// Plan 473 T6：WinEventHook 事件处理（B7 回收 / C4 拖走 undock；
/// LocationChange 在自同步与最小化时不误判）。未命中槽位（噪声/他窗）
/// 静默忽略；486 起未 docked 窗口改走 [`drive_drag_watch`] 手势会话。
#[cfg(windows)]
pub(super) fn handle_native_slot_event(
    state: &mut crate::ui::session::DesktopSession,
    hwnd_value: isize,
    kind: crate::ui::native_dock::NativeSlotEventKind,
) {
    use crate::ui::native_dock::{
        win32 as ndw, NativeSlotEventKind, SlotEvent, USER_DRAG_THRESHOLD_PX,
    };
    let docked = state
        .host
        .as_ref()
        .map(|h| h.wm.native_slot_id_of_hwnd(hwnd_value).is_some())
        .unwrap_or(false);
    if !docked {
        drive_drag_watch(state, hwnd_value, kind);
        return;
    }
    let Some(id) = state
        .host
        .as_ref()
        .and_then(|h| h.wm.native_slot_id_of_hwnd(hwnd_value))
    else {
        return;
    };
    match kind {
        NativeSlotEventKind::LocationChange | NativeSlotEventKind::MoveSizeEnd => {
            // C4：位置变化后偏离槽位超阈值判用户拖走（最小化/自同步不误判）。
            let info = state.host.as_ref().and_then(|h| {
                let s = h.wm.native_slots.get(&id)?;
                Some((s.hwnd, s.slot_rect, s.title_cache.clone()))
            });
            let Some((hwnd, slot_rect, title)) = info else {
                return;
            };
            if ndw::is_minimized(hwnd) {
                return;
            }
            let Some(cur) = ndw::get_bounds(hwnd) else {
                return;
            };
            let threshold = (USER_DRAG_THRESHOLD_PX as f64 * ndw::dpi_scale_of(hwnd)) as i32;
            if crate::ui::native_dock::detect_user_drag(cur, slot_rect, threshold) {
                if undock_native_slot(state, id) {
                    let layout = state.host.as_ref().map(|h| h.wm.layout).unwrap_or_default();
                    state.wm_set_layout(layout);
                    sync_native_geometry(state);
                    push_desktop_toast(state, "success", &format!("已恢复 {title}"));
                }
            }
        }
        NativeSlotEventKind::Destroy => {
            // B7：目标自毁 → 回收槽位并 relayout（不留僵尸框）。
            let removed = state
                .host
                .as_mut()
                .map(|h| h.wm.advance_native_slot(id, SlotEvent::TargetClosed).1)
                .unwrap_or(false);
            if removed {
                if let Some(host) = state.host.as_mut() {
                    host.wm.native_slot_local_rects.remove(&id);
                }
                let layout = state.host.as_ref().map(|h| h.wm.layout).unwrap_or_default();
                state.wm_set_layout(layout);
                sync_native_geometry(state);
                push_desktop_toast(state, "success", "原生窗口已关闭，槽位已回收");
            }
        }
        NativeSlotEventKind::MinimizeStart | NativeSlotEventKind::MinimizeEnd => {}
        // 已 docked 窗口的拖动起点：C4 拖走判定在 MoveSizeEnd 读回几何时做；
        // START 只驱动未 docked 窗口的 DragWatch 手势会话（486，session 侧接线）。
        NativeSlotEventKind::MoveSizeStart => {}
        // PLAN-709：OS 前台切换 → WM focused 跟随（AC-04）。命中 docked
        // 槽位 = 伪 Wid 进 focused（WM 侧置顶 + restack 旗标，带内重申随
        // 排水拍）。边界裁定：未命中槽位的前台切换（他窗/桌面自身）不改
        // WM focused——桌面自身前台 = iced 域点击聚焦已自管；他窗前台维持
        // 原焦点（避免 alt-tab 路过他窗时 WM 焦点抖动），473「native 焦点
        // 域在 OS 层」语义的窄化收口。自身 SetForegroundWindow（focus 臂）
        // 回波 = 幂等重申，无环路。
        NativeSlotEventKind::Foreground => {
            state.wm_focus_native_slot(id);
        }
    }
}

/// 桌面窗物理↔逻辑坐标映射 + 全帧物理矩形（指针入桌面判定域；无桌面窗/
/// headless → None）。
#[cfg(windows)]
pub(super) fn drag_mapper() -> Option<(
    crate::ui::native_dock::CoordMapper,
    crate::ui::native_dock::Rect,
)> {
    use crate::ui::native_dock::{win32 as ndw, CoordMapper, Rect};
    let desktop = ndw::find_largest_own_window()?;
    let scale = ndw::dpi_scale_of(desktop);
    let frame = ndw::get_bounds(desktop)?;
    let mapper = CoordMapper {
        origin_x: frame.x as f64,
        origin_y: frame.y as f64,
        scale,
    };
    Some((mapper, Rect::new(frame.x, frame.y, frame.w, frame.h)))
}

#[cfg(not(windows))]
fn drag_mapper() -> Option<(
    crate::ui::native_dock::CoordMapper,
    crate::ui::native_dock::Rect,
)> {
    None
}

/// Plan 505 D（债 P488-D4）：当前拖出会话代号（win32 实态；非 Windows/
/// 未开 native-dnd 恒 0——锚定永不触发，交付回退 v1 焦点语义）。
pub(super) fn dnd_drag_generation() -> u64 {
    #[cfg(all(windows, feature = "native-dnd"))]
    {
        crate::ui::native_dnd::win32::drag_session_generation()
    }
    #[cfg(not(all(windows, feature = "native-dnd")))]
    {
        0
    }
}

/// Plan 505 D：dispatch 环后锚定判定——代际变化（本次 dispatch 内发起并
/// 完成了一次 DoDragDrop）即锚定发起方 App 为 on_dnd_finished 交付目标。
pub(super) fn maybe_anchor_dnd_initiator(
    state: &mut crate::ui::session::DesktopSession,
    app: crate::ui::session::AppId,
    gen_before: u64,
    gen_after: u64,
) {
    if gen_after != gen_before {
        state.desktop.dnd_initiator = Some(app);
    }
}

/// Plan 505 D：on_dnd_finished 交付目标——发起时锚定（取走）> 完成时
/// 焦点 > primary（v1 回退序保持）。
pub(super) fn dnd_finished_target(
    state: &mut crate::ui::session::DesktopSession,
) -> Option<crate::ui::session::AppId> {
    state
        .desktop
        .dnd_initiator
        .take()
        .or_else(|| state.wm_focused_app())
        .or_else(|| state.primary_app())
}

/// Plan 488 T2：NativeDrop 抽取 → `on_native_drop` 载荷字段
/// {text: str|null, files: [str], image: {path,width,height}|null,
/// screen_x: int, screen_y: int, formats: [str]}（VM 堆记录编码见
/// [`crate::ui::vm_bridge::RecordValue`]）。
pub(super) fn native_drop_record_fields(
    data: &crate::ui::native_dnd::NativeDropData,
) -> Vec<(String, crate::ui::vm_bridge::RecordValue)> {
    use crate::ui::vm_bridge::RecordValue;
    vec![
        // 缺省恒空串（heap-record Str 字段与 nil 的 != 比较在 VM 侧破坏
        // 求值栈——488 T6 载具调试发现，空串哨兵绕开；债登记见计划）。
        (
            "text".into(),
            RecordValue::Str(data.text.clone().unwrap_or_default()),
        ),
        ("files".into(), RecordValue::StrList(data.files.clone())),
        // image 拍平三字段（嵌套记录的 Str 字段在 VM 侧取值不稳——
        // 宽高 Int 可达、path Str 产出错误码；扁平 Str 实证可达）。
        (
            "image_path".into(),
            RecordValue::Str(
                data.image
                    .as_ref()
                    .map(|(p, _, _)| p.clone())
                    .unwrap_or_default(),
            ),
        ),
        (
            "image_w".into(),
            match &data.image {
                Some((_, w, _)) => RecordValue::Int(*w as i32),
                None => RecordValue::Int(0),
            },
        ),
        (
            "image_h".into(),
            match &data.image {
                Some((_, _, h)) => RecordValue::Int(*h as i32),
                None => RecordValue::Int(0),
            },
        ),
        ("screen_x".into(), RecordValue::Int(data.screen_x)),
        ("screen_y".into(), RecordValue::Int(data.screen_y)),
        ("formats".into(), RecordValue::StrList(data.formats.clone())),
    ]
}

/// Plan 488：宿主注入面 —— App 级原生事件（on_native_drop /
/// on_native_paste / on_dnd_finished）经既有 handler 管线直注（472/479
/// 同型：bridge.call_handler；未声明 handler 的 App 静默忽略）。
pub(super) fn inject_native_event(
    state: &mut crate::ui::session::DesktopSession,
    app: crate::ui::session::AppId,
    event: &str,
    args: &[auto_val::Value],
) {
    if let Some(sess) = state.app_mut(app) {
        let _ = sess.component.bridge_mut().call_handler(event, args);
    }
}

/// Plan 488 T7：OS 剪贴板 → `on_native_paste` 载荷字段。读取优先级
/// text（418）→ files（485）→ image（485）；whichever 可用并存（形状与
/// on_native_drop 的 text/files/image 域一致）。
pub(super) fn clipboard_paste_record_fields() -> Vec<(String, crate::ui::vm_bridge::RecordValue)> {
    use crate::ui::vm_bridge::RecordValue;
    let text = crate::ui::clipboard::clipboard_get();

    #[cfg(windows)]
    let files = crate::ui::clipboard_native::clipboard_files_get();
    #[cfg(not(windows))]
    let files: Vec<String> = Vec::new();

    #[cfg(windows)]
    let image = crate::ui::clipboard_native::clipboard_image_get()
        .map(|t| (t.path.to_string_lossy().into_owned(), t.width, t.height));
    #[cfg(not(windows))]
    let image: Option<(String, u32, u32)> = None;

    vec![
        ("text".into(), RecordValue::Str(text.unwrap_or_default())),
        ("files".into(), RecordValue::StrList(files)),
        (
            "image_path".into(),
            RecordValue::Str(
                image
                    .as_ref()
                    .map(|(p, _, _)| p.clone())
                    .unwrap_or_default(),
            ),
        ),
        (
            "image_w".into(),
            match &image {
                Some((_, w, _)) => RecordValue::Int(*w as i32),
                None => RecordValue::Int(0),
            },
        ),
        (
            "image_h".into(),
            match &image {
                Some((_, _, h)) => RecordValue::Int(*h as i32),
                None => RecordValue::Int(0),
            },
        ),
    ]
}

/// Plan 486：拖入高亮落位——候选槽位物理矩形 → 会话逻辑字段（view 直绘）。
/// 物理域消息 [`crate::ui::session::DesktopEvent::NativeDragOver`] 与 DragWatch
/// 采样共用本入口；无桌面窗的 headless 形态按恒等退化（E2E 直注即所得）。
pub(super) fn set_native_drag_over(
    state: &mut crate::ui::session::DesktopSession,
    phys: Option<crate::ui::native_dock::Rect>,
) {
    let logical = phys.map(|r| {
        let (x, y, w, h) = if let Some((mapper, _)) = drag_mapper() {
            let l = mapper.screen_to_local(r);
            (l.x, l.y, l.w, l.h)
        } else {
            (r.x as f32, r.y as f32, r.w as f32, r.h as f32)
        };
        iced::Rectangle::new(
            iced::Point::new(x, y),
            iced::Size::new(w.max(1.0), h.max(1.0)),
        )
    });
    state.native_drag_over = logical;
}

/// 进程起点（DragWatch 节流时基）。
#[cfg(windows)]
fn drag_now_ms() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Plan 486：未 docked 窗口的拖入手势会话驱动（G1/G2 触发面）。事件语义：
/// `MOVESIZESTART` 起会话；被拖窗的 `LOCATIONCHANGE` 喂光标采样（指针入
/// 桌面 → 候选槽位高亮 / 出桌面 → 清除）；`MOVESIZEEND` 指针在桌面内 →
/// `DockCandidate` 转 `dock_native` 执行臂（复用 473），在外 → 丢弃会话；
/// `DESTROY` → 作废会话并清高亮。他窗噪声经 `watched_hwnd` 过滤。
#[cfg(windows)]
fn drive_drag_watch(
    state: &mut crate::ui::session::DesktopSession,
    hwnd_value: isize,
    kind: crate::ui::native_dock::NativeSlotEventKind,
) {
    use crate::ui::native_dock::{
        win32 as ndw, DragSample, DragWatchOutcome, LogicalRect, NativeHwnd, NativeSlotEventKind,
    };
    if !state.is_desktop() {
        return;
    }
    let hwnd = NativeHwnd(hwnd_value);
    match kind {
        NativeSlotEventKind::MoveSizeStart => {
            state.native_drag_watch.start(hwnd);
        }
        NativeSlotEventKind::LocationChange | NativeSlotEventKind::MoveSizeEnd => {
            if state.native_drag_watch.watched_hwnd() != Some(hwnd) {
                return; // 他窗噪声（会话只跟随被拖窗的事件流）
            }
            let Some((mapper, desktop_rect)) = drag_mapper() else {
                return;
            };
            let Some(pointer) = ndw::cursor_pos() else {
                return;
            };
            // 候选槽位（物理域）：与 dock 执行臂同源的级联占位（高亮即落点）。
            let logical = native_candidate_logical(state);
            let cell = mapper.local_to_screen(LogicalRect {
                x: logical.x,
                y: logical.y,
                w: logical.width,
                h: logical.height,
            });
            let cells = [cell];
            match kind {
                NativeSlotEventKind::LocationChange => {
                    match state.native_drag_watch.sample(
                        pointer,
                        desktop_rect,
                        &cells,
                        drag_now_ms(),
                    ) {
                        DragSample::Overlay(rect) => set_native_drag_over(state, rect),
                        DragSample::NoChange => {}
                    }
                }
                NativeSlotEventKind::MoveSizeEnd => {
                    match state.native_drag_watch.end(pointer, desktop_rect) {
                        Some(DragWatchOutcome::DockCandidate(h)) => {
                            set_native_drag_over(state, None);
                            execute_dock_native(
                                state,
                                crate::ui::session::NativeTarget::ByHwnd(h.0),
                            );
                        }
                        Some(DragWatchOutcome::Abandon) => {
                            set_native_drag_over(state, None);
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        NativeSlotEventKind::Destroy => {
            if state.native_drag_watch.watched_hwnd() == Some(hwnd) {
                state.native_drag_watch.reset();
                set_native_drag_over(state, None);
            }
        }
        NativeSlotEventKind::MinimizeStart | NativeSlotEventKind::MinimizeEnd => {}
        // PLAN-709：未 docked 窗口的前台切换与拖入手势会话无关（噪声）。
        NativeSlotEventKind::Foreground => {}
    }
}

#[cfg(not(windows))]
fn handle_native_slot_event(
    _state: &mut crate::ui::session::DesktopSession,
    _hwnd_value: isize,
    _kind: crate::ui::native_dock::NativeSlotEventKind,
) {
}

/// Plan 473 T6 / B8：桌面退出批量恢复——不吞窗口：全部 docked 槽位恢复
/// pre-dock bounds/样式/圆角。注册表随进程退出回收，无需逐项出表。
#[cfg(windows)]
pub(super) fn restore_all_native_slots(state: &mut crate::ui::session::DesktopSession) {
    use crate::ui::native_dock::win32 as ndw;
    let slots: Vec<_> = state
        .host
        .as_ref()
        .map(|h| {
            h.wm.native_slots
                .values()
                .map(|s| (s.hwnd, s.pre_dock_bounds, s.pre_dock_style))
                .collect()
        })
        .unwrap_or_default();
    for (hwnd, bounds, style) in slots {
        let _ = ndw::set_bounds(hwnd, bounds);
        if style != 0 {
            let _ = ndw::restore_chrome(hwnd, style);
        }
        let _ = ndw::restore_corner_preference(hwnd);
    }
    // Plan 494：桌面退出清 Region 洞（窗口将销毁，复位全窗形状——不留
    // 带洞僵尸窗口在 OS 层）。
    if state.desktop.hole_mode {
        if let Some(desktop_hwnd) = ndw::find_largest_own_window() {
            if let Some(frame) = ndw::get_bounds(desktop_hwnd) {
                let _ = ndw::apply_hole_regions(desktop_hwnd, frame, &[]);
            }
        }
    }
}

#[cfg(not(windows))]
fn restore_all_native_slots(_state: &mut crate::ui::session::DesktopSession) {}

/// LaunchApp 执行体（463 T4 主体抽出；472 T4 起被 LaunchApp/ActivateApp
/// 两臂共用）。失败转 toast + 占位页（Design 24 §6.5）。
/// PLAN-016 T-07（协议 v1.7）：open_with 执行体——注册表校验（opens 声明
/// 面未命中即拒）→ 未运行 launch 后向目标 App state 写 `auto_open_path` /
/// 已运行聚焦 + 同款写入；目标 App（041/031 接收臂）Tick 消费。写入后
/// view_dirty 即标——目标下一帧可见。App 未声明 `auto_open_path` 时写入
/// 静默无效（capability 自声明，无害）。
fn execute_open_with(state: &mut crate::ui::session::DesktopSession, app_id: &str, path: &str) {
    // 注册表校验：app 存在 + opens 声明面（空 = 不参与校验，放行）。
    let ext = std::path::Path::new(path)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let opens_ok = state
        .desktop
        .app_resolver
        .as_ref()
        .and_then(|r| r(app_id))
        .map(|spec| {
            spec.opens.is_empty() || spec.opens.iter().any(|o| o.eq_ignore_ascii_case(&ext))
        });
    match opens_ok {
        None => {
            push_notification(state, "error", &format!("未知应用: {app_id}"));
            return;
        }
        Some(false) => {
            push_notification(
                state,
                "error",
                &format!("{app_id} 未声明打开 {ext} 文件（pac opens）"),
            );
            return;
        }
        Some(true) => {}
    }
    // 已运行窗（registry_id 命中）→ 聚焦 + 写入；未运行 → launch + 写入。
    let running = state.host.as_ref().and_then(|h| {
        h.wm.wins
            .iter()
            .find(|(_, v)| v.registry_id.as_deref() == Some(app_id))
            .map(|(wid, v)| (*wid, v.app))
    });
    let delivered = match running {
        Some((wid, app)) => {
            state.wm_focus(wid);
            deliver_open_arg(state, app, path)
        }
        None => match state.launch_app(app_id) {
            Ok(wid) => {
                let app = state
                    .host
                    .as_ref()
                    .and_then(|h| h.wm.wins.get(&wid))
                    .map(|v| v.app);
                match app {
                    Some(app) => deliver_open_arg(state, app, path),
                    None => false,
                }
            }
            Err(err) => {
                push_notification(state, "error", &format!("启动 {app_id} 失败: {err}"));
                false
            }
        },
    };
    if !delivered {
        // 目标未声明 auto_open_path（非接收臂 App）——聚焦语义仍达成。
        push_notification(
            state,
            "info",
            &format!("已聚焦 {app_id}（目标未声明 auto_open_path 接收臂）"),
        );
    }
}

/// PLAN-016 T-07：向目标 App 写 `auto_open_path` + view_dirty。返回目标
/// 是否声明了接收臂（写成功 = true）。
fn deliver_open_arg(
    state: &mut crate::ui::session::DesktopSession,
    app: crate::ui::session::AppId,
    path: &str,
) -> bool {
    let Some(a) = state.apps.get_mut(&app) else {
        return false;
    };
    let ok = a
        .component
        .write_state("auto_open_path", auto_val::Value::str(path))
        .is_ok();
    if ok {
        *a.state.view_dirty.borrow_mut() = true;
    }
    ok
}

pub(super) fn execute_launch_app(state: &mut crate::ui::session::DesktopSession, name: &str) {
    // PLAN-041 T-03：崩溃围栏——VM front 初始化崩溃族（base 补链后可达、
    // master 期被链接失败掩蔽）：
    //   · 017-chat：widget 树/布局失配（timer 重建 × bounds operate 竞态 →
    //     iced container `layout.children().next().unwrap()` 崩桌面，三次复
    //     现）；
    //   · auto-term：Init future_all/race 空未来列表（handler_App_Init/Tick
    //     每 拍 崩、空闲 20 分钟 2.6 万行日志，复审走查实录；终端窗渲染但
    //     永不 tick）。
    // 两者待各自根修（债 P041-D1）后摘围栏。
    // PLAN-043 走查（2026-09-24）：围栏实验性放行 017 复测——桌面即崩
    // （iced_widget container.rs:291 unwrap，与 PLAN-041 三次复现同点）。
    // **2026-09-25 根治**：iced_widget 本地补丁（crates/patches/iced_widget，
    // [patch.crates-io] 挂载）把 container.rs 布局空窗 unwrap 全族改良性
    // 降级——017 摘围栏放行（auto-term 无本补丁独立实证，暂留围栏观察）。
    // **2026-09-25 续**：auto-term 亦摘围栏实验——其 Init 已被
    // PLAN-025/026 tick-snapshot 聚合改造重写（计划期"future_all 空列表"
    // 形态在现行源码 grep 不到），若复现旧崩形再恢复围栏。
    const VM_LAUNCH_FENCE: [&str; 0] = [];
    let _ = VM_LAUNCH_FENCE;
    if VM_LAUNCH_FENCE.contains(&name) {
        push_notification(
            state,
            "error",
            &format!("launch `{name}` 失败: VM front 初始化崩溃围栏中（P041-D1 根修后放行）"),
        );
        show_launch_unavailable(state, name);
        return;
    }
    // 2026-10-10 用户裁定（小组件独立于 app 窗配套）：已有孵化 mini 会话
    // → 升格开窗（face/窗同会话零分家，`__dashboard_open` 臂同语义）——
    // 不再 launch 出第二会话（状态分家 + 后端计数翻倍）。无孵化 → 照常
    // launch（WinAct 面板/桌面图标/launcher 三入口同此汇集）。
    if let Some(hatched) = state.hatched_mini_of(name) {
        if let Err(err) = state.open_window_for_session(name, hatched) {
            eprintln!("[session] launch promote failed: {err}");
            return;
        }
        return;
    }
    match state.launch_app(name) {
        // PLAN-002 N3（用户复核裁定 2026-09-09）：启动成功不再发通知——
        // toast 层逐动态视图面各渲染一份（shell + 每 app 窗，见 412 toast
        // 双层 Stack），启动成功弹满屏属噪音；通知中心历史/未读一并不再
        // 入（launch 成功非常规事件）。失败臂保留（Design 24 §6.5）。
        Ok(_wid) => {}
        Err(err) => {
            push_notification(state, "error", &err);
            // Plan 463 T7：启动失败占位页（Design 24 §6.5）——
            // 可见反馈窗代替白屏；占位页自身构建失败则仅 toast。
            show_launch_unavailable(state, name);
        }
    }
}

/// 启动失败占位窗（Design 24 §6.5；PLAN-041 T-03 自 Err 臂提炼共用——
/// vue 门禁与启动失败同面）。占位页自身构建失败则仅 toast（通知已发）。
fn show_launch_unavailable(state: &mut crate::ui::session::DesktopSession, name: &str) {
    if let Ok(comp) = crate::ui::shell::build_launch_fallback(name) {
        let app_id = state.allocate_app(comp);
        let usable =
            crate::ui::layout::usable_rect(state.host_viewport(), state.desktop.dock_edges);
        let index = state.host.as_ref().map(|h| h.wm.wins.len()).unwrap_or(0);
        let size = iced::Size::new(usable.width * 0.4, usable.height * 0.4);
        let rect = crate::ui::layout::cascade_rect(index, size, usable);
        let wid = state.wm_add_win(app_id, format!("{name}（不可用）"), rect);
        state.wm_focus(wid);
    }
}

/// Plan 504：fit 测量单轮自重试上限——首帧界面未建好时 `__fit_measured`
/// 找不到 `aura_fit_root_*` 锚点会重新发起测量，超限放弃本轮（fit_pending
/// 保留，下个触发点重新武装；常态 1-2 帧内命中）。
/// PLAN-035 T-17：10 次（@400ms tick = 4s）在调试构建/高负载下不够——
/// 012-clock 冷启动首帧超 4s，fit 放弃后窗口停留默认尺寸（用户实测
/// 宽短窗）。提至 150 次（=60s）覆盖慢启动；命中即止无空转成本。
const FIT_MEASURE_MAX_RETRIES: u32 = 150;

/// Plan 512：fit 动态重测滞回阈值（px）——|Δw|、|Δh| 均 ≤ 此值忽略，
/// 防输入态抖动触发 resize 循环（待澄清⑤：T1 定稿初值 8，实证微调再改）。
const FIT_REMEASURE_THRESHOLD: f32 = 8.0;

/// Plan 512：fit 重测尺寸决策（纯函数，T1 单测靶）。
///
/// - 用户手动 resize 锁定的窗口 → `None`（v1 一次性锁定，待澄清③）；
/// - 双轴差值均 ≤ 滞回阈值 → `None`；
/// - 否则 `Some(测量值 clamp ≥200)`——增大与缩小双向跟随（如 011
///   Scientific ↔ Basic 切换）。
///
/// 帧合并不由本函数承担：单程闸 `fit_measure_in_flight` + ServiceTick
/// 节拍保证测量总在帧边界后对最新布局树取末值。
pub(super) fn decide_fit_resize(
    current: (f32, f32),
    measured: (f32, f32),
    user_locked: bool,
) -> Option<iced::Size<f32>> {
    if user_locked {
        return None;
    }
    let w = measured.0.max(200.0);
    let h = measured.1.max(200.0);
    if (w - current.0).abs() <= FIT_REMEASURE_THRESHOLD
        && (h - current.1).abs() <= FIT_REMEASURE_THRESHOLD
    {
        return None;
    }
    Some(iced::Size::new(w, h))
}

/// Plan 504：发起一次 fit 内容测量。LayoutCollector 操作布全窗口树
/// （daemon 下 Action::Widget 遍历所有界面），回执 `__fit_measured`
/// 路由到 `win` 所在 DM::Window 臂。
pub(super) fn fit_measure_task(win: iced::window::Id) -> iced::Task<crate::ui::session::DesktopMessage> {
    iced::advanced::widget::operate(crate::ui::iced::LayoutCollector::new()).map(move |bounds| {
        crate::ui::session::DesktopMessage::Window(
            win,
            IcedMessage {
                widget: String::new(),
                event: "__fit_measured".to_string(),
                input_value: Some(serde_json::to_string(&bounds).unwrap_or_default()),
            },
        )
    })
}

/// 宿主窗 resize 同步（desktop，`__window_resized` 臂本体；测试直驱）：
/// 宿主条目 window_size 落新值 → 虚拟窗全量同步但 **fit 窗跳过**（其
/// window_size = 内容尺寸，fit 迟滞重测与排布格内居中消费——2026-10-09
/// 用户实录：本笔全覆盖曾把日历内容尺寸污染成宿主尺寸，后续任意重排
/// fit 居中按污染值钳位退化铺满整格）→ 全场重排（apply_dock_edges_now
/// 同型：排布窗格跟随新 viewport，fit 窗在新格内再居中；此前 resize 后
/// 窗格停留旧尺寸，直到下一次 launch/切布局才被动重排）→ 快照随撤（裁剪
/// 区域全部 stale）+ 原生槽位几何排水。
pub(crate) fn sync_host_resized(state: &mut crate::ui::session::DesktopSession, size: iced::Size) {
    if let Some(host) = state.host.as_ref() {
        if let Some(entry) = state.windows.get(&host.window) {
            *entry.window_size.borrow_mut() = size;
        }
        for v in host.wm.wins.values() {
            if v.fit_enabled.get() {
                continue;
            }
            *v.window_size.borrow_mut() = size;
        }
    }
    for app in state.apps.values() {
        *app.state.view_dirty.borrow_mut() = true;
    }
    // 视口直接取消息携带的新尺寸（宿主条目 window_size 已同步；不绕
    // host_viewport 的条目查找，免测试/早期帧的回退默认值）。
    let viewport = iced::Rectangle::new(iced::Point::ORIGIN, size);
    let edges = state.desktop.dock_edges;
    if let Some(host) = state.host.as_mut() {
        crate::ui::layout::apply_layout(&mut host.wm, viewport, edges);
    }
    crate::ui::iced::snapshot::invalidate_all();
    sync_native_geometry(state);
}

/// Plan 504：`__fit_measured` 回执——按测量锚点 `aura_fit_root_<appid>` 的
/// 内容自然尺寸收缩目标窗口：独立模式 `window::resize` OS 窗；desktop 模式
/// 改虚拟窗矩形（chrome 外沿 = 内容 + TITLEBAR_H + 2·BORDER，clamp 进可用
/// 区，越界回挪）。锚点缺席（界面尚未建好）按上限自重试。
pub(super) fn apply_fit_measured(
    state: &mut crate::ui::session::DesktopSession,
    win: iced::window::Id,
    payload: Option<&str>,
) -> iced::Task<crate::ui::session::DesktopMessage> {
    state.desktop.fit_measure_in_flight.set(false);
    let bounds: std::collections::HashMap<String, (f32, f32, f32, f32)> = payload
        .and_then(|p| serde_json::from_str(p).ok())
        .unwrap_or_default();
    // P679-D1 诊断 trace（AUTO_FIT_TRACE=1）：测量回执全量 bounds——
    // fit 塌缩定位面（锚点量到什么/其余 widget 量到什么）。
    if std::env::var("AUTO_FIT_TRACE").as_deref() == Ok("1") {
        let mut rows: Vec<(&String, &(f32, f32, f32, f32))> = bounds.iter().collect();
        rows.sort_by(|a, b| a.0.cmp(b.0));
        for (k, (x, y, w, h)) in &rows {
            eprintln!("[fit-trace] {k} = ({x:.0},{y:.0}) {w:.0}x{h:.0}");
        }
    }
    let mut task = iced::Task::none();
    let mut measured_any = false;
    for (key, (_x, _y, w, h)) in &bounds {
        let Some(raw) = key.strip_prefix("aura_fit_root_") else {
            continue;
        };
        let Ok(appid) = raw.parse::<u64>() else {
            continue;
        };
        let app_id = crate::ui::session::AppId(appid);
        let content = iced::Size::new(w.max(200.0), h.max(200.0));
        let mut done = false;
        // 独立模式：OS 窗条目（app + fit_pending 双条件定位）。
        let os_win = state
            .windows
            .iter()
            .find(|(_, e)| e.app == app_id && e.fit_pending.get())
            .map(|(id, _)| *id);
        if let Some(os_win) = os_win {
            let entry = &state.windows[&os_win];
            entry.fit_pending.set(false);
            entry.fit_dirty.set(false);
            *entry.window_size.borrow_mut() = content;
            // Plan 512：程序化 resize 记录应用值（OS 回波豁免用）。
            *entry.fit_last_applied.borrow_mut() = Some(content);
            task = iced::window::resize(os_win, content);
            done = true;
        } else {
            // desktop：虚拟窗矩形（内容 + 标题条 + 边框 = chrome 外沿）。
            // 坐标系先取（viewport/edges 共享借阅在 as_mut 前终结）。
            let viewport = state.host_viewport();
            let edges = state.desktop.dock_edges;
            let usable = crate::ui::layout::usable_rect(viewport, edges);
            if let Some(host) = state.host.as_mut() {
                let hit = host
                    .wm
                    .wins
                    .iter()
                    .find(|(_, v)| v.app == app_id && v.fit_pending.get())
                    .map(|(wid, _)| *wid);
                if let Some(wid) = hit {
                    use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
                    let w = (content.width + 2.0 * BORDER).min(usable.width);
                    let h = (content.height + TITLEBAR_H + 2.0 * BORDER).min(usable.height);
                    {
                        let v = host.wm.wins.get_mut(&wid).expect("fit hit wid");
                        v.fit_pending.set(false);
                        v.fit_dirty.set(false);
                        *v.window_size.borrow_mut() = content;
                    }
                    if host.wm.layout != crate::ui::layout::LayoutMode::Free {
                        // 排布模式：矩形统一由 apply_layout 一轮写回——fit 窗
                        // 按内容外沿在排布格内居中（2026-10-09 用户裁定：日历
                        // 等小个子 app 窗格默认居中；此臂只更新 window_size）。
                        crate::ui::layout::apply_layout(&mut host.wm, viewport, edges);
                    } else {
                        // free：原位收缩（位置优先不变；右/下越界回挪）。
                        let v = host.wm.wins.get_mut(&wid).expect("fit hit wid");
                        let mut r = v.rect.borrow_mut();
                        r.width = w;
                        r.height = h;
                        r.x = r.x.max(usable.x).min(usable.x + usable.width - w);
                        r.y = r.y.max(usable.y).min(usable.y + usable.height - h);
                    }
                    done = true;
                }
            }
        }
        // Plan 512：动态重测臂——首测已完成的 fit 窗按滞回决策双向跟随
        // 内容尺寸（用户手动 resize 锁定的窗由 decide_fit_resize 判 Skip）。
        let measured_wh = (*w, *h);
        if !done {
            let os_re = state
                .windows
                .iter()
                .find(|(_, e)| {
                    e.app == app_id
                        && e.fit_enabled.get()
                        && !e.fit_pending.get()
                        && e.fit_dirty.get()
                })
                .map(|(id, _)| *id);
            if let Some(os_win) = os_re {
                let entry = &state.windows[&os_win];
                entry.fit_dirty.set(false);
                let cur = *entry.window_size.borrow();
                if let Some(size) = decide_fit_resize(
                    (cur.width, cur.height),
                    measured_wh,
                    entry.fit_user_locked.get(),
                ) {
                    *entry.window_size.borrow_mut() = size;
                    *entry.fit_last_applied.borrow_mut() = Some(size);
                    task = iced::window::resize(os_win, size);
                    done = true;
                }
            } else {
                // desktop 动态重测臂（坐标系先取，同首测臂）。
                let viewport = state.host_viewport();
                let edges = state.desktop.dock_edges;
                let usable = crate::ui::layout::usable_rect(viewport, edges);
                if let Some(host) = state.host.as_mut() {
                    let hit = host
                        .wm
                        .wins
                        .iter()
                        .find(|(_, v)| {
                            v.app == app_id
                                && v.fit_enabled.get()
                                && !v.fit_pending.get()
                                && v.fit_dirty.get()
                        })
                        .map(|(wid, _)| *wid);
                    if let Some(wid) = hit {
                        let content2 = {
                            let v = host.wm.wins.get_mut(&wid).expect("fit re hit wid");
                            v.fit_dirty.set(false);
                            let cur = *v.window_size.borrow();
                            decide_fit_resize(
                                (cur.width, cur.height),
                                measured_wh,
                                v.fit_user_locked.get(),
                            )
                        };
                        if let Some(content2) = content2 {
                            use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
                            let w = (content2.width + 2.0 * BORDER).min(usable.width);
                            let h =
                                (content2.height + TITLEBAR_H + 2.0 * BORDER).min(usable.height);
                            {
                                let v = host.wm.wins.get_mut(&wid).expect("fit re hit wid");
                                *v.window_size.borrow_mut() = content2;
                            }
                            if host.wm.layout != crate::ui::layout::LayoutMode::Free {
                                // 排布模式：再居中走 apply_layout 统一写回（同首测臂）。
                                crate::ui::layout::apply_layout(&mut host.wm, viewport, edges);
                            } else {
                                // free：原位收缩（位置优先不变；右/下越界回挪）。
                                let v = host.wm.wins.get_mut(&wid).expect("fit re hit wid");
                                let mut r = v.rect.borrow_mut();
                                r.width = w;
                                r.height = h;
                                r.x = r.x.max(usable.x).min(usable.x + usable.width - w);
                                r.y = r.y.max(usable.y).min(usable.y + usable.height - h);
                            }
                            done = true;
                        }
                    }
                }
            }
        }
        if done {
            measured_any = true;
            if let Some(app) = state.apps.get(&app_id) {
                *app.state.view_dirty.borrow_mut() = true;
            }
        }
    }
    if measured_any {
        state.desktop.fit_measure_retries.set(0);
        // desktop：vwin 矩形已变——任务栏投影即时刷新（同 WM 变更惯例）。
        if state.is_desktop() {
            sync_shell_windows(state);
        }
        return task;
    }
    // 锚点缺席：界面尚未建好 → 自重试；超限仅放弃本轮——fit_pending 保留，
    // 下个 __window_resized / ServiceTick 触发重新武装（计数清零）。
    if state.has_fit_pending() {
        let n = state.desktop.fit_measure_retries.get() + 1;
        state.desktop.fit_measure_retries.set(n);
        if n <= FIT_MEASURE_MAX_RETRIES {
            state.desktop.fit_measure_in_flight.set(true);
            return fit_measure_task(win);
        }
        crate::syslog!(
            SyslogLevel::Warn,
            "host",
            "[session] fit measure: anchor missing after {FIT_MEASURE_MAX_RETRIES} retries; \
             keeping fit_pending for the next trigger (plan-504)"
        );
    }
    iced::Task::none()
}

/// Plan 472 T4：dock 数据级配置 → 布局预留边（协议 v1 §6：storage 键
/// `shell.dock.position` top/bottom、`shell.dock.enabled`；缺席回退 pack
/// 默认 bottom/48）。boot 期读一次写 `DesktopState.dock_edges`（v1 不做
/// 运行时热更；auto-os-config settings 写手为 shell-track M4）。
pub(crate) fn desktop_dock_edges(
    cfg: &crate::ui::desktop_config::DesktopConfig,
) -> crate::ui::layout::ReservedEdges {
    let mut edges = crate::ui::layout::ReservedEdges::taskbar();
    if !cfg.dock_enabled {
        return crate::ui::layout::ReservedEdges::default();
    }
    if cfg.dock_position == "top" {
        edges.bottom = 0.0;
        edges.top = crate::ui::layout::TASKBAR_HEIGHT;
    }
    edges
}

/// Plan 494：真洞模式 boot 读入（storage `shell.native.hole`，"true" 开；
/// 缺席/坏值 = off——坏配置不炸桌面，472 dock 配置同型）。
pub(super) fn load_native_hole_mode() -> bool {
    auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.native.hole").as_deref()
        == Some("true")
}

/// Plan 508 G4：远程 WS token 读入（storage `shell.remote.token`；空值/
/// 缺席 = None 不监听——缺省拒绝，远程面零暴露）。
pub(super) fn load_remote_token() -> Option<String> {
    auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.remote.token")
        .filter(|t| !t.trim().is_empty())
}

/// Plan 540 T2：dock pinned 消费点唯一化——boot 期直接取
/// `session.desktop.config.dock_pinned`（单源 config.at 装载 + 旧键迁移
/// 已在 `DesktopConfig::load()` 完成；`load_dock_pinned` storage 读退役）。

/// Plan 490 T3：热键覆盖读入（storage `shell.keys.<action>` > 内置默认）。
/// 宿主侧无键枚举面——逐个读已知动作位（11 个），缺席 = 不覆盖；坏值由
/// `HotkeyTable::apply_override` 静默拒绝（缺省回退——坏配置不炸桌面，
/// 472 dock 配置同型）。
pub(super) fn load_hotkey_overrides() -> Vec<(String, String)> {
    use crate::ui::session::HotkeyAction;
    let mut out = Vec::new();
    for action in [
        HotkeyAction::ExitDesktop,
        HotkeyAction::CycleSwitcher,
        HotkeyAction::SummonLauncher,
        HotkeyAction::SetLayoutGrid,
        HotkeyAction::SetLayoutStack,
        HotkeyAction::SetLayoutFree,
        HotkeyAction::WorkspaceNext,
        HotkeyAction::WorkspacePrev,
        HotkeyAction::SendToNext,
        HotkeyAction::SendToPrev,
        HotkeyAction::Paste,
        HotkeyAction::CycleWindow,
    ] {
        let key = format!("shell.keys.{}", action.storage_suffix());
        if let Some(v) = auto_lang_core::vm::ffi::stdlib::storage_host_read(&key) {
            if !v.trim().is_empty() {
                out.push((action.storage_suffix().to_string(), v));
            }
        }
    }
    out
}

/// Plan 472 T5：把 pinned 表解析为 {id,icon} Obj 数组注入 shell
/// `__dock_pinned`（icon 自注册表实时查，缺省回退 "app-window"）。boot 期
/// registry scan 之后调用；pinned 未运行条目也入列（dock 固定区常驻）。
/// PLAN-012 O2：`__dock_pinned` 条目构建（{id,icon,running}）——
/// `running` = 该 app 当前有非隐藏运行窗（"1"/""），pinned 图标灰条判据
/// （原 shell 侧 `__wm_running.contains(...)` 为 view 条件方法调用死点，
/// O2 实机探针定性）。inject_dock_pinned（pin/unpin 即时臂）与
/// sync_shell_windows（fp 差分刷新臂，窗开合即刷新）共用本构建，两写者
/// 同形幂等。
fn dock_pinned_objs(state: &crate::ui::session::DesktopSession) -> Vec<auto_val::Value> {
    state
        .desktop
        .dock_pinned
        .iter()
        .map(|id| {
            let icon = state
                .desktop
                .registry_entries
                .iter()
                .find(|e| &e.id == id)
                .map(|e| e.icon.clone())
                .unwrap_or_else(|| "app-window".to_string());
            let running = state
                .host
                .as_ref()
                .map(|host| {
                    host.wm
                        .wins
                        .values()
                        .any(|v| !v.hidden.get() && v.registry_id.as_deref() == Some(id.as_str()))
                })
                .unwrap_or(false);
            auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                ("id", auto_val::Value::Str(id.clone().into())),
                ("icon", auto_val::Value::Str(icon.into())),
                (
                    "running",
                    auto_val::Value::Str(if running { "1" } else { "".into() }.into()),
                ),
            ])))
        })
        .collect()
}

pub(super) fn inject_dock_pinned(state: &mut crate::ui::session::DesktopSession) {
    let Some(shell) = state.desktop.shell_app else {
        return;
    };
    let pinned = dock_pinned_objs(state);
    let Some(app) = state.apps.get_mut(&shell) else {
        return;
    };
    let _ = app.component.write_state_vec("__dock_pinned", pinned);
    *app.state.view_dirty.borrow_mut() = true;
}

/// PLAN-712 r2 T-14：分主题默认壁纸（用户钦定对：**深 `purple.png` /
/// 浅 `songyu.png`**），置于解析出的壁纸目录下（与目录探测默认同源——
/// stella-os 素材目录，见 `wallpapers_dir_or_default`）。文件缺席 =
/// 无主题默认（None → 走下一档目录首图）。
fn wallpapers_theme_default_image(
    cfg: &crate::ui::desktop_config::DesktopConfig,
) -> Option<String> {
    let dir = wallpapers_dir_or_default(cfg)?;
    let name = if cfg.dark_theme {
        "purple.png"
    } else {
        "songyu.png"
    };
    let p = dir.join(name);
    p.is_file().then(|| p.to_string_lossy().to_string())
}

/// Plan 496 M5 + Plan 540 T2 + PLAN-712 r2 T-13/T-14：boot 壁纸解析（单源
/// `config.wallpaper_path`）。**优先级（T-13 用户裁定「任何时候打开都
/// 加载默认壁纸」+ T-14 钦定默认对）**：存在图片路径 > 分主题默认图
/// （深 `purple.png` / 浅 `songyu.png`）> 壁纸目录首图 > `#hex`/`builtin:`
/// 色值 > [`crate::ui::session::DESKTOP_WALLPAPER_DEFAULT`]。纯色/builtin
/// 不再短路图片回退——它们是「无图片壁纸」的表述，机器有壁纸目录时
/// boot 面上默认图优先（PLAN-526 T19 目录首图语义延伸到色值分支）；
/// 无目录的机器色值仍直传（便携/极简场景诚实降级）。
pub(super) fn load_desktop_wallpaper(cfg: &crate::ui::desktop_config::DesktopConfig) -> String {
    use crate::ui::session::DESKTOP_WALLPAPER_DEFAULT;
    let v = cfg.wallpaper_path.trim().to_string();
    let color_like = v.starts_with('#') || v.starts_with("builtin:");
    // ① 图片路径（存在即用）。
    if !v.is_empty() && !color_like && std::path::Path::new(&v).is_file() {
        return v;
    }
    // ② 分主题默认壁纸（T-14 钦定对：深 purple.png / 浅 songyu.png）。
    if let Some(d) = wallpapers_theme_default_image(cfg) {
        return d;
    }
    // ③ 壁纸目录首图（config dir 字段 → env → stella 素材目录探测）。
    if let Some(dir) = wallpapers_dir_or_default(cfg) {
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| {
                            let e = e.to_ascii_lowercase();
                            e == "jpg" || e == "jpeg" || e == "png"
                        })
                        .unwrap_or(false)
            })
            .collect();
        files.sort();
        if let Some(first) = files.first() {
            return first.to_string_lossy().to_string();
        }
    }
    // ④ 色值直传（仅当目录不可用——有目录时色值不再阻塞默认壁纸）。
    if !v.is_empty() && color_like {
        return v;
    }
    // ⑤ 内置兜底。
    DESKTOP_WALLPAPER_DEFAULT.to_string()
}

/// PLAN-526 T19 + Plan 540 T2：壁纸目录解析链——单源 `config.wallpapers_dir`
/// → env `AUTO_DESKTOP_WALLPAPERS_DIR` → 探测 stella-os 素材目录（机器
/// 便利默认，存在即用；目录不存在/未配置 = None，scan_wallpapers_dir
/// 与缺省壁纸链共用）。
pub(super) fn wallpapers_dir_or_default(
    cfg: &crate::ui::desktop_config::DesktopConfig,
) -> Option<std::path::PathBuf> {
    let d = cfg.wallpapers_dir.trim().to_string();
    if !d.is_empty() {
        return Some(std::path::PathBuf::from(d));
    }
    if let Ok(d) = std::env::var("AUTO_DESKTOP_WALLPAPERS_DIR") {
        if !d.trim().is_empty() {
            return Some(std::path::PathBuf::from(d));
        }
    }
    let stella = std::path::PathBuf::from(r"D:\Down\stella-os\wallpapers");
    if stella.is_dir() {
        return Some(stella);
    }
    None
}

/// Plan 496 M5：boot 桌面条目读入——id 逗号串解析（自定义
/// `shell.desktop.icons` / 排除 `shell.desktop.hidden`，
/// `load_dock_pinned` 同形：空段剔除、缺席空表）。
pub(super) fn load_desktop_id_list(key: &str) -> Vec<String> {
    auto_lang_core::vm::ffi::stdlib::storage_host_read(key)
        .map(|raw| {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// PLAN-012 W5 T11：桌面图标格子分配——storage `shell.desktop.positions`
/// （追加式 "id=c,r,..." csv，**last-wins** 解析——shell 落子为追加写）。
/// 已定位图标占据其 (c,r)；未定位图标按既有 order 行主序填充首个空格；
/// 输出行主序全格子表（空格 = spacer 占位）。返回 (cells, ids, cs, rs)：
/// cells = Obj 数组（view 渲染），ids/cs/rs = 平行字符串列表（handler
/// 下标读——B12 规避，IconDropOn 交换换位数据面）。
pub(super) fn desktop_icon_cells(
    order: &[(String, &str)],
    hidden: &[String],
    registry: &[crate::ui::app_registry::AppRegistryEntry],
    layout_key: Option<&str>,
    rows: usize,
) -> (
    Vec<auto_val::Value>,
    Vec<String>,
    Vec<auto_val::Value>,
    Vec<auto_val::Value>,
) {
    const COLS: usize = 8;
    let visible: Vec<&(String, &str)> = order
        .iter()
        .filter(|(id, _)| !hidden.contains(id))
        .collect();
    // PLAN-019 T-06：布局键控读（当前壁纸键 → 缺席回退缺省底稿）。
    let positions = load_desktop_positions_for(layout_key);
    // ① 已定位：last-wins 解析的表中取 (c,r)。
    let mut placed: Vec<(usize, usize, &&(String, &str))> = Vec::new();
    let mut free: Vec<&&(String, &str)> = Vec::new();
    for e in &visible {
        match positions.iter().rev().find(|(id, _)| *id == e.0) {
            Some((_, (c, r))) => placed.push((*c, *r, e)),
            None => free.push(e),
        }
    }
    // ② 未定位：**列主序**填首个空格（2026-09-15 用户裁定——纵向优先，
    // 左列自上而下占满后再排下一列；跳过已占位）。rows 下限保证容量。
    let rows = rows.max((visible.len() + COLS - 1) / COLS).max(1);
    let mut taken: std::collections::BTreeSet<usize> =
        placed.iter().map(|&(c, r, _)| r * COLS + c).collect();
    for e in &free {
        let mut k = 0usize;
        loop {
            // 列主序序数 k → (c, r) = (k / rows, k % rows)。
            let (c, r) = (k / rows, k % rows);
            let linear = r * COLS + c;
            if !taken.contains(&linear) {
                taken.insert(linear);
                placed.push((c, r, e));
                break;
            }
            k += 1;
        }
    }
    // ③ 行主序输出（spacer 填空格）。
    placed.sort_by_key(|&(c, r, _)| (r, c));
    let mut cells = Vec::new();
    let mut ids = Vec::new();
    let mut cs = Vec::new();
    let mut rs = Vec::new();
    let mut cursor = 0usize;
    for &(c, r, e) in &placed {
        let linear = r * COLS + c;
        let id = e.0.clone();
        let reg = registry.iter().find(|e2| e2.id == id);
        let icon = reg
            .map(|e2| e2.icon.clone())
            .unwrap_or_else(|| "app-window".to_string());
        let label = reg
            .map(|e2| e2.display_title().to_string())
            .unwrap_or_else(|| id.clone());
        let src = e.1.to_string();
        let color = crate::ui::app_registry::badge_color_for(&id);
        // PLAN-018-FU2：iconfile（真位图资产）= 满幅 tile 渲染旗标。
        let full = if icon.starts_with("iconfile:") {
            "1"
        } else {
            ""
        };
        while cursor < linear {
            cells.push(auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                ("spacer", auto_val::Value::str("1")),
                ("c", auto_val::Value::str((cursor % COLS).to_string())),
                ("r", auto_val::Value::str((cursor / COLS).to_string())),
            ]))));
            ids.push(String::new());
            cs.push(auto_val::Value::str((cursor % COLS).to_string()));
            rs.push(auto_val::Value::str((cursor / COLS).to_string()));
            cursor += 1;
        }
        cells.push(auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", auto_val::Value::Str(id.clone().into())),
            ("icon", auto_val::Value::Str(icon.into())),
            // 满幅 tile 渲染旗标（"1" = 图标铺满格子、无 badge 色底框；
            // lucide 字标应用保持色块 chip + 白 glyph——用户裁定「保留
            // 原图圆角板、外框不要」）。
            ("full", auto_val::Value::str(full)),
            ("label", auto_val::Value::Str(label.into())),
            ("src", auto_val::Value::Str(src.into())),
            ("color", auto_val::Value::Str(color.into())),
            ("c", auto_val::Value::str(c.to_string())),
            ("r", auto_val::Value::str(r.to_string())),
        ]))));
        ids.push(id);
        cs.push(auto_val::Value::str(c.to_string()));
        rs.push(auto_val::Value::str(r.to_string()));
        cursor = linear + 1;
    }
    (cells, ids, cs, rs)
}

/// `shell.desktop.positions` 追加式 csv 解析——"id=c,r" 段（逗号分隔），
/// 同 id 后写胜（shell 落子为追加写，末段即最新位）。
pub(super) fn load_desktop_positions() -> Vec<(String, (usize, usize))> {
    load_desktop_positions_for(None)
}

/// PLAN-019 T-06：壁纸布局键控读——`key` = Some(壁纸布局键) 时优先读该键
/// （缺席回退缺省底稿 `shell.desktop.positions`——首次切到该壁纸继承默认
/// 摆布）；None = 缺省底稿直读（#hex/builtin 壁纸无布局语境）。
pub(super) fn load_desktop_positions_for(key: Option<&str>) -> Vec<(String, (usize, usize))> {
    let raw = key
        .and_then(|k| auto_lang_core::vm::ffi::stdlib::storage_host_read(k))
        .or_else(|| auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.desktop.positions"))
        .unwrap_or_default();
    parse_positions_csv(&raw)
}

/// 位置 csv 解析核心（`load_desktop_positions_for` 的解析体）。
pub(super) fn parse_positions_csv(raw: &str) -> Vec<(String, (usize, usize))> {
    let mut out: Vec<(String, (usize, usize))> = Vec::new();
    for seg in raw.split(',') {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        let Some((id, pos)) = seg.split_once('=') else {
            continue;
        };
        let Some((c, r)) = pos.split_once(':') else {
            continue;
        };
        if let (Ok(c), Ok(r)) = (c.trim().parse::<usize>(), r.trim().parse::<usize>()) {
            out.retain(|(existing, _)| existing != id);
            out.push((id.trim().to_string(), (c, r)));
        }
    }
    out
}

/// 位置表 → 全量 csv（"id=c:r," 逗号封尾；apply_drop 终态重写共用）。
pub(super) fn serialize_positions(positions: &[(String, (usize, usize))]) -> String {
    let mut csv = String::new();
    for (id, (c, r)) in positions {
        csv.push_str(&format!("{id}={c}:{r},"));
    }
    csv
}

/// PLAN-019 T-06：壁纸布局键——归一化路径（`\`→`/` + ASCII 小写折叠）
/// 的 FNV-1a 64 位十六进制。归一化规则（SD-03 spec 定案）：Windows 路径
/// 大小写/分隔符差异不影响键控；非 ASCII 不折叠（NTFS 大小写折叠仅 ASCII
/// 等价）。`#hex`/`builtin:` 无布局语境 → 调用方不产键。
pub(super) fn wallpaper_layout_key(wallpaper: &str) -> String {
    let norm: String = wallpaper
        .trim()
        .chars()
        .map(|c| {
            if c == '\\' {
                '/'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in norm.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("shell.desktop.positions.wp/{hash:016x}")
}

/// 当前壁纸的布局键（None = 壁纸缺省/色值/内置——布局走缺省底稿）。
fn current_wallpaper_layout_key(state: &crate::ui::session::DesktopSession) -> Option<String> {
    let wp = state.desktop.desktop_wallpaper.as_str();
    if wp.is_empty() || wp.starts_with('#') || wp.starts_with("builtin:") {
        return None;
    }
    Some(wallpaper_layout_key(wp))
}

/// Plan 496 M5：桌面本体投影注入——pinned ∪ 自定义条目合并去重
/// （pinned 先列，custom 重叠去重接排；hidden 两者通用排除），条目
/// {id,icon,label,src,color}（icon/label 注册表解析，缺省回退 app-window/id；
/// color = per-app 徽标底色,Plan 518 G4③ 按名哈希分配）；
/// `__desktop_bg`（#hex → "bg-[#hex]" 根 bg 实铺片段；图片路径 → ""，由
/// 宿主壁纸图层铺底——DSL 无重叠布局）+ `__desktop_hidden`（移除臂
/// `shell.desktop.hidden` 续写底稿）。boot 期 inject_dock_pinned 邻位。
/// PLAN-012 F2 走查：拖拽落格目标形态——图标 id（落到其所在格）或像素
/// 换算后的线性格。
enum DropTarget {
    Icon(String),
    Slot(usize),
}

/// 图标 id 未命中（竞态：目标已被移除）时的兜底格 = 表尾。
fn target_slot_default() -> usize {
    usize::MAX
}

/// PLAN-012 F2 走查：拖拽落格宿主执行——共用落格计算：dragged → 目标
/// 线性格；目标被其他图标占据时占位者**挤到下一空格**（行主序先下后右
/// 列，用户裁定 UX）；重写 `shell.desktop.positions`（全量 last-wins csv）
/// + `inject_desktop_surface` 重注入。返回是否发生变更。
fn desktop_icon_apply_drop(
    state: &mut crate::ui::session::DesktopSession,
    dragged: &str,
    target: DropTarget,
) -> bool {
    const COLS: usize = 8;
    let hidden = load_desktop_id_list("shell.desktop.hidden");
    let customs = load_desktop_id_list("shell.desktop.icons");
    let visible: Vec<String> = customs
        .into_iter()
        .filter(|id| !hidden.contains(id))
        .collect();
    if !visible.iter().any(|id| id == dragged) {
        return false;
    }
    // PLAN-019 T-06：键控读（当前壁纸键 → 回退缺省底稿）。
    let positions = load_desktop_positions_for(current_wallpaper_layout_key(state).as_deref());
    let cell_of = |id: &str| -> Option<(usize, usize)> {
        positions
            .iter()
            .rev()
            .find(|(eid, _)| eid == id)
            .map(|(_, c)| *c)
    };
    // 现行有效占位（无位者按**列主序**补位——PLAN-043 Phase 4 与
    // desktop_icon_cells 的 2026-09-15 列主序裁定真正同口径；此前本臂
    // 线性行主序扫描、注释却写"同口径"，拖拽一发生补位图标横插，
    // 与展示网格竖排形态分裂）。rows 口径照抄 desktop_icon_cells。
    let rows = {
        let viewport = state.host_viewport();
        let reserved = desktop_dock_edges(&state.desktop.config);
        (((viewport.height - reserved.top - reserved.bottom) / 80.0).floor() as usize)
            .clamp(4, 24)
            .max((visible.len() + COLS - 1) / COLS)
            .max(1)
    };
    let mut taken: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let mut cells: Vec<(String, usize)> = Vec::new();
    for id in &visible {
        let slot = match cell_of(id) {
            Some((c, r)) => r * COLS + c,
            None => {
                // 列主序首个空格：序数 k → (c, r) = (k / rows, k % rows)，
                // 跳过已占位（desktop_icon_cells ②同构）。
                let mut k = 0usize;
                loop {
                    let (c, r) = (k / rows, k % rows);
                    let s = r * COLS + c;
                    k += 1;
                    if !taken.contains(&s) {
                        taken.insert(s);
                        cells.push((id.clone(), s));
                        break;
                    }
                }
                continue;
            }
        };
        taken.insert(slot);
        cells.push((id.clone(), slot));
    }
    let target_slot = match target {
        DropTarget::Slot(s) => s,
        DropTarget::Icon(tid) => cells
            .iter()
            .find(|(id, _)| id == &tid)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| {
                // 兜底 = 现有最大格 + 1（追加到表尾，而非 usize::MAX）。
                cells.iter().map(|(_, s)| *s).max().map_or(0, |m| m + 1)
            }),
    };
    let dragged_slot = cells
        .iter()
        .find(|(id, _)| id == dragged)
        .map(|(_, s)| *s)
        .unwrap_or(target_slot);
    // 目标被其他图标占据 → 占位者挤到下一空格（target+1 起行主序）。
    let occupant = cells
        .iter()
        .find(|(id, slot)| *id != dragged && *slot == target_slot)
        .map(|(id, _)| id.clone());
    let mut new_cells: Vec<(String, usize)> = cells
        .into_iter()
        .map(|(id, slot)| {
            if id == dragged {
                (id, target_slot)
            } else {
                (id, slot)
            }
        })
        .collect();
    if let Some(oid) = &occupant {
        let mut free = target_slot + 1;
        loop {
            let taken_by_other = new_cells
                .iter()
                .any(|(id, slot)| id != oid && *slot as usize == free);
            if !taken_by_other {
                break;
            }
            free += 1;
        }
        for (id, slot) in new_cells.iter_mut() {
            if id == oid {
                *slot = free;
            }
        }
    }
    // 全量 positions 重写（last-wins csv 直接生成终态，非追加）。
    // PLAN-019 T-06：写当前壁纸键（键控壁纸）；缺省壁纸（#hex/builtin）
    // 落缺省底稿键。
    let positions: Vec<(String, (usize, usize))> = new_cells
        .iter()
        .map(|(id, slot)| (id.clone(), (slot % COLS, slot / COLS)))
        .collect();
    let write_key = current_wallpaper_layout_key(state)
        .unwrap_or_else(|| "shell.desktop.positions".to_string());
    auto_lang_core::vm::ffi::stdlib::storage_host_publish(
        &write_key,
        serialize_positions(&positions),
    );
    true
}

fn execute_desktop_icon_drop(
    state: &mut crate::ui::session::DesktopSession,
    dragged: &str,
    target: String,
) {
    let changed = desktop_icon_apply_drop(state, dragged, DropTarget::Icon(target));
    if changed {
        inject_desktop_surface(state);
    }
    clear_desktop_drag_visual(state);
}

/// PLAN-012 F2 走查：松手落格（光标形态）——光标取 wm.last_cursor
/// （desktop 本地坐标），格换算 + 落格 + 清 shell 视觉拖拽态。
pub(super) fn execute_desktop_icon_drop_at_cursor(
    state: &mut crate::ui::session::DesktopSession,
    dragged: &str,
) {
    let (cx, cy) = state
        .host
        .as_ref()
        .map(|h| {
            let p = h.wm.last_cursor.get();
            (p.x, p.y)
        })
        .unwrap_or((f32::NEG_INFINITY, f32::NEG_INFINITY));
    execute_desktop_icon_drop_at(state, dragged, &format!("{cx},{cy}"));
}

fn execute_desktop_icon_drop_at(
    state: &mut crate::ui::session::DesktopSession,
    dragged: &str,
    xy: &str,
) {
    // 2026-09-15：未超位移阈值 = 点击——两路落格动词统一拒落（兜底
    // __mouse_released 臂已拦，此处覆盖 BlankDrop 路径），只清视觉态。
    if !state.desktop.icon_drag_moved {
        clear_desktop_drag_visual(state);
        return;
    }
    // x,y → 线性格换算在闭包内完成（解析失败 = no-op）。
    // desktop 本地像素 → 格：面根 p-3(12px)；横向 80px 格宽 + gap 8 =
    // 88px 栅距；纵向 72px 格（图标 48px + 标题回归）+ gap 8 = 80px 栅距
    // （FU1 曾翻倍至 168，FU5 用户裁定回半）。
    let changed = (|| -> Option<bool> {
        let (xs, ys) = xy.split_once(',')?;
        let x = xs.trim().parse::<f32>().ok()?;
        let y = ys.trim().parse::<f32>().ok()?;
        const ORIGIN: f32 = 12.0;
        const XPITCH: f32 = 88.0;
        const YPITCH: f32 = 80.0;
        let col = (((x - ORIGIN) / XPITCH).floor() as i32).clamp(0, 7) as usize;
        let row = (((y - ORIGIN) / YPITCH).floor() as i32).clamp(0, 96) as usize;
        Some(desktop_icon_apply_drop(
            state,
            dragged,
            DropTarget::Slot(row * 8 + col),
        ))
    })()
    .unwrap_or(false);
    if changed {
        inject_desktop_surface(state);
    }
    clear_desktop_drag_visual(state);
}

/// PLAN-012 F2 走查：清桌面面视觉拖拽态（落格完成后调用）。
pub(super) fn clear_desktop_drag_visual(state: &mut crate::ui::session::DesktopSession) {
    state.desktop.icon_drag_moved = false;
    if let Some(surface) = state.desktop.desktop_app {
        if let Some(app) = state.apps.get_mut(&surface) {
            let _ = app
                .component
                .write_state("drag_id", auto_val::Value::str(""));
            // 2026-09-15：落点高亮/幽灵数据面同步清零。
            let _ = app
                .component
                .write_state("drop_c", auto_val::Value::str(""));
            let _ = app
                .component
                .write_state("drop_r", auto_val::Value::str(""));
            let _ = app
                .component
                .write_state("drag_icon", auto_val::Value::str(""));
            let _ = app
                .component
                .write_state("drag_moved", auto_val::Value::str(""));
            *app.state.view_dirty.borrow_mut() = true;
        }
    }
}

/// 2026-09-15：壁纸亮度判定——#hex 直算 luma；图片壁纸解码缩 32×32 求
/// 平均亮度（<128 = 暗）。结果按路径缓存（inject 在拖拽落格等时机高频
/// 重入，解码不可重复）。判定失败（解码失败/未知形态）= 亮（前景色兜底）。
fn desktop_wallpaper_dark(wallpaper: &str) -> bool {
    if let Some(hex) = wallpaper.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                // Rec.601 luma（u8 溢出防护用 u16 中间量）。
                let luma = (r as u16 * 299 + g as u16 * 587 + b as u16 * 114) / 1000;
                return luma < 128;
            }
        }
        return false;
    }
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    fn cache() -> &'static Mutex<HashMap<String, bool>> {
        static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
        CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }
    if let Some(hit) = cache().lock().ok().and_then(|c| c.get(wallpaper).copied()) {
        return hit;
    }
    let dark = load_image_bytes(wallpaper)
        .and_then(|data| image::load_from_memory(&data).ok())
        .map(|img| {
            let img = img.thumbnail(32, 32).to_rgb8();
            let n = img.pixels().len().max(1) as u32;
            let sum: u32 = img
                .pixels()
                .map(|p| (p[0] as u32 * 299 + p[1] as u32 * 587 + p[2] as u32 * 114) / 1000)
                .sum();
            sum / n < 128
        })
        .unwrap_or(false);
    if let Ok(mut c) = cache().lock() {
        c.insert(wallpaper.to_string(), dark);
    }
    dark
}

/// PLAN-030 T-04：desktop 面快照构建（单源——in-proc 写集与 outproc
/// wire 推送共用；纯派生，无 App 会话触碰）。
pub(crate) fn build_desktop_surface_snapshot(
    state: &crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::DesktopSurfaceSnapshot {
    use crate::ui::shell_projection::{DesktopCellEntry, DesktopIconEntry, DesktopSurfaceSnapshot};
    let hidden = load_desktop_id_list("shell.desktop.hidden");
    let customs = load_desktop_id_list("shell.desktop.icons");
    // PLAN-012 F2（用户裁定）：dock 固定与桌面快捷方式**分离**——桌面图标
    // = 仅自定义列表（shell.desktop.icons − hidden），pinned 不再并入
    // （固定到任务栏 ≠ 发送到桌面，两件事）。
    let order: Vec<(String, &str)> = customs.iter().map(|id| (id.clone(), "custom")).collect();
    let icons: Vec<DesktopIconEntry> = order
        .iter()
        .filter(|(id, _)| !hidden.contains(id))
        .map(|(id, _)| {
            let reg = state.desktop.registry_entries.iter().find(|e| &e.id == id);
            DesktopIconEntry {
                id: id.clone(),
                icon: reg
                    .map(|e| e.icon.clone())
                    .unwrap_or_else(|| "app-window".to_string()),
                label: reg
                    .map(|e| e.display_title().to_string())
                    .unwrap_or_else(|| id.clone()),
                src: "custom".to_string(),
                color: crate::ui::app_registry::badge_color_for(id).to_string(),
            }
        })
        .collect();
    let bg = if state.desktop.desktop_wallpaper.starts_with('#') {
        format!("bg-[{}]", state.desktop.desktop_wallpaper)
    } else {
        String::new()
    };
    // PLAN-012 W5：格子分配（拖拽换位数据面）；PLAN-019 T-06：布局键控
    // ——当前壁纸键优先。rows 随视口高度（2026-09-15 裁定）。
    let layout_key = current_wallpaper_layout_key(state);
    let rows = {
        let viewport = state.host_viewport();
        let reserved = desktop_dock_edges(&state.desktop.config);
        (((viewport.height - reserved.top - reserved.bottom) / 80.0).floor() as usize).clamp(4, 24)
    };
    let (cells, cell_ids, cell_cs, cell_rs) = desktop_icon_cells(
        &order,
        &hidden,
        &state.desktop.registry_entries,
        layout_key.as_deref(),
        rows,
    );
    let cells: Vec<DesktopCellEntry> = cells
        .iter()
        .map(|v| match v {
            auto_val::Value::Obj(obj) => {
                let get = |k: &str| {
                    obj.get(k)
                        .and_then(|v| match v {
                            auto_val::Value::Str(s) => Some(s.as_str().to_string()),
                            _ => None,
                        })
                        .unwrap_or_default()
                };
                let spacer = obj.get("spacer").is_some();
                DesktopCellEntry {
                    spacer,
                    id: get("id"),
                    icon: get("icon"),
                    full: !get("full").is_empty(),
                    label: get("label"),
                    src: get("src"),
                    color: get("color"),
                    c: get("c").parse().unwrap_or(0),
                    r: get("r").parse().unwrap_or(0),
                }
            }
            _ => DesktopCellEntry::default(),
        })
        .collect();
    let val_str = |v: &auto_val::Value| match v {
        auto_val::Value::Str(s) => s.as_str().to_string(),
        _ => String::new(),
    };
    DesktopSurfaceSnapshot {
        icons,
        cells,
        cell_ids,
        cell_cs: cell_cs.iter().map(val_str).collect(),
        cell_rs: cell_rs.iter().map(val_str).collect(),
        bg,
        label_dark: desktop_wallpaper_dark(&state.desktop.desktop_wallpaper),
        hidden: hidden.join(","),
        // __wm_running 由 shell 面投影组同拍带入（apply 侧同源）。
        running_csv: String::new(),
        events: Vec::new(),
        fp: String::new(),
    }
}

/// PLAN-030 T-04：壳表面内容（伪窗 wid → 壳连接对应表面合成帧 canvas；
/// background[0] / chrome[1]，Layer 序见 view_desktop_fn 两槽位）。
pub(super) fn shell_surface_element(
    state: &crate::ui::session::DesktopSession,
    wid: crate::ui::session::Wid,
) -> Option<iced::Element<'_, crate::ui::session::DesktopMessage>> {
    shell_surface_element_pipe(state, state.desktop.shell_pipe.clone(), wid)
}

/// PLAN-036 T-07（B3）：定向 pipe 表面帧（launcher 独立管线——壳管线
/// 同体泛化）。
pub(super) fn shell_surface_element_pipe(
    state: &crate::ui::session::DesktopSession,
    pipe: Option<String>,
    wid: crate::ui::session::Wid,
) -> Option<iced::Element<'_, crate::ui::session::DesktopMessage>> {
    let pipe = pipe.as_ref()?;
    let client = state.broker_clients.get(pipe)?;
    let surface = client.wid_surface.get(&wid.0)?;
    let list = client.surfaces.front(*surface)?;
    Some(crate::ui::iced::broker_surface::drawlist_element(list))
}

pub(super) fn inject_desktop_surface(state: &mut crate::ui::session::DesktopSession) {
    let Some(surface) = state.desktop.desktop_app else {
        return;
    };
    let snap = build_desktop_surface_snapshot(state);
    // PLAN-035 T-11：boot 注入 __wm_dashboard（右键菜单 checkbox 判据）。
    let dash = if state.dashboard_visible() { "1" } else { "" };
    let Some(app) = state.apps.get_mut(&surface) else {
        return;
    };
    for w in snap.interpreted_writes() {
        match w {
            crate::ui::shell_projection::ShellWrite::Scalar(k, v) => {
                let _ = app.component.write_state(k, v);
            }
            crate::ui::shell_projection::ShellWrite::Array(k, vs) => {
                let _ = app.component.write_state_vec(k, vs);
            }
        }
    }
    // PLAN-035 T-11：__wm_dashboard boot 注入（见上）。
    let _ = app
        .component
        .write_state("__wm_dashboard", auto_val::Value::str(dash));
    *app.state.view_dirty.borrow_mut() = true;
}

/// Plan 496 M5：壁纸底层元素（图片分支——solid 分支由 desktop.at 根 bg
/// 承接，不推本层）。无事件纯铺底；运行中图片被删的防御臂 = 默认色容器
/// （boot 解析已验存在）。
pub(super) fn desktop_wallpaper_element<M: 'static>(path: &str) -> iced::Element<'static, M> {
    match load_image_bytes(path) {
        Some(data) => {
            let handle = get_or_create_image_handle(path, data, 0.0, None, None);
            iced::widget::image(handle)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                // PLAN-526 T19：Cover 铺满（iced 默认 Contain 按比例
                // letterbox——实测宽高比不匹配时顶部留空一条）。
                .content_fit(iced::ContentFit::Cover)
                .into()
        }
        None => iced::widget::container(iced::widget::Space::new())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(20, 26, 41))),
                ..Default::default()
            })
            .into(),
    }
}

/// Plan 503 M3：壁纸罩层 —— 图片壁纸上叠 background 语义色 scrim
/// （light 10% / dark 35%），提升桌面图标/文本可读性（stella 壁纸罩层
/// 移植；降格为纯色叠加，无 blur——parity 条款）。纯色壁纸分支不叠：
/// 底色无对比噪音，且桌面本体面不透明底会盖住本层，叠之无效。
pub(super) fn desktop_wallpaper_scrim<M: 'static>() -> iced::Element<'static, M> {
    let dark = crate::ui::style::theme::dark_mode();
    let pct: u32 = if dark { 35 } else { 10 };
    let (r, g, b) =
        crate::ui::style::theme::resolve_semantic_rgb(&crate::ui::style::Color::Background)
            .unwrap_or((20, 26, 41));
    let alpha = (pct * 255 / 100) as f32 / 255.0;
    iced::widget::container(iced::widget::Space::new())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba8(
                r, g, b, alpha,
            ))),
            ..Default::default()
        })
        .into()
}

/// Plan 478 T3：投影条目构建（wins/mru 两段共用；协议 v1.1 条目同型六字段）。
/// entry 字段全用串：.at 侧 onclick 参数拼接（"focus\t" + wid）与条件渲染
/// 都走字符串语义（dashboard/041 对象列表同型）。icon 自注册表实时查
/// （registry_entries 唯一事实源；未登记/无条目回退 "app-window"）。
/// PLAN-012 O2：`pinned`/`dup_app` 宿主派生判据面（"1"/""）——dock 去重
/// 与"同类 app 共享一图标"在 .at view 条件侧无 `contains`/聚合原语
///（eval_condition_with 方法调用臂缺失，T9 发现②同族：静默塌缩恒 false，
/// 实机 dock 双图标实证），故由宿主单点派生、shell 等式消费。
fn projection_win_entry(
    registry_entries: &[crate::ui::app_registry::AppRegistryEntry],
    v: &crate::ui::session::VWinState,
    focused: bool,
    pager: &str,
    pinned: bool,
    dup_app: bool,
) -> auto_val::Value {
    let icon = v
        .registry_id
        .as_ref()
        .and_then(|id| registry_entries.iter().find(|e| &e.id == id))
        .map(|e| e.icon.clone())
        .unwrap_or_else(|| "app-window".to_string());
    auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
        ("wid", auto_val::Value::Str(v.wid.0.to_string().into())),
        ("title", auto_val::Value::Str(v.title.clone().into())),
        (
            "focused",
            auto_val::Value::Str(if focused { "1" } else { "".into() }.into()),
        ),
        (
            "workspace",
            auto_val::Value::Str(v.workspace.to_string().into()),
        ),
        // Plan 486 v1.3：App 条目恒空串（native 分支判据统一）。
        ("native", auto_val::Value::Str(String::new().into())),
        (
            "app",
            auto_val::Value::Str(v.registry_id.clone().unwrap_or_default().into()),
        ),
        ("icon", auto_val::Value::Str(icon.into())),
        // Plan 505 B2 v1.5：pager 派生面——本窗是否属其分区缩略前 4
        //（"1"/""；mru/native 条目恒 ""，判据统一不缺字段）。
        ("pager", auto_val::Value::Str(pager.into())),
        // PLAN-012 O2：本窗 app 已固定 / 同 app 已有更前位窗（z_order 序
        // 首见之外）——dock 条目跳过判据（.at 等式消费）。
        (
            "pinned",
            auto_val::Value::Str(if pinned { "1" } else { "".into() }.into()),
        ),
        (
            "dup_app",
            auto_val::Value::Str(if dup_app { "1" } else { "".into() }.into()),
        ),
    ])))
}

/// Plan 505 C：实机验收通道注入排空（ServiceTick 节拍，≤400ms 生效）。
/// Bus 记录并入 shell `__desktop_cmd`（同拍 drain 同臂消费——真实 shell
/// 按钮的同一执行路径）；Handler 直呼特权 App handler（onclick 同一
/// 管线）。入队面见 [`crate::ui::session::desktop_inject_push`]（MCP
/// `autoui_desktop` 工具，AUTOUI_ACCEPTANCE=1 门控）。
/// PLAN-030 T-06：DesktopBus inbox 排空（ServiceTick 泵后调用 + 单测口）——
/// 直调 execute_desktop_commands（同拍生效）；归因 notify_source 随末段。
pub(crate) fn drain_desktop_bus_inbox(
    state: &mut crate::ui::session::DesktopSession,
) -> Vec<iced::Task<crate::ui::session::DesktopMessage>> {
    let inbox = std::mem::take(&mut state.desktop.desktop_bus_inbox);
    if inbox.is_empty() {
        return Vec::new();
    }
    let cmds: Vec<_> = inbox.iter().map(|(_, c)| c.clone()).collect();
    let source = inbox.last().map(|(s, _)| s.clone()).unwrap_or_default();
    state.desktop.notify_source.replace(Some(source));
    let (_, tasks) = execute_desktop_commands(state, cmds);
    tasks
}

pub(crate) fn apply_desktop_injects(state: &mut crate::ui::session::DesktopSession) {
    use crate::ui::session::DesktopInject;
    for inj in crate::ui::session::desktop_inject_take() {
        match inj {
            DesktopInject::Bus(record) => {
                // PLAN-030 T-06：outproc 壳改道——验收 Bus 记录与生产上行
                // 同臂（desktop_bus_inbox → 泵后排空执行；in-proc 镜像组件
                // 写入对 outproc 壳恒空[D4-5 实锤]）。in-proc 轨原路径零
                // 变化（I1）。
                if state.desktop.shell_pipe.is_some() {
                    for cmd in crate::ui::session::DesktopCommand::parse_records(&record) {
                        state
                            .desktop
                            .desktop_bus_inbox
                            .push(("shell".to_string(), cmd));
                    }
                    continue;
                }
                let Some(shell) = state.desktop.shell_app else {
                    continue;
                };
                let Some(app) = state.apps.get_mut(&shell) else {
                    continue;
                };
                let cur = match app.component.read_state("__desktop_cmd") {
                    Ok(auto_val::Value::Str(s)) => s.to_string(),
                    _ => String::new(),
                };
                let joined = if cur.is_empty() {
                    record
                } else {
                    format!("{cur}\n{record}")
                };
                let _ = app
                    .component
                    .write_state("__desktop_cmd", auto_val::Value::str(&joined));
            }
            DesktopInject::Handler {
                app: which,
                handler,
                arg,
                widget,
            } => {
                let app_id = match which {
                    "shell" => state.desktop.shell_app,
                    "notification" => state.desktop.notification_app,
                    "launcher" => state.desktop.launcher_app,
                    // Plan 505 C：桌面本体面（496 M5——图标交互/壁纸面）。
                    "desktop" => state.desktop.desktop_app,
                    // PLAN-035 T-18：dashboard 面板槽（分页 pill/SelectTab
                    // 验收注入）。
                    "dashboard" => state.desktop.dashboard_app,
                    // Plan 551：settings 槽 = os-config 窗（⚙️ 同靶——
                    // 540 的 045 专用槽随窗退役，验收通道按 registry_id
                    // 定位 launch-or-focus 后的 os-config 前端）。
                    "settings" => state.host.as_ref().and_then(|h| {
                        h.wm.wins
                            .iter()
                            .find(|(_, v)| v.registry_id.as_deref() == Some(OSCONFIG_APP_ID))
                            .map(|(_, v)| v.app)
                    }),
                    // PLAN-721 T-4/T-DOCS-1：验收通道任意内嵌 app 直呼——
                    // registry_id 反查（settings 臂同款机制泛化；launch 后窗
                    // 条目带注册名）。特权名先行匹配不受遮蔽（上方臂优先）。
                    // 2026-10-10：孵化 mini 会话无窗——registry 反查落空时回退
                    // hatched_minis（小组件成为一等验收靶面：独立播控/取数
                    // 直呼）。
                    which => state
                        .host
                        .as_ref()
                        .and_then(|h| {
                            h.wm.wins
                                .iter()
                                .find(|(_, v)| v.registry_id.as_deref() == Some(which))
                                .map(|(_, v)| v.app)
                        })
                        .or_else(|| state.hatched_mini_of(which)),
                };
                let Some(app_id) = app_id else { continue };
                if let Some(result) = state.broker_named_action(app_id, &handler, arg.as_deref(), widget.as_deref()) {
                    if let Err(error) = result {
                        eprintln!("[desktop-input] handler {handler}: {error}");
                    }
                    continue;
                }
                let Some(sess) = state.apps.get_mut(&app_id) else {
                    continue;
                };
                let args = arg
                    .map(|a| vec![auto_val::Value::str(&a)])
                    .unwrap_or_default();
                match &widget {
                    // Plan 559 W7: (app, widget) targeting — dispatch into
                    // the named sub-widget's handler context (namespaced
                    // call_handler_for, the onclick pipeline), falling back
                    // to the root bare-name dispatch when the widget has no
                    // live state object.
                    Some(w) => {
                        let _ = sess.component.call_widget_handler(w, &handler, &args);
                    }
                    None => {
                        let _ = sess.component.bridge_mut().call_handler(&handler, &args);
                    }
                }
                *sess.state.view_dirty.borrow_mut() = true;
            }
            // PLAN-029 T-03（D2 辅腿）：acceptance key verb——真桌面 update
            // 循环内驱动 live 输入生产路由（与 DesktopEvent::LiveInput 臂
            // 同一入口 `route_live_input`；AUTOUI_ACCEPTANCE 门在工具侧）。
            DesktopInject::Key(input) => {
                state.route_live_input(&input);
            }
            // PLAN-060：消息注入（组件事件循环；VM 异步 API 正常挂起续跑）。
            DesktopInject::MsgValues { app, msg, args } => {
                let app_id = if app == "shell" {
                    state.desktop.shell_app
                } else {
                    state.host.as_ref().and_then(|host| {
                        host.wm.wins.values()
                            .find(|window| window.registry_id.as_deref() == Some(app.as_str()))
                            .map(|window| window.app)
                    }).or_else(|| state.hatched_mini_of(&app))
                };
                if let Some(app_id) = app_id {
                    if let Err(error) = state.dispatch_scalar_message(app_id, msg, &args) {
                        eprintln!("[desktop-input] typed message: {error}");
                    }
                }
            }
            DesktopInject::Msg { app, msg, arg } => {
                let app_id = if app == "shell" {
                    state.desktop.shell_app
                } else {
                    state
                        .host
                        .as_ref()
                        .and_then(|h| {
                            h.wm.wins
                                .iter()
                                .find(|(_, v)| v.registry_id.as_deref() == Some(app.as_str()))
                                .map(|(_, v)| v.app)
                        })
                        .or_else(|| state.hatched_mini_of(&app))
                };
                let Some(app_id) = app_id else { continue };
                // PLAN-060 扩展：broker/adopt 应用（子进程持有组件）→ 走协议
                // Action 下行；in-process 组件走既有 on() 直派。
                if std::env::var_os("AUTO_PLAN060_DIAG").is_some() {
                    let wins: Vec<String> = state
                        .host
                        .as_ref()
                        .map(|h| {
                            h.wm.wins
                                .iter()
                                .map(|(w, v)| format!("{}:app={}:reg={:?}", w.0, v.app.0, v.registry_id))
                                .collect()
                        })
                        .unwrap_or_default();
                    let clients: Vec<String> = state
                        .broker_clients
                        .values()
                        .map(|c| format!("wid={:?} surfaces={:?} name={:?}", c.wid, c.wid_surface.keys().collect::<Vec<_>>(), c.app_name))
                        .collect();
                    eprintln!("[plan060] action {msg} app_id={} wins={wins:?} clients={clients:?}", app_id.0);
                }
                if let Some(result) = state.broker_named_action(app_id, &msg, arg.as_deref(), None) {
                    if let Err(error) = result {
                        eprintln!("[desktop-input] message {msg}: {error}");
                    }
                    continue;
                }
                let Some(sess) = state.apps.get_mut(&app_id) else { continue };
                let args = arg
                    .map(|a| vec![auto_val::Value::str(&a)])
                    .unwrap_or_default();
                sess.component.on(crate::ui::interpreter::DynamicMessage::Typed {
                    widget_name: "App".to_string(),
                    event_name: msg,
                    args,
                });
                *sess.state.view_dirty.borrow_mut() = true;
            }
            // PLAN-060：验收通道指针注入——走真实指针消费臂
            // （hit_test → wm_focus → owning client PointerPressed/Released）。
            DesktopInject::Pointer {
                phase,
                x,
                y,
                button,
            } => {
                use crate::ui::session::PointerPhase;
                // PLAN-060 诊断：命中窗与本臂返回（验收通道排障用，稳态保留）。
                let __hit = state.host.as_ref().and_then(|h| h.wm.hit_test(x, y));
                eprintln!("[plan060] pointer {:?} ({:.1},{:.1}) hit={:?}", phase, x, y, __hit);
                match phase {
                    PointerPhase::Down => {
                        state.broker_pointer_down(x, y, button);
                    }
                    PointerPhase::Up => {
                        state.broker_pointer_up(x, y, button);
                    }
                    PointerPhase::Click => {
                        let __d = state.broker_pointer_down(x, y, button);
                        eprintln!("[plan060] pointer down delivered={__d}");
                        if __d {
                            let __u = state.broker_pointer_up(x, y, button);
                            eprintln!("[plan060] pointer up delivered={__u}");
                        }
                    }
                }
            }
        }
    }
}

/// PLAN-526 T6：boot 直挂窗的注册表对齐。462/459 直挂组件（include_str!
/// 编译，无 pac 上下文）按 source path 归一化后缀对齐注册表 `entry`
/// （"<appdir>/src/front/app.at" 或 "<appdir>/app.at" 段）：唯一命中 →
/// 回填 `registry_id` + armed `fit_pending/fit_enabled`（504 launch 路径
/// 同款语义，`window: "fit"` 声明随 400ms ServiceTick 首测收缩）。无
/// source path / 歧义 / 无注册表均维持 462 固定初值（现状不回归）。
pub(super) fn arm_boot_fit_windows(session: &mut crate::ui::session::DesktopSession) {
    use crate::ui::session::{AppId, Wid};
    let entries = session.desktop.registry_entries.clone();
    if entries.is_empty() {
        return;
    }
    let pairs: Vec<(Wid, AppId)> = {
        let Some(host) = session.host.as_ref() else {
            return;
        };
        host.wm.wins.values().map(|v| (v.wid, v.app)).collect()
    };
    for (wid, app_id) in pairs {
        let Some(entry) = session.apps.get(&app_id) else {
            continue;
        };
        let Some(src) = entry.component.source_path() else {
            continue;
        };
        let norm = src.to_string_lossy().replace('\\', "/");
        let mut hits = entries
            .iter()
            .filter(|e| boot_entry_matches(&norm, &e.entry.to_string_lossy().replace('\\', "/")));
        // 唯一命中才 armed（歧义回退现状——待澄清⑦裁定默认）。
        let (Some(first), None) = (hits.next(), hits.next()) else {
            continue;
        };
        if let Some(host) = session.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid) {
                v.registry_id = Some(first.id.clone());
                if first.fit {
                    v.fit_pending.set(true);
                    v.fit_enabled.set(true);
                }
            }
        }
    }
}

/// PLAN-526 T6：boot 对齐匹配核（归一化 `/` 分隔后的双向后缀；纯函数
/// 便于单测——相对源路径 vs 绝对注册表 entry 任一方向收尾命中即真）。
pub(super) fn boot_entry_matches(src_norm: &str, entry_norm: &str) -> bool {
    entry_norm.ends_with(src_norm) || src_norm.ends_with(entry_norm)
}

/// Plan 463 T5：WM → shell 状态注入（T1 报告 §3 反方向，`window_width`
/// 同型约定）。Plan 472 T3 升级为**投影协议 v1**（合同
/// `docs/schema/projection-protocol-v1.md`）：`__wm_wins` 条目增
/// workspace/app/icon 字段、新增 `__wm_workspaces` 分区投影、指纹段扩为
/// "逐窗;|meta|逐分区;"。窗口列表为**跨分区全集**（dock 运行指示消费），
/// 虚拟窗层绘制自过滤。指纹未变则跳过写（防每帧 churn），写后置 shell
/// view_dirty。每 update 周期在排空点邻位调用（O(窗数) 串接，便宜）。
/// Plan 478 T3 升级 **v1.1**：新增 `__wm_mru`（当前分区 MRU 序，switcher
/// 消费）、`__wm_workspaces.label`（1 基标签）；指纹分区段扩 label、
/// 尾接 mru 段（§3 式）。
/// Plan 479 T4 升级 **v1.2**：新增 `__wm_notes`（通知历史全量 {id,kind,msg,
/// at} Obj 数组）+ `__wm_notes_unread`（未读串，badge 消费）；指纹尾接
/// `|notes:{len}:{front_id}:{unread};` 段（T1 施工图定案 6）。
/// Plan 486 升级 **v1.3**：`__wm_wins` 纳入 native 槽位条目
/// {wid:"N<slot>",title,focused,native,icon}（仅 Docked 态；字段集与
/// App 条目部分相交——workspace/app 不适用省略）；指纹窗段并入
/// "N{slot}:{focused},"。
pub(super) fn sync_shell_windows(state: &mut crate::ui::session::DesktopSession) {
    // PLAN-030 T-04：outproc 壳臂——typed 载体 wire 推送（ShellProjection
    // shell 面 + DesktopSurfaceSnapshot desktop 面，per-face 指纹门宿主侧
    // 缓存；None = 全量推——respawn attach 强制失效）。in-proc 轨字节级
    // 零变化（I1：下方原路径全保留）。
    if state.desktop.shell_pipe.is_some() {
        push_shell_projection_outproc(state);
        publish_workspace_previews(state);
        return;
    }
    let Some(shell) = state.desktop.shell_app else {
        return;
    };
    {
        let vp = state.host_viewport();
        if let Some(host_mut) = state.host.as_mut() {
            let mut changed = false;
            for fields in [
                &mut host_mut.shell_fields.window_size,
                &mut host_mut.launcher_fields.window_size,
                &mut host_mut.switcher_fields.window_size,
                &mut host_mut.notification_fields.window_size,
            ] {
                let ws = fields.get_mut();
                if (ws.width - vp.width).abs() > f32::EPSILON
                    || (ws.height - vp.height).abs() > f32::EPSILON
                {
                    *ws = iced::Size::new(vp.width, vp.height);
                    changed = true;
                }
            }
            if changed {
                if let Some(app) = state.apps.get(&shell) {
                    *app.state.view_dirty.borrow_mut() = true;
                }
            }
        }
    }

    // PLAN-027 T-05：投影派生单源化——快照构建（typed 载体
    // shell_projection::ShellProjection）与解释轨回写（指纹门控保留 +
    // 写集逐字节一致）拆分；本函数 = 调度点。desktop 层
    // __wm_running/RunningSync 推送随 apply；workspace_preview 数据发布
    // 独立（PLAN-012 W3 400ms 节拍面，不入指纹组）。
    let proj = build_shell_projection(state);
    apply_shell_projection_interpreted(state, shell, &proj);
    publish_workspace_previews(state);
}

/// PLAN-030 T-04：outproc 壳投影推送泵（sync_shell_windows 壳臂体）——
/// shell 面（17 键指纹门）+ desktop 面（快照指纹门）分面增量；推送失败
/// 不清指纹（下拍重推）。
pub(crate) fn push_shell_projection_outproc(state: &mut crate::ui::session::DesktopSession) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let proj = build_shell_projection(state);
    if std::env::var("AUTO030_TRACE").is_ok() {
        eprintln!(
            "[p030-host] push gate: fp_prev={:?} fp_now={} wins={} running={}",
            state.desktop.shell_push_fp,
            proj.fp,
            proj.wins.len(),
            proj.running_csv
        );
    }
    if state.desktop.shell_push_fp.as_deref() != Some(proj.fp.as_str()) {
        let mut payload = Vec::new();
        crate::ui::shell_projection::ShellProjection::wire_encode(&proj, &mut payload);
        let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
            face: shell_face::SHELL,
            payload,
        });
        if state.push_shell_control(&msg) {
            state.desktop.shell_push_fp = Some(proj.fp.clone());
        }
    }
    push_desktop_surface_outproc(state, &proj.running_csv);
}

/// PLAN-030 T-04：desktop 面 outproc 推送（inject_desktop_surface 壳臂
/// 等价——快照构建单源 build_desktop_surface_snapshot；指纹 = 叶面拼接）。
/// `running_csv` 随 shell 面投影组同拍带入（in-proc apply 同源）。
pub(crate) fn push_desktop_surface_outproc(
    state: &mut crate::ui::session::DesktopSession,
    running_csv: &str,
) {
    use crate::ui::desktop_protocol::message::{shell_face, ControlMsg, ProtocolMsg};
    let mut snap = build_desktop_surface_snapshot(state);
    snap.running_csv = running_csv.to_string();
    let mut fp = String::new();
    for e in &snap.icons {
        fp.push_str(&e.id);
        fp.push('|');
    }
    for e in &snap.cells {
        if !e.spacer {
            fp.push_str(&e.id);
            fp.push('@');
            fp.push_str(&e.c.to_string());
            fp.push(',');
            fp.push_str(&e.r.to_string());
            fp.push(';');
        }
    }
    fp.push_str(&snap.bg);
    fp.push('|');
    fp.push_str(&snap.hidden);
    fp.push('|');
    fp.push_str(if snap.label_dark { "d" } else { "l" });
    fp.push('|');
    fp.push_str(&snap.running_csv);
    if state.desktop.shell_desk_fp.as_deref() == Some(fp.as_str()) {
        return;
    }
    let mut payload = Vec::new();
    crate::ui::shell_projection::DesktopSurfaceSnapshot::wire_encode(&snap, &mut payload);
    let msg = ProtocolMsg::Control(ControlMsg::ShellProjectionPush {
        face: shell_face::DESKTOP_SURFACE,
        payload,
    });
    if state.push_shell_control(&msg) {
        state.desktop.shell_desk_fp = Some(fp);
    }
}

/// PLAN-027 T-05：shell（任务栏）面投影快照构建——原 sync_shell_windows
/// 内联派生逻辑逐行平移（wins/native 槽位/分区/mru/notes/pinned/running/
/// meta + 指纹段式），leaf 形态由 shell_projection 载体承载。native_icon
/// ensure 幂等缓存副作用保留在构建点（Plan 515 D1）。
pub(crate) fn build_shell_projection(
    state: &crate::ui::session::DesktopSession,
) -> crate::ui::shell_projection::ShellProjection {
    use crate::ui::shell_projection::{
        DockPin, ShellNote, ShellProjection, ShellWin, ShellWorkspace,
    };
    let mut proj = ShellProjection::default();
    let Some(host) = state.host.as_ref() else {
        return proj;
    };
    let mut fp = String::new();
    // Plan 505 B2（债 P497-1）v1.5：分区缩略 pager 派生面（宿主单一事实源，
    // 指纹不需扩段——逐窗 workspace 已入指纹）。
    let mut pager_shown = std::collections::HashSet::new();
    let mut ws_more: Vec<String> = vec![String::new(); host.wm.workspaces.len()];
    // PLAN-012 O2：同 app 首见追踪（z_order 序；dup_app 派生源）。
    let mut seen_apps: std::collections::HashSet<String> = Default::default();
    {
        let mut per_ws: std::collections::HashMap<usize, usize> = Default::default();
        for &wid in &host.wm.z_order {
            let Some(v) = host.wm.wins.get(&wid) else {
                continue;
            };
            let idx = *per_ws.entry(v.workspace).or_insert(0);
            per_ws.insert(v.workspace, idx + 1);
            if idx < 4 {
                pager_shown.insert(wid);
            } else if let Some(m) = ws_more.get_mut(v.workspace) {
                *m = format!("+{}", idx - 3);
            }
        }
    }
    // PLAN-012 W1：常驻隐藏窗投影全排除（__wm_wins/pager/指纹/运行集）。
    for &wid in &host.wm.z_order {
        let Some(v) = host.wm.wins.get(&wid) else {
            continue;
        };
        if v.hidden.get() {
            continue;
        }
        // PLAN-030：壳伪窗不入投影（background/chrome 承接面——任务栏/
        // 运行集零污染）。
        if state.desktop.shell_pseudo_wids.contains(&wid) {
            continue;
        }
        let focused = host.wm.focused == Some(wid);
        // PLAN-012 O2：dock 去重判据宿主派生（pinned/dup_app，等式消费）。
        let (pinned_flag, dup_flag) = match v.registry_id.as_deref() {
            Some(id) => (
                state.desktop.dock_pinned.iter().any(|p| p == id),
                !seen_apps.insert(id.to_string()),
            ),
            None => (false, false),
        };
        let icon = v
            .registry_id
            .as_ref()
            .and_then(|id| state.desktop.registry_entries.iter().find(|e| &e.id == id))
            .map(|e| e.icon.clone())
            .unwrap_or_else(|| "app-window".to_string());
        proj.wins.push(ShellWin {
            wid: wid.0.to_string(),
            title: v.title.clone(),
            focused,
            workspace: Some(v.workspace),
            native: false,
            app: v.registry_id.clone().unwrap_or_default(),
            icon,
            pager: pager_shown.contains(&wid),
            pinned: pinned_flag,
            dup_app: dup_flag,
        });
        // 指纹窗段：{wid}:{focused},{workspace};（协议 v1 §2.3）
        fp.push_str(&format!("{}:{},{},", wid.0, focused as u8, v.workspace));
    }
    // Plan 486 v1.3：native 槽位条目（wid="N<slot_id>"；仅 Docked 态投影）。
    // PLAN-709 v1.11：workspace 实时（分区感知投影）+ focused 实时（WM
    // focused 伪 Wid 域代管；v1.3「focused 恒空」退役）。Plan 515 D1：
    // HICON 真图标幂等提取入缓存。
    for (id, slot) in &host.wm.native_slots {
        if slot.state != crate::ui::native_dock::SlotState::Docked {
            continue;
        }
        crate::ui::iced::native_icon::ensure(id.0, slot.hwnd);
        let focused =
            host.wm.focused == Some(crate::ui::session::WmState::native_slot_pseudo_wid(*id));
        proj.wins.push(ShellWin {
            wid: format!("N{}", id.0),
            title: slot.title_cache.clone(),
            focused,
            workspace: Some(slot.workspace),
            native: true,
            app: String::new(),
            icon: crate::ui::iced::native_icon::icon_field(id.0).to_string(),
            pager: false,
            pinned: false,
            dup_app: false,
        });
        fp.push_str(&format!("N{}:{},{},", id.0, focused as u8, slot.workspace));
    }
    let layout_name = match host.wm.layout {
        crate::ui::layout::LayoutMode::Free => "free",
        crate::ui::layout::LayoutMode::Grid => "grid",
        crate::ui::layout::LayoutMode::MasterStack => "master-stack",
    };
    let focused_wid = host.wm.focused.map(|w| w.0.to_string()).unwrap_or_default();
    let meta = format!("{}\t{}", layout_name, focused_wid);
    fp.push_str(&format!("|{}", meta));
    proj.meta = meta;
    // Plan 472 T4：运行中 app 集合派生串（",id1,id2,"）。
    let mut running = String::from(",");
    let mut seen: Vec<&str> = Vec::new();
    for &wid in &host.wm.z_order {
        if let Some(v) = host.wm.wins.get(&wid) {
            // PLAN-012 W1：常驻隐藏窗不入运行集（齿轮高亮语义）。
            if v.hidden.get() {
                continue;
            }
            // PLAN-030：壳伪窗不入运行集（面名非 app——p030 e2e 腿 3
            // trace 实锤泄漏，T-06 收口）。
            if state.desktop.shell_pseudo_wids.contains(&wid) {
                continue;
            }
            if let Some(id) = v.registry_id.as_deref() {
                if !seen.contains(&id) {
                    seen.push(id);
                    running.push_str(id);
                    running.push(',');
                }
            }
        }
    }
    proj.running_csv = running.clone();
    // PLAN-012 W7 v1.6：聚焦 app id（fp 不扩段——聚焦变化必经 meta 段）。
    proj.focused_app = host
        .wm
        .focused
        .and_then(|wid| host.wm.wins.get(&wid))
        .and_then(|v| v.registry_id.clone())
        .unwrap_or_default();
    // PLAN-012 O2：⚙️ 高亮判据标量（非隐藏 os-config 窗在场）。
    proj.settings_open = host
        .wm
        .wins
        .values()
        .any(|v| !v.hidden.get() && v.registry_id.as_deref() == Some(OSCONFIG_APP_ID));
    // PLAN-019 v1.7：__wm_showdesk 标量（当前分区 = 负一屏）。
    proj.showdesk = host.wm.on_showdesk();
    // Plan 472 T3 / 478 T3 v1.1：分区投影段 + 指纹段。
    fp.push('|');
    for ws in &host.wm.workspaces {
        // PLAN-019 v1.7：负一屏保留分区不入投影。
        if host.wm.showdesk_ws == Some(ws.id) {
            continue;
        }
        let current = host.wm.current_workspace == ws.id;
        let label = (ws.id + 1).to_string();
        proj.workspaces.push(ShellWorkspace {
            id: ws.id,
            name: ws.name.clone(),
            current,
            label: label.clone(),
            more: ws_more.get(ws.id).cloned().unwrap_or_default(),
        });
        fp.push_str(&format!("{}:{},{};", ws.id, current as u8, label));
    }
    // Plan 478 T3 v1.1：__wm_mru 段（switcher 消费；隐藏窗不入；
    // pinned/dup/pager 判据恒空——投影判据面统一约定）。
    fp.push('|');
    for wid in host.wm.mru_in_workspace(host.wm.current_workspace) {
        // PLAN-709 v1.11：槽位条目并入 __wm_mru（Docked 门控同 wins 段；
        // 指纹段 N 形态与窗段同型）。
        if let Some(slot_id) = crate::ui::session::WmState::native_slot_id_of_pseudo(wid) {
            let Some(slot) = host.wm.native_slots.get(&slot_id) else {
                continue;
            };
            if slot.state != crate::ui::native_dock::SlotState::Docked {
                continue;
            }
            let focused = host.wm.focused == Some(wid);
            proj.mru.push(ShellWin {
                wid: format!("N{}", slot_id.0),
                title: slot.title_cache.clone(),
                focused,
                workspace: Some(slot.workspace),
                native: true,
                app: String::new(),
                icon: crate::ui::iced::native_icon::icon_field(slot_id.0).to_string(),
                pager: false,
                pinned: false,
                dup_app: false,
            });
            fp.push_str(&format!("N{};", slot_id.0));
            continue;
        }
        if let Some(v) = host.wm.wins.get(&wid) {
            if v.hidden.get() {
                continue;
            }
            let focused = host.wm.focused == Some(wid);
            let icon = v
                .registry_id
                .as_ref()
                .and_then(|id| state.desktop.registry_entries.iter().find(|e| &e.id == id))
                .map(|e| e.icon.clone())
                .unwrap_or_else(|| "app-window".to_string());
            proj.mru.push(ShellWin {
                wid: v.wid.0.to_string(),
                title: v.title.clone(),
                focused,
                workspace: Some(v.workspace),
                native: false,
                app: v.registry_id.clone().unwrap_or_default(),
                icon,
                pager: false,
                pinned: false,
                dup_app: false,
            });
        }
        fp.push_str(&format!("{};", wid.0));
    }
    // Plan 479 T4 v1.2：__wm_notes 段 + 指纹尾段
    // "|notes:{len}:{front_id}:{unread}:{visible};"。
    let notes_snapshot: Vec<crate::ui::session::NotificationEntry> =
        state.desktop.notifications.borrow().clone();
    for n in &notes_snapshot {
        proj.notes.push(ShellNote {
            id: n.id,
            kind: n.kind.clone(),
            msg: n.msg.clone(),
            at: n.at.clone(),
            app: n.app.clone(),
        });
    }
    let notes_front = notes_snapshot
        .first()
        .map(|n| n.id.to_string())
        .unwrap_or_else(|| "0".to_string());
    let notes_unread = state.desktop.notes_unread.get();
    proj.notes_unread = notes_unread;
    // PLAN-012 W7 v1.6：通知面板可见性。
    let notes_visible = state.notification_visible();
    proj.notes_visible = notes_visible;
    fp.push_str(&format!(
        "|notes:{}:{}:{}:{};",
        notes_snapshot.len(),
        notes_front,
        notes_unread,
        if notes_visible { "1" } else { "" }
    ));
    // PLAN-012 W4 v1.6：__dock_pinned_csv 派生串 + 指纹尾段。
    let mut pinned_csv = String::from(",");
    for id in &state.desktop.dock_pinned {
        pinned_csv.push_str(id);
        pinned_csv.push(',');
    }
    if pinned_csv.len() > 1 {
        fp.push_str(&format!(
            "|pinned:{};",
            &pinned_csv[1..pinned_csv.len() - 1]
        ));
    } else {
        fp.push_str("|pinned:;");
    }
    proj.dock_pinned_csv = pinned_csv;
    // PLAN-012 O2：__dock_pinned Obj 数组（icon 宿主自注册表解析；
    // running = 非隐藏运行窗在场）。
    for id in &state.desktop.dock_pinned {
        let icon = state
            .desktop
            .registry_entries
            .iter()
            .find(|e| &e.id == id)
            .map(|e| e.icon.clone())
            .unwrap_or_else(|| "app-window".to_string());
        let running = host
            .wm
            .wins
            .values()
            .any(|v| !v.hidden.get() && v.registry_id.as_deref() == Some(id.as_str()));
        proj.dock_pinned.push(DockPin {
            id: id.clone(),
            icon,
            running,
        });
    }
    // PLAN-024 v1.8：dashboard 可见性投影。
    proj.dashboard_visible = state.dashboard_visible();
    // PLAN-014 W-07：badge 显示串宿主派生（>9 → "9+"、0 → ""）。
    proj.notes_badge = if notes_unread > 9 {
        "9+".to_string()
    } else if notes_unread > 0 {
        notes_unread.to_string()
    } else {
        String::new()
    };
    // PLAN-012 F2 复验：布局名标量投影。
    proj.layout = layout_name.to_string();
    proj.fp = fp;
    proj
}

/// PLAN-027 T-05：解释轨回写（指纹门控保留——fp 不变整组跳写；有变按
/// interpreted_writes 序列整组原子换装 + view_dirty）。desktop 层
/// __wm_running 推送 + RunningSync 召唤随行（PLAN-014 W-04）。行为与
/// 改造前逐写点一致；编译轨替换点 = T-08 ShellSurface::apply_projection。
pub(crate) fn apply_shell_projection_interpreted(
    state: &mut crate::ui::session::DesktopSession,
    shell: crate::ui::session::AppId,
    proj: &crate::ui::shell_projection::ShellProjection,
) {
    use crate::ui::shell_projection::ShellWrite;
    let app = match state.apps.get_mut(&shell) {
        Some(a) => a,
        None => return,
    };
    if matches!(
        app.component.read_state("__wm_fp"),
        Ok(auto_val::Value::Str(ref s)) if s.to_string() == proj.fp
    ) {
        return;
    }
    for w in proj.interpreted_writes() {
        match w {
            ShellWrite::Scalar(k, v) => {
                let _ = app.component.write_state(k, v);
            }
            ShellWrite::Array(k, vs) => {
                let _ = app.component.write_state_vec(k, vs);
            }
        }
    }
    *app.state.view_dirty.borrow_mut() = true;
    // PLAN-014 W-04：__wm_running 注入面扩注 desktop 层 + RunningSync 召唤
    // （宿主写状态不触发 handler；启动失败残态由 Init/重注入求差自愈）。
    // PLAN-035 T-11：__wm_dashboard 投影注入 desktop 层（空白右键菜单
    // 「桌面小组件」checkbox 判据；可见性随 shell 投影差分同步）。
    let dash_visible = state.dashboard_visible();
    // PLAN-035 T-20：sliver tooltip 坐标注入 shell 层（任务栏上缘右角；
    // 坐标锚 popover 的面板落点）。
    {
        let vp = state.host_viewport();
        let usable = crate::ui::layout::usable_rect(vp, state.desktop.dock_edges);
        let tip_x = usable.x + usable.width - 120.0;
        let tip_y = usable.y + usable.height - 46.0;
        if let Some(sapp) = state.apps.get_mut(&shell) {
            let _ = sapp
                .component
                .write_state("__sliver_tip_x", auto_val::Value::Float(tip_x as f64));
            let _ = sapp
                .component
                .write_state("__sliver_tip_y", auto_val::Value::Float(tip_y as f64));
        }
    }
    if let Some(desktop_id) = state.desktop.desktop_app {
        if let Some(dapp) = state.apps.get_mut(&desktop_id) {
            let _ = dapp
                .component
                .write_state("__wm_running", auto_val::Value::str(&proj.running_csv));
            let _ = dapp.component.write_state(
                "__wm_dashboard",
                auto_val::Value::str(if dash_visible { "1" } else { "" }),
            );
            if let Err(err) = dapp.component.bridge_mut().call_handler("RunningSync", &[]) {
                crate::syslog!(
                    SyslogLevel::Error,
                    "host",
                    "[session] desktop RunningSync failed: {err}"
                );
            }
            *dapp.state.view_dirty.borrow_mut() = true;
        }
    }
    // PLAN-035 T-12：面板 faces 状态面随窗开合即时回正——投影 fp 变化
    // 即窗开合/焦点变化，dashboard 可见时重推 faces（关窗 → 重孵化回
    // 归孵化态；开窗 → 升格态），关窗不再连带 face 消失（AC-12）。
    // refresh 自带 dashboard_visible 早退；此处先检避免无谓调用。
    if state.dashboard_visible() {
        refresh_dashboard_panel(state);
    }
}

/// PLAN-040 F3：快照冻结集全量校正（sync 每 tick）——冻结 = 最小化 ‖
/// 隐藏 ‖ 不在当前分区 ‖ 被更高 z 序可见窗矩形相交（部分遮挡即污染
/// 裁剪）。冻结窗：不重抓、缩略不 TTL 过期（末帧保留，OS 惯例）。
pub(super) fn sync_snapshot_frozen(state: &mut crate::ui::session::DesktopSession) {
    let Some(host) = state.host.as_ref() else {
        return;
    };
    let current = host.wm.current_workspace;
    // z 序自顶向下的可见窗矩形集——后出现的窗可遮挡先出现的。被遮挡
    // 的可见窗仍遮挡其下方窗，故遮挡集收录一切"在屏"窗（冻结分级：
    // 被判遮挡的窗自身冻结，但其矩形照常参与下方窗的遮挡判定）。
    let mut above: Vec<(f32, f32, f32, f32)> = Vec::new();
    for wid in host.wm.z_order.iter().rev() {
        let Some(v) = host.wm.wins.get(wid) else {
            continue;
        };
        let (rx, ry, rw, rh) = {
            let r = v.rect.borrow();
            (r.x, r.y, r.width, r.height)
        };
        let offscreen = v.minimized.get() || v.hidden.get() || v.workspace != current;
        let covered = !offscreen
            && above
                .iter()
                .any(|(x, y, w, h)| rx < x + *w && *x < rx + rw && ry < y + *h && *y < ry + rh);
        if !offscreen {
            crate::ui::iced::snapshot::set_frozen(*wid, covered);
            above.push((rx, ry, rw, rh));
        } else {
            crate::ui::iced::snapshot::set_frozen(*wid, true);
        }
    }
}

/// 2026-09-15：workspace_preview 数据发布（原 sync_shell_windows 内联块）。
/// usable 区逐窗 tile（常驻隐藏窗排除），壁纸 #hex 基色；渲染臂直查
/// snapshot 缓存（SWR）。
pub(super) fn publish_workspace_previews(state: &mut crate::ui::session::DesktopSession) {
    sync_snapshot_frozen(state);
    let viewport = state.host_viewport();
    let usable = crate::ui::layout::usable_rect(viewport, state.desktop.dock_edges);
    let mut per_ws: std::collections::BTreeMap<
        String,
        Vec<crate::ui::iced::workspace_preview::PreviewTile>,
    > = Default::default();
    if let Some(host) = state.host.as_ref() {
        for &wid in &host.wm.z_order {
            let Some(v) = host.wm.wins.get(&wid) else {
                continue;
            };
            if v.hidden.get() {
                continue;
            }
            let r = v.rect.borrow();
            per_ws.entry(v.workspace.to_string()).or_default().push(
                crate::ui::iced::workspace_preview::PreviewTile {
                    wid: wid.0,
                    x: r.x - usable.x,
                    y: r.y - usable.y,
                    w: r.width,
                    h: r.height,
                },
            );
        }
    }
    crate::ui::iced::workspace_preview::publish(crate::ui::iced::workspace_preview::Published {
        usable: (usable.width, usable.height),
        wallpaper: crate::ui::iced::workspace_preview::wallpaper_rgb(
            &state.desktop.config.wallpaper_path,
        ),
        workspaces: per_ws,
        wallpaper_src: state.desktop.desktop_wallpaper.clone(),
    });
}







