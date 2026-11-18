//! PLAN-980 ④-A #9：原样 `010-contact-form` 短回归。**诚实口径**：010 当前
//! lane **0/6**（`var faqs = [...]` 无类型列表字段=文件级 lane 编译拒 +
//! `.SetTheme` if-else 体=控制流边界——两族 981 域语言面墙，最小复现随
//! style 复测/#9 收据在案）。故：①浏览器行为基线=legacy 产物（输入绑定/
//! 提交回执/零错误）；②"ACE 断言"以**键下具名拒实证**承载（fail-closed
//! 合同的可观察行为——缺席清单含 .Submit/.SetTheme/.SetAccent 全量）；
//! ③键关 legacy 产物含三 handler（主题切换的产物面证据——宿主通道腿
//! 无页面内触发件）。
use auto_ui_tools::vue::incremental_compile_changed;

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            copy_dir(&p, &dst.join(entry.file_name()));
        } else {
            std::fs::copy(&p, dst.join(entry.file_name())).unwrap();
        }
    }
}

fn temp_010(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("p980-010-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    copy_dir(&manifest.join("../../examples/ui/010-contact-form"), &dir);
    dir
}

/// ①–③单函数顺序执行（env 键进程级，测试间不可并行切换——与
/// `plan980_ace_vue_tool_wiring` 同纪律）。
#[test]
fn p980_010_short_regression_named_reject_legacy_and_mount() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("browser tier not scheduled (set AUTO_UI_ACE_WEB_E2E=1) — CPU 段照跑");
    }
    // ①键在场=具名拒（0/6 全量缺席——fail-closed 实证；两族 981 域墙见文件头）。
    let ws = temp_010("keyon");
    std::env::set_var("AUTO_UI_ACE_VUE", "1");
    let err = incremental_compile_changed(&ws, None)
        .expect_err("010 under the key must fail closed with the full absent list");
    std::env::remove_var("AUTO_UI_ACE_VUE");
    let msg = err.to_string();
    for handler in [".Submit", ".SetTheme", ".SetAccent"] {
        assert!(msg.contains(handler), "absent list names {handler}: {msg}");
    }
    assert!(
        msg.contains("ace.vue-lane-unsupported"),
        "named code: {msg}"
    );
    let _ = std::fs::remove_dir_all(&ws);

    // ②键未设=legacy 生成（App.vue 含三 handler——主题切换的产物面证据）。
    let ws = temp_010("keyoff");
    std::env::remove_var("AUTO_UI_ACE_VUE");
    incremental_compile_changed(&ws, None).expect("legacy generation ok");
    let app = std::fs::read_to_string(ws.join("gen/front/vue/src/App.vue")).unwrap();
    for handler in ["function Submit", "function SetTheme", "function SetAccent"] {
        assert!(app.contains(handler), "legacy artifact has {handler}");
    }
    let _ = std::fs::remove_dir_all(&ws);

    // ③浏览器短回归（排窗）：输入绑定+提交回执。
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("010 mount tier: not scheduled (set AUTO_UI_ACE_WEB_E2E=1)");
        return;
    }
    let ws = temp_010("mount");
    std::env::remove_var("AUTO_UI_ACE_VUE");
    incremental_compile_changed(&ws, None).expect("legacy generation ok");
    let dir = std::env::temp_dir().join(format!("p980-010-artifact-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(ws.join("gen/front/vue/src/App.vue"), dir.join("App.sfc")).unwrap();
    let _ = std::fs::remove_dir_all(&ws);
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_010_form.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(script.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .output()
        .expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success() && stdout.contains("P980_010_FORM_OK"),
        "010 form short regression: {stdout} / {stderr}"
    );
    for line in stdout
        .lines()
        .filter(|l| l.starts_with("[+]") || l.starts_with("[-]"))
    {
        eprintln!("{line}");
    }
}
