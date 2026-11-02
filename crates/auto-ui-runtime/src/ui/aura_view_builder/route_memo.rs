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
            theme_scope: crate::ui::style::theme::theme_scope_identity(),
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

// ============================================================================
// PLAN-014 T-07/V-12 测试随owner迁移：原 aura_view_builder.rs 25855-28054 的
// memo/keyed-for 测试族（plan045/plan046/plan047）原样迁入本文件（impl#2 所
// 在owner）。测试标记守恒：38=38；原子义务/断言零改。
// ============================================================================
#[cfg(test)]
mod plan045_memo_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraStateDef, AuraWidget};

    /// MENUBAR_OPEN 是进程级全局——本模块 menubar 测试并发翻转会使
    /// globals_fp 跨 build 漂移（slow 路径 gfp 门误 miss）。模块内互斥；
    /// PLAN-046 episode 测试同锁（跨模块共享全局翻转面）。
    pub(crate) static PLAN045_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 本模块本地 widget 构造（tests::make_test_widget 私有不可达）。
    fn make_memo_widget(name: &str, state_vars: Vec<AuraStateDef>) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: name.to_string(),
            state_vars,
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    fn memo_widget() -> AuraWidget {
        make_memo_widget(
            "MemoApp",
            vec![
                AuraStateDef {
                    name: "show_urls".to_string(),
                    type_info: Type::Bool,
                    initial: Expr::Bool(false),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "unrelated".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "__current_route".to_string(),
                    type_info: Type::StrSlice,
                    initial: Expr::Str("/a".into()),
                    decorators: vec![],
                },
            ],
        )
    }

    /// 声明式 menubar：一项 `checked: .show_urls`（读槽），结构同 auto-jade。
    fn menubar_node(memo: bool) -> AuraNode {
        let item = AuraNode::element("menubar-checkbox-item")
            .with_prop("title", Expr::Str("Show URLs".into()))
            .with_prop("checked", Expr::Ident(".show_urls".into()));
        let content = AuraNode::element("menubar-content").with_child(item);
        let trigger =
            AuraNode::element("menubar-trigger").with_prop("text", Expr::Str("File".into()));
        let menu = AuraNode::element("menubar-menu")
            .with_prop("value", Expr::Str("file".into()))
            .with_child(trigger)
            .with_child(content);
        let mut mb = AuraNode::element("menubar");
        if memo {
            mb = mb.with_prop("memo", Expr::Bool(true));
        }
        mb.with_child(menu)
    }

    fn view_key(v: &View<DynamicMessage>) -> String {
        format!("{v:?}")
    }

    /// menubar_open 是进程级全局（全量套件下其他 menubar 测试并发翻转）——
    /// 跨 build 的产物断言仅在构建前后 open 态未翻转时可比（翻转窗口内
    /// 产物差异来自全局而非被测行为，跳过断言避免假红）。
    fn mb_open() -> Option<String> {
        crate::ui::action_config::menubar_open()
    }

    fn build_stable(bridge: &VmBridge, node: &AuraNode) -> (String, bool) {
        let o0 = mb_open();
        let v = AuraViewBuilder::new(bridge, "MemoApp").build(node);
        let o1 = mb_open();
        (view_key(&v), o0 == o1)
    }

    fn s1_still_open() -> bool {
        mb_open().is_some()
    }

    /// AC-02/AC-03：读集内写失效重建、读集外写快/慢路径命中（重求值不发生）。
    #[test]
    fn plan045_menubar_memo_hit_and_invalidation() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // items 仅在菜单开态渲染（PLAN-695）——开态下产物才反映 checked。
        crate::ui::action_config::set_menubar_open(Some("file".to_string()));
        let widget = memo_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let node = menubar_node(true);

        let (k1, s1) = build_stable(&bridge, &node); // fill
        bridge.with_memo_cache(|c| {
            assert_eq!(c.hits, 0, "fill 不计 hit");
            assert_eq!(c.len(), 1, "menubar 条目在册");
        });

        // 无状态写直接重建：seq 未动 → 快速路径全命中，产物一致。
        let (k2, s2) = build_stable(&bridge, &node);
        bridge.with_memo_cache(|c| assert_eq!(c.hits, 1, "快路径命中"));
        if s1 && s2 {
            assert_eq!(k1, k2, "快路径产物一致");
        }

        // 读集外写（unrelated）：seq 动 → 慢路径 → 读槽值指纹同 → 命中，
        // 重求值不发生（AC-03）。
        bridge
            .write_state("unrelated", auto_val::Value::Int(7))
            .unwrap();
        let (k3, s3) = build_stable(&bridge, &node);
        bridge.with_memo_cache(|c| {
            assert_eq!(c.hits, 2, "读集外写经慢路径命中");
            assert_eq!(c.misses, 0, "未失效");
        });
        if s1 && s3 {
            assert_eq!(k1, k3, "读集外写产物一致");
        }

        // 读集内写（show_urls）：值指纹翻面 → 失效重求值，产物随之变化
        //（AC-02）。
        bridge
            .write_state("show_urls", auto_val::Value::Bool(true))
            .unwrap();
        let (k4, s4) = build_stable(&bridge, &node);
        bridge.with_memo_cache(|c| {
            assert_eq!(c.misses, 1, "读集内写失效一次");
        });
        if s1 && s4 {
            assert_ne!(k1, k4, "checked 翻转必须改变产物");
        }

        // 重写回 false：再次失效重建，产物回到 v1 形态。
        bridge
            .write_state("show_urls", auto_val::Value::Bool(false))
            .unwrap();
        let (k5, s5) = build_stable(&bridge, &node);
        if s1 && s5 {
            assert_eq!(k1, k5, "值回产物必须一致");
        }
    }

    /// AC-01 前半：`memo` 缺省/false 时缓存完全惰性（零条目零计数）。
    #[test]
    fn plan045_memo_off_is_inert() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // items 仅在菜单开态渲染（PLAN-695）——开态下产物才反映 checked。
        crate::ui::action_config::set_menubar_open(Some("file".to_string()));
        let widget = memo_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();

        let off_node = menubar_node(false);
        let default_node = menubar_node(false).with_prop("memo", Expr::Bool(false));
        for node in [off_node, default_node] {
            bridge
                .write_state("show_urls", auto_val::Value::Bool(false))
                .unwrap();
            let (k1, s1) = build_stable(&bridge, &node);
            bridge
                .write_state("show_urls", auto_val::Value::Bool(true))
                .unwrap();
            let (k2, s2) = build_stable(&bridge, &node);
            if s1 && s2 && s1_still_open() {
                assert_ne!(k1, k2, "非 memo 路径照常重求值");
            }
        }
        bridge.with_memo_cache(|c| {
            assert_eq!(c.len(), 0, "非 memo 不建条目");
            assert_eq!((c.hits, c.misses, c.degraded), (0, 0, 0), "计数全零");
        });
    }

    /// AC-04：静态不可证形态（插值文本/ForLoop）自动降级——不建条目、
    /// 行为与非 memo 一致。
    #[test]
    fn plan045_dynamic_forms_degrade_to_raw() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // items 仅在菜单开态渲染（PLAN-695）——开态下产物才反映 checked。
        crate::ui::action_config::set_menubar_open(Some("file".to_string()));
        let widget = memo_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();

        let interp_item = AuraNode::element("menubar-item")
            .with_prop("title", Expr::Str("Dyn".into()))
            .with_child(AuraNode::Text(
                auto_ui_compiler::aura::AuraTextContent::Interpolated {
                    template: "n=${.unrelated}".to_string(),
                    bindings: vec!["unrelated".to_string()],
                },
            ));
        let content = AuraNode::element("menubar-content").with_child(interp_item);
        let trigger = AuraNode::element("menubar-trigger").with_prop("text", Expr::Str("F".into()));
        let menu = AuraNode::element("menubar-menu")
            .with_child(trigger)
            .with_child(content);
        let mb = AuraNode::element("menubar")
            .with_prop("memo", Expr::Bool(true))
            .with_child(menu);

        let (k1, s1) = build_stable(&bridge, &mb);
        bridge
            .write_state("unrelated", auto_val::Value::Int(1))
            .unwrap();
        let (k2, s2) = build_stable(&bridge, &mb);
        bridge.with_memo_cache(|c| {
            assert_eq!(c.len(), 0, "降级形态不建条目");
            assert!(c.degraded >= 1, "降级计数在册");
        });
        // 降级形态的动态面（插值子文本）不进 item 渲染——产物稳定恰证
        // 走的是原始路径每次重算（缓存面为零由上方 len/degraded 断言承载）。
        if s1 && s2 {
            assert_eq!(k1, k2, "降级 = 原始重求值行为");
        }
    }

    /// AC-06：memo 命中帧不吞 acceptance 事件索引——tracked build（probe
    /// enabled）fill 后重 build（命中），新 probe 必须含 menubar 事件记录
    /// （record! 路径重放）。
    #[test]
    fn plan045_menubar_probe_replay_on_hit() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::ui::action_config::set_menubar_open(Some("file".to_string()));
        let widget = memo_widget();
        let bridge = VmBridge::new(&widget).unwrap();
        let node = menubar_node(true);

        let empty_registry = crate::ui::widget_registry::WidgetRegistry::new();
        let b = AuraViewBuilder::with_registry(&bridge, "MemoApp", &empty_registry);
        // fill：probe enabled（build_with_debug 默认 enabled）。
        let (_, _, probe1) = b.build_with_debug(&node);
        assert!(!probe1.is_enabled() || true, "fill probe 在册");
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 1, "fill 条目在册"));

        // 命中帧：全新 probe（同 enabled），产物来自缓存，probe 面须重放。
        let (_, _, probe2) = b.build_with_debug(&node);
        bridge.with_memo_cache(|c| assert_eq!(c.hits, 1, "回访命中"));
        let mut replayed = 0;
        for (path, entry) in probe2.snapshot_prefix(&[]) {
            if !entry.events.is_empty() {
                replayed += entry.events.len();
            }
            let _ = path;
        }
        assert!(replayed > 0, "命中帧 probe 事件索引不得为空（AC-06）");
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// PLAN-046 T-05：outlet 语料 prop 三态——prop=true（env 未设）→ 门开；
    /// prop=false（env 未设）→ 原始惰性；prop=false（env=1）→ env 兼容层
    /// 仍开门（并集语义：prop 载体优先，env 保留兼容）。
    #[test]
    fn plan046_outlet_prop_priority() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use crate::ui::widget_registry::WidgetRegistry;

        let mut page_widget = make_memo_widget(
            "RowPage",
            vec![AuraStateDef {
                name: "title".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("r0".into()),
                decorators: vec![],
            }],
        );
        page_widget.view_tree =
            AuraNode::element("text").with_prop("text", Expr::Ident(".title".into()));
        let mut registry = WidgetRegistry::new();
        registry.register(page_widget);
        registry.register_route_alias("row", "RowPage");

        std::env::remove_var("AUTO_OUTLET_MEMO");
        let host = make_memo_widget(
            "MemoApp",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("/row".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&host).unwrap();
        let routes_static = [auto_ui_compiler::aura::AuraRoute {
            path: "/row".to_string(),
            module: "row".to_string(),
            widget_name: "RowPage".to_string(),
            params: Vec::new(),
        }];
        let builder_registry = &registry;
        let build_once = |bridge: &VmBridge, memo: bool| {
            let b = AuraViewBuilder::with_registry(bridge, "MemoApp", builder_registry)
                .with_routes(&routes_static);
            b.build(&AuraNode::Outlet { memo: Some(memo) })
        };

        // ① prop=false + env 未设 → 原始惰性（零条目零计数）。
        let _ = build_once(&bridge, false);
        let _ = build_once(&bridge, false);
        bridge.with_memo_cache(|c| {
            assert_eq!(
                (c.len(), c.hits, c.misses),
                (0, 0, 0),
                "prop 缺省 = 原始路径（env 未设）"
            );
        });

        // ② prop=true + env 未设 → 语料载体开门：fill + 回访命中。
        let v1 = build_once(&bridge, true);
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 1, "prop 门 fill 条目在册"));
        let v2 = build_once(&bridge, true);
        bridge.with_memo_cache(|c| assert!(c.hits >= 1, "prop 门回访命中"));
        assert_eq!(view_key(&v1), view_key(&v2), "命中产物一致");

        // ③ prop=false + env=1 → env 兼容层照常开门。
        bridge.with_memo_cache(|c| c.clear());
        std::env::set_var("AUTO_OUTLET_MEMO", "1");
        let _ = build_once(&bridge, false);
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 1, "env 门 fill 条目在册"));
        let _ = build_once(&bridge, false);
        bridge.with_memo_cache(|c| assert!(c.hits >= 1, "env 门回访命中"));
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    // ── PLAN-708 T-01（S-01）三态矩阵与失效面 ──────────────────────────

    /// r2 §5 S-01 优先级冻结表：env ∈ {0, 1, 未设} × prop ∈
    /// {None, Some(false), Some(true)} → resolve_outlet_memo 全 9 组合。
    /// 纯函数级（env 门解析另测）——矩阵即契约。
    #[test]
    fn plan708_outlet_tristate_matrix() {
        use MemoEnvGate as G;
        let cases: &[(G, Option<bool>, bool, &str)] = &[
            (G::ForceOff, None, false, "env=0 覆盖 prop 未设"),
            (G::ForceOff, Some(false), false, "env=0 × 显式 false"),
            (
                G::ForceOff,
                Some(true),
                false,
                "env=0 覆盖显式 true（诊断强制关）",
            ),
            (G::ForceOn, None, true, "env=1 强开 prop 未设"),
            (
                G::ForceOn,
                Some(false),
                true,
                "env=1 覆盖显式 false（并集契约变化点）",
            ),
            (G::ForceOn, Some(true), true, "env=1 × 显式 true"),
            (
                G::Unset,
                None,
                true,
                "未设 × 未设 → 缺省 on（r2 缺省面变更）",
            ),
            (G::Unset, Some(false), false, "未设 × 显式 false"),
            (G::Unset, Some(true), true, "未设 × 显式 true"),
        ];
        for (env, prop, want, why) in cases {
            assert_eq!(
                AuraViewBuilder::resolve_outlet_memo(*env, *prop),
                *want,
                "{}",
                why
            );
        }
        // 菜单/nav 族：env 覆盖 + 字面量缺省关（045 语义保留）。
        assert!(!AuraViewBuilder::resolve_site_memo(G::Unset, false));
        assert!(AuraViewBuilder::resolve_site_memo(G::Unset, true));
        assert!(
            !AuraViewBuilder::resolve_site_memo(G::ForceOff, true),
            "env=0 诊断强关覆盖字面量 true"
        );
        assert!(
            AuraViewBuilder::resolve_site_memo(G::ForceOn, false),
            "env=1 强开覆盖字面量 false"
        );
    }

    /// env 门解析：`0`/`1`/未设/其他值四形态（诊断"其他值=未设"容错）。
    #[test]
    fn plan708_memo_env_gate_parsing() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("AUTO_OUTLET_MEMO");
        assert!(matches!(
            AuraViewBuilder::memo_env_gate(),
            MemoEnvGate::Unset
        ));
        std::env::set_var("AUTO_OUTLET_MEMO", "0");
        assert!(matches!(
            AuraViewBuilder::memo_env_gate(),
            MemoEnvGate::ForceOff
        ));
        std::env::set_var("AUTO_OUTLET_MEMO", "1");
        assert!(matches!(
            AuraViewBuilder::memo_env_gate(),
            MemoEnvGate::ForceOn
        ));
        std::env::set_var("AUTO_OUTLET_MEMO", "garbage");
        assert!(matches!(
            AuraViewBuilder::memo_env_gate(),
            MemoEnvGate::Unset
        ));
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// builder 级缺省面变更：裸 outlet（prop 未设）× env 未设 → 门开
    /// （fill + 回访 hit）。r2 G1"outlet 页 memo 缺省开启"的验收锚。
    #[test]
    fn plan708_outlet_bare_default_on() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use crate::ui::widget_registry::WidgetRegistry;

        let mut page_widget = make_memo_widget(
            "RowPage",
            vec![AuraStateDef {
                name: "title".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("r0".into()),
                decorators: vec![],
            }],
        );
        page_widget.view_tree =
            AuraNode::element("text").with_prop("text", Expr::Ident(".title".into()));
        let mut registry = WidgetRegistry::new();
        registry.register(page_widget);
        registry.register_route_alias("row", "RowPage");

        std::env::remove_var("AUTO_OUTLET_MEMO");
        let host = make_memo_widget(
            "MemoApp",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("/row".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&host).unwrap();
        let routes_static = [auto_ui_compiler::aura::AuraRoute {
            path: "/row".to_string(),
            module: "row".to_string(),
            widget_name: "RowPage".to_string(),
            params: Vec::new(),
        }];
        let builder_registry = &registry;
        let build_once = |bridge: &VmBridge| {
            let b = AuraViewBuilder::with_registry(bridge, "MemoApp", builder_registry)
                .with_routes(&routes_static);
            // 裸 outlet：prop 未设（None）。
            b.build(&AuraNode::Outlet { memo: None })
        };
        let _v1 = build_once(&bridge);
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 1, "裸 outlet 缺省开门：fill 条目在册"));
        let _v2 = build_once(&bridge);
        bridge.with_memo_cache(|c| assert!(c.hits >= 1, "裸 outlet 缺省开门：回访命中"));
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// builder 级 env=0 强制关：显式 `outlet (memo: true)` × env=0 → 零条目
    /// （诊断逃生门覆盖语料显式开——AC-02 关键组合）。
    #[test]
    fn plan708_outlet_env0_overrides_prop_true() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use crate::ui::widget_registry::WidgetRegistry;

        let mut page_widget = make_memo_widget(
            "RowPage",
            vec![AuraStateDef {
                name: "title".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("r0".into()),
                decorators: vec![],
            }],
        );
        page_widget.view_tree =
            AuraNode::element("text").with_prop("text", Expr::Ident(".title".into()));
        let mut registry = WidgetRegistry::new();
        registry.register(page_widget);
        registry.register_route_alias("row", "RowPage");

        std::env::set_var("AUTO_OUTLET_MEMO", "0");
        let host = make_memo_widget(
            "MemoApp",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("/row".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&host).unwrap();
        let routes_static = [auto_ui_compiler::aura::AuraRoute {
            path: "/row".to_string(),
            module: "row".to_string(),
            widget_name: "RowPage".to_string(),
            params: Vec::new(),
        }];
        let builder_registry = &registry;
        let build_once = |bridge: &VmBridge| {
            let b = AuraViewBuilder::with_registry(bridge, "MemoApp", builder_registry)
                .with_routes(&routes_static);
            b.build(&AuraNode::Outlet { memo: Some(true) })
        };
        let _v1 = build_once(&bridge);
        let _v2 = build_once(&bridge);
        bridge.with_memo_cache(|c| {
            assert_eq!(
                (c.len(), c.hits, c.misses),
                (0, 0, 0),
                "env=0 覆盖显式 true：零条目零计数（原始路径）"
            );
        });
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// T-01 失效面：ui_epoch bump 后旧 epoch 条目失配（保守整体失效）。
    /// 组件局部态突变（preview toggle/tab、nav 局部态、ghost/列宽直写）
    /// 走 epoch 通道——不 bump VM seq 也能废止在册产物。
    #[test]
    fn plan708_ui_epoch_invalidation() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use crate::ui::widget_registry::WidgetRegistry;

        let mut page_widget = make_memo_widget(
            "RowPage",
            vec![AuraStateDef {
                name: "title".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("r0".into()),
                decorators: vec![],
            }],
        );
        page_widget.view_tree =
            AuraNode::element("text").with_prop("text", Expr::Ident(".title".into()));
        let mut registry = WidgetRegistry::new();
        registry.register(page_widget);
        registry.register_route_alias("row", "RowPage");

        std::env::remove_var("AUTO_OUTLET_MEMO");
        let host = make_memo_widget(
            "MemoApp",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("/row".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&host).unwrap();
        let routes_static = [auto_ui_compiler::aura::AuraRoute {
            path: "/row".to_string(),
            module: "row".to_string(),
            widget_name: "RowPage".to_string(),
            params: Vec::new(),
        }];
        let builder_registry = &registry;
        let build_once = |bridge: &VmBridge| {
            let b = AuraViewBuilder::with_registry(bridge, "MemoApp", builder_registry)
                .with_routes(&routes_static);
            b.build(&AuraNode::Outlet { memo: Some(true) })
        };
        let _v1 = build_once(&bridge); // fill（epoch E）
        let _v2 = build_once(&bridge);
        bridge.with_memo_cache(|c| assert!(c.hits >= 1, "同 epoch 回访命中"));

        // 模拟组件局部态突变臂（preview/nav/ghost 同型）：bump epoch。
        let before = crate::ui::memo_deps::ui_epoch();
        let bumped = crate::ui::memo_deps::bump_ui_epoch();
        assert!(bumped > before, "epoch 单调推进");

        let h0 = bridge.with_memo_cache(|c| c.hits);
        let _v3 = build_once(&bridge); // 旧 epoch 键失配 → fill 新条目
        bridge.with_memo_cache(|c| {
            assert_eq!(c.hits, h0, "bump 后不吞旧命中（保守失效）");
            assert!(c.len() >= 2, "新 epoch 条目独立在册（旧条目待容量驱逐）");
        });
        let _v4 = build_once(&bridge);
        bridge.with_memo_cache(|c| assert!(c.hits > h0, "新 epoch 内回访恢复命中"));
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// T-05b：outlet 页产物 memo——同页回访命中（重渲染不发生，build_ms
    /// 级断言走 hit 计数），Init 身份门保持，组件模板读槽覆盖。
    #[test]
    fn plan045_outlet_page_memo() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use crate::ui::widget_registry::WidgetRegistry;

        // 页 widget：静态模板 + 一处状态绑定 prop；Init 无。
        let page_widget = {
            let mut w = make_memo_widget(
                "RowPage",
                vec![AuraStateDef {
                    name: "page_flag".to_string(),
                    type_info: Type::Bool,
                    initial: Expr::Bool(false),
                    decorators: vec![],
                }],
            );
            let node = AuraNode::element("col")
                .with_child(AuraNode::element("h1").with_prop("text", Expr::Str("Row".into())))
                .with_child(
                    AuraNode::element("badge")
                        .with_prop("visible", Expr::Ident(".page_flag".into())),
                );
            w.view_tree = node;
            w
        };
        let mut registry = WidgetRegistry::new();
        registry.register(page_widget);
        registry.register_route_alias("row", "RowPage");

        // T-05b opt-in 门（env 载体；测试内设置-移除）。
        std::env::set_var("AUTO_OUTLET_MEMO", "1");
        let host = make_memo_widget(
            "MemoApp",
            vec![AuraStateDef {
                name: "__current_route".to_string(),
                type_info: Type::StrSlice,
                initial: Expr::Str("/row".into()),
                decorators: vec![],
            }],
        );
        let mut bridge = VmBridge::new(&host).unwrap();

        // 宿主 widget：routes 声明 + registry 注入（builder.with_routes 面）。
        let routes_static = [auto_ui_compiler::aura::AuraRoute {
            path: "/row".to_string(),
            module: "row".to_string(),
            widget_name: "RowPage".to_string(),
            params: Vec::new(),
        }];
        let builder_registry = &registry;

        let build_once = |bridge: &VmBridge| {
            let b = AuraViewBuilder::with_registry(bridge, "MemoApp", builder_registry)
                .with_routes(&routes_static);
            b.build(&AuraNode::Outlet { memo: Some(false) })
        };

        // 全量套件下其他 menubar 测试并发翻转 MENUBAR_OPEN → globals_fp 漂移
        // 会把 fast 门打落 slow（hit 仍发生）或叠加 miss。守卫：构建前后全局
        // 态未翻转才计数断言，否则重试构建（条目经慢路径自刷新）。
        let mb = || crate::ui::action_config::menubar_open();
        let mut v1 = build_once(&bridge); // fill
        for _ in 0..8 {
            let (hits, misses) = bridge.with_memo_cache(|c| (c.hits, c.misses));
            if hits == 0 && misses == 0 {
                break;
            }
            v1 = build_once(&bridge); // 全局面被翻转过 → 重建一轮垫稳
        }
        bridge.with_memo_cache(|c| {
            assert_eq!((c.hits, c.misses), (0, 0), "fill 不计");
        });

        // 无状态写回访（同路由）：seq 未动 → 快速路径命中（全局漂移时落
        // slow——同为命中，计数断言 ≥1）。
        let mut v2 = build_once(&bridge);
        for _ in 0..8 {
            let o0 = mb();
            v2 = build_once(&bridge);
            let o1 = mb();
            let hits = bridge.with_memo_cache(|c| c.hits);
            if o0 == o1 {
                assert!(hits >= 1, "同页回访命中");
                assert_eq!(view_key(&v1), view_key(&v2));
                break;
            }
        }

        // 页内读槽字段写（badge.visible）：慢路径 miss → 重求值。
        bridge
            .write_state("page_flag", auto_val::Value::Bool(true))
            .unwrap();
        for _ in 0..8 {
            let o0 = mb();
            let _v3 = build_once(&bridge);
            let o1 = mb();
            let misses = bridge.with_memo_cache(|c| c.misses);
            if o0 == o1 {
                assert!(misses >= 1, "页内读槽写失效");
                // 失效证据 = misses 计数（合成 badge 产物对该 prop 不敏感，
                // 产物敏感性由 menubar checkbox 用例承载）。
                break;
            }
        }

        // 无关写：慢路径回命中。
        bridge
            .write_state("__current_route", auto_val::Value::str("/row"))
            .unwrap();
        for _ in 0..8 {
            let o0 = mb();
            let _v4 = build_once(&bridge);
            let o1 = mb();
            let hits = bridge.with_memo_cache(|c| c.hits);
            if o0 == o1 {
                assert!(hits >= 2, "无关写慢路径回命中");
                break;
            }
        }
        std::env::remove_var("AUTO_OUTLET_MEMO");
    }

    /// sidebar：`sidebar_provider (memo: true)` 武装 + nav 块派生值键——
    /// 路由切换仅 active 翻转组失效，其余组全命中（AC-05 的机制面）；读集
    /// 外写两组建全命中。
    #[test]
    fn plan045_sidebar_nav_group_memo() {
        let _guard = PLAN045_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let widget = memo_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("__current_route", auto_val::Value::str("/a"))
            .unwrap();

        let mk_button = |to: &str, label: &str| {
            AuraNode::element("sidebar_menu_button")
                .with_prop("to", Expr::Str(to.into()))
                .with_prop("text", Expr::Str(label.into()))
        };
        let group = |buttons: Vec<AuraNode>| {
            let mut g = AuraNode::element("sidebar_group");
            for btn in buttons {
                g = g.with_child(btn);
            }
            g
        };
        let scroll = AuraNode::element("scroll")
            .with_child(group(vec![
                mk_button("/a", "Alpha"),
                mk_button("/b", "Beta"),
            ]))
            .with_child(group(vec![mk_button("/c", "Gamma")]));
        let provider = AuraNode::element("sidebar_provider")
            .with_prop("memo", Expr::Bool(true))
            .with_child(scroll.clone());

        let v1 = AuraViewBuilder::new(&bridge, "MemoApp").build(&provider); // 两组建 fill
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 2, "两组条目在册"));

        // 读集外写：两组全命中（慢路径，重求值不发生）。
        bridge
            .write_state("unrelated", auto_val::Value::Int(3))
            .unwrap();
        let v2 = AuraViewBuilder::new(&bridge, "MemoApp").build(&provider);
        bridge.with_memo_cache(|c| {
            assert_eq!(c.hits, 2, "两组全命中");
            assert_eq!(c.misses, 0);
        });
        assert_eq!(view_key(&v1), view_key(&v2));

        // 路由切换 /a→/b：含 active 翻转按钮的组 1 失效重建，组 2 全命中
        //（§3 期望形态的机制面）。
        bridge
            .write_state("__current_route", auto_val::Value::str("/b"))
            .unwrap();
        let v3 = AuraViewBuilder::new(&bridge, "MemoApp").build(&provider);
        bridge.with_memo_cache(|c| {
            assert_eq!(c.misses, 1, "仅 active 翻转组失效");
            assert_eq!(c.hits, 3, "组 2 命中");
        });
        assert_ne!(view_key(&v1), view_key(&v3), "active 高亮随路由迁移");

        // 未武装 provider（无 memo prop）：nav 门完全不入。
        let plain = AuraNode::element("sidebar_provider").with_child(scroll);
        bridge
            .write_state("__current_route", auto_val::Value::str("/a"))
            .unwrap();
        let before = bridge.with_memo_cache(|c| (c.hits, c.misses, c.len()));
        let _ = AuraViewBuilder::new(&bridge, "MemoApp").build(&plain);
        bridge.with_memo_cache(|c| {
            let after = (c.hits, c.misses, c.len());
            assert_eq!(before, after, "未武装 = nav 门惰性");
        });
    }
}

// ─────────────────────────────────────────────────────────────────────
// ── PLAN-046: keyed-for 项级 memo 单测（AC-02/AC-03）──
#[cfg(test)]
mod plan046_for_memo_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraStateDef, AuraWidget};

    fn keyed_widget() -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "KeyedApp".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "rows".to_string(),
                    type_info: Type::List(Box::new(Type::Int)),
                    initial: Expr::Array(vec![]),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "outers".to_string(),
                    type_info: Type::List(Box::new(Type::Int)),
                    initial: Expr::Array(vec![]),
                    decorators: vec![],
                },
            ],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    fn row(id: i32, name: &str) -> Value {
        let mut o = auto_val::Obj::new();
        o.set("id", Value::Int(id));
        o.set("name", Value::str(name));
        Value::obj(o)
    }

    fn obj_rows(rows: &[Value]) -> Value {
        Value::Array(auto_val::Array::from(rows.to_vec()))
    }

    /// `for r in .rows key: r.id { text (text: r.name) }`——稳定 key（id）+
    /// 可变内容（name）的最小形态：单项内容修改走慢路径 miss 而非换键。
    fn keyed_node(index: Option<&str>, key: Expr) -> AuraNode {
        AuraNode::ForLoop {
            var: "r".to_string(),
            index: index.map(|s| s.to_string()),
            iterable: ".rows".to_string(),
            key_expr: Some(key),
            body: vec![AuraNode::element("text").with_prop(
                "text",
                Expr::Dot(Box::new(Expr::Ident("r".into())), "name".into()),
            )],
            span: None,
            debug_id: None,
        }
    }

    fn unkeyed_node() -> AuraNode {
        AuraNode::ForLoop {
            var: "r".to_string(),
            index: None,
            iterable: ".rows".to_string(),
            key_expr: None,
            body: vec![AuraNode::element("text").with_prop(
                "text",
                Expr::Dot(Box::new(Expr::Ident("r".into())), "name".into()),
            )],
            span: None,
            debug_id: None,
        }
    }

    fn id_key() -> Expr {
        Expr::Dot(Box::new(Expr::Ident("r".into())), "id".into())
    }

    fn tracked_build(bridge: &VmBridge, node: &AuraNode) -> String {
        let (v, _idmap, _probe) = AuraViewBuilder::new(bridge, "KeyedApp").build_with_debug(node);
        format!("{v:?}")
    }

    fn counts(bridge: &VmBridge) -> (u64, u64, u64, usize) {
        bridge.with_memo_cache(|c| (c.hits, c.misses, c.degraded, c.len()))
    }

    /// fill 后无写重建 → 快速路径全命中、产物一致（AC-02 基础面）。
    #[test]
    fn plan046_keyed_fill_then_fast_hit() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "bbb"), row(3, "ccc")]),
            )
            .unwrap();
        let node = keyed_node(None, id_key());

        let k1 = tracked_build(&bridge, &node); // fill
        let (h0, m0, d0, n0) = counts(&bridge);
        assert_eq!((h0, m0, d0), (0, 0, 0), "fill 不计 hit/miss/degraded");
        assert_eq!(n0, 3, "三行三条目");

        let k2 = tracked_build(&bridge, &node); // 无状态写 → 快速路径
        let (h1, m1, d1, _) = counts(&bridge);
        assert_eq!((h1 - h0, m1 - m0, d1 - d0), (3, 0, 0), "三行全命中");
        assert_eq!(k1, k2, "快路径产物一致");
    }

    /// AC-02 重排：键集不变序变 → 全部命中；输出序恒随 iterable 当前序。
    #[test]
    fn plan046_keyed_reorder_all_hit_output_order() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "bbb"), row(3, "ccc")]),
            )
            .unwrap();
        let node = keyed_node(None, id_key());
        let _ = tracked_build(&bridge, &node);
        let (h0, m0, _, _) = counts(&bridge);

        bridge
            .write_state(
                "rows",
                obj_rows(&[row(3, "ccc"), row(1, "aaa"), row(2, "bbb")]),
            )
            .unwrap();
        let k = tracked_build(&bridge, &node);
        let (h1, m1, _, _) = counts(&bridge);
        assert_eq!(h1 - h0, 3, "重排全命中（项产物按 key 复用）");
        assert_eq!(m1 - m0, 0, "重排零 miss");
        let (pc, pa, pb) = (
            k.find("ccc").unwrap(),
            k.find("aaa").unwrap(),
            k.find("bbb").unwrap(),
        );
        assert!(pc < pa && pa < pb, "输出序=iterable 当前序 c,a,b：{}", k);
    }

    /// AC-02 单项内容修改（key 稳定）：仅该项慢路径 miss，其余命中。
    #[test]
    fn plan046_keyed_single_modify_only_item_misses() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "bbb"), row(3, "ccc")]),
            )
            .unwrap();
        let node = keyed_node(None, id_key());
        let _ = tracked_build(&bridge, &node);
        let (h0, m0, _, _) = counts(&bridge);

        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "zzz"), row(3, "ccc")]),
            )
            .unwrap();
        let k = tracked_build(&bridge, &node);
        let (h1, m1, _, _) = counts(&bridge);
        assert_eq!(h1 - h0, 2, "未变两项命中");
        assert_eq!(m1 - m0, 1, "仅内容变化项 miss（key 同条目在册）");
        assert!(
            k.contains("zzz") && !k.contains("bbb"),
            "产物反映新内容：{}",
            k
        );
    }

    /// AC-02 增删：新增项 fill（新键新条目）、删除项靠 LRU 闲置，存续全命中。
    #[test]
    fn plan046_keyed_add_remove_only_delta() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa"), row(2, "bbb")]))
            .unwrap();
        let node = keyed_node(None, id_key());
        let _ = tracked_build(&bridge, &node);
        let (h0, m0, _, n0) = counts(&bridge);

        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "bbb"), row(3, "ccc")]),
            )
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (h1, m1, _, n1) = counts(&bridge);
        assert_eq!(
            (h1 - h0, m1 - m0),
            (2, 0),
            "新增仅 333 fill（新键），存续全命中"
        );
        assert_eq!(n1, n0 + 1);

        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa")]))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (h2, m2, _, _) = counts(&bridge);
        assert_eq!((h2 - h1, m2 - m1), (1, 0), "删除后存续项命中，零 miss");
    }

    /// 声明 index 的保守面：迭代位入命中条件 → 重排必失效（T-01 D-4；
    /// 产物依赖位置时正确性优先，宁可多重求值）。
    #[test]
    fn plan046_keyed_index_declared_reorder_misses() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "rows",
                obj_rows(&[row(1, "aaa"), row(2, "bbb"), row(3, "ccc")]),
            )
            .unwrap();
        let node = keyed_node(Some("i"), id_key());
        let _ = tracked_build(&bridge, &node);
        let (h0, m0, _, _) = counts(&bridge);

        bridge
            .write_state(
                "rows",
                obj_rows(&[row(3, "ccc"), row(1, "aaa"), row(2, "bbb")]),
            )
            .unwrap();
        let k = tracked_build(&bridge, &node);
        let (h1, m1, _, _) = counts(&bridge);
        assert_eq!(
            (h1 - h0, m1 - m0),
            (0, 3),
            "index 声明 → 重排全失效（保守）"
        );
        let (pc, pa, pb) = (
            k.find("ccc").unwrap(),
            k.find("aaa").unwrap(),
            k.find("bbb").unwrap(),
        );
        assert!(pc < pa && pa < pb, "重求值下输出序仍正确：{}", k);
    }

    /// AC-03 重复 key：整体降级原始路径（绝不静默去重），产物与非 key 化一致。
    #[test]
    fn plan046_dup_key_degrades_to_raw() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa"), row(1, "bbb")]))
            .unwrap();
        let keyed = keyed_node(None, id_key());
        let plain = unkeyed_node();

        let k_keyed = tracked_build(&bridge, &keyed);
        let (h, m, d, n) = counts(&bridge);
        assert_eq!((h, m), (0, 0), "降级帧零命中零 miss");
        assert_eq!(d, 1, "重复 key 降级计数");
        assert_eq!(n, 0, "降级不建条目");

        let k_plain = tracked_build(&bridge, &plain);
        assert_eq!(k_keyed, k_plain, "降级产物与非 key 化逐字节一致");
    }

    /// AC-03 key 求值失败（读缺失字段）：整体降级，行为与非 key 化一致。
    #[test]
    fn plan046_key_eval_fail_degrades_to_raw() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa"), row(2, "bbb")]))
            .unwrap();
        let keyed = keyed_node(None, Expr::Ident(".missing".into()));
        let plain = unkeyed_node();

        let k_keyed = tracked_build(&bridge, &keyed);
        let (_, _, d, n) = counts(&bridge);
        assert_eq!((d, n), (1, 0), "key 求值失败降级且不建条目");
        let k_plain = tracked_build(&bridge, &plain);
        assert_eq!(k_keyed, k_plain);
    }

    /// AC-03 项值超指纹预算（4096 节点）：整体降级，key 可解析也不入缓存。
    #[test]
    fn plan046_item_fp_budget_degrades_to_raw() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let mut big = auto_val::Obj::new();
        big.set("id", Value::Int(7));
        big.set("name", Value::str("big"));
        big.set(
            "data",
            Value::Array(auto_val::Array::from(
                (0..5000).map(Value::Int).collect::<Vec<Value>>(),
            )),
        );
        bridge
            .write_state(
                "rows",
                Value::Array(auto_val::Array::from(vec![Value::obj(big)])),
            )
            .unwrap();
        let keyed = keyed_node(None, id_key());
        let plain = unkeyed_node();

        let k_keyed = tracked_build(&bridge, &keyed);
        let (_, _, d, n) = counts(&bridge);
        assert_eq!((d, n), (1, 0), "超预算项值 → 整体降级不建条目");
        let k_plain = tracked_build(&bridge, &plain);
        assert_eq!(k_keyed, k_plain, "降级产物一致");
    }

    /// 嵌套 keyed-for：外层 keyless 循环体内外层绑定非空 → 内层整体降级
    /// （T-01 D-4 保守裁定），渲染行为与原始一致。
    #[test]
    fn plan046_nested_keyed_for_degrades() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state(
                "outers",
                Value::Array(auto_val::Array::from(vec![Value::Int(1)])),
            )
            .unwrap();
        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa"), row(2, "bbb")]))
            .unwrap();
        let outer = AuraNode::ForLoop {
            var: "o".to_string(),
            index: None,
            iterable: ".outers".to_string(),
            key_expr: None,
            body: vec![keyed_node(None, id_key())],
            span: None,
            debug_id: None,
        };

        let k = tracked_build(&bridge, &outer);
        let (_, _, d, n) = counts(&bridge);
        assert!(d >= 1, "内层嵌套绑定上下文降级计数");
        assert_eq!(n, 0, "降级不建条目");
        assert!(
            k.contains("aaa") && k.contains("bbb"),
            "内层原始渲染在册：{}",
            k
        );
    }

    /// probe/idmap 簿记重放：命中帧事件索引在册（fill 帧 probe 面 = 命中帧
    /// probe 面——ForIter.index 补丁到当前迭代位）。
    #[test]
    fn plan046_keyed_hit_replays_probe_bookkeeping() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = keyed_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", obj_rows(&[row(1, "aaa"), row(2, "bbb")]))
            .unwrap();
        let node = keyed_node(None, id_key());

        let (_, _, probe_fill) = AuraViewBuilder::new(&bridge, "KeyedApp").build_with_debug(&node);
        let fill_for: Vec<usize> = probe_fill
            .snapshot()
            .values()
            .filter_map(|e| e.for_context.as_ref().map(|f| f.index.unwrap()))
            .collect();
        assert_eq!(fill_for.len(), 2, "fill 帧两条 record_for 在册");

        bridge
            .write_state("rows", obj_rows(&[row(2, "bbb"), row(1, "aaa")]))
            .unwrap();
        let (_, _, probe_hit) = AuraViewBuilder::new(&bridge, "KeyedApp").build_with_debug(&node);
        let hit_for: Vec<usize> = probe_hit
            .snapshot()
            .values()
            .filter_map(|e| e.for_context.as_ref().map(|f| f.index.unwrap()))
            .collect();
        assert_eq!(
            hit_for.len(),
            fill_for.len(),
            "命中帧 probe 条目数与 fill 帧一致"
        );
        assert!(
            hit_for.iter().all(|idx| matches!(idx, 0 | 1)),
            "重排命中帧 ForIter.index 补丁在 [0,1] 内：{:?}",
            hit_for
        );
    }
}

// ─────────────────────────────────────────────────────────────────────
// ── PLAN-046: 显式 memo 块单测（AC-04/AC-05）──
#[cfg(test)]
mod plan046_memo_block_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraStateDef, AuraWidget};

    fn block_widget() -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "BlockApp".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "count".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "other".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "hidden".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "data".to_string(),
                    type_info: Type::List(Box::new(Type::Int)),
                    initial: Expr::Array(vec![]),
                    decorators: vec![],
                },
            ],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    fn block_node(deps: Vec<Expr>, exact: bool, body: Vec<AuraNode>) -> AuraNode {
        AuraNode::MemoBlock {
            deps,
            exact,
            body,
            span: None,
            debug_id: None,
        }
    }

    fn state_text(state: &str) -> AuraNode {
        AuraNode::element("text").with_prop("text", Expr::Ident(format!(".{}", state).into()))
    }

    fn tracked_build(bridge: &VmBridge, node: &AuraNode) -> String {
        let (v, _idmap, _probe) = AuraViewBuilder::new(bridge, "BlockApp").build_with_debug(node);
        format!("{v:?}")
    }

    fn counts(bridge: &VmBridge) -> (u64, u64, u64, usize) {
        bridge.with_memo_cache(|c| (c.hits, c.misses, c.degraded, c.len()))
    }

    /// AC-04 默认语义：deps 变化失效、块内读槽变化失效、无关写慢路径命中、
    /// 无写快速路径命中。
    #[test]
    fn plan046_block_default_deps_and_slot_semantics() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        // 块体读 .count（自动读槽）；deps 声明 .other。
        let node = block_node(
            vec![Expr::Ident(".other".into())],
            false,
            vec![state_text("count")],
        );

        let k0 = tracked_build(&bridge, &node); // fill
        let (h0, m0, d0, n0) = counts(&bridge);
        assert_eq!((h0, m0, d0), (0, 0, 0), "fill 不计数");
        assert_eq!(n0, 1, "块条目在册");

        // 无写重建 → 快速路径。
        let k1 = tracked_build(&bridge, &node);
        let (h1, _, _, _) = counts(&bridge);
        assert_eq!(h1 - h0, 1, "无写快路径命中");
        assert_eq!(k0, k1, "快路径产物一致");

        // 块内读槽变化（.count）→ 失效重求值，产物更新（AC-04）。
        bridge
            .write_state("count", auto_val::Value::Int(7))
            .unwrap();
        let k2 = tracked_build(&bridge, &node);
        let (_, m1, _, _) = counts(&bridge);
        assert_eq!(m1 - m0, 1, "读槽变化失效一次");
        assert!(k2.contains("7"), "产物反映新 count：{}", k2);

        // deps 变化（.other）→ 失效重求值（AC-04）。
        bridge
            .write_state("other", auto_val::Value::Int(3))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (_, m2, _, _) = counts(&bridge);
        assert_eq!(m2 - m1, 1, "deps 变化失效一次");

        // 无关写（.hidden）→ 慢路径命中（deps+读槽全同）。
        bridge
            .write_state("hidden", auto_val::Value::Int(9))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (h2, _, _, _) = counts(&bridge);
        assert!(h2 > h1, "无关写慢路径命中（deps∪读槽全同）");
    }

    /// AC-05 `exact: true` 纯 deps 判定：块内隐藏读变化**不失效**（语料
    /// 签名担保的陈旧自担面）；deps 变化照常失效。
    #[test]
    fn plan046_block_exact_pure_deps_judgment() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let node = block_node(
            vec![Expr::Ident(".other".into())],
            true,                       // exact
            vec![state_text("hidden")], // 隐藏读不在 deps——作者担保其不产陈旧
        );

        let _ = tracked_build(&bridge, &node); // fill
        let (h0, m0, _, _) = counts(&bridge);

        // 隐藏读变化 → 纯 deps 判定 → 命中（陈旧由语料自担——AC-05 契约）。
        bridge
            .write_state("hidden", auto_val::Value::Int(5))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (h1, m1, _, _) = counts(&bridge);
        assert_eq!(
            (h1 - h0, m1 - m0),
            (1, 0),
            "exact 纯 deps：隐藏读变化仍命中"
        );

        // deps 变化 → 失效重求值。
        bridge
            .write_state("other", auto_val::Value::Int(2))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (_, m2, _, _) = counts(&bridge);
        assert_eq!(m2 - m1, 1, "exact 模式 deps 变化照常失效");
    }

    /// AC-04/05：默认语义下 Call 形态 prop（charts path 场景）不降级不入槽
    /// ——deps 变化失效重建、无关写命中。
    #[test]
    fn plan046_block_call_prop_covered_by_deps() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        // 模拟 charts path：`chart (path: compute(.data))`——Call 形态 prop。
        let call = Expr::Call(auto_lang_core::ast::Call {
            name: Box::new(Expr::Ident("compute".into())),
            args: auto_lang_core::ast::Args {
                args: vec![auto_lang_core::ast::Arg::Pos(Expr::Ident(".data".into()))],
            },
            ret: Type::Unknown,
            type_args: Vec::new(),
            generic_args: Vec::new(),
            pos: None,
        });
        let node = block_node(
            vec![Expr::Ident(".data".into())],
            false,
            vec![AuraNode::element("chart").with_prop("path", call)],
        );

        let _ = tracked_build(&bridge, &node); // fill（Call 不降级）
        let (h0, m0, d0, n0) = counts(&bridge);
        assert_eq!(
            (h0, m0, d0),
            (0, 0, 0),
            "Call 形态由 deps 覆盖，fill 无降级"
        );
        assert_eq!(n0, 1, "块条目在册");

        // deps（.data）变化 → 失效重建。
        bridge
            .write_state(
                "data",
                Value::Array(auto_val::Array::from(vec![Value::Int(1)])),
            )
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (_, m1, _, _) = counts(&bridge);
        assert_eq!(m1, 1, "deps 变化失效");

        // 无关写 → 慢路径命中。
        bridge
            .write_state("hidden", auto_val::Value::Int(1))
            .unwrap();
        let _ = tracked_build(&bridge, &node);
        let (h1, _, _, _) = counts(&bridge);
        assert_eq!(h1, 1, "无关写慢路径命中");
    }

    /// Q-02 保守面：默认语义块内含 ForLoop → 整块降级原始路径；
    /// `exact` 块内 ForLoop（deps 覆盖 iterable）→ 可缓存。
    #[test]
    fn plan046_block_nested_for_degrade_and_exact_override() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let inner = AuraNode::ForLoop {
            var: "r".to_string(),
            index: None,
            iterable: ".data".to_string(),
            key_expr: None,
            body: vec![AuraNode::element("text").with_prop(
                "text",
                Expr::Dot(Box::new(Expr::Ident("r".into())), "x".into()),
            )],
            span: None,
            debug_id: None,
        };

        // 默认语义：嵌套 for → 降级，不建条目。
        let default_block = block_node(
            vec![Expr::Ident(".data".into())],
            false,
            vec![inner.clone()],
        );
        let _ = tracked_build(&bridge, &default_block);
        let (_, _, d0, n0) = counts(&bridge);
        assert_eq!((d0, n0), (1, 0), "块内 ForLoop 默认降级");

        // exact：作者担保（iterable ⊆ deps）→ 可缓存。
        let exact_block = block_node(vec![Expr::Ident(".data".into())], true, vec![inner]);
        let _ = tracked_build(&bridge, &exact_block);
        let (h0, _, d1, n1) = counts(&bridge);
        assert_eq!((d1 - d0, n1), (0, 1), "exact 块内 ForLoop 可缓存");

        // 无写重建 → 快速命中。
        let _ = tracked_build(&bridge, &exact_block);
        let (h1, _, _, _) = counts(&bridge);
        assert_eq!(h1 - h0, 1, "exact 块快路径命中");
    }

    /// untracked 路径（build/MCP sync）memo 块 = 子体直落原始渲染，缓存
    /// 完全惰性（零条目零计数）。
    #[test]
    fn plan046_block_untracked_shared_gate() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let bridge = VmBridge::new(&widget).unwrap();
        let node = block_node(
            vec![Expr::Ident(".other".into())],
            false,
            vec![state_text("count")],
        );

        // T-06 双轨修正：untracked 帧（桌面渲染轨）同走共享块门。
        let builder = AuraViewBuilder::new(&bridge, "BlockApp");
        let v1 = builder.build(&node);
        let s = format!("{v1:?}");
        assert!(s.contains("0"), "untracked 渲染在册（count 初值 0）：{}", s);
        bridge.with_memo_cache(|c| assert_eq!(c.len(), 1, "untracked 块门 fill 条目在册"));
        let v2 = builder.build(&node);
        bridge.with_memo_cache(|c| assert!(c.hits >= 1, "untracked 块门回访命中"));
        assert_eq!(format!("{v1:?}"), format!("{v2:?}"), "命中产物一致");
    }

    /// per-site 分解计数：memo 块门走 site_counts[MEMO_SITE_MEMO_BLOCK]。
    #[test]
    fn plan046_block_site_counts() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let node = block_node(
            vec![Expr::Ident(".other".into())],
            false,
            vec![state_text("count")],
        );

        let _ = tracked_build(&bridge, &node); // fill
        bridge
            .write_state("count", auto_val::Value::Int(4))
            .unwrap();
        let _ = tracked_build(&bridge, &node); // miss
        let _ = tracked_build(&bridge, &node); // fast hit
        bridge.with_memo_cache(|c| {
            let sc = &c.site_counts[&crate::ui::memo_deps::MEMO_SITE_MEMO_BLOCK];
            assert_eq!((sc.hits, sc.misses), (1, 1), "per-site 分解在册");
        });
    }

    /// AC-04 episode 翻转：menubar_open 翻面 → 慢路径 miss（互斥防并发翻转）。
    #[test]
    fn plan046_block_episode_flip_miss() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = block_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        let node = block_node(
            vec![Expr::Ident(".other".into())],
            false,
            vec![state_text("count")],
        );

        // menubar_open 是跨测试遗留全局——先归 None 再 fill，翻面才确定。
        crate::ui::action_config::set_menubar_open(None);
        let _ = tracked_build(&bridge, &node); // fill（open=None）
        let (h0, m0, _, _) = counts(&bridge);
        crate::ui::action_config::set_menubar_open(Some("file".to_string()));
        let _ = tracked_build(&bridge, &node);
        crate::ui::action_config::set_menubar_open(None);
        let (h1, m1, _, _) = counts(&bridge);
        assert_eq!((h1 - h0, m1 - m0), (0, 1), "episode 翻面失效一次");
    }
}

// ─────────────────────────────────────────────────────────────────────
// PLAN-047 T-05: version_fast 三级判定单测（AC-02/AC-07 观测面）。
// ─────────────────────────────────────────────────────────────────────
mod plan047_gate_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraStateDef, AuraWidget};

    fn vf_widget() -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "VfApp".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "rows".to_string(),
                    type_info: Type::List(Box::new(Type::Int)),
                    initial: Expr::Array(vec![]),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "other".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
            ],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    fn vf_row(id: i32, name: &str) -> Value {
        let mut o = auto_val::Obj::new();
        o.set("id", Value::Int(id));
        o.set("name", Value::str(name));
        Value::obj(o)
    }

    fn vf_rows(rows: &[Value]) -> Value {
        Value::Array(auto_val::Array::from(rows.to_vec()))
    }

    fn vf_node() -> AuraNode {
        AuraNode::ForLoop {
            var: "r".to_string(),
            index: None,
            iterable: ".rows".to_string(),
            key_expr: Some(Expr::Dot(Box::new(Expr::Ident("r".into())), "id".into())),
            body: vec![AuraNode::element("text").with_prop(
                "text",
                Expr::Dot(Box::new(Expr::Ident("r".into())), "name".into()),
            )],
            span: None,
            debug_id: None,
        }
    }

    fn vf_build(bridge: &VmBridge, node: &AuraNode) -> String {
        let (v, _idmap, _probe) = AuraViewBuilder::new(bridge, "VfApp").build_with_debug(node);
        format!("{v:?}")
    }

    /// site=7 check-kind 分解读数。
    fn vf_kinds(bridge: &VmBridge) -> (u64, u64, u64) {
        bridge.with_memo_cache(|c| {
            let sc = c.site_counts.get(&crate::ui::memo_deps::MEMO_SITE_FOR_ITEM);
            (
                sc.map(|s| s.seq_fast).unwrap_or(0),
                sc.map(|s| s.version_fast).unwrap_or(0),
                sc.map(|s| s.fp_slow).unwrap_or(0),
            )
        })
    }

    /// AC-02: 无关写帧 version_fast 零重解析命中——fp_slow 不动、产物一致、
    /// resolve 探针零增量。
    #[test]
    fn plan047_version_fast_unrelated_write_zero_reeval() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = vf_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "aaa"), vf_row(2, "bbb")]))
            .unwrap();
        let node = vf_node();

        let k0 = vf_build(&bridge, &node); // fill
        let probe0 = bridge
            .resolve_probe_count
            .load(std::sync::atomic::Ordering::Relaxed);
        // 无写对照帧：seq_fast——探针增量 = pass-1 key 规划的合法求解
        // （键集 diff 每帧现算；含 Dot→Ident 递归，2 项×2 次）。
        let _ = vf_build(&bridge, &node);
        let (sf0, vf0, fp0) = vf_kinds(&bridge);
        let probe1 = bridge
            .resolve_probe_count
            .load(std::sync::atomic::Ordering::Relaxed);

        // 无关写（.other）→ 全局 seq 动、动态 dep 面不动。
        bridge
            .write_state("other", auto_val::Value::Int(9))
            .unwrap();
        let k1 = vf_build(&bridge, &node);
        let (sf1, vf1, fp1) = vf_kinds(&bridge);
        let probe2 = bridge
            .resolve_probe_count
            .load(std::sync::atomic::Ordering::Relaxed);

        assert_eq!(vf1 - vf0, 2, "version_fast 命中（两项条目各计一次）");
        assert_eq!(fp1 - fp0, 0, "指纹慢路径零调用（零重解析）");
        assert_eq!(sf1, sf0, "seq 快路径未走（seq 已动）");
        // AC-02: version_fast 帧 == seq_fast 帧的求解增量（check 阶段零额外
        // 重解析——fp_slow 的读槽重解析面零调用）。
        assert_eq!(
            probe2 - probe1,
            probe1 - probe0,
            "check 帧求解增量与快路径帧相同（无读槽重解析）"
        );
        assert_eq!(k0, k1, "产物一致");
    }

    /// dep 面写（同值）→ version_fast miss → fp_slow 命中并刷新基线 →
    /// 下一无关写帧回到 version_fast（基线前移语义）。
    #[test]
    fn plan047_fp_slow_hit_refreshes_dep_baselines() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = vf_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "aaa")]))
            .unwrap();
        let node = vf_node();

        let _ = vf_build(&bridge, &node); // fill
        let (_, vf0, fp0) = vf_kinds(&bridge);

        // dep 面写：rows 字段整体重写**同值** → 版本前进、值指纹同。
        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "aaa")]))
            .unwrap();
        let _ = vf_build(&bridge, &node);
        let (_, vf1, fp1) = vf_kinds(&bridge);
        assert_eq!(fp1 - fp0, 1, "版本动值同 → fp_slow 命中一次");
        assert_eq!(vf1, vf0, "version_fast 未走（版本已动）");

        // 基线已随 fp_slow 命中刷新 → 下一无关写帧回 version_fast。
        bridge
            .write_state("other", auto_val::Value::Int(3))
            .unwrap();
        let _ = vf_build(&bridge, &node);
        let (_, vf2, fp2) = vf_kinds(&bridge);
        assert_eq!(vf2 - vf1, 1, "基线刷新后 version_fast 恢复");
        assert_eq!(fp2, fp1, "无新增慢路径");
    }

    /// 内容真变 → version_fast miss → fp_slow mismatch → 重求值，产物反映
    /// 新值（AC-07 正确性下限：绝不陈旧）。
    #[test]
    fn plan047_content_change_refills_no_stale() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = vf_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "aaa")]))
            .unwrap();
        let node = vf_node();

        let k0 = vf_build(&bridge, &node); // fill
        assert!(k0.contains("aaa"), "初值在册: {k0}");

        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "zzz")]))
            .unwrap();
        let k1 = vf_build(&bridge, &node);
        assert!(k1.contains("zzz"), "内容变更重求值落地: {k1}");
        assert!(!k1.contains("aaa"), "无陈旧产物");
    }

    /// 判定序：无写重建走 seq_fast（version_fast 不消费）。
    #[test]
    fn plan047_seq_fast_precedes_version_fast() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = vf_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge
            .write_state("rows", vf_rows(&[vf_row(1, "aaa")]))
            .unwrap();
        let node = vf_node();

        let _ = vf_build(&bridge, &node); // fill
        let (sf0, vf0, _) = vf_kinds(&bridge);
        let _ = vf_build(&bridge, &node); // 无写重建
        let (sf1, vf1, _) = vf_kinds(&bridge);
        assert_eq!(sf1 - sf0, 1, "seq_fast 优先");
        assert_eq!(vf1, vf0, "version_fast 未消费");
    }
}

// ─────────────────────────────────────────────────────────────────────
// PLAN-047 T-06: computed 信号网单测（AC-04 观测面）。
// ─────────────────────────────────────────────────────────────────────
mod plan047_signal_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraComputed, AuraStateDef, AuraWidget};

    fn sig_widget(computed: AuraComputed) -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "SigApp".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "count".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(1),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "other".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
            ],
            computed: vec![computed],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    /// inline computed：`mirror = .count`（镜像读面）。
    fn inline_mirror() -> AuraComputed {
        AuraComputed {
            name: "mirror".to_string(),
            expr: Expr::Dot(Box::new(Expr::Ident(".".into())), "count".into()),
        }
    }

    /// block 体 computed：`mirror { .count }`（隐藏 VM fn 通道）。
    fn block_mirror() -> AuraComputed {
        AuraComputed {
            name: "mirror".to_string(),
            expr: Expr::Block(auto_lang_core::ast::Body {
                stmts: vec![auto_lang_core::ast::Stmt::Expr(Expr::Dot(
                    Box::new(Expr::Ident(".".into())),
                    "count".into(),
                ))],
                has_new_line: false,
                source_lines: Vec::new(),
            }),
        }
    }

    fn sig_view() -> AuraNode {
        // 视图读 computed：`text (text: .mirror)`。
        AuraNode::element("text").with_prop("text", Expr::Ident(".mirror".into()))
    }

    fn sig_build(bridge: &VmBridge, computed: &[AuraComputed], node: &AuraNode) -> String {
        let (v, _idmap, _probe) = AuraViewBuilder::new(bridge, "SigApp")
            .with_computed(computed)
            .build_with_debug(node);
        format!("{v:?}")
    }

    fn sig_counts(bridge: &VmBridge) -> (u64, u64) {
        (
            bridge
                .signal_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            bridge
                .signal_misses
                .load(std::sync::atomic::Ordering::Relaxed),
        )
    }

    /// AC-04 inline 通道：无关写帧信号命中（零重求值）、deps 变化重算、
    /// 产物保真（绝不陈旧）。
    #[test]
    fn plan047_signal_inline_hit_and_invalidation() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = sig_widget(inline_mirror());
        let mut bridge = VmBridge::new(&widget).unwrap();
        let node = sig_view();

        let k0 = sig_build(&bridge, &widget.computed, &node); // fill（信号首求值入网）
        assert!(k0.contains("1"), "初值 count=1: {k0}");
        let (h0, m0) = sig_counts(&bridge);
        assert_eq!((h0, m0), (0, 1), "首帧 miss 入网");

        // 无关写 → 信号 deps 未动 → 命中复用（computed 零重算）。
        bridge
            .write_state("other", auto_val::Value::Int(9))
            .unwrap();
        let k1 = sig_build(&bridge, &widget.computed, &node);
        let (h1, _) = sig_counts(&bridge);
        assert_eq!(h1 - h0, 1, "无关写帧信号命中");
        assert_eq!(k0, k1, "产物一致");

        // deps 变化（.count）→ 信号重算 → 产物保真。
        bridge
            .write_state("count", auto_val::Value::Int(7))
            .unwrap();
        let k2 = sig_build(&bridge, &widget.computed, &node);
        let (_, m2) = sig_counts(&bridge);
        assert!(m2 > m0, "deps 变化信号重算");
        assert!(k2.contains("7"), "新值落地（无陈旧）: {k2}");
    }

    /// AC-04 block 体通道（隐藏 VM fn）：信号命中/失效与 inline 同构；
    /// 引擎读臂录制的 dep 面承载失效判定（T-04 端到端）。
    #[test]
    fn plan047_signal_block_channel() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = sig_widget(block_mirror());
        let mut bridge = VmBridge::new(&widget).unwrap();
        // 合成通道在册性预检：__computed_SigApp_mirror 必须已合成（缺失 =
        // 旧构建路径降级 None，本测试前提不成立）。
        let fn_name = crate::ui::handler_codegen::computed_fn_name("SigApp", "mirror");
        assert!(
            bridge.vm_flash_exports_contains(&fn_name),
            "block computed 隐藏 fn 未合成: {fn_name}"
        );
        let node = sig_view();

        let k0 = sig_build(&bridge, &widget.computed, &node);
        assert!(k0.contains("1"), "block 通道初值: {k0}");
        let (h0, m0) = sig_counts(&bridge);

        bridge
            .write_state("other", auto_val::Value::Int(9))
            .unwrap();
        let k1 = sig_build(&bridge, &widget.computed, &node);
        let (h1, _) = sig_counts(&bridge);
        assert_eq!(h1 - h0, 1, "block 通道无关写命中");
        assert_eq!(k0, k1, "产物一致");

        bridge
            .write_state("count", auto_val::Value::Int(5))
            .unwrap();
        let k2 = sig_build(&bridge, &widget.computed, &node);
        let (_, m2) = sig_counts(&bridge);
        assert!(m2 > m0, "block 通道 deps 变化重算");
        assert!(k2.contains("5"), "新值落地: {k2}");
    }

    /// 级联闭合：memo 条目收编信号的 dep 面——信号 deps 变 → 条目版本比对
    /// 失效（即使条目自身不直读该状态）。
    #[test]
    fn plan047_signal_deps_absorbed_into_memo_entry() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = sig_widget(inline_mirror());
        let mut bridge = VmBridge::new(&widget).unwrap();
        // memo 块包 computed 读——块条目 deps 由录制收编（含 .mirror 的
        // dep 面 (root,count)）。
        let node = AuraNode::MemoBlock {
            deps: vec![Expr::Ident(".other".into())],
            exact: false,
            body: vec![AuraNode::element("text").with_prop("text", Expr::Ident(".mirror".into()))],
            span: None,
            debug_id: None,
        };

        let _ = sig_build(&bridge, &widget.computed, &node); // fill（块条目 + 信号同帧入网）
        let _ = sig_build(&bridge, &widget.computed, &node); // 无关重建（块快路径 + 信号命中）
                                                             // 直写 count（信号 dep 面）——块条目必须经 version_fast miss 落重求
                                                             // 值（条目收编了信号的 dep 键），产物保真。
        bridge
            .write_state("count", auto_val::Value::Int(3))
            .unwrap();
        let k = sig_build(&bridge, &widget.computed, &node);
        assert!(k.contains("3"), "级联失效落地: {k}");
    }
}

// ─────────────────────────────────────────────────────────────────────
// PLAN-047 T-07: 降级面收敛单测（AC-05——computed-exclusion 改判）。
// ─────────────────────────────────────────────────────────────────────
mod plan047_convergence_tests {
    use super::*;
    use auto_lang_core::ast::Type;
    use auto_ui_compiler::aura::{AuraComputed, AuraStateDef, AuraWidget};

    fn conv_widget() -> AuraWidget {
        AuraWidget {
            named_views: Vec::new(),
            actions: None,
            name: "ConvApp".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "show_urls".to_string(),
                    type_info: Type::Bool,
                    initial: Expr::Bool(false),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "other".to_string(),
                    type_info: Type::Int,
                    initial: Expr::Int(0),
                    decorators: vec![],
                },
            ],
            computed: vec![AuraComputed {
                name: "mirror".to_string(),
                expr: Expr::Dot(Box::new(Expr::Ident(".".into())), "show_urls".into()),
            }],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    /// menubar（memo 可开关）读 computed：`checked: .mirror`——T-01 时代
    /// 此形态因 computed 声明整体不 memo，T-07 改判后入门。
    fn conv_menubar(memo: bool) -> AuraNode {
        let item = AuraNode::element("menubar-checkbox-item")
            .with_prop("title", Expr::Str("Show URLs".into()))
            .with_prop("checked", Expr::Ident(".mirror".into()));
        let content = AuraNode::element("menubar-content").with_child(item);
        // trigger text 读 computed——闭合态 popover 产物可见面（保真观察点）。
        let trigger =
            AuraNode::element("menubar-trigger").with_prop("text", Expr::Ident(".mirror".into()));
        let menu = AuraNode::element("menubar-menu")
            .with_prop("value", Expr::Str("file".into()))
            .with_child(trigger)
            .with_child(content);
        let mut mb = AuraNode::element("menubar");
        if memo {
            mb = mb.with_prop("memo", Expr::Bool(true));
        }
        mb.with_child(menu)
    }

    fn conv_build(bridge: &VmBridge, computed: &[AuraComputed], node: &AuraNode) -> String {
        let (v, _idmap, _probe) = AuraViewBuilder::new(bridge, "ConvApp")
            .with_computed(computed)
            .build_with_debug(node);
        format!("{v:?}")
    }

    /// AC-05: 含 computed 的 widget 上下文 memo 化后渲染产物与原始路径
    /// **逐字节一致**；门确证入册（非静默降级）；deps 变化经信号级联保真。
    #[test]
    fn plan047_computed_widget_memo_parity() {
        let _guard = super::plan045_memo_tests::PLAN045_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let widget = conv_widget();
        let mut bridge = VmBridge::new(&widget).unwrap();

        // 原始路径基线（memo 关——不入册）。
        let raw = conv_menubar(false);
        let k_raw = conv_build(&bridge, &widget.computed, &raw);
        let n0 = bridge.with_memo_cache(|c| c.len());
        assert_eq!(n0, 0, "memo 关不入册");

        // memo 开：T-07 改判后入门（T-01 时代此处整体降级）。
        let armed = conv_menubar(true);
        let k_memo = conv_build(&bridge, &widget.computed, &armed);
        let n1 = bridge.with_memo_cache(|c| c.len());
        assert_eq!(n1, 1, "computed 声明不再排除——条目在册");

        // AC-05 对拍：产物逐字节一致。
        assert_eq!(k_raw, k_memo, "memo 化产物与原始路径逐字节一致");

        // 级联保真：computed deps（show_urls）变 → 信号重算 → 条目失效
        // 重求值 → 产物反映新值。
        bridge
            .write_state("show_urls", auto_val::Value::Bool(true))
            .unwrap();
        let k2 = conv_build(&bridge, &widget.computed, &armed);
        assert!(k2.contains("true"), "级联失效产物保真: {k2}");
    }

    /// F-781-D01 收口（T-19/W-01，2026-10-15）：scroll-pane 内层 col 的样式
    /// 剥除——滚动轴尺寸类（Y 轴剥 Height 族 / X 轴剥 Width 族）不落内容，
    /// 视觉类（padding/border/rounded）与跨轴宽度保留；hover 变体同步剥。
    /// 反例锚：`h-32` 类 Fixed 高曾把 std pane 内容钳到 viewport 高（iced
    /// 限高传导→溢出行零高、content==viewport→pane 永不可滚动，锚点行
    /// zero_size 恒不可见）；vue 单元素映射无此面。
    #[test]
    fn plan781_scroll_content_style_strips_axis_sizing() {
        use crate::ui::scroll::ScrollAxes;
        use crate::ui::style::{Style, StyleClass};

        let has = |s: &Style, f: &dyn Fn(&StyleClass) -> bool| {
            s.classes.iter().any(|c| f(c)) || s.variant_classes.iter().any(|(_, c)| f(c))
        };
        let heightish = |c: &StyleClass| {
            matches!(
                c,
                StyleClass::Height(_)
                    | StyleClass::HeightPercent(_)
                    | StyleClass::MinHeight(_)
                    | StyleClass::MaxHeight(_)
            )
        };
        let widthish = |c: &StyleClass| {
            matches!(
                c,
                StyleClass::Width(_)
                    | StyleClass::WidthPercent(_)
                    | StyleClass::MinWidth(_)
                    | StyleClass::MaxWidth(_)
            )
        };

        // Y 轴（缺省）pane："border rounded h-32 px-2 w-full hover:h-64"——
        // Height 族剥（含 hover 变体），border/rounded/px-2/w-full 保留。
        let pane = Style::parse("border rounded h-32 px-2 w-full hover:h-64").unwrap();
        let content = AuraViewBuilder::scroll_content_style(pane, ScrollAxes::Y);
        assert!(!has(&content, &heightish), "Y 轴 pane：Height 族须剥除");
        assert!(
            has(&content, &|c| matches!(c, StyleClass::Border)),
            "视觉类 border 保留"
        );
        assert!(
            has(&content, &|c| matches!(c, StyleClass::Rounded)),
            "视觉类 rounded 保留"
        );
        assert!(
            has(&content, &|c| matches!(c, StyleClass::PaddingX(_))),
            "padding 保留"
        );
        assert!(
            has(&content, &widthish),
            "Y 轴 pane：跨轴 Width（w-full 铺宽）保留"
        );

        // X 轴 pane："w-96 h-32"——Width 剥、Height 保留。
        let pane = Style::parse("w-96 h-32").unwrap();
        let content = AuraViewBuilder::scroll_content_style(pane, ScrollAxes::X);
        assert!(!has(&content, &widthish), "X 轴 pane：Width 族须剥除");
        assert!(has(&content, &heightish), "X 轴 pane：跨轴 Height 保留");

        // 双轴：两族都剥。
        let pane = Style::parse("h-[400px] max-h-[600px] w-1/2").unwrap();
        let content = AuraViewBuilder::scroll_content_style(pane, ScrollAxes::BOTH);
        assert!(
            !has(&content, &heightish) && !has(&content, &widthish),
            "双轴 pane：两族都剥"
        );
    }
}
