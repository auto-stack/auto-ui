use auto_lang_core::database::UICache;
use std::path::{Path, PathBuf};

#[test]
fn ui_ast_schema_survives_reload_and_invalidates_old_cache_identities() {
    let temp = tempfile::tempdir().expect("cache tempdir");
    let current = auto_ui_compiler::ui_cache_fingerprint().expect("UICache identity");
    assert!(current.contains(&format!(
        "ui-ast-schema-{}",
        auto_ui_compiler::UI_AST_SCHEMA_VERSION
    )));
    auto_lang_core::database::set_generator_fingerprint(Some(current.clone()));

    let source = PathBuf::from("app.at");
    let mut cache = UICache::new();
    cache.update(source.clone(), 42, vec![]);
    cache.save(temp.path()).expect("save current cache");

    let reloaded = UICache::load(temp.path());
    assert_eq!(reloaded.file_count(), 1, "same schema survives reload");
    assert!(!reloaded.is_dirty(&source, 42));

    let cache_path = UICache::cache_path(temp.path());
    let persisted = std::fs::read_to_string(&cache_path).expect("persisted cache");
    let old_schema = format!(
        "ui-ast-schema-{}",
        auto_ui_compiler::UI_AST_SCHEMA_VERSION - 1
    );
    let old_persisted = persisted.replace(
        &format!("ui-ast-schema-{}", auto_ui_compiler::UI_AST_SCHEMA_VERSION),
        &old_schema,
    );
    assert_ne!(
        old_persisted, persisted,
        "negative control changes cache identity"
    );
    std::fs::write(&cache_path, old_persisted).expect("write old-schema cache");

    let invalidated = UICache::load(temp.path());
    assert_eq!(
        invalidated.file_count(),
        0,
        "old AST schema drops artifacts"
    );
    assert!(invalidated.is_dirty(Path::new("app.at"), 42));
}
