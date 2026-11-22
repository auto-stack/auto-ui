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
    /// PLAN-1075（公共 AV 会话接线）：多模块作者闭包。Gallery 来源是
    /// `use <id>: …` 的模块集（如 g01 的 resources/shapes/main）；单模块会话
    /// 恒为 `[("main", source)]`。`source` 对闭包会话是各模块按序拼接
    /// （仅供 send_args 诊断 anchor；模块真身在本表，按 id 编辑）。
    modules: Vec<(String, String)>,
    /// PLAN-1075：出口模块 id（编译出口 `app::<entry>`；单模块会话恒 "main"）。
    entry: String,
    /// R63-1：stop 后的最后观察值（停止期间不受编辑/调度影响）。
    last_observed: Option<ObservedAgePackage>,
    stopped: bool,
    /// 已应用的调度次数（tick/send；被拒的调度不计数）。
    dispatches: u64,
    /// body 代数（成功 update_body / restart 递增；与调度计数分开，避免混用）。
    body_generation: u64,
    /// R64-10：Run/Stop 计数——多轮 Run/Stop 之后必须净归零（runs == stops）。
    runs: u64,
    stops: u64,
    /// PLAN-981 B-W1（R64-10 异步面）：异步票账面（spawned / settled）——
    /// 多轮 Run/Stop 后必须净归零（spawned == settled，无悬挂票）。
    async_spawned: u64,
    async_settled: u64,
    active: bool,
    pid: u64,
}

/// PLAN-981 B-W1（R64-10 异步面）：AGE 会话的异步票（spawn 时钉住身份）。
#[derive(Clone, Debug)]
pub struct AgeAsyncTicket {
    pub pid: u64,
    pub generation: u64,
    pub completion_msg: String,
    pub task_src: String,
}

/// 异步等待结果：完成（回写后的调度计数）或具名丢弃原因。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgeAsyncOutcome {
    Completed(u64),
    Dropped(&'static str),
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
            modules: vec![("main".into(), src.to_string())],
            entry: "main".into(),
            last_observed: None,
            stopped: false,
            pid: NEXT_PID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            dispatches: 0,
            body_generation: 0,
            runs: 1,
            stops: 0,
            async_spawned: 0,
            async_settled: 0,
            active: true,
        })
    }

    /// PLAN-1075（公共 AV 会话接线）：按**多模块作者闭包**打开 Gallery 来源。
    /// `modules` = `(模块 id, 源文本)`（Gallery 各变体闭包见
    /// `<project>/generated/<variant>/closure/`，id = 文件名去扩展）；
    /// `entry` = 出口模块 id（编译出口 `app::<entry>`；基础变体通常是
    /// `main`，变体闭包以其变体名入口，如 g01 的 `v-color`）。模块 id 不得
    /// 重复、entry 必须在闭包内；诊断路径与单模块会话一致（共享 verify）。
    pub fn open_modules(
        modules: &[(String, String)],
        entry: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<Self, Vec<String>> {
        Self::validate_modules(modules, entry)?;
        let compiled = Self::compile_modules(modules, entry, instances, ops.clone())?;
        let source = modules
            .iter()
            .map(|(_, src)| src.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        Ok(Self {
            compiled,
            instances,
            ops,
            source,
            modules: modules.to_vec(),
            entry: entry.to_string(),
            last_observed: None,
            stopped: false,
            pid: NEXT_PID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            dispatches: 0,
            body_generation: 0,
            runs: 1,
            stops: 0,
            async_spawned: 0,
            async_settled: 0,
            active: true,
        })
    }

    /// PLAN-1075：闭包会话的**单模块**热更新（同 PID，坏编辑保旧）。
    /// 未知名具名拒；编译失败保留上一好代与全部账面（generation 不动）。
    pub fn update_module(&mut self, id: &str, src: &str) -> Result<u64, Vec<String>> {
        if self.stopped {
            return Err(vec![
                "source.session.stopped: restart the session before editing".into(),
            ]);
        }
        let Some(index) = self.modules.iter().position(|(mid, _)| mid == id) else {
            return Err(vec![format!(
                "source.session.module-unknown: {id} (declared: {})",
                self.modules
                    .iter()
                    .map(|(mid, _)| mid.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )]);
        };
        // R63-3: reuse the open-time scenario ops (not an empty schedule).
        let mut next_modules = self.modules.clone();
        next_modules[index].1 = src.to_string();
        let next =
            Self::compile_modules(&next_modules, &self.entry, self.instances, self.ops.clone())?;
        self.compiled = next;
        self.modules = next_modules;
        self.source = self
            .modules
            .iter()
            .map(|(_, src)| src.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        self.body_generation += 1;
        Ok(self.body_generation)
    }

    /// PLAN-1075：闭包会话的模块表（id 序，供宿主对账来源身份）。
    pub fn module_ids(&self) -> Vec<String> {
        self.modules.iter().map(|(id, _)| id.clone()).collect()
    }

    fn validate_modules(modules: &[(String, String)], entry: &str) -> Result<(), Vec<String>> {
        if modules.is_empty() {
            return Err(vec!["source.session.modules-empty".into()]);
        }
        let mut seen: Vec<&str> = Vec::new();
        for (id, _) in modules {
            if seen.contains(&id.as_str()) {
                return Err(vec![format!("source.session.module-duplicate: {id}")]);
            }
            seen.push(id);
        }
        if !seen.contains(&entry) {
            return Err(vec![format!(
                "source.session.module-entry-missing: closure has no `{entry}` module (declared: {})",
                seen.join(", ")
            )]);
        }
        Ok(())
    }

    /// 同 PID 的 body 更新：成功替换并递增 generation；失败保留旧代。
    /// （闭包会话等价于 `update_module("main", src)`——只换入口模块。）
    pub fn update_body(&mut self, src: &str) -> Result<u64, Vec<String>> {
        self.update_module("main", src)
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

    /// PLAN-981 B-W1（IF-981-EVENT-PAYLOAD-B）：带载荷投递——typed 值抬升为
    /// 字面量候选表达式，与作者源同走唯一 verify（声明→typed 参数→一次求值、
    /// 按位绑定）。载荷变体必须带值派发（空 `send` 是 arity 域错误）；错
    /// type/arity 在效果前拒且不毒化调度（R64-1 原子性沿旧路径）。宿主值没有
    /// 作者 span——anchor 取整源区间（arity/unsupported 诊断本就指向变体声明）。
    pub fn send_args(
        &mut self,
        msg: &str,
        args: &[ace_core::source::age_component::InitialValue],
    ) -> Result<u64, Vec<String>> {
        self.send_args_to(0, msg, args)
    }

    /// 同 [`Self::send_args`]，定向到指定实例。
    pub fn send_args_to(
        &mut self,
        instance: usize,
        msg: &str,
        args: &[ace_core::source::age_component::InitialValue],
    ) -> Result<u64, Vec<String>> {
        let anchor = ace_core::atom_text::Span {
            start: 0,
            end: self.source.len(),
        };
        self.schedule(vec![
            ace_core::source::age_component::ScenarioOp::send_args_from_values(
                instance, msg, args, anchor,
            ),
        ])
    }

    /// PLAN-981 B-W1（D-981-SESSION-MULTI ①余量）：**组件级**定向——把 unit
    /// 消息发给包内第 `node` 个组件的 `instance` 实例（`ForNode` 场景扩展）。
    pub fn send_to_component(
        &mut self,
        node: usize,
        instance: usize,
        msg: &str,
    ) -> Result<u64, Vec<String>> {
        self.schedule(vec![ace_core::source::age_component::ScenarioOp::ForNode {
            node,
            op: Box::new(ace_core::source::age_component::ScenarioOp::Send {
                instance,
                msg: msg.to_string(),
            }),
        }])
    }

    /// PLAN-981 B-W1：组件级**带载荷**定向（同 [`Self::send_args_to`] 的语义，
    /// 加组件限定）。
    pub fn send_args_to_component(
        &mut self,
        node: usize,
        instance: usize,
        msg: &str,
        args: &[ace_core::source::age_component::InitialValue],
    ) -> Result<u64, Vec<String>> {
        let anchor = ace_core::atom_text::Span {
            start: 0,
            end: self.source.len(),
        };
        self.schedule(vec![ace_core::source::age_component::ScenarioOp::ForNode {
            node,
            op: Box::new(
                ace_core::source::age_component::ScenarioOp::send_args_from_values(
                    instance, msg, args, anchor,
                ),
            ),
        }])
    }

    /// PLAN-981 B-W1（R64-10 异步面）：AGE 异步票——完成消息 + 任务源，钉住
    /// `(pid, body_generation)`（与 bridge 的 AsyncTicket 同一代际纪律）。
    pub fn spawn_async(
        &mut self,
        completion_msg: &str,
        task_src: &str,
    ) -> Result<AgeAsyncTicket, Vec<String>> {
        if self.stopped {
            return Err(vec![
                "source.session.stopped: restart the session before scheduling".into(),
            ]);
        }
        self.async_spawned += 1;
        Ok(AgeAsyncTicket {
            pid: self.pid,
            generation: self.body_generation,
            completion_msg: completion_msg.to_string(),
            task_src: task_src.to_string(),
        })
    }

    /// 等待异步完成：任务源先在 ACE scheduler **真实执行**到静止（968 面），
    /// 随后按代际纪律回写——只有未停止、同 pid 且同 body 世代的会话收到
    /// 完成消息；过期票具名丢弃（计数入 settled，不悬挂）。
    pub fn await_async(&mut self, ticket: AgeAsyncTicket) -> Result<AgeAsyncOutcome, Vec<String>> {
        // 真实异步执行先行（无论回写是否会被丢弃，任务本身必须真实跑过）。
        let run = ace_av::scheduler_run::run_task_source(
            &ticket.task_src,
            &format!("{}.task.at", self.source.len()),
        )
        .map_err(|e| vec![e])?;
        if !run.success {
            return Err(vec![format!(
                "async task did not succeed: {} (stdout: {})",
                run.outcome_debug, run.stdout
            )]);
        }
        self.async_settled += 1;
        // 丢弃原因按**精确度**排序：跨会话/跨世代的票即使将来 restart 也永远
        // 不可投递（stale），先于"当前停止"（停止只是暂时不可投递）。
        if ticket.pid != self.pid {
            return Ok(AgeAsyncOutcome::Dropped("session"));
        }
        if ticket.generation != self.body_generation {
            return Ok(AgeAsyncOutcome::Dropped("stale-generation"));
        }
        if self.stopped {
            return Ok(AgeAsyncOutcome::Dropped("stopped"));
        }
        let dispatches = self.send(&ticket.completion_msg)?;
        Ok(AgeAsyncOutcome::Completed(dispatches))
    }

    /// R64-10：异步票账面（spawned / settled / 悬挂数）——多轮 Run/Stop 后
    /// 必须净归零（pending == 0）。
    pub fn async_balance(&self) -> (u64, u64, u64) {
        (
            self.async_spawned,
            self.async_settled,
            self.async_spawned - self.async_settled,
        )
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
        let next =
            Self::compile_modules(&self.modules, &self.entry, self.instances, next_ops.clone())?;
        self.ops = next_ops;
        self.compiled = next;
        self.dispatches += 1;
        Ok(self.dispatches)
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

    /// R64-10：显式 Run（`open` 已经算第一次 Run）。停止后再次 Run 会重新开始。
    pub fn run(&mut self) -> Result<u64, Vec<String>> {
        if self.active {
            return Ok(self.runs);
        }
        // run() 的返回值就是 Run 计数（与 body 代数分开）。
        self.restart()?;
        self.runs += 1;
        self.active = true;
        Ok(self.runs)
    }

    /// 停止：冻结最后观察值（停止期间的调度/编辑都被拒）。
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }
        self.last_observed = Some(self.observe()?);
        self.stopped = true;
        self.active = false;
        self.stops += 1;
        Ok(())
    }

    /// R64-10：Run/Stop 账面（`runs == stops` 表示没有悬挂的运行中句柄）。
    pub fn run_stop_balance(&self) -> (u64, u64, bool) {
        (self.runs, self.stops, self.active)
    }

    /// 恢复：同一 PID 重新启动（Start 再次为 1），generation 递增。
    pub fn restart(&mut self) -> Result<u64, Vec<String>> {
        let next = Self::compile_modules(&self.modules, &self.entry, self.instances, Vec::new())?;
        self.compiled = next;
        self.ops.clear();
        self.stopped = false;
        self.active = true;
        self.last_observed = None;
        self.body_generation += 1;
        Ok(self.body_generation)
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

    /// 已应用的调度次数（tick/send）。
    pub fn generation(&self) -> u64 {
        self.dispatches
    }

    /// body 代数（成功 update_body / restart）。
    pub fn body_generation(&self) -> u64 {
        self.body_generation
    }

    fn compile(
        src: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<AgePackageCompilation, Vec<String>> {
        Self::compile_modules(
            &[("main".to_string(), src.to_string())],
            "main",
            instances,
            ops,
        )
    }

    /// PLAN-1075：多模块闭包编译（与单模块同一受检前门与 verify；模块集即
    /// `PackageModuleInput` 列表，出口 `app::<entry>`）。
    fn compile_modules(
        modules: &[(String, String)],
        entry: &str,
        instances: usize,
        ops: Vec<ace_core::source::age_component::ScenarioOp>,
    ) -> Result<AgePackageCompilation, Vec<String>> {
        let scenario = ComponentScenario { instances, ops };
        let inputs: Vec<PackageModuleInput> = modules
            .iter()
            .map(|(id, src)| PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: id.clone(),
                source: src.clone(),
                provider: None,
            })
            .collect();
        compile_age_package_with_scenario(
            &inputs,
            &format!("app::{entry}"),
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
    /// PLAN-981（B-LANGUAGE-V）：view 属性里的**事件绑定**事实
    /// `(属性键, 消息名, (start, end))`——`onclick: .Inc` 这类真实 A 源形态。
    /// 绑定事实给 980/AV 消费；点击→dispatch 的宿主行为不在本切片。
    pub fn view_events_of(
        &self,
        index: usize,
    ) -> Result<Vec<(String, String, (usize, usize))>, String> {
        self.compiled
            .description
            .components
            .get(index)
            .map(|c| {
                c.view_events
                    .iter()
                    .map(|(k, m, s)| (k.clone(), m.clone(), (s.start, s.end)))
                    .collect()
            })
            .ok_or_else(|| format!("no component at index {index}"))
    }

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
