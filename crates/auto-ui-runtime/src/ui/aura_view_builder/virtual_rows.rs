//! PLAN-126 T-03：固定行高纵向虚拟列表（VList）的构建侧。
//!
//! 收录 `AuraViewBuilder` 的 pane 作用域管理入口与窗口化 `for` 构建；窗口几何
//! 台账（测量/反解）在 [`crate::ui::virtual_list`]。契约要点（plan §5.1）：
//!
//! - 作用域来自 `overflow-y-auto`/`overflow-auto` 列与 `scroll` 元素：转换
//!   自身子件期间压栈（线程本地，child widget builder 共享同一构建栈），子件
//!   中的 `for` 循环认领它（同 pane 只认领一个）。
//! - 认领后若台账已有「固定行高 + 列表起点 + pane 视口/偏移」，只构建可见行
//!   + 两侧 2 行 overscan，首尾用 `View::SpacePx` 表达屏外逻辑高度；构建带用
//!   `View::VirtualMark` 标记，供 probe 反解几何。
//! - 尚无几何时做**一次标定构建**：构建列表顶部至多 [`CALIBRATION_ROWS`] 行
//!   并打标记；probe 用「band 高度 / 实际行数」反解固定行高与列表起点，下一帧
//!   起按真实窗口裁行。
//! - 检索过滤态、嵌套循环绑定、dotted 属性路径或不可按索引读取的数据源一律
//!   返回 `None`，调用方保留完整构建语义。
//! - 索引语义：行绑定、事件参数与 `[for: …]` 记录一律使用**原数组索引**，不因
//!   窗口起点而重编号。

use super::*;
use crate::ui::virtual_list;

/// 标定帧最多构建的行数（无几何时的一次性构建；超出部分仍由占位表达）。
const CALIBRATION_ROWS: usize = 256;

/// pane 作用域 Drop guard（转换内提前 return 也不会漏 pop）。
pub(super) struct VlistScopeGuard<'b, 'a> {
    b: &'b AuraViewBuilder<'a>,
}

impl Drop for VlistScopeGuard<'_, '_> {
    fn drop(&mut self) {
        self.b.pop_vlist_scope();
    }
}

impl<'a> AuraViewBuilder<'a> {
    /// 进入虚拟列表 pane 作用域；返回的 guard 在离开作用域时自动出栈。
    pub(super) fn enter_vlist_scope(&self) -> VlistScopeGuard<'_, 'a> {
        virtual_list::push_scope();
        VlistScopeGuard { b: self }
    }

    /// 退出当前 pane 作用域（guard 用）。
    pub(super) fn pop_vlist_scope(&self) {
        virtual_list::pop_scope();
    }

    /// 当前最内层 pane 作用域 key。
    pub(super) fn vlist_scope(&self) -> Option<String> {
        virtual_list::current_scope()
    }

    /// 检索过滤是否生效（`search` 非空 → 可见索引与数组索引不再一一对应，
    /// 窗口化按 §5.1 回退完整构建）。
    fn vlist_search_active(&self) -> bool {
        match self.read_state("search") {
            Ok(Value::Str(s)) => !s.to_string().is_empty(),
            Ok(Value::String(s)) => !s.to_string().is_empty(),
            _ => false,
        }
    }

    /// 解析可窗口化的列表值（仅 state/store 标量路径；嵌套绑定与 dotted
    /// 属性路径一律回退）。
    ///
    /// 顺序与 `convert_node_with` 的 ForLoop 臂一致：先 `read_state`（可保住
    /// 堆引用 → O(1) 长度/按索引读取），失败再 `read_state_as_vec`（此时已付
    /// 一次整表展开，窗口化仍省下绑定/构建/布局）。
    fn vlist_list_value(&self, iterable: &str, bindings: &Bindings) -> Option<Value> {
        let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
        let store_field = self.store_source_field(stripped);
        if stripped.contains('.') && store_field.is_none() {
            return None;
        }
        let state_name = store_field.as_deref().unwrap_or(stripped);
        if bindings.get(state_name).is_some() {
            return None;
        }
        if let Ok(v) = self.read_state(state_name) {
            if self.bridge.list_len_of(&v).is_some() {
                return Some(v);
            }
        }
        match self.read_state_as_vec(state_name) {
            Ok(vec) => Some(Value::Array(auto_val::Array::from(vec))),
            Err(_) => {
                if virtual_list::diag() {
                    eprintln!("[VLIST] scope=… list source `{state_name}` not indexable");
                }
                None
            }
        }
    }

    /// 诊断覆盖：`AUTOUI_VLIST_ROW_H` / `AUTOUI_VLIST_VIEWPORT_H` /
    /// `AUTOUI_VLIST_LIST_TOP` / `AUTOUI_VLIST_OFFSET`（纯构建测试与故障定位；
    /// 生产缺省不用）。
    fn vlist_env_plan(total: usize) -> Option<virtual_list::WindowPlan> {
        let (row_h, list_top) = virtual_list::env_geometry_override()?;
        let viewport_h = std::env::var("AUTOUI_VLIST_VIEWPORT_H")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())?;
        let offset = std::env::var("AUTOUI_VLIST_OFFSET")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        let (start, end) = virtual_list::visible_range(
            offset,
            viewport_h,
            list_top,
            row_h,
            total,
            virtual_list::OVERSCAN_ROWS,
        );
        Some(virtual_list::WindowPlan {
            total,
            start,
            end,
            row_h,
            list_top,
            offset_y: offset,
            viewport_h,
        })
    }

    /// 尝试窗口化构建一个 `for` 循环；不可窗口化返回 `None`（调用方走完整构建）。
    pub(super) fn try_virtual_for(
        &self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        bindings: &Bindings,
    ) -> Option<View<DynamicMessage>> {
        if !virtual_list::enabled() {
            return None;
        }
        let scope = self.vlist_scope()?;
        if self.vlist_search_active() {
            return None;
        }
        let list_val = self.vlist_list_value(iterable, bindings)?;
        let total = self.bridge.list_len_of(&list_val)?;
        if total == 0 {
            return None;
        }

        // 台账几何优先 → 诊断覆盖 → 标定帧（顶部窗口，几何待 probe 反解）。
        let mut calibrating = false;
        let mut plan = match virtual_list::plan(&scope, total) {
            Some(p) => p,
            None => match Self::vlist_env_plan(total) {
                Some(p) => p,
                None => {
                    calibrating = true;
                    virtual_list::WindowPlan {
                        total,
                        start: 0,
                        end: total.min(CALIBRATION_ROWS),
                        row_h: 0.0,
                        list_top: 0.0,
                        offset_y: 0.0,
                        viewport_h: 0.0,
                    }
                }
            },
        };

        // 窗口内逐行读取 + 绑定 + 构建（绝对索引）。
        let mut rows: Vec<View<DynamicMessage>> = Vec::with_capacity(plan.rows());
        for i in plan.start..plan.end {
            let item = match self.bridge.list_get(&list_val, i) {
                Some(v) => v,
                // 索引读失败（非索引可读表示）→ 整体回退完整构建。
                None => return None,
            };
            let mut loop_bindings = bindings.clone();
            loop_bindings.insert(var.to_string(), self.bridge.materialize_obj_ref(&item));
            if let Some(idx_var) = index {
                loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
            }
            let views: Vec<View<DynamicMessage>> = body
                .iter()
                .map(|n| self.convert_node_with(n, &loop_bindings))
                .collect();
            if views.is_empty() {
                continue;
            }
            let row = if views.len() == 1 {
                let v = views.into_iter().next().unwrap();
                if matches!(v, View::Empty) {
                    continue;
                }
                v
            } else {
                View::Column {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                    on_right_click: None,
                }
            };
            rows.push(row);
        }

        if calibrating {
            // band 的行数用**实际构建行数**，行高在 probe 回执里按
            // 「band 高度 / 行数」反解；row_h 仍为 0 表示尚未解出。
            plan.end = plan.start + rows.len();
        }
        virtual_list::note_plan(&scope, plan);
        if !calibrating {
            virtual_list::record_build(&scope, &plan);
        }
        // 下一拍 probe 刷新 pane 几何 / 构建带 rect（首帧与滚动后都要）。
        virtual_list::request_measure();

        let band = View::VirtualMark {
            key: scope,
            child: Box::new(View::Column {
                children: rows,
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
                on_right_click: None,
            }),
        };
        let mut out: Vec<View<DynamicMessage>> = Vec::with_capacity(3);
        if plan.start > 0 && plan.row_h > 0.0 {
            out.push(View::SpacePx {
                width: f32::NAN,
                height: (plan.start as f64 * plan.row_h) as f32,
            });
        }
        out.push(band);
        let trailing = total.saturating_sub(plan.end);
        if trailing > 0 && plan.row_h > 0.0 {
            out.push(View::SpacePx {
                width: f32::NAN,
                height: (trailing as f64 * plan.row_h) as f32,
            });
        }
        Some(if out.len() == 1 {
            out.into_iter().next().unwrap()
        } else {
            View::Column {
                children: out,
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
                on_right_click: None,
            }
        })
    }
}
