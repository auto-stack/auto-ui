use autographics_auto_driver::authoring::{
    assets,
    document::{Dimension, Document, Group, Kind, Object},
    spatial::Descriptor,
};
fn model() -> agp::assets::ModelData {
    assets::decode(
        include_bytes!("../research/asset-probe/two-parts.glb").to_vec(),
        None,
    )
    .unwrap()
    .model
    .clone()
}
fn fixture() -> Document {
    let mut d = Document::empty("geometry".into());
    let mut g = Group::new(10, Dimension::Three);
    g.translation = [3., 2., 1.];
    d.groups.push(g);
    for id in [11, 12] {
        let mut o = Object::new(id, Kind::Cylinder);
        o.parent = Some(10);
        o.translation = [8., 0., 0.];
        o.scale = [2., 0.5, 1.25];
        o.yaw = 0.7;
        o.pitch = 0.3;
        d.objects.push(o);
    }
    let mut circle = Object::new(13, Kind::Circle);
    circle.scale = [12., 8., 1.];
    d.objects.push(circle);
    d
}
// Independent scalar Rx then Ry, no production bounds/transform calls.
fn rotate(p: [f32; 3], o: &Object) -> [f32; 3] {
    let x = p[0] * o.scale[0];
    let y = p[1] * o.scale[1];
    let z = p[2] * o.scale[2];
    let y1 = o.pitch.cos() * y - o.pitch.sin() * z;
    let z1 = o.pitch.sin() * y + o.pitch.cos() * z;
    [
        o.yaw.cos() * x + o.yaw.sin() * z1,
        y1,
        0.0 - o.yaw.sin() * x + o.yaw.cos() * z1,
    ]
}
#[test]
fn all_mesh_box_transform_cache_dimensions_and_stale_identity() {
    let mut m = model();
    m.meshes[1].name = m.meshes[0].name.clone();
    let d = fixture();
    let a = Descriptor::build(&d, |_| Some(&m)).unwrap();
    assert_eq!(a.local_scans, 1);
    assert_eq!(a.rows()[0], [0.; 7]);
    assert_eq!(a.rows()[3], [1., -6., -4., 0., 6., 4., 0.]);
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for p in m.meshes.iter().flat_map(|m| &m.positions) {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let mut expect_lo = [f32::INFINITY; 3];
    let mut expect_hi = [f32::NEG_INFINITY; 3];
    for x in [lo[0], hi[0]] {
        for y in [lo[1], hi[1]] {
            for z in [lo[2], hi[2]] {
                let p = rotate([x, y, z], &d.objects[0]);
                for k in 0..3 {
                    expect_lo[k] = expect_lo[k].min(p[k]);
                    expect_hi[k] = expect_hi[k].max(p[k]);
                }
            }
        }
    }
    for k in 0..3 {
        assert!((a.rows()[1][k + 1] - expect_lo[k]).abs() < 1e-3);
        assert!((a.rows()[1][k + 4] - expect_hi[k]).abs() < 1e-3);
    }
    for p in m.meshes.iter().flat_map(|m| &m.positions) {
        let p = rotate(*p, &d.objects[0]);
        for k in 0..3 {
            assert!(p[k] >= a.rows()[1][k + 1] - 1e-3 && p[k] <= a.rows()[1][k + 4] + 1e-3);
        }
    }
    let mut missing = m.clone();
    missing.meshes.truncate(1);
    let wrong = Descriptor::build(&d, |_| Some(&missing)).unwrap();
    assert_ne!(
        a.rows()[1],
        wrong.rows()[1],
        "second-mesh omission control must differ"
    );
    let mut changed = d.clone();
    changed.objects[0].scale[0] = 3.;
    assert!(a.validate(&changed).is_err());
    assert_ne!(
        a.hash(),
        Descriptor::build(&changed, |_| Some(&m)).unwrap().hash()
    );
    changed = d.clone();
    changed.objects.swap(0, 1);
    assert!(a.validate(&changed).is_err());
}
#[test]
fn finite_degenerate_and_unavailable_are_not_fabricated_zero_boxes() {
    let d = fixture();
    let mut m = model();
    for mesh in &mut m.meshes {
        mesh.positions = vec![[0.; 3]];
    }
    let a = Descriptor::build(&d, |_| Some(&m)).unwrap();
    assert_eq!(a.rows()[1], [1., 0., 0., 0., 0., 0., 0.]);
    m.meshes.clear();
    let a = Descriptor::build(&d, |_| Some(&m)).unwrap();
    assert_eq!(a.rows()[1], [0.; 7]);
}
