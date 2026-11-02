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
