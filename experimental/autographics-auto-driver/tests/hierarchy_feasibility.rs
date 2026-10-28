//! T-02 scratch whole path; the production codec and tree will replace scratch storage.
use agp::host::iced_surface::{IcedSurface, SurfaceElement, SurfaceUi};
use autographics_auto_driver::authoring::{
    document::{Document, Kind, Object},
    host::Host,
    session::Edit,
};
use autographics_auto_driver::{
    ResultKind,
    Scalar::{Float, Int},
    Session,
};
#[derive(Default)]
struct Tree {
    moved: bool,
}
impl SurfaceUi for Tree {
    type Message = ();
    fn view(&self) -> SurfaceElement<'_, ()> {
        iced::widget::column![
            iced::widget::text("2D / group / circle"),
            iced::widget::button("Reparent / move group").on_press(())
        ]
        .into()
    }
    fn update(&mut self, _: ()) {
        self.moved = true;
    }
    fn background(&self) -> iced::Color {
        iced::Color::BLACK
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
fn derived(source: &str, shift: f32) -> Result<Vec<[f32; 3]>, String> {
    let mut vm = Session::new(source)?;
    vm.call("reset", &[], ResultKind::Unit)?;
    // Two-dimensional group/two leaves, three-dimensional nested groups/original GLB.
    for (i, (kind, dim, parent, x)) in [
        (5, 2, -1, shift),
        (0, 2, 0, -55.0),
        (1, 2, 0, 55.0),
        (5, 3, -1, 0.5),
        (5, 3, 3, 0.5),
        (3, 3, 4, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        vm.call(
            "identity",
            &[Int(i as i32 + 1), Int(kind), Int(dim), Int(parent)],
            ResultKind::Unit,
        )?;
        vm.call("visibility", &[Int(1)], ResultKind::Unit)?;
        for f in [x, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.9, 0.3, 0.2, 1.0] {
            vm.call("append_field", &[Float(f)], ResultKind::Unit)?;
        }
    }
    vm.call("derive", &[], ResultKind::Unit)?;
    assert_eq!(vm.int("profile", &[])?, 4);
    (0..6)
        .map(|i| {
            Ok([
                vm.float("field", &[Int(i), Int(0)])?,
                vm.float("field", &[Int(i), Int(1)])?,
                vm.float("field", &[Int(i), Int(2)])?,
            ])
        })
        .collect()
}
#[test]
fn scratch_real_auto_tree_pixels_failure_undo_storage() {
    let source = include_str!("../samples/authoring_hierarchy/main.at");
    let a = derived(source, 0.0).unwrap();
    assert_eq!(a[1], [-55.0, 0.0, 0.0]);
    assert_eq!(a[5], [0.0, 0.0, 0.0]);
    let mut d = Document::empty("hierarchy-scratch".into());
    for (id, kind, world) in [
        (2, Kind::Circle, a[1]),
        (3, Kind::Quad, a[2]),
        (6, Kind::Player, a[5]),
    ] {
        let mut o = Object::new(id, kind);
        o.translation = world;
        d.objects.push(o);
    }
    let mut h = Host::new(d.clone()).unwrap();
    h.frame(None).unwrap();
    let mut ui = Tree::default();
    let mut surface = IcedSurface::new(h.rt.gpu(), iced::Size::new(350.0, 100.0), 1.0).unwrap();
    let cursor = iced_wgpu::core::mouse::Cursor::Available(iced::Point::new(30.0, 40.0));
    surface.frame(&mut ui, &[], cursor);
    let events = [
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonPressed(
            iced_wgpu::core::mouse::Button::Left,
        )),
        iced_wgpu::core::Event::Mouse(iced_wgpu::core::mouse::Event::ButtonReleased(
            iced_wgpu::core::mouse::Button::Left,
        )),
    ];
    surface.frame(&mut ui, &events, cursor);
    assert!(ui.moved, "actual Iced tree button");
    let moved = derived(source, 30.0).unwrap();
    assert_eq!(moved[1], [-25.0, 0.0, 0.0]);
    assert_eq!(moved[2], [85.0, 0.0, 0.0]);
    assert_eq!(
        autographics_auto_driver::authoring::interaction::pick2(&d.objects, [345.0, 190.0]),
        Some(2)
    );
    h.editor.select(Some(2)).unwrap();
    let mut o = d.objects[0].clone();
    o.translation = moved[1];
    let stamp = h.editor.stamp;
    h.editor.begin(stamp, Edit::Replace(o.clone())).unwrap();
    h.settle().unwrap();
    assert_eq!(h.editor.history().0, 1);
    assert_eq!(
        autographics_auto_driver::authoring::interaction::pick2(
            &h.editor.preview.objects,
            [375.0, 190.0]
        ),
        Some(2)
    );
    let pixels = h.rt.capture(h.two, "moved").unwrap().0;
    h.fail_resource = true;
    o.translation[0] += 20.0;
    h.editor
        .begin(h.editor.stamp, Edit::Replace(o.clone()))
        .unwrap();
    h.settle().unwrap();
    assert_eq!(h.rt.capture(h.two, "failed").unwrap().0, pixels);
    h.fail_resource = false;
    h.editor.begin(h.editor.stamp, Edit::Replace(o)).unwrap();
    h.settle().unwrap();
    h.editor.begin(h.editor.stamp, Edit::Undo).unwrap();
    h.settle().unwrap();
    assert_eq!(h.rt.capture(h.two, "restored").unwrap().0, pixels);
    // Scratch persistence records topology separately; production migration is T-03/T-07.
    let scratch = serde_json::json!({"schema":"autographics.scene/2","parents":[-1,0,0,-1,3,4],"world":moved});
    let bytes = serde_json::to_vec(&scratch).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        scratch
    );
    let wrong = source.replace(
        "world = world + worlds[parent * 3 + axis]",
        "world = world - worlds[parent * 3 + axis]",
    );
    assert_ne!(
        derived(&wrong, 30.0).unwrap()[1],
        moved[1],
        "parent accumulation distinguishes wrong Auto"
    );
}
