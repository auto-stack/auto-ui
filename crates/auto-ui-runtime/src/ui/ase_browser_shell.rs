//! PLAN-1118 AutoScape 本地浏览器七区外壳。
//!
//! 接缝合同（T-01 冻结，见 auto-lang `docs/plans/1118-*.md` §8 T-01）：
//!
//! - **七区**：标题/菜单栏（文件/查看/导航/收藏）、地址栏（后退/前进/
//!   刷新-停止/输入 + `Ctrl+L`）、右侧工具栏（收藏当前页/收藏面板/窗口
//!   菜单）、页签行（`Ctrl+T` 新建、`Ctrl+W` 关闭、点击切换）、收藏栏
//!   （`Ctrl+D` 添加、打开、删除、重启保留）、中央内容区（guest AutoFrame
//!   或内建 Gallery 目录——非 guest 原生内容进入同一视口）、底部状态栏
//!   （已提交来源、加载/错误阶段、悬停目录链接目标）。
//! - **导航**：每页签独立 `nav_generation`（进行中请求代次）、已提交地址、
//!   历史游标与视口。受理导航 = 代次推进 + **分段装载**（`begin_open` 真实
//!   编译成票据）；挂载 = 下一 tick `mount_open` 前核代次与容量——停止/
//!   新请求/关页使旧票据失效（drop 即引擎释放，零 session/零 UI 效果）。
//!   成功的新目标提交一次（标题/地址/历史截断前进/视口）；后退/前进只移
//!   游标，未驻留的历史页按原地址显式重开（不加历史）；刷新不加历史。
//!   失败保留已提交页面并单独给出失败请求与重试入口。
//! - **身份与解析**：冻结注册表 `app://auto.demo.counter/`（Counter）、
//!   `app://auto.demo.clock/`（Clock）、`app://auto.gallery/`（内建目录，
//!   零 guest/零编译/零监听）。地址输入归一化：trim → 显示名（大小写不
//!   敏感）/裸 identity → `app://<identity>/`（补根路由）→ ase-m0
//!   `parse_app_uri` 单一语法源严格解析 → 冻结表 lookup；未登记具名拒，
//!   已提交地址不变。
//! - **会话层**：原样复用 [`AseSessionHost`]（两活上限/选中车道/close
//!   代次/Clock 唤醒）；每页签至多一个 guest 会话，导航替换在新目标可
//!   呈现后释放旧会话；容量不足具名拒并保留已提交页面。
//! - **收藏/配置**：`~/.config/autoos/apps/autoscape/bookmarks.json`
//!   （env `AUTOSCAPE_HOME` 覆盖根，隔离测试），原子写经既有
//!   `auto_lang_core::state_file::atomic_write`；读写失败向外壳报告并
//!   保留最后有效配置。
//!
//! 本模块不复制 FFI/解析/窗口，不修改 980/981 模块；观察口与负控共用
//! 同一真实调用边界。

use crate::ui::ace_node_desktop::{current_session, StepOutcome};
use crate::ui::ase_m0_sessions::{AseApp, AseOpenTicket, AseSessionHost};
use crate::ui::autoweb_nav::{AutowebNavOptions, WebNavWorker};
use crate::ui::component::FocusRequest;
use crate::ui::{Component, View};
use ase_m0::autoweb::WebAddress;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

/// 冻结的 Counter 演示身份（源：auto-lang `experimental/ase-m0/fixtures/
/// counter-a/main.at`）。
pub const SHELL_IDENTITY_COUNTER: &str = "auto.demo.counter";
/// 冻结的 Clock 演示身份（源：`fixtures/clock/main.at`）。
pub const SHELL_IDENTITY_CLOCK: &str = "auto.demo.clock";
/// 冻结的内建 Gallery 目录身份（元数据页，零 guest）。
pub const SHELL_IDENTITY_GALLERY: &str = "auto.gallery";

/// 窄窗判据（逻辑 px）：低于此宽度收藏栏自动收纳、页签标题截断（次要
/// 工具经窗口菜单 ⋯ 保持可达）。
pub const NARROW_WINDOW_PX: f32 = 720.0;

// ---------------------------------------------------------------------------
// 地址与应用身份
// ---------------------------------------------------------------------------

/// 外壳冻结登记表的一项。
#[derive(Clone, Debug)]
pub struct ShellApp {
    pub identity: String,
    pub display: String,
    pub kind: ShellAppKind,
}

#[derive(Clone, Debug)]
pub enum ShellAppKind {
    /// 普通 Node 应用（真实 ASE 会话；源为显式路径）。
    Node { path: String },
    /// 内建目录页（元数据渲染，零装载）。
    Gallery,
    /// AutoWeb 只读网络文档（PLAN-1119；HTTPS/`autoweb://` 同一导航入口）。
    WebDoc,
}

impl ShellApp {
    pub fn canonical(&self) -> String {
        format!("app://{}/", self.identity)
    }
}

/// 一个已解析的导航目标（提交/历史/收藏记录的规范形态）。
#[derive(Clone, Debug)]
pub struct ResolvedTarget {
    pub identity: String,
    pub display: String,
    pub canonical: String,
    pub kind: ShellAppKind,
    /// 网络目标的已解析地址（`WebDoc` 恒有；本地目标为 None）。
    pub web: Option<WebAddress>,
}

/// 解析失败（具名；不改变已提交状态）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveRefusal(pub String);

/// 输入归一化 + 严格解析 + 冻结表 lookup（T-01 冻结规则）。
///
/// - trim 后为空 → 具名拒。
/// - 网络写法（`https://`/`autoweb://`/`http://`）→ ase-m0
///   `parse_web_address` 单一语法源（别名映射/降级拒/userinfo 拒/片段
///   剥离），不走 app 表（PLAN-1119）。
/// - 含 `://` → 原样交严格解析（scheme 非 `app` 具名拒）。
/// - 不含 scheme → 先试显示名（大小写不敏感），再按裸 identity 补
///   `app://<input>/`（输入尾部 `/` 保留），随后严格解析。
/// - 严格解析复用 ase-m0 `parse_app_uri`（单一语法源：小写 authority、
///   仅根路由、无 userinfo/port/query/fragment）。
/// - 解析通过但身份未登记 → 具名拒。
pub fn resolve_address(input: &str, apps: &[ShellApp]) -> Result<ResolvedTarget, ResolveRefusal> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ResolveRefusal("empty address".to_string()));
    }
    // PLAN-1119：网络写法先行——协议层具名拒绝（含 http 降级拒）。
    if let Some((scheme, _)) = input.split_once("://") {
        if matches!(
            scheme.to_ascii_lowercase().as_str(),
            "https" | "autoweb" | "http"
        ) {
            let address = ase_m0::autoweb::parse_web_address(input)
                .map_err(|e| ResolveRefusal(format!("resolve: {e}")))?;
            return Ok(ResolvedTarget {
                identity: format!("web:{}", address.host),
                display: address.host.clone(),
                canonical: address.display_address(),
                kind: ShellAppKind::WebDoc,
                web: Some(address),
            });
        }
    }
    let candidate = if input.contains("://") {
        // 根路由缺省补全：`app://identity` → `app://identity/`（地址栏友好
        // 归一化；其余语法面仍交严格解析拒绝）。
        match input.split_once("://") {
            Some((scheme, authority)) if scheme == "app" && !authority.contains('/') => {
                format!("{input}/")
            }
            _ => input.to_string(),
        }
    } else if let Some(app) = apps.iter().find(|a| a.display.eq_ignore_ascii_case(input)) {
        app.canonical()
    } else {
        format!("app://{input}/")
    };
    let parsed =
        ase_m0::parse_app_uri(&candidate).map_err(|e| ResolveRefusal(format!("resolve: {e}")))?;
    let app = apps
        .iter()
        .find(|a| a.identity == parsed.identity)
        .ok_or_else(|| {
            ResolveRefusal(format!(
                "resolve: identity {:?} is not registered (supported: {:?})",
                parsed.identity,
                apps.iter().map(|a| a.identity.as_str()).collect::<Vec<_>>()
            ))
        })?;
    Ok(ResolvedTarget {
        identity: app.identity.clone(),
        display: app.display.clone(),
        canonical: app.canonical(),
        kind: app.kind.clone(),
        web: None,
    })
}

/// 权威 fixture 根解析：env `ASE_M0_TEXT_ROOT` → 组内 auto-lang 兄弟树
/// （与 ase_m0_sessions 测试同型；两级皆缺 → None，具名准备失败）。
pub fn resolve_fixture_root() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("ASE_M0_TEXT_ROOT") {
        if !root.trim().is_empty() {
            return Some(PathBuf::from(root));
        }
    }
    let sibling = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../auto-lang/experimental/ase-m0/fixtures");
    sibling.exists().then(|| sibling)
}

/// 组装冻结登记表（Counter/Clock 指向 fixture_root 下的权威源）。
pub fn shell_app_table(fixture_root: &Path) -> Vec<ShellApp> {
    let node = |identity: &str, display: &str, file: &str| ShellApp {
        identity: identity.to_string(),
        display: display.to_string(),
        kind: ShellAppKind::Node {
            path: fixture_root.join(file).to_string_lossy().to_string(),
        },
    };
    vec![
        node(SHELL_IDENTITY_COUNTER, "Counter", "counter-a/main.at"),
        node(SHELL_IDENTITY_CLOCK, "Clock", "clock/main.at"),
        ShellApp {
            identity: SHELL_IDENTITY_GALLERY.to_string(),
            display: "Gallery".to_string(),
            kind: ShellAppKind::Gallery,
        },
    ]
}

// ---------------------------------------------------------------------------
// 收藏与配置（宿主用户配置目录；独立于 app 数据根）
// ---------------------------------------------------------------------------

/// 一条收藏（标题 + 规范地址 + 打开方式；M0 冻结 open_as="tab"）。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub address: String,
    pub open_as: String,
}

/// 收藏存储 owner：`<root>/bookmarks.json`，原子写。
#[derive(Clone, Debug)]
pub struct BookmarkStore {
    root: PathBuf,
}

/// 收藏配置根：env `AUTOSCAPE_HOME` → 该目录本身；缺省
/// `~/.config/autoos/apps/autoscape`（与 desktop_config 同布局约定）。
pub fn autoscape_config_root() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("AUTOSCAPE_HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    Some(
        dirs::home_dir()?
            .join(".config")
            .join("autoos")
            .join("apps")
            .join("autoscape"),
    )
}

/// 读取结果：文件缺席 = 空表（Ok）；坏 JSON = 具名错误 + 空表可用
/// （盘上旧有效写入不被覆盖，等待下次成功保存修复）。
#[derive(Debug)]
pub enum BookmarkLoad {
    Loaded(Vec<Bookmark>),
    Missing(Vec<Bookmark>),
    Corrupt {
        message: String,
        fallback: Vec<Bookmark>,
    },
}

impl BookmarkStore {
    pub fn new(root: PathBuf) -> Self {
        BookmarkStore { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self) -> PathBuf {
        self.root.join("bookmarks.json")
    }

    pub fn load(&self) -> BookmarkLoad {
        match std::fs::read_to_string(self.path()) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BookmarkLoad::Missing(Vec::new()),
            Err(e) => BookmarkLoad::Corrupt {
                message: format!("read {}: {e}", self.path().display()),
                fallback: Vec::new(),
            },
            Ok(text) => match serde_json::from_str::<Vec<Bookmark>>(&text) {
                Ok(list) => BookmarkLoad::Loaded(list),
                Err(e) => BookmarkLoad::Corrupt {
                    message: format!("parse {}: {e}", self.path().display()),
                    fallback: Vec::new(),
                },
            },
        }
    }

    /// 原子写（tempfile 同目录 + rename；失败保留盘上旧内容）。
    pub fn save(&self, bookmarks: &[Bookmark]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let bytes = serde_json::to_string_pretty(bookmarks)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        auto_lang_core::state_file::atomic_write(&self.path(), bytes.as_bytes())
    }
}

// ---------------------------------------------------------------------------
// 页签与导航
// ---------------------------------------------------------------------------

/// 一条历史记录（地址 + 提交时标题）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub address: String,
    pub title: String,
}

/// 已提交的网络只读文档页（PLAN-1119；一次可呈现时与地址/历史/视口
/// 一起提交——已下载描述不构成已打开文档）。
#[derive(Clone, Debug)]
pub struct WebDocPage {
    /// 保留写法的地址（历史/收藏形态）。
    pub address: String,
    /// 规范 https URL。
    pub canonical: String,
    pub title: String,
    pub release: String,
    pub app_id: String,
    /// 已验证的 AutoDown 源（sha256 与描述声明一致 + 只读准入通过）。
    pub body: String,
    pub sha256_hex: String,
}

/// 已提交页面（导航在新目标可呈现时一次提交）。
#[derive(Clone, Debug)]
pub struct CommittedPage {
    pub target: ResolvedTarget,
    /// 该页签持有的 guest 会话（Gallery/新页签/网络文档为 None）。
    pub session: Option<usize>,
    /// 网络只读文档内容（`WebDoc` 提交时恒有；本地目标为 None）。
    pub web: Option<WebDocPage>,
}

/// 导航受理模式（决定成功提交时的历史语义）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavMode {
    /// 新目标（地址栏/目录卡片/收藏）：成功且地址不同 → 截断前进并追加。
    New,
    /// 刷新：不加历史。
    Refresh,
    /// 后退/前进命中的未驻留历史页：按原地址显式重开，不加历史、不动游标。
    Reopen,
}

/// 进行中的导航（每页签至多一个；generation 不匹配即整票失效）。
pub struct PendingNav {
    pub generation: u64,
    pub target: ResolvedTarget,
    pub mode: NavMode,
    /// guest 目标的已装载票据（Gallery/WebDoc 为 None）。
    pub ticket: Option<AseOpenTicket>,
    /// 网络目标的取消旗标（PLAN-1119；置位后 worker 在下一 chunk/跳中止）。
    pub web: Option<Arc<std::sync::atomic::AtomicBool>>,
}

/// 最近一次失败请求（独立展示 + 重试入口；不动已提交页面）。
#[derive(Clone, Debug)]
pub struct NavFailure {
    pub address: String,
    pub message: String,
    pub generation: u64,
}

/// 一个页签（轻量元数据；与实际驻留会话分离）。
pub struct ShellTab {
    pub id: usize,
    pub history: Vec<HistoryEntry>,
    pub cursor: usize,
    pub committed: Option<CommittedPage>,
    pub pending: Option<PendingNav>,
    pub failure: Option<NavFailure>,
    pub nav_generation: u64,
}

impl ShellTab {
    fn new_gallery(id: usize) -> Self {
        ShellTab {
            id,
            history: Vec::new(),
            cursor: 0,
            committed: None,
            pending: None,
            failure: None,
            nav_generation: 0,
        }
    }

    /// 页签标题（提交地址的显示名；无提交 = Gallery）。
    pub fn title(&self) -> &str {
        self.history
            .get(self.cursor)
            .map(|entry| entry.title.as_str())
            .unwrap_or("Gallery")
    }

    pub fn committed_address(&self) -> Option<&str> {
        self.committed
            .as_ref()
            .map(|page| page.target.canonical.as_str())
    }
}

/// 一条导航事件（证据面）。
#[derive(Clone, Debug)]
pub struct NavEvent {
    pub tick: usize,
    pub tab: usize,
    pub generation: u64,
    pub action: &'static str,
    pub address: String,
    pub outcome: String,
}

/// 菜单（七区：标题/菜单栏）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMenu {
    File,
    View,
    Nav,
    Favorites,
    Window,
}

/// 外壳脚本动作（e2e exe 与 CPU 测试同一边界；窗口按钮同一 on 消息面）。
#[derive(Clone, Debug)]
pub enum ShellAction {
    Navigate(String),
    Back,
    Forward,
    Refresh,
    Stop,
    NewTab,
    SelectTab(usize),
    CloseTab(usize),
    Click(usize),
    FavoriteAdd,
    FavoriteDelete(usize),
    ToggleBar,
    TogglePanel,
    HoverIn(String),
    HoverOut,
}

/// AutoScape 浏览器宿主：冻结登记表 + 页签表 + 导航状态机 + 收藏 +
/// guest 会话层（[`AseSessionHost`] 原样复用）。
pub struct AutoscapeHost {
    apps: Vec<ShellApp>,
    fixture_root: PathBuf,
    store: BookmarkStore,
    sessions: AseSessionHost,
    tabs: Vec<ShellTab>,
    active: usize,
    next_tab_id: usize,
    bookmarks: Vec<Bookmark>,
    show_favorites_bar: bool,
    favorites_panel_open: bool,
    open_menu: Option<ShellMenu>,
    /// 地址栏编辑态（None = 显示已提交来源）。
    address_editing: Option<String>,
    /// 状态栏悬停目标（MouseArea enter/exit 驱动）。
    hover_hint: Option<String>,
    /// PLAN-1119：网络导航选项（信任根/限额；测试注入隔离 CA）。
    web_options: AutowebNavOptions,
    /// PLAN-1119：网络导航 worker（首次网络导航时惰性启动）。
    web_worker: Option<WebNavWorker>,
    /// 待执行的地址栏聚焦（take_focus_request 消费一次）。
    focus_epoch: Arc<AtomicU64>,
    focus_address: bool,
    pub nav_log: Vec<NavEvent>,
    pub errors: Vec<String>,
    script: Vec<(usize, ShellAction)>,
    tick_no: usize,
}

impl AutoscapeHost {
    /// 真实根构造（fixture 根 + 配置根显式注入；缺席具名入 errors）。
    pub fn with_roots(fixture_root: PathBuf, config_root: PathBuf) -> Self {
        let apps = shell_app_table(&fixture_root);
        let session_apps: Vec<AseApp> = apps
            .iter()
            .filter_map(|app| match &app.kind {
                ShellAppKind::Node { path } => Some(AseApp {
                    key: app.identity.clone(),
                    path: path.clone(),
                }),
                ShellAppKind::Gallery | ShellAppKind::WebDoc => None,
            })
            .collect();
        let mut errors = Vec::new();
        for app in &apps {
            if let ShellAppKind::Node { path } = &app.kind {
                if !Path::new(path).exists() {
                    errors.push(format!(
                        "preparation: fixture for {} missing at {path} (ASE_M0_TEXT_ROOT or \
                         the group sibling auto-lang tree)",
                        app.identity
                    ));
                }
            }
        }
        let store = BookmarkStore::new(config_root);
        let bookmarks = match store.load() {
            BookmarkLoad::Loaded(list) | BookmarkLoad::Missing(list) => list,
            BookmarkLoad::Corrupt { message, fallback } => {
                errors.push(format!("bookmarks: {message}; starting from empty list"));
                fallback
            }
        };
        let mut sessions = AseSessionHost::new(session_apps.clone());
        if sessions
            .app_keys()
            .iter()
            .any(|k| *k == SHELL_IDENTITY_CLOCK)
        {
            match sessions.with_clock_app(SHELL_IDENTITY_CLOCK.to_string()) {
                Ok(host) => sessions = host,
                Err(error) => {
                    errors.push(format!("clock registration: {error}"));
                    sessions = AseSessionHost::new(session_apps);
                }
            }
        }
        let mut shell = AutoscapeHost {
            apps,
            fixture_root,
            store,
            sessions,
            tabs: Vec::new(),
            active: 0,
            next_tab_id: 1,
            bookmarks,
            show_favorites_bar: true,
            favorites_panel_open: false,
            open_menu: None,
            address_editing: None,
            hover_hint: None,
            web_options: AutowebNavOptions::default(),
            web_worker: None,
            focus_epoch: Arc::new(AtomicU64::new(0)),
            focus_address: false,
            nav_log: Vec::new(),
            errors,
            script: Vec::new(),
            tick_no: 0,
        };
        shell.new_tab();
        shell
    }

    /// 默认根构造（fixture 根解析 + 配置根缺省/env）。
    pub fn new() -> Result<Self, String> {
        let fixture_root = resolve_fixture_root().ok_or(
            "preparation failure: ASE fixtures root not found (ASE_M0_TEXT_ROOT or the group \
             sibling auto-lang tree)",
        )?;
        let config_root = autoscape_config_root()
            .ok_or_else(|| "preparation failure: home directory unavailable".to_string())?;
        Ok(Self::with_roots(fixture_root, config_root))
    }

    /// PLAN-1119：网络导航选项注入（测试注入隔离测试 CA 与更小限额；
    /// 生产缺省=系统可信根 + 冻结限额）。
    pub fn with_autoweb_options(mut self, options: AutowebNavOptions) -> Self {
        self.web_options = options;
        self
    }

    /// PLAN-1119：网络导航 worker 观察口（在途计数/事件 drain 由 step 承担）。
    pub fn web_in_flight(&self) -> usize {
        self.web_worker.as_ref().map_or(0, |w| w.in_flight())
    }

    // ----- 观察面 -----

    pub fn apps(&self) -> &[ShellApp] {
        &self.apps
    }

    pub fn fixture_root(&self) -> &Path {
        &self.fixture_root
    }

    pub fn store(&self) -> &BookmarkStore {
        &self.store
    }

    pub fn tabs(&self) -> &[ShellTab] {
        &self.tabs
    }

    pub fn active_tab(&self) -> &ShellTab {
        &self.tabs[self.active]
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn bookmarks(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    pub fn favorites_bar_visible(&self) -> bool {
        self.show_favorites_bar
    }

    pub fn favorites_panel_open(&self) -> bool {
        self.favorites_panel_open
    }

    pub fn open_menu(&self) -> Option<ShellMenu> {
        self.open_menu
    }

    pub fn hover_hint(&self) -> Option<&str> {
        self.hover_hint.as_deref()
    }

    pub fn address_editing(&self) -> Option<&str> {
        self.address_editing.as_deref()
    }

    /// 状态栏文本：悬停目标 > 失败阶段 > 加载阶段 > 已提交来源。
    pub fn status_line(&self) -> String {
        if let Some(hint) = &self.hover_hint {
            return format!("目标: {hint}");
        }
        let tab = self.active_tab();
        if let Some(failure) = &tab.failure {
            return format!("错误: {}", failure.message);
        }
        if tab.pending.is_some() {
            return format!(
                "加载中: {}",
                tab.pending
                    .as_ref()
                    .map(|p| p.target.canonical.clone())
                    .unwrap_or_default()
            );
        }
        tab.committed
            .as_ref()
            .map(|page| match page.target.kind {
                ShellAppKind::Node { .. } => {
                    format!("本地应用 {}", page.target.canonical)
                }
                ShellAppKind::Gallery => format!("本地目录 {}", page.target.canonical),
                ShellAppKind::WebDoc => format!("网络文档 {}", page.target.canonical),
            })
            .unwrap_or_else(|| "就绪".to_string())
    }

    fn log(&mut self, action: &'static str, address: impl Into<String>, outcome: String) {
        let tab = self.tabs[self.active].id;
        let generation = self.tabs[self.active].nav_generation;
        self.nav_log.push(NavEvent {
            tick: self.tick_no,
            tab,
            generation,
            action,
            address: address.into(),
            outcome,
        });
    }

    // ----- 页签 -----

    /// 新建页签（初始提交 Gallery 目录；元数据零装载）。
    pub fn new_tab(&mut self) -> usize {
        let mut tab = ShellTab::new_gallery(self.next_tab_id);
        self.next_tab_id += 1;
        let gallery = self
            .apps
            .iter()
            .find(|a| a.identity == SHELL_IDENTITY_GALLERY)
            .cloned();
        if let Some(app) = gallery {
            tab.committed = Some(CommittedPage {
                target: ResolvedTarget {
                    identity: app.identity.clone(),
                    display: app.display.clone(),
                    canonical: app.canonical(),
                    kind: app.kind.clone(),
                    web: None,
                },
                session: None,
                web: None,
            });
            tab.history.push(HistoryEntry {
                address: app.canonical(),
                title: app.display.clone(),
            });
            tab.cursor = 0;
        }
        let id = tab.id;
        self.tabs.push(tab);
        // 新页签成为活动页签（无 guest 会话 → 会话层选中态清空由
        // commit/select 语义维护；此处显式落位）。
        self.activate_tab_index(self.tabs.len() - 1);
        id
    }

    /// 切换活动页签；其 guest 会话经会话层 `select` 恢复显示车道（引擎
    /// 常驻、切回零装载零派发；Clock 恢复只发一条新校时——980/1109 语义）。
    pub fn select_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err(format!(
                "select tab refused: index {index} out of range ({} tabs)",
                self.tabs.len()
            ));
        }
        if index != self.active {
            self.activate_tab_index(index);
        }
        Ok(())
    }

    fn activate_tab_index(&mut self, index: usize) {
        self.active = index;
        self.open_menu = None;
        self.hover_hint = None;
        self.address_editing = None;
        let session = self.tabs[index].committed.as_ref().and_then(|p| p.session);
        match session {
            Some(session) => {
                if let Err(error) = self.sessions.select(session) {
                    self.errors.push(format!("select tab: {error}"));
                }
            }
            // 无 guest 的页签（Gallery/新页签）：撤出向车道、停时钟资格——
            // 隐藏页签零业务/零呈现；引擎按会话常驻不释放。
            None => {
                if let Err(error) = self.sessions.deselect() {
                    self.errors.push(format!("select tab: {error}"));
                }
            }
        }
    }

    /// 关闭页签：释放其 guest 会话（引擎 owner 释放）；活动权交给相邻
    /// 页签，最后一个页签关闭后新建 Gallery 页签。焦点回落到地址栏。
    pub fn close_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err(format!(
                "close tab refused: index {index} out of range ({} tabs)",
                self.tabs.len()
            ));
        }
        // 进行中导航整票失效（票据 drop → 引擎释放；网络旗标置位 →
        // worker 中止并释放连接）。
        let tab = self.tabs.remove(index);
        if let Some(pending) = tab.pending {
            if let Some(cancel) = &pending.web {
                cancel.store(true, Ordering::Relaxed);
            }
            drop(pending.ticket);
            self.nav_log.push(NavEvent {
                tick: self.tick_no,
                tab: tab.id,
                generation: pending.generation,
                action: "close-tab",
                address: pending.target.canonical.clone(),
                outcome: "pending navigation invalidated by tab close".to_string(),
            });
        }
        if let Some(session) = tab.committed.and_then(|page| page.session) {
            if let Err(error) = self.sessions.close_session(session) {
                self.errors.push(format!("close tab: {error}"));
            }
        }
        if self.tabs.is_empty() {
            self.new_tab();
        } else if self.active >= self.tabs.len() {
            self.activate_tab_index(self.tabs.len() - 1);
        } else if index <= self.active {
            // 活动页签自身或其前者被关 → 相邻页签接管活动权。
            let target = self.active.min(self.tabs.len() - 1);
            self.activate_tab_index(target);
        }
        self.request_address_focus();
        Ok(())
    }

    // ----- 导航核心 -----

    /// 受理导航（地址栏/目录卡片/收藏/菜单共用入口）：归一化解析 →
    /// 代次推进 → 分段装载（guest）成票据；提交在下一 step（新目标可
    /// 呈现时一次提交）。解析/装载失败 → 记入该页签 failure（重试入口），
    /// 已提交页面不动。
    pub fn navigate(&mut self, input: &str) -> Result<(), String> {
        self.navigate_with_mode(input, NavMode::New)
    }

    fn navigate_with_mode(&mut self, input: &str, mode: NavMode) -> Result<(), String> {
        let target = match resolve_address(input, &self.apps) {
            Ok(target) => target,
            Err(refusal) => {
                let message = refusal.0.clone();
                self.tabs[self.active].failure = Some(NavFailure {
                    address: input.trim().to_string(),
                    message: message.clone(),
                    generation: self.tabs[self.active].nav_generation,
                });
                self.log("navigate", input, format!("refused: {message}"));
                return Err(message);
            }
        };
        // 同页刷新语义：地址栏重复提交当前已提交地址 = 刷新（不加历史）。
        let same_as_committed = self.tabs[self.active]
            .committed
            .as_ref()
            .is_some_and(|page| page.target.canonical == target.canonical);
        let mode = if same_as_committed && mode == NavMode::New {
            NavMode::Refresh
        } else {
            mode
        };
        // 旧进行中请求整票失效（同页新请求使旧完成失效）；网络目标同时
        // 置位取消旗标（worker 在下一 chunk/跳中止并释放连接）。
        self.tabs[self.active].nav_generation += 1;
        let generation = self.tabs[self.active].nav_generation;
        if let Some(stale) = self.tabs[self.active].pending.take() {
            if let Some(cancel) = &stale.web {
                cancel.store(true, Ordering::Relaxed);
            }
            drop(stale.ticket);
            self.nav_log.push(NavEvent {
                tick: self.tick_no,
                tab: self.tabs[self.active].id,
                generation,
                action: "navigate",
                address: stale.target.canonical.clone(),
                outcome: format!(
                    "superseded by new request (generation {} → {generation})",
                    stale.generation
                ),
            });
        }
        self.tabs[self.active].failure = None;
        // 分段装载：guest 真实编译成票据；Gallery 仅元数据；网络目标交
        // worker（事件驱动完成——可更新所属后台页签）。
        let (ticket, web_cancel) = match &target.kind {
            ShellAppKind::Node { .. } => match self.sessions.begin_open(&target.identity) {
                Ok(ticket) => (Some(ticket), None),
                Err(error) => {
                    self.tabs[self.active].failure = Some(NavFailure {
                        address: target.canonical.clone(),
                        message: error.clone(),
                        generation,
                    });
                    self.log("navigate", &target.canonical, format!("refused: {error}"));
                    self.errors
                        .push(format!("navigate {}: {error}", target.canonical));
                    return Err(error);
                }
            },
            ShellAppKind::Gallery => (None, None),
            ShellAppKind::WebDoc => {
                let address = target
                    .web
                    .clone()
                    .expect("web target carries a parsed address");
                let worker = self
                    .web_worker
                    .get_or_insert_with(|| WebNavWorker::spawn(self.web_options.clone()));
                let cancel = worker.start(self.tabs[self.active].id, generation, address);
                (None, Some(cancel))
            }
        };
        self.log(
            "navigate",
            &target.canonical,
            format!("staged (generation {generation}, mode {mode:?})"),
        );
        self.tabs[self.active].pending = Some(PendingNav {
            generation,
            target,
            mode,
            ticket,
            web: web_cancel,
        });
        self.address_editing = None;
        Ok(())
    }

    /// 挂载点提交（step 内调用；仅活动页签的进行中**本地**请求提交）。
    /// 网络请求不经此路径：由事件驱动完成（[`Self::drain_web_events`]，
    /// 可更新所属后台页签——PLAN-1119 §5）。
    fn commit_active_pending(&mut self) {
        if self.tabs[self.active]
            .pending
            .as_ref()
            .is_some_and(|p| p.web.is_some())
        {
            return;
        }
        let tab = &mut self.tabs[self.active];
        let Some(pending) = tab.pending.take() else {
            return;
        };
        let tab_id = tab.id;
        let generation = pending.generation;
        let mode = pending.mode;
        let target = pending.target.clone();
        if generation != tab.nav_generation {
            drop(pending.ticket);
            self.nav_log.push(NavEvent {
                tick: self.tick_no,
                tab: tab_id,
                generation,
                action: "commit",
                address: target.canonical.clone(),
                outcome: format!(
                    "stale completion dropped (ticket generation {generation} != current {}); \
                     committed page unchanged",
                    tab.nav_generation
                ),
            });
            return;
        }
        // 挂载：guest 先落位（权威容量复检），成功后才释放旧会话——失败
        // 保留已提交页面与原记录。
        let mounted_session = match pending.ticket {
            Some(ticket) => match self.sessions.mount_open(ticket) {
                Ok(session) => Some(session),
                Err(error) => {
                    self.tabs[self.active].failure = Some(NavFailure {
                        address: target.canonical.clone(),
                        message: error.clone(),
                        generation,
                    });
                    self.log("commit", &target.canonical, format!("refused: {error}"));
                    self.errors
                        .push(format!("commit {}: {error}", target.canonical));
                    return;
                }
            },
            None => None,
        };
        let old_session = self.tabs[self.active]
            .committed
            .take()
            .and_then(|page| page.session);
        if let Some(old) = old_session {
            if Some(old) != mounted_session {
                if let Err(error) = self.sessions.close_session(old) {
                    self.errors.push(format!("commit rotate: {error}"));
                }
            }
        }
        self.tabs[self.active].committed = Some(CommittedPage {
            target: target.clone(),
            session: mounted_session,
            web: None,
        });
        // 历史语义：New 且地址与游标记录不同 → 截断前进并追加；
        // Refresh/Reopen 不动历史。
        let tab = &mut self.tabs[self.active];
        if mode == NavMode::New {
            let current = tab.history.get(tab.cursor);
            if current.map(|e| e.address.as_str()) != Some(target.canonical.as_str()) {
                tab.history.truncate(tab.cursor + 1);
                tab.history.push(HistoryEntry {
                    address: target.canonical.clone(),
                    title: target.display.clone(),
                });
                tab.cursor = tab.history.len() - 1;
            }
        }
        tab.failure = None;
        self.nav_log.push(NavEvent {
            tick: self.tick_no,
            tab: tab_id,
            generation,
            action: "commit",
            address: target.canonical.clone(),
            outcome: format!("committed (session {:?})", mounted_session),
        });
    }

    /// 后退：游标前移；目标历史页未驻留（会话已释放/未持有）→ 按原地址
    /// 显式重开（不加历史）。失败保留游标与已提交页面。
    pub fn back(&mut self) -> Result<(), String> {
        self.step_cursor(-1)
    }

    /// 前进：游标后移（同 [`Self::back`] 语义）。
    pub fn forward(&mut self) -> Result<(), String> {
        self.step_cursor(1)
    }

    fn step_cursor(&mut self, delta: i32) -> Result<(), String> {
        let (history_len, cursor, committed_address, tab_id) = {
            let tab = &self.tabs[self.active];
            (
                tab.history.len(),
                tab.cursor,
                tab.committed_address().map(|s| s.to_string()),
                tab.id,
            )
        };
        let next = cursor as i64 + delta as i64;
        let action = if delta < 0 { "back" } else { "forward" };
        if next < 0 || next >= history_len as i64 {
            self.log(
                action,
                committed_address.as_deref().unwrap_or(""),
                "noop at history edge".into(),
            );
            return Ok(());
        }
        let next = next as usize;
        let entry = self.tabs[self.active].history[next].clone();
        if committed_address.as_deref() == Some(entry.address.as_str()) {
            self.tabs[self.active].cursor = next;
            self.log(
                action,
                &entry.address,
                "cursor moved (page retained)".into(),
            );
            Ok(())
        } else {
            // 未驻留历史页：游标先落位（重开失败回退），按原地址显式重开。
            self.tabs[self.active].cursor = next;
            let result = self.navigate_with_mode(&entry.address, NavMode::Reopen);
            if result.is_err() {
                self.tabs[self.active].cursor = cursor;
                let _ = tab_id;
                self.log(
                    action,
                    &entry.address,
                    "reopen refused; cursor retained".into(),
                );
            }
            result
        }
    }

    /// 刷新当前目标：同地址重新装载（guest 新会话新代次、种子 0），不加
    /// 历史。空提交（无历史）具名拒。
    pub fn refresh(&mut self) -> Result<(), String> {
        let tab = &self.tabs[self.active];
        let Some(address) = tab.committed_address().map(|s| s.to_string()) else {
            let message = "refresh refused: no committed page".to_string();
            self.log("refresh", "", message.clone());
            return Err(message);
        };
        self.navigate_with_mode(&address, NavMode::Refresh)
    }

    /// 停止：使活动页签的进行中请求失效（票据 drop → 引擎释放；网络旗
    /// 标置位 → worker 中止），保留已提交页面与原历史记录。
    pub fn stop(&mut self) -> Result<(), String> {
        let committed_address = self.tabs[self.active]
            .committed_address()
            .map(|s| s.to_string());
        let pending = self.tabs[self.active].pending.take();
        let Some(pending) = pending else {
            self.log(
                "stop",
                committed_address.as_deref().unwrap_or(""),
                "noop without pending".into(),
            );
            return Ok(());
        };
        let address = pending.target.canonical.clone();
        let generation = pending.generation;
        if let Some(cancel) = &pending.web {
            cancel.store(true, Ordering::Relaxed);
        }
        drop(pending.ticket);
        let tab = &mut self.tabs[self.active];
        tab.nav_generation += 1;
        let new_generation = tab.nav_generation;
        self.log(
            "stop",
            &address,
            format!("stopped (generation {generation} invalidated → {new_generation})"),
        );
        Ok(())
    }

    /// 重试最近一次失败请求（同一地址新代次）。
    pub fn retry_failed(&mut self) -> Result<(), String> {
        let address = self.tabs[self.active]
            .failure
            .as_ref()
            .map(|f| f.address.clone());
        match address {
            Some(address) => self.navigate(&address),
            None => {
                let message = "retry refused: no failed request".to_string();
                self.log("retry", "", message.clone());
                Err(message)
            }
        }
    }

    // ----- 地址栏 / 状态栏 -----

    /// 地址栏编辑（on_text）：编辑态只影响输入框显示，不触碰已提交来源。
    pub fn address_edited(&mut self, text: String) {
        self.address_editing = Some(text);
    }

    /// 地址栏提交（回车）：编辑文本走统一导航入口。
    pub fn submit_address(&mut self) -> Result<(), String> {
        match self.address_editing.clone() {
            Some(text) if !text.trim().is_empty() => self.navigate(&text),
            _ => {
                self.address_editing = None;
                Ok(())
            }
        }
    }

    /// 悬停合法目录链接 → 状态栏显示目标；退出恢复当前页状态。
    pub fn hover_in(&mut self, address: String) {
        self.hover_hint = Some(address);
    }

    pub fn hover_out(&mut self) {
        self.hover_hint = None;
    }

    fn request_address_focus(&mut self) {
        self.focus_epoch.fetch_add(1, Ordering::AcqRel);
        self.focus_address = true;
    }

    // ----- 菜单/工具/收藏栏 -----

    pub fn toggle_menu(&mut self, menu: ShellMenu) {
        self.open_menu = if self.open_menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
    }

    pub fn close_menu(&mut self) {
        self.open_menu = None;
    }

    pub fn toggle_favorites_bar(&mut self) {
        self.show_favorites_bar = !self.show_favorites_bar;
    }

    pub fn toggle_favorites_panel(&mut self) {
        self.favorites_panel_open = !self.favorites_panel_open;
    }

    /// 收藏当前页（`Ctrl+D`/菜单）：标题=显示名、规范地址、open_as="tab"
    /// （M0 冻结）；同地址去重（更新标题）。保存失败具名报告，盘上旧
    /// 有效写入保留。
    pub fn add_favorite(&mut self) -> Result<(), String> {
        let Some(page) = self.tabs[self.active].committed.clone() else {
            let message = "favorite refused: no committed page".to_string();
            self.log("favorite", "", message.clone());
            return Err(message);
        };
        let bookmark = Bookmark {
            title: page.target.display.clone(),
            address: page.target.canonical.clone(),
            open_as: "tab".to_string(),
        };
        match self
            .bookmarks
            .iter()
            .position(|b| b.address == bookmark.address)
        {
            Some(index) => self.bookmarks[index] = bookmark,
            None => self.bookmarks.push(bookmark),
        }
        if let Err(error) = self.store.save(&self.bookmarks) {
            let message = format!("favorite save failed (disk keeps last valid write): {error}");
            self.errors.push(message.clone());
            self.log("favorite", "", message.clone());
            return Err(message);
        }
        let address = self.tabs[self.active]
            .committed_address()
            .map(|s| s.to_string());
        self.log(
            "favorite",
            address.as_deref().unwrap_or(""),
            format!("saved ({} bookmarks)", self.bookmarks.len()),
        );
        Ok(())
    }

    /// 删除收藏（索引序）；保存失败具名报告。
    pub fn remove_favorite(&mut self, index: usize) -> Result<(), String> {
        if index >= self.bookmarks.len() {
            let message = format!(
                "unfavorite refused: index {index} out of range ({} bookmarks)",
                self.bookmarks.len()
            );
            return Err(message);
        }
        self.bookmarks.remove(index);
        if let Err(error) = self.store.save(&self.bookmarks) {
            let message = format!("unfavorite save failed (disk keeps last valid write): {error}");
            self.errors.push(message.clone());
            return Err(message);
        }
        Ok(())
    }

    /// frame 业务按钮（活动页签 guest；真实 980 typed 队列注入）。
    pub fn click(&mut self, button: usize) -> Result<(), String> {
        self.sessions.click(button)
    }

    // ----- 受控步进（窗口 tick / CPU 测试同一边界） -----

    pub fn install_script(&mut self, script: Vec<(usize, ShellAction)>) {
        self.script = script;
    }

    pub fn script_done(&self) -> bool {
        self.script.iter().all(|(t, _)| *t <= self.tick_no)
    }

    fn run_due_script(&mut self) {
        let due: Vec<ShellAction> = self
            .script
            .iter()
            .filter(|(t, _)| *t == self.tick_no)
            .map(|(_, a)| a.clone())
            .collect();
        for action in due {
            let result = match &action {
                ShellAction::Navigate(input) => self.navigate(input).map(|_| ()),
                ShellAction::Back => self.back(),
                ShellAction::Forward => self.forward(),
                ShellAction::Refresh => self.refresh(),
                ShellAction::Stop => self.stop(),
                ShellAction::NewTab => {
                    self.new_tab();
                    Ok(())
                }
                ShellAction::SelectTab(index) => self.select_tab(*index),
                ShellAction::CloseTab(index) => self.close_tab(*index),
                ShellAction::Click(target) => self.click(*target),
                ShellAction::FavoriteAdd => self.add_favorite(),
                ShellAction::FavoriteDelete(index) => self.remove_favorite(*index),
                ShellAction::ToggleBar => {
                    self.toggle_favorites_bar();
                    Ok(())
                }
                ShellAction::TogglePanel => {
                    self.toggle_favorites_panel();
                    Ok(())
                }
                ShellAction::HoverIn(address) => {
                    self.hover_in(address.clone());
                    Ok(())
                }
                ShellAction::HoverOut => {
                    self.hover_out();
                    Ok(())
                }
            };
            if let Err(error) = result {
                self.errors.push(format!("script {action:?}: {error}"));
            }
        }
    }

    /// 网络完成事件裁决（step 内 drain）：按 `tab_id` 找页签、按代次判
    /// 新鲜度；成功在内容可呈现时一次提交（地址/标题/历史/视口/状态），
    /// 失败具名记录且已提交页面不动。切页签后有效结果仅更新所属页签。
    fn drain_web_events(&mut self) {
        while let Some(crate::ui::autoweb_nav::WebNavEvent::Outcome {
            tab_id,
            generation,
            address,
            result,
        }) = self.web_worker.as_mut().and_then(|w| w.try_recv_event())
        {
            let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
                self.nav_log.push(NavEvent {
                    tick: self.tick_no,
                    tab: tab_id,
                    generation,
                    action: "web-commit",
                    address,
                    outcome: "late completion dropped (tab already closed)".to_string(),
                });
                continue;
            };
            let (current_generation, has_web) = {
                let tab = &self.tabs[index];
                (
                    tab.nav_generation,
                    tab.pending.as_ref().is_some_and(|p| p.web.is_some()),
                )
            };
            // pending 缺席（停止/关页后）或代次不匹配（已被更新请求抢占，
            // pending 属于新请求须保留）→ 迟到完成整票丢弃。
            let fresh = has_web
                && self.tabs[index]
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.generation == generation);
            if !fresh {
                self.nav_log.push(NavEvent {
                    tick: self.tick_no,
                    tab: tab_id,
                    generation,
                    action: "web-commit",
                    address: address.clone(),
                    outcome: format!(
                        "stale completion dropped (generation {generation} vs current \
                         {current_generation}); committed page unchanged"
                    ),
                });
                continue;
            }
            let pending = self.tabs[index]
                .pending
                .take()
                .expect("fresh implies pending");
            let mode = pending.mode;
            let target = pending.target.clone();
            match result {
                Ok(outcome) => {
                    // 网络文档替换任何旧 guest（先释放旧会话再落位）。
                    let old_session = self.tabs[index]
                        .committed
                        .take()
                        .and_then(|page| page.session);
                    if let Some(old) = old_session {
                        if let Err(error) = self.sessions.close_session(old) {
                            self.errors.push(format!("web-commit rotate: {error}"));
                        }
                    }
                    let title = outcome.title.clone();
                    let page = crate::ui::ase_browser_shell::WebDocPage {
                        address: pending.target.canonical.clone(),
                        canonical: outcome.canonical_url.clone(),
                        title: outcome.title,
                        release: outcome.release,
                        app_id: outcome.app_id,
                        body: outcome.document,
                        sha256_hex: outcome.document_sha256_hex,
                    };
                    self.tabs[index].committed = Some(CommittedPage {
                        target: target.clone(),
                        session: None,
                        web: Some(page),
                    });
                    let tab = &mut self.tabs[index];
                    if mode == NavMode::New {
                        let current = tab.history.get(tab.cursor);
                        if current.map(|e| e.address.as_str()) != Some(target.canonical.as_str()) {
                            tab.history.truncate(tab.cursor + 1);
                            tab.history.push(HistoryEntry {
                                address: target.canonical.clone(),
                                title,
                            });
                            tab.cursor = tab.history.len() - 1;
                        }
                    }
                    tab.failure = None;
                    self.nav_log.push(NavEvent {
                        tick: self.tick_no,
                        tab: tab_id,
                        generation,
                        action: "web-commit",
                        address: address.clone(),
                        outcome: format!(
                            "committed (web document, {} request(s), cache_reused={})",
                            outcome.request_count, outcome.cache_reused
                        ),
                    });
                }
                Err(message) => {
                    self.tabs[index].failure = Some(NavFailure {
                        address: address.clone(),
                        message: message.clone(),
                        generation,
                    });
                    self.nav_log.push(NavEvent {
                        tick: self.tick_no,
                        tab: tab_id,
                        generation,
                        action: "web-commit",
                        address: address.clone(),
                        outcome: format!("refused: {message}"),
                    });
                }
            }
        }
    }

    /// 受控步进：到期脚本 → 网络完成事件裁决 → 活动页签进行中本地导航的
    /// 挂载提交 → 会话层步进（AV poll → 真实 handler → 差量 apply；
    /// Clock 到期校时）。
    pub fn step(&mut self) -> StepOutcome {
        self.tick_no += 1;
        self.run_due_script();
        self.drain_web_events();
        self.commit_active_pending();
        self.sessions.step_selected()
    }

    /// 关窗收口：释放全部 guest 会话（引擎 owner + 状态根）；网络请求
    /// 全部置位取消（worker 在通道关闭后随宿主退出）。
    pub fn close_all(&mut self) -> Result<(), String> {
        for tab in &mut self.tabs {
            if let Some(pending) = tab.pending.take() {
                if let Some(cancel) = &pending.web {
                    cancel.store(true, Ordering::Relaxed);
                }
                drop(pending.ticket);
            }
        }
        self.sessions.close_all()
    }

    // ----- 证据 -----

    pub fn session_host(&self) -> &AseSessionHost {
        &self.sessions
    }

    /// 证据 JSON（CPU 测试/窗口 exe 消费）。
    pub fn evidence(&self) -> serde_json::Value {
        let tabs: Vec<serde_json::Value> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                serde_json::json!({
                    "index": index,
                    "id": tab.id,
                    "title": tab.title(),
                    "committed_address": tab.committed_address(),
                    "committed_session": tab.committed.as_ref().and_then(|p| p.session),
                    "pending": tab.pending.as_ref().map(|p| serde_json::json!({
                        "generation": p.generation,
                        "address": p.target.canonical,
                        "mode": format!("{:?}", p.mode),
                        "web": p.web.is_some(),
                    })),
                    "failure": tab.failure.as_ref().map(|f| serde_json::json!({
                        "address": f.address, "message": f.message,
                    })),
                    "committed_web": tab.committed.as_ref().and_then(|page| page.web.as_ref())
                        .map(|web| serde_json::json!({
                            "address": web.address, "canonical": web.canonical,
                            "title": web.title, "release": web.release,
                            "app_id": web.app_id, "sha256": web.sha256_hex,
                            "body_bytes": web.body.len(),
                        })),
                    "nav_generation": tab.nav_generation,
                    "history": tab.history.iter().map(|e| serde_json::json!({
                        "address": e.address, "title": e.title,
                    })).collect::<Vec<_>>(),
                    "cursor": tab.cursor,
                })
            })
            .collect();
        serde_json::json!({
            "apps": self.apps.iter().map(|a| serde_json::json!({
                "identity": a.identity, "display": a.display,
                "kind": match &a.kind {
                    ShellAppKind::Node { path } => serde_json::json!({"node": path}),
                    ShellAppKind::Gallery => serde_json::json!("gallery"),
                    ShellAppKind::WebDoc => serde_json::json!("webdoc"),
                },
            })).collect::<Vec<_>>(),
            "web_in_flight": self.web_in_flight(),
            "fixture_root": self.fixture_root.to_string_lossy(),
            "config_root": self.store.root().to_string_lossy(),
            "tabs": tabs,
            "active_tab": self.active,
            "bookmarks": self.bookmarks,
            "favorites_bar": self.show_favorites_bar,
            "favorites_panel": self.favorites_panel_open,
            "open_menu": self.open_menu.map(|m| format!("{m:?}")),
            "hover_hint": self.hover_hint,
            "status_line": self.status_line(),
            "session_host": self.sessions.evidence(),
            "nav_log": self.nav_log.iter().map(|e| serde_json::json!({
                "tick": e.tick, "tab": e.tab, "generation": e.generation,
                "action": e.action, "address": e.address, "outcome": e.outcome,
            })).collect::<Vec<_>>(),
            "errors": self.errors,
        })
    }
}

// ---------------------------------------------------------------------------
// 窗口组件：七区渲染 + 消息面（与脚本/测试同一 on 边界）
// ---------------------------------------------------------------------------

static SHELL_HOST: OnceLock<RwLock<Option<AutoscapeHost>>> = OnceLock::new();
/// 外壳渲染帧计数（真实渲染观察面）。
pub static AUTOSCAPE_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// 地址栏输入框的确定性 focus id（与 iced 侧 `derive_input_id` 的
/// `auto_input_{placeholder}_{width}_{password}` 格式一致；placeholder 冻结
/// "地址"，无显式 width（经 style w-full 填充）→ 0，非密码 → false）。
pub fn address_input_id() -> String {
    format!("auto_input_{}_{}_{}", ADDRESS_PLACEHOLDER, 0, false)
}

const ADDRESS_PLACEHOLDER: &str = "地址";

/// 安装外壳宿主（窗口进程一次）。
pub fn install_shell_host(host: AutoscapeHost) {
    let cell = SHELL_HOST.get_or_init(|| RwLock::new(None));
    *cell.write().unwrap() = Some(host);
}

/// 访问外壳宿主（窗口 tick/按钮与收口线程共用）。
pub fn with_shell_host<R>(f: impl FnOnce(&mut AutoscapeHost) -> R) -> Option<R> {
    let cell = SHELL_HOST.get()?;
    let mut guard = cell.write().ok()?;
    guard.as_mut().map(f)
}

#[derive(Clone, Debug)]
pub enum ShellMsg {
    MenuToggle(ShellMenu),
    MenuFileNewTab,
    MenuFileCloseTab,
    MenuViewFavoritesBar,
    MenuNavBack,
    MenuNavForward,
    MenuNavRefresh,
    MenuNavStop,
    MenuFavoritesAdd,
    MenuFavoritesPanel,
    AddressFocus,
    AddressEdited(String),
    AddressSubmit,
    AddressBack,
    AddressForward,
    AddressRefreshStop,
    ToolbarFavorite,
    ToolbarPanel,
    ToolbarWindowMenu,
    TabNew,
    TabSelect(usize),
    TabClose(usize),
    OpenAddress(String),
    HoverIn(String),
    HoverOut,
    FavoriteOpen(usize),
    FavoriteDelete(usize),
    RetryFailed,
    FrameClick(usize),
    ShellTick,
}

/// AutoScape 浏览器窗口：单真实宿主七区 + guest AutoFrame。菜单/地址/
/// 页签/收藏只发导航与外壳消息；业务事件入真实 980 typed 队列（与脚本
/// 注入同一边界）；受控步进只在窗口 tick 主线程。退出经系统窗口控制
/// （run_app 返回后由入口收口证据与全部会话）。
#[derive(Debug, Default)]
pub struct AutoscapeWindow;

impl Component for AutoscapeWindow {
    type Msg = ShellMsg;

    fn on(&mut self, msg: ShellMsg) {
        match msg {
            ShellMsg::MenuToggle(menu) => {
                let _ = with_shell_host(|h| h.toggle_menu(menu));
            }
            ShellMsg::MenuFileNewTab | ShellMsg::TabNew => {
                let _ = with_shell_host(|h| {
                    h.new_tab();
                });
            }
            ShellMsg::MenuFileCloseTab => {
                let _ = with_shell_host(|h| h.close_tab(h.active_index()));
            }
            ShellMsg::MenuViewFavoritesBar => {
                let _ = with_shell_host(|h| h.toggle_favorites_bar());
            }
            ShellMsg::MenuNavBack | ShellMsg::AddressBack => {
                let _ = with_shell_host(|h| h.back());
            }
            ShellMsg::MenuNavForward | ShellMsg::AddressForward => {
                let _ = with_shell_host(|h| h.forward());
            }
            ShellMsg::MenuNavRefresh => {
                let _ = with_shell_host(|h| h.refresh());
            }
            ShellMsg::MenuNavStop => {
                let _ = with_shell_host(|h| h.stop());
            }
            ShellMsg::AddressRefreshStop => {
                // 有进行中请求 → 停止；否则刷新（设计：刷新/停止共用）。
                let stopping = with_shell_host(|h| {
                    h.tabs()
                        .get(h.active_index())
                        .is_some_and(|tab| tab.pending.is_some())
                })
                .unwrap_or(false);
                let _ = with_shell_host(|h| if stopping { h.stop() } else { h.refresh() });
            }
            ShellMsg::MenuFavoritesAdd | ShellMsg::ToolbarFavorite => {
                let _ = with_shell_host(|h| h.add_favorite());
            }
            ShellMsg::MenuFavoritesPanel | ShellMsg::ToolbarPanel => {
                let _ = with_shell_host(|h| h.toggle_favorites_panel());
            }
            ShellMsg::ToolbarWindowMenu => {
                let _ = with_shell_host(|h| h.toggle_menu(ShellMenu::Window));
            }
            ShellMsg::AddressFocus => {
                let _ = with_shell_host(|h| h.request_address_focus());
            }
            ShellMsg::AddressEdited(text) => {
                let _ = with_shell_host(|h| h.address_edited(text));
            }
            ShellMsg::AddressSubmit => {
                let _ = with_shell_host(|h| h.submit_address());
            }
            ShellMsg::TabSelect(index) => {
                let _ = with_shell_host(|h| h.select_tab(index));
            }
            ShellMsg::TabClose(index) => {
                let _ = with_shell_host(|h| h.close_tab(index));
            }
            ShellMsg::OpenAddress(address) => {
                let _ = with_shell_host(|h| h.navigate(&address));
            }
            ShellMsg::HoverIn(address) => {
                let _ = with_shell_host(|h| h.hover_in(address));
            }
            ShellMsg::HoverOut => {
                let _ = with_shell_host(|h| h.hover_out());
            }
            ShellMsg::FavoriteOpen(index) => {
                let address =
                    with_shell_host(|h| h.bookmarks().get(index).map(|b| b.address.clone()))
                        .flatten();
                if let Some(address) = address {
                    let _ = with_shell_host(|h| h.navigate(&address));
                }
            }
            ShellMsg::FavoriteDelete(index) => {
                let _ = with_shell_host(|h| h.remove_favorite(index));
            }
            ShellMsg::RetryFailed => {
                let _ = with_shell_host(|h| h.retry_failed());
            }
            ShellMsg::FrameClick(target) => {
                let _ = with_shell_host(|h| h.click(target));
            }
            ShellMsg::ShellTick => {
                let _ = with_shell_host(|h| h.step());
            }
        }
    }

    fn tick_interval_ms(&self) -> Option<u32> {
        Some(20)
    }

    fn tick_msg(&self) -> Option<ShellMsg> {
        Some(ShellMsg::ShellTick)
    }

    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        [
            ("Ctrl+T", "new_tab"),
            ("Ctrl+W", "close_tab"),
            ("Ctrl+D", "favorite_current"),
            ("Ctrl+L", "focus_address"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    fn key_message(&self, key: &str) -> Option<ShellMsg> {
        match key {
            "Ctrl+T" => Some(ShellMsg::TabNew),
            "Ctrl+W" => Some(ShellMsg::MenuFileCloseTab),
            "Ctrl+D" => Some(ShellMsg::ToolbarFavorite),
            "Ctrl+L" => Some(ShellMsg::AddressFocus),
            _ => None,
        }
    }

    fn take_focus_request(&mut self) -> Option<FocusRequest> {
        with_shell_host(|h| {
            if !h.focus_address {
                return None;
            }
            h.focus_address = false;
            let expected = h.focus_epoch.load(Ordering::Acquire);
            Some(FocusRequest::new(
                address_input_id(),
                Arc::clone(&h.focus_epoch),
                expected,
            ))
        })
        .flatten()
    }

    fn view(&self) -> View<ShellMsg> {
        AUTOSCAPE_FRAMES.fetch_add(1, Ordering::Relaxed);
        let snapshot = with_shell_host(|h| {
            (
                h.tabs()
                    .iter()
                    .map(|tab| {
                        (
                            tab.id,
                            tab.title().to_string(),
                            tab.committed_address().map(|s| s.to_string()),
                            tab.pending.is_some(),
                            tab.failure.as_ref().map(|f| f.message.clone()),
                        )
                    })
                    .collect::<Vec<_>>(),
                h.active_index(),
                h.address_editing().map(|s| s.to_string()),
                h.active_tab().committed_address().map(|s| s.to_string()),
                h.active_tab()
                    .committed
                    .as_ref()
                    .and_then(|page| page.web.clone()),
                h.bookmarks().to_vec(),
                h.favorites_bar_visible(),
                h.favorites_panel_open(),
                h.open_menu(),
                h.hover_hint().map(|s| s.to_string()),
                h.status_line(),
                h.apps().to_vec(),
            )
        });
        let Some((
            tabs,
            active,
            editing,
            committed_address,
            web_doc,
            bookmarks,
            show_bar,
            panel_open,
            open_menu,
            hover_hint,
            status_line,
            apps,
        )) = snapshot
        else {
            return View::col()
                .padding(16)
                .child(View::text("AutoScape host not installed"))
                .build();
        };
        let narrow = crate::ui::style::theme::window_width() < NARROW_WINDOW_PX;
        let effective_bar = show_bar && !narrow;

        let mut root = View::col().spacing(6).padding(8);

        // ── 区域 1：标题/菜单栏（窗口标题"AutoScape"由入口设置；菜单行在此）
        //    右端并置区域 3 工具（收藏当前页/收藏面板/窗口菜单 ⋯）。──
        let mut menu_row = View::row().spacing(4);
        menu_row = menu_row.child(View::text_styled("AutoScape", "font-bold"));
        for menu in [
            ShellMenu::File,
            ShellMenu::View,
            ShellMenu::Nav,
            ShellMenu::Favorites,
        ] {
            let style = if open_menu == Some(menu) {
                "px-2 py-1 rounded bg-accent text-accent-foreground"
            } else {
                "px-2 py-1 rounded bg-muted text-foreground"
            };
            menu_row = menu_row.child(View::button_styled(
                menu_label(menu),
                ShellMsg::MenuToggle(menu),
                style,
            ));
        }
        menu_row = menu_row
            .child(View::button_styled(
                "☆",
                ShellMsg::ToolbarFavorite,
                "px-2 py-1 rounded bg-secondary text-secondary-foreground",
            ))
            .child(View::button_styled(
                "收藏",
                ShellMsg::ToolbarPanel,
                "px-2 py-1 rounded bg-secondary text-secondary-foreground",
            ))
            .child(View::button_styled(
                "⋯",
                ShellMsg::ToolbarWindowMenu,
                "px-2 py-1 rounded bg-secondary text-secondary-foreground",
            ));
        root = root.child(menu_row.build());

        // 展开的菜单动作行（再次点击菜单名、切换页签或执行窗口菜单动作收起）。
        if let Some(menu) = open_menu {
            let menu_button = |label: &str, msg: ShellMsg| {
                View::button_styled(label, msg, "px-2 py-1 rounded bg-muted text-foreground")
            };
            let items: Vec<View<ShellMsg>> = match menu {
                ShellMenu::File => vec![
                    menu_button("新建页签", ShellMsg::TabNew),
                    menu_button("关闭页签", ShellMsg::MenuFileCloseTab),
                ],
                ShellMenu::View => vec![menu_button("收藏栏", ShellMsg::MenuViewFavoritesBar)],
                ShellMenu::Nav => vec![
                    menu_button("后退", ShellMsg::AddressBack),
                    menu_button("前进", ShellMsg::AddressForward),
                    menu_button("刷新", ShellMsg::MenuNavRefresh),
                    menu_button("停止", ShellMsg::MenuNavStop),
                ],
                ShellMenu::Favorites => vec![
                    menu_button("添加收藏", ShellMsg::ToolbarFavorite),
                    menu_button("收藏面板", ShellMsg::ToolbarPanel),
                ],
                // 窄窗收纳位：次要工具全部保持可达。
                ShellMenu::Window => vec![
                    menu_button("新建页签", ShellMsg::TabNew),
                    menu_button("关闭页签", ShellMsg::MenuFileCloseTab),
                    menu_button("后退", ShellMsg::AddressBack),
                    menu_button("前进", ShellMsg::AddressForward),
                    menu_button("刷新", ShellMsg::MenuNavRefresh),
                    menu_button("停止", ShellMsg::MenuNavStop),
                    menu_button("收藏栏", ShellMsg::MenuViewFavoritesBar),
                    menu_button("添加收藏", ShellMsg::ToolbarFavorite),
                    menu_button("收藏面板", ShellMsg::ToolbarPanel),
                ],
            };
            let mut row = View::row().spacing(4);
            for item in items {
                row = row.child(item);
            }
            root = root.child(row.build());
        }

        // ── 区域 2：地址栏（后退/前进/刷新-停止 + 输入；Ctrl+L 聚焦）──
        let pending = tabs
            .get(active)
            .map(|(_, _, _, pending, _)| *pending)
            .unwrap_or(false);
        let mut address_row = View::row().spacing(4);
        address_row = address_row
            .child(View::button_styled(
                "‹",
                ShellMsg::AddressBack,
                "px-2 py-1 rounded bg-muted text-foreground",
            ))
            .child(View::button_styled(
                "›",
                ShellMsg::AddressForward,
                "px-2 py-1 rounded bg-muted text-foreground",
            ))
            .child(View::button_styled(
                if pending { "✕" } else { "↻" },
                ShellMsg::AddressRefreshStop,
                "px-2 py-1 rounded bg-muted text-foreground",
            ));
        let shown = editing
            .clone()
            .or_else(|| committed_address.clone())
            .unwrap_or_default();
        address_row = address_row.child(
            View::input(ADDRESS_PLACEHOLDER)
                .value(shown)
                .on_text(ShellMsg::AddressEdited)
                .on_submit(ShellMsg::AddressSubmit)
                .style("w-full")
                .build(),
        );
        root = root.child(address_row.build());

        // ── 区域 4：页签行（窄窗截断标题；次要工具在 ⋯ 菜单）──
        let mut tabs_row = View::row().spacing(4);
        for (index, (id, title, _address, tab_pending, _failure)) in tabs.iter().enumerate() {
            let is_active = index == active;
            let style = if is_active {
                "px-3 py-1 rounded-t-lg bg-accent text-accent-foreground"
            } else {
                "px-3 py-1 rounded-t-lg bg-muted text-foreground"
            };
            let label = if narrow && title.chars().count() > 6 {
                let prefix: String = title.chars().take(6).collect();
                format!("{prefix}…")
            } else {
                title.clone()
            };
            let indicator = if *tab_pending { "…" } else { "" };
            tabs_row = tabs_row.child(View::button_styled(
                format!("{label}{indicator}"),
                ShellMsg::TabSelect(index),
                style,
            ));
            tabs_row = tabs_row.child(View::button_styled(
                format!("×{id}"),
                ShellMsg::TabClose(index),
                "px-1 py-1 rounded bg-destructive text-destructive-foreground",
            ));
        }
        tabs_row = tabs_row.child(View::button_styled(
            "＋",
            ShellMsg::TabNew,
            "px-2 py-1 rounded bg-primary text-primary-foreground",
        ));
        root = root.child(tabs_row.build());

        // ── 区域 5：收藏栏（窄窗自动收纳；⋯ 菜单/查看菜单可再开）──
        if effective_bar {
            let mut bar = View::row().spacing(4);
            if bookmarks.is_empty() {
                bar = bar.child(View::text_styled("收藏栏（空）", "text-sm"));
            }
            for (index, bookmark) in bookmarks.iter().enumerate() {
                bar = bar
                    .child(View::button_styled(
                        bookmark.title.clone(),
                        ShellMsg::FavoriteOpen(index),
                        "px-2 py-1 rounded bg-muted text-foreground",
                    ))
                    .child(View::button_styled(
                        "×",
                        ShellMsg::FavoriteDelete(index),
                        "px-1 py-1 rounded bg-destructive text-destructive-foreground",
                    ));
            }
            root = root.child(bar.build());
        }

        // ── 区域 6：中央内容区（+ 可收起的右侧收藏面板）──
        let mut body = View::row().spacing(6);
        let mut content = View::col().spacing(6);

        // 失败行（独立展示 + 重试入口；已提交内容保留在下方）。
        if let Some((_, _, _, _, Some(failure))) = tabs.get(active) {
            let mut failure_row = View::row().spacing(6);
            failure_row = failure_row
                .child(View::text_styled(
                    format!("导航失败: {failure}"),
                    "text-destructive",
                ))
                .child(View::button_styled(
                    "重试",
                    ShellMsg::RetryFailed,
                    "px-2 py-1 rounded bg-primary text-primary-foreground",
                ));
            content = content.child(failure_row.build());
        }
        if pending {
            content = content.child(View::text_styled("加载中…", "text-sm"));
        }
        // 内容视口：guest AutoFrame（全局 HOST 当前 session 的 AV 真值）或
        // 内建 Gallery 目录（非 guest 原生内容进入同一视口）。
        let committed_is_guest = committed_address.as_deref().is_some_and(|address| {
            apps.iter().any(|app| {
                app.canonical() == address && matches!(app.kind, ShellAppKind::Node { .. })
            })
        });
        if let Some(web) = web_doc {
            // 网络只读文档（PLAN-1119）：来源行 + 已验证 AutoDown 只读视图
            // （渲染前准入已拒图片/HTML/组件指令/越域链接——零隐式资源）。
            let mut doc = View::col().spacing(6).padding(12);
            doc = doc
                .child(View::text_styled(
                    format!("{} — {}（release {}）", web.title, web.address, web.release),
                    "text-sm",
                ))
                .child(View::text_styled(
                    format!("sha256 {}", web.sha256_hex),
                    "text-xs",
                ));
            doc = doc.child(crate::ui::autodown_render::render_document::<ShellMsg>(
                &web.body, true,
            ));
            content = content.child(doc.build());
        } else if committed_is_guest {
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
                    format!("frame session generation {generation}"),
                    "text-sm",
                ));
                for i in 0..texts.len().max(states.len()) {
                    for (leaf, text) in texts.get(i).into_iter().flatten().enumerate() {
                        frame = frame.child(View::text(format!("t{leaf}: {text}")));
                    }
                    for (name, value) in states.get(i).into_iter().flatten() {
                        frame = frame.child(View::text(format!("{name} = {value}")));
                    }
                }
                let mut row = View::row().spacing(6);
                for (target, (label, _msg)) in buttons.iter().enumerate() {
                    row = row.child(View::button_styled(
                        label.clone(),
                        ShellMsg::FrameClick(target),
                        "px-2 py-1 rounded bg-secondary text-secondary-foreground",
                    ));
                }
                frame = frame.child(row.build());
                content = content.child(frame.build());
            }
        } else if committed_address.is_some() {
            // Gallery 目录卡片（元数据；悬停显示目标；点击装载 guest）。
            let mut gallery = View::col().spacing(8).padding(12);
            gallery = gallery.child(View::text_styled("Gallery", "text-lg font-bold"));
            gallery = gallery.child(View::text_styled(
                "本地应用目录（点击卡片打开；此处零装载）",
                "text-sm",
            ));
            let mut cards = View::row().spacing(8);
            for app in &apps {
                if matches!(app.kind, ShellAppKind::Gallery) {
                    continue;
                }
                let canonical = app.canonical();
                let mut card = View::col().spacing(2);
                card = card
                    .child(View::text_styled(app.display.clone(), "font-bold"))
                    .child(View::text_styled(canonical.clone(), "text-sm"));
                let card = card
                    .child(View::button_styled(
                        "打开",
                        ShellMsg::OpenAddress(canonical.clone()),
                        "px-2 py-1 rounded bg-primary text-primary-foreground",
                    ))
                    .build();
                // 悬停命中区：进入/离开 → 状态栏目标提示（七区语义）。
                let hovered = View::MouseArea {
                    content: Box::new(card),
                    on_enter: Some(ShellMsg::HoverIn(canonical.clone())),
                    on_exit: Some(ShellMsg::HoverOut),
                    on_double_click: None,
                    on_click: Some(ShellMsg::OpenAddress(canonical.clone())),
                    on_context_menu: None,
                    on_release: None,
                    on_move: None,
                    logical_extent: None,
                    style: Some(
                        crate::ui::style::Style::parse("p-3 rounded-lg border bg-card")
                            .expect("frozen card style"),
                    ),
                };
                cards = cards.child(hovered);
            }
            gallery = gallery.child(cards.build());
            content = content.child(gallery.build());
        } else {
            content = content.child(View::text("新页签"));
        }

        body = body.child(content.build());
        // 可收起的右侧收藏面板。
        if panel_open {
            let mut panel = View::col().spacing(4).padding(8);
            panel = panel.child(View::text_styled("收藏", "font-bold"));
            if bookmarks.is_empty() {
                panel = panel.child(View::text_styled("（空）", "text-sm"));
            }
            for (index, bookmark) in bookmarks.iter().enumerate() {
                panel = panel.child(View::text_styled(
                    format!("{} — {}", bookmark.title, bookmark.address),
                    "text-sm",
                ));
                let mut row = View::row().spacing(4);
                row = row
                    .child(View::button_styled(
                        "打开",
                        ShellMsg::FavoriteOpen(index),
                        "px-2 py-1 rounded bg-primary text-primary-foreground",
                    ))
                    .child(View::button_styled(
                        "删除",
                        ShellMsg::FavoriteDelete(index),
                        "px-2 py-1 rounded bg-destructive text-destructive-foreground",
                    ));
                panel = panel.child(row.build());
            }
            body = body.child(panel.build());
        }
        root = root.child(body.build());

        // ── 区域 7：底部状态栏（悬停目标 > 错误/加载/已提交来源；右端就绪）──
        let mut status = View::row().spacing(8);
        status = status
            .child(View::text_styled(status_line, "text-sm"))
            .child(View::text_styled(
                if hover_hint.is_some() { "" } else { "就绪" },
                "text-sm",
            ));
        root = root.child(status.build());

        root.build()
    }
}

fn menu_label(menu: ShellMenu) -> &'static str {
    match menu {
        ShellMenu::File => "文件",
        ShellMenu::View => "查看",
        ShellMenu::Nav => "导航",
        ShellMenu::Favorites => "收藏",
        ShellMenu::Window => "⋯",
    }
}
