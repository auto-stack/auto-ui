//! auto-notes NOTES-004 原生门阻塞回归：`use` 模块自身携带的
//! `use stylekit.styles: <name>` 命名导入，必须在 **plan-446 C1 致命门控的那次
//! 解析**（`collect_module_imports`）前预注册进活配方注册表。
//!
//! 现场（auto-notes sidebar.at/editor.at，PLAN-637 stylekit 契约形态）：
//! PLAN-668 A-02 只给子件注册解析加了逐模块预注册；紧随其后的
//! `register_transitive_widgets` 对每个子模块再次 `prepare_style_recipe_imports`
//! （内部 clear）把注册表覆盖，到 `collect_module_imports` 解析同一模块时
//! `caption_text` 已不在表中 → `check_symbol` UndefinedVariable → 整模块 fatal。
//!
//! 夹具复刻触发条件：模块既 `use stylekit.styles: caption_text`，又 `use` 另一
//! 个可解析的兄弟模块（让传递注册递归发生并清表）。同时钉住报告中被误归因的
//! "本地样式名进 if/else 分支"形态——它走 parser 作用域 `bind_var`，本就可过。

use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, src: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, src).unwrap();
}

const PAC: &str = r#"name: "notes"
version: "1.0.0"
scene: "ui"
render: "vue"
title: "Notes"

dep stylekit {
    path: "vendor/stylekit"
}
"#;

const STYLEKIT_PAC: &str = r#"name: "stylekit"
version: "0.1.0"
scene: "ui"
render: "vue"
title: "StyleKit"
"#;

const STYLEKIT_STYLES: &str = r#"
pub style caption_text = "text-xs text-muted-foreground"
"#;

const HOST: &str = r#"
use sidebar: NavTree

widget App {
    view {
        col {
            NavTree() {}
        }
    }
}
"#;

/// 兄弟模块：让 sidebar 的传递注册递归发生（递归中对本模块 prepare → 清表）。
const HELPER: &str = r#"
widget Helper {
    view {
        text "helper"
    }
}
"#;

/// 复刻 auto-notes sidebar.at 的三类 style 值形态：
/// 1. 本地配方名作 if/else 分支（报告构造 1，实际可过）
/// 2. 跨包命名导入配方名（报告构造 2，真实失败点）
/// 3. 字面量
const SIDEBAR: &str = r##"
use helper: Helper

style chip = "h-7 rounded-full bg-muted"
style chip_on = "h-7 rounded-full bg-primary/10 text-primary"

use stylekit.styles: caption_text

widget NavTree {
    model {
        var on bool = true
    }
    view {
        col {
            button {
                style: if .on { chip_on } else { chip }
                text "#" { style: caption_text }
                text "tag" { style: "text-xs" }
            }
            text "No matching notes" { style: caption_text }
            Helper() {}
        }
    }
}
"##;

fn layout(root: &Path) -> PathBuf {
    write(root, "pac.at", PAC);
    write(root, "vendor/stylekit/pac.at", STYLEKIT_PAC);
    write(root, "vendor/stylekit/src/front/styles.at", STYLEKIT_STYLES);
    write(root, "src/front/app.at", HOST);
    write(root, "src/front/helper.at", HELPER);
    write(root, "src/front/sidebar.at", SIDEBAR);
    root.join("src/front/app.at")
}

/// 主断言：带 stylekit 命名导入 + 兄弟 use 的模块经真实 VM 构建链不落
/// plan-446 C1 致命解析失败。
#[test]
fn module_level_stylekit_named_import_survives_collect_module_imports() {
    let dir = tempfile::TempDir::new().unwrap();
    let host_path = layout(dir.path());
    let path_str = host_path.to_string_lossy().to_string();
    crate::ui_module_parse_failures().lock().unwrap().clear();

    let built = crate::build_dynamic_component(HOST, Some(&path_str));
    let failures: Vec<String> = crate::ui_module_parse_failures()
        .lock()
        .unwrap()
        .clone();
    assert!(
        failures.is_empty(),
        "子模块解析不得落 plan-446 C1 失败登记（stylekit 命名导入须在解析前入域）: {failures:?}"
    );
    let err = built.as_ref().err().map(|e| e.to_string());
    assert!(
        built.is_ok(),
        "带 `use stylekit.styles: caption_text` 的子模块必须随宿主构建成功: {err:?}"
    );
}

/// 运行期面：导入配方在子模块视图里实际展开为类串（不是落空成空串/状态读）。
#[test]
fn imported_recipe_expands_in_child_module_view() {
    let dir = tempfile::TempDir::new().unwrap();
    let host_path = layout(dir.path());
    let path_str = host_path.to_string_lossy().to_string();
    crate::ui_module_parse_failures().lock().unwrap().clear();

    let mut comp =
        crate::build_dynamic_component(HOST, Some(&path_str)).expect("宿主 + 子模块构建");
    let _ = comp.view_with_debug();
    comp.drive_scheduler_to_quiescence(10_000);
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    // caption_text = "text-xs text-muted-foreground" → 编译为类型化类
    // （TextXs + TextColor(OnSurface)），两处使用点各展开一次。
    assert!(
        rendered.matches("TextColor(OnSurface)").count() >= 2,
        "子模块里的 `style: caption_text` 应展开为 stylekit 配方类型化类: {rendered:.1200}"
    );
    // chip_on = "h-7 rounded-full bg-primary/10 text-primary"：条件分支本地
    // 配方在抽取期展开，按钮不得落空成零 class。
    assert!(
        rendered.contains("RoundedFull"),
        "本地配方 if/else 分支（chip_on）应照常展开为类型化类: {rendered:.1200}"
    );
}
