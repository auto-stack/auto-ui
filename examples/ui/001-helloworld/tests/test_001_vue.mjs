// PLAN-033 T-03: 001-helloworld Vue leg (theme migration acceptance).
// Channels: setupState model drive (the Vue analog of the VM fixture channel —
// real reactive state write, never CSS tampering) + __AUTO_UI_THEME__
// bootstrap for per-load mode seeding (host channel, Plan 458).
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'playwright'
  ];
  for (const p of possiblePaths) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      return (await import(target)).chromium;
    } catch (_) {}
  }
  throw new Error('Playwright not found');
}

const outDir = path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([a-zA-Z]:)/, '$1')), 'screenshots');
fs.mkdirSync(outDir, { recursive: true });

const chromium = await resolvePlaywright();
const browser = await chromium.launch({ headless: true, channel: 'msedge' })
  .catch(() => chromium.launch({ headless: true }));
const page = await (await browser.newContext({ viewport: { width: 1280, height: 800 } })).newPage();

let failures = 0;
const check = (name, ok) => {
  if (ok) console.log(`[+] OK: ${name}`);
  else { failures++; console.log(`[-] FAIL: ${name}`); }
};
const rootBg = (page) => page.evaluate(() =>
  getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim());
const drive = (page, theme, dark) => page.evaluate(([t, d]) => {
  const st = document.querySelector('#app').__vue_app__._instance.setupState;
  st.theme_name = t;
  st.dark_mode = d;
}, [theme, dark]);

await page.goto(process.argv[2] || 'http://localhost:5431/', { waitUntil: 'networkidle' });

// Original content + selectable text.
const hello = await page.textContent('body');
check('greeting present', hello.includes('Hello, World!'));
const selectable = await page.evaluate(() => {
  const el = [...document.querySelectorAll('[data-auto-tag="text"]')]
    .find(e => e.textContent.includes('Hello, World!'));
  return el ? getComputedStyle(el).userSelect : 'none';
});
check('greeting selectable (Plan 481)', selectable === 'select' || selectable === 'text');

// Declared default: Auto/light — package value.
let bg = await rootBg(page);
check('boot Auto/light bg = package value (0 0% 100%)', bg === '0 0% 100%');
await page.screenshot({ path: path.join(outDir, '001_vue_auto_light.png'), fullPage: true });

// 2x2 round-trip via model drive; effective container var per combo.
await drive(page, 'Solarized', false);
await page.waitForTimeout(400);
bg = await rootBg(page);
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '001_vue_solarized_light.png'), fullPage: true });

await drive(page, 'Solarized', true);
await page.waitForTimeout(400);
bg = await rootBg(page);
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '001_vue_solarized_dark.png'), fullPage: true });

await drive(page, 'Auto', true);
await page.waitForTimeout(400);
bg = await rootBg(page);
check('Auto/dark bg = 240 10% 4% (#09090b)', bg === '240 10% 4%');

await drive(page, 'Auto', false);
await page.waitForTimeout(400);
bg = await rootBg(page);
check('round-trip back to Auto/light', bg === '0 0% 100%');
check('content preserved after round-trip', (await page.textContent('body')).includes('Hello, World!'));

// Host system channel: standalone page host = OS — the delivered bootstrap
// follows prefers-color-scheme live (Plan 672 chain) and seeds dark_mode.
await page.emulateMedia({ colorScheme: 'dark' });
await page.reload({ waitUntil: 'networkidle' });
await page.waitForTimeout(600);
const darkBooted = await page.evaluate(() =>
  document.querySelector('[data-auto-theme-root]').classList.contains('dark'));
check('OS prefers-color-scheme=dark boot seeds dark_mode (system channel)', darkBooted);
const bgDark = await rootBg(page);
check('system-dark boot keeps declared family values (Auto dark bg)', bgDark === '240 10% 4%');
await page.screenshot({ path: path.join(outDir, '001_vue_bootstrap_dark.png'), fullPage: true });

await browser.close();
if (failures > 0) {
  console.error(`[-] ${failures} assertion(s) failed`);
  process.exit(1);
}
console.log('[+] All Vue assertions passed!');
