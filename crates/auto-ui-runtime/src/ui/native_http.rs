//! Native `Http.*` facade for the backend profile captured by generation.
//! Relative paths belong to that app; absolute URLs keep their own origin.
//! Resolving a request never mutates the desktop process environment.

fn resolve_url(url: &str, base: &str) -> String {
    if url.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), url)
    } else {
        url.to_owned()
    }
}

fn response_body(response: (u32, String)) -> String {
    if response.0 == 0 {
        // The UI's existing try/catch reports a connection failure. Never put
        // a query token, request body or transport error payload in the panic.
        panic!("native HTTP backend unavailable");
    }
    response.1
}

#[doc(hidden)]
pub fn post(url: &str, body: &str, base: &str) -> String {
    response_body(a2r_std::http::post_sync(&resolve_url(url, base), body, ""))
}

#[doc(hidden)]
pub fn get(url: &str, base: &str) -> String {
    response_body(a2r_std::http::get_sync(&resolve_url(url, base)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    #[test]
    fn p060_native_http_profile_routes_real_requests_and_preserves_json() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let worker = std::thread::spawn(move || {
            for (method, path, body, status, reply) in [
                (
                    "POST",
                    "/api/register",
                    r#"{"name":"中文+54.25"}"#,
                    "200 OK",
                    r#"{"ok":true,"id":"u-54"}"#,
                ),
                (
                    "GET",
                    "/api/messages?conversation_id=c-54",
                    "",
                    "200 OK",
                    "[]",
                ),
                (
                    "POST",
                    "/api/login",
                    "{}",
                    "403 Forbidden",
                    r#"{"ok":false,"err":"unauthorized"}"#,
                ),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut chunk = [0; 1024];
                let header_end = loop {
                    let n = stream.read(&mut chunk).unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&chunk[..n]);
                    if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break index + 4;
                    }
                };
                let headers = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
                assert!(headers.starts_with(&format!("{method} {path} HTTP/1.1\r\n")));
                let len = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                while bytes.len() < header_end + len {
                    let n = stream.read(&mut chunk).unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&chunk[..n]);
                }
                assert_eq!(&bytes[header_end..header_end + len], body.as_bytes());
                write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}", reply.len()).unwrap();
            }
        });
        let registered: serde_json::Value =
            serde_json::from_str(&post("/api/register", r#"{"name":"中文+54.25"}"#, &base))
                .unwrap();
        assert_eq!(registered["id"], "u-54");
        // An absolute URL must not be redirected to the configured app base.
        assert_eq!(
            get(
                &format!("{base}/api/messages?conversation_id=c-54"),
                "http://invalid.invalid"
            ),
            "[]"
        );
        let rejected: serde_json::Value =
            serde_json::from_str(&post("/api/login", "{}", &base)).unwrap();
        assert_eq!(rejected["ok"], false);
        worker.join().unwrap();
    }

    #[test]
    fn p060_native_http_transport_failure_is_distinct_from_api_rejection() {
        let error = std::panic::catch_unwind(|| post("not-a-url", "{}", "")).unwrap_err();
        assert_eq!(
            error.downcast_ref::<&str>(),
            Some(&"native HTTP backend unavailable")
        );
    }
}
