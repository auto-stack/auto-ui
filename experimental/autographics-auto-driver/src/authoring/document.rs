//! Persistent scene identity and strict bounded codec; no runtime handles.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
pub const MAX_FILE: usize = 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Circle,
    Quad,
    Cylinder,
    Player,
    Mob,
    Model = 6,
}
impl Kind {
    pub fn is_2d(self) -> bool {
        matches!(self, Self::Circle | Self::Quad)
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Quad => "quad",
            Self::Cylinder => "cylinder",
            Self::Player => "player",
            Self::Mob => "mob",
            Self::Model => "model",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    pub translation: [f32; 3],
    pub scale: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub rgba: [f32; 4],
    pub visible: bool,
    pub parent: Option<u64>,
    pub resource: Option<String>,
    /// PLAN-004: the accepted PNG visual bound to this 2D leaf. Identity is
    /// the image content hash; the frame is integer pixels inside that image
    /// and the display size stays the object scale, so a frame never changes
    /// the entity bounds.
    pub sprite: Option<SpriteBinding>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteBinding {
    pub asset: String,
    pub frame: [u32; 4],
    pub flip: bool,
}
impl SpriteBinding {
    pub fn validate(&self, assets: &[super::assets::AssetRef]) -> Result<(), String> {
        let asset = assets
            .iter()
            .find(|a| a.key() == self.asset)
            .ok_or_else(|| "sprite asset reference".to_string())?;
        if !asset.is_image() {
            return Err("sprite requires an image resource".into());
        }
        let (width, height) = match (asset.width, asset.height) {
            (Some(w), Some(h)) => (w, h),
            _ => return Err("sprite image dimensions".into()),
        };
        let [x, y, w, h] = self.frame;
        if w == 0 || h == 0 || w > width || h > height || x > width - w || y > height - h {
            return Err("sprite frame bounds".into());
        }
        Ok(())
    }
}
impl Object {
    pub fn new(id: u64, kind: Kind) -> Self {
        Self {
            id,
            name: format!("{} {id}", kind.key()),
            kind,
            translation: [0.0; 3],
            scale: if kind.is_2d() {
                [50.0, 50.0, 1.0]
            } else {
                [1.0; 3]
            },
            yaw: 0.0,
            pitch: 0.0,
            rgba: [0.9, 0.3, 0.2, 1.0],
            visible: true,
            parent: None,
            resource: None,
            sprite: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub behavior: Option<super::behavior_project::Binding>,
    pub schema: String,
    pub document_id: String,
    pub resources: Vec<String>,
    pub objects: Vec<Object>,
    pub groups: Vec<Group>,
    pub assets: Vec<super::assets::AssetRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    #[serde(rename = "2d")]
    Two,
    #[serde(rename = "3d")]
    Three,
}
impl Dimension {
    pub fn of(kind: Kind) -> Self {
        if kind.is_2d() {
            Self::Two
        } else {
            Self::Three
        }
    }
    pub fn code(self) -> i32 {
        if self == Self::Two {
            2
        } else {
            3
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub dimension: Dimension,
    pub parent: Option<u64>,
    pub translation: [f32; 3],
    pub visible: bool,
}
impl Group {
    pub fn new(id: u64, dimension: Dimension) -> Self {
        Self {
            id,
            name: format!("Group {id}"),
            dimension,
            parent: None,
            translation: [0.0; 3],
            visible: true,
        }
    }
}
impl Document {
    pub fn empty(id: String) -> Self {
        Self {
            schema: "autographics.scene/5".into(),
            behavior: None,
            document_id: id,
            resources: ["circle", "quad", "cylinder", "player", "mob"]
                .map(str::to_owned)
                .to_vec(),
            objects: vec![],
            groups: vec![],
            assets: vec![],
        }
    }
    pub fn sample() -> Self {
        let mut d = Self::empty("authoring-sample".into());
        for (i, k) in [Kind::Circle, Kind::Quad, Kind::Player, Kind::Mob]
            .into_iter()
            .enumerate()
        {
            let mut o = Object::new(i as u64 + 1, k);
            o.name = "Object".into();
            o.translation[0] = if k.is_2d() {
                (i as f32) * 110.0 - 55.0
            } else {
                (i as f32 - 2.5) * 3.0
            };
            d.objects.push(o)
        }
        d
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "autographics.scene/5" {
            return Err("unsupported schema".into());
        }
        if self.document_id.is_empty() || self.document_id.chars().count() > 128 {
            return Err("document identity".into());
        }
        if self.resources != Self::empty("x".into()).resources {
            return Err("fixed resource manifest".into());
        }
        if self.objects.len() + self.groups.len() > 128 {
            return Err("node resource limit 128".into());
        }
        let mut ids = BTreeSet::new();
        let mut assets = BTreeSet::new();
        let mut raw_bytes = 0;
        for a in &self.assets {
            a.validate()?;
            if !assets.insert(a.key()) {
                return Err("duplicate asset identity".into());
            }
            raw_bytes += a.byte_length;
        }
        if assets.len() > super::assets::MAX_ASSETS || raw_bytes > super::assets::MAX_RAW_TOTAL {
            return Err("asset manifest budget".into());
        }
        let mut bound = 0usize;
        for o in &self.objects {
            if o.kind == Kind::Model {
                if !o.resource.as_ref().is_some_and(|k| assets.contains(k)) {
                    return Err("model resource reference".into());
                }
                let payload = self
                    .assets
                    .iter()
                    .find(|a| Some(a.key()).as_ref() == o.resource.as_ref())
                    .ok_or_else(|| "model resource reference".to_string())?;
                if payload.is_image() {
                    return Err("model requires a geometry resource".into());
                }
            } else if o.resource.is_some() {
                return Err("builtin resource mismatch".into());
            }
            if let Some(sprite) = &o.sprite {
                if !o.kind.is_2d() {
                    return Err("sprite binding on a 2D leaf only".into());
                }
                sprite.validate(&self.assets)?;
                bound += 1;
                if bound > super::runtime_sprites::MAX_TARGETS {
                    return Err("sprite target budget".into());
                }
            }
            if o.id == 0 || !ids.insert(o.id) {
                return Err("duplicate/invalid object identity".into());
            }
            if o.name.chars().count() > 128 {
                return Err("name limit 128".into());
            }
            if o.translation
                .iter()
                .chain(&o.scale)
                .chain([&o.yaw, &o.pitch])
                .any(|v| !v.is_finite() || v.abs() > 100000.0)
                || o.scale.iter().any(|v| *v <= 0.0)
                || o.rgba
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("numeric range".into());
            }
            if o.kind.is_2d()
                && (o.translation[2] != 0.0 || o.scale[2] != 1.0 || o.yaw != 0.0 || o.pitch != 0.0)
            {
                return Err("2D unsupported property".into());
            }
            if !o.kind.is_2d() && o.rgba != [0.9, 0.3, 0.2, 1.0] {
                return Err("3D material is fixed".into());
            }
        }
        for g in &self.groups {
            if g.id == 0 || !ids.insert(g.id) {
                return Err("duplicate/invalid node identity".into());
            }
            if g.name.chars().count() > 128 {
                return Err("name limit 128".into());
            }
            if g.translation
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 100000.0)
            {
                return Err("group numeric range".into());
            }
            if g.dimension == Dimension::Two && g.translation[2] != 0.0 {
                return Err("2D group z".into());
            }
        }
        if let Some(b) = &self.behavior {
            b.validate(self)?;
        }
        super::hierarchy::derive(self)?;
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let b = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        if b.len() > MAX_FILE {
            return Err("file limit 1MiB".into());
        }
        Ok(b)
    }
    pub fn hash(&self) -> Result<String, String> {
        Ok(agp::renderer::capture::sha256_hex(&self.encode()?))
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_FILE {
            return Err("file limit 1MiB".into());
        }
        let mut de = serde_json::Deserializer::from_slice(bytes);
        let v = Unique::deserialize(&mut de).map_err(|e| e.to_string())?;
        de.end().map_err(|e| e.to_string())?;
        let mut v = v.0;
        let schema = v
            .get("schema")
            .and_then(Value::as_str)
            .ok_or("schema required")?
            .to_owned();
        if schema == "autographics.scene/1" || schema == "autographics.scene/2" {
            if v.get("assets").is_some() {
                return Err("legacy unknown assets".into());
            }
            for o in v
                .get("objects")
                .and_then(Value::as_array)
                .ok_or("objects array")?
            {
                if o.get("resource").is_some()
                    || o.get("kind").and_then(Value::as_str) == Some("model")
                {
                    return Err("legacy unknown resource/model".into());
                }
            }
        }
        if v.get("schema").and_then(Value::as_str) == Some("autographics.scene/1") {
            let root = v.as_object_mut().ok_or("document object")?;
            if root.contains_key("groups") {
                return Err("v1 unknown groups".into());
            }
            for o in root
                .get_mut("objects")
                .and_then(Value::as_array_mut)
                .ok_or("objects array")?
            {
                let o = o.as_object_mut().ok_or("object record")?;
                if o.contains_key("parent") {
                    return Err("v1 unknown parent".into());
                }
                o.insert("parent".into(), Value::Null);
            }
            root.insert("groups".into(), Value::Array(vec![]));
            root.insert(
                "schema".into(),
                Value::String("autographics.scene/2".into()),
            );
        }
        for key in ["objects", "groups"] {
            for node in v.get(key).and_then(Value::as_array).ok_or("node arrays")? {
                if node.get("parent").is_none() {
                    return Err("explicit parent required".into());
                }
            }
        }
        if schema == "autographics.scene/1" || schema == "autographics.scene/2" {
            v["schema"] = Value::String("autographics.scene/3".into());
            v["assets"] = Value::Array(vec![]);
            for o in v["objects"].as_array_mut().ok_or("objects")? {
                o["resource"] = Value::Null;
            }
        } else {
            for o in v["objects"].as_array().ok_or("objects")? {
                if o.get("resource").is_none() {
                    return Err("explicit resource required".into());
                }
            }
        }
        if [
            "autographics.scene/1",
            "autographics.scene/2",
            "autographics.scene/3",
        ]
        .contains(&schema.as_str())
        {
            if v.get("behavior").is_some() {
                return Err("legacy unknown behavior".into());
            }
            v["behavior"] = Value::Null;
            v["schema"] = Value::String("autographics.scene/4".into());
        } else if v.get("behavior").is_none() {
            return Err("explicit behavior required".into());
        }
        // Nullable actor fields are nevertheless required in the wire format.
        if let Some(b) = v.get("behavior").filter(|b| !b.is_null()) {
            for k in ["two_d", "three_d"] {
                if b.get("actors").and_then(|a| a.get(k)).is_none() {
                    return Err("explicit behavior actors required".into());
                }
            }
        }
        // PLAN-004 scene/5 adds the optional PNG visual binding. Older schemas
        // keep their exact meaning: a legacy file carrying the new field or a
        // non-model resource is rejected rather than silently upgraded.
        if [
            "autographics.scene/1",
            "autographics.scene/2",
            "autographics.scene/3",
            "autographics.scene/4",
        ]
        .contains(&schema.as_str())
        {
            for a in v["assets"].as_array().ok_or("assets")? {
                if a.get("format").and_then(Value::as_str) != Some("glb") {
                    return Err("legacy unknown asset format".into());
                }
            }
            for o in v["objects"].as_array_mut().ok_or("objects")? {
                if o.get("sprite").is_some() {
                    return Err("legacy unknown sprite".into());
                }
                o["sprite"] = Value::Null;
            }
            v["schema"] = Value::String("autographics.scene/5".into());
        } else if schema == "autographics.scene/5" {
            for o in v["objects"].as_array().ok_or("objects")? {
                if o.get("sprite").is_none() {
                    return Err("explicit sprite required".into());
                }
            }
        } else {
            return Err("unsupported schema".into());
        }
        let d: Self = serde_json::from_value(v).map_err(|e| e.to_string())?;
        d.validate()?;
        Ok(d)
    }
    pub fn contains(&self, id: u64) -> bool {
        self.objects.iter().any(|o| o.id == id) || self.groups.iter().any(|g| g.id == id)
    }
    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.objects
            .iter()
            .map(|o| o.id)
            .chain(self.groups.iter().map(|g| g.id))
    }
    pub fn parent(&self, id: u64) -> Result<Option<u64>, String> {
        if let Some(o) = self.objects.iter().find(|o| o.id == id) {
            return Ok(o.parent);
        }
        self.groups
            .iter()
            .find(|g| g.id == id)
            .map(|g| g.parent)
            .ok_or("target missing".into())
    }
    /// A local group origin for the existing translation-only gesture; never a renderable leaf.
    pub fn local_target(&self, id: u64) -> Option<Object> {
        self.objects
            .iter()
            .find(|o| o.id == id)
            .cloned()
            .or_else(|| {
                let g = self.groups.iter().find(|g| g.id == id)?;
                let mut o = Object::new(
                    g.id,
                    if g.dimension == Dimension::Two {
                        Kind::Quad
                    } else {
                        Kind::Cylinder
                    },
                );
                o.name = g.name.clone();
                o.translation = g.translation;
                o.visible = g.visible;
                o.parent = g.parent;
                Some(o)
            })
    }
}
// Preserve duplicate-key evidence before serde_json::Value could discard it.
struct Unique(Value);
pub fn unique_json(bytes: &[u8]) -> Result<Value, String> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let v = Unique::deserialize(&mut de).map_err(|e| format!("invalid: {e}"))?;
    de.end().map_err(|e| format!("invalid: {e}"))?;
    Ok(v.0)
}
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(UVisitor)
    }
}
struct UVisitor;
impl<'de> Visitor<'de> for UVisitor {
    type Value = Unique;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("bounded unique-key JSON")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Unique, M::Error> {
        let mut out = serde_json::Map::new();
        while let Some(k) = m.next_key::<String>()? {
            if out.contains_key(&k) {
                return Err(de::Error::custom("duplicate JSON key"));
            }
            let v = m.next_value::<Unique>()?;
            out.insert(k, v.0);
        }
        Ok(Unique(Value::Object(out)))
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut s: S) -> Result<Unique, S::Error> {
        let mut out = vec![];
        while let Some(v) = s.next_element::<Unique>()? {
            out.push(v.0)
        }
        Ok(Unique(Value::Array(out)))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
        Ok(Unique(Value::String(v.into())))
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
        Ok(Unique(Value::Bool(v)))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
        Ok(Unique(v.into()))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
        Ok(Unique(v.into()))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
        serde_json::Number::from_f64(v)
            .map(|n| Unique(Value::Number(n)))
            .ok_or_else(|| E::custom("finite number"))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
        Ok(Unique(Value::Null))
    }
}
