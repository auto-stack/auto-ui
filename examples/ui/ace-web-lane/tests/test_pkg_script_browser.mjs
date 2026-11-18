// PLAN-980 ④-A 包闭包·浏览器执行档：SFC `<script>` 段的 helper+调用在真实
// 浏览器中执行（同一份发射文本；不是另写的等价 JS）。
//
// 用法：node test_pkg_script_browser.mjs <dir-with-index.html> <before> <after>
// 断言：点击 #step 后 #count 文本从 <before> 变为 <after>；无 page errors。
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

async function resolvePlaywright() {
  const possiblePaths = [
    'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs',
    'd:/autostack/auto-os-config/node_modules/playwright/index.mjs',
    'd:/autostack/auto-down/autodown/node_modules/playwright/index.mjs',
    'playwright',
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
  const wantBefore = process.argv[3];
  const wantAfter = process.argv[4];
  if (!dir || !wantBefore || !wantAfter || !fs.existsSync(path.join(dir, 'index.html'))) {
    console.error('usage: node test_pkg_script_browser.mjs <dir> <before> <after>');
    process.exit(2);
  }
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
  const page = await (await browser.newContext()).newPage();
  const errors = [];
  page.on('pageerror', (err) => errors.push(err.message));
  await page.goto(pathToFileURL(path.join(dir, 'index.html')).href);
  await page.waitForSelector('#count', { timeout: 5000 });

  const before = await page.textContent('#count');
  await page.click('#step');
  const after = await page.textContent('#count');

  let failures = 0;
  const check = (name, ok) => {
    if (ok) console.log(`[+] OK: ${name}`);
    else {
      failures++;
      console.log(`[-] FAIL: ${name}`);
    }
  };
  check(`count before=${wantBefore}`, before === wantBefore);
  check(
    `count after=${wantAfter} (emitted helper call ran in the browser)`,
    after === wantAfter
  );
  check('no page errors', errors.length === 0);

  await browser.close();
  if (failures > 0 || errors.length > 0) {
    console.error(`FAILURES: ${failures}, ERRORS: ${errors.join('; ')}`);
    process.exit(1);
  }
  console.log('PKG_SCRIPT_BROWSER_OK');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
