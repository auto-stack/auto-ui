use autographics_auto_driver::authoring::document::*;
#[test]
fn strict_codec_and_boundaries() {
    let d = Document::sample();
    assert_eq!(Document::decode(&d.encode().unwrap()).unwrap(), d);
    let b = String::from_utf8(d.encode().unwrap()).unwrap();
    for s in [
        b.replacen("{", "{\"schema\":\"x\",", 1),
        b.replace("\"objects\":", "\"unknown\":0,\"objects\":"),
        b.replace("autographics.scene/5", "autographics.scene/999"),
        b.replace("\"id\":2", "\"id\":1"),
        b.replace("\"circle\"", "\"bad\""),
        b.replace("\"visible\":true", "\"visible\":1"),
    ] {
        assert!(Document::decode(s.as_bytes()).is_err(), "{s}");
    }
    for n in [f32::NAN, f32::INFINITY, 100001.0] {
        let mut x = d.clone();
        x.objects[0].translation[0] = n;
        assert!(x.encode().is_err());
    }
    let mut x = d.clone();
    x.objects[0].name = "文".repeat(128);
    x.objects[0].translation[0] = 100000.0;
    x.objects[0].rgba = [0.0, 1.0, 0.0, 1.0];
    x.encode().unwrap();
    x.objects[0].name.push('文');
    assert!(x.encode().is_err());
    let mut x = Document::empty("max".into());
    for i in 1..=128 {
        x.objects.push(Object::new(i, Kind::Circle))
    }
    x.encode().unwrap();
    x.objects.push(Object::new(129, Kind::Circle));
    assert!(x.encode().is_err());
    assert!(Document::decode(&vec![b' '; MAX_FILE + 1]).is_err());
}
