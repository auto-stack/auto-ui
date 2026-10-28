//! Bounded scratch protocol, before the production play module. No Editor writes.
use agp::host::surface_runtime::SurfaceContent;
use autographics_auto_driver::{
    authoring::{
        assets,
        document::{Dimension, Document, Group, Kind, Object},
        host::Host,
    },
    Progress, ResultKind, Scalar, Session,
};
use glam::{Mat4, Vec3};
const SCRIPT: &str = "var x=0.0\nvar ticks=0\nfn main() {}\nfn profile() int {return 6}\nfn init(parent float, local float) {x=parent+local; ticks=0}\nfn tick(axis int, dt float) {x=x+axis*3.0*dt; ticks=ticks+1}\nfn world() float {return x}\nfn count() int {return ticks}\n";
#[test]
fn retained_auto_current_imported_scene_restore_and_faults() {
    let asset = assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap();
    let mut d = Document::empty("play-feasibility".into());
    let mut group = Group::new(10, Dimension::Three);
    group.translation = [1.0, 0.0, 0.0];
    d.groups.push(group);
    let mut model = Object::new(11, Kind::Model);
    model.parent = Some(10);
    model.translation = [-1.0, 0.0, 0.0];
    model.resource = Some(asset.identity.key());
    d.objects.push(model);
    let mut two = Object::new(12, Kind::Quad);
    two.translation = [-100.0, 0.0, 0.0];
    d.objects.push(two);
    d.assets.push(asset.identity.clone());
    let registry = assets::Registry::from([(asset.identity.key(), asset)]);
    let mut h = Host::with_assets(d.clone(), registry).unwrap();
    h.frame(None).unwrap();
    let original = h.rt.capture(h.top, "before").unwrap().0;
    let history = h.editor.history();
    let stamp = h.editor.stamp;
    let owner = h.editor.preview.vm.owner_probe();
    let baseline = h.resource_snapshot();
    let mut vm = Session::new(SCRIPT).unwrap();
    let alive = vm.owner_probe();
    vm.call(
        "init",
        &[Scalar::Float(1.0), Scalar::Float(-1.0)],
        ResultKind::Unit,
    )
    .unwrap();
    let started = std::time::Instant::now();
    for _ in 0..60 {
        vm.call(
            "tick",
            &[Scalar::Int(1), Scalar::Float(1.0 / 60.0)],
            ResultKind::Unit,
        )
        .unwrap();
    }
    let x = vm.float("world", &[]).unwrap();
    assert!((x - 3.0).abs() < 1e-4);
    assert_eq!(vm.int("count", &[]).unwrap(), 60);
    if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(h.three) {
        assert_eq!(c.instances.len(), 2);
        for i in &mut c.instances {
            i.model = Mat4::from_translation(Vec3::new(x, 0.0, 0.0));
        }
    }
    h.rt.graph.mark_content_dirty(h.three);
    h.frame(None).unwrap();
    for (i, dx) in [-1.5, 1.5].into_iter().enumerate() {
        let p = autographics_auto_driver::authoring::interaction::project(
            h.camera(),
            Vec3::new(x + dx, 0.5, 0.0),
        )
        .unwrap();
        let pixel = h.rt.probe_pixel(h.top, [p.x, p.y + 400.0]).unwrap();
        assert!(
            pixel[i] > 80 && pixel[1 - i] < 20,
            "all imported parts at Auto world: {pixel:?}"
        );
    }
    assert_ne!(h.rt.capture(h.top, "running").unwrap().0, original);
    // Paused has no dispatch; one neutral step is an actual Auto increment.
    assert_eq!(vm.int("count", &[]).unwrap(), 60);
    vm.call(
        "tick",
        &[Scalar::Int(0), Scalar::Float(1.0 / 60.0)],
        ResultKind::Unit,
    )
    .unwrap();
    assert_eq!(vm.int("count", &[]).unwrap(), 61);
    vm.close();
    assert!(!alive());
    assert!(owner());
    // Restore the accepted scene's retained GPU instances, without compile/reimport.
    if let Some(SurfaceContent::Scene3d(c)) = h.rt.content_mut(h.three) {
        for i in &mut c.instances {
            i.model = Mat4::IDENTITY;
        }
    }
    h.rt.graph.mark_content_dirty(h.three);
    h.frame(None).unwrap();
    assert_eq!(h.rt.capture(h.top, "stopped").unwrap().0, original);
    assert_eq!(h.editor.document, d);
    assert_eq!(h.editor.stamp, stamp);
    assert_eq!(h.editor.history(), history);
    assert_eq!(
        h.resource_snapshot()["mesh_uploads"],
        baseline["mesh_uploads"]
    );
    let mut next = Session::new(SCRIPT).unwrap();
    next.call(
        "init",
        &[Scalar::Float(1.0), Scalar::Float(-1.0)],
        ResultKind::Unit,
    )
    .unwrap();
    assert_eq!(next.int("count", &[]).unwrap(), 0);
    assert_eq!(next.float("world", &[]).unwrap(), 0.0);
    let mut bad = Session::new(&SCRIPT.replace("return x", "return 0")).unwrap();
    assert!(bad.start("world", &[], ResultKind::Float).is_err());
    assert!(bad.closed());
    let mut budget = Session::new("fn main(){}\nfn tick(){while true {}}\n").unwrap();
    let mut result = budget.start("tick", &[], ResultKind::Unit);
    while matches!(result, Ok(Progress::Pending)) {
        result = budget.pump();
    }
    assert!(result.is_err());
    assert!(budget.closed());
    println!("scratch retained Auto 61 ticks; whole imported GPU parts/restore/owner/type/budget/restart pass; tick+GPU us={}",started.elapsed().as_micros());
}
