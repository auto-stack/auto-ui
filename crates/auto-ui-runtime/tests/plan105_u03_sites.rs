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
/// 把 `use <unit>: f1, f2` 引用的单元里的 `pub fn` 补齐到合并源（prep 只内联
/// 可通过组件面探针的那部分；工具函数常含 `var xs = []`/record 字面量而缺席，
/// lane 的 SFC 仍会引用它们）。
///
/// 取 `store <Name> { … }` 整块（大括号配平）。
fn extract_store_block_text(src: &str, name: &str) -> Option<String> {
    let needle = format!("store {name}");
    let pos = src.find(&needle)?;
    let start = src[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let bytes = src.as_bytes();
    let open = start + src[start..].find('{')?;
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
    Some(src[start..j].to_string())
}
fn append_used_module_fns(root: &Path, app_src: &str, merged: &mut String) {
    let mut wanted: Vec<String> = Vec::new();
    for line in app_src.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("use ") else { continue };
        let Some((module, items)) = rest.split_once(':') else { continue };
        let module = module.trim();
        if module.contains('.') {
            continue; // 依赖包（themes/stylekit/datetime）不在此处补
        }
        for item in items.split(',') {
            let name = item.trim();
            if name.is_empty() {
                continue;
            }
            wanted.push(name.to_string());
        }
    }
    if wanted.is_empty() {
        return;
    }
    let dir = root.join("src/front");
    let mut extra = String::new();
    // 先按 `use` 显式引用补，再**全量**补同目录单元的顶层 fn（工具函数内部还会
    // 调用 `weekday_of`/`days_in_month` 等未在 `use` 里点名的兄弟函数）。
    let mut names: Vec<String> = wanted.clone();
    let mut units: Vec<(std::path::PathBuf, String)> = Vec::new();
    for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("at") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("app.at") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("pub fn ").or_else(|| t.strip_prefix("fn ")) {
                if let Some(paren) = rest.find('(') {
                    names.push(rest[..paren].trim().to_string());
                }
            }
        }
        units.push((path, text));
    }
    for (_path, text) in &units {
        for name in &names {
            if merged.contains(&format!("fn {name}(")) || extra.contains(&format!("fn {name}(")) {
                continue;
            }
            if let Some(block) = extract_fn_block(text, name) {
                extra.push_str(&block);
                extra.push('\n');
            }
        }
    }
    merged.push('\n');
    merged.push_str(&extra);
}

/// 取 `pub fn <name>(…) { … }`（大括号配平）整块。
fn extract_fn_block(src: &str, name: &str) -> Option<String> {
    let needle = format!("fn {name}(");
    let pos = src.find(&needle)?;
    let start = src[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let bytes = src.as_bytes();
    let open = start + src[start..].find('{')?;
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
    Some(src[start..j].to_string())
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
    let mut merged = dedupe_store_blocks(&joined);
    append_used_module_fns(&root, &std::fs::read_to_string(root.join("src/front/app.at")).unwrap_or_default(), &mut merged);
    // prep 的 store 内联要过组件面探针；大 store（015 的 NotesStore）可能被跳过——
    // 合并源里没有 store 声明时，把同目录单元里的 store 块补进来（保持单份）。
    if !merged.lines().any(|l| l.starts_with("store ")) {
        let mut found: Option<String> = None;
        for entry in std::fs::read_dir(root.join("src/front"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let Some(line) = text.lines().find(|l| l.starts_with("store ")) else { continue };
            let name = line
                .trim_start_matches("store ")
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()
                .unwrap_or("")
                .to_string();
            if let Some(block) = extract_store_block_text(&text, &name) {
                found = Some(block);
                break;
            }
        }
        if let Some(block) = found {
            merged.push('\n');
            merged.push_str(&block);
        }
    }
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
        &main_js_for(&root, &merged),
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
    let apps = u03_apps();
    let mut built = Vec::new();
    let mut failed = Vec::new();
    for (app, units) in &apps {
        match build_site(app, units) {
            Ok(p) => { println!("SITE {app}: OK {}", p.display()); built.push(*app); }
            Err(e) => { println!("SITE {app}: ERR {e}"); failed.push((*app, e)); }
        }
    }
    println!("SITES built={:?} failed={:?}", built, failed.iter().map(|(a, _)| *a).collect::<Vec<_>>());
    assert!(failed.is_empty(), "站点构建失败: {failed:?}");
}

/// `main.js`：应用带 `routes { "…" -> use <page> }` 时装配 vue-router（单页站点
/// 也要把板块/页面挂到 `/`），否则直接 mount App。
fn main_js_for(root: &Path, merged: &str) -> String {
    let mut routes: Vec<(String, String)> = Vec::new();
    if let Some(start) = merged.find("routes {") {
        for line in merged[start..].lines().skip(1) {
            let t = line.trim();
            if t.starts_with('}') {
                break;
            }
            let Some((path_part, target)) = t.split_once("->") else { continue };
            let path = path_part.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            let target = target.trim().trim_start_matches("use ").trim().to_string();
            if !path.is_empty() && !target.is_empty() {
                routes.push((path, target));
            }
        }
    }
    if routes.is_empty() {
        return "import { createApp } from 'vue'\nimport App from './App.vue'\ncreateApp(App).mount('#app')\n".to_string();
    }
    let mut imports = String::new();
    let mut table = String::new();
    for (i, (_path, target)) in routes.iter().enumerate() {
        let file = root
            .join("src/front/pages")
            .join(format!("{target}.at"));
        let comp = if file.exists() {
            format!("Page{i}")
        } else {
            format!("{target}")
        };
        imports.push_str(&format!("import {comp} from './components/{target}.vue'\n"));
        table.push_str(&format!("  {{ path: '{_path}', component: {comp} }},\n", _path = routes[i].0));
    }
    format!(
        "import {{ createApp }} from 'vue'\nimport {{ createRouter, createWebHashHistory }} from 'vue-router'\nimport App from './App.vue'\n{imports}const router = createRouter({{ history: createWebHashHistory(), routes: [\n{table}] }})\ncreateApp(App).use(router).mount('#app')\n"
    )
}

fn u03_apps() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("013-todo", vec!["src/front/app.at", "src/front/todo_list.at"]),
        ("015-notes", vec!["src/front/app.at", "src/front/editor.at", "src/front/sidebar.at"]),
        ("016-calendar", vec!["src/front/app.at"]),
        ("022-kanban", vec!["src/front/app.at", "src/front/pages/board.at"]),
    ]
}

/// 浏览器验收脚本（静态服务 + Playwright；与上游 app 用例同口径的最小集）。
const VERIFY_MJS: &str = r#"import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'
const root = path.resolve('dist')
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' }
const server = http.createServer((req, res) => {
  const p = req.url === '/' ? '/index.html' : req.url.split('?')[0]
  const file = path.join(root, p)
  if (!fs.existsSync(file)) { res.writeHead(404); res.end(); return }
  res.writeHead(200, { 'content-type': types[path.extname(file)] || 'application/octet-stream' })
  res.end(fs.readFileSync(file))
})
await new Promise(r => server.listen(0, '127.0.0.1', r))
const port = server.address().port
const m = await import(pathToFileURL(process.env.P105_PLAYWRIGHT || 'd:/autostack/auto-os-config/node_modules/playwright/index.mjs').href)
const browser = await m.chromium.launch({ headless: true })
const page = await (await browser.newContext({ viewport: { width: 1280, height: 960 } })).newPage()
const errors = []
page.on('pageerror', e => errors.push(e.message))
await page.goto(`http://127.0.0.1:${port}`, { waitUntil: 'networkidle' })
await page.waitForTimeout(700)
let failures = 0
const check = (name, ok) => { console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`); if (!ok) failures++ }
const body = (await page.textContent('body')) || ''
check('渲染了文本内容（body 非空）', body.trim().length > 10)
check('有可交互元素（button/input）', (await page.locator('button, input').count()) > 0)
check('无页面异常', errors.length === 0)
if (errors.length) console.log('ERRORS:', errors.slice(0, 3))
await browser.close(); server.close()
console.log(`RESULT ${JSON.stringify({ failures })}`)
process.exit(failures === 0 ? 0 : 1)
"#;

/// 依赖已经备好的 app → 需要额外 junction 的包。
fn extra_deps(app: &str) -> &'static [(&'static str, &'static str)] {
    match app {
        "015-notes" => &[(
            "lucide-vue-next",
            "D:/autostack/auto-lang/packages/auto-forge-ui/node_modules/lucide-vue-next",
        )],
        "022-kanban" => &[(
            "vue-router",
            "D:/autostack/auto-lang/examples/vue-gallery/node_modules/vue-router",
        )],
        _ => &[],
    }
}

fn node_modules_dir() -> &'static str {
    "D:/autostack/auto-os-config/node_modules"
}

/// 写验收脚本 + 建 `node_modules` junction（已存在则跳过；`mklink /J` 免管理员）。
fn scaffold_site(out: &std::path::Path, app: &str) -> Result<(), String> {
    std::fs::write(out.join("verify.mjs"), VERIFY_MJS).map_err(|e| e.to_string())?;
    let nm = out.join("node_modules");
    if !nm.exists() {
        let status = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&nm)
            .arg(node_modules_dir())
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("{app}: node_modules junction 创建失败"));
        }
    }
    for (name, target) in extra_deps(app) {
        let link = nm.join(name);
        if !link.exists() {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "mklink", "/J"])
                .arg(&link)
                .arg(target)
                .status();
        }
    }
    Ok(())
}

/// 浏览器腿的**可跑门**：构建四 app 站点 → `vite build` → `node verify.mjs`。
/// 默认忽略（依赖本机 node_modules/Playwright）；手动跑：
/// `cargo test -p auto-ui-runtime --features ace-bridge --test plan105_u03_sites -- --ignored --nocapture`
#[test]
#[ignore = "浏览器门：需要本机 node_modules junction + Playwright"]
fn p105_u03_browser_gate() {
    let apps = u03_apps();
    let mut failures: Vec<(String, String)> = Vec::new();
    for (app, units) in &apps {
        let out = match build_site(app, units) {
            Ok(p) => p,
            Err(e) => { failures.push((app.to_string(), format!("build_site: {e}"))); continue; }
        };
        if let Err(e) = scaffold_site(&out, app) {
            failures.push((app.to_string(), e));
            continue;
        }
        let build = std::process::Command::new("node")
            .current_dir(&out)
            .args(["node_modules/vite/bin/vite.js", "build"])
            .output()
            .map_err(|e| e.to_string());
        match build {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let tail = String::from_utf8_lossy(&o.stderr);
                failures.push((app.to_string(), format!("vite build: {}", tail.lines().take(6).collect::<Vec<_>>().join(" | "))));
                continue;
            }
            Err(e) => { failures.push((app.to_string(), format!("vite spawn: {e}"))); continue; }
        }
        let run = std::process::Command::new("node")
            .current_dir(&out)
            .arg("verify.mjs")
            .output()
            .map_err(|e| e.to_string());
        match run {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                println!("--- {app} ---\n{stdout}");
                if !stdout.contains("\"failures\":0") {
                    failures.push((app.to_string(), "浏览器断言失败".to_string()));
                }
            }
            Err(e) => failures.push((app.to_string(), format!("verify spawn: {e}"))),
        }
    }
    assert!(failures.is_empty(), "浏览器门失败: {failures:#?}");
}

