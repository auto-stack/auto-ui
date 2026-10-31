//! PLAN-016 T-03/T-06 负面门（R1 修复 F-01..F-04/F-09）：闭包后语义校验
//! 的执行前定位拒绝，每类一个最小反例 + 一个合法对照。
//! Run: cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml
//!      --locked --offline --test node2d_lower_gates
use std::path::Path;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/minimal")
            .join(name),
    )
    .expect("fixture")
}

/// 在 fixtures/minimal 副本上叠加替换后加载工程；断言诊断含 `needle`。
fn expect_reject(file: &str, replacements: &[(&str, &str)], needle: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in [
        "main.at",
        "resources.at",
        "types.at",
        "player.at",
        "hazard.at",
        "coin.at",
        "patrol.at",
    ] {
        let mut text = fixture(name);
        if name == file {
            for (from, to) in replacements {
                assert!(
                    text.contains(from),
                    "calibration: '{from}' present in {name}"
                );
                text = text.replace(from, to);
            }
        }
        std::fs::write(dir.path().join(name), text).expect("write");
    }
    let err = autographics_node2d_pilot::load_project(&dir.path().join("main.at"))
        .expect_err("project must be rejected");
    let text = format!("{err}");
    assert!(
        text.contains(needle),
        "diagnosis must contain '{needle}', got: {text}"
    );
}

/// 合法对照：未修改的 minimal 工程必须加载成功。
#[test]
fn control_minimal_project_loads() {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in [
        "main.at",
        "resources.at",
        "types.at",
        "player.at",
        "hazard.at",
        "coin.at",
        "patrol.at",
    ] {
        std::fs::write(dir.path().join(name), fixture(name)).expect("write");
    }
    let project =
        autographics_node2d_pilot::load_project(&dir.path().join("main.at")).expect("loads");
    assert_eq!(project.root, "Main");
}

/// F-01 错导出：具名 use 项不是目标模块导出 → 定位拒绝。
#[test]
fn wrong_export_rejected() {
    expect_reject(
        "main.at",
        &[("use patrol: patrol_y", "use patrol: nonexistent_helper")],
        "does not export 'nonexistent_helper'",
    );
}

/// F-02 坏 actor 引用：refs.<未知实例>.actor → 定位拒绝。
#[test]
fn unknown_actor_ref_rejected() {
    expect_reject(
        "main.at",
        &[("target: refs.player.actor", "target: refs.ghost.actor")],
        "references unknown root instance 'ghost'",
    );
}

/// F-03 未知实例属性：不在 node 参数表内的属性 → 定位拒绝。
#[test]
fn unknown_instance_property_rejected() {
    expect_reject(
        "main.at",
        &[("speed: 2.0", "speed: 2.0\n            spawn_z: 9.0")],
        "unknown property 'spawn_z' on instance 'player'",
    );
}

/// F-04 重复局部绑定：model 字段与参数重名 → 定位拒绝。
#[test]
fn duplicate_binding_rejected() {
    expect_reject(
        "hazard.at",
        &[(
            "        var phase = phase0",
            "        var phase = phase0\n        var x = 0.0",
        )],
        "duplicate binding 'x' on node 'Hazard'",
    );
}

/// F-04 另一形态：参数互重。
#[test]
fn duplicate_parameter_rejected() {
    expect_reject(
        "hazard.at",
        &[("speed: float)", "speed: float, speed: float)")],
        "duplicate parameter 'speed'",
    );
}

/// F-09 Life.Frame 显式越界拒绝（不冒充支持，不静默死代码）。
#[test]
fn life_frame_out_of_scope_rejected() {
    expect_reject(
        "hazard.at",
        &[(
            "        Life.Start -> {",
            "        Life.Frame(dt) -> {}\n        Life.Start -> {",
        )],
        "Life.Frame",
    );
}

/// 合法对照：use 整模块导入（无具名项）不受错导出校验影响。
#[test]
fn whole_module_import_control_loads() {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in [
        "main.at",
        "resources.at",
        "types.at",
        "player.at",
        "hazard.at",
        "coin.at",
        "patrol.at",
    ] {
        let mut text = fixture(name);
        if name == "main.at" {
            text = text.replace("use patrol: patrol_y", "use patrol");
        }
        std::fs::write(dir.path().join(name), text).expect("write");
    }
    let _ = autographics_node2d_pilot::load_project(&dir.path().join("main.at"))
        .expect("whole-module import loads");
    let _ = Path::new("unused");
}

/// F-10：Life.Start 与具名业务 `.Start` 同名共存 → 两个不同处理器
/// （`main_lifestart` vs `main_start`），系统事件与业务事件互不混线。
#[test]
fn life_start_and_business_start_are_distinct() {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in ["resources.at", "patrol.at", "types.at"] {
        std::fs::write(dir.path().join(name), fixture(name)).expect("write");
    }
    let main = r#"
node Main {
    model {
        var starts = 0
        var started = 0
    }
    msg { Start }
    view-2d {
        solid2d ground {
            position: (15.0, -0.5)
            mesh: resources.ground_mesh
            material: resources.ground_color
        }
    }
    on {
        Life.Start -> {
            .starts = 1
        }
        .Start -> {
            .started = 1
        }
        Life.Tick(dt) -> {
            .started = .started
        }
    }
}
"#;
    std::fs::write(dir.path().join("main.at"), main).expect("write");
    let project = autographics_node2d_pilot::load_project(&dir.path().join("main.at"))
        .expect("loads with both handlers");
    let world = autographics_node2d_pilot::lower_project(&project).expect("lowers");
    let src = &world.user_source;
    // 系统事件处理器：Life.Start → lifestart 命名（app_start 调用）。
    assert!(src.contains("fn main_lifestart()"), "life start handler");
    // 业务事件处理器：snake(Start) = start（与 lifestart 不同名）。
    assert!(src.contains("fn main_start()"), "business start handler");
    // 调用面分离：app_start 调 lifestart，绝不调业务 start；业务 start
    // 无自动调用点（仅 emit/显式投递可达）。
    let app_start = src.split("fn app_start").nth(1).expect("app_start");
    assert!(app_start.contains("main_lifestart()"));
    assert!(!app_start.contains("main_start()"));
}
