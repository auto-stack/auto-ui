// Iced renderer - converts abstract View<M> into Iced Elements with style support
//
// Migrated from auto-ui-iced with style integration via IcedStyle adapter.
// Each View variant applies style properties (padding, gap/spacing, font_size,
// text_color, background_color, border, rounded, width, height) where Iced supports them.
// Unsupported properties (margin) are silently skipped.

use crate::ui::app::AppResult;
use crate::ui::component::Component;
use crate::ui::style::{Color, SizeValue, Style, StyleClass};
use crate::ui::style_iced_adapter::{
    IcedAlign, IcedFontSize, IcedFontWeight, IcedJustify, IcedOverflow, IcedShadowSize, IcedSize,
    IcedStyle,
};
use crate::ui::view::View as AbstractView;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, row, scrollable, svg, text,
    text_editor, text_input, tooltip,
};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::ui::dynamic::DynamicComponent;
// PLAN-042 T-02：诊断家族双写（eprintln 载重站点 → `crate::syslog!` 宏，
// stderr 归档层保留 + 环实时层入环）。
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::syslog::SyslogLevel;
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_ui_compiler::aura::{AuraNodeId, SpanInfo};

// Thread-local storage for the last input text value.
// Used by the static code path to pass input text from on_input callbacks
// to Component::on() handlers, since the generic message type M cannot carry String.
thread_local! {
    static INPUT_TEXT: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

// Plan 309 续篇 II: when true, interactive widgets are built WITHOUT their
// event handlers so they don't capture presses/hovers — letting the
// `wrap_debug` mouse_area capture inspect hover/click over EVERY element
// (buttons, inputs, sliders, …). Set once per view build at `dynamic_view`
// entry from `debug_mode && inspect_mode && !alt_held`. Read in `into_iced`
// (to gate handlers) and `wrap_debug` (to gate the capturing mouse_area).
thread_local! {
    static INSPECT_CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Plan 453 T4c（裁定 M1 落地）：原 LAST_MODIFIERS thread-local 已删除——
// 修饰键唯一事实源为 `DesktopState.current_modifiers`：窗口事件订阅把
// `modifiers.bits()` 塞进 `__modifiers_changed` 消息载荷，update 既有臂解析
// 写回；view 直接读字段决定 INSPECT_CAPTURE（施工图 §4）。

/// Helper: is the inspect picker currently in "capture" mode (plain click =
/// inspect over all widgets)?
fn inspect_capture_active() -> bool {
    INSPECT_CAPTURE.with(|c| c.get())
}

/// Static storage for textarea editor contents.
/// Required because iced's `text_editor` widget needs `&'static Content<Renderer>`.
/// Each entry is a leaked Box that lives for the entire process lifetime.
/// We store `&'static mut` but only ever mutate under the Mutex, so the
/// shared references we hand out remain valid.
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref TEXTAREA_CONTENTS: Mutex<std::collections::HashMap<String, &'static mut text_editor::Content>> =
        Mutex::new(std::collections::HashMap::new());
    /// Plan 057 续(keydown 失焦修复):onkeydown.* 派发的 handler 消息键
    /// ("{widget}_{event}",如 PromptBar_AcceptGhost)→ 所属编辑器的 action key
    /// (如 PromptBar_OnInputComplete)。这些 handler 会改 input/ghost/suggestions,
    /// 重建时候选行显隐造成树位移丢焦点 —— update 据此映射触发重聚焦。
    static ref TEXTAREA_KEYDOWN_TO_EDITOR: Mutex<std::collections::HashMap<String, String>> =
        Mutex::new(std::collections::HashMap::new());
}

/// Plan 057 (ash-gui 光标可见): PlainText highlighter 的 to_format —— 把编辑器
/// 文字整体染透明(overlay 高亮层负责显示彩色文字),让 style.value 专职 caret 颜色。
fn text_editor_text_transparent(
    _highlight: &(),
    _theme: &iced::Theme,
) -> iced_widget::core::text::highlighter::Format<iced::Font> {
    iced_widget::core::text::highlighter::Format {
        color: Some(iced::Color::TRANSPARENT),
        font: None,
    }
}

/// Plan 057 续(Tab 补全):把 iced KeyPress 规范化为 .at onkeydown 的键名
/// ("tab"/"up"/"down"/"ctrl.r"/"ctrl.right"…)。修饰键前缀 ctrl./alt./shift.。
fn key_press_to_binding_name(kp: &text_editor::KeyPress) -> String {
    use iced::keyboard::{key, Key};
    let base = match &kp.key {
        Key::Named(n) => match n {
            key::Named::Tab => "tab".to_string(),
            key::Named::ArrowUp => "up".to_string(),
            key::Named::ArrowDown => "down".to_string(),
            key::Named::ArrowLeft => "left".to_string(),
            key::Named::ArrowRight => "right".to_string(),
            key::Named::Enter => "enter".to_string(),
            key::Named::Escape => "escape".to_string(),
            key::Named::Home => "home".to_string(),
            key::Named::End => "end".to_string(),
            key::Named::Delete => "delete".to_string(),
            key::Named::Backspace => "backspace".to_string(),
            key::Named::PageUp => "pageup".to_string(),
            key::Named::PageDown => "pagedown".to_string(),
            key::Named::Space => " ".to_string(),
            _ => String::new(),
        },
        Key::Character(c) => c.to_string(),
        _ => String::new(),
    };
    if base.is_empty() {
        return String::new();
    }
    let mut name = String::new();
    if kp.modifiers.control() {
        name.push_str("ctrl.");
    }
    if kp.modifiers.alt() {
        name.push_str("alt.");
    }
    if kp.modifiers.shift() && matches!(kp.key, Key::Named(_)) {
        name.push_str("shift.");
    }
    name + &base
}

// ═══ Plan 057 续(行内编辑 keymap):快捷键与功能分离 ═════════════════
// 三层架构:.at onkeydown.*(App 语义) → 模式键位表(emacs/vi-*) → iced
// 默认。纯编辑键(移动/词移/kill/yank/transpose)在渲染层直接执行
// (Content::perform 组合 + 合成 on_change 消息同步状态),不经 .at 往返;
// 语义键(Tab 补全/历史/ghost 接受/模式切换)派发 .at handler。

/// 独立 kill-ring(readline 语义:最近 16 条;不污染系统剪贴板)。
lazy_static::lazy_static! {
    static ref TEXTAREA_KILL_RING: Mutex<std::collections::VecDeque<String>> =
        Mutex::new(std::collections::VecDeque::new());
    /// echo 标志:update 检测到消息源自编辑器自身(编辑/光标动作)时置位;
    /// 下一次该 key 的 content 重建保持光标字节偏移(编辑点),而非钉到
    /// value 末尾 —— C-b 后继续打字光标不再被拽走(行内编辑地基)。
    static ref TEXTAREA_ECHO_KEYS: Mutex<std::collections::HashSet<String>> =
        Mutex::new(std::collections::HashSet::new());
}

// PLAN-725 T-02：键入载荷静态消费判定的绑定表面。render_dynamic_view 无
// state 访问，input_state_map（(b) 消费点）经此 thread_local 随帧下发
// （dynamic_view_impl 装配期填充；render 结束即清——防跨帧陈旧读）。
thread_local! {
    static P725_INPUT_BINDINGS: std::cell::RefCell<Option<std::collections::HashMap<String, String>>> =
        const { std::cell::RefCell::new(None) };
}

/// PLAN-725 T-02：编辑器 on_change 闭包的载荷消费判定（绑定表面随帧）。
/// 绑定表缺席时保守返回 true（宁缺勿错——判缺席=携带全文，行为同旧）。
fn editor_onchange_carries_text(event_name: &str) -> bool {
    P725_INPUT_BINDINGS.with(|b| {
        let guard = b.borrow();
        crate::ui::dynamic::input_payload_consumed(event_name, guard.as_ref())
    })
}

/// 与键位无关的行编辑功能抽象。表条目按模式声明,执行统一走 le_execute。
#[derive(Debug, Clone)]
enum LineEditOp {
    /// 纯 iced 原生绑定(Move/Backspace/Enter…)—— 编辑器自己执行,保持
    /// 捕获/选区语义;on_action 照常回投(状态同步 + echo 光标保持)。
    Native(text_editor::Binding<IcedMessage>),
    /// kill 到指定方向(End=行尾 C-k / Home=行首到光标 C-u / Word* =词)。
    Kill(text_editor::Motion),
    /// C-y:kill-ring 末条粘贴。
    Yank,
    /// C-t:交换光标两侧字符(行尾时交换最后两个)。
    TransposeChars,
    /// C-d:空输入(剥离 ghost 后)派发 .at 退出 handler;否则前删一字符。
    DeleteCharOrExit,
    /// vi x:前删一字符;空输入无动作(不退出)。
    DeleteForward,
    /// 语义键派发 .at handler(onkeydown 表)。
    Handler,
    /// 切换编辑模式(合成 SetEditMode 派发,input_value = 目标模式)。
    SetMode(&'static str),
    /// 复合操作(vi:append = 切 insert + 右移)。
    Seq(Vec<LineEditOp>),
    /// vi-normal 的 d 前缀进入 pending(dd/dw/db)。
    BeginPending,
    /// 吞掉按键(捕获但不动作)—— vi-normal 未映射可打印字符的兜底。
    Swallow,
}

fn le_push_kill(text: String) {
    let mut ring = TEXTAREA_KILL_RING.lock().unwrap();
    if ring.back().map(|b| b == &text).unwrap_or(false) {
        return; // 连续 kill 合并(readline 语义)
    }
    ring.push_back(text);
    while ring.len() > 16 {
        ring.pop_front();
    }
}

/// Content 光标 → 全文字节偏移(列即行内字节偏移,cosmic-text 语义)。
fn le_cursor_offset(c: &text_editor::Content) -> usize {
    let cur = c.cursor();
    let mut off = 0usize;
    for (i, line) in c.lines().enumerate() {
        if i == cur.position.line {
            return off + cur.position.column.min(line.text.len());
        }
        off += line.text.len() + 1; // '\n'
    }
    off
}

/// 全文字节偏移 → move_to(单/多行,列按字节)。
fn le_move_to_offset(c: &mut text_editor::Content, text: &str, off: usize) {
    let pos = le_offset_to_pos(text, off);
    c.move_to(text_editor::Cursor {
        position: pos,
        selection: None,
    });
}

fn le_offset_to_pos(text: &str, off: usize) -> iced_widget::core::text::editor::Position {
    let off = off.min(text.len());
    let mut line = 0usize;
    let mut before = 0usize;
    for (i, l) in text.split('\n').enumerate() {
        if before + l.len() >= off {
            line = i;
            break;
        }
        before += l.len() + 1;
        line = i + 1;
    }
    iced_widget::core::text::editor::Position {
        line,
        column: off.saturating_sub(before),
    }
}

fn ch_len_at(s: &str, i: usize) -> usize {
    s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1)
}

/// 在 key 的 Content 上删除显式字节范围 → 入 kill-ring。
/// 返回删除的文本(None = 范围空)。不用 cosmic 的 Select(motion)——其
/// 选区锚点会吸附词边界(实测起点 = 光标+1,跨空格错位);显式范围精确。
fn le_kill_range(key: &str, range: std::ops::Range<usize>) -> Option<String> {
    if range.start >= range.end {
        return None;
    }
    let mut map = TEXTAREA_CONTENTS.lock().unwrap();
    let content = map.get_mut(key)?;
    let text = content.text();
    let range = range.start.min(text.len())..range.end.min(text.len());
    if range.start >= range.end {
        return None;
    }
    let killed = text[range.clone()].to_string();
    // 不用 cosmic 选区删除 —— 显式选区 + Edit::Delete 的实测语义比设定
    // 范围多删一个字形起始(Affinity::Before)。改为手动拼接重建,光标
    // 落在删除起点(编辑点),完全确定性。
    let new_text = format!("{}{}", &text[..range.start], &text[range.end..]);
    **content = text_editor::Content::with_text(&new_text);
    le_move_to_offset(content, &new_text, range.start);
    Some(killed)
}

/// readline 语义的 kill 方向计算(基于当前光标,自算边界):
/// End = 光标→行尾;Home = 行首→光标;WordRight = 跳过空白再吃一个词;
/// WordLeft(unix-word-rubout)= 连空白带词向后吃。
fn le_kill_motion(key: &str, motion: text_editor::Motion) -> Option<String> {
    let (text, cur) = {
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        let c = map.get(key)?;
        (c.text(), le_cursor_offset(c))
    };
    let line_start = text[..cur].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[cur..]
        .find('\n')
        .map(|i| cur + i)
        .unwrap_or(text.len());
    let range = match motion {
        text_editor::Motion::End => cur..line_end,
        text_editor::Motion::Home => line_start..cur,
        text_editor::Motion::DocumentEnd => cur..text.len(),
        text_editor::Motion::DocumentStart => 0..cur,
        text_editor::Motion::WordRight => {
            // 跳过空白,吃一个词(非空白连续段),停在词尾。
            let b = text.as_bytes();
            let mut i = cur;
            while i < b.len() && (b[i] as char).is_whitespace() {
                i += 1;
            }
            while i < b.len() && !(b[i] as char).is_whitespace() {
                i += 1;
            }
            cur..i
        }
        _ => {
            // WordLeft / 其他:向后连空白带词一起吃。
            let mut i = cur;
            let b = text.as_bytes();
            while i > 0 && (b[i - 1] as char).is_whitespace() {
                i -= 1;
            }
            while i > 0 && !(b[i - 1] as char).is_whitespace() {
                i -= 1;
            }
            i..cur
        }
    };
    le_kill_range(key, range)
}

/// 执行复合原生操作;文本变化时返回合成 on_change 消息(Custom 派发)。
fn le_execute(
    key: &str,
    op: LineEditOp,
    on_change: &IcedMessage,
) -> Option<text_editor::Binding<IcedMessage>> {
    use text_editor::Binding;
    let sync = |new_text: &str| -> Binding<IcedMessage> {
        Binding::Custom(IcedMessage {
            widget: on_change.widget.clone(),
            event: on_change.event.clone(),
            input_value: Some(strip_ghost_suffix(key, new_text)),
        })
    };
    let dispatch = |event: &str, val: &str| -> Binding<IcedMessage> {
        Binding::Custom(IcedMessage {
            widget: on_change.widget.clone(),
            event: event.to_string(),
            input_value: Some(val.to_string()),
        })
    };
    match op {
        LineEditOp::Native(b) => Some(b),
        LineEditOp::Swallow => Some(Binding::Sequence(vec![])),
        LineEditOp::Seq(ops) => {
            // 复合:依次执行,返回最后一个产生的绑定。
            let mut last = Binding::Sequence(vec![]);
            for sub in ops {
                if let Some(b) = le_execute(key, sub, on_change) {
                    last = b;
                }
            }
            Some(last)
        }
        LineEditOp::Kill(motion) => match le_kill_motion(key, motion) {
            Some(k) => {
                le_push_kill(k);
                let text = TEXTAREA_CONTENTS
                    .lock()
                    .unwrap()
                    .get(key)
                    .map(|c| c.text())
                    .unwrap_or_default();
                Some(sync(&text))
            }
            None => Some(Binding::Sequence(vec![])),
        },
        LineEditOp::Yank => {
            let yank_text = TEXTAREA_KILL_RING.lock().unwrap().back().cloned();
            match yank_text {
                Some(t) => {
                    let text = textarea_perform_action(
                        key,
                        text_editor::Action::Edit(text_editor::Edit::Paste(std::sync::Arc::new(t))),
                    );
                    Some(sync(&text))
                }
                None => Some(Binding::Sequence(vec![])),
            }
        }
        LineEditOp::TransposeChars => {
            let mut map = TEXTAREA_CONTENTS.lock().unwrap();
            let content = map.get_mut(key)?;
            let text = content.text();
            if text.chars().count() < 2 {
                return Some(Binding::Sequence(vec![]));
            }
            let off = le_cursor_offset(content).min(text.len());
            // readline:光标在行尾时交换最后两个字符,否则交换光标前后字符。
            let (a, b, c1, c2) = if off >= text.len() {
                // 行尾:最后两个字符
                let mut idx = text.char_indices().collect::<Vec<_>>();
                idx.reverse();
                let last_start = idx.first().map(|(i, _)| *i).unwrap_or(0);
                let second_start = idx.get(1).map(|(i, _)| *i).unwrap_or(0);
                (
                    second_start,
                    text.len(),
                    text[second_start..last_start].chars().next().unwrap_or(' '),
                    text[last_start..].chars().next().unwrap_or(' '),
                )
            } else {
                let b_end = (off + ch_len_at(&text, off)).min(text.len());
                let mut a_start = 0;
                for (i, _) in text.char_indices() {
                    if i + ch_len_at(&text, i) >= off {
                        a_start = i;
                        break;
                    }
                }
                (
                    a_start,
                    b_end,
                    text[a_start..off].chars().last().unwrap_or(' '),
                    text[off..].chars().next().unwrap_or(' '),
                )
            };
            let _ = (a, b);
            // 选区 [a, b) → Delete → 按 c2、c1 顺序插入(净效果 = 交换)。
            content.move_to(text_editor::Cursor {
                position: le_offset_to_pos(&text, a),
                selection: Some(le_offset_to_pos(&text, b)),
            });
            content.perform(text_editor::Action::Edit(text_editor::Edit::Delete));
            content.perform(text_editor::Action::Edit(text_editor::Edit::Insert(c2)));
            content.perform(text_editor::Action::Edit(text_editor::Edit::Insert(c1)));
            let new_text = content.text();
            drop(map);
            Some(sync(&new_text))
        }
        LineEditOp::DeleteCharOrExit => {
            let mut map = TEXTAREA_CONTENTS.lock().unwrap();
            let content = map.get_mut(key)?;
            let stripped_len = strip_ghost_suffix(key, &content.text()).len();
            if stripped_len == 0 {
                drop(map);
                Some(dispatch("OnCtrlD", ""))
            } else {
                content.perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                let t = content.text();
                drop(map);
                Some(sync(&t))
            }
        }
        LineEditOp::DeleteForward => {
            let mut map = TEXTAREA_CONTENTS.lock().unwrap();
            let content = map.get_mut(key)?;
            if content.text().is_empty() {
                return Some(Binding::Sequence(vec![]));
            }
            content.perform(text_editor::Action::Edit(text_editor::Edit::Delete));
            let t = content.text();
            drop(map);
            Some(sync(&t))
        }
        LineEditOp::SetMode(mode) => Some(dispatch("SetEditMode", mode)),
        LineEditOp::BeginPending => Some(Binding::Sequence(vec![])), // 闭包层处理
        LineEditOp::Handler => None, // 交由外层回落(.at 未声明则默认)
    }
}

/// 各模式键位表。键名与 key_press_to_binding_name 规范一致。
fn line_edit_keymap(mode: &str) -> std::collections::HashMap<String, LineEditOp> {
    use text_editor::{Binding, Motion};
    let nat = |b: Binding<IcedMessage>| LineEditOp::Native(b);
    let mut m: std::collections::HashMap<String, LineEditOp> = std::collections::HashMap::new();
    // ── Emacs(readline)全集 ──
    m.insert("ctrl.a".into(), nat(Binding::Move(Motion::Home)));
    m.insert("ctrl.b".into(), nat(Binding::Move(Motion::Left)));
    m.insert("ctrl.e".into(), nat(Binding::Move(Motion::End)));
    m.insert("ctrl.f".into(), nat(Binding::Move(Motion::Right)));
    m.insert("ctrl.h".into(), nat(Binding::Backspace));
    m.insert("alt.b".into(), nat(Binding::Move(Motion::WordLeft)));
    m.insert("alt.f".into(), nat(Binding::Move(Motion::WordRight)));
    m.insert("ctrl.k".into(), LineEditOp::Kill(Motion::End));
    m.insert("ctrl.u".into(), LineEditOp::Kill(Motion::Home));
    m.insert("alt.d".into(), LineEditOp::Kill(Motion::WordRight));
    m.insert("alt.backspace".into(), LineEditOp::Kill(Motion::WordLeft));
    m.insert("ctrl.w".into(), LineEditOp::Kill(Motion::WordLeft));
    m.insert("ctrl.y".into(), LineEditOp::Yank);
    m.insert("ctrl.t".into(), LineEditOp::TransposeChars);
    m.insert("ctrl.d".into(), LineEditOp::DeleteCharOrExit);
    m.insert("enter".into(), nat(Binding::Enter));
    match mode {
        "vi-insert" => {
            // insert 态 = Emacs 等价 + Esc 回 normal。
            m.insert("escape".into(), LineEditOp::SetMode("vi-normal"));
        }
        "vi-normal" => {
            m.clear();
            m.insert("h".into(), nat(Binding::Move(Motion::Left)));
            m.insert("l".into(), nat(Binding::Move(Motion::Right)));
            m.insert("0".into(), nat(Binding::Move(Motion::Home)));
            m.insert("^".into(), nat(Binding::Move(Motion::Home)));
            m.insert("$".into(), nat(Binding::Move(Motion::End)));
            m.insert("w".into(), nat(Binding::Move(Motion::WordRight)));
            m.insert("b".into(), nat(Binding::Move(Motion::WordLeft)));
            m.insert("e".into(), nat(Binding::Move(Motion::WordRight)));
            m.insert("G".into(), nat(Binding::Move(Motion::End)));
            m.insert("x".into(), LineEditOp::DeleteForward);
            m.insert("X".into(), nat(Binding::Backspace));
            m.insert("D".into(), LineEditOp::Kill(Motion::End));
            m.insert("d".into(), LineEditOp::BeginPending);
            m.insert("i".into(), LineEditOp::SetMode("vi-insert"));
            m.insert(
                "I".into(),
                LineEditOp::Seq(vec![
                    LineEditOp::SetMode("vi-insert"),
                    LineEditOp::Native(Binding::Move(Motion::Home)),
                ]),
            );
            m.insert(
                "a".into(),
                LineEditOp::Seq(vec![
                    LineEditOp::SetMode("vi-insert"),
                    LineEditOp::Native(Binding::Move(Motion::Right)),
                ]),
            );
            m.insert(
                "A".into(),
                LineEditOp::Seq(vec![
                    LineEditOp::SetMode("vi-insert"),
                    LineEditOp::Native(Binding::Move(Motion::End)),
                ]),
            );
            m.insert("enter".into(), nat(Binding::Enter));
        }
        _ => {}
    }
    m
}

/// Plan 057 续(Tab 补全):给编辑器挂 key_binding — 声明过的 onkeydown 键
/// 以 Binding::Custom 派发 handler(不落入编辑器默认行为,Tab 不再被静默
/// 丢弃、up/down 走历史导航);未声明的键回落 iced 默认(输入/Enter/退格…)。
fn with_textarea_keydown<H>(
    editor: iced::widget::TextEditor<'static, H, IcedMessage>,
    keydown: std::collections::HashMap<String, IcedMessage>,
    ta_key: &str,
    keymap_mode: &str,
    on_change: Option<IcedMessage>,
) -> iced::widget::TextEditor<'static, H, IcedMessage>
where
    H: iced_widget::core::text::Highlighter,
{
    if keydown.is_empty() && keymap_mode == "emacs" {
        return editor; // 纯默认路径(无声明且默认模式),保持零开销
    }
    if keydown.is_empty() {
        return editor;
    }
    // Plan 057 续(keydown 失焦修复):登记 事件键 → 编辑器 action key,
    // update 里据此为 keydown 派发的 handler 触发重聚焦。
    {
        let mut reg = TEXTAREA_KEYDOWN_TO_EDITOR.lock().unwrap();
        for msg in keydown.values() {
            reg.insert(format!("{}_{}", msg.widget, msg.event), ta_key.to_string());
        }
    }
    // Plan 057 续(行内编辑 keymap):派发优先级 .at onkeydown 声明(App
    // 语义,含 ghost 门控) → 模式键位表(emacs/vi-*) → vi-normal 吞可打印
    // → iced 默认。复合原生操作(kill/yank/transpose)经 le_execute 在
    // Content 上直接执行并合成 on_change 消息同步状态。
    let mode = keymap_mode.to_string();
    let action_key = ta_key.to_string();
    let pending: std::rc::Rc<std::cell::RefCell<Option<&'static str>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));
    let pending2 = pending.clone();
    editor.key_binding(move |kp| {
        let name = key_press_to_binding_name(&kp);
        // vi pending(d 前缀):dd/dw/db;其他键取消 pending。
        if let Some(pfx) = pending2.borrow_mut().take() {
            if pfx == "d" {
                let op = match name.as_str() {
                    "d" => Some(LineEditOp::Kill(text_editor::Motion::Home)),
                    "w" => Some(LineEditOp::Kill(text_editor::Motion::WordRight)),
                    "b" => Some(LineEditOp::Kill(text_editor::Motion::WordLeft)),
                    _ => None,
                };
                if pfx == "d" && name == "d" {
                    // dd = kill 整行:先 kill 到行首,再 kill 到行尾
                    if let Some(oc) = on_change.as_ref() {
                        let _ = le_execute(
                            &action_key,
                            LineEditOp::Kill(text_editor::Motion::Home),
                            oc,
                        );
                        return le_execute(
                            &action_key,
                            LineEditOp::Kill(text_editor::Motion::End),
                            oc,
                        );
                    }
                    return Some(text_editor::Binding::Sequence(vec![]));
                }
                if let Some(op) = op {
                    if let Some(oc) = on_change.as_ref() {
                        return le_execute(&action_key, op, oc);
                    }
                }
                return Some(text_editor::Binding::Sequence(vec![]));
            }
        }
        if !name.is_empty() {
            // 1. .at 声明优先
            if let Some(msg) = keydown.get(&name) {
                return Some(text_editor::Binding::Custom(msg.clone()));
            }
            // 2. 模式表
            if let Some(op) = line_edit_keymap(&mode).get(&name).cloned() {
                match op {
                    LineEditOp::Handler => {}
                    LineEditOp::BeginPending => {
                        *pending2.borrow_mut() = Some("d");
                        return Some(text_editor::Binding::Sequence(vec![]));
                    }
                    op => {
                        if let Some(oc) = on_change.as_ref() {
                            if let Some(b) = le_execute(&action_key, op, oc) {
                                return Some(b);
                            }
                        }
                    }
                }
            }
        }
        // 3. vi-normal 兜底:吞掉可打印字符
        if mode == "vi-normal" {
            if let Some(t) = kp.text.as_deref() {
                if t.chars().any(|c| !c.is_control()) {
                    return Some(text_editor::Binding::Sequence(vec![]));
                }
            }
        }
        // 4. iced 默认
        text_editor::Binding::from_key_press(kp)
    })
}

/// Plan 057 续(富文本输入):textarea 的 on_action 事件接线 — Enter 走
/// on_submit(onenter),其余走 on_change(oninput);input_value 经 ghost 剥离。
/// 泛型于 Highlighter,使 PlainText(存量)与 SpanHighlighter(富文本)共用。
fn wire_textarea_actions<H>(
    editor: iced::widget::TextEditor<'static, H, IcedMessage>,
    on_change: Option<IcedMessage>,
    on_submit: Option<IcedMessage>,
) -> iced::Element<'static, IcedMessage>
where
    H: iced_widget::core::text::Highlighter,
{
    // In inspect-capture mode, render read-only (no on_action) so
    // wrap_debug's mouse_area can capture hover/click.
    let on_change = if inspect_capture_active() {
        None
    } else {
        on_change
    };
    let on_submit = if inspect_capture_active() {
        None
    } else {
        on_submit
    };
    // Plan 053 M4: Enter fires the on_submit (onenter) handler instead of
    // on_change. Plan 057 续:ghost 嵌入内容,派发前剥离已知后缀。
    let is_enter = |action: &text_editor::Action| {
        matches!(action, text_editor::Action::Edit(text_editor::Edit::Enter))
    };
    if let Some(msg) = on_change {
        let msg_clone = msg.clone();
        let submit_clone = on_submit.clone();
        editor
            .on_action(move |action| {
                let action_key = format!("{}_{}", msg_clone.widget, msg_clone.event);
                if is_enter(&action) {
                    if let Some(sm) = submit_clone.clone() {
                        let text = strip_ghost_suffix(
                            &action_key,
                            &textarea_perform_action(&action_key, action),
                        );
                        IcedMessage {
                            widget: sm.widget.clone(),
                            event: sm.event.clone(),
                            input_value: Some(text),
                        }
                    } else {
                        let text = strip_ghost_suffix(
                            &action_key,
                            &textarea_perform_action(&action_key, action),
                        );
                        IcedMessage {
                            widget: msg_clone.widget.clone(),
                            event: msg_clone.event.clone(),
                            input_value: Some(text),
                        }
                    }
                } else {
                    let text = strip_ghost_suffix(
                        &action_key,
                        &textarea_perform_action(&action_key, action),
                    );
                    IcedMessage {
                        widget: msg_clone.widget.clone(),
                        event: msg_clone.event.clone(),
                        input_value: Some(text),
                    }
                }
            })
            .into()
    } else if let Some(sm) = on_submit {
        let sm_clone = sm.clone();
        let action_key = format!("{}_{}", sm.widget, sm.event);
        editor
            .on_action(move |action| {
                let enter = is_enter(&action);
                let text =
                    strip_ghost_suffix(&action_key, &textarea_perform_action(&action_key, action));
                if enter {
                    IcedMessage {
                        widget: sm_clone.widget.clone(),
                        event: sm_clone.event.clone(),
                        input_value: Some(text),
                    }
                } else {
                    IcedMessage {
                        widget: sm_clone.widget.clone(),
                        event: sm_clone.event.clone(),
                        input_value: Some(text),
                    }
                }
            })
            .into()
    } else {
        editor.into()
    }
}

// ── Plan 057 续(富文本输入):原生着色 textarea ────────────────────────
// 语法着色不再走「透明文字 + 叠加层」,而是 text_editor 原生 Highlighter:
// (text, kind) 连续段 + ghost 建议后缀 → 逐段颜色/字重直接落进编辑器文字。

/// 语义着色段类型 — 对齐 .at DoTokenize 的 kind,外加 Ghost(灰色建议)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Command,
    ExternalCmd,
    String,
    Variable,
    Operator,
    Redirect,
    Comment,
    Flag,
    Whitespace,
    Arg,
    Ghost,
    /// PLAN-493: mention 命中段（textarea mentions 能力）——builder 期
    /// mention_segments 产出 kind "mention"。
    Mention,
    Plain,
}

fn parse_span_kind(kind: &str) -> SpanKind {
    match kind {
        "Command" => SpanKind::Command,
        "ExternalCmd" => SpanKind::ExternalCmd,
        "String" => SpanKind::String,
        "Variable" => SpanKind::Variable,
        "Operator" => SpanKind::Operator,
        "Redirect" => SpanKind::Redirect,
        "Comment" => SpanKind::Comment,
        "Flag" => SpanKind::Flag,
        "Whitespace" => SpanKind::Whitespace,
        "Arg" => SpanKind::Arg,
        "Ghost" => SpanKind::Ghost,
        "mention" => SpanKind::Mention,
        _ => SpanKind::Plain,
    }
}

/// 按行预切的着色段(字节范围已重定位到行内)。Highlighter 的 Settings。
pub type SpanLines = Vec<Vec<(std::ops::Range<usize>, SpanKind)>>;

/// 预计算着色的 Highlighter:lines 全量替换(update 时重置行游标,从行 0
/// 重着色 —— iced_highlighter 的 update 同款约定),highlight_line 顺序消费。
/// Plan 058(着色修复):Settings 携带完整文本参与相等比较。此前仅
/// SpanLines(范围+kind)时,"ls Cargo.toml" 与 "ls Cargo.lock" 的 spans
/// 完全相等(同长度同类型)→ iced settings-diff 判定无变化 → 不调
/// update()/不重置行游标 → cosmic 增量着色跳过变更行 → 残留旧属性
/// (Tab 轮换时 ls 时而普通色的根因)。文本入 Settings 后任何文本变化
/// 都触发 update + 重置,增量契约恢复。
#[derive(Debug, Clone, PartialEq)]
pub struct SpanSettings {
    pub text: String,
    pub lines: SpanLines,
}

pub struct SpanHighlighter {
    lines: SpanLines,
    line_idx: usize,
}

impl iced_widget::core::text::Highlighter for SpanHighlighter {
    type Settings = SpanSettings;
    type Highlight = SpanKind;
    type Iterator<'a> = std::vec::IntoIter<(std::ops::Range<usize>, SpanKind)>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            lines: settings.lines.clone(),
            line_idx: 0,
        }
    }

    fn update(&mut self, settings: &Self::Settings) {
        // 整体替换 → 重置游标,让增量 highlight 从行 0 重喂。
        self.lines = settings.lines.clone();
        self.line_idx = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.line_idx = line;
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let idx = self.line_idx;
        self.line_idx += 1;
        self.lines.get(idx).cloned().unwrap_or_default().into_iter()
    }

    fn current_line(&self) -> usize {
        self.line_idx
    }
}

/// kind → Format(颜色/字重)。调色板对齐 .at 视图层原有的 Tailwind 语义
/// (emerald-400/sky-300/amber-300/red-400/pink-400/gray-400/purple-300)。
fn span_kind_to_format(
    highlight: &SpanKind,
    _theme: &iced::Theme,
) -> iced_widget::core::text::highlighter::Format<iced::Font> {
    let c = |r: u8, g: u8, b: u8| iced::Color::from_rgb8(r, g, b);
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    };
    match highlight {
        SpanKind::Command => iced_widget::core::text::highlighter::Format {
            color: Some(c(52, 211, 153)), // emerald-400
            font: Some(bold),
        },
        SpanKind::ExternalCmd => iced_widget::core::text::highlighter::Format {
            color: Some(c(125, 211, 252)), // sky-300
            font: None,
        },
        SpanKind::String => iced_widget::core::text::highlighter::Format {
            color: Some(c(252, 211, 77)), // amber-300
            font: None,
        },
        SpanKind::Variable => iced_widget::core::text::highlighter::Format {
            color: Some(c(248, 113, 113)), // red-400
            font: None,
        },
        SpanKind::Operator => iced_widget::core::text::highlighter::Format {
            color: Some(c(244, 114, 182)), // pink-400
            font: Some(bold),
        },
        SpanKind::Redirect | SpanKind::Comment => iced_widget::core::text::highlighter::Format {
            color: Some(c(156, 163, 175)), // gray-400
            font: None,
        },
        SpanKind::Flag => iced_widget::core::text::highlighter::Format {
            color: Some(c(216, 180, 254)), // purple-300
            font: None,
        },
        SpanKind::Ghost => iced_widget::core::text::highlighter::Format {
            // 低饱和半透明灰 — 对齐 vue 的 text-muted-foreground/35。
            color: Some(iced::Color::from_rgba(0.45, 0.48, 0.54, 0.45)),
            font: None,
        },
        SpanKind::Mention => iced_widget::core::text::highlighter::Format {
            // PLAN-493: mention 前景蓝 — 对齐 musk 的
            // text-[hsl(220_90%_56%)]（≈ blue-600）。iced Format 无 per-span
            // 背景，web 侧的 bg tint 不复刻（跨端差异登记）。
            color: Some(c(37, 99, 235)),
            font: None,
        },
        SpanKind::Whitespace | SpanKind::Arg | SpanKind::Plain => {
            iced_widget::core::text::highlighter::Format::default()
        }
    }
}

/// (text, kind) 连续段 + ghost → 按行预切的字节范围段。spans 必须无缝覆盖
/// value(DoTokenize 语义);不满足时返回空(不着色,防错位)。
fn build_span_lines(highlight: &[(String, String)], value: &str, ghost: &str) -> SpanSettings {
    let mut full: Vec<(std::ops::Range<usize>, SpanKind)> = Vec::new();
    let mut concat = String::new();
    let mut off = 0usize;
    for (text, kind) in highlight {
        concat.push_str(text);
        let end = off + text.len();
        if end > off {
            full.push((off..end, parse_span_kind(kind)));
        }
        off = end;
    }
    // 防线:段文本拼接必须恰为 value —— CJK 字节钳制等来源可能造成缺口/重叠。
    if concat != value {
        return SpanSettings {
            text: String::new(),
            lines: Vec::new(),
        };
    }
    if !ghost.is_empty() {
        full.push((value.len()..value.len() + ghost.len(), SpanKind::Ghost));
    }
    let content = format!("{}{}", value, ghost);
    // 切行:段跨 \n 时拆成行内片段(Whitespace 段可含续行换行)。
    let mut lines: SpanLines = vec![Vec::new()];
    let mut line_start = 0usize;
    for (r, kind) in full {
        let mut s = r.start;
        while s < r.end {
            match content[s..r.end].find('\n') {
                Some(rel) => {
                    let nl = s + rel;
                    if nl > s {
                        lines
                            .last_mut()
                            .unwrap()
                            .push((s - line_start..nl - line_start, kind));
                    }
                    lines.push(Vec::new());
                    line_start = nl + 1;
                    s = nl + 1;
                }
                None => {
                    lines
                        .last_mut()
                        .unwrap()
                        .push((s - line_start..r.end - line_start, kind));
                    s = r.end;
                }
            }
        }
    }
    SpanSettings {
        text: content,
        lines,
    }
}

/// Plan 446 批五 U3: textarea 超大内容降级门槛(字节)。
///
/// 现场机理(auto-os-config roles 详情态,§P/U3):sidecar 数据被转义翻倍
/// bug 损坏成 ~1.31M 连续反斜杠进入 textarea(iced `text_editor`)——
/// cosmic-text 对 `-`/`\` 等字形的整形/布局代价约为常观数字的百倍量级
/// (corpus 实测近似线性 ≈60µs/字符:5K≈0.3s / 80K≈5s / 320K≈20s /
/// 655K+ 每帧分钟级),iced 事件循环被单帧渲染永久占用 —— 截图通道 10s
/// 超时只是最显眼症状,实质是整个 UI 冻结。分行/零宽断行符均无效(代价
/// 按字符计,与段落结构无关,corpus `ta_lines`/ZWSP 变体实证)。
///
/// 处方:超过本门槛的 textarea 值不再进编辑器,降级为**只读预览**
/// (`oversized_textarea_preview`:提示行 + 截断快照)。无编辑器即无
/// onchange 回流 —— store 原值完整保留,不存在截断落盘风险;正常尺寸
/// (<64KB)的编辑路径零变化。阈值取 64KB:现场健康 sidecar 为 KB 量级,
/// 误伤面可忽略。
const TEXTAREA_INLINE_EDIT_CAP: usize = 64 * 1024;

/// 预览快照展示的头部长度(字符)。只读预览是"看",不是"编"——完整内容
/// 始终在 store,外部编辑器工作流不受影响。
const TEXTAREA_PREVIEW_HEAD_CHARS: usize = 32 * 1024;

/// 判定 textarea 值是否超过内联编辑门槛(降级只读预览)。
pub(crate) fn textarea_needs_preview(value: &str) -> bool {
    value.len() > TEXTAREA_INLINE_EDIT_CAP
}

/// 超大 textarea 值的只读预览:提示行 + 截断快照(Plan 446 批五 U3)。
/// 泛型于消息类型:纯展示元素,不携带任何事件。
pub(crate) fn oversized_textarea_preview<M: Clone + std::fmt::Debug + 'static>(
    value: &str,
) -> iced::Element<'static, M> {
    let total_chars = value.chars().count();
    let head: String = value.chars().take(TEXTAREA_PREVIEW_HEAD_CHARS).collect();
    let hint = format!(
        "(autoui) value too large for inline editing ({} chars / {} bytes) — read-only preview of the first {} chars; full value is preserved in state.",
        total_chars,
        value.len(),
        TEXTAREA_PREVIEW_HEAD_CHARS
    );
    // 预览文本必须 Fill 宽度:text 默认 Shrink,其 min 宽 = 最长不可断行
    // (病态数据即百万像素),会把宿主行整体挤出窗口(实测主区只剩标题)。
    iced::widget::column![
        iced::widget::text(hint).size(11).width(iced::Length::Fill),
        iced::widget::text(head).width(iced::Length::Fill),
    ]
    .width(iced::Length::Fill)
    .into()
}

/// Get or create a `&'static text_editor::Content` for the given key, synced to `value`.
fn get_textarea_content(key: &str, value: &str) -> &'static text_editor::Content {
    // Phase 1: ensure the entry exists (under lock)
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        map.entry(key.to_string())
            .or_insert_with(|| Box::leak(Box::new(text_editor::Content::with_text(value))));
    }
    // Phase 2: update content in-place (under lock) — ONLY when the text
    // actually changed. Unconditionally replacing on every render reset the
    // cursor to offset 0 each frame, so every freshly typed character was
    // inserted at the START of the line (typing "ls" showed "sl").
    // When the value did change (e.g. handler wrote state), rebuild and park
    // the caret at the document end — the sane default for programmatic sets.
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        if let Some(content) = map.get_mut(key) {
            if content.text() != value {
                **content = text_editor::Content::with_text(value);
                content.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
            }
        }
    }
    // Phase 3: get a raw pointer under lock, return as &'static outside lock.
    // SAFETY: The Box is leaked so the allocation lives for 'static.
    // We only mutate under the Mutex. The raw pointer is derived from
    // a &'static mut that came from Box::leak, so it remains valid.
    let ptr: *const text_editor::Content;
    {
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        ptr = map.get(key).map(|c| &**c as *const _).unwrap();
    }
    // SAFETY: ptr points to a leaked Box that lives for 'static.
    unsafe { &*ptr }
}

/// Perform an action on a textarea content and return the resulting text.
fn textarea_perform_action(key: &str, action: text_editor::Action) -> String {
    let mut map = TEXTAREA_CONTENTS.lock().unwrap();
    if let Some(content) = map.get_mut(key) {
        content.perform(action);
        content.text()
    } else {
        String::new()
    }
}

// ── Plan 057 续(富文本输入):ghost 嵌入内容 + 编辑事件剥离 ────────────

/// 每个 textarea key 当前嵌入的 ghost 后缀(渲染时写入,编辑派发时读)。
lazy_static::lazy_static! {
    static ref TEXTAREA_GHOSTS: Mutex<std::collections::HashMap<String, String>> =
        Mutex::new(std::collections::HashMap::new());
    /// Plan 057 续(Ctrl+F 光标修复):每个 key 上次渲染的 (value, ghost)。
    /// 拼接文本相同但 (value,ghost) 边界移动时(如 Ctrl+F 接受全部 ghost:
    /// "ec"+"ho…" → "echo…"+""),content 文本不变但光标必须重定位到新
    /// input 末尾 —— 树位移重置的编辑器状态也会被全量重建纠正。
    static ref TEXTAREA_LAST_INPUT: Mutex<std::collections::HashMap<String, (String, String)>> =
        Mutex::new(std::collections::HashMap::new());
}

/// 富文本版内容缓存:content = value + ghost(ghost 经 Highlighter 染灰)。
/// value 或 ghost 变化才重建;重建后光标置于 **value 末尾**(DocumentEnd 再
/// 回退 ghost 字符数),避免后续键入落到 ghost 之后。
fn get_textarea_content_rich(key: &str, value: &str, ghost: &str) -> &'static text_editor::Content {
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        map.entry(key.to_string())
            .or_insert_with(|| Box::leak(Box::new(text_editor::Content::with_text(value))));
    }
    {
        let mut ghosts = TEXTAREA_GHOSTS.lock().unwrap();
        ghosts.insert(key.to_string(), ghost.to_string());
    }
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        let want = format!("{}{}", value, ghost);
        // Plan 057 续(Ctrl+F 光标修复):重建条件从「拼接文本变化」放宽为
        // 「(value, ghost) 变化」。Ctrl+F 接受全部 ghost 后拼接文本不变,
        // 但 input/ghost 边界移动了 —— 不重建的话光标停在旧 input 末尾
        // (或树位移重置后的行首),而期望在完整 input 末尾。
        let boundary_moved = {
            let mut last = TEXTAREA_LAST_INPUT.lock().unwrap();
            let moved = last
                .get(key)
                .map(|(v, g)| v != value || g != ghost)
                .unwrap_or(true);
            last.insert(key.to_string(), (value.to_string(), ghost.to_string()));
            moved
        };
        if let Some(content) = map.get_mut(key) {
            if content.text() != want || boundary_moved {
                // Plan 057 续(行内编辑):echo 重建(编辑器自身编辑/移动引起)
                // 保持光标字节偏移(编辑点,钳回 value 区);外部变更(历史/
                // 补全/ghost 接受)维持 value 末尾 —— C-b 后打字不再被拽尾。
                let echo = TEXTAREA_ECHO_KEYS.lock().unwrap().remove(key);
                let preserve = if echo {
                    Some(le_cursor_offset(content))
                } else {
                    None
                };
                **content = text_editor::Content::with_text(&want);
                match preserve {
                    Some(off) => {
                        le_move_to_offset(content, &want, off.min(value.len()));
                    }
                    None => {
                        content
                            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                        for _ in 0..ghost.chars().count() {
                            content.perform(text_editor::Action::Move(text_editor::Motion::Left));
                        }
                    }
                }
            }
        }
    }
    let ptr: *const text_editor::Content;
    {
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        ptr = map.get(key).map(|c| &**c as *const _).unwrap();
    }
    // SAFETY: 同 get_textarea_content —— Box::leak 的 'static 分配,仅锁内变更。
    unsafe { &*ptr }
}

/// 编辑事件剥离:新内容若以本编辑器已知 ghost 后缀结尾 → 剥掉;否则(用户
/// 改写/选中覆盖了 ghost)整段视为真实输入,ghost 由状态重算自愈。
fn strip_ghost_suffix(key: &str, text: &str) -> String {
    let ghosts = TEXTAREA_GHOSTS.lock().unwrap();
    if let Some(g) = ghosts.get(key) {
        if !g.is_empty() && text.len() >= g.len() && text.ends_with(g.as_str()) {
            return text[..text.len() - g.len()].to_string();
        }
    }
    text.to_string()
}

/// Retrieve the last input text value captured by an Input's on_input callback.
/// Called from generated Component::on() handlers to read user input.
pub fn last_input_text() -> String {
    INPUT_TEXT.with(|t| t.borrow().clone())
}

/// PLAN-025 T-02（D2 定案 A）：INPUT_TEXT 代写口——native queue 投影器
/// 聚焦框编辑后同线程写入（与 iced 窗口路径写点语义等价：写入 = 焦点框
/// 当前文本；MCP 派发臂 renderer.rs 同款先例），生成 `on()` 经
/// [`last_input_text`] 读面消费。同线程纪律：ClientPump 泵与
/// `component.on` 同线程。
pub fn store_input_text(text: &str) {
    INPUT_TEXT.with(|t| *t.borrow_mut() = text.to_string());
}


// PLAN-014 T-06：按职责家族拆分的子模块（原单文件条目的纯移动）。
// mod.rs 保留：文件头助手与 thread-locals/statics（INPUT_TEXT/
// INSPECT_CAPTURE/TEXTAREA_CONTENTS/P725_INPUT_BINDINGS/SpanHighlighter/
// IntoIcedElement trait）、IcedMessage 与 convert_view_messages、全部测试。
// 消息序/active 判据/last-owner exit/纹理与 external pin 寿命零改。
mod style_widgets;
mod media;
mod projection;
mod host;
mod subscriptions;

// PLAN-014 T-06：公开面恢复——外部经 `ui::iced::renderer::X` 的路径与拆分前逐名一致。
pub use devtools::{
    ComponentIced, DevToolsWrapper, NativePixelsMsg, find_view_by_path_generic, run_app,
    run_app_devtools, run_app_with_task, run_app_with_task_devtools, run_app_with_title,
    run_native_iced_pixels,
};
pub use host::{
    DesktopOptions, RunMode, extract_root_view_fit_size, run_dynamic_desktop,
    run_dynamic_desktop_fullscreen, run_dynamic_desktop_with_options, run_dynamic_iced,
    run_dynamic_iced_multi, run_dynamic_iced_pixels, startup_window_fit, startup_window_hidden,
    startup_window_level, startup_window_position, startup_window_size, window_title,
};
pub use style_widgets::{
    GridCellSpec, INTER_FONT, INTER_FONT_MEDIUM, INTER_FONT_REGULAR, INTER_FONT_SEMIBOLD,
    IntoIcedElement, WALLPAPER_INKWASH, WALLPAPER_RICEPAPER,
};
pub use subscriptions::{
    host_exit_generation, host_exit_window_begin, inject_shell_event, request_host_exit,
};
use devtools::*;
use host::*;
use media::*;
use projection::*;
use shell_state::*;
use style_widgets::*;
use subscriptions::*;

pub(crate) use devtools::{DebugElementInfo, DebugTreeNode, retain_input_values_after_handler, track_input_text};
pub(crate) use style_widgets::{build_column, dynamic_abs_layer_position, layout_hover_flag, wrap_layout_events};
pub(crate) use devtools::{column_layer_partition, patch_input_values};
pub(crate) use shell_state::{
    build_dashboard_snapshot, build_desktop_surface_snapshot, build_launcher_snapshot,
    build_notes_snapshot, build_shell_projection, build_switcher_snapshot, drain_desktop_bus_inbox,
    push_switcher_snapshot,
    push_dashboard_snapshot, push_launcher_snapshot, push_notes_snapshot,
    push_desktop_surface_outproc, push_shell_projection_outproc, summon_launcher,
};
    pub(crate) use media::{load_image_bytes, lucide_svg_doc_with, shadcn_theme};
    pub(crate) use style_widgets::rewrite_viewport_units;
pub(crate) use shell_state::{InspectorSections, ToastReq};
pub(crate) use subscriptions::{DevToolsTab, InspectorSubTab};
mod shell_state;
mod devtools;

/// Sentinel event name for hot-reload tick messages.
const HOT_RELOAD_EVENT: &str = "__hot_reload";

/// Sentinel event name for periodic .Tick messages (stopwatch, timers, etc.)
const TICK_EVENT: &str = "__tick";

/// Send-safe message type for the iced boundary.
///
/// `DynamicMessage` contains `Vec<Value>` where `Value` uses `Rc<RefCell<T>>`
/// internally, making it NOT `Send`. This wrapper carries only the event name
/// and widget name — sufficient for all current AuraViewBuilder events (onclick
/// handlers always have empty args).
///
/// Since `IcedMessage` only has `String` fields, it IS `Send` by default.
#[derive(Clone, Debug)]
pub struct IcedMessage {
    pub widget: String,
    pub event: String,
    /// Carries the text value from input `on_input` callbacks.
    pub input_value: Option<String>,
}

/// Delimiter used to embed a typed onclick payload in the `event` String so it
/// survives iced's `Send` boundary. Pair with `decode_payload` in
/// `dynamic.rs`. `\u{1F}` (ASCII unit separator) cannot appear in a handler
/// name. Format: `{event}\u{1F}{typechar}\u{1F}{value}`.
// PLAN-814 R9-03：单源定义上移 ui::mcp_types（headless 同轨）；此处再
// 导出保旧路径与 const 值单源。
pub(crate) use crate::ui::mcp_types::PAYLOAD_SEP;

/// PLAN-765 T-02（纪要三附注，F-5）：update 通用路径的事件名推导——`.`
/// 前缀剥离 + 路径装饰 `::` 尾段截取。关键修正：`::` 截取**只作用于首个
/// [`PAYLOAD_SEP`] 之前的事件名段**——载荷体（HTTP 回填 JSON 的 body 含
/// 代码文本 `std::process` 等）携带 `::` 时，旧实现按全文 `rfind("::")`
/// 截断会把 handler 名截成载荷尾段，回填静默丢拍（musk K10"长寿实例
/// 空渲染/特定会话内容"观察同形）。
pub(crate) fn derive_event_name(raw: &str) -> String {
    let name = raw.trim_start_matches('.');
    let head_end = name.find(PAYLOAD_SEP).unwrap_or(name.len());
    let head = &name[..head_end];
    match head.rfind("::") {
        // 常见路径：事件名段无 `::` 装饰——整串（含载荷后缀）直接接管，
        // 与旧行为零差异。
        None => name.to_string(),
        // 罕见路径：装饰名（可能含载荷）——重组，载荷后缀原样保留。
        Some(pos) => format!("{}{}", &head[pos + 2..], &name[head_end..]),
    }
}

/// PLAN-044 T5/T6：ghost 占位 state 写入（OnEditorFocus 拦截与 __mcp_click
/// 合成点击共用一条写入路径）。ghost_id = "block-N" 字符串（vue
/// data-block-id 同源；空串 = 无 ghost）；ghost_height 带 +1e-3 强制分数
/// 形态（防 nanbox 整值丢标签，0.001px 误差不可见——043 T6 frac 同口径）。
/// id < 0 = 失焦变体（清空）。
fn write_ghost_state(component: &mut crate::ui::dynamic::DynamicComponent, id: i32, h: f32) {
    let frac = |v: f64| auto_val::Value::Double(v + 0.001);
    if id >= 0 {
        let _ = component.write_state(
            "ghost_id",
            auto_val::Value::Str(format!("block-{id}").into()),
        );
        let _ = component.write_state("ghost_height", frac(h as f64));
    } else {
        let _ = component.write_state("ghost_id", auto_val::Value::Str(String::new().into()));
        let _ = component.write_state("ghost_height", frac(0.0));
    }
}

/// PLAN-057：editor_drag 坐标序列解析（"x0,y0;x1,y1;..." → (x, y) 对）。
/// 纯字符串面（拦截臂与单测共用）；mcp_server 侧已整体校验，此处坏段
/// 防御跳过。
fn parse_drag_points(pts: &str) -> Vec<(f32, f32)> {
    pts.split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|p| {
            let mut it = p.split(',');
            match (
                it.next()?.trim().parse::<f32>().ok(),
                it.next()?.trim().parse::<f32>().ok(),
            ) {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            }
        })
        .collect()
}

/// PLAN-045 T5/T7: 表格列宽 state 写入（OnColResize 拦截与 __mcp_resize_col
/// 合成动作共用一条写入路径）。读-改-写保留其余表键；宽 +1e-3 防 nanbox
/// 整值丢标签（write_ghost_state frac 同口径）。
fn write_table_width_state(
    component: &mut crate::ui::dynamic::DynamicComponent,
    key: &str,
    col: usize,
    w: f32,
) {
    let mut obj = match component.read_state("table_widths") {
        Ok(auto_val::Value::Obj(o)) => o,
        _ => Box::new(auto_val::Obj::new()),
    };
    let mut list: Vec<auto_val::Value> = match obj.get(key) {
        Some(auto_val::Value::Array(a)) => a.iter().cloned().collect(),
        _ => Vec::new(),
    };
    while list.len() <= col {
        list.push(auto_val::Value::Double(0.001));
    }
    list[col] = auto_val::Value::Double(w as f64 + 0.001);
    obj.set(key, auto_val::Value::Array(auto_val::Array::from(list)));
    let _ = component.write_state("table_widths", auto_val::Value::Obj(obj));
}

/// Embed all onclick payload args (type-tagged) into the event string so they
/// can be carried by the `Send` `IcedMessage`. Each arg is encoded as
/// `{tc}{SEP}{val}` appended after the event name. Multi-arg handlers like
/// `.Reveal(cell.x, cell.y)` need both args; previously only the first was
/// encoded (Plan 402 bug 4). See `decode_payload`.
// PLAN-814 R9-03：单源上移 ui::mcp_types；此处再导出保旧路径。
pub(crate) use crate::ui::mcp_types::encode_payload;

impl IcedMessage {
    /// Convert a `DynamicMessage` reference into an `IcedMessage`.
    ///
    /// `Value` is not `Send`, so the typed payload `args` cannot ride in the
    /// `Send` `IcedMessage` directly. Instead `encode_payload` embeds the first
    /// onclick payload arg (type-tagged) into the `event` String; the renderer's
    /// dispatcher (`DynamicComponent::on_with_input`) decodes it back and
    /// forwards it to `call_handler`. Without this, `onclick: .SelectDay(cell.date)`
    /// arrives with no payload and the handler runs with the wrong arg count.
    fn from_dynamic(msg: &DynamicMessage) -> Self {
        match msg {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                args,
            } => IcedMessage {
                widget: widget_name.clone(),
                event: encode_payload(event_name, args),
                input_value: None,
            },
            // PLAN-041 T-02: instance-bound events carry the identity across
            // the Send boundary embedded in the widget name (U+0001 marker,
            // stripped at the on_with_input_for entry — same convention as
            // the `@seq` path decoration).
            DynamicMessage::TypedInst {
                widget_name,
                event_name,
                args,
                instance,
            } => IcedMessage {
                widget: format!("{}\u{1}{}", widget_name, instance),
                event: encode_payload(event_name, args),
                input_value: None,
            },
            DynamicMessage::String(name) => IcedMessage {
                widget: String::new(),
                event: name.clone(),
                input_value: None,
            },
        }
    }
}

// ============================================================================
// PLAN-737：wikilink 激活合成通道派发表
// ============================================================================

/// PLAN-737：storage key → VM 轨 wikilink 激活回调（IcedMessage 产物）。
///
/// 为什么需要：PLAN-732 的激活出口在 widget `publish`（真实事件泵路径）；
/// 而 MCP 合成通道 `__mcp_drag_ade` 直调 `core.handle_input`（057 keyed
/// 寻址、不经 widget）——其抬起腿放行的 `DocOutput.link_activated` 此前
/// 被整条丢弃，真实 merged 生产窗口经该通道的完整点击**永远零激活**
/// （jade PLAN-037 T-02 解锁缺陷根因，本仓探针 + 插桩定谳：点击门
/// same=true 恒成立、ACTIVATED publish 恒不发生）。
///
/// 装配：lower 期 `convert_view_messages` AutodownEditor 臂注册（同 key
/// 覆写）——与 widget `on_link` 共用**同一 LinkCallback 闭包**（消息构造
/// 单源，无第二套事件名/载荷拼装）。消费：`__mcp_drag_ade` 抬起腿经
/// [`ade_link_dispatch_message`]。
///
/// 契约边界（不弱化 SD-01）：真实事件泵路径零变化——widget 仍经自身
/// on_link 发布（C-04 结构性实例绑定不动摇）；本表是 MCP 合成通道的
/// keyed 寻址补充（与 `__mcp_key`/`__mcp_drag_ade` 既有寻址模型同域），
/// 生命周期与编辑壳 core 注册表同键同域（同 key 覆写、无 dispose 面
/// ——core 注册表同样无过期，两表一致）。`__mcp_click`（press-only
/// ghost 语义）不受影响——门不闭合零激活（K-04 冻结负例锚）。
#[cfg(feature = "autodown")]
static ADE_LINK_DISPATCH: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, crate::ui::view::LinkCallback<IcedMessage>>>,
> = std::sync::OnceLock::new();

/// PLAN-737：合成通道派发注册（lowering 装配期；同 key 覆写）。
#[cfg(feature = "autodown")]
pub(crate) fn ade_link_dispatch_register(sk: &str, cb: crate::ui::view::LinkCallback<IcedMessage>) {
    ADE_LINK_DISPATCH
        .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
        .lock()
        .unwrap()
        .insert(sk.to_string(), cb);
}

/// PLAN-737：合成通道激活消费——抬起腿放行的 link_activated → 注册回调
/// 构造 IcedMessage（`IcedMessage::from_dynamic` 同款 Send 边界封装在
/// 装配闭包内）。None = 该 key 无装配（未挂载/泛型轨/缺回调——与既有
/// keyed 通道语义一致，零派发）。
#[cfg(feature = "autodown")]
pub(crate) fn ade_link_dispatch_message(
    sk: &str,
    act: &crate::ui::autodown_editor::LinkActivation,
) -> Option<IcedMessage> {
    // 键归一：MCP 载荷 sk 为裸键（`wiki/Src.ad`），装配注册键为 storage
    // key 归一形——幂等归一后同键域（PLAN-737 探针 MISS 实录后定谳）。
    let sk = crate::ui::autodown_editor::core::normalize_payload_key(sk);
    let cb = {
        let map = ADE_LINK_DISPATCH
            .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
            .lock()
            .unwrap();
        let cb = map.get(&sk).cloned();
        if cb.is_none() {
            crate::ui::autodown_editor::core::ade_trace(&format!(
                "link_dispatch MISS {sk} (table keys={:?})",
                map.keys().cloned().collect::<Vec<_>>()
            ));
        }
        cb
    }?;
    Some(cb.call(crate::ui::view::WikiLinkMetrics {
        target: act.target.clone(),
        anchor: act.anchor.clone(),
    }))
}

// ============================================================================
// Dynamic Todo List helpers
// ============================================================================

/// A single todo item with text and done state.
pub(crate) struct TodoItem {
    text: String,
    done: bool,
}

/// Sync `state.app.todos` (Rust-side) to VM state so the `for todo in .todos` loop can read them.
fn sync_todos_to_vm(todos: &[TodoItem], component: &mut DynamicComponent) {
    let values: Vec<auto_val::Value> = todos
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut obj = auto_val::Obj::new();
            obj.set("id", auto_val::Value::Int(i as i32));
            obj.set("text", auto_val::Value::str(&t.text));
            obj.set("done", auto_val::Value::Bool(t.done));
            auto_val::Value::Obj(Box::new(obj))
        })
        .collect();
    let _ = component.write_state(
        "todos",
        auto_val::Value::Array(auto_val::Array::from(values)),
    );
}

/// Parse an indexed event name like "Toggle:3" into (base, Some(index)).
/// Returns (event_name, None) if no colon-index suffix.
fn parse_indexed_event(event: &str) -> (&str, Option<usize>) {
    if let Some(pos) = event.rfind(':') {
        if let Ok(idx) = event[pos + 1..].parse::<usize>() {
            return (&event[..pos], Some(idx));
        }
    }
    (event, None)
}

/// Build view rows for each todo item.
fn build_todo_rows(items: &[TodoItem], widget_name: &str) -> Vec<AbstractView<DynamicMessage>> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let display = if item.done {
                format!("~~{}~~", item.text)
            } else {
                item.text.clone()
            };
            AbstractView::Row {
                onclick: None,
                on_right_click: None,
                children: vec![
                    AbstractView::Checkbox {
                        is_checked: item.done,
                        label: String::new(),
                        on_toggle: Some(DynamicMessage::Typed {
                            widget_name: widget_name.to_string(),
                            event_name: format!("Toggle:{}", i),
                            args: vec![],
                        }),
                        style: None,
                    },
                    AbstractView::Text {
                        content: display,
                        style: None,
                        selectable: false,
                    },
                    AbstractView::Button {
                        disabled: false,
                        label: "x".into(),
                        onclick: DynamicMessage::Typed {
                            widget_name: widget_name.to_string(),
                            event_name: format!("Delete:{}", i),
                            args: vec![],
                        },
                        style: None,
                        on_right_click: None,
                        content: None,
                    },
                ],
                spacing: 0,
                padding: 0,
                style: Some("w-full items-center gap-3 py-3 border-b".into()),
            }
        })
        .collect()
}

/// Recursively walk the view tree and replace the `__TODO_LIST__` marker text
/// with a Column containing the todo rows.
fn replace_marker(
    view: &mut AbstractView<DynamicMessage>,
    todo_views: Vec<AbstractView<DynamicMessage>>,
) {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            for child in children.iter_mut() {
                if let AbstractView::Text { ref content, .. } = child {
                    if content == "__TODO_LIST__" {
                        if todo_views.is_empty() {
                            *child = AbstractView::Empty;
                        } else {
                            *child = AbstractView::Column {
                                children: todo_views,
                                spacing: 0,
                                padding: 0,
                                style: None,
                                onclick: None,
                                on_right_click: None,
                            };
                        }
                        return;
                    }
                }
                replace_marker(child, todo_views.clone());
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            replace_marker(child, todo_views);
        }
        AbstractView::List { items, .. } => {
            for item in items.iter_mut() {
                replace_marker(item, todo_views.clone());
            }
        }
        // Plan 319: recurse into Grid cells so a `__TODO_LIST__` marker inside
        // a grid is still expanded (else the `_ => {}` wildcard would skip it).
        AbstractView::Grid { cells, .. } => {
            for cell in cells.iter_mut() {
                replace_marker(cell, todo_views.clone());
            }
        }
        _ => {}
    }
}

/// Inject dynamic todo rows into the view tree by replacing the marker.
fn inject_todo_list(
    view: &mut AbstractView<DynamicMessage>,
    todos: &[TodoItem],
    widget_name: &str,
) {
    let todo_views = build_todo_rows(todos, widget_name);
    replace_marker(view, todo_views);
}

/// Recursively convert `View<DynamicMessage>` to `View<IcedMessage>`.
///
/// Each variant that carries a message is mapped through
/// [`IcedMessage::from_dynamic`]. Variants without messages are passed through
/// unchanged. Navigation callback variants (Accordion, Tabs, NavigationRail)
/// and Slider use function-pointer or Arc-callback types that cannot be
/// trivially converted, so they are mapped to `View::Empty` as fallback.
fn convert_view_messages(view: AbstractView<DynamicMessage>) -> AbstractView<IcedMessage> {
    match view {
        AbstractView::Empty => AbstractView::Empty,

        AbstractView::Text {
            content,
            style,
            selectable,
        } => AbstractView::Text {
            content,
            style,
            selectable,
        },

        AbstractView::Button {
            label,
            content,
            onclick,
            style,
            on_right_click,
            disabled,
        } => AbstractView::Button {
            label,
            content: content.map(|c| Box::new(convert_view_messages(*c))),
            onclick: IcedMessage::from_dynamic(&onclick),
            style,
            disabled,
            on_right_click: on_right_click.map(|rc| IcedMessage::from_dynamic(&rc)),
        },

        AbstractView::Row {
            children,
            spacing,
            padding,
            style,
            onclick,
            on_right_click,
        } => AbstractView::Row {
            children: children.into_iter().map(convert_view_messages).collect(),
            spacing,
            padding,
            style,
            // Plan 490 G4：布局件 onclick 穿透转换（DynamicMessage → IcedMessage）。
            onclick: onclick.map(|d| IcedMessage::from_dynamic(&d)),
            // PLAN-002 B：布局件右键同穿透（D-GAP 第四例教训——新字段
            // 每条消息桥臂都必须显式接）。
            on_right_click: on_right_click.map(|d| IcedMessage::from_dynamic(&d)),
        },

        AbstractView::Column {
            children,
            spacing,
            padding,
            style,
            onclick,
            on_right_click,
        } => AbstractView::Column {
            children: children.into_iter().map(convert_view_messages).collect(),
            spacing,
            padding,
            style,
            onclick: onclick.map(|d| IcedMessage::from_dynamic(&d)),
            on_right_click: on_right_click.map(|d| IcedMessage::from_dynamic(&d)),
        },

        AbstractView::Input {
            placeholder,
            value,
            on_change,
            on_submit,
            width,
            password,
            style,
            disabled,
        } => AbstractView::Input {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            on_submit: on_submit.map(|m| IcedMessage::from_dynamic(&m)),
            width,
            password,
            style,
            disabled,
        },

        AbstractView::Textarea {
            placeholder,
            value,
            on_change,
            on_submit,
            height,
            style,
            disabled,
            highlight,
            ghost,
            keydown,
            keymap,
        } => AbstractView::Textarea {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            on_submit: on_submit.map(|m| IcedMessage::from_dynamic(&m)),
            height,
            style,
            disabled,
            highlight,
            ghost,
            keydown: keydown
                .into_iter()
                .map(|(k, m)| (k, IcedMessage::from_dynamic(&m)))
                .collect(),
            keymap,
        },

        AbstractView::CodeEditor {
            key,
            value,
            lang,
            line_numbers,
            wrap,
            vi,
            highlight_current_line,
            readonly,
            tab_width,
            font_size,
            on_change,
            on_cursor,
            on_context_menu,
            search,
            style,
        } => AbstractView::CodeEditor {
            key,
            value,
            lang,
            line_numbers,
            wrap,
            vi,
            highlight_current_line,
            readonly,
            tab_width,
            font_size,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            on_cursor: on_cursor.map(|m| IcedMessage::from_dynamic(&m)),
            on_context_menu: on_context_menu.map(|m| IcedMessage::from_dynamic(&m)),
            search,
            style,
        },

        AbstractView::AutodownEditor {
            key,
            value,
            is_final,
            on_change,
            on_focus,
            on_link,
            placeholder,
            style,
        } => {
            // PLAN-732: wikilink 激活回调跨消息类型包装（LinkCallback
            // newtype；Typed args 双 Str 经 from_dynamic→encode_payload 过
            // Send 边界，decode 后直达 VM handler 形参——499 M2 同路）。
            let on_link = on_link.map(|cb| {
                crate::ui::view::LinkCallback::new(move |m| IcedMessage::from_dynamic(&cb.call(m)))
            });
            // PLAN-737：装配期注册合成通道派发表（同 key 覆写）——
            // __mcp_drag_ade 完整点击门放行的激活经同款回调构造消息
            // （单源闭包；真实事件泵的 widget publish 路径零变化）。
            #[cfg(feature = "autodown")]
            if let Some(cb) = &on_link {
                ade_link_dispatch_register(
                    &crate::ui::autodown_editor::storage_key(&key),
                    cb.clone(),
                );
            }
            #[cfg(not(feature = "autodown"))]
            let _ = &on_link;
            AbstractView::AutodownEditor {
                key,
                value,
                is_final,
                on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
                // Plan 044 T2: 块聚焦读出回调跨消息类型包装（FocusCallback newtype）。
                on_focus: on_focus.map(|cb| {
                    crate::ui::view::FocusCallback::new(move |m| {
                        IcedMessage::from_dynamic(&cb.call(m))
                    })
                }),
                on_link,
                placeholder,
                style,
            }
        }

        AbstractView::Checkbox {
            is_checked,
            label,
            on_toggle,
            style,
        } => AbstractView::Checkbox {
            is_checked,
            label,
            on_toggle: on_toggle.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },

        // PLAN-063 T-04d-2: 锚槽递归转换 child（缺臂会掉进 _ => Empty——
        // 右栏整体消失，本计划实测）。
        AbstractView::AnchorSlot { index, child } => AbstractView::AnchorSlot {
            index,
            child: Box::new(convert_view_messages(*child)),
        },

        // PLAN-781 T-01: 锚点标记递归转换 child（同锚槽纪律）。
        AbstractView::ScrollAnchorMark { pane, key, child } => AbstractView::ScrollAnchorMark {
            pane,
            key,
            child: Box::new(convert_view_messages(*child)),
        },

        AbstractView::Container {
            child,
            padding,
            width,
            height,
            center_x,
            center_y,
            style,
            onclick,
            on_right_click,
        } => AbstractView::Container {
            child: Box::new(convert_view_messages(*child)),
            padding,
            width,
            height,
            center_x,
            center_y,
            style,
            onclick: onclick.map(|d| IcedMessage::from_dynamic(&d)),
            on_right_click: on_right_click.map(|d| IcedMessage::from_dynamic(&d)),
        },

        AbstractView::Scrollable {
            child,
            width,
            height,
            style,
            auto_scroll,
            offset,
            on_scroll,
            axes,
            scrollbar_policy,
            controller,
        } => AbstractView::Scrollable {
            child: Box::new(convert_view_messages(*child)),
            width,
            height,
            style,
            auto_scroll,
            offset,
            // Plan 043 T1: 滚动读出回调跨消息类型包装（ScrollCallback newtype）。
            on_scroll: on_scroll.map(|cb| {
                crate::ui::view::ScrollCallback::new(move |m| {
                    IcedMessage::from_dynamic(&cb.call(m))
                })
            }),
            // PLAN-656: 纯数据透传。
            axes,
            scrollbar_policy,
            controller,
        },

        AbstractView::Radio {
            label,
            is_selected,
            on_select,
            style,
        } => AbstractView::Radio {
            label,
            is_selected,
            on_select: on_select.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },

        AbstractView::List {
            items,
            spacing,
            style,
        } => AbstractView::List {
            items: items.into_iter().map(convert_view_messages).collect(),
            spacing,
            style,
        },

        AbstractView::Table {
            headers,
            rows,
            spacing,
            col_spacing,
            style,
            table_key,
            col_widths,
            on_col_resize,
        } => AbstractView::Table {
            headers: headers.into_iter().map(convert_view_messages).collect(),
            rows: rows
                .into_iter()
                .map(|r| r.into_iter().map(convert_view_messages).collect())
                .collect(),
            spacing,
            col_spacing,
            style,
            table_key,
            col_widths,
            // Plan 045 T1: 列宽拖拽回调跨消息类型包装（ScrollCallback 同款）。
            on_col_resize: on_col_resize.map(|cb| {
                crate::ui::view::ColResizeCallback::new(move |m| {
                    IcedMessage::from_dynamic(&cb.call(m))
                })
            }),
        },

        AbstractView::ProgressBar {
            progress,
            style,
            on_seek,
        } => AbstractView::ProgressBar {
            progress,
            style,
            on_seek: on_seek.map(|h| {
                crate::ui::view::PointerMoveHandler::new(move |x, y| {
                    IcedMessage::from_dynamic(&h.call(x, y))
                })
            }),
        },

        AbstractView::Image { src, style } => AbstractView::Image { src, style },
        AbstractView::ImageSurface {
            src,
            alt,
            width,
            height,
            quality,
            fit,
            zoom,
            offset_x,
            offset_y,
            rotation,
            filter,
            on_error,
            on_loaded,
            on_wheel,
            on_pan,
            on_double_click,
            style,
        } => AbstractView::ImageSurface {
            src,
            alt,
            width,
            height,
            quality,
            fit,
            zoom,
            offset_x,
            offset_y,
            rotation,
            filter,
            on_error: on_error.map(|m| IcedMessage::from_dynamic(&m)),
            on_loaded: on_loaded.map(|m| IcedMessage::from_dynamic(&m)),
            on_wheel: on_wheel.map(|m| IcedMessage::from_dynamic(&m)),
            on_pan: on_pan.map(|m| IcedMessage::from_dynamic(&m)),
            on_double_click: on_double_click.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },

        // PLAN-617 T-19: `video` **必须**显式臂——否则掉进下方 `_ => Empty`
        // 兜底，VM 动态路径下整个播放面静默消失（与上面 Grid / MouseArea / select
        // 同一坑：这已经是第四次踩它了）。
        // PLAN-712 T-04: 上行 handler 随消息类型重映射（DynamicMessage →
        // IcedMessage，ScrollCallback 臂同款 Arc 包装）。
        AbstractView::Video {
            src,
            paused,
            position,
            volume,
            muted,
            rate,
            label,
            on_time_update,
            on_loaded_metadata,
            on_play_state,
            on_ended,
            on_media_error,
            media_key,
            style,
        } => AbstractView::Video {
            src,
            paused,
            position,
            volume,
            muted,
            rate,
            label,
            on_time_update: on_time_update.map(|cb| {
                crate::ui::view::MediaEventHandler::new(move |p| {
                    IcedMessage::from_dynamic(&cb.call(p))
                })
            }),
            on_loaded_metadata: on_loaded_metadata.map(|cb| {
                crate::ui::view::MediaEventHandler::new(move |p| {
                    IcedMessage::from_dynamic(&cb.call(p))
                })
            }),
            on_play_state: on_play_state.map(|cb| {
                crate::ui::view::MediaEventHandler::new(move |p| {
                    IcedMessage::from_dynamic(&cb.call(p))
                })
            }),
            on_ended: on_ended.map(|m| IcedMessage::from_dynamic(&m)),
            on_media_error: on_media_error.map(|cb| {
                crate::ui::view::MediaEventHandler::new(move |p| {
                    IcedMessage::from_dynamic(&cb.call(p))
                })
            }),
            media_key,
            style,
        },

        // Plan 319: recurse into Grid cells. MUST be explicit — the `_ => Empty`
        // catch-all below would silently drop the entire grid (the calendar's
        // dates vanished because Grid hit the wildcard).
        AbstractView::Grid {
            cols,
            gap,
            cells,
            style,
        } => AbstractView::Grid {
            cols,
            gap,
            cells: cells.into_iter().map(convert_view_messages).collect(),
            style,
        },

        // Overlay has no messages of its own, but both layers must survive the
        // DynamicMessage → IcedMessage bridge. Otherwise absolute-positioned
        // gallery examples lose their entire preview before reaching the renderer.
        AbstractView::Overlay {
            base,
            content,
            position,
        } => AbstractView::Overlay {
            base: Box::new(convert_view_messages(*base)),
            content: Box::new(convert_view_messages(*content)),
            position,
        },

        // Plan 422: Popover 携带 on_dismiss 消息 —— MUST be explicit,否则掉进
        // 下方 `_ => Empty` 兜底,弹层在 VM 模式整体消失(menubar 迁移的生命线)。
        AbstractView::Popover {
            anchor,
            content,
            placement,
            open,
            on_dismiss,
        } => {
            use crate::ui::view::PopoverAnchor;
            let anchor = match anchor {
                PopoverAnchor::Widget(w) => {
                    PopoverAnchor::Widget(Box::new(convert_view_messages(*w)))
                }
                PopoverAnchor::Point { x, y } => PopoverAnchor::Point { x, y },
            };
            AbstractView::Popover {
                anchor,
                content: Box::new(convert_view_messages(*content)),
                placement,
                open,
                on_dismiss: on_dismiss.map(|m| IcedMessage::from_dynamic(&m)),
            }
        }

        // PLAN-002 A2 根因修复: window_thumbnail 纯数据变体(无消息回调)此前
        // 掉进 `_ => Empty` 兜底——VM 模式(Vue 壳/桌面 shell 经
        // convert_view_messages 过桥)下 dock hover 缩略整件消失,面板只剩
        // p-1 空壳(8×8 亮点+首开定位漂移,526 KNOWN-DEBT 🟢×2 的真根因;
        // 497 交付时只加了 into_iced 臂,本映射通道漏臂——Grid 319/menubar
        // 422/496 MouseArea 同坑第四例)。
        AbstractView::WindowThumbnail {
            wid,
            fallback_icon,
            style,
        } => AbstractView::WindowThumbnail {
            wid,
            fallback_icon,
            style,
        },

        // PLAN-012 W3：显式臂——A1 fence（缺臂落 Empty 兜底，PLAN-002
        // 同坑第五例防线）。
        AbstractView::WorkspacePreview {
            ws,
            fallback_icon,
            style,
        } => AbstractView::WorkspacePreview {
            ws,
            fallback_icon,
            style,
        },

        // PLAN-760：宿主数据叶透传臂——显式（A1 fence 同坑防线；无消息
        // 回调，输入经渲染臂注册表直发不经 View 消息面）。
        AbstractView::RemoteFrame { url, style } => AbstractView::RemoteFrame { url, style },

        // OS-013 T3: terminal 显式臂——PLAN-009 P1 只接了 at-gen 直渲染
        // (Component → into_iced)与检视占位,VM 动态应用经本转换落
        // `_ => Empty` 兜底,视口整件消失(496 MouseArea 同坑)。select/
        // menu/input 三消息经 from_dynamic 映射;行文本/光标格原样透传
        // (数据已在 convert_terminal 物化)。
        // PLAN-656 T-06: managed content 纯数据透传。
        AbstractView::ManagedScrollContent {
            key,
            logical_w,
            logical_h,
            axes,
        } => AbstractView::ManagedScrollContent {
            key,
            logical_w,
            logical_h,
            axes,
        },

        AbstractView::Terminal {
            key,
            cols,
            rows,
            lines,
            scroll_offset,
            preedit,
            on_select,
            on_menu,
            on_input,
            cursor_row,
            cursor_col,
            history,
            scheme,
            shortcuts,
            style,
        } => AbstractView::Terminal {
            key,
            cols,
            rows,
            lines,
            scroll_offset,
            preedit,
            on_select: on_select.map(|m| IcedMessage::from_dynamic(&m)),
            on_menu: on_menu.map(|m| IcedMessage::from_dynamic(&m)),
            on_input: on_input.map(|m| IcedMessage::from_dynamic(&m)),
            cursor_row,
            cursor_col,
            history,
            scheme,
            shortcuts: shortcuts
                .iter()
                .map(|(k, m)| (k.clone(), IcedMessage::from_dynamic(m)))
                .collect(),
            style,
        },

        // Plan 496 M5: MouseArea 必须显式臂——此前 VM 动态路径走 `_ => Empty`
        // 兜底(484 图表族经 Rust codegen 不经本转换,故未暴露)。桌面图标
        // 双击臂依赖本臂;enter/exit/double_click 三消息递归映射。
        // Plan 498 M0: 增 on_click 映射(chart legend 点击切换显隐)。
        // Plan 499 M2: on_move handler 递归复合(调用产出 DynamicMessage 经
        // from_dynamic 编码为 IcedMessage;逻辑坐标在闭包现场追加)。
        AbstractView::MouseArea {
            content,
            on_enter,
            on_exit,
            on_double_click,
            on_click,
            on_context_menu,
            on_release,
            on_move,
            logical_extent,
            style,
        } => AbstractView::MouseArea {
            content: Box::new(convert_view_messages(*content)),
            on_enter: on_enter.map(|m| IcedMessage::from_dynamic(&m)),
            on_exit: on_exit.map(|m| IcedMessage::from_dynamic(&m)),
            on_double_click: on_double_click.map(|m| IcedMessage::from_dynamic(&m)),
            on_click: on_click.map(|m| IcedMessage::from_dynamic(&m)),
            on_context_menu: on_context_menu.map(|m| IcedMessage::from_dynamic(&m)),
            on_release: on_release.map(|m| IcedMessage::from_dynamic(&m)),
            on_move: on_move.map(|h| {
                crate::ui::view::PointerMoveHandler::new(move |x, y| {
                    IcedMessage::from_dynamic(&h.call(x, y))
                })
            }),
            logical_extent,
            style,
        },

        // Plan 563: 画布 —— 显式臂(缺臂落 Empty 兜底,496 MouseArea
        // 同坑);pen handler 包装同 on_move。
        AbstractView::Canvas {
            scene,
            logical_extent,
            clear,
            on_pen_start,
            on_pen_move,
            on_pen_end,
            on_hit,
            style,
        } => {
            AbstractView::Canvas {
                scene,
                logical_extent,
                clear,
                on_pen_start: on_pen_start.map(|h| {
                    crate::ui::view::PointerMoveHandler::new(move |x, y| {
                        IcedMessage::from_dynamic(&h.call(x, y))
                    })
                }),
                on_pen_move: on_pen_move.map(|h| {
                    crate::ui::view::PointerMoveHandler::new(move |x, y| {
                        IcedMessage::from_dynamic(&h.call(x, y))
                    })
                }),
                on_pen_end: on_pen_end.map(|h| {
                    crate::ui::view::PointerMoveHandler::new(move |x, y| {
                        IcedMessage::from_dynamic(&h.call(x, y))
                    })
                }),
                // PLAN-661 T-05：onhit 换型包装（ElementHitHandler）。
                on_hit: on_hit.map(|h| {
                    crate::ui::view::ElementHitHandler::new(move |id| {
                        IcedMessage::from_dynamic(&h.call(id))
                    })
                }),
                style,
            }
        }

        // PLAN-641：Tabs 显式臂——on_select 是 Arc 回调，与 PointerMoveHandler
        // 同款 from_dynamic 包装即可跨消息类型转换（此前落 `_ => Empty`，VM
        // live 树中 Tabs 整枝被折空）。
        AbstractView::Tabs {
            labels,
            contents,
            selected,
            position,
            on_select,
            style,
            variant,
        } => AbstractView::Tabs {
            labels,
            contents: contents.into_iter().map(convert_view_messages).collect(),
            selected,
            position,
            on_select: on_select.map(|cb| {
                crate::ui::view::TabsSelectCallback::new(move |idx| {
                    IcedMessage::from_dynamic(&cb.call(idx))
                })
            }),
            style,
            variant,
        },

        // PLAN-082 T-01：Rich 段落显式直通——此前漏臂被下方 `_ => Empty`
        // 兜底摊平，markdown 段落/表格单元格/引用正文/列表项正文（autodown
        // render_inlines 的 Rich 承载）在 VM 动态路径整体不可见（080 期
        // 「into_iced Rich 臂零命中」的真因：Rich 在本函数上游臂就被吞，
        // 永远到不了 into_iced）。RichSpanView 仅 content+style，无消息
        // 载荷需要转换，原样透传即可。
        AbstractView::Rich { spans, style } => AbstractView::Rich { spans, style },

        // PLAN-029 T-02：Custom（NativeWidgetRegistry Element 通道，View::
        // Custom）显式直通——此前漏臂落 `_ => Empty`（PLAN-082 Rich 同族
        // 坑），doc-surface 等外部原生组件在 VM 动态路径整体不可见（永远
        // 到不了 into_iced）。events 消息经 IcedMessage::from_dynamic 转换
        // （Button onclick 同式）。
        AbstractView::Custom { name, props, events, style } => AbstractView::Custom {
            name,
            props,
            events: events
                .into_iter()
                .map(|(k, m)| (k, IcedMessage::from_dynamic(&m)))
                .collect(),
            style,
        },

        // Select, Slider, Accordion, Sidebar, NavigationRail use
        // callback types (SelectCallback, fn pointers, Arc<...>) that
        // cannot be trivially converted. Map them to Empty as fallback.
        _ => AbstractView::Empty,
    }
}

// ===========================================================================
// Plan 459：per-App 订阅基建
// ===========================================================================
//
// `Subscription::map`/`filter_map` 组合子的闭包被 const 检查强制零尺寸
// （不可捕获 —— 453 时代 `map_to_app` fn 指针的由来），因此打标不能走
// 组合子。这里用两个 iced_futures 的正式扩展点承载按 App 打标：
//
// 1. `AppTickRecipe`（自定义 `Recipe`）：tokio interval 驱动的周期 tick /
//    全局通道轮询，消息直接产成 `DesktopMessage::App`；Recipe 身份含
//    (AppId, 事件名/轮询函数, 间隔)，多 App 订阅去重互不误伤。
// 2. `iced_futures::subscription::filter_map`（自由函数版，闭包可捕获）：
//    per-App 键盘订阅 —— bindings 随闭包按 App 捕获 + 本窗事件过滤。

#[cfg(test)]
mod tests {
    use super::*;

    // ── PLAN-663 C1b: 视口边界重写 ──

    fn col(style: &str, children: Vec<AbstractView<IcedMessage>>) -> AbstractView<IcedMessage> {
        AbstractView::Column {
            children,
            spacing: 0,
            padding: 0,
            style: Style::parse(style).ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    fn classes_of(v: &AbstractView<IcedMessage>) -> &Vec<StyleClass> {
        match v {
            AbstractView::Column { style: Some(s), .. }
            | AbstractView::Container { style: Some(s), .. }
            | AbstractView::Scrollable { style: Some(s), .. } => &s.classes,
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    fn height_of(v: &AbstractView<IcedMessage>) -> SizeValue {
        classes_of(v)
            .iter()
            .find_map(|c| match c {
                StyleClass::Height(sv) => Some(*sv),
                _ => None,
            })
            .expect("no Height class")
    }

    #[test]
    fn p663_boundary_rewrites_screen_child() {
        let mut root = col(
            "w-full h-[720px] overflow-hidden",
            vec![col("w-full h-screen bg-background", vec![])],
        );
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        assert_eq!(height_of(&children[0]), SizeValue::Pixels(720.0));
    }

    #[test]
    fn p663_no_boundary_keeps_screen() {
        let mut root = col("w-full", vec![col("h-screen", vec![])]);
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        assert_eq!(height_of(&children[0]), SizeValue::Screen);
    }

    #[test]
    fn p663_min_height_screen_rewritten() {
        let mut root = col("h-[720px]", vec![col("min-h-screen", vec![])]);
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        let classes = classes_of(&children[0]);
        let mh = classes
            .iter()
            .find_map(|c| match c {
                StyleClass::MinHeight(m) => Some(*m),
                _ => None,
            })
            .expect("no MinHeight");
        assert_eq!(mh, 720.0);
    }

    #[test]
    fn p663_width_axis_independent() {
        // 只有定宽边界：w-screen 重写、h-screen 不动
        let mut root = col("w-[1024px]", vec![col("w-screen h-screen", vec![])]);
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        let classes = classes_of(&children[0]);
        let w = classes
            .iter()
            .find_map(|c| match c {
                StyleClass::Width(sv) => Some(*sv),
                _ => None,
            })
            .expect("no Width");
        assert_eq!(w, SizeValue::Pixels(1024.0));
        assert_eq!(height_of(&children[0]), SizeValue::Screen);
    }

    #[test]
    fn p663_nested_boundary_overrides() {
        let mut root = col(
            "h-[720px]",
            vec![
                col("h-[400px]", vec![col("h-screen", vec![])]),
                col("h-screen", vec![]),
            ],
        );
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        let AbstractView::Column {
            children: inner, ..
        } = &children[0]
        else {
            panic!()
        };
        assert_eq!(
            height_of(&inner[0]),
            SizeValue::Pixels(400.0),
            "嵌套边界换锚"
        );
        assert_eq!(
            height_of(&children[1]),
            SizeValue::Pixels(720.0),
            "外层边界直达子"
        );
    }

    #[test]
    fn p663_spacing_height_is_not_boundary() {
        // spacing 定高（h-44=176px）不建立边界（PLAN-663 px-only 裁定）
        let mut root = col("h-44", vec![col("h-screen", vec![])]);
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        assert_eq!(height_of(&children[0]), SizeValue::Screen);
    }

    #[test]
    fn p663_margin_y_auto_expands_in_definite_column() {
        let mut root = col(
            "w-full h-[600px] bg-background",
            vec![col("h-10", vec![]), col("my-auto w-40", vec![])],
        );
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        assert!(
            matches!(classes_of(&children[0])[0], StyleClass::Height(_)),
            "非 auto 子不动"
        );
        // my-auto 子被包进 h-full justify-center 包裹列
        let AbstractView::Column {
            children: wrap,
            style: Some(ws),
            ..
        } = &children[1]
        else {
            panic!("my-auto 子应被包裹");
        };
        assert_eq!(wrap.len(), 1, "包裹列恰含原子");
        assert!(
            ws.classes
                .iter()
                .any(|c| matches!(c, StyleClass::JustifyCenter)),
            "包裹列须 justify-center"
        );
        assert!(
            ws.classes
                .iter()
                .any(|c| matches!(c, StyleClass::Height(SizeValue::Full))),
            "包裹列须 h-full"
        );
    }

    #[test]
    fn p663_margin_y_auto_skips_indefinite_and_overflow_columns() {
        // 无定高：不展开
        let mut root = col("w-full", vec![col("my-auto", vec![])]);
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        assert!(matches!(
            classes_of(&children[0])[0],
            StyleClass::MarginYAuto
        ));

        // overflow-y-auto 定高列：不展开（滚动语义归 T-12 机制）
        let mut root2 = col("h-[720px] overflow-y-auto", vec![col("my-auto", vec![])]);
        rewrite_viewport_units(&mut root2);
        let AbstractView::Column { children: c2, .. } = &root2 else {
            panic!()
        };
        assert!(matches!(classes_of(&c2[0])[0], StyleClass::MarginYAuto));
    }

    #[test]
    fn p663_chain_through_scrollable_and_container() {
        let mut root = col(
            "h-[720px]",
            vec![AbstractView::Scrollable {
                child: Box::new(AbstractView::Container {
                    child: Box::new(col("h-screen", vec![])),
                    padding: 0,
                    width: None,
                    height: None,
                    center_x: false,
                    center_y: false,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }),
                width: None,
                height: None,
                style: None,
                auto_scroll: false,
                offset: None,
                on_scroll: None,
                axes: crate::ui::scroll::ScrollAxes::Y,
                scrollbar_policy: crate::ui::scroll::ScrollbarPolicy::Auto,
                controller: None,
            }],
        );
        rewrite_viewport_units(&mut root);
        let AbstractView::Column { children, .. } = &root else {
            panic!()
        };
        let AbstractView::Scrollable { child, .. } = &children[0] else {
            panic!()
        };
        let AbstractView::Container { child, .. } = child.as_ref() else {
            panic!()
        };
        assert_eq!(height_of(child), SizeValue::Pixels(720.0));
    }

    /// PLAN-057：editor_drag 坐标序列解析——正常序列、坏段防御跳过、空串。
    #[test]
    fn plan057_parse_drag_points() {
        assert_eq!(
            parse_drag_points("10,20;30,20;50,20"),
            vec![(10.0, 20.0), (30.0, 20.0), (50.0, 20.0)]
        );
        // 单点序列（仅 MousePressed + Release，无 Dragged 段）。
        assert_eq!(parse_drag_points("5,7"), vec![(5.0, 7.0)]);
        // 坏段（缺 y / 非数字）防御跳过；空串 → 空。
        assert_eq!(
            parse_drag_points("10,20;bad;30,40"),
            vec![(10.0, 20.0), (30.0, 40.0)]
        );
        assert_eq!(parse_drag_points("1,2;oops"), vec![(1.0, 2.0)]);
        assert!(parse_drag_points("").is_empty());
    }

    /// Plan 547 Task 26: ImageSurface uses the dedicated renderer path,
    /// accepts the media URI contract, and builds a clipped element without
    /// invoking the legacy synchronous Image loader.
    #[test]
    fn image_surface_renderer() {
        let view = AbstractView::<()>::ImageSurface {
            src: "/api/__auto/media/opaque/1".into(),
            alt: "preview".into(),
            width: 640,
            height: 480,
            quality: 90,
            fit: "contain".into(),
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0,
            filter: "high".into(),
            on_error: None,
            on_loaded: None,
            on_wheel: None,
            on_pan: None,
            on_double_click: None,
            style: None,
        };
        let _element = view.into_iced();
        assert_eq!(
            crate::ui::iced::image_surface::ImageSurfaceFit::parse("fit-width"),
            crate::ui::iced::image_surface::ImageSurfaceFit::Width
        );
    }

    #[test]
    fn native_media_uri_resolves_registry_before_http_fallback() {
        let registry = auto_media_core::image_pipeline::global_media_registry();
        let ticket = registry.queue(
            auto_media_core::image_pipeline::MediaAssetKey {
                source_fingerprint: "renderer-fixture".into(),
                orientation: Default::default(),
                rendition: auto_media_core::image_pipeline::RenditionSpec::original(),
                revision: 1,
            },
            auto_media_core::image_pipeline::MediaMetadata::default(),
        );
        let uri = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
        assert!(
            load_image_bytes(&uri).is_none(),
            "pending ticket should not fall back to a file"
        );
        registry
            .transition(
                ticket.id,
                auto_media_core::image_pipeline::MediaAssetState::Reading,
            )
            .unwrap();
        registry
            .transition(
                ticket.id,
                auto_media_core::image_pipeline::MediaAssetState::Decoding,
            )
            .unwrap();
        registry
            .transition(
                ticket.id,
                auto_media_core::image_pipeline::MediaAssetState::Transforming,
            )
            .unwrap();
        registry
            .publish_ready(
                ticket.id,
                ticket.revision,
                std::sync::Arc::<[u8]>::from([0, 255, 2]),
            )
            .unwrap();
        assert_eq!(load_image_bytes(&uri), Some(vec![0, 255, 2]));
        assert!(
            load_image_bytes("relative/path.png").is_none(),
            "ordinary file fallback remains available"
        );
    }

    #[test]
    fn plan606_load_image_bytes_data_urls() {
        // SVG data URL (utf8 / plain)
        let svg_url = "data:image/svg+xml;utf8,<svg viewBox=\"0 0 10 10\"><circle cx=\"5\" cy=\"5\" r=\"4\"/></svg>";
        let svg_bytes = load_image_bytes(svg_url).expect("SVG data URL should decode");
        let svg_text = String::from_utf8(svg_bytes).expect("Valid utf8");
        assert!(svg_text.contains("<svg") && svg_text.contains("circle"));

        // Base64 data URL (1x1 PNG)
        let png_b64 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
        let png_bytes = load_image_bytes(png_b64).expect("PNG base64 data URL should decode");
        assert_eq!(&png_bytes[0..4], &[0x89, b'P', b'N', b'G']);

        // Malformed data URL without comma
        assert!(load_image_bytes("data:no-comma").is_none());
    }

    /// PLAN-530 步骤5 回归（B 内存崩塌主源）：lucide_svg 同名 icon 必须命中
    /// 去重缓存（同一 &'static str），不得每次调用 Box::leak 一份新 16×16
    /// 文档——每帧每图标 ~350B 的无界泄漏随重建频率线性增长（homepage 实测
    /// ~67MB/min 的主分量）。指针同一性 = 缓存命中的可观测契约。
    #[test]
    fn p530_lucide_svg_same_name_returns_same_static() {
        let a = lucide_svg("bell").expect("bell 已登记");
        let b = lucide_svg("bell").expect("bell 已登记");
        assert!(
            std::ptr::eq(a, b),
            "同名 icon 返回了不同的 'static 串——缓存未命中，正在泄漏"
        );
        let c = lucide_svg("copy").expect("copy 已登记");
        assert!(!std::ptr::eq(a, c), "不同 icon 不应串缓存");
    }

    /// PLAN-530 步骤5 回归：placeholder 'static 化必须按文本去重——text_editor
    /// placeholder 契约要求 &'static str，旧实现每次重建 Box::leak 一份新串。
    #[test]
    fn p530_leaked_placeholder_dedupes_same_text() {
        let a = leaked_placeholder("Search widgets...");
        let b = leaked_placeholder("Search widgets...");
        assert!(
            std::ptr::eq(a, b),
            "同文本 placeholder 返回了不同的 'static 串——正在泄漏"
        );
        assert_eq!(a, "Search widgets...");
    }

    /// PLAN-530 步骤3 回归（W10/W11 双份叠印根因）：Column 叠层分区只认
    /// absolute 脱流子；z-index-only 子必须留在流内。472 时代的"前缀
    /// overlay"路径把 0..=elev_idx 整段重渲染进叠层 → 非提升子被 base +
    /// overlay 双渲染（widgets-gallery 首页 <768 整页 ×2 叠印实测 73/75
    /// widget id 布局两次），且 base 压 compact 把正文顶进 header 之下。
    #[test]
    fn p530_column_layer_partition_zindex_stays_in_flow() {
        use crate::ui::style::Style;
        let styled = |classes: &'static str| AbstractView::<IcedMessage>::Container {
            child: Box::new(AbstractView::Empty),
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: Style::parse(classes).ok(),
            onclick: None,
            on_right_click: None,
        };
        // widgets-gallery app.at 骨架同构：header(sticky z-40) / 正文 /
        // 移动底栏(fixed z-40) / 帮助浮层(absolute z-50)。
        let children = vec![
            styled("sticky z-40"),
            styled(""),
            styled("fixed z-40"),
            styled("absolute z-50"),
        ];
        let (flow, floating) = super::column_layer_partition(&children);
        assert_eq!(floating, vec![3], "仅 absolute 子脱流入叠层");
        assert_eq!(
            flow,
            vec![0, 1, 2],
            "z-index-only 子保持流内（每子恰渲染一次）"
        );
    }

    /// PLAN-022 T-03 空层判定加固:声明背景色的空容器(分隔条 = bg div
    /// + mouse-area 子件;codegen 缺臂时代内层落空 col)不再判空入栈
    /// 跳过——bg 即可见 chrome。纯透明空层(backdrop 族)语义不变。
    #[test]
    fn plan022_empty_layer_with_background_stays_in_stack() {
        use crate::ui::style::Style;
        let empty_col = |classes: &'static str| AbstractView::<IcedMessage>::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: Style::parse(classes).ok(),
            onclick: None,
            on_right_click: None,
        };
        // 分隔条内层形态:bg 薄条 + Fill 空子(空 col)。
        let divider_inner = AbstractView::<IcedMessage>::Column {
            children: vec![empty_col("w-full h-full")],
            spacing: 0,
            padding: 0,
            style: Style::parse("absolute z-20 w-[8px] h-[410px] bg-[#8899aa]").ok(),
            onclick: None,
            on_right_click: None,
        };
        assert!(
            !super::is_empty_stack_layer(&divider_inner),
            "带 bg 的空容器不得判空(分隔条 chrome 消失回归)"
        );
        // 无 bg 的纯空 col(旧 backdrop 形态):保持判空。
        assert!(super::is_empty_stack_layer(&empty_col("w-full h-full")));
    }

    /// PLAN-526 T6：boot 对齐匹配核——双向后缀命中（相对源路径 vs 绝对
    /// 注册表 entry）+ 无关路径不命中。
    #[test]
    fn boot_entry_matches_suffix_both_directions() {
        assert!(super::boot_entry_matches(
            "examples/ui/011-calculator/src/front/app.at",
            "C:/autostack/auto-ui/examples/ui/011-calculator/src/front/app.at",
        ));
        assert!(super::boot_entry_matches(
            "C:/repo/examples/ui/459-dual-app/app.at",
            "459-dual-app/app.at",
        ));
        assert!(!super::boot_entry_matches(
            "examples/ui/011-calculator/src/front/app.at",
            "examples/ui/013-todo/src/front/app.at",
        ));
    }

    /// Plan 527 T6:变体管道按钮状态面 —— focus/active/disabled 可声明、
    /// merged_with_variant 可应用(变体后应用胜出)。focus 在按钮回调面为
    /// iced 0.14 上限(Status 无 Focused 档),本测锁定「类可合并构建」的
    /// applied 面;hover 沿既有先例零回归。
    #[test]
    fn plan527_t6_variant_button_status_applied() {
        use crate::ui::style::{Style, Variant};
        let s = Style::parse(
            "bg-surface hover:bg-primary/20 focus:bg-blue-500 active:bg-muted disabled:opacity-50",
        )
        .unwrap();
        let base = build_button_style(&crate::ui::style_iced_adapter::IcedStyle::from_style(&s));
        // hover:变体色覆盖 base(既有先例)
        let hover = build_button_style(
            &crate::ui::style_iced_adapter::IcedStyle::merged_with_variant(&s, Variant::Hover),
        );
        assert!(hover.background.is_some());
        // focus/active/disabled:同构合并,各自底色生效
        let focus = build_button_style(
            &crate::ui::style_iced_adapter::IcedStyle::merged_with_variant(&s, Variant::Focus),
        );
        let active = build_button_style(
            &crate::ui::style_iced_adapter::IcedStyle::merged_with_variant(&s, Variant::Active),
        );
        let disabled = build_button_style(
            &crate::ui::style_iced_adapter::IcedStyle::merged_with_variant(&s, Variant::Disabled),
        );
        let bg_of = |st: &iced::widget::button::Style| {
            st.background.map(|b| match b {
                iced::Background::Color(c) => (c.r, c.g, c.b),
                _ => (-1.0, -1.0, -1.0),
            })
        };
        let (fb, ab, db) = (bg_of(&focus), bg_of(&active), bg_of(&disabled));
        assert!(
            fb.is_some() && fb != bg_of(&base),
            "focus:bg-blue-500 生效(合并面)"
        );
        assert!(ab.is_some() && ab != bg_of(&base), "active:bg-muted 生效");
        // disabled:opacity-50 不改底色但改透明度——锁定 opacity 生效
        if let Some(iced::Background::Color(c)) = disabled.background {
            assert!(c.a < 1.0, "disabled:opacity-50 半透明生效,got a={}", c.a);
        } else {
            panic!("disabled 状态应有底色");
        }
    }

    /// Plan 043 T1: Scrollable offset 绑定写入的消费测——build_scrollable
    /// 首次见 (id, offset) 入队，排空取走；同值重建不重复入队（去抖）。
    /// build_scrollable 本体无法在 headless 下断言滚动效果，这里锁
    /// pending 队列的登记/排空/去抖语义（发射面 = update_inner 尾部
    /// 排空 → operation::scroll_to）。
    #[test]
    fn p043_scrollable_offset_pending_registry() {
        use super::{build_scrollable, drain_pending_scroll_offsets};

        let id = format!("p043_test_scroll_{}", std::process::id());
        // 清残留（同进程其它用例可能用过同 id）
        {
            let mut reg = super::pending_scroll_offsets().lock().unwrap();
            reg.remove(&id);
        }
        let child: iced::Element<'static, IcedMessage> = iced::widget::text("body").into();
        let _el = build_scrollable(
            child,
            None,
            None,
            None,
            Some(id.clone()),
            Some((0.0, 100.0)),
            None,
            crate::ui::scroll::ScrollAxes::Y,
            crate::ui::scroll::ScrollbarPolicy::Auto,
            None,
        );
        let drained = drain_pending_scroll_offsets();
        assert!(
            drained
                .iter()
                .any(|(i, (x, y))| i == &id && *x == 0.0 && *y == 100.0),
            "first build must enqueue: got {:?}",
            drained
        );
        // 同值重建 → 不再入队
        let child: iced::Element<'static, IcedMessage> = iced::widget::text("body").into();
        let _el = build_scrollable(
            child,
            None,
            None,
            None,
            Some(id.clone()),
            Some((0.0, 100.0)),
            None,
            crate::ui::scroll::ScrollAxes::Y,
            crate::ui::scroll::ScrollbarPolicy::Auto,
            None,
        );
        let drained = drain_pending_scroll_offsets();
        assert!(
            !drained.iter().any(|(i, _)| i == &id),
            "same-offset rebuild must not re-enqueue: got {:?}",
            drained
        );
        // 值变化（>0.5px）→ 再入队
        let child: iced::Element<'static, IcedMessage> = iced::widget::text("body").into();
        let _el = build_scrollable(
            child,
            None,
            None,
            None,
            Some(id.clone()),
            Some((0.0, 250.0)),
            None,
            crate::ui::scroll::ScrollAxes::Y,
            crate::ui::scroll::ScrollbarPolicy::Auto,
            None,
        );
        let drained = drain_pending_scroll_offsets();
        assert!(
            drained.iter().any(|(i, (_, y))| i == &id && *y == 250.0),
            "changed offset must re-enqueue"
        );
        // 微抖（≤0.5px）→ 不入队
        let child: iced::Element<'static, IcedMessage> = iced::widget::text("body").into();
        let _el = build_scrollable(
            child,
            None,
            None,
            None,
            Some(id.clone()),
            Some((0.0, 250.3)),
            None,
            crate::ui::scroll::ScrollAxes::Y,
            crate::ui::scroll::ScrollbarPolicy::Auto,
            None,
        );
        let drained = drain_pending_scroll_offsets();
        assert!(
            !drained.iter().any(|(i, _)| i == &id),
            "sub-epsilon jitter must not re-enqueue"
        );
    }

    /// PLAN-057 T8：musk 图标桥全量锁定——icons.web.at 的 use.web component
    /// 名单（lucide-vue-next 44 项）经 pascal_to_kebab_icon 规则（大写断词、
    /// 数字前补划线）必须全部命中 lucide_svg 字形表。054 R2 已扩容到位，
    /// 本测锁死"新增图标必同步字形表"的回归面（未命中→VM 实机 [Image]
    /// 空图标/ext-stub 警告）。
    #[test]
    fn p057_musk_icon_bridge_full_coverage() {
        let musk_icons: &[(&str, &str)] = &[
            ("ArrowDown", "arrow-down"),
            ("ArrowUp", "arrow-up"),
            ("BookOpen", "book-open"),
            ("Check", "check"),
            ("ChevronDown", "chevron-down"),
            ("ChevronRight", "chevron-right"),
            ("ChevronUp", "chevron-up"),
            ("Clock", "clock"),
            ("Copy", "copy"),
            ("CopyCheck", "copy-check"),
            ("Download", "download"),
            ("ExternalLink", "external-link"),
            ("Eye", "eye"),
            ("File", "file"),
            ("FileIcon", "file-icon"),
            ("FileText", "file-text"),
            ("Folder", "folder"),
            ("FolderInput", "folder-input"),
            ("FolderOpen", "folder-open"),
            ("FolderPlus", "folder-plus"),
            ("HelpCircle", "help-circle"),
            ("Inbox", "inbox"),
            ("Info", "info"),
            ("ListTodo", "list-todo"),
            ("Loader2", "loader-2"),
            ("MessageSquare", "message-square"),
            ("Monitor", "monitor"),
            ("Moon", "moon"),
            ("Orbit", "orbit"),
            ("PanelLeft", "panel-left"),
            ("Pencil", "pencil"),
            ("Plus", "plus"),
            ("Scroll", "scroll"),
            ("Search", "search"),
            ("Send", "send"),
            ("Settings", "settings"),
            ("Square", "square"),
            ("Sun", "sun"),
            ("Terminal", "terminal"),
            ("Trash2", "trash-2"),
            ("Unlink", "unlink"),
            ("UploadCloud", "upload-cloud"),
            ("Wrench", "wrench"),
            ("X", "x"),
        ];
        let mut missing = Vec::new();
        for (pascal, kebab) in musk_icons {
            if lucide_svg(kebab).is_none() {
                missing.push(format!("{pascal}→{kebab}"));
            }
        }
        assert!(
            missing.is_empty(),
            "musk 图标桥字形缺失（lucide_svg 表补齐后重跑）：{:?}",
            missing
        );
    }

    /// Plan 504：pac `window: "fit"` env 解析（大小写不敏感；"WxH" / 缺席 → false）。
    #[test]
    fn startup_window_fit_parses_env() {
        std::env::set_var("AUTO_VM_WINDOW", "fit");
        assert!(startup_window_fit());
        std::env::set_var("AUTO_VM_WINDOW", "FIT");
        assert!(startup_window_fit());
        std::env::set_var("AUTO_VM_WINDOW", "800x600");
        assert!(!startup_window_fit());
        std::env::remove_var("AUTO_VM_WINDOW");
        assert!(!startup_window_fit());
    }

    /// Plan 504：`__fit_measured` 回执 desktop 臂——虚拟窗矩形收缩到
    /// 内容 + chrome（TITLEBAR_H + 2·BORDER），fit_pending 清除、
    /// window_size 写内容尺寸、在飞标记复位。
    #[test]
    fn apply_fit_measured_shrinks_desktop_vwin() {
        let mut ds = t3_session_with_shell();
        let wid = t3_add_win(&mut ds, "FitProbe");
        let host = ds.host.as_ref().unwrap();
        let app = host.wm.wins[&wid].app;
        host.wm.wins[&wid].fit_pending.set(true);
        let hw = host.window;
        let payload = format!("{{\"aura_fit_root_{}\": [0.0, 0.0, 384.0, 392.0]}}", app.0);
        let _ = apply_fit_measured(&mut ds, hw, Some(&payload));
        let host = ds.host.as_ref().unwrap();
        let v = &host.wm.wins[&wid];
        use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
        assert!(!v.fit_pending.get(), "测量命中后 fit_pending 清除");
        assert_eq!(v.rect.borrow().width, 384.0 + 2.0 * BORDER);
        assert_eq!(v.rect.borrow().height, 392.0 + TITLEBAR_H + 2.0 * BORDER);
        assert_eq!(*v.window_size.borrow(), iced::Size::new(384.0, 392.0));
        assert!(
            !ds.desktop.fit_measure_in_flight.get(),
            "回执到达即在飞复位"
        );
    }

    /// Plan 504：锚点缺席（界面未建好）——保留 fit_pending、计数 +1、
    /// 重新置在飞自重试。
    #[test]
    fn apply_fit_measured_missing_anchor_retries() {
        let mut ds = t3_session_with_shell();
        let wid = t3_add_win(&mut ds, "FitProbe");
        ds.host.as_ref().unwrap().wm.wins[&wid]
            .fit_pending
            .set(true);
        let hw = ds.host.as_ref().unwrap().window;
        let _ = apply_fit_measured(&mut ds, hw, Some("{}"));
        assert!(ds.has_fit_pending(), "锚点缺席保留 fit_pending");
        assert_eq!(ds.desktop.fit_measure_retries.get(), 1);
        assert!(ds.desktop.fit_measure_in_flight.get(), "自重试重新置在飞");
    }

    /// Plan 512 T1：`decide_fit_resize` 纯逻辑——锁定/滞回/双向跟随/clamp。
    #[test]
    fn decide_fit_resize_threshold_lock_clamp() {
        // 用户锁定：任意差值都跳过。
        assert!(decide_fit_resize((384.0, 392.0), (384.0, 600.0), true).is_none());
        // 滞回：双轴差均 ≤8px 跳过（含仅单轴小差）。
        assert!(decide_fit_resize((384.0, 392.0), (392.0, 400.0), false).is_none());
        assert!(decide_fit_resize((384.0, 392.0), (391.9, 392.0), false).is_none());
        // 增大跟随：单轴超阈即触发。
        assert_eq!(
            decide_fit_resize((384.0, 392.0), (384.0, 600.0), false),
            Some(iced::Size::new(384.0, 600.0))
        );
        // 缩小跟随：Scientific → Basic 回缩。
        assert_eq!(
            decide_fit_resize((384.0, 600.0), (384.0, 392.0), false),
            Some(iced::Size::new(384.0, 392.0))
        );
        // clamp：测量值小于 200 下限。
        assert_eq!(
            decide_fit_resize((384.0, 392.0), (50.0, 100.0), false),
            Some(iced::Size::new(200.0, 200.0))
        );
    }

    /// Plan 512 T2：动态重测臂（desktop）——首测已完成的 fit 窗内容增高
    /// 超阈 → 虚拟窗矩形跟随（chrome 外沿同 504 公式），fit_dirty 消费清除。
    #[test]
    fn apply_fit_measured_remeasure_follows_content_growth() {
        let mut ds = t3_session_with_shell();
        let wid = t3_add_win(&mut ds, "FitRe");
        let app = ds.host.as_ref().unwrap().wm.wins[&wid].app;
        {
            // 模拟首测已完成态：fit_pending=false、内容尺寸已回填。
            let v = ds.host.as_mut().unwrap().wm.wins.get_mut(&wid).unwrap();
            v.fit_enabled.set(true);
            *v.window_size.borrow_mut() = iced::Size::new(384.0, 392.0);
            v.fit_dirty.set(true);
        }
        let hw = ds.host.as_ref().unwrap().window;
        let payload = format!("{{\"aura_fit_root_{}\": [0.0, 0.0, 384.0, 600.0]}}", app.0);
        let _ = apply_fit_measured(&mut ds, hw, Some(&payload));
        let host = ds.host.as_ref().unwrap();
        let v = &host.wm.wins[&wid];
        use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
        assert_eq!(
            v.rect.borrow().height,
            600.0 + TITLEBAR_H + 2.0 * BORDER,
            "矩形跟随内容增高"
        );
        assert_eq!(*v.window_size.borrow(), iced::Size::new(384.0, 600.0));
        assert!(!v.fit_dirty.get(), "回执消费 fit_dirty");
    }

    /// Plan 512 T2：用户手动 resize 锁定的 fit 窗——重测跳过（矩形不变），
    /// fit_dirty 仍被消费（不积压）。
    #[test]
    fn apply_fit_measured_remeasure_skips_when_user_locked() {
        let mut ds = t3_session_with_shell();
        let wid = t3_add_win(&mut ds, "FitLock");
        let app = ds.host.as_ref().unwrap().wm.wins[&wid].app;
        let (orig_w, orig_h);
        {
            let v = ds.host.as_mut().unwrap().wm.wins.get_mut(&wid).unwrap();
            v.fit_enabled.set(true);
            v.fit_user_locked.set(true);
            *v.window_size.borrow_mut() = iced::Size::new(384.0, 392.0);
            v.fit_dirty.set(true);
            let r = v.rect.borrow();
            orig_w = r.width;
            orig_h = r.height;
        }
        let hw = ds.host.as_ref().unwrap().window;
        let payload = format!("{{\"aura_fit_root_{}\": [0.0, 0.0, 384.0, 600.0]}}", app.0);
        let _ = apply_fit_measured(&mut ds, hw, Some(&payload));
        let host = ds.host.as_ref().unwrap();
        let v = &host.wm.wins[&wid];
        let r = v.rect.borrow();
        assert_eq!((r.width, r.height), (orig_w, orig_h), "锁定窗矩形不变");
        assert!(!v.fit_dirty.get(), "锁定窗 fit_dirty 仍被消费");
    }

    /// Plan 512 T2：动态重测臂（独立模式）——OS 窗 window_size 跟随并记录
    /// fit_last_applied（OS 回波豁免凭据）。
    #[test]
    fn apply_fit_measured_remeasure_standalone_records_applied() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        let comp = crate::build_dynamic_component(T3_WIN_AT, None).unwrap();
        let app = ds.allocate_app(comp);
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(384.0, 392.0));
        {
            let entry = ds.windows.get_mut(&win).unwrap();
            entry.fit_enabled.set(true);
            entry.fit_dirty.set(true);
        }
        let payload = format!("{{\"aura_fit_root_{}\": [0.0, 0.0, 384.0, 600.0]}}", app.0);
        let _ = apply_fit_measured(&mut ds, win, Some(&payload));
        let entry = &ds.windows[&win];
        assert_eq!(*entry.window_size.borrow(), iced::Size::new(384.0, 600.0));
        assert_eq!(
            *entry.fit_last_applied.borrow(),
            Some(iced::Size::new(384.0, 600.0)),
            "程序化 resize 记录应用值"
        );
        assert!(!entry.fit_dirty.get());
    }

    /// PLAN-493 T4: mention 段 kind 流通——parse_span_kind 认 "mention"，
    /// build_span_lines 对 mention 段按无缝覆盖契约切出行内范围。
    #[test]
    fn plan493_mention_kind_parses_and_covers() {
        assert_eq!(parse_span_kind("mention"), SpanKind::Mention);
        assert_eq!(parse_span_kind("Command"), SpanKind::Command);
        let value = "@assistant hi";
        let spans = vec![
            ("@assistant".to_string(), "mention".to_string()),
            (" hi".to_string(), "text".to_string()),
        ];
        let settings = build_span_lines(&spans, value, "");
        assert!(!settings.lines.is_empty(), "无缝覆盖段必须切出 SpanLines");
        let first = &settings.lines[0];
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].1, SpanKind::Mention);
        assert_eq!(&value[first[0].0.clone()], "@assistant");
    }

    /// PLAN-051 P2: textarea 高度解析——prop 优先 / h-full → Fill /
    /// min-h → Fixed / 均无 → 历史缺省 30。
    #[test]
    fn plan051_textarea_height_consumes_style() {
        use crate::ui::style::Style;
        assert_eq!(
            textarea_editor_height(Some(20), None),
            iced::Length::Fixed(20.0)
        );
        assert_eq!(
            textarea_editor_height(None, None),
            iced::Length::Fixed(30.0)
        );
        let hfull = Style::parse("h-full").unwrap();
        assert_eq!(
            textarea_editor_height(None, Some(&hfull)),
            iced::Length::Fill,
            "h-full 必须消费为 Fill(composer 命中区修复)"
        );
        let minh = Style::parse("min-h-20").unwrap();
        assert_eq!(
            textarea_editor_height(None, Some(&minh)),
            iced::Length::Fixed(80.0)
        );
    }

    /// Plan 482 T13: nav 组件族采用清单的 lucide 图标在 lucide_svg 内嵌集内
    /// （musk rail 四图标 + 折叠 chevron/搜索）——缺名静默降级空占位，
    /// 此处以测试锁住。
    #[test]
    fn nav_family_lucide_icons_present() {
        for name in [
            "message-square",
            "list-todo",
            "scroll",
            "book-open",
            "chevron-down",
            "chevron-right",
            "search",
        ] {
            assert!(
                lucide_svg(name).is_some(),
                "nav 组件族图标 {name} 缺失于 lucide_svg 内嵌集"
            );
        }
    }

    use super::*;

    // ---- PLAN-050 C1: content-subtree 按钮的内容对齐决策（类串→解析→映射） ----
    fn p050_style(classes: Vec<StyleClass>) -> IcedStyle {
        IcedStyle::from_style(&Style {
            classes,
            hover_classes: Vec::new(),
            variant_classes: Vec::new(),
        })
    }

    #[test]
    fn plan050_content_align_justify_start_is_left() {
        let is = p050_style(vec![StyleClass::Flex, StyleClass::JustifyStart]);
        let (h, v) = plan050_content_align(Some(&is));
        assert_eq!(h, Some(iced::alignment::Horizontal::Left));
        assert_eq!(v, None);
    }

    #[test]
    fn plan050_content_align_text_left_fallback() {
        let is = p050_style(vec![StyleClass::TextLeft]);
        let (h, _) = plan050_content_align(Some(&is));
        assert_eq!(h, Some(iced::alignment::Horizontal::Left));
    }

    #[test]
    fn plan050_content_align_items_start_is_top() {
        let is = p050_style(vec![StyleClass::ItemsStart]);
        let (_, v) = plan050_content_align(Some(&is));
        assert_eq!(v, Some(iced::alignment::Vertical::Top));
    }

    #[test]
    fn plan050_side_border_flags_parse() {
        let is = p050_style(vec![StyleClass::BorderBottom, StyleClass::BorderRight]);
        assert!(is.border_bottom && is.border_right);
        assert!(!is.border_top && !is.border_left);
        let is = p050_style(vec![StyleClass::BorderTop, StyleClass::BorderLeft]);
        assert!(is.border_top && is.border_left);
        assert!(!is.border_bottom && !is.border_right);
    }

    #[test]
    fn plan050_content_align_none_keeps_centered_default() {
        let is = p050_style(vec![]);
        let (h, v) = plan050_content_align(Some(&is));
        assert_eq!(h, None);
        assert_eq!(v, None);
        let (h, v) = plan050_content_align(None);
        assert_eq!(h, None);
        assert_eq!(v, None);
    }

    #[test]
    fn plan414_height_branch_yields_to_explicit_align() {
        // nav-item(h-9 w-full text-left items-center):水平轴让位 text-left,
        // 高度臂不再无条件 center_x——否则内层 Fill 容器恒居中,外层 050
        // 包装无法抵消,VM 轨 rail 内容居中而 web 左对齐。
        use iced::alignment::{Horizontal, Vertical};
        let (ax, ay) = plan414_content_alignment(Some(Horizontal::Left), Some(Vertical::Center));
        assert_eq!(ax, Horizontal::Left);
        assert_eq!(ay, Vertical::Center);
        // 垂直轴同理:items-start 压过 409 的默认纵向居中。
        let (_, ay) = plan414_content_alignment(None, Some(Vertical::Top));
        assert_eq!(ay, Vertical::Top);
        // 无显式对齐类:维持 Plan 409/414 双向居中(工具栏定宽图标钮)。
        let (ax, ay) = plan414_content_alignment(None, None);
        assert_eq!(ax, Horizontal::Center);
        assert_eq!(ay, Vertical::Center);
    }

    // ---- PLAN-050 P2 #3: mt-auto → Column 弹性占位判定 ----
    #[test]
    fn plan050_mt_auto_child_gets_fill_spacer() {
        // 全链：mt-auto 经 master 通用 mt-<size> 解析为 MarginTop(SizeValue::Auto)，
        // 命中占位判定；定值 mt-4 与空类不命中。
        let auto = crate::ui::style::StyleClass::parse_single("mt-auto").unwrap();
        assert!(plan050_mt_auto_spacer(&[auto]));
        let fixed = crate::ui::style::StyleClass::parse_single("mt-4").unwrap();
        assert!(!plan050_mt_auto_spacer(&[fixed]));
        assert!(!plan050_mt_auto_spacer(&[]));
    }
    // ---- Plan 472 T3：投影协议 v1（__wm_* formalize + __wm_workspaces + 指纹门控）----

    const T3_SHELL_AT: &str = r#"widget ShellProbe {
    model {
        var __desktop_cmd str = ""
        var __wm_wins = []
        var __wm_meta str = ""
        var __wm_fp str = ""
        var __wm_workspaces = []
        var __wm_mru = []
        var __wm_notes = []
        var __wm_notes_unread str = ""
        var __wm_running str = ""
        var __wm_focused_app str = ""
        var __wm_notes_visible str = ""
        var __wm_showdesk str = ""
        var __dock_pinned_csv str = ""
    }
    view { col { text "shell" } }
}
"#;

    const T3_WIN_AT: &str =
        "widget T3Win {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n";

    /// Plan 551：os-config 合成桩——最小可编译 .at（真实前端居兄弟仓，
    /// CI 不依赖）。仅承载 ⚙️ launch-or-focus 窗语义；配置读写单源走
    /// daemon，页面契约由 os-config 仓自测。
    const T551_OSCONFIG_STUB_AT: &str = "widget App {
    model { var ready int = 0 }
    view { text \"os-config stub\" }
}
";

    fn t3_session_with_shell() -> crate::ui::session::DesktopSession {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::build_dynamic_component(T3_SHELL_AT, None).unwrap();
        ds.desktop.shell_app = Some(ds.allocate_app(comp));
        ds
    }

    /// Plan 540 T7 测试座：t3 会话 + 真注册表 resolver（扫仓库 examples/ui，
    /// boot 同构——launch 链 build + 504/540 播种可无头全跑）。
    fn t540_resolver_session() -> crate::ui::session::DesktopSession {
        let mut ds = t3_session_with_shell();
        // PLAN-007 T-03：注册表语料扫本仓 examples/ui。
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui");
        let apps = std::sync::Arc::new(crate::ui::app_registry::scan_apps(
            &root,
            &crate::ui::app_registry::ScanOptions::default(),
        ));
        ds.desktop.registry_entries = (*apps).clone();
        ds.desktop.app_resolver = Some({
            let apps = apps.clone();
            std::sync::Arc::new(move |name: &str| {
                // Plan 551：⚙️ 靶 = os-config——真实前端居兄弟仓（CI 不依赖
                // 兄弟仓检出），合成最小桩承载 launch-or-focus/ensure 机制
                // 测试；页面契约归 os-config 仓自测 + T8 实机验收。
                if name == OSCONFIG_APP_ID {
                    return Some(crate::ui::session::LaunchSpec {
                        media_root: None,
                        photo_root: None,
                        back_entry: None,

                        code: T551_OSCONFIG_STUB_AT.to_string(),
                        source_path: None,
                        title: Some("auto-os-config-front".to_string()),
                        name: Some("os-config".to_string()),
                        fit: false,
                        daemon: Some("autoos".to_string()),
                        back_root: None,
                        exe: None,
                        opens: Vec::new(),
                        render_decl: None,
                    });
                }
                let e = apps.iter().find(|a| a.id == name)?;
                Some(crate::ui::session::LaunchSpec {
                    media_root: None,
                    photo_root: None,
                    back_entry: None,

                    code: std::fs::read_to_string(&e.entry).ok()?,
                    source_path: Some(e.entry.to_string_lossy().to_string()),
                    title: Some(e.display_title().to_string()),
                    name: e.name.clone(),
                    fit: e.fit,
                    daemon: None,
                    back_root: None,
                    exe: None,
                    opens: Vec::new(),
                    render_decl: None,
                })
            })
        });
        ds
    }

    /// 已开设置窗的 AppId（按 registry_id 定位——T7 后无 overlay 槽）。
    fn t540_settings_app(ds: &crate::ui::session::DesktopSession) -> crate::ui::session::AppId {
        ds.host
            .as_ref()
            .and_then(|h| {
                h.wm.wins
                    .iter()
                    .find(|(_, v)| v.registry_id.as_deref() == Some(OSCONFIG_APP_ID))
                    .map(|(_, v)| v.app)
            })
            .expect("设置窗已开")
    }

    /// Plan 486 T1/T4：拖入高亮落位/清除（set_native_drag_over 经映射或
    /// headless 恒等——断言映射无关的存在性）+ 高亮元素构建冒烟。
    #[test]
    #[cfg(windows)]
    fn native_drag_over_overlay_sets_and_clears() {
        let mut ds = t3_session_with_shell();
        set_native_drag_over(
            &mut ds,
            Some(crate::ui::native_dock::Rect::new(10, 20, 300, 200)),
        );
        let r = ds.native_drag_over.expect("高亮应落位");
        // headless 测试进程无自有可见窗口 → 恒等退化；有窗口环境经
        // CoordMapper 换算——两者宽度语义一致（不缩放，仅平移），故锁宽高。
        assert_eq!((r.width, r.height), (300.0, 200.0));
        let _el = crate::ui::iced::virtual_window::native_drag_over_element(r);
        set_native_drag_over(&mut ds, None);
        assert!(ds.native_drag_over.is_none(), "清除后不应残留高亮");
    }

    fn t3_add_win(
        ds: &mut crate::ui::session::DesktopSession,
        title: &str,
    ) -> crate::ui::session::Wid {
        let comp = crate::build_dynamic_component(T3_WIN_AT, None).unwrap();
        let app = ds.allocate_app(comp);
        ds.wm_add_win(
            app,
            title.to_string(),
            iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(200.0, 150.0)),
        )
    }

    fn t3_read(ds: &crate::ui::session::DesktopSession, field: &str) -> auto_val::Value {
        let shell = ds.desktop.shell_app.unwrap();
        ds.apps
            .get(&shell)
            .unwrap()
            .component
            .read_state(field)
            .unwrap_or_else(|e| panic!("{field} 读回失败: {e}"))
    }

    /// 数组读回：声明 `var x = []` 的字段经 write_state_vec 后可能物化为
    /// VmRef/Int(id) 指向 ListData 堆对象（EDGE-16/Plan 390 H3b 同型）——
    /// 测试侧解引用取元素。
    fn t3_read_array(ds: &crate::ui::session::DesktopSession, field: &str) -> Vec<auto_val::Value> {
        let shell = ds.desktop.shell_app.unwrap();
        let app = ds.apps.get(&shell).unwrap();
        let val = app
            .component
            .read_state(field)
            .unwrap_or_else(|e| panic!("{field} 读回失败: {e}"));
        match val {
            auto_val::Value::Array(a) => a.values,
            auto_val::Value::VmRef(r) => t3_deref_list(app, r.id as u64, field),
            auto_val::Value::Int(id) => t3_deref_list(app, id as u64, field),
            other => panic!("{field} 既非数组也非列表引用: {other:?}"),
        }
    }

    fn t3_deref_list(
        app: &crate::ui::session::AppSession,
        id: u64,
        field: &str,
    ) -> Vec<auto_val::Value> {
        let vm = app.component.bridge().vm();
        let obj = vm
            .get_heap_object(id)
            .unwrap_or_else(|| panic!("{field} 堆对象 {id} 不存在"));
        let guard = obj.read().unwrap();
        if let Some(list) = guard
            .as_any()
            .downcast_ref::<auto_lang_core::vm::types::ListData<auto_val::Value>>()
        {
            return list.elems.clone();
        }
        // `.split(",")` 等 string 方法产物是 ListData<String>（单型列表）。
        if let Some(list) = guard
            .as_any()
            .downcast_ref::<auto_lang_core::vm::types::ListData<std::string::String>>()
        {
            return list
                .elems
                .iter()
                .map(|s| auto_val::Value::Str(s.clone().into()))
                .collect();
        }
        panic!(
            "{field} 堆对象非 ListData（type_tag={:?}）",
            guard.type_tag()
        )
    }

    fn t3_obj_str(o: &auto_val::Obj, key: &str) -> String {
        match o.get(key) {
            Some(auto_val::Value::Str(s)) => s.to_string(),
            other => panic!("字段 {key} 异常: {other:?}"),
        }
    }

    /// 投影往返：wins 条目字段集 v1（wid/title/focused/workspace/app/icon）+
    /// meta + workspaces（默认单分区 current）+ 指纹非空。
    #[test]
    fn projection_v1_roundtrip_wins_meta_workspaces() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        let b = t3_add_win(&mut ds, "Beta");
        sync_shell_windows(&mut ds);

        let wins = t3_read_array(&ds, "__wm_wins");
        assert_eq!(wins.len(), 2, "投影窗口数");
        let auto_val::Value::Obj(first) = &wins[0] else {
            panic!("wins 条目应为 Obj: {:?}", wins[0])
        };
        assert_eq!(t3_obj_str(first, "title"), "Alpha");
        assert_eq!(t3_obj_str(first, "wid"), a.0.to_string());
        assert_eq!(t3_obj_str(first, "workspace"), "0", "v1 新字段：分区归属");
        assert_eq!(t3_obj_str(first, "app"), "", "boot 窗 registry_id 缺省空串");
        assert_eq!(t3_obj_str(first, "icon"), "app-window", "icon 缺省回退");
        assert_eq!(t3_obj_str(first, "focused"), "", "后开窗为焦点");
        let auto_val::Value::Obj(second) = &wins[1] else {
            panic!("wins 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(second, "focused"), "1");

        match t3_read(&ds, "__wm_meta") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), format!("free\t{}", b.0)),
            other => panic!("__wm_meta 读回异常: {other:?}"),
        }
        let wss = t3_read_array(&ds, "__wm_workspaces");
        assert_eq!(wss.len(), 2, "pack 默认 2 分区（T5 补记）");
        let auto_val::Value::Obj(ws) = &wss[0] else {
            panic!("workspace 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(ws, "id"), "0");
        assert_eq!(t3_obj_str(ws, "name"), "Desktop 1");
        assert_eq!(t3_obj_str(ws, "current"), "1");

        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(!s.to_string().is_empty(), "指纹已置"),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
    }

    /// Plan 486 T2 v1.3：native 槽位投影——Docked 态条目进 `__wm_wins`
    /// 尾部（wid="N<slot>" 隔离编码、五字段集、focused 空）；瞬时态
    /// （Candidate）不投影；槽位增删触发指纹变化（第二次同步重写哨兵）。
    #[test]
    fn projection_v13_native_slot_entries_and_fingerprint() {
        use crate::ui::native_dock::{NativeSlot, NativeSlotId, Rect, SlotState};
        let mut ds = t3_session_with_shell();
        t3_add_win(&mut ds, "Alpha");
        let mut docked = NativeSlot::new_candidate(
            NativeSlotId(3),
            crate::ui::native_dock::NativeHwnd(0x1234),
            4242,
            "记事本",
            Rect::new(10, 10, 800, 600),
            Rect::new(0, 0, 640, 480),
        );
        docked.state = SlotState::Docked;
        // PLAN-709：workspace 归属（v1.11 投影面）。
        docked.workspace = 0;
        let mut transient = docked.clone();
        transient.state = SlotState::Candidate;
        {
            let host = ds.host.as_mut().unwrap();
            host.wm.native_slots.insert(NativeSlotId(3), docked);
            host.wm.native_slots.insert(NativeSlotId(4), transient);
        }
        sync_shell_windows(&mut ds);
        let wins = t3_read_array(&ds, "__wm_wins");
        assert_eq!(
            wins.len(),
            2,
            "App 窗 + 1 个 Docked 槽位（Candidate 不投影）"
        );
        let auto_val::Value::Obj(native) = &wins[1] else {
            panic!("native 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(native, "wid"), "N3");
        assert_eq!(t3_obj_str(native, "title"), "记事本");
        assert_eq!(t3_obj_str(native, "native"), "1");
        assert_eq!(t3_obj_str(native, "icon"), "app-window");
        // PLAN-709 v1.11：workspace 实时 + focused 实时位（未聚焦 = ""）。
        assert_eq!(t3_obj_str(native, "focused"), "");
        assert_eq!(t3_obj_str(native, "workspace"), "0");
        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(
                s.to_string().contains("N3:0,0,"),
                "指纹应含 native 槽位段（focused,workspace 同型）: {}",
                s.to_string()
            ),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
        // PLAN-709 v1.11：槽位聚焦 → focused 位实时（伪 Wid 域）+ 指纹翻位。
        {
            let host = ds.host.as_mut().unwrap();
            assert!(host.wm.focus_native_slot(NativeSlotId(3)));
        }
        sync_shell_windows(&mut ds);
        let wins = t3_read_array(&ds, "__wm_wins");
        let auto_val::Value::Obj(native) = &wins[1] else {
            panic!("native 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(native, "focused"), "1", "聚焦后 focused 实时");
        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(
                s.to_string().contains("N3:1,0,"),
                "指纹 focused 位翻 1: {}",
                s.to_string()
            ),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
        // 指纹门控：槽位移除后再次同步 → 指纹变化 → 哨兵被覆盖。
        let shell = ds.desktop.shell_app.unwrap();
        ds.apps
            .get_mut(&shell)
            .unwrap()
            .component
            .write_state("__wm_meta", auto_val::Value::str("sentinel"))
            .unwrap();
        ds.host
            .as_mut()
            .unwrap()
            .wm
            .native_slots
            .remove(&NativeSlotId(3));
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_meta") {
            auto_val::Value::Str(s) => assert_ne!(s.to_string(), "sentinel", "槽位变化应触发重写"),
            other => panic!("__wm_meta 读回异常: {other:?}"),
        }
    }

    /// PLAN-012 W4 v1.6：`__dock_pinned_csv` 派生投影——",id1,id2," 串
    /// （shell view `contains` 消费，dock 窗口条目与固定图标去重判据面）
    /// + 指纹 "|pinned:" 段；会话域 dock_pinned 变更 → 下次同步指纹差分
    /// 拾取重写（哨兵机制同 v1.3 native 槽位测试）。
    #[test]
    fn projection_v16_dock_pinned_csv_and_fingerprint() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        // PLAN-012 W7：聚焦窗挂 registry id → __wm_focused_app 派生读回
        // （三态底条/高亮标量判据面）。
        if let Some(v) = ds.host.as_mut().unwrap().wm.wins.get_mut(&a) {
            v.registry_id = Some("011-calculator".to_string());
        }
        t3_add_win(&mut ds, "Beta");
        ds.desktop.dock_pinned = vec!["011-calculator".to_string(), "013-todo".to_string()];
        sync_shell_windows(&mut ds);
        // 后开窗为焦点（无 registry id）→ focused_app 空。
        match t3_read(&ds, "__wm_focused_app") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "", "无 registry 聚焦窗 → 空串"),
            other => panic!("__wm_focused_app 读回异常: {other:?}"),
        }
        // 焦点移回 calculator 窗 → focused_app 派生翻转（fp 经 focused_wid 变化触发重写）。
        ds.wm_focus(a);
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_focused_app") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "011-calculator"),
            other => panic!("__wm_focused_app 读回异常: {other:?}"),
        }
        match t3_read(&ds, "__dock_pinned_csv") {
            auto_val::Value::Str(s) => {
                assert_eq!(
                    s.to_string(),
                    ",011-calculator,013-todo,",
                    "csv 前后逗号封边"
                )
            }
            other => panic!("__dock_pinned_csv 读回异常: {other:?}"),
        }
        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(
                s.to_string().contains("|pinned:011-calculator,013-todo;"),
                "指纹应含 pinned 段: {}",
                s.to_string()
            ),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
        // 指纹门控：会话域变更（unpin 一枚）→ 指纹变化 → 下次同步重写。
        let shell = ds.desktop.shell_app.unwrap();
        ds.apps
            .get_mut(&shell)
            .unwrap()
            .component
            .write_state("__dock_pinned_csv", auto_val::Value::str("sentinel"))
            .unwrap();
        ds.desktop.dock_pinned = vec!["013-todo".to_string()];
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__dock_pinned_csv") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), ",013-todo,", "变更应触发重写"),
            other => panic!("__dock_pinned_csv 读回异常: {other:?}"),
        }
        // 空表 = 单纯 ","（空 csv 串——shell contains 条件天然不命中）。
        ds.desktop.dock_pinned = Vec::new();
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__dock_pinned_csv") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), ",", "空表 → 逗号哨兵串"),
            other => panic!("__dock_pinned_csv 读回异常: {other:?}"),
        }
    }

    /// 指纹门控：无 WM 变化 → 跳过重写（哨兵不被覆盖、dirty 不置位）；
    /// WM 变化（焦点切换）→ 整组重写 + view_dirty 置位。
    #[test]
    fn projection_fingerprint_gates_rewrite_and_view_dirty() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        t3_add_win(&mut ds, "Beta");
        sync_shell_windows(&mut ds);
        let shell = ds.desktop.shell_app.unwrap();
        // 哨兵：无变化时第二次同步不得覆盖。
        ds.apps
            .get_mut(&shell)
            .unwrap()
            .component
            .write_state("__wm_meta", auto_val::Value::str("sentinel"))
            .unwrap();
        *ds.apps
            .get_mut(&shell)
            .unwrap()
            .state
            .view_dirty
            .borrow_mut() = false;
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_meta") {
            auto_val::Value::Str(ref s) => {
                assert_eq!(s.to_string(), "sentinel", "指纹未变跳过重写")
            }
            other => panic!("__wm_meta 读回异常: {other:?}"),
        }
        assert!(
            !*ds.apps.get(&shell).unwrap().state.view_dirty.borrow(),
            "跳过时不置 dirty"
        );
        // WM 变化：焦点切回 Alpha → meta 重写 + dirty 置位。
        ds.wm_focus(a);
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_meta") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), format!("free\t{}", a.0)),
            other => panic!("__wm_meta 读回异常: {other:?}"),
        }
        assert!(
            *ds.apps.get(&shell).unwrap().state.view_dirty.borrow(),
            "有变重写并置 dirty"
        );
    }

    /// 分区切换反映到投影：workspaces current 翻转 + wins focused 段随焦点
    /// 让渡重算（跨分区窗保留在全集，workspace 字段不动）。
    #[test]
    fn projection_v1_reflects_workspace_switch() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        ds.wm_set_workspace(1); // pack 默认分区 1（T5 补记：默认 2 分区）
        let b = t3_add_win(&mut ds, "Beta");
        sync_shell_windows(&mut ds);

        let wins = t3_read_array(&ds, "__wm_wins");
        assert_eq!(wins.len(), 2, "全集投影（跨分区）");
        let auto_val::Value::Obj(first) = &wins[0] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(first, "workspace"), "0", "Alpha 仍在分区 0");
        let auto_val::Value::Obj(second) = &wins[1] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(second, "wid"), b.0.to_string());
        assert_eq!(t3_obj_str(second, "workspace"), "1");
        assert_eq!(t3_obj_str(second, "focused"), "1", "分区 1 焦点 = Beta");

        let wss = t3_read_array(&ds, "__wm_workspaces");
        assert_eq!(wss.len(), 2, "pack 默认 2 分区");
        let auto_val::Value::Obj(ws0) = &wss[0] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(ws0, "current"), "", "分区 0 非当前");
        let auto_val::Value::Obj(ws1o) = &wss[1] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(ws1o, "current"), "1", "分区 1 当前");
        let _ = a;
    }

    /// registry_id → app/icon 字段（launch_app 回填；icon 自注册表实时查）。
    #[test]
    fn projection_v1_registry_icon_and_app_fields() {
        use crate::ui::app_registry::AppRegistryEntry;
        let mut ds = t3_session_with_shell();
        ds.desktop.registry_entries = vec![AppRegistryEntry {
            media_root: None,
            photo_root: None,
            back_entry: None,

            id: "011-calculator".to_string(),
            title: "calculator".to_string(),
            title_zh: None,
            name: None,
            icon: "calculator".to_string(),
            category: "tool".to_string(),
            entry: std::path::PathBuf::from("011-calculator"),
            render: "vm".to_string(),
            daemon: None,
            back_root: None,
            fit: false,
            desktop_visible: true,
            desktop_exe: None,
            desktop_render: None,
            opens: Vec::new(),
        }];
        ds.desktop.app_resolver = Some(std::sync::Arc::new(|name: &str| {
            (name == "011-calculator").then(|| crate::ui::session::LaunchSpec {
                media_root: None,
                photo_root: None,
                back_entry: None,

                code: T3_WIN_AT.to_string(),
                source_path: None,
                title: Some("calculator".to_string()),
                name: None,
                daemon: None,
                back_root: None,
                fit: false,
                exe: None,
                opens: Vec::new(),
                render_decl: None,
            })
        }));
        ds.launch_app("011-calculator").expect("launch");
        sync_shell_windows(&mut ds);
        let wins = t3_read_array(&ds, "__wm_wins");
        assert_eq!(wins.len(), 1);
        let auto_val::Value::Obj(o) = &wins[0] else {
            panic!("Obj")
        };
        assert_eq!(
            t3_obj_str(o, "app"),
            "011-calculator",
            "launch 回填 registry_id"
        );
        assert_eq!(t3_obj_str(o, "icon"), "calculator", "icon 自注册表解析");
    }

    // ---- Plan 478 T3：投影协议 v1.1（__wm_mru + workspaces label + 指纹扩展）----

    /// v1.1：`__wm_mru` = 当前分区 MRU 序（front=最近聚焦，条目同
    /// `__wm_wins` 六字段）+ `__wm_workspaces.label` 1 基人读标签。
    #[test]
    fn projection_v11_mru_order_and_workspace_label() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        t3_add_win(&mut ds, "Beta");
        let c = t3_add_win(&mut ds, "Gamma");
        sync_shell_windows(&mut ds);

        let mru = t3_read_array(&ds, "__wm_mru");
        assert_eq!(mru.len(), 3, "当前分区全集（boot 窗全在分区 0）");
        let auto_val::Value::Obj(first) = &mru[0] else {
            panic!("mru 条目应为 Obj")
        };
        assert_eq!(
            t3_obj_str(first, "wid"),
            c.0.to_string(),
            "MRU front=最近聚焦"
        );
        assert_eq!(
            t3_obj_str(first, "title"),
            "Gamma",
            "条目同 __wm_wins 六字段"
        );
        // 聚焦 a → MRU 前插，投影序随之。
        ds.wm_focus(a);
        sync_shell_windows(&mut ds);
        let mru = t3_read_array(&ds, "__wm_mru");
        let auto_val::Value::Obj(first) = &mru[0] else {
            panic!("mru 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(first, "wid"), a.0.to_string(), "聚焦后 front=a");

        // label = 1 基人读标签（宿主投影，避开 .at 算术——T1 施工图 §4.1）。
        let wss = t3_read_array(&ds, "__wm_workspaces");
        assert_eq!(wss.len(), 2);
        let auto_val::Value::Obj(ws0) = &wss[0] else {
            panic!("workspace 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(ws0, "label"), "1");
        let auto_val::Value::Obj(ws1o) = &wss[1] else {
            panic!("workspace 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(ws1o, "label"), "2");
    }

    /// v1.1：mru 投影按当前分区过滤（焦点环不跨分区，472 语义延续）+
    /// 指纹 v1.1 段（分区段含 label、尾接 mru 段）门控翻转。
    #[test]
    fn projection_v11_mru_filters_partition_and_fingerprint_segments() {
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha"); // ws0
        ds.wm_set_workspace(1);
        let b = t3_add_win(&mut ds, "Beta"); // ws1
        sync_shell_windows(&mut ds);

        let mru = t3_read_array(&ds, "__wm_mru");
        assert_eq!(mru.len(), 1, "mru 过滤当前分区（不跨分区）");
        let auto_val::Value::Obj(first) = &mru[0] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(first, "wid"), b.0.to_string());

        // 指纹 v1.1：分区段 "{id}:{current},{label};" + 尾段 mru "{wid};"。
        // Plan 479 v1.2：mru 段后尾接 notes 段（协议 §3——金样随版升级）。
        let fp = match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => s.to_string(),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        };
        assert!(fp.contains("0:0,1;1:1,2;"), "指纹分区段含 label: {fp}");
        assert!(
            fp.contains(&format!("|{};|notes:", b.0)),
            "指纹 mru 段后接 notes 段（v1.2 尾段）: {fp}"
        );

        // 切回分区 0：mru 段翻转 → 指纹变 → 重写。
        ds.wm_set_workspace(0);
        sync_shell_windows(&mut ds);
        let mru = t3_read_array(&ds, "__wm_mru");
        assert_eq!(mru.len(), 1);
        let auto_val::Value::Obj(first) = &mru[0] else {
            panic!("Obj")
        };
        assert_eq!(t3_obj_str(first, "wid"), a.0.to_string());
    }

    // ---- Plan 478 T4：switcher overlay（summon/advance/confirm 无头流；
    // 464 summon_launcher_mounts_and_injects 同型。switcher.at 为进程内嵌
    // 资产——本测直载真源，非裁剪副本）----

    /// switcher 召唤推进确认无头流：懒挂载（真 assets/switcher.at）+ MRU
    /// 快照注入（rows 序 = MRU 序）→ Advance 环走 sel → confirm（Focus
    /// handler）写 `focus\t<wid>` 记录 + 自隐 → drain 可达执行体。
    #[test]
    // ---- Plan 497 T3：S3 Status 栏（时钟/托盘组/每窗口缩略）----
    // 命名族 desktop_mcp_*：`cargo t desktop_mcp` 过滤面（计划验证命令）。

    /// 时钟注入 + 分钟内去重：update_shell_clock 首调注入 HH:MM +
    /// view_dirty；同分钟复调零写入零 dirty（ServiceTick 400ms 帧泵下
    /// 稳态零重建）。**真资产 assets/shell.at 直载**（478 switcher 先例
    /// ——非 ShellProbe 裁剪副本；同时覆盖 497 托盘组/时钟节点的 .at
    /// 装载面）。
    #[test]
    fn desktop_mcp_clock_injects_and_dedupes() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::ui::shell::build_shell_component().expect("真 shell.at 装载");
        ds.desktop.shell_app = Some(ds.allocate_app(comp));
        update_shell_clock(&mut ds);
        let v = t3_read(&ds, "__wm_clock");
        let auto_val::Value::Str(hhmm) = v else {
            panic!("__wm_clock 应为字符串: {v:?}")
        };
        let hhmm = hhmm.to_string();
        assert_eq!(hhmm.len(), 5, "HH:MM 形态: {hhmm}");
        assert_eq!(hhmm.as_bytes()[2], b':', "HH:MM 分隔符: {hhmm}");
        assert!(
            hhmm[..2].chars().all(|c| c.is_ascii_digit())
                && hhmm[3..].chars().all(|c| c.is_ascii_digit()),
            "HH:MM 数字段: {hhmm}"
        );
        // 首调已置 view_dirty——复位后同分钟复调不得再置。
        let shell = ds.desktop.shell_app.unwrap();
        *ds.apps.get(&shell).unwrap().state.view_dirty.borrow_mut() = false;
        update_shell_clock(&mut ds);
        assert!(
            !*ds.apps.get(&shell).unwrap().state.view_dirty.borrow(),
            "同分钟复调零 dirty（分钟变化才注入）"
        );
        assert_eq!(
            t3_read(&ds, "__wm_clock"),
            auto_val::Value::Str(hhmm.as_str().into()),
            "同分钟复调零写入"
        );
    }

    #[test]
    /// Plan 497 T3：switcher mru_thumbs 注入 + miss 预抓。cache_put 一窗
    /// 快照后召唤：就绪标记按 MRU 序注入（"1"/""）；miss 窗进抓取队列
    /// （宿主 ServiceTick 一次整窗截图服务）。
    #[test]
    fn desktop_mcp_switcher_thumbs_injected_and_requested() {
        crate::ui::iced::snapshot::invalidate_all();
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        let b = t3_add_win(&mut ds, "Beta");
        // b（MRU front）预置快照；a 保持 miss。
        crate::ui::iced::snapshot::cache_put(
            b,
            crate::ui::iced::snapshot::WindowSnapshot {
                rgba: vec![0x88; 4 * 64 * 48],
                w: 64,
                h: 48,
            },
        );
        let _task = summon_switcher(&mut ds);
        assert!(ds.switcher_visible(), "召唤后 switcher visible");
        let sw = ds.desktop.switcher_app.expect("懒挂载完成");
        let app = ds.apps.get(&sw).unwrap();
        let thumbs = match app.component.read_state("mru_thumbs").unwrap() {
            auto_val::Value::Array(a) => a.values,
            auto_val::Value::VmRef(r) => t3_deref_list(app, r.id as u64, "mru_thumbs"),
            auto_val::Value::Int(id) => t3_deref_list(app, id as u64, "mru_thumbs"),
            other => panic!("mru_thumbs 应为数组: {other:?}"),
        };
        assert_eq!(thumbs.len(), 2, "MRU 全量两窗");
        assert_eq!(thumbs[0], auto_val::Value::str("1"), "front=b 快照就绪");
        assert_eq!(thumbs[1], auto_val::Value::str(""), "a miss 标空串");
        // miss 窗 a 已入抓取队列（b 命中不入队）。
        let pending = crate::ui::iced::snapshot::take_capture_requests();
        assert_eq!(pending, vec![a], "miss 窗入队预抓");
        // 行消费面：rows[0].wid = b（缩略行挂 window_thumbnail 的 wid 源）。
        drop(app);
        let app = ds.apps.get_mut(&sw).unwrap();
        app.component
            .bridge_mut()
            .call_handler("RebuildMru", &[])
            .expect("RebuildMru");
        let rows = match ds
            .apps
            .get(&sw)
            .unwrap()
            .component
            .read_state("rows")
            .unwrap()
        {
            auto_val::Value::VmRef(r) => {
                t3_deref_list(ds.apps.get(&sw).unwrap(), r.id as u64, "rows")
            }
            auto_val::Value::Int(id) => t3_deref_list(ds.apps.get(&sw).unwrap(), id as u64, "rows"),
            other => panic!("rows 应为 VmRef: {other:?}"),
        };
        let vm = ds.apps.get(&sw).unwrap().component.bridge().vm();
        let wid_of = |i: usize| -> String {
            let auto_val::Value::VmRef(r) = &rows[i] else {
                panic!("rows[{i}] VmRef")
            };
            let obj = vm.get_heap_object(r.id as u64).expect("heap");
            let g = obj.read().unwrap();
            let od = g
                .as_any()
                .downcast_ref::<auto_lang_core::vm::types::ObjectData>()
                .expect("OD");
            match od.get(&auto_val::ValueKey::from("wid".to_string())) {
                Some(auto_val::Value::Str(s)) => s.to_string(),
                other => panic!("wid: {other:?}"),
            }
        };
        assert_eq!(wid_of(0), b.0.to_string(), "rows[0].wid=b（就绪行）");
        assert_eq!(
            wid_of(1),
            a.0.to_string(),
            "rows[1].wid=a（miss 行 fallback）"
        );
        crate::ui::iced::snapshot::invalidate_all();
    }

    #[test]
    /// Plan 497 T3：dock/pager hover 预览。真 shell.at 直载 + 两窗（跨分区）
    /// 投影后：HoverWin/HoverWs handler 写态 → View 树中出现 open popover +
    /// window_thumbnail 缩略叶（fallback 路径——headless 无快照）；
    /// HoverEnd/HoverWsEnd 后 open popover 消失。
    #[test]
    fn desktop_mcp_dock_pager_hover_popovers() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::ui::shell::build_shell_component().expect("真 shell.at 装载");
        ds.desktop.shell_app = Some(ds.allocate_app(comp));
        let a = t3_add_win(&mut ds, "Alpha");
        let b = t3_add_win(&mut ds, "Beta");
        // b 送分区 1（pager 预览分区面）。
        if let Some(host) = ds.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&b) {
                v.workspace = 1;
            }
        }
        sync_shell_windows(&mut ds);
        let shell = ds.desktop.shell_app.unwrap();

        // 树扫描：BFS 收 (popover_open_count, thumb_count)。view_children
        // 不含 Popover/MouseArea 子——此处补齐（测试专用窄遍历）。
        fn scan(
            v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
            pops: &mut usize,
            thumbs: &mut usize,
        ) {
            use crate::ui::view::View as V;
            {
                let kind = match v {
                    V::Popover { .. } => "Popover",
                    V::WorkspacePreview { .. } => "WorkspacePreview",
                    V::WindowThumbnail { .. } => "Thumb",
                    _ => ".",
                };
                let _ = kind;
            }
            let extra: Vec<&V<_>> = match v {
                V::Popover {
                    anchor,
                    content,
                    open,
                    ..
                } => {
                    if *open {
                        *pops += 1;
                    }
                    let mut v = vec![content.as_ref()];
                    if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                        v.push(w.as_ref());
                    }
                    v
                }
                V::MouseArea { content, .. } => vec![content.as_ref()],
                V::WindowThumbnail { .. } => {
                    *thumbs += 1;
                    vec![]
                }
                V::WorkspacePreview { .. } => {
                    *thumbs += 1;
                    vec![]
                }
                _ => vec![],
            };
            for c in view_children(v).into_iter().chain(extra) {
                scan(c, pops, thumbs);
            }
        }
        fn counts(ds: &crate::ui::session::DesktopSession) -> (usize, usize) {
            let shell = ds.desktop.shell_app.unwrap();
            let (view, _, _) = ds
                .apps
                .get(&shell)
                .unwrap()
                .component
                .view_with_debug_gated(false);
            let (mut pops, mut thumbs) = (0, 0);
            scan(&view, &mut pops, &mut thumbs);
            (pops, thumbs)
        }

        // 基线：无 hover——popover 全收起（open 计数 0）；缩略叶为树构建
        // 面（popover content 预构建、open 才走 iced overlay 渲染）：dock
        // 条目 ×2（a/b）。PLAN-526 T18：直列分区条（含 pager hover
        // popover）退役为切换面板——缩略叶 4→2。PLAN-012 W3：切换器
        // popover 化（content 预构建）且每分区卡一枚 workspace_preview
        // 叶 ×2（pack 默认分区）→ 预构建缩略/预览叶 2→4。
        // PLAN-040 F2b：条目 hover 预览列**全部同 app 实例**（合并 icon
        // 的预览选实例语义）——夹具两窗 app 串均空（同 app 全匹配），每
        // 条目预构建 2 枚缩略 → 缩略叶 2→4，预览叶 2 不变，基线 4→6。
        let (p0, t0) = counts(&ds);
        assert_eq!(
            (p0, t0),
            (0, 6),
            "无 hover 基线零 open、四缩略（2 条目×全实例）+两预览叶"
        );

        // dock hover b：b 条目 popover open ×1（缩略叶已在预构建集内）。
        {
            let app = ds.apps.get_mut(&shell).unwrap();
            app.component
                .bridge_mut()
                .call_handler("HoverWin", &[auto_val::Value::str(b.0.to_string())])
                .expect("HoverWin");
        }
        let (p1, t1) = counts(&ds);
        assert_eq!(p1, 1, "dock hover 打开单个 popover");
        assert_eq!(t1, 6, "缩略/预览叶集不变（open 不增建；F2b 后基线 6）");
        // PLAN-010 N6c：hover 离开只清预览态（HoverLeave，原 HoverEnd 退役）。
        {
            let app = ds.apps.get_mut(&shell).unwrap();
            app.component
                .bridge_mut()
                .call_handler("HoverLeave", &[])
                .expect("HoverLeave");
        }
        assert_eq!(counts(&ds).0, 0, "HoverLeave 收起 hover 预览");

        // N6c 回归：右键开菜单（win_menu=b）→ 菜单开启期 hover 预览被
        // win_menu 门抑制；hover 离开不再兼清菜单（原 HoverEnd 兼清使
        // 菜单被秒关、菜单项不可达——用户实测）；WinMenuClose 单独收菜单
        // （desktop.at icon 菜单 MenuClose 同构）。
        {
            let app = ds.apps.get_mut(&shell).unwrap();
            app.component
                .bridge_mut()
                .call_handler("WinMenu", &[auto_val::Value::str(b.0.to_string())])
                .expect("WinMenu");
        }
        assert_eq!(counts(&ds).0, 1, "win_menu 开菜单（无 hover）");
        {
            let app = ds.apps.get_mut(&shell).unwrap();
            app.component
                .bridge_mut()
                .call_handler("HoverLeave", &[])
                .expect("leave during menu");
        }
        assert_eq!(counts(&ds).0, 1, "菜单开启期 hover 离开不收菜单（N6c）");
        {
            let app = ds.apps.get_mut(&shell).unwrap();
            app.component
                .bridge_mut()
                .call_handler("WinMenuClose", &[])
                .expect("WinMenuClose");
        }
        assert_eq!(counts(&ds).0, 0, "WinMenuClose 收菜单");

        // PLAN-526 T18：直列分区条（HoverWs/HoverWsEnd pager popover）
        // 退役——hover 面只剩 dock 条目臂，base 不回归即可。
        assert_eq!(counts(&ds).0, 0, "无 hover 时零 open（T18 后基线）");
        let _ = a;
    }

    /// Plan 505 B2（债 P497-1）：pager 派生面（协议 v1.5）——同分区 6 窗
    /// 时 per-win `pager` 只保前 4、`ws.more` = "+2"；空分区 more 为
    /// 空串（badge 条件消费不出现）。shell.at 消费面（w.pager 旗标门）
    /// 见 `desktop_mcp_dock_pager_hover_popovers`。
    #[test]
    fn shell_projection_pager_truncation_v15() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::ui::shell::build_shell_component().expect("真 shell.at 装载");
        ds.desktop.shell_app = Some(ds.allocate_app(comp));
        for i in 0..6 {
            let _ = t3_add_win(&mut ds, &format!("W{i}"));
        }
        sync_shell_windows(&mut ds);
        let shell = ds.desktop.shell_app.unwrap();
        let comp = &ds.apps.get(&shell).unwrap().component;
        let wins = comp.read_state_as_vec("__wm_wins").expect("__wm_wins");
        let flagged = wins
            .iter()
            .filter(|w| {
                matches!(w, auto_val::Value::Obj(o)
                    if o.get("pager").is_some_and(|v| v.as_str() == "1"))
            })
            .count();
        assert_eq!(flagged, 4, "同分区 6 窗仅前 4 带 pager 旗标");
        let wss = comp
            .read_state_as_vec("__wm_workspaces")
            .expect("__wm_workspaces");
        let more_of = |v: &auto_val::Value| match v {
            auto_val::Value::Obj(o) => o
                .get("more")
                .map(|s| s.as_str().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        };
        assert_eq!(more_of(&wss[0]), "+2", "分区 0 溢出标签 +2");
        assert_eq!(more_of(&wss[1]), "", "空分区 more 空串");
    }

    #[test]
    /// Plan 505 C：验收通道注入排空——Bus 记录并入 shell `__desktop_cmd`
    /// （真实按钮同一消费臂）；Handler 直呼 shell onclick 同名 handler
    /// （写入总线记录同臂排空）。
    #[test]
    fn desktop_injects_flow_through_real_arms() {
        use crate::ui::session::{desktop_inject_push, DesktopCommand, DesktopInject};
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::ui::shell::build_shell_component().expect("真 shell.at 装载");
        ds.desktop.shell_app = Some(ds.allocate_app(comp));
        let shell = ds.desktop.shell_app.unwrap();

        // Bus 注入 → shell __desktop_cmd → drain_desktop_commands 同臂。
        desktop_inject_push(DesktopInject::Bus("open_settings".to_string()));
        apply_desktop_injects(&mut ds);
        let bus = match ds
            .apps
            .get(&shell)
            .unwrap()
            .component
            .read_state("__desktop_cmd")
        {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            _ => String::new(),
        };
        assert_eq!(bus, "open_settings", "Bus 注入落入 shell 总线状态");
        let cmds = ds.drain_desktop_commands();
        assert!(matches!(cmds.as_slice(), [DesktopCommand::OpenSettings]));

        // Handler 注入 → 真 shell.at OpenSettingsPanel（齿轮 onclick 同名）
        // → open_settings 记录入总线 → 同臂排空。
        desktop_inject_push(DesktopInject::Handler {
            app: "shell",
            handler: "OpenSettingsPanel".to_string(),
            arg: None,
            widget: None,
        });
        apply_desktop_injects(&mut ds);
        let cmds = ds.drain_desktop_commands();
        assert!(
            matches!(cmds.as_slice(), [DesktopCommand::OpenSettings]),
            "handler 写入的总线记录同臂排空"
        );
        // 排空后再无残余（幂等取尽）。
        apply_desktop_injects(&mut ds);
        assert!(ds.drain_desktop_commands().is_empty());
    }

    /// PLAN-030 T-06：DesktopBus 上行全链（inbox → drain → execute +
    /// 归因）——词表单点 encode 产记录逐族抽样（布局族/通知族/壁纸族；
    /// 召唤/设置/窗口族经 p030 e2e 腿 2 真按钮动词覆盖）。
    #[test]
    fn desktop_bus_inbox_drains_through_real_arms() {
        use crate::ui::layout::LayoutMode;
        use crate::ui::session::{DesktopCommand, DesktopSession};
        let mut ds = DesktopSession::__test_session();
        // PLAN-759 T-05：壁纸机器态隔离——空临时目录存活至测试结束，挡掉
        // env/stella 探测两级（712 T-13/T-14 后 #hex 不再短路目录回退，
        // 本机有 D:\Down\stella-os\wallpapers 时档②主题默认图会盖掉
        // SetWallpaper 的 #hex 断言——机器相关红）。断言保持逐字精确。
        let _wall_dir = tempfile::TempDir::new().expect("wallpapers 隔离目录");
        ds.desktop.config.wallpapers_dir = _wall_dir.path().to_string_lossy().to_string();
        ds.open_desktop(iced::window::Id::unique());
        let samples: Vec<(&str, DesktopCommand)> = vec![
            ("shell", DesktopCommand::SetLayout(LayoutMode::Grid)),
            (
                "shell",
                DesktopCommand::Notify("toast".into(), "bus 全链".into()),
            ),
            (
                "desktop-face",
                DesktopCommand::SetWallpaper("#101014".into()),
            ),
        ];
        for (src, cmd) in &samples {
            ds.desktop
                .desktop_bus_inbox
                .push((src.to_string(), cmd.clone()));
        }
        let _ = drain_desktop_bus_inbox(&mut ds);
        // 布局族：SetLayout 落 wm.layout。
        assert!(matches!(
            ds.host.as_ref().unwrap().wm.layout,
            LayoutMode::Grid
        ));
        // 通知族：Notify 落通知历史（归因 app = 末段 source）。
        assert_eq!(ds.desktop.notifications.borrow().len(), 1);
        assert_eq!(
            ds.desktop.notifications.borrow()[0].app,
            "desktop-face",
            "registry_id 归因随段（D4）"
        );
        // 壁纸族：SetWallpaper 落桌面壁纸态。
        assert_eq!(ds.desktop.desktop_wallpaper, "#101014");
        // inbox 取尽（幂等）。
        assert!(drain_desktop_bus_inbox(&mut ds).is_empty());
    }

    /// 宿主窗 resize 不变量（sync_host_resized）：fit 窗 window_size 保持
    /// 内容尺寸（不被宿主尺寸污染）且矩形在新窗格内居中；常规窗 window_size
    /// 随新窗格（else 分支）。
    #[test]
    fn host_resize_preserves_fit_window_and_relayouts() {
        use crate::ui::iced::virtual_window::{BORDER, TITLEBAR_H};
        use crate::ui::layout::{usable_rect, LayoutMode};
        use crate::ui::session::{AppId, DesktopSession};
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let add = |ds: &mut DesktopSession, title: &str| {
            ds.host.as_mut().unwrap().wm.add_win(
                AppId(1),
                title.into(),
                iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(800.0, 600.0)),
            )
        };
        let normal = add(&mut ds, "N");
        let fit = add(&mut ds, "F");
        {
            let host = ds.host.as_mut().unwrap();
            let v = host.wm.wins.get_mut(&fit).unwrap();
            v.fit_enabled.set(true);
            *v.window_size.borrow_mut() = iced::Size::new(460.0, 540.0);
            host.wm.layout = LayoutMode::Grid;
        }
        sync_host_resized(&mut ds, iced::Size::new(1600.0, 1000.0));
        let host = ds.host.as_ref().unwrap();
        let usable = usable_rect(
            iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1600.0, 1000.0)),
            ds.desktop.dock_edges,
        );
        // Grid 2 窗：左右均分，fit 窗在右格。
        let pane_w = usable.width / 2.0;
        let pane = iced::Rectangle::new(
            iced::Point::new(usable.x + pane_w, usable.y),
            iced::Size::new(pane_w, usable.height),
        );
        // 核心不变量：fit 窗内容尺寸不被宿主尺寸覆盖。
        assert_eq!(
            *host.wm.wins[&fit].window_size.borrow(),
            iced::Size::new(460.0, 540.0)
        );
        // 矩形 = 内容外沿在新窗格内居中（不铺满格）。
        let outer_w = 460.0 + 2.0 * BORDER;
        let outer_h = 540.0 + TITLEBAR_H + 2.0 * BORDER;
        let rf = *host.wm.wins[&fit].rect.borrow();
        assert!((rf.width - outer_w).abs() < 0.51 && (rf.height - outer_h).abs() < 0.51);
        assert!(
            ((rf.x - (pane.x + (pane_w - outer_w) / 2.0)).abs() < 0.51)
                && ((rf.y - (pane.y + (usable.height - outer_h) / 2.0)).abs() < 0.51),
            "fit rect {{x:{}, y:{}, w:{}, h:{}}} not centered in pane {{x:{}, y:{}, w:{}, h:{}}}",
            rf.x,
            rf.y,
            rf.width,
            rf.height,
            pane.x,
            pane.y,
            pane.width,
            pane.height
        );
        // 常规窗：window_size 随新窗格（else 分支语义保持）。
        assert_eq!(
            *host.wm.wins[&normal].window_size.borrow(),
            iced::Size::new(pane_w, usable.height)
        );
    }

    /// PLAN-030 T-06：投影推送指纹门（未变不推/变了推/强制失效全量推）。
    #[test]
    fn shell_projection_push_fingerprint_gate() {
        use crate::ui::desktop_protocol::stage3::BrokerClient;
        use crate::ui::desktop_protocol::transport::Transport;
        use crate::ui::session::DesktopSession;
        use std::collections::VecDeque;
        use std::sync::{Arc, Mutex};

        /// 测试传输对（Send 版 loopback——LoopbackEnd 为 Rc 不可入
        /// BrokerClient）。
        struct MockPipe(Arc<Mutex<VecDeque<Vec<u8>>>>);
        impl Transport for MockPipe {
            fn send(
                &mut self,
                msg: &crate::ui::desktop_protocol::message::ProtocolMsg,
            ) -> Result<(), crate::ui::desktop_protocol::TransportError> {
                self.0.lock().unwrap().push_back(msg.encode());
                Ok(())
            }
            fn try_recv(
                &mut self,
            ) -> Option<
                Result<
                    crate::ui::desktop_protocol::message::ProtocolMsg,
                    crate::ui::desktop_protocol::CodecError,
                >,
            > {
                let mut q = self.0.lock().unwrap();
                q.pop_front()
                    .map(|bytes| crate::ui::desktop_protocol::message::ProtocolMsg::decode(&bytes))
            }
            fn pending(&self) -> usize {
                self.0.lock().unwrap().len()
            }
        }
        let (tx, rx) = {
            let shared: Arc<Mutex<VecDeque<Vec<u8>>>> = Arc::new(Mutex::new(VecDeque::new()));
            let rx: Arc<Mutex<VecDeque<Vec<u8>>>> = Arc::clone(&shared);
            (MockPipe(shared), MockPipe(rx))
        };
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let mut client = BrokerClient::new("shell-test".into(), Box::new(tx));
        client.app_name = Some("shell".into());
        ds.broker_clients.insert("shell-test".into(), client);
        ds.desktop.shell_pipe = Some("shell-test".into());

        let mut rx = rx;
        let recv_pushes = |rx: &mut MockPipe| -> usize {
            let mut n = 0;
            while let Some(Ok(msg)) = rx.try_recv() {
                if matches!(
                    &msg,
                    crate::ui::desktop_protocol::message::ProtocolMsg::Control(
                        crate::ui::desktop_protocol::message::ControlMsg::ShellProjectionPush { .. }
                    )
                ) {
                    n += 1;
                }
            }
            n
        };

        // 首推：指纹空 → 全量（shell 面 + desktop 面）。
        push_shell_projection_outproc(&mut ds);
        assert_eq!(recv_pushes(&mut rx), 2, "首拍 shell+desktop 两面各一推");
        assert!(ds.desktop.shell_push_fp.is_some());
        assert!(ds.desktop.shell_desk_fp.is_some());
        // 未变不推（指纹门）。
        push_shell_projection_outproc(&mut ds);
        assert_eq!(recv_pushes(&mut rx), 0, "指纹未变零推送");
        // 强制失效（respawn attach 同位）→ 全量重推。
        ds.desktop.shell_push_fp = None;
        push_shell_projection_outproc(&mut ds);
        assert_eq!(recv_pushes(&mut rx), 1, "失效后 shell 面重推");
    }

    /// Plan 505 B1 回归：单份任务栏结构——根 col 承载位置类（bottom 缺省
    /// → flex-col-reverse；top → 无翻转类），任务栏按钮群在视图树在位。
    /// （S7 实拍曾现任务栏消失——此测锁定视图树侧结构正确性。）
    #[test]
    fn shell_root_col_position_classes_and_taskbar_present() {
        use crate::ui::style::StyleClass;
        use crate::ui::view::View as V;
        fn count_buttons(v: &V<crate::ui::interpreter::DynamicMessage>) -> usize {
            let mut n = match v {
                V::Button { .. } => 1,
                _ => 0,
            };
            // PLAN-012 W3/W4：切换器钮入 popover、窗口条目钮入
            // popover+mouse-area——计数器须随结构钻入，否则漏计。
            let mut kids = view_children(v);
            if let V::Popover {
                anchor, content, ..
            } = v
            {
                if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                    kids.push(w.as_ref());
                }
                kids.push(content.as_ref());
            }
            if let V::MouseArea { content, .. } = v {
                kids.push(content.as_ref());
            }
            for c in kids {
                n += count_buttons(c);
            }
            n
        }
        let comp = crate::ui::shell::build_shell_component().expect("真 shell.at 装载");
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let id = ds.allocate_app(comp);
        ds.desktop.shell_app = Some(id);

        // bottom（pack 缺省）→ 根 col 含 FlexColReverse（B1 翻转承载）。
        let (view, _, _) = ds
            .apps
            .get(&id)
            .unwrap()
            .component
            .view_with_debug_gated(false);
        let V::Column {
            style, children, ..
        } = &view
        else {
            panic!(
                "shell 根节点应为 Column，实际 {:?}",
                std::mem::discriminant(&view)
            );
        };
        let has_reverse = style.as_ref().map_or(false, |s| {
            s.classes
                .iter()
                .any(|c| matches!(c, StyleClass::FlexColReverse))
        });
        assert!(
            has_reverse,
            "bottom 缺省根 col 应含 flex-col-reverse（翻转承载）"
        );
        let buttons = count_buttons(&view);
        // PLAN-012 W4：缺省 pinned 置空——任务栏常驻钮 = ⊞/切换器/布局×2/
        // 铃铛/齿轮/电源 = 7（pinned×3 退役 + 无注入窗口条目）。
        assert!(
            buttons >= 7,
            "任务栏常驻按钮群应在位（缺省无 pinned 后 ≥7），实得 {buttons}"
        );
        assert!(
            !children.is_empty(),
            "根 col 应有子（taskbar 块 + 让位 spacer）"
        );
        // 任务栏行（taskbar → View::Row）应带 h-14 等类（503 视觉规格；条件样式
        // border-t/b 经 extract_style_with 求值落位——S7 实拍回归锚）。
        let mut taskbar_row: Option<&V<crate::ui::interpreter::DynamicMessage>> = None;
        fn find_row_with_h12<'a>(
            v: &'a V<crate::ui::interpreter::DynamicMessage>,
            out: &mut Option<&'a V<crate::ui::interpreter::DynamicMessage>>,
        ) {
            if out.is_some() {
                return;
            }
            if let V::Row { style, .. } = v {
                if style.as_ref().map_or(false, |s| {
                    s.classes.iter().any(|c| {
                        matches!(c, crate::ui::style::StyleClass::Height(_) if format!("{c:?}").contains("14"))
                    })
                }) {
                    *out = Some(v);
                    return;
                }
            }
            for c in view_children(v) {
                find_row_with_h12(c, out);
            }
        }
        find_row_with_h12(&view, &mut taskbar_row);
        assert!(
            taskbar_row.is_some(),
            "taskbar 行应带高度类（条件样式求值落位）"
        );

        // top → 无翻转类（border-b 分支）。
        {
            let app = ds.apps.get_mut(&id).unwrap();
            let _ = app
                .component
                .write_state("__dock_position", auto_val::Value::str("top"));
        }
        let (view, _, _) = ds
            .apps
            .get(&id)
            .unwrap()
            .component
            .view_with_debug_gated(false);
        let V::Column { style, .. } = &view else {
            panic!("root Column")
        };
        let has_reverse = style.as_ref().map_or(false, |s| {
            s.classes
                .iter()
                .any(|c| matches!(c, StyleClass::FlexColReverse))
        });
        assert!(!has_reverse, "top 时根 col 不应含 flex-col-reverse");
    }

    /// Plan 505 D（债 P488-D4）：on_dnd_finished 交付锚定——发起时锚定
    /// 优先于完成时焦点（偏差场景：拖出后焦点漂移）；无锚回退焦点/primary
    /// （v1 语义保持）；锚定消费即取走（一次性）。
    #[test]
    fn dnd_finished_delivery_anchors_at_initiator() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let a = t3_add_win(&mut ds, "Alpha");
        let b = t3_add_win(&mut ds, "Beta");
        ds.wm_focus(b);
        let app_of = |ds: &crate::ui::session::DesktopSession, wid: crate::ui::session::Wid| {
            ds.host
                .as_ref()
                .unwrap()
                .wm
                .wins
                .get(&wid)
                .expect("vwin 在册")
                .app
        };
        let (app_a, app_b) = (app_of(&ds, a), app_of(&ds, b));

        // 代际不变（dispatch 内无拖出）→ 不锚定。
        let gen = dnd_drag_generation();
        maybe_anchor_dnd_initiator(&mut ds, app_a, gen, gen);
        assert!(ds.desktop.dnd_initiator.is_none(), "无拖出不锚定");

        // 代际变化 → 锚定发起方 a；交付目标 = a（焦点在 b 的偏差场景下
        // 仍交付发起方）。
        maybe_anchor_dnd_initiator(&mut ds, app_a, gen, gen + 1);
        assert_eq!(ds.desktop.dnd_initiator, Some(app_a));
        assert_eq!(dnd_finished_target(&mut ds), Some(app_a), "锚定优先于焦点");

        // 锚定取走后回退完成时焦点（v1 语义）。
        assert_eq!(dnd_finished_target(&mut ds), Some(app_b), "无锚回退焦点");
    }

    #[test]
    fn switcher_summon_advance_confirm_roundtrip() {
        let mut ds = t3_session_with_shell();
        let _a = t3_add_win(&mut ds, "Alpha");
        let b = t3_add_win(&mut ds, "Beta");
        // boot MRU 序：b（最近）, a。
        let _task = summon_switcher(&mut ds);
        assert!(ds.switcher_visible(), "召唤后 switcher visible");
        let sw = ds.desktop.switcher_app.expect("懒挂载完成");
        {
            let app = ds.apps.get(&sw).unwrap();
            let rows = app.component.read_state("rows").expect("rows 读回");
            let rows = match rows {
                auto_val::Value::Array(a) => a.values,
                auto_val::Value::VmRef(ref r) => t3_deref_list(app, r.id as u64, "rows"),
                auto_val::Value::Int(id) => t3_deref_list(app, id as u64, "rows"),
                other => panic!("rows 应为数组: {other:?}"),
            };
            assert_eq!(rows.len(), 2, "MRU 快照全量（RebuildMru 重建）");
            // handler 自建对象物化为 VmRef 堆对象（VM ObjectData）——解引用读字段。
            let auto_val::Value::VmRef(ref r0) = &rows[0] else {
                panic!("rows 条目应为 VmRef 对象: {:?}", rows[0])
            };
            let vm = app.component.bridge().vm();
            let heap_obj = vm.get_heap_object(r0.id as u64).expect("rows[0] 堆对象");
            let guard = heap_obj.read().unwrap();
            let od = guard
                .as_any()
                .downcast_ref::<auto_lang_core::vm::types::ObjectData>()
                .expect("rows[0] 为 ObjectData");
            assert_eq!(
                match od.get(&auto_val::ValueKey::from("wid".to_string())) {
                    Some(auto_val::Value::Str(s)) => s.to_string(),
                    other => panic!("wid 字段异常: {other:?}"),
                },
                b.0.to_string(),
                "rows 序 = MRU 序（front=最近聚焦）"
            );
        }
        // PLAN-014 W-10：RebuildMru 预选 MRU 第 2 项（nres>1 → sel=1，
        // Alt-Tab 惯例——第 1 项 = 当前聚焦窗）。
        {
            let app = ds.apps.get(&sw).unwrap();
            match app.component.read_state("sel") {
                Ok(auto_val::Value::Int(1)) => {}
                other => panic!("召唤后 sel 应预选 1（W-10）: {other:?}"),
            }
        }
        // Advance：sel 1→0 回绕（handler 直调——与宿主 Ctrl+Tab 直投
        // .Advance 消息同落同一 handler；DM::App 分派管线为 464 已证路径）。
        let app = ds.apps.get_mut(&sw).unwrap();
        app.component
            .bridge_mut()
            .call_handler("Advance", &[])
            .expect("Advance handler");
        match app.component.read_state("sel") {
            Ok(auto_val::Value::Int(0)) => {}
            other => panic!("Advance 后 sel 应回绕为 0: {other:?}"),
        }
        // confirm（Enter 同落 Focus handler）：写 focus 记录 + 自隐。
        app.component
            .bridge_mut()
            .call_handler("Focus", &[auto_val::Value::str(b.0.to_string())])
            .expect("Focus handler");
        match app.component.read_state("visible") {
            Ok(auto_val::Value::Str(ref s)) => assert_eq!(s.to_string(), "0", "confirm 后自隐"),
            other => panic!("visible 读回异常: {other:?}"),
        }
        // 上行记录可达宿主执行体。
        let cmds = ds.drain_app_desktop_commands(sw);
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::FocusWindow(b)],
            "confirm → focus 动词记录"
        );
    }

    // ---- Plan 479 T2：通知历史/未读/持久化/双面一体（施工图 §1）----
    // push_notification 系单测须先隔离 storage（STORAGE_MAP 进程级全局 +
    // storage_file() 按 cwd 哈希回退；nextest 每测独立进程 + 各自临时文件，
    // 互不污染、不落开发者共享库）。
    /// P487-2 债家族串行锁：隔离之外再加互斥——KD-048 raw localStorage
    /// 落盘修复后 raw 读点会 load 盘键，并行测试线程经共享 STORAGE_MAP
    /// 互串的窗口被放大（notif/dock/stdlib-storage 实测三红）。锁本体在
    /// stdlib（auto_lang_core::vm::ffi::stdlib::lock_storage_for_test），双方同锁。
    struct StorageTestIsolation(std::sync::MutexGuard<'static, ()>, std::path::PathBuf);
    impl std::ops::Deref for StorageTestIsolation {
        type Target = std::path::PathBuf;
        fn deref(&self) -> &Self::Target {
            &self.1
        }
    }
    impl AsRef<std::path::Path> for StorageTestIsolation {
        fn as_ref(&self) -> &std::path::Path {
            self.1.as_ref()
        }
    }
    fn t2_isolate_storage(tag: &str) -> StorageTestIsolation {
        let serial = auto_lang_core::vm::ffi::stdlib::lock_storage_for_test();
        let path = std::env::temp_dir().join(format!(
            "auto-479-notif-{}-{}.json",
            tag,
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("AUTO_VM_STORAGE_FILE", &path);
        // Plan 540 T2：桌面单源 config 文件同场隔离——DesktopState::new 的
        // DesktopConfig::load 读 env 指向的空文件（缺席 = 内置默认，且迁移
        // 写不落真实家目录）。
        let cfg_path =
            std::env::temp_dir().join(format!("auto-540-config-{}-{}.at", tag, std::process::id()));
        let _ = std::fs::remove_file(&cfg_path);
        std::env::set_var("AUTOOS_DESKTOP_CONFIG", &cfg_path);
        StorageTestIsolation(serial, path)
    }

    /// FIFO 容量与 MRU 序：front=最新、NOTES_CAP=50 环绕（最旧弹出）。
    #[test]
    fn notif_history_fifo_capacity_and_mru_order() {
        let _guard = t2_isolate_storage("fifo");
        let mut ds = t3_session_with_shell();
        push_notification(&mut ds, "success", "first");
        push_notification(&mut ds, "error", "second");
        push_notification(&mut ds, "info", "third");
        {
            let notes = ds.desktop.notifications.borrow();
            assert_eq!(notes.len(), 3);
            assert_eq!(notes[0].msg, "third", "front=最新（MRU 序）");
            assert_eq!(notes[2].msg, "first");
            assert_eq!(notes[0].kind, "info");
            assert_eq!(notes[0].at.len(), 5, "at = HH:MM 五字符");
            assert_eq!(&notes[0].at[2..3], ":");
            assert!(notes[0].id > notes[2].id, "id 单调递增");
        }
        for i in 0..60 {
            push_notification(&mut ds, "info", &format!("bulk{i}"));
        }
        let notes = ds.desktop.notifications.borrow();
        assert_eq!(notes.len(), 50, "NOTES_CAP=50 FIFO 环绕");
        assert_eq!(notes[0].msg, "bulk59");
        assert_eq!(notes[49].msg, "bulk10", "容量截断丢最旧");
    }

    /// 未读语义：面板关 +1 / 面板开不加 / 开面板清零 / boot 恢复后 0。
    #[test]
    fn notif_unread_semantics_panel_visibility() {
        let _guard = t2_isolate_storage("unread");
        let mut ds = t3_session_with_shell();
        push_notification(&mut ds, "success", "one");
        assert_eq!(ds.desktop.notes_unread.get(), 1, "面板缺席（不可见）+1");
        // 挂载可见探针 App（真面板 T3 落码；本测只消费 visible 判定位）。
        let probe_at = "widget NotifProbe {\n    model { var visible str = \"0\" }\n    view { if .visible == \"1\" { text \"on\" } }\n}\n";
        let comp = crate::build_dynamic_component(probe_at, None).unwrap();
        let probe = ds.allocate_app(comp);
        ds.desktop.notification_app = Some(probe);
        push_notification(&mut ds, "success", "two");
        assert_eq!(ds.desktop.notes_unread.get(), 2, "挂载但 visible=0 仍 +1");
        if let Some(app) = ds.apps.get_mut(&probe) {
            let _ = app
                .component
                .write_state("visible", auto_val::Value::str("1"));
        }
        assert!(ds.notification_visible(), "探针 visible 判定可达");
        push_notification(&mut ds, "success", "three");
        assert_eq!(ds.desktop.notes_unread.get(), 2, "面板可见时不加未读");
        // 模拟开面板清零（真执行体 T3 落码）。
        ds.desktop.notes_unread.set(0);
        assert_eq!(ds.desktop.notes_unread.get(), 0);
        // boot 恢复后未读恒 0（会话概念不落盘）。
        restore_notifications(&mut ds);
        assert_eq!(ds.desktop.notes_unread.get(), 0, "boot 恢复未读归零");
    }

    /// PLAN-012 W5：格子分配单测——positions 定位优先 + 未定位行主序
    /// 填充 + spacer 填位 + 平行列表同构（ids/cs/rs 等长且下标一致）。
    #[test]
    fn w5_desktop_icon_cells_assignment() {
        // 4 图标，第 2 枚定位 (3,0)，第 4 枚定位 (1,1)。
        let order = vec![
            ("a".to_string(), "pinned"),
            ("b".to_string(), "custom"),
            ("c".to_string(), "custom"),
            ("d".to_string(), "custom"),
        ];
        let _guard = t2_isolate_storage("w5-cells");
        let raw = "b=3:0,d=1:1,";
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.positions",
            raw.to_string(),
        );
        let pos = load_desktop_positions();
        // 2026-09-15：未定位图标改**列主序**填充（rows=8；纵向优先——左列
        // 自上而下占满再排下一列）。
        let (cells, ids, cs, rs) = desktop_icon_cells(&order, &[], &[], None, 8);
        eprintln!("[w5-cells] positions={pos:?} ids={ids:?} cs={cs:?} rs={rs:?}");
        // b 占 (3,0)、d 占 (1,1)。列主序填充：a→k=0→(0,0)，c→k=1→(0,1)
        // （线性格 8 未被占）。
        let id_of = |i: usize| -> String {
            match &cells[i] {
                auto_val::Value::Obj(o) => match o.get("id") {
                    Some(auto_val::Value::Str(s)) => s.to_string(),
                    _ => String::new(),
                },
                _ => String::new(),
            }
        };
        assert_eq!(cells.len(), ids.len(), "cells 与 ids 等长");
        assert_eq!(cs.len(), rs.len());
        assert_eq!(id_of(0), "a", "列主序首格 (0,0)");
        // 行主序输出：(0,1) = 线性下标 8。
        assert_eq!(id_of(8), "c", "列主序次格 (0,1) → 行主序下标 8");
        let cell_str = |v: &auto_val::Value| -> String {
            match v {
                auto_val::Value::Str(x) => x.to_string(),
                other => other.to_string(),
            }
        };
        // b 定位 (3,0) → 行主序下标 3；其前 (1,0)(2,0) 为 spacer。
        let bi = ids.iter().position(|x| x == "b").unwrap();
        assert_eq!(bi, 3, "b 在行主序下标 3");
        assert_eq!(cell_str(&cs[bi]), "3", "b 的列 = 定位值");
        assert_eq!(cell_str(&rs[bi]), "0", "b 的行 = 定位值");
        let spacer_at_1 = match &cells[1] {
            auto_val::Value::Obj(o) => o
                .get("spacer")
                .map(|v| matches!(v, auto_val::Value::Str(x) if x.to_string() == "1"))
                .unwrap_or(false),
            _ => false,
        };
        assert!(spacer_at_1, "(1,0) 应为 spacer 填位");
        let di = ids.iter().position(|x| x == "d").unwrap();
        assert_eq!(cell_str(&cs[di]), "1", "d 定位列");
        assert_eq!(cell_str(&rs[di]), "1", "d 定位行");
    }

    /// PLAN-012 W5：positions 追加式 csv last-wins 解析（同 id 后写胜）。
    #[test]
    fn w5_desktop_positions_last_wins() {
        let _guard = t2_isolate_storage("w5-positions");
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.positions",
            "a=0:0,b=1:0,a=2:3,".to_string(),
        );
        let pos = load_desktop_positions();
        assert_eq!(pos.len(), 2, "同 id 去重（last-wins）");
        assert_eq!(pos[0].0, "b");
        let a = pos.iter().find(|(id, _)| id == "a").unwrap();
        assert_eq!(a.1, (2, 3), "后写胜");
    }

    /// PLAN-012 W5：refresh_desktop_icons 无参动词解析。
    #[test]
    fn w5_refresh_desktop_icons_verb_parse() {
        assert_eq!(
            crate::ui::session::DesktopCommand::parse_records("refresh_desktop_icons"),
            vec![crate::ui::session::DesktopCommand::RefreshDesktopIcons]
        );
        assert_eq!(
            crate::ui::session::DesktopCommand::RefreshDesktopIcons.encode(),
            "refresh_desktop_icons"
        );
    }

    /// PLAN-012 W1：os-config close→hide 拦截臂单测——CloseWindow 对
    /// os-config 窗改写 hidden（窗存续、apps 保留、投影全排除），focus 臂
    /// unhide+聚焦（重开 = 纯聚焦臂）；非 os-config 窗照常回收。
    #[test]
    fn w1_osconfig_close_hides_and_focus_unhides() {
        let path = t2_isolate_storage("w1-close-hide");
        let mut ds = t540_resolver_session();

        // ① launch os-config → 窗在册。
        let wid = ds.launch_app(OSCONFIG_APP_ID).expect("os-config launch");
        assert!(ds.host.as_ref().unwrap().wm.wins.contains_key(&wid));

        // ② CloseWindow → hidden 置位，窗与 App 存续。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::CloseWindow(wid)],
        );
        let host = ds.host.as_ref().unwrap();
        let v = host
            .wm
            .wins
            .get(&wid)
            .expect("os-config 窗应存续（hide 非 close）");
        assert!(v.hidden.get(), "close 应改写为 hidden");
        assert!(ds.apps.values().any(|_| true), "apps 会话应保留");
        drop(host);

        // ③ 投影排除：__wm_wins 无条目 + __wm_running 不含 os-config
        //（W7 齿轮高亮回落，AC-12 语义自洽）。
        sync_shell_windows(&mut ds);
        let wins = t3_read_array(&ds, "__wm_wins");
        assert!(wins.is_empty(), "隐藏窗不应投影 __wm_wins: {wins:?}");
        match t3_read(&ds, "__wm_running") {
            auto_val::Value::Str(s) => assert!(
                !s.to_string().contains(",os-config,"),
                "隐藏窗不应入运行集: {}",
                s.to_string()
            ),
            other => panic!("__wm_running 读回异常: {other:?}"),
        }

        // ④ 重开（execute_open_settings 聚焦臂）→ unhide + 聚焦。
        execute_open_settings(&mut ds);
        let host = ds.host.as_ref().unwrap();
        let v = host.wm.wins.get(&wid).expect("重开不新开窗（存续）");
        assert!(!v.hidden.get(), "focus 臂应取消隐藏");
        assert_eq!(host.wm.focused, Some(wid), "重开应聚焦设置窗");
        assert_eq!(
            ds.desktop.config.dock_pinned.len(),
            0,
            "sanity：会话未被污染"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// PLAN-012 W1：非 os-config 窗 close 照常回收（拦截臂不误伤）。
    #[test]
    fn w1_non_osconfig_close_still_removes() {
        let _guard = t2_isolate_storage("w1-plain-close");
        let mut ds = t540_resolver_session();
        let wid = ds.launch_app("011-calculator").expect("calculator launch");
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::CloseWindow(wid)],
        );
        assert!(
            !ds.host.as_ref().unwrap().wm.wins.contains_key(&wid),
            "普通窗 close 应照常移除"
        );
    }

    /// PLAN-012 W8 DSL 级 fence：popover content 内 `for` + workspace_preview
    /// 必须物化为 WorkspacePreview 叶（VM 轨视图构建；切换器分区卡链）。
    #[test]
    fn w8_workspace_preview_materializes_in_popover_for() {
        let src = r#"widget ShellProbe {
    model {
        var __wm_workspaces = []
        var switcher_open str = ""
    }
    view {
        col {
            popover (open: .switcher_open == "1", placement: "top-end", ondismiss: .SwitcherToggle, class: "p-2 border rounded bg-card") {
                text `ANCHOR`
                row {
                    text `MARKER`
                    for ws in .__wm_workspaces {
                        col {
                            text ws.label
                            workspace_preview (ws: ws.id, fallback: "app-window") { style: "w-44 h-16 rounded-lg" }
                        }
                    }
                }
            }
        }
    }
    on {
        .SwitcherToggle -> {}
    }
}
"#;
        let mut comp = crate::build_dynamic_component(src, None).unwrap();
        // 注入两分区（sync 同构——宿主 write_state_vec）。
        let _ = comp.write_state_vec(
            "__wm_workspaces",
            vec![
                auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                    ("id", auto_val::Value::str("0")),
                    ("label", auto_val::Value::str("1")),
                    ("current", auto_val::Value::str("1")),
                ]))),
                auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                    ("id", auto_val::Value::str("1")),
                    ("label", auto_val::Value::str("2")),
                    ("current", auto_val::Value::str("")),
                ]))),
            ],
        );
        let (view, _, _) = comp.view_with_debug_gated(false);
        fn count(
            v: &AbstractView<crate::ui::interpreter::DynamicMessage>,
            wp: &mut usize,
            cards: &mut usize,
        ) {
            match v {
                AbstractView::WorkspacePreview { .. } => *wp += 1,
                AbstractView::Text { content, .. }
                    if content == "MARKER"
                        || content == "ANCHOR"
                        || content == "1"
                        || content == "2" =>
                {
                    *cards += 1
                }
                _ => {}
            }
            let mut kids = view_children(v);
            if let AbstractView::Popover {
                anchor, content, ..
            } = v
            {
                if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                    kids.push(w.as_ref());
                }
                kids.push(content.as_ref());
            }
            if let AbstractView::MouseArea { content, .. } = v {
                kids.push(content.as_ref());
            }
            for c in kids {
                count(c, wp, cards);
            }
        }
        let (mut wp, mut cards) = (0, 0);
        count(&view, &mut wp, &mut cards);
        eprintln!("[w8] previews={wp} cards={cards}");
        assert_eq!(cards, 4, "for 应物化两分区卡（含 MARKER/ANCHOR 锚点文本）");
        assert_eq!(wp, 2, "每卡一枚 workspace_preview 叶");
    }

    /// PLAN-012 W7 v1.6：`__wm_notes_visible` 投影——overlay 组件 visible
    /// 直读（"1"/""）写入 shell 状态 + 指纹 notes 段尾 `:v`（翻转即重写，
    /// 铃铛打开态高亮数据面）。
    #[test]
    fn projection_v16_notes_visible_fingerprint() {
        let mut ds = t3_session_with_shell();
        let probe_at = "widget NotifProbe {\n    model { var visible str = \"0\" }\n    view { if .visible == \"1\" { text \"on\" } }\n}\n";
        let comp = crate::build_dynamic_component(probe_at, None).unwrap();
        let probe = ds.allocate_app(comp);
        ds.desktop.notification_app = Some(probe);
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_notes_visible") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "", "面板隐藏 → 空串"),
            other => panic!("__wm_notes_visible 读回异常: {other:?}"),
        }
        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(
                s.to_string().contains("|notes:0:0:0:;"),
                "指纹 notes 段应含 :v 尾标（关）: {}",
                s.to_string()
            ),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
        // visible 置位 → 指纹变化触发重写 → 投影翻转（铃铛高亮数据面）。
        if let Some(app) = ds.apps.get_mut(&probe) {
            let _ = app
                .component
                .write_state("visible", auto_val::Value::str("1"));
        }
        sync_shell_windows(&mut ds);
        match t3_read(&ds, "__wm_notes_visible") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "1", "面板可见 → \"1\""),
            other => panic!("__wm_notes_visible 读回异常: {other:?}"),
        }
        match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => assert!(
                s.to_string().contains("|notes:0:0:0:1;"),
                "指纹 notes 段 :v 尾标应翻转: {}",
                s.to_string()
            ),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        }
    }

    /// 持久化槽 round-trip：写 10 槽（slot0=MRU front）→ 新会话 boot 读回；
    /// 坏槽跳过；超 10 条截断只落前 10。
    #[test]
    fn notif_storage_roundtrip_slots() {
        let path = t2_isolate_storage("roundtrip");
        let mut ds = t3_session_with_shell();
        for i in 0..12 {
            push_notification(&mut ds, "success", &format!("m{i}"));
        }
        // 槽位形态：slot0 = 最新，仅 10 槽落盘。
        let slot0 = auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.notes.0")
            .expect("slot0 已写");
        assert!(
            slot0.contains("\"msg\":\"m11\""),
            "slot0=MRU front: {slot0}"
        );
        assert!(slot0.contains("\"kind\":\"success\""));
        let slot11 = auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.notes.11");
        assert!(
            slot11.is_none() || slot11.as_deref() == Some(""),
            "仅 10 槽，无 slot11"
        );
        let slot9 = auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.notes.9")
            .expect("slot9 已写");
        assert!(
            slot9.contains("\"msg\":\"m2\""),
            "最旧落盘到 slot9: {slot9}"
        );
        // 新会话 boot 恢复（读侧）。
        let mut ds2 = t3_session_with_shell();
        restore_notifications(&mut ds2);
        {
            let notes = ds2.desktop.notifications.borrow();
            assert_eq!(notes.len(), 10, "落盘 10 条全恢复（12 条内存截断 10）");
            assert_eq!(notes[0].msg, "m11", "恢复序 = MRU 序");
            assert_eq!(notes[9].msg, "m2");
            assert_eq!(notes[0].kind, "success");
            assert!(notes[0].id > 0, "id 恢复保真");
        }
        let _ = std::fs::remove_file(&path);
    }

    /// push_notification 双面一体：单调用 = 入史 + toast + 未读三联动。
    #[test]
    fn notif_push_dual_face_history_and_toast() {
        let _guard = t2_isolate_storage("dual");
        let mut ds = t3_session_with_shell();
        push_notification(&mut ds, "error", "boom");
        {
            let notes = ds.desktop.notifications.borrow();
            assert_eq!(notes.len(), 1, "入史");
            assert_eq!(notes[0].kind, "error");
            assert_eq!(notes[0].msg, "boom");
        }
        let toasts = ds.desktop.toasts.borrow();
        assert_eq!(toasts.len(), 1, "同步浮现");
        assert_eq!(toasts[0].kind, "error");
        assert_eq!(toasts[0].msg, "boom");
        assert_eq!(ds.desktop.notes_unread.get(), 1, "未读联动");
    }

    // ---- Plan 479 T3：通知中心面板（summon/dismiss/clearAll 无头流；
    // switcher_summon_advance_confirm_roundtrip 同型。notification_center.at
    // 为进程内嵌资产——本测直载真源，非裁剪副本）----

    /// notes_toggle 召唤无头流：懒挂载（真 assets/notification_center.at）+
    /// 快照注入（rows 序 = 历史 MRU 序）+ 开面板未读清零 → Dismiss 写
    /// `notes_dismiss\t<id>` + ClearAll 写 `notes_clear` + Escape 自隐。
    #[test]
    fn notif_center_summon_headless() {
        let _guard = t2_isolate_storage("panel");
        let mut ds = t3_session_with_shell();
        assert!(ds.desktop.notification_app.is_none(), "未召唤不挂载");
        let _a = t3_add_win(&mut ds, "Alpha");
        // 两条通知入史（面板关 → 未读 2）。
        push_notification(&mut ds, "success", "welcome");
        push_notification(&mut ds, "error", "launch failed");
        assert_eq!(ds.desktop.notes_unread.get(), 2);
        // 召唤（NotesToggle 臂同执行体）。
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::NotesToggle],
        );
        assert!(ds.notification_visible(), "召唤后面板 visible");
        let panel = ds.desktop.notification_app.expect("懒挂载完成");
        assert_eq!(ds.desktop.notes_unread.get(), 0, "开面板未读清零");
        {
            let app = ds.apps.get(&panel).unwrap();
            let rows = app.component.read_state("rows").expect("rows 读回");
            let rows = match rows {
                auto_val::Value::Array(a) => a.values,
                auto_val::Value::VmRef(ref r) => t3_deref_list(app, r.id as u64, "rows"),
                auto_val::Value::Int(id) => t3_deref_list(app, id as u64, "rows"),
                other => panic!("rows 应为数组: {other:?}"),
            };
            assert_eq!(rows.len(), 2, "快照全量（RebuildNotes 重建）");
            let auto_val::Value::VmRef(ref r0) = &rows[0] else {
                panic!("rows[0] 应为 VmRef 对象: {:?}", rows[0])
            };
            let vm = app.component.bridge().vm();
            let heap_obj = vm.get_heap_object(r0.id as u64).expect("rows[0] 堆对象");
            let guard = heap_obj.read().unwrap();
            let od = guard
                .as_any()
                .downcast_ref::<auto_lang_core::vm::types::ObjectData>()
                .expect("rows[0] 为 ObjectData");
            assert_eq!(
                match od.get(&auto_val::ValueKey::from("msg".to_string())) {
                    Some(auto_val::Value::Str(s)) => s.to_string(),
                    other => panic!("msg 字段异常: {other:?}"),
                },
                "launch failed",
                "rows 序 = 历史 MRU 序（front=最新）"
            );
        }
        // 逐条 ×：Dismiss(id) 写上行记录 + 可达宿主执行体（历史删除）。
        let target = {
            let notes = ds.desktop.notifications.borrow();
            notes.iter().find(|n| n.msg == "launch failed").unwrap().id
        };
        let app = ds.apps.get_mut(&panel).unwrap();
        app.component
            .bridge_mut()
            .call_handler("Dismiss", &[auto_val::Value::str(target.to_string())])
            .expect("Dismiss handler");
        let cmds = ds.drain_app_desktop_commands(panel);
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::NotesDismiss(target)],
            "逐条 × → notes_dismiss 动词记录"
        );
        let (_, _tasks) = execute_desktop_commands(&mut ds, cmds);
        assert_eq!(ds.desktop.notifications.borrow().len(), 1, "宿主臂删除");
        // 全部清除：ClearAll 写 notes_clear + 执行后历史空。
        let app = ds.apps.get_mut(&panel).unwrap();
        app.component
            .bridge_mut()
            .call_handler("ClearAll", &[])
            .expect("ClearAll handler");
        let cmds = ds.drain_app_desktop_commands(panel);
        assert_eq!(cmds, vec![crate::ui::session::DesktopCommand::NotesClear]);
        let (_, _tasks) = execute_desktop_commands(&mut ds, cmds);
        assert!(ds.desktop.notifications.borrow().is_empty(), "全部清除");
        // Esc 自隐。
        let app = ds.apps.get_mut(&panel).unwrap();
        app.component
            .bridge_mut()
            .call_handler("Escape", &[])
            .expect("Escape handler");
        assert!(!ds.notification_visible(), "Esc 后自隐");
    }

    /// Plan 479 T4：投影 v1.2——`__wm_notes`（历史 Obj 数组 {id,kind,msg,at}）
    /// + `__wm_notes_unread`（串）+ 指纹 notes 段 `|notes:{len}:{front}:{unread};`
    /// 门控翻转（T1 定案 6）。
    #[test]
    fn notif_projection_notes_and_fingerprint() {
        let _guard = t2_isolate_storage("proj");
        let mut ds = t3_session_with_shell();
        let _a = t3_add_win(&mut ds, "Alpha");
        sync_shell_windows(&mut ds);
        assert_eq!(
            t3_read(&ds, "__wm_notes_unread"),
            auto_val::Value::str("0"),
            "零历史初值（宿主已同步——.at model 缺省空串不出 badge）"
        );
        assert!(t3_read_array(&ds, "__wm_notes").is_empty());
        // 入史两条 → 投影全量 + 未读串 + 指纹段。
        push_notification(&mut ds, "success", "first");
        push_notification(&mut ds, "error", "second");
        sync_shell_windows(&mut ds);
        let notes = t3_read_array(&ds, "__wm_notes");
        assert_eq!(notes.len(), 2, "__wm_notes 全量投影");
        let auto_val::Value::Obj(front) = &notes[0] else {
            panic!("notes 条目应为 Obj")
        };
        assert_eq!(t3_obj_str(front, "msg"), "second", "MRU 序 front=最新");
        assert_eq!(t3_obj_str(front, "kind"), "error");
        assert_eq!(t3_obj_str(front, "at").len(), 5, "at = HH:MM");
        assert_eq!(t3_read(&ds, "__wm_notes_unread"), auto_val::Value::str("2"));
        let fp = match t3_read(&ds, "__wm_fp") {
            auto_val::Value::Str(s) => s.to_string(),
            other => panic!("__wm_fp 读回异常: {other:?}"),
        };
        let front_id = match &notes[0] {
            auto_val::Value::Obj(o) => t3_obj_str(o, "id"),
            other => panic!("Obj expected: {other:?}"),
        };
        assert!(
            // PLAN-012 W7 v1.6：notes 段尾扩 `:v`（可见性指纹位，空 = 隐藏）。
            fp.contains(&format!("|notes:2:{front_id}:2:;")),
            "指纹 notes 段: {fp}"
        );
        // 开面板清零 → 指纹翻 → 未读串归 0（同 len 同 front，仅 unread 段变）。
        ds.desktop.notes_unread.set(0);
        sync_shell_windows(&mut ds);
        assert_eq!(t3_read(&ds, "__wm_notes_unread"), auto_val::Value::str("0"));
        // 无变化 → 指纹门控跳过（幂等）。
        let fp_before = t3_read(&ds, "__wm_fp").clone();
        sync_shell_windows(&mut ds);
        assert_eq!(t3_read(&ds, "__wm_fp"), fp_before, "指纹未变零写");
    }

    /// Plan 479 T4：真 assets/shell.at 冒烟——铃铛钮/badge 视图编译（构建
    /// 即检语法）+ NotificationToggle → `notes_toggle` 动词记录接线。
    #[test]
    fn notif_shell_at_smoke_toggle_and_badge() {
        let mut ds = t3_session_with_shell();
        // 换装真 shell 资产（t3_session_with_shell 挂的是裁剪探针）。
        let shell = ds.desktop.shell_app.expect("probe shell");
        let real =
            crate::ui::shell::build_shell_component().expect("真 shell.at 编译（铃铛+badge 语法）");
        ds.apps.remove(&shell);
        ds.desktop.shell_app = Some(ds.allocate_app(real));
        // badge 面：宿主写未读串（写状态不触发 handler——与生产投影一致）。
        let shell = ds.desktop.shell_app.expect("real shell");
        if let Some(app) = ds.apps.get_mut(&shell) {
            let _ = app
                .component
                .write_state("__wm_notes_unread", auto_val::Value::str("3"));
        }
        // 铃铛面：NotificationToggle handler → notes_toggle 记录可达宿主。
        let app = ds.apps.get_mut(&shell).unwrap();
        app.component
            .bridge_mut()
            .call_handler("NotificationToggle", &[])
            .expect("NotificationToggle handler");
        let cmds = ds.drain_desktop_commands();
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::NotesToggle],
            "铃铛钮 → notes_toggle 动词"
        );
    }

    /// Plan 479 T5：无头端到端——notify → 投影 badge → notes_toggle 开面板
    /// （未读清零）→ Dismiss 逐条删 → 落盘断言 → 新会话 boot 重读（接线 =
    /// boot Desktop 分支 dock_edges 邻位调 restore_notifications，实机链路
    /// T6 验收）。
    #[test]
    fn notif_end_to_end_toggle_dismiss_restore() {
        let path = t2_isolate_storage("e2e");
        let mut ds = t3_session_with_shell();
        let _a = t3_add_win(&mut ds, "Alpha");
        // ① notify：notify 动词 → 入史 + 未读（面板关）。
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::Notify(
                "success".to_string(),
                "hello world".to_string(),
            )],
        );
        assert_eq!(ds.desktop.notifications.borrow().len(), 1, "notify 入史");
        assert_eq!(ds.desktop.notes_unread.get(), 1);
        // ② badge：投影同步 → 未读串可达 shell。
        sync_shell_windows(&mut ds);
        assert_eq!(t3_read(&ds, "__wm_notes_unread"), auto_val::Value::str("1"));
        // ③ 开面板：notes_toggle → visible + 未读清零。
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::NotesToggle],
        );
        assert!(ds.notification_visible());
        assert_eq!(ds.desktop.notes_unread.get(), 0, "开面板清零");
        sync_shell_windows(&mut ds);
        assert_eq!(t3_read(&ds, "__wm_notes_unread"), auto_val::Value::str("0"));
        // ④ 逐条 ×：面板 Dismiss → 上行 → 宿主臂删除 + 落盘。
        let target = ds.desktop.notifications.borrow()[0].id;
        let panel = ds.desktop.notification_app.expect("面板已挂载");
        let app = ds.apps.get_mut(&panel).unwrap();
        app.component
            .bridge_mut()
            .call_handler("Dismiss", &[auto_val::Value::str(target.to_string())])
            .expect("Dismiss handler");
        let cmds = ds.drain_app_desktop_commands(panel);
        let (_, _tasks) = execute_desktop_commands(&mut ds, cmds);
        assert!(
            ds.desktop.notifications.borrow().is_empty(),
            "dismiss 后历史空"
        );
        // ⑤ 落盘断言：slot0 = 空串（全量重写语义）。
        assert_eq!(
            auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.notes.0").as_deref(),
            Some(""),
            "空历史落盘为空槽"
        );
        // ⑥ 重读：新会话 restore（boot 同函数）→ 空历史，无 panic。
        let mut ds2 = t3_session_with_shell();
        restore_notifications(&mut ds2);
        assert!(ds2.desktop.notifications.borrow().is_empty());
        let _ = std::fs::remove_file(&path);
    }

    // ---- Plan 478 T6：宿主臂无头覆盖（注入通道受限项的 headless 指针，
    // 472 T5 先例成文）——WorkspaceAdd 增分区即入 / WorkspaceClose toast 门
    // 与末分区保底 / SendTo 跨区发送。----
    #[test]
    fn workspace_v11_host_arms_add_close_send() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha"); // ws0，焦点窗
                                              // `+`：增分区并即入新分区（验收②）。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::WorkspaceAdd]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.workspaces.len(), 3);
            assert_eq!(host.wm.current_workspace, 2, "增分区即入新分区");
            assert_eq!(host.wm.wins[&a].workspace, 0, "原窗归属不变");
        }
        // `send_to`：a 发往当前分区 2（发前先聚焦 a）= 恒等迁移（焦点保持）。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::SendTo(a, 2)]);
        assert!(!exit);
        assert_eq!(ds.host.as_ref().unwrap().wm.wins[&a].workspace, 2);
        ds.wm_focus(a);
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::SendTo(a, 2)]);
        assert!(!exit);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.focused,
            Some(a),
            "当前分区发送焦点保持"
        );
        // `×`：分区 2 含窗 → toast 提示不删（T1 定案：toast 门最少意外）。
        let toasts_before = ds.desktop.toasts.borrow().len();
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(2)]);
        assert!(!exit);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.workspaces.len(),
            3,
            "非空分区不删"
        );
        assert!(
            ds.desktop.toasts.borrow().len() > toasts_before,
            "非空 × 出 toast 提示"
        );
        // send_to 隐分区：a 发回分区 0 → 焦点让渡（分区 2 空 → None）。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::SendTo(a, 0)]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.wins[&a].workspace, 0);
            assert_eq!(host.wm.focused, None, "隐分区发送焦点让渡");
        }
        // `×`：分区 2 已空 → 删除；current clamp 2→1。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(2)]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.workspaces.len(), 2);
            assert_eq!(host.wm.current_workspace, 1, "current clamp");
        }
        // `×`：删到单分区再删 → 末分区保底 no-op + toast。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(1)]);
        assert!(!exit);
        let toasts_mid = ds.desktop.toasts.borrow().len();
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(0)]);
        assert!(!exit);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.workspaces.len(),
            1,
            "末分区保底"
        );
        assert!(
            ds.desktop.toasts.borrow().len() > toasts_mid,
            "末分区 × 出 toast 提示"
        );
    }

    // ---- PLAN-019 v1.7：负一屏显示桌面（show_desktop/showdesk_return +
    // 排除规则 + __wm_showdesk 投影）----

    /// 负一屏状态机：懒建保留分区、origin 记录/消费、幂等不覆盖、
    /// showdesk_return 顺带关 picker。
    #[test]
    fn showdesk_state_machine_roundtrip() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha"); // ws0
        assert!(!ds.host.as_ref().unwrap().wm.on_showdesk());

        // 首次 show_desktop：懒建保留分区（pack 默认 2 分区 → 下标 2），
        // origin = 0，窗口归属不变。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.showdesk_ws, Some(2), "懒建保留分区");
            assert_eq!(host.wm.showdesk_origin, Some(0), "origin 记录");
            assert_eq!(host.wm.current_workspace, 2, "切入负一屏");
            assert!(host.wm.on_showdesk());
            assert_eq!(host.wm.wins[&a].workspace, 0, "原窗归属不变");
        }

        // 幂等：负一屏重复到达不覆盖 origin（wallpaper_pick 归属规则判据）。
        ds.host.as_mut().unwrap().wm.showdesk_origin = Some(1);
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        assert!(!exit);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.showdesk_origin,
            Some(1),
            "重复 show_desktop 不覆盖 origin"
        );

        // picker 开着 return：先关 picker（簿记清零）再回 origin。
        {
            let host = ds.host.as_mut().unwrap();
            host.wm.picker_open = true;
            host.wm.picker_return_on_close = true;
        }
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::ShowdeskReturn]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(!host.wm.picker_open, "return 顺带关 picker");
            assert!(!host.wm.picker_return_on_close);
            assert_eq!(host.wm.current_workspace, 1, "返回记录到的 origin");
            assert_eq!(host.wm.showdesk_origin, None, "origin 消费清零");
            assert!(!host.wm.on_showdesk());
        }

        // 无簿记 return = no-op（已不在负一屏）。
        let (exit, _) = execute_desktop_commands(&mut ds, vec![DC::ShowdeskReturn]);
        assert!(!exit);
        assert_eq!(ds.host.as_ref().unwrap().wm.current_workspace, 1);
    }

    /// PLAN-709 rev2：win_rect 执行臂——rect 直写 + min 钳制 + window_size
    /// 同步；未知 wid / 槽位伪 Wid no-op；workspace/焦点不动。
    #[test]
    fn win_rect_execution_moves_clamps_and_noops() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha");
        // 常规摆位：立即生效（移动+缩放一体）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![DC::WinRect {
                wid: a,
                x: 40.0,
                y: 30.0,
                w: 500.0,
                h: 400.0,
            }],
        );
        {
            let host = ds.host.as_ref().unwrap();
            let r = *host.wm.wins[&a].rect.borrow();
            assert_eq!((r.x, r.y, r.width, r.height), (40.0, 30.0, 500.0, 400.0));
            let sz = *host.wm.wins[&a].window_size.borrow();
            assert_eq!((sz.width, sz.height), (500.0, 400.0));
        }
        // min 钳制（160/120 地板）；不夺焦点。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![DC::WinRect {
                wid: a,
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 5.0,
            }],
        );
        {
            let host = ds.host.as_ref().unwrap();
            let r = *host.wm.wins[&a].rect.borrow();
            assert_eq!((r.width, r.height), (160.0, 120.0));
            assert_eq!(host.wm.focused, Some(a), "win_rect 不夺焦点");
        }
        // 未知 wid no-op；槽位伪 Wid no-op（几何归交互状态机）。
        let before = *ds.host.as_ref().unwrap().wm.wins[&a].rect.borrow();
        let _ = execute_desktop_commands(
            &mut ds,
            vec![
                DC::WinRect {
                    wid: crate::ui::session::Wid(999),
                    x: 1.0,
                    y: 1.0,
                    w: 10.0,
                    h: 10.0,
                },
                DC::WinRect {
                    wid: crate::ui::session::WmState::native_slot_pseudo_wid(
                        crate::ui::native_dock::NativeSlotId(7),
                    ),
                    x: 1.0,
                    y: 1.0,
                    w: 10.0,
                    h: 10.0,
                },
            ],
        );
        assert_eq!(*ds.host.as_ref().unwrap().wm.wins[&a].rect.borrow(), before);
    }

    /// 排除规则：环切跳过负一屏、删除/发送拒绝（簿记压实跟随）、
    /// 负一屏上 activate 先回 origin 再启动。
    #[test]
    fn showdesk_exclusion_rules() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let a = t3_add_win(&mut ds, "Alpha"); // ws0
        let _ = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        assert_eq!(ds.host.as_ref().unwrap().wm.current_workspace, 2);

        // 环切跳过负一屏：2 → next 越过 2 落 0；0 → prev 越过 2 落 1。
        ds.wm_next_workspace();
        assert_eq!(
            ds.host.as_ref().unwrap().wm.current_workspace,
            0,
            "next 跳过负一屏"
        );
        ds.wm_prev_workspace();
        assert_eq!(
            ds.host.as_ref().unwrap().wm.current_workspace,
            1,
            "prev 跳过负一屏"
        );

        // 删除拒绝：负一屏 no-op（无 toast——守卫先于保底门）；簿记压实
        // 跟随：删分区 0 → 负一屏下标 2→1、origin 0→0（并入 target 0）。
        let toasts_before = ds.desktop.toasts.borrow().len();
        let _ = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(2)]);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.workspaces.len(),
            3,
            "负一屏不可删"
        );
        assert_eq!(
            ds.desktop.toasts.borrow().len(),
            toasts_before,
            "负一屏删除静默 no-op"
        );
        // 簿记压实跟随：删空分区 1（含窗分区被既有门拦，非本测对象）→
        // 负一屏下标 2→1、origin（0）不变。
        let _ = execute_desktop_commands(&mut ds, vec![DC::WorkspaceClose(1)]);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.workspaces.len(), 2);
            assert_eq!(host.wm.showdesk_ws, Some(1), "簿记下标压实跟随");
            assert_eq!(
                host.wm.showdesk_origin,
                Some(0),
                "origin 不受低下标外删除影响"
            );
        }

        // 发送拒绝：窗口不可发往负一屏（现下标 1）。
        ds.wm_focus(a);
        let _ = execute_desktop_commands(&mut ds, vec![DC::SendTo(a, 1)]);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.wins[&a].workspace,
            0,
            "send_to 负一屏拒绝"
        );

        // 负一屏上 activate = 先回 origin 再启动（新窗落 origin 分区）。
        ds.desktop.app_resolver = Some(std::sync::Arc::new(|name: &str| {
            (name == "011-calculator").then(|| crate::ui::session::LaunchSpec {
                media_root: None,
                photo_root: None,
                back_entry: None,

                code: T3_WIN_AT.to_string(),
                source_path: None,
                title: Some("calculator".to_string()),
                name: None,
                daemon: None,
                back_root: None,
                fit: false,
                ..Default::default()
            })
        }));
        let _ = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        assert!(ds.host.as_ref().unwrap().wm.on_showdesk());
        let (exit, _) =
            execute_desktop_commands(&mut ds, vec![DC::ActivateApp("011-calculator".to_string())]);
        assert!(!exit);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(!host.wm.on_showdesk(), "activate 先回 origin");
            let wid = host.wm.focused.expect("activate 后有焦点窗");
            assert_eq!(
                host.wm.wins[&wid].registry_id.as_deref(),
                Some("011-calculator")
            );
            assert_eq!(
                host.wm.wins[&wid].workspace, host.wm.current_workspace,
                "新窗落返回后的分区"
            );
        }
    }

    /// 投影：负一屏不入 `__wm_workspaces`；`__wm_showdesk` 随进出翻转。
    #[test]
    fn showdesk_projection_filter_and_flag() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let _ = t3_add_win(&mut ds, "Alpha");
        let _ = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        sync_shell_windows(&mut ds);

        let wss = t3_read_array(&ds, "__wm_workspaces");
        assert_eq!(wss.len(), 2, "负一屏被投影过滤（pack 默认 2 分区）");
        match t3_read(&ds, "__wm_showdesk") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "1", "负一屏态标量 = 1"),
            other => panic!("__wm_showdesk 读回异常: {other:?}"),
        }

        let _ = execute_desktop_commands(&mut ds, vec![DC::ShowdeskReturn]);
        sync_shell_windows(&mut ds);
        let wss = t3_read_array(&ds, "__wm_workspaces");
        assert_eq!(wss.len(), 2);
        match t3_read(&ds, "__wm_showdesk") {
            auto_val::Value::Str(s) => assert_eq!(s.to_string(), "", "返回后标量清零"),
            other => panic!("__wm_showdesk 读回异常: {other:?}"),
        }
    }

    /// 组合簿记归属规则（AC-03/04）：他分区进入 pick → return_on_close=1
    /// （close 自动回 origin）；负一屏自入 pick → return_on_close=0
    /// （close 只关不切回）。
    #[test]
    fn wallpaper_pick_close_return_ownership() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let _ = t3_add_win(&mut ds, "Alpha");

        // 他分区进入：pick = 组合调用（origin 记录 + 代管返回）。
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperPick]);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(host.wm.picker_open, "pick 开 picker");
            assert!(host.wm.picker_return_on_close, "组合调用 = 自动返回");
            assert_eq!(host.wm.showdesk_origin, Some(0), "origin 记录");
            assert!(host.wm.on_showdesk(), "切到负一屏");
        }
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperClose]);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(!host.wm.picker_open, "close 关 picker");
            assert_eq!(
                host.wm.current_workspace, 0,
                "return_on_close → 自动回 origin"
            );
            assert!(!host.wm.on_showdesk());
        }

        // 负一屏自入：sliver 进（show_desktop）→ pick = 不代管返回。
        let _ = execute_desktop_commands(&mut ds, vec![DC::ShowDesktop]);
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperPick]);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(host.wm.picker_open);
            assert!(!host.wm.picker_return_on_close, "自入 = 不代管返回");
        }
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperClose]);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(!host.wm.picker_open);
            assert!(host.wm.on_showdesk(), "自入关闭留在负一屏（AC-04）");
        }
    }

    /// PLAN-019-FU8：os-config 设置面「切换壁纸」请求通道——config 差分
    /// wallpaper_request 值变化(非空) = wallpaper_pick 组合（同
    /// DC::WallpaperPick：切负一屏 + picker 开 + 代管返回）；同值重放不
    /// 触发；交替新值再触发。
    #[test]
    fn wallpaper_request_diff_triggers_pick() {
        use crate::ui::session::DesktopCommand as DC;
        let mut ds = t3_session_with_shell();
        let _ = t3_add_win(&mut ds, "Alpha");

        // 变化 + 非空 → 组合触发。
        let mut cfg = ds.desktop.config.clone();
        cfg.wallpaper_request = "1".to_string();
        apply_external_config_diff(&mut ds, &cfg);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(host.wm.picker_open, "request 变化 → picker 开");
            assert!(host.wm.picker_return_on_close, "组合语义 = 代管返回");
            assert!(host.wm.on_showdesk(), "切到负一屏");
        }

        // 同值重放（宿主已记账）→ 无副作用。
        let same = cfg.clone();
        apply_external_config_diff(&mut ds, &same);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(host.wm.picker_open);
            assert!(host.wm.on_showdesk());
        }

        // 关闭后同值仍不触发；交替新值再触发。
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperClose]);
        assert!(!ds.host.as_ref().unwrap().wm.picker_open);
        let same2 = cfg.clone();
        apply_external_config_diff(&mut ds, &same2);
        assert!(!ds.host.as_ref().unwrap().wm.picker_open, "同值不触发");
        cfg.wallpaper_request = "2".to_string();
        apply_external_config_diff(&mut ds, &cfg);
        assert!(ds.host.as_ref().unwrap().wm.picker_open, "交替新值再触发");
    }

    /// 导航数学（宿主收口）：栅格态 flip 轮换并应用（游标环绕）；预览态
    /// 游标环绕不触发应用；picker 关 = no-op。config 落盘经
    /// AUTOOS_DESKTOP_CONFIG 隔离（nextest 每测独立进程，env 无竞争）。
    #[test]
    fn wallpaper_nav_cursor_and_flip() {
        use crate::ui::session::DesktopCommand as DC;
        let tmp = std::env::temp_dir().join(format!(
            "plan019-nav-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::env::set_var("AUTOOS_DESKTOP_CONFIG", tmp.join("config.at"));
        let mut ds = t3_session_with_shell();
        {
            let host = ds.host.as_mut().unwrap();
            host.wm.picker_open = true;
            // 7 枚候选 > 5 枚窗口：FU7 滑窗语义可测。
            host.wm.picker_paths = vec![
                "a.jpg".into(),
                "b.jpg".into(),
                "c.jpg".into(),
                "d.jpg".into(),
                "e.jpg".into(),
                "f.jpg".into(),
                "g.jpg".into(),
            ];
            host.wm.picker_win = 0;
        }

        // 栅格态 next：滑窗推进（FU7 起导航不应用——点选才应用）。
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.picker_win, 1, "滑窗推进");
            assert_eq!(ds.desktop.config.wallpaper_path, "", "导航不应用");
        }
        // 上边界 clamp：推到 len-5=2 后不再前进。
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        assert_eq!(ds.host.as_ref().unwrap().wm.picker_win, 2, "滑窗右界 clamp");
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("prev".into())]);
        assert_eq!(ds.host.as_ref().unwrap().wm.picker_win, 1, "prev 回退");

        // 预览态：游标环绕移动，不触发应用。
        ds.host.as_mut().unwrap().wm.picker_preview = Some(2);
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.picker_preview, Some(3), "预览游标推进");
        }
        // 环绕：尾 → next → 0。
        ds.host.as_mut().unwrap().wm.picker_preview = Some(6);
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.picker_preview,
            Some(0),
            "预览尾环绕回首"
        );

        // picker 关 = no-op。
        ds.host.as_mut().unwrap().wm.picker_open = false;
        let _ = execute_desktop_commands(&mut ds, vec![DC::WallpaperNav("next".into())]);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.picker_preview,
            Some(0),
            "关态导航 no-op"
        );
    }

    /// Esc 链 update 臂（AC-05）：预览态 → 回栅格；栅格态 → 关闭（归属
    /// 簿记决定是否返回）。picker_key_message 纯函数键面映射。
    #[test]
    fn wallpaper_escape_chain_and_keymap() {
        use crate::ui::session::DesktopEvent as DE;
        use iced::keyboard::{key::Named, Key};

        // 键面映射：←/→/Esc 命中，其他键穿透。
        assert!(matches!(
            picker_key_message(&Key::Named(Named::ArrowLeft)),
            Some(crate::ui::session::DesktopMessage::Desktop(
                DE::WallpaperKeyNav("prev")
            ))
        ));
        assert!(matches!(
            picker_key_message(&Key::Named(Named::ArrowRight)),
            Some(crate::ui::session::DesktopMessage::Desktop(
                DE::WallpaperKeyNav("next")
            ))
        ));
        assert!(matches!(
            picker_key_message(&Key::Named(Named::Escape)),
            Some(crate::ui::session::DesktopMessage::Desktop(
                DE::WallpaperKeyEscape
            ))
        ));
        assert!(picker_key_message(&Key::Named(Named::Enter)).is_none());

        // Esc 链：预览态回栅格（picker 保持开）。
        let mut ds = t3_session_with_shell();
        ds.host.as_mut().unwrap().wm.picker_open = true;
        ds.host.as_mut().unwrap().wm.picker_preview = Some(1);
        execute_wallpaper_escape(&mut ds);
        {
            let host = ds.host.as_ref().unwrap();
            assert!(host.wm.picker_open, "预览 Esc 只回栅格");
            assert_eq!(host.wm.picker_preview, None, "预览态清零");
        }
        // 栅格态 Esc：关闭 + return_on_close → 回 origin。
        ds.host.as_mut().unwrap().wm.picker_return_on_close = true;
        execute_wallpaper_escape(&mut ds);
        let host = ds.host.as_ref().unwrap();
        assert!(!host.wm.picker_open, "栅格 Esc 关闭 picker");
        assert!(!host.wm.on_showdesk(), "return_on_close → 自动返回");
    }

    /// PLAN-019 T-06：布局键——归一化（`\`→`/` + ASCII 小写折叠）稳定、
    /// 异路径异键；#hex/builtin/空 无布局语境（None）；csv roundtrip。
    #[test]
    fn wallpaper_layout_key_and_csv_roundtrip() {
        let a1 = wallpaper_layout_key("D:\\Down\\stella-os\\Wallpapers\\raiden.JPG");
        let a2 = wallpaper_layout_key("d:/down/stella-os/wallpapers/raiden.jpg");
        assert_eq!(a1, a2, "大小写/分隔符归一");
        assert!(a1.starts_with("shell.desktop.positions.wp/"));
        assert_ne!(
            a1,
            wallpaper_layout_key("d:/down/stella-os/wallpapers/room.jpg"),
            "异路径异键"
        );
        let positions = vec![
            ("011-calculator".to_string(), (1usize, 2usize)),
            ("015-notes".to_string(), (0usize, 0usize)),
        ];
        let back = parse_positions_csv(&serialize_positions(&positions));
        assert_eq!(back, positions, "csv roundtrip");
        // 无布局语境壁纸。
        assert!(current_wallpaper_layout_key_for("#243b55").is_none());
        assert!(current_wallpaper_layout_key_for("builtin:ricepaper").is_none());
        assert!(current_wallpaper_layout_key_for("").is_none());
        assert!(current_wallpaper_layout_key_for("D:/x/wp.jpg").is_some());
    }

    fn current_wallpaper_layout_key_for(wp: &str) -> Option<String> {
        if wp.is_empty() || wp.starts_with('#') || wp.starts_with("builtin:") {
            return None;
        }
        Some(wallpaper_layout_key(wp))
    }

    /// PLAN-019 T-06：布局随壁纸迁移（AC-08）——壁纸 A 键控摆布与壁纸 B
    /// 互不覆盖；切回 A 布局恢复；缺省底稿回退（新壁纸首次继承默认）。
    #[test]
    fn wallpaper_layout_memory_migration() {
        let _guard = t2_isolate_storage("wp-layout");
        let mut ds = t3_session_with_shell();
        // 壁纸 A/B 布局键（真文件不必在——键控层与 storage 层直驱）。
        let key_a = wallpaper_layout_key("D:/wp/a.jpg");
        let key_b = wallpaper_layout_key("D:/wp/b.jpg");
        // A：calc 摆 (0,0)；B：calc 摆 (3,3)。
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            &key_a,
            "011-calculator=0:0,".to_string(),
        );
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            &key_b,
            "011-calculator=3:3,".to_string(),
        );
        // 键控读各自命中。
        let la = load_desktop_positions_for(Some(&key_a));
        let lb = load_desktop_positions_for(Some(&key_b));
        assert_eq!(la, vec![("011-calculator".to_string(), (0usize, 0usize))]);
        assert_eq!(lb, vec![("011-calculator".to_string(), (3usize, 3usize))]);
        // C 键缺席 → 回退缺省底稿（首次切到该壁纸继承默认摆布）。
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.positions",
            "011-calculator=5:1,".to_string(),
        );
        let key_c = wallpaper_layout_key("D:/wp/c.jpg");
        let lc = load_desktop_positions_for(Some(&key_c));
        assert_eq!(
            lc,
            vec![("011-calculator".to_string(), (5usize, 1usize))],
            "缺席键回退缺省底稿"
        );
        // 迁移快照语义：effective 布局写回键（即使缺席键也固化当前摆布）。
        let snapshot = load_desktop_positions_for(Some(&key_c));
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            &key_c,
            serialize_positions(&snapshot),
        );
        assert_eq!(load_desktop_positions_for(Some(&key_c)), lc, "快照幂等固化");
    }

    // ---- Plan 472 T4：dock 升级（activate 执行体 + 配置边距 + 资产装载）----

    /// activate：未运行 → launch（带 registry_id 回填）；运行中 → 聚焦不新增窗。
    #[test]
    fn activate_app_focuses_running_or_launches_new() {
        let mut ds = t3_session_with_shell();
        ds.desktop.app_resolver = Some(std::sync::Arc::new(|name: &str| {
            (name == "011-calculator").then(|| crate::ui::session::LaunchSpec {
                media_root: None,
                photo_root: None,
                back_entry: None,

                code: T3_WIN_AT.to_string(),
                source_path: None,
                title: Some("calculator".to_string()),
                name: None,
                daemon: None,
                back_root: None,
                fit: false,
                exe: None,
                opens: Vec::new(),
                render_decl: None,
            })
        }));
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::ActivateApp(
                "011-calculator".to_string(),
            )],
        );
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.wins.len(), 1, "未运行 activate → 启动新窗");
        assert!(
            host.wm
                .wins
                .values()
                .all(|v| v.registry_id.as_deref() == Some("011-calculator")),
            "launch 路径回填 registry_id"
        );
        let wid = host.wm.focused.unwrap();
        // 已运行 → 聚焦原窗，不新增。
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::ActivateApp(
                "011-calculator".to_string(),
            )],
        );
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.wins.len(), 1, "运行中 activate 不新增窗");
        assert_eq!(host.wm.focused, Some(wid));
    }

    /// activate：目标窗在隐藏分区 → 先切分区再聚焦（dock 点击跨分区召唤）。
    #[test]
    fn activate_app_switches_to_hidden_workspace() {
        let mut ds = t3_session_with_shell();
        ds.desktop.app_resolver = Some(std::sync::Arc::new(|name: &str| {
            (name == "011-calculator").then(|| crate::ui::session::LaunchSpec {
                media_root: None,
                photo_root: None,
                back_entry: None,

                code: T3_WIN_AT.to_string(),
                source_path: None,
                title: Some("calculator".to_string()),
                name: None,
                daemon: None,
                back_root: None,
                fit: false,
                exe: None,
                opens: Vec::new(),
                render_decl: None,
            })
        }));
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::ActivateApp(
                "011-calculator".to_string(),
            )],
        );
        let wid = ds.host.as_ref().unwrap().wm.focused.unwrap();
        let ws1 = ds.host.as_mut().unwrap().wm.add_workspace();
        ds.wm_set_workspace(ws1);
        let (_, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::ActivateApp(
                "011-calculator".to_string(),
            )],
        );
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.current_workspace, 0, "activate 切回窗所在分区");
        assert_eq!(host.wm.focused, Some(wid));
    }

    /// dock 数据级配置 → 布局预留边（协议 v1 §6：缺席回退 pack 默认
    /// bottom/48；top 反转；enabled=false 全零）。
    /// Plan 540 T2：读源切单源 config——直接构造 DesktopConfig 断言
    /// （storage 全局态不再参与，隔离不再需要）。
    #[test]
    fn desktop_dock_edges_reads_config_overrides() {
        use crate::ui::desktop_config::DesktopConfig;

        // 缺省 → pack 默认（bottom 48）。
        let e = desktop_dock_edges(&DesktopConfig::default());
        assert_eq!(e.bottom, crate::ui::layout::TASKBAR_HEIGHT);
        assert_eq!(e.top, 0.0);

        // position=top → 预留翻转到顶。
        let cfg = DesktopConfig {
            dock_position: "top".to_string(),
            ..DesktopConfig::default()
        };
        let e = desktop_dock_edges(&cfg);
        assert_eq!(e.top, crate::ui::layout::TASKBAR_HEIGHT);
        assert_eq!(e.bottom, 0.0);

        // enabled=false → dock 关，零预留。
        let cfg = DesktopConfig {
            dock_enabled: false,
            ..DesktopConfig::default()
        };
        let e = desktop_dock_edges(&cfg);
        assert_eq!(e.bottom, 0.0);
        assert_eq!(e.top, 0.0);
    }

    /// Plan 494：真洞模式位 boot 读入——`shell.native.hole`=="true" 开，
    /// 缺席/坏值 off（隔离同 489-dock-edges 型：storage 全局态防跨测污染）。
    #[test]
    fn native_hole_mode_reads_storage_key() {
        let _store = t2_isolate_storage("494-hole-mode");
        let key = "shell.native.hole";
        auto_lang_core::vm::ffi::stdlib::storage_raw_remove(key);
        // 缺省 off。
        assert!(!load_native_hole_mode());
        // 坏值 off。
        auto_lang_core::vm::ffi::stdlib::storage_raw_set(key.into(), "yes".into());
        assert!(!load_native_hole_mode());
        // "true" 开。
        auto_lang_core::vm::ffi::stdlib::storage_raw_set(key.into(), "true".into());
        assert!(load_native_hole_mode());
        auto_lang_core::vm::ffi::stdlib::storage_raw_remove(key);
        let _ = std::fs::remove_file(&_store);
    }

    /// Plan 494 验收 3：Region 失败回退路径实测一次——stale hwnd 驱动
    /// `apply_hole_regions` Err → hole_mode 翻 off + 假洞 z 重申路径执行
    ///（无 slot 时重申空转，仅断言模式翻转与日志路径可达）。
    // PLAN-018 T-03：test-native-dock 门退役——真窗口 dock 族保持显式手动
    // 准入（--ignored 运行），不进日常/集成档（原条件等价转写）。
    #[cfg(all(windows, test))]
    #[test]
    #[ignore = "PLAN-018: 真窗口 dock 交互族（原 test-native-dock 手动门），显式 --ignored 运行"]
    fn hole_region_fallback_flips_mode_off() {
        use crate::ui::native_dock::win32 as ndw;
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        ds.desktop.hole_mode = true;
        // 真 scratch 窗创建后销毁 → stale hwnd（alive 检查必 Err，回退分支
        // 全程真实执行）。
        let scratch = ndw::test_support::spawn(
            "e2e-fallback-dead",
            crate::ui::native_dock::Rect::new(60, 60, 200, 120),
        );
        let dead = scratch.0;
        drop(scratch);
        let frame = crate::ui::native_dock::Rect::new(0, 0, 800, 600);
        refresh_hole_regions_at(&mut ds, dead, frame);
        assert!(
            !ds.desktop.hole_mode,
            "Region 失败应自动回退 off（假洞语义）"
        );
    }

    /// Plan 487 M4：set_dock_position/enabled 执行臂——storage 键写回 +
    /// dock_edges 键重推导（boot `desktop_dock_edges` 同函数，I9 单一事实）
    /// + relayout 全窗写回（Grid 窗几何随预留边翻转）+ shell.at 投影热同步
    ///（`__dock_*` 直写——Init 只跑一次）。
    #[test]
    fn settings_dock_arms_hot_apply_and_persist() {
        // 存储隔离（t2 同型）：执行臂 storage_host_publish 走落盘链——必须
        // 指向临时文件，防跨进程污染真实 store（首跑教训：默认 store 残键
        // 打破 summon 测试的键缺席前提）。
        let path = t2_isolate_storage("487-dock-arms");
        let mut ds = t3_session_with_shell();
        // 换装真 shell（探针无 __dock_* 声明——投影同步断言需要）。
        let probe = ds.desktop.shell_app.expect("probe shell");
        let real = crate::ui::shell::build_shell_component().expect("真 shell.at 编译");
        ds.apps.remove(&probe);
        ds.desktop.shell_app = Some(ds.allocate_app(real));
        // Grid 布局 + 一窗：relayout 可观测（Grid 单窗满铺可用区，y 随边走）。
        ds.wm_set_layout(crate::ui::layout::LayoutMode::Grid);
        let wid = t3_add_win(&mut ds, "Alpha");
        let shell = ds.desktop.shell_app.expect("real shell");

        // ① set_dock_position(top)：单源 config 写回 + 边翻转 + 投影 +
        // relayout（Plan 540 T3：storage 键写退役）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockPosition(true)],
        );
        assert_eq!(
            ds.desktop.config.dock_position, "top",
            "位置单源 config 写回"
        );
        assert_eq!(ds.desktop.dock_edges.top, crate::ui::layout::TASKBAR_HEIGHT);
        assert_eq!(ds.desktop.dock_edges.bottom, 0.0);
        {
            let app = ds.apps.get(&shell).unwrap();
            match app.component.read_state("__dock_position") {
                Ok(auto_val::Value::Str(ref s)) => {
                    assert_eq!(s.to_string(), "top", "shell 投影热同步")
                }
                other => panic!("__dock_position 读回异常: {other:?}"),
            }
        }
        {
            let host = ds.host.as_ref().unwrap();
            let rect = *host.wm.wins.get(&wid).unwrap().rect.borrow();
            assert_eq!(
                rect.y,
                crate::ui::layout::TASKBAR_HEIGHT,
                "Grid 窗 y=top 预留"
            );
        }

        // ② set_dock_enabled(false)：零预留 + config false + 投影关 + 窗回满铺。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockEnabled(false)],
        );
        assert!(!ds.desktop.config.dock_enabled, "开关单源 config 写回");
        assert_eq!(ds.desktop.dock_edges.top, 0.0);
        assert_eq!(ds.desktop.dock_edges.bottom, 0.0);
        {
            let app = ds.apps.get(&shell).unwrap();
            match app.component.read_state("__dock_enabled") {
                Ok(auto_val::Value::Str(ref s)) => {
                    assert_eq!(s.to_string(), "0", "shell 开关投影关")
                }
                other => panic!("__dock_enabled 读回异常: {other:?}"),
            }
        }
        {
            let host = ds.host.as_ref().unwrap();
            let rect = *host.wm.wins.get(&wid).unwrap().rect.borrow();
            assert_eq!(rect.y, 0.0, "dock 关 → 可用区回满铺");
        }

        // ③ 重开：位置键保留——按 top 恢复（enabled 只决定零/非零）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockEnabled(true)],
        );
        assert_eq!(ds.desktop.dock_edges.top, crate::ui::layout::TASKBAR_HEIGHT);
        assert_eq!(ds.desktop.dock_edges.bottom, 0.0, "按保留位置键恢复 top");

        let _ = std::fs::remove_file(&path);
    }

    /// 资产 shell.at（widget Desktop）装载冒烟：编译 + fire_init 读 storage
    /// 缺席回退 pack 默认（enabled=1 / position=bottom）；pinned 由宿主
    /// 解析注入（{id,icon} Obj 数组，icon 自注册表）。
    /// Plan 489 merge 补隔离（同 desktop_dock_edges…——实机 store 键打破
    /// 缺席前提，P487-2 同族）。
    #[test]
    fn desktop_shell_at_builds_with_dock_defaults() {
        let _store = t2_isolate_storage("489-shell-defaults");
        let comp = crate::ui::shell::build_shell_component().expect("shell.at 装载");
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let id = ds.allocate_app(comp);
        ds.desktop.shell_app = Some(id);
        // 宿主侧 pinned 解析注入（pack 默认表 + 注册表图标）。
        ds.desktop.registry_entries = vec![crate::ui::app_registry::AppRegistryEntry {
            media_root: None,
            photo_root: None,
            back_entry: None,

            id: "011-calculator".to_string(),
            title: "calculator".to_string(),
            title_zh: None,
            name: None,
            icon: "calculator".to_string(),
            category: "tool".to_string(),
            entry: std::path::PathBuf::from("011-calculator"),
            render: "vm".to_string(),
            daemon: None,
            back_root: None,
            fit: false,
            desktop_visible: true,
            desktop_exe: None,
            desktop_render: None,
            opens: Vec::new(),
        }];
        inject_dock_pinned(&mut ds);
        {
            let app = ds.apps.get(&id).unwrap();
            match app.component.read_state("__dock_enabled") {
                Ok(auto_val::Value::Str(ref s)) => assert_eq!(s.to_string(), "1"),
                other => panic!("__dock_enabled 读回异常: {other:?}"),
            }
            match app.component.read_state("__dock_position") {
                Ok(auto_val::Value::Str(ref s)) => assert_eq!(s.to_string(), "bottom"),
                other => panic!("__dock_position 读回异常: {other:?}"),
            }
        }
        // PLAN-012 W4：缺省 pinned 置空（缺省三枚退役——缺键 = 显式空 =
        // 空表；用户经右键固定后此处按序出现）。
        let pinned = t3_read_array(&ds, "__dock_pinned");
        assert!(
            pinned.is_empty(),
            "pack 默认 pinned 应为空（W4 置空），实得 {pinned:?}"
        );

        // Plan 478 T5：pager 升格——v1.1 投影形状（含 label/current）注入 +
        // 新消息臂写总线记录。PLAN-014 W-01：workspace_close 臂随死组退役
        // （pager × 按钮已删，pack 无发送者），fixture 仅保 workspace_add。
        let app = ds.apps.get_mut(&id).unwrap();
        let _ = app.component.write_state_vec(
            "__wm_workspaces",
            vec![
                auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                    ("id", auto_val::Value::Str("0".into())),
                    ("name", auto_val::Value::Str("Desktop 1".into())),
                    ("current", auto_val::Value::Str("1".into())),
                    ("label", auto_val::Value::Str("1".into())),
                ]))),
                auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                    ("id", auto_val::Value::Str("1".into())),
                    ("name", auto_val::Value::Str("Desktop 2".into())),
                    ("current", auto_val::Value::Str("".into())),
                    ("label", auto_val::Value::Str("2".into())),
                ]))),
            ],
        );
        app.component
            .bridge_mut()
            .call_handler("WorkspaceAdd", &[])
            .expect("WorkspaceAdd handler");
        match app.component.read_state("__desktop_cmd") {
            Ok(auto_val::Value::Str(ref s)) => {
                assert_eq!(
                    s.to_string(),
                    "workspace_add",
                    "pager + 写 workspace_add 记录"
                )
            }
            other => panic!("__desktop_cmd 读回异常: {other:?}"),
        }
        let _ = std::fs::remove_file(&_store);
    }

    /// Plan 488 T2：on_native_drop 载荷字段形状——text/files/image
    /// whichever（缺省 Null），含屏幕坐标与观察格式表。
    #[test]
    fn plan488_native_drop_payload_record() {
        let data = crate::ui::native_dnd::NativeDropData {
            text: Some("hi".into()),
            files: vec!["C:/a.txt".into()],
            image: None,
            screen_x: 12,
            screen_y: 34,
            formats: vec!["CF_HDROP".into(), "CF_UNICODETEXT".into()],
        };
        let fields = native_drop_record_fields(&data);
        let get = |k: &str| fields.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        assert_eq!(
            get("text"),
            Some(&crate::ui::vm_bridge::RecordValue::Str("hi".into()))
        );
        assert_eq!(
            get("screen_x"),
            Some(&crate::ui::vm_bridge::RecordValue::Int(12))
        );
        assert_eq!(
            get("screen_y"),
            Some(&crate::ui::vm_bridge::RecordValue::Int(34))
        );
        assert_eq!(
            get("files"),
            Some(&crate::ui::vm_bridge::RecordValue::StrList(vec![
                "C:/a.txt".into()
            ]))
        );
        // image 缺省空串（空串哨兵——VM 侧 record 字段判 nil 比较不稳）。
        assert_eq!(
            get("image_path"),
            Some(&crate::ui::vm_bridge::RecordValue::Str(String::new()))
        );
    }

    /// Plan 488 T7：桌面级 Ctrl+V 注入面——OS 剪贴板 → on_native_paste
    /// 载荷字段。文本往返 + files/image 域形状（485 测试锁串行化，防
    /// nextest 并行进程互清剪贴板）。
    #[test]
    #[cfg(windows)]
    fn plan488_clipboard_paste_payload_reads_os_clipboard() {
        let _lock = crate::ui::clipboard_native::GlobalClipboardTestLock::acquire();
        crate::ui::clipboard::clipboard_set("paste-me");
        let fields = clipboard_paste_record_fields();
        let get = |k: &str| fields.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        assert_eq!(
            get("text"),
            Some(&crate::ui::vm_bridge::RecordValue::Str("paste-me".into()))
        );
        assert!(
            matches!(
                get("files"),
                Some(crate::ui::vm_bridge::RecordValue::StrList(_))
            ),
            "files 域恒为列表"
        );
        assert!(
            get("image_path").is_some(),
            "image_path 域恒存在（空串或有值）"
        );
    }

    /// Plan 486 T3：真 assets/shell.at native 条目——v1.3 投影形状注入
    /// （App 条目 native 空串 + native 条目五字段集）→ view if 分支过编译
    /// 语法门 + NativeFocus/NativeClose 消息臂写总线记录（wid 直传
    /// "N<slot>"，宿主 parse 剥前缀归一——动词派发全链接线）。
    #[test]
    fn native_dock_shell_at_entries_and_verbs() {
        let comp = crate::ui::shell::build_shell_component().expect("shell.at 装载");
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let id = ds.allocate_app(comp);
        ds.desktop.shell_app = Some(id);
        {
            let app = ds.apps.get_mut(&id).unwrap();
            let _ = app.component.write_state_vec(
                "__wm_wins",
                vec![
                    auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                        ("wid", auto_val::Value::Str("1".into())),
                        ("title", auto_val::Value::Str("Alpha".into())),
                        ("focused", auto_val::Value::Str("1".into())),
                        ("workspace", auto_val::Value::Str("0".into())),
                        ("app", auto_val::Value::Str("".into())),
                        ("icon", auto_val::Value::Str("app-window".into())),
                        ("native", auto_val::Value::Str("".into())),
                    ]))),
                    auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                        ("wid", auto_val::Value::Str("N3".into())),
                        ("title", auto_val::Value::Str("记事本".into())),
                        ("focused", auto_val::Value::Str("".into())),
                        ("native", auto_val::Value::Str("1".into())),
                        ("icon", auto_val::Value::Str("app-window".into())),
                    ]))),
                ],
            );
        }
        let app = ds.apps.get_mut(&id).unwrap();
        app.component
            .bridge_mut()
            .call_handler("NativeFocus", &[auto_val::Value::str("N3")])
            .expect("NativeFocus handler");
        // 总线单记录覆盖语义（一 update 周期一事件）——逐事件排空断言。
        let cmds = ds.drain_desktop_commands();
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::FocusNative(3)],
            "native 条目点击 → focus_native（N 前缀归一）"
        );
        let app = ds.apps.get_mut(&id).unwrap();
        app.component
            .bridge_mut()
            .call_handler("NativeClose", &[auto_val::Value::str("N3")])
            .expect("NativeClose handler");
        let cmds = ds.drain_desktop_commands();
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::CloseNative(3)],
            "native 条目 × → close_native（N 前缀归一）"
        );
    }
    /// Plan 487 M4 步骤7 + Plan 540 T7 → Plan 551 换向：真 assets/shell.at
    /// 齿轮冒烟——OpenSettingsPanel handler → `open_settings` 记录 → 联合
    /// 排空 → os-config 窗 launch（launch-or-focus 语义；probe 注入保持
    /// ensure 无头化）。
    #[test]
    fn settings_shell_at_smoke_gear_to_panel() {
        let path = t2_isolate_storage("487-gear");
        let mut ds = t540_resolver_session();
        ds.desktop.osconfig_daemon_probe = Some(std::sync::Arc::new(|| {
            crate::ui::osconfig_daemon::DaemonStatus::Running(
                crate::ui::osconfig_daemon::default_daemon_url(),
            )
        }));
        // 换装真 shell 资产（t3_session_with_shell 挂的是裁剪探针）。
        let probe = ds.desktop.shell_app.expect("probe shell");
        let real = crate::ui::shell::build_shell_component().expect("真 shell.at 编译（齿轮语法）");
        ds.apps.remove(&probe);
        ds.desktop.shell_app = Some(ds.allocate_app(real));
        let shell = ds.desktop.shell_app.expect("real shell");
        // 齿轮面：OpenSettingsPanel handler → open_settings 记录可达宿主。
        let app = ds.apps.get_mut(&shell).unwrap();
        app.component
            .bridge_mut()
            .call_handler("OpenSettingsPanel", &[])
            .expect("OpenSettingsPanel handler");
        let cmds = ds.drain_desktop_commands();
        assert_eq!(
            cmds,
            vec![crate::ui::session::DesktopCommand::OpenSettings],
            "齿轮钮 → open_settings 动词"
        );
        // 排空执行 → os-config 窗 launch（registry_id 定位；页面契约归
        // 兄弟仓自测——桩只证窗语义）。
        let _ = execute_desktop_commands(&mut ds, cmds);
        let panel = t540_settings_app(&ds);
        assert!(ds.apps.get(&panel).is_some(), "os-config 窗在位");

        let _ = std::fs::remove_file(&path);
    }

    /// Plan 487 M4 步骤6 → Plan 551 改驾：通知门控消费链——SetNotesEnabled
    /// 执行臂 config 落盘 + 479 notify 全链路短路/恢复。045 面板 handler
    /// 驱动随窗退役（写 emitter 迁 os-config daemon 单源 + T6 热应用）——
    /// 动词臂直驱（宿主能力面保留）。
    #[test]
    fn settings_notes_gate_and_about_section() {
        let path = t2_isolate_storage("487-notes");
        let mut ds = t540_resolver_session();
        ds.desktop.notifications.borrow_mut().clear();
        ds.desktop.notes_unread.set(0);

        // ① 关：SetNotesEnabled(false) → config 落盘。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetNotesEnabled(false)],
        );
        assert!(!ds.desktop.config.notes_enabled, "开关单源 config 落盘");
        // ② 门控：notify 动词全链路短路（零入史/零未读）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::Notify(
                "success".to_string(),
                "muted".to_string(),
            )],
        );
        assert!(
            ds.desktop.notifications.borrow().is_empty(),
            "关 → notify 短路"
        );
        assert_eq!(ds.desktop.notes_unread.get(), 0);
        // ③ 恢复：SetNotesEnabled(true) → notify 入史 + 未读。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetNotesEnabled(true)],
        );
        assert!(ds.desktop.config.notes_enabled);
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::Notify(
                "success".to_string(),
                "audible".to_string(),
            )],
        );
        assert_eq!(
            ds.desktop.notifications.borrow().len(),
            1,
            "开 → notify 入史"
        );
        assert_eq!(ds.desktop.notes_unread.get(), 1);

        let _ = std::fs::remove_file(&path);
    }

    /// Plan 487 M4 步骤5 + Plan 540 T3/T7 → Plan 551 改驾：Dock 动词臂
    /// 直驱（045 面板 handler 随窗退役）——SetDockPosition/SetDockEnabled/
    /// SetDockPinned 执行：热生效（edges 翻转）+ config 落盘 + 会话域
    /// pinned 同步。PLAN-012 W4：空 csv = 空表（缺省三枚回退退役）+
    /// DockPin/DockUnpin 单枚增删去重（幂等重复调用无副作用）。
    #[test]
    fn settings_dock_section_dispatch_and_pinned_storage() {
        let path = t2_isolate_storage("487-dock-section");
        let mut ds = t540_resolver_session();

        // ① SetDockPosition(top) → 热生效（edges 翻转 + config 落盘）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockPosition(true)],
        );
        assert_eq!(ds.desktop.dock_edges.top, crate::ui::layout::TASKBAR_HEIGHT);

        // ② SetDockEnabled(false) → 全零边。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockEnabled(false)],
        );
        assert_eq!(ds.desktop.dock_edges.top, 0.0);
        assert_eq!(ds.desktop.dock_edges.bottom, 0.0);

        // ③ SetDockPinned csv → config 落盘 + 会话域同步。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockPinned(
                "011-calculator,013-todo,015-notes,020-newapp".to_string(),
            )],
        );
        assert_eq!(
            ds.desktop.config.dock_pinned,
            vec![
                "011-calculator".to_string(),
                "013-todo".to_string(),
                "015-notes".to_string(),
                "020-newapp".to_string()
            ],
            "set_dock_pinned 单源 config 落盘"
        );
        assert_eq!(
            ds.desktop.dock_pinned, ds.desktop.config.dock_pinned,
            "会话域 pinned 同步"
        );
        // ④ PLAN-012 W4：空值 = 空表（缺省三枚回退语义退役——显式空即空）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::SetDockPinned(
                String::new(),
            )],
        );
        assert!(ds.desktop.config.dock_pinned.is_empty(), "空 csv → 空表");

        // ⑤ PLAN-012 W4 协议 v1.6：dock_pin/dock_unpin 单枚增删去重 +
        // 落盘 + 会话域同步（幂等：重复 pin/unpin 不在表即无操作）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::DockPin(
                "011-calculator".to_string(),
            )],
        );
        assert_eq!(
            ds.desktop.config.dock_pinned,
            vec!["011-calculator".to_string()],
            "dock_pin 入表"
        );
        let _ = execute_desktop_commands(
            &mut ds,
            vec![
                crate::ui::session::DesktopCommand::DockPin("013-todo".to_string()),
                crate::ui::session::DesktopCommand::DockPin("011-calculator".to_string()),
                crate::ui::session::DesktopCommand::DockUnpin("038-minesweeper".to_string()),
            ],
        );
        assert_eq!(
            ds.desktop.config.dock_pinned,
            vec!["011-calculator".to_string(), "013-todo".to_string()],
            "重复 pin 去重、unpin 不在表幂等"
        );
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::DockUnpin(
                "011-calculator".to_string(),
            )],
        );
        assert_eq!(
            ds.desktop.config.dock_pinned,
            vec!["013-todo".to_string()],
            "dock_unpin 出表"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// Plan 487 M4 步骤4 + Plan 540 T7 → Plan 551 换向：⚙️ 打开 =
    /// launch-or-focus os-config（已开聚焦不开新窗）+ pac `daemon: autoos`
    /// launch 期 ensure（probe 注入无头化，501 注入位）。cfg_*/徽标播种
    /// 契约随 045 设置窗退役——配置读写单源走 daemon（os-config 仓自测），
    /// 宿主侧外写热应用见 T6 轮询链测试。
    #[test]
    fn settings_panel_summon_headless() {
        let path = t2_isolate_storage("487-summon");
        let mut ds = t540_resolver_session();
        // ⓪ daemon ensure 无头化：probe 注入（真 ensure_ready 会探网/spawn）。
        ds.desktop.osconfig_daemon_probe = Some(std::sync::Arc::new(|| {
            crate::ui::osconfig_daemon::DaemonStatus::Running(
                crate::ui::osconfig_daemon::default_daemon_url(),
            )
        }));
        // ① 开：launch os-config 桩 + ensure（probe → Running → env 注入）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::OpenSettings],
        );
        let panel = t540_settings_app(&ds);
        assert!(
            matches!(
                ds.desktop.osconfig_status,
                crate::ui::osconfig_daemon::DaemonStatus::Running(_)
            ),
            "pac daemon: autoos → launch 期 ensure 探活（probe → Running）"
        );
        // ② 再召唤：launch-or-focus——不开新窗（AppId 不变、恰一窗）。
        let _ = execute_desktop_commands(
            &mut ds,
            vec![crate::ui::session::DesktopCommand::OpenSettings],
        );
        let panel2 = t540_settings_app(&ds);
        assert_eq!(panel, panel2, "已开再召唤 = 聚焦（AppId 不变）");
        let settings_wins = ds
            .host
            .as_ref()
            .map(|h| {
                h.wm.wins
                    .iter()
                    .filter(|(_, v)| v.registry_id.as_deref() == Some(OSCONFIG_APP_ID))
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(settings_wins, 1, "恰好一 os-config 窗（无重复 launch）");

        let _ = std::fs::remove_file(&path);
    }
    /// Plan 551 T6:外写热应用轮询——首采样不应用/外写差异应用(主题+
    /// pinned+透明度+壁纸目录)/宿主自写后内容相等早退(防回环)。
    #[test]
    fn external_config_poll_hot_apply_loopsafe() {
        let path = t2_isolate_storage("551-poll");
        let mut ds = t3_session_with_shell();
        // ① 首采样:只建哨兵锚不应用(文件尚未存在,mtime 锚 = None)。
        poll_external_config(&mut ds);
        assert!(ds.desktop.config_poll_sampled, "首采样落哨兵");
        assert!(ds.desktop.config_poll_mtime.is_none(), "文件缺席锚 None");
        // ② 外写(模拟 daemon PUT):多字段一次落盘 → 下一 tick 差异应用。
        // PLAN-615 T-06:主题面外写须伴 theme_source="manual"——system 源
        // (缺省)在 load() 期从 OS 派生 dark_theme,外写值会被跟随语义覆盖。
        let mut cfg = ds.desktop.config.clone();
        cfg.dark_theme = !cfg.dark_theme;
        cfg.theme_source = "manual".to_string();
        cfg.dock_pinned = vec!["011-calculator".to_string()];
        cfg.transparency = "high".to_string();
        cfg.wallpapers_dir = r"D:\wallpapers".to_string();
        let cfg_path = crate::ui::desktop_config::desktop_config_path().unwrap();
        crate::ui::desktop_config::save_to(&cfg_path, &cfg);
        poll_external_config(&mut ds);
        assert_eq!(ds.desktop.config.dark_theme, cfg.dark_theme, "主题热应用");
        assert_eq!(
            ds.desktop.config.dock_pinned, cfg.dock_pinned,
            "pinned 热应用"
        );
        assert_eq!(ds.desktop.dock_pinned, cfg.dock_pinned, "会话域同步");
        assert_eq!(ds.desktop.config.transparency, "high", "透明度面同步");
        assert_eq!(
            ds.desktop.config.wallpapers_dir, r"D:\wallpapers",
            "壁纸目录面同步"
        );
        // ③ 防回环:宿主自写(save)后 poll → mtime 变但内容相等早退。
        // PLAN-668 R-06：②应用后 theme_source 归一化回 "system"，若照原样
        // 自写，load() 会按跟随语义从 OS 派生 dark_theme——OS=light 机器
        // 必红（P667 期"单跑绿"实为 OS=dark 的巧合）。本段断言的是防回环
        // 非主题跟随语义，自写前显式 manual 源锚定存储值。
        ds.desktop.config.theme_source = "manual".to_string();
        crate::ui::desktop_config::save(&ds.desktop.config);
        let snap = ds.desktop.config.clone();
        poll_external_config(&mut ds);
        assert_eq!(ds.desktop.config, snap, "内容相等早退(无回环)");

        let _ = std::fs::remove_file(&path);
    }
    /// PLAN-601 R1（复审）:boot 读回——config.at 持久 theme_name 在
    /// open_desktop 激活。此前仅动词臂/外写 diff 臂会 set_theme,重启后
    /// 主题不存活(轮询首采样只锚不应用,551 语义对结构化消费面成立、
    /// 对需 ACTIVE_THEME 激活的主题面不成立)。未知名容错=保持缺省。
    #[test]
    fn desktop_boot_activates_persisted_theme_name() {
        let _guard = t2_isolate_storage("601-boot-theme");
        let cfg_path = crate::ui::desktop_config::desktop_config_path().unwrap();

        // ① 未知名容错:set_theme 拒绝 → 保持缺省(负例先行,线程内
        // ACTIVE_THEME 尚未被本测激活过)。
        let mut bad = crate::ui::desktop_config::DesktopConfig::default();
        bad.theme_name = Some("nonsense".to_string());
        let _ = crate::ui::desktop_config::save_to(&cfg_path, &bad);
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        assert_eq!(
            crate::ui::style::theme::theme_name(),
            "scaffold",
            "未知名拒绝保持轨缺省（PLAN-619 T-03：scaffold）"
        );

        // ② boot:载入结构体≠激活(锚定语义,此时仍缺省);
        // open_desktop 激活 → 主题重启存活。
        let mut cfg = crate::ui::desktop_config::DesktopConfig::default();
        cfg.theme_name = Some("zinc".to_string());
        let _ = crate::ui::desktop_config::save_to(&cfg_path, &cfg);
        let mut ds2 = crate::ui::session::DesktopSession::__test_session();
        assert_eq!(
            crate::ui::style::theme::theme_name(),
            "scaffold",
            "载入结构体≠激活(首采样锚定语义)"
        );
        ds2.open_desktop(iced::window::Id::unique());
        assert_eq!(
            crate::ui::style::theme::theme_name(),
            "zinc",
            "boot 读回激活(主题重启存活)"
        );

        let _ = std::fs::remove_file(&cfg_path);
    }
    /// Plan 487 M4 步骤1：资产 settings.at（widget Settings）装载冒烟——编译 +
    /// Init 默认（visible=0/section=dock）+ Nav 分区切换 + pinned 平行列表
    /// 注入 → RebuildPinned 重建 rows（B12 规避形态）。控件→动词接线在
    /// 执行步骤 5/6 扩测。
    // ---- Plan 496 M5：桌面本体数据链（T2——storage 往返 + 合并去重）----

    /// T2 族读回辅助：desktop_app 面的标量/数组状态（t3_read* 绑 shell_app，
    /// 本族面不同——独立小助手）。
    fn t496_read(ds: &crate::ui::session::DesktopSession, field: &str) -> auto_val::Value {
        let surface = ds.desktop.desktop_app.unwrap();
        ds.apps
            .get(&surface)
            .unwrap()
            .component
            .read_state(field)
            .unwrap_or_else(|e| panic!("{field} 读回失败: {e}"))
    }

    fn t496_read_array(
        ds: &crate::ui::session::DesktopSession,
        field: &str,
    ) -> Vec<auto_val::Value> {
        let surface = ds.desktop.desktop_app.unwrap();
        let app = ds.apps.get(&surface).unwrap();
        let val = app
            .component
            .read_state(field)
            .unwrap_or_else(|e| panic!("{field} 读回失败: {e}"));
        match val {
            auto_val::Value::Array(a) => a.values,
            auto_val::Value::VmRef(r) => t3_deref_list(app, r.id as u64, field),
            auto_val::Value::Int(id) => t3_deref_list(app, id as u64, field),
            other => panic!("{field} 既非数组也非列表引用: {other:?}"),
        }
    }

    fn t496_val_obj(v: &auto_val::Value) -> &auto_val::Obj {
        match v {
            auto_val::Value::Obj(o) => o,
            other => panic!("条目应为 Obj: {other:?}"),
        }
    }

    /// Plan 518 G4④：图标命名覆盖表——shell 面 .at 资产的字面 icon 名
    /// （`icon: "x"` 按钮形态 + `icon (name: "x")` 独立组件形态）+ 核心清单
    /// （pac 注册表 icon 值 + 缺省回退）逐名命中 lucide_svg。未命中即
    /// 渲染空图标（"占位色块"根因之一），此表为契约：新 icon 名先补
    /// lucide 臂再消费。
    /// PLAN-619（§8.5）回归锚：lucide 文档必须按**几何尺寸**渲染。
    /// 修前（fragment 未包 `<g>`）usvg 0.45 把 `circle`+`path` 组合画成 viewBox
    /// 的 0.583，导致跨端图标 ink 系统性偏小（AC-02 的 0.68 主因之一）。
    /// 断言 = glyph 的 ink 充满度落在几何值 0.833 的邻域（修前必红）。
    #[test]
    fn plan619_lucide_doc_renders_geometric_ink() {
        // 16px 下细线 AA 会吃掉边缘像素（测量阈值敏感）→ 取 24/48 两档。
        for px in [24u32, 48] {
            let doc = lucide_svg_doc_with("search", 2.0)
                .expect("doc")
                .replace("currentColor", "#ffffff");
            let tree =
                resvg::usvg::Tree::from_str(&doc, &resvg::usvg::Options::default()).expect("parse");
            let mut pm = tiny_skia::Pixmap::new(px, px).expect("pixmap");
            let ts = tree.size();
            resvg::render(
                &tree,
                tiny_skia::Transform::from_scale(px as f32 / ts.width(), px as f32 / ts.height()),
                &mut pm.as_mut(),
            );
            let buf = pm.take();
            let (mut a, mut b) = (px, 0u32);
            for y in 0..px {
                for x in 0..px {
                    if buf[((y * px + x) * 4 + 3) as usize] > 2 {
                        a = a.min(x);
                        b = b.max(x);
                    }
                }
            }
            let ratio = (b - a + 1) as f32 / px as f32;
            assert!(
                (0.78..=0.95).contains(&ratio),
                "px={px}: `search` ink 充满度 {ratio:.3} 应为几何值 ≈0.833（<g> 包装修法回归）"
            );
        }
    }

    #[test]
    fn lucide_icon_coverage_manifest_all_hit() {
        // ① .at 资产字面量扫描（五份内嵌资产,含 shell/desktop/switcher/
        // settings/notification_center——独立 icon 组件臂的字面 name 同扫）。
        let assets = [
            ("shell.at", crate::ui::shell::SHELL_AT),
            ("desktop.at", crate::ui::shell::DESKTOP_AT),
            ("switcher.at", crate::ui::shell::SWITCHER_AT),
            (
                "notification_center.at",
                crate::ui::shell::NOTIFICATION_CENTER_AT,
            ),
        ];
        let mut names: Vec<(String, String)> = Vec::new();
        for (file, src) in assets {
            for line in src.lines() {
                let line = line.trim();
                for pat in ["icon: \"", "name: \""] {
                    if let Some(pos) = line.find(pat) {
                        let rest = &line[pos + pat.len()..];
                        if let Some(end) = rest.find('"') {
                            // PLAN-668 R-03：`iconfile:` 是位图文件通道
                            // （2026-09-15 任务栏五钮裁定），不属 lucide
                            // 命名空间——命中断言只管 lucide 名。
                            if rest[..end].starts_with("iconfile:") {
                                continue;
                            }
                            names.push((file.to_string(), rest[..end].to_string()));
                        }
                    }
                }
            }
        }
        assert!(
            !names.is_empty(),
            "资产扫描应至少命中 bell/settings 等字面名"
        );
        // ② 核心清单:pac 注册表 icon 值 + registry 缺省回退（扫描 examples
        // 目录可用时动态并入,缺席时清单兜底——命中表不因布局漂移空转）。
        let mut manifest: Vec<String> = [
            "app-window", // registry 缺省回退
            "calculator",
            "bomb",
            "list-checks",
            "notebook", // dock 默认 pinned
            "folder",
            "search", // 027-file-manager / 028-launcher pac
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        if let Ok(entries) = std::fs::read_dir("../../examples/ui") {
            for e in entries.flatten() {
                let pac = e.path().join("pac.at");
                if let Ok(content) = std::fs::read_to_string(&pac) {
                    for line in content.lines() {
                        let line = line.trim();
                        if let Some(pos) = line.find("icon: \"") {
                            let rest = &line[pos + 7..];
                            if let Some(end) = rest.find('"') {
                                manifest.push(format!(
                                    "{}:{}",
                                    e.file_name().to_string_lossy(),
                                    &rest[..end]
                                ));
                            }
                        }
                    }
                }
            }
        }
        // ③ 逐名命中断言（源标注进 panic 信息,补名直达）。
        let mut misses: Vec<String> = Vec::new();
        for (file, name) in &names {
            if lucide_svg(name).is_none() {
                misses.push(format!("{file}:{name}"));
            }
        }
        for entry in &manifest {
            let name = entry.rsplit(':').next().unwrap_or(entry);
            if lucide_svg(name).is_none() {
                misses.push(format!("manifest:{entry}"));
            }
        }
        assert!(
            misses.is_empty(),
            "lucide 命中表缺口（补 lucide_svg 臂或改名）: {misses:?}"
        );
        // ④ G4② stroke-width 参数化冒烟:默认 2.0 / 大尺寸 1.5 文档生成。
        let doc_default = lucide_svg_doc("bell").expect("bell doc");
        assert!(doc_default.contains("stroke-width=\"2\""));
        let doc_thin = lucide_svg_doc_with("bell", 1.5).expect("bell thin doc");
        assert!(doc_thin.contains("stroke-width=\"1.5\""));
        // ⑤ G4③ 徽标色:哈希稳定 + 8 色板内 + 白字可读（WCAG 相对亮度
        // 线性化,≥4.5:1 AA 界）。
        let c1 = crate::ui::app_registry::badge_color_for("011-calculator");
        assert_eq!(
            c1,
            crate::ui::app_registry::badge_color_for("011-calculator")
        );
        assert!(c1.starts_with('#') && c1.len() == 7);
        let lin = |v: u8| {
            let c = v as f64 / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        for id in [
            "011-calculator",
            "013-todo",
            "015-notes",
            "019-video-app",
            "x",
        ] {
            let hex = crate::ui::app_registry::badge_color_for(id);
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
            let lum = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
            let contrast = 1.05 / (lum + 0.05);
            assert!(
                contrast >= 4.5,
                "徽标色 {hex} 白 glyph 对比 {contrast:.2}:1 不足 AA"
            );
        }
    }

    /// T2a：壁纸解析 + 图标/排除键 storage 往返——#hex 直传、坏路径回退
    /// 默认色、存在路径保留；逗号键解析（shell.dock.pinned 同形）。
    #[test]
    fn desktop_surface_storage_roundtrip_and_wallpaper_resolution() {
        let _guard = t2_isolate_storage("d496-t2a");
        use crate::ui::desktop_config::DesktopConfig;
        // 壁纸三分支（Plan 540 T2：源切单源 config.wallpaper_path——
        // storage publish 不再参与壁纸解析）。PLAN-526 T19：值缺席/坏路径
        // 不直接回退内置——先取壁纸目录首图（解析链：config dir 字段 →
        // env → stella 素材目录探测），目录不可用才回退 builtin；断言按
        // 新链放行两种合法结果（目录命中=存在文件路径 / 兜底=builtin 常量）。
        let absent = load_desktop_wallpaper(&DesktopConfig::default());
        assert!(
            absent == crate::ui::session::DESKTOP_WALLPAPER_DEFAULT
                || std::path::Path::new(&absent).is_file(),
            "键缺席 → 目录首图或内置兜底，得到 {absent}"
        );
        let hex_cfg = DesktopConfig {
            wallpaper_path: "#123456".to_string(),
            ..DesktopConfig::default()
        };
        // PLAN-712 r2 T-13/T-14：色值不再短路图片回退——主题默认图在场
        // （深 purple.png）时让位钦定对；无目录机器保持色值直传。
        let hex = load_desktop_wallpaper(&hex_cfg);
        assert!(
            hex.ends_with("purple.png") || hex == "#123456",
            "#hex → 主题默认图或（无目录机器）色值直传，得到 {hex}"
        );
        let light_hex_cfg = DesktopConfig {
            wallpaper_path: "#123456".to_string(),
            dark_theme: false,
            ..DesktopConfig::default()
        };
        let light_hex = load_desktop_wallpaper(&light_hex_cfg);
        assert!(
            light_hex.ends_with("songyu.png") || light_hex == "#123456",
            "浅色 hex → songyu.png 或色值直传，得到 {light_hex}"
        );
        let bad_cfg = DesktopConfig {
            wallpaper_path: "Z:/no/such/file.png".to_string(),
            ..DesktopConfig::default()
        };
        let bad = load_desktop_wallpaper(&bad_cfg);
        assert!(
            bad == crate::ui::session::DESKTOP_WALLPAPER_DEFAULT
                || std::path::Path::new(&bad).is_file(),
            "坏路径 → 目录首图或内置兜底，得到 {bad}"
        );
        let tmp = std::env::temp_dir().join(format!("autoui-496-wp-{}.png", std::process::id()));
        std::fs::write(&tmp, b"png").expect("临时壁纸写入");
        let file_cfg = DesktopConfig {
            wallpaper_path: tmp.to_string_lossy().to_string(),
            ..DesktopConfig::default()
        };
        assert_eq!(
            load_desktop_wallpaper(&file_cfg),
            tmp.to_string_lossy(),
            "存在路径保留"
        );
        // Plan 518：builtin: 内嵌资产方案——默认值即 builtin:ricepaper
        //(壁纸资产化,深浅主题共用浅色),用户显式键入亦为合法值直传。
        assert_eq!(
            crate::ui::session::DESKTOP_WALLPAPER_DEFAULT,
            "builtin:ricepaper",
            "518 默认壁纸 = 内嵌宣纸资产"
        );
        let builtin_cfg = DesktopConfig {
            wallpaper_path: "builtin:inkwash".to_string(),
            ..DesktopConfig::default()
        };
        // PLAN-712 r2 T-13：builtin 同 #hex——目录在场让位目录首图。
        let builtin = load_desktop_wallpaper(&builtin_cfg);
        assert!(
            builtin == "builtin:inkwash" || std::path::Path::new(&builtin).is_file(),
            "builtin → 目录首图或直传，得到 {builtin}"
        );
        // 内嵌资产可解码:load_image_bytes 返回 JPEG 字节(FF D8 魔数)。
        let bytes = load_image_bytes("builtin:ricepaper").expect("内嵌壁纸字节");
        assert_eq!(&bytes[0..2], &[0xFF, 0xD8], "JPEG 魔数");
        assert!(bytes.len() > 100_000, "整图非截断");
        assert!(
            load_image_bytes("builtin:no-such").is_none(),
            "未知 builtin 项"
        );
        // 图标/排除键逗号解析（空段剔除）。
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.icons",
            " a , b,,c ".into(),
        );
        auto_lang_core::vm::ffi::stdlib::storage_host_publish("shell.desktop.hidden", " b ".into());
        assert_eq!(
            load_desktop_id_list("shell.desktop.icons"),
            vec!["a", "b", "c"]
        );
        assert_eq!(load_desktop_id_list("shell.desktop.hidden"), vec!["b"]);
        assert!(load_desktop_id_list("shell.desktop.nonexistent").is_empty());
    }

    /// T2b：boot 合并去重——pinned ∪ 自定义（pinned 先列，custom 重叠去重）
    /// - hidden 排除（pinned/custom 通用）+ `__desktop_*` 投影注入形状
    /// （icon/label 注册表解析、缺省回退；`__desktop_bg` 壁纸片段）。
    #[test]
    fn desktop_surface_merge_dedupe_and_injection() {
        let _guard = t2_isolate_storage("d496-t2b");
        let mut ds = t3_session_with_shell();
        // PLAN-759 T-05：壁纸机器态隔离（同 desktop_bus_inbox 测试注记）——
        // 空目录挡主题默认图/目录首图两级，#hex 直传恢复确定性；断言保持
        // `bg-[#243b55]` 逐字精确（不设「或任意图片存在」宽松放行）。
        let _wall_dir = tempfile::TempDir::new().expect("wallpapers 隔离目录");
        ds.desktop.config.wallpapers_dir = _wall_dir.path().to_string_lossy().to_string();
        // 真 desktop.at 装载（z 槽面）。
        let comp = crate::ui::shell::build_desktop_surface_component().expect("desktop.at 装载");
        ds.desktop.desktop_app = Some(ds.allocate_app(comp));
        // 注册表：011/015 带标题与图标；013 缺席（label/icon 回退臂）。
        ds.desktop.registry_entries = vec![
            crate::ui::app_registry::AppRegistryEntry {
                media_root: None,
                photo_root: None,
                back_entry: None,

                id: "011-calculator".into(),
                title: "计算器".into(),
                title_zh: None,
                name: None,
                icon: "calculator".into(),
                category: "app".into(),
                entry: std::path::PathBuf::from("x/a.at"),
                render: "vm".into(),
                daemon: None,
                back_root: None,
                fit: false,
                desktop_visible: true,
                desktop_exe: None,
                desktop_render: None,
                opens: Vec::new(),
            },
            crate::ui::app_registry::AppRegistryEntry {
                media_root: None,
                photo_root: None,
                back_entry: None,

                id: "015-notes".into(),
                title: "便签".into(),
                title_zh: None,
                name: None,
                icon: "sticky-note".into(),
                category: "app".into(),
                entry: std::path::PathBuf::from("x/b.at"),
                render: "vm".into(),
                daemon: None,
                back_root: None,
                fit: false,
                desktop_visible: true,
                desktop_exe: None,
                desktop_render: None,
                opens: Vec::new(),
            },
        ];
        // PLAN-012 W4：dock_pinned 缺省空（t3_session_with_shell 不动）。
        // storage：custom 014-weather + 重叠 011-calculator；hidden 013-todo。
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.icons",
            "014-weather,011-calculator".into(),
        );
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.hidden",
            "013-todo".into(),
        );
        ds.desktop.config.wallpaper_path = "#243b55".to_string();
        ds.desktop.desktop_wallpaper = load_desktop_wallpaper(&ds.desktop.config);
        inject_desktop_surface(&mut ds);
        let entries = t496_read_array(&ds, "__desktop_icons");
        let ids: Vec<String> = entries
            .iter()
            .map(|e| t3_obj_str(t496_val_obj(e), "id"))
            .collect();
        assert_eq!(
            ids,
            vec!["014-weather", "011-calculator"],
            "custom 按 storage 序注入 + 011 重叠去重（缺省 pinned 空后无先列组）"
        );
        let first = t496_val_obj(&entries[0]);
        assert_eq!(
            t3_obj_str(first, "icon"),
            "app-window",
            "未登记回退占位图标"
        );
        assert_eq!(t3_obj_str(first, "label"), "014-weather", "label 回退 id");
        assert_eq!(t3_obj_str(first, "src"), "custom");
        let second = t496_val_obj(&entries[1]);
        assert_eq!(t3_obj_str(second, "icon"), "calculator", "icon 注册表解析");
        assert_eq!(t3_obj_str(second, "label"), "计算器", "label=注册表标题");
        assert_eq!(t3_obj_str(second, "src"), "custom");
        match t496_read(&ds, "__desktop_bg") {
            auto_val::Value::Str(ref s) => {
                assert_eq!(s.to_string(), "bg-[#243b55]", "#hex → 根 bg 实铺片段")
            }
            other => panic!("__desktop_bg 异常: {other:?}"),
        }
        match t496_read(&ds, "__desktop_hidden") {
            auto_val::Value::Str(ref s) => assert_eq!(s.to_string(), "013-todo"),
            other => panic!("__desktop_hidden 异常: {other:?}"),
        }
        // 图片壁纸分支：__desktop_bg 空（宿主壁纸图层铺底）。
        let tmp = std::env::temp_dir().join(format!("autoui-496-img-{}.png", std::process::id()));
        std::fs::write(&tmp, b"png").expect("临时壁纸写入");
        ds.desktop.config.wallpaper_path = tmp.to_string_lossy().to_string();
        ds.desktop.desktop_wallpaper = load_desktop_wallpaper(&ds.desktop.config);
        inject_desktop_surface(&mut ds);
        match t496_read(&ds, "__desktop_bg") {
            auto_val::Value::Str(ref s) => assert_eq!(s.to_string(), "", "图片路径 → 空片段"),
            other => panic!("__desktop_bg（图片）异常: {other:?}"),
        }
    }

    /// T1 族视图走查辅助：递归数 (mouse_area 双击臂数, 单击臂数, 指定文本出现数)。
    /// Plan 498 M0: 增单击臂计数(onclick 电路与双击同构)。
    fn t496_walk(
        v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
        dbl: &mut usize,
        clk: &mut usize,
        texts: &mut Vec<String>,
    ) {
        use crate::ui::view::View;
        match v {
            View::MouseArea {
                content,
                on_double_click,
                on_click,
                ..
            } => {
                if on_double_click.is_some() {
                    *dbl += 1;
                }
                if on_click.is_some() {
                    *clk += 1;
                }
                t496_walk(content, dbl, clk, texts);
            }
            View::Text { content, .. } => texts.push(content.clone()),
            // button "打开" 的 label 在 Button.label（非 Text 子节点）。
            View::Button { label, content, .. } => {
                texts.push(label.clone());
                if let Some(c) = content {
                    t496_walk(c, dbl, clk, texts);
                }
            }
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    t496_walk(c, dbl, clk, texts);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                t496_walk(child, dbl, clk, texts)
            }
            View::Grid { cells, .. } => {
                for c in cells {
                    t496_walk(c, dbl, clk, texts);
                }
            }
            // PLAN-526 T36：popover 下钻（blank 菜单 popover 化后桌面根即
            // popover 祖先——walk 不下钻则全部交互臂计数为 0）。
            View::Popover {
                anchor, content, ..
            } => {
                if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                    t496_walk(w, dbl, clk, texts);
                }
                t496_walk(content, dbl, clk, texts);
            }
            _ => {}
        }
    }

    /// PLAN-002 B：布局件事件面计数——(on_right_click 臂数, hover 变体类臂数)。
    /// 布局件三节点（Row/Column/Container）的右键与 hover 类消费是 pilot 的
    /// 断言面（desktop.at 图标格）。
    fn t002_walk_layout_events(
        v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
        rc: &mut usize,
        hv: &mut usize,
    ) {
        use crate::ui::view::View;
        let has_hover = |style: &Option<crate::ui::style::Style>| {
            style
                .as_ref()
                .map_or(false, |s| s.has_variant(crate::ui::style::Variant::Hover))
        };
        match v {
            View::Column {
                children,
                style,
                on_right_click,
                ..
            }
            | View::Row {
                children,
                style,
                on_right_click,
                ..
            } => {
                if on_right_click.is_some() {
                    *rc += 1;
                }
                if has_hover(style) {
                    *hv += 1;
                }
                for c in children {
                    t002_walk_layout_events(c, rc, hv);
                }
            }
            View::Container {
                child,
                style,
                on_right_click,
                ..
            } => {
                if on_right_click.is_some() {
                    *rc += 1;
                }
                if has_hover(style) {
                    *hv += 1;
                }
                t002_walk_layout_events(child, rc, hv);
            }
            View::MouseArea { content, .. } | View::Scrollable { child: content, .. } => {
                t002_walk_layout_events(content, rc, hv)
            }
            View::Popover {
                anchor, content, ..
            } => {
                if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                    t002_walk_layout_events(w, rc, hv);
                }
                t002_walk_layout_events(content, rc, hv);
            }
            View::Button { content, .. } => {
                if let Some(c) = content {
                    t002_walk_layout_events(c, rc, hv);
                }
            }
            View::Grid { cells, .. } => {
                for c in cells {
                    t002_walk_layout_events(c, rc, hv);
                }
            }
            _ => {}
        }
    }

    /// T1：desktop.at 装载 + 注入 headless——条目渲染数（网格 for 展开）、
    /// 双击 mouse-area on_double_click 电路（VM 动态路径 convert_view_messages
    /// 后存活——496 补臂前落 Empty 兜底）、右键菜单出现（IconMenu → 菜单
    /// 三项文本）、MenuRemove storage 直写、BlankPress 关菜单、ActivateApp
    /// → activate 动词记录（472 两臂复用）。
    #[test]
    fn desktop_surface_at_loads_interactions_and_dispatch() {
        let _guard = t2_isolate_storage("d496-t1");
        let mut ds = t3_session_with_shell();
        let comp = crate::ui::shell::build_desktop_surface_component().expect("desktop.at 装载");
        ds.desktop.desktop_app = Some(ds.allocate_app(comp));
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.icons",
            "014-weather,011-calculator".into(),
        );
        // PLAN-012 W4：缺省 dock_pinned 置空——custom 两枚（011 重叠去重）
        // → 2 条目（原"默认三枚 + custom 两枚 → 4"随缺省置空退役）。
        inject_desktop_surface(&mut ds);
        let surface = ds.desktop.desktop_app.unwrap();
        let (mut dbl, mut clk, mut texts) = (0usize, 0usize, Vec::new());
        {
            let app = ds.apps.get(&surface).unwrap();
            let (view, _, _) = app.component.view_with_debug_gated(false);
            t496_walk(&view, &mut dbl, &mut clk, &mut texts);
        }

        assert_eq!(
            dbl, 2,
            "每个图标格一枚 mouse-area 双击臂（缺省 pinned 置空后）"
        );
        // PLAN-526 T10：空白点击自布局件 onclick 迁至根 mouse-area on_press。
        // PLAN-012 W5：图标格 mouse-area onclick（IconPress 拖拽拾起臂）
        // ×2 格 + 根 ×1 = 3。
        assert_eq!(clk, 3, "根 onclick 1 + 图标 IconPress 2（W5 拖拽态机）");
        // PLAN-002 B pilot：图标格布局件右键挂点 + hover 变体类消费
        //（oncontextmenu 自 button 迁至格 col；格上 hover:bg-white/10）。
        let (mut rc, mut hv) = (0usize, 0usize);
        {
            let app = ds.apps.get(&surface).unwrap();
            let (view, _, _) = app.component.view_with_debug_gated(false);
            t002_walk_layout_events(&view, &mut rc, &mut hv);
        }
        assert_eq!(
            rc, 2,
            "每格一枚布局件右键臂（oncontextmenu → View::Column::on_right_click）"
        );
        assert_eq!(hv, 2, "每格一枚 hover: 变体类（布局件 hover 消费面）");
        assert_eq!(
            texts.iter().filter(|t| t.as_str() == "014-weather").count(),
            1,
            "label 渲染（未登记条目回退 id）"
        );
        // 转换后存活（convert_view_messages VM 动态路径——Empty 兜底即双击死）。
        let (mut dbl2, mut clk2, mut texts2) = (0usize, 0usize, Vec::new());
        {
            let app = ds.apps.get(&surface).unwrap();
            let (view, _, _) = app.component.view_with_debug_gated(false);
            let converted = convert_view_messages(view);
            t496_walk_iced(&converted, &mut dbl2, &mut clk2, &mut texts2);
        }
        assert_eq!(dbl2, 2, "convert_view_messages 后 mouse_area 双击臂存活");
        // 右键菜单：IconMenu 打开 → 三项文本出现；BlankPress 关闭消失。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("IconMenu", &[auto_val::Value::str("014-weather")])
                .expect("IconMenu handler");
        }
        let (mut dbl3, mut clk3, mut texts3) = (0usize, 0usize, Vec::new());
        {
            let app = ds.apps.get(&surface).unwrap();
            let (view, _, _) = app.component.view_with_debug_gated(false);
            t496_walk(&view, &mut dbl3, &mut clk3, &mut texts3);
        }
        for item in ["打开", "从桌面移除", "更换壁纸…"] {
            assert!(
                texts3.iter().any(|t| t == item),
                "菜单项「{item}」应出现（menu_id 打开态）"
            );
        }
        // MenuRemove：hidden 直写（storage）+ 菜单关闭。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("MenuRemove", &[])
                .expect("MenuRemove handler");
        }
        assert_eq!(
            auto_lang_core::vm::ffi::stdlib::storage_host_read("shell.desktop.hidden").as_deref(),
            Some("014-weather"),
            "移除 = shell.desktop.hidden 直写"
        );
        // BlankPress：关菜单。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("BlankPress", &[])
                .expect("BlankPress handler");
            app.component
                .bridge_mut()
                .call_handler("ActivateApp", &[auto_val::Value::str("011-calculator")])
                .expect("ActivateApp handler");
        }
        match t496_read(&ds, "__desktop_cmd") {
            auto_val::Value::Str(ref s) => {
                assert_eq!(
                    s.to_string(),
                    "activate\t011-calculator",
                    "双击/打开 → activate 动词（472 两臂复用）"
                )
            }
            other => panic!("__desktop_cmd 异常: {other:?}"),
        }
        // PLAN-014 W-02 追加语义：bus 记录由宿主每 update 周期排空——此处
        // 排空既有记录（兼断 activate 到达宿主），后续 wallpaper 臂读单记录。
        assert_eq!(
            ds.drain_app_desktop_commands(surface),
            vec![crate::ui::session::DesktopCommand::ActivateApp(
                "011-calculator".into()
            )],
            "activate 记录宿主排空（W-02 配套）"
        );
        match t496_read(&ds, "menu_id") {
            auto_val::Value::Str(ref s) => assert_eq!(s.to_string(), "", "空白点击关菜单"),
            other => panic!("menu_id 异常: {other:?}"),
        }
        // MenuWallpaper → open_settings（v1.4 无参动词）。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("IconMenu", &[auto_val::Value::str("013-todo")])
                .expect("IconMenu handler");
            app.component
                .bridge_mut()
                .call_handler("MenuWallpaper", &[])
                .expect("MenuWallpaper handler");
        }
        match t496_read(&ds, "__desktop_cmd") {
            auto_val::Value::Str(ref s) => {
                // PLAN-019 v1.7：更换壁纸入口改发组合动词（借负一屏 + 开
                // picker + 归属簿记宿主收口；open_settings 仅存「显示设置」臂）。
                assert_eq!(
                    s.to_string(),
                    "wallpaper_pick",
                    "更换壁纸入口 → wallpaper_pick"
                )
            }
            other => panic!("__desktop_cmd（wallpaper）异常: {other:?}"),
        }
    }

    /// PLAN-014 F-03 回归（builder 端到端）：条件样式（style: if 链）分支体
    /// 内的注释行此前把 Plan 339「单表达式」合同推到 stmts.len()==2 → 求值
    /// 落空串 → Style::parse 失败 → 布局件 style 整体 None → hover 变体类
    /// VM 轨归零（desktop.at 图标格五分支链现场）。合同：注释/空行不计入
    /// 分支体合同，深层 else 照常求值。
    #[test]
    fn conditional_style_with_comments_in_branches_resolves_hover() {
        let _guard = t2_isolate_storage("f03-style-comments");
        // PLAN-668 R-04：探针体原用 `state {}`（非块关键字）——旧解析器
        // 按视图元素误吞后真 view 块静默覆盖（Plan 425 兜底），PLAN-024
        // T-01 改重复 view 报错后即红；改用真关键字 model 保持探针语义。
        let src = concat!(
            "widget StyleProbe {\n",
            "    model {\n",
            "        var mode str = \"\"\n",
            "    }\n",
            "    view {\n",
            "        col {\n",
            "            style: if .mode == \"a\" {\n",
            "                \"bg-white/20 opacity-50\"\n",
            "            } else {\n",
            "                // 注释在 else 分支体内（F-03 现场）\n",
            "                if .mode == \"b\" {\n",
            "                    \"bg-primary/20\"\n",
            "                } else {\n",
            "                    // 再一层注释\n",
            "                    \"w-20 h-[72px] hover:bg-white/10\"\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let comp = crate::build_dynamic_component(src, None).expect("装载");
        let mut ds = t3_session_with_shell();
        let id = ds.allocate_app(comp);
        let app = ds.apps.get(&id).unwrap();
        let (view, _, _) = app.component.view_with_debug_gated(false);
        // mode="" → 两层条件全 false → 最深 else（hover 串）。
        let mut hover_cols = 0usize;
        let mut total_cols = 0usize;
        fn walk(
            v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
            hover_cols: &mut usize,
            total_cols: &mut usize,
        ) {
            use crate::ui::view::View;
            match v {
                View::Column {
                    children, style, ..
                } => {
                    *total_cols += 1;
                    if style
                        .as_ref()
                        .map_or(false, |s| s.has_variant(crate::ui::style::Variant::Hover))
                    {
                        *hover_cols += 1;
                    }
                    for c in children {
                        walk(c, hover_cols, total_cols);
                    }
                }
                View::Row { children, .. } => {
                    for c in children {
                        walk(c, hover_cols, total_cols);
                    }
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    walk(child, hover_cols, total_cols)
                }
                _ => {}
            }
        }
        walk(&view, &mut hover_cols, &mut total_cols);
        assert!(total_cols >= 1, "探针 col 存在");
        assert_eq!(hover_cols, 1, "注释不废链：最深 else 的 hover 变体类存活");
    }

    /// PLAN-010 T4 回归：desktop.at popover 的 ondismiss 必须提取自 events
    /// 桶（parser 把 `on*` 键升格为 ViewEvent，props 恒 miss——回退
    /// __popover_close 对 VM 态 popover 是 no-op，外点/Esc 关闭全失效）。
    /// 断言三层：VM 视图 on_dismiss 事件名（MenuClose×4 icon + BlankClose×1
    /// blank）、convert_view_messages 后存活（发布侧真实携带）、MenuClose
    /// handler 闭环（menu_id 清位 = 面板内/外点/Esc 三路关闭的落点）。
    #[test]
    fn p010_popover_ondismiss_extracted_from_events() {
        let _guard = t2_isolate_storage("p010-regress");
        let mut ds = t3_session_with_shell();
        let comp = crate::ui::shell::build_desktop_surface_component().expect("desktop.at 装载");
        ds.desktop.desktop_app = Some(ds.allocate_app(comp));
        auto_lang_core::vm::ffi::stdlib::storage_host_publish(
            "shell.desktop.icons",
            "014-weather,011-calculator".into(),
        );
        inject_desktop_surface(&mut ds);
        let surface = ds.desktop.desktop_app.unwrap();
        // 开 icon 菜单（menu_id 置位 → per-icon popover open）。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("IconMenu", &[auto_val::Value::str("014-weather")])
                .expect("IconMenu handler");
        }
        fn walk_vm(
            v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
            dismiss: &mut Vec<Option<String>>,
        ) {
            use crate::ui::view::View;
            match v {
                View::Popover {
                    anchor,
                    content,
                    on_dismiss,
                    ..
                } => {
                    dismiss.push(on_dismiss.as_ref().map(|m| match m {
                        crate::ui::interpreter::DynamicMessage::Typed { event_name, .. } => {
                            event_name.clone()
                        }
                        other => format!("{other:?}"),
                    }));
                    if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                        walk_vm(w, dismiss);
                    }
                    walk_vm(content, dismiss);
                }
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children {
                        walk_vm(c, dismiss);
                    }
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    walk_vm(child, dismiss)
                }
                View::MouseArea { content, .. } => walk_vm(content, dismiss),
                View::Grid { cells, .. } => {
                    for c in cells {
                        walk_vm(c, dismiss);
                    }
                }
                View::Button { content, .. } => {
                    if let Some(c) = content {
                        walk_vm(c, dismiss);
                    }
                }
                _ => {}
            }
        }
        fn walk_iced(v: &crate::ui::view::View<IcedMessage>, dismiss: &mut Vec<Option<String>>) {
            use crate::ui::view::View;
            match v {
                View::Popover {
                    anchor,
                    content,
                    on_dismiss,
                    ..
                } => {
                    dismiss.push(on_dismiss.as_ref().map(|m| m.event.clone()));
                    if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                        walk_iced(w, dismiss);
                    }
                    walk_iced(content, dismiss);
                }
                View::Column { children, .. } | View::Row { children, .. } => {
                    for c in children {
                        walk_iced(c, dismiss);
                    }
                }
                View::Container { child, .. } | View::Scrollable { child, .. } => {
                    walk_iced(child, dismiss)
                }
                View::MouseArea { content, .. } => walk_iced(content, dismiss),
                View::Grid { cells, .. } => {
                    for c in cells {
                        walk_iced(c, dismiss);
                    }
                }
                View::Button { content, .. } => {
                    if let Some(c) = content {
                        walk_iced(c, dismiss);
                    }
                }
                _ => {}
            }
        }
        let app = ds.apps.get(&surface).unwrap();
        let (view, _, _) = app.component.view_with_debug_gated(false);
        let mut vm_names = Vec::new();
        walk_vm(&view, &mut vm_names);
        assert_eq!(
            vm_names
                .iter()
                .filter(|n| n.as_deref() == Some("MenuClose"))
                .count(),
            2,
            "icon 菜单 popover 的 ondismiss 应提取为 MenuClose（夹具发布 2 枚图标；\
             历史期望 4 与夹具不符——本机基线实证预存红，PLAN-019 T-07 顺带修正）: {vm_names:?}"
        );
        assert_eq!(
            vm_names
                .iter()
                .filter(|n| n.as_deref() == Some("BlankClose"))
                .count(),
            1,
            "blank 菜单 popover 的 ondismiss 应提取为 BlankClose: {vm_names:?}"
        );
        // PLAN-019 v1.7：壁纸 picker popover 的 ondismiss（第三个浮层）。
        assert_eq!(
            vm_names
                .iter()
                .filter(|n| n.as_deref() == Some("PickerDismiss"))
                .count(),
            1,
            "picker popover 的 ondismiss 应提取为 PickerDismiss: {vm_names:?}"
        );
        assert!(
            vm_names
                .iter()
                .all(|n| n.as_deref() != Some("__popover_close")),
            "不允许再落 __popover_close 回退（VM 态 popover 无此处理语义）: {vm_names:?}"
        );
        // 转换后存活（发布侧真实携带——popover Panel shell.publish 的消息）。
        let converted = convert_view_messages(view);
        let mut iced_names = Vec::new();
        walk_iced(&converted, &mut iced_names);
        assert_eq!(
            iced_names
                .iter()
                .filter(|n| n.as_deref() == Some("MenuClose"))
                .count(),
            2,
            "convert_view_messages 后 MenuClose ondismiss 存活（夹具 2 枚图标）"
        );
        assert_eq!(
            iced_names
                .iter()
                .filter(|n| n.as_deref() == Some("PickerDismiss"))
                .count(),
            1,
            "convert_view_messages 后 PickerDismiss ondismiss 存活（PLAN-019 picker 浮层）"
        );
        // handler 闭环：MenuClose 到达即 menu_id 清位（外点/Esc 关闭落点）。
        {
            let app = ds.apps.get_mut(&surface).unwrap();
            app.component
                .bridge_mut()
                .call_handler("MenuClose", &[])
                .expect("MenuClose handler");
        }
        match t496_read(&ds, "menu_id") {
            auto_val::Value::Str(ref s) => {
                assert_eq!(s.to_string(), "", "MenuClose → menu_id 清位")
            }
            other => panic!("menu_id 异常: {other:?}"),
        }
    }

    /// T1 走查（iced 侧）：View<IcedMessage> 版（convert_view_messages 后）。
    /// Plan 498 M0: 增单击臂计数。
    fn t496_walk_iced(
        v: &crate::ui::view::View<IcedMessage>,
        dbl: &mut usize,
        clk: &mut usize,
        texts: &mut Vec<String>,
    ) {
        use crate::ui::view::View;
        match v {
            View::MouseArea {
                content,
                on_double_click,
                on_click,
                ..
            } => {
                if on_double_click.is_some() {
                    *dbl += 1;
                }
                if on_click.is_some() {
                    *clk += 1;
                }
                t496_walk_iced(content, dbl, clk, texts);
            }
            View::Text { content, .. } => texts.push(content.clone()),
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    t496_walk_iced(c, dbl, clk, texts);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                t496_walk_iced(child, dbl, clk, texts)
            }
            View::Grid { cells, .. } => {
                for c in cells {
                    t496_walk_iced(c, dbl, clk, texts);
                }
            }
            // PLAN-526 T36：popover 下钻（同 t496_walk 注）。
            View::Popover {
                anchor, content, ..
            } => {
                if let crate::ui::view::PopoverAnchor::Widget(w) = anchor {
                    t496_walk_iced(w, dbl, clk, texts);
                }
                t496_walk_iced(content, dbl, clk, texts);
            }
            _ => {}
        }
    }

    /// Plan 464 T4：summon_launcher 无头单测——懒挂载 + 下行注入（真注册表
    /// 覆盖 mock / hosted / visible 置位 / ApplyFilter 同步重算）。键流与
    /// 启动后隐匿半边走 ui_desktop 实机（T4 验收 §5.1）。
    #[test]
    fn summon_launcher_mounts_and_injects() {
        use crate::ui::app_registry::AppRegistryEntry;

        // 临时 launcher 源（裁剪形态：seam 状态 + ApplyFilter 可调用）
        let dir = std::env::temp_dir().join("autoui-464-summon-test");
        let _ = std::fs::create_dir_all(&dir);
        let src = dir.join("app.at");
        std::fs::write(
            &src,
            r#"widget TestLauncher {
    model {
        var visible str = "0"
        var hosted str = "0"
        var __focus_input str = "0"
        var apps_names = []
        var ranked = []
        var nres int = 0
    }
    msg { ApplyFilter }
    view { col { text "palette" } }
    on {
        .ApplyFilter -> {
            .ranked = []
            var i int = 0
            while i < .apps_names.len() {
                var row = { name: .apps_names[i] }
                .ranked.push(row)
                i = i + 1
            }
            .nres = .ranked.len()
        }
    }
}
"#,
        )
        .unwrap();

        let entry = |id: &str, title: &str| AppRegistryEntry {
            media_root: None,
            photo_root: None,
            back_entry: None,

            id: id.to_string(),
            title: title.to_string(),
            title_zh: None,
            name: None,
            icon: "app-window".to_string(),
            category: "app".to_string(),
            entry: std::path::PathBuf::from(id),
            render: "vue".to_string(),
            daemon: None,
            back_root: None,
            fit: false,
            desktop_visible: true,
            desktop_exe: None,
            desktop_render: None,
            opens: Vec::new(),
        };

        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        ds.desktop.launcher_entry = Some(src);
        ds.desktop.registry_entries = vec![
            entry("011-calculator", "Calculator"),
            entry("028-launcher", "launcher"),
        ];

        let _focus_task = summon_launcher(&mut ds);
        let _ = &entry;

        // 懒挂载 + visible 置位（launcher_visible 读回）
        assert!(ds.desktop.launcher_app.is_some(), "launcher 应已挂载");
        assert!(ds.launcher_visible(), "召唤后 visible 应为 1");

        let la = ds.desktop.launcher_app.unwrap();
        let app = ds.apps.get(&la).expect("launcher app in session");
        // hosted 置位（隐藏独立模式开关）
        match app.component.read_state("hosted") {
            Ok(auto_val::Value::Str(ref s)) => assert_eq!(s.to_string(), "1"),
            other => panic!("hosted 读回异常: {other:?}"),
        }
        // 聚焦请求置位（update_inner 尾部下个消息周期消费）
        match app.component.read_state("__focus_input") {
            Ok(auto_val::Value::Str(ref s)) => assert_eq!(s.to_string(), "1"),
            other => panic!("__focus_input 读回异常: {other:?}"),
        }
        // 真注册表注入覆盖 mock + ApplyFilter 同步重算（nres = 1）。
        // PLAN-012 W6 fence：注册表含 028-launcher 条目（:11609 发现链
        // 同源 mock）→ 注入清单必须排除 launcher 自身，召唤链不受影响。
        match app.component.read_state("nres") {
            Ok(auto_val::Value::Int(n)) => {
                assert_eq!(n, 1, "ApplyFilter 应按注入清单重算（launcher 自身已过滤）")
            }
            other => panic!("nres 读回异常: {other:?}"),
        }
        let names = match app.component.read_state("apps_names") {
            Ok(auto_val::Value::Array(a)) => a.values,
            Ok(auto_val::Value::VmRef(r)) => t3_deref_list(app, r.id as u64, "apps_names"),
            other => panic!("apps_names 读回异常: {other:?}"),
        };
        assert_eq!(names.len(), 1, "注入清单应仅剩非 launcher 条目");
        assert!(
            !names.iter().any(|v| v.to_string().contains("launcher")),
            "apps_names 不应含 launcher 自身"
        );
    }

    /// Plan 464 T4 探针：宿主 write_state_vec 注入 Obj 数组后，VM 侧
    /// `for a in .apps` + `a.title` 字段读（任务栏同型）是否保真。
    #[test]
    fn injected_obj_array_for_field_read() {
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::build_dynamic_component(
            r#"widget Probe {
    model {
        var apps_names = []
        var apps_titles = []
        var joined str = ""
    }
    msg { Collect }
    view { col { text .joined } }
    on {
        .Collect -> {
            .joined = ""
            var i int = 0
            while i < .apps_names.len() {
                .joined = .joined + .apps_titles[i] + ","
                i = i + 1
            }
        }
    }
}
"#,
            None,
        )
        .unwrap();
        let id = ds.allocate_app(comp);
        // 串列并行注入（Obj 数组的 VM handler 字段读失效——上面断言钉死；
        // 字符串列表的下标读另测）
        let apps = vec![
            auto_val::Value::Str("011-calculator".into()),
            auto_val::Value::Str("028-launcher".into()),
        ];
        let titles = vec![
            auto_val::Value::Str("Calculator".into()),
            auto_val::Value::Str("launcher".into()),
        ];
        {
            let app = ds.apps.get_mut(&id).unwrap();
            let _ = app.component.write_state_vec("apps_names", apps);
            let _ = app.component.write_state_vec("apps_titles", titles);
        }
        {
            let app = ds.apps.get_mut(&id).unwrap();
            let r = app.component.bridge_mut().call_handler("Collect", &[]);
            if let Err(e) = r {
                panic!("Collect failed: {e}");
            }
        }
        let app = ds.apps.get(&id).unwrap();
        match app.component.read_state("joined") {
            Ok(auto_val::Value::Str(ref s)) => {
                assert_eq!(s.to_string(), "Calculator,launcher,", "for 绑定字段读保真")
            }
            other => panic!("joined 读回异常: {other:?}"),
        }
        let _ = ds;
    }

    /// PLAN-042 T-09（AC-04 执行臂归因 headless 断言）：`log` 动词执行臂
    /// 入环 source = notify_source（注册表分段归因 registry_id；特权面
    /// None → "privileged"）。
    #[test]
    fn p042_log_verb_ring_attribution() {
        use crate::ui::session::DesktopCommand as DC;
        use crate::ui::syslog::SyslogLevel;
        let mut ds = crate::ui::session::DesktopSession::__test_session();
        // 特权面（notify_source None）→ source="privileged"。
        let (exit, _tasks) = execute_desktop_commands(
            &mut ds,
            vec![DC::Syslog(SyslogLevel::Info, "p042-privileged-line".into())],
        );
        assert!(!exit);
        // 注册表分段（notify_source 置位）→ source=registry_id。
        *ds.desktop.notify_source.borrow_mut() = Some("039-syslog".into());
        let _ = execute_desktop_commands(
            &mut ds,
            vec![DC::Syslog(
                SyslogLevel::Error,
                "p042-attributed-line".into(),
            )],
        );
        *ds.desktop.notify_source.borrow_mut() = None;
        let snap = crate::ui::syslog::snapshot();
        let priv_e = snap
            .iter()
            .find(|e| e.msg == "p042-privileged-line")
            .expect("privileged line in ring");
        assert_eq!(priv_e.source, "privileged");
        assert_eq!(priv_e.level, SyslogLevel::Info);
        let attr_e = snap
            .iter()
            .find(|e| e.msg == "p042-attributed-line")
            .expect("attributed line in ring");
        assert_eq!(attr_e.source, "039-syslog");
        assert_eq!(attr_e.level, SyslogLevel::Error);
    }

    /// PLAN-042 T-09（AC-05/AC-06 headless 实证；#[ignore] + P042_APPS_DIR
    /// env 门——apps/auto-syslog 源在跨仓 auto-os 工作树，CI 不依赖）：
    /// 注册表 launch → 窗在册 registry_id=="039-syslog" → 注入泵 seq 门 +
    /// 500ms 节流 + 平行列表下行 + 显式 Rebuild。
    /// 运行：P042_APPS_DIR=<auto-os>/apps cargo nextest run -p auto-lang
    /// --lib --features ui-iced p042_syslog_window_launch_and_pump -- --ignored
    #[test]
    #[ignore = "requires P042_APPS_DIR pointing at auto-os worktree apps/"]
    fn p042_syslog_window_launch_and_pump() {
        use crate::ui::syslog::SyslogLevel;
        let root = std::path::PathBuf::from(
            std::env::var("P042_APPS_DIR").expect("P042_APPS_DIR required"),
        );
        let mut ds = t3_session_with_shell();
        let apps = std::sync::Arc::new(crate::ui::app_registry::scan_apps(
            &root,
            &crate::ui::app_registry::ScanOptions::default(),
        ));
        assert!(
            apps.iter().any(|e| e.id == "039-syslog"),
            "注册表应含 039-syslog"
        );
        ds.desktop.registry_entries = (*apps).clone();
        ds.desktop.app_resolver = Some({
            let apps = apps.clone();
            std::sync::Arc::new(move |name: &str| {
                let e = apps.iter().find(|a| a.id == name)?;
                Some(crate::ui::session::LaunchSpec {
                    media_root: None,
                    photo_root: None,
                    back_entry: None,
                    code: std::fs::read_to_string(&e.entry).ok()?,
                    source_path: Some(e.entry.to_string_lossy().to_string()),
                    title: Some(e.title.clone()),
                    name: e.name.clone(),
                    fit: e.fit,
                    daemon: None,
                    back_root: None,
                    exe: None,
                    opens: Vec::new(),
                    render_decl: None,
                })
            })
        });
        // ① launch → 窗在册 + registry_id 归因（AC-05）。
        let wid = ds.launch_app("039-syslog").expect("syslog launch");
        let v = &ds.host.as_ref().unwrap().wm.wins[&wid];
        assert_eq!(v.registry_id.as_deref(), Some("039-syslog"));
        let app_id = v.app;
        // ② 环入三条 → 首拍注入（last=None 立即放行基线）。
        crate::ui::syslog::push(SyslogLevel::Info, "host", "[p042] line one".into());
        crate::ui::syslog::push(SyslogLevel::Warn, "host", "[p042] line two".into());
        crate::ui::syslog::push(SyslogLevel::Error, "host", "[p042] line three".into());
        syslog_inject_tick(&mut ds);
        {
            let app = ds.apps.get(&app_id).unwrap();
            let seqs = app
                .component
                .bridge()
                .read_state_as_vec("__syslog_seq")
                .expect("seq list readable");
            let consumed = app.component.read_state("consumed").ok();
            assert_eq!(seqs.len(), 3, "基线注入三行（平行列表）");
            match consumed {
                Some(auto_val::Value::Int(n)) => assert_eq!(n, 3, "Rebuild 显式臂已消费"),
                other => panic!("注入面状态异常: {other:?}"),
            }
        }
        // ③ seq 无变化 → 零注入（重注入幂等不可直接观测——断言簿记不变 +
        //    无 panic 即门走通）。
        syslog_inject_tick(&mut ds);
        // ④ 新行 + 立即拍 → 500ms 攒批拒绝（内容不含 line four）。
        let seq4 = crate::ui::syslog::push(SyslogLevel::Info, "host", "[p042] line four".into());
        syslog_inject_tick(&mut ds);
        {
            let app = ds.apps.get(&app_id).unwrap();
            let consumed = app.component.read_state("consumed").ok();
            match consumed {
                Some(auto_val::Value::Int(n)) => assert_eq!(n, 3, "节流窗内新行未注入"),
                other => panic!("consumed 异常: {other:?}"),
            }
        }
        // ⑤ 过节流窗 → 一拍全量追平。
        std::thread::sleep(std::time::Duration::from_millis(600));
        syslog_inject_tick(&mut ds);
        {
            let app = ds.apps.get(&app_id).unwrap();
            let consumed = app.component.read_state("consumed").ok();
            match consumed {
                Some(auto_val::Value::Int(n)) => {
                    assert!(n >= 4, "过节流窗追平（含 line four seq={seq4}）");
                }
                other => panic!("consumed 异常: {other:?}"),
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum TestMessage {
        Click,
        Toggle(bool),
    }

    /// Plan 442 A6: a lang-bearing code text (codeblock 的 lang-<token>
    /// class)走 syntect highlight-only API——span 文本拼接还原原文,
    /// 关键字/字符串着色且与 lang-less 手写 tokenizer 色板不同源。
    #[test]
    fn test_highlight_code_with_lang_uses_syntect() {
        let code = "fn main() { let s = \"hi\"; }";
        let spans = highlight_code(code, Some("rust"));
        let joined: String = spans.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(joined, code);
        let kw = spans
            .iter()
            .find(|s| s.text.as_ref() == "fn")
            .expect("fn span");
        assert!(kw.color.is_some(), "syntect keyword must be colored");
        // 语法把字符串引号与内容拆为不同 region——按内容匹配。
        let st = spans
            .iter()
            .find(|s| s.text.as_ref().contains("hi"))
            .expect("string span");
        assert!(st.color.is_some(), "syntect string must be colored");
    }

    /// Plan 442 A6: codeblock 视图携带 lang-<token> class,renderer 提取
    /// 后用于语法选择(Style::parse → classes → CodeLang)。
    #[test]
    fn test_codeblock_lang_class_roundtrip() {
        let style = crate::ui::style::Style::parse("font-mono text-sm text-zinc-50 lang-rust")
            .expect("style parses");
        let lang = style.classes.iter().find_map(|c| match c {
            StyleClass::CodeLang(l) => Some(l.clone()),
            _ => None,
        });
        assert_eq!(lang.as_deref(), Some("rust"));
        // 未知/畸形 lang token 不解析为 class(不产生样式)。
        let bad = crate::ui::style::Style::parse("font-mono lang- bad").unwrap();
        assert!(bad
            .classes
            .iter()
            .all(|c| !matches!(c, StyleClass::CodeLang(_))));
    }

    #[test]
    fn test_text_conversion() {
        let view: AbstractView<TestMessage> = AbstractView::text("Hello".to_string());
        let _element = view.into_iced();
    }

    // ========== Plan 411 P2-A① — Prism tomorrow 色板 ==========

    #[test]
    fn test_highlight_prism_palette() {
        // Plan 442 A6: lang-less mono text keeps the hand-rolled tokenizer.
        let spans = highlight_code("fn install() // done\nlet ok = \"hi\" && true", None);
        let color_of = |t: &str| -> Option<iced::Color> {
            spans
                .iter()
                .find(|s| s.text.as_ref() == t)
                .unwrap_or_else(|| panic!("span {:?} not found", t))
                .color
        };
        // keyword #cc99cd(紫)
        assert_eq!(
            color_of("fn"),
            Some(iced::Color::from_rgb8(0xcc, 0x99, 0xcd))
        );
        // function(ident 后随 `(`)与 boolean/number 同为 #f08d49(橙)
        assert_eq!(
            color_of("install"),
            Some(iced::Color::from_rgb8(0xf0, 0x8d, 0x49))
        );
        assert_eq!(
            color_of("true"),
            Some(iced::Color::from_rgb8(0xf0, 0x8d, 0x49))
        );
        // 字符串 #7ec699(绿)
        assert_eq!(
            color_of("\"hi\""),
            Some(iced::Color::from_rgb8(0x7e, 0xc6, 0x99))
        );
        // 括号 punctuation #ccc、运算符 #67cdcc(青)
        assert_eq!(
            color_of("("),
            Some(iced::Color::from_rgb8(0xcc, 0xcc, 0xcc))
        );
        assert_eq!(
            color_of("&&"),
            Some(iced::Color::from_rgb8(0x67, 0xcd, 0xcc))
        );
        // 普通标识符不着色(基色)
        assert_eq!(color_of("ok"), None);
    }

    #[test]
    fn test_highlight_bash_cmd_and_comment() {
        // Plan 442 A6: lang-less mono text keeps the hand-rolled tokenizer.
        let spans = highlight_code("npx degit hi # vite", None);
        let color_of = |t: &str| -> Option<iced::Color> {
            spans.iter().find(|s| s.text.as_ref() == t).unwrap().color
        };
        // prism-bash 的 function 橙近似
        assert_eq!(
            color_of("npx"),
            Some(iced::Color::from_rgb8(0xf0, 0x8d, 0x49))
        );
        // 注释灰 #999
        assert_eq!(
            color_of("# vite"),
            Some(iced::Color::from_rgb8(0x99, 0x99, 0x99))
        );
    }

    // ========== Plan 411 P2-A④ — 表头样式注入 ==========

    #[test]
    fn test_plan045_col_fixed_width_dispatch() {
        // Plan 045 T2: 两态分派——Some 按列号取值；不足尾列补 None（自然宽）；
        // 超列数尾值自然截断（get 越界即 None）。
        let ws = vec![200.0f32, 300.0];
        assert_eq!(col_fixed_width(Some(&ws), 0), Some(200.0));
        assert_eq!(col_fixed_width(Some(&ws), 1), Some(300.0));
        assert_eq!(col_fixed_width(Some(&ws), 2), None, "长度不足尾列补自然宽");
        assert_eq!(col_fixed_width(None, 0), None, "None 态全自然宽");
        assert_eq!(col_fixed_width(Some(&[]), 0), None, "空数组=自然宽");
    }

    #[test]
    fn test_table_header_style_recursive() {
        let mut v: AbstractView<TestMessage> = AbstractView::Row {
            children: vec![
                AbstractView::Text {
                    content: "Prop".to_string(),
                    style: None,
                    selectable: false,
                },
                AbstractView::Text {
                    content: "Type".to_string(),
                    style: Some(Style {
                        classes: vec![],
                        hover_classes: vec![],
                        variant_classes: vec![],
                    }),
                    selectable: false,
                },
            ],
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
            on_right_click: None,
        };
        apply_table_header_style(&mut v);
        match v {
            AbstractView::Row { children, .. } => {
                for c in &children {
                    if let AbstractView::Text { style, .. } = c {
                        let s = style.as_ref().expect("style created");
                        assert!(s
                            .classes
                            .iter()
                            .any(|c| matches!(c, StyleClass::FontMedium)));
                        assert!(s
                            .classes
                            .iter()
                            .any(|c| matches!(c, StyleClass::TextColor(Color::OnSurface))));
                    } else {
                        panic!("expected text");
                    }
                }
            }
            _ => panic!("expected row"),
        }
    }

    // ========== Plan 412 F2 — grid auto-placement ==========

    #[test]
    fn test_plan412_grid_placements_plain_and_tail_row() {
        // 5 cells, cols=3 → rows [0,1,2], [3,4](尾行不满,由 Space 槽补齐)
        let p = grid_row_placements(&[1, 1, 1, 1, 1], 3);
        assert_eq!(
            p,
            vec![
                vec![(0, 0, 1), (1, 1, 1), (2, 2, 1)],
                vec![(3, 0, 1), (4, 1, 1)]
            ]
        );
    }

    #[test]
    fn test_plan412_grid_placements_col_span_wraps() {
        // cols=3, [1,1,2,1] → 第 3 个 cell span-2 放不下(2+2>3)换行占 0..2
        let p = grid_row_placements(&[1, 1, 2, 1], 3);
        assert_eq!(
            p,
            vec![vec![(0, 0, 1), (1, 1, 1)], vec![(2, 0, 2), (3, 2, 1)]]
        );
        // span-2 恰好填满尾位(1+2=3)不换行
        let p = grid_row_placements(&[1, 2], 3);
        assert_eq!(p, vec![vec![(0, 0, 1), (1, 1, 2)]]);
    }

    #[test]
    fn test_plan412_grid_placements_span_clamped_to_cols() {
        // span > cols → 钳制为整行
        let p = grid_row_placements(&[5, 1], 3);
        assert_eq!(p, vec![vec![(0, 0, 3)], vec![(1, 0, 1)]]);
    }

    #[test]
    fn test_plan412_grid_placements_full_span_sequence() {
        // /grid-span 混排画廊:cols=3, [2,1,1,3]
        // (0,0,2)+(1,2,1) 填满首行;1 换行;(3) span-3=整行再换行
        let p = grid_row_placements(&[2, 1, 1, 3], 3);
        assert_eq!(
            p,
            vec![vec![(0, 0, 2), (1, 2, 1)], vec![(2, 0, 1)], vec![(3, 0, 3)],]
        );
    }

    #[test]
    fn test_plan412_justify_spacer_math() {
        // around: lead/trail 权重 1、between 权重 2(总 2n → 端=半格)
        assert_eq!(
            row_justify_spacers(Some(IcedJustify::Around)),
            (Some(1), Some(2), Some(1))
        );
        // evenly: n+1 个等权(含两端)
        assert_eq!(
            row_justify_spacers(Some(IcedJustify::Evenly)),
            (Some(1), Some(1), Some(1))
        );
        assert_eq!(
            row_justify_spacers(Some(IcedJustify::Between)),
            (None, Some(1), None)
        );
        assert_eq!(row_justify_spacers(None), (None, None, None));
    }

    #[test]
    fn test_button_conversion() {
        let view = AbstractView::button(("Click me".to_string(), TestMessage::Click));
        let _element = view.into_iced();
    }

    #[test]
    fn test_column_conversion() {
        let view = AbstractView::col()
            .spacing(10)
            .padding(20)
            .child(AbstractView::text("Item 1"))
            .child(AbstractView::button((
                "Click".to_string(),
                TestMessage::Click,
            )))
            .build();

        let _element = view.into_iced();
    }

    #[test]
    fn test_apply_highlight_mut_wraps_selected_child() {
        // Plan 311 P2-B-2: selecting a VNode wraps exactly that node in an
        // orange-bordered Container (canvas outline), leaving siblings + the
        // root structure intact. Path scheme matches view_to_vtree_with_paths.
        use crate::ui::vnode::{id_from_path, VNodeId};
        let mut view: AbstractView<TestMessage> = AbstractView::col()
            .child(AbstractView::text("a".to_string()))
            .child(AbstractView::text("b".to_string()))
            .build();
        // child[0] sits at path [0].
        let target = VNodeId::new(id_from_path(&[0u16]));
        apply_highlight_mut(&mut view, Some(target));

        let AbstractView::Column { children, .. } = &view else {
            panic!("root must remain a Column");
        };
        assert_eq!(children.len(), 2, "sibling count unchanged");
        assert!(
            matches!(&children[0], AbstractView::Container { .. }),
            "selected child[0] must be wrapped in a Container"
        );
        if let AbstractView::Container { child, .. } = &children[0] {
            assert!(
                matches!(child.as_ref(), AbstractView::Text { content, .. } if content == "a"),
                "wrapped payload is the original Text(\"a\")"
            );
        }
        assert!(
            matches!(&children[1], AbstractView::Text { content, .. } if content == "b"),
            "unselected child[1] untouched"
        );
    }

    #[test]
    fn test_apply_highlight_mut_none_is_noop() {
        let mut view: AbstractView<TestMessage> = AbstractView::col()
            .child(AbstractView::text("a".to_string()))
            .build();
        apply_highlight_mut(&mut view, None);
        assert!(matches!(view, AbstractView::Column { .. }));
    }

    #[test]
    fn test_checkbox_conversion() {
        let view = AbstractView::checkbox(true, "Check me").on_toggle(TestMessage::Toggle(true));
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_text() {
        let view: AbstractView<TestMessage> =
            AbstractView::text_styled("Styled", "text-lg font-bold text-red-500");
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_column() {
        let view: AbstractView<TestMessage> = AbstractView::col()
            .style("gap-4 p-6 bg-white")
            .child(AbstractView::text("Child"))
            .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_button() {
        let view: AbstractView<TestMessage> = AbstractView::button_styled(
            "Styled Button",
            TestMessage::Click,
            "px-4 py-2 bg-blue-500 text-white rounded",
        );
        let _element = view.into_iced();
    }

    #[test]
    fn test_container_with_style() {
        let view: AbstractView<TestMessage> =
            AbstractView::container(AbstractView::text("Content"))
                .style("p-8 bg-white w-full")
                .center()
                .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_input_with_style() {
        let view: AbstractView<TestMessage> = AbstractView::input("Placeholder")
            .style("px-3 py-2 border")
            .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_scrollable_with_style() {
        let view: AbstractView<TestMessage> =
            AbstractView::scrollable(AbstractView::text("Content"))
                .style("w-full h-64")
                .build();
        let _element = view.into_iced();
    }

    // Plan 319: patch_input_values must descend into Grid cells — without the
    // explicit Grid arm the `_ => {}` catch-all would silently leave nested
    // Input values stale.
    #[test]
    fn test_grid_patch_input_values_updates_cell() {
        use std::collections::HashMap;
        let mut view: AbstractView<DynamicMessage> = AbstractView::Grid {
            cols: 2,
            gap: 4,
            cells: vec![AbstractView::Input {
                placeholder: "type".to_string(),
                value: String::new(),
                on_change: Some(DynamicMessage::String("name".to_string())),
                on_submit: None,
                width: None,
                password: false,
                style: None,
                disabled: false,
            }],
            style: None,
        };

        let mut map = HashMap::new();
        map.insert("name".to_string(), "patched".to_string());
        patch_input_values(&mut view, &map);

        match view {
            AbstractView::Grid { cells, .. } => match &cells[0] {
                AbstractView::Input { value, .. } => assert_eq!(value, "patched"),
                _ => panic!("expected Input cell"),
            },
            _ => panic!("expected Grid after patch"),
        }
    }

    // Plan 319 regression: convert_view_messages MUST preserve Grid. Its match
    // has a `_ => Empty` wildcard (non-exhaustive — invisible to the compiler),
    // so without an explicit Grid arm the entire grid silently became Empty
    // (the calendar's dates vanished in VM mode). This test is the only guard.
    #[test]
    fn test_convert_view_messages_preserves_grid() {
        let grid: AbstractView<DynamicMessage> = AbstractView::Grid {
            cols: 7,
            gap: 0,
            cells: vec![
                AbstractView::Text {
                    content: "Su".to_string(),
                    style: None,
                    selectable: false,
                },
                AbstractView::Button {
                    disabled: false,
                    label: "1".to_string(),
                    onclick: DynamicMessage::String("pick".to_string()),
                    style: None,
                    on_right_click: None,
                    content: None,
                },
            ],
            style: None,
        };

        let converted = convert_view_messages(grid);

        match converted {
            AbstractView::Grid { cols, cells, .. } => {
                assert_eq!(cols, 7);
                assert_eq!(cells.len(), 2);
                assert!(matches!(cells[0], AbstractView::Text { .. }));
                assert!(matches!(cells[1], AbstractView::Button { .. }));
            }
            _ => panic!("convert_view_messages dropped the Grid (hit the _ => Empty wildcard)"),
        }
    }

    // PLAN-002 A2 regression: convert_view_messages MUST preserve WindowThumbnail.
    // 497 added the tag to the builder + into_iced but missed THIS VM-mode
    // message bridge — every dock hover thumbnail became Empty (the 8×8 shell
    // speck + offset, 526 KNOWN-DEBT 🟢×2). Same class as Grid 319 / menubar
    // 422 / 496 MouseArea; this test is the fourth fence of that ditch.
    #[test]
    fn test_convert_view_messages_preserves_window_thumbnail() {
        let thumb: AbstractView<DynamicMessage> = AbstractView::WindowThumbnail {
            wid: "2".to_string(),
            fallback_icon: "app-window".to_string(),
            style: crate::ui::style::Style::parse("w-48 h-28 rounded").ok(),
        };

        let converted = convert_view_messages(thumb);

        match converted {
            AbstractView::WindowThumbnail { wid, fallback_icon, style } => {
                assert_eq!(wid, "2");
                assert_eq!(fallback_icon, "app-window");
                assert!(style.is_some(), "style must survive the bridge");
            }
            _ => panic!(
                "convert_view_messages dropped the WindowThumbnail (hit the _ => Empty wildcard) — dock hover thumbnail vanishes in VM mode"
            ),
        }
    }

    /// PLAN-002 A1 教训第五例 fence（PLAN-012 W3）：convert_view_messages
    /// 必须保留 WorkspacePreview——VM 轨切换器分区卡整桌面预览不被 Empty
    /// 通配符吞掉。
    #[test]
    fn test_convert_view_messages_preserves_workspace_preview() {
        let view: AbstractView<DynamicMessage> = AbstractView::WorkspacePreview {
            ws: "0".to_string(),
            fallback_icon: "app-window".to_string(),
            style: crate::ui::style::Style::parse("w-44 h-16 rounded").ok(),
        };

        let converted = convert_view_messages(view);

        match converted {
            AbstractView::WorkspacePreview { ws, fallback_icon, style } => {
                assert_eq!(ws, "0");
                assert_eq!(fallback_icon, "app-window");
                assert!(style.is_some(), "style must survive the bridge");
            }
            _ => panic!(
                "convert_view_messages dropped the WorkspacePreview (hit the _ => Empty wildcard) — 切换器整桌面预览在 VM 轨消失"
            ),
        }
    }

    /// PLAN-002 B：布局件右键穿透消息桥（D-GAP 第四例教训的护栏——新增
    /// View 字段必须在 convert_view_messages 显式接；漏接即静默丢臂）。
    #[test]
    fn test_convert_view_messages_preserves_layout_right_click() {
        let view: AbstractView<DynamicMessage> = AbstractView::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: crate::ui::style::Style::parse("p-2 hover:bg-primary/10").ok(),
            onclick: None,
            on_right_click: Some(DynamicMessage::Typed {
                widget_name: "Desktop".to_string(),
                event_name: "BlankMenu".to_string(),
                args: vec![],
            }),
        };

        let converted = convert_view_messages(view);

        match converted {
            AbstractView::Column {
                on_right_click,
                style,
                ..
            } => {
                assert!(
                    on_right_click.is_some(),
                    "on_right_click must survive the bridge"
                );
                assert!(style.is_some(), "style must survive the bridge");
            }
            _ => panic!("convert_view_messages dropped the Column (hit the _ => Empty wildcard)"),
        }
    }

    /// PLAN-002 B：hover 标志门控——只有声明 `hover:` 变体类的样式才构造
    /// 标志（无声明走零开销原路径）。
    #[test]
    fn test_layout_hover_flag_only_with_hover_variant() {
        let plain = crate::ui::style::Style::parse("w-full p-2 bg-card").ok();
        assert!(
            layout_hover_flag(plain.as_ref()).is_none(),
            "无 hover: 类不应构造标志"
        );

        let hovered = crate::ui::style::Style::parse("w-full p-2 bg-card hover:bg-primary/10").ok();
        assert!(
            layout_hover_flag(hovered.as_ref()).is_some(),
            "hover: 类应构造标志"
        );
        assert!(layout_hover_flag(None).is_none());
    }

    /// PLAN-002 B：样式闭包按标志在 base/hover 两套样式间二选一（hover 类
    /// 经 merged_with_variant 覆盖 base——背景色是 pilot 的主消费面）。
    #[test]
    fn test_layout_style_fn_selects_hover_on_flag() {
        let base_is = IcedStyle::from_style(
            &crate::ui::style::Style::parse("w-full bg-transparent").unwrap(),
        );
        let hover_is = IcedStyle::merged_with_variant(
            &crate::ui::style::Style::parse("w-full bg-transparent hover:bg-primary/10").unwrap(),
            crate::ui::style::Variant::Hover,
        );
        let base_cs = build_container_style(&base_is);
        let hover_cs = build_container_style(&hover_is);
        assert_ne!(
            base_cs.background, hover_cs.background,
            "hover 类应改变背景"
        );

        let flag = crate::ui::iced::hover_area::HoverFlag::default();
        let style_fn = layout_style_fn(base_cs, Some(hover_cs), Some(flag.clone()));
        let theme = iced::Theme::Light;
        assert_eq!(style_fn(&theme), base_cs, "标志未置位 = base");

        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(style_fn(&theme), hover_cs, "标志置位 = hover 合并样式");

        // 无 hover 声明时闭包恒等 base（零行为变化）。
        let plain_fn = layout_style_fn(base_cs, None, None);
        assert_eq!(plain_fn(&theme), base_cs);
    }

    // PLAN-002 N5：vwin 客户区根节点默认底角圆角（round_bottom_root_default）。
    #[test]
    fn test_round_bottom_root_default_pushes_only_when_radius_absent() {
        use crate::ui::style::Style;
        use crate::ui::style::{RoundedSize, StyleClass};
        use crate::ui::view::View;
        // 无 radius 类的 col 根 → 推 rounded-b-2xl(底角 16px 与窗框
        // 对齐;顶部保持方角——2026-09-20 复测裁定,用户明确不要顶角)。
        let mut bare = View::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: Style::parse("h-full bg-background").ok(),
            onclick: None::<()>,
            on_right_click: None,
        };
        round_bottom_root_default(&mut bare);
        let View::Column { style: Some(s), .. } = &bare else {
            panic!("col root must keep its style");
        };
        assert!(
            s.classes
                .contains(&StyleClass::RoundedB(Some(RoundedSize::Xxl))),
            "无 radius 声明的根应获得默认底角"
        );
        assert!(
            !s.classes
                .contains(&StyleClass::RoundedT(Some(RoundedSize::Xxl))),
            "顶部保持方角(PLAN-024 复测裁定)"
        );
        // 应用作者已声明 radius（任意角）→ 不干预。
        let mut authored = View::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: Style::parse("rounded-xl bg-background").ok(),
            onclick: None::<()>,
            on_right_click: None,
        };
        round_bottom_root_default(&mut authored);
        let View::Column { style: Some(s), .. } = &authored else {
            panic!("col root must keep its style");
        };
        assert!(
            !s.classes
                .contains(&StyleClass::RoundedB(Some(RoundedSize::Xxl))),
            "显式 radius 声明优先，不追加默认"
        );
        // 根无 style（透明）与 非 style 化根（text）都不动。
        let mut transparent = View::Column {
            children: vec![],
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None::<()>,
            on_right_click: None,
        };
        round_bottom_root_default(&mut transparent);
        let View::Column { style, .. } = &transparent else {
            panic!("variant unchanged");
        };
        assert!(style.is_none(), "透明根不处理");
    }

    // ── Shell SSE bridge (ash-gui M1) ──────────────────────────────────
    // 验证执行器线程的命令执行 + 事件回流闭环,不依赖 VM/UI。
    // 这覆盖 M1 的 Rust 核心逻辑:队列提交 → std::process 执行 → command_output/
    // command_result 事件经 mpsc channel 产出。VM 侧的预置字段派发由 smoke 测试
    // + 端到端测试(M3,需先修 view_template 不展开 Component 的 MCP 缺陷)覆盖。
    //
    // 注意:执行器用 OnceLock 全局量(SHELL_EXEC_HANDLE / SHELL_EVENT_RX),进程
    // 生命期只初始化一次。测试间共享同一执行器线程 + receiver,故合并到一个测试
    // 函数里顺序提交两条命令,避免全局量二次初始化导致命令丢进无人读的队列。

    #[test]
    fn test_shell_executor_success_and_failure() {
        let rx = start_shell_executor();

        // ── 成功路径:echo ──
        {
            // Plan 060:提交走 vm::shell_bridge(与 shell.at 的
            // auto.shell.exec_submit native 同一队列)。
            auto_lang_core::vm::shell_bridge::submit(
                auto_lang_core::vm::shell_bridge::ShellExecRequest {
                    kind: auto_lang_core::vm::shell_bridge::ShellExecKind::Process,
                    block_id: 42,
                    cmd: "echo hello_m1_bridge".to_string(),
                    cwd: ".".to_string(),
                    result_json: String::new(),
                },
            );
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut got_success = false;
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(ev) if ev.event == "command_result" => {
                    let v: serde_json::Value =
                        serde_json::from_str(&ev.payload_json).expect("result is JSON");
                    assert_eq!(v["block_id"], 42, "success block_id matches");
                    assert_eq!(
                        v["status"], "Success",
                        "echo should succeed: {}",
                        ev.payload_json
                    );
                    got_success = true;
                    break;
                }
                Ok(_) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("executor channel disconnected before success result");
                }
            }
        }
        assert!(got_success, "executor emitted Success result for echo");

        // ── 失败路径:nonexistent command ──
        {
            auto_lang_core::vm::shell_bridge::submit(
                auto_lang_core::vm::shell_bridge::ShellExecRequest {
                    kind: auto_lang_core::vm::shell_bridge::ShellExecKind::Process,
                    block_id: 7,
                    cmd: "nonexistent_cmd_xyz_m1_test".to_string(),
                    cwd: ".".to_string(),
                    result_json: String::new(),
                },
            );
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut got_failed = false;
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(ev) if ev.event == "command_result" => {
                    let v: serde_json::Value =
                        serde_json::from_str(&ev.payload_json).expect("result is JSON");
                    assert_eq!(v["block_id"], 7, "failure block_id matches");
                    assert!(
                        v["status"].is_object(),
                        "nonexistent command should fail: {}",
                        ev.payload_json
                    );
                    assert!(
                        v["status"]["Failed"].is_string(),
                        "Failed variant carries a message"
                    );
                    got_failed = true;
                    break;
                }
                Ok(_) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("executor channel disconnected before failure result");
                }
            }
        }
        assert!(got_failed, "executor emitted Failed result for bad command");
    }
}

// ═══ Plan 057 续(行内编辑)单元测试:Content 纯数据结构,无需窗口 ════
#[cfg(test)]
mod line_edit_tests {
    use super::*;
    // PLAN-018：layout 族修编——UiCode/UiStmt 自 dialect 域模型显式导入
    //（841 迁移后测试门未重编，导入漂移在此补齐）。
    use auto_ui_dialect::{UiCode, UiStmt};

    /// 每个测试独立的 key(避免 TEXTAREA_CONTENTS 全局残留)。
    fn setup(key: &str, text: &str) {
        TEXTAREA_CONTENTS.lock().unwrap().insert(
            key.to_string(),
            Box::leak(Box::new(text_editor::Content::with_text(text))),
        );
        TEXTAREA_GHOSTS
            .lock()
            .unwrap()
            .insert(key.to_string(), String::new());
    }

    fn content_text(key: &str) -> String {
        TEXTAREA_CONTENTS
            .lock()
            .unwrap()
            .get(key)
            .map(|c| c.text())
            .unwrap_or_default()
    }

    fn set_cursor(key: &str, text: &str, off: usize) {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        if let Some(c) = map.get_mut(key) {
            le_move_to_offset(c, text, off);
        }
    }

    fn dummy_on_change() -> IcedMessage {
        IcedMessage {
            widget: "W".into(),
            event: "E".into(),
            input_value: None,
        }
    }

    #[test]
    fn test_cursor_offset_roundtrip() {
        let key = "ut_rt";
        setup(key, "echo hi");
        set_cursor(key, "echo hi", 5);
        let off = {
            let map = TEXTAREA_CONTENTS.lock().unwrap();
            le_cursor_offset(map.get(key).unwrap())
        };
        assert_eq!(off, 5);
    }

    #[test]
    fn test_cursor_offset_multibyte() {
        // 中文字节偏移(3 字节/字符):"你好" 光标在两字之间 = 3
        let key = "ut_cjk";
        setup(key, "你好");
        set_cursor(key, "你好", 3);
        let off = {
            let map = TEXTAREA_CONTENTS.lock().unwrap();
            le_cursor_offset(map.get(key).unwrap())
        };
        assert_eq!(off, 3);
    }

    #[test]
    fn test_kill_to_end() {
        let key = "ut_ke";
        setup(key, "echo hi");
        set_cursor(key, "echo hi", 4); // echo| hi
        let killed = le_kill_motion(key, text_editor::Motion::End);
        assert_eq!(killed.as_deref(), Some(" hi"));
        assert_eq!(content_text(key), "echo");
    }

    #[test]
    fn test_kill_to_home() {
        let key = "ut_kh";
        setup(key, "echo hi");
        set_cursor(key, "echo hi", 4); // |echo hi 前缀删除
        let killed = le_kill_motion(key, text_editor::Motion::Home);
        assert_eq!(killed.as_deref(), Some("echo"));
        assert_eq!(content_text(key), " hi");
    }

    #[test]
    fn test_kill_word_right() {
        let key = "ut_kw";
        setup(key, "echo hi world");
        set_cursor(key, "echo hi world", 4); // echo| hi world
        let killed = le_kill_motion(key, text_editor::Motion::WordRight);
        assert_eq!(killed.as_deref(), Some(" hi"));
        assert_eq!(content_text(key), "echo world");
    }

    #[test]
    fn test_kill_empty_selection_noop() {
        let key = "ut_kn";
        setup(key, "abc");
        set_cursor(key, "abc", 3); // 光标在行尾,kill-to-end 选区为空
        let killed = le_kill_motion(key, text_editor::Motion::End);
        assert_eq!(killed, None);
        assert_eq!(content_text(key), "abc");
    }

    #[test]
    fn test_transpose_mid() {
        let key = "ut_tm";
        setup(key, "ab");
        set_cursor(key, "ab", 1); // 光标在 a b 之间 → 交换 → ba
        le_execute(key, LineEditOp::TransposeChars, &dummy_on_change());
        assert_eq!(content_text(key), "ba");
    }

    #[test]
    fn test_transpose_at_end() {
        let key = "ut_te";
        setup(key, "ab");
        set_cursor(key, "ab", 2); // 行尾 → 交换最后两个 → ba
        le_execute(key, LineEditOp::TransposeChars, &dummy_on_change());
        assert_eq!(content_text(key), "ba");
    }

    #[test]
    fn test_transpose_short_noop() {
        let key = "ut_ts";
        setup(key, "a");
        le_execute(key, LineEditOp::TransposeChars, &dummy_on_change());
        assert_eq!(content_text(key), "a");
    }

    #[test]
    fn test_yank_after_kill() {
        let key = "ut_yk";
        setup(key, "hello world");
        set_cursor(key, "hello world", 5); // hello| world
        le_execute(
            key,
            LineEditOp::Kill(text_editor::Motion::End),
            &dummy_on_change(),
        );
        assert_eq!(content_text(key), "hello");
        le_execute(key, LineEditOp::Yank, &dummy_on_change());
        assert_eq!(content_text(key), "hello world");
    }

    #[test]
    fn test_kill_ring_merge_consecutive() {
        le_push_kill("ab".into());
        le_push_kill("ab".into()); // 相同内容合并
        let len = TEXTAREA_KILL_RING.lock().unwrap().len();
        assert!(len >= 1);
    }

    #[test]
    fn test_delete_forward() {
        let key = "ut_df";
        setup(key, "abc");
        set_cursor(key, "abc", 0);
        le_execute(key, LineEditOp::DeleteForward, &dummy_on_change());
        assert_eq!(content_text(key), "bc");
    }

    #[test]
    fn test_echo_cursor_preserved_on_rebuild() {
        let key = "ut_echo";
        setup(key, "ab");
        set_cursor(key, "ab", 1); // 光标在 a|b
        TEXTAREA_ECHO_KEYS.lock().unwrap().insert(key.to_string());
        // echo 重建:新 value "aZb"(行中插入 Z 的等价外部重算),光标应保持 1→2?
        // 语义:保持偏移(编辑点)。这里 value 变 "aZ" 光标钳 value 区。
        get_textarea_content_rich(key, "aZ", "");
        let off = {
            let map = TEXTAREA_CONTENTS.lock().unwrap();
            le_cursor_offset(map.get(key).unwrap())
        };
        // 重建后光标保持字节偏移 1(echo 语义:编辑点)
        assert_eq!(off, 1);
        assert_eq!(content_text(key), "aZ");
    }

    #[test]
    fn test_external_change_cursor_to_value_end() {
        let key = "ut_ext";
        setup(key, "ab");
        set_cursor(key, "ab", 0);
        // 不置 echo(外部变更):光标钉 value 末尾
        get_textarea_content_rich(key, "abcd", "");
        let off = {
            let map = TEXTAREA_CONTENTS.lock().unwrap();
            le_cursor_offset(map.get(key).unwrap())
        };
        assert_eq!(off, 4);
    }

    #[test]
    fn test_keymap_tables_sanity() {
        let em = line_edit_keymap("emacs");
        for k in [
            "ctrl.a", "ctrl.b", "ctrl.e", "ctrl.f", "ctrl.k", "ctrl.u", "ctrl.y", "ctrl.t",
            "alt.b", "alt.f", "alt.d", "ctrl.w",
        ] {
            assert!(em.contains_key(k), "emacs 缺 {}", k);
        }
        let vi = line_edit_keymap("vi-normal");
        for k in ["h", "l", "0", "$", "w", "b", "e", "x", "i", "a", "A", "d"] {
            assert!(vi.contains_key(k), "vi-normal 缺 {}", k);
        }
        let vii = line_edit_keymap("vi-insert");
        assert!(vii.contains_key("escape"));
        assert!(vii.contains_key("ctrl.k"));
    }

    // ---- Plan 483: VM 双 input 双焦点/键盘双投递(上游 auto-musk 011) ----
    // 042-two-inputs-child / musk 登录页同构:条件渲染子 widget 内两个
    // input(不同 placeholder、各自 on_change)。走 VM 主路径
    // render_dynamic_view(非泛型 IntoIcedElement 路径)构建 Element。

    /// 双 input 列(VM 主路径同构输入)。on_change 挂 IcedMessage,
    /// widget/event 对齐 042 example 的 LoginChild/UserChanged|PassChanged。
        fn p483_two_input_view() -> AbstractView<IcedMessage> {
        let user = AbstractView::Input {
            placeholder: "Enter username".to_string(),
            value: String::new(),
            on_change: Some(IcedMessage {
                widget: "LoginChild".to_string(),
                event: "UserChanged".to_string(),
                input_value: None,
            }),
            on_submit: None,
            width: None,
            password: false,
            style: None,
           disabled: false,
        };
        let pass = AbstractView::Input {
            placeholder: "Enter password".to_string(),
            value: String::new(),
            on_change: Some(IcedMessage {
                widget: "LoginChild".to_string(),
                event: "PassChanged".to_string(),
                input_value: None,
            }),
            on_submit: None,
            width: None,
            password: true,
            style: None,
           disabled: false,
        };
        AbstractView::Column {
            children: vec![user, pass],
            spacing: 8,
            padding: 8,
            style: None,
            onclick: None,
            on_right_click: None,
        }
    }

    /// derive_input_id 契约:主键唯一性 + 跨调用稳定性 + 三元组兜底。
    #[test]
        fn p483_derive_input_id_unique_and_stable() {
        let a = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        let b = derive_input_id(
            Some(("LoginChild", "PassChanged")),
            "Enter password",
            None,
            true,
        );
        assert_ne!(
            format!("{a:?}"),
            format!("{b:?}"),
            "widget+event 主键必须互异"
        );
        // 同参数重复派生稳定(跨重建不变)。
        let a2 = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        assert_eq!(format!("{a:?}"), format!("{a2:?}"));
        // 主键互异优先于同参:两框同 placeholder/width/password 也不撞。
        let c = derive_input_id(Some(("X", "A")), "same", None, false);
        let d = derive_input_id(Some(("X", "B")), "same", None, false);
        assert_ne!(format!("{c:?}"), format!("{d:?}"));
        // 无主键兜底:三元组(泛型路径行为持平)。
        let e = derive_input_id(None, "ph", Some(120), true);
        assert!(format!("{e:?}").contains("auto_input_ph_120_true"));
    }

    /// 红:VM 路径两个 input 的 iced Id 必须互异。iced 的 Focus operation
    /// 对所有同 Id 的 focusable 逐个置焦(focusable.rs)——同 Id 即一次
    /// focus() 全置焦,__focus_input/Tab/refocus 等任一路径触发即双焦点,
    /// 键盘事件随之双投递(042 evidence r4/r5:username="adminadmin")。
    /// 现状(红):两 input 同为 "prompt_input"(13700 整体覆盖)。
    #[test]
        fn p483_vm_two_inputs_have_distinct_iced_ids() {
        use iced::Rectangle;
        use iced_test::selector::{Candidate, Selector};

        let el = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);

        let first: (String, Rectangle) = ui
            .find(|c: Candidate<'_>| match c {
                Candidate::Focusable { id, bounds, .. } => Some((format!("{:?}", id), bounds)),
                _ => None,
            })
            .expect("at least one focusable (username input)");

        let second: String = ui
            .find(|c: Candidate<'_>| match c {
                Candidate::Focusable { id, bounds, .. }
                    if bounds.y > first.1.y + first.1.height =>
                {
                    Some(format!("{:?}", id))
                }
                _ => None,
            })
            .expect("second focusable below the first (password input)");

        assert_ne!(
            first.0, second,
            "VM 主路径两个 input 的 iced Id 必须互异(同 Id ⇒ iced focus \
             operation 一次全置焦 ⇒ 双焦点/键盘双投递,Plan 483)"
        );
    }

    /// Plan 483 T3:042 真实形态端到端(parse→registry→DynamicComponent→
    /// view_with_debug_gated→render_dynamic_view VM 主路径)。镜像
    /// tests::plan437_child_init_tests::build_inline 的生产构建路径;子 widget
    /// 以 kebab tag 直接实例化(免 use 导入),根 widget 持 .authed 条件。
        fn p483_build_login_shape() -> crate::ui::dynamic::DynamicComponent {
        use auto_lang_core::ast::Stmt;
        let src = r##"
widget App {
    model {
        var authed bool = false
    }
    view {
        if .authed != true {
            login-child
        } else {
            text "in"
        }
    }
}

widget LoginChild {
    msg { Submit, UserChanged, PassChanged }
    model {
        var user str = ""
        var pass str = ""
    }
    view {
        col {
            col {
                row {
                    text "TwoInputs"
                }
                col {
                    col {
                        label "Username"
                        input {
                            value: .user
                            oninput: .UserChanged
                            placeholder: "Enter username"
                        }
                    }
                    col {
                        label "Password"
                        input {
                            value: .pass
                            oninput: .PassChanged
                            placeholder: "Enter password"
                        }
                    }
                    button {
                        onclick: .Submit
                        text "Login"
                    }
                }
            }
        }
    }
    on {
        .UserChanged -> { .user = .user }
        .PassChanged -> { .pass = .pass }
        .Submit -> { .authed = true }
    }
}
"##;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let mut root_decl = None;
        let mut view_widget = None;
        let mut child_decls = Vec::new();
        for stmt in &ast.0.items {
            if let auto_parser_api::DomainStmt::Domain(auto_ui_dialect::UiStmt::WidgetDecl(decl)) =
                stmt
            {
                if root_decl.is_none() {
                    root_decl = Some(decl.clone());
                    view_widget = Some(
                        auto_ui_compiler::aura::extract_widget_from_decl(&decl)
                            .map_err(|e| e.to_string())
                            .unwrap(),
                    );
                } else {
                    child_decls.push(decl.clone());
                }
            }
        }
        let root_decl = root_decl.expect("root widget");
        let view_widget = view_widget.expect("view widget");
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        for d in &child_decls {
            let w = auto_ui_compiler::aura::extract_widget_from_decl(d)
                .map_err(|e| e.to_string())
                .unwrap();
            registry.register(w);
        }
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &root_decl,
            &child_decls,
            &view_widget,
            registry,
            Vec::new(),
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component builds")
    }

    /// T3 布局假说:真实 042 形态下两个 input 的 bounds 必须不重叠、
    /// 非零高——重叠则一次点击落双框(双双 is_focused=Some),静态树即
    /// 双投递;这也直接复现/排除「子 widget 拼接形态布局塌陷」。
    #[test]
        fn p483_login_shape_inputs_disjoint_and_delivery_single() {
        use iced_test::selector::Bounded;

        let comp = p483_build_login_shape();
        let (view, _ids, _probe) = comp.view_with_debug_gated(false);
        let view = convert_view_messages(view);
        let el = render_dynamic_view(view, None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);

        let bu = ui
            .find("Enter username")
            .expect("username placeholder")
            .bounds();
        let bp = ui
            .find("Enter password")
            .expect("password placeholder")
            .bounds();
        assert!(bu.height > 0.0, "username input must have height: {bu:?}");
        assert!(bp.height > 0.0, "password input must have height: {bp:?}");
        assert!(
            bp.y >= bu.y + bu.height,
            "inputs must not overlap (click would hit both): user={bu:?} pass={bp:?}"
        );

        ui.click("Enter password").expect("click password");
        let _ = ui.typewrite("admin");
        let msgs: Vec<IcedMessage> = ui.into_messages().collect();
        assert!(!msgs.is_empty(), "typing must produce on_change messages");
        for m in &msgs {
            assert_eq!(
                m.event, "PassChanged",
                "login-shape: keystrokes must only reach the focused input; got {m:?}"
            );
        }
    }

    /// T3 追击:跨「view 重建」的焦点保持。活应用里 view 以 2.5+/s 重建
    /// (desktop 帧泵/心跳),击键之间必然夹着重建;Simulator 单帧静态树
    /// 无法覆盖。这里手工搭迷你仿真:UserInterface + into_cache 重建
    /// (iced daemon 的真机制:Cache 保留 Tree 状态、按槽位 diff)。
    /// 断言:点击 password→键 a→重建(pass="a" 形态)→键 d,全程仅
    /// PassChanged 消息;若 UserChanged 混入=双投递在重建路径复现。
    #[test]
        fn p483_rebuild_between_keystrokes_keeps_single_delivery() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::renderer::Headless;
        use iced_test::runtime::core::{Event, Size};
        use iced_test::runtime::user_interface;

        // 借 Simulator 定位 password 框中心(独立实例,仅取 bounds)。
        let el_probe = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut probe = iced_test::simulator(el_probe);
        use iced_test::selector::Bounded;
        let bp = probe
            .find("Enter password")
            .expect("password bounds")
            .bounds();
        let center = iced::Point::new(bp.x + bp.width / 2.0, bp.y + bp.height / 2.0);

        let mut renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            12.0_f32.into(),
            None,
        ))
        .expect("headless renderer");
        let size = Size::new(1024.0, 768.0);

        let mut msgs: Vec<IcedMessage> = Vec::new();
        fn feed(
            ui: &mut user_interface::UserInterface<'_, IcedMessage, iced::Theme, iced::Renderer>,
            events: &[Event],
            center: iced::Point,
            renderer: &mut iced::Renderer,
            msgs: &mut Vec<IcedMessage>,
        ) {
            let _ = ui.update(
                events,
                iced::mouse::Cursor::Available(center),
                renderer,
                &mut clipboard::Null,
                msgs,
            );
        }

        // 帧 1:双空框;点击 password + 键入 "a"。
        let view_a = p483_two_input_view();
        let el_a = render_dynamic_view(view_a, None, &mut Vec::new());
        let mut ui = user_interface::UserInterface::build(
            el_a,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );
        let mut click_events: Vec<Event> = Vec::new();
        click_events.push(Event::Mouse(iced::mouse::Event::CursorMoved {
            position: center,
        }));
        click_events.extend(iced_test::simulator::click());
        feed(&mut ui, &click_events, center, &mut renderer, &mut msgs);
        let ev_a: Vec<Event> = iced_test::simulator::typewrite("a").collect();
        feed(&mut ui, &ev_a, center, &mut renderer, &mut msgs);

        // 帧 2:重建(pass="a" 形态,结构同产线 dynamic_view 重建)。
        let mut view_b = p483_two_input_view();
        if let AbstractView::Column { children, .. } = &mut view_b {
            if let AbstractView::Input { value, .. } = &mut children[1] {
                *value = "a".to_string();
            }
        }
        let el_b = render_dynamic_view(view_b, None, &mut Vec::new());
        let cache = ui.into_cache();
        let mut ui2 = user_interface::UserInterface::build(el_b, size, cache, &mut renderer);

        // 帧 2:键入 "d" —— 若重建错移焦点,UserChanged 将混入(红)。
        let ev_d: Vec<Event> = iced_test::simulator::typewrite("d").collect();
        feed(&mut ui2, &ev_d, center, &mut renderer, &mut msgs);

        assert!(!msgs.is_empty(), "must produce on_change messages");
        for m in &msgs {
            assert_eq!(
                m.event, "PassChanged",
                "delivery across rebuild must stay single-focus; got {m:?}"
            );
        }
    }

    /// T3 机制级端到端(根因复现):产线触发链 = iced text_input 不捕获
    /// Tab(iced_widget 0.14.2 源码无 Tab 臂)→ 未捕获 Tab 进
    /// keyboard_event_message → FOCUS_PROMPT_EVENT → update 8532 臂
    /// focus(prompt_input_id=="prompt_input") → iced Focus operation 对
    /// **所有**同 Id focusable 置焦。本测试注入同一 focus operation 后
    /// 打字,断言仅单框消费;当前(红):两框同 Id "prompt_input" →
    /// UserChanged 与 PassChanged 齐发(= musk "adminadmin" 双投递)。
    /// 修复后(绿):各框唯一 Id → 聚焦首个 input 的 Id 仅命中一个框。
    #[test]
        fn p483_focus_op_then_typing_stays_single_delivery() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::renderer::Headless;
        use iced_test::runtime::core::{Event, Size};
        use iced_test::runtime::user_interface;
        use iced_test::selector::Bounded;

        let el_probe = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut probe = iced_test::simulator(el_probe);
        let bu = probe
            .find("Enter username")
            .expect("username bounds")
            .bounds();
        let center = iced::Point::new(bu.x + bu.width / 2.0, bu.y + bu.height / 2.0);

        let mut renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            12.0_f32.into(),
            None,
        ))
        .expect("headless renderer");
        let size = Size::new(1024.0, 768.0);
        let mut msgs: Vec<IcedMessage> = Vec::new();

        let el = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut ui = user_interface::UserInterface::build(
            el,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );

        // 产线等价:聚焦路径(未捕获 Tab/__focus_input/召唤,Plan 483 改址
        // 后)以「登记表首个 input 的派生 Id」为目标发出 focus Task,在视图
        // 重建后经 operate 打到树上(iced daemon 同机制)。修复前:首个
        // input 的 Id = 共享 "prompt_input" → 两框齐焦(红);修复后:唯一
        // Id 仅命中 username → 单投递(绿)。
        let mut focus_op = iced_test::runtime::core::widget::operation::focusable::focus(
            iced::widget::Id::new("auto_input_LoginChild_UserChanged"),
        );
        ui.operate(&mut renderer, &mut focus_op);

        let events: Vec<Event> = iced_test::simulator::typewrite("admin").collect();
        let _ = ui.update(
            &events,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut msgs,
        );

        let user_msgs = msgs.iter().filter(|m| m.event == "UserChanged").count();
        let pass_msgs = msgs.iter().filter(|m| m.event == "PassChanged").count();
        assert!(
            user_msgs == 0 || pass_msgs == 0,
            "focus op must not multi-focus: UserChanged x{user_msgs} + PassChanged x{pass_msgs} (双投递,Plan 483)"
        );
    }

    /// 控制组(修复前后都须绿):静态树内点击第二个 input 后键入文本,
    /// 仅聚焦框的 on_change 触发——锁定正常单投递不回归。
    #[test]
        fn p483_click_and_type_delivers_to_focused_input_only() {
        let el = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);

        ui.click("Enter password")
            .expect("password input found by placeholder");
        let _ = ui.typewrite("admin");

        let msgs: Vec<IcedMessage> = ui.into_messages().collect();
        assert!(!msgs.is_empty(), "typing must produce on_change messages");
        for m in &msgs {
            assert_eq!(
                m.event, "PassChanged",
                "keystrokes must only reach the focused input; got {m:?}"
            );
        }
    }

    // ---- Plan 491: VM 轨 Tab/Shift+Tab input 焦点环遍历 ----
    // 483 登记表基建(input_ids DFS 序 + 每框唯一 Id)上的遍历扩展:有
    // input 聚焦时未捕获 Tab → 下一个(尾回环),Shift+Tab → 上一个(首
    // 回环);无聚焦/不在表内 → 登记表首个(= 483 fallback 语义)。机制级
    // 三段:分派(keyboard_event_message 生产纯函数)/求址(focus_traverse)
    // /端到端(点击直聚 + 探针读焦 + 内建 focus operation,复刻 update
    // 臂 operate(FindFocusedInput).then(focus_traverse → focus) 链)。

    /// 未捕获 Tab/Shift+Tab 键盘事件(对齐产线订阅喂给
    /// keyboard_event_message 的形状;status 由调用方传 Ignored)。
        fn p491_tab_event(shift: bool) -> iced::Event {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
            modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: if shift {
                iced::keyboard::Modifiers::SHIFT
            } else {
                iced::keyboard::Modifiers::default()
            },
            repeat: false,
            text: None,
        })
    }

    /// 042 双 input 形态的登记表快照(DFS 序 = 视觉树序:user 在前)。
        fn p491_registry() -> Vec<iced::widget::Id> {
        let mut ids = Vec::new();
        collect_input_ids(&p483_two_input_view(), &mut ids);
        ids
    }

    /// 测试侧聚焦探针:读当前实际持焦 widget 的 Id(与产线
    /// FindFocusedInput 同逻辑;点击直聚的焦点同样可见)。
        fn p491_probe_focus(
        ui: &mut iced_test::runtime::user_interface::UserInterface<
            '_,
            IcedMessage,
            iced::Theme,
            iced::Renderer,
        >,
        renderer: &mut iced::Renderer,
    ) -> Option<iced::widget::Id> {
        struct Probe(Option<iced::widget::Id>);
        impl iced::advanced::widget::Operation<()> for Probe {
            fn focusable(
                &mut self,
                id: Option<&iced::widget::Id>,
                _bounds: iced::Rectangle,
                state: &mut dyn iced::advanced::widget::operation::Focusable,
            ) {
                if state.is_focused() {
                    if let Some(id) = id {
                        self.0 = Some(id.clone());
                    }
                }
            }
            fn traverse(
                &mut self,
                operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation<()>),
            ) {
                operate(self);
            }
        }
        let mut probe = Probe(None);
        ui.operate(renderer, &mut probe);
        probe.0
    }

    /// 端到端遍历链复刻:探针读当前聚焦(含点击直聚)→ focus_traverse
    /// 按登记表求址 → 内建 focus operation 置焦;返回目标 Id。
        fn p491_apply_traversal(
        ui: &mut iced_test::runtime::user_interface::UserInterface<
            '_,
            IcedMessage,
            iced::Theme,
            iced::Renderer,
        >,
        renderer: &mut iced::Renderer,
        ids: &[iced::widget::Id],
        forward: bool,
    ) -> Option<iced::widget::Id> {
        let current = p491_probe_focus(ui, renderer);
        let target = focus_traverse(ids, current.as_ref(), forward)?;
        let mut focus_op =
            iced_test::runtime::core::widget::operation::focusable::focus(target.clone());
        ui.operate(renderer, &mut focus_op);
        Some(target)
    }

    /// 建无头 UserInterface 并点击指定 placeholder 的 input(真实点击直聚)。
    /// `view` 以闭包供给(探针侧与主 UI 各建一份实例);返回 (ui, renderer),
    /// 键入消息由调用方收集。
        fn p491_ui_and_click(
        view: impl Fn() -> AbstractView<IcedMessage>,
        placeholder: &str,
        msgs: &mut Vec<IcedMessage>,
    ) -> (
        iced_test::runtime::user_interface::UserInterface<
            'static,
            IcedMessage,
            iced::Theme,
            iced::Renderer,
        >,
        iced::Renderer,
    ) {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::renderer::Headless;
        use iced_test::runtime::core::{Event, Size};
        use iced_test::runtime::user_interface;
        use iced_test::selector::Bounded;

        // 借一次性 Simulator 定位目标框中心(同 p483 惯例)。
        let el_probe = render_dynamic_view(view(), None, &mut Vec::new());
        let mut probe = iced_test::simulator(el_probe);
        let b = probe
            .find(placeholder)
            .expect("input by placeholder")
            .bounds();
        let center = iced::Point::new(b.x + b.width / 2.0, b.y + b.height / 2.0);
        drop(probe);

        let mut renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            12.0_f32.into(),
            None,
        ))
        .expect("headless renderer");
        let size = Size::new(1024.0, 768.0);
        let el = render_dynamic_view(view(), None, &mut Vec::new());
        let mut ui = user_interface::UserInterface::build(
            el,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );

        let mut click_events: Vec<Event> = vec![Event::Mouse(iced::mouse::Event::CursorMoved {
            position: center,
        })];
        click_events.extend(iced_test::simulator::click());
        let _ = ui.update(
            &click_events,
            iced::mouse::Cursor::Available(center),
            &mut renderer,
            &mut clipboard::Null,
            msgs,
        );
        (ui, renderer)
    }

    /// T1 红①(转绿于 T3/T2):未捕获 Tab(无 shift)分派 → `__focus_next_input`;
    /// 求址 user → pass;端到端:点击 user(真实直聚)→ 遍历链 → pass 持焦,
    /// 键入只进 PassChanged。旧状(红):分派为 `__focus_prompt` 且无
    /// focus_traverse。
    #[test]
        fn p491_tab_next() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::Event;

        // (a) 分派:未捕获 Tab → __focus_next_input
        let msg = keyboard_event_message(
            p491_tab_event(false),
            iced::event::Status::Ignored,
            &std::collections::HashMap::new(),
            true,
        )
        .expect("uncaptured Tab must map to a focus message");
        assert_eq!(
            msg.event, "__focus_next_input",
            "Tab dispatch must be focus-NEXT (Plan 491); got {:?}",
            msg.event
        );

        // (b) 求址:登记表 DFS 序 [user, pass],user 前进 → pass
        let ids = p491_registry();
        let user = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        let pass = derive_input_id(
            Some(("LoginChild", "PassChanged")),
            "Enter password",
            None,
            true,
        );
        assert_eq!(ids.len(), 2, "registry must hold both inputs");
        assert_eq!(
            focus_traverse(&ids, Some(&user), true).as_ref(),
            Some(&pass),
            "Tab from username must address password"
        );

        // (c) 端到端:点击 user → 遍历 → pass 持焦,键入只进 pass
        let mut msgs: Vec<IcedMessage> = Vec::new();
        let (mut ui, mut renderer) =
            p491_ui_and_click(p483_two_input_view, "Enter username", &mut msgs);
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(user),
            "click must focus username (probe must see click-focus)"
        );
        assert_eq!(
            p491_apply_traversal(&mut ui, &mut renderer, &ids, true),
            Some(pass.clone()),
            "traversal from click-focused username must land on password"
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(pass),
            "focus operation must have moved focus to password"
        );

        let evs: Vec<Event> = iced_test::simulator::typewrite("x").collect();
        let _ = ui.update(
            &evs,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut msgs,
        );
        assert!(
            !msgs.is_empty(),
            "typing after traversal must produce messages"
        );
        for m in &msgs {
            assert_eq!(
                m.event, "PassChanged",
                "post-Tab keystrokes must only reach password; got {m:?}"
            );
        }
    }

    /// T1 红②(转绿于 T3/T2):未捕获 Shift+Tab 分派 → `__focus_prev_input`
    /// (旧:Named 臂不加修饰前缀,与 Tab 同臂当前进处理=bug);求址 pass →
    /// user;端到端:点击 pass → 反向遍历 → user 持焦。
    #[test]
        fn p491_shift_tab_prev() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::Event;

        let msg = keyboard_event_message(
            p491_tab_event(true),
            iced::event::Status::Ignored,
            &std::collections::HashMap::new(),
            true,
        )
        .expect("uncaptured Shift+Tab must map to a focus message");
        assert_eq!(
            msg.event, "__focus_prev_input",
            "Shift+Tab dispatch must be focus-PREV (Plan 491); got {:?}",
            msg.event
        );

        let ids = p491_registry();
        let user = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        let pass = derive_input_id(
            Some(("LoginChild", "PassChanged")),
            "Enter password",
            None,
            true,
        );
        assert_eq!(
            focus_traverse(&ids, Some(&pass), false).as_ref(),
            Some(&user),
            "Shift+Tab from password must address username"
        );

        let mut msgs: Vec<IcedMessage> = Vec::new();
        let (mut ui, mut renderer) =
            p491_ui_and_click(p483_two_input_view, "Enter password", &mut msgs);
        assert_eq!(
            p491_apply_traversal(&mut ui, &mut renderer, &ids, false),
            Some(user.clone()),
            "reverse traversal from click-focused password must land on username"
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(user),
            "focus operation must have moved focus to username"
        );

        let evs: Vec<Event> = iced_test::simulator::typewrite("x").collect();
        let _ = ui.update(
            &evs,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut msgs,
        );
        assert!(
            !msgs.is_empty(),
            "typing after traversal must produce messages"
        );
        for m in &msgs {
            assert_eq!(
                m.event, "UserChanged",
                "post-Shift+Tab keystrokes must only reach username; got {m:?}"
            );
        }
    }

    /// T1 红③(转绿于 T2):回环求址——Tab 尾→首,Shift+Tab 首→尾;端到端
    /// 双向实证(点击 pass → 前进回环 user;点击 user → 后退回环 pass)。
    #[test]
        fn p491_wrap() {
        let ids = p491_registry();
        let user = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        let pass = derive_input_id(
            Some(("LoginChild", "PassChanged")),
            "Enter password",
            None,
            true,
        );
        // 尾 → 首(Tab)
        assert_eq!(
            focus_traverse(&ids, Some(&pass), true).as_ref(),
            Some(&user),
            "Tab from last input must wrap to first"
        );
        // 首 → 尾(Shift+Tab)
        assert_eq!(
            focus_traverse(&ids, Some(&user), false).as_ref(),
            Some(&pass),
            "Shift+Tab from first input must wrap to last"
        );

        // 端到端:点击 pass(尾)→ 前进 → 回环 user
        let mut msgs: Vec<IcedMessage> = Vec::new();
        let (mut ui, mut renderer) =
            p491_ui_and_click(p483_two_input_view, "Enter password", &mut msgs);
        assert_eq!(
            p491_apply_traversal(&mut ui, &mut renderer, &ids, true),
            Some(user.clone()),
            "forward traversal from last input must wrap to first"
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(user),
            "wrap-forward must end focused on username"
        );
    }

    /// T2 采集探针单测:产线 FindFocusedInput (a) 恒出值——无聚焦时
    /// finish() 也是 `Some(None)`(内建 find_focused 无聚焦返回
    /// `Outcome::None` 会断 Task::then 链,「无聚焦→聚焦首个」分支依赖
    /// 恒出值);(b) 点击直聚的焦点经 Operation 遍历可读(探针孪生
    /// 结构实证——UserInterface::operate 收 dyn Operation<()>,typed
    /// 产线探针不可直穿,以同逻辑孪生代验,产线侧由 T6 实机代验)。
    #[test]
        fn p491_find_focused_probe() {
        // (a) 恒出值
        match iced::advanced::widget::Operation::finish(&FindFocusedInput::new()) {
            iced::advanced::widget::operation::Outcome::Some(None) => {}
            other => panic!("fresh probe must yield Some(None), got {other:?}"),
        }

        // (b) 点击直聚可读(孪生探针)
        let user = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        let mut msgs: Vec<IcedMessage> = Vec::new();
        let (mut ui, mut renderer) =
            p491_ui_and_click(p483_two_input_view, "Enter username", &mut msgs);
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(user),
            "probe must see click-focus (any focus source lives in the widget Tree)"
        );
    }

    /// T3 锚（预期直接绿，锁 483 fallback 语义防回归）：无 input 聚焦时
    /// Tab/Shift+Tab 同臂回落登记表**首个**——分派事件名分 next/prev，
    /// 但无聚焦求址两者皆首项；端到端无点击直聚 → 遍历 → 首框持焦，
    /// 键入只进 UserChanged（兼锁 483 单投递）。
    #[test]
        fn p491_unfocused_fallback() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::renderer::Headless;
        use iced_test::runtime::core::{Event, Size};
        use iced_test::runtime::user_interface;

        // 分派:无聚焦不改变事件名——next/prev 各自派发(回落语义在求址)
        let msg = keyboard_event_message(
            p491_tab_event(false),
            iced::event::Status::Ignored,
            &std::collections::HashMap::new(),
            true,
        )
        .expect("uncaptured Tab must map to a focus message");
        assert_eq!(msg.event, "__focus_next_input");
        let msg = keyboard_event_message(
            p491_tab_event(true),
            iced::event::Status::Ignored,
            &std::collections::HashMap::new(),
            true,
        )
        .expect("uncaptured Shift+Tab must map to a focus message");
        assert_eq!(msg.event, "__focus_prev_input");

        // 求址:无聚焦 → 两方向皆首项
        let ids = p491_registry();
        let user = derive_input_id(
            Some(("LoginChild", "UserChanged")),
            "Enter username",
            None,
            false,
        );
        assert_eq!(
            focus_traverse(&ids, None, true).as_ref(),
            Some(&user),
            "no-focus Tab must address the FIRST input (483 fallback semantics)"
        );
        assert_eq!(
            focus_traverse(&ids, None, false).as_ref(),
            Some(&user),
            "no-focus Shift+Tab must also address the FIRST input (same arm)"
        );

        // 端到端:无聚焦 → 遍历 → 首框持焦,键入只进 UserChanged
        let mut renderer = iced_test::futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            12.0_f32.into(),
            None,
        ))
        .expect("headless renderer");
        let el = render_dynamic_view(p483_two_input_view(), None, &mut Vec::new());
        let mut ui = user_interface::UserInterface::build(
            el,
            Size::new(1024.0, 768.0),
            user_interface::Cache::default(),
            &mut renderer,
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            None,
            "fresh UI must have no focus"
        );
        assert_eq!(
            p491_apply_traversal(&mut ui, &mut renderer, &ids, true),
            Some(user.clone()),
            "no-focus traversal must land on the first input"
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(user),
            "first input must hold focus after no-focus Tab"
        );

        let mut msgs: Vec<IcedMessage> = Vec::new();
        let evs: Vec<Event> = iced_test::simulator::typewrite("admin").collect();
        let _ = ui.update(
            &evs,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut msgs,
        );
        assert!(!msgs.is_empty(), "typing must produce messages");
        for m in &msgs {
            assert_eq!(
                m.event, "UserChanged",
                "keystrokes must only reach the first input (483 single-delivery); got {m:?}"
            );
        }
    }

    /// 单 input 视图(遍历环长度 1:自环即旧行为,防漂移)。
        fn p491_single_input_view() -> AbstractView<IcedMessage> {
        AbstractView::Input {
            placeholder: "Only box".to_string(),
            value: String::new(),
            on_change: Some(IcedMessage {
                widget: "Solo".to_string(),
                event: "SoloChanged".to_string(),
                input_value: None,
            }),
            on_submit: None,
            width: None,
            password: false,
            style: None,
            disabled: false,
        }
    }

    /// T4 边界锚:单 input 场景 next/prev 取模回环到自身——行为=旧
    /// fallback(聚焦不变),不失焦、不漂移;键入照常单投递。
    #[test]
        fn p491_single_input() {
        use iced_test::runtime::core::clipboard;
        use iced_test::runtime::core::Event;

        let solo = derive_input_id(Some(("Solo", "SoloChanged")), "Only box", None, false);
        let mut ids = Vec::new();
        collect_input_ids(&p491_single_input_view(), &mut ids);
        assert_eq!(ids.len(), 1, "single-input registry");
        // 双向自环(取模回环到自身)
        assert_eq!(
            focus_traverse(&ids, Some(&solo), true).as_ref(),
            Some(&solo),
            "single input: next wraps to itself"
        );
        assert_eq!(
            focus_traverse(&ids, Some(&solo), false).as_ref(),
            Some(&solo),
            "single input: prev wraps to itself"
        );

        // 端到端:点击唯一框 → 前进 → 仍持焦;键入照常送达
        let mut msgs: Vec<IcedMessage> = Vec::new();
        let (mut ui, mut renderer) =
            p491_ui_and_click(p491_single_input_view, "Only box", &mut msgs);
        assert_eq!(
            p491_apply_traversal(&mut ui, &mut renderer, &ids, true),
            Some(solo.clone()),
            "single-input traversal must address itself"
        );
        assert_eq!(
            p491_probe_focus(&mut ui, &mut renderer),
            Some(solo),
            "single input must not lose focus on Tab (no drift)"
        );

        let evs: Vec<Event> = iced_test::simulator::typewrite("x").collect();
        let _ = ui.update(
            &evs,
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut msgs,
        );
        assert!(!msgs.is_empty(), "typing must still produce messages");
        for m in &msgs {
            assert_eq!(
                m.event, "SoloChanged",
                "keystrokes must keep reaching the sole input; got {m:?}"
            );
        }
    }

    /// T4 控制组(修复前后都须绿):聚焦中的 prompt 编辑器 Tab 补全
    /// (onkeydown.tab)是 Captured 事件——根本到不了未捕获 fallback,
    /// Tab/Shift+Tab 遍历臂与 prompt 编辑器捕获路径零交集。
    #[test]
        fn p491_prompt_tab_captured_not_fallback() {
        for shift in [false, true] {
            let msg = keyboard_event_message(
                p491_tab_event(shift),
                iced::event::Status::Captured,
                &std::collections::HashMap::new(),
                true,
            );
            assert!(
                msg.is_none(),
                "Captured Tab(shift={shift}) must not reach the uncaptured fallback; got {msg:?}"
            );
        }
    }

    // ---- Plan 490 G4：VM 轨布局件（row/col/div）onclick 点击 parity ----
    // 用户实测缺陷：028-launcher 候选行（row{onclick:.Launch}）只能键盘选
    // 不能鼠标点——VM 轨转换层不提取布局件 onclick（Vue 轨泛映射已通，
    // ui_gen/vue.rs:12249）= 双端 parity 缺口。端到端自 .at 源构建（沿
    // p483_build_login_shape 生产路径），Simulator 坐标点击 → 断言声明
    // 消息到达。现状（红）：onclick 被静默丢弃，点击零消息。

    /// G4 测试载体：for-loop 行（launcher 形态）+ 无 onclick 锚行 + col/div
    /// 三形态。msg 声明/带参 handler/on 块全沿 028 真实形态。
        fn p490_build_click_shape() -> crate::ui::dynamic::DynamicComponent {
        use auto_lang_core::ast::Stmt;
        let src = r##"
widget ClickApp {
    msg { Pick(str), Launch(str) }
    model {
        var items = ["alpha", "beta"]
        var ranked = [
            { i: 0, name: "notes", title: "Notes" },
            { i: 1, name: "music", title: "Music" }
        ]
        var picked str = ""
    }
    view {
        col {
            for it in .items {
                row {
                    onclick: .Pick(it)
                    text it
                }
            }
            row {
                text "gamma"
            }
            col {
                onclick: .Pick("col")
                text "colbox"
            }
            div {
                onclick: .Pick("div")
                text "divbox"
            }
            for r in .ranked {
                row {
                    onclick: .Launch(r.name)
                    text r.title
                }
            }
        }
    }
    on {
        .Pick(name) -> { .picked = name }
        .Launch(name) -> { .picked = name }
    }
}
"##;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = UiCode(
            parser
                .parse_domain(&auto_ui_dialect::UiDialect)
                .expect("parse"),
        );
        let decls: Vec<auto_ui_dialect::WidgetDecl> = ast
            .domain_stmts()
            .filter_map(|s| match s {
                UiStmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(decls.len(), 1, "single root widget");
        let root_decl = &decls[0];
        let view_widget = auto_ui_compiler::aura::extract_widget_from_decl(root_decl)
            .map_err(|e| e.to_string())
            .unwrap();
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            root_decl,
            &[],
            &view_widget,
            crate::ui::widget_registry::WidgetRegistry::new(),
            Vec::new(),
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component builds")
    }

    /// G4 构建并点击指定文本（点击落在其父布局件的矩形上）→ 收集消息。
        fn p490_click_collect(placeholder: &str) -> Vec<IcedMessage> {
        let comp = p490_build_click_shape();
        let (view, _ids, _probe) = comp.view_with_debug_gated(false);
        let view = convert_view_messages(view);
        let el = render_dynamic_view(view, None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);
        ui.click(placeholder).expect("click target found");
        ui.into_messages().collect()
    }

    /// T4 红①：row 挂 onclick（for-loop 形态，028 候选行同构）——
    /// 点击必须发射 .Pick(it) 消息。现状（红）：零消息。
    #[test]
        fn p490_row_onclick_fires() {
        let msgs = p490_click_collect("alpha");
        assert!(
            msgs.iter()
                .any(|m| m.event.contains("Pick") && m.event.contains("alpha")),
            "row onclick must fire .Pick(\"alpha\") on click; got {msgs:?}"
        );
    }

    /// T4 红②：col 挂 onclick——点击发射 .Pick("col")。
    #[test]
        fn p490_col_onclick_fires() {
        let msgs = p490_click_collect("colbox");
        assert!(
            msgs.iter()
                .any(|m| m.event.contains("Pick") && m.event.contains("col")),
            "col onclick must fire on click; got {msgs:?}"
        );
    }

    /// T4 红③：div（=Container）挂 onclick（musk specs 树形态）——
    /// 点击必须发射 .Pick("div")。
    #[test]
        fn p490_container_div_onclick_fires() {
        let msgs = p490_click_collect("divbox");
        assert!(
            msgs.iter()
                .any(|m| m.event.contains("Pick") && m.event.contains("div")),
            "div/container onclick must fire on click; got {msgs:?}"
        );
    }

    /// T4 锚（绿）：无 onclick 的布局行点击零消息——不引入误触发。
    #[test]
        fn p490_layout_without_onclick_inert() {
        let msgs = p490_click_collect("gamma");
        assert!(
            msgs.is_empty(),
            "plain row without onclick must stay inert; got {msgs:?}"
        );
    }

    /// T4 红④：launcher 真实形态（for r in .ranked → row{onclick:
    /// .Launch(r.name)}，对象列表带参派发）——点击 Notes 行必须发射
    /// .Launch("notes")。
    #[test]
        fn p490_launcher_row_launches() {
        let msgs = p490_click_collect("Notes");
        assert!(
            msgs.iter()
                .any(|m| m.event.contains("Launch") && m.event.contains("notes")),
            "launcher-shaped row must fire .Launch(\"notes\") on click; got {msgs:?}"
        );
        // 第二行参数独立性：Music → notes 之外的参数
        let msgs = p490_click_collect("Music");
        assert!(
            msgs.iter()
                .any(|m| m.event.contains("Launch") && m.event.contains("music")),
            "second launcher row must fire .Launch(\"music\"); got {msgs:?}"
        );
    }
}

/// autodown doc editor 本文字色（暗/亮两档基础灰阶；主题变量接线登记余量）。
fn ade_fg_dark() -> crate::ui::code_editor::theme::Rgba {
    crate::ui::code_editor::theme::Rgba {
        r: 0.92,
        g: 0.93,
        b: 0.95,
        a: 1.0,
    }
}
fn ade_fg_light() -> crate::ui::code_editor::theme::Rgba {
    crate::ui::code_editor::theme::Rgba {
        r: 0.12,
        g: 0.13,
        b: 0.15,
        a: 1.0,
    }
}

// ── PLAN-080 UAT F-UAT-1：CJK 用户气泡 pct-hug 布局 ─────────────────────
// 真机实证（2026-09-21 musk UAT）：中文消息气泡空文本。定罪链：headless
// 同构泡测宽 CJK/latin 均正确（112 vs 86px，本模块 GEOM 回归钉）→ 测宽
// 层无罪；真机缺陷在 v-html 降级链（VM length=字符数/sub=字节切片语义
// 不一致 × musk render_mentions 逐字符扫描），修于 musk 22960ca。
// 本模块保留两笔资产：①pct-hug 路径 CJK/latin 测宽对等回归钉；②bisect
// 文档化测试——MaxWidthPct 子树内的 text 节点对 iced_selector/Find 操作
// 恒不可见（container 候选可见、text 候选缺失，operate traverse 修复无
// 效）——MCP 快照对被包节点的文本上报缺口（§10-11 capped 盒伪影家族）
// 待后续专项（候选通道：MaxWidthPct tag/state 委托引发的 Tree 错位）。
#[cfg(test)]
mod plan080_uat_cjk_bubble_tests {
    use super::*;

    /// musk user_message.at 气泡同构：外层行（max-w-[70%] self-end）>
    /// 泡 col（px-14/py-10 圆角）> 文本。
    fn bubble_view(text: &str) -> AbstractView<IcedMessage> {
        AbstractView::Column {
            children: vec![AbstractView::Column {
                children: vec![AbstractView::Text {
                    content: text.to_string(),
                    style: Style::parse("break-words whitespace-pre-wrap").ok(),
                    selectable: false,
                }],
                spacing: 0,
                padding: 0,
                style: Style::parse("px-[14px] py-[10px] rounded-xl break-words bg-primary").ok(),
                onclick: None,
                on_right_click: None,
            }],
            spacing: 0,
            padding: 0,
            style: Style::parse("flex flex-col gap-[3px] max-w-[70%] self-end items-end").ok(),
            onclick: None,
            on_right_click: None,
        }
    }

    /// F-UAT-2 探针：Rich 段落布局尺寸（容器包 Rich，dump 容器 bounds）。
    #[test]
        fn plan080_uat_fuat2_rich_layout_bounds() {
        for (label, spans) in [
            (
                "latin-rich",
                vec![
                    ("This is a bold paragraph with code.", false),
                    (" tail", false),
                ],
            ),
            ("cjk-rich", vec![("这是中文段落加粗测试。", false)]),
        ] {
            let rich_spans: Vec<crate::ui::view::RichSpanView> = spans
                .into_iter()
                .map(|(c, _)| crate::ui::view::RichSpanView {
                    content: c.to_string(),
                    style: None,
                })
                .collect();
            let el = render_dynamic_view(
                AbstractView::Column {
                    children: vec![AbstractView::Rich {
                        spans: rich_spans,
                        style: None,
                    }],
                    spacing: 0,
                    padding: 0,
                    style: Style::parse("w-full").ok(),
                    onclick: None,
                    on_right_click: None,
                },
                None,
                &mut Vec::new(),
            );
            let mut ui = iced_test::simulator(el);
            let seen = std::sync::Mutex::new(Vec::new());
            let dump = |c: iced_test::selector::Candidate<'_>| -> Option<()> {
                use iced_test::selector::Candidate::*;
                match c {
                    Container { bounds, .. } => {
                        seen.lock().unwrap().push(format!("container@{bounds:?}"));
                        None
                    }
                    Text {
                        bounds, content, ..
                    } => {
                        seen.lock().unwrap().push(format!(
                            "text@{bounds:?}={}",
                            content.chars().take(16).collect::<String>()
                        ));
                        None
                    }
                    _ => None,
                }
            };
            let _ = ui.find(dump);
            eprintln!("[RICH-GEOM] {label:?} -> {:?}", seen.lock().unwrap());
        }
    }

    /// 回归钉：pct-hug 路径 CJK 测宽必须与 latin 同量级（全角 ≈ 2× 半角）。
    /// 读法用闭包收集器（不走 text 选择器——见模块头②）。    /// 回归钉：pct-hug 路径 CJK 测宽必须与 latin 同量级（全角 ≈ 2× 半角）。
    /// 读法用闭包收集器（不走 text 选择器——见模块头②）。
    #[test]
        fn plan080_uat_cjk_bubble_hug_width_not_collapsed() {
        for (label, floor) in [("你好世界早上好", 100.0f32), ("hello world!", 50.0f32)] {
            let el = render_dynamic_view(bubble_view(label), None, &mut Vec::new());
            let mut ui = iced_test::simulator(el);
            let seen = std::sync::Mutex::new(Vec::new());
            let dump = |c: iced_test::selector::Candidate<'_>| -> Option<()> {
                use iced_test::selector::Candidate::*;
                match c {
                    Container { bounds, .. } => {
                        seen.lock().unwrap().push(format!("{bounds:?}"));
                        None
                    }
                    _ => None,
                }
            };
            let _ = ui.find(dump);
            let containers = seen.lock().unwrap();
            let last = containers.last().expect("innermost padding container");
            let width: f32 = last
                .split("width: ")
                .nth(1)
                .expect("width field")
                .split(',')
                .next()
                .expect("width value")
                .trim()
                .parse()
                .expect("width parse");
            assert!(
                width >= floor,
                "F-UAT-1 回归钉：{label:?} 泡宽 {width} < 下限 {floor}（测宽塌缩）"
            );
        }
    }

    /// F-UAT-2 复现探针：autodown 文档（标题+段落+行内 marks）→
    /// simulator 布局，dump 全部候选 bounds——验证 Rich 段落是否零尺寸。
    #[test]
    #[cfg(feature = "autodown")]
    fn plan080_uat_fuat2_rich_paragraph_geometry() {
        let doc = "# Heading

This is a **bold** and *italic* paragraph with `code` and [link](https://x).

这是中文段落，**加粗**测试。
";
        let view = crate::ui::autodown_render::render_document::<IcedMessage>(doc, true);
        let el = render_dynamic_view(view, None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);
        let seen = std::sync::Mutex::new(Vec::new());
        let dump = |c: iced_test::selector::Candidate<'_>| -> Option<()> {
            use iced_test::selector::Candidate::*;
            match c {
                Text {
                    bounds, content, ..
                } => {
                    seen.lock().unwrap().push(format!(
                        "text@{bounds:?}={}",
                        content.chars().take(24).collect::<String>()
                    ));
                }
                Container { bounds, .. } => {
                    seen.lock().unwrap().push(format!("container@{bounds:?}"));
                }
                _ => {}
            }
            None
        };
        let _ = ui.find(dump);
        eprintln!("[F-UAT-2] tree={:?}", seen.lock().unwrap());
        // PLAN-082 T-02 升断言：文档列几何下限——两段落 Rich 各占一行高
        // （~20.8px），列高应 ≥90；Rich 被吞成 Empty（零高 Space）时列高
        // 塌到 heading 附近（~69）。iced_test 对 Rich 无 text 候选（模块头
        // ②观测盲区），几何是本探针可断言的存活信号。
        let max_h = seen
            .lock()
            .unwrap()
            .iter()
            .filter_map(|s| {
                s.strip_prefix("container@")?
                    .split("height: ")
                    .nth(1)?
                    .split(' ')
                    .next()?
                    .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.')
                    .parse::<f32>()
                    .ok()
            })
            .fold(0.0f32, |a, b| a.max(b));
        assert!(
            max_h >= 90.0,
            "文档列高 {max_h} < 90：Rich 段落疑似被吞（Empty 化），tree={:?}",
            seen.lock().unwrap()
        );
    }

    /// PLAN-082 T-02 回归钉：VM 动态主链的 convert_view_messages 必须保
    /// Rich——漏臂曾落 `_ => Empty` 兜底，markdown 段落/表格单元格/引用
    /// 正文在 VM 动态路径整体不可见（080 KNOWN-DEBT ②「表格单元格空文本」
    /// + 081 r6「粗体 CJK 段落不进逻辑树」的统一根因；View 新变体四站点
    /// 显式臂定律的转换链站点）。生产链形态：render_document（Dynamic）
    /// → convert_view_messages（Iced）。
    #[test]
    #[cfg(feature = "autodown")]
    fn plan082_convert_view_messages_preserves_rich() {
        let doc = "中文 **粗体** 与 `码`。\n\n| 名 | 值 |\n| --- | --- |\n| 甲 | 乙 |\n";
        let view = crate::ui::autodown_render::render_document::<DynamicMessage>(doc, true);
        let converted = convert_view_messages(view);
        fn collect(v: &AbstractView<IcedMessage>, rich: &mut Vec<String>, text: &mut Vec<String>) {
            match v {
                AbstractView::Rich { spans, .. } => {
                    rich.push(spans.iter().map(|s| s.content.clone()).collect())
                }
                AbstractView::Text { content, .. } => text.push(content.clone()),
                AbstractView::AnchorSlot { child, .. } => collect(child, rich, text),
                AbstractView::Container { child, .. } => collect(child, rich, text),
                AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
                    children.iter().for_each(|c| collect(c, rich, text))
                }
                AbstractView::Table { headers, rows, .. } => {
                    for h in headers {
                        collect(h, rich, text);
                    }
                    for row in rows {
                        for cell in row {
                            collect(cell, rich, text);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut rich = Vec::new();
        let mut text = Vec::new();
        collect(&converted, &mut rich, &mut text);
        assert!(
            rich.iter().any(|t| t == "中文 粗体 与 码。"),
            "段落 Rich 必须存活且 spans 全文保留，rich={rich:?} text={text:?}"
        );
        for cell in ["名", "值", "甲", "乙"] {
            assert!(
                rich.iter().any(|t| t == cell),
                "表格 cell「{cell}」必须存活（Rich 承载），rich={rich:?}"
            );
        }
        // PLAN-082 review 补强：引用正文/列表项正文与段落同走 render_inlines
        // →Rich→convert 单点（AC-03 离线面）。
        let doc2 = "> 引用正文存活。

- 列表项甲：正文
- 列表项乙：正文
";
        let view2 = crate::ui::autodown_render::render_document::<DynamicMessage>(doc2, true);
        let converted2 = convert_view_messages(view2);
        let mut rich2 = Vec::new();
        let mut text2 = Vec::new();
        collect(&converted2, &mut rich2, &mut text2);
        for piece in ["引用正文存活。", "列表项甲：正文", "列表项乙：正文"] {
            assert!(
                rich2.iter().any(|t| t.contains(piece)) || text2.iter().any(|t| t.contains(piece)),
                "「{piece}」必须存活（引用/列表正文 Rich 承载），rich={rich2:?} text={text2:?}"
            );
        }
    }

    /// PLAN-082 T-05 回归钉：自然宽表格跨行列对齐——同列 cell 的 x 坐标
    /// 必须一致。旧 per-row Row lowering 各行独立 hug（用户截图实证列
    /// 错位：表头「类型/名称/说明」与数据行的列起点逐行漂移）；现在
    /// col_widths/on_col_resize 皆 None 分派 TableAlign 自持整表布局
    /// （每列宽=该列 header+body 最大内容宽）。直构 AbstractView::Table
    /// （cell 用 Text——autodown 链的 cell 是 Rich，iced_test 候选盲区，
    /// 见模块头②；本钉断言 lowering 层的几何）。
    #[test]
        fn plan082_table_columns_align_across_rows() {
        let cell = |s: &str| AbstractView::Text {
            content: s.to_string(),
            style: None,
            selectable: false,
        };
        let view = AbstractView::Table {
            headers: vec![cell("类型"), cell("名称"), cell("说明")],
            rows: vec![
                vec![
                    cell("DIR"),
                    cell("crates/"),
                    cell("Rust workspace 子 crate"),
                ],
                vec![cell("FILE"), cell("README.md"), cell("项目说明")],
            ],
            spacing: 0,
            col_spacing: 8,
            style: None,
            table_key: None,
            col_widths: None,
            on_col_resize: None,
        };
        let el = render_dynamic_view(view, None, &mut Vec::new());
        let mut ui = iced_test::simulator(el);
        let seen = std::sync::Mutex::new(Vec::new());
        let dump = |c: iced_test::selector::Candidate<'_>| -> Option<()> {
            use iced_test::selector::Candidate::*;
            match c {
                Text {
                    bounds, content, ..
                } => {
                    seen.lock()
                        .unwrap()
                        .push(format!("{}@{},{}", content, bounds.x, bounds.y));
                }
                _ => {}
            }
            None
        };
        let _ = ui.find(dump);
        let texts = seen.lock().unwrap();
        let x_of = |label: &str| {
            texts
                .iter()
                .find(|s| s.contains(label))
                .and_then(|s| s.rsplit('@').next())
                .and_then(|xy| xy.split(',').next())
                .and_then(|x| x.parse::<f32>().ok())
                .unwrap_or_else(|| panic!("cell {label:?} not found, texts={texts:?}"))
        };
        // 第 2 列：crates/ 与 README.md 同列 x。
        assert_eq!(
            x_of("crates/"),
            x_of("README.md"),
            "第 2 列跨行 x 必须对齐，texts={texts:?}"
        );
        // 第 3 列：两行说明同列 x，且在名称列右侧。
        let dx1 = x_of("Rust workspace");
        let dx2 = x_of("项目说明");
        assert_eq!(dx1, dx2, "第 3 列跨行 x 必须对齐，texts={texts:?}");
        assert!(dx1 > x_of("crates/"), "说明列必须在名称列右侧");
    }

    /// 文档化（无断言）：MaxWidthPct 子树 text 节点对 Find 操作不可见、
    /// container 可见——见模块头②。
    #[test]
        fn plan080_uat_max_width_pct_operation_visibility_bisect() {
        for (outer, expect_text) in [("flex flex-col", true), ("max-w-[70%]", false)] {
            let el = render_dynamic_view(
                AbstractView::Column {
                    children: vec![AbstractView::Text {
                        content: "bisect text".into(),
                        style: None,
                        selectable: false,
                    }],
                    spacing: 0,
                    padding: 0,
                    style: Style::parse(outer).ok(),
                    onclick: None,
                    on_right_click: None,
                },
                None,
                &mut Vec::new(),
            );
            let mut ui = iced_test::simulator(el);
            let seen_text = std::sync::Mutex::new(false);
            let dump = |c: iced_test::selector::Candidate<'_>| -> Option<()> {
                use iced_test::selector::Candidate::*;
                if matches!(c, Text { .. }) {
                    *seen_text.lock().unwrap() = true;
                }
                None
            };
            let _ = ui.find(dump);
            assert_eq!(
                *seen_text.lock().unwrap(),
                expect_text,
                "outer={outer:?}: text 候选可见性文档化（true=无 pct 对照组）"
            );
        }
    }
}
