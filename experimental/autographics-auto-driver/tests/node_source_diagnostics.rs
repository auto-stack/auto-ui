//! PLAN-044 T-02/T-04：driver 侧真实 SDK compile span → 原模块源定位
//! 全链先证（V-S01 driver 侧 + V-S03 变异拒用）。
//!
//! 链路：pilot 生成的冻结 world.at（diag-fault：跨模块 helper `speedup`
//! 体内 float+string 类型错误）→ SourceSnapshot 受检装配（user region）
//! → SDK compile 真实 span → node-source-map 旁件 + 受检 closure →
//! broken.at 行列 + range 精度 + helper 上下文。坏图/坏闭包回退原诊断。

use std::path::Path;

use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
use autographics_auto_driver::authoring::behavior_source_map::{Precision, SourceMap};
use autographics_auto_driver::Session;

const WORLD: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/diag-fault/world.at");
const MAP_JSON: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/diag-fault/source-map.json");
const BROKEN_AT: &str =
    include_str!("../../autographics-node2d-pilot/generated-components2d/diag-fault/closure/broken.at");

fn mapped_diag(world: &str, map_json: &str, closure_dir: &Path) -> String {
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("world.at"),
        world.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("snapshot assembles");
    let world = std::str::from_utf8(&snap.bytes).expect("world utf8");
    let map = SourceMap::parse(map_json, world).expect("map parses+binds");
    let closure = map.closure_sources(closure_dir).expect("closure verifies");
    let diags = std::cell::RefCell::new(Vec::new());
    let _ = Session::new_with_diagnostics(&snap.source, |stage, e| {
        let d = snap.sdk_diagnostic_mapped(stage, e, Some(&map), Some(&closure));
        diags.borrow_mut().push(d.clone());
        d
    });
    let diags = diags.into_inner();
    assert_eq!(diags.len(), 1);
    diags.into_iter().next().unwrap()
}

#[test]
fn sdk_compile_span_maps_back_to_original_module_helper() {
    // 原诊断（无映射）：仍指向 world.at 的 user 区行列（行为保持）。
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("world.at"),
        WORLD.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("snapshot assembles");
    let plain = std::cell::RefCell::new(Vec::new());
    let result = Session::new_with_diagnostics(&snap.source, |stage, e| {
        let d = snap.sdk_diagnostic(stage, e);
        plain.borrow_mut().push(d.clone());
        d
    });
    assert!(result.is_err());
    let plain = plain.into_inner();
    assert_eq!(plain.len(), 1);
    assert!(
        plain[0].contains("world.at"),
        "unmapped diagnostic keeps world.at: {}",
        plain[0]
    );

    // 映射诊断：回到 broken.at 的 speedup 体（float+string 类型错误行）。
    let closure_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../autographics-node2d-pilot/generated-components2d/diag-fault/closure"
    ));
    let diag = mapped_diag(WORLD, MAP_JSON, closure_dir);
    // broken.at 第 8 行是 `    return v + absent_helper(1)`（1-based，
    // 列 16 恰为 absent_helper 标识符起点——重命名差分回算实证）。
    assert!(
        diag.contains("broken.at:8:16"),
        "mapped diagnostic must point at broken.at:8:16: {diag}"
    );
    assert!(
        diag.contains("node-source-map Range") && diag.contains("helper=speedup"),
        "precision + helper context required: {diag}"
    );
    assert!(
        diag.contains("Undefined symbol"),
        "original error text preserved: {diag}"
    );
    // 行内容实证：第 8 行确实是 fault 行。
    let line8 = BROKEN_AT.lines().nth(7).expect("line 8");
    assert!(line8.contains("absent_helper"), "fixture line 8 is the fault");
    // 不误报同链路的其它模块（helpera/helperb 的 bump 无错）。
    assert!(!diag.contains("helpera"), "must not cross into helpera: {diag}");
    assert!(!diag.contains("helperb"), "must not cross into helperb: {diag}");
}

#[test]
fn mutated_or_missing_map_falls_back_without_covering_error() {
    let closure_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../autographics-node2d-pilot/generated-components2d/diag-fault/closure"
    ));
    // 坏 world hash：整图拒用 → 回退原诊断。
    let mut m: serde_json::Value = serde_json::from_str(MAP_JSON).unwrap();
    m["world_sha256"] = serde_json::Value::String("0".repeat(64));
    let bad = serde_json::to_string(&m).unwrap();
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("world.at"),
        WORLD.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .unwrap();
    assert!(SourceMap::parse(&bad, WORLD).is_err(), "bad world hash must reject");
    // 旧 world（字节漂移）：拒用。
    let mut drifted = WORLD.to_string();
    drifted.push('\n');
    assert!(
        SourceMap::parse(MAP_JSON, &drifted).is_err(),
        "stale map for drifted world must reject"
    );
    // 受检 mapped 路径在坏图上 = 原诊断（fallback 不覆盖原错误）。
    let map = SourceMap::parse(MAP_JSON, WORLD).unwrap();
    let closure = map.closure_sources(closure_dir).unwrap();
    let diags = std::cell::RefCell::new(Vec::new());
    let _ = Session::new_with_diagnostics(&snap.source, |stage, e| {
        let d = snap.sdk_diagnostic_mapped(stage, e, None, Some(&closure));
        diags.borrow_mut().push(d.clone());
        d
    });
    let diags = diags.into_inner();
    assert_eq!(diags.len(), 1);
    assert!(diags[0].contains("world.at"), "no-map fallback: {}", diags[0]);
}

#[test]
fn tampered_closure_rejects_and_fallback_holds() {
    let closure_dir = std::env::temp_dir().join(format!("plan044-closure-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&closure_dir);
    std::fs::create_dir_all(&closure_dir).unwrap();
    // 合法闭包先过。
    let src = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../autographics-node2d-pilot/generated-components2d/diag-fault/closure"
    ));
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), closure_dir.join(entry.file_name())).unwrap();
    }
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("world.at"),
        WORLD.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .unwrap();
    let map = SourceMap::parse(MAP_JSON, WORLD).unwrap();
    let closure = map.closure_sources(&closure_dir).expect("good closure verifies");
    assert!(closure.contains_key("broken"));
    // 篡改 broken.at（注释漂移即 sha 不符）→ 拒用。
    std::fs::write(closure_dir.join("broken.at"), "// tampered\n").unwrap();
    assert!(map.closure_sources(&closure_dir).is_err(), "tampered closure must reject");
    std::fs::remove_dir_all(&closure_dir).ok();
}

#[test]
fn precision_matrix_generated_segments_never_map() {
    // generated 段（SoA/调度）命中时 diagnose 返回 None → 调用方回退。
    let snap = SourceSnapshot::from_bytes_for(
        Path::new("world.at"),
        WORLD.as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .unwrap();
    let map = SourceMap::parse(MAP_JSON, WORLD).unwrap();
    let gen = map
        .segments
        .iter()
        .find(|s| s.precision == Precision::Generated)
        .expect("a generated segment exists");
    let closure_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../autographics-node2d-pilot/generated-components2d/diag-fault/closure"
    ));
    let closure = map.closure_sources(closure_dir).unwrap();
    assert!(
        map.diagnose_user_offset(gen.start, WORLD, &closure, None)
            .is_none(),
        "generated segments never map to author lines"
    );
}
