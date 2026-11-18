//! PLAN-980 ④-A 首片 #2d：**浏览器执行档**——包片段 lane 发射的 `<script>` 段
//! （被调 helper 定义 + handler 调用）在真实浏览器（playwright/chromium）中执行。
//!
//! 输入与 #2a–#2c 同一派生真实例（`ace-node-counter` 派生 + 点分
//! `use util.steps: step`）。判据：浏览器里点击一次 → 计数从 `helper(0)` 变为
//! `helper(helper(0))`：base helper（+2）为 2→4；改 helper（+10）为 10→20。
//!
//! 边界（诚实）：本片执行的是 **SFC 的 `<script>` 段**（同一份发射文本）；AGE 源的
//! 完整 SFC template 需要"view 树→Vue 模板"生成器道（既有方言解析器不认 `node`
//! 形，实测见 §9），不在本片。
use ace_core::source::packages::PackageModuleInput;
use auto_ui_runtime::ui::ace_package_lane::PackageFragmentLane;

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

fn main_src() -> String {
    "use util.steps: step\n\
use themes.themes: Auto, Solarized\n\
\n\
node Counter {\n\
    msg { Inc, Reset }\n\
\n\
    model {\n\
        var count int = step(0)\n\
    }\n\
\n\
    view {\n\
        col {\n\
            text \"Counter\"\n\
            row {\n\
                button \"Reset\" { onclick: .Reset }\n\
                button \"+\" { onclick: .Inc }\n\
            }\n\
        }\n\
    }\n\
\n\
    on {\n\
        .Inc -> { .count = step(.count) }\n\
        .Reset -> { .count = 0 }\n\
    }\n\
}\n"
    .to_string()
}

const HELPER: &str = "pub fn step(n int) int {\n    return n + 2\n}\n";
const HELPER_EDITED: &str = "pub fn step(n int) int {\n    return n + 10\n}\n";

fn emit_script(helper: &str) -> (String, String, Vec<String>, Vec<String>) {
    let lane = PackageFragmentLane::compile(
        &[
            unit("app", "main", &main_src()),
            unit("util", "steps", helper),
        ],
        "app::main",
        &["Inc"],
    )
    .expect("package lane compiles");
    let span = lane.find_span("step(.count)").expect("handler span");
    let handler = lane
        .emit_expr_at_span("__age_state_0_count", span)
        .expect("handler emitted");
    let helper_name = handler
        .helpers
        .first()
        .expect("handler declares the helper")
        .clone();
    let body = lane
        .emit_function_body("_steps_step")
        .expect("helper body emitted");
    let params: Vec<String> = body
        .locals
        .iter()
        .filter(|l| l.role == "param")
        .map(|l| l.emitted_name.clone())
        .collect();
    (helper_name, body.text.join("\n"), handler.text, params)
}

fn run_browser(helper: &str) -> String {
    let (helper_name, helper_body, handler_lines, params) = emit_script(helper);
    let handler_join = handler_lines.join("\n");
    // 与既有浏览器档同式的 checked-add runtime（a2x_rt 冻结语义的最小内联）。
    let rt = "function a2x_add(a, b) {\n  if (!Number.isSafeInteger(a) || !Number.isSafeInteger(b) || !Number.isSafeInteger(a + b)) throw new RangeError('a2x_add overflow');\n  return a + b;\n}\n";
    let html = format!(
        "<!DOCTYPE html>\n<html><body>\n<div id=\"count\"></div>\n<button id=\"step\" onclick=\"Step()\">+</button>\n<script>\n{rt}\n// 被调 helper（包片段 lane 发射的同一份定义）\nfunction {helper_name}({params}) {{\n{helper_body}\n}}\nlet count = {helper_name}(0);\ndocument.getElementById('count').textContent = String(count);\nfunction Step() {{\n  const l0___age_s_0_count = count;\n{handler_join}\n  count = t0;\n  document.getElementById('count').textContent = String(count);\n}}\n</script>\n</body></html>\n",
        params = params.join(", "),
    );
    let dir = std::env::temp_dir().join(format!(
        "p980-pkg-script-{}-{}",
        std::process::id(),
        if helper.contains("+ 10") {
            "edited"
        } else {
            "base"
        }
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), html).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_pkg_script_browser.mjs");
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let out = std::process::Command::new(&node)
        .arg(script.to_str().unwrap())
        .arg(dir.to_str().unwrap())
        .arg(if helper.contains("+ 10") { "10" } else { "2" })
        .arg(if helper.contains("+ 10") { "20" } else { "4" })
        .output()
        .expect("node spawn");
    let _ = std::fs::remove_dir_all(&dir);
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// 真实浏览器：base helper（+2）点击一次 2→4；改 helper（+10）10→20。
#[test]
fn p980_closure_script_runs_in_a_real_browser() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_closure_script_runs_in_a_real_browser: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright required)"
        );
        return;
    }
    let base = run_browser(HELPER);
    assert!(
        base.contains("PKG_SCRIPT_BROWSER_OK"),
        "base helper browser run: {base}"
    );
    let edited = run_browser(HELPER_EDITED);
    assert!(
        edited.contains("PKG_SCRIPT_BROWSER_OK"),
        "edited helper browser run: {edited}"
    );
}
