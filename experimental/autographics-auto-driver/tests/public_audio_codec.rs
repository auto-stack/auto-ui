//! Finite source fixtures and predecoder rejection controls, PLAN-012 V-01.
use autographics_auto_driver::authoring::public_audio::codec;
use std::{path::Path, sync::Arc};
fn sound(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/public_audio/sounds")
            .join(name),
    )
    .unwrap()
}
#[test]
fn original_three_vorbis_sources_decode_inside_shared_cpu_envelope() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../research/public-audio-codec-reference.json"
    ))
    .unwrap();
    for name in ["jump.ogg", "coin.ogg", "land.ogg"] {
        let bytes = sound(name);
        let expected = &reference["cases"][name];
        assert_eq!(
            agp::renderer::capture::sha256_hex(&bytes),
            expected["source_sha256"].as_str().unwrap()
        );
        let (pcm, header) = codec::decode(Arc::from(bytes), 64 * 1024 * 1024)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(pcm.frames.len(), header.frames);
        assert_eq!(
            pcm.frames.len() as u64,
            expected["frames"].as_u64().unwrap(),
            "independent libsndfile interval; padded block output is not the clip length"
        );
        for sample in expected["samples"].as_array().unwrap() {
            let i = sample["frame"].as_u64().unwrap() as usize;
            for channel in 0..2 {
                assert!(
                    (pcm.frames[i][channel] - sample["stereo"][channel].as_f64().unwrap() as f32)
                        .abs()
                        <= 0.00002,
                    "{name} frame{i} channel{channel}"
                );
            }
        }
        assert!(pcm.frames.iter().flatten().any(|f| f.abs() > 0.01));
        assert!(pcm.frames.iter().flatten().all(|f| f.is_finite()));
        println!(
            "{name} rate={} channels={} frames={} setup_reservation={} pcm_bytes={}",
            header.rate,
            header.channels,
            header.frames,
            header.scratch,
            pcm.frames.len() * 8
        );
    }
}
fn wav(channels: u16, rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + samples.len() as u32 * 2).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
    b.extend_from_slice(&(channels * 2).to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes())
    }
    b
}
#[test]
fn pcm16_wav_true_samples_and_container_rate_span_budget_controls() {
    for (rate, channels) in [(22050, 1), (44100, 2), (48000, 1)] {
        let data = wav(channels, rate, &[0, 16384, -32768, 32767]);
        let (pcm, h) = codec::decode(Arc::from(data.clone()), 64 * 1024 * 1024).unwrap();
        assert_eq!(h.channels, channels);
        assert_eq!(pcm.frames[1][0], if channels == 1 { 0.5 } else { -1.0 });
        assert!(codec::inspect(&data, 1)
            .unwrap_err()
            .contains("resource_limit"));
        let mut bad = data.clone();
        bad[4] = 0;
        assert!(codec::inspect(&bad, 64 * 1024 * 1024)
            .unwrap_err()
            .contains("invalid"));
        let mut bad = data;
        bad[22..24].copy_from_slice(&65535u16.to_le_bytes());
        assert!(codec::inspect(&bad, 64 * 1024 * 1024)
            .unwrap_err()
            .contains("unsupported"));
    }
    assert!(codec::inspect(&wav(1, 96000, &[1]), 64 * 1024 * 1024)
        .unwrap_err()
        .contains("unsupported"));
    let mut ogg = sound("jump.ogg");
    ogg[30] ^= 1;
    assert!(codec::inspect(&ogg, 64 * 1024 * 1024)
        .unwrap_err()
        .contains("CRC"));
}
fn repair_crc(bytes: &mut [u8]) {
    let mut offset = 0;
    while offset < bytes.len() {
        let n = bytes[offset + 26] as usize;
        let end = offset
            + 27
            + n
            + bytes[offset + 27..offset + 27 + n]
                .iter()
                .map(|b| *b as usize)
                .sum::<usize>();
        bytes[offset + 22..offset + 26].fill(0);
        let mut crc = 0u32;
        for b in &bytes[offset..end] {
            crc ^= u32::from(*b) << 24;
            for _ in 0..8 {
                crc = (crc << 1) ^ if crc & 0x80000000 != 0 { 0x04c11db7 } else { 0 };
            }
        }
        bytes[offset + 22..offset + 26].copy_from_slice(&crc.to_le_bytes());
        offset = end;
    }
}
#[test]
fn expansive_vorbis_setup_and_duration_reject_before_decoder_construction() {
    use autographics_auto_driver::authoring::assets::decode_attempts;
    let original = sound("jump.ogg");
    let setup = original
        .windows(7)
        .position(|w| w == b"\x05vorbis")
        .unwrap();
    let ident = original
        .windows(7)
        .position(|w| w == b"\x01vorbis")
        .unwrap();
    for mutation in 0..5 {
        let mut bad = original.clone();
        let expected = match mutation {
            0 => {
                bad[setup + 13..setup + 16].fill(255);
                "resource_limit"
            }
            1 => {
                bad[setup + 11..setup + 13].fill(0);
                "invalid"
            }
            2 => {
                bad[ident + 11] = 6;
                "unsupported"
            }
            3 => {
                bad[ident + 28] = 0xff;
                "invalid"
            }
            _ => {
                let last = bad.windows(4).rposition(|w| w == b"OggS").unwrap();
                bad[last + 6..last + 14].copy_from_slice(&480001u64.to_le_bytes());
                "resource_limit"
            }
        };
        repair_crc(&mut bad);
        let attempts = decode_attempts();
        let error = codec::decode(Arc::from(bad), 64 * 1024 * 1024).unwrap_err();
        assert!(error.contains(expected), "mutation{mutation}: {error}");
        assert_eq!(
            decode_attempts(),
            attempts,
            "preflight must reject before actual decode"
        );
    }
    // An intact CRC-recomputed legitimate control still decodes real samples.
    let mut control = original;
    repair_crc(&mut control);
    assert!(codec::decode(Arc::from(control), 64 * 1024 * 1024).is_ok());
}
