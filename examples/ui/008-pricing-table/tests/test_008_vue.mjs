// PLAN-033 T-04: 008-pricing-table Vue leg (theme migration acceptance).
// Original business: BuyPlan1/2/3 console print feedback, RECOMMENDED badge;
// equal-height cards MEASURED (V-04 layout oracle); theme family/mode
// round-trip via setupState model drive with package effective values.
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
const page = await (await browser.newContext({ viewport: { width: 1280, height: 900 } })).newPage();

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

await page.goto(process.argv[2] || 'http://localhost:5435/', { waitUntil: 'networkidle' });

// Original content (pac dark boot).
const body = await page.textContent('body');
for (const m of ['Pricing Plans', 'RECOMMENDED', 'Single Developer', 'Team', 'Enterprise', '$39', '$99']) {
  check(`content marker: ${m}`, body.includes(m));
}
check('no in-app Settings header', (await page.$('button:has-text("Settings")')) === null);

// Equal-height cards MEASURED (V-04): the three plan cards' bounding boxes.
const heights = await page.evaluate(() => {
  const cards = [...document.querySelectorAll('[data-auto-theme-root] [class*="rounded-2xl"]')]
    .filter(e => e.textContent.includes('Buy Now') || e.textContent.includes('Contact Us'));
  return cards.slice(0, 3).map(e => Math.round(e.getBoundingClientRect().height));
});
check('three plan cards measured', heights.length === 3);
if (heights.length === 3) {
  const spread = Math.max(...heights) - Math.min(...heights);
  check(`equal-height cards (heights=${heights.join('/')} spread=${spread}px ≤2px)`, spread <= 2);
}
await page.screenshot({ path: path.join(outDir, '008_vue_dark_initial.png'), fullPage: true });

// BuyPlan1/2/3 console print feedback.
const buyBtns = await page.$$('button:has-text("Buy Now")');
const contactBtn = await page.$('button:has-text("Contact Us")');
check('two Buy Now + one Contact Us buttons', buyBtns.length === 2 && contactBtn !== null);
const expects = ['Selected Plan: Single Developer', 'Selected Plan: Team', 'Selected Plan: Enterprise'];
if (buyBtns.length === 2 && contactBtn) {
  await buyBtns[0].click(); await page.waitForTimeout(300);
  await buyBtns[1].click(); await page.waitForTimeout(300);
  await contactBtn.click(); await page.waitForTimeout(300);
  for (const e of expects) {
    check(`console print: ${e}`, consoleLines.some(l => l.includes(e)));
  }
}

// Theme 2x2 round-trip with package effective values.
await drive('Solarized', false);
await page.waitForTimeout(400);
let bg = await rootBg();
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '008_vue_solarized_light.png'), fullPage: true });

await drive('Solarized', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '008_vue_solarized_dark.png'), fullPage: true });

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Auto/dark bg = 240 10% 4% (pac default)', bg === '240 10% 4%');

await drive('Solarized', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('mixed round-trip step (Solarized/light again)', bg === '44 87% 94%');

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('round-trip back to Auto/dark (pac default)', bg === '240 10% 4%');
check('content preserved after round-trip',
  (await page.textContent('body')).includes('RECOMMENDED'));

// BuyPlan still dispatches after theme round-trips.
const before = consoleLines.length;
const buy2 = await page.$$('button:has-text("Buy Now")');
if (buy2.length >= 2) { await buy2[1].click(); await page.waitForTimeout(300); }
check('BuyPlan2 still dispatches after round-trip',
  consoleLines.slice(before).some(l => l.includes('Selected Plan: Team')));

await browser.close();
if (failures > 0) {
  console.error(`[-] ${failures} assertion(s) failed`);
  process.exit(1);
}
console.log('[+] All Vue assertions passed!');
