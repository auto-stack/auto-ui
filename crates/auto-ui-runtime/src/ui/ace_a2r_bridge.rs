//! PLAN-980 RUST（AXR0 先行切片）：同源 AGE 组件经**新 A2R** 生成 Rust 文件
//! 集——组件场景面（`__age_drive` 语义入口）与 F0×A2R 面（967/T-12：text-
//! record 源声明 provider 符号、rustc 链接冻结 provider import lib）。
//!
//! 分工（纪律）：本桥只做**发射**（编译/视图/lower/文件集组装——进程内、零
//! rustc，可进日用例）；真 rustc 编译+运行属**显式构建道**
//! （`examples/ace_a2r_axr0_build.rs` 一次性执行，回执落 target/），日用例
//! 消费回执与发射身份。旧 `rust_ui.rs` AST emitter 路径不动（AXR0 为并行新
//! 面；产品级多文件替换的边界与归属见计划/KNOWN-DEBT）。

use ace_a2x_core::target::{Family, TargetProvider};
use ace_a2x_core::SemanticView;
use ace_av::age_component::{
    compile_component_scenario_with_initials, ComponentInitials, ComponentScenario,
    LegacyWidgetAlias,
};

/// 组件场景 A2R 文件集（Leg A：同源语义 → `__age_drive` i32 入口）。
pub struct ComponentA2rFileset {
    pub files: Vec<(String, String)>,
    pub manifest: String,
    pub input_hash: String,
    pub entry: String,
}

/// 发射 Leg A：AGE 组件源 → 场景编译（共享 verify）→ 语义视图 → lower →
/// record-text 文件集。零 rustc；改变 scenario/handler 即改变产物（身份 oracle）。
pub fn component_a2r_fileset(
    src: &str,
    module_path: &str,
    scenario: &ComponentScenario,
    initials: &ComponentInitials,
    observed: Option<&str>,
) -> Result<ComponentA2rFileset, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario_with_initials(
        src,
        module_path,
        LegacyWidgetAlias::Accept,
        &caps,
        scenario,
        observed,
        initials,
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let view = SemanticView::from_checked(&compiled.checked)
        .map_err(|e| format!("semantic view: {e:?}"))?;
    let program = ace_a2x_rust::lower_program(&view, "__age_drive")
        .map_err(|e| format!("lower: {e:?}"))?;
    let set = ace_a2x_rust::assemble_record_text_file_set(
        &view,
        &program,
        &compiled.input_hash,
        "__age_drive",
    );
    Ok(ComponentA2rFileset {
        files: set.files,
        manifest: set.manifest,
        input_hash: compiled.input_hash,
        entry: "__age_drive".to_string(),
    })
}

/// F0×A2R 文件集（Leg B：967/T-12 冻结桥——text-record 源携带文本字节，
/// generated 声明 provider `#[no_mangle]` 符号；rustc 需链接 provider import
/// lib，由构建道完成）。
pub struct F0A2rFileset {
    pub files: Vec<(String, String)>,
    /// provider 产物（cdylib + import lib）——构建道随文件集同目录布防。
    pub provider_cdylib: std::path::PathBuf,
    pub provider_import_lib: std::path::PathBuf,
}

/// 合成 Leg B 源：同一文本字节经 `text_byte_len` 交给冻结 provider。
pub fn f0_text_source(text: &str) -> String {
    format!(
        "use.rs f0_text\nfn main() int {{ return text_byte_len({}) }}",
        escape_auto_string(text)
    )
}

/// 发射 Leg B：text-record 源（foreign environment = 已准入的冻结 provider）
/// → 文件集（含 extern 块）。零 rustc。`build` 来自预构建的 F0 provider
/// （见 `ace_hello::stable_f0_root` + 新鲜度缓存）。
pub fn f0_a2r_fileset(
    source: &str,
    build: &ace_abi::rust_provider::RustProviderBuild,
) -> Result<F0A2rFileset, String> {
    let authority = ace_abi::rust_provider::f0_authority(build, 1);
    let environment = build
        .manual
        .clone()
        .admit(&authority)
        .map_err(|e| format!("admit: {e}"))?;
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p980axr", "m980axr");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        source, "p980axr", &caps, "p980axr", "m980axr", &mut query,
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let view = SemanticView::from_checked(&product.compiled.checked)
        .map_err(|e| format!("semantic view: {e:?}"))?;
    let provider = ace_a2x_rust::RustTargetProvider::v1();
    let family = Family::RecordText;
    let runtime = provider
        .runtime_source(&view, family)
        .map_err(|e| format!("axr0 bridge: {e:?}"))?;
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let fragments = provider
        .produce_fragments(&view, &fns, family)
        .map_err(|e| format!("fragments: {e:?}"))?;
    let mut generated = String::from("// AUTO-GENERATED (PLAN-980 AXR0) — DO NOT EDIT.\n");
    generated.push_str("use super::a2x_runtime::*;\n\n");
    for f in &fragments {
        generated.push_str(f);
        generated.push_str("\n\n");
    }
    let wrapper = "// AUTO-GENERATED (PLAN-980 AXR0) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nfn main() {\n    let outcome = generated::fn_main();\n    match outcome {\n        Ok(v) => println!(\"value:{}\", v),\n        Err(t) => println!(\"trap:{}\", t.name()),\n    }\n}\n";
    Ok(F0A2rFileset {
        files: vec![
            ("main.rs".to_string(), wrapper.to_string()),
            ("a2x_runtime.rs".to_string(), runtime),
            ("generated.rs".to_string(), generated),
        ],
        provider_cdylib: build.cdylib.clone(),
        provider_import_lib: build
            .cdylib
            .with_file_name("f0_text_provider.dll.lib"),
    })
}

/// Auto 源字符串字面量转义（与 ace_hello 同规则，共享实现）。
pub(crate) fn escape_auto_string(s: &str) -> String {
    crate::ui::ace_hello::escape_auto_string_for_source(s)
}
