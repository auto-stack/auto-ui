use autographics_auto_driver::{host::Host, squash::source};
#[test]
fn measured_single_shared_and_independent_instance_stability() {
    let started = std::time::Instant::now();
    let mut host = Host::new_squash(&source()).unwrap();
    host.enable_audio().unwrap();
    host.settle().unwrap();
    let cold = started.elapsed().as_micros();
    fn working_set() -> u64 {
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {}).WorkingSet64", std::process::id()),
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse().ok())
            .unwrap()
    }
    let memory_before = working_set();
    let pipelines = agp::renderer::pipeline_creations();
    let targets = host.rt.live_target_slots();
    let mut results = vec![];
    let mut tick = 0u64;
    for mode in 0..3 {
        if mode == 1 {
            let portal = host
                .rt
                .graph
                .portals()
                .iter()
                .filter(|p| p.desc.child == host.game_surfaces[0])
                .last()
                .unwrap()
                .id;
            host.rt.remove_portal(portal).unwrap();
        }
        if mode == 2 {
            host.squash.as_mut().unwrap().scripts[1].close();
        }
        let mut samples = vec![];
        for _ in 0..60 {
            let start = std::time::Instant::now();
            host.tick(tick);
            host.settle().unwrap();
            tick += 1;
            samples.push(start.elapsed().as_micros());
            assert_eq!(host.rt.live_target_slots(), targets);
            assert_eq!(agp::renderer::pipeline_creations(), pipelines);
        }
        samples.sort_unstable();
        let label = [
            "shared mounts/two instances",
            "single mounts/two instances",
            "single mounts/one active instance",
        ][mode];
        results.push(serde_json::json!({"mode":label,"cpu_submission_wall_p50_us":samples[30],"cpu_submission_wall_p95_us":samples[57]}));
    }
    let resources = host.squash.as_ref().unwrap().resource_metrics(&host);
    let selected = resources["profile2_surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["owned_bytes"].as_u64().unwrap())
        .sum::<u64>()
        + resources["music_decoded_bytes"].as_u64().unwrap()
        + resources["target_bytes_all_surfaces"].as_u64().unwrap();
    assert_eq!(
        selected,
        resources["selected_owned_total"].as_u64().unwrap()
    );
    let metrics = serde_json::json!({"cold_host_us":cold,"cases":results,"working_set_before":memory_before,"working_set_after":working_set(),"targets":targets,"pipelines":pipelines,"resources":resources,"gpu_time":"unavailable; CPU submission wall timing is not GPU timing","driver_residency":"unavailable","samples_per_case":60});
    std::fs::create_dir_all("research").unwrap();
    std::fs::write(
        "research/834-stability.json",
        serde_json::to_vec_pretty(&metrics).unwrap(),
    )
    .unwrap();
    println!("{metrics}");
    assert!(host.errors.is_empty());
}
