include!("support/sweep.rs");
const BOX: [f32; 7] = [1., -1., -1., -1., 1., 1., 1.];
fn tick(s: &mut Session) {
    s.call("_ag_tick", &[Int(1), Float(1. / 60.)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("_ag_status", &[]).unwrap(), 0)
}
#[test]
fn dirty_axes_visibility_and_move_preserve_the_shared_graph() {
    let update="ag_set_local(0,0,2.0);ag_set_local(0,1,-3.0);ag_set_local(0,2,4.0);ag_set_visible(0,0);if ag_visible(1)!=0 {observed=1};if ag_world(1,0)!=2.0 || ag_world(1,1)!= -3.0 || ag_world(1,2)!=4.0 {observed=2};ag_set_visible(0,1);ag_move_local(0,1.0,2.0,-3.0);if ag_visible(1)!=1 {observed=3};if ag_world(1,0)!=3.0 || ag_world(1,1)!= -1.0 || ag_world(1,2)!=1.0 {observed=4};ag_set_local(0,1,7.0);ag_set_visible(0,0)";
    let rows = [(5, 3, [0.; 7]), (2, 3, BOX)];
    let mut s = scene(update, &rows);
    let heap = s.heap_objects();
    tick(&mut s);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
    assert_eq!(s.float("_ag_field", &[Int(1), Int(0)]).unwrap(), 3.);
    assert_eq!(s.float("_ag_field", &[Int(1), Int(1)]).unwrap(), 7.);
    assert_eq!(s.float("_ag_field", &[Int(1), Int(2)]).unwrap(), 1.);
    assert_eq!(s.int("_ag_visible", &[Int(1)]).unwrap(), 0);
    assert_eq!(s.heap_objects(), heap);
}
#[test]
fn missing_setter_axis_or_visibility_mark_is_detected_with_another_dirty_component() {
    let update = "ag_set_local(0,0,2.0);ag_set_local(0,2,4.0);ag_set_visible(0,0)";
    let rows = [(5, 3, [0.; 7]), (2, 3, BOX)];
    let mut good = scene(update, &rows);
    tick(&mut good);
    assert_eq!(good.float("_ag_field", &[Int(1), Int(2)]).unwrap(), 4.);
    assert_eq!(good.int("_ag_visible", &[Int(1)]).unwrap(), 0);
    for (function, mark) in [
        ("fn ag_set_local(", "_ag_segment_dz=1"),
        ("fn ag_set_visible(", "_ag_segment_dv=1"),
    ] {
        let mut wrong = scene_custom(update, &rows, false, |s| {
            let a = s.find(function).unwrap();
            let b = s[a + 3..].find("fn ").unwrap() + a + 3;
            assert!(s[a..b].contains(mark));
            format!("{}{}{}", &s[..a], s[a..b].replace(mark, ""), &s[b..])
        });
        tick(&mut wrong);
        if function.contains("set_local") {
            assert_ne!(wrong.float("_ag_field", &[Int(1), Int(2)]).unwrap(), 4.)
        } else {
            assert_ne!(wrong.int("_ag_visible", &[Int(1)]).unwrap(), 0)
        }
    }
}
#[test]
fn world_addition_keeps_f32_order_and_user_source_is_unchanged() {
    let update="ag_set_local(0,0,100000000.0);ag_set_local(1,0,-100000000.0);if ag_world(1,0)!=0.0 {observed=1};ag_set_local(0,0,100000008.0);if ag_world(1,0)!=8.0 {observed=2}";
    let mut s = scene(update, &[(5, 3, [0.; 7]), (2, 3, BOX)]);
    tick(&mut s);
    assert_eq!(s.int("probe_observed", &[]).unwrap(), 0);
    let raw = b"var text=\"literal; { } // intact\"\nfn app_start(){}\nfn app_update(dt float){}\n";
    let snapshot =
        SourceSnapshot::from_bytes_for(Path::new("raw.at"), raw.to_vec(), Api::SweepV4).unwrap();
    assert_eq!(snapshot.bytes, raw);
    assert!(snapshot.source.contains(std::str::from_utf8(raw).unwrap()));
}
