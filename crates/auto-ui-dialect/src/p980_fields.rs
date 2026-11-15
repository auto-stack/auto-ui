//! PLAN-980（B-DIALECT-I，IF-980-FIELDS）：`WidgetDecl` 完整字段分母的
//! **可执行清单** + 穷尽出现集抽取。
//!
//! 交接合同（PLAN-980 r10 §5 / 979 r11 §5 V-11）：按现行
//! [`crate::ast::WidgetDecl`] 及其全部引用类型逐字段建表——每行含稳定字段/
//! 构造 ID、来源（AST/Spec 锚点 + source hash）、原 span、父角色、类型/形状
//! 规则、适用性（是否必需/条件理由）、投影位置、共享验证准入与实际 consumer
//! owner。清单不是 CSV：本模块把它落成可执行数据 + 穷尽 walk——
//!
//! - [`widget_field_manifest`]：版本化必需清单（`ui.widget-decl/v1`）。
//! - [`P980_CONSTRUCTS`]：映射实现（抽取器认识的全部构造键）。
//! - [`widget_field_occurrences_with`]：`UiCode` → 字段出现集（穷尽 walk：
//!   每个实际构造一个出现；AST 未携带 span 的槽位 span 为 `None`，view 树
//!   带 `(byte_offset, byte_len)` 原 span）。
//! - 构造不在映射实现里 = [`FieldExtractRejection`]（`ui.field-unknown`，
//!   具名拒绝，绝不静默略过——AST 新增字段未登记分母时在此红出）。
//! - [`reconcile_field_manifest`]：清单 ↔ 映射实现双向对账（删清单行/删映射
//!   项分别可区分地红）。
//!
//! 覆盖判定（必需行缺席/未知 ID）由共享机制执行：语言侧
//! `ace-core::source::field_manifest`（接既有 `mapping_complete`/
//! `component_coverage` 的字段粒度层；`ui_domain_validation` 经 dev-dep 消费，
//! 本 crate 不引入 ACE 依赖）。

use crate::ast::*;
use crate::model::{UiCode, UiStmt};

/// 清单领域身份（进 PLAN-980/979 交接件：`<domain>/v<major>.<minor>`）。
pub const P980_FIELD_MANIFEST_DOMAIN: &str = "ui.widget-decl";
/// 清单版本（AST 分母变化时 minor 递增；删行/语义变化 major 递增）。
pub const P980_FIELD_MANIFEST_VERSION: (u16, u16) = (1, 0);

/// 一行的适用性（PLAN-980 r10 冻结语义；条件适用必带可核理由）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetFieldApplicability {
    /// 必需：在锚定 fixture（002-counter 原源）内出现即必须投影；覆盖时缺席 = 红。
    Required { mode: String },
    /// 可选：出现时投影，不出现不算缺。
    Optional { mode: String },
    /// 条件适用：须给可核理由；行不得以“不在某 Demo 中”删除。
    Conditional { mode: String, reason: String },
}

impl WidgetFieldApplicability {
    pub fn is_required(&self) -> bool {
        matches!(self, WidgetFieldApplicability::Required { .. })
    }
}

/// 必需字段/构造清单的一行（语义字段按 PLAN-980 r10 §5 冻结）。
#[derive(Debug, Clone)]
pub struct WidgetFieldRow {
    /// 稳定构造 ID（= 抽取器构造键；清单与映射实现按完整字符串连接）。
    pub id: &'static str,
    /// 来源：AST/Spec 锚点 + source hash（冻结身份，见模块文档）。
    pub origin: String,
    /// 锚定源内的原 span（`(byte_offset, byte_len)`；AST 无 span 的槽位为
    /// `None`，由锚定测试的独立定位器核对）。
    pub source_span: Option<(usize, usize)>,
    /// 父角色。
    pub parent_role: &'static str,
    /// 类型/形状规则（投影侧按此判同类型非法父槽）。
    pub type_rule: &'static str,
    /// 适用性与模式。
    pub applicability: WidgetFieldApplicability,
    /// 投影位置（领域投影/消费层）。
    pub projection_path: &'static str,
    /// 共享验证准入（核对本行的永久检查）。
    pub check_id: &'static str,
    /// 实际 consumer owner。
    pub consumer_owner: &'static str,
}

/// 抽取器产出的一个字段出现（构造 ID + 原 span——view 树为真实字节区间，
/// 其余槽位 AST 不携带 span，为 `None`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldOccurrence {
    pub id: &'static str,
    pub span: Option<(usize, usize)>,
}

/// 未登记构造的具名拒绝（AST 里出现了映射实现不认识的构造）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldExtractRejection {
    /// 恒为 `ui.field-unknown`。
    pub code: &'static str,
    /// 未登记的构造描述（AST 位置 + 种类）。
    pub construct: String,
    pub span: Option<(usize, usize)>,
}

/// `WidgetDecl` 声明头的两种拼写（UI 面 `widget` 规范；AGE Schema 面经
/// legacy-widget 兼容别名连接同一 `widget.head` 行，规范 `node` 拼写同-role）。
pub const P980_HEAD_SPELLINGS: [&str; 2] = ["widget", "node"];

/// 映射实现：抽取器认识的全部构造键（清单 ID 的超集恰为自身——
/// [`reconcile_field_manifest`] 双向对账）。按 ast.rs 冻结；新增 AST 字段
/// 必须在此登记，否则抽取以 `ui.field-unknown` 具名拒绝。
pub const P980_CONSTRUCTS: &[&str] = &[
    // ---- 模块级项（UiStmt）----
    "module.widget",
    "module.store",
    "module.actions",
    "module.view-fn",
    "module.style-recipe",
    "module.theme-data",
    "module.free-msg",
    "module.free-model",
    "module.free-view",
    "module.use-web",
    "use-web.entry",
    "use-web.entry.kind",
    "use-web.entry.call-args",
    "use-web.entry.ref-fields",
    // ---- widget 声明（WidgetDecl 19 字段）----
    "widget.head",
    "widget.name",
    "widget.msg",
    "widget.msg.variant",
    "widget.msg.variant.quoted",
    "widget.msg.variant.payload",
    "widget.msg.variant.payload-name",
    "widget.model",
    "widget.model.field",
    "widget.model.field.var",
    "widget.model.field.primary",
    "widget.model.field.decorator",
    "widget.computed",
    "widget.computed.property",
    "widget.view",
    "widget.view.named",
    "widget.on",
    "widget.on.handler",
    "widget.on.handler.stream",
    "widget.bind",
    "widget.bind.key",
    "widget.prop",
    "widget.prop.default",
    "widget.routes",
    "widget.routes.def",
    "widget.routes.def.param",
    "widget.lifecycle.method",
    "widget.design",
    "widget.design.entry",
    "widget.style",
    "widget.use.ext-import",
    "widget.use.ext-import.kind",
    "widget.use.ext-import.call-args",
    "widget.use.ext-import.ref-fields",
    "widget.watch.decl",
    "widget.watch.source",
    "widget.watch.immediate",
    "widget.watch.deep",
    "widget.setup",
    "widget.setup.ref-annotation",
    "widget.expose.name",
    "widget.actions.action",
    "widget.actions.action.attr",
    "widget.actions.menubar",
    "widget.actions.menu",
    "widget.actions.menu-item",
    "widget.actions.toolbar",
    "widget.actions.toolbar-item",
    "widget.timer",
    "widget.timer.entry",
    "widget.timer.entry.when",
    // ---- store 声明（StoreDecl）----
    "store.name",
    "store.msg",
    "store.msg.variant",
    "store.model",
    "store.model.field",
    "store.model.field.var",
    "store.model.field.primary",
    "store.model.field.decorator",
    "store.computed",
    "store.computed.property",
    "store.on",
    "store.on.handler",
    "store.on.handler.stream",
    "store.watch.decl",
    "store.watch.source",
    "store.watch.immediate",
    "store.watch.deep",
    "store.timer",
    "store.timer.entry",
    "store.timer.entry.when",
    // ---- view 树（widget.view/named view/view-fn/free view 共用）----
    "view.node.element",
    "view.prop",
    "view.prop.style-binding",
    "view.event",
    "view.event.inline",
    "view.node.text",
    "view.node.text.interp",
    "view.node.text.binding",
    "view.node.for",
    "view.node.for.index",
    "view.node.for.key",
    "view.node.if",
    "view.node.if.else",
    "view.node.component",
    "view.node.outlet",
    "view.node.outlet.memo",
    "view.node.memo",
    "view.node.memo.exact",
    "view.node.link",
    // ---- view fn / style recipe / theme data ----
    "view-fn.name",
    "view-fn.param",
    "style-recipe.name",
    "style-recipe.param",
    "style-recipe.body",
    "style-recipe.pub",
    "style-recipe.doc",
    "theme-data.name",
    "theme-data.pub",
    "theme-data.tree",
    "theme-data.doc",
];

/// 穷尽 walk 的工作集：构造键 → 出现（映射实现可注入，默认 [`P980_CONSTRUCTS`]）。
struct Walker {
    constructs: &'static [&'static str],
    out: Vec<FieldOccurrence>,
}

impl Walker {
    fn push(
        &mut self,
        id: &'static str,
        span: Option<(usize, usize)>,
    ) -> Result<(), FieldExtractRejection> {
        if self.constructs.contains(&id) {
            self.out.push(FieldOccurrence { id, span });
            Ok(())
        } else {
            Err(FieldExtractRejection {
                code: "ui.field-unknown",
                construct: id.to_string(),
                span,
            })
        }
    }

    fn model(&mut self, fields: &[ModelField], owner: &str) -> Result<(), FieldExtractRejection> {
        let (field, var, primary, decorator) = if owner == "widget" {
            (
                "widget.model.field",
                "widget.model.field.var",
                "widget.model.field.primary",
                "widget.model.field.decorator",
            )
        } else {
            (
                "store.model.field",
                "store.model.field.var",
                "store.model.field.primary",
                "store.model.field.decorator",
            )
        };
        for f in fields {
            self.push(field, None)?;
            if f.mutable {
                self.push(var, None)?;
            }
            if f.is_primary {
                self.push(primary, None)?;
            }
            for _d in &f.decorators {
                self.push(decorator, None)?;
            }
        }
        Ok(())
    }

    fn actions(&mut self, a: &ActionsBlock) -> Result<(), FieldExtractRejection> {
        for entry in &a.actions {
            self.push("widget.actions.action", None)?;
            if entry.title.is_some()
                || entry.icon.is_some()
                || entry.shortcut.is_some()
                || entry.enabled_if.is_some()
                || entry.checked_if.is_some()
            {
                self.push("widget.actions.action.attr", None)?;
            }
        }
        if let Some(menubar) = &a.menubar {
            self.push("widget.actions.menubar", None)?;
            for menu in &menubar.menus {
                self.push("widget.actions.menu", None)?;
                for _item in &menu.items {
                    self.push("widget.actions.menu-item", None)?;
                }
            }
        }
        if let Some(toolbar) = &a.toolbar {
            self.push("widget.actions.toolbar", None)?;
            for _item in &toolbar.items {
                self.push("widget.actions.toolbar-item", None)?;
            }
        }
        Ok(())
    }

    fn view(&mut self, node: &ViewNode) -> Result<(), FieldExtractRejection> {
        match node {
            ViewNode::Element {
                props,
                events,
                children,
                span,
                ..
            } => {
                self.push("view.node.element", *span)?;
                for p in props {
                    self.push("view.prop", *span)?;
                    if let ViewPropValue::StyleBinding(entries) = &p.value {
                        for _e in entries {
                            self.push("view.prop.style-binding", *span)?;
                        }
                    }
                }
                for e in events {
                    self.push("view.event", *span)?;
                    if e.inline.is_some() {
                        self.push("view.event.inline", *span)?;
                    }
                }
                for c in children {
                    self.view(c)?;
                }
            }
            ViewNode::Text(t) => {
                self.push("view.node.text", None)?;
                if let ViewText::Interpolated { bindings, .. } = t {
                    self.push("view.node.text.interp", None)?;
                    for _b in bindings {
                        self.push("view.node.text.binding", None)?;
                    }
                }
            }
            ViewNode::ForLoop {
                index,
                key_expr,
                body,
                span,
                ..
            } => {
                self.push("view.node.for", *span)?;
                if index.is_some() {
                    self.push("view.node.for.index", *span)?;
                }
                if key_expr.is_some() {
                    self.push("view.node.for.key", *span)?;
                }
                for c in body {
                    self.view(c)?;
                }
            }
            ViewNode::Conditional {
                then_body,
                else_body,
                span,
                ..
            } => {
                self.push("view.node.if", *span)?;
                for c in then_body {
                    self.view(c)?;
                }
                if let Some(else_body) = else_body {
                    self.push("view.node.if.else", *span)?;
                    for c in else_body {
                        self.view(c)?;
                    }
                }
            }
            ViewNode::Component {
                props,
                events,
                span,
                ..
            } => {
                self.push("view.node.component", *span)?;
                for _p in props {
                    self.push("view.prop", *span)?;
                }
                for _e in events {
                    self.push("view.event", *span)?;
                }
            }
            ViewNode::Outlet { memo } => {
                self.push("view.node.outlet", None)?;
                if memo.is_some() {
                    self.push("view.node.outlet.memo", None)?;
                }
            }
            ViewNode::MemoBlock {
                exact, body, span, ..
            } => {
                self.push("view.node.memo", *span)?;
                if *exact {
                    self.push("view.node.memo.exact", *span)?;
                }
                for c in body {
                    self.view(c)?;
                }
            }
            ViewNode::Link { children, span, .. } => {
                self.push("view.node.link", *span)?;
                for c in children {
                    self.view(c)?;
                }
            }
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &UiStmt) -> Result<(), FieldExtractRejection> {
        match stmt {
            UiStmt::WidgetDecl(w) => {
                self.push("module.widget", None)?;
                self.push("widget.head", None)?;
                self.push("widget.name", None)?;
                for m in &w.messages {
                    self.push("widget.msg", None)?;
                    for v in &m.variants {
                        self.push("widget.msg.variant", None)?;
                        if v.quoted {
                            self.push("widget.msg.variant.quoted", None)?;
                        }
                        if !v.payload.is_empty() {
                            self.push("widget.msg.variant.payload", None)?;
                        }
                        for p in &v.payload_names {
                            if p.is_some() {
                                self.push("widget.msg.variant.payload-name", None)?;
                            }
                        }
                    }
                }
                if let Some(model) = &w.model {
                    self.push("widget.model", None)?;
                    self.model(&model.fields, "widget")?;
                }
                if let Some(c) = &w.computed {
                    self.push("widget.computed", None)?;
                    for _p in &c.properties {
                        self.push("widget.computed.property", None)?;
                    }
                }
                if let Some(v) = &w.view {
                    self.push("widget.view", None)?;
                    self.view(&v.root)?;
                }
                for nv in &w.named_views {
                    self.push("widget.view.named", None)?;
                    self.view(&nv.1.root)?;
                }
                if let Some(on) = &w.on {
                    self.push("widget.on", None)?;
                    for h in &on.handlers {
                        self.push("widget.on.handler", None)?;
                        if h.stream.is_some() {
                            self.push("widget.on.handler.stream", None)?;
                        }
                    }
                }
                if let Some(b) = &w.bind {
                    self.push("widget.bind", None)?;
                    for _k in &b.bindings {
                        self.push("widget.bind.key", None)?;
                    }
                }
                for p in &w.props {
                    self.push("widget.prop", None)?;
                    if p.default.is_some() {
                        self.push("widget.prop.default", None)?;
                    }
                }
                if let Some(r) = &w.routes {
                    self.push("widget.routes", None)?;
                    for d in &r.routes {
                        self.push("widget.routes.def", None)?;
                        for _param in &d.params {
                            self.push("widget.routes.def.param", None)?;
                        }
                    }
                }
                for _l in &w.lifecycle {
                    self.push("widget.lifecycle.method", None)?;
                }
                if let Some(d) = &w.design {
                    self.push("widget.design", None)?;
                    for _e in &d.entries {
                        self.push("widget.design.entry", None)?;
                    }
                }
                if w.style.is_some() {
                    self.push("widget.style", None)?;
                }
                for e in &w.ext_imports {
                    self.push("widget.use.ext-import", None)?;
                    self.push("widget.use.ext-import.kind", None)?;
                    if !e.call_args.is_empty() {
                        self.push("widget.use.ext-import.call-args", None)?;
                    }
                    if !e.ref_fields.is_empty() {
                        self.push("widget.use.ext-import.ref-fields", None)?;
                    }
                }
                for wd in &w.watch {
                    self.push("widget.watch.decl", None)?;
                    for _s in &wd.sources {
                        self.push("widget.watch.source", None)?;
                    }
                    if wd.immediate {
                        self.push("widget.watch.immediate", None)?;
                    }
                    if wd.deep {
                        self.push("widget.watch.deep", None)?;
                    }
                }
                if let Some(setup) = &w.setup {
                    self.push("widget.setup", None)?;
                    for _ra in &setup.ref_annotations {
                        self.push("widget.setup.ref-annotation", None)?;
                    }
                }
                for _n in &w.expose {
                    self.push("widget.expose.name", None)?;
                }
                if let Some(a) = &w.actions {
                    self.actions(a)?;
                }
                if let Some(t) = &w.timer {
                    self.push("widget.timer", None)?;
                    for e in &t.entries {
                        self.push("widget.timer.entry", None)?;
                        if e.when.is_some() {
                            self.push("widget.timer.entry.when", None)?;
                        }
                    }
                }
            }
            UiStmt::StoreDecl(s) => {
                self.push("module.store", None)?;
                self.push("store.name", None)?;
                for m in &s.messages {
                    self.push("store.msg", None)?;
                    for _v in &m.variants {
                        self.push("store.msg.variant", None)?;
                    }
                }
                if let Some(model) = &s.model {
                    self.push("store.model", None)?;
                    self.model(&model.fields, "store")?;
                }
                if let Some(c) = &s.computed {
                    self.push("store.computed", None)?;
                    for _p in &c.properties {
                        self.push("store.computed.property", None)?;
                    }
                }
                if let Some(on) = &s.on {
                    self.push("store.on", None)?;
                    for h in &on.handlers {
                        self.push("store.on.handler", None)?;
                        if h.stream.is_some() {
                            self.push("store.on.handler.stream", None)?;
                        }
                    }
                }
                for wd in &s.watch {
                    self.push("store.watch.decl", None)?;
                    for _src in &wd.sources {
                        self.push("store.watch.source", None)?;
                    }
                    if wd.immediate {
                        self.push("store.watch.immediate", None)?;
                    }
                    if wd.deep {
                        self.push("store.watch.deep", None)?;
                    }
                }
                if let Some(t) = &s.timer {
                    self.push("store.timer", None)?;
                    for e in &t.entries {
                        self.push("store.timer.entry", None)?;
                        if e.when.is_some() {
                            self.push("store.timer.entry.when", None)?;
                        }
                    }
                }
            }
            UiStmt::ActionsDecl(a) => {
                self.push("module.actions", None)?;
                self.actions(a)?;
            }
            UiStmt::ViewFragmentDecl(f) => {
                self.push("module.view-fn", None)?;
                self.push("view-fn.name", None)?;
                for _p in &f.params {
                    self.push("view-fn.param", None)?;
                }
                self.view(&f.body)?;
            }
            UiStmt::StyleRecipeDecl(r) => {
                self.push("module.style-recipe", None)?;
                self.push("style-recipe.name", None)?;
                for _p in &r.params {
                    self.push("style-recipe.param", None)?;
                }
                self.push("style-recipe.body", None)?;
                if r.is_pub {
                    self.push("style-recipe.pub", None)?;
                }
                if r.doc.is_some() {
                    self.push("style-recipe.doc", None)?;
                }
            }
            UiStmt::ThemeDataDecl(t) => {
                self.push("module.theme-data", None)?;
                self.push("theme-data.name", None)?;
                if t.is_pub {
                    self.push("theme-data.pub", None)?;
                }
                self.push("theme-data.tree", None)?;
                if t.doc.is_some() {
                    self.push("theme-data.doc", None)?;
                }
            }
            UiStmt::MsgDecl(_) => self.push("module.free-msg", None)?,
            UiStmt::ModelBlock(_) => self.push("module.free-model", None)?,
            UiStmt::ViewBlock(v) => {
                self.push("module.free-view", None)?;
                self.view(&v.root)?;
            }
            UiStmt::UseWeb(entries) => {
                self.push("module.use-web", None)?;
                for e in entries {
                    self.push("use-web.entry", None)?;
                    self.push("use-web.entry.kind", None)?;
                    if !e.call_args.is_empty() {
                        self.push("use-web.entry.call-args", None)?;
                    }
                    if !e.ref_fields.is_empty() {
                        self.push("use-web.entry.ref-fields", None)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// `UiCode` → 穷尽字段出现集（默认映射实现 = [`P980_CONSTRUCTS`]）。
pub fn widget_field_occurrences(
    ui: &UiCode,
) -> Result<Vec<FieldOccurrence>, FieldExtractRejection> {
    widget_field_occurrences_with(ui, P980_CONSTRUCTS)
}

/// 同 [`widget_field_occurrences`]，但映射实现可注入（反控：删一项映射 →
/// 抽取以 `ui.field-unknown` 具名拒绝；恢复 → 绿）。
pub fn widget_field_occurrences_with(
    ui: &UiCode,
    constructs: &'static [&'static str],
) -> Result<Vec<FieldOccurrence>, FieldExtractRejection> {
    let mut walker = Walker {
        constructs,
        out: Vec::new(),
    };
    for stmt in ui.domain_stmts() {
        walker.stmt(stmt)?;
    }
    Ok(walker.out)
}

/// 清单 ↔ 映射实现双向对账：每个构造键恰有一行、每行的 ID 在构造集中。
/// 删清单行（映射仍在）→ 该构造缺行红；加多余行 → 无构造可产红。
pub fn reconcile_field_manifest(
    rows: &[WidgetFieldRow],
    constructs: &[&'static str],
) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    let ids: Vec<&str> = rows.iter().map(|r| r.id).collect();
    for c in constructs {
        if !ids.contains(c) {
            problems.push(format!("construct `{c}` has no manifest row"));
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for id in &ids {
        if !constructs.contains(id) {
            problems.push(format!("manifest row `{id}` maps to no known construct"));
        }
        if seen.contains(id) {
            problems.push(format!("manifest row `{id}` is duplicated"));
        }
        seen.push(id);
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

// ---------------------------------------------------------------------------
// 版本化必需清单（分母数据；冻结来源：auto-ui-dialect/src/ast.rs @
// 1d10f5ff2566422e…、route.rs、examples/ui/002-counter/src/front/app.at @
// 04937c35325faa74…、docs/specs/ui/*（结构/实例规约）。行按 P980_CONSTRUCTS
// 顺序补全；适用性以 002-counter 锚定为 Required 基准，其余为可选/条件——
// 条件行不得以“不在 Hello/002 中”删除。）
// ---------------------------------------------------------------------------

fn required(mode: &str) -> WidgetFieldApplicability {
    WidgetFieldApplicability::Required {
        mode: mode.to_string(),
    }
}
fn optional(mode: &str) -> WidgetFieldApplicability {
    WidgetFieldApplicability::Optional {
        mode: mode.to_string(),
    }
}
fn conditional(mode: &str, reason: &str) -> WidgetFieldApplicability {
    WidgetFieldApplicability::Conditional {
        mode: mode.to_string(),
        reason: reason.to_string(),
    }
}

fn row(
    id: &'static str,
    parent: &'static str,
    type_rule: &'static str,
    applicability: WidgetFieldApplicability,
    projection_path: &'static str,
    check_id: &'static str,
    consumer_owner: &'static str,
) -> WidgetFieldRow {
    WidgetFieldRow {
        id,
        origin: format!(
            "auto-ui-dialect/src/ast.rs::WidgetDecl refs [{id}] sha256/16=1d10f5ff2566422e"
        ),
        source_span: None,
        parent_role: parent,
        type_rule,
        applicability,
        projection_path,
        check_id,
        consumer_owner,
    }
}

const HANDOFF_CHECK: &str = "p980_required_fields_handoff";
/// 删除反控（p980_required_field_mapping_controls）实际变异的目标行——
/// 删映射项用 `widget.msg`（出现密集、缺席立即可见），删清单行用
/// `widget.design.entry`（002 原业务的代表槽，themes/model/design 不得裁）。
pub const P980_CONTROL_TARGET_ROWS: [&str; 2] = ["widget.msg", "widget.design.entry"];

/// 完整 `WidgetDecl` 字段/构造分母（版本化必需清单）。
pub fn widget_field_manifest() -> Vec<WidgetFieldRow> {
    vec![
        row("module.widget", "module", "widget decl item", required("002-counter"), "UiStmt::WidgetDecl", HANDOFF_CHECK, "980/DIALECT"),
        row("module.store", "module", "store decl item (view-less widget)", optional("store modules"), "UiStmt::StoreDecl", HANDOFF_CHECK, "980/STATE"),
        row("module.actions", "module", "top-level actions block", optional("app-shell projects"), "UiStmt::ActionsDecl", HANDOFF_CHECK, "980/PARITY"),
        row("module.view-fn", "module", "view fn fragment decl", optional("view-fn projects"), "UiStmt::ViewFragmentDecl", HANDOFF_CHECK, "980/DIALECT"),
        row("module.style-recipe", "module", "style recipe decl", optional("style-recipe projects"), "UiStmt::StyleRecipeDecl", HANDOFF_CHECK, "980/WEB"),
        row("module.theme-data", "module", "pub theme data decl", optional("theme packages"), "UiStmt::ThemeDataDecl", HANDOFF_CHECK, "980/PARITY"),
        row("module.free-msg", "module", "free msg block", conditional("non-widget module context", "legacy/scenario contexts parse free blocks"), "UiStmt::MsgDecl", HANDOFF_CHECK, "980/DIALECT"),
        row("module.free-model", "module", "free model block", conditional("non-widget module context", "legacy/scenario contexts parse free blocks"), "UiStmt::ModelBlock", HANDOFF_CHECK, "980/DIALECT"),
        row("module.free-view", "module", "free view block", conditional("non-widget module context", "legacy/scenario contexts parse free blocks"), "UiStmt::ViewBlock", HANDOFF_CHECK, "980/DIALECT"),
        row("module.use-web", "module", "use { ... } web import block", optional("web backends"), "UiStmt::UseWeb", HANDOFF_CHECK, "980/WEB"),
        row("use-web.entry", "module.use-web", "one ext import entry", optional("web backends"), "ExtImport", HANDOFF_CHECK, "980/WEB"),
        row("use-web.entry.kind", "use-web.entry", "Fn/ExplicitFn/Component/Composable/Package", optional("web backends"), "ExtImport.kind", HANDOFF_CHECK, "980/WEB"),
        row("use-web.entry.call-args", "use-web.entry", "composable call args", optional("composables"), "ExtImport.call_args", HANDOFF_CHECK, "980/WEB"),
        row("use-web.entry.ref-fields", "use-web.entry", "refs: [...] annotation", optional("composables"), "ExtImport.ref_fields", HANDOFF_CHECK, "980/WEB"),
        row("widget.head", "module.widget", "declaration head spelling `widget` (AGE face: canonical `node` + legacy alias same role)", required("002-counter"), "WidgetDecl head / AGE ComponentNode.head", HANDOFF_CHECK, "980/DIALECT"),
        row("widget.name", "widget.head", "ident", required("002-counter"), "WidgetDecl.name", HANDOFF_CHECK, "980/DIALECT"),
        row("widget.msg", "widget", "msg block", required("002-counter"), "WidgetDecl.messages", HANDOFF_CHECK, "980/STATE"),
        row("widget.msg.variant", "widget.msg", "Variant [payload]", required("002-counter"), "MsgDecl.variants", HANDOFF_CHECK, "980/STATE"),
        row("widget.msg.variant.quoted", "widget.msg.variant", "quoted contractual emit name", optional("contractual emits"), "MsgVariant.quoted", HANDOFF_CHECK, "980/WEB"),
        row("widget.msg.variant.payload", "widget.msg.variant", "tuple payload types", optional("payload variants"), "MsgVariant.payload", HANDOFF_CHECK, "980/STATE"),
        row("widget.msg.variant.payload-name", "widget.msg.variant", "named payload param", optional("named payloads"), "MsgVariant.payload_names", HANDOFF_CHECK, "980/WEB"),
        row("widget.model", "widget", "model block", required("002-counter"), "WidgetDecl.model", HANDOFF_CHECK, "980/STATE"),
        row("widget.model.field", "widget.model", "name ty = init", required("002-counter"), "ModelBlock.fields", HANDOFF_CHECK, "980/STATE"),
        row("widget.model.field.var", "widget.model.field", "var mutability keyword", required("002-counter"), "ModelField.mutable", HANDOFF_CHECK, "980/STATE"),
        row("widget.model.field.primary", "widget.model.field", "primary-property shorthand flag", optional("primary props"), "ModelField.is_primary", HANDOFF_CHECK, "980/STATE"),
        row("widget.model.field.decorator", "widget.model.field", "#[Provide]/#[Consume] decorator", optional("state mgmt decorators"), "ModelField.decorators", HANDOFF_CHECK, "980/STATE"),
        row("widget.computed", "widget", "computed block", optional("derived state"), "WidgetDecl.computed", HANDOFF_CHECK, "980/STATE"),
        row("widget.computed.property", "widget.computed", "name => expr", optional("derived state"), "ComputedBlock.properties", HANDOFF_CHECK, "980/STATE"),
        row("widget.view", "widget", "view block", required("002-counter"), "WidgetDecl.view", HANDOFF_CHECK, "980/DIALECT"),
        row("widget.view.named", "widget", "named view face `view <name> { ... }`", optional("multi-face widgets"), "WidgetDecl.named_views", HANDOFF_CHECK, "980/PARITY"),
        row("widget.on", "widget", "on block", required("002-counter"), "WidgetDecl.on", HANDOFF_CHECK, "980/STATE"),
        row("widget.on.handler", "widget.on", "pattern => body", required("002-counter"), "OnBlock.handlers", HANDOFF_CHECK, "980/STATE"),
        row("widget.on.handler.stream", "widget.on.handler", "on stream sse(url) subscription", optional("stream handlers"), "OnHandler.stream", HANDOFF_CHECK, "980/WEB"),
        row("widget.bind", "widget", "bind block (key bindings)", optional("bind blocks"), "WidgetDecl.bind", HANDOFF_CHECK, "980/NATIVE"),
        row("widget.bind.key", "widget.bind", "\"key\" -> .Handler", optional("bind blocks"), "BindBlock.bindings", HANDOFF_CHECK, "980/NATIVE"),
        row("widget.prop", "widget.head.params", "prop decl name ty [= default]", optional("reusable components"), "WidgetDecl.props", HANDOFF_CHECK, "980/DIALECT"),
        row("widget.prop.default", "widget.prop", "default value expr", optional("reusable components"), "PropDecl.default", HANDOFF_CHECK, "980/DIALECT"),
        row("widget.routes", "widget", "routes block", conditional("router widgets", "only router widgets declare routes (Plan 105)"), "WidgetDecl.routes", HANDOFF_CHECK, "980/WEB"),
        row("widget.routes.def", "widget.routes", "\"path\" -> use module", conditional("router widgets", "only router widgets declare routes"), "RoutesBlock.routes", HANDOFF_CHECK, "980/WEB"),
        row("widget.routes.def.param", "widget.routes.def", ":param path segment", conditional("parametrized routes", "only :param routes carry params"), "RouteDef.params", HANDOFF_CHECK, "980/WEB"),
        row("widget.lifecycle.method", "widget", "lifecycle { aboutToAppear => ... }", optional("lifecycle hooks"), "WidgetDecl.lifecycle", HANDOFF_CHECK, "980/STATE"),
        row("widget.design", "widget", "design block", required("002-counter"), "WidgetDecl.design", HANDOFF_CHECK, "980/PARITY"),
        row("widget.design.entry", "widget.design", "theme:/mode: binding expr", required("002-counter"), "DesignBlock.entries", HANDOFF_CHECK, "980/PARITY"),
        row("widget.style", "widget", "raw CSS style block", optional("vue style blocks"), "WidgetDecl.style", HANDOFF_CHECK, "980/WEB"),
        row("widget.use.ext-import", "widget", "use { ... } ext import entry", optional("ext imports"), "WidgetDecl.ext_imports", HANDOFF_CHECK, "980/WEB"),
        row("widget.use.ext-import.kind", "widget.use.ext-import", "Fn/ExplicitFn/Component/Composable/Package", optional("ext imports"), "ExtImport.kind", HANDOFF_CHECK, "980/WEB"),
        row("widget.use.ext-import.call-args", "widget.use.ext-import", "composable call args", optional("composables"), "ExtImport.call_args", HANDOFF_CHECK, "980/WEB"),
        row("widget.use.ext-import.ref-fields", "widget.use.ext-import", "refs: [...] annotation", optional("composables"), "ExtImport.ref_fields", HANDOFF_CHECK, "980/WEB"),
        row("widget.watch.decl", "widget", "watch decl (sources => body)", optional("watch blocks"), "WidgetDecl.watch", HANDOFF_CHECK, "980/WEB"),
        row("widget.watch.source", "widget.watch.decl", "watched field name", optional("watch blocks"), "WatchDecl.sources", HANDOFF_CHECK, "980/WEB"),
        row("widget.watch.immediate", "widget.watch.decl", ".immediate modifier", optional("watch blocks"), "WatchDecl.immediate", HANDOFF_CHECK, "980/WEB"),
        row("widget.watch.deep", "widget.watch.decl", ".deep modifier", optional("watch blocks"), "WatchDecl.deep", HANDOFF_CHECK, "980/WEB"),
        row("widget.setup", "widget", "setup block", optional("setup preambles"), "WidgetDecl.setup", HANDOFF_CHECK, "980/WEB"),
        row("widget.setup.ref-annotation", "widget.setup", "refs: [...] on let", optional("setup refs"), "SetupBlock.ref_annotations", HANDOFF_CHECK, "980/WEB"),
        row("widget.expose.name", "widget", "expose { .name } entry", optional("expose blocks"), "WidgetDecl.expose", HANDOFF_CHECK, "980/WEB"),
        row("widget.actions.action", "widget.actions", "action (id, handler, ...) entry", optional("actions blocks"), "ActionsBlock.actions", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.action.attr", "widget.actions.action", "title/icon/shortcut/enabled_if/checked_if", optional("actions blocks"), "ActionEntry attrs", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.menubar", "widget.actions", "menubar { ... } sub-block", optional("actions blocks"), "ActionsBlock.menubar", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.menu", "widget.actions.menubar", "menu (id, title) { ... }", optional("actions blocks"), "MenubarBlock.menus", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.menu-item", "widget.actions.menu", "item (action: ...) | sep", optional("actions blocks"), "MenuEntry.items", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.toolbar", "widget.actions", "toolbar { ... } sub-block", optional("actions blocks"), "ActionsBlock.toolbar", HANDOFF_CHECK, "980/PARITY"),
        row("widget.actions.toolbar-item", "widget.actions.toolbar", "item (action: ...) | sep", optional("actions blocks"), "ToolbarBlock.items", HANDOFF_CHECK, "980/PARITY"),
        row("widget.timer", "widget", "timer block", optional("timer widgets"), "WidgetDecl.timer", HANDOFF_CHECK, "980/STATE"),
        row("widget.timer.entry", "widget.timer", "Msg (every_ms, [when])", optional("timer widgets"), "TimerBlock.entries", HANDOFF_CHECK, "980/STATE"),
        row("widget.timer.entry.when", "widget.timer.entry", "when: condition", optional("timer widgets"), "TimerEntry.when", HANDOFF_CHECK, "980/STATE"),
        row("store.name", "module.store", "ident", optional("store modules"), "StoreDecl.name", HANDOFF_CHECK, "980/STATE"),
        row("store.msg", "store", "msg block", optional("store modules"), "StoreDecl.messages", HANDOFF_CHECK, "980/STATE"),
        row("store.msg.variant", "store.msg", "Variant [payload]", optional("store modules"), "MsgDecl.variants", HANDOFF_CHECK, "980/STATE"),
        row("store.model", "store", "model block", optional("store modules"), "StoreDecl.model", HANDOFF_CHECK, "980/STATE"),
        row("store.model.field", "store.model", "name ty = init", optional("store modules"), "ModelBlock.fields", HANDOFF_CHECK, "980/STATE"),
        row("store.model.field.var", "store.model.field", "var mutability keyword", optional("store modules"), "ModelField.mutable", HANDOFF_CHECK, "980/STATE"),
        row("store.model.field.primary", "store.model.field", "primary-property flag", optional("store modules"), "ModelField.is_primary", HANDOFF_CHECK, "980/STATE"),
        row("store.model.field.decorator", "store.model.field", "#[Provide]/#[Consume]", optional("store modules"), "ModelField.decorators", HANDOFF_CHECK, "980/STATE"),
        row("store.computed", "store", "computed block", optional("store modules"), "StoreDecl.computed", HANDOFF_CHECK, "980/STATE"),
        row("store.computed.property", "store.computed", "name => expr", optional("store modules"), "ComputedBlock.properties", HANDOFF_CHECK, "980/STATE"),
        row("store.on", "store", "on block", optional("store modules"), "StoreDecl.on", HANDOFF_CHECK, "980/STATE"),
        row("store.on.handler", "store.on", "pattern => body", optional("store modules"), "OnBlock.handlers", HANDOFF_CHECK, "980/STATE"),
        row("store.on.handler.stream", "store.on.handler", "on stream subscription", optional("store modules"), "OnHandler.stream", HANDOFF_CHECK, "980/WEB"),
        row("store.watch.decl", "store", "watch decl", optional("store modules"), "StoreDecl.watch", HANDOFF_CHECK, "980/WEB"),
        row("store.watch.source", "store.watch.decl", "watched field", optional("store modules"), "WatchDecl.sources", HANDOFF_CHECK, "980/WEB"),
        row("store.watch.immediate", "store.watch.decl", ".immediate", optional("store modules"), "WatchDecl.immediate", HANDOFF_CHECK, "980/WEB"),
        row("store.watch.deep", "store.watch.decl", ".deep", optional("store modules"), "WatchDecl.deep", HANDOFF_CHECK, "980/WEB"),
        row("store.timer", "store", "timer block", optional("store modules"), "StoreDecl.timer", HANDOFF_CHECK, "980/STATE"),
        row("store.timer.entry", "store.timer", "Msg (every_ms, [when])", optional("store modules"), "TimerBlock.entries", HANDOFF_CHECK, "980/STATE"),
        row("store.timer.entry.when", "store.timer.entry", "when: condition", optional("store modules"), "TimerEntry.when", HANDOFF_CHECK, "980/STATE"),
        row("view.node.element", "view", "tag { props events children }", required("002-counter"), "ViewNode::Element", HANDOFF_CHECK, "980/DIALECT"),
        row("view.prop", "view.node.element", "name: expr", required("002-counter"), "ViewProp", HANDOFF_CHECK, "980/DIALECT"),
        row("view.prop.style-binding", "view.prop", "{ style: cond, ... }", optional("style bindings"), "ViewPropValue::StyleBinding", HANDOFF_CHECK, "980/WEB"),
        row("view.event", "view.node.element", "onclick: .Msg | handler", required("002-counter"), "ViewEvent", HANDOFF_CHECK, "980/STATE"),
        row("view.event.inline", "view.event",
        "() => { ... } inline lambda (parse-time minted to __evt handler; surfaces via widget.msg.variant/widget.on.handler)",
        optional("inline shorthand"), "ViewEvent.inline -> minted on-handler", HANDOFF_CHECK, "980/STATE"),
        row("view.node.text", "view", "literal/interpolated text node", optional("shorthand text"), "ViewNode::Text", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.text.interp", "view.node.text", "${...} interpolation", optional("interpolated text"), "ViewText::Interpolated", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.text.binding", "view.node.text.interp", "state ref inside template", optional("interpolated text"), "ViewText bindings", HANDOFF_CHECK, "980/STATE"),
        row("view.node.for", "view", "for x in .list { ... }", optional("list mounting"), "ViewNode::ForLoop", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.for.index", "view.node.for", "for i, x in ...", optional("indexed loops"), "ViewNode::ForLoop.index", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.for.key", "view.node.for", "key: expr clause", optional("keyed lists"), "ViewNode::ForLoop.key_expr", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.if", "view", "if cond { ... }", optional("conditional mounting"), "ViewNode::Conditional", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.if.else", "view.node.if", "else { ... }", optional("conditional mounting"), "ViewNode::Conditional.else_body", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.component", "view", "Component { props events }", optional("component use"), "ViewNode::Component", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.outlet", "view", "outlet", conditional("router widgets", "outlet renders matched child routes (Plan 105)"), "ViewNode::Outlet", HANDOFF_CHECK, "980/WEB"),
        row("view.node.outlet.memo", "view.node.outlet", "(memo: bool)", optional("outlet memo"), "ViewNode::Outlet.memo", HANDOFF_CHECK, "980/WEB"),
        row("view.node.memo", "view", "memo (deps) { ... }", optional("memo blocks"), "ViewNode::MemoBlock", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.memo.exact", "view.node.memo", "exact: true", optional("memo blocks"), "ViewNode::MemoBlock.exact", HANDOFF_CHECK, "980/DIALECT"),
        row("view.node.link", "view", "link to/text/href", conditional("router widgets", "links belong to routing surfaces (Plan 105)"), "ViewNode::Link", HANDOFF_CHECK, "980/WEB"),
        row("view-fn.name", "module.view-fn", "ident", optional("view-fn projects"), "ViewFragmentDecl.name", HANDOFF_CHECK, "980/DIALECT"),
        row("view-fn.param", "module.view-fn", "name: type", optional("view-fn projects"), "ViewFragmentDecl.params", HANDOFF_CHECK, "980/DIALECT"),
        row("style-recipe.name", "module.style-recipe", "ident", optional("style-recipe projects"), "StyleRecipeDecl.name", HANDOFF_CHECK, "980/WEB"),
        row("style-recipe.param", "module.style-recipe", "name: type = default", optional("style-recipe projects"), "StyleRecipeDecl.params", HANDOFF_CHECK, "980/WEB"),
        row("style-recipe.body", "module.style-recipe", "class-string expr", optional("style-recipe projects"), "StyleRecipeDecl.body", HANDOFF_CHECK, "980/WEB"),
        row("style-recipe.pub", "module.style-recipe", "pub visibility", optional("style-recipe projects"), "StyleRecipeDecl.is_pub", HANDOFF_CHECK, "980/WEB"),
        row("style-recipe.doc", "module.style-recipe", "doc comment", optional("style-recipe projects"), "StyleRecipeDecl.doc", HANDOFF_CHECK, "980/WEB"),
        row("theme-data.name", "module.theme-data", "ident", optional("theme packages"), "ThemeDataDecl.name", HANDOFF_CHECK, "980/PARITY"),
        row("theme-data.pub", "module.theme-data", "pub visibility", optional("theme packages"), "ThemeDataDecl.is_pub", HANDOFF_CHECK, "980/PARITY"),
        row("theme-data.tree", "module.theme-data", "object tree / extend(...)", optional("theme packages"), "ThemeDataDecl.tree", HANDOFF_CHECK, "980/PARITY"),
        row("theme-data.doc", "module.theme-data", "doc comment", optional("theme packages"), "ThemeDataDecl.doc", HANDOFF_CHECK, "980/PARITY"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UiCode;

    /// manifest 自检 + 与映射实现双向对账（删行/缺行都红）。
    #[test]
    fn p980_manifest_is_valid_and_reconciled() {
        let rows = widget_field_manifest();
        assert!(!rows.is_empty());
        let mut seen: Vec<&str> = Vec::new();
        for r in &rows {
            assert!(!seen.contains(&r.id), "duplicate row {}", r.id);
            seen.push(r.id);
            assert!(!r.origin.is_empty(), "{} origin", r.id);
            assert!(!r.parent_role.is_empty(), "{} parent_role", r.id);
            assert!(!r.type_rule.is_empty(), "{} type_rule", r.id);
            assert!(!r.projection_path.is_empty(), "{} projection_path", r.id);
            assert!(!r.check_id.is_empty(), "{} check_id", r.id);
            assert!(!r.consumer_owner.is_empty(), "{} consumer_owner", r.id);
            if let WidgetFieldApplicability::Conditional { mode, reason } = &r.applicability {
                assert!(
                    !mode.is_empty() && !reason.is_empty(),
                    "{} conditional reason",
                    r.id
                );
            }
            match &r.applicability {
                WidgetFieldApplicability::Required { mode }
                | WidgetFieldApplicability::Optional { mode }
                | WidgetFieldApplicability::Conditional { mode, .. } => {
                    assert!(!mode.is_empty(), "{} mode", r.id);
                }
            }
        }
        reconcile_field_manifest(&rows, P980_CONSTRUCTS)
            .expect("manifest and mapping implementation reconcile bidirectionally");
        // 反控（删清单行）：对账红、指名缺失构造。
        let mut without = rows.clone();
        without.retain(|r| r.id != "widget.design.entry");
        let problems = reconcile_field_manifest(&without, P980_CONSTRUCTS)
            .expect_err("deleting a manifest row must be caught");
        assert!(
            problems.iter().any(|p| p.contains("widget.design.entry")),
            "{problems:?}"
        );
    }

    fn sample_ui() -> UiCode {
        // 直接构造 AST（lib 面不依赖 parse_ui；覆盖 widget/msg/model/view/on
        // + store + view-fn + actions + timer 等代表构造）。
        let msg = MsgDecl {
            variants: vec![MsgVariant {
                name: "Inc".into(),
                quoted: false,
                payload: Vec::new(),
                payload_names: Vec::new(),
            }],
        };
        let model = ModelBlock {
            fields: vec![ModelField {
                name: "count".into(),
                ty: auto_ast::ast::Type::Int,
                init: auto_ast::ast::Expr::Int(0),
                mutable: true,
                is_primary: false,
                decorators: Vec::new(),
            }],
        };
        let view = ViewBlock {
            root: ViewNode::Element {
                tag: "col".into(),
                props: vec![ViewProp {
                    name: "style".into(),
                    value: ViewPropValue::Expr(auto_ast::ast::Expr::Str("p-2".into())),
                }],
                events: Vec::new(),
                children: vec![ViewNode::Text(ViewText::Literal("hi".into()))],
                span: Some((0, 20)),
            },
        };
        let on = OnBlock {
            handlers: vec![OnHandler {
                pattern: ".Inc".into(),
                params: Vec::new(),
                body: auto_ast::ast::Body {
                    stmts: Vec::new(),
                    has_new_line: false,
                    source_lines: Vec::new(),
                },
                stream: None,
            }],
        };
        let widget = WidgetDecl {
            name: "App".into(),
            messages: vec![msg],
            model: Some(model),
            computed: None,
            view: Some(view),
            named_views: Vec::new(),
            on: Some(on),
            bind: None,
            props: Vec::new(),
            routes: None,
            lifecycle: Vec::new(),
            design: None,
            style: None,
            ext_imports: Vec::new(),
            watch: Vec::new(),
            setup: None,
            expose: Vec::new(),
            actions: None,
            timer: None,
        };
        let store = StoreDecl {
            name: "Shared".into(),
            messages: Vec::new(),
            model: None,
            computed: None,
            on: None,
            watch: Vec::new(),
            timer: None,
        };
        UiCode::from_domain(vec![UiStmt::WidgetDecl(widget), UiStmt::StoreDecl(store)])
    }

    #[test]
    fn p980_walk_maps_every_construct_with_spans_where_available() {
        let ui = sample_ui();
        let occ = widget_field_occurrences(&ui).expect("every construct maps");
        let ids: Vec<&str> = occ.iter().map(|o| o.id).collect();
        for expected in [
            "module.widget",
            "widget.head",
            "widget.name",
            "widget.msg",
            "widget.msg.variant",
            "widget.model",
            "widget.model.field",
            "widget.model.field.var",
            "widget.view",
            "view.node.element",
            "view.prop",
            "view.node.text",
            "widget.on",
            "widget.on.handler",
            "module.store",
            "store.name",
        ] {
            assert!(
                ids.contains(&expected),
                "missing occurrence {expected}: {ids:?}"
            );
        }
        // view 树的真实 span 随出现携带；其余槽位 AST 无 span（None）。
        let element = occ.iter().find(|o| o.id == "view.node.element").unwrap();
        assert_eq!(element.span, Some((0, 20)));
        assert!(occ
            .iter()
            .find(|o| o.id == "widget.msg")
            .unwrap()
            .span
            .is_none());
    }

    /// 删一项映射实现 → 具名拒绝（ui.field-unknown）；恢复 → 绿。
    #[test]
    fn p980_walk_rejects_when_a_mapping_entry_is_deleted() {
        let ui = sample_ui();
        let reduced: &[&'static str] = &["module.widget", "widget.head", "widget.name"];
        let err = widget_field_occurrences_with(&ui, reduced)
            .expect_err("an unmapped construct must be rejected, not dropped");
        assert_eq!(err.code, "ui.field-unknown");
        assert_eq!(err.construct, "widget.msg");
        // 恢复（默认映射）后同一入口绿。
        widget_field_occurrences(&ui).expect("restored mapping is green");
    }
}
