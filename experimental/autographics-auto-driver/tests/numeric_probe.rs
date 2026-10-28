use autographics_auto_driver::{ResultKind, Scalar::*, Session};
#[test]
fn initialize_container_and_repeated_f32_updates() {
    let mut s = Session::new(include_str!("../samples/breakout/main.at")).unwrap();
    s.call("init", &[], ResultKind::Unit).unwrap();
    assert_eq!(
        s.float("value", &[Int(2)]).unwrap().to_bits(),
        ((0.5f32 / (0.5f32).sqrt()) * 400.0).to_bits()
    );
    assert_eq!(s.int("integer", &[Int(0)]).unwrap(), 0);
    for i in 0..56 {
        assert_eq!(s.int("brick", &[Int(i)]).unwrap(), 1);
    }
    s.call("update", &[Int(1)], ResultKind::Unit).unwrap();
    assert_eq!(
        s.float("value", &[Int(4)]).unwrap(),
        500.0f32 * (1.0f32 / 60.0)
    );
}
