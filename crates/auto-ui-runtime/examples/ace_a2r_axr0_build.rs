//! PLAN-980 RUST（AXR0）：**显式构建道**——同源 AGE 组件经新 A2R 生成 →
//! 真 rustc 编译 → 真运行，回执落 `target/ace-a2r-axr0/receipt.json`。
//!
//! 纪律（2026-11-14 / 978 build-lane 裁定）：真 rustc 不进日用测试体；本
//! example 是一次性构建道入口（V 阶段显式执行），日用例只消费回执与发射
//! 身份。两腿：
//! * Leg A（同源语义）：fixture `app-a2r.at` → 组件场景 A2R 文件集 →
//!   rustc → 运行 → `value:<drive>`（冻结 scenario：2×Inc → count=2）。
//! * Leg B（F0×A2R，967/T-12 桥）：同一 fixture 文本字节（"Hello A2R"）
//!   经 text-record 源 → 文件集（声明 provider 符号）→ rustc 链接**预构建**
//!   provider（target/ace-hello-f0 新鲜度缓存）→ 运行 → `value:10`。
//!
//! 用法：ace_a2r_axr0_build.exe [--receipt <path>]

use auto_ui_runtime::ui::ace_a2r_bridge::{component_a2r_fileset, f0_a2r_fileset, f0_text_source};
use auto_ui_runtime::ui::ace_hello::stable_f0_root;
use ace_av::age_component::{ComponentInitials, ComponentScenario, ScenarioOp};

fn main() {
    let mut receipt_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--receipt" {
            receipt_path = args.next();
        }
    }
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("../../examples/ui/ace-hello-minimal/src/front/app-a2r.at");
    let src = std::fs::read_to_string(&fixture).expect("fixture readable");
    let text: &str = "Hello A2R";

    let out_root = manifest_dir.join("../../target/ace-a2r-axr0");
    let _ = std::fs::remove_dir_all(&out_root);
    std::fs::create_dir_all(&out_root).expect("out root");

    // ---- Leg A：组件场景（同源语义） ----
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send { instance: 0, msg: "Inc".to_string() },
            ScenarioOp::Send { instance: 0, msg: "Inc".to_string() },
        ],
    };
    let initials = ComponentInitials { per_instance: vec![Vec::new()] };
    let set = component_a2r_fileset(&src, "app-a2r.at", &scenario, &initials, Some("count"))
        .expect("component fileset emits");
    let dir_a = out_root.join("component");
    std::fs::create_dir_all(&dir_a).expect("dir a");
    for (name, content) in &set.files {
        std::fs::write(dir_a.join(name), content).expect("write fileset file");
    }
    std::fs::write(dir_a.join("manifest.json"), &set.manifest).expect("write manifest");
    let (exit_a, out_a, rustc_a) = rustc_and_run(&dir_a, "axr0_component.exe", None);

    // ---- Leg B：F0×A2R（同文本字节，链接预构建 provider） ----
    auto_lang_ace_host::install().expect("ace host installs");
    let provider_root = stable_f0_root();
    if let Err(e) = auto_lang_ace_host::build_f0_text_provider("f0_hello", &provider_root) {
        assert!(e.contains("already registered"), "{e}");
    }
    // stable root 即 scratch 根（f0-provider-crate/ + out/）；新鲜度缓存命中时零 rustc。
    let build = ace_abi::rust_provider::build_f0(&provider_root)
        .expect("prebuilt provider (cached)");
    let f0set = f0_a2r_fileset(&f0_text_source(text), &build).expect("f0 fileset emits");
    let dir_b = out_root.join("f0");
    std::fs::create_dir_all(&dir_b).expect("dir b");
    for (name, content) in &f0set.files {
        std::fs::write(dir_b.join(name), content).expect("write f0 fileset file");
    }
    // provider 产物随目录布防（loader 找 dll；rustc -L 找 import lib）。
    std::fs::copy(&f0set.provider_cdylib, dir_b.join("f0_text_provider.dll")).expect("stage dll");
    std::fs::copy(&f0set.provider_import_lib, dir_b.join("f0_text_provider.lib")).expect("stage lib");
    let (exit_b, out_b, rustc_b) = rustc_and_run(&dir_b, "axr0_f0.exe", Some(&dir_b));

    // ---- 回执 ----
    let receipt = serde_json::json!({
        "fixture": fixture.display().to_string(),
        "leg_a": {
            "dir": dir_a.display().to_string(),
            "exit": exit_a,
            "stdout": out_a,
            "expected_value": 2,
            "input_hash": set.input_hash,
            "rustc": rustc_a,
        },
        "leg_b": {
            "dir": dir_b.display().to_string(),
            "exit": exit_b,
            "stdout": out_b,
            "expected_value": text.len() as i64,
            "text_bytes": text.len(),
            "rustc": rustc_b,
            "provider_root": provider_root.display().to_string(),
        },
        "component_exe": dir_a.join("axr0_component.exe").display().to_string(),
        "f0_exe": dir_b.join("axr0_f0.exe").display().to_string(),
    });
    let path = receipt_path
        .map(std::path::PathBuf::from)
        .unwrap_or(out_root.join("receipt.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).expect("write receipt");
    println!("axr0 build-lane receipt: {}", path.display());
    println!("leg_a: exit={exit_a} out={}", out_a.trim());
    println!("leg_b: exit={exit_b} out={}", out_b.trim());
    if exit_a != 0 || exit_b != 0 || !out_a.trim().ends_with("value:2") || !out_b.trim().ends_with(&format!("value:{}", text.len())) {
        std::process::exit(9);
    }
}

fn rustc_and_run(
    dir: &std::path::Path,
    exe_name: &str,
    link_dir: Option<&std::path::Path>,
) -> (i32, String, String) {
    let rustc_identity = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut cmd = std::process::Command::new("rustc");
    cmd.args(["--edition", "2021", "-O", "main.rs", "-o", exe_name]);
    if let Some(ld) = link_dir {
        cmd.arg("-L").arg(ld.display().to_string());
    }
    cmd.current_dir(dir);
    let compiled = cmd.output().expect("rustc spawn");
    assert!(
        compiled.status.success(),
        "rustc failed: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let exe = dir.join(exe_name);
    let run = std::process::Command::new(&exe).current_dir(dir).output().expect("run spawn");
    (
        run.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&run.stdout).into_owned(),
        rustc_identity,
    )
}
