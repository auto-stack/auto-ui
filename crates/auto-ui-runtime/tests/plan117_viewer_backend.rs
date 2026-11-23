#![cfg(feature = "ui")]

use auto_ui_runtime::back_proxy::{start, BackProxyConfig, SessionSpec};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(feature = "ace-bridge")]
#[test]
fn checked_viewer_emits_get_and_json_post_in_suspending_handlers() {
    use auto_ui_runtime::ui::ace_package_sfc::generate_linked_front_sfc;
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/plan117/viewer_http.at");
    let source = include_str!("fixtures/plan117/viewer_http.at");
    let generated = generate_linked_front_sfc(&entry, source, &[])
        .unwrap()
        .unwrap();
    assert!(generated
        .app
        .contains("a2x_str_to_owned(a2x_str_lit((yield { url:"));
    assert!(generated
        .app
        .contains("a2x_json_parse((yield a2x_str_utf8("));
    assert!(generated.app.contains("options.method = 'POST'"));
}

#[cfg(feature = "ace-bridge")]
#[test]
fn original_viewer_emits_the_checked_thumbnail_and_backend_handlers() {
    use auto_ui_runtime::ui::ace_package_sfc::generate_linked_front_sfc;

    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/031-image-viewer/src/front/app.at");
    let original = std::fs::read_to_string(&entry).unwrap();
    let generated = generate_linked_front_sfc(&entry, &original, &[])
        .unwrap()
        .unwrap();
    assert!(generated.app.contains("Image Viewer"));
    assert!(generated.app.contains("fetchFileBackend"));
    assert!(generated.app.contains("yield* a2x_fn_p4_backm3_api_request_view("));
}

#[cfg(feature = "ace-bridge")]
#[test]
fn declared_viewer_api_binds_typed_calls_to_the_private_transport() {
    use auto_ui_runtime::ui::ace_package_sfc::generate_linked_front_sfc;

    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/031-image-viewer/src/front/app.at");
    let fixture = include_str!("fixtures/plan117/viewer_contract.at");
    let generated = generate_linked_front_sfc(&entry, fixture, &[])
        .unwrap()
        .unwrap();
    assert!(generated.app.contains("import { fetchFileBackend } from '@/lib/file-browser'"));
    assert!(generated.app.contains("a2x_str_lit(\"auto:back:/api/viewer/open-file\")"));
    assert!(generated.app.contains("a2x_str_lit(\"auto:back:/api/viewer/names?\")"));
    assert!(generated.app.lines().any(|line| {
        line.contains("a2x_dyn_take(") && line.contains(", [\"list\", \"Str\"])")
    }));

    let bad_import = fixture.replace("open_file", "unlisted_endpoint");
    let error = generate_linked_front_sfc(&entry, &bad_import, &[]).err().unwrap();
    assert!(error.contains("undeclared endpoint `unlisted_endpoint`"), "{error}");
}

struct Reply {
    status: u16,
    headers: String,
    body: Vec<u8>,
}

fn request(base: &str, method: &str, path: &str, body: Option<Value>) -> Reply {
    let (authority, app) = base
        .strip_prefix("http://")
        .unwrap()
        .split_once('/')
        .unwrap();
    let body = body.map(|value| value.to_string()).unwrap_or_default();
    let mut socket = TcpStream::connect(authority).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(socket,
        "{method} /{app}{path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len(),
    ).unwrap();
    let mut bytes = Vec::new();
    socket.read_to_end(&mut bytes).unwrap();
    let split = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .unwrap();
    let headers = String::from_utf8(bytes[..split].to_vec()).unwrap();
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    Reply {
        status,
        headers,
        body: bytes.split_off(split + 4),
    }
}

fn control(base: &str, method: &str, path: &str, body: Option<Value>) -> Value {
    let reply = request(base, method, path, body);
    assert_eq!(
        reply.status,
        200,
        "{}",
        String::from_utf8_lossy(&reply.body)
    );
    serde_json::from_slice(&reply.body).unwrap()
}

fn ready(base: &str, session: &str) -> (String, Reply) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let uri = control(
            base,
            "GET",
            &format!("/api/viewer/current-uri?session={session}"),
            None,
        );
        if let Some(uri) = uri.as_str().filter(|uri| !uri.is_empty()) {
            assert!(uri.starts_with("/api/__auto/media/"));
            let reply = request(base, "GET", uri, None);
            if reply.status != 503 {
                return (uri.to_owned(), reply);
            }
        }
        assert!(Instant::now() < deadline, "media did not settle");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn original_viewer_uses_only_the_live_private_file_grant() {
    let project =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui/031-image-viewer");
    let fixtures = project.join("tests/fixtures");
    let original = std::fs::read(fixtures.join("scenic-320x240.png")).unwrap();
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("granted");
    let other_root = fixture.path().join("other");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&other_root).unwrap();
    std::fs::write(root.join("scenic.png"), &original).unwrap();
    std::fs::copy(fixtures.join("corrupt.bin"), root.join("bad.png")).unwrap();
    std::fs::write(other_root.join("scenic.png"), &original).unwrap();

    let spec = SessionSpec {
        app_id: "image-viewer".into(),
        back_entry: project.join("src/back/api.at"),
    };
    let proxy = start(BackProxyConfig::default()).unwrap();
    let first = proxy.open_file_session(spec.clone()).unwrap();
    let second = proxy.open_file_session(spec).unwrap();
    let first_url = first.base_url();
    let second_url = second.base_url();
    let open_file = |base: &str, path: &str| {
        control(
            base,
            "POST",
            "/api/viewer/open-file",
            Some(json!({"path": path})),
        )
    };
    let close = |base: &str, session: &str| {
        control(
            base,
            "POST",
            "/api/viewer/close",
            Some(json!({"session": session})),
        )
    };
    assert_eq!(
        open_file(&first_url, root.join("scenic.png").to_str().unwrap()),
        ""
    );
    first.grant_read(&root).unwrap();
    second.grant_read(&other_root).unwrap();
    assert_eq!(
        open_file(&first_url, other_root.join("scenic.png").to_str().unwrap()),
        ""
    );

    let directory = control(
        &first_url,
        "POST",
        "/api/viewer/open-directory",
        Some(json!({"path": root})),
    );
    let directory = directory.as_str().filter(|id| !id.is_empty()).unwrap();
    let (bad_uri, bad) = ready(&first_url, directory);
    assert_eq!(bad.status, 422);
    assert_eq!(
        control(
            &first_url,
            "GET",
            &format!("/api/viewer/names?session={directory}"),
            None
        ),
        json!(["bad.png", "scenic.png"])
    );
    assert_eq!(
        control(
            &second_url,
            "GET",
            &format!("/api/viewer/names?session={directory}"),
            None
        ),
        json!([])
    );
    control(
        &first_url,
        "POST",
        "/api/viewer/navigate",
        Some(json!({"session": directory, "delta": 1})),
    );
    let (uri, image) = ready(&first_url, directory);
    assert_ne!(uri, bad_uri);
    assert_eq!(image.status, 200);
    assert_eq!(image.body, original);
    let headers = image.headers.to_ascii_lowercase();
    let lengths: Vec<_> = headers
        .lines()
        .filter_map(|line| line.strip_prefix("content-length: "))
        .collect();
    let cache_policy: Vec<_> = headers
        .lines()
        .filter_map(|line| line.strip_prefix("cache-control: "))
        .collect();
    assert_eq!(lengths, [original.len().to_string()]);
    assert_eq!(cache_policy, ["no-store"]);
    let head = request(&first_url, "HEAD", &uri, None);
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    assert!(head
        .headers
        .to_ascii_lowercase()
        .contains(&format!("content-length: {}", original.len())));
    assert_eq!(request(&second_url, "GET", &uri, None).status, 404);
    assert_eq!(
        request(
            &format!("http://127.0.0.1:{}/apps/image-viewer", proxy.port),
            "GET",
            &uri,
            None
        )
        .status,
        404
    );
    assert_eq!(close(&first_url, directory), true);
    assert_eq!(close(&first_url, directory), false);
    assert_eq!(request(&first_url, "GET", &uri, None).status, 404);

    let opened = open_file(&first_url, "scenic.png");
    let opened = opened.as_str().filter(|id| !id.is_empty()).unwrap();
    let (_, image) = ready(&first_url, opened);
    assert_eq!(image.status, 200);
    control(
        &first_url,
        "POST",
        "/api/viewer/request-view",
        Some(json!({
            "session": opened, "index": 0, "viewport_width": 64, "viewport_height": 48, "quality": 90,
        })),
    );
    let (issued, image) = ready(&first_url, opened);
    assert_eq!(image.status, 200);
    first.revoke();
    assert_eq!(request(&first_url, "GET", &issued, None).status, 404);
    assert_eq!(
        control(
            &first_url,
            "GET",
            &format!("/api/viewer/current-uri?session={opened}"),
            None
        ),
        ""
    );
    first.grant_read(&root).unwrap();
    assert_eq!(request(&first_url, "GET", &issued, None).status, 404);

    let reopened = open_file(&first_url, "scenic.png");
    let reopened = reopened.as_str().filter(|id| !id.is_empty()).unwrap();
    let (reopened_uri, image) = ready(&first_url, reopened);
    assert_eq!(image.status, 200);
    first.close().unwrap().join().unwrap();
    assert_eq!(request(&first_url, "GET", &reopened_uri, None).status, 404);
    second.close().unwrap().join().unwrap();
}
