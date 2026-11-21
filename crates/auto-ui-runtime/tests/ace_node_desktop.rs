//! PLAN-980 NATIVE r13（MIN-A-UI）永久 case。
//!
//! 分层：CPU 面（恒跑）——三类源装载（字面量 Hello / 函数中文 / 002 派生
//! Node Counter）、计数轨迹 0→1→0→0 与两实例定向 2:1（业务全部经 AV 场景
//! 重放，宿主只路由 typed 事件）、坏 Schema（未绑定 target/未声明 msg）/
//! 坏 ABI（未知 handle/越槽）/坏权限（provider 撤权）在 UI 效果前具名拒、
//! 关闭/重开/late 事件世代纪律、**byte_len-only 车道不是 UI dispatch** 反例；
//! 窗口面（`AUTO_UI_ACE_NODE_E2E=1` 排窗，单实例串行）——真窗 + FFI 实效果
//! + typed 事件回 AV + 证据 JSON。
//!
//! 交互档边界（具名）：真实鼠标/键盘注入归手工/环境门控档（仓内 t5_smoke/
//! plan804 先例）；本文件的 typed 事件注入与窗口点击同一边界（设备输入进入
//! typed 队列）。001/002 完整主题/选择/全部组件归 W1 B-AV。
//!
//! 准备（e2e 档）：`cargo build -p auto-ui-runtime --features ace-bridge
//! --example ace_node_desktop_window` + `ace_node_desktop_window.exe --prep-only
//! --evidence <any>`（provider 预构建/缓存）——不在 case 计时内。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_hello::stable_f0_root;
use auto_ui_runtime::ui::ace_node_desktop::{
    ensure_ui980_provider, host_session_display, host_session_stats, push_event, push_late_event,
    ui980_status, NodeDesktopBridge, ScriptStep, StepOutcome, UI980_PROVIDER_NAME,
};

fn hello_fixture(name: &str) -> (String, String) {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest
        .join("../../examples/ui/ace-hello-minimal/src/front")
        .join(name);
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{name} readable: {e}"));
    let path = at.to_string_lossy().to_string();
    (src, path)
}

fn counter_fixture() -> (String, String) {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/ace-node-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("counter readable: {e}"));
    let path = at.to_string_lossy().to_string();
    (src, path)
}

#[test]
fn p107_window_deadline_fails_with_partial_evidence() {
    if std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref() != Some("1") {
        eprintln!("p107_window_deadline_fails_with_partial_evidence: window lane not scheduled");
        return;
    }

    std::env::var("ACE_NODE_DESKTOP_PREPARED_RUSTC")
        .expect("prepare the ui980 provider and set ACE_NODE_DESKTOP_PREPARED_RUSTC");
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = manifest.join("../../target/debug/examples/ace_node_desktop_window.exe");
    let (_, source) = counter_fixture();
    let directory = tempfile::tempdir().expect("evidence directory");
    let evidence = directory.path().join("partial.json");
    let min_frames = 1_000_000_u64;
    let output = std::process::Command::new(executable)
        .args(["--source", &source, "--instances", "1", "--hold-ms", "1000"])
        .arg("--min-frames")
        .arg(min_frames.to_string())
        .arg("--evidence")
        .arg(&evidence)
        .output()
        .expect("run the prepared desktop window example");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(6), "{stderr}");
    assert!(stderr.contains("deadline reached"), "{stderr}");
    let partial: serde_json::Value =
        serde_json::from_slice(&std::fs::read(evidence).expect("partial window evidence"))
            .expect("valid window evidence JSON");
    assert_eq!(partial["node"], "Counter");
    assert!(partial["frames_rendered"].as_u64().unwrap() < min_frames);
}

/// 按钮声明序（fixture 冻结）：0=`-`(Dec)、1=Reset、2=`+`(Inc)。
const TARGET_DEC: usize = 0;
const TARGET_RESET: usize = 1;
const TARGET_INC: usize = 2;

fn count_of(bridge: &NodeDesktopBridge, instance: usize) -> i32 {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
    bridge
        .states(instance)
        .unwrap_or_else(|| panic!("instance {instance} live"))
        .into_iter()
        .find(|(n, _)| n == "count")
        .map(|(_, v)| match v {
            AceStateValue::Int(i) => i,
            other => panic!("count is not int: {other:?}"),
        })
        .unwrap_or_else(|| panic!("no count field on instance {instance}"))
}

fn displayed_count(session: i32, instance: usize) -> String {
    let (_, states) = host_session_display(session).expect("session display");
    states
        .get(instance)
        .and_then(|row| {
            row.iter()
                .find(|(n, _)| n == "count")
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_else(|| panic!("instance {instance} count displayed"))
}

/// V-UI-A-MIN 核心（CPU 面）：002 派生 Node Counter——单例轨迹 0→1→0→0、
/// 两实例定向 2:1；计数全部经 AV 场景重放，UI 更新全部经 ui980 FFI 实际
/// 提交（applies 计数与显示模型同源核对）。
#[test]
fn p980_node_desktop_counter_trajectory_and_two_instance_ratio() {
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 2).expect("counter loads");
    let session = bridge.open().expect("open");
    assert!(session >= 1);

    // 初值：两实例 count=0；初始提交已发生（文本叶×2 + 状态×2）。
    assert_eq!(count_of(&bridge, 0), 0);
    assert_eq!(count_of(&bridge, 1), 0);
    let applies_before = host_session_stats(session).unwrap().applies;
    assert!(applies_before >= 4, "initial FFI applies: {applies_before}");

    // 单例轨迹 0→1→0→0：Inc → Dec → Reset。
    push_event(0, TARGET_INC);
    assert!(matches!(
        bridge.step(),
        StepOutcome::Dispatched { instance: 0, msg, .. } if msg == "Inc"
    ));
    assert_eq!(count_of(&bridge, 0), 1);
    push_event(0, TARGET_DEC);
    bridge.step();
    assert_eq!(count_of(&bridge, 0), 0);
    push_event(0, TARGET_RESET);
    bridge.step();
    assert_eq!(count_of(&bridge, 0), 0);

    // 两实例定向 2:1：i0 Inc×2、i1 Inc×1（业务在 AV，实例不串）。
    push_event(0, TARGET_INC);
    bridge.step();
    push_event(0, TARGET_INC);
    bridge.step();
    push_event(1, TARGET_INC);
    assert!(matches!(
        bridge.step(),
        StepOutcome::Dispatched { instance: 1, msg, .. } if msg == "Inc"
    ));
    assert_eq!(count_of(&bridge, 0), 2, "instance 0 targeted 2");
    assert_eq!(count_of(&bridge, 1), 1, "instance 1 targeted 1");

    // 宿主显示模型与 AV 真值一致（FFI 提交的实际效果）。
    assert_eq!(displayed_count(session, 0), "2");
    assert_eq!(displayed_count(session, 1), "1");

    // applies 只增不减：每次变化字段的实际 UI 提交都有账。
    let applies_after = host_session_stats(session).unwrap().applies;
    assert!(
        applies_after > applies_before,
        "event-driven applies: {applies_before} -> {applies_after}"
    );
    assert!(bridge.errors.is_empty(), "errors: {:?}", bridge.errors);
}

/// 三类源中的两类 Hello：字面量与函数中文——AV 真值文本经 FFI 进入 UI
/// session（字节级一致），无事件目标时步进 Idle。
#[test]
fn p980_node_desktop_hello_sources_av_texts_through_ffi() {
    for (name, expect) in [("app.at", "Hello"), ("app-body.at", "你好 Auto")] {
        let (src, path) = hello_fixture(name);
        let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
        let session = bridge.open().expect("open");
        let (texts, _) = host_session_display(session).unwrap();
        assert_eq!(
            texts,
            vec![vec![expect.to_string()]],
            "{name} AV text through FFI"
        );
        assert!(bridge.buttons().is_empty(), "{name} has no event targets");
        assert!(matches!(bridge.step(), StepOutcome::Idle));
        assert!(
            bridge.errors.is_empty(),
            "{name} errors: {:?}",
            bridge.errors
        );
    }
}

/// 坏 Schema/坏 ABI 在 UI 效果前具名拒：无按钮绑定的 target、未声明 msg 的
/// 按钮（ACE 面具名拒、引擎回滚不毒化）、未知 handle/越槽 apply——全部
/// 零 UI 效果，且失败后同入口恢复。
#[test]
fn p980_node_desktop_bad_schema_and_abi_refuse_before_ui_effects() {
    // —— 坏 Schema 近邻 ①：无按钮绑定的 target（队列注入越界设备事件）。
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
    let session = bridge.open().expect("open");
    push_event(0, 99);
    match bridge.step() {
        // B-AV U01：目标空间扩至 button/input 后文案随之（判别不弱化）。
        StepOutcome::LaneError(e) => assert!(e.contains("no button/input binding"), "{e}"),
        other => panic!("unbound target must be a named lane error, got {other:?}"),
    }
    // 恢复：合法事件继续走通（失败后恢复，同入口）。
    push_event(0, TARGET_INC);
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_eq!(count_of(&bridge, 0), 1);

    // —— 坏 Schema 近邻 ②：未声明 msg 的按钮（inline 源）。
    let bad = "node Bad {\n    msg { Inc }\n    model { var count int = 0 }\n    view { col { text \"Bad\" button \"x\" { onclick: .Nope } } }\n    on { .Inc -> { .count += 1 } }\n}\n";
    let mut bad_bridge = NodeDesktopBridge::load(bad, "bad-inline.at", 1)
        .expect("compiles (binding is a runtime dispatch concern)");
    let bad_session = bad_bridge.open().expect("open");
    let applies0 = host_session_stats(bad_session).unwrap().applies;
    push_event(0, 0);
    match bad_bridge.step() {
        StepOutcome::DispatchRefused(e) => {
            assert!(e.contains("component-event"), "named msg refusal: {e}")
        }
        other => panic!("undeclared msg must refuse by name, got {other:?}"),
    }
    assert_eq!(
        host_session_stats(bad_session).unwrap().applies,
        applies0,
        "msg refusal has no UI effect"
    );

    // —— 坏 ABI：未知 handle（显式句柄车道）与越槽/越实例 apply 具名拒，
    // 且不产生 UI 提交。
    let before = host_session_stats(session).unwrap().applies;
    assert_eq!(raw_poll(99), ui980_status::UNKNOWN_HANDLE);
    assert_eq!(raw_close(99), ui980_status::UNKNOWN_HANDLE);
    assert_eq!(raw_apply_text(0, 99, "x"), ui980_status::BAD_SLOT);
    assert_eq!(raw_apply_int(5, 0, 1), ui980_status::BAD_SLOT);
    assert_eq!(raw_apply_int(0, 99, 1), ui980_status::BAD_SLOT);
    assert_eq!(
        host_session_stats(session).unwrap().applies,
        before,
        "bad ABI: zero UI effects"
    );
}

/// 关闭/重开/late 事件世代纪律：close 后 apply/poll 具名拒；late 事件丢弃
/// 计数（不复活）；重开=新世代新 session，业务状态延续、显示恢复提交。
#[test]
fn p980_node_desktop_close_reopen_late_events() {
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
    let s1 = bridge.open().expect("open");
    push_event(0, TARGET_INC);
    bridge.step();
    assert_eq!(count_of(&bridge, 0), 1);

    bridge.close_session().expect("close via AV");
    let stats_at_close = host_session_stats(s1).unwrap();
    assert!(stats_at_close.closed);
    // close 后：apply/poll 具名拒（-2），晚到事件丢弃计数。
    assert_eq!(raw_apply_int(0, 0, 9), ui980_status::SESSION_CLOSED);
    assert_eq!(raw_apply_text(0, 0, "x"), ui980_status::SESSION_CLOSED);
    assert_eq!(raw_poll(s1), ui980_status::SESSION_CLOSED);
    push_event(0, TARGET_INC); // 无活动 session → late（不复活）
    assert_eq!(
        host_session_stats(s1).unwrap().late_dropped,
        stats_at_close.late_dropped + 1,
        "post-close click is late-dropped"
    );

    // 重开：新 session/新世代；AV 业务状态延续（engine 未丢）；显示重提交。
    let s2 = bridge.reopen().expect("reopen");
    assert_ne!(s1, s2, "reopen is a new session handle");
    assert!(
        host_session_stats(s2).unwrap().generation > host_session_stats(s1).unwrap().generation
    );
    assert_eq!(count_of(&bridge, 0), 1, "business state survives reopen");
    assert!(
        host_session_stats(s2).unwrap().applies >= 2,
        "reopen resubmits the display model over FFI"
    );
    assert_eq!(displayed_count(s2, 0), "1", "reopened display matches AV");
    // late 到旧代：定向旧 session 的事件只增 late_dropped，不进新队列。
    push_late_event(s1);
    assert_eq!(
        host_session_stats(s1).unwrap().late_dropped,
        stats_at_close.late_dropped + 2
    );
    // 新代继续可用；旧代 applies 冻结。
    push_event(0, TARGET_INC);
    assert!(matches!(bridge.step(), StepOutcome::Dispatched { .. }));
    assert_eq!(count_of(&bridge, 0), 2);
    assert_eq!(
        host_session_stats(s1).unwrap().applies,
        stats_at_close.applies,
        "closed session stays frozen"
    );
}

/// 反例（合同红线）：移除实际 UI 提交而只留 `text_byte_len` 不是 UI
/// dispatch——byte_len 调用成功返回字节数，但 UI session 的 applies 不动、
/// 显示模型不变。V-UI-A-MIN 的 oracle（applies>0 ∧ 显示=AV 值）对这种伪
/// 车道必然红；本 case 把该判别力固化为永久断言。
#[test]
fn p980_node_desktop_byte_len_lane_is_not_ui_dispatch() {
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
    let session = bridge.open().expect("open");
    push_event(0, TARGET_INC);
    bridge.step();
    assert_eq!(count_of(&bridge, 0), 1);
    let stats_before = host_session_stats(session).unwrap();
    let display_before = host_session_display(session).unwrap();

    // 伪车道：以 F0 text_byte_len 携带同样的值字节（走另一 provider）。
    auto_lang_ace_host::install().expect("host");
    let root = stable_f0_root();
    if let Err(e) = auto_lang_ace_host::build_f0_text_provider("p980_nd_bytelen", &root) {
        assert!(e.contains("already registered"), "{e}");
    }
    let av = "use.rs f0_text\nfn main() int { return text_byte_len(\"1\") }";
    let mut request = auto_lang_core::AceRequest::simple(av);
    request.provider = Some("p980_nd_bytelen");
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::Success {
            value: auto_lang_core::AceValue::I32(n),
            ..
        } => assert_eq!(*n, 1, "byte_len lane itself succeeds"),
        other => panic!("byte_len control dispatch failed: {other:?}"),
    }

    // 判别力：伪车道零 UI 效果（真车道在 case 1 里 applies>0 ∧ 显示=AV 值）。
    assert_eq!(
        host_session_stats(session).unwrap().applies,
        stats_before.applies,
        "byte_len-only lane must not count as UI dispatch"
    );
    assert_eq!(
        host_session_display(session).unwrap(),
        display_before,
        "byte_len-only lane must not move the display model"
    );
}

/// 坏权限：provider 撤权后请求在分派前拒（零 UI 效果）；复权后同一公开门
/// 恢复（exact-fix 先例）。
/// 复核反例（B-NATIVE-R authored）：work 用例未覆盖的三个纪律面——
/// ① 车道标签：错 op tag / 负打包值具名拒（BAD_OP/BAD_SLOT），零 UI 效果；
/// ② 跨 session 队列隔离：显式句柄 poll 只见本 session 队列（不偷当前
///    session 的事件）；③ 双重 close 同句柄 → stale 具名拒（不复活）。
#[test]
fn p980_node_desktop_lane_tag_queue_isolation_and_double_close() {
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
    let s1 = bridge.open().expect("open s1");
    let before = host_session_stats(s1).unwrap();

    // ① 错标签（大小写差一位）→ BAD_OP；负打包值 → BAD_SLOT；均零 UI 效果。
    assert_eq!(
        raw_ui980_call("ui_apply_int(\"apply-BOOL\", 0, 5)"),
        ui980_status::BAD_OP
    );
    assert_eq!(
        raw_ui980_call("ui_apply_int(\"apply-int\", -1, 5)"),
        ui980_status::BAD_SLOT
    );
    assert_eq!(
        host_session_stats(s1).unwrap().applies,
        before.applies,
        "lane-discipline refusals leave zero UI effects"
    );

    // ② 第二个 live session：current 切到 s2，s1 存活但非当前。
    let s2 = bridge.open().expect("open s2");
    assert_ne!(s1, s2);
    push_event(0, TARGET_INC); // 落入当前 session（s2）的 typed 队列
    assert_eq!(
        raw_poll(s1),
        0,
        "explicit-handle poll must not dequeue another session's events"
    );
    assert_eq!(
        host_session_stats(s2).unwrap().polled,
        0,
        "s2 not polled by s1's poll"
    );
    // s2 自己的 poll 取到事件编码：(0+1)*1000+(TARGET_INC+1)=1003。
    assert_eq!(raw_poll(s2), 1003);
    assert_eq!(host_session_stats(s2).unwrap().polled, 1);

    // ③ 显式 close 存活的非当前 session → 0；同句柄再 close → -2（不复活）。
    assert_eq!(raw_close(s1), ui980_status::OK);
    assert_eq!(raw_close(s1), ui980_status::SESSION_CLOSED);
}

#[test]
fn p980_node_desktop_permission_revocation_refuses_before_dispatch() {
    ensure_ui980_provider().expect("provider ready");
    let (src, path) = counter_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 1).expect("loads");
    let session = bridge.open().expect("open");
    let applies_before = host_session_stats(session).unwrap().applies;

    assert!(auto_lang_ace_host::revoke_provider(UI980_PROVIDER_NAME));
    let av =
        format!("use.rs ui980_native\nfn main() int {{ return ui_apply_int({session}, 0, 0, 7) }}");
    let mut request = auto_lang_core::AceRequest::simple(&av);
    request.provider = Some(UI980_PROVIDER_NAME);
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::Denied { code, .. } => {
            assert!(code.contains("revoked"), "named revocation refusal: {code}")
        }
        other => panic!("revoked provider must refuse before dispatch, got {other:?}"),
    }
    assert_eq!(
        host_session_stats(session).unwrap().applies,
        applies_before,
        "refused request has zero UI effect"
    );

    assert!(auto_lang_ace_host::readmit_provider(UI980_PROVIDER_NAME));
    push_event(0, TARGET_INC);
    assert!(
        matches!(bridge.step(), StepOutcome::Dispatched { .. }),
        "readmitted lane serves again"
    );
    assert_eq!(count_of(&bridge, 0), 1);
}

/// 窗口+FFI 面（排窗）：真窗渲染（帧数）+ 脚本化 typed 事件（轨迹 0→1→0→0、
/// 2:1、close/reopen/late）+ 证据 JSON。准备阶段见文件头注记。
#[test]
fn p980_node_desktop_window_e2e_counter_full_script() {
    if std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_node_desktop_window_e2e: not scheduled (set AUTO_UI_ACE_NODE_E2E=1 after \
             the prep stage; window single-instance discipline)"
        );
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ace_node_desktop_window.exe")
        .canonicalize()
        .unwrap();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — run the prep stage first",
        exe.display()
    );
    let (_, path) = counter_fixture();
    let tmp = std::env::temp_dir().join(format!("p980-nd-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let ev_path = tmp.join("evidence.json");

    // 轨迹：i0 Inc(t3)→Dec(t5)→Reset(t7)=0→1→0→0；i0 Inc(t9,t11)+i1 Inc(t13)
    // =2:1；close(t15) → late(t17) → reopen(t19)。
    let script = "3:click:0:2,5:click:0:0,7:click:0:1,9:click:0:2,11:click:0:2,13:click:1:2,15:close,17:late,19:reopen";
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&path)
        .arg("--evidence")
        .arg(&ev_path)
        .arg("--instances")
        .arg("2")
        .arg("--script")
        .arg(script)
        .arg("--hold-ms")
        .arg("12000")
        .current_dir(&tmp)
        .output()
        .expect("spawn window exe");
    assert!(
        out.status.success(),
        "window exe failed: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let ev: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap())
        .unwrap_or_else(|e| panic!("evidence json: {e}"));

    assert_eq!(ev["node"].as_str().unwrap(), "Counter");
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 4,
        "real window rendered"
    );
    assert!(
        ev["errors"].as_array().unwrap().is_empty(),
        "errors: {}",
        ev["errors"]
    );

    // dispatch 轨迹（AV 重放面）与最终计数。
    let log = ev["dispatch_log"].as_array().unwrap();
    let msgs: Vec<&str> = log.iter().map(|d| d["msg"].as_str().unwrap()).collect();
    assert_eq!(
        msgs,
        vec!["Inc", "Dec", "Reset", "Inc", "Inc", "Inc"],
        "0→1→0→0 then 2:1 dispatch order"
    );
    let count_at = |entry: &serde_json::Value| -> String {
        entry["states"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|kv| {
                let pair = kv.as_array()?;
                (pair[0].as_str() == Some("count")).then(|| pair[1].as_str().unwrap().to_string())
            })
            .unwrap()
    };
    let last_i0 = log.iter().filter(|d| d["instance"] == 0).last().unwrap();
    let last_i1 = log.iter().filter(|d| d["instance"] == 1).last().unwrap();
    assert_eq!(count_at(last_i0), "2", "instance 0 final count");
    assert_eq!(count_at(last_i1), "1", "instance 1 final count");

    // 会话面：两代 session；旧代 closed 且 late_dropped≥1；新代重提交。
    let sessions = ev["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2, "close + reopen = two sessions");
    assert_eq!(sessions[0]["closed"], true, "first session closed");
    assert!(
        sessions[0]["late_dropped"].as_u64().unwrap() >= 1,
        "late events dropped on the closed generation"
    );
    assert!(!sessions[1]["closed"].as_bool().unwrap());
    assert!(
        sessions[1]["applies"].as_u64().unwrap() >= 2,
        "reopen resubmits"
    );
    // 新代显示模型与 AV 一致（[i0=2, i1=1]）。
    let s2_states = sessions[1]["states"].as_array().unwrap();
    assert_eq!(s2_states.len(), 2, "two instances displayed");
    let display_count = |row: &serde_json::Value| -> String {
        row.as_array()
            .unwrap()
            .iter()
            .find_map(|kv| {
                let pair = kv.as_array()?;
                (pair[0].as_str() == Some("count")).then(|| pair[1].as_str().unwrap().to_string())
            })
            .unwrap()
    };
    assert_eq!(display_count(&s2_states[0]), "2");
    assert_eq!(display_count(&s2_states[1]), "1");
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// AV 车道原始调用（坏 ABI/权限反例；与桥同门的受检调用，不走桥差量）
// ---------------------------------------------------------------------------

fn raw_apply_int(instance: i32, slot: i32, value: i32) -> i32 {
    let packed = instance * 1000 + slot;
    raw_ui980_call(&format!("ui_apply_int(\"apply-int\", {packed}, {value})"))
}

fn raw_close(session: i32) -> i32 {
    raw_ui980_call(&format!("ui_close(\"close\", {session}, 0)"))
}

fn raw_apply_text(instance: i32, leaf: i32, text: &str) -> i32 {
    let escaped = auto_ui_runtime::ui::ace_hello::escape_auto_string_for_source(text);
    let packed = instance * 1000 + leaf;
    raw_ui980_call(&format!("ui_apply_text({escaped}, {packed}, 0)"))
}

fn raw_poll(session: i32) -> i32 {
    raw_ui980_call(&format!("ui_poll_event(\"poll\", {session}, 0)"))
}

fn raw_ui980_call(body: &str) -> i32 {
    let src = format!("use.rs ui980_native\nfn main() int {{ return {body} }}");
    let mut request = auto_lang_core::AceRequest::simple(&src);
    request.provider = Some(UI980_PROVIDER_NAME);
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::Success {
            value: auto_lang_core::AceValue::I32(n),
            ..
        } => *n,
        other => panic!("raw ui980 call `{body}` failed: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// PLAN-980 B-AV U01（首个实施切片）：文本输入组件的 AV 窗口面。
// ---------------------------------------------------------------------------

/// U01 输入夹具（converter-mini：str+int 双输入、无④依赖——主题/导入/
/// f64 均不入本面；003/011 全量承载压在④域合同，见 §9 分母冻结）。
fn input_fixture() -> (String, String) {
    let src = "widget NoteForm {
    msg Msg { SetNote(str), SetCount(int) }
    model { var note str = \"\"
        var count int = 7 }
    view { col {
        input (value: .note) { oninput: .SetNote }
        input (value: .count) { oninput: .SetCount }
        text .note { }
    } }
    on {
        .SetNote(s) -> { .note = s }
        .SetCount(v) -> { .count = v }
    }
}
"
    .to_string();
    (src, "p980-u01-input.at".to_string())
}

/// 输入目标编号（fixture 冻结：无按钮，inputs 从 0 起）。
const INPUT_NOTE: usize = 0;
const INPUT_COUNT: usize = 1;

fn str_state(bridge: &NodeDesktopBridge, instance: usize, field: &str) -> String {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
    bridge
        .states(instance)
        .unwrap_or_else(|| panic!("instance {instance} live"))
        .into_iter()
        .find(|(n, _)| n == field)
        .map(|(_, v)| match v {
            AceStateValue::Str(s) => s,
            other => panic!("{field} is not str: {other:?}"),
        })
        .unwrap_or_else(|| panic!("field {field} present"))
}

fn int_state(bridge: &NodeDesktopBridge, instance: usize, field: &str) -> i32 {
    use auto_ui_runtime::ui::ace_state_bridge::AceStateValue;
    bridge
        .states(instance)
        .unwrap_or_else(|| panic!("instance {instance} live"))
        .into_iter()
        .find(|(n, _)| n == field)
        .map(|(_, v)| match v {
            AceStateValue::Int(i) => i,
            other => panic!("{field} is not int: {other:?}"),
        })
        .unwrap_or_else(|| panic!("field {field} present"))
}

/// U01 oracle 全件：原值/中文回显（str 载荷经 typed 队列→AV 重放→apply
/// str）、数值输入（int 绑定桥面解析）、**非法输入**（非数值→具名拒零
/// 效果——count 不动、无 apply）、两实例隔离。
#[test]
fn p980_u01_input_text_numeric_reject_and_two_instance_isolation() {
    let (src, path) = input_fixture();
    let mut bridge = NodeDesktopBridge::load(&src, &path, 2).expect("input fixture loads");
    assert_eq!(bridge.buttons().len(), 0, "no buttons in fixture");
    assert_eq!(bridge.inputs().len(), 2, "two input targets");
    assert!(bridge.inputs()[INPUT_NOTE].field == "note" && !bridge.inputs()[INPUT_NOTE].numeric);
    assert!(bridge.inputs()[INPUT_COUNT].field == "count" && bridge.inputs()[INPUT_COUNT].numeric);
    bridge.open().expect("open");
    bridge.install_script(vec![
        (
            1,
            ScriptStep::TypeText {
                instance: 0,
                target: INPUT_NOTE,
                text: "你好 Auto".to_string(),
            },
        ),
        (
            2,
            ScriptStep::TypeText {
                instance: 0,
                target: INPUT_COUNT,
                text: "42".to_string(),
            },
        ),
        // 非法输入：非数值进 int 绑定——具名拒零效果。
        (
            3,
            ScriptStep::TypeText {
                instance: 0,
                target: INPUT_COUNT,
                text: "abc".to_string(),
            },
        ),
        // 两实例隔离：实例 1 各自键入。
        (
            4,
            ScriptStep::TypeText {
                instance: 1,
                target: INPUT_NOTE,
                text: "乙侧".to_string(),
            },
        ),
        (
            5,
            ScriptStep::TypeText {
                instance: 1,
                target: INPUT_COUNT,
                text: "-5".to_string(),
            },
        ),
    ]);

    let mut refused = 0usize;
    let mut dispatched: Vec<(usize, String)> = Vec::new();
    for _ in 0..12 {
        match bridge.step() {
            StepOutcome::Dispatched { instance, msg, .. } => {
                assert!(
                    msg == "SetNote" || msg == "SetCount",
                    "declared msg only: {msg}"
                );
                dispatched.push((instance, msg));
            }
            StepOutcome::DispatchRefused(e) => {
                assert!(e.contains("invalid numeric input"), "named rejection: {e}");
                refused += 1;
            }
            _ => {}
        }
    }
    assert_eq!(refused, 1, "exactly one invalid-input rejection");

    // 原值/中文（str 经 typed 队列与 AV 场景重放）。
    assert_eq!(str_state(&bridge, 0, "note"), "你好 Auto");
    // 数值输入（int 绑定桥面解析→SendArgs int）。
    assert_eq!(
        int_state(&bridge, 0, "count"),
        42,
        "abc was rejected, 42 stands"
    );
    // 两实例隔离。
    assert_eq!(str_state(&bridge, 1, "note"), "乙侧");
    assert_eq!(int_state(&bridge, 1, "count"), -5);

    // dispatch 轨迹：4 次 typed 交付（非法那次不产生 dispatch）——每实例
    // 各一条 SetNote + SetCount。
    assert_eq!(bridge.dispatch_log.len(), 4, "four typed dispatches");
    assert_eq!(dispatched.len(), 4);
    assert!(dispatched.contains(&(0, "SetNote".to_string())));
    assert!(dispatched.contains(&(0, "SetCount".to_string())));
    assert!(dispatched.contains(&(1, "SetNote".to_string())));
    assert!(dispatched.contains(&(1, "SetCount".to_string())));
    // apply 证据面：非法输入零 apply——SetCount 仅成功值各一次（i0=42、
    // i1=-5；初值 7 在 open 已提交，不在此后重放）。
    let evidence = bridge.evidence();
    // 串行套件下 sessions 累积——取本 bridge 的 session（末位）。
    let sessions = evidence["sessions"].as_array().unwrap();
    let this = sessions.last().expect("own session recorded");
    let applies_after: Vec<(usize, &str)> = this["applied"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["instance"].as_i64().unwrap() as usize,
                a["kind"].as_str().unwrap(),
            )
        })
        .collect();
    let str_applies = applies_after.iter().filter(|(_, k)| *k == "str").count();
    // 每实例 2 次键入 str（note）+ 2 次初值（note=""）——非法输入不增。
    assert_eq!(
        str_applies, 4,
        "two notes + two initial strs applied: {applies_after:?}"
    );
    let int_applies = applies_after.iter().filter(|(_, k)| *k == "int").count();
    // open 初值 2 例 + 成功数值 2 例 = 4；非法输入不增。
    assert_eq!(
        int_applies, 4,
        "initials + two valid numbers only: {applies_after:?}"
    );
}

/// 负控：静态面源（无 model）带输入目标 → 装载期具名拒（事件面缺席）。
#[test]
fn p980_u01_input_on_static_face_refuses() {
    let src = "widget HelloOnly {\n    view { col { input (value: .x) { oninput: .Go } } }\n}\n";
    let err = match NodeDesktopBridge::load(src, "p980-u01-static.at", 1) {
        Ok(_) => panic!("static face cannot host input targets"),
        Err(e) => e,
    };
    assert!(
        err.contains("event targets need a model/msg surface"),
        "named refusal: {err}"
    );
}

/// U01 真窗输入档（排窗，`AUTO_UI_ACE_NODE_E2E=1`）：真实窗口 + 脚本键入
/// （`type` 动作）→ typed 队列 → AV 重放 → ui980 FFI apply → 受控回显；
/// 含非法数值具名拒零效果与两实例隔离。**边界**：键入经桥注入（与真实
/// 键盘同一注入边界），OS 级键盘事件不可自动化；窗口面验证=真窗渲染
/// input 目标 + FFI 提交后的状态行（回显源）。
#[test]
fn p980_u01_window_input_e2e() {
    if std::env::var("AUTO_UI_ACE_NODE_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_u01_window_input_e2e: not scheduled (set AUTO_UI_ACE_NODE_E2E=1 after \
             the prep stage; window single-instance discipline)"
        );
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ace_node_desktop_window.exe")
        .canonicalize()
        .unwrap();
    assert!(
        exe.is_file(),
        "preparation failure: {} missing — run the prep stage first",
        exe.display()
    );
    let tmp = std::env::temp_dir().join(format!("p980-u01-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (src, _) = input_fixture();
    let src_path = tmp.join("note_form.at");
    std::fs::write(&src_path, &src).expect("write input fixture");
    let ev_path = tmp.join("evidence.json");

    // i0：note=你好 Auto、count=42、count=abc（具名拒）；i1：note=乙侧、count=-5。
    let script = "3:type:0:0:你好 Auto,5:type:0:1:42,7:type:0:1:abc,9:type:1:0:乙侧,11:type:1:1:-5";
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&src_path)
        .arg("--evidence")
        .arg(&ev_path)
        .arg("--instances")
        .arg("2")
        .arg("--script")
        .arg(script)
        .arg("--hold-ms")
        .arg("6000")
        .current_dir(&tmp)
        .output()
        .expect("spawn window exe");
    assert!(
        out.status.success(),
        "window exe failed: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let ev: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap())
        .unwrap_or_else(|e| panic!("evidence json: {e}"));

    assert_eq!(ev["node"].as_str().unwrap(), "NoteForm");
    assert!(
        ev["frames_rendered"].as_u64().unwrap_or(0) >= 4,
        "real window rendered"
    );

    // 非法数值=具名拒（证据记名，不静默；其余无错误）。
    let errors: Vec<&str> = ev["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap_or(""))
        .collect();
    assert!(
        errors.iter().any(|e| e.contains("invalid numeric input")),
        "named invalid-input refusal: {errors:?}"
    );

    // dispatch 轨迹：4 次 typed 交付（非法那次不产生 dispatch），实例隔离。
    let log = ev["dispatch_log"].as_array().unwrap();
    let msgs: Vec<(u64, &str)> = log
        .iter()
        .map(|d| (d["instance"].as_u64().unwrap(), d["msg"].as_str().unwrap()))
        .collect();
    assert_eq!(
        msgs,
        vec![
            (0, "SetNote"),
            (0, "SetCount"),
            (1, "SetNote"),
            (1, "SetCount")
        ],
        "typed dispatch order/instance isolation"
    );

    let state_of = |entry: &serde_json::Value, field: &str| -> String {
        entry["states"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|kv| {
                let pair = kv.as_array()?;
                (pair[0].as_str() == Some(field)).then(|| pair[1].as_str().unwrap().to_string())
            })
            .unwrap_or_else(|| panic!("state {field} present: {entry}"))
    };
    let last_i0 = log.iter().filter(|d| d["instance"] == 0).last().unwrap();
    let last_i1 = log.iter().filter(|d| d["instance"] == 1).last().unwrap();
    assert_eq!(state_of(last_i0, "note"), "你好 Auto", "str input echo");
    assert_eq!(state_of(last_i0, "count"), "42", "abc rejected, 42 stands");
    assert_eq!(state_of(last_i1, "note"), "乙侧");
    assert_eq!(state_of(last_i1, "count"), "-5");
}
