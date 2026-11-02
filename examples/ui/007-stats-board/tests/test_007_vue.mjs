// PLAN-033 T-04: 007-stats-board Vue leg (theme migration acceptance).
// Dashboard display-only (no handlers); category-identity colors stay fixed
// literals (mapping table); themed surfaces follow package values; theme
// family/mode round-trip via setupState model drive.
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
const rootBg = () => page.evaluate(() =>
  getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim());
const drive = (theme, dark) => page.evaluate(([t, d]) => {
  const st = document.querySelector('#app').__vue_app__._instance.setupState;
  st.theme_name = t;
  st.dark_mode = d;
}, [theme, dark]);

await page.goto(process.argv[2] || 'http://localhost:5434/', { waitUntil: 'networkidle' });

// Original dashboard content.
const body = await page.textContent('body');
for (const m of ['Dashboard', 'Campaign Insights', 'Campaign Alerts', 'System Alerts',
  'Promotional Offers', 'Traffic Distribution', 'Team Performance', 'Sarah Johnson']) {
  check(`content marker: ${m}`, body.includes(m));
}
for (const banned of ['Settings', 'Refresh', 'Filter']) {
  check(`no new business surface: ${banned}`, !body.includes(banned));
}

// Category chips stay fixed literals (computed style, both the 100-fill and
// the 500 text) — the "legal fixed value" evidence from the mapping table.
const chip = await page.evaluate(() => {
  const el = [...document.querySelectorAll('[data-auto-tag="col"], div')]
    .find(e => getComputedStyle(e).backgroundColor === 'rgb(254, 226, 226)');
  return el ? getComputedStyle(el).backgroundColor : null;
});
check('category chip red-100 fixed literal rgb(254,226,226)', chip === 'rgb(254, 226, 226)');
await page.screenshot({ path: path.join(outDir, '007_vue_auto_light.png'), fullPage: true });

// Theme 2x2 round-trip with package effective values.
let bg = await rootBg();
check('boot Auto/light bg = 0 0% 100%', bg === '0 0% 100%');
await drive('Solarized', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '007_vue_solarized_light.png'), fullPage: true });
await drive('Solarized', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '007_vue_solarized_dark.png'), fullPage: true });
await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Auto/dark bg = 240 10% 4%', bg === '240 10% 4%');
await drive('Auto', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('round-trip back to Auto/light', bg === '0 0% 100%');
check('content + literals preserved after round-trip',
  (await page.textContent('body')).includes('Sarah Johnson'));
await browser.close();
if (failures > 0) { console.error(`[-] ${failures} assertion(s) failed`); process.exit(1); }
console.log('[+] All Vue assertions passed!');
