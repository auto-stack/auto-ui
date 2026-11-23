use ace_av::age_component::ScenarioOp;
use ace_av::age_package::live_session::HostInput;
use ace_core::atom_text::Span;
use ace_core::source::{
    age_package::{AgePackageCompilation, AgePackageComponent, FILE_LIST_BINDING},
    ast::{Expr, ExprKind, Ident, Item, NamedCtorField, TypeAnn, TypeRef, TypeWord},
};
use auto_lang_core::file_session::{DirectoryEntry, DirectoryListing, FileSession};
use std::path::Path;

use super::AceStateValue;

const PENDING: &str = "file_request_pending";
const PATH: &str = "file_request_path";
pub(super) const REQUEST_FIELDS: &[&str] = &[PENDING, PATH];

#[derive(Clone)]
pub(crate) struct FileDirectoryBinding {
    reply: String,
    anchor: Span,
    pub(crate) include_extension: bool,
}

impl FileDirectoryBinding {
    pub(super) fn list(access: &FileSession, path: &str) -> Option<DirectoryListing> {
        if path.is_empty() {
            access.list_root()
        } else {
            access.list(Path::new(path))
        }.ok()
    }

    pub(crate) fn from_package(compiled: &AgePackageCompilation, component: &AgePackageComponent) -> Result<Option<Self>, String> {
        let mut bindings = component.members.iter()
            .filter(|row| row.role == "bind" && row.name == FILE_LIST_BINDING);
        let Some(binding) = bindings.next() else { return Ok(None) };
        if bindings.next().is_some() {
            return Err("ace.desktop.file-binding: duplicate file:list binding".into());
        }
        let reply = binding.text.as_deref().ok_or("ace.desktop.file-binding: missing reply handler")?;
        if !component.handler_msgs.iter().any(|name| name == reply)
            || !component.messages.iter().any(|msg| msg.name == reply && msg.payload == ["DirListing"])
        {
            return Err("ace.desktop.file-binding: file:list requires a DirListing reply handler".into());
        }
        let field_type = |name| component.fields.iter()
            .find(|(field, _)| field == name).map(|(_, ty)| ty.as_str());
        if field_type(PENDING) != Some("bool") || !matches!(field_type(PATH), Some("str" | "Str")) {
            return Err("ace.desktop.file-binding: file:list requires file_request_pending bool and file_request_path str".into());
        }
        let anchor = Span { start: binding.span.start, end: binding.span.start };
        let include_extension = compiled.bound.units.get(&component.module).is_some_and(|unit| {
            unit.program.items.iter().any(|item| matches!(item, Item::Type(record)
                if record.name.name == "DirEntry" && record.fields.iter().any(|field| field.name.name == "extension")))
        });
        Ok(Some(Self { reply: reply.to_owned(), anchor, include_extension }))
    }

    pub(crate) fn reply_name(&self) -> &str {
        &self.reply
    }

    pub(super) fn pending_path<'a>(&self, states: &'a [(String, AceStateValue)]) -> Result<Option<&'a str>, String> {
        let value = |name| states.iter().find(|(field, _)| field == name).map(|(_, v)| v);
        match (value(PENDING), value(PATH)) {
            (Some(AceStateValue::Bool(false)), _) => Ok(None),
            (Some(AceStateValue::Bool(true)), Some(AceStateValue::Str(path))) => Ok(Some(path)),
            _ => Err("ace.desktop.file-binding: invalid file request state".into()),
        }
    }

    pub(super) fn check_replies(&self, node: usize) -> Vec<ScenarioOp> {
        vec![
            self.reply(node, None),
            self.reply(node, Some(DirectoryListing {
                path: String::new(),
                entries: vec![DirectoryEntry {
                    name: String::new(),
                    path: String::new(),
                    is_dir: false,
                    size: u64::MAX,
                    mtime: i64::MIN,
                    extension: String::new(),
                }],
            })),
        ]
    }

    pub(super) fn host_reply(&self, listing: Option<DirectoryListing>) -> HostInput {
        let ok = listing.is_some();
        let DirectoryListing { path, entries } = listing.unwrap_or_else(|| DirectoryListing {
            path: String::new(),
            entries: Vec::new(),
        });
        let entries = entries.into_iter().map(|entry| {
            let mut fields = vec![
                ("name".into(), HostInput::Text(entry.name)),
                ("path".into(), HostInput::Text(entry.path)),
                ("is_dir".into(), HostInput::Bool(entry.is_dir)),
                ("size".into(), HostInput::U64(entry.size)),
                ("mtime".into(), HostInput::I64(entry.mtime)),
            ];
            if self.include_extension {
                fields.push(("extension".into(), HostInput::Text(entry.extension)));
            }
            HostInput::Record(fields)
        }).collect();
        HostInput::Record(vec![
            ("ok".into(), HostInput::Bool(ok)),
            ("path".into(), HostInput::Text(path)),
            ("entries".into(), HostInput::List(entries)),
        ])
    }

    pub(super) fn reply(&self, node: usize, listing: Option<DirectoryListing>) -> ScenarioOp {
        let ok = listing.is_some();
        let DirectoryListing { path, entries } = listing.unwrap_or_else(|| DirectoryListing {
            path: String::new(),
            entries: Vec::new(),
        });
        let at = self.anchor;
        let ident = |name: &str| Ident { name: name.into(), span: at };
        let expr = |kind| Expr { kind, span: at };
        let record = |ty, fields: Vec<(&str, Expr)>| {
            expr(ExprKind::CtorNamed {
                ty: ident(ty),
                args: Vec::new(),
                fields: fields.into_iter().map(|(name, value)| NamedCtorField {
                    name: ident(name), value, span: at,
                }).collect(),
            })
        };
        let entries = if entries.is_empty() {
            expr(ExprKind::MethodCall {
                recv: Box::new(expr(ExprKind::GenericType {
                    name: ident("List"),
                    args: vec![TypeAnn { kind: TypeRef::Record(ident("DirEntry")), span: at }],
                })),
                name: ident("new"),
                args: Vec::new(),
                instance: None,
            })
        } else {
            expr(ExprKind::ArrayLit(entries.into_iter().map(|entry| {
                let mut fields = vec![
                    ("name", expr(ExprKind::StrLit(entry.name))),
                    ("path", expr(ExprKind::StrLit(entry.path))),
                    ("is_dir", expr(ExprKind::BoolLit(entry.is_dir))),
                    ("size", expr(ExprKind::TypedLit { ty: TypeWord::U64, bits: entry.size })),
                    ("mtime", expr(ExprKind::TypedLit { ty: TypeWord::I64, bits: entry.mtime as u64 })),
                ];
                if self.include_extension {
                    fields.push(("extension", expr(ExprKind::StrLit(entry.extension))));
                }
                record("DirEntry", fields)
            }).collect()))
        };
        let reply = expr(ExprKind::CtorNamed {
            ty: ident("DirListing"),
            args: Vec::new(),
            fields: [
                ("ok", expr(ExprKind::BoolLit(ok))),
                ("path", expr(ExprKind::StrLit(path))),
                ("entries", entries),
            ].into_iter().map(|(name, value)| NamedCtorField {
                name: ident(name), value, span: at,
            }).collect(),
        });
        ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::SendArgs {
                instance: 0,
                msg: self.reply.clone(),
                args: vec![reply],
            }),
        }
    }
}
