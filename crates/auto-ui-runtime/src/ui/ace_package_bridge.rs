//! PLAN-980 ④-A 首片 #2b：**AGE 包闭包的桌面桥引擎**。
//!
//! 组件面引擎（`AceStateEngine`）只吃单源；本引擎把"编译单元集合 + 入口"作为
//! 输入，dispatch 时按场景（实例数 + 定向 op）重编译，再用
//! `ace_av::age_package::observe_age_package` 取 AV 真值——与 ④-A 首片 #2a 的
//! AV 观察同一条 `lower → MIR verify → ABC → AV` 链，不引入第二解释器。
//!
//! 首版边界（具名）：只观察 `int/i32` model 字段（`observe_age_package` 的
//! 状态面即该域）；包面场景**无 initials 面**，非空 props 种子具名拒；
//! 带载荷 dispatch（`SendArgs`）本片未接线（具名拒）。
use ace_av::age_component::{ComponentScenario, InitialValue, LegacyWidgetAlias, ScenarioOp};
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

use crate::ui::ace_state_bridge::{AceStateValue, DispatchOutcome};

/// 一个实例槽（keyed identity + 该实例的定向 op 历史）。
struct Slot {
    identity: String,
    live: bool,
    generation: u64,
    ops: Vec<ScenarioOp>,
}

/// AGE 包闭包的状态执行引擎（每 engine = 一个包工程；实例 = keyed 槽）。
pub struct PackageStateEngine {
    units: Vec<PackageModuleInput>,
    entry: String,
    alias: LegacyWidgetAlias,
    slots: Vec<Slot>,
    next_generation: u64,
}

impl PackageStateEngine {
    /// 以一次试验编译（1 实例、零 op）校验工程形状；错误带具名诊断码。
    pub fn new(
        units: Vec<PackageModuleInput>,
        entry: &str,
        alias: LegacyWidgetAlias,
    ) -> Result<Self, String> {
        // PLAN-105 T-02：包闭包单元统一 f-string 脱糖（组件面同一边界）。
        let units = crate::ui::ace_package_units::desugar_units(units)?;
        let engine = Self {
            units,
            entry: entry.to_string(),
            alias,
            slots: Vec::new(),
            next_generation: 1,
        };
        engine.compile(1, Vec::new())?;
        Ok(engine)
    }

    /// 试验/重放共用：按场景编译一次包闭包。
    fn compile(
        &self,
        instances: usize,
        ops: Vec<ScenarioOp>,
    ) -> Result<ace_core::source::age_package::AgePackageCompilation, String> {
        let caps = [PACKAGES_CAPABILITY.to_string()];
        let scenario = ComponentScenario { instances, ops };
        compile_age_package_with_scenario(&self.units, &self.entry, self.alias, &caps, &scenario)
            .map_err(|ds| {
                ds.first()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .unwrap_or_else(|| "package scene refused".to_string())
            })
    }

    fn slot(&self, identity: &str) -> Option<usize> {
        self.slots.iter().position(|s| s.identity == identity)
    }

    /// 挂载 keyed 实例。包面场景**没有** props/initials 面：非空种子具名拒
    /// （不静默丢种子、也不假装已承接）。
    pub fn mount(
        &mut self,
        identity: &str,
        initials: Vec<(String, InitialValue)>,
    ) -> Result<u64, String> {
        if !initials.is_empty() {
            return Err(
                "the AGE package scenario has no props/initials surface yet; seed the model \
                 in source (package initials are a named pending surface)"
                    .to_string(),
            );
        }
        if let Some(i) = self.slot(identity) {
            if self.slots[i].live {
                return Err(format!("instance `{identity}` is already mounted"));
            }
        }
        let generation = self.next_generation;
        self.next_generation += 1;
        match self.slot(identity) {
            Some(i) => {
                let s = &mut self.slots[i];
                s.live = true;
                s.generation = generation;
                s.ops.clear();
            }
            None => self.slots.push(Slot {
                identity: identity.to_string(),
                live: true,
                generation,
                ops: Vec::new(),
            }),
        }
        Ok(generation)
    }

    /// 逐次重放：所有存活槽的 op 按槽序扁平化后编译场景，返回该实例的状态行
    /// （字段声明序；只含 int 字段——[`observe_age_package`] 的状态面）。
    pub fn read_all(&self, identity: &str) -> Option<Vec<(String, AceStateValue)>> {
        let inst = self.slot(identity)?;
        if !self.slots[inst].live {
            return None;
        }
        let ops = self.flatten_ops();
        let compiled = self.compile(self.slots.len().max(1), ops).ok()?;
        self.states_for(&compiled, inst)
    }

    /// 该实例的文本叶值（AV 观察面）。
    pub fn texts(&self, identity: &str) -> Option<Vec<String>> {
        let inst = self.slot(identity)?;
        if !self.slots[inst].live {
            return None;
        }
        let ops = self.flatten_ops();
        let compiled = self.compile(self.slots.len().max(1), ops).ok()?;
        let observed =
            ace_av::age_package::observe_age_package(&compiled, self.slots.len().max(1)).ok()?;
        observed
            .components
            .first()
            .and_then(|c| c.instance_texts.get(inst).cloned())
    }

    /// 定向 unit 消息：追加 op → 重编译 → 观察（失败即具名拒，槽不被毒化：
    /// op 先入临时表，编译成功才提交）。
    pub fn dispatch(&mut self, identity: &str, msg: &str) -> Result<DispatchOutcome, String> {
        let inst = self
            .slot(identity)
            .ok_or_else(|| format!("instance `{identity}` is not mounted"))?;
        if !self.slots[inst].live {
            return Err(format!("instance `{identity}` is released"));
        }
        let mut ops = self.flatten_ops();
        ops.push(ScenarioOp::Send {
            instance: inst,
            msg: msg.to_string(),
        });
        let compiled = self.compile(self.slots.len().max(1), ops.clone())?;
        let states = self
            .states_for(&compiled, inst)
            .ok_or_else(|| "package observation produced no state row".to_string())?;
        // 只有在整条观察链成功后才把 op 提交进槽。
        self.slots[inst].ops.push(ScenarioOp::Send {
            instance: inst,
            msg: msg.to_string(),
        });
        Ok(DispatchOutcome::Delivered(states))
    }

    /// 本片未接线的载荷形：具名拒（不静默丢参数）。
    pub fn dispatch_with_args(
        &mut self,
        _identity: &str,
        _msg: &str,
        _args: &[AceStateValue],
    ) -> Result<DispatchOutcome, String> {
        Err(
            "typed payload dispatch is not wired on the AGE package bridge yet (unit messages \
             only in this slice)"
                .to_string(),
        )
    }

    fn flatten_ops(&self) -> Vec<ScenarioOp> {
        self.slots
            .iter()
            .filter(|s| s.live)
            .flat_map(|s| s.ops.iter().cloned())
            .collect()
    }

    fn states_for(
        &self,
        compiled: &ace_core::source::age_package::AgePackageCompilation,
        inst: usize,
    ) -> Option<Vec<(String, AceStateValue)>> {
        let observed =
            ace_av::age_package::observe_age_package(compiled, self.slots.len().max(1)).ok()?;
        let component = observed.components.first()?;
        let fields: Vec<(String, String)> = compiled.description.components.first()?.fields.clone();
        let int_fields: Vec<&str> = fields
            .iter()
            .filter(|(_, ty)| ty == "int" || ty == "i32")
            .map(|(n, _)| n.as_str())
            .collect();
        let row = component.instance_states.get(inst)?;
        Some(
            int_fields
                .iter()
                .zip(row.iter())
                .map(|(name, value)| ((*name).to_string(), AceStateValue::Int(*value)))
                .collect(),
        )
    }
}
