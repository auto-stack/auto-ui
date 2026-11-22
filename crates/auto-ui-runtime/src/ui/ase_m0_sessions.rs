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

use crate::ui::ace_node_desktop::{
    current_session, host_session_text_applies, push_event, NodeDesktopBridge, StepOutcome,
};
use crate::ui::ace_state_bridge::{AceStateValue, EngineExecutionAudit};
use crate::ui::{Component, View};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CLOCK_PERIOD: Duration = Duration::from_secs(1);
const UTC_DAY_SECONDS: i32 = 86_400;

/// One deadline tied to the session, its generation, and one visibility interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockWake {
    session: usize,
    generation: u64,
    visibility: u64,
    deadline: Instant,
}

struct VisibleClock {
    wake: ClockWake,
    pending: bool,
    ready: bool,
}

#[derive(Default)]
struct ClockMetrics {
    samples: u64,
    delivered: u64,
    stale_wakes: u64,
    rejected_samples: u64,
}

/// Real wall-clock time in UTC seconds since midnight.
fn sample_wall_clock() -> Result<i32, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| (time.as_secs() % UTC_DAY_SECONDS as u64) as i32)
        .map_err(|e| format!("system time before Unix epoch: {e}"))
}

/// 本 profile 的活会话上限（第三个请求具名 `session.limit` 拒）。
pub const ASE_M0_SESSION_LIMIT: usize = 2;

/// 一个已注册的 ASE 应用：key + 源文件显式路径（权威 fixtures 在 auto-lang
/// `experimental/ase-m0/fixtures/`；UI 只显式路径消费，不复制权威源）。
#[derive(Clone, Debug)]
pub struct AseApp {
    pub key: String,
    pub path: String,
}

/// 分段打开票据（PLAN-1118）：一次已完成的 `NodeDesktopBridge::load`，等待
/// 挂载。未挂载即 drop → 引擎 owner 释放，零 session/零 UI 效果。
pub struct AseOpenTicket {
    pub(crate) app_key: String,
    pub(crate) bridge: NodeDesktopBridge,
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

/// 会话脚本动作（e2e exe 与 CPU 测试同一边界；窗口按钮走同一 on 消息面）。
#[derive(Clone, Debug)]
pub enum AseSessionAction {
    Open(String),
    Select(usize),
    Close(usize),
    /// frame 按钮（声明序 target；与窗口点击同一 typed 队列）。
    Click(usize),
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
    retired_audit: Option<EngineExecutionAudit>,
}

struct LiveSession {
    record: SessionRecord,
    bridge: NodeDesktopBridge,
    ui_session: Option<i32>,
    clock: bool,
}

/// ASE 交互会话宿主：app 注册表 + 会话表（活≤2）+ 选中态 + 观察口。
pub struct AseSessionHost {
    apps: Vec<AseApp>,
    live: Vec<LiveSession>,
    closed: Vec<SessionRecord>,
    selected: Option<usize>,
    next_id: usize,
    clock_app: Option<String>,
    visibility: u64,
    clock: Option<VisibleClock>,
    clock_metrics: ClockMetrics,
    clock_lanes: Vec<i32>,
    /// 切换计数（证据面：隐藏/切回次数）。
    pub switches: u64,
    pub metrics: AseMetrics,
    pub log: Vec<String>,
    pub errors: Vec<String>,
    script: Vec<(usize, AseSessionAction)>,
    tick_no: usize,
}

impl AseSessionHost {
    pub fn new(apps: Vec<AseApp>) -> Self {
        Self {
            apps,
            live: Vec::new(),
            closed: Vec::new(),
            selected: None,
            next_id: 1,
            clock_app: None,
            visibility: 0,
            clock: None,
            clock_metrics: ClockMetrics::default(),
            clock_lanes: Vec::new(),
            switches: 0,
            metrics: AseMetrics::default(),
            log: Vec::new(),
            errors: Vec::new(),
            script: Vec::new(),
            tick_no: 0,
        }
    }

    /// Register the app that receives the host's one-second Tick(int) samples.
    pub fn with_clock_app(mut self, app_key: String) -> Result<Self, String> {
        self.app_path(&app_key)?;
        self.clock_app = Some(app_key);
        Ok(self)
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
        let ticket = self.begin_open(app_key)?;
        self.mount_open(ticket)
    }

    /// 分段打开第一步（PLAN-1118）：容量速检 + 真实装载（编译+引擎挂载，
    /// 计 `loads`）→ 返回待挂载票据。票据未挂载即 drop 时引擎 owner 随之
    /// 释放、零 session/零 UI 效果。容量在挂载点还有一次权威复检（分段间
    /// 可能有并发 open 落位）。
    pub fn begin_open(&mut self, app_key: &str) -> Result<AseOpenTicket, String> {
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
        let bridge = NodeDesktopBridge::load(&src, &path.to_string_lossy(), 1)
            .map_err(|e| format!("load refused for {app_key}: {e}"))?;
        if self.clock_app.as_deref() == Some(app_key) && !bridge.accepts_clock_tick() {
            return Err(format!(
                "clock app {app_key:?} must declare a Tick(int) handler"
            ));
        }
        Ok(AseOpenTicket {
            app_key: app_key.to_string(),
            bridge,
        })
    }

    /// 分段打开第二步（PLAN-1118）：权威容量复检 → 出向车道撤（引擎常驻）
    /// → 开车道 → 记录/选中。容量不足 → 具名 `session.limit` 拒，票据
    /// drop（零 session、零 UI 效果）。
    pub fn mount_open(&mut self, ticket: AseOpenTicket) -> Result<usize, String> {
        if self.live.len() >= ASE_M0_SESSION_LIMIT {
            let msg = format!(
                "session.limit: {} live sessions held ({}); refuse mount of {:?} \
                 — ticket dropped, existing sessions unchanged",
                self.live.len(),
                self.live
                    .iter()
                    .map(|s| format!("{}#{}", s.record.app_key, s.record.id))
                    .collect::<Vec<_>>()
                    .join(", "),
                ticket.app_key
            );
            drop(ticket);
            self.errors.push(msg.clone());
            return Err(msg);
        }
        let AseOpenTicket {
            app_key,
            mut bridge,
        } = ticket;
        let clock = self.clock_app.as_deref() == Some(app_key.as_str());
        // 新会话成为选中：出向车道先撤（引擎常驻，只撤显示副本）。
        self.close_outgoing_lane("open-session")?;
        self.metrics.ui_lane_opens += 1;
        let ui_session = if clock {
            bridge.open_awaiting_event()
        } else {
            bridge.open()
        }
        .map_err(|e| format!("open refused for {app_key}: {e}"))?;
        let generation = self.next_generation_of(&app_key);
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
                app_key: app_key.clone(),
                generation,
                closed: false,
                last_ui_session: Some(ui_session),
                retired_audit: None,
            },
            bridge,
            ui_session: Some(ui_session),
            clock,
        });
        self.activate(id, generation, clock);
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
        let session = &mut self.live[idx];
        let ui_session = if session.clock {
            session.bridge.open_awaiting_event()
        } else {
            session.bridge.open()
        }
        .map_err(|e| format!("select refused: reopening lane for {id}: {e}"))?;
        session.ui_session = Some(ui_session);
        session.record.last_ui_session = Some(ui_session);
        let generation = session.record.generation;
        let clock = session.clock;
        self.switches += 1;
        self.activate(id, generation, clock);
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
        if self.selected == Some(id) {
            self.close_outgoing_lane("close-session")?;
        } else {
            self.live[idx].bridge.close_session()?;
        }
        self.retire_session(idx);
        Ok(())
    }

    /// 撤销选中而不关闭会话（PLAN-1118 外壳：切到无 guest 的页签——关出向
    /// 显示车道并停时钟资格，**引擎常驻**；已无选中 → no-op）。后台页签
    /// 由此获得"隐藏零业务/零呈现"语义（Clock 隐藏期零派发）。
    pub fn deselect(&mut self) -> Result<(), String> {
        if self.selected.is_none() && self.clock.is_none() {
            return Ok(());
        }
        self.close_outgoing_lane("deselect")?;
        Ok(())
    }

    fn retire_session(&mut self, idx: usize) {
        let mut session = self.live.remove(idx);
        session.record.retired_audit = session.bridge.close_engine();
        session.record.closed = true;
        self.log.push(format!(
            "close session {} app {} gen {} (engine owner released)",
            session.record.id, session.record.app_key, session.record.generation
        ));
        self.closed.push(session.record);
    }

    /// Cancel the selection before closing its display lane. A failed close keeps
    /// the lane handle for a later retry, without granting it event ownership.
    fn close_outgoing_lane(&mut self, reason: &'static str) -> Result<(), String> {
        self.clock = None;
        self.selected = None;
        if let Some(session) = self
            .live
            .iter_mut()
            .find(|session| session.ui_session.is_some())
        {
            let id = session.record.id;
            session
                .bridge
                .close_session()
                .map_err(|error| format!("{reason}: closing lane of session {id}: {error}"))?;
            session.ui_session = None;
            self.log.push(format!(
                "{reason}: lane of session {id} closed (engine retained)"
            ));
        }
        Ok(())
    }

    fn activate(&mut self, id: usize, generation: u64, clock: bool) {
        self.selected = Some(id);
        if clock {
            self.visibility += 1;
            if let Some(lane) = self.ui_session_of(id) {
                if self.clock_lanes.last() != Some(&lane) {
                    self.clock_lanes.push(lane);
                }
            }
            self.clock = Some(VisibleClock {
                wake: ClockWake {
                    session: id,
                    generation,
                    visibility: self.visibility,
                    deadline: Instant::now(),
                },
                pending: false,
                ready: false,
            });
        }
    }

    /// Close all sessions after the native window loop returns.
    pub fn close_all(&mut self) -> Result<(), String> {
        self.clock = None;
        self.selected = None;
        let mut failure = None;
        while let Some(session) = self.live.first_mut() {
            if let Err(error) = session.bridge.close_for_shutdown() {
                let error = format!("close session {} lane: {error}", session.record.id);
                self.errors.push(error.clone());
                failure.get_or_insert(error);
            }
            self.retire_session(0);
        }
        failure.map_or(Ok(()), Err)
    }

    pub fn clock_wake(&self) -> Option<ClockWake> {
        self.clock.as_ref().map(|clock| clock.wake)
    }

    /// Offer a due host wake to the one-slot queue. Duplicate offers coalesce.
    pub fn queue_clock_wake(&mut self, wake: ClockWake, now: Instant) -> bool {
        let Some(clock) = self.clock.as_mut().filter(|clock| clock.wake == wake) else {
            self.clock_metrics.stale_wakes += 1;
            return false;
        };
        if clock.pending || now < wake.deadline {
            return false;
        }
        clock.pending = true;
        true
    }

    /// Validate the selected owner again immediately before sampling and dispatch.
    pub fn dispatch_clock_wake(
        &mut self,
        now: Instant,
        sample: impl FnOnce() -> Result<i32, String>,
    ) -> StepOutcome {
        let Some(wake) = self
            .clock
            .as_ref()
            .filter(|clock| clock.pending)
            .map(|clock| clock.wake)
        else {
            return StepOutcome::Idle;
        };
        let Some(idx) = self.live_idx(wake.session).filter(|&idx| {
            let owner = &self.live[idx];
            self.selected == Some(wake.session)
                && owner.record.generation == wake.generation
                && owner.bridge.can_dispatch_to_current_lane()
        }) else {
            self.clock = None;
            if self.selected == Some(wake.session) {
                self.selected = None;
            }
            self.clock_metrics.stale_wakes += 1;
            return StepOutcome::Idle;
        };

        let clock = self
            .clock
            .as_mut()
            .expect("wake belongs to the active clock");
        clock.pending = false;
        // Schedule from the current host turn. Missed intervals are never replayed.
        clock.wake.deadline = now + CLOCK_PERIOD;
        self.clock_metrics.samples += 1;
        let sample = match sample() {
            Ok(seconds) if (0..UTC_DAY_SECONDS).contains(&seconds) => seconds,
            other => {
                self.clock_metrics.rejected_samples += 1;
                return StepOutcome::DispatchRefused(format!("clock sample refused: {other:?}"));
            }
        };

        let bridge = &mut self.live[idx].bridge;
        let prior_errors = bridge.errors.len();
        let outcome = bridge.dispatch_typed(0, "Tick", &[AceStateValue::Int(sample)]);
        let present_errors = &bridge.errors[prior_errors..];
        self.errors.extend_from_slice(present_errors);
        if matches!(outcome, StepOutcome::Dispatched { .. }) {
            self.metrics.dispatches += 1;
            if let Some(error) = present_errors.first() {
                let error = format!("clock frame publication failed: {error}");
                self.clock = None;
                self.selected = None;
                return StepOutcome::LaneError(error);
            }
            clock.ready = true;
            self.clock_metrics.delivered += 1;
        }
        outcome
    }

    pub fn clock_queue_depth(&self) -> usize {
        usize::from(self.clock.as_ref().is_some_and(|clock| clock.pending))
    }

    pub fn execution_audit_of(&self, id: usize) -> Option<EngineExecutionAudit> {
        if let Some(idx) = self.live_idx(id) {
            return self.live[idx].bridge.execution_audit();
        }
        self.closed
            .iter()
            .find(|r| r.id == id)?
            .retired_audit
            .clone()
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
    /// `metrics.dispatches`（真实 handler 执行走势）。到期脚本先注入（tick
    /// 从 1 起；与 980/1022 脚本纪律同构——窗口按钮与脚本同一 on 消息面）。
    pub fn step_selected(&mut self) -> StepOutcome {
        self.tick_no += 1;
        self.run_due_script();
        let now = Instant::now();
        if let Some(wake) = self.clock_wake() {
            self.queue_clock_wake(wake, now);
            let outcome = self.dispatch_clock_wake(now, sample_wall_clock);
            if !matches!(outcome, StepOutcome::Idle) {
                return outcome;
            }
        }
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

    /// 安装会话脚本（tick 从 1 起；e2e exe 与 CPU 测试同一边界）。
    pub fn install_script(&mut self, script: Vec<(usize, AseSessionAction)>) {
        self.script = script;
    }

    /// 脚本是否已全部注入（e2e 收口观察面）。
    pub fn script_done(&self) -> bool {
        self.script.iter().all(|(t, _)| *t <= self.tick_no)
    }

    fn run_due_script(&mut self) {
        let due: Vec<AseSessionAction> = self
            .script
            .iter()
            .filter(|(t, _)| *t == self.tick_no)
            .map(|(_, a)| a.clone())
            .collect();
        for action in due {
            let result = match &action {
                AseSessionAction::Open(key) => self.open_session(key).map(|_| ()),
                AseSessionAction::Select(id) => self.select(*id),
                AseSessionAction::Close(id) => self.close_session(*id),
                AseSessionAction::Click(target) => self.click(*target),
            };
            if let Err(e) = result {
                self.errors.push(format!("script {action:?}: {e}"));
            }
        }
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
        if self.selected != Some(id) {
            return None;
        }
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
                    "engine_audit": r.retired_audit.as_ref().map(|audit| serde_json::json!({
                        "compile_count": audit.compile_count,
                        "links": audit.links,
                        "init_runs": audit.init_runs,
                        "handler_entries": audit.handler_entries,
                        "retired_handler_entries": audit.retired_handler_entries,
                        "released_roots": audit.released_roots,
                    })),
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
            "clock": {
                "registrations": usize::from(self.clock.is_some()),
                "pending": self.clock_queue_depth(),
                "samples": self.clock_metrics.samples,
                "delivered": self.clock_metrics.delivered,
                "stale_wakes": self.clock_metrics.stale_wakes,
                "rejected_samples": self.clock_metrics.rejected_samples,
                "lanes": self.clock_lanes.iter().map(|&lane| serde_json::json!({
                    "id": lane,
                    "text_applies": host_session_text_applies(lane),
                })).collect::<Vec<_>>(),
            },
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
        let (app_keys, live, selected, error, clock_ready) = with_ase_sessions_host(|h| {
            (
                h.app_keys()
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>(),
                h.live_ids(),
                h.selected_id(),
                h.errors.last().cloned(),
                h.clock.as_ref().map(|clock| clock.ready),
            )
        })
        .unwrap_or((Vec::new(), Vec::new(), None, None, None));

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
        if clock_ready == Some(false) {
            return root.child(View::text("Loading time…")).build();
        }
        if selected.is_none() {
            return root.build();
        }
        if let Some((texts, states, buttons, generation)) = current_session(|s| {
            (
                s.texts.clone(),
                s.states.clone(),
                s.buttons.clone(),
                s.generation,
            )
        }) {
            let mut frame = View::col().spacing(6).padding(12);
            if clock_ready.is_none() {
                frame = frame.child(View::text_styled(
                    &format!("frame session generation {generation}"),
                    "text-sm",
                ));
            }
            for i in 0..texts.len().max(states.len()) {
                for (leaf, text) in texts.get(i).into_iter().flatten().enumerate() {
                    frame = if clock_ready == Some(true) {
                        frame.child(View::text_styled(text, "text-4xl font-mono"))
                    } else {
                        frame.child(View::text(format!("t{leaf}: {text}")))
                    };
                }
                if clock_ready.is_none() {
                    for (name, value) in states.get(i).into_iter().flatten() {
                        frame = frame.child(View::text(format!("{name} = {value}")));
                    }
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
