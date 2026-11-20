//! PLAN-105 T-02（U03）：把「真实 app 源 → 可构建 Vue 站点」落成**可复现构建器**。
//!
//! 产物写入 `plan-artifacts/plan-105/sites/<app>/`：入口 SFC、子组件 SFC、store
//! composable、运行时 mjs、vite 脚手架（index.html/main.js/vite.config.js）。
//! 之后由 node + vite + Playwright 做真实浏览器验收（见各站点的 verify_*.mjs）。
use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;
use std::path::{Path, PathBuf};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui")
}

/// 去掉重复的 `store <Name> { … }` 块（按名保留首个；各单元 prep 会各内联一次）。
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

/// 去掉重复的顶层 `function <name>(…) { … }`（各单元 prep 会把 `use` 的 helper
/// 各内联一次；SFC 里重复声明会让 babel/TS 报重复定义）。
fn dedupe_functions(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut drops: Vec<(usize, usize)> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    let mut i = 0usize;
    while i + 9 < bytes.len() {
        let at_line_start = i == 0 || bytes[i - 1] == 10;
        if at_line_start && &bytes[i..i + 9] == b"function " {
            let rest = &src[i + 9..];
            let name_end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let name = rest[..name_end].to_string();
            if let Some(rel) = rest[name_end..].find('{') {
                let open = i + 9 + name_end + rel;
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
fn build_site(app: &str, units: &[&str]) -> Result<PathBuf, String> {
    std::env::set_var("P105_MEM_BACKEND", "1");
    let root = examples().join(app);
    let mut joined = String::new();
    for rel in units {
        let src = std::fs::read_to_string(root.join(rel))
            .map_err(|e| format!("{app}/{rel}: {e}"))?;
        let prepped = prepare_component_source(&root, &src)
            .map(|p| p.source)
            .unwrap_or(src);
        joined.push_str(&prepped);
        joined.push('\n');
    }
    let merged = dedupe_store_blocks(&joined);
    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &merged,
        "src/front/app.at",
        "./a2x_rt.mjs",
        "./a2x_rt.mjs",
    )
    .map_err(|e| format!("{app}: {e}"))?;
    let out = Path::new("D:/autostack/plan-artifacts/plan-105/sites").join(app);
    std::fs::create_dir_all(out.join("components")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out.join("stores")).map_err(|e| e.to_string())?;
    std::fs::write(out.join("a2x_rt.mjs"), &multi.runtime_mjs).map_err(|e| e.to_string())?;
    for s in &stores {
        std::fs::write(out.join("stores").join(format!("use{}.ts", s.name)), &s.code)
            .map_err(|e| e.to_string())?;
    }
    for w in &multi.widgets {
        let sfc = w.sfc.replace("@/components/", "./components/");
        let target = if w.name == "App" {
            out.join("App.vue")
        } else {
            out.join("components").join(format!("{}.vue", w.name))
        };
        std::fs::write(target, sfc).map_err(|e| e.to_string())?;
    }
    // vite 脚手架（node_modules 由外部 junction 提供）。
    std::fs::write(
        out.join("index.html"),
        "<!doctype html>\n<html lang=\"en\">\n  <head><meta charset=\"utf-8\" /><title>app</title></head>\n  <body><div id=\"app\"></div><script type=\"module\" src=\"/main.js\"></script></body>\n</html>\n",
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        out.join("main.js"),
        "import { createApp } from 'vue'\nimport App from './App.vue'\ncreateApp(App).mount('#app')\n",
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        out.join("vite.config.js"),
        "import { defineConfig } from 'vite'\nimport vue from '@vitejs/plugin-vue'\nimport { fileURLToPath, URL } from 'node:url'\nexport default defineConfig({ plugins: [vue()], resolve: { alias: { '@': fileURLToPath(new URL('./', import.meta.url)), vue: 'vue/dist/vue.esm-bundler.js' } } })\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

#[test]
fn p105_build_u03_sites() {
    let apps: Vec<(&str, Vec<&str>)> = vec![
        ("013-todo", vec!["src/front/app.at", "src/front/todo_list.at"]),
        ("015-notes", vec!["src/front/app.at", "src/front/editor.at", "src/front/sidebar.at"]),
        ("016-calendar", vec!["src/front/app.at"]),
        ("022-kanban", vec!["src/front/app.at"]),
    ];
    let mut built = Vec::new();
    let mut failed = Vec::new();
    for (app, units) in &apps {
        match build_site(app, units) {
            Ok(p) => { println!("SITE {app}: OK {}", p.display()); built.push(*app); }
            Err(e) => { println!("SITE {app}: ERR {e}"); failed.push((*app, e)); }
        }
    }
    println!("SITES built={:?} failed={:?}", built, failed.iter().map(|(a, _)| *a).collect::<Vec<_>>());
}
