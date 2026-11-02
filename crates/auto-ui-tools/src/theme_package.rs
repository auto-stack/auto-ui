//! PLAN-031 T-04：UI tools 的正式主题包输入入口。
//!
//! 工具链（构建/安装/发行）只经本模块校验主题包：owner/版本/schema/文件
//! hash/族模式由 compiler 校验器给出具名结果；失败即拒绝该包，不静默回落到
//! 另一个来源。官方包身份固定在下方常量，版本升级须与 `packages/themes`
//! 的 pac/manifest 同步。

pub use auto_ui_compiler::design_tokens::theme_package::{
    verify_theme_package, ThemePackageError, ThemePackageExpectation, VerifiedThemePackage,
};

/// 官方主题包身份（工具链固定期望）。
pub const OFFICIAL_THEME_PACKAGE_OWNER: &str = "auto-ui";
pub const OFFICIAL_THEME_PACKAGE_NAME: &str = "themes";
pub const OFFICIAL_THEME_PACKAGE_VERSION: &str = "0.2.0";
pub const OFFICIAL_THEME_PACKAGE_SCHEMA: u64 = 1;

/// 校验官方主题包目录（工具链默认入口；具名拒绝）。
pub fn verify_official_theme_package(
    dir: &std::path::Path,
) -> Result<VerifiedThemePackage, ThemePackageError> {
    verify_theme_package(
        dir,
        &ThemePackageExpectation {
            owner: OFFICIAL_THEME_PACKAGE_OWNER,
            package: OFFICIAL_THEME_PACKAGE_NAME,
            version: OFFICIAL_THEME_PACKAGE_VERSION,
            schema: OFFICIAL_THEME_PACKAGE_SCHEMA,
        },
    )
}

/// 发行快照：校验源包后，只把**清单列出**的文件 + 清单本身复制到独立根
/// （如消费根 `vendor/themes`），再对快照做一次同标准校验。源包不合法
/// （错版/缺件/篡改）时整体拒绝且不写快照——发行物不可编辑、不是第二权威源。
pub fn publish_theme_package_snapshot(
    src: &std::path::Path,
    dest: &std::path::Path,
) -> Result<VerifiedThemePackage, ThemePackageError> {
    // 先校验源包（失败零写入）。
    verify_official_theme_package(src)?;
    let manifest_src = src.join("theme-package.json");
    let text = std::fs::read_to_string(&manifest_src)
        .map_err(|_| ThemePackageError::MissingManifest(manifest_src.clone()))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ThemePackageError::BadManifest(format!("json: {e}")))?;
    let files = v
        .get("files")
        .and_then(|x| x.as_array())
        .ok_or_else(|| ThemePackageError::BadManifest("files 必须是非空数组".into()))?;

    std::fs::create_dir_all(dest).map_err(|e| {
        ThemePackageError::BadManifest(format!("快照目录创建失败 {}: {e}", dest.display()))
    })?;
    std::fs::copy(&manifest_src, dest.join("theme-package.json")).map_err(|e| {
        ThemePackageError::BadManifest(format!("清单复制失败: {e}"))
    })?;
    for f in files {
        let rel = f.get("path").and_then(|x| x.as_str()).unwrap_or("");
        if rel.is_empty() || rel.contains("..") || rel.starts_with('/') || rel.contains('\\') {
            return Err(ThemePackageError::PathEscape(rel.to_string()));
        }
        std::fs::copy(src.join(rel), dest.join(rel))
            .map_err(|_| ThemePackageError::MissingFile(rel.to_string()))?;
    }

    // 快照必须是可独立校验的发行输入（不依赖源根）。
    verify_official_theme_package(dest)
}

/// 主题包内容摘要：清单列出文件（按路径排序）的 sha256，供生成/缓存 receipt
/// 失效使用。摘要与绝对根无关（同一包移动后不变），文件内容变化即变化。
pub fn theme_package_content_digest(
    dir: &std::path::Path,
) -> Result<String, ThemePackageError> {
    use sha2::{Digest, Sha256};
    let manifest = dir.join("theme-package.json");
    let text = std::fs::read_to_string(&manifest)
        .map_err(|_| ThemePackageError::MissingManifest(manifest.clone()))?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ThemePackageError::BadManifest(format!("json: {e}")))?;
    let files = v
        .get("files")
        .and_then(|x| x.as_array())
        .ok_or_else(|| ThemePackageError::BadManifest("files 必须是非空数组".into()))?;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for f in files {
        let rel = f.get("path").and_then(|x| x.as_str()).unwrap_or("");
        if rel.is_empty() || rel.contains("..") || rel.starts_with('/') || rel.contains('\\') {
            return Err(ThemePackageError::PathEscape(rel.to_string()));
        }
        let bytes = std::fs::read(dir.join(rel))
            .map_err(|_| ThemePackageError::MissingFile(rel.to_string()))?;
        entries.push((rel.to_string(), bytes));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (path, bytes) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0u8]);
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}