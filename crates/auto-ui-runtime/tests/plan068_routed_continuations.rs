//! Real HTTP park/resume, instance state and production route/view consumers.
//! Root read_all_state is deliberately not the child-result oracle.
use auto_ui_runtime::ui::dynamic::DynamicComponent;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct ReplyServer {
    url: String,
    requests: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ReplyServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let counted = requests.clone();
        let stopping = stop.clone();
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // Accepted sockets inherit nonblocking mode on Windows.
                        stream.set_nonblocking(false).unwrap();
                        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                        let mut raw = [0; 4096];
                        let length = stream.read(&mut raw).unwrap();
                        assert!(String::from_utf8_lossy(&raw[..length]).starts_with("GET /reply "));
                        counted.fetch_add(1, Ordering::SeqCst);
                        // Force a genuine parked segment before making the reply ready.
                        thread::sleep(Duration::from_millis(80));
                        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 7\r\nConnection: close\r\n\r\n\"reply\"").unwrap();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(3));
                    }
                    Err(error) => panic!("HTTP listener: {error}"),
                }
            }
        });
        Self { url, requests, stop, worker: Some(worker) }
    }
}

impl Drop for ReplyServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Err(error) = self.worker.take().unwrap().join() {
            if thread::panicking() {
                eprintln!("HTTP worker also failed while the assertion unwound: {error:?}");
            } else {
                std::panic::resume_unwind(error);
            }
        }
    }
}

fn fixture(server: &ReplyServer) -> (tempfile::TempDir, DynamicComponent) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/plan068_routes/src/front");
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("pages")).unwrap();
    for relative in ["app.at", "pages/status_page.at", "pages/done_page.at"] {
        let text = std::fs::read_to_string(source.join(relative)).unwrap();
        std::fs::write(directory.path().join(relative), text.replace("__P068_URL__", &server.url)).unwrap();
    }
    let app = directory.path().join("app.at");
    let mut component = auto_ui_runtime::build_dynamic_component(
        &std::fs::read_to_string(&app).unwrap(), app.to_str()).unwrap();
    component.set_route("/");
    render(&mut component);
    component.drive_scheduler_to_quiescence(32);
    (directory, component)
}

fn render(component: &mut DynamicComponent) -> String {
    format!("{:?}", component.view_with_debug_gated(false).0)
}

fn identity(component: &DynamicComponent, widget: &str) -> String {
    let identities: Vec<_> = component.bridge().child_instance_identities().into_iter()
        .filter(|id| id.contains(widget)).collect();
    assert_eq!(identities.len(), 1, "live {widget}: {identities:?}");
    identities[0].clone()
}

fn finish_http(component: &mut DynamicComponent, server: &ReplyServer) {
    assert!(component.bridge().has_parked_io_tasks(), "not a real HTTP suspension");
    let deadline = Instant::now() + Duration::from_secs(5);
    while component.bridge().has_parked_tasks() && Instant::now() < deadline {
        component.poll_parked_resumes();
        thread::sleep(Duration::from_millis(3));
    }
    assert!(!component.bridge().has_parked_tasks(), "HTTP continuation did not settle");
    assert_eq!(server.requests.load(Ordering::SeqCst), 1);
}

#[test]
fn root_http_result_is_the_accepted_control() {
    let server = ReplyServer::start();
    let (_directory, mut component) = fixture(&server);
    component.on_with_input_for_instance("App", "RootLoad", None, None);
    finish_http(&mut component, &server);
    assert!(render(&mut component).contains("root completed"));
}

#[test]
fn routed_http_result_reaches_own_state_and_conditional_view() {
    let server = ReplyServer::start();
    let (_directory, mut component) = fixture(&server);
    let page = identity(&component, "StatusPage");
    assert!(!render(&mut component).contains("Result summary"));
    component.on_with_input_for_instance("StatusPage", "Load", None, Some(&page));
    finish_http(&mut component, &server);
    assert_eq!(component.bridge().read_instance_state(&page, "result").unwrap().as_str(),
        "success\nfailure\ncancelled");
    let visible = render(&mut component);
    assert!(visible.contains("Result summary") && visible.contains("cancelled"), "{visible}");
}

fn assert_destination(component: &mut DynamicComponent) {
    assert_eq!(component.bridge().read_state("__current_route").unwrap().as_str(), "/done/42");
    render(component);
    component.drive_scheduler_to_quiescence(32);
    let destination = identity(component, "DonePage");
    assert_eq!(component.bridge().read_instance_state(&destination, "seen").unwrap().as_str(), "42");
    assert!(render(component).contains("Destination"));
}

#[test]
fn child_sync_navigation_writes_its_app_root() {
    let server = ReplyServer::start();
    let (_directory, mut component) = fixture(&server);
    let page = identity(&component, "StatusPage");
    component.on_with_input_for_instance("StatusPage", "PushSync", None, Some(&page));
    assert_destination(&mut component);
    assert_eq!(server.requests.load(Ordering::SeqCst), 0);
}

#[test]
fn child_http_navigation_publishes_route_and_params() {
    let server = ReplyServer::start();
    let (_directory, mut component) = fixture(&server);
    let page = identity(&component, "StatusPage");
    component.on_with_input_for_instance("StatusPage", "Push", None, Some(&page));
    finish_http(&mut component, &server);
    assert_destination(&mut component);
}

#[test]
fn child_http_back_consumes_the_owning_apps_history() {
    let server = ReplyServer::start();
    let (_directory, mut component) = fixture(&server);
    component.set_route("/done/42");
    render(&mut component);
    component.drive_scheduler_to_quiescence(32);
    let page = identity(&component, "DonePage");
    component.on_with_input_for_instance("DonePage", "Back", None, Some(&page));
    finish_http(&mut component, &server);
    assert_eq!(component.bridge().read_state("__current_route").unwrap().as_str(), "/");
    assert!(render(&mut component).contains("Load results"));
}
