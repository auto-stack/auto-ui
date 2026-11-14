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

/// PLAN-083 T-03/D-04：声明 daemon 就是该应用后端时的共享判定。
///
/// 冻结规则：pac `back_port` == manifest `daemon.port` 且条目声明了 daemon →
/// 共享 provider——不启动第二份自有后端、不复用/清理他人端口，路由到已核验
/// 身份的 daemon；普通端口占用仍按既有规则拒绝。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedBackendDecision {
    NotShared,
    Shared { daemon_port: u16 },
}

pub(crate) fn shared_backend_decision(
    app_port: u16,
    declared: Option<&auto_ui_runtime::ui::app_registry::ManifestDaemon>,
) -> SharedBackendDecision {
    match declared {
        Some(def) if def.port == app_port => SharedBackendDecision::Shared {
            daemon_port: def.port,
        },
        _ => SharedBackendDecision::NotShared,
    }
}

impl DesktopBackends {
    pub fn start(output: &Path) -> AutoResult<Self> {
        let entries: Vec<DesktopBackend> =
            serde_json::from_str(&fs::read_to_string(output.join("desktop-backends.json"))?)?;
        // PLAN-083 T-03：先解析"声明 daemon 即本应用后端"的共享 provider
        // （含跨进程单实例门与身份核对），再校验/启动其余自有后端——共享
        // 端口不参与"已被占用"拒绝，也不允许悄悄起第二份后端。
        let mut shared: std::collections::HashMap<
            String,
            (String, auto_ui_runtime::ui::osconfig_daemon::HostBackendEnv),
        > = std::collections::HashMap::new();
        let mut shared_ports: std::collections::HashSet<u16> = std::collections::HashSet::new();
        for app in entries.iter().filter(|app| app.profile == "rust") {
            let declared = auto_ui_runtime::ui::app_registry::manifest_daemon_lookup(
                Path::new(".."),
                &app.id,
            );
            let decision =
                shared_backend_decision(app.port, declared.as_ref().map(|(_, def)| def));
            if let SharedBackendDecision::Shared { daemon_port } = decision {
                if daemon_port != app.port {
                    continue;
                }
                match auto_ui_runtime::ui::osconfig_daemon::resolve_manifest_host_shared(
                    Path::new(".."),
                    &app.id,
                    std::time::Duration::from_secs(5),
                ) {
                    auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Ready {
                        url,
                        env,
                    } => {
                        shared_ports.insert(app.port);
                        shared.insert(app.id.clone(), (url, env));
                    }
                    auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Invalid(
                        reason,
                    )
                    | auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::Offline(
                        reason,
                    ) => {
                        return Err(format!(
                            "desktop app {}: 声明的共享 provider 无法就绪——{reason}",
                            app.id
                        )
                        .into())
                    }
                    auto_ui_runtime::ui::osconfig_daemon::ManifestHostResolution::NotDeclared => {}
                }
            }
        }
        validate_assignments(&entries, &shared_ports)?;
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
            // 共享 provider 应用：不起自有后端、不碰其端口，直接路由到
            // 已核验身份的 daemon（Notes 主窗口 / Vue 桌面 API 去重）。
            if let Some((url, _env)) = shared.get(&app.id) {
                routes.push(
                    serde_json::json!({"id": app.id, "target": url.clone(), "strip": true}),
                );
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
            // PLAN-083 T-02/R-1（F-2）：声明的 daemon（非共享情形）必须在
            // 后端 spawn **之前** ensure 并解析宿主配置——后端在桌面启动期
            // 就绪，之后再补投递不可行。daemon Offline 仍不阻断后端启动
            // （既有 "Offline 不阻断 launch" 语义），但要明确记录"后端在无
            // 宿主配置下启动"；声明非法直接拒绝。
            let host_env = match auto_ui_runtime::ui::osconfig_daemon::resolve_declared_backend(
                Path::new(".."),
                &app.id,
                std::time::Duration::from_secs(5),
            ) {
                auto_ui_runtime::ui::osconfig_daemon::DeclaredBackendConfig::NotDeclared => None,
                auto_ui_runtime::ui::osconfig_daemon::DeclaredBackendConfig::Ready(env) => {
                    Some(env)
                }
                auto_ui_runtime::ui::osconfig_daemon::DeclaredBackendConfig::Invalid(reason) => {
                    eprintln!(
                        "[plan083] desktop app {}: 声明能力非法，拒绝启动其后端——{reason}",
                        app.id
                    );
                    return Err(format!("desktop app {}: {reason}", app.id).into());
                }
                auto_ui_runtime::ui::osconfig_daemon::DeclaredBackendConfig::Offline(reason) => {
                    eprintln!(
                        "[plan083] desktop app {}: 声明 daemon 未就绪（{reason}）；\
                         后端在无宿主配置下启动（既有 Offline 不阻断语义）",
                        app.id
                    );
                    None
                }
            };
            let old_port = std::env::var_os("AUTO_HTTP_PORT");
            let old_base = std::env::var_os("AUTO_HTTP_BASE");
            std::env::set_var("AUTO_HTTP_PORT", app.port.to_string());
            std::env::remove_var("AUTO_HTTP_BASE");
            let launched = crate::rust_ui::start_desktop_api_server(&app.root, host_env.as_ref());
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

fn validate_assignments(
    entries: &[DesktopBackend],
    shared_ports: &std::collections::HashSet<u16>,
) -> AutoResult<()> {
    let mut ports = std::collections::HashSet::new();
    for app in entries
        .iter()
        .filter(|app| app.profile == "rust" && !shared_ports.contains(&app.port))
    {
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
        assert!(validate_assignments(&[app("busy", port)], &Default::default())
            .unwrap_err()
            .to_string()
            .contains("already in use"));
        drop(listener);
        assert!(
            validate_assignments(&[app("a", port), app("b", port)], &Default::default())
                .unwrap_err()
                .to_string()
                .contains("more than once")
        );
        assert!(validate_assignments(&[app("a", port)], &Default::default()).is_ok());
    }

    // ---- PLAN-083 T-03：共享 provider 判定（Notes 17819 双绑去重） ----

    #[test]
    fn shared_backend_decision_matches_declared_daemon_port_only() {
        let def = |port: u16| auto_ui_runtime::ui::app_registry::ManifestDaemon {
            port,
            bin: None,
            env_port: None,
            health_interface: Some("notes.fpack/1".to_string()),
            ready_timeout_ms: None,
            env: Default::default(),
            host: None,
            instance_key: None,
            data_root: None,
        };
        assert_eq!(
            shared_backend_decision(17819, Some(&def(17819))),
            SharedBackendDecision::Shared { daemon_port: 17819 },
            "back_port 与声明 daemon 端口相同 = 共享 provider（不再起第二份后端）"
        );
        assert_eq!(
            shared_backend_decision(17819, Some(&def(17820))),
            SharedBackendDecision::NotShared,
            "端口不同 = 自有后端（普通占用仍按既有规则拒绝）"
        );
        assert_eq!(
            shared_backend_decision(17819, None),
            SharedBackendDecision::NotShared,
            "未声明 daemon = 零行为变化"
        );
        // 共享端口不参与"已被占用"拒绝；非共享端口仍拒绝。
        let app = |id: &str, port| DesktopBackend {
            id: id.into(),
            root: PathBuf::new(),
            profile: "rust".into(),
            port,
        };
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let shared: std::collections::HashSet<u16> = [port].into_iter().collect();
        assert!(
            validate_assignments(&[app("015-notes", port)], &shared).is_ok(),
            "共享 provider 端口已被 daemon 持有不算冲突"
        );
        assert!(
            validate_assignments(&[app("other", port)], &Default::default()).is_err(),
            "普通占用者仍拒绝"
        );
        drop(listener);
    }
}
