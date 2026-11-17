// PLAN-980 WEB 切片#6j 浏览器档（param handler 参数环境）：lane 派生的
// 带参 handler 体在真实浏览器事件中实读参。沿 #6h test_lane_browser.mjs
// 先例（playwright 解析/无头 chromium/断言模式）。
//
// 输入：<dir>/index.html（Rust 测试侧产出——lane 派生体 + a2x_rt helpers
// 内联 + reactive 宿主）。断言：①str 输入事件（QueryInput 形——输入文本
// 经形参流入状态）；②多载荷点击（3+4=7——两形参都参与求值）；③bool
// 点击（true 生效）。
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
    console.error('usage: node test_lane_browser_params.mjs <dir-with-index.html>');
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
  await page.waitForSelector('#q', { timeout: 5000 });

  // ① str 输入事件：真实 DOM input → 形参 q → last 状态 → DOM 文本。
  await page.fill('#q', 'hello-lane');
  const last = await page.textContent('#last');
  // ② 多载荷点击：真实 click → (3, 4) → dx+dy。
  await page.click('#move');
  const x = await page.textContent('#x');
  // ③ bool 点击：true → dark。
  await page.click('#dark-btn');
  const dark = await page.textContent('#dark');

  let failures = 0;
  const check = (name, ok) => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name}`); }
  };
  check('str param flows from a real input event', last === 'hello-lane');
  check('both int params read (3+4=7)', x === '7');
  check('bool param read (true)', dark === 'true');
  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0 || errors.length > 0) {
    console.error(`FAILURES: ${failures}, ERRORS: ${errors.join('; ')}`);
    process.exit(1);
  }
  console.log('LANE_PARAM_BROWSER_OK');
}

main().catch(e => { console.error(e); process.exit(1); });
