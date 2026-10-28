use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    behavior_project::*,
    document::Document,
    session::{Edit, Editor},
    storage::{Fault, Storage},
};
use std::{path::Path, sync::Arc};
fn settle(e: &mut Editor) {
    while !e.pump().unwrap() {}
    e.commit().unwrap();
}
fn edit(c: Code, api: Api) -> Edit {
    Edit::Behavior {
        binding: Some(Binding {
            api: api.version().into(),
            source: c.identity().clone(),
            actors: Actors {
                two_d: Some(1),
                three_d: None,
            },
        }),
        code: Some(c),
    }
}
#[test]
fn same_raw_api_switch_undo_source_admission_and_save_reopen() {
    let raw = b"fn app_start(){}\nfn app_update(dt float){}\n";
    let c = CodePayload::from_bytes_for(Path::new("raw.at"), raw.to_vec(), Api::SpatialV2).unwrap();
    let mut e = Editor::new(Document::sample()).unwrap();
    e.begin(e.stamp, edit(c.clone(), Api::V1)).unwrap();
    settle(&mut e);
    let first = e.document.hash().unwrap();
    e.begin(e.stamp, edit(c.clone(), Api::SpatialV2)).unwrap();
    settle(&mut e);
    assert_ne!(e.document.hash().unwrap(), first);
    assert!(Arc::ptr_eq(e.code.as_ref().unwrap(), &c));
    e.begin(e.stamp, Edit::Undo).unwrap();
    settle(&mut e);
    assert_eq!(e.document.behavior.as_ref().unwrap().api, Api::V1.version());
    e.begin(e.stamp, Edit::Redo).unwrap();
    settle(&mut e);
    assert_eq!(
        e.document.behavior.as_ref().unwrap().api,
        Api::SpatialV2.version()
    );
    let helper=b"fn ag_aabb_min(n int,a int) float{return 7.0}\nfn app_start(){}\nfn app_update(dt float){}\n";
    let h = CodePayload::from_bytes(Path::new("helper.at"), helper.to_vec()).unwrap();
    let before = e.isolation_identity();
    assert!(e.begin(e.stamp, edit(h, Api::SpatialV2)).is_err());
    assert_eq!(e.isolation_identity(), before);
    let spatial = CodePayload::from_bytes_for(
        Path::new("query.at"),
        b"fn app_start(){let v=ag_aabb_min(0,0)}\nfn app_update(dt float){}\n".to_vec(),
        Api::SpatialV2,
    )
    .unwrap();
    assert!(e.begin(e.stamp, edit(spatial, Api::V1)).is_err());
    assert_eq!(e.isolation_identity(), before);
    let root = std::env::temp_dir().join(format!("ag846-project-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut storage = Storage::default();
    storage
        .save(&mut e, &root.join("scene.agscene"), Fault::None)
        .unwrap();
    let (doc, assets, code, _) =
        Storage::load_project(&root.join("scene.agscene"), &Default::default()).unwrap();
    let opened = Editor::with_project(doc, assets, code).unwrap();
    assert_eq!(
        opened.document.behavior.as_ref().unwrap().api,
        Api::SpatialV2.version()
    );
    assert!(!opened.dirty());
    assert_eq!(opened.code.as_ref().unwrap().bytes(), raw);
    let a = SourceSnapshot::from_bytes_for(Path::new("raw.at"), raw.to_vec(), Api::V1).unwrap();
    let b =
        SourceSnapshot::from_bytes_for(Path::new("raw.at"), raw.to_vec(), Api::SpatialV2).unwrap();
    assert_eq!(a.user_hash, b.user_hash);
    assert_ne!(a.api_hash, b.api_hash);
    assert_ne!(a.combined_hash, b.combined_hash);
}
