//! Finite one-shot service. Only logical accept can insert a voice; rendering
//! and capture have no audio side effects. Device and offline paths share PCM.
pub mod bank;
pub mod codec;
pub mod mixer;
pub mod native;

use super::{
    assets::{self, Asset, AssetRef, Registry},
    document::Document,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, sync::Arc};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub rate: u32,
    pub channels: u16,
    pub frames: u32,
}
impl Metadata {
    pub fn validate(&self) -> Result<(), String> {
        if ![22050, 44100, 48000].contains(&self.rate) || !(1..=2).contains(&self.channels) {
            return Err("audio_unsupported: sample metadata domain".into());
        }
        if self.frames == 0 || self.frames > self.rate * 10 {
            return Err("audio_resource_limit: sample metadata frames".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueRef {
    pub name: String,
    pub asset: String,
    pub start_frame: u32,
    pub end_frame: u32,
    pub gain: f32,
}
pub fn validate_cues(doc: &Document) -> Result<(), String> {
    let cues = doc.cues.as_deref().unwrap_or(&[]);
    if cues.len() > 8 {
        return Err("audio_resource_limit: eight cues".into());
    }
    let mut names = std::collections::BTreeSet::new();
    for cue in cues {
        let metadata = doc
            .assets
            .iter()
            .find(|a| a.key() == cue.asset && a.is_audio())
            .and_then(|a| a.audio.as_ref())
            .ok_or("audio_invalid: cue resource identity")?;
        metadata.validate()?;
        if cue.name.is_empty()
            || cue.name.len() > 128
            || !names.insert(&cue.name)
            || cue.start_frame >= cue.end_frame
            || cue.end_frame > metadata.frames
            || !cue.gain.is_finite()
            || !(0.0..=1.0).contains(&cue.gain)
        {
            return Err("audio_invalid: cue name/span/gain".into());
        }
    }
    Ok(())
}
pub fn resolve_cues(doc: &Document, registry: &Registry) -> Result<Vec<mixer::Cue>, String> {
    validate_cues(doc)?;
    doc.cues
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|cue| {
            let declaration = doc
                .assets
                .iter()
                .find(|a| a.key() == cue.asset)
                .ok_or("audio_invalid: declaration")?;
            let asset = registry
                .get(&cue.asset)
                .ok_or("audio_invalid: missing PCM payload")?;
            let pcm = asset
                .audio
                .as_ref()
                .ok_or("audio_invalid: wrong resource kind")?;
            let metadata = declaration
                .audio
                .as_ref()
                .ok_or("audio_invalid: missing metadata")?;
            if !asset.identity.same_content(declaration)
                || metadata.rate != pcm.rate
                || metadata.frames as usize != pcm.frames.len()
            {
                return Err("audio_invalid: PCM content identity".into());
            }
            let value = mixer::Cue {
                pcm: pcm.clone(),
                start: cue.start_frame as usize,
                end: cue.end_frame as usize,
                gain: cue.gain,
            };
            value.validate()?;
            Ok(value)
        })
        .collect()
}
pub fn import(path: &Path, existing: &Registry) -> Result<Arc<Asset>, String> {
    let bytes = super::textured_models::RawPool::new(existing).read(path)?;
    import_raw(
        bytes,
        existing,
        path.file_name()
            .map(|n| n.to_string_lossy().chars().take(128).collect()),
    )
}
pub fn import_raw(
    bytes: Arc<[u8]>,
    existing: &Registry,
    name: Option<String>,
) -> Result<Arc<Asset>, String> {
    let start = std::time::Instant::now();
    let format = if bytes.starts_with(b"OggS") {
        "ogg"
    } else if bytes.starts_with(b"RIFF") {
        "wav"
    } else {
        return Err("audio_unsupported: Ogg/Vorbis or PCM16 WAV required".into());
    };
    let sha = agp::renderer::capture::sha256_hex(&bytes);
    if let Some(old) = existing.get(&format!("sha256:{sha}")) {
        if old.bytes != bytes || old.identity.format != format || old.audio.is_none() {
            return Err("audio_invalid: cached identity/kind".into());
        }
        return Ok(old.clone());
    }
    let (_, _, resident_cpu, _) = super::textured_models::usage(existing);
    let resident_cpu = resident_cpu - super::textured_models::uncovered_audio_service_cpu(existing);
    // Manager, fixed observation ring, four requests, eight voices and cue
    // metadata join the same CPU envelope, including when imported before Run.
    let service = native::Native::reserved_bytes() + 8 * std::mem::size_of::<CueRef>() + 4096;
    let available = assets::MAX_GEOMETRY.saturating_sub(resident_cpu + service);
    let header = codec::inspect(&bytes, available)?;
    let retained = bytes.len()
        + header.max_frames * 8
        + std::mem::size_of::<Asset>()
        + std::mem::size_of::<mixer::Pcm>()
        + service;
    super::textured_models::candidate_audio_budget(existing, &sha, bytes.len(), retained)?;
    let (pcm, header) = codec::decode(bytes.clone(), available)?;
    let identity = AssetRef {
        closure: None,
        audio: Some(Metadata {
            rate: header.rate,
            channels: header.channels,
            frames: header.frames as u32,
        }),
        format: format.into(),
        sha256: sha,
        byte_length: bytes.len(),
        name,
        width: None,
        height: None,
    };
    identity.validate()?;
    let cpu_bytes = bytes.len()
        + pcm.frames.len() * 8
        + std::mem::size_of::<Asset>()
        + std::mem::size_of::<mixer::Pcm>()
        + service;
    let asset = Arc::new(Asset {
        audio: Some(pcm),
        model_images: vec![],
        dependencies: BTreeMap::new(),
        identity,
        bytes,
        model: agp::assets::ModelData {
            source_sha256: String::new(),
            meshes: vec![],
        },
        animation: None,
        image: None,
        cpu_bytes,
        gpu_bytes: 0,
        decode_us: start.elapsed().as_micros(),
    });
    let mut union = existing.clone();
    assets::insert(&mut union, asset.clone())?;
    Ok(asset)
}
