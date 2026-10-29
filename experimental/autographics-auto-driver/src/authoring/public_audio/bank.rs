//! Auditable PCM16 bank derivation: no resampling, dropped frames or gain edit.
use super::codec;
use serde_json::{json, Value};
use std::sync::Arc;
pub struct Bank {
    pub bytes: Vec<u8>,
    pub spans: Vec<(u32, u32)>,
    pub audit: Value,
}
pub fn derive(inputs: &[(&str, Arc<[u8]>)], available_cpu: usize) -> Result<Bank, String> {
    if inputs.is_empty() || inputs.len() > 8 {
        return Err("audio_resource_limit: bank inputs".into());
    }
    let raw = inputs
        .iter()
        .try_fold(0usize, |n, (_, b)| n.checked_add(b.len()))
        .ok_or("audio_resource_limit: bank raw overflow")?;
    if raw > 32 * 1024 * 1024 {
        return Err("audio_resource_limit: bank raw union".into());
    }
    let rate = codec::inspect(&inputs[0].1, available_cpu)?.rate;
    let capacity = 44 + rate as usize * 10 * 4;
    if raw + capacity > available_cpu {
        return Err("audio_resource_limit: bank CPU".into());
    }
    let mut bytes = Vec::with_capacity(capacity);
    bytes.resize(44, 0);
    let mut spans = Vec::with_capacity(inputs.len());
    let mut sources = vec![];
    let mut total = 0u32;
    for (name, input) in inputs {
        if name.len() > 128 {
            return Err("audio_invalid: bank source name".into());
        }
        let (pcm, header) = codec::decode(input.clone(), available_cpu - raw - capacity)?;
        if pcm.rate != rate {
            return Err("audio_unsupported: bank sources must share rate (no resampling)".into());
        }
        let end = total
            .checked_add(header.frames as u32)
            .ok_or("audio_resource_limit: bank frames overflow")?;
        if end > rate * 10 {
            return Err("audio_resource_limit: bank duration".into());
        }
        let mut error = 0.0f32;
        let mut peak = 0.0f32;
        for frame in pcm.frames.iter() {
            for sample in frame {
                if !sample.is_finite() || sample.abs() > 1.0 {
                    return Err(
                        "audio_unsupported: bank source exceeds PCM16 amplitude (no clipping)"
                            .into(),
                    );
                }
                let q = (sample * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
                error = error.max((sample - f32::from(q) / 32768.0).abs());
                peak = peak.max(sample.abs());
                bytes.extend_from_slice(&q.to_le_bytes());
            }
        }
        if error > 1.0 / 32768.0 {
            return Err("audio_invalid: bank quantization error".into());
        }
        spans.push((total, end));
        sources.push(json!({"name":name,"sha256":agp::renderer::capture::sha256_hex(input),"byte_length":input.len(),"rate":rate,"channels":header.channels,"frames":header.frames,"start_frame":total,"end_frame":end,"peak":peak,"max_quantization_error":error}));
        total = end;
    }
    let length = bytes.len();
    bytes[..4].copy_from_slice(b"RIFF");
    bytes[4..8].copy_from_slice(&(length as u32 - 8).to_le_bytes());
    bytes[8..16].copy_from_slice(b"WAVEfmt ");
    bytes[16..20].copy_from_slice(&16u32.to_le_bytes());
    bytes[20..22].copy_from_slice(&1u16.to_le_bytes());
    bytes[22..24].copy_from_slice(&2u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&rate.to_le_bytes());
    bytes[28..32].copy_from_slice(&(rate * 4).to_le_bytes());
    bytes[32..34].copy_from_slice(&4u16.to_le_bytes());
    bytes[34..36].copy_from_slice(&16u16.to_le_bytes());
    bytes[36..40].copy_from_slice(b"data");
    bytes[40..44].copy_from_slice(&(length as u32 - 44).to_le_bytes());
    let audit = json!({"format":"pcm16-wave-stereo","rate":rate,"frames":total,"byte_length":bytes.len(),"sha256":agp::renderer::capture::sha256_hex(&bytes),"mono_transform":"duplicate channel; no resampling","sources":sources});
    Ok(Bank {
        bytes,
        spans,
        audit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::{
        assets::{self, Registry},
        public_audio, textured_models,
    };
    use std::path::Path;
    #[test]
    fn fixed_three_cue_bank_fits_real_five_visual_raws_and_a_reserved_font_slot() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let inputs: Vec<_> = ["jump.ogg", "coin.ogg", "land.ogg"]
            .into_iter()
            .map(|n| {
                (
                    n,
                    Arc::<[u8]>::from(
                        std::fs::read(root.join("samples/public_audio/sounds").join(n)).unwrap(),
                    ),
                )
            })
            .collect();
        let bank = derive(&inputs, 64 * 1024 * 1024).unwrap();
        let mut registry = Registry::new();
        for name in ["character.glb", "coin.glb", "flag.glb", "platform.glb"] {
            let path = root.join("samples/textured_models/models").join(name);
            let asset = if name == "character.glb" {
                assets::import_animated(&path, &registry)
            } else {
                assets::import(&path, &registry)
            }
            .unwrap();
            assets::insert(&mut registry, asset).unwrap();
        }
        assert_eq!(textured_models::usage(&registry).0, 5);
        let bank_asset = public_audio::import_raw(
            bank.bytes.clone().into(),
            &registry,
            Some("three-cue-bank.wav".into()),
        )
        .unwrap();
        assets::insert(&mut registry, bank_asset.clone()).unwrap();
        let mut doc = crate::authoring::document::Document::empty("bank-budget".into());
        doc.schema = "autographics.scene/7".into();
        doc.assets = registry.values().map(|a| a.identity.clone()).collect();
        doc.cues = Some(
            bank.spans
                .iter()
                .enumerate()
                .map(|(i, (start, end))| public_audio::CueRef {
                    name: format!("cue{i}"),
                    asset: bank_asset.identity.key(),
                    start_frame: *start,
                    end_frame: *end,
                    gain: 1.0,
                })
                .collect(),
        );
        doc.validate().unwrap();
        let cues = public_audio::resolve_cues(&doc, &registry).unwrap();
        assert_eq!(cues.len(), 3);
        for cue in &cues {
            assert!(
                Arc::ptr_eq(&cue.pcm, bank_asset.audio.as_ref().unwrap()),
                "three public cues share one immutable PCM owner"
            );
        }
        let pcm = bank_asset.audio.as_ref().unwrap();
        for (i, (start, end)) in bank.spans.iter().enumerate() {
            let (source, _) = codec::decode(inputs[i].1.clone(), 64 * 1024 * 1024).unwrap();
            assert_eq!((*end - *start) as usize, source.frames.len());
            for (a, b) in source
                .frames
                .iter()
                .zip(&pcm.frames[*start as usize..*end as usize])
            {
                for ch in 0..2 {
                    assert!((a[ch] - b[ch]).abs() <= 1.0 / 32768.0);
                }
            }
        }
        let (raw_count, raw, cpu, gpu) = textured_models::usage(&registry);
        assert_eq!(raw_count, 6);
        // 013 owns the future font format/renderer. Exercise the SAME actual
        // shared budget with its frozen raw length and conservative decoded/
        // atlas reservation, without pretending font import is supported012.
        textured_models::candidate_budget(
            &registry,
            "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa",
            26828,
            4 * 1024 * 1024,
            4 * 1024 * 1024,
            false,
        )
        .unwrap();
        assert!(
            raw + 26828 < 32 * 1024 * 1024
                && cpu + 4 * 1024 * 1024 < 64 * 1024 * 1024
                && gpu + 4 * 1024 * 1024 < 64 * 1024 * 1024
        );
        println!("bank={} actual_raws={raw_count} raw={raw} cpu={cpu} gpu={gpu}; font reservation is seventh identity, not font service delivery",bank.audit);
    }
}
