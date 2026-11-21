//! PLAN-980 ④-A 首片 #2c：**包片段 lane**——AGE 包闭包的 SFC `<script>` 段发射。
//!
//! 与组件面 `ComponentFragmentLane` 同构，但输入是**包单元集合 + 入口**：包前门按
//! 场景（实例 1 + 待定位 handler 的 msg 重放）编译出状态函数（`__age_state_*`），
//! lane 在状态函数体里按**原源 span** 定位 handler 表达式，经 976 `a2x-ts` 片段
//! 发射器产出 TS 文本。跨模块 helper 的调用名由同一 `SemanticView` 决定；本模块
//! 另提供 helper 函数体发射，供 SFC `<script>` 把**被调方**一并落进产物。
//!
//! Handler expressions use A2TS fragments. Package helpers use its whole-function
//! lowerer so their local declarations and cleanup share one scope.
use ace_a2x_core::{SemanticView, Ty};
use ace_core::source::packages::PackageModuleInput;
use std::collections::BTreeSet;

/// 包片段 lane：受检模块 + 脱糖源（span 基准）。
pub struct PackageFragmentLane {
    entry_src: String,
    entry_offset: usize,
    checked: ace_core::verify::CheckedModule,
}

/// 一处 handler 表达式的发射结果。
#[derive(Debug, Clone)]
pub struct PackageFragment {
    /// 逐行文本（976 fragment 的原始行序）。
    pub text: Vec<String>,
    pub expr: String,
    pub helpers: Vec<String>,
    /// 该 body 上下文的局部/参数名（发射侧命名权威）。
    pub locals: Vec<ace_a2x_ts::FragmentLocal>,
}

/// 函数体发射结果（helper 定义落产物用）。
pub struct PackageFunctionBody {
    pub text: Vec<String>,
    pub locals: Vec<ace_a2x_ts::FragmentLocal>,
    pub helpers: Vec<String>,
}

pub(super) struct CheckedHandlerBody {
    pub params: Vec<String>,
    pub lines: Vec<String>,
    pub helpers: BTreeSet<String>,
}

impl PackageFragmentLane {
    /// 编译包闭包（实例 1、ops=待定位 handler 的 msg 重放）并持有 checked。
    pub fn compile(
        units: &[PackageModuleInput],
        entry: &str,
        handler_msgs: &[&str],
    ) -> Result<Self, String> {
        // PLAN-105 T-02：单元统一 f-string 脱糖（组件面/包桥同一入口）。
        let units = &crate::ui::ace_package_units::desugar_units(units.to_vec())?[..];
        let entry_src = units
            .iter()
            .find(|u| format!("{}::{}", u.package, u.id) == entry)
            .map(|u| u.source.clone())
            .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
        let caps = [ace_core::source::packages::PACKAGES_CAPABILITY.to_string()];
        let ops = handler_msgs
            .iter()
            .map(|m| ace_av::age_component::ScenarioOp::Send {
                instance: 0,
                msg: (*m).to_string(),
            })
            .collect();
        let scenario = ace_av::age_component::ComponentScenario { instances: 1, ops };
        let compiled = ace_core::source::age_package::compile_age_package_with_scenario(
            units,
            entry,
            ace_av::age_component::LegacyWidgetAlias::Accept,
            &caps,
            &scenario,
        )
        .map_err(|ds| {
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let entry_offset = compiled
            .description
            .modules
            .iter()
            .find(|module| module.key == entry)
            .ok_or_else(|| format!("compiled package has no entry module {entry:?}"))?
            .flat_start;
        Ok(Self {
            entry_src,
            entry_offset,
            checked: compiled.checked,
        })
    }

    /// Authored source length used to validate spans.
    pub fn source_len(&self) -> usize {
        self.entry_src.len()
    }

    /// 受检模块（view 构建用）。
    pub fn checked_module(&self) -> &ace_core::verify::CheckedModule {
        &self.checked
    }

    /// 受检视图里的函数源名（调用方定位状态函数/helper 用）。
    pub fn function_names(&self) -> Result<Vec<String>, String> {
        let view = SemanticView::from_checked_age_package(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        Ok(view
            .functions()
            .iter()
            .map(|f| f.source_name.to_string())
            .collect())
    }

    /// 源里第一次出现 `needle` 的 span（测试/调用方定位 handler 表达式用）。
    pub fn find_span(&self, needle: &str) -> Result<std::ops::Range<usize>, String> {
        self.entry_src
            .find(needle)
            .map(|start| start..start + needle.len())
            .ok_or_else(|| format!("`{needle}` not found in the entry source"))
    }

    /// 在指定函数的体内、按**精确 span** 发射表达式（同 span 取索引最大者）。
    pub fn emit_expr_at_span(
        &self,
        fn_suffix: &str,
        span: std::ops::Range<usize>,
    ) -> Result<PackageFragment, String> {
        let view = SemanticView::from_checked_age_package(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name.ends_with(fn_suffix))
            .ok_or_else(|| format!("no function ending with `{fn_suffix}` in the package view"))?;
        let compiled_span = span.start + self.entry_offset..span.end + self.entry_offset;
        let mut best: Option<(usize, ace_core::hir::ExprId)> = None;
        for (i, e) in facts.body.exprs.iter().enumerate() {
            if e.span.start == compiled_span.start && e.span.end == compiled_span.end {
                best = Some((i, ace_core::hir::ExprId(i as u32)));
            }
        }
        let (_, expr_id) = best.ok_or_else(|| {
            format!(
                "no HIR expression with span [{}..{}) in `{}`",
                span.start, span.end, facts.source_name
            )
        })?;
        let ctx = ace_a2x_ts::FragmentProgramContext::from_view(&view);
        let raw = ace_a2x_ts::emit_expr_fragment(
            ace_a2x_ts::ScriptLang::TypeScript,
            &view,
            facts,
            expr_id,
            &ctx,
        )
        .map_err(|e| format!("expr fragment: {e:?}"))?;
        Ok(PackageFragment {
            text: raw.text,
            expr: raw.expr.ok_or_else(|| "expression fragment returned no value".to_string())?,
            helpers: raw.helpers,
            locals: raw.locals,
        })
    }

    /// 发射一个函数的**全部语句**（helper 定义落产物用；返回逐语句文本）。
    pub fn emit_function_body(&self, fn_suffix: &str) -> Result<PackageFunctionBody, String> {
        let view = SemanticView::from_checked_age_package(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == fn_suffix)
            .or_else(|| {
                view.functions()
                    .iter()
                    .find(|f| f.source_name.ends_with(fn_suffix))
            })
            .ok_or_else(|| format!("no function ending with `{fn_suffix}` in the package view"))?;
        let ctx = ace_a2x_ts::FragmentProgramContext::from_view(&view);
        let body = ace_a2x_ts::emit_function_body_fragment(
            ace_a2x_ts::ScriptLang::JavaScript,
            &view,
            facts,
            &ctx,
        )
        .map_err(|err| format!("function body: {err:?}"))?;
        Ok(PackageFunctionBody {
            text: body.text,
            locals: body.locals,
            helpers: body.helpers,
        })
    }

    /// The check function has model seeds, the handler statements, and a model
    /// probe return. Emit the middle section and bind its locals to live refs.
    pub(super) fn emit_checked_handler_body(
        &self,
        symbol: &str,
        fields: &[(String, String)],
    ) -> Result<CheckedHandlerBody, String> {
        use ace_core::hir::Stmt;

        let view = SemanticView::from_checked(&self.checked)
            .map_err(|err| format!("semantic view: {err:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name == symbol)
            .ok_or_else(|| format!("checked handler `{symbol}` is missing"))?;
        let entry = facts.body.entry.0;
        let stmts = &facts.body.blocks[entry as usize].stmts;
        let (Stmt::Return { .. }, before_return) = stmts
            .split_last()
            .ok_or_else(|| "checked handler has no result".to_owned())?
        else {
            return Err("checked handler must end in a model probe".into());
        };
        let (seeds, _) = before_return
            .split_at_checked(fields.len())
            .ok_or_else(|| "checked handler is missing model bindings".to_owned())?;

        let ctx = ace_a2x_ts::FragmentProgramContext::from_view(&view);
        let emit = |i| {
            ace_a2x_ts::emit_stmt_fragment(
                ace_a2x_ts::ScriptLang::JavaScript, &view, facts, entry, i, &ctx,
            )
            .map_err(|err| format!("handler statement {i}: {err:?}"))
        };
        let locals = emit(stmts.len() - 1)?.locals;
        if locals.iter().any(|local| {
            local.role == "capture" || !matches!(local.ty, Ty::I32 | Ty::Bool | Ty::F32)
        }) {
            return Err("checked handlers require copyable int, bool or f32 bindings".into());
        }
        let params = locals
            .iter()
            .filter(|local| local.role == "param")
            .map(|local| local.emitted_name.clone())
            .collect();
        let mut lines = Vec::new();
        let mut state = Vec::new();
        for (seed, (field, _)) in seeds.iter().zip(fields) {
            let Stmt::Let { local, .. } = seed else {
                return Err(format!("expected a model binding for `{field}`"));
            };
            let binding = locals
                .iter()
                .find(|binding| binding.index == local.0 && binding.source_name == format!("__age_s_0_{field}"))
                .ok_or_else(|| format!("checked model binding does not match `{field}`"))?;
            let name = &binding.emitted_name;
            lines.push(format!("let {name} = {field}.value;"));
            state.push((binding.index, format!("{field}.value = {name};")));
        }
        for binding in locals.iter().filter(|local| local.role == "binding") {
            if !state.iter().any(|(index, _)| *index == binding.index) {
                lines.push(format!("let {};", binding.emitted_name));
            }
        }
        let mut helpers = BTreeSet::new();
        for i in fields.len()..stmts.len() - 1 {
            let fragment = emit(i)?;
            helpers.extend(fragment.helpers);
            lines.push("{".into());
            lines.extend(fragment.text);
            lines.push("}".into());
        }
        lines.extend(state.into_iter().map(|(_, write)| write));
        Ok(CheckedHandlerBody {
            params,
            lines,
            helpers,
        })
    }
}
