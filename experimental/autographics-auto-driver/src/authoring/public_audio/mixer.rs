use crate::Scalar;
use std::sync::Arc;
pub const MAX_REQUESTS: usize = 4;
pub const MAX_VOICES: usize = 8;
pub const MAX_ORDINAL: i32 = i32::MAX - 1;
pub type Stereo = [f32; 2];

#[derive(Debug)]
pub struct Pcm {
    pub rate: u32,
    pub frames: Arc<[Stereo]>,
}
#[derive(Clone, Debug)]
pub struct Cue {
    pub pcm: Arc<Pcm>,
    pub start: usize,
    pub end: usize,
    pub gain: f32,
}
impl Cue {
    pub fn validate(&self) -> Result<(), String> {
        if ![22050, 44100, 48000].contains(&self.pcm.rate) {
            return Err("audio_unsupported: sample rate".into());
        }
        if self.start >= self.end
            || self.end > self.pcm.frames.len()
            || !self.gain.is_finite()
            || !(0.0..=1.0).contains(&self.gain)
        {
            return Err("audio_invalid: cue span/gain".into());
        }
        if self.pcm.frames.len() > self.pcm.rate as usize * 10
            || self.pcm.frames.len() * 8 > 16 * 1024 * 1024
        {
            return Err("audio_resource_limit: PCM".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub ordinal: i32,
    pub cue: usize,
    pub gain: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    pub run: i32,
    pub tick: i32,
    pub bundle: i32,
    pub requests: Vec<Request>,
}
impl Packet {
    /// No float-to-int coercion, even for apparently integral values.
    pub fn from_wire(data: &[Scalar], run: i32, tick: i32) -> Result<Self, String> {
        let int = |i: usize| match data.get(i) {
            Some(Scalar::Int(v)) => Ok(*v),
            _ => Err("audio_invalid: exact integer wire".to_string()),
        };
        if data.len() < 4 {
            return Err("audio_invalid: header".into());
        }
        let count = int(3)?;
        if !(0..=4).contains(&count) || data.len() != 4 + 3 * count as usize {
            return Err("audio_invalid: bounded packet length".into());
        }
        let mut requests = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let gain = match data[6 + i * 3] {
                Scalar::Float(v) => v,
                _ => return Err("audio_invalid: gain wire".into()),
            };
            let ordinal = int(4 + i * 3)?;
            let cue = int(5 + i * 3)?;
            if cue < 0 {
                return Err("audio_invalid: cue integer".into());
            }
            requests.push(Request {
                ordinal,
                cue: cue as usize,
                gain,
            });
        }
        let packet = Self {
            run: int(0)?,
            tick: int(1)?,
            bundle: int(2)?,
            requests,
        };
        if packet.run != run || packet.tick != tick {
            return Err("audio_invalid: Run/tick identity".into());
        }
        Ok(packet)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accepted {
    Played,
    SuppressedByStep,
    Replay,
}
#[derive(Debug)]
struct Voice {
    cue: Cue,
    cursor: f64,
    gain: f32,
}
#[derive(Debug)]
pub struct Mixer {
    run: i32,
    last: Option<Packet>,
    ordinal: i32,
    voices: [Option<Voice>; MAX_VOICES],
    paused: bool,
    fault: Option<String>,
    pub decode_count: u64,
}
impl Default for Mixer {
    fn default() -> Self {
        Self {
            run: 0,
            last: None,
            ordinal: 0,
            voices: std::array::from_fn(|_| None),
            paused: false,
            fault: None,
            decode_count: 0,
        }
    }
}
impl Mixer {
    pub fn begin(&mut self, run: i32) -> Result<(), String> {
        if run <= 0 {
            return Err("audio_invalid: Run".into());
        }
        self.stop();
        self.run = run;
        self.ordinal = 0;
        self.last = None;
        self.paused = false;
        self.fault = None;
        Ok(())
    }
    pub fn run(&self) -> i32 {
        self.run
    }
    pub fn voices(&self) -> usize {
        self.voices.iter().flatten().count()
    }
    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }
    pub fn pause(&mut self, run: i32, paused: bool) -> Result<(), String> {
        if run != self.run || run <= 0 {
            return Err("audio_invalid: stale control".into());
        }
        self.paused = paused;
        Ok(())
    }
    pub fn stop(&mut self) {
        self.voices.fill_with(|| None);
        self.run = 0;
        self.last = None;
        self.ordinal = 0;
        self.paused = false;
    }
    pub fn output_fault(&mut self, message: String) {
        self.stop();
        self.fault = Some(format!("audio_fault: {message}"));
    }
    /// Caller holds the manager mutex across independent resource checks and
    /// common logical/visual accept. The closure must succeed before insertion;
    /// insertion itself is infallible and uses the already checked eight slots.
    pub fn accept<F>(
        &mut self,
        packet: Packet,
        cues: &[Cue],
        step: bool,
        accept: F,
    ) -> Result<Accepted, String>
    where
        F: FnOnce() -> Result<(), String>,
    {
        if let Some(fault) = &self.fault {
            return Err(fault.clone());
        }
        if packet.run <= 0
            || packet.run != self.run
            || packet.tick < 0
            || !(0..=1).contains(&packet.bundle)
            || packet.requests.len() > MAX_REQUESTS
            || cues.len() > 8
        {
            return Err("audio_invalid: packet identity/domain".into());
        }
        if let Some(last) = &self.last {
            if packet.tick == last.tick && packet == *last {
                return Ok(Accepted::Replay);
            }
            if packet.tick <= last.tick {
                return Err("audio_invalid: stale/replaced tick".into());
            }
        }
        if self.paused && !step {
            return Err("audio_invalid: request while paused".into());
        }
        let mut ordinal = self.ordinal;
        for request in &packet.requests {
            if request.ordinal <= ordinal || request.ordinal > MAX_ORDINAL {
                return Err("audio_invalid: ordinal".into());
            }
            if !request.gain.is_finite() || !(0.0..=1.0).contains(&request.gain) {
                return Err("audio_invalid: gain".into());
            }
            cues.get(request.cue)
                .ok_or("audio_invalid: cue identity")?
                .validate()?;
            ordinal = request.ordinal;
        }
        if !step && self.voices() + packet.requests.len() > MAX_VOICES {
            return Err("audio_resource_limit: eight voices".into());
        }
        accept()?;
        if !step {
            for request in &packet.requests {
                let slot = self
                    .voices
                    .iter_mut()
                    .find(|v| v.is_none())
                    .expect("capacity checked under same lock");
                *slot = Some(Voice {
                    cue: cues[request.cue].clone(),
                    cursor: 0.0,
                    gain: request.gain,
                });
            }
        }
        self.ordinal = ordinal;
        self.last = Some(packet);
        Ok(if step {
            Accepted::SuppressedByStep
        } else {
            Accepted::Played
        })
    }
    /// Exactly the same function feeds offline oracles and the native stream.
    /// A native callback locks for one whole quantum; control applies by next
    /// quantum. The device's already-buffered audio cannot be withdrawn.
    pub fn render(&mut self, out: &mut [Stereo], rate: u32) {
        out.fill([0.0; 2]);
        if self.paused || self.run == 0 || self.fault.is_some() || rate == 0 {
            return;
        }
        for frame in out {
            for slot in &mut self.voices {
                let Some(v) = slot else { continue };
                let frame_count = v.cue.end - v.cue.start;
                let position = v.cursor.floor() as usize;
                if position >= frame_count {
                    *slot = None;
                    continue;
                }
                let fraction = (v.cursor - position as f64) as f32;
                let a = v.cue.pcm.frames[v.cue.start + position];
                let b = v.cue.pcm.frames[v.cue.start + (position + 1).min(frame_count - 1)];
                for ch in 0..2 {
                    frame[ch] +=
                        (a[ch] * (1.0 - fraction) + b[ch] * fraction) * v.gain * v.cue.gain;
                }
                v.cursor += f64::from(v.cue.pcm.rate) / f64::from(rate);
                if v.cursor >= frame_count as f64 {
                    *slot = None;
                }
            }
            for channel in frame {
                *channel = channel.clamp(-1.0, 1.0);
            }
        }
    }
    pub fn reserved_bytes() -> usize {
        std::mem::size_of::<Self>() + 4 * std::mem::size_of::<Request>()
    }
}
