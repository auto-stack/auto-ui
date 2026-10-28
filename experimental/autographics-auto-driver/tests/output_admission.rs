use autographics_auto_driver::{
    output::{Packet, Publication},
    ResultKind,
    Scalar::Int,
    Session,
};
fn packet() -> Packet {
    let mut s = Session::new(include_str!("../samples/scene-control/main.at")).unwrap();
    s.call("init", &[], ResultKind::Unit).unwrap();
    Packet::read(&mut s, 10, 1).unwrap()
}
#[test]
fn whole_packet_rejection_preserves_scene_score_audio_and_identity() {
    let p = packet();
    let mut out = Publication::new(10);
    out.publish(p.clone()).unwrap();
    for fault in 0..11 {
        let mut bad = p.clone();
        bad.sequence = 2;
        bad.objects[0].values[0] = 50.0;
        bad.score = 100;
        bad.sounds = 1;
        bad.events = vec![[0, 0, 0], [1, 1, 0]];
        match fault {
            0 => bad.objects[1].values[0] = f32::NAN,
            1 => bad.version = 2,
            2 => bad.objects[1].generation = 99,
            3 => bad.session = 11,
            4 => bad.objects[1].slot = 900,
            5 => bad.sequence = 1,
            6 => bad.objects[1].values[8] = 2.0,
            7 => bad.events[0][0] = 99,
            8 => bad.background[0] = f32::NAN,
            9 => bad.events = vec![[1, 1, 0]],
            _ => bad.events = vec![[0, 5, 0], [1, 1, 0], [2, 0, 1]],
        }
        assert!(out.publish(bad).is_err(), "fault {fault}");
        assert_eq!(out.last(), Some(&p));
    }
    let mut removed = p.clone();
    removed.sequence = 2;
    removed.objects.clear();
    out.publish(removed.clone()).unwrap();
    let mut stale = p.clone();
    stale.sequence = 3;
    assert!(out.publish(stale).is_err());
    assert_eq!(out.last(), Some(&removed));
    let mut fresh = p.clone();
    fresh.sequence = 3;
    for o in &mut fresh.objects {
        o.generation += 1;
    }
    out.publish(fresh).unwrap();
    out.close();
    let mut late = p;
    late.sequence = 999;
    assert!(out.publish(late).is_err());
}
#[test]
fn real_source_output_and_last_owners() {
    for _ in 0..20 {
        let mut s = Session::new(include_str!("../samples/scene-control/main.at")).unwrap();
        s.call("init", &[], ResultKind::Unit).unwrap();
        let p = Packet::read(&mut s, 1, 1).unwrap();
        assert_eq!(p.objects.len(), 2);
        s.call("update", &[Int(1)], ResultKind::Unit).unwrap();
        let moved = Packet::read(&mut s, 1, 2).unwrap();
        assert_eq!(moved.objects[0].values[0], p.objects[0].values[0] + 2.0);
        s.call("destroy", &[], ResultKind::Unit).unwrap();
        assert!(Packet::read(&mut s, 1, 3).unwrap().objects.is_empty());
        let alive = s.owner_probe();
        assert!(alive());
        s.close();
        assert!(!alive(), "last SDK owners remain");
    }
}
