// PLAN-104（U02 余例 A2Vue 浏览器档）：**通用 lane 产物挂载 rig**。
//
// 输入目录含 `component.sfc` + `a2x_rt.mjs`（`generate_sfcs_with_lane_product`
// 原样落盘，零注入）；`<expected>` 为分号分隔的期望子串（必须全部出现在
// `#app` 文本中）。判据：①SFC 解析/编译无错；②页面挂载成功；③全部期望子串
// 出现；④零 page error、零 console error。
//
// 用法：node test_p104_generic_mount.mjs <spec.json>
//   spec = { apps: [ { label, dir, expected: [..] } ] }（单浏览器会话跑全部应用）
import fs from 'fs';
import path from 'path';
import http from 'http';
import { pathToFileURL } from 'url';

const COMPILER_CANDIDATES = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
  'd:/autostack/auto-os-config/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
  'd:/autostack/auto-down/autodown/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js',
];
const VUE_CANDIDATES = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/vue/dist/vue.esm-browser.js',
  'd:/autostack/auto-os-config/node_modules/vue/dist/vue.esm-browser.js',
  'd:/autostack/auto-down/autodown/node_modules/vue/dist/vue.esm-browser.js',
];
const PLAYWRIGHT_CANDIDATES = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
  'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
  'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
  'playwright',
];
const ESBUILD_CANDIDATES = [
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/esbuild/lib/main.js',
  'd:/autostack/auto-os-config/node_modules/esbuild/lib/main.js',
  'd:/autostack/auto-down/autodown/node_modules/esbuild/lib/main.js',
];

function firstExisting(candidates) {
  for (const c of candidates) {
    if ((c.includes(':') || c.startsWith('/')) && fs.existsSync(c)) return c;
  }
  return candidates[candidates.length - 1];
}

async function resolvePlaywright() {
  for (const p of PLAYWRIGHT_CANDIDATES) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      const mod = await import(target);
      return mod.chromium;
    } catch (_) {}
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

async function main() {
  const specPath = process.argv[2];
  if (!specPath || !fs.existsSync(specPath)) {
    console.error('usage: node test_p104_generic_mount.mjs <spec.json>');
    process.exit(2);
  }
  const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));
  const apps = spec.apps || [];
  if (!apps.length) {
    console.error('spec has no apps');
    process.exit(2);
  }
  const compiler = await import(pathToFileURL(firstExisting(COMPILER_CANDIDATES)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const vuePath = firstExisting(VUE_CANDIDATES);
  const vueSource = fs.readFileSync(vuePath);
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};

  const chromium = await resolvePlaywright();
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
  } catch (_) {
    try {
      browser = await chromium.launch({ headless: true, channel: 'msedge' });
    } catch (_) {
      browser = await chromium.launch({ headless: true, channel: 'chrome' });
    }
  }

  const timings = [];
  try {
    for (const app of apps) {
      const started = Date.now();
      const dir = app.dir;
      if (!dir || !fs.existsSync(path.join(dir, 'component.sfc')) || !fs.existsSync(path.join(dir, 'a2x_rt.mjs'))) {
        console.error(`[${app.label}] missing component.sfc/a2x_rt.mjs in ${dir}`);
        process.exit(1);
      }
      const source = fs.readFileSync(path.join(dir, 'component.sfc'), 'utf8');
      const { descriptor, errors } = parse(source, { filename: 'component.sfc' });
      if (errors && errors.length) {
        console.error(`[${app.label}] SFC parse errors: ${JSON.stringify(errors)}`);
        process.exit(1);
      }
      const compiled = compileScript(descriptor, { id: `p104-${app.label}`, inlineTemplate: true });
      const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
      fs.writeFileSync(path.join(dir, 'component.mjs'), js.replace(/from ["']vue["']/g, 'from "/vue.js"'));
      fs.writeFileSync(
        path.join(dir, 'index.html'),
        `<!DOCTYPE html>
<html><body>
<div id="app"></div>
<script type="module">
  import { createApp } from '/vue.js';
  import App from '/component.mjs';
  const app = createApp(App);
  app.mount('#app');
</script>
</body></html>
`
      );

      const server = http.createServer((req, res) => {
        const url = (req.url || '/').split('?')[0];
        if (url === '/vue.js') {
          res.writeHead(200, { 'Content-Type': 'text/javascript' });
          res.end(vueSource);
          return;
        }
        const name = url === '/' ? '/index.html' : url;
        const file = path.resolve(path.join(dir, name));
        if (!file.startsWith(path.resolve(dir)) || !fs.existsSync(file)) {
          res.writeHead(404);
          res.end('not found');
          return;
        }
        const type = name.endsWith('.mjs') || name.endsWith('.js') ? 'text/javascript' : 'text/html';
        res.writeHead(200, { 'Content-Type': type });
        res.end(fs.readFileSync(file));
      });
      await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
      const port = server.address().port;

      const page = await (await browser.newContext()).newPage();
      const pageErrors = [];
      page.on('pageerror', (err) => pageErrors.push(err.message));
      const consoleErrors = [];
      page.on('console', (m) => {
        if (m.type() === 'error') consoleErrors.push(m.text());
      });
      const functionalConsoleErrors = (texts) => texts.filter((t) => !/Failed to load resource/i.test(t));
      await page.goto(`http://127.0.0.1:${port}/index.html`);
      await page.waitForSelector('#app', { timeout: 8000 });
      await page.waitForTimeout(250);
      const body = await page.evaluate(() => document.querySelector('#app')?.innerText || '');
      const expected = app.expected || [];
      const missing = expected.filter((e) => !body.includes(e));
      await page.close();
      server.close();
      const ms = Date.now() - started;
      timings.push({ label: app.label, ms });
      if (pageErrors.length || functionalConsoleErrors(consoleErrors).length) {
        console.error(`[${app.label}] page errors: ${JSON.stringify(pageErrors)} / console: ${JSON.stringify(functionalConsoleErrors(consoleErrors))}`);
        process.exit(1);
      }
      if (missing.length) {
        console.error(`[${app.label}] missing expected text: ${JSON.stringify(missing)}\n--- body ---\n${body}`);
        process.exit(1);
      }
      for (const e of expected) console.log(`[+] ${app.label}: ${e}`);
      console.log(`[t] ${app.label} ${ms}ms`);
    }
  } finally {
    await browser.close();
  }
  console.log(`P104_GENERIC_MOUNT_OK apps=${apps.length} timings=${JSON.stringify(timings)}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});