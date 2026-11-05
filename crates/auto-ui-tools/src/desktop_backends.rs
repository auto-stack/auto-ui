//! Desktop Vue apps retain their declared backend profile and port. Reuse the
//! existing Rust server launcher and VM session proxy; own only spawned children.
use auto_val::AutoResult;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct DesktopBackend {
    pub id: String,
    pub root: PathBuf,
    pub profile: String,
    pub port: u16,
}

impl DesktopBackend {
    pub fn read(id: &str, root: &Path) -> AutoResult<Option<Self>> {
        if auto_lang_core::config::resolve_back_api(root).is_none() {
            return Ok(None);
        }
        let config = auto_lang_core::config::AutoConfig::read(&root.join("pac.at"))?;
        let profile = config.root.get_prop("api").to_astr().to_string();
        let profile = if profile.is_empty() {
            "vm".to_string()
        } else {
            profile
        };
        if profile == "none" {
            return Ok(None);
        }
        if !matches!(profile.as_str(), "rust" | "vm") {
            return Err(format!("desktop app {id}: unsupported backend profile {profile}").into());
        }
        let port = config
            .root
            .get_prop("back_port")
            .to_astr()
            .parse::<u16>()
            .unwrap_or(0);
        if profile == "rust" && port == 0 {
            return Err(
                format!("desktop app {id}: Rust backend requires a nonzero back_port").into(),
            );
        }
        Ok(Some(Self {
            id: id.into(),
            root: root.to_path_buf(),
            profile,
            port,
        }))
    }
}

pub(crate) struct DesktopBackends {
    io: Arc<crate::proc_host::RunIo>,
    child_mark: usize,
    owns_io: bool,
}

impl Drop for DesktopBackends {
    fn drop(&mut self) {
        self.io.kill_children_since(self.child_mark);
        if self.owns_io {
            crate::proc_host::uninstall();
        }
    }
}

impl DesktopBackends {
    pub fn start(output: &Path) -> AutoResult<Self> {
        let entries: Vec<DesktopBackend> =
            serde_json::from_str(&fs::read_to_string(output.join("desktop-backends.json"))?)?;
        validate_assignments(&entries)?;
        let owns_io = if crate::proc_host::current().is_none() {
            crate::proc_host::install()
        } else {
            false
        };
        let io = crate::proc_host::current().ok_or("desktop backend process owner unavailable")?;
        let owned = Self {
            child_mark: io.child_mark(),
            io,
            owns_io,
        };
        let mut sessions = Vec::new();
        let mut routes = Vec::new();
        for app in entries {
            if crate::proc_host::cancelled() {
                return Err("desktop backend startup cancelled".into());
            }
            if app.profile == "vm" {
                sessions.push(auto_ui_runtime::back_proxy::SessionSpec {
                    app_id: app.id.clone(),
                    back_entry: auto_lang_core::config::resolve_back_api(&app.root)
                        .ok_or("missing VM backend contract")?,
                });
                continue;
            }
            // Never let the single-app launcher's legacy port cleanup kill a
            // pre-existing service on behalf of a desktop app.
            if std::net::TcpStream::connect(("127.0.0.1", app.port)).is_ok() {
                return Err(format!(
                    "desktop app {}: backend port {} is already in use",
                    app.id, app.port
                )
                .into());
            }
            let old_port = std::env::var_os("AUTO_HTTP_PORT");
            let old_base = std::env::var_os("AUTO_HTTP_BASE");
            std::env::set_var("AUTO_HTTP_PORT", app.port.to_string());
            std::env::remove_var("AUTO_HTTP_BASE");
            let launched = crate::rust_ui::start_desktop_api_server(&app.root);
            for (key, old) in [("AUTO_HTTP_PORT", old_port), ("AUTO_HTTP_BASE", old_base)] {
                if let Some(value) = old {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
            match launched {
                Ok(Some(mut child)) => {
                    // The installed owner must always take this path's trees.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("desktop backend escaped process ownership".into());
                }
                Ok(None) => {}
                Err(err) => return Err(format!("desktop app {}: {err}", app.id).into()),
            }
            routes.push(serde_json::json!({"id": app.id, "target": format!("http://127.0.0.1:{}", app.port), "strip": true}));
        }
        if !sessions.is_empty() {
            let ids: Vec<_> = sessions.iter().map(|s| s.app_id.clone()).collect();
            let proxy =
                auto_ui_runtime::back_proxy::start(auto_ui_runtime::back_proxy::BackProxyConfig {
                    sessions,
                    lazy_sessions: true,
                    ..Default::default()
                })
                .map_err(|err| format!("desktop VM back-proxy: {err}"))?;
            for id in ids {
                routes.push(serde_json::json!({"id": id, "target": format!("http://127.0.0.1:{}", proxy.port), "strip": false}));
            }
        }
        fs::write(
            output.join("desktop-proxies.json"),
            serde_json::to_string_pretty(&routes)?,
        )?;
        Ok(owned)
    }
}

fn validate_assignments(entries: &[DesktopBackend]) -> AutoResult<()> {
    let mut ports = std::collections::HashSet::new();
    for app in entries.iter().filter(|app| app.profile == "rust") {
        if !ports.insert(app.port) {
            return Err(format!(
                "desktop backend port {} is assigned more than once",
                app.port
            )
            .into());
        }
        if std::net::TcpStream::connect(("127.0.0.1", app.port)).is_ok() {
            return Err(format!(
                "desktop app {}: backend port {} is already in use",
                app.id, app.port
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "real Rust backend compile/start/HTTP/owned-tree cleanup; requires PLAN053_PROBE_DIR"]
    fn desktop_rust_backend_roundtrip_and_cleanup() {
        use std::io::{Read, Write};
        let root = PathBuf::from(std::env::var_os("PLAN053_PROBE_DIR").expect("PLAN053_PROBE_DIR"));
        fs::create_dir_all(root.join("src/back")).unwrap();
        fs::create_dir_all(root.join("host")).unwrap();
        let reservation = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = reservation.local_addr().unwrap().port();
        fs::write(root.join("pac.at"), format!("name: \"desktop_api_probe\"\nversion: \"0.1.0\"\nscene: \"ui\"\nrender: \"vue\"\napi: \"rust\"\nback_port: {port}\n")).unwrap();
        fs::write(root.join("src/back/api.at"), "#[api(method = \"GET\", path = \"/api/probe\")]\npub fn value() int { return 53 }\n").unwrap();
        let backend = DesktopBackend::read("probe", &root).unwrap().unwrap();
        fs::write(root.join("host/desktop-backends.json"), serde_json::to_string(&vec![backend]).unwrap()).unwrap();
        drop(reservation);
        {
            let _owned = DesktopBackends::start(&root.join("host")).unwrap();
            let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            stream.write_all(b"GET /api/probe HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            let (headers, body) = response.split_once("\r\n\r\n").unwrap();
            assert!(headers.lines().next().unwrap().contains("200"), "{headers}");
            assert_eq!(body.trim(), "53");
            let proxies: serde_json::Value = serde_json::from_str(&fs::read_to_string(root.join("host/desktop-proxies.json")).unwrap()).unwrap();
            assert_eq!(proxies[0]["id"], "probe");
            assert_eq!(proxies[0]["strip"], true);
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            assert!(std::time::Instant::now() < deadline, "owned backend survived cleanup");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn desktop_profile_and_port_are_read_from_each_app() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src/back")).unwrap();
        fs::write(
            temp.path().join("src/back/api.at"),
            "pub fn status() int { return 1 }",
        )
        .unwrap();
        fs::write(
            temp.path().join("pac.at"),
            "name: \"a\"\napi: \"rust\"\nback_port: 17801\n",
        )
        .unwrap();
        let app = DesktopBackend::read("a", temp.path()).unwrap().unwrap();
        assert_eq!((app.profile.as_str(), app.port), ("rust", 17801));
        fs::write(temp.path().join("pac.at"), "name: \"a\"\n").unwrap();
        assert_eq!(
            DesktopBackend::read("a", temp.path())
                .unwrap()
                .unwrap()
                .profile,
            "vm"
        );
        fs::write(temp.path().join("pac.at"), "api: \"rust\"\n").unwrap();
        assert!(DesktopBackend::read("a", temp.path()).is_err());
        fs::write(temp.path().join("pac.at"), "api: \"other\"\n").unwrap();
        assert!(DesktopBackend::read("a", temp.path()).is_err());
    }

    #[test]
    fn desktop_refuses_duplicate_or_occupied_ports_before_starting_children() {
        let app = |id: &str, port| DesktopBackend {
            id: id.into(),
            root: PathBuf::new(),
            profile: "rust".into(),
            port,
        };
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(validate_assignments(&[app("busy", port)])
            .unwrap_err()
            .to_string()
            .contains("already in use"));
        drop(listener);
        assert!(validate_assignments(&[app("a", port), app("b", port)])
            .unwrap_err()
            .to_string()
            .contains("more than once"));
        assert!(validate_assignments(&[app("a", port)]).is_ok());
    }
}
