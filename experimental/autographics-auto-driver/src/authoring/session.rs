//! Document/preview/history publication. GPU preparation must succeed before commit.
use super::{
    document::{Document, Group, Object},
    preview::{Building, Preview, SOURCE},
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
};
static EPOCH: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub epoch: u64,
    pub revision: u64,
}
#[derive(Clone, Debug)]
pub enum Edit {
    Behavior {
        binding: Option<super::behavior_project::Binding>,
        code: Option<super::behavior_project::Code>,
    },
    AssetObject {
        asset: Option<std::sync::Arc<super::assets::Asset>>,
        object: Object,
        create: bool,
    },
    /// PLAN-004: create a 2D leaf carrying an accepted PNG visual, rebind the
    /// selected 2D leaf to that payload, or detach it (asset and sprite None).
    SpriteObject {
        asset: Option<std::sync::Arc<super::assets::Asset>>,
        object: Object,
        create: bool,
    },
    Replace(Object),
    Create(Object),
    Delete(u64),
    CreateGroup(Group),
    ReplaceGroup(Group),
    Reparent {
        target: u64,
        expected_parent: Option<u64>,
        parent: Option<u64>,
    },
    Undo,
    Redo,
}
#[derive(Clone)]
struct Step {
    before: Document,
    after: Document,
    bytes: usize,
    before_code: Option<super::behavior_project::Code>,
    after_code: Option<super::behavior_project::Code>,
}
enum Intent {
    Edit,
    Undo,
    Redo,
    Open,
}
struct Pending {
    code: Option<super::behavior_project::Code>,
    doc: Document,
    build: Option<Building>,
    preview: Option<Preview>,
    intent: Intent,
    assets: super::assets::Registry,
}
pub struct Editor {
    pub code: Option<super::behavior_project::Code>,
    codes: super::behavior_project::Pool,
    pub assets: super::assets::Registry,
    pub document: Document,
    pub preview: Preview,
    pub stamp: Stamp,
    pub selection: Option<u64>,
    saved_hash: String,
    undo: VecDeque<Step>,
    redo: Vec<Step>,
    pending: Option<Pending>,
    pub history_limit: usize,
    closed: bool,
    high_water: u64,
}
impl Editor {
    pub fn new(d: Document) -> Result<Self, String> {
        Self::with_assets(d, Default::default())
    }
    pub fn with_assets(d: Document, assets: super::assets::Registry) -> Result<Self, String> {
        Self::with_project(d, assets, None)
    }
    pub fn with_project(
        d: Document,
        assets: super::assets::Registry,
        code: Option<super::behavior_project::Code>,
    ) -> Result<Self, String> {
        super::behavior_project::validate_payload(&d, code.as_ref())?;
        let mut codes = super::behavior_project::Pool::default();
        let code = code.map(|c| codes.admit(c)).transpose()?;
        validate_assets(&d, &assets)?;
        let preview = super::preview::build(&d)?;
        let saved_hash = d.hash()?;
        let high_water = d.ids().max().unwrap_or(0);
        Ok(Self {
            code,
            codes,
            assets,
            document: d,
            preview,
            stamp: Stamp {
                epoch: EPOCH.fetch_add(1, Ordering::Relaxed),
                revision: 0,
            },
            selection: None,
            saved_hash,
            undo: VecDeque::new(),
            redo: vec![],
            pending: None,
            history_limit: 16 * 1024 * 1024,
            closed: false,
            high_water,
        })
    }
    pub fn dirty(&self) -> bool {
        self.document
            .hash()
            .map(|h| h != self.saved_hash)
            .unwrap_or(true)
    }
    /// Read-only content/history fingerprint for runtime isolation, including redo bytes.
    pub fn isolation_identity(&self) -> serde_json::Value {
        let history: Vec<_> = self
            .undo
            .iter()
            .chain(&self.redo)
            .map(|s| (&s.before, &s.after, s.bytes))
            .collect();
        serde_json::json!({"document":self.document,"stamp":[self.stamp.epoch,self.stamp.revision],"selection":self.selection,"saved_hash":self.saved_hash,"dirty":self.dirty(),"history":self.history(),"history_hash":agp::renderer::capture::sha256_hex(&serde_json::to_vec(&history).unwrap()),"high_water":self.high_water,"closed":self.closed})
    }
    pub fn confirm_saved(&mut self, s: Stamp, hash: String) -> bool {
        if !self.closed && s.epoch == self.stamp.epoch {
            self.saved_hash = hash;
            true
        } else {
            false
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn next_id(&self) -> Option<u64> {
        self.high_water.checked_add(1)
    }
    pub fn history(&self) -> (usize, usize, usize) {
        (
            self.undo.len(),
            self.redo.len(),
            self.undo.iter().chain(&self.redo).map(|s| s.bytes).sum(),
        )
    }
    pub fn select(&mut self, id: Option<u64>) -> Result<(), String> {
        if self.pending() {
            return Err("pending transaction".into());
        }
        if id.is_some_and(|id| !self.document.contains(id)) {
            return Err("selection target".into());
        }
        self.selection = id;
        Ok(())
    }
    pub fn begin(&mut self, stamp: Stamp, edit: Edit) -> Result<bool, String> {
        self.begin_source(stamp, edit, SOURCE)
    }
    pub fn begin_source(&mut self, stamp: Stamp, edit: Edit, source: &str) -> Result<bool, String> {
        if self.closed || stamp != self.stamp {
            return Err("stale epoch/revision".into());
        }
        if self.pending() {
            return Err("pending transaction".into());
        }
        let mut code = self.code.clone();
        let mut doc = self.document.clone();
        let mut assets = self.assets.clone();
        let intent = match edit {
            Edit::Behavior {
                binding,
                code: next,
            } => {
                doc.behavior = binding;
                super::behavior_project::validate_payload(&doc, next.as_ref())?;
                code = next.map(|c| self.codes.admit(c)).transpose()?;
                Intent::Edit
            }
            Edit::AssetObject {
                asset,
                object,
                create,
            } => {
                if object.kind != super::document::Kind::Model {
                    return Err("asset edit requires Model kind".into());
                }
                if let Some(asset) = asset {
                    super::assets::insert(&mut assets, asset.clone())?;
                    if asset.identity.closure.is_some() {
                        doc.schema = "autographics.scene/6".into();
                    }
                    if !doc.assets.iter().any(|a| a.key() == asset.identity.key()) {
                        doc.assets.push(asset.identity.clone());
                    }
                }
                if create {
                    if object.id <= self.high_water {
                        return Err("created identity must exceed high-water".into());
                    }
                    doc.objects.push(object);
                } else {
                    if self.selection != Some(object.id) {
                        return Err("stale selection".into());
                    }
                    let target = doc
                        .objects
                        .iter_mut()
                        .find(|o| o.id == object.id)
                        .ok_or("target missing")?;
                    let mut expected = target.clone();
                    expected.kind = object.kind;
                    expected.resource = object.resource.clone();
                    if target.kind.is_2d() || expected != object {
                        return Err(
                            "resource replacement preserves 3D target/local/parent/properties"
                                .into(),
                        );
                    }
                    *target = object;
                }
                Intent::Edit
            }
            Edit::SpriteObject {
                asset,
                object,
                create,
            } => {
                // PLAN-004: the visual binding is an immutable image payload on
                // a 2D leaf. A 3D leaf or a model resource is rejected, and the
                // accepted payload is the one the binding names.
                if !object.kind.is_2d() {
                    return Err("sprite edit requires a 2D leaf".into());
                }
                if object.resource.is_some() {
                    return Err("sprite edit forbids a model resource".into());
                }
                if let Some(asset) = asset {
                    let key = asset.identity.key();
                    if object.sprite.as_ref().is_none_or(|s| s.asset != key) {
                        return Err("sprite binding identity".into());
                    }
                    if let Some(binding) = &object.sprite {
                        binding.validate(std::slice::from_ref(&asset.identity))?;
                    }
                    if asset.image.is_none() {
                        return Err("sprite edit requires an image payload".into());
                    }
                    super::assets::insert(&mut assets, asset.clone())?;
                    if !doc.assets.iter().any(|a| a.sha256 == asset.identity.sha256) {
                        doc.assets.push(asset.identity.clone());
                    }
                }
                if create {
                    if object.id <= self.high_water {
                        return Err("created identity must exceed high-water".into());
                    }
                    doc.objects.push(object);
                } else {
                    if self.selection != Some(object.id) {
                        return Err("stale selection".into());
                    }
                    let target = doc
                        .objects
                        .iter_mut()
                        .find(|o| o.id == object.id)
                        .ok_or("target missing")?;
                    if !target.kind.is_2d() || target.kind != object.kind {
                        return Err("sprite binding preserves the 2D target kind".into());
                    }
                    if target.translation != object.translation || target.parent != object.parent {
                        return Err("sprite binding preserves target/local/parent".into());
                    }
                    *target = object;
                }
                Intent::Edit
            }
            Edit::Replace(o) => {
                if self.selection != Some(o.id) {
                    return Err("stale selection".into());
                }
                let target = doc
                    .objects
                    .iter_mut()
                    .find(|x| x.id == o.id)
                    .ok_or("target missing")?;
                if target.parent != o.parent {
                    return Err("use keep-world reparent".into());
                }
                *target = o;
                Intent::Edit
            }
            Edit::Create(o) => {
                if o.id <= self.high_water {
                    return Err("created identity must exceed high-water".into());
                }
                doc.objects.push(o);
                Intent::Edit
            }
            Edit::CreateGroup(g) => {
                if g.id <= self.high_water {
                    return Err("created identity must exceed high-water".into());
                }
                doc.groups.push(g);
                Intent::Edit
            }
            Edit::ReplaceGroup(g) => {
                if self.selection != Some(g.id) {
                    return Err("stale selection".into());
                }
                let target = doc
                    .groups
                    .iter_mut()
                    .find(|x| x.id == g.id)
                    .ok_or("target missing")?;
                if target.parent != g.parent {
                    return Err("use keep-world reparent".into());
                }
                *target = g;
                Intent::Edit
            }
            Edit::Reparent {
                target,
                expected_parent,
                parent,
            } => {
                if self.selection != Some(target) {
                    return Err("stale selection".into());
                }
                doc = super::hierarchy::reparent(&doc, target, expected_parent, parent)?;
                Intent::Edit
            }
            Edit::Delete(id) => {
                let ids = super::hierarchy::descendants(&doc, id)?;
                doc.objects.retain(|o| !ids.contains(&o.id));
                doc.groups.retain(|g| !ids.contains(&g.id));
                Intent::Edit
            }
            Edit::Undo => {
                doc = self.undo.back().ok_or("undo empty")?.before.clone();
                code = self.undo.back().unwrap().before_code.clone();
                Intent::Undo
            }
            Edit::Redo => {
                doc = self.redo.last().ok_or("redo empty")?.after.clone();
                code = self.redo.last().unwrap().after_code.clone();
                Intent::Redo
            }
        };
        super::behavior_project::validate_payload(&doc, code.as_ref())?;
        let bytes = doc.encode()?.len() + self.document.encode()?.len();
        validate_assets(&doc, &assets)?;
        if bytes > self.history_limit {
            return Err("history candidate exceeds byte budget".into());
        }
        if doc == self.document {
            return Ok(false);
        }
        let build = Building::new(&doc, source)?;
        self.pending = Some(Pending {
            code,
            doc,
            build: Some(build),
            preview: None,
            intent,
            assets,
        });
        Ok(true)
    }
    pub fn begin_open(&mut self, d: Document) -> Result<(), String> {
        self.begin_open_assets(d, Default::default())
    }
    pub fn begin_open_assets(
        &mut self,
        d: Document,
        assets: super::assets::Registry,
    ) -> Result<(), String> {
        self.begin_open_project(d, assets, None)
    }
    pub fn begin_open_project(
        &mut self,
        d: Document,
        assets: super::assets::Registry,
        code: Option<super::behavior_project::Code>,
    ) -> Result<(), String> {
        if self.closed || self.pending() {
            return Err("closed/pending transaction".into());
        }
        super::behavior_project::validate_payload(&d, code.as_ref())?;
        let code = code.map(|c| self.codes.admit(c)).transpose()?;
        validate_assets(&d, &assets)?;
        let build = Building::new(&d, SOURCE)?;
        self.pending = Some(Pending {
            code,
            doc: d,
            build: Some(build),
            preview: None,
            intent: Intent::Open,
            assets,
        });
        Ok(())
    }
    pub fn pump(&mut self) -> Result<bool, String> {
        let result = (|| {
            let p = self.pending.as_mut().ok_or("no transaction")?;
            if let Some(b) = &mut p.build {
                if !b.pump()? {
                    return Ok(false);
                }
                let b = p.build.take().unwrap();
                p.preview = Some(b.finish()?);
            }
            Ok(true)
        })();
        if result.is_err() {
            self.pending = None
        }
        result
    }
    pub fn candidate(&self) -> Option<&Preview> {
        self.pending.as_ref()?.preview.as_ref()
    }
    pub fn candidate_assets(&self) -> &super::assets::Registry {
        self.pending.as_ref().map_or(&self.assets, |p| &p.assets)
    }
    pub fn candidate_owner_probe(&self) -> Option<Box<dyn Fn() -> bool>> {
        let p = self.pending.as_ref()?;
        if let Some(build) = &p.build {
            Some(build.owner_probe())
        } else {
            let probe = p.preview.as_ref()?.vm.owner_probe();
            Some(Box::new(probe))
        }
    }
    pub fn reject(&mut self) {
        self.pending = None;
    }
    pub fn commit(&mut self) -> Result<(), String> {
        if self
            .pending
            .as_ref()
            .and_then(|p| p.preview.as_ref())
            .is_none()
        {
            return Err("preview not ready".into());
        }
        let p = self.pending.take().unwrap();
        match p.intent {
            Intent::Edit => {
                let bytes = self.document.encode()?.len() + p.doc.encode()?.len();
                self.redo.clear();
                self.undo.push_back(Step {
                    before_code: self.code.clone(),
                    after_code: p.code.clone(),
                    before: self.document.clone(),
                    after: p.doc.clone(),
                    bytes,
                });
                while self.undo.len() > 64
                    || self.undo.iter().map(|s| s.bytes).sum::<usize>() > self.history_limit
                {
                    self.undo.pop_front();
                }
            }
            Intent::Undo => self.redo.push(self.undo.pop_back().ok_or("undo changed")?),
            Intent::Redo => self.undo.push_back(self.redo.pop().ok_or("redo changed")?),
            Intent::Open => {
                self.high_water = p.doc.ids().max().unwrap_or(0);
                self.undo.clear();
                self.redo.clear();
                self.saved_hash = p.doc.hash()?;
                self.selection = None;
                self.stamp.epoch = EPOCH.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.code = p.code;
        self.document = p.doc;
        self.assets = p.assets;
        let keep: std::collections::BTreeSet<_> = std::iter::once(&self.document)
            .chain(
                self.undo
                    .iter()
                    .chain(&self.redo)
                    .flat_map(|s| [&s.before, &s.after]),
            )
            .flat_map(|d| d.assets.iter().map(|a| a.key()))
            .collect();
        self.assets.retain(|k, _| keep.contains(k));
        self.high_water = self.high_water.max(self.document.ids().max().unwrap_or(0));
        self.preview = p.preview.unwrap();
        self.stamp.revision = self
            .stamp
            .revision
            .checked_add(1)
            .ok_or("revision exhausted")?;
        if self.selection.is_some_and(|id| !self.document.contains(id)) {
            self.selection = None
        }
        Ok(())
    }
    /// Admit the already checked final Auto preview through the ordinary history commit.
    pub fn commit_gesture(
        &mut self,
        stamp: Stamp,
        doc: Document,
        preview: Preview,
    ) -> Result<(), String> {
        if stamp != self.stamp || self.pending() || self.closed {
            return Err("stale/busy gesture".into());
        }
        doc.validate()?;
        if doc.behavior != self.document.behavior {
            return Err("gesture cannot change behavior binding".into());
        }
        super::behavior_project::validate_payload(&doc, self.code.as_ref())?;
        if !preview.matches(&doc) {
            return Err("gesture preview identity".into());
        }
        if doc.encode()?.len() + self.document.encode()?.len() > self.history_limit {
            return Err("history candidate exceeds byte budget".into());
        }
        if doc == self.document {
            return Ok(());
        }
        self.pending = Some(Pending {
            code: self.code.clone(),
            doc,
            preview: Some(preview),
            build: None,
            intent: Intent::Edit,
            assets: self.assets.clone(),
        });
        self.commit()
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.pending = None;
        self.undo.clear();
        self.redo.clear();
        self.preview.vm.close();
        self.selection = None;
        self.assets.clear();
        self.code = None;
    }
}
fn validate_assets(d: &Document, r: &super::assets::Registry) -> Result<(), String> {
    super::assets::check_registry(r)?;
    for a in &d.assets {
        let loaded = r.get(&a.key()).ok_or("missing asset payload")?;
        if loaded.identity.sha256 != a.sha256
            || loaded.identity.closure != a.closure
            || loaded.identity.byte_length != a.byte_length
            || loaded.identity.format != a.format
            || loaded.identity.width != a.width
            || loaded.identity.height != a.height
        {
            return Err("asset content identity/length".into());
        }
    }
    Ok(())
}
