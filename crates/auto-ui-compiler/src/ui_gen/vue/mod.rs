//! Vue3/JavaScript Code Generator
//!
//! Generates Vue 3 Single File Components (SFC) from AURA widgets.
//! Supports two output modes:
//!
//! 1. **Plain Tailwind** - Native HTML elements with Tailwind CSS classes
//! 2. **shadcn-vue** - Pre-built accessible components from shadcn-vue
//!
//! ## Output Format (shadcn-vue mode)
//!
//! ```vue
//! <script setup>
//! import { ref } from 'vue'
//! import { Button } from '@/components/ui/button'
//! import { Input } from '@/components/ui/input'
//!
//! const count = ref(0)
//!
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col gap-2">
//!     <Button @click="handleInc">Increment</Button>
//!     <Input v-model="count" />
//!   </div>
//! </template>
//! ```
//!
//! ## Output Format (Plain Tailwind mode)
//!
//! ```vue
//! <script setup>
//! import { ref, computed } from 'vue'
//!
//! // State variables → ref()
//! const count = ref(0)
//!
//! // Event handlers
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col">
//!     <button @click="handleInc">+</button>
//!     <h2>Count: {{ count }}</h2>
//!   </div>
//! </template>
//!
//! <style>
//! /* Component styles */
//! </style>
//! ```
//!
//! ## Supported shadcn-vue Components (Plan 099)
//!
//! ### Content Elements
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `button` | Button | variant, size, disabled, text→slot |
//! | `input` | Input | v-model, type, placeholder, disabled |
//! | `textarea` | Textarea | v-model, placeholder, rows, disabled |
//! | `checkbox` | Checkbox | v-model:checked, disabled |
//! | `toggle`/`switch` | Switch | v-model:checked, disabled |
//! | `select` | Select | v-model, disabled |
//!
//! ### Layout & Navigation (Phase 3)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `scroll` | ScrollArea | class, orientation, hide_delay, size (PLAN-692 thumb px → --sb-size) |
//! | `tabs` | Tabs | v-model, default-value |
//! | `tab` | TabsTrigger | value, disabled, text→slot |
//! | `card` | Card | variant, title→slot |
//! | `divider` | Separator | orientation, decorative, label |
//!
//! ### Overlay & Feedback (Phase 4)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `modal` | Dialog | v-model:open, title, description |
//! | `tooltip` | Tooltip | content→slot, side, delay |
//! | `spinner` | Skeleton | class, width, height |
//! | `progress` | Progress | v-model, max |
//! | `badge` | Badge | variant, text→slot |
//!
//! ### Data Components (Phase 5)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `table` | Table | class |
//! | `thead`/`tbody`/`tr` | TableHeader/TableBody/TableRow | class |
//! | `th`/`td` | TableHead/TableCell | class, colspan, rowspan |
//! | `tree` | Collapsible | class |
//! | `tree_item` | CollapsibleItem | v-model:open, text→slot |
//! | `avatar` | Avatar | src, name→slot |
//!
//! ### Form Components (Phase 6)
//! | AURA Tag | shadcn-vue | Props |
//! |----------|------------|-------|
//! | `slider` | （无——PLAN-661 R-5 退役，原生 `<input type="range">`） | :value, min, max, step, disabled, @input 载荷 |
//! | `radiogroup` | RadioGroup | v-model, name, disabled |
//! | `radio` | RadioGroupItem | value, id, disabled, label→slot |

use super::{BackendGenerator, GenError, GenResult, WidgetRegistry};
/// Plan 435 P4:折叠相等(剥 -/_ + 小写;schema 别名策略同源)。
/// Plan 008 (auto-os-config batch 4): find a `key:` prop on a loop body's
/// children (or one conditional level down) and return its expression text —
/// used to key the fallback v-for wrapper div (container-level keys are fine
/// on vue but kill the vm subtree, so keys live on branch-first texts).
/// Dot 链扁平化:a.b.c(根须为 Ident;否则 None)。
fn dot_chain_path(expr: &auto_lang_core::ast::Expr) -> Option<String> {
    match expr {
        auto_lang_core::ast::Expr::Ident(n) => Some(n.to_string()),
        auto_lang_core::ast::Expr::Dot(inner, name) => {
            Some(format!("{}.{}", dot_chain_path(inner)?, name))
        }
        _ => None,
    }
}

/// PLAN-639 T-04: quote-aware scan for a tag's closing `>`.
/// `String::find('>')` misfires when an attribute VALUE contains `>`
/// (e.g. the auto search-filter's `(n: any) =>` lambda inside v-for) and
/// yields a position inside the attribute — inserting `:key` mid-value and
/// corrupting the emitted tag (046-bp-import NoteList v-for 实测).
fn find_tag_close(html: &str) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut quote: Option<u8> = None;
    for (i, b) in bytes.iter().enumerate() {
        match quote {
            Some(q) if *b == q => quote = None,
            Some(_) => {}
            None => {
                if *b == b'"' || *b == b'\'' {
                    quote = Some(*b);
                } else if *b == b'>' {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn find_loop_child_key(body: &[AuraNode]) -> Option<String> {
    // depth-bounded recursive search: the key may sit on a text nested a few
    // containers inside a conditional branch
    fn search(node: &AuraNode, depth: usize) -> Option<String> {
        if depth > 6 {
            return None;
        }
        match node {
            AuraNode::Element {
                props, children, ..
            } => {
                for (k, v) in props {
                    if k == "key" {
                        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(n)) = v {
                            return Some(n.to_string());
                        }
                        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(sv)) = v {
                            return Some(sv.to_string());
                        }
                        // key: e.key — Dot field access (Plan 008)
                        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Dot(inner, name)) = v
                        {
                            if let auto_lang_core::ast::Expr::Ident(base) = inner.as_ref() {
                                return Some(format!("{}.{}", base, name));
                            }
                            // Plan 041a(musk 041 Phase 5 R006 收口): 嵌套 Dot
                            // 链(key: be.sib.id)扁平化为点路径——分支内多级
                            // 字段 key 此前提取失败,v-for 包装缺 :key 触发
                            // R006(ChatsView 分叉切换器实测)。
                            if let Some(base_path) = dot_chain_path(inner) {
                                return Some(format!("{}.{}", base_path, name));
                            }
                        }
                    }
                }
                for c in children {
                    if let Some(k) = search(c, depth + 1) {
                        return Some(k);
                    }
                }
                None
            }
            AuraNode::Conditional {
                then_body,
                else_body,
                ..
            } => {
                for sub in then_body.iter().chain(else_body.iter().flatten()) {
                    if let Some(k) = search(sub, depth + 1) {
                        return Some(k);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn key_of(node: &AuraNode) -> Option<String> {
        match node {
            AuraNode::Element { props, .. } => {
                for (k, v) in props {
                    if k == "key" {
                        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(n)) = v {
                            return Some(n.to_string());
                        }
                        if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(sv)) = v {
                            return Some(sv.to_string());
                        }
                    }
                }
                None
            }
            AuraNode::Component { props, .. } => {
                for (k, v) in props {
                    if k == "key" {
                        if let auto_lang_core::ast::Expr::Ident(n) = v {
                            return Some(n.to_string());
                        }
                        if let auto_lang_core::ast::Expr::Str(sv) = v {
                            return Some(sv.to_string());
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
    for child in body {
        if let Some(k) = search(child, 0) {
            return Some(k);
        }
    }
    None
}

fn fold_eq(a: &str, b: &str) -> bool {
    let f = |s: &str| {
        s.chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_lowercase()
    };
    f(a) == f(b)
}

/// Plan 548 T7: view 树折叠匹配(kebab/underscore/Pascal 同键)查找 tag,
/// 递归 Element/ForLoop/Conditional/Component/Link 的全部子节点。
fn view_tree_has_tag(node: &AuraNode, tag: &str) -> bool {
    match node {
        AuraNode::Element {
            tag: t, children, ..
        } => fold_eq(t, tag) || children.iter().any(|c| view_tree_has_tag(c, tag)),
        AuraNode::ForLoop { body, .. } => body.iter().any(|c| view_tree_has_tag(c, tag)),
        AuraNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(|c| view_tree_has_tag(c, tag))
                || else_body
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .any(|c| view_tree_has_tag(c, tag))
        }
        AuraNode::Component { children, .. } | AuraNode::Link { children, .. } => {
            children.iter().any(|c| view_tree_has_tag(c, tag))
        }
        _ => false,
    }
}

/// Plan 435 P4:tag 是否为内置(schema 三级折叠解析可命中)。
/// 内置优先(Plan 408 推广):内置 tag 不参与子组件折叠桥接。
/// PLAN-684:package_origin 元素(chart 四裸名等)不算 builtin——它们由
/// official 组件包供给,须走 known_sub_widgets 折叠桥接(否则落 div 空壳)。
/// 与 component_registry::is_builtin_fold/resolve 同口径。
fn tag_is_builtin(tag: &str) -> bool {
    crate::aura::default_schema_cached()
        .map(|s| {
            s.resolve_tag(tag)
                .map(|(canonical, _)| {
                    s.meta.get(canonical).map(|m| m.tier)
                        != Some(crate::aura::schema::ElementTier::PackageOrigin)
                })
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Plan 435 P4:折叠桥接的命名空间形状守卫 —— 裸小写单词(pre/code/div)
/// 属元素命名空间,即使折叠命中组件名也不桥接(否则 `pre {}` 会变成
/// 自引用 `<Pre/>`);kebab(copy-button)或含大写(TabTrigger)才是组件形态。
fn tag_has_component_shape(tag: &str) -> bool {
    tag.contains('-') || tag.contains('_') || tag.chars().any(|c| c.is_uppercase())
}

/// Plan 435 P3:HashMap/HashSet 迭代序跨进程不确定(RandomState),SFC 发射必须确定。
/// 约定:各发射点收集后自行内联 `.sort()`(lucide_icons / api_functions_used /
/// ext_import_lines / custom_imports / generate_shadcn_imports 均如此);
/// 不设公共排序助手,避免"绕过助手 = 静默不确定"的假统一。
use crate::aura::{
    AuraEvent, AuraNode, AuraProp, AuraPropValue, AuraStyleBinding, AuraTextContent, AuraWidget,
    LogicPayload,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Vue Generator
// ============================================================================

/// Generation mode for Vue output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VueMode {
    /// Plain HTML with Tailwind CSS classes
    #[default]
    Plain,
    /// shadcn-vue components with accessibility built-in
    Shadcn,
    /// Self-contained library widgets (Plan 331): each primitive emits an
    /// independent SFC importing `reka-ui` directly, never `@/components/ui/*`.
    Library,
}

/// PLAN-493: 从发射 attr 串中提取 `class="…"` 的值（无 class 属性 → None）。
fn extract_class_attr(attr_str: &str) -> Option<String> {
    let idx = attr_str.find("class=\"")?;
    let rest = &attr_str[idx + 7..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// PLAN-493: text-* token 是否为字号（而非颜色）——backdrop 保底着色判断用。
/// text-xs/sm/base/lg/xl…nxl 与 text-[15px]/[1.5rem] 视为字号；text-foreground、
/// text-red-500、text-[hsl(…)]/#/rgb 视为颜色。
fn is_text_size_token(tok: &str) -> bool {
    let Some(s) = tok.strip_prefix("text-") else {
        return false;
    };
    if matches!(
        s,
        "xs" | "sm"
            | "base"
            | "lg"
            | "xl"
            | "2xl"
            | "3xl"
            | "4xl"
            | "5xl"
            | "6xl"
            | "7xl"
            | "8xl"
            | "9xl"
    ) {
        return true;
    }
    s.starts_with('[')
        && (s.contains("px") || s.contains("rem") || s.contains("em"))
        && !s.contains("hsl")
        && !s.contains('#')
        && !s.contains("rgb")
}

/// PLAN-493: textarea 类串 → backdrop 类串推导（几何镜像）。token 级规则：
/// 删输入面专属 token（`text-transparent` 位换 `text-foreground` 保序，
/// 避免同特异性时 CSS 顺序不确定；`caret-*`/`resize-*`/`outline*`/
/// `focus:*`/`disabled:*`/`read-only*`/`selection:*`/`placeholder:*`）；
/// 若删后无任何 text-**颜色** token（仅字号），保底追加 `text-foreground`
/// （backdrop 是可视文字层，透明 textarea 的颜色不继承使命）；增背板
/// 结构 token（`pointer-events-none overflow-hidden whitespace-pre-wrap
/// break-words`，幂等去重）。
fn derive_mention_backdrop_class(ta_class: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for tok in ta_class.split_whitespace() {
        if tok == "text-transparent" {
            out.push("text-foreground".to_string());
        } else if tok.starts_with("caret-")
            || tok.starts_with("resize-")
            || tok.starts_with("outline")
            || tok.starts_with("focus:")
            || tok.starts_with("disabled:")
            || tok.starts_with("read-only")
            || tok.starts_with("selection:")
            || tok.starts_with("placeholder:")
        {
            // 输入面专属，背板不携带。
        } else {
            out.push(tok.to_string());
        }
    }
    let has_text_color = out
        .iter()
        .any(|t| t.starts_with("text-") && !is_text_size_token(t));
    if !has_text_color {
        out.push("text-foreground".to_string());
    }
    for structural in [
        "pointer-events-none",
        "overflow-hidden",
        "whitespace-pre-wrap",
        "break-words",
    ] {
        if !out.iter().any(|t| t == structural) {
            out.push(structural.to_string());
        }
    }
    out.join(" ")
}

/// Vue3 SFC generator
pub struct VueGenerator {
    /// PLAN-646: Select Anything DOM 标记（data-auto-*）开关——dev 运行面
    /// （`auto run` 经 AUTOUI_SELECT_MARKERS=1）开启；产物构建保持输出
    /// 逐字节不变（既有快照测试锚定精确 HTML，恒定注入会大面积破坏）。
    select_markers: bool,
    /// PLAN-646: 本生成器实例对应的 .at 源文件 stem（data-auto-src 注入，
    /// overlay 据此在 AUTO_SOURCES 中选对源文——子件 span 归各自 .at）。
    source_stem: String,
    /// Current widget name
    current_widget: Option<String>,

    /// PLAN-031 F-03：本 widget 的 `design { … }` 条目（theme/mode 等），
    /// 供生成器把 design 表达式投影为组件内 computed（此前只有保留名约定）。
    design_entries: Vec<(String, auto_lang_core::ast::Expr)>,

    /// PLAN-031 T-03：widget 模板根是否已挂 `ref="__autoThemeRoot"`——
    /// 主题/强调色运行时据此把 CSS vars 写到本 App 容器子树（多 App 同
    /// 文档不共享 document 变量）；无根 `<div>` 的模板保持 document legacy。
    theme_root_ref: bool,

    /// Collected imports
    imports: Vec<String>,

    /// State variable names (for ref() detection)
    state_names: Vec<String>,

    /// Prop names (for defineProps — no .value suffix needed)
    prop_names: Vec<String>,

    /// Prop name → TS type (for computed expression type inference)
    prop_types: HashMap<String, String>,

    /// State variable name → TS type (for computed expression type inference)
    state_types: HashMap<String, String>,

    /// Plan 014: state/prop names declared int-typed at the Auto level
    /// (`Int`/`i64`/...; TS collapses int and float to `number`, so this is
    /// tracked separately). Feeds ts_adapter division lowering — `Math.trunc`
    /// only when both operands are proven int.
    int_names: std::collections::HashSet<String>,

    /// Store dependencies from `use store:` (Plan 351)
    store_deps: Vec<String>,

    /// PLAN-010: 项目级 store 暴露面（store 真名 → 暴露方法名集，取自各
    /// store 的 handlers/msg）。多 store widget 的裸 `store.Method()` 依此
    /// 做归属路由；空表 = 不路由（单 store 与旧调用方行为零变化）。
    store_surface: std::collections::HashMap<String, std::collections::HashSet<String>>,

    /// Plan 446 批三 G1: store composable 导入路径前缀。默认 `@/stores`；
    /// 部署管线若把生成物迁至子目录（os-config regen 的 `src/stores/auto/`
    /// 布局），配 `@/stores/auto` 使 vue-tsc 的模块解析与实际落盘位置
    /// 对齐（此前恒 `@/stores` → TS2307，下游被迫 regen 过滤规避）。
    store_import_prefix: String,

    /// Whether the project depends on @autodown/editor (from pac.at npm_deps).
    /// Drives R003 validation + main.ts CSS import.
    uses_autodown: bool,

    /// Warnings from the last validate_sfc run (Plan 361).
    pub last_validation_warnings: Vec<crate::ui_gen::validators::ValidationWarning>,

    /// Plan 012 Batch A: warnings raised DURING generation (dropped/degraded/
    /// passed-through codegen behavior). RefCell because the expression
    /// transpilers (`expr_to_js`, ts_adapter contexts) only hold `&self`.
    /// Drained into `last_validation_warnings` at the end of `generate_sfc`
    /// so every caller surfaces them through the single validation channel.
    codegen_warnings: std::cell::RefCell<Vec<crate::ui_gen::validators::ValidationWarning>>,

    /// Names of the current widget's `computed { ... }` properties. Set at
    /// the start of script generation; used to unwrap computed refs
    /// (`.c` → `c.value`) in script expressions (Plan 012 Batch A, gap 44).
    computed_names: std::collections::HashSet<String>,

    /// Event handler definitions (name, body, is_async)
    handlers: Vec<(String, String, bool)>,

    /// Plan 053 M5/P5-6: handlers whose body calls `complete(...)` (backend
    /// completion). They get wrapped in a setTimeout(80ms) debounce with a
    /// sequence guard so rapid typing fires one request per pause and a stale
    /// (slower) result can't clobber the newest suggestions.
    debounced_handlers: HashSet<String>,

    /// Event names for emit
    emit_events: Vec<String>,

    /// Subset of emit_events that came from QUOTED msg variants
    /// (contractual emit names like "update"/"link-click"): always
    /// declared in defineEmits and always trailing-emitted, even when no
    /// template binding references the handler.
    quoted_events: std::collections::HashSet<String>,

    /// PLAN-037 T3: subset of emit_events that are callback-CONTRACT
    /// channels — the Pascal name of an `on_xxx: msg` prop with a matching
    /// msg variant (prop dropped from defineProps, parent binds `@Pascal`).
    /// Like quoted events they are ALWAYS declared in defineEmits and carry
    /// the variant payload type, even when no template binding fires them
    /// (the body may invoke the callback from a handler instead).
    callback_contract_events: std::collections::HashSet<String>,

    /// Whether emit is needed
    has_emit: bool,

    /// Component references (other widgets)
    component_refs: Vec<String>,

    /// Lucide icon components used (for import collection)
    lucide_icons: HashSet<String>,
    /// PLAN-018：iconfile 双主题位图 <img> 出现过 → generate_style 追加
    /// 浅/深切换规则（宿主根 .dark class；无该机制宿主恒浅表——优雅降级）。
    iconfile_theme_css: bool,

    /// Tailwind classes for wrapper
    wrapper_classes: String,

    /// Generation mode (Plain or Shadcn)
    mode: VueMode,

    /// pac.at `default_classes:` toggle (Plan 014). Default true = inject the
    /// doc-theme default Tailwind classes in `extract_classes`. false = skip
    /// them for everything except the structural layout primitives
    /// (row/col/column/grid/scroll/center/container/square), so pixel-exact
    /// replica projects can bring their own styling without losing layout.
    default_classes: bool,

    /// Unified widget registry (replaces ShadcnRegistry)
    #[allow(dead_code)]
    widget_registry: WidgetRegistry,

    /// Track which shadcn-vue components are used (for import collection)
    shadcn_components_used: HashSet<String>,

    /// Whether to output TypeScript (Plan 100: a2js → a2ts)
    use_typescript: bool,

    /// Counter for unique previewcard IDs
    previewcard_counter: usize,

    /// Data for each previewcard (id, auto_code, vue_code)
    previewcard_data: Vec<PreviewCardData>,

    /// Whether copyCode function is needed
    needs_copy_code: bool,

    /// Counter for unique codeblock IDs
    codeblock_counter: usize,

    /// Data for each codeblock (id, code, lang)
    codeblock_data: Vec<CodeBlockData>,

    /// Whether router is needed (has outlet, link, or nav() calls) - Plan 105
    needs_router: bool,
    /// Whether useRoute is needed (has route.param/query/path access) - Plan 235
    needs_route: bool,

    /// API functions used in handlers (Plan 132)
    api_functions_used: HashSet<String>,

    /// Project-specific API function names loaded from dist/.api_functions
    project_api_functions: Vec<String>,

    /// Handler names actually referenced in the template
    used_handlers: HashSet<String>,

    /// Whether the widget has an isDark state var (dark mode toggle)
    has_dark_mode: bool,
    /// PLAN-007 T-05：模型声明 `theme_name` → 生成主题热切换 watch。
    has_theme_name: bool,
    /// Name of the dark mode state variable (e.g. "isDark" or "dark_mode")
    dark_mode_var: Option<String>,
    /// Plan 458: true when the widget itself declares `dark_mode` in its
    /// state_vars (root-App pattern, e.g. 006-hero-section) — the template
    /// binds the local ref; false means the store convention (`store.dark_mode`,
    /// e.g. 015-notes) or the handler-only fallback.
    dark_mode_on_widget: bool,

    /// Plan 409 §8: whether the widget owns an `accent_color` state var —
    /// triggers widget-path applyAccent() injection (mirrors the store path).
    has_accent_color: bool,

    /// Whether theme-toggle component is used
    use_theme_toggle: bool,
    /// Set when a handler calls toast() so we emit `import { toast } from 'vue-sonner'`.
    needs_toast_import: bool,

    /// Plan 444 (ash-shell-057 ④): a handler body referenced a VM-only
    /// native (`fs.*` / `File.*`) — the SFC must declare the shared
    /// `__vmOnly` throwing stub the ts_adapter rewrites those calls into.
    needs_vm_only_helper: bool,

    /// Names of known sub-widgets in the same project (e.g. "Sidebar", "EditorPanel")
    /// When a tag matches one of these, skip shadcn component mapping and treat as custom component
    known_sub_widgets: HashSet<String>,

    /// PLAN-037 T5: sub-widget name -> its model var names (the bindable
    /// channels). A call-site prop whose name hits a channel and whose value
    /// is a writable state slot compiles to `v-model:name`; any other value
    /// shape is a hard error (model channels require writable slots).
    sub_widget_models: std::collections::HashMap<String, Vec<String>>,

    /// Plan 444 (ash-shell-057 ①b): sub-widget name -> the emit names its
    /// SFC actually fires ([`Self::widget_emit_set`]). Parent-side
    /// `on_x: .Y` callback bindings consult it: prop-derived event when the
    /// child declares it, else the bound handler's msg (the pass-through
    /// idiom) when the child emits THAT.
    sub_widget_msgs: std::collections::HashMap<String, Vec<String>>,

    /// Plan 443: the CURRENT widget's model channels that a parent call-site
    /// actually binds. Set via `with_bound_model_channels` (from the
    /// file-level / workspace-level binding scan) before `generate`.
    bound_model_channels: std::collections::HashSet<String>,

    /// Plan 443: every `(tag, channel)` pair for which THIS generator's
    /// template actually emitted a `v-model:channel` binding. Harvested by
    /// the two-pass driver (`generate_component_from_file`) to decide which
    /// child channels are bound — recording at the emission point itself
    /// keeps the scan byte-for-byte consistent with the codegen decision.
    pub(crate) emitted_model_bindings: Vec<(String, String)>,

    /// Current for-loop variable name (e.g., "note") — used to pass loop var as event arg
    /// When inside a `for note in .notes { ... }`, this is set to Some("note")
    current_loop_var: Option<String>,

    /// Plan 502 M1: 当前节点是否位于 svg 元素子树内(svg/path/circle/…
    /// 任一祖先)。node_to_html 进入 svg 族元素时置位、发射尾出口复原;
    /// 子树内 `text` 据此直通为 SVG `<text>`(子树外仍 DSL text→span)。
    in_svg_subtree: bool,

    /// True when `current_loop_var` is the loop INDEX variable
    /// (`for i, note in .notes` → current_loop_var = "i", an int).
    /// Index vars are primitive ints, so the auto-:key must not emit `i?.id`.
    current_loop_var_is_index: bool,

    /// Per-widget key counter — increments for every component usage in the template.
    /// Produces stable unique keys like 'AutoDownEditor-1', 'NavTree-2', etc. so that
    /// two components with the same name (e.g. in different v-if branches) don't collide.
    widget_key_counter: usize,

    /// PLAN-493: textarea mentions 能力 —— template 发射期置位（至少一个
    /// mentions textarea），script 生成期消费（`__autoMentionHtml` helper
    /// 随组件一次发射）。
    mention_helper_needed: bool,

    /// Handlers that need a loop-id parameter (e.g., "SelectNote" needs `i: any`)
    /// Populated during template generation, consumed during script generation.
    /// Maps handler name → loop variable name (e.g., "SelectNote" → "i").
    loop_param_handlers: HashMap<String, String>,

    /// Current widget's msg variants: handler name → payload arity (Plan 043 H2).
    /// Used to decide whether a sub-widget event binding forwards the child's
    /// EMIT payload (`$event`) or the for-loop variable. When the parent's msg
    /// variant takes a payload (e.g. `.Rerun(str)`), the child emit carries the
    /// value and the binding must pass `$event`, not the loop var.
    msg_payload_arities: HashMap<String, usize>,

    /// Plan 421: current widget's on-block handler params (verbatim pattern →
    /// param names, e.g. ".CursorMoved(line, col)" → ["line", "col"]).
    /// Populated in generate_sfc before template generation so the shadcn
    /// event loop can forward the code_editor scaffold's cursor/contextmenu
    /// payload to handlers that declare parameters.
    handler_params: HashMap<String, Vec<String>>,

    /// Whether to generate handleChildDelete function (auto-wired when sub-widget emits Delete)
    needs_child_delete_handler: bool,

    /// Whether API functions were explicitly imported via `use back.api: ...`
    /// When true, skip AST scanning and use the explicit import list
    explicit_api_imports: bool,

    /// document/window-level event listeners declared in the view via
    /// `.window` / `.document` event modifiers (e.g. `onmousemove.window`).
    /// Populated during template generation, consumed during script generation
    /// to emit addEventListener/removeEventListener pairs.
    global_listeners: Vec<GlobalListener>,

    /// Template refs declared in the view via `ref: "menuEl"`.
    /// Populated during template generation, consumed during script generation
    /// to emit `const menuEl = ref<HTMLElement | null>(null)` declarations.
    template_refs: Vec<String>,
    /// Plan 547: ImageSurface onpan 合成登记(pointer down/move/up 三绑定,
    /// script 臂发包装函数——DOM 无 pan 事件,裸 @pan 既不触发也过不了
    /// vue-tsc)。
    surface_pans: Vec<(String, bool)>,

    /// Subset of `template_refs` attached to child components (not DOM
    /// elements) via `ref: "canvasRef"` on a sub-widget instantiation.
    /// Declared as `ref<any>(null)` since the child's defineExpose surface
    /// is unknown to the parent.
    component_ref_names: HashSet<String>,

    /// Plan 053 后续: `scroll(auto_scroll: "<prop>")` specs accumulated during
    /// template generation. Each entry = (watch source prop, sentinel ref name).
    /// Drained in generate_script to emit
    /// `watch(() => props.<prop>, () => nextTick(() => sentinel.scrollIntoView()))`.
    scroll_auto_scroll: Vec<(String, String)>,
    /// PLAN-656 T-07: 需要注入 scroll controller JS helper 族（snake_case
    /// 同名直通——ts 转译对未知原生是蛇形名原样输出，注入同名函数即接通）。
    pub(crate) scroll_ctl_helpers_needed: bool,

    /// Pending sentinel ref name set by the scroll arm of generate_shadcn_attrs
    /// and consumed (taken) in node_to_html right before child recursion, so the
    /// sentinel <div ref="..."> can be injected as the last child of the
    /// ScrollArea. Option because it's a hand-off between the two methods.
    pending_scroll_sentinel: Option<String>,

    /// External Vue components declared via the widget-level
    /// `use { component: Name from "..." }` block. Keyed by every accepted
    /// view tag form (exact name, snake_case, kebab-case) so `FancyBadge`,
    /// `fancy_badge`, and `fancy-badge` all resolve to the component.
    ext_components: HashMap<String, ExtComponent>,

    /// Import lines from the widget `use { ... }` block, as
    /// (imported symbol names, is_component, line). `is_component` entries
    /// are dropped at emission time when the built-in registry already
    /// imported the same symbol (avoids duplicate bindings).
    ext_import_lines: Vec<(Vec<String>, bool, String)>,

    /// Composables from `use { composable: ... }` to call once at
    /// `<script setup>` top level: (local const name, callee name).
    ext_composables: Vec<(String, String, String)>,
    /// Plan 563: canvas 元素登记(node_to_html 模板臂收集,generate_script
    /// 消费——redraw/pen 包装/全局监听/resize 重绘)。
    canvas_specs: Vec<CanvasSpec>,
    /// PLAN-617 T-07: 受控 `video` 元素登记（模板臂收集、generate_script
    /// 消费——下行 apply 函数 + watchEffect + 上行原生监听包装）。
    video_specs: Vec<VideoSpec>,
    /// PLAN-617 后续: 可拖拽进度条登记（`progress` + `onseek`）。存 handler
    /// 函数名——script 臂据此生成 pointer 三包装（按下/拖动/抬起）。
    progress_seek_specs: Vec<String>,
    /// Plan 408 P12 §10.4: composable ref 字段标注——`use { composable: useX(refs: [a, b]) }`
    /// → key = local name（"x"），value = 标注为 ref 的字段名集合。script 表达式
    /// 访问这些字段时加 `.value`（composable 返回普通对象时 ref 不自动 unwrap）。
    facade_ref_fields: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Plan 426: setup preamble binding names (`let` names). They join the
    /// facade locals — script/handler access injects no `.value` (the binding
    /// itself is not a ref); the template sees them via script-setup top-level.
    setup_locals: Vec<String>,
    /// Plan 426: refs annotations of setup bindings (binding → ref fields),
    /// same mechanism as facade_ref_fields (script-side field access + .value).
    setup_ref_fields: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Plan 426: current widget's setup statements (stashed in generate_sfc
    /// for generate_script — the generator methods receive the widget
    /// piecemeal).
    current_setup_stmts: Option<Vec<auto_lang_core::ast::Stmt>>,
    /// Plan 451 P2: current widget's `actions {}` block (stashed in
    /// generate_sfc). The `menubar {}`/`toolbar {}` placeholder tags in the
    /// view synthesize their component trees from it.
    current_actions: Option<auto_ui_dialect::ast::ActionsBlock>,
    /// Plan 451 P2: normalized shortcut → handler fn name (first declaration
    /// wins, mirroring the vm's shortcut_bindings collision rule). Emitted as
    /// the global keydown fallback layer in generate_script.
    actions_key_bindings: Vec<(String, String)>,
    /// Plan 464 T2: `bind {}` 块 key → handler fn name。vm 侧经
    /// keyboard_subscription 查 key_bindings；vue 侧此前无对应层（bind 静默
    /// 失效），补 `__autoBindKeymap` window keydown 层（emit 紧邻 actions 层）。
    bind_key_bindings: Vec<(String, String)>,
    /// Module-level plain functions attached to the widget file (e.g. `fn eval_expr(...)`).
    pub module_fns: Vec<crate::aura::AuraModuleFn>,
    /// Plan 522: `use` 导入模块的 fn 池 —— 模块内全部模块级 fn(含未被
    /// use items 列出的兄弟 fn;依赖闭包按裸名拉取要用全量池)。由 api.rs
    /// 的 `collect_use_module_fns` 产出(与 vm 侧全模块加载对齐)。
    pub use_module_fns: Vec<crate::aura::AuraModuleFn>,
    /// Plan 522: `use module: a, b` items 显式导入的裸名集合 —— widget
    /// computed/handler 体引用的合法入口门(与 vm import_aliases 同口径;
    /// 池内闭包拉取不受此门限)。
    pub use_imported_names: std::collections::HashSet<String>,
}

/// A Vue component declared in a widget-level `use { component: ... }` block.
#[derive(Debug, Clone)]
struct ExtComponent {
    /// Component name as imported (e.g. "FancyBadge").
    name: String,
}

/// A document/window-level event listener (Plan: generic DOM events).
///
/// Declared in the view with a `.window` or `.document` event modifier:
/// `onmousemove.window: .DragMove($event)`. Instead of a template attribute,
/// the generator emits `target.addEventListener(...)` in onMounted and the
/// matching `removeEventListener(...)` in onUnmounted.
#[derive(Debug, Clone)]
struct GlobalListener {
    /// "window" or "document"
    target: String,
    /// DOM event name (e.g. "mousemove", "wheel")
    event: String,
    /// Listener expression: either the handler function reference or the
    /// name of a generated wrapper function (when prevent/stop/args apply)
    listener: String,
    /// capture phase flag (mirrored on removeEventListener)
    capture: bool,
    /// Explicit passive option (`passive: false` is required for
    /// preventDefault on document-level wheel/touch listeners in Chrome)
    passive: Option<bool>,
    /// Wrapper function source, emitted once before the onMounted block
    wrapper: Option<String>,
}

/// Data for generating interactive preview cards
#[derive(Debug, Clone)]
struct PreviewCardData {
    /// Unique identifier (e.g., "preview", "variants")
    id: String,
    /// Auto (AURA) source code
    auto_code: String,
    /// Vue.js source code
    vue_code: String,
}

/// 转义嵌入 SFC <script> 反引号模板字面量的代码串:字面反引号 / `${` 会
/// 提前终止 JS 字面量;字面 `</script` 会被 SFC 解析器当作脚本块结尾,
/// 截断整个组件(toast 页 Usage 示例含 </script> 曾整页编译失败)。
fn sanitize_embedded_code(code: &str) -> String {
    code.replace('`', "\\`")
        .replace("${", "\\${")
        .replace("</script", "<\\/script")
}

/// Data for generating code blocks with copy button
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CodeBlockData {
    /// Unique identifier (e.g., "install-button", "install-card")
    id: String,
    /// Code content
    code: String,
    /// Language (e.g., "bash", "typescript")
    lang: String,
}

/// Plan 563: canvas 元素登记(模板臂收集,script 臂消费)——一个 canvas
/// 元素 = 一条 spec:模板出 `<canvas ref>`,script 出 redraw(watch deep
/// + onMounted 初绘)、pen 三包装函数(mousedown 内联;document
/// mousemove/mouseup 经 global_listeners 挂载——出界即收笔与 iced
/// PenArea 同语义)、window resize 重绘。
#[derive(Debug, Clone)]
struct CanvasSpec {
    /// 模板 ref 名(__canvas_ref_N,进 template_refs)。
    ref_name: String,
    /// scene 状态前缀(strokes → strokes_pts/strokes_meta 两 ref)。
    prefix: String,
    /// coords "WxH" 逻辑幅面(None = raw px)。
    extent: Option<(f32, f32)>,
    /// 背景色(CSS 串;None = 透明,eraser 笔画取白)。
    clear: Option<String>,
    /// onpenstart/onpenmove/onpenend 的 handler 函数名(裸名,包装函数
    /// 里以 (x, y) 逻辑坐标实参调用;None = 该事件未声明)。
    on_start: Option<String>,
    on_move: Option<String>,
    on_end: Option<String>,
    /// PLAN-661 T-05（R-1）：onhit handler 函数名（收命中元素 id 字符串；
    /// tap 判定 = pointerdown→pointerup 位移 ≤ 4px，nodes 声明序倒序
    /// topmost，与 iced PenArea 同语义不共享代码）。
    on_hit: Option<String>,
    /// PLAN-661 T-05：图元三表是否在 model 声明（未声明 → 重绘/命中空集，
    /// 不引用未声明 ref 防 vue-tsc 编译红）。
    nodes_declared: bool,
    edges_declared: bool,
    labels_declared: bool,
}

/// PLAN-617 T-07: 受控 `video` 元素登记（模板臂收集、script 臂消费）。
///
/// 触发条件：元素**声明了 §2.3 的任一受控下行 prop**（`paused`/`position`/
/// `volume`/`muted`/`rate`）。未声明时**不进这条臂**，走通用原生元素路径，
/// 生成结果与引入本契约之前逐字节一致（兼容 019-video-app 等既有示例）。
///
/// 契约形状与 `crates/auto-lang/src/ui/mpv/contract.rs`（VM 侧的同一份契约）
/// 同形同名同单位：`volume` 是作者面 **0..100**，本臂在 JS 里翻成元素的 0..1；
/// `position` 是**秒**，语义是「目标值变化才 seek」（不是「与当前播放位置不同
/// 就 seek」——后者会让播放在前进时每帧重 seek，把画面钉死在目标点）。
#[derive(Debug, Clone, Default)]
struct VideoSpec {
    /// 模板 ref 名（`__video_ref_N`，进 template_refs 与真元素类型声明）。
    ref_name: String,
    /// 下行字段的**脚本域** JS 表达式（`transpile_expr_pub` 产出，ref 带
    /// `.value`）。未声明的字段为 `None`，`apply` 实参按默认值兜底。
    down_paused: Option<String>,
    down_position: Option<String>,
    down_volume: Option<String>,
    down_muted: Option<String>,
    down_rate: Option<String>,
    /// `src` 的脚本域表达式。**不是**受控字段（§2.3 明写 `src` 为透传），
    /// 仅用于「换片即作废上一次 seek 目标」的判定。
    down_src: Option<String>,
    /// 上行回灌：契约事件名 → handler 函数名（如 `onOnTime`）。未声明的
    /// 事件不发射监听器。
    up_time: Option<String>,
    up_metadata: Option<String>,
    up_playstate: Option<String>,
    up_ended: Option<String>,
    up_error: Option<String>,
    /// PLAN-617 T-11/AC-16(b)：音轨可用性回灌（`onaudiotrack`）。handler 收
    /// 一个 bool —— `false` = 实际播放超过 1s 仍解不出任何音频字节（Chromium
    /// 专有探针 `webkitAudioDecodedByteCount`），界面须如实标注「音轨不受
    /// 支持」；探针不存在（非 Chromium）时**不发射任何结论**（handler 不被
    /// 调用），VM/mpv 端无此事件（mpv 自带 Dolby 解码，无此问题）。
    up_audiotrack: Option<String>,
}

/// Result of evaluating a style/class `if`-branch body.
/// `Leaf(s)` is a plain string literal → emitted as `'s'`.
/// `Nested(t)` is an inner if expression → emitted as `(t)` (a ternary).
/// (DF-1: nested if in style binding was previously flattened to empty string.)
enum StyleBranch {
    Leaf(String),
    Nested(String),
}


// PLAN-014 T-04：按职责家族拆分的 impl 子模块（原单文件 impl 块的纯移动）。
// mod.rs 保留：类型定义（VueGenerator/VueMode/payload/StyleBranch/
// WidgetTemplate）、自由助手、BackendGenerator/Default trait impl、
// generate_sfc 编排与测试；家族方法分散于下列文件，全部仍 impl VueGenerator。
mod nav_sidebar;
mod script;
mod template_media;
mod design_theme;
mod expr;
mod tags;
mod style_helpers;
mod shadcn_attrs;
mod project_files;
mod store;
mod library;

// PLAN-014 T-04：跨模块 library 助手（拆分后经路径导入，语义不变）。
use library::{attribution_header, library_template, LIBRARY_UTILS_TS};

impl VueGenerator {
    /// Create a new Vue generator (Plain Tailwind mode, TypeScript output)
    pub fn new() -> Self {
        Self {
            select_markers: std::env::var("AUTOUI_SELECT_MARKERS").as_deref() == Ok("1"),
            source_stem: "app".to_string(),
            current_widget: None,
            design_entries: Vec::new(),
            theme_root_ref: false,
            imports: Vec::new(),
            state_names: Vec::new(),
            prop_names: Vec::new(),
            prop_types: HashMap::new(),
            state_types: HashMap::new(),
            int_names: std::collections::HashSet::new(),
            store_deps: Vec::new(),
            store_surface: Default::default(),
            store_import_prefix: "@/stores".to_string(),
            uses_autodown: false,
            last_validation_warnings: Vec::new(),
            codegen_warnings: std::cell::RefCell::new(Vec::new()),
            computed_names: std::collections::HashSet::new(),
            handlers: Vec::new(),
            debounced_handlers: HashSet::new(),
            emit_events: Vec::new(),
            quoted_events: std::collections::HashSet::new(),
            callback_contract_events: std::collections::HashSet::new(),
            has_emit: false,
            component_refs: Vec::new(),
            lucide_icons: HashSet::new(),
            iconfile_theme_css: false,
            wrapper_classes: String::new(),
            mode: VueMode::Plain,
            default_classes: true,
            widget_registry: WidgetRegistry::with_defaults(),
            shadcn_components_used: HashSet::new(),
            use_typescript: true, // Plan 100: TypeScript by default
            previewcard_counter: 0,
            previewcard_data: Vec::new(),
            needs_copy_code: false,
            codeblock_counter: 0,
            codeblock_data: Vec::new(),
            needs_router: false,
            needs_route: false,
            api_functions_used: HashSet::new(),
            project_api_functions: {
                // DEPRECATED: Env var fallback for backward compatibility.
                // New code should use `with_project_api_functions()` from explicit imports.
                let val = std::env::var("AUTO_API_FUNCTIONS").unwrap_or_default();
                val.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            },
            used_handlers: HashSet::new(),
            has_dark_mode: false,
            has_theme_name: false,
            dark_mode_var: None,
            dark_mode_on_widget: false,
            has_accent_color: false,
            use_theme_toggle: false,
            needs_toast_import: false,
            needs_vm_only_helper: false,
            known_sub_widgets: HashSet::new(),
            sub_widget_models: std::collections::HashMap::new(),
            sub_widget_msgs: std::collections::HashMap::new(),
            bound_model_channels: std::collections::HashSet::new(),
            emitted_model_bindings: Vec::new(),
            current_loop_var: None,
            current_loop_var_is_index: false,
            in_svg_subtree: false,
            widget_key_counter: 0,
            mention_helper_needed: false,
            loop_param_handlers: HashMap::new(),
            msg_payload_arities: HashMap::new(),
            handler_params: HashMap::new(),
            needs_child_delete_handler: false,
            explicit_api_imports: false,
            global_listeners: Vec::new(),
            template_refs: Vec::new(),
            surface_pans: Vec::new(),
            component_ref_names: HashSet::new(),
            scroll_auto_scroll: Vec::new(),
            scroll_ctl_helpers_needed: false,
            pending_scroll_sentinel: None,
            canvas_specs: Vec::new(),
            video_specs: Vec::new(),
            progress_seek_specs: Vec::new(),
            ext_components: HashMap::new(),
            ext_import_lines: Vec::new(),
            ext_composables: Vec::new(),
            current_actions: None,
            actions_key_bindings: Vec::new(),
            bind_key_bindings: Vec::new(),
            facade_ref_fields: std::collections::HashMap::new(),
            setup_locals: Vec::new(),
            setup_ref_fields: std::collections::HashMap::new(),
            current_setup_stmts: None,
            module_fns: Vec::new(),
            use_module_fns: Vec::new(),
            use_imported_names: std::collections::HashSet::new(),
        }
    }

    /// Attach module-level plain functions from the source file
    pub fn with_module_fns(mut self, module_fns: Vec<crate::aura::AuraModuleFn>) -> Self {
        self.module_fns = module_fns;
        self
    }

    /// Plan 522: attach the use-imported module fn pool + the bare names the
    /// widget file explicitly imports (`use module: a, b` items). Fns referenced
    /// from computed/handler bodies are transpiled into the SFC script tail.
    pub fn with_use_module_fns(
        mut self,
        fns: Vec<crate::aura::AuraModuleFn>,
        imported_names: Vec<String>,
    ) -> Self {
        self.use_module_fns = fns;
        self.use_imported_names = imported_names.into_iter().collect();
        self
    }

    /// Set known sub-widget names (to avoid shadcn name collisions)
    pub fn with_sub_widgets(mut self, names: Vec<String>) -> Self {
        self.known_sub_widgets = names.into_iter().collect();
        self
    }

    /// PLAN-037 T5: provide the sub-widget model-var map (name -> channels).
    pub fn with_sub_widget_models(
        mut self,
        models: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        self.sub_widget_models = models;
        self
    }

    /// Plan 443: names of the CURRENT widget's model channels that some
    /// parent call-site actually binds (emits `v-model:name`). Only those
    /// channels compile to `defineModel`; every other model var stays a
    /// plain `ref` so unbound widgets keep deep reactivity
    /// (`doc.value.shapes.push()` must track — defineModel's unbound get
    /// returns the raw local value, not a reactive proxy).
    pub fn with_bound_model_channels(mut self, channels: Vec<String>) -> Self {
        self.bound_model_channels = channels.into_iter().collect();
        self
    }

    /// Set project-specific API function names (from explicit `use back.api: ...` imports)
    /// When set via this method (from explicit imports), skip AST scanning and use this list directly.
    pub fn with_project_api_functions(mut self, functions: Vec<String>) -> Self {
        if !functions.is_empty() {
            self.explicit_api_imports = true;
        }
        self.project_api_functions = functions;
        self
    }

    /// Set store dependencies from `use store:` declarations (Plan 351).
    pub fn with_store_deps(mut self, deps: Vec<String>) -> Self {
        self.store_deps = deps;
        self
    }

    /// PLAN-010: set the project-level store surface（store 真名 → 暴露方法名）。
    pub fn with_store_surface(
        mut self,
        surface: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        self.store_surface = surface
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        self
    }

    /// Plan 446 批三 G1: set the store composable import path prefix.
    /// Default `@/stores`; deployments relocating generated composables
    /// (e.g. `src/stores/auto/`) pass the matching prefix.
    pub fn with_store_import_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.store_import_prefix = prefix.into();
        self
    }

    /// Plan 444 (ash-shell-057 ①b): provide the sub-widget emit roster —
    /// widget name → the event names that widget actually emits (see
    /// [`Self::widget_emit_set`]). Parent-side `on_x: .Y` callback bindings
    /// consult it to pick the event the child really fires.
    pub fn with_sub_widget_msgs(
        mut self,
        msgs: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        self.sub_widget_msgs = msgs;
        self
    }

    /// Plan 444 (ash-shell-057 ①b): the full set of emit names a widget's
    /// SFC can fire: its msg variant names plus the Pascal forms of every
    /// `on_*: msg` callback prop its handlers invoke (`props.on_x()` /
    /// bare `on_x()` — both rewrite to `emit('<Pascal>', ...)`). Drivers
    /// harvest this per sub-widget and hand it to the parents' generators.
    pub fn widget_emit_set(widget: &AuraWidget) -> Vec<String> {
        let mut emits: Vec<String> = widget
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.clone()))
            .collect();
        let callback_props: Vec<String> = widget
            .props
            .iter()
            .filter(|p| Self::prop_is_emitted_callback(p, widget))
            .filter_map(|p| p.name.strip_prefix("on_").map(|s| s.to_string()))
            .collect();
        if callback_props.is_empty() {
            return emits;
        }

        for payload in widget.handlers.values() {
            if matches!(payload, crate::aura::LogicPayload::AstStmts(_)) {
                for snake in &callback_props {
                    let pascal = Self::snake_to_pascal(snake);
                    if !emits.contains(&pascal) {
                        emits.push(pascal);
                    }
                }
            }
        }
        emits
    }

    /// Mark whether the project depends on @autodown/editor (Plan 361).
    /// Drives R003 validation (CSS import check) and is consumed by
    /// auto-man's generate_main_ts to inject the stylesheet import.

    /// Check if a name is a known API function (static list OR project-specific)
    fn is_api_function(&self, name: &str) -> bool {
        Self::API_FUNCTIONS.contains(&name) || self.project_api_functions.iter().any(|f| f == name)
    }

    /// Get the combined list of all known API function names
    fn all_api_functions(&self) -> Vec<String> {
        let mut fns: Vec<String> = Self::API_FUNCTIONS.iter().map(|s| s.to_string()).collect();
        fns.extend(self.project_api_functions.iter().cloned());
        fns
    }

    /// Create a new Vue generator in shadcn-vue mode
    pub fn new_shadcn() -> Self {
        Self {
            mode: VueMode::Shadcn,
            widget_registry: WidgetRegistry::with_defaults(),
            ..Self::new()
        }
    }

    /// Create a new Vue generator in library mode (Plan 331): emits
    /// self-contained per-widget SFCs backed by `reka-ui`.
    pub fn new_library() -> Self {
        Self {
            mode: VueMode::Library,
            widget_registry: WidgetRegistry::with_defaults(),
            ..Self::new()
        }
    }

    /// Set the generation mode
    pub fn with_mode(mut self, mode: VueMode) -> Self {
        self.mode = mode;
        self
    }

    /// fix-ui-tier：显式 select 标记开关。PLAN-646 dev 运行面的默认激活
    /// 仍是 AUTOUI_SELECT_MARKERS env（构造器读取，见 new()）；测试/嵌入
    /// 侧一律走本 builder——env 是进程级全局，`cargo tu` 单进程档下测试
    /// set_var 会串扰同进程所有后续生成器构造（a2vue golden 23 测齐红实录）。
    pub fn with_select_markers(mut self, on: bool) -> Self {
        self.select_markers = on;
        self
    }

    /// Set the `default_classes` toggle (pac.at `default_classes: off`).
    /// When false, `extract_classes` skips the doc-theme default Tailwind
    /// classes for everything except layout primitives (row/col/grid/...).
    /// PLAN-646: 设置源文件 stem（data-auto-src 注入）。
    pub fn with_source_stem(mut self, stem: impl Into<String>) -> Self {
        self.source_stem = stem.into();
        self
    }

    pub fn with_default_classes(mut self, default_classes: bool) -> Self {
        self.default_classes = default_classes;
        self
    }

    /// Set whether to use TypeScript output (Plan 100)
    pub fn with_typescript(mut self, use_typescript: bool) -> Self {
        self.use_typescript = use_typescript;
        self
    }

    /// Check if using shadcn-vue mode
    pub fn is_shadcn(&self) -> bool {
        self.mode == VueMode::Shadcn
    }

    /// Check if using library mode (Plan 331)
    pub fn is_library(&self) -> bool {
        self.mode == VueMode::Library
    }

    /// Check if outputting TypeScript (Plan 100)

    /// Generate a self-contained SFC for a single primitive widget (Plan 331).
    ///
    /// Emits a standalone `.vue` file backed by `reka-ui` (never
    /// `@/components/ui/*`), driven by the widget's library template.
    pub fn generate_widget_sfc(&mut self, name: &str) -> GenResult<String> {
        let tpl =
            library_template(name).ok_or_else(|| GenError::UnknownWidget(name.to_string()))?;
        Ok(format!(
            "{header}\n<script setup lang=\"ts\">\n{script}\n</script>\n\n<template>\n{template}\n</template>\n",
            header = attribution_header(name),
            script = tpl.script,
            template = tpl.template,
        ))
    }

    /// Emit the per-widget support files (relative path, contents) that the
    /// generated SFC depends on, so a copied component is self-contained.
    pub fn generate_widget_support_files(&self, name: &str) -> Vec<(String, String)> {
        let pascal = pascal_case(name);
        // Collect every `.vue` file this widget emits: the primary SFC plus any
        // companion `.vue` files declared in `extra_support_files` (composite
        // widgets like card/dialog/tabs ship several SFCs in one directory).
        let mut vue_files = vec![format!("{pascal}.vue")];
        let mut extras: Vec<(String, String)> = Vec::new();
        if let Some(tpl) = library_template(name) {
            for (n, c) in tpl.extra_support_files.iter() {
                let n = n.to_string();
                if n.ends_with(".vue") {
                    vue_files.push(n.clone());
                }
                extras.push((n, c.to_string()));
            }
        }
        // index.ts re-exports every emitted `.vue` by its PascalCase basename.
        let mut index = String::new();
        for file in &vue_files {
            let stem = file.trim_end_matches(".vue");
            index.push_str(&format!("export {{ default as {stem} }} from './{file}'\n"));
        }
        let mut files = vec![("index.ts".to_string(), index)];
        files.extend(extras);
        files
    }

    /// Files shared by every library widget, written once at the registry root
    /// (Plan 331). Currently the `cn` class-merge helper that all generated
    /// SFCs import as `../utils`. `auto ui build` emits these alongside the
    /// widget directories; `auto-ui add` copies them into the consumer root.
    pub fn library_shared_files(&self) -> Vec<(&'static str, &'static str)> {
        vec![("utils.ts", LIBRARY_UTILS_TS)]
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.imports.clear();
        self.state_names.clear();
        self.prop_names.clear();
        self.prop_types.clear();
        self.state_types.clear();
        self.int_names.clear();
        self.handlers.clear();
        self.emit_events.clear();
        self.quoted_events.clear();
        self.has_emit = false;
        self.component_refs.clear();
        self.lucide_icons.clear();
        self.wrapper_classes.clear();
        self.current_loop_var = None;
        self.current_loop_var_is_index = false;
        self.in_svg_subtree = false;
        self.widget_key_counter = 0;
        self.mention_helper_needed = false;
        self.loop_param_handlers.clear();
        self.handler_params.clear();
        self.needs_child_delete_handler = false;
        // NOTE: explicit_api_imports is NOT reset — it's a config-level setting from with_project_api_functions()
        self.shadcn_components_used.clear();
        self.previewcard_counter = 0;
        self.previewcard_data.clear();
        self.needs_copy_code = false;
        self.codeblock_counter = 0;
        self.codeblock_data.clear();
        self.needs_router = false;
        self.needs_route = false;
        self.api_functions_used.clear();
        // NOTE: project_api_functions is NOT cleared on reset — it's config-level,
        // loaded once from AUTO_API_FUNCTIONS env var, and persists across widget generation.
        self.used_handlers.clear();
        self.computed_names.clear();
        self.codegen_warnings.borrow_mut().clear();
        self.has_dark_mode = false;
        self.dark_mode_var = None;
        self.has_theme_name = false;
        self.dark_mode_on_widget = false;
        self.has_accent_color = false;
        self.use_theme_toggle = false;
        self.needs_toast_import = false;
        self.needs_vm_only_helper = false;
        self.global_listeners.clear();
        self.template_refs.clear();
        self.surface_pans.clear();
        self.canvas_specs.clear();
        self.video_specs.clear();
        self.progress_seek_specs.clear();
        self.ext_components.clear();
        self.ext_import_lines.clear();
        self.ext_composables.clear();
        self.facade_ref_fields.clear();
        self.setup_locals.clear();
        self.setup_ref_fields.clear();
        self.current_setup_stmts = None;
        self.current_actions = None;
        self.actions_key_bindings.clear();
        self.bind_key_bindings.clear();
    }

    /// Convert kebab-case icon name to PascalCase Lucide component name
    fn kebab_to_pascal(s: &str) -> String {
        s.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    /// PLAN-706: 字面量 icon 名 → 合法 lucide 组件名。
    /// 未命中全量表（如 kitchen-sink 的占位 `"sample"`）回退 `Circle`，
    /// 与动态名降级同族，避免 `import { Sample }` 整页炸掉。
    fn lucide_component_name(kebab: &str) -> String {
        let pascal = Self::kebab_to_pascal(kebab);
        // PLAN-705 merge-fix：全量表在 ui-gated 的 lucide_generated 里——
        // 纯 VM 形态（--no-default-features）无此表，全量校验跳过（保守
        // 直出 Pascal 名，706 的回退语义仅在 ui 形态生效）。
        {
            if crate::lucide_generated::is_known(kebab) {
                return pascal;
            }
            return "Circle".to_string();
        }
        #[cfg(not(feature = "ui"))]
        {
            let _ = kebab;
            pascal
        }
    }

    // ====================================================================
    // Widget `use { ... }` external TS/Vue imports (escape hatch)
    // ====================================================================

    /// True when a widget `use { ... }` import path refers to a
    /// project-local file (copied into the generated project under
    /// `src/ext/` by auto-man) rather than an npm package specifier.
    fn ext_is_local_path(path: &str) -> bool {
        // Plan 028 P1（T18）：平台能力协议声明（"platform:markdown" 等）——
        // 由 auto-man 挂载实现到 gen src/platform/，导入 @/platform/<name>。
        path.starts_with("platform:")
            || path.starts_with('.')
            || path.starts_with('/')
            || path.ends_with(".vue")
            || path.ends_with(".ts")
            || path.ends_with(".tsx")
            || path.ends_with(".js")
            || path.ends_with(".mjs")
            // Plan 028 M1: `.at` fn modules — auto-man transpiles them to a
            // TS module at the same ext path (extension stripped).
            || path.ends_with(".at")
    }

    /// Map a declared import path to the specifier used in the generated
    /// SFC. npm specifiers pass through unchanged; local files are copied
    /// by auto-man into `src/ext/<path>` (preserving the
    /// project-root-relative layout so sibling relative imports keep
    /// working) and imported through the `@` alias. TypeScript/JavaScript
    /// extensions are dropped (bundler resolution); `.vue` is kept.
    fn ext_import_specifier(path: &str) -> String {
        if let Some(name) = path.strip_prefix("platform:") {
            // 实现 .vue 需显式扩展名（vue-tsc 的模块解析）
            return format!("@/platform/{}.vue", name);
        }
        if !Self::ext_is_local_path(path) {
            return path.to_string();
        }
        let rel = path.trim_start_matches("./").trim_start_matches('/');
        let stem = rel
            .strip_suffix(".tsx")
            .or_else(|| rel.strip_suffix(".ts"))
            .or_else(|| rel.strip_suffix(".mjs"))
            .or_else(|| rel.strip_suffix(".js"))
            .or_else(|| rel.strip_suffix(".at"))
            .unwrap_or(rel);
        format!("@/ext/{}", stem)
    }

    /// The view tag spellings accepted for an external component: the
    /// declared name itself plus its snake_case and kebab-case forms
    /// (`FancyBadge` → `FancyBadge`, `fancy_badge`, `fancy-badge`).
    fn ext_tag_keys(name: &str) -> Vec<String> {
        let mut snake = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_ascii_uppercase() && i > 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        }
        let kebab = snake.replace('_', "-");
        let mut keys = vec![name.to_string(), snake.clone()];
        if kebab != snake {
            keys.push(kebab);
        }
        keys
    }

    /// `useMenuBounds` → `menuBounds` (strip the `use` prefix, lowercase
    /// the first letter) — the local const a composable's return value is
    /// bound to at `<script setup>` top level.
    fn ext_composable_local_name(callee: &str) -> String {
        let base = callee.strip_prefix("use").unwrap_or(callee);
        let mut chars = base.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
            None => base.to_string(),
        }
    }

    /// Register a widget's `use { ... }` external imports: build the
    /// tag → component lookup used while generating the view tree, the
    /// import lines emitted into `<script setup>`, and the composable
    /// call list. Called from `generate_sfc` before template generation.
    fn register_ext_imports(&mut self, widget: &AuraWidget) {
        for imp in &widget.ext_imports {
            let specifier = Self::ext_import_specifier(&imp.path);
            let symbols: Vec<String> = imp.symbols.iter().map(|s| s.as_str().to_string()).collect();
            match imp.kind {
                // Plan 435 P4:包引用不产 ES import —— 包组件经 ComponentRegistry
                // 注册,tag 走 sub-widget 生成路径(api 层已并入 sub_widgets)。
                auto_ui_dialect::ast::ExtImportKind::Package => continue,
                auto_ui_dialect::ast::ExtImportKind::Fn
                | auto_ui_dialect::ast::ExtImportKind::ExplicitFn
                | auto_ui_dialect::ast::ExtImportKind::Composable => {
                    self.ext_import_lines.push((
                        symbols.clone(),
                        false,
                        format!("import {{ {} }} from '{}'\n", symbols.join(", "), specifier),
                    ));
                    if imp.kind == auto_ui_dialect::ast::ExtImportKind::Composable {
                        // Plan 022: composable 调用参数 → JS 字符串（逗号分隔）。
                        // 空 call_args 生成 ""（→ 无参调用 useX()，向后兼容）。
                        // Plan 012 P0#13 follow-up: an arg form the bound-value
                        // transpiler rejects used to silently become `null`
                        // (then dropped by unwrap_or_default) — warn R013.
                        let args_js = imp
                            .call_args
                            .iter()
                            .map(|a| {
                                self.bound_value_or_warn(
                                    a,
                                    &format!("composable `{}` call args", imp.path),
                                    "null",
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        for sym in &symbols {
                            let local = Self::ext_composable_local_name(sym);
                            self.ext_composables.push((
                                local.clone(),
                                sym.clone(),
                                args_js.clone(),
                            ));
                            // Plan 408 P12 §10.4: 收集 ref 字段标注。
                            if !imp.ref_fields.is_empty() {
                                self.facade_ref_fields.insert(
                                    local,
                                    imp.ref_fields
                                        .iter()
                                        .map(|f| f.as_str().to_string())
                                        .collect(),
                                );
                            }
                        }
                    }
                }
                auto_ui_dialect::ast::ExtImportKind::Component => {
                    for sym in &symbols {
                        // Plan 408: empty `from` (no source path) means this is
                        // a reference to a same-project `component fn` SFC,
                        // synthesized to `@/components/{Name}.vue`. No file copy
                        // happens — the SFC is produced by the build itself.
                        // Treat it like a local .vue (default export, same path
                        // convention as sub-widget imports in vue.rs:1860).
                        let line = if imp.path.is_empty() {
                            format!("import {} from '@/components/{}.vue'\n", sym, sym)
                        } else if imp.path.starts_with("platform:") {
                            // Plan 028 T18：平台实现（platform:markdown 等）是
                            // .vue default export，auto-man 挂载到 src/platform/。
                            format!("import {} from '{}'\n", sym, specifier)
                        } else if imp.path.ends_with(".vue") {
                            // Local `.vue` files are default-exported; everything
                            // else (npm packages, .ts modules) uses named exports.
                            format!("import {} from '{}'\n", sym, specifier)
                        } else {
                            format!("import {{ {} }} from '{}'\n", sym, specifier)
                        };
                        self.ext_import_lines.push((vec![sym.clone()], true, line));
                        let comp = ExtComponent { name: sym.clone() };
                        for key in Self::ext_tag_keys(sym) {
                            self.ext_components.insert(key, comp.clone());
                        }
                    }
                }
            }
        }
    }

    /// Get Tailwind color classes for category section
    fn category_color_classes(color: &str) -> (&'static str, &'static str) {
        match color {
            "blue" => ("bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800", "bg-blue-500"),
            "emerald" => ("bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800", "bg-emerald-500"),
            "amber" => ("bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-800", "bg-amber-500"),
            "purple" => ("bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-800", "bg-purple-500"),
            "rose" => ("bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-200 dark:border-rose-800", "bg-rose-500"),
            // Plan 412 F7: Layout 分组用 sky(#0ea5e9)
            "sky" => ("bg-sky-500/10 text-sky-600 dark:text-sky-400 border-sky-200 dark:border-sky-800", "bg-sky-500"),
            _ => ("bg-gray-500/10 text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-800", "bg-gray-500"),
        }
    }

    /// Generate category-section HTML (component grid with heading)
    // ─────────────────────────────────────────────────────────────────────
    // Plan 482: nav 组件族 —— nav-item / nav-group / nav(search:)。
    // shadcn 模式 → NavItem/NavGroup 脚手架组件（@/components/ui/nav，
    // 类契约内建于组件）；非 shadcn 模式（如 os-config）→ 内联契约标记
    // （与 ui_gen::nav_contract 同串，VM aura_view_builder 同构）。
    // ─────────────────────────────────────────────────────────────────────

    /// Plan 482: prop → attr 片段。字符串字面量 → 静态属性；其余表达式 →
    /// `:key="expr"` 绑定（bound_value_or_warn 通道，保操作符）。
    fn nav_attr_fragment(&self, key: &str, value: &AuraPropValue) -> Option<String> {
        use auto_lang_core::ast::Expr;
        match value {
            AuraPropValue::Expr(Expr::Str(s)) | AuraPropValue::Expr(Expr::CStr(s)) => Some(
                format!("{}=\"{}\"", key, Self::escape_html_attr(s.as_str())),
            ),
            AuraPropValue::Expr(Expr::Bool(b)) => Some(format!(":{}=\"{}\"", key, b)),
            AuraPropValue::Expr(expr) => {
                let js = self.bound_value_or_warn(expr, &format!("nav prop `{key}`"), "null");
                Some(format!(":{}=\"{}\"", key, js))
            }
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Plan 482: 用户 class/style 追加属性（契约类之后，逃生通道）。
    fn nav_user_class_attr(&self, props: &HashMap<String, AuraPropValue>) -> Option<String> {
        let user = props.get("class").or_else(|| props.get("style"))?;
        if let Some(s) = self.extract_string_value(user) {
            if s.trim().is_empty() {
                return None;
            }
            return Some(format!("class=\"{}\"", Self::escape_html_attr(s.trim())));
        }
        if let Some(state_ref) = self.extract_state_ref(user) {
            return Some(format!(":class=\"{}\"", state_ref));
        }
        None
    }

    /// Plan 482: nav-item 事件（onclick/click）→ `@click="..."` 片段，
    /// 循环变量转发与通用元素路径同规（3651-3673）。
    fn nav_click_attr(&mut self, events: &HashMap<String, AuraEvent>) -> Option<String> {
        let event = events.get("onclick").or_else(|| events.get("click"))?;
        let mut handler_fn =
            self.handler_to_function_call_with_params(&event.handler, &event.params);
        let handler_name = self.handler_to_function_call(&event.handler);
        if let Some(ref loop_var) = self.current_loop_var {
            if event.params.is_empty() {
                let handler_takes_payload = self
                    .msg_payload_arities
                    .get(&handler_name)
                    .map(|n| *n > 0)
                    .unwrap_or(false);
                if handler_takes_payload {
                    handler_fn = format!("{}($event)", handler_fn);
                } else {
                    handler_fn = format!("{}({})", handler_fn, loop_var);
                    self.loop_param_handlers
                        .insert(handler_name.clone(), loop_var.clone());
                }
            }
        }
        self.used_handlers.insert(handler_name);
        Some(format!("@click=\"{}\"", handler_fn))
    }

}
impl VueGenerator {
    /// Generate complete Vue3 SFC
    pub fn generate_sfc(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.reset();
        // PLAN-031 F-03：design 块随 widget 携带，投影前先落到生成器
        //（reset 之后设置，避免被清空）。
        self.design_entries = widget.design.clone();
        // PLAN-037 T5 (D1 rule 3): a prop and a model var of the same name
        // would compile to conflicting defineProps/defineModel entries.
        for prop in &widget.props {
            if widget.state_vars.iter().any(|sv| sv.name == prop.name) {
                return Err(crate::ui_gen::GenError::InvalidStateRef(format!(
                    "widget `{}` declares both a prop and a model var named `{}`; rename one (model vars are bindable channels via `v-model:{}`)",
                    widget.name, prop.name, prop.name
                )));
            }
        }
        // Widget `use { ... }` external imports — must be registered before
        // template generation so view tags resolve to external components.
        self.register_ext_imports(widget);

        // Plan 451 P2: stash the `actions {}` block and resolve the shortcut
        // table (normalized key → handler fn, first declaration wins — same
        // collision rule as the vm's shortcut_bindings). Template synthesis
        // (menubar/toolbar placeholders) and the script-side keydown fallback
        // layer both read these.
        self.current_actions = widget.actions.clone();
        if let Some(ref acts) = widget.actions {
            use std::collections::HashSet;
            let mut seen: HashSet<String> = HashSet::new();
            for a in &acts.actions {
                let Some(sc) = a.shortcut.as_deref() else {
                    continue;
                };
                let key = crate::ui_gen::normalize_shortcut(sc);
                if key.is_empty() || !seen.insert(key.clone()) {
                    continue;
                }
                let handler_fn = self.pattern_to_handler_name(&a.handler);
                self.actions_key_bindings.push((key, handler_fn));
            }
        }

        // Plan 464 T2: `bind {}` 块 → bind_key_bindings（vm keyboard_subscription
        // 的 key_bindings 同源；AuraWidget 已把 bind 块化简为该 map，handler
        // 形如 ".MoveUp"，剥点得 fn 名——vm keyboard_event_message 同规则）。
        // 排序保证生成输出稳定（HashMap 无序）。
        if !widget.key_bindings.is_empty() {
            let mut entries: Vec<(&String, &String)> = widget.key_bindings.iter().collect();
            entries.sort();
            for (key, handler) in entries {
                let handler_fn = handler.strip_prefix('.').unwrap_or(handler).to_string();
                self.bind_key_bindings.push((key.clone(), handler_fn));
            }
        }

        // Plan 426: setup preamble — register binding names (facade-local
        // semantics) and refs annotations; name collisions with state/prop
        // are a compile error (PLAN-037 T5 precedent).
        if let Some(setup) = &widget.setup {
            // Plan 426 §1.5: setup statements run BEFORE state/computed/props
            // declarations (bindings first) — a reference to any of them is a
            // runtime TDZ error, rejected at compile time.
            let forbidden: std::collections::HashSet<&str> = widget
                .state_vars
                .iter()
                .map(|sv| sv.name.as_str())
                .chain(widget.computed.iter().map(|c| c.name.as_str()))
                .chain(widget.props.iter().map(|p| p.name.as_str()))
                .collect();
            for stmt in &setup.body.stmts {
                if let Some(name) = Self::stmt_references_any(stmt, &forbidden) {
                    return Err(crate::ui_gen::GenError::InvalidStateRef(format!(
                        "widget `{}` setup statement references `{}`, which is declared AFTER the setup preamble (state/computed/props run later); move the logic out of setup or into a handler",
                        widget.name, name
                    )));
                }
                if let auto_lang_core::ast::Stmt::Store(store) = stmt {
                    let name = store.name.as_str();
                    if widget.state_vars.iter().any(|sv| sv.name == name) {
                        return Err(crate::ui_gen::GenError::InvalidStateRef(format!(
                            "widget `{}` setup binding `{}` collides with a model var; rename one",
                            widget.name, name
                        )));
                    }
                    if widget.props.iter().any(|p| p.name == name) {
                        return Err(crate::ui_gen::GenError::InvalidStateRef(format!(
                            "widget `{}` setup binding `{}` collides with a prop; rename one",
                            widget.name, name
                        )));
                    }
                    self.setup_locals.push(name.to_string());
                }
            }
            for (binding, fields) in &setup.ref_annotations {
                self.setup_ref_fields
                    .insert(binding.clone(), fields.iter().cloned().collect());
            }
            self.current_setup_stmts = Some(setup.body.stmts.clone());
        }

        // Plan 043 H2: index this widget's msg variants (handler name → payload
        // arity) so sub-widget event bindings can decide between forwarding the
        // child's emit payload ($event) vs the for-loop variable.
        for msg in &widget.messages {
            for variant in &msg.variants {
                let handler_name = Self::sanitize_ident(&variant.name);
                self.msg_payload_arities
                    .insert(handler_name, variant.payload.len());
            }
        }

        // Plan 421: index the on-block handler params (pattern → param names)
        // before template generation so the event loop can forward the
        // code_editor scaffold's cursor/contextmenu payloads to handlers that
        // declare parameters.
        self.handler_params = widget.handler_params.clone();

        // Detect dark mode: check widget state vars, view tree, or handler names
        self.has_dark_mode = widget
            .state_vars
            .iter()
            .any(|s| s.name == "isDark" || s.name == "dark_mode");
        // Determine the dark mode state variable name for template binding
        self.dark_mode_var = widget
            .state_vars
            .iter()
            .find(|s| s.name == "isDark" || s.name == "dark_mode")
            .map(|s| s.name.clone());
        // Plan 458: widget-local declaration (root-App pattern) vs store
        // convention — decides the template's dark class binding source.
        self.dark_mode_on_widget = widget.state_vars.iter().any(|s| s.name == "dark_mode");
        // Plan 409 §8: widget-path accent support — the widget owns the
        // `accent_color` state (mirrors the store composable detection at
        // Plan 360; gallery App is a widget, not a store).
        self.has_accent_color = widget.state_vars.iter().any(|s| s.name == "accent_color");
        // PLAN-007 T-05：theme_name 状态（design.theme 数据绑定的 Vue 面）——
        // 声明即接通树状/命名主题热切换 watch。
        self.has_theme_name = widget.state_vars.iter().any(|s| s.name == "theme_name");
        // Also check if ToggleDarkMode handler exists (D1 fix: removed
        // format!("{:?}", widget.view_tree) which caused OOM on large
        // view trees with Expr::If nodes — handler check is sufficient)
        if !self.has_dark_mode {
            // Handler keys are stored as ".ToggleDarkMode" (with leading dot)
            let has_toggle = widget.handlers.contains_key(".ToggleDarkMode")
                || widget
                    .lifecycle
                    .iter()
                    .any(|l| l.name.contains("DarkMode") || l.name.contains("dark_mode"));
            if has_toggle {
                self.has_dark_mode = true;
                self.dark_mode_var = Some("dark_mode".to_string());
            }
        }

        // Pre-populate state_names so expr_to_js recognizes refs during template generation
        for state in &widget.state_vars {
            self.state_names.push(state.name.clone());
            self.state_types.insert(
                state.name.clone(),
                Self::auto_type_to_ts_type(&state.type_info),
            );
            if Self::is_int_auto_type(&state.type_info) {
                self.int_names.insert(state.name.clone());
            }
        }

        // Plan 012 Batch A (gap 44): register computed names so script-side
        // expressions can unwrap computed refs (`.c` → `c.value`), matching
        // the template side's auto-unwrap.
        for computed_prop in &widget.computed {
            self.computed_names.insert(computed_prop.name.clone());
        }

        // Register prop names (props are NOT refs — no .value suffix in script)
        for prop in &widget.props {
            self.prop_names.push(prop.name.clone());
            self.prop_types.insert(
                prop.name.clone(),
                Self::auto_type_to_ts_type(&prop.type_info),
            );
            if Self::is_int_auto_type(&prop.type_info) {
                self.int_names.insert(prop.name.clone());
            }
        }

        // PLAN-037 T3: callback-contract channels (`on_xxx: msg` prop with a
        // matching Pascal msg variant) always declare their emit.
        for prop in &widget.props {
            if Self::prop_is_emitted_callback(prop, widget) {
                if let Some(snake) = prop.name.strip_prefix("on_") {
                    self.callback_contract_events
                        .insert(Self::snake_to_pascal(snake));
                }
            }
        }

        // Plan 351: 'store' is a local const (from `const store = reactive(useXxxStore())`).
        // NOT a prop — must not be in prop_names, or ts_adapter rewrites .store.xxx
        // to props.store.xxx (wrong). As a bare ident, it passes through correctly
        // (not in state_names → no .value suffix; not in prop_names → no props. prefix).

        // Activate emit generation for sub-widgets that have messages
        for msg in &widget.messages {
            for variant in &msg.variants {
                // Plan 448 C: auto-sync mint variants never fire from a Vue
                // template (v-model carries the sync) — keep them out so the
                // defineEmits block disappears entirely.
                if variant.name.as_str().starts_with("__bind_") {
                    continue;
                }
                self.has_emit = true;
                self.emit_events.push(variant.name.clone());
                if variant.quoted {
                    self.quoted_events.insert(variant.name.clone());
                }
            }
        }
        // Plan 444 (ash-shell-057 ①a): callback-contract channels whose
        // Pascal name has no matching variant (017-chat `on_send` — Msg is
        // `DoSend`; the child rewrites `on_send()`/`props.on_send()` to
        // `emit('Send', ...)`) must still DECLARE that emit, or the rewrite
        // targets an undeclared event (TS2345 on emit()).
        if !self.callback_contract_events.is_empty() {
            self.has_emit = true;
            // Sorted for deterministic emission order (jade gap 56 lesson).
            let mut contracts: Vec<&String> = self.callback_contract_events.iter().collect();
            contracts.sort();
            for contract in contracts {
                if !self.emit_events.iter().any(|e| e == contract) {
                    self.emit_events.push(contract.clone());
                }
            }
        }

        // Generate template first to collect shadcn components used and detect Outlet/Link
        let template = self.generate_template(&widget.view_tree)?;

        // Plan 105: Check handlers for NavCall
        if self.widget_needs_router(widget) {
            self.needs_router = true;
        }
        // Plan 235: Check handlers for route access
        if Self::widget_needs_route(widget) {
            self.needs_route = true;
        }
        // Plan 235: Pre-analyze handlers for route access and navigation
        // (ts_adapter builtins like router.param() emit useRoute() which we need to import)
        for payload in widget.handlers.values() {
            if let Ok(body) = self.generate_handler_body(payload) {
                if body.contains("useRoute") {
                    self.needs_route = true;
                }
            }
            // Also check raw AST for router.push / router.replace
            match payload {
                LogicPayload::AstStmts(stmts) => {
                    if crate::ui_gen::ts_adapter::stmts_have_router_nav(stmts) {
                        self.needs_router = true;
                    }
                }
                _ => {}
            }
        }
        // Lifecycle handlers (.Init → onMounted, .Destroy → onUnmounted) can
        // also read route params (e.g. .Init -> { .id = router.param("id") }),
        // so check them for route access + navigation too.
        for lc in &widget.lifecycle {
            if let Ok(body) = self.generate_handler_body(&lc.payload) {
                if body.contains("useRoute") {
                    self.needs_route = true;
                }
            }
            if let LogicPayload::AstStmts(stmts) = &lc.payload {
                if crate::ui_gen::ts_adapter::stmts_have_router_nav(stmts) {
                    self.needs_router = true;
                }
            }
        }

        // Then generate script (which can now include shadcn imports and router)
        let script = self.generate_script(widget)?;
        let style = self.generate_style();

        // Widget-level native CSS (`style { ... }` block) — captured verbatim
        // by the lexer and emitted unchanged into a dedicated `<style scoped>`
        // block. Never interpreted, never merged with the generated <style>.
        let scoped_style = match &widget.style_css {
            Some(css) => format!("\n<style scoped>\n{}</style>\n", css),
            None => String::new(),
        };

        // Plan 100: Add lang="ts" for TypeScript output
        let script_tag = if self.use_typescript {
            r#"<script setup lang="ts">"#
        } else {
            r#"<script setup>"#
        };

        let sfc = format!(
            r#"<!-- {} component - Auto-generated from Auto language -->
{}
{}
</script>

<template>
{}
</template>

<style>
{}
</style>
{}"#,
            widget.name, script_tag, script, template, style, scoped_style
        );

        // Plan 361: Post-generation validation. Run all rules against the SFC
        // and stash warnings so callers (generate_component_from_file, auto-man)
        // can print/log them. Non-fatal — generation always succeeds.
        let ctx = crate::ui_gen::validators::ValidationContext {
            store_deps: self.store_deps.clone(),
            uses_autodown: self.uses_autodown,
            used_handlers: self.used_handlers.iter().cloned().collect(),
            strict: false,
        };
        let warnings = crate::ui_gen::validate_sfc(&sfc, &widget.name, &ctx);
        // Plan 012 Batch A: codegen-time warnings (dropped/degraded/passed-through
        // behavior detected while generating) share the same channel. Dedup by
        // (rule, message) — handler bodies are transpiled more than once
        // (route pre-analysis + real generation), so R010 notes repeat.
        let mut all_warnings: Vec<crate::ui_gen::validators::ValidationWarning> = Vec::new();
        let mut seen: std::collections::HashSet<(&'static str, String)> =
            std::collections::HashSet::new();
        for w in self
            .codegen_warnings
            .borrow_mut()
            .drain(..)
            .chain(warnings.into_iter())
        {
            if seen.insert((w.rule, w.message.clone())) {
                all_warnings.push(w);
            }
        }
        self.last_validation_warnings = all_warnings;

        Ok(sfc)
    }

    /// Plan 012 Batch A: record a codegen warning through the unified channel.
    /// Usable from `&self` contexts (expression transpilers). Warnings are
    /// surfaced via `last_validation_warnings` after `generate_sfc` finishes.
    fn warn(
        &self,
        rule: &'static str,
        severity: crate::ui_gen::validators::Severity,
        message: impl Into<String>,
    ) {
        let widget = self
            .current_widget
            .clone()
            .unwrap_or_else(|| "?".to_string());
        self.codegen_warnings
            .borrow_mut()
            .push(crate::ui_gen::validators::ValidationWarning::new(
                rule, severity, &widget, message,
            ));
    }

}
impl BackendGenerator for VueGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_sfc(widget)
    }

    fn extension(&self) -> &'static str {
        "vue"
    }
}

impl Default for VueGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Plan 331: Library templates — self-contained per-widget SFC definitions.
// ============================================================================

/// The rendered pieces of a single primitive widget for `VueMode::Library`.
struct WidgetTemplate {
    /// Body inside `<script setup lang="ts">`.
    script: &'static str,
    /// Body inside `<template>`.
    template: &'static str,
    /// Support files beyond `index.ts` (e.g. `variants.ts`), as (name, body).
    extra_support_files: Vec<(&'static str, &'static str)>,
}

/// Convert a kebab/lower widget key (`button`) to PascalCase (`Button`).
fn pascal_case(name: &str) -> String {
    name.split('_')
        .flat_map(|part| part.split('-'))
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Plan 653 D4:Tailwind 类串模板 → 真 inline CSS 翻译器单测。
    /// 语料取自 auto-term app.at 槽位/divider 实际写法。
    #[test]
    fn test_tailwind_class_template_to_inline_style() {
        // 槽位浮层:position/z/几何插值全识别 → 翻译。
        assert_eq!(
            Some(
                "`position:absolute;z-index:10;top:${y1}px;left:${x1}px;width:${w1}px;height:${h1}px`"
                    .to_string()
            ),
            VueGenerator::tailwind_class_template_to_inline_style(
                "`absolute z-10 top-[${y1}px] left-[${x1}px] w-[${w1}px] h-[${h1}px]`"
            )
        );
        // divider:定值 + bg 色。
        assert_eq!(
            Some("`position:absolute;z-index:20;top:${dY1}px;left:8px;width:8px;height:${dH1}px;background:#8899aa`".to_string()),
            VueGenerator::tailwind_class_template_to_inline_style(
                "`absolute z-20 top-[${dY1}px] left-[8px] w-[8px] h-[${dH1}px] bg-[#8899aa]`"
            )
        );
        // CSS 声明串(含冒号)不翻译,回退原语义。
        assert_eq!(
            None,
            VueGenerator::tailwind_class_template_to_inline_style("`color: rgb(1,2,3)`")
        );
        // 未知 token → 整体放弃(不部分翻译)。
        assert_eq!(
            None,
            VueGenerator::tailwind_class_template_to_inline_style(
                "`absolute top-[${y}px] shadow-lg`"
            )
        );
        // 非字面量形态(拼接表达式)不处理。
        assert_eq!(
            None,
            VueGenerator::tailwind_class_template_to_inline_style("('a' + b)")
        );
    }

    /// Plan 653 D4 生成级:widget 里 `style: f"absolute z-10 top-…"` 类
    /// 串必须落成含 `position:absolute;…` 的真 inline `:style`,而非把
    /// 类名串原样塞进 `:style`(浏览器整串忽略,槽位零定位,页面塌成
    /// 2209px 纵向文档)。
    #[test]
    fn test_slot_style_class_string_becomes_real_inline_style() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget TermView {
    model {
        var y1 int = 0
        var x1 int = 0
        var w1 int = 0
        var h1 int = 0
        var visible1 int = 1
    }
    view {
        div {
            style: f"absolute z-10 top-[${.y1}px] left-[${.x1}px] w-[${.w1}px] h-[${.h1}px]"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":style=\"`position:absolute;z-index:10;top:${y1}px;left:${x1}px;width:${w1}px;height:${h1}px`\""),
            "slot style must become real inline CSS in :style:\n{sfc}"
        );
        assert!(
            !sfc.contains(":style=\"`absolute"),
            "raw tailwind class list must not reach :style:\n{sfc}"
        );
    }

    /// Form submit 布线（extract 层 `variant: "submit"` → input onenter）
    /// 在 vue 轨的落地：未声明 onenter 的 input 全部得到
    /// `@keyup.enter="Submit"`；且 `submit` 是行为语义不是视觉变体——
    /// 不得作为 variant 属性下发给 shadcn Button。
    #[test]
    fn test_form_submit_wiring_emits_keyup_enter() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget LoginPage {
    msg { Submit, UsernameChanged, PasswordChanged }
    model {
        var username str = ""
        var password str = ""
    }
    view {
        col {
            input { value: .username, oninput: .UsernameChanged, placeholder: "Enter username" }
            input { value: .password, oninput: .PasswordChanged, password: true, placeholder: "Enter password" }
            button { onclick: .Submit, variant: "submit", text "Login" }
        }
    }
    on {
        .Submit -> {}
        .UsernameChanged -> {}
        .PasswordChanged -> {}
    }
}
"#,
        );
        // IME guard wraps the handler (LAUNCHER-001 R-02): both inputs still
        // bind keyup.enter and invoke Submit when not composing.
        assert_eq!(
            sfc.matches("@keyup.enter=").count(),
            2,
            "both inputs must emit keyup.enter→Submit:\n{sfc}"
        );
        assert_eq!(
            sfc.matches("{ Submit() }").count(),
            2,
            "both onenter guards must invoke Submit():\n{sfc}"
        );
        assert!(
            sfc.contains("isComposing"),
            "onenter must be IME-guarded:\n{sfc}"
        );
        assert!(
            !sfc.contains("variant=\"submit\""),
            "submit is behavioral, not a visual variant — must not reach shadcn Button:\n{sfc}"
        );
    }

    /// Plan 421: code_editor oncursor/oncontextmenu — the scaffold emits
    /// `cursor` with `{line, column}` (1-based) and `contextmenu` with
    /// `{x, y}`. Handlers that declare params get the payload threaded
    /// through (`$event.line, $event.column` / `$event.x, $event.y`); a
    /// single declared param receives the whole `$event`; a param-less
    /// handler stays a plain reference.
    /// Plan 481:text/label 声明 selectable: true → vue 输出显式
    /// `style="user-select: text"`(防应用级 none 吞掉);缺省零改动。
    #[test]
    fn test_text_selectable_emits_user_select() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget SelText {
    view {
        col {
            text "pick me" { selectable: true }
            text "plain" {}
            label "form label" { selectable: true }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("user-select: text"),
            "selectable: true must emit explicit user-select style:
{sfc}"
        );
        // 两个声明位(text+label)都带显式化 → 出现 2 次。
        assert_eq!(
            sfc.matches("user-select: text").count(),
            2,
            "text and label each emit once:
{sfc}"
        );
        // plain text 不声明 → 不出现额外 user-select(仅 2 次)已由上式锁定。
    }

    /// PLAN-493 T5: mentions textarea 发射——backdrop+透明 textarea 兄弟对
    /// （backdrop 在前，v-html 调 __autoMentionHtml(text, mentionNames, cls)；
    /// backdrop 类串自 textarea 类串推导：换 text-foreground、删 caret/
    /// resize/outline/focus token、增背板结构 token；textarea 注入
    /// text-transparent；helper 随 script 恰一次；mentions/mention_class
    /// 不作 attr 透传）。
    #[test]
    fn plan493_vue_mentions_emits_backdrop_pair() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget MentionComposer {
    model {
        var text str = ""
        var mentionNames list = []
    }
    view {
        div class="relative" {
            textarea {
                style: "chats-input absolute inset-0 py-2 px-3 text-[15px] leading-[1.5] text-transparent caret-[hsl(var(--foreground))] outline-none resize-none focus:outline-none"
                value: .text
                mentions: .mentionNames
                mention_class: "text-[hsl(220_90%_56%)] bg-[hsl(220_90%_56%/0.12)] rounded-[3px] px-[0.2rem] font-medium"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("v-html=\"__autoMentionHtml(text, mentionNames, 'text-[hsl(220_90%_56%)]"),
            "backdrop 必须以 v-html 调 helper:\n{sfc}"
        );
        let backdrop_pos = sfc
            .find("v-html=\"__autoMentionHtml")
            .expect("backdrop div");
        let ta_pos = sfc.find("text-transparent ").expect("textarea injected");
        assert!(
            backdrop_pos < ta_pos,
            "backdrop 必须在 textarea 之前(兄弟对)"
        );
        // backdrop 类串推导：结构 token 在、输入面 token 不在。
        let backdrop_line = sfc
            .lines()
            .find(|l| l.contains("v-html=\"__autoMentionHtml"))
            .expect("backdrop div line");
        for present in [
            "text-foreground",
            "pointer-events-none",
            "overflow-hidden",
            "whitespace-pre-wrap",
            "break-words",
            "py-2",
            "text-[15px]",
            "leading-[1.5]",
        ] {
            assert!(
                backdrop_line.contains(present),
                "backdrop 类须含 {present}: {backdrop_line}"
            );
        }
        for absent in [
            "text-transparent",
            "caret-",
            "resize-",
            "focus:",
            "outline-none",
        ] {
            assert!(
                !backdrop_line.contains(absent),
                "backdrop 类不得含 {absent}: {backdrop_line}"
            );
        }
        // helper 随 script 恰一次。
        assert_eq!(sfc.matches("function __autoMentionHtml").count(), 1);
        // r9（musk-073 消费）：token 级小写匹配（扩展字符集扫描 + 名单最长
        // 匹配）+ 引用 token 的 mention-token 语义类。
        assert!(
            sfc.contains("known.has(escaped.slice(i + 1, k).toLowerCase())"),
            "helper 必须 token 级小写匹配（最长匹配）"
        );
        assert!(
            sfc.contains("isTokenChar"),
            "helper 必须含扩展 token 字符集判定"
        );
        assert!(
            sfc.contains("' mention-token'"),
            "引用 token 命中须附加 mention-token 类"
        );
        // mentions/mention_class 不作 attr 透传。
        assert!(!sfc.contains("mentions="));
        assert!(!sfc.contains("mention_class="));
    }

    /// PLAN-493 T5: 无 mentions 的 textarea 零变化——不发射 backdrop、
    /// 不发射 helper、textarea 保持普通形态。
    #[test]
    fn plan493_vue_textarea_without_mentions_unchanged() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget PlainComposer {
    model { var text str = "" }
    view {
        textarea {
            style: "border rounded-md px-3 py-2"
            value: .text
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("__autoMentionHtml"),
            "无 mentions 不得发射 helper:\n{sfc}"
        );
        assert!(
            !sfc.contains("v-html="),
            "无 mentions 不得发射 backdrop:\n{sfc}"
        );
        assert!(
            !sfc.contains("text-transparent"),
            "无 mentions 不得注入透明:\n{sfc}"
        );
        assert!(sfc.contains("<textarea"), "普通 textarea 发射保持");
    }

    /// PLAN-493 T5: backdrop 类串推导单元——musk 形态类串的镜像规则。
    #[test]
    fn plan493_vue_mention_backdrop_class_derivation() {
        let ta_class = "chats-input absolute inset-0 h-full py-2 px-3 block w-full border-none rounded-none bg-transparent text-[15px] leading-[1.5] text-transparent caret-[hsl(var(--foreground))] outline-none resize-none focus:outline-none";
        let got = derive_mention_backdrop_class(ta_class);
        for present in [
            "text-foreground",
            "pointer-events-none",
            "overflow-hidden",
            "whitespace-pre-wrap",
            "break-words",
            "py-2",
            "px-3",
            "text-[15px]",
            "leading-[1.5]",
            "bg-transparent",
        ] {
            assert!(got.contains(present), "须保留/新增 {present}: {got}");
        }
        for absent in [
            "text-transparent",
            "caret-",
            "resize-",
            "outline-none",
            "focus:",
        ] {
            assert!(!got.contains(absent), "须删除 {absent}: {got}");
        }
        // 幂等：推导两次结果一致（结构 token 不重复堆积）。
        assert_eq!(got, derive_mention_backdrop_class(&got));
        // 无 text-transparent（.at 已删）且无其他颜色 token → 保底追加
        // text-foreground（musk 迁移后形态）。
        let got2 =
            derive_mention_backdrop_class("chats-input absolute inset-0 py-2 px-3 text-[15px]");
        assert!(got2.contains("text-foreground"), "保底着色: {got2}");
        // 已有显式颜色 token 时不重复追加。
        let got3 = derive_mention_backdrop_class("p-2 text-red-500 text-sm");
        assert_eq!(
            got3.matches("text-foreground").count(),
            0,
            "已有颜色不追加: {got3}"
        );
        assert!(got3.contains("text-red-500") && got3.contains("text-sm"));
    }

    #[test]
    fn test_code_editor_events_payload_threading() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget EditorDemo {
    model {
        var source str = "fn main() {}"
    }
    view {
        col {
            code_editor (key: "ed", lang: "auto") {
                content: .source
                oncursor: .CursorMoved
                oncontextmenu: .CtxMenu
            }
            code_editor (key: "one") {
                content: .source
                oncursor: .CursorPos
            }
        }
    }
    on {
        .CursorMoved(line, col) -> { }
        .CtxMenu(x, y) -> { }
        .CursorPos(pos) -> { }
    }
}
"##,
        );

        // Two declared params → destructured position args, and the script
        // side declares the matching `(line: any, col: any)` signature.
        assert!(
            sfc.contains("@cursor=\"CursorMoved($event.line, $event.column)\""),
            "cursor payload args:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function CursorMoved(line: any, col: any)"),
            "cursor handler signature:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@contextmenu=\"CtxMenu($event.x, $event.y)\""),
            "contextmenu payload args:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function CtxMenu(x: any, y: any)"),
            "contextmenu handler signature:\n{}",
            sfc
        );
        // One declared param → the whole payload object.
        assert!(
            sfc.contains("@cursor=\"CursorPos($event)\""),
            "single-param payload forwarding:\n{}",
            sfc
        );
    }

    /// Plan 421: a param-less oncursor handler keeps the plain reference —
    /// no payload is forced onto a handler that takes none.
    #[test]
    fn test_code_editor_events_no_params_plain_binding() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget EditorDemo {
    model {
        var source str = "fn main() {}"
    }
    view {
        col {
            code_editor (key: "ed") {
                content: .source
                oncursor: .CursorMoved
            }
        }
    }
    on {
        .CursorMoved -> { }
    }
}
"##,
        );
        assert!(
            sfc.contains("@cursor=\"CursorMoved\""),
            "param-less handler stays plain:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("@cursor=\"CursorMoved("),
            "no payload call for param-less handler:\n{}",
            sfc
        );
    }

    /// Plan musk-022 Phase 3: golden-test helper for the a2vue codegen.
    fn test_a2vue(case: &str) -> Result<(), Box<dyn std::error::Error>> {
        test_a2vue_with(case, false)
    }

    /// PLAN-026 缺陷①: shadcn-mode variant — col/row/grid/center/container 是
    /// shadcn 布局基元，走 `generate_shadcn_attrs`，需用 `new_shadcn()` 测试。
    fn test_a2vue_shadcn(case: &str) -> Result<(), Box<dyn std::error::Error>> {
        test_a2vue_with(case, true)
    }

    fn test_a2vue_with(case: &str, shadcn: bool) -> Result<(), Box<dyn std::error::Error>> {
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = d.join(format!("test/a2vue/{}/input.at", case));
        let src = std::fs::read_to_string(&src_path)
            .map_err(|e| format!("read {}: {}", src_path.display(), e))?;

        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src.as_str()).with_session(session);
        let ast = crate::parse_ui(&src)?;

        let mut widgets = Vec::new();
        for stmt in ast.domain_stmts() {
            if let auto_ui_dialect::UiStmt::WidgetDecl(widget_decl) = stmt {
                let aura_widget = crate::aura::extract_widget_from_decl(widget_decl)?;
                widgets.push(aura_widget);
            }
        }
        if widgets.is_empty() {
            return Err("No widget declarations found in input file".into());
        }

        let mut gen = if shadcn {
            VueGenerator::new_shadcn()
        } else {
            VueGenerator::new()
        };
        let output = gen.generate_sfc(&widgets[0])?;

        let exp_path = d.join(format!("test/a2vue/{}/input.expected.vue", case));
        let expected = if exp_path.is_file() {
            std::fs::read_to_string(&exp_path)?
        } else {
            String::new()
        };

        let output_n = normalize_vue_output(&output);
        let expected_n = normalize_vue_output(&expected);
        if output_n != expected_n {
            let wrong_path = d.join(format!("test/a2vue/{}/input.wrong.vue", case));
            std::fs::write(&wrong_path, &output)?;
            return Err(format!(
                "a2vue mismatch for '{}'. See input.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                case, expected_n, output_n
            ).into());
        }
        Ok(())
    }

    fn normalize_vue_output(s: &str) -> String {
        s.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    #[test]
    fn test_vue_generator_creation() {
        let gen = VueGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    #[test]
    fn test_simple_counter() {
        // Real parse path (plan 012 batch C): widget source → parser → aura
        // extract → generate. No hand-built AST.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Counter {
    msg Msg { Inc, Dec }
    model { var count int = 0 }
    view { col { text "Count: 0" } }
}
"#,
        );

        // Plan 100: Default is now TypeScript, so check for lang="ts"
        assert!(sfc.contains(r#"<script setup lang="ts">"#));
        // plan 015: defineModel is a macro — never imported (TS2440 guard).
        assert!(!sfc.contains("defineModel } from 'vue'"));
        // plan 443: unbound model var stays a plain ref (deep reactivity);
        // defineModel only for parent-bound channels.
        assert!(sfc.contains("const count = ref<number>(0)"));
        assert!(sfc.contains("<template>"));
        assert!(sfc.contains("<style>"));
    }

    #[test]
    fn zzz_probe_icon_size_in_button_child() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ProbeIcon {
    view {
        col {
            sidebar_provider (class: "w-auto min-h-0 flex") {
            col {
                style: "w-80 shrink-0 h-full flex flex-col"
                row {
                    style: "shrink-0 gap-1 px-3 pb-1"
                    button {
                        style: "h-7 w-auto"
                        icon (name: "folder", size: 11)
                        text "work"
                    }
                }
                sidebar_content (style: "px-2 pb-2") {
                    sidebar_group {
                        sidebar_group_label { text "Pinned" }
                        sidebar_group_content {
                            sidebar_menu {
                                sidebar_menu_button {
                                    style: "h-auto py-1.5"
                                    icon (name: "check", size: 13)
                                    text "x"
                                }
                            }
                        }
                    }
                }
            }
            }
        }
    }
}
"#,
        );
        for line in sfc.lines() {
            if line.contains("Folder") || line.contains("Check") {
                eprintln!("[PROBE] {}", line.trim());
            }
        }
    }

    /// PLAN-619 T-02（用户裁决：`.at` 的 `size:` 是权威）：icon 元素带显式
    /// `size:` 时尺寸必须落发射且缺省类 `w-5 h-5` 让位。发射机制以 PLAN-617
    /// 的口径为准（与 VM 端 `with_icon_size` 同一套）：无显式 `w-*`/`h-*`/`size-*`
    /// 类时把 size 折成**内联 style**（`width:Npx;height:Npx`，Tailwind 类表达
    /// 不了任意值，且内联样式的特异性还压过 shadcn 资产的 `[&>svg]:size-*`）；
    /// 有显式尺寸类时类优先；缺省（无 size:、无显式类）回落 w-5 h-5（=20px，
    /// 与 VM 端 DEFAULT_ICON_PX 同源）。PLAN-619 最初的 `:size` 绑定发射在
    /// 合并 PLAN-617 时被此机制取代（机制更强：块式 `style:` 与动态 class
    /// 也覆盖）。
    #[test]
    fn plan619_icon_size_prop_emits_bound_size() {
        let sized = gen_sfc_from_widget_src(
            r#"
widget IconSized {
    view {
        col {
            icon (name: "search", size: 14, style: "text-muted-foreground")
        }
    }
}
"#,
        );
        assert!(
            sized.contains("style=\"width:14px;height:14px\""),
            "显式 size 必须折成内联尺寸样式:
{sized}"
        );
        assert!(
            !sized.contains("w-5 h-5"),
            "显式 size 时必须撤掉缺省 w-5 h-5（同权重 Tailwind 类会与内联尺寸打架）:
{sized}"
        );
        assert!(
            sized.contains("text-muted-foreground"),
            "icon 的用户 style 类应并存:
{sized}"
        );

        let defaulted = gen_sfc_from_widget_src(
            r#"
widget IconDefault {
    view {
        col {
            icon (name: "search")
        }
    }
}
"#,
        );
        assert!(
            defaulted.contains("style=\"width:") == false,
            "无 size 声明时不应发射内联尺寸样式:
{defaulted}"
        );
        assert!(
            defaulted.contains("w-5 h-5"),
            "无 size 声明时回落标签缺省 w-5 h-5（=20px，与 VM 端缺省同源）:
{defaulted}"
        );
    }

    /// PLAN-621（T-04）: icon state 契约 Web 臂——字面量 on/off、显式
    /// text-* 优先、bool 绑定三元、未声明零改动、48px 细线基档。
    #[test]
    fn plan621_icon_state_literal_on_off() {
        let on = gen_sfc_from_widget_src(
            r#"
widget IconStateOn {
    view {
        col {
            icon (name: "bell", state: "on")
        }
    }
}
"#,
        );
        assert!(
            on.contains("text-primary"),
            "state:on 应发射 text-primary:\n{on}"
        );
        assert!(
            on.contains(":stroke-width=\"2.5\""),
            "state:on 应发射描边加重 2.5（默认 20px 盒基档 2.0+0.5）:\n{on}"
        );
        assert!(
            !on.contains("text-muted-foreground"),
            "state:on 不应带 off 色:\n{on}"
        );

        let off = gen_sfc_from_widget_src(
            r#"
widget IconStateOff {
    view {
        col {
            icon (name: "bell", state: "off")
        }
    }
}
"#,
        );
        assert!(
            off.contains("text-muted-foreground"),
            "state:off 应发射 muted dim:\n{off}"
        );
        assert!(!off.contains(":stroke-width"), "state:off 不应加重:\n{off}");
    }

    /// 显式 text-* 类在场：颜色归作者，描边加重照常（与 VM 同规则）。
    #[test]
    fn plan621_icon_state_explicit_text_class_wins() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget IconStateExplicit {
    view {
        col {
            icon (name: "bell", state: "on", style: "text-red-500")
        }
    }
}
"#,
        );
        assert!(sfc.contains("text-red-500"), "作者色必须保留:\n{sfc}");
        assert!(
            !sfc.contains("text-primary"),
            "显式 text-* 在场时不得注入 state 色:\n{sfc}"
        );
        assert!(
            sfc.contains(":stroke-width=\"2.5\""),
            "描边加重应照常生效:\n{sfc}"
        );
    }

    /// bool/变量绑定 → `:class` 三元 + `:stroke-width` 三元（false →
    /// undefined → Vue 移除 attr）。
    #[test]
    fn plan621_icon_state_binding_ternary() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget IconStateBound {
    model { liked bool = false }
    view {
        col {
            icon (name: "heart", state: .liked)
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("'text-primary' : 'text-muted-foreground'"),
            "绑定应发射三元 class:\n{sfc}"
        );
        assert!(
            sfc.contains(":stroke-width=") && sfc.contains("undefined"),
            "绑定应发射三元描边（false → undefined）:\n{sfc}"
        );
    }

    /// 未声明 state → 零改动（G3）；48px 盒加重走细线基档 1.5+0.5=2.0。
    #[test]
    fn plan621_icon_state_undeclared_zero_diff_and_48px() {
        let none = gen_sfc_from_widget_src(
            r#"
widget IconStateNone {
    view {
        col {
            icon (name: "search")
        }
    }
}
"#,
        );
        assert!(
            !none.contains(":stroke-width")
                && !none.contains("text-primary")
                && !none.contains("text-muted-foreground"),
            "未声明 state 时输出必须与既有形态逐字节一致:\n{none}"
        );

        let large = gen_sfc_from_widget_src(
            r#"
widget IconStateLarge {
    view {
        col {
            icon (name: "zap", size: 48, state: "on")
        }
    }
}
"#,
        );
        assert!(
            large.contains(":stroke-width=\"2\""),
            "48px 盒加重应为 1.5+0.5=2.0（细线基档，与 VM 同式）:\n{large}"
        );
    }

    /// Widget-level native CSS (`style { ... }` → `AuraWidget.style_css`) is
    /// emitted verbatim into a dedicated `<style scoped>` block.
    #[test]
    fn test_widget_style_css_scoped_passthrough() {
        // Real parse path (plan 012 batch C): the widget-level `style { ... }`
        // block is captured verbatim by the lexer into `AuraWidget.style_css`.
        let css = "\n.autodown-editor {\n  --ad-border: #333;\n}\n.autodown-editor:hover {\n  border-color: var(--ad-border);\n}\n@media (max-width: 768px) {\n  .autodown-editor {\n    font-size: 12px;\n  }\n}\n";
        let src = format!(
            "widget Styled {{\n    view {{ col {{ text \"hi\" }} }}\n    style {{{css}}}\n}}"
        );
        let sfc = gen_sfc_from_widget_src(&src);

        assert!(sfc.contains("<style scoped>"), "sfc:\n{}", sfc);
        // The CSS body is byte-for-byte identical inside the scoped block.
        assert!(
            sfc.contains(css),
            "scoped css not verbatim in sfc:\n{}",
            sfc
        );
        assert!(sfc.contains(".autodown-editor:hover"));
        assert!(sfc.contains("@media (max-width: 768px) {"));
        // The plain generated <style> block is still present.
        assert!(sfc.contains("<style>"));
    }

    /// Plan 029 T13: fn 模块体内含 await（Http.get 双 await 展开）时，
    /// 生成的 fn 必须标 async 且省略返回注解；纯同步 fn 保持原样。
    #[test]
    fn test_fn_module_async_when_await() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
fn sync_fn(x int) int { return x + 1 }

fn fetch_things() []Value {
    try {
        let data = Http.get("/api/things")
        return data.things ?? []
    } catch {
        return []
    }
}
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("fn module must parse");
        let fns: Vec<crate::aura::AuraModuleFn> = ast
            .core_stmts()
            .filter_map(|s| match s {
                auto_lang_core::ast::Stmt::Fn(f) => crate::aura::extract_module_fn(f),
                _ => None,
            })
            .collect();
        let code = VueGenerator::generate_fn_module(&fns);
        assert!(
            code.contains("export function sync_fn(x: any)"),
            "sync fn unchanged:\n{}",
            code
        );
        assert!(
            code.contains("export async function fetch_things()"),
            "awaiting fn must be async:\n{}",
            code
        );
        assert!(
            !code.contains("fetch_things(): "),
            "async fn drops ret anno:\n{}",
            code
        );
    }

    /// Plan 424 T1: ports 符号转发——fn 模块混写三种 `use.web` kind 时,
    /// fn 走 import(现有 wrapper 路径),component/composable 走 ES
    /// re-export(specifier 复用 ext_import_specifier:npm 原样、
    /// platform: → `@/platform/x.vue`、本地 → `@/ext/...`)。
    #[test]
    fn test_fn_module_reexport_component_composable() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
use.web format_rel from "./time_fmt.ts"
use.web component MessageSquare, ListTodo from "lucide-vue-next"
use.web component Markdown from "platform:markdown"
use.web component Deck from "src/platform/deck.vue"
use.web composable useT from "./i18n.ts"

fn rel_time(ts int) str {
    return format_rel(ts)
}
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("port module must parse");
        let fns: Vec<crate::aura::AuraModuleFn> = ast
            .core_stmts()
            .filter_map(|s| match s {
                auto_lang_core::ast::Stmt::Fn(f) => crate::aura::extract_module_fn(f),
                _ => None,
            })
            .collect();
        let web_imports: Vec<auto_ui_dialect::ast::ExtImport> = ast
            .domain_stmts()
            .filter_map(|s| match s {
                auto_ui_dialect::UiStmt::UseWeb(entries) => Some(entries.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        let code = VueGenerator::generate_fn_module_full(&fns, &web_imports);
        // fn kind: import(wrapper 路径不变)+ export(纯转发)
        assert!(
            code.contains("import { format_rel } from '@/ext/time_fmt'"),
            "fn kind still imports:
{}",
            code
        );
        assert!(
            code.contains("export { format_rel } from '@/ext/time_fmt'"),
            "fn kind also re-exports:
{}",
            code
        );
        // component kind: npm specifier 原样 re-export
        assert!(
            code.contains("export { MessageSquare, ListTodo } from 'lucide-vue-next'"),
            "npm component re-export:
{}",
            code
        );
        // component kind: platform: → @/platform/x.vue(default export → default as)
        assert!(
            code.contains("export { default as Markdown } from '@/platform/markdown.vue'"),
            "platform component re-export:
{}",
            code
        );
        // component kind: 本地 .vue 同为 default export(.vue 扩展名保留)
        assert!(
            code.contains("export { default as Deck } from '@/ext/src/platform/deck.vue'"),
            "local .vue component re-export:
{}",
            code
        );
        // composable kind: 本地 → @/ext/...
        assert!(
            code.contains("export { useT } from '@/ext/i18n'"),
            "composable re-export:
{}",
            code
        );
        // wrapper fn 照常导出
        assert!(
            code.contains("export function rel_time(ts: any)"),
            "wrapper fn still exported:
{}",
            code
        );
    }

    /// Plan 424 T3: 纯转发端口——零 wrapper fn,仅 use.web 条目。生成侧
    /// 照常发射 re-export(fns 为空不报错;auto-man 侧同步放宽)。
    #[test]
    fn test_fn_module_pure_forwarding_port() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
use.web component MessageSquare, ListTodo from "lucide-vue-next"
use.web composable useT from "./i18n.ts"
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("pure port must parse");
        let fns: Vec<crate::aura::AuraModuleFn> = ast
            .core_stmts()
            .filter_map(|s| match s {
                auto_lang_core::ast::Stmt::Fn(f) => crate::aura::extract_module_fn(f),
                _ => None,
            })
            .collect();
        let web_imports: Vec<auto_ui_dialect::ast::ExtImport> = ast
            .domain_stmts()
            .filter_map(|s| match s {
                auto_ui_dialect::UiStmt::UseWeb(entries) => Some(entries.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        assert!(fns.is_empty(), "pure port has no wrappers");
        let code = VueGenerator::generate_fn_module_full(&fns, &web_imports);
        // 收口补强:零 wrapper 时无 import 行(不再依赖 noUnusedLocals: false)。
        assert!(
            !code.contains("import {"),
            "pure forwarding port emits no unused import:\n{}",
            code
        );
        assert!(
            code.contains("export { MessageSquare, ListTodo } from 'lucide-vue-next'"),
            "component forwarding:
{}",
            code
        );
        assert!(
            code.contains("export { useT } from '@/ext/i18n'"),
            "composable forwarding:
{}",
            code
        );
    }

    /// Plan 424 MVP-4: 调用方门控——`use.web component/composable X from
    /// "<port>.at"` 与直接引用同路径:named import 指向端口模块
    /// (`@/ext/<port>`),tag 注册与 composable 自动调用机制不变。
    /// 零 parser 改动,此测试锁定该不变式。(widget use 块与文件级
    /// use.web 最终都汇入 widget.ext_imports,走同一 register_ext_imports。)
    #[test]
    fn test_port_referenced_component_composable_caller_side() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Toolbar {
    use {
        component: MessageSquare from "ports/icons.web.at"
        composable: useT from "ports/i18n.web.at"
    }
    view {
        col {
            MessageSquare {
                class: "w-4 h-4"
            }
            text (text: t("greeting"))
        }
    }
}
"#,
        );
        // component: named import 指向端口模块(.at 扩展名剥掉)
        assert!(
            sfc.contains("import { MessageSquare } from '@/ext/ports/icons.web'"),
            "component via port:
{}",
            sfc
        );
        // composable: 同 specifier 规则 + 自动调用 const 绑定
        assert!(
            sfc.contains("import { useT } from '@/ext/ports/i18n.web'"),
            "composable via port:
{}",
            sfc
        );
        assert!(sfc.contains("const t = useT()"), "auto-call kept:\n{}", sfc);
        // tag 注册不变:模板内实例化成功
        assert!(sfc.contains("<MessageSquare"), "tag registered:\n{}", sfc);
    }

    /// Plan 029 T17: dom 内建模块转译（主题/快捷键/剪贴板/外链最小 DOM 面）。
    #[test]
    fn test_fn_module_dom_builtins() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
fn apply_theme(dark bool) {
    dom.set_dark(dark)
}

fn system_dark() bool {
    return dom.prefers_dark()
}

fn brand(name str, val str) {
    dom.set_css_var(name, val)
}

fn focus_search() {
    dom.focus_first(".chats-search")
}

fn open_link(url str) {
    dom.open_url(url)
}

fn copy_id(id str) {
    dom.copy_text(id)
}

fn new_session() {
    dom.click_first(".sidebar-new-btn")
}
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("dom fn module must parse");
        let fns: Vec<crate::aura::AuraModuleFn> = ast
            .core_stmts()
            .filter_map(|s| match s {
                auto_lang_core::ast::Stmt::Fn(f) => crate::aura::extract_module_fn(f),
                _ => None,
            })
            .collect();
        let code = VueGenerator::generate_fn_module(&fns);
        assert!(
            code.contains("classList)['add' : 'remove'])('dark')")
                || code.contains("'add' : 'remove'"),
            "set_dark:
{}",
            code
        );
        assert!(
            code.contains("matchMedia('(prefers-color-scheme: dark)').matches"),
            "{}",
            code
        );
        assert!(
            code.contains("document.documentElement.style.setProperty(name, val)"),
            "{}",
            code
        );
        assert!(
            code.contains(
                "(document.querySelector('.chats-search') as HTMLElement | null)?.focus()"
            ),
            "{}",
            code
        );
        assert!(code.contains("window.open(url, '_blank')"), "{}", code);
        assert!(
            code.contains("navigator.clipboard.writeText(id)"),
            "{}",
            code
        );
        assert!(
            code.contains(
                "(document.querySelector('.sidebar-new-btn') as HTMLElement | null)?.click()"
            ),
            "{}",
            code
        );
    }

    /// Template ref escape hatch: a `ref` prop on a view element emits a
    /// static `ref="menuEl"` template attribute plus a
    /// `const menuEl = ref<HTMLElement | null>(null)` script declaration,
    /// and pulls in the `ref` import even without state vars.
    #[test]
    fn test_template_ref_declaration() {
        // Real parse path (plan 012 batch C).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Menu {
    view { col(ref: "menuEl") { text "hi" } }
}
"#,
        );

        assert!(sfc.contains("import { ref } from 'vue'"), "sfc:\n{}", sfc);
        assert!(
            sfc.contains("const menuEl = ref<HTMLElement | null>(null)"),
            "sfc:\n{}",
            sfc
        );
        assert!(sfc.contains("ref=\"menuEl\""), "sfc:\n{}", sfc);
    }

    /// Without a style block, no `<style scoped>` is emitted.
    #[test]
    fn test_no_widget_style_css_no_scoped_block() {
        // Real parse path (plan 012 batch C).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Plain {
    view { col { text "hi" } }
}
"#,
        );
        assert!(!sfc.contains("<style scoped>"), "sfc:\n{}", sfc);
    }

    /// Plan 413/421: code_editor renders the scaffolded CodeMirror shell with
    /// the full prop contract (model value, lang, wrap, search, line numbers,
    /// highlight current line, tab width, font size, vi) and the
    /// default-style component import.
    /// (Pre-existing quirk fixed in 421: a stray stacked `#[test]` made this
    /// fn run TWICE while the Plan 354 AutoDownEditor test below it lost its
    /// attribute and never ran.)
    #[test]
    fn test_code_editor_rendering() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget EditorDemo {
    msg Msg { SourceChanged }
    model {
        var source str = "fn main() {}"
        var query str = ""
    }
    view {
        col {
            code_editor (lang: "rust", wrap: true) {
                content: .source
                oninput: .SourceChanged
            }
            code_editor (key: "searchable", lang: "auto") {
                content: .source
                search: .query
            }
            code_editor (key: "full", lang: "auto", line_numbers: false, highlight_current_line: false, tab_width: 2, font_size: 16, vi: true) {
                content: .source
            }
        }
    }
}
"##,
        );

        assert!(
            sfc.contains("<CodeEditor"),
            "component tag:
{}",
            sfc
        );
        assert!(
            sfc.contains("import CodeEditor from '@/components/CodeEditor.vue'"),
            "default-style import:
{}",
            sfc
        );
        assert!(
            sfc.contains("lang=\"rust\""),
            "static lang:
{}",
            sfc
        );
        assert!(
            sfc.contains(":wrap=\"true\""),
            "wrap prop:
{}",
            sfc
        );
        assert!(
            sfc.contains("v-model=\"source\"") || sfc.contains(":modelValue=\"source\""),
            "value binding:
{}",
            sfc
        );
        assert!(
            sfc.contains(":search=\"query\""),
            "search binding:
{}",
            sfc
        );
        // Plan 421: the five props all flow through the shell contract.
        assert!(
            sfc.contains(":line-numbers=\"false\""),
            "line-numbers off:
{}",
            sfc
        );
        assert!(
            sfc.contains(":highlight-current-line=\"false\""),
            "highlight-current-line off:
{}",
            sfc
        );
        assert!(
            sfc.contains(":tab-width=\"2\""),
            "tab-width:
{}",
            sfc
        );
        assert!(
            sfc.contains(":font-size=\"16\""),
            "font-size:
{}",
            sfc
        );
        // vi binding flows (the vue shell accepts-but-ignores it; iced consumes).
        assert!(
            sfc.contains(":vi=\"true\""),
            "vi binding:
{}",
            sfc
        );
        // Defaults for the bool props stay explicit bindings.
        assert!(
            sfc.contains(":line-numbers=\"true\""),
            "line-numbers default:
{}",
            sfc
        );
        assert!(
            sfc.contains("@update:modelValue"),
            "oninput event mapping:
{}",
            sfc
        );
    }

    /// Plan 354 Phase C: AutoDownEditor renders as a Vue component with
    /// camelCase prop bindings, @autodown/editor named import, and events
    /// mapped to @update / @save / @cancel.
    #[test]
    fn test_autodown_editor_rendering() {
        // Real parse path (plan 012 batch C): the previous version hand-built
        // `Expr::Ident(".body")` — a shape the real parser never produces
        // (dot_item yields Dot(Ident("self"), name)).
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget NoteEditor {
    msg Msg { BodyChanged }
    model { var body str = "# Welcome" }
    view {
        col {
            autodown_editor {
                content: .body
                onupdate: .BodyChanged
            }
        }
    }
}
"##,
        );

        // Renders as PascalCase component, not <div> or <autodown_editor>.
        assert!(
            sfc.contains("<AutoDownEditor"),
            "tag is AutoDownEditor:\n{}",
            sfc
        );
        // content bound to the body state ref.
        assert!(
            sfc.contains(":content=\"body\""),
            "content bound to body:\n{}",
            sfc
        );
        // Defaults: can-edit and show-actions default to true.
        assert!(
            sfc.contains(":can-edit=\"true\""),
            "can_edit defaults true:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":show-actions=\"true\""),
            "show_actions defaults true:\n{}",
            sfc
        );
        // Event: onupdate → @update with the dot-handler name.
        assert!(
            sfc.contains("@update=\"BodyChanged\""),
            "onupdate → @update:\n{}",
            sfc
        );
        // Named import from @autodown/editor.
        assert!(
            sfc.contains("import { AutoDownEditor } from '@autodown/engine'"),
            "named import from @autodown/editor:\n{}",
            sfc
        );
    }

    /// Plan 040: autodown 只读臂 streaming 族 props 发射 + editor placeholder。
    /// - 显式 streaming 优先，final 反相发射为 :streaming="!final"；
    /// - placeholder_block_id/placeholder_height snake→kebab 绑定；
    /// - scroll_sync 缺省 true；编辑臂 placeholder 字面发射。
    #[test]
    fn test_autodown_streaming_props_and_editor_placeholder() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget DocStream {
    model {
        var body str = ""
        var pid str = "ghost"
        var ph f64 = 96.0
        var done bool = false
    }
    view {
        col {
            autodown {
                content: .body
                streaming: .done
                placeholder_block_id: .pid
                placeholder_height: .ph
            }
            autodown {
                content: .body
                final: false
            }
            autodown_editor {
                content: .body
                placeholder: "Start typing..."
            }
        }
    }
}
"#,
        );
        // 显式 streaming 绑定优先。
        assert!(
            sfc.contains(":streaming=\"done\""),
            "explicit streaming:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":placeholder-block-id=\"pid\""),
            "block id:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":placeholder-height=\"ph\""),
            "height:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":scroll-sync=\"true\""),
            "scroll sync default:\n{}",
            sfc
        );
        // final: false → :streaming="true"（反相）。
        assert!(
            sfc.contains(":streaming=\"true\""),
            "final inversion:\n{}",
            sfc
        );
        // 编辑臂 placeholder：解析器字符串字面量走绑定分支（:placeholder=
        // "'...'"，与 content 字面同约定）。
        assert!(
            sfc.contains(":placeholder=\"'Start typing...'\""),
            "editor placeholder:\n{}",
            sfc
        );
        // 换绑超集组件（legacy markdown 别名路径由 002 golden 锁定）。
        // 多组件共用 import 源时合并为一条 import 语句。
        assert!(
            sfc.contains("StreamingRenderer } from '@autodown/engine'")
                || sfc.contains("import { StreamingRenderer } from '@autodown/engine'"),
            "import:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<StreamingRenderer"),
            "component tag:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("MarkdownRender"),
            "MarkdownRender fully rebound:\n{}",
            sfc
        );
    }

    /// PLAN-051 T5：双轨组件主题声明入口——`dark_mode`/`accent` props 经
    /// autodown_editor/autodown 两臂发射为 `:dark-mode`/`:accent`（绑定）；
    /// 缺省 `:dark-mode="false"`（引擎默认档零差异）+ 无 accent 发射。
    /// 排查路径锚：demo app.at 声明 → 本发射 → 引擎 darkMode/accent props。
    #[test]
    fn test_autodown_theme_props_emit() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget Themed {
    model {
        var body str = "# Welcome"
        var dark_mode bool = false
        var accent_color str = "indigo"
    }
    view {
        col {
            autodown_editor {
                content: .body
                dark_mode: .dark_mode
                accent: .accent_color
            }
            autodown {
                content: .body
                dark_mode: .dark_mode
                accent: .accent_color
            }
        }
    }
}
"##,
        );

        assert_eq!(
            sfc.matches(":dark-mode=\"dark_mode\"").count(),
            2,
            "editor + renderer arms each emit the bound dark_mode:\n{}",
            sfc
        );
        assert_eq!(
            sfc.matches(":accent=\"accent_color\"").count(),
            2,
            "both arms carry the bound accent:\n{}",
            sfc
        );
        assert!(!sfc.contains("R013"), "no warnings:\n{}", sfc);
    }

    #[test]
    fn test_autodown_theme_props_defaults() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget Plain {
    model { var body str = "# Welcome" }
    view {
        col {
            autodown_editor { content: .body }
            autodown { content: .body }
        }
    }
}
"##,
        );

        assert_eq!(
            sfc.matches(":dark-mode=\"false\"").count(),
            2,
            "default light档 on both arms:\n{}",
            sfc
        );
        assert!(
            !sfc.contains(":accent"),
            "no accent attr when omitted:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("accent=\""),
            "no literal accent either:\n{}",
            sfc
        );
    }

    #[test]
    fn test_autodown_editor_props_camelcase_and_events() {
        // Real parse path (plan 012 batch C).
        // Verify bool literals map through and onsave/oncancel events resolve.
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget NoteEditor {
    msg Msg { Save, Cancel }
    model { var note_body str = "" }
    view {
        col {
            autodown_editor {
                content: .note_body
                can_edit: false
                show_actions: true
                onsave: .Save
                oncancel: .Cancel
            }
        }
    }
}
"#,
        );

        // Field-access content binding: .note_body → note_body.
        assert!(
            sfc.contains(":content=\"note_body\""),
            "field access content:\n{}",
            sfc
        );
        // Bool literals propagate.
        assert!(
            sfc.contains(":can-edit=\"false\""),
            "can_edit false:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":show-actions=\"true\""),
            "show_actions true:\n{}",
            sfc
        );
        // onsave/oncancel → @save/@cancel.
        assert!(sfc.contains("@save=\"Save\""), "onsave → @save:\n{}", sfc);
        assert!(
            sfc.contains("@cancel=\"Cancel\""),
            "oncancel → @cancel:\n{}",
            sfc
        );
    }

    /// Plan 043 M5 B-1/B-2: `on_*: msg` callback props are typed from the msg
    /// variant payload (Pick(str) → (arg0: string) => void, Stop → () => void),
    /// and custom types nested in containers (List<Block>) are imported from
    /// api.ts as type-only imports.
    #[test]
    fn test_msg_prop_signature_and_custom_type_import() {
        // B-1/B-2 (as amended by R4): `on_*: msg` callback props with a
        // matching msg variant are delivered via the EMIT — they are dropped
        // from defineProps (`on_pick`/`on_stop`), and the payload-aware
        // signature moves to defineEmits (`Pick: [string]`, `Stop: []`).
        // Container-nested custom types still import from api.ts.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Child(blocks: []Block, on_pick: msg, on_stop: msg) {
    msg Msg { Pick(str), Stop }
    view {
        col {
            button "pick" {
                onclick: .Pick("x")
            }
            button "stop" {
                onclick: .Stop
            }
        }
    }
    on {
        .Pick(s) -> { }
        .Stop -> { }
    }
}
"#,
        );

        // R4: emitted-callback props dropped from defineProps.
        assert!(
            !sfc.contains("on_pick"),
            "on_pick dropped from defineProps:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("on_stop"),
            "on_stop dropped from defineProps:\n{}",
            sfc
        );
        // B-1 payload signature now lives on the emit.
        assert!(
            sfc.contains("Pick: [string]"),
            "Pick emit payload:\n{}",
            sfc
        );
        assert!(sfc.contains("Stop: []"), "Stop emit no payload:\n{}", sfc);
        // Standalone parse can't resolve `Block`, so the container type is
        // `any[]` here; the Block import is covered by
        // test_custom_type_import_in_define_props below.
        assert!(sfc.contains("blocks: any[]"), "blocks type:\n{}", sfc);
        assert!(
            !sfc.contains("import type { msg }"),
            "msg must not be imported:\n{}",
            sfc
        );
    }

    #[test]
    fn test_custom_type_import_in_define_props() {
        // B-2: a container-nested custom type (List<Block>) must still import
        // from api.ts even after R4 drops emitted-callback props.
        //
        // KEEP hand-built (plan 012 batch C): this test injects a fully
        // RESOLVED `Type::User(Block)` — a state the standalone parse path
        // cannot produce (type resolution needs the project type_store;
        // parsing `widget Child(blocks: []Block)` alone yields `any[]`, which
        // is what test_msg_prop_signature_and_custom_type_import asserts).
        // The `blocks: Block[]` assertion below locks the post-resolution
        // behavior, so the struct literal stays.
        use auto_lang_core::ast::{Type, TypeDecl, TypeDeclKind};
        let user = |name: &str| {
            Type::User(TypeDecl {
                consts: Vec::new(),
                name: name.into(),
                kind: TypeDeclKind::UserType,
                parent: None,
                has: Vec::new(),
                specs: Vec::new(),
                spec_impls: Vec::new(),
                generic_params: Vec::new(),
                members: Vec::new(),
                delegations: Vec::new(),
                methods: Vec::new(),
                attrs: vec![],
                impl_attrs: vec![],
                doc: None,
                is_pub: false,
            })
        };
        let widget = AuraWidget {
            design: Vec::new(),
            named_views: Vec::new(),
            actions: None,
            name: "Child".to_string(),
            state_vars: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![AuraProp {
                name: "blocks".to_string(),
                type_info: Type::List(Box::new(user("Block"))),
                default: None,
            }],
            computed: vec![],
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
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();
        assert!(sfc.contains("blocks: Block[]"), "blocks type:\n{}", sfc);
        assert!(
            sfc.contains("import type { Block } from '@/lib/api'"),
            "Block import:\n{}",
            sfc
        );
    }

    #[test]
    fn test_expr_to_js() {
        let gen = VueGenerator::new();

        assert_eq!(
            gen.expr_to_js(&auto_lang_core::ast::Expr::Int(42)).unwrap(),
            "42"
        );
        assert_eq!(
            gen.expr_to_js(&auto_lang_core::ast::Expr::Bool(true))
                .unwrap(),
            "true"
        );
        assert_eq!(
            gen.expr_to_js(&auto_lang_core::ast::Expr::Str("hello".into()))
                .unwrap(),
            "'hello'"
        );
    }

    /// PLAN-604 T08 (KD-VM4/AC-06): `.as(int)` 的 vue 侧降级必须是
    /// `Math.trunc`——VM TYPE_CAST_I32 为 Rust `f as i32` 截断
    /// (engine.rs),双端一致。整型族全覆盖;浮点目标直通。
    #[test]
    fn test_as_cast_int_lowers_to_math_trunc() {
        let gen = VueGenerator::new();
        let cast = |t: auto_lang_core::ast::Type| auto_lang_core::ast::Expr::Cast {
            expr: Box::new(auto_lang_core::ast::Expr::Float(3.7, "".into())),
            target_type: t,
        };
        for t in [
            auto_lang_core::ast::Type::Int,
            auto_lang_core::ast::Type::I64,
            auto_lang_core::ast::Type::Uint,
        ] {
            assert_eq!(
                gen.expr_to_js(&cast(t.clone())).unwrap(),
                "Math.trunc(3.7)",
                "as({t}) 必须降级 Math.trunc"
            );
            assert_eq!(
                gen.expr_to_vue_bound_value(&cast(t.clone())).unwrap(),
                "Math.trunc(3.7)",
                "as({t}) 绑定位必须降级 Math.trunc"
            );
        }
        // 浮点目标:JS 原生 f64,直通不截断。
        assert_eq!(
            gen.expr_to_js(&cast(auto_lang_core::ast::Type::Double))
                .unwrap(),
            "3.7"
        );
        // 二元内层保持括号:Math.trunc((a / b))。
        let bin = auto_lang_core::ast::Expr::Cast {
            expr: Box::new(auto_lang_core::ast::Expr::Bina(
                Box::new(auto_lang_core::ast::Expr::Ident("a".into())),
                auto_val::Op::Div,
                Box::new(auto_lang_core::ast::Expr::Ident("b".into())),
            )),
            target_type: auto_lang_core::ast::Type::Int,
        };
        assert_eq!(gen.expr_to_js(&bin).unwrap(), "Math.trunc((a / b))");
    }

    /// 图标尺寸口径的**双端一致性锚点**（Vue 侧）。
    ///
    /// 规则：显式 `w-*`/`h-*` 类 > `size` prop > 默认 20px（`w-5 h-5`）。
    /// 与 VM 侧 `aura_view_builder::with_icon_size` 同源；VM 侧的对应断言在
    /// `musk_vm_track_tests` 之外由本测试的兄弟（`icon_size_matches_vm_rule`
    /// 的 VM 版）覆盖。
    ///
    /// 背景：此前 Vue 端**无条件**注入 `w-5 h-5`，Tailwind 同权重靠样式表顺序
    /// 定胜负 ⇒ 用户写的 `class: "h-3 w-3"` 被静默盖成 20px，而 VM 端老老实实
    /// 按 12px 画 —— 同一个 `.at` 两端不一样大。
    #[test]
    fn test_icon_size_precedence() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Icons {
    model { var dummy bool = false }
    view {
        col {
            icon (name: "play")
            icon (name: "pause", size: 12)
            icon (name: "volume-2", class: "h-3 w-3 text-muted-foreground")
        }
    }
}
"##,
        );
        // ① 都没有 → 默认 20px（w-5 h-5）
        assert!(
            sfc.contains("<Play class=\"w-5 h-5\" />"),
            "默认尺寸应为 w-5 h-5：\n{sfc}"
        );
        // ② 只有 size → px 内联样式，且**不得**再带默认类
        assert!(
            sfc.contains("<Pause style=\"width:12px;height:12px\" />"),
            "size 应落成 px 内联尺寸：\n{sfc}"
        );
        assert!(
            !sfc.contains("<Pause class=\"w-5 h-5\""),
            "size 生效时不得再注入默认类：\n{sfc}"
        );
        // ③ 显式尺寸类 → 原样，不加默认、不加 style（否则又会静默盖掉用户意图）
        assert!(
            sfc.contains("<Volume2 class=\"h-3 w-3 text-muted-foreground\" />"),
            "显式尺寸类应原样输出：\n{sfc}"
        );
    }

    /// 可拖拽进度条（`progress` + `onseek`）的生成锚点：
    /// 外层指针三包装 + 内层仍是 shadcn Progress；**未声明 onseek 时**输出
    /// 与改动前一致（兼容约束）。
    #[test]
    fn test_progress_onseek_wrapper() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget Bar {
    msg { SeekFraction(float) }
    model { var pct float = 25.0 }
    on { .SeekFraction(f) -> { .pct = f } }
    view {
        col {
            progress (value: .pct, max: 100.0, class: "h-2 w-full", onseek: .SeekFraction($0))
        }
    }
}
"##,
        );
        assert!(
            sfc.contains("@pointerdown=\"__seekDown_0($event)\""),
            "按下包装:
{sfc}"
        );
        assert!(
            sfc.contains("@pointermove=\"__seekMove_0($event)\""),
            "拖动包装:
{sfc}"
        );
        assert!(
            sfc.contains("@pointerup=\"__seekUp_0($event)\""),
            "抬起包装:
{sfc}"
        );
        assert!(
            sfc.contains("function __seekFrac_0(e: PointerEvent, el: HTMLElement): number"),
            "比例换算:
{sfc}"
        );
        assert!(
            sfc.contains("if (!__seekDrag_0) return"),
            "悬停不 scrub 的按下门控:
{sfc}"
        );
        assert!(
            sfc.contains("SeekFraction(__seekFrac_0(e, el))"),
            "交比例而非像素:
{sfc}"
        );
        assert!(
            sfc.contains("r.width"),
            "比例用条自身宽度:
{sfc}"
        );
        // 内层仍是普通 Progress 组件
        assert!(
            sfc.contains("<Progress :model-value="),
            "内层 Progress 绑定:
{sfc}"
        );

        // 兼容性：不带 onseek 的 progress 不产生任何包装
        let plain = gen_sfc_from_widget_src_shadcn(
            r##"
widget Plain {
    model { var pct float = 10.0 }
    view { progress (value: .pct, max: 100.0) }
}
"##,
        );
        assert!(
            !plain.contains("__seekDown_"),
            "未声明 onseek 不得多出指针包装:
{plain}"
        );
        assert!(
            !plain.contains("__seekFrac_"),
            "未声明 onseek 不得多出比例函数:
{plain}"
        );
    }

    #[test]
    fn test_map_tag_native_html_passthrough() {
        // Plan 041a①: 原生语义元素不再坍缩为 div(native_select/native_button
        // 逃生名 + pre/code/ol/ul/li/option 等直通)。
        let mut gen = VueGenerator::new();
        assert_eq!(gen.map_tag("native_select", false), "select");
        assert_eq!(gen.map_tag("native_button", false), "button");
        assert_eq!(gen.map_tag("pre", false), "pre");
        assert_eq!(gen.map_tag("code", false), "code");
        assert_eq!(gen.map_tag("ol", false), "ol");
        assert_eq!(gen.map_tag("ul", false), "ul");
        assert_eq!(gen.map_tag("li", false), "li");
        assert_eq!(gen.map_tag("option", false), "option");
        assert_eq!(gen.map_tag("blockquote", false), "blockquote");
        assert_eq!(gen.map_tag("video", false), "video");
        assert_eq!(gen.map_tag("Video", false), "video");
        assert_eq!(gen.map_tag("audio", false), "audio");
        assert_eq!(gen.map_tag("Audio", false), "audio");
    }

    #[test]
    fn test_map_tag() {
        let mut gen = VueGenerator::new();

        assert_eq!(gen.map_tag("col", true), "div");
        assert_eq!(gen.map_tag("button", false), "button");
        assert_eq!(gen.map_tag("h2", false), "h2");
    }

    #[test]
    fn test_map_tag_hyphenated() {
        let mut gen = VueGenerator::new();

        // Hyphenated tags should pass through correctly
        assert_eq!(gen.map_tag("preview-card", false), "div");
        assert_eq!(gen.map_tag("preview-card", true), "div");

        // Both previewcard and preview-card should map to the same thing
        assert_eq!(
            gen.map_tag("previewcard", false),
            gen.map_tag("preview-card", false)
        );

        // Other hyphenated tags (fallback to div for unknown)
        assert_eq!(gen.map_tag("my-custom-tag", false), "div");

        // Known tags with hyphens in HTML5
        // (these would pass through if added to the match)
    }

    /// Plan 442 A4: SVG 元素按原名直通(不再退化 div;musk-038 T9 canary
    /// 的 <div :viewBox=...> 退化根因),非 SVG 未知小写标签仍退化。
    #[test]
    fn test_map_tag_svg_passthrough() {
        let mut gen = VueGenerator::new();
        for tag in [
            "svg",
            "path",
            "circle",
            "rect",
            "line",
            "polyline",
            "polygon",
            "ellipse",
            "g",
            "defs",
            "use",
            "stop",
            "linearGradient",
            "radialGradient",
            "clipPath",
            "mask",
        ] {
            assert_eq!(gen.map_tag(tag, false), tag, "svg family tag passthrough");
        }
        // 未知小写标签仍走 div 兜底。
        assert_eq!(gen.map_tag("my-custom-tag", false), "div");
    }

    /// Plan 442 A4: svg 子树发射为原生 SVG 标记——字面量属性为静态
    /// attribute(viewBox="0 0 24 24"),不再出现 :viewBox="'…'" 绑定。
    #[test]
    fn test_svg_element_static_attributes() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget W {
    view {
        svg (viewBox: "0 0 24 24") {
            path (d: "M4 4h16v16h-16z", fill: "currentColor")
        }
    }
}
"#,
        );
        assert!(sfc.contains("<svg"), "svg tag emitted verbatim:\n{}", sfc);
        assert!(
            sfc.contains("viewBox=\"0 0 24 24\""),
            "viewBox as static attribute:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<path") && sfc.contains("d=\"M4 4h16v16h-16z\""),
            "path with static d:\n{}",
            sfc
        );
        assert!(!sfc.contains(":viewBox"), "no bound viewBox:\n{}", sfc);
        assert!(!sfc.contains(":d="), "no bound d:\n{}", sfc);
    }

    #[test]
    fn test_conditional_style_prop() {
        // Plan 346: `style: if cond { "a" } else { "b" }` must emit a
        // `:class="cond ? 'a' : 'b'"` binding — NOT be silently dropped.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget W {
    model { var active bool = false }
    view {
        col {
            text "hi" {
                style: if .active { "text-amber-400" } else { "text-emerald-500" }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":class=") && sfc.contains("active ?"),
            "conditional style → :class ternary:\n{}",
            sfc
        );
    }

    #[test]
    fn test_conditional_style_in_view_fn_with_loop_index() {
        // Mirrors block_body.at's RenderTable: a view fn with `for idx, cell`
        // and `style: if idx == 0 { ... } else { ... }` on a text element.
        // gen_sfc_from_widget_src doesn't register view-fn fragments (the
        // production path does via generate_component_from_file), so register
        // them here to exercise the inline path.
        let ast = crate::parse_ui(
            r#"
widget W {
    view {
        col {
            RenderT(a: .a)
        }
    }
}
view fn RenderT(a Any) {
    table {
        tbody {
            for idx, cell in a {
                td {
                    text cell {
                        style: if idx == 0 { "cursor-pointer text-sky-400 hover:underline" } else { "cursor-pointer" }
                    }
                }
            }
        }
    }
}
"#,
        )
        .expect("widget source must parse");
        crate::aura::extract::clear_view_fragments();
        for stmt in ast.domain_stmts() {
            if let auto_ui_dialect::UiStmt::ViewFragmentDecl(frag) = stmt {
                crate::aura::extract::register_view_fragment(frag);
            }
        }
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        // Real builds use Shadcn mode.
        let sfc = VueGenerator::new()
            .with_mode(crate::ui_gen::VueMode::Shadcn)
            .generate(&widget)
            .expect("generate SFC");
        assert!(
            !sfc.contains("import RenderT"),
            "RenderT must inline, not fall back to a component import:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":class=") && sfc.contains("idx == 0 ?"),
            "conditional style with loop index in view fn:\n{}",
            sfc
        );
    }

    #[test]
    fn test_conditional_style_else_if_chain_nested_ternary() {
        // Plan 043 M5 #tag-coloring: `style: if a {x} else if b {y} else {z}`
        // must become a NESTED ternary — previously every branch after the
        // first was silently dropped (only branches.first() was read).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget W {
    model { var kind str = "" }
    view {
        col {
            text "hi" {
                style: if .kind == "Dir" { "text-sky-400" } else if .kind == "CodeAtRs" { "text-emerald-400" } else if .kind == "Config" { "text-amber-300" } else { "text-foreground" }
            }
        }
    }
}
"#,
        );
        assert!(
            // Plan 054 M6: the whole ternary is parenthesized so it stays safe
            // inside string concatenation (JS `+` binds tighter than `?:`).
            sfc.contains(":class=\"(kind == 'Dir' ? 'text-sky-400' : (kind == 'CodeAtRs' ? 'text-emerald-400' : (kind == 'Config' ? 'text-amber-300' : 'text-foreground')))\""),
            "else-if chain → nested ternary:\n{}",
            sfc
        );
    }

    /// DF-1: a nested `if` in the *then-branch body* of a style binding must
    /// produce a nested ternary, not be flattened to an empty string.
    /// `style: if a { if b { "x" } else { "y" } } else { "z" }`
    /// → `a ? (b ? 'x' : 'y') : 'z'`
    #[test]
    fn test_nested_if_in_style_branch() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget W {
    model { var sid str = "" }
    view {
        col {
            text "t" {
                style: if .sid != "" { if .sid == "a" { "item active" } else { "item" } } else { "item" }
            }
        }
    }
}
"#,
        );
        // The then-branch is a nested if → must become a parenthesized
        // ternary, NOT an empty string. (Plan 054 M6: whole-ternary outer
        // parens + the nested-branch's own parens → double layer.)
        assert!(
            sfc.contains(
                ":class=\"(sid != '' ? ((sid == 'a' ? 'item active' : 'item')) : 'item')\""
            ),
            "nested if in then-branch → nested ternary:\n{}",
            sfc
        );
    }

    /// DF-1 variant: a nested `if` in the *else-branch body*.
    #[test]
    fn test_nested_if_in_style_else_branch() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget W {
    model { var a str = "" }
    view {
        col {
            text "t" {
                style: if .a == "x" { "foo" } else { if .a == "y" { "bar" } else { "baz" } }
            }
        }
    }
}
"#,
        );
        assert!(
            // Plan 054 M6: outer parens around the whole ternary; the nested
            // else-branch ternary keeps its own layer (double-parenthesized).
            sfc.contains(":class=\"(a == 'x' ? 'foo' : ((a == 'y' ? 'bar' : 'baz')))\""),
            "nested if in else-branch → nested ternary:\n{}",
            sfc
        );
    }

    fn test_pattern_to_handler_name() {
        let gen = VueGenerator::new();

        assert_eq!(gen.pattern_to_handler_name("Msg::Inc"), "onInc");
        assert_eq!(gen.pattern_to_handler_name(".Inc"), "Inc");
        assert_eq!(gen.pattern_to_handler_name(".openSidebar"), "openSidebar");
        assert_eq!(gen.pattern_to_handler_name("Dec"), "onDec");
    }

    #[test]
    fn test_shadcn_mode() {
        let gen = VueGenerator::new_shadcn();
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new();
        assert!(!gen.is_shadcn());
    }

    #[test]
    fn test_library_mode_constructor() {
        let gen = VueGenerator::new_library();
        assert!(gen.is_library());
        assert!(!gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Library);
        assert!(gen.is_library());
    }

    #[test]
    fn test_library_button_sfc_is_self_contained() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("button").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(
            !sfc.contains("@/components/ui/"),
            "must NOT import shadcn-vue"
        );
        assert!(sfc.contains("reka-ui"), "uses reka-ui as backend");
    }

    #[test]
    fn test_library_button_support_files() {
        let gen = VueGenerator::new_library();
        let files = gen.generate_widget_support_files("button");
        let names: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert!(
            names.contains(&"variants.ts"),
            "variants.ts present: {:?}",
            names
        );
        assert!(names.contains(&"index.ts"), "index.ts present: {:?}", names);
        let index = files.iter().find(|(p, _)| p == "index.ts").unwrap();
        assert!(index.1.contains("Button"), "index re-exports Button");
    }

    #[test]
    fn test_library_input_sfc_is_self_contained() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("input").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(
            !sfc.contains("@/components/ui/"),
            "must NOT import shadcn-vue"
        );
    }

    #[test]
    fn test_library_label_sfc_uses_reka_ui() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("label").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(sfc.contains("reka-ui"), "label uses reka-ui Label");
        assert!(
            !sfc.contains("@/components/ui/"),
            "must NOT import shadcn-vue"
        );
    }

    /// Plan 400 B-phase: ChatMessage library widget generates a self-contained
    /// SFC with header + bubble structure.
    #[test]
    fn test_library_chat_message_sfc() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("chat_message").unwrap();
        assert!(sfc.contains("<template>"), "has template");
        assert!(sfc.contains("<script setup"), "has script setup");
        assert!(sfc.contains("roleLabel"), "has role label computed");
        assert!(sfc.contains("msg-row"), "has msg-row class");
        assert!(sfc.contains("bg-primary"), "user bubble uses primary bg");
        assert!(sfc.contains("bg-card"), "ai bubble uses card bg");
    }

    #[test]
    fn test_library_unknown_widget_errors() {
        let mut gen = VueGenerator::new_library();
        let err = gen.generate_widget_sfc("does-not-exist").unwrap_err();
        assert!(format!("{err}").contains("Unknown widget"), "got: {err}");
    }

    #[test]
    fn test_library_sfc_has_attribution_header() {
        let mut gen = VueGenerator::new_library();
        let sfc = gen.generate_widget_sfc("button").unwrap();
        assert!(
            sfc.starts_with("<!-- Generated by AutoUI"),
            "must start with attribution header: {}",
            sfc.lines().next().unwrap_or("")
        );
        assert!(
            sfc.contains("shadcn-vue (MIT)"),
            "must cite shadcn-vue (MIT)"
        );
        assert!(sfc.contains("NOTICES"), "must point to NOTICES");
    }

    #[test]
    fn test_library_all_widgets_self_contained() {
        let mut gen = VueGenerator::new_library();
        for name in VueGenerator::LIBRARY_WIDGETS {
            let sfc = gen
                .generate_widget_sfc(name)
                .unwrap_or_else(|e| panic!("generate {name}: {e}"));
            assert!(sfc.contains("<template>"), "{name}: has template");
            assert!(sfc.contains("<script setup"), "{name}: has script setup");
            assert!(!sfc.contains("@/components/ui/"), "{name}: self-contained");
            assert!(
                sfc.starts_with("<!-- Generated by AutoUI"),
                "{name}: attribution header"
            );
        }
    }

    #[test]
    fn test_library_reka_ui_backed_widgets() {
        let mut gen = VueGenerator::new_library();
        // widget -> a marker that proves it binds the right reka-ui primitive.
        let markers: &[(&str, &str)] = &[
            ("checkbox", "CheckboxRoot"),
            ("switch", "SwitchRoot"),
            ("separator", "<Separator"),
            ("avatar", "AvatarRoot"),
            ("dialog", "DialogRoot"),
            ("tabs", "TabsRoot"),
            ("label", "Label"),
        ];
        for (name, marker) in markers {
            let sfc = gen.generate_widget_sfc(name).unwrap();
            assert!(sfc.contains(marker), "{name}: should use {marker}");
        }
    }

    #[test]
    fn test_library_composite_widget_support_files() {
        let gen = VueGenerator::new_library();
        // card ships 5 companion SFCs.
        let card_files: Vec<String> = gen
            .generate_widget_support_files("card")
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        for companion in [
            "index.ts",
            "CardHeader.vue",
            "CardTitle.vue",
            "CardDescription.vue",
            "CardContent.vue",
            "CardFooter.vue",
        ] {
            assert!(
                card_files.contains(&companion.to_string()),
                "card missing {companion}"
            );
        }
        // tabs ships 3 companion SFCs.
        let tabs_files: Vec<String> = gen
            .generate_widget_support_files("tabs")
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        for companion in [
            "index.ts",
            "TabsList.vue",
            "TabsTrigger.vue",
            "TabsContent.vue",
        ] {
            assert!(
                tabs_files.contains(&companion.to_string()),
                "tabs missing {companion}"
            );
        }
    }

    #[test]
    fn test_library_index_reexports_all_vue_files() {
        let gen = VueGenerator::new_library();
        let files = gen.generate_widget_support_files("card");
        let index = files.iter().find(|(n, _)| n == "index.ts").unwrap();
        // primary + 5 companions = 6 re-exports
        assert_eq!(index.1.matches("export").count(), 6, "index: {}", index.1);
        assert!(index.1.contains("Card"), "re-exports Card");
        assert!(index.1.contains("CardHeader"), "re-exports CardHeader");
    }

    /// Plan 337: pin LIBRARY_WIDGETS self-consistency — every advertised
    /// widget must have a renderable template, and the list must be sorted.
    #[test]
    fn test_library_widgets_list_is_self_consistent() {
        for name in VueGenerator::LIBRARY_WIDGETS {
            let mut gen = VueGenerator::new_library();
            gen.generate_widget_sfc(name).unwrap_or_else(|e| {
                panic!("LIBRARY_WIDGETS lists '{name}' but template is missing: {e}")
            });
        }
        let mut sorted = VueGenerator::LIBRARY_WIDGETS.to_vec();
        sorted.sort();
        assert_eq!(
            VueGenerator::LIBRARY_WIDGETS.to_vec(),
            sorted,
            "LIBRARY_WIDGETS must be sorted"
        );
    }

    /// Plan 337 Task 2.1: drift guard — every library widget must exist in
    /// the AURA registry (exact tag or prefix-grouped composite).
    #[test]
    fn test_library_widgets_exist_in_aura_registry() {
        let reg = WidgetRegistry::with_defaults();
        let aura_tags: std::collections::HashSet<&str> =
            reg.all_widgets().keys().map(|s| s.as_str()).collect();
        for w in VueGenerator::LIBRARY_WIDGETS {
            let known =
                aura_tags.contains(*w) || aura_tags.iter().any(|t| t.starts_with(&format!("{w}-")));
            assert!(
                known,
                "LIBRARY_WIDGETS has '{w}' but AURA registry has no such widget"
            );
        }
    }

    /// Plan 337 Task 2.1: covers_aura_tag composite coverage logic.
    #[test]
    fn test_covers_aura_tag_composite() {
        // Exact match
        assert!(VueGenerator::covers_aura_tag("button"));
        assert!(VueGenerator::covers_aura_tag("card"));
        // Composite (prefix-dash)
        assert!(VueGenerator::covers_aura_tag("card-content"));
        assert!(VueGenerator::covers_aura_tag("card-header"));
        // Non-covered
        assert!(!VueGenerator::covers_aura_tag("accordion"));
        assert!(!VueGenerator::covers_aura_tag("data-table"));
    }

    #[test]
    fn test_shadcn_map_tag() {
        let mut gen = VueGenerator::new_shadcn();

        // Should return shadcn component names
        assert_eq!(gen.map_tag("button", false), "Button");
        assert_eq!(gen.map_tag("input", true), "Input");
        assert_eq!(gen.map_tag("textarea", true), "Textarea");
        assert_eq!(gen.map_tag("checkbox", true), "Checkbox");
        assert_eq!(gen.map_tag("toggle", true), "Switch");
        assert_eq!(gen.map_tag("select", false), "Select");
        assert_eq!(gen.map_tag("progress", true), "Progress");
        assert_eq!(gen.map_tag("badge", true), "Badge");
        assert_eq!(gen.map_tag("card", false), "Card");
        assert_eq!(gen.map_tag("avatar", true), "Avatar");
        // PLAN-661 T-04（R-5）：slider 统一原生 range——shadcn 组件名退役，
        // 双模式恒映射 input（属性面 type="range" 注入）。
        assert_eq!(gen.map_tag("slider", true), "input");

        // Layout elements should still return div
        assert_eq!(gen.map_tag("col", false), "div");
        assert_eq!(gen.map_tag("row", false), "div");
    }

    #[test]
    fn test_shadcn_registry() {
        // Plan 435 P4-5a:活链路(docs/schema/aura.at → overlay → WidgetRegistry)。
        // 旧 ShadcnRegistry(死表,仅测试引用)已删除;其独有映射是死数据,
        // 活链路从未使用 —— 负断言存档。
        let registry = super::WidgetRegistry::with_defaults();
        let has_component = |tag: &str| registry.get_primary_component("vue", tag).is_some();
        let primary_component = |tag: &str| registry.get_primary_component("vue", tag);

        // 活映射存在
        for tag in ["button", "input", "checkbox", "tabs", "table", "card"] {
            assert!(has_component(tag), "live mapping missing: {tag}");
        }
        // 主组件名(schema vue.component)
        assert_eq!(primary_component("button"), Some("Button".to_string()));
        assert_eq!(primary_component("input"), Some("Input".to_string()));
        assert_eq!(primary_component("toggle"), Some("Switch".to_string()));
        // 死表独有(活链路从未使用):tab/divider 走原生/家族路径
        assert!(!has_component("tab"));
        assert!(!has_component("divider"));
    }

    // NOTE (plan 012 batch C): the remaining literal-only
    // `test_generate_shadcn_attrs_*` tests below call the internal
    // attr-mapper directly with hand-built literal props (Expr::Str/Int/Bool).
    // That carries no fake-green risk — the parser produces byte-identical
    // Expr nodes for literals — so they stay as unit tests of the internal
    // API. Every test that referenced STATE via `Expr::Ident` (a shape the
    // real parser never produces; dot_item yields Dot(Ident("self"), name))
    // has been converted to the real parse path (see area/bar/line/donut
    // charts, input, checkbox, tabs-with-model, radiogroup).

    #[test]
    fn test_charts_gallery_compiles() {
        // Integration test (Plan 484): the charts gallery consumes the
        // official chart components via M4 bare-name primitives. PLAN-601
        // T-11 修复预存红：fixture 随 484 M4 迁 examples/ui/024-charts（旧
        // examples/charts-gallery 路径已不存在）；断言按现行架构改写——
        // 裸名 chart 经包折叠生成数据绑定占位（<div :data=...>），不再是
        // M3 时代的 PascalCase SFC 引用（vue 腿裸名组件 SFC 化=已知缺口，
        // SVG/组件 token 通道随 P601-T11 债项跟进）。
        // 包感知入口(generate_component_from_file 链):裸名 tag 经
        // use{package} → known_sub_widgets 折叠解析。
        use crate::ui_gen::{generate_component_from_file, ComponentGenOptions};
        let result = generate_component_from_file(
            std::path::Path::new("../../examples/ui/024-charts/src/front/app.at"),
            ComponentGenOptions {
                shadcn: Some(true),
                ..Default::default()
            },
        );
        assert!(
            result.is_ok(),
            "charts gallery should compile: {:?}",
            result.err()
        );
        let result = result.unwrap();
        let code = result.vue_code;

        // Bare-name chart fold: data-bound placeholder per chart branch
        assert!(
            code.contains(":data="),
            "chart data binding missing: {code}"
        );
        assert!(
            code.contains("chartType == 'line'"),
            "line branch missing: {code}"
        );
        assert!(
            code.contains("chartType == 'bar'"),
            "bar branch missing: {code}"
        );
        assert!(
            code.contains("chartType == 'area'"),
            "area branch missing: {code}"
        );
        assert!(
            code.contains("chartType == 'donut'"),
            "donut branch missing: {code}"
        );

        // Retired shadcn/unovis chart family must NOT appear
        assert!(
            !code.contains("chart-area"),
            "retired chart-area scaffold leaked"
        );
        assert!(
            !code.contains("unovis"),
            "retired @unovis dependency leaked"
        );
        assert!(
            !code.contains("CurveType"),
            "retired CurveType mapping leaked"
        );

        // Key props of the migrated (024-charts) demo data are emitted
        assert!(
            code.contains(":data=\"monthly\""),
            "monthly data binding missing: {code}"
        );
        assert!(
            code.contains(":index=\"'m'\""),
            "index prop missing: {code}"
        );
        assert!(
            code.contains("'desktop', 'mobile', 'tablet'"),
            "series fields missing: {code}"
        );
        assert!(
            code.contains("windowLen"),
            "windowLen binding missing: {code}"
        );
    }
    #[test]
    fn test_generate_shadcn_attrs_button() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test button with text
        props.insert(
            "text".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("Click me".into())),
        );
        let (attrs, _slot_content, slot_children) =
            gen.generate_shadcn_attrs("button", &props, &events);

        assert!(slot_children.is_some());
        assert!(slot_children.unwrap().contains("Click me"));
    }

    #[test]
    fn test_generate_shadcn_attrs_input() {
        // Real parse path (plan 012 batch C): `value: .name` on a state field
        // must fold to v-model (Dot path, not the fake bare Ident).
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget W {
    model { var name str = "" }
    view {
        input(value: .name, placeholder: "Enter name")
    }
}
"#,
        );
        assert!(sfc.contains("v-model=\"name\""), "input v-model:\n{}", sfc);
        assert!(
            sfc.contains("placeholder=\"Enter name\""),
            "placeholder:\n{}",
            sfc
        );
    }

    #[test]
    fn test_generate_shadcn_attrs_checkbox() {
        // Real parse path (plan 012 batch C).
        // Test checkbox with v-model (reka-ui uses modelValue, not checked)
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget W {
    model { var done bool = false }
    view {
        checkbox(checked: .done)
    }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"done\""),
            "checkbox v-model:\n{}",
            sfc
        );
    }

    #[test]
    fn test_generate_project_files() {
        // Test scaffold file generation
        let components_json = VueGenerator::generate_components_json();
        assert!(components_json.contains("shadcn-vue"));
        assert!(components_json.contains("tailwind"));

        let package_json = VueGenerator::generate_package_json("test-project");
        assert!(package_json.contains("test-project"));
        assert!(package_json.contains("radix-vue"));
        assert!(package_json.contains("tailwind-merge"));

        let vite_config = VueGenerator::generate_vite_config();
        assert!(vite_config.contains("@vitejs/plugin-vue"));
        assert!(vite_config.contains("alias"));

        let utils_ts = VueGenerator::generate_utils_ts();
        assert!(utils_ts.contains("cn"));
        assert!(utils_ts.contains("clsx"));
        assert!(utils_ts.contains("tailwind-merge"));
        // PLAN-601 T-09/D3：generate_base_css 退役（P593-D3，唯一调用方
        // =本测试；zinc 主题表存续 design_tokens registry 作可切换内置）。
    }

    // ========================================
    // Phase 3: Layout & Navigation Tests
    // ========================================

    /// PLAN-656 T-07: scroll-pane vue codegen 黄金——axis 映射（plain 类 +
    /// shadcn orientation）、hidden 内联样式、controller 锚 + helper 注入
    /// 置位、onscroll 8 位置实参箭头、T-08 spacer（managed bridge）内联
    /// 逻辑尺寸。
    #[test]
    fn p656_scroll_pane_vue_codegen() {
        use auto_lang_core::ast::Expr;
        // plain: axis 三值 → overflow 类。
        let mut gen = VueGenerator::new();
        let cls = |axis: &str, sb: Option<&str>| {
            let mut props = HashMap::new();
            props.insert(
                "axis".to_string(),
                AuraPropValue::Expr(Expr::Str(axis.into())),
            );
            if let Some(sb) = sb {
                props.insert(
                    "scrollbar".to_string(),
                    AuraPropValue::Expr(Expr::Str(sb.into())),
                );
            }
            gen.extract_classes("scroll-pane", &props).0
        };
        assert!(
            cls("y", None).contains("overflow-y-auto"),
            "{}",
            cls("y", None)
        );
        assert!(
            cls("x", None).contains("overflow-x-auto"),
            "{}",
            cls("x", None)
        );
        assert!(
            cls("both", None).contains("overflow-auto"),
            "{}",
            cls("both", None)
        );
        assert!(cls("y", Some("always")).contains("overflow-y-scroll"));
        // legacy direction 映射（axis 缺席）。
        let mut props = HashMap::new();
        props.insert(
            "direction".to_string(),
            AuraPropValue::Expr(Expr::Str("horizontal".into())),
        );
        assert!(gen
            .extract_classes("scroll", &props)
            .0
            .contains("overflow-x-auto"));

        // shadcn: axis → orientation；controller 锚 + helper 置位 + hidden。
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        props.insert(
            "axis".to_string(),
            AuraPropValue::Expr(Expr::Str("x".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &HashMap::new());
        assert!(
            attrs
                .iter()
                .any(|a| a.contains("orientation=\"horizontal\"")),
            "{:?}",
            attrs
        );
        props.insert(
            "controller".to_string(),
            AuraPropValue::Expr(Expr::Ident("sc".into())),
        );
        props.insert(
            "scrollbar".to_string(),
            AuraPropValue::Expr(Expr::Str("hidden".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &HashMap::new());
        assert!(
            attrs.iter().any(|a| a.contains(":data-scroll-ctl=\"sc\"")),
            "{:?}",
            attrs
        );
        assert!(
            attrs.iter().any(|a| a.contains("scrollbar-width")),
            "{:?}",
            attrs
        );
        assert!(
            gen.scroll_ctl_helpers_needed,
            "controller helper injection flag"
        );

        // PLAN-692: size prop → --sb-size 内联 CSS 变量（thumb 宽 px）。
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        props.insert("size".to_string(), AuraPropValue::Expr(Expr::Int(16)));
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &HashMap::new());
        assert!(
            attrs.iter().any(|a| a.contains("--sb-size: 16px")),
            "{:?}",
            attrs
        );
        // size + scrollbar:hidden 合并进同一 style 属性。
        props.insert(
            "scrollbar".to_string(),
            AuraPropValue::Expr(Expr::Str("hidden".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &HashMap::new());
        let joined = attrs
            .iter()
            .find(|a| a.starts_with("style="))
            .expect("style attr");
        assert!(
            joined.contains("--sb-size: 16px") && joined.contains("scrollbar-width: none"),
            "{}",
            joined
        );
    }

    /// PLAN-781 T-04: scroll-anchor → data-scroll-anchor（plain 路）——
    /// 静态字面量 → 静态 attr；动态绑定（Ident，for 循环键同机制）→
    /// :data-scroll-anchor v-bind；无裸 scroll-anchor 属性泄漏；
    /// helper 族（scroll_to_child/scroll_anchor_state + __sctl 内核）注入。
    #[test]
    fn plan781_vue_scroll_anchor_attrs_plain() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Test {
    model {
        var rsc str = "ag"
        var akey str = "dyn"
    }
    view {
        scroll-pane (controller: .rsc, style: "h-40") {
            col {
                scroll-anchor: "p0"
                text "first"
            }
            col {
                scroll-anchor: .akey
                text "second"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("data-scroll-anchor=\"p0\""),
            "静态锚点 attr:
{sfc}"
        );
        assert!(
            sfc.contains(":data-scroll-anchor=\"akey\""),
            "动态锚点绑定:
{sfc}"
        );
        assert!(
            !sfc.contains(" scroll-anchor=\"") && !sfc.contains(":scroll-anchor="),
            "裸 scroll-anchor 属性不得泄漏:
{sfc}"
        );
        assert!(
            sfc.contains("function scroll_to_child(")
                && sfc.contains("function scroll_anchor_state("),
            "公共蛇形名 helper 注入:
{sfc}"
        );
        assert!(
            sfc.contains("function __sctlToChild(") && sfc.contains("__sctlAnchorState"),
            "决策命名的 __sctl 内核在 SFC:
{sfc}"
        );
        assert!(
            sfc.contains("data-scroll-ctl="),
            "pane 寻址锚（controller）在场:
{sfc}"
        );
    }

    /// PLAN-781 T-04: shadcn 路 scroll-anchor 单点转发——generate_shadcn_attrs
    /// 顶部发射 + push_passthrough_attrs 不双发；managed 桥标记。
    #[test]
    fn plan781_vue_scroll_anchor_attrs_shadcn() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        props.insert(
            "scroll-anchor".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("p40".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("col", &props, &HashMap::new());
        let hits = attrs.iter().filter(|a| a.contains("scroll-anchor")).count();
        assert_eq!(
            hits, 1,
            "恰好一条 scroll-anchor attr（单点转发，无双发）:
{attrs:?}"
        );
        assert!(
            attrs.iter().any(|a| a == "data-scroll-anchor=\"p40\""),
            "静态 attr 形态:
{attrs:?}"
        );
        assert!(gen.scroll_ctl_helpers_needed, "helper 注入置位");

        // managed 桥标记：scroll-test-content 元素发射 data-scroll-managed
        //（scroll_to_child 的 unsupported 判据）。
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Test {
    model { var rsc str = "ag" }
    view {
        scroll-pane (controller: .rsc) {
            scroll-test-content (extent_w: 1000, extent_h: 2000) {}
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("data-scroll-managed=\"1\""),
            "managed 桥标记:
{sfc}"
        );
    }

    /// PLAN-781 T-04: 锚点 helper 族 TS 注解（strict 面——p701 同形）与
    /// JS 模式零类型泄漏（注解仅 TS 形态出现）。
    #[test]
    fn plan781_vue_anchor_helpers_typed() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    model { var rsc str = "ag" }
    view {
        scroll-pane (controller: .rsc, style: "h-40") {
            col { scroll-anchor: "p0", text "t" }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("function scroll_to_child(k: string, key: string): string"),
            "scroll_to_child 注解:
{sfc}"
        );
        assert!(
            sfc.contains("function scroll_anchor_state(k: string, key: string): string"),
            "scroll_anchor_state 注解:
{sfc}"
        );
        assert!(
            sfc.contains("function __sctlToChild(e: Element, t: Element): void"),
            "__sctlToChild 注解:
{sfc}"
        );
        assert!(
            sfc.contains("const __sctlAnchorStates: Record<string, string> = {}"),
            "store 注解:
{sfc}"
        );
        assert!(
            sfc.contains("as string | undefined"),
            "strict 下索引访问的 undefined 收窄需要显式 cast（TS2367 规避）:
{sfc}"
        );

        // JS 模式：无任何 TS 注解泄漏。
        let ast = crate::parse_ui(
            r#"
widget Test {
    model { var rsc str = "ag" }
    view {
        scroll-pane (controller: .rsc, style: "h-40") {
            col { scroll-anchor: "p0", text "t" }
        }
    }
}
"#,
        )
        .expect("parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new().with_typescript(false);
        let sfc_js = gen.generate(&widget).expect("generate SFC");
        assert!(
            sfc_js.contains("function scroll_to_child(k, key)"),
            "JS 裸形:
{sfc_js}"
        );
        assert!(
            !sfc_js.contains("scroll_to_child(k: string")
                && !sfc_js.contains("as string | undefined"),
            "JS 模式不得泄漏 TS 注解:
{sfc_js}"
        );
    }

    /// F-781-R02（T-20）：全部 scroll-pane 元素发射 `data-scroll-pane`
    /// 所有权标记（含无 controller 者——嵌套归属过滤的锚点）；helper 以
    /// `closest('[data-scroll-pane]')===pane` 判最近 pane 归属 + 属性比对
    /// 寻址（弃 CSS selector 拼接——引号/冒号/反斜杠 key 与句柄安全）。
    #[test]
    fn plan781_vue_scroll_pane_ownership_marker_and_helper() {
        // 有 controller 与无 controller 的 scroll-pane 都带标记。
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Test {
    model { var c str = "a" }
    view {
        col {
            scroll-pane (controller: .c, style: "h-20") {
                col { scroll-anchor: "nest:dup", text "x" }
            }
            scroll-pane (style: "h-20") {
                col { scroll-anchor: "plain-inner", text "y" }
            }
        }
    }
}
"#,
        );
        let markers = sfc.matches("data-scroll-pane=\"1\"").count();
        assert_eq!(
            markers, 2,
            "两个 scroll-pane 各发射一次所有权标记（含无 controller 者）:\n{sfc}"
        );
        assert!(
            sfc.contains("closest('[data-scroll-pane]')"),
            "helper 归属谓词在场:\n{sfc}"
        );
        assert!(
            sfc.contains("getAttribute('data-scroll-anchor') === key"),
            "锚点 key 属性比对（selector 拼接退役）:\n{sfc}"
        );
        assert!(
            !sfc.contains("[data-scroll-anchor=\"' + key + '\"]"),
            "旧 selector 拼接形态不得残留:\n{sfc}"
        );
        assert!(
            sfc.contains("getAttribute('data-scroll-ctl') === k"),
            "句柄寻址同为属性比对:\n{sfc}"
        );
        // managed 判据走同一归属谓词（嵌套 managed 不误伤外层）。
        assert!(
            sfc.contains("if (owns(el)) return 'error:unsupported'"),
            "managed 归属过滤在场:\n{sfc}"
        );
    }

    /// F-781-R02（T-20）：shadcn 路 scroll（ScrollArea）同样发射所有权标记。
    #[test]
    fn plan781_vue_scroll_area_ownership_marker() {
        let mut gen = VueGenerator::new_shadcn();
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &HashMap::new(), &HashMap::new());
        assert!(
            attrs.iter().any(|a| a == "data-scroll-pane=\"1\""),
            "shadcn scroll 臂所有权标记:\n{attrs:?}"
        );
    }

    /// PLAN-692 W-2: command 家族子件（command_input 等）必须映射到
    /// shadcn 组件而非降级 stub div。
    #[test]
    fn p692_command_family_shadcn_mapping() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        command (style: "x") {
            command-input (placeholder: "p") {}
            command-empty "No results."
            command-list {
                command-group (heading: "G") {
                    command-item (value: "Vue", text: "Vue") {}
                }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Command "),
            "command root must map to <Command>:
{}",
            sfc
        );
        assert!(
            sfc.contains("<CommandInput"),
            "command_input must map to <CommandInput>:
{}",
            sfc
        );
        assert!(
            sfc.contains("<CommandEmpty"),
            "command_empty must map to <CommandEmpty>:
{}",
            sfc
        );
        assert!(
            sfc.contains("<CommandList"),
            "command_list must map to <CommandList>:
{}",
            sfc
        );
        assert!(
            sfc.contains("<CommandGroup"),
            "command_group must map to <CommandGroup>:
{}",
            sfc
        );
        assert!(
            sfc.contains("<CommandItem"),
            "command_item must map to <CommandItem>:
{}",
            sfc
        );
    }

    /// PLAN-701 供⑤b: scroll controller helper 族在 TS 模式必须带类型
    /// 注解——原裸形参在 `strict: true` 下 TS7006（隐式 any）×12 build 红
    /// （m1-supply §14/§15；auto-edit App.vue:23-28 实测）。
    #[test]
    fn p701_scroll_ctl_helpers_typed() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        col {
            scroll (axis: "y", controller: .sc, style: "h-40") {
                text "content"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("function scroll_to(k: string, a: string | number, b: number): void"),
            "scroll_to must carry TS annotations:
{sfc}"
        );
        assert!(
            sfc.contains("function __scrollCtlEl(k: string): Element | null"),
            "__scrollCtlEl must annotate k + nullable return:
{sfc}"
        );
        assert!(
            sfc.contains("function scroll_controller(): string"),
            "scroll_controller must annotate return:
{sfc}"
        );
        assert!(
            sfc.contains("function scroll_state(k: string): { offset_x: number;"),
            "scroll_state must annotate the record return:
{sfc}"
        );
        // TS7006 反断面：helper 形参行不得再出现无注解裸形参。
        assert!(
            !sfc.contains("function scroll_to(k, a, b)"),
            "unannotated scroll_to must not survive:
{sfc}"
        );
    }

    /// PLAN-692: code_editor 字面串 content 此前被静默丢弃（挂载即空白编辑器）
    /// ——字面量走单向 `:model-value` 初始值，`"` 以 &quot; 实体化保护属性分隔符；
    /// 状态绑定 content 的 v-model 形态不受影响。
    #[test]
    fn p692_code_editor_literal_content() {
        // 字面串 content → :model-value 单向初始值（含换行/双引号转义）。
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        code_editor (key: "ed", lang: "auto", style: "h-56 w-full") {
            content: "widget Hello {\n    model { name str = \"world\" }\n}\n"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":model-value="),
            "literal content must emit :model-value:\n{}",
            sfc
        );
        assert!(
            sfc.contains("&quot;world&quot;"),
            "double quotes must be entity-escaped:\n{}",
            sfc
        );
        assert!(sfc.contains("\\n"), "newlines must be JS-escaped:\n{}", sfc);
        assert!(
            !sfc.contains("v-model="),
            "literal content must not emit v-model:\n{}",
            sfc
        );

        // 状态绑定 content → v-model（既有契约不回归）。
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    model { source str = "" }
    view {
        code_editor (key: "ed", lang: "rust") {
            content: .source
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"source\""),
            "bound content must keep v-model:\n{}",
            sfc
        );
    }

    /// PLAN-692: utility 滚动路径（overflow-*-auto 类）统一原生滚动条皮肤——
    /// plain 装配点对带 overflow-* 语义类的容器自动挂 .ash-scroll；语义 scroll
    /// 元素走 ScrollArea 组件不挂；无 overflow 的容器不挂。
    #[test]
    fn p692_utility_scroll_ash_scroll_class() {
        // div + overflow-y-auto → 挂 ash-scroll。
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        div (style: "flex flex-col h-40 overflow-y-auto border rounded-md p-2") {
            text "items"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("ash-scroll"),
            "expected ash-scroll on overflow container:\n{}",
            sfc
        );

        // 无 overflow 的 div → 不挂。
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        div (style: "flex flex-col gap-2 border rounded-md") {
            text "items"
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("ash-scroll"),
            "unexpected ash-scroll without overflow:\n{}",
            sfc
        );

        // 语义 scroll 元素 → ScrollArea 组件，不挂 ash-scroll。
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        scroll (style: "h-40 border rounded-md") {
            text "items"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("ScrollArea"),
            "expected ScrollArea component:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("ash-scroll"),
            "scroll element must not get ash-scroll:\n{}",
            sfc
        );
    }

    #[test]
    fn test_generate_shadcn_attrs_scroll() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test scroll area with orientation
        props.insert(
            "orientation".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("vertical".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("scroll", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tabs with default value
        props.insert(
            "default".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("tab1".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("tabs", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("default-value=\"tab1\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tabs_with_model() {
        // Real parse path (plan 012 batch C).
        // Test tabs with v-model
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget W {
    model { var activeTab str = "a" }
    view {
        tabs(value: .activeTab) {
            tab(value: "a", text: "A")
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"activeTab\""),
            "tabs v-model:\n{}",
            sfc
        );
    }

    #[test]
    fn test_generate_shadcn_attrs_tab() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tab trigger with value and text
        props.insert(
            "value".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("tab1".into())),
        );
        props.insert(
            "text".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("First Tab".into())),
        );
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tab", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("value=\"tab1\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "First Tab");
    }

    #[test]
    fn test_generate_shadcn_attrs_card() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test card with variant and title
        props.insert(
            "variant".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("outline".into())),
        );
        props.insert(
            "title".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("Card Title".into())),
        );
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("card", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("variant=\"outline\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Card Title");
    }

    #[test]
    fn test_generate_shadcn_attrs_divider() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test separator with orientation
        props.insert(
            "orientation".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("vertical".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("divider", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("orientation=\"vertical\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_divider_decorative() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test decorative separator
        props.insert(
            "decorative".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(true)),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("divider", &props, &events);

        assert!(attrs.iter().any(|a| a == "decorative"));
    }

    #[test]
    fn test_shadcn_registry_phase3_components() {
        // Plan 435 P4-5a:活链路(docs/schema/aura.at → overlay → WidgetRegistry)。
        // 旧 ShadcnRegistry(死表,仅测试引用)已删除;其独有映射是死数据,
        // 活链路从未使用 —— 负断言存档。
        let registry = super::WidgetRegistry::with_defaults();
        let has_component = |tag: &str| registry.get_primary_component("vue", tag).is_some();
        let primary_component = |tag: &str| registry.get_primary_component("vue", tag);

        assert!(has_component("scroll"));
        assert!(has_component("tabs"));
        assert!(has_component("card"));
        assert_eq!(primary_component("scroll"), Some("ScrollArea".to_string()));
        assert_eq!(primary_component("card"), Some("Card".to_string()));
        assert_eq!(primary_component("divider").is_none(), true);

        // 导入路径与家族 extras(活链路:spec.backend("vue"))
        let scroll = registry.get("scroll").unwrap();
        let m = scroll.backend("vue").unwrap();
        assert!(m.import.as_deref().unwrap_or("").contains("scroll-area"));
        // Plan 484: chart 家族(ChartTooltip/ChartLegend/ChartStyle extras)
        // 已随 shadcn-vue chart 路线退役——裸名让位 official 包 Auto 组件。
        assert!(
            registry.get("chart").is_none(),
            "shadcn chart family must be retired"
        );
    }

    // ========================================
    // Phase 4: Overlay & Feedback Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_modal() {
        // Build the props through the REAL parse path (widget source → parser →
        // aura extract → shadcn attrs). The previous version hand-built
        // `Expr::Ident("showDialog")`, which no real `.state` ref ever produces
        // (dot_item yields Dot(Ident("self"), name)) — so the test stayed green
        // while v-model:open was silently dropped for real widgets.
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget ConfirmDialog {
    model { var showDialog bool = false }
    view {
        modal(open: .showDialog, title: "Confirm Delete") {
            text "Are you sure?"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Dialog v-model:open=\"showDialog\""),
            "modal routed to <Dialog> with v-model:open:\n{}",
            sfc
        );
        assert!(
            sfc.contains("data-title=\"Confirm Delete\""),
            "title preserved:\n{}",
            sfc
        );
    }

    /// Item 1 regression: builtin `dialog` tag routed through the shadcn
    /// Dialog component keeps its v-model:open binding (real parse path).
    #[test]
    fn test_dialog_vmodel_open_real_dsl() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    model { var show bool = false }
    view {
        dialog(open: .show) {
            text "hi"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Dialog v-model:open=\"show\""),
            "dialog → <Dialog v-model:open>:\n{}",
            sfc
        );
    }

    #[test]
    fn parser_minted_modal_toggle_is_not_forwarded_to_vue_trigger() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    view {
        dialog {
            dialog_trigger "Open dialog"
            dialog_content { dialog_title "Details" }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Dialog v-model:open=\"__dlg_open_1\""),
            "minted state remains bound:\n{sfc}"
        );
        assert!(
            sfc.contains("<DialogTrigger"),
            "native Vue trigger remains:\n{sfc}"
        );
        assert!(
            !sfc.contains("DialogTrigger @click=\"__dlg_toggle_1\""),
            "Vue's trigger must own the open transition:\n{sfc}"
        );

        let explicit = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    msg { OpenDialog }
    on { .OpenDialog -> { } }
    view {
        dialog {
            dialog_trigger (onclick: .OpenDialog) "Open dialog"
            dialog_content { dialog_title "Details" }
        }
    }
}
"#,
        );
        assert!(
            explicit.contains("DialogTrigger @click=\"OpenDialog\""),
            "explicit handler remains:\n{explicit}"
        );
    }

    /// Item 1 regression: builtin `alertdialog` keeps v-model:open (this was
    /// the original silent-drop: extract_state_ref only matched bare Ident).
    #[test]
    fn test_alertdialog_vmodel_open_real_dsl() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    model { var confirm_open bool = false }
    view {
        alertdialog(open: .confirm_open) {
            alertdialogtitle(text: "Sure?")
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<AlertDialog v-model:open=\"confirm_open\""),
            "alertdialog → <AlertDialog v-model:open>:\n{}",
            sfc
        );
    }

    #[test]
    fn schema_snake_case_overlay_tags_use_shadcn_components() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget OverlayDemo {
    view {
        alert_dialog {
            alert_dialog_trigger { button (text: "Open dialog") {} }
            alert_dialog_content {
                alert_dialog_header {
                    alert_dialog_title "Confirm"
                    alert_dialog_description "Continue?"
                }
                alert_dialog_footer {
                    alert_dialog_cancel "Cancel"
                    alert_dialog_action "Continue"
                }
            }
        }
        sheet {
            sheet_trigger { button (text: "Open sheet") {} }
            sheet_content { sheet_header { sheet_title "Details" } }
        }
    }
}
"#,
        );
        assert!(sfc.contains("<AlertDialog"), "root mapping missing:\n{sfc}");
        assert!(
            sfc.contains("<AlertDialogContent"),
            "content mapping missing:\n{sfc}"
        );
        assert!(
            sfc.contains("<AlertDialogTrigger"),
            "trigger mapping missing:\n{sfc}"
        );
        assert!(
            sfc.contains("<SheetContent"),
            "sheet content mapping missing:\n{sfc}"
        );
        assert!(
            sfc.contains("<SheetTitle"),
            "sheet title mapping missing:\n{sfc}"
        );
        assert!(
            !sfc.contains("<div data-auto-tag=\"alert_dialog\""),
            "snake_case root must not degrade to an HTML div:\n{sfc}"
        );
    }

    #[test]
    fn test_generate_shadcn_attrs_tooltip() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tooltip with content and side
        props.insert(
            "content".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("Help text".into())),
        );
        props.insert(
            "side".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("right".into())),
        );
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tooltip", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("side=\"right\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Help text");
    }

    #[test]
    fn test_generate_shadcn_attrs_spinner() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test spinner/skeleton
        props.insert(
            "class".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("w-10 h-10".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("spinner", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"w-10 h-10\"")));
    }

    #[test]
    fn test_shadcn_registry_phase4_components() {
        // Plan 435 P4-5a:活链路(docs/schema/aura.at → overlay → WidgetRegistry)。
        // 旧 ShadcnRegistry(死表,仅测试引用)已删除;其独有映射是死数据,
        // 活链路从未使用 —— 负断言存档。
        let registry = super::WidgetRegistry::with_defaults();
        let has_component = |tag: &str| registry.get_primary_component("vue", tag).is_some();
        let primary_component = |tag: &str| registry.get_primary_component("vue", tag);

        assert!(has_component("modal"));
        assert!(has_component("tooltip"));
        assert_eq!(primary_component("modal"), Some("Dialog".to_string()));
        // spinner 死表独有;活链路无 vue 映射
        assert!(!has_component("spinner"));
    }

    // ========================================
    // Phase 5: Data Components Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_table() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test table
        props.insert(
            "class".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("w-full".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("table", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"w-full\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_table_cells() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test th with colspan
        props.insert(
            "colspan".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Int(2)),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("th", &props, &events);

        assert!(attrs.iter().any(|a| a.contains(":colspan=\"2\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree
        props.insert(
            "class".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("pl-4".into())),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("tree", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("class=\"pl-4\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_tree_item() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test tree_item with text
        props.insert(
            "text".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("Node 1".into())),
        );
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("tree_item", &props, &events);

        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Node 1");
    }

    #[test]
    fn test_shadcn_registry_phase5_components() {
        // Plan 435 P4-5a:活链路(docs/schema/aura.at → overlay → WidgetRegistry)。
        // 旧 ShadcnRegistry(死表,仅测试引用)已删除;其独有映射是死数据,
        // 活链路从未使用 —— 负断言存档。
        let registry = super::WidgetRegistry::with_defaults();
        let has_component = |tag: &str| registry.get_primary_component("vue", tag).is_some();
        let primary_component = |tag: &str| registry.get_primary_component("vue", tag);

        assert!(has_component("table"));
        // Plan 408 P8:table 家族原生 HTML 直通,不做 vue 映射(死表曾错误声明)
        for tag in ["thead", "tbody", "tr", "th", "td"] {
            assert!(!has_component(tag), "{tag} 应保持原生(Plan 408 P8)");
        }
    }

    // ========================================
    // Phase 6: Form Components Tests
    // ========================================

    #[test]
    fn test_generate_shadcn_attrs_radiogroup() {
        // Real parse path (plan 012 batch C).
        // Test radiogroup with v-model
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget W {
    model { var selectedOption str = "" }
    view {
        radiogroup(value: .selectedOption, name: "options") {
            radio(value: "option1", label: "Option 1")
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"selectedOption\""),
            "radiogroup v-model:\n{}",
            sfc
        );
        assert!(sfc.contains("name=\"options\""), "name attr:\n{}", sfc);
    }

    #[test]
    fn test_generate_shadcn_attrs_radio() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test radio with value and label
        props.insert(
            "value".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("option1".into())),
        );
        props.insert(
            "label".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("Option 1".into())),
        );
        let (attrs, slot_content, _) = gen.generate_shadcn_attrs("radio", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("value=\"option1\"")));
        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Option 1");
    }

    #[test]
    fn test_generate_shadcn_attrs_radio_disabled() {
        let mut gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test disabled radio
        props.insert(
            "value".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str("option2".into())),
        );
        props.insert(
            "disabled".to_string(),
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(true)),
        );
        let (attrs, _, _) = gen.generate_shadcn_attrs("radio", &props, &events);

        assert!(attrs.iter().any(|a| a == "disabled"));
    }

    #[test]
    fn test_shadcn_registry_phase6_components() {
        // Plan 435 P4-5a:活链路(docs/schema/aura.at → overlay → WidgetRegistry)。
        // 旧 ShadcnRegistry(死表,仅测试引用)已删除;其独有映射是死数据,
        // 活链路从未使用 —— 负断言存档。
        let registry = super::WidgetRegistry::with_defaults();
        let has_component = |tag: &str| registry.get_primary_component("vue", tag).is_some();
        let primary_component = |tag: &str| registry.get_primary_component("vue", tag);

        // PLAN-661 T-04（R-5）：slider 组件映射随 aura.at vue 行退役——
        // DSL slider 恒走原生 input range，注册面不再持有 Slider 组件。
        assert!(!has_component("slider"));
        assert!(has_component("radiogroup"));
        assert_eq!(primary_component("slider"), None);
        assert_eq!(
            primary_component("radiogroup"),
            Some("RadioGroup".to_string())
        );
        // radio 死表独有(a2ui/原生路径)
        assert!(!has_component("radio"));
    }

    // ========================================================================
    // Router Tests (Plan 105)
    // ========================================================================

    #[test]
    fn test_router_generation() {
        use crate::aura::AuraRoute;

        let routes = vec![
            AuraRoute {
                path: "/".to_string(),
                module: "index".to_string(),
                widget_name: "Index".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/about".to_string(),
                module: "about".to_string(),
                widget_name: "About".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/user/:id".to_string(),
                module: "user".to_string(),
                widget_name: "User".to_string(),
                params: vec!["id".to_string()],
            },
        ];

        let output = VueGenerator::generate_router_file(&routes);

        // Check imports
        assert!(output.contains("import { createRouter, createWebHashHistory }"));

        // Check lazy loading imports (Plan 106)
        assert!(output.contains("component: () => import('@/pages/index.vue')"));
        assert!(output.contains("component: () => import('@/pages/about.vue')"));
        assert!(output.contains("component: () => import('@/pages/user.vue')"));

        // Check route definitions
        assert!(output.contains("path: '/'"));
        assert!(output.contains("path: '/about'"));
        assert!(output.contains("path: '/user/:id'"));

        // Check route with params has props: true
        assert!(output.contains("props: true"));
    }

    #[test]
    fn test_router_generation_empty() {
        let routes: Vec<crate::aura::AuraRoute> = vec![];
        let output = VueGenerator::generate_router_file(&routes);

        // Should still generate valid router structure
        assert!(output.contains("import { createRouter, createWebHashHistory }"));
        assert!(output.contains("const routes: RouteRecordRaw[] = ["));
        assert!(output.contains("export default router"));
    }

    #[test]
    fn test_button_with_text_full_widget() {
        // Real parse path (plan 012 batch C): the previous version hand-built
        // `AuraNode::Element { tag: "Button" }` — a tag the parser never
        // produces (DSL tags are lowercase; "Button" would be a sub-widget
        // reference) — and had been FAILING on that unreachable input. The
        // DSL `button "Click Me"` routes to the shadcn <Button> component.
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Test {
    view {
        button "Click Me"
    }
}
"#,
        );

        // Check that the button is NOT self-closing and has text content
        assert!(
            sfc.contains("<Button") && sfc.contains("Click Me"),
            "sfc:\n{}",
            sfc
        );
        // Should NOT be self-closing (should have >Click Me< pattern)
        assert!(
            sfc.contains("Click Me") && sfc.contains("</Button>"),
            "sfc:\n{}",
            sfc
        );
    }

    // ------------------------------------------------------------------
    // Widget `computed` regressions (AutoDown editor Phase 0 findings)
    // ------------------------------------------------------------------

    /// Bug: `.language` (prop dot-ref) in a computed block was generated as
    /// `self.language`. It must resolve to `props.language`.
    #[test]
    fn test_computed_prop_dot_ref_uses_props_not_self() {
        // Real parse path (plan 012 batch C).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Icon(language: str) {
    computed {
        label => .language
    }
    view { div { text "hi" } }
}
"#,
        );

        assert!(
            sfc.contains("const label = computed<string>(() => props.language)"),
            "computed prop must use props.language:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("self.language"),
            "no self. in output:\n{}",
            sfc
        );
    }

    /// Bug: string concatenation in a computed block was inferred as
    /// `computed<number>`. `"data:" + .language` must be `computed<string>`.
    #[test]
    fn test_computed_string_concat_infers_string() {
        // Real parse path (plan 012 batch C).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Icon(language: str) {
    computed {
        full => "data:" + .language
    }
    view { div { text "hi" } }
}
"#,
        );

        assert!(
            sfc.contains("const full = computed<string>(() => 'data:' + props.language)"),
            "string concat must infer string:\n{}",
            sfc
        );
    }

    /// Plan 448 H1: a block-bodied computed returns its trailing expression
    /// (`transpile_body_as_return`), infers the tail's TS type, and keeps an
    /// explicit trailing `return` unchanged. Previously the block rendered
    /// via transpile_handler_body — the tail was a bare statement, so every
    /// block computed yielded `undefined` with `computed<any>` (the
    /// 018-book-reader NOTE).
    #[test]
    fn test_computed_block_body_returns_tail() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var w int = 3 }
    computed {
        doubled => {
            let base = .w * 2
            base + 1
        }
        labeled => {
            return "v=" + .doubled
        }
    }
    view { col { text `d: ${.doubled}` } }
}
"#,
        );
        assert!(
            sfc.contains("return base + 1;"),
            "trailing expression becomes the return value:\n{}",
            sfc
        );
        assert!(
            sfc.contains("computed<number>"),
            "block type inferred from the tail expression (was <any>):\n{}",
            sfc
        );
        assert!(
            sfc.contains("return 'v=' +"),
            "explicit trailing return renders unchanged:\n{}",
            sfc
        );
    }

    /// Plan 043 M5: if/else-if/else in a computed block must transpile to an
    /// IIFE. Previously Expr::If fell through expr_to_js's catch-all and
    /// emitted `undefined`, so status glyphs/classes silently vanished
    /// (generated `computed<any>(() => undefined)`).
    #[test]
    fn test_computed_if_chain_transpiles_to_ternary() {
        // Real parse path (plan 012 batch C): `status` is now a real prop, so
        // the accessor is `props.status.kind` — the previous hand-built widget
        // had NO props/state, an undeclared-ref shape the DSL cannot express.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget StatusIcon(status: Any) {
    computed {
        glyph => if .status.kind == "Success" { "✓" } else if .status.kind == "Failed" { "✗" } else { "…" }
    }
    view { div { text "hi" } }
}
"#,
        );

        // Plan 408 P9 / §7.2 缺陷 3: a computed if/else-if/else chain now
        // transpiles to a nested ternary (cleaner + type-inferable) instead of
        // the previous IIFE. Plan 043 H1's invariant — each branch's value is
        // preserved (no silent `undefined`) — carries over: the ternary yields
        // the matched branch's expression.
        assert!(
            // Plan 054 M6: whole-ternary outer parens (safe in concat contexts).
            sfc.contains("const glyph = computed<any>(() => (props.status.kind === 'Success' ? '✓' : (props.status.kind === 'Failed' ? '✗' : '…')))"),
            "computed if chain must transpile to a nested ternary preserving each branch value:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("=> undefined"),
            "Expr::If must not fall through to undefined:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("(() => {"),
            "computed if must NOT use the legacy IIFE wrapper:\n{}",
            sfc
        );
    }

    /// Bug: bare identifiers (`lang => language`) and plain function calls
    /// (`btoa(language)`) in a computed block failed to parse with
    /// "Expected term, got RBrace". They must parse and resolve against the
    /// widget's props.
    #[test]
    fn test_computed_parses_bare_ident_and_plain_call() {
        let src = r#"
widget Icon(language: str) {
    computed {
        label => language
        encoded => btoa(language)
    }
    view {
        div {
            class: "icon",
            "hi"
        }
    }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(src).expect("widget computed with bare ident/call must parse");

        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(
            sfc.contains("const label = computed<string>(() => props.language)"),
            "bare ident resolves to prop:\n{}",
            sfc
        );
        assert!(
            sfc.contains("btoa(props.language)"),
            "plain call passes through with resolved arg:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Plan 425: component fn 糖化(widget 单轨)+ view 可选化。
    // ====================================================================

    /// Plan 425 T1 等价性核心断言:同一组件两种写法——`component fn Card
    /// { body }`(糖化 → WidgetDecl)vs `widget Card { blocks... view { body } }`
    /// ——产物逐字节相同(App 的引用与 Card 的 SFC 均)。
    #[test]
    fn test_plan425_component_fn_widget_equivalence() {
        let tmp_sugar = std::env::temp_dir().join("plan425_equiv_sugar");
        let tmp_plain = std::env::temp_dir().join("plan425_equiv_plain");
        let _ = std::fs::remove_dir_all(&tmp_sugar);
        let _ = std::fs::remove_dir_all(&tmp_plain);
        std::fs::create_dir_all(&tmp_sugar).unwrap();
        std::fs::create_dir_all(&tmp_plain).unwrap();

        let sugar_src = concat!(
            "widget App {\n",
            "    model { var heading str = \"demo\" }\n",
            "    view { Card(title: .heading) }\n",
            "}\n",
            "\n",
            "component fn Card(title: str) {\n",
            "    msg Msg { Picked(str) }\n",
            "    model { var clicks int = 0 }\n",
            "    on { .Picked(t) -> { .clicks = .clicks + 1 } }\n",
            "    col {\n",
            "        style: \"p-2\"\n",
            "        text .title\n",
            "        onclick: .Picked(.title)\n",
            "    }\n",
            "}\n",
        );
        let plain_src = concat!(
            "widget App {\n",
            "    model { var heading str = \"demo\" }\n",
            "    view { Card(title: .heading) }\n",
            "}\n",
            "\n",
            "widget Card(title: str) {\n",
            "    msg Msg { Picked(str) }\n",
            "    model { var clicks int = 0 }\n",
            "    view {\n",
            "        col {\n",
            "            style: \"p-2\"\n",
            "            text .title\n",
            "            onclick: .Picked(.title)\n",
            "        }\n",
            "    }\n",
            "    on { .Picked(t) -> { .clicks = .clicks + 1 } }\n",
            "}\n",
        );
        let p1 = tmp_sugar.join("app.at");
        std::fs::write(&p1, sugar_src).unwrap();
        let p2 = tmp_plain.join("app.at");
        std::fs::write(&p2, plain_src).unwrap();

        let r1 = crate::ui_gen::generate_component_from_file(
            &p1,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .expect("sugar spelling must compile");
        let r2 = crate::ui_gen::generate_component_from_file(
            &p2,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .expect("widget spelling must compile");

        let card1 = r1
            .all_widget_codes
            .iter()
            .find(|(n, _)| n == "Card")
            .map(|(_, c)| c)
            .unwrap();
        let card2 = r2
            .all_widget_codes
            .iter()
            .find(|(n, _)| n == "Card")
            .map(|(_, c)| c)
            .unwrap();
        assert_eq!(
            normalize_vue_output(card1),
            normalize_vue_output(card2),
            "Card SFC must be byte-identical:\n--- component fn ---\n{}\n--- widget ---\n{}",
            card1,
            card2
        );
        assert_eq!(
            normalize_vue_output(&r1.vue_code),
            normalize_vue_output(&r2.vue_code),
            "App SFC must be byte-identical:\n--- component fn ---\n{}\n--- widget ---\n{}",
            r1.vue_code,
            r2.vue_code
        );
        // The sugared declaration is a true WidgetDecl (single track).
        assert!(
            r1.widgets.iter().any(|w| w.name == "Card"),
            "Card in widgets list"
        );
        // App references it via the component path.
        assert!(
            r1.vue_code.contains("<Card"),
            "App renders <Card>:\n{}",
            r1.vue_code
        );

        let _ = std::fs::remove_dir_all(&tmp_sugar);
        let _ = std::fs::remove_dir_all(&tmp_plain);
    }

    // ====================================================================
    // Plan 426: setup preamble slot (per-instance setup phase).
    // ====================================================================

    /// Plan 425 兼容性锁定:旧 component fn 的 `on<event>` 回调拼写
    /// (`ontogglecollapse: .Bump`,无下划线)在糖化后经事件路径
    /// (on 前缀 ViewEvent → sub_widget_event_to_vue)仍产出
    /// `@togglecollapse`——与旧 Component 节点轨道一致,非普通 prop。
    /// 根序语义变化(源序首个 widget 为根)由 App 前置体现。
    #[test]
    fn test_plan425_legacy_on_event_spelling() {
        let tmp = std::env::temp_dir().join("audit_on_event");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        std::fs::write(
            &at_path,
            concat!(
                "widget App {
",
                "    model { var count int = 0 }
",
                "    view {
",
                "        CollapseBtn(label: \"go\", ontogglecollapse: .Bump)
",
                "    }
",
                "    on { .Bump -> { .count = .count + 1 } }
",
                "}
",
                "
",
                "component fn CollapseBtn(label: str) {
",
                "    msg Msg { ToggleCollapse }
",
                "    model { var collapsed bool = false }
",
                "    on { .ToggleCollapse -> { .collapsed = !.collapsed } }
",
                "    button {
",
                "        text .label
",
                "        onclick: .ToggleCollapse
",
                "    }
",
                "}
",
            ),
        )
        .unwrap();
        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .expect("must compile");
        let app = result.vue_code.clone();
        assert!(
            app.contains("@togglecollapse"),
            "legacy event spelling works:
{}",
            app
        );
        assert!(
            !app.contains(":ontogglecollapse"),
            "not a plain prop:
{}",
            app
        );
    }

    /// Plan 426 T2: setup statements land at <script setup> top level BEFORE
    /// state/computed definitions (bindings first); the binding is usable in
    /// the template and in on handlers.
    #[test]
    fn test_plan426_setup_preamble_position_and_bindings() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    setup {
        let greeting = "hello"
    }
    msg Msg { Bump }
    model { var count int = 0 }
    view {
        col {
            text .greeting
            text .count
        }
    }
    on {
        .Bump -> { .count = .count + 1 }
    }
}
"#,
        );
        let setup_pos = sfc.find("const greeting");
        let state_pos = sfc.find("const count = ref<number>(0)");
        assert!(setup_pos.is_some(), "setup statement emitted:\n{}", sfc);
        assert!(state_pos.is_some(), "state emitted:\n{}", sfc);
        assert!(
            setup_pos.unwrap() < state_pos.unwrap(),
            "setup statements BEFORE state/computed:\n{}",
            sfc
        );
        // Template binding visible (script-setup top-level const).
        assert!(sfc.contains("greeting"), "template sees binding:\n{}", sfc);
    }

    /// Plan 426 T2: refs annotation — `let t = useT() refs: [locale]` marks
    /// `t.locale` for `.value` injection in handler/computed contexts
    /// (same mechanism as the composable kind's facade_ref_fields).
    #[test]
    fn test_plan426_setup_refs_annotation() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    setup {
        let i18n = useI18n()
        refs i18n: [locale]
    }
    msg Msg { Pick }
    model { var chosen str = "" }
    view {
        col {
            button "pick" {
                onclick: .Pick
            }
        }
    }
    on {
        .Pick -> { .chosen = i18n.locale }
    }
}
"#,
        );
        assert!(
            sfc.contains("const i18n = useI18n()"),
            "setup binding emits const:\n{}",
            sfc
        );
        assert!(!sfc.contains("let i18n"), "no let leakage:\n{}", sfc);
        assert!(
            sfc.contains("i18n.locale.value"),
            "refs field gains .value:\n{}",
            sfc
        );
    }

    /// Plan 426 收口补强:setup 语句引用 model 变量/computed/prop 在发射
    /// 顺序上必 TDZ(setup 先于 state/computed/props 声明),编译期拒绝。
    #[test]
    fn test_plan426_setup_rejects_state_reference() {
        let result = std::panic::catch_unwind(|| {
            gen_sfc_from_widget_src(
                r#"
widget App {
    setup {
        let label = .count
    }
    model { var count int = 0 }
    view { col { text .count } }
}
"#,
            )
        });
        assert!(
            result.is_err(),
            "setup referencing a model var must be a compile error"
        );
    }

    /// Plan 426 T2 MVP: `await` inside a setup block is a hard parse error
    /// (async setup needs a Suspense boundary — registered as future work).
    #[test]
    fn test_plan426_setup_rejects_await() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = "widget App {\n    setup {\n        let x = load().await\n    }\n    view { col { text \"x\" } }\n}\n";
        // PLAN-841: UI sources enter through the explicit domain port;
        // plain parse() no longer carries the UI grammar.
        let err = auto_lang_core::parser::Parser::from(src)
            .with_session(session)
            .parse_domain(&auto_ui_dialect::UiDialect)
            .err()
            .expect("await in setup must fail to parse");
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("await") || msg.contains("Suspense"),
            "error mentions await/Suspense: {}",
            msg
        );
    }

    /// Plan 426 T2: a setup binding colliding with a model var / prop is a
    /// compile error (PLAN-037 T5 same-name precedent).
    #[test]
    fn test_plan426_setup_name_collision() {
        let result = std::panic::catch_unwind(|| {
            gen_sfc_from_widget_src(
                r#"
widget App {
    setup {
        let count = 1
    }
    model { var count int = 0 }
    view { col { text .count } }
}
"#,
            )
        });
        assert!(result.is_err(), "collision with model var must error");
    }

    /// Plan 425 T1 view 可选化:`widget X { col {...} }`(体即视图,无 view
    /// 块)与 `widget X { view { col {...} } }` 产物逐字节相同;混合块顺序
    /// (model 后跟裸元素)同样合法。
    #[test]
    fn test_plan425_widget_view_optional() {
        let with_view = gen_sfc_from_widget_src(
            r#"
widget Counter {
    model { var count int = 0 }
    view {
        col {
            text .count
        }
    }
}
"#,
        );
        let body_as_view = gen_sfc_from_widget_src(
            r#"
widget Counter {
    model { var count int = 0 }
    col {
        text .count
    }
}
"#,
        );
        assert_eq!(
            normalize_vue_output(&with_view),
            normalize_vue_output(&body_as_view),
            "view 可选化:体即视图产物必须与显式 view 块逐字节相同"
        );
        // 解析层断言:无 view 块的 widget 得到 Some(ViewBlock{..})。
        let ast = crate::parse_ui(
            "widget W {\n    model { var x int = 0 }\n    button { text \"go\" }\n}\n",
        )
        .expect("mixed order must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        assert!(decl.view.is_some(), "body-as-view wraps into a view block");
        assert!(
            decl.model.is_some(),
            "model block still parsed before the body"
        );
    }

    // ====================================================================
    // Generic DOM events: keyboard/mouse/wheel events, event modifiers,
    // the $event object, and window/document-level listeners.
    // ====================================================================

    /// Parse a widget source and generate its Vue SFC (full pipeline).
    /// Plan 008 (auto-os-config batch 4): mixed-if loop bodies carry keys on
    /// branch-first texts — the fallback v-for wrapper must hoist one.
    #[test]
    fn test_plan008_loop_wrapper_hoists_child_key() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Rows {
    msg Msg { Go(str) }
    model { var items Array<str> = [] }
    view {
        col {
            for e in .items {
                if e.kind == "text" {
                    row {
                        text (text: e.label, key: e.key) {}
                        button (text: "Go") { onclick: .Go(e.key) }
                    }
                }
                if e.kind == "toggle" {
                    row {
                        text (text: e.label, key: e.key) {}
                    }
                }
            }
        }
    }
    on {
        .Go(k) -> { }
    }
}
"#,
        );
        assert!(
            sfc.contains("v-for=\"e in"),
            "loop must emit v-for:
{}",
            sfc
        );
        let wrapper = sfc.lines().find(|l| l.contains("v-for=\"e in")).unwrap();
        let keylines: Vec<&str> = sfc.lines().filter(|l| l.contains("key")).collect();
        assert!(
            wrapper.contains(":key="),
            "fallback wrapper must hoist the child key:
{}
--- keylines: {:?}",
            wrapper,
            keylines
        );
    }

    /// PLAN-677 T-05b: `json.to_value(<api call>)` 在 vue 轨必须发
    /// JSON.parse——api 客户端 response.json() 解出的是 wire 上的 JSON
    /// 字符串本体（api fn 声明返回 str），恒等映射会把串赋给 Array
    /// 声明态（消费方 auto-edit tree_nodes 崩实测）。
    #[test]
    fn plan677_json_to_value_of_api_call_wraps_parse() {
        let src = r#"
use back.api: tree
store S {
    model {
        var nodes list = []
        var ws str = ""
    }
    on {
        .Load -> {
            .nodes = json.to_value(tree(.ws, 4))
        }
    }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::StoreDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("store decl");
        let mut store =
            crate::aura::extract::extract_store_from_decl(&decl).expect("extract store");
        // 文件级 use back.api 由真实管线注入 api_imports；本测试直填。
        store.api_imports = vec!["tree".to_string()];
        let out = crate::ui_gen::vue::VueGenerator::generate_store_composable(&store);
        assert!(
            out.contains("JSON.parse(await tree("),
            "json.to_value(api call) must emit JSON.parse:
{}",
            out
        );
        assert!(
            !out.contains("= await tree(ws_dir.value, 4);"),
            "raw string assignment must not survive:
{}",
            out
        );
    }

    /// PLAN-677 T-05: code_editor 动态 key（循环内表达式）发
    /// :editor-key 绑定，供 editorBridge 按键寻址；字面量 key 维持
    /// data-editor-key 既有契约。
    #[test]
    fn test_plan677_code_editor_emits_dynamic_editor_key() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget Ed {
    msg M { }
    model {
        var tabs Array<str> = []
        var active str = ""
    }
    view {
        col {
            for i, t in .tabs {
                if t == .active {
                    code_editor (key: t.key, lang: "auto", style: "flex-1 w-full") {
                        content: t.src
                    }
                }
            }
        }
    }
    on { .X -> { } }
}
"#,
        );
        assert!(
            sfc.contains(":editor-key="),
            "dynamic key must emit :editor-key binding:
{}",
            sfc
        );
        let sfc2 = gen_sfc_from_widget_src_shadcn(
            r#"
widget Ed2 {
    msg M { }
    model { var doc str = "" }
    view {
        col {
            code_editor (key: "tab-main", lang: "auto") {
                content: .doc
            }
        }
    }
    on { .X -> { } }
}
"#,
        );
        assert!(
            sfc2.contains("data-editor-key=\"tab-main\""),
            "literal key keeps data-editor-key contract:
{}",
            sfc2
        );
    }

    #[test]
    fn test_plan677_single_if_loop_wraps_in_template_not_div() {
        // PLAN-677 T-01: 单 Conditional 体循环的 fallback 包装必须透明
        // (<template v-for>)——div 包装盒斩断 flex 高度链(auto-edit
        // 编辑器循环实测塌缩)。多语句体维持 div 包装(块级布局语义)。
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Ed {
    msg M { }
    model {
        var tabs Array<str> = []
        var active str = ""
    }
    view {
        col {
            for i, t in .tabs {
                if t == .active {
                    text (text: t, key: t) {}
                }
            }
        }
    }
    on { .X -> { } }
}
"#,
        );
        assert!(
            sfc.contains("<template v-for=\"(t, i) in"),
            "single-if loop must emit transparent <template v-for>:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("<div v-for="),
            "single-if loop must not emit a div wrapper:\n{}",
            sfc
        );

        // 负例:多语句体(two ifs)维持 div 包装——Plan 008 既有语义。
        let sfc2 = gen_sfc_from_widget_src(
            r#"
widget Rows {
    msg Msg { Go(str) }
    model { var items Array<str> = [] }
    view {
        col {
            for e in .items {
                if e.kind == "text" {
                    text (text: e.label, key: e.key) {}
                }
                if e.kind == "toggle" {
                    text (text: e.label, key: e.key) {}
                }
            }
        }
    }
    on {
        .Go(k) -> { }
    }
}
"#,
        );
        assert!(
            sfc2.contains("<div v-for=\"e in"),
            "multi-statement body must keep the div wrapper:\n{}",
            sfc2
        );
    }

    #[test]
    fn test_bare_value_input_folds_to_vmodel_without_mint_noise() {
        // Plan 448 C: a bare `value: .x` input (no oninput) mints an empty
        // `__bind_oninput_<n>` auto-sync handler for the iced/VM and native
        // Rust paths. On Vue the v-model fold already realizes the sync, so
        // the mint must leave NO trace: no @input attr, no stub function,
        // no defineEmits entry, no trailing emit.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var value str = "" }
    view { col { input { placeholder: "search", value: .value } } }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"value\""),
            "bare value folds to v-model:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("__bind_"),
            "auto-sync mint must be invisible on Vue:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("@input"),
            "no redundant input listener:\n{}",
            sfc
        );
        assert!(!sfc.contains("defineEmits"), "nothing to emit:\n{}", sfc);
        assert!(!sfc.contains("TODO"), "no empty stub fn:\n{}", sfc);

        // Explicit oninput keeps its handler attr (and still v-model folds).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    msg { Changed }
    model { var value str = "" }
    view { col { input { value: .value, oninput: .Changed } } }
    on { .Changed -> { .value = .value } }
}
"#,
        );
        assert!(
            sfc.contains("v-model=\"value\""),
            "explicit path still folds:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@input=\"Changed\""),
            "explicit handler attr kept:\n{}",
            sfc
        );
    }

    #[test]
    fn test_grid_dynamic_cols_gap_inline_style() {
        // Plan 448 I: literal cols/gap keep their static Tailwind classes;
        // dynamic expressions bind the equivalent inline styles instead —
        // a dynamic `grid-cols-${n}` class would be invisible to Tailwind
        // JIT scanning.
        // The real app path (`auto build` → shadcn generator) owns the grid
        // arm; the plain generator passes cols through as a dead attr
        // (pre-existing, see §I 边界).
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    model {
        var cols int = 3
        var gap int = 2
    }
    view {
        col {
            grid { grid-item { text "a" {} } cols: 7, gap: 0 }
            grid { grid-item { text "b" {} } cols: .cols, gap: .gap }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("grid-cols-7"),
            "literal cols keep the static class:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("grid-cols-{{"),
            "no dynamic Tailwind class template:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'grid-template-columns: repeat(' + (cols) + ', minmax(0, 1fr))'"),
            "dynamic cols binds inline repeat style:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'gap: ' + (gap) + 'px'"),
            "dynamic gap binds inline px style:\n{}",
            sfc
        );
    }

    #[test]
    fn test_style_array_and_class_safe_concat() {
        // Plan 448 D: `style: ["base", if … {…} else {…}, .ref]` is array-of-
        // class-parts — every element joins into ONE :class binding (literals
        // quoted, conditionals as ternaries, refs as bound values; empty
        // parts filtered). A manual concat of literals/conditionals is
        // class-safe by structure → :class. A concat carrying refs keeps the
        // Plan 043 H5 inline-CSS reading → :style.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var dark bool = true, cls str = "extra", red str = "34" }
    view {
        col {
            text "a" { style: ["items-center gap-1", if .dark { " border-zinc-800" } else { " border-gray-200" }] }
            text "b" { style: "py-3 border-b" + if .dark { " z-800" } else { " g-200" } }
            text "c" { style: ["base", .cls] }
            text "d" { style: "color: rgb(" + .red + ")" }
            row (style: ["row-base", if .dark { "" } else { "opacity-40" }]) { text "e" {} }
        }
    }
}
"#,
        );
        // Array form: joined :class with filter(Boolean).
        assert!(
            sfc.contains(".filter(Boolean).join(' ')"),
            "array style must join into one :class:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'items-center gap-1'"),
            "literal array part quoted:\n{}",
            sfc
        );
        // Layout primitive (row, paren form) joins the same way.
        assert!(
            sfc.contains("'row-base'"),
            "row paren-form array part joins:\n{}",
            sfc
        );
        assert!(
            sfc.matches(".filter(Boolean).join(' ')").count() >= 2,
            "both plain-element and layout paths join:\n{}",
            sfc
        );
        // Ref part inside the array joins too (array is explicitly class
        // semantics — refs allowed).
        assert!(
            sfc.contains("'base', cls"),
            "state ref part inside array join:\n{}",
            sfc
        );
        // Class-safe manual concat → :class (never :style).
        assert!(
            sfc.contains(":class=\"'py-3 border-b' +"),
            "class-safe concat routes to :class:\n{}",
            sfc
        );
        // CSS concat with a ref operand stays :style (H5 semantics).
        assert!(
            sfc.contains(":style=\"'color: rgb(' +"),
            "ref-carrying concat stays :style:\n{}",
            sfc
        );
    }

    fn gen_sfc_from_widget_src(src: &str) -> String {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new();
        gen.generate(&widget).expect("generate SFC")
    }

    /// Same as gen_sfc_from_widget_src, but with PLAN-646 select markers
    /// force-enabled via the explicit builder（不走 env——单进程档串扰，
    /// 见 with_select_markers 注）。
    fn gen_sfc_from_widget_src_markers(src: &str) -> String {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new().with_select_markers(true);
        gen.generate(&widget).expect("generate SFC")
    }

    /// PLAN-018：`icon (name: "iconfile:system-monitor")` → 双主题位图
    /// <img> 对（浅表常显 + 深表 .dark 切换）+ SFC 追加切换规则；
    /// 纯 lucide 名照旧推导组件不受影响。
    #[test]
    fn plan018_iconfile_dual_theme_bitmap() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget IconProbe {
    view {
        col {
            icon (name: "iconfile:system-monitor", size: 44)
            icon (name: "calculator")
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("src=\"/icons/light/system-monitor.png\""),
            "浅表 img: {sfc}"
        );
        assert!(
            sfc.contains("src=\"/icons/dark/system-monitor.png\""),
            "深表 img: {sfc}"
        );
        assert!(
            sfc.contains("app-icon-light") && sfc.contains("app-icon-dark"),
            "主题切换类: {sfc}"
        );
        assert!(
            sfc.contains(".dark .app-icon-light { display: none; }"),
            "SFC 切换规则: {sfc}"
        );
        assert!(
            !sfc.contains("iconfile:system-monitor\n") || !sfc.contains("<Iconfile"),
            "iconfile 前缀不得泄入 Lucide 推导: {sfc}"
        );
        assert!(sfc.contains("<Calculator"), "lucide 组件照旧: {sfc}");
    }

    /// PLAN-646 T-07: Select Anything DOM 标记——通用元素注入
    /// data-auto-tag / data-auto-id / data-auto-span；span 值为 off:len
    /// 数字形态（与 AuraNode.span 字节区间一致）；text 子节点不注入
    /// （归父元素）。
    #[test]
    fn plan646_vue_emits_data_auto_markers() {
        // fix-ui-tier：改走显式 builder 开关（原 set_var AUTOUI_SELECT_MARKERS
        // 依赖 nextest 每测试一进程的隔离；cargo tu 单进程下串扰同进程全部
        // 生成器构造——a2vue golden 23 测齐红实录）。dev 运行面的 env 读取
        // 路径（构造器默认）保持不变、由生产侧另行守护。
        let sfc = gen_sfc_from_widget_src_markers(
            r#"
widget SelectMarkers {
    view {
        col {
            text "hello"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("data-auto-tag=\"col\""),
            "col tag marker:
{sfc}"
        );
        assert!(
            sfc.contains("data-auto-id=\"aura_"),
            "debug_id marker:
{sfc}"
        );
        assert!(
            sfc.contains("data-auto-src=\"app\""),
            "source stem marker:
{sfc}"
        );
        // span 形态 off:len（数字:数字）
        let span_vals: Vec<String> = sfc
            .split("data-auto-span=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next().map(str::to_string))
            .collect();
        assert!(
            !span_vals.is_empty(),
            "span marker 存在:
{sfc}"
        );
        for v in &span_vals {
            let (off, len) = v.split_once(':').unwrap_or_else(|| {
                panic!(
                    "span 必须为 off:len 形态, got {v}:
{{sfc}}"
                )
            });
            assert!(
                !off.is_empty()
                    && off.bytes().all(|b| b.is_ascii_digit())
                    && !len.is_empty()
                    && len.bytes().all(|b| b.is_ascii_digit()),
                "span off:len 必须全数字, got {v}"
            );
        }
        // `text "hello" {}` 是 tag="text" 的 Element（有自己的 span）——
        // 照常注入；不注入的是 AuraNode::Text（DOM 文本，不经 Element 臂）。
        assert!(
            sfc.contains("data-auto-tag=\"text\""),
            "text 元素自身照常注入:
{sfc}"
        );
    }

    /// Same as gen_sfc_from_widget_src, but in shadcn-vue mode (real widgets
    /// compile with `render: "vue"` + shadcn enabled; dialog/input arms only
    /// run in this mode).
    fn gen_sfc_from_widget_src_shadcn(src: &str) -> String {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new_shadcn();
        gen.generate(&widget).expect("generate SFC")
    }

    /// Plan 482: shadcn 模式 nav 组件族 —— NavItem/NavGroup 脚手架组件 +
    /// nav(search:) 内联搜索行 + 契约 import。
    #[test]
    fn test_nav_family_shadcn_sfc() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget NavDemo {
    model {
        var cur str = "chats"
        var q str = ""
    }
    view {
        nav (search: true, search_value: .q, onsearch: .SearchChanged, search_placeholder: "Search...", style: "flex flex-col gap-1 p-2") {
            nav-group (label: "Workspace", collapsible: true) {
                nav-item (to: "/chats", icon: "message-square", label: "Chats", badge: "3")
                nav-item (onclick: .ShowPlans, active: .cur == "plans", icon: "list-todo", label: "Plans")
            }
        }
    }
    on {
        .SearchChanged(q) -> { }
        .ShowPlans -> { }
    }
}
"##,
        );
        assert!(
            sfc.contains("<NavItem"),
            "NavItem 组件发射:
{sfc}"
        );
        assert!(
            sfc.contains("<NavGroup"),
            "NavGroup 组件发射:
{sfc}"
        );
        assert!(
            sfc.contains("from '@/components/ui/nav'"),
            "契约 import:
{sfc}"
        );
        assert!(
            sfc.contains("NavItem"),
            "NavItem named import:
{sfc}"
        );
        assert!(
            sfc.contains(":icon-comp=\"MessageSquare\""),
            "lucide 图标组件:
{sfc}"
        );
        assert!(
            sfc.contains("to=\"/chats\""),
            "路由 to:
{sfc}"
        );
        assert!(
            sfc.contains(":active="),
            "active 绑定:
{sfc}"
        );
        assert!(
            sfc.contains("nav-search"),
            "搜索行:
{sfc}"
        );
        assert!(
            sfc.contains(":value=\"q\""),
            "搜索 :value 绑定:
{sfc}"
        );
        assert!(
            sfc.contains("@input="),
            "onsearch 处理器:
{sfc}"
        );
        assert!(
            sfc.contains(":collapsible=\"true\""),
            "NavGroup 可折叠:
{sfc}"
        );
        assert!(
            sfc.contains("MessageSquare"),
            "lucide import 收集:
{sfc}"
        );
    }

    /// Plan 563 T6: canvas 元素生成 —— `<canvas ref>` + pen 三包装
    /// (mousedown 内联 + document mousemove/mouseup)+ redraw(watch
    /// deep + onMounted)+ 坐标换算(coords 逻辑幅面)。
    #[test]
    fn plan563_canvas_element_sfc() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Sketch {
    msg { PenStart(float, float), PenMove(float, float), PenEnd(float, float) }
    model {
        var strokes_pts = ["10,20|30,40"]
        var strokes_meta = ["#111827,3,0"]
    }
    on {
        .PenStart(x, y) -> { }
        .PenMove(x, y) -> { }
        .PenEnd(x, y) -> { }
    }
    view {
        canvas (scene: .strokes, coords: "400x300", clear: "#ffffff", onpenstart: .PenStart, onpenmove: .PenMove, onpenend: .PenEnd) {
            style: "w-[400px] h-[300px] border"
        }
    }
}
"##,
        );
        assert!(
            sfc.contains("<canvas ref=\"__canvas_ref_0\""),
            "<canvas ref> 模板:\n{sfc}"
        );
        assert!(
            sfc.contains("@mousedown=\"__canvasDown_0\""),
            "mousedown 内联起笔:\n{sfc}"
        );
        assert!(
            sfc.contains("function __canvasRedraw_0()"),
            "redraw 函数:\n{sfc}"
        );
        assert!(
            sfc.contains("strokes_pts.value"),
            "redraw 读 pts 双表:\n{sfc}"
        );
        assert!(
            sfc.contains("/ r.width * 400"),
            "coords 逻辑换算(400x300):\n{sfc}"
        );
        assert!(
            sfc.contains("ctx.lineCap = 'round'"),
            "round 线帽规约:\n{sfc}"
        );
        assert!(sfc.contains("__canvasIsOut_0"), "出界即收笔检查:\n{sfc}");
        assert!(sfc.contains("e.buttons & 1"), "buttons&1 按下门控:\n{sfc}");
        assert!(
            sfc.contains("document.addEventListener('mousemove', __canvasMove_0)"),
            "document mousemove 挂载:\n{sfc}"
        );
        assert!(
            sfc.contains("document.addEventListener('mouseup', __canvasUp_0)"),
            "document mouseup 挂载:\n{sfc}"
        );
        assert!(
            sfc.contains("watch([strokes_pts, strokes_meta],"),
            "watch deep 双表(T1b 先例):\n{sfc}"
        );
        assert!(
            sfc.contains("PenStart(x, y)"),
            "penstart 坐标实参调用:\n{sfc}"
        );
        assert!(
            sfc.contains("ref<HTMLCanvasElement | null>"),
            "canvas ref 真类型:\n{sfc}"
        );
    }

    /// PLAN-617 T-07（AC-05..AC-09、AC-12 的生成器侧）：声明了受控下行 prop 的
    /// `video` → ref + 下行 apply + watchEffect + 上行原生监听包装；契约名与单位
    /// 和 `ui/mpv/contract.rs` 同形（`volume` 作者面 0..100 → 元素 0..1，
    /// `position` 秒且「目标变化才 seek」）。
    #[test]
    fn test_controlled_video_contract_sfc() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Player {
    msg { OnTime(float), OnDuration(float), OnPlayState(bool), OnEnded, OnMediaError(str), TogglePlay }
    model {
        var current_url str = "/api/media/stream/a"
        var is_playing bool = false
        var seek_target float = 0.0
        var volume int = 80
        var is_muted bool = false
        var playback_rate float = 1.0
    }
    on {
        .OnTime(t) -> { }
        .OnDuration(d) -> { }
        .OnPlayState(p) -> { }
        .OnEnded -> { }
        .OnMediaError(m) -> { }
        .TogglePlay -> { .is_playing = !.is_playing }
    }
    view {
        video {
            src: .current_url
            controls: false
            paused: .is_playing == false
            position: .seek_target
            volume: .volume
            muted: .is_muted
            rate: .playback_rate
            ontimeupdate: .OnTime($0)
            onloadedmetadata: .OnDuration($0)
            onplaystatechange: .OnPlayState($0)
            onended: .OnEnded
            onmediaerror: .OnMediaError($0)
            onclick: .TogglePlay
        }
    }
}
"##,
        );
        // 模板：ref + 透传属性在位，受控下行 prop **不得**作为元素属性出现
        assert!(
            sfc.contains("<video ref=\"__video_ref_0\""),
            "受控 video ref:\n{sfc}"
        );
        assert!(
            sfc.contains(":src=\"current_url\""),
            "src 仍为透传绑定:\n{sfc}"
        );
        assert!(
            sfc.contains(":controls=\"false\""),
            "controls 仍为透传绑定:\n{sfc}"
        );
        assert!(
            !sfc.contains(":paused="),
            "受控 prop 不得打到只读 DOM 属性:\n{sfc}"
        );
        assert!(
            !sfc.contains(":volume="),
            "受控 prop 不得打到单位不同的属性:\n{sfc}"
        );
        // 上行：契约事件换成生成器发射的原生监听包装
        assert!(
            sfc.contains("@timeupdate=\"__videoTime_0\""),
            "timeupdate 包装:\n{sfc}"
        );
        assert!(
            sfc.contains("@loadedmetadata=\"__videoMeta_0\""),
            "loadedmetadata 包装:\n{sfc}"
        );
        assert!(
            sfc.contains("function __videoSync_0()"),
            "元素侧重入点（换源后元素自行 paused=true，必须有重推通道）:\n{sfc}"
        );
        assert!(
            sfc.contains("__videoSync_0()") && sfc.contains("__videoLast_0 = ["),
            "loadedmetadata 里重推最近一次下行值:\n{sfc}"
        );
        assert!(
            sfc.contains("@play=\"__videoState_0\""),
            "play 包装:\n{sfc}"
        );
        assert!(
            sfc.contains("@pause=\"__videoState_0\""),
            "pause 包装:\n{sfc}"
        );
        assert!(
            sfc.contains("@ended=\"__videoEnded_0\""),
            "ended 包装:\n{sfc}"
        );
        assert!(
            sfc.contains("@error=\"__videoError_0\""),
            "error 包装:\n{sfc}"
        );
        // 契约键之外的事件保留通用映射
        assert!(
            sfc.contains("@click=\"TogglePlay\""),
            "onclick 走通用映射:\n{sfc}"
        );
        // script：ref 真类型 + watchEffect 下行 + 单位换算 + seek 目标语义
        assert!(
            sfc.contains("ref<HTMLVideoElement | null>"),
            "video ref 真类型:\n{sfc}"
        );
        assert!(
            sfc.contains("watchEffect(() =>"),
            "下行 watchEffect:\n{sfc}"
        );
        assert!(
            sfc.contains("onMounted(() => {")
                && sfc.find("onMounted(() => {").unwrap() < sfc.find("watchEffect(() =>").unwrap(),
            "watchEffect 必须包在 onMounted 内（避开 store facade TDZ）:\n{sfc}"
        );
        assert!(
            sfc.contains("__videoLast_0 = [is_playing.value == false, seek_target.value, volume.value, is_muted.value, playback_rate.value, current_url.value]"),
            "下行实参取脚本域值:\n{sfc}"
        );
        assert!(sfc.contains("/ 100"), "volume 0..100 → 0..1:\n{sfc}");
        assert!(
            sfc.contains("__videoSeek_0"),
            "position 走「目标变化才 seek」latch:\n{sfc}"
        );
        assert!(
            sfc.contains("el.currentTime = position"),
            "seek 落到元素:\n{sfc}"
        );
        assert!(
            sfc.contains("OnTime(el.currentTime)"),
            "上行取真实 currentTime:\n{sfc}"
        );
        assert!(
            sfc.contains("OnDuration(el.duration)"),
            "上行取真实 duration:\n{sfc}"
        );
        assert!(
            sfc.contains("OnPlayState(!el.paused)"),
            "playstate 由元素合成:\n{sfc}"
        );
        assert!(
            sfc.contains("OnMediaError(msg)"),
            "mediaerror 带真实文案:\n{sfc}"
        );
        // 未声明 onaudiotrack → 音轨探针一行都不发射（AC-16b 的兼容面）
        assert!(
            !sfc.contains("__videoAudio_0"),
            "未声明 onaudiotrack 不得有探针包装:\n{sfc}"
        );
        assert!(
            !sfc.contains("__videoAudioDone_0"),
            "未声明 onaudiotrack 不得有探针状态:\n{sfc}"
        );
    }

    /// PLAN-617 T-11 / AC-16(b)：声明 `onaudiotrack` 的受控 video → 音轨可用性
    /// 探针（复用 timeupdate 节拍；播放 >1s 才判定；Chromium 专有探针缺失时
    /// 不调用 handler；换片复位一次性标志）。
    #[test]
    fn test_controlled_video_audio_probe_sfc() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Player {
    msg { OnAudioTrack(bool), TogglePlay }
    model {
        var current_url str = "/api/media/stream/a"
        var is_playing bool = false
        var seek_target float = 0.0
    }
    on {
        .OnAudioTrack(ok) -> { }
        .TogglePlay -> { .is_playing = !.is_playing }
    }
    view {
        video {
            src: .current_url
            paused: .is_playing == false
            position: .seek_target
            onaudiotrack: .OnAudioTrack($0)
        }
    }
}
"##,
        );
        assert!(
            sfc.contains("@timeupdate=\"__videoAudio_0\""),
            "探针挂到 timeupdate 节拍:\n{sfc}"
        );
        assert!(
            sfc.contains("let __videoAudioDone_0 = false"),
            "一次性标志声明:\n{sfc}"
        );
        assert!(
            sfc.contains("__videoAudioDone_0 = false\n  }"),
            "换片（src 变化分支）复位标志:\n{sfc}"
        );
        assert!(
            sfc.contains("if (__videoAudioDone_0 || el.paused || el.currentTime <= 1.0) return"),
            "暂停/前 1s 不构成结论:\n{sfc}"
        );
        assert!(
            sfc.contains("if (typeof probe !== 'number') return"),
            "非 Chromium（探针缺失）不主张任何结论:\n{sfc}"
        );
        assert!(
            sfc.contains("OnAudioTrack(probe > 0)"),
            "上行 bool = 实测是否解出音频字节:\n{sfc}"
        );
    }

    /// ontimeupdate 与 onaudiotrack **同时**声明：Vue 模板不允许同一元素出现
    /// 两个 `@timeupdate`（Duplicate attribute 编译错，e2e 实测抓到）——
    /// 模板只挂 `__videoTime_`，探针由其链尾调用。
    #[test]
    fn test_controlled_video_audio_probe_chains_with_time() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Player {
    msg { OnTime(float), OnAudioTrack(bool) }
    model {
        var current_url str = "/api/media/stream/a"
        var is_playing bool = false
        var seek_target float = 0.0
    }
    on {
        .OnTime(t) -> { }
        .OnAudioTrack(ok) -> { }
    }
    view {
        video {
            src: .current_url
            paused: .is_playing == false
            position: .seek_target
            ontimeupdate: .OnTime($0)
            onaudiotrack: .OnAudioTrack($0)
        }
    }
}
"##,
        );
        assert_eq!(
            sfc.matches("@timeupdate=").count(),
            1,
            "timeupdate 属性只能出现一次:\n{sfc}"
        );
        assert!(
            sfc.contains("@timeupdate=\"__videoTime_0\""),
            "属性归 ontimeupdate 包装:\n{sfc}"
        );
        assert!(
            !sfc.contains("@timeupdate=\"__videoAudio_0\""),
            "探针不得重复占用模板属性:\n{sfc}"
        );
        assert!(
            sfc.contains("function __videoAudio_0(e: Event)"),
            "探针函数仍在:\n{sfc}"
        );
        assert!(
            sfc.contains("OnTime(el.currentTime)\n  __videoAudio_0(e)"),
            "探针由 __videoTime_ 链尾调用:\n{sfc}"
        );
    }

    /// PLAN-617 T-07 **兼容性约束**（AC-12）：未声明任何受控 prop 的 `video`
    /// 生成结果必须与引入契约之前逐字节一致（裸 `<video :src :class />`，
    /// 无 ref、无 watchEffect、无监听包装）。019-video-app 等既有示例靠这条
    /// 断言防回归。
    #[test]
    fn test_uncontrolled_video_is_byte_identical() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget Bare {
    model {
        var current_url str = "https://example.invalid/a.mp4"
    }
    on {
        .Noop -> { }
    }
    view {
        video {
            src: .current_url
            controls: true
            class: "w-full"
        }
    }
}
"##,
        );
        assert!(
            sfc.contains("<video class=\"w-full\" :controls=\"true\" :src=\"current_url\" />"),
            "裸 video 模板:\n{sfc}"
        );
        assert!(!sfc.contains("__video_ref_"), "不得凭空多出 ref:\n{sfc}");
        assert!(
            !sfc.contains("__videoApply_"),
            "不得凭空多出下行控制器:\n{sfc}"
        );
        assert!(
            !sfc.contains("watchEffect"),
            "不得凭空多出 watchEffect:\n{sfc}"
        );
    }

    /// Plan 482: 非 shadcn 模式（os-config 形态）—— 内联契约标记 + active
    /// 三元 + emoji icon + desc 行 + 折叠组绑定 open/ontoggle。
    #[test]
    fn test_nav_family_inline_sfc() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget NavPlain {
    model {
        var active_id str = ""
        var group_open bool = true
        var q str = ""
    }
    view {
        nav (search: true, search_value: .q, onsearch: .SearchChanged) {
            nav-group (label: "Network", collapsible: true, open: .group_open, ontoggle: .ToggleGroup) {
                nav-item (onclick: .Select("net"), active: .active_id == "net", icon: "🔌", label: "Network", desc: "Adapters and proxy")
            }
        }
    }
    on {
        .SearchChanged(q) -> { }
        .ToggleGroup -> { }
        .Select(id) -> { }
    }
}
"##,
        );
        assert!(!sfc.contains("<NavItem"), "非 shadcn 不用脚手架组件");
        assert!(
            sfc.contains("nav-item flex w-full"),
            "契约基类内联:
{sfc}"
        );
        assert!(
            sfc.contains(":class="),
            "active 三元:
{sfc}"
        );
        assert!(
            sfc.contains("bg-primary/10"),
            "active 块:
{sfc}"
        );
        assert!(
            sfc.contains("hover:bg-accent"),
            "hover 块:
{sfc}"
        );
        assert!(
            sfc.contains("@click=\"Select('net')\"")
                || sfc.contains("@click=\"Select(\"net\")\"")
                || sfc.contains("@click=\"Select("),
            "click 处理器:
{sfc}"
        );
        assert!(
            sfc.contains("🔌"),
            "emoji icon:
{sfc}"
        );
        assert!(
            sfc.contains("text-xs text-muted-foreground"),
            "desc 次行:
{sfc}"
        );
        assert!(
            sfc.contains("v-show="),
            "折叠组 v-show:
{sfc}"
        );
        assert!(
            sfc.contains("▾"),
            "折叠 chevron 文本字形:
{sfc}"
        );
        assert!(
            sfc.contains("nav-search"),
            "搜索行:
{sfc}"
        );
    }

    // ====================================================================
    // v-model contracts (Item 2): a widget can declare a true Vue v-model
    // target by pairing a `modelValue` prop with a quoted msg variant whose
    // name is a literal Vue emit name (`msg Msg { "update:modelValue"(str) }`).
    // ====================================================================

    /// Child side: quoted msg variant "update:modelValue" → typed defineEmits
    /// with a QUOTED key, a sanitized forwarding handler fn, and emit() with
    /// the verbatim event name.
    #[test]
    fn test_vmodel_contract_child_quoted_emit() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget TextField(modelValue: str) {
    msg Msg { "update:modelValue"(str) }
    view {
        col {
            input { value: .modelValue, oninput: ."update:modelValue" }
        }
    }
    on {
        ."update:modelValue"(v) -> { }
    }
}
"#,
        );
        assert!(
            sfc.contains("modelValue: string"),
            "modelValue prop declared:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'update:modelValue': [string]"),
            "quoted emit key in defineEmits:\n{}",
            sfc
        );
        assert!(
            sfc.contains("emit('update:modelValue', v)"),
            "emit with verbatim name + payload:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function update_modelValue(v: any): void"),
            "sanitized handler fn:\n{}",
            sfc
        );
        // Prop-backed value: one-way :modelValue down (never v-model on a prop),
        // @update:modelValue up — a controlled v-model contract.
        assert!(
            sfc.contains(":modelValue=\"modelValue\""),
            "one-way :modelValue binding:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@update:modelValue=\"update_modelValue\""),
            "update event wiring:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("v-model=\"modelValue\""),
            "no v-model on a read-only prop:\n{}",
            sfc
        );
    }

    /// Parent side (manual wiring, no sugar): a DSL parent can already bind a
    /// v-model contract via the modelValue prop + a quoted custom event key
    /// `on "update:modelValue":` (Plan-367 quoted event names).
    #[test]
    fn test_vmodel_contract_parent_manual_wiring() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    msg Msg { NameChanged(str) }
    model { var name str = "" }
    view {
        col {
            TextField(modelValue: .name) {
                on "update:modelValue": .NameChanged
            }
        }
    }
    on {
        .NameChanged(v) -> { .name = v }
    }
}
"#,
        );
        assert!(
            sfc.contains(":modelValue=\"name\""),
            "modelValue prop down:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@update:modelValue=\"NameChanged\""),
            "quoted update event up:\n{}",
            sfc
        );
    }

    /// jade 5.3c self-heal lock: a quoted component event whose emit carries
    /// TWO args (e.g. the editor's `open-wiki-link` (title, blockId)) wires a
    /// bare handler reference, so Vue passes BOTH args to the handler — and
    /// the multi-param handler (gap 37b) keeps both params in scope. Locks
    /// the AutoJade editor_tab migration off the prop-callback channel.
    #[test]
    fn test_quoted_component_event_multi_arg_handler() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget EditorTab {
    msg Msg { OpenWikiLink }
    model { var tabsStore list = [] }
    view {
        col {
            EditorShell {
                on "open-wiki-link": .OpenWikiLink
            }
        }
    }
    on {
        .OpenWikiLink(title, block_id) -> {
            log(title)
            log(block_id)
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("@open-wiki-link=\"OpenWikiLink\""),
            "quoted event wires a bare handler ref (all emit args flow):\n{}",
            sfc
        );
        assert!(
            sfc.contains("function OpenWikiLink(title: any, block_id: any)"),
            "both handler params in the generated signature:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Dynamic components (dyn) and reactive watchers (watch).
    // ====================================================================

    /// dyn (.item.icon) { size: 16, class: "..." } inside a for loop →
    /// <component :is="(item.icon) as any" :size="16" class="..." />.
    #[test]
    fn test_dyn_component_in_for_loop() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget SlashMenu {
    model { var items list = [] }
    view {
        col {
            for item in .items {
                dyn (.item.icon) { size: 16, class: "w-4 h-4 shrink-0" }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#":is="(item.icon) as any""#),
            "dyn :is binding:\n{}",
            sfc
        );
        assert!(
            sfc.contains(r#"v-for="item in items""#),
            "v-for on dyn element:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("/ v-for"),
            "v-for not inserted after self-closing slash:\n{}",
            sfc
        );
        assert!(sfc.contains(r#":size="16""#), "extra prop bound:\n{}", sfc);
        assert!(
            sfc.contains(r#"class="w-4 h-4 shrink-0""#),
            "static class:\n{}",
            sfc
        );
    }

    /// dyn without parentheses: is as a plain prop; model-field source.
    #[test]
    fn test_dyn_component_is_prop_model_field() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget IconBox {
    model { var current_icon str = "x" }
    view {
        col {
            dyn { is: .current_icon }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<component :is="(current_icon) as any" />"#),
            "is prop form:\n{}",
            sfc
        );
    }

    /// watch { .computed -> { ... } } → watch(filtered, () => { ... }) with
    /// on-handler body conventions (.field = state access via .value).
    #[test]
    fn test_watch_computed_source() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget SlashMenu {
    model {
        var query str = ""
        var selected_index int = 0
    }
    computed {
        filtered => query
    }
    view { col { text "hi" } }
    watch {
        .filtered -> { .selected_index = 0 }
    }
}
"#,
        );
        assert!(
            sfc.contains("import { ref, computed, watch } from 'vue'"),
            "watch imported (defineModel is a macro, never imported — plan 015):\n{}",
            sfc
        );
        assert!(
            sfc.contains("watch(filtered, () => {"),
            "watch call:\n{}",
            sfc
        );
        assert!(
            sfc.contains("selected_index.value = 0"),
            "handler body transpiled like on block:\n{}",
            sfc
        );
    }

    /// Prop source needs a getter; .immediate/.deep become watch options;
    /// multiple sources become an array.
    #[test]
    fn test_watch_prop_source_and_modifiers() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Scrollbar(ratio: int, viewport_h: int) {
    model { var thumb_h int = 0 }
    view { col { text "hi" } }
    watch {
        .ratio, .viewport_h.immediate -> { .thumb_h = .viewport_h }
        .ratio.deep -> { .thumb_h = 0 }
    }
}
"#,
        );
        assert!(
            sfc.contains("watch([() => props.ratio, () => props.viewport_h], () => {"),
            "multi-source prop getters:\n{}",
            sfc
        );
        assert!(
            sfc.contains("{ immediate: true }"),
            "immediate option:\n{}",
            sfc
        );
        assert!(sfc.contains("{ deep: true }"), "deep option:\n{}", sfc);
        assert!(
            sfc.contains("thumb_h.value = props.viewport_h"),
            "prop read in body:\n{}",
            sfc
        );
    }

    // ====================================================================
    // defineExpose: widget-level `expose { ... }` block.
    // ====================================================================

    /// expose { .fit } where `fit` is an imported TS fn →
    /// defineExpose({ fit }) in <script setup>.
    #[test]
    fn test_expose_single_imported_fn() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget GraphView {
    use {
        fn: fitGraph from "src/front/utils/graph.ts"
    }
    model { var zoom int = 1 }
    view { col { text "hi" } }
    expose {
        .fitGraph
    }
}
"#,
        );
        assert!(
            sfc.contains("defineExpose({ fitGraph })"),
            "defineExpose with imported fn:\n{}",
            sfc
        );
    }

    /// Multiple members on one line and across lines; an exposed `on`
    /// handler is emitted even though the template never references it.
    #[test]
    fn test_expose_multiple_members_and_handler() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget GraphView {
    msg Msg { Fit, Relayout }
    model { var zoom int = 1 }
    computed { doubled => .zoom * 2 }
    view { col { text "hi" } }
    on {
        .Fit -> { .zoom = 1 }
        .Relayout -> { .zoom = .zoom }
    }
    expose {
        .Fit, .Relayout
        .doubled
    }
}
"#,
        );
        assert!(
            sfc.contains("defineExpose({ Fit, Relayout, doubled })"),
            "defineExpose with handler + computed names:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function Fit()"),
            "exposed handler emitted despite no template use:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function Relayout()"),
            "second exposed handler emitted:\n{}",
            sfc
        );
    }

    /// Expose a model state var and a template ref (ref: "graphEl" in the
    /// view): both exist as script-setup refs; defineExpose exposes the ref
    /// objects (Vue's expose proxy unwraps them on parent access).
    #[test]
    fn test_expose_state_and_template_ref() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Editor {
    model { var content str = "" }
    view {
        col {
            ref: "rootEl"
            text "editor"
        }
    }
    expose {
        .content, .rootEl
    }
}
"#,
        );
        assert!(
            sfc.contains("const content = ref<string>('')"),
            "state ref exists:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const rootEl = ref<HTMLElement | null>(null)"),
            "template ref exists:\n{}",
            sfc
        );
        assert!(
            sfc.contains("defineExpose({ content, rootEl })"),
            "defineExpose with state + template ref:\n{}",
            sfc
        );
    }

    /// A widget without an expose block emits no defineExpose.
    #[test]
    fn test_no_expose_block_unchanged() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Plain {
    model { var count int = 0 }
    view { col { text "hi" } }
}
"#,
        );
        assert!(
            !sfc.contains("defineExpose"),
            "no defineExpose without expose block:\n{}",
            sfc
        );
    }

    /// Parent side: `ref: "canvasRef"` on a child component → static
    /// `ref="canvasRef"` attribute + `ref<any>` declaration, so handlers can
    /// call the child's exposed methods via `.canvasRef.method()`.
    #[test]
    fn test_component_template_ref() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    msg Msg { DoFit }
    model { var n int = 0 }
    view {
        col {
            GraphView { ref: "canvasRef" }
            button "fit" { onclick: .DoFit }
        }
    }
    on {
        .DoFit -> { .canvasRef.Fit() }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<GraphView ref="canvasRef""#),
            "static ref attr on child component:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const canvasRef = ref<any>(null)"),
            "component ref typed any:\n{}",
            sfc
        );
        assert!(
            sfc.contains("canvasRef.value!.Fit()"),
            "exposed method call through the ref:\n{}",
            sfc
        );
    }

    /// Element-level generic events: keyboard, mouse, wheel, contextmenu.
    #[test]
    fn test_generic_dom_events() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Menu {
    msg Msg { Key, Down, Move, Up, Wheel, Ctx }
    model { var x int = 0 }
    view {
        col {
            onkeydown: .Key,
            onmousedown: .Down,
            onmousemove: .Move,
            onmouseup: .Up,
            onwheel: .Wheel,
            oncontextmenu: .Ctx
        }
    }
    on {
        .Key -> { .x = 1 }
        .Down -> { .x = 2 }
        .Move -> { .x = 3 }
        .Up -> { .x = 4 }
        .Wheel -> { .x = 5 }
        .Ctx -> { .x = 6 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@keydown=\"Key\""),
            "@keydown emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mousedown=\"Down\""),
            "@mousedown emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mousemove=\"Move\""),
            "@mousemove emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mouseup=\"Up\""),
            "@mouseup emitted:\n{}",
            sfc
        );
        assert!(sfc.contains("@wheel=\"Wheel\""), "@wheel emitted:\n{}", sfc);
        assert!(
            sfc.contains("@contextmenu=\"Ctx\""),
            "@contextmenu emitted:\n{}",
            sfc
        );
    }

    /// Plan 499 M2: mouse-area onmousemove + coords → 内联箭头换算逻辑坐标
    /// (生成端=引擎层完成屏幕→逻辑换算;与 iced PointerArea 同一坐标语义)。
    /// 缺省 coords → raw px 形态。
    #[test]
    fn test_mouse_area_onmousemove_logical_coords() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Chart {
    msg Msg { PointerMove, RawMove }
    model { var x int = 0 }
    view {
        col {
            mouse-area (coords: "560x300", onmousemove: .PointerMove) {}
            mouse-area (onmousemove: .RawMove) {}
        }
    }
    on {
        .PointerMove -> { .x = 1 }
        .RawMove -> { .x = 2 }
    }
}
"#,
        );
        // PLAN-043 T9: 箭头改 currentTarget 矩形相对坐标（e.offsetX/Y 相对
        // 事件 target——mouse-area 带子元素时拖拽期退化为子元素局部坐标）。
        assert!(
            sfc.contains(
                "@mousemove=\"e => PointerMove((e.clientX - (e.currentTarget as HTMLElement).getBoundingClientRect().left) / (e.currentTarget as HTMLElement).clientWidth * 560, (e.clientY - (e.currentTarget as HTMLElement).getBoundingClientRect().top) / (e.currentTarget as HTMLElement).clientHeight * 300)\""
            ),
            "coords 换算箭头必须发射:\n{}",
            sfc
        );
        assert!(
            sfc.contains(
                "@mousemove=\"e => RawMove(e.clientX - (e.currentTarget as HTMLElement).getBoundingClientRect().left, e.clientY - (e.currentTarget as HTMLElement).getBoundingClientRect().top)\""
            ),
            "raw px 形态必须发射:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Plan 451 P2: `actions {}` 声明块——vue 侧消费（keydown 回退层 +
    // menubar/toolbar 占位合成 + enabled_if/checked_if 条件转译）。
    // ====================================================================

    const ACTIONS_SRC: &str = r#"
widget App {
    msg Msg { ActNew, ActSave, ActUndo }
    model { var n int = 0 }
    actions {
        action (id: "file.new",  handler: .ActNew,  title: "新建", icon: "file-plus",   shortcut: "Ctrl+N")
        action (id: "file.save", handler: .ActSave, title: "保存", shortcut: "Ctrl+S",
                enabled_if: ".n > 0")
        action (id: "edit.undo", handler: .ActUndo, title: "撤销", shortcut: "Ctrl+Z",
                checked_if: ".n > 0")
        menubar {
            menu (id: "file", title: "文件") {
                item (action: "file.new")
                sep
                item (action: "file.save")
            }
            menu (id: "edit", title: "编辑") {
                item (action: "edit.undo")
            }
        }
        toolbar {
            item (action: "file.new")
            sep
            item (action: "edit.undo")
        }
    }
    view {
        col {
            menubar {}
            toolbar (style: "ml-auto") {}
        }
    }
    on {
        .ActNew -> { .n = .n + 1 }
        .ActSave -> { .n = 0 }
        .ActUndo -> { .n = 0 }
    }
}
"#;

    /// Plan 464 T2: `bind {}` 块 → 全局 keydown 层（`__autoBindKeymap`）。
    /// vm 侧对应 keyboard_subscription 的 key_bindings 查找；vue 此前无该层，
    /// bind 块静默失效（011-calculator 键盘仅 vm 可用）。命名键（方向键/
    /// Escape/Tab/Enter）即使焦点在 input 上也照常派发（vm 单行 input 不捕获
    /// 这组键——palette 键盘流的前提）；单字符/Backspace 在输入框内让位给
    /// 打字（镜像 vm Captured 语义）。
    #[test]
    fn test_bind_block_keydown_layer() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget L {
    msg Msg { MoveUp, MoveDown, Pick, Dismiss, Open }
    model { var n int = 0 }
    view { col { input { value: .n } } }
    bind {
        "ArrowUp" -> .MoveUp
        "ArrowDown" -> .MoveDown
        "Enter" -> .Pick
        "Escape" -> .Dismiss
        "Ctrl+ " -> .Open
    }
    on {
        .MoveUp -> { .n = 1 }
        .MoveDown -> { .n = 2 }
        .Pick -> { .n = 3 }
        .Dismiss -> { .n = 4 }
        .Open -> { .n = 5 }
    }
}
"#,
        );
        assert!(
            sfc.contains("'ArrowDown': MoveDown"),
            "named key entry:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'Ctrl+ ': Open"),
            "ctrl+space combo entry:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.addEventListener('keydown', __autoBindKeydown)"),
            "bind keydown registered:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('keydown', __autoBindKeydown)"),
            "bind keydown removed on unmount:\n{}",
            sfc
        );
        assert!(
            sfc.contains("if (e.isComposing || e.keyCode === 229) return"),
            "IME composition guard on bind keydown:\n{}",
            sfc
        );
    }

    /// LAUNCHER-001 R-02: input onenter must not fire while IME composing.
    #[test]
    fn test_onenter_ime_guard_wraps_handler() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget L {
    msg Msg { Pick }
    model { var q str = "" }
    view { col { input { value: .q, onenter: .Pick } } }
    on { .Pick -> { .q = "x" } }
}
"#,
        );
        assert!(
            sfc.contains("isComposing"),
            "onenter handler must be IME-guarded:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@keyup.enter="),
            "onenter still maps to keyup.enter:\n{}",
            sfc
        );
        assert!(
            sfc.contains("Pick()"),
            "guarded handler must invoke Pick():\n{}",
            sfc
        );
    }

    /// 快捷键 → 全局 keydown 回退层（normalize_shortcut 判定形态：单字符键
    /// 小写、修饰键归一），onMounted/onUnmounted 成对注册。
    #[test]
    fn test_actions_shortcut_keydown_layer() {
        let sfc = gen_sfc_from_widget_src_shadcn(ACTIONS_SRC);
        assert!(
            sfc.contains("'Ctrl+n': ActNew"),
            "normalized keymap entry:\n{}",
            sfc
        );
        assert!(
            sfc.contains("'Ctrl+s': ActSave") && sfc.contains("'Ctrl+z': ActUndo"),
            "all three shortcuts normalized:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.addEventListener('keydown', __autoActionsKeydown)"),
            "keydown registered:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('keydown', __autoActionsKeydown)"),
            "keydown removed on unmount:\n{}",
            sfc
        );
        assert!(
            sfc.contains("e.preventDefault()"),
            "matched key prevents browser default:\n{}",
            sfc
        );
    }

    /// `menubar {}` 占位标签 → shadcn Menubar 组件树（trigger 文本/sep/
    /// 条件转译），无 actions 时保持原行为。
    ///
    /// PLAN-630 T-02: declarative menubar component family → shadcn
    /// Menubar tree (tags route through the generic shadcn element path
    /// with the menubar_* prop arms).
    #[test]
    fn plan630_declarative_menubar_generates_shadcn_tree() {
        let src = concat!(
            "widget App {\n",
            "    model { var console_open bool = false }\n",
            "    view {\n",
            "        col {\n",
            "            menubar {\n",
            "                menubar-menu (value: \"view\") {\n",
            "                    menubar-trigger \"视图\"\n",
            "                    menubar-content {\n",
            "                        menubar-checkbox-item (title: \"切换 Console\", checked: .console_open) { onclick: .ActConsole }\n",
            "                        menubar-separator\n",
            "                        menubar-item (title: \"全选\", shortcut: \"Ctrl+A\") { onclick: .ActSelectAll }\n",
            "                    }\n",
            "                }\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "    on { .ActConsole -> { } .ActSelectAll -> { } }\n",
            "}\n",
        );
        let sfc = gen_sfc_from_widget_src_shadcn(src);
        assert!(
            sfc.contains("Menubar"),
            "menubar component present:\n{}",
            sfc
        );
        assert!(
            sfc.contains("切换 Console"),
            "checkbox item title present:\n{}",
            sfc
        );
        assert!(sfc.contains("全选"), "plain item title present:\n{}", sfc);
        assert!(
            sfc.contains("ActConsole") && sfc.contains("ActSelectAll"),
            "onclick handlers wired:\n{}",
            sfc
        );
    }

    /// PLAN-677 T-03: 视图声明式 menubar 族项渲染契约——icon→lucide
    /// 子组件、title→span、shortcut→右对齐 span、enabled→:disabled 取反、
    /// checkbox→:checked 单向绑定、分隔线自闭合。消费方 auto-edit 源形态。
    #[test]
    fn plan677_view_menubar_item_composition() {
        let src = concat!(
            "widget App {
",
            "    model {
",
            "        var console_open bool = false
",
            "        var tab_count int = 0
",
            "    }
",
            "    view {
",
            "        col {
",
            "            menubar {
",
            "                menubar-menu (value: \"view\") {
",
            "                    menubar-trigger \"视图\"
",
            "                    menubar-content {
",
            "                        menubar-checkbox-item (title: \"切换 Console\", icon: \"terminal\", shortcut: \"Ctrl+J\", checked: .console_open) { onclick: .ActConsole }
",
            "                        menubar-separator
",
            "                        menubar-item (title: \"保存\", icon: \"save\", shortcut: \"Ctrl+S\", enabled: .tab_count > 0) { onclick: .ActSave }
",
            "                    }
",
            "                }
",
            "            }
",
            "        }
",
            "    }
",
            "    on { .ActConsole -> { } .ActSave -> { } }
",
            "}
",
        );
        let sfc = gen_sfc_from_widget_src_shadcn(src);
        assert!(
            sfc.contains("<MenubarCheckboxItem @click=\"ActConsole\" :checked=\"console_open\">"),
            "checkbox item opens with one-way checked + click:
{}",
            sfc
        );
        assert!(
            sfc.contains("<Terminal class=\"mr-2 h-4 w-4\" />"),
            "icon prop lowers to lucide child:
{}",
            sfc
        );
        assert!(
            sfc.contains("<span>切换 Console</span>"),
            "title renders as slot text:
{}",
            sfc
        );
        assert!(
            sfc.contains(">Ctrl+J</MenubarShortcut>"),
            "shortcut renders as MenubarShortcut component (muted token, PLAN-695 浅色修正):
{}",
            sfc
        );
        assert!(
            !sfc.contains("text-zinc-500"),
            "hardcoded zinc shortcut color retired (PLAN-695):
{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarSeparator />"),
            "separator self-closes:
{}",
            sfc
        );
        assert!(
            sfc.contains(":disabled=\"!(tab_count > 0)\""),
            "enabled prop negates to :disabled:
{}",
            sfc
        );
        assert!(
            sfc.contains("<Save class=\"mr-2 h-4 w-4\" />") && sfc.contains("<span>保存</span>"),
            "plain item composes icon + title:
{}",
            sfc
        );
        assert!(
            sfc.contains("MenubarCheckboxItem, MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarShortcut, MenubarTrigger"),
            "full family imported from ui/menubar:
{}",
            sfc
        );
        assert!(
            !sfc.contains("<div :title="),
            "no bare div fallback remains for family tags:
{}",
            sfc
        );
    }

    /// PLAN-695 T-03: 族补全契约——label/shortcut 独立件、radio-group
    /// :model-value 单向绑定、radio-item value 透传、sub 三件套嵌套、
    /// group 容器透传；零手写 div、零硬编码 zinc 色。
    #[test]
    fn p695_view_menubar_full_family() {
        let src = concat!(
            "widget App {
",
            "    model {
",
            "        var theme string = \"light\"
",
            "        var autosave bool = true
",
            "    }
",
            "    view {
",
            "        col {
",
            "            menubar {
",
            "                menubar-menu (value: \"profiles\") {
",
            "                    menubar-trigger \"Profiles\"
",
            "                    menubar-content {
",
            "                        menubar-label (text: \"账号\")
",
            "                        menubar-radio-group (value: .theme) {
",
            "                            menubar-radio-item (value: \"light\", title: \"浅色\") { onclick: .ActLight }
",
            "                            menubar-radio-item (value: \"dark\", title: \"深色\", disabled: true)
",
            "                        }
",
            "                        menubar-separator
",
            "                        menubar-sub {
",
            "                            menubar-sub-trigger (text: \"更多\")
",
            "                            menubar-sub-content {
",
            "                                menubar-item (title: \"导入\") { onclick: .ActImport }
",
            "                            }
",
            "                        }
",
            "                        menubar-group {
",
            "                            menubar-item (title: \"自动保存\", checked: .autosave)
",
            "                            menubar-shortcut \"⌘S\"
",
            "                        }
",
            "                    }
",
            "                }
",
            "            }
",
            "        }
",
            "    }
",
            "    on { .ActLight -> { } .ActImport -> { } }
",
            "}
",
        );
        let sfc = gen_sfc_from_widget_src_shadcn(src);
        assert!(
            sfc.contains("<MenubarLabel>账号</MenubarLabel>"),
            "label emits MenubarLabel:
{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarRadioGroup :model-value=\"theme\">"),
            "radio group binds one-way :model-value:
{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarRadioItem @click=\"ActLight\" value=\"light\">")
                && sfc.contains("<MenubarRadioItem disabled value=\"dark\">"),
            "radio items carry value + click + disabled:
{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarSub>")
                && sfc.contains("<MenubarSubTrigger>更多</MenubarSubTrigger>")
                && sfc.contains("<MenubarSubContent>"),
            "sub triad nests:
{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarGroup>") && sfc.contains("<MenubarShortcut>⌘S</MenubarShortcut>"),
            "group container + standalone shortcut emit:
{}",
            sfc
        );
        assert!(
            !sfc.contains("text-zinc"),
            "zero hardcoded zinc colors (AC-02):
{}",
            sfc
        );
    }

    fn test_actions_menubar_synthesis() {
        let sfc = gen_sfc_from_widget_src_shadcn(ACTIONS_SRC);
        assert!(
            sfc.contains("<MenubarMenu value=\"file\">"),
            "menu entry:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarTrigger>文件</MenubarTrigger>"),
            "trigger title:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarItem @click=\"ActNew\">"),
            "item dispatches the on-block handler:\n{}",
            sfc
        );
        assert!(
            sfc.contains(":disabled=\"!(n > 0)\""),
            "enabled_if translated via convert_condition:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<Check v-if=\"n > 0\""),
            "checked_if drives the check slot:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<MenubarSeparator />"),
            "sep synthesized:\n{}",
            sfc
        );
        assert!(
            sfc.contains("import { Menubar, MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarTrigger } from '@/components/ui/menubar'"),
            "family imported (incl. separator):\n{}",
            sfc
        );
        // 快捷键显示文本（vm 同款右对齐弱色）。
        assert!(
            sfc.contains(">Ctrl+N</span>"),
            "shortcut hint rendered:\n{}",
            sfc
        );
    }

    /// `toolbar {}` 占位标签 → 图标按钮 + 分隔线；enabled_if → :disabled。
    #[test]
    fn test_actions_toolbar_synthesis() {
        let sfc = gen_sfc_from_widget_src_shadcn(ACTIONS_SRC);
        assert!(
            sfc.contains("class=\"flex flex-row items-center ml-auto\""),
            "toolbar row with the placeholder's user class:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@click=\"ActNew\" title=\"新建\"")
                || sfc.contains("title=\"新建\" @click=\"ActNew\""),
            "icon button with title tooltip:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<FilePlus class=\"h-4 w-4\" />"),
            "lucide icon component:\n{}",
            sfc
        );
        assert!(
            sfc.contains("class=\"w-px h-4 bg-zinc-500 mx-1 shrink-0\""),
            "toolbar separator:\n{}",
            sfc
        );
        assert!(
            sfc.contains("FilePlus }") || sfc.contains("FilePlus,"),
            "lucide icon import collected:\n{}",
            sfc
        );
    }

    /// enabled_if/checked_if 的裸 DSL 表达式拼写（Plan 451 P2）与引号串
    /// 等价——AST 里同为规范条件串。
    #[test]
    fn test_actions_condition_bare_expression_spelling() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    msg Msg { ActSave, ActUndo }
    model { var n int = 0 }
    actions {
        action (id: "file.save", handler: .ActSave, title: "保存", enabled_if: .n > 0)
        action (id: "edit.undo", handler: .ActUndo, title: "撤销", checked_if: .n > 0)
        menubar {
            menu (id: "edit", title: "编辑") { item (action: "edit.undo") }
        }
        toolbar { item (action: "file.save") }
    }
    view { col { menubar {}  toolbar {} } }
    on {
        .ActSave -> { .n = 0 }
        .ActUndo -> { .n = 0 }
    }
}
"#,
        );
        assert!(
            sfc.contains(":disabled=\"!(n > 0)\""),
            "bare enabled_if compiles to the same condition:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<Check v-if=\"n > 0\""),
            "bare checked_if compiles to the same condition:\n{}",
            sfc
        );
    }

    /// 无 actions 块时占位标签不合成（保持既有空行为），也不发射 keymap。
    #[test]
    fn test_actions_placeholder_without_block_untouched() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    msg Msg { Go }
    model { var n int = 0 }
    view { col { menubar {} } }
    on { .Go -> { .n = 1 } }
}
"#,
        );
        assert!(
            !sfc.contains("__autoActionsKeymap"),
            "no keymap without actions:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("<MenubarMenu"),
            "no menu synthesis without a menubar block:\n{}",
            sfc
        );
    }

    /// Event modifiers: key modifiers (up/escape), prevent, stop.
    #[test]
    fn test_event_modifiers() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Menu {
    msg Msg { MoveUp, Close, Ctx, Tap }
    model { var x int = 0 }
    view {
        col {
            onkeydown.up: .MoveUp,
            onkeydown.escape.prevent: .Close,
            oncontextmenu.prevent: .Ctx,
            onclick.stop: .Tap
        }
    }
    on {
        .MoveUp -> { .x = 1 }
        .Close -> { .x = 2 }
        .Ctx -> { .x = 3 }
        .Tap -> { .x = 4 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@keydown.up=\"MoveUp\""),
            "key modifier up:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@keydown.esc.prevent=\"Close\""),
            "escape normalizes to esc, prevent appended:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@contextmenu.prevent=\"Ctx\""),
            "prevent modifier:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@click.stop=\"Tap\""),
            "stop modifier:\n{}",
            sfc
        );
    }

    /// Regression lock (plan 402 follow-up): modifier-carrying event keys must
    /// survive extraction intact — no collapsing to the base name
    /// (`onclick.self` → `@click`) and no same-base overwrite
    /// (`onkeydown.enter/down/up` → a single `@keydown`).
    #[test]
    fn test_event_modifiers_same_base_no_overwrite() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Nav {
    msg Msg { X, A, B, C }
    model { var n int = 0 }
    view {
        col {
            onclick.self: .X,
            onkeydown.enter.prevent: .A,
            onkeydown.down.prevent: .B,
            onkeydown.up.prevent: .C
        }
    }
    on {
        .X -> { .n = 1 }
        .A -> { .n = 2 }
        .B -> { .n = 3 }
        .C -> { .n = 4 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@click.self=\"X\""),
            "self modifier kept:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@keydown.enter.prevent=\"A\""),
            "enter handler kept:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@keydown.down.prevent=\"B\""),
            "down handler kept:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@keydown.up.prevent=\"C\""),
            "up handler kept:\n{}",
            sfc
        );
        // The regressed form: base-only key with the last writer winning.
        assert!(
            !sfc.contains("@click=\"X\""),
            "no collapsed @click:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("@keydown=\"C\""),
            "no collapsed @keydown:\n{}",
            sfc
        );
    }

    /// The $event object flows into the template handler call, with field
    /// access ($event.key, $event.clientY) preserved.
    #[test]
    fn test_event_object_param() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Menu {
    msg Msg { Key, Drag }
    model { var x int = 0 }
    view {
        col {
            onkeydown: .Key($event),
            onmousedown: .Drag($event.clientY)
        }
    }
    on {
        .Key(e) -> { .x = 1 }
        .Drag(y) -> { .x = 2 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@keydown=\"Key($event)\""),
            "$event passed to handler:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mousedown=\"Drag($event.clientY)\""),
            "$event.clientY field access:\n{}",
            sfc
        );
        // Handler functions keep their declared params.
        assert!(sfc.contains("function Key(e: any)"), "Key param:\n{}", sfc);
        assert!(
            sfc.contains("function Drag(y: any)"),
            "Drag param:\n{}",
            sfc
        );
    }

    /// P0#12: a STATE field reference inside a map-literal event argument
    /// (`.H({ q: .query, e: $event })`) must emit the bare setup-scope binding
    /// (`query` — Vue templates auto-unwrap setup refs), NOT `this.query`:
    /// `this` is invalid in Vue 3 template expressions and breaks silently at
    /// runtime. Loop variables in the same position pass through untouched.
    #[test]
    fn test_map_literal_event_arg_state_field_no_this() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget SearchBar {
    msg Msg { Search }
    model { var query str = "" }
    view {
        col {
            input {
                oninput: .Search({ q: .query, e: $event })
            }
        }
    }
    on {
        .Search(payload) -> { .query = "" }
    }
}
"#,
        );
        assert!(
            !sfc.contains("this."),
            "template event args must never reference `this` (invalid in Vue 3 templates):\n{}",
            sfc
        );
        assert!(
            sfc.contains("@input=\"Search({ q: query, e: $event })\""),
            "state field in map-literal arg must emit the bare binding:\n{}",
            sfc
        );
    }

    /// DEBTS 引擎窗口 #6 (plan-066 发现,plan-008 契约收口): a bare
    /// `oninput: .H` binding forwards the typed text ONLY when the handler
    /// DECLARES exactly one param. HistorySearch's `.OnQuery -> {}` (0 params,
    /// reads the v-model-synced state) must stay a bare call —
    /// `OnQuery(($event.target as HTMLInputElement).value)` was TS2554 against
    /// the generated 0-param function; the plan-008 single-param contract
    /// (`.Draft(v str)`) keeps the wrap.
    #[test]
    fn test_input_text_arg_follows_declared_handler_arity() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget QueryProbe {
    model { var query str = "" }
    view {
        col {
            input (value: .query) {
                oninput: .OnQuery
            }
        }
    }
    on {
        .OnQuery -> {
            var q str = .query.trim().to_lower()
            if q != "" {
                .query = q
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("@input=\"OnQuery\""),
            "0-param handler must stay a bare call (typed-text arg is TS2554):\n{}",
            sfc
        );

        let sfc = gen_sfc_from_widget_src(
            r#"
widget DraftProbe {
    model { var draft str = "" }
    view {
        col {
            input (value: .draft) {
                oninput: .Draft
            }
        }
    }
    on {
        .Draft(v str) -> {
            .draft = v
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("@input=\"Draft(($event.target as HTMLInputElement).value)\""),
            "single-param handler receives the typed text (plan-008 contract):\n{}",
            sfc
        );
    }

    /// Same bug class, sibling positions: `this.` not at the very start of the
    /// param string — nested inside a call argument and inside a nested map.
    #[test]
    fn test_nested_event_arg_state_field_no_this() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget NestedArgProbe {
    msg Msg { Apply, Store }
    model { var raw str = "" }
    view {
        col {
            onclick: .Apply(fmt(.raw)),
            onblur: .Store({ outer: { inner: .raw } })
        }
    }
    on {
        .Apply(v) -> { .raw = v }
        .Store(p) -> { .raw = "" }
    }
}
"#,
        );
        assert!(
            !sfc.contains("this."),
            "nested event args must never reference `this`:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@click=\"Apply(fmt(raw))\""),
            "state field nested in call arg must emit the bare binding:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@blur=\"Store({ outer: { inner: raw } })\""),
            "state field in nested map must emit the bare binding:\n{}",
            sfc
        );
    }

    /// Plan 559 W2: `$event.target.value` / `$event.target.checked` written in
    /// the .at source is VM-track change-payload syntax (os-config
    /// collection_browser.at `ApplyEntry(e.key, $event.target.value)` family).
    /// Vue's `EventTarget` carries neither member, and `target` is nullable —
    /// the raw passthrough fails vue-tsc with TS2339/TS18047. Every handler
    /// arg flows through `vue_event_param`, so the narrowing lives there.
    #[test]
    fn test_event_arg_target_payload_narrowed() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget TargetPayloadProbe {
    msg Msg { ApplyEntry, MsToggle }
    model { var active str = "" }
    view {
        col {
            input (value: "x", "type": "text") {
                onchange: .ApplyEntry(.active, $event.target.value)
            }
            input ("type": "checkbox", checked: false) {
                onchange: .MsToggle(.active, $event.target.checked)
            }
        }
    }
    on {
        .ApplyEntry(k, v) -> { .active = k }
        .MsToggle(k, c) -> { .active = k }
    }
}
"#,
        );
        assert!(
            sfc.contains("ApplyEntry((.active), ($event.target as HTMLInputElement).value)")
                || sfc.contains("ApplyEntry(.active, ($event.target as HTMLInputElement).value)")
                || sfc.contains("ApplyEntry(active, ($event.target as HTMLInputElement).value)"),
            "target.value in handler args must narrow to HTMLInputElement:\n{}",
            sfc
        );
        assert!(
            sfc.contains("($event.target as HTMLInputElement).checked"),
            "target.checked in handler args must narrow to HTMLInputElement:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("$event.target.value") && !sfc.contains("$event.target.checked"),
            "raw $event.target payloads must not survive into the template:\n{}",
            sfc
        );
    }

    /// Global (window/document) listener args go through a separate codegen
    /// path (`try_register_global_listener`) — same `this.` constraint applies.
    #[test]
    fn test_global_listener_map_arg_state_field_no_this() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget GlobalArgProbe {
    msg Msg { Track }
    model { var origin str = "" }
    view {
        col {
            onmousemove.window: .Track({ o: .origin, e: $event })
        }
    }
    on {
        .Track(p) -> { .origin = "" }
    }
}
"#,
        );
        assert!(
            !sfc.contains("this."),
            "global listener args must never reference `this`:\n{}",
            sfc
        );
        assert!(
            sfc.contains("Track({ o: origin, e: e })"),
            "state field in global listener map arg must emit the bare binding:\n{}",
            sfc
        );
    }

    /// window-level mouse listeners: drag tracking outside the element.
    /// - with $event → wrapper function adapting args
    /// - without params → bare function reference
    #[test]
    fn test_global_window_mouse_listeners() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Scrollbar {
    msg Msg { DragMove, DragEnd }
    model { var y int = 0 }
    view {
        col {
            onmousemove.window: .DragMove($event),
            onmouseup.window: .DragEnd
        }
    }
    on {
        .DragMove(e) -> { .y = 0 }
        .DragEnd -> { .y = 1 }
    }
}
"#,
        );
        // No template attribute for global listeners.
        assert!(!sfc.contains("@mousemove"), "no @mousemove attr:\n{}", sfc);
        assert!(!sfc.contains("@mouseup"), "no @mouseup attr:\n{}", sfc);
        // add/remove pairs in onMounted/onUnmounted.
        assert!(
            sfc.contains("window.addEventListener('mousemove', __auto_gl_mousemove_DragMove)"),
            "window mousemove add:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('mousemove', __auto_gl_mousemove_DragMove)"),
            "window mousemove remove:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.addEventListener('mouseup', DragEnd)"),
            "bare fn ref when no params:\n{}",
            sfc
        );
        assert!(
            sfc.contains("window.removeEventListener('mouseup', DragEnd)"),
            "bare fn ref removal:\n{}",
            sfc
        );
        // Wrapper adapts $event → e.
        assert!(
            sfc.contains("function __auto_gl_mousemove_DragMove(e: any) {\n  DragMove(e)\n}"),
            "wrapper adapts event arg:\n{}",
            sfc
        );
        // Lifecycle imports present even without an explicit .Init/.Destroy.
        assert!(
            sfc.contains("onMounted") && sfc.contains("onUnmounted"),
            "lifecycle hooks imported/emitted:\n{}",
            sfc
        );
    }

    /// document-level wheel lock: capture phase + preventDefault, with
    /// passive: false (required by Chrome for document-level wheel listeners).
    #[test]
    fn test_global_document_wheel_capture_prevent() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget CodeMenu {
    msg Msg { LockWheel }
    model { var locked int = 0 }
    view {
        col {
            onwheel.document.capture.prevent: .LockWheel($event)
        }
    }
    on {
        .LockWheel(e) -> { .locked = 1 }
    }
}
"#,
        );
        assert!(
            sfc.contains(
                "document.addEventListener('wheel', __auto_gl_wheel_LockWheel, { capture: true, passive: false })"
            ),
            "capture+passive options on add:\n{}",
            sfc
        );
        assert!(
            sfc.contains(
                "document.removeEventListener('wheel', __auto_gl_wheel_LockWheel, { capture: true })"
            ),
            "capture option on remove (no passive):\n{}",
            sfc
        );
        assert!(
            sfc.contains("function __auto_gl_wheel_LockWheel(e: any) {\n  e.preventDefault()\n  LockWheel(e)\n}"),
            "wrapper calls preventDefault then handler:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Widget `use { ... }` external TS/Vue imports (escape hatch)
    // ====================================================================

    /// The widget-level `use { ... }` block parses into `decl.ext_imports`
    /// with kind, symbols, and path intact.
    #[test]
    fn test_ext_use_block_parses() {
        let src = r#"
widget Icon(language: str) {
    use {
        fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
        fn: marked, purify from "marked"
        component: FancyBadge from "src/front/components/FancyBadge.vue"
        component: Smile from "lucide-vue-next"
        composable: useClock from "src/front/composables/useClock.ts"
    }
    view {
        div { "hi" }
    }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget use block must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");

        assert_eq!(decl.ext_imports.len(), 5, "all five entries parse");
        assert_eq!(
            decl.ext_imports[0].kind,
            auto_ui_dialect::ast::ExtImportKind::Fn
        );
        assert_eq!(decl.ext_imports[0].symbols.len(), 1);
        assert_eq!(
            decl.ext_imports[0].symbols[0].as_str(),
            "getLanguageIconUrl"
        );
        assert_eq!(
            decl.ext_imports[0].path.as_str(),
            "src/front/utils/codeBlockLanguage.ts"
        );
        // Comma-separated symbol list
        assert_eq!(decl.ext_imports[1].symbols.len(), 2);
        assert_eq!(decl.ext_imports[1].symbols[1].as_str(), "purify");
        assert_eq!(
            decl.ext_imports[2].kind,
            auto_ui_dialect::ast::ExtImportKind::Component
        );
        assert_eq!(
            decl.ext_imports[4].kind,
            auto_ui_dialect::ast::ExtImportKind::Composable
        );
    }

    /// `fn:` imports become named ES imports; npm specifiers pass through,
    /// local files are rewritten to the `@/ext/...` alias. The imported
    /// symbol is callable from `on` handlers and `computed`.
    #[test]
    fn test_ext_fn_import_and_call_sites() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Icon(language: str) {
    use {
        fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
        fn: marked from "marked"
    }
    msg Msg { Render }
    model { var url str = "" }
    computed {
        iconUrl => getLanguageIconUrl(language)
    }
    view {
        col {
            onclick: .Render
        }
    }
    on {
        .Render -> { .url = getLanguageIconUrl(.language) }
    }
}
"#,
        );
        // Local path → @/ext alias, .ts extension dropped.
        assert!(
            sfc.contains(
                "import { getLanguageIconUrl } from '@/ext/src/front/utils/codeBlockLanguage'"
            ),
            "local fn import via @/ext alias:\n{}",
            sfc
        );
        // npm specifier passes through unchanged.
        assert!(
            sfc.contains("import { marked } from 'marked'"),
            "npm fn import passthrough:\n{}",
            sfc
        );
        // Callable from computed (prop resolves to props.language).
        assert!(
            sfc.contains("getLanguageIconUrl(props.language)"),
            "fn call in computed:\n{}",
            sfc
        );
        // Callable from on handler.
        assert!(
            sfc.contains("url.value = getLanguageIconUrl(props.language)"),
            "fn call in on handler:\n{}",
            sfc
        );
    }

    /// `component:` from a local `.vue` file → default import; usable as a
    /// view tag (PascalCase or snake_case) with generic `:prop` bindings
    /// and `@event` listeners.
    #[test]
    fn test_ext_component_local_vue() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    use {
        component: FancyBadge from "src/front/components/FancyBadge.vue"
    }
    msg Msg { Picked }
    model { var label str = "hi" }
    view {
        col {
            FancyBadge {
                label: .label,
                onselected: .Picked
            }
            fancy_badge {
                label: .label
            }
        }
    }
    on {
        .Picked -> { .label = "picked" }
    }
}
"#,
        );
        assert!(
            sfc.contains("import FancyBadge from '@/ext/src/front/components/FancyBadge.vue'"),
            "default import for local .vue component:\n{}",
            sfc
        );
        // Both tag spellings render as the PascalCase component.
        let count = sfc.matches("<FancyBadge").count();
        assert_eq!(
            count, 2,
            "both tag spellings instantiate FancyBadge:\n{}",
            sfc
        );
        // Generic prop binding + event listener.
        assert!(sfc.contains(":label=\"label\""), "prop v-bind:\n{}", sfc);
        assert!(
            sfc.contains("@selected=\"Picked\""),
            "event listener:\n{}",
            sfc
        );
        // No fallback `@/components/FancyBadge.vue` import (registry path).
        assert!(
            !sfc.contains("from '@/components/FancyBadge.vue'"),
            "ext component must not use the @/components fallback:\n{}",
            sfc
        );
    }

    /// `component:` from an npm package → named import (lucide-style).
    #[test]
    fn test_ext_component_npm_named_import() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    use {
        component: Smile from "lucide-vue-next"
    }
    view {
        col {
            Smile { }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("import { Smile } from 'lucide-vue-next'"),
            "named npm component import:\n{}",
            sfc
        );
        assert!(sfc.contains("<Smile"), "component tag rendered:\n{}", sfc);
    }

    /// `composable:` → named import + a single call at `<script setup>`
    /// top level, return value bound to a derived local const.
    #[test]
    fn test_ext_composable_setup_call() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    use {
        composable: useMenuBounds from "src/front/composables/useMenuBounds.ts"
    }
    msg Msg { Open }
    model { var x int = 0 }
    view {
        col {
            onclick: .Open
        }
    }
    on {
        .Open -> { .x = 1 }
    }
}
"#,
        );
        assert!(
            sfc.contains(
                "import { useMenuBounds } from '@/ext/src/front/composables/useMenuBounds'"
            ),
            "composable import:\n{}",
            sfc
        );
        assert!(
            sfc.contains("const menuBounds = useMenuBounds()"),
            "composable called at setup top level:\n{}",
            sfc
        );
    }

    /// The new mechanism expresses the AutoDownEditor case: a declared
    /// `component: AutoDownEditor from "@autodown/editor"` produces the
    /// same import + PascalCase tag as the hardcoded registry path.
    #[test]
    fn test_ext_component_expresses_autodown_editor_case() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    use {
        component: AutoDownEditor from "@autodown/editor"
    }
    model { var body str = "" }
    view {
        col {
            AutoDownEditor {
                content: .body
            }
        }
    }
}
"#,
        );
        assert!(
            // DSL 显式 use.web component from 走声明包（应用作者自选，存量
            // vendor 仍 @autodown/editor）；registry 路径才默认 engine（plan
            // 019 Phase 4，见 15214 处用例）。
            sfc.contains("import { AutoDownEditor } from '@autodown/editor'"),
            "declared-source import shape (use.web from) must be honored:\n{}",
            sfc
        );
        assert!(sfc.contains("<AutoDownEditor"), "component tag:\n{}", sfc);
        assert!(sfc.contains(":content=\"body\""), "prop binding:\n{}", sfc);
    }

    /// ext `component:` declaration wins over the auto sub-widget import: a
    /// widget that both declares `component: CommandPalette from "...ext.ts"`
    /// and references the sibling sub-widget `CommandPalette` in its view must
    /// emit exactly one `CommandPalette` import (the ext one). Emitting both
    /// binds the identifier twice (TS2300) — the ext declaration is explicit
    /// user intent (the ext module may wrap/rename the real component).
    #[test]
    fn test_ext_component_shadows_auto_sub_widget_import() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
widget AppShell {
    use {
        component: CommandPalette from "src/front/utils/app_shell_ext.ts"
    }
    model { var open bool = false }
    view {
        col {
            CommandPalette {
                open: .open
            }
        }
    }
}
"#,
            &["CommandPalette"],
        );
        // The ext declaration's import is emitted (local .ts → @/ext alias).
        assert!(
            sfc.contains("import { CommandPalette } from '@/ext/src/front/utils/app_shell_ext'"),
            "ext component import emitted:\n{}",
            sfc
        );
        // The plan-408 auto `@/components/CommandPalette.vue` import is skipped.
        assert!(
            !sfc.contains("from '@/components/CommandPalette.vue'"),
            "no duplicate auto sub-widget import:\n{}",
            sfc
        );
        // Exactly one import line binds the CommandPalette identifier.
        let import_lines = sfc
            .lines()
            .filter(|l| l.starts_with("import") && l.contains("CommandPalette"))
            .count();
        assert_eq!(
            import_lines, 1,
            "single CommandPalette import (no TS2300):\n{}",
            sfc
        );
        // Template resolution is untouched: the tag still renders as the
        // component (registration/resolution logic unchanged).
        assert!(
            sfc.contains("<CommandPalette"),
            "component tag rendered:\n{}",
            sfc
        );
    }

    /// Regression guard: without an ext declaration, the plan-408 auto
    /// sub-widget import behavior is unchanged.
    #[test]
    fn test_auto_sub_widget_import_without_ext_decl_unchanged() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
widget BlockList {
    view {
        col {
            BlockItem { }
        }
    }
}
"#,
            &["BlockItem"],
        );
        assert!(
            sfc.contains("import BlockItem from '@/components/BlockItem.vue'"),
            "auto sub-widget import still emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<BlockItem"),
            "sub-widget tag rendered:\n{}",
            sfc
        );
    }

    /// The same name-based dedup applies when an ext `fn:` (or composable)
    /// declaration already binds the identifier — a second default import of
    /// the same name would still be TS2300.
    #[test]
    fn test_ext_fn_name_collision_skips_auto_sub_widget_import() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
widget App {
    use {
        fn: Helper from "src/front/utils/helpers.ts"
    }
    view {
        col {
            Helper { }
        }
    }
}
"#,
            &["Helper"],
        );
        assert!(
            sfc.contains("import { Helper } from '@/ext/src/front/utils/helpers'"),
            "ext fn import emitted:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("from '@/components/Helper.vue'"),
            "auto sub-widget import skipped on name collision:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Quoted custom event names: on "autodown:slash-open" — global
    // (document/window) listeners and element-level bindings.
    // ====================================================================

    /// document-level CustomEvent listeners with ':'/'-' in the event name
    /// (SlashMenu integration: a hand-written TS extension dispatches
    /// `autodown:slash-open` etc. on document).
    #[test]
    fn test_global_custom_event_listeners() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget SlashMenu {
    msg Msg { OnOpen, OnClose }
    model { var query str = "" }
    view {
        col {
            on "autodown:slash-open".document: .OnOpen($event),
            on "autodown:slash-close".document: .OnClose
        }
    }
    on {
        .OnOpen(e) -> { .query = "x" }
        .OnClose -> { .query = "" }
    }
}
"#,
        );
        // No template attribute for global listeners.
        assert!(
            !sfc.contains("@autodown:slash-open"),
            "no template attr:\n{}",
            sfc
        );
        // add/remove pairs carry the raw custom event name.
        assert!(
            sfc.contains("document.addEventListener('autodown:slash-open', __auto_gl_autodown_slash_open_OnOpen)"),
            "custom event add (sanitized wrapper name):\n{}",
            sfc
        );
        assert!(
            sfc.contains("document.removeEventListener('autodown:slash-open', __auto_gl_autodown_slash_open_OnOpen)"),
            "custom event remove:\n{}",
            sfc
        );
        // No-param handler → bare function reference.
        assert!(
            sfc.contains("document.addEventListener('autodown:slash-close', OnClose)"),
            "bare fn ref:\n{}",
            sfc
        );
        // Wrapper adapts the DOM event arg.
        assert!(
            sfc.contains("function __auto_gl_autodown_slash_open_OnOpen(e: any) {\n  OnOpen(e)\n}"),
            "wrapper body:\n{}",
            sfc
        );
    }

    /// Element-level custom event binding (child component emit with a
    /// ':'/'-' name). Verified against @vue/compiler-dom + runtime-dom:
    /// `@autodown:slash-open` compiles and patches to
    /// addEventListener('autodown:slash-open').
    #[test]
    fn test_element_custom_event_listener() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Menu {
    msg Msg { Poked }
    model { var x int = 0 }
    view {
        col {
            on "demo:poke": .Poked($event)
        }
    }
    on {
        .Poked(e) -> { .x = 1 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@demo:poke=\"Poked($event)\""),
            "element-level custom event binding:\n{}",
            sfc
        );
    }

    // ====================================================================
    // style_obj: inline-style object binding (:style), distinct from the
    // style: { class: cond } dynamic-class binding.
    // ====================================================================

    /// style_obj generates a Vue :style object binding; values are arbitrary
    /// expressions including f-string px concatenation; hyphenated CSS
    /// property names are quoted.
    #[test]
    fn test_style_obj_inline_style_binding() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Popover {
    msg Msg { Nop }
    model {
        var menu_x int = 50
        var menu_y int = 100
        var menu_vis str = "visible"
    }
    view {
        col {
            style_obj: { top: f"${.menu_y}px", left: f"${.menu_x}px", "z-index": 50, visibility: .menu_vis }
        }
    }
    on {
        .Nop -> { .menu_x = 0 }
    }
}
"#,
        );
        assert!(
            sfc.contains(":style=\"({ top: `${menu_y}px`, left: `${menu_x}px`, 'z-index': 50, visibility: menu_vis } as any)\""),
            ":style object binding:\n{}",
            sfc
        );
        // Must NOT be emitted as a class binding.
        assert!(
            !sfc.contains(":class=\"{ top:"),
            "not a class binding:\n{}",
            sfc
        );
    }

    /// The classic style: { class: cond } map stays a dynamic class binding;
    /// hyphenated class names are quoted (previously emitted as a bare,
    /// invalid JS key).
    #[test]
    fn test_style_class_binding_quotes_hyphenated_keys() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Todo {
    msg Msg { Nop }
    model { var done int = 0 }
    view {
        col {
            text "x" { style: { completed: .done, "line-through": .done } }
        }
    }
    on {
        .Nop -> { .done = 1 }
    }
}
"#,
        );
        assert!(
            sfc.contains(":class=\"{ completed: done, 'line-through': done }\""),
            "hyphenated class key quoted:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Slots: `slot` outlet in a widget view (default + named) and
    // `slot(name:) { ... }` named-slot targeting at the parent side.
    // ====================================================================

    /// Bare `slot` in a widget view → default outlet `<slot />`
    /// (previously swallowed by the unknown-tag div fallback).
    #[test]
    fn test_slot_outlet_default() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            class: "panel",
            slot
        }
    }
}
"#,
        );
        assert!(sfc.contains("<slot />"), "default slot outlet:\n{}", sfc);
        assert!(
            !sfc.contains("<div>\n</div>") && !sfc.contains("<div />"),
            "slot must not fall back to an empty div:\n{}",
            sfc
        );
    }

    /// `slot(name: "header")` in a widget view → named outlet
    /// `<slot name="header" />`.
    #[test]
    fn test_slot_outlet_named() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            slot(name: "header")
            slot
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<slot name=\"header\" />"),
            "named slot outlet:\n{}",
            sfc
        );
        assert!(
            sfc.contains("<slot />"),
            "default slot outlet too:\n{}",
            sfc
        );
    }

    /// Parent side: `slot(name: "header") { ... }` inside a component
    /// instantiation's children block → `<template #header>...</template>`.
    #[test]
    fn test_slot_named_template_parent_side() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel(title: "hi") {
                slot(name: "header") {
                    text "My Title"
                }
                text "body content"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<template #header>"),
            "named slot template in parent:\n{}",
            sfc
        );
        assert!(
            sfc.contains(">My Title</span>"),
            "named slot content rendered:\n{}",
            sfc
        );
        assert!(
            sfc.contains(">body content</span>"),
            "default-slot child still rendered:\n{}",
            sfc
        );
    }

    /// Regression: plain children passed to a component still emit
    /// unchanged (no <template> wrapping, no slot outlet interference).
    #[test]
    fn test_slot_default_children_unchanged() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel(title: "hi") {
                text "plain child"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(">plain child</span>"),
            "default children unchanged:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("<template #"),
            "no named-slot template without slot(name:):\n{}",
            sfc
        );
    }

    /// Outlet-less sub-widget receiving children → build-time warning via
    /// AuraWidget::slot_children_warnings; widget WITH a matching outlet
    /// produces no warning.
    #[test]
    fn test_slot_children_warning() {
        let parse_widget = |src: &str| {
            let session = auto_lang_core::session::CompilerSession::ui();
            let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
            let ast = crate::parse_ui(&src).expect("widget source must parse");
            let decl = ast
                .domain_stmts()
                .find_map(|s| match s {
                    auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                    _ => None,
                })
                .expect("widget decl");
            crate::aura::extract_widget_from_decl(decl).expect("extract widget")
        };

        let outletless = parse_widget(
            r#"
widget Panel {
    model { var n int = 0 }
    view { col { text "no outlets" } }
}
"#,
        );
        let with_outlets = parse_widget(
            r#"
widget Panel {
    model { var n int = 0 }
    view {
        col {
            slot(name: "header")
            slot
        }
    }
}
"#,
        );
        assert!(with_outlets.slot_outlet_names().contains(&String::new()));
        assert!(with_outlets
            .slot_outlet_names()
            .contains(&"header".to_string()));
        assert!(outletless.slot_outlet_names().is_empty());

        let app = parse_widget(
            r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            Panel {
                slot(name: "header") {
                    text "My Title"
                }
                text "body content"
            }
        }
    }
}
"#,
        );

        // Outlet-less target: both the named template and the default
        // children must warn.
        let mut outlets = std::collections::HashMap::new();
        outlets.insert("Panel".to_string(), outletless.slot_outlet_names());
        let warnings = app.slot_children_warnings(&outlets);
        assert_eq!(
            warnings.len(),
            2,
            "named + default warnings:\n{:?}",
            warnings
        );
        assert!(warnings.iter().any(|w| w.contains("'header' slot outlet")));
        assert!(warnings
            .iter()
            .any(|w| w.contains("no default slot outlet")));

        // Target with matching outlets: no warnings.
        outlets.insert("Panel".to_string(), with_outlets.slot_outlet_names());
        let warnings = app.slot_children_warnings(&outlets);
        assert!(
            warnings.is_empty(),
            "no warnings when outlets match:\n{:?}",
            warnings
        );
    }

    // ====================================================================
    // v-for :key — explicit `key:` prop override + index-var fallback.
    //
    // Syntax: any node in a loop body may take a `key: <expr>` prop; it is
    // emitted as `:key="<expr>"` and wins over the auto-generated reuse key.
    // ====================================================================

    /// Same as gen_sfc_from_widget_src, but registers sibling sub-widget
    /// names (the production app-build path passes them via
    /// with_sub_widgets; the known-sub-widget auto-:key logic only runs then).
    /// PLAN-037 T6: `use.web` statement parses all four forms into ExtImport
    /// entries with the right kind/symbols/path (refs for composable).
    #[test]
    fn test_use_web_stmt_four_forms() {
        use auto_lang_core::ast::Stmt;
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
use.web agentAvatarData from "src/front/forge_helpers.at"
use.web component MessageSquare, ListTodo from "lucide-vue-next"
use.web composable useT from "src/front/composables/useT.ts"
use.web composable useI18n refs: [locale] from "vue-i18n"
widget App { view { col { text "x" { } } } }
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("parse");
        let entries: Vec<&auto_ui_dialect::ast::ExtImport> = ast
            .domain_stmts()
            .filter_map(|s| match s {
                auto_ui_dialect::UiStmt::UseWeb(v) => v.first(),
                _ => None,
            })
            .collect();
        assert_eq!(entries.len(), 4, "four use.web entries");
        assert!(matches!(
            entries[0].kind,
            auto_ui_dialect::ast::ExtImportKind::Fn
        ));
        assert_eq!(entries[0].symbols.len(), 1);
        assert!(entries[0].symbols[0].as_str() == "agentAvatarData");
        assert!(matches!(
            entries[1].kind,
            auto_ui_dialect::ast::ExtImportKind::Component
        ));
        assert_eq!(entries[1].symbols.len(), 2);
        assert!(matches!(
            entries[2].kind,
            auto_ui_dialect::ast::ExtImportKind::Composable
        ));
        assert!(entries[3].ref_fields.len() == 1);
        assert!(entries[3].ref_fields[0].as_str() == "locale");
        assert!(entries[3].path.as_str() == "vue-i18n");
    }

    /// PLAN-037 T5: call-site model addressing — a prop name matching the
    /// child's model var with a writable state-slot target folds to
    /// v-model:key; the same channel fed an expression is a hard error.
    #[test]
    fn test_model_channel_addressing_vmodel() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let src = r#"
widget App {
    model { var draft str = "" }
    view { col { BindChild(value: .draft) text .draft { } } }
}
"#;
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
        let mut models = std::collections::HashMap::new();
        models.insert("BindChild".to_string(), vec!["value".to_string()]);
        let mut gen = VueGenerator::new_shadcn()
            .with_sub_widgets(vec!["BindChild".to_string()])
            .with_sub_widget_models(models);
        let sfc = gen.generate_sfc(&widget).expect("generate");
        assert!(
            sfc.contains("v-model:value=\"draft\""),
            "channel folds to v-model:
{}",
            sfc
        );

        // Non-slot target on the same channel must fail generation.
        let session2 = auto_lang_core::session::CompilerSession::ui();
        let src2 = r#"
widget App {
    model { var draft str = "" }
    view { col { BindChild(value: .draft + "!") text .draft { } } }
}
"#;
        let mut parser2 = auto_lang_core::parser::Parser::from(src2).with_session(session2);
        let ast2 = crate::parse_ui(src2).expect("parse");
        let decl2 = ast2
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget2 = crate::aura::extract_widget_from_decl(decl2).expect("extract");
        let mut models2 = std::collections::HashMap::new();
        models2.insert("BindChild".to_string(), vec!["value".to_string()]);
        let mut gen2 = VueGenerator::new_shadcn()
            .with_sub_widgets(vec!["BindChild".to_string()])
            .with_sub_widget_models(models2);
        // Plan 435 挂账②:非可写值不再硬错 —— 降级为单向 prop 绑定
        // (值流入子件 defineModel),R016 Info 告警提示非双向。
        let sfc2 = gen2.generate_sfc(&widget2).expect("degrade, not fail");
        assert!(
            sfc2.contains(":value="),
            "one-way :value binding should be emitted: {}",
            sfc2
        );
        assert!(
            gen2.last_validation_warnings
                .iter()
                .any(|w| w.rule == "R016"),
            "R016 advisory should be raised"
        );
    }

    fn gen_sfc_with_sub_widgets(src: &str, subs: &[&str]) -> String {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen =
            VueGenerator::new().with_sub_widgets(subs.iter().map(|s| s.to_string()).collect());
        gen.generate(&widget).expect("generate SFC")
    }

    /// Explicit `key:` on a sub-widget component instantiation in a loop
    /// wins over the auto-key: exactly one :key, bound to the given expr.
    #[test]
    fn test_vfor_explicit_key_on_sub_widget() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for tab in .tabs {
                EditorTab(key: tab.path, path: tab.path)
            }
        }
    }
}
"#,
            &["EditorTab"],
        );
        assert!(
            sfc.contains(r#":key="tab.path""#),
            "explicit key emitted:\n{}",
            sfc
        );
        assert_eq!(
            sfc.matches(":key=").count(),
            1,
            "exactly one :key (no duplicate auto-key):\n{}",
            sfc
        );
        assert!(
            !sfc.contains("tab?.id"),
            "no auto-key ?.id chain when explicit key given:\n{}",
            sfc
        );
    }

    /// Explicit `key:` on a plain element in a loop is emitted as :key.
    #[test]
    fn test_vfor_explicit_key_on_plain_element() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var names list = [] }
    view {
        col {
            for name in .names {
                span(key: name) { text "x" }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<span :key="name" v-for="name in names">"#),
            "explicit key on plain element:\n{}",
            sfc
        );
    }

    /// PLAN-682 F2 合并期回归钉：layout 伪映射部件（row/col）走
    /// generate_shadcn_attrs 的 layout 臂，其 `push_passthrough_attrs`
    /// 透传曾把 `key` prop 在函数顶部单点转发之后**再发一次** `:key`
    /// ——同一元素双 `:key` 属性，Vue 编译器 X_DUPLICATE_ATTRIBUTE 拒收
    /// （bps-gallery DataTableCrud 行 div 实证）。透传环内现按「attrs
    /// 已含 `:key=` 则跳过」去重。shadcn 模式=CLI 真实路由（plain 模式
    /// 走 plain 臂不经过此路径）。
    #[test]
    fn test_vfor_explicit_key_on_layout_widget_single_emission() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget App {
    model { var rows list = [] }
    view {
        for row in .rows {
            row (key: row.id) { text "x" }
        }
    }
}
"#,
        );
        let head = sfc
            .lines()
            .find(|l| l.contains("v-for="))
            .unwrap_or_else(|| panic!("no v-for line:\n{}", sfc));
        assert_eq!(
            head.matches(":key=").count(),
            1,
            "layout widget explicit key must emit exactly one :key:\n{}",
            sfc
        );
    }

    /// Regression: without an explicit key, the auto-key heuristic is
    /// unchanged (`'Tag-N-' + (item?.id ?? item)`).
    #[test]
    fn test_vfor_auto_key_unchanged_without_explicit_key() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#,
            &["EditorTab"],
        );
        assert!(
            // Plan 028: 键表达式演进为 `(tab as any)?.id ?? tab` —— 对未声明
            // .id 的具类型循环变量加 as-any 断言，避免 vue-tsc 报错。
            sfc.contains(r#":key="'EditorTab-1-' + (((tab as any)?.id ?? tab))""#),
            "auto-key heuristic unchanged:\n{}",
            sfc
        );
    }

    /// Indexed loop: the loop var is the primitive int index — the key must
    /// use the index itself, never `i?.id`.
    #[test]
    fn test_vfor_indexed_loop_key_uses_index() {
        // Sub-widget path (known sibling widget).
        let sfc = gen_sfc_with_sub_widgets(
            r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    view {
        col {
            for i, tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#,
            &["EditorTab"],
        );
        assert!(
            sfc.contains(r#":key="'EditorTab-1-' + i""#),
            "index var used as key:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("i?.id"),
            "no ?.id on primitive index:\n{}",
            sfc
        );

        // Generic Vue-component path (Plan 360 fallback).
        let sfc2 = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var tabs list = [] }
    view {
        col {
            for i, tab in .tabs {
                EditorTab(path: tab.path)
            }
        }
    }
}
"#,
        );
        assert!(
            sfc2.contains(r#":key="'EditorTab-1-' + i""#),
            "index var used as key (generic path):\n{}",
            sfc2
        );
        assert!(
            !sfc2.contains("i?.id"),
            "no ?.id on primitive index (generic path):\n{}",
            sfc2
        );
    }

    // ====================================================================
    // v-show (gap 52) — `show: <expr>` prop emits `v-show="<expr>"` instead
    // of a `:show` binding. The element/component stays mounted; only inline
    // display toggles (jade MainArea keep-alive tabs are the driving case).
    // ====================================================================

    /// Dynamic condition on a plain element (brace prop form).
    #[test]
    fn test_vshow_plain_element_dynamic() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var active_path str = "" }
    view {
        col {
            div(show: .active_path == "graph", class: "graph-pane") { text "G" }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-show="active_path == 'graph'""#),
            "v-show emitted with bound condition:\n{}",
            sfc
        );
        assert!(!sfc.contains(":show="), "no :show binding leaks:\n{}", sfc);
    }

    /// Static-ish model ref on a plain element, block prop form.
    #[test]
    fn test_vshow_plain_element_model_ref() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var visible bool = true }
    view {
        col {
            div {
                show: .visible
                text "hi"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-show="visible""#),
            "v-show with bare model ref:\n{}",
            sfc
        );
        assert!(!sfc.contains(":show="), "no :show binding leaks:\n{}", sfc);
    }

    /// v-show on a sub-widget component instantiation (v-show works on
    /// components in Vue — the directive lands on the component root).
    #[test]
    fn test_vshow_on_sub_widget_component() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    model { var active_path str = "" }
    view {
        col {
            for tab in .tabs {
                EditorTab(key: tab.path, path: tab.path, show: tab.path == .active_path)
            }
        }
    }
}
"#,
            &["EditorTab"],
        );
        assert!(
            sfc.contains(r#"v-show="tab.path == active_path""#),
            "v-show on component:\n{}",
            sfc
        );
        assert!(!sfc.contains(":show="), "no :show binding leaks:\n{}", sfc);
    }

    /// v-show on a dyn (`<component :is>`) node.
    #[test]
    fn test_vshow_on_dyn_component() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var open bool = false }
    view {
        col {
            dyn (.Teleport) {
                show: .open
                text "overlay"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-show="open""#),
            "v-show on dyn component:\n{}",
            sfc
        );
        assert!(!sfc.contains(":show="), "no :show binding leaks:\n{}", sfc);
    }

    // ====================================================================
    // v-html (plan 012 P2 gap 22) — on a PLAIN HTML element, `html: <expr>`
    // emits `v-html="<expr>"` instead of a `:html` binding. Vue renders the
    // raw HTML string as the element's inner content; any children/text are
    // ignored at runtime (R014 warns on that conflict).
    //
    // dyn (`<component :is>`) and component-instantiation paths do NOT
    // intercept: `html:` passes through as a plain `:html` prop. v-html on
    // a component would shadow its slot and steal the prop from ext
    // functional components using the `innerHTML` pattern (jade's gap-22
    // escape hatch — e2e-verified regression when intercepted).
    // ====================================================================

    /// Dynamic content on a plain element (brace prop form).
    #[test]
    fn test_vhtml_plain_element_dynamic() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var content str = "" }
    view {
        col {
            div(html: .content, class: "preview") { }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-html="content""#),
            "v-html emitted with bound expr:\n{}",
            sfc
        );
        assert!(!sfc.contains(":html="), "no :html binding leaks:\n{}", sfc);
    }

    /// `html:` on a dyn (`<component :is>`) node is NOT intercepted: it
    /// passes through as a plain `:html` prop so ext functional components
    /// (the `h('div', { innerHTML: props.html })` pattern) keep receiving it.
    #[test]
    fn test_vhtml_on_dyn_component() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var content str = "" }
    view {
        col {
            dyn (.Teleport) {
                html: .content
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#":html="content""#),
            "html passes through as :html prop on dyn:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("v-html="),
            "no v-html on dyn component:\n{}",
            sfc
        );
    }

    /// `html:` on a sub-widget component instantiation is NOT intercepted:
    /// passes through as `:html` (v-html on a component would shadow its
    /// slot and steal the prop).
    #[test]
    fn test_vhtml_on_sub_widget_component() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
use tab: EditorTab

widget App {
    model { var tabs list = [] }
    model { var content str = "" }
    view {
        col {
            for tab in .tabs {
                EditorTab(key: tab.path, path: tab.path, html: .content)
            }
        }
    }
}
"#,
            &["EditorTab"],
        );
        assert!(
            sfc.contains(r#":html="content""#),
            "html passes through as :html prop on component:\n{}",
            sfc
        );
        assert!(!sfc.contains("v-html="), "no v-html on component:\n{}", sfc);
    }

    /// Locks the jade ext functional-component escape hatch (the exact
    /// gap-22 pattern that regressed when dyn intercepted `html:`):
    /// `HtmlDiv = (props) => h('div', { innerHTML: props.html ?? '' })`
    /// driven by `dyn (.HtmlDiv) { html: r.html }` in a for-loop must keep
    /// its `:html="r.html"` prop channel.
    #[test]
    fn test_vhtml_dyn_passthrough_ext_component_pattern() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var rows list = [] }
    view {
        col {
            for r in .rows {
                dyn (.HtmlDiv) {
                    class: "md-body"
                    html: r.html
                }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#":html="r.html""#),
            "ext component receives the html prop:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("v-html="),
            "no v-html interception on dyn:\n{}",
            sfc
        );
    }

    /// Conflict: html: + child nodes → R014 warning. v-html is still
    /// emitted and wins; the children are emitted too (Vue ignores them
    /// at runtime).
    #[test]
    fn test_vhtml_conflicting_children_warns_r014() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget VHtmlConflictProbe {
    model { var content str = "" }
    view {
        col {
            div(html: .content) {
                text "ignored"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-html="content""#),
            "v-html still emitted despite children:\n{sfc}"
        );
        let r014 = warnings_for_rule(&warnings, "R014");
        assert_eq!(
            r014.len(),
            1,
            "one R014 conflict warning, got: {warnings:?}"
        );
        assert!(
            r014[0].message.contains("div"),
            "warning names the element tag: {}",
            r014[0].message
        );
        assert!(
            r014[0].message.contains("children ignored"),
            "warning explains the conflict: {}",
            r014[0].message
        );
    }

    /// Conflict via the `text:` prop form also warns R014.
    #[test]
    fn test_vhtml_conflicting_text_prop_warns_r014() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget VHtmlTextConflictProbe {
    model { var content str = "" }
    view {
        col {
            div(html: .content, text: "ignored") { }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"v-html="content""#),
            "v-html still emitted despite text prop:\n{sfc}"
        );
        let r014 = warnings_for_rule(&warnings, "R014");
        assert_eq!(
            r014.len(),
            1,
            "one R014 conflict warning, got: {warnings:?}"
        );
        assert!(
            r014[0].message.contains("children ignored"),
            "warning explains the conflict: {}",
            r014[0].message
        );
    }

    /// v-html on a plain element keeps `?` propagation: an unsupported
    /// content form is a hard codegen error, not a degraded binding.
    #[test]
    fn test_p013_catchall_vhtml_is_hard_error() {
        let ast = crate::parse_ui(
            r#"
widget VHtmlCatchAllProbe {
    model {
        var a str = "x"
        var b str = "y"
    }
    view {
        col {
            span (html: .a ?? .b) {
                text "label"
            }
        }
    }
}
"#,
        )
        .expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new();
        let err = gen
            .generate(&widget)
            .expect_err("unsupported v-html expr must fail codegen");
        assert!(
            matches!(err, GenError::UnsupportedExpr(_)),
            "expected UnsupportedExpr, got: {err:?}"
        );
    }

    // ====================================================================
    // teleport element (plan 012 P2 gap 27) — Vue's built-in <Teleport>.
    // `teleport (to: "body") { ... }` emits `<Teleport to="body">...</Teleport>`
    // instead of silently degrading to `<div :to="'body'">` via the
    // unknown-tag fallback in map_tag.
    // ====================================================================

    /// Static string target: to: "body" → to="body", children recursed.
    #[test]
    fn test_teleport_static_to() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    view {
        teleport (to: "body") {
            div { text "x" }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<Teleport to="body">"#),
            "Teleport with static to:\n{}",
            sfc
        );
        assert!(sfc.contains("</Teleport>"), "Teleport closed:\n{}", sfc);
        assert!(sfc.contains("x"), "children emitted:\n{}", sfc);
        assert!(
            !sfc.contains(r#"<div :to="#),
            "no unknown-tag div fallback:\n{}",
            sfc
        );
    }

    /// Expression target: to: .target → :to="target".
    #[test]
    fn test_teleport_expr_to() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var target str = "body" }
    view {
        teleport (to: .target) {
            text "overlay"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<Teleport :to="target">"#),
            "Teleport with bound to:\n{}",
            sfc
        );
    }

    /// Optional disabled toggle: disabled: .off → :disabled="off".
    #[test]
    fn test_teleport_disabled_expr() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    model { var off bool = false }
    view {
        teleport (to: "body", disabled: .off) {
            text "overlay"
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<Teleport to="body" :disabled="off">"#),
            "Teleport with bound disabled:\n{}",
            sfc
        );
    }

    /// teleport nested inside layout containers works normally.
    #[test]
    fn test_teleport_nested_in_containers() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    view {
        col {
            row {
                teleport (to: "body") {
                    div { text "nested" }
                }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(r#"<Teleport to="body">"#),
            "Teleport nested in col/row:\n{}",
            sfc
        );
        assert!(sfc.contains("nested"), "children emitted:\n{}", sfc);
        assert!(
            !sfc.contains(r#"<div :to="#),
            "no unknown-tag div fallback:\n{}",
            sfc
        );
    }

    /// Missing `to` prop warns R015 (Vue's <Teleport> is a no-op without it).
    #[test]
    fn test_teleport_missing_to_warns_r015() {
        let (_sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget TeleportNoToProbe {
    view {
        teleport {
            text "nowhere"
        }
    }
}
"#,
        );
        let r015 = warnings_for_rule(&warnings, "R015");
        assert_eq!(r015.len(), 1, "one R015 warning, got: {warnings:?}");
        assert!(
            r015[0].message.contains("to"),
            "warning names the missing prop: {}",
            r015[0].message
        );
    }

    // ====================================================================
    // try/catch/finally in handler bodies (gap 4) — Plan 010 gave the parser
    // `try { } catch (e) { }`; Plan 012 P2 adds `finally { }` and the
    // ts_adapter emission (previously Stmt::Try fell into the a2ts fallback,
    // which has no Try case, and was SILENTLY DROPPED from handlers).
    // ====================================================================

    /// Widget handler: try/catch/finally emits real JS with AURA-aware
    /// bodies (state refs → .value) in all three clauses.
    #[test]
    fn test_try_catch_finally_in_widget_handler() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget App {
    msg Msg { Save }
    model {
        var busy bool = false
        var error str = ""
    }
    view { col { button "save" { onclick: .Save } } }
    on {
        .Save -> {
            try {
                .error = ""
                .busy = true
            } catch (e) {
                .error = "failed"
            } finally {
                .busy = false
            }
        }
    }
}
"#,
        );
        assert!(sfc.contains("try {"), "try emitted:\n{}", sfc);
        assert!(sfc.contains("catch (e) {"), "catch with binding:\n{}", sfc);
        assert!(sfc.contains("finally {"), "finally emitted:\n{}", sfc);
        assert!(
            sfc.contains("error.value = ''"),
            "state ref in try body:\n{}",
            sfc
        );
        assert!(
            sfc.contains("error.value = 'failed'"),
            "state ref in catch body:\n{}",
            sfc
        );
        assert!(
            sfc.contains("busy.value = false"),
            "state ref in finally body:\n{}",
            sfc
        );
    }

    /// Store handler (jade's actual gap-4 site): try/catch in an on-block
    /// survives into the generated composable.
    #[test]
    fn test_try_catch_in_store_handler() {
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store Docs {
    model { var error str = "" }
    msg Msg { Save(str) }
    on {
        .Save(args) -> {
            try {
                .error = ""
            } catch (e) {
                .error = "failed"
            }
        }
    }
}
"#,
        ));
        assert!(code.contains("try {"), "store try emitted:\n{}", code);
        assert!(
            code.contains("catch (e) {"),
            "store catch emitted:\n{}",
            code
        );
        assert!(
            code.contains("error.value = 'failed'"),
            "state ref in catch:\n{}",
            code
        );
    }

    /// Plan 043 M5 #3: an `if / else if / else if / else` chain must flatten
    /// into contiguous sibling `<template>` nodes (`v-if` → `v-else-if` →
    /// `v-else`), not nest as `<template v-else><template v-if>`.
    #[test]
    fn test_else_if_chain_flattens_to_v_else_if() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Dispatch {
    model { var kind str = "Table" }
    view {
        if .kind == "Table" {
            text "T"
        } else if .kind == "Record" {
            text "R"
        } else if .kind == "Code" {
            text "C"
        } else {
            text "Other"
        }
    }
}
"#,
        );
        // Head of the chain.
        assert!(
            sfc.contains(r#"<template v-if="kind == 'Table'">"#),
            "chain head v-if:\n{}",
            sfc
        );
        // Both continuations flatten to v-else-if (NOT nested v-else + v-if).
        assert!(
            sfc.contains(r#"<template v-else-if="kind == 'Record'">"#),
            "first continuation v-else-if:\n{}",
            sfc
        );
        assert!(
            sfc.contains(r#"<template v-else-if="kind == 'Code'">"#),
            "second continuation v-else-if:\n{}",
            sfc
        );
        // The bug symptom: a v-else immediately followed by a nested v-if.
        assert!(
            !sfc.contains("<template v-else>\n<template v-if="),
            "no nested v-else>v-if:\n{}",
            sfc
        );
        // Final else arm.
        assert!(
            sfc.contains("<template v-else>"),
            "trailing else arm:\n{}",
            sfc
        );
    }

    /// Plan 012 batch F (gap 54): the jade three-branch shape —
    /// loading / error / content — emits v-if / v-else-if / v-else as
    /// contiguous siblings, in source order (Vue requires adjacency).
    #[test]
    fn test_view_else_if_three_branch_loading_error_content() {
        // Real parse path.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Triple {
    model {
        var loading bool = false
        var error str = ""
    }
    view {
        if .loading {
            text "Loading..."
        } else if .error != "" {
            text "Error"
        } else {
            text "Canvas"
        }
    }
}
"#,
        );
        let pos_if = sfc
            .find(r#"<template v-if="loading">"#)
            .expect("v-if arm present:\n");
        let pos_elseif = sfc
            .find(r#"<template v-else-if="error != ''">"#)
            .expect("v-else-if arm present");
        let pos_else = sfc.find("<template v-else>").expect("v-else arm present");
        // Chain arms emitted in source order (adjacent siblings).
        assert!(
            pos_if < pos_elseif && pos_elseif < pos_else,
            "arms in source order: if={} elseif={} else={}\n{}",
            pos_if,
            pos_elseif,
            pos_else,
            sfc
        );
        // Exactly one chain: the else-if arm is a continuation, not a fresh v-if.
        assert_eq!(
            sfc.matches("<template v-if=").count(),
            1,
            "single chain head:\n{}",
            sfc
        );
        // Branch contents landed in their arms.
        assert!(sfc.contains("Loading..."), "then arm:\n{}", sfc);
        assert!(sfc.contains("Error"), "else-if arm:\n{}", sfc);
        assert!(sfc.contains("Canvas"), "else arm:\n{}", sfc);
    }

    /// Plan 012 batch F (gap 54): two-branch `if / else` (no else-if) emits
    /// v-if + v-else as adjacent siblings.
    #[test]
    fn test_view_if_else_single_else() {
        // Real parse path.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget LoadState {
    model { var loading bool = true }
    view {
        if .loading {
            text "loading"
        } else {
            text "content"
        }
    }
}
"#,
        );
        let pos_if = sfc
            .find(r#"<template v-if="loading">"#)
            .expect("v-if arm present");
        let pos_else = sfc.find("<template v-else>").expect("v-else arm present");
        assert!(pos_if < pos_else, "if before else:\n{}", sfc);
        assert!(
            !sfc.contains("v-else-if"),
            "no v-else-if in a two-branch chain:\n{}",
            sfc
        );
    }

    /// Plan 012 batch F (gap 54): else-if chain arms compose freely — a for
    /// loop inside the v-if arm, a `show:` prop inside the v-else-if arm, and
    /// a nested layout in the else arm. Props do not interact with the chain.
    #[test]
    fn test_view_else_if_chain_composes_loop_and_props() {
        // Real parse path.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Mixed {
    model {
        var kind str = "list"
        var items list = []
        var active bool = true
    }
    view {
        if .kind == "list" {
            for item in .items {
                text "row"
            }
        } else if .kind == "grid" {
            button(show: .active) { text "grid" }
        } else {
            col { text "empty" }
        }
    }
}
"#,
        );
        // Chain shape.
        assert!(
            sfc.contains(r#"<template v-if="kind == 'list'">"#),
            "v-if arm:\n{}",
            sfc
        );
        assert!(
            sfc.contains(r#"<template v-else-if="kind == 'grid'">"#),
            "v-else-if arm:\n{}",
            sfc
        );
        assert!(sfc.contains("<template v-else>"), "v-else arm:\n{}", sfc);
        // The for loop survives inside the v-if arm.
        assert!(
            sfc.contains(r#"v-for="item in items""#),
            "loop inside v-if arm:\n{}",
            sfc
        );
        // show: prop still emits v-show inside the v-else-if arm.
        assert!(
            sfc.contains(r#"v-show="active""#),
            "show: prop inside v-else-if arm:\n{}",
            sfc
        );
        // Else arm content present.
        assert!(sfc.contains("empty"), "else arm content:\n{}", sfc);
    }

    /// Plan 043 M5 #2: a multi-statement computed body must render its logic
    /// (not collapse to `undefined`). Previously expr_to_js had no Block branch
    /// so `x => { ...; return y }` emitted `computed(() => undefined)`.
    #[test]
    fn test_computed_multiline_body_renders_js() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Counter {
    model { count int = 0 }
    computed {
        summary => {
            var label = "count="
            return label
        }
    }
    view { col { text "hi" } }
}
"#,
        );
        // The computed body must keep its logic — the `return label` and the
        // local binding must survive, not be replaced by `undefined`.
        assert!(
            sfc.contains("const summary = computed"),
            "computed wrapper present:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("computed(() => undefined)"),
            "multiline body must not collapse to undefined:\n{}",
            sfc
        );
        assert!(
            sfc.contains("let label"),
            "local binding from block body survives:\n{}",
            sfc
        );
        assert!(
            sfc.contains("return label"),
            "return statement from block body survives:\n{}",
            sfc
        );
    }

    /// Plan 043 store-codegen / Plan 012 Batch B (gap 2): the 015-notes-specific
    /// `all_tags` getter auto-inject was removed entirely — a store only gets an
    /// PLAN-041 T-03: per-App store factory + context-aware facade.
    /// - state refs/watchers live inside `create{Name}Store()` (one instance
    ///   per App context);
    /// - `use{Name}Store()` keeps its export signature, resolves through the
    ///   App store context, and falls back to a lazy module singleton only
    ///   for mounts that provide no context (named legacy boundary).
    #[test]
    fn test_plan041_store_factory_and_context_facade() {
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store TabsStore {
    model {
        var active_id str = ""
        var saved bool = true
    }
    watch {
        .active_id -> { .saved = false }
    }
}
"#,
        ));

        // Per-App factory wraps the state refs.
        assert!(
            code.contains("export function createTabsStore(): any {"),
            "per-App factory must be emitted, got:\n{}",
            code
        );
        // The factory appears BEFORE the refs (state lives in factory scope).
        let factory_at = code.find("export function createTabsStore").expect("factory");
        let refs_at = code.find("const active_id = ref<string>('')").expect("ref");
        assert!(
            factory_at < refs_at,
            "state refs must live inside the factory scope"
        );
        // Watchers ride the factory (per-App lifetime).
        let watch_at = code.find("watch(active_id, () => {").expect("watch");
        assert!(
            factory_at < watch_at && watch_at < code.rfind('}').unwrap_or(watch_at),
            "watchers must be emitted inside the factory scope"
        );

        // Context-aware facade keeps the export signature.
        assert!(
            code.contains("export function useTabsStore(): any {"),
            "use facade must keep its export signature, got:\n{}",
            code
        );
        let use_at = code.find("export function useTabsStore").expect("use fn");
        assert!(
            use_at > factory_at,
            "use facade must be emitted after the factory"
        );
        assert!(
            code.contains("inject<any | null>('__autoStoreContext', null)"),
            "context resolution must inject the App store context, got:\n{}",
            code
        );
        assert!(
            code.contains("__ctx.use('TabsStore', createTabsStore)"),
            "context hit must resolve via ctx.use with the factory, got:\n{}",
            code
        );
        assert!(
            code.contains("__legacyTabsStoreHolder.instance = createTabsStore()"),
            "context miss must fall back to the lazy legacy singleton, got:\n{}",
            code
        );
    }

    /// `all_tags` getter when it declares one in its `computed {}` block. This
    /// test pins that a plain store composable contains no `all_tags` (and that
    /// array inits render as `[]`).
    #[test]
    fn test_store_composable_no_notes_no_all_tags() {
        // Real parse path (plan 012 batch C).
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store ShellStore {
    model {
        var blocks list = []
        var cwd str = ""
    }
}
"#,
        ));

        // Array init rendered as [].
        assert!(
            code.contains("const blocks = ref<any>([])"),
            "array init should render as [], got:\n{}",
            code
        );
        // No all_tags getter (no `notes` state var).
        assert!(
            !code.contains("all_tags"),
            "all_tags must NOT be injected for a store without `notes`, got:\n{}",
            code
        );
        // The store composable function name follows the use{Name}Store convention.
        assert!(
            code.contains("export function useShellStore()"),
            "composable function name, got:\n{}",
            code
        );
    }

    /// Plan 012 Batch G (gap 15): a store `var x map = {}` initializer must
    /// emit the object literal verbatim (`ref({})`), matching the widget-side
    /// behavior. Previously `store_init_to_js` had no `Expr::Object` arm, so
    /// map-typed inits silently degraded to `ref(null)` and any early `.x`
    /// field access null-deref'd at runtime.
    #[test]
    fn test_store_map_literal_init_preserved() {
        // Real parse path.
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store Graph {
    model {
        var settings map = {}
        var ui map = { mode: "editor", depth: 1, dark: true }
        var blocks list = []
        var cwd str = ""
    }
}
"#,
        ));

        assert!(
            code.contains("const settings = ref<any>({})"),
            "empty map init must render as an object literal, got:\n{}",
            code
        );
        assert!(
            code.contains("const ui = ref<any>({ mode: 'editor', depth: 1, dark: true })"),
            "populated map init must render its pairs, got:\n{}",
            code
        );
        // Existing literal kinds keep working unchanged. (Plan 055 E(a): a
        // str-typed state ref is now typed `ref<string>`, not `ref<any>`.)
        assert!(
            code.contains("const blocks = ref<any>([])"),
            "array init:\n{}",
            code
        );
        assert!(
            code.contains("const cwd = ref<string>('')"),
            "str init:\n{}",
            code
        );
        // No state var may degrade to null anymore.
        assert!(
            !code.contains("ref<any>(null)"),
            "no map/array/str init may fall back to null, got:\n{}",
            code
        );
    }

    /// Plan 012 Batch G (gap 10 regression guard): a store handler with
    /// `return expr` must parse and emit a real `return <value>` in the
    /// generated action. Returning from a Pinia action is legal, and callers
    /// of the composable action receive the value — this removes the need
    /// to move value-returning methods into facade/ext modules. The path
    /// self-healed once with-value `return` reached the general statement
    /// layer (batch F) — this test pins the store-side end-to-end path.
    #[test]
    fn test_store_handler_return_value() {
        // Real parse path.
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store Counter {
    model {
        var count int = 0
    }
    msg Msg { Bump, GetCount }
    on {
        .Bump -> {
            .count += 1
            return .count
        }
        .GetCount -> {
            return .count
        }
    }
}
"#,
        ));

        assert!(
            code.contains("return count.value;"),
            "return value must emit through state-ref rewriting, got:\n{}",
            code
        );
        // Both actions stay exposed on the composable return object.
        assert!(code.contains("Bump,"), "Bump exposed:\n{}", code);
        assert!(code.contains("GetCount,"), "GetCount exposed:\n{}", code);
    }

    /// Plan 012 Batch G (gap 12): a store-level `watch { ... }` block must
    /// parse, extract, and emit as module-level `watch(...)` calls in the
    /// store composable (with the `watch` import added). Mirrors the
    /// widget-side watch support; replaces the facade-module workaround.
    #[test]
    fn test_store_watch_block_emits_module_watch() {
        // Real parse path.
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store Tabs {
    model {
        var active_id str = ""
        var open_tabs list = []
        var saved bool = true
    }

    watch {
        .active_id -> { .saved = false }
        .open_tabs.deep -> { .saved = false }
    }
}
"#,
        ));

        // PLAN-041 T-03: context-aware imports; watchers live inside the
        // per-App factory (one watcher instance per App context).
        assert!(
            code.contains("import { ref, watch, getCurrentInstance, inject } from 'vue'"),
            "watch import, got:\n{}",
            code
        );
        // Watch calls (per-App factory scope — was module-level singleton).
        assert!(
            code.contains("watch(active_id, () => {"),
            "single-source watch, got:\n{}",
            code
        );
        assert!(
            code.contains("watch(open_tabs, () => {"),
            "deep watch source, got:\n{}",
            code
        );
        assert!(
            code.contains("{ deep: true }"),
            "deep modifier options, got:\n{}",
            code
        );
        // The handler body writes through .value (state-ref rewriting).
        assert!(
            code.contains("saved.value = false"),
            "state-ref rewrite in watch body, got:\n{}",
            code
        );
        // Watch calls are module-level: they must appear BEFORE the
        // composable export function.
        let watch_pos = code.find("watch(active_id").expect("watch call");
        let fn_pos = code
            .find("export function useTabsStore()")
            .expect("export fn");
        assert!(
            watch_pos < fn_pos,
            "module-level watch must precede the composable fn, got:\n{}",
            code
        );
    }

    /// multi-type payload (`Open(str, str)`) must parse, and the handler
    /// `.Open(a, b) ->` must bind both payload slots as action parameters.
    /// This self-healed via the Plan 043 M5 msg-payload work (MsgDecl.payload
    /// is `Vec<Type>`); this test pins the store-side end-to-end path so it
    /// cannot regress back to single-type-only.
    #[test]
    fn test_store_msg_multi_type_payload() {
        // Real parse path.
        let store = store_from_src(
            r#"
store Files {
    model {
        var path str = ""
    }
    msg Msg { Open(str, str), Close }
    on {
        .Open(a, b) -> {
            .path = a
        }
        .Close -> {
            .path = ""
        }
    }
}
"#,
        );

        // AST level: the variant carries both payload types.
        let open = store
            .messages
            .iter()
            .flat_map(|m| m.variants.iter())
            .find(|v| v.name == "Open")
            .expect("Open variant");
        assert_eq!(open.payload.len(), 2, "Open payload types: {:?}", open);

        // Handler params bind both slots.
        let params = store
            .handler_params
            .get(".Open(a, b)")
            .expect("Open handler params");
        assert_eq!(params, &vec!["a".to_string(), "b".to_string()]);

        // Codegen: the action takes both parameters. (Plan 055 E(b): scalar
        // payload slots are typed — (str, str) → (a: string, b: string).)
        let code = VueGenerator::generate_store_composable(&store);
        assert!(
            code.contains("const Open = (a: string, b: string) =>"),
            "two-arg action signature, got:\n{}",
            code
        );
        assert!(
            code.contains("const Close = () =>"),
            "zero-arg action signature, got:\n{}",
            code
        );
    }

    /// Parse a store source and extract its AuraStore (real parse path,
    /// plan 012 batch C). NOTE: the standalone parse path does NOT populate
    /// `api_imports` / `stream_endpoints` — the full build fills those from
    /// project type info (the `consume ~Stream<T>` pattern) — so the two SSE
    /// tests below legitimately keep hand-built AuraStore literals.
    fn store_from_src(src: &str) -> crate::aura::AuraStore {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("store source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::StoreDecl(d) => Some(d),
                _ => None,
            })
            .expect("store decl");
        crate::aura::extract_store_from_decl(decl).expect("extract store")
    }

    /// Plan 043 M5 G1: a store that imports `stream` and declares
    /// RunOutput/RunResult (the "consume ~Stream<T>" pattern) must wire a
    /// single EventSource('/api/stream') that dispatches command_output /
    /// command_result into those actions. Without this, run_command results
    /// never reach the UI.
    #[test]
    fn test_store_composable_wires_sse_stream() {
        // KEEP hand-built (plan 012 batch C): this test needs
        // `stream_endpoints` + `api_imports` populated, which the standalone
        // parse path cannot do (extract_store_from_decl leaves both empty;
        // the full build fills them from project type info).
        use crate::aura::{AuraStateDef, AuraStore};
        use auto_lang_core::ast::Expr;
        use std::collections::HashMap;

        let store = AuraStore {
            name: "ShellStore".to_string(),
            state_vars: vec![AuraStateDef {
                name: "blocks".to_string(),
                type_info: auto_lang_core::ast::Type::Unknown,
                initial: Expr::Array(vec![]),
                decorators: vec![],
            }],
            messages: vec![],
            timers: Vec::new(),
            handlers: std::collections::BTreeMap::from([
                (
                    ".RunOutput(output)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
                (
                    ".RunResult(result)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
            ]),
            handler_params: HashMap::from([
                (".RunOutput(output)".to_string(), vec!["output".to_string()]),
                (".RunResult(result)".to_string(), vec!["result".to_string()]),
            ]),
            api_imports: vec!["stream".to_string(), "run_command".to_string()],
            stream_endpoints: vec![crate::aura::StreamEndpoint {
                fn_name: "stream".to_string(),
                path: "/api/stream".to_string(),
                item_type: "ShellEvent".to_string(),
                discriminator: "event".to_string(),
                variants: vec![],
            }],
            stream_handlers: vec![],
            computed: vec![],
            watchers: vec![],
            module_fns: vec![],
            sibling_stores: Vec::new(),
        };

        let code = VueGenerator::generate_store_composable(&store);

        // Module-level single-connection guard (Plan musk-022: per-path name).
        assert!(
            code.contains("let __streamConnected_api_stream = false;"),
            "module guard:\n{}",
            code
        );
        // The connection is opened inside the composable, guarded, before return.
        assert!(
            code.contains("if (!__streamConnected_api_stream) {"),
            "guard check:\n{}",
            code
        );
        assert!(
            code.contains("new EventSource('/api/stream')"),
            "EventSource:\n{}",
            code
        );
        // Dispatch into the store's actions (legacy fallback: empty variants →
        // command_output/RunOutput + command_result/RunResult via data.event).
        assert!(
            code.contains("if (data.event === 'command_output') RunOutput(data);"),
            "output dispatch:\n{}",
            code
        );
        assert!(
            code.contains("else if (data.event === 'command_result') RunResult(data);"),
            "result dispatch:\n{}",
            code
        );
    }

    /// Plan musk-022 Phase 1: a store with an endpoint whose inner type resolves
    /// to a `#[serde(tag = "type", rename_all = "snake_case")] pub tag` emits a
    /// data-driven dispatch keyed on `data.type` with one clause per variant
    /// (snake_case wire value → PascalCase action name). This mirrors the
    /// auto-musk forge stream contract.
    #[test]
    fn test_store_composable_sse_multi_variant_data_driven() {
        use crate::aura::{AuraStateDef, AuraStore};
        use auto_lang_core::ast::Expr;
        use std::collections::HashMap;

        let store = AuraStore {
            name: "ForgeStore".to_string(),
            state_vars: vec![AuraStateDef {
                name: "messages".to_string(),
                type_info: auto_lang_core::ast::Type::Unknown,
                initial: Expr::Array(vec![]),
                decorators: vec![],
            }],
            messages: vec![],
            timers: Vec::new(),
            handlers: std::collections::BTreeMap::from([
                (
                    ".Delta(data)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
                (
                    ".ToolCall(data)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
                (
                    ".Done(data)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
            ]),
            handler_params: HashMap::from([
                (".Delta(data)".to_string(), vec!["data".to_string()]),
                (".ToolCall(data)".to_string(), vec!["data".to_string()]),
                (".Done(data)".to_string(), vec!["data".to_string()]),
            ]),
            api_imports: vec!["chat_stream".to_string()],
            stream_endpoints: vec![crate::aura::StreamEndpoint {
                fn_name: "chat_stream".to_string(),
                path: "/api/chats/session/{id}/stream".to_string(),
                item_type: "SseEventDto".to_string(),
                discriminator: "type".to_string(),
                variants: vec![
                    ("delta".to_string(), "Delta".to_string()),
                    ("tool_call".to_string(), "ToolCall".to_string()),
                    ("done".to_string(), "Done".to_string()),
                ],
            }],
            stream_handlers: vec![],
            computed: vec![],
            watchers: vec![],
            module_fns: vec![],
            sibling_stores: Vec::new(),
        };

        let code = VueGenerator::generate_store_composable(&store);

        // Per-path guard (slashes/braces collapse to _; {id} → _id_).
        assert!(
            code.contains("let __streamConnected_api_chats_session__id__stream = false;"),
            "guard var:\n{}",
            code
        );
        assert!(
            code.contains("new EventSource('/api/chats/session/{id}/stream')"),
            "EventSource:\n{}",
            code
        );
        // Data-driven dispatch keyed on data.type (NOT data.event).
        assert!(
            code.contains("if (data.type === 'delta') Delta(data);"),
            "delta dispatch:\n{}",
            code
        );
        assert!(
            code.contains("else if (data.type === 'tool_call') ToolCall(data);"),
            "tool_call dispatch:\n{}",
            code
        );
        assert!(
            code.contains("else if (data.type === 'done') Done(data);"),
            "done dispatch:\n{}",
            code
        );
        // The legacy command_output/command_result must NOT appear here.
        assert!(!code.contains("command_output"), "legacy leak:\n{}", code);
    }

    /// Plan musk-022 Phase 1: multi-endpoint store emits one EventSource block
    /// per endpoint, each with its own per-path guard.
    #[test]
    fn test_store_composable_sse_multi_endpoint() {
        use crate::aura::AuraStore;
        use std::collections::HashMap;

        let store = AuraStore {
            name: "MultiStreamStore".to_string(),
            state_vars: vec![],
            messages: vec![],
            timers: Vec::new(),
            handlers: std::collections::BTreeMap::from([
                (
                    ".RunOutput(data)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
                (
                    ".RunResult(data)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
            ]),
            handler_params: HashMap::from([
                (".RunOutput(data)".to_string(), vec!["data".to_string()]),
                (".RunResult(data)".to_string(), vec!["data".to_string()]),
            ]),
            // Both streaming endpoints are imported by this store, so both get
            // SSE wiring. (Plan musk-022 Phase 4 filters stream_endpoints by
            // api_imports — a store only wires the streams it actually imports,
            // so AuthStore doesn't accidentally pick up chat_stream's dispatchers.)
            api_imports: vec!["stream".to_string(), "events".to_string()],
            stream_endpoints: vec![
                crate::aura::StreamEndpoint {
                    fn_name: "stream".to_string(),
                    path: "/api/stream".to_string(),
                    item_type: "ShellEvent".to_string(),
                    discriminator: "event".to_string(),
                    variants: vec![],
                },
                crate::aura::StreamEndpoint {
                    fn_name: "events".to_string(),
                    path: "/api/events".to_string(),
                    item_type: "Event".to_string(),
                    discriminator: "event".to_string(),
                    variants: vec![],
                },
            ],
            stream_handlers: vec![],
            computed: vec![],
            watchers: vec![],
            module_fns: vec![],
            sibling_stores: Vec::new(),
        };

        let code = VueGenerator::generate_store_composable(&store);
        // Two distinct per-path guards.
        assert!(
            code.contains("let __streamConnected_api_stream = false;"),
            "guard 1:\n{}",
            code
        );
        assert!(
            code.contains("let __streamConnected_api_events = false;"),
            "guard 2:\n{}",
            code
        );
        // Two EventSource openings.
        assert!(
            code.contains("new EventSource('/api/stream')"),
            "es 1:\n{}",
            code
        );
        assert!(
            code.contains("new EventSource('/api/events')"),
            "es 2:\n{}",
            code
        );
    }

    /// Plan 043 M5 G1 negative: without the `stream` api import, no
    /// EventSource wiring is generated (a store with RunOutput/RunResult but
    /// no stream API must stay plain).
    #[test]
    fn test_store_composable_no_sse_without_stream_api() {
        // Real parse path (plan 012 batch C): a store declaring the
        // RunOutput/RunResult consume-pattern handlers but no stream
        // endpoint stays plain. (The previous version also hand-set
        // `api_imports: ["run_command"]` — the standalone parse path cannot
        // populate api_imports, but the asserted contract is unchanged.)
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store PlainStore {
    msg Msg { RunOutput(output), RunResult(result) }
    on {
        .RunOutput(output) -> { }
        .RunResult(result) -> { }
    }
}
"#,
        ));

        assert!(
            !code.contains("EventSource"),
            "no EventSource without stream api:\n{}",
            code
        );
        assert!(
            !code.contains("__streamConnected"),
            "no guard without stream api:\n{}",
            code
        );
    }

    /// Plan 043 stream phase: SSE wiring is TYPE-DRIVEN via `stream_endpoints`,
    /// not the old name-heuristic. The EventSource URL must come from the
    /// endpoint's `path` (here a custom "/events"), and must NOT require the
    /// import to be literally named "stream".
    #[test]
    fn test_store_composable_sse_type_driven() {
        // KEEP hand-built (plan 012 batch C): needs `stream_endpoints`
        // populated with a custom fn name/path — internal-only state that
        // the standalone parse path cannot produce (see store_from_src).
        use crate::aura::{AuraStore, StreamEndpoint};
        use std::collections::HashMap;

        let store = AuraStore {
            name: "MyStore".to_string(),
            state_vars: vec![],
            messages: vec![],
            timers: Vec::new(),
            handlers: std::collections::BTreeMap::from([
                (
                    ".RunOutput(output)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
                (
                    ".RunResult(result)".to_string(),
                    crate::aura::LogicPayload::AstStmts(vec![]),
                ),
            ]),
            handler_params: HashMap::new(),
            // NOTE: import name is "subscribe" (NOT "stream") — old heuristic
            // would NOT wire SSE. Type-driven wiring keys off stream_endpoints.
            api_imports: vec!["subscribe".to_string()],
            stream_endpoints: vec![StreamEndpoint {
                fn_name: "subscribe".to_string(),
                path: "/events".to_string(),
                item_type: "ShellEvent".to_string(),
                discriminator: "event".to_string(),
                variants: vec![],
            }],
            stream_handlers: vec![],
            computed: vec![],
            watchers: vec![],
            module_fns: vec![],
            sibling_stores: Vec::new(),
        };

        let code = VueGenerator::generate_store_composable(&store);

        // Wiring fires because a stream endpoint is declared, regardless of name.
        // (Plan musk-022: guard is now per-path, so __streamConnected_events.)
        assert!(
            code.contains("let __streamConnected_events = false;"),
            "module guard:\n{}",
            code
        );
        // Path comes from the endpoint, not a hardcoded "/api/stream".
        assert!(
            code.contains("new EventSource('/events')"),
            "custom path:\n{}",
            code
        );
        assert!(
            !code.contains("'/api/stream'"),
            "must not use hardcoded path:\n{}",
            code
        );
        // Dispatch unchanged (legacy discriminator-route on data.event).
        assert!(code.contains("RunOutput(data);"), "dispatch:\n{}", code);
        // The streaming fn ("subscribe") is consumed via SSE, so it must NOT appear
        // in the api import line (api.ts does not export it).
        assert!(
            !code.contains("import { subscribe"),
            "stream fn excluded from import:\n{}",
            code
        );
    }

    // ====================================================================
    // Plan 043 M5 R4 — sub-widget callback events: parent binds `on_run` →
    // `@Run` (matching the child's emit), and the child drops the redundant
    // `on_run` prop from defineProps (the callback arrives via the emit).
    // ====================================================================

    #[test]
    fn test_sub_widget_on_prop_binds_pascal_emit_name() {
        // Parent side: `on_run: .RunCmd` on a known sub-widget must emit
        // `@Run="RunCmd"`, NOT the DOM fallback `@_run` (which the child
        // never fires — it emits the msg variant name `Run`).
        let parent = gen_sfc_with_sub_widgets(
            r#"
widget App {
    model { var n int = 0 }
    view {
        col {
            PromptBar(
                on_run: .RunCmd,
                on_clear: .Reset
            )
        }
    }
    on {
        .RunCmd(cmd) -> { .n = .n + 1 }
        .Reset -> { .n = 0 }
    }
}
"#,
            &["PromptBar"],
        );
        assert!(
            parent.contains("@Run=\"RunCmd\""),
            "parent binds @Run (Pascal msg variant), not @_run:\n{}",
            parent
        );
        assert!(
            parent.contains("@Clear=\"Reset\""),
            "parent binds @Clear for on_clear:\n{}",
            parent
        );
        assert!(
            !parent.contains("@_run") && !parent.contains("@_clear"),
            "no DOM-fallback `@_*` bindings for callback props:\n{}",
            parent
        );
    }

    #[test]
    fn test_sub_widget_omits_emitted_callback_prop_from_define_props() {
        // Child side: `on_run: msg` with a matching `Run(str)` msg variant is
        // delivered via the emit — declaring it as a required prop would make
        // the parent's `{ ..., @Run }` miss `on_run` (TS2345).
        let child = gen_sfc_from_widget_src(
            r#"
widget PromptBar(cwd: str, on_run: msg, on_clear: msg, on_exit: msg) {
    msg Msg { Run(str), Clear, Exit }
    model { var input str = "" }
    view {
        col {
            input {
                value: .input
                onenter: .Run(.input)
            }
        }
    }
    on {
        .Run(cmd) -> { }
        .Clear -> { }
        .Exit -> { }
    }
}
"#,
        );
        assert!(
            !child.contains("on_run") && !child.contains("on_clear") && !child.contains("on_exit"),
            "emitted-callback props dropped from defineProps:\n{}",
            child
        );
        assert!(
            child.contains("cwd: string"),
            "non-callback props stay in defineProps:\n{}",
            child
        );
        assert!(
            child.contains("Run: [string]"),
            "child still declares the Run emit with payload:\n{}",
            child
        );
    }

    #[test]
    fn test_pascalcase_fallback_element_on_prop_binds_pascal_emit_name() {
        // Phase 1 front files compile WITHOUT known_sub_widgets — a sibling
        // sub-widget (`use prompt_bar: HistorySearch`) renders via map_tag's
        // PascalCase fallback (plain-element path). Its `on_*` callback props
        // must still bind to the Pascal msg-variant name (`@Run`), not the DOM
        // fallback `@_run` — the child emits `Run`.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Panel(history: []str) {
    msg Msg { Run(str), Close }
    model { var open bool = false }
    view {
        col {
            HistorySearch(
                history: .history,
                open: .open,
                on_run: .Run,
                on_close: .Close
            )
        }
    }
    on {
        .Run(cmd) -> { }
        .Close -> { }
    }
}
"#,
        );
        assert!(
            sfc.contains("@Run=\"Run\""),
            "on_run → @Run on PascalCase fallback element:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@Close=\"Close\""),
            "on_close → @Close on PascalCase fallback element:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("@_run") && !sfc.contains("@_close"),
            "no DOM-fallback @_* bindings on PascalCase components:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Plan 043 §5.9 parity fixes (H1/H2/H5) — codegen parity with hand-written
    // ====================================================================

    /// H2: inside a for-loop, a sub-widget event whose parent msg variant takes
    /// a PAYLOAD must forward the child's emit value via `$event`, NOT the loop
    /// variable (which would clobber e.g. a Rerun(command) with the block object).
    #[test]
    fn test_loop_sub_widget_payload_event_forwards_dollar_event() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
widget BlockList {
    msg Msg { Rerun(str), Stop }
    view {
        col {
            for b in .blocks {
                BlockItem(on_rerun: .Rerun, on_stop: .Stop)
            }
        }
    }
    on {
        .Rerun(cmd) -> { }
        .Stop -> { }
    }
}
"#,
            &["BlockItem"],
        );
        // Rerun(str) takes a payload → forward $event.
        assert!(
            sfc.contains("@Rerun=\"Rerun($event)\""),
            "payload handler must forward $event:\n{}",
            sfc
        );
        // Stop takes no payload → legacy loop-var behavior (b).
        assert!(
            sfc.contains("@Stop=\"Stop(b)\""),
            "no-payload handler still passes loop var:\n{}",
            sfc
        );
    }

    /// H5: a dynamic style expression (string concat like "color: rgb(r,g,b)")
    /// on a text element must render as `:style="<expr>"`, not be silently
    /// dropped. Previously push_style_class only handled Str literals and
    //  Expr::If, dropping Expr::Bina concatenations.
    #[test]
    fn test_text_dynamic_style_concat_renders_style_binding() {
        let sfc = gen_sfc_with_sub_widgets(
            r#"
widget CodeView {
    view {
        col {
            for span in .spans {
                text span.text {
                    style: "color: rgb(" + span.r + "," + span.g + "," + span.b + ")"
                }
            }
        }
    }
}
"#,
            &[],
        );
        assert!(
            sfc.contains(":style=\"'color: rgb(' + span.r + ',' + span.g + ',' + span.b + ')'\""),
            "dynamic style concat must render as :style binding:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Plan 012 Batch A — silent-emission guards (gaps 19/20/30/44/45/47)
    // Every test drives the REAL parse path (Parser → extract → generate);
    // hand-built ASTs caused fake-green tests before.
    // ====================================================================

    /// Same as gen_sfc_from_widget_src, but also returns the warnings
    /// collected through the unified codegen warning channel.
    fn gen_sfc_and_warnings(
        src: &str,
    ) -> (String, Vec<crate::ui_gen::validators::ValidationWarning>) {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).expect("generate SFC");
        (sfc, gen.last_validation_warnings.clone())
    }

    fn warnings_for_rule<'w>(
        warnings: &'w [crate::ui_gen::validators::ValidationWarning],
        rule: &str,
    ) -> Vec<&'w crate::ui_gen::validators::ValidationWarning> {
        warnings.iter().filter(|w| w.rule == rule).collect()
    }

    // --- gap 30: stray comma between view children -----------------------

    #[test]
    fn test_gap30_stray_comma_warns_and_emits_nothing() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget CommaProbe {
    view {
        col {
            button "a"
            ,
            button "b",
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("<div />"),
            "stray commas must not emit junk spacer divs:\n{sfc}"
        );
        let r008 = warnings_for_rule(&warnings, "R008");
        assert!(
            !r008.is_empty(),
            "a stray comma must produce an R008 warning, got: {warnings:?}"
        );
        assert!(
            r008.iter().all(|w| w.message.contains("comma")),
            "R008 message should name the stray comma: {r008:?}"
        );
        // Both buttons still render.
        assert!(sfc.contains(">a</button>"), "button a missing:\n{sfc}");
        assert!(sfc.contains(">b</button>"), "button b missing:\n{sfc}");
    }

    // --- gap 20: dynamic class: expression on plain element --------------

    #[test]
    fn test_gap20_dynamic_class_ref_is_bound() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ClassProbe {
    model { var cls str = "x" }
    view {
        col {
            span {
                class: .cls
                text "label"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":class=\"cls\""),
            "dynamic class: .cls must emit a :class binding (template auto-unwraps refs):\n{sfc}"
        );
    }

    #[test]
    fn test_gap20_dynamic_class_if_emits_ternary() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ClassIfProbe {
    model { var done bool = false }
    view {
        col {
            span {
                class: if .done { "line-through" } else { "text-muted" }
                text "label"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":class=") && sfc.contains("line-through") && sfc.contains("text-muted"),
            "class: if-expr must emit a :class ternary keeping both branches:\n{sfc}"
        );
    }

    // --- Plan 012 P0#13: the three broken `class:` forms ------------------
    // Repro of the downstream dynclass probe: static+dynamic duplication,
    // map form key quoting, `+` concat / array forms emitting `null`.

    /// Form 1: two `class:` props on one element (static string + dynamic
    /// expr). Vue semantics union them; dropping either is silent data loss.
    #[test]
    fn test_p013_duplicate_class_props_merge() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget DupClassProbe {
    model { var busy bool = false }
    view {
        col {
            span {
                class: "static-a"
                class: .busy ? "on" : "off"
                text "label"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("static-a"),
            "static class from the first `class:` prop must survive:\n{sfc}"
        );
        assert!(
            sfc.contains(":class=") && sfc.contains("busy") && sfc.contains("on"),
            "dynamic class from the second `class:` prop must be bound:\n{sfc}"
        );
    }

    /// Form 2: map form keys must be quoted when they aren't valid JS
    /// identifiers (`line-through`), mirroring the `style:` StyleBinding path.
    #[test]
    fn test_p013_class_map_form_quotes_keys() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget MapClassProbe {
    model { var done bool = false }
    view {
        col {
            span {
                class: { "line-through": .done }
                text "label"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("'line-through': done"),
            "class: map form must quote non-identifier keys:\n{sfc}"
        );
        assert!(
            !sfc.contains("{ line-through: done }"),
            "unquoted dashed key is a JS syntax error:\n{sfc}"
        );
    }

    /// Form 3a: array form with a ternary element used to emit literal `null`.
    #[test]
    fn test_p013_class_array_ternary_no_null() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ArrClassProbe {
    model { var busy bool = false }
    view {
        col {
            span {
                class: ["static-arr", .busy ? "on" : "off"]
                text "array form"
            }
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("null"),
            "class: array form must never emit literal null:\n{sfc}"
        );
        assert!(
            sfc.contains("'static-arr'") && sfc.contains("busy ? 'on' : 'off'"),
            "class: array form must bind both elements:\n{sfc}"
        );
    }

    /// Form 3b: string concat `class: "a" + .cls` used to emit literal `null`.
    #[test]
    fn test_p013_class_concat_no_null() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ConcatClassProbe {
    model { var cls str = "x" }
    view {
        col {
            span {
                class: "base-" + .cls
                text "concat"
            }
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("null"),
            "class: concat must never emit literal null:\n{sfc}"
        );
        assert!(
            sfc.contains("'base-' + cls"),
            "class: concat must emit a bound string-concat expr:\n{sfc}"
        );
    }

    /// The shadcn choke point (Batch D, push_native_classes) shares
    /// extract_classes — the array form must be fixed there too.
    #[test]
    fn test_p013_class_array_shadcn_choke_point() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget ArrShadcnProbe {
    model { var busy bool = false }
    view {
        col {
            label (class: ["static-arr", .busy ? "on" : "off"]) {
                text "depth"
            }
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("null"),
            "shadcn choke point must not emit literal null for class arrays:\n{sfc}"
        );
        assert!(
            sfc.contains("'static-arr'") && sfc.contains("busy ? 'on' : 'off'"),
            "shadcn choke point must bind the class array:\n{sfc}"
        );
    }

    // --- Plan 012 P0#13 follow-up: catch-all hardening (R013) --------------
    // The `_ => Ok("null")` catch-all in expr_to_vue_bound_value now returns
    // Err; bound-position call sites either propagate the hard error or warn
    // R013 and keep their old fallback. Probes use `??` (Expr::NullCoalesce),
    // a form with no bound-value arm.

    /// style: map form — the `unwrap_or_else(|_| "false")` site. Before: the
    /// condition silently emitted `null` (the fallback was unreachable).
    /// After: `false` fallback + a loud R013.
    #[test]
    fn test_p013_catchall_style_binding_warns_r013() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget StyleCatchAllProbe {
    model {
        var a str = "x"
        var b str = "y"
    }
    view {
        col {
            span {
                style: { "line-through": .a ?? .b }
                text "label"
            }
        }
    }
}
"#,
        );
        let r013 = warnings_for_rule(&warnings, "R013");
        assert!(
            !r013.is_empty(),
            "unsupported expr in style: binding must warn R013, got: {warnings:?}"
        );
        assert!(
            r013.iter().any(|w| w.message.contains("line-through")),
            "R013 should name the style key: {r013:?}"
        );
        assert!(
            sfc.contains("'line-through': false"),
            "style: binding must keep the `false` fallback:\n{sfc}"
        );
    }

    /// class: expr form — the class-specific site keeps its R011 warning,
    /// now fed by the Err (with the expression's Debug shape) instead of a
    /// literal-"null" string compare.
    #[test]
    fn test_p013_catchall_class_expr_warns_r011() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget ClassCatchAllProbe {
    model {
        var a str = "x"
        var b str = "y"
    }
    view {
        col {
            span {
                class: .a ?? .b
                text "label"
            }
        }
    }
}
"#,
        );
        let r011 = warnings_for_rule(&warnings, "R011");
        assert!(
            !r011.is_empty(),
            "unsupported class: expr must warn R011, got: {warnings:?}"
        );
        assert!(
            r011.iter().any(|w| w.message.contains("NullCoalesce")),
            "R011 should carry the expression's Debug shape: {r011:?}"
        );
        assert!(
            !sfc.contains(":class=\"null\""),
            "class: must never bind literal null:\n{sfc}"
        );
    }

    /// v-show on a plain element keeps `?` propagation: an unsupported
    /// condition form is a hard codegen error, not a degraded binding.
    #[test]
    fn test_p013_catchall_vshow_is_hard_error() {
        let ast = crate::parse_ui(
            r#"
widget VShowCatchAllProbe {
    model {
        var a str = "x"
        var b str = "y"
    }
    view {
        col {
            span (show: .a ?? .b) {
                text "label"
            }
        }
    }
}
"#,
        )
        .expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new();
        let err = gen
            .generate(&widget)
            .expect_err("unsupported v-show condition must fail codegen");
        assert!(
            matches!(err, GenError::UnsupportedExpr(_)),
            "expected UnsupportedExpr, got: {err:?}"
        );
    }

    /// style_obj: — the `unwrap_or_else(|_| "null")` site keeps its `null`
    /// fallback but must warn R013.
    #[test]
    fn test_p013_catchall_style_obj_warns_r013() {
        let (sfc, warnings) = gen_sfc_and_warnings(
            r#"
widget StyleObjCatchAllProbe {
    model {
        var a int = 1
        var b int = 2
    }
    view {
        col {
            span {
                style_obj: { top: .a ?? .b }
                text "label"
            }
        }
    }
}
"#,
        );
        let r013 = warnings_for_rule(&warnings, "R013");
        assert!(
            !r013.is_empty(),
            "unsupported expr in style_obj must warn R013, got: {warnings:?}"
        );
        assert!(
            r013.iter().any(|w| w.message.contains("top")),
            "R013 should name the style_obj key: {r013:?}"
        );
        assert!(
            sfc.contains("top: null"),
            "style_obj must keep the `null` fallback:\n{sfc}"
        );
    }

    // --- Plan 012: label/select keep class on the shadcn path ------------
    // Regression: once `label` was registered in the vue widget registry
    // (6f3fe403, Plan 337 drift guard), shadcn mode routed it through
    // generate_shadcn_attrs, whose label arm never forwarded `class:` —
    // the attribute was silently dropped while div/span/input kept theirs.

    #[test]
    fn test_label_keeps_static_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget LabelProbe {
    view {
        col {
            label {
                class: "slider-row"
                span { text "depth" }
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<label class=\"slider-row\">"),
            "label must keep its static class on the shadcn path (native <label>):\n{sfc}"
        );
    }

    #[test]
    fn test_label_keeps_dynamic_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget LabelDynProbe {
    model { var cls str = "x" }
    view {
        col {
            label (class: .cls) {
                text "depth"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(":class=\"cls\""),
            "label must bind a dynamic class expr as :class (Batch A gap 20):\n{sfc}"
        );
    }

    /// Sibling audit: `select` is schema-registered Form too and maps to the
    /// shadcn Select component — its arm had the same silent class drop.
    #[test]
    fn test_select_keeps_static_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget SelectProbe {
    view {
        col {
            select (class: "control-row") {
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("class=\"control-row\""),
            "select must forward its static class on the shadcn path:\n{sfc}"
        );
    }

    // --- Plan 012 Batch D (P0#11 leftover): class forwarding on the
    // remaining shadcn arms ---------------------------------------------
    // ~130 generate_shadcn_attrs arms never looked at `class:` at all — it
    // was dropped silently. A post-match choke point now forwards class/
    // style (static + dynamic) on every arm whose emitted element did not
    // already consume it (Vue attr fallthrough makes class work on
    // components automatically). Representative sample from different
    // families (overlay, sidebar, form, layout), each using a DSL tag that
    // genuinely dispatches through the widget registry into the arm (a tag
    // the registry doesn't know would take the plain-element path, where
    // class always worked — asserting the emitted component tag guards
    // against that false green).

    #[test]
    fn test_dialogtitle_forwards_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget DialogTitleProbe {
    view {
        col {
            dialogtitle {
                class: "text-lg"
                text "Title"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<DialogTitle") && sfc.contains("class=\"text-lg\""),
            "DialogTitle (overlay family) must forward its static class via attr fallthrough:\n{sfc}"
        );
    }

    #[test]
    fn test_sidebar_forwards_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget SidebarProbe {
    view {
        col {
            sidebar (class: "w-64 border-r") {
                text "nav"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Sidebar") && sfc.contains("class=\"w-64 border-r\""),
            "Sidebar (sidebar family) must forward its static class:\n{sfc}"
        );
    }

    /// Plan 548 T6: sidebar_menu_button to/active 臂 —— to 触发 as-child
    /// 多态(RouterLink 内嵌,button 套 a 是非法 HTML),active 映射资产的
    /// isActive prop(模板 kebab 形 is-active),variant/size 透传。
    #[test]
    fn test_sidebar_menu_button_to_active_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget SidebarMenuButtonProbe {
    model { var flag bool = true }
    view {
        col {
            sidebar {
                sidebar_menu {
                    sidebar_menu_item {
                        sidebar_menu_button (to: "/dash", active: .flag, variant: "outline", size: "lg") {
                            text "Dashboard"
                        }
                    }
                }
            }
        }
    }
}
"##,
        );
        assert!(
            sfc.contains("<SidebarMenuButton as-child"),
            "as-child 多态:\n{sfc}"
        );
        assert!(
            sfc.contains("<RouterLink to=\"/dash\">"),
            "内嵌 RouterLink:\n{sfc}"
        );
        assert!(
            sfc.contains(":is-active=\"flag\""),
            "active → :is-active:\n{sfc}"
        );
        assert!(sfc.contains("variant=\"outline\""), "variant 透传:\n{sfc}");
        assert!(sfc.contains("size=\"lg\""), "size 透传:\n{sfc}");
        assert!(sfc.contains("Dashboard"), "text slot 内容:\n{sfc}");
        assert!(
            sfc.contains("SidebarMenuButton") && sfc.contains("@/components/ui/sidebar"),
            "sidebar 契约 import:\n{sfc}"
        );
        assert!(sfc.contains("useRouter"), "to 置位 needs_router:\n{sfc}");
    }

    /// Plan 548 T6: to 存在且 active 缺省 → $route.path 自动探测(exact 或
    /// 前缀段匹配);sidebar_menu_sub_button 同款。
    #[test]
    fn test_sidebar_menu_button_route_autodetect_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget SidebarAutoActiveProbe {
    view {
        col {
            sidebar {
                sidebar_menu {
                    sidebar_menu_item {
                        sidebar_menu_button (to: "/settings") {
                            text "Settings"
                        }
                        sidebar_menu_sub {
                            sidebar_menu_sub_item {
                                sidebar_menu_sub_button (to: "/settings/net") {
                                    text "Network"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"##,
        );
        assert!(
            sfc.contains(":is-active=\"$route.path === '/settings' || $route.path.startsWith('/settings' + '/')\""),
            "静态 to 自动探测:\n{sfc}"
        );
        assert!(
            sfc.contains("<SidebarMenuSubButton as-child"),
            "sub_button as-child 多态:\n{sfc}"
        );
        assert!(
            sfc.contains(":is-active=\"$route.path === '/settings/net' || $route.path.startsWith('/settings/net' + '/')\""),
            "sub_button 自动探测:\n{sfc}"
        );
    }

    /// Plan 562 T6: Plain 模式（pac.at shadcn:off）sidebar_menu_button 落
    /// 原生 <button>（不坍缩 div），active/class/事件经通用属性透传。
    #[test]
    fn test_sidebar_menu_button_plain_mode_button_semantics() {
        let sfc = gen_sfc_from_widget_src(
            r##"
widget SidebarPlainProbe {
    model { var flag bool = true }
    view {
        col {
            sidebar_provider {
                sidebar_menu {
                    sidebar_menu_item {
                        sidebar_menu_button (active: .flag, size: "lg", class: "my-item") {
                            text "会话"
                        }
                    }
                }
            }
        }
    }
}
"##,
        );
        assert!(sfc.contains("<button"), "Plain 模式保 button 语义:\n{sfc}");
        assert!(
            !sfc.contains("SidebarMenuButton"),
            "Plain 模式不引 shadcn 组件:\n{sfc}"
        );
        assert!(sfc.contains("my-item"), "用户 class 透传:\n{sfc}");
        assert!(
            sfc.contains(":active="),
            "active 通用属性透传（测试锚）:\n{sfc}"
        );
    }

    /// Plan 548 T7: 含 sidebar 且无 sidebar_provider → 模板根自动包
    /// <SidebarProvider>(useSidebar() inject 缺祖先白屏,同 TooltipProvider
    /// 教训);显式写了 provider 的不重复包裹;非 shadcn 模式不动。
    #[test]
    fn test_sidebar_provider_autowrap_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r##"
widget SidebarAutowrapProbe {
    view {
        col {
            sidebar {
                sidebar_content { text "nav" }
            }
        }
    }
}
"##,
        );
        assert!(sfc.contains("<SidebarProvider>"), "模板根自动包裹:\n{sfc}");
        assert!(sfc.contains("</SidebarProvider>"), "包裹闭合:\n{sfc}");
        assert!(
            sfc2_check_import(&sfc, "SidebarProvider"),
            "SidebarProvider import 生成:\n{sfc}"
        );

        let explicit = gen_sfc_from_widget_src_shadcn(
            r##"
widget SidebarExplicitProbe {
    view {
        sidebar_provider {
            sidebar {
                sidebar_content { text "nav" }
            }
        }
    }
}
"##,
        );
        assert_eq!(
            explicit.matches("<SidebarProvider").count(),
            1,
            "显式 sidebar_provider 不重复包裹:\n{explicit}"
        );

        let plain = gen_sfc_from_widget_src(
            r##"
widget SidebarPlainProbe {
    view {
        col {
            sidebar { text "nav" }
        }
    }
}
"##,
        );
        assert!(
            !plain.contains("SidebarProvider"),
            "非 shadcn 模式不包裹:\n{plain}"
        );
    }

    /// T7 断言辅助:SFC 的 import 段含 sidebar 契约路径且点名组件。
    fn sfc2_check_import(sfc: &str, component: &str) -> bool {
        sfc.contains("@/components/ui/sidebar")
            && sfc
                .lines()
                .any(|l| l.contains("@/components/ui/sidebar") && l.contains(component))
    }

    #[test]
    fn test_slider_forwards_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget SliderProbe {
    model { var vol int = 0 }
    view {
        col {
            slider (class: "w-48", value: .vol)
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<input") && sfc.contains("type=\"range\"") && sfc.contains("w-48"),
            "slider (native range, PLAN-661 R-5) must forward its static class:\n{sfc}"
        );
    }

    /// PLAN-661 T-04（R-5）：slider 原生 range 全属性面——type/min/max/step
    /// 静态、value 响应式绑定、onchange → @input valueAsNumber 载荷派发
    /// （f32 语义对齐 a2r/VM 轨 .SetVol(v float)）。
    #[test]
    fn test_slider_native_range_attrs_and_payload() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget SliderProbe {
    msg { SetVol(float) }
    model { var vol float = 30.0 }
    view {
        col {
            slider (value: .vol, min: 0.0, max: 100.0, step: 1.0) { onchange: .SetVol }
        }
    }
    on {
        .SetVol(v float) -> { .vol = v }
    }
}
"#,
        );
        assert!(
            sfc.contains("type=\"range\""),
            "range type injected:\n{sfc}"
        );
        assert!(sfc.contains("min=\"0"), "min static:\n{sfc}");
        assert!(sfc.contains("max=\"100"), "max static:\n{sfc}");
        assert!(sfc.contains("step=\"1"), "step static:\n{sfc}");
        assert!(
            sfc.contains(":value=\"vol\""),
            "value reactive binding:\n{sfc}"
        );
        assert!(
            sfc.contains("@input=\"SetVol(($event.target as HTMLInputElement).valueAsNumber)\""),
            "onchange → @input valueAsNumber payload dispatch:\n{sfc}"
        );
        assert!(
            !sfc.contains("<Slider"),
            "shadcn Slider component path retired:\n{sfc}"
        );
        assert!(
            !sfc.contains("default-value"),
            "Plan 408 :default-value array form retired:\n{sfc}"
        );
    }

    #[test]
    fn test_card_forwards_dynamic_class_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget CardProbe {
    model { var cls str = "x" }
    view {
        col {
            card (class: .cls) {
                text "body"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("<Card") && sfc.contains(":class=\"cls\""),
            "Card (layout family) must bind a dynamic class expr as :class:\n{sfc}"
        );
    }

    /// No double emission: an arm that already consumed the class (button →
    /// shadcn Button via push_style_class) must not get a second class attr
    /// from the choke point.
    #[test]
    fn test_button_class_not_duplicated_shadcn() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget ButtonProbe {
    view {
        col {
            button (class: "px-8") {
                text "go"
            }
        }
    }
}
"#,
        );
        let occurrences = sfc.matches("class=\"px-8\"").count();
        assert_eq!(
            occurrences, 1,
            "button class must appear exactly once (no choke-point duplicate):\n{sfc}"
        );
    }

    // --- gap 44: computed referencing another computed -------------------

    #[test]
    fn test_gap44_computed_ref_unwrapped_in_computed() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ComputedProbe {
    model { var open bool = false }
    computed {
        is_expanded => .open
        show_body => if .is_expanded { "yes" } else { "no" }
    }
    view { col { text "x" } }
}
"#,
        );
        assert!(
            sfc.contains("is_expanded.value"),
            "computed referencing another computed must use .value:\n{sfc}"
        );
        // The bare-ref bug emitted `is_expanded ? ...` (always truthy).
        assert!(
            !sfc.contains("(is_expanded ?"),
            "bare computed ref would be always-truthy:\n{sfc}"
        );
    }

    #[test]
    fn test_gap44_explicit_value_suffix_not_doubled() {
        // The pre-fix workaround (writing `.c.value` by hand) must keep
        // compiling to `c.value`, not degrade to `c.value.value`.
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ComputedWorkaroundProbe {
    model { var open bool = false }
    computed {
        is_expanded => .open
        show_body => if .is_expanded.value { "yes" } else { "no" }
    }
    view { col { text "x" } }
}
"#,
        );
        assert!(
            sfc.contains("is_expanded.value"),
            "explicit .value workaround must still resolve:\n{sfc}"
        );
        assert!(
            !sfc.contains("is_expanded.value.value"),
            "explicit .value must not be doubled:\n{sfc}"
        );
    }

    // --- gap 45: expose {} with parameterized handlers --------------------

    #[test]
    fn test_gap45_exposed_parameterized_handler_generated() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget ExposeProbe {
    msg Msg { Open(str) }
    view { col { text "x" } }
    on {
        .Open(entry) -> { }
    }
    expose {
        .Open
    }
}
"#,
        );
        assert!(
            sfc.contains("function Open(entry"),
            "exposed parameterized handler must be generated as a local fn:\n{sfc}"
        );
        assert!(
            sfc.contains("defineExpose({ Open })"),
            "expose must reference the generated handler:\n{sfc}"
        );
    }

    // --- gap 19: .remove/.contains receiver gating ------------------------

    const GAP19_SRC: &str = r#"
widget RemoveProbe {
    use { composable: useRecentFilesStore from "src/front/composables/useRecentFilesStore.ts" }
    msg Msg { Del(int), FacadeDel(int), ExtDel(int), Check(int) }
    model {
        var items []int = []
        var name str = ""
        var has_x bool = false
    }
    view {
        col {
            button "del" { onclick: .Del(0) }
            button "fdel" { onclick: .FacadeDel(0) }
            button "xdel" { onclick: .ExtDel(0) }
            button "check" { onclick: .Check(0) }
        }
    }
    on {
        .Del(i) -> { .items.remove(i) }
        .FacadeDel(i) -> { store.facade.remove(i) }
        .ExtDel(i) -> { .recentFilesStore.remove(i) }
        .Check(i) -> { .has_x = .name.contains("x") }
    }
}
"#;

    const A1_STORE_QUAL_SRC: &str = r#"
use store: AuthStore

widget A1Probe {
    msg Msg { Boot }
    view {
        col {
            button "b" { onclick: .Boot }
        }
    }
    on {
        .Boot() -> { AuthStore.Init() }
    }
}
"#;

    /// plan446 批二 A1(多 store 消歧)配套:限定 store 调用 `X.Method()` 在
    /// vue 轨须发 facade(`store.Method()`),否则 TS2304(标识符不存在)。
    /// v1 单 store per file:接收者 ∈ store_deps 即改写;多 store 场景留待
    /// 命名 facade 上游项。
    #[test]
    fn test_a1_qualified_store_call_emits_facade() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser =
            auto_lang_core::parser::Parser::from(A1_STORE_QUAL_SRC).with_session(session);
        let ast = crate::parse_ui(A1_STORE_QUAL_SRC).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = VueGenerator::new().with_store_deps(vec!["AuthStore".to_string()]);
        let sfc = gen.generate(&widget).expect("generate SFC");
        assert!(
            sfc.contains("const store = reactive(useAuthStore())"),
            "use store decl must yield the facade const:
{sfc}"
        );
        assert!(
            sfc.contains("store.Init()"),
            "qualified AuthStore.Init() must emit the facade receiver:
{sfc}"
        );
        assert!(
            !sfc.contains("AuthStore.Init()"),
            "raw store name must not leak into TS:
{sfc}"
        );
    }

    /// PLAN-010: 多 store widget 裸 `store.Method()` 按归属路由——
    /// LoadFeed 唯一归属 ArticleStore（首 store 为 AuthStore）→ emit
    /// `articleStore.LoadFeed()`；Init 同理；Logout 首 store 自有 → 保持
    /// `store.Logout()`。缺省（无 surface）不路由，发射与旧版一致。
    #[test]
    fn test_plan010_bare_store_method_routing() {
        let src = r#"
use auth_store: AuthStore
use article_store: ArticleStore

widget App {
    msg { Init, Logout, GoDemo }
    view {
        col {
            link (to: "/demo") { text "Demo" }
        }
    }
    on {
        .Init -> {
            store.Init()
            store.LoadFeed()
        }
        .Logout -> {
            store.Logout()
        }
        .GoDemo -> { router.push("/demo") }
    }
}
"#;
        let ast = crate::parse_ui(src).expect("fixture must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");

        let mut surface = std::collections::HashMap::new();
        surface.insert(
            "AuthStore".to_string(),
            vec![
                "Login".to_string(),
                "Register".to_string(),
                "Logout".to_string(),
            ],
        );
        surface.insert(
            "ArticleStore".to_string(),
            vec![
                "Init".to_string(),
                "LoadFeed".to_string(),
                "SetToken".to_string(),
            ],
        );

        let mut gen = VueGenerator::new()
            .with_store_deps(vec!["AuthStore".to_string(), "ArticleStore".to_string()])
            .with_store_surface(surface);
        let sfc = gen.generate(&widget).expect("generate SFC");
        assert!(
            sfc.contains("articleStore.LoadFeed()"),
            "LoadFeed 唯一归属 ArticleStore 必须路由到 facade:
{sfc}"
        );
        assert!(
            sfc.contains("articleStore.Init()"),
            "Init 唯一归属 ArticleStore 必须路由到 facade:
{sfc}"
        );
        assert!(
            !sfc.contains("store.LoadFeed()"),
            "路由后不得残留 `store.LoadFeed()`:
{sfc}"
        );
        assert!(
            sfc.contains("store.Logout()"),
            "Logout 首 store 自有必须保持 `store.`:
{sfc}"
        );

        // 回归守卫：无 surface（旧调用方/单 store 路径）不路由
        let mut gen_plain = VueGenerator::new()
            .with_store_deps(vec!["AuthStore".to_string(), "ArticleStore".to_string()]);
        let sfc_plain = gen_plain
            .generate(&widget)
            .expect("generate SFC (no surface)");
        assert!(
            sfc_plain.contains("store.LoadFeed()"),
            "无 surface 时保持现状 `store.LoadFeed()`:
{sfc_plain}"
        );
        assert!(
            !sfc_plain.contains("articleStore.LoadFeed()"),
            "无 surface 时不得出现路由:
{sfc_plain}"
        );
    }

    /// Plan 446 批三 G1: store 导入前缀可配。默认 `@/stores` 不变；配
    /// `@/stores/auto` 时 SFC import 与迁址后的生成物对齐（os-config
    /// regen 布局，此前恒 @/stores → vue-tsc TS2307）。
    #[test]
    fn test_g1_store_import_prefix_configurable() {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser =
            auto_lang_core::parser::Parser::from(A1_STORE_QUAL_SRC).with_session(session);
        let ast = crate::parse_ui(A1_STORE_QUAL_SRC).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");

        // 默认前缀不变（回归守卫）
        let mut gen_default = VueGenerator::new().with_store_deps(vec!["AuthStore".to_string()]);
        let sfc_default = gen_default
            .generate(&widget)
            .expect("generate SFC (default)");
        assert!(
            sfc_default.contains("from '@/stores/useAuthStore'"),
            "default prefix must stay @/stores:\n{sfc_default}"
        );

        // 可配前缀落在 import 行
        let mut gen_cfg = VueGenerator::new()
            .with_store_deps(vec!["AuthStore".to_string()])
            .with_store_import_prefix("@/stores/auto");
        let sfc_cfg = gen_cfg
            .generate(&widget)
            .expect("generate SFC (configured)");
        assert!(
            sfc_cfg.contains("from '@/stores/auto/useAuthStore'"),
            "configured prefix must reach the import line:\n{sfc_cfg}"
        );
        assert!(
            !sfc_cfg.contains("from '@/stores/useAuthStore'"),
            "stale default-prefix import must not remain:\n{sfc_cfg}"
        );
    }

    #[test]
    fn test_gap19_typed_array_remove_keeps_splice() {
        let (sfc, warnings) = gen_sfc_and_warnings(GAP19_SRC);
        assert!(
            sfc.contains("items.value.splice(i, 1)"),
            ".remove on a proven []int state must keep the splice mapping:\n{sfc}"
        );
        assert!(
            warnings_for_rule(&warnings, "R010")
                .iter()
                .all(|w| !w.message.contains("`items`")),
            "no passthrough note for a proven array receiver: {warnings:?}"
        );
    }

    #[test]
    fn test_gap19_store_facade_remove_passes_through() {
        let (sfc, warnings) = gen_sfc_and_warnings(GAP19_SRC);
        assert!(
            sfc.contains("store.facade.remove(i)"),
            "store.facade.remove must pass through unchanged:\n{sfc}"
        );
        assert!(
            !sfc.contains("store.facade.splice"),
            "facade receiver must not get the splice mapping:\n{sfc}"
        );
        assert!(
            warnings_for_rule(&warnings, "R010")
                .iter()
                .any(|w| w.message.contains("store.facade")),
            "facade passthrough must be noted (R010): {warnings:?}"
        );
    }

    #[test]
    fn test_gap19_ext_composable_facade_remove_passes_through() {
        let (sfc, warnings) = gen_sfc_and_warnings(GAP19_SRC);
        assert!(
            sfc.contains("recentFilesStore.remove(i)"),
            "ext-composable facade .remove must pass through unchanged:\n{sfc}"
        );
        assert!(
            !sfc.contains("recentFilesStore.splice"),
            "ext-composable facade must not get the splice mapping:\n{sfc}"
        );
        assert!(
            warnings_for_rule(&warnings, "R010")
                .iter()
                .any(|w| w.message.contains("recentFilesStore")),
            "facade passthrough must be noted (R010): {warnings:?}"
        );
    }

    #[test]
    fn test_gap19_typed_string_contains_maps_to_includes() {
        let (sfc, warnings) = gen_sfc_and_warnings(GAP19_SRC);
        assert!(
            sfc.contains("name.value.includes('x')"),
            ".contains on a proven str state must map to .includes:\n{sfc}"
        );
        assert!(
            warnings_for_rule(&warnings, "R010")
                .iter()
                .all(|w| !w.message.contains("`name`")),
            "no passthrough note for a proven string receiver: {warnings:?}"
        );
    }

    // --- Plan 053 M1: 字符串方法映射补全 --------------------------------
    // 原先仅 len/contains/to_string/to_int/to_float 有映射，其余 .at 字符串
    // 方法走兜底 `.{method}()` 原样输出，生成无效 JS（HistorySearch 产物的
    // `.to_lower()` 是实测症状）。下面覆盖两条 codegen 路径。

    #[test]
    fn test_plan053_string_mapping_handler_body() {
        // handler body → ts_adapter::transpile_handler_body 路径
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053Str {
    model {
        var s str = "Hello"
        var sink str = ""
    }
    on {
        .Run -> {
            .sink = .s.to_lower()
            .sink = .s.lower()
            .sink = .s.to_upper()
            .sink = .s.starts_with("He")
            .sink = .s.ends_with("lo")
            .sink = .s.char_at(0)
            .sink = .s.find("ll")
            .sink = .s.substr(1, 3)
            .sink = .s.slice(0, 2)
            .sink = .s.replace("l", "L")
            .sink = .s.repeat(2)
            .sink = .s.trim_left()
            .sink = .s.trim_right()
        }
    }
    view { col { button "run" { onclick: .Run } text .sink } }
}
"#,
        );
        assert!(
            sfc.contains(".toLowerCase()"),
            "to_lower/lower→toLowerCase:\n{sfc}"
        );
        assert!(
            sfc.contains(".toUpperCase()"),
            "to_upper→toUpperCase:\n{sfc}"
        );
        assert!(
            sfc.contains(".startsWith("),
            "starts_with→startsWith:\n{sfc}"
        );
        assert!(sfc.contains(".endsWith("), "ends_with→endsWith:\n{sfc}");
        assert!(sfc.contains(".charAt("), "char_at→charAt:\n{sfc}");
        assert!(sfc.contains(".indexOf("), "find→indexOf:\n{sfc}");
        assert!(
            sfc.contains(".substring("),
            "substr/sub/slice→substring:\n{sfc}"
        );
        assert!(sfc.contains(".replaceAll("), "replace→replaceAll:\n{sfc}");
        assert!(sfc.contains(".repeat("), "repeat→repeat:\n{sfc}");
        assert!(sfc.contains(".trimStart()"), "trim_left→trimStart:\n{sfc}");
        assert!(sfc.contains(".trimEnd()"), "trim_right→trimEnd:\n{sfc}");
        // 关键回归：坏 JS（原样输出）不得再出现
        assert!(
            !sfc.contains(".to_lower("),
            "to_lower must map, not pass through:\n{sfc}"
        );
        assert!(!sfc.contains(".lower("), "lower must map:\n{sfc}");
        assert!(
            !sfc.contains(".starts_with("),
            "starts_with must map:\n{sfc}"
        );
        assert!(!sfc.contains(".char_at("), "char_at must map:\n{sfc}");
        assert!(!sfc.contains(".substr("), "substr must map:\n{sfc}");
    }

    #[test]
    fn test_plan053_string_mapping_computed_expr() {
        // computed 单表达式 → vue.rs expr_to_js 路径（与 handler body 的
        // ts_adapter 路径相对，覆盖第一处 codegen 改动）
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053Tmpl {
    model { var s str = "hi" }
    computed {
        upper => .s.to_upper()
        low => .s.lower()
        sw => .s.starts_with("h")
        ch => .s.char_at(0)
        ie => .s.is_empty()
    }
    view { col { text .upper } }
}
"#,
        );
        assert!(sfc.contains(".toUpperCase()"), "computed to_upper:\n{sfc}");
        assert!(sfc.contains(".toLowerCase()"), "computed lower:\n{sfc}");
        assert!(sfc.contains(".startsWith("), "computed starts_with:\n{sfc}");
        assert!(sfc.contains(".charAt("), "computed char_at:\n{sfc}");
        assert!(sfc.contains(".length === 0"), "computed is_empty:\n{sfc}");
        assert!(
            !sfc.contains(".to_upper("),
            "computed to_upper must map:\n{sfc}"
        );
        assert!(
            !sfc.contains(".starts_with("),
            "computed starts_with must map:\n{sfc}"
        );
    }

    #[test]
    fn test_plan053_p53_view_expr_method_mapping() {
        // P5-3: view fn / 模板表达式（属性绑定 + 文本插值）此前不走 method
        // map，字符串方法原样输出坏 JS（实测 BlockBody 产物
        // `field[1].Text.to_float()` 触发 vue-tsc 报错）。修复后与
        // expr_to_js / handler body 共享 map_method_to_js 单源。
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053ViewMap {
    model { var s str = "75%" }
    view {
        col {
            progress {
                value: .s.to_float()
                max: 100
            }
            if .s.starts_with("7") {
                text "yes"
            }
        }
    }
}
"#,
        );
        // progress value → bound 位置：to_float → parseFloat
        assert!(
            sfc.contains("parseFloat(s)"),
            "bound to_float → parseFloat:\n{sfc}"
        );
        assert!(
            !sfc.contains(".to_float("),
            "bound to_float must map:\n{sfc}"
        );
        // view if 条件（同样走 bound 位置）：starts_with → startsWith
        assert!(
            sfc.contains(".startsWith("),
            "bound starts_with → startsWith:\n{sfc}"
        );
        assert!(
            !sfc.contains(".starts_with("),
            "bound starts_with must map:\n{sfc}"
        );
    }

    // --- find 双语义守卫：数组 find(λ) 不得映射 indexOf ------------------
    // `find → indexOf` 是字符串方法映射（Rust str::find 返回 index）。
    // 当第一个实参是 lambda/闭包时，receiver 必为数组（Array.find 谓词
    // 语义，JS 同名），映射成 indexOf 会让函数对象去比较引用、永远返回
    // -1。实测症状：jade-garden tabs_store.at 的
    // `.tabs.find(t => t.path == path)` 被编成
    // `tabs.value.indexOf((t) => t.path == path)`。

    #[test]
    fn test_find_lambda_handler_body_keeps_find() {
        // handler body → ts_adapter::transpile_expr 路径
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget FindLambdaHandler {
    model {
        var tabs Array<str> = []
        var found str = ""
    }
    on {
        .Go -> {
            .found = .tabs.find(t => t.path == "x")
        }
    }
    view { col { button "go" { onclick: .Go } text .found } }
}
"#,
        );
        assert!(
            sfc.contains(".find("),
            "array find(λ) in handler body must keep .find():\n{sfc}"
        );
        assert!(
            !sfc.contains(".indexOf("),
            "array find(λ) must NOT map to .indexOf():\n{sfc}"
        );
    }

    #[test]
    fn test_find_lambda_computed_keeps_find() {
        // computed 单表达式 → vue.rs expr_to_js / map_method_to_js 路径
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget FindLambdaComputed {
    model { var tabs Array<str> = [] }
    computed {
        hit => .tabs.find(t => t.path == "x")
    }
    view { col { text .hit } }
}
"#,
        );
        assert!(
            sfc.contains(".find(("),
            "array find(λ) in computed must keep .find():\n{sfc}"
        );
        assert!(
            !sfc.contains(".indexOf("),
            "array find(λ) must NOT map to .indexOf():\n{sfc}"
        );
    }

    #[test]
    fn test_find_lambda_store_composable_keeps_find() {
        // store codegen 复现 jade-garden tabs_store.at 的实际形态：
        // computed 与 action 体里都是 `.tabs.find(t => t.path == ...)`。
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store FindTabs {
    model {
        var tabs Array<str> = []
        var active_path ?str = None
    }
    msg Msg { Open(map) }
    computed {
        active_tab => .tabs.find(t => t.path == .active_path)
    }
    on {
        .Open(args) -> {
            var path = args.path
            var existing = .tabs.find(t => t.path == path)
            if existing != null {
                .active_path = path
            }
        }
    }
}
"#,
        ));
        assert!(
            code.contains(".find("),
            "store array find(λ) must keep .find():\n{code}"
        );
        assert!(
            !code.contains(".indexOf("),
            "store array find(λ) must NOT map to .indexOf():\n{code}"
        );
    }

    #[test]
    fn test_find_string_arg_still_maps_to_index_of() {
        // 回归保护：字符串 str::find 语义（非 lambda 实参）仍映射 indexOf。
        // handler body 与 view 条件两条路径都查。
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget FindStrRegress {
    model {
        var s str = "Hello"
        var sink str = ""
    }
    on {
        .Go -> {
            .sink = .s.find("ll")
        }
    }
    view {
        col {
            button "go" { onclick: .Go }
            if .s.find("H") {
                text "has H"
            }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains(".indexOf("),
            "string find must still map to .indexOf():\n{sfc}"
        );
        assert!(
            !sfc.contains(".find("),
            "string find must not pass through as .find():\n{sfc}"
        );
    }

    #[test]
    fn test_find_guard_in_condition_string() {
        // view 条件的 convert_condition 是字符串替换路径。DSL 的 view if
        // 条件不支持 lambda（parser 直接拒绝 `=>`），所以该守卫无法走
        // parse 路径端到端复现，这里直接单测映射函数：lambda 实参保留
        // .find（含原空格形态），字符串实参仍映射 indexOf，混合场景逐处判断。
        assert_eq!(
            VueGenerator::map_find_in_condition("tabs.find ( t => t.path == 'x' )"),
            "tabs.find ( t => t.path == 'x' )"
        );
        assert_eq!(
            VueGenerator::map_find_in_condition("tabs.find(t => t.path == 'x')"),
            "tabs.find(t => t.path == 'x')"
        );
        assert_eq!(
            VueGenerator::map_find_in_condition("s.find ( 'H' )"),
            "s.indexOf( 'H' )"
        );
        assert_eq!(
            VueGenerator::map_find_in_condition("s.find('H')"),
            "s.indexOf('H')"
        );
        // 混合：lambda 的保留，字符串的映射
        assert_eq!(
            VueGenerator::map_find_in_condition("tabs.find(t => t.x) && s.find('y')"),
            "tabs.find(t => t.x) && s.indexOf('y')"
        );
    }

    // --- Plan 053 M3 / P5-7: handler 间自调用识别 ------------------------

    #[test]
    fn test_plan053_p57_self_called_handler_is_emitted() {
        // P5-7: handler 被 OnInput/AcceptGhost/Word 自调用，无模板绑定，不 expose。
        // mark_self_called_handlers 应把它标 used → 生成 function（否则 TS2304）。
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053SelfCall {
    model { var x str = "" }
    on {
        .Run -> {
            .Helper()
        }
        .Helper -> {
            .x = "done"
        }
    }
    view { col { button "go" { onclick: .Run } text .x } }
}
"#,
        );
        assert!(
            sfc.contains("function Helper"),
            "Helper (called via .Helper() from Run, no template binding, no expose) must be emitted:\n{sfc}"
        );
        assert!(
            sfc.contains("Helper()"),
            "Run body must call Helper():\n{sfc}"
        );
    }

    #[test]
    fn test_plan053_p57_self_call_does_not_promote_non_handler() {
        // P5-7 gate: .save()/.push() share the self-call shape but are NOT
        // handlers — must NOT be marked used (no false "function save" etc).
        // (They emit as method calls regardless; this test just ensures the
        // scanner doesn't crash on them or pollute used_handlers.)
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053Gate {
    model { var items []str = [] }
    on {
        .Run -> {
            .items.push("x")
        }
    }
    view { col { button "go" { onclick: .Run } } }
}
"#,
        );
        assert!(
            !sfc.contains("function push"),
            ".push() is a store method, must NOT be emitted as a handler function:\n{sfc}"
        );
    }

    // --- Plan 053 M5 / P5-6: complete-handler debounce injection -----------

    #[test]
    fn test_plan053_p56_complete_handler_is_debounced() {
        // P5-6: a handler whose body calls `complete(...)` gets wrapped in
        // setTimeout(80ms) with a sequence counter; the widget gains top-level
        // `let __completeTimer` / `let __completeSeq` declarations.
        // (complete's `await`/import come from `use back.api` in real code;
        // here we only exercise the debounce wrapping, which keys off the
        // bare call name, so the use-decl is unnecessary in the fixture.)
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053Debounce {
    model {
        var input str = ""
        var suggestions []str = []
    }
    on {
        .OnInputComplete -> {
            if .input.trim() != "" {
                var items []str = complete(.input, .input.len())
                .suggestions = items
            } else {
                .suggestions = []
            }
        }
    }
    view {
        col {
            textarea {
                value: .input
                oninput: .OnInputComplete
            }
        }
    }
}
"#,
        );
        assert!(sfc.contains("let __completeTimer"), "timer decl:\n{sfc}");
        assert!(sfc.contains("let __completeSeq = 0"), "seq decl:\n{sfc}");
        assert!(
            sfc.contains("clearTimeout(__completeTimer)"),
            "clear pending timer:\n{sfc}"
        );
        assert!(sfc.contains("++__completeSeq"), "bump seq:\n{sfc}");
        assert!(
            sfc.contains("setTimeout(async () => {"),
            "setTimeout wrapper:\n{sfc}"
        );
        assert!(sfc.contains("}, 80)"), "80ms delay:\n{sfc}");
    }

    #[test]
    fn test_plan053_p56_debounce_guard_on_state_assign() {
        // P5-6: inside the debounced body, state-ref assignments
        // (`.suggestions =`) get a seq guard so a stale complete result can't
        // clobber the newest suggestions. Local `var items` is NOT guarded.
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053DebounceGuard {
    model {
        var input str = ""
        var suggestions []str = []
    }
    on {
        .OnInputComplete -> {
            if .input.trim() != "" {
                var items []str = complete(.input, .input.len())
                .suggestions = items
            } else {
                .suggestions = []
            }
        }
    }
    view {
        col {
            textarea {
                value: .input
                oninput: .OnInputComplete
            }
        }
    }
}
"#,
        );
        // Both branches' `.suggestions =` are guarded; `let items` is not.
        assert!(
            sfc.contains("if (__seq === __completeSeq) { suggestions.value ="),
            "state-ref assignment must be seq-guarded:\n{sfc}"
        );
        assert!(
            sfc.contains("if (__seq === __completeSeq) { suggestions.value = []"),
            "empty-list branch assignment must also be guarded:\n{sfc}"
        );
    }

    #[test]
    fn test_plan053_p56_non_complete_handler_not_debounced() {
        // P5-6 gate: handlers that DON'T call complete stay eager — no timer,
        // no setTimeout wrapper, no seq infrastructure emitted at all.
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053NoDebounce {
    model {
        var input str = ""
        var ghost str = ""
    }
    on {
        .OnInput -> {
            .ghost = ""
            if .input != "" {
                .ghost = "x"
            }
        }
    }
    view {
        col {
            textarea {
                value: .input
                onkeyup: .OnInput
            }
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("__completeTimer"),
            "no timer for non-complete handler:\n{sfc}"
        );
        assert!(!sfc.contains("setTimeout"), "no setTimeout wrapper:\n{sfc}");
        assert!(
            !sfc.contains("__completeSeq"),
            "no seq for non-complete handler:\n{sfc}"
        );
        assert!(
            sfc.contains("function OnInput"),
            "handler still emitted:\n{sfc}"
        );
    }

    #[test]
    fn test_plan053_p55_textarea_skips_default_classes() {
        // P5-5: textarea was missing from user_class_skip_elements → user
        // style still got the default `border rounded px-2 py-1` forced in.
        // With user class present, defaults must be skipped (mirrors `input`).
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053Textarea {
    model { var s str = "" }
    view {
        col {
            textarea {
                value: .s
                rows: 1
                style: "w-full resize-none"
            }
        }
    }
}
"#,
        );
        assert!(sfc.contains("<textarea"), "textarea tag:\n{sfc}");
        assert!(sfc.contains("resize-none"), "user class kept:\n{sfc}");
        assert!(
            !sfc.contains("border rounded"),
            "textarea must skip default border classes when user class present:\n{sfc}"
        );
        assert!(sfc.contains(":rows=\"1\""), "rows attr:\n{sfc}");
    }

    #[test]
    fn test_plan053_p57_residual_self_called_handler_no_emit() {
        // P5-7 residual (Plan 053 M4 companion): self-called-only handlers
        // must NOT get a trailing emit('X') — they aren't in the defineEmits
        // type (built from template-bound handlers before
        // mark_self_called_handlers), so emitting would be TS2769
        // (observed on DoTokenize).
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget Plan053NoEmit {
    msg Msg { Run, Helper }
    model { var x str = "" }
    on {
        .Run -> {
            .Helper()
        }
        .Helper -> {
            .x = "done"
        }
    }
    view { col { button "go" { onclick: .Run } text .x } }
}
"#,
        );
        assert!(
            sfc.contains("function Helper"),
            "self-called handler emitted:\n{sfc}"
        );
        assert!(
            !sfc.contains("emit('Helper')"),
            "self-called-only handler must not emit (not in defineEmits type):\n{sfc}"
        );
        assert!(
            sfc.contains("emit('Run')"),
            "template-bound handler keeps its emit:\n{sfc}"
        );
    }

    // --- gap 47: != null / == null semantics ------------------------------

    #[test]
    fn test_gap47_null_checks_are_loose() {
        let (sfc, _) = gen_sfc_and_warnings(
            r#"
widget NullProbe {
    model { var x str = "" }
    computed {
        has_x => .x != null
        no_x => .x == null
    }
    view { col { text "x" } }
}
"#,
        );
        assert!(
            sfc.contains("x.value != null"),
            "`.x != null` must compile to a loose null check (covers undefined):\n{sfc}"
        );
        assert!(
            sfc.contains("x.value == null"),
            "`.x == null` must compile to a loose null check:\n{sfc}"
        );
        assert!(
            !sfc.contains("!== undefined") && !sfc.contains("!== null"),
            "strict undefined/null checks would change semantics:\n{sfc}"
        );
    }

    /// Plan 465 T4 (I4): desktop virtual_window/taskbar leaves resolve to the
    /// @/wm/* components registered in docs/schema/aura.at (`vue:` mapping).
    #[test]
    fn test_a2vue_virtual_window() {
        test_a2vue_shadcn("virtual_window").expect("a2vue virtual_window golden mismatch");
    }

    /// Plan 497 T7 (I4): window_thumbnail leaf resolves to @/wm/WindowThumbnail
    /// (docs/schema/aura.at `vue:` mapping) with wid/fallback_icon passthrough.
    /// Vue side is the v1 placeholder component (icon + border) — real web
    /// thumbnails are a follow-up enhancement (plan 497 待澄清①).
    #[test]
    fn test_a2vue_window_thumbnail() {
        test_a2vue_shadcn("window_thumbnail").expect("a2vue window_thumbnail golden mismatch");
    }

    /// PLAN-760 T-04: remote-frame leaf golden — div passthrough with
    /// data-remote-frame marker + url via data-remote-frame-url + class on
    /// the leaf (host frontend adopts; this repo ships no JS renderer).
    #[test]
    fn test_a2vue_remote_frame() {
        test_a2vue_shadcn("remote_frame").expect("a2vue remote-frame golden mismatch");
    }

    #[test]
    fn test_a2vue_counter() {
        test_a2vue("001_counter").expect("a2vue counter golden mismatch");
        // Plan 500 T4：001-helloworld 真源 → vue 臂金样（三臂对拍基线的
        // vue 臂；queue 臂 = desktop_protocol parity_001_queue_golden，
        // iced 像素臂留实机档——497 headless 栅格化不可行结论）。
        test_a2vue_shadcn("p500_001_helloworld")
            .expect("a2vue p500 001 helloworld golden mismatch");
    }

    /// PLAN-026 缺陷③: `!= ""` / `== ""` in computed (script ctx) must emit
    /// truthy/falsy (`!!x` / `!x`), not strict `!== ''` — the latter misclassifies
    /// a missing field (undefined) as non-empty (`undefined !== ''` is true).
    #[test]
    fn test_a2vue_neq_empty_safe() {
        test_a2vue("008_neq_empty_safe").expect("a2vue neq-empty-safe golden mismatch");
    }

    /// PLAN-026 缺陷①: col（shadcn 布局基元）上的动态 class 绑定。验证
    /// `col { class: .rowClass }` 生成 `:class="rowClass"`（而非被丢弃）。
    #[test]
    fn test_a2vue_shadcn_col_dynamic_class() {
        test_a2vue_shadcn("009_shadcn_col_dynamic_class")
            .expect("a2vue shadcn col dynamic class mismatch");
    }

    /// Plan 481 a2vue golden: text/label `selectable: true` 往返 —— 输出携带
    /// 显式 `style="user-select: text"`;缺省零输出(金样锁 prop 透传,不锁
    /// 选区行为——vue 端跟随浏览器原生)。
    #[test]
    fn test_a2vue_text_selectable() {
        test_a2vue("011_text_selectable").expect("a2vue 011_text_selectable mismatch");
    }

    /// Plan 496 M5 I8：desktop.at 双端同源对拍基线——真资产
    /// （`assets/desktop.at`，include_str 同文件零拷贝——`ui::shell::DESKTOP_AT`
    /// 在 ui 特性后，本测试留默认档）经 a2vue 生成 SFC，金样
    /// `test/a2vue/desktop_surface_asset/expected.vue` 对拍。重生成：
    /// `AUTO_LANG_UPDATE_GOLDEN=1 cargo test test_a2vue_desktop_surface_asset`。
    #[test]
    fn test_a2vue_desktop_surface_asset() {
        // PLAN-814 T-03：desktop.at 属 shell 资产家族（runtime 消费面），整族归
        // PLAN-814 T-04：资产 owner=auto-ui-runtime（assets/ 随 ui/ 迁）；经 MANIFEST_DIR
        // 相对路径引同仓单源副本（工作区兄弟布局），不复制。
        const DESKTOP_SURFACE_AT: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../auto-ui-runtime/assets/desktop.at"
        ));
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser =
            auto_lang_core::parser::Parser::from(DESKTOP_SURFACE_AT).with_session(session);
        let ast = crate::parse_ui(DESKTOP_SURFACE_AT).expect("desktop.at 解析");
        let mut widgets = Vec::new();
        for stmt in ast.domain_stmts() {
            if let auto_ui_dialect::UiStmt::WidgetDecl(decl) = stmt {
                widgets.push(
                    crate::aura::extract_widget_from_decl(decl).expect("desktop.at aura 提取"),
                );
            }
        }
        assert_eq!(widgets.len(), 1, "desktop.at 单 widget");
        let mut gen = VueGenerator::new_shadcn();
        let output = gen.generate_sfc(&widgets[0]).expect("desktop.at → vue SFC");
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let exp_path = d.join("test/a2vue/desktop_surface_asset/expected.vue");
        if std::env::var("AUTO_LANG_UPDATE_GOLDEN").is_ok() {
            std::fs::create_dir_all(exp_path.parent().unwrap()).unwrap();
            std::fs::write(&exp_path, &output).expect("金样重生成写盘");
            return;
        }
        let expected = std::fs::read_to_string(&exp_path)
            .unwrap_or_else(|e| panic!("金样缺失（先 AUTO_LANG_UPDATE_GOLDEN=1 重生成）: {e}"));
        assert_eq!(
            normalize_vue_output(&output),
            normalize_vue_output(&expected),
            "desktop.at a2vue 金样对拍（改动 vue 生成器/资产后须同步金样）"
        );
        // 关键接缝标记：双击/右键/布局件点击三事件面在 vue 产物可见。
        assert!(output.contains("@dblclick"), "ondblclick → @dblclick");
        assert!(
            output.contains("@contextmenu"),
            "oncontextmenu → @contextmenu"
        );
    }

    /// Plan 498 M0：mouse-area onclick → vue `@click` 生成断言（与 496
    /// `@dblclick` 同族；chart legend 点击切换显隐的 vue 通路——事件名经
    /// 通用 base_event_to_dom 映射，mouse-area 本体仍走 div 直通）。
    /// Plan 547 Task 23: ImageSurface is emitted as a URI-bound crop
    /// container with an interactive transform and normalized DOM events.
    #[test]
    fn image_surface_vue_codegen() {
        let sfc = gen_sfc_from_widget_src_shadcn(
            r#"
widget ImageViewer {
    model {
        var asset_src str = "/api/__auto/media/demo/1"
        var fit_mode str = "contain"
        var zoom float = 1.0
        var offset_x float = 0.0
        var offset_y float = 0.0
        var rotation int = 0
    }
    view {
        image_surface (
            src: .asset_src,
            fit: .fit_mode,
            zoom: .zoom,
            offset_x: .offset_x,
            offset_y: .offset_y,
            rotation: .rotation,
            filter: "none",
            alt: "hero",
            onload: .ImageLoaded,
            onerror: .ImageFailed,
            onwheel: .ZoomAt,
            onpan: .PanBy,
            ondblclick: .ToggleFit
        ) {}
    }
    on {
        .ImageLoaded -> { }
        .ImageFailed -> { }
        .ZoomAt -> { }
        .PanBy -> { }
        .ToggleFit -> { }
    }
}
"#,
        );
        assert!(
            sfc.contains("class=\"relative overflow-hidden\""),
            "crop wrapper:\n{sfc}"
        );
        assert!(
            sfc.contains(":src=\"asset_src\""),
            "URI source binding:\n{sfc}"
        );
        assert!(
            sfc.contains(":style=\"'transform:' +"),
            "interactive transform (string style):\n{sfc}"
        );
        assert!(
            sfc.contains("object-fit:' + (fit_mode)"),
            "fit binding (string style):\n{sfc}"
        );
        assert!(
            sfc.contains("@load=\"ImageLoaded\""),
            "onload normalization:\n{sfc}"
        );
        assert!(
            sfc.contains("@error=\"ImageFailed\""),
            "onerror normalization:\n{sfc}"
        );
        assert!(
            sfc.contains("@wheel=\"ZoomAt\""),
            "onwheel normalization:\n{sfc}"
        );
        // Plan 547: onpan -> pointer 三绑定 + script 合成包装(DOM 无 pan 事件)
        assert!(
            sfc.contains("@pointerdown=\"__surfPanD_0\""),
            "onpan pointer down:\n{sfc}"
        );
        assert!(
            sfc.contains("@pointermove=\"__surfPanM_0\""),
            "onpan pointer move:\n{sfc}"
        );
        assert!(
            sfc.contains("@pointerup=\"__surfPanU_0\""),
            "onpan pointer up:\n{sfc}"
        );
        assert!(
            sfc.contains("function __surfPanM_0(e: PointerEvent)"),
            "pan wrapper fn:\n{sfc}"
        );
        assert!(sfc.contains("e.buttons & 1"), "pan buttons gate:\n{sfc}");
        assert!(
            sfc.contains("PanBy()"),
            "pan handler arity-0 call:
{sfc}"
        );
        assert!(
            sfc.contains("@dblclick=\"ToggleFit\""),
            "ondblclick normalization:\n{sfc}"
        );
        for forbidden in [
            "FileReader",
            "decodeImage",
            "prefetch",
            "imageCache",
            "cache.put",
        ] {
            assert!(
                !sfc.to_ascii_lowercase()
                    .contains(&forbidden.to_ascii_lowercase()),
                "forbidden browser-side media implementation {forbidden}:\n{sfc}"
            );
        }
    }

    #[test]
    fn test_a2vue_mouse_area_onclick() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ClickZone {
    msg { Ping }
    model { var n int = 0 }
    view {
        col {
            mouse-area (style: "w-4 h-4", onmouseenter: .Ping, onmouseleave: .Ping, onclick: .Ping) {}
            text "zone" {}
        }
    }
    on {
        .Ping -> { .n = .n + 1 }
    }
}
"#,
        );
        assert!(
            sfc.contains("@click=\"Ping\""),
            "onclick → @click:\n{}",
            sfc
        );
        assert!(
            sfc.contains("@mouseenter=\"Ping\""),
            "onmouseenter 共存:\n{}",
            sfc
        );
    }

    /// PLAN-026 缺陷②: component fn 的 `style { }` 块必须 emit 到 SFC `<style
    /// scoped>`（之前 extract_widget_from_fragment 硬编码 style_css: None 丢弃）。
    #[test]
    fn test_a2vue_component_fn_style() {
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let at_path = d.join("test/a2vue/010_component_fn_style/input.at");
        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .expect("010_component_fn_style generation failed");

        let code = result
            .all_widget_codes
            .iter()
            .find(|(name, _)| name == "StyledCard")
            .map(|(_, code)| code)
            .expect("StyledCard component fn not synthesized");
        let golden = d.join("test/a2vue/010_component_fn_style/StyledCard.expected.vue");
        let expected = normalize_vue_output(&std::fs::read_to_string(&golden).unwrap_or_default());
        let actual = normalize_vue_output(code);
        if actual != expected {
            let wrong = d.join("test/a2vue/010_component_fn_style/StyledCard.wrong.vue");
            std::fs::write(&wrong, code).unwrap();
            panic!(
                "010_component_fn_style mismatch. See StyledCard.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                expected, actual
            );
        }
        assert!(
            code.contains("<style scoped>"),
            "component fn SFC must have <style scoped>: {}",
            code
        );
        assert!(
            code.contains(".card-title"),
            "<style> must contain the CSS rule: {}",
            code
        );
    }

    /// Plan 041a⑤(musk 041 Phase 5 T22): 表单 value 绑定目标是 prop 时
    /// 不折 v-model(只读绑定编译红),保持 :value 单向;model 变量目标
    /// 照旧双折(带/不带 oninput)。
    #[test]
    fn t22_value_on_prop_stays_one_way() {
        let src = "widget Editor(content: str) {
  msg Msg { Input }
  model { var draft str = \"\" }
  view {
    input (value: .content)
    textarea {
      value: .content
      oninput: .Input
    }
    input {
      value: .draft
      oninput: .Input
    }
  }
}
";
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("widget");
        let mut gen = VueGenerator::new_shadcn();
        let out = gen.generate_sfc(&widget).unwrap();
        let vmodels: Vec<&str> = out.lines().filter(|l| l.contains("v-model")).collect();
        assert_eq!(
            vmodels.len(),
            1,
            "only the model-var input folds; got {:?}
{}",
            vmodels,
            out
        );
        assert!(
            vmodels[0].contains("draft"),
            "fold targets the state var: {}",
            vmodels[0]
        );
        assert!(
            out.contains(":modelValue=\"content\""),
            "prop-bound inputs stay one-way (:modelValue):
{}",
            out
        );
        assert!(
            !out.contains("v-model=\"content\""),
            "prop target must never fold to v-model:
{}",
            out
        );
    }

    /// Plan 022 Phase 7c: markdown content prop binding (fixes §10).
    /// Uses shadcn mode (like real `auto build`) so `markdown` maps to
    /// <MarkdownRender> via the widget registry + generate_shadcn_attrs.
    #[test]
    fn test_a2vue_markdown() {
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = d.join("test/a2vue/002_markdown/input.at");
        let src = std::fs::read_to_string(&src_path).unwrap();
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src.as_str()).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("no widget in 002_markdown input");
        let mut gen = VueGenerator::new_shadcn();
        let output = gen.generate_sfc(&widget).unwrap();
        let exp_path = d.join("test/a2vue/002_markdown/input.expected.vue");
        let expected = std::fs::read_to_string(&exp_path).unwrap_or_default();
        let output_n = normalize_vue_output(&output);
        let expected_n = normalize_vue_output(&expected);
        if output_n != expected_n {
            let wrong_path = d.join("test/a2vue/002_markdown/input.wrong.vue");
            std::fs::write(&wrong_path, &output).unwrap();
            panic!(
                "a2vue markdown mismatch. See input.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                expected_n, output_n
            );
        }
    }

    /// Plan 022 限制1: fn 调用作为组件 prop（expr_to_vue_bound_value 的 Expr::Call 分支）。
    /// 验证 `ItemList { items: getList(.msg) }` 生成 `:items="getList(msg)"`（非 null）。
    #[test]
    fn test_a2vue_fn_call_prop() {
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = d.join("test/a2vue/003_fn_call_prop/input.at");
        let src = std::fs::read_to_string(&src_path).unwrap();
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src.as_str()).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("no widget in 003_fn_call_prop input");
        let mut gen = VueGenerator::new();
        let output = gen.generate_sfc(&widget).unwrap();
        let exp_path = d.join("test/a2vue/003_fn_call_prop/input.expected.vue");
        let expected = std::fs::read_to_string(&exp_path).unwrap_or_default();
        let output_n = normalize_vue_output(&output);
        let expected_n = normalize_vue_output(&expected);
        if output_n != expected_n {
            let wrong_path = d.join("test/a2vue/003_fn_call_prop/input.wrong.vue");
            std::fs::write(&wrong_path, &output).unwrap();
            panic!(
                "a2vue fn_call_prop mismatch. See input.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                expected_n, output_n
            );
        }
    }

    /// DF-1: nested if in style binding → nested ternary (golden).
    #[test]
    fn test_a2vue_nested_if_style() {
        test_a2vue("004_nested_if_style").expect("a2vue nested_if_style golden mismatch");
    }

    /// Plan 407: text 节点函数调用 primary prop（i18n t() 支持）。
    /// 验证 `text t("nav.chat")` 生成 `{{ t('nav.chat') }}`。
    #[test]
    fn test_a2vue_text_fn_call() {
        test_a2vue("005_text_fn_call").expect("a2vue text_fn_call golden mismatch");
    }

    /// Plan 407: 外部组件（lucide 图标）作 HTML 元素子节点 + text 共存。
    /// 验证 `button { Plus { size: 14 } text "新建" }` 生成
    /// `<button><Plus :size="14" .../>新建</button>`，覆盖 text+children 共存分支。
    #[test]
    fn test_a2vue_icon_child() {
        test_a2vue("006_icon_child").expect("a2vue icon_child golden mismatch");
    }

    /// PLAN-621: icon state 契约双端金样（Web 腿）——五种形态见
    /// `test/a2vue/012_icon_state/input.at` 头注；VM 腿由
    /// `aura_view_builder` 的 6 个 plan621 单测覆盖。
    #[test]
    fn test_a2vue_icon_state() {
        test_a2vue("012_icon_state").expect("a2vue icon_state golden mismatch");
    }

    /// os-config OS-015/016: select 双形态选项 + value/label 契约（Web 腿）——
    /// `test/a2vue/013_select_children/input.at` 头注；VM 腿缺陷
    /// （convert_select 循环选项漏收等）由 examples/select_demo.at + 上游
    /// 修复计划覆盖。
    #[test]
    fn test_a2vue_select_children() {
        test_a2vue("013_select_children").expect("a2vue select_children golden mismatch");
    }

    /// Plan 408: `component fn` → independent Vue SFC synthesis.
    ///
    /// `component fn Card(title str)` must be synthesized to its own Card.vue
    /// (with `defineProps<{ title: string }>()`), and the App widget must
    /// reference it via `<Card :title=.../>` + `import Card from ...` instead
    /// of inlining the fragment body. Verifies the full production path
    /// (`generate_component_from_file`), not just extraction.
    #[test]
    fn test_a2vue_component_fn() {
        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let at_path = d.join("test/a2vue/007_component_fn/input.at");
        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .expect("007_component_fn generation failed");

        // App is the first widget; Card is synthesized after it.
        let app_code = result
            .all_widget_codes
            .iter()
            .find(|(name, _)| name == "App")
            .map(|(_, code)| code)
            .expect("App widget not in all_widget_codes");
        let card_code = result
            .all_widget_codes
            .iter()
            .find(|(name, _)| name == "Card")
            .map(|(_, code)| code)
            .expect("Card component fn not synthesized into all_widget_codes");

        // App references Card as a component, not inlined DOM.
        let app_n = normalize_vue_output(app_code);
        let card_n = normalize_vue_output(card_code);

        // Write golden files (created on first run / updated intentionally).
        let app_golden = d.join("test/a2vue/007_component_fn/App.expected.vue");
        let card_golden = d.join("test/a2vue/007_component_fn/Card.expected.vue");
        let app_expected =
            normalize_vue_output(&std::fs::read_to_string(&app_golden).unwrap_or_default());
        let card_expected =
            normalize_vue_output(&std::fs::read_to_string(&card_golden).unwrap_or_default());

        if app_n != app_expected {
            let wrong = d.join("test/a2vue/007_component_fn/App.wrong.vue");
            std::fs::write(&wrong, app_code).unwrap();
            panic!(
                "007_component_fn App mismatch. See App.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                app_expected, app_n
            );
        }
        if card_n != card_expected {
            let wrong = d.join("test/a2vue/007_component_fn/Card.wrong.vue");
            std::fs::write(&wrong, card_code).unwrap();
            panic!(
                "007_component_fn Card mismatch. See Card.wrong.vue.\n--- expected ---\n{}\n--- actual ---\n{}",
                card_expected, card_n
            );
        }

        // Structural assertions: the key Plan 408 invariants.
        assert!(
            app_code.contains("import Card from '@/components/Card.vue'"),
            "App must import Card component: {}",
            app_code
        );
        assert!(
            app_code.contains("<Card"),
            "App must reference Card via a component tag: {}",
            app_code
        );
        assert!(
            !app_code.contains("text-muted-foreground"),
            "App must NOT inline Card's body DOM (no text-muted-foreground): {}",
            app_code
        );
        assert!(
            card_code.contains("defineProps"),
            "Card SFC must define props: {}",
            card_code
        );
        assert!(
            card_code.contains("title"),
            "Card SFC must declare the title prop: {}",
            card_code
        );
    }

    /// Plan 022 限制2: composable 带参调用。验证
    /// `composable: useStreamingDocument(.source) from "..."` 生成
    /// `const streamingDocument = useStreamingDocument(source)`，
    /// 且无参 composable 仍是 `useX()`（向后兼容）。
    #[test]
    fn test_composable_with_args() {
        let src = r#"
widget CompWithComposable {
    use {
        composable: useStreamingDocument(.source) from "src/front/useStreamingDocument.ts"
        composable: useClock from "src/front/useClock.ts"
    }
    model { var source str = "" }
    view { col { text "hi" } }
    on { }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("no widget");
        let mut gen = VueGenerator::new();
        let output = gen.generate_sfc(&widget).unwrap();
        // 带参：useStreamingDocument(source)
        assert!(
            output.contains("const streamingDocument = useStreamingDocument(source)"),
            "带参 composable 应生成 useStreamingDocument(source)，实际:\n{}",
            output
        );
        // 无参默认：useClock()（向后兼容）
        assert!(
            output.contains("const clock = useClock()"),
            "无参 composable 应仍是 useClock()，实际:\n{}",
            output
        );
    }

    /// Plan 012 batch F (gap 6): early return inside an on-handler guard.
    /// `if .busy { return }` emits an early `return;` inside the if; the
    /// statements after the guard are still emitted (unreachable at runtime
    /// when the guard holds — same as JS semantics).
    #[test]
    fn test_early_return_guarded_if() {
        // Real parse path (plan 012 batch C idiom).
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Guarded {
    msg Msg { Save }
    model {
        var count int = 0
        var busy bool = false
    }
    view { col { button "save" onclick: .Save } }
    on {
        .Save -> {
            if .busy { return }
            .count += 1
        }
    }
}
"#,
        );
        assert!(sfc.contains("function Save()"), "handler emitted:\n{}", sfc);
        assert!(
            sfc.contains("if (busy.value) {return;"),
            "early return inside guard:\n{}",
            sfc
        );
        assert!(
            sfc.contains("count.value += 1;"),
            "statements after the guard still emitted:\n{}",
            sfc
        );
    }

    /// Plan 448 B1: `onclick: () => { ... }` inline lambda — the template
    /// binds the minted `__evt_*` function, the body transpiles like any
    /// on-handler, and NO trailing `emit('__evt_...')` is appended (the
    /// synthetic event is widget-internal; no parent can listen for it).
    #[test]
    fn test_inline_lambda_event_generates_handler_without_emit() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Counter {
    model { var count int = 0 }
    view {
        row {
            button "-" { onclick: () => {.count -= 1} }
            button "+" { onclick: () => {.count += 1} }
        }
    }
}
"#,
        );
        assert!(
            sfc.contains("@click=\"__evt_onclick_1\""),
            "template binds the minted name:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function __evt_onclick_1()"),
            "minted handler function emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("count.value -= 1;"),
            "body transpiled:\n{}",
            sfc
        );
        assert!(
            sfc.contains("count.value += 1;"),
            "second body transpiled:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("emit('__evt_"),
            "no trailing emit for synthetic events:\n{}",
            sfc
        );
    }

    /// Plan 012 batch F (gap 6): a bare `return` at end of line is an early
    /// return, NOT a return of the next statement. Regression lock for the
    /// old newline-skipping parse that turned
    ///     return
    ///     .count += 1
    /// into `return count.value += 1;`.
    #[test]
    fn test_early_return_bare_newline_terminates_statement() {
        // Real parse path.
        let sfc = gen_sfc_from_widget_src(
            r#"
widget TopLevel {
    msg Msg { Save }
    model { var count int = 0 }
    view { col { button "save" onclick: .Save } }
    on {
        .Save -> {
            return
            .count += 1
        }
    }
}
"#,
        );
        assert!(
            !sfc.contains("return count.value"),
            "bare return must not swallow the next statement:\n{}",
            sfc
        );
        assert!(sfc.contains("return;"), "bare return emitted:\n{}", sfc);
        // The following statement is still emitted (runtime-unreachable,
        // matching JS semantics of code after `return;`).
        assert!(
            sfc.contains("count.value += 1;"),
            "statement after bare return still emitted:\n{}",
            sfc
        );
    }

    /// Plan 409 §8: a widget (non-store) that owns an `accent_color` state
    /// must get the widget-path applyAccent injection — palette JS,
    /// onMounted restore, and `applyAccent(...)` calls appended to the
    /// SetAccent handler (mirrors the store composable path at Plan 360).
    #[test]
    fn test_widget_accent_color_injects_apply_accent() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ThemeApp {
    msg Msg { SetAccent(str) }
    model {
        themeOpen bool = false
        accent_color str = "indigo"
    }
    view {
        col {
            button "palette" onclick: .toggleTheme
            button "coral" onclick: .SetAccent("coral")
        }
    }
    on {
        .toggleTheme -> { themeOpen = !themeOpen }
        .SetAccent(name) -> { accent_color = name }
    }
}
"#,
        );
        assert!(
            sfc.contains("const ACCENT_PALETTES"),
            "widget path must inject the accent palette JS:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function applyAccent"),
            "widget path must inject applyAccent():\n{}",
            sfc
        );
        assert!(
            sfc.contains("getSavedAccent()"),
            "widget path must restore saved accent on mount:\n{}",
            sfc
        );
        // N-22：容器作用域 widget 的 handler 必须把容器根传给 applyAccent
        //（否则 N-11 的嵌套根跳过会让本实例的 --primary 停留在旧值）。
        assert!(
            sfc.contains("applyAccent(accent_color.value, __autoThemeRoot.value ?"),
            "SetAccent handler must pass the container ref to applyAccent:\n{}",
            sfc
        );
        assert!(
            sfc.contains("accent_color = ref<string>"),
            "accent_color must be declared as a ref from the model:\n{}",
            sfc
        );
    }

    /// PLAN-601 T-06: the widget path (accent gate) also injects the full
    /// named-theme runtime — THEME_PALETTES (five builtins), applyTheme /
    /// getActiveTheme, boot restore, and the mode-flip watch upgraded from
    /// accent-only re-apply to a whole-theme re-apply (dark pass keeps a
    /// non-default theme across `.dark` flips).
    #[test]
    fn test_widget_theme_runtime_injected_with_accent() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ThemeApp {
    msg Msg { ToggleDark }
    model {
        dark_mode bool = false
        accent_color str = "indigo"
    }
    view { col { button "go dark" onclick: .ToggleDark } }
    on { .ToggleDark -> { dark_mode = !dark_mode } }
}
"#,
        );
        assert!(
            sfc.contains("const THEME_PALETTES"),
            "widget path must inject THEME_PALETTES:\n{}",
            sfc
        );
        for name in ["zinc", "scaffold", "stella", "tauri", "cli-vue"] {
            assert!(
                sfc.contains(&format!("'{name}': {{")),
                "THEME_PALETTES must carry builtin '{name}':\n{}",
                sfc
            );
        }
        assert!(
            sfc.contains("function applyTheme") && sfc.contains("function getActiveTheme"),
            "widget path must inject applyTheme()/getActiveTheme():\n{}",
            sfc
        );
        assert!(
            sfc.contains("if (__th !== 'scaffold') applyTheme(__th,"),
            "widget onMounted must restore the active theme:\n{}",
            sfc
        );
        assert!(
            sfc.contains("applyTheme(__thName(), v, accent_color.value"),
            "mode-flip watch must re-apply the whole active theme:\n{}",
            sfc
        );
        // Accent overlay stays cohesive inside applyTheme (last write wins).
        assert!(
            sfc.contains("if (accentName) applyAccent(accentName, isDark"),
            "applyTheme must end with the accent overlay:\n{}",
            sfc
        );
    }

    /// PLAN-031 T-06（F-1）：声明 `design { theme: .theme_name }` 的 App，
    /// 首帧必须应用**声明的默认族**——mount 取 `theme_name.value` 优先于
    /// localStorage 的持久化选择；否则官方包（Auto/Solarized）的颜色在 Vue
    /// 侧永远不上身，只剩静态 scaffold 表。
    /// N-34：theme_name 存在但没有 dark_mode 时，`watch(theme_name, …)` 仍会
    /// 发射——`watch` 的 import 不能只按 has_dark_mode 门控，否则运行期
    /// `ReferenceError: watch is not defined`。
    #[test]
    fn test_theme_name_without_dark_mode_imports_watch() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ThemeNameOnly {
    model {
        var theme_name str = "Auto"
    }
    view { col { text "probe" } }
}
"#,
        );
        assert!(
            sfc.contains("watch(theme_name"),
            "theme_name watcher must be emitted:\n{sfc}"
        );
        let import_line = sfc
            .lines()
            .find(|l| l.trim_start().starts_with("import {"))
            .unwrap_or("");
        assert!(
            import_line.contains("watch"),
            "watch must be imported for theme_name-only widgets, import line = {import_line:?}\n{sfc}"
        );
    }

    #[test]
    fn test_widget_mount_applies_declared_design_family() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget ThemeFamilyApp {
    model {
        var theme_name str = "Auto"
        var dark_mode bool = false
    }
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }
    view { col { text "probe" } }
}
"#,
        );
        // PLAN-031 F-03：声明了 design.theme 的 widget 走 computed 投影，
        // `__thName` 优先读投影值（theme_name 作兼容别名兜底）。
        assert!(
            sfc.contains("const __designTheme = computed((): string =>"),
            "design.theme 应投影为 computed（F-03）：\n{}",
            sfc
        );
        assert!(
            sfc.contains("__designTheme.value"),
            "自动应用点必须优先读 design 投影（F-03）：\n{}",
            sfc
        );
        assert!(
            sfc.contains("watch(theme_name"),
            "theme_name 热切换 watcher 必须保留：\n{}",
            sfc
        );
        // F-1 续：多 App 同文档时 `.dark` 挂 <html>，作用域根是容器——
        // dark 面必须直接写到容器根，否则子树停在 light 组（半暗半亮）。
        assert!(
            sfc.contains("if (rootEl || root.classList.contains('dark')) writeVars(root, pal!.dark)"),
            "容器作用域的 dark 面必须写到 rootEl：\n{}",
            sfc
        );
    }

    /// PLAN-601 T-06: no accent_color state → zero theme runtime injection
    /// (zero-change rule for apps outside the theme feature cohort).
    #[test]
    fn test_widget_theme_runtime_absent_without_accent() {
        let sfc = gen_sfc_from_widget_src(
            r#"
widget Plain {
    model { var count int = 0 }
    view { col { button "inc" onclick: .inc } }
    on { .inc -> { count += 1 } }
}
"#,
        );
        assert!(
            !sfc.contains("THEME_PALETTES") && !sfc.contains("function applyTheme"),
            "plain widget must not carry the theme runtime:\n{}",
            sfc
        );
    }

    /// PLAN-601 T-06: the store composable path injects the same runtime and
    /// restores the active theme in its module bootstrap before the accent.
    #[test]
    fn test_store_composable_theme_runtime() {
        let code = VueGenerator::generate_store_composable(&store_from_src(
            r#"
store Shell {
    model {
        var dark_mode bool = false
        var accent_color str = "indigo"
    }
    msg Msg { ToggleDark }
    on { .ToggleDark -> { dark_mode = !dark_mode } }
}
"#,
        ));
        assert!(
            code.contains("const THEME_PALETTES") && code.contains("function applyTheme"),
            "store path must inject the theme runtime:\n{}",
            code
        );
        assert!(
            code.contains("const __th = getActiveTheme()"),
            "store bootstrap must restore the active theme:\n{}",
            code
        );
        assert!(
            code.contains("if (accentName) applyAccent(accentName, isDark"),
            "accent overlay must be cohesive in applyTheme:\n{}",
            code
        );
    }

    #[test]
    fn test_spacer_generates_flex1() {
        let mut gen = VueGenerator::new();
        let props = HashMap::new();
        let (classes, _, _) = gen.extract_classes("spacer", &props);
        assert_eq!(classes, "flex-1");

        let mut gen_shadcn = VueGenerator::new_shadcn();
        let (classes_shadcn, _, _) = gen_shadcn.extract_classes("spacer", &props);
        assert_eq!(classes_shadcn, "flex-1");

        let src = r#"
widget TestSpacer {
    view {
        row {
            span "Left"
            spacer
            span "Right"
        }
    }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("no widget");
        let mut gen_sfc = VueGenerator::new_shadcn();
        let sfc = gen_sfc.generate_sfc(&widget).unwrap();
        assert!(
            sfc.contains("<div class=\"flex-1\" />"),
            "SFC should contain <div class=\"flex-1\" />, got:\n{}",
            sfc
        );
    }

    #[test]
    fn test_col_gap_override_no_gap4() {
        let src = r#"
widget TestGap {
    view {
        col {
            span "Title"
            span "Subtitle"
            style: "gap-1"
        }
    }
}
"#;
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(src).with_session(session);
        let ast = crate::parse_ui(&src).unwrap();
        let widget = ast
            .domain_stmts()
            .find_map(|s| {
                if let auto_ui_dialect::UiStmt::WidgetDecl(w) = s {
                    crate::aura::extract_widget_from_decl(w).ok()
                } else {
                    None
                }
            })
            .expect("no widget");
        let mut gen_sfc = VueGenerator::new_shadcn();
        let sfc = gen_sfc.generate_sfc(&widget).unwrap();
        assert!(
            sfc.contains("<div class=\"flex flex-col gap-1\">"),
            "SFC should contain <div class=\"flex flex-col gap-1\">, got:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("gap-4"),
            "SFC should NOT contain gap-4, got:\n{}",
            sfc
        );
    }

    // ====================================================================
    // Plan 522 — use 导入的模块级 helper fn 按需发射进消费方 SFC。
    // 池 + 导入名集合由 api.rs 的 collect_use_module_fns 从真实模块文件
    // 产出；单测用模块源串直接构造同样的池（Parser → extract_module_fn），
    // 与 gen_sfc_from_widget_src 一样走真实解析路径。
    // ====================================================================

    /// Plan 522: widget 源 + 模块源 + 显式导入的裸名集合 → SFC。
    fn gen_sfc_with_use_fns(widget_src: &str, module_src: &str, imported: &[&str]) -> String {
        gen_sfc_with_use_fns_and_warnings(widget_src, module_src, imported).0
    }

    fn gen_sfc_with_use_fns_and_warnings(
        widget_src: &str,
        module_src: &str,
        imported: &[&str],
    ) -> (String, Vec<crate::ui_gen::validators::ValidationWarning>) {
        let session = auto_lang_core::session::CompilerSession::ui();
        let mut parser = auto_lang_core::parser::Parser::from(widget_src).with_session(session);
        let ast = crate::parse_ui(widget_src).expect("widget source must parse");
        let decl = ast
            .domain_stmts()
            .find_map(|s| match s {
                auto_ui_dialect::UiStmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");

        let mast = crate::parse_ui(module_src).expect("module source must parse");
        let pool: Vec<crate::aura::AuraModuleFn> = mast
            .core_stmts()
            .filter_map(|s| {
                if let auto_lang_core::ast::Stmt::Fn(f) = s {
                    crate::aura::extract_module_fn(f)
                } else {
                    None
                }
            })
            .collect();

        let mut gen = VueGenerator::new()
            .with_use_module_fns(pool, imported.iter().map(|s| s.to_string()).collect());
        let sfc = gen.generate(&widget).expect("generate SFC");
        (sfc, gen.last_validation_warnings.clone())
    }

    #[test]
    fn test_plan522_use_fn_called_in_expr_computed_is_emitted() {
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Doublers {
    model { var n int = 3 }
    computed { twice => double(.n) }
    view { col { text `d: ${.twice}` } }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        assert!(
            sfc.contains("function double(x: any): number {"),
            "helper fn transpiled into SFC script:\n{}",
            sfc
        );
        assert!(
            sfc.contains("double(n.value)"),
            "call site in computed keeps bare name (zero rewriting):\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_unreferenced_is_not_emitted() {
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Pick {
    model { var n int = 3 }
    computed { twice => double(.n) }
    view { col { text `d: ${.twice}` } }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}

pub fn unused_helper(x int) int {
    return x + 100
}
"#,
            &["double", "unused_helper"],
        );
        assert!(
            sfc.contains("function double(x: any): number {"),
            "referenced helper emitted:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("unused_helper"),
            "unreferenced helper must NOT be emitted (on-demand pull):\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_closure_deps_pulled_to_fixpoint() {
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Grid {
    model { var n int = 3 }
    computed { labeled => label_it(square(.n)) }
    view { col { text `d: ${.labeled}` } }
}
"#,
            r#"
pub fn square(x int) int {
    return triple(x) + x
}

pub fn triple(x int) int {
    return x * 3
}

pub fn label_it(x int) str {
    return "v=" + x.to_string()
}

pub fn unrelated(x int) int {
    return x
}
"#,
            &["square", "label_it", "triple", "unrelated"],
        );
        assert!(
            sfc.contains("function square(x: any): number {"),
            "entry fn emitted:\n{}",
            sfc
        );
        assert!(
            sfc.contains("function triple(x: any): number {"),
            "intra-module closure dep emitted (square -> triple):\n{}",
            sfc
        );
        assert!(
            sfc.contains("function label_it(x: any): string {"),
            "second entry fn emitted:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("unrelated"),
            "non-referenced pool fn not emitted:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_block_body_computed_calls_helper() {
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Blocky {
    model { var n int = 3 }
    computed {
        twice => {
            let base = double(.n)
            return base + 1
        }
    }
    view { col { text `d: ${.twice}` } }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        assert!(
            sfc.contains("function double(x: any): number {"),
            "block-body computed referencing helper pulls it in:\n{}",
            sfc
        );
        assert!(
            sfc.contains("double(n.value)"),
            "call site in block body unchanged:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_called_in_handler_body_is_emitted() {
        let sfc = gen_sfc_with_use_fns(
            r#"
widget HandlerCaller {
    msg Msg { Go }
    model { var n int = 3 }
    view { col { button "go" (onclick: .Go) } }
    on {
        .Go -> { .n = double(.n) }
    }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        assert!(
            sfc.contains("function double(x: any): number {"),
            "handler-body call pulls helper fn in:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_not_imported_is_not_pulled() {
        // `name_it` lives in the pool but is not in any `use module: ...`
        // items list — widget-body bare calls to it must NOT pull it in
        // (mirrors vm import_aliases: only imported items resolve bare).
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Gated {
    model { var n int = 3 }
    computed { twice => name_it(.n) }
    view { col { text `d: ${.twice}` } }
}
"#,
            r#"
pub fn name_it(x int) str {
    return "v"
}
"#,
            &["something_else"],
        );
        assert!(
            !sfc.contains("function name_it"),
            "pool fn outside the imported-name gate must not be emitted:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_dedup_across_computed_and_handler() {
        // Same helper referenced from BOTH a computed and a handler — the
        // pulled set is name-keyed, so the SFC must contain exactly one
        // function declaration.
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Dedup {
    msg Msg { Go }
    model { var n int = 3 }
    computed { twice => double(.n) }
    view { col { button "go" (onclick: .Go) } }
    on {
        .Go -> { .n = double(.n) }
    }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        let count = sfc.matches("function double(").count();
        assert_eq!(
            count, 1,
            "helper referenced from computed AND handler emits exactly once:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_suppressed_by_ext_import_same_name() {
        // ext_imports (Plan 051 hand-written TS escape hatch) already binds
        // the same symbol — hand-written wins, the pooled fn is NOT emitted.
        let sfc = gen_sfc_with_use_fns(
            r#"
widget ExtWins {
    use { fn: double from "./double.ts" }
    model { var n int = 3 }
    computed { twice => double(.n) }
    view { col { text `d: ${.twice}` } }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        assert!(
            sfc.contains("import { double } from '@/ext/double'"),
            "ext import line still emitted:\n{}",
            sfc
        );
        assert!(
            !sfc.contains("function double("),
            "ext_imports-declared symbol suppresses pooled fn emission:\n{}",
            sfc
        );
    }

    #[test]
    fn test_plan522_use_fn_name_conflict_with_state_warns_and_skips() {
        // Helper shares its name with a state var — emitting the function
        // declaration would collide in JS scope. v1: R013 warning + skip
        // emission (ext_imports is the escape hatch).
        let (sfc, warnings) = gen_sfc_with_use_fns_and_warnings(
            r#"
widget Clashing {
    msg Msg { Go }
    model { var double int = 1 }
    computed { twice => double(.n) }
    view { col { button "go" (onclick: .Go) } }
    on {
        .Go -> { .double = 2 }
    }
}
"#,
            r#"
pub fn double(x int) int {
    return x * 2
}
"#,
            &["double"],
        );
        assert!(
            !sfc.contains("function double("),
            "name-conflicting helper must not be emitted:\n{}",
            sfc
        );
        let r013 = warnings_for_rule(&warnings, "R013");
        assert!(
            r013.iter().any(|w| w.message.contains("double")),
            "name conflict must raise an R013 warning naming the fn:\n{warnings:?}"
        );
    }

    #[test]
    fn test_plan522_use_fn_trailing_expr_body_emits_return() {
        // Auto fn 体的尾表达式即返回值(VM 语义)——发射的 TS 必须带上
        // `return`,否则 Vue 运行时拿到 undefined(谓词恒 falsy)。
        let sfc = gen_sfc_with_use_fns(
            r#"
widget Evenness {
    model { var n int = 4 }
    computed { ok => is_even(.n) }
    view { col { text `e: ${.ok}` } }
}
"#,
            r#"
pub fn is_even(x int) bool {
    x % 2 == 0
}
"#,
            &["is_even"],
        );
        assert!(
            sfc.contains("return x %2 == 0;"),
            "trailing-expression fn body must emit `return`:\n{}",
            sfc
        );
    }

    /// Plan 522 语料回归锁:016-calendar 真实源(含 use calendar_util 导入
    /// + computed 调用)经生产管线生成 —— helper fn 闭包全量转译进 SFC。
    #[test]
    fn test_plan522_016_corpus_helpers_emitted() {
        let front = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples/ui/016-calendar/src/front");
        let Ok(result) = crate::ui_gen::api::generate_component_from_file(
            &front.join("app.at"),
            crate::ui_gen::ComponentGenOptions::default(),
        ) else {
            eprintln!("016 corpus unreadable — skip");
            return;
        };
        let sfc = &result.vue_code;
        // 入口 fn(month_name/build_month_grid)与闭包依赖(day_style 等)都发射。
        for fn_name in [
            "month_name",
            "build_month_grid",
            "day_style",
            "weekday_of",
            "format_date",
        ] {
            assert!(
                sfc.contains(&format!("function {fn_name}(")),
                "helper `{fn_name}` must be transpiled into the 016 SFC:\n{sfc}"
            );
        }
        // computed 调用点零改写(裸名 + store facade 解析)。
        assert!(
            sfc.contains("const month_label = computed<any>(() => month_name(store.month))"),
            "month_label computed keeps the bare-name call:\n{sfc}"
        );
        assert!(
            sfc.contains(
                "build_month_grid(store.year, store.month, store.today, store.selected_date)"
            ),
            "days computed keeps the bare-name call:\n{sfc}"
        );
    }

    // PLAN-528 W7 回归:togglegroup 族标签必须经 WidgetRegistry 映射到
    // shadcn-vue 组件(此前 schema/registry 双缺,塌缩为裸 div,B/I/U
    // 纵向裸排)。三种书写拼写(无连字符/kebab)全部覆盖。
    #[test]
    fn test_plan528_toggle_group_registry_mapping() {
        let mut gen = VueGenerator::new().with_mode(crate::ui_gen::VueMode::Shadcn);
        assert_eq!(gen.map_tag("togglegroup", false), "ToggleGroup");
        assert_eq!(gen.map_tag("toggle-group", false), "ToggleGroup");
        assert_eq!(gen.map_tag("togglegroup-item", false), "ToggleGroupItem");
        assert_eq!(gen.map_tag("toggle-group-item", false), "ToggleGroupItem");
    }

    #[test]
    fn test_plan522_use_fn_pattern_match_body_warns_and_skips() {
        // v1 transpile boundary: pattern-matching fn bodies (`is` statements)
        // are not guaranteed to transpile — R013 warning + no emission
        // (fallback to the pre-Plan-522 behavior), never blocking generation.
        let (sfc, warnings) = gen_sfc_with_use_fns_and_warnings(
            r#"
widget Patterny {
    model { var n int = 3 }
    computed {
        label => classify(.n)
        twice => double(.n)
    }
    view { col { text `d: ${.label}` } }
}
"#,
            r#"
pub fn classify(x int) str {
    is x {
        0 -> { return "zero" }
        else -> { return "many" }
    }
    return "?"
}

pub fn double(x int) int {
    return x * 2
}
"#,
            &["classify", "double"],
        );
        assert!(
            !sfc.contains("function classify("),
            "pattern-matching helper must not be emitted:\n{}",
            sfc
        );
        let r013 = warnings_for_rule(&warnings, "R013");
        assert!(
            r013.iter().any(|w| w.message.contains("classify")),
            "unsupported transpile shape must raise an R013 warning naming the fn:\n{warnings:?}"
        );
        // Other, supported helpers are unaffected.
        assert!(
            sfc.contains("function double(x: any): number {"),
            "supported sibling helper still emitted:\n{}",
            sfc
        );
    }
}

/// PLAN-063 Phase B T15 (KD 061 D13): store 组合式命名归一——DSL store
/// 名以 `Store` 结尾时(如 AuthStore)剥一层再补,消除 useAuthStoreStore
/// 双后缀(文件名/导出 fn/消费 import 三面同源)。裸名(NotesLike)不变。
pub fn store_composable_name(store_name: &str) -> String {
    let stem = store_name
        .strip_suffix("Store")
        .filter(|s| !s.is_empty())
        .unwrap_or(store_name);
    format!("use{}Store", stem)
}

/// PLAN-048 (auto-musk A 线): 跨 store facade 变量名——`ForgeStore` →
/// `forgeStore`(剥 Store 后缀、首字母小写、回补 Store)。
fn facade_var_for(dep: &str) -> String {
    let stem = dep.strip_suffix("Store").unwrap_or(dep);
    let mut c = stem.chars();
    match c.next() {
        Some(f) => format!(
            "{}{}Store",
            f.to_lowercase().collect::<String>(),
            c.as_str()
        ),
        None => format!("{}Store", stem),
    }
}

// ============================================================================
// Plan 522 — use 导入 helper fn 的引用收集(裸名调用点扫描)
// ============================================================================

/// 收集表达式中所有裸名调用的被调名(`foo(...)` 的 `foo`;含嵌套实参内
/// 的调用)。方法调用(`obj.foo(...)`)不计 —— helper fn 只以裸名形态被
/// widget 体引用。覆盖 handler/fn 体可能出现的复合表达式形态;未列变体
/// 走 `_ => {}`(与 ts_adapter 既有 walker 的口径一致)。
pub(crate) fn collect_called_bare_names_expr(
    expr: &auto_lang_core::ast::Expr,
    out: &mut std::collections::HashSet<String>,
) {
    use auto_lang_core::ast::Expr;
    match expr {
        Expr::Call(call) => {
            if let auto_lang_core::ast::Expr::Ident(name)
            | auto_lang_core::ast::Expr::GenName(name) = call.name.as_ref()
            {
                out.insert(name.to_string());
            }
            for arg in &call.args.args {
                collect_called_bare_names_expr(&arg.get_expr(), out);
            }
        }
        Expr::View(e)
        | Expr::Mut(e)
        | Expr::Move(e)
        | Expr::Take(e)
        | Expr::Unary(_, e)
        | Expr::ErrorPropagate(e)
        | Expr::Some(e)
        | Expr::Ok(e)
        | Expr::Err(e)
        | Expr::BoxExpr(e)
        | Expr::ArcExpr(e)
        | Expr::Await { expr: e }
        | Expr::Go { expr: e } => {
            collect_called_bare_names_expr(e, out);
        }
        Expr::NullCoalesce(l, r) => {
            collect_called_bare_names_expr(l, out);
            collect_called_bare_names_expr(r, out);
        }
        Expr::Bina(l, _, r) => {
            collect_called_bare_names_expr(l, out);
            collect_called_bare_names_expr(r, out);
        }
        Expr::Dot(obj, _) => collect_called_bare_names_expr(obj, out),
        Expr::Range(r) => {
            collect_called_bare_names_expr(&r.start, out);
            collect_called_bare_names_expr(&r.end, out);
        }
        Expr::Array(items) => {
            for e in items {
                collect_called_bare_names_expr(e, out);
            }
        }
        Expr::Pair(p) => collect_called_bare_names_expr(&p.value, out),
        Expr::Object(pairs) => {
            for p in pairs {
                collect_called_bare_names_expr(&p.value, out);
            }
        }
        Expr::Block(body) => collect_called_bare_names_stmts(&body.stmts, out),
        Expr::Index(a, i) => {
            collect_called_bare_names_expr(a, out);
            collect_called_bare_names_expr(i, out);
        }
        Expr::Lambda(f) => collect_called_bare_names_stmts(&f.body.stmts, out),
        Expr::Closure(c) => collect_called_bare_names_expr(&c.body, out),
        Expr::FStr(fstr) => {
            for part in &fstr.parts {
                collect_called_bare_names_expr(part, out);
            }
        }
        Expr::If(if_expr) => collect_if_called_names(if_expr, out),
        Expr::Cast { expr: e, .. } | Expr::To { expr: e, .. } => {
            collect_called_bare_names_expr(e, out)
        }
        Expr::TupleDestruct { expr: e, .. } => collect_called_bare_names_expr(e, out),
        Expr::NavCall { path, params } => {
            collect_called_bare_names_expr(path, out);
            for p in params {
                collect_called_bare_names_expr(&p.value, out);
            }
        }
        Expr::AsyncBlock { body, .. } => collect_called_bare_names_stmts(&body.stmts, out),
        _ => {}
    }
}

fn collect_if_called_names(
    if_expr: &auto_lang_core::ast::If,
    out: &mut std::collections::HashSet<String>,
) {
    for branch in &if_expr.branches {
        collect_called_bare_names_expr(&branch.cond, out);
        collect_called_bare_names_stmts(&branch.body.stmts, out);
    }
    if let Some(else_body) = &if_expr.else_ {
        collect_called_bare_names_stmts(&else_body.stmts, out);
    }
}

/// 语句序列的裸名调用收集(handler 体 / helper fn 体共用)。
pub(crate) fn collect_called_bare_names_stmts(
    stmts: &[auto_lang_core::ast::Stmt],
    out: &mut std::collections::HashSet<String>,
) {
    use auto_lang_core::ast::Stmt;
    for stmt in stmts {
        match stmt {
            Stmt::Expr(e) => collect_called_bare_names_expr(e, out),
            Stmt::If(if_expr) => collect_if_called_names(if_expr, out),
            Stmt::For(f) => {
                collect_called_bare_names_expr(&f.range, out);
                if let Some(init) = &f.init {
                    collect_called_bare_names_stmts(std::slice::from_ref(init), out);
                }
                collect_called_bare_names_stmts(&f.body.stmts, out);
            }
            Stmt::Try(t) => {
                collect_called_bare_names_stmts(&t.body.stmts, out);
                collect_called_bare_names_stmts(&t.catch_body.stmts, out);
                if let Some(fb) = &t.finally_body {
                    collect_called_bare_names_stmts(&fb.stmts, out);
                }
            }
            Stmt::Is(is) => {
                collect_called_bare_names_expr(&is.target, out);
                for branch in &is.branches {
                    match branch {
                        auto_lang_core::ast::IsBranch::EqBranch(patterns, body) => {
                            for p in patterns {
                                collect_called_bare_names_expr(p, out);
                            }
                            collect_called_bare_names_stmts(&body.stmts, out);
                        }
                        auto_lang_core::ast::IsBranch::IfBranch(cond, body) => {
                            collect_called_bare_names_expr(cond, out);
                            collect_called_bare_names_stmts(&body.stmts, out);
                        }
                        auto_lang_core::ast::IsBranch::ElseBranch(body) => {
                            collect_called_bare_names_stmts(&body.stmts, out);
                        }
                    }
                }
            }
            Stmt::Store(s) => collect_called_bare_names_expr(&s.expr, out),
            Stmt::Block(b) => collect_called_bare_names_stmts(&b.stmts, out),
            Stmt::Return(e) | Stmt::Reply(e) => collect_called_bare_names_expr(e, out),
            Stmt::Fn(f) => collect_called_bare_names_stmts(&f.body.stmts, out),
            _ => {}
        }
    }
}

/// Plan 522 v1 转译边界预扫描:fn 体含模式匹配(`is`/cover 族)或借用/堆
/// 语义表达式时不发射(R013 警告 + 回退现状)。返回命中的原因描述。
/// 纯算术/字符串/列表 fn 不触发 —— 这正是 v1 保证支持的形态。
fn use_fn_body_unsupported(stmts: &[auto_lang_core::ast::Stmt]) -> Option<&'static str> {
    use auto_lang_core::ast::{Expr, Stmt};

    fn expr_unsupported(expr: &Expr) -> Option<&'static str> {
        match expr {
            Expr::Cover(_) | Expr::Uncover(_) => Some("cover pattern matching"),
            Expr::OptionPattern(_)
            | Expr::ResultPattern(_)
            | Expr::OptionUncover(_)
            | Expr::ResultUncover(_)
            | Expr::StructPattern(_) => Some("pattern matching"),
            Expr::View(_) | Expr::Mut(_) | Expr::Move(_) | Expr::Take(_) | Expr::Hold(_) => {
                Some("borrow/ownership semantics")
            }
            Expr::Call(call) => expr_unsupported(&call.name).or_else(|| {
                call.args
                    .args
                    .iter()
                    .find_map(|a| expr_unsupported(&a.get_expr()))
            }),
            Expr::Bina(l, _, r) => expr_unsupported(l).or_else(|| expr_unsupported(r)),
            Expr::Unary(_, e) => expr_unsupported(e),
            _ => None,
        }
    }

    fn stmts_unsupported(stmts: &[Stmt]) -> Option<&'static str> {
        for stmt in stmts {
            match stmt {
                Stmt::Is(_) => return Some("pattern matching (`is`)"),
                Stmt::Expr(e) => {
                    if let Some(r) = expr_unsupported(e) {
                        return Some(r);
                    }
                }
                Stmt::If(i) => {
                    for b in &i.branches {
                        if let Some(r) = expr_unsupported(&b.cond) {
                            return Some(r);
                        }
                        if let Some(r) = stmts_unsupported(&b.body.stmts) {
                            return Some(r);
                        }
                    }
                    if let Some(else_body) = &i.else_ {
                        if let Some(r) = stmts_unsupported(&else_body.stmts) {
                            return Some(r);
                        }
                    }
                }
                Stmt::For(f) => {
                    if let Some(r) = expr_unsupported(&f.range) {
                        return Some(r);
                    }
                    if let Some(init) = &f.init {
                        if let Some(r) = stmts_unsupported(std::slice::from_ref(init)) {
                            return Some(r);
                        }
                    }
                    if let Some(r) = stmts_unsupported(&f.body.stmts) {
                        return Some(r);
                    }
                }
                Stmt::Try(t) => {
                    for body in [
                        &t.body,
                        &t.catch_body,
                        t.finally_body.as_ref().unwrap_or(&t.body),
                    ] {
                        // finally 为 None 时重复查 body 无害(单调判定)。
                        if let Some(r) = stmts_unsupported(&body.stmts) {
                            return Some(r);
                        }
                    }
                }
                Stmt::Store(s) => {
                    if let Some(r) = expr_unsupported(&s.expr) {
                        return Some(r);
                    }
                }
                Stmt::Block(b) => {
                    if let Some(r) = stmts_unsupported(&b.stmts) {
                        return Some(r);
                    }
                }
                Stmt::Return(e) | Stmt::Reply(e) => {
                    if let Some(r) = expr_unsupported(e) {
                        return Some(r);
                    }
                }
                Stmt::Fn(f) => {
                    if let Some(r) = stmts_unsupported(&f.body.stmts) {
                        return Some(r);
                    }
                }
                _ => {}
            }
        }
        None
    }

    stmts_unsupported(stmts)
}

// ── PLAN-571: Vue 臂 cva 表与 Rust 侧 variants.rs 互锁锚定 ──────────
// ui feature 关闭时 crate::ui 不存在，互锁无意义——整模块随门关闭。
#[cfg(all(test, feature = "ui"))]
mod plan571_variants_cva_interlock_tests {
    use super::library_template;
    use crate::style::variants::button_variant_preset;

    /// cva variant 值与 Rust preset 的核心类逐 token 一致（rounded-md/font-medium
    /// 由 cva base 承担，跳过）。改任一侧须同步另一侧。
    #[test]
    fn button_cva_variant_values_match_rust_presets() {
        let tpl = library_template("button").expect("button 模板在册");
        let variants_ts = tpl
            .extra_support_files
            .iter()
            .find(|(name, _)| *name == "variants.ts")
            .map(|(_, body)| *body)
            .expect("variants.ts 在册");
        // 严格互锁范围 = PLAN-571 触碰的键 + destructive（串面完全一致）。
        // ghost 不入strict集：VM 臂 hover:bg-secondary vs web hover:bg-accent
        // 是既有分歧（iced 无 Accent token，KNOWN-DEBT），待独立 plan 收口。
        for key in ["default", "primary", "submit", "secondary", "destructive"] {
            let line = variants_ts
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{}:", key)))
                .unwrap_or_else(|| panic!("cva 缺 {} 键", key));
            let value = line.split('\'').nth(1).expect("cva 值带引号");
            for class in button_variant_preset(key).split_whitespace() {
                if class == "rounded-md" || class == "font-medium" {
                    continue; // cva base 类承担
                }
                assert!(
                    value.split_whitespace().any(|c| c == class),
                    "cva[{}]={:?} 缺 Rust preset 类 {:?}",
                    key,
                    value,
                    class
                );
            }
        }
        // PLAN-080：Button cva 无 defaultVariants——裸 Button（无 variant/
        // size prop）不落任何 variant/size 类，与 Rust variants.rs 缺省
        // chromeless 互锁；显式 variant='default' 才得 PLAN-571 基线。
        assert!(
            !variants_ts.contains("defaultVariants"),
            "button cva 不应再有 defaultVariants（PLAN-080 缺省 chromeless）"
        );
    }
}
