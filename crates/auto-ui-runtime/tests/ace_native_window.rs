//! PLAN-980 NATIVE 永久 case。
//!
//! 分层：CPU 面（恒跑）——001 真源装载（legacy widget 头 Accept）、文本叶
//! 值/属性（selectable/style 来自源 view 树）、状态写（dark_mode 主题翻转经
//! style_iced_adapter 线程局部，可 CPU 侧观察）；窗口面（
//! `AUTO_UI_ACE_HELLO_E2E=1` 排窗，单实例串行）——真窗渲染 + 帧内状态写 +
//! 证据 JSON + **关闭重开**（顺序两轮进程级窗口）。
//!
//! 交互档边界（具名）：文字选择/复制的交互实测在仓内既有档（selectable_text
//! 单测族 + 手工/环境门控档），本文件不复制；IME/媒体/终端交互=手工档。

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_native::AceNativeSession;

fn fixture_001() -> (String, String) {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/001-helloworld/src/front/app.at");
    (
        std::fs::read_to_string(&at).expect("001 real source readable"),
        at.to_string_lossy().to_string(),
    )
}

/// 001 真源（legacy `widget` 头 + design + col + 带属性文本叶）经 ACE 面
/// 装载：AV 文本真值 + 源属性（selectable/style）+ 状态（str/bool）齐。
#[test]
fn p980_native_001_loads_with_props_and_states() {
    let (src, path) = fixture_001();
    let s = AceNativeSession::load(&src, &path).expect("001 loads on the ACE face");
    assert!(
        s.snapshot.legacy_widget,
        "legacy widget head admitted via alias"
    );
    assert_eq!(s.snapshot.texts.len(), 1);
    let leaf = &s.snapshot.texts[0];
    assert_eq!(leaf.value, "Hello, World!");
    assert!(
        leaf.selectable,
        "selectable prop rides the source view tree"
    );
    assert!(
        leaf.style.as_deref().unwrap_or("").contains("text-4xl"),
        "style prop preserved: {:?}",
        leaf.style
    );
    let get = |n: &str| {
        s.snapshot
            .states
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, v)| format!("{v:?}"))
            .unwrap_or_default()
    };
    assert_eq!(get("theme_name"), "Str(\"Auto\")");
    assert_eq!(get("dark_mode"), "Bool(false)");
}

/// 状态写 → 重编译重观察：dark_mode=true 翻转窗口主题线程局部（CPU 侧
/// 直接可观察）；theme_name 写入同链路。
#[test]
fn p980_native_state_write_flips_theme_and_reobserves() {
    let (src, path) = fixture_001();
    let mut s = AceNativeSession::load(&src, &path).expect("load");
    auto_ui_runtime::ui::style_iced_adapter::set_dark_mode(false);
    s.write_state_bool("dark_mode", true)
        .expect("write dark_mode");
    assert_eq!(
        s.snapshot
            .states
            .iter()
            .find(|(k, _)| k == "dark_mode")
            .map(|(_, v)| format!("{v:?}")),
        Some("Bool(true)".to_string())
    );
    assert!(
        auto_ui_runtime::ui::style_iced_adapter::dark_mode(),
        "window theme thread-local flipped by the state write"
    );
    // 未知字段写=具名拒；主题不被污染。
    let err = s
        .write_state_bool("nope", true)
        .expect_err("unknown field refuses");
    assert!(err.contains("no model field named `nope`"), "{err}");
    // 文本真值不随状态写漂移（重新观察仍来自同一源）。
    assert_eq!(s.snapshot.texts[0].value, "Hello, World!");
}

/// 出面外源具名拒（run 通道不静默回旧 VM 的 CPU 侧 oracle）：多节点源
/// （001+002 同文件）在组件面 shape 拒。
#[test]
fn p980_native_out_of_face_source_refuses_by_name() {
    let (src, _path) = fixture_001();
    let two = format!("{src}\nnode Extra {{ view {{ text \"x\" {{}} }} }}\n");
    let err = AceNativeSession::load(&two, "two.at").expect_err("two nodes refuse");
    assert!(err.contains("source.age.component-shape"), "{err}");
}

/// 窗口面（排窗 env 门控）：真窗渲染 + 帧内状态写（主题翻转进证据）+
/// 关闭重开（顺序两轮进程级窗口——PLAN-698 单事件环约束下的重开语义）。
#[test]
fn p980_native_window_state_write_and_close_reopen() {
    if std::env::var("AUTO_UI_ACE_HELLO_E2E").ok().as_deref() != Some("1") {
        eprintln!("p980_native_window: not scheduled (AUTO_UI_ACE_HELLO_E2E=1 after prep)");
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ace_native_window.exe")
        .canonicalize()
        .expect("window exe built (prep: cargo build -p auto-ui-runtime --features ace-bridge --example ace_native_window)");
    let (_, path) = fixture_001();
    let tmp = std::env::temp_dir().join(format!("p980-native-win-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    for round in 1..=2u32 {
        let ev = tmp.join(format!("ev{round}.json"));
        let out = std::process::Command::new(&exe)
            .arg("--source")
            .arg(&path)
            .arg("--evidence")
            .arg(&ev)
            .arg("--set-state")
            .arg("dark_mode=true")
            .arg("--set-at-frame")
            .arg("2")
            .arg("--hold-ms")
            .arg("5000")
            .current_dir(&tmp)
            .output()
            .expect("spawn window");
        assert!(
            out.status.success(),
            "round {round} window exe failed: {} / {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let e: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ev).unwrap()).unwrap();
        assert_eq!(e["texts"][0]["value"].as_str().unwrap(), "Hello, World!");
        assert_eq!(e["texts"][0]["selectable"].as_bool().unwrap(), true);
        assert_eq!(e["set_state_applied"].as_bool().unwrap(), true);
        assert_eq!(e["set_state_ok"].as_bool().unwrap(), true);
        assert_eq!(e["dark_mode_after"].as_bool().unwrap(), true);
        assert_eq!(
            e["states_after"]["dark_mode"].as_str().unwrap(),
            "Bool(true)"
        );
        assert!(e["frames_rendered"].as_u64().unwrap() >= 2, "real frames");
        assert_eq!(e["window_result"].as_str().unwrap(), "closed");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// 复核反例（B-NATIVE-R）：多文本叶的属性对齐——带属性体叶与裸叶混排时
/// style/selectable 逐叶对齐（zip 顺序错位即红）；无 selectable 的叶默认 false。
#[test]
fn p980_native_multi_leaf_props_align_by_declaration_order() {
    const TWO: &str = concat!(
        "node T {
",
        "    model {
",
        "        var name str = \"Auto\"
",
        "    }
",
        "    view {
",
        "        text \"plain\" {}
",
        "        text \"fancy\" { style: \"text-2xl\", selectable: true }
",
        "    }
",
        "}
",
    );
    let s = AceNativeSession::load(TWO, "two.at").expect("two-leaf source loads");
    assert_eq!(s.snapshot.texts.len(), 2);
    assert_eq!(s.snapshot.texts[0].value, "plain");
    assert_eq!(s.snapshot.texts[0].selectable, false);
    assert_eq!(s.snapshot.texts[0].style, None);
    assert_eq!(s.snapshot.texts[1].value, "fancy");
    assert_eq!(s.snapshot.texts[1].selectable, true);
    assert_eq!(s.snapshot.texts[1].style.as_deref(), Some("text-2xl"));
}
