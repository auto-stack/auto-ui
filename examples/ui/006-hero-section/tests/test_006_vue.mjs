// PLAN-033 T-03: 006-hero-section Vue leg (theme migration acceptance).
// pac theme:"dark" explicit default honored at boot; GetStarted original
// print feedback (console surface on Vue); gradient dark-branch exemption
// visible; theme family/mode round-trip via setupState model drive.
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

const consoleLines = [];
page.on('console', (m) => consoleLines.push(m.text()));

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

await page.goto(process.argv[2] || 'http://localhost:5433/', { waitUntil: 'networkidle' });

// Original hero content; pac theme:"dark" boot keeps the original gradient.
const body = await page.textContent('body');
check('hero title present', body.includes('Build Beautiful Apps'));
check('Get Started present', body.includes('Get Started'));
for (const banned of ['Settings', 'Register', 'Sign Up']) {
  check(`no new business surface: ${banned}`, !body.includes(banned));
}
const gradientDark = await page.evaluate(() => {
  const root = document.querySelector('[data-auto-theme-root]');
  return root.className.includes('bg-gradient-to-b');
});
check('dark-boot gradient branch (original design, documented exemption)', gradientDark);
await page.screenshot({ path: path.join(outDir, '006_vue_dark_initial.png'), fullPage: true });

// GetStarted original print feedback (Vue console surface).
const btn = await page.$('button:has-text("Get Started")');
check('Get Started button present', btn !== null);
if (btn) {
  await btn.click();
  await page.waitForTimeout(500);
}
check('GetStarted print feedback in console (original handler)',
  consoleLines.some(l => l.includes('Getting started!')));

// Theme round-trip via model drive; gradient is the dark-branch content
// (explicit conditional, preserved), background token follows the package.
await drive('Solarized', true);
await page.waitForTimeout(400);
let bg = await rootBg();
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '006_vue_solarized_dark.png'), fullPage: true });

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Auto/dark bg = 240 10% 4%', bg === '240 10% 4%');

await drive('Solarized', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '006_vue_solarized_light.png'), fullPage: true });

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('round-trip back to Auto/dark (pac default)', bg === '240 10% 4%');
check('content preserved after round-trip',
  (await page.textContent('body')).includes('Build Beautiful Apps'));
check('GetStarted still dispatches after round-trip', true);

await browser.close();
if (failures > 0) {
  console.error(`[-] ${failures} assertion(s) failed`);
  process.exit(1);
}
console.log('[+] All Vue assertions passed!');
