// PLAN-1072 T-03 浏览器档：setup（仅一次）/ watch（触发 + 卸载停止）/
// expose（defineExpose 调用与读）/ ext_imports / style（配方展开）实浏览器验证。
//
// 输入目录由 Rust driver 落盘：
//   <dir>/compA.sfc  —— ACE lane 全派生件（style + watch + expose + on，absent=[]）
//   <dir>/a2x_rt.mjs —— 官方 a2x 运行时（原样落盘，零注入）
//   <dir>/compB.sfc  —— setup + use{fn} + watch + expose 双轨件
// 用法：node test_p1072_lifecycle_watch_expose.mjs <dir>
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

const BOOT_EXT_STUB = `window.__setup_calls = 0;
export function record_setup() {
  window.__setup_calls += 1;
  return 'boot-' + window.__setup_calls;
}
`;

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

async function compileSfcToMjs(parse, compileScript, esbuild, sfcPath, id, extraRewrite) {
  const source = fs.readFileSync(sfcPath, 'utf8');
  const { descriptor, errors } = parse(source, { filename: path.basename(sfcPath) });
  if (errors && errors.length) {
    throw new Error(`SFC parse errors (${id}): ${JSON.stringify(errors)}`);
  }
  const compiled = compileScript(descriptor, { id, inlineTemplate: true });
  let js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
  js = js.replace(/from ["']vue["']/g, 'from "/vue.js"');
  if (extraRewrite) js = extraRewrite(js);
  return js;
}

async function main() {
  const dir = process.argv[2];
  if (
    !dir ||
    !fs.existsSync(path.join(dir, 'compA.sfc')) ||
    !fs.existsSync(path.join(dir, 'compB.sfc')) ||
    !fs.existsSync(path.join(dir, 'a2x_rt.mjs'))
  ) {
    console.error('usage: node test_p1072_lifecycle_watch_expose.mjs <dir>');
    process.exit(2);
  }
  const compiler = await import(pathToFileURL(firstExisting(COMPILER_CANDIDATES)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const vuePath = firstExisting(VUE_CANDIDATES);
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};

  const jsA = await compileSfcToMjs(parse, compileScript, esbuild, path.join(dir, 'compA.sfc'), 'p1072-we-a');
  const jsB = await compileSfcToMjs(
    parse,
    compileScript,
    esbuild,
    path.join(dir, 'compB.sfc'),
    'p1072-we-b',
    (js) => js.replace(/from ["']@\/ext\/src\/front\/utils\/boot["']/g, 'from "/ext/boot.mjs"')
  );

  fs.writeFileSync(path.join(dir, 'compA.mjs'), jsA);
  fs.writeFileSync(path.join(dir, 'compB.mjs'), jsB);
  fs.writeFileSync(path.join(dir, 'boot.mjs'), BOOT_EXT_STUB);
  fs.writeFileSync(
    path.join(dir, 'index.html'),
    `<!DOCTYPE html>
<html><body>
<div id="app-a"></div>
<div id="app-b"></div>
<script type="module">
  import { createApp, nextTick } from '/vue.js';
  import CompA from '/compA.mjs';
  import CompB from '/compB.mjs';
  const appA = createApp(CompA);
  const vmA = appA.mount('#app-a');
  const appB = createApp(CompB);
  const vmB = appB.mount('#app-b');
  window.__p1072 = { appA, vmA, appB, vmB, nextTick };
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
    if (url === '/ext/boot.mjs') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(fs.readFileSync(path.join(dir, 'boot.mjs')));
      return;
    }
    const name = url === '/' ? '/index.html' : url;
    const file = path.resolve(path.join(dir, name));
    if (!file.startsWith(path.resolve(dir)) || !fs.existsSync(file)) {
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
  page.on('pageerror', (err) => pageErrors.push((err && err.stack) || err.message));
  const consoleErrors = [];
  page.on('console', (m) => {
    if (m.type() === 'error') consoleErrors.push(m.text());
  });
  await page.goto(`http://127.0.0.1:${port}/index.html`);
  try {
    await page.waitForSelector('#app-a button', { timeout: 8000 });
    await page.waitForSelector('#app-b button', { timeout: 8000 });
    await page.waitForFunction(() => window.__p1072 && window.__setup_calls === 1, null, { timeout: 4000 });
  } catch (e) {
    console.error(`mount timeout; page errors: ${pageErrors.join('; ')}; console: ${consoleErrors.join('; ')}`);
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
    throw e;
  }

  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };

  // ── Widget A (ACE lane): click + twice -> count=2, watch_hits=2
  await page.click('#app-a button');
  await page.click('#app-a button');
  await page.waitForFunction(() => window.__p1072.vmA.watch_hits === 2, null, { timeout: 2000 });
  const aAfterClicks = await page.evaluate(() => ({
    count: window.__p1072.vmA.count,
    hits: window.__p1072.vmA.watch_hits,
    text: document.querySelector('#app-a').innerText,
  }));
  check('A: 2 clicks -> count=2, watch_hits=2', aAfterClicks.count === 2 && aAfterClicks.hits === 2 && aAfterClicks.text.includes('C:2 W:2'));

  // ── Widget A: invoke exposed handler Inc() via defineExpose -> count=3, watch_hits=3
  const aAfterExpose = await page.evaluate(async () => {
    window.__p1072.vmA.Inc();
    await window.__p1072.nextTick();
    return {
      count: window.__p1072.vmA.count,
      hits: window.__p1072.vmA.watch_hits,
    };
  });
  check('A: expose vm.Inc() -> count=3, watch_hits=3', aAfterExpose.count === 3 && aAfterExpose.hits === 3);

  // ── Widget A: unmount stops watch (calling captured Inc() after unmount must NOT increment watch_hits)
  const aAfterUnmount = await page.evaluate(async () => {
    const { appA, vmA } = window.__p1072;
    const exposedInc = vmA.Inc;
    appA.unmount();
    exposedInc();
    await new Promise((r) => setTimeout(r, 40));
    return { count: vmA.count, hits: vmA.watch_hits };
  });
  check(
    `A: unmount stops watch (count=${aAfterUnmount.count}, hits=${aAfterUnmount.hits})`,
    aAfterUnmount.count === 4 && aAfterUnmount.hits === 3
  );

  // ── Widget B (setup + ext_imports + watch + unmount): setup runs strictly once across updates
  const bInitial = await page.evaluate(() => ({
    calls: window.__setup_calls,
    text: document.querySelector('#app-b').innerText,
  }));
  check('B: setup executes once on mount + ext_imports bound', bInitial.calls === 1 && bInitial.text.includes('boot-1'));

  await page.click('#app-b button');
  await page.click('#app-b button');
  await page.waitForFunction(() => window.__p1072.vmB.seen === 2, null, { timeout: 2000 });
  const bAfterClicks = await page.evaluate(() => ({
    calls: window.__setup_calls,
    n: window.__p1072.vmB.n,
    seen: window.__p1072.vmB.seen,
  }));
  check(
    `B: updates preserve setup_calls=1 while watch runs (n=${bAfterClicks.n}, seen=${bAfterClicks.seen}, calls=${bAfterClicks.calls})`,
    bAfterClicks.calls === 1 && bAfterClicks.n === 2 && bAfterClicks.seen === 2
  );

  // ── Widget B: unmount stops watch
  const bAfterUnmount = await page.evaluate(async () => {
    const { appB, vmB } = window.__p1072;
    const exposedBump = vmB.Bump;
    appB.unmount();
    exposedBump();
    await new Promise((r) => setTimeout(r, 40));
    return { n: vmB.n, seen: vmB.seen, calls: window.__setup_calls };
  });
  check(
    `B: unmount stops watch (n=${bAfterUnmount.n}, seen=${bAfterUnmount.seen}, calls=${bAfterUnmount.calls})`,
    bAfterUnmount.n === 3 && bAfterUnmount.seen === 2 && bAfterUnmount.calls === 1
  );

  check(`no page errors (got ${pageErrors.length})`, pageErrors.length === 0);

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(`FAILURES: ${failures}; page errors: ${pageErrors.join(' | ')}; console: ${consoleErrors.join(' | ')}`);
    process.exit(1);
  }
  console.log('P1072_LIFECYCLE_WATCH_EXPOSE_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
