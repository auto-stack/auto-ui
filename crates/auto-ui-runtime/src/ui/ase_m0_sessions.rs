//! PLAN-1023 ASE M0：交互会话宿主——两个存活应用会话、真实保留状态与
//! 关闭代次（T-01 冻结的公共接线）。
//!
//! 接线策略（T-01 冻结，两策略比较后的裁定）：
//!
//! - **S-A 双开显示车道被拒**：980 MIN-A-UI 的 `ui_apply_*` 宿主钩子定向
//!   **进程全局 current session**（唯一 setter 是 `ui_open`；无公开切换口）。
//!   两条车道同时 open 时，非选中会话的差量 apply 会写进另一会话的显示
//!   模型——不经 980 owner 改动无法成立，具名放弃。
//! - **S-B 选中车道 + 引擎常驻（本模块采用）**：每个应用会话持有一个
//!   `NodeDesktopBridge`，其 `AceStateEngine` 是该会话**业务状态权威**，
//!   随会话整个生命周期常驻；UI 车道（显示副本）只为**选中**会话打开，
//!   切换 = 关出向车道 + 开目标车道（`open()` 从引擎当前真值重提交显示，
//!   980 已交 reopen 语义）。切换零编译、零业务初始化、零旧 handler 重复
//!   执行；apply 恒落在 current=选中会话。
//!
//! ASE 只管理 app/session/frame 映射与关闭代次：ASE 级关闭撤显示车道并
//!   **释放引擎 owner**（bridge drop）——重开为新代次、种子 0；同 app 双
//!   会话 = 两次独立装载（双 engine，模型互不合并）。代次语义（冻结）：
//!   `generation` = 该 app 的会话序号（每次 open 递增；同 app 双上下文各
//!   自成代，关闭重开必然新代次且高于被关者）。本模块不复制 FFI/
//!   解析/窗口，不修改 980/981 模块。
//!
//! 观察口（T-01 冻结，负控与真实路径共用同一观察语义；全部在真实调用
//! 边界计量，不用假 PASS 字段）：`metrics.loads`（真实装载=编译+引擎挂载，
//! 切换不得增长）、`metrics.ui_lane_opens`（显示车道 open，切换会增长——
//! 与引擎重建可区分）、`metrics.clicks_offered`（注入 980 队列的真实点击）、
//! `dispatches*`（真实 handler 执行计数，`dispatch_log` 同源）。

use crate::ui::ace_node_desktop::{current_session, push_event, NodeDesktopBridge, StepOutcome};
use crate::ui::ace_state_bridge::AceStateValue;
use crate::ui::{Component, View};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};

/// 本 profile 的活会话上限（第三个请求具名 `session.limit` 拒）。
pub const ASE_M0_SESSION_LIMIT: usize = 2;

/// 一个已注册的 ASE 应用：key + 源文件显式路径（权威 fixtures 在 auto-lang
/// `experimental/ase-m0/fixtures/`；UI 只显式路径消费，不复制权威源）。
#[derive(Clone, Debug)]
pub struct AseApp {
    pub key: String,
    pub path: String,
}

/// 保留性观察口（T-01 冻结）：真实调用边界上的计量。真实路径与负控共用
/// 同一结构/语义；`dispatches*` 经 [`AseSessionHost::dispatches_of`] 从引擎
/// `dispatch_log` 派生（真实 handler 执行，不另行自增）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AseMetrics {
    /// `NodeDesktopBridge::load` 次数（每次=一次真实编译+引擎挂载+初值
    /// 观察）。普通切换不得增长（V-07）；负控每切回重建则增长（V-08 红）。
    pub loads: u64,
    /// 显示车道 `open()` 次数（显示副本重提交；与引擎重建可区分的量）。
    pub ui_lane_opens: u64,
    /// 注入 980 事件队列的真实点击数（与 handler 执行数对账）。
    pub clicks_offered: u64,
    /// 跨宿主生命周期的累计真实 handler 执行数（step 边界计数；关闭会话
    /// 的 per-session 记录随 owner 释放，此计数保留全程走势对账）。
    pub dispatches: u64,
}

/// 一条会话生命周期记录（证据面；关闭后保留记录、释放 owner）。
#[derive(Clone, Debug)]
pub struct SessionRecord {
    pub id: usize,
    pub app_key: String,
    /// 该 app 槽位第几代（关闭→重开递增；新代次种子 0）。
    pub generation: u64,
    pub closed: bool,
    /// 最近一次打开的显示车道句柄（关闭后仅作证据；车道已撤）。
    pub last_ui_session: Option<i32>,
}

struct LiveSession {
    record: SessionRecord,
    bridge: NodeDesktopBridge,
    ui_session: Option<i32>,
}

/// ASE 交互会话宿主：app 注册表 + 会话表（活≤2）+ 选中态 + 观察口。
pub struct AseSessionHost {
    apps: Vec<AseApp>,
    live: Vec<LiveSession>,
    closed: Vec<SessionRecord>,
    selected: Option<usize>,
    next_id: usize,
    /// 切换计数（证据面：隐藏/切回次数）。
    pub switches: u64,
    pub metrics: AseMetrics,
    pub log: Vec<String>,
    pub errors: Vec<String>,
}

impl AseSessionHost {
    pub fn new(apps: Vec<AseApp>) -> Self {
        Self {
            apps,
            live: Vec::new(),
            closed: Vec::new(),
            selected: None,
            next_id: 1,
            switches: 0,
            metrics: AseMetrics::default(),
            log: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 已注册应用 key（声明序）。
    pub fn app_keys(&self) -> Vec<&str> {
        self.apps.iter().map(|a| a.key.as_str()).collect()
    }

    /// 活会话 id 列表（打开序）。
    pub fn live_ids(&self) -> Vec<usize> {
        self.live.iter().map(|s| s.record.id).collect()
    }

    /// 当前选中会话（显示车道持有者）。
    pub fn selected_id(&self) -> Option<usize> {
        self.selected
    }

    fn live_idx(&self, id: usize) -> Option<usize> {
        self.live.iter().position(|s| s.record.id == id)
    }

    fn app_path(&self, key: &str) -> Result<PathBuf, String> {
        let app = self.apps.iter().find(|a| a.key == key).ok_or_else(|| {
            format!(
                "unknown app key {key:?} (registered: {:?})",
                self.app_keys()
            )
        })?;
        Ok(PathBuf::from(&app.path))
    }

    /// 打开一个新应用会话：装载（真实编译+引擎挂载，计 `loads`）→ 开显示
    /// 车道（计 `ui_lane_opens`）→ 选为当前。活会话已达上限 → 具名
    /// `session.limit` 拒（现有会话不变）；未知 app 具名拒（零装载）。
    /// 同 app 重复打开 = 双会话（两次独立装载，双 engine，模型不合并）。
    pub fn open_session(&mut self, app_key: &str) -> Result<usize, String> {
        if self.live.len() >= ASE_M0_SESSION_LIMIT {
            let msg = format!(
                "session.limit: {} live sessions held ({}); refuse new open for {app_key:?} \
                 — existing sessions unchanged",
                self.live.len(),
                self.live
                    .iter()
                    .map(|s| format!("{}#{}", s.record.app_key, s.record.id))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            self.errors.push(msg.clone());
            return Err(msg);
        }
        let path = self.app_path(app_key)?;
        let src = std::fs::read_to_string(&path)
            .map_err(|e| format!("read app {app_key} source {}: {e}", path.display()))?;
        self.metrics.loads += 1;
        let mut bridge = NodeDesktopBridge::load(&src, &path.to_string_lossy(), 1)
            .map_err(|e| format!("load refused for {app_key}: {e}"))?;
        // 新会话成为选中：出向车道先撤（引擎常驻，只撤显示副本）。
        self.close_outgoing_lane("open-session")?;
        self.metrics.ui_lane_opens += 1;
        let ui_session = bridge
            .open()
            .map_err(|e| format!("open refused for {app_key}: {e}"))?;
        let generation = self.next_generation_of(app_key);
        let id = self.next_id;
        self.next_id += 1;
        self.log.push(format!(
            "open session {id} app {app_key} gen {generation} ui {ui_session} \
             (loads={}, lane_opens={})",
            self.metrics.loads, self.metrics.ui_lane_opens
        ));
        self.live.push(LiveSession {
            record: SessionRecord {
                id,
                app_key: app_key.to_string(),
                generation,
                closed: false,
                last_ui_session: Some(ui_session),
            },
            bridge,
            ui_session: Some(ui_session),
        });
        self.selected = Some(id);
        Ok(id)
    }

    /// 切换选中：出向车道关闭（引擎保留）→ 目标车道打开（显示从引擎真值
    /// 重提交，980 reopen 语义）。已选中 → no-op。未知/已关闭会话具名拒。
    /// 切换零装载、零业务初始化、零旧 handler 执行（V-07 观察口断言）。
    pub fn select(&mut self, id: usize) -> Result<(), String> {
        if self.selected == Some(id) {
            return Ok(());
        }
        let idx = self
            .live_idx(id)
            .ok_or_else(|| format!("select refused: session {id} is not a live session"))?;
        self.close_outgoing_lane("select")?;
        self.metrics.ui_lane_opens += 1;
        let ui_session = self.live[idx]
            .bridge
            .open()
            .map_err(|e| format!("select refused: reopening lane for {id}: {e}"))?;
        self.live[idx].ui_session = Some(ui_session);
        self.live[idx].record.last_ui_session = Some(ui_session);
        self.switches += 1;
        self.selected = Some(id);
        self.log.push(format!(
            "select session {id} ui {ui_session} (switch #{}, loads={}, lane_opens={})",
            self.switches, self.metrics.loads, self.metrics.ui_lane_opens
        ));
        Ok(())
    }

    /// ASE 级关闭：撤显示车道 + **释放引擎 owner**（bridge drop）→ 重开经
    /// [`Self::open_session`] 为新代次、种子 0。未知/已关闭具名拒。
    pub fn close_session(&mut self, id: usize) -> Result<(), String> {
        let idx = self
            .live_idx(id)
            .ok_or_else(|| format!("close refused: session {id} is not a live session"))?;
        let mut session = self.live.remove(idx);
        if let Err(e) = session.bridge.close_session() {
            let msg = format!("close session {id} lane: {e}");
            self.errors.push(msg.clone());
            return Err(msg);
        }
        session.record.closed = true;
        self.log.push(format!(
            "close session {} app {} gen {} (engine owner released)",
            session.record.id, session.record.app_key, session.record.generation
        ));
        if self.selected == Some(id) {
            self.selected = None;
        }
        self.closed.push(session.record);
        Ok(())
    }

    /// 关闭当前选中者的显示车道（引擎保留；空选中 no-op）。
    fn close_outgoing_lane(&mut self, reason: &'static str) -> Result<(), String> {
        if let Some(id) = self.selected {
            if let Some(idx) = self.live_idx(id) {
                self.live[idx]
                    .bridge
                    .close_session()
                    .map_err(|e| format!("{reason}: closing lane of session {id}: {e}"))?;
                self.live[idx].ui_session = None;
                self.log.push(format!(
                    "{reason}: lane of session {id} closed (engine retained)"
                ));
            }
        }
        Ok(())
    }

    /// 点击选中会话的第 `button_idx` 个按钮（声明序；真实 980 队列注入，
    /// 计 `clicks_offered`）。无选中具名拒。
    pub fn click(&mut self, button_idx: usize) -> Result<(), String> {
        let Some(id) = self.selected else {
            let msg = "click refused: no selected session".to_string();
            self.errors.push(msg.clone());
            return Err(msg);
        };
        // 车道不变式：选中会话的显示车道必须是 980 current（push_event 的
        // 路由目标）。open/select 维护该不变式；此处只注入并计数。
        push_event(0, button_idx);
        self.metrics.clicks_offered += 1;
        self.log.push(format!("click s{id} btn{button_idx}"));
        Ok(())
    }

    /// 受控步进选中会话（AV poll → 真实 handler 执行 → 差量 apply 到当前
    /// 车道）。无选中 → Idle。`Dispatched` 在 step 边界计入累计
    /// `metrics.dispatches`（真实 handler 执行走势）。
    pub fn step_selected(&mut self) -> StepOutcome {
        let Some(id) = self.selected else {
            return StepOutcome::Idle;
        };
        let Some(idx) = self.live_idx(id) else {
            return StepOutcome::Idle;
        };
        let outcome = self.live[idx].bridge.step();
        if matches!(outcome, StepOutcome::Dispatched { .. }) {
            self.metrics.dispatches += 1;
        }
        outcome
    }

    /// 引擎权威状态读回（业务真值；显示模型只是只读镜像）。已关闭会话 →
    /// None（owner 已释放）。
    pub fn engine_states(&self, id: usize) -> Option<Vec<(String, AceStateValue)>> {
        self.live_idx(id)
            .and_then(|idx| self.live[idx].bridge.states(0))
    }

    /// 该会话真实 handler 执行计数（引擎 `dispatch_log` 同源）。
    pub fn dispatches_of(&self, id: usize) -> Option<usize> {
        self.live_idx(id)
            .map(|idx| self.live[idx].bridge.dispatch_log.len())
    }

    /// 全部活会话 handler 执行总数。
    pub fn dispatches(&self) -> usize {
        self.live.iter().map(|s| s.bridge.dispatch_log.len()).sum()
    }

    /// 会话当前显示车道句柄（仅选中会话非 None）。
    pub fn ui_session_of(&self, id: usize) -> Option<i32> {
        self.live_idx(id).and_then(|idx| self.live[idx].ui_session)
    }

    /// 会话最近一次显示车道句柄（含隐藏会话——980 HOST 保留已关车道的
    /// 显示模型快照，作显示面证据；业务真值仍以 [`Self::engine_states`]
    /// 为准）。
    pub fn last_lane_of(&self, id: usize) -> Option<i32> {
        self.live_idx(id)
            .and_then(|idx| self.live[idx].record.last_ui_session)
    }

    /// 该 app 的下一代次号（同 app 历史会话数 + 1；关闭重开递增）。
    fn next_generation_of(&self, app_key: &str) -> u64 {
        let live_gens = self
            .live
            .iter()
            .filter(|s| s.record.app_key == app_key)
            .count() as u64;
        let closed_gens = self.closed.iter().filter(|r| r.app_key == app_key).count() as u64;
        live_gens + closed_gens + 1
    }

    /// 证据 JSON（CPU 测试/窗口 exe 消费）：app 表、活/已关会话、选中、
    /// 观察口计量、生命周期日志、错误。
    pub fn evidence(&self) -> serde_json::Value {
        let live: Vec<serde_json::Value> = self
            .live
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.record.id,
                    "app": s.record.app_key,
                    "generation": s.record.generation,
                    "closed": s.record.closed,
                    "ui_session": s.ui_session,
                    "dispatches": s.bridge.dispatch_log.len(),
                    "engine_states": s.bridge.states(0).map(|row| row.into_iter()
                        .map(|(n, v)| (n, format!("{v:?}"))).collect::<Vec<_>>()),
                })
            })
            .collect();
        let closed: Vec<serde_json::Value> = self
            .closed
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id, "app": r.app_key, "generation": r.generation,
                    "closed": r.closed, "last_ui_session": r.last_ui_session,
                })
            })
            .collect();
        serde_json::json!({
            "apps": self.apps.iter().map(|a| serde_json::json!({
                "key": a.key, "path": a.path,
            })).collect::<Vec<_>>(),
            "live": live,
            "closed": closed,
            "selected": self.selected,
            "switches": self.switches,
            "metrics": {
                "loads": self.metrics.loads,
                "ui_lane_opens": self.metrics.ui_lane_opens,
                "clicks_offered": self.metrics.clicks_offered,
                "dispatches_total": self.metrics.dispatches,
                "dispatches_live": self.dispatches(),
            },
            "log": self.log,
            "errors": self.errors,
        })
    }
}

// ---------------------------------------------------------------------------
// 窗口组件（T-03 tab/frame 绑定）：app 打开行 + 会话 tab 行 + frame 区
// （选中会话显示模型 + 真实按钮点击入 980 队列）
// ---------------------------------------------------------------------------

static ASE_SESSIONS_HOST: OnceLock<RwLock<Option<AseSessionHost>>> = OnceLock::new();
/// ASE 会话窗口渲染帧计数（真实渲染观察面）。
pub static ASE_M0_SESSIONS_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// 安装 ASE 会话宿主（窗口进程一次；本槽独占会话/车道轮换）。
pub fn install_ase_sessions_host(host: AseSessionHost) {
    let cell = ASE_SESSIONS_HOST.get_or_init(|| RwLock::new(None));
    *cell.write().unwrap() = Some(host);
}

/// 访问 ASE 会话宿主（窗口 tick/按钮与收口线程共用）。
pub fn with_ase_sessions_host<R>(f: impl FnOnce(&mut AseSessionHost) -> R) -> Option<R> {
    let cell = ASE_SESSIONS_HOST.get()?;
    let mut guard = cell.write().ok()?;
    guard.as_mut().map(f)
}

#[derive(Clone, Debug)]
pub enum AseSessionsMsg {
    /// 打开 app 的新会话（app 注册序索引）。
    OpenApp(usize),
    /// 切换选中会话（会话 id）。
    Select(usize),
    /// ASE 级关闭会话（会话 id；释放引擎 owner）。
    CloseSession(usize),
    /// frame 内业务按钮（声明序 target；真实 980 队列注入）。
    FrameClick(usize),
    SessionTick,
}

/// ASE M0 交互会话窗口：单真实宿主 + tab 行 + frame 区。frame 区渲染**选中**
/// 会话的显示模型（全局 HOST 当前 session 的 AV 真值）；业务按钮经
/// [`push_event`] 入真实 typed 队列（与脚本注入同一边界）；受控步进只在
/// 窗口 tick 主线程。无选中会话 → 空 frame。
#[derive(Debug, Default)]
pub struct AseM0SessionsWindow;

impl Component for AseM0SessionsWindow {
    type Msg = AseSessionsMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AseSessionsMsg::OpenApp(idx) => {
                let key = with_ase_sessions_host(|h| h.app_keys().get(idx).map(|k| k.to_string()))
                    .flatten();
                if let Some(key) = key {
                    // 拒绝/错误进 host.errors（view 错误行展示；具名可辨）。
                    let _ = with_ase_sessions_host(|h| h.open_session(&key));
                }
            }
            AseSessionsMsg::Select(id) => {
                let _ = with_ase_sessions_host(|h| h.select(id));
            }
            AseSessionsMsg::CloseSession(id) => {
                let _ = with_ase_sessions_host(|h| h.close_session(id));
            }
            AseSessionsMsg::FrameClick(target) => {
                let _ = with_ase_sessions_host(|h| h.click(target));
            }
            AseSessionsMsg::SessionTick => {
                with_ase_sessions_host(|h| h.step_selected());
            }
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(20)
    }

    fn tick_msg(&self) -> Option<Self::Msg> {
        Some(AseSessionsMsg::SessionTick)
    }

    fn view(&self) -> View<Self::Msg> {
        ASE_M0_SESSIONS_FRAMES.fetch_add(1, Ordering::Relaxed);
        let (app_keys, live, selected, error) = with_ase_sessions_host(|h| {
            (
                h.app_keys()
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>(),
                h.live_ids(),
                h.selected_id(),
                h.errors.last().cloned(),
            )
        })
        .unwrap_or((Vec::new(), Vec::new(), None, None));

        let mut root = View::col().spacing(10).padding(16);
        // app 打开行。
        let mut apps_row = View::row().spacing(8);
        for (idx, key) in app_keys.iter().enumerate() {
            apps_row = apps_row.child(View::button_styled(
                format!("打开{key}"),
                AseSessionsMsg::OpenApp(idx),
                "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground",
            ));
        }
        root = root.child(apps_row.build());
        if let Some(error) = error {
            root = root.child(View::text(format!("error: {error}")));
        }

        // 会话 tab 行：选中高亮；每 tab 可关闭（ASE 级，释放引擎 owner）。
        let mut tabs = View::row().spacing(8);
        for id in &live {
            let is_selected = selected == Some(*id);
            let style = if is_selected {
                "px-3 py-1.5 rounded-lg bg-accent text-accent-foreground"
            } else {
                "px-3 py-1.5 rounded-lg bg-muted text-foreground"
            };
            tabs = tabs.child(View::button_styled(
                format!("会话{id}{}", if is_selected { "●" } else { "" }),
                AseSessionsMsg::Select(*id),
                style,
            ));
            tabs = tabs.child(View::button_styled(
                format!("关{id}"),
                AseSessionsMsg::CloseSession(*id),
                "px-2 py-1.5 rounded-lg bg-destructive text-destructive-foreground",
            ));
        }
        root = root.child(tabs.build());

        // frame 区：选中会话显示模型（AV 真值经 ui980 FFI 提交）+ 业务按钮
        // （点击=真实 typed 队列注入）。无活动 → 空 frame。
        if let Some((texts, states, buttons, generation)) = current_session(|s| {
            (
                s.texts.clone(),
                s.states.clone(),
                s.buttons.clone(),
                s.generation,
            )
        }) {
            let mut frame = View::col().spacing(6).padding(12);
            frame = frame.child(View::text_styled(
                &format!("frame session generation {generation}"),
                "text-sm",
            ));
            for i in 0..texts.len().max(states.len()) {
                for (leaf, text) in texts.get(i).into_iter().flatten().enumerate() {
                    frame = frame.child(View::text(format!("t{leaf}: {text}")));
                }
                for (_slot, (name, value)) in states.get(i).into_iter().flatten().enumerate() {
                    frame = frame.child(View::text(format!("{name} = {value}")));
                }
            }
            let mut row = View::row().spacing(8);
            for (target, (label, _msg)) in buttons.iter().enumerate() {
                row = row.child(View::button_styled(
                    label.clone(),
                    AseSessionsMsg::FrameClick(target),
                    "px-3 py-1.5 rounded-lg bg-secondary text-secondary-foreground",
                ));
            }
            frame = frame.child(row.build());
            root = root.child(frame.build());
        }
        root.build()
    }
}
