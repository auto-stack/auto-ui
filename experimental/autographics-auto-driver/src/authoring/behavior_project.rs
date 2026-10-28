//! Immutable single-file behavior payloads, admitted by the public behavior validator.
use super::{
    behavior::{self, SourceSnapshot},
    document::Document,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Weak},
};
pub const MAX_CODES: usize = 8;
pub const MAX_BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeRef {
    pub format: String,
    pub sha256: String,
    pub byte_length: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actors {
    pub two_d: Option<u64>,
    pub three_d: Option<u64>,
}
impl Actors {
    pub fn ids(&self) -> [Option<u64>; 2] {
        [self.two_d, self.three_d]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub api: String,
    pub source: CodeRef,
    pub actors: Actors,
}
impl CodeRef {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "auto-source"
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.byte_length == 0
            || self.byte_length > behavior::MAX_USER
        {
            return Err("invalid behavior source identity/length".into());
        }
        Ok(())
    }
    pub fn relative_path(&self) -> String {
        format!("scripts/{}.at", self.sha256)
    }
}
impl Binding {
    pub fn validate(&self, d: &Document) -> Result<(), String> {
        self.source.validate()?;
        behavior::Api::parse(&self.api)?;
        for (i, id) in self.actors.ids().iter().enumerate() {
            if let Some(id) = id {
                if !d
                    .objects
                    .iter()
                    .any(|o| o.id == *id && o.kind.is_2d() == (i == 0))
                {
                    return Err("behavior actor reference/dimension: detach or retarget before deleting actor".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct CodePayload {
    identity: CodeRef,
    bytes: Arc<[u8]>,
    admitted_api: behavior::Api,
}
impl CodePayload {
    pub fn from_bytes(path: &Path, bytes: Vec<u8>) -> Result<Arc<Self>, String> {
        Self::from_bytes_for(path, bytes, behavior::Api::V1)
    }
    pub fn from_bytes_for(
        path: &Path,
        bytes: Vec<u8>,
        api: behavior::Api,
    ) -> Result<Arc<Self>, String> {
        let s = SourceSnapshot::from_bytes_for(path, bytes, api)?;
        let identity = CodeRef {
            format: "auto-source".into(),
            sha256: s.user_hash.clone(),
            byte_length: s.bytes.len(),
        };
        identity.validate()?;
        let mut vm =
            crate::Session::new_with_diagnostics(&s.source, |stage, e| s.sdk_diagnostic(stage, e))?;
        // Compilation creates the VM but does not dispatch main/app_start/app_update.
        vm.close();
        Ok(Arc::new(Self {
            identity,
            bytes: s.bytes.into(),
            admitted_api: api,
        }))
    }
    pub fn load_declared(path: &Path, expected: &CodeRef) -> Result<Arc<Self>, String> {
        Self::load_declared_for(path, expected, behavior::Api::V1)
    }
    pub fn load_declared_for(
        path: &Path,
        expected: &CodeRef,
        api: behavior::Api,
    ) -> Result<Arc<Self>, String> {
        expected.validate()?;
        let bytes = super::storage::read_code(path)?;
        if bytes.len() != expected.byte_length
            || agp::renderer::capture::sha256_hex(&bytes) != expected.sha256
        {
            return Err(format!(
                "behavior source closure identity: {}",
                path.display()
            ));
        }
        Self::from_bytes_for(path, bytes, api)
    }
    pub fn read(path: &Path) -> Result<Arc<Self>, String> {
        let s = SourceSnapshot::read(path)?;
        Self::from_bytes(path, s.bytes)
    }
    pub fn read_for(path: &Path, api: behavior::Api) -> Result<Arc<Self>, String> {
        let s = SourceSnapshot::read_for(path, api)?;
        Self::from_bytes_for(path, s.bytes, api)
    }
    pub fn identity(&self) -> &CodeRef {
        &self.identity
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn pin_bytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }
    pub fn snapshot(&self, path: &Path) -> Result<SourceSnapshot, String> {
        self.verify()?;
        SourceSnapshot::from_bytes(path, self.bytes.to_vec())
    }
    pub fn snapshot_for(&self, path: &Path, api: behavior::Api) -> Result<SourceSnapshot, String> {
        self.verify()?;
        SourceSnapshot::from_bytes_for(path, self.bytes.to_vec(), api)
    }
    pub fn verify(&self) -> Result<(), String> {
        self.identity.validate()?;
        if self.bytes.len() != self.identity.byte_length
            || agp::renderer::capture::sha256_hex(&self.bytes) != self.identity.sha256
        {
            return Err("behavior payload identity".into());
        }
        Ok(())
    }
}
pub type Code = Arc<CodePayload>;
#[derive(Default)]
pub struct Pool {
    live: BTreeMap<String, Weak<CodePayload>>,
}
impl Pool {
    pub fn admit(&mut self, code: Code) -> Result<Code, String> {
        code.verify()?;
        self.live.retain(|_, v| v.strong_count() > 0);
        if let Some(old) = self.live.get(&code.identity.sha256).and_then(Weak::upgrade) {
            if old.bytes != code.bytes {
                return Err("conflicting behavior bytes".into());
            }
            return Ok(old);
        }
        let size: usize = self
            .live
            .values()
            .filter_map(Weak::upgrade)
            .map(|v| v.bytes.len())
            .sum();
        if self.live.len() >= MAX_CODES || size + code.bytes.len() > MAX_BYTES {
            return Err("behavior payload budget: 8 sources/2MiB".into());
        }
        self.live
            .insert(code.identity.sha256.clone(), Arc::downgrade(&code));
        Ok(code)
    }
}
pub fn validate_payload(d: &Document, code: Option<&Code>) -> Result<(), String> {
    match (&d.behavior, code) {
        (None, None) => Ok(()),
        (Some(b), Some(c)) => {
            b.validate(d)?;
            c.verify()?;
            if b.source != c.identity {
                return Err("behavior binding/payload mismatch".into());
            }
            let api = behavior::Api::parse(&b.api)?;
            if c.admitted_api != api {
                let s = c.snapshot_for(Path::new("<accepted-behavior.at>"), api)?;
                let mut vm = crate::Session::new_with_diagnostics(&s.source, |stage, e| {
                    s.sdk_diagnostic(stage, e)
                })?;
                vm.close();
            }
            Ok(())
        }
        _ => Err("missing/unexpected behavior payload".into()),
    }
}
