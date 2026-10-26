//! auto-ui-compiler — UI 编译/生成域（PLAN-814）。
//!
//! 从 auto-lang 迁入的 UI 分析（aura 提取/schema）、设计 token 与
//! vue/jet/ark/rust 四生成器及 widget registry（T-03 迁移面）。
//!
//! 边界合同：依赖纯语言 SDK（auto-lang-core），不被 SDK 反向依赖；
//! 生成的 runtime import 路径策略单源在本 crate；跨层集成测试归上层。

// ---- PLAN-814 T-03 Batch 1：style DSL 分析/模型、action 配置解析、lucide 表 ----
// （iced 后端适配器留 auto-lang ui/style_iced_adapter.rs——runtime 件。）
pub mod action_config;
// PLAN-815 T-04: 画廊 app 元信息扫描 + i18n 查询（自 auto-lang ui 迁入；
// 旧包经 ui/mod.rs 再导出保持 auto_lang::ui::{app_registry,i18n_lookup} 路径）。
pub mod app_registry;
pub mod design_tokens;
pub mod i18n_lookup;
pub mod lucide_generated;
pub mod store_deps;
pub mod style;
// ---- PLAN-814 T-03 Batch 2：AURA 模型/校验、a2ui、四生成器、route 分析 ----
pub mod a2ui;
pub mod aura;
pub mod route;
pub mod ui_ast;
pub mod ui_gen;

pub use ui_ast::{
    find_store_decl_files, lower_ui, parse_ui, resolve_use_module, LoweredUi, UseModuleResolution,
    UI_AST_SCHEMA_VERSION,
};

/// 生成器源指纹（本 crate build.rs 对 src/ui_gen 扫描；Plan 692 W-1
/// 失效语义单源）。None=无 build.rs 环境；UICache 使用下方组合身份。
pub fn generator_fingerprint() -> Option<String> {
    option_env!("AUTO_UI_GEN_FINGERPRINT").map(|s| s.to_string())
}

/// UICache identity covers both generator code and the persisted UI AST schema.
pub fn ui_cache_fingerprint() -> Option<String> {
    let generator = generator_fingerprint().unwrap_or_else(|| "unknown".to_string());
    Some(format!("{generator}:ui-ast-schema-{UI_AST_SCHEMA_VERSION}"))
}
