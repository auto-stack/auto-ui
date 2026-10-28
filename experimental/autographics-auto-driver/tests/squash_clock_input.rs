static GPU: std::sync::Mutex<()> = std::sync::Mutex::new(());
use agp::games::squash::live::LiveKey;
use autographics_auto_driver::{
    host::Host,
    squash::{source, SquashScript},
};
#[test]
fn focus_unmount_blur_shared_tick_and_independent_instance() {
    let _serial = GPU.lock().unwrap();
    let mut host = Host::new_squash(&source()).unwrap();
    host.settle().unwrap();
    host.click([80.0, 70.0]).unwrap();
    let q = host.squash.as_mut().unwrap();
    q.inputs[0].key_event(LiveKey::ArrowRight, true, false);
    q.inputs[0].key_event(LiveKey::Space, true, false);
    host.tick(0);
    host.tick(0);
    host.settle().unwrap();
    let q = host.squash.as_mut().unwrap();
    assert_eq!(q.scripts[0].completed_ticks, 1);
    assert_eq!(q.scripts[1].completed_ticks, 1);
    let x = q.scripts[0]
        .session
        .float("state", &[autographics_auto_driver::Scalar::Int(0)])
        .unwrap();
    assert!(x > 0.0);
    assert_eq!(
        q.scripts[1]
            .session
            .float("state", &[autographics_auto_driver::Scalar::Int(0)])
            .unwrap(),
        0.0
    );
    let portal = host
        .rt
        .graph
        .portals()
        .iter()
        .find(|p| p.desc.child == host.game_surfaces[0])
        .unwrap()
        .id;
    host.rt.remove_portal(portal).unwrap();
    host.tick(1);
    host.settle().unwrap();
    assert!(host.input.keyboard_target().is_none());
    assert_eq!(
        host.squash.as_mut().unwrap().scripts[0]
            .session
            .float("state", &[autographics_auto_driver::Scalar::Int(0)])
            .unwrap(),
        x
    );
    host.click([500.0, 70.0]).unwrap();
    host.squash.as_mut().unwrap().inputs[0].key_event(LiveKey::ArrowRight, true, false);
    host.input.window_focus_lost();
    host.tick(2);
    host.settle().unwrap();
    assert_eq!(
        host.squash.as_mut().unwrap().scripts[0]
            .session
            .float("state", &[autographics_auto_driver::Scalar::Int(0)])
            .unwrap(),
        x
    );
    for i in 0..2 {
        assert_eq!(host.rt.stats.last_frame_renders[&host.game_surfaces[i]], 1);
    }
}
fn run(batch: u64) -> autographics_auto_driver::squash::Packet3d {
    let mut script = SquashScript::new(&source()).unwrap();
    script.settle().unwrap();
    script.outputs.clear();
    for tick in 0..360 {
        script
            .tick(
                script.id(),
                tick,
                [
                    if tick < 90 { 1 } else { 0 },
                    0,
                    if tick == 0 { 1 } else { 0 },
                ],
            )
            .unwrap();
        if (tick + 1) % batch == 0 {
            script.settle().unwrap();
            script.outputs.clear();
        }
    }
    script.settle().unwrap();
    assert_eq!(script.completed_ticks, 360);
    script.publication.last().unwrap().clone()
}
#[test]
fn fixed_ticks_are_independent_of_render_batch_and_no_input_never_jumps() {
    let mut a = run(1);
    let b = run(3);
    a.session = b.session;
    assert_eq!(a, b);
    let mut script = SquashScript::new(&source()).unwrap();
    script.settle().unwrap();
    script.outputs.clear();
    for tick in 0..240 {
        script.tick(script.id(), tick, [0, 0, 0]).unwrap();
        script.settle().unwrap();
        script.outputs.clear();
    }
    assert_eq!(
        script
            .session
            .int("status", &[autographics_auto_driver::Scalar::Int(5)])
            .unwrap(),
        0
    );
}

#[test]
fn one_real_runnable_fault_preserves_scene_and_other_instance_ui_progress() {
    let _serial = GPU.lock().unwrap();
    let original = source();
    let input = original.replace(
        "fn begin_tick(x int, z int, jump int) {",
        "fn begin_tick(x int, z int, jump int) { if x == 1 { while true {} }",
    );
    assert_ne!(input, original);
    let mut host = Host::new_squash(&input).unwrap();
    host.settle().unwrap();
    let previous = host.squash.as_ref().unwrap().accepted[0].clone();
    let image = host
        .rt
        .capture(host.game_surfaces[0], "last good")
        .unwrap()
        .0;
    host.click([80.0, 70.0]).unwrap();
    host.squash.as_mut().unwrap().inputs[0].key_event(LiveKey::ArrowRight, true, false);
    host.tick(0);
    host.settle().unwrap();
    assert!(host.squash.as_ref().unwrap().scripts[0].session.closed());
    assert_eq!(host.squash.as_ref().unwrap().accepted[0], previous);
    assert_eq!(
        host.rt
            .capture(host.game_surfaces[0], "after fault")
            .unwrap()
            .0,
        image
    );
    for tick in 1..20 {
        host.tick(tick);
        host.settle().unwrap();
    }
    assert_eq!(host.squash.as_ref().unwrap().scripts[1].completed_ticks, 20);
    assert_eq!(host.errors.len(), 1);
    assert!(host.errors[0].contains("budget"));
    host.click([1000.0, 25.0]).unwrap();
    host.frame(None).unwrap();
    assert!(
        host.paused,
        "actual stock UI stopped responding after an unrelated script fault"
    );
}

#[test]
fn queued_tick_before_initialization_is_not_counted_as_two_ticks() {
    let mut script = SquashScript::new(&source()).unwrap();
    script.tick(script.id(), 0, [0, 0, 0]).unwrap();
    script.settle().unwrap();
    assert_eq!(script.completed_ticks, 1);
    assert_eq!(script.outputs.len(), 2);
    assert_eq!(script.outputs[0].sequence, 1);
    assert_eq!(script.outputs[1].sequence, 2);
    assert_eq!(
        script
            .session
            .int("status", &[autographics_auto_driver::Scalar::Int(2)])
            .unwrap(),
        1
    );
}
