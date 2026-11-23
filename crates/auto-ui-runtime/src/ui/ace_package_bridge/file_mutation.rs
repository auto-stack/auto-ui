use ace_av::age_component::{InitialValue, ScenarioOp};
use ace_core::atom_text::Span;
use ace_core::source::age_package::{AgePackageComponent, FILE_MUTATE_BINDING};
use auto_lang_core::file_session::{FileMutation, FileSession};

use super::AceStateValue;

const PENDING: &str = "file_mutation_pending";
const OPERATION: &str = "file_mutation_operation";
const PATH: &str = "file_mutation_path";
const TARGET: &str = "file_mutation_target";
pub(super) const REQUEST_FIELDS: &[&str] = &[PENDING, OPERATION, PATH, TARGET];

#[derive(Clone)]
pub(crate) struct FileMutationBinding {
    reply: String,
    anchor: Span,
}

pub(super) struct MutationRequest<'a> {
    operation: &'a str,
    path: &'a str,
    target: &'a str,
}

impl MutationRequest<'_> {
    pub(super) fn execute(self, session: &FileSession) -> bool {
        FileMutation::from_request(self.operation, self.path, self.target)
            .is_some_and(|action| session.mutate(action).is_ok())
    }
}

impl FileMutationBinding {
    pub(crate) fn from_component(component: &AgePackageComponent) -> Result<Option<Self>, String> {
        let mut bindings = component.members.iter()
            .filter(|row| row.role == "bind" && row.name == FILE_MUTATE_BINDING);
        let Some(binding) = bindings.next() else { return Ok(None) };
        if bindings.next().is_some() {
            return Err("ace.widget.file-binding: duplicate file:mutate binding".into());
        }
        let reply = binding.text.as_deref().ok_or("ace.widget.file-binding: missing mutation reply handler")?;
        if !component.handler_msgs.iter().any(|name| name == reply)
            || !component.messages.iter().any(|msg| msg.name == reply && msg.payload == ["bool"])
        {
            return Err("ace.widget.file-binding: file:mutate requires a bool reply handler".into());
        }
        let field_type = |name| component.fields.iter()
            .find(|(field, _)| field == name).map(|(_, ty)| ty.as_str());
        if field_type(PENDING) != Some("bool") || [OPERATION, PATH, TARGET].into_iter()
            .any(|field| !matches!(field_type(field), Some("str" | "Str")))
        {
            return Err("ace.widget.file-binding: file:mutate requires pending bool, operation str, path str and target str fields".into());
        }
        let anchor = Span { start: binding.span.start, end: binding.span.start };
        Ok(Some(Self { reply: reply.to_owned(), anchor }))
    }

    pub(crate) fn reply_name(&self) -> &str {
        &self.reply
    }

    pub(super) fn pending_request<'a>(&self, states: &'a [(String, AceStateValue)]) -> Result<Option<MutationRequest<'a>>, String> {
        let value = |name| states.iter().find(|(field, _)| field == name).map(|(_, value)| value);
        match (value(PENDING), value(OPERATION), value(PATH), value(TARGET)) {
            (Some(AceStateValue::Bool(false)), _, _, _) => Ok(None),
            (Some(AceStateValue::Bool(true)), Some(AceStateValue::Str(operation)),
                Some(AceStateValue::Str(path)), Some(AceStateValue::Str(target))) => {
                Ok(Some(MutationRequest { operation, path, target }))
            }
            _ => Err("ace.widget.file-binding: invalid file mutation state".into()),
        }
    }

    pub(super) fn reply(&self, node: usize, ok: bool) -> ScenarioOp {
        ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::send_args_from_values(
                0, &self.reply, &[InitialValue::Bool(ok)], self.anchor,
            )),
        }
    }
}
