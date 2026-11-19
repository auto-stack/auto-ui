// PLAN-980 lambda-if 刀浏览器档（排窗）：view 内联 lambda 体 if 形的 lane
// 派生拼接（`__age_vl_*` 值函数 If 整语句发射 → SFC `__evt_*` 拼接）在
// 真实浏览器点击中执行——toggle 语义：then 臂（a2x_add 增）与 else 臂
// （清零）都被真实走到。沿 #6k test_lane_browser_lambda.mjs 先例。
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
    console.error('usage: node test_lane_browser_lambda_if.mjs <dir-with-index.html>');
    process.exit(2);
  }
  const chromium = await resolvePlaywright();
  let browser;
  try {
    browser = await chromium.launch({ headless: true, channel: 'msedge' });
  } catch {
    try { browser = await chromium.launch({ headless: true, channel: 'chrome' }); }
    catch { browser = await chromium.launch({ headless: true }); }
  }
  const page = await (await browser.newContext()).newPage();
  const errors = [];
  page.on('pageerror', err => errors.push(err.message));

  const url = pathToFileURL(path.join(dir, 'index.html')).href;
  await page.goto(url);
  await page.waitForSelector('#toggle', { timeout: 5000 });

  const before = await page.textContent('#n');
  await page.click('#toggle');
  const afterThen = await page.textContent('#n');
  await page.click('#toggle');
  const afterElse = await page.textContent('#n');

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };
  check('n before=0', before === '0');
  check('n after-then=1 (then arm a2x_add runs on a real click)', afterThen === '1');
  check('n after-else=0 (else arm plain assign runs on the second click)', afterElse === '0');
  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0 || errors.length > 0) {
    console.error(`FAILURES: ${failures}, ERRORS: ${errors.join('; ')}`);
    process.exit(1);
  }
  console.log('LANE_LAMBDA_IF_BROWSER_OK');
}

main().catch(e => { console.error(e); process.exit(1); });
