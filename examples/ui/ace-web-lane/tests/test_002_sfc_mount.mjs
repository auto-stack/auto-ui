// PLAN-980 U0 预验收（④-A #4，A 路径=单文件组件路径）：**未改写 002-counter**
// 的 lane SFC（generate_sfc_with_lane 产出）真实 Vue 挂载档。
//
// 输入：<dir>/002.sfc（Rust 侧生成）+ <dir>/a2x_rt.mjs（a2x-ts 官方
// script_runtime_body 产出——单文件路径的 SFC 脚本不含 lane helper 定义，
// 挂载时按需注入 import；helper 名=component.mjs 实际引用 ∩ rt 导出面）。
// 步骤：@vue/compiler-sfc 编译（inlineTemplate）→ 本地 HTTP 提供页面
// （file:// 下 ESM 跨源被拒）→ playwright 真实点击 加/减/重置 → 页面文本
// `Counter: 0 → 1 → 0`（再两次 `+` → 2）→ 零 page error。
//
// 用法：node test_002_sfc_mount.mjs <dir>
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
  if (!dir || !fs.existsSync(path.join(dir, '002.sfc'))) {
    console.error('usage: node test_002_sfc_mount.mjs <dir-with-002.sfc>');
    process.exit(2);
  }
  const compiler = await import(pathToFileURL(firstExisting(COMPILER_CANDIDATES)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const vuePath = firstExisting(VUE_CANDIDATES);
  const source = fs.readFileSync(path.join(dir, '002.sfc'), 'utf8');
  const { descriptor, errors } = parse(source, { filename: '002.sfc' });
  if (errors && errors.length) {
    console.error('SFC parse errors:', errors);
    process.exit(1);
  }
  const compiled = compileScript(descriptor, { id: 'u0-002-sfc', inlineTemplate: true });

  // 单文件路径 SFC 为 `lang="ts"`（compileScript 输出保留 TS 注记）——
  // 浏览器执行前经 esbuild 转 JS（仅剥类型，语义零改写）。
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, {
    loader: 'ts',
    format: 'esm',
  }).code;

  // lane helper 注入：component 引用的 a2x_* 名 ∩ rt 实际导出面。
  const rtPath = path.join(dir, 'a2x_rt.mjs');
  const rtSource = fs.existsSync(rtPath) ? fs.readFileSync(rtPath, 'utf8') : '';
  const rtExports = new Set([...rtSource.matchAll(/export function (a2x_\w+)/g)].map((m) => m[1]));
  const needed = [...new Set([...js.matchAll(/\b(a2x_\w+)\b/g)].map((m) => m[1]))].filter((n) =>
    rtExports.has(n)
  );
  if (needed.length === 0) {
    console.error('no a2x runtime helpers resolvable (rt exports: ' + [...rtExports].join(', ') + ')');
    process.exit(1);
  }
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    `import { ${needed.join(', ')} } from '/a2x_rt.mjs';\n` +
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
    if (url === '/a2x_rt.mjs') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(rtSource);
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
  console.log('U0_002_MOUNT_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
