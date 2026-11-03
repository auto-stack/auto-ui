// PLAN-037 T-02: 016-calendar Vue leg (official theme package adoption;
// first Vue regression entry for this example).
// Explicit pac `theme: "light"` default preserved (boot light); theme
// family/mode round-trip via setupState drive; calendar identity (selection,
// month nav) preserved across switches; settings Dark/Light still work.
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'playwright'
  ];
  for (const p of possiblePaths) {
    try {
      const target = p.includes(':') || p.startsWith('/') ? pathToFileURL(p).href : p;
      const mod = await import(target);
      return mod.chromium;
    } catch (_) {}
  }
  throw new Error('Playwright not found in standard node_modules locations.');
}

async function main() {
  const url = process.argv[2] || 'http://localhost:5436';
  const outDir = path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([a-zA-Z]:)/, '$1')), 'screenshots');
  fs.mkdirSync(outDir, { recursive: true });

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
  const page = await (await browser.newContext({ viewport: { width: 900, height: 900 } })).newPage();
  const errors = [];
  page.on('pageerror', err => errors.push(err.message));

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };

  console.log(`[*] Navigating to ${url}...`);
  await page.goto(url, { waitUntil: 'networkidle' });

  const themeRoot = () => page.evaluate(() =>
    getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim());
  const drive = (theme, dark) => page.evaluate(([t, d]) => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    st.theme_name = t;
    st.dark_mode = d;
  }, [theme, dark]);
  const calState = () => page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    const store = st.store && (st.store.value ?? st.store);
    return {
      year: String(store.year), month: String(store.month),
      selected: String(store.selected_date),
    };
  });

  // Original business markers.
  const body = await page.textContent('body');
  for (const marker of ['Today', 'Selected:', 'Su', 'Mo']) {
    check(`content marker: ${marker}`, body.includes(marker));
  }
  await page.screenshot({ path: path.join(outDir, '016_vue_initial.png'), fullPage: true });

  // Explicit light default preserved at boot.
  check('boot Auto/light bg = 0 0% 100% (pac theme:"light" preserved)',
    (await themeRoot()) === '0 0% 100%');

  // Select day 15.
  const before = await calState();
  await page.getByRole('button', { name: '15', exact: true }).first().click();
  await page.waitForTimeout(400);
  const afterSel = await calState();
  check('day 15 selected updates store', afterSel.selected.includes('-15'), afterSel.selected);

  // Theme 2x2 with selection preservation.
  await drive('Solarized', true);
  await page.waitForTimeout(400);
  check('Solarized/dark bg = 192 100% 11% (#002b36)', (await themeRoot()) === '192 100% 11%');
  check('selection preserved in Solarized/dark', (await calState()).selected === afterSel.selected);
  await page.screenshot({ path: path.join(outDir, '016_vue_solarized_dark.png'), fullPage: true });

  // Month nav across theme: advance and come back, selection intact.
  await page.getByRole('button', { name: '>', exact: true }).click();
  await page.waitForTimeout(400);
  const afterNext = await calState();
  check('NextMonth advances under dark theme', afterNext.month !== before.month || afterNext.year !== before.year,
    `${before.year}-${before.month} -> ${afterNext.year}-${afterNext.month}`);
  check('selection preserved across nav+theme', afterNext.selected === afterSel.selected);
  await page.getByRole('button', { name: '<', exact: true }).click();
  await page.waitForTimeout(400);

  for (const [t, d] of [['Solarized', false], ['Auto', true], ['Auto', false]]) {
    await drive(t, d);
    await page.waitForTimeout(350);
  }
  check('round-trip back to Auto/light', (await themeRoot()) === '0 0% 100%');
  check('selection preserved after full round-trip', (await calState()).selected === afterSel.selected);

  // Settings popup Dark/Light buttons still drive the mode.
  await page.getByRole('button', { name: '⚙ Settings' }).click();
  await page.waitForTimeout(300);
  await page.getByRole('button', { name: '🌙 Dark' }).click();
  await page.waitForTimeout(400);
  check('settings Dark button switches mode', (await themeRoot()) === '240 10% 4%');
  await page.getByRole('button', { name: '☀ Light' }).click();
  await page.waitForTimeout(400);
  check('settings Light button restores light', (await themeRoot()) === '0 0% 100%');

  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    process.exit(1);
  }
  console.log('[+] All 016 Vue assertions passed!');
}

main().catch(err => {
  console.error('[-] Error in 016 Vue runner:', err);
  process.exit(1);
});
