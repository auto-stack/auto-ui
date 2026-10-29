use super::mixer::{Mixer, Stereo};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
pub const TRACE_FRAMES: usize = 4096;
#[derive(Debug)]
pub struct Observation {
    pub callbacks: u64,
    pub frames: u64,
    pub nonzero: u64,
    pub peak: f32,
    pub max_us: u64,
    pub deadline_misses: u64,
    pub last_quantum: usize,
    pub last_run: i32,
    pub last_nonzero: u64,
    pub ring: Box<[Stereo; TRACE_FRAMES]>,
    pub cursor: usize,
}
impl Default for Observation {
    fn default() -> Self {
        Self {
            callbacks: 0,
            frames: 0,
            nonzero: 0,
            peak: 0.0,
            max_us: 0,
            deadline_misses: 0,
            last_quantum: 0,
            last_run: 0,
            last_nonzero: 0,
            ring: Box::new([[0.0; 2]; TRACE_FRAMES]),
            cursor: 0,
        }
    }
}
/// Host owns this stream, including across Stop. There is no global singleton
/// or background queue that can preserve an old Run's sound owner.
pub struct Native {
    _stream: cpal::Stream,
    pub mixer: Arc<Mutex<Mixer>>,
    pub observation: Arc<Mutex<Observation>>,
    pub device: String,
    pub rate: u32,
    pub channels: u16,
    pub sample_format: String,
}
impl Native {
    pub fn open() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("audio_unavailable: default output device")?;
        let supported = device
            .default_output_config()
            .map_err(|e| format!("audio_unavailable: output config: {e}"))?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        if config.channels == 0 {
            return Err("audio_unavailable: zero output channels".into());
        }
        let name = device
            .name()
            .map_err(|e| format!("audio_unavailable: name: {e}"))?;
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        let observation = Arc::new(Mutex::new(Observation::default()));
        let stream = match format {
            cpal::SampleFormat::F32 => build::<f32>(&device, &config, &mixer, &observation),
            cpal::SampleFormat::I16 => build::<i16>(&device, &config, &mixer, &observation),
            cpal::SampleFormat::U16 => build::<u16>(&device, &config, &mixer, &observation),
            _ => return Err("audio_unavailable: output sample format".into()),
        }?;
        stream
            .play()
            .map_err(|e| format!("audio_unavailable: stream play: {e}"))?;
        Ok(Self {
            _stream: stream,
            mixer,
            observation,
            device: name,
            rate: config.sample_rate.0,
            channels: config.channels,
            sample_format: format!("{format:?}"),
        })
    }
    pub fn reserved_bytes() -> usize {
        Mixer::reserved_bytes()
            + std::mem::size_of::<Self>()
            + std::mem::size_of::<Observation>()
            + TRACE_FRAMES * 8
            + 4 * std::mem::size_of::<usize>()
    }
}
fn build<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mixer: &Arc<Mutex<Mixer>>,
    observation: &Arc<Mutex<Observation>>,
) -> Result<cpal::Stream, String>
where
    f32: cpal::FromSample<T>,
{
    let output = mixer.clone();
    let failed = mixer.clone();
    let observe = observation.clone();
    let rate = config.sample_rate.0;
    let channels = config.channels as usize;
    device
        .build_output_stream(
            config,
            move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                let begin = Instant::now();
                let mut m = output.lock().unwrap();
                let mut s = observe.lock().unwrap();
                s.last_run = m.run();
                s.last_nonzero = 0;
                s.last_quantum = out.len() / channels;
                // Fixed stack scratch; no callback heap allocation, even for a backend
                // quantum longer than the retained observation ring.
                let mut scratch = [[0.0; 2]; 128];
                for chunk in out.chunks_mut(channels * 128) {
                    let len = chunk.len() / channels;
                    m.render(&mut scratch[..len], rate);
                    for (dst, src) in chunk.chunks_mut(channels).zip(&scratch[..len]) {
                        let submitted = write_frame(dst, *src);
                        s.frames += 1;
                        s.nonzero += u64::from(submitted[0] != 0.0 || submitted[1] != 0.0);
                        s.last_nonzero += u64::from(submitted[0] != 0.0 || submitted[1] != 0.0);
                        s.peak = s.peak.max(submitted[0].abs()).max(submitted[1].abs());
                        let i = s.cursor;
                        s.ring[i] = submitted;
                        s.cursor = (i + 1) % TRACE_FRAMES;
                    }
                }
                let elapsed = begin.elapsed().as_micros() as u64;
                s.callbacks += 1;
                s.max_us = s.max_us.max(elapsed);
                s.deadline_misses +=
                    u64::from(elapsed > (s.last_quantum as u64 * 1_000_000 / u64::from(rate)));
            },
            move |e| {
                failed.lock().unwrap().output_fault(e.to_string());
            },
            None,
        )
        .map_err(|e| format!("audio_unavailable: output stream: {e}"))
}
/// Observe the exact samples written into THIS stream, after channel routing
/// and device-format quantization. A mono cancellation must observe silence.
fn write_frame<T: cpal::SizedSample + cpal::FromSample<f32>>(dst: &mut [T], src: Stereo) -> Stereo
where
    f32: cpal::FromSample<T>,
{
    let channels = dst.len();
    let mut observed = [0.0; 2];
    for (ch, value) in dst.iter_mut().enumerate() {
        *value = T::from_sample(if channels == 1 {
            (src[0] + src[1]) * 0.5
        } else if ch < 2 {
            src[ch]
        } else {
            0.0
        });
        if ch < 2 {
            observed[ch] = cpal::Sample::from_sample(*value);
        }
    }
    if channels == 1 {
        observed[1] = observed[0];
    }
    observed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_native_pcm_is_after_mono_mapping_and_signed_unsigned_quantization() {
        let mut mono = [9.0f32];
        assert_eq!(write_frame(&mut mono, [0.75, -0.75]), [0.0; 2]);
        assert_eq!(mono, [0.0]);
        let mut stereo = [0i16; 2];
        let observed = write_frame(&mut stereo, [0.25, -0.5]);
        assert_eq!(stereo, [8192, -16384]);
        assert_eq!(observed, [0.25, -0.5]);
        let mut unsigned = [0u16; 2];
        let observed = write_frame(&mut unsigned, [0.25, -0.5]);
        assert_eq!(unsigned, [40960, 16384]);
        assert_eq!(observed, [0.25, -0.5]);
        let mut wider = [9.0f32; 4];
        assert_eq!(write_frame(&mut wider, [0.25, -0.5]), [0.25, -0.5]);
        assert_eq!(wider, [0.25, -0.5, 0.0, 0.0]);
    }
}
