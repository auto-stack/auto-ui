// PLAN-980 ④-A #9：原样 `010-contact-form` 短回归（legacy 产物行为基线；
// ACE 断言在 CPU 档以键开具名拒实证承载——lane 0/6 的 981 域墙见 980 §9）。
// 判据：三字段输入绑定（v-model）；点击 Send ⇒ 回执可见；零 page error。
// 用法：node test_010_form.mjs <dir-with-App.sfc>
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
  if (!dir || !fs.existsSync(path.join(dir, 'App.sfc'))) {
    console.error('usage: node test_010_form.mjs <dir-with-App.sfc>');
    process.exit(2);
  }
  const compiler = await import(pathToFileURL(firstExisting(COMPILER_CANDIDATES)).href);
  const { parse, compileScript } = compiler.default ?? compiler;
  const vuePath = firstExisting(VUE_CANDIDATES);
  const source = fs.readFileSync(path.join(dir, 'App.sfc'), 'utf8');
  const { descriptor, errors } = parse(source, { filename: 'App.sfc' });
  if (errors && errors.length) {
    console.error('SFC parse errors:', errors);
    process.exit(1);
  }
  const compiled = compileScript(descriptor, { id: 'p980-010', inlineTemplate: true });
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
  // 宿主侧具名 stub：`@/components/ui/*` 全族（材质件属脚手架面，非被测
  // 行为）——Button/Input/Textarea 渲染原生元素并透传 v-model，其余
  // div+slot 透传。
  const UI_STUB = [
    "import { h } from '/vue.js';",
    "const native = (tag) => ({ name: tag, props: ['modelValue'], emits: ['update:modelValue'], setup(p, { slots, emit }) { return () => h(tag, { value: p.modelValue, onInput: (e) => emit('update:modelValue', e.target.value) }, slots.default ? slots.default() : []); } });",
    "export const Button = { name: 'Button', props: ['variant','size'], setup(_, { slots }) { return () => h('button', { class: 'stub-button' }, slots.default ? slots.default() : []); } };",
    "export const Input = native('input');",
    "export const Textarea = native('textarea');",
    "export const Card = { name: 'Card', setup(_, { slots }) { return () => h('div', { class: 'stub-card' }, slots.default ? slots.default() : []); } };",
    "const passthrough = (name) => ({ name, setup(_, { slots }) { return () => h('div', { class: 'stub-' + name }, slots.default ? slots.default() : []); } });",
    "export const CardHeader = passthrough('card-header');",
    "export const CardTitle = passthrough('card-title');",
    "export const CardContent = passthrough('card-content');",
    "export const Label = passthrough('label');",
    "",
  ].join(String.fromCharCode(10));
  fs.writeFileSync(path.join(dir, 'stub_ui.mjs'), UI_STUB);
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    js
      .replace(/from ["']vue["']/g, 'from "/vue.js"')
      .replace(/from ["']@\/components\/ui\/[a-z-]+["']/g, 'from "/stub_ui.mjs"')
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
    if (url === '/stub_ui.mjs') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      return res.end(fs.readFileSync(path.join(dir, 'stub_ui.mjs')));
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

  // ①回执初始不可见。
  const feedbackHidden =
    (await page.locator('text=Thank you! Your message has been sent.').count()) === 0;
  check('feedback hidden before submit', feedbackHidden);
  // ②三字段输入绑定（v-model 承载——010 的 value 绑定无显式 oninput）。
  await page.fill('#app input[placeholder="Your name"]', 'Ada Lovelace');
  await page.fill('#app input[placeholder="you@example.com"]', 'ada@example.com');
  await page.fill('#app textarea', 'Hello from the short regression.');
  // ③点击 Send ⇒ 回执出现（Submit 只置 submitted——现有行为）。
  await page.click('#app button');
  const feedback = await page.waitForSelector('text=Thank you! Your message has been sent.', {
    timeout: 4000,
  });
  check('submit shows feedback', feedback !== null);
  // ④零 page error。
  check('no page errors', pageErrors.length === 0);

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(`FAILURES: ${failures}; page errors: ${pageErrors.join(' | ')}`);
    process.exit(1);
  }
  console.log('P980_010_FORM_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
