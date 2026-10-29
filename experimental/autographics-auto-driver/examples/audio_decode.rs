//! Bounded PCM export for independent codec verification; never records input audio.
use autographics_auto_driver::authoring::{assets, public_audio::codec};
use std::{io::Write, path::Path, sync::Arc};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: audio_decode <local Ogg/WAV> <new stereo-f32le output>".into());
    }
    let input = assets::read_blob(Path::new(&args[1]))?;
    let (pcm, header) = codec::decode(Arc::from(input), 64 * 1024 * 1024)?;
    assets::regular_path(Path::new(&args[2]), true)?;
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])
        .map_err(|e| e.to_string())?;
    for frame in pcm.frames.iter() {
        for channel in frame {
            out.write_all(&channel.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
    }
    out.sync_all().map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"frames":header.frames,"channels":header.channels,"rate":header.rate,"scratch":header.scratch})
    );
    Ok(())
}
