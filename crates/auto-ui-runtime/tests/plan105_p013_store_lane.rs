//! PLAN-105 T-02（U03 013）：**store 驱动的 A2Vue 产品**——`store X { … }` 生成
//! Vue composable，widget SFC 拿到 `store` 绑定（真实挂载的页面因此可渲染列表/计数）。
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;
use std::path::Path;

fn dedupe_store_blocks(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut drops: Vec<(usize, usize)> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    let mut i = 0usize;
    while i + 6 < bytes.len() {
        if &bytes[i..i + 6] == b"store " && (i == 0 || bytes[i - 1] == 10) {
            let rest = &src[i + 6..];
            let name_end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let name = rest[..name_end].to_string();
            if let Some(rel) = rest[name_end..].find('{') {
                let open = i + 6 + name_end + rel;
                let mut depth = 0i32;
                let mut j = open;
                while j < bytes.len() {
                    match bytes[j] {
                        123 => depth += 1,
                        125 => {
                            depth -= 1;
                            if depth == 0 {
                                j += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if seen.contains(&name) {
                    drops.push((i, j));
                } else {
                    seen.insert(name);
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    let mut out = String::with_capacity(src.len());
    let mut cursor = 0usize;
    for (a, b) in drops {
        out.push_str(&src[cursor..a]);
        cursor = b;
    }
    out.push_str(&src[cursor..]);
    out
}

#[test]
fn p105_p013_store_composable_and_binding() {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/013-todo");
    let prep = |rel: &str| {
        let src = std::fs::read_to_string(root.join(rel)).unwrap();
        prepare_component_source(&root, &src).map(|p| p.source).unwrap_or(src)
    };
    // 每个单元各自 prep（store 随 `use todo_store:` 内联一次），合并后按名去重。
    let joined = format!("{}\n{}\n", prep("src/front/app.at"), prep("src/front/todo_list.at"));
    let merged = dedupe_store_blocks(&joined);
    assert_eq!(merged.matches("store TodoStore").count(), 1, "store 块恰好一份");

    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &merged,
        "src/front/app.at",
        "./a2x_rt.mjs",
        "./a2x_rt.mjs",
    )
    .expect("013 store 驱动产品");

    assert_eq!(stores.len(), 1, "一个 store composable");
    let store = &stores[0];
    assert_eq!(store.name, "TodoStore");
    assert!(store.code.contains("export function useTodoStore"), "composable 出口");
    assert!(store.code.contains("export function createTodoStore"), "工厂出口");

    let app = multi.widgets.iter().find(|w| w.name == "App").expect("App SFC");
    assert!(
        app.sfc.contains("import { useTodoStore }"),
        "App 导入 store composable: {}",
        app.sfc.lines().take(8).collect::<Vec<_>>().join(" | ")
    );
    assert!(app.sfc.contains("const store = reactive(useTodoStore())"), "App 绑定 store");
    let list = multi.widgets.iter().find(|w| w.name == "TodoList").expect("TodoList SFC");
    assert!(list.sfc.contains("useTodoStore"), "子组件共享同一 store");
    assert!(list.sfc.contains("v-for"), "子组件列表渲染");
}
