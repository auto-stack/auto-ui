// PLAN-980 WEB 浏览器挂载档（切片 #6h）：lane 派生 handler 体在真实
// 浏览器 reactive 宿主中的执行验证。沿 013-todo test_013_vue.mjs 先例
// （playwright 解析/无头 chromium/断言模式）。
//
// 输入：host.mjs（Rust 测试侧产出——含 lane 派生的 handler 体与 a2x_rt
// helpers 内联）+ index.html（挂载按钮与显示）。断言：点击后计数文本变化
// （lane 体在浏览器 reactive 系统中语义正确）。
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
    console.error('usage: node test_lane_browser.mjs <dir-with-index.html>');
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
  await page.waitForSelector('#count', { timeout: 5000 });

  const before = await page.textContent('#count');
  await page.click('#inc');
  const after = await page.textContent('#count');

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };
  check('count before=41', before === '41');
  check('count after=42 (lane handler body runs in browser)', after === '42');
  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0 || errors.length > 0) {
    console.error(`FAILURES: ${failures}, ERRORS: ${errors.join('; ')}`);
    process.exit(1);
  }
  console.log('LANE_BROWSER_OK');
}

main().catch(e => { console.error(e); process.exit(1); });
