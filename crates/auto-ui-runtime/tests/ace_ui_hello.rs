//! PLAN-980 HELLO：首个 ACE AGE Node 文字窗口的永久 case。
//!
//! 分层（窗口/GPU 单实例纪律）：
//! - CPU 面（恒跑，每条 ≤10s）：AGE Schema 装载/AV 文本真值/根名与文本自由
//!   变体/F0 受检调用与拒绝面/20 轮生命周期与 teardown。
//! - 窗口 + FFI 面（`AUTO_UI_ACE_HELLO_E2E=1` 显式排窗执行；单实例、串行）：
//!   预构建产物（`cargo build -p auto-ui-runtime --example ace_hello_window` +
//!   F0 provider 预构建于 target/ace-hello-f0）就绪后，子进程真窗 + 证据 JSON
//!   断言。缺预构建 = 具名准备失败（不充 PASS）。
//!
//! 预备（V 阶段实际命令）：
//!   cargo build -p auto-ui-runtime --example ace_hello_window
//!   target/debug/examples/ace_hello_window.exe --source <fixture> \
//!       --evidence <prep.json> --provider f0_hello   # 触发 F0 预构建（缓存 stamp）
//!   AUTO_UI_ACE_HELLO_E2E=1 cargo test -p auto-ui-runtime --test ace_ui_hello \
//!       -- --test-threads=1

#![cfg(all(windows, feature = "ace-bridge"))]

use auto_ui_runtime::ui::ace_hello::{stable_f0_root, HelloSession};

fn fixture(name: &str) -> (String, String) {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest
        .join("../../examples/ui/ace-hello-minimal/src/front")
        .join(name);
    let src = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{name} readable: {e}"));
    let path = at.to_string_lossy().to_string();
    (src, path)
}

/// V-HELLO-SCHEMA（CPU 面）：正式输入 root=node；AGE Schema 身份在案；改
/// Node 名/文本仍装载；无 AGE（坏 Schema）在 FFI 前具名拒绝。
#[test]
fn p980_hello_schema_identity_and_rename_tolerance() {
    let (src, path) = fixture("app.at");
    let s = HelloSession::load(&src, &path).expect("fixture loads");
    assert_eq!(s.evidence.node_name, "Hello");
    assert_eq!(s.evidence.head_role, "node");
    assert!(
        !s.evidence.grammar_id.is_empty(),
        "AGE grammar identity recorded"
    );
    assert_eq!(s.evidence.av_text, "Hello");

    // 任意合法 Node 名/文本都必须工作（不硬编码 Hello）。
    let renamed = src
        .replace("\"Hello\"", "\"Grüzi\"")
        .replace("Hello", "Gruss");
    let s2 = HelloSession::load(&renamed, &path).expect("rename loads");
    assert_eq!(s2.evidence.node_name, "Gruss");
    assert_eq!(s2.evidence.av_text, "Grüzi");

    // 坏 Schema（无 AGE 构造）在 FFI 前拒绝，具名诊断。
    let err = HelloSession::load("fn main() int { return 0 }", "plain.at")
        .expect_err("plain source has no AGE node");
    // 零 node 是 H0 面的合法接受（观察无节点）；桥的装载拒绝（无文本可渲染）
    // 与共享面诊断一样都是 FFI 前的具名拒绝。
    assert!(
        err.contains("no AGE node") || err.contains("source."),
        "named refusal before FFI: {err}"
    );
}

/// V-HELLO-BODY（CPU 面）：文本来自普通 Auto 函数返回值（中文/ASCII));
/// 改函数返回文本 → AV 值随变（同装载入口）。
#[test]
fn p980_hello_body_text_from_plain_auto_function() {
    let (src, path) = fixture("app-body.at");
    let s = HelloSession::load(&src, &path).expect("body fixture loads");
    assert_eq!(s.evidence.av_text, "你好 Auto");
    assert_eq!(s.evidence.node_name, "HelloBody");

    let edited = src.replace("你好 Auto", "Bonjour Auto");
    let s2 = HelloSession::load(&edited, &path).expect("edited body loads");
    assert_eq!(s2.evidence.av_text, "Bonjour Auto");
}

/// V-HELLO-REJECT（CPU 面）：坏文本类型/坏 ABI 身份在 FFI 前具名拒绝；
/// 错误 provider 名 → E0 具名 HostFailure，callee=0（不静默）。
#[test]
fn p980_hello_rejections_are_named_with_zero_callee() {
    // 非文本叶：int 字面量 → H0 文本类型拒绝。
    let err = HelloSession::load(
        "node B {\n    view {\n        text 5 {}\n    }\n}\n",
        "bad.at",
    )
    .expect_err("int text leaf must refuse");
    assert!(err.contains("source.age"), "named AGE diagnostic: {err}");

    // 错 provider 名：宿主安装 + 未注册名 → E0 具名拒绝（无 dispatch）。
    auto_lang_ace_host::install().expect("ace host installs");
    let mut request = auto_lang_core::AceRequest::simple(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"x\") }",
    );
    request.provider = Some("p980_no_such_provider");
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::HostFailure { code, .. } => {
            assert_eq!(code, "ace.provider/unknown");
        }
        other => panic!("expected named provider refusal, got {other:?}"),
    }
    assert_ne!(report.callee_count, Some(1), "no dispatch may happen");
}

/// V-HELLO-LIFE（CPU 面）：20 轮装载/AV 观察/F0 分派/释放循环 + provider
/// teardown 后续分派具名拒（关闭旧代不能写新会话）。内存断言以功能面 +
/// 释放可重复为准（进程级 RSS 在测试内不可稳定观测，具名边界）。
#[test]
fn p980_hello_twenty_round_lifecycle_and_teardown_refusal() {
    let (src, path) = fixture("app.at");
    auto_lang_ace_host::install().expect("host installs");
    let root = stable_f0_root();
    auto_lang_ace_host::build_f0_text_provider("p980_hello_life", &root)
        .expect("prepared provider (cached)");
    for round in 0..20 {
        let mut s = HelloSession::load(&src, &path).expect("load");
        s.run_f0("p980_hello_life").expect("f0 dispatch");
        assert_eq!(s.evidence.av_text, "Hello", "round {round}");
        assert_eq!(s.evidence.f0.as_ref().unwrap().byte_len, 5);
        drop(s); // 释放：AV 会话与文本值随 drop 归还
    }
    // teardown：旧代分派入口具名失效（不再有新 dispatch）。桥的 run_f0 会按
    // 幂等语义重新注册（=新代恢复），故旧名失效用直连门面验证。
    assert!(auto_lang_ace_host::teardown_provider("p980_hello_life"));
    let mut request = auto_lang_core::AceRequest::simple(
        "use.rs f0_text
fn main() int { return text_byte_len(\"x\") }",
    );
    request.provider = Some("p980_hello_life");
    let report = auto_lang_core::run_ace(auto_lang_core::AceEngineChoice::AceAV, &request);
    match &report.outcome {
        auto_lang_core::AceOutcome::HostFailure { code, .. } => {
            assert_eq!(code, "ace.provider/unknown");
        }
        other => panic!("torn-down provider must refuse by name, got {other:?}"),
    }
    // 重注册（新代）恢复正常。
    auto_lang_ace_host::build_f0_text_provider("p980_hello_life", &root).expect("re-register");
    let mut s = HelloSession::load(&src, &path).expect("reload");
    s.run_f0("p980_hello_life")
        .expect("new generation dispatches");
}

/// V-HELLO-TEXT/BODY + V-HELLO-PACK 的窗口+FFI 面：预构建就绪时子进程真窗。
#[test]
fn p980_hello_window_e2e_text_body_and_f0() {
    if std::env::var("AUTO_UI_ACE_HELLO_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_hello_window_e2e: not scheduled (set AUTO_UI_ACE_HELLO_E2E=1 after the \
             prep stage; window single-instance discipline)"
        );
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest
        .join("../../target/debug/examples/ace_hello_window.exe")
        .canonicalize()
        .unwrap();
    if !exe.is_file() {
        panic!(
            "preparation failure: {} missing — run \
             `cargo build -p auto-ui-runtime --example ace_hello_window` first",
            exe.display()
        );
    }
    let root = stable_f0_root();
    let stamp = root.join("out/f0-build-stamp.txt");
    assert!(
        stamp.is_file(),
        "preparation failure: F0 provider not prebuilt at {} — run the prep stage first",
        root.display()
    );

    let tmp = std::env::temp_dir().join(format!("p980-hello-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    for (name, expect_text, expect_bytes) in [
        ("app.at", "Hello", 5usize),
        ("app-body.at", "你好 Auto", "你好 Auto".len()),
    ] {
        let (src, path) = fixture(name);
        let _ = &src;
        let ev_path = tmp.join(format!("{name}.json"));
        // 任意 cwd（仓外异根）：从临时目录起子进程，路径全绝对。
        let out = std::process::Command::new(&exe)
            .arg("--source")
            .arg(&path)
            .arg("--evidence")
            .arg(&ev_path)
            .arg("--provider")
            .arg("f0_hello")
            .arg("--hold-ms")
            .arg("4000")
            .current_dir(&tmp)
            .output()
            .expect("spawn window exe");
        assert!(
            out.status.success(),
            "{name}: window exe failed: {} / {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let ev: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ev_path).unwrap())
                .unwrap_or_else(|e| panic!("{name} evidence json: {e}"));
        assert_eq!(
            ev["av_text"].as_str().unwrap(),
            expect_text,
            "{name} AV text"
        );
        assert_eq!(ev["av_text_bytes"].as_u64().unwrap(), expect_bytes as u64);
        assert_eq!(
            ev["frames_rendered"].as_u64().unwrap_or(0) >= 2,
            true,
            "{name} rendered"
        );
        let f0 = &ev["f0"];
        assert_eq!(
            f0["byte_len"].as_i64().unwrap(),
            expect_bytes as i64,
            "{name} f0 bytes"
        );
        assert_eq!(
            f0["callee_count"].as_i64().unwrap(),
            1,
            "{name} one real dispatch"
        );
        assert_eq!(f0["rejected_count"].as_i64().unwrap(), 0);
        assert_eq!(ev["f0_root"].as_str().unwrap(), root.display().to_string());
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// 复核反例（T-HELLO-R）：F0 分派的字节等式是逐次核验的，不是装样子——
/// AV 文本与分派载荷错配（漂移注入）必须在当次分派具名红。
#[test]
fn p980_hello_f0_bytes_drift_must_refuse() {
    let (src, path) = fixture("app.at");
    let mut s = HelloSession::load(&src, &path).expect("load");
    // 漂移注入：证据文本被改（模拟上游不同步），分派载荷仍按证据走。
    s.evidence.av_text = "HelloX".to_string();
    auto_lang_ace_host::install().expect("host installs");
    let root = stable_f0_root();
    if let Err(e) = auto_lang_ace_host::build_f0_text_provider("p980_hello_drift", &root) {
        assert!(e.contains("already registered"), "{e}");
    }
    let err = s
        .run_f0("p980_hello_drift")
        .expect_err("drifted bytes must refuse");
    assert!(err.contains("drifted"), "{err}");
}

/// 复核反例（T-HELLO-R）：窗口 exe 的证据不可伪造——坏源在开窗前具名退出
/// （exit 4），不产出证据文件。
#[test]
fn p980_hello_window_exe_bad_source_exits_named() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = manifest.join("../../target/debug/examples/ace_hello_window.exe");
    let Ok(exe) = exe.canonicalize() else {
        panic!("preparation failure: window exe not built (prep stage first)");
    };
    let tmp = std::env::temp_dir().join(format!("p980-hello-bad-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let bad = tmp.join("bad.at");
    std::fs::write(
        &bad,
        "fn main() int { return 1 }
",
    )
    .unwrap();
    let ev = tmp.join("ev.json");
    let out = std::process::Command::new(&exe)
        .arg("--source")
        .arg(&bad)
        .arg("--evidence")
        .arg(&ev)
        .current_dir(&tmp)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(4), "AGE load failure exit code");
    assert!(!ev.is_file(), "no evidence for a failed load");
    let _ = std::fs::remove_dir_all(&tmp);
}
