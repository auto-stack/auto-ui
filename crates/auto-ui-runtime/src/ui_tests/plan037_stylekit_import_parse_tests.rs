//! PLAN-037 有界修复回归：投机解析路径（collect_module_imports）此前缺
//! stylekit 共享配方预注册（build_dynamic_component_inner A-02 的对应面），
//! 自带 `use stylekit.styles: <name>` 的子模块（PLAN-637 sidebar.at/editor.at
//! 实案）在 plan-446 C1 引导门 fatal（015-notes VM 启动反例，主检出基线
//! 复现：sidebar.at:111:18 / editor.at:59:3 undefined variable）。
//! 修复后本测试钉住：此类模块经 collect_module_imports 解析不再登记
//! parse-failure（ui_module_parse_failures 为空，符号照常入 out）。
use std::collections::HashSet;
use std::path::PathBuf;

fn write(path: &std::path::Path, content: &str) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[test]
fn stylekit_styles_import_module_parses_without_c1_fatal() {
    let tmp = std::env::temp_dir().join(format!(
        "p037-stylekit-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    // stylekit 共享配方模块（styles.at，PLAN-635/637 通道形态：pub 配方）。
    write(
        &tmp.join("stylekit").join("styles.at"),
        r#"pub style caption_text = "text-xs text-muted-foreground"
"#,
    );
    // 消费模块（015-notes sidebar.at/editor.at 同构：命名导入 + 视图内使用）。
    let module = tmp.join("panel.at");
    write(
        &module,
        r#"use stylekit.styles: caption_text

widget Panel {
    view {
        col {
            text "Notes" {
                style: caption_text
            }
        }
    }
}
"#,
    );

    {
        let mut failures = crate::ui_module_parse_failures().lock().unwrap();
        failures.clear();
    }

    let mut visited = HashSet::new();
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut session = auto_lang_core::compile::CompileSession::new();
    let mut import_store_decls = Vec::new();
    crate::dynamic_build::collect_module_imports(
        &module,
        &mut visited,
        &mut out,
        &mut seen,
        &mut session,
        None,
        &mut import_store_decls,
    );

    // 修复的可观察面 = plan-446 C1 致命门的登记表：修复前该模块必然登记
    // parse failure（015 VM 引导 fatal 的同型）；修复后解析在预注册的配方
    // 注册表上成功，不产生登记。（遍历后注册表可能被后续递归模块重 clear——
    // 预注册只需在该模块解析期存活——故以登记表为准，不以后置注册表为准。）
    let failures = crate::ui_module_parse_failures().lock().unwrap();
    let module_canon: PathBuf = module.canonicalize().unwrap_or(module.clone());
    let hit: Vec<&String> = failures
        .iter()
        .filter(|f| f.contains(module_canon.to_string_lossy().as_ref()))
        .collect();
    assert!(
        hit.is_empty(),
        "stylekit.styles import module still registered as parse failure: {hit:?}"
    );
    drop(failures);
    // 导入确实被跟随解析（stylekit/styles.at 进入 visited）。
    let styles_canon = tmp
        .join("stylekit")
        .join("styles.at")
        .canonicalize()
        .unwrap_or_else(|_| tmp.join("stylekit").join("styles.at"));
    assert!(
        visited.contains(&styles_canon),
        "stylekit/styles.at not visited (import not followed): {visited:?}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
