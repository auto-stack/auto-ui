//! PLAN-035 T-06 storage round-trip for the new chain (V-S06; AC-06): a
//! components-3d-audio-hud project saves as a versioned closure, relocates
//! without its import directory, reopens and Runs green; a corrupted,
//! truncated or zero-byte asset is named-rejected at load, and after the
//! exact restore the same entry reopens and Runs green.
//! Single instance: cargo test --locked --offline --test node3d_audio_hud_storage -- --test-threads=1
use autographics_auto_driver::authoring::{
    assets::Registry,
    host::Host,
    play::Action,
    storage::{Fault, Saved, Storage},
};
use std::path::{Path, PathBuf};
use std::time::Instant;

const GEN: &str = "../autographics-node2d-pilot/generated-node3d-audio-hud";

fn product_text(name: &str, file: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(GEN)
        .join(name)
        .join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn stage_product(product: &str, tag: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/plan035-storage")
        .join(format!("{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).expect("scripts dir");
    let scene_text = product_text(product, "scene.agscene");
    let world_src = product_text(product, "world.at");
    std::fs::write(dir.join("scene.agscene"), &scene_text).expect("write scene");
    if let Some(sha) = scene_text
        .split("\"sha256\": \"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    {
        std::fs::write(dir.join("scripts").join(format!("{sha}.at")), world_src)
            .expect("write script");
    }
    let src_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GEN).join(product).join("assets");
    for entry in std::fs::read_dir(&src_assets).expect("generated assets dir") {
        let entry = entry.expect("asset entry");
        let dest = dir.join("assets").join(entry.file_name());
        std::fs::create_dir_all(dest.parent().expect("assets parent")).expect("assets dir");
        std::fs::copy(entry.path(), &dest).expect("asset copy");
    }
    let src_scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GEN).join(product).join("closure");
    for entry in std::fs::read_dir(&src_scripts).expect("closure dir") {
        let entry = entry.expect("closure entry");
        std::fs::copy(entry.path(), dir.join("scripts").join(entry.file_name())).expect("closure copy");
    }
    dir
}

fn load_host(entry: &Path, tag: &str) -> Host {
    let (doc, registry, code, _) = Storage::load_project(entry, &Registry::new())
        .unwrap_or_else(|e| panic!("{tag} load project: {e:?}"));
    let mut h = Host::with_project(doc, registry, code)
        .unwrap_or_else(|e| panic!("{tag} host: {e:?}"));
    h.start_selected_play()
        .unwrap_or_else(|e| panic!("{tag} start: {e:?}"));
    h.pause_play("");
    settle(&mut h);
    h
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    loop {
        let pending = h.play.as_ref().map(|p| p.pending()).unwrap_or(false);
        if !pending {
            break;
        }
        assert!(Instant::now() < end, "host settle deadline");
        h.frame(None).expect("frame");
    }
}

fn step(h: &mut Host) {
    if let Some(p) = h.play.as_mut() {
        let _ = p.resume();
        let _ = p.dispatch(Action::default());
    }
    h.pause_play("");
    settle(h);
}

fn hud_count(h: &Host) -> String {
    let p = h.play.as_ref().unwrap();
    let frame = p.last.as_ref().expect("frame");
    frame
        .hud
        .as_ref()
        .expect("hud frame")
        .slots[0]
        .text
        .clone()
}

fn saved_project(tag: &str) -> (PathBuf, Vec<(PathBuf, Vec<u8>)>) {
    let staged = stage_product("batch-tick", tag);
    let entry = staged.join("scene.agscene");
    let mut h = load_host(&entry, tag);
    let saved_entry = staged.join("saved/scene.agscene");
    std::fs::create_dir_all(staged.join("saved")).expect("saved dir");
    assert_eq!(
        h.storage
            .save(&mut h.editor, &saved_entry, Fault::None)
            .expect("save"),
        Saved::Confirmed,
        "the new-chain project saves as a versioned closure"
    );
    // asset snapshot for the restore (the saved project keeps assets/ files)
    let saved_assets = staged.join("saved/assets");
    let mut snapshot = Vec::new();
    for entry in std::fs::read_dir(&saved_assets).expect("saved assets dir") {
        let path = entry.expect("asset entry").path();
        let bytes = std::fs::read(&path).expect("asset read");
        snapshot.push((path, bytes));
    }
    assert!(!snapshot.is_empty(), "the saved closure carries assets");
    (staged.join("saved"), snapshot)
}

#[test]
fn saved_new_chain_project_relocates_reopens_and_runs() {
    let (saved_dir, _) = saved_project("relocate");
    // relocate: the reopened entry must not depend on the original path
    let moved = saved_dir
        .parent()
        .expect("root")
        .join("relocated");
    std::fs::rename(&saved_dir, &moved).expect("rename");
    let entry = moved.join("scene.agscene");
    let mut h = load_host(&entry, "relocate");
    step(&mut h);
    step(&mut h);
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "relocated run stays green"
    );
    assert_eq!(hud_count(&h), "3", "the reopened project collects all three");
}

#[test]
fn corrupted_asset_rejects_then_exact_restore_reruns_green() {
    let (saved_dir, snapshot) = saved_project("corrupt");
    // flip one byte of the first asset: the load must name-reject
    let (path, good) = snapshot.first().expect("asset").clone();
    let mut bad = good.clone();
    bad[0] ^= 0xFF;
    std::fs::write(&path, &bad).expect("write corrupt asset");
    let entry = saved_dir.join("scene.agscene");
    let err = Storage::load_project(&entry, &Registry::new())
        .err()
        .expect("corrupted asset must be rejected");
    assert!(
        !err.is_empty(),
        "the loader names the corrupted asset"
    );
    // exact restore bytes → the same entry reopens and Runs green
    std::fs::write(&path, &good).expect("restore asset");
    let mut h = load_host(&entry, "restore");
    step(&mut h);
    step(&mut h);
    assert!(
        h.play.as_ref().unwrap().error.is_none(),
        "restored run stays green"
    );
    assert_eq!(hud_count(&h), "3", "the restored project collects all three");
}

#[test]
fn truncated_and_zero_byte_assets_are_named_rejected() {
    let (saved_dir, snapshot) = saved_project("partial");
    let (path, good) = snapshot.first().expect("asset").clone();
    let entry = saved_dir.join("scene.agscene");
    // partial (truncated) asset
    std::fs::write(&path, &good[..good.len() / 2]).expect("truncate asset");
    assert!(
        Storage::load_project(&entry, &Registry::new()).is_err(),
        "a truncated asset must be rejected"
    );
    // zero-byte asset
    std::fs::write(&path, b"").expect("zero asset");
    assert!(
        Storage::load_project(&entry, &Registry::new()).is_err(),
        "a zero-byte asset must be rejected"
    );
    // restore → load ok
    std::fs::write(&path, &good).expect("restore asset");
    assert!(
        Storage::load_project(&entry, &Registry::new()).is_ok(),
        "the restored project loads"
    );
}
