use autographics_auto_driver::authoring::{
    host::Host,
    play::{Action, State},
    storage::Storage,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn settle(h: &mut Host) {
    let limit = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < limit)
    }
}
#[test]
fn circle_aabb_corner_is_a_hit_and_visible_in_the_final_pixels() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (d, a, c, _) = Storage::load_project(
        &root.join("samples/spatial_projects/zone2d/scene.agscene"),
        &Default::default(),
    )
    .unwrap();
    let mut h = Host::with_project(d, a, c).unwrap();
    h.start_selected_play().unwrap();
    h.pause_play("");
    settle(&mut h);
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action {
            space: 2,
            x: 1,
            y: 0,
            toggle: false,
        })
        .unwrap();
    settle(&mut h);
    let p = h.play.as_ref().unwrap();
    assert_eq!(p.state, State::Paused);
    let objects = &p.last.as_ref().unwrap().objects;
    let circle = &objects[0];
    let corner = objects.iter().find(|o| o.id == 5).unwrap();
    assert!(corner.visible);
    assert_eq!(
        corner.translation,
        [circle.translation[0] - 5., circle.translation[1] - 5., 0.]
    );
    assert!(
        ((corner.translation[0] - circle.translation[0]) / 5.).powi(2)
            + ((corner.translation[1] - circle.translation[1]) / 5.).powi(2)
            > 1.
    );
    let point = [400. + corner.translation[0], 190. - corner.translation[1]];
    let pixel = h.rt.probe_pixel(h.top, point).unwrap();
    assert!(
        pixel[0] > 240 && pixel[1] > 240 && pixel[2] < 120,
        "proxy corner feedback pixel {pixel:?}"
    );
    h.stop_play();
}
