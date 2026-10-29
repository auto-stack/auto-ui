//! Driver-local retained Auto world. Publication is explicit, after GPU preparation.
use super::{
    assets,
    document::{Document, Object},
    game_input::GameInput,
    hierarchy::{self, Derived, Node},
    runtime_camera::{
        basis_from_yaw, normalize_yaw_deg, CameraDim, CameraInput, RuntimeCameraFrame, DIST_MAX,
        DIST_MIN, ORTHO_SIZE_MAX, ORTHO_SIZE_MIN, PITCH_MAX_DEG, PITCH_MIN_DEG,
        ZOOM_FACTOR_PER_NOTCH,
    },
    session::Editor,
};
use crate::{Progress, ResultKind, Scalar, Session};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::atomic::{AtomicI32, Ordering},
    time::{Duration, Instant},
};
pub const SOURCE: &str = include_str!("../../samples/authoring_play/main.at");
pub const DT: f32 = 1.0 / 60.0;
static RUN: AtomicI32 = AtomicI32::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Starting,
    Running,
    Pausing,
    Paused,
    Stopping,
    Faulted,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Action {
    pub space: i32,
    pub x: i32,
    pub y: i32,
    pub toggle: bool,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct TickReport {
    pub tick: i32,
    pub space: i32,
    pub x: i32,
    pub y: i32,
    pub toggle: bool,
    pub elapsed_us: u128,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Toggle,
}
/// Which public character profile a run is executing. The wire identities are
/// independent (`autographics.character/1` vs `autographics.character3d/1`)
/// and the routers only emit the matching action range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharProfile {
    TwoD,
    ThreeD,
}
impl CharProfile {
    pub fn actions(self) -> usize {
        match self {
            Self::TwoD => super::game_input::ACTIONS,
            Self::ThreeD => super::game_input::ACTIONS3,
        }
    }
    pub fn map_call(self) -> &'static str {
        match self {
            Self::TwoD => "_ag_char_map",
            Self::ThreeD => "_ag_char3_map",
        }
    }
    pub fn input_call(self) -> &'static str {
        match self {
            Self::TwoD => "_ag_char_input",
            Self::ThreeD => "_ag_char3_input",
        }
    }
}
#[derive(Default)]
pub struct Input {
    focus: i32,
    held: [bool; 4],
    toggle: bool,
}
impl Input {
    pub fn clear(&mut self) {
        self.held = [false; 4];
        self.toggle = false;
    }
    pub fn focus(&mut self, space: i32) {
        if self.focus != space {
            self.clear();
        }
        self.focus = space;
    }
    pub fn key(&mut self, k: Key, down: bool, repeat: bool) {
        let i = match k {
            Key::Left => 0,
            Key::Right => 1,
            Key::Up => 2,
            Key::Down => 3,
            Key::Toggle => 4,
        };
        if i < 4 && !down {
            self.held[i] = false;
            return;
        }
        if repeat || !down || !matches!(self.focus, 2 | 3) {
            return;
        }
        if i == 4 {
            self.toggle = !self.toggle;
        } else {
            self.held[i] = true;
        }
    }
    pub fn sample(&mut self) -> Action {
        let a = Action {
            space: self.focus,
            x: i32::from(self.held[1]) - i32::from(self.held[0]),
            y: i32::from(self.held[2]) - i32::from(self.held[3]),
            toggle: self.toggle,
        };
        self.toggle = false;
        a
    }
}
#[derive(Clone, Debug)]
pub struct Frame {
    pub document: Document,
    pub objects: Vec<Object>,
    pub nodes: BTreeMap<u64, Derived>,
    pub tick: i32,
    pub run_id: i32,
    /// The camera outputs admitted with this frame; frame and camera pass
    /// prepare/accept/publish as one transaction (contract section 5).
    pub camera: Option<RuntimeCameraFrame>,
    /// The admitted runtime object instances; objects, character and camera
    /// are one transaction. `None` outside the objects profiles.
    pub instances: Option<super::runtime_objects::RuntimeObjectsFrame>,
    /// The admitted trigger events and restart marker; the whole candidate is
    /// one transaction. `None` outside the triggers profiles.
    pub rounds: Option<super::runtime_round::RuntimeRoundFrame>,
    /// PLAN-004 admitted sprite frames of every bound leaf and live slot. It
    /// rides the same candidate, so a new position can never publish beside an
    /// old frame.
    pub sprites: Option<super::runtime_sprites::SpriteFrame>,
    pub animations: Option<super::runtime_node_animation::AnimationFrame>,
}

/// Per-tick camera output block sizes appended to the frame read layout.
fn camera_block_len(dim: Option<CameraDim>) -> usize {
    match dim {
        Some(CameraDim::TwoD) => 10,
        Some(CameraDim::ThreeD) => 9,
        None => 0,
    }
}
struct Call {
    name: &'static str,
    args: Vec<Scalar>,
    kind: ResultKind,
}
impl Clone for Call {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            args: self.args.clone(),
            kind: self.kind,
        }
    }
}
impl Call {
    fn unit(name: &'static str, args: Vec<Scalar>) -> Self {
        Self {
            name,
            args,
            kind: ResultKind::Unit,
        }
    }
}
pub struct Play {
    pub vm_output_measurement: super::measurement::Measurement,
    pub admission_measurement: super::measurement::Measurement,
    pub state: State,
    pub error: Option<String>,
    pub run_id: i32,
    pub source_hash: String,
    pub initial_document_hash: Option<String>,
    pub user_source: Option<super::behavior::SourceSnapshot>,
    pub base_identity: serde_json::Value,
    pub assets: assets::Registry,
    pub last: Option<Frame>,
    pub input: Input,
    /// The actor binding this Run was admitted with; a round restart rebuilds
    /// the Play from the same snapshot and the same actors.
    pub actors: [Option<u64>; 2],
    pub compile_us: u128,
    pub spatial_configuration_calls: usize,
    pub spatial_configuration_us: u128,
    pub reports: VecDeque<TickReport>,
    base: Document,
    order: Vec<Node>,
    vm: Option<Session>,
    queue: VecDeque<Call>,
    read: Vec<Scalar>,
    active: bool,
    candidate: Option<Frame>,
    transaction: Option<Instant>,
    starting: Instant,
    next_tick: i32,
    step: bool,
    pause_after_start: bool,
    last_wall: Option<Instant>,
    debt: f64,
    credits: u8,
    action: Action,
    expected_profile: i32,
    spatial: Option<std::sync::Arc<super::spatial::Descriptor>>,
    character: Option<CharProfile>,
    char: GameInput,
    camera_dim: Option<CameraDim>,
    camera_frame: Option<RuntimeCameraFrame>,
    cam_sent: Option<[f32; 3]>,
    cam: CameraInput,
    /// Finite run-object profile (8 reserved slots) riding this Run.
    objects_dim: Option<super::runtime_objects::Dim>,
    /// Per-slot generation high-water of every ADMITTED frame; a rebirth must
    /// exceed it, so a stale handle value can never re-enter through admission.
    objects_high_water: [i32; 8],
    /// Finite trigger/round profile (8 explicit pairs) riding this Run.
    rounds_dim: Option<super::runtime_objects::Dim>,
    /// Per-pair contact state of the last ADMITTED frame; the enter/stay/exit
    /// oracle carries it forward under an unchanged binding generation.
    round_contact: [bool; super::runtime_round::PAIRS],
    /// Per-pair binding-generation high-water of every ADMITTED frame.
    round_bind_water: [i32; super::runtime_round::PAIRS],
    /// PLAN-004 frozen bound-leaf table of this Run (None outside the sprite
    /// profile). The admitted block is validated against exactly this table.
    sprite_targets: Option<Vec<super::runtime_sprites::Target>>,
    animation_targets: Option<Vec<super::runtime_node_animation::Target>>,
}
impl Play {
    pub fn new(e: &Editor, actors: [Option<u64>; 2], source: &str) -> Result<Self, String> {
        Self::new_program(e, actors, source, None)
    }
    pub fn new_user(
        e: &Editor,
        actors: [Option<u64>; 2],
        snapshot: super::behavior::SourceSnapshot,
    ) -> Result<Self, String> {
        if snapshot.version != super::behavior::VERSION {
            return Err("spatial behavior requires the actual-resource constructor".into());
        }
        Self::new_user_with_spatial(e, actors, snapshot, None)
    }
    /// The actual-resource constructor: the accepted document, its immutable
    /// payloads and the frozen spatial descriptor are the only inputs. Public so
    /// a fresh consumer (and the CPU probe) can drive the same path the Host
    /// uses instead of a test-only copy.
    pub fn new_user_spatial(
        e: &Editor,
        actors: [Option<u64>; 2],
        snapshot: super::behavior::SourceSnapshot,
        descriptor: super::spatial::Descriptor,
    ) -> Result<Self, String> {
        if snapshot.version != super::behavior::VERSION2
            && snapshot.version != super::behavior::VERSION3
            && snapshot.version != super::behavior::VERSION4
            && snapshot.version != super::behavior::VERSION5
            && snapshot.version != super::behavior::CHARACTER_VERSION
            && snapshot.version != super::behavior::CHARACTER3_VERSION
            && snapshot.version != super::behavior::CAMERA2_VERSION
            && snapshot.version != super::behavior::CAMERA3_VERSION
            && snapshot.version != super::behavior::OBJECTS2_VERSION
            && snapshot.version != super::behavior::OBJECTS3_VERSION
            && snapshot.version != super::behavior::TRIGGERS2_VERSION
            && snapshot.version != super::behavior::TRIGGERS3_VERSION
            && snapshot.version != super::behavior::SPRITES2_VERSION
            && snapshot.version != super::behavior::NODE_ANIMATION3_VERSION
        {
            return Err("spatial constructor API mismatch".into());
        }
        descriptor.validate(&e.document)?;
        Self::new_user_with_spatial(e, actors, snapshot, Some(descriptor))
    }
    fn new_user_with_spatial(
        e: &Editor,
        actors: [Option<u64>; 2],
        mut snapshot: super::behavior::SourceSnapshot,
        descriptor: Option<super::spatial::Descriptor>,
    ) -> Result<Self, String> {
        let api = super::behavior::Api::parse(&snapshot.version)?;
        let rebuilt = super::behavior::SourceSnapshot::from_bytes_for(
            &snapshot.path,
            snapshot.bytes.clone(),
            api,
        )?;
        if snapshot.version != rebuilt.version
            || snapshot.source != rebuilt.source
            || snapshot.api_hash != rebuilt.api_hash
            || snapshot.bridge_hash != rebuilt.bridge_hash
            || snapshot.combined_hash != rebuilt.combined_hash
            || snapshot.user_hash != rebuilt.user_hash
            || snapshot.regions != rebuilt.regions
        {
            return Err("behavior source/API/bridge identity drift".into());
        }
        let mut base = serde_json::json!({"document_hash":e.document.hash()?,"assets":e.document.assets,"actors":actors,"stamp":[e.stamp.epoch,e.stamp.revision]});
        if let Some(d) = &descriptor {
            base["spatial_hash"] = d.hash().into();
        }
        if snapshot
            .base_scene
            .as_ref()
            .is_some_and(|prior| *prior != base)
        {
            return Err("behavior base scene/resource identity drift".into());
        }
        snapshot.base_scene = Some(base);
        let source = snapshot.source.clone();
        let mut play = Self::new_program(e, actors, &source, Some(snapshot))?;
        if let Some(descriptor) = descriptor {
            let position = play
                .queue
                .iter()
                .position(|c| c.name == "_ag_start")
                .ok_or("missing start boundary")?;
            let mut configuration = vec![Call::unit(
                "_ag_spatial_reset",
                vec![Scalar::Int(descriptor.rows().len() as i32)],
            )];
            for (index, value) in descriptor.rows().iter().flatten().enumerate() {
                configuration.push(Call::unit(
                    "_ag_spatial_config",
                    vec![Scalar::Int(index as i32), Scalar::Float(*value)],
                ));
            }
            let character = match api {
                super::behavior::Api::CharacterV1
                | super::behavior::Api::Camera2V1
                | super::behavior::Api::Objects2V1
                | super::behavior::Api::Triggers2V1
                | super::behavior::Api::Sprites2V1 => Some(CharProfile::TwoD),
                super::behavior::Api::Character3V1
                | super::behavior::Api::Camera3V1
                | super::behavior::Api::Objects3V1
                | super::behavior::Api::Triggers3V1
                | super::behavior::Api::NodeAnimation3V1 => Some(CharProfile::ThreeD),
                _ => None,
            };
            let camera_dim = match api {
                super::behavior::Api::Camera2V1
                | super::behavior::Api::Objects2V1
                | super::behavior::Api::Triggers2V1
                | super::behavior::Api::Sprites2V1 => Some(CameraDim::TwoD),
                super::behavior::Api::Camera3V1
                | super::behavior::Api::Objects3V1
                | super::behavior::Api::Triggers3V1
                | super::behavior::Api::NodeAnimation3V1 => Some(CameraDim::ThreeD),
                _ => None,
            };
            let objects_dim = api.objects();
            if objects_dim.is_some() {
                if let Some(error) = super::runtime_objects::slot_capacity_error(play.order.len()) {
                    return Err(error);
                }
                // Every accepted node is addressable as a spawn template by its
                // stable document id; eligibility re-checks happen on use.
                for (index, node) in play.order.iter().enumerate() {
                    let stable = i32::try_from(node.id)
                        .map_err(|_| "object template id exceeds 32-bit".to_string())?;
                    configuration.push(Call::unit(
                        "_ag_obj_map",
                        vec![Scalar::Int(stable), Scalar::Int(index as i32)],
                    ));
                }
            }
            if let Some(profile) = character {
                // Persistent document id -> run index table for user binds.
                for (index, node) in play.order.iter().enumerate() {
                    let stable = i32::try_from(node.id)
                        .map_err(|_| "character stable id exceeds 32-bit".to_string())?;
                    configuration.push(Call::unit(
                        profile.map_call(),
                        vec![Scalar::Int(stable), Scalar::Int(index as i32)],
                    ));
                }
            }
            let rounds_dim = api.triggers();
            if rounds_dim.is_some() {
                if let Some(error) = super::runtime_round::pair_capacity_error(play.order.len()) {
                    return Err(error);
                }
                // Every accepted node is addressable as a pair subject/source
                // by its stable document id; eligibility re-checks on use.
                for (index, node) in play.order.iter().enumerate() {
                    let stable = i32::try_from(node.id)
                        .map_err(|_| "trigger stable id exceeds 32-bit".to_string())?;
                    configuration.push(Call::unit(
                        "_ag_trg_map",
                        vec![Scalar::Int(stable), Scalar::Int(index as i32)],
                    ));
                }
            }
            for (offset, call) in configuration.iter().enumerate() {
                play.queue.insert(position + offset, call.clone());
            }
            play.spatial = Some(std::sync::Arc::new(descriptor));
            play.expected_profile = api.profile();
            play.character = character;
            play.camera_dim = camera_dim;
            play.objects_dim = objects_dim;
            play.rounds_dim = rounds_dim;
            play.push_camera_queries();
            play.push_objects_queries();
            play.push_round_queries();
            if api == super::behavior::Api::NodeAnimation3V1 {
                let targets =
                    super::runtime_node_animation::targets(&e.document, &play.order, &e.assets)?;
                let position = play
                    .queue
                    .iter()
                    .position(|c| c.name == "_ag_start")
                    .ok_or("missing start boundary")?;
                for (offset, t) in targets.iter().enumerate() {
                    play.queue.insert(
                        position + offset,
                        Call::unit(
                            "_ag_vis_map",
                            vec![
                                Scalar::Int(
                                    i32::try_from(t.node_id)
                                        .map_err(|_| "visual stable id exceeds 32-bit")?,
                                ),
                                Scalar::Int(t.node_index),
                                Scalar::Int(t.asset_index),
                                Scalar::Int(
                                    t.asset.animation.as_ref().map_or(0, |a| a.clips.len()) as i32,
                                ),
                            ],
                        ),
                    );
                }
                play.animation_targets = Some(targets);
                play.push_animation_queries();
            }
            if api.sprites() {
                // Every bound leaf must still resolve to the accepted payload:
                // the persisted identity is the image SHA, and the frozen table
                // is what the admitted block is checked against.
                let targets = super::runtime_sprites::targets(&e.document, &play.order)?;
                for t in &targets {
                    let payload = e.assets.get(&t.asset).ok_or("missing sprite payload")?;
                    let image = payload.image_data()?;
                    if image.width != t.width || image.height != t.height {
                        return Err("sprite payload dimensions".into());
                    }
                    if !payload
                        .identity
                        .same_content(&e.document.assets[t.asset_index as usize])
                    {
                        return Err("sprite payload identity".into());
                    }
                }
                let position = play
                    .queue
                    .iter()
                    .position(|c| c.name == "_ag_start")
                    .ok_or("missing start boundary")?;
                // The frozen target table is sent as four bounded calls per
                // leaf (the host call profile admits at most four arguments):
                // identity, image size, default origin/width, height/flip.
                let mut sprite_calls = vec![];
                for t in &targets {
                    let stable = i32::try_from(t.node_id)
                        .map_err(|_| "sprite stable id exceeds 32-bit".to_string())?;
                    sprite_calls.push(Call::unit(
                        "_ag_spr_map",
                        vec![
                            Scalar::Int(stable),
                            Scalar::Int(t.node_index),
                            Scalar::Int(t.asset_index),
                        ],
                    ));
                    sprite_calls.push(Call::unit(
                        "_ag_spr_img",
                        vec![
                            Scalar::Int(t.node_index),
                            Scalar::Int(t.width as i32),
                            Scalar::Int(t.height as i32),
                        ],
                    ));
                    sprite_calls.push(Call::unit(
                        "_ag_spr_def",
                        vec![
                            Scalar::Int(t.node_index),
                            Scalar::Int(t.frame[0] as i32),
                            Scalar::Int(t.frame[1] as i32),
                            Scalar::Int(t.frame[2] as i32),
                        ],
                    ));
                    sprite_calls.push(Call::unit(
                        "_ag_spr_defh",
                        vec![
                            Scalar::Int(t.node_index),
                            Scalar::Int(t.frame[3] as i32),
                            Scalar::Int(i32::from(t.flip)),
                        ],
                    ));
                }
                for (offset, call) in sprite_calls.iter().enumerate() {
                    play.queue.insert(position + offset, call.clone());
                }
                play.sprite_targets = Some(targets);
                play.push_sprite_queries();
            }
        }
        Ok(play)
    }
    fn new_program(
        e: &Editor,
        actors: [Option<u64>; 2],
        source: &str,
        user_source: Option<super::behavior::SourceSnapshot>,
    ) -> Result<Self, String> {
        if e.pending() {
            return Err("pending transaction".into());
        }
        e.document.validate()?;
        assets::check_registry(&e.assets)?;
        if e.assets.values().any(|a| a.animation.is_some())
            && user_source
                .as_ref()
                .is_none_or(|s| s.version != super::behavior::NODE_ANIMATION3_VERSION)
        {
            return Err("unsupported: animated payload requires node_animation3d profile".into());
        }
        for a in &e.document.assets {
            let payload = e.assets.get(&a.key()).ok_or("missing play asset")?;
            if !payload.identity.same_content(a) {
                return Err("play asset identity".into());
            }
        }
        for (i, id) in actors.iter().enumerate() {
            if let Some(id) = id {
                if !e
                    .document
                    .objects
                    .iter()
                    .any(|o| o.id == *id && o.kind.is_2d() == (i == 0))
                {
                    return Err("play actor dimension/identity".into());
                }
            }
        }
        let order = hierarchy::ordered(&e.document)?;
        let run_id = RUN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| "run identity exhausted")?;
        let started = Instant::now();
        let vm = if let Some(snapshot) = &user_source {
            Session::new_with_diagnostics(source, |stage, error| {
                snapshot.sdk_diagnostic(stage, error)
            })?
        } else {
            Session::new(source)?
        };
        let compile_us = vm.compile_us;
        let ord = |id: Option<u64>| {
            id.and_then(|id| order.iter().position(|n| n.id == id))
                .map(|v| v as i32)
                .unwrap_or(-1)
        };
        let mut queue = VecDeque::from([Call::unit(
            "reset",
            vec![
                Scalar::Int(run_id),
                Scalar::Int(ord(actors[0])),
                Scalar::Int(ord(actors[1])),
            ],
        )]);
        if user_source.is_some() {
            queue.push_front(Call::unit("main", vec![]));
        }
        for (i, n) in order.iter().enumerate() {
            let parent = n
                .parent
                .map(|p| order.iter().position(|n| n.id == p).unwrap() as i32)
                .unwrap_or(-1);
            queue.push_back(Call::unit(
                "identity",
                vec![
                    Scalar::Int(i as i32 + 1),
                    Scalar::Int(n.kind),
                    Scalar::Int(n.dimension.code()),
                    Scalar::Int(parent),
                ],
            ));
            queue.push_back(Call::unit(
                "visibility",
                vec![Scalar::Int(i32::from(n.visible))],
            ));
            queue.push_back(Call::unit(
                "bind_resource",
                vec![Scalar::Int(super::preview::resource_ordinal(
                    &e.document,
                    n.id,
                )?)],
            ));
            let mut fields = n.local.to_vec();
            fields.extend(static_fields(&e.document, n.id));
            for v in fields {
                queue.push_back(Call::unit("append_field", vec![Scalar::Float(v)]));
            }
        }
        if user_source.is_some() {
            queue.push_back(Call::unit("_ag_start", vec![]));
        }
        queue.push_back(Call::unit("derive", vec![]));
        let expected_profile = if user_source.is_some() { 7 } else { 6 };
        let mut p = Self {
            vm_output_measurement: Default::default(),
            admission_measurement: Default::default(),
            state: State::Starting,
            error: None,
            run_id,
            source_hash: agp::renderer::capture::sha256_hex(source.as_bytes()),
            initial_document_hash: None,
            user_source,
            base_identity: e.isolation_identity(),
            assets: e.assets.clone(),
            last: None,
            input: Input::default(),
            actors,
            compile_us,
            spatial_configuration_calls: 0,
            spatial_configuration_us: 0,
            reports: VecDeque::new(),
            base: e.document.clone(),
            order,
            vm: Some(vm),
            queue,
            read: vec![],
            active: false,
            candidate: None,
            transaction: Some(Instant::now()),
            starting: started,
            next_tick: 0,
            step: false,
            pause_after_start: false,
            last_wall: None,
            debt: 0.0,
            credits: 0,
            action: Action::default(),
            expected_profile,
            spatial: None,
            character: None,
            char: GameInput::default(),
            camera_dim: None,
            camera_frame: None,
            cam_sent: None,
            cam: CameraInput::default(),
            objects_dim: None,
            objects_high_water: [0; 8],
            rounds_dim: None,
            round_contact: [false; super::runtime_round::PAIRS],
            round_bind_water: [0; super::runtime_round::PAIRS],
            sprite_targets: None,
            animation_targets: None,
        };
        p.queries();
        Ok(p)
    }
    pub fn spatial_owner_probe(&self) -> Option<std::sync::Weak<super::spatial::Descriptor>> {
        self.spatial.as_ref().map(std::sync::Arc::downgrade)
    }
    pub fn spatial_scans(&self) -> usize {
        self.spatial.as_ref().map_or(0, |d| d.local_scans)
    }
    pub fn owner_probe(&self) -> Option<Box<dyn Fn() -> bool>> {
        self.vm
            .as_ref()
            .map(|v| Box::new(v.owner_probe()) as Box<dyn Fn() -> bool>)
    }
    pub fn pending(&self) -> bool {
        self.transaction.is_some()
    }
    pub fn candidate(&self) -> Option<&Frame> {
        self.candidate.as_ref()
    }
    pub fn tick(&self) -> i32 {
        self.last.as_ref().map_or(0, |v| v.tick)
    }
    pub fn fail(&mut self, error: String) {
        let error = if let Some(s) = &self.user_source {
            if error.contains(&s.path.display().to_string()) {
                error
            } else {
                s.diagnostic(
                    if error.contains("deadline") || error.contains("budget") {
                        "budget"
                    } else {
                        "output"
                    },
                    if self.next_tick == 0 {
                        "app_start"
                    } else {
                        "app_update"
                    },
                    &error,
                )
            }
        } else {
            error
        };
        self.vm.take();
        self.queue.clear();
        self.read.clear();
        self.candidate = None;
        self.transaction = None;
        self.active = false;
        self.state = State::Faulted;
        self.pause_after_start = false;
        self.error = Some(error);
        self.clear_clock();
    }
    fn clear_clock(&mut self) {
        self.input.clear();
        self.char.clear();
        // cam_sent is transaction-scoped (what the in-flight tick was sent):
        // a mid-transaction Pause/Fault must not decouple it from the echo
        // check, so it survives until the next dispatch overwrites it. The
        // pending input accumulators (cam) do clear with the clock.
        self.cam.clear();
        self.last_wall = None;
        self.debt = 0.0;
        self.credits = 0;
    }
    /// Character runs only: feed one key event into the single GameInput
    /// sampler. Edges accumulate only while `Running`; releases always clear.
    pub fn char_key(&mut self, action: usize, down: bool, repeat: bool) {
        let Some(profile) = self.character else {
            return;
        };
        if action >= profile.actions() {
            return;
        }
        if down && self.state != State::Running {
            // Held state still tracks the physical key; the edge waits for a
            // Running dispatch (contract: no edge accumulation while paused).
            if !repeat && action < profile.actions() {
                self.char.held_track(action);
            }
            return;
        }
        self.char.key(action, down, repeat);
    }
    pub fn character_run(&self) -> bool {
        self.character.is_some()
    }
    pub fn character_profile(&self) -> Option<CharProfile> {
        self.character
    }
    /// Which runtime camera dimension this Run owns, if any.
    pub fn camera_dim(&self) -> Option<CameraDim> {
        self.camera_dim
    }
    pub fn camera_frame(&self) -> Option<&RuntimeCameraFrame> {
        self.camera_frame.as_ref()
    }
    pub fn camera_saturation(&self) -> u32 {
        self.cam.saturated
    }
    /// Feed one drag delta (logical pixels converted to degrees by the host).
    /// Only a Running 3D camera Run accumulates; everything else drops it.
    pub fn cam_drag(&mut self, dyaw: f32, dpitch: f32) {
        if self.state != State::Running || self.camera_dim != Some(CameraDim::ThreeD) {
            return;
        }
        self.cam.active = true;
        self.cam.drag(dyaw, dpitch);
    }
    /// Feed one wheel zoom in notches (Running 3D camera Runs only).
    pub fn cam_zoom(&mut self, notches: f32) {
        if self.state != State::Running || self.camera_dim != Some(CameraDim::ThreeD) {
            return;
        }
        self.cam.active = true;
        self.cam.zoom(notches);
    }
    /// Fold at most one orbit/zoom candidate per dispatch from the fixed
    /// scalar accumulators. A neutral Step keeps the current orbit.
    fn fold_cam_candidate(&mut self) {
        let (Some(dim), Some(last)) = (self.camera_dim, self.camera_frame.as_ref()) else {
            return;
        };
        if dim != CameraDim::ThreeD {
            return;
        }
        let yaw = normalize_yaw_deg(last.yaw + self.cam.pending_yaw);
        let pitch = (last.pitch + self.cam.pending_pitch).clamp(PITCH_MIN_DEG, PITCH_MAX_DEG);
        // powf is not an exact identity at zero, so a no-zoom fold must skip
        // it to keep the echoed candidate bit-identical to the sent orbit.
        let distance = if self.cam.pending_notches == 0.0 {
            last.distance
        } else {
            (last.distance * ZOOM_FACTOR_PER_NOTCH.powf(-self.cam.pending_notches))
                .clamp(DIST_MIN, DIST_MAX)
        };
        self.cam.pending_yaw = 0.0;
        self.cam.pending_pitch = 0.0;
        self.cam.pending_notches = 0.0;
        self.cam_sent = Some([yaw, pitch, distance]);
    }
    /// Re-assert a held key without an edge (Pause cleared the sampler; the
    /// contract's fresh-press rule governs edges, not restored held state).
    pub fn char_held_restore(&mut self, action: usize) {
        if let Some(profile) = self.character {
            if action < profile.actions() {
                self.char.held_track(action);
            }
        }
    }
    pub fn stop(&mut self) {
        self.state = State::Stopping;
        self.pause_after_start = false;
        self.vm.take();
        self.queue.clear();
        self.read.clear();
        self.candidate = None;
        self.transaction = None;
        self.clear_clock();
    }
    pub fn pause(&mut self, why: &str) {
        if self.state == State::Starting {
            // Finish the initial transaction under its 30s budget, then stay paused.
            self.pause_after_start = true;
            self.error = (!why.is_empty()).then(|| why.into());
            self.clear_clock();
            return;
        }
        if matches!(self.state, State::Running | State::Pausing) {
            self.state = if self.pending() {
                State::Pausing
            } else {
                State::Paused
            };
            self.error = (!why.is_empty()).then(|| why.into());
            self.clear_clock();
        }
    }
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state != State::Paused || self.pending() {
            return Err("resume requires idle Paused".into());
        }
        self.state = State::Running;
        self.error = None;
        self.clear_clock();
        Ok(())
    }
    pub fn single_step(&mut self) -> Result<(), String> {
        if self.state != State::Paused || self.pending() {
            return Err("step requires idle Paused".into());
        }
        self.step = true;
        self.char.set_neutral_step();
        self.cam.pending_yaw = 0.0;
        self.cam.pending_pitch = 0.0;
        self.cam.pending_notches = 0.0;
        self.dispatch(Action::default())
    }
    pub fn dispatch(&mut self, a: Action) -> Result<(), String> {
        if self.pending() || (!matches!(self.state, State::Running | State::Paused)) {
            return Err("tick busy/state".into());
        }
        if !matches!(a.space, 0 | 2 | 3) || !(-1..=1).contains(&a.x) || !(-1..=1).contains(&a.y) {
            return Err("action domain".into());
        }
        let Some(next) = self.tick().checked_add(1) else {
            self.fail("tick identity exhausted".into());
            return Err("tick identity exhausted".into());
        };
        self.next_tick = next;
        self.queue.push_back(Call::unit(
            "action",
            vec![
                Scalar::Int(a.space),
                Scalar::Int(a.x),
                Scalar::Int(a.y),
                Scalar::Int(i32::from(a.toggle)),
            ],
        ));
        if let Some(profile) = self.character {
            let (held, pressed) = self.char.fold();
            self.queue.push_back(Call::unit(
                profile.input_call(),
                vec![Scalar::Int(held), Scalar::Int(pressed)],
            ));
        }
        if self.camera_dim == Some(CameraDim::ThreeD) {
            self.fold_cam_candidate();
            if let Some([yaw, pitch, dist]) = self.cam_sent {
                let b = basis_from_yaw(yaw);
                self.queue.push_back(Call::unit(
                    "_ag_cam3_basis",
                    vec![
                        Scalar::Float(b.forward_x),
                        Scalar::Float(b.forward_z),
                        Scalar::Float(b.right_x),
                        Scalar::Float(b.right_z),
                    ],
                ));
                self.queue.push_back(Call::unit(
                    "_ag_cam3_orbit",
                    vec![
                        Scalar::Float(yaw),
                        Scalar::Float(pitch),
                        Scalar::Float(dist),
                    ],
                ));
            }
        }
        self.queue.push_back(Call::unit(
            "tick",
            vec![Scalar::Int(self.next_tick), Scalar::Float(DT)],
        ));
        self.action = a;
        self.read.clear();
        self.transaction = Some(Instant::now());
        self.queries();
        Ok(())
    }
    fn queries(&mut self) {
        if self.user_source.is_some() {
            for name in [
                "_ag_version",
                "_ag_status",
                "_ag_error_node",
                "_ag_error_argument",
            ] {
                self.queue.push_back(Call {
                    name,
                    args: vec![],
                    kind: ResultKind::Int,
                });
            }
        }
        for name in ["profile", "run_id", "tick_id", "count"] {
            self.queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        for i in 0..self.order.len() {
            for name in [
                "id",
                "kind",
                "dimension",
                "parent",
                "resource",
                "local_visible",
                "visible",
            ] {
                self.queue.push_back(Call {
                    name,
                    args: vec![Scalar::Int(i as i32)],
                    kind: ResultKind::Int,
                });
            }
            for f in 0..3 {
                self.queue.push_back(Call {
                    name: "local",
                    args: vec![Scalar::Int(i as i32), Scalar::Int(f)],
                    kind: ResultKind::Float,
                });
            }
            for f in 0..12 {
                self.queue.push_back(Call {
                    name: "field",
                    args: vec![Scalar::Int(i as i32), Scalar::Int(f)],
                    kind: ResultKind::Float,
                });
            }
        }
        self.push_camera_queries();
        self.push_objects_queries();
        self.push_round_queries();
        self.push_sprite_queries();
        self.push_animation_queries();
    }

    /// The camera output block rides every camera-Run tick after the node
    /// queries. `queries()` runs before the API-dependent wiring, so the
    /// spatial constructor appends it once the camera dimension is known.
    fn push_camera_queries(&mut self) {
        match self.camera_dim {
            Some(CameraDim::TwoD) => {
                for name in [
                    "_ag_cam2_status",
                    "_ag_cam2_target",
                    "_ag_cam2_run",
                    "_ag_cam2_tick",
                ] {
                    self.queue.push_back(Call {
                        name,
                        args: vec![],
                        kind: ResultKind::Int,
                    });
                }
                for name in [
                    "_ag_cam2_min_x_v",
                    "_ag_cam2_min_y_v",
                    "_ag_cam2_max_x_v",
                    "_ag_cam2_max_y_v",
                    "_ag_cam2_view_w_v",
                    "_ag_cam2_view_h_v",
                ] {
                    self.queue.push_back(Call {
                        name,
                        args: vec![],
                        kind: ResultKind::Float,
                    });
                }
            }
            Some(CameraDim::ThreeD) => {
                for name in [
                    "_ag_cam3_status",
                    "_ag_cam3_target",
                    "_ag_cam3_run",
                    "_ag_cam3_tick",
                    "_ag_cam3_proj_v",
                ] {
                    self.queue.push_back(Call {
                        name,
                        args: vec![],
                        kind: ResultKind::Int,
                    });
                }
                for name in [
                    "_ag_cam3_yaw_v",
                    "_ag_cam3_pitch_v",
                    "_ag_cam3_dist_v",
                    "_ag_cam3_size_v",
                ] {
                    self.queue.push_back(Call {
                        name,
                        args: vec![],
                        kind: ResultKind::Float,
                    });
                }
            }
            None => {}
        }
    }
    /// Shared round budget, including every query. The host prepares before accept().
    pub fn pump_until(&mut self, deadline: Instant) -> Result<(), String> {
        let measurement_start = self.vm_output_measurement.start();
        let result = self.pump_inner(deadline);
        self.vm_output_measurement.finish(measurement_start);
        if let Err(e) = &result {
            self.fail(e.clone());
            return Err(self.error.clone().unwrap_or_else(|| e.clone()));
        }
        result
    }
    fn pump_inner(&mut self, deadline: Instant) -> Result<(), String> {
        if self.state == State::Faulted || self.candidate.is_some() || !self.pending() {
            return Ok(());
        }
        if self.state == State::Starting {
            if self.starting.elapsed() > Duration::from_secs(30) {
                return Err("play initialization deadline 30s".into());
            }
        } else if self.transaction.unwrap().elapsed() > Duration::from_secs(2) {
            return Err("play whole tick deadline 2s".into());
        }
        while Instant::now() < deadline {
            let Some(c) = self.queue.front() else {
                let start = self.admission_measurement.start();
                let admitted = self.admit();
                self.admission_measurement.finish(start);
                self.candidate = Some(admitted?);
                return Ok(());
            };
            let vm = self.vm.as_mut().ok_or("closed play VM")?;
            let spatial_timing =
                matches!(c.name, "_ag_spatial_reset" | "_ag_spatial_config").then(Instant::now);
            if spatial_timing.is_some() && !self.active {
                self.spatial_configuration_calls += 1;
            }
            let outcome = if self.active {
                vm.pump()
            } else {
                vm.start(
                    if self.user_source.is_some() {
                        super::behavior::transport_call(c.name)
                    } else {
                        c.name
                    },
                    &c.args,
                    c.kind,
                )
            };
            if let Some(start) = spatial_timing {
                self.spatial_configuration_us += start.elapsed().as_micros();
            }
            let outcome = outcome.map_err(|e| {
                self.user_source.as_ref().map_or(e.clone(), |s| {
                    s.diagnostic(
                        if e.contains("budget") {
                            "budget"
                        } else {
                            "runtime"
                        },
                        match c.name {
                            "tick" => "app_update",
                            "_ag_start" => "app_start",
                            name => name,
                        },
                        &e,
                    )
                })
            })?;
            match outcome {
                Progress::Pending => self.active = true,
                Progress::Complete(v) => {
                    if let Some(v) = v {
                        self.read.push(v);
                    }
                    self.queue.pop_front();
                    self.active = false;
                }
            }
        }
        Ok(())
    }
    fn admit(&mut self) -> Result<Frame, String> {
        // Cloned so the objects admission can take &mut self for the
        // generation high-water without aliasing the VM read buffer.
        let read = self.read.clone();
        let data: &[Scalar] = if let Some(snapshot) = &self.user_source {
            match read.as_slice() {
                [Scalar::Int(version), Scalar::Int(0), Scalar::Int(_), Scalar::Int(_), tail @ ..]
                    if *version == super::behavior::Api::parse(&snapshot.version)?.number() =>
                {
                    tail
                }
                [Scalar::Int(_), Scalar::Int(code), Scalar::Int(node), Scalar::Int(arg), ..] => {
                    return Err(snapshot.diagnostic(
                        "output",
                        if self.next_tick == 0 {
                            "app_start"
                        } else {
                            "app_update"
                        },
                        &format!(
                            "API version/error code={code} node={node} argument={arg} {}",
                            if snapshot.version != super::behavior::VERSION {
                                match code {
                                    1 => "InvalidHandle",
                                    2 => "InvalidAxis",
                                    6 => "GroupNotQueryable",
                                    7 => "InvalidParameter",
                                    8 => "SpaceMismatch",
                                    9 => "GeometryUnavailable",
                                    10 => "QueryBudget",
                                    11 => "SpatialConfiguration",
                                    12 => "ArithmeticNonFinite",
                                    13 => {
                                        if snapshot.version == super::behavior::VERSION4 {
                                            "InvalidQueryResult"
                                        } else {
                                            "InvalidResult"
                                        }
                                    }
                                    14 => "ResultIdExhausted",
                                    15 => "InvalidMotionShape",
                                    _ => "APIError",
                                }
                            } else {
                                ""
                            }
                        ),
                    ))
                }
                _ => {
                    return Err(snapshot.diagnostic(
                        "output",
                        "bridge",
                        "invalid API version/error header",
                    ))
                }
            }
        } else {
            read.as_slice()
        };
        let int = |i: usize| match data.get(i) {
            Some(Scalar::Int(v)) => Ok(*v),
            _ => Err("play integer output".to_string()),
        };
        let float = |i: usize| match data.get(i) {
            Some(Scalar::Float(v)) if v.is_finite() => Ok(*v),
            _ => Err("play float output".to_string()),
        };
        let objects_len = if self.objects_dim.is_some() {
            super::runtime_objects::BLOCK_LEN
        } else {
            0
        };
        let rounds_len = if self.rounds_dim.is_some() {
            super::runtime_round::BLOCK_LEN
        } else {
            0
        };
        let sprite_len = self
            .sprite_targets
            .as_ref()
            .map_or(0, |t| super::runtime_sprites::block_len(t.len()));
        if data.len()
            != 4 + 22 * self.order.len()
                + camera_block_len(self.camera_dim)
                + objects_len
                + rounds_len
                + sprite_len
                + self
                    .animation_targets
                    .as_ref()
                    .map_or(0, |t| super::runtime_node_animation::block_len(t.len()))
            || int(0)? != self.expected_profile
            || int(1)? != self.run_id
            || int(2)? != self.next_tick
            || int(3)? != self.order.len() as i32
        {
            return Err("play profile/run/tick/count output".into());
        }
        let mut doc = self.base.clone();
        for (i, n) in self.order.iter().enumerate() {
            let b = 4 + i * 22;
            let parent = n
                .parent
                .map(|id| self.order.iter().position(|n| n.id == id).unwrap() as i32)
                .unwrap_or(-1);
            if [int(b)?, int(b + 1)?, int(b + 2)?, int(b + 3)?, int(b + 4)?]
                != [
                    i as i32 + 1,
                    n.kind,
                    n.dimension.code(),
                    parent,
                    super::preview::resource_ordinal(&self.base, n.id)?,
                ]
            {
                return Err("play identity/kind/dimension/parent/resource output".into());
            }
            let visible = int(b + 5)?;
            if !matches!(visible, 0 | 1) || !matches!(int(b + 6)?, 0 | 1) {
                return Err("play visibility domain".into());
            }
            let translation = [float(b + 7)?, float(b + 8)?, float(b + 9)?];
            if let Some(o) = doc.objects.iter_mut().find(|o| o.id == n.id) {
                o.translation = translation;
                o.visible = visible == 1;
            } else {
                let g = doc.groups.iter_mut().find(|g| g.id == n.id).unwrap();
                g.translation = translation;
                g.visible = visible == 1;
            }
            for (j, v) in static_fields(&self.base, n.id).iter().enumerate() {
                if float(b + 13 + j)? != *v {
                    return Err("play immutable fields output".into());
                }
            }
        }
        doc.validate()?;
        let camera = self.admit_camera(&int, &float)?;
        let instances = self.admit_objects(data)?;
        let expected = hierarchy::derive(&doc)?;
        let mut nodes = BTreeMap::new();
        let mut objects = doc.objects.clone();
        for (i, n) in self.order.iter().enumerate() {
            let b = 4 + i * 22;
            let world = [float(b + 10)?, float(b + 11)?, float(b + 12)?];
            let visible = int(b + 6)? == 1;
            if world != expected[&n.id].translation || visible != expected[&n.id].visible {
                return Err("play world/effective output".into());
            }
            // Consume the actual returned values; expected is only an admission oracle.
            if let Some(o) = objects.iter_mut().find(|o| o.id == n.id) {
                o.translation = world;
                o.visible = visible;
            }
            nodes.insert(
                n.id,
                Derived {
                    translation: world,
                    visible,
                },
            );
        }
        // The rounds block is checked last: its geometry oracle reads the same
        // derive worlds and the same candidate instance table the unit saw.
        let rounds = self.admit_rounds(data, &expected, instances.as_ref())?;
        let sprites = self.admit_sprites(data, instances.as_ref())?;
        let animations = self.admit_animations(data, instances.as_ref())?;
        Ok(Frame {
            document: doc,
            objects,
            nodes,
            tick: self.next_tick,
            run_id: self.run_id,
            camera,
            instances,
            rounds,
            sprites,
            animations,
        })
    }

    /// Validate the runtime-object block that rides every objects-Run tick.
    /// The fixed block, the template eligibility and the legitimate
    /// birth/death/reuse generation sequence are all checked before a
    /// candidate can be accepted; every rejection names its stage.
    fn admit_objects(
        &mut self,
        data: &[Scalar],
    ) -> Result<Option<super::runtime_objects::RuntimeObjectsFrame>, String> {
        let Some(dim) = self.objects_dim else {
            return Ok(None);
        };
        let base = 4 + 22 * self.order.len() + camera_block_len(self.camera_dim);
        let mut block = Vec::with_capacity(super::runtime_objects::BLOCK_LEN);
        for i in 0..super::runtime_objects::BLOCK_LEN {
            match data.get(base + i) {
                Some(v @ (Scalar::Int(_) | Scalar::Float(_))) => block.push(*v),
                _ => return Err("objects block output".into()),
            }
        }
        let order = &self.order;
        let rules = super::runtime_objects::TemplateRules { order, dim };
        let previous = self
            .last
            .as_ref()
            .and_then(|f| f.instances.as_ref())
            .cloned();
        let mut high_water = self.objects_high_water;
        let frame = super::runtime_objects::admit_block(
            &block,
            self.run_id,
            self.next_tick,
            &rules,
            previous.as_ref(),
            &mut high_water,
        )?;
        self.objects_high_water = high_water;
        Ok(Some(frame))
    }

    /// Validate the camera output block that rides every camera-Run tick.
    /// Tick 0 admits the frozen configuration; later ticks must echo the
    /// exact host-sent candidate (3D) and the unchanged configuration (2D).
    /// Every rejection names its stage and never falls back silently.
    fn admit_camera(
        &self,
        int: &impl Fn(usize) -> Result<i32, String>,
        float: &impl Fn(usize) -> Result<f32, String>,
    ) -> Result<Option<RuntimeCameraFrame>, String> {
        let Some(dim) = self.camera_dim else {
            return Ok(None);
        };
        let b = 4 + 22 * self.order.len();
        let fail = |name: &str| -> Result<Option<RuntimeCameraFrame>, String> {
            Err(format!("camera {name} output"))
        };
        let status = int(b).map_err(|_| format!("camera status output"))?;
        if status != 1 {
            return fail(if self.next_tick == 0 {
                "configuration missing"
            } else {
                "status"
            });
        }
        let target = int(b + 1).map_err(|_| format!("camera target output"))?;
        if target < 0 || target >= self.order.len() as i32 {
            return fail("target");
        }
        if int(b + 2).map_err(|_| format!("camera run output"))? != self.run_id
            || int(b + 3).map_err(|_| format!("camera tick output"))? != self.next_tick
        {
            return fail("run/tick");
        }
        let camera = match dim {
            CameraDim::TwoD => {
                let rect = [
                    float(b + 4).map_err(|_| format!("camera rect output"))?,
                    float(b + 5).map_err(|_| format!("camera rect output"))?,
                    float(b + 6).map_err(|_| format!("camera rect output"))?,
                    float(b + 7).map_err(|_| format!("camera rect output"))?,
                ];
                let view = [
                    float(b + 8).map_err(|_| format!("camera view output"))?,
                    float(b + 9).map_err(|_| format!("camera view output"))?,
                ];
                if rect[0] >= rect[2]
                    || rect[1] >= rect[3]
                    || view[0] <= 0.0
                    || view[1] <= 0.0
                    || view[0] > rect[2] - rect[0]
                    || view[1] > rect[3] - rect[1]
                {
                    return fail("configuration domain");
                }
                RuntimeCameraFrame {
                    dim,
                    target,
                    target_id: self.order[target as usize].id,
                    run_id: self.run_id,
                    tick: self.next_tick,
                    rect,
                    view,
                    yaw: 0.0,
                    pitch: 0.0,
                    distance: 0.0,
                    projection: 0,
                    ortho_size: 0.0,
                }
            }
            CameraDim::ThreeD => {
                let projection = int(b + 4).map_err(|_| format!("camera projection output"))?;
                let yaw = float(b + 5).map_err(|_| format!("camera orbit output"))?;
                let pitch = float(b + 6).map_err(|_| format!("camera orbit output"))?;
                let distance = float(b + 7).map_err(|_| format!("camera orbit output"))?;
                let ortho_size = float(b + 8).map_err(|_| format!("camera projection output"))?;
                if !matches!(projection, 0 | 1)
                    || !(0.0..360.0).contains(&yaw)
                    || !(PITCH_MIN_DEG..=PITCH_MAX_DEG).contains(&pitch)
                    || !(DIST_MIN..=DIST_MAX).contains(&distance)
                    || (projection == 1 && !(ORTHO_SIZE_MIN..=ORTHO_SIZE_MAX).contains(&ortho_size))
                {
                    return fail("orbit/projection domain");
                }
                let camera = RuntimeCameraFrame {
                    dim,
                    target,
                    target_id: self.order[target as usize].id,
                    run_id: self.run_id,
                    tick: self.next_tick,
                    rect: [0.0; 4],
                    view: [0.0; 2],
                    yaw,
                    pitch,
                    distance,
                    projection,
                    ortho_size,
                };
                if self.next_tick == 0 {
                    if yaw != 0.0 {
                        return fail("orbit initial");
                    }
                } else if self.cam_sent != Some([yaw, pitch, distance]) {
                    let sent = self.cam_sent.map_or("none".into(), |s| format!("{s:?}"));
                    return Err(format!(
                        "camera orbit echo mismatch: sent {sent} got [{yaw} {pitch} {distance}]"
                    ));
                }
                camera
            }
        };
        if self.next_tick == 0 {
            if let Some(last) = &self.camera_frame {
                if last.rect != camera.rect || last.view != camera.view {
                    return fail("configuration drift");
                }
            }
        } else if let Some(last) = &self.camera_frame {
            if last.rect != camera.rect
                || last.view != camera.view
                || last.projection != camera.projection
                || last.ortho_size != camera.ortho_size
                || last.target != camera.target
            {
                return fail("configuration drift");
            }
        }
        Ok(Some(camera))
    }
    /// The runtime-object block rides every objects-Run tick after the camera
    /// block: 4 header ints plus 8 canonical rows of 6 values, in slot order.
    fn push_objects_queries(&mut self) {
        if self.objects_dim.is_none() {
            return;
        }
        for name in [
            "_ag_obj_status",
            "_ag_obj_count",
            "_ag_obj_run",
            "_ag_obj_tick",
        ] {
            self.queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        for slot in 0..super::runtime_objects::SLOTS as i32 {
            for (name, kind) in [
                ("_ag_obj_gv", ResultKind::Int),
                ("_ag_obj_tv", ResultKind::Int),
                ("_ag_obj_xv", ResultKind::Float),
                ("_ag_obj_yv", ResultKind::Float),
                ("_ag_obj_zv", ResultKind::Float),
                ("_ag_obj_vv", ResultKind::Int),
            ] {
                self.queue.push_back(Call {
                    name,
                    args: vec![Scalar::Int(slot)],
                    kind,
                });
            }
        }
    }
    /// Validate the trigger/round block that rides every triggers-Run tick
    /// after the objects block. The independent geometry oracle recomputes
    /// every pair from the derive worlds, the descriptor rows and the
    /// candidate instances, so events must match enter/stay/exit exactly.
    fn admit_rounds(
        &mut self,
        data: &[Scalar],
        expected: &BTreeMap<u64, Derived>,
        instances: Option<&super::runtime_objects::RuntimeObjectsFrame>,
    ) -> Result<Option<super::runtime_round::RuntimeRoundFrame>, String> {
        let Some(dim) = self.rounds_dim else {
            return Ok(None);
        };
        let descriptor = self.spatial.as_ref().ok_or("rounds missing descriptor")?;
        let base = 4
            + 22 * self.order.len()
            + camera_block_len(self.camera_dim)
            + super::runtime_objects::BLOCK_LEN;
        let mut block = Vec::with_capacity(super::runtime_round::BLOCK_LEN);
        for i in 0..super::runtime_round::BLOCK_LEN {
            match data.get(base + i) {
                Some(v @ (Scalar::Int(_) | Scalar::Float(_))) => block.push(*v),
                _ => return Err("rounds block output".into()),
            }
        }
        let worlds: Vec<[f32; 3]> = self
            .order
            .iter()
            .map(|n| expected[&n.id].translation)
            .collect();
        let rules = super::runtime_round::RoundRules {
            order: &self.order,
            rows: &descriptor.rows(),
            worlds: &worlds,
            dim,
        };
        let previous = self.last.as_ref().and_then(|f| f.rounds.as_ref()).cloned();
        let mut contact = self.round_contact;
        let mut bind_water = self.round_bind_water;
        let frame = super::runtime_round::admit_round_block(
            &block,
            self.run_id,
            self.next_tick,
            &rules,
            instances,
            previous.as_ref(),
            &mut contact,
            &mut bind_water,
        )?;
        self.round_contact = contact;
        self.round_bind_water = bind_water;
        Ok(Some(frame))
    }

    /// The trigger/round block rides every triggers-Run tick after the objects
    /// block: 8 header ints, 8 binding rows of 9 values and 8 event rows of 8
    /// values, in pair order.
    fn push_round_queries(&mut self) {
        if self.rounds_dim.is_none() {
            return;
        }
        for name in [
            "_ag_trg_status",
            "_ag_trg_run",
            "_ag_trg_tick",
            "_ag_trg_count_v",
            "_ag_trg_req_v",
            "_ag_trg_req_tick_v",
            "_ag_trg_seq_v",
            "_ag_trg_zero",
        ] {
            self.queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        for pair in 0..super::runtime_round::PAIRS as i32 {
            for name in [
                "_ag_trg_b0",
                "_ag_trg_b1",
                "_ag_trg_b2",
                "_ag_trg_b3",
                "_ag_trg_b4",
                "_ag_trg_b5",
                "_ag_trg_b6",
                "_ag_trg_b7",
            ] {
                self.queue.push_back(Call {
                    name,
                    args: vec![Scalar::Int(pair)],
                    kind: ResultKind::Int,
                });
            }
            self.queue.push_back(Call {
                name: "_ag_trg_b8",
                args: vec![Scalar::Int(pair)],
                kind: ResultKind::Float,
            });
        }
        for row in 0..super::runtime_round::PAIRS as i32 {
            for name in [
                "_ag_trg_e0",
                "_ag_trg_e1",
                "_ag_trg_e2",
                "_ag_trg_e3",
                "_ag_trg_e4",
                "_ag_trg_e5",
                "_ag_trg_e6",
                "_ag_trg_e7",
            ] {
                self.queue.push_back(Call {
                    name,
                    args: vec![Scalar::Int(row)],
                    kind: ResultKind::Int,
                });
            }
        }
    }
    /// The sprite block rides every sprite-Run tick after the rounds block: 4
    /// header ints, then a fixed table of 8 bound rows and 8 slot rows of 7
    /// values each, in frozen target and slot order. Rows without a bound leaf
    /// are canonical zeros, so the layout never depends on the document again.
    fn push_sprite_queries(&mut self) {
        if self.sprite_targets.is_none() {
            return;
        }
        for name in [
            "_ag_spr_status",
            "_ag_spr_count",
            "_ag_spr_run",
            "_ag_spr_tick",
        ] {
            self.queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        let bound = self.sprite_targets.as_ref().map_or(0, Vec::len);
        for row in 0..bound as i32 {
            for f in 0..super::runtime_sprites::ROW_FIELDS as i32 {
                self.queue.push_back(Call {
                    name: "_ag_spr_pv",
                    args: vec![Scalar::Int(row), Scalar::Int(f)],
                    kind: ResultKind::Int,
                });
            }
        }
        for row in 0..super::runtime_objects::SLOTS as i32 {
            for f in 0..super::runtime_sprites::ROW_FIELDS as i32 {
                self.queue.push_back(Call {
                    name: "_ag_spr_qv",
                    args: vec![Scalar::Int(row), Scalar::Int(f)],
                    kind: ResultKind::Int,
                });
            }
        }
    }
    /// Validate the PLAN-004 sprite block that rides every sprite-Run tick
    /// after the rounds block. Identity, frame bounds, run/tick and slot
    /// generation are re-derived here from the frozen target table and the
    /// accepted instance table, never from the output itself.
    fn admit_sprites(
        &mut self,
        data: &[crate::Scalar],
        instances: Option<&super::runtime_objects::RuntimeObjectsFrame>,
    ) -> Result<Option<super::runtime_sprites::SpriteFrame>, String> {
        let Some(targets) = self.sprite_targets.as_ref() else {
            return Ok(None);
        };
        let base = 4
            + 22 * self.order.len()
            + camera_block_len(self.camera_dim)
            + super::runtime_objects::BLOCK_LEN
            + super::runtime_round::BLOCK_LEN;
        let expected = super::runtime_sprites::block_len(targets.len());
        let mut block = Vec::with_capacity(expected);
        for i in 0..expected {
            match data.get(base + i) {
                Some(v @ (Scalar::Int(_) | Scalar::Float(_))) => block.push(*v),
                _ => return Err("sprite block output".into()),
            }
        }
        let assets = self.assets.clone();
        let dims = |key: &str| -> Option<(u32, u32)> {
            let asset = assets.get(key)?;
            let image = asset.image.as_ref()?;
            Some((image.width, image.height))
        };
        let targets = targets.clone();
        super::runtime_sprites::admit_block(
            &block,
            self.run_id,
            self.next_tick,
            &targets,
            &dims,
            instances,
        )
        .map(Some)
    }
    fn push_animation_queries(&mut self) {
        let Some(targets) = self.animation_targets.as_ref() else {
            return;
        };
        let count = targets.len();
        for name in [
            "_ag_vis_status",
            "_ag_vis_count",
            "_ag_vis_run",
            "_ag_vis_tick",
        ] {
            self.queue.push_back(Call {
                name,
                args: vec![],
                kind: ResultKind::Int,
            });
        }
        for (ints, floats, rows) in [
            ("_ag_vis_pi", "_ag_vis_pf", count),
            ("_ag_vis_qi", "_ag_vis_qf", 8),
        ] {
            for row in 0..rows as i32 {
                for f in 0..4 {
                    self.queue.push_back(Call {
                        name: ints,
                        args: vec![Scalar::Int(row), Scalar::Int(f)],
                        kind: ResultKind::Int,
                    });
                }
                for f in 0..2 {
                    self.queue.push_back(Call {
                        name: floats,
                        args: vec![Scalar::Int(row), Scalar::Int(f)],
                        kind: ResultKind::Float,
                    });
                }
            }
        }
    }
    fn admit_animations(
        &self,
        data: &[Scalar],
        instances: Option<&super::runtime_objects::RuntimeObjectsFrame>,
    ) -> Result<Option<super::runtime_node_animation::AnimationFrame>, String> {
        let Some(targets) = self.animation_targets.as_ref() else {
            return Ok(None);
        };
        let base = 4
            + 22 * self.order.len()
            + camera_block_len(self.camera_dim)
            + super::runtime_objects::BLOCK_LEN
            + super::runtime_round::BLOCK_LEN;
        super::runtime_node_animation::admit_block(
            data.get(base..).ok_or("visual block output")?,
            self.run_id,
            self.next_tick,
            targets,
            instances,
        )
        .map(Some)
    }
    pub fn accept(&mut self) -> Result<(), String> {
        let frame = self.candidate.take().ok_or("no complete play frame")?;
        let duration = self
            .transaction
            .take()
            .ok_or("no play transaction")?
            .elapsed();
        if (self.state == State::Starting && self.starting.elapsed() > Duration::from_secs(30))
            || (self.state != State::Starting && duration > Duration::from_secs(2))
        {
            self.fail("play preparation deadline".into());
            return Err("play preparation deadline".into());
        }
        let a = self.action;
        self.reports.push_back(TickReport {
            tick: frame.tick,
            space: a.space,
            x: a.x,
            y: a.y,
            toggle: a.toggle,
            elapsed_us: duration.as_micros(),
        });
        if self.reports.len() > 4096 {
            self.reports.pop_front();
        }
        if frame.tick == 0 {
            self.initial_document_hash = Some(frame.document.hash()?);
        }
        self.camera_frame = frame.camera.clone();
        self.last = Some(frame);
        if self.state == State::Starting {
            self.state = if std::mem::take(&mut self.pause_after_start) {
                State::Paused
            } else {
                State::Running
            };
            self.clear_clock();
        }
        if self.state == State::Pausing || self.step {
            self.state = State::Paused;
            self.step = false;
            self.clear_clock();
        }
        Ok(())
    }
    /// Credit count is bounded; completed output is never coalesced or skipped.
    pub fn clock(&mut self, now: Instant) {
        if self.state != State::Running {
            return;
        }
        let Some(last) = self.last_wall.replace(now) else {
            return;
        };
        let gap = now.saturating_duration_since(last);
        if gap > Duration::from_millis(250) {
            self.pause("auto-Pause: display gap >250ms");
            return;
        }
        self.debt += gap.as_secs_f64();
        let ticks = (self.debt / (1.0 / 60.0)).floor() as u64;
        if ticks + u64::from(self.credits) > 4 {
            self.pause("auto-Pause: tick backlog >4");
            return;
        }
        self.debt -= ticks as f64 * (1.0 / 60.0);
        self.credits += ticks as u8;
    }
    pub fn dispatch_credit(&mut self) -> Result<bool, String> {
        if self.state != State::Running || self.pending() || self.credits == 0 {
            return Ok(false);
        }
        self.credits -= 1;
        let a = self.input.sample();
        if let Err(e) = self.dispatch(a) {
            self.fail(e.clone());
            return Err(e);
        }
        Ok(true)
    }
}
fn static_fields(d: &Document, id: u64) -> Vec<f32> {
    if let Some(o) = d.objects.iter().find(|o| o.id == id) {
        o.scale
            .iter()
            .copied()
            .chain([o.yaw, o.pitch])
            .chain(o.rgba)
            .collect()
    } else {
        vec![1.0, 1.0, 1.0, 0.0, 0.0, 0.9, 0.3, 0.2, 1.0]
    }
}

#[cfg(test)]
mod deadline_tests {
    use super::*;
    #[test]
    fn user_profile_rejects_actual_wrong_profile_stale_run_and_resource_output() {
        let e = Editor::new(Document::sample()).unwrap();
        let snapshot = super::super::behavior::SourceSnapshot::from_bytes(
            std::path::Path::new("D:/user/main.at"),
            b"fn app_start(){}\nfn app_update(dt float){}".to_vec(),
        )
        .unwrap();
        // Deliberately bypass source admission here to exercise the actual VM
        // output consumer against a corrupt managed producer.
        for (from, to) in [
            ("return 7", "return 6"),
            ("return _ag_run", "return _ag_run-1"),
            ("return _ag_resources[i]", "return -50"),
        ] {
            assert!(snapshot.source.contains(from), "{from}");
            let source = snapshot.source.replace(from, to);
            let mut p =
                Play::new_program(&e, [None, None], &source, Some(snapshot.clone())).unwrap();
            let owner = p.owner_probe().unwrap();
            let end = Instant::now() + Duration::from_secs(31);
            let error = loop {
                assert!(Instant::now() < end);
                match p.pump_until(Instant::now() + Duration::from_millis(8)) {
                    Err(e) => break e,
                    Ok(_) => {
                        if p.candidate().is_some() {
                            break p.accept().unwrap_err();
                        }
                    }
                }
            };
            assert!(
                error.contains("profile/run/tick/count") || error.contains("resource"),
                "{error}"
            );
            assert_eq!(p.state, State::Faulted);
            assert!(!owner());
            assert!(p.last.is_none());
        }
    }
    fn ready(p: &mut Play) {
        while p.candidate().is_none() {
            p.pump_until(Instant::now() + Duration::from_millis(8))
                .unwrap();
        }
        p.accept().unwrap();
    }
    #[test]
    fn whole_tick_start_and_preparation_deadlines_close_actual_vm() {
        let e = Editor::new(Document::sample()).unwrap();
        let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
        let alive = p.owner_probe().unwrap();
        p.starting = Instant::now() - Duration::from_secs(31);
        assert!(p
            .pump_until(Instant::now() + Duration::from_millis(8))
            .unwrap_err()
            .contains("30s"));
        assert!(!alive());
        assert!(p.last.is_none());
        let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
        ready(&mut p);
        let alive = p.owner_probe().unwrap();
        p.dispatch(Action::default()).unwrap();
        p.transaction = Some(Instant::now() - Duration::from_secs(3));
        assert!(p
            .pump_until(Instant::now() + Duration::from_millis(8))
            .unwrap_err()
            .contains("2s"));
        assert_eq!(p.tick(), 0);
        assert!(!alive());
        let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
        ready(&mut p);
        p.dispatch(Action::default()).unwrap();
        while p.candidate().is_none() {
            p.pump_until(Instant::now() + Duration::from_millis(8))
                .unwrap();
        }
        p.transaction = Some(Instant::now() - Duration::from_secs(3));
        assert!(p.accept().is_err());
        assert_eq!(p.tick(), 0);
        assert_eq!(p.state, State::Faulted);
        let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
        ready(&mut p);
        p.last.as_mut().unwrap().tick = i32::MAX;
        let alive = p.owner_probe().unwrap();
        assert!(p
            .dispatch(Action::default())
            .unwrap_err()
            .contains("exhausted"));
        assert_eq!(p.state, State::Faulted);
        assert!(!alive());
    }

    #[test]
    fn canceled_initialization_keeps_start_budget_then_neutral_step() {
        let e = Editor::new(Document::sample()).unwrap();
        let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
        let alive = p.owner_probe().unwrap();
        p.pause("auto-Pause: focus lost");
        p.starting = Instant::now() - Duration::from_secs(3);
        p.transaction = Some(Instant::now() - Duration::from_secs(3));
        assert_eq!(p.state, State::Starting);
        assert!(p.resume().is_err());
        ready(&mut p);
        assert_eq!(p.state, State::Paused);
        assert_eq!(p.tick(), 0);
        assert_eq!(p.last.as_ref().unwrap().document, e.document);
        assert!(alive());
        assert!(p.error.as_ref().unwrap().contains("focus lost"));
        assert!(!p.pause_after_start);
        p.single_step().unwrap();
        ready(&mut p);
        assert_eq!(p.state, State::Paused);
        assert_eq!(p.tick(), 1);
        assert_eq!(p.last.as_ref().unwrap().document, e.document);
        // The startup latch cannot grant the initialization budget to later runtime ticks.
        p.single_step().unwrap();
        p.transaction = Some(Instant::now() - Duration::from_secs(3));
        assert!(p
            .pump_until(Instant::now() + Duration::from_millis(8))
            .unwrap_err()
            .contains("2s"));
        assert_eq!(p.state, State::Faulted);
        assert_eq!(p.tick(), 1);
        assert!(!alive());
    }

    #[test]
    fn canceled_initialization_deadlines_close_vm_before_or_after_preparation() {
        let e = Editor::new(Document::sample()).unwrap();
        for after_output in [false, true] {
            let mut p = Play::new(&e, [None, None], SOURCE).unwrap();
            let alive = p.owner_probe().unwrap();
            p.pause("auto-Pause: window occluded");
            if after_output {
                while p.candidate().is_none() {
                    p.pump_until(Instant::now() + Duration::from_millis(8))
                        .unwrap();
                }
            }
            p.starting = Instant::now() - Duration::from_secs(31);
            let error = if after_output {
                p.accept().unwrap_err()
            } else {
                p.pump_until(Instant::now() + Duration::from_millis(8))
                    .unwrap_err()
            };
            assert!(error.contains(if after_output {
                "preparation deadline"
            } else {
                "30s"
            }));
            assert_eq!(p.state, State::Faulted);
            assert!(p.last.is_none());
            assert!(!p.pending());
            assert!(!p.pause_after_start);
            assert!(!alive());
        }
    }
}

#[cfg(test)]
mod spatial_admission_tests {
    use super::super::{
        behavior::{Api, SourceSnapshot},
        document::Kind,
        spatial::Descriptor,
    };
    use super::*;
    use std::path::Path;
    #[test]
    fn incomplete_duplicate_out_of_order_and_foreign_config_never_publish() {
        for api in [Api::SpatialV2, Api::SegmentV3] {
            check_spatial_configuration(api);
        }
    }
    fn check_spatial_configuration(api: Api) {
        let mut d = Document::empty("config".into());
        d.objects.push(Object::new(1, Kind::Circle));
        let e = Editor::new(d).unwrap();
        let src = || {
            SourceSnapshot::from_bytes_for(
                Path::new("config.at"),
                b"var starts=0\nfn start_count() int {return starts}\nfn app_start(){starts=starts+1}\nfn app_update(dt float){}\n".to_vec(),
                api,
            )
            .unwrap()
        };
        for mode in ["missing", "duplicate", "out-of-order"] {
            let descriptor = Descriptor::build(&e.document, |_| None).unwrap();
            let mut p = Play::new_user_spatial(&e, [Some(1), None], src(), descriptor).unwrap();
            let owner = p.owner_probe().unwrap();
            let n = p
                .queue
                .iter()
                .position(|c| c.name == "_ag_spatial_config")
                .unwrap();
            match mode {
                "missing" => {
                    p.queue.remove(n);
                }
                "duplicate" => {
                    let args = p.queue[n].args.clone();
                    p.queue.insert(n, Call::unit("_ag_spatial_config", args));
                }
                _ => p.queue[n].args[0] = Scalar::Int(1),
            }
            let deadline = Instant::now() + Duration::from_secs(31);
            let error = loop {
                assert!(Instant::now() < deadline);
                match p.pump_inner(Instant::now() + Duration::from_millis(8)) {
                    Err(e) => break e,
                    Ok(()) => assert!(
                        p.candidate().is_none(),
                        "partial initialized frame published"
                    ),
                }
            };
            assert!(error.contains("SpatialConfiguration"), "{mode}: {error}");
            assert!(p.last.is_none());
            assert_eq!(
                p.vm.as_mut().unwrap().int("start_count", &[]).unwrap(),
                0,
                "invalid spatial config must not execute app_start"
            );
            p.fail(error);
            assert!(!owner());
        }
        let descriptor = Descriptor::build(&e.document, |_| None).unwrap();
        let mut valid = Play::new_user_spatial(&e, [Some(1), None], src(), descriptor).unwrap();
        while valid.candidate().is_none() {
            valid
                .pump_until(Instant::now() + Duration::from_millis(8))
                .unwrap();
        }
        assert_eq!(
            valid.vm.as_mut().unwrap().int("start_count", &[]).unwrap(),
            1,
            "valid configuration must execute app_start exactly once"
        );
        valid.accept().unwrap();
        valid.stop();
        let mut foreign = e.document.clone();
        foreign.objects[0].scale[0] = 1.;
        let descriptor = Descriptor::build(&foreign, |_| None).unwrap();
        assert!(Play::new_user_spatial(&e, [Some(1), None], src(), descriptor).is_err());
    }
}

#[cfg(test)]
mod triggers_admission_tests {
    use super::super::{
        behavior::{Api, SourceSnapshot},
        document::Kind,
    };
    use super::*;
    use std::path::Path;

    fn triggers_document() -> super::super::document::Document {
        let mut d = super::super::document::Document::empty("triggers-admit".into());
        let mut character = super::super::document::Object::new(1, Kind::Quad);
        character.translation = [2.0, 1.0, 0.0];
        character.scale = [0.8, 1.0, 1.0];
        d.objects.push(character);
        let mut coin = super::super::document::Object::new(3, Kind::Circle);
        coin.translation = [6.0, 2.0, 0.0];
        coin.scale = [0.6, 0.6, 1.0];
        coin.visible = false;
        d.objects.push(coin);
        d.validate().unwrap();
        d
    }

    fn triggers_source() -> String {
        "var ok=0
var h=0
fn app_start() {
 ag_set_local(0,1,0.5)
 ok=ok+ag_char_bind(1)
 ok=ok+ag_camera_follow_2d(1, 0.0, -4.0, 30.0, 12.0, 12.0, 8.0)
 h=ag_object_spawn(0, 3, 6.0, 1.0, 0.0)
 ok=ok+ag_trigger_bind(0, 1, 1, h, 0, 0.0)
}
fn app_update(dt float) {
 ok=ok+ag_char_move(0.05,0.0,0.002)
}
"
        .into()
    }

    fn ready_triggers_play(
        mutant: impl FnOnce(&mut VecDeque<Call>),
    ) -> (Play, Box<dyn Fn() -> bool>) {
        let e = Editor::new(triggers_document()).unwrap();
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("triggers-admit.at"),
            triggers_source().into_bytes(),
            Api::Triggers2V1,
        )
        .unwrap();
        let descriptor = super::super::spatial::Descriptor::build(&e.document, |_| None).unwrap();
        let mut p = Play::new_user_spatial(&e, [Some(1), None], snap, descriptor).unwrap();
        let end = Instant::now() + Duration::from_secs(31);
        while p.candidate().is_none() {
            assert!(Instant::now() < end);
            p.pump_until(Instant::now() + Duration::from_millis(8))
                .unwrap();
        }
        p.accept().unwrap();
        let owner = p.owner_probe().unwrap();
        p.dispatch(Action::default()).unwrap();
        mutant(&mut p.queue);
        let end = Instant::now() + Duration::from_secs(31);
        loop {
            assert!(Instant::now() < end);
            match p.pump_until(Instant::now() + Duration::from_millis(8)) {
                Err(e) => {
                    p.fail(e);
                    break;
                }
                Ok(()) => {
                    if p.candidate().is_some() {
                        let error = p.accept().unwrap_err();
                        p.fail(error);
                        break;
                    }
                }
            }
        }
        (p, owner)
    }

    #[test]
    fn corrupted_rounds_blocks_fault_the_named_stage_and_close_the_vm() {
        // A duplicated binding-row emitter breaks the frozen output layout.
        let (p, owner) = ready_triggers_play(|queue| {
            let index = queue
                .iter()
                .position(|c| c.name == "_ag_trg_b0")
                .expect("rounds query present");
            let duplicate = queue[index].clone();
            queue.insert(index + 1, duplicate);
        });
        assert!(p
            .error
            .as_ref()
            .expect("fault")
            .contains("play profile/run/tick/count"));
        assert_eq!(p.state, State::Faulted);
        assert!(!owner());

        // A shifted row argument misattributes the pair: the emitted binding
        // row no longer matches the admitted frame identity.
        let (p, owner) = ready_triggers_play(|queue| {
            let index = queue
                .iter()
                .position(|c| c.name == "_ag_trg_b0")
                .expect("rounds query present");
            queue[index].args = vec![Scalar::Int(1)];
        });
        let error = p.error.as_ref().expect("fault").clone();
        assert!(error.contains("rounds"), "{error}");
        assert_eq!(p.state, State::Faulted);
        assert!(!owner());
    }
}
