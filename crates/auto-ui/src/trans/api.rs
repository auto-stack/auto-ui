// High-level Transpiler API for AutoUI
//
// Provides simple API for transpiling .at files to Rust code

use auto_lang::Parser;
use std::path::Path;

/// Transpile Auto language file to Rust code
///
/// # Arguments
/// * `input_path` - Path to .at file
/// * `output_path` - Optional path to write .rs file (if None, returns code as string)
///
/// # Returns
/// Generated Rust code as string
///
/// # Example
/// ```ignore
/// let rust_code = transpile_file("scratch/hello.at", None)?;
/// println!("{}", rust_code);
/// ```
pub fn transpile_file(input_path: impl AsRef<Path>, output_path: Option<&str>) -> Result<String, String> {
    let input_path = input_path.as_ref();

    // Read the .at file
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read file {}: {}", input_path.display(), e))?;

    // Parse the Auto code
    // Parser::new requires code and universe scope
    use std::rc::Rc;
    use std::cell::RefCell;
    use auto_lang::Universe;

    let universe = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(&source, universe);
    let ast = parser.parse()
        .map_err(|e| format!("Failed to parse {}: {:?}", input_path.display(), e))?;

    // Generate Rust code
    let rust_code = transpile_ast(&ast)?;

    // Write to output file if specified
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

    // Process all type declarations (widgets)
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::TypeDecl(type_decl) = stmt {
            // Check if it's a widget (has Widget trait or has view method)
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
    // Check if has Widget in specs (traits/specs implemented)
    // Spec is just a type alias for AutoStr (String)
    let has_widget_spec = type_decl.specs.iter()
        .any(|spec| spec.as_str() == "Widget");

    // Or check if has view method
    let has_view_method = type_decl.methods.iter()
        .any(|m| m.name == "view");

    has_widget_spec || has_view_method
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_hello() {
        // This test requires actual .at file
        // For now, just verify the function compiles
        assert!(true);
    }
}
