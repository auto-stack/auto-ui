use crate::{
    output::{Object, Packet, Publication},
    Progress, ResultKind, Scalar, Session,
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
#[derive(Clone)]
pub enum Action {
    Tick(i32),
    Pointer([f32; 2]),
    Key(i32),
    Destroy,
}
#[derive(Clone, Copy)]
enum Read {
    Action,
    Version,
    Background(usize),
    Count,
    Changed(i32),
    Alive(i32),
    Generation(i32),
    Shape(i32),
    Field(i32, usize),
    Score,
    Sound,
    EventCount,
    Event(i32, usize),
}
struct Query {
    name: &'static str,
    args: Vec<Scalar>,
    kind: ResultKind,
    read: Read,
}
pub struct Script {
    pub session: Session,
    pub publication: Publication,
    id: u64,
    sequence: u64,
    queue: VecDeque<Action>,
    query: Option<Query>,
    pending: bool,
    read: Read,
    draft: Packet,
    current: Option<Object>,
    count: i32,
    last_tick: Option<u64>,
    initializing: bool,
    pub completed_ticks: u64,
    pub outputs: VecDeque<Packet>,
    pub last_error: Option<String>,
    pub pump_us: Vec<u128>,
}
impl Script {
    pub fn new(source: &str) -> Result<Self, String> {
        let id = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            session: Session::new(source)?,
            publication: Publication::new(id),
            id,
            sequence: 0,
            queue: VecDeque::new(),
            query: None,
            pending: false,
            read: Read::Action,
            draft: Packet {
                session: id,
                sequence: 0,
                version: 0,
                background: [0.0, 0.0, 0.0, 1.0],
                objects: vec![],
                score: 0,
                sounds: 0,
                events: vec![],
            },
            current: None,
            count: 0,
            last_tick: None,
            initializing: true,
            completed_ticks: 0,
            outputs: VecDeque::new(),
            last_error: None,
            pump_us: vec![],
        })
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn tick(&mut self, tick: u64, direction: i32) -> Result<bool, String> {
        if !(-1..=1).contains(&direction) {
            return Err("invalid direction".into());
        }
        if self.last_tick.is_some_and(|v| tick <= v) {
            return Ok(false);
        }
        self.enqueue(self.id, Action::Tick(direction))?;
        self.last_tick = Some(tick);
        Ok(true)
    }
    pub fn enqueue(&mut self, session: u64, action: Action) -> Result<(), String> {
        if self.session.closed() || session != self.id {
            return Err("closed/stale input session".into());
        }
        if self.queue.len() >= 64 {
            return Err("action queue full".into());
        }
        self.queue.push_back(action);
        Ok(())
    }
    pub fn idle(&self) -> bool {
        !self.initializing
            && self.queue.is_empty()
            && self.query.is_none()
            && matches!(self.read, Read::Action)
    }
    pub fn close(&mut self) {
        self.outputs.clear();
        self.queue.clear();
        self.query = None;
        self.current = None;
        self.draft.objects.clear();
        self.pending = false;
        self.session.close();
        self.publication.close();
    }
    fn make_query(&mut self) -> Option<Query> {
        use ResultKind::{Float as F, Int as I, Unit as U};
        use Scalar::{Float, Int};
        let (name, args, kind) = match self.read {
            Read::Action => {
                if self.initializing {
                    ("init", vec![], U)
                } else {
                    match self.queue.pop_front()? {
                        Action::Tick(v) => ("update", vec![Int(v)], U),
                        Action::Pointer(p) => ("pointer", vec![Float(p[0]), Float(p[1])], U),
                        Action::Key(v) => ("key", vec![Int(v)], U),
                        Action::Destroy => ("destroy", vec![], U),
                    }
                }
            }
            Read::Version => ("protocol", vec![], I),
            Read::Background(f) => ("background", vec![Int(f as i32)], F),
            Read::Count => ("object_count", vec![], I),
            Read::Changed(i) => ("changed", vec![Int(i)], I),
            Read::Alive(i) => ("object_alive", vec![Int(i)], I),
            Read::Generation(i) => ("generation", vec![Int(i)], I),
            Read::Shape(i) => ("shape", vec![Int(i)], I),
            Read::Field(i, f) => ("field", vec![Int(i), Int(f as i32)], F),
            Read::Score => ("integer", vec![Int(0)], I),
            Read::Sound => ("integer", vec![Int(1)], I),
            Read::EventCount => ("event_count", vec![], I),
            Read::Event(i, f) => (["event_kind", "event_a", "event_b"][f], vec![Int(i)], I),
        };
        Some(Query {
            name,
            args,
            kind,
            read: self.read,
        })
    }
    fn int(v: Option<Scalar>) -> Result<i32, String> {
        if let Some(Scalar::Int(v)) = v {
            Ok(v)
        } else {
            Err("protocol expected int".into())
        }
    }
    fn next_object(&mut self, i: i32) {
        self.read = if i + 1 < self.count {
            Read::Changed(i + 1)
        } else {
            Read::Score
        };
    }
    fn publish_draft(&mut self) -> Result<(), String> {
        self.publication.publish(self.draft.clone())?;
        self.outputs.push_back(self.draft.clone());
        Ok(())
    }
    fn accept(&mut self, r: Read, v: Option<Scalar>, is_tick: bool) -> Result<(), String> {
        match r {
            Read::Action => {
                if self.initializing {
                    self.initializing = false
                } else if is_tick {
                    self.completed_ticks += 1
                }
                self.sequence += 1;
                self.draft = Packet {
                    session: self.id,
                    sequence: self.sequence,
                    version: 0,
                    background: [0.0, 0.0, 0.0, 1.0],
                    objects: self
                        .publication
                        .last()
                        .map_or_else(Vec::new, |p| p.objects.clone()),
                    score: 0,
                    sounds: 0,
                    events: vec![],
                };
                self.read = Read::Version;
            }
            Read::Version => {
                self.draft.version = Self::int(v)?;
                if self.draft.version != 1 {
                    return Err("protocol version".into());
                }
                self.read = Read::Background(0);
            }
            Read::Background(f) => {
                let Some(Scalar::Float(value)) = v else {
                    return Err("background type".into());
                };
                self.draft.background[f] = value;
                self.read = if f == 3 {
                    Read::Count
                } else {
                    Read::Background(f + 1)
                };
            }
            Read::Count => {
                self.count = Self::int(v)?;
                if !(0..=128).contains(&self.count) {
                    return Err("object limit".into());
                }
                self.draft.objects.retain(|o| o.slot < self.count as u32);
                self.read = if self.count > 0 {
                    Read::Changed(0)
                } else {
                    Read::Score
                };
            }
            Read::Changed(i) => match Self::int(v)? {
                0 => self.next_object(i),
                1 => self.read = Read::Alive(i),
                _ => return Err("changed flag".into()),
            },
            Read::Alive(i) => match Self::int(v)? {
                0 => {
                    self.draft.objects.retain(|o| o.slot != i as u32);
                    self.next_object(i)
                }
                1 => {
                    self.current = Some(Object {
                        slot: i as u32,
                        generation: 0,
                        shape: 0,
                        values: [0.0; 10],
                    });
                    self.read = Read::Generation(i);
                }
                _ => return Err("alive flag".into()),
            },
            Read::Generation(i) => {
                let g = Self::int(v)?;
                if g <= 0 {
                    return Err("generation".into());
                }
                self.current.as_mut().unwrap().generation = g as u32;
                self.read = Read::Shape(i);
            }
            Read::Shape(i) => {
                let s = Self::int(v)?;
                if !(0..=2).contains(&s) {
                    return Err("shape".into());
                }
                self.current.as_mut().unwrap().shape = s as u8;
                self.read = Read::Field(i, 0);
            }
            Read::Field(i, f) => {
                let Some(Scalar::Float(value)) = v else {
                    return Err("field type".into());
                };
                self.current.as_mut().unwrap().values[f] = value;
                if f == 9 {
                    self.draft.objects.retain(|o| o.slot != i as u32);
                    self.draft.objects.push(self.current.take().unwrap());
                    self.next_object(i);
                } else {
                    self.read = Read::Field(i, f + 1);
                }
            }
            Read::Score => {
                self.draft.score = Self::int(v)?;
                self.read = Read::Sound;
            }
            Read::Sound => {
                self.draft.sounds = Self::int(v)?;
                self.read = Read::EventCount;
            }
            Read::EventCount => {
                let n = Self::int(v)?;
                if !(0..=256).contains(&n) {
                    return Err("event limit".into());
                }
                self.draft.events = vec![[0; 3]; n as usize];
                if n == 0 {
                    self.publish_draft()?;
                    self.read = Read::Action;
                } else {
                    self.read = Read::Event(0, 0);
                }
            }
            Read::Event(i, f) => {
                self.draft.events[i as usize][f] = Self::int(v)?;
                if f < 2 {
                    self.read = Read::Event(i, f + 1);
                } else if (i as usize) + 1 < self.draft.events.len() {
                    self.read = Read::Event(i + 1, 0);
                } else {
                    self.publish_draft()?;
                    self.read = Read::Action;
                }
            }
        }
        Ok(())
    }
    /// One event-loop round; no callback reentry and no partial output publication.
    pub fn pump(&mut self) -> Result<bool, String> {
        if self.session.closed() {
            return Err(self.last_error.clone().unwrap_or("closed".into()));
        }
        let now = Instant::now();
        let before = self.publication.last().map(|p| p.sequence);
        let result: Result<bool, String> = (|| {
            while now.elapsed() < Duration::from_millis(8) {
                if self.outputs.len() >= 64 {
                    break;
                }
                if self.query.is_none() {
                    self.query = self.make_query();
                    if self.query.is_none() {
                        break;
                    }
                }
                let q = self.query.as_ref().unwrap();
                let p = if self.pending {
                    self.session.pump()?
                } else {
                    self.session.start(q.name, &q.args, q.kind)?
                };
                match p {
                    Progress::Pending => {
                        self.pending = true;
                    }
                    Progress::Complete(v) => {
                        self.pending = false;
                        let q = self.query.take().unwrap();
                        self.accept(q.read, v, q.name == "update")?;
                    }
                }
            }
            Ok(before != self.publication.last().map(|p| p.sequence))
        })();
        if let Err(e) = &result {
            self.last_error = Some(e.clone());
            let outputs = std::mem::take(&mut self.outputs);
            self.close();
            self.outputs = outputs
        }
        if self.pump_us.len() < 10000 {
            self.pump_us.push(now.elapsed().as_micros());
        }
        result
    }
}
impl Drop for Script {
    fn drop(&mut self) {
        self.close()
    }
}
