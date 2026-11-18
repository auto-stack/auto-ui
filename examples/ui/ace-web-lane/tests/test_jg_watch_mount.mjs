// PLAN-980 B-WEB jade-garden watch 路径挂载档（④-A #6 首刀）：**原样
// `quick_switcher.at`**（auto-down jade-garden，sha256 冻结）经产品出口
// `generate_sfc_with_lane_product` 生成的 SFC 在真实 Vue 挂载中验证
// **watch 族行为**——
//   watch(open)：打开 ⇒ query 清空 + selected_index=0 + focusSwitcherInput；
//   watch(filtered)：过滤结果变化 ⇒ selected_index 归零（高亮回第一项）。
// 宿主侧夹具（rig 所有，非产物注入）：quick_switcher_ext 的最小 stub
// （组合式 store 返回固定文件树/记录式 tabs.open/hotkey 回调暴露到
// window.__jg）——真实链路中该文件由 app 的 ext 提供（vite `@/ext` 别名）。
// 产物侧零改写：仅 dev-server 式别名（'vue'→/vue.js、'@/ext/…'→stub）。
//
// 用法：node test_jg_watch_mount.mjs <dir-with-component.sfc>
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

const EXT_STUB = `// rig 夹具：quick_switcher_ext 最小宿主（真实 app 由 vite @/ext 别名供真件）。
window.__jg = { focus_calls: 0, opened: null, open: null, close: null };
export function listenSwitcherHotkeys(open_cb, close_cb) { window.__jg.open = open_cb; window.__jg.close = close_cb; }
export function unlistenSwitcherHotkeys() {}
export function focusSwitcherInput() { window.__jg.focus_calls += 1; }
export const Search = { name: 'Search', render() { return null; } };
export function useFileTreeStore() {
  return { files: [
    { is_dir: false, path: 'docs/alpha.md', name: 'alpha.md' },
    { is_dir: false, path: 'docs/alphabet.md', name: 'alphabet.md' },
    { is_dir: false, path: 'src/beta.vue', name: 'beta.vue' },
    { is_dir: false, path: 'notes/gamma.md', name: 'gamma.md' },
  ] };
}
export function useTabsStore() { return { open(p) { window.__jg.opened = p; } }; }
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
    console.error('usage: node test_jg_watch_mount.mjs <dir-with-component.sfc>');
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
  const compiled = compileScript(descriptor, { id: 'jg-watch', inlineTemplate: true });
  const esbuildPath = firstExisting(ESBUILD_CANDIDATES);
  const esbuild = (await import(pathToFileURL(esbuildPath).href)).default ?? {};
  const js = esbuild.transformSync(compiled.content, { loader: 'ts', format: 'esm' }).code;

  // dev-server 式别名（模块解析职责，产物内容零改写）：'vue'→本地 vue、
  // '@/ext/…quick_switcher_ext'→rig stub。
  fs.writeFileSync(
    path.join(dir, 'component.mjs'),
    js
      .replace(/from ["']vue["']/g, 'from "/vue.js"')
      .replace(/from ["']@\/ext\/src\/front\/utils\/quick_switcher_ext["']/g, 'from "/ext/quick_switcher_ext.mjs"')
  );
  fs.writeFileSync(path.join(dir, 'ext_stub.mjs'), EXT_STUB);
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
    if (process.env.JG_DEBUG) console.error('REQ', url);
    if (url === '/vue.js') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(vueSource);
      return;
    }
    if (url === '/ext/quick_switcher_ext.mjs') {
      res.writeHead(200, { 'Content-Type': 'text/javascript' });
      res.end(fs.readFileSync(path.join(dir, 'ext_stub.mjs')));
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
    // 根节点在 open=false 时渲染空 div（不可见）——等挂载完成（attached）
    // 且宿主回调已注册（onMounted 执行完）。
    await page.waitForSelector('#app', { state: 'attached', timeout: 8000 });
    await page.waitForFunction(() => window.__jg && window.__jg.open, null, {
      timeout: 8000,
    });
  } catch (e) {
    console.error(
      `mount timeout; page errors: ${pageErrors.join('; ')}; console: ${consoleErrors.join('; ')}`
    );
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
    throw e;
  }

  const results = [];
  const check = (name, ok) => {
    results.push([name, ok]);
    console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}`);
  };
  // 高亮项序（li 中 class 含 bg-accent 者；-1=无高亮）。
  const highlightedIndex = async () =>
    await page.evaluate(() => {
      const lis = [...document.querySelectorAll('#app li')];
      // 精确 token：非高亮臂含 `hover:bg-accent/50`（子串误报源）。
      const at = lis.findIndex((li) => li.classList.contains('bg-accent'));
      return at;
    });
  const itemCount = async () =>
    await page.evaluate(() => document.querySelectorAll('#app li').length);

  // ①初始：open=false——浮层（input）不渲染。
  const inputHiddenInitially = (await page.locator('#app input').count()) === 0;
  check('overlay hidden while open=false', inputHiddenInitially);

  // ②watch(open)：宿主回调打开 ⇒ 浮层出现 + query 空 + focus 被调。
  await page.evaluate(() => window.__jg.open());
  await page.waitForSelector('#app input', { timeout: 4000 });
  const queryAfterOpen = await page.inputValue('#app input');
  const focusCalls = await page.evaluate(() => window.__jg.focus_calls);
  check('watch(open): overlay renders + query cleared', queryAfterOpen === '');
  check('watch(open): focusSwitcherInput invoked', focusCalls >= 1);

  // ③过滤：'alph' ⇒ alpha/alphabet 两项（filtered 计算 + oninput 旧道体）。
  await page.fill('#app input', 'alph');
  const n2 = await itemCount();
  check(`filter 'alph' -> 2 items (got ${n2})`, n2 === 2);
  const hl0 = await highlightedIndex();
  check(`selected_index starts at 0 (highlight ${hl0})`, hl0 === 0);

  // ④ArrowDown ⇒ next_index(0,2)=1——高亮到第二项。
  await page.press('#app input', 'ArrowDown');
  const hl1 = await highlightedIndex();
  check(`ArrowDown -> highlight item 1 (got ${hl1})`, hl1 === 1);

  // ⑤watch(filtered)：'alphabe' ⇒ 过滤缩为 1 项且 selected_index 归零。
  await page.fill('#app input', 'alphabe');
  const n1 = await itemCount();
  const hl2 = await highlightedIndex();
  check(`filter 'alphabe' -> 1 item (got ${n1})`, n1 === 1);
  check(`watch(filtered): selected_index reset to 0 (highlight ${hl2})`, hl2 === 0);

  // ⑥page error：具名容忍**预存部署缺陷**（真实部署件同形，证据=
  // jade-garden front/src/components/QuickSwitcher.vue:144）：旧道生成器对
  // `oninput: .QueryInput($event)` 的双重解包——模板预取 $event.target.value
  // + 脚本体再 e.target.value ⇒ 每击键 TypeError（v-model 掩盖状态面）。
  // 归属=生成器 input 面（B-WEB 生成器道后续刀）；本 rig 只容忍该具名族。
  const known = (e) => String(e).includes('QueryInput') && String(e).includes('value');
  const unexpected = pageErrors.filter((e) => !known(e));
  check(
    `no page errors beyond the named pre-existing QueryInput double-deref (unexpected=${unexpected.length})`,
    unexpected.length === 0
  );

  await browser.close();
  await new Promise((resolve) => server.close(resolve));
  const failures = results.filter(([, ok]) => !ok).length;
  if (failures > 0) {
    console.error(
      `FAILURES: ${failures}; page errors: ${pageErrors.join(' | ')}`
    );
    process.exit(1);
  }
  console.log('JG_WATCH_MOUNT_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
