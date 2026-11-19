//! PLAN-1022 ASE M0：同宿主 frame 轮换的薄消费适配（UI owner）。
//!
//! 合同（T-01 冻结）：ASE 控制面（本模块）在**同一个真实宿主窗口**内轮换
//! 独立普通 Node 应用——打开 A → 关闭（空 frame）→ 打开 B；每页是独立
//! `NodeDesktopBridge::load` 的普通 Node 源（AV 真值文本），显示与关闭全部
//! 复用 980 MIN-A-UI 公共面（`ui980` provider 真实 UI-FFI + 全局 HOST
//! session 表）。本模块不复制 FFI/解析/窗口，不修改 980 模块；窗口组件只
//! 发导航消息（打开/重开/关闭），每页重启窗口或第二事件环不算嵌入。
//!
//! 所有权与代次：导航代次（`nav_generation`）在每个被受理的导航指令（打开/
//! 重开/关闭）上推进；装载票据（`AseLoadTicket`）挂载时核代次——陈旧结果
//! 被拒且零 UI 效果（不 open → 不建 session），不覆盖新页。轮换时旧 owner
//! 先经 `close_session` 释放（不留存活 session）；桥由本 host 独占持有并
//! 步进，**不安装 980 全局 BRIDGE 槽**（避免覆盖全局桥遗留旧 owner）。

use crate::ui::ace_node_desktop::{
    current_session, host_session_display, host_session_stats, NodeDesktopBridge, StepOutcome,
};
use crate::ui::{Component, View};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};

/// 一个已注册的 ASE 页面：key + 源文件显式路径（UI 显式路径消费，不复制
/// 权威源；权威 fixtures 在 auto-lang `experimental/ase-m0/fixtures/`）。
#[derive(Clone, Debug)]
pub struct AsePage {
    pub key: String,
    pub path: String,
}

/// 装载票据：一次已完成的 `NodeDesktopBridge::load`，等待挂载。挂载时核
/// 对 `generation` 与当前导航代次——不相等即陈旧，整票丢弃（零 UI 效果）。
pub struct AseLoadTicket {
    generation: u64,
    key: String,
    bridge: NodeDesktopBridge,
}

/// 一条导航记录（证据面：代次、动作、结果）。
#[derive(Clone, Debug)]
pub struct NavRecord {
    pub generation: u64,
    pub action: &'static str,
    pub key: Option<String>,
    pub outcome: String,
    pub session: Option<i32>,
}

/// 当前活动页（host 独占持有桥与 session）。
struct ActivePage {
    key: String,
    bridge: NodeDesktopBridge,
}

/// 导航脚本动作（e2e exe 与 CPU 测试同一边界；真实窗口按钮走同一 on 消息）。
#[derive(Clone, Debug)]
pub enum AseNavAction {
    Open(String),
    Close,
    Reopen,
}

/// ASE frame 宿主：页面注册表 + 活动页 + 导航代次 + 证据。
pub struct AseFrameHost {
    pages: Vec<AsePage>,
    active: Option<ActivePage>,
    nav_generation: u64,
    seen_sessions: Vec<i32>,
    pub nav_log: Vec<NavRecord>,
    pub errors: Vec<String>,
    script: Vec<(usize, AseNavAction)>,
    tick_no: usize,
}

impl AseFrameHost {
    pub fn new(pages: Vec<AsePage>) -> Self {
        Self {
            pages,
            active: None,
            nav_generation: 0,
            seen_sessions: Vec::new(),
            nav_log: Vec::new(),
            errors: Vec::new(),
            script: Vec::new(),
            tick_no: 0,
        }
    }

    /// 已注册页面（声明序）。
    pub fn pages(&self) -> &[AsePage] {
        &self.pages
    }

    /// 当前地址（活动页 key；空 frame → None）。
    pub fn current_key(&self) -> Option<&str> {
        self.active.as_ref().map(|p| p.key.as_str())
    }

    /// 当前 session 句柄（空 frame → None）。
    pub fn current_session_id(&self) -> Option<i32> {
        self.active.as_ref().and_then(|p| p.bridge.session)
    }

    /// 当前导航代次。
    pub fn nav_generation(&self) -> u64 {
        self.nav_generation
    }

    /// 安装导航脚本（tick 从 1 起；与 980 脚本纪律同构）。
    pub fn install_script(&mut self, script: Vec<(usize, AseNavAction)>) {
        self.script = script;
    }

    /// 脚本是否已全部注入。
    pub fn script_done(&self) -> bool {
        self.script.iter().all(|(t, _)| *t <= self.tick_no)
    }

    fn record(
        &mut self,
        action: &'static str,
        key: Option<String>,
        outcome: String,
        session: Option<i32>,
    ) {
        self.nav_log.push(NavRecord {
            generation: self.nav_generation,
            action,
            key,
            outcome,
            session,
        });
    }

    /// 受理打开：推进导航代次，执行装载，返回待挂载票据。未知 key → 具名
    /// 拒（零装载、零执行、代次不动）。拆分 begin/mount 即"延迟一次真实
    /// 装载结果"的受控面（V-04 陈旧拒绝用；同步打开用 [`Self::open`]）。
    pub fn begin_open(&mut self, key: &str) -> Result<AseLoadTicket, String> {
        let Some(page) = self.pages.iter().find(|p| p.key == key).cloned() else {
            let msg = format!(
                "unknown app key {key:?} (registered: {:?})",
                self.page_keys()
            );
            self.record(
                "open",
                Some(key.to_string()),
                format!("refused: {msg}"),
                None,
            );
            return Err(msg);
        };
        self.nav_generation += 1;
        let generation = self.nav_generation;
        let src = std::fs::read_to_string(&page.path)
            .map_err(|e| format!("read page {} source {}: {e}", page.key, page.path))?;
        match NodeDesktopBridge::load(&src, &page.path, 1) {
            Ok(bridge) => Ok(AseLoadTicket {
                generation,
                key: page.key,
                bridge,
            }),
            Err(e) => {
                let msg = format!("load refused: {e}");
                self.record("open", Some(key.to_string()), msg.clone(), None);
                Err(msg)
            }
        }
    }

    /// 挂载装载票据：代次不符 → 陈旧整票拒绝（不 open、不建 session、零
    /// UI 效果，不覆盖当前页）。代次相符 → 释放旧 owner（close）后 open，
    /// frame 切到新页。
    pub fn mount(&mut self, ticket: AseLoadTicket) -> Result<i32, String> {
        let AseLoadTicket {
            generation,
            key,
            mut bridge,
        } = ticket;
        if generation != self.nav_generation {
            let msg = format!(
                "stale load result for {key:?} (ticket generation {generation} != current {}); \
                 dropped without open — zero UI effect",
                self.nav_generation
            );
            // 整票丢弃：bridge drop（未 open → 未建 session）。
            drop(bridge);
            self.record(
                "mount",
                Some(key),
                format!("stale-rejected: ticket generation {generation} refused"),
                None,
            );
            self.errors.push(msg.clone());
            return Err(msg);
        }
        // 轮换：旧 owner 先关闭释放（不留存活 session、不遗留旧 owner）。
        let rotated_from = self.close_active("open-rotation");
        match bridge.open() {
            Ok(session) => {
                self.seen_sessions.push(session);
                self.active = Some(ActivePage {
                    key: key.clone(),
                    bridge,
                });
                let outcome = match rotated_from {
                    Some(old) => format!("opened session {session} (rotated from {old})"),
                    None => format!("opened session {session}"),
                };
                self.record("open", Some(key), outcome, Some(session));
                Ok(session)
            }
            Err(e) => {
                let msg = format!("open refused: {e}");
                self.record("open", Some(key), msg.clone(), None);
                self.errors.push(msg.clone());
                Err(msg)
            }
        }
    }

    /// 同步打开（begin + 立即 mount）。
    pub fn open(&mut self, key: &str) -> Result<i32, String> {
        let ticket = self.begin_open(key)?;
        self.mount(ticket)
    }

    /// 关闭当前页：AV `ui_close`（真实 FFI）→ owner 释放 → 空 frame。空
    /// frame 上关闭为 no-op（仍是受控导航，代次推进）。
    pub fn close(&mut self) -> Result<(), String> {
        self.nav_generation += 1;
        match self.close_active("close") {
            Some(old) => {
                self.record("close", Some(old), "closed; frame empty".into(), None);
                Ok(())
            }
            None => {
                self.record("close", None, "noop on empty frame".into(), None);
                Ok(())
            }
        }
    }

    /// 重新打开当前页：旧 session 先关闭释放（`open` 只建新代不关旧代——
    /// 与轮换同律，不留存活 session），再开新 session；显示模型重新提交，
    /// 业务状态延续（引擎随桥存活，980 reopen 语义）。空 frame → 具名拒。
    pub fn reopen(&mut self) -> Result<i32, String> {
        self.nav_generation += 1;
        let Some(active) = self.active.as_mut() else {
            let msg = "reopen refused: no active page".to_string();
            self.record("reopen", None, msg.clone(), None);
            return Err(msg);
        };
        if let Err(e) = active.bridge.close_session() {
            let msg = format!("reopen refused: closing old generation: {e}");
            let key = active.key.clone();
            self.record("reopen", Some(key), msg.clone(), None);
            self.errors.push(msg.clone());
            return Err(msg);
        }
        match active.bridge.open() {
            Ok(session) => {
                self.seen_sessions.push(session);
                let key = active.key.clone();
                self.record(
                    "reopen",
                    Some(key),
                    format!("reopened session {session}"),
                    Some(session),
                );
                Ok(session)
            }
            Err(e) => {
                let msg = format!("reopen refused: {e}");
                let key = active.key.clone();
                self.record("reopen", Some(key), msg.clone(), None);
                self.errors.push(msg.clone());
                Err(msg)
            }
        }
    }

    /// 关闭并释放活动页（返回旧 key）。失败计入 errors（session 仍尽力释放）。
    fn close_active(&mut self, reason: &'static str) -> Option<String> {
        let mut active = self.active.take()?;
        let key = active.key.clone();
        if let Err(e) = active.bridge.close_session() {
            self.errors.push(format!("{reason} close for {key}: {e}"));
        }
        Some(key)
    }

    fn page_keys(&self) -> Vec<&str> {
        self.pages.iter().map(|p| p.key.as_str()).collect()
    }

    /// 受控步进：注入到期脚本 → 活动桥 poll→AV 重放→差量 apply（与 980
    /// tick 受控点同构；只步进自己的 owner）。
    pub fn step(&mut self) -> StepOutcome {
        self.tick_no += 1;
        self.run_due_script();
        let Some(active) = self.active.as_mut() else {
            return StepOutcome::Idle;
        };
        active.bridge.step()
    }

    fn run_due_script(&mut self) {
        let due: Vec<AseNavAction> = self
            .script
            .iter()
            .filter(|(t, _)| *t == self.tick_no)
            .map(|(_, a)| a.clone())
            .collect();
        for action in due {
            let result = match &action {
                AseNavAction::Open(key) => self.open(key).map(|_| ()),
                AseNavAction::Close => self.close(),
                AseNavAction::Reopen => self.reopen().map(|_| ()),
            };
            if let Err(e) = result {
                self.errors.push(format!("script {action:?}: {e}"));
            }
        }
    }

    /// 证据 JSON（CPU 测试/窗口 exe 消费）：页面表、导航轨迹、活动页、
    /// 各已建 session 的宿主统计与显示模型、错误。
    pub fn evidence(&self) -> serde_json::Value {
        let sessions: Vec<serde_json::Value> = self
            .seen_sessions
            .iter()
            .filter_map(|id| {
                let stats = host_session_stats(*id)?;
                let (texts, states) = host_session_display(*id)?;
                Some(serde_json::json!({
                    "id": stats.id,
                    "generation": stats.generation,
                    "closed": stats.closed,
                    "applies": stats.applies,
                    "polled": stats.polled,
                    "late_dropped": stats.late_dropped,
                    "stale_rejected": stats.stale_rejected,
                    "texts": texts,
                    "states": states,
                }))
            })
            .collect();
        serde_json::json!({
            "pages": self.pages.iter().map(|p| serde_json::json!({
                "key": p.key, "path": p.path,
            })).collect::<Vec<_>>(),
            "nav_generation": self.nav_generation,
            "active": self.active.as_ref().map(|a| serde_json::json!({
                "key": a.key, "session": a.bridge.session,
            })),
            "nav_log": self.nav_log.iter().map(|r| serde_json::json!({
                "generation": r.generation, "action": r.action,
                "key": r.key, "outcome": r.outcome, "session": r.session,
            })).collect::<Vec<_>>(),
            "sessions": sessions,
            "errors": self.errors,
            "frames_rendered": ASE_M0_FRAMES.load(Ordering::Relaxed),
        })
    }
}

// ---------------------------------------------------------------------------
// 窗口组件：地址行 + 导航行（只发导航）+ frame 区（current_session 同源渲染）
// ---------------------------------------------------------------------------

static ASE_HOST: OnceLock<RwLock<Option<AseFrameHost>>> = OnceLock::new();
/// ASE 窗口渲染帧计数（真实渲染观察面）。
pub static ASE_M0_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// 安装 ASE frame 宿主（窗口进程一次；本槽独占桥轮换，不碰 980 全局桥）。
pub fn install_ase_host(host: AseFrameHost) {
    let cell = ASE_HOST.get_or_init(|| RwLock::new(None));
    *cell.write().unwrap() = Some(host);
}

/// 访问 ASE frame 宿主（窗口 tick/按钮与收口线程共用）。
pub fn with_ase_host<R>(f: impl FnOnce(&mut AseFrameHost) -> R) -> Option<R> {
    let cell = ASE_HOST.get()?;
    let mut guard = cell.write().ok()?;
    guard.as_mut().map(f)
}

#[derive(Clone, Debug)]
pub enum AseM0Msg {
    /// 打开页面（索引按注册序）。
    Open(usize),
    Reopen,
    Close,
    NavTick,
}

/// ASE M0 文字浏览器窗口：单真实宿主 + 导航控制面。frame 区渲染全局 HOST
/// 的**当前** session（同一渲染源：ui980 FFI 提交的 AV 真值）；导航按钮只
/// 发导航消息，业务仍在 AV。无活动 session → 空 frame。
#[derive(Debug, Default)]
pub struct AseM0TextWindow;

impl Component for AseM0TextWindow {
    type Msg = AseM0Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AseM0Msg::Open(idx) => {
                let key = with_ase_host(|h| h.pages().get(idx).map(|p| p.key.clone())).flatten();
                if let Some(key) = key {
                    // 拒绝/错误进 host.errors（view 的错误行展示；具名可辨）。
                    let _ = with_ase_host(|h| h.open(&key));
                }
            }
            AseM0Msg::Reopen => {
                let _ = with_ase_host(|h| h.reopen());
            }
            AseM0Msg::Close => {
                let _ = with_ase_host(|h| h.close());
            }
            AseM0Msg::NavTick => {
                with_ase_host(|h| h.step());
            }
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(20)
    }

    fn tick_msg(&self) -> Option<Self::Msg> {
        Some(AseM0Msg::NavTick)
    }

    fn view(&self) -> View<Self::Msg> {
        ASE_M0_FRAMES.fetch_add(1, Ordering::Relaxed);
        let (address, error, keys) = with_ase_host(|h| {
            (
                h.pages()
                    .iter()
                    .find(|p| Some(p.key.as_str()) == h.current_key())
                    .map(|p| p.path.clone()),
                h.errors.last().cloned(),
                h.pages().iter().map(|p| p.key.clone()).collect::<Vec<_>>(),
            )
        })
        .unwrap_or((None, None, Vec::new()));

        let mut root = View::col().spacing(10).padding(16);
        // 地址行：当前页显式路径（空 frame → —）。
        root = root.child(View::text(format!(
            "address: {}",
            address.as_deref().unwrap_or("—")
        )));
        if let Some(error) = error {
            root = root.child(View::text(format!("error: {error}")));
        }
        // 导航行（只发导航）。
        let mut nav = View::row().spacing(8);
        for (idx, key) in keys.iter().enumerate() {
            nav = nav.child(View::button_styled(
                format!("打开{key}"),
                AseM0Msg::Open(idx),
                "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground",
            ));
        }
        nav = nav
            .child(View::button_styled(
                "重新打开",
                AseM0Msg::Reopen,
                "px-3 py-1.5 rounded-lg bg-secondary text-secondary-foreground",
            ))
            .child(View::button_styled(
                "关闭",
                AseM0Msg::Close,
                "px-3 py-1.5 rounded-lg bg-muted text-foreground",
            ));
        root = root.child(nav.build());

        // frame 区：当前 session（AV 真值经 ui980 FFI 提交）；无活动 → 空。
        let session = current_session(|s| {
            (
                s.texts.clone(),
                s.states.clone(),
                s.buttons.clone(),
                s.inputs.clone(),
                s.generation,
            )
        });
        if let Some((texts, states, buttons, inputs, generation)) = session {
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
                let _ = (&buttons, &inputs);
            }
            root = root.child(frame.build());
        }
        root.build()
    }
}
