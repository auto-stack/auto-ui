//! PLAN-066 T-05 / V-04: same-process GPU owner accounting and exit safety.
//!
//! Eight hosts are created, used and dropped sequentially in one process; each
//! host runs the frozen authored 3D chain for twenty Start/Stop rounds. Every
//! round must return the Run-level owners (surface targets/bytes) to the
//! pre-run baseline, keep the shared host-level store (mesh count/bytes) flat,
//! and leave no live target after Stop. The process must survive all eight
//! host teardowns and a final ninth host (same-device exit path) without an
//! access violation, which is the AG021 same-process symptom this Plan owns.
//! GPU single instance: run with `--test-threads=1`.
use autographics_auto_driver::authoring::{assets::Registry, host::Host, storage::Storage};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const G07A_SCENE: &str = include_str!(
    "../../autographics-node2d-pilot/generated-components3d-runtime/g07a/scene.agscene"
);
const G07A_WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components3d-runtime/g07a/world.at");

const HOSTS: usize = 8;
const ROUNDS: usize = 20;

fn write_project() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan066-lifetime")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&dir);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).expect("scripts dir");
    std::fs::write(dir.join("scene.agscene"), G07A_SCENE).expect("write scene");
    let sha = G07A_SCENE
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .expect("scene world sha");
    std::fs::write(scripts.join(format!("{sha}.at")), G07A_WORLD).expect("write script");
    dir
}

fn settle(h: &mut Host) {
    let end = Instant::now() + Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().is_some_and(|p| p.pending());
        if !pending {
            break;
        }
        assert!(Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

fn step(h: &mut Host) {
    {
        let p = h.play.as_mut().expect("play");
        p.resume().expect("resume");
        p.char_held_restore(2);
        p.dispatch(Default::default()).expect("dispatch");
    }
    h.pause_play("");
    settle(h);
}

/// The frozen ledger: (target slots, target bytes, mesh count, mesh bytes).
fn ledger(h: &Host) -> (u64, u64, u64, u64) {
    let s = h.resource_snapshot();
    let n = |k: &str| s[k].as_u64().unwrap_or_else(|| panic!("ledger {k}"));
    (
        n("target_count"),
        n("target_bytes"),
        n("mesh_count"),
        n("mesh_bytes"),
    )
}

/// The authored chain re-enters the SDK parser per Start; give it the same
/// 64 MiB runway the other runtime-camera tests use.
fn big_stack(test: impl FnOnce() + Send) {
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(s, test)
            .expect("spawn")
            .join()
            .unwrap_or_else(|e| {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "opaque panic".to_string());
                panic!("big_stack worker: {msg}");
            });
    });
}

#[test]
fn v04_eight_hosts_and_twenty_rounds_keep_the_owner_ledger_flat() {
    big_stack(|| {
        let dir = write_project();
        let mut hosts = Vec::new();
        for host_index in 0..HOSTS {
            let (doc, registry, code, _) =
                Storage::load_project(&dir.join("scene.agscene"), &Registry::new())
                    .unwrap_or_else(|e| panic!("host {host_index} load: {e:?}"));
            let mut h = Host::with_project(doc, registry, code).expect("host");
            for _ in 0..3 {
                h.frame(None).expect("warm frame");
            }
            let base = ledger(&h);
            let mut peak = base;
            for round in 0..ROUNDS {
                h.start_selected_play()
                    .unwrap_or_else(|e| panic!("host {host_index} round {round} start: {e:?}"));
                h.pause_play("");
                settle(&mut h);
                step(&mut h);
                assert!(
                    h.play.as_ref().is_some_and(|p| p.error.is_none()),
                    "host {host_index} round {round} fault: {:?}",
                    h.play.as_ref().and_then(|p| p.error.clone())
                );
                h.stop_play();
                let after = ledger(&h);
                assert_eq!(
                        (after.0, after.1),
                        (base.0, base.1),
                        "host {host_index} round {round}: Run-level targets must return to the pre-run baseline"
                    );
                assert_eq!(
                    (after.2, after.3),
                    (base.2, base.3),
                    "host {host_index} round {round}: the shared store must stay warm and flat"
                );
                assert_eq!(
                    h.rt.live_target_slots(),
                    base.0 as usize,
                    "host {host_index} round {round}: no live target after Stop"
                );
                assert_eq!(h.rt.owned_target_bytes(), base.1, "target bytes after Stop");
                peak = (
                    peak.0.max(after.0),
                    peak.1.max(after.1),
                    peak.2.max(after.2),
                    peak.3.max(after.3),
                );
            }
            assert!(h.play.is_none(), "host {host_index}: Stop releases the Run");
            hosts.push(format!(
                "{{\"host\":{host_index},\"base\":{base:?},\"peak\":{peak:?},\"live_run\":{}}}",
                h.play.is_some()
            ));
            drop(h);
        }
        // Final same-device exit path: one more host after every prior host
        // was dropped; the process must still create, run and drop it.
        let (doc, registry, code, _) =
            Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).expect("tail load");
        let mut tail = Host::with_project(doc, registry, code).expect("tail host");
        tail.start_selected_play().expect("tail start");
        tail.pause_play("");
        settle(&mut tail);
        step(&mut tail);
        tail.stop_play();
        let tail_ledger = ledger(&tail);
        drop(tail);
        eprintln!(
            "T05_LEDGER {{\"hosts\":[{}],\"tail\":{tail_ledger:?}}}",
            hosts.join(",")
        );
    });
}
