use agp::assets::node_animation::{yaw, Animation};
use glam::Vec3;
use serde_json::{json, Value};

fn original() -> (Value, Vec<u8>) {
    let b = include_bytes!(
        "../../autographics-auto-driver/samples/textured_models/models/character.glb"
    );
    let n = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
    (
        serde_json::from_slice(&b[20..20 + n]).unwrap(),
        b[28 + n..].to_vec(),
    )
}
fn fixture(step: bool, negate: bool) -> (Value, Vec<u8>) {
    // Parent rotates 120 degrees about Y in 2s; child translates 1 -> 3 on X.
    // At 0.5s the parent must turn 30 degrees. Normalized component lerp would
    // produce ~27.8 degrees and fails the independent point assertion below.
    let sign = if negate { -1. } else { 1. };
    let data = [
        0.,
        2.,
        0.,
        0.,
        0.,
        1.,
        0.,
        sign * (3.0f32).sqrt() / 2.,
        0.,
        sign * 0.5,
        1.,
        0.,
        0.,
        3.,
        0.,
        0.,
    ];
    let bin = data.into_iter().flat_map(f32::to_le_bytes).collect();
    let j = json!({"nodes":[{"children":[1]},{"translation":[1,0,0]}],
      "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":8},{"buffer":0,"byteOffset":8,"byteLength":32},{"buffer":0,"byteOffset":40,"byteLength":24}],
      "accessors":[{"bufferView":0,"componentType":5126,"count":2,"type":"SCALAR"},{"bufferView":1,"componentType":5126,"count":2,"type":"VEC4"},{"bufferView":2,"componentType":5126,"count":2,"type":"VEC3"}],
      "animations":[{"name":"turn","samplers":[{"input":0,"output":1,"interpolation":if step {"STEP"} else {"LINEAR"}},{"input":0,"output":2}],"channels":[{"sampler":0,"target":{"node":0,"path":"rotation"}},{"sampler":1,"target":{"node":1,"path":"translation"}}]}]});
    (j, bin)
}
fn near(a: Vec3, b: Vec3) {
    assert!((a - b).length() < 2e-5, "{a:?} != {b:?}");
}

#[test]
fn fixed_character_all_four_clips_keep_original_hierarchy_and_move_nodes() {
    let (j, b) = original();
    let a = Animation::from_parts(&j, &b).unwrap();
    assert_eq!(a.nodes.len(), 8);
    assert_eq!(
        a.clips.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["static", "idle", "walk", "jump"]
    );
    assert_eq!(a.clips.iter().map(|c| c.channels.len()).sum::<usize>(), 44);
    assert_eq!(
        a.clips
            .iter()
            .flat_map(|c| &c.channels)
            .map(|c| c.times.len())
            .sum::<usize>(),
        337
    );
    assert!(a.cpu_bytes >= 6456);
    assert_eq!(a.nodes[7].parent, Some(4));
    let base = a.sample(None, 0., false).unwrap();
    near(
        base[5].transform_point3(Vec3::ZERO),
        Vec3::new(0.3, 0.375, -0.02375),
    );
    for id in 1..4 {
        let start = a.sample(Some(id), 0., false).unwrap();
        let mid = a
            .sample(Some(id), a.clips[id].duration * 0.37, false)
            .unwrap();
        assert!(
            start
                .iter()
                .zip(&mid)
                .any(|(a, b)| !a.abs_diff_eq(*b, 1e-5)),
            "clip{id} must animate actual nodes"
        );
    }
}

#[test]
fn independent_slerp_hierarchy_step_shortest_arc_and_phase_boundaries() {
    for negate in [false, true] {
        let (j, b) = fixture(false, negate);
        let a = Animation::from_parts(&j, &b).unwrap();
        let p = a.sample(Some(0), 0.5, false).unwrap();
        near(
            p[1].transform_point3(Vec3::ZERO),
            Vec3::new(1.5 * (3.0f32).sqrt() / 2., 0., -0.75),
        );
        let start = a.sample(Some(0), 0., false).unwrap();
        assert_eq!(a.sample(Some(0), 2., true).unwrap(), start);
        assert_eq!(a.sample(Some(0), 4., true).unwrap(), start);
        assert_eq!(
            a.sample(Some(0), 2., false).unwrap(),
            a.sample(Some(0), 99., false).unwrap()
        );
        near(
            a.sample(None, 50., true).unwrap()[1].transform_point3(Vec3::ZERO),
            Vec3::X,
        );
        let (j, b) = fixture(true, negate);
        let a = Animation::from_parts(&j, &b).unwrap();
        near(
            a.sample(Some(0), 0.5, false).unwrap()[1].transform_point3(Vec3::ZERO),
            Vec3::new(1.5, 0., 0.),
        );
        near(
            a.sample(Some(0), 2., false).unwrap()[1].transform_point3(Vec3::ZERO),
            Vec3::new(-1.5, 0., -1.5 * (3.0f32).sqrt()),
        );
    }
    let (mut j, mut b) = fixture(false, false);
    // Single-key, zero-duration track stays at its one pose for all phases.
    j["accessors"][0]["count"] = json!(1);
    j["accessors"][1]["count"] = json!(1);
    j["accessors"][2]["count"] = json!(1);
    b[..4].copy_from_slice(&0f32.to_le_bytes());
    let a = Animation::from_parts(&j, &b).unwrap();
    assert_eq!(
        a.sample(Some(0), 0., false).unwrap(),
        a.sample(Some(0), 999., true).unwrap()
    );
    for value in [f32::NAN, f32::INFINITY, -1.] {
        assert!(a.sample(Some(0), value, false).is_err());
    }
    assert!(a.sample(Some(1), 0., false).is_err());
    assert_eq!(yaw(std::f32::consts::PI).unwrap(), -std::f32::consts::PI);
    assert_eq!(yaw(-std::f32::consts::PI).unwrap(), -std::f32::consts::PI);
    assert!(yaw(f32::NAN).is_err());
    assert!(yaw(1e10).is_err());
}

#[test]
fn malformed_tracks_and_unsupported_modes_are_classified_before_sampling() {
    let (j, b) = fixture(false, false);
    let cases: Vec<(&str, Box<dyn Fn(&mut Value)>, &str)> = vec![
        (
            "cubic",
            Box::new(|j| j["animations"][0]["samplers"][0]["interpolation"] = json!("CUBICSPLINE")),
            "unsupported:",
        ),
        (
            "weights",
            Box::new(|j| j["animations"][0]["channels"][0]["target"]["path"] = json!("weights")),
            "unsupported:",
        ),
        (
            "skin",
            Box::new(|j| j["skins"] = json!([{}])),
            "unsupported:",
        ),
        (
            "morph",
            Box::new(|j| j["nodes"][0]["weights"] = json!([0])),
            "unsupported:",
        ),
        (
            "duplicate",
            Box::new(|j| {
                let c = j["animations"][0]["channels"][0].clone();
                j["animations"][0]["channels"]
                    .as_array_mut()
                    .unwrap()
                    .push(c);
            }),
            "invalid:",
        ),
        (
            "count",
            Box::new(|j| j["accessors"][1]["count"] = json!(1)),
            "invalid:",
        ),
        (
            "span",
            Box::new(|j| j["bufferViews"][1]["byteLength"] = json!(31)),
            "invalid:",
        ),
        (
            "huge span",
            Box::new(|j| j["accessors"][1]["byteOffset"] = json!(u64::MAX)),
            "invalid:",
        ),
        (
            "type",
            Box::new(|j| j["accessors"][1]["type"] = json!("VEC3")),
            "invalid:",
        ),
        (
            "normalization",
            Box::new(|j| j["accessors"][1]["normalized"] = json!(true)),
            "invalid:",
        ),
        (
            "node",
            Box::new(|j| j["animations"][0]["channels"][0]["target"]["node"] = json!(2)),
            "invalid:",
        ),
        (
            "parent",
            Box::new(|j| j["nodes"][0]["children"] = json!([1, 1])),
            "invalid:",
        ),
        (
            "cycle",
            Box::new(|j| j["nodes"][1]["children"] = json!([0])),
            "invalid:",
        ),
        (
            "negative scale",
            Box::new(|j| j["nodes"][0]["scale"] = json!([-1, 1, 1])),
            "invalid:",
        ),
        (
            "singular scale",
            Box::new(|j| j["nodes"][0]["scale"] = json!([0, 1, 1])),
            "invalid:",
        ),
        (
            "quaternion",
            Box::new(|j| j["nodes"][0]["rotation"] = json!([0, 0, 0, 0])),
            "invalid:",
        ),
        (
            "nodes129",
            Box::new(|j| j["nodes"] = json!(vec![json!({}); 129])),
            "resource limit:",
        ),
        (
            "clips17",
            Box::new(|j| {
                let c = j["animations"][0].clone();
                j["animations"] = json!(vec![c; 17]);
            }),
            "resource limit:",
        ),
        (
            "channels129",
            Box::new(|j| {
                let c = j["animations"][0]["channels"][0].clone();
                j["animations"][0]["channels"] = json!(vec![c; 129]);
            }),
            "resource limit:",
        ),
    ];
    for (name, mutate, prefix) in cases {
        let mut j = j.clone();
        mutate(&mut j);
        let e = Animation::from_parts(&j, &b).unwrap_err();
        assert!(e.starts_with(prefix), "{name}: {e}");
    }
    for (name, offset, value) in [
        ("duplicate time", 4, 0f32),
        ("negative time", 0, -1.),
        ("nan output", 8, f32::NAN),
        ("nonunit quaternion", 20, 0.),
    ] {
        let mut b = b.clone();
        b[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            Animation::from_parts(&j, &b)
                .unwrap_err()
                .starts_with("invalid:"),
            "{name}"
        );
    }
}

#[test]
fn matrix_parent_is_preserved_without_decomposition_and_sample_limit_is_exact() {
    let (mut j, b) = fixture(false, false);
    // Matrix parent itself has no animated channel; the translated child does.
    j["animations"][0]["channels"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    j["nodes"][0]["matrix"] = json!([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 7, 0, 0, 1]);
    let a = Animation::from_parts(&j, &b).unwrap();
    near(
        a.sample(Some(0), 0.5, false).unwrap()[1].transform_point3(Vec3::ZERO),
        Vec3::new(8.5, 0., 0.),
    );
    let (mut j, _) = fixture(false, false);
    let n = 4096;
    let mut b = Vec::new();
    for i in 0..n {
        b.extend_from_slice(&(i as f32).to_le_bytes());
    }
    for _ in 0..n {
        for v in [0f32, 0., 0., 1.] {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    for _ in 0..n {
        for v in [1f32, 0., 0.] {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    j["bufferViews"] = json!([{"buffer":0,"byteLength":n*4},{"buffer":0,"byteOffset":n*4,"byteLength":n*16},{"buffer":0,"byteOffset":n*20,"byteLength":n*12}]);
    for a in j["accessors"].as_array_mut().unwrap() {
        a["count"] = json!(n);
    }
    let a = Animation::from_parts(&j, &b).unwrap();
    assert_eq!(
        a.clips[0]
            .channels
            .iter()
            .map(|c| c.times.len())
            .sum::<usize>(),
        8192
    );
    // An otherwise valid third channel exceeds the aggregate bound before
    // allocating key/output vectors; distinct target prevents duplicate error.
    j["animations"][0]["channels"]
        .as_array_mut()
        .unwrap()
        .push(json!({"sampler":1,"target":{"node":0,"path":"translation"}}));
    assert!(Animation::from_parts(&j, &b)
        .unwrap_err()
        .starts_with("resource limit:"));
}

#[test]
fn preflight_and_retention_agree_and_large_finite_linear_endpoints_do_not_overflow() {
    let (mut j, mut bin) = fixture(false, false);
    // Child translation endpoints are -3e38 and +3e38 on X. Their midpoint
    // is exactly zero; subtract-then-lerp would overflow before weighting.
    bin[40..44].copy_from_slice(&(-3e38f32).to_le_bytes());
    bin[52..56].copy_from_slice(&(3e38f32).to_le_bytes());
    let bytes = Animation::preflight(&j, &bin).unwrap();
    let a = Animation::from_parts(&j, &bin).unwrap();
    assert_eq!(bytes, a.cpu_bytes);
    near(
        a.sample(Some(0), 1., false).unwrap()[1].transform_point3(Vec3::ZERO),
        Vec3::ZERO,
    );
    // Direct public AGP admission also checks every channel's values before
    // allocating any retained key/output vectors, not just the driver wrapper.
    bin[8..12].copy_from_slice(&f32::NAN.to_le_bytes());
    let before = agp::assets::node_animation::channel_allocation_count();
    assert!(Animation::from_parts(&j, &bin).is_err());
    assert_eq!(
        agp::assets::node_animation::channel_allocation_count(),
        before
    );
    j["animations"][0]["channels"] = serde_json::json!([]);
    assert!(Animation::preflight(&j, &bin)
        .unwrap_err()
        .starts_with("invalid:"));
}
