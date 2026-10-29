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
) -> Result<cpal::Stream, String> {
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
                        for (ch, v) in dst.iter_mut().enumerate() {
                            *v = T::from_sample(if channels == 1 {
                                (src[0] + src[1]) * 0.5
                            } else if ch < 2 {
                                src[ch]
                            } else {
                                0.0
                            });
                        }
                        s.frames += 1;
                        s.nonzero += u64::from(src[0] != 0.0 || src[1] != 0.0);
                        s.last_nonzero += u64::from(src[0] != 0.0 || src[1] != 0.0);
                        s.peak = s.peak.max(src[0].abs()).max(src[1].abs());
                        let i = s.cursor;
                        s.ring[i] = *src;
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
