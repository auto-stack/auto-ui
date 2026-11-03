// PLAN-037 T-02: 014-weather Vue leg (official theme package adoption).
// Original business (city select / forecast tabs / layout toggle / demo
// data) preserved; theme family/mode round-trip via setupState model drive
// with effective package values; ToggleTheme handler keeps working.
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
  const url = process.argv[2] || 'http://localhost:5435';
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
  const page = await (await browser.newContext({ viewport: { width: 1100, height: 760 } })).newPage();
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

  // Original demo-business markers (weather demo data, not a real product API).
  const body = await page.textContent('body');
  for (const marker of ['北京', '24小时', '5日', '刷新']) {
    check(`content marker: ${marker}`, body.includes(marker));
  }
  await page.screenshot({ path: path.join(outDir, '014_vue_initial.png'), fullPage: true });

  // Theme 2x2 with effective package values.
  check('boot family Auto (theme_name model default)',
    (await page.evaluate(() => {
      const st = document.querySelector('#app').__vue_app__._instance.setupState;
      return String(st.theme_name);
    })) === 'Auto');

  await drive('Solarized', false);
  await page.waitForTimeout(400);
  check('Solarized/light bg = 44 87% 94% (#fdf6e3)', (await themeRoot()) === '44 87% 94%');
  await page.screenshot({ path: path.join(outDir, '014_vue_solarized_light.png'), fullPage: true });

  // City select survives a theme switch.
  await page.getByRole('button', { name: '上海', exact: true }).click();
  await page.waitForTimeout(400);
  await drive('Solarized', true);
  await page.waitForTimeout(400);
  check('Solarized/dark bg = 192 100% 11% (#002b36)', (await themeRoot()) === '192 100% 11%');
  const cityAfter = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return String(st.city_id);
  });
  check('city selection preserved across theme switch', cityAfter === 'shanghai');
  check('forecast tabs still present after switch',
    (await page.textContent('body')).includes('5日'));
  await page.screenshot({ path: path.join(outDir, '014_vue_solarized_dark_shanghai.png'), fullPage: true });

  // In-app ToggleTheme still drives the mode under the design binding.
  const toggle = page.getByRole('button', { name: '☀️', exact: true });
  const toggleAlt = page.getByRole('button', { name: '🌙', exact: true });
  const btn = (await toggle.count()) ? toggle : toggleAlt;
  if (await btn.count()) {
    const dmBefore = await page.evaluate(() => {
      const st = document.querySelector('#app').__vue_app__._instance.setupState;
      return !!st.dark_mode;
    });
    await btn.click();
    await page.waitForTimeout(400);
    const dmAfter = await page.evaluate(() => {
      const st = document.querySelector('#app').__vue_app__._instance.setupState;
      return !!st.dark_mode;
    });
    check('ToggleTheme flips dark_mode under design binding', dmBefore !== dmAfter);
  } else {
    check('theme toggle button found', false);
  }

  // Round-trip back to boot defaults.
  await drive('Auto', true);
  await page.waitForTimeout(300);
  check('Auto/dark bg = 240 10% 4%', (await themeRoot()) === '240 10% 4%');
  await drive('Auto', false);
  await page.waitForTimeout(300);
  check('round-trip Auto/light bg = 0 0% 100%', (await themeRoot()) === '0 0% 100%');
  check('content preserved after round-trip',
    (await page.textContent('body')).includes('上海'));

  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    process.exit(1);
  }
  console.log('[+] All 014 Vue assertions passed!');
}

main().catch(err => {
  console.error('[-] Error in 014 Vue runner:', err);
  process.exit(1);
});
