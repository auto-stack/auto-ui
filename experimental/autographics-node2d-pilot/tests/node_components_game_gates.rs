//! PLAN-023 components-2d-game 降层门（V-01；T-03）。
//! 正面：T-02 游戏项目（fixtures/components2d-game）以 game2d 作者块降层
//! Ok——两次再生字节一致（V-01 byte-repro）；生成世界携带冻结 closeout
//! 骨架（game_cue/game_events/hud_*_text/game_closeout）、5 触发绑定、
//! 全部声明 handler 的接收 fn（F-8.2，无 forced dispatch 站点）、visq
//! 呈现缓存（F-8.1 game 面）、根标量 count/state（作者）+ gate/ord/np/
//! p0/p1（生成）；scene 为 autographics.scene/8（assets wav+ttf、cues、
//! 四槽 hud）。负面（fixtures/components2d-game/neg-*，全部 module/line/
//! symbol 具名拒绝，不 panic）：区域规则（静态叶目标）/未知 cue/槽 >3/
//! 缺 count/生成基础设施字段被作者声明/game2d 非 root/game2d 进 frozen
//! 档；cue span 越 bank 与 gain 越域以真实 bank 在临时目录验证。运行：
//! cargo test --locked --offline --test node_components_game_gates
use std::fs;
use std::path;

use autographics_node2d_pilot::{
    load_project, lower_project_components, lower_project_components_game, LowerOutputGame,
};

const FIXTURE_DIR: &str = "fixtures/components2d-game";

fn lower_dir(dir: &path::Path) -> Result<LowerOutputGame, String> {
    let project = load_project(&dir.join("main.at")).map_err(|e| e.to_string())?;
    lower_project_components_game(&project, dir).map_err(|e| e.to_string())
}

fn neg_dir(name: &str) -> path::PathBuf {
    path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(name)
}

// ------------------------------------------------------------- 正面 ----

#[test]
fn game_fixture_lowers_with_frozen_game_shape_and_is_byte_stable() {
    // V-01 正面：真实夹具目录（含资产）降层 Ok，两次再生字节一致。
    let dir = path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR);
    let a = lower_dir(&dir).expect("game fixture lowers");
    let b = lower_dir(&dir).expect("game fixture lowers again");
    let wa = a.base.user_source.as_ref().expect("world.at");
    let wb = b.base.user_source.as_ref().expect("world.at");
    assert_eq!(wa, wb, "regeneration is byte-identical (V-01)");

    // F-4 根合同：作者字段 + 生成基础设施，各自只出现一次。
    for (v, n) in [
        ("var r_count=0", 1),
        ("var r_state=0", 1),
        ("var r_gate=0", 1),
        ("var r_ord=0", 1),
        ("var r_np=0", 1),
        ("var r_p0=0", 1),
        ("var r_p1=0", 1),
    ] {
        assert_eq!(wa.matches(v).count(), n, "`{v}` declared exactly once");
    }
    for f in ["count", "state", "gate", "ord", "np", "p0", "p1"] {
        assert!(
            wa.contains(&format!("fn get_r_{f}() int {{return r_{f}}}")),
            "observation getter for r_{f}"
        );
    }

    // F-8.1（game 面）：zone 撞名缓存改 visq，coin 默认名不变（与 T-02
    // 手扩世界同形）；present 比较 visq vs model。
    assert!(wa.contains("var zone_vis=[1,0]"), "zone model SoA");
    assert!(wa.contains("var zone_visq=[1,0]"), "zone renamed cache");
    assert!(wa.contains("var coin_vis=[1,1,1]"), "coin cache keeps _vis");
    assert!(
        wa.contains("if zone_visq[i]!=zone_vis[i] {"),
        "present compares cache vs model"
    );

    // F-8.2：每个声明 handler 的接收 fn 都在（作者无需 forced 站点）。
    for f in [
        "fn c1_collect(j int, n int) {",
        "fn c2_collect(j int, n int) {",
        "fn c3_collect(j int, n int) {",
        "fn player_idle(j int, n int) {",
        "fn pad_enter(j int, n int) {",
        "fn hazard_enter(j int, n int) {",
    ] {
        assert!(wa.contains(f), "receiver `{f}` emitted");
    }
    assert!(
        !wa.contains("r_state == 9"),
        "no forced dispatch site in the game authoring"
    );

    // F-2a/F-2c/F-6：绑定、poll 前奏、closeout 收口。
    assert_eq!(wa.matches("ag_trigger_bind(").count(), 5, "5 binds");
    assert!(
        wa.contains("fn main_lifetick(dt float) {\n game_events()"),
        "poll prologue first in the root lifetick"
    );
    assert!(
        wa.contains(" game_closeout()\n}"),
        "closeout last in app_update"
    );
    for f in [
        "fn game_cue(c int) {",
        "fn game_events() {",
        "fn game_closeout() {",
    ] {
        assert!(wa.contains(f), "closeout skeleton `{f}`");
    }
    assert!(
        wa.contains("1, hud_count_text(r_count), 1, hud_state_text(r_state), 0, \"\", 0, \"\""),
        "four-slot closeout with the two bound slots"
    );

    // scene：schema 8 + 资产 + cues + 四槽。
    assert!(
        a.base
            .scene_json
            .contains("\"schema\": \"autographics.scene/8\"")
    );
    assert!(a.base.scene_json.contains("\"format\": \"wav\""));
    assert!(a.base.scene_json.contains("\"format\": \"ttf\""));
    for cue in ["coin", "win", "fail"] {
        assert!(
            a.base.scene_json.contains(&format!("\"name\": \"{cue}\"")),
            "scene cue {cue}"
        );
    }
    assert_eq!(a.base.scene_json.matches("\"slot\": ").count(), 4);
}

// ------------------------------------------------------------- 负面 ----

#[test]
fn negative_game_fixtures_fail_with_named_diagnostics_and_no_panic() {
    // 区域规则：触发目标定义无动态 shape（static visible）→ 静态叶形态。
    let err = lower_dir(&neg_dir("neg-region")).err().expect("fails");
    assert!(err.contains("region rule"), "{err}");
    assert!(err.contains("module zone") && err.contains("line "), "{err}");
    assert!(
        err.contains("'hazard'") || err.contains("'pad'") || err.contains("'c1'"),
        "{err}"
    );

    // 未知 cue（cue id 越 scene 范围的降层形态）。
    let err = lower_dir(&neg_dir("neg-cue-unknown"))
        .err()
        .expect("fails");
    assert!(err.contains("unknown game2d cue"), "{err}");
    assert!(err.contains("'nope'") && err.contains("line "), "{err}");

    // 槽 >3。
    let err = lower_dir(&neg_dir("neg-slot4")).err().expect("fails");
    assert!(err.contains("slot index must be 0..3"), "{err}");
    assert!(err.contains("line "), "{err}");

    // 缺 count（作者所有字段必须声明；缺 state 同一诊断形态）。
    let err = lower_dir(&neg_dir("neg-missing-count"))
        .err()
        .expect("fails");
    assert!(err.contains("'count'"), "{err}");
    assert!(err.contains("must declare int field"), "{err}");

    // 生成基础设施字段被作者声明（生成器不发明游戏状态）。
    let err = lower_dir(&neg_dir("neg-infra-field"))
        .err()
        .expect("fails");
    assert!(err.contains("generated game infrastructure"), "{err}");
    assert!(err.contains("'ord'"), "{err}");

    // game2d 非 root：game2d 块出现在被实例化的定义（Coin）上。
    let err = lower_dir(&neg_dir("neg-nonroot")).err().expect("fails");
    assert!(err.contains("game2d block is root-node only"), "{err}");
    assert!(err.contains("module coin") && err.contains("line "), "{err}");

    // frozen 档拒绝 game2d（不静默忽略）。
    let dir = path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR);
    let project = load_project(&dir.join("main.at")).expect("loads");
    let err = lower_project_components(&project)
        .err()
        .expect("frozen profile rejects game2d");
    let err = format!("{err}");
    assert!(err.contains("components-2d-game"), "{err}");
    assert!(err.contains("module main") && err.contains("line "), "{err}");
}

#[test]
fn cue_span_and_gain_validated_against_the_real_bank() {
    // 真实 bank 在临时目录内：span 越 frames 与 gain 越域都具名拒绝
    // （资产面诊断，需要真实字节；fixtures 携带正本，这里复制）。
    let setup = |tag: u32| -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::SeqCst);
        let src = path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR);
        let dir = path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("lcg-cue-{}-{tag}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        for f in [
            "main.at",
            "coin.at",
            "player.at",
            "zone.at",
            "resources.at",
        ] {
            fs::copy(src.join(f), dir.join(f)).expect("fixture copy");
        }
        let assets = dir.join("assets");
        fs::create_dir_all(&assets).expect("assets");
        for f in [
            "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav",
            "lilita_one_regular.ttf",
            "license.txt",
        ] {
            fs::copy(src.join("assets").join(f), assets.join(f)).expect("asset copy");
        }
        dir
    };
    let main_at = fs::read_to_string(
        path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIR)
            .join("main.at"),
    )
    .expect("main fixture");

    let dir = setup(1);
    let bad_span = main_at.replace("span: (20046, 22504)", "span: (20046, 99999)");
    fs::write(dir.join("main.at"), bad_span).expect("write");
    let err = lower_dir(&dir).err().expect("span beyond bank must fail");
    assert!(
        err.contains("span (20046, 99999) out of the bank frame range"),
        "{err}"
    );
    assert!(err.contains("'fail'"), "{err}");
    assert!(err.contains("module main") && err.contains("line "), "{err}");

    let dir = setup(2);
    let bad_gain = main_at.replace(
        "        cue coin {\n            span: (0, 7403)\n            gain: 1.0\n        }",
        "        cue coin {\n            span: (0, 7403)\n            gain: 1.01\n        }",
    );
    assert_ne!(bad_gain, main_at, "gain mutation must hit the coin cue");
    fs::write(dir.join("main.at"), bad_gain).expect("write");
    let err = lower_dir(&dir).err().expect("gain beyond 1.0 must fail");
    assert!(err.contains("gain 1.01 out of domain"), "{err}");
    assert!(err.contains("'coin'"), "{err}");
    assert!(err.contains("module main") && err.contains("line "), "{err}");
}
