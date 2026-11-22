//! PLAN-1118 AutoScape 外壳导航永久 case（auto-ui 侧消费面）。
//!
//! 覆盖 V-02（同一导航入口三路、Gallery 零 guest）、V-03（历史/游标/提交、
//! 停止/迟到/关页负例）、V-04（双页签同源隔离、容量拒与重试、Clock 隐藏
//! 零派发+恢复一次校时）、V-05（收藏重启一致、坏配置/写失败可观测）的
//! CPU 档；真窗档（V-01/V-06 的自动面）在 `AUTO_UI_ASE_M0_E2E=1` 排窗
//! 后跑（窗口单例纪律）。真鼠标/键盘/真人走查归手工档（1022/1023/1109
//! 同界分账）。
//!
//! 准备（全部档，不在 case 计时内；2026-11-14 测试纪律——日常用例零
//! rustc）：`cargo build -p auto-ui-runtime --features ace-bridge --example
//! autoscape_window` 后运行 `autoscape_window.exe --prep-only`（ui980
//! provider 预构建/新鲜度缓存；本 worktree target 独立，首跑前必须执行）。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
use auto_ui_runtime::ui::ase_browser_shell::{
    resolve_address, AutoscapeHost, AutoscapeWindow, Bookmark, BookmarkLoad, BookmarkStore,
    ShellAppKind, SHELL_IDENTITY_CLOCK, SHELL_IDENTITY_COUNTER, SHELL_IDENTITY_GALLERY,
};
use auto_ui_runtime::ui::Component;
use std::path::PathBuf;
use std::time::Instant;

/// 非坏配置读取的收藏表（坏配置 case 自行 match 具名错误）。
fn unwrap_bookmarks(load: BookmarkLoad) -> Vec<Bookmark> {
    match load {
        BookmarkLoad::Loaded(list) | BookmarkLoad::Missing(list) => list,
        BookmarkLoad::Corrupt { message, .. } => panic!("unexpected corrupt store: {message}"),
    }
}

const TARGET_INC: usize = 2;

mod tempdir {
    use std::path::PathBuf;
    /// 用例收尾删除临时目录（不依赖外部 crate）。
    pub struct TempDirGuard(pub PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// 真实 fixture 根（显式路径；具名准备失败不静默）：env `ASE_M0_TEXT_ROOT`
/// → 组内 auto-lang 兄弟树（与 1022/1023 同约定）。
fn fixture_root() -> PathBuf {
    let root = match std::env::var("ASE_M0_TEXT_ROOT") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../../auto-lang/experimental/ase-m0/fixtures")
        }
    };
    assert!(
        root.join("counter-a/main.at").is_file() && root.join("clock/main.at").is_file(),
        "preparation failure: ASE fixtures missing (ASE_M0_TEXT_ROOT or the group sibling \
         auto-lang tree): {}",
        root.display()
    );
    root
}

/// 隔离配置目录 + 真实 fixture 根的外壳宿主。
fn shell_host(tag: &str) -> (tempdir::TempDirGuard, AutoscapeHost) {
    let dir = std::env::temp_dir().join(format!("p1118-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp config dir");
    let host = AutoscapeHost::with_roots(fixture_root(), dir.clone());
    (tempdir::TempDirGuard(dir), host)
}

/// 活动页签已提交 guest 的会话 id。
fn active_session(host: &AutoscapeHost) -> Option<usize> {
    host.tabs()
        .get(host.active_index())
        .and_then(|tab| tab.committed.as_ref())
        .and_then(|page| page.session)
}

/// 活动页签已提交地址。
fn active_address(host: &AutoscapeHost) -> Option<String> {
    host.tabs()
        .get(host.active_index())
        .and_then(|tab| tab.committed_address().map(|s| s.to_string()))
}

/// 引擎权威 count（业务真值；经 bridge.states=engine.read_all）。
fn engine_count(host: &AutoscapeHost, session: usize) -> i32 {
    let row = host
        .session_host()
        .engine_states(session)
        .unwrap_or_else(|| panic!("session {session} live"));
    for (name, value) in row {
        if name == "count" {
            if let AceStateValue::Int(i) = value {
                return i;
            }
            panic!("count not int: {value:?}");
        }
    }
    panic!("no count field on session {session}");
}

/// 受理 + 一步提交（真实窗口=下一 tick 挂载；CPU 档同一 step 边界）。
fn navigate_and_commit(host: &mut AutoscapeHost, input: &str) -> Result<(), String> {
    host.navigate(input)?;
    host.step();
    Ok(())
}

/// V-02 预面：地址归一化与严格解析语料（拒绝面与 ase-m0 `uri.rs` 同一
/// 语法源——不复制语法，只镜像语料锚定归一化层）。
#[test]
fn p1118_address_resolution_corpus() {
    let apps = auto_ui_runtime::ui::ase_browser_shell::shell_app_table(&fixture_root());
    let canon = |identity: &str| format!("app://{identity}/");
    // 接受面：完整 URI / 根路由缺省补全 / 显示名大小写不敏感 / 裸 identity。
    for (input, identity) in [
        ("app://auto.demo.counter/", SHELL_IDENTITY_COUNTER),
        ("app://auto.demo.counter", SHELL_IDENTITY_COUNTER),
        ("Counter", SHELL_IDENTITY_COUNTER),
        ("counter", SHELL_IDENTITY_COUNTER),
        ("  app://auto.demo.clock/  ", SHELL_IDENTITY_CLOCK),
        ("Clock", SHELL_IDENTITY_CLOCK),
        ("auto.demo.clock", SHELL_IDENTITY_CLOCK),
        ("Gallery", SHELL_IDENTITY_GALLERY),
        ("app://auto.gallery/", SHELL_IDENTITY_GALLERY),
    ] {
        let target = resolve_address(input, &apps)
            .unwrap_or_else(|e| panic!("{input:?} should resolve: {e:?}"));
        assert_eq!(target.identity, identity, "{input:?}");
        assert_eq!(target.canonical, canon(identity), "{input:?}");
    }
    // 拒绝面（具名；不动已提交状态由导航 case 证明）。
    for bad in [
        "",
        "   ",
        "http://auto.demo.counter/",
        "app://Auto.Demo.Counter/",
        "app:///",
        "app://user@auto.demo.counter/",
        "app://auto.demo.counter:8080/",
        "app://auto.demo.counter/?q=1",
        "app://auto.demo.counter/#frag",
        "app://auto.demo.counter/ui",
        "app://auto.unknown.app/",
        "unknown-app",
        "app://auto.gallery/x",
    ] {
        assert!(
            resolve_address(bad, &apps).is_err(),
            "{bad:?} must be refused"
        );
    }
    // 冻结表形状：三项，Gallery 为内建目录（零装载源）。
    assert_eq!(apps.len(), 3);
    assert!(apps
        .iter()
        .any(|a| a.identity == SHELL_IDENTITY_GALLERY && matches!(a.kind, ShellAppKind::Gallery)));
    eprintln!(
        "p1118_address_resolution_corpus elapsed: {:?}",
        Instant::now().elapsed()
    );
}

/// V-02：新建/目录首页零 guest——只进入 Gallery（新建页签与显式目录导航）
/// 不产生编译、session 或装载计数。
#[test]
fn p1118_v02_gallery_home_zero_guests() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("gallery-zero");
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.gallery/"),
        "new tab commits the gallery directory"
    );
    assert_eq!(
        host.session_host().metrics.loads,
        0,
        "zero loads on gallery home"
    );
    assert!(host.tabs().len() == 1);
    // 再显式经地址进入 Gallery（另一新页签）——仍零装载。
    host.new_tab();
    navigate_and_commit(&mut host, "Gallery").expect("gallery nav");
    assert_eq!(
        host.session_host().metrics.loads,
        0,
        "directory nav stays zero-load"
    );
    assert_eq!(
        host.session_host().live_ids().len(),
        0,
        "zero live sessions"
    );
    assert!(host.errors.is_empty(), "errors: {:?}", host.errors);
    eprintln!(
        "p1118_v02_gallery_home_zero_guests elapsed: {:?}",
        started.elapsed()
    );
}

/// V-02：地址栏 / 目录卡片（同一 OpenAddress 入口）/ 收藏三路到达同一
/// resolver 目标；真实 Counter handler 经 typed 队列执行（引擎真值 3）。
#[test]
fn p1118_v02_three_entry_points_reach_real_counter() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("three-entries");
    // 1) 地址栏（显示名归一化）。
    navigate_and_commit(&mut host, "Counter").expect("address entry");
    let session_a = active_session(&host).expect("counter session after address entry");
    for _ in 0..3 {
        host.click(TARGET_INC).expect("click inc");
        host.step();
    }
    assert_eq!(
        engine_count(&host, session_a),
        3,
        "real handler ran 3 times"
    );
    let counter_canonical = "app://auto.demo.counter/";
    assert_eq!(active_address(&host).as_deref(), Some(counter_canonical));

    // 2) 目录卡片入口：回 Gallery 后经同一 OpenAddress 路径再达 Counter。
    navigate_and_commit(&mut host, "app://auto.gallery/").expect("back to gallery");
    assert_eq!(
        host.session_host().metrics.loads,
        1,
        "gallery nav adds no load"
    );
    navigate_and_commit(&mut host, counter_canonical).expect("card entry");
    assert_eq!(active_address(&host).as_deref(), Some(counter_canonical));
    // 同页签替换导航释放旧会话：仍是同一 identity 的新会话。
    let session_b = active_session(&host).expect("counter session after card entry");
    assert_ne!(session_a, session_b, "replacement opens a fresh session");

    // 3) 收藏入口：收藏当前页 → 新页签经收藏打开同一目标。
    host.add_favorite().expect("favorite current");
    assert_eq!(host.bookmarks().len(), 1);
    assert_eq!(host.bookmarks()[0].address, counter_canonical);
    host.new_tab();
    let favorite_address = host.bookmarks()[0].address.clone();
    navigate_and_commit(&mut host, &favorite_address).expect("favorite entry");
    assert_eq!(active_address(&host).as_deref(), Some(counter_canonical));
    eprintln!(
        "p1118_v02_three_entry_points_reach_real_counter elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03：原生导航生产路径 Gallery→Counter→Clock→后退→后退→前进；每页
/// 完整历史、游标与已提交地址；边缘 noop。
#[test]
fn p1118_v03_history_cursor_semantics() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("history");
    navigate_and_commit(&mut host, "app://auto.demo.counter/").expect("counter");
    navigate_and_commit(&mut host, "app://auto.demo.clock/").expect("clock");
    let tab = host.tabs().first().unwrap();
    let addresses: Vec<&str> = tab.history.iter().map(|e| e.address.as_str()).collect();
    assert_eq!(
        addresses,
        vec![
            "app://auto.gallery/",
            "app://auto.demo.counter/",
            "app://auto.demo.clock/"
        ],
        "full per-tab history"
    );
    assert_eq!(tab.cursor, 2);
    // 后退：Clock 会话已随 Counter? 不——后退到 Counter：Counter 会话在
    // Clock 提交时被释放 → 按原地址显式重开（不加历史、游标先行落位）。
    host.back().expect("back to counter");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/")
    );
    assert_eq!(
        host.tabs()[0].cursor,
        1,
        "cursor moved without history change"
    );
    assert_eq!(
        host.tabs()[0].history.len(),
        3,
        "reopen adds no history entry"
    );
    // 再后退到 Gallery（无 guest；保留游标语义）。
    host.back().expect("back to gallery");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.gallery/")
    );
    assert_eq!(host.tabs()[0].cursor, 0);
    // 边缘 noop。
    host.back().expect("edge back noop");
    assert_eq!(host.tabs()[0].cursor, 0);
    // 前进回 Counter（显式重开）。
    host.forward().expect("forward to counter");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/")
    );
    assert_eq!(host.tabs()[0].cursor, 1);
    assert_eq!(
        host.tabs()[0].history.len(),
        3,
        "forward adds no history entry"
    );
    eprintln!(
        "p1118_v03_history_cursor_semantics elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03：新目标截断前进记录——Gallery→Counter→后退（Gallery）→新目标
/// Clock：前进记录清空、游标在尾。
#[test]
fn p1118_v03_new_target_truncates_forward() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("truncate");
    navigate_and_commit(&mut host, "app://auto.demo.counter/").expect("counter");
    host.back().expect("back to gallery");
    host.step();
    assert_eq!(host.tabs()[0].cursor, 0);
    navigate_and_commit(&mut host, "app://auto.demo.clock/").expect("clock");
    let tab = host.tabs().first().unwrap();
    assert_eq!(
        tab.history
            .iter()
            .map(|e| e.address.as_str())
            .collect::<Vec<_>>(),
        vec!["app://auto.gallery/", "app://auto.demo.clock/"],
        "forward records truncated by the new target"
    );
    assert_eq!(tab.cursor, tab.history.len() - 1);
    host.forward().expect("forward at edge noop");
    assert_eq!(host.tabs()[0].cursor, 1);
    eprintln!(
        "p1118_v03_new_target_truncates_forward elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03：刷新当前目标不加历史、会话换新（释放旧 owner、种子 0）。
#[test]
fn p1118_v03_refresh_no_history_fresh_session() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("refresh");
    navigate_and_commit(&mut host, "Counter").expect("counter");
    let first = active_session(&host).expect("first session");
    for _ in 0..2 {
        host.click(TARGET_INC).expect("inc");
        host.step();
    }
    assert_eq!(engine_count(&host, first), 2);
    let history_len = host.tabs()[0].history.len();
    navigate_and_commit(&mut host, "app://auto.demo.counter/").expect("refresh (same address)");
    let second = active_session(&host).expect("second session");
    assert_ne!(first, second, "refresh opens a fresh session");
    assert_eq!(
        engine_count(&host, second),
        0,
        "fresh session starts from seed 0"
    );
    assert_eq!(
        host.tabs()[0].history.len(),
        history_len,
        "refresh adds no history entry"
    );
    assert_eq!(host.tabs()[0].cursor, history_len - 1);
    eprintln!(
        "p1118_v03_refresh_no_history_fresh_session elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03 负控：停止使进行中请求整票失效——票据 drop（引擎释放、零
/// session/零 UI 效果），已提交页面与历史不动。
#[test]
fn p1118_v03_stop_invalidates_pending() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("stop");
    navigate_and_commit(&mut host, "Counter").expect("counter first");
    let session = active_session(&host).expect("committed counter");
    // 受理新导航（Clock）但不提交 → 停止。
    host.navigate("Clock").expect("stage clock");
    assert!(
        host.tabs()[0].pending.is_some(),
        "request staged before commit"
    );
    let staged_generation = host.tabs()[0].nav_generation;
    host.stop().expect("stop");
    assert!(host.tabs()[0].pending.is_none(), "pending dropped");
    assert_eq!(
        host.tabs()[0].nav_generation,
        staged_generation + 1,
        "stop advanced the request generation"
    );
    host.step();
    // 已提交 Counter 不被旧完成覆盖（迟到票据已随 stop drop）。
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/")
    );
    assert_eq!(
        active_session(&host),
        Some(session),
        "committed session retained"
    );
    eprintln!(
        "p1118_v03_stop_invalidates_pending elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03 负控：同页新请求使旧完成失效——A 受理后 B 抢占，只有 B 提交；
/// A 的票据被 drop，A 的装载不产生 session（迟到完成不污染地址/目标）。
#[test]
fn p1118_v03_superseded_request_only_newest_commits() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("supersede");
    host.navigate("Counter").expect("stage counter");
    let loads_after_stage = host.session_host().metrics.loads;
    assert_eq!(
        loads_after_stage, 1,
        "staged request performed the real load"
    );
    host.navigate("Clock").expect("supersede with clock");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.clock/"),
        "only the newest request commits"
    );
    assert_eq!(
        host.session_host().live_ids().len(),
        1,
        "superseded ticket never mounted"
    );
    let supersede_logged = host
        .nav_log
        .iter()
        .any(|e| e.outcome.contains("superseded by new request"));
    assert!(
        supersede_logged,
        "supersede is evidenced: {:?}",
        host.nav_log
    );
    eprintln!(
        "p1118_v03_superseded_request_only_newest_commits elapsed: {:?}",
        started.elapsed()
    );
}

/// V-03 负控：关页使进行中请求失效——票据 drop、会话释放；剩余页签提交
/// 不受晚到完成影响。
#[test]
fn p1118_v03_close_tab_invalidates_pending() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("close-pending");
    host.navigate("Counter").expect("stage counter");
    assert!(host.tabs()[0].pending.is_some());
    host.close_tab(0).expect("close the staging tab");
    assert!(host.tabs().len() == 1, "a fresh gallery tab takes over");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.gallery/"),
        "late completion of the closed tab never mounts"
    );
    assert_eq!(host.session_host().live_ids().len(), 0);
    eprintln!(
        "p1118_v03_close_tab_invalidates_pending elapsed: {:?}",
        started.elapsed()
    );
}

/// V-04：容量上限——两活会话后第三请求具名 `session.limit` 拒，已提交
/// 页面保留；释放后重试成功。
#[test]
fn p1118_v04_capacity_refusal_then_retry() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("capacity");
    navigate_and_commit(&mut host, "Counter").expect("tab0 counter");
    host.new_tab(); // tab1（成为活动页签）
    navigate_and_commit(&mut host, "Clock").expect("tab1 clock");
    assert_eq!(host.session_host().live_ids().len(), 2);
    // 切回 tab0：其上再开 guest——容量不足具名拒，已提交 Counter 保留。
    host.select_tab(0).expect("back to tab0");
    let err = host
        .navigate("Clock")
        .expect_err("capacity refusal expected");
    assert!(err.contains("session.limit"), "named refusal: {err}");
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/"),
        "committed page retained on capacity refusal"
    );
    assert!(
        host.tabs()[0].failure.is_some(),
        "failure recorded for retry"
    );
    // 释放 tab1 → 重试成功。
    host.close_tab(1).expect("release tab1");
    assert_eq!(host.session_host().live_ids().len(), 1);
    host.retry_failed().expect("retry after release");
    host.step();
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.clock/"),
        "retry reaches the target after release"
    );
    eprintln!(
        "p1118_v04_capacity_refusal_then_retry elapsed: {:?}",
        started.elapsed()
    );
}

/// V-04：两个并存页签、两份同源 Counter 隔离（3/7 经真实队列）；切回保
/// 留（引擎常驻、切换零装载）。
#[test]
fn p1118_v04_two_tabs_isolated_same_source_counters() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("isolation");
    navigate_and_commit(&mut host, "Counter").expect("tab0 counter");
    let a = active_session(&host).expect("tab0 session");
    host.new_tab();
    navigate_and_commit(&mut host, "app://auto.demo.counter/").expect("tab1 same-source counter");
    let b = active_session(&host).expect("tab1 session");
    assert_ne!(a, b, "independent sessions for the same source");
    let loads_pinned = host.session_host().metrics.loads;

    for _ in 0..3 {
        host.click(TARGET_INC).expect("tab1 inc");
        host.step();
    }
    assert_eq!(engine_count(&host, b), 3, "tab1 engine authority = 3");

    host.select_tab(0).expect("switch to tab0");
    for _ in 0..7 {
        host.click(TARGET_INC).expect("tab0 inc");
        host.step();
    }
    assert_eq!(engine_count(&host, a), 7, "tab0 engine authority = 7");
    assert_eq!(
        host.session_host().metrics.loads,
        loads_pinned,
        "switching reuses resident engines (zero reload)"
    );
    eprintln!(
        "p1118_v04_two_tabs_isolated_same_source_counters elapsed: {:?}",
        started.elapsed()
    );
}

/// V-04：切回仍驻留页签复用会话；Clock 隐藏期零业务派发，恢复只见一次
/// 新校时（980/1109 语义：选中车道 + 首个新样本）。
#[test]
fn p1118_v04_clock_hidden_zero_dispatch_resume_one_sample() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("clock-resume");
    navigate_and_commit(&mut host, "Clock").expect("clock");
    let clock_session = active_session(&host).expect("clock session");
    // 首个样本送达（恢复即一条新校时）。
    host.step();
    let delivered_after_open = host.session_host().evidence()["clock"]["delivered"]
        .as_u64()
        .unwrap();
    assert_eq!(delivered_after_open, 1, "first calibration delivered once");

    // 隐藏期：切到新 Gallery 页签，多轮 step 零派发/零校时。
    host.new_tab();
    let dispatches_hidden_start = host
        .session_host()
        .dispatches_of(clock_session)
        .unwrap_or(0);
    for _ in 0..5 {
        host.step();
    }
    assert_eq!(
        host.session_host().dispatches_of(clock_session),
        Some(dispatches_hidden_start),
        "hidden clock runs zero business dispatches"
    );
    assert_eq!(
        host.session_host().evidence()["clock"]["delivered"].as_u64(),
        Some(delivered_after_open),
        "no clock sample while hidden"
    );

    // 切回：同一会话（引擎常驻）+ 恰一条新校时。
    host.select_tab(0).expect("back to clock tab");
    assert_eq!(
        active_session(&host),
        Some(clock_session),
        "same session resumed"
    );
    host.step();
    assert_eq!(
        host.session_host().evidence()["clock"]["delivered"].as_u64(),
        Some(delivered_after_open + 1),
        "resume delivers exactly one new calibration"
    );
    eprintln!(
        "p1118_v04_clock_hidden_zero_dispatch_resume_one_sample elapsed: {:?}",
        started.elapsed()
    );
}

/// V-05：添加/删除收藏经重启一致（隔离配置目录，两代宿主实例）。
#[test]
fn p1118_v05_favorites_roundtrip_across_restart() {
    let started = Instant::now();
    let dir = std::env::temp_dir().join(format!("p1118-fav-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let guard = tempdir::TempDirGuard(dir.clone());
    let root = fixture_root();

    let mut host = AutoscapeHost::with_roots(root.clone(), dir.clone());
    navigate_and_commit(&mut host, "Counter").expect("counter");
    host.add_favorite().expect("favorite counter");
    navigate_and_commit(&mut host, "Clock").expect("clock");
    host.add_favorite().expect("favorite clock");
    assert_eq!(host.bookmarks().len(), 2);
    assert_eq!(
        host.bookmarks()[0],
        Bookmark {
            title: "Counter".into(),
            address: "app://auto.demo.counter/".into(),
            open_as: "tab".into(),
        }
    );
    host.remove_favorite(0).expect("remove counter favorite");
    assert_eq!(host.bookmarks().len(), 1);
    drop(host);

    // 重启（新进程面：新宿主实例读同一配置目录）。
    let restarted = AutoscapeHost::with_roots(root, dir.clone());
    assert_eq!(restarted.bookmarks().len(), 1, "favorites survive restart");
    assert_eq!(restarted.bookmarks()[0].title, "Clock");
    assert!(
        restarted.errors.is_empty(),
        "errors: {:?}",
        restarted.errors
    );
    drop(restarted);
    drop(guard);
    eprintln!(
        "p1118_v05_favorites_roundtrip_across_restart elapsed: {:?}",
        started.elapsed()
    );
}

/// V-05：坏配置具名记录 + 空表可用；盘上旧有效写入不被覆盖/可读回。
#[test]
fn p1118_v05_corrupt_config_named_error_old_write_readable() {
    let started = Instant::now();
    let dir = std::env::temp_dir().join(format!("p1118-corrupt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let guard = tempdir::TempDirGuard(dir.clone());
    // 旧有效写入。
    let good = r#"[{"title":"Clock","address":"app://auto.demo.clock/","open_as":"tab"}]"#;
    std::fs::write(dir.join("bookmarks.json"), good).unwrap();
    // 坏配置：同目录换成损坏内容。
    std::fs::write(dir.join("bookmarks.json"), "{not json").unwrap();
    let mut host = AutoscapeHost::with_roots(fixture_root(), dir.clone());
    assert!(
        host.errors.iter().any(|e| e.contains("bookmarks:")),
        "corrupt config is named: {:?}",
        host.errors
    );
    assert!(
        host.bookmarks().is_empty(),
        "usable empty list on corrupt config"
    );
    // 坏配置不阻断外壳运行：导航与收藏仍工作（修复性保存）。
    navigate_and_commit(&mut host, "Clock").expect("clock under corrupt config");
    host.add_favorite().expect("repairing save");
    assert_eq!(host.bookmarks().len(), 1);
    // 新实例读回修复后的有效写入。
    let restarted = AutoscapeHost::with_roots(fixture_root(), dir.clone());
    assert_eq!(restarted.bookmarks().len(), 1);
    assert!(
        restarted.errors.is_empty(),
        "errors: {:?}",
        restarted.errors
    );
    drop(restarted);
    drop(guard);
    eprintln!(
        "p1118_v05_corrupt_config_named_error_old_write_readable elapsed: {:?}",
        started.elapsed()
    );
}

/// V-05：写入失败可观测——配置根不可写（普通文件占位目录名）时宿主保存
/// 具名失败并保留内存态；盘上旧有效写入未被破坏（有效根仍可读回）。
#[test]
fn p1118_v05_write_failure_observable() {
    let started = Instant::now();
    let dir = std::env::temp_dir().join(format!("p1118-wfail-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let guard = tempdir::TempDirGuard(dir.clone());
    // 先在有效根完成一次成功写入。
    let mut host = AutoscapeHost::with_roots(fixture_root(), dir.clone());
    navigate_and_commit(&mut host, "Clock").expect("clock");
    host.add_favorite().expect("first save ok");
    let valid = std::fs::read(dir.join("bookmarks.json")).unwrap();
    // 阻断根：普通文件占位目录名 → create_dir_all 失败。
    let blocked = dir.join("blocked");
    std::fs::write(&blocked, b"not a dir").unwrap();
    // 存储层：保存具名失败。
    assert!(
        BookmarkStore::new(blocked.clone())
            .save(&[Bookmark {
                title: "X".into(),
                address: "app://auto.gallery/".into(),
                open_as: "tab".into()
            }])
            .is_err(),
        "save against a blocked root must fail"
    );
    // 宿主层：配置根被阻断的宿主收藏 → 具名错误入 errors。
    let mut blocked_host = AutoscapeHost::with_roots(fixture_root(), blocked);
    navigate_and_commit(&mut blocked_host, "Clock").expect("clock under blocked root");
    let error = blocked_host.add_favorite().expect_err("blocked save fails");
    assert!(
        error.contains("favorite save failed"),
        "named failure: {error}"
    );
    // 旧有效写入未被破坏；有效根仍可读回。
    assert_eq!(std::fs::read(dir.join("bookmarks.json")).unwrap(), valid);
    assert_eq!(
        unwrap_bookmarks(BookmarkStore::new(dir.clone()).load()).len(),
        1,
        "last valid write still readable"
    );
    drop(host);
    drop(blocked_host);
    drop(guard);
    eprintln!(
        "p1118_v05_write_failure_observable elapsed: {:?}",
        started.elapsed()
    );
}

/// V-01 CPU 面：地址栏编辑与悬停提示不提前替换已提交来源/状态。
#[test]
fn p1118_v01_editing_and_hover_keep_committed_source() {
    let started = Instant::now();
    let (_guard, mut host) = shell_host("edit-hover");
    navigate_and_commit(&mut host, "Counter").expect("counter");
    host.address_edited("app://auto.unknown/".to_string());
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/"),
        "editing never touches the committed source"
    );
    host.hover_in("app://auto.demo.clock/".to_string());
    assert!(host.status_line().contains("目标: app://auto.demo.clock/"));
    host.hover_out();
    assert!(
        host.status_line().contains("app://auto.demo.counter/"),
        "status returns to the committed source after hover-out: {}",
        host.status_line()
    );
    // 编辑坏地址提交 → 失败可观测，已提交地址不变。
    let err = host.submit_address().expect_err("unknown address refused");
    assert!(err.contains("not registered"), "named refusal: {err}");
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/")
    );
    assert!(
        host.tabs()[0].failure.is_some(),
        "failure surfaced for retry"
    );
    host.retry_failed()
        .expect_err("retry still unknown → refused");
    assert_eq!(
        active_address(&host).as_deref(),
        Some("app://auto.demo.counter/"),
        "committed page survives failed retry"
    );
    eprintln!(
        "p1118_v01_editing_and_hover_keep_committed_source elapsed: {:?}",
        started.elapsed()
    );
}

/// V-01 CPU 面：快捷键绑定面（Ctrl+T/W/D/L → 组件消息）与 Ctrl+L 聚焦
/// 请求一次性消费（确定性 input id；投递面由共享 keyboard_subscription
/// 的声明式扩展承载，真窗注入归排窗/手工档）。
#[test]
fn p1118_v01_component_bindings_and_address_focus() {
    let started = Instant::now();
    let (_guard, host) = shell_host("bindings");
    auto_ui_runtime::ui::ase_browser_shell::install_shell_host(host);
    let mut window = AutoscapeWindow;
    let bindings = window.key_bindings();
    for key in ["Ctrl+T", "Ctrl+W", "Ctrl+D", "Ctrl+L"] {
        assert!(bindings.contains_key(key), "{key} declared");
        assert!(window.key_message(key).is_some(), "{key} maps to a message");
    }
    assert!(
        window.key_message("Ctrl+X").is_none(),
        "undeclared combo inert"
    );
    // Ctrl+L：聚焦请求一次性消费。
    assert!(
        auto_ui_runtime::ui::component::Component::take_focus_request(&mut window).is_none(),
        "no focus request before Ctrl+L"
    );
    window.on(auto_ui_runtime::ui::ase_browser_shell::ShellMsg::AddressFocus);
    assert!(
        auto_ui_runtime::ui::component::Component::take_focus_request(&mut window).is_some(),
        "Ctrl+L issues the address focus request"
    );
    assert!(
        auto_ui_runtime::ui::component::Component::take_focus_request(&mut window).is_none(),
        "focus request consumed exactly once"
    );
    eprintln!(
        "p1118_v01_component_bindings_and_address_focus elapsed: {:?}",
        started.elapsed()
    );
}

/// 窗口档（排窗 `AUTO_UI_ASE_M0_E2E=1` 单实例；V-01/V-06 自动面）：常用
/// 与窄窗（`AUTO_VM_WINDOW=520x700`）两档真窗跑同一导航/收藏脚本；断言
/// 帧数、历史/收藏证据与零错误。真鼠标/键盘注入与真人走查归手工档。
#[test]
fn p1118_window_e2e_navigation_normal_and_narrow() {
    if std::env::var("AUTO_UI_ASE_M0_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p1118_window_e2e_navigation_normal_and_narrow: not scheduled (set \
             AUTO_UI_ASE_M0_E2E=1 after the prep stage; window single-instance discipline)"
        );
        return;
    }
    let started = Instant::now();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/autoscape_window.exe")
        .canonicalize()
        .unwrap();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — run the prep stage first",
        exe.display()
    );
    let script = "2:open:Counter,5:click:2,7:click:2,9:open:app://auto.demo.clock/,\
                  14:back,18:forward,22:favadd,26:panel,30:bar,34:stop,40:hoverin:app://auto.demo.clock/,44:hoverout";
    for (tag, window_env) in [
        ("normal", None),
        ("narrow", Some(("AUTO_VM_WINDOW", "520x700"))),
    ] {
        let tmp = std::env::temp_dir().join(format!("p1118-e2e-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let ev_path = tmp.join("evidence.json");
        let mut cmd = std::process::Command::new(&exe);
        cmd.arg("--evidence")
            .arg(&ev_path)
            .arg("--config-home")
            .arg(tmp.join("config"))
            .arg("--script")
            .arg(script)
            .arg("--hold-ms")
            .arg("8000")
            .arg("--min-frames")
            .arg("20")
            .current_dir(&tmp);
        if let Some((key, value)) = window_env {
            cmd.env(key, value);
        }
        let mut child = cmd.spawn().expect("spawn autoscape window exe");
        let out = child.wait().expect("wait autoscape window exe");
        assert!(out.success(), "window exe failed ({tag})");
        let ev: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap()).unwrap();
        assert!(
            ev["frames_rendered"].as_u64().unwrap_or(0) >= 20,
            "{tag}: real window kept rendering: {}",
            ev["frames_rendered"]
        );
        assert!(
            ev["errors"].as_array().unwrap().is_empty(),
            "{tag}: errors: {}",
            ev["errors"]
        );
        let tabs = ev["tabs"].as_array().unwrap();
        assert_eq!(tabs.len(), 1, "{tag}: single scripted tab");
        let history: Vec<&str> = tabs[0]["history"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["address"].as_str().unwrap())
            .collect();
        assert_eq!(
            history,
            vec![
                "app://auto.gallery/",
                "app://auto.demo.counter/",
                "app://auto.demo.clock/"
            ],
            "{tag}: full navigation history"
        );
        assert_eq!(
            tabs[0]["committed_address"].as_str(),
            Some("app://auto.demo.clock/"),
            "{tag}: forward re-committed the clock page"
        );
        assert_eq!(
            ev["bookmarks"].as_array().unwrap().len(),
            1,
            "{tag}: favorite persisted in the isolated config home"
        );
        // 真实业务执行证据：前进重开的 Clock 是唯一活会话；Counter 各代次
        // 均已释放（closed 在案）。
        let host_ev = &ev["session_host"];
        let live = host_ev["live"].as_array().unwrap();
        assert_eq!(
            live.len(),
            1,
            "{tag}: only the re-opened clock stays live: {live:?}"
        );
        assert_eq!(
            live[0]["app"].as_str(),
            Some("auto.demo.clock"),
            "{tag}: live session is the clock"
        );
        let closed = host_ev["closed"].as_array().unwrap();
        assert!(
            closed
                .iter()
                .any(|s| s["app"].as_str() == Some("auto.demo.counter")),
            "{tag}: counter generations released: {closed:?}"
        );
        let nav = ev["nav_log"].as_array().unwrap();
        assert!(
            nav.iter().any(|e| e["action"] == "stop"),
            "{tag}: stop evidenced"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
    eprintln!(
        "p1118_window_e2e_navigation_normal_and_narrow elapsed: {:?}",
        started.elapsed()
    );
}

/// 语料锚定：外壳冻结表 = shell_app_table（Counter/Clock/Gallery）。
#[test]
fn p1118_frozen_table_shape() {
    let apps = auto_ui_runtime::ui::ase_browser_shell::shell_app_table(&fixture_root());
    let identities: Vec<&str> = apps.iter().map(|a| a.identity.as_str()).collect();
    assert_eq!(
        identities,
        vec![
            SHELL_IDENTITY_COUNTER,
            SHELL_IDENTITY_CLOCK,
            SHELL_IDENTITY_GALLERY
        ]
    );
}
