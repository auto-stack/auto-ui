//! Local same-volume single-writer saves. Rename commits; verification confirms.
use super::{
    document::{Document, MAX_FILE},
    session::{Editor, Stamp},
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static SEQ: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fault {
    #[default]
    None,
    Prepare,
    Write,
    Flush,
    BeforeReplace,
    AfterReplaceRead,
    AssetAfterPrepare,
    AssetAfterInstall,
    ClosureAfterReplace,
    CodeAfterPrepare,
    CodeAfterInstall,
    CodeClosureAfterReplace,
}
#[derive(Clone)]
pub struct Snapshot {
    pub code: Option<super::behavior_project::Code>,
    pub stamp: Stamp,
    pub hash: String,
    pub bytes: Vec<u8>,
    pub assets: super::assets::Registry,
}
impl Snapshot {
    pub fn capture(e: &Editor) -> Result<Self, String> {
        if e.pending() {
            return Err("save waits for accepted edit".into());
        }
        super::behavior_project::validate_payload(&e.document, e.code.as_ref())?;
        Ok(Self {
            code: e.code.clone(),
            stamp: e.stamp,
            hash: e.document.hash()?,
            bytes: e.document.encode()?,
            assets: e
                .document
                .assets
                .iter()
                .map(|a| {
                    Ok((
                        a.key(),
                        e.assets
                            .get(&a.key())
                            .ok_or("missing snapshot asset")?
                            .clone(),
                    ))
                })
                .collect::<Result<_, String>>()?,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Saved {
    Confirmed,
    VerificationPending,
}
struct OwnedFile {
    path: PathBuf,
    identity: Vec<u8>,
}
impl Drop for OwnedFile {
    fn drop(&mut self) {
        if fs::read(&self.path).ok().as_deref() == Some(&self.identity) {
            let _ = fs::remove_file(&self.path);
        }
    }
}
#[derive(Default)]
pub struct Storage {
    pub path: Option<PathBuf>,
    expected: Option<Vec<u8>>,
    // Only identities verified in the current project root; never retains source owners.
    confirmed_sources: Vec<super::behavior_project::CodeRef>,
    unverified: Option<(PathBuf, Snapshot)>,
    pub residuals: Vec<PathBuf>,
}
fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut b = Vec::new();
    f.take(MAX_FILE as u64 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > MAX_FILE {
        return Err("file limit 1MiB".into());
    }
    Ok(b)
}
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::metadata(path) {
        Ok(_) => read_limited(path).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
impl Storage {
    pub fn load(path: &Path) -> Result<(Document, Self), String> {
        let (d, _, s) = Self::load_assets(path, &Default::default())?;
        Ok((d, s))
    }
    pub fn load_assets(
        path: &Path,
        existing: &super::assets::Registry,
    ) -> Result<(Document, super::assets::Registry, Self), String> {
        let (d, a, c, s) = Self::load_project(path, existing)?;
        if c.is_some() {
            return Err("bound scene requires load_project".into());
        }
        Ok((d, a, s))
    }
    pub fn load_project(
        path: &Path,
        existing: &super::assets::Registry,
    ) -> Result<
        (
            Document,
            super::assets::Registry,
            Option<super::behavior_project::Code>,
            Self,
        ),
        String,
    > {
        super::assets::regular_path(path, false)?;
        let path = absolute(path)?;
        let bytes = read_limited(&path)?;
        let doc = Document::decode(&bytes)?;
        let mut assets = existing.clone();
        for a in &doc.assets {
            let blob = path
                .parent()
                .ok_or("project root")?
                .join("assets")
                .join(&blob_name(a)?);
            let loaded = super::assets::load_declared(&blob, a, &assets)?;
            super::assets::insert(&mut assets, loaded)?;
        }
        let code = if let Some(b) = &doc.behavior {
            let blob = path
                .parent()
                .ok_or("project root")?
                .join(b.source.relative_path());
            let c = super::behavior_project::CodePayload::load_declared_for(
                &blob,
                &b.source,
                super::behavior::Api::parse(&b.api)?,
            )?;
            super::behavior_project::validate_payload(&doc, Some(&c))?;
            Some(c)
        } else {
            None
        };
        let confirmed_sources = code.iter().map(|c| c.identity().clone()).collect();
        Ok((
            doc,
            assets,
            code,
            Self {
                path: Some(path),
                expected: Some(bytes),
                confirmed_sources,
                unverified: None,
                residuals: vec![],
            },
        ))
    }
    /// Attribution uses completed closure verification, not scene dirtiness or a disk reread.
    pub fn source_origin(&self, source: &super::behavior_project::CodeRef) -> PathBuf {
        self.path
            .as_ref()
            .filter(|_| self.confirmed_sources.contains(source))
            .and_then(|p| p.parent())
            .map(|root| root.join(source.relative_path()))
            .unwrap_or_else(|| PathBuf::from(format!("<memory:{}>.at", source.sha256)))
    }
    pub fn retained_assets(&self) -> Option<&super::assets::Registry> {
        self.unverified.as_ref().map(|(_, s)| &s.assets)
    }
    pub fn unverified(&self) -> bool {
        self.unverified.is_some()
    }
    pub fn verify(&mut self, e: &mut Editor) -> Result<bool, String> {
        let Some((path, _)) = &self.unverified else {
            return Ok(false);
        };
        let _root = root_lock(path)?;
        self.verify_locked(e)
    }
    fn verify_locked(&mut self, e: &mut Editor) -> Result<bool, String> {
        let Some((path, snapshot)) = &self.unverified else {
            return Ok(false);
        };
        let bytes = read_limited(path)?;
        if bytes != snapshot.bytes {
            return Err("committed result differs; resolve external conflict".into());
        }
        verify_closure(path, &snapshot.assets)?;
        verify_code(path, snapshot.code.as_ref())?;
        if !e.confirm_saved(snapshot.stamp, snapshot.hash.clone()) {
            return Err("old epoch save result".into());
        }
        // A new root cannot inherit proof about blobs in the old root. Pending or
        // failed saves never reach this point; old-revision receipts confirm only
        // their own source identity, not the editor's current binding.
        if self.path.as_ref().and_then(|p| p.parent()) != path.parent() {
            self.confirmed_sources.clear();
        }
        if let Some(code) = &snapshot.code {
            if !self.confirmed_sources.contains(code.identity()) {
                if self.confirmed_sources.len() == super::behavior_project::MAX_CODES {
                    self.confirmed_sources.remove(0);
                }
                self.confirmed_sources.push(code.identity().clone());
            }
        }
        self.path = Some(path.clone());
        self.expected = Some(bytes);
        let blobs = super::textured_models::blobs(&snapshot.assets)?;
        self.residuals.retain(|p| {
            !blobs.values().any(|a| {
                p == &path
                    .parent()
                    .unwrap()
                    .join("assets")
                    .join(blob_name(&a.identity).expect("validated blob format"))
            })
        });
        if let Some(c) = &snapshot.code {
            let blob = path.parent().unwrap().join(c.identity().relative_path());
            self.residuals.retain(|p| p != &blob);
        }
        self.unverified = None;
        Ok(true)
    }
    pub fn save(&mut self, e: &mut Editor, target: &Path, fault: Fault) -> Result<Saved, String> {
        let snapshot = Snapshot::capture(e)?;
        self.save_snapshot(e, target, snapshot, fault)
    }
    pub fn save_snapshot(
        &mut self,
        e: &mut Editor,
        target: &Path,
        snapshot: Snapshot,
        fault: Fault,
    ) -> Result<Saved, String> {
        let declared = Document::decode(&snapshot.bytes)?;
        super::behavior_project::validate_payload(&declared, snapshot.code.as_ref())?;
        super::assets::check_registry(&snapshot.assets)?;
        if declared.assets.len() != snapshot.assets.len() {
            return Err("snapshot asset closure count".into());
        }
        for a in &declared.assets {
            let loaded = snapshot
                .assets
                .get(&a.key())
                .ok_or("missing declared snapshot asset")?;
            if !loaded.identity.same_content(a) {
                return Err("snapshot asset identity".into());
            }
        }
        if declared.hash()? != snapshot.hash {
            return Err("snapshot hash mismatch".into());
        }
        if snapshot.stamp.epoch != e.stamp.epoch {
            return Err("old epoch save request".into());
        }
        super::assets::regular_path(target, true)?;
        // Classify the destination that rename will actually replace. In the
        // Windows profile, spelling/case/.. aliases must protect the same blob
        // and lock, including an existing file's canonical long name.
        let target = absolute(target)?;
        let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.eq_ignore_ascii_case("scripts")
            || name.eq_ignore_ascii_case("assets")
            || name.to_ascii_lowercase().ends_with(".authoring-lock")
            || reserved_temp(name)
            || (target
                .parent()
                .and_then(Path::file_name)
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("scripts"))
                && name.len() == 67
                && name.as_bytes()[64..].eq_ignore_ascii_case(b".at")
                && name.as_bytes()[..64].iter().all(|b| b.is_ascii_hexdigit()))
            || name.eq_ignore_ascii_case(".autographics-assets-lock")
            || (target
                .parent()
                .and_then(Path::file_name)
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("assets"))
                && name.len() == 68
                && (name.as_bytes()[64..].eq_ignore_ascii_case(b".glb")
                    || name.as_bytes()[64..].eq_ignore_ascii_case(b".png")
                    || name.as_bytes()[64..].eq_ignore_ascii_case(b".ogg")
                    || name.as_bytes()[64..].eq_ignore_ascii_case(b".wav")
                    || name.as_bytes()[64..].eq_ignore_ascii_case(b".ttf"))
                && name.as_bytes()[..64].iter().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(
                "scene entry cannot replace a reserved project lock or immutable asset blob".into(),
            );
        }
        if self.unverified.is_some() {
            // A prior Save As receipt may belong to another root. Verify it
            // under that root's lock, then acquire this save's root separately.
            self.verify(e)?;
        }
        let _root = root_lock(&target)?;
        let parent = target.parent().ok_or("target parent")?;
        if !parent.is_dir() {
            return Err("parent directory must already exist".into());
        }
        if fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("symlink target unsupported".into());
        }
        let mut lockname = target.as_os_str().to_os_string();
        lockname.push(".authoring-lock");
        let lockpath = PathBuf::from(lockname);
        let token = format!(
            "{}:{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        )
        .into_bytes();
        let mut lock = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lockpath)
            .map_err(|e| format!("save occupied: {e}"))?;
        lock.write_all(&token).map_err(|e| e.to_string())?;
        lock.sync_all().map_err(|e| e.to_string())?;
        drop(lock);
        let _guard = OwnedFile {
            path: lockpath,
            identity: token,
        };
        let original = read_optional(&target)?;
        if self.path.as_ref() == Some(&target) && original != self.expected {
            return Err("external file modification".into());
        }
        if fault == Fault::Prepare {
            return Err("injected prepare failure".into());
        }
        prepare_closure(&target, &snapshot.assets, fault, &mut self.residuals)?;
        prepare_code(&target, snapshot.code.as_ref(), fault, &mut self.residuals)?;
        let name = target
            .file_name()
            .ok_or("target filename")?
            .to_string_lossy();
        let mut chosen = None;
        for _ in 0..32 {
            let temp = parent.join(format!(
                ".{name}.authoring-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&temp) {
                Ok(f) => {
                    chosen = Some((temp, f));
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        }
        let (temp, mut f) = chosen.ok_or("staging names occupied")?;
        // We own the newly created path. Cleanup checks its exact current content.
        let mut owned = OwnedFile {
            path: temp.clone(),
            identity: vec![],
        };
        let written = (|| {
            if fault == Fault::Write {
                return Err("injected write failure".into());
            }
            f.write_all(&snapshot.bytes).map_err(|e| e.to_string())?;
            owned.identity = snapshot.bytes.clone();
            if fault == Fault::Flush {
                return Err("injected flush failure".into());
            }
            f.sync_all().map_err(|e| e.to_string())?;
            Ok::<_, String>(())
        })();
        drop(f);
        written?;
        if read_limited(&temp)? != snapshot.bytes {
            return Err("staging readback mismatch".into());
        }
        if fault == Fault::BeforeReplace {
            return Err("injected replace failure".into());
        }
        if read_optional(&target)? != original {
            return Err("external modification during save".into());
        }
        fs::rename(&temp, &target).map_err(|e| format!("replace failed: {e}"))?;
        self.unverified = Some((target.clone(), snapshot));
        if fault == Fault::AfterReplaceRead
            || fault == Fault::ClosureAfterReplace
            || fault == Fault::CodeClosureAfterReplace
        {
            return Ok(Saved::VerificationPending);
        }
        match self.verify_locked(e) {
            Ok(true) => Ok(Saved::Confirmed),
            Ok(false) => Err("missing save receipt".into()),
            Err(_) => Ok(Saved::VerificationPending),
        }
    }
}
fn absolute(p: &Path) -> Result<PathBuf, String> {
    match p.canonicalize() {
        Ok(path) => return Ok(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?.join(p)
    };
    let parent = p
        .parent()
        .ok_or("parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    Ok(parent.join(p.file_name().ok_or("filename")?))
}
fn root_lock(entry: &Path) -> Result<OwnedFile, String> {
    super::assets::regular_path(entry, true)?;
    let root = entry.parent().ok_or("project root")?;
    let path = root.join(".autographics-assets-lock");
    let identity = format!(
        "{}:{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    )
    .into_bytes();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("project save occupied: {e}"))?;
    // Install guard before writing; partial/changed contents are never removed.
    let guard = OwnedFile {
        path,
        identity: identity.clone(),
    };
    f.write_all(&identity)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(guard)
}
fn disk_budget(dir: &Path, reserve: usize) -> Result<(), String> {
    super::assets::regular_path(dir, false)?;
    let (mut files, mut bytes) = (0usize, 0usize);
    for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        files += 1;
        if files > super::assets::MAX_ASSETS {
            return Err("resource limit: root asset files 8 (including residuals/temps)".into());
        }
        super::assets::regular_path(&item.path(), false)?;
        let m = item.metadata().map_err(|e| e.to_string())?;
        if !m.is_file() {
            return Err(format!("foreign asset path: {}", item.path().display()));
        }
        bytes = bytes
            .checked_add(usize::try_from(m.len()).map_err(|_| "disk length")?)
            .ok_or("disk length overflow")?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name.len() != 68
            || !(name.ends_with(".glb")
                || name.ends_with(".png")
                || name.ends_with(".ogg")
                || name.ends_with(".wav")
                || name.ends_with(".ttf"))
            || !name[..64]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(format!(
                "foreign or incomplete asset path (leave untouched): {}",
                item.path().display()
            ));
        }
    }
    if files + usize::from(reserve > 0) > super::assets::MAX_ASSETS
        || bytes
            .checked_add(reserve)
            .is_none_or(|n| n > super::assets::MAX_RAW_TOTAL)
    {
        return Err(
            "resource limit: root assets 8 files/32MiB incl residuals and temporary reservation"
                .into(),
        );
    }
    Ok(())
}
/// PLAN-004: a project-owned blob is named by its content identity and its
/// format extension. The image and model payloads live in the same assets
/// directory and share one file/byte budget, so neither gains a separate quota.
fn blob_name(a: &super::assets::AssetRef) -> Result<String, String> {
    let extension = match a.format.as_str() {
        "png" => "png",
        "glb" => "glb",
        "ogg" => "ogg",
        "wav" => "wav",
        "ttf" => "ttf",
        _ => return Err("invalid: closure blob format".into()),
    };
    if a.format != extension {
        return Err("invalid: closure blob format".into());
    }
    Ok(format!("{}.{}", a.sha256, extension))
}
fn verify_closure(entry: &Path, assets: &super::assets::Registry) -> Result<(), String> {
    let blobs = super::textured_models::blobs(assets)?;
    for a in blobs.values() {
        let path = entry
            .parent()
            .ok_or("root")?
            .join("assets")
            .join(&blob_name(&a.identity)?);
        let bytes = super::assets::read_blob(&path)?;
        if bytes.as_slice() != a.bytes.as_ref()
            || bytes.len() != a.identity.byte_length
            || agp::renderer::capture::sha256_hex(&bytes) != a.identity.sha256
        {
            return Err(format!("conflict: asset closure {}", path.display()));
        }
    }
    Ok(())
}
fn prepare_closure(
    entry: &Path,
    assets: &super::assets::Registry,
    fault: Fault,
    residuals: &mut Vec<PathBuf>,
) -> Result<(), String> {
    super::assets::check_registry(assets)?;
    let blobs = super::textured_models::blobs(assets)?;
    let dir = entry.parent().ok_or("root")?.join("assets");
    if assets.is_empty() {
        return Ok(());
    }
    super::assets::regular_path(&dir, true)?;
    if !dir.exists() {
        fs::create_dir(&dir).map_err(|e| format!("create assets: {e}"))?;
    }
    for asset in blobs.values() {
        disk_budget(&dir, 0)?;
        let target = dir.join(blob_name(&asset.identity)?);
        if target.exists() {
            let bytes = super::assets::read_blob(&target)?;
            if bytes.as_slice() != asset.bytes.as_ref() {
                return Err(format!("conflict: immutable blob {}", target.display()));
            }
            continue;
        }
        disk_budget(&dir, asset.bytes.len())?;
        let temp = dir.join(format!(
            ".ag-asset-{}-{}.tmp",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        let mut guard = OwnedFile {
            path: temp.clone(),
            identity: vec![],
        };
        let written = f.write_all(&asset.bytes);
        if written.is_ok() {
            guard.identity = asset.bytes.to_vec();
        }
        let written = written.and_then(|_| f.sync_all());
        drop(f);
        if let Err(e) = written {
            drop(guard);
            if temp.exists() {
                residuals.push(temp.clone());
                return Err(format!(
                    "asset staging failed; residual {}: {e}",
                    temp.display()
                ));
            }
            return Err(format!(
                "asset staging failed, owned temporary removed: {e}"
            ));
        }
        if super::assets::read_blob(&temp)?.as_slice() != asset.bytes.as_ref() {
            residuals.push(temp.clone());
            return Err(format!("asset staging mismatch: {}", temp.display()));
        }
        if fault == Fault::AssetAfterPrepare {
            return Err("injected asset-before-install failure".into());
        }
        install_blob(&temp, &target).map_err(|e| format!("{}: {e}", target.display()))?;
        residuals.push(target);
        if fault == Fault::AssetAfterInstall {
            return Err(format!(
                "injected asset-before-entry failure; owned residuals: {residuals:?}"
            ));
        }
    }
    verify_closure(entry, assets)
}
#[cfg(windows)]
fn install_blob(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let from: Vec<_> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // WRITE_THROUGH, deliberately no REPLACE_EXISTING. A single rename means
    // the final name and temporary reservation never count as two files.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 8) } == 0 {
        return Err(format!(
            "immutable install (no overwrite): {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
#[cfg(not(windows))]
fn install_blob(_: &Path, _: &Path) -> Result<(), String> {
    Err("unsupported: asset persistence profile is Windows".into())
}

pub(crate) fn read_code(path: &Path) -> Result<Vec<u8>, String> {
    super::assets::regular_path(path, false)?;
    let f = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err(format!("regular source required: {}", path.display()));
    }
    let mut bytes = vec![];
    f.take(super::behavior::MAX_USER as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > super::behavior::MAX_USER {
        return Err("source exceeds 256KiB".into());
    }
    Ok(bytes)
}
fn verify_code(entry: &Path, code: Option<&super::behavior_project::Code>) -> Result<(), String> {
    if let Some(c) = code {
        c.verify()?;
        let path = entry
            .parent()
            .ok_or("root")?
            .join(c.identity().relative_path());
        if read_code(&path)? != c.bytes() {
            return Err(format!("conflict: behavior closure {}", path.display()));
        }
    }
    Ok(())
}
fn code_disk_budget(dir: &Path, reserve: usize) -> Result<(), String> {
    super::assets::regular_path(dir, false)?;
    let (mut files, mut bytes) = (0usize, 0usize);
    for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        super::assets::regular_path(&path, false)?;
        let name = item.file_name().to_string_lossy().into_owned();
        if !item.metadata().map_err(|e| e.to_string())?.is_file()
            || name.len() != 67
            || !name.ends_with(".at")
            || !name.as_bytes()[..64]
                .iter()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
        {
            return Err(format!(
                "foreign or incomplete source path (leave untouched): {}",
                path.display()
            ));
        }
        let raw = read_code(&path)?;
        if raw.is_empty() || agp::renderer::capture::sha256_hex(&raw) != name[..64] {
            return Err(format!("conflict: immutable source {}", path.display()));
        }
        files += 1;
        bytes = bytes.checked_add(raw.len()).ok_or("source disk overflow")?;
    }
    if files + usize::from(reserve > 0) > super::behavior_project::MAX_CODES
        || bytes
            .checked_add(reserve)
            .is_none_or(|n| n > super::behavior_project::MAX_BYTES)
    {
        return Err("source disk budget: 8 files/2MiB incl residuals/temporary reservation".into());
    }
    Ok(())
}
fn prepare_code(
    entry: &Path,
    code: Option<&super::behavior_project::Code>,
    fault: Fault,
    residuals: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let Some(c) = code else {
        return Ok(());
    };
    c.verify()?;
    let dir = entry.parent().ok_or("root")?.join("scripts");
    super::assets::regular_path(&dir, true)?;
    if !dir.exists() {
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
    }
    code_disk_budget(&dir, 0)?;
    let target = entry.parent().unwrap().join(c.identity().relative_path());
    if target.exists() {
        return verify_code(entry, Some(c));
    }
    code_disk_budget(&dir, c.bytes().len())?;
    let temp = dir.join(format!(
        ".ag-code-{}-{}.tmp",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let mut guard = OwnedFile {
        path: temp.clone(),
        identity: vec![],
    };
    let result = f.write_all(c.bytes());
    if result.is_ok() {
        guard.identity = c.bytes().to_vec();
    }
    let result = result.and_then(|_| f.sync_all());
    drop(f);
    if let Err(e) = result {
        drop(guard);
        if temp.exists() {
            residuals.push(temp.clone());
        }
        return Err(format!(
            "source staging {}: {e}; residuals: {residuals:?}",
            temp.display()
        ));
    }
    if read_code(&temp)? != c.bytes() {
        return Err("source staging readback mismatch".into());
    }
    if fault == Fault::CodeAfterPrepare {
        return Err("injected code-before-install failure".into());
    }
    install_blob(&temp, &target)?;
    residuals.push(target);
    if fault == Fault::CodeAfterInstall {
        return Err(format!(
            "injected code-before-entry failure; residuals: {residuals:?}"
        ));
    }
    verify_code(entry, Some(c))
}

fn reserved_temp(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let pair = |s: &str| {
        s.split_once('-').is_some_and(|(a, b)| {
            !a.is_empty() && !b.is_empty() && a.bytes().chain(b.bytes()).all(|b| b.is_ascii_digit())
        })
    };
    (lower.starts_with('.')
        && lower
            .rsplit_once(".authoring-")
            .is_some_and(|(_, tail)| pair(tail)))
        || [".ag-code-", ".ag-asset-"].iter().any(|prefix| {
            lower
                .strip_prefix(prefix)
                .and_then(|s| s.strip_suffix(".tmp"))
                .is_some_and(pair)
        })
}
