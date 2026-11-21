#![cfg(feature = "ui")]

use auto_lang_core::file_session::FileWritePermissions;
use auto_ui_runtime::back_proxy::{
    start, start_file_browser, start_file_browser_with_writes, BackProxyConfig, BrowserFileServer,
    SessionSpec,
};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn original_backend() -> SessionSpec {
    SessionSpec {
        app_id: "file-manager".into(),
        back_entry: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/027-file-manager/src/back/api.at"),
    }
}

fn get(url: &str, endpoint: &str) -> (String, String) {
    let (authority, app) = url
        .strip_prefix("http://")
        .unwrap()
        .split_once('/')
        .unwrap();
    let mut stream = TcpStream::connect(authority).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "GET /{app}/api/fs/{endpoint} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (header, body) = response.split_once("\r\n\r\n").unwrap();
    (header.to_owned(), body.to_owned())
}

fn get_json(url: &str, endpoint: &str) -> Value {
    let (header, body) = get(url, endpoint);
    assert!(header.starts_with("HTTP/1.1 200 "), "{header}\n{body}");
    serde_json::from_str(&body).unwrap()
}

fn list(url: &str, path: &Path) -> Value {
    let encoded = urlencoding::encode(path.to_str().unwrap());
    get_json(url, &format!("list?path={encoded}"))
}

#[test]
fn original_file_manager_reads_only_from_the_live_host_grant() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("first");
    let nested = root.join("nested");
    let empty = root.join("empty");
    let other = fixture.path().join("second");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir(&empty).unwrap();
    std::fs::create_dir(&other).unwrap();
    let inside = nested.join("目录.É");
    std::fs::write(&inside, "checked fixture").unwrap();
    std::fs::write(other.join("outside.txt"), "disjoint fixture").unwrap();
    let outside_link = root.join("outside-link");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&other, &outside_link).unwrap();
    #[cfg(windows)]
    {
        // A directory junction exercises the Windows boundary without requiring
        // the privilege needed to create directory symlinks.
        let result = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "New-Item -ItemType Junction -Path $env:AUTO_TEST_FILE_LINK -Target $env:AUTO_TEST_FILE_TARGET -ErrorAction Stop | Out-Null",
            ])
            .env("AUTO_TEST_FILE_LINK", &outside_link)
            .env("AUTO_TEST_FILE_TARGET", &other)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    let spec = original_backend();
    let proxy = start(BackProxyConfig {
        sessions: vec![spec.clone()],
        lazy_sessions: true,
        ..Default::default()
    })
    .unwrap();
    let first = proxy.open_file_session(spec.clone()).unwrap();
    let first_url = first.base_url();
    let second = proxy.open_file_session(spec.clone()).unwrap();
    let second_url = second.base_url();
    let public_url = format!("http://127.0.0.1:{}/apps/file-manager", proxy.port);
    let denied = json!({"ok": false, "path": "", "entries": []});

    assert_ne!(first_url, second_url);
    assert!(get(&first_url, "home")
        .0
        .contains("Cache-Control: no-store"));
    assert_eq!(get_json(&first_url, "home"), "");
    assert_eq!(get_json(&first_url, "drives"), json!([]));
    assert_eq!(list(&first_url, &nested), denied);

    first.grant_read(&root).unwrap();
    second.grant_read(&other).unwrap();
    assert_eq!(get_json(&first_url, "home"), root.to_str().unwrap());
    assert_eq!(
        get_json(&first_url, "drives")[0]["path"],
        root.to_str().unwrap()
    );
    let listing = list(&first_url, &nested);
    let metadata = std::fs::metadata(&inside).unwrap();
    let mtime = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    assert_eq!(
        listing,
        json!({"ok": true, "path": nested, "entries": [{
            "name": "目录.É", "path": inside, "is_dir": false, "extension": "é",
            "size": metadata.len(), "mtime": mtime.as_secs()
        }]})
    );
    assert_eq!(list(&first_url, &root)["ok"], true);
    assert_eq!(
        list(&first_url, &empty),
        json!({"ok":true,"path":empty,"entries":[]})
    );
    assert_eq!(list(&first_url, &other), denied);
    assert_eq!(list(&first_url, &root.join("../second")), denied);
    assert_eq!(list(&first_url, &root.join("missing")), denied);
    #[cfg(any(windows, unix))]
    assert_eq!(list(&first_url, &outside_link), denied);
    assert_eq!(list(&public_url, &nested), denied);
    assert_eq!(list(&second_url, &nested), denied);
    assert_eq!(
        list(&second_url, &other)["entries"][0]["name"],
        "outside.txt"
    );

    first.revoke();
    assert_eq!(get_json(&first_url, "home"), "");
    assert_eq!(list(&first_url, &nested), denied);
    first.grant_read(&root).unwrap();
    assert_eq!(list(&first_url, &nested), listing);
    first.close().unwrap().join().unwrap();
    let (header, _) = get(&first_url, "home");
    assert!(header.starts_with("HTTP/1.1 404 "));
    assert!(header.contains("Cache-Control: no-store"));
    assert!(first.grant_read(&root).is_err());
    assert_eq!(list(&second_url, &other)["ok"], true);

    let replacement = proxy.open_file_session(spec).unwrap();
    let replacement_url = replacement.base_url();
    assert_ne!(replacement_url, first_url);
    assert_eq!(list(&replacement_url, &nested), denied);
    replacement.grant_read(&root).unwrap();
    assert_eq!(list(&replacement_url, &nested), listing);
    drop(replacement);
    assert!(get(&replacement_url, "home").0.starts_with("HTTP/1.1 404 "));
    second.close().unwrap().join().unwrap();
    proxy.remove_app("file-manager").unwrap().join().unwrap();
}

fn browser_request(port: u16, method: &str, path: &str, site: &str, credential: &str) -> TcpStream {
    browser_request_with_body(port, method, path, site, credential, "")
}

fn browser_request_with_body(
    port: u16, method: &str, path: &str, site: &str, credential: &str, body: &str,
) -> TcpStream {
    let mut socket = TcpStream::connect(("127.0.0.1", port)).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    write!(socket, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nSec-Fetch-Site: {site}\r\nX-Auto-File-Browser: {credential}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    socket
}

fn browser_response(mut socket: TcpStream) -> (String, String) {
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    let (header, body) = response.split_once("\r\n\r\n").unwrap();
    (header.to_owned(), body.to_owned())
}

fn browser_session(port: u16, credential: &str) -> (BufReader<TcpStream>, String) {
    let mut stream = BufReader::new(browser_request(
        port, "POST", "/__auto/file-browser/session", "same-origin", credential,
    ));
    let mut headers = String::new();
    while !headers.ends_with("\r\n\r\n") {
        assert_ne!(stream.read_line(&mut headers).unwrap(), 0, "{headers}");
    }
    assert!(
        headers.starts_with("HTTP/1.1 200 ") && headers.contains("Cache-Control: no-store")
    );
    let mut base = String::new();
    stream.read_line(&mut base).unwrap();
    assert!(base.starts_with("/apps/file-session-"));
    (stream, base.trim_end().to_owned())
}

#[test]
fn browser_connection_owns_an_independent_read_grant() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("approved");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("真实.txt"), "test").unwrap();
    std::fs::write(fixture.path().join("private.txt"), "separate").unwrap();
    let server = start_file_browser(original_backend(), &root).unwrap();
    let port = server.port;
    let credential = server.proxy_authorization();
    let bootstrap = "/__auto/file-browser/session";

    for (site, key) in [("same-origin", ""), ("cross-site", credential)] {
        let (header, _) = browser_response(browser_request(port, "POST", bootstrap, site, key));
        assert!(header.starts_with("HTTP/1.1 404 "));
    }
    let unwanted_root = format!(
        "{bootstrap}?root={}",
        urlencoding::encode(fixture.path().to_str().unwrap())
    );
    let (header, _) = browser_response(browser_request(
        port,
        "POST",
        &unwanted_root,
        "same-origin",
        credential,
    ));
    assert!(header.starts_with("HTTP/1.1 400 "));

    let (first, first_path) = browser_session(port, credential);
    let (second, second_path) = browser_session(port, credential);
    assert_ne!(first_path, second_path);
    let get = |base: &str, endpoint: &str| {
        browser_response(browser_request(
            port,
            "GET",
            &format!("{base}/api/fs/{endpoint}"),
            "same-origin",
            credential,
        ))
    };
    let root_query = urlencoding::encode(root.to_str().unwrap());
    let (_, listing) = get(&first_path, &format!("list?path={root_query}"));
    assert_eq!(
        serde_json::from_str::<Value>(&listing).unwrap()["entries"][0]["name"],
        "真实.txt"
    );
    let other_query = urlencoding::encode(fixture.path().to_str().unwrap());
    let (_, denied) = get(&first_path, &format!("list?path={other_query}"));
    assert_eq!(serde_json::from_str::<Value>(&denied).unwrap()["ok"], false);

    drop(first);
    let deadline = Instant::now() + Duration::from_secs(2);
    while !get(&first_path, "home").0.starts_with("HTTP/1.1 404 ") {
        assert!(
            Instant::now() < deadline,
            "closed page session was retained"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let (_, home) = get(&second_path, "home");
    assert_eq!(
        serde_json::from_str::<String>(&home).unwrap(),
        root.to_str().unwrap()
    );

    drop(server);
    let mut second = second;
    let mut tail = String::new();
    second.read_to_string(&mut tail).unwrap();
    assert!(tail.trim().is_empty());
}

#[test]
fn browser_file_actions_follow_the_current_root_and_operation_grant() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("allowed");
    std::fs::create_dir(&root).unwrap();
    let source = root.join("原件.txt");
    let copy = root.join("副本.txt");
    let moved = root.join("移动.txt");
    let directory = root.join("新建目录");
    let outside = fixture.path().join("outside.txt");
    std::fs::write(&outside, "outside").unwrap();

    let request = |server: &BrowserFileServer, base: &str, site: &str, operation: &str, path: &Path, target: &Path| {
        let payload = json!({ "operation": operation, "path": path, "target": target });
        browser_response(browser_request_with_body(
            server.port, "POST", &format!("{base}/api/fs/mutate"), site,
            server.proxy_authorization(), &payload.to_string(),
        ))
    };
    let empty = Path::new("");
    let read_only = start_file_browser(original_backend(), &root).unwrap();
    let (page, base) = browser_session(read_only.port, read_only.proxy_authorization());
    let (header, body) = request(&read_only, &base, "same-origin", "create-file", &source, empty);
    assert!(header.starts_with("HTTP/1.1 200 "), "{header}\n{body}");
    assert_eq!(body, "false");
    assert!(!source.exists());
    drop(page);
    drop(read_only);

    let create_only = start_file_browser_with_writes(
        original_backend(), &root, FileWritePermissions { create: true, ..Default::default() },
    ).unwrap();
    let (page, base) = browser_session(create_only.port, create_only.proxy_authorization());
    let (_, created) = request(&create_only, &base, "same-origin", "create-file", &source, empty);
    assert_eq!(created, "true");
    let (_, renamed) = request(&create_only, &base, "same-origin", "rename", &source, &moved);
    assert_eq!(renamed, "false");
    assert!(source.exists() && !moved.exists());
    drop(page);
    drop(create_only);

    let server = start_file_browser_with_writes(
        original_backend(), &root,
        FileWritePermissions { create: true, copy: true, rename: true, delete: true },
    ).unwrap();
    let (page, base) = browser_session(server.port, server.proxy_authorization());
    std::fs::write(&source, "allowed").unwrap();
    let (header, _) = request(&server, &base, "cross-site", "delete-file", &source, empty);
    assert!(header.starts_with("HTTP/1.1 404 "));
    for (operation, path, target) in [
        ("copy", &source, &outside), ("copy", &outside, &copy),
        ("rename", &source, &outside), ("rename", &outside, &moved),
        ("create-directory", &directory, &copy), ("unrecognized", &source, &copy),
    ] {
        let (_, body) = request(&server, &base, "same-origin", operation, path, target);
        assert_eq!(body, "false", "{operation}");
    }
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "outside");
    assert_eq!(std::fs::read_to_string(&source).unwrap(), "allowed");
    assert!(!copy.exists() && !moved.exists() && !directory.exists());
    for (operation, path, target) in [
        ("create-directory", directory.as_path(), empty), ("copy", &source, &copy),
        ("rename", &copy, &moved), ("delete-directory", &directory, empty),
    ] {
        let (header, body) = request(&server, &base, "same-origin", operation, path, target);
        assert!(header.starts_with("HTTP/1.1 200 ") && header.contains("Cache-Control: no-store"));
        assert_eq!(body, "true", "{operation}");
    }
    assert_eq!(std::fs::read_to_string(&moved).unwrap(), "allowed");
    assert!(!copy.exists() && !directory.exists());
    let (_, body) = request(&server, &base, "same-origin", "delete-file", &moved, empty);
    assert_eq!(body, "true");
    assert!(!moved.exists());

    drop(page);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        // Probe with a non-mutating listing until the lease thread has closed the session.
        let read = format!("{base}/api/fs/home");
        let (header, _) = browser_response(browser_request(
            server.port, "GET", &read, "same-origin", server.proxy_authorization(),
        ));
        if header.starts_with("HTTP/1.1 404 ") { break; }
        assert!(Instant::now() < deadline, "closed page session was retained");
        std::thread::sleep(Duration::from_millis(25));
    }
    let (header, _) = request(&server, &base, "same-origin", "delete-file", &source, empty);
    assert!(header.starts_with("HTTP/1.1 404 "));
    assert!(source.exists());
}
