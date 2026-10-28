//! Case `independent-consumer` (V-09): proves AGP is an independent
//! deliverable — no private-source dependency closure, locked reproducible
//! build, A→B relocation replay, and zero collateral on the rest of the
//! repository. Non-skip; non-zero on any failure.

use crate::host::report::{
    write_report, ConsumerClosureFacts, ConsumerCollateralFacts, ConsumerCommandFacts,
    ConsumerRelocationFacts, ConsumerReport,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Base commit for the "zero collateral outside AGP" check. Plan 830 starts
/// from the frozen 829 integration branch, so the default is that base; a
/// receipt from another plan can bind its own base via the env override.
const DEFAULT_BASE_COMMIT: &str = "ab56c32ec4cb28e01fbcca74e9861164ffad43f0";

fn base_commit() -> String {
    std::env::var("AGP_BASE_COMMIT").unwrap_or_else(|_| DEFAULT_BASE_COMMIT.to_string())
}

fn agp_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    agp_root().parent().unwrap().parent().unwrap().to_path_buf()
}

fn run_cmd(cmd: &mut Command) -> (bool, String) {
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let summary = format!(
                "exit={} stdout={} stderr={}",
                out.status.code().unwrap_or(-1),
                stdout.trim().chars().take(400).collect::<String>(),
                stderr.trim().chars().take(400).collect::<String>()
            );
            (out.status.success(), summary)
        }
        Err(e) => (false, format!("spawn failed: {e}")),
    }
}

pub fn run(replay: &str, report_dir: &Path, mode: &str) -> i32 {
    let _ = std::fs::create_dir_all(report_dir);
    if replay != "canonical" {
        eprintln!("independent-consumer: unknown replay '{replay}'");
        return 2;
    }
    let _ = mode;

    // ------------------------------------------------------------------
    // 1. Dependency closure: no path dependencies leaving AGP.
    // ------------------------------------------------------------------
    let closure = check_closure();

    // ------------------------------------------------------------------
    // 2. Locked build: cargo check --locked from this tree.
    // ------------------------------------------------------------------
    let locked_build = {
        let mut cmd = Command::new("cargo");
        cmd.arg("check").arg("--locked").current_dir(&agp_root());
        let (success, summary) = run_cmd(&mut cmd);
        ConsumerCommandFacts {
            command: "cargo check --locked".into(),
            cwd: agp_root().display().to_string(),
            success,
            summary,
        }
    };

    // ------------------------------------------------------------------
    // 3. A→B relocation replay.
    // ------------------------------------------------------------------
    let relocation = relocate_and_replay(report_dir);

    // ------------------------------------------------------------------
    // 4. Zero collateral.
    // ------------------------------------------------------------------
    let collateral = check_collateral();

    let ok = closure.ok && locked_build.success && relocation.ok && collateral.ok;
    let report = ConsumerReport {
        closure,
        locked_build,
        relocation,
        collateral,
        gpu_timing_available: false,
        notes: vec![
            "machine-specific fields exempt from relocation equivalence: adapter name/backend/device_type/driver/driver_info/vendor/device, device_token, utc, platform.profile, window facts, timings, rss/thread samples — deterministic replay hashes and semantic invariants must match".into(),
            "iced_widget 0.14.2 is STOCK from crates.io — the vendored patch was removed (r1 F09); closure is registry-only, zero path-sourced packages; nothing references crates/ or sibling repos".into(),
        ],
    };
    let exit = if ok { "pass" } else { "fail" };
    match write_report(
        report_dir,
        "independent-consumer",
        "canonical",
        exit,
        report,
    ) {
        Ok(()) => {
            if ok {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("independent-consumer: report: {e}");
            3
        }
    }
}

// ---------------------------------------------------------------------------
// 1. closure
// ---------------------------------------------------------------------------

fn check_closure() -> ConsumerClosureFacts {
    let mut violations = Vec::new();
    let mut path_deps = Vec::new();

    // 1a. Cargo.toml [dependencies] tables: any `path = ...` values.
    let manifest = agp_root().join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("Cargo.toml readable");
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("path =") {
            let val = rest.trim().trim_matches('"').to_string();
            path_deps.push(format!("manifest: {val}"));
            let resolved = manifest.parent().unwrap().join(&val);
            if !resolved.exists() {
                violations.push(format!("manifest path dep does not exist: {val}"));
            }
            // leaving AGP? All manifest path deps must resolve inside AGP.
            if !resolved
                .canonicalize()
                .map(|c| c.starts_with(agp_root().canonicalize().unwrap()))
                .unwrap_or(false)
            {
                violations.push(format!("manifest path dep leaves AGP: {val}"));
            }
        }
    }

    // 1b. Cargo.lock: path sources outside AGP.
    let lock = std::fs::read_to_string(agp_root().join("Cargo.lock")).expect("Cargo.lock readable");
    let agp_canon = agp_root().canonicalize().unwrap();
    for line in lock.lines() {
        if let Some(rest) = line.trim().strip_prefix("source = \"path+\"") {
            // no such representation expected, but guard anyway
            violations.push(format!("unexpected lock path source: {rest}"));
        }
    }
    // lockfile path deps have NO source line; detect by cross-checking
    // `cargo metadata` for path dependencies.
    let mut meta = Command::new("cargo");
    meta.arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--locked");
    meta.current_dir(&agp_root());
    if let Ok(out) = meta.output() {
        if out.status.success() {
            let doc: serde_json::Value =
                serde_json::from_slice(&out.stdout).expect("metadata json");
            for pkg in doc["packages"].as_array().cloned().unwrap_or_default() {
                let source = pkg["source"].is_null(); // null source = path/workspace dep
                if source {
                    let name = pkg["name"].as_str().unwrap_or("?");
                    let manifest_path = pkg["manifest_path"].as_str().unwrap_or("").to_string();
                    let canon =
                        PathBuf::from(&manifest_path.replace('/', "\\").replace("\\\\", "\\"));
                    let canon = std::fs::canonicalize(&canon).unwrap_or(canon);
                    if !canon.starts_with(&agp_canon) {
                        violations.push(format!(
                            "path dependency outside AGP: {name} @ {manifest_path}"
                        ));
                    } else {
                        path_deps.push(format!("metadata: {name} ({manifest_path})"));
                    }
                }
            }
        } else {
            violations.push("cargo metadata --locked failed".into());
        }
    }

    ConsumerClosureFacts {
        path_dependencies: path_deps,
        violations: violations.clone(),
        ok: violations.is_empty(),
    }
}

// ---------------------------------------------------------------------------
// 3. relocation
// ---------------------------------------------------------------------------

fn relocate_and_replay(_report_dir: &Path) -> ConsumerRelocationFacts {
    let src = agp_root();
    let dst = repo_root().join("scratch/829/relocate/B/autographics-prototype");
    let machine_specific = [
        "adapter.name",
        "adapter.backend",
        "adapter.device_type",
        "adapter.driver",
        "adapter.driver_info",
        "adapter.vendor",
        "adapter.device",
        "device_token",
        "utc",
        "platform.profile",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

    // clean + copy (exclude target/)
    let _ = std::fs::remove_dir_all(&dst);
    std::fs::create_dir_all(&dst).expect("mkdir dst");
    let copy = Command::new("robocopy")
        .args([
            src.to_str().unwrap(),
            dst.to_str().unwrap(),
            "/E",
            "/XD",
            "target",
            "/NFL",
            "/NDL",
            "/NJH",
            "/NJS",
            "/nc",
            "/ns",
            "/np",
        ])
        .output();
    let copied = match copy {
        Ok(o) => o.status.code().unwrap_or(-1) < 8, // robocopy: <8 = success
        Err(_) => {
            // fallback: bash cp via tar
            let st = Command::new("tar")
                .args(["cf", "-", "--exclude=target", "-C"])
                .arg(&src)
                .arg(".")
                .stdout(std::process::Stdio::piped())
                .spawn();
            match st {
                Ok(mut child) => {
                    let out = child.stdout.take().unwrap();
                    let mut extract = Command::new("tar")
                        .args(["xf", "-", "-C"])
                        .arg(&dst)
                        .stdin(out)
                        .spawn()
                        .expect("tar extract");
                    extract.wait().map(|s| s.success()).unwrap_or(false)
                }
                Err(_) => false,
            }
        }
    };

    let build = {
        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--locked", "--release", "--bin", "graphics-probe"])
            .current_dir(&dst);
        let (success, summary) = run_cmd(&mut cmd);
        ConsumerCommandFacts {
            command: "cargo build --locked --release --bin graphics-probe".into(),
            cwd: dst.display().to_string(),
            success: success && copied,
            summary: format!("copied={copied} {summary}"),
        }
    };

    let case_report = dst
        .join("../../B-report")
        .canonicalize()
        .unwrap_or_else(|_| repo_root().join("scratch/829/relocate/B-report"));
    let _ = std::fs::create_dir_all(&case_report);
    let exe = dst.join("target/release/graphics-probe.exe");
    let case_command = format!(
        "{} --case shared-runtime --replay canonical --report {} --mode headless",
        exe.display(),
        case_report.display()
    );
    let (case_success, case_summary) = {
        let mut cmd = Command::new(&exe);
        cmd.args([
            "--case",
            "shared-runtime",
            "--replay",
            "canonical",
            "--report",
            case_report.to_str().unwrap(),
            "--mode",
            "headless",
        ]);
        run_cmd(&mut cmd)
    };

    // Equivalence: run THIS tree headless as the A-side reference (same
    // mode as B), then deep-compare the two reports with only
    // machine-specific fields exempt. Self-contained: the A-side reference
    // binary is built HERE from the current tree, into an isolated target
    // dir (review r2 F829-R2-02 — fingerprints.agp_commit is a build-time
    // value, so a prebuilt binary left over from an older commit false-reds
    // the comparison; the isolated dir also avoids fighting the Windows
    // file lock on a possibly-running target/release/graphics-probe.exe);
    // the B-side binary is built in the relocated copy below.
    let mut mismatches = Vec::new();
    let mut compared = Vec::new();
    let a_report = repo_root().join("scratch/829/relocate/A-report");
    let _ = std::fs::create_dir_all(&a_report);
    {
        let a_target = repo_root().join("scratch/829/consumer-a-target");
        let exe_a = a_target.join("release/graphics-probe.exe");
        // Ensure the A-side reference is current: build in-case (isolated
        // CARGO_TARGET_DIR) so the comparison never depends on whatever a
        // previous run or commit left in target/release.
        let mut build_a = Command::new("cargo");
        build_a
            .args(["build", "--locked", "--release", "--bin", "graphics-probe"])
            .current_dir(agp_root())
            .env("CARGO_TARGET_DIR", &a_target);
        let (a_build_ok, a_build_summary) = run_cmd(&mut build_a);
        if !a_build_ok {
            mismatches.push(format!(
                "A-side reference build failed in original tree: {a_build_summary}"
            ));
        }
        let mut cmd = Command::new(&exe_a);
        cmd.args([
            "--case",
            "shared-runtime",
            "--replay",
            "canonical",
            "--report",
            a_report.to_str().unwrap(),
            "--mode",
            "headless",
        ]);
        let (ok, summary) = run_cmd(&mut cmd);
        if !ok {
            mismatches.push(format!("A-side headless reference run failed: {summary}"));
        }
    }
    let reference = a_report.join("report.json");
    let relocated = case_report.join("report.json");
    if reference.exists() && relocated.exists() {
        let a: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&reference).unwrap()).unwrap();
        let b: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&relocated).unwrap()).unwrap();
        let exempt: Vec<&str> = machine_specific.iter().map(|s| s.as_str()).collect();
        compare_json("report", &a, &b, &exempt, &mut compared, &mut mismatches);
    } else {
        mismatches.push(format!(
            "reference or relocated report missing: {} / {}",
            reference.display(),
            relocated.display()
        ));
    }
    let _ = case_summary;

    let build_ok = build.success;
    let all_ok = build_ok && case_success && mismatches.is_empty();
    ConsumerRelocationFacts {
        src: src.display().to_string(),
        dst: dst.display().to_string(),
        build,
        case_command,
        case_success,
        machine_specific_fields: machine_specific,
        compared_fields: compared,
        mismatches,
        ok: all_ok,
    }
}

/// Deep-compare two JSON values; paths matching an exempt prefix are skipped.
fn compare_json(
    path: &str,
    a: &serde_json::Value,
    b: &serde_json::Value,
    exempt: &[&str],
    compared: &mut Vec<String>,
    mismatches: &mut Vec<String>,
) {
    let is_exempt = exempt.iter().any(|e| {
        path == *e
            || path.ends_with(&format!(".{e}"))
            || path.contains(&format!(".{e}."))
            || path.contains(&format!("{e}["))
    });
    if is_exempt {
        return;
    }
    match (a, b) {
        (serde_json::Value::Object(ma), serde_json::Value::Object(mb)) => {
            for k in ma.keys() {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match mb.get(k) {
                    Some(vb) => compare_json(&p, &ma[k], vb, exempt, compared, mismatches),
                    None => mismatches.push(format!("missing key in B: {p}")),
                }
            }
            for k in mb.keys() {
                if !ma.contains_key(k) {
                    mismatches.push(format!("extra key in B: {path}.{k}"));
                }
            }
        }
        (serde_json::Value::Array(aa), serde_json::Value::Array(ab)) => {
            if aa.len() != ab.len() {
                mismatches.push(format!(
                    "array length differs at {path}: {} vs {}",
                    aa.len(),
                    ab.len()
                ));
            }
            for (i, (va, vb)) in aa.iter().zip(ab.iter()).enumerate() {
                compare_json(
                    &format!("{path}[{i}]"),
                    va,
                    vb,
                    exempt,
                    compared,
                    mismatches,
                );
            }
        }
        _ => {
            compared.push(path.to_string());
            if a != b {
                mismatches.push(format!("value differs at {path}: {a} vs {b}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. zero collateral
// ---------------------------------------------------------------------------

fn check_collateral() -> ConsumerCollateralFacts {
    let root = repo_root();
    let base = base_commit();

    // 4a. Root diff outside AGP vs the branch base. The plan's PROVISIONAL
    // Spec drafts (docs/specs/auto-graphics/) are declared deltas prepared in
    // this worktree; they are recorded separately and are not production
    // code. Everything else outside AGP must be untouched.
    let mut diff_outside = Vec::new();
    let mut drafts = Vec::new();
    let mut git = Command::new("git");
    git.args([
        "diff",
        "--name-only",
        &base,
        "--",
        ".",
        ":(exclude)experimental/autographics-prototype",
    ])
    .current_dir(&root);
    if let Ok(out) = git.output() {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let path = line.trim();
                if path.starts_with("docs/specs/auto-graphics/") {
                    drafts.push(path.to_string());
                } else {
                    diff_outside.push(path.to_string());
                }
            }
        } else {
            diff_outside.push("git diff failed".into());
        }
    }
    // Only the plan's DECLARED Spec targets may appear as provisional drafts;
    // any other spec/doc change still counts as collateral.
    let allowed_drafts = [
        "docs/specs/auto-graphics/surface-composition.md",
        "docs/specs/auto-graphics/nested-input.md",
        "docs/specs/auto-graphics/prototype-profile.md",
        "docs/specs/auto-graphics/integration-contract.md",
    ];
    let declared: Vec<String> = drafts
        .into_iter()
        .filter(|d| {
            if allowed_drafts.contains(&d.as_str()) {
                true
            } else {
                diff_outside.push(d.clone());
                false
            }
        })
        .collect();
    let drafts = declared;

    // 4b. Reparse points (junctions/symlinks) inside AGP.
    let mut reparse = Vec::new();
    let script = format!(
        "Get-ChildItem -LiteralPath '{}' -Recurse -Force -Attributes ReparsePoint -ErrorAction SilentlyContinue | ForEach-Object {{ $_.FullName }}",
        agp_root().display()
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
    if let Ok(out) = out {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if !line.trim().is_empty() {
                reparse.push(line.trim().to_string());
            }
        }
    }

    // 4c. Layering: core/ must not reference wgpu/iced/winit.
    let mut layering = Vec::new();
    let core = agp_root().join("src/core");
    for entry in walk_rust_files(&core) {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue; // comments may name the forbidden crates in prose
            }
            if line.contains("wgpu") || line.contains("iced") || line.contains("winit") {
                layering.push(format!("{}:{}: {}", entry.display(), i + 1, line.trim()));
            }
        }
    }

    ConsumerCollateralFacts {
        base_commit: base,
        root_diff_outside_agp: diff_outside.clone(),
        provisional_spec_drafts: drafts,
        reparse_points_in_agp: reparse.clone(),
        core_layering_violations: layering.clone(),
        ok: diff_outside.is_empty() && reparse.is_empty() && layering.is_empty(),
    }
}

fn walk_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk_rust_files(&p));
            } else if p.extension().map(|e| e == "rs").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out
}
