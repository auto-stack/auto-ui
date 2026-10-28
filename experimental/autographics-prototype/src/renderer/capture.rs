//! Offscreen readback → PNG keyframe capture (probe evidence).

use crate::renderer::device::GpuDevice;
use crate::renderer::error::RendererError;
use crate::renderer::target::OffscreenTarget;

/// Synchronously copy an offscreen target into CPU memory (RGBA8, row-padded
/// to 256 bytes as wgpu requires), then encode a PNG.
pub fn capture_png(
    gpu: &GpuDevice,
    target: &OffscreenTarget,
) -> Result<(Vec<u8>, image::RgbaImage), RendererError> {
    capture_png_with(gpu.device(), gpu.queue(), target)
}

/// Raw-device variant (Iced seam).
pub fn capture_png_with(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &OffscreenTarget,
) -> Result<(Vec<u8>, image::RgbaImage), RendererError> {
    let width = target.width;
    let height = target.height;
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = (unpadded_bytes_per_row + 255) & !255; // wgpu COPY_BYTES_PER_ROW_ALIGNMENT

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("agp readback"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("agp readback encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: target.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());

    let png = {
        let mapped = buffer.slice(..).get_mapped_range();
        let mut raw: Vec<u8> = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
        for y in 0..height {
            raw.extend_from_slice(
                &mapped[(y * padded_bytes_per_row) as usize
                    ..(y * padded_bytes_per_row + unpadded_bytes_per_row) as usize],
            );
        }
        drop(mapped);
        buffer.unmap();
        image::RgbaImage::from_raw(width, height, raw)
            .ok_or_else(|| RendererError::ReportIo("rgba buffer size".into()))?
    };

    let mut out: Vec<u8> = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut out);
        png.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| RendererError::ReportIo(format!("png encode: {e}")))?;
    }
    Ok((out, png))
}

/// sha256 hex of a byte slice (keyframe hashing for reports).
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// Original frozen-sample sRGB8 image oracle: absolute channel error.
pub fn compare_png(a: &[u8], b: &[u8]) -> Result<(f64, u8), RendererError> {
    let a = image::load_from_memory(a)
        .map_err(|e| RendererError::ReportIo(e.to_string()))?
        .to_rgba8();
    let b = image::load_from_memory(b)
        .map_err(|e| RendererError::ReportIo(e.to_string()))?
        .to_rgba8();
    if a.dimensions() != b.dimensions() {
        return Err(RendererError::ReportIo("image extent mismatch".into()));
    }
    let mut bins = [0u64; 256];
    let mut total = 0u64;
    for (a, b) in a.as_raw().iter().zip(b.as_raw()) {
        let d = a.abs_diff(*b);
        bins[d as usize] += 1;
        total += u64::from(d);
    }
    let n = a.as_raw().len() as u64;
    let mut cumulative = 0;
    let mut p99 = 0;
    for (i, count) in bins.iter().enumerate() {
        cumulative += count;
        if cumulative * 100 >= n * 99 {
            p99 = i as u8;
            break;
        }
    }
    Ok((total as f64 / n as f64, p99))
}

pub fn compare_png_region(a: &[u8], b: &[u8], rect: [u32; 4]) -> Result<(f64, u8), RendererError> {
    let decode = |bytes: &[u8]| {
        image::load_from_memory(bytes)
            .map(|image| image.to_rgba8())
            .map_err(|e| RendererError::ReportIo(e.to_string()))
    };
    let a = decode(a)?;
    let b = decode(b)?;
    let [x, y, w, h] = rect;
    if a.dimensions() != b.dimensions()
        || w == 0
        || h == 0
        || x + w > a.width()
        || y + h > a.height()
    {
        return Err(RendererError::ReportIo("image crop range".into()));
    }
    let mut bins = [0u64; 256];
    let mut total = 0u64;
    for yy in y..y + h {
        for xx in x..x + w {
            for (a, b) in a.get_pixel(xx, yy).0.iter().zip(b.get_pixel(xx, yy).0) {
                let d = a.abs_diff(b);
                bins[d as usize] += 1;
                total += u64::from(d);
            }
        }
    }
    let n = u64::from(w) * u64::from(h) * 4;
    let mut cumulative = 0;
    let mut p99 = 0;
    for (i, count) in bins.iter().enumerate() {
        cumulative += count;
        if cumulative * 100 >= n * 99 {
            p99 = i as u8;
            break;
        }
    }
    Ok((total as f64 / n as f64, p99))
}
pub fn shadow_probe_position(png: &[u8]) -> Result<[u32; 2], RendererError> {
    let image = image::load_from_memory(png)
        .map_err(|e| RendererError::ReportIo(e.to_string()))?
        .to_rgba8();
    for y in 2..image.height() - 2 {
        for x in 2..image.width() - 2 {
            let mut shadow = true;
            for yy in y - 2..=y + 2 {
                for xx in x - 2..=x + 2 {
                    let p = image.get_pixel(xx, yy).0;
                    if !(128..=225).contains(&p[0]) || p[0] != p[1] || p[1] != p[2] {
                        shadow = false;
                        break;
                    }
                }
                if !shadow {
                    break;
                }
            }
            if shadow {
                return Ok([x, y]);
            }
        }
    }
    Err(RendererError::ReportIo(
        "original shadow interior missing".into(),
    ))
}
