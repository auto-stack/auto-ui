// PLAN-033 T-03: 004-profile-card Vue leg (theme migration acceptance).
// Business display-only (Follow/Message handler-less); avatar real-URL
// semantics; theme family/mode round-trip via setupState model drive.
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

await page.goto(process.argv[2] || 'http://localhost:5432/', { waitUntil: 'networkidle' });

// Original profile content.
const body = await page.textContent('body');
for (const marker of ['Jane Cooper', 'Full Stack Developer', 'Follow', 'Message', 'Active']) {
  check(`content marker: ${marker}`, body.includes(marker));
}
for (const banned of ['Settings', 'Subscribe', 'Payment']) {
  check(`no new business surface: ${banned}`, !body.includes(banned));
}

// Avatar: real remote URL semantics — element present; success or documented
// failure placeholder both acceptable (network-dependent in headless runs),
// the source URL is never replaced.
const avatar = await page.evaluate(() => {
  const img = document.querySelector('img');
  if (!img) return { present: false };
  return {
    present: true,
    src: img.getAttribute('src') || '',
    complete: img.complete,
    naturalWidth: img.naturalWidth,
  };
});
check('avatar image element present', avatar.present);
check('avatar keeps original remote URL semantics',
  !avatar.present || avatar.src.includes('cravatar.com'));
console.log(`[i] avatar load state: complete=${avatar.complete} naturalWidth=${avatar.naturalWidth} (network-dependent)`);
await page.screenshot({ path: path.join(outDir, '004_vue_initial.png'), fullPage: true });

// Handler-less buttons are safe no-ops.
for (const label of ['Follow', 'Message']) {
  const btn = await page.$(`button:has-text("${label}")`);
  check(`button ${label} present`, btn !== null);
  if (btn) {
    await btn.click();
    await page.waitForTimeout(300);
  }
}
check('page stable after handler-less clicks',
  (await page.textContent('body')).includes('Jane Cooper'));

// Theme 2x2 round-trip via model drive with package effective values.
let bg = await rootBg();
check('boot Auto/light bg = 0 0% 100%', bg === '0 0% 100%');

await drive('Solarized', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '004_vue_solarized_light.png'), fullPage: true });

await drive('Solarized', true);
await page.waitForTimeout(400);
bg = await rootBg(page);
bg = await rootBg();
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '004_vue_solarized_dark.png'), fullPage: true });

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Auto/dark bg = 240 10% 4%', bg === '240 10% 4%');

await drive('Auto', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('round-trip back to Auto/light', bg === '0 0% 100%');
check('content preserved after round-trip',
  (await page.textContent('body')).includes('Jane Cooper'));

await browser.close();
if (failures > 0) {
  console.error(`[-] ${failures} assertion(s) failed`);
  process.exit(1);
}
console.log('[+] All Vue assertions passed!');
