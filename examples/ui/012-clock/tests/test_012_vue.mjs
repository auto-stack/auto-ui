// PLAN-037 T-02: 012-clock Vue leg (official theme package adoption).
// Original business (tabs/timer/world clock) preserved; theme family/mode
// round-trip via setupState model drive with effective package values
// (B1 oracle values); timer keeps ticking across theme switches.
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
  const url = process.argv[2] || 'http://localhost:5434';
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
  const page = await (await browser.newContext({ viewport: { width: 1280, height: 900 } })).newPage();
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

  // Original business markers (clock tab default; five tab pills).
  const body = await page.textContent('body');
  for (const marker of ['时钟 Clock', '秒表', '计时器', '世界时钟', '闹钟']) {
    check(`content marker: ${marker}`, body.includes(marker));
  }
  await page.screenshot({ path: path.join(outDir, '012_vue_initial.png'), fullPage: true });

  // Theme 2x2 with effective package values (default Auto/light boot).
  check('boot Auto/light bg = 0 0% 100% (no explicit default)', (await themeRoot()) === '0 0% 100%');

  // Timer runs; theme switch mid-run must not reset or re-speed the tick.
  // (Default setting is 1s — extend to 5s so the run spans the theme switch.)
  await page.getByRole('button', { name: '计时器', exact: true }).click();
  await page.waitForTimeout(400);
  for (let i = 0; i < 4; i++) {
    await page.getByRole('button', { name: '秒+', exact: true }).click();
  }
  await page.getByRole('button', { name: '开始', exact: true }).click();
  await page.waitForTimeout(1200);
  const onBefore = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    const unwrap = v => (v && typeof v === 'object' && 'value' in v) ? v.value : v;
    return { on: String(unwrap(st.t_on)), set: String(unwrap(st.t_set_s)) };
  });
  check('timer running before theme switch', onBefore.on === 'true');

  await drive('Solarized', true);
  await page.waitForTimeout(400);
  check('Solarized/dark bg = 192 100% 11% (#002b36)', (await themeRoot()) === '192 100% 11%');
  await page.screenshot({ path: path.join(outDir, '012_vue_solarized_dark_timer.png'), fullPage: true });
  const onAfter = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    const unwrap = v => (v && typeof v === 'object' && 'value' in v) ? v.value : v;
    return { on: String(unwrap(st.t_on)), set: String(unwrap(st.t_set_s)) };
  });
  check('timer still running after theme switch (state preserved)',
    onAfter.on === 'true' && onAfter.set === onBefore.set);
  await page.waitForTimeout(3200);
  const bannerBody = await page.textContent('body');
  check('timer reaches zero and banner appears across theme switch',
    bannerBody.includes('时间到'));

  // Dismiss, back to world clock; family round-trip.
  const dismiss = page.getByRole('button', { name: '知道了', exact: true });
  if (await dismiss.count()) { await dismiss.click(); }
  await page.waitForTimeout(300);
  await page.getByRole('button', { name: '世界时钟', exact: true }).click();
  await page.waitForTimeout(1200);
  const worldBody = await page.textContent('body');
  check('world clock rows render (Beijing/London)', worldBody.includes('Beijing') && worldBody.includes('London'));

  await drive('Solarized', false);
  await page.waitForTimeout(400);
  check('Solarized/light bg = 44 87% 94% (#fdf6e3)', (await themeRoot()) === '44 87% 94%');
  await page.screenshot({ path: path.join(outDir, '012_vue_solarized_light.png'), fullPage: true });
  await drive('Auto', true);
  await page.waitForTimeout(400);
  check('Auto/dark bg = 240 10% 4%', (await themeRoot()) === '240 10% 4%');
  await drive('Auto', false);
  await page.waitForTimeout(400);
  check('round-trip back to Auto/light', (await themeRoot()) === '0 0% 100%');
  check('world clock content preserved after round-trip',
    (await page.textContent('body')).includes('Beijing'));

  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    process.exit(1);
  }
  console.log('[+] All 012 Vue assertions passed!');
}

main().catch(err => {
  console.error('[-] Error in 012 Vue runner:', err);
  process.exit(1);
});
