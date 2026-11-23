use std::{io, path::Path};

use ace_av::age_component::{InitialValue, ScenarioOp};
use ace_core::atom_text::Span;
use ace_core::source::age_package::{AgePackageComponent, FILE_OPEN_BINDING};
use auto_lang_core::file_session::FileSession;

use super::AceStateValue;

const PENDING: &str = "file_open_pending";
const PATH: &str = "file_open_path";
pub(super) const REQUEST_FIELDS: &[&str] = &[PENDING, PATH];

#[derive(Clone)]
pub(crate) struct FileOpenBinding {
    reply: String,
    anchor: Span,
}

impl FileOpenBinding {
    pub(crate) fn from_component(component: &AgePackageComponent) -> Result<Option<Self>, String> {
        let mut bindings = component.members.iter().filter(|row| {
            row.role == "bind" && row.name == FILE_OPEN_BINDING
        });
        let Some(binding) = bindings.next() else { return Ok(None) };
        if bindings.next().is_some() {
            return Err("ace.widget.file-binding: duplicate file:open binding".into());
        }
        let reply = binding.text.as_deref().ok_or("ace.widget.file-binding: missing open reply handler")?;
        if !component.handler_msgs.iter().any(|name| name == reply)
            || !component.messages.iter().any(|msg| msg.name == reply && msg.payload == ["int"])
        {
            return Err("ace.widget.file-binding: file:open requires an int reply handler".into());
        }
        let field_type = |name| component.fields.iter()
            .find(|(field, _)| field == name).map(|(_, ty)| ty.as_str());
        if field_type(PENDING) != Some("bool") || !matches!(field_type(PATH), Some("str" | "Str")) {
            return Err("ace.widget.file-binding: file:open requires pending bool and path str fields".into());
        }
        let anchor = Span { start: binding.span.start, end: binding.span.start };
        Ok(Some(Self { reply: reply.to_owned(), anchor }))
    }

    pub(crate) fn reply_name(&self) -> &str {
        &self.reply
    }

    pub(super) fn pending_path<'a>(&self, states: &'a [(String, AceStateValue)]) -> Result<Option<&'a str>, String> {
        let value = |name| states.iter().find(|(field, _)| field == name).map(|(_, value)| value);
        match (value(PENDING), value(PATH)) {
            (Some(AceStateValue::Bool(false)), _) => Ok(None),
            (Some(AceStateValue::Bool(true)), Some(AceStateValue::Str(path))) => Ok(Some(path)),
            _ => Err("ace.widget.file-binding: invalid file open state".into()),
        }
    }

    pub(super) fn reply(&self, node: usize, status: i32) -> ScenarioOp {
        ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::send_args_from_values(
                0, &self.reply, &[InitialValue::Int(i64::from(status))], self.anchor,
            )),
        }
    }
}

pub(crate) const UNAVAILABLE: i32 = 0;
pub(super) const OPENED: i32 = 1;
pub(super) const DENIED: i32 = 2;

pub(super) fn open_document(session: &FileSession, path: &str) -> bool {
    session.with_open_target(Path::new(path), open_associated_document).is_ok()
}

#[cfg(windows)]
fn open_associated_document(path: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Component, Prefix};
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

    // Canonical Win32 paths use the extended prefix even for ordinary disk files.
    // The shell association expects a drive or UNC path.
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected an absolute disk path"));
    };
    if !matches!(prefix.kind(), Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(_, _))
        || path.components().any(|part| match part {
            Component::Normal(name) => name.encode_wide().last().is_some_and(|unit| unit == 32 || unit == 46),
            _ => false,
        })
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "path cannot be represented as a shell filename"));
    }
    let mut name: Vec<u16> = path.as_os_str().encode_wide().collect();
    let extended_unc = [b'\\', b'\\', b'?', b'\\', b'U', b'N', b'C', b'\\'].map(u16::from);
    let extended = [b'\\', b'\\', b'?', b'\\'].map(u16::from);
    if name.starts_with(&extended_unc) {
        name.splice(..extended_unc.len(), [u16::from(b'\\'); 2]);
    } else if name.starts_with(&extended) {
        name.drain(..extended.len());
    }
    if name.contains(&0) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "file path contains NUL"));
    }
    name.push(0);
    // The filename is NUL-terminated and stays alive through the shell call.
    let result = unsafe {
        ShellExecuteW(
            HWND::default(), w!("open"), PCWSTR::from_raw(name.as_ptr()),
            PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(io::Error::other("the desktop could not open this file association"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn open_associated_document(_: &Path) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "file association is unavailable"))
}
