//! PLAN-044 gates：Node 源映射结构/锚定/变异拒用。
//!
//! 覆盖 V-S01 的映射侧与 V-S03 的拒用侧：pooled helper 跨模块 exact/range
//! 定位、generated 补齐、lookup、以及坏 hash/越界/重叠/逃逸 key/伪精度/
//! 非字符边界的变异红。真实 compile 全链先证在 driver
//! tests/node_source_diagnostics.rs（T-02/T-04）。

use std::collections::BTreeMap;

use autographics_node2d_pilot::load::{LoadedModule, Project};
use autographics_node2d_pilot::source_map::{
    build_source_map, fill_generated_gaps, locate_pooled_fns, Precision, Segment, SourceMap,
    MAP_FORMAT,
};
use autographics_node2d_pilot::scene::sha256_hex;

const GENERATOR: &str = "node2d-lower[components-2d]";

fn module(name: &str, source: &str, fns: Vec<(&str, usize, usize)>) -> LoadedModule {
    LoadedModule {
        name: name.to_string(),
        source: source.to_string(),
        nodes: Vec::new(),
        fns: fns
            .into_iter()
            .map(|(n, s, e)| (n.to_string(), s..e))
            .collect(),
        resources: Vec::new(),
        uses: Vec::new(),
        enums: BTreeMap::new(),
    }
}

/// 夹具源：util.at 提供跨模块 helper（内部调同模块 sibling，模拟重命名面）。
fn fixture_sources() -> (String, String) {
    let util = "fn score(x int) int {\n  return double(x)\n}\nfn double(x int) int {\n  return x * 2\n}\n".to_string();
    let main = "node Game {\n  on .Tick {\n    let v = util.score(3)\n  }\n}\n".to_string();
    (util, main)
}

/// 模拟发射器：header（generated）+ pooled helper 锚定段 + 尾部调度（generated）。
fn emit_like_pooled(util_source: &str, renamed: bool) -> String {
    let mut src = String::from("// GENERATED header\nvar ok=0.0\n");
    // 只发射 score（double 在真实发射器同样有自己的锚定段；本夹具单段足够）。
    let mut body = util_source[..util_source.find("fn double").unwrap()].to_string();
    if renamed {
        // 模块调用重命名模拟：行内 token 替换（行号不变，列可能偏移）。
        body = body.replace("double(x)", "util_double(x)");
    }
    src.push_str("// pooled fn score from util\n");
    src.push_str(body.trim_end());
    src.push('\n');
    src.push_str("fn app_start() {\n}\n");
    src
}

fn project_with_util() -> (Project, String) {
    let (util, main) = fixture_sources();
    let score_start = util.find("fn score").unwrap();
    let score_len = autographics_node2d_pilot::lower::slice_fn_source(&util, "score")
        .expect("score slice")
        .len();
    let mut project = Project::default();
    project
        .modules
        .insert("util".into(), module("util", &util, vec![("score", score_start, score_start + score_len)]));
    project
        .modules
        .insert("main".into(), module("main", &main, Vec::new()));
    (project, util)
}

#[test]
fn pooled_helper_exact_cross_module() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    let segments = locate_pooled_fns(&project, &src).expect("locate");
    assert_eq!(segments.len(), 1);
    let seg = &segments[0];
    assert_eq!(seg.precision, Precision::Exact);
    assert_eq!(seg.module.as_deref(), Some("util"));
    assert_eq!(seg.helper.as_deref(), Some("score"));
    // exact：片段字节 == 原模块源切片字节。
    let slice = &project.modules["util"].source[seg.source_start.unwrap()..seg.source_end.unwrap()];
    assert_eq!(src[seg.start..seg.end].trim_start_matches("// pooled fn score from util\n"), slice.trim_end());
}

#[test]
fn pooled_helper_rename_downgrades_to_range() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, true);
    let segments = locate_pooled_fns(&project, &src).expect("locate");
    assert_eq!(segments[0].precision, Precision::Range);
    // 行号可证：原切片行数 == 片段行数。
    let slice = &project.modules["util"].source
        [segments[0].source_start.unwrap()..segments[0].source_end.unwrap()];
    assert_eq!(slice.lines().count(), src[segments[0].start..segments[0].end].lines().count() - 1);
}

#[test]
fn build_map_fills_generated_gaps_and_validates() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    let world_sha = sha256_hex(src.as_bytes());
    let map = build_source_map(&project, &src, GENERATOR, &world_sha).expect("build");
    map.validate(&src).expect("validate");
    assert_eq!(map.format, MAP_FORMAT);
    assert_eq!(map.user_source_bytes, src.len());
    // 段连续覆盖、不重叠：header(generated) + helper(exact) + 尾部(generated)。
    assert!(map.segments.len() >= 3);
    let mut cursor = 0;
    for seg in &map.segments {
        assert_eq!(seg.start, cursor, "segments must tile the source");
        cursor = seg.end;
    }
    assert_eq!(cursor, src.len());
    // lookup：header 命中 generated，helper 体内命中 exact。
    let helper = map
        .segments
        .iter()
        .find(|s| s.precision == Precision::Exact)
        .unwrap();
    assert_eq!(map.lookup(helper.start + 1).unwrap().helper.as_deref(), Some("score"));
    assert_eq!(map.lookup(0).unwrap().precision, Precision::Generated);
    assert_eq!(map.lookup(src.len()), None);
    // 字节重算行（exact 证据核心）：片段内相对偏移的行号增量必须等于
    // 原模块源同相对偏移的行号增量（verbatim 行号可证）。
    let body_off = helper.start + "// pooled fn score from util\n".len();
    let k = 8; // "fn score" 体内的相对偏移
    let gen_rel = src[body_off..body_off + k].bytes().filter(|b| *b == b'\n').count();
    let src_rel = project.modules["util"].source[helper.source_start.unwrap()..helper.source_start.unwrap() + k]
        .bytes()
        .filter(|b| *b == b'\n')
        .count();
    assert_eq!(gen_rel, src_rel, "verbatim 片段行号增量必须可证");
}

#[test]
fn gap_fill_merges_and_tiles() {
    let segs = vec![
        Segment { start: 4, end: 6, module: Some("m".into()), node: None, event: None, helper: None, precision: Precision::Exact, source_start: Some(0), source_end: Some(2) },
    ];
    let out = fill_generated_gaps(segs, 10);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], Segment { start: 0, end: 4, module: None, node: None, event: None, helper: None, precision: Precision::Generated, source_start: None, source_end: None });
    assert_eq!(out[2].precision, Precision::Generated);
    assert_eq!(out[2].start, 6);
    assert_eq!(out[2].end, 10);
}

fn valid_map(project: &Project, src: &str) -> SourceMap {
    let world_sha = sha256_hex(src.as_bytes());
    build_source_map(project, src, GENERATOR, &world_sha).expect("build")
}

#[test]
fn mutation_rejects_bad_world_hash() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    let mut map = valid_map(&project, &src);
    // 格式合法但与实际 world 内容不符的 hash（旧 Run / 错 world 绑定）：
    // 结构校验可过（无 world 字节可比对），载入侧绑定校验必须拒用。
    map.world_sha256 = sha256_hex(b"not-the-world");
    map.validate(&src).expect("format-valid hash passes structural check");
    assert!(map.validate_against_world(&src).is_err());
    // 磁盘 world 被改 → user sha 不符 → 拒用且不改错误指向。
    assert!(map.validate_against_world(&(src.clone() + "\n")).is_err());
    // 非 64 位十六进制在结构层即拒用。
    let mut map2 = valid_map(&project, &src);
    map2.world_sha256 = "xyz".into();
    assert!(map2.validate(&src).is_err());
}

#[test]
fn mutation_rejects_source_bytes_drift() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    let mut map = valid_map(&project, &src);
    map.user_source_bytes += 1;
    assert!(map.validate(&src).is_err());
    // 磁盘源被改（sha 不符）→ 整图拒用，不改错误指向。
    let mut map2 = valid_map(&project, &src);
    map2.user_source_sha256 = sha256_hex(b"tampered");
    assert!(map2.validate(&src).is_err());
}

#[test]
fn mutation_rejects_overlap_out_of_bounds_and_char_boundary() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    let mut map = valid_map(&project, &src);
    // 越界
    let mut m = map.clone();
    m.segments[0].end = src.len() + 1;
    assert!(m.validate(&src).is_err());
    // 重叠/乱序
    let mut m = map.clone();
    if m.segments.len() >= 2 {
        m.segments[1].start = m.segments[0].start;
        assert!(m.validate(&src).is_err());
    }
    // 非字符边界（多字节 UTF-8 中段）：构造独立小图。
    let utf8 = "中\nfn app_start() {\n}\n";
    let mut m = map.clone();
    m.user_source_bytes = utf8.len();
    m.user_source_sha256 = sha256_hex(utf8.as_bytes());
    m.segments = vec![Segment {
        start: 1,
        end: utf8.len(),
        module: None,
        node: None,
        event: None,
        helper: None,
        precision: Precision::Generated,
        source_start: None,
        source_end: None,
    }];
    assert!(m.validate(utf8).is_err(), "非字符边界必须拒用");
}

#[test]
fn mutation_rejects_escape_keys_and_fake_precision() {
    let (project, _util) = project_with_util();
    let src = emit_like_pooled(&project.modules["util"].source, false);
    for bad in ["../evil", "a/b", "C:\\x", "a:b"] {
        let mut m = valid_map(&project, &src);
        m.modules.insert(bad.into(), m.modules["util"].clone());
        assert!(m.validate(&src).is_err(), "escape key {bad} must reject");
    }
    // 伪精度：generated 段不得携带 module/source。
    let mut m = valid_map(&project, &src);
    let gen = m
        .segments
        .iter_mut()
        .find(|s| s.precision == Precision::Generated)
        .unwrap();
    gen.module = Some("util".into());
    assert!(m.validate(&src).is_err());
    // mapped 段必须有源区间。
    let mut m = valid_map(&project, &src);
    let seg = m
        .segments
        .iter_mut()
        .find(|s| s.precision == Precision::Exact)
        .unwrap();
    seg.source_end = None;
    assert!(m.validate(&src).is_err());
}
