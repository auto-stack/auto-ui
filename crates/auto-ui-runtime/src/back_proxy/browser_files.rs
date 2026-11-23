//! The browser lease holds a scoped VM session for the life of its connection.

use super::{
    error_json, file_nonce, open_file_session, start_proxy, write_response, BackProxyConfig,
    ParsedRequest, ProxyReply, ProxyShared, SessionSpec,
};
use auto_lang_core::file_session::{FileSession, FileWritePermissions};
use std::io::{self, ErrorKind, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

pub(super) const SESSION_PATH: &str = "/__auto/file-browser/session";

/// The Vue launcher owns this value for the duration of its dev server.
pub struct BrowserFileServer {
    pub port: u16,
    files: Arc<BrowserFiles>,
}

impl BrowserFileServer {
    /// Sent by the dev proxy only. This credential stays out of client assets.
    pub fn proxy_authorization(&self) -> &str {
        &self.files.authorization
    }
}

impl Drop for BrowserFileServer {
    fn drop(&mut self) {
        if let Some(grant) = self.files.grant.lock().unwrap().take() {
            for session in grant
                .sessions
                .into_iter()
                .filter_map(|session| session.upgrade())
            {
                session.close();
            }
        }
    }
}

pub fn start_file_browser(spec: SessionSpec, root: &Path) -> io::Result<BrowserFileServer> {
    start_file_browser_with_writes(spec, root, FileWritePermissions::default())
}

pub fn start_file_browser_with_writes(
    spec: SessionSpec,
    root: &Path,
    writes: FileWritePermissions,
) -> io::Result<BrowserFileServer> {
    start_scoped_browser(spec, root, writes, FileSession::default())
}

pub fn start_file_viewer(
    spec: SessionSpec,
    root: &Path,
    source_dir: &Path,
) -> io::Result<BrowserFileServer> {
    let authority = FileSession::for_media_source(source_dir).map_err(io::Error::other)?;
    start_scoped_browser(spec, root, FileWritePermissions::default(), authority)
}

fn start_scoped_browser(
    spec: SessionSpec,
    root: &Path,
    writes: FileWritePermissions,
    root_authority: FileSession,
) -> io::Result<BrowserFileServer> {
    root_authority.grant(root, writes).map_err(io::Error::other)?;
    let files = Arc::new(BrowserFiles {
        authorization: file_nonce()?,
        spec,
        grant: Mutex::new(Some(LaunchGrant {
            root: root_authority,
            sessions: Vec::new(),
        })),
    });
    let proxy = start_proxy(BackProxyConfig::default(), Some(files.clone()))?;
    Ok(BrowserFileServer {
        port: proxy.port,
        files,
    })
}

pub(super) struct BrowserFiles {
    authorization: String,
    spec: SessionSpec,
    grant: Mutex<Option<LaunchGrant>>,
}

struct LaunchGrant {
    root: FileSession,
    sessions: Vec<Weak<FileSession>>,
}

impl BrowserFiles {
    pub(super) fn is_active(&self) -> bool {
        self.grant.lock().unwrap().is_some()
    }

    pub(super) fn scope_media_uri(&self, session: &str, reply: &mut ProxyReply) {
        if self.spec.app_id != "image-viewer" {
            return;
        }
        let ProxyReply::Response { status: 200, content_type: "application/json", body } = reply else {
            return;
        };
        let Ok(uri) = serde_json::from_slice::<&str>(body) else {
            return;
        };
        if auto_media_core::image_pipeline::parse_media_path(uri).is_some() {
            *body = serde_json::to_vec(&format!("/apps/{session}{uri}"))
                .expect("a media URI serializes as JSON");
        }
    }

    pub(super) fn permits(&self, req: &ParsedRequest) -> bool {
        let header = |name| {
            req.headers
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
        };
        header("sec-fetch-site") == Some("same-origin")
            && header("x-auto-file-browser") == Some(self.authorization.as_str())
            && self.is_active()
    }

    fn open_grant(&self) -> io::Result<Arc<FileSession>> {
        let mut launch = self.grant.lock().unwrap();
        let launch = launch
            .as_mut()
            .ok_or_else(|| io::Error::from(ErrorKind::NotConnected))?;
        let session = Arc::new(launch.root.fork_grant().map_err(io::Error::other)?);
        launch
            .sessions
            .retain(|session| session.strong_count() != 0);
        launch.sessions.push(Arc::downgrade(&session));
        Ok(session)
    }

    pub(super) fn serve_session(
        &self,
        mut stream: TcpStream,
        shared: &Arc<ProxyShared>,
        req: &ParsedRequest,
    ) -> io::Result<()> {
        if req.method != "POST" || !req.query.is_empty() || !req.body.is_empty() {
            return write_response(
                &mut stream,
                ProxyReply::json(400, error_json("invalid file session request")),
                true,
            );
        }
        let session = match self.open_grant().and_then(|access| {
            open_file_session(shared, stream.local_addr()?.port(), &self.spec, access)
        }) {
            Ok(session) => session,
            Err(_) => {
                return write_response(
                    &mut stream,
                    ProxyReply::json(503, error_json("file session unavailable")),
                    true,
                )
            }
        };
        let interval = Duration::from_millis(500);
        stream.set_read_timeout(Some(interval))?;
        stream.set_write_timeout(Some(interval))?;
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n/apps/{}\n", session.key)?;
        while self.is_active() {
            match stream.peek(&mut [0]) {
                Ok(_) => break,
                Err(error)
                    if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) =>
                {
                    stream.write_all(b"\n")?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
