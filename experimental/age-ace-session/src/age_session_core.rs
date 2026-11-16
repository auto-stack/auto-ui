// PLAN-981（B-SESSION-I）：AGE typed 宿主的**唯一实现**（被 bridge crate 与
// driver 通过 `include!` 共同编译；两边只在依赖图上不同——live ACE vs 快照内
// vendored ACE——从而不产生两份会漂移的逻辑）。
use ace_core::source::age_component::ComponentScenario;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package_with_scenario, lifecycle_budget, select_exit, AgeLifecycleBudget,
    AgePackageCompilation, AgeSelectionError, AgeViewExit, AgeViewRequest,
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
    /// R63-1：Session 保存最近一次受检源，供 restart 与增量调度复用。
    source: String,
    /// R63-1：stop 后的最后观察值（停止期间不受编辑/调度影响）。
    last_observed: Option<ObservedAgePackage>,
    stopped: bool,
    pid: u64,
    generation: u64,
}

static NEXT_PID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl AgeSession {
    /// `instances = 0` 表示"只编译、按默认单实例观察"（`observe()` 仍给 1 个实例
    /// 的值），不是"没有实例可观察"；需要多实例时给 >0 的实例数。
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
            source: src.to_string(),
            last_observed: None,
            stopped: false,
            pid: NEXT_PID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            generation: 0,
        })
    }

    /// 同 PID 的 body 更新：成功替换并递增 generation；失败保留旧代。
    pub fn update_body(&mut self, src: &str) -> Result<u64, Vec<String>> {
        if self.stopped {
            return Err(vec![
                "source.session.stopped: restart the session before editing".into(),
            ]);
        }
        // R63-3: reuse the open-time scenario ops (not an empty schedule).
        let next = Self::compile(src, self.instances, self.ops.clone())?;
        self.compiled = next;
        self.source = src.to_string();
        self.generation += 1;
        Ok(self.generation)
    }

    // ---- R63-1：SESSION I 的调度/回调/资源/停止恢复/布局义务 ----

    /// 追加 `n` 次 Tick 并重跑受检场景（进程内重编译 + AV 执行，无工具链）。
    pub fn tick(&mut self, n: u32) -> Result<u64, Vec<String>> {
        self.schedule(
            (0..n)
                .map(|_| ace_core::source::age_component::ScenarioOp::Tick { instance: 0 })
                .collect(),
        )
    }

    /// 投递一条作者消息（`Send`），触发对应 `on <Msg>` 回调。
    pub fn send(&mut self, msg: &str) -> Result<u64, Vec<String>> {
        self.schedule(vec![ace_core::source::age_component::ScenarioOp::Send {
            instance: 0,
            msg: msg.to_string(),
        }])
    }

    fn schedule(
        &mut self,
        extra: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<u64, Vec<String>> {
        if self.stopped {
            return Err(vec![
                "source.session.stopped: restart the session before scheduling".into(),
            ]);
        }
        // R64-1: the request must be atomic — a rejected dispatch may not poison
        // the session's schedule (compile the tentative ops first, commit after).
        let mut next_ops = self.ops.clone();
        next_ops.extend(extra);
        let next = Self::compile(&self.source, self.instances, next_ops.clone())?;
        self.ops = next_ops;
        self.compiled = next;
        self.generation += 1;
        Ok(self.generation)
    }

    /// 已声明的资源行（端口/模块/名字），按声明序——资源由共享语义判定。
    pub fn resources(&self) -> Vec<(String, String, String)> {
        self.compiled
            .description
            .resources
            .iter()
            .map(|r| (r.name.clone(), r.port.clone(), r.module.clone()))
            .collect()
    }

    /// 用已声明的出口回答一个呈现请求（AGE-L04 选择序；不兼容布局具名拒）。
    pub fn request_layout(
        &self,
        request: AgeViewRequest,
    ) -> Result<AgeViewExit, AgeSelectionError> {
        let declared: Vec<AgeViewExit> = self
            .compiled
            .description
            .components
            .first()
            .map(|c| c.exits.clone())
            .unwrap_or_default();
        select_exit(&declared, request)
    }

    /// 停止：冻结最后观察值（停止期间的调度/编辑都被拒）。
    pub fn stop(&mut self) -> Result<(), String> {
        if self.stopped {
            return Ok(());
        }
        self.last_observed = Some(self.observe()?);
        self.stopped = true;
        Ok(())
    }

    /// 恢复：同一 PID 重新启动（Start 再次为 1），generation 递增。
    pub fn restart(&mut self) -> Result<u64, Vec<String>> {
        let next = Self::compile(&self.source, self.instances, Vec::new())?;
        self.compiled = next;
        self.ops.clear();
        self.stopped = false;
        self.last_observed = None;
        self.generation += 1;
        Ok(self.generation)
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// 停止期间返回冻结值；运行中按 AV 重新观察。
    pub fn observe(&self) -> Result<ObservedAgePackage, String> {
        if let Some(last) = &self.last_observed {
            return Ok(last.clone());
        }
        ace_av::age_package::observe_age_package(&self.compiled, self.instances)
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

    /// 主出口（首个声明），保留兼容；多出口请用 [`Self::exits`]。
    pub fn exit(&self) -> AgeViewExit {
        self.compiled
            .description
            .components
            .first()
            .map(|c| c.exit)
            .unwrap_or(AgeViewExit::View)
    }

    // ---- R64-3：多组件宿主面（组件选择器/逐组件观察与布局） ----

    /// 本 Session 的组件数（按声明序）。
    pub fn component_count(&self) -> usize {
        self.compiled.description.components.len()
    }

    /// 组件名（按声明序）。
    pub fn component_names(&self) -> Vec<String> {
        self.compiled
            .description
            .components
            .iter()
            .map(|c| c.name.clone())
            .collect()
    }

    /// 逐组件观察（`None` 表示越界，不静默取首组件）。
    pub fn observe_component(&self, index: usize) -> Result<ObservedAgeComponent, String> {
        let all = self.observe()?;
        all.components
            .into_iter()
            .nth(index)
            .ok_or_else(|| format!("no component at index {index}"))
    }

    /// 指定组件的已声明出口。
    pub fn exits_of(&self, index: usize) -> Result<Vec<AgeViewExit>, String> {
        self.compiled
            .description
            .components
            .get(index)
            .map(|c| c.exits.clone())
            .ok_or_else(|| format!("no component at index {index}"))
    }

    /// 用指定组件的出口回答布局请求（选择序/不兼容拒绝与单组件一致）。
    pub fn request_layout_for(
        &self,
        index: usize,
        request: AgeViewRequest,
    ) -> Result<AgeViewExit, AgeSelectionError> {
        let declared = self.exits_of(index).unwrap_or_default();
        select_exit(&declared, request)
    }

    /// 只给某个实例追加 Tick（双组件 2:1 事件归属）。
    pub fn tick_instance(&mut self, instance: usize, n: u32) -> Result<u64, Vec<String>> {
        self.schedule(
            (0..n)
                .map(|_| ace_core::source::age_component::ScenarioOp::Tick { instance })
                .collect(),
        )
    }

    /// 只给某个实例投递消息。
    pub fn send_to(&mut self, instance: usize, msg: &str) -> Result<u64, Vec<String>> {
        self.schedule(vec![ace_core::source::age_component::ScenarioOp::Send {
            instance,
            msg: msg.to_string(),
        }])
    }
    /// 该 Session 主组件声明的全部出口（按声明序，去重）。
    pub fn exits(&self) -> Vec<AgeViewExit> {
        self.compiled
            .description
            .components
            .first()
            .map(|c| c.exits.clone())
            .unwrap_or_default()
    }

    pub fn lifecycle(&self, ticks: u32, frames: u32) -> Result<AgeLifecycleBudget, String> {
        // R63-8: 第三参是**出口数**（不是组件数）——这样"0 出口"在宿主 API 上也
        // 会命中 oracle 的具名拒绝，而不是永远传 ≥1。
        let exits = self
            .compiled
            .description
            .components
            .first()
            .map(|c| c.exits.len())
            .unwrap_or(0);
        lifecycle_budget(ticks, frames, exits).map_err(|e| e.message)
    }
}
