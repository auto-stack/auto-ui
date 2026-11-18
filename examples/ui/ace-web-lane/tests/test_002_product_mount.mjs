// PLAN-980 U0 产品出口挂载档（④-A #5）：页面从**产品产物树**启动——
// <dir>/component.sfc 与 <dir>/a2x_rt.mjs 均为 `generate_sfc_with_lane_product`
// 返回物原样落盘，本 rig **零注入**（不扫 a2x 名、不写 import、不供 rt 内容）。
// 步骤：@vue/compiler-sfc 编译（inlineTemplate）→ esbuild 剥 TS 注记
// （标准 Vue 工具链转换，vite 同族引擎）→ 本地 HTTP 供页面（rt 按 SFC 内
// 显式 `./a2x_rt.mjs` 相对 import 自然解析）→ playwright 真实点击
// 加/减/重置 → 页面文本 `Counter: 0 → 1 → 0`（再两次 `+` → 2）→ 零 page error。
// 唯一改写=裸 `'vue'` 说明符别名到 /vue.js（dev-server 模块解析职责，
// 与 vite alias 同理；产物内容零改写）。
//
// 用法：node test_002_product_mount.mjs <dir>
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
  const dir = process.argv[2];
  if (!dir || !fs.existsSync(path.join(dir, 'component.sfc')) || !fs.existsSync(path.join(dir, 'a2x_rt.mjs'))) {
    console.error('usage: node test_002_product_mount.mjs <dir-with-component.sfc+a2x_rt.mjs>');
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
  const compiled = compileScript(descriptor, { id: 'u0-002-product', inlineTemplate: true });

  // 标准 Vue 工具链转换：`lang="ts"` 的 TS 注记经 esbuild 剥除（vite 同族）。
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;

  // 零注入：a2x_rt import 已由生成器发射（相对 ./a2x_rt.mjs——按目录解析）；
  // 裸 'vue' 说明符做 dev-server 式别名（模块解析，非产物内容改写）。
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    js.replace(/from ["']vue["']/g, 'from "/vue.js"')
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
  app.mount('#app');
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

  const counter = async () => {
    const text = await page.evaluate(() => document.body.innerText);
    const m = text.match(/Counter:\s*(-?\d+)/);
    return m ? Number(m[1]) : null;
  };
  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };

  const before = await counter();
  check(`initial page text Counter: 0 (got ${before})`, before === 0);
  await page.click('button:text-is("+")');
  const afterInc = await counter();
  await page.click('button:text-is("-")');
  const afterDec = await counter();
  await page.click('button:text-is("+")');
  await page.click('button:text-is("+")');
  await page.click('button:text-is("Reset")');
  const afterReset = await counter();
  await page.click('button:text-is("+")');
  await page.click('button:text-is("+")');
  const afterTwo = await counter();

  check(`0 -> 1 after + (加, got ${afterInc})`, afterInc === 1);
  check(`1 -> 0 after - (减, got ${afterDec})`, afterDec === 0);
  check(`2 -> 0 after Reset (重置, got ${afterReset})`, afterReset === 0);
  check(`two + clicks -> 2 (got ${afterTwo})`, afterTwo === 2);
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
  console.log('U0_002_PRODUCT_MOUNT_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
