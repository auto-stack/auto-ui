// PLAN-057 real-browser oracle: Vue dual-App store isolation & owner
// lifecycle (V-01..V-03). Usage:
//   node plan057_browser.mjs <front-url> <phase: single|dual>
//
// Owns the local SSE mock on the frozen fixture port 127.0.0.1:39579
// (loopback only): /events is the stream the generated store subscribes to,
// /emit broadcasts {"kind": ...} messages, /stats reports per-connection
// delivery accounting (open/closed/delivered) — the resource-exit oracle.
// Exits 0 only when every phase assertion passes; prints PLAN057-RESULT JSON.

import http from 'node:http'

const SSE_PORT = 39579
const [, , baseArg, phaseArg] = process.argv
const base = (baseArg || 'http://127.0.0.1:3000').replace(/\/$/, '')
const phase = phaseArg || 'dual'

// ── SSE mock ────────────────────────────────────────────────────────────
const conns = new Map() // id -> { id, res, delivered, closed }
let seq = 0
let rejected = 0

// Only accept streams from the phase's own front origin. Stale pages from
// earlier failed runs (zombie tabs) keep reconnecting to the frozen mock
// port forever — their Origin pins a dead port and is filtered here, so
// connection accounting stays exact for the live phase.
const expectOrigin = new URL(base).origin

function broadcast(kind) {
  const payload = `data: ${JSON.stringify({ kind })}\n\n`
  for (const rec of conns.values()) {
    if (!rec.closed) {
      rec.delivered += 1
      try { rec.res.write(payload) } catch { /* client vanished */ }
    }
  }
}

const mock = http.createServer((req, res) => {
  if (req.url === '/events') {
    if ((req.headers.origin || '') !== expectOrigin) {
      rejected += 1
      res.writeHead(403, { 'Access-Control-Allow-Origin': '*' })
      res.end()
      return
    }
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Access-Control-Allow-Origin': '*',
    })
    const rec = { id: ++seq, res, delivered: 0, closed: false }
    conns.set(rec.id, rec)
    res.write(':hi\n\n')
    req.on('close', () => { rec.closed = true })
  } else if (req.url === '/stats') {
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    })
    const list = [...conns.values()].map(({ id, delivered, closed }) => ({ id, delivered, closed }))
    res.end(JSON.stringify({ seq, open: list.filter((c) => !c.closed).length, conns: list }))
  } else if (req.url.startsWith('/emit')) {
    const kind = new URL(req.url, 'http://mock').searchParams.get('kind') || 'msg'
    broadcast(kind)
    res.writeHead(200, { 'Access-Control-Allow-Origin': '*' })
    res.end('ok')
  } else {
    res.writeHead(404)
    res.end()
  }
})
await new Promise((ok) => mock.listen(SSE_PORT, '127.0.0.1', ok))
const heartbeat = setInterval(() => {
  for (const rec of conns.values()) {
    if (!rec.closed) { try { rec.res.write(':hb\n\n') } catch { /* ignore */ } }
  }
}, 2000)

const stats = async () => await (await fetch(`http://127.0.0.1:${SSE_PORT}/stats`)).json()
const emit = async (kind) => { await fetch(`http://127.0.0.1:${SSE_PORT}/emit?kind=${kind}`) }
let lastStats = null

// ── browser ─────────────────────────────────────────────────────────────
const pwPath = (process.env.PLAN057_PLAYWRIGHT_PATH ||
  'd:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs')
  .replace(/\\/g, '/')
const { chromium } = await import('file:///' + pwPath.replace(/^\/+/, ''))

let browser
try {
  browser = await chromium.launch({ headless: true, channel: 'msedge' })
} catch {
  browser = await chromium.launch({ headless: true })
}
const page = await browser.newPage()
page.setDefaultTimeout(15000)
const pageErrors = []
page.on('pageerror', (e) => pageErrors.push(String(e)))

// Any failure must close the browser before exiting: an orphaned headless
// page keeps its EventSources reconnecting to the frozen mock port forever,
// polluting every later phase/run that serves the same origin.
process.on('exit', (code) => {
  if (code !== 0) { try { browser.process()?.kill() } catch { /* already gone */ } }
})

// ── helpers ─────────────────────────────────────────────────────────────
const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms))

async function waitUntil(label, fn, timeoutMs = 10000) {
  const deadline = Date.now() + timeoutMs
  let last
  while (Date.now() < deadline) {
    try { last = await fn(); if (last) return } catch { /* retry */ }
    await sleep(120)
  }
  throw new Error(`PLAN057 timeout waiting for: ${label} (last=${JSON.stringify(last)})`)
}

function assert(cond, msg) {
  if (!cond) throw new Error('PLAN057 assert FAILED: ' + msg)
}

// Read one mounted app's rendered stats from its container text. Readers are
// captured as a list — every entry must stay equal (same-App sharing).
async function readApp(page, sel) {
  const text = await page.locator(sel).textContent()
  const num = (k) => {
    const m = text.match(new RegExp(k + ':(-?\\d+)'))
    return m ? parseInt(m[1], 10) : null
  }
  const readers = [...text.matchAll(/r:(-?\d+)/g)].map((m) => parseInt(m[1], 10))
  // `last:` is the last stat line; the DOM text concatenates the button
  // labels ("w11inc") right after — cut there (kinds never contain "w11").
  const last = (text.match(/last:(.*?)w11/) || [])[1] || null
  return { v1: num('v1'), v2: num('v2'), readers, ticks: num('ticks'), sse: num('sse'), watch: num('watch'), last }
}

const A = (p) => readApp(p, '#app-a')
const B = (p) => readApp(p, '#app-b')
const S1 = (p) => readApp(p, '#app-s1')
const S2 = (p) => readApp(p, '#app-s2')
const S3 = (p) => readApp(p, '#app-s3')
const L1 = (p) => readApp(p, '#app-l1')
const L2 = (p) => readApp(p, '#app-l2')

async function click(page, sel, name) {
  await page.locator(sel).getByRole('button', { name, exact: true }).click()
}

// ═══ phase: single — cold-scaffold path, generated main.ts boots ═══════
if (phase === 'single') {
  await page.goto(base + '/')
  await waitUntil('single app renders v1:22', async () => (await readApp(page, '#app')).v1 === 22)
  const s0 = await readApp(page, '#app')
  assert(s0.v1 === 22 && s0.v2 === 22, `single: both direct views init 22 (got ${s0.v1}/${s0.v2})`)
  assert(s0.readers.length >= 2 && s0.readers.every((r) => r === 22), `single: readers init 22 (${s0.readers})`)

  await click(page, '#app', 'inc')
  await waitUntil('single: v1 advances to 23', async () => (await readApp(page, '#app')).v1 === 23)
  const s1 = await readApp(page, '#app')
  assert(s1.v2 === 23, 'single: second direct view follows 23')
  assert(s1.readers.every((r) => r === 23), `single: child readers follow 23 (${s1.readers})`)
  assert(s1.watch >= 1, 'single: store watcher fired on total change')

  const ticksBefore = s1.ticks
  await waitUntil('single: store timer ticks advance', async () => (await readApp(page, '#app')).ticks > ticksBefore, 8000)

  await emit('hello')
  await waitUntil('single: SSE message delivered', async () => (await readApp(page, '#app')).sse === 1)
  const s2 = await readApp(page, '#app')
  assert(s2.last === 'hello', `single: SSE payload routed to handler (last=${s2.last})`)

  assert(pageErrors.length === 0, `single: pageerror free (${pageErrors.join(' | ')})`)
  console.log('PLAN057-RESULT', JSON.stringify({ phase, ok: true, pageErrors: 0, zombieRejected: rejected }))
  await browser.close()
  clearInterval(heartbeat)
  mock.close()
  process.exit(0)
}

// ═══ phase: dual — isolation / explicit share / legacy / owner exit ════
await page.goto(base + '/dual.html')
await waitUntil('all five mounts render init 22', async () => {
  const [a, b, s1v, s2v, l1v] = await Promise.all([A(page), B(page), S1(page), S2(page), L1(page)])
  return [a, b, s1v, s2v, l1v].every((s) => s && s.v1 === 22)
})

// ── V-01: per-App isolation + same-App consumer sharing ────────────────
await click(page, '#app-a', 'w11')
await waitUntil('A writes 11', async () => (await A(page)).v1 === 11)
let a = await A(page)
assert(a.v2 === 11 && a.readers.every((r) => r === 11), `A same-App consumers all 11 (v2=${a.v2} readers=${a.readers})`)
assert(a.watch === 1, `A watcher fired exactly once (watch=${a.watch})`)
let b = await B(page)
assert(b.v1 === 22, `B untouched at 22 — module-singleton regression would share (b=${b.v1})`)
assert((await S1(page)).v1 === 22 && (await L1(page)).v1 === 22, 'S1/L1 untouched')

await click(page, '#app-b', 'inc')
await waitUntil('B increments to 23', async () => (await B(page)).v1 === 23)
b = await B(page)
assert(b.readers.every((r) => r === 23), `B readers follow 23 (${b.readers})`)
assert((await A(page)).v1 === 11, 'A still 11 after B write')

// ── V-02: explicit shared context syncs; legacy behavior preserved ─────
await click(page, '#app-s1', 'w11')
await waitUntil('S1 writes 11', async () => (await S1(page)).v1 === 11)
let s2v = await S2(page)
assert(s2v.v1 === 11, `S2 mirrors S1 through the explicit shared context (s2=${s2v.v1})`)
assert(s2v.watch === 1, `shared instance has ONE watcher across S1+S2 (watch=${s2v.watch})`)
assert((await A(page)).v1 === 11 && (await B(page)).v1 === 23, 'explicit share did not leak into per-App mounts')

await click(page, '#app-l1', 'w11')
await waitUntil('L1 writes 11 (legacy singleton)', async () => (await L1(page)).v1 === 11)
await page.evaluate(() => window.__p057.mountL2())
await waitUntil('L2 mounts and shares the legacy singleton', async () => (await L2(page)).v1 === 11)
assert((await A(page)).v1 === 11 && (await B(page)).v1 === 23, 'legacy singleton stays confined to legacy mounts')

// ── V-03: resource owners — one connection per live instance ───────────
await waitUntil('four live connections (A,B,S-shared,L-legacy)', async () => {
  const st = await stats()
  if (!(st.open === 4 && st.seq === 4)) {
    lastStats = st
    return false
  }
  return true
}, 10000).catch((e) => {
  throw new Error(`${e.message} :: stats=${JSON.stringify(lastStats)}`)
})
await emit('m1')
await waitUntil('SSE message m1 reaches every instance', async () => {
  const [av, bv, s1v, l1v] = await Promise.all([A(page), B(page), S1(page), L1(page)])
  return av.sse === 1 && bv.sse === 1 && s1v.sse === 1 && l1v.sse === 1
})
assert((await S1(page)).last === 'm1', 'SSE payload parsed and routed (last=m1)')

// A exits → its connection closes, others keep processing; late m2 never
// reaches the exited owner.
await page.evaluate(() => window.__p057.unmount('a'))
await waitUntil('A connection closed (3 open)', async () => (await stats()).open === 3)
let st = await stats()
assert(st.conns[0].closed === true, 'first connection (A) observed closed by the mock')
await emit('m2')
await waitUntil('m2 processed by surviving B', async () => (await B(page)).sse === 2)
st = await stats()
assert(st.conns[0].delivered === 1, `late m2 NOT delivered to exited A (delivered=${st.conns[0].delivered})`)
assert((await S1(page)).sse === 2 && (await L1(page)).sse === 2, 'S and legacy instances processed m2')

// Shared family: S1 exits — S2 still owns the connection.
await page.evaluate(() => window.__p057.unmount('s1'))
await waitUntil('S connection survives S1 exit (still 3 open)', async () => (await stats()).open === 3)
await emit('m3')
await waitUntil('S2 keeps processing after S1 exit', async () => (await S2(page)).sse === 3)

// Last shared owner exits → bounded release: connection closed, timer cleared.
const frozenProbe = async () => page.evaluate(() => window.__p057.sharedInst.tick_count.value)
const beforeRelease = await frozenProbe()
await page.evaluate(() => window.__p057.unmount('s2'))
await waitUntil('shared connection closed (2 open)', async () => (await stats()).open === 2)
await sleep(600)
const afterRelease = await frozenProbe()
assert(afterRelease === beforeRelease, `shared instance timer frozen after last-owner exit (${beforeRelease} → ${afterRelease})`)
const sseProbe = async () => page.evaluate(() => window.__p057.sharedInst.sse_count.value)
const sseBeforeLate = await sseProbe()
await emit('m4')
await sleep(400)
assert((await sseProbe()) === sseBeforeLate, 'released shared instance receives no late messages')

// Remount a fresh shared-family owner: exactly one new connection, zero
// stale values, zero double subscriptions.
await page.evaluate(() => window.__p057.mountS3())
await waitUntil('S3 mounts fresh at 22 (no stale values)', async () => (await S3(page)).v1 === 22)
const s3Before = await S3(page)
await waitUntil('S3 timer ticks fresh', async () => (await S3(page)).ticks > s3Before.ticks, 8000)
await waitUntil('exactly one new connection opened (5 total)', async () => {
  st = await stats()
  return st.seq === 5 && st.open === 3
})
await emit('m5')
await waitUntil('S3 processes m5', async () => (await S3(page)).sse === 1)
assert((await sseProbe()) === sseBeforeLate, 'old released instance still silent after remount')

await click(page, '#app-s3', 'w11')
await waitUntil('S3 writes 11', async () => (await S3(page)).v1 === 11)
assert((await S3(page)).watch === 1, `S3 has exactly one fresh watcher (watch=${(await S3(page)).watch})`)

// Legacy pair and B continue unaffected through the whole sequence.
assert((await L2(page)).v1 === 11 && (await B(page)).v1 === 23, 'legacy pair and B unaffected at the end')

assert(pageErrors.length === 0, `dual: pageerror free (${pageErrors.join(' | ')})`)
console.log('PLAN057-RESULT', JSON.stringify({ phase, ok: true, pageErrors: 0, connections: seq, zombieRejected: rejected }))
await browser.close()
clearInterval(heartbeat)
mock.close()
process.exit(0)
