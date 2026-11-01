//! PLAN-028 components-3d-events 降层门（V-S01；AC-01/02/05 编译面）。
//! 正面：g11-main/sensor 声明链降层 Ok 且两次再生长字节一致；typed
//! spawn/bind/event/object/round 端口 1:1 映射（零隐藏读取）、主题文档 id
//! 表、模板行（parked/唯一/kind/scale）、objects3d/triggers 池序、
//! triggers3d/1 身份、triple/alt/oldgen 变体参数面钉死。负面：全部执行前
//! 具名拒绝（module/line/symbol，不 panic）——槽/pair 池包络、模板形状
//! （实例化/params/model/双叶/solid）、池归属、未知名字、bind 主题/形状/
//! 半径域；旧档内置视觉 scale 拒绝保持。运行：cargo test --offline --test
//! node3d_events_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{
    load_project, lower_project_components3d_events, lower_project_components3d_runtime,
    LowerOutputBasic2D,
};

const MAIN_MAIN: &str = include_str!("../fixtures/components3d-events/g11-main/main.at");
const MAIN_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/g11-main/character.at");
const MAIN_PROPS: &str = include_str!("../fixtures/components3d-events/g11-main/props.at");
const ALT_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/g11-alt/character.at");
const TRIPLE_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/g11-triple/character.at");
const OLDGEN_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/g11-oldgen/character.at");
const SENSOR_MAIN: &str = include_str!("../fixtures/components3d-events/sensor/main.at");
const SENSOR_MARKERS: &str = include_str!("../fixtures/components3d-events/sensor/markers.at");
const SENSOR_SENTINEL: &str =
    include_str!("../fixtures/components3d-events/sensor/sentinel.at");
const SENSOR_VISITOR: &str =
    include_str!("../fixtures/components3d-events/sensor/visitor_def.at");

const NEG_SLOT_NINE: &str = include_str!("../fixtures/components3d-events/neg-slot-nine/main.at");
const NEG_SLOT_NINE_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-slot-nine/character.at");
const NEG_SLOT_NINE_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-slot-nine/props.at");
const NEG_PAIR_NINE: &str = include_str!("../fixtures/components3d-events/neg-pair-nine/main.at");
const NEG_PAIR_NINE_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-pair-nine/character.at");
const NEG_INSTANCED: &str =
    include_str!("../fixtures/components3d-events/neg-template-instanced/main.at");
const NEG_INSTANCED_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-template-instanced/character.at");
const NEG_INSTANCED_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-template-instanced/props.at");
const NEG_PARAMS: &str =
    include_str!("../fixtures/components3d-events/neg-template-params/main.at");
const NEG_PARAMS_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-template-params/character.at");
const NEG_PARAMS_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-template-params/props.at");
const NEG_MODEL: &str =
    include_str!("../fixtures/components3d-events/neg-template-model/main.at");
const NEG_MODEL_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-template-model/character.at");
const NEG_MODEL_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-template-model/props.at");
const NEG_TWO_LEAVES: &str =
    include_str!("../fixtures/components3d-events/neg-template-two-leaves/main.at");
const NEG_TWO_LEAVES_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-template-two-leaves/character.at");
const NEG_TWO_LEAVES_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-template-two-leaves/props.at");
const NEG_TPL_SOLID: &str =
    include_str!("../fixtures/components3d-events/neg-template-solid/main.at");
const NEG_TPL_SOLID_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-template-solid/character.at");
const NEG_TPL_SOLID_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-template-solid/props.at");
const NEG_POOL_NONROOT: &str =
    include_str!("../fixtures/components3d-events/neg-pool-nonroot/main.at");
const NEG_POOL_NONROOT_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-pool-nonroot/character.at");
const NEG_UNKNOWN_SLOT: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-slot/main.at");
const NEG_UNKNOWN_SLOT_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-slot/character.at");
const NEG_UNKNOWN_SLOT_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-slot/props.at");
const NEG_UNKNOWN_TEMPLATE: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-template/main.at");
const NEG_UNKNOWN_TEMPLATE_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-template/character.at");
const NEG_UNKNOWN_TEMPLATE_PROPS: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-template/props.at");
const NEG_UNKNOWN_PAIR: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-pair/main.at");
const NEG_UNKNOWN_PAIR_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-unknown-pair/character.at");
const NEG_BIND_SUBJECT: &str =
    include_str!("../fixtures/components3d-events/neg-bind-non-const-subject/main.at");
const NEG_BIND_SUBJECT_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-bind-non-const-subject/character.at");
const NEG_BIND_SHAPE: &str =
    include_str!("../fixtures/components3d-events/neg-bind-shape/main.at");
const NEG_BIND_SHAPE_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-bind-shape/character.at");
const NEG_BIND_RADIUS: &str =
    include_str!("../fixtures/components3d-events/neg-bind-radius/main.at");
const NEG_BIND_RADIUS_CHARACTER: &str =
    include_str!("../fixtures/components3d-events/neg-bind-radius/character.at");

fn write_dir(dir: &path::Path, files: &[(&str, &str)]) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_events(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components3d_events(&project).map_err(|e| e.to_string())
}

fn case(files: &[(&str, &str)]) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3de-{}-{n}", std::process::id()));
    write_dir(&dir, files);
    lower_events(&dir)
}

fn reject(files: &[(&str, &str)], needle: &str) {
    let err = case(files).map(|_| ()).expect_err("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    // V-S01 合同：每条诊断带定位（plan 时 module/line/symbol；events
    // translate 时 at line N），无 panic、无半产物。
    assert!(
        err.contains("symbol '") || err.contains(".at") || err.contains("at line "),
        "diagnostic '{err}' lacks a location anchor"
    );
}

// ------------------------------------------------ 正面：形状/字节稳定性 --

#[test]
fn g11_main_lowers_ok_with_typed_ports_and_is_byte_stable() {
    let files = [
        ("main", MAIN_MAIN),
        ("character", MAIN_CHARACTER),
        ("props", MAIN_PROPS),
    ];
    let a = case(&files).expect("g11-main lowers");
    let b = case(&files).expect("g11-main lowers again");
    let world = a.user_source.as_ref().expect("behavior world");
    assert_eq!(a.source_sha256, b.source_sha256, "two-lower byte equality");
    // typed spawn：槽/模板由名字表解析（槽 0..5、模板行 3..8），位置逐字
    // 原 G11 值；Start place 展开 y=0.5（tick0 接受位姿，编辑行仍 y=1）。
    assert!(world.contains("ag_set_local(character_body_rows[i], 1, 0.5)"), "{world}");
    assert!(world.contains("ag_object_spawn(0, 3, 8.0, 1.0, (-3.0))"), "{world}");
    assert!(world.contains("ag_object_spawn(3, 6, 9.0, character_hy[i], (-3.0))"), "{world}");
    assert!(world.contains("ag_object_spawn(5, 8, 3.0, 1.0, (-8.0))"), "{world}");
    // typed bind：pair 0..5、主题=本定义主题表、球对半径 0.5、AABB 0.0。
    assert!(world.contains("var character_body_subject_ids=[2]"), "{world}");
    assert!(world.contains("ag_trigger_bind(3, character_body_subject_ids[i], 1, character_h3[i], 1, 0.5)"), "{world}");
    assert!(world.contains("ag_trigger_bind(0, character_body_subject_ids[i], 1, character_h0[i], 0, 0.0)"), "{world}");
    // 事件读梯（移动 1 条/静止至多 3 条）+ 巡逻跳过 + 重开请求：全部作者代码。
    assert!(world.contains("let n = ag_event_next()"), "{world}");
    assert!(world.contains("let p = ag_event_pair()"), "{world}");
    assert!(world.contains("ag_object_despawn(character_h0[i])"), "{world}");
    assert!(world.contains("if (n == 0) {\n  ag_object_move(character_h3[i], 9.0, character_hy[i], (-3.0))\n }"), "{world}");
    assert!(world.contains("ag_action_pressed(5)"), "{world}");
    assert!(world.contains("ag_round_request()"), "{world}");
    // 相机相对移动（held(1) × right 基 × speed；free basis 读）。
    assert!(world.contains("let hr = ag_action_held(1)"), "{world}");
    assert!(world.contains("ag_cam3_right_x()"), "{world}");
    assert!(world.contains("ag_char3_move("), "{world}");
    // 场景：模板行 park（可见 false、x=-30）、kind/scale 身份、身份串。
    let scene = &a.scene_json;
    assert!(scene.contains("\"document_id\": \"plan028-components-3d-events-main\""), "{scene}");
    assert!(scene.contains("\"api\": \"autographics.triggers3d/1\""), "{scene}");
    assert!(scene.contains("\"name\": \"ShardA\",\n   \"kind\": \"mob\""), "{scene}");
    assert!(scene.contains("\"scale\": [\n    0.6,\n    0.6,\n    0.6\n   ]"), "{scene}");
    assert!(scene.contains("\"name\": \"Saw\""), "{scene}");
    // 模板行不可见（park 由作者声明携带，降层不造魔法）。
    let vis_count = scene.matches("\"visible\": false").count();
    assert_eq!(vis_count, 6, "six parked templates, got {vis_count}: {scene}");
}

#[test]
fn alt_and_triple_variants_pin_their_parameter_faces() {
    fn files(ch: &'static str) -> [(&'static str, &'static str); 3] {
        [
            ("main", MAIN_MAIN),
            ("character", ch),
            ("props", MAIN_PROPS),
        ]
    }
    let base = case(&files(MAIN_CHARACTER)).expect("main lowers");
    let alt = case(&files(ALT_CHARACTER)).expect("alt lowers");
    let triple = case(&files(TRIPLE_CHARACTER)).expect("triple lowers");
    let bw = base.user_source.expect("world").clone();
    let aw = alt.user_source.expect("world");
    let tw = triple.user_source.expect("world");
    // alt：speed 2.6、巡逻基线 0.9/0.08——只有参数行字节不同。
    assert!(aw.contains("var character_speed=[2.6]"), "{aw}");
    assert!(aw.contains("var character_hy=[0.9]"), "{aw}");
    assert!(aw.contains("var character_hbase=[0.9]"), "{aw}");
    assert!(aw.contains("var character_hdir=[0.08]"), "{aw}");
    // triple：三碎片全部 x=3.0（同 tick 三事件的引擎块来源）。
    assert!(tw.contains("ag_object_spawn(0, 3, 3.0, 1.0, (-3.0))"), "{tw}");
    assert!(tw.contains("ag_object_spawn(1, 4, 3.0, 1.0, (-3.0))"), "{tw}");
    assert!(tw.contains("ag_object_spawn(2, 5, 3.0, 1.0, (-3.0))"), "{tw}");
    // oldgen：tick4 同槽重生（同一 slot 名 = 同槽新代）、tick8 显式重绑。
    let og_files = [
        ("main", include_str!("../fixtures/components3d-events/g11-oldgen/main.at")),
        ("character", OLDGEN_CHARACTER),
        ("props", MAIN_PROPS),
    ];
    let og = case(&og_files).expect("oldgen lowers");
    let ow = og.user_source.expect("world");
    assert!(ow.contains("ag_object_spawn(0, 3, 3.0, 1.0, (-3.0))"), "{ow}");
    assert!(ow.contains("ag_object_spawn(0, 4, 3.0, 1.0, (-3.0))"), "{ow}");
    assert!(ow.contains("if (character_tick[i] == 8)"), "{ow}");
    assert!(ow.contains("ag_trigger_bind(0, character_body_subject_ids[i], 1, character_h1[i], 0, 0.0)"), "{ow}");
    assert!(!ow.contains("ag_char3_solid"), "oldgen root carries no solid ground");
    assert_ne!(base.source_sha256, og.source_sha256);
    let _ = bw;
}

#[test]
fn sensor_lowers_with_msg_routing_and_instance_isolation() {
    let files = [
        ("main", SENSOR_MAIN),
        ("markers", SENSOR_MARKERS),
        ("sentinel", SENSOR_SENTINEL),
        ("visitor_def", SENSOR_VISITOR),
    ];
    let a = case(&files).expect("sensor lowers");
    let world = a.user_source.as_ref().expect("behavior world");
    // 两条 msg 面：向下投递（m_a/m_b 实例表 + 接收 fn）与向上冒泡
    // （main_pinged）；两个同定义 Marker 独立 SoA 状态。
    assert!(world.contains("var marker_pings=[0,0]"), "{world}");
    assert!(world.contains("fn m_a_ping(j int, n int)"), "{world}");
    assert!(world.contains("fn m_b_ping(j int, n int)"), "{world}");
    assert!(world.contains("fn main_pinged(tag int)"), "{world}");
    assert!(world.contains("m_a_ping(0, 0)"), "{world}");
    assert!(world.contains("m_b_ping(1, 0)"), "{world}");
    // 静态叶主题 bind（pad 文档 id 1/2 字面主题）+ visitor 句柄源
    // （source_kind 1，root 变量）。
    assert!(world.contains("ag_trigger_bind(0, 1, 1, r_h, 0, 0.0)"), "{world}");
    assert!(world.contains("ag_trigger_bind(1, 2, 1, r_h, 0, 0.0)"), "{world}");
    // 非游戏：无 collect/win/fail 命名业务（改名正控）。
    assert!(!world.to_lowercase().contains("collect"), "{world}");
    assert!(!world.to_lowercase().contains("goal"), "{world}");
}

// ------------------------------------------------ 负面：执行前具名拒绝 --

#[test]
fn pool_envelope_and_ownership_rejections_are_named() {
    reject(
        &[
            ("main", NEG_SLOT_NINE),
            ("character", NEG_SLOT_NINE_CHARACTER),
            ("props", NEG_SLOT_NINE_PROPS),
        ],
        "objects3d slot pool exceeds the frozen 8-slot pool (9 names)",
    );
    reject(
        &[
            ("main", NEG_PAIR_NINE),
            ("character", NEG_PAIR_NINE_CHARACTER),
        ],
        "triggers pair pool exceeds the frozen 8-pair pool (9 names)",
    );
    reject(
        &[
            ("main", NEG_POOL_NONROOT),
            ("character", NEG_POOL_NONROOT_CHARACTER),
        ],
        "objects3d block is root-only (the Run object slot pool is one Run-owned table)",
    );
}

#[test]
fn template_shape_rejections_are_named() {
    reject(
        &[
            ("main", NEG_INSTANCED),
            ("character", NEG_INSTANCED_CHARACTER),
            ("props", NEG_INSTANCED_PROPS),
        ],
        "object3d template 'Ghost' must not be instanced in any view",
    );
    reject(
        &[
            ("main", NEG_PARAMS),
            ("character", NEG_PARAMS_CHARACTER),
            ("props", NEG_PARAMS_PROPS),
        ],
        "object3d template must not declare params",
    );
    reject(
        &[
            ("main", NEG_MODEL),
            ("character", NEG_MODEL_CHARACTER),
            ("props", NEG_MODEL_PROPS),
        ],
        "object3d template 'Ghost' must not declare model/msg/on",
    );
    reject(
        &[
            ("main", NEG_TWO_LEAVES),
            ("character", NEG_TWO_LEAVES_CHARACTER),
            ("props", NEG_TWO_LEAVES_PROPS),
        ],
        "object3d template 'Ghost' must have exactly one view-3d leaf element (got 2 elems, 0 groups)",
    );
    reject(
        &[
            ("main", NEG_TPL_SOLID),
            ("character", NEG_TPL_SOLID_CHARACTER),
            ("props", NEG_TPL_SOLID_PROPS),
        ],
        "object3d template leaf must not be solid",
    );
}

#[test]
fn name_and_domain_rejections_are_named() {
    reject(
        &[
            ("main", NEG_UNKNOWN_SLOT),
            ("character", NEG_UNKNOWN_SLOT_CHARACTER),
            ("props", NEG_UNKNOWN_SLOT_PROPS),
        ],
        "unknown spawn slot 'nope' (objects3d pool)",
    );
    reject(
        &[
            ("main", NEG_UNKNOWN_TEMPLATE),
            ("character", NEG_UNKNOWN_TEMPLATE_CHARACTER),
            ("props", NEG_UNKNOWN_TEMPLATE_PROPS),
        ],
        "unknown spawn template 'Ghost2' (no object3d template def of that name)",
    );
    reject(
        &[
            ("main", NEG_UNKNOWN_PAIR),
            ("character", NEG_UNKNOWN_PAIR_CHARACTER),
        ],
        "unknown bind pair 'ghost' (triggers pool)",
    );
    reject(
        &[
            ("main", NEG_BIND_SUBJECT),
            ("character", NEG_BIND_SUBJECT_CHARACTER),
        ],
        "trigger bind subject 'refs.ground' is not an element of this definition",
    );
    reject(
        &[
            ("main", NEG_BIND_SHAPE),
            ("character", NEG_BIND_SHAPE_CHARACTER),
        ],
        "bind shape must be aabb or sphere (nominal symbols)",
    );
    reject(
        &[
            ("main", NEG_BIND_RADIUS),
            ("character", NEG_BIND_RADIUS_CHARACTER),
        ],
        "sphere bind radius must be finite within (0, 100000]",
    );
}

#[test]
fn old_profile_builtin_scale_rejection_is_preserved() {
    // 027 档边界不拓宽：mob3d 携带静态 scale 仍被具名拒绝（模板 scale
    // 只在新 events 档开放）。
    let src = "use character: Character\nnode Main {\n    view-3d {\n        Character hero { }\n    }\n}\n";
    let ch = "node Character {\n    view-3d {\n        mob3d body {\n            position: (0.0, 1.0, 0.0)\n            scale: (0.5, 0.5, 0.5)\n        }\n    }\n    character3d { }\n}\n";
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3de-legacy-{}-{n}", std::process::id()));
    write_dir(&dir, &[("main", src), ("character", ch)]);
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string()).expect("load");
    let err = lower_project_components3d_runtime(&project)
        .err()
        .expect("runtime profile must reject mob3d scale")
        .to_string();
    assert!(
        err.contains("mob3d has intrinsic built-in geometry and takes no 'scale' prop"),
        "{err}"
    );
}
