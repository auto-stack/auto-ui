use autographics_auto_driver::authoring::{document::Document, host::Host};
#[test]
fn orbit_and_pan_capture_only_end_on_the_owning_mouse_button() {
    let mut out = Vec::new();
    for orbit in [true, false] {
        for other_button in [false, true] {
            let mut h = Host::new(Document::sample()).unwrap();
            h.frame(None).unwrap();
            let initial = h.view3d();
            if orbit {
                h.navigation3d_down([400., 600.], true)
            } else {
                h.navigation_down([400., 600.])
            };
            h.pointer_move([440., 600.]);
            let first = h.view3d();
            if other_button {
                if orbit {
                    h.navigation_down([440., 600.]);
                    h.navigation_up([440., 600.]);
                } else {
                    h.navigation3d_down([440., 600.], true);
                    h.navigation3d_up([440., 600.]);
                }
            }
            h.pointer_move([480., 600.]);
            let second = h.view3d();
            if orbit {
                h.navigation3d_up([480., 600.])
            } else {
                h.navigation_up([480., 600.])
            };
            let changed = if orbit {
                (second.yaw - first.yaw).abs()
            } else {
                (second.target - first.target).length()
            };
            println!("orbit={orbit} other_button={other_button} changed_after_other_release={changed} initial={initial:?} first={first:?} second={second:?}");
            out.push((orbit, other_button, changed));
        }
    }
    assert!(
        out[0].2 > 0.01 && out[2].2 > 0.01,
        "normal uninterrupted controls must move"
    );
    assert!(
        out[1].2 > 0.01 && out[3].2 > 0.01,
        "releasing another mouse button must not terminate current capture"
    );
}
