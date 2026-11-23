const page = new AbortController()
export const fileBrowserSignal = page.signal
let session: Promise<string> | undefined

const close = () => page.abort()
const restore = (event: PageTransitionEvent) => {
  if (event.persisted && page.signal.aborted) window.location.reload()
}
window.addEventListener('pagehide', close)
window.addEventListener('pageshow', restore)
import.meta.hot?.dispose(() => {
  close()
  window.removeEventListener('pagehide', close)
  window.removeEventListener('pageshow', restore)
})

async function monitor(reader: ReadableStreamDefaultReader<string>) {
  try {
    while (!(await reader.read()).done) { /* drain the connection */ }
  } catch {
    // Closing the page or the launcher terminates this stream.
  } finally {
    close()
  }
}

async function openSession(): Promise<string> {
  const response = await globalThis.fetch('/__auto/file-browser/session', {
    method: 'POST', cache: 'no-store', signal: page.signal,
  })
  if (!response.ok || !response.body) throw new Error('File browser session is unavailable')
  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader()
  let start = ''
  while (!start.includes('\n')) {
    const { value, done } = await reader.read()
    if (done || start.length > 128) throw new Error('Invalid file browser session')
    start += value
  }
  const base = start.slice(0, start.indexOf('\n'))
  if (!/^\/apps\/file-session-[a-f0-9]{32}$/.test(base)) {
    await reader.cancel()
    throw new Error('Invalid file browser session')
  }
  void monitor(reader)
  return base
}

export async function fetchFileBackend(input: string, init?: RequestInit): Promise<Response> {
  if (!/^\/api(?:\/[a-zA-Z0-9_-]+)+(?:\?|$)/.test(input) || input.includes('#')) {
    throw new Error('Invalid file backend route')
  }
  const signal = init?.signal ? AbortSignal.any([page.signal, init.signal]) : page.signal
  signal.throwIfAborted()
  session ??= openSession().catch(error => { close(); throw error })
  const base = await session
  signal.throwIfAborted()
  return globalThis.fetch(`${base}${input}`, { ...init, cache: 'no-store', redirect: 'error', signal })
}

export const fileBrowserFetch: typeof fetch = (input, init) => {
  if (typeof input !== 'string' || !input.startsWith('/api/fs/')) {
    return globalThis.fetch(input, init)
  }
  return fetchFileBackend(input, init)
}

type JsonReviver = (key: string, value: unknown, context?: { source?: string }) => unknown
const parseExact: (text: string, reviver: JsonReviver) => unknown = JSON.parse

function exactInteger(key: string, value: unknown, context?: { source?: string }) {
  if ((key !== 'size' && key !== 'mtime') || typeof value !== 'number') return value
  if (Number.isSafeInteger(value)) return BigInt(value)
  const source = context?.source
  if (source !== undefined && /^-?\d+$/.test(source)) return BigInt(source)
  throw new Error('Directory metadata lost integer precision')
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

export async function readFileDirectory(path: string, signal: AbortSignal, includeExtension = false) {
  if (path === '') {
    const home = await fileBrowserFetch('/api/fs/home', { signal })
    if (!home.ok) return null
    const root: unknown = await home.json()
    if (typeof root !== 'string' || root === '') return null
    path = root
  }
  const response = await fileBrowserFetch(`/api/fs/list?${new URLSearchParams({ path })}`, { signal })
  if (!response.ok) return null
  const data = parseExact(await response.text(), exactInteger)
  signal.throwIfAborted()
  fileBrowserSignal.throwIfAborted()
  if (!isRecord(data) || data.ok !== true || typeof data.path !== 'string' || !Array.isArray(data.entries)) return null
  const entries = data.entries.map((entry: unknown) => {
    if (!isRecord(entry) || typeof entry.name !== 'string' || typeof entry.path !== 'string'
      || typeof entry.is_dir !== 'boolean' || typeof entry.size !== 'bigint' || typeof entry.mtime !== 'bigint'
      || entry.size !== BigInt.asUintN(64, entry.size) || entry.mtime !== BigInt.asIntN(64, entry.mtime)) {
      throw new Error('Invalid directory entry')
    }
    const row = { name: entry.name, path: entry.path, is_dir: entry.is_dir, size: entry.size, mtime: entry.mtime }
    if (includeExtension) {
      if (typeof entry.extension !== 'string') throw new Error('Invalid directory entry extension')
      return { ...row, extension: entry.extension }
    }
    return row
  })
  return { path: data.path, entries }
}

export async function applyFileMutation(operation: string, path: string, target: string, signal: AbortSignal) {
  const response = await fileBrowserFetch('/api/fs/mutate', {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ operation, path, target }), signal,
  })
  return response.ok && (await response.json()) === true
}
