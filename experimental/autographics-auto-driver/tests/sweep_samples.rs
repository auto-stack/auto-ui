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
    let deadline = Instant::now() + Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        h.frame(None).unwrap();
        assert!(Instant::now() < deadline);
    }
    assert_eq!(
        h.play.as_ref().unwrap().state,
        State::Paused,
        "{:?}",
        h.play.as_ref().unwrap().error
    );
}
#[test]
fn real_user_dash_stops_with_gap_reverses_and_neutral_step_preserves_position() {
    for (name, space) in [("dash2d", 2), ("dash3d", 3)] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (d, a, c, _) = Storage::load_project(
            &root.join(format!("samples/sweep_projects/{name}/scene.agscene")),
            &Default::default(),
        )
        .unwrap();
        let mut h = Host::with_project(d, a, c).unwrap();
        let base = h.editor.isolation_identity();
        h.start_selected_play().unwrap();
        h.pause_play("");
        settle(&mut h);
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space,
                x: 1,
                ..Default::default()
            })
            .unwrap();
        settle(&mut h);
        let stopped = h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation[0];
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space,
                x: 1,
                ..Default::default()
            })
            .unwrap();
        settle(&mut h);
        assert!(
            (h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation[0] - stopped)
                .abs()
                < 1e-5
        );
        let stopped = h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation[0];
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action::default())
            .unwrap();
        settle(&mut h);
        assert_eq!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation[0],
            stopped
        );
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space,
                x: -1,
                ..Default::default()
            })
            .unwrap();
        settle(&mut h);
        assert!(
            h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation[0]
                < stopped - 1.
        );
        h.stop_play();
        assert_eq!(h.editor.isolation_identity(), base);
    }
}

#[test]
fn real_user_initial_overlap_remains_blocked_with_visible_feedback() {
    for (name, space) in [("dash2d", 2), ("dash3d", 3)] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (mut d, a, c, _) = Storage::load_project(
            &root.join(format!("samples/sweep_projects/{name}/scene.agscene")),
            &Default::default(),
        )
        .unwrap();
        let px = d.objects[0]
            .parent
            .map(|id| d.groups.iter().find(|g| g.id == id).unwrap().translation[0])
            .unwrap_or(0.);
        d.objects[0].translation[0] = d.objects[1].translation[0] - px;
        let mut h = Host::with_project(d, a, c).unwrap();
        h.start_selected_play().unwrap();
        h.pause_play("");
        settle(&mut h);
        let before = h.play.as_ref().unwrap().last.as_ref().unwrap().objects[0].translation;
        h.play
            .as_mut()
            .unwrap()
            .dispatch(Action {
                space,
                x: 1,
                ..Default::default()
            })
            .unwrap();
        settle(&mut h);
        let f = h.play.as_ref().unwrap().last.as_ref().unwrap();
        assert_eq!(f.objects[0].translation, before);
        assert!(
            f.objects
                .iter()
                .find(|o| o.id == if space == 2 { 4 } else { 3 })
                .unwrap()
                .visible
        );
        h.stop_play();
    }
}
