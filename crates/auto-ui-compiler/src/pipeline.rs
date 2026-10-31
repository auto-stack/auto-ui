//! UI 解析/生成管线装配（PLAN-900 T-03 预案 a：库内装配迁移）。
//!
//! 原 auto-lang facade lib.rs 的 `ui_build` 族（Plan 096 UI Backend
//! Generators 段，ee6feb7ea 基线 606-1159 行）整体随依赖方向反转迁入本
//! crate：dialect 装配（`Parser::parse_domain(&UiDialect)`）、aura 提取、
//! design tokens/recipe、ui_gen 后端（vue/rust/jet）组合点全部在本 crate
//! 单源。函数签名与行为逐位保持；外部消费者（auto-man 等）此前已直连
//! `auto_ui_compiler::ui_gen::generate_component_from_file`，本模块为
//! 组合层完整落位。

use auto_lang_core::error::AutoResult;
use auto_lang_core::parser::Parser;


/// Build UI components from Auto files using AURA pipeline
///
/// This is the main entry point for UI scenario compilation.
///
/// # Arguments
/// * `path` - Input file or directory
/// * `scenario` - Compilation scenario (core, ui, shell)
/// * `backend` - Backend target (vue, rust)
/// * `output` - Optional output directory
pub fn ui_build(
    path: &str,
    scenario: &str,
    backend: &str,
    output: Option<&str>,
) -> AutoResult<String> {
    use auto_lang_core::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, JetGenerator, RustGenerator, VueGenerator};

    // Parse scenario
    let session = match scenario {
        "ui" => CompilerSession::ui().with_backend(backend),
        "core" => CompilerSession::default(),
        "shell" => CompilerSession::shell(),
        _ => CompilerSession::ui().with_backend(backend),
    };

    // Read input file
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // PLAN-635: pre-register use-imported recipes before the parse (parser
    // symbol-checker hook), then replay after the parse.
    let base_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let recipe_imports =
        crate::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, &code)
            .map_err(|e| e.to_string())?;

    // Parse with scenario
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session.clone());
    // PLAN-841: UI sources enter via the explicit domain port.
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .and_then(|ui| crate::ui_ast::lower_ui(&ui).map(|lowered| lowered.ui))
        .map_err(|e| format!("Parse/validation error: {:?}", e))?;

    // Extract AURA widgets from AST
    crate::aura::extract::clear_view_fragments();
    for frag in ast.domain_stmts().filter_map(|s| match s {
        auto_ui_dialect::UiStmt::ViewFragmentDecl(f) => Some(f),
        _ => None,
    }) {
        crate::aura::extract::register_view_fragment(frag);
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    crate::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast,
    )
    .map_err(|e| e.to_string())?;
    let mut widgets = Vec::new();
    for widget_decl in ast.domain_stmts().filter_map(|s| match s {
        auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
        _ => None,
    }) {
        {
            // Convert WidgetDecl to AuraWidget
            let aura_widget =
                crate::aura::extract_widget_from_decl(widget_decl).map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget or store declarations found in input file".into());
    }

    // Generate code based on backend
    // Rust backend uses auto-ui abstract components (Iced handled by auto-ui crate)
    // Jet backend generates Kotlin/Compose code for Android
    let mut output_code = String::new();
    match backend {
        "vue" => {
            let mut gen = VueGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        "jet" => {
            let mut gen = JetGenerator::new();
            for widget in &widgets {
                let code = gen.generate(widget).map_err(|e| e.to_string())?;
                output_code.push_str(&code);
                output_code.push_str("\n\n");
            }
        }
        _ => {
            return Err(format!("Unknown backend: {}. Available: vue, rust, jet", backend).into());
        }
    }

    // Write output if specified
    if let Some(out_dir) = output {
        let ext = match backend {
            "vue" => "vue",
            "rust" => "rs",
            "jet" => "kt",
            _ => "txt",
        };
        std::fs::create_dir_all(out_dir).ok();
        for widget in &widgets {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.{}", widget.name, ext));
            // Generate individual widget code
            let widget_code = match backend {
                "vue" => {
                    let mut gen = VueGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                "rust" => {
                    let mut gen = RustGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                "jet" => {
                    let mut gen = JetGenerator::new();
                    gen.generate(widget).map_err(|e| e.to_string())?
                }
                _ => output_code.clone(),
            };
            std::fs::write(&out_path, &widget_code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
    }

    Ok(output_code)
}

/// Build UI components with shadcn-vue mode enabled
///
/// This is a convenience function for generating Vue components
/// with shadcn-vue support enabled.
pub fn ui_build_shadcn(path: &str, output: Option<&str>) -> AutoResult<String> {
    use auto_lang_core::session::CompilerSession;
    use crate::ui_gen::{BackendGenerator, VueGenerator, VueMode};

    // Read input file
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // PLAN-635: pre-register use-imported recipes before the parse (parser
    // symbol-checker hook), then replay after the parse.
    let base_dir = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let recipe_imports =
        crate::design_tokens::recipe::prepare_style_recipe_imports(&base_dir, &code)
            .map_err(|e| e.to_string())?;

    // Parse with UI scenario
    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);
    // PLAN-841: UI sources enter via the explicit domain port.
    let ast = parser
        .parse_domain(&auto_ui_dialect::UiDialect)
        .map(auto_ui_dialect::UiCode)
        .and_then(|ui| crate::ui_ast::lower_ui(&ui).map(|lowered| lowered.ui))
        .map_err(|e| format!("Parse/validation error: {:?}", e))?;

    // Extract AURA widgets from AST
    crate::aura::extract::clear_view_fragments();
    for frag in ast.domain_stmts().filter_map(|s| match s {
        auto_ui_dialect::UiStmt::ViewFragmentDecl(f) => Some(f),
        _ => None,
    }) {
        crate::aura::extract::register_view_fragment(frag);
    }
    // PLAN-607: register and validate style recipes before extracting widget.
    // PLAN-635: replay imported (source-tagged) + local recipes in order.
    crate::design_tokens::recipe::load_and_validate_style_recipes_with_imports(
        &recipe_imports,
        &ast,
    )
    .map_err(|e| e.to_string())?;
    let mut widgets = Vec::new();
    for widget_decl in ast.domain_stmts().filter_map(|s| match s {
        auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
        _ => None,
    }) {
        {
            let aura_widget =
                crate::aura::extract_widget_from_decl(widget_decl).map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widget or store declarations found in input file".into());
    }

    // Generate code with shadcn-vue mode
    let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
    let mut output_code = String::new();

    for widget in &widgets {
        let code = gen.generate(widget).map_err(|e| e.to_string())?;
        output_code.push_str(&code);
        output_code.push_str("\n\n");
    }

    // Write output if specified
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for widget in &widgets {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", widget.name));
            let mut gen = VueGenerator::new().with_mode(VueMode::Shadcn);
            let widget_code = gen.generate(widget).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &widget_code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
    }

    Ok(output_code)
}

/// Build UI components with shadcn-vue mode enabled and return parsed widgets
///
/// This function is similar to `ui_build_shadcn` but also returns the parsed
/// AuraWidget structs, allowing callers to inspect widget metadata like routes.
///
/// # Arguments
/// * `path` - Input file path
/// * `output` - Optional output directory for generated files
///
/// # Returns
/// A tuple of (generated_code, widgets) on success
///
/// # Example
/// ```no_run
/// use auto_lang::ui_build_shadcn_with_widgets;
///
/// let (code, widgets) = ui_build_shadcn_with_widgets("app.at", None).unwrap();
///
/// // Check if any widget has routes
/// let has_routes = widgets.iter().any(|w| w.routes.is_some());
/// ```
pub fn ui_build_shadcn_with_widgets(
    path: &str,
    output: Option<&str>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>)> {
    let (vue_code, widgets, _stores) =
        ui_build_shadcn_with_widgets_and_stores(path, output, None, None, None)?;
    Ok((vue_code, widgets))
}

/// Plan 043 store-codegen: same as `ui_build_shadcn_with_widgets`, but also
/// returns the generated store composable files `(filename, code)`. Use this
/// when compiling multiple .at files in a loop and you need to collect stores
/// explicitly (the `STORE_EXTRA_FILES` thread-local is cleared per call).
///
/// `root_dir` (Plan 043 stream phase): when provided, streaming endpoints
/// (`~Stream<T>`) are resolved from `back/api.at` so the store composable can
/// wire type-driven SSE. Pass the project root (where `src/back/api.at` lives).
///
/// `shadcn` (Plan 013): `None`/`Some(true)` = shadcn-vue mapping (default);
/// `Some(false)` = native HTML elements, no `@/components/ui/*` imports.
///
/// `default_classes` (Plan 014): `None`/`Some(true)` = inject doc-theme
/// default Tailwind classes (default); `Some(false)` = skip them except for
/// structural layout primitives (row/col/grid/...).
pub fn ui_build_shadcn_with_widgets_and_stores(
    path: &str,
    output: Option<&str>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>, Vec<(String, String)>)> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    // Plan 012 Batch A: surface codegen validation warnings on the build path
    // (deduplicated — auto-man pre-scans each file before the real pass).
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    // Write output if specified (legacy behavior)
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        // Also write store composables
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code).ok();
        }
    }

    Ok((result.vue_code, result.widgets, store_composables))
}

/// Plan 408 P11 / KNOWN-DEBT pages 写盘：薄包装，返回完整 `GeneratedComponent`
/// （含 `all_widget_codes`——每个 component fn / widget 一个 SFC），让调用方
/// （auto-man `from_workspace` 的 pages 路径）能把 component fn SFC 也写盘到
/// components/，而非只取 first widget 的 vue_code。与
/// `ui_build_shadcn_with_widgets_and_stores` 同语义，只是不丢弃 all_widget_codes。
pub fn ui_build_shadcn_all_widget_codes(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<crate::ui_gen::GeneratedComponent> {
    ui_build_shadcn_all_widget_codes_with_bound(path, root_dir, shadcn, default_classes, None)
}

/// Plan 443: `ui_build_shadcn_all_widget_codes` + the workspace-aggregated
/// bound-channel map (which model channels some parent actually binds).
pub fn ui_build_shadcn_all_widget_codes_with_bound(
    path: &str,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<std::collections::HashMap<String, Vec<String>>>,
) -> AutoResult<crate::ui_gen::GeneratedComponent> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};
    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        stream_endpoints,
        shadcn,
        default_classes,
        bound_model_channels,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);
    Ok(result)
}

/// Like `ui_build_shadcn_with_widgets`, but accepts a list of known sub-widget names
/// to avoid naming collisions with shadcn-vue components (e.g. custom "Sidebar" widget
/// should not be mapped to shadcn's Sidebar).
pub fn ui_build_shadcn_with_sub_widgets(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>)> {
    let (vue_code, widgets, _stores) = ui_build_shadcn_with_sub_widgets_and_stores(
        path,
        output,
        sub_widget_names,
        None,
        None,
        None,
    )?;
    Ok((vue_code, widgets))
}

/// Plan 043 store-codegen: same as `ui_build_shadcn_with_sub_widgets`, but also
/// returns the generated store composable files `(filename, code)`. Callers that
/// need to write store files explicitly (e.g. `VueProject::from_workspace`) use
/// this variant instead of relying on the `STORE_EXTRA_FILES` thread-local,
/// which is cleared at the start of every `generate_component_from_file` call
/// and thus loses stores when multiple .at files are compiled in sequence.
///
/// `root_dir` (Plan 043 stream phase): when provided, streaming endpoints
/// (`~Stream<T>`) are resolved from `back/api.at` for type-driven SSE wiring.
///
/// `shadcn` (Plan 013): `None`/`Some(true)` = shadcn-vue mapping (default);
/// `Some(false)` = native HTML elements, no `@/components/ui/*` imports.
///
/// `default_classes` (Plan 014): `None`/`Some(true)` = inject doc-theme
/// default Tailwind classes (default); `Some(false)` = skip them except for
/// structural layout primitives (row/col/grid/...).
/// PLAN-037 T5: `ui_build_shadcn_with_sub_widgets_and_stores` + the
/// cross-file sub-widget model-var map (call-site model addressing). The
/// legacy entry delegates here with `None`.
pub fn ui_build_shadcn_with_sub_widgets_and_stores_full(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
    sub_widget_models: Option<std::collections::HashMap<String, Vec<String>>>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
    bound_model_channels: Option<std::collections::HashMap<String, Vec<String>>>,
    sub_widget_msgs: Option<std::collections::HashMap<String, Vec<String>>>,
) -> AutoResult<(
    String,
    Vec<crate::aura::AuraWidget>,
    Vec<(String, String)>,
    Vec<(String, String, String)>,
)> {
    // PLAN-024：第四返回段 = named_view_codes（widget, view, SFC code）——
    // 桌面 vue 宿主 Mini.vue 产物源（仅本 fn 的唯一调用方 auto-man 消费）。
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        sub_widget_models,
        bound_model_channels,
        stream_endpoints,
        shadcn,
        default_classes,
        sub_widget_msgs,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write store file: {}", e))?;
        }
    }

    Ok((
        result.vue_code,
        result.widgets,
        store_composables,
        result.named_view_codes,
    ))
}

pub fn ui_build_shadcn_with_sub_widgets_and_stores(
    path: &str,
    output: Option<&str>,
    sub_widget_names: Vec<String>,
    root_dir: Option<&str>,
    shadcn: Option<bool>,
    default_classes: Option<bool>,
) -> AutoResult<(String, Vec<crate::aura::AuraWidget>, Vec<(String, String)>)> {
    use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let at_path = std::path::Path::new(path);
    let stream_endpoints = root_dir.map(crate::ui_gen::api::resolve_stream_endpoints_for_project);
    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        stream_endpoints,
        shadcn,
        default_classes,
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts).map_err(|e| format!("{}", e))?;
    let store_composables = result.store_composables.clone();

    // Plan 012 Batch A: surface codegen validation warnings (deduplicated).
    crate::ui_gen::validators::print_warnings_once(path, &result.validation_warnings);

    // Write output if specified (legacy behavior)
    if let Some(out_dir) = output {
        std::fs::create_dir_all(out_dir).ok();
        for (name, code) in &result.all_widget_codes {
            let out_path = std::path::Path::new(out_dir).join(format!("{}.vue", name));
            std::fs::write(&out_path, code)
                .map_err(|e| format!("Failed to write output file: {}", e))?;
        }
        for (filename, code) in &result.store_composables {
            let out_path = std::path::Path::new(out_dir).join(filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&out_path, code).ok();
        }
    }

    Ok((result.vue_code, result.widgets, store_composables))
}
