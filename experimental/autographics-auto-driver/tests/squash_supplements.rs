use autographics_auto_driver::{squash::source, ResultKind, Scalar::*, Session};
fn fixture() -> Session {
    let text = format!(
        "{}\n{}",
        source(),
        r#"
fn fixture_player(x float,y float,z float,yaw float) { px=x;py=y;pz=z;facing=yaw;step=60 }
fn fixture_mob(x float,y float,z float,yaw float) { mx[0]=x;my[0]=y;mz[0]=z;mvx[0]=-14.0*sine(yaw);mvz[0]=-14.0*cosine(yaw);alive[0]=1;slot_count=1 }
"#
    );
    let mut s = Session::new(&text).unwrap();
    s.call("init", &[], ResultKind::Unit).unwrap();
    s
}
#[test]
fn spawn_targets_current_player_and_random_sequence_is_transactional() {
    let mut s = fixture();
    s.call(
        "fixture_player",
        &[Float(7.0), Float(0.792278), Float(-4.0), Float(0.0)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call("begin_tick", &[Int(0), Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    assert_eq!(s.int("random_request", &[]).unwrap(), 1);
    let mut rng = agp::core::DeterministicRng::new(27);
    let r = rng.next_f32();
    let jitter = -std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_2 * rng.next_f32();
    let speed = 10.0 + 8.0 * rng.next_f32();
    assert_eq!(s.int("random_value", &[Int(2), Float(r)]).unwrap(), 0);
    assert_eq!(s.int("random_count", &[]).unwrap(), 0);
    let mut rng = agp::core::DeterministicRng::new(27);
    for sequence in 1..=3 {
        assert_eq!(
            s.int("random_value", &[Int(sequence), Float(rng.next_f32())])
                .unwrap(),
            1
        );
    }
    let (x, z) = agp::games::squash::spawn_position(r);
    let yaw = (-(7.0 - x)).atan2(-(-4.0 - z)) + jitter;
    let vx = s.float("mob_value", &[Int(0), Int(2)]).unwrap();
    let vz = s.float("mob_value", &[Int(0), Int(3)]).unwrap();
    assert!((vx - (-speed * yaw.sin())).abs() < 0.001);
    assert!((vz - (-speed * yaw.cos())).abs() < 0.001);
    assert_eq!(s.int("random_value", &[Int(4), Float(0.5)]).unwrap(), 0);
    assert_eq!(s.int("random_count", &[]).unwrap(), 3);
}
#[test]
fn detector_channel_and_global_rotations_match_frozen_sibling_cases() {
    use glam::{Mat4, Vec3};
    for (offset, relative, expected) in [
        (Vec3::new(-2.0, 0.5, -0.7), 0.0, false),
        (Vec3::new(0.716365, 1.0 - 0.792278, 2.1), 0.0, true),
        (Vec3::new(0.455268, 0.95 - 0.792278, 1.4423), 0.0, true),
        (Vec3::new(0.716365, 2.0 - 0.792278, 2.1), 0.0, false),
    ] {
        for yaw in [
            0.0,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            -1.1,
            0.7,
        ] {
            let player = Vec3::new(0.0, 0.792278, 0.0);
            let mob = player + Mat4::from_rotation_y(yaw).transform_vector3(offset);
            let mut s = fixture();
            s.call(
                "fixture_player",
                &[
                    Float(player.x),
                    Float(player.y),
                    Float(player.z),
                    Float(yaw),
                ],
                ResultKind::Unit,
            )
            .unwrap();
            s.call(
                "fixture_mob",
                &[
                    Float(mob.x),
                    Float(mob.y),
                    Float(mob.z),
                    Float(yaw + relative),
                ],
                ResultKind::Unit,
            )
            .unwrap();
            assert_eq!(
                s.int("detector", &[Int(0)]).unwrap(),
                expected as i32,
                "offset={offset:?} yaw={yaw}"
            );
            if offset == Vec3::new(0.716365, 1.0 - 0.792278, 2.1) {
                assert_eq!(
                    s.int("contact", &[Int(0)]).unwrap(),
                    0,
                    "independent detector needs no body contact"
                );
            }
        }
    }
}

#[test]
fn reused_slots_preserve_birth_order_and_at_most_one_squash() {
    let text = format!(
        "{}\n{}",
        source(),
        r#"
fn contact_pair(){slot_count=2;order=[1,0];alive[0]=1;alive[1]=1;ids[0]=1;ids[1]=0;px=0.0;py=1.8;pz=0.0;vy=0.0;step=1;mx[0]=0.0;mx[1]=0.0;my[0]=0.0;my[1]=0.0;mz[0]=0.0;mz[1]=0.0}
"#
    );
    let mut s = Session::new(&text).unwrap();
    s.call("init", &[], ResultKind::Unit).unwrap();
    s.call("contact_pair", &[], ResultKind::Unit).unwrap();
    s.call("finish_tick", &[], ResultKind::Unit).unwrap();
    assert_eq!(s.int("event_count", &[]).unwrap(), 1);
    assert_eq!(s.int("event_field", &[Int(0), Int(0)]).unwrap(), 1);
    assert_eq!(s.int("event_field", &[Int(0), Int(1)]).unwrap(), 0);
    assert_eq!(s.int("status", &[Int(0)]).unwrap(), 1);
    assert_eq!(s.int("object_alive", &[Int(1)]).unwrap(), 1);
    assert_eq!(s.int("object_alive", &[Int(2)]).unwrap(), 0);
}

#[test]
fn axis_aligned_back_movement_preserves_frozen_facing_angle() {
    let mut s = Session::new(&source()).unwrap();
    s.call("init", &[], ResultKind::Unit).unwrap();
    s.call("begin_tick", &[Int(0), Int(1), Int(0)], ResultKind::Unit)
        .unwrap();
    s.call("finish_tick", &[], ResultKind::Unit).unwrap();
    let got = s.float("state", &[Int(4)]).unwrap();
    let expected = (-0.0f32).atan2(-14.0);
    assert!(
        (got - expected).abs() < 0.00001,
        "raw facing {got} differs from frozen {expected}"
    );
}

#[test]
fn axis_aligned_spawn_preserves_raw_yaw_without_wrapping_it() {
    let mut s = fixture();
    s.call(
        "fixture_player",
        &[Float(-0.5), Float(0.792278), Float(0.0), Float(0.0)],
        ResultKind::Unit,
    )
    .unwrap();
    s.call("begin_tick", &[Int(0), Int(0), Int(0)], ResultKind::Unit)
        .unwrap();
    for (i, v) in [0.125, 0.5, 0.5].into_iter().enumerate() {
        assert_eq!(
            s.int("random_value", &[Int(i as i32 + 1), Float(v)])
                .unwrap(),
            1
        );
    }
    assert_eq!(
        s.float("mob_value", &[Int(0), Int(5)]).unwrap(),
        -std::f32::consts::PI
    );
}
