//! PLAN-017 g11-2d 降层负面门与声明敏感性（V-01；合同 §3/§7）。
//! 每个诊断在执行前给出定位拒绝（文件/构造），无 panic/无限递归；
//! 敏感性证明：改名字/重排声明不改变行为形态，改被用资源/位置则
//! 生成物实际改变。运行：cargo test --offline --test node2d_g11_gates
use std::fs;

const RESOURCES: &str = include_str!("../fixtures/g11-2d/resources.at");
const TYPES: &str = include_str!("../fixtures/g11-2d/types.at");
const PLAYER: &str = include_str!("../fixtures/g11-2d/player.at");
const COIN: &str = include_str!("../fixtures/g11-2d/coin.at");
const HAZARD: &str = include_str!("../fixtures/g11-2d/hazard.at");
const GOAL: &str = include_str!("../fixtures/g11-2d/goal.at");
const VOID: &str = include_str!("../fixtures/g11-2d/void_zone.at");
const PATROL: &str = include_str!("../fixtures/g11-2d/patrol.at");
const MAIN: &str = include_str!("../fixtures/g11-2d/main.at");

fn write_project(dir: &path::Path, main: &str) {
    fs::create_dir_all(dir).expect("dir");
    for (name, body) in [
        ("main", main),
        ("resources", RESOURCES),
        ("types", TYPES),
        ("player", PLAYER),
        ("coin", COIN),
        ("hazard", HAZARD),
        ("goal", GOAL),
        ("void_zone", VOID),
        ("patrol", PATROL),
    ] {
        fs::write(dir.join(format!("{name}.at")), body).expect("write");
    }
}

fn lower(dir: &path::Path, entry: &str) -> Result<String, String> {
    let project = autographics_node2d_pilot::load_project(&dir.join(entry).join("..").join(entry))
        .map_err(|e| e.to_string())?;
    let out = autographics_node2d_pilot::lower_project_g112d(&project)
        .map_err(|e| e.to_string())?;
    Ok(out.user_source)
}

use std::path;

#[test]
fn unbound_event_port_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-unbound");
    let main = MAIN.replace("on_collect: .Collect(Key.Amber)", "on_collect: .Collect(Key.Amber)");
    //  coin 定义要求 on_collect 绑定：构造缺绑定的实例（去掉该 prop 行）。
    let broken = MAIN.replace("                on_collect: .Collect(Key.Amber)\n", "");
    write_project(&dir, &main);
    let project = autographics_node2d_pilot::load_project(&dir.join("main.at"));
    assert!(project.is_ok(), "baseline loads");
    let broken_dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-unbound2");
    write_project(&broken_dir, &broken);
    let project = autographics_node2d_pilot::load_project(&broken_dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = project.expect("unbound event port must be rejected");
    assert!(err.to_string().contains("missing prop 'on_collect'"), "{err}");
}

#[test]
fn receiver_arity_mismatch_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-arity");
    // 根 .GoalLeft 处理器带形参，而 on_left 绑定无载荷 → 定位拒绝。
    let broken = MAIN.replace(".GoalLeft -> {", ".GoalLeft(actor) -> {");
    write_project(&dir, &broken);
    let project = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = project.expect("arity mismatch must be rejected");
    assert!(
        err.to_string().contains("declares 1 param"),
        "{err}"
    );
}

#[test]
fn actor_ref_bound_to_event_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-actor-ev");
    let broken = MAIN.replace(
        "                target: refs.player.actor\n                on_collect: .Collect(Key.Amber)",
        "                target: .NoSuch\n                on_collect: .Collect(Key.Amber)",
    );
    write_project(&dir, &broken);
    let err = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = err.expect("actor port bound to an event must be rejected");
    assert!(
        err.to_string().contains("ActorRef2D ports take refs.<inst>.actor"),
        "{err}"
    );
}

#[test]
fn refs_unknown_instance_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-bad-ref");
    let broken = MAIN.replace("target: refs.player.actor", "target: refs.ghost.actor");
    write_project(&dir, &broken);
    let err = autographics_node2d_pilot::load_project(&dir.join("main.at")).err();
    let err = err.expect("refs to an unknown instance must be rejected");
    assert!(err.to_string().contains("unknown root instance 'ghost'"), "{err}");
}

#[test]
fn solid_outside_root_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-solid");
    let broken = PLAYER.replace(
        "    view-2d {\n        character2d body {",
        "    view-2d {\n        solid2d wall {\n            position: (5.0, 0.0)\n            mesh: resources.ground_mesh\n            material: resources.ground_color\n        }\n        character2d body {",
    );
    write_project(&dir, MAIN);
    fs::write(dir.join("player.at"), broken).expect("write");
    let err = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = err.expect("solid2d outside the root view must be rejected");
    assert!(err.to_string().contains("outside the root view"), "{err}");
}

#[test]
fn second_character_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-char2");
    // Coin 定义再加一个 character2d → 全工程角色数 = 2 → 拒绝。
    let broken = COIN.replace(
        "    view-2d {\n        trigger2d body {",
        "    view-2d {\n        character2d extra {\n            position: (.position_x, .position_y)\n            mesh: resources.player_mesh\n            material: resources.orange\n        }\n        trigger2d body {",
    );
    write_project(&dir, MAIN);
    fs::write(dir.join("coin.at"), broken).expect("write");
    let err = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = err.expect("a second character2d must be rejected");
    assert!(err.to_string().contains("exactly one character2d"), "{err}");
}

#[test]
fn bad_shape_rejected() {
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-shape");
    let broken = HAZARD.replace("shape: circle(radius: 0.5)", "shape: circle(radius: -1.0)");
    write_project(&dir, MAIN);
    fs::write(dir.join("hazard.at"), broken).expect("write");
    let err = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .ok()
        .and_then(|p| autographics_node2d_pilot::lower_project_g112d(&p).err());
    let err = err.expect("non-positive shape radius must be rejected");
    assert!(err.to_string().contains("radius"), "{err}");
}

#[test]
fn rename_and_reorder_keep_behavior_shape() {
    // 敏感性（正控）：改实例名 + 重排声明，生成物结构（pair 数/实例数/
    // 模板数）不变；pair 顺序跟随声明序（非名字）。
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-rename");
    let main2 = MAIN
        .replace("Player player {", "Player walker {").replace("refs.player.actor", "refs.walker.actor")
        .replace("Coin amber_coin {", "Coin item_a {")
        .replace("Coin azure_coin {", "Coin item_b {")
        .replace("Coin rose_coin {", "Coin item_c {")
        .replace("Hazard hazard {", "Hazard saw {")
        .replace("Goal goal {", "Goal door {")
        .replace("VoidZone void_zone {", "VoidZone pit {")
        .replace("node Main {", "node Level {");
    // 重排：把 hazard 实例移到 amber 之前（pair 顺序跟随声明序）。
    let hazard_block_start = main2.find("        Hazard saw {").expect("hazard block");
    let hazard_block_end = main2[hazard_block_start..]
        .find("        }
")
        .map(|i| hazard_block_start + i + "        }
".len())
        .expect("hazard block end");
    let hazard_block = main2[hazard_block_start..hazard_block_end].to_string();
    let head = main2[..hazard_block_start]
        .replacen(
            "        if .amber == 1 {",
            &format!("{hazard_block}        if .amber == 1 {{"),
            1,
        )
        .to_string();
    let mut reordered = String::new();
    reordered.push_str(&head);
    reordered.push_str(&main2[hazard_block_end..]);
    write_project(&dir, &reordered);
    let project = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .expect("renamed project loads");
    let out = autographics_node2d_pilot::lower_project_g112d(&project)
        .expect("renamed project lowers");
    assert!(out.user_source.contains("ag_trigger_bind(5,"), "six pairs");
    assert!(
        out.roles_json.contains("{\"pair\": 0, \"instance\": \"saw\""),
        "pair order follows declaration order: {}",
        out.roles_json
    );
}

#[test]
fn resource_and_position_changes_move_outputs() {
    // 改被用资源 → 模板几何/颜色变；改位置 → 数组值变。
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-mutate");
    let main2 = MAIN.replace("position_x: 8.0", "position_x: 9.5");
    write_project(&dir, &main2);
    let project = autographics_node2d_pilot::load_project(&dir.join("main.at"))
        .expect("mutated project loads");
    let out = autographics_node2d_pilot::lower_project_g112d(&project).expect("lowers");
    assert!(out.user_source.contains("coin_position_x=[9.5,"), "position moved");
    assert!(!out.user_source.contains("coin_position_x=[8.0,"), "old value gone");

    let dir2 = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-mutate2");
    let res2 = RESOURCES.replace(
        "let gold = material2d.color(0.95, 0.8, 0.2, 1.0)",
        "let gold = material2d.color(0.1, 0.2, 0.3, 1.0)",
    );
    fs::create_dir_all(&dir2).expect("dir");
    for (name, body) in [
        ("main", MAIN),
        ("resources", res2.as_str()),
        ("types", TYPES),
        ("player", PLAYER),
        ("coin", COIN),
        ("hazard", HAZARD),
        ("goal", GOAL),
        ("void_zone", VOID),
        ("patrol", PATROL),
    ] {
        fs::write(dir2.join(format!("{name}.at")), body).expect("write");
    }
    let project = autographics_node2d_pilot::load_project(&dir2.join("main.at"))
        .expect("resource-mutated project loads");
    let out = autographics_node2d_pilot::lower_project_g112d(&project).expect("lowers");
    assert!(
        out.scene_json.contains("[0.1, 0.2, 0.3, 1]"),
        "template colour follows the resource symbol: {}",
        out.scene_json
    );
}

#[test]
fn lowered_world_shape_invariants() {
    // 基线世界结构不变量：六对、角色 row 1、模板 3..8、枚举常量、
    // 差值轴、pressed 边沿、round 请求、moved 标记。
    let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("g11g-base");
    write_project(&dir, MAIN);
    let project = autographics_node2d_pilot::load_project(&dir.join("main.at")).expect("loads");
    let out = autographics_node2d_pilot::lower_project_g112d(&project).expect("lowers");
    let src = &out.user_source;
    for needle in [
        "ag_trigger_bind(0,",
        "ag_trigger_bind(5,",
        "ag_char_bind(1)",
        "ag_char_solid(2)",
        "ag_camera_follow_2d(1, 0.0, (-4.0), 30.0, 12.0, 12.0, 8.0)",
        "(ag_action_held(1) - ag_action_held(0))",
        "ag_action_pressed(3)",
        "ag_round_request()",
        "_n2d_moved",
        "ag_event_phase()",
        "coin_bind_COLLECT_0=[0,1,2]",
        "patrol_next(",
    ] {
        assert!(src.contains(needle), "missing {needle}");
    }
    let _ = lower; // keeps the helper referenced
}
