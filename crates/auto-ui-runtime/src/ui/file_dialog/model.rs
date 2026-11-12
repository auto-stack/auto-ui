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
}

enum Completion {
    Navigated(Navigation),
    Validated(Vec<FileChoice>),
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
    pub selected: bool,
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
    listing: Option<Listing>,
    selected: HashSet<String>,
    path_input: String,
    name_query: String,
    page: usize,
    error: Option<Failure>,
    events: Vec<DialogEvent>,
    home: String,
    roots: Vec<String>,
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
            listing: None,
            selected: HashSet::new(),
            path_input: String::new(),
            name_query: String::new(),
            page: 0,
            error: None,
            events: Vec::new(),
            home: String::new(),
            roots: Vec::new(),
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
        format!("{}:{}:{}", self.session, self.revision, path)
    }

    pub fn toggle(&mut self, row_id: &str) -> bool {
        if !self.active || self.pending.is_some() {
            return false;
        }
        let path = self
            .visible()
            .into_iter()
            .find(|e| !e.is_dir && self.row_id(&e.path) == row_id)
            .map(|e| e.path.clone());
        let Some(path) = path else {
            return false;
        };
        if !self.selected.remove(&path) {
            if self.config.selection_mode == SelectionMode::SingleFile {
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
        if !self.active || self.pending.is_some() || self.selected.is_empty() {
            return false;
        }
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
                self.revision += 1;
                self.page = 0;
                self.error = None;
                if let Some(path) = nav.preselect {
                    if let Some(index) = self
                        .visible()
                        .iter()
                        .position(|e| !e.is_dir && e.path == path)
                    {
                        self.page = index / PAGE_SIZE;
                        self.selected.insert(path);
                    }
                }
            }
            Ok(Completion::Validated(files)) => {
                if files.is_empty() {
                    self.fail(Failure::new("invalid_path", "没有可打开的文件", true));
                } else {
                    self.active = false;
                    self.events.push(DialogEvent::Accepted(files));
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
            can_accept: self.active && self.pending.is_none() && !self.selected.is_empty(),
            error: self.error.clone(),
        }
    }
}
