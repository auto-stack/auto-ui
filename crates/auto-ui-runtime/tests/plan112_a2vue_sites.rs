//! PLAN-112 T-02 / T-06（025-sys-monitor A2Vue 腿）：lane 产品 → 可构建站点 →
//! 冷构建 → 真实浏览器。
//!
//! 站点 = A2Vue lane 产品（`generate_sfcs_with_lane_product_stores`：App +
//! 两张子视图 + `SysStore` composable，**原样落盘、零改写**）+ 官方 TS API
//! 客户端（`auto_web::generate_api`，同一单算法自原样 `src/back/api.at` 生成）
//! + 最小 vite/tailwind 脚手架。`/api/*` 由测试服务器按 standalone `auto run`
//! 的 vite 代理合同转到 `back_proxy` 的 `/apps/025-sys-monitor/api/*`。
//!
//! 三档：
//! - `p112_a2vue_site_generates_from_original_sources`（日常，进程内）：生成面
//!   断言——不经构建、不碰 node_modules。
//! - `p112_a2vue_site_cold_build`（显式构建验证道）：冷 `vite build`。
//! - `p112_a2vue_sys_monitor_browser_e2e`（浏览器验证道）：真 Chrome + 真
//!   `back_proxy`（原样后端 + `SysKillPolicy::AllowedPids`）。
//!
//! 后两档 `#[ignore]` + `AUTO_UI_ACE_WEB_E2E=1` 双门——未排程时是 ignored 而
//! 不是静默 PASS：
//!   cargo nextest run -p auto-ui-runtime --features ace-bridge \
//!     --test plan112_a2vue_sites --run-ignored only --success-output immediate
//! 预装依赖：`P112_NODE_MODULES`（缺省 = 主检出 025 生成工程的 node_modules），
//! 浏览器驱动：`P112_PLAYWRIGHT`（缺省 = auto-os-config 的 playwright）。
//!
//! 破坏性操作只落在两个**隔离自建 node 子进程**上：授权的那个被真实结束，
//! 未授权的那个必须存活。

#![cfg(feature = "ace-bridge")]

use auto_ui_runtime::back_proxy::{
    start, BackProxyConfig, SessionSpec, SysKillPolicy, SYS_KILL_PERMISSION_DENIED,
};
use auto_ui_runtime::ui::ace_vue_script::generate_sfcs_with_lane_product_stores;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const APP: &str = "025-sys-monitor";
const TIER_KEY: &str = "AUTO_UI_ACE_WEB_E2E";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn app_dir() -> PathBuf {
    repo_root().join("examples/ui/025-sys-monitor")
}

fn sites_root() -> PathBuf {
    std::env::var("P112_SITES_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("D:/autostack/plan-artifacts/plan-112/sites"))
}

fn site_dir() -> PathBuf {
    sites_root().join(APP)
}

/// 官方 API 客户端的生成工作区（原样 `pac.at` + `src/back/api.at` 的最小拷贝
/// ——不在任何检出里生成产物）。
fn api_ws_dir() -> PathBuf {
    sites_root().join(format!("{APP}-api"))
}

fn node_modules_dir() -> PathBuf {
    std::env::var("P112_NODE_MODULES")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                "D:/autostack/auto-ui/examples/ui/025-sys-monitor/gen/front/vue/node_modules",
            )
        })
}

fn playwright_mjs() -> PathBuf {
    std::env::var("P112_PLAYWRIGHT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from("D:/autostack/auto-os-config/node_modules/playwright/index.mjs")
        })
}

fn node_bin() -> std::ffi::OsString {
    std::env::var_os("A2TS_NODE").unwrap_or_else(|| "node".into())
}

fn write(rel: &Path, content: &str) {
    std::fs::create_dir_all(rel.parent().expect("parent")).expect("mkdir");
    std::fs::write(rel, content).unwrap_or_else(|e| panic!("write {}: {e}", rel.display()));
}

/// 生成面的源绑定指纹：站点内除 `node_modules`/`dist` 外每个文件的 blake3。
fn sources_fingerprint(site: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, rel: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(entries) = std::fs::read_dir(rel) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == "node_modules" || name == "dist" || name == "shot.png" || name.starts_with("verify_p112") {
                continue;
            }
            if path.is_dir() {
                walk(root, &path, out);
            } else if let Ok(bytes) = std::fs::read(&path) {
                out.insert(
                    path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"),
                    blake3::hash(&bytes).to_hex().to_string(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(site, site, &mut out);
    out
}

const TAILWIND_CONFIG: &str = r#"module.exports = {
  darkMode: ['class'],
  content: ['./index.html', './App.vue', './components/**/*.vue'],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--border))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: { DEFAULT: 'hsl(var(--primary))', foreground: 'hsl(var(--primary-foreground))' },
        secondary: { DEFAULT: 'hsl(var(--secondary))', foreground: 'hsl(var(--secondary-foreground))' },
        destructive: { DEFAULT: 'hsl(var(--destructive))', foreground: 'hsl(var(--destructive-foreground))' },
        muted: { DEFAULT: 'hsl(var(--muted))', foreground: 'hsl(var(--muted-foreground))' },
        accent: { DEFAULT: 'hsl(var(--accent))', foreground: 'hsl(var(--accent-foreground))' },
        card: { DEFAULT: 'hsl(var(--card))', foreground: 'hsl(var(--card-foreground))' },
      },
      borderRadius: { lg: 'var(--radius)', md: 'calc(var(--radius) - 2px)', sm: 'calc(var(--radius) - 4px)' },
    },
  },
  plugins: [],
}
"#;

const POSTCSS_CONFIG: &str = "module.exports = { plugins: { tailwindcss: {}, autoprefixer: {} } }\n";

/// shadcn 令牌（indigo 强调色 = 生成器 ACCENT_PALETTE 的 `indigo`）；`<html class="dark">`
/// 承接 pac `theme: "dark"` 的声明默认。
const INDEX_CSS: &str = r#"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%; --foreground: 240 10% 4%;
    --card: 0 0% 100%; --card-foreground: 240 10% 4%;
    --primary: 239 84% 67%; --primary-foreground: 0 0% 100%;
    --secondary: 240 4.8% 95.9%; --secondary-foreground: 240 5.9% 10%;
    --muted: 240 4.8% 95.9%; --muted-foreground: 240 3.8% 46.1%;
    --accent: 240 4.8% 95.9%; --accent-foreground: 240 5.9% 10%;
    --destructive: 0 84.2% 60.2%; --destructive-foreground: 0 0% 98%;
    --border: 240 5.9% 90%; --radius: 0.5rem;
  }
  .dark {
    --background: 240 10% 4%; --foreground: 0 0% 98%;
    --card: 240 10% 4%; --card-foreground: 0 0% 98%;
    --primary: 239 84% 67%; --primary-foreground: 0 0% 100%;
    --secondary: 240 3.7% 15.9%; --secondary-foreground: 0 0% 98%;
    --muted: 240 3.7% 15.9%; --muted-foreground: 240 5% 64.9%;
    --accent: 240 3.7% 15.9%; --accent-foreground: 0 0% 98%;
    --destructive: 0 62.8% 30.6%; --destructive-foreground: 0 0% 98%;
    --border: 240 3.7% 15.9%;
  }
  * { border-color: hsl(var(--border)); }
  body { background-color: hsl(var(--background)); color: hsl(var(--foreground)); }
}
"#;

/// 生成站点源（幂等）：先摘掉 node_modules junction 再清目录（Rust 的
/// `remove_dir_all` 不跟随 reparse point，摘链是双保险）。不建 junction、不构建。
fn generate_site_sources() -> PathBuf {
    let site = site_dir();
    let nm = site.join("node_modules");
    match std::fs::symlink_metadata(&nm) {
        Ok(_) => {
            std::fs::remove_dir(&nm).expect("unlink node_modules junction");
        }
        Err(_) => {}
    }
    let _ = std::fs::remove_dir_all(&site);

    // ── A2Vue lane 产品（原样写盘）──
    let front = app_dir().join("src/front");
    let mut merged = String::new();
    for unit in ["app.at", "processes.at", "performance.at", "sys_store.at"] {
        merged.push_str(&std::fs::read_to_string(front.join(unit)).expect("front unit"));
        merged.push('\n');
    }
    let store_path = front.join("sys_store.at");
    let (multi, stores) = generate_sfcs_with_lane_product_stores(
        &merged,
        &store_path.to_string_lossy(),
        "./a2x_rt.mjs",
        "../a2x_rt.mjs",
    )
    .expect("025 lane product derives");
    write(&site.join("a2x_rt.mjs"), &multi.runtime_mjs);
    for w in &multi.widgets {
        let target = if w.name == "App" {
            site.join("App.vue")
        } else {
            site.join("components").join(format!("{}.vue", w.name))
        };
        write(&target, &w.sfc);
    }
    for s in &stores {
        write(&site.join("stores").join(format!("use{}.ts", s.name)), &s.code);
    }

    // ── 官方 TS API 客户端（自原样 src/back/api.at）──
    let ws = api_ws_dir();
    let _ = std::fs::remove_dir_all(&ws);
    std::fs::create_dir_all(ws.join("src/back")).expect("mkdir api workspace");
    std::fs::copy(app_dir().join("pac.at"), ws.join("pac.at")).expect("copy pac.at");
    std::fs::copy(front.with_file_name("back").join("api.at"), ws.join("src/back/api.at"))
        .expect("copy api.at");
    auto_web::generate_api(&ws, "vue").expect("official TS client generation");
    let api_ts = std::fs::read_to_string(ws.join("dist/src/lib/api.ts"))
        .expect("official api.ts");

    // ── 脚手架 ──
    write(&site.join("lib/api.ts"), &api_ts);
    write(&site.join("package.json"), &format!("{{\n  \"name\": \"p112-{APP}-site\",\n  \"private\": true,\n  \"type\": \"module\"\n}}\n"));
    write(&site.join("index.html"), "<!doctype html>\n<html lang=\"zh\" class=\"dark\">\n  <head><meta charset=\"utf-8\" /><title>System Monitor</title></head>\n  <body><div id=\"app\"></div><script type=\"module\" src=\"/main.js\"></script></body>\n</html>\n");
    write(&site.join("main.js"), "import './index.css'\nimport { createApp } from 'vue'\nimport App from './App.vue'\ncreateApp(App).mount('#app')\n");
    write(&site.join("index.css"), INDEX_CSS);
    write(&site.join("tailwind.config.cjs"), TAILWIND_CONFIG);
    write(&site.join("postcss.config.cjs"), POSTCSS_CONFIG);
    write(
        &site.join("vite.config.js"),
        "import { defineConfig } from 'vite'\nimport vue from '@vitejs/plugin-vue'\nimport { fileURLToPath, URL } from 'node:url'\nexport default defineConfig({ plugins: [vue()], resolve: { alias: { '@': fileURLToPath(new URL('./', import.meta.url)) } } })\n",
    );
    site
}

fn link_node_modules(site: &Path) {
    let nm = site.join("node_modules");
    if nm.join("vite/bin/vite.js").exists() {
        return;
    }
    let target = node_modules_dir();
    assert!(
        target.join("vite/bin/vite.js").exists(),
        "prepared node_modules missing at {} (set P112_NODE_MODULES)",
        target.display()
    );
    // junction 建在 plan-artifacts 的站点目录内（不在任何工作树内）。
    let out = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &nm.to_string_lossy().replace('/', "\\"),
            &target.to_string_lossy().replace('/', "\\"),
        ])
        .output()
        .expect("spawn mklink /J");
    assert!(
        out.status.success(),
        "mklink /J failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_vite_build(site: &Path) -> Duration {
    let started = Instant::now();
    let out = Command::new(node_bin())
        .arg("node_modules/vite/bin/vite.js")
        .arg("build")
        .current_dir(site)
        .output()
        .expect("run vite build");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "vite build failed:\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
    );
    eprintln!("[vite build]\n{stdout}");
    started.elapsed()
}

fn write_build_manifest(site: &Path) {
    let manifest = serde_json::to_string(&sources_fingerprint(site)).expect("manifest JSON");
    write(&site.join("dist/.p112-build.json"), &manifest);
}

/// 构建产物与当前站点源绑定：缺 `dist/` 或指纹不符 = 具名准备失败（不隐式重建）。
fn require_built_site() -> PathBuf {
    let site = site_dir();
    let manifest_path = site.join("dist/.p112-build.json");
    let bound = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<BTreeMap<String, String>>(&raw).ok());
    match bound {
        Some(recorded) if recorded == sources_fingerprint(&site) && site.join("dist/index.html").exists() => site,
        _ => panic!(
            "preparation failure: site build missing or stale for the current sources — run `p112_a2vue_site_cold_build` first (manifest: {})",
            manifest_path.display()
        ),
    }
}

/// 日常档：生成面（进程内，不构建、不碰 node_modules）。
#[test]
fn p112_a2vue_site_generates_from_original_sources() {
    let site = generate_site_sources();
    let read = |rel: &str| std::fs::read_to_string(site.join(rel)).unwrap_or_else(|_| panic!("read {rel}"));

    for expected in ["App.vue", "components/ProcessesView.vue", "components/PerformanceView.vue", "stores/useSysStore.ts"] {
        assert!(site.join(expected).is_file(), "lane product file {expected}");
    }
    let store = read("stores/useSysStore.ts");
    assert!(store.contains("import { system_snapshot, kill_process } from '@/lib/api'"));
    assert!(store.contains("await system_snapshot()") && store.contains("await kill_process("));

    // 官方客户端：两端点、相对 `/api/` 字面量（改由代理合同承接，不在产物里改写）。
    let api = read("lib/api.ts");
    for needle in [
        "export async function system_snapshot()",
        "export async function kill_process(pid: number)",
        "fetch(`/api/system/snapshot`",
        "fetch(`/api/system/kill`",
        "if (!response.ok) throw new Error(`HTTP ${response.status}`)",
    ] {
        assert!(api.contains(needle), "official api.ts lacks `{needle}`:\n{api}");
    }
    assert!(!api.contains("/apps/"), "standalone client keeps relative /api/ URLs");

    // 站点内零测试钩子/零产物改写。
    for (rel, code) in [("App.vue", read("App.vue")), ("lib/api.ts", api.clone())] {
        assert!(!code.contains("__AUTO_LAST_API_ERROR__"), "{rel} carries no test hook");
        assert!(!code.contains("__p112"), "{rel} carries no test hook");
    }
}

/// 显式构建验证道：冷 `vite build`（空 `dist/` 起步）。
#[test]
#[ignore = "构建档：需 AUTO_UI_ACE_WEB_E2E=1 + 预装 node_modules（P112_NODE_MODULES）"]
fn p112_a2vue_site_cold_build() {
    if std::env::var(TIER_KEY).ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set {TIER_KEY}=1; build tier)");
        return;
    }
    let site = generate_site_sources();
    link_node_modules(&site);
    let elapsed = run_vite_build(&site);
    eprintln!("cold vite build wall time: {elapsed:?}");

    let index = std::fs::read_to_string(site.join("dist/index.html")).expect("dist/index.html");
    assert!(index.contains("assets/"), "built entry references bundled assets");
    let assets: Vec<_> = std::fs::read_dir(site.join("dist/assets"))
        .expect("dist/assets")
        .flatten()
        .map(|e| e.path())
        .collect();
    let js: String = assets
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "js"))
        .map(|p| std::fs::read_to_string(p).expect("js"))
        .collect();
    for marker in ["后端正常", "后端离线", "结束进程", "CPU 使用率实时曲线", "localStorage.getItem", "clearInterval"] {
        assert!(js.contains(marker), "bundle lacks `{marker}`");
    }
    let css: String = assets
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "css"))
        .map(|p| std::fs::read_to_string(p).expect("css"))
        .collect();
    assert!(css.contains("--background") && css.contains("position:fixed"), "tailwind tokens + utilities compiled:\n{}", &css[..css.len().min(600)]);
    write_build_manifest(&site);
}

/// 隔离自建 node 子进程：占住 `mib` MiB 常驻内存，使它在进程表「内存 ↓」排序里
/// 稳定落进 UI 的前 120 行。Drop 兜底结束（只按自己的 pid）。
struct IsolatedNodeChild {
    child: Option<Child>,
}

impl IsolatedNodeChild {
    fn spawn_weighted(mib: usize) -> Self {
        let script = format!(
            "const b=Buffer.alloc({mib}*1024*1024,1);setInterval(()=>{{b[0]=(b[0]+1)&255;}},500);setTimeout(()=>{{}},60000);"
        );
        let child = Command::new(node_bin())
            .args(["-e", &script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated node child");
        Self { child: Some(child) }
    }

    fn pid(&self) -> i32 {
        self.child.as_ref().expect("child present").id() as i32
    }

    fn is_alive(&mut self) -> bool {
        self.child.as_mut().is_some_and(|c| matches!(c.try_wait(), Ok(None)))
    }

    fn wait_exited(&mut self, timeout: Duration) -> bool {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if let Some(child) = self.child.as_mut() {
                if let Ok(Some(_)) = child.try_wait() {
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        false
    }
}

impl Drop for IsolatedNodeChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn ephemeral_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind :0")
        .local_addr()
        .expect("local_addr")
        .port()
}

fn start_proxy() -> auto_ui_runtime::back_proxy::RunningProxy {
    start(BackProxyConfig {
        port: ephemeral_port(),
        sessions: vec![SessionSpec {
            app_id: APP.into(),
            back_entry: app_dir().join("src/back/api.at"),
        }],
        lazy_sessions: false,
        native_media: Vec::new(),
        native_photos: Vec::new(),
    })
    .expect("start back_proxy")
}

/// 真实浏览器腿（Playwright + 系统 Chrome）。判据见各 `check`：原 `tests/smoke.spec.ts`
/// 的适用 oracle（T1/T2/T4/T5/T6，T3 按现 UI 的右键流）+ 本 Plan 的权限拒绝
/// 具名可恢复 + 关闭释放刷新任务。失败与统计以末行 `RESULT {json}` 回传。
const VERIFY_MJS: &str = r#"import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const proxyPort = Number(process.env.P112_PROXY_PORT)
const grantedPid = Number(process.env.P112_GRANTED_PID)
const ungrantedPid = Number(process.env.P112_UNGRANTED_PID)
const root = path.resolve('dist')
const mime = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8' }
const stats = { snapshot: 0, kill200: 0, kill403: 0, lastKill403: null, lastKill200: null }

const server = http.createServer((req, res) => {
  const urlPath = (req.url || '/').split('?')[0]
  if (urlPath.startsWith('/api/')) {
    const chunks = []
    req.on('data', c => chunks.push(c))
    req.on('end', () => {
      const body = Buffer.concat(chunks)
      const up = http.request(
        // Content-Length 必带：back_proxy 的简易 HTTP 解析不读 chunked 体（缺它则
        // 请求体丢失、后端看到 pid=null）。
        { host: '127.0.0.1', port: proxyPort, path: `/apps/025-sys-monitor${req.url}`, method: req.method, headers: { 'content-type': req.headers['content-type'] || 'application/json', 'content-length': Buffer.byteLength(body) } },
        upRes => {
          const rs = []
          upRes.on('data', c => rs.push(c))
          upRes.on('end', () => {
            const raw = Buffer.concat(rs)
            if (urlPath.endsWith('/api/system/snapshot') && upRes.statusCode === 200) stats.snapshot++
            if (urlPath.endsWith('/api/system/kill')) {
              if (upRes.statusCode === 200) { stats.kill200++; stats.lastKill200 = raw.toString() }
              if (upRes.statusCode === 403) { stats.kill403++; stats.lastKill403 = { req: body.toString(), rsp: raw.toString() } }
            }
            const headers = { ...upRes.headers, 'content-type': upRes.headers['content-type'] || 'application/json' }
            delete headers.connection
            delete headers['transfer-encoding']
            res.writeHead(upRes.statusCode || 500, headers)
            res.end(raw)
          })
        }
      )
      up.on('error', e => { res.writeHead(502); res.end(String(e)) })
      if (body.length) up.write(body)
      up.end()
    })
    return
  }
  if (urlPath === '/favicon.ico') { res.writeHead(204); res.end(); return }
  const file = path.join(root, urlPath === '/' ? '/index.html' : urlPath)
  if (!file.startsWith(root) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end('Not Found'); return }
  res.writeHead(200, { 'content-type': mime[path.extname(file)] || 'application/octet-stream' })
  fs.createReadStream(file).pipe(res)
})
await new Promise(r => server.listen(0, '127.0.0.1', r))
const port = server.address().port

const pw = await import(pathToFileURL(process.env.P112_PLAYWRIGHT).href)
const browser = await pw.chromium.launch({ channel: 'chrome', headless: true })
const context = await browser.newContext({ viewport: { width: 1280, height: 900 } })
const page = await context.newPage()
const pageErrors = []
page.on('pageerror', e => pageErrors.push(String(e.message)))
// Vue 的事件包装把 async handler 的 rejection 送进框架错误处理（console.error），
// 不会成为 unhandledrejection——观察点取 console。
const consoleErrors = []
page.on('console', m => { if (m.type() === 'error') consoleErrors.push(m.text()) })

let failures = 0
const check = (name, ok, extra = '') => {
  console.log(`${ok ? '[+] OK' : '[-] FAIL'}: ${name}${extra ? ' :: ' + extra : ''}`)
  if (!ok) failures++
}
const bodyText = async () => (await page.locator('body').innerText()) || ''
const rows = () => page.locator('table > div').last().locator(':scope > div')
const dialogPid = async () => {
  const t = await page.locator('text=/确定结束 .*\\(PID \\d+\\)/').first().innerText()
  const m = t.match(/\(PID (\d+)\)/)
  return m ? Number(m[1]) : NaN
}
const ctxMenu = () => page.locator('div.fixed.inset-0')
const openKillDialogFor = async pid => {
  const n = await rows().count()
  for (let i = 0; i < n; i++) {
    const name = (await rows().nth(i).locator(':scope > div').first().innerText()).trim()
    if (name !== 'node.exe') continue
    await rows().nth(i).click({ button: 'right' })
    await ctxMenu().getByText('结束进程', { exact: true }).waitFor({ state: 'visible', timeout: 1500 })
    await ctxMenu().getByText('结束进程', { exact: true }).click()
    await page.locator('text=/确定结束 .*\\(PID \\d+\\)/').first().waitFor({ state: 'visible', timeout: 1500 })
    const shown = await dialogPid()
    if (shown === pid) return true
    await page.getByText('取消', { exact: true }).click()
  }
  return false
}
const cancelDialog = async () => {
  await page.getByText('取消', { exact: true }).click()
  await page.waitForTimeout(80)
}

await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: 'domcontentloaded' })
// `backend_ok` 初值即 "true"——「后端正常」不能当数据到达信号；等真实进程行出现。
await rows().first().waitFor({ state: 'attached', timeout: 8000 })

// ── 原 oracle（tests/smoke.spec.ts）──
let body = await bodyText()
check('T1 壳与 2 页导航', ['系统监视器', 'Task Manager', '进程', '性能'].every(m => body.includes(m)))
check('T1 退役页负 pin', !body.includes('详细信息视图') && !body.includes('已登录与系统用户账户'))
check('T2 KPI 四卡', ['CPU', '内存', '磁盘', '网络'].every(m => body.includes(m)))
check('T2 真实 sysinfo 值（已用 N MB, N>0）', /已用 [1-9]\d* MB/.test(body))
const initialRows = await rows().count()
check('T2 真实进程列表 > 0 行', initialRows > 5, `rows=${initialRows}`)
const snap0 = stats.snapshot
await page.waitForTimeout(1300)
check('真实采集随刷新出现（轮询在跑）', stats.snapshot > snap0, `snapshots=${snap0}->${stats.snapshot}`)

// 表格静止后再定位行：暂停轮询。
await page.getByText('内存', { exact: true }).last().click()
await page.locator('button', { hasText: '⏸' }).click()
await page.waitForTimeout(350)
check('列排序：表头变为 内存 ↓', (await page.locator('table > div').first().innerText()).includes('内存 ↓'))

// ── T3（现 UI：右键结束 → 二次确认）：取消路径 ──
check('在进程表里定位到授权子进程行', await openKillDialogFor(grantedPid), `pid=${grantedPid}`)
await cancelDialog()
check('T3 取消后确认框复位（PID 0）', (await dialogPid()) === 0)
check('T3 取消后右键菜单关闭', (await ctxMenu().count()) === 0)

// ── 权限受限：未授权 pid 具名拒 + UI 可恢复 ──
check('在进程表里定位到未授权子进程行', await openKillDialogFor(ungrantedPid), `pid=${ungrantedPid}`)
const errorsBefore = consoleErrors.length
await page.getByText('结束进程', { exact: true }).click()
await page.waitForTimeout(250)
const denial = stats.lastKill403 ? JSON.parse(stats.lastKill403.rsp) : null
check('未授权结束请求被宿主策略拒绝（HTTP 403）', stats.kill403 === 1, JSON.stringify(stats.lastKill403))
check('拒绝具名：code = sys.kill.permission_denied 且指向目标 pid', !!denial && denial.code === 'sys.kill.permission_denied' && denial.ok === false && denial.pid === ungrantedPid, JSON.stringify(denial))
const newErrors = consoleErrors.slice(errorsBefore)
// 一次拒绝在 console 留两条：浏览器网络层的 403 资源日志 + 应用层抛出的 `Error: HTTP 403`。
const appLayer403 = newErrors.filter(t => t.includes('Error: HTTP 403'))
check('拒绝以生成客户端的既有错误路径浮出（应用层恰一次 Error: HTTP 403）', appLayer403.length === 1, JSON.stringify(newErrors))
check('拒绝后确认框仍指目标（handler 在 await 处中止）', (await dialogPid()) === ungrantedPid)
await cancelDialog()
check('UI 可恢复：取消后确认框复位', (await dialogPid()) === 0)
check('UI 可恢复：拒绝后应用仍显示「后端正常」', (await bodyText()).includes('后端正常'))

// ── 授权 pid：经后端真实结束 ──
check('重新定位授权子进程行', await openKillDialogFor(grantedPid), `pid=${grantedPid}`)
await page.getByText('结束进程', { exact: true }).click()
await page.waitForTimeout(250)
check('授权结束请求到达后端（HTTP 200 true）', stats.kill200 === 1 && stats.lastKill200 === 'true', `kill200=${stats.kill200} body=${stats.lastKill200}`)
check('结束后确认框复位（PID 0）', (await dialogPid()) === 0)

// ── 恢复轮询 + T4/T5/T6 ──
await page.locator('button', { hasText: '▶' }).click()
await page.getByText('性能', { exact: true }).click()
await page.waitForTimeout(200)
body = await bodyText()
check('T4 性能页实时曲线', body.includes('CPU 使用率实时曲线'))
check('T4 SVG path > 0', (await page.locator('svg path').count()) > 0, `paths=${await page.locator('svg path').count()}`)
await page.getByText('进程', { exact: true }).click()
await page.waitForTimeout(150)
check('T5 回到进程页业务保持', (await rows().count()) > 0)
await page.locator('button', { hasText: '250ms' }).click()
await page.locator('button', { hasText: '⏸' }).click()
await page.waitForTimeout(80)
check('T6 暂停后出现 ▶', (await page.locator('button', { hasText: '▶' }).count()) === 1)
await page.locator('button', { hasText: '▶' }).click()
await page.waitForTimeout(80)
check('T6 继续后出现 ⏸', (await page.locator('button', { hasText: '⏸' }).count()) === 1)

// ── T7（适用部分）：pac `theme: "dark"` 的声明默认 → 主题根 --background 为 dark 面。
// T7 的换肤驱动经 dev 构建的 `setupState`，生产包（inlineTemplate）不可达——具名边界。──
const themeBackground = await page.evaluate(() => getComputedStyle(document.querySelector('[data-auto-theme-root]')).getPropertyValue('--background').trim())
check('T7 声明默认 dark 主题的 --background', themeBackground === '240 10% 4%', themeBackground)

await page.screenshot({ path: 'shot.png' })

// ── 关闭：卸载后刷新任务不再发起请求 ──
await page.waitForTimeout(450)
const live0 = stats.snapshot
await page.waitForTimeout(550)
check('关闭前刷新在跑（250ms 档）', stats.snapshot > live0, `snapshots=${live0}->${stats.snapshot}`)
await page.evaluate(() => document.querySelector('#app').__vue_app__.unmount())
// 卸载瞬间在途的 fetch 先让其落定，再观察是否还有新请求。
await page.waitForTimeout(400)
const atClose = stats.snapshot
await page.waitForTimeout(900)
check('关闭后释放刷新任务（卸载后零快照请求）', stats.snapshot === atClose, `atClose=${atClose} now=${stats.snapshot}`)
check('关闭后页面已卸载', (await page.locator('#app').innerText()).trim() === '')

const unexpected = pageErrors.filter(m => !/(^|\W)HTTP 403$/.test(m))
check('无非预期的未捕获页面异常', unexpected.length === 0, JSON.stringify(pageErrors))
const otherConsole = consoleErrors.filter(t => !t.includes('403 (Forbidden)') && !t.includes('Error: HTTP 403'))
check('除该次 403 外无其他 console error', otherConsole.length === 0, JSON.stringify(otherConsole))

await browser.close()
server.close()
console.log(`RESULT ${JSON.stringify({ failures, stats, pageErrors, consoleErrors, unmountSnapshot: atClose })}`)
process.exit(failures === 0 ? 0 : 1)
"#;

/// 浏览器验证道：真 Chrome + 真 `back_proxy`。
#[test]
#[ignore = "浏览器档：需 AUTO_UI_ACE_WEB_E2E=1 + Chrome/Playwright + 预装 node_modules"]
fn p112_a2vue_sys_monitor_browser_e2e() {
    if std::env::var(TIER_KEY).ok().as_deref() != Some("1") {
        eprintln!("not scheduled (set {TIER_KEY}=1; browser tier)");
        return;
    }
    let site = require_built_site();
    let mut granted = IsolatedNodeChild::spawn_weighted(192);
    let mut ungranted = IsolatedNodeChild::spawn_weighted(192);
    // 让子进程的常驻缓冲落地后再被首次 sysinfo 采样。
    std::thread::sleep(Duration::from_millis(300));

    let proxy = start_proxy();
    proxy.set_sys_kill_policy(
        APP,
        SysKillPolicy::AllowedPids([granted.pid()].into_iter().collect()),
    );

    let script = site.join("verify_p112.mjs");
    write(&script, VERIFY_MJS);
    let started = Instant::now();
    let out = Command::new(node_bin())
        .arg(&script)
        .current_dir(&site)
        .env("P112_PROXY_PORT", proxy.port.to_string())
        .env("P112_GRANTED_PID", granted.pid().to_string())
        .env("P112_UNGRANTED_PID", ungranted.pid().to_string())
        .env("P112_PLAYWRIGHT", &playwright_mjs())
        .output()
        .expect("run verify_p112.mjs");
    let browser_leg = started.elapsed();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    eprintln!("=== browser leg (wall {browser_leg:?}) ===\n{stdout}");
    if !stderr.is_empty() {
        eprintln!("=== browser stderr ===\n{stderr}");
    }

    // 进程事实先于脚本结论落定——脚本失败时也能看到授权/未授权目标的真实状态。
    let granted_exited = granted.wait_exited(Duration::from_secs(3));
    let ungranted_alive = ungranted.is_alive();
    assert!(
        out.status.success() && stdout.contains("\"failures\":0"),
        "browser evidence failed (exit {:?})",
        out.status.code()
    );
    assert!(granted_exited, "granted isolated child must be terminated by the backend");
    assert!(ungranted_alive, "ungranted isolated child must survive the denied kill");

    // Rust 侧独立复核线上证据（不依赖脚本自报的 check 结论）。
    let result = stdout
        .lines()
        .find_map(|line| line.strip_prefix("RESULT "))
        .expect("RESULT line");
    let result: serde_json::Value = serde_json::from_str(result).expect("RESULT JSON");
    assert_eq!(result["stats"]["kill403"], 1, "exactly one denied kill");
    assert_eq!(result["stats"]["kill200"], 1, "exactly one granted kill");
    let denial: serde_json::Value =
        serde_json::from_str(result["stats"]["lastKill403"]["rsp"].as_str().expect("403 body"))
            .expect("403 body JSON");
    assert_eq!(denial["code"], SYS_KILL_PERMISSION_DENIED);
    assert_eq!(denial["pid"], ungranted.pid());
    assert_eq!(
        result["stats"]["lastKill403"]["req"],
        format!(r#"{{"pid":{}}}"#, ungranted.pid()),
        "the denied request targeted exactly the ungranted pid"
    );
    let app_layer_403 = result["consoleErrors"]
        .as_array()
        .map(|errors| {
            errors
                .iter()
                .filter(|e| e.as_str().is_some_and(|t| t.contains("Error: HTTP 403")))
                .count()
        })
        .expect("consoleErrors array");
    assert_eq!(
        app_layer_403, 1,
        "the denial surfaces exactly once through the generated client's error path: {result}"
    );
}
