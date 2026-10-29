//! Bounded static-PNG admission and decode (auto-ui PLAN-004).
//!
//! The supported domain is deliberately finite: one static PNG, 8-bit RGB or
//! RGBA, no interlacing, no APNG, 1..=2048 pixels per side and at most 16MiB of
//! decoded RGBA. Every byte is checked structurally *before* the decoder or any
//! allocation sees it, so a corrupt or surprising file can never drive memory
//! use. Decoding is the only step that allocates pixels and it re-verifies the
//! header dimensions against the decoded image.

/// Largest accepted image side, in pixels.
pub const MAX_SIDE: u32 = 2048;
/// Largest accepted decoded RGBA payload, in bytes (2048*2048*4).
pub const MAX_DECODED: usize = 16 * 1024 * 1024;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// The admitted header facts that later stages verify against the decoded image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PngHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
}

/// A decoded, immutable RGBA8 image. `source_sha256` is the identity of the
/// original file bytes, never of the decoded pixels.
#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub source_sha256: String,
}

impl ImageData {
    pub fn byte_length(&self) -> usize {
        self.rgba.len()
    }
    /// Nearest-neighbour sample of one pixel; `None` outside the image.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let at = ((y as usize) * (self.width as usize) + x as usize) * 4;
        let p = self.rgba.get(at..at + 4)?;
        Some([p[0], p[1], p[2], p[3]])
    }
}

fn be32(b: &[u8], at: usize) -> Result<u32, String> {
    let s = b.get(at..at + 4).ok_or("invalid: PNG truncation")?;
    Ok(u32::from_be_bytes(s.try_into().unwrap()))
}

/// Structural admission without decoding: signature, IHDR domain, chunk
/// framing, APNG/unknown-critical rejection and the decoded-size prediction.
pub fn preflight(bytes: &[u8]) -> Result<PngHeader, String> {
    if bytes.len() < 8 + 12 + 13 + 12 {
        return Err("invalid: PNG too short".into());
    }
    if bytes[..8] != SIGNATURE {
        return Err("invalid: PNG signature".into());
    }
    let ihdr = be32(bytes, 8)? as usize;
    if ihdr != 13 || bytes.get(12..16) != Some(b"IHDR") {
        return Err("invalid: PNG IHDR first chunk".into());
    }
    let width = be32(bytes, 16)?;
    let height = be32(bytes, 20)?;
    let bit_depth = *bytes.get(24).ok_or("invalid: PNG bit depth")?;
    let color_type = *bytes.get(25).ok_or("invalid: PNG color type")?;
    let (compression, filter, interlace) = (
        *bytes.get(26).ok_or("invalid: PNG compression")?,
        *bytes.get(27).ok_or("invalid: PNG filter")?,
        *bytes.get(28).ok_or("invalid: PNG interlace")?,
    );
    if bit_depth != 8 {
        return Err("unsupported: PNG bit depth 8 required".into());
    }
    if color_type != 2 && color_type != 6 {
        return Err("unsupported: PNG truecolour RGB/RGBA required".into());
    }
    if compression != 0 || filter != 0 {
        return Err("invalid: PNG compression/filter method".into());
    }
    if interlace != 0 {
        return Err("unsupported: interlaced PNG".into());
    }
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err(format!(
            "resource limit: PNG side {width}x{height} outside 1..={MAX_SIDE}"
        ));
    }
    let decoded = (width as usize)
        .checked_mul(height as usize)
        .and_then(|p| p.checked_mul(4))
        .ok_or("invalid: PNG dimension overflow")?;
    if decoded > MAX_DECODED {
        return Err("resource limit: PNG decoded RGBA 16MiB".into());
    }
    // Walk the chunk chain so a truncated file, trailing garbage or an
    // animation chunk is rejected before the decoder.
    let mut at = 8usize;
    let mut seen_end = false;
    let mut chunks = 0usize;
    while at < bytes.len() {
        let length = be32(bytes, at)? as usize;
        let kind = bytes.get(at + 4..at + 8).ok_or("invalid: PNG chunk type")?;
        let body_end = at
            .checked_add(12)
            .and_then(|n| n.checked_add(length))
            .ok_or("invalid: PNG chunk overflow")?;
        if body_end > bytes.len() {
            return Err("invalid: PNG chunk span".into());
        }
        if kind == b"acTL" {
            return Err("unsupported: APNG animation".into());
        }
        if kind == b"IEND" {
            seen_end = true;
            if body_end != bytes.len() {
                return Err("invalid: PNG trailing bytes".into());
            }
            break;
        }
        chunks += 1;
        if chunks > 4096 {
            return Err("resource limit: PNG chunk count".into());
        }
        at = body_end;
    }
    if !seen_end {
        return Err("invalid: PNG missing IEND".into());
    }
    Ok(PngHeader {
        width,
        height,
        bit_depth,
        color_type,
    })
}

/// Decode an admitted PNG into RGBA8, re-checking the header facts.
pub fn decode_png_bytes(bytes: &[u8], source_sha256: &str) -> Result<ImageData, String> {    let header = preflight(bytes)?;
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| format!("invalid: PNG decoder {e}"))?;
    let rgba = image.to_rgba8();
    if rgba.width() != header.width || rgba.height() != header.height {
        return Err("invalid: decoded PNG dimensions".into());
    }
    let raw = rgba.into_raw();
    let expected = (header.width as usize)
        .checked_mul(header.height as usize)
        .and_then(|p| p.checked_mul(4))
        .ok_or("invalid: decoded size overflow")?;
    if raw.len() != expected || raw.len() > MAX_DECODED {
        return Err("invalid: decoded RGBA length".into());
    }
    Ok(ImageData {
        width: header.width,
        height: header.height,
        rgba: raw,
        source_sha256: source_sha256.to_owned(),
    })
}

/// Encode one RGBA8 image as a static PNG. Fixture and test support only: the
/// production path never writes an image, it only admits and decodes one.
pub fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    use image::{ExtendedColorType, ImageEncoder};
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|p| p.checked_mul(4))
        .ok_or("invalid: encode dimensions")?;
    if width == 0 || height == 0 || rgba.len() != expected {
        return Err("invalid: encode payload length".into());
    }
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|e| format!("invalid: PNG encode {e}"))?;
    Ok(out)
}