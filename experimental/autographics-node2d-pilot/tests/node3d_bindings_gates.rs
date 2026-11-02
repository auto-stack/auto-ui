//! PLAN-030 components-3d-bindings 降层门（V-S01/V-S02 声明面；AC-01/02/05）。
//! 正面：bg11-main 八组件链降层 Ok 且两次再生长字节一致——三同定义 Coin
//! 的逐实例 slot/template/pair SoA 列（0,1,2 / 3,4,5 / 0,1,2）、跨 Node
//! Actor 主题（player_body_subject_ids[0]）、msg 参数投递（main_moved/
//! main_collected/main_failed/main_won）、根显式阶梯（移动读 1/静止至多
//! 3、n==0 巡逻提交、hy 每拍推进、result 门重开）、Start place y=0.5、
//! triggers3d/1 身份；bg11-alt 参数面（2.6/0.9/0.08，折返仍 ±0.05）；
//! sensor2 双父同定义 receiver/嵌套参数转传/祖先链/第二有限策略。负面：
//! 全部执行前具名拒绝——类别错配、未知池/模板/actor、actor 叶无效、参数
//! 只读、msg 载荷、转传类别、池归属。运行：cargo test --offline --test
//! node3d_bindings_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{
    load_project, lower_project_components3d_bindings, lower_project_components3d_events,
    LowerOutputBasic2D,
};

const MAIN_MAIN: &str = include_str!("../fixtures/components3d-bindings/bg11-main/main.at");
const MAIN_PLAYER: &str = include_str!("../fixtures/components3d-bindings/bg11-main/player.at");
const MAIN_COIN: &str = include_str!("../fixtures/components3d-bindings/bg11-main/coin.at");
const MAIN_EXTRA: &str = include_str!("../fixtures/components3d-bindings/bg11-main/extra.at");
const MAIN_PROPS: &str = include_str!("../fixtures/components3d-bindings/bg11-main/props.at");
const ALT_MAIN: &str = include_str!("../fixtures/components3d-bindings/bg11-alt/main.at");
const SENSOR_MAIN: &str = include_str!("../fixtures/components3d-bindings/sensor2/main.at");
const SENSOR_PODS: &str = include_str!("../fixtures/components3d-bindings/sensor2/pods.at");
const SENSOR_MARKERS: &str =
    include_str!("../fixtures/components3d-bindings/sensor2/markers.at");
const SENSOR_SENTINEL: &str =
    include_str!("../fixtures/components3d-bindings/sensor2/sentinel.at");
const SENSOR_VISITOR: &str =
    include_str!("../fixtures/components3d-bindings/sensor2/visitor.at");

const NEG_KIND_MIX: &str =
    include_str!("../fixtures/components3d-bindings/bneg-kind-mix/main.at");
const NEG_ACTOR_BARE: &str =
    include_str!("../fixtures/components3d-bindings/bneg-actor-bare/main.at");
const NEG_ACTOR_UNKNOWN: &str =
    include_str!("../fixtures/components3d-bindings/bneg-actor-unknown/main.at");
const NEG_ACTOR_ELEM: &str =
    include_str!("../fixtures/components3d-bindings/bneg-actor-elem/coin.at");
const NEG_SLOT_UNKNOWN: &str =
    include_str!("../fixtures/components3d-bindings/bneg-slot-unknown/main.at");
const NEG_PAIR_UNKNOWN: &str =
    include_str!("../fixtures/components3d-bindings/bneg-pair-unknown/main.at");
const NEG_TEMPLATE_UNKNOWN: &str =
    include_str!("../fixtures/components3d-bindings/bneg-template-unknown/main.at");
const NEG_REF_WRITE: &str =
    include_str!("../fixtures/components3d-bindings/bneg-ref-write/coin.at");
const NEG_MSG_PAYLOAD: &str =
    include_str!("../fixtures/components3d-bindings/bneg-msg-payload/main.at");
const NEG_POOL_NONROOT: &str =
    include_str!("../fixtures/components3d-bindings/bneg-pool-nonroot/coin.at");
const NEG_ACTOR_GROUP: &str =
    include_str!("../fixtures/components3d-bindings/bneg-actor-group/main.at");
const NEG_FORWARD_KIND_MAIN: &str =
    include_str!("../fixtures/components3d-bindings/bneg-forward-kind/main.at");
const NEG_FORWARD_KIND_PODS: &str =
    include_str!("../fixtures/components3d-bindings/bneg-forward-kind/pods.at");
const NEG_FORWARD_KIND_NMARKER: &str =
    include_str!("../fixtures/components3d-bindings/bneg-forward-kind/nmarker.at");
const NEG_FORWARD_KIND_MARKERS: &str =
    include_str!("../fixtures/components3d-bindings/bneg-forward-kind/markers.at");

fn write_dir(dir: &path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).expect("mkdir");
    for (name, body) in files {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower_bindings(dir: &path::Path) -> Result<LowerOutputBasic2D, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components3d_bindings(&project).map_err(|e| e.to_string())
}

fn case(files: &[(&str, &str)]) -> Result<LowerOutputBasic2D, String> {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3db-{}-{n}", std::process::id()));
    write_dir(&dir, files);
    lower_bindings(&dir)
}

fn reject(files: &[(&str, &str)], needle: &str) {
    let err = case(files).map(|_| ()).expect_err("named rejection expected");
    assert!(
        err.contains(needle),
        "diagnostic '{err}' does not contain '{needle}'"
    );
    // V-S01 合同：每条诊断带定位（module/line/symbol 或 parse span），无
    // panic、无半产物。
    assert!(
        err.contains("symbol '")
            || err.contains(".at")
            || err.contains("at line ")
            || err.contains("research parse"),
        "diagnostic '{err}' lacks a location anchor"
    );
}

fn bg11_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("main", MAIN_MAIN),
        ("player", MAIN_PLAYER),
        ("coin", MAIN_COIN),
        ("extra", MAIN_EXTRA),
        ("props", MAIN_PROPS),
    ]
}

// ------------------------------------------------ 正面：八组件链形状 --

#[test]
fn bg11_main_lowers_ok_and_is_byte_stable() {
    let files = bg11_files();
    let a = case(&files).expect("bg11-main lowers");
    let b = case(&files).expect("bg11-main lowers again");
    let world = a.user_source.as_ref().expect("behavior world");
    assert_eq!(a.source_sha256, b.source_sha256, "two-lower byte equality");
    // 恰八有状态 Node（root+Player+3Coin+Hazard+Goal+VoidTrap）；子 Node
    // 零 Life.Tick——app_update 只驱动根（作者显式调度）。
    assert!(world.contains("// Root: Main; instances (declaration pre-order): Main/hero:Player,Main/ca:Coin,Main/cb:Coin,Main/cc:Coin,Main/hz:Hazard,Main/gl:Goal,Main/vz:VoidTrap"), "{world}");
    assert!(world.contains("fn app_update(dt float) {\n main_lifetick(dt)\n}"), "{world}");
    assert!(!world.contains("_lifetick(0,dt)"), "no child Life.Tick may run: {world}");
    // 三同定义 Coin 的逐实例具名绑定 SoA 列（槽 0,1,2 / 模板行 3,4,5 /
    // pair 0,1,2）——作者零数字 ID，下降期按声明序解析。
    assert!(world.contains("var coin_slot=[0,1,2]"), "{world}");
    assert!(world.contains("var coin_template=[3,4,5]"), "{world}");
    assert!(world.contains("var coin_pair=[0,1,2]"), "{world}");
    // 引用参数经 spawn/bind 到原数值 ABI；Actor 主题 = Player 身体行表。
    assert!(world.contains("coin_h[i] = ag_object_spawn(coin_slot[i], coin_template[i], coin_px[i], coin_py[i], coin_pz[i])"), "{world}");
    assert!(world.contains("ag_trigger_bind(coin_pair[i], player_body_subject_ids[0], 1, coin_h[i], 0, 0.0)"), "{world}");
    assert!(world.contains("ag_trigger_bind(hazard_pair[i], player_body_subject_ids[0], 1, hazard_h[i], 1, 0.5)"), "{world}");
    // msg 参数投递（017 血统）：绑定根事件 recv fn 直呼。
    assert!(world.contains("main_moved()"), "{world}");
    assert!(world.contains("main_collected()"), "{world}");
    assert!(world.contains("main_failed()"), "{world}");
    assert!(world.contains("main_won()"), "{world}");
    // 根显式阶梯：移动拍读 1（无二次读——r_moved=1 时嵌套分支不可达）、
    // 静止拍至多 3、事件拍跳过巡逻提交、hy 每拍推进、result 门重开。
    assert!(world.contains("if (r_moved == 0) {\n   let n1 = ag_event_next()"), "{world}");
    assert!(world.contains("if (n == 0) {\n  hz_commit(0)\n }\n hz_advance(0)"), "{world}");
    assert!(world.contains("if (r_result != 0) {\n  if (ag_action_pressed(5) == 1) {\n   ag_round_request()"), "{world}");
    // Start place（编辑角色 y=1、Start y=0.5 走线）+ 角色/相机注入位。
    assert!(world.contains("ag_set_local(player_body_rows[i], 1, 0.5)"), "{world}");
    assert!(world.contains("ok=ok+ag_char3_bind(2)"), "{world}");
    assert!(world.contains("ok=ok+ag_camera_orbit_3d(2, 8.0, 45.0, 0, 0.0)"), "{world}");
    // 根调度先推进后消费（hero.Move 在 event.next 之前）。
    let tick_at = world.find("fn main_lifetick").expect("tick");
    let move_at = world[tick_at..].find("hero_move(0, dt)").expect("move call") + tick_at;
    let next_at = world[tick_at..].find("ag_event_next()").expect("next") + tick_at;
    assert!(move_at < next_at, "player msg must run before event reads");
    // 身份串与 API。
    let scene = &a.scene_json;
    assert!(scene.contains("\"document_id\": \"plan030-components-3d-bindings-main\""), "{scene}");
    assert!(scene.contains("\"api\": \"autographics.triggers3d/1\""), "{scene}");
}

#[test]
fn bg11_alt_pins_the_parameter_face() {
    let mut files = bg11_files();
    files[0] = ("main", ALT_MAIN);
    let a = case(&files).expect("bg11-alt lowers");
    let world = a.user_source.as_ref().expect("behavior world");
    // 主/alt 仅作者参数差：speed 2.6、Saw 基线 0.9、初始率 0.08（折返
    // 常量仍 ±0.05 —— 028 alt 同值）。
    assert!(world.contains("var player_speed=[2.6]"), "{world}");
    assert!(world.contains("var hazard_hy=[0.9]"), "{world}");
    assert!(world.contains("var hazard_hdir=[0.08]"), "{world}");
    assert!(world.contains("var hazard_base_y=[0.9]"), "{world}");
    assert!(world.contains("hazard_hdir[j] = (0.0 - 0.05)"), "{world}");
    assert!(world.contains("hazard_hdir[j] = 0.05"), "{world}");
}

#[test]
fn sensor2_pins_forwarding_two_parents_and_second_policy() {
    let files = [
        ("main", SENSOR_MAIN),
        ("pods", SENSOR_PODS),
        ("markers", SENSOR_MARKERS),
        ("sentinel", SENSOR_SENTINEL),
        ("visitor", SENSOR_VISITOR),
    ];
    let a = case(&files).expect("sensor2 lowers");
    let world = a.user_source.as_ref().expect("behavior world");
    // 两父同定义 receiver（Pod×2，各带两 Marker 子实例）+ 嵌套参数转传
    // （父 px/tag 逐实例传到子：tag 列 [1,1,2,2]）。
    assert!(world.contains("// Root: Main; instances (declaration pre-order): Main/pod_a:Pod,Main/pod_a/m1:Marker,Main/pod_a/m2:Marker,Main/pod_b:Pod,Main/pod_b/m1:Marker,Main/pod_b/m2:Marker,Main/watch:Sentinel"), "{world}");
    assert!(world.contains("var marker_tag=[1,1,2,2]"), "{world}");
    // 子投递表按父实例分派（pod_a 的 m1 → 行 0，pod_b 的 m1 → 行 2）。
    assert!(world.contains("var m1_bind_PING=[0,2]"), "{world}");
    assert!(world.contains("var m2_bind_PING=[1,3]"), "{world}");
    // 祖先链：Marker → 最近祖先 Pod（marker_emit_PINGED），Pod 再向上。
    assert!(world.contains("var marker_emit_PINGED=[0,0,1,1]"), "{world}");
    assert!(world.contains("pod_relayed[j] = (pod_relayed[j] + 1)"), "{world}");
    // 第二有限策略：非移动耦合的至多两条读取；root spawn/bind 走点根符号。
    assert!(world.contains("ag_object_spawn(0, 8, 2.0, 0.5, 0.0)"), "{world}");
    let n1_count = world.matches("let n1 = ag_event_next()").count();
    assert_eq!(n1_count, 1, "second policy reads at most two per tick: {world}");
    assert!(!world.contains("let n2 = ag_event_next()"), "{world}");
    // Marker 视觉叶存在（逻辑组件可无 leaf 的对照面）。
    assert!(world.contains("var marker_marker_rows=[2,3,4,5]"), "{world}");
}

// ------------------------------------------------ 负面：具名拒绝面 --

#[test]
fn named_rejections_cover_the_reference_surface() {
    // 类别错配：pair 根进 slot 端口（Slot 与 Pair 不能互传）。
    let mut files = bg11_files();
    files[0] = ("main", NEG_KIND_MIX);
    reject(&files, "wrong symbol root");
    // actor 裸名不可解析（parse 期具名拒绝，带位置）。
    let mut files = bg11_files();
    files[0] = ("main", NEG_ACTOR_BARE);
    reject(&files, "UndefinedVariable");
    // 未知 actor 路径（作用域门：静态实例路径必须存在于本 Run）。
    let mut files = bg11_files();
    files[0] = ("main", NEG_ACTOR_UNKNOWN);
    reject(&files, "unknown actor path 'refs.nobody'");
    // actor 主题元素无效（非目标定义静态叶）。
    let mut files = bg11_files();
    files[2] = ("coin", NEG_ACTOR_ELEM);
    reject(&files, "not a static view-3d leaf of the bound actor");
    // 未知槽/pair/模板。
    let mut files = bg11_files();
    files[0] = ("main", NEG_SLOT_UNKNOWN);
    reject(&files, "binds unknown slot 'item_x'");
    let mut files = bg11_files();
    files[0] = ("main", NEG_PAIR_UNKNOWN);
    reject(&files, "binds unknown pair 'nopair'");
    let mut files = bg11_files();
    files[0] = ("main", NEG_TEMPLATE_UNKNOWN);
    reject(&files, "binds unknown template 'NoSuch'");
    // 引用参数只读（赋值具名拒绝）。
    let mut files = bg11_files();
    files[2] = ("coin", NEG_REF_WRITE);
    reject(&files, "params are read-only");
    // msg 绑定载荷拒绝（零实参形态）。
    let mut files = bg11_files();
    files[0] = ("main", NEG_MSG_PAYLOAD);
    reject(&files, "carries a payload (zero-arg form only");
    // 池归属：objects3d 仅 root。
    let mut files = bg11_files();
    files[2] = ("coin", NEG_POOL_NONROOT);
    reject(&files, "objects3d block is root-only");
    // actor 指向非实例元素（group/叶不是 actor）。
    let mut files = bg11_files();
    files[0] = ("main", NEG_ACTOR_GROUP);
    reject(&files, "unknown actor path 'refs.ground'");
    // 嵌套转传类别错配（PairRef3D 父参转发进 SlotRef3D 子参）。
    let files = [
        ("main", NEG_FORWARD_KIND_MAIN),
        ("pods", NEG_FORWARD_KIND_PODS),
        ("markers", NEG_FORWARD_KIND_MARKERS),
        ("nmarker", NEG_FORWARD_KIND_NMARKER),
        ("sentinel", SENSOR_SENTINEL),
        ("visitor", SENSOR_VISITOR),
    ];
    reject(&files, "forwards '.pk' with kind mismatch (want SlotRef3D)");
}

#[test]
fn old_events_profile_still_lowers_after_the_bindings_surface() {
    // 028 档字节冻结的声明面守护：同夹具经 components-3d-events 入口在
    // 本 crate 内仍可降层（字节等值由 verify_plan028/flow 冻结源钉死）。
    let files = [
        ("main", include_str!("../fixtures/components3d-events/g11-main/main.at")),
        ("character", include_str!("../fixtures/components3d-events/g11-main/character.at")),
        ("props", include_str!("../fixtures/components3d-events/g11-main/props.at")),
    ];
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("n3de-old-{}-{n}", std::process::id()));
    write_dir(&dir, &files);
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string()).expect("load");
    let a = lower_project_components3d_events(&project)
        .map_err(|e| e.to_string())
        .expect("028 g11-main still lowers");
    let world = a.user_source.as_ref().expect("behavior world");
    assert!(world.contains("character_h0[i] = ag_object_spawn(0, 3, 8.0, 1.0, (-3.0))"), "{world}");
}
