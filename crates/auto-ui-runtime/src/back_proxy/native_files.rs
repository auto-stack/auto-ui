//! In-process transport for checked desktop file backends and their media.

use super::{
    open_file_session, route_request, FileBackendSession, ParsedRequest, ProxyReply, ProxyShared,
    SessionSpec, MAX_BODY_BYTES,
};
use auto_lang_core::file_session::FileSession;
use auto_media_core::image_pipeline::{parse_media_path, MediaAssetId, MediaSurfaceStatus};
use std::path::Path;
use std::sync::{Arc, LazyLock};

static NATIVE_FILES: LazyLock<Arc<ProxyShared>> =
    LazyLock::new(|| Arc::new(ProxyShared::default()));
const MEDIA_PREFIX: &str = "native-file-media:";

pub(crate) fn open(spec: &SessionSpec, source_dir: &Path) -> Result<FileBackendSession, String> {
    let access = FileSession::for_media_source(source_dir).map_err(|error| error.to_string())?;
    open_file_session(&NATIVE_FILES, 0, spec, Arc::new(access)).map_err(|error| error.to_string())
}

impl FileBackendSession {
    pub(crate) fn file_access(&self) -> Arc<FileSession> {
        self.access.clone()
    }

    pub(crate) fn native_json(
        &self,
        verb: &str,
        target: &str,
        body: Option<&str>,
    ) -> Result<String, String> {
        let refused = || String::from("invalid checked backend request");
        let (method, path, query, body) = match verb {
            "auto.http.get-json.v1" if body.is_none() => {
                let (path, params) = target.split_once('?').ok_or_else(refused)?;
                let params =
                    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(params)
                        .map_err(|_| refused())?;
                let mut url = reqwest::Url::parse("http://127.0.0.1/").expect("fixed URL");
                for (key, value) in params {
                    let value = match value {
                        serde_json::Value::String(value) => value,
                        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
                            value.to_string()
                        }
                        _ => return Err(refused()),
                    };
                    url.query_pairs_mut().append_pair(&key, &value);
                }
                ("GET", path, url.query().unwrap_or_default().to_owned(), "")
            }
            "auto.http.post-json.v1" => ("POST", target, String::new(), body.ok_or_else(refused)?),
            _ => return Err(refused()),
        };
        if body.len() > MAX_BODY_BYTES
            || !path.strip_prefix("/api/").is_some_and(|parts| {
                parts.split('/').all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                })
            })
        {
            return Err(refused());
        }
        let req = ParsedRequest {
            method: method.into(),
            path: format!("/apps/{}{path}", self.key),
            query,
            headers: vec![("content-type".into(), "application/json".into())],
            body: body.as_bytes().to_vec(),
        };
        match route_request(&self.shared, &req) {
            ProxyReply::Response { status, body, .. } if (200..300).contains(&status) => {
                if let Ok(uri) = serde_json::from_slice::<&str>(&body) {
                    if parse_media_path(uri).is_some() {
                        return serde_json::to_string(&format!("{MEDIA_PREFIX}{}{uri}", self.key))
                            .map_err(|error| error.to_string());
                    }
                }
                String::from_utf8(body).map_err(|error| error.to_string())
            }
            ProxyReply::Response { status, .. } => {
                Err(format!("checked backend returned HTTP {status}"))
            }
            _ => Err("checked backend expected a JSON response".into()),
        }
    }
}

pub(crate) fn is_media(source: &str) -> bool {
    source.starts_with(MEDIA_PREFIX)
}

fn with_media<T>(
    source: &str,
    use_media: impl FnOnce(&FileSession, &str) -> Option<T>,
) -> Option<T> {
    let (key, path) = source.strip_prefix(MEDIA_PREFIX)?.split_once('/')?;
    let access = NATIVE_FILES
        .sessions
        .lock()
        .ok()?
        .get(key)?
        .file_access
        .clone()?;
    use_media(&access, &format!("/{path}"))
}

pub(crate) fn media_status(source: &str) -> MediaSurfaceStatus {
    with_media(source, FileSession::media_status)
        .unwrap_or_else(|| MediaSurfaceStatus::Failed("private media ticket unavailable".into()))
}

pub(crate) fn media_render(source: &str) -> Option<(MediaAssetId, u32, u32, Arc<[u8]>)> {
    with_media(source, FileSession::media_render)
}
