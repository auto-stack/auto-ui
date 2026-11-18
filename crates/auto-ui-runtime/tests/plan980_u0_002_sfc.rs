//! PLAN-980 U0 预验收（④-A #4，用户 2026-11-18 选 A：**单文件组件路径**）：
//! **未改写的 `examples/ui/002-counter`** 经 `generate_sfc_with_lane`
//! （编译器 SFC 管线 + `ComponentFragmentLane` 体拼接）生成 SFC，并在
//! 真实 Vue 挂载中验收。
//!
//! 判据（用户口径）：①全 handler lane 派生（absent 空——入口对缺席
//! handler **保留编译器旧道体**，仅凭 DOM 变动不能确认 ACE 覆盖，lane
//! 覆盖必须显式断言）；②加/减/重置的拼接体为 a2x lane 形（旧 TS
//! fallback `+= 1` 形不得在位）；③真实挂载后页面文本 `Counter: 0 → 1 → 0`
//! （加、减、重置三键各经真实点击；再两次 `+` 得 2）。
use auto_ui_runtime::ui::ace_vue_script::{generate_sfc_with_lane, LaneHandlerBody};

fn src_002() -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../examples/ui/002-counter/src/front/app.at");
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("002 source: {e}"))
}

/// 未改写 002 → 单文件路径 SFC（挂载档与生成档共用同一出口；证据 dump
/// 经 `P980_U0_DUMP` 落 plan-artifacts，不入仓）。
fn gen_002() -> (String, Vec<LaneHandlerBody>, Vec<String>) {
    let src = src_002();
    let (sfc, bodies, absent) = generate_sfc_with_lane(&src, "p980-u0-002.at")
        .expect("unmodified 002 generates a lane SFC via the single-file path");
    if let Ok(dir) = std::env::var("P980_U0_DUMP") {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(std::path::Path::new(&dir).join("002.sfc"), &sfc);
    }
    (sfc, bodies, absent)
}

/// SFC 内 `function <Name>(…): void { … }` 的函数体文本（括号配对截取，
/// 与 `splice_lane_handler_bodies` 同锚）。
fn sfc_fn_body<'a>(sfc: &'a str, name: &str) -> &'a str {
    let fn_start = sfc
        .find(&format!("function {name}("))
        .unwrap_or_else(|| panic!("fn {name} not found in SFC"));
    let open = sfc[fn_start..]
        .find("): void {")
        .map(|o| fn_start + o + "): void {".len())
        .unwrap_or_else(|| panic!("fn {name} signature tail not found"));
    let bytes = sfc.as_bytes();
    let mut depth = 1usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &sfc[open..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    panic!("fn {name} unbalanced braces");
}

/// ①生成档：未改写 002 全 handler lane 派生 + 加/减/重置为 a2x lane 形。
/// 旧 TS fallback（`+= 1`/`-= 1`）不得在位——DOM 变动不证明 ACE 覆盖，
/// 此为生成级 lane 证据（挂载档输入=同一 SFC 字节）。
#[test]
fn p980_u0_002_single_file_all_lane_and_a2x_bodies() {
    let (sfc, bodies, absent) = gen_002();
    assert!(
        absent.is_empty(),
        "002 all-lane via the single-file path: {absent:?}"
    );
    assert_eq!(bodies.len(), 9, "6 on-block handlers + 3 view lambdas");

    let lane_body = |pat: &str| {
        bodies
            .iter()
            .find(|b| b.pattern == pat)
            .unwrap_or_else(|| {
                panic!(
                    "{pat} in {:?}",
                    bodies.iter().map(|b| &b.pattern).collect::<Vec<_>>()
                )
            })
            .lines
            .join("\n")
    };
    // on-block：加/减/重置（lane 形=a2x helper / 字面 0）。
    assert!(lane_body(".Inc").contains("a2x_add"), "Inc lane body (加)");
    assert!(lane_body(".Dec").contains("a2x_sub"), "Dec lane body (减)");
    assert!(
        lane_body(".Reset").contains("count.value = 0;"),
        "Reset lane body (重置)"
    );
    // view 内联 lambda（铸造计数序=源文本序：-、Reset、+）。
    assert!(lane_body(".__evt_onclick_1").contains("a2x_sub"));
    assert!(lane_body(".__evt_onclick_2").contains("count.value = 0;"));
    assert!(lane_body(".__evt_onclick_3").contains("a2x_add"));

    // 拼接体=lane 体逐字在位（六个计数 handler）。判别性负控只对 inc/dec
    // 族（旧 TS fallback=`+= 1`/`-= 1` 形）；reset 族旧道体与 lane 体同为
    // `count.value = 0;`，判别由 absent 空 + 拼接逐字在位承担（lane 行恰为
    // 该一行、无 helper 行）。
    let pairs = [
        (".Inc", "Inc", Some("+= 1")),
        (".Dec", "Dec", Some("-= 1")),
        (".Reset", "Reset", None),
        (".__evt_onclick_1", "__evt_onclick_1", Some("-= 1")),
        (".__evt_onclick_2", "__evt_onclick_2", None),
        (".__evt_onclick_3", "__evt_onclick_3", Some("+= 1")),
    ];
    for (pat, fn_name, fallback_form) in pairs {
        let spliced = sfc_fn_body(&sfc, fn_name);
        let lane = lane_body(pat);
        let lane_lines: Vec<&str> = lane
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        assert!(
            !lane_lines.is_empty() && lane_lines.iter().all(|l| spliced.contains(l)),
            "{fn_name} spliced body carries the lane body:\n{spliced}\nvs lane:\n{lane}"
        );
        if let Some(form) = fallback_form {
            assert!(
                !spliced.contains(form),
                "{fn_name} must not keep the old TS fallback form `{form}`: {spliced}"
            );
        } else {
            assert_eq!(
                spliced.trim(),
                "count.value = 0;",
                "{fn_name} reset spliced body is the bare literal assignment"
            );
        }
    }
    assert!(sfc.contains("Counter:"), "counter text leaf in template");
}

/// ②挂载档（排窗 `AUTO_UI_ACE_WEB_E2E=1`，playwright + Vue 真浏览器）：
/// 页面文本 `Counter: 0 → 1 → 0`——加、减、重置三键各经真实点击；再两次
/// `+` 得 2（复合）；零 page error。
#[test]
fn p980_u0_002_sfc_mounts_and_counts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_u0_002_sfc_mounts_and_counts: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright + vue required)"
        );
        return;
    }
    let (sfc, bodies, absent) = gen_002();
    assert!(
        absent.is_empty() && bodies.len() == 9,
        "mount tier requires the all-lane SFC (absent={absent:?})"
    );
    let dir = std::env::temp_dir().join(format!("p980-u0-002-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("002.sfc"), &sfc).unwrap();
    // lane helper 运行时（单文件路径 SFC 脚本不含 helper 定义；rig 按
    // component 引用 ∩ rt 导出注入 import）。
    std::fs::write(
        dir.join("a2x_rt.mjs"),
        ace_a2x_ts::script_runtime_body(ace_a2x_ts::ScriptLang::JavaScript, &[]),
    )
    .unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_002_sfc_mount.mjs");
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
        out.status.success() && stdout.contains("U0_002_MOUNT_OK"),
        "002 single-file mount: {stdout} / {stderr}"
    );
    // 判据行随 --nocapture 可见（收据证据出口）。
    for line in stdout
        .lines()
        .filter(|l| l.starts_with("[+]") || l.starts_with("[-]"))
    {
        eprintln!("{line}");
    }
}
