//! PLAN-980 B-WEB 真实语料首刀（④-A #6）：**原样 Jade Garden
//! `quick_switcher.at`**（auto-down，宿主侧 sha 漂移以行数+锚点行+生成级
//! 断言侦测）经产品出口 `generate_sfc_with_lane_product` 沿正式生成链验证
//! **watch 族行为**——独立可验收路径（无 back.api 合同；ext 由宿主侧
//! 夹具承载，与真实链路 `@/ext` 别名同职责）。
//!
//! lane 边界（诚实）：本文件 `use {` 块（TS ext 组合式）命中组件面
//! 单文件 use 具名拒（981 合同）——on-block 7 handler 全缺席（旧道承载，
//! 双轨期合同）；本刀验证=产品链消费原样真实语料 + watch 族行为，不含
//! lane 覆盖声明。
use auto_ui_runtime::ui::ace_vue_script::generate_sfc_with_lane_product;

/// 语料位置（auto-down 主检出/组内 sibling 同形相对路径）；缺失=具名跳过。
fn jg_source() -> Option<String> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let at = manifest.join("../../../auto-down/jade-garden/front/auto/src/front/quick_switcher.at");
    std::fs::read_to_string(&at).ok()
}

/// 漂移侦测（无 sha2 依赖的轻量钉）：行数 + watch 锚点行 + use 块原样。
fn assert_corpus_shape(src: &str) {
    let n = src.lines().count();
    assert_eq!(n, 219, "jade-garden quick_switcher.at drifted (lines={n})");
    for anchor in ["watch {", ".open -> {", ".filtered -> {"] {
        assert!(
            src.contains(anchor),
            "anchor `{anchor}` missing — corpus drifted"
        );
    }
    assert!(
        src.contains("use {") && src.contains("quick_switcher_ext.ts"),
        "ext use block missing — corpus drifted"
    );
}

/// ①生成档（CPU，语料缺失时具名跳过）：原样语料经产品出口可生成；
/// watch 双条发射进 SFC；lane 边界如实=on-block 7 handler 具名缺席
/// （use 块墙，981 域）；产物为旧道承载（双轨合同）。
#[test]
fn p980_bweb_jg_quick_switcher_product_generation() {
    let Some(src) = jg_source() else {
        eprintln!("p980_bweb_jg: corpus not present (auto-down checkout missing) — named skip");
        return;
    };
    assert_corpus_shape(&src);
    let product = generate_sfc_with_lane_product(&src, "jg-quick-switcher.at")
        .expect("original jade-garden source generates through the product entry");
    let mut absent = product.absent.clone();
    absent.sort();
    assert_eq!(
        absent,
        vec![
            ".CloseOverlay",
            ".HoverFile",
            ".NextItem",
            ".PrevItem",
            ".QueryInput",
            ".SelectCurrent",
            ".SelectFile",
        ],
        "use-block wall: all on-block handlers stay on the compiler track (981 domain)"
    );
    assert!(product.lane_bodies.is_empty(), "no lane coverage claimed");
    // watch 族发射在位（生成器道现状），ext 导入原样。
    assert!(product.sfc.contains("watch(open,"), "watch(open) emitted");
    assert!(
        product.sfc.contains("watch(filtered,"),
        "watch(filtered) emitted"
    );
    assert!(
        product.sfc.contains("quick_switcher_ext"),
        "ext import preserved from the original source"
    );
    // 双轨产物不带 a2x_rt import（脚本零 a2x 引用——产品出口合同）。
    assert!(
        !product.sfc.contains("./a2x_rt.mjs"),
        "no a2x import when the script references no helpers"
    );
}

/// ②挂载档（排窗 `AUTO_UI_ACE_WEB_E2E=1` + 语料在场）：真实浏览器验证
/// watch 行为——watch(open)=query 清空+focus；watch(filtered)=过滤变化
/// ⇒ selected_index 归零（高亮回第一项）；过滤/键盘导航（旧道体）正常。
#[test]
fn p980_bweb_jg_quick_switcher_watch_behavior_mounts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_bweb_jg_watch_behavior_mounts: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright + vue required)"
        );
        return;
    }
    let Some(src) = jg_source() else {
        eprintln!("p980_bweb_jg: corpus not present (auto-down checkout missing) — named skip");
        return;
    };
    assert_corpus_shape(&src);
    let product = generate_sfc_with_lane_product(&src, "jg-quick-switcher.at")
        .expect("product SFC generates");
    let dir = std::env::temp_dir().join(format!("p980-jg-watch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), &product.sfc).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_jg_watch_mount.mjs");
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
        out.status.success() && stdout.contains("JG_WATCH_MOUNT_OK"),
        "jg watch mount: {stdout} / {stderr}"
    );
    for line in stdout
        .lines()
        .filter(|l| l.starts_with("[+]") || l.starts_with("[-]"))
    {
        eprintln!("{line}");
    }
}
