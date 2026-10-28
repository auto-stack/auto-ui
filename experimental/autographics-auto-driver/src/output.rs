use super::{Scalar::Int, Session};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub slot: u32,
    pub generation: u32,
    pub shape: u8,
    pub values: [f32; 10],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    pub background: [f32; 4],
    pub session: u64,
    pub sequence: u64,
    pub version: i32,
    pub objects: Vec<Object>,
    pub score: i32,
    pub sounds: i32,
    pub events: Vec<[i32; 3]>,
}
impl Packet {
    pub fn read(s: &mut Session, session: u64, sequence: u64) -> Result<Self, String> {
        let version = s.int("protocol", &[])?;
        let count = s.int("object_count", &[])?;
        if !(0..=128).contains(&count) {
            return Err("object limit".into());
        }
        let mut objects = Vec::new();
        for i in 0..count {
            match s.int("object_alive", &[Int(i)])? {
                0 => continue,
                1 => {}
                _ => return Err("invalid alive flag".into()),
            }
            let generation = s.int("generation", &[Int(i)])?;
            let shape = s.int("shape", &[Int(i)])?;
            if generation <= 0 || !(0..=2).contains(&shape) {
                return Err("invalid object identity/shape".into());
            }
            let mut values = [0.0; 10];
            for f in 0..10 {
                values[f] = s.float("field", &[Int(i), Int(f as i32)])?
            }
            objects.push(Object {
                slot: i as u32,
                generation: generation as u32,
                shape: shape as u8,
                values,
            });
        }
        let n = s.int("event_count", &[])?;
        if !(0..=256).contains(&n) {
            return Err("event limit".into());
        }
        let mut events = vec![];
        for i in 0..n {
            events.push([
                s.int("event_kind", &[Int(i)])?,
                s.int("event_a", &[Int(i)])?,
                s.int("event_b", &[Int(i)])?,
            ]);
        }
        let mut background = [0.0; 4];
        for i in 0..4 {
            background[i] = s.float("background", &[Int(i as i32)])?;
        }
        let p = Self {
            background,
            events,
            session,
            sequence,
            version,
            objects,
            score: s.int("integer", &[Int(0)])?,
            sounds: s.int("integer", &[Int(1)])?,
        };
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self
            .background
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("background range".into());
        }
        if self.version != 1 {
            return Err("protocol version".into());
        }
        if self.objects.len() > 128 || self.score < 0 || self.sounds < 0 {
            return Err("packet range".into());
        }
        if self.events.len() > 256 {
            return Err("event limit".into());
        }
        for e in &self.events {
            if match e[0] {
                0 => !(0..=60).contains(&e[1]) || !(0..=3).contains(&e[2]),
                1 => e[1] <= 0 || e[2] != 0,
                2 => {
                    !(0..56).contains(&e[1])
                        || !(1..=56).contains(&e[2])
                        || self.objects.iter().any(|o| o.slot == e[1] as u32 + 6)
                }
                _ => true,
            } {
                return Err("event range/identity".into());
            }
        }
        let mut gone_started = false;
        let mut gone = std::collections::BTreeSet::new();
        for (i, e) in self.events.iter().enumerate() {
            match e[0] {
                0 => {
                    if gone_started || self.events.get(i + 1).is_none_or(|next| next[0] != 1) {
                        return Err("contact/sound pairing".into());
                    }
                }
                1 => {
                    if i == 0 || self.events[i - 1][0] != 0 {
                        return Err("contact/sound pairing".into());
                    }
                }
                2 => {
                    gone_started = true;
                    if !gone.insert(e[1])
                        || !self.events.iter().any(|c| c[0] == 0 && c[1] == e[1] + 5)
                    {
                        return Err("unknown/duplicate destroyed collider".into());
                    }
                }
                _ => {}
            }
        }
        let mut ids = BTreeSet::new();
        for o in &self.objects {
            if o.slot >= 128 || o.generation == 0 || o.shape > 2 || !ids.insert(o.slot) {
                return Err("object identity".into());
            }
            if o.values
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 100000.0)
                || o.values[3..6].iter().any(|v| *v <= 0.0)
                || o.values[6..10].iter().any(|v| !(0.0..=1.0).contains(v))
            {
                return Err("object numeric range".into());
            }
        }
        Ok(())
    }
}
/// Entire output admission happens before the last-good packet changes.
/// Tombstones prevent destroyed slots from being resurrected with old generations.
pub struct Publication {
    session: u64,
    last: Option<Packet>,
    generations: std::collections::BTreeMap<u32, u32>,
    closed: bool,
}
impl Publication {
    pub fn new(session: u64) -> Self {
        Self {
            session,
            last: None,
            generations: Default::default(),
            closed: false,
        }
    }
    pub fn last(&self) -> Option<&Packet> {
        self.last.as_ref()
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.generations.clear();
    }
    pub fn publish(&mut self, p: Packet) -> Result<(), String> {
        if self.closed || p.session != self.session {
            return Err("closed/foreign session".into());
        }
        if self
            .last
            .as_ref()
            .is_some_and(|old| p.sequence <= old.sequence)
        {
            return Err("stale sequence".into());
        }
        p.validate()?;
        let prior = self.last.as_ref().map_or(0, |p| p.sounds);
        let sounds: Vec<_> = p.events.iter().filter(|e| e[0] == 1).collect();
        if p.sounds < prior
            || p.sounds - prior != sounds.len() as i32
            || sounds
                .iter()
                .enumerate()
                .any(|(i, e)| e[1] != prior + i as i32 + 1)
        {
            return Err("sound event sequence".into());
        }
        for event in p.events.iter().filter(|e| e[0] == 2) {
            if !self
                .last
                .as_ref()
                .is_some_and(|old| old.objects.iter().any(|o| o.slot == event[1] as u32 + 6))
            {
                return Err("unknown destroyed object".into());
            }
        }
        for o in &p.objects {
            if let Some(g) = self.generations.get(&o.slot) {
                let live = self
                    .last
                    .as_ref()
                    .is_some_and(|old| old.objects.iter().any(|v| v.slot == o.slot));
                if (live && o.generation != *g) || (!live && o.generation <= *g) {
                    return Err("stale generation".into());
                }
            }
        }
        for o in &p.objects {
            self.generations.insert(o.slot, o.generation);
        }
        self.last = Some(p);
        Ok(())
    }
}
