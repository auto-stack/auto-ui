//! PLAN-860 fresh-process probe runner. One invocation = one measured process
//! (853 sampling convention: five fresh processes per workload class happen at
//! the tool level). Writes one JSON report; exits nonzero on any failure.
use autographics_auto_driver::research::game_runtime::{self as gr, Candidate, Route};
use serde_json::{json, Value};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| -> Option<String> {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let result = (|| -> Result<Value, String> {
        let case = get("--case").ok_or("missing --case")?;
        let candidate = match get("--candidate").unwrap_or_else(|| "a".into()).as_str() {
            "a" => Candidate::A,
            "b" => Candidate::B,
            other => return Err(format!("unknown candidate {other}")),
        };
        let steps: usize = match get("--steps") {
            Some(v) => v.parse().map_err(|e| format!("--steps: {e}"))?,
            None => 0,
        };
        let output: PathBuf = get("--output").ok_or("missing --output")?.into();
        let raw = get("--raw").map(|v| v == "1").unwrap_or(false);

        let started = std::time::Instant::now();
        let (_name, report, samples, identity, heap_objects, extra) = match case.as_str() {
            "g2" | "g3" => {
                let route = if case == "g2" { Route::G2 } else { Route::G3 };
                let steps = if steps == 0 {
                    match route {
                        Route::G2 => gr::G2_STEPS,
                        Route::G3 => gr::G3_STEPS,
                    }
                } else {
                    steps
                };
                if steps > gr::MAX_ROUTE_STEPS {
                    return Err("route steps above frozen 1800 bound".into());
                }
                let rows = if route == Route::G2 {
                    gr::g2_rows()
                } else {
                    gr::g3_rows()
                };
                let user = gr::route_user(route, candidate);
                let source_name = gr::route_name(route, candidate);
                let snap = gr::combined_source(user, source_name, candidate)?;
                let identity = json!({
                    "user_bytes": snap.bytes.len(), "combined_bytes": snap.source.len(),
                    "user_hash": snap.user_hash, "api_hash": snap.api_hash,
                    "bridge_hash": snap.bridge_hash, "combined_hash": snap.combined_hash,
                    "version": snap.version,
                });
                let mut s = gr::session_for(user, source_name, candidate, &rows, false)?;
                let reads: Vec<&str> = if candidate == Candidate::B {
                    vec!["scans", "support"]
                } else {
                    vec![]
                };
                let samples = gr::run_steps(&mut s, steps, &reads)?;
                let heap = s.heap_objects();
                let rep = gr::route_report(route, &samples);
                (
                    format!("{case}-{candidate:?}").to_lowercase(),
                    json!(rep),
                    samples,
                    identity,
                    heap,
                    json!({}),
                )
            }
            "wide128" | "deep128" => {
                let deep = case == "deep128";
                let steps = if steps == 0 { 35 } else { steps };
                let rows = gr::load_rows(deep);
                let user = gr::load_source(candidate);
                let name = gr::load_name(candidate);
                let snap = gr::combined_source(user, name, candidate)?;
                let identity = json!({
                    "user_bytes": snap.bytes.len(), "combined_bytes": snap.source.len(),
                    "user_hash": snap.user_hash, "api_hash": snap.api_hash,
                    "bridge_hash": snap.bridge_hash, "combined_hash": snap.combined_hash,
                    "version": snap.version,
                });
                let mut s = gr::session_for(user, name, candidate, &rows, deep)?;
                let reads: Vec<&str> = if candidate == Candidate::B {
                    vec!["scans", "support"]
                } else {
                    vec![]
                };
                // Five neutral warmup ticks then measured ticks (853 style);
                // warmup samples are dropped from percentiles below.
                let all = gr::run_steps(&mut s, steps, &reads)?;
                let heap = s.heap_objects();
                // The tick source hides node 127 unless every move reported
                // status 1 (Clipped at the last-iterated blocker).
                let blocker_visible =
                    s.int("ag_visible", &[autographics_auto_driver::Scalar::Int(127)])?;
                (
                    format!("{case}-{candidate:?}").to_lowercase(),
                    json!({"kind":"load","deep":deep,"blocker_visible":blocker_visible}),
                    all.into_iter().skip(5).collect(),
                    identity,
                    heap,
                    json!({}),
                )
            }
            other => return Err(format!("unknown case {other}")),
        };

        let tick_samples: Vec<u64> = samples.iter().map(|s| s.tick_ns).collect();
        let read_samples: Vec<u64> = samples.iter().map(|s| s.reads_ns).collect();
        let report = json!({
            "schema": "autographics.game-probe/1",
            "case": case, "candidate": format!("{candidate:?}").to_lowercase(),
            "steps": samples.len(),
            "identity": identity,
            "configuration": {
                "rust_min_stack": std::env::var("RUST_MIN_STACK").ok(),
                "rustflags": std::env::var("RUSTFLAGS").ok(),
                "rustc_wrapper": std::env::var("RUSTC_WRAPPER").ok(),
                "profile": "debug",
            },
            "tick_cpu_ns": {
                "p50": gr::percentile(tick_samples.clone(), 0.50),
                "p95": gr::percentile(tick_samples.clone(), 0.95),
                "max": tick_samples.iter().max().copied().unwrap_or(0),
            },
            "reads_cpu_ns": {
                "p50": gr::percentile(read_samples.clone(), 0.50),
                "p95": gr::percentile(read_samples, 0.95),
            },
            "credits_max": samples.iter().map(|s| s.credits).max().unwrap_or(0),
            "resumes_total": samples.last().map(|s| s.resumes).unwrap_or(0),
            "scans_max": samples.iter().map(|s| s.scans).max().unwrap_or(0),
            "heap_objects": heap_objects,
            "wall_ns": started.elapsed().as_nanos() as u64,
            "report": report,
            "extra": extra,
            "samples": if raw { json!(samples) } else { Value::Null },
            "completed": true,
        });
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(
            &output,
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(report)
    })();
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
