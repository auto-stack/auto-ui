// PLAN-980 WEB 切片#6k 浏览器档（V-EVT-B-ORIGIN）：view 内联 lambda 的
// lane 派生体（`__age_vl_*` 值函数发射、SFC 合成 `__evt_*` 拼接）在真实
// 浏览器点击事件中执行。沿 #6h/#6j 先例（playwright/无头 chromium）。
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
  const dir = process.argv[2];
  if (!dir || !fs.existsSync(path.join(dir, 'index.html'))) {
    console.error('usage: node test_lane_browser_lambda.mjs <dir-with-index.html>');
    process.exit(2);
  }
  const chromium = await resolvePlaywright();
  let browser;
  try {
    browser = await chromium.launch({ headless: true, channel: 'msedge' });
  } catch (_) {
    try { browser = await chromium.launch({ headless: true, channel: 'chrome' }); }
    catch (_) { browser = await chromium.launch({ headless: true }); }
  }
  const page = await (await browser.newContext()).newPage();
  const errors = [];
  page.on('pageerror', err => errors.push(err.message));

  const url = pathToFileURL(path.join(dir, 'index.html')).href;
  await page.goto(url);
  await page.waitForSelector('#inc', { timeout: 5000 });

  const before = await page.textContent('#n');
  await page.click('#inc');
  const after = await page.textContent('#n');

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };
  check('n before=5', before === '5');
  check('n after=6 (lambda-origin handler body runs on a real click)', after === '6');
  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0 || errors.length > 0) {
    console.error(`FAILURES: ${failures}, ERRORS: ${errors.join('; ')}`);
    process.exit(1);
  }
  console.log('LANE_LAMBDA_BROWSER_OK');
}

main().catch(e => { console.error(e); process.exit(1); });
