//! App-owned, non-blocking local directory IO for the FileDialog component.
//!
//! Workers own paths and cancellation flags, never VM state. Results carry the
//! request generation; the owner alone decides whether a result is still live.

use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

mod model;
pub use model::{
    DialogConfig, DialogEvent, DialogModel, FileChoice, FileSystem, Filter, LocalFileSystem,
    Navigation, Projection, Row, SelectionMode,
};

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub hidden: bool,
    pub size: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Failure {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Failure {
    pub fn new(code: &str, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }
    fn io(path: &Path, error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        Self {
            code: match error.kind() {
                ErrorKind::NotFound => "not_found",
                ErrorKind::PermissionDenied => "permission_denied",
                ErrorKind::InvalidInput => "invalid_path",
                _ => "io_error",
            }
            .into(),
            message: format!("{}: {error}", path.display()),
            retryable: true,
        }
    }

    fn unsupported(path: &Path) -> Self {
        Self {
            code: "unsupported_path".into(),
            message: format!("Path is not UTF-8: {}", path.display()),
            retryable: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Listing {
    pub directory: String,
    pub parent: Option<String>,
    pub entries: Vec<Entry>,
}

fn utf8(path: &Path) -> Result<String, Failure> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Failure::unsupported(path))
}

/// Local IO only; callers must run this on a worker, never on a render thread.
pub fn list_directory(path: &Path, cancelled: &AtomicBool) -> Result<Listing, Failure> {
    let directory = std::fs::canonicalize(path).map_err(|e| Failure::io(path, e))?;
    let mut entries = Vec::new();
    for item in std::fs::read_dir(&directory).map_err(|e| Failure::io(&directory, e))? {
        if cancelled.load(Ordering::Acquire) {
            break;
        }
        let item = item.map_err(|e| Failure::io(&directory, e))?;
        let path = item.path();
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| Failure::unsupported(&path))?;
        let metadata = std::fs::metadata(&path).map_err(|e| Failure::io(&path, e))?;
        #[cfg(windows)]
        let hidden = {
            use std::os::windows::fs::MetadataExt;
            name.starts_with('.') || metadata.file_attributes() & (2 | 4) != 0
        };
        #[cfg(not(windows))]
        let hidden = name.starts_with('.');
        // Special devices are not selectable file entries.
        if metadata.is_file() || metadata.is_dir() {
            entries.push(Entry {
                name,
                path: utf8(&path)?,
                is_dir: metadata.is_dir(),
                hidden,
                size: metadata.len(),
            });
        }
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(Listing {
        directory: utf8(&directory)?,
        parent: directory.parent().map(utf8).transpose()?,
        entries,
    })
}

/// At most one live request per component. Superseded workers cannot publish
/// into the current receiver, including when the owner closes and reopens.
#[derive(Default)]
pub struct DirectoryRequest {
    pending: Option<(Arc<AtomicBool>, mpsc::Receiver<Result<Listing, Failure>>)>,
}

impl DirectoryRequest {
    pub fn start(&mut self, path: PathBuf) -> Result<(), Failure> {
        self.cancel();
        let (tx, rx) = mpsc::sync_channel(1);
        let flag = Arc::new(AtomicBool::new(false));
        let worker_flag = flag.clone();
        std::thread::Builder::new()
            .name("file-dialog-list".into())
            .spawn(move || {
                let result = list_directory(&path, &worker_flag);
                if !worker_flag.load(Ordering::Acquire) {
                    let _ = tx.send(result);
                }
            })
            .map_err(|e| Failure::io(Path::new("directory worker"), e))?;
        self.pending = Some((flag, rx));
        Ok(())
    }

    pub fn poll(&mut self) -> Option<Result<Listing, Failure>> {
        let (_, rx) = self.pending.as_ref()?;
        match rx.try_recv() {
            Ok(result) => {
                self.pending = None;
                Some(result)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                Some(Err(Failure {
                    code: "io_error".into(),
                    message: "Directory worker disconnected".into(),
                    retryable: true,
                }))
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Some((flag, _)) = self.pending.take() {
            flag.store(true, Ordering::Release);
        }
    }
}

impl Drop for DirectoryRequest {
    fn drop(&mut self) {
        self.cancel();
    }
}

mod component_io;
pub(crate) use component_io::ComponentIo;
