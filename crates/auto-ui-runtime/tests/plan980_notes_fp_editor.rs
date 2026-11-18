//! PLAN-980 ④-A #9：AutoNotes `FpNoteEditor` 浏览器输入链基线——**只读消费
//! 原样链路的现成产品产物**（auto-notes `gen/front/vue/src/components/
//! FpNoteEditor.vue`），宿主回调=emit 契约（与 app.at `on_draft_edit` 通道
//! 同一接线）。ACE 侧具名阻塞：原样 `fp_editor.at` 的 view `if-else` 形被
//! 组件面拒（`source.age.slot`——最小复现已交 981）；本基线在 981 落 view
//! 面后由 components/ 通道（④-A #9 已接线）翻转为产品 lane 件并复用同一 rig。
use std::path::PathBuf;

/// 解析序（仓内惯例）：`AUTO_NOTES_ROOT` env → 组内 sibling（.wt/lang-980
/// 组无 auto-notes 时落空）→ 主检出 `D:/autostack/auto-notes`。
fn fp_artifact() -> Option<PathBuf> {
    let rel = "gen/front/vue/src/components/FpNoteEditor.vue";
    if let Ok(root) = std::env::var("AUTO_NOTES_ROOT") {
        let at = PathBuf::from(root).join(rel);
        if at.exists() {
            return Some(at);
        }
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest.join("../../../auto-notes").join(rel);
    if sibling.exists() {
        return Some(sibling);
    }
    let main = PathBuf::from("D:/autostack/auto-notes").join(rel);
    main.exists().then_some(main)
}

/// 漂移侦测（轻量钉）：emit 契约锚点 + 键入链路锚点。
fn assert_artifact_shape(src: &str) {
    for anchor in [
        "emit('DraftEdit', v)",
        "function DraftEdit(v: any): void",
        "defineEmits",
    ] {
        assert!(
            src.contains(anchor),
            "anchor `{anchor}` missing — artifact drifted"
        );
    }
}

#[test]
fn p980_notes_fp_editor_input_chain_mounts() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!(
            "p980_notes_fp_editor_input_chain_mounts: not scheduled \
             (set AUTO_UI_ACE_WEB_E2E=1; browser tier, playwright + vue required)"
        );
        return;
    }
    let Some(artifact) = fp_artifact() else {
        eprintln!("p980_notes_fp_editor: auto-notes gen artifact not present — named skip");
        return;
    };
    let src = std::fs::read_to_string(&artifact).expect("artifact readable");
    assert_artifact_shape(&src);
    let dir = std::env::temp_dir().join(format!("p980-notes-fp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("component.sfc"), &src).unwrap();
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest.join("../../examples/ui/ace-web-lane/tests/test_notes_fp_editor.mjs");
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
        out.status.success() && stdout.contains("NOTES_FP_OK"),
        "notes fp editor input chain: {stdout} / {stderr}"
    );
    for line in stdout
        .lines()
        .filter(|l| l.starts_with("[+]") || l.starts_with("[-]"))
    {
        eprintln!("{line}");
    }
}
