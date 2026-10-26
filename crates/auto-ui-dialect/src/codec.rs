use crate::{UiCode, UiStmt, ViewNode};
use auto_parser_api::DomainStmt;
use serde::Deserialize;

pub const UI_AST_SCHEMA_VERSION: u32 = 1;
pub const UI_AST_DOMAIN: &str = "auto-ui";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UiEnvelope {
    domain: String,
    schema: u32,
    code: UiCode,
}

pub fn encode_ui_code(code: &UiCode) -> Result<Vec<u8>, String> {
    let envelope = UiEnvelope {
        domain: UI_AST_DOMAIN.to_string(),
        schema: UI_AST_SCHEMA_VERSION,
        code: code.clone(),
    };
    serde_json::to_vec(&envelope).map_err(|error| format!("UI AST encode failed: {error}"))
}

pub fn decode_ui_code(input: &[u8]) -> Result<UiCode, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let envelope = UiEnvelope::deserialize(serde_stacker::Deserializer::new(&mut deserializer))
        .map_err(|error| format!("UI AST schema/tag validation failed: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("UI AST schema/tag validation failed: {error}"))?;
    if envelope.domain != UI_AST_DOMAIN {
        return Err(format!(
            "UI AST domain mismatch: expected {UI_AST_DOMAIN}, got {}",
            envelope.domain
        ));
    }
    if envelope.schema != UI_AST_SCHEMA_VERSION {
        return Err(format!(
            "UI AST schema version mismatch: expected {UI_AST_SCHEMA_VERSION}, got {}; migrate or regenerate the artifact",
            envelope.schema
        ));
    }
    crate::validate_ui(&envelope.code)?;
    Ok(envelope.code)
}

pub trait UiAstVisitor {
    fn visit_ui_stmt(&mut self, _stmt: &UiStmt) {}
    fn visit_core_stmt(&mut self, _stmt: &auto_ast::ast::Stmt) {}
    fn visit_view_node(&mut self, _node: &ViewNode) {}
}

pub fn walk_ui_code<V: UiAstVisitor>(code: &UiCode, visitor: &mut V) {
    for item in &code.0.items {
        match item {
            DomainStmt::Core(stmt) => visitor.visit_core_stmt(stmt),
            DomainStmt::Domain(stmt) => walk_ui_stmt(stmt, visitor),
        }
    }
}

pub fn walk_ui_stmt<V: UiAstVisitor>(stmt: &UiStmt, visitor: &mut V) {
    visitor.visit_ui_stmt(stmt);
    match stmt {
        UiStmt::WidgetDecl(widget) => {
            if let Some(view) = &widget.view {
                walk_view_node(&view.root, visitor);
            }
            for (_, view) in &widget.named_views {
                walk_view_node(&view.root, visitor);
            }
        }
        UiStmt::ViewBlock(view) => walk_view_node(&view.root, visitor),
        UiStmt::ViewFragmentDecl(fragment) => walk_view_node(&fragment.body, visitor),
        _ => {}
    }
}

pub fn walk_view_node<V: UiAstVisitor>(node: &ViewNode, visitor: &mut V) {
    visitor.visit_view_node(node);
    match node {
        ViewNode::Element { children, .. } => {
            for child in children {
                walk_view_node(child, visitor);
            }
        }
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for child in then_body {
                walk_view_node(child, visitor);
            }
            if let Some(body) = else_body {
                for child in body {
                    walk_view_node(child, visitor);
                }
            }
        }
        ViewNode::ForLoop { body, .. } | ViewNode::MemoBlock { body, .. } => {
            for child in body {
                walk_view_node(child, visitor);
            }
        }
        ViewNode::Link { children, .. } => {
            for child in children {
                walk_view_node(child, visitor);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ModelBlock, ModelField, MsgDecl, ViewBlock, ViewText, WidgetDecl};
    use auto_ast::ast::{Expr, Fn, Stmt, Type};
    use auto_parser_api::DomainCode;

    fn sample() -> UiCode {
        let widget = WidgetDecl {
            name: "Counter".into(),
            messages: vec![MsgDecl { variants: vec![] }],
            model: Some(ModelBlock {
                fields: vec![ModelField {
                    name: "count".into(),
                    ty: Type::Int,
                    init: Expr::Int(0),
                    mutable: true,
                    is_primary: false,
                    decorators: vec![],
                }],
            }),
            computed: None,
            view: Some(ViewBlock {
                root: ViewNode::Text(ViewText::Literal("count".into())),
            }),
            named_views: vec![],
            on: None,
            bind: None,
            props: vec![],
            routes: None,
            lifecycle: vec![],
            style: None,
            ext_imports: vec![],
            watch: vec![],
            setup: None,
            expose: vec![],
            actions: None,
            timer: None,
        };
        UiCode(DomainCode {
            items: vec![
                DomainStmt::Domain(UiStmt::WidgetDecl(widget)),
                DomainStmt::Core(Stmt::Fn(Fn::default())),
            ],
            source_lines: vec![7, 11],
            file_attrs: vec!["tool".into()],
        })
    }

    #[test]
    fn codec_roundtrip_preserves_domain_core_source_and_clone() {
        let original = sample();
        let encoded = encode_ui_code(&original).expect("encode");
        let decoded = decode_ui_code(&encoded).expect("decode");
        let encoded_again = encode_ui_code(&decoded).expect("re-encode");
        assert_eq!(encoded, encoded_again);
        let cloned = decoded.clone();
        assert_eq!(cloned.0.source_lines, vec![7, 11]);
        assert_eq!(cloned.0.file_attrs[0].as_str(), "tool");
        assert_eq!(encode_ui_code(&cloned).unwrap(), encoded);
    }

    #[test]
    fn codec_roundtrip_preserves_real_parse_spans_and_source_lines() {
        let src = concat!(
            "widget Counter {\n",
            "    msg { Go }\n",
            "    model { count int = 0 }\n",
            "    view { col { button (onclick: .Go) {} } }\n",
            "}\n",
            "fn main() {\n",
            "    print(1)\n",
            "}\n"
        );
        let mut parser = auto_lang_core::parser::Parser::from(src)
            .with_session(auto_lang_core::session::CompilerSession::ui());
        let ui = UiCode(
            parser
                .parse_domain(&crate::UiDialect)
                .expect("real UI+Core source parses"),
        );

        // Baseline facts from the live parse: view span, source lines, Core fn.
        let original_span = ui
            .domain_stmts()
            .find_map(|stmt| match stmt {
                UiStmt::WidgetDecl(widget) => widget.view.as_ref().map(|view| &view.root),
                _ => None,
            })
            .and_then(|root| match root {
                ViewNode::Element { span, .. } => span.clone(),
                _ => None,
            })
            .expect("parsed element carries a byte span");
        assert!(!ui.0.source_lines.is_empty(), "parse records source lines");
        assert!(ui.core_stmts().any(|stmt| matches!(stmt, Stmt::Fn(_))));

        let encoded = encode_ui_code(&ui).expect("encode");
        let decoded = decode_ui_code(&encoded).expect("decode");
        assert_eq!(
            encode_ui_code(&decoded).unwrap(),
            encoded,
            "re-encode is byte-identical: fields, spans and lines survive"
        );
        assert_eq!(decoded.0.source_lines, ui.0.source_lines);
        let decoded_span = decoded
            .domain_stmts()
            .find_map(|stmt| match stmt {
                UiStmt::WidgetDecl(widget) => widget.view.as_ref().map(|view| &view.root),
                _ => None,
            })
            .and_then(|root| match root {
                ViewNode::Element { span, .. } => span.clone(),
                _ => None,
            })
            .expect("decoded element still carries its span");
        assert_eq!(original_span, decoded_span);
        assert!(decoded.core_stmts().any(|stmt| matches!(stmt, Stmt::Fn(_))));
    }

    #[test]
    fn codec_roundtrips_supported_nested_views() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                for depth in [4usize, 16, 32, 48] {
                    let source = format!(
                        "widget App {{ view {{ {}text \"leaf\" {{}}{} }} }}",
                        "col { ".repeat(depth),
                        " }".repeat(depth)
                    );
                    let mut parser = auto_lang_core::parser::Parser::from(&source)
                        .with_session(auto_lang_core::session::CompilerSession::ui());
                    let ui = UiCode(
                        parser
                            .parse_domain(&crate::UiDialect)
                            .expect("legal nested source parses"),
                    );
                    crate::validate_ui(&ui).expect("legal nested source validates");
                    let bytes = encode_ui_code(&ui).expect("encoder accepts legal source");
                    let decoded = decode_ui_code(&bytes)
                        .expect("codec decodes its own supported nested output");
                    assert_eq!(encode_ui_code(&decoded).unwrap(), bytes);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn codec_rejects_decoded_invalid_domain_data() {
        let mut invalid = sample();
        let widget = match &invalid.0.items[0] {
            DomainStmt::Domain(UiStmt::WidgetDecl(widget)) => widget.clone(),
            _ => unreachable!(),
        };
        invalid
            .0
            .items
            .push(DomainStmt::Domain(UiStmt::WidgetDecl(widget)));
        let bytes = encode_ui_code(&invalid).expect("encode invalid probe input");
        let error = decode_ui_code(&bytes).expect_err("decoded domain must be validated");
        assert!(error.contains("UI domain validation failed"), "{error}");
        assert!(error.contains("duplicate widget declaration"), "{error}");
    }

    #[test]
    fn codec_rejects_trailing_data_and_accepts_trailing_whitespace() {
        let bytes = encode_ui_code(&sample()).expect("encode");
        let mut whitespace = bytes.clone();
        whitespace.extend_from_slice(b" \n\t");
        decode_ui_code(&whitespace).expect("trailing whitespace is valid JSON");

        for suffix in [
            b"garbage".as_slice(),
            b"{}".as_slice(),
            b"{\"schema\":0}".as_slice(),
        ] {
            let mut input = bytes.clone();
            input.extend_from_slice(suffix);
            assert!(serde_json::from_slice::<serde_json::Value>(&input).is_err());
            let error = decode_ui_code(&input)
                .expect_err("decoder must consume the complete artifact, not a valid prefix");
            assert!(error.contains("schema/tag validation failed"), "{error}");
        }
    }

    #[test]
    fn codec_rejects_invalid_message_payload_shape() {
        let mut invalid = sample();
        let widget = match &mut invalid.0.items[0] {
            DomainStmt::Domain(UiStmt::WidgetDecl(widget)) => widget,
            _ => unreachable!(),
        };
        widget.messages = vec![MsgDecl {
            variants: vec![crate::MsgVariant {
                name: "Go".into(),
                quoted: false,
                payload: vec![Type::Int],
                payload_names: vec![],
            }],
        }];
        let bytes = encode_ui_code(&invalid).expect("encode invalid probe input");
        let error = decode_ui_code(&bytes).expect_err("invalid payload shape must be rejected");
        assert!(
            error.contains("mismatched payload names and types"),
            "{error}"
        );
    }

    #[test]
    fn codec_rejects_bad_domain_version_and_tag() {
        let encoded = String::from_utf8(encode_ui_code(&sample()).unwrap()).unwrap();
        let wrong_domain = encoded.replace("\"auto-ui\"", "\"other-ui\"");
        assert!(decode_ui_code(wrong_domain.as_bytes())
            .unwrap_err()
            .contains("domain mismatch"));
        let old_schema = encoded.replace("\"schema\":1", "\"schema\":0");
        assert!(decode_ui_code(old_schema.as_bytes())
            .unwrap_err()
            .contains("schema version mismatch"));
        let bad_tag = encoded.replace("WidgetDecl", "WidgetDeclZero");
        assert!(decode_ui_code(bad_tag.as_bytes())
            .unwrap_err()
            .contains("schema/tag validation failed"));
    }

    #[test]
    fn codec_clone_and_decode_have_bounded_last_owner_lifetimes() {
        let shared = auto_val::shared(auto_ast::ast::Tag::new("Owned".into(), vec![]));
        let original_weak = std::rc::Rc::downgrade(&shared);
        let original = UiCode::from_domain(vec![UiStmt::ModelBlock(ModelBlock {
            fields: vec![ModelField {
                name: "value".into(),
                ty: Type::Tag(shared),
                init: Expr::Int(0),
                mutable: false,
                is_primary: false,
                decorators: vec![],
            }],
        })]);
        let cloned = original.clone();
        let encoded = encode_ui_code(&original).expect("encode owned graph");
        let decoded = decode_ui_code(&encoded).expect("decode owned graph");

        drop(original);
        assert!(original_weak.upgrade().is_some(), "clone retains the owner");
        let decoded_weak = decoded
            .domain_stmts()
            .find_map(|stmt| match stmt {
                UiStmt::ModelBlock(model) => Some(model.fields[0].ty.clone()),
                _ => None,
            })
            .and_then(|ty| match ty {
                Type::Tag(shared) => Some(std::rc::Rc::downgrade(&shared)),
                _ => None,
            })
            .expect("decoded shared type");

        drop(cloned);
        assert!(
            original_weak.upgrade().is_none(),
            "decoded data must not retain the original through a global side table"
        );
        drop(decoded);
        assert!(
            decoded_weak.upgrade().is_none(),
            "last decoded owner releases the graph"
        );
    }

    #[test]
    fn visitor_observes_ui_view_and_core_items() {
        #[derive(Default)]
        struct Counts {
            ui: usize,
            view: usize,
            core: usize,
        }
        impl UiAstVisitor for Counts {
            fn visit_ui_stmt(&mut self, _stmt: &UiStmt) {
                self.ui += 1;
            }
            fn visit_core_stmt(&mut self, _stmt: &auto_ast::ast::Stmt) {
                self.core += 1;
            }
            fn visit_view_node(&mut self, _node: &ViewNode) {
                self.view += 1;
            }
        }
        let mut counts = Counts::default();
        walk_ui_code(&sample(), &mut counts);
        assert_eq!((counts.ui, counts.view, counts.core), (1, 1, 1));
    }
}
