//! PLAN-031 T-04：UI tools 的正式主题包输入入口。
//!
//! 工具链（构建/安装/发行）只经本模块校验主题包：owner/版本/schema/文件
//! hash/族模式由 compiler 校验器给出具名结果；失败即拒绝该包，不静默回落到
//! 另一个来源。官方包身份固定在下方常量，版本升级须与 `packages/themes`
//! 的 pac/manifest 同步。

pub use auto_ui_compiler::design_tokens::theme_package::{
    theme_package_identity, verify_theme_package, ThemePackageError, ThemePackageExpectation,
    VerifiedThemePackage,
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
    // PLAN-036 T-02（V-07）：快照目标必须为空——非空目录可能属于未知来源或
    // 活动负载，绝不静默合并/覆盖（发布原语不得破坏已有合法目标；换装由
    // 调用方以独立 staging+swap 完成）。
    if let Ok(mut entries) = std::fs::read_dir(dest) {
        if entries.next().is_some() {
            return Err(ThemePackageError::BadManifest(format!(
                "快照目标目录非空，拒绝覆盖未知内容（清空或换独立 staging 目录后重试）：{}",
                dest.display()
            )));
        }
    }
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
/// PLAN-036 R-01/R-02 修正（final review 2026-11-03）：官方包**声明判定改为
/// 权威解析面**——用 `auto_project_model::parse_file` 解析 pac.at，遍历全部
/// dep（名无关、Bare/Quoted/单行/多行形态同权）：任何 dep 的解析目标目录含
/// `theme-package.json` 即为官方包声明（校验通过→消费，失败→具名 Err）。
/// 行扫描的「恰名 themes」语法窄于模块系统的真实消费语法（dep_roots 名无关
/// 消费），曾使 `dep themesX`/`dep "themes"` 声明的非法包被静默消费——本函数
/// 是唯一声明判定入口，不依赖 dep 名。多个 dep 同时指向主题包目录 = 具名
/// 歧义错误。未声明 = Ok(None)（旧行为：零 UI 资产、零 receipt、零 front-input 键）。
fn parsed_theme_declaration(
    app_dir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ThemePackageError> {
    let pac_path = app_dir.join("pac.at");
    if !pac_path.is_file() {
        return Ok(None);
    }
    let doc = auto_project_model::parse_file(&pac_path).map_err(|e| {
        ThemePackageError::BadManifest(format!("pac.at 解析失败（{}）：{e}", pac_path.display()))
    })?;
    let mut declared: Option<std::path::PathBuf> = None;
    for dep in &doc.deps {
        let Some(rel) = &dep.path else {
            continue; // 无 path 的 dep 无法解析目录，不构成主题包声明
        };
        let dir = app_dir.join(rel);
        if !dir.join("theme-package.json").is_file() {
            continue; // 普通依赖目录，非主题包声明
        }
        if declared.is_some() {
            return Err(ThemePackageError::BadManifest(
                "多个主题包声明（多个 dep 指向含 theme-package.json 的目录）——保留恰好一个"
                    .into(),
            ));
        }
        declared = Some(dir);
    }
    Ok(declared)
}

/// PLAN-031 T-04/T-07（V-09）+ PLAN-036 R-01 修正：从 App 目录判定官方包
/// 声明并返回**校验通过**的包根。未声明 = Ok(None)（该 App 不消费官方包，
/// 旧行为）；声明了但解析/校验失败（错版/缺件/篡改/不可达）= 具名 Err——
/// 调用方必须中止生成（C-11），绝不静默回落或当未声明。
pub fn official_theme_package_root_for_app(
    app_dir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ThemePackageError> {
    let Some(dir) = parsed_theme_declaration(app_dir)? else {
        return Ok(None);
    };
    let root = dir.canonicalize().map_err(|_| {
        ThemePackageError::MissingManifest(dir.join("theme-package.json"))
    })?;
    verify_official_theme_package(&root)?;
    Ok(Some(root))
}

/// PLAN-036 T-02：统一已验证主题输入身份——所有消费轨（Vue/VM/Rust/delivery）
/// 共用的「本次消费了哪个包」描述。`content_digest` 为冻结算法（OS053 兼容）；
/// `identity_digest` 额外纳入清单身份/能力元数据（见 compiler
/// [`theme_package_identity`]），能力变化即使同版本同内容也必然失效。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePackageInputIdentity {
    pub owner: String,
    pub name: String,
    pub version: String,
    pub schema: u64,
    pub content_digest: String,
    pub identity_digest: String,
}

/// 校验包并给出统一输入身份（verify + 内容摘要 + 语义身份，三者必须同根）。
pub fn verified_theme_input_identity(
    root: &std::path::Path,
) -> Result<ThemePackageInputIdentity, ThemePackageError> {
    let verified = verify_official_theme_package(root)?;
    let content_digest = theme_package_content_digest(root)?;
    let identity_digest = theme_package_identity(root)?;
    Ok(ThemePackageInputIdentity {
        owner: verified.owner,
        name: verified.package,
        version: verified.version,
        schema: verified.schema,
        content_digest,
        identity_digest,
    })
}

/// PLAN-031 T-04/T-07（V-09）+ PLAN-036 T-02：把官方包的**统一输入身份**写成
/// 生成输入 receipt（`<out>/.theme-inputs.json`，含内容摘要与语义身份）。
/// 返回 `None` = App 未声明官方包（零 UI 资产、旧行为，不写文件）；
/// `Err` = 声明了但非法（错版/缺件/篡改，具名错误）或 receipt 写入失败
/// （消费失败，调用方必须中止，不得警告后继续）。
pub fn write_theme_inputs_receipt(
    out_dir: &std::path::Path,
    app_dir: &std::path::Path,
) -> Result<Option<ThemePackageInputIdentity>, ThemePackageError> {
    let identity = resolve_theme_input_identity(app_dir)?;
    let Some(identity) = identity else {
        return Ok(None);
    };
    let body = format!(
        "{{\n  \"schema\": 1,\n  \"package\": {{\n    \"owner\": \"{}\",\n    \"name\": \"{}\",\n    \"version\": \"{}\",\n    \"digest\": \"{}\",\n    \"identity\": \"{}\"\n  }}\n}}\n",
        identity.owner,
        identity.name,
        identity.version,
        identity.content_digest,
        identity.identity_digest
    );
    let path = out_dir.join(".theme-inputs.json");
    std::fs::write(&path, body)
        .map_err(|_| ThemePackageError::MissingFile(".theme-inputs.json".into()))?;
    Ok(Some(identity))
}

/// PLAN-036 R-03：解析 App 当前应消费的统一输入身份（不写任何文件）——
/// 准入冻结与 receipt 写点共用的解析面；未声明 = Ok(None)。
pub fn resolve_theme_input_identity(
    app_dir: &std::path::Path,
) -> Result<Option<ThemePackageInputIdentity>, ThemePackageError> {
    let Some(root) = verify_app_theme_inputs(app_dir)? else {
        return Ok(None);
    };
    Ok(Some(verified_theme_input_identity(&root)?))
}

/// 读回生成输入 receipt 的内容摘要（无 receipt = None）。
pub fn read_theme_inputs_receipt(out_dir: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(out_dir.join(".theme-inputs.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("package")?
        .get("digest")?
        .as_str()
        .map(|s| s.to_string())
}

/// PLAN-036 T-02：读回 receipt 的语义身份摘要（旧 receipt 无此字段 = None，
/// 调用方必须按不新鲜处理并安全重验，绝不当 fresh）。
pub fn read_theme_inputs_identity(out_dir: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(out_dir.join(".theme-inputs.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("package")?
        .get("identity")?
        .as_str()
        .map(|s| s.to_string())
}
/// PLAN-031 T-04/T-07（V-09）+ PLAN-036 R-01 修正：生成准入——App 声明了
/// 官方包就必须校验通过（声明判定=已解析 dep 集，见
/// [`official_theme_package_root_for_app`]）；未声明返回 Ok(None)；声明但
/// 非法（错版/缺件/篡改/不可达/歧义）返回具名 Err，调用方必须中止生成
/// （不发半成品，C-11）。
pub fn verify_app_theme_inputs(
    app_dir: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ThemePackageError> {
    official_theme_package_root_for_app(app_dir)
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

/// PLAN-036 T-04（AC-01/V-01）+ R-03：全部 Vue 生成/构建/run 入口共用的准入——
/// 声明官方包就必须校验通过；非法具名中止（不发半成品，C-11）。返回的
/// 统一输入身份供 receipt 写点做**冻结-比对**（R-03：生成期间合法换包 →
/// 身份漂移 → receipt 置扣，不签新收据给旧产物）；未声明 = Ok(None)。
pub(crate) fn admit_generation_theme_inputs(
    app_dir: &std::path::Path,
) -> Result<Option<ThemePackageInputIdentity>, String> {
    match verify_app_theme_inputs(app_dir) {
        Ok(None) => Ok(None),
        Ok(Some(root)) => verified_theme_input_identity(&root)
            .map(Some)
            .map_err(|e| {
                format!(
                    "主题包生成准入失败（{}）：{e}——修正 packages/themes 的清单/hash 后重试",
                    app_dir.display()
                )
            }),
        Err(e) => Err(format!(
            "主题包生成准入失败（{}）：{e}——修正 packages/themes 的清单/hash 后重试",
            app_dir.display()
        )),
    }
}