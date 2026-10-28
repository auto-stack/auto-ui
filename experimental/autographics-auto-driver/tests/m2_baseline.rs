use autographics_auto_driver::authoring::{
    behavior::{Api, SourceSnapshot},
    measurement::{Measurement, SAMPLE_LIMIT},
};
use std::{path::Path, time::Instant};

#[test]
fn default_off_bounded_scalars_and_disable_clear() {
    let mut m = Measurement::default();
    assert!(m.start().is_none());
    m.finish(None);
    assert!(m.samples_ns.is_empty());
    m.enable(true);
    for _ in 0..SAMPLE_LIMIT + 3 {
        m.finish(Some(Instant::now()));
    }
    assert_eq!(m.samples_ns.len(), SAMPLE_LIMIT);
    assert_eq!(m.dropped, 3);
    m.enable(false);
    assert!(m.samples_ns.is_empty());
    assert_eq!(m.dropped, 0);
    assert!(m.start().is_none());
}
#[test]
fn all_five_actual_generated_regions_raw_transport_and_diagnostics() {
    for api in [
        Api::V1,
        Api::SpatialV2,
        Api::SegmentV3,
        Api::SweepV4,
        Api::MotionV5,
    ] {
        for crlf in [false, true] {
            let raw="// Unicode 基线\nfn app_start(){let reused=1}\nfn app_update(dt float){let reused=2}\n";
            let raw = if crlf {
                raw.replace('\n', "\r\n")
            } else {
                raw.into()
            };
            let s = SourceSnapshot::from_bytes_for(
                Path::new("fixture.at"),
                raw.as_bytes().to_vec(),
                api,
            )
            .unwrap();
            assert_eq!(s.bytes, raw.as_bytes());
            assert_eq!(s.regions.len(), 4);
            assert_eq!(
                &s.source.as_bytes()[s.regions[2].start..s.regions[2].end],
                s.bytes
            );
            assert_eq!(s.user_hash, agp::renderer::capture::sha256_hex(&s.bytes));
            assert_eq!(
                s.combined_hash,
                agp::renderer::capture::sha256_hex(s.source.as_bytes())
            );
            assert_eq!(s.version, api.version());
            assert!(s
                .diagnostic("runtime", "app_update", "bad")
                .contains("runtime/app_update"));
        }
        let e =
            SourceSnapshot::from_bytes_for(Path::new("bad.at"), b"fn app_start( {".to_vec(), api)
                .unwrap_err();
        assert!(e.contains("parse") && e.contains("user line"), "{e}");
    }
    assert!(Api::parse("autographics.behavior/6").is_err());
}
