//! PLAN-106 T-06（014-weather A2Vue 浏览器腿）：lane 产品 → 可构建站点 →
//! 真实浏览器挂载与交互断言。
//!
//! 入口与 105 的站点构建器同构（`generate_sfcs_with_lane_product_stores` +
//! vite 脚手架 + Playwright），证据写入 `plan-artifacts/plan-106/sites/014-weather/`。
//! 判据（§6 V-04）：真实源路径派生 → SFC 含业务 marker → vite build →
//! 真 Chromium 挂载后断言 北京/上海/24小时/5日/刷新 可见、城市切换、
//! 预报 Tab 切换、主题切换、零 page error。排窗档：`AUTO_UI_ACE_WEB_E2E=1`。
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
const pw = process.env.P106_PLAYWRIGHT || 'd:/autostack/auto-os-config/node_modules/playwright/index.mjs'
const m = await import(pathToFileURL(pw).href)
const browser = await m.chromium.launch({ headless: true })
const page = await (await browser.newContext({ viewport: { width: 1280, height: 960 } })).newPage()
const errors = []
page.on('pageerror', e => errors.push(e.message))
await page.goto(`http://127.0.0.1:${port}`, { waitUntil: 'networkidle' })
await page.waitForTimeout(700)
let failures = 0
const check = (name, ok) => { console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`); if (!ok) failures++ }
const body = async () => (await page.textContent('body')) || ''

// 初始负载 marker（weather_data.at 静态样本的北京默认）。
let b = await body()
for (const marker of ['北京', '24小时', '5日', '刷新', '当前详情', '湿度', '体感', '演示样本']) {
  check(`初始 marker: ${marker}`, b.includes(marker))
}
// 城市切换：点上海 → 城市名切到上海（原样本合同：SelectCity 首段按
// ASCII id 更新 city_zh；第二段数据分支比较中文 id，按钮传 ASCII，故
// 英雄数据保持 beijing 默认——这是原 pac.at 的冻结行为，vm_smoke.py
// 也只断言 city_id/city_zh，不断言数据字段）。
const sh = page.getByRole('button', { name: '上海', exact: true })
if (await sh.count()) {
  await sh.click()
  await page.waitForTimeout(500)
  b = await body()
  check('城市切换：上海可见', b.includes('上海'))
  // 原样本：city_zh 已切、英雄仍 beijing 默认（23°）——两端一致的固定负载。
  check('原样行为：英雄保持 beijing 默认（23°）', b.includes('23°'))
} else {
  check('找到上海按钮', false)
}
// 预报 Tab 切换：点 5日 → 今天/周二 日卡可见。
const tab = page.getByRole('button', { name: '5日', exact: true })
if (await tab.count()) {
  await tab.click()
  await page.waitForTimeout(400)
  b = await body()
  check('Tab 5日：今天', b.includes('今天'))
  check('Tab 5日：周二', b.includes('周二'))
} else {
  check('找到 5日 Tab', false)
}
// 主题切换按钮存在（☀️/🌙）。
check('主题按钮存在', (await page.getByRole('button', { name: '☀️', exact: true }).count()) +
  (await page.getByRole('button', { name: '🌙', exact: true }).count()) > 0)
check('无页面异常', errors.length === 0)
if (errors.length) console.log('ERRORS:', errors.slice(0, 3))
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
fn p106_014_vue_browser_evidence() {
    if std::env::var("AUTO_UI_ACE_WEB_E2E").ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set AUTO_UI_ACE_WEB_E2E=1; browser tier)");
        return;
    }
    use auto_ui_runtime::ui::ace_source_prep::prepare_component_source;
    use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;

    let root = repo_root();
    let app_dir = root.join("examples/ui/014-weather");
    let src = std::fs::read_to_string(app_dir.join("src/front/app.at")).expect("read 014 app.at");
    let prepped = prepare_component_source(&app_dir, &src)
        .map(|p| p.source)
        .unwrap_or(src.clone());
    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &prepped,
        "src/front/app.at",
        "./a2x_rt.mjs",
        "./a2x_rt.mjs",
    )
    .expect("014 lane product derives");

    // lane 产品断言：模板含业务 marker + 条件样式三元。
    let sfc = multi.widgets.iter().find(|w| w.name == "App").expect("App SFC");
    for tok in ["北京", "上海", "24小时", "5日", "刷新", "v-for", ":class", "SelectCity"] {
        assert!(
            sfc.sfc.contains(tok),
            "generated App.vue missing `{tok}`"
        );
    }
    assert!(sfc.sfc.contains("? '"), "conditional style ternary emitted");
    assert!(stores.is_empty(), "014 has no store units: {stores:?}");

    // 站点脚手架。
    let out = Path::new("D:/autostack/plan-artifacts/plan-106/sites/014-weather");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(out.join("components")).expect("mkdir components");
    std::fs::write(out.join("a2x_rt.mjs"), &multi.runtime_mjs).expect("write runtime");
    std::fs::write(out.join("App.vue"), sfc.sfc.replace("@/components/", "./components/"))
        .expect("write App.vue");
    std::fs::write(
        out.join("index.html"),
        "<!doctype html>\n<html lang=\"en\">\n  <head><meta charset=\"utf-8\" /><title>weather</title></head>\n  <body><div id=\"app\"></div><script type=\"module\" src=\"/main.js\"></script></body>\n</html>\n",
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
        "014 browser evidence failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&out_res.stderr)
    );
}
