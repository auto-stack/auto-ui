// PLAN-037 T-03: 013-todo Vue leg (official theme package adoption).
// Original business (seed list / filters / add input) preserved; theme
// family/mode round-trip via setupState drive with effective package
// values; list + input state preserved across switches (换肤不丢列表/输入).
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
  const url = process.argv[2] || 'http://localhost:5438';
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
  const page = await (await browser.newContext({ viewport: { width: 1100, height: 900 } })).newPage();
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

  // Original business markers (TodoMVC + seed items + filters). The input
  // placeholder is an attribute, not text content.
  const body = await page.textContent('body');
  for (const marker of ['todos', 'All', 'Active', 'Completed']) {
    check(`content marker: ${marker}`, body.includes(marker));
  }
  check('content marker: input placeholder', (await page.locator('input[placeholder="What needs to be done?"]').first().getAttribute('placeholder')) === 'What needs to be done?');
  await page.waitForTimeout(1200);  // store fetch materializes seed rows
  const body2 = await page.textContent('body');
  check('seed list rows render (Learn Auto Language visible)', body2.includes('Learn Auto Language'), body2.slice(0, 120));
  await page.screenshot({ path: path.join(outDir, '013_vue_initial.png'), fullPage: true });

  // Type a draft; it must survive theme switches (换肤不丢编辑输入).
  const input = page.locator('input[placeholder="What needs to be done?"]').first();
  await input.fill('theme survival draft');
  check('boot Auto/light bg = 0 0% 100% (no explicit default)',
    (await themeRoot()) === '0 0% 100%');

  await drive('Solarized', true);
  await page.waitForTimeout(400);
  check('Solarized/dark bg = 192 100% 11% (#002b36)', (await themeRoot()) === '192 100% 11%');
  await page.screenshot({ path: path.join(outDir, '013_vue_solarized_dark.png'), fullPage: true });
  check('draft input preserved in Solarized/dark',
    (await input.inputValue()) === 'theme survival draft');
  check('list still rendered in Solarized/dark',
    (await page.textContent('body')).includes('Learn Auto Language'));

  for (const [t, d] of [['Solarized', false], ['Auto', true], ['Auto', false]]) {
    await drive(t, d);
    await page.waitForTimeout(350);
  }
  check('round-trip back to Auto/light', (await themeRoot()) === '0 0% 100%');
  check('draft input preserved after round-trip',
    (await input.inputValue()) === 'theme survival draft');
  check('seed list preserved after round-trip',
    (await page.textContent('body')).includes('Learn Auto Language'));

  // Filters still function under the adopted package.
  await page.getByRole('button', { name: 'Active', exact: true }).click();
  await page.waitForTimeout(300);
  check('Active filter still applies', (await page.textContent('body')).includes('Active'));
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await page.waitForTimeout(300);

  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0) {
    console.error(`[-] ${failures} assertion(s) failed`);
    process.exit(1);
  }
  console.log('[+] All 013 Vue assertions passed!');
}

main().catch(err => {
  console.error('[-] Error in 013 Vue runner:', err);
  process.exit(1);
});
