//! PLAN-980 ④-A #3：**AGE `view` 树 → Vue SFC** 的生成与真实挂载。
//!
//! 判据：①同一份源生成的 SFC 含模板（按钮/事件绑定）与脚本（refs/handler/helper/
//! `defineExpose` 状态）；②真实 Vue 挂载后点击 `+`/`Reset` 得 `0 → 1 → 0`（2 次 `+`
//! 得 2）；③未改写 `002-counter` 的**首个实际阻断**被具名记录（不静默）。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_sfc::generate_package_sfc;

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 派生真实例（0 基计数器 + 跨模块 helper `inc1`）：`+` 走 helper、`-` 内联减一、
/// `Reset` 归零——模板由 view 树生成，脚本由包片段 lane 发射。
fn counter_src() -> String {
    "use util.steps: inc1\n\nnode Counter {\n    msg { Inc, Dec, Reset }\n\n    model {\n        var count int = 0\n    }\n\n    view {\n        col {\n            text \"Counter\"\n            row {\n                button \"Reset\" { onclick: .Reset }\n                button \"-\" { onclick: .Dec }\n                button \"+\" { onclick: .Inc }\n            }\n        }\n    }\n\n    on {\n        .Inc -> { .count = inc1(.count) }\n        .Dec -> { .count = .count - 1 }\n        .Reset -> { .count = 0 }\n    }\n}\n"
        .to_string()
}

const HELPER: &str = "pub fn inc1(n int) int {\n    return n + 1\n}\n";

fn gen() -> String {
    generate_package_sfc(
        &[
            unit("app", "main", &counter_src()),
            unit("util", "steps", HELPER),
        ],
        "app::main",
    )
    .expect("derived counter SFC generates")
}

/// ①生成物结构：模板按钮/事件、refs、handler、helper 定义与公开状态。
#[test]
fn p980_package_sfc_has_template_script_and_state() {
    let sfc = gen();
    assert!(sfc.contains("<template>"), "template block");
    assert!(
        sfc.contains("<button @click=\"onInc\">+</button>"),
        "button + is bound to the generated handler: {sfc}"
    );
    assert!(
        sfc.contains("<button @click=\"onReset\">Reset</button>"),
        "button Reset is bound"
    );
    assert!(
        sfc.contains("const count = ref(0)"),
        "model initial is a ref"
    );
    assert!(sfc.contains("function onInc()"), "handler function emitted");
    assert!(
        sfc.contains("a2x_fn_") && sfc.contains("inc1"),
        "helper definition is emitted with its mapped name"
    );
    assert!(
        sfc.contains("defineExpose({ __state:"),
        "component publishes its state"
    );
}

/// ②真实 Vue 挂载 + 点击：0 → 1 → 0（再两次 `+` → 2）。
#[test]
fn p980_package_sfc_mounts_and_counts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_package_sfc_mounts_and_counts: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright + vue required)"
        );
        return;
    }
    let dir = std::env::temp_dir().join(format!("p980-sfc-mount-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), gen()).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_package_sfc_mount.mjs");
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
        out.status.success() && stdout.contains("SFC_MOUNT_OK"),
        "vue mount: {stdout} / {stderr}"
    );
}

/// ③未改写的 `002-counter` 原源：记录**首个实际阻断**（不通过即具名，不静默）。
#[test]
fn p980_package_sfc_002_first_blocker_is_named() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    let src = std::fs::read_to_string(&at).expect("002 source readable");
    let err = generate_package_sfc(&[unit("app", "main", &src)], "app::main")
        .expect_err("raw 002 is still blocked by a language gap; record it by name");
    assert!(
        err.contains("source.lex") || err.contains("source.syntax") || err.contains("source.age."),
        "first blocker must be a named diagnostic: {err}"
    );
    eprintln!("002 first blocker: {err}");
}
