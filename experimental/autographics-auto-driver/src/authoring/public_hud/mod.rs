use serde::{Deserialize, Serialize};

pub const SLOTS: usize = 4;
pub const MAX_TEXT: usize = 32;
pub const ROW_LEN: usize = MAX_TEXT + 2;
pub const BLOCK_LEN: usize = SLOTS * ROW_LEN;
pub const FONT_SHA: &str = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa";
pub const FONT_LEN: usize = 26828;
pub const LICENSE_SHA: &str = "31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b";
pub const LICENSE_LEN: usize = 4391;
pub const LICENSE_TEXT: &str = include_str!("../../../samples/public_hud/fonts/license.txt");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentHud {
    pub font: String,
    pub license: String,
    pub slots: Vec<Slot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    pub slot: i32,
    pub visible: bool,
    pub text: String,
    pub anchor: i32,
    pub offset: [f32; 2],
    pub size: f32,
    pub rgba: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub slots: [SlotState; SLOTS],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlotState {
    pub visible: bool,
    pub text: String,
}

pub fn validate_text(text: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT || !text.bytes().all(|b| (32..=126).contains(&b)) {
        return Err("hud_invalid: visible ASCII text length/code".into());
    }
    Ok(())
}

pub fn validate_document(document: &super::document::Document) -> Result<(), String> {
    let versioned = document.schema == "autographics.scene/8";
    if versioned != document.hud.is_some() {
        return Err("hud_invalid: versioned HUD field".into());
    }
    if !versioned && document.assets.iter().any(super::assets::AssetRef::is_font) {
        return Err("hud_invalid: legacy font format".into());
    }
    let Some(hud) = &document.hud else {
        return Ok(());
    };
    let font = document
        .assets
        .iter()
        .find(|asset| asset.key() == hud.font)
        .ok_or("hud_invalid: font reference")?;
    if !font.is_font() {
        return Err("hud_invalid: font resource kind".into());
    }
    if hud.license.len() != LICENSE_LEN
        || agp::renderer::capture::sha256_hex(hud.license.as_bytes()) != LICENSE_SHA
    {
        return Err("hud_invalid: font license identity".into());
    }
    if hud.slots.len() != SLOTS {
        return Err("hud_invalid: complete four slots required".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut total = 0;
    for slot in &hud.slots {
        if !(0..SLOTS as i32).contains(&slot.slot) || !seen.insert(slot.slot) {
            return Err("hud_invalid: missing/duplicate slot".into());
        }
        validate_text(&slot.text)?;
        total += slot.text.len();
        if !(0..=3).contains(&slot.anchor)
            || !slot.size.is_finite()
            || !(16.0..=48.0).contains(&slot.size)
            || slot.offset.iter().any(|v| !v.is_finite())
            || slot
                .rgba
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("hud_invalid: slot layout".into());
        }
    }
    if total > SLOTS * MAX_TEXT {
        return Err("hud_resource_limit: total text".into());
    }
    Ok(())
}

pub fn default_document(font: String) -> DocumentHud {
    let slot = |slot, visible, text: &str, anchor| Slot {
        slot,
        visible,
        text: text.into(),
        anchor,
        offset: [8.0, 8.0],
        size: 32.0,
        rgba: [1.0; 4],
    };
    DocumentHud {
        font,
        license: LICENSE_TEXT.into(),
        slots: vec![
            slot(0, true, "0", 0),
            slot(1, true, "Playing", 1),
            slot(2, false, "", 2),
            slot(3, true, "Won", 3),
        ],
    }
}

pub fn initial_frame(document: &super::document::Document) -> Result<Option<Frame>, String> {
    let Some(hud) = &document.hud else {
        return Ok(None);
    };
    let mut slots: [SlotState; SLOTS] = std::array::from_fn(|_| SlotState {
        visible: false,
        text: String::new(),
    });
    for slot in &hud.slots {
        slots[slot.slot as usize] = SlotState {
            visible: slot.visible,
            text: slot.text.clone(),
        };
    }
    Ok(Some(Frame { slots }))
}

pub fn admit_block(data: &[crate::Scalar]) -> Result<Frame, String> {
    if data.len() != BLOCK_LEN {
        return Err("hud_invalid: complete wire length".into());
    }
    let mut slots: [SlotState; SLOTS] = std::array::from_fn(|_| SlotState {
        visible: false,
        text: String::new(),
    });
    let mut total = 0;
    for (slot, state) in slots.iter_mut().enumerate() {
        let row = &data[slot * ROW_LEN..(slot + 1) * ROW_LEN];
        let visible = match row[0] {
            crate::Scalar::Int(0) => false,
            crate::Scalar::Int(1) => true,
            _ => return Err("hud_invalid: visible wire".into()),
        };
        let len = match row[1] {
            crate::Scalar::Int(v) if (0..=MAX_TEXT as i32).contains(&v) => v as usize,
            _ => return Err("hud_invalid: text length wire".into()),
        };
        let mut bytes = Vec::with_capacity(len);
        for (index, value) in row[2..].iter().enumerate() {
            match (index < len, value) {
                (true, crate::Scalar::Int(v)) if (32..=126).contains(v) => {
                    bytes.push(*v as u8);
                }
                (false, crate::Scalar::Int(0)) => {}
                _ => return Err("hud_invalid: ASCII/canonical code wire".into()),
            }
        }
        total += len;
        *state = SlotState {
            visible,
            text: String::from_utf8(bytes).map_err(|_| "hud_invalid: ASCII text")?,
        };
    }
    if total > SLOTS * MAX_TEXT {
        return Err("hud_resource_limit: total text wire".into());
    }
    Ok(Frame { slots })
}

pub const ATLAS_UNION_RESERVE: usize = 4 * (1024 * 1024 + 128 * 1024);

pub fn import(
    path: &std::path::Path,
    existing: &super::assets::Registry,
) -> Result<std::sync::Arc<super::assets::Asset>, String> {
    let bytes = super::textured_models::RawPool::new(existing).read(path)?;
    import_raw(
        bytes,
        existing,
        path.file_name()
            .map(|name| name.to_string_lossy().chars().take(128).collect()),
    )
}

pub fn import_raw(
    bytes: std::sync::Arc<[u8]>,
    existing: &super::assets::Registry,
    name: Option<String>,
) -> Result<std::sync::Arc<super::assets::Asset>, String> {
    use std::sync::Arc;
    let start = std::time::Instant::now();
    if bytes.len() != FONT_LEN || agp::renderer::capture::sha256_hex(&bytes) != FONT_SHA {
        return Err("hud_invalid: fixed font bytes".into());
    }
    if let Some(old) = existing.get(&format!("sha256:{FONT_SHA}")) {
        if !old.identity.is_font() {
            return Err("hud_invalid: font identity conflict".into());
        }
        return Ok(old.clone());
    }
    let retained = ATLAS_UNION_RESERVE + std::mem::size_of::<super::assets::Asset>();
    super::textured_models::candidate_budget(
        existing,
        FONT_SHA,
        bytes.len(),
        retained,
        retained,
        false,
    )?;
    let identity = super::assets::AssetRef {
        closure: None,
        audio: None,
        format: "ttf".into(),
        sha256: FONT_SHA.into(),
        byte_length: bytes.len(),
        name,
        width: None,
        height: None,
    };
    identity.validate()?;
    let asset = Arc::new(super::assets::Asset {
        audio: None,
        model_images: vec![],
        dependencies: std::collections::BTreeMap::new(),
        identity,
        bytes,
        model: agp::assets::ModelData {
            source_sha256: String::new(),
            meshes: vec![],
        },
        animation: None,
        image: None,
        cpu_bytes: retained,
        gpu_bytes: retained,
        decode_us: start.elapsed().as_micros(),
    });
    let mut union = existing.clone();
    super::assets::insert(&mut union, asset.clone())?;
    Ok(asset)
}

#[derive(Clone, Default)]
pub struct AtlasCache {
    entries:
        std::collections::BTreeMap<(String, u32, u64), std::sync::Arc<agp::render2d::TextRenderer>>,
}

impl AtlasCache {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn weak_owners(&self) -> Vec<std::sync::Weak<agp::render2d::TextRenderer>> {
        self.entries
            .values()
            .map(std::sync::Arc::downgrade)
            .collect()
    }
}

static ATLAS_BUILDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn atlas_builds() -> u64 {
    ATLAS_BUILDS.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn prepare_overlay(
    gpu: &agp::renderer::GpuDevice,
    accepted: &AtlasCache,
    assets: &super::assets::Registry,
    document: &super::document::Document,
    frame: Option<&Frame>,
    camera: Option<&super::runtime_camera::RuntimeCameraFrame>,
    three: bool,
) -> Result<
    (
        Option<agp::host::surface_runtime::PublicHudOverlay>,
        AtlasCache,
    ),
    String,
> {
    prepare_overlay_with_clip(gpu, accepted, assets, document, frame, camera, three, None)
}

pub fn prepare_overlay_with_clip(
    gpu: &agp::renderer::GpuDevice,
    accepted: &AtlasCache,
    assets: &super::assets::Registry,
    document: &super::document::Document,
    frame: Option<&Frame>,
    camera: Option<&super::runtime_camera::RuntimeCameraFrame>,
    three: bool,
    clip_override: Option<[f32; 4]>,
) -> Result<
    (
        Option<agp::host::surface_runtime::PublicHudOverlay>,
        AtlasCache,
    ),
    String,
> {
    use agp::host::surface_runtime::{PublicHudLayer, PublicHudOverlay};
    use std::sync::Arc;
    let Some(hud) = &document.hud else {
        return Ok((None, accepted.clone()));
    };
    let owned_initial;
    let state = match frame {
        Some(frame) => frame,
        None => {
            owned_initial = initial_frame(document)?.ok_or("hud_invalid: initial state")?;
            &owned_initial
        }
    };
    let logical_size = if three {
        super::runtime_camera::SURFACE_3D.to_array()
    } else {
        super::runtime_camera::SURFACE_2D.to_array()
    };
    let clip = match camera {
        Some(camera) if !three && camera.dim == super::runtime_camera::CameraDim::TwoD => {
            let (_, content, bars) = camera.letterbox_2d().ok_or("hud_invalid: letterbox")?;
            [bars.x, bars.y, content.x, content.y]
        }
        _ => [0.0, 0.0, logical_size[0], logical_size[1]],
    };
    let clip = clip_override.unwrap_or(clip);
    let font = assets.get(&hud.font).ok_or("hud_invalid: accepted font")?;
    if !font.identity.is_font() || font.bytes.len() != FONT_LEN {
        return Err("hud_invalid: accepted font identity".into());
    }
    let mut staged = accepted.clone();
    let device = gpu.token().ordinal;
    let mut layers: std::collections::BTreeMap<
        u32,
        (
            Arc<agp::render2d::TextRenderer>,
            Vec<agp::render2d::GlyphQuad>,
        ),
    > = Default::default();
    for slot in &hud.slots {
        let state = &state.slots[slot.slot as usize];
        if !state.visible || state.text.is_empty() {
            continue;
        }
        validate_text(&state.text)?;
        let size = slot.size.round() as u32;
        let key = (font.identity.sha256.clone(), size, device);
        if !staged.entries.contains_key(&key) {
            if staged.entries.len() >= 4 {
                return Err("hud_resource_limit: atlas key union".into());
            }
            let chars: String = (32u8..=126).map(char::from).collect();
            let renderer = agp::render2d::TextRenderer::new_checked(
                gpu,
                agp::renderer::target::OffscreenTarget::FORMAT,
                &font.bytes,
                size as f32,
                &chars,
            )
            .map_err(|error| format!("hud_atlas: {error:?}"))?;
            staged.entries.insert(key.clone(), Arc::new(renderer));
            ATLAS_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let renderer = staged.entries[&key].clone();
        let width = renderer.try_measure(&state.text)?;
        let x = match slot.anchor {
            0 | 2 => clip[0] + slot.offset[0],
            1 | 3 => clip[0] + clip[2] - slot.offset[0] - width,
            _ => return Err("hud_invalid: anchor".into()),
        };
        let y = match slot.anchor {
            0 | 1 => clip[1] + slot.offset[1],
            2 | 3 => clip[1] + clip[3] - slot.offset[1] - slot.size,
            _ => return Err("hud_invalid: anchor".into()),
        };
        if !x.is_finite() || !y.is_finite() {
            return Err("hud_invalid: finite layout".into());
        }
        let color =
            agp::core::scene::Color::rgba(slot.rgba[0], slot.rgba[1], slot.rgba[2], slot.rgba[3]);
        let glyphs = renderer.try_layout(&state.text, x, y, color)?;
        let layer = layers.entry(size).or_insert_with(|| (renderer, vec![]));
        layer.1.extend(glyphs);
        if layer.1.len() > SLOTS * MAX_TEXT {
            return Err("hud_resource_limit: glyph instances".into());
        }
    }
    Ok((
        Some(PublicHudOverlay {
            layers: layers
                .into_iter()
                .map(|(_, (renderer, glyphs))| PublicHudLayer { renderer, glyphs })
                .collect(),
            logical_size,
            clip,
        }),
        staged,
    ))
}
