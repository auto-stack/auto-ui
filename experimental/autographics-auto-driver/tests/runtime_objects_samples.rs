//! auto-ui PLAN-001 runtime objects sample acceptance (contract V-04 work
//! scope): both dimensions' sample projects import through the production
//! loader, their public sources admit, the real Auto trajectory and lifecycle
//! change with the source parameters (main vs main-alt), removing a spawn
//! changes the actual published content (frozen-input mutation), and the 3D
//! sample drives a real multi-instance lifecycle through the Host/GPU path.
//! Windowless GPU with public logical inputs only. Run single-threaded:
//! cargo test --locked --test runtime_objects_samples -- --test-threads=1
use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    document::{Document, Kind},
    host::Host,
    play::Action,
    runtime_objects::Dim,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn sample(dim: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/runtime-objects")
        .join(dim)
        .join(name)
}

fn source_of(dim: &str, name: &str) -> String {
    std::fs::read_to_string(sample(dim, name)).expect("sample source")
}

fn settle(h: &mut Host) {
    let end = Instant::now() + std::time::Duration::from_secs(31);
    while h.play.as_ref().unwrap().pending() {
        assert!(Instant::now() < end, "host deadline");
        h.frame(None).expect("frame");
    }
}

fn step(h: &mut Host) {
    let before = format!(
        "{:?} err={:?}",
        h.play.as_ref().unwrap().state,
        h.play.as_ref().unwrap().error
    );
    h.play
        .as_mut()
        .unwrap()
        .resume()
        .unwrap_or_else(|e| panic!("resume from {before}: {e}"));
    h.play
        .as_mut()
        .unwrap()
        .dispatch(Action::default())
        .expect("dispatch");
    h.pause_play("");
    settle(h);
}

fn run_steps(h: &mut Host, steps: usize) {
    for _ in 0..steps {
        step(h);
    }
}

fn play_error(h: &Host) -> Option<String> {
    h.play.as_ref().and_then(|p| p.error.clone())
}

fn instances_of(
    h: &Host,
) -> autographics_auto_driver::authoring::runtime_objects::RuntimeObjectsFrame {
    h.play
        .as_ref()
        .unwrap()
        .last
        .as_ref()
        .expect("admitted frame")
        .instances
        .as_ref()
        .expect("objects block")
        .clone()
}

#[test]
fn sample_projects_import_and_drive_real_object_lifecycles() {
    // Both scene layouts import through the production loader with their
    // frozen template sets (the spawnable leaves included).
    let two = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        "two",
        "layout_a.agscene",
    ))
    .expect("2D layout import")
    .0;
    assert_eq!(two.objects.len(), 4);
    assert_eq!(two.objects[2].kind, Kind::Circle, "the Coin template");
    assert_eq!(two.objects[3].kind, Kind::Quad, "the Gem template");
    let three = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        "three",
        "layout_a.agscene",
    ))
    .expect("3D layout import")
    .0;
    assert_eq!(three.objects.len(), 4);
    assert_eq!(three.objects[2].kind, Kind::Mob, "the Crystal template");

    // The public sources admit on their combined profiles.
    let _ = SourceSnapshot::from_bytes_for(
        Path::new("two-main.at"),
        source_of("two", "main.at").as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("2D sample admits");
    let _ = SourceSnapshot::from_bytes_for(
        Path::new("two-alt.at"),
        source_of("two", "main-alt.at").as_bytes().to_vec(),
        Api::Objects2V1,
    )
    .expect("2D alt admits");
    let _ = SourceSnapshot::from_bytes_for(
        Path::new("three-main.at"),
        source_of("three", "main.at").as_bytes().to_vec(),
        Api::Objects3V1,
    )
    .expect("3D sample admits");
    let _ = SourceSnapshot::from_bytes_for(
        Path::new("three-alt.at"),
        source_of("three", "main-alt.at").as_bytes().to_vec(),
        Api::Objects3V1,
    )
    .expect("3D alt admits");
}

#[test]
fn two_d_sample_trajectory_and_lifecycle_follow_the_source_parameters() {
    let scene = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        "two",
        "layout_a.agscene",
    ))
    .expect("2D layout import")
    .0;
    // The frozen sample: the coin drifts at 2/s and recycles at x<0.
    let mut h = Host::with_project(scene.clone(), Default::default(), None).expect("host");
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(
            Path::new("two-run.at"),
            source_of("two", "main.at").as_bytes().to_vec(),
            Api::Objects2V1,
        )
        .expect("admission"),
    )
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    let coin = instances_of(&h).get(0).expect("coin spawned").clone();
    assert_eq!((coin.generation, coin.template_id), (1, 3));
    // The alt source: 4/s drift recycling at x<10 — the SAME scene, a
    // different real trajectory: after 6 ticks the positions differ and the
    // faster coin has already recycled (generation 2).
    let mut ha = Host::with_project(scene, Default::default(), None).expect("host");
    ha.start_user_snapshot(
        SourceSnapshot::from_bytes_for(
            Path::new("two-alt-run.at"),
            source_of("two", "main-alt.at").as_bytes().to_vec(),
            Api::Objects2V1,
        )
        .expect("admission"),
    )
    .expect("start");
    ha.pause_play("");
    settle(&mut ha);
    run_steps(&mut h, 6);
    run_steps(&mut ha, 6);
    let coin = instances_of(&h).get(0).unwrap().clone();
    let alt = instances_of(&ha).get(0).unwrap().clone();
    assert!(
        (coin.translation[0] - alt.translation[0]).abs() > 1.0,
        "the speed parameter changes the real trajectory: {} vs {}",
        coin.translation[0],
        alt.translation[0]
    );
    assert!(
        alt.generation > coin.generation,
        "the faster coin already recycled: alt gen {} vs main gen {}",
        alt.generation,
        coin.generation
    );
    // Only logs differ is not a green path: the published 2D streams differ.
    let draws = |h: &Host| match h.rt.content(h.two) {
        Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => g
            .commands
            .iter()
            .map(|c| (c.center.map(|v| (v * 10.0).round() as i32), c.shape))
            .collect::<Vec<_>>(),
        _ => panic!("2D content"),
    };
    assert_ne!(draws(&h), draws(&ha), "the pixels follow the source");
}

#[test]
fn three_d_sample_lifecycle_recycles_through_the_same_slot() {
    let scene = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        "three",
        "layout_a.agscene",
    ))
    .expect("3D layout import")
    .0;
    // The frozen sample: orbit 1.6/s recycling at x>1.5 -> tick ~57.
    let mut h = Host::with_project(scene.clone(), Default::default(), None).expect("host");
    h.start_user_snapshot(
        SourceSnapshot::from_bytes_for(
            Path::new("three-run.at"),
            source_of("three", "main.at").as_bytes().to_vec(),
            Api::Objects3V1,
        )
        .expect("admission"),
    )
    .expect("start");
    h.pause_play("");
    settle(&mut h);
    if let Some(error) = play_error(&h) {
        panic!("3D sample faulted: {error}");
    }
    assert_eq!(instances_of(&h).dim, Dim::ThreeD);
    run_steps(&mut h, 70);
    let crystal = instances_of(&h).get(0).expect("crystal alive").clone();
    assert!(
        crystal.generation >= 2,
        "the sweep crossed the recycle line and respawned"
    );
    // The alt source recycles at x>0.5 -> tick ~50: at tick 55 the alt has
    // already recycled while the main source has not (parameter timing).
    let mut ha = Host::with_project(scene, Default::default(), None).expect("host");
    ha.start_user_snapshot(
        SourceSnapshot::from_bytes_for(
            Path::new("three-alt-run.at"),
            source_of("three", "main-alt.at").as_bytes().to_vec(),
            Api::Objects3V1,
        )
        .expect("admission"),
    )
    .expect("start");
    ha.pause_play("");
    settle(&mut ha);
    for _ in 0..55 {
        step(&mut ha);
    }
    let alt = instances_of(&ha).get(0).unwrap().clone();
    assert!(alt.generation >= 2, "the alt already recycled by tick 55");
    // The published 3D instances follow each source's live position.
    let at = |h: &Host, x: f32| match h.rt.content(h.three) {
        Some(agp::host::surface_runtime::SurfaceContent::Scene3d(c)) => c
            .instances
            .iter()
            .filter(|i| (i.model.w_axis.x - x).abs() < 1e-3)
            .count(),
        _ => panic!("3D content"),
    };
    let x_main = instances_of(&h).get(0).unwrap().translation[0];
    assert!(
        at(&h, x_main) >= 1,
        "the main crystal renders where it lives"
    );
    assert_ne!(x_main, alt.translation[0], "the two sources diverge");
}

#[test]
fn frozen_spawn_mutation_changes_the_real_published_content() {
    // Removing the gem spawn from the sample source must remove the real
    // instance (and its published draw); restoring the source restores it.
    let scene = autographics_auto_driver::authoring::storage::Storage::load(&sample(
        "two",
        "layout_a.agscene",
    ))
    .expect("2D layout import")
    .0;
    let full = source_of("two", "main.at");
    // The mutation swaps the gem's frozen template (quad -> circle); every
    // other line and the slot stay identical, so the admitted table and the
    // published shapes must follow the input.
    let mutated = full.replace(
        " h_gem=ag_object_spawn(1, 4, 20.0, 2.5, 0.0)\n",
        " h_gem=ag_object_spawn(1, 3, 20.0, 2.5, 0.0)\n",
    );
    assert_ne!(full, mutated, "the mutation must change the input");
    let run = |name: &str, source: &str| {
        let mut h = Host::with_project(scene.clone(), Default::default(), None).expect("host");
        h.start_user_snapshot(
            SourceSnapshot::from_bytes_for(
                Path::new(name),
                source.as_bytes().to_vec(),
                Api::Objects2V1,
            )
            .expect("admission"),
        )
        .expect("start");
        h.pause_play("");
        settle(&mut h);
        if let Some(error) = play_error(&h) {
            let slots = h
                .play
                .as_ref()
                .unwrap()
                .last
                .as_ref()
                .map(|f| {
                    f.instances
                        .as_ref()
                        .map(|i| i.slots.iter().map(|x| x.generation).collect::<Vec<_>>())
                })
                .unwrap_or(None);
            panic!("{name} faulted: {error} slots={slots:?}");
        }
        let gem = instances_of(&h)
            .get(1)
            .map(|s| (s.generation, s.template_id));
        let shapes = match h.rt.content(h.two) {
            Some(agp::host::surface_runtime::SurfaceContent::Game2d(g)) => {
                let quads = g
                    .commands
                    .iter()
                    .filter(|c| c.shape == agp::renderer::pipeline::Shape::Rect)
                    .count();
                let circles = g
                    .commands
                    .iter()
                    .filter(|c| c.shape == agp::renderer::pipeline::Shape::Circle)
                    .count();
                (quads, circles)
            }
            _ => panic!("2D content"),
        };
        h.stop_play();
        (gem, shapes.0, shapes.1)
    };
    let (gem_full, quads_full, circles_full) = run("mut-full.at", &full);
    assert_eq!(gem_full, Some((1, 4)), "the gem spawned from the template");
    let (gem_mutated, quads_mutated, circles_mutated) = run("mut-swap.at", &mutated);
    assert_eq!(
        gem_mutated,
        Some((1, 3)),
        "the mutated source spawns the circle template"
    );
    assert!(
        quads_full > quads_mutated && circles_mutated > circles_full,
        "the published content follows the template swap: quads {quads_full}->{quads_mutated} circles {circles_full}->{circles_mutated}"
    );
}

/// The sample projects compile four trusted units plus the user source; run
/// every body on a dedicated big-stack thread.
fn big_stack(body: fn()) {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, body)
            .expect("test thread")
            .join()
            .expect("test body panicked");
    });
}

#[test]
fn sample_projects_import_and_drive_real_object_lifecycles_big_stack() {
    big_stack(sample_projects_import_and_drive_real_object_lifecycles);
}

#[test]
fn two_d_sample_trajectory_and_lifecycle_follow_the_source_parameters_big_stack() {
    big_stack(two_d_sample_trajectory_and_lifecycle_follow_the_source_parameters);
}

#[test]
fn three_d_sample_lifecycle_recycles_through_the_same_slot_big_stack() {
    big_stack(three_d_sample_lifecycle_recycles_through_the_same_slot);
}

#[test]
fn frozen_spawn_mutation_changes_the_real_published_content_big_stack() {
    big_stack(frozen_spawn_mutation_changes_the_real_published_content);
}
