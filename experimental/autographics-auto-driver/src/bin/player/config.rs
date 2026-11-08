//! PLAN-072 versioned player launch description (`player.json`).
//!
//! The file lives beside the exe; every path it names is resolved relative to
//! the exe directory, never the current directory. Unknown fields and unknown
//! schema versions are rejected before any window or GPU exists, so a bad
//! release layout fails with a named error instead of a blank run.
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const SCHEMA: &str = "age-game-player/1";
pub const PROFILE_2D: &str = "components2d";
pub const PROFILE_3D: &str = "components3d";
const MIN_SIZE: u32 = 320;
const MAX_SIZE: u32 = 3840;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerConfig {
    pub schema: String,
    pub game_id: String,
    pub scene: String,
    pub profile: String,
    pub display: Display,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Display {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl PlayerConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("player.json unreadable: {e}"))?;
        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("player.json invalid: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err(format!(
                "player.json schema {} unsupported (expected {SCHEMA})",
                self.schema
            ));
        }
        if self.game_id.is_empty() || self.game_id.len() > 64 {
            return Err("player.json game_id must be 1..=64 bytes".into());
        }
        let scene = Path::new(&self.scene);
        if !scene.is_relative()
            || self.scene.contains("..")
            || scene.extension().and_then(|e| e.to_str()) != Some("agscene")
        {
            return Err(format!(
                "player.json scene {:?} must be a relative .agscene path under the exe directory without '..'",
                self.scene
            ));
        }
        if self.profile != PROFILE_2D && self.profile != PROFILE_3D {
            return Err(format!(
                "player.json profile {:?} unsupported (expected {PROFILE_2D} or {PROFILE_3D})",
                self.profile
            ));
        }
        if self.display.title.is_empty() || self.display.title.len() > 120 {
            return Err("player.json display.title must be 1..=120 bytes".into());
        }
        let (w, h) = (self.display.width, self.display.height);
        if !(MIN_SIZE..=MAX_SIZE).contains(&w) || !(MIN_SIZE..=MAX_SIZE).contains(&h) {
            return Err(format!(
                "player.json display size {w}x{h} outside {MIN_SIZE}..={MAX_SIZE}"
            ));
        }
        Ok(())
    }

    pub fn two_d(&self) -> bool {
        self.profile == PROFILE_2D
    }

    /// The scene path under the exe root; the root itself comes from
    /// `current_exe`, never from the working directory.
    pub fn scene_path(&self, root: &Path) -> Result<PathBuf, String> {
        let path = root.join(&self.scene);
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("scene {:?} missing under exe root: {e}", self.scene))?;
        let root = root
            .canonicalize()
            .map_err(|e| format!("exe root missing: {e}"))?;
        if !canonical.starts_with(&root) {
            return Err("scene escapes the exe root".into());
        }
        Ok(canonical)
    }
}

/// The scene's project API must match the configured profile before a Host
/// exists; a mismatched pair is a named startup rejection, never a fallback.
pub fn profile_api(profile: &str) -> &'static str {
    if profile == PROFILE_2D {
        autographics_auto_driver::authoring::behavior::COMPONENTS2_VERSION
    } else {
        autographics_auto_driver::authoring::behavior::COMPONENTS3_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> serde_json::Value {
        serde_json::json!({
            "schema": SCHEMA,
            "game_id": "platformer-2d",
            "scene": "data/scene.agscene",
            "profile": PROFILE_2D,
            "display": {"title": "Platformer 2D", "width": 1600, "height": 760}
        })
    }

    #[test]
    fn valid_config_parses_and_resolves_relative_scene() {
        let config: PlayerConfig =
            serde_json::from_value(base()).expect("valid config");
        config.validate().expect("validated");
        assert!(config.two_d());
        let root = std::env::temp_dir().join("age-player-config-root");
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(root.join("data/scene.agscene"), b"{}").unwrap();
        let resolved = config.scene_path(&root).expect("resolved");
        assert!(resolved.starts_with(root.canonicalize().unwrap()));
    }

    #[test]
    fn every_bad_field_is_named_rejection() {
        let rejects = [
            ("schema", serde_json::json!("age-game-player/2")),
            ("game_id", serde_json::json!("")),
            ("scene", serde_json::json!("../escape.agscene")),
            ("scene", serde_json::json!("C:/abs/scene.agscene")),
            ("scene", serde_json::json!("data/scene.json")),
            ("profile", serde_json::json!("components4d")),
            ("display.title", serde_json::json!("")),
            ("display.width", serde_json::json!(100)),
        ];
        for (field, value) in rejects {
            let mut raw = base();
            let target = if field.starts_with("display.") {
                &mut raw["display"][&field[8..]]
            } else {
                &mut raw[field]
            };
            *target = value;
            let config: Result<PlayerConfig, _> = serde_json::from_value(raw.clone());
            let error = match config {
                Ok(c) => c.validate().unwrap_err(),
                Err(e) => e.to_string(),
            };
            assert!(!error.is_empty(), "{field} must be rejected");
        }
        // Unknown fields are rejected outright: the versioned contract has no
        // silently ignored extension surface.
        let mut raw = base();
        raw["extra"] = serde_json::json!(1);
        assert!(serde_json::from_value::<PlayerConfig>(raw).is_err());
    }
}
