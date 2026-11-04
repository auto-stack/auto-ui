// PLAN-047 T-03/T-04: 024-charts Vue leg (official theme package adoption;
// first Vue regression entry for this example — B2 016 首建入口先例).
// F-9 定位修定的数据面腿落位：axisLabels/donutLegend/statsD 是 484 组件化前
// App 状态契约，VM 快照/状态对 SVG 内图例不可文本观测（desktop_mcp.py 头注
// 实证）——数据值/图例/t 系列/seed 恢复 7 腿在本入口按同不变式断言；主题腿
// 经 setupState 驱动 + [data-auto-theme-root] CSS-var 逐值 oracle（B2 C-03）。
// Usage: node test_024_vue.mjs [url]   (url 默认 http://localhost:4024)
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
  const url = process.argv[2] || 'http://localhost:4024';
  const here = path.resolve(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([a-zA-Z]:)/, '$1')));
  const goldenDir = path.join(here, 'golden');
  fs.mkdirSync(goldenDir, { recursive: true });

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
  const page = await (await browser.newContext({ viewport: { width: 1200, height: 900 } })).newPage();
  const errors = [];
  page.on('pageerror', err => errors.push(err.message));

  let failures = 0;
  const check = (name, ok, detail = '') => {
    if (ok) { console.log(`[+] OK: ${name}`); }
    else { failures++; console.log(`[-] FAIL: ${name} ${String(detail).slice(0, 120)}`); }
  };

  const setup = () => page.evaluate(() =>
    document.querySelector('#app').__vue_app__._instance.setupState);
  const drive = (theme, dark) => page.evaluate(([t, d]) => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    st.theme_name = t; st.dark_mode = d;
  }, [theme, dark]);
  const cssVar = (name) => page.evaluate(([n]) =>
    getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue(n).trim(), [name]);
  const press = async (label) => {
    await page.getByRole('button', { name: label }).first().click();
    await page.waitForTimeout(300);
  };

  console.log(`[*] Navigating to ${url}...`);
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(1200);

  // ---- F-9 数据面腿（同不变式，484 后观测点在 vue DOM/setupState）----
  const monthly = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return JSON.parse(JSON.stringify(st.monthly));
  });
  check('init labels Jan..Jun',
    monthly?.length === 6 && monthly[0].m === 'Jan' && monthly[5].m === 'Jun', monthly?.map(r => r.m));

  // donut 图例（现 UI 语义：4 槽图例按数据序渲染月份名；百分比在悬停
  // tip 里——"desktop 49%   mobile 32%   tablet 19%" 是 484 前 App 状态
  // donutLegend 退役契约，F-9 关闭口径=按现 UI 修定，不补退役 feature。
  // 固定种子 → 图例名确定性 → golden 逐字节）。
  await press('Donut');
  const bodyText = await page.locator('body').innerText();
  const legendOk = ['Jan', 'Feb', 'Mar', 'Apr'].every(m => bodyText.includes(m));
  check('donut legend renders seed slices', legendOk, bodyText.slice(0, 100));
  const legendNorm = (bodyText.match(/^(Jan|Feb|Mar|Apr|May|Jun|t\d+)$/gm) || []).join(' ');
  const goldenPath = path.join(goldenDir, 'donut_legend_vue.txt');
  if (process.env.UPDATE_GOLDEN === '1' || !fs.existsSync(goldenPath)) {
    fs.writeFileSync(goldenPath, legendNorm, 'utf8');
    check('golden donut_legend (recorded)', true);
  } else {
    const expected = fs.readFileSync(goldenPath, 'utf8');
    check('golden donut_legend', expected === legendNorm, `${expected} != ${legendNorm}`);
  }
  await press('Line');

  // 流式 t 系列 + seed 恢复
  await press('▶ Play');
  await page.waitForTimeout(2600);
  const slid = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return String(st.monthly?.[0]?.m ?? '');
  });
  // 滑窗语义：窗口保尾（最新 t 点在尾端），2.6s@400ms ≈ 6 tick → 月名
  // 从头部滑出、t 系列自尾进入（monthly.some(t*)）。
  const slidArr = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return JSON.parse(JSON.stringify(st.monthly)).map(r => String(r.m));
  });
  check('labels slid to t-series', slidArr.some(m => m.startsWith('t')), slidArr.join(','));
  await press('Reset');
  const resetState = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return { m: JSON.parse(JSON.stringify(st.monthly)), s: JSON.parse(JSON.stringify(st.seed)), tickN: st.tickN };
  });
  check('reset labels back to Jan..Jun', resetState.m?.[0]?.m === 'Jan', resetState.m?.map(r => r.m));
  check('reset stats back to seed', JSON.stringify(resetState.m) === JSON.stringify(resetState.s), 'monthly!=seed');

  // ---- PLAN-047 主题腿：两族×明暗 + CSS-var 逐值 + 业务守恒 ----
  const sentinel = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return { type: st.chartType, seed0: st.seed?.[0]?.desktop, tickN: st.tickN };
  });
  const expectations = {
    'Auto|false': '0 0% 100%', 'Auto|true': '240 10% 4%',
    'Solarized|false': '44 87% 94%', 'Solarized|true': '192 100% 11%',
  };
  for (const fam of ['Solarized', 'Auto']) {
    for (const dark of [false, true]) {
      await drive(fam, dark);
      await page.waitForTimeout(350);
      const st = await setup();
      const bg = await cssVar('--background');
      check(`drive ${fam}/${dark} applied`,
        String(st.theme_name) === fam && String(st.dark_mode) === String(dark) && bg === expectations[`${fam}|${dark}`],
        `theme_name=${st.theme_name} dark=${st.dark_mode} bg=${bg}`);
    }
  }
  const after = await page.evaluate(() => {
    const st = document.querySelector('#app').__vue_app__._instance.setupState;
    return { type: st.chartType, seed0: st.seed?.[0]?.desktop, tickN: st.tickN };
  });
  check('theme round-trip preserves business',
    sentinel.type === after.type && sentinel.seed0 === after.seed0 && sentinel.tickN === after.tickN,
    JSON.stringify(after));
  await drive('Auto', false);
  await page.waitForTimeout(300);

  check('no page errors', errors.length === 0, errors.join('; '));
  await browser.close();
  console.log(failures === 0 ? '\n==== 024 vue leg: ALL PASS ====' : `\n==== 024 vue leg: ${failures} FAILURES ====`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch(e => { console.error('[-] FATAL:', e.message); process.exit(1); });
