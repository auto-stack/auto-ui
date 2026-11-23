use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use auto_lang_core::vm::ffi::{
    async_http::{cancel_live_op, register_live_op, submit_client_job, take_live_op},
    stdlib::{alloc_async_id, resolve_http_base_url, AsyncResult},
};
use auto_media_core::image_pipeline::{
    global_media_registry, global_media_worker_pool, media_surface_status, media_uri,
    MediaAssetError, MediaAssetKey, MediaAssetTicket, MediaMetadata, MediaOrientation,
    MediaPriority, MediaSurfaceStatus, RenditionSpec, MAX_IMAGE_FILE_BYTES,
};
use reqwest::Url;

use crate::ui::ace_package_bridge::PackageQueuedAction;

use super::widgets::PublicImages;

// Bound downloaded images retained while the shared decoder is busy.
const MAX_HTTP_READS: usize = 4;

#[derive(Debug)]
pub(super) struct PackageImages {
    public: Option<PublicImages>,
    backend: Option<Url>,
    surfaces: Mutex<BTreeMap<(usize, Vec<usize>), HttpSurface>>,
}

impl PackageImages {
    pub(super) fn new(public_root: Option<Arc<PathBuf>>) -> Self {
        let backend = Url::parse(&resolve_http_base_url("/")).ok().filter(is_http);
        Self {
            public: public_root.map(PublicImages::new),
            backend,
            surfaces: Mutex::default(),
        }
    }

    pub(super) fn http_url(&self, source: &str) -> Option<Url> {
        let backend = self.backend.as_ref()?;
        let url = if source.starts_with("/api/") {
            backend.join(source.trim_start_matches('/')).ok()?
        } else {
            Url::parse(source).ok()?
        };
        (is_http(&url) && backend.origin() == url.origin()).then_some(url)
    }

    pub(super) fn public_source(&self, source: &str) -> Option<String> {
        self.public.as_ref()?.surface_source(source)
    }

    pub(super) fn project_http(
        &self,
        instance: usize,
        path: &[usize],
        url: Url,
        identity: &str,
        callbacks: SurfaceCallbacks,
    ) -> String {
        let mut surfaces = self.surfaces.lock().unwrap();
        let surface = surfaces
            .entry((instance, path.to_vec()))
            .or_insert_with(|| HttpSurface::request(url.clone(), identity));
        if surface.url.as_ref() != Some(&url) || surface.identity != identity {
            *surface = HttpSurface::request(url, identity);
        }
        surface.callbacks = callbacks;
        surface.ready_projected = matches!(
            media_surface_status(&surface.source),
            MediaSurfaceStatus::Ready
        );
        surface.source.clone()
    }

    pub(super) fn project_private(
        &self,
        instance: usize,
        path: &[usize],
        source: &str,
        callbacks: SurfaceCallbacks,
    ) {
        let mut surfaces = self.surfaces.lock().unwrap();
        let surface = surfaces.entry((instance, path.to_vec()))
            .or_insert_with(|| HttpSurface::private(source));
        if surface.source != source {
            *surface = HttpSurface::private(source);
        }
        surface.callbacks = callbacks;
        surface.ready_projected = matches!(
            crate::back_proxy::native_files::media_status(source), MediaSurfaceStatus::Ready
        );
    }

    pub(super) fn finish_frame(&self, instance: usize, revision: u64) {
        self.surfaces.lock().unwrap().retain(|(root, _), surface| {
            *root != instance || surface.callbacks.revision == revision
        });
    }

    pub(super) fn next_event(
        &self,
        resolve: impl Fn(usize, &PackageQueuedAction) -> Option<(u64, usize)>,
    ) -> Option<(usize, u64, usize)> {
        let mut surfaces = self.surfaces.lock().unwrap();
        let mut reads = surfaces
            .values()
            .filter(|surface| {
                matches!(surface.download, Download::Active(_) | Download::Encoded(_))
            })
            .count();
        for ((instance, _), surface) in surfaces.iter_mut() {
            if matches!(surface.download, Download::Queued(_)) {
                if reads >= MAX_HTTP_READS {
                    continue;
                }
                reads += 1;
            }
            surface.poll_request();
            let Some(notice) = surface.notification() else {
                continue;
            };
            let Some((revision, target)) = resolve(*instance, notice) else {
                continue;
            };
            surface.notified = true;
            return Some((*instance, revision, target));
        }
        None
    }
}

fn is_http(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.username().is_empty()
        && url.password().is_none()
}

#[derive(Debug, Default)]
pub(super) struct SurfaceCallbacks {
    pub(super) revision: u64,
    pub(super) loaded: Option<PackageQueuedAction>,
    pub(super) error: Option<PackageQueuedAction>,
}

#[derive(Debug)]
struct HttpSurface {
    url: Option<Url>,
    identity: String,
    source: String,
    ticket: Option<MediaAssetTicket>,
    download: Download,
    callbacks: SurfaceCallbacks,
    ready_projected: bool,
    notified: bool,
}

#[derive(Debug)]
enum Download {
    Queued(u64),
    Active(u64),
    Encoded(Arc<[u8]>),
    Finished,
}

impl HttpSurface {
    fn request(url: Url, identity: &str) -> Self {
        let request = alloc_async_id();
        let ticket = global_media_registry().queue(
            MediaAssetKey {
                source_fingerprint: format!("checked-package-http:{request}"),
                orientation: MediaOrientation::Normal,
                rendition: RenditionSpec::original(),
                revision: 1,
            },
            MediaMetadata::default(),
        );
        Self {
            url: Some(url),
            identity: identity.to_owned(),
            source: media_uri(ticket),
            ticket: Some(ticket),
            download: Download::Queued(request),
            callbacks: SurfaceCallbacks::default(),
            ready_projected: false,
            notified: false,
        }
    }

    fn private(source: &str) -> Self {
        Self {
            url: None,
            identity: String::new(),
            source: source.to_owned(),
            ticket: None,
            download: Download::Finished,
            callbacks: SurfaceCallbacks::default(),
            ready_projected: false,
            notified: false,
        }
    }

    fn poll_request(&mut self) {
        let (Some(ticket), Some(url)) = (self.ticket, self.url.as_ref()) else { return };
        let request = match self.download {
            Download::Queued(request) => {
                register_live_op(request);
                submit_client_job(request, fetch_image(url.clone()));
                self.download = Download::Active(request);
                return;
            }
            Download::Active(request) => request,
            Download::Encoded(ref body) => {
                if global_media_worker_pool().submit_bytes(
                    ticket,
                    Arc::clone(body),
                    RenditionSpec::original(),
                    MediaPriority::Current,
                    1,
                    ticket.revision,
                ) {
                    self.download = Download::Finished;
                }
                return;
            }
            Download::Finished => return,
        };
        let Some(result) = take_live_op(request) else {
            return;
        };
        self.download = Download::Finished;
        let error = match result {
            Ok(AsyncResult::Structured { body, .. }) => {
                self.download = Download::Encoded(body.into());
                return;
            }
            Err(error) => error,
            _ => "invalid image response".into(),
        };
        let _ = global_media_registry().fail(ticket.id, MediaAssetError::Rejected(error));
    }

    fn notification(&self) -> Option<&PackageQueuedAction> {
        if self.notified {
            return None;
        }
        let status = if self.ticket.is_some() {
            media_surface_status(&self.source)
        } else {
            crate::back_proxy::native_files::media_status(&self.source)
        };
        match status {
            MediaSurfaceStatus::Ready if self.ready_projected => self.callbacks.loaded.as_ref(),
            MediaSurfaceStatus::Failed(_) => self.callbacks.error.as_ref(),
            _ => None,
        }
    }
}

impl Drop for HttpSurface {
    fn drop(&mut self) {
        if let Download::Active(request) = self.download {
            cancel_live_op(request);
        }
        if let Some(ticket) = self.ticket {
            global_media_registry().release(ticket.id);
        }
    }
}

async fn fetch_image(url: Url) -> Result<AsyncResult, String> {
    static CLIENT: LazyLock<Result<reqwest::Client, reqwest::Error>> = LazyLock::new(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
    });
    let client = CLIENT.as_ref().map_err(|error| error.to_string())?;
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP image request returned {status}"));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_IMAGE_FILE_BYTES)
    {
        return Err(MediaAssetError::FileTooLarge.to_string());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if body.len() as u64 + chunk.len() as u64 > MAX_IMAGE_FILE_BYTES {
            return Err(MediaAssetError::FileTooLarge.to_string());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(AsyncResult::Structured {
        status: status.as_u16(),
        headers: Vec::new(),
        body,
    })
}
