//! PLAN-980 ④-A 首片 #2c：**包片段 lane**——AGE 包闭包的 SFC `<script>` 段发射。
//!
//! 与组件面 `ComponentFragmentLane` 同构，但输入是**包单元集合 + 入口**：包前门按
//! 场景（实例 1 + 待定位 handler 的 msg 重放）编译出状态函数（`__age_state_*`），
//! lane 在状态函数体里按**原源 span** 定位 handler 表达式，经 976 `a2x-ts` 片段
//! 发射器产出 TS 文本。跨模块 helper 的调用名由同一 `SemanticView` 决定；本模块
//! 另提供 helper 函数体发射，供 SFC `<script>` 把**被调方**一并落进产物。
//!
//! 边界（具名）：本片只发射"handler 体 + 被引用的包内 helper 函数体"，不产出
//! 完整 SFC（template/样式归既有方言道）；只支持 int/str/bool 标量体（与 976
//! fragment 域的既有拒绝面一致）。
use ace_a2x_core::SemanticView;
use ace_core::source::packages::PackageModuleInput;

/// 包片段 lane：受检模块 + 脱糖源（span 基准）。
pub struct PackageFragmentLane {
    ds_src: String,
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
}

impl PackageFragmentLane {
    /// 编译包闭包（实例 1、ops=待定位 handler 的 msg 重放）并持有 checked。
    pub fn compile(
        units: &[PackageModuleInput],
        entry: &str,
        handler_msgs: &[&str],
    ) -> Result<Self, String> {
        let entry_src = units
            .iter()
            .find(|u| format!("{}::{}", u.package, u.id) == entry)
            .map(|u| u.source.clone())
            .ok_or_else(|| format!("entry module `{entry}` is not in the package units"))?;
        let ds_src = ace_core::source::fstr::desugar_fstrings_widget(&entry_src)
            .map_err(|e| format!("fstr desugar: {e}"))?;
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
        Ok(Self {
            ds_src,
            checked: compiled.checked,
        })
    }

    /// 脱糖源长度（span 上界校验用）。
    pub fn source_len(&self) -> usize {
        self.ds_src.len()
    }

    /// 受检模块（view 构建用）。
    pub fn checked_module(&self) -> &ace_core::verify::CheckedModule {
        &self.checked
    }

    /// 受检视图里的函数源名（调用方定位状态函数/helper 用）。
    pub fn function_names(&self) -> Result<Vec<String>, String> {
        let view = SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        Ok(view
            .functions()
            .iter()
            .map(|f| f.source_name.to_string())
            .collect())
    }

    /// 源里第一次出现 `needle` 的 span（测试/调用方定位 handler 表达式用）。
    pub fn find_span(&self, needle: &str) -> Result<std::ops::Range<usize>, String> {
        self.ds_src
            .find(needle)
            .map(|start| start..start + needle.len())
            .ok_or_else(|| format!("`{needle}` not found in the desugared source"))
    }

    /// 在指定函数的体内、按**精确 span** 发射表达式（同 span 取索引最大者）。
    pub fn emit_expr_at_span(
        &self,
        fn_suffix: &str,
        span: std::ops::Range<usize>,
    ) -> Result<PackageFragment, String> {
        let view = SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name.ends_with(fn_suffix))
            .ok_or_else(|| format!("no function ending with `{fn_suffix}` in the package view"))?;
        let mut best: Option<(usize, ace_core::hir::ExprId)> = None;
        for (i, e) in facts.body.exprs.iter().enumerate() {
            if e.span.start == span.start && e.span.end == span.end {
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
        let view = SemanticView::from_checked(&self.checked)
            .map_err(|e| format!("semantic view: {e:?}"))?;
        let facts = view
            .functions()
            .iter()
            .find(|f| f.source_name.ends_with(fn_suffix))
            .ok_or_else(|| format!("no function ending with `{fn_suffix}` in the package view"))?;
        let ctx = ace_a2x_ts::FragmentProgramContext::from_view(&view);
        let body = facts
            .body
            .blocks
            .first()
            .ok_or_else(|| format!("function `{}` has no body block", facts.source_name))?;
        let mut out = Vec::new();
        let mut locals = Vec::new();
        for stmt in 0..body.stmts.len() {
            let raw = ace_a2x_ts::emit_stmt_fragment(
                ace_a2x_ts::ScriptLang::TypeScript,
                &view,
                facts,
                0,
                stmt,
                &ctx,
            )
            .map_err(|e| format!("stmt fragment ({stmt}): {e:?}"))?;
            out.extend(raw.text);
            if locals.is_empty() {
                locals = raw.locals;
            }
        }
        Ok(PackageFunctionBody { text: out, locals })
    }
}
