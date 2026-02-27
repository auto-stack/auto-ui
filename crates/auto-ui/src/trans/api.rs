// High-level Transpiler API for AutoUI
//
// Provides simple API for transpiling .at files to Rust code
//
// Plan 096 Phase 5: Now uses AURA pipeline instead of DSL preprocessing

use auto_lang::Parser;
use auto_lang::session::CompilerSession;
use std::path::Path;

// Legacy: Keep for backward compatibility
#[allow(deprecated)]
use crate::trans::dsl_preprocess::dsl_preprocess;

// ============================================================================
// New AURA-based API (Plan 096)
// ============================================================================

/// Transpile Auto UI file to Rust code using AURA pipeline
///
/// # Arguments
/// * `input_path` - Path to .at file
/// * `output_path` - Optional path to write .rs file
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_file(
    input_path: impl AsRef<Path>,
    output_path: Option<&str>,
) -> Result<String, String> {
    let input_path = input_path.as_ref();
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;

    transpile_aura(&source, output_path)
}

/// Transpile UI source code to Rust using AURA pipeline (Plan 096)
///
/// This is the preferred method for transpiling UI components.
/// It uses the new AURA-based architecture without DSL preprocessing.
pub fn transpile_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    use auto_lang::ui_gen::RustGenerator;
    use auto_lang::ui_gen::BackendGenerator;
    use auto_lang::aura::extract_widget_from_decl;

    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = RustGenerator::new();

    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

/// Transpile UI source code to Vue3 SFC using AURA pipeline (Plan 096)
pub fn transpile_vue_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
    use auto_lang::ui_gen::VueGenerator;
    use auto_lang::ui_gen::BackendGenerator;
    use auto_lang::aura::extract_widget_from_decl;

    // Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(source).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Failed to parse: {:?}", e))?;

    // Extract and generate
    let mut code = String::new();
    let mut generator = VueGenerator::new();

    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| format!("Failed to extract widget: {}", e))?;
            let widget_code = generator.generate(&aura_widget)
                .map_err(|e| format!("Failed to generate: {}", e))?;
            code.push_str(&widget_code);
            code.push('\n');
        }
    }

    if let Some(output) = output_path {
        std::fs::write(output, &code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(code)
}

// ============================================================================
// Legacy API (Deprecated - uses DSL preprocessing)
// ============================================================================

/// Transpile DSL-style Auto code to Rust code
///
/// # Deprecated
/// Use `transpile_aura` instead for the new AURA-based pipeline.
#[deprecated(since = "0.2.0", note = "Use transpile_aura instead")]
pub fn transpile_dsl(source: &str, output_path: Option<&str>) -> Result<String, String> {
    let preprocessed = dsl_preprocess(source);
    transpile_auto_legacy(&preprocessed, output_path)
}

/// Transpile standard Auto code to Rust code (legacy)
#[deprecated(since = "0.2.0", note = "Use transpile_aura instead")]
pub fn transpile_auto_legacy(source: &str, output_path: Option<&str>) -> Result<String, String> {
    let mut parser = Parser::from(source);
    let ast = parser
        .parse()
        .map_err(|e| format!("Failed to parse: {:?}", e))?;

    let rust_code = transpile_ast_legacy(&ast)?;

    if let Some(output) = output_path {
        std::fs::write(output, &rust_code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(rust_code)
}

/// Transpile parsed AST to Rust code (legacy)
#[deprecated(since = "0.2.0", note = "Use transpile_aura instead")]
pub fn transpile_ast_legacy(ast: &auto_lang::ast::Code) -> Result<String, String> {
    #[allow(deprecated)]
    use crate::trans::rust_gen::RustCodeGenerator;

    let mut generator = RustCodeGenerator::new();
    let mut code = String::new();

    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
            if is_widget_type(type_decl) {
                let widget_code = generator.generate_widget(type_decl)?;
                code.push_str(&widget_code);
                code.push('\n');
            }
        }
    }

    Ok(code)
}

/// Check if type declaration is a widget (legacy)
fn is_widget_type(type_decl: &auto_lang::ast::TypeDecl) -> bool {
    let has_widget_spec = type_decl.specs.iter().any(|spec| spec.as_str() == "Widget");
    let has_view_method = type_decl.methods.iter().any(|m| m.name == "view");
    has_widget_spec || has_view_method
}

// ============================================================================
// Vue Backend (Legacy)
// ============================================================================

#[cfg(feature = "vue-backend")]
pub mod vue {
    use super::*;
    #[allow(deprecated)]
    use crate::trans::vue_gen::VueCodeGenerator;

    #[deprecated(since = "0.2.0", note = "Use transpile_vue_aura instead")]
    pub fn transpile_vue_file(
        input_path: impl AsRef<Path>,
        output_path: Option<&str>,
    ) -> Result<String, String> {
        let input_path = input_path.as_ref();
        let source = std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;
        transpile_vue_dsl(&source, output_path)
    }

    #[deprecated(since = "0.2.0", note = "Use transpile_vue_aura instead")]
    pub fn transpile_vue_dsl(source: &str, output_path: Option<&str>) -> Result<String, String> {
        let preprocessed = crate::trans::dsl_preprocess::dsl_preprocess(source);
        transpile_vue_auto(&preprocessed, output_path)
    }

    #[deprecated(since = "0.2.0", note = "Use transpile_vue_aura instead")]
    pub fn transpile_vue_auto(source: &str, output_path: Option<&str>) -> Result<String, String> {
        let mut parser = Parser::from(source);
        let ast = parser
            .parse()
            .map_err(|e| format!("Failed to parse: {:?}", e))?;
        let vue_code = transpile_vue_ast(&ast)?;
        if let Some(output) = output_path {
            std::fs::write(output, &vue_code)
                .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
        }
        Ok(vue_code)
    }

    #[deprecated(since = "0.2.0", note = "Use transpile_vue_aura instead")]
    pub fn transpile_vue_ast(ast: &auto_lang::ast::Code) -> Result<String, String> {
        let mut generator = VueCodeGenerator::new();
        let mut code = String::new();

        for stmt in &ast.stmts {
            if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
                if super::is_widget_type(type_decl) {
                    let component_code = generator.generate_widget(type_decl)?;
                    code.push_str(&component_code);
                    code.push('\n');
                }
            }
        }

        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_hello() {
        assert!(true);
    }
}
