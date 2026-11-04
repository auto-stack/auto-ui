//! PLAN-044 gates：Node 源映射结构/锚定/变异拒用/真实链。
//!
//! 覆盖 V-S01 的映射侧与 V-S03 的拒用侧：pooled helper 跨模块 range
//! 定位与列回算、generated 补齐、lookup、坏 hash/越界/重叠/逃逸 key/伪
//! 精度/非字符边界变异红，以及 helpers 夹具真实 load→lower→map 全链。
//! 真实 SDK compile 全链先证在 driver tests/node_source_diagnostics.rs
//! （T-02/T-04）。

use std::collections::BTreeMap;

use autographics_node2d_pilot::load::{load_project, LoadedModule, Project};
use autographics_node2d_pilot::lower_project_components;
use autographics_node2d_pilot::scene::sha256_hex;
use autographics_node2d_pilot::source_map::{
    build_source_map, fill_generated_gaps, locate_pooled_fns, remap_range_offset, Precision,
    Segment, SourceMap, MAP_FORMAT,
};

const GENERATOR: &str = "node2d-lower[components-2d]";

const HELP_MAIN: &str = include_str!("../fixtures/components2d/helpers/main.at");
const HELP_A: &str = include_str!("../fixtures/components2d/helpers/helpera.at");
const HELP_B: &str = include_str!("../fixtures/components2d/helpers/helperb.at");
const HELP_RES: &str = include_str!("../fixtures/components2d/resources.at");

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

/// 模拟真实发射器：header（generated）+ 按 (module, fn) 排序的 pooled
/// 锚定段（声明与调用处 `{name}(` → `{module}_{name}(`，同 rename_module_calls
/// 对本夹具的净效果）+ 尾部调度（generated）。
fn emit_like_pooled(project: &Project) -> String {
    let mut src = String::from("// GENERATED header\nvar ok=0.0\n");
    let mut fn_names: Vec<(String, String)> = Vec::new();
    for module in project.modules.values() {
        for (fname, _) in &module.fns {
            fn_names.push((module.name.clone(), fname.clone()));
        }
    }
    fn_names.sort();
    for (module_name, fname) in &fn_names {
        let module = &project.modules[module_name];
        let text = autographics_node2d_pilot::lower::slice_fn_source(&module.source, fname)
            .expect("slice");
        let mut renamed = text;
        for (sib, _) in &module.fns {
            renamed = renamed.replace(&format!("{sib}("), &format!("{module_name}_{sib}("));
        }
        src.push_str(&format!("// pooled fn {fname} from {module_name}\n"));
        src.push_str(renamed.trim_end());
        src.push('\n');
    }
    src.push_str("fn app_start() {\n}\n");
    src
}

fn project_with_util() -> Project {
    let (util, main) = fixture_sources();
    let score_start = util.find("fn score").unwrap();
    let score_len = autographics_node2d_pilot::lower::slice_fn_source(&util, "score")
        .expect("score slice")
        .len();
    let double_start = util.find("fn double").unwrap();
    let double_len = autographics_node2d_pilot::lower::slice_fn_source(&util, "double")
        .expect("double slice")
        .len();
    let mut project = Project::default();
    project.modules.insert(
        "util".into(),
        module(
            "util",
            &util,
            vec![
                ("score", score_start, score_start + score_len),
                ("double", double_start, double_start + double_len),
            ],
        ),
    );
    project
        .modules
        .insert("main".into(), module("main", &main, Vec::new()));
    project
}

#[test]
fn pooled_helper_range_and_remap() {
    let project = project_with_util();
    let src = emit_like_pooled(&project);
    let segments = locate_pooled_fns(&project, &src).expect("locate");
    assert_eq!(segments.len(), 2, "double+score 两段（sorted: double 先）");
    // 恒为 range：声明经 `{module}_` 前缀改写。
    for seg in &segments {
        assert_eq!(seg.precision, Precision::Range);
        assert_eq!(seg.module.as_deref(), Some("util"));
    }
    assert_eq!(segments[0].helper.as_deref(), Some("double"));
    assert_eq!(segments[1].helper.as_deref(), Some("score"));
    // remap：score 片段内 `util_double` 调用点的列回算到原 `double` 标识符。
    let score_seg = &segments[1];
    let util_double_pos = src[score_seg.start..score_seg.end].find("util_double").unwrap() + score_seg.start;
    assert!(util_double_pos >= score_seg.start && util_double_pos < score_seg.end);
    let mapped = remap_range_offset(&project, score_seg, &src, util_double_pos).expect("remap");
    let util = &project.modules["util"].source;
    assert_eq!(&util[mapped..mapped + "double".len()], "double");
    // 行号可证：生成片段行号增量 == 原源行号增量（同一相对位置）。
    let marker = "// pooled fn score from util\n";
    let body_start = score_seg.start + marker.len();
    let k = util_double_pos - body_start;
    let gen_rel = src[body_start..body_start + k].bytes().filter(|b| *b == b'\n').count();
    let src_rel = util[score_seg.source_start.unwrap()..score_seg.source_start.unwrap() + k - 5]
        .bytes()
        .filter(|b| *b == b'\n')
        .count();
    assert_eq!(gen_rel, src_rel, "行号增量必须可证（k 含 5 字节插入位移）");
    // 片段外的 offset 与未锚定内容不回算。
    assert!(remap_range_offset(&project, score_seg, &src, score_seg.start + 2).is_none());
}

#[test]
fn build_map_fills_generated_gaps_and_validates() {
    let project = project_with_util();
    let src = emit_like_pooled(&project);
    let world_sha = sha256_hex(src.as_bytes());
    let map = build_source_map(&project, &src, GENERATOR, &world_sha).expect("build");
    map.validate(&src).expect("validate");
    assert_eq!(map.format, MAP_FORMAT);
    assert_eq!(map.user_source_bytes, src.len());
    // 段连续覆盖、不重叠：header(generated) + 2 helper(range) + 尾(generated)。
    assert!(map.segments.len() >= 4);
    let mut cursor = 0;
    for seg in &map.segments {
        assert_eq!(seg.start, cursor, "segments must tile the source");
        cursor = seg.end;
    }
    assert_eq!(cursor, src.len());
    // lookup：header 命中 generated，helper 体内命中 range。
    let helper = map
        .segments
        .iter()
        .find(|s| s.precision == Precision::Range)
        .unwrap();
    assert_eq!(map.lookup(helper.start + 1).unwrap().helper.as_deref(), Some("double"));
    assert_eq!(map.lookup(0).unwrap().precision, Precision::Generated);
    assert_eq!(map.lookup(src.len()), None);
}

// ------------------------------------------------- 真实链：helpers 夹具 --

fn lower_helpers_fixture() -> (Project, String) {
    let dir = std::env::temp_dir().join(format!(
        "plan044-help-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("dir");
    for (name, body) in [
        ("main", HELP_MAIN),
        ("helpera", HELP_A),
        ("helperb", HELP_B),
        ("resources", HELP_RES),
    ] {
        std::fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
    let project = load_project(&dir.join("main.at")).expect("load helpers fixture");
    let lowered = lower_project_components(&project).expect("lower helpers fixture");
    let world = lowered.user_source.expect("world.at");
    std::fs::remove_dir_all(&dir).ok();
    (project, world)
}

#[test]
fn real_chain_same_name_helpers_do_not_cross_files() {
    let (project, world) = lower_helpers_fixture();
    let world_sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    map.validate_against_world(&world).expect("validate");
    // 两个同名 helper 段：各自落在自己的模块，不串文件/实例。
    let a = map
        .segments
        .iter()
        .find(|s| s.module.as_deref() == Some("helpera"))
        .expect("helpera segment");
    let b = map
        .segments
        .iter()
        .find(|s| s.module.as_deref() == Some("helperb"))
        .expect("helperb segment");
    assert_eq!(a.helper.as_deref(), Some("bump"));
    assert_eq!(b.helper.as_deref(), Some("bump"));
    assert!(a.start < b.start || b.start < a.start);
    assert!(!(a.start..a.end).contains(&b.start) && !(b.start..b.end).contains(&a.start));
    // 模块源切片可区分（+1.0 vs +2.0）：身份与内容一致。
    let sa = &project.modules["helpera"].source[a.source_start.unwrap()..a.source_end.unwrap()];
    let sb = &project.modules["helperb"].source[b.source_start.unwrap()..b.source_end.unwrap()];
    assert!(sa.contains("+ 1.0"), "helpera bump adds 1.0");
    assert!(sb.contains("+ 2.0"), "helperb bump adds 2.0");
    // remap：helpera 片段内 `1.0` 回算到 helpera 源；helperb 同理（不互串）。
    let gen_a_1p0 = world[a.start..a.end].find("1.0").unwrap() + a.start;
    let mapped_a = remap_range_offset(&project, a, &world, gen_a_1p0).expect("remap a");
    assert_eq!(
        &project.modules["helpera"].source[mapped_a..mapped_a + 3],
        "1.0"
    );
    let gen_b_2p0 = world[b.start..b.end].find("2.0").unwrap() + b.start;
    let mapped_b = remap_range_offset(&project, b, &world, gen_b_2p0).expect("remap b");
    assert_eq!(
        &project.modules["helperb"].source[mapped_b..mapped_b + 3],
        "2.0"
    );
    // 行号一致：remapped 位置行号 == 生成位置行号（相对片段起点的增量）。
    let line_delta = |src: &str, from: usize, to: usize| {
        src[from..to].bytes().filter(|b| *b == b'\n').count()
    };
    let marker_a = "// pooled fn bump from helpera\n";
    assert_eq!(
        line_delta(&world, a.start + marker_a.len(), gen_a_1p0),
        line_delta(
            &project.modules["helpera"].source,
            a.source_start.unwrap(),
            mapped_a
        )
    );
}

#[test]
fn real_chain_handler_context_segments_anchor_node_event() {
    let (project, world) = lower_helpers_fixture();
    let world_sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    map.validate_against_world(&world).expect("validate");
    // Alpha/Beta 的 Life.Tick handler：context 精度 + node/event 身份 +
    // 声明区间锚（helpera.at / helperb.at 各自的 on 块）。
    let handlers: Vec<_> = map
        .segments
        .iter()
        .filter(|s| s.precision == Precision::Context)
        .collect();
    assert_eq!(handlers.len(), 2, "alpha/beta 各一个 Life.Tick handler");
    for h in &handlers {
        assert_eq!(h.event.as_deref(), Some("Life.Tick"));
        assert!(h.node.as_deref().unwrap() == "Alpha" || h.node.as_deref().unwrap() == "Beta");
        let (ss, se) = (h.source_start.unwrap(), h.source_end.unwrap());
        let src = &project.modules[h.module.as_deref().unwrap()].source;
        assert!(se <= src.len() && ss <= se);
        // 声明区间锚在事件标识符（dialect Handler.span 实证锚事件名 token，
        // 如 "Life"），须是 display 事件名的非空前缀。
        let span_text = &src[ss..se];
        assert!(!span_text.is_empty());
        assert!(
            h.event.as_deref().unwrap().starts_with(span_text),
            "handler 声明区间须锚事件标识符: {span_text:?} vs {:?}",
            h.event
        );
    }
    // 不同模块 handler 不互串：Alpha 段锚 helpera.at，Beta 段锚 helperb.at。
    let a = handlers.iter().find(|h| h.node.as_deref() == Some("Alpha")).unwrap();
    let b = handlers.iter().find(|h| h.node.as_deref() == Some("Beta")).unwrap();
    assert_eq!(a.module.as_deref(), Some("helpera"));
    assert_eq!(b.module.as_deref(), Some("helperb"));
    // lookup：handler 体内偏移命中 context 段且带事件身份。
    let in_body = a.start + world[a.start..a.end].find("helpera_bump").map(|p| p + 4).unwrap();
    let hit = map.lookup(in_body).expect("handler body covered");
    assert_eq!(hit.precision, Precision::Context);
    assert_eq!(hit.event.as_deref(), Some("Life.Tick"));
}

#[test]
fn probe_member_event_handler_segment_covers_recv_fn() {
    // probe 夹具（Panel→meter 业务事件）：Member handler 的唯一发射
    // `fn meter_add(j int, n int)` 必须被锚定为 node/event 上下文段。
    const PROBE_MAIN: &str = include_str!("../fixtures/components2d/main.at");
    const PROBE_PANEL: &str = include_str!("../fixtures/components2d/panel.at");
    const PROBE_GAUGE: &str = include_str!("../fixtures/components2d/gauge.at");
    const PROBE_PLAYER: &str = include_str!("../fixtures/components2d/player.at");
    const PROBE_RES: &str = include_str!("../fixtures/components2d/resources.at");
    let dir = std::env::temp_dir().join(format!(
        "plan044-probe-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("dir");
    for (name, body) in [
        ("main", PROBE_MAIN),
        ("panel", PROBE_PANEL),
        ("gauge", PROBE_GAUGE),
        ("player", PROBE_PLAYER),
        ("resources", PROBE_RES),
    ] {
        std::fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
    let project = load_project(&dir.join("main.at")).expect("load probe");
    let world = lower_project_components(&project)
        .expect("lower probe")
        .user_source
        .expect("world");
    std::fs::remove_dir_all(&dir).ok();
    let world_sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    map.validate_against_world(&world).expect("validate");
    let meter = map
        .segments
        .iter()
        .find(|s| s.event.as_deref() == Some("Add"))
        .expect("member recv segment (meter_add → Gauge.Add)");
    assert_eq!(meter.precision, Precision::Context);
    assert_eq!(meter.node.as_deref(), Some("Gauge"));
    assert_eq!(meter.module.as_deref(), Some("gauge"));
    // 片段确为 recv fn 体（世界内唯一 meter_add 定义）。
    let frag = &world[meter.start..meter.end];
    assert!(frag.starts_with("fn meter_add("), "fragment is the recv fn: {frag:.40}");
    // 源锚 = gauge.at 的 `.Add` 事件 token 行。
    let gauge = &project.modules["gauge"].source;
    let anchor_line = gauge
        .lines()
        .nth(
            gauge[..gauge.find(".Add").unwrap()]
                .bytes()
                .filter(|b| *b == b'\n')
                .count(),
        )
        .unwrap();
    assert!(
        anchor_line.contains(".Add"),
        "anchored at the .Add declaration: {anchor_line}"
    );
    // 唯一属主规则负控：gauge 的 Life handler 仍由 Life 规则覆盖（不重复归属）。
    let life: Vec<_> = map
        .segments
        .iter()
        .filter(|s| s.event.as_deref().map(|e| e.starts_with("Life.")).unwrap_or(false))
        .collect();
    assert!(!life.is_empty(), "life handler segments intact");
}

#[test]
fn utf8_multibyte_columns_recompute_on_scalar_boundaries() {
    // V-S04 映射侧：多字节字符（中文/组合符）存在于 helper 之前时，
    // remap 的列必须按 Unicode scalar（.chars() 口径）独立重算，
    // 且所有段边界落在字符边界上。
    let util = "fn 评分(x int) int {\n  // 中文注释 + e\u{0301} 组合符\n  return x + 1\n}\n".to_string();
    let main = "node Game {\n}\n".to_string();
    let start = util.find("fn 评分").unwrap();
    let len = autographics_node2d_pilot::lower::slice_fn_source(&util, "评分")
        .expect("slice")
        .len();
    let mut project = Project::default();
    project.modules.insert(
        "util".into(),
        module("util", &util, vec![("评分", start, start + len)]),
    );
    project
        .modules
        .insert("main".into(), module("main", &main, Vec::new()));
    // 模拟发射（声明改名 util_评分）。
    let mut world = String::from("// GENERATED\nvar ok=0.0\n");
    world.push_str("// pooled fn 评分 from util\n");
    world.push_str(&util.replace("评分(x int)", "util_评分(x int)").trim_end());
    world.push('\n');
    world.push_str("fn app_start() {\n}\n");
    let world_sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    map.validate_against_world(&world).expect("validate");
    let seg = map
        .segments
        .iter()
        .find(|s| s.helper.as_deref() == Some("评分"))
        .expect("helper segment");
    assert_eq!(seg.precision, Precision::Range);
    // remap `return` 行内的偏移：列按 chars 口径，world 与源一致。
    let ret_off_world = world[seg.start..seg.end].find("return").unwrap() + seg.start;
    let mapped = remap_range_offset(&project, seg, &world, ret_off_world).expect("remap");
    assert_eq!(&util[mapped..mapped + 6], "return");
    let col_world = world[..ret_off_world].rsplit('\n').next().unwrap().chars().count() + 1;
    let col_src = util[..mapped].rsplit('\n').next().unwrap().chars().count() + 1;
    assert_eq!(col_world, col_src, "列必须按 Unicode scalar 独立重算");
    // 段边界字符边界性已由 validate 保证；含多字节源的整图校验通过即边界成立。
}

#[test]
fn crlf_helper_source_remaps_by_bytes() {
    // V-S04：CRLF 模块源的 helper 逐字节 remap（\r\n 两字节照常参与
    // 行号/差分计算；列按 chars 口径）。
    let util = "fn score(x int) int {\r\n  return double(x)\r\n}\r\n".to_string();
    let main = "node Game {\r\n}\r\n".to_string();
    let start = util.find("fn score").unwrap();
    let len = autographics_node2d_pilot::lower::slice_fn_source(&util, "score")
        .expect("slice")
        .len();
    let double_start = util.find("fn double").unwrap_or(util.len()); // 本夹具无 double
    let _ = double_start;
    let mut project = Project::default();
    project
        .modules
        .insert("util".into(), module("util", &util, vec![("score", start, start + len)]));
    project
        .modules
        .insert("main".into(), module("main", &main, Vec::new()));
    let mut world = String::from("// GENERATED\nvar ok=0.0\n");
    world.push_str("// pooled fn score from util\n");
    world.push_str(&util.replace("fn score(", "fn util_score(").trim_end());
    world.push_str("\nfn app_start() {\n}\n");
    let world_sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    map.validate_against_world(&world).expect("validate");
    let seg = map
        .segments
        .iter()
        .find(|s| s.helper.as_deref() == Some("score"))
        .expect("score segment");
    // 无 sibling 调用 → 声明处仍被 `{module}_` 改写（siblings 含自身）。
    assert_eq!(seg.precision, Precision::Range);
    let ret = world[seg.start..seg.end].find("return").unwrap() + seg.start;
    let mapped = remap_range_offset(&project, seg, &world, ret).expect("remap");
    assert_eq!(&util[mapped..mapped + 6], "return");
    let line = util[..mapped].bytes().filter(|b| *b == b'\n').count() + 1;
    assert_eq!(line, 2, "CRLF 源行号重算");
}

#[test]
fn build_map_is_pure_world_bytes_unchanged() {
    // T-03 默认字节保持（单元侧）：建图是纯函数，world 串不被改写；
    // CLI 默认无旁件的对照在 verify_plan044.py（T-06）以真实进程跑。
    let (project, world) = lower_helpers_fixture();
    let before = world.clone();
    let world_sha = sha256_hex(world.as_bytes());
    build_source_map(&project, &world, GENERATOR, &world_sha).expect("build");
    assert_eq!(world, before, "map build must not mutate the world source");
}

#[test]
fn map_size_scales_linearly_and_largest_legal_source_fits() {
    // V-S05：段数/旁件体积随源规模线性增长；最大合法源（近 256KiB
    // user 上限的合成世界）旁件必须仍在冻结预算内，超限分类明确
    //（拒用→generated fallback，不截断不部分采用）。
    fn synth(n_helpers: usize, body_pad: usize) -> (Project, String) {
        // 单模块 n 个 helper（每个带 pad 行）+ 主 node，合成合法世界。
        let mut util = String::new();
        for i in 0..n_helpers {
            util.push_str(&format!("fn h{i}(x int) int {{\n"));
            for j in 0..body_pad {
                util.push_str(&format!("  // pad line {j} 中文字段\u{0301}\n"));
            }
            util.push_str("  return x + 1\n}\n");
        }
        let main = "node Game {\n}\n".to_string();
        let mut fns = Vec::new();
        let mut cursor = 0usize;
        for i in 0..n_helpers {
            let m = format!("fn h{i}(");
            let s = util.find(&m).unwrap();
            let l = autographics_node2d_pilot::lower::slice_fn_source(&util, &format!("h{i}"))
                .unwrap()
                .len();
            fns.push((format!("h{i}"), s, s + l));
            cursor = s + l;
        }
        let _ = cursor;
        let mut project = Project::default();
        project.modules.insert(
            "util".into(),
            LoadedModule {
                name: "util".into(),
                source: util.clone(),
                nodes: Vec::new(),
                fns: fns.into_iter().map(|(n, s, e)| (n, s..e)).collect(),
                resources: Vec::new(),
                uses: Vec::new(),
                enums: BTreeMap::new(),
            },
        );
        project
            .modules
            .insert("main".into(), module("main", &main, Vec::new()));
        // 模拟发射：锚定注释 + 声明改名。
        let mut world = String::from("// GENERATED\nvar ok=0.0\n");
        let mut names: Vec<String> = (0..n_helpers).map(|i| format!("h{i}")).collect();
        names.sort();
        for n in &names {
            world.push_str(&format!("// pooled fn {n} from util\n"));
            let slice = autographics_node2d_pilot::lower::slice_fn_source(&util, n).unwrap();
            let mut renamed = slice;
            for (sib, _) in &project.modules["util"].fns {
                renamed = renamed.replace(&format!("{sib}("), &format!("util_{sib}("));
            }
            world.push_str(renamed.trim_end());
            world.push('\n');
        }
        world.push_str("fn app_start() {\n}\n");
        (project, world)
    }
    let mut prev: Option<(usize, usize)> = None;
    for n in [8usize, 32, 128] {
        let (project, world) = synth(n, 6);
        assert!(world.len() <= 256 * 1024, "synthetic world within user cap");
        let sha = sha256_hex(world.as_bytes());
        let map = build_source_map(&project, &world, GENERATOR, &sha).expect("build");
        let json = map.to_json();
        // 线性：段数 ≈ n 个 range + 补齐段；json/world 比率有界。
        assert!(map.segments.len() >= n && map.segments.len() <= 2 * n + 2);
        if let Some((pn, pj)) = prev {
            // 体积随 helper 数近线性：每 helper 的 json 增量稳定（±50% 内容容差）。
            let per_now = json.len() as f64 / n as f64;
            let per_prev = pj as f64 / pn as f64;
            assert!(
                per_now < per_prev * 1.5 + 200.0,
                "map growth super-linear: {per_now} vs {per_prev}"
            );
        }
        prev = Some((n, json.len()));
    }
    // 实测包线（T-05 重冻结依据，见 Plan §9）：worst ratio 3.42（密集
    // helper 小源）收敛 ~2.2-3.4；上界 = max(64KiB, 4×world)。
    let (project, world) = synth(128, 6);
    let sha = sha256_hex(world.as_bytes());
    let map = build_source_map(&project, &world, GENERATOR, &sha).unwrap();
    let json_len = map.to_json().len();
    let bound = (4 * world.len()).max(64 * 1024);
    assert!(
        json_len <= bound,
        "map {json_len} exceeds T-05 frozen bound {bound} (world {})",
        world.len()
    );
    // 线性存储：段数 ≈ 2n+1，json 增量随 n 稳定（上面循环已断言非超线性）。
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
    let project = project_with_util();
    let src = emit_like_pooled(&project);
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
    let project = project_with_util();
    let src = emit_like_pooled(&project);
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
    let project = project_with_util();
    let src = emit_like_pooled(&project);
    let map = valid_map(&project, &src);
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
    let project = project_with_util();
    let src = emit_like_pooled(&project);
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
        .find(|s| s.precision == Precision::Range)
        .unwrap();
    seg.source_end = None;
    assert!(m.validate(&src).is_err());
}
