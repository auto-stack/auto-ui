//! PLAN-113 T-06（026-database A2Vue 浏览器腿）：lane 产品 → 可构建站点 →
//! 真实浏览器挂载与交互断言。
//!
//! 入口与 105/106 的站点构建器同构（`generate_sfcs_with_lane_product_stores` +
//! vite 脚手架 + Playwright），证据写入 `plan-artifacts/plan-113/sites/026-database/`。
//! 判据（§6 V-04）：真实源路径派生 → SFC 含业务 marker → vite build →
//! 真 Chromium 挂载后断言 Database Studio 结构、Tab 切换、SQL 执行成功与
//! 错误路径、Commit/Rollback 在位、零 page error。排窗档：`AUTO_UI_ACE_WEB_E2E=1`。
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest().join("../..")
}

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
const pw = process.env.P113_PLAYWRIGHT || 'd:/autostack/auto-os-config/node_modules/playwright/index.mjs'
const m = await import(pathToFileURL(pw).href)
const browser = await m.chromium.launch({ headless: true })
const page = await (await browser.newContext({ viewport: { width: 1280, height: 960 } })).newPage()
const errors = []
const logs = []
page.on('pageerror', e => errors.push(e.message))
page.on('console', m => logs.push(`${m.type()}: ${m.text()}`))
await page.goto(`http://127.0.0.1:${port}`, { waitUntil: 'networkidle' })
await page.waitForTimeout(400)
if (logs.length) { console.log('CONSOLE:', logs.slice(0, 20).join('\n')) }
await page.screenshot({ path: 'shot.png' })
let failures = 0
const check = (name, ok) => { console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`); if (!ok) failures++ }
const body = async () => (await page.textContent('body')) || ''

// 初始负载 marker（026 原 UI：Database Studio / 对象树 / 表数据）。
let b = await body()
for (const marker of ['Database Studio', 'northwind.db', 'Tables', 'Views', 'Indexes', 'Data', 'Schema', 'SQL', 'customers']) {
  check(`初始 marker: ${marker}`, b.includes(marker))
}

// Tab 切换：Schema → 列结构/DDL 可见。
const schemaTab = page.getByRole('button', { name: 'Schema', exact: true })
if (await schemaTab.count()) {
  await schemaTab.click()
  await page.waitForTimeout(250)
  b = await body()
  check('Schema Tab：列定义可见', b.includes('Column') || b.includes('column'))
  check('Schema Tab：DDL 可见', b.includes('CREATE TABLE'))
} else {
  check('找到 Schema Tab', false)
}

// SQL Console：执行预设查询成功 + 非法查询错误路径。
const sqlTab = page.getByRole('button', { name: 'SQL', exact: true })
if (await sqlTab.count()) {
  await sqlTab.click()
  await page.waitForTimeout(250)
  b = await body()
  check('SQL Tab 激活', b.includes('SQL') || b.includes('Run'))
  const runBtn = page.getByRole('button', { name: /Run/i })
  if (await runBtn.count()) {
    await runBtn.first().click()
    await page.waitForTimeout(300)
    b = await body()
    check('SQL 执行成功路径（结果/耗时）', b.includes('Success') || b.includes('rows') || b.includes('耗时'))
  } else {
    check('找到 Run SQL 按钮', false)
  }
  // 非法查询错误反馈（非 existing 表）。
  const errPreset = page.getByRole('button', { name: /Error|Syntax/i })
  if (await errPreset.count()) {
    await errPreset.first().click()
    await page.waitForTimeout(150)
    const run2 = page.getByRole('button', { name: /Run/i })
    if (await run2.count()) {
      await run2.first().click()
      await page.waitForTimeout(300)
      b = await body()
      check('SQL 错误路径反馈', b.includes('Error') || b.includes('error') || b.includes('not found'))
    }
  } else {
    check('找到 Error 预设按钮', false)
  }
} else {
  check('找到 SQL Tab', false)
}

// Commit / Rollback 按钮：新增行后事务控制台可见（dirtyCount > 0 条件内）。
const dataTab = page.getByRole('button', { name: 'Data', exact: true })
if (await dataTab.count()) { await dataTab.click(); await page.waitForTimeout(200) }
const addRow = page.getByRole('button', { name: /Add Row/i })
if (await addRow.count()) {
  await addRow.first().click()
  await page.waitForTimeout(150)
  const save = page.getByRole('button', { name: /Save Record/i })
  if (await save.count()) {
    await save.click()
    await page.waitForTimeout(250)
    b = await body()
    check('Commit 按钮在位', b.includes('Commit'))
    check('Rollback 按钮在位', b.includes('Rollback'))
    const commit = page.getByRole('button', { name: /Commit/i })
    if (await commit.count()) {
      await commit.first().click()
      await page.waitForTimeout(200)
      b = await body()
      check('Commit 事务反馈', b.includes('COMMITTED') || b.includes('committed'))
    } else {
      check('找到 Commit 按钮', false)
    }
  } else {
    check('找到 Save Record 按钮', false)
  }
} else {
  check('找到 Add Row 按钮', false)
}

check('无页面异常', errors.length === 0)
if (errors.length) console.log('ERRORS:', errors.slice(0, 5))
await browser.close(); server.close()
console.log(`RESULT ${JSON.stringify({ failures })}`)
process.exit(failures === 0 ? 0 : 1)
"#;

fn node_modules_dir() -> &'static str {
    "D:/autostack/auto-os-config/node_modules"
}

fn mklink_junction(link: &Path, target: &Path) -> Result<(), String> {
    let link = link.to_string_lossy().replace('/', "\\");
    let target = target.to_string_lossy().replace('/', "\\");
    let output = Command::new("cmd")
        .args(["/c", "mklink", "/J", &link, &target])
        .output()
        .map_err(|e| format!("mklink spawn: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[test]
fn p113_026_vue_browser_evidence() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set AUTO_UI_ACE_WEB_E2E=1; browser tier)");
        return;
    }
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product;
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;

    let root = repo_root();
    let app_dir = root.join("examples/ui/026-database");
    let src = std::fs::read_to_string(app_dir.join("src/front/app.at")).expect("read 026 app.at");
    // A2Vue lane 用**原始源**（use 导入行必须保留——lane 侧解析导入池；
    // `prepare_component_source` 会把 use 行空白化，那是 AV 组件面单源投影
    // 的职责，A2Vue 官方链同样传原始内容）。
    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &src,
        app_dir.join("src/front/app.at").to_string_lossy().as_ref(),
        "./a2x_rt.mjs",
        "../a2x_rt.mjs",
    )
    .expect("026 lane product derives");

    // lane 产品断言：模板含业务 marker + 条件样式三元。
    let sfc = multi.widgets.iter().find(|w| w.name == "App").expect("App SFC");
    for tok in [
        "Database Studio", "Data", "Schema", "SQL", "customers", "products", "orders",
        "RunSql", "CommitTx", "RollbackTx", "v-for", ":class", "PaginationControl",
    ] {
        assert!(sfc.sfc.contains(tok), "generated App.vue missing `{tok}`");
    }
    assert!(stores.is_empty(), "026 has no store units: {stores:?}");

    // 站点脚手架。
    let out = Path::new("D:/autostack/plan-artifacts/plan-113/sites/026-database");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(out.join("components")).expect("mkdir components");
    std::fs::write(out.join("a2x_rt.mjs"), &multi.runtime_mjs).expect("write runtime");
    for w in &multi.widgets {
        let body = w.sfc.replace("@/components/", "./components/");
        if w.name == "App" {
            std::fs::write(out.join("App.vue"), body).expect("write App.vue");
        } else {
            std::fs::write(out.join("components").join(format!("{}.vue", w.name)), body)
                .unwrap_or_else(|e| panic!("write components/{}.vue: {e}", w.name));
        }
    }
    // 包组件 SFC（PaginationControl/TreeView/TreeIcon）走与 App 同一条 lane
    // 产品道（`generate_sfcs_with_lane_product`——官方 ACE-VUE 组件腿同入口），
    // 落 src/components/{W}.vue 位；rt href=`../a2x_rt.mjs`（组件位相对引用）。
    for (rel, widget) in [
        ("packages/pagination/pagination_control.at", "PaginationControl"),
        ("packages/tree-view/tree_view.at", "TreeView"),
        ("packages/tree-view/tree_icon.at", "TreeIcon"),
    ] {
        let path = root.join(rel);
        let source = std::fs::read_to_string(&path).expect("read package widget source");
        let multi = generate_sfcs_with_lane_product(
            &source,
            &path.to_string_lossy(),
            "../a2x_rt.mjs",
            "../a2x_rt.mjs",
        )
        .unwrap_or_else(|e| panic!("lane product for {rel}: {e}"));
        let sfc = multi
            .widgets
            .iter()
            .find(|w| w.name == widget)
            .unwrap_or_else(|| panic!("{rel} declares no {widget} widget: {:?}", multi.widgets.iter().map(|w| w.name.clone()).collect::<Vec<_>>()))
            .sfc
            .clone();
        std::fs::write(out.join("components").join(format!("{widget}.vue")), sfc)
            .unwrap_or_else(|e| panic!("write components/{widget}.vue: {e}"));
    }
    std::fs::write(
        out.join("index.html"),
        "<!doctype html>\n<html lang=\"en\">\n  <head><meta charset=\"utf-8\" /><title>database</title></head>\n  <body><div id=\"app\"></div><script type=\"module\" src=\"/main.js\"></script></body>\n</html>\n",
    )
    .expect("write index.html");
    std::fs::write(
        out.join("main.js"),
        "import { createApp } from 'vue'\nimport App from './App.vue'\ncreateApp(App).mount('#app')\n",
    )
    .expect("write main.js");
    std::fs::write(
        out.join("vite.config.js"),
        "import { defineConfig } from 'vite'\nimport vue from '@vitejs/plugin-vue'\nimport { fileURLToPath, URL } from 'node:url'\nexport default defineConfig({ plugins: [vue()], resolve: { alias: { '@': fileURLToPath(new URL('./', import.meta.url)), vue: 'vue/dist/vue.esm-bundler.js' } } })\n",
    )
    .expect("write vite.config.js");
    std::fs::write(out.join("verify.mjs"), VERIFY_MJS).expect("write verify.mjs");
    let nm = out.join("node_modules");
    if !nm.exists() {
        mklink_junction(&nm, Path::new(node_modules_dir())).expect("node_modules junction");
    }

    // vite build。
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".into());
    let build = Command::new(&node)
        .args(["node_modules/vite/bin/vite.js", "build"])
        .current_dir(out)
        .output()
        .expect("vite build spawn");
    let btail = String::from_utf8_lossy(&build.stderr);
    assert!(
        build.status.success(),
        "vite build failed: {}",
        btail.lines().take(8).collect::<Vec<_>>().join(" | ")
    );

    // Playwright 浏览器腿。
    let node_version = Command::new(&node)
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let out_res = Command::new(&node)
        .arg("verify.mjs")
        .current_dir(out)
        .output()
        .expect("verify.mjs spawn");
    let stdout = String::from_utf8_lossy(&out_res.stdout).to_string();
    eprintln!("node {node_version}");
    eprintln!("{stdout}");
    assert!(
        out_res.status.success() && stdout.contains("RESULT {\"failures\":0}"),
        "026 browser evidence failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&out_res.stderr)
    );
}
