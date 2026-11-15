//! PLAN-971 B-PROOF-I/V (A batch): the permanent real-client/CLI LSP
//! roundtrip fixture — spawns the REAL `auto-lsp` binary and speaks
//! LSP/JSON-RPC (Content-Length framing) over stdio, no in-process
//! service shortcut.
//!
//! Oracle (frozen by the T-01 freeze record, PLAN-971 §5): a bad source
//! opened in the editor is diagnosed via `publishDiagnostics` for THAT
//! document (the original file) with a range inside the document; after
//! a full didChange to the fixed source the diagnostics disappear; an
//! unknown JSON-RPC method is a protocol-error response (never a hang
//! or a silent success); shutdown/exit terminates the server with 0.
//! A wrong binary (a non-LSP process) is refused by the client harness
//! with a NAMED failure — the roundtrip never treats it as a server.
//!
//! A-batch scope: this fixture locks the CLIENT/CLI protocol behavior;
//! the LSP analysis backend stays with its current owner (migration is
//! the B batch, T-08). Prerequisite: the binary is pre-built by the
//! test harness (`CARGO_BIN_EXE_auto-lsp`) — nothing compiles here.

use std::process::Stdio;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// A real LSP client over the spawned server's stdio.
struct LspCli {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    next_id: i64,
}

impl LspCli {
    fn spawn(exe: &str) -> Result<LspCli, String> {
        let mut child = Command::new(exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn `{exe}` failed: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "server stdin not piped".to_string())?;
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| "server stdout not piped".to_string())?,
        );
        Ok(LspCli {
            child,
            stdin,
            stdout,
            next_id: 0,
        })
    }

    /// Send one JSON-RPC message (Content-Length framing).
    async fn send(&mut self, message: &Value) -> Result<(), String> {
        let body = serde_json::to_string(message).map_err(|e| e.to_string())?;
        self.stdin
            .write_all(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_bytes())
            .await
            .map_err(|e| format!("write to server failed: {e}"))
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await?;
        self.wait_for_response(id).await
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
            .await
    }

    /// Read one framed message (headers + body). A closed/EOF stream is
    /// a NAMED failure, never a hang (the read is deadline-bounded).
    async fn read_message(&mut self) -> Result<Value, String> {
        let read = async {
            let mut content_length: Option<usize> = None;
            loop {
                let mut line = String::new();
                let n = self
                    .stdout
                    .read_line(&mut line)
                    .await
                    .map_err(|e| e.to_string())?;
                if n == 0 {
                    return Err("server closed the connection".to_string());
                }
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                if let Some(len) = line
                    .strip_prefix("Content-Length:")
                    .map(|v| v.trim().to_string())
                {
                    content_length = len.parse::<usize>().ok();
                }
            }
            let len = content_length.ok_or_else(|| "frame without Content-Length".to_string())?;
            let mut body = vec![0u8; len];
            self.stdout
                .read_exact(&mut body)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::from_slice(&body).map_err(|e| format!("malformed frame body: {e}"))
        };
        timeout(READ_TIMEOUT, read)
            .await
            .map_err(|_| "timed out waiting for a server message".to_string())?
    }

    /// Read messages until the response with `id` arrives (skipping
    /// server-initiated notifications such as logMessage; every read is
    /// individually deadline-bounded).
    async fn wait_for_response(&mut self, id: i64) -> Result<Value, String> {
        loop {
            let message = self.read_message().await?;
            if message.get("id").and_then(Value::as_i64) == Some(id) {
                return Ok(message);
            }
        }
    }

    /// Read messages until the next `textDocument/publishDiagnostics`
    /// for `uri`.
    async fn wait_for_diagnostics(&mut self, uri: &str) -> Result<Vec<Value>, String> {
        loop {
            let message = self.read_message().await?;
            if message.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
                && message["params"]["uri"].as_str() == Some(uri)
            {
                return message["params"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .ok_or_else(|| "diagnostics is not an array".to_string());
            }
        }
    }
}

const BAD_SOURCE: &str = "fn main( int { return 2 + }";
const FIXED_SOURCE: &str = "fn main() int { return 2 + 3 * 4 }";

#[tokio::test]
async fn p971_lsp_cli_real_binary_roundtrip() {
    let started = std::time::Instant::now();
    let exe = env!("CARGO_BIN_EXE_auto-lsp");
    let mut cli = LspCli::spawn(exe).expect("spawn the real auto-lsp binary");
    let uri = "file:///p971/proof/main.at";

    // initialize handshake with the real server.
    let init = cli
        .request(
            "initialize",
            json!({
                "processId": null,
                "rootUri": null,
                "capabilities": {},
            }),
        )
        .await
        .expect("initialize response");
    assert!(init.get("result").is_some(), "init must succeed: {init}");
    assert!(
        init["result"]["capabilities"].is_object(),
        "init result carries capabilities: {init}"
    );
    cli.notify("initialized", json!({}))
        .await
        .expect("initialized notify");

    // Bad source → diagnostics for THAT document, range inside the text.
    cli.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "auto",
                "version": 1,
                "text": BAD_SOURCE,
            }
        }),
    )
    .await
    .expect("didOpen");
    let diags = cli
        .wait_for_diagnostics(uri)
        .await
        .expect("diagnostics for the bad source");
    assert!(
        !diags.is_empty(),
        "a bad source must be diagnosed (bad: {BAD_SOURCE:?})"
    );
    for diagnostic in &diags {
        let range = &diagnostic["range"];
        let start = range["start"]["character"].as_u64().unwrap_or(u64::MAX);
        let end = range["end"]["character"].as_u64().unwrap_or(0);
        let (start_line, end_line) = (
            range["start"]["line"].as_u64().unwrap_or(u64::MAX),
            range["end"]["line"].as_u64().unwrap_or(u64::MAX),
        );
        // The bad source is single-line ASCII: every diagnostic must sit
        // on line 0 with a character range inside the document.
        assert!(
            start_line == 0
                && end_line == 0
                && end <= BAD_SOURCE.len() as u64
                && start <= end,
            "diagnostic range {range} must stay inside the opened document"
        );
    }

    // Recovery: full didChange with the fixed source → diagnostics clear.
    cli.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": 2 },
            "contentChanges": [{ "text": FIXED_SOURCE }],
        }),
    )
    .await
    .expect("didChange");
    let cleared = cli
        .wait_for_diagnostics(uri)
        .await
        .expect("diagnostics after recovery");
    assert!(
        cleared.is_empty(),
        "diagnostics must disappear after the fix, got {cleared:?}"
    );

    // Unknown method: a protocol-error response, never a hang/success.
    let unknown = cli
        .request("p971/nonexistent", json!({}))
        .await
        .expect("a response for the unknown method");
    assert!(
        unknown.get("error").is_some(),
        "unknown method must be a JSON-RPC error: {unknown}"
    );

    // Clean shutdown: null params, null result, then exit → code 0.
    cli.next_id += 1;
    let shutdown_id = cli.next_id;
    cli.send(&json!({"jsonrpc": "2.0", "id": shutdown_id, "method": "shutdown", "params": null}))
        .await
        .expect("send shutdown");
    let shutdown = cli
        .wait_for_response(shutdown_id)
        .await
        .expect("shutdown response");
    assert!(
        shutdown.get("result").is_some(),
        "shutdown result: {shutdown}"
    );
    cli.notify("exit", json!({})).await.expect("exit notify");
    let status = timeout(READ_TIMEOUT, cli.child.wait())
        .await
        .expect("server exits promptly after exit")
        .expect("wait server");
    assert_eq!(status.code(), Some(0), "clean exit code");

    eprintln!(
        "p971_lsp_cli_real_binary_roundtrip: {:.0}ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
}

/// The wrong-binary negative control: a non-LSP process is a NAMED
/// client-side failure (EOF/garbage), never a treated-as-server hang.
#[tokio::test]
async fn p971_lsp_cli_wrong_binary_is_a_named_failure() {
    // A process that is definitely not an LSP server: exits immediately.
    let mut cli = LspCli::spawn("cmd")
        .or_else(|_| LspCli::spawn("sh"))
        .expect("spawn a non-LSP process");
    let outcome = cli
        .request(
            "initialize",
            json!({"processId": null, "rootUri": null, "capabilities": {}}),
        )
        .await;
    assert!(
        outcome.is_err(),
        "a non-LSP process must be a named refusal, got {outcome:?}"
    );
    let message = outcome.unwrap_err();
    assert!(
        !message.is_empty(),
        "the refusal must carry a reason (closed/garbled stream)"
    );
}
