//! auto-ui PLAN-054 mixed_regions_lifecycle_storage: 冷 A→B 消费与存储身份链
//! （V-S05/AC-05；T-06）。冷消费者 B（全新 MixedRegionsUiApp，无生产者状态）
//! 读同一冻结产物观测同一链；身份链在消费入口强制——损坏 world.at（sha
//! 漂移）、漂移 closure 模块、漂移 schema/生成器身份各自在任何 Run 之前具名
//! 拒绝；精确修复（恢复字节）在**下一次实际 Run**恢复，无需生产者帮助。
//! Run: cargo test --locked --test mixed_regions_lifecycle_storage -- --test-threads=1
use autographics_auto_driver::node_mixed::parse_mixed_artifact;
use autographics_auto_driver::node_mixed_regions::{
    parse_mixed_regions_artifact, MixedRegionsUiApp,
};

const SWAP_UI: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/ui.json"
);
const SWAP_WORLD: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/world.at"
);
const SWAP_IDENTITY: &str = include_str!(
    "../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/identity.json"
);
const CLOSURE_BAR: &str =
    include_str!("../../autographics-node2d-pilot/generated-node-mixed-regions/ui-panel-swapped/closure/bar.at");

#[test]
fn cold_consumer_b_reads_same_products_and_drives_both_dimensions() {
    // A（生成器）→ B（本消费者）：冷 app；UI 维（按钮）与图形维（实例区域
    // 点击）走同一 Session 队列与同一已接受状态缓存。
    let mut b = MixedRegionsUiApp::new(SWAP_UI, SWAP_WORLD, "plan054-storage-b").expect("cold B");
    b.start_run().expect("run");
    // UI 维：bump 按钮 → count=1。
    b.ui
        .enqueue_click(0, "panel_ui_click_bump".into(), None, "cold-b".into());
    let r = b.drain();
    assert_eq!(r.dispatched, 1);
    assert_eq!(b.ui.texts(0)[0], "1");
    assert!(b.ui.error.is_none());
    // 图形维：组件实例区域点击（Panel/b1 带 def_idx 0 实参）。冷消费者
    // 自持挂载代次注册表（fail-closed 合同：无注册表时区域声明一律具名
    // 拒绝——挂载知识由消费者声明）。
    let epochs = std::rc::Rc::new(std::cell::RefCell::new(
        std::collections::HashMap::new(),
    ));
    epochs
        .borrow_mut()
        .insert("Panel/b1#band".to_string(), 1);
    b.ui.set_region_epochs(epochs);
    assert!(b.enqueue_region_click_at("Panel/b1", "band", 1));
    let r = b.drain();
    assert_eq!(r.dispatched, 1, "component region click dispatches with def_idx");
    let st = b.scene_state();
    let b1 = st.iter().find(|s| s.path == "Panel/b1").expect("b1");
    match &b1.elems[0] {
        autographics_auto_driver::node_mixed::SceneElemState::Quad { x, .. } => {
            assert_eq!(*x, 9.0, "scene binding follows the same accepted state");
        }
        _ => panic!("quad"),
    }
}

#[test]
fn damaged_world_copy_named_rejected_and_repairs_next_run() {
    // 损坏副本（一字节漂移）在消费入口具名拒绝；恢复字节后下一次实际 Run
    // 直接恢复（无生产者参与）。
    let mut damaged = SWAP_WORLD.to_string();
    let pos = damaged.find("fn panel_ui_click_bump").expect("anchor") + 4;
    damaged.replace_range(pos..pos + 1, "X");
    let err = match MixedRegionsUiApp::new(SWAP_UI, &damaged, "plan054-storage-dmg") {
        Err(e) => e,
        Ok(_) => panic!("damaged copy must be named-rejected"),
    };
    assert!(
        err.contains("behavior source identity"),
        "unexpected diagnostic: {err}"
    );
    // 修复＝原始字节：下一实际 Run（新 app）照常消费。
    let mut b = MixedRegionsUiApp::new(SWAP_UI, SWAP_WORLD, "plan054-storage-fix").expect("repair");
    b.start_run().expect("run");
    assert!(b.ui.error.is_none());
}

#[test]
fn identity_chain_binds_world_schema_and_closure() {
    // identity.json 冻结溯源：api/ui_schema/world sha/closure 逐模块绑定
    // 同一字节面。
    let identity: serde_json::Value =
        serde_json::from_str(SWAP_IDENTITY).expect("identity.json");
    assert_eq!(
        identity["api"], "autographics.node-ui-mixed-regions/2",
        "api drift"
    );
    assert_eq!(identity["ui_schema"], "autographics.node-ui-mixed-regions/2");
    let world_sha = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(SWAP_WORLD.as_bytes());
        format!("{:x}", h.finalize())
    };
    assert_eq!(
        identity["provenance"]["world_sha256"], world_sha,
        "world sha drift"
    );
    let bar_sha = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(CLOSURE_BAR.as_bytes());
        format!("{:x}", h.finalize())
    };
    assert_eq!(
        identity["module:bar"], bar_sha,
        "closure bar module sha drift"
    );
    // v1/v2 互拒（schema 界）在存储面复述：v2 产物不可入 v1 入口。
    assert!(parse_mixed_artifact(SWAP_UI).is_err());
    let parsed = parse_mixed_regions_artifact(SWAP_UI).expect("v2 parses");
    assert_eq!(parsed.generator, "autographics-node2d-pilot plan054 node-ui-mixed-regions");
}
