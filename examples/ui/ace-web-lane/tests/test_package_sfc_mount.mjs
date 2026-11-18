// PLAN-980 ④-A #3：**AGE view → Vue SFC 的真实挂载档**。
//
// 输入：<dir>/component.sfc（Rust 侧 generate_package_sfc 产出）。
// 步骤：@vue/compiler-sfc 编译（inlineTemplate）→ 本地 HTTP 提供页面（file:// 下
// ESM 跨源模块会被 Chromium 拒绝）→ playwright 点击按钮 → 经组件
// defineExpose.__state() 读真实响应式状态（0 → 1 → 0，再两次 + → 2）。
//
// 用法：node test_package_sfc_mount.mjs <dir>
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
  const dir = process.argv[2];
  if (!dir || !fs.existsSync(path.join(dir, 'component.sfc'))) {
    console.error('usage: node test_package_sfc_mount.mjs <dir-with-component.sfc>');
    process.exit(2);
  }
  const compiler = await import(pathToFileURL(firstExisting(COMPILER_CANDIDATES)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const vuePath = firstExisting(VUE_CANDIDATES);
  const source = fs.readFileSync(path.join(dir, 'component.sfc'), 'utf8');
  const { descriptor, errors } = parse(source, { filename: 'component.sfc' });
  if (errors && errors.length) {
    console.error('SFC parse errors:', errors);
    process.exit(1);
  }
  const compiled = compileScript(descriptor, { id: 'age-sfc', inlineTemplate: true });
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    compiled.content.replace(/from ["']vue["']/g, 'from "/vue.js"')
  );
  fs.writeFileSync(
    path.join(dir, 'index.html'),
    `<!DOCTYPE html>
<html><body>
<div id="app"></div>
<script type="module">
  import { createApp } from '/vue.js';
  import App from '/component.mjs';
  const app = createApp(App);
  const vm = app.mount('#app');
  window.__age_inst = app._instance ?? (vm && vm._instance);
</script>
</body></html>
`
  );

  const vueSource = fs.readFileSync(vuePath);
  const server = http.createServer((req, res) => {
    const url = (req.url || '/').split('?')[0];
    if (url === '/vue.js') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(vueSource);
      return;
    }
    const name = url === '/' ? '/index.html' : url;
    const file = path.join(dir, name);
    if (!file.startsWith(dir) || !fs.existsSync(file)) {
      res.writeHead(404);
      res.end('not found');
      return;
    }
    const type =
      name.endsWith('.mjs') || name.endsWith('.js') ? 'text/javascript' : 'text/html';
    res.writeHead(200, { 'Content-Type': type });
    res.end(fs.readFileSync(file));
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const port = server.address().port;

  const chromium = await resolvePlaywright();
  let browser;
  try {
    browser = await chromium.launch({ headless: true, channel: 'msedge' });
  } catch (_) {
    try {
      browser = await chromium.launch({ headless: true, channel: 'chrome' });
    } catch (_) {
      browser = await chromium.launch({ headless: true });
    }
  }
  const page = await (await browser.newContext()).newPage();
  const pageErrors = [];
  page.on('pageerror', (err) => pageErrors.push(err.message));
  const consoleErrors = [];
  page.on('console', (m) => {
    if (m.type() === 'error') consoleErrors.push(m.text());
  });
  await page.goto(`http://127.0.0.1:${port}/index.html`);
  try {
    await page.waitForSelector('#app button', { timeout: 8000 });
  } catch (e) {
    console.error(
      `mount timeout; page errors: ${pageErrors.join('; ')}; console: ${consoleErrors.join('; ')}`
    );
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
    throw e;
  }

  const read = async () =>
    await page.evaluate(() => {
      const inst = window.__age_inst;
      if (!inst || !inst.exposed || typeof inst.exposed.__state !== 'function') return null;
      return inst.exposed.__state();
    });

  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };
  const count = (s) => (s && typeof s.count === 'number' ? s.count : null);

  const before = await read();
  check(`initial state readable (got ${JSON.stringify(before)})`, before !== null);
  await page.click('text=+');
  const afterInc = await read();
  await page.click('text=Reset');
  const afterReset = await read();
  await page.click('text=+');
  await page.click('text=+');
  const afterTwo = await read();

  check(`0 -> 1 after + (got ${count(afterInc)})`, count(afterInc) === 1);
  check(`1 -> 0 after Reset (got ${count(afterReset)})`, count(afterReset) === 0);
  check(`two + clicks -> 2 (got ${count(afterTwo)})`, count(afterTwo) === 2);
  check('no page errors', pageErrors.length === 0);

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(
      `FAILURES: ${failures}; page errors: ${pageErrors.join('; ')}; console errors: ${consoleErrors.join('; ')}`
    );
    process.exit(1);
  }
  console.log('SFC_MOUNT_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
