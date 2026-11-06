//! PLAN-057 real-browser e2e: Vue dual-App store isolation & owner lifecycle.
//!
//! Heavy environment harness (node + pnpm + playwright/msedge + npm mirror):
//! run explicitly via the recorded plan command —
//!   cargo test -p auto-ui-tools --test plan057_vue_instances -- --ignored --nocapture
//!
//! Chain (plan §5/§6, V-01..V-04):
//!   1. cold scaffold path: fixture .at → `VueProject::generate()` (fresh dir)
//!      → string gates on the REAL generated main.ts / __autoStoreContext.ts /
//!      useP057Store.ts (dispose wiring present).
//!   2. overlay test-owned dual-mount entry (`dual.html` + `src/main-dual.ts`)
//!      — never hand-written store equivalents, only mount harness code.
//!   3. serve (vite dev, free loopback port, strictPort; SSE mock frozen at
//!      127.0.0.1:39579 inside the node oracle script).
//!   4. browser phase `single` — generated main.ts real mount (AC-04).
//!   5. run/start self-heal path: delete src/main.ts + vite.config.ts →
//!      `regenerate_source_files()` + `refresh_vite_config()` (050 two-path
//!      coverage) → serve again → browser phase `dual` (AC-01..03).
//!
//! Scratch evidence home: D:/autostack/plan-artifacts/plan-057/e2e — never
//! inside any repo checkout.

use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use auto_ui_tools::vue::{refresh_vite_config, VueProject};

/// Frozen SSE mock endpoint baked into the fixture store (plan §6: loopback,
/// dedicated port). The node oracle script owns the listener.
const SSE_PORT: u16 = 39579;
const PLAYWRIGHT_ENV: &str = "PLAN057_PLAYWRIGHT_PATH";
const DEFAULT_PLAYWRIGHT: &str =
    "d:/autostack/auto-lang/packages/auto-forge-ui/node_modules/playwright/index.mjs";

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-ui-runtime/tests/fixtures/plan057_vue_instances")
}

fn scratch_root() -> PathBuf {
    Path::new("D:/autostack/plan-artifacts/plan-057").join("e2e")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn free_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind :0")
        .local_addr()
        .expect("local_addr")
        .port()
}

fn wait_port(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

/// Spawn `pnpm run dev` (vite reads AUTO_FRONT_PORT; scaffold sets
/// strictPort: true so an occupied port fails loudly instead of drifting).
/// Output goes to a scratch log file — pnpm re-exposes inherited std handles
/// through the cmd chain, so `Stdio::null` does not prevent the long-lived
/// dev server from holding the caller's pipe open.
fn spawn_vite(out_dir: &Path, port: u16) -> ViteGuard {
    let log_path = scratch_root().join(format!("vite-{port}.log"));
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("open vite log file");
    let child = Command::new("cmd")
        .args(["/C", "pnpm", "run", "dev"])
        .current_dir(out_dir)
        .env("AUTO_FRONT_PORT", port.to_string())
        // Suppress vite's `open: !TAURI_ENV` — without this every dev-server
        // start plants a tab in the operator's real browser, and stale tabs
        // reconnect their EventSources to the frozen mock port forever.
        .env("TAURI_ENV", "1")
        .stdout(log.try_clone().expect("clone vite log handle"))
        .stderr(log)
        .spawn()
        .expect("spawn vite dev server");
    ViteGuard { pid: child.id(), port }
}

/// Kills the dev server on drop (also on test panic): tree-kill by pid, then
/// a by-port sweep for the case where pnpm re-parents the node process out
/// of the cmd tree.
struct ViteGuard {
    pid: u32,
    port: u16,
}

impl Drop for ViteGuard {
    fn drop(&mut self) {
        let _ = Command::new("taskkill")
            .args(["/PID", &self.pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        // By-port sweep: whatever still LISTENs on the dev port must die,
        // or it keeps this test's stdout pipe (and the port) hostage.
        if let Ok(out) = Command::new("netstat")
            .args(["-ano"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let needle = format!(":{}", self.port);
            for line in text.lines() {
                if line.contains(&needle) && line.contains("LISTENING") {
                    if let Some(pid) = line.split_whitespace().last() {
                        let _ = Command::new("taskkill")
                            .args(["/PID", pid, "/T", "/F"])
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status();
                    }
                }
            }
        }
    }
}

/// Run one browser oracle phase; the node script also hosts the SSE mock.
fn run_browser(phase: &str, port: u16) {
    let mjs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/plan057_browser.mjs");
    let status = Command::new("node")
        .arg(&mjs)
        .arg(format!("http://127.0.0.1:{port}"))
        .arg(phase)
        .env(PLAYWRIGHT_ENV, DEFAULT_PLAYWRIGHT)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("spawn node browser oracle");
    assert!(
        status.success(),
        "PLAN-057 browser phase `{phase}` FAILED (exit {status:?})"
    );
}

const DUAL_HTML: &str = r#"<!doctype html>
<html>
  <head><meta charset="UTF-8" /><title>plan057 dual</title></head>
  <body>
    <div id="app-a"></div>
    <div id="app-b"></div>
    <div id="app-s1"></div>
    <div id="app-s2"></div>
    <div id="app-l1"></div>
    <div id="app-l2"></div>
    <div id="app-s3"></div>
    <script type="module" src="/src/main-dual.ts"></script>
  </body>
</html>
"#;

/// Test-owned mount harness over the GENERATED modules only (App.vue +
/// stores/__autoStoreContext). Mount shapes: A/B per-App contexts, S1/S2 one
/// explicitly shared context, L1/L2 legacy (no provider → 041 legacy module
/// singleton). `sharedInst` captures the shared store instance reference for
/// resource-freeze oracles (released instances leave the memo).
const MAIN_DUAL_TS: &str = r#"// PLAN-057 e2e overlay (test-owned): mounts the same generated App five ways.
import { createApp } from 'vue'
import App from './App.vue'
import { installAutoStoreContext, createAutoStoreContext } from './stores/__autoStoreContext'

const entries: Record<string, { app: any; ctx: any }> = {}

function mountInto(key: string, sel: string, shared?: any): any {
  const el = document.querySelector(sel)
  if (!el) return null
  const app = createApp(App)
  const ctx: any = installAutoStoreContext(app, shared)
  app.mount(el)
  entries[key] = { app, ctx }
  return { key }
}

// Legacy control: NO provider at all — useP057Store() must fall back to the
// named 041 legacy module singleton (installAutoStoreContext would provide a
// fresh context and the mount would not be legacy).
function mountLegacy(key: string, sel: string): any {
  const el = document.querySelector(sel)
  if (!el) return null
  const app = createApp(App)
  app.mount(el)
  entries[key] = { app, ctx: null }
  return { key }
}

const sharedCtx: any = createAutoStoreContext()

;(window as any).__p057 = {
  mountInto,
  sharedCtx,
  mountS3: () => mountInto('s3', '#app-s3', createAutoStoreContext()),
  mountL2: () => mountLegacy('l2', '#app-l2'),
  unmount: (key: string) => {
    const e = entries[key]
    if (!e) return
    e.app.unmount()
    e.ctx?.releaseApp?.(e.app)
    delete entries[key]
  },
}

mountInto('a', '#app-a')
mountInto('b', '#app-b')
mountInto('s1', '#app-s1', sharedCtx)
mountInto('s2', '#app-s2', sharedCtx)
mountLegacy('l1', '#app-l1')

// Capture the shared instance for freeze oracles. use() with a never-called
// factory returns the memoized instance without creating anything.
;(window as any).__p057.sharedInst = sharedCtx.use('P057Store', () => null)
"#;

/// Stage the fixture into scratch and run the cold scaffold + string gates
/// on the REAL generated output (fast — selected by default `cargo test`).
fn cold_generate_and_gate() -> (PathBuf, PathBuf) {
    let fixture = fixture_root();
    assert!(fixture.join("pac.at").exists(), "fixture missing: {fixture:?}");

    let cold = scratch_root().join("cold");
    if cold.exists() {
        fs::remove_dir_all(&cold).expect("clean scratch cold dir");
    }
    copy_dir_recursive(&fixture, &cold).expect("copy fixture to scratch");

    // Tool-flow order (tool_ops generate_profile "vue"): the incremental
    // compile drains compiler-side store files into src/stores/ BEFORE the
    // scaffold assembly reads `src/stores` to decide the main.ts context
    // install. Calling generate() without it drops the store context from
    // main.ts — the exact 051 regression class this ordering prevents.
    auto_ui_tools::vue::incremental_compile_changed(&cold, None)
        .expect("incremental compile (tool-flow store drain)");
    let project = VueProject::from_workspace(&cold).expect("from_workspace");
    if !project.exists() {
        project.generate().expect("cold generate");
    }
    let out = cold.join("gen").join("front").join("vue");
    assert!(out.join("package.json").exists(), "scaffold missing package.json");

    let ctx_src = fs::read_to_string(out.join("src/stores/__autoStoreContext.ts"))
        .expect("generated __autoStoreContext.ts");
    for needle in [
        "__autoOnCtxDispose",
        "releaseApp",
        "__autoCollect",
        "owners",
        "installAutoStoreContext(app: { provide(key: string, value: unknown): void }, existing?: AutoStoreContext)",
    ] {
        assert!(
            ctx_src.contains(needle),
            "__autoStoreContext.ts missing `{needle}`\n---\n{ctx_src}"
        );
    }

    let store_src = fs::read_to_string(out.join("src/stores/useP057Store.ts"))
        .expect("generated useP057Store.ts");
    for needle in [
        "import { __autoOnCtxDispose } from './__autoStoreContext'",
        "__auto_sse_live",
        "__autoOnCtxDispose(() => __auto_w0())",
        "__autoOnCtxDispose(() => { clearInterval(__auto_t0) })",
        "__autoOnCtxDispose(() => { __auto_sse_live = false; es.onerror = null; es.close() })",
        "inject<any | null>('__autoStoreContext', null)",
    ] {
        assert!(
            store_src.contains(needle),
            "useP057Store.ts missing `{needle}`\n---\n{store_src}"
        );
    }

    let main_src = fs::read_to_string(out.join("src/main.ts")).expect("generated main.ts");
    assert!(
        main_src.contains("installAutoStoreContext(app)"),
        "main.ts must install the per-App store context:\n{main_src}"
    );

    (cold, out)
}

/// Default-selected generation gate (no node/pnpm/browser needed): the cold
/// scaffold emits the PLAN-057 dispose wiring and keeps the 041 resolution
/// shape on the real fixture output.
#[test]
fn plan057_generated_dispose_wiring_and_legacy_boundary() {
    let (_cold, out) = cold_generate_and_gate();
    let _ = out; // gates ran inside the helper
    println!("PLAN057 generation gate OK (cold scaffold + dispose wiring strings)");
}

#[test]
#[ignore = "browser e2e — needs node/pnpm/playwright/msedge + npm mirror; run via the plan-recorded command"]
fn plan057_vue_instances_real_browser_e2e() {
    // ── 1. cold scaffold path + real-output string gates ────────────────
    let (cold, out) = cold_generate_and_gate();
    let project = VueProject::from_workspace(&cold).expect("from_workspace (second pass)");

    // ── 2. overlay dual-mount entry + install ───────────────────────────
    fs::write(out.join("dual.html"), DUAL_HTML).expect("write dual.html");
    fs::write(out.join("src/main-dual.ts"), MAIN_DUAL_TS).expect("write main-dual.ts");
    project.npm_install().expect("pnpm install");

    // SSE mock port must be free — the node oracle binds it per phase.
    assert!(
        TcpListener::bind(("127.0.0.1", SSE_PORT)).is_ok(),
        "SSE mock port {SSE_PORT} is occupied — free it before rerunning"
    );

    // ── 4. cold-path mount: generated main.ts boots a real App (AC-04) ──
    let port_a = free_port();
    let _vite_a = spawn_vite(&out, port_a);
    assert!(
        wait_port(port_a, Duration::from_secs(180)),
        "vite (cold path) did not open port {port_a}"
    );
    run_browser("single", port_a);

    // ── 5. run/start self-heal path (050 two-path coverage) ─────────────
    // main.ts is deleted to prove the run path rewrites it unconditionally;
    // vite.config.ts is REFRESHED in place (refresh_vite_config is a
    // update-in-place self-heal, not a recreate) and must stay byte-identical.
    let vite_cfg_before = fs::read_to_string(out.join("vite.config.ts")).expect("vite.config.ts");
    fs::remove_file(out.join("src/main.ts")).expect("remove main.ts");
    project.regenerate_source_files().expect("regen sources (run path)");
    refresh_vite_config(&out, &project.npm_deps);
    let vite_cfg_after = fs::read_to_string(out.join("vite.config.ts")).expect("vite.config.ts");
    assert_eq!(
        vite_cfg_before, vite_cfg_after,
        "run-path refresh_vite_config must regenerate the identical vite config (050)"
    );
    let main_regen = fs::read_to_string(out.join("src/main.ts")).expect("regen main.ts");
    assert!(
        main_regen.contains("installAutoStoreContext(app)"),
        "regenerated main.ts must still install the store context:\n{main_regen}"
    );

    let port_b = free_port();
    let _vite_b = spawn_vite(&out, port_b);
    assert!(
        wait_port(port_b, Duration::from_secs(180)),
        "vite (run path) did not open port {port_b}"
    );
    run_browser("dual", port_b);

    println!("PLAN057 E2E OK — cold path mount ✓, run path mount ✓, dual isolation/owner oracles ✓");
}
