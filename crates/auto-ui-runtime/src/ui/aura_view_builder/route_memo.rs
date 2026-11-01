//! PLAN-014 T-05：route/memo 责任（PLAN-045 memo 门三态、memo_gate_*、
//! keyed-for、outlet 页面 memo、MemoEnvGate）（原样收编；含 impl#2 整块）。
use super::*;

pub(crate) fn count_keyed_loops_free(node: &AuraNode) -> usize {
    match node {
        AuraNode::ForLoop { key_expr, body, .. } => {
            usize::from(key_expr.is_some()) + body.iter().map(count_keyed_loops_free).sum::<usize>()
        }
        AuraNode::Element { children, .. } | AuraNode::Link { children, .. } => {
            children.iter().map(count_keyed_loops_free).sum::<usize>()
        }
        AuraNode::Conditional {
            then_body,
            else_body,
            ..
        } => then_body
            .iter()
            .chain(else_body.iter().flatten())
            .map(count_keyed_loops_free)
            .sum::<usize>(),
        AuraNode::MemoBlock { body, .. } => body.iter().map(count_keyed_loops_free).sum::<usize>(),
        AuraNode::Component { children, .. } => {
            children.iter().map(count_keyed_loops_free).sum::<usize>()
        }
        _ => 0,
    }
}

// ─────────────────────────────────────────────────────────────────────
// PLAN-045: 组件级 memo 引擎（菜单族四件 + sidebar nav 块）
//
// 机制与正确性论证见 ui/memo_deps.rs 模块注记与 auto-os
// docs/specs/shell/vm-render-memo.md（T-01 决策注记为准：值指纹慢路径，
// 全局 seq 快速路径，降级宁缺勿错；键 = ctx_state_obj + site + 骨架指纹 +
// probe_on）。所有 memo 臂均为 opt-in：`memo:` prop 缺省/false 时 memo_gate
// 直接 Off，原始代码体零重排零包裹。
// ─────────────────────────────────────────────────────────────────────

/// PLAN-708 T-01（S-01）: `AUTO_OUTLET_MEMO` 环境门三态。
#[derive(Clone, Copy, Debug)]
pub(crate) enum MemoEnvGate {
    /// env=0：诊断强制关（覆盖显式 prop true——逃生门优先级最高）。
    ForceOff,
    /// env=1：显式强开（兼容层保留）。
    ForceOn,
    /// 未设/其他值：回落 prop 语义（outlet 缺省 on；菜单族字面量缺省关）。
    Unset,
}

/// PLAN-045: memo 门三态（菜单族/nav 共用）。
pub(crate) enum MemoGateBegin {
    /// 非 memo / 降级 / 未武装——直接原始体（现状代码，零变化）。
    Off,
    /// 命中：产物 + probe/id_map 重放面（调用方合并进当前帧收集器）。
    Hit {
        product: View<DynamicMessage>,
        probe_replay: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)>,
        idmap_replay: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)>,
    },
    /// fill：body 执行后连同产物交 `memo_gate_store` 入缓存。
    Fill(MemoGateFill),
}

/// PLAN-045: fill 阶段载体（门前置产出，门后置消费）。
pub(crate) struct MemoGateFill {
    pub(crate) key: crate::ui::memo_deps::MemoKey,
    pub(crate) slots: Vec<Expr>,
    pub(crate) extra_dyn: Option<u64>,
}

impl<'a> AuraViewBuilder<'a> {
    /// memo site 常量（MemoKey.site）。
    pub(crate) const MEMO_SITE_MENUBAR_COMPONENT: u8 = 1;
    pub(crate) const MEMO_SITE_MENUBAR_DSL: u8 = 2;
    pub(crate) const MEMO_SITE_TOOLBAR: u8 = 3;
    pub(crate) const MEMO_SITE_DIALOG_FAMILY: u8 = 4;
    pub(crate) const MEMO_SITE_SIDEBAR_GROUP: u8 = 5;

    /// PLAN-708 T-01（S-01）: `AUTO_OUTLET_MEMO` 三态门。
    /// `0` → ForceOff（诊断强制关，覆盖一切显式开）；`1` → ForceOn（显式
    /// 强开兼容层）；未设/其他值 → Unset（回落 prop 语义）。
    pub(crate) fn memo_env_gate() -> MemoEnvGate {
        match std::env::var("AUTO_OUTLET_MEMO").as_deref() {
            Ok("0") => MemoEnvGate::ForceOff,
            Ok("1") => MemoEnvGate::ForceOn,
            _ => MemoEnvGate::Unset,
        }
    }

    /// outlet 页三态解析（r2 §5 S-01 优先级冻结表）。
    /// Unset 臂的 prop 未设 = **on**（r2 缺省面变更：outlet 页 memo 缺省开启）。
    pub(crate) fn resolve_outlet_memo(env: MemoEnvGate, prop: Option<bool>) -> bool {
        match env {
            MemoEnvGate::ForceOff => false,
            MemoEnvGate::ForceOn => true,
            MemoEnvGate::Unset => prop.unwrap_or(true),
        }
    }

    /// 菜单/nav 族三态解析：env 覆盖 + 字面量缺省关（PLAN-045 原语义保留；
    /// env=0 诊断强制关/env=1 强制开对其同样生效）。
    pub(crate) fn resolve_site_memo(env: MemoEnvGate, prop_literal: bool) -> bool {
        match env {
            MemoEnvGate::ForceOff => false,
            MemoEnvGate::ForceOn => true,
            MemoEnvGate::Unset => prop_literal,
        }
    }

    /// `memo:` prop 解析：bool 字面量为开关；缺省/`false`/非 bool 容错按
    /// 关（非 bool 一次性诊断——PLAN-045 §1 容错条款）。
    /// PLAN-708 T-01：env 三态覆盖（0 强制关/1 强制开）作用于所有 memo
    /// site（诊断开关全局面）；Unset 臂逐字节保持 045 字面量语义。
    pub(crate) fn memo_prop_on(props: &HashMap<String, AuraPropValue>) -> bool {
        let literal = match props.get("memo") {
            Some(AuraPropValue::Expr(Expr::Bool(b))) => *b,
            Some(AuraPropValue::Expr(other)) => {
                use std::sync::atomic::{AtomicBool, Ordering};
                static MEMO_NONBOOL_WARNED: AtomicBool = AtomicBool::new(false);
                if !MEMO_NONBOOL_WARNED.swap(true, Ordering::Relaxed) {
                    eprintln!(
                        "[PLAN-045] memo prop 非 bool 字面量（{:?}），按 false 处理",
                        other
                    );
                }
                false
            }
            _ => false,
        };
        Self::resolve_site_memo(Self::memo_env_gate(), literal)
    }

    /// memo 上下文门。PLAN-045 T-01 原裁定：bindings 非空或 widget 声明
    /// computed → 整体不 memo（静态不可证）。PLAN-047 T-07（档 C SD-11）
    /// 降级面收敛：**computed 声明不再整体排除**——computed 的读面由动态
    /// 依赖录制闭合（桥读通道 + 引擎读臂 + 信号网三通道；T-06 级联吸收
    /// 使 memo 条目覆盖信号 dep 面），产物逐字节对拍承载等价性（AC-05）；
    /// bindings 保持排除（循环变量版本静态不可证且不随录制域——v1 保守
    /// 边界沿袭）。
    pub(crate) fn memo_ctx_ok(&self, bindings: &Bindings) -> bool {
        bindings.is_empty()
    }

    pub(crate) fn memo_ctx_obj(&self) -> u64 {
        self.override_state_obj_id
            .unwrap_or_else(|| self.bridge.state_obj_id())
    }

    /// 全局 episode 指纹：theme_epoch + menubar/popover 开态 + action_config
    /// 身份。任一翻转 → 条目慢路径 miss（快路径也拦在 gfp 比对上）。
    pub(crate) fn memo_episode_fp(&self) -> u64 {
        let ep = crate::ui::memo_deps::GlobalEpisode {
            theme_epoch: crate::ui::style::theme::theme_epoch(),
            menubar_open: crate::ui::action_config::menubar_open(),
            popover_open: crate::ui::action_config::popover_open(),
            action_config_ptr: crate::ui::action_config::action_config()
                .map(|a| std::sync::Arc::as_ptr(&a) as usize)
                .unwrap_or(0),
        };
        crate::ui::memo_deps::globals_fingerprint(&ep)
    }

    /// 读槽重解析值指纹。`None` = 存在不可内容展开的堆引用 → 整条目降级
    /// （宁多失效，绝不按含堆 id 的指纹命中——原地突变会漏检成陈旧）。
    pub(crate) fn memo_slots_fp(&self, slots: &[Expr], bindings: &Bindings) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        slots.len().hash(&mut h);
        for e in slots {
            match self.resolve_expr_to_value(e, bindings) {
                Some(v) => {
                    // PLAN-711 T-12: 展开失败（深/大堆对象——FileTree computed
                    // 列表实录）不再以 None 失败整个 slots_fp（那会让条目永不
                    // 插入 → 恒 FILL 每帧全量重建）：以标记参与指纹，条目照
                    // 插；该槽的陈旧防护由 dyn_deps 版本检查承担（fill 渲染
                    // 期已录 dep；deps_unchanged 失配即全量重渲，绝不陈旧）。
                    let fp = crate::ui::memo_deps::fingerprint_value(&v, &|x| {
                        self.bridge.expand_heap_for_fingerprint(x)
                    })
                    .unwrap_or_else(|| {
                        use std::hash::{Hash, Hasher};
                        let mut mh = std::collections::hash_map::DefaultHasher::new();
                        "unexpandable".hash(&mut mh);
                        mh.finish()
                    });
                    fp.hash(&mut h);
                }
                None => {
                    // 解析空是确定性结果（字段缺失等），以标记参与指纹。
                    "unresolved".hash(&mut h);
                }
            }
        }
        Some(h.finish())
    }

    /// 门前置：扫描降级判定 + 查表 + 快/慢路径。`extra_dyn` = 转换器隐藏
    /// 读面的派生指纹（sidebar nav：per-button active + 组开态），并入 dyn
    /// 比对；菜单族传 None。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn memo_gate_begin(
        &self,
        site: u8,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
        probe_on: bool,
        extra_dyn: Option<u64>,
        prop_gated: bool,
    ) -> MemoGateBegin {
        use crate::ui::memo_deps::{scan_static, MemoKey, ScanVerdict};

        // prop_gated=false：武装旗标门（sidebar nav 组——旗标已由 provider
        // wrapper 判定），跳过节点自身 memo prop 检查。
        if prop_gated && !Self::memo_prop_on(props) {
            return MemoGateBegin::Off;
        }
        if !self.memo_ctx_ok(bindings) {
            self.bridge.with_memo_cache(|c| c.note_degraded());
            return MemoGateBegin::Off;
        }
        let slots = match scan_static(props, children) {
            ScanVerdict::Slots(s) => s,
            ScanVerdict::Degrade(reason) => {
                self.bridge.with_memo_cache(|c| c.note_degraded());
                if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                    eprintln!("[MEMO-DIAG] site={site} DEGRADE reason={reason}");
                }
                return MemoGateBegin::Off;
            }
        };
        let key = MemoKey {
            ctx_state_obj: self.memo_ctx_obj(),
            site,
            skeleton_fp: crate::ui::memo_deps::skeleton_fingerprint(props, children),
            probe_on,
            item_key: None,
            ui_epoch: crate::ui::memo_deps::ui_epoch(),
        };
        let seq = self.bridge.state_mutation_seq();
        let gfp = self.memo_episode_fp();
        // 快路径：seq 未动 ∧ 全局 episode 同 → 零重解析全命中。
        let cached = self.bridge.with_memo_cache(|c| {
            c.lookup(&key).map(|e| {
                (
                    e.seq_at_fill,
                    e.globals_fp,
                    e.dyn_fp,
                    e.product.clone(),
                    e.probe_replay.clone(),
                    e.idmap_replay.clone(),
                    e.dyn_deps.clone(),
                )
            })
        });
        if let Some((
            seq_at_fill,
            gfp_entry,
            dyn_fp,
            product,
            probe_replay,
            idmap_replay,
            dyn_deps,
        )) = cached
        {
            if seq_at_fill == seq && gfp_entry == gfp {
                self.bridge.with_memo_cache(|c| {
                    c.note_hit();
                    c.note_seq_fast(None);
                });
                if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                    eprintln!("[MEMO-DIAG] site={site} HIT fast");
                }
                return MemoGateBegin::Hit {
                    product,
                    probe_replay,
                    idmap_replay,
                };
            }
            if gfp_entry == gfp {
                // PLAN-047 T-05（档 C SD-11）: version_fast——动态 dep 集版本
                // 全同 → 零重解析命中（fill 后任何归因写都会前进对应 path
                // 版本；C 类全局写不动既有内容，命中安全）。两例外落指纹
                // 慢路径：fill 未录到集（None，行为同档 A/B）；extra_dyn
                // 派生面（sidebar nav 路由/组开态——Rust 侧读通道，VM 录制
                // 盲区，由 extra_dyn 指纹慢路径承载，同档 A/B）。
                if let Some(pairs) = &dyn_deps {
                    if extra_dyn.is_none() && self.bridge.deps_unchanged(pairs) {
                        self.bridge.with_memo_cache(|c| {
                            c.refresh_seq(&key, seq);
                            c.note_hit();
                            c.note_version_fast(None);
                        });
                        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                            eprintln!("[MEMO-DIAG] site={site} HIT version-fast");
                        }
                        return MemoGateBegin::Hit {
                            product,
                            probe_replay,
                            idmap_replay,
                        };
                    }
                }
                // 慢路径：读槽重解析 + 派生面 → 值指纹比对（T-01 值指纹设计）。
                if let (Some(slots_fp), Some(extra)) =
                    (self.memo_slots_fp(&slots, bindings), extra_dyn)
                {
                    let cur = Self::memo_combine_dyn(slots_fp, Some(extra));
                    if Some(cur) == dyn_fp {
                        // fp_slow 命中：基线前移（seq + 动态 dep 版本）——后续
                        // 帧回到 seq/version 快路径。
                        let refreshed = dyn_deps.as_ref().map(|pairs| {
                            pairs
                                .iter()
                                .map(|(k, _)| {
                                    let v = self.bridge.vm().path_version(k.heap_id, &k.path);
                                    (k.clone(), v)
                                })
                                .collect()
                        });
                        self.bridge.with_memo_cache(|c| {
                            c.refresh_seq(&key, seq);
                            if let Some(pairs) = refreshed {
                                c.refresh_deps(&key, pairs);
                            }
                            c.note_hit();
                            c.note_fp_slow(None);
                        });
                        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                            eprintln!("[MEMO-DIAG] site={site} HIT slow");
                        }
                        return MemoGateBegin::Hit {
                            product,
                            probe_replay,
                            idmap_replay,
                        };
                    }
                } else if extra_dyn.is_none() {
                    if let Some(slots_fp) = self.memo_slots_fp(&slots, bindings) {
                        let cur = Self::memo_combine_dyn(slots_fp, None);
                        if Some(cur) == dyn_fp {
                            let refreshed = dyn_deps.as_ref().map(|pairs| {
                                pairs
                                    .iter()
                                    .map(|(k, _)| {
                                        let v = self.bridge.vm().path_version(k.heap_id, &k.path);
                                        (k.clone(), v)
                                    })
                                    .collect()
                            });
                            self.bridge.with_memo_cache(|c| {
                                c.refresh_seq(&key, seq);
                                if let Some(pairs) = refreshed {
                                    c.refresh_deps(&key, pairs);
                                }
                                c.note_hit();
                                c.note_fp_slow(None);
                            });
                            if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                                eprintln!("[MEMO-DIAG] site={site} HIT slow");
                            }
                            return MemoGateBegin::Hit {
                                product,
                                probe_replay,
                                idmap_replay,
                            };
                        }
                    }
                }
            }
            if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                eprintln!("[MEMO-DIAG] site={site} MISS (slow mismatch)");
            }
            self.bridge.with_memo_cache(|c| c.note_miss());
        }
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!("[MEMO-DIAG] site={site} FILL (no entry)");
        }
        MemoGateBegin::Fill(MemoGateFill {
            key,
            slots,
            extra_dyn,
        })
    }

    /// 门后置：fill——body 执行后采集（产物对应 body 后状态），读槽重解析
    /// 指纹 + probe/id_map 快照入缓存。不可展开堆引用面 → 降级不入缓存。
    /// PLAN-047 T-05: `rec` = body 求值期的依赖录制集（fill_with_recording
    /// 产出）——空集/超预算 → `dyn_deps = None`（version_fast 面缺失，行为
    /// 与档 A/B 一致）。
    pub(crate) fn memo_gate_store(
        &self,
        fill: MemoGateFill,
        bindings: &Bindings,
        product: View<DynamicMessage>,
        probe_snap: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)>,
        idmap_snap: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)>,
        rec: crate::ui::memo_deps::RecState,
    ) -> View<DynamicMessage> {
        let MemoGateFill {
            key,
            slots,
            extra_dyn,
        } = fill;
        let Some(slots_fp) = self.memo_slots_fp(&slots, bindings) else {
            self.bridge.with_memo_cache(|c| c.note_degraded());
            return product;
        };
        let dyn_deps = if rec.overflow || rec.deps.is_empty() {
            None
        } else {
            Some(self.bridge.dep_pairs(&rec.deps))
        };
        let entry = crate::ui::memo_deps::MemoEntry {
            seq_at_fill: self.bridge.state_mutation_seq(),
            globals_fp: self.memo_episode_fp(),
            read_exprs: slots,
            dyn_fp: Some(Self::memo_combine_dyn(slots_fp, extra_dyn)),
            product: product.clone(),
            probe_replay: probe_snap,
            idmap_replay: idmap_snap,
            replay_relative: false,
            dyn_deps,
        };
        self.bridge.with_memo_cache(|c| c.insert(key, entry));
        product
    }

    /// PLAN-047 T-05: fill 侧录制包裹——body 求值全程依赖录制（桥读通道 +
    /// 引擎读臂双通道，guard 嵌套并集），返回 (产物, 录制集)。产物型泛化
    /// （View / Option<View>——keyed-for 项级 body 可空）。
    pub(crate) fn fill_with_recording<R>(
        &self,
        body: impl FnOnce() -> R,
    ) -> (R, crate::ui::memo_deps::RecState) {
        let g = self.bridge.dep_recording_guard();
        let out = body();
        let rec = g.finish();
        (out, rec)
    }

    /// PLAN-047 T-05: 录制集 → 条目 dep 面（空集/超预算 → None，行为退回
    /// 档 A/B）。
    pub(crate) fn rec_to_dyn_deps(
        &self,
        rec: &crate::ui::memo_deps::RecState,
    ) -> Option<Vec<(crate::ui::memo_deps::DepKey, u64)>> {
        if rec.overflow || rec.deps.is_empty() {
            None
        } else {
            Some(self.bridge.dep_pairs(&rec.deps))
        }
    }

    /// PLAN-047 T-05: 现版本基线对（fp_slow/version_fast 命中后刷新——
    /// 值同/版本同证明基线可前移，后续帧回快路径）；`None` 透传。
    pub(crate) fn refreshed_dep_pairs(
        &self,
        dyn_deps: &Option<Vec<(crate::ui::memo_deps::DepKey, u64)>>,
    ) -> Option<Vec<(crate::ui::memo_deps::DepKey, u64)>> {
        dyn_deps.as_ref().map(|pairs| {
            pairs
                .iter()
                .map(|(k, _)| {
                    let v = self.bridge.vm().path_version(k.heap_id, &k.path);
                    (k.clone(), v)
                })
                .collect()
        })
    }

    pub(crate) fn memo_combine_dyn(slots_fp: u64, extra_dyn: Option<u64>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        slots_fp.hash(&mut h);
        extra_dyn.hash(&mut h);
        h.finish()
    }

    /// sidebar nav 块派生指纹：collapsible + nav_group_states 全表 + 每个
    /// menu button 的 (to, exact, active)。active 复刻转换器判定序（显式
    /// `active:` prop > `nav_route_active` 路由自动），路由变化时仅含旧/新
    /// active 翻转按钮的组翻面——其余组全命中（PLAN-045 §3）。
    pub(crate) fn memo_nav_group_extra_dyn(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        let collapsible = self.extract_bool(props, "collapsible").unwrap_or(false)
            || self
                .extract_bool_expr(props, "collapsible", bindings)
                .unwrap_or(false);
        collapsible.hash(&mut h);
        if let Some(states) = self.nav_group_states {
            let mut keys: Vec<(&String, &bool)> = states.iter().collect();
            keys.sort();
            keys.len().hash(&mut h);
            for (k, v) in keys {
                k.hash(&mut h);
                v.hash(&mut h);
            }
        }
        self.memo_nav_buttons_walk(children, bindings, &mut h);
        h.finish()
    }

    pub(crate) fn memo_nav_buttons_walk(
        &self,
        children: &[AuraNode],
        bindings: &Bindings,
        h: &mut std::collections::hash_map::DefaultHasher,
    ) {
        use std::hash::Hash;
        for c in children {
            if let AuraNode::Element {
                tag,
                props,
                children,
                ..
            } = c
            {
                let tag_lc = tag.replace('_', "-");
                if tag_lc == "sidebar-menu-button" || tag_lc == "sidebar-menu-sub-button" {
                    let to = self
                        .extract_string_with(props, "to", bindings)
                        .unwrap_or_default();
                    let exact = self.extract_bool(props, "exact").unwrap_or(false);
                    let active = self
                        .extract_bool_expr(props, "active", bindings)
                        .unwrap_or_else(|| self.nav_route_active(&to, exact));
                    to.hash(h);
                    exact.hash(h);
                    active.hash(h);
                }
                self.memo_nav_buttons_walk(children, bindings, h);
            }
        }
    }

    /// PLAN-046 T-03：keyed-for 项级缓存门（`for x in .items key: .id`）。
    ///
    /// 键 = `(ctx_state_obj, MEMO_SITE_FOR_ITEM, 站点骨架指纹, probe_on,
    /// key 值指纹)`；命中 = 快速路径（seq 未动 ∧ episode 同）或慢路径
    /// （项值指纹 ∧ 项体读槽重解析 ∧ [声明 index 时迭代位] 全同——
    /// `memo_combine_for`）。降级（宁缺勿错，回退体=convert_for_unkeyed
    /// 同一实现，行为与非 key 化一致）：嵌套绑定上下文、项体扫描降级、
    /// key 求值失败/重复 key（绝不静默去重）、项值超指纹预算（AC-03
    /// 整体降级）、存侧读槽指纹失败。输出序恒随 iterable 当前序（缓存
    /// 改变的是求值次数，不是输出顺序——AC-02）。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn convert_for_keyed(
        &self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        key_expr: &Expr,
        body: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        const SITE: u8 = crate::ui::memo_deps::MEMO_SITE_FOR_ITEM;
        let probe_on = probe.is_enabled();
        let base_len = path.len();
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!("[MEMO-DIAG] site=7 enter var={var} iterable={iterable}");
        }

        // ── 降级前置门：嵌套绑定上下文（T-01 D-4 保守裁定——外层循环变量
        // 进入项体求值，其读面版本静态不可证；memo_ctx_ok 的 for 面）。
        if !bindings.is_empty() {
            self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
            memo_diag("for_item", "nested-bindings");
            return self
                .convert_for_unkeyed(var, index, iterable, body, path, id_map, probe, bindings);
        }

        // ── iterable 解析（与 keyless 同一通道，None → View::Empty）。
        let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
        let Some(array) = self.resolve_for_iterable(iterable, state_name, bindings) else {
            return View::Empty;
        };

        // ── 静态门：项体扫描（外部状态读入槽）+ 站点骨架指纹。
        let loop_vars: std::collections::HashSet<String> = {
            let mut s = std::collections::HashSet::new();
            s.insert(var.to_string());
            if let Some(iv) = index {
                s.insert(iv.clone());
            }
            s
        };
        let slots = match crate::ui::memo_deps::scan_for_item_body(
            body,
            self.widget_registry,
            &loop_vars,
        ) {
            crate::ui::memo_deps::ScanVerdict::Slots(s) => s,
            crate::ui::memo_deps::ScanVerdict::Degrade(reason) => {
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                memo_diag("for_item", &format!("scan:{}", reason));
                return self.convert_for_unkeyed(
                    var, index, iterable, body, path, id_map, probe, bindings,
                );
            }
        };
        let skeleton_fp = crate::ui::memo_deps::for_site_fingerprint(
            var,
            index.as_deref(),
            iterable,
            key_expr,
            body,
            self.widget_registry,
            &slots,
        );
        let ctx_obj = self.memo_ctx_obj();
        let seq = self.bridge.state_mutation_seq();
        let gfp = self.memo_episode_fp();

        // ── pass 1：逐项 key/item 指纹 + 循环绑定预构。重复 key / key 求值
        // 失败 / 项值超指纹预算 → 整体降级原始路径（AC-03；此时尚未与缓存
        // 交互，无半帧状态）。
        struct KeyedPlan {
            key_fp: u64,
            item_fp: u64,
            loop_bindings: Bindings,
            item: Value,
        }
        let mut plans: Vec<KeyedPlan> = Vec::with_capacity(array.len());
        let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for (i, item) in array.iter().enumerate() {
            let mut lb = bindings.clone();
            lb.insert(var.to_string(), self.bridge.materialize_obj_ref(item));
            if let Some(idx_var) = index {
                lb.insert(idx_var.clone(), Value::Int(i as i32));
            }
            let Some(kv) = self.resolve_expr_to_value(key_expr, &lb) else {
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                memo_diag("for_item", "key-eval");
                return self.convert_for_unkeyed(
                    var, index, iterable, body, path, id_map, probe, bindings,
                );
            };
            let expand = |x: &Value| self.bridge.expand_heap_for_fingerprint(x);
            let Some(kfp) = crate::ui::memo_deps::fingerprint_value(&kv, &expand) else {
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                memo_diag("for_item", "key-fp");
                return self.convert_for_unkeyed(
                    var, index, iterable, body, path, id_map, probe, bindings,
                );
            };
            if !seen.insert(kfp) {
                // 重复 key → 整体降级（一次性诊断），绝不静默去重。
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                memo_diag("for_item", "dup-key");
                return self.convert_for_unkeyed(
                    var, index, iterable, body, path, id_map, probe, bindings,
                );
            }
            let Some(ifp) = crate::ui::memo_deps::fingerprint_value(item, &expand) else {
                // 项值超 4096 预算 → 整体降级（AC-03；展开不了的堆引用
                // 绝不按含 id 指纹命中）。
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                memo_diag("for_item", "item-fp-budget");
                return self.convert_for_unkeyed(
                    var, index, iterable, body, path, id_map, probe, bindings,
                );
            };
            plans.push(KeyedPlan {
                key_fp: kfp,
                item_fp: ifp,
                loop_bindings: lb,
                item: item.clone(),
            });
        }

        // ── 容量抬升（Q-03：千行列表防 LRU 抖动，硬顶 MEMO_CACHE_CAP_MAX）。
        self.bridge
            .with_memo_cache(|c| c.ensure_capacity(plans.len()));

        // ── pass 2：按 iterable 当前序逐项命中/fill。
        let base_u16: Vec<u16> = path.iter().map(|&x| x as u16).collect();
        let mut child_views: Vec<View<DynamicMessage>> = Vec::with_capacity(plans.len());
        // T-06 计量：帧内门汇总（AUTO_MEMO_DIAG=1 逐帧一行，千行列表不刷屏）。
        let mut diag_hit = 0u64;
        let mut diag_miss = 0u64;
        let mut diag_fill = 0u64;
        let mut diag_skip = 0u64;
        for (i, plan) in plans.iter().enumerate() {
            // 搜索过滤每帧按当前态现算（帧级集合语义；不进项缓存）。
            if !self.matches_search(&plan.item) {
                diag_skip += 1;
                continue;
            }
            let key = crate::ui::memo_deps::MemoKey {
                ctx_state_obj: ctx_obj,
                site: SITE,
                skeleton_fp,
                probe_on,
                item_key: Some(plan.key_fp),
                ui_epoch: crate::ui::memo_deps::ui_epoch(),
            };
            let cached = self.bridge.with_memo_cache(|c| {
                c.lookup(&key).map(|e| {
                    (
                        e.seq_at_fill,
                        e.globals_fp,
                        e.dyn_fp,
                        e.product.clone(),
                        e.probe_replay.clone(),
                        e.idmap_replay.clone(),
                        e.read_exprs.clone(),
                        e.dyn_deps.clone(),
                    )
                })
            });
            let mut reused: Option<View<DynamicMessage>> = None;
            if let Some((seq0, gfp0, dyn0, product, prel, irel, slots0, dyn_deps)) = cached {
                // PLAN-047 T-05: 三级判定——seq_fast → version_fast（动态 dep
                // 版本全同零重解析）→ fp_slow（既有指纹慢路径兜底）。
                #[derive(PartialEq)]
                enum Kind {
                    SeqFast,
                    VersionFast,
                    FpSlow,
                }
                let mut kind = Kind::FpSlow;
                let hit = if seq0 == seq && gfp0 == gfp {
                    // 快速路径：fill 后全局 seq 未动 ∧ episode 同。
                    kind = Kind::SeqFast;
                    true
                } else if gfp0 == gfp
                    && dyn_deps
                        .as_ref()
                        .is_some_and(|p| self.bridge.deps_unchanged(p))
                {
                    // PLAN-047 T-05: version_fast——fill 后归因写未触及本项
                    // 依赖面（无关写帧零重解析命中）。
                    kind = Kind::VersionFast;
                    true
                } else if gfp0 == gfp {
                    // 慢路径：项体读槽重解析（循环绑定上下文）∧ 项值 ∧
                    // index 位 → 值指纹比对（档 A 慢路径同源）。
                    self.memo_slots_fp(&slots0, &plan.loop_bindings)
                        .map(|sfp| {
                            Some(Self::memo_combine_for(
                                sfp,
                                plan.item_fp,
                                index.as_ref().map(|_| i as u64),
                            )) == dyn0
                        })
                        .unwrap_or(false)
                } else {
                    false
                };
                if hit {
                    self.bridge.with_memo_cache(|c| {
                        c.note_hit_site(SITE);
                        match kind {
                            Kind::SeqFast => c.note_seq_fast(Some(SITE)),
                            Kind::VersionFast => {
                                c.refresh_seq(&key, seq);
                                if let Some(pairs) = self.refreshed_dep_pairs(&dyn_deps) {
                                    c.refresh_deps(&key, pairs);
                                }
                                c.note_version_fast(Some(SITE));
                            }
                            Kind::FpSlow => {
                                c.refresh_seq(&key, seq);
                                if let Some(pairs) = self.refreshed_dep_pairs(&dyn_deps) {
                                    c.refresh_deps(&key, pairs);
                                }
                                c.note_fp_slow(Some(SITE));
                            }
                        }
                    });
                    diag_hit += 1;
                    Self::replay_for_item(probe, id_map, &base_u16, i, prel, irel);
                    reused = Some(product);
                } else {
                    self.bridge.with_memo_cache(|c| c.note_miss_site(SITE));
                    diag_miss += 1;
                }
            }
            let product = match reused {
                Some(p) => p,
                None => {
                    // fill：全新求值（render_for_item_view 内部 push/pop [i]）。
                    // PLAN-047 T-05: 求值期依赖录制（项体读面——循环绑定上下文
                    // 的项值展开 + 外部状态读）。**iterable 依赖显式注入**：
                    // pass-1 的 iterable 解析在录制域外（帧级面）——键集/项值/
                    // 输出序的失效由 iterable 版本承载；注入在录制域内（guard
                    // 外录制被丢弃）；iterable 非直读状态（computed 源）→
                    // dyn_deps=None（退回档 A/B 行为）。
                    let ((out, slots_fp, iterable_dep_ok), rec) = self.fill_with_recording(|| {
                        let iterable_dep_ok = match self.bridge.read_state(state_name) {
                            Ok(v) => {
                                self.bridge.record_dep_value_heap(&v);
                                true
                            }
                            Err(_) => false,
                        };
                        let out = self.render_for_item_view(
                            i,
                            &plan.item,
                            var,
                            index,
                            iterable,
                            body,
                            path,
                            id_map,
                            probe,
                            &plan.loop_bindings,
                        );
                        // 项体读槽录制域内估值（未走分支的槽入集，保守）。
                        let slots_fp = self.memo_slots_fp(&slots, &plan.loop_bindings);
                        (out, slots_fp, iterable_dep_ok)
                    });
                    let dyn_deps = if iterable_dep_ok {
                        self.rec_to_dyn_deps(&rec)
                    } else {
                        None
                    };
                    let Some(out) = out else {
                        // 搜索剔除/空体——与 keyless 同跳过，不入缓存。
                        diag_skip += 1;
                        continue;
                    };
                    diag_fill += 1;
                    // 簿记快照：base+[i] 前缀下本帧新增条目 → 项内相对路径。
                    let mut prefix_u16 = base_u16.clone();
                    prefix_u16.push(i as u16);
                    let mut prefix_us: Vec<usize> = path[..base_len].to_vec();
                    prefix_us.push(i);
                    let prel_rel: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)> = probe
                        .snapshot_prefix(&prefix_u16)
                        .into_iter()
                        .map(|(mut p, e)| {
                            p.drain(..prefix_u16.len());
                            (p, e)
                        })
                        .collect();
                    let irel_rel: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)> = id_map
                        .snapshot_prefix(&prefix_us)
                        .into_iter()
                        .map(|(mut p, e)| {
                            p.drain(..prefix_us.len());
                            (p, e)
                        })
                        .collect();
                    // 存侧指纹（fill 后状态）：读槽重解析失败 → 该项不入缓存。
                    // PLAN-047 T-05: 槽指纹已在录制域内估值（fill 时点）——此处
                    // 复用，不在录制域外二次解析。
                    match slots_fp {
                        Some(sfp) => {
                            let entry = crate::ui::memo_deps::MemoEntry {
                                seq_at_fill: self.bridge.state_mutation_seq(),
                                globals_fp: gfp,
                                read_exprs: slots.clone(),
                                dyn_fp: Some(Self::memo_combine_for(
                                    sfp,
                                    plan.item_fp,
                                    index.as_ref().map(|_| i as u64),
                                )),
                                product: out.clone(),
                                probe_replay: prel_rel,
                                idmap_replay: irel_rel,
                                replay_relative: true,
                                dyn_deps,
                            };
                            self.bridge.with_memo_cache(|c| c.insert(key, entry));
                        }
                        None => {
                            self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                        }
                    }
                    out
                }
            };
            child_views.push(product);
        }
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!(
                "[MEMO-DIAG] site=7 frame hits={diag_hit} misses={diag_miss} fills={diag_fill} skip={diag_skip}"
            );
        }
        View::Column {
            children: child_views,
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
            on_right_click: None,
        }
    }

    /// keyed-for 项级慢路径动态指纹组合：读槽 ∧ 项值 ∧ [index 位]。
    /// index 声明即参与（保守：无法廉价证明体不读 index——重排必失效）。
    pub(crate) fn memo_combine_for(slots_fp: u64, item_fp: u64, idx: Option<u64>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        "for_item".hash(&mut h);
        slots_fp.hash(&mut h);
        item_fp.hash(&mut h);
        idx.hash(&mut h);
        h.finish()
    }

    /// PLAN-046 T-03：命中帧簿记重放（keyed-for 项）——项内相对路径以当前
    /// 基路径+[i] 前缀化，ForIter.index 补丁为当前迭代位。
    pub(crate) fn replay_for_item(
        probe: &mut BuildProbe,
        id_map: &mut DebugIdMap,
        base_u16: &[u16],
        i: usize,
        probe_rel: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)>,
        idmap_rel: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)>,
    ) {
        Self::replay_memo_subtree(probe, id_map, base_u16, Some(i), probe_rel, idmap_rel);
    }
    /// PLAN-046 T-04：显式 memo 块门（`memo (deps: .a, .b) { … }`）。
    ///
    /// 默认语义（保守）：deps 值指纹 ∪ 块内自动可证读槽
    /// （`scan_memo_block_body`——Call/VM 代码形态不入槽不降级[其读面由
    /// deps 声明覆盖]，ForLoop/Outlet/嵌套块 → 整块降级）；episode 翻转
    /// 失效。`exact: true`（语料签名担保）：纯 deps 判定——不扫描不重解析
    /// 读槽，唯一允许"纯 deps 判定"的形态（canonical 档明示陈旧风险语料
    /// 自担）。块内组件节点照常走自身 memo 门（块条目与组件条目独立，不
    /// 嵌套失效）。命中帧簿记：块内相对路径重放（上游结构变化时落位仍
    /// 正确）。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn convert_memo_block(
        &self,
        deps: &[Expr],
        exact: bool,
        body: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        const SITE: u8 = crate::ui::memo_deps::MEMO_SITE_MEMO_BLOCK;
        let probe_on = probe.is_enabled();
        let base_len = path.len();

        // 降级前置门：bindings 非空上下文（for 体内 memo 块——Q-02 保守
        // 裁定：外层循环变量进入块内求值，其版本不随 deps）。
        if !bindings.is_empty() {
            self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
            return self.render_memo_block_body(body, path, id_map, probe, bindings);
        }

        // 静态门：默认语义扫描（exact 跳过——作者签名担保读面 ⊆ deps）。
        let slots = if exact {
            Vec::new()
        } else {
            match crate::ui::memo_deps::scan_memo_block_body(body, self.widget_registry) {
                crate::ui::memo_deps::ScanVerdict::Slots(s) => s,
                crate::ui::memo_deps::ScanVerdict::Degrade(reason) => {
                    self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                    memo_diag("memo_block", reason);
                    return self.render_memo_block_body(body, path, id_map, probe, bindings);
                }
            }
        };
        let key = crate::ui::memo_deps::MemoKey {
            ctx_state_obj: self.memo_ctx_obj(),
            site: SITE,
            skeleton_fp: crate::ui::memo_deps::memo_block_site_fingerprint(
                deps,
                exact,
                body,
                self.widget_registry,
            ),
            ui_epoch: crate::ui::memo_deps::ui_epoch(),
            probe_on,
            item_key: None,
        };
        let seq = self.bridge.state_mutation_seq();
        let gfp = self.memo_episode_fp();
        let cached = self.bridge.with_memo_cache(|c| {
            c.lookup(&key).map(|e| {
                (
                    e.seq_at_fill,
                    e.globals_fp,
                    e.dyn_fp,
                    e.product.clone(),
                    e.probe_replay.clone(),
                    e.idmap_replay.clone(),
                    e.read_exprs.clone(),
                    e.dyn_deps.clone(),
                )
            })
        });
        if let Some((seq0, gfp0, dyn0, product, prel, irel, slots0, dyn_deps)) = cached {
            // PLAN-047 T-05: 三级判定（同 keyed-for——seq_fast → version_fast
            // → fp_slow）。
            #[derive(PartialEq)]
            enum Kind {
                SeqFast,
                VersionFast,
                FpSlow,
            }
            let mut kind = Kind::FpSlow;
            let hit = if seq0 == seq && gfp0 == gfp {
                // 快速路径：fill 后全局 seq 未动 ∧ episode 同。
                kind = Kind::SeqFast;
                true
            } else if gfp0 == gfp
                && dyn_deps
                    .as_ref()
                    .is_some_and(|p| self.bridge.deps_unchanged(p))
            {
                // PLAN-047 T-05: version_fast——无关写帧零重解析命中。
                kind = Kind::VersionFast;
                true
            } else if gfp0 == gfp {
                // 慢路径：deps 值指纹恒比；默认语义再加读槽重解析。
                match self.memo_slots_fp(deps, bindings) {
                    Some(dfp) if exact => Some(Self::memo_combine_block(dfp, None)) == dyn0,
                    Some(dfp) => self
                        .memo_slots_fp(&slots0, bindings)
                        .map(|sfp| Some(Self::memo_combine_block(dfp, Some(sfp))) == dyn0)
                        .unwrap_or(false),
                    None => false,
                }
            } else {
                false
            };
            if hit {
                self.bridge.with_memo_cache(|c| {
                    c.note_hit_site(SITE);
                    match kind {
                        Kind::SeqFast => c.note_seq_fast(Some(SITE)),
                        Kind::VersionFast => {
                            c.refresh_seq(&key, seq);
                            if let Some(pairs) = self.refreshed_dep_pairs(&dyn_deps) {
                                c.refresh_deps(&key, pairs);
                            }
                            c.note_version_fast(Some(SITE));
                        }
                        Kind::FpSlow => {
                            c.refresh_seq(&key, seq);
                            if let Some(pairs) = self.refreshed_dep_pairs(&dyn_deps) {
                                c.refresh_deps(&key, pairs);
                            }
                            c.note_fp_slow(Some(SITE));
                        }
                    }
                });
                if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                    let label = match kind {
                        Kind::SeqFast => "HIT fast",
                        Kind::VersionFast => "HIT version-fast",
                        Kind::FpSlow => "HIT slow",
                    };
                    eprintln!("[MEMO-DIAG] site=8 {label} (exact={exact})");
                }
                let base_u16: Vec<u16> = path[..base_len].iter().map(|&x| x as u16).collect();
                Self::replay_memo_subtree(probe, id_map, &base_u16, None, prel, irel);
                return product;
            }
            self.bridge.with_memo_cache(|c| c.note_miss_site(SITE));
            if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
                eprintln!("[MEMO-DIAG] site=8 MISS (slow mismatch, exact={exact})");
            }
        }

        // fill：原始渲染 + 簿记相对快照 + 入缓存。PLAN-047 T-05: 求值期依赖
        // 录制（块体读面——deps 声明外的动态读由运行时事实覆盖）；**deps 与
        // 读槽估值同在录制域内**（声明的 deps/静态槽是块显式读面——不录制
        // 则 version_fast 漏失效面=陈旧风险）。
        if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
            eprintln!("[MEMO-DIAG] site=8 FILL (exact={exact})");
        }
        let ((out, deps_fp, slots_fp), rec) = self.fill_with_recording(|| {
            let out = self.render_memo_block_body(body, path, id_map, probe, bindings);
            let deps_fp = self.memo_slots_fp(deps, bindings);
            let slots_fp = if exact {
                None
            } else {
                self.memo_slots_fp(&slots, bindings)
            };
            (out, deps_fp, slots_fp)
        });
        let dyn_deps = self.rec_to_dyn_deps(&rec);
        let base_u16: Vec<u16> = path[..base_len].iter().map(|&x| x as u16).collect();
        let base_us: Vec<usize> = path[..base_len].to_vec();
        let prel_rel: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)> = probe
            .snapshot_prefix(&base_u16)
            .into_iter()
            .map(|(mut p, e)| {
                p.drain(..base_u16.len());
                (p, e)
            })
            .collect();
        let irel_rel: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)> = id_map
            .snapshot_prefix(&base_us)
            .into_iter()
            .map(|(mut p, e)| {
                p.drain(..base_us.len());
                (p, e)
            })
            .collect();
        // PLAN-047 T-05: deps/槽指纹已在录制域内估值（fill 时点）——复用。
        match deps_fp {
            Some(dfp) => {
                let sfp = if exact { None } else { slots_fp };
                if exact || sfp.is_some() {
                    let entry = crate::ui::memo_deps::MemoEntry {
                        seq_at_fill: self.bridge.state_mutation_seq(),
                        globals_fp: gfp,
                        read_exprs: slots,
                        dyn_fp: Some(Self::memo_combine_block(dfp, sfp)),
                        product: out.clone(),
                        probe_replay: prel_rel,
                        idmap_replay: irel_rel,
                        replay_relative: true,
                        dyn_deps,
                    };
                    self.bridge.with_memo_cache(|c| c.insert(key, entry));
                } else {
                    // 存侧读槽指纹失败（不可展开堆引用）→ 不入缓存。
                    self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
                }
            }
            None => {
                // deps 不可内容指纹（堆引用展开不了）→ 不入缓存。
                self.bridge.with_memo_cache(|c| c.note_degraded_site(SITE));
            }
        }
        out
    }

    /// memo 块体原始渲染（fill 与降级回退共用）：镜像 Conditional 臂的
    /// 展开规则（单子直出/多子包 Column/[bi] 仅多子时入路径）。
    pub(crate) fn render_memo_block_body(
        &self,
        body: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let body_len = body.len();
        let child_views: Vec<View<DynamicMessage>> = body
            .iter()
            .enumerate()
            .map(|(bi, n)| {
                if body_len > 1 {
                    path.push(bi);
                }
                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                if body_len > 1 {
                    path.pop();
                }
                v
            })
            .collect();
        if child_views.is_empty() {
            View::Empty
        } else if child_views.len() == 1 {
            child_views.into_iter().next().unwrap()
        } else {
            View::Column {
                children: child_views,
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
                on_right_click: None,
            }
        }
    }

    /// memo 块慢路径动态指纹组合：deps 值指纹 ∧ [读槽指纹]（exact=None）。
    pub(crate) fn memo_combine_block(deps_fp: u64, slots_fp: Option<u64>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        "memo_block".hash(&mut h);
        deps_fp.hash(&mut h);
        slots_fp.hash(&mut h);
        h.finish()
    }

    /// PLAN-046 T-04/T-03 共用：子树命中帧簿记重放。`extra_seg` = 追加段
    /// （keyed-for 项 = Some(i)——同时补丁 ForIter.index；memo 块 = None——
    /// 存量 for_context 保持 fill 时刻索引）。
    pub(crate) fn replay_memo_subtree(
        probe: &mut BuildProbe,
        id_map: &mut DebugIdMap,
        base_u16: &[u16],
        extra_seg: Option<usize>,
        probe_rel: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)>,
        idmap_rel: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)>,
    ) {
        let mut prefixed: Vec<(Vec<u16>, crate::ui::debug::ProbeEntry)> =
            Vec::with_capacity(probe_rel.len());
        for (rel, mut e) in probe_rel {
            if let (Some(i), Some(fc)) = (extra_seg, e.for_context.as_mut()) {
                fc.index = Some(i);
            }
            let mut abs = base_u16.to_vec();
            if let Some(i) = extra_seg {
                abs.push(i as u16);
            }
            abs.extend_from_slice(&rel);
            prefixed.push((abs, e));
        }
        probe.merge_entries(prefixed);
        let mut idabs: Vec<(Vec<usize>, auto_ui_compiler::aura::AuraNodeId)> =
            Vec::with_capacity(idmap_rel.len());
        for (rel, id) in idmap_rel {
            let mut abs: Vec<usize> = base_u16.iter().map(|&x| x as usize).collect();
            if let Some(i) = extra_seg {
                abs.push(i);
            }
            abs.extend_from_slice(&rel);
            idabs.push((abs, id));
        }
        id_map.merge_entries(idabs);
    }
}

/// PLAN-046：keyed-for / memo 块降级一次性诊断（正确性相关降级作者应知情；
/// AUTO_MEMO_DIAG=1 时每次都打）。进程级一次，避免日志风暴。
fn memo_diag(label: &str, reason: &str) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FOR_DIAG_SHOWN: AtomicBool = AtomicBool::new(false);
    if std::env::var("AUTO_MEMO_DIAG").ok().as_deref() == Some("1") {
        eprintln!("[MEMO-DIAG] {label} DEGRADE reason={reason}");
        return;
    }
    if !FOR_DIAG_SHOWN.swap(true, Ordering::Relaxed) {
        eprintln!(
            "[PLAN-046] {label} 降级原始路径（reason={reason}，本进程仅提示一次）；AUTO_MEMO_DIAG=1 可看每次"
        );
    }
}
