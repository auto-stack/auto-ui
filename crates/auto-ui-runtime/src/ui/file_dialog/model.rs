//! The selection state has one owner. The .at view projects this model; it does
//! not maintain a second selection/filter algorithm.
use super::{list_directory, utf8, Entry, Failure, Listing};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc,
};

const PAGE_SIZE: usize = 100;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SelectionMode {
    #[default]
    SingleFile,
    MultipleFiles,
    SingleDirectory,
    SaveFile,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Filter {
    pub id: String,
    pub label: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DialogConfig {
    pub initial_directory: String,
    pub selection_mode: SelectionMode,
    pub filters: Vec<Filter>,
    pub active_filter_id: String,
    pub show_hidden: bool,
    pub default_file_name: String,
    pub default_extension: String,
}

/// A validated save destination. `identity` fingerprints an existing regular
/// file so a confirmed overwrite can re-detect replacement; `Create` targets
/// carry no identity because the path does not exist yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveTarget {
    pub name: String,
    pub path: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveDecision {
    Create(SaveTarget),
    Overwrite(SaveTarget),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct FileChoice {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogEvent {
    Accepted(Vec<FileChoice>),
    Cancelled,
    Error(Failure),
}

pub struct Navigation {
    pub listing: Listing,
    pub preselect: Option<String>,
    pub home: String,
    pub roots: Vec<String>,
}

/// IO boundary. All methods execute on a worker and receive a cooperative
/// cancellation flag. Implementations never receive the model or VM.
pub trait FileSystem: Send + Sync + 'static {
    fn navigate(
        &self,
        input: &str,
        current: Option<&str>,
        cancel: &AtomicBool,
    ) -> Result<Navigation, Failure>;
    fn validate(
        &self,
        files: &[FileChoice],
        cancel: &AtomicBool,
    ) -> Result<Vec<FileChoice>, Failure>;
    /// PLAN-084: resolve a save destination without creating, truncating or
    /// renaming anything. Permission probing reads metadata only.
    fn validate_save(
        &self,
        directory: &str,
        file_name: &str,
        cancel: &AtomicBool,
    ) -> Result<SaveDecision, Failure>;
    /// PLAN-084: recheck a directory candidate before acceptance (existence,
    /// directory type, canonical identity, UTF-8 representability).
    fn validate_directory(
        &self,
        path: &str,
        cancel: &AtomicBool,
    ) -> Result<FileChoice, Failure>;
}

#[derive(Default)]
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn navigate(
        &self,
        input: &str,
        current: Option<&str>,
        cancel: &AtomicBool,
    ) -> Result<Navigation, Failure> {
        if input.is_empty() {
            return Err(Failure::new("invalid_path", "目录路径不能为空", true));
        }
        if input.contains('\0') {
            return Err(Failure::new("invalid_path", "路径包含空字符", true));
        }
        let path = Path::new(input);
        #[cfg(windows)]
        // A drive-relative C:foo or current-drive \foo is not relative to the
        // displayed folder and would depend on process ambient state.
        if !path.is_absolute()
            && (path.has_root()
                || matches!(
                    path.components().next(),
                    Some(std::path::Component::Prefix(_))
                ))
        {
            return Err(Failure::new(
                "invalid_path",
                "请提供完整盘符绝对路径或相对当前目录的路径",
                true,
            ));
        }
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            let base = current
                .map(PathBuf::from)
                .or_else(dirs::home_dir)
                .ok_or_else(|| Failure::new("not_found", "无法找到用户主目录", true))?;
            base.join(path)
        };
        let resolved = std::fs::canonicalize(&path).map_err(|e| Failure::io(&path, e))?;
        let metadata = std::fs::metadata(&resolved).map_err(|e| Failure::io(&resolved, e))?;
        let (directory, preselect) = if metadata.is_file() {
            (
                resolved
                    .parent()
                    .ok_or_else(|| Failure::new("invalid_path", "文件缺少父目录", true))?
                    .to_owned(),
                Some(utf8(&resolved)?),
            )
        } else if metadata.is_dir() {
            (resolved, None)
        } else {
            return Err(Failure::new("invalid_path", "只支持目录或普通文件", true));
        };
        let listing = list_directory(&directory, cancel)?;
        let home = dirs::home_dir()
            .map(|p| utf8(&p))
            .transpose()?
            .unwrap_or_default();
        let mut roots = Vec::new();
        #[cfg(windows)]
        {
            // Enumerate root names without probing every mapped network drive.
            // Reachability is checked only when the user navigates to a root.
            #[link(name = "kernel32")]
            extern "system" {
                fn GetLogicalDrives() -> u32;
            }
            let drives = unsafe { GetLogicalDrives() };
            for index in 0..26 {
                if drives & (1 << index) != 0 {
                    roots.push(format!("{}:\\", (b'A' + index as u8) as char));
                }
            }
        }
        #[cfg(not(windows))]
        roots.push("/".into());
        if !home.is_empty() && !roots.contains(&home) {
            roots.push(home.clone());
        }
        Ok(Navigation {
            listing,
            preselect,
            home,
            roots,
        })
    }

    fn validate(
        &self,
        files: &[FileChoice],
        cancel: &AtomicBool,
    ) -> Result<Vec<FileChoice>, Failure> {
        let mut unique = HashSet::new();
        let mut out = Vec::new();
        let mut failures = Vec::new();
        for file in files {
            if cancel.load(Ordering::Acquire) {
                return Err(Failure::new("cancelled", "操作已取消", false));
            }
            let path = Path::new(&file.path);
            let checked = (|| {
                let canonical = std::fs::canonicalize(path).map_err(|e| Failure::io(path, e))?;
                let metadata = std::fs::metadata(&canonical).map_err(|e| Failure::io(path, e))?;
                if !metadata.is_file() {
                    return Err(Failure::new(
                        "invalid_path",
                        format!("{} 不是普通文件", file.name),
                        true,
                    ));
                }
                std::fs::File::open(path).map_err(|e| Failure::io(path, e))?;
                utf8(&canonical)
            })();
            match checked {
                Ok(identity) => {
                    if unique.insert(identity) {
                        out.push(file.clone());
                    }
                }
                Err(error) => failures.push(error),
            }
        }
        if !failures.is_empty() {
            let code = failures[0].code.clone();
            return Err(Failure::new(
                &code,
                failures
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join("\n"),
                true,
            ));
        }
        Ok(out)
    }

    fn validate_save(
        &self,
        directory: &str,
        file_name: &str,
        cancel: &AtomicBool,
    ) -> Result<SaveDecision, Failure> {
        if cancel.load(Ordering::Acquire) {
            return Err(Failure::new("cancelled", "操作已取消", false));
        }
        let parent = std::fs::canonicalize(directory).map_err(|e| Failure::io(Path::new(directory), e))?;
        if !std::fs::metadata(&parent)
            .map_err(|e| Failure::io(&parent, e))?
            .is_dir()
        {
            return Err(Failure::new("invalid_path", "保存目录不存在或不是目录", true));
        }
        let target = parent.join(file_name);
        let decision = match std::fs::symlink_metadata(&target) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => SaveDecision::Create(SaveTarget {
                name: file_name.to_owned(),
                path: utf8(&target)?,
                identity: String::new(),
            }),
            Err(error) => return Err(Failure::io(&target, error)),
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(Failure::new(
                        "unsupported_path",
                        format!("保存目标为符号链接: {}", target.display()),
                        false,
                    ));
                }
                if metadata.is_dir() {
                    return Err(Failure::new(
                        "invalid_path",
                        format!("保存目标已是目录: {}", target.display()),
                        true,
                    ));
                }
                if !metadata.is_file() {
                    return Err(Failure::new(
                        "unsupported_path",
                        format!("保存目标是特殊文件: {}", target.display()),
                        false,
                    ));
                }
                save_writable_probe(&metadata, &target)?;
                SaveDecision::Overwrite(SaveTarget {
                    name: file_name.to_owned(),
                    path: utf8(&target)?,
                    identity: save_identity(&metadata),
                })
            }
        };
        Ok(decision)
    }

    fn validate_directory(
        &self,
        path: &str,
        cancel: &AtomicBool,
    ) -> Result<FileChoice, Failure> {
        if cancel.load(Ordering::Acquire) {
            return Err(Failure::new("cancelled", "操作已取消", false));
        }
        let canonical = std::fs::canonicalize(path).map_err(|e| Failure::io(Path::new(path), e))?;
        let metadata = std::fs::metadata(&canonical).map_err(|e| Failure::io(&canonical, e))?;
        if !metadata.is_dir() {
            return Err(Failure::new(
                "invalid_path",
                format!("{} 不是目录", canonical.display()),
                true,
            ));
        }
        let canonical_path = utf8(&canonical)?;
        let name = Path::new(&canonical_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| canonical_path.clone());
        Ok(FileChoice {
            name,
            path: canonical_path,
        })
    }
}

/// Read-only attributes only; the selector never creates a probe file. The
/// final write permission and write result stay with the consumer app.
fn save_writable_probe(metadata: &std::fs::Metadata, target: &Path) -> Result<(), Failure> {
    let readonly = {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 1 != 0
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o200 == 0
        }
    };
    if readonly {
        return Err(Failure::new(
            "permission_denied",
            format!("保存目标为只读: {}", target.display()),
            true,
        ));
    }
    Ok(())
}

/// Length + nanosecond modification time fingerprint. A replaced or renamed
/// target between confirmation rounds changes this identity, so a confirmed
/// overwrite cannot silently hit different bytes.
fn save_identity(metadata: &std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("len:{}:mtime:{}", metadata.len(), modified)
}

/// Windows reserved device names, case-insensitive, with or without extension.
#[cfg(windows)]
fn save_name_is_windows_reserved(raw: &str) -> bool {
    let stem = raw.split('.').next().unwrap_or("");
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6"
            | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6"
            | "LPT7" | "LPT8" | "LPT9"
    )
}

enum Completion {
    Navigated(Navigation),
    Validated(Vec<FileChoice>),
    ValidatedSave(SaveDecision),
    ValidatedDirectory(FileChoice),
}
struct Worker {
    cancelled: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<Completion, Failure>>,
}
impl Worker {
    fn spawn(
        job: impl FnOnce(&AtomicBool) -> Result<Completion, Failure> + Send + 'static,
    ) -> Result<Self, Failure> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_flag = cancelled.clone();
        std::thread::Builder::new()
            .name("file-dialog-io".into())
            .spawn(move || {
                let result = job(&worker_flag);
                if !worker_flag.load(Ordering::Acquire) {
                    let _ = sender.send(result);
                }
            })
            .map_err(|e| Failure::io(Path::new("FileDialog worker"), e))?;
        Ok(Self {
            cancelled,
            receiver,
        })
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub size_label: String,
    pub selected: bool,
}

/// PLAN-084: the visible overwrite-confirmation payload. The confirmed path
/// is shown verbatim; identity stays model-side and never crosses the view.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverwriteView {
    pub name: String,
    pub path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Projection {
    pub session: u64,
    pub active: bool,
    pub loading: bool,
    pub current_directory: String,
    pub path_input: String,
    pub parent: Option<String>,
    pub home: String,
    pub roots: Vec<String>,
    pub name_query: String,
    pub active_filter_id: String,
    pub show_hidden: bool,
    pub entries: Vec<Row>,
    pub total_entries: usize,
    pub visible_entries: usize,
    pub visible_file_count: usize,
    pub page: usize,
    pub page_count: usize,
    pub selected_count: usize,
    pub selected_paths: Vec<String>,
    pub can_accept: bool,
    /// PLAN-084: frozen session mode (one of the four selectionMode values).
    pub mode: SelectionMode,
    /// PLAN-084 save mode: the editable filename draft.
    pub save_name: String,
    /// PLAN-084: present while an overwrite confirmation owns the session.
    pub overwrite: Option<OverwriteView>,
    pub error: Option<Failure>,
}

pub struct DialogModel {
    fs: Arc<dyn FileSystem>,
    config: DialogConfig,
    session: u64,
    revision: u64,
    active: bool,
    pending: Option<Worker>,
    validating: bool,
    /// PLAN-084: the in-flight validation is a save-target resolution whose
    /// directory/name inputs a draft edit may supersede.
    validating_save: bool,
    listing: Option<Listing>,
    selected: HashSet<String>,
    path_input: String,
    name_query: String,
    page: usize,
    error: Option<Failure>,
    events: Vec<DialogEvent>,
    home: String,
    roots: Vec<String>,
    /// PLAN-084 save-mode filename draft. Never trimmed or rewritten; the
    /// completed extension only exists in the accepted result.
    save_name: String,
    /// Pending overwrite confirmation. Identity binds the confirmed bytes;
    /// any session input change clears it.
    overwrite: Option<SaveTarget>,
    /// The first navigation of a save session may prefill the basename of an
    /// initial complete file path; later navigations never rewrite user input.
    save_prefilled: bool,
}

impl Default for DialogModel {
    fn default() -> Self {
        Self::with_fs(Arc::new(LocalFileSystem))
    }
}

impl DialogModel {
    pub fn with_fs(fs: Arc<dyn FileSystem>) -> Self {
        Self {
            fs,
            config: Default::default(),
            session: 0,
            revision: 0,
            active: false,
            pending: None,
            validating: false,
            validating_save: false,
            listing: None,
            selected: HashSet::new(),
            path_input: String::new(),
            name_query: String::new(),
            page: 0,
            error: None,
            events: Vec::new(),
            home: String::new(),
            roots: Vec::new(),
            save_name: String::new(),
            overwrite: None,
            save_prefilled: false,
        }
    }

    pub fn open(&mut self, mut config: DialogConfig) -> bool {
        if self.active {
            return false;
        }
        for filter in &mut config.filters {
            filter.extensions = filter
                .extensions
                .iter()
                .map(|e| e.trim().trim_start_matches('.').to_lowercase())
                .filter(|e| !e.is_empty())
                .collect();
        }
        if !config
            .filters
            .iter()
            .any(|f| f.id == config.active_filter_id)
        {
            config.active_filter_id = config
                .filters
                .first()
                .map(|f| f.id.clone())
                .unwrap_or_default();
        }
        self.config = config;
        self.session = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
        self.revision += 1;
        self.active = true;
        self.pending = None;
        self.validating = false;
        self.validating_save = false;
        self.home = dirs::home_dir()
            .and_then(|p| p.to_str().map(str::to_owned))
            .unwrap_or_default();
        self.roots = if self.home.is_empty() {
            Vec::new()
        } else {
            vec![self.home.clone()]
        };
        self.listing = None;
        self.selected.clear();
        self.name_query.clear();
        self.page = 0;
        self.error = None;
        self.events.clear();
        // PLAN-084: mode props are frozen into this session's config; the
        // save draft starts from the explicit default only. An initial
        // complete file path prefills its basename after the first navigation
        // confirms it is a file (worker result), never before.
        self.save_name = if self.config.selection_mode == SelectionMode::SaveFile {
            self.config.default_file_name.clone()
        } else {
            String::new()
        };
        self.overwrite = None;
        self.save_prefilled = false;
        let initial = if self.config.initial_directory.is_empty() {
            dirs::home_dir()
                .and_then(|p| p.to_str().map(str::to_owned))
                .unwrap_or_else(|| "/".into())
        } else {
            self.config.initial_directory.clone()
        };
        self.navigate(initial);
        true
    }

    pub fn set_path_input(&mut self, input: String) {
        if self.active {
            self.path_input = input;
        }
    }

    pub fn navigate(&mut self, input: String) {
        if !self.active {
            return;
        }
        self.path_input = input.clone();
        self.validating = false;
        self.validating_save = false;
        // A directory change invalidates a pending overwrite confirmation.
        self.overwrite = None;
        // Replacing the receiver invalidates even already-completed old work.
        self.pending = None;
        let fs = self.fs.clone();
        let current = self.listing.as_ref().map(|l| l.directory.clone());
        match Worker::spawn(move |flag| {
            fs.navigate(&input, current.as_deref(), flag)
                .map(Completion::Navigated)
        }) {
            Ok(worker) => self.pending = Some(worker),
            Err(error) => self.fail(error),
        }
    }

    pub fn up(&mut self) {
        if let Some(parent) = self.listing.as_ref().and_then(|l| l.parent.clone()) {
            self.navigate(parent);
        }
    }

    pub fn refresh(&mut self) {
        if let Some(directory) = self.listing.as_ref().map(|l| l.directory.clone()) {
            self.navigate(directory);
        } else {
            self.navigate(self.path_input.clone());
        }
    }

    fn matches_file(&self, entry: &Entry) -> bool {
        if entry.is_dir {
            return true;
        }
        let name = entry.name.to_lowercase();
        let extension = Path::new(&entry.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let type_matches = self
            .config
            .filters
            .iter()
            .find(|f| f.id == self.config.active_filter_id)
            .map(|f| {
                f.extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(extension))
            })
            .unwrap_or(true);
        type_matches && name.contains(&self.name_query.to_lowercase())
    }

    fn visible(&self) -> Vec<&Entry> {
        self.listing
            .as_ref()
            .map(|l| {
                l.entries
                    .iter()
                    .filter(|e| (self.config.show_hidden || !e.hidden) && self.matches_file(e))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn invalidate_selection(&mut self) {
        // A validation in flight used the previous visible set. Never publish
        // its result after a filter/selection change.
        if self.validating {
            self.pending = None;
            self.validating = false;
        }
        self.selected.clear();
        // Listing-state changes invalidate a pending overwrite confirmation.
        self.overwrite = None;
        self.page = 0;
        self.revision += 1;
        self.error = None;
    }

    pub fn set_name_query(&mut self, query: String) {
        if self.active && self.name_query != query {
            self.name_query = query;
            self.invalidate_selection();
        }
    }

    pub fn set_filter(&mut self, id: &str) {
        if !self.active || self.config.active_filter_id == id {
            return;
        }
        if !self.config.filters.iter().any(|f| f.id == id) {
            return;
        }
        self.config.active_filter_id = id.into();
        self.invalidate_selection();
    }

    pub fn set_show_hidden(&mut self, show: bool) {
        if self.active && self.config.show_hidden != show {
            self.config.show_hidden = show;
            self.invalidate_selection();
        }
    }

    pub fn clear_selection(&mut self) {
        if !self.active {
            return;
        }
        self.selected.clear();
        // Validation is an IO transaction, so changing selection cancels it.
        if self.validating {
            self.pending = None;
            self.validating = false;
        }
    }

    fn row_id(&self, path: &str) -> String {
        // Tokens cross the shared typed-message framing. Keep raw filesystem
        // control characters out of that framing, without changing the path.
        format!("{}:{}:{}", self.session, self.revision, serde_json::to_string(path).expect("UTF-8 path JSON encoding"))
    }

    pub fn toggle(&mut self, row_id: &str) -> bool {
        if !self.active || self.pending.is_some() {
            return false;
        }
        // PLAN-084: while an overwrite confirmation owns the session, listing
        // interactions are rejected at the model level in addition to the
        // modal scope narrowing. Save mode has no selection at all.
        if self.overwrite.is_some() || self.config.selection_mode == SelectionMode::SaveFile {
            return false;
        }
        let want_dir = self.config.selection_mode == SelectionMode::SingleDirectory;
        let path = self
            .visible()
            .into_iter()
            .find(|e| e.is_dir == want_dir && self.row_id(&e.path) == row_id)
            .map(|e| e.path.clone());
        let Some(path) = path else {
            return false;
        };
        if !self.selected.remove(&path) {
            if matches!(
                self.config.selection_mode,
                SelectionMode::SingleFile | SelectionMode::SingleDirectory
            ) {
                self.selected.clear();
            }
            self.selected.insert(path);
        }
        self.error = None;
        true
    }

    pub fn enter(&mut self, row_id: &str) -> bool {
        if !self.active {
            return false;
        }
        // An open overwrite confirmation owns the session; navigation must go
        // through 返回修改 first.
        if self.overwrite.is_some() {
            return false;
        }
        let path = self
            .visible()
            .into_iter()
            .find(|e| e.is_dir && self.row_id(&e.path) == row_id)
            .map(|e| e.path.clone());
        if let Some(path) = path {
            self.navigate(path);
            true
        } else {
            false
        }
    }

    pub fn set_page(&mut self, page: usize) {
        self.page = page.min(self.visible().len().saturating_sub(1) / PAGE_SIZE);
    }

    pub fn confirm(&mut self) -> bool {
        if !self.active || self.pending.is_some() {
            return false;
        }
        // A pending overwrite confirmation is settled only through
        // confirm_overwrite; a stale footer confirm must not accept a
        // changed target.
        if self.overwrite.is_some() {
            return false;
        }
        match self.config.selection_mode {
            SelectionMode::SingleFile | SelectionMode::MultipleFiles => self.confirm_files(),
            SelectionMode::SingleDirectory => self.confirm_directory(),
            SelectionMode::SaveFile => self.confirm_save(),
        }
    }

    fn confirm_files(&mut self) -> bool {
        let files = self
            .visible()
            .into_iter()
            .filter(|e| !e.is_dir && self.selected.contains(&e.path))
            .map(|e| FileChoice {
                name: e.name.clone(),
                path: e.path.clone(),
            })
            .collect::<Vec<_>>();
        if files.is_empty() {
            return false;
        }
        let fs = self.fs.clone();
        self.validating = true;
        self.validating_save = false;
        match Worker::spawn(move |flag| fs.validate(&files, flag).map(Completion::Validated)) {
            Ok(worker) => self.pending = Some(worker),
            Err(error) => {
                self.validating = false;
                self.fail(error);
                return false;
            }
        }
        true
    }

    fn confirm_directory(&mut self) -> bool {
        let Some(path) = self.selected.iter().next().cloned() else {
            return false;
        };
        let fs = self.fs.clone();
        self.validating = true;
        self.validating_save = false;
        match Worker::spawn(move |flag| {
            fs.validate_directory(&path, flag).map(Completion::ValidatedDirectory)
        }) {
            Ok(worker) => self.pending = Some(worker),
            Err(error) => {
                self.validating = false;
                self.fail(error);
                return false;
            }
        }
        true
    }

    /// PLAN-084 save-mode confirm: resolve the destination on a worker.
    /// Structural name/extension checks run here (pure logic); the existing
    /// target and its writability are checked on the worker.
    fn confirm_save(&mut self) -> bool {
        let Some(directory) = self.listing.as_ref().map(|l| l.directory.clone()) else {
            return false;
        };
        let raw = self.save_name.clone();
        if raw.is_empty() {
            return false;
        }
        let final_name = match self.resolve_save_name(&raw) {
            Ok(name) => name,
            Err(error) => {
                self.fail(error);
                return true;
            }
        };
        let fs = self.fs.clone();
        self.validating = true;
        self.validating_save = true;
        match Worker::spawn(move |flag| {
            fs.validate_save(&directory, &final_name, flag)
                .map(Completion::ValidatedSave)
        }) {
            Ok(worker) => self.pending = Some(worker),
            Err(error) => {
                self.validating = false;
                self.validating_save = false;
                self.fail(error);
                return false;
            }
        }
        true
    }

    /// Filename constraints (basename only, no separators/dot entries/NUL)
    /// and extension completion/mismatch rules. The draft input is never
    /// rewritten; completion exists only in the returned name.
    fn resolve_save_name(&self, raw: &str) -> Result<String, Failure> {
        if raw.contains('\0') {
            return Err(Failure::new("invalid_path", "文件名包含空字符", true));
        }
        if raw.contains('/') || raw.contains('\\') {
            return Err(Failure::new(
                "invalid_path",
                "文件名不能包含路径分隔符",
                true,
            ));
        }
        if raw == "." || raw == ".." {
            return Err(Failure::new("invalid_path", "文件名不能是 . 或 ..", true));
        }
        #[cfg(windows)]
        if save_name_is_windows_reserved(raw) {
            return Err(Failure::new(
                "invalid_path",
                format!("文件名是 Windows 保留设备名: {raw}"),
                true,
            ));
        }
        let explicit_extension = Path::new(raw)
            .extension()
            .and_then(|e| e.to_str())
            .filter(|e| !e.is_empty())
            .map(str::to_owned);
        let default_extension = self.config.default_extension.trim().to_lowercase();
        let active_group_extensions = self
            .config
            .filters
            .iter()
            .find(|f| f.id == self.config.active_filter_id)
            .map(|f| f.extensions.clone())
            .unwrap_or_default();
        let group_allows = |extension: &str| {
            active_group_extensions.is_empty()
                || active_group_extensions
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(extension))
        };
        match explicit_extension {
            Some(extension) => {
                if !group_allows(&extension) {
                    return Err(Failure::new(
                        "invalid_path",
                        format!("扩展名 .{extension} 与当前类型组不匹配"),
                        true,
                    ));
                }
                Ok(raw.to_owned())
            }
            None if !default_extension.is_empty() => {
                if !group_allows(&default_extension) {
                    return Err(Failure::new(
                        "invalid_path",
                        format!(
                            "defaultExtension .{} 与当前类型组不兼容",
                            self.config.default_extension
                        ),
                        false,
                    ));
                }
                Ok(format!("{raw}.{default_extension}"))
            }
            None => Ok(raw.to_owned()),
        }
    }

    /// PLAN-084: the filename draft setter. Typing invalidates a pending
    /// overwrite confirmation and an in-flight save validation (the old
    /// confirm must never accept a new name).
    pub fn set_save_name(&mut self, value: String) {
        if !self.active {
            return;
        }
        if self.validating_save {
            self.pending = None;
            self.validating = false;
            self.validating_save = false;
        }
        if self.save_name != value {
            self.overwrite = None;
        }
        self.save_name = value;
    }

    /// PLAN-084 directory mode: select the currently listed directory (root
    /// included). Selected rows and this share one selection slot.
    pub fn select_current(&mut self) -> bool {
        if !self.active || self.overwrite.is_some() || self.pending.is_some() {
            return false;
        }
        let Some(directory) = self.listing.as_ref().map(|l| l.directory.clone()) else {
            return false;
        };
        self.selected.clear();
        self.selected.insert(directory);
        self.error = None;
        true
    }

    /// PLAN-084: 确认覆盖 — revalidate the same session target. Same
    /// identity → accept; target vanished → accept (create); replaced or
    /// retyped identity → re-present the layer for the new target; changed
    /// kind → named error. Never accepts a target other than the one the
    /// user confirmed.
    pub fn confirm_overwrite(&mut self) -> bool {
        if !self.active || self.pending.is_some() || self.overwrite.is_none() {
            return false;
        }
        let Some(directory) = self.listing.as_ref().map(|l| l.directory.clone()) else {
            return false;
        };
        let raw = self.save_name.clone();
        if raw.is_empty() {
            return false;
        }
        let final_name = match self.resolve_save_name(&raw) {
            Ok(name) => name,
            Err(error) => {
                self.overwrite = None;
                self.fail(error);
                return true;
            }
        };
        let fs = self.fs.clone();
        self.validating = true;
        self.validating_save = true;
        match Worker::spawn(move |flag| {
            fs.validate_save(&directory, &final_name, flag)
                .map(Completion::ValidatedSave)
        }) {
            Ok(worker) => self.pending = Some(worker),
            Err(error) => {
                self.validating = false;
                self.validating_save = false;
                self.fail(error);
                return false;
            }
        }
        true
    }

    /// PLAN-084: 返回修改 — keep directory/name drafts and continue the
    /// session.
    pub fn overwrite_back(&mut self) -> bool {
        if !self.active || self.overwrite.is_none() {
            return false;
        }
        self.overwrite = None;
        true
    }

    /// PLAN-084: Esc / explicit close path. An open overwrite confirmation
    /// returns to the selector; otherwise the session cancels.
    pub fn request_close(&mut self) -> bool {
        if self.overwrite.is_some() {
            self.overwrite = None;
            return true;
        }
        self.cancel()
    }

    fn fail(&mut self, error: Failure) {
        self.error = Some(error.clone());
        self.events.push(DialogEvent::Error(error));
    }

    pub fn poll(&mut self) -> bool {
        let Some(worker) = &self.pending else {
            return false;
        };
        let result = match worker.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Failure::new("io_error", "目录任务意外结束，请重试", true))
            }
        };
        self.pending = None;
        self.validating = false;
        self.validating_save = false;
        if !self.active {
            return false;
        }
        match result {
            Ok(Completion::Navigated(nav)) => {
                self.listing = Some(nav.listing);
                self.home = nav.home;
                self.roots = nav.roots;
                self.path_input = self.listing.as_ref().unwrap().directory.clone();
                self.selected.clear();
                self.overwrite = None;
                self.revision += 1;
                self.page = 0;
                self.error = None;
                let preselect = nav.preselect;
                if let Some(path) = preselect {
                    if self.config.selection_mode == SelectionMode::SaveFile {
                        // PLAN-084: an initial complete file path prefills its
                        // basename once per session and only into an empty
                        // draft; later navigations never rewrite user input.
                        if !self.save_prefilled && self.save_name.is_empty() {
                            if let Some(name) = Path::new(&path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(str::to_owned)
                            {
                                self.save_name = name;
                            }
                        }
                    } else if let Some(index) = self
                        .visible()
                        .iter()
                        .position(|e| !e.is_dir && e.path == path)
                    {
                        self.page = index / PAGE_SIZE;
                        self.selected.insert(path);
                    }
                }
                self.save_prefilled = true;
            }
            Ok(Completion::Validated(files)) => {
                if files.is_empty() {
                    self.fail(Failure::new("invalid_path", "没有可打开的文件", true));
                } else {
                    self.active = false;
                    self.events.push(DialogEvent::Accepted(files));
                }
            }
            Ok(Completion::ValidatedDirectory(choice)) => {
                self.active = false;
                self.events.push(DialogEvent::Accepted(vec![choice]));
            }
            Ok(Completion::ValidatedSave(decision)) => {
                let confirmed = self.overwrite.take();
                match decision {
                    SaveDecision::Create(target) => {
                        self.active = false;
                        self.events.push(DialogEvent::Accepted(vec![FileChoice {
                            name: target.name,
                            path: target.path,
                        }]));
                    }
                    SaveDecision::Overwrite(target) => match confirmed {
                        Some(previous) if previous.identity == target.identity => {
                            // Same bytes the user confirmed: accept.
                            self.active = false;
                            self.events.push(DialogEvent::Accepted(vec![FileChoice {
                                name: target.name,
                                path: target.path,
                            }]));
                        }
                        Some(_) => {
                            // Replaced/retyped target: the old confirmation is
                            // void; re-present the layer for the new identity.
                            self.overwrite = Some(target);
                        }
                        None => {
                            // First validation found an existing file.
                            self.overwrite = Some(target);
                        }
                    },
                }
            }
            Err(error) => self.fail(error),
        }
        true
    }

    pub fn cancel(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.active = false;
        self.pending = None;
        self.validating = false;
        self.validating_save = false;
        self.overwrite = None;
        self.events.push(DialogEvent::Cancelled);
        true
    }

    pub fn take_events(&mut self) -> Vec<DialogEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn projection(&self) -> Projection {
        let visible = self.visible();
        let page_count = visible.len().div_ceil(PAGE_SIZE).max(1);
        let page = self.page.min(page_count - 1);
        Projection {
            session: self.session,
            active: self.active,
            loading: self.pending.is_some(),
            current_directory: self
                .listing
                .as_ref()
                .map(|l| l.directory.clone())
                .unwrap_or_default(),
            path_input: self.path_input.clone(),
            parent: self.listing.as_ref().and_then(|l| l.parent.clone()),
            home: self.home.clone(),
            roots: self.roots.clone(),
            name_query: self.name_query.clone(),
            active_filter_id: self.config.active_filter_id.clone(),
            show_hidden: self.config.show_hidden,
            entries: visible
                .iter()
                .skip(page * PAGE_SIZE)
                .take(PAGE_SIZE)
                .map(|e| Row {
                    id: self.row_id(&e.path),
                    name: e.name.clone(),
                    path: e.path.clone(),
                    is_dir: e.is_dir,
                    size: e.size,
                    size_label: if e.size < 1024 {
                        format!("{} B", e.size)
                    } else if e.size < 1024 * 1024 {
                        format!("{} KB", e.size / 1024)
                    } else {
                        format!("{} MB", e.size / (1024 * 1024))
                    },
                    selected: self.selected.contains(&e.path),
                })
                .collect(),
            total_entries: self.listing.as_ref().map(|l| l.entries.len()).unwrap_or(0),
            visible_entries: visible.len(),
            visible_file_count: visible.iter().filter(|e| !e.is_dir).count(),
            page,
            page_count,
            selected_count: self.selected.len(),
            selected_paths: visible
                .iter()
                .filter(|e| self.selected.contains(&e.path))
                .map(|e| e.path.clone())
                .collect(),
            can_accept: match self.config.selection_mode {
                SelectionMode::SaveFile => {
                    self.active
                        && self.pending.is_none()
                        && self.listing.is_some()
                        && !self.save_name.is_empty()
                }
                SelectionMode::SingleDirectory => {
                    self.active && self.pending.is_none() && !self.selected.is_empty()
                }
                _ => self.active && self.pending.is_none() && !self.selected.is_empty(),
            },
            mode: self.config.selection_mode,
            save_name: self.save_name.clone(),
            overwrite: self.overwrite.as_ref().map(|target| OverwriteView {
                name: target.name.clone(),
                path: target.path.clone(),
            }),
            error: self.error.clone(),
        }
    }
}


#[cfg(test)]
mod message_identity_tests {
    use super::*;
    #[test]
    fn file_dialog_row_token_roundtrips_without_aliasing_another_path() {
        let model = DialogModel::default();
        let id = model.row_id("/directory/prefix\u{1f}suffix.txt");
        let other = model.row_id("/directory/prefix");
        let wire = crate::ui::mcp_types::encode_payload("Choose", &[auto_val::Value::str(id.clone())]);
        let (event, args) = crate::ui::dynamic::decode_payload(&wire);
        assert_eq!(event, "Choose");
        assert_eq!(args.len(), 1);
        assert_ne!(args[0], auto_val::Value::str(other), "a separator in one filename must not select a different file");
        assert_eq!(args[0], auto_val::Value::str(id), "the real shared message codec must preserve the whole row identity");
    }
}
