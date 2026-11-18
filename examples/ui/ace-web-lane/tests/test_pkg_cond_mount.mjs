// PLAN-980 view-if 边界挂载档：包版 SFC 条件渲染的**真 Vue 编译+分支切换**
// ——空 then 锚收放（v-if 空模板+v-else）、else-if 三态轮换、（含引号条件
// 的转义判据在 crate 单元测——包面 checker 条件值域边界）。
//
// 用法：node test_pkg_cond_mount.mjs <dir-with-component.sfc>
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
  if (!dir || !fs.existsSync(path.join(dir, 'component.sfc'))) {
    console.error('usage: node test_pkg_cond_mount.mjs <dir-with-component.sfc>');
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
  const compiled = compileScript(descriptor, { id: 'pkg-cond', inlineTemplate: true });
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    js.replace(/from ["']vue["']/g, 'from "/vue.js"')
  );
  fs.writeFileSync(
    path.join(dir, 'index.html'),
    `<!DOCTYPE html>\n<html><body>\n<div id="app"></div>\n<script type="module">\n  import { createApp } from '/vue.js';\n  import App from '/component.mjs';\n  createApp(App).mount('#app');\n</script>\n</body></html>\n`
  );

  const vueSource = fs.readFileSync(vuePath);
  const server = http.createServer((req, res) => {
    const url = (req.url || '/').split('?')[0];
    if (url === '/vue.js') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      return res.end(vueSource);
    }
    const name = url === '/' ? '/index.html' : url;
    const file = path.resolve(path.join(dir, name));
    if (!file.startsWith(path.resolve(dir)) || !fs.existsSync(file)) {
      res.writeHead(404);
      return res.end('not found');
    }
    res.writeHead(200, {
      'Content-Type': name.endsWith('.mjs') || name.endsWith('.js') ? 'text/javascript' : 'text/html',
    });
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
  await page.goto(`http://127.0.0.1:${port}/`);
  try {
    await page.waitForSelector('#app button', { timeout: 8000 });
  } catch (e) {
    console.error(`mount timeout; page errors: ${pageErrors.join(' | ')}`);
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
    throw e;
  }

  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };
  const visible = (needle) =>
    page.evaluate((n) => document.body.innerText.includes(n), needle);

  // ①空 then 锚：flag=false → (flag!==true)=true → 空 then 分支（else 收起）。
  check('empty-then: hidden while cond true (flag=false)', !(await visible('empty-then-fallback')));
  // ②切换：T → flag=true → cond false → else 臂可见（空 v-if 锚配对生效）。
  await page.click('button:text-is("T")');
  check('empty-then: else arm shows on cond false', await visible('empty-then-fallback'));
  // ③再切回：T → 空 then 复收。
  await page.click('button:text-is("T")');
  check('empty-then: collapses again on cond true', !(await visible('empty-then-fallback')));
  // ④else-if 三态：n=0→many；C→one；C→two；C→many。
  check('else-if: many at n=0', await visible('many'));
  await page.click('button:text-is("C")');
  check('else-if: one at n=1', (await visible('one')) && !(await visible('many')));
  await page.click('button:text-is("C")');
  check('else-if: two at n=2', (await visible('two')) && !(await visible('one')));
  await page.click('button:text-is("C")');
  check('else-if: back to many at n=3', (await visible('many')) && !(await visible('two')));
  // ⑤零 page error（含空 v-if 锚的真编译无错）。
  check('no page errors', pageErrors.length === 0);

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(`FAILURES: ${failures}; page errors: ${pageErrors.join(' | ')}`);
    process.exit(1);
  }
  console.log('PKG_COND_MOUNT_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
