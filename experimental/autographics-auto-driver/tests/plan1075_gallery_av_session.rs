//! PLAN-1075（T-02/T-03：Gallery 公共 AV 会话接线——永久用例）。
//!
//! 冻结的 Gallery 作者闭包（`tests/fixtures_plan1075/`，来源 age-examples
//! `ea0fc59`，逐文件 SHA256 见 manifest.json）经 `AgeSession::open_modules`
//! 多模块受检前门开为 **AV 会话**：typed 场景行/资源/观察值对 g01 全 5 变体
//! 与 g02/main 的原验收事实（B01-01..12、g02 静态 GLB），并核会话纪律
//! （坏编辑保旧、模块热更新恢复、Run/Stop 账面、PID 稳定）与接线负控
//! （连字符入口 id 必须由适配层净化、未名模块/未知入口具名拒）。
//!
//! 其余 50 个 Gallery 闭包（g03–g12 嵌套 group2d/角色/精灵/查询/相机/音频
//! 面）超出本批受检前门形状（复用不扩门，见计划 §2/§5）——具名 pending 台账
//! 在计划 §9 T-02 普查收据，不在此伪造绿。
use ace_core::source::age_package::AgeSceneValue;
use autographics_auto_driver::age_session::AgeSession;

fn fixture(closure: &str) -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures_plan1075")
        .join(closure);
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{closure}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            (
                p.file_stem().unwrap().to_string_lossy().replace('-', "_"),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

fn open(closure: &str, entry: &str) -> AgeSession {
    let modules = fixture(closure);
    AgeSession::open_modules(&modules, entry, 0, Vec::new())
        .unwrap_or_else(|ds| panic!("{closure}: {ds:?}"))
}

/// 入口组件（`app::<entry>` 模块的 `Main`）的场景行——包描述还含 `Disc` 等
/// 被引用组件自身的参数化行（`position: (x, y)` 不可解码），不属于入口行序。
fn rows_of(sess: &AgeSession, entry: &str) -> Vec<(String, String, Vec<(String, AgeSceneValue)>)> {
    let key = format!("app::{entry}");
    sess.compiled()
        .description
        .components
        .iter()
        .find(|c| c.module == key && c.name == "Main")
        .unwrap_or_else(|| panic!("entry component {key}::Main missing"))
        .scene
        .iter()
        .map(|r| ("Main".to_string(), r.name.clone(), r.cpu.clone()))
        .collect()
}

fn cpu_f32(cpu: &[(String, AgeSceneValue)], key: &str) -> f32 {
    match cpu.iter().find(|(k, _)| k == key) {
        Some((_, AgeSceneValue::Float(v))) => *v,
        other => panic!("cpu[{key}] not a Float: {other:?}"),
    }
}

fn cpu_floats(cpu: &[(String, AgeSceneValue)], key: &str) -> Vec<f32> {
    match cpu.iter().find(|(k, _)| k == key) {
        Some((_, AgeSceneValue::Floats(v))) => v.clone(),
        other => panic!("cpu[{key}] not Floats: {other:?}"),
    }
}

fn cpu_symbol(cpu: &[(String, AgeSceneValue)], key: &str) -> String {
    match cpu.iter().find(|(k, _)| k == key) {
        Some((_, AgeSceneValue::Symbol(v))) => v.clone(),
        other => panic!("cpu[{key}] not a Symbol: {other:?}"),
    }
}

fn cpu_bool(cpu: &[(String, AgeSceneValue)], key: &str) -> bool {
    match cpu.iter().find(|(k, _)| k == key) {
        Some((_, AgeSceneValue::Bool(v))) => *v,
        other => panic!("cpu[{key}] not a Bool: {other:?}"),
    }
}

fn material_components(sess: &AgeSession, module: &str, name: &str) -> Vec<f32> {
    let key = format!("app::{module}");
    sess.compiled()
        .description
        .resources
        .iter()
        .find(|r| r.module == key && r.name == name)
        .unwrap_or_else(|| panic!("resource {key}::{name} missing"))
        .components
        .clone()
}

/// B01 基线：四行按声明序（= 2D 绘制序/混合序），几何/材质/可见性与原
/// scene（dbb4b75）一致：红圆 (-20,10) d30 a0.5、蓝方 (0,0) 30 a0.5、
/// 绿圆 (25,-15) d24 a0.6、黄方 (-45,-25) 12 a1.0 **hidden**。
#[test]
fn plan1075_g01_base_rows_match_original_baseline() {
    let sess = open("g01-main/main", "main");
    let rows = rows_of(&sess, "main");
    let names: Vec<&str> = rows.iter().map(|(_, n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["red_disc", "blue_quad", "green_disc", "yellow_quad"]
    );

    let red = &rows[0].2;
    assert_eq!(cpu_f32(red, "x"), -20.0);
    assert_eq!(cpu_f32(red, "y"), 10.0);
    assert_eq!(cpu_f32(red, "diameter"), 30.0);
    assert_eq!(cpu_symbol(red, "tint"), "red_alpha");

    let blue = &rows[1].2;
    assert_eq!(cpu_symbol(blue, "kind"), "quad");
    assert_eq!(cpu_floats(blue, "position"), vec![0.0, 0.0]);
    assert_eq!(cpu_floats(blue, "size"), vec![30.0, 30.0]);
    assert_eq!(cpu_symbol(blue, "material"), "blue_alpha");

    let green = &rows[2].2;
    assert_eq!(cpu_f32(green, "x"), 25.0);
    assert_eq!(cpu_f32(green, "y"), -15.0);
    assert_eq!(cpu_f32(green, "diameter"), 24.0);
    assert_eq!(cpu_symbol(green, "tint"), "green_alpha");

    let yellow = &rows[3].2;
    assert_eq!(cpu_floats(yellow, "position"), vec![-45.0, -25.0]);
    assert!(!cpu_bool(yellow, "visible"), "B01-06: yellow hidden");

    // 程序化资源事实（模块 resources，声明序内取值）：alpha 0.5/0.5/0.6/1.0。
    assert_eq!(
        material_components(&sess, "resources", "red_alpha"),
        vec![0.9, 0.3, 0.2, 0.5]
    );
    assert_eq!(
        material_components(&sess, "resources", "blue_alpha"),
        vec![0.2, 0.3, 0.9, 0.5]
    );
    assert_eq!(
        material_components(&sess, "resources", "green_alpha"),
        vec![0.2, 0.8, 0.3, 0.6]
    );
    assert_eq!(
        material_components(&sess, "resources", "yellow_opaque"),
        vec![0.95, 0.9, 0.1, 1.0]
    );
    assert_eq!(
        material_components(&sess, "resources", "deep_green"),
        vec![0.1, 0.8, 0.2, 0.5]
    );

    // AV 观察面：观察覆盖包内全部组件（入口 Main + 被引用 Disc）。
    let observed = sess.observe().expect("observe");
    let names: Vec<&str> = observed
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["Main", "Disc"]);
}

/// B01-09/10（v-color）：蓝方材质换绑 deep_green（蓝消失、绿覆盖变大），
/// 其余行不动——变体是**声明入口**，不是基线产品的字符串替换。
#[test]
fn plan1075_g01_v_color_rebinds_blue_material() {
    let sess = open("g01-main/v-color", "v_color");
    let rows = rows_of(&sess, "v_color");
    assert_eq!(cpu_symbol(&rows[1].2, "material"), "deep_green");
    assert_eq!(cpu_symbol(&rows[0].2, "tint"), "red_alpha", "red 不动");
    assert_eq!(
        cpu_floats(&rows[1].2, "position"),
        vec![0.0, 0.0],
        "蓝位置不动"
    );
}

/// B01-08（v-order）：声明序翻转（蓝先于红）——等 alpha 混合上层面换边。
#[test]
fn plan1075_g01_v_order_flips_declaration_order() {
    let sess = open("g01-main/v-order", "v_order");
    let rows = rows_of(&sess, "v_order");
    let names: Vec<&str> = rows.iter().map(|(_, n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["blue_quad", "red_disc", "green_disc", "yellow_quad"]
    );
}

/// B01-11（v-position）：蓝方移到 (40,0)，红蓝重叠消失、蓝自身仍可见。
#[test]
fn plan1075_g01_v_position_separates_overlap() {
    let sess = open("g01-main/v-position", "v_position");
    let rows = rows_of(&sess, "v_position");
    assert_eq!(cpu_floats(&rows[1].2, "position"), vec![40.0, 0.0]);
    assert_eq!(cpu_symbol(&rows[1].2, "material"), "blue_alpha");
}

/// B01-12（v-visible）：黄方 visible 翻转为 true（出现，身份保留）。
#[test]
fn plan1075_g01_v_visible_shows_yellow() {
    let sess = open("g01-main/v-visible", "v_visible");
    let rows = rows_of(&sess, "v_visible");
    assert!(cpu_bool(&rows[3].2, "visible"));
    assert_eq!(
        cpu_floats(&rows[3].2, "position"),
        vec![-45.0, -25.0],
        "几何不动"
    );
}

/// g02/main：静态 GLB——`model3d badge` 行的 `model:` 具名资源引用与
/// `position (0, 0.6, 0)`；资源端口 Model3D、GLB 路径字符串实参保留。
#[test]
fn plan1075_g02_static_glb_row_and_model_resource() {
    let sess = open("g02-main/main", "main");
    let rows = rows_of(&sess, "main");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, "badge");
    assert_eq!(cpu_symbol(&rows[0].2, "model"), "g02_glb");
    assert_eq!(cpu_floats(&rows[0].2, "position"), vec![0.0, 0.6, 0.0]);
    let res = sess
        .compiled()
        .description
        .resources
        .iter()
        .find(|r| r.name == "g02_glb")
        .expect("g02_glb resource");
    assert_eq!(res.port, "Model3D");
    assert!(res.op.contains("glb"), "op={}", res.op);
    let path = res.symbol.clone().expect("GLB path symbol arg");
    assert!(
        path.ends_with(".glb") && path.contains("42c9da673"),
        "path={path}"
    );
}

/// 会话纪律（V-B-SESSION 的 Gallery 闭包面）：坏编辑保旧（编译失败保留
/// 上一好代与全部观察）、修复后恢复且 body 代数递增；未名模块具名拒；
/// PID 全程稳定；Run/Stop 账面净零。
#[test]
fn plan1075_closure_session_bad_edit_recovery_and_balance() {
    let mut sess = open("g01-main/main", "main");
    let pid = sess.pid();
    let baseline_rows = rows_of(&sess, "main");
    let gen0 = sess.body_generation();

    // 坏编辑：resources 模块删掉 red_alpha 声明（red_disc 的 tint 将无来源）。
    let sabotaged = fixture("g01-main/main")
        .into_iter()
        .find(|(id, _)| id == "resources")
        .map(|(_, src)| src.replace("let red_alpha = material2d.color(0.9, 0.3, 0.2, 0.5)\n", ""))
        .unwrap();
    assert!(
        sess.update_module("resources", &sabotaged).is_err(),
        "坏编辑必须被拒"
    );
    assert_eq!(sess.body_generation(), gen0, "坏编辑不进代数");
    assert_eq!(
        rows_of(&sess, "main"),
        baseline_rows,
        "坏编辑保旧（行事实不变）"
    );
    assert_eq!(
        material_components(&sess, "resources", "red_alpha"),
        vec![0.9, 0.3, 0.2, 0.5],
        "坏编辑保旧（资源事实不变）"
    );

    // 未名模块：具名拒，状态不动。
    let err = sess.update_module("nope", "let x = 1\n").unwrap_err();
    assert!(err[0].contains("module-unknown: nope"), "{err:?}");
    assert_eq!(rows_of(&sess, "main"), baseline_rows);

    // 修复+真实变更：红色改冷色 (0.4, 0.3, 0.9, 0.5)——同一 PID 热更新。
    let edited = fixture("g01-main/main")
        .into_iter()
        .find(|(id, _)| id == "resources")
        .map(|(_, src)| {
            src.replace(
                "let red_alpha = material2d.color(0.9, 0.3, 0.2, 0.5)",
                "let red_alpha = material2d.color(0.4, 0.3, 0.9, 0.5)",
            )
        })
        .unwrap();
    let gen1 = sess
        .update_module("resources", &edited)
        .expect("修复后接受");
    assert_eq!(gen1, gen0 + 1);
    assert_eq!(
        material_components(&sess, "resources", "red_alpha"),
        vec![0.4, 0.3, 0.9, 0.5],
        "热更新后新值可见（同 PID，无 cargo/rustc——进程内 AV 重编译）"
    );
    assert_eq!(sess.pid(), pid, "PID 稳定");

    // 生命周期账面：stop 冻结观察 → run（内部 restart，同 PID）→ stop，净零。
    sess.stop().expect("stop");
    let (runs, stops, active) = sess.run_stop_balance();
    assert_eq!((runs, stops, active), (1, 1, false));
    let frozen = sess.observe().expect("frozen observe");
    drop(frozen);
    assert_eq!(sess.run().expect("run"), 2);
    sess.stop().expect("stop 2");
    let (runs, stops, active) = sess.run_stop_balance();
    assert_eq!((runs, stops, active), (2, 2, false), "Run/Stop 净零");
    assert_eq!(sess.pid(), pid, "restart 后 PID 仍稳定");
    // 停止态拒绝编辑（调度/编辑纪律）。
    assert!(sess.update_module("resources", &edited).is_err());
}

/// 接线负控：未净化的连字符入口 id 被受检前门具名拒（module-identity），
/// 证明净化是接线适配层的职责，不是语言放宽；未知入口具名拒。
#[test]
fn plan1075_entry_id_sanitization_is_wiring_not_language() {
    let mut modules = fixture("g01-main/v-color");
    // 还原作者文件名的连字符（fixtures 文件本身未改名，loader 净化——
    // 这里取原始拼写构造，验证前门拒绝）。
    let entry_index = modules.iter().position(|(id, _)| id == "v_color").unwrap();
    modules[entry_index].0 = "v-color".to_string();
    let err = match AgeSession::open_modules(&modules, "v-color", 0, Vec::new()) {
        Err(ds) => ds,
        Ok(_) => panic!("hyphen entry id must be rejected"),
    };
    assert!(
        err.iter().any(|d| d.contains("module-identity")),
        "连字符 id 必须被拒: {err:?}"
    );
    // 未知入口：具名拒（闭包里没有该模块）。
    let modules = fixture("g01-main/main");
    let err = match AgeSession::open_modules(&modules, "not_there", 0, Vec::new()) {
        Err(ds) => ds,
        Ok(_) => panic!("unknown entry must be rejected"),
    };
    assert!(
        err.iter().any(|d| d.contains("module-entry-missing")),
        "{err:?}"
    );
}
