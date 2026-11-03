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
/// PLAN-053 T-04：算法唯一实现点迁至 compiler（包身份语义同址——runtime
/// 侧目录注入也消费，不能反向依赖 tools）；本函数保留 re-export 形态。
pub fn theme_package_content_digest(
    dir: &std::path::Path,
) -> Result<String, ThemePackageError> {
    auto_ui_compiler::design_tokens::theme_package::theme_package_content_digest(dir)
}
/// PLAN-031 T-04/T-07（V-09）：解析 pac `dep themes { path: … }` 的**原始**路径
/// （未校验）。None = 该 App 未声明官方包依赖。
pub fn app_theme_dep_path(app_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let pac = std::fs::read_to_string(app_dir.join("pac.at")).ok()?;
    let mut in_dep = false;
    let mut depth = 0i32;
    let mut rel: Option<String> = None;
    for raw in pac.lines() {
        let line = raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !in_dep {
            if line.starts_with("dep") && line.contains("themes") {
                in_dep = true;
                depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            }
            continue;
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        if let Some(rest) = line.strip_prefix("path:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !v.is_empty() {
                rel = Some(v);
            }
        }
        if depth <= 0 {
            break;
        }
    }
    let rel = rel?;
    Some(app_dir.join(rel))
}

/// PLAN-031 T-04/T-07（V-09）：从 App 目录解析 pac `dep themes { path: … }`
/// 并返回**校验通过**的官方包根。未声明 = None（该 App 不消费官方包）；
/// 声明了但校验失败（错版/缺件/篡改）同样返回 None——调用方据
/// [`app_theme_dep_path`] 区分「未声明」与「声明但非法」。
pub fn official_theme_package_root_for_app(
    app_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let root = app_theme_dep_path(app_dir)?.canonicalize().ok()?;
    verify_official_theme_package(&root).ok().map(|_| root)
}

/// PLAN-031 T-04/T-07（V-09）：把官方包的**身份 + 内容摘要**写成生成输入
/// receipt（`<out>/.theme-inputs.json`）。该文件是「本次生成消费了哪个包内容」
/// 的可审计凭证：内容变化（即使 mtime 不变）会让摘要变化，缓存/落地方可据此
/// 判定新鲜度，而不是靠 mtime 或主题名。
pub fn write_theme_inputs_receipt(
    out_dir: &std::path::Path,
    app_dir: &std::path::Path,
) -> Result<String, ThemePackageError> {
    let root = official_theme_package_root_for_app(app_dir)
        .ok_or_else(|| ThemePackageError::MissingManifest(out_dir.join("theme-package.json")))?;
    let digest = theme_package_content_digest(&root)?;
    let body = format!(
        "{{\n  \"schema\": 1,\n  \"package\": {{\n    \"owner\": \"{}\",\n    \"name\": \"{}\",\n    \"version\": \"{}\",\n    \"digest\": \"{}\"\n  }}\n}}\n",
        OFFICIAL_THEME_PACKAGE_OWNER, OFFICIAL_THEME_PACKAGE_NAME, OFFICIAL_THEME_PACKAGE_VERSION, digest
    );
    let path = out_dir.join(".theme-inputs.json");
    std::fs::write(&path, body)
        .map_err(|_| ThemePackageError::MissingFile(".theme-inputs.json".into()))?;
    Ok(digest)
}

/// 读回生成输入 receipt 的摘要（无 receipt = None）。
pub fn read_theme_inputs_receipt(out_dir: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(out_dir.join(".theme-inputs.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("package")?
        .get("digest")?
        .as_str()
        .map(|s| s.to_string())
}
/// PLAN-031 T-04/T-07（V-09）：生成准入——App 声明了官方包就必须校验通过；
/// 未声明返回 Ok(None)；声明但非法（错版/缺件/篡改）返回具名 Err，调用方
/// 必须中止生成（不发半成品，C-11）。
pub fn verify_app_theme_inputs(
    app_dir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ThemePackageError> {
    let Some(raw) = app_theme_dep_path(app_dir) else {
        return Ok(None);
    };
    let root = raw
        .canonicalize()
        .map_err(|_| ThemePackageError::MissingManifest(raw.join("theme-package.json")))?;
    verify_official_theme_package(&root)?;
    Ok(Some(root))
}
/// 单文件 sha256（hex）——工具链更新清单/校验时共用。
pub fn theme_file_sha256(path: &std::path::Path) -> Result<String, ThemePackageError> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)
        .map_err(|_| ThemePackageError::MissingFile(path.display().to_string()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(format!("{:x}", h.finalize()))
}