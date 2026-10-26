use auto_lang_core::ast::Code;
use auto_lang_core::error::{AutoError, AutoResult};
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_parser_api::DomainStmt;
use auto_ui_dialect::{UiCode, UiDialect, UiStmt};

pub use auto_ui_dialect::UI_AST_SCHEMA_VERSION;

#[derive(Debug, Clone)]
pub struct LoweredUi {
    pub ui: UiCode,
    pub core: Code,
}

pub fn parse_ui(source: &str) -> AutoResult<UiCode> {
    let mut parser = Parser::from(source).with_session(CompilerSession::ui());
    parser.parse_domain(&UiDialect).map(UiCode)
}

pub fn parse_validated_ui(source: &str) -> AutoResult<UiCode> {
    let ui = parse_ui(source)?;
    lower_ui(&ui).map(|lowered| lowered.ui)
}

pub fn lower_ui(ui: &UiCode) -> AutoResult<LoweredUi> {
    auto_ui_dialect::validate_ui(ui).map_err(AutoError::Msg)?;
    let mut stmts = Vec::new();
    let mut source_lines = Vec::new();
    for (index, item) in ui.0.items.iter().enumerate() {
        if let DomainStmt::Core(stmt) = item {
            stmts.push(stmt.clone());
            source_lines.push(ui.0.source_lines.get(index).copied().unwrap_or(0));
        }
    }
    Ok(LoweredUi {
        ui: ui.clone(),
        core: Code {
            stmts,
            source_lines,
            file_attrs: ui.0.file_attrs.clone(),
        },
    })
}

pub enum UseModuleResolution {
    Module(std::path::PathBuf),
    StoreFiles(std::collections::HashMap<String, std::path::PathBuf>),
    None,
}

pub fn resolve_use_module(
    base_dir: &std::path::Path,
    use_stmt: &auto_lang_core::use_scanner::UseStatement,
) -> UseModuleResolution {
    if let Some(path) = auto_lang_core::resolve_module_path(base_dir, &use_stmt.module) {
        return UseModuleResolution::Module(path);
    }
    if use_stmt.module == "store" && !use_stmt.items.is_empty() {
        return UseModuleResolution::StoreFiles(find_store_decl_files(base_dir, &use_stmt.items));
    }
    if !use_stmt.items.is_empty() {
        for item in &use_stmt.items {
            let mut candidates = vec![auto_lang_core::store_name_snake(item)];
            if *candidates.last().unwrap() != *item {
                candidates.push(item.clone());
            }
            for candidate in candidates {
                let dotted = format!("{}.{}", use_stmt.module, candidate);
                if let Some(path) = auto_lang_core::resolve_module_path(base_dir, &dotted) {
                    return UseModuleResolution::Module(path);
                }
            }
        }
    }
    UseModuleResolution::None
}

pub fn find_store_decl_files(
    base_dir: &std::path::Path,
    names: &[String],
) -> std::collections::HashMap<String, std::path::PathBuf> {
    use std::collections::HashMap;

    let mut found = HashMap::new();
    let mut missing = Vec::new();
    for name in names {
        let snake = auto_lang_core::store_name_snake(name);
        let mut hit = None;
        for candidate in [
            format!("{}.at", snake),
            format!("{}_store.at", snake.trim_end_matches("_store")),
        ] {
            let path = base_dir.join(candidate);
            if path.exists() {
                hit = Some(path);
                break;
            }
        }
        if let Some(path) = hit {
            found.insert(name.clone(), path);
        } else {
            missing.push(name);
        }
    }
    if missing.is_empty() {
        return found;
    }

    let mut budget = 400usize;
    let mut scan = |root: &std::path::Path,
                    found: &mut HashMap<String, std::path::PathBuf>,
                    missing: &[&String]| {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if path.is_dir() {
                    if matches!(
                        file_name.as_str(),
                        "gen" | "node_modules" | "target" | "dist" | ".git" | "build"
                    ) {
                        continue;
                    }
                    stack.push(path);
                } else if file_name.ends_with(".at") && budget > 0 {
                    budget -= 1;
                    let Ok(source) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    let source = auto_lang_core::back_prefix::apply(&path, source);
                    let Ok(ast) = parse_validated_ui(&source) else {
                        continue;
                    };
                    for stmt in ast.domain_stmts() {
                        if let UiStmt::StoreDecl(store) = stmt {
                            let name = store.name.as_str().to_string();
                            if missing.iter().any(|item| *item == &name)
                                && !found.contains_key(&name)
                            {
                                found.insert(name, path.clone());
                            }
                        }
                    }
                }
            }
        }
    };
    scan(base_dir, &mut found, &missing);
    let remaining: Vec<&String> = missing
        .iter()
        .filter(|name| !found.contains_key(name.as_str()))
        .copied()
        .collect();
    if !remaining.is_empty() {
        if let Some(parent) = base_dir.parent() {
            scan(parent, &mut found, &remaining);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::extract_widget_from_decl;
    use crate::ui_gen::{BackendGenerator, JetGenerator};
    use auto_lang_core::ast::Stmt;

    const REPRESENTATIVE: &str = r#"
style card = "bg-card"
actions {
    action (id: "app.go", handler: .Go, title: "Go")
}
store Shared {
    msg { Ping }
    model { count int = 0 }
    on { .Ping -> { .count += 1 } }
}
widget Counter {
    msg { Go }
    model { count int = 0 }
    view { col { button (onclick: .Go) {} } }
    on { .Go -> { .count += 1 } }
}
fn main() {
    let count = 1
    let borrowed = count.view
    print(borrowed)
}
"#;

    #[test]
    fn representative_domain_lowering_vm_and_four_generators() {
        let ui = parse_ui(REPRESENTATIVE).expect("UI and Core parse");
        assert_eq!(ui.domain_stmts().count(), 4);
        assert!(ui.core_stmts().any(|stmt| matches!(stmt, Stmt::Fn(_))));
        let lowered = lower_ui(&ui).expect("validated lowering");
        assert!(lowered
            .core
            .stmts
            .iter()
            .all(|stmt| matches!(stmt, Stmt::Fn(_))));

        let mut codegen = auto_lang_core::vm::codegen::Codegen::new();
        for stmt in &lowered.core.stmts {
            codegen.compile_stmt(stmt).expect("Core codegen");
        }
        let package = codegen.into_compiled_package();
        let entry = package.exports.get("main").copied().unwrap_or(0) as usize;
        let flash = auto_lang_core::vm::virt_memory::VirtualFlash::from_vec_with_metadata(
            package.bytecode,
            package.exports,
            package.object_keys,
            package.object_types,
        );
        let (mut vm, output) = auto_lang_core::vm::engine::AutoVM::new_with_capture(flash, 8192);
        vm.load_strings(package.string_pool);
        let mut natives = auto_lang_core::vm::native::NativeInterface::new();
        natives.register_std_shims();
        auto_lang_core::vm::ffi::stdlib::register_stdlib_ffi(&mut natives);
        vm.merge_native_interface(&natives);
        vm.spawn_task(entry, 16384);
        auto_lang_core::block_on_autovm_local(vm.run_task_loop());
        assert_eq!(output.read().unwrap().trim(), "1");

        let rust =
            crate::ui_gen::api::transpile_aura(REPRESENTATIVE, None).expect("Rust generator");
        assert!(rust.contains("Counter"));
        let vue =
            crate::ui_gen::api::transpile_vue_aura(REPRESENTATIVE, None).expect("Vue generator");
        assert!(vue.contains("Counter"));

        let widget = ui
            .domain_stmts()
            .find_map(|stmt| match stmt {
                UiStmt::WidgetDecl(widget) => Some(widget),
                _ => None,
            })
            .expect("widget");
        let aura = extract_widget_from_decl(widget).expect("AURA extraction");
        let jet = JetGenerator::new().generate(&aura).expect("Jet generator");
        assert!(jet.contains("Counter"));
        let ark = crate::ui_gen::ark::ArkGenerator::new()
            .generate(&aura)
            .expect("Ark generator");
        assert!(ark.contains("Counter"));
    }

    #[test]
    fn representative_cross_file_import_uses_owned_domain_modules() {
        let temp = tempfile::tempdir().expect("tempdir");
        let answers = temp.path().join("answers.at");
        let module_source = concat!(
            "fn answer() int { 42 }\n",
            "widget Imported { view { text \"module\" {} } }\n"
        );
        std::fs::write(&answers, module_source).expect("shared source");
        let app = temp.path().join("app.at");
        let source = "use answers: answer\nwidget App { view { text \"x\" {} } }\n";
        std::fs::write(&app, source).expect("app source");

        let (functions, imported) = crate::ui_gen::api::collect_use_module_fns(&app, source);
        assert_eq!(imported, vec!["answer"]);
        assert!(functions.iter().any(|function| function.name == "answer"));
        let ui = parse_ui(source).expect("host parse");
        assert!(ui.core_stmts().any(|stmt| matches!(stmt, Stmt::Use(_))));
        let host_widgets = ui
            .domain_stmts()
            .filter_map(|stmt| match stmt {
                UiStmt::WidgetDecl(widget) => Some(widget.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(host_widgets, vec!["App"]);

        let module_ui = parse_ui(module_source).expect("module-owned parse");
        let module_widgets = module_ui
            .domain_stmts()
            .filter_map(|stmt| match stmt {
                UiStmt::WidgetDecl(widget) => Some(widget.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(module_widgets, vec!["Imported"]);
    }

    #[test]
    fn all_ui_statement_families_require_the_domain_port() {
        let sources = [
            "widget App { view { text \"x\" {} } }",
            "store Shared { model { count int = 0 } }",
            "msg Msg { Save }",
            "model { count int = 0 }",
            "component fn Card() { view { text \"x\" {} } }",
            "view fn Item() { text \"x\" {} }",
            "actions { action (id: \"app.go\", handler: .Go, title: \"Go\") }",
            "style card = \"bg-card\"",
            "pub style card = \"bg-card\"",
            "use.web.fn fetch from \"web\"",
        ];
        for source in sources {
            let ui = parse_ui(source).unwrap_or_else(|error| {
                panic!("domain port must accept UI statement `{source}`: {error}")
            });
            assert_eq!(
                ui.domain_stmts().count(),
                1,
                "UI statement must remain in the owned domain: {source}"
            );
            assert!(
                lower_ui(&ui)
                    .expect("validated lowering")
                    .core
                    .stmts
                    .is_empty(),
                "domain statements must not leak into Core code: {source}"
            );
        }
    }

    #[test]
    fn minimal_store_and_widget_domain_parse() {
        let store = parse_ui("style card = \"bg-card\"\nactions { action (id: \"app.go\", handler: .Go, title: \"Go\") }\nstore Shared { msg { Ping } model { count int = 0 } on { .Ping -> { .count += 1 } } }");
        assert!(store.is_ok(), "{:?}", store.err());
        let widget = parse_ui("widget App { view { text \"x\" {} } }");
        assert!(widget.is_ok(), "{:?}", widget.err());
    }
}
