//! Composed-pixel gate controls (review r6 F829-R6-02): the analyzer the
//! iced case consumes must independently anchor the expected UI/view
//! geometry and observe out-of-bounds content for BOTH views. Bound
//! corrupt inputs regenerate R6's exact mutations on a constructed
//! positive composite (the same byte-path the real screenshots take).
//! Live positive/negative evidence: the iced-shared-device case runs
//! (positive exit 0; AGP_SABOTAGE_ICED=transparent_blit exit 1).

use agp::host::iced_spike::analyze_composite_shot;

const W: u32 = 2000;
const H: u32 = 1600;
const SCALE: f32 = 2.0;
// Anchored layout (the spike's own view()): padding 10, control band 31,
// spacing 8, Fixed(320) views — physical = logical * SCALE. When the
// column overflows the window content box, Iced squeezes the LAST view
// (F829-R10-01): e.g. 860x700@2 keeps 2D at 320 logical and constrains
// 3D to 313 logical (physical y=[754,1380)).
const PAD: u32 = 20;
const CTRL_BOT: u32 = 82; // (10 + 31) * 2
const V2_TOP: u32 = 98; // + 8 * 2
const VIEW_H: u32 = 640;
const GAP: u32 = 16;
const V2_BOT: u32 = V2_TOP + VIEW_H;
const V3_TOP: u32 = V2_BOT + GAP; // 754
const V3_BOT: u32 = V3_TOP + VIEW_H; // 1394

// 860x700 logical @2 = 1720x1400 physical: last Fixed(320) is constrained
// to 313 logical = 626 physical (content_h 1360 = 62+16+640+16+626).
const RESIZE_W: u32 = 1720;
const RESIZE_H: u32 = 1400;
const RESIZE_V3_H: u32 = 626;
const RESIZE_V3_BOT: u32 = V3_TOP + RESIZE_V3_H; // 1380

const BG: [u8; 3] = [255, 255, 255];
const V2_BG: [u8; 3] = [20, 23, 31];
const SKY: [u8; 3] = [165, 167, 171];
const GROUND: [u8; 3] = [224, 224, 224];
const BUTTON: [u8; 3] = [193, 198, 250];
const TEXT: [u8; 3] = [60, 45, 90];
const MODEL: [u8; 3] = [40, 20, 10];

fn set_px(img: &mut [u8], x: u32, y: u32, c: [u8; 3]) {
    let i = ((y * W + x) * 4) as usize;
    img[i] = c[0];
    img[i + 1] = c[1];
    img[i + 2] = c[2];
    img[i + 3] = 255;
}

fn fill(img: &mut [u8], x0: u32, y0: u32, x1: u32, y1: u32, c: [u8; 3]) {
    for y in y0..y1.min(H) {
        for x in x0..x1.min(W) {
            set_px(img, x, y, c);
        }
    }
}

/// Construct the normal positive composite: white background, a filled
/// button + text in the control band, the 2D view cleared to its
/// source-anchored color, the 3D view with sky/ground/model content —
/// all at the anchored rects.
fn synth_positive() -> Vec<u8> {
    let mut img = vec![0u8; (W * H * 4) as usize];
    fill(&mut img, 0, 0, W, H, BG);
    // control band: button fill + text glyphs
    fill(&mut img, PAD, 24, 420, CTRL_BOT - 6, BUTTON);
    for i in 0..6 {
        fill(&mut img, 60 + i * 48, 40, 76 + i * 48, 66, TEXT);
    }
    fill(&mut img, 500, 40, 900, 50, TEXT);
    // 2D view: clear color
    fill(&mut img, PAD, V2_TOP, W - PAD, V2_BOT, V2_BG);
    // a couple of animated quads (content inside the rect)
    fill(
        &mut img,
        300,
        V2_TOP + 200,
        380,
        V2_TOP + 260,
        [230, 90, 76],
    );
    fill(
        &mut img,
        1500,
        V2_TOP + 380,
        1560,
        V2_TOP + 440,
        [51, 153, 230],
    );
    // 3D view: sky upper 55%, ground lower 45%, one model
    let horizon = V3_TOP + VIEW_H * 55 / 100;
    fill(&mut img, PAD, V3_TOP, W - PAD, horizon, SKY);
    fill(&mut img, PAD, horizon, W - PAD, V3_BOT, GROUND);
    fill(&mut img, 950, horizon - 90, 1050, horizon + 30, MODEL);
    img
}

fn analyze(tag: &str, img: &[u8]) -> agp::host::iced_spike::CompositeShotFacts {
    analyze_composite_shot(tag, img, W, H, SCALE)
}

#[test]
fn r6_composite_gate_normal_positive() {
    let f = analyze("positive", &synth_positive());
    assert!(f.ok(), "normal positive must pass: {f:?}");
    assert!(f.position_ok, "anchored position must hold: {:?}", f.detail);
}

#[test]
fn r6_composite_gate_transparent_negative() {
    // the R4/R6 transparent mutation: everything is the flat window bg.
    let img = vec![255u8; (W * H * 4) as usize];
    let f = analyze("transparent", &img);
    assert!(!f.ok(), "fully transparent composite must fail: {f:?}");
    assert!(!f.non_blank && !f.view2d_present && !f.view3d_present);
}

#[test]
fn r6_composite_gate_3d_bleed_negative() {
    // R6 bound corrupt input #1: recolor the padding strips alongside the
    // 3D view to the source sky color — 25,600 px outside the view.
    let mut img = synth_positive();
    fill(&mut img, 0, V3_TOP, PAD, V3_BOT, SKY);
    fill(&mut img, W - PAD, V3_TOP, W, V3_BOT, SKY);
    let f = analyze("3d-bleed", &img);
    assert!(
        !f.ok() && !f.clip_ok,
        "3D sky pixels outside the 3D rect must trip the clip gate: {f:?}"
    );
}

#[test]
fn r6_composite_gate_displaced_views_negative() {
    // R6 bound corrupt input #2: shift BOTH complete views down 80 physical
    // px on the same fixed-size window (observed top 178 vs expected 98).
    let mut img = synth_positive();
    let mut shifted = vec![255u8; (W * H * 4) as usize];
    for y in 0..(H - 80) {
        for x in 0..W {
            let src = ((y * W + x) * 4) as usize;
            let dst = (((y + 80) * W + x) * 4) as usize;
            shifted[dst..dst + 4].copy_from_slice(&img[src..src + 4]);
        }
    }
    std::mem::swap(&mut img, &mut shifted);
    let f = analyze("displaced", &img);
    // The anchored position fact fires even though the shifted rects still
    // overlap the expected ones (partial-overlap view facts can stay true;
    // the position invariant is the binding control).
    assert!(
        !f.ok() && !f.position_ok,
        "displaced views must fail the anchored position gate: {f:?}"
    );
    assert!(
        !f.position_2d_ok && !f.position_3d_ok,
        "both views moved: BOTH per-view position facts must fire: {f:?}"
    );
}

/// F829-R8-01 bound input #1: move ONLY the 3D view down 80 physical px
/// (2D view and controls untouched) — the per-view position fact must
/// fire independently for the 3D view while the 2D view stays green.
#[test]
fn r8_3d_only_displacement_negative() {
    let mut img = synth_positive();
    // extract the 3D view content rows and rewrite them 80 px lower
    let mut content = vec![[0u8; 4]; (W * VIEW_H) as usize];
    for dy in 0..VIEW_H {
        for x in 0..W {
            let src = (((V3_TOP + dy) * W + x) * 4) as usize;
            content[(dy * W + x) as usize] = [img[src], img[src + 1], img[src + 2], img[src + 3]];
        }
    }
    fill(&mut img, PAD, V3_TOP, W - PAD, V3_BOT, BG); // vacate the rect
    for dy in 0..VIEW_H {
        for x in 0..W {
            let dst = (((V3_TOP + 80 + dy) * W + x) * 4) as usize;
            let c = content[(dy * W + x) as usize];
            img[dst] = c[0];
            img[dst + 1] = c[1];
            img[dst + 2] = c[2];
            img[dst + 3] = 255;
        }
    }
    let f = analyze("3d-only-displaced", &img);
    assert!(
        !f.ok() && !f.position_3d_ok,
        "3D-only displacement must fire the 3D position fact: {f:?}"
    );
    assert!(
        f.position_2d_ok,
        "the untouched 2D view must stay green (per-view independence): {f:?}"
    );
}

/// F829-R8-01 bound input #2: 156,800 sky-colored pixels (1960×80) just
/// outside the 3D view's BOTTOM edge — the full-perimeter clip must fire.
#[test]
fn r8_bottom_bleed_negative() {
    let mut img = synth_positive();
    fill(&mut img, PAD, V3_BOT, W - PAD, V3_BOT + 80, SKY);
    let f = analyze("bottom-bleed", &img);
    assert!(
        !f.ok() && !f.clip_ok,
        "sky below the 3D view must trip the full-perimeter clip: {f:?}"
    );
}

/// F829-R8-01 bound input #3: 3D-only down-80 with the excess below
/// y=1394 removed (blank leading band, nothing outside the bottom edge).
/// Placement must still fail independently of clipping.
#[test]
fn r8_3d_only_displacement_clipped_negative() {
    let mut img = synth_positive();
    let mut content = vec![[0u8; 4]; (W * VIEW_H) as usize];
    for dy in 0..VIEW_H {
        for x in 0..W {
            let src = (((V3_TOP + dy) * W + x) * 4) as usize;
            content[(dy * W + x) as usize] = [img[src], img[src + 1], img[src + 2], img[src + 3]];
        }
    }
    // vacate the original rect, then write only the rows that still fit
    fill(&mut img, PAD, V3_TOP, W - PAD, V3_BOT, BG);
    for dy in 0..VIEW_H {
        let y = V3_TOP + 80 + dy;
        if y >= V3_BOT {
            break;
        }
        for x in 0..W {
            let dst = ((y * W + x) * 4) as usize;
            let c = content[(dy * W + x) as usize];
            img[dst] = c[0];
            img[dst + 1] = c[1];
            img[dst + 2] = c[2];
            img[dst + 3] = 255;
        }
    }
    let f = analyze("3d-only-displaced-clipped", &img);
    assert!(
        !f.ok() && !f.position_3d_ok,
        "clipped 3D-only displacement must still fail placement: {f:?}"
    );
    assert!(
        f.clip_ok,
        "clipped variant must NOT trip clip (placement vs overflow separated): {f:?}"
    );
    assert!(
        f.position_2d_ok,
        "the untouched 2D view must stay green: {f:?}"
    );
}

#[test]
fn r6_composite_gate_resize_dpi_control() {
    // Resized window 860x700 logical @2x = 1720x1400 physical: same
    // anchored tops, narrower content, and the REAL constrained Iced
    // layout (F829-R10-01) — 3D is 313 logical, not the nominal 320.
    let w = RESIZE_W;
    let h = RESIZE_H;
    let mut img = vec![0u8; (w * h * 4) as usize];
    let set = |img: &mut [u8], x: u32, y: u32, c: [u8; 3]| {
        let i = ((y * w + x) * 4) as usize;
        img[i] = c[0];
        img[i + 1] = c[1];
        img[i + 2] = c[2];
        img[i + 3] = 255;
    };
    let fillw = |img: &mut [u8], x0: u32, y0: u32, x1: u32, y1: u32, c: [u8; 3]| {
        for y in y0..y1.min(h) {
            for x in x0..x1.min(w) {
                set(img, x, y, c);
            }
        }
    };
    fillw(&mut img, 0, 0, w, h, BG);
    fillw(&mut img, PAD, 24, 420, CTRL_BOT - 6, BUTTON);
    fillw(&mut img, PAD, V2_TOP, w - PAD, V2_BOT, V2_BG);
    let horizon = V3_TOP + RESIZE_V3_H * 55 / 100;
    fillw(&mut img, PAD, V3_TOP, w - PAD, horizon, SKY);
    fillw(&mut img, PAD, horizon, w - PAD, RESIZE_V3_BOT, GROUND);
    fillw(&mut img, 950, horizon - 90, 1050, horizon + 30, MODEL);
    let f = analyze_composite_shot("resized", &img, w, h, SCALE);
    assert!(f.ok(), "resized/DPI control must stay green: {f:?}");
    assert!(
        f.position_3d_ok,
        "constrained 3D rect must be accepted: {:?}",
        f.detail
    );
}

/// F829-R8-01 / R10: each view may not slide left/right inside the
/// original clip rect (blank band at one edge). Placement must fail,
/// clipping of the leftover content stays true.
#[test]
fn r10_2d_only_dx_clipped_negative() {
    for dx in [-80i32, 80i32] {
        let mut img = synth_positive();
        // vacate then re-copy 2D content shifted horizontally, clipped to
        // the original rect.
        let mut content = vec![[0u8; 4]; (W * VIEW_H) as usize];
        for dy in 0..VIEW_H {
            for x in 0..W {
                let src = (((V2_TOP + dy) * W + x) * 4) as usize;
                content[(dy * W + x) as usize] =
                    [img[src], img[src + 1], img[src + 2], img[src + 3]];
            }
        }
        fill(&mut img, PAD, V2_TOP, W - PAD, V2_BOT, BG);
        for dy in 0..VIEW_H {
            for x in PAD..(W - PAD) {
                let tx = x as i32 + dx;
                if tx >= PAD as i32 && tx < (W - PAD) as i32 {
                    let dst = (((V2_TOP + dy) * W + tx as u32) * 4) as usize;
                    let c = content[(dy * W + x) as usize];
                    img[dst] = c[0];
                    img[dst + 1] = c[1];
                    img[dst + 2] = c[2];
                    img[dst + 3] = 255;
                }
            }
        }
        let f = analyze("2d-dx", &img);
        assert!(
            !f.ok() && !f.position_2d_ok,
            "2D dx={dx} must fail horizontal placement: {f:?}"
        );
        assert!(f.position_3d_ok, "3D untouched must stay green: {f:?}");
        assert!(f.clip_ok, "clipped shift is placement, not overflow: {f:?}");
    }
}

#[test]
fn r10_3d_only_dx_clipped_negative() {
    for dx in [-80i32, 80i32] {
        let mut img = synth_positive();
        let mut content = vec![[0u8; 4]; (W * VIEW_H) as usize];
        for dy in 0..VIEW_H {
            for x in 0..W {
                let src = (((V3_TOP + dy) * W + x) * 4) as usize;
                content[(dy * W + x) as usize] =
                    [img[src], img[src + 1], img[src + 2], img[src + 3]];
            }
        }
        fill(&mut img, PAD, V3_TOP, W - PAD, V3_BOT, BG);
        for dy in 0..VIEW_H {
            for x in PAD..(W - PAD) {
                let tx = x as i32 + dx;
                if tx >= PAD as i32 && tx < (W - PAD) as i32 {
                    let dst = (((V3_TOP + dy) * W + tx as u32) * 4) as usize;
                    let c = content[(dy * W + x) as usize];
                    img[dst] = c[0];
                    img[dst + 1] = c[1];
                    img[dst + 2] = c[2];
                    img[dst + 3] = 255;
                }
            }
        }
        let f = analyze("3d-dx", &img);
        assert!(
            !f.ok() && !f.position_3d_ok,
            "3D dx={dx} must fail horizontal placement: {f:?}"
        );
        assert!(f.position_2d_ok, "2D untouched must stay green: {f:?}");
        assert!(f.clip_ok, "clipped shift is placement, not overflow: {f:?}");
    }
}

/// R10: visible out-of-bounds CONTENT (not only sky/clear) must trip clip.
#[test]
fn r10_ground_color_side_bleed_negative() {
    let mut img = synth_positive();
    fill(&mut img, 0, V3_TOP, PAD, V3_BOT, GROUND);
    fill(&mut img, W - PAD, V3_TOP, W, V3_BOT, GROUND);
    let f = analyze("ground-bleed", &img);
    assert!(
        !f.ok() && !f.clip_ok,
        "ground-colored side gutter must trip clip: {f:?}"
    );
}

#[test]
fn r10_model_color_side_bleed_negative() {
    let mut img = synth_positive();
    fill(&mut img, 0, V3_TOP, PAD, V3_BOT, MODEL);
    fill(&mut img, W - PAD, V3_TOP, W, V3_BOT, MODEL);
    let f = analyze("model-bleed", &img);
    assert!(
        !f.ok() && !f.clip_ok,
        "model-colored side gutter must trip clip: {f:?}"
    );
}

/// R12 F829-R8-01: the complete background complement — every pixel
/// outside both view rects and the legitimate controls row — must reject
/// visible content, including the previously unobserved bands.
#[test]
fn r12_pre_2d_gap_bleed_negative() {
    for (tag, c) in [("sky", SKY), ("ground", GROUND), ("model", MODEL)] {
        let mut img = synth_positive();
        // y=ctrl_bot..v2_top (82..98), x=pad..w-pad
        fill(&mut img, PAD, CTRL_BOT, W - PAD, V2_TOP, c);
        let f = analyze("pre-2d-gap", &img);
        assert!(
            !f.ok() && !f.clip_ok,
            "pre-2D gap {tag} must trip clip: {f:?}"
        );
    }
}

#[test]
fn r12_outer_control_gutter_bleed_negative() {
    for (tag, c) in [("sky", SKY), ("ground", GROUND), ("model", MODEL)] {
        let mut img = synth_positive();
        // side gutters of the controls row
        fill(&mut img, 0, PAD, PAD, CTRL_BOT, c);
        fill(&mut img, W - PAD, PAD, W, CTRL_BOT, c);
        let f = analyze("ctrl-gutter", &img);
        assert!(
            !f.ok() && !f.clip_ok,
            "outer control gutter {tag} must trip clip: {f:?}"
        );
    }
}

#[test]
fn r12_bottom_outer_corner_bleed_negative() {
    for (tag, c) in [("sky", SKY), ("ground", GROUND), ("model", MODEL)] {
        let mut img = synth_positive();
        fill(&mut img, 0, V3_BOT, PAD, V3_BOT + 80, c);
        fill(&mut img, W - PAD, V3_BOT, W, V3_BOT + 80, c);
        let f = analyze("bottom-corner", &img);
        assert!(
            !f.ok() && !f.clip_ok,
            "bottom outer corner {tag} must trip clip: {f:?}"
        );
    }
}

#[test]
fn r12_top_padding_bleed_negative() {
    // A real view color that is outside the named clear/sky/ground/modelish
    // signatures must still fail when painted into always-background top padding.
    for (tag, c) in [
        ("viewish", [24u8, 38u8, 52u8]),
        ("sky", SKY),
        ("ground", GROUND),
    ] {
        let mut img = synth_positive();
        fill(&mut img, PAD, 0, W - PAD, PAD, c);
        let f = analyze("top-pad", &img);
        assert!(
            !f.ok() && !f.clip_ok,
            "top padding {tag} must trip clip: {f:?}"
        );
    }
}

/// R10-02: delayed `CompositeShot` must block phase progression; a
/// missing request/completion is diagnosed, never counted as done.
#[test]
fn r10_screenshot_barrier_waits_for_completion() {
    use agp::host::iced_spike::{screenshot_barrier, ScreenshotBarrier};
    let received = |m: &str| m == "after_button";
    assert_eq!(
        screenshot_barrier(Some("initial"), &|m| m == "initial", "initial"),
        ScreenshotBarrier::Complete
    );
    assert_eq!(
        screenshot_barrier(Some("initial"), &|_| false, "initial"),
        ScreenshotBarrier::Wait
    );
    assert_eq!(
        screenshot_barrier(None, &|_| false, "initial"),
        ScreenshotBarrier::Missing
    );
    assert_eq!(
        screenshot_barrier(Some("after_button"), &received, "after_button"),
        ScreenshotBarrier::Complete
    );
}
