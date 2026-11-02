// PLAN-033 T-04: 009-article-feed Vue leg (theme migration acceptance).
// Original business: ReadArticle1/2/3 console print feedback, article fields,
// emoji thumb placeholders (original content assets). Theme family/mode
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

await page.goto(process.argv[2] || 'http://localhost:5436/', { waitUntil: 'networkidle' });

// Original article content.
const body = await page.textContent('body');
for (const m of ['Blog Posts', 'Read more', 'Getting Started with shadcn/ui Components',
  'Building Accessible Web Applications', 'Modern Design Systems with Tailwind CSS',
  'Sarah Chen', 'Marcus Rodriguez', 'Emma Thompson', 'Tutorial', 'Accessibility']) {
  check(`content marker: ${m}`, body.includes(m));
}
check('no in-app Settings header', (await page.$('button:has-text("Settings")')) === null);
await page.screenshot({ path: path.join(outDir, '009_vue_dark_initial.png'), fullPage: true });

// ReadArticle1/2/3 console print feedback (document order).
const readBtns = await page.$$('button:has-text("Read more")');
check('three Read more buttons', readBtns.length === 3);
const expects = ['Reading article 1', 'Reading article 2', 'Reading article 3'];
for (let i = 0; i < Math.min(readBtns.length, 3); i++) {
  await readBtns[i].click();
  await page.waitForTimeout(300);
  check(`console print: ${expects[i]}`, consoleLines.some(l => l.includes(expects[i])));
}

// Theme 2x2 round-trip with package effective values.
await drive('Solarized', false);
await page.waitForTimeout(400);
let bg = await rootBg();
check('Solarized/light bg = 44 87% 94% (#fdf6e3)', bg === '44 87% 94%');
await page.screenshot({ path: path.join(outDir, '009_vue_solarized_light.png'), fullPage: true });

await drive('Solarized', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Solarized/dark bg = 192 100% 11% (#002b36)', bg === '192 100% 11%');
await page.screenshot({ path: path.join(outDir, '009_vue_solarized_dark.png'), fullPage: true });

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('Auto/dark bg = 240 10% 4% (pac default)', bg === '240 10% 4%');

await drive('Auto', false);
await page.waitForTimeout(400);
bg = await rootBg();
check('light step (Auto/light)', bg === '0 0% 100%');

await drive('Auto', true);
await page.waitForTimeout(400);
bg = await rootBg();
check('round-trip back to Auto/dark (pac default)', bg === '240 10% 4%');
check('content preserved after round-trip',
  (await page.textContent('body')).includes('Sarah Chen'));

// ReadArticle still dispatches after theme round-trips.
const before = consoleLines.length;
const read2 = await page.$$('button:has-text("Read more")');
if (read2.length >= 3) { await read2[2].click(); await page.waitForTimeout(300); }
check('ReadArticle3 still dispatches after round-trip',
  consoleLines.slice(before).some(l => l.includes('Reading article 3')));

await browser.close();
if (failures > 0) {
  console.error(`[-] ${failures} assertion(s) failed`);
  process.exit(1);
}
console.log('[+] All Vue assertions passed!');
