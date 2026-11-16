//! PLAN-981（B-SESSION-I，S2）：AGE 源的 typed 宿主适配（driver 内嵌）。
//!
//! 这块代码从 plan-981 的独立桥接 crate 迁入 driver：driver 只有一个依赖图
//! （`.sdk-source` 快照 + 其中的 ACE），因此 AGE 适配不能再以第二个 crate 的
//! 形式引入（否则 auto-atom 会在锁文件里出现两份）。语义仍是"窄适配"：
//!
//! - 编译：`ace_core::source::age_package::compile_age_package_with_scenario`
//!   （包模块面 ∪ AGE 面一次解析，共享 bind/verify）；
//! - 观察：`ace_av::age_package::observe_age_package`（lower → MIR verify →
//!   ABC → AV 执行），返回值是真实状态/文本；
//! - 生命周期：`lifecycle_budget`（Start=1 / Tick=N / Frame=M，不随出口倍增）；
//! - 同 PID body 更新：成功才递增 `generation`，失败保留旧代。
//!
//! 不复制语法/语义/求值，不手写 Rust 玩法，也不执行 cargo/rustc。
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
        let compiled = Self::compile(src, instances, ops)?;
        Ok(Self {
            compiled,
            instances,
            pid: NEXT_PID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            generation: 0,
        })
    }

    /// 同 PID 的 body 更新：成功替换并递增 generation；失败保留旧代。
    pub fn update_body(&mut self, src: &str) -> Result<u64, Vec<String>> {
        let next = Self::compile(src, self.instances, Vec::new())?;
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
