//! Explicit profile 2. Reuses the public Session CPU-slice bridge; no Rust game simulation.
use crate::{Progress, ResultKind, Scalar, Session};
use glam::{Mat4, Vec3};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use Scalar::{Float, Int};
static SESSION: AtomicU64 = AtomicU64::new(1000000);
pub fn source() -> String {
    format!(
        "{}\n{}",
        include_str!("../samples/squash/math.at"),
        include_str!("../samples/squash/main.at")
    )
}
#[derive(Clone, Debug, PartialEq)]
pub struct Object3d {
    pub slot: u32,
    pub generation: u32,
    pub identity: i32,
    pub resource: i32,
    pub fields: [f32; 9],
}
impl Object3d {
    pub fn matrix(&self) -> Mat4 {
        let f = self.fields;
        Mat4::from_translation(Vec3::new(f[0], f[1], f[2]))
            * Mat4::from_rotation_y(f[3])
            * Mat4::from_rotation_x(f[4])
            * Mat4::from_rotation_z(f[5])
            * Mat4::from_scale(Vec3::new(f[6], f[7], f[8]))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Packet3d {
    pub session: u64,
    pub sequence: u64,
    pub profile: i32,
    pub objects: Vec<Object3d>,
    pub score: i32,
    pub dead: bool,
    pub events: Vec<[i32; 4]>,
}
impl Packet3d {
    pub fn validate(&self) -> Result<(), String> {
        if self.profile != 2 {
            return Err("3d profile mismatch".into());
        }
        if self.objects.len() > 65 || self.events.len() > 130 || self.score < 0 {
            return Err("3d capacity/range".into());
        }
        let mut slots = BTreeSet::new();
        let mut identities = BTreeSet::new();
        for o in &self.objects {
            if o.slot > 64
                || o.generation == 0
                || !slots.insert(o.slot)
                || !identities.insert(o.identity)
                || !matches!(
                    (o.slot, o.resource, o.identity),
                    (0, 0, -1) | (1..=64, 1, 0..)
                )
            {
                return Err("3d object identity/resource".into());
            }
            if o.fields
                .iter()
                .any(|f| !f.is_finite() || f.abs() > 100000.0)
                || o.fields[6..].iter().any(|f| *f <= 0.0)
            {
                return Err("3d numeric transform".into());
            }
        }
        if self.objects.iter().any(|o| o.slot == 0) == self.dead {
            return Err("3d player alive mismatch".into());
        }
        Ok(())
    }
}
pub struct Publication3d {
    session: u64,
    last: Option<Packet3d>,
    generations: BTreeMap<u32, u32>,
    closed: bool,
    next_spawn: i32,
}
impl Publication3d {
    pub fn new(session: u64) -> Self {
        Self {
            session,
            last: None,
            generations: BTreeMap::new(),
            closed: false,
            next_spawn: 0,
        }
    }
    pub fn last(&self) -> Option<&Packet3d> {
        self.last.as_ref()
    }
    pub fn publish(&mut self, p: Packet3d) -> Result<(), String> {
        p.validate()?;
        if self.closed
            || p.session != self.session
            || Some(p.sequence)
                != self
                    .last
                    .as_ref()
                    .map_or(Some(1), |v| v.sequence.checked_add(1))
        {
            return Err("3d stale session/sequence".into());
        }
        let old = self.last.as_ref();
        let mut alive: BTreeMap<_, _> = old
            .into_iter()
            .flat_map(|v| v.objects.iter())
            .filter(|o| o.slot > 0)
            .map(|o| (o.identity, o.slot))
            .collect();
        let mut score = old.map_or(0, |v| v.score);
        let mut dead = old.is_some_and(|v| v.dead);
        let mut next_spawn = self.next_spawn;
        let mut phase = 0;
        let mut squashes = 0;
        for e in &p.events {
            let event_phase = match e[0] {
                0 => 0,
                2 => 1,
                1 | 3 => 2,
                _ => 3,
            };
            if event_phase < phase {
                return Err("3d event ordering".into());
            }
            phase = event_phase;
            if e[0] == 0 {
                if e[1] != next_spawn {
                    return Err("3d spawn ordinal".into());
                }
                next_spawn = next_spawn.checked_add(1).ok_or("3d identity capacity")?;
            }
            if e[0] == 1 {
                squashes += 1;
                if squashes > 1 {
                    return Err("3d more than one squash".into());
                }
            }

            match e[0] {
                0 if !dead && e[1] >= 0 && (1..=64).contains(&e[2]) && e[3] == 0 => {
                    if alive.values().any(|s| *s == e[2] as u32)
                        || alive.insert(e[1], e[2] as u32).is_some()
                    {
                        return Err("duplicate 3d spawn".into());
                    }
                }
                1 if !dead && e[3] == score + 1 => {
                    if alive.remove(&e[1]) != Some(e[2] as u32) {
                        return Err("unknown squash".into());
                    }
                    score += 1;
                }
                2 if e[3] == 0 => {
                    if alive.remove(&e[1]) != Some(e[2] as u32) {
                        return Err("unknown despawn".into());
                    }
                }
                3 if !dead && e[1] == 0 && e[2] == 1 && e[3] == score => {
                    dead = true;
                }
                _ => return Err("3d event state/order".into()),
            }
        }
        if score != p.score
            || dead != p.dead
            || alive
                != p.objects
                    .iter()
                    .filter(|o| o.slot > 0)
                    .map(|o| (o.identity, o.slot))
                    .collect()
        {
            return Err("3d event/snapshot mismatch".into());
        }
        for o in &p.objects {
            if let Some(g) = self.generations.get(&o.slot) {
                let was_live = old.is_some_and(|p| p.objects.iter().any(|v| v.slot == o.slot));
                if (was_live && o.generation != *g) || (!was_live && o.generation <= *g) {
                    return Err("3d stale generation".into());
                }
            }
        }
        for o in &p.objects {
            self.generations.insert(o.slot, o.generation);
        }
        self.next_spawn = next_spawn;
        self.last = Some(p);
        Ok(())
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.generations.clear();
    }
}
#[derive(Clone, Copy)]
enum Read {
    ObjectCount,
    Unit,
    Profile,
    Request,
    Random(i32),
    Alive(i32),
    Generation,
    Identity,
    Resource,
    Field(usize),
    Score,
    Dead,
    Count,
    Event(usize, usize),
    Failure,
}
struct Op {
    name: &'static str,
    args: Vec<Scalar>,
    kind: ResultKind,
    read: Read,
}
pub struct SquashScript {
    pub session: Session,
    pub publication: Publication3d,
    pub outputs: VecDeque<Packet3d>,
    id: u64,
    sequence: u64,
    ops: VecDeque<Op>,
    active: Option<Op>,
    pending: bool,
    draft: Packet3d,
    current: Option<Object3d>,
    rng: agp::core::DeterministicRng,
    candidate_rng: Option<agp::core::DeterministicRng>,
    staged_rng: Option<agp::core::DeterministicRng>,
    staged_sequence: i32,
    random_sequence: i32,
    last_tick: Option<u64>,
    actions: VecDeque<[i32; 3]>,
    active_tick: bool,
    pub completed_ticks: u64,
    pub pump_us: Vec<u128>,
    pub last_error: Option<String>,
}
impl SquashScript {
    pub fn new(source: &str) -> Result<Self, String> {
        let id = SESSION.fetch_add(1, Ordering::Relaxed);
        let mut s = Self {
            session: Session::new(source)?,
            publication: Publication3d::new(id),
            outputs: VecDeque::new(),
            id,
            sequence: 0,
            ops: VecDeque::new(),
            active: None,
            pending: false,
            draft: Packet3d {
                session: id,
                sequence: 0,
                profile: 0,
                objects: vec![],
                score: 0,
                dead: false,
                events: vec![],
            },
            current: None,
            rng: agp::core::DeterministicRng::new(27),
            candidate_rng: None,
            staged_rng: None,
            staged_sequence: 0,
            random_sequence: 0,
            last_tick: None,
            actions: VecDeque::new(),
            active_tick: false,
            completed_ticks: 0,
            pump_us: vec![],
            last_error: None,
        };
        s.push("profile", vec![], ResultKind::Int, Read::Profile);
        s.push("init", vec![], ResultKind::Unit, Read::Unit);
        for (i, v) in agp::render3d::Camera3d::sample_camera(1.5)
            .view_proj()
            .to_cols_array()
            .iter()
            .enumerate()
        {
            s.push(
                "setup_camera",
                vec![Int(i as i32), Float(*v)],
                ResultKind::Unit,
                Read::Unit,
            );
        }
        s.snapshot();
        Ok(s)
    }
    pub fn accepted_random_draws(&self) -> i32 {
        self.random_sequence
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    fn push(&mut self, name: &'static str, args: Vec<Scalar>, kind: ResultKind, read: Read) {
        self.ops.push_back(Op {
            name,
            args,
            kind,
            read,
        });
    }
    fn front(&mut self, name: &'static str, args: Vec<Scalar>, kind: ResultKind, read: Read) {
        self.ops.push_front(Op {
            name,
            args,
            kind,
            read,
        });
    }
    pub fn idle(&self) -> bool {
        self.ops.is_empty() && self.active.is_none() && self.actions.is_empty()
    }
    pub fn tick(&mut self, session: u64, tick: u64, action: [i32; 3]) -> Result<bool, String> {
        if session != self.id || self.session.closed() {
            return Err("stale/closed 3d input".into());
        }
        if self.last_tick.is_some_and(|t| tick <= t) {
            return Ok(false);
        }
        if self.actions.len() >= 64 {
            return Err("3d action backlog".into());
        }
        if !(-1..=1).contains(&action[0])
            || !(-1..=1).contains(&action[1])
            || !(0..=1).contains(&action[2])
        {
            return Err("3d action range".into());
        }
        self.actions.push_back(action);
        self.last_tick = Some(tick);
        Ok(true)
    }
    fn begin_action(&mut self, action: [i32; 3]) {
        self.active_tick = true;
        self.staged_rng = Some(self.rng.clone());
        self.staged_sequence = self.random_sequence;
        self.push(
            "begin_tick",
            action.into_iter().map(Int).collect(),
            ResultKind::Unit,
            Read::Unit,
        );
        self.push("random_request", vec![], ResultKind::Int, Read::Request);
        for _ in 0..8 {
            self.push("move_chunk", vec![], ResultKind::Unit, Read::Unit);
        }
        self.push("finish_tick", vec![], ResultKind::Unit, Read::Unit);
        self.snapshot();
    }
    fn snapshot(&mut self) {
        self.push("profile", vec![], ResultKind::Int, Read::Profile);
        self.push("status", vec![Int(4)], ResultKind::Int, Read::Failure);
        self.push("object_count", vec![], ResultKind::Int, Read::ObjectCount);
        self.push("status", vec![Int(0)], ResultKind::Int, Read::Score);
        self.push("status", vec![Int(1)], ResultKind::Int, Read::Dead);
        self.push("event_count", vec![], ResultKind::Int, Read::Count);
    }
    fn integer(v: Option<Scalar>) -> Result<i32, String> {
        if let Some(Int(i)) = v {
            Ok(i)
        } else {
            Err("3d int result".into())
        }
    }
    fn accept(&mut self, r: Read, v: Option<Scalar>) -> Result<(), String> {
        match r {
            Read::ObjectCount => {
                let n = Self::integer(v)?;
                if !(1..=65).contains(&n) {
                    return Err("3d object capacity".into());
                }
                for i in (0..n).rev() {
                    self.front(
                        "object_alive",
                        vec![Int(i)],
                        ResultKind::Int,
                        Read::Alive(i),
                    );
                }
            }
            Read::Unit => {}
            Read::Profile => {
                if Self::integer(v)? != 2 {
                    return Err("3d profile mismatch".into());
                }
            }
            Read::Request => match Self::integer(v)? {
                0 => {}
                1 => {
                    let mut rng = self.staged_rng.as_ref().unwrap_or(&self.rng).clone();
                    let value = rng.next_f32();
                    self.candidate_rng = Some(rng);
                    self.front(
                        "random_value",
                        vec![Int(self.staged_sequence + 1), Float(value)],
                        ResultKind::Int,
                        Read::Random(self.staged_sequence + 1),
                    );
                }
                _ => return Err("3d random request".into()),
            },
            Read::Random(sequence) => {
                if Self::integer(v)? != 1 {
                    return Err("3d random rejected".into());
                }
                self.staged_rng = self.candidate_rng.take();
                self.staged_sequence = sequence;
                self.front("random_request", vec![], ResultKind::Int, Read::Request);
            }
            Read::Alive(i) => match Self::integer(v)? {
                0 => {}
                1 => {
                    self.current = Some(Object3d {
                        slot: i as u32,
                        generation: 0,
                        identity: 0,
                        resource: -1,
                        fields: [0.0; 9],
                    });
                    self.front(
                        "generation",
                        vec![Int(i)],
                        ResultKind::Int,
                        Read::Generation,
                    )
                }
                _ => return Err("3d alive flag".into()),
            },
            Read::Generation => {
                let value = Self::integer(v)?;
                if value <= 0 {
                    return Err("3d generation".into());
                }
                let o = self.current.as_mut().unwrap();
                o.generation = value as u32;
                let slot = o.slot;
                self.front(
                    "identity",
                    vec![Int(slot as i32)],
                    ResultKind::Int,
                    Read::Identity,
                )
            }
            Read::Identity => {
                let value = Self::integer(v)?;
                let o = self.current.as_mut().unwrap();
                o.identity = value;
                let slot = o.slot;
                self.front(
                    "role",
                    vec![Int(slot as i32)],
                    ResultKind::Int,
                    Read::Resource,
                )
            }
            Read::Resource => {
                let value = Self::integer(v)?;
                let o = self.current.as_mut().unwrap();
                o.resource = value;
                let slot = o.slot;
                self.front(
                    "transform_field",
                    vec![Int(slot as i32), Int(0)],
                    ResultKind::Float,
                    Read::Field(0),
                )
            }
            Read::Field(f) => {
                let Some(Float(value)) = v else {
                    return Err("3d float result".into());
                };
                let o = self.current.as_mut().unwrap();
                o.fields[f] = value;
                let slot = o.slot;
                if f < 8 {
                    self.front(
                        "transform_field",
                        vec![Int(slot as i32), Int(f as i32 + 1)],
                        ResultKind::Float,
                        Read::Field(f + 1),
                    )
                } else {
                    self.draft.objects.push(self.current.take().unwrap());
                }
            }
            Read::Score => self.draft.score = Self::integer(v)?,
            Read::Dead => {
                self.draft.dead = match Self::integer(v)? {
                    0 => false,
                    1 => true,
                    _ => return Err("3d dead flag".into()),
                }
            }
            Read::Count => {
                let n = Self::integer(v)?;
                if !(0..=130).contains(&n) {
                    return Err("3d event capacity".into());
                }
                self.draft.events = vec![[0; 4]; n as usize];
                if n == 0 {
                    self.publish()?
                } else {
                    self.front(
                        "event_field",
                        vec![Int(0), Int(0)],
                        ResultKind::Int,
                        Read::Event(0, 0),
                    )
                }
            }
            Read::Event(i, f) => {
                self.draft.events[i][f] = Self::integer(v)?;
                if f < 3 {
                    self.front(
                        "event_field",
                        vec![Int(i as i32), Int(f as i32 + 1)],
                        ResultKind::Int,
                        Read::Event(i, f + 1),
                    )
                } else if i + 1 < self.draft.events.len() {
                    self.front(
                        "event_field",
                        vec![Int(i as i32 + 1), Int(0)],
                        ResultKind::Int,
                        Read::Event(i + 1, 0),
                    )
                } else {
                    self.publish()?
                }
            }
            Read::Failure => {
                if Self::integer(v)? != 0 {
                    return Err("3d object capacity".into());
                }
            }
        }
        Ok(())
    }
    fn publish(&mut self) -> Result<(), String> {
        self.sequence += 1;
        self.draft.sequence = self.sequence;
        self.draft.profile = 2;
        self.publication.publish(self.draft.clone())?;
        if let Some(rng) = self.staged_rng.take() {
            self.rng = rng;
            self.random_sequence = self.staged_sequence;
        }
        self.outputs.push_back(self.draft.clone());
        self.draft.objects.clear();
        self.draft.events.clear();
        if self.active_tick {
            self.completed_ticks += 1;
            self.active_tick = false;
        }
        Ok(())
    }
    pub fn pump(&mut self) -> Result<(), String> {
        let start = Instant::now();
        let result: Result<(), String> = (|| {
            while start.elapsed() < Duration::from_millis(8) && self.outputs.len() < 64 {
                if self.active.is_none() && self.ops.is_empty() {
                    if let Some(action) = self.actions.pop_front() {
                        self.begin_action(action);
                    }
                }
                if self.active.is_none() {
                    self.active = self.ops.pop_front();
                }
                let Some(op) = &self.active else { break };
                let p = if self.pending {
                    self.session.pump()?
                } else {
                    self.session.start(op.name, &op.args, op.kind)?
                };
                match p {
                    Progress::Pending => self.pending = true,
                    Progress::Complete(v) => {
                        self.pending = false;
                        let op = self.active.take().unwrap();
                        self.accept(op.read, v)?;
                    }
                }
            }
            Ok(())
        })();
        if self.pump_us.len() < 10000 {
            self.pump_us.push(start.elapsed().as_micros());
        }
        if let Err(e) = &result {
            self.last_error = Some(e.clone());
            self.close();
        }
        result
    }
    pub fn settle(&mut self) -> Result<(), String> {
        let start = Instant::now();
        while !self.idle() {
            self.pump()?;
            if start.elapsed() > Duration::from_secs(2) {
                self.close();
                return Err("3d settle deadline".into());
            }
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.actions.clear();
        self.ops.clear();
        self.active = None;
        self.candidate_rng = None;
        self.staged_rng = None;
        self.current = None;
        self.draft.objects.clear();
        self.outputs.clear();
        self.session.close();
        self.publication.close();
    }
}
impl Drop for SquashScript {
    fn drop(&mut self) {
        self.close();
    }
}
