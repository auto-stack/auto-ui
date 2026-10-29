//! PLAN-004 AGP sprite resources: the finite image domain, nearest/clamp
//! sampling, the UV-only flip, alpha over the clear colour, warm reuse and the
//! same-device/identity rules, all checked against real rendered pixels.
//!
//! `cargo test --locked --test sprite_resources -- --test-threads=1`
use agp::{
    assets::ImageData,
    core::scene::Color,
    host::surface_runtime::{Game2dContent, SurfaceContent, SurfaceRuntime},
    renderer::{
        capture::sha256_hex,
        sprite::{frame_uv, SpriteCmd, SpriteStore},
        target::OffscreenTarget,
        GpuDevice,
    },
};

fn image(width: u32, height: u32, rgba: &[u8]) -> ImageData {
    assert_eq!(rgba.len(), (width * height * 4) as usize);
    ImageData {
        width,
        height,
        rgba: rgba.to_vec(),
        source_sha256: sha256_hex(rgba),
    }
}

/// A full-surface quad (64x64 logical pixels in the 0..64 ortho camera).
fn quad(uv: [f32; 4], texture: &str) -> SpriteCmd {
    SpriteCmd {
        center: [32.0, 32.0],
        size: [64.0, 64.0],
        uv,
        color: [1.0, 1.0, 1.0, 1.0],
        texture: texture.to_owned(),
    }
}

fn surface() -> (SurfaceRuntime, agp::core::composition::SurfaceId) {
    let gpu = GpuDevice::new().expect("GPU");
    let mut rt = SurfaceRuntime::new(gpu).expect("runtime");
    let id = rt
        .add_surface(
            [64.0, 64.0],
            SurfaceContent::Game2d(Box::new(Game2dContent::simple(
                rt.gpu(),
                Color::rgb(0.0, 0.0, 0.0),
            ))),
        )
        .expect("surface");
    rt.add_view(id, true).expect("view");
    (rt, id)
}

fn draw(
    rt: &mut SurfaceRuntime,
    id: agp::core::composition::SurfaceId,
    cmds: Vec<SpriteCmd>,
    store: SpriteStore,
) {
    if let Some(SurfaceContent::Game2d(g)) = rt.content_mut(id) {
        g.sprites = cmds;
        g.sprite_store = store;
    }
    rt.graph.mark_content_dirty(id);
    rt.frame(None, &Default::default()).expect("frame");
}

fn probe(rt: &mut SurfaceRuntime, id: agp::core::composition::SurfaceId, p: [f32; 2]) -> [u8; 4] {
    rt.probe_pixel(id, p).expect("probe")
}

#[test]
fn nearest_clamp_flip_alpha_and_warm_reuse_draw_real_pixels() {
    let (mut rt, id) = surface();
    // 2x2 image: (0,0) red, (1,0) blue, (0,1) transparent, (1,1) green.
    let img = image(
        2,
        2,
        &[
            255, 0, 0, 255, 0, 0, 255, 255, // top row
            0, 0, 0, 0, 0, 255, 0, 255, // bottom row
        ],
    );
    let mut store = SpriteStore::default();
    store
        .get_or_upload(rt.gpu(), "img", &img)
        .expect("texture upload");
    assert_eq!(store.created, 1);
    assert_eq!(store.bytes_uploaded, 16);
    let uv = frame_uv([0, 0, 2, 2], 2, 2, false).expect("uv");
    draw(&mut rt, id, vec![quad(uv, "img")], store.clone());
    // The image's first row is the TOP of the quad: red | blue over
    // transparent | green. Nearest sampling makes each texel a 32x32 block.
    let red = probe(&mut rt, id, [16.0, 16.0]);
    let blue = probe(&mut rt, id, [48.0, 16.0]);
    let clear = probe(&mut rt, id, [16.0, 48.0]);
    let green = probe(&mut rt, id, [48.0, 48.0]);
    assert!(red[0] > 200 && red[1] < 30 && red[2] < 30, "red {red:?}");
    assert!(
        blue[2] > 200 && blue[0] < 30 && blue[1] < 30,
        "blue {blue:?}"
    );
    assert!(
        clear[0] < 20 && clear[1] < 20 && clear[2] < 20,
        "transparent texel shows the clear colour {clear:?}"
    );
    assert!(
        green[1] > 200 && green[0] < 30 && green[2] < 30,
        "green {green:?}"
    );

    // The flip swaps U only: the same world rect now shows blue | red.
    let flipped = frame_uv([0, 0, 2, 2], 2, 2, true).expect("uv");
    draw(&mut rt, id, vec![quad(flipped, "img")], store.clone());
    let left = probe(&mut rt, id, [16.0, 16.0]);
    let right = probe(&mut rt, id, [48.0, 16.0]);
    assert!(left[2] > 200 && left[0] < 30, "flipped left {left:?}");
    assert!(right[0] > 200 && right[2] < 30, "flipped right {right:?}");

    // A frame crop is a UV rectangle: showing only the right column paints the
    // blue/green half of the world rect and leaves the rest at the clear colour.
    let cropped = frame_uv([1, 0, 1, 2], 2, 2, false).expect("uv");
    draw(&mut rt, id, vec![quad(cropped, "img")], store.clone());
    let left = probe(&mut rt, id, [16.0, 16.0]);
    let right = probe(&mut rt, id, [48.0, 16.0]);
    assert!(left[2] > 200 && left[0] < 30, "crop left {left:?}");
    assert!(right[2] > 200 && right[0] < 30, "crop right {right:?}");

    // Clamp: UVs beyond the right edge repeat the last texel, never wrap.
    draw(
        &mut rt,
        id,
        vec![quad([0.75, 0.0, 1.25, 1.0], "img")],
        store.clone(),
    );
    let clamped = probe(&mut rt, id, [60.0, 16.0]);
    assert!(
        clamped[2] > 200 && clamped[0] < 30,
        "clamped right edge {clamped:?}"
    );

    // Warm frames never re-decode or re-upload pixels: the same content key
    // keeps one texture and one upload for any number of frame/flip changes,
    // while the instance buffer is still rewritten for the new UV.
    let before = match rt.content(id) {
        Some(SurfaceContent::Game2d(g)) => g.sprite.uploads.snapshot(),
        _ => panic!("2d content"),
    };
    let mut warm = store.clone();
    for flip in [true, false, true] {
        let uv = frame_uv([1, 0, 1, 2], 2, 2, flip).expect("uv");
        warm.get_or_upload(rt.gpu(), "img", &img).expect("warm");
        draw(&mut rt, id, vec![quad(uv, "img")], warm.clone());
    }
    assert_eq!(warm.created, 1, "no second texture for the same content");
    assert_eq!(
        warm.bytes_uploaded, 16,
        "no pixel re-upload on warm frame changes"
    );
    let after = match rt.content(id) {
        Some(SurfaceContent::Game2d(g)) => g.sprite.uploads.snapshot(),
        _ => panic!("2d content"),
    };
    assert!(
        after.buffer_rewrites > before.buffer_rewrites,
        "the instance buffer is still rewritten for the new UV"
    );
}

#[test]
fn identity_device_and_missing_texture_rules_reject_before_drawing() {
    let (mut rt, id) = surface();
    let img = image(
        2,
        2,
        &[
            255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
        ],
    );
    let second = GpuDevice::new().expect("second GPU");
    let mut store = SpriteStore::default();
    let texture = store
        .get_or_upload(rt.gpu(), "img", &img)
        .expect("texture upload");
    let weak = std::sync::Arc::downgrade(&texture);
    drop(texture);
    // A drifted payload can never reuse an existing content identity.
    let mut drifted = img.clone();
    drifted.height = 1;
    drifted.rgba.truncate(8);
    let error = store
        .get_or_upload(rt.gpu(), "img", &drifted)
        .err()
        .expect("drift rejected");
    assert!(format!("{error:?}").contains("drift"), "{error:?}");
    // A foreign device is refused for the same key.
    assert!(store.get_or_upload(&second, "img", &img).is_err());
    // A command naming a texture the store does not hold rejects the frame
    // instead of silently drawing nothing.
    if let Some(SurfaceContent::Game2d(g)) = rt.content_mut(id) {
        g.sprites = vec![quad([0.0, 0.0, 1.0, 1.0], "missing")];
        g.sprite_store = store.clone();
    }
    rt.graph.mark_content_dirty(id);
    let error = rt
        .frame(None, &Default::default())
        .err()
        .expect("missing texture");
    assert!(
        format!("{error:?}").contains("missing at draw time"),
        "{error:?}"
    );
    assert!(store.get("missing").is_none());
    // The accepted content still draws correctly on the next frame.
    draw(
        &mut rt,
        id,
        vec![quad([0.0, 0.0, 1.0, 1.0], "img")],
        store.clone(),
    );
    let recovered = probe(&mut rt, id, [16.0, 16.0]);
    assert!(
        recovered[0] > 200 && recovered[1] < 30,
        "recovered {recovered:?}"
    );
    // The surface content is an owner too; releasing both the local store and
    // the published cache leaves no pin, so the texture really goes away.
    drop(store);
    assert!(
        weak.upgrade().is_some(),
        "the surface still pins the texture"
    );
    if let Some(SurfaceContent::Game2d(g)) = rt.content_mut(id) {
        g.sprites.clear();
        g.sprite_store = SpriteStore::default();
    }
    assert!(weak.upgrade().is_none(), "texture weak owner released");
}

#[test]
fn frame_uv_and_capacity_are_bounded() {
    assert!(frame_uv([0, 0, 32, 32], 96, 32, false).is_ok());
    assert!(frame_uv([64, 0, 32, 32], 96, 32, false).is_ok());
    assert!(frame_uv([65, 0, 32, 32], 96, 32, false).is_err());
    assert!(frame_uv([0, 0, 0, 32], 96, 32, false).is_err());
    assert!(frame_uv([0, 0, 32, 32], 96, 0, false).is_err());
    // PLAN-004 review F-02: an extent larger than the image is a named
    // rejection, never a debug subtract-overflow panic or a release wrap.
    assert!(frame_uv([0, 0, 97, 32], 96, 32, false).is_err());
    assert!(frame_uv([0, 0, 32, 33], 96, 32, false).is_err());
    assert!(frame_uv([0, 0, u32::MAX, 32], 96, 32, false).is_err());
    assert!(frame_uv([95, 0, 1, 32], 96, 32, false).is_ok());
    assert!(frame_uv([96, 0, 1, 32], 96, 32, false).is_err());
    assert!(agp::renderer::sprite::MAX_SPRITES >= 16);
    let _ = OffscreenTarget::FORMAT;
}
