//! Versioned, bounded source snapshots. User bytes are never rewritten.
use super::{assets, play::SOURCE};
use auto_lang_core::{
    ast::{self, Arg, Body, Expr, Stmt, Type},
    parser::Parser,
};
use auto_val::Op;
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
pub const VERSION: &str = "autographics.behavior/1";
pub const MOTION_API: &str = include_str!("../../api/behavior_motion_v5.at");
pub const MOTION_SCAN_API: &str = include_str!("../../api/behavior_motion_scan_v5.at");
pub const VERSION5: &str = "autographics.behavior/5";
pub const VERSION4: &str = "autographics.behavior/4";
pub const SWEEP_API: &str = include_str!("../../api/behavior_sweep_v4.at");
pub const VERSION3: &str = "autographics.behavior/3";
pub const SEGMENT_API: &str = include_str!("../../api/behavior_segment_v3.at");
pub const VERSION2: &str = "autographics.behavior/2";
pub const SPATIAL_API: &str = include_str!("../../api/behavior_spatial_v2.at");
pub const CHARACTER_VERSION: &str = "autographics.character/1";
pub const CHARACTER_API: &str = include_str!("../../api/character2d_v1.at");
pub const CHARACTER3_VERSION: &str = "autographics.character3d/1";
pub const CHARACTER3_API: &str = include_str!("../../api/character3d_v1.at");
pub const CAMERA2_VERSION: &str = "autographics.runtime_camera2d/1";
pub const CAMERA2_API: &str = include_str!("../../api/runtime_camera2d_v1.at");
pub const CAMERA3_VERSION: &str = "autographics.runtime_camera3d/1";
pub const CAMERA3_API: &str = include_str!("../../api/runtime_camera3d_v1.at");
pub const OBJECTS2_VERSION: &str = "autographics.objects2d/1";
pub const OBJECTS3_VERSION: &str = "autographics.objects3d/1";
pub const OBJECTS_API: &str = include_str!("../../api/runtime_objects_v1.at");
pub const TRIGGERS2_VERSION: &str = "autographics.triggers2d/1";
pub const TRIGGERS3_VERSION: &str = "autographics.triggers3d/1";
pub const TRIGGERS_API: &str = include_str!("../../api/runtime_triggers_v1.at");
/// PLAN-004: the finite 2D sprite profile layers the frame/flip unit on the
/// unchanged triggers2d composition. It is an explicit new version; every
/// earlier profile keeps its exact assembled bytes.
pub const SPRITES2_VERSION: &str = "autographics.sprites2d/1";
pub const SPRITES_API: &str = include_str!("../../api/sprite2d_v1.at");
pub const NODE_ANIMATION3_VERSION: &str = "autographics.node_animation3d/1";
pub const NODE_ANIMATION_API: &str = include_str!("../../api/node_animation3d_v1.at");
pub const AUDIO2_VERSION: &str = "autographics.audio2d/1";
pub const AUDIO3_VERSION: &str = "autographics.audio3d/1";
pub const AUDIO_API: &str = include_str!("../../api/public_audio_v1.at");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Api {
    V1,
    SpatialV2,
    SegmentV3,
    SweepV4,
    MotionV5,
    CharacterV1,
    Character3V1,
    Camera2V1,
    Camera3V1,
    Objects2V1,
    Objects3V1,
    Triggers2V1,
    Triggers3V1,
    Sprites2V1,
    NodeAnimation3V1,
    Audio2V1,
    Audio3V1,
}
impl Api {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            VERSION => Ok(Self::V1),
            VERSION2 => Ok(Self::SpatialV2),
            VERSION5 => Ok(Self::MotionV5),
            VERSION4 => Ok(Self::SweepV4),
            VERSION3 => Ok(Self::SegmentV3),
            CHARACTER_VERSION => Ok(Self::CharacterV1),
            CHARACTER3_VERSION => Ok(Self::Character3V1),
            CAMERA2_VERSION => Ok(Self::Camera2V1),
            CAMERA3_VERSION => Ok(Self::Camera3V1),
            OBJECTS2_VERSION => Ok(Self::Objects2V1),
            OBJECTS3_VERSION => Ok(Self::Objects3V1),
            TRIGGERS2_VERSION => Ok(Self::Triggers2V1),
            TRIGGERS3_VERSION => Ok(Self::Triggers3V1),
            SPRITES2_VERSION => Ok(Self::Sprites2V1),
            AUDIO2_VERSION => Ok(Self::Audio2V1),
            AUDIO3_VERSION => Ok(Self::Audio3V1),
            NODE_ANIMATION3_VERSION => Ok(Self::NodeAnimation3V1),
            _ => Err("unsupported behavior API".into()),
        }
    }
    pub fn version(self) -> &'static str {
        match self {
            Self::V1 => VERSION,
            Self::SpatialV2 => VERSION2,
            Self::SegmentV3 => VERSION3,
            Self::SweepV4 => VERSION4,
            Self::MotionV5 => VERSION5,
            Self::CharacterV1 => CHARACTER_VERSION,
            Self::Character3V1 => CHARACTER3_VERSION,
            Self::Camera2V1 => CAMERA2_VERSION,
            Self::Camera3V1 => CAMERA3_VERSION,
            Self::Objects2V1 => OBJECTS2_VERSION,
            Self::Objects3V1 => OBJECTS3_VERSION,
            Self::Triggers2V1 => TRIGGERS2_VERSION,
            Self::Triggers3V1 => TRIGGERS3_VERSION,
            Self::Sprites2V1 => SPRITES2_VERSION,
            Self::NodeAnimation3V1 => NODE_ANIMATION3_VERSION,
            Self::Audio2V1 => AUDIO2_VERSION,
            Self::Audio3V1 => AUDIO3_VERSION,
        }
    }
    pub fn number(self) -> i32 {
        match self {
            Self::V1 => 1,
            Self::SpatialV2 => 2,
            Self::SegmentV3 => 3,
            Self::SweepV4 => 4,
            Self::MotionV5 => 5,
            Self::CharacterV1 => 101,
            Self::Character3V1 => 102,
            Self::Camera2V1 => 103,
            Self::Camera3V1 => 104,
            Self::Objects2V1 => 105,
            Self::Objects3V1 => 106,
            Self::Triggers2V1 => 107,
            Self::Triggers3V1 => 108,
            Self::Sprites2V1 => 109,
            Self::NodeAnimation3V1 => 110,
            Self::Audio2V1 => 111,
            Self::Audio3V1 => 112,
        }
    }
    pub fn profile(self) -> i32 {
        match self {
            Self::V1 => 7,
            Self::SpatialV2 => 8,
            Self::SegmentV3 => 9,
            Self::SweepV4 => 10,
            Self::MotionV5 => 11,
            Self::CharacterV1 => 12,
            Self::Character3V1 => 13,
            Self::Camera2V1 => 14,
            Self::Camera3V1 => 15,
            Self::Objects2V1 => 16,
            Self::Objects3V1 => 17,
            Self::Triggers2V1 => 18,
            Self::Triggers3V1 => 19,
            Self::Sprites2V1 => 20,
            Self::NodeAnimation3V1 => 21,
            Self::Audio2V1 => 22,
            Self::Audio3V1 => 23,
        }
    }
    /// The finite run-object profiles including the trigger profiles that
    /// carry the objects unit: one dimension per Run, 8 reserved slots.
    pub fn objects(self) -> Option<super::runtime_objects::Dim> {
        match self {
            Self::Objects2V1 | Self::Triggers2V1 | Self::Sprites2V1 | Self::Audio2V1 => {
                Some(super::runtime_objects::Dim::TwoD)
            }
            Self::Objects3V1 | Self::Triggers3V1 | Self::NodeAnimation3V1 | Self::Audio3V1 => {
                Some(super::runtime_objects::Dim::ThreeD)
            }
            _ => None,
        }
    }
    /// The finite trigger/round profiles (PLAN-002): 8 explicit pairs and an
    /// audited restart request on top of the objects composition.
    pub fn triggers(self) -> Option<super::runtime_objects::Dim> {
        match self {
            Self::Triggers2V1 | Self::Sprites2V1 | Self::Audio2V1 => {
                Some(super::runtime_objects::Dim::TwoD)
            }
            Self::Triggers3V1 | Self::NodeAnimation3V1 | Self::Audio3V1 => {
                Some(super::runtime_objects::Dim::ThreeD)
            }
            _ => None,
        }
    }
    /// The PLAN-004 sprite profile: the bound-leaf frame/flip table and the
    /// reserved-slot table riding the unchanged triggers2d composition.
    pub fn sprites(self) -> bool {
        matches!(self, Self::Sprites2V1 | Self::Audio2V1)
    }
    /// Every shipped profile, in the order the command line lists them.
    pub fn audio(self) -> bool {
        matches!(self, Self::Audio2V1 | Self::Audio3V1)
    }
    pub fn node_animation(self) -> bool {
        matches!(self, Self::NodeAnimation3V1 | Self::Audio3V1)
    }
    pub fn all() -> [Self; 17] {
        [
            Self::V1,
            Self::SpatialV2,
            Self::SegmentV3,
            Self::SweepV4,
            Self::MotionV5,
            Self::CharacterV1,
            Self::Character3V1,
            Self::Camera2V1,
            Self::Camera3V1,
            Self::Objects2V1,
            Self::Objects3V1,
            Self::Triggers2V1,
            Self::Triggers3V1,
            Self::Sprites2V1,
            Self::NodeAnimation3V1,
            Self::Audio2V1,
            Self::Audio3V1,
        ]
    }
    /// The command-line name of this profile. The public entry selects a Run by
    /// this name, so every shipped profile must have one.
    pub fn cli_name(self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::SpatialV2 => "v2",
            Self::SegmentV3 => "v3",
            Self::SweepV4 => "v4",
            Self::MotionV5 => "v5",
            Self::CharacterV1 => "character",
            Self::Character3V1 => "character3d",
            Self::Camera2V1 => "camera2d",
            Self::Camera3V1 => "camera3d",
            Self::Objects2V1 => "objects2d",
            Self::Objects3V1 => "objects3d",
            Self::Triggers2V1 => "triggers2d",
            Self::Triggers3V1 => "triggers3d",
            Self::Sprites2V1 => "sprites2d",
            Self::NodeAnimation3V1 => "node-animation3d",
            Self::Audio2V1 => "audio2d",
            Self::Audio3V1 => "audio3d",
        }
    }
    /// Parse one `--behavior-api` value; an unknown name is a usage error.
    pub fn parse_cli(value: &str) -> Option<Self> {
        Self::all().into_iter().find(|api| api.cli_name() == value)
    }
    /// The exact accepted value list, derived from the names above so a new
    /// profile cannot be added without also being selectable.
    pub fn cli_usage() -> String {
        Self::all()
            .iter()
            .map(|api| api.cli_name())
            .collect::<Vec<_>>()
            .join(" or ")
    }
}
pub const API: &str = include_str!("../../api/behavior_v1.at");
/// The managed bridge envelope. r5 (PLAN-002): the triggers composition
/// measured 72141/74050 bytes assembled against the previous 64KiB limit
/// (Objects2V1 fits at 61950), so the frozen envelope was raised explicitly —
/// never silently — to 96KiB with the counterexample recorded in the Plan;
/// the 256KiB user-source limit and every runtime budget are unchanged.
pub const MAX_BRIDGE: usize = 96 * 1024;
pub const MAX_USER: usize = 256 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Region {
    pub file: String,
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct SourceSnapshot {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub user_hash: String,
    pub version: String,
    pub api_hash: String,
    pub bridge_hash: String,
    pub combined_hash: String,
    pub regions: Vec<Region>,
    pub base_scene: Option<serde_json::Value>,
    #[serde(skip)]
    pub source: String,
}
fn hash(bytes: &[u8]) -> String {
    agp::renderer::capture::sha256_hex(bytes)
}
mod assembly;
/// Shared complete transport units from Builtin6, never copies of its graph logic.
pub fn bridge() -> String {
    assembly::bridge(SOURCE).expect("trusted transport assembly")
}
/// Rename only trusted transport tokens. The user compilation unit is intact.
fn namespace(source: &str) -> Result<String, String> {
    let normalized = auto_lang_core::strip_line_block_comments(&source.replace("\r\n", "\n"));
    let source = normalized.as_str();
    let ast = Parser::from(SOURCE)
        .parse()
        .map_err(|e| format!("trusted transport AST: {e:?}"))?;
    let names: BTreeSet<_> = ast
        .stmts
        .iter()
        .filter_map(root_name)
        .filter(|n| *n != "main")
        .map(str::to_owned)
        .collect();
    let tokens = auto_lang_core::lexer::Lexer::new(source)
        .tokenize_all()
        .map_err(|e| format!("trusted transport tokens: {e:?}"))?;
    let mut result = String::new();
    let mut previous = 0;
    for token in tokens {
        if matches!(token.kind, auto_lang_core::token::TokenKind::Ident)
            && names.contains(token.text.as_str())
        {
            let start = token.pos.pos;
            let end = start + token.pos.len;
            if start < previous || source.get(start..end) != Some(token.text.as_str()) {
                return Err(format!(
                    "trusted transport token span mismatch {:?} at {}..{} got {:?}",
                    token.text,
                    start,
                    end,
                    source.get(start..end)
                ));
            }
            result.push_str(&source[previous..start]);
            result.push_str("_ag_");
            result.push_str(token.text.as_str());
            previous = end;
        }
    }
    result.push_str(&source[previous..]);
    Ok(result)
}
pub fn transport_call(name: &'static str) -> &'static str {
    match name {
        "reset" => "_ag_reset",
        "identity" => "_ag_identity",
        "visibility" => "_ag_visibility",
        "bind_resource" => "_ag_bind_resource",
        "append_field" => "_ag_append_field",
        "derive" => "_ag_derive",
        "action" => "_ag_action",
        "tick" => "_ag_tick",
        "profile" => "_ag_profile",
        "run_id" => "_ag_run_id",
        "tick_id" => "_ag_tick_id",
        "count" => "_ag_count",
        "id" => "_ag_id",
        "kind" => "_ag_kind",
        "dimension" => "_ag_dimension",
        "parent" => "_ag_parent",
        "resource" => "_ag_resource",
        "local_visible" => "_ag_local_visible",
        "visible" => "_ag_visible",
        "local" => "_ag_local",
        "field" => "_ag_field",
        other => other,
    }
}
pub fn resolve(selected: &Path, scene: Option<&Path>) -> Result<PathBuf, String> {
    if selected.is_absolute() {
        return Ok(selected.to_path_buf());
    }
    let scene = scene.ok_or("relative behavior requires a saved scene entry")?;
    let scene = if scene.is_absolute() {
        scene.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(scene)
    };
    Ok(scene.parent().ok_or("scene root missing")?.join(selected))
}
impl SourceSnapshot {
    pub fn read(path: &Path) -> Result<Self, String> {
        Self::read_for(path, Api::V1)
    }
    pub fn read_for(path: &Path, selected: Api) -> Result<Self, String> {
        let result = (|| {
            assets::regular_path(path, false)?;
            if path.extension().and_then(|s| s.to_str()) != Some("at") {
                return Err("single .at file required".into());
            }
            let file = File::open(path).map_err(|e| e.to_string())?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("regular source file required".into());
            }
            let mut bytes = Vec::new();
            file.take(MAX_USER as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            Self::from_bytes_for(path, bytes, selected)
        })();
        result.map_err(|e: String| format!("{} [read/admission]: {e}", path.display()))
    }
    pub fn from_bytes(path: &Path, bytes: Vec<u8>) -> Result<Self, String> {
        Self::from_bytes_for(path, bytes, Api::V1)
    }
    pub fn from_bytes_for(path: &Path, bytes: Vec<u8>, selected: Api) -> Result<Self, String> {
        let error = |stage: &str, e: String| format!("{} [{stage}]: {e}", path.display());
        if bytes.len() > MAX_USER {
            return Err(error("read", "source exceeds 256KiB".into()));
        }
        if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(error("read", "UTF-8 BOM is not accepted".into()));
        }
        let user = std::str::from_utf8(&bytes).map_err(|e| error("read", format!("UTF-8: {e}")))?;
        let units = assembly::assemble(selected).map_err(|e| error("internal assembly", e))?;
        let assembly::Units {
            transport,
            api: api_text,
            entry: entry_text,
            api_region,
        } = units;
        if api_text.len() + transport.len() + entry_text.len() > MAX_BRIDGE {
            return Err(error("admission", "managed source exceeds 64KiB".into()));
        }
        let managed = format!("{transport}\n{api_text}\n{entry_text}");
        admit(user, &managed).map_err(|e| error("admission", e))?;
        // All declarations/initializers precede managed main; the SDK initializes
        // globals there. Both sides are complete top-level compilation units.
        let mut source = String::new();
        let mut regions = Vec::new();
        let transport = namespace(&transport)?;
        let api = namespace(&api_text)?;
        let entry = namespace(&entry_text)?;
        for (file, text) in [
            ("<scene-transport>", transport.as_str()),
            (api_region, api.as_str()),
            (path.to_str().unwrap_or("<user.at>"), user),
            ("<behavior-bridge>", entry.as_str()),
        ] {
            let start = source.len();
            source.push_str(text);
            let end = source.len();
            regions.push(Region {
                file: file.into(),
                start,
                end,
            });
            source.push('\n');
        }
        if source.len() > MAX_USER + MAX_BRIDGE {
            return Err(error("admission", "combined source exceeds 320KiB".into()));
        }
        Ok(Self {
            path: path.into(),
            user_hash: hash(&bytes),
            bytes,
            version: selected.version().into(),
            api_hash: hash(api_text.as_bytes()),
            bridge_hash: hash(format!("{transport}{entry_text}").as_bytes()),
            combined_hash: hash(source.as_bytes()),
            regions,
            base_scene: None,
            source,
        })
    }
    pub fn diagnostic(&self, stage: &str, callback: &str, message: &str) -> String {
        // Runtime errors from this SDK have no structured source span.
        let message = bounded_diagnostic(message);
        format!(
            "{} [{stage}/{callback}; SDK supplied no mapped line]: {message}",
            self.path.display()
        )
    }
    pub fn sdk_diagnostic(&self, stage: &str, error: &auto_lang_core::error::AutoError) -> String {
        let text = bounded_diagnostic(&format!("{error:?}"));
        if let Some(offset) = error_offset(error) {
            if let Some(region) = self
                .regions
                .iter()
                .find(|r| offset >= r.start && offset < r.end)
            {
                if let Some(before) = self.source.get(region.start..offset) {
                    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
                    let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
                    return format!(
                        "{}:{line}:{column} [{stage}/module; SDK source span]: {text}",
                        region.file
                    );
                }
            }
        }
        self.diagnostic(stage, "module", &text)
    }
}
fn bounded_diagnostic(text: &str) -> String {
    if text.len() <= 8192 {
        return text.into();
    }
    let end = text
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= 8192)
        .last()
        .unwrap_or(0);
    format!("{} [diagnostic truncated at 8KiB]", &text[..end])
}
fn error_offset(error: &auto_lang_core::error::AutoError) -> Option<usize> {
    use auto_lang_core::error::{
        AutoError as E, LexerError as L, NameError as N, SyntaxError as S, TypeError as T,
    };
    match error {
        E::MsgWithSource(e) => Some(e.span.offset()),
        E::MultipleErrors { errors, .. } => errors.iter().find_map(error_offset),
        E::SyntaxWithSource(e) => error_offset(&E::Syntax(e.error.clone())),
        E::LexerWithSource(e) => error_offset(&E::Lexer(e.error.clone())),
        E::Syntax(
            S::UnexpectedToken { span, .. }
            | S::InvalidExpression { span }
            | S::UnterminatedString { span }
            | S::UnterminatedComment { span }
            | S::InvalidEscapeSequence { span, .. }
            | S::MissingDelimiter { span, .. }
            | S::Generic { span, .. },
        ) => Some(span.offset()),
        E::Lexer(
            L::UnknownEscapeSequence { span, .. }
            | L::UnterminatedChar { span }
            | L::EmptyChar { span }
            | L::InvalidIdentifierStart { span, .. }
            | L::UnknownCharacter { span, .. },
        ) => Some(span.offset()),
        E::Name(
            N::UndefinedVariable { span, .. }
            | N::DuplicateDefinition { span, .. }
            | N::ImmutableAssignment { span, .. }
            | N::UndefinedFunction { span, .. },
        ) => Some(span.offset()),
        E::Type(
            T::Mismatch { span, .. }
            | T::InvalidOperation { span, .. }
            | T::NotCallable { span, .. }
            | T::InvalidIndexType { span, .. }
            | T::InvalidArraySize { span }
            | T::FieldMismatch { span, .. }
            | T::ConstraintViolation { span, .. }
            | T::UndefinedSpec { span, .. }
            | T::CannotGetDecl { span, .. }
            | T::CannotModifyViewParam { span, .. },
        ) => Some(span.offset()),
        _ => None,
    }
}
fn root_name(s: &Stmt) -> Option<&str> {
    match s {
        Stmt::Fn(f) => Some(f.name.as_str()),
        Stmt::Store(v) => Some(v.name.as_str()),
        Stmt::TypeDecl(t) => Some(t.name.as_str()),
        Stmt::EnumDecl(t) => Some(t.name.as_str()),
        Stmt::TypeAlias(t) => Some(t.name.as_str()),
        Stmt::Alias(t) => Some(t.alias.as_str()),
        Stmt::Union(t) => Some(t.name.as_str()),
        Stmt::Tag(t) => Some(t.name.as_str()),
        _ => None,
    }
}
fn private(name: &str) -> bool {
    name.starts_with("_ag_")
}
pub fn admit(user: &str, managed: &str) -> Result<(), String> {
    let fixed = Parser::from(managed)
        .parse()
        .map_err(|e| format!("managed API parse: {e:?}"))?;
    let reserved: BTreeSet<String> = fixed
        .stmts
        .iter()
        .filter_map(root_name)
        .map(str::to_owned)
        .collect();
    let ast = Parser::from(user).parse().map_err(|e| {
        if let Some(offset) = error_offset(&e) {
            if let Some(before) = user.get(..offset) {
                return format!(
                    "parse: user line {} column {}: {e:?}",
                    before.bytes().filter(|b| *b == b'\n').count() + 1,
                    before.rsplit('\n').next().unwrap_or("").chars().count() + 1
                );
            }
        }
        format!("parse: SDK supplied no line: {e:?}")
    })?;
    let rules = Rules {
        reserved: &reserved,
        unit_callback: false,
    };
    let mut constants = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut callbacks = [false; 2];
    for s in &ast.stmts {
        // Parser line counters do not account for every multiline token. Only
        // byte spans can safely be mapped back into the unchanged user text.
        let location = if let Stmt::Fn(f) = s {
            f.span
                .and_then(|(offset, _)| user.get(..offset))
                .map(|before| {
                    format!(
                        "user line {}",
                        before.bytes().filter(|b| *b == b'\n').count() + 1
                    )
                })
        } else {
            None
        }
        .unwrap_or_else(|| "SDK supplied no mapped line".into());
        let check = (|| {
            if let Some(name) = root_name(s) {
                if !seen.insert(name.to_string()) {
                    return Err(format!("duplicate root symbol {name}"));
                }
                if reserved.contains(name) || private(name) {
                    return Err(format!("reserved root declaration {name}"));
                }
            }
            match s {
                Stmt::Fn(f) => {
                    let callback = f.name == "app_start" || f.name == "app_update";
                    if callback {
                        let update = f.name == "app_update";
                        callbacks[usize::from(update)] = true;
                        if !matches!(f.ret, Type::Void | Type::Unknown)
                            || f.params.len() != usize::from(update)
                            || !f.type_params.is_empty()
                            || !f.const_params.is_empty()
                            || (update
                                && (!matches!(f.params[0].ty, Type::Float)
                                    || f.params[0].default.is_some()
                                    || f.params[0].destructure.is_some()))
                        {
                            return Err(format!("callback signature {}", f.name));
                        }
                    }
                    function(f, &rules.in_function(callback), &BTreeSet::new())
                }
                Stmt::Store(v) => {
                    check_type(&v.ty, &rules, &BTreeSet::new())?;
                    if !constant(&v.expr, &constants) {
                        return Err("top-level initializer must be a literal or pure constant expression; move calls into app_start".into());
                    }
                    expression(&v.expr, &rules, &BTreeSet::new())?;
                    if matches!(v.kind, ast::StoreKind::Const | ast::StoreKind::Let) {
                        constants.insert(v.name.to_string());
                    }
                    Ok(())
                }
                Stmt::TypeDecl(t) => {
                    if matches!(t.kind, ast::TypeDeclKind::CType) {
                        return Err("foreign type is unsupported".into());
                    }
                    if let Some(parent) = &t.parent {
                        check_type(parent, &rules, &BTreeSet::new())?
                    }
                    for ty in &t.has {
                        check_type(ty, &rules, &BTreeSet::new())?
                    }
                    for m in &t.members {
                        binding(m.name.as_str())?;
                        check_type(&m.ty, &rules, &BTreeSet::new())?;
                        if let Some(e) = &m.value {
                            if !constant(e, &constants) {
                                return Err("type initializer requires constant".into());
                            }
                            expression(e, &rules, &BTreeSet::new())?
                        }
                    }
                    for c in &t.consts {
                        binding(c.name.as_str())?;
                        if !constant(&c.expr, &constants) {
                            return Err("associated constant requires constant expression".into());
                        }
                        expression(&c.expr, &rules, &BTreeSet::new())?
                    }
                    for f in &t.methods {
                        binding(f.name.as_str())?;
                        function(f, &rules, &BTreeSet::new())?
                    }
                    Ok(())
                }
                Stmt::EnumDecl(e) => {
                    if let ast::EnumKind::Heterogeneous { methods, .. } = &e.kind {
                        for f in methods {
                            function(f, &rules, &BTreeSet::new())?
                        }
                    }
                    Ok(())
                }
                Stmt::Tag(t) => {
                    for f in &t.methods {
                        function(f, &rules, &BTreeSet::new())?
                    }
                    Ok(())
                }
                Stmt::TypeAlias(t) => check_type(&t.target, &rules, &BTreeSet::new()),
                Stmt::Alias(t) => reference(t.target.as_str(), &rules, &BTreeSet::new()),
                Stmt::Union(t) => {
                    for f in &t.fields {
                        binding(f.name.as_str())?;
                        check_type(&f.ty, &rules, &BTreeSet::new())?
                    }
                    Ok(())
                }
                Stmt::EmptyLine(_) | Stmt::Comment(_) => Ok(()),
                _ => Err("unsupported top-level execution/import/foreign declaration".into()),
            }
        })();
        check.map_err(|e| format!("{location}: {e}"))?;
    }
    for (i, name) in ["app_start", "app_update"].iter().enumerate() {
        if !callbacks[i] {
            return Err(format!("user line 1: missing callback {name}"));
        }
    }
    Ok(())
}
/// Source-order immutable bindings only: no forward/cyclic or mutable reads.
/// This classifies AST expressions; it neither evaluates nor rewrites user code.
fn constant(e: &Expr, constants: &BTreeSet<String>) -> bool {
    let pure = |e: &Expr| constant(e, constants);
    match e {
        Expr::Int(_)
        | Expr::Uint(_)
        | Expr::I8(_)
        | Expr::U8(_)
        | Expr::I64(_)
        | Expr::U64(_)
        | Expr::Byte(_)
        | Expr::Float(..)
        | Expr::Double(..)
        | Expr::Bool(_)
        | Expr::Char(_)
        | Expr::Str(_)
        | Expr::CStr(_)
        | Expr::Nil
        | Expr::Null
        | Expr::None => true,
        Expr::Ident(n) => constants.contains(n.as_str()),
        Expr::Unary(Op::Add | Op::Sub | Op::Not | Op::Bang, v) => pure(v),
        Expr::Bina(a, op, b) => {
            matches!(
                op,
                Op::Add
                    | Op::Sub
                    | Op::Mul
                    | Op::Div
                    | Op::Mod
                    | Op::And
                    | Op::Or
                    | Op::Eq
                    | Op::Neq
                    | Op::Lt
                    | Op::Gt
                    | Op::Le
                    | Op::Ge
                    | Op::In
                    | Op::Range
                    | Op::RangeEq
                    | Op::QuestionQuestion
            ) && pure(a)
                && pure(b)
        }
        Expr::Array(v) | Expr::Tuple(v) => v.iter().all(pure),
        Expr::Object(p) => p.iter().all(|p| pure(&p.value)),
        Expr::FStr(f) => f.parts.iter().all(pure),
        Expr::Some(v) | Expr::Ok(v) | Expr::Err(v) => pure(v),
        Expr::Index(a, b) | Expr::NullCoalesce(a, b) => pure(a) && pure(b),
        Expr::Dot(v, _) => pure(v),
        Expr::Range(v) => pure(&v.start) && pure(&v.end),
        Expr::Cast { expr, target_type } | Expr::To { expr, target_type } => {
            matches!(
                target_type,
                Type::Int
                    | Type::Float
                    | Type::Bool
                    | Type::Double
                    | Type::Uint
                    | Type::I64
                    | Type::U64
                    | Type::Byte
            ) && pure(expr)
        }
        _ => false,
    }
}
/// The same traversal checks references and returns, so expression operands
/// cannot evade callback policy. A nested function/closure starts its own policy.
#[derive(Clone, Copy)]
struct Rules<'a> {
    reserved: &'a BTreeSet<String>,
    unit_callback: bool,
}
impl Rules<'_> {
    fn contains(&self, name: &str) -> bool {
        self.reserved.contains(name)
    }
    fn in_function(&self, unit_callback: bool) -> Self {
        Self {
            unit_callback,
            ..*self
        }
    }
}
fn binding(n: &str) -> Result<(), String> {
    if private(n) {
        Err(format!("private binding {n}"))
    } else {
        Ok(())
    }
}
fn reference(n: &str, r: &Rules<'_>, locals: &BTreeSet<String>) -> Result<(), String> {
    if private(n) || (r.contains(n) && !n.starts_with("ag_") && !locals.contains(n)) {
        Err(format!("reserved reference {n}"))
    } else {
        Ok(())
    }
}
fn function(f: &ast::Fn, r: &Rules<'_>, outer: &BTreeSet<String>) -> Result<(), String> {
    binding(f.name.as_str())?;
    if !matches!(
        f.kind,
        ast::FnKind::Function | ast::FnKind::Lambda | ast::FnKind::Method
    ) || !f.attrs.is_empty()
        || f.api_attrs.is_some()
        || f.export_abi.is_some()
    {
        return Err("foreign/annotated functions are unsupported".into());
    }
    check_type(&f.ret, r, outer)?;
    let mut locals = outer.clone();
    for p in &f.params {
        binding(p.name.as_str())?;
        check_type(&p.ty, r, &locals)?;
        if let Some(e) = &p.default {
            expression(e, r, &locals)?
        }
        locals.insert(p.name.to_string());
    }
    body(&f.body, r, &locals)
}
fn check_type(t: &Type, r: &Rules<'_>, l: &BTreeSet<String>) -> Result<(), String> {
    match t {
        Type::User(t) => reference(t.name.as_str(), r, l)?,
        Type::Union(t) => reference(t.name.as_str(), r, l)?,
        Type::Tag(t) => reference(t.borrow().name.as_str(), r, l)?,
        Type::Enum(t) => reference(t.borrow().name.as_str(), r, l)?,
        Type::Spec(t) => reference(t.borrow().name.as_str(), r, l)?,
        Type::List(t)
        | Type::Reference(t)
        | Type::Linear(t)
        | Type::Option(t)
        | Type::Result(t) => check_type(t, r, l)?,
        Type::Map(k, v) => {
            check_type(k, r, l)?;
            check_type(v, r, l)?
        }
        Type::Tuple(v) => {
            for t in v {
                check_type(t, r, l)?
            }
        }
        Type::Fn(p, t) => {
            for t in p {
                check_type(t, r, l)?
            }
            check_type(t, r, l)?
        }
        Type::Array(v) => check_type(&v.elem, r, l)?,
        Type::Slice(v) => check_type(&v.elem, r, l)?,
        Type::Ptr(v) => check_type(&v.of.borrow(), r, l)?,
        Type::RuntimeArray(v) => {
            check_type(&v.elem, r, l)?;
            expression(&v.size_expr, r, l)?
        }
        Type::GenericInstance(v) => {
            reference(v.base_name.as_str(), r, l)?;
            if v.source.is_some() {
                return Err("foreign generic type".into());
            }
            for t in &v.args {
                check_type(t, r, l)?
            }
        }
        Type::CStruct(_) | Type::Rust(_) | Type::Handle { .. } => {
            return Err("foreign/async type is unsupported".into())
        }
        _ => {}
    }
    Ok(())
}
fn body(b: &Body, r: &Rules<'_>, outer: &BTreeSet<String>) -> Result<(), String> {
    let mut locals = outer.clone();
    for s in &b.stmts {
        match s {
            Stmt::Store(v) => {
                binding(v.name.as_str())?;
                check_type(&v.ty, r, &locals)?;
                expression(&v.expr, r, &locals)?;
                locals.insert(v.name.to_string());
            }
            Stmt::Expr(Expr::TupleDestruct { names, expr }) => {
                expression(expr, r, &locals)?;
                for n in names {
                    binding(n.as_str())?;
                    locals.insert(n.to_string());
                }
            }
            Stmt::Expr(e) => expression(e, r, &locals)?,
            Stmt::Return(e) => {
                if r.unit_callback && !matches!(e.as_ref(), Expr::Nil | Expr::Null) {
                    return Err("callback must return Unit".into());
                }
                expression(e, r, &locals)?;
            }
            Stmt::Block(b) => body(b, r, &locals)?,
            Stmt::If(v) => conditional(v, r, &locals)?,
            Stmt::Node(n) => expression(&Expr::Node(n.clone()), r, &locals)?,
            Stmt::For(f) => {
                let mut l = locals.clone();
                if let Some(init) = &f.init {
                    body(
                        &Body {
                            stmts: vec![*init.clone()],
                            ..Body::new()
                        },
                        r,
                        &l,
                    )?;
                    // The initializer's bindings belong to the loop condition
                    // and body, not to the surrounding block.
                    match init.as_ref() {
                        Stmt::Store(v) => {
                            l.insert(v.name.to_string());
                        }
                        Stmt::Expr(Expr::TupleDestruct { names, .. }) => {
                            l.extend(names.iter().map(ToString::to_string));
                        }
                        _ => {}
                    }
                }
                expression(&f.range, r, &l)?;
                match &f.iter {
                    ast::Iter::Named(n) => {
                        binding(n.as_str())?;
                        l.insert(n.to_string());
                    }
                    ast::Iter::Indexed(a, b) | ast::Iter::Destructured(a, b) => {
                        binding(a.as_str())?;
                        binding(b.as_str())?;
                        l.insert(a.to_string());
                        l.insert(b.to_string());
                    }
                    ast::Iter::Ever | ast::Iter::Cond => {}
                    ast::Iter::Call(c) => {
                        expression(&Expr::Call(c.clone()), r, &l)?;
                    }
                }
                body(&f.body, r, &l)?;
            }
            Stmt::Fn(f) => {
                if r.contains(f.name.as_str()) || private(f.name.as_str()) {
                    return Err("reserved nested function".into());
                }
                function(f, &r.in_function(false), &locals)?;
            }
            Stmt::Try(t) => {
                body(&t.body, r, &locals)?;
                let mut l = locals.clone();
                if let Some(n) = &t.catch_param {
                    binding(n)?;
                    l.insert(n.clone());
                }
                body(&t.catch_body, r, &l)?;
                if let Some(b) = &t.finally_body {
                    body(b, r, &locals)?
                }
            }
            Stmt::Is(v) => matching(v, r, &locals)?,
            Stmt::Break | Stmt::Continue | Stmt::EmptyLine(_) | Stmt::Comment(_) => {}
            _ => return Err("unsupported nested import/async/foreign declaration".into()),
        }
    }
    Ok(())
}
fn conditional(v: &ast::If, r: &Rules<'_>, l: &BTreeSet<String>) -> Result<(), String> {
    for b in &v.branches {
        expression(&b.cond, r, l)?;
        body(&b.body, r, l)?
    }
    if let Some(b) = &v.else_ {
        body(b, r, l)?
    }
    Ok(())
}
fn matching(v: &ast::Is, r: &Rules<'_>, l: &BTreeSet<String>) -> Result<(), String> {
    expression(&v.target, r, l)?;
    for b in &v.branches {
        match b {
            ast::IsBranch::EqBranch(v, b) => {
                let mut ls = l.clone();
                for e in v {
                    expression(e, r, l)?;
                    pattern_bindings(e, &mut ls)?
                }
                body(b, r, &ls)?
            }
            ast::IsBranch::IfBranch(e, b) => {
                expression(e, r, l)?;
                body(b, r, l)?
            }
            ast::IsBranch::ElseBranch(b) => body(b, r, l)?,
        }
    }
    Ok(())
}
fn pattern_bindings(e: &Expr, l: &mut BTreeSet<String>) -> Result<(), String> {
    let mut add = |n: &str| {
        binding(n)?;
        l.insert(n.to_string());
        Ok::<_, String>(())
    };
    match e {
        Expr::Cover(ast::Cover::Tag(c)) => {
            for n in &c.bindings {
                add(n.as_str())?
            }
        }
        Expr::OptionPattern(p) => {
            if let Some(n) = &p.binding {
                add(n.as_str())?
            }
        }
        Expr::ResultPattern(p) => {
            if let Some(n) = &p.binding {
                add(n.as_str())?
            }
            if let Some(e) = &p.inner {
                pattern_bindings(e, l)?
            }
        }
        Expr::StructPattern(p) => {
            for f in &p.fields {
                add(f.binding.as_str())?
            }
        }
        _ => {}
    }
    Ok(())
}
fn expression(e: &Expr, r: &Rules<'_>, l: &BTreeSet<String>) -> Result<(), String> {
    match e {
        Expr::Ident(n) | Expr::GenName(n) | Expr::Ref(n) => reference(n.as_str(), r, l)?,
        Expr::Call(c) => {
            if let Expr::Ident(n) = c.name.as_ref() {
                if r.contains(n.as_str()) && !n.starts_with("ag_") {
                    return Err(format!("reserved transport call {n}"));
                }
            }
            expression(&c.name, r, l)?;
            for a in &c.args.args {
                match a {
                    Arg::Pos(e) | Arg::Pair(_, e) => expression(e, r, l)?,
                    Arg::Name(n) => reference(n.as_str(), r, l)?,
                }
            }
        }
        Expr::Unary(_, v)
        | Expr::View(v)
        | Expr::Mut(v)
        | Expr::Move(v)
        | Expr::Take(v)
        | Expr::Some(v)
        | Expr::Ok(v)
        | Expr::Err(v)
        | Expr::BoxExpr(v)
        | Expr::ArcExpr(v)
        | Expr::ErrorPropagate(v) => expression(v, r, l)?,
        Expr::Bina(a, _, b) | Expr::Index(a, b) | Expr::NullCoalesce(a, b) => {
            expression(a, r, l)?;
            expression(b, r, l)?
        }
        Expr::Dot(v, n) => {
            expression(v, r, l)?;
            if private(n.as_str()) {
                return Err("private member access".into());
            }
        }
        Expr::Range(v) => {
            expression(&v.start, r, l)?;
            expression(&v.end, r, l)?
        }
        Expr::Array(v) | Expr::Tuple(v) => {
            for e in v {
                expression(e, r, l)?
            }
        }
        Expr::Object(v) => {
            for p in v {
                expression(&p.value, r, l)?
            }
        }
        Expr::Pair(p) => expression(&p.value, r, l)?,
        Expr::Block(b) => body(b, r, l)?,
        Expr::If(v) => conditional(v, r, l)?,
        Expr::Is(v) => matching(v, r, l)?,
        Expr::Lambda(f) => function(f, &r.in_function(false), l)?,
        Expr::Closure(c) => {
            let own_rules = r.in_function(false);
            let mut ls = l.clone();
            for p in &c.params {
                binding(p.name.as_str())?;
                if let Some(t) = &p.ty {
                    check_type(t, &own_rules, &ls)?;
                }
                ls.insert(p.name.to_string());
            }
            if let Some(t) = &c.ret {
                check_type(t, &own_rules, &ls)?;
            }
            expression(&c.body, &own_rules, &ls)?
        }
        Expr::FStr(f) => {
            for e in &f.parts {
                expression(e, r, l)?
            }
        }
        Expr::Cast { expr, target_type } | Expr::To { expr, target_type } => {
            check_type(target_type, r, l)?;
            expression(expr, r, l)?
        }
        Expr::Node(n) => {
            reference(n.name.as_str(), r, l)?;
            for arg in &n.args.args {
                match arg {
                    Arg::Pos(e) | Arg::Pair(_, e) => expression(e, r, l)?,
                    Arg::Name(n) => reference(n.as_str(), r, l)?,
                }
            }
            body(&n.body, r, l)?
        }
        Expr::Grid(g) => {
            for arg in &g.head.args {
                match arg {
                    Arg::Pos(e) | Arg::Pair(_, e) => expression(e, r, l)?,
                    Arg::Name(n) => reference(n.as_str(), r, l)?,
                }
            }
            for row in &g.data {
                for e in row {
                    expression(e, r, l)?
                }
            }
        }
        Expr::TupleDestruct { names, expr } => {
            for n in names {
                binding(n.as_str())?
            }
            expression(expr, r, l)?
        }
        Expr::Cover(ast::Cover::Tag(c)) => {
            reference(c.kind.as_str(), r, l)?;
            binding(c.tag.as_str())?;
            for n in &c.bindings {
                binding(n.as_str())?
            }
        }
        Expr::StructPattern(p) => {
            reference(p.type_name.as_str(), r, l)?;
            for f in &p.fields {
                binding(f.field.as_str())?;
                binding(f.binding.as_str())?
            }
        }
        Expr::OptionPattern(p) => {
            if let Some(n) = &p.binding {
                binding(n.as_str())?
            }
        }
        Expr::ResultPattern(p) => {
            if let Some(n) = &p.binding {
                binding(n.as_str())?
            }
            if let Some(e) = &p.inner {
                expression(e, r, l)?
            }
        }
        Expr::Uncover(p) => {
            reference(p.src.as_str(), r, l)?;
            binding(p.binding.as_str())?;
            reference(p.cover.kind.as_str(), r, l)?
        }
        Expr::OptionUncover(p) => {
            reference(p.src.as_str(), r, l)?;
            binding(p.binding.as_str())?
        }
        Expr::ResultUncover(p) => {
            reference(p.src.as_str(), r, l)?;
            binding(p.binding.as_str())?
        }
        Expr::Hold(h) => {
            expression(&h.path, r, l)?;
            binding(h.name.as_str())?;
            let mut ls = l.clone();
            ls.insert(h.name.to_string());
            body(&h.body, r, &ls)?
        }
        Expr::Int(_)
        | Expr::Uint(_)
        | Expr::I8(_)
        | Expr::U8(_)
        | Expr::I64(_)
        | Expr::U64(_)
        | Expr::Byte(_)
        | Expr::Float(..)
        | Expr::Double(..)
        | Expr::Bool(_)
        | Expr::Char(_)
        | Expr::Str(_)
        | Expr::CStr(_)
        | Expr::Nil
        | Expr::Null
        | Expr::None => {}
        _ => return Err("unsupported async/compile-time/foreign expression".into()),
    }
    Ok(())
}
