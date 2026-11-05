use std::io::{Read, Write};

#[test]
fn desktop_vm_backends_initialize_local_and_imported_constants() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("constants.at"), "const DEC_ERR_SYNTAX = -2000000000.0\npub fn read_error() float { return DEC_ERR_SYNTAX }\n").unwrap();
    let entry = root.path().join("api.at");
    std::fs::write(&entry, concat!(
        "use constants\nconst LOCAL_LIMIT = 53\n",
        "#[api(method = \"GET\", path = \"/api/local\")]\npub fn local_value() int { return LOCAL_LIMIT }\n",
        "#[api(method = \"GET\", path = \"/api/imported\")]\npub fn imported_value() float { return constants.read_error() }\n",
    )).unwrap();
    let proxy = auto_ui_runtime::back_proxy::start(auto_ui_runtime::back_proxy::BackProxyConfig {
        sessions: vec![auto_ui_runtime::back_proxy::SessionSpec { app_id: "constants".into(), back_entry: entry }],
        lazy_sessions: true,
        ..Default::default()
    }).unwrap();
    for (path, expected) in [("local", 53.0), ("imported", -2000000000.0)] {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", proxy.port)).unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(35))).unwrap();
        write!(stream, "GET /apps/constants/api/{path} HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (headers, body) = response.split_once("\r\n\r\n").unwrap();
        assert!(headers.lines().next().unwrap().contains("200"), "{response}");
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value.as_f64(), Some(expected), "{path}: {body}");
    }
}

#[test]
fn desktop_vm_whole_body_preserves_named_binding_and_missing_checks() {
    let root = tempfile::tempdir().unwrap();
    let entry = root.path().join("api.at");
    std::fs::write(&entry, concat!(
        "#[api(method = \"POST\", path = \"/api/echo\")]\npub fn echo(payload str) str { return payload }\n",
        "#[api(method = \"POST\", path = \"/api/pair\")]\npub fn pair(first str, second str) str { return second }\n",
    )).unwrap();
    let proxy = auto_ui_runtime::back_proxy::start(auto_ui_runtime::back_proxy::BackProxyConfig {
        sessions: vec![auto_ui_runtime::back_proxy::SessionSpec { app_id: "body".into(), back_entry: entry }],
        lazy_sessions: true,
        ..Default::default()
    }).unwrap();
    for (path, body, status, expected) in [
        ("echo", r#"{"payload":"命名+&"}"#, "200", Some("命名+&")),
        ("echo", r#""标量+&""#, "200", Some(r#""标量+&""#)),
        ("echo", r#"{"path":"图表053.csv"}"#, "200", Some(r#"{"path":"图表053.csv"}"#)),
        ("echo", "raw+&文本", "200", Some("raw+&文本")),
        ("pair", r#"{"first":"one"}"#, "400", None),
        ("echo", "", "400", None),
    ] {
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", proxy.port)).unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(35))).unwrap();
        write!(stream, "POST /apps/body/api/{path} HTTP/1.0\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (headers, response_body) = response.split_once("\r\n\r\n").unwrap();
        assert!(headers.lines().next().unwrap().contains(status), "{response}");
        if let Some(expected) = expected {
            assert_eq!(serde_json::from_str::<serde_json::Value>(response_body).unwrap().as_str(), Some(expected));
        }
    }
}
