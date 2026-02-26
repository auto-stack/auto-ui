// High-level Transpiler API for AutoUI
//
// Provides simple API for transpiling .at files to Rust code

use auto_lang::Parser;
use std::path::Path;

use crate::trans::dsl_preprocess::dsl_preprocess;

/// Transpile Auto language file to Rust code
///
/// # Arguments
/// * `input_path` - Path to .at file
/// * `output_path` - Optional path to write .rs file (if None, returns code as string)
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

    transpile_dsl(&source, output_path)
}

/// Transpile DSL-style Auto code to Rust code
///
/// # Arguments
/// * `source` - DSL source code (widget/msg/model/view/on/style syntax)
/// * `output_path` - Optional path to write .rs file
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_dsl(source: &str, output_path: Option<&str>) -> Result<String, String> {
    let preprocessed = dsl_preprocess(source);

    transpile_auto(&preprocessed, output_path)
}

/// Transpile standard Auto code to Rust code
///
/// # Arguments
/// * `source` - Standard Auto source code
/// * `output_path` - Optional path to write .rs file
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_auto(source: &str, output_path: Option<&str>) -> Result<String, String> {
    let mut parser = Parser::from(source);
    let ast = parser
        .parse()
        .map_err(|e| format!("Failed to parse: {:?}", e))?;

    let rust_code = transpile_ast(&ast)?;

    if let Some(output) = output_path {
        std::fs::write(output, &rust_code)
            .map_err(|e| format!("Failed to write file {}: {}", output, e))?;
    }

    Ok(rust_code)
}

/// Transpile parsed AST to Rust code
///
/// # Arguments
/// * `ast` - Parsed Auto language AST
///
/// # Returns
/// Generated Rust code as string
pub fn transpile_ast(ast: &auto_lang::ast::Code) -> Result<String, String> {
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

/// Check if type declaration is a widget
fn is_widget_type(type_decl: &auto_lang::ast::TypeDecl) -> bool {
    let has_widget_spec = type_decl.specs.iter().any(|spec| spec.as_str() == "Widget");
    let has_view_method = type_decl.methods.iter().any(|m| m.name == "view");
    has_widget_spec || has_view_method
}

#[cfg(feature = "vue-backend")]
pub mod vue {
    use super::*;
    use crate::trans::vue_gen::VueCodeGenerator;

    pub fn transpile_vue_file(
        input_path: impl AsRef<Path>,
        output_path: Option<&str>,
    ) -> Result<String, String> {
        let input_path = input_path.as_ref();
        let source = std::fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;
        transpile_vue_dsl(&source, output_path)
    }

    pub fn transpile_vue_dsl(source: &str, output_path: Option<&str>) -> Result<String, String> {
        let preprocessed = crate::trans::dsl_preprocess::dsl_preprocess(source);
        transpile_vue_auto(&preprocessed, output_path)
    }

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
