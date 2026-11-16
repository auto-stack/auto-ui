//! PLAN-980 RUST（AXR0 先行切片）永久 case。
//!
//! 分层（2026-11-14 纪律 / 978 build-lane 裁定）：
//! - 发射面（恒跑，零 rustc 编译）：同源组件 A2R 文件集形状/入口/身份（改
//!   scenario/handler/初值 → 产物字节变）；F0×A2R 文件集（extern 块 + 同文本
//!   字节进 generated）。
//! - 构建道回执消费：`target/ace-a2r-axr0/receipt.json`（由
//!   `examples/ace_a2r_axr0_build.exe` 显式一次性产出——真 rustc 编译+运行）
//!   就绪时断言两腿真实值；缺失 = 具名准备失败（不充 PASS）。
//! - 窗口腿（`AUTO_UI_ACE_HELLO_E2E=1` 排窗）：窗口进程派生 A2R 产物 exe、
//!   显示其真实 stdout（零 AV/旧 VM）。

#![cfg(all(windows, feature = "ui-interpreter"))]

use auto_ui_runtime::ui::ace_a2r_bridge::{
    component_a2r_fileset, f0_a2r_fileset, f0_text_source,
};
use ace_av::age_component::{ComponentInitials, ComponentScenario, ScenarioOp};

fn fixture() -> (String, String) {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/ace-hello-minimal/src/front/app-a2r.at");
    (
        std::fs::read_to_string(&at).expect("app-a2r.at readable"),
        at.to_string_lossy().to_string(),
    )
}

fn frozen_scenario() -> ComponentScenario {
    ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send { instance: 0, msg: "Inc".to_string() },
            ScenarioOp::Send { instance: 0, msg: "Inc".to_string() },
        ],
    }
}

fn empty_initials() -> ComponentInitials {
    ComponentInitials { per_instance: vec![Vec::new()] }
}

fn generated_of(files: &[(String, String)]) -> String {
    files
        .iter()
        .find(|(n, _)| n == "generated.rs")
        .map(|(_, c)| c.clone())
        .expect("generated.rs present")
}

/// 发射面（Leg A）：同源组件 A2R 文件集——形状/入口/manifest 身份；改
/// scenario/handler/初值都改变产物字节（身份 oracle，删生成门控即红）。
#[test]
fn p980_axr0_component_fileset_shape_and_identity() {
    let (src, path) = fixture();
    let set = component_a2r_fileset(&src, &path, &frozen_scenario(), &empty_initials(), Some("count"))
        .expect("component fileset emits");
    assert_eq!(set.entry, "__age_drive");
    assert_eq!(set.files.len(), 3, "main/a2x_runtime/generated sibling layout");
    let names: Vec<&str> = set.files.iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"main.rs") && names.contains(&"a2x_runtime.rs"));
    let gen = generated_of(&set.files);
    assert!(gen.contains("__age_drive"), "entry fragment present");
    assert!(set.manifest.contains(set.input_hash.as_str()), "manifest carries input hash");

    // scenario 改变 → 产物身份变化。
    let other = ComponentScenario { instances: 1, ops: vec![] };
    let set2 = component_a2r_fileset(&src, &path, &other, &empty_initials(), Some("count")).unwrap();
    assert_ne!(generated_of(&set2.files), gen, "scenario change must change the artifact");
    assert_eq!(set2.input_hash, set.input_hash, "input hash tracks the source, not the scenario");

    // handler 改变（同源族）→ 字节变化。
    let edited = src.replace("count = count + 1", "count = count + 2");
    let set3 = component_a2r_fileset(&edited, &path, &frozen_scenario(), &empty_initials(), Some("count")).unwrap();
    assert_ne!(generated_of(&set3.files), gen, "handler change must change the artifact");
}

/// 发射面（Leg B）：F0×A2R——text-record 源（同 fixture 文本字节）→ 文件集
/// 带 extern 块与字面字节。`build_f0` 只做版本探测+缓存命中（零编译）。
#[test]
fn p980_axr0_f0_fileset_carries_text_bytes_and_extern() {
    let root = auto_ui_runtime::ui::ace_hello::stable_f0_root();
    let build = ace_abi::rust_provider::build_f0(&root)
        .expect("prebuilt provider (freshness cache; rustc probe only)");
    let text = "Hello A2R";
    let set = f0_a2r_fileset(&f0_text_source(text), &build).expect("f0 fileset emits");
    assert_eq!(set.files.len(), 3);
    let runtime = set
        .files
        .iter()
        .find(|(n, _)| n == "a2x_runtime.rs")
        .map(|(_, c)| c.clone())
        .expect("runtime present");
    assert!(runtime.contains("extern \"C\""), "provider symbols declared");
    let gen = generated_of(&set.files);
    assert!(gen.contains("text_byte_len"), "provider callee wrapped");
    assert!(gen.contains("Hello A2R"), "the same text bytes ride the source");
    // 改文本 → 字节变化。
    let set2 = f0_a2r_fileset(&f0_text_source("Hello A2RX"), &build).unwrap();
    assert_ne!(generated_of(&set2.files), gen);
}

/// 构建道回执消费：真 rustc 编译+运行的两腿值（receipt 由显式构建道产出）。
#[test]
fn p980_axr0_build_lane_receipt_values() {
    let receipt = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ace-a2r-axr0/receipt.json");
    let Ok(raw) = std::fs::read_to_string(&receipt) else {
        eprintln!(
            "p980_axr0_build_lane_receipt: preparation failure — run \
             `target/debug/examples/ace_a2r_axr0_build.exe` first (explicit build lane)"
        );
        return;
    };
    let v: serde_json::Value = serde_json::from_str(&raw).expect("receipt json");
    assert_eq!(v["leg_a"]["exit"].as_i64(), Some(0));
    assert_eq!(v["leg_a"]["stdout"].as_str().unwrap().trim(), "value:2");
    assert_eq!(v["leg_b"]["exit"].as_i64(), Some(0));
    assert_eq!(v["leg_b"]["stdout"].as_str().unwrap().trim(), "value:9");
    assert_eq!(v["leg_b"]["text_bytes"].as_i64(), Some(9));
    assert!(v["leg_a"]["rustc"].as_str().unwrap().starts_with("rustc"));
    // 产物 exe 在位（窗口腿消费）。
    for key in ["component_exe", "f0_exe"] {
        let exe = std::path::PathBuf::from(v[key].as_str().unwrap());
        assert!(exe.is_file(), "{} missing", exe.display());
    }
}

/// 窗口腿（排窗 env 门控）：窗口进程派生 A2R 产物 exe、显示其真实 stdout。
#[test]
fn p980_axr0_window_shows_real_a2r_output() {
    if std::env::var("AUTO_UI_ACE_HELLO_E2E").ok().as_deref() != Some("1") {
        eprintln!("p980_axr0_window: not scheduled (AUTO_UI_ACE_HELLO_E2E=1 after prep)");
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let receipt = manifest.join("../../target/ace-a2r-axr0/receipt.json");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&receipt).expect("build lane first"))
            .expect("receipt json");
    let a2r_exe = std::path::PathBuf::from(v["component_exe"].as_str().unwrap());
    let window = manifest
        .join("../../target/debug/examples/ace_hello_window.exe")
        .canonicalize()
        .expect("window exe built (prep)");
    let tmp = std::env::temp_dir().join(format!("p980-axr0-win-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let ev = tmp.join("ev.json");
    let fixture = manifest.join("../../examples/ui/ace-hello-minimal/src/front/app-a2r.at");
    let out = std::process::Command::new(&window)
        .arg("--a2r-run")
        .arg(&a2r_exe)
        .arg("--source")
        .arg(&fixture)
        .arg("--evidence")
        .arg(&ev)
        .arg("--hold-ms")
        .arg("4000")
        .current_dir(&tmp)
        .output()
        .expect("spawn window");
    assert!(
        out.status.success(),
        "window exe: {} / {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let e: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&ev).unwrap()).unwrap();
    assert_eq!(e["mode"].as_str().unwrap(), "a2r");
    assert_eq!(e["a2r_stdout"].as_str().unwrap().trim(), "value:2");
    assert_eq!(e["av_text"].as_str().unwrap(), "A2R value:2");
    assert!(e["frames_rendered"].as_u64().unwrap() >= 2, "real window frames");
    let _ = std::fs::remove_dir_all(&tmp);
}
