// PLAN-1072 刀A/T-03（V-B-WEB 真浏览器档）：原样 `005-login` 的 **lane 产品
// 产物**挂载——<dir>/component.sfc 与 <dir>/a2x_rt.mjs 均为
// `generate_sfcs_with_lane_product` 返回物原样落盘（零注入；02 rig 同模式）。
// 判据（005 源 expected：必填校验+λ-if 错误清理+主题按钮零错误）：
// ①初始无错误文本；②空提交 ⇒ 双必填错误出现（Email/Password is required
// ——Submit 双 if-else 受检体）；③输入 email ⇒ 该错误清除（λ-if `!=` 体）
// 而 password 错误仍在；④输入 password ⇒ 全清；⑤主题三按钮点击零错误；
// ⑥再提交 ⇒ 错误不再出现；⑦零 page error。
// 用法：node test_005_login.mjs <dir-with-component.sfc+a2x_rt.mjs>
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
  if (
    !dir ||
    !fs.existsSync(path.join(dir, 'component.sfc')) ||
    !fs.existsSync(path.join(dir, 'a2x_rt.mjs'))
  ) {
    console.error('usage: node test_005_login.mjs <dir-with-component.sfc+a2x_rt.mjs>');
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
  const compiled = compileScript(descriptor, { id: 'p1072-005', inlineTemplate: true });
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
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

  const bodyText = async () => (await page.evaluate(() => document.body.innerText));
  const errCount = async () => {
    const t = await bodyText();
    return (t.match(/is required/g) || []).length;
  };
  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };

  // ①初始无必填错误。
  check(`no required errors initially (got ${await errCount()})`, (await errCount()) === 0);

  // ②空提交 ⇒ 双必填错误（Submit 双 if-else 受检体）。
  await page.click('button:text-is("Sign In")');
  const twoErrors = await errCount();
  check(`empty submit shows both required errors (got ${twoErrors})`, twoErrors === 2);

  // ③输入 email ⇒ email 错误清除（λ-if `!=` 体），password 错误仍在。
  const email = page.locator('input').first();
  await email.fill('a@b.c');
  const afterEmail = await errCount();
  check(`typing email clears one error (got ${afterEmail})`, afterEmail === 1);

  // ④输入 password ⇒ 全清。
  const password = page.locator('input[type="password"]');
  await password.fill('secret');
  const afterPassword = await errCount();
  check(`typing password clears the rest (got ${afterPassword})`, afterPassword === 0);

  // ⑤主题三按钮（Auto/Solarized/切换明暗）零错误点击。
  for (const label of ['Auto', 'Solarized', '切换明暗']) {
    await page.click(`button:text-is("${label}")`);
  }
  check(`theme buttons click cleanly`, (await errCount()) === 0);

  // ⑥已填再提交 ⇒ 错误不再出现。
  await page.click('button:text-is("Sign In")');
  check(`filled submit shows no errors (got ${await errCount()})`, (await errCount()) === 0);

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
  console.log('P1072_005_LOGIN_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
