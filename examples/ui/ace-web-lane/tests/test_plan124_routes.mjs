import assert from 'node:assert/strict';
import { createReadStream } from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const [caseName, bundleDir, playwright, channel] = process.argv.slice(2);
assert.ok(['reader', 'kanban'].includes(caseName) && bundleDir && playwright,
  'usage: node test_plan124_routes.mjs reader|kanban <prepared web dir> <playwright/index.mjs> [channel]');
const { chromium } = await import(pathToFileURL(path.resolve(playwright)).href);
const server = http.createServer((req, res) => {
  const files = {
    '/': ['index.html', 'text/html; charset=utf-8'],
    '/app.js': ['app.js', 'text/javascript; charset=utf-8'],
  };
  const file = files[req.url];
  if (!file) {
    res.writeHead(404).end();
    return;
  }
  res.setHeader('Content-Type', file[1]);
  const stream = createReadStream(path.join(bundleDir, file[0]));
  stream.on('error', () => res.writeHead(500).end());
  stream.pipe(res);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
  browser = await chromium.launch({ headless: true, timeout: 2500, channel });
  let context = await browser.newContext();
  let page = await context.newPage();
  page.setDefaultTimeout(1300);
  const errors = [];
  page.on('pageerror', error => { errors.push(error.message); console.error(error); });
  const open = async () => {
    await page.goto(base, { waitUntil: 'domcontentloaded', timeout: 1700 });
  };
  const location = async (expected, text) => {
    await page.waitForFunction(value => window.location.hash === '#' + value, expected);
    await page.locator('.age-sfc .age-sfc').getByText(text, { exact: false }).waitFor();
  };
  const state = () => page.locator('.age-sfc .age-sfc').evaluate(
    el => el.__vueParentComponent.exposed.__state(),
  );
  const click = (role, name) => page.getByRole(role, { name, exact: true }).click();

  await open();
  if (caseName === 'kanban') {
    await location('/', 'Board page');
    assert.deepEqual(await state(), { moved: 0 });
    await click('button', 'Move card');
    assert.deepEqual(await state(), { moved: 1 });
  } else {
    await location('/', 'Bookshelf page');
    await click('button', 'Open book');
    await location('/book/17', 'Book detail page');
    await click('button', 'Read again');
    assert.deepEqual(await state(), { book_id: 17, total: 17 });
    const released = await page.getByRole('button', { name: 'Back', exact: true }).elementHandle();
    await click('link', 'Book 42');
    await location('/book/42', 'Book detail page');
    assert.deepEqual(await state(), { book_id: 42, total: 0 });
    await released.evaluate(element => element.click());
    await page.waitForFunction(() => window.location.hash === '#/book/42');
    await click('button', 'Back');
    await location('/book/17', 'Book detail page');

    for (const [label, route, diagnostic] of [
      ['Missing', '/missing', 'module is missing'],
      ['Broken', '/broken', 'module has no page widget'],
      ['Unmatched', '/unknown', 'ace.route.no-match: no route for /unknown'],
    ]) {
      await click('link', label);
      await page.waitForFunction(value => window.location.hash === '#' + value, route);
      await page.getByText(diagnostic, { exact: false }).waitFor();
    }
    await click('link', 'Chapter eight');
    await location('/book/23/chapter/8', 'Chapter page');
    await click('button', 'Score chapter');
    assert.deepEqual(await state(), { score: 77 });

    await click('link', 'Settings');
    await location('/settings', 'Settings page');
    await click('link', 'Library');
    await location('/', 'Bookshelf page');

    await context.close();
    context = await browser.newContext();
    page = await context.newPage();
    page.setDefaultTimeout(1300);
    page.on('pageerror', error => { errors.push(error.message); console.error(error); });
    await open();
    await location('/', 'Bookshelf page');
    await click('link', 'Book 17');
    await location('/book/17', 'Book detail page');
    assert.deepEqual(await state(), { book_id: 17, total: 0 });
  }
  assert.deepEqual(errors, []);
  await context.close();
  console.log(`PLAN124_BROWSER_OK ${caseName}`);
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
