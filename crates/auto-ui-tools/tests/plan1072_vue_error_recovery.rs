//! PLAN-1072 T-03（Vue 产品链冷生成 + 错误保旧 + 恢复验收）：
//! - Arm 1：冷目录 `gen_vue_project` 在 `AUTO_UI_ACE_VUE=1` 下编 `005-login` 与
//!   `010-contact-form`（含 `write_ace_vue_rt` 写 `src/a2x_rt.mjs`、scaffold、
//!   `@vue/compiler-sfc` + `esbuild` 编译检查——无产物构建缺件）。
//! - Arm 2：增量 `incremental_compile_changed` 遇语法错 / lane 不支持源时的
//!   **错误保旧**（`App.vue` 与 `a2x_rt.mjs` 逐字节不变）+ 修复恢复。
//! - Arm 3：全量 `gen_vue_project`（`VueProject::from_workspace` +
//!   `regenerate_source_files`）遇 lane 拒绝时的**错误保旧** + 修复恢复。
//!
//! 顺序执行单函数（`AUTO_UI_ACE_VUE` 为进程级 env）；退出前清理 env 与临时根。
use std::path::{Path, PathBuf};

use auto_ui_tools::vue::{gen_vue_project, incremental_compile_changed};

struct EnvGuard;
impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("AUTO_UI_ACE_VUE");
    }
}

struct StrictGuard;
impl StrictGuard {
    fn on() -> Self {
        auto_ui_compiler::ui_gen::validators::set_strict(true);
        StrictGuard
    }
}
impl Drop for StrictGuard {
    fn drop(&mut self) {
        auto_ui_compiler::ui_gen::validators::set_strict(false);
    }
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let p = entry.path();
        let to = dst.join(entry.file_name());
        if p.is_dir() {
            copy_dir(&p, &to);
        } else {
            std::fs::copy(&p, &to).unwrap();
        }
    }
}

/// 保持 `../../../packages/themes` 相对拓扑（原样 `005`/`010` `pac.at` 零改写）。
fn setup_example_root(tag: &str, example: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("p1072-errrec-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    copy_dir(
        &manifest.join("../../packages/themes"),
        &root.join("packages/themes"),
    );
    let ws = root.join("examples/ui").join(example);
    copy_dir(&manifest.join("../../examples/ui").join(example), &ws);
    (root, ws)
}

fn verify_sfcs_compile_with_vue_esbuild(sfc_paths: &[&Path]) {
    let script = r#"
const fs = require('fs');
const { pathToFileURL } = require('url');
const COMPILERS = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
  'd:/autostack/auto-os-config/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
  'd:/autostack/auto-down/autodown/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
];
const ESBUILDS = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/esbuild/lib/main.js',
  'd:/autostack/auto-os-config/node_modules/esbuild/lib/main.js',
  'd:/autostack/auto-down/autodown/node_modules/esbuild/lib/main.js',
];
const first = (xs) => xs.find((p) => fs.existsSync(p));
(async () => {
  const compiler = await import(pathToFileURL(first(COMPILERS)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const esbuild = (await import(pathToFileURL(first(ESBUILDS)).href)).default;
  for (const p of process.argv.slice(1)) {
    const src = fs.readFileSync(p, 'utf8');
    const { descriptor, errors } = parse(src, { filename: p });
    if (errors && errors.length) throw new Error(p + ': ' + JSON.stringify(errors));
    const compiled = compileScript(descriptor, { id: p, inlineTemplate: true });
    const out = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' });
    if (!out.code || out.code.length === 0) throw new Error(p + ': empty esbuild output');
  }
  console.log('SFC_COMPILE_CHECK_OK');
})();
"#;
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let mut cmd = std::process::Command::new(&node);
    cmd.arg("-e").arg(script);
    for p in sfc_paths {
        cmd.arg(p.to_str().unwrap());
    }
    let out = cmd.output().expect("node spawn");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success() && stdout.contains("SFC_COMPILE_CHECK_OK"),
        "SFC compile check failed: {stdout} / {stderr}"
    );
}

#[test]
fn p1072_vue_cold_gen_error_preserve_and_recovery() {
    let _guard = EnvGuard;
    std::env::set_var("AUTO_UI_ACE_VUE", "1");

    // ── Arm 1: Cold `gen_vue_project` on 005-login and 010-contact-form ──
    let (root_005, ws_005) = setup_example_root("cold-005", "005-login");
    let (root_010, ws_010) = setup_example_root("cold-010", "010-contact-form");

    gen_vue_project(&ws_005).expect("cold gen_vue_project 005 under AUTO_UI_ACE_VUE=1");
    gen_vue_project(&ws_010).expect("cold gen_vue_project 010 under AUTO_UI_ACE_VUE=1");

    let vue_005 = ws_005.join("gen/front/vue");
    let app_005_path = vue_005.join("src/App.vue");
    let rt_005_path = vue_005.join("src/a2x_rt.mjs");
    let app_005 = std::fs::read_to_string(&app_005_path).expect("005 App.vue");
    let rt_005 = std::fs::read_to_string(&rt_005_path)
        .expect("005 a2x_rt.mjs written by VueProject::generate");
    assert!(
        app_005.contains("a2x_str_eq(a2x_str_lit(email.value")
            && app_005.contains("from \"./a2x_rt.mjs\"")
            && !app_005.contains(":style=\"field_label\""),
        "005 cold gen App.vue carries lane markers + rt import + expanded style"
    );
    assert!(
        rt_005.starts_with("// AUTO-GENERATED by auto-a2x-ts (PLAN-976)"),
        "005 rt header"
    );
    for req_file in [
        "package.json",
        "vite.config.ts",
        "tsconfig.json",
        "src/main.ts",
    ] {
        assert!(
            vue_005.join(req_file).exists(),
            "005 scaffold {req_file} exists"
        );
    }

    let vue_010 = ws_010.join("gen/front/vue");
    let app_010_path = vue_010.join("src/App.vue");
    let rt_010_path = vue_010.join("src/a2x_rt.mjs");
    let app_010 = std::fs::read_to_string(&app_010_path).expect("010 App.vue");
    let rt_010 = std::fs::read_to_string(&rt_010_path)
        .expect("010 a2x_rt.mjs written by VueProject::generate");
    assert!(
        app_010.contains("a2x_str_eq")
            && app_010.contains("v-for=")
            && app_010.contains("from \"./a2x_rt.mjs\"")
            && !app_010.contains(":style=\"field_col\""),
        "010 cold gen App.vue carries lane markers + v-for + rt import + expanded style"
    );
    assert!(
        rt_010.starts_with("// AUTO-GENERATED by auto-a2x-ts (PLAN-976)"),
        "010 rt header"
    );
    for req_file in [
        "package.json",
        "vite.config.ts",
        "tsconfig.json",
        "src/main.ts",
    ] {
        assert!(
            vue_010.join(req_file).exists(),
            "010 scaffold {req_file} exists"
        );
    }

    verify_sfcs_compile_with_vue_esbuild(&[&app_005_path, &app_010_path]);

    // ── Arm 2: Incremental `incremental_compile_changed` 错误保旧 + 恢复 ──
    let (root_inc, ws_inc) = setup_example_root("incr-005", "005-login");
    let orig_005 = std::fs::read_to_string(ws_inc.join("src/front/app.at")).unwrap();
    let n0 = incremental_compile_changed(&ws_inc, None).expect("baseline incremental ok");
    assert!(n0 > 0);
    let inc_app_path = ws_inc.join("gen/front/vue/src/App.vue");
    let inc_rt_path = ws_inc.join("gen/front/vue/src/a2x_rt.mjs");
    let base_app_bytes = std::fs::read(&inc_app_path).unwrap();
    let base_rt_bytes = std::fs::read(&inc_rt_path).unwrap();

    // Failure A: broken syntax — watch(non-strict) degrades to warning + Ok(0) preserving disk; strict hard-fails preserving disk
    std::fs::write(ws_inc.join("src/front/app.at"), "widget App { broken !!!").unwrap();
    let n_bad = incremental_compile_changed(&ws_inc, None)
        .expect("non-strict incremental watch keeps loop alive with Ok(0)");
    assert_eq!(n_bad, 0, "broken syntax must write 0 files");
    assert_eq!(
        std::fs::read(&inc_app_path).unwrap(),
        base_app_bytes,
        "App.vue preserved on non-strict syntax failure"
    );
    assert_eq!(
        std::fs::read(&inc_rt_path).unwrap(),
        base_rt_bytes,
        "a2x_rt.mjs preserved on non-strict syntax failure"
    );
    {
        let _strict = StrictGuard::on();
        let err_strict = incremental_compile_changed(&ws_inc, None)
            .expect_err("strict incremental build must fail on broken syntax");
        assert!(
            err_strict.to_string().contains("Failed to compile"),
            "strict error names failure: {err_strict}"
        );
    }
    assert_eq!(
        std::fs::read(&inc_app_path).unwrap(),
        base_app_bytes,
        "App.vue preserved on strict syntax failure"
    );
    assert_eq!(
        std::fs::read(&inc_rt_path).unwrap(),
        base_rt_bytes,
        "a2x_rt.mjs preserved on strict syntax failure"
    );

    // Failure B: valid Auto syntax but lane-unsupported (F-1 shadowed param)
    let lane_absent_src = "widget App {\n    msg Msg { Set(int) }\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"S\" }\n    }\n    on {\n        .Set(n) -> { .n = n }\n    }\n}\n";
    std::fs::write(ws_inc.join("src/front/app.at"), lane_absent_src).unwrap();
    let err_b = incremental_compile_changed(&ws_inc, None).expect_err("lane-unsupported must fail");
    let msg_b = err_b.to_string();
    assert!(
        msg_b.contains("ace.vue-lane-unsupported") && msg_b.contains(".Set"),
        "named reject: {msg_b}"
    );
    assert_eq!(
        std::fs::read(&inc_app_path).unwrap(),
        base_app_bytes,
        "App.vue preserved on lane failure"
    );
    assert_eq!(
        std::fs::read(&inc_rt_path).unwrap(),
        base_rt_bytes,
        "a2x_rt.mjs preserved on lane failure"
    );

    // Recovery: write valid modified 005-login
    let recovered_inc_src = orig_005.replacen("Sign In", "Sign In Recovered", 1);
    std::fs::write(ws_inc.join("src/front/app.at"), &recovered_inc_src).unwrap();
    let n_rec = incremental_compile_changed(&ws_inc, None).expect("incremental recovery ok");
    assert!(n_rec > 0, "recovered file compiled");
    let rec_app = std::fs::read_to_string(&inc_app_path).unwrap();
    assert!(
        rec_app.contains("Sign In Recovered")
            && rec_app.contains("a2x_str_eq(a2x_str_lit(email.value")
            && rec_app.contains("from \"./a2x_rt.mjs\""),
        "recovered incremental App.vue has updated content + lane markers"
    );
    assert_eq!(
        std::fs::read(&inc_rt_path).unwrap(),
        base_rt_bytes,
        "a2x_rt.mjs intact after recovery"
    );

    // ── Arm 3: Full-path `gen_vue_project` 错误保旧 + 恢复 ──
    let base_005_full_bytes = std::fs::read(&app_005_path).unwrap();
    let base_005_rt_bytes = std::fs::read(&rt_005_path).unwrap();
    std::fs::write(ws_005.join("src/front/app.at"), lane_absent_src).unwrap();
    let err_full = gen_vue_project(&ws_005).expect_err("full path lane-unsupported must fail");
    assert!(err_full.to_string().contains("ace.vue-lane-unsupported"));
    assert_eq!(
        std::fs::read(&app_005_path).unwrap(),
        base_005_full_bytes,
        "full path App.vue preserved on failure"
    );
    assert_eq!(
        std::fs::read(&rt_005_path).unwrap(),
        base_005_rt_bytes,
        "full path a2x_rt.mjs preserved on failure"
    );

    let recovered_full_src = orig_005.replacen("Sign In", "Sign In Full Recovered", 1);
    std::fs::write(ws_005.join("src/front/app.at"), &recovered_full_src).unwrap();
    gen_vue_project(&ws_005).expect("full path recovery ok");
    let rec_full_app = std::fs::read_to_string(&app_005_path).unwrap();
    assert!(
        rec_full_app.contains("Sign In Full Recovered")
            && rec_full_app.contains("a2x_str_eq(a2x_str_lit(email.value")
            && rec_full_app.contains("from \"./a2x_rt.mjs\""),
        "recovered full App.vue has updated content + lane markers"
    );
    assert_eq!(
        std::fs::read(&rt_005_path).unwrap(),
        base_005_rt_bytes,
        "full path a2x_rt.mjs intact"
    );

    let _ = std::fs::remove_dir_all(&root_005);
    let _ = std::fs::remove_dir_all(&root_010);
    let _ = std::fs::remove_dir_all(&root_inc);
}
