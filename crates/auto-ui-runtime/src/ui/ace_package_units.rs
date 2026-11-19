//! PLAN-105 T-02（U03）：AGE 包闭包单元的 **widget f-string 统一脱糖**入口。
//!
//! 传统 AutoUI 源（012/013/016）的文本叶用 `f"…${.x}…"`；ACE 包前门只吃
//! 脱糖后的 `__fstr(...)` 调用（PLAN-964/980 边界）。组件面已由
//! `ace_core::source::age_component` 门面统一脱糖；包面/包片段车道此前只脱
//! **entry** 单元，其余单元里的 f-string 会在 verify 期具名拒。本模块把
//! 「单元集合 → 脱糖后单元集合」收成一处，供 `PackageStateEngine`、
//! `generate_package_sfc`、`PackageFragmentLane` 共用（单一实现，不各写一遍）。
//!
//! 边界：脱糖是源文本重写（同 PLAN-964）——f-string 内部 span 可能移位，
//! 外部字节保持；对无 f-string 的源是逐字节 no-op。
use ace_core::source::packages::PackageModuleInput;

/// 把每个单元的源文本过一次 `desugar_widget_source`（反引号 + `f"` 两遍）。
pub fn desugar_units(
    units: Vec<PackageModuleInput>,
) -> Result<Vec<PackageModuleInput>, String> {
    units
        .into_iter()
        .map(|mut u| {
            u.source = ace_core::source::fstr::desugar_widget_source(&u.source)
                .map_err(|m| format!("package unit `{}::{}` f-string: {m}", u.package, u.id))?;
            Ok(u)
        })
        .collect()
}
