//! PLAN-980 NATIVE：ACE 原生 run 通道——001 形真源（legacy `widget` 头经
//! Accept 别名）→ 组件面（免驱动或带驱动）编译 → AV 真值（texts+states）→
//! 现有 View→Iced 投影/窗口服务（`run_app` 泛型宿主），状态写=按当前状态
//! 重建初值重编译重观察（001 的主题切换语义：外部 state 写，非 handler）。
//!
//! 与 HELLO 桥分工：HELLO=单 text 最小首窗；本模块=真实 001 形（model
//! str/bool + 带属性文本叶 style/selectable + design 存在性）+ 状态写驱动的
//! 主题翻转（`style_iced_adapter::set_dark_mode`）+ 关闭重开（进程级，窗口
//! 单例纪律）。结构性 view 树（元素/属性行）经 `project_component_candidates`
//! 消费（DIALECT 已交面）；值经 `observe_component_states_and_texts`。
//!
//! 边界（具名）：路由/页面切换、IME/媒体/终端的**交互**实测为手工/环境门控
//! 档（仓内先例 t5_smoke/plan804）；宿主适配器由
//! `install_ui_host_capabilities` 安装（与旧链同一安装面）。

use crate::ui::ace_state_bridge::AceStateValue;
use crate::ui::{Component, View};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};

/// 一个文本叶的渲染事实（值来自 AV，属性来自源 view 树）。
#[derive(Clone, Debug)]
pub struct NativeTextLeaf {
    pub value: String,
    pub style: Option<String>,
    pub selectable: bool,
}

/// 一次 ACE 原生会话的快照（状态写后重建）。
#[derive(Clone, Debug)]
pub struct NativeSnapshot {
    pub node_name: String,
    pub legacy_widget: bool,
    pub texts: Vec<NativeTextLeaf>,
    pub states: Vec<(String, AceStateValue)>,
    pub dark_mode: bool,
}

/// ACE 原生会话：源装载 + AV 观察 + 状态写（=初值重建）。
#[derive(Debug)]
pub struct AceNativeSession {
    src: String,
    path: String,
    pub snapshot: NativeSnapshot,
    pub frames: std::sync::Arc<AtomicUsize>,
}

fn initials_of(states: &[(String, AceStateValue)]) -> ace_av::age_component::ComponentInitials {
    ace_av::age_component::ComponentInitials {
        per_instance: vec![states
            .iter()
            .filter_map(|(n, v)| {
                use ace_av::age_component::InitialValue;
                let iv = match v {
                    AceStateValue::Int(i) => InitialValue::Int(*i as i64),
                    AceStateValue::Bool(b) => InitialValue::Bool(*b),
                    AceStateValue::Str(s) => InitialValue::Str(s.clone()),
                    AceStateValue::F32(f) => InitialValue::Float(*f),
                };
                Some((n.clone(), iv))
            })
            .collect()],
    }
}

fn observe(
    src: &str,
    path: &str,
    initials: &ace_av::age_component::ComponentInitials,
) -> Result<NativeSnapshot, String> {
    use ace_av::age_component::{
        compile_component_scenario_with_initials, observe_component_states_and_texts,
        project_component_candidates, ComponentScenario, LegacyWidgetAlias,
    };
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let caps = ["ac.runtime.text.v1".to_string()];
    // 免驱动（001 形）或带驱动（有 int 字段）都走同一入口；observed 缺省。
    let compiled = compile_component_scenario_with_initials(
        src,
        path,
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
        None,
        initials,
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let (states, texts) = observe_component_states_and_texts(&compiled)?;

    // 结构面：候选投影的 view 树，按声明序取文本叶属性行（style/selectable）。
    let candidates = project_component_candidates(src).map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let node = candidates
        .first()
        .ok_or_else(|| "no AGE node candidate in source".to_string())?;
    let mut leaf_props: Vec<(Option<String>, bool)> = Vec::new();
    walk_text_props(&node.view.tree, &mut leaf_props);

    let mut leaves = Vec::new();
    for (i, value) in texts
        .first()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
    {
        let (style, selectable) = leaf_props.get(i).cloned().unwrap_or((None, false));
        leaves.push(NativeTextLeaf {
            value,
            style,
            selectable,
        });
    }
    let dark_mode = states
        .per_instance
        .first()
        .map(|r| r.as_slice())
        .unwrap_or(&[])
        .iter()
        .find(|(n, _)| n == "dark_mode")
        .map(|(_, v)| matches!(v, ace_av::age_component::ComponentFieldValue::Bool(true)))
        .unwrap_or(false);
    Ok(NativeSnapshot {
        node_name: node.name.name.clone(),
        legacy_widget: node.legacy_widget,
        texts: leaves,
        states: states
            .per_instance
            .first()
            .map(|row| {
                row.iter()
                    .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                    .collect()
            })
            .unwrap_or_default(),
        dark_mode,
    })
}

/// 属性行的字面值提取（style=StrLit；selectable=BoolLit）。
fn entry_str(e: &ace_core::source::age_component::ComponentViewEntry) -> Option<String> {
    if let ace_core::source::age_component::ComponentEntryValue::Expr(expr) = &e.value {
        if let ace_core::source::ast::ExprKind::StrLit(s) = &expr.kind {
            return Some(s.clone());
        }
    }
    None
}

fn entry_bool(e: &ace_core::source::age_component::ComponentViewEntry) -> Option<bool> {
    if let ace_core::source::age_component::ComponentEntryValue::Expr(expr) = &e.value {
        if let ace_core::source::ast::ExprKind::BoolLit(b) = &expr.kind {
            return Some(*b);
        }
    }
    None
}

/// view 树内按声明序收集文本叶的（style, selectable）属性。
fn walk_text_props(
    items: &[ace_core::source::age_component::ComponentViewItem],
    out: &mut Vec<(Option<String>, bool)>,
) {
    use ace_core::source::age_component::ComponentViewItem;
    for item in items {
        match item {
            ComponentViewItem::Text(_) => out.push((None, false)),
            ComponentViewItem::Element(el) => {
                // 带属性体的文本元素（001 形）：tag=text 时其属性行描述该叶。
                if el.tag == "text" {
                    let mut style = None;
                    let mut selectable = false;
                    for sub in &el.items {
                        if let ComponentViewItem::Entry(e) = sub {
                            match e.key.as_str() {
                                "style" => style = entry_str(e),
                                "selectable" => selectable = entry_bool(e).unwrap_or(false),
                                _ => {}
                            }
                        }
                    }
                    out.push((style, selectable));
                } else {
                    walk_text_props(&el.items, out);
                }
            }
            ComponentViewItem::Entry(_) => {}
        }
    }
}

impl AceNativeSession {
    /// 装载 001 形真源（legacy `widget` 头经 Accept 别名准入）。
    pub fn load(src: &str, path: &str) -> Result<Self, String> {
        let snapshot = observe(src, path, &empty_initials())?;
        Ok(Self {
            src: src.to_string(),
            path: path.to_string(),
            snapshot,
            frames: std::sync::Arc::new(AtomicUsize::new(0)),
        })
    }

    /// 状态写：按当前状态重建初值、覆盖单字段、重编译重观察（001 的外部
    /// state 写语义——主题切换经 dark_mode 到窗口主题线程局部）。
    pub fn write_state_bool(&mut self, name: &str, value: bool) -> Result<(), String> {
        let mut states = self.snapshot.states.clone();
        match states.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = AceStateValue::Bool(value),
            None => return Err(format!("no model field named `{name}` to write")),
        }
        let initials = initials_of(&states);
        self.snapshot = observe(&self.src, &self.path, &initials)?;
        if self.snapshot.dark_mode {
            crate::ui::style_iced_adapter::set_dark_mode(true);
        } else if name == "dark_mode" {
            crate::ui::style_iced_adapter::set_dark_mode(false);
        }
        Ok(())
    }
}

fn empty_initials() -> ace_av::age_component::ComponentInitials {
    ace_av::age_component::ComponentInitials {
        per_instance: vec![Vec::new()],
    }
}

// ---------------------------------------------------------------------------
// 窗口组件（run_app 泛型宿主消费；会话经 RwLock 注入，Default 成立）
// ---------------------------------------------------------------------------

static NATIVE_SESSION: OnceLock<RwLock<Option<AceNativeSession>>> = OnceLock::new();

pub fn install_native_session(session: AceNativeSession) {
    let cell = NATIVE_SESSION.get_or_init(|| RwLock::new(None));
    *cell.write().unwrap() = Some(session);
}

pub fn with_native_session<R>(f: impl FnOnce(&AceNativeSession) -> R) -> Option<R> {
    let cell = NATIVE_SESSION.get()?;
    let guard = cell.read().unwrap();
    guard.as_ref().map(f)
}

pub fn native_session_mut<R>(f: impl FnOnce(&mut AceNativeSession) -> R) -> Option<R> {
    let cell = NATIVE_SESSION.get()?;
    let mut guard = cell.write().unwrap();
    guard.as_mut().map(f)
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AceNativeMsg {
    #[default]
    Idle,
}

/// ACE 原生窗口组件：快照的文本叶按源属性渲染（selectable/style），主题经
/// dark_mode 状态写翻转（style_iced_adapter 线程局部——与旧链同机制）。
#[derive(Debug, Default)]
pub struct AceNativeWindow;

impl Component for AceNativeWindow {
    type Msg = AceNativeMsg;

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<Self::Msg> {
        with_native_session(|s| {
            s.frames.fetch_add(1, Ordering::Relaxed);
            let mut col = View::col().spacing(12).padding(30);
            for leaf in &s.snapshot.texts {
                let text = View::Text {
                    content: leaf.value.clone(),
                    style: leaf
                        .style
                        .as_ref()
                        .map(|st| crate::ui::style::Style::parse(st).expect("source style parses")),
                    selectable: leaf.selectable,
                };
                col = col.child(text);
            }
            col.build()
        })
        .unwrap_or_else(|| {
            View::col()
                .padding(30)
                .child(View::text("ace-native: no session"))
                .build()
        })
    }
}

/// ACE 原生 run 通道入口（`AUTO_UI_ENGINE=ace` 消费）。窗口阻塞至关闭——
/// 交互/自动化驱动经 `examples/ace_native_window.exe`（证据/自动退出/状态写）。
#[cfg(feature = "ace-bridge")]
pub fn run_ace_native_ui(src: &str, path: &str) -> Result<(), String> {
    let session = AceNativeSession::load(src, path)?;
    install_native_session(session);
    crate::ui::iced::run_app::<AceNativeWindow>().map_err(|e| format!("native window: {e:?}"))
}

#[cfg(not(feature = "ace-bridge"))]
pub fn run_ace_native_ui(_src: &str, _path: &str) -> Result<(), String> {
    Err(
        "AUTO_UI_ENGINE=ace requires the ace-bridge feature (build auto-ui-runtime with \
         --features ace-bridge)"
            .to_string(),
    )
}
