//! Spatial native acceptance: OS controls and an independent input/geometry oracle.
use autographics_auto_driver::authoring::{
    behavior::Api,
    document::Object,
    host::Host,
    play::{State, DT},
};
use glam::{Mat4, Vec3};
use serde_json::{json, Value};
pub struct Probe {
    pub phase: u8,
    pub done: bool,
    pub error: Option<String>,
    pub records: Vec<Value>,
    pub update: bool,
    pub segment: bool,
    pub sweep: bool,
    pub motion: bool,
    update_stage: u8,
    old_source: Option<String>,
    at: u64,
    space: i32,
    narrow: bool,
    fault: Option<String>,
    before: Option<Value>,
    images: Option<[String; 2]>,
    tick: i32,
}
impl Probe {
    pub fn new(space: i32, narrow: bool, fault: Option<String>) -> Self {
        Self {
            phase: 0,
            done: false,
            error: None,
            records: vec![],
            update: false,
            segment: false,
            sweep: false,
            motion: false,
            update_stage: 0,
            old_source: None,
            at: 0,
            space,
            narrow,
            fault,
            before: None,
            images: None,
            tick: 0,
        }
    }
    fn check(b: bool, s: &str) -> Result<(), String> {
        if b {
            Ok(())
        } else {
            Err(s.into())
        }
    }
    fn click(h: &mut Host, id: &str) -> Result<bool, String> {
        let b = h
            .widget_bounds(id)
            .ok_or_else(|| format!("spatial missing widget {id}"))?;
        if b.y < 0. || b.y + b.height >= 800. {
            super::native_interaction::Probe::scroll(h, b.y > 0.)?;
            return Ok(false);
        }
        super::native_interaction::Probe::click(
            h,
            [810. + b.x + b.width * 0.5, b.y + b.height * 0.5],
        )?;
        Ok(true)
    }
    fn images(h: &mut Host) -> Result<[String; 2], String> {
        Ok([
            h.rt.capture(h.two, "spatial-two").ok_or("2d capture")?.0,
            h.rt.capture(h.three, "spatial-three")
                .ok_or("3d capture")?
                .0,
        ])
    }
    // Independent all-mesh local box and eight-corner reference. Never calls spatial.rs.
    pub(super) fn offsets(h: &Host, o: &Object) -> Result<([f32; 3], [f32; 3]), String> {
        if o.kind.is_2d() {
            return Ok((
                [-o.scale[0] * 0.5, -o.scale[1] * 0.5, 0.],
                [o.scale[0] * 0.5, o.scale[1] * 0.5, 0.],
            ));
        }
        let asset = h
            .editor
            .assets
            .get(o.resource.as_ref().ok_or("reference needs GLB")?)
            .ok_or("reference asset")?;
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for p in asset.model.meshes.iter().flat_map(|m| &m.positions) {
            lo = lo.min(Vec3::from(*p));
            hi = hi.max(Vec3::from(*p));
        }
        let t = Mat4::from_rotation_y(o.yaw)
            * Mat4::from_rotation_x(o.pitch)
            * Mat4::from_scale(o.scale.into());
        let mut a = Vec3::splat(f32::INFINITY);
        let mut b = Vec3::splat(f32::NEG_INFINITY);
        for x in [lo.x, hi.x] {
            for y in [lo.y, hi.y] {
                for z in [lo.z, hi.z] {
                    let p = t.transform_point3(Vec3::new(x, y, z));
                    a = a.min(p);
                    b = b.max(p);
                }
            }
        }
        Ok((a.into(), b.into()))
    }
    fn trace(&mut self, h: &Host) -> Result<(), String> {
        if self.motion {
            return super::native_motion::trace(h, self.space, self.narrow, &mut self.records);
        }
        if self.sweep {
            return super::native_sweep::trace(h, self.space, self.narrow, &mut self.records);
        }
        if self.segment {
            return super::native_segment::trace(h, self.space, self.narrow, &mut self.records);
        }
        let p = h.play.as_ref().ok_or("missing spatial world")?;
        let f = p.last.as_ref().ok_or("missing spatial frame")?;
        let d = &h.editor.document;
        let actor = &d.objects[0];
        let zone = &d.objects[1];
        let (lo, hi) = Self::offsets(h, actor)?;
        let (zl, zh) = Self::offsets(h, zone)?;
        let mut local = actor.translation;
        let mut group = actor
            .parent
            .map(|id| d.groups.iter().find(|g| g.id == id).unwrap().translation)
            .unwrap_or([0.; 3]);
        let hit = |world: [f32; 3]| {
            (0..self.space as usize).all(|k| {
                if self.narrow {
                    world[k] >= zone.translation[k] + zl[k]
                        && world[k] <= zone.translation[k] + zh[k]
                } else {
                    world[k] + hi[k] >= zone.translation[k] + zl[k]
                        && world[k] + lo[k] <= zone.translation[k] + zh[k]
                }
            })
        };
        let mut world = std::array::from_fn(|i| local[i] + group[i]);
        let mut previous = hit(world);
        let (mut enters, mut stays, mut exits) = (0, 0, 0);
        let mut events = vec![];
        for r in &p.reports {
            if r.space == self.space {
                let dx = r.x as f32 * if self.space == 2 { 60. } else { 3. } * DT;
                if self.space == 2 {
                    local[0] += dx;
                    local[1] += r.y as f32 * 60. * DT;
                } else {
                    group[0] += dx;
                    group[2] += 0. - r.y as f32 * 3. * DT;
                }
            }
            world = std::array::from_fn(|i| local[i] + group[i]);
            let current = hit(world);
            if current {
                if previous {
                    stays += 1;
                } else {
                    enters += 1;
                    events.push(json!({"event":"enter","tick":r.tick}));
                }
            } else if previous {
                exits += 1;
                events.push(json!({"event":"exit","tick":r.tick}));
            }
            previous = current;
        }
        let actual = &f.objects[0];
        for k in 0..3 {
            Self::check(
                (actual.translation[k] - world[k]).abs() < 1e-3,
                "spatial input/world trace",
            )?;
        }
        let marker = f
            .objects
            .iter()
            .find(|o| o.id == if self.space == 2 { 3 } else { 4 })
            .unwrap();
        let door = f
            .objects
            .iter()
            .find(|o| o.id == if self.space == 2 { 4 } else { 3 })
            .unwrap();
        Self::check(
            (marker.translation[0] - (-100. + enters as f32 * 10.)).abs() < 1e-3,
            "spatial enter count",
        )?;
        Self::check(
            (marker.translation[1] - (50. + exits as f32 * 10.)).abs() < 1e-3,
            "spatial exit count",
        )?;
        Self::check(door.visible == previous, "spatial door state")?;
        let v = if self.space == 2 {
            door.translation[0]
        } else {
            door.translation[1]
        };
        let expected = if self.space == 2 {
            100. + stays as f32 * 0.1
        } else {
            1. + stays as f32 * 0.01
        };
        Self::check((v - expected).abs() < 1e-3, "spatial stay count")?;
        self.records.push(json!({"case":"spatial-trace","tick":p.tick(),"space":self.space,"narrow":self.narrow,"events":events,"enters":enters,"stays":stays,"exits":exits,"inside":previous,"source":p.source_hash,"api":p.user_source.as_ref().map(|s|&s.version),"identity":p.user_source.as_ref().map(|s|json!({"path":s.path,"user":s.user_hash,"api":s.api_hash,"bridge":s.bridge_hash,"combined":s.combined_hash,"geometry":s.base_scene.as_ref().map(|b|b["spatial_hash"].clone())})),"sdk_commit":serde_json::from_str::<Value>(include_str!("../../../research/baseline.json.txt")).unwrap()["sdk"]["repository_commit"],"actions":p.reports}));
        Ok(())
    }
    fn pixels(&mut self, h: &mut Host, stage: &str) -> Result<(), String> {
        let objects = h
            .play
            .as_ref()
            .and_then(|p| p.last.as_ref())
            .map(|f| f.objects.clone())
            .unwrap_or_else(|| h.editor.preview.objects.clone());
        let o = &objects[0];
        let mut checks = vec![];
        if self.space == 2 {
            let p =
                h.rt.probe_pixel(h.top, [400. + o.translation[0], 190. - o.translation[1]])
                    .ok_or("2d pixel")?;
            checks.push(
                json!({"part":"circle","pixel":p,"passed":p[0]>100&&p[0]>p[1].saturating_add(30)}),
            );
        } else {
            let model = &h
                .editor
                .assets
                .get(o.resource.as_ref().unwrap())
                .ok_or("pixel asset")?
                .model;
            let mut points = vec![];
            let t = Mat4::from_translation(o.translation.into())
                * Mat4::from_rotation_y(o.yaw)
                * Mat4::from_rotation_x(o.pitch)
                * Mat4::from_scale(o.scale.into());
            for mesh in &model.meshes {
                let mut lo = Vec3::splat(f32::INFINITY);
                let mut hi = Vec3::splat(f32::NEG_INFINITY);
                for v in &mesh.positions {
                    lo = lo.min(Vec3::from(*v));
                    hi = hi.max(Vec3::from(*v));
                }
                points.push(t.transform_point3((lo + hi) * 0.5));
            }
            for (i, point) in points.into_iter().enumerate() {
                let point =
                    autographics_auto_driver::authoring::interaction::project(h.camera(), point)
                        .ok_or("part offscreen")?;
                let p =
                    h.rt.probe_pixel(h.top, [point.x, point.y + 400.])
                        .ok_or("3d pixel")?;
                checks.push(json!({"part":i,"pixel":p,"passed":p[i]>80&&p[1-i]<20}));
            }
        }
        let passed = checks.iter().all(|c| c["passed"] == true);
        self.records
            .push(json!({"case":"spatial-pixels","stage":stage,"checks":checks,"passed":passed}));
        Self::check(passed, "spatial pixels final-content failure")
    }
    pub fn step(&mut self, h: &mut Host, frame: u64, w: &winit::window::Window) {
        if self.done || self.error.is_some() || frame - self.at < 12 {
            return;
        }
        if frame - self.at > 360 {
            self.error = Some(format!("spatial timeout phase {}", self.phase));
            return;
        }
        match self.run(h, w) {
            Ok(true) => {
                self.phase += 1;
                self.at = frame;
                if self.phase >= 14 {
                    self.done = true
                }
            }
            Ok(false) => {}
            Err(e) => self.error = Some(e),
        }
    }
    fn run(&mut self, h: &mut Host, w: &winit::window::Window) -> Result<bool, String> {
        if let Some(p) = &h.play {
            if p.state == State::Faulted {
                return Err(format!("spatial runtime fault {:?}", p.error));
            }
        }
        match self.phase {
            0 => {
                if !w.has_focus() {
                    w.focus_window();
                    return Ok(false);
                };
                w.set_cursor_position(winit::dpi::PhysicalPosition::new(400., 190.))
                    .map_err(|e| e.to_string())?;
            }
            1 => {
                return Self::click(
                    h,
                    if self.motion {
                        "action-behavior-api-v5"
                    } else if self.sweep {
                        "action-behavior-api-v4"
                    } else if self.segment {
                        "action-behavior-api-v3"
                    } else {
                        "action-behavior-api-v2"
                    },
                )
            }
            2 => {
                if self.update {
                    match self.update_stage {
                        0 => {
                            self.old_source = h
                                .editor
                                .document
                                .behavior
                                .as_ref()
                                .map(|b| b.source.sha256.clone());
                            if Self::click(h, "action-behavior-bind")? {
                                self.update_stage = 1;
                            }
                            return Ok(false);
                        }
                        1 => {
                            if h.editor.pending() {
                                return Ok(false);
                            }
                            Self::check(
                                h.editor
                                    .document
                                    .behavior
                                    .as_ref()
                                    .map(|b| b.source.sha256.clone())
                                    != self.old_source
                                    && h.editor.dirty(),
                                "OS source update must change accepted binding",
                            )?;
                            let save_as = if self.segment || self.sweep || self.motion {
                                let args: Vec<_> = std::env::args().collect();
                                args.iter()
                                    .position(|v| {
                                        v == if self.motion {
                                            "--motion-saveas"
                                        } else if self.sweep {
                                            "--sweep-saveas"
                                        } else {
                                            "--segment-saveas"
                                        }
                                    })
                                    .and_then(|i| args.get(i + 1))
                                    .cloned()
                            } else {
                                None
                            };
                            let action = if let Some(path) = save_as {
                                h.ui_mut().path = path;
                                h.rt.graph.mark_content_dirty(h.panel);
                                "action-saveas"
                            } else {
                                "action-save"
                            };
                            if Self::click(h, action)? {
                                self.update_stage = 2;
                            }
                            return Ok(false);
                        }
                        2 => {
                            if h.editor.pending() {
                                return Ok(false);
                            }
                            Self::check(
                                !h.editor.dirty() && !h.storage.unverified(),
                                "OS Save must confirm updated source closure",
                            )?;
                            if Self::click(h, "action-play-project-default")? {
                                self.update_stage = 3;
                            }
                            return Ok(false);
                        }
                        3 => {
                            Self::check(h.project_default, "OS project default after Update")?;
                            self.records.push(json!({"case":"spatial-update-saved","old":self.old_source,"new":h.editor.document.behavior,"path":h.storage.path}));
                            self.update_stage = 4;
                        }
                        _ => {}
                    }
                }
                Self::check(
                    h.ui().behavior_api
                        == if self.motion {
                            Api::MotionV5
                        } else if self.sweep {
                            Api::SweepV4
                        } else if self.segment {
                            Api::SegmentV3
                        } else {
                            Api::SpatialV2
                        },
                    "OS API choice",
                )?;
                self.before = Some(h.editor.isolation_identity());
                self.images = Some(Self::images(h)?);
                self.pixels(h, "editing")?;
                h.asset_render_fault = self.fault.clone();
                return Self::click(h, "action-play-run");
            }
            3 => {
                let Some(p) = &h.play else {
                    if !h.ui().error.is_empty() {
                        return Err(h.ui().error.clone());
                    }
                    return Ok(false);
                };
                if p.state == State::Starting {
                    return Ok(false);
                }
                Self::check(p.state == State::Running, "spatial Running")?;
                Self::check(
                    p.user_source.as_ref().is_some_and(|s| {
                        s.version
                            == if self.motion {
                                Api::MotionV5.version()
                            } else if self.sweep {
                                Api::SweepV4.version()
                            } else if self.segment {
                                Api::SegmentV3.version()
                            } else {
                                Api::SpatialV2.version()
                            }
                            && Some(&s.user_hash)
                                == h.editor
                                    .document
                                    .behavior
                                    .as_ref()
                                    .map(|b| &b.source.sha256)
                    }),
                    "actual bound spatial source/API identity",
                )?;
                super::native_interaction::Probe::click(
                    h,
                    [400., if self.space == 2 { 190. } else { 600. }],
                )?;
                super::native_interaction::post(0x100, 0x27, (0, 0))?;
            }
            4 => {
                let p = h.play.as_ref().unwrap();
                if p.reports
                    .iter()
                    .filter(|r| r.space == self.space && r.x == 1)
                    .count()
                    < 80
                {
                    return Ok(false);
                }
                super::native_interaction::post(0x101, 0x27, (0, 0))?;
                return Self::click(h, "action-play-pause");
            }
            5 => {
                let p = h.play.as_ref().unwrap();
                if p.pending() {
                    return Ok(false);
                }
                Self::check(p.state == State::Paused, "spatial Paused")?;
                self.tick = p.tick();
                self.trace(h)?;
                let r = self.records.last().unwrap();
                Self::check(
                    self.segment
                        || self.sweep
                        || self.motion
                        || (r["enters"].as_i64().unwrap() >= 1
                            && r["stays"].as_i64().unwrap() >= 1
                            && r["exits"].as_i64().unwrap() >= 1),
                    "outside enter stay exit required",
                )?;
                self.pixels(h, "after-exit")?;
            }
            6 => {
                Self::check(
                    h.play.as_ref().unwrap().tick() == self.tick,
                    "Pause must not repeat events",
                )?;
                return Self::click(h, "action-play-step");
            }
            7 => {
                let p = h.play.as_ref().unwrap();
                if p.pending() {
                    return Ok(false);
                }
                Self::check(
                    p.tick() == self.tick + 1 && p.state == State::Paused,
                    "one neutral Step",
                )?;
                self.trace(h)?;
                self.pixels(h, "after-step")?;
                return Self::click(h, "action-play-resume");
            }
            8 => {
                if h.play.as_ref().unwrap().tick() < self.tick + 3 {
                    return Ok(false);
                }
                return Self::click(h, "action-play-pause");
            }
            9 => {
                if h.play.as_ref().unwrap().pending() {
                    return Ok(false);
                }
                self.trace(h)?;
                return Self::click(h, "action-play-stop");
            }
            10 => {
                Self::check(h.play.is_none(), "Stop must drop world")?;
                Self::check(
                    Some(h.editor.isolation_identity()) == self.before,
                    "Stop preserves accepted editor",
                )?;
                Self::check(
                    Some(Self::images(h)?) == self.images,
                    "Stop restores both canvases",
                )?;
                h.asset_render_fault = None;
            }
            11 => return Self::click(h, "action-play-run"),
            12 => {
                let Some(p) = &h.play else { return Ok(false) };
                if p.state == State::Starting {
                    return Ok(false);
                }
                self.trace(h)?;
                let r = self.records.last().unwrap();
                Self::check(
                    self.segment
                        || self.sweep
                        || self.motion
                        || (r["enters"] == 0 && r["stays"] == 0 && r["exits"] == 0),
                    "new Run clears private events",
                )?;
                return Self::click(h, "action-play-stop");
            }
            13 => {
                Self::check(h.play.is_none(), "second Stop")?;
                Self::check(
                    Some(Self::images(h)?) == self.images,
                    "second Stop restores pixels",
                )?;
            }
            _ => return Err("spatial invalid phase".into()),
        }
        Ok(true)
    }
}
