use auto_ui_runtime::ui::file_dialog::{
    DialogConfig, DialogEvent, DialogModel, Entry, Failure, FileChoice, FileSystem, Filter,
    Listing, Navigation, SelectionMode,
};
use std::collections::{HashMap, HashSet};
use std::sync::{atomic::AtomicBool, mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

fn settle(model: &mut DialogModel) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while model.projection().loading {
        model.poll();
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn config(path: &std::path::Path) -> DialogConfig {
    DialogConfig {
        initial_directory: path.to_str().unwrap().into(),
        selection_mode: SelectionMode::MultipleFiles,
        filters: vec![
            Filter {
                id: "books".into(),
                label: "Books".into(),
                extensions: vec!["txt".into(), "md".into()],
            },
            Filter {
                id: "images".into(),
                label: "Images".into(),
                extensions: vec!["png".into()],
            },
        ],
        active_filter_id: "books".into(),
        show_hidden: false,
    }
}

fn select(model: &mut DialogModel, name: &str) -> String {
    let row = model
        .projection()
        .entries
        .into_iter()
        .find(|e| e.name == name)
        .unwrap();
    assert!(model.toggle(&row.id));
    row.id
}

#[test]
fn exact_filter_sets_navigation_failure_and_direct_file_preselection() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("subfolder")).unwrap();
    for name in ["A.TXT", "notes.md", "cover.PNG", ".secret.txt"] {
        std::fs::write(temp.path().join(name), name).unwrap();
    }
    let mut model = DialogModel::default();
    assert!(model.open(config(temp.path())));
    settle(&mut model);
    assert_eq!(
        model
            .projection()
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["subfolder", "A.TXT", "notes.md"]
    );
    let old = select(&mut model, "A.TXT");
    let current = model.projection().current_directory;
    model.navigate("absent".into());
    settle(&mut model);
    let failed = model.projection();
    assert_eq!(failed.current_directory, current);
    assert_eq!(
        failed.selected_count, 1,
        "failed navigation preserves selection"
    );
    assert_eq!(failed.error.unwrap().code, "not_found");
    assert!(
        model.toggle(&old),
        "previous valid row remains valid after failed navigation"
    );
    assert_eq!(model.projection().selected_count, 0);
    select(&mut model, "notes.md");
    model.set_name_query("a.t".into());
    assert_eq!(model.projection().selected_count, 0);
    assert!(
        !model.toggle(&old),
        "filter changes invalidate captured row actions"
    );
    assert_eq!(
        model
            .projection()
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["subfolder", "A.TXT"]
    );
    model.set_name_query(String::new());
    model.set_filter("images");
    assert_eq!(
        model
            .projection()
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["subfolder", "cover.PNG"]
    );
    model.navigate(temp.path().join("A.TXT").to_str().unwrap().into());
    settle(&mut model);
    assert_eq!(
        model.projection().selected_count,
        0,
        "full file path cannot bypass the active type filter"
    );
    model.set_filter("books");
    model.navigate(temp.path().join("A.TXT").to_str().unwrap().into());
    settle(&mut model);
    assert_eq!(model.projection().selected_count, 1);
    assert!(
        model
            .take_events()
            .iter()
            .all(|event| !matches!(event, DialogEvent::Accepted(_))),
        "direct file path still requires confirmation"
    );
    model.set_show_hidden(true);
    assert_eq!(model.projection().selected_count, 0);
    assert!(model
        .projection()
        .entries
        .iter()
        .any(|e| e.name == ".secret.txt"));
    let dir = model
        .projection()
        .entries
        .iter()
        .find(|e| e.is_dir)
        .unwrap()
        .id
        .clone();
    assert!(model.enter(&dir));
    settle(&mut model);
    assert_eq!(model.projection().total_entries, 0);
    assert_eq!(model.projection().visible_entries, 0);
    assert!(
        model.projection().error.is_none(),
        "empty directory is a successful listing"
    );
    model.up();
    settle(&mut model);
    assert_eq!(model.projection().current_directory, current);
    model.set_name_query("no match".into());
    assert_eq!(
        model.projection().entries.len(),
        1,
        "directories remain navigable under name filters"
    );
    assert_eq!(
        model.projection().total_entries,
        5,
        "no match must not masquerade as an empty directory"
    );
}

#[test]
fn validation_is_atomic_recoverable_and_results_use_list_order() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["a.txt", "b.md", "c.txt"] {
        std::fs::write(temp.path().join(name), name).unwrap();
    }
    let mut model = DialogModel::default();
    model.open(config(temp.path()));
    settle(&mut model);
    select(&mut model, "b.md");
    select(&mut model, "a.txt");
    std::fs::remove_file(temp.path().join("b.md")).unwrap();
    assert!(model.confirm());
    assert!(
        !model.confirm(),
        "repeated confirm cannot start a second transaction"
    );
    settle(&mut model);
    assert!(model.projection().active);
    assert_eq!(model.projection().selected_count, 2);
    assert_eq!(model.projection().error.unwrap().code, "not_found");
    let events = model.take_events();
    assert!(
        matches!(events.as_slice(), [DialogEvent::Error(_)]),
        "never accept only the readable subset: {events:?}"
    );
    std::fs::write(temp.path().join("b.md"), b"restored").unwrap();
    assert!(model.confirm());
    settle(&mut model);
    let events = model.take_events();
    let [DialogEvent::Accepted(files)] = events.as_slice() else {
        panic!("expected one accepted event: {events:?}");
    };
    assert_eq!(
        files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        ["a.txt", "b.md"]
    );
    for file in files {
        assert_eq!(
            std::fs::read(&file.path).unwrap(),
            std::fs::read(temp.path().join(&file.name)).unwrap()
        );
    }
    assert!(
        !model.cancel(),
        "accepted and cancelled are mutually exclusive"
    );
    assert!(!model.confirm());
    assert!(model.take_events().is_empty());
}

#[test]
fn single_selection_clear_refresh_and_reopen_reject_old_rows() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("one.txt"), "one").unwrap();
    std::fs::write(temp.path().join("two.txt"), "two").unwrap();
    let mut cfg = config(temp.path());
    cfg.selection_mode = SelectionMode::SingleFile;
    let mut model = DialogModel::default();
    model.open(cfg.clone());
    settle(&mut model);
    let old = select(&mut model, "one.txt");
    select(&mut model, "two.txt");
    assert_eq!(model.projection().selected_count, 1);
    assert!(!model.projection().entries[0].selected);
    model.clear_selection();
    assert!(!model.projection().can_accept);
    select(&mut model, "one.txt");
    model.refresh();
    settle(&mut model);
    assert_eq!(model.projection().selected_count, 0);
    assert!(!model.toggle(&old));
    let session = model.projection().session;
    assert!(
        !model.open(cfg.clone()),
        "duplicate open must preserve the current session"
    );
    assert_eq!(model.projection().session, session);
    assert!(model.cancel());
    assert!(!model.cancel());
    assert_eq!(model.take_events(), [DialogEvent::Cancelled]);
    model.open(cfg);
    settle(&mut model);
    assert_ne!(model.projection().session, session);
    assert!(!model.toggle(&old));
    assert_eq!(model.projection().selected_count, 0);
}

#[test]
fn thousand_entry_directory_is_not_truncated_and_tail_is_selectable() {
    let temp = tempfile::tempdir().unwrap();
    let expected = (0..1003)
        .map(|n| format!("中文 long book {n:04}.TXT"))
        .collect::<HashSet<_>>();
    for name in &expected {
        std::fs::write(temp.path().join(name), name).unwrap();
    }
    let mut model = DialogModel::default();
    let start = Instant::now();
    model.open(config(temp.path()));
    settle(&mut model);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "local 1000-entry model/provider response budget exceeded"
    );
    assert_eq!(model.projection().page_count, 11);
    assert_eq!(model.projection().total_entries, 1003);
    let mut seen = HashSet::new();
    for page in 0..11 {
        model.set_page(page);
        let projection = model.projection();
        assert!(projection.entries.len() <= 100);
        for row in projection.entries {
            assert!(seen.insert(row.name));
        }
    }
    assert_eq!(seen, expected);
    let last = model.projection().entries.last().unwrap().name.clone();
    assert_eq!(last, "中文 long book 1002.TXT");
    select(&mut model, &last);
    model.confirm();
    settle(&mut model);
    let events = model.take_events();
    let [DialogEvent::Accepted(files)] = events.as_slice() else {
        panic!("tail selection failed: {events:?}");
    };
    assert_eq!(files[0].name, last);
}

struct ControlledFs {
    gates: Mutex<HashMap<String, mpsc::Receiver<()>>>,
    started: mpsc::Sender<String>,
    finished: mpsc::Sender<String>,
}
impl FileSystem for ControlledFs {
    fn navigate(
        &self,
        input: &str,
        _current: Option<&str>,
        _cancel: &AtomicBool,
    ) -> Result<Navigation, Failure> {
        self.started.send(input.into()).unwrap();
        let gate = self.gates.lock().unwrap().remove(input);
        if let Some(gate) = gate {
            gate.recv().unwrap();
        }
        self.finished.send(input.into()).unwrap();
        let path = format!("{input}/same.txt");
        Ok(Navigation {
            listing: Listing {
                directory: input.into(),
                parent: None,
                entries: vec![Entry {
                    name: "same.txt".into(),
                    path,
                    is_dir: false,
                    hidden: false,
                    size: 4,
                }],
            },
            preselect: None,
            home: "/".into(),
            roots: vec!["/".into()],
        })
    }
    fn validate(
        &self,
        files: &[FileChoice],
        _cancel: &AtomicBool,
    ) -> Result<Vec<FileChoice>, Failure> {
        Ok(files.to_vec())
    }
}

#[test]
fn slow_old_directory_and_closed_session_never_overwrite_current_state() {
    let (allow_a, gate_a) = mpsc::channel();
    let (allow_c, gate_c) = mpsc::channel();
    let (started, starts) = mpsc::channel();
    let (finished, finishes) = mpsc::channel();
    let fs = Arc::new(ControlledFs {
        gates: Mutex::new(HashMap::from([
            ("/slow-a".into(), gate_a),
            ("/slow-c".into(), gate_c),
        ])),
        started,
        finished,
    });
    let mut model = DialogModel::with_fs(fs);
    model.open(DialogConfig {
        initial_directory: "/slow-a".into(),
        ..Default::default()
    });
    assert_eq!(
        starts.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/slow-a"
    );
    model.navigate("/fast-b".into());
    settle(&mut model);
    assert_eq!(
        starts.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/fast-b"
    );
    assert_eq!(
        finishes.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/fast-b"
    );
    let current_row = model.projection().entries[0].id.clone();
    allow_a.send(()).unwrap();
    assert_eq!(
        finishes.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/slow-a"
    );
    model.poll();
    assert_eq!(model.projection().current_directory, "/fast-b");
    assert!(
        model.toggle(&current_row),
        "positive current-generation control remains actionable"
    );
    model.navigate("/slow-c".into());
    assert_eq!(
        starts.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/slow-c"
    );
    model.cancel();
    assert_eq!(model.take_events(), [DialogEvent::Cancelled]);
    model.open(DialogConfig {
        initial_directory: "/new-session".into(),
        ..Default::default()
    });
    settle(&mut model);
    // Wait for the new positive request before releasing the abandoned worker.
    assert_eq!(
        starts.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/new-session"
    );
    assert_eq!(
        finishes.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/new-session"
    );
    allow_c.send(()).unwrap();
    assert_eq!(
        finishes.recv_timeout(Duration::from_secs(2)).unwrap(),
        "/slow-c"
    );
    model.poll();
    assert_eq!(model.projection().current_directory, "/new-session");
    assert!(
        !model.toggle(&current_row),
        "old session row must not select the same row number in a new session"
    );
    assert!(model.take_events().is_empty());
}

#[test]
fn empty_path_and_missing_initial_folder_are_named_recoverable_errors() {
    let temp = tempfile::tempdir().unwrap();
    let mut model = DialogModel::default();
    model.open(config(&temp.path().join("absent")));
    settle(&mut model);
    assert_eq!(model.projection().error.unwrap().code, "not_found");
    assert!(
        !model.projection().home.is_empty(),
        "home recovery is available even when initial navigation fails"
    );
    model.navigate(String::new());
    settle(&mut model);
    assert_eq!(model.projection().error.unwrap().code, "invalid_path");
    model.navigate(temp.path().to_str().unwrap().into());
    settle(&mut model);
    assert!(model.projection().error.is_none());
    assert!(model.projection().active);
    assert_eq!(model.projection().total_entries, 0);
    assert!(!model.confirm());
}

#[test]
fn row_tokens_and_json_paths_are_isolated_between_two_models() {
    let temp = tempfile::tempdir().unwrap();
    let name = "中文 空格.txt";
    std::fs::write(temp.path().join(name), "bytes").unwrap();
    let mut a = DialogModel::default();
    let mut b = DialogModel::default();
    a.open(config(temp.path()));
    b.open(config(temp.path()));
    settle(&mut a);
    settle(&mut b);
    let old = a.projection().entries[0].id.clone();
    assert!(!b.toggle(&old));
    select(&mut a, name);
    assert_eq!(b.projection().selected_count, 0);
    let projection = a.projection();
    let encoded = serde_json::to_string(&projection).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded["entries"][0]["name"], name);
    assert_eq!(decoded["entries"][0]["path"], projection.entries[0].path);
    select(&mut b, name);
    assert!(b.confirm());
    settle(&mut b);
    assert!(matches!(
        b.take_events().as_slice(),
        [DialogEvent::Accepted(_)]
    ));
    assert!(a.projection().active);
    assert_eq!(a.projection().selected_count, 1);
}

struct ValidationGate {
    gate: Mutex<mpsc::Receiver<()>>,
    started: mpsc::Sender<()>,
    finished: mpsc::Sender<()>,
}

impl FileSystem for ValidationGate {
    fn navigate(
        &self,
        input: &str,
        current: Option<&str>,
        cancel: &AtomicBool,
    ) -> Result<Navigation, Failure> {
        auto_ui_runtime::ui::file_dialog::LocalFileSystem.navigate(input, current, cancel)
    }
    fn validate(
        &self,
        files: &[FileChoice],
        _cancel: &AtomicBool,
    ) -> Result<Vec<FileChoice>, Failure> {
        self.started.send(()).unwrap();
        self.gate.lock().unwrap().recv().unwrap();
        self.finished.send(()).unwrap();
        Ok(files.to_vec())
    }
}

#[test]
fn late_validation_cannot_accept_after_filter_change_cancel_or_owner_drop() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("one.txt"), "one").unwrap();
    for transition in ["filter", "cancel", "drop"] {
        let (allow, gate) = mpsc::channel();
        let (started, starts) = mpsc::channel();
        let (finished, finishes) = mpsc::channel();
        let fs = Arc::new(ValidationGate {
            gate: Mutex::new(gate),
            started,
            finished,
        });
        let mut model = DialogModel::with_fs(fs);
        model.open(config(temp.path()));
        settle(&mut model);
        select(&mut model, "one.txt");
        model.confirm();
        starts.recv_timeout(Duration::from_secs(2)).unwrap();
        match transition {
            "filter" => {
                model.set_filter("images");
                allow.send(()).unwrap();
                finishes.recv_timeout(Duration::from_secs(2)).unwrap();
                model.poll();
                assert!(model.take_events().is_empty());
                assert!(model.projection().active);
                assert_eq!(model.projection().selected_count, 0);
            }
            "cancel" => {
                model.cancel();
                assert_eq!(model.take_events(), [DialogEvent::Cancelled]);
                model.open(config(temp.path()));
                settle(&mut model);
                allow.send(()).unwrap();
                finishes.recv_timeout(Duration::from_secs(2)).unwrap();
                model.poll();
                assert!(model.take_events().is_empty());
                assert!(model.projection().active);
                assert_eq!(model.projection().selected_count, 0);
            }
            "drop" => {
                drop(model);
                allow.send(()).unwrap();
                finishes.recv_timeout(Duration::from_secs(2)).unwrap();
                // The worker holds no owner/VM reference and exits without a
                // receiver. A fresh owner accepts its own normal transaction.
                let mut fresh = DialogModel::default();
                fresh.open(config(temp.path()));
                settle(&mut fresh);
                select(&mut fresh, "one.txt");
                fresh.confirm();
                settle(&mut fresh);
                assert!(matches!(
                    fresh.take_events().as_slice(),
                    [DialogEvent::Accepted(_)]
                ));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn filter_change_while_navigating_preserves_the_current_request() {
    let (allow, gate) = mpsc::channel();
    let (started, starts) = mpsc::channel();
    let (finished, finishes) = mpsc::channel();
    let fs = Arc::new(ControlledFs {
        gates: Mutex::new(HashMap::from([("/wait".into(), gate)])),
        started,
        finished,
    });
    let mut model = DialogModel::with_fs(fs);
    model.open(DialogConfig {
        initial_directory: "/wait".into(),
        ..Default::default()
    });
    starts.recv_timeout(Duration::from_secs(2)).unwrap();
    model.set_name_query("same".into());
    model.set_show_hidden(true);
    model.clear_selection();
    assert!(
        model.projection().loading,
        "presentation changes must not abandon navigation"
    );
    allow.send(()).unwrap();
    finishes.recv_timeout(Duration::from_secs(2)).unwrap();
    settle(&mut model);
    assert_eq!(model.projection().entries.len(), 1);
    assert_eq!(model.projection().name_query, "same");
}

#[cfg(windows)]
#[test]
fn windows_hidden_and_system_attributes_follow_the_hidden_switch() {
    use std::os::windows::{ffi::OsStrExt, fs::MetadataExt};
    #[link(name = "kernel32")]
    extern "system" {
        fn SetFileAttributesW(path: *const u16, attributes: u32) -> i32;
    }
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("visible.txt"), "visible").unwrap();
    for (name, attribute) in [("hidden.txt", 2), ("system.txt", 4)] {
        let path = temp.path().join(name);
        std::fs::write(&path, name).unwrap();
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        assert_ne!(unsafe { SetFileAttributesW(wide.as_ptr(), attribute) }, 0);
        assert_ne!(
            std::fs::metadata(&path).unwrap().file_attributes() & attribute,
            0
        );
    }
    let mut model = DialogModel::default();
    model.open(config(temp.path()));
    settle(&mut model);
    assert_eq!(
        model
            .projection()
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["visible.txt"]
    );
    model.set_show_hidden(true);
    assert_eq!(model.projection().visible_entries, 3);
}

#[cfg(unix)]
#[test]
fn unix_non_utf8_names_broken_links_and_real_permissions_are_named() {
    use auto_ui_runtime::ui::file_dialog::LocalFileSystem;
    use std::os::unix::{
        ffi::OsStringExt,
        fs::{symlink, PermissionsExt},
    };
    let temp = tempfile::tempdir().unwrap();
    let invalid = temp.path().join(std::ffi::OsString::from_vec(vec![0xff]));
    std::fs::write(&invalid, "bad name").unwrap();
    let error = LocalFileSystem
        .navigate(temp.path().to_str().unwrap(), None, &AtomicBool::new(false))
        .err()
        .unwrap();
    assert_eq!(error.code, "unsupported_path");
    std::fs::remove_file(invalid).unwrap();
    symlink(temp.path().join("missing"), temp.path().join("broken.txt")).unwrap();
    let error = LocalFileSystem
        .navigate(temp.path().to_str().unwrap(), None, &AtomicBool::new(false))
        .err()
        .unwrap();
    assert_eq!(error.code, "not_found");
    std::fs::remove_file(temp.path().join("broken.txt")).unwrap();
    let path = temp.path().join("denied.txt");
    std::fs::write(&path, "denied").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0)).unwrap();
    // Require an OS rejection. A root runner cannot provide this oracle.
    let denied = std::fs::File::open(&path).unwrap_err();
    assert_eq!(
        denied.kind(),
        std::io::ErrorKind::PermissionDenied,
        "run this test as an unprivileged user"
    );
    let error = LocalFileSystem
        .validate(
            &[FileChoice {
                name: "denied.txt".into(),
                path: path.to_str().unwrap().into(),
            }],
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert_eq!(error.code, "permission_denied");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
