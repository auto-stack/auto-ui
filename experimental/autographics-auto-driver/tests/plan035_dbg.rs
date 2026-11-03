//! throwaway g3dbg boot probe (deleted after diagnosis)
use autographics_auto_driver::authoring::{assets::Registry, host::Host, play::Action, storage::Storage};
use std::path::PathBuf;

#[test]
fn g3dbg_boots() {
    let gen = PathBuf::from("D:/autostack/plan-artifacts/plan-035/auto-ui/g3dbg-out");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/plan035-dbg");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::copy(gen.join("scene.agscene"), dir.join("scene.agscene")).unwrap();
    let scene = std::fs::read_to_string(dir.join("scene.agscene")).unwrap();
    let sha = scene.split("\"sha256\": \"").nth(1).and_then(|r| r.split('"').next()).unwrap().to_string();
    std::fs::copy(gen.join("world.at"), dir.join("scripts").join(format!("{sha}.at"))).unwrap();
    for e in std::fs::read_dir(gen.join("assets")).unwrap() {
        let e = e.unwrap();
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::copy(e.path(), dir.join("assets").join(e.file_name())).unwrap();
    }
    for e in std::fs::read_dir(gen.join("closure")).unwrap() {
        let e = e.unwrap();
        std::fs::copy(e.path(), dir.join("scripts").join(e.file_name())).unwrap();
    }
    let (doc, registry, code, _) = Storage::load_project(&dir.join("scene.agscene"), &Registry::new()).unwrap();
    let mut h = Host::with_project(doc, registry, code).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    {
        let p = h.play.as_mut().unwrap();
        let _ = p.resume();
        let _ = p.dispatch(Action::default());
    }
    h.pause_play("");
    let err = h.play.as_ref().unwrap().error.clone().unwrap_or_default();
    assert!(err.is_empty(), "g3dbg tick1: {err}");
}
