//! PLAN-981（B-SESSION-I，S1）：AGE Session 的 **typed 宿主**桥接层。
//!
//! 让 AGE 作者源经 ACE 的唯一受检语义 + AV 执行链跑起来，替代 Session 里旧的
//! `AutoVM` 路径。本 crate 只是**窄适配层**，不复制语法/语义/求值：
//!
//! - 编译：`ace_core::source::age_package::compile_age_package_with_scenario`
//!   （包模块面 ∪ AGE 面一次解析，共享绑定/verify）；
//! - 观察：`ace_av::age_package::observe_age_package`（按 def id 走
//!   lower → MIR verify → ABC → AV 执行，拿真实文本/状态值）；
//! - 生命周期计数：`ace_core::source::age_package::lifecycle_budget`（LANGUAGE
//!   冻结的 oracle：Start=1 / Tick=N / Frame=M，不随出口倍增）。
//!
//! 与 driver 的关系：driver（`autographics-auto-driver`）当前依赖冻结 SDK
//! `.sdk-source`；本 crate 依赖 **live** auto-lang 计划树（组内兄弟仓相对路径）。
//! 两者同名同版本但路径不同，Cargo 在同一构建图里会报 package collision——
//! 因此 driver 的接线（S2）以 SDK 刷新（978）为前提，本 crate 先独立成型并被
//! SESSION 的 V 用它自己的测试驱动。不建 junction/symlink。

use ace_core::source::age_component::ComponentScenario;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package_with_scenario, lifecycle_budget, AgeLifecycleBudget, AgePackageCompilation,
    AgeViewExit,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

pub use ace_av::age_package::{ObservedAgeComponent, ObservedAgePackage};

/// 一个已打开（已受检）的 AGE Session。
pub struct AgeSession {
    compiled: AgePackageCompilation,
    instances: usize,
}

impl AgeSession {
    /// 打开一个单模块（`app::main`）的 AGE 源。
    ///
    /// `instances` 是场景实例数（与 `ComponentScenario` 的实例数一致）；0 表示
    /// 只编译不实例化。错误以 `code: message` 形式返回，保留原诊断码。
    pub fn open(src: &str, instances: usize) -> Result<Self, Vec<String>> {
        Self::open_with(src, instances, Vec::new())
    }

    /// 同 `open`，但带定向事件（`ops`）——Tick/消息由场景驱动，宿主不手写玩法。
    pub fn open_with(
        src: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<Self, Vec<String>> {
        let scenario = ComponentScenario { instances, ops };
        let inputs = [PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }];
        let compiled = compile_age_package_with_scenario(
            &inputs,
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &scenario,
        )
        .map_err(|ds| {
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
        })?;
        Ok(Self { compiled, instances })
    }

    /// 受检编译结果（只读：消费者按 def id/描述面取事实）。
    pub fn compiled(&self) -> &AgePackageCompilation {
        &self.compiled
    }

    /// 该 Session 主组件的呈现出口（AGE-L03 的五出口之一）。
    pub fn exit(&self) -> AgeViewExit {
        self.compiled
            .description
            .components
            .first()
            .map(|c| c.exit)
            .unwrap_or(AgeViewExit::View)
    }

    /// 生命周期 oracle：给定 Tick/Frame 调度次数，返回期望计数。
    ///
    /// 出口数只做合法性检查（0 出口拒绝），**不参与计数**——这就是
    /// "Frame/Tick 不随 view 倍增"的机械保证。
    pub fn lifecycle(&self, ticks: u32, frames: u32) -> Result<AgeLifecycleBudget, String> {
        lifecycle_budget(ticks, frames, self.compiled.description.components.len())
            .map_err(|e| e.message)
    }

    /// 按实例观察真实值（文本叶 / int 状态 / handler 名字）。
    pub fn observe(&self) -> Result<ObservedAgePackage, String> {
        ace_av::age_package::observe_age_package(&self.compiled, self.instances)
    }
}