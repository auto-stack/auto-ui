use autographics_auto_driver::{Progress, ResultKind, Scalar::*, Session};
#[test]
fn scalar_callbacks_state_f32_and_source_control() {
    let source = include_str!("../research/scalar.at");
    let mut s = Session::new(source).unwrap();
    assert_eq!(s.float("mix", &[Int(7), Float(1.25)]).unwrap(), 68.75);
    assert_eq!(s.float("update", &[Int(3), Float(0.5)]).unwrap(), 1.5);
    assert_eq!(s.float("update", &[Int(4), Float(0.5)]).unwrap(), 3.5);
    let mut altered = Session::new(&source.replace("10.0", "20.0")).unwrap();
    assert_eq!(
        altered.float("mix", &[Int(7), Float(1.25)]).unwrap(),
        138.75
    );
    assert_eq!(s.float("update", &[Int(1), Float(0.5)]).unwrap(), 4.0);
}
#[test]
fn missing_wrong_return_and_loop_are_terminal() {
    let source = include_str!("../research/scalar.at");
    let mut s = Session::new(source).unwrap();
    assert!(s.int("missing", &[]).unwrap_err().contains("not found"));
    assert!(s.closed());
    let mut s = Session::new(source).unwrap();
    assert!(s
        .int("mix", &[Int(7), Float(1.25)])
        .unwrap_err()
        .contains("type mismatch"));
    assert!(s.closed());
    let mut s = Session::new(source).unwrap();
    let mut p = s.start("spin", &[], ResultKind::Unit).unwrap();
    assert!(matches!(p, Progress::Pending));
    assert!(s
        .start("mix", &[], ResultKind::Float)
        .unwrap_err()
        .contains("already running"));
    loop {
        match p {
            Progress::Pending => match s.pump() {
                Ok(v) => p = v,
                Err(e) => {
                    assert!(e.contains("budget"));
                    break;
                }
            },
            _ => panic!("loop completed"),
        }
    }
    assert!(s.yielded > 0);
    assert!(s.closed());
    assert!(s.int("mix", &[]).is_err());
    let mut s = Session::new(source).unwrap();
    assert!(matches!(
        s.start("spin", &[], ResultKind::Unit).unwrap(),
        Progress::Pending
    ));
    s.close();
    assert!(s.pump().is_err());
}

#[test]
fn wrong_argument_type_or_count_is_refused_before_dispatch() {
    for args in [vec![Float(7.0), Int(1)], vec![Int(7)]] {
        let mut s = Session::new(include_str!("../research/scalar.at")).unwrap();
        assert!(s
            .float("mix", &args)
            .unwrap_err()
            .contains("argument type/count"));
        assert!(s.closed());
    }
}
