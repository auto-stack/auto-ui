// PLAN-981（B-SESSION-I）：AGE typed 宿主的**唯一实现**（被 bridge crate 与
// driver 通过 `include!` 共同编译；两边只在依赖图上不同——live ACE vs 快照内
// vendored ACE——从而不产生两份会漂移的逻辑）。
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
    /// R63-3: keep the open-time scenario ops so a body update re-evaluates with
    /// the SAME schedule (a 5-Tick session must not silently become 0-Tick).
    ops: Vec<ace_core::source::age_component::ScenarioOp>,
    pid: u64,
    generation: u64,
}

static NEXT_PID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl AgeSession {
    pub fn open(src: &str, instances: usize) -> Result<Self, Vec<String>> {
        Self::open_with(src, instances, Vec::new())
    }

    pub fn open_with(
        src: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<Self, Vec<String>> {
        let compiled = Self::compile(src, instances, ops.clone())?;
        Ok(Self {
            compiled,
            instances,
            ops,
            pid: NEXT_PID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            generation: 0,
        })
    }

    /// 同 PID 的 body 更新：成功替换并递增 generation；失败保留旧代。
    pub fn update_body(&mut self, src: &str) -> Result<u64, Vec<String>> {
        // R63-3: reuse the open-time scenario ops (not an empty schedule).
        let next = Self::compile(src, self.instances, self.ops.clone())?;
        self.compiled = next;
        self.generation += 1;
        Ok(self.generation)
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    fn compile(
        src: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<AgePackageCompilation, Vec<String>> {
        let scenario = ComponentScenario { instances, ops };
        let inputs = [PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }];
        compile_age_package_with_scenario(
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
        })
    }

    pub fn compiled(&self) -> &AgePackageCompilation {
        &self.compiled
    }

    pub fn exit(&self) -> AgeViewExit {
        self.compiled
            .description
            .components
            .first()
            .map(|c| c.exit)
            .unwrap_or(AgeViewExit::View)
    }

    pub fn lifecycle(&self, ticks: u32, frames: u32) -> Result<AgeLifecycleBudget, String> {
        lifecycle_budget(ticks, frames, self.compiled.description.components.len())
            .map_err(|e| e.message)
    }

    pub fn observe(&self) -> Result<ObservedAgePackage, String> {
        ace_av::age_package::observe_age_package(&self.compiled, self.instances)
    }
}
