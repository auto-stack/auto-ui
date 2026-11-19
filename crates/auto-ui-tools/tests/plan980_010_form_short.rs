//! PLAN-980 ④-A #9：原样 `010-contact-form` 短回归。
//! **PLAN-1072 刀B 修订（2026-11-19）**：010 整例 lane 翻转（view-for
//! 结构承载+faqs 无类型列表字段=仅视图数据（1071 边界）+`style: if` 条件
//! 样式值+`__bind_*` 方言合成件出 lane 规格集）——①键下**生成成功**且
//! 产物含 lane 体标记（a2x 检查函数道：SetTheme 的 a2x_str_eq）；②键关
//! legacy 产物含三 handler；③浏览器短回归（排窗）不变（legacy 基线）。
//! 历史口径（0/6 具名拒=faqs+控制流两族 981 域墙）随刀收档。
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
    // ①键在场=lane 生成成功（PLAN-1072 刀B 翻转：absent=[]，产物含受检体
    // 标记——SetTheme 检查函数道的 a2x_str_eq；faqs 经 view-data 承载）。
    let ws = temp_010("keyon");
    std::env::set_var("AUTO_UI_ACE_VUE", "1");
    incremental_compile_changed(&ws, None).expect("010 derives under the key (knife B flip)");
    std::env::remove_var("AUTO_UI_ACE_VUE");
    let app = std::fs::read_to_string(ws.join("gen/front/vue/src/App.vue")).unwrap();
    assert!(
        app.contains("a2x_str_eq") && app.contains("a2x_rt"),
        "lane body markers present in the keyed artifact"
    );
    assert!(
        app.contains("v-for=") && app.contains("faqs"),
        "the FAQ loop renders via v-for over the view-data field"
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
