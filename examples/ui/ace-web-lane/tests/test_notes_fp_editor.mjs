// PLAN-980 ④-A #9：AutoNotes `FpNoteEditor` 浏览器输入链基线（原样链路的
// **现成产品产物**——auto-notes gen/front/vue/src/components/FpNoteEditor.vue
// 只读消费；宿主回调=emit('DraftEdit', v)，与 app.at 的 on_draft_edit 通道
// 同一契约）。判据：textarea 键入 ⇒ 宿主 DraftEdit 收到逐字真实载荷；
// Submit 点击 ⇒ SubmitText 收到 prop 往返值；零预期外 page error。
// 宿主侧具名 stub：`@/components/ui/button`（Button=slot 透传）与
// `lucide-vue-next`（AlertTriangle）——被测链路（textarea→DraftEdit→emit）
// 不经它们；产物内容零改写（vue/@ 别名=dev-server 模块解析职责）。
//
// 用法：node test_notes_fp_editor.mjs <dir-with-component.sfc>
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

// 宿主侧具名 stub（被测输入链不经它们）。
const BUTTON_STUB = `import { h } from '/vue.js';\nexport const Button = { name: 'Button', props: ['variant','size'], setup(_, { slots }) { return () => h('button', { class: 'stub-button' }, slots.default ? slots.default() : []); } };\nexport default Button;\n`;
const LUCIDE_STUB = `export const AlertTriangle = { name: 'AlertTriangle', render() { return null; } };\n`;
const HOST_APP = `import { createApp, ref } from '/vue.js';
import FpNoteEditor from '/component.mjs';
window.__notes = { draft: [], submit: [], rebase: 0 };
const Host = {
  components: { FpNoteEditor },
  setup() {
    const draft_text = ref('seed-body');
    const onDraft = (v) => { window.__notes.draft.push(String(v)); };
    const onSubmit = (t) => { window.__notes.submit.push(String(t)); };
    const onRebase = () => { window.__notes.rebase += 1; };
    return { draft_text, onDraft, onSubmit, onRebase };
  },
  template: '<FpNoteEditor mode="create" note_label="" :draft_text="draft_text" save_state="idle" '
      + ':conflict_revision="0" server_body="" '
      + '@DraftEdit="onDraft" @SubmitText="onSubmit" @Rebase="onRebase" />',
};
createApp(Host).mount('#app');
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

async function main() {
  const dir = process.argv[2];
  if (!dir || !fs.existsSync(path.join(dir, 'component.sfc'))) {
    console.error('usage: node test_notes_fp_editor.mjs <dir-with-component.sfc>');
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
  const compiled = compileScript(descriptor, { id: 'notes-fp', inlineTemplate: true });
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;
  // dev-server 式别名（模块解析职责，产物内容零改写）。
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    js
      .replace(/from ["']vue["']/g, 'from "/vue.js"')
      .replace(/from ["']@\/components\/ui\/button["']/g, 'from "/stub_button.mjs"')
      .replace(/from ["']lucide-vue-next["']/g, 'from "/stub_lucide.mjs"')
  );
  fs.writeFileSync(path.join(dir, 'stub_button.mjs'), BUTTON_STUB);
  fs.writeFileSync(path.join(dir, 'stub_lucide.mjs'), LUCIDE_STUB);
  fs.writeFileSync(path.join(dir, 'host.mjs'), HOST_APP);
  fs.writeFileSync(
    path.join(dir, 'index.html'),
    `<!DOCTYPE html>\n<html><body>\n<div id="app"></div>\n<script type="module" src="/host.mjs"></script>\n</body></html>\n`
  );

  const vueSource = fs.readFileSync(vuePath);
  const server = http.createServer((req, res) => {
    const url = (req.url || '/').split('?')[0];
    const serve = (body) => {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(body);
    };
    if (url === '/vue.js') return serve(vueSource);
    if (url === '/stub_button.mjs') return serve(fs.readFileSync(path.join(dir, 'stub_button.mjs')));
    if (url === '/stub_lucide.mjs') return serve(fs.readFileSync(path.join(dir, 'stub_lucide.mjs')));
    if (url === '/host.mjs') return serve(fs.readFileSync(path.join(dir, 'host.mjs')));
    if (url === '/component.mjs') return serve(fs.readFileSync(path.join(dir, 'component.mjs')));
    if (url === '/') {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      return res.end(fs.readFileSync(path.join(dir, 'index.html')));
    }
    res.writeHead(404);
    res.end('not found');
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
    await page.waitForSelector('#app textarea', { timeout: 8000 });
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
  const notes = () => page.evaluate(() => window.__notes);

  // ①键入草稿 ⇒ 宿主 DraftEdit 收到逐字真实载荷（中文+空格）。
  await page.fill('#app textarea', '你好 FP draft');
  let st = await notes();
  const got1 = st.draft[st.draft.length - 1];
  check(`DraftEdit payload exact (got ${JSON.stringify(got1)})`, got1 === '你好 FP draft');
  // ②继续键入 ⇒ 载荷随键更新（fill 全量值）。
  await page.fill('#app textarea', '你好 FP draft v2');
  st = await notes();
  const got2 = st.draft[st.draft.length - 1];
  check(`DraftEdit follows input (got ${JSON.stringify(got2)})`, got2 === '你好 FP draft v2');
  // ③Submit ⇒ 宿主 SubmitText 收到 prop 往返值（draft_text=seed-body）。
  await page.click('#app button');
  st = await notes();
  const got3 = st.submit[st.submit.length - 1];
  check(`SubmitText carries the prop round-trip (got ${JSON.stringify(got3)})`, got3 === 'seed-body');
  // ④零预期外 page error。
  check('no page errors', pageErrors.length === 0);

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(`FAILURES: ${failures}; page errors: ${pageErrors.join(' | ')}`);
    process.exit(1);
  }
  console.log('NOTES_FP_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
