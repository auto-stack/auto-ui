//! PLAN-031 T-04：正式主题包受管来源校验（V-08 的「获得→校验→真实数据」腿）。
//!
//! 输入 = 一个主题包目录（含 `theme-package.json` + `themes.at` + `pac.at`）。
//! 校验 owner/package/version/schema、文件清单只在包内、逐文件 sha256、
//! 主题数据经真实解析且 family 声明的模式可用。任何不符都返回具名错误，
//! 调用方不得继续使用该包（不允许静默回落到另一来源）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::theme_tree::{self, ThemeTree};

/// 期望的包身份（外部获得固定版本时由调用方给出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePackageExpectation<'a> {
    pub owner: &'a str,
    pub package: &'a str,
    pub version: &'a str,
    pub schema: u64,
}

/// 校验通过的主题包。
#[derive(Debug, Clone)]
pub struct VerifiedThemePackage {
    pub root: PathBuf,
    pub owner: String,
    pub package: String,
    pub version: String,
    pub schema: u64,
    /// family 名 → 声明的模式列表（manifest 能力表；数据一致性已校验）。
    pub families: BTreeMap<String, Vec<String>>,
    /// 经真实主题解析得到的数据（消费方直接使用）。
    pub trees: BTreeMap<String, ThemeTree>,
}

/// 具名校验错误（错版/缺件/等长篡改/逃逸路径/数据不一致）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemePackageError {
    MissingManifest(PathBuf),
    BadManifest(String),
    FieldMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    PathEscape(String),
    MissingFile(String),
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    BadThemeData(String),
    ModeMismatch {
        family: String,
        mode: String,
    },
}

impl std::fmt::Display for ThemePackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifest(p) => write!(f, "主题包清单缺失：{}", p.display()),
            Self::BadManifest(m) => write!(f, "主题包清单非法：{m}"),
            Self::FieldMismatch {
                field,
                expected,
                actual,
            } => write!(f, "主题包字段 `{field}` 不符：期望 `{expected}`，实际 `{actual}`"),
            Self::PathEscape(p) => write!(f, "主题包清单路径逃出包目录：`{p}`"),
            Self::MissingFile(p) => write!(f, "主题包文件缺失：`{p}`"),
            Self::HashMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "主题包文件 `{path}` sha256 不符：期望 `{expected}`，实际 `{actual}`"
            ),
            Self::BadThemeData(m) => write!(f, "主题数据解析/校验失败：{m}"),
            Self::ModeMismatch { family, mode } => {
                write!(f, "主题族 `{family}` 声明支持 {mode}，但数据未声明该模式")
            }
        }
    }
}

impl std::error::Error for ThemePackageError {}

fn field<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

/// 校验一个主题包目录并返回真实数据。
pub fn verify_theme_package(
    dir: &Path,
    expect: &ThemePackageExpectation<'_>,
) -> Result<VerifiedThemePackage, ThemePackageError> {
    let manifest_path = dir.join("theme-package.json");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|_| ThemePackageError::MissingManifest(manifest_path.clone()))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ThemePackageError::BadManifest(format!("json: {e}")))?;

    for (field_name, expected) in [
        ("owner", expect.owner),
        ("package", expect.package),
        ("version", expect.version),
    ] {
        let actual = field(&v, field_name);
        if actual != expected {
            return Err(ThemePackageError::FieldMismatch {
                field: match field_name {
                    "owner" => "owner",
                    "package" => "package",
                    _ => "version",
                },
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
    }
    let schema = v.get("schema").and_then(|x| x.as_u64()).unwrap_or(0);
    if schema != expect.schema {
        return Err(ThemePackageError::FieldMismatch {
            field: "schema",
            expected: expect.schema.to_string(),
            actual: schema.to_string(),
        });
    }

    // 文件清单：只在包内 + sha256 逐文件核验。
    let files = v
        .get("files")
        .and_then(|x| x.as_array())
        .ok_or_else(|| ThemePackageError::BadManifest("files 必须是非空数组".into()))?;
    if files.is_empty() {
        return Err(ThemePackageError::BadManifest("files 不得为空".into()));
    }
    for f in files {
        let rel = field(f, "path");
        if rel.is_empty() || rel.contains("..") || rel.starts_with('/') || rel.contains('\\') {
            return Err(ThemePackageError::PathEscape(rel.to_string()));
        }
        let bytes = std::fs::read(dir.join(rel))
            .map_err(|_| ThemePackageError::MissingFile(rel.to_string()))?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        let expected = field(f, "sha256");
        if actual != expected {
            return Err(ThemePackageError::HashMismatch {
                path: rel.to_string(),
                expected: expected.to_string(),
                actual,
            });
        }
    }

    // 真实主题数据：经生产解析路径加载（不是清单空口）。
    let themes_at = dir.join("themes.at");
    let src = std::fs::read_to_string(&themes_at)
        .map_err(|_| ThemePackageError::MissingFile("themes.at".into()))?;
    let trees = theme_tree::collect_named_themes(&themes_at, &src)
        .map_err(|e| ThemePackageError::BadThemeData(e.to_string()))?;

    let families_json = v
        .get("families")
        .and_then(|x| x.as_array())
        .ok_or_else(|| ThemePackageError::BadManifest("families 必须是非空数组".into()))?;
    let mut families = BTreeMap::new();
    for fam in families_json {
        let name = field(fam, "name").to_string();
        let modes: Vec<String> = fam
            .get("modes")
            .and_then(|m| m.as_array())
            .map(|a| a.iter().filter_map(|m| m.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let tree = trees
            .get(&name)
            .ok_or_else(|| ThemePackageError::BadThemeData(format!("family `{name}` 缺失")))?;
        for mode in &modes {
            let dark = match mode.as_str() {
                "light" => false,
                "dark" => true,
                other => {
                    return Err(ThemePackageError::BadThemeData(format!("未知模式 `{other}`")))
                }
            };
            if !theme_tree::mode_available(tree, dark) {
                return Err(ThemePackageError::ModeMismatch {
                    family: name.clone(),
                    mode: mode.clone(),
                });
            }
        }
        families.insert(name, modes);
    }

    Ok(VerifiedThemePackage {
        root: dir.to_path_buf(),
        owner: field(&v, "owner").to_string(),
        package: field(&v, "package").to_string(),
        version: field(&v, "version").to_string(),
        schema,
        families,
        trees,
    })
}
