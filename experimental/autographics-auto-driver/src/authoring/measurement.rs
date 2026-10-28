//! Bounded, opt-in CPU observations. Stores numbers only, never runtime owners.
use std::time::Instant;

pub const SAMPLE_LIMIT: usize = 4096;
#[derive(Default, Debug, serde::Serialize)]
pub struct Measurement {
    pub enabled: bool,
    pub samples_ns: Vec<u64>,
    pub dropped: usize,
}
impl Measurement {
    pub fn start(&self) -> Option<Instant> {
        self.enabled.then(Instant::now)
    }
    pub fn finish(&mut self, start: Option<Instant>) {
        if let Some(start) = start {
            if self.samples_ns.len() < SAMPLE_LIMIT {
                self.samples_ns
                    .push(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
            } else {
                self.dropped = self.dropped.saturating_add(1);
            }
        }
    }
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.samples_ns.clear();
        self.dropped = 0;
    }
}
