//! PLAN-980 × PLAN-981（D-981-F32-CONSUMER-WINDOW）**F32 组件桥**验收（980 侧）。
//!
//! 与 981 分支 `plan-981-f32-window`（`17fafb1d9`）配对：`ComponentFieldValue::F32`
//! 按位相等、`InitialValue::Float` props 种子。本文件验证 980 `auto-ui-runtime` 侧：
//! ①`AceStateValue::F32` 与 981 同策略（按位相等）；②种子 `Float(1.25)` → 一次更新
//! → 读回 `1.5`（bit-exact）；③呈现=Rust `f32` Display（`1.5`，不是 `1.500000`、
//! 不用逗号小数）；④`Int` 种子具名拒（不隐式提升）。
//!
//! 边界：本窗只验**组件桥**；包桥 F32 另案（包桥现只观察 int、无 props/initials）。
use auto_ui_runtime::ui::ace_node_desktop::{ensure_ui980_provider, push_event, NodeDesktopBridge};
use auto_ui_runtime::ui::ace_state_bridge::{
    AceInitialValue, AceStateEngine, AceStateValue, DispatchOutcome,
};

/// 最小真实 AGE 源：f32 模型字段 + 一条同型积分 handler（与 981 的 MOVER 同形）。
const MOVER: &str = "node Gauge {\n    model {\n        ratio f32 = 1.25\n    }\n    msg { Step }\n    view {\n        col {\n            text \"g\"\n            button \"Step\" { onclick: .Step }\n        }\n    }\n    on {\n        .Step -> { .ratio = .ratio + 0.25 }\n    }\n}\n";

fn engine() -> AceStateEngine {
    AceStateEngine::new(
        MOVER,
        "f32-window.at",
        auto_ui_runtime::ui::ace_state_bridge::AceAlias::Accept,
    )
    .expect("f32 component source compiles")
}

fn ratio(bridge: &AceStateEngine, id: &str) -> f32 {
    bridge
        .read_all(id)
        .expect("live instance")
        .into_iter()
        .find(|(n, _)| n == "ratio")
        .map(|(_, v)| match v {
            AceStateValue::F32(f) => f,
            other => panic!("ratio is not F32: {other:?}"),
        })
        .expect("ratio field")
}

/// ①种子 `Float(1.25)` 经一次 `.Step`（+0.25）读回 `1.5`；`F32` 按位相等。
#[test]
fn p980_f32_component_bridge_seed_and_update() {
    let mut e = engine();
    e.mount("i0", vec![("ratio".into(), AceInitialValue::Float(1.25))])
        .expect("Float seed accepted");
    assert_eq!(ratio(&e, "i0"), 1.25, "seed read back");
    match e.dispatch("i0", "Step").expect("Step dispatches") {
        DispatchOutcome::Delivered(states) => {
            let got = states
                .iter()
                .find(|(n, _)| n == "ratio")
                .map(|(_, v)| match v {
                    AceStateValue::F32(f) => *f,
                    other => panic!("ratio is not F32: {other:?}"),
                })
                .expect("ratio in delivered states");
            assert_eq!(got.to_bits(), 1.5f32.to_bits(), "bit-exact 1.5");
        }
        other => panic!("expected delivery, got {other:?}"),
    }
    assert_eq!(ratio(&e, "i0").to_bits(), 1.5f32.to_bits());
    // 按位相等策略与 981 对齐：-0.0 != 0.0、NaN payload 可判别。
    assert_ne!(AceStateValue::F32(-0.0), AceStateValue::F32(0.0));
    assert_ne!(
        AceStateValue::F32(f32::from_bits(0x7fc0_0001)),
        AceStateValue::F32(f32::from_bits(0x7fc0_0002))
    );
    assert_eq!(AceStateValue::F32(1.5), AceStateValue::F32(1.5));
}

/// ②`Int` 种子不隐式提升 → 具名拒（与 981 `validate_initials` 同口径）。
#[test]
fn p980_f32_component_bridge_rejects_int_seed() {
    let mut e = engine();
    let err = e
        .mount("i0", vec![("ratio".into(), AceInitialValue::Int(1))])
        .expect_err("Int seed must not promote to f32");
    assert!(
        err.contains("must be `f32`") || err.contains("component-unsupported"),
        "named int-seed rejection: {err}"
    );
}

/// ③界面呈现=Rust `f32` Display：窗口状态行 `1.5`（无补零、无逗号小数）。
#[test]
fn p980_f32_component_bridge_display_is_rust_display() {
    ensure_ui980_provider().expect("ui980 provider");
    let mut bridge = NodeDesktopBridge::load(MOVER, "f32-window.at", 1).expect("bridge loads");
    bridge.open().expect("open");
    push_event(0, 0); // view 里唯一的按钮 = Step
    match bridge.step() {
        auto_ui_runtime::ui::ace_node_desktop::StepOutcome::Dispatched { states, .. } => {
            let shown = states
                .iter()
                .find(|(n, _)| n == "ratio")
                .map(|(_, v)| v.clone())
                .expect("ratio row");
            assert_eq!(shown, "1.5", "Rust Display, locale-free");
        }
        other => panic!("expected dispatched Step, got {other:?}"),
    }
    let evidence = bridge.evidence();
    let sessions = evidence["sessions"].as_array().expect("sessions");
    let this = sessions.last().expect("own session");
    let row = this["states"]
        .as_array()
        .unwrap()
        .get(0)
        .expect("instance 0 row")
        .as_array()
        .unwrap()
        .iter()
        .find_map(|kv| {
            let pair = kv.as_array()?;
            (pair[0].as_str() == Some("ratio")).then(|| pair[1].as_str().unwrap().to_string())
        })
        .expect("ratio display row");
    assert_eq!(row, "1.5", "host display model follows the engine");
    assert!(!row.contains(','), "no locale decimal comma");
}
