//! PLAN-014 T-04：script 责任（<script setup> 发射、handler/TS 管线、API 调用提取、Vue 文本/边界值发射）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    /// Generate <script setup> content
    pub(super) fn generate_script(&mut self, widget: &AuraWidget) -> GenResult<String> {
        let mut script = String::new();

        // Determine needed imports
        // plan 443: only BOUND model channels compile to defineModel (a
        // macro — no import). Unbound model vars emit `ref(...)` again, so
        // they imply the `ref` import. The other ref() emitters are:
        // template refs, the tick timer, and the previewcard/codeblock
        // copy state.
        let needs_ref = widget
            .state_vars
            .iter()
            .any(|s| !self.bound_model_channels.contains(&s.name))
            || !self.template_refs.is_empty()
            || widget.tick_interval.is_some()
            || !self.previewcard_data.is_empty()
            || !self.codeblock_data.is_empty();
        let needs_computed = !widget.computed.is_empty();

        // Generate Vue import statement
        let mut imports = Vec::new();
        if needs_ref {
            imports.push("ref");
        }
        // plan 015: defineModel is a <script setup> COMPILER MACRO — like
        // defineProps/defineEmits it must NOT be imported. Importing it
        // collides with Volar's injected macro shim (TS2440 "Import
        // declaration conflicts with local declaration"), breaking vue-tsc
        // in real projects (autodown editor + jade front gates).
        if needs_computed {
            imports.push("computed");
        }
        // Plan 106: Add watch, nextTick, onMounted for Prism.js re-highlighting
        if !self.previewcard_data.is_empty() {
            imports.push("watch");
            imports.push("nextTick");
            imports.push("onMounted");
        }
        // Plan 053 后续: scroll auto_scroll needs watch + nextTick for the
        // generated scroll-to-bottom watcher (ref is pulled in by template_refs).
        if !self.scroll_auto_scroll.is_empty() {
            if !imports.contains(&"watch") {
                imports.push("watch");
            }
            if !imports.contains(&"nextTick") {
                imports.push("nextTick");
            }
        }
        // Plan 563: canvas specs need watch + nextTick + onMounted for the
        // redraw effect (deep watch on the stroke tables); document-level
        // move/up listeners bring onUnmounted along (global listener pair).
        if !self.canvas_specs.is_empty() {
            if !imports.contains(&"watch") {
                imports.push("watch");
            }
            if !imports.contains(&"nextTick") {
                imports.push("nextTick");
            }
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        // PLAN-617 T-07: 受控 video 的下行同步用 watchEffect（读下行表达式即
        // 登记依赖，模板 ref 的挂载赋值也在其依赖里），外层包 onMounted 以
        // 避开 setup 期的 store facade TDZ（见 video_script_block 注释）。
        if !self.video_specs.is_empty() {
            if !imports.contains(&"watchEffect") {
                imports.push("watchEffect");
            }
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        // Plan 051 C7: timer 块需要 onMounted + onUnmounted（发射点晚于
        // import 语句生成，须在此声明需求）。
        if !widget.timers.is_empty() {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        // Timer/tick mechanism needs onMounted + onUnmounted
        if widget.tick_interval.is_some() {
            // The tick timer (`const tickTimer = ref<...>(null)`) always uses
            // `ref`, but `interval` is stripped from state_vars (extract.rs),
            // so `needs_ref` above can be false when interval is the only var.
            // Ensure `ref` is imported whenever the timer code is emitted.
            if !imports.contains(&"ref") {
                imports.push("ref");
            }
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            // PLAN-018 F-1 修复:守卫补齐——timer 块臂(上方)已推过
            // onUnmounted 时(every_ms 计时器两臂同触),无守卫重推产出
            // `import { …, onUnmounted, onUnmounted }`(vue/compiler-sfc
            // 重复声明,App.vue 编译炸;auto-term 实测)。
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
            // If there's a 'running' state var, timer is gated by watch()
            let has_running = widget.state_vars.iter().any(|s| s.name == "running");
            // If elapsed + time_display/ms_display exist, watch formats the display
            let has_elapsed = widget.state_vars.iter().any(|s| s.name == "elapsed");
            let has_time_display = widget.state_vars.iter().any(|s| s.name == "time_display");
            let has_ms_display = widget.state_vars.iter().any(|s| s.name == "ms_display");
            if has_running || (has_elapsed && (has_time_display || has_ms_display)) {
                imports.push("watch");
            }
        }
        // Dark mode: needs onMounted for system preference detection
        if self.has_dark_mode {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        // Plan 409 §8: accent bootstrap needs onMounted (applyAccent restore).
        if self.has_accent_color && !imports.contains(&"onMounted") {
            imports.push("onMounted");
        }
        // Plan 458: theme flips must re-apply the accent (applyAccent writes
        // the .dark-scoped --primary too, which otherwise falls back to the
        // stylesheet's indigo).
        if (self.has_accent_color || self.has_theme_name) && self.has_dark_mode && !imports.contains(&"watch") {
            imports.push("watch");
        }
        // Lifecycle: .Init → onMounted, .Destroy → onUnmounted
        let has_init = widget.lifecycle.iter().any(|l| l.name == "Init");
        let has_destroy = widget.lifecycle.iter().any(|l| l.name == "Destroy");
        if has_init {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            // PLAN-684 ADR-19 Vue 对等：带 props 的组件 Init 在 props 变更时
            // 重放（VM 轨每渲染帧补发 Init；Vue 轨以 watch(props, deep) 对齐）。
            // 否则父级 data 重播种（如 024 流式）后几何不重算，图不更新。
            if !widget.props.is_empty() && !imports.contains(&"watch") {
                imports.push("watch");
            }
        }
        // Auto-edit onMounted for sub-widgets with editing state + note prop
        let _has_editing = self.state_names.iter().any(|n| n == "editing");
        let _has_note_prop = self.prop_names.iter().any(|n| n == "note");
        if _has_editing && _has_note_prop {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
        }
        if has_destroy {
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        // Widget-level `watch { ... }` block → Vue watch() calls
        if !widget.watchers.is_empty() && !imports.contains(&"watch") {
            imports.push("watch");
        }
        // Global (window/document-level) listeners are registered in onMounted
        // and removed in onUnmounted.
        if !self.global_listeners.is_empty() {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        // Plan 451 P2: `actions {}`-declared shortcuts register a window
        // keydown fallback layer (mounted/unmounted, same lifecycle).
        if !self.actions_key_bindings.is_empty() || !self.bind_key_bindings.is_empty() {
            if !imports.contains(&"onMounted") {
                imports.push("onMounted");
            }
            if !imports.contains(&"onUnmounted") {
                imports.push("onUnmounted");
            }
        }
        if !imports.is_empty() {
            script.push_str(&format!("import {{ {} }} from 'vue'\n", imports.join(", ")));
        }
        // Plan 106: Add Prism import for syntax highlighting
        if !self.previewcard_data.is_empty() {
            script.push_str("import Prism from 'prismjs'\n");
        }

        // Plan 105: Add router import if needed
        if self.needs_router {
            script.push_str("import { useRouter } from 'vue-router'\n");
            script.push_str("const router = useRouter()\n\n");
        }
        // Plan 235: Add useRoute import if needed
        if self.needs_route {
            script.push_str("import { useRoute } from 'vue-router'\n");
            script.push_str("const route = useRoute()\n\n");
        }

        // Generate shadcn-vue imports (if any components were used in template)
        let shadcn_imports = self.generate_shadcn_imports();
        if !shadcn_imports.is_empty() {
            script.push_str(&shadcn_imports);
            script.push('\n');
        }

        // Generate lucide-vue-next imports (if any icons were used)
        if !self.lucide_icons.is_empty() {
            let mut icons: Vec<String> = self.lucide_icons.iter().cloned().collect();
            icons.sort();
            script.push_str(&format!(
                "import {{ {} }} from 'lucide-vue-next'\n",
                icons.join(", ")
            ));
            script.push('\n');
        }

        // Widget `use { ... }` external imports (hand-written TS/Vue escape
        // hatch). Component imports already provided by the built-in widget
        // registry are dropped to avoid duplicate symbol bindings.
        if !self.ext_import_lines.is_empty() {
            let mut lines: Vec<String> = self
                .ext_import_lines
                .iter()
                .filter(|(names, is_component, _)| {
                    !*is_component
                        || !names
                            .iter()
                            .any(|n| self.shadcn_components_used.contains(n))
                })
                .map(|(_, _, line)| line.clone())
                .collect();
            lines.sort();
            lines.dedup();
            for line in &lines {
                script.push_str(line);
            }
            script.push('\n');
        }

        // Widget `use { composable: ... }` — call each composable once at
        // <script setup> top level, binding the return value to a local
        // const (`useMenuBounds` → `menuBounds`) reachable from handlers.
        for (local, callee, args) in &self.ext_composables {
            // Plan 022: 空 args 生成 useX()（向后兼容无参 store composable），
            // 有 args 生成 useX(arg1, arg2)（支持有参 composable 如 useStreamingDocument(.source)）。
            script.push_str(&format!("const {} = {}({})\n", local, callee, args));
        }
        if !self.ext_composables.is_empty() {
            script.push('\n');
        }

        // Plan 426: setup preamble — statements at <script setup> top level,
        // BEFORE state/computed definitions (bindings first). Bindings are
        // registered as facade locals; refs-annotated field access gains
        // `.value`; await was rejected at parse time (MVP).
        if let Some(setup) = &self.current_setup_stmts {
            let ctx = self.handler_ts_ctx();
            // Plan 426 §1.2: setup 的 let 绑定 = script 顶层 **const**(组件
            // 局部绑定,不可重赋值);逐语句转译,首行 let → const(`var`
            // 绑定保持 let 可变)。语句输出必以 "let " 开头,replacen(1) 安全。
            let mut emitted: Vec<String> = Vec::new();
            for stmt in setup {
                let body = crate::ui_gen::ts_adapter::transpile_handler_body(
                    std::slice::from_ref(stmt),
                    &ctx,
                );
                let body = match stmt {
                    auto_lang_core::ast::Stmt::Store(store)
                        if matches!(store.kind, auto_lang_core::ast::StoreKind::Let) =>
                    {
                        body.replacen("let ", "const ", 1)
                    }
                    _ => body,
                };
                emitted.push(body);
            }
            self.drain_ctx_warnings(&ctx);
            for chunk in emitted {
                for line in chunk.lines() {
                    script.push_str(line);
                    script.push('\n');
                }
            }
            script.push('\n');
        }

        // Import ThemeToggle custom component if used
        if self.use_theme_toggle {
            script.push_str("import ThemeToggle from '@/components/ThemeToggle.vue'\n");
        }

        // Plan 234: Import custom PascalCase components referenced in template
        // (e.g. A2UIRenderer and other embedded Vue components)
        let mut custom_imports = Vec::new();
        // Names already bound by a widget `use { component:/fn:/composable:
        // X from ... }` declaration. The ext declaration is explicit user
        // intent (it may wrap/rename the real component), so the auto
        // `@/components/X.vue` import must yield — emitting both binds the
        // same identifier twice (TS2300).
        let ext_imported_names: HashSet<&str> = self
            .ext_import_lines
            .iter()
            .flat_map(|(names, _, _)| names.iter().map(String::as_str))
            .collect();
        for comp in &self.component_refs {
            if *comp == "ThemeToggle" {
                continue; // Already handled above
            }
            // Skip shadcn components (already imported via generate_shadcn_imports)
            if self.shadcn_components_used.contains(comp) {
                continue;
            }
            // Skip names already imported via an ext `use { ... }` declaration.
            if ext_imported_names.contains(comp.as_str()) {
                continue;
            }
            custom_imports.push(format!(
                "import {} from '@/components/{}.vue'\n",
                comp, comp
            ));
        }
        if !custom_imports.is_empty() {
            custom_imports.sort();
            custom_imports.dedup();
            for imp in &custom_imports {
                script.push_str(imp);
            }
        }
        if self.use_theme_toggle || !custom_imports.is_empty() {
            script.push('\n');
        }

        // Plan 132: Scan handlers for API function calls
        if !self.explicit_api_imports {
            // Legacy mode: scan AST to discover API calls
            for (_pattern, payload) in &widget.handlers {
                self.extract_api_calls_from_payload(payload);
            }
            // Also scan lifecycle events (.Init, .Destroy) for API calls
            for lc in &widget.lifecycle {
                self.extract_api_calls_from_payload(&lc.payload);
            }
        } else {
            // Explicit import mode: collect which declared imports are actually used
            for (_pattern, payload) in &widget.handlers {
                self.extract_api_calls_from_payload(payload);
            }
            for lc in &widget.lifecycle {
                self.extract_api_calls_from_payload(&lc.payload);
            }
        }
        // PLAN-074: watch 块体的 API 调用与 handlers/lifecycle 同权注册——
        // 缺此扫描时 watch 内的契约调用（`use back.api:` → '@/lib/api'
        // import 行）有 emission 无 import（TS2304；jade backlinks_panel
        // 下沉首件实证：on 块调用绿、watch 块调用 TS2304）。
        for w in &widget.watchers {
            self.extract_api_calls_from_payload(&w.payload);
        }

        // Plan 053 M5/P5-6: identify debounced complete-handlers (body calls
        // `complete`). Their function bodies get wrapped in setTimeout(80ms)
        // with a sequence guard so rapid typing fires one backend request per
        // pause, and a stale (slower) result can't clobber the newest one.
        for (pattern, payload) in &widget.handlers {
            if let LogicPayload::AstStmts(stmts) = payload {
                if Self::stmts_call_complete(stmts) {
                    self.debounced_handlers
                        .insert(self.pattern_to_handler_name(pattern));
                }
                // Plan 410: detect toast() calls so we import { toast } from vue-sonner.
                if Self::stmts_call_toast(stmts) {
                    self.needs_toast_import = true;
                }
            }
        }

        // Plan 410: import toast() from vue-sonner when a handler calls it.
        // Plan 412 续:vue-sonner v2 不再运行时注入样式,必须显式导入
        // style.css —— 缺它 [data-sonner-toaster] 是 static 定位,toast 会
        // 内联参与布局挤压页面而不是 fixed 悬浮在视口角落。
        if self.needs_toast_import {
            script.push_str("import { toast } from 'vue-sonner'\n");
            script.push_str("import 'vue-sonner/style.css'\n");
        }

        // Plan 132: Add API imports if needed (Plan 435 P6-1: HashSet
        // 迭代序不确定,import 列表与 deprecation warning 都必须排序发射)
        if !self.api_functions_used.is_empty() {
            let mut api_funcs: Vec<String> = self.api_functions_used.iter().cloned().collect();
            api_funcs.sort();
            script.push_str(&format!(
                "import {{ {} }} from '@/lib/api'\n",
                api_funcs.join(", ")
            ));
            // Deprecation warning for implicit API usage
            if !self.explicit_api_imports {
                eprintln!(
                    "  warning: Widget '{}' uses API functions [{}] without explicit import. Add `use back.api: {}` at the top of the file.",
                    self.current_widget.as_deref().unwrap_or("unknown"),
                    api_funcs.join(", "),
                    api_funcs.join(", "),
                );
            }
        }
        script.push('\n');

        // defineProps FIRST: state var initializers can read props
        // (`model { var draft str = .code }` → `ref(props.code)`) and are
        // EAGER — emitted before defineProps they hit the TDZ
        // (ReferenceError at runtime; found live in autodown plan-023's
        // CodeEditorBlock). Computeds are lazy so they never tripped it.
        // (auto-down debt #1, 2026-08-29)

        // Plan 367 P1-1: custom type names that need importing from api.ts.
        // Collected from prop types (recursively) AND from defineEmits
        // payloads below; the import statement itself is emitted after both
        // blocks, later in this function.
        let mut custom_types: Vec<String> = Vec::new();

        // Generate defineProps if widget has props (sub-widget component)
        // Plan 028 F6: props with declared defaults wrap in
        // `withDefaults(defineProps, {...})` so the default VALUES apply at
        // runtime, not just `?:` optionality.
        if !widget.props.is_empty() {
            let has_defaults = widget
                .props
                .iter()
                .any(|p| p.default.is_some() && !Self::prop_is_emitted_callback(p, widget));
            script.push_str(if has_defaults {
                "const props = withDefaults(defineProps<{\n"
            } else {
                "const props = defineProps<{\n"
            });
            for prop in &widget.props {
                // Plan 367 P1-1: map Auto types to TS types instead of using 'any'.
                // Plan 043 M5 B-1: `on_*: msg` callback props are typed from the
                // msg variant payload — `on_pick` for `Pick(str)` yields
                // `(arg0: string) => void`, not `() => void` (which rejected the
                // parent's `(name: any) => void` handler with TS2322).
                // Plan 043 M5 R4: skip `on_*` callback props with a matching
                // msg variant entirely — the parent wires them via the emit
                // (`@Run`), so a required `on_run` prop would make the parent's
                // object literal miss it (TS2345).
                if Self::prop_is_emitted_callback(prop, widget) {
                    continue;
                }
                let ts_type = Self::prop_to_ts_type(prop, widget);
                // Track custom types for import generation. Recurse into
                // containers (List<Block>, []ToolEntry, Option<T>, ...) so a
                // nested custom type still triggers `import type { ... }`.
                Self::collect_custom_types(&prop.type_info, &mut custom_types);
                if prop.default.is_some() {
                    script.push_str(&format!("  {}?: {}\n", prop.name, ts_type));
                } else {
                    script.push_str(&format!("  {}: {}\n", prop.name, ts_type));
                }
            }
            if has_defaults {
                // The macro form is `withDefaults(defineProps<{...}>(), {...})`
                // — the `()` after the type argument is required: without it
                // `defineProps<{...}>` parses as an instantiation expression
                // and vue-tsc no longer recognizes the macro (props collapse
                // to `unknown`, TS2344/TS2339 all over the SFC).
                script.push_str("}>(), {\n");
                for prop in &widget.props {
                    if Self::prop_is_emitted_callback(prop, widget) {
                        continue;
                    }
                    if let Some(default_expr) = &prop.default {
                        let default_js = self.expr_to_js(default_expr)?;
                        // withDefaults' InferDefault rejects a `null` default
                        // for non-nullable prop types (e.g. `any[]`). The DSL
                        // uses `= null` as "optional, unset" for callback/array
                        // props; cast so the declared prop type stays intact.
                        let ts_type = Self::prop_to_ts_type(prop, widget);
                        let default_js = if default_js.trim() == "null"
                            && ts_type != "any"
                            && !ts_type.contains("null")
                        {
                            format!("({default_js} as any)")
                        } else if default_js.trim().starts_with('[') {
                            format!("() => {}", default_js.trim())
                        } else if default_js.trim().starts_with('{') {
                            format!("() => ({})", default_js.trim())
                        } else {
                            default_js
                        };
                        script.push_str(&format!("  {}: {},\n", prop.name, default_js));
                    }
                }
                script.push_str("})\n\n");
            } else {
                script.push_str("}>()\n\n");
            }
        }

        // Generate state variables: bound model channels compile to
        // defineModel; everything else stays a plain ref.
        for state in &widget.state_vars {
            if !self.state_names.contains(&state.name) {
                self.state_names.push(state.name.clone());
            }
            let init = self.expr_to_js(&state.initial)?;

            if self.bound_model_channels.contains(&state.name) {
                // PLAN-037 T4 (plan 443 narrowing): a channel some parent
                // call-site actually binds (`v-model:name`) compiles to
                // `defineModel<T>('name', { default })`. Object/array
                // LITERAL defaults must be factory-wrapped — defineModel
                // defaults flow through Vue's props default machinery,
                // where a literal `{}`/`[]` is shared across every
                // component instance.
                let default_expr = match &state.initial {
                    auto_lang_core::ast::Expr::Object(_) | auto_lang_core::ast::Expr::Array(_) => {
                        format!("() => ({})", init)
                    }
                    _ => init.clone(),
                };
                if self.use_typescript {
                    let ts_type = self.expr_to_ts_type(&state.initial);
                    script.push_str(&format!(
                        "const {} = defineModel<{}>(\"{}\", {{ default: {} }})\n",
                        state.name, ts_type, state.name, default_expr
                    ));
                } else {
                    script.push_str(&format!(
                        "const {} = defineModel(\"{}\", {{ default: {} }})\n",
                        state.name, state.name, default_expr
                    ));
                }
            } else {
                // Plan 443: unbound model var = local state, pre-T4 shape.
                // NOT defineModel — its unbound get returns the raw local
                // value (useModel customRef), so deep mutations
                // (`doc.value.shapes.push`) would lose reactivity.
                if self.use_typescript {
                    let ts_type = self.expr_to_ts_type(&state.initial);
                    script.push_str(&format!(
                        "const {} = ref<{}>({})\n",
                        state.name, ts_type, init
                    ));
                } else {
                    script.push_str(&format!("const {} = ref({})\n", state.name, init));
                }
            }
        }

        // Plan 458: seed DECLARED dark_mode / accent_color refs from the
        // `auto run` env defaults (emitted as window globals by the
        // theme-aware index.html bootstrap; absent when the env vars are
        // unset, so a widget's own initial value stands). Mirrors the VM
        // seeding in run_dynamic_iced.
        if self.has_dark_mode && self.dark_mode_on_widget {
            let var = self
                .dark_mode_var
                .clone()
                .unwrap_or_else(|| "dark_mode".to_string());
            script.push_str(&format!(
                "// Plan 458: seed theme default from index.html bootstrap.\nif ((window as any).__AUTO_UI_THEME__ === 'light' || (window as any).__AUTO_UI_THEME__ === 'dark') {var}.value = (window as any).__AUTO_UI_THEME__ === 'dark'\n",
                var = var
            ));
        }
        if self.has_accent_color {
            script.push_str("// Plan 458: seed accent default from index.html bootstrap.\nif (typeof (window as any).__AUTO_UI_ACCENT__ === 'string') accent_color.value = (window as any).__AUTO_UI_ACCENT__\n");
        }

        if !widget.state_vars.is_empty() {
            script.push('\n');
        }

        // Plan 053 M5/P5-6: debounce infrastructure for complete-handlers.
        // `let` (not const) — clearTimeout() and ++__completeSeq mutate them.
        if !self.debounced_handlers.is_empty() {
            if self.use_typescript {
                script
                    .push_str("let __completeTimer: ReturnType<typeof setTimeout> | null = null\n");
            } else {
                script.push_str("let __completeTimer = null\n");
            }
            script.push_str("let __completeSeq = 0\n");
            script.push('\n');
        }

        // Template refs (`ref: "menuEl"` in the view) → Vue template-ref
        // declarations. Handlers access them via `.menuEl` → `menuEl.value!`.
        // Refs attached to child components are typed `any` (their exposed
        // surface is unknown to the parent).
        for ref_name in &self.template_refs {
            if self.component_ref_names.contains(ref_name) {
                script.push_str(&format!("const {} = ref<any>(null)\n", ref_name));
            } else if ref_name.starts_with("__canvas_ref_") {
                // Plan 563: canvas ref 按真元素类型声明(width/height 属性
                // 在 HTMLElement 上不存在,vue-tsc 门禁)。
                script.push_str(&format!(
                    "const {} = ref<HTMLCanvasElement | null>(null)\n",
                    ref_name
                ));
            } else if ref_name.starts_with("__video_ref_") {
                // PLAN-617 T-07: 受控 video 的 ref 同理——`currentTime`/
                // `playbackRate`/`readyState` 都不在 HTMLElement 上。
                script.push_str(&format!(
                    "const {} = ref<HTMLVideoElement | null>(null)\n",
                    ref_name
                ));
            } else {
                script.push_str(&format!(
                    "const {} = ref<HTMLElement | null>(null)\n",
                    ref_name
                ));
            }
        }
        if !self.template_refs.is_empty() {
            script.push('\n');
        }

        // Plan 547: ImageSurface onpan 合成包装(pointer 三绑定的函数体;
        // buttons&1 门控 + start/move/end 相位,handler 收 (dx, dy, phase))。
        if !self.surface_pans.is_empty() {
            for (i, (call, with_args)) in self.surface_pans.iter().enumerate() {
                // 按事件声明参数个数决定调用形态:空声明 -> 无参(vue-tsc
                // 严格匹配);声明了参数 -> (dx, dy, phase) 全参。
                let (start_call, move_call, end_call) = if *with_args {
                    (
                        format!("{call}(0, 0, 'start')"),
                        format!("{call}(dx, dy, 'move')"),
                        format!("{call}(0, 0, 'end')"),
                    )
                } else {
                    (
                        format!("{call}()"),
                        format!("{call}()"),
                        format!("{call}()"),
                    )
                };
                script.push_str(&format!(
                    concat!(
                        "let __surfPanLast_{i}: [number, number] | null = null\n",
                        "function __surfPanD_{i}(e: PointerEvent) {{\n",
                        "  __surfPanLast_{i} = [e.clientX, e.clientY]\n",
                        "  {start_call}\n",
                        "}}\n",
                        "function __surfPanM_{i}(e: PointerEvent) {{\n",
                        "  if (!__surfPanLast_{i} || !(e.buttons & 1)) return\n",
                        "  const dx = e.clientX - __surfPanLast_{i}[0], dy = e.clientY - __surfPanLast_{i}[1]\n",
                        "  __surfPanLast_{i} = [e.clientX, e.clientY]\n",
                        "  {move_call}\n",
                        "}}\n",
                        "function __surfPanU_{i}(_e: PointerEvent) {{\n",
                        "  if (!__surfPanLast_{i}) return\n",
                        "  __surfPanLast_{i} = null\n",
                        "  {end_call}\n",
                        "}}\n"
                    ),
                    i = i,
                    start_call = start_call,
                    move_call = move_call,
                    end_call = end_call
                ));
            }
            script.push('\n');
        }

        // Plan 563: canvas —— redraw + pen 包装 + watch deep(T1b 复用
        // 先例)。clone 后逐块生成(canvas_script_block 内 push
        // global_listeners,与 self.canvas_specs 迭代解耦)。
        if !self.canvas_specs.is_empty() {
            let specs = self.canvas_specs.clone();
            for (i, spec) in specs.iter().enumerate() {
                script.push_str(&self.canvas_script_block(i, spec));
                script.push('\n');
            }
        }

        // PLAN-617 T-07: 受控 video —— 下行 apply + watchEffect 同步 + 上行
        // 原生监听包装(§2.3 受控媒体契约的 Vue 侧实现)。
        if !self.video_specs.is_empty() {
            let specs = self.video_specs.clone();
            for (i, spec) in specs.iter().enumerate() {
                script.push_str(&self.video_script_block(i, spec));
                script.push('\n');
            }
        }

        // PLAN-617 后续：可拖拽进度条的三包装（按下/拖动/抬起）。
        if !self.progress_seek_specs.is_empty() {
            let specs = self.progress_seek_specs.clone();
            for (i, handler) in specs.iter().enumerate() {
                script.push_str(&self.progress_seek_script_block(i, handler));
                script.push('\n');
            }
        }

        // Dark mode: detect system preference on mount (only for isDark pattern)
        if self.has_dark_mode && self.dark_mode_var.as_deref() != Some("dark_mode") {
            script.push_str("onMounted(() => {\n");
            script.push_str(
                "  isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches\n",
            );
            script.push_str("})\n\n");
        }

        // Generate computed properties
        for computed_prop in &widget.computed {
            let expr_js = self.expr_to_js(&computed_prop.expr)?;

            // Plan 100: Add type annotation for TypeScript
            if self.use_typescript {
                let ts_type = self.expr_to_ts_type(&computed_prop.expr);
                script.push_str(&format!(
                    "const {} = computed<{}>(() => {})\n",
                    computed_prop.name, ts_type, expr_js
                ));
            } else {
                script.push_str(&format!(
                    "const {} = computed(() => {})\n",
                    computed_prop.name, expr_js
                ));
            }
        }

        if !widget.computed.is_empty() {
            script.push('\n');
        }

        // Widget-level `expose { ... }`: exposed `on` handlers must be
        // treated as used even when the template never references them — the
        // parent calls them through a template ref (defineExpose below).
        // Marked before defineEmits so their emit() payloads are declared.
        //
        // Plan 012 Batch A (gap 45): match by BASE pattern, not exact key.
        // Parameterized handlers are keyed ".Open(entry)" (Plan 374) and
        // quoted emit names sanitize to different fn names — an exact-key
        // lookup missed them, so the handler was never generated and
        // `defineExpose({ Open })` silently resolved to a GLOBAL at runtime
        // (`window.open`!), with vue-tsc passing clean.
        for name in &widget.exposes {
            let trimmed = name.trim_matches('"').trim_start_matches('.');
            for pattern in widget.handlers.keys() {
                let fn_name = self.pattern_to_handler_name(pattern);
                let base = Self::base_pattern(pattern)
                    .trim_start_matches('.')
                    .trim_matches('"');
                if base == trimmed || fn_name == trimmed || fn_name == Self::sanitize_ident(trimmed)
                {
                    self.used_handlers.insert(fn_name);
                }
            }
        }

        // Generate emit if needed
        if self.has_emit {
            // Build event → payload type map from msg declarations.
            // Each variant may carry a single payload type (e.g. SelectTag(str) → str).
            // handler_params stores param names (e.g. ["t"]) for the on-block's .Handler(t).
            // We use the variant's payload type when available.
            let mut event_payload_types: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for msg in &widget.messages {
                for variant in &msg.variants {
                    // Plan 043 M5 #1: payload is now Vec<Type>; the TS event
                    // path carries a single payload type, so use the first
                    // (multi-param variants are a Rust-backend feature for now).
                    if !variant.payload.is_empty() {
                        // Plan 015 P1#6: carry ALL payload types. The old
                        // first()-only truncation made "Resize(w, h)" declare
                        // [number] while the emit call forwarded both args —
                        // a latent TS mismatch.
                        // Plan 444 (ash-shell-057 ③): the old handler-params
                        // gate is dropped — a variant's payload IS the emit
                        // contract. Template-referenced handlers without an
                        // on-block entry (Sort(int,int)/Filter(str)) get
                        // synthesized-arg emit bridges now, so the emit()
                        // call always forwards the declared arity; gating on
                        // declared params produced `Sort: []` + TS2554 at
                        // the call site.
                        {
                            let types: Vec<String> = variant
                                .payload
                                .iter()
                                .map(|ty| {
                                    Self::collect_custom_types(ty, &mut custom_types);
                                    Self::auto_type_to_ts_type(ty)
                                })
                                .collect();
                            event_payload_types.insert(variant.name.clone(), types.join(", "));
                        }
                    }
                }
            }
            // Fallback: the emit() call passes handler params (see below), so a
            // handler with declared params but no msg payload type must still
            // declare matching payload arity — otherwise vue-tsc reports
            // "Expected 1 arguments, but got 2" on the emit('X', arg) call.
            for (pattern, params) in &widget.handler_params {
                if params.is_empty() {
                    continue;
                }
                let name = Self::base_pattern(pattern).trim_start_matches('.');
                if self.emit_events.contains(&name.to_string())
                    && !event_payload_types.contains_key(name)
                {
                    let any_params: Vec<&str> = params.iter().map(|_| "any").collect();
                    event_payload_types.insert(name.to_string(), any_params.join(", "));
                }
            }

            // PLAN-668 R-23 附（017 随检）：回调契约 emit 的载荷元数取自调用位
            // `on_x(.y)`——契约事件（on_xxx: msg prop → Pascal emit）既无
            // variant 载荷也无 handler 参数可循（017 sidebar 的
            // on_search(.local_query)），缺省 `[]` 与 emit('X', arg) 调用
            // 位及其父绑定 handler 签名均不符（TS2345/TS2322）。
            {
                fn count_cb_arity(
                    stmts: &[auto_lang_core::ast::Stmt],
                    out: &mut std::collections::HashMap<String, usize>,
                ) {
                    use auto_lang_core::ast::{Arg, Expr, Stmt};
                    for stmt in stmts {
                        let Stmt::Expr(e) = stmt else { continue };
                        let Expr::Call(call) = e else { continue };
                        let Expr::Ident(name) = call.name.as_ref() else {
                            continue;
                        };
                        let Some(prop) = name.as_str().strip_prefix("on_") else {
                            continue;
                        };
                        let arity = call
                            .args
                            .args
                            .iter()
                            .filter(|a| matches!(a, Arg::Pos(_)))
                            .count();
                        out.entry(VueGenerator::snake_to_pascal(prop))
                            .and_modify(|n| *n = (*n).max(arity))
                            .or_insert(arity);
                    }
                }
                let mut arity: std::collections::HashMap<String, usize> = Default::default();
                for payload in widget.handlers.values() {
                    if let crate::ui_gen::vue::LogicPayload::AstStmts(stmts) = payload {
                        count_cb_arity(stmts, &mut arity);
                    }
                }
                for (event, n) in arity {
                    if n > 0
                        && self.emit_events.iter().any(|e| *e == event)
                        && !event_payload_types.contains_key(&event)
                    {
                        let anys = vec!["any"; n].join(", ");
                        event_payload_types.insert(event, anys);
                    }
                }
            }
            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                // Plan 448 C: auto-sync mint events never fire from the
                // template (their @input attr is suppressed; v-model carries
                // the sync) — declaring them is dead surface.
                if event.starts_with("__bind_") {
                    continue;
                }
                // Plan 367 P1-4: only declare events for handlers that are
                // actually used in the template. Unused handlers don't get
                // function definitions, so declaring their emit types is noise.
                // used_handlers holds sanitized fn names (emit names like
                // "update:modelValue" sanitize to update_modelValue).
                // QUOTED variants are contractual emit names (plan 013 Phase
                // 2): they must ALWAYS be declared — bridge/extension code
                // emits them via getCurrentInstance().emit with no handler,
                // and undeclared listeners would fall through as native DOM
                // listeners on the root element.
                // Plan 015 P1#4: declare every declared event — the used_handlers
                // gate dropped handlers that extensions emit/call directly,
                // and undeclared listeners fall through as native DOM events.
                let _ = (&self.quoted_events, &self.callback_contract_events);
                // TS object-literal keys must be quoted when the emit name is
                // not a plain identifier (e.g. 'update:modelValue').
                let key = if event
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                {
                    event.clone()
                } else {
                    format!("'{}'", event)
                };
                if let Some(ts_type) = event_payload_types.get(event) {
                    script.push_str(&format!("  {}: [{}]\n", key, ts_type));
                } else {
                    script.push_str(&format!("  {}: []\n", key));
                }
            }
            script.push_str("}>()\n\n");
        }

        // Import custom types from api.ts (type-only import) — after both
        // defineProps and defineEmits so prop types and emit payload types
        // are collected.
        if !custom_types.is_empty() {
            script.push_str(&format!(
                "import type {{ {} }} from '@/lib/api'\n\n",
                custom_types.join(", ")
            ));
        }

        // Plan 351: store composable imports + const store
        if !self.store_deps.is_empty() {
            for dep in &self.store_deps {
                // Plan 446 批三 G1: 导入前缀可配（默认 @/stores），对齐
                // 部署管线的实际生成位置（os-config: src/stores/auto/）。
                // PLAN-063 Phase B T15 (KD 061 D13): 命名经归一助手
                //(AuthStore → useAuthStore,无双后缀)。
                let comp = store_composable_name(dep);
                script.push_str(&format!(
                    "import {{ {comp} }} from '{}/{}'\n",
                    self.store_import_prefix, comp
                ));
            }
            // v1: single store → const store = reactive(useXxxStore())
            // reactive() auto-unwraps nested refs so templates can use store.notes directly
            let first = &self.store_deps[0];
            script.push_str(&format!("import {{ reactive }} from 'vue'\n"));
            script.push_str(&format!(
                "const store = reactive({}())\n\n",
                store_composable_name(first)
            ));
            // Plan 742 D8: index.html bootstrap 只播种 widget 本地 dark_mode
            // ref——store 模型若也声明 dark_mode，两者会分叉（App ref=light 而
            // store=true），Theme 快切只翻 store，watch 不触发，html.dark 永不变
            // （015-notes smoke T11 实证）。store 侧同源播种，保持单一事实源。
            if self.has_dark_mode {
                script.push_str(
                    "// Plan 458/742: seed store dark_mode from the same bootstrap.
if ((window as any).__AUTO_UI_THEME__ === 'light' || (window as any).__AUTO_UI_THEME__ === 'dark') store.dark_mode = (window as any).__AUTO_UI_THEME__ === 'dark'
",
                );
            }
            // PLAN-048 (auto-musk A 线): 跨 store 依赖(deps[1..])各发独立
            // facade(如 ForgeStore → forgeStore),配套 ts_adapter 的 Ident
            // 映射——此前第二个及以后的 store 调用裸发名字(TS2304)。
            for dep in self.store_deps.iter().skip(1) {
                let var = facade_var_for(dep);
                script.push_str(&format!(
                    "const {var} = reactive({}())\n\n",
                    store_composable_name(dep)
                ));
            }
        }

        // Widget-level `watch { ... }` block → Vue watch() calls.
        // Emitted after state/computed/defineProps so the watched refs are
        // already initialized (watch() runs immediately at setup time).
        // Source resolution: model fields and computed are refs (watched
        // directly); props need a getter (`() => props.x`).
        for watcher in &widget.watchers {
            let body = self.generate_handler_body(&watcher.payload)?;
            let sources: Vec<String> = watcher
                .sources
                .iter()
                .map(|s| {
                    if self.prop_names.contains(s) {
                        format!("() => props.{}", s)
                    } else {
                        s.clone()
                    }
                })
                .collect();
            let source = if sources.len() == 1 {
                sources[0].clone()
            } else {
                format!("[{}]", sources.join(", "))
            };
            let opts = match (watcher.immediate, watcher.deep) {
                (false, false) => String::new(),
                (immediate, deep) => {
                    let mut parts = Vec::new();
                    if immediate {
                        parts.push("immediate: true");
                    }
                    if deep {
                        parts.push("deep: true");
                    }
                    format!(", {{ {} }}", parts.join(", "))
                }
            };
            let indented = Self::indent_body(&body, "  ");
            // Plan 408 P12 §10.7: watch 回调含 await/API 调用时用 async。
            let async_kw = if self.handler_has_api_calls(&watcher.payload) {
                "async "
            } else {
                ""
            };
            script.push_str(&format!(
                "watch({}, {}() => {{\n{}\n}}{})\n\n",
                source, async_kw, indented, opts
            ));
        }

        // Plan 053 后续: scroll auto_scroll — emit a scroll-to-bottom watcher for
        // each scroll node that opted in via `auto_scroll: "<prop>"`. deep:true
        // fires on push AND on streamed-text mutation, so the view stays pinned
        // to the bottom while command output streams in.
        for (src, ref_name) in &self.scroll_auto_scroll {
            script.push_str(&format!(
                "watch(() => props.{}, () => {{ nextTick(() => {{ {}.value?.scrollIntoView({{ block: 'end' }}) }}) }}, {{ deep: true }})\n\n",
                src, ref_name
            ));
        }

        // PLAN-656 T-07: scroll controller helper 族——.at 端 snake_case
        // 原生调用（scroll_to_end(scroll) 等）经 ts 转译蛇形名直通，此处
        // 注入同名 JS 函数接通（双端语义：与 VM natives 族一一对应；DOM
        // 寻址经 pane 的 data-scroll-ctl 锚）。
        // PLAN-701 供⑤b: strict（vue-tsc）下补 TS 类型注解——原裸形参数
        // 在 `strict: true` tsconfig 下 TS7006（隐式 any）×12 build 红
        // （m1-supply §14/§15 登记；auto-edit App.vue helper 行实测）。
        if self.scroll_ctl_helpers_needed {
            let sig = |name: &str, params: &str, ret: &str| -> String {
                if self.use_typescript {
                    format!("function {name}({params}): {ret}")
                } else {
                    format!("function {name}({params})")
                }
            };
            let (k, ax, a, b) = if self.use_typescript {
                ("k: string", "ax: string", "a: string | number", "b: number")
            } else {
                ("k", "ax", "a", "b")
            };
            let el_ret_t = if self.use_typescript {
                "Element | null"
            } else {
                ""
            };
            let state_ret_t = if self.use_typescript {
                "{ offset_x: number; offset_y: number; viewport_w: number; viewport_h: number; content_w: number; content_h: number; progress_x: number; progress_y: number }"
            } else {
                ""
            };
            script.push_str(&format!(
                r#"let __scrollCtlSeq = 0
{sc} {{ return '__scrollctl_' + (++__scrollCtlSeq) }}
// F-781-R02：属性比对寻址（弃 selector 拼接）——句柄含引号/反斜杠等
// 合法字符时 selector 拼接会异常或错配；全量扫描按 getAttribute 精确等值。
{sce} {{
  const els = document.querySelectorAll('[data-scroll-ctl]')
  for (const el of els) {{ if (el.getAttribute('data-scroll-ctl') === k) return el }}
  return null
}}
{sts} {{ const e = __scrollCtlEl(k); if (!e) return; if (ax === 'x') e.scrollTo({{ left: 0 }}); else e.scrollTo({{ top: 0 }}) }}
{ste} {{ const e = __scrollCtlEl(k); if (!e) return; if (ax === 'x') e.scrollTo({{ left: e.scrollWidth }}); else e.scrollTo({{ top: e.scrollHeight }}) }}
{sb} {{ const e = __scrollCtlEl(k); if (!e) return; if (typeof a === 'string') {{ if (a === 'x') e.scrollLeft += b; else e.scrollTop += b }} else {{ e.scrollTop += a }} }}
{st} {{ const e = __scrollCtlEl(k); if (!e) return; if (typeof a === 'string') {{ if (a === 'x') e.scrollLeft = b; else e.scrollTop = b }} else {{ e.scrollTo(a, b) }} }}
{ss} {{
  const e = __scrollCtlEl(k)
  if (!e) return {{ offset_x: 0, offset_y: 0, viewport_w: 0, viewport_h: 0, content_w: 0, content_h: 0, progress_x: 0, progress_y: 0 }}
  return {{
    offset_x: e.scrollLeft, offset_y: e.scrollTop,
    viewport_w: e.clientWidth, viewport_h: e.clientHeight,
    content_w: e.scrollWidth, content_h: e.scrollHeight,
    progress_x: Math.min(1, e.scrollLeft / Math.max(1, e.scrollWidth - e.clientWidth)),
    progress_y: Math.min(1, e.scrollTop / Math.max(1, e.scrollHeight - e.clientHeight)),
  }}
}}

"#,
                sc = sig("scroll_controller", "", "string"),
                sce = sig("__scrollCtlEl", k, el_ret_t),
                sts = sig("scroll_to_start", &format!("{k}, {ax}"), "void"),
                ste = sig("scroll_to_end", &format!("{k}, {ax}"), "void"),
                sb = sig("scroll_by", &format!("{k}, {a}, {b}"), "void"),
                st = sig("scroll_to", &format!("{k}, {a}, {b}"), "void"),
                ss = sig("scroll_state", k, state_ret_t),
            ));

            // PLAN-781 T-04: 子内容锚点族（9922/9923 的 vue 轨运行面）。
            // 语义与 controller.rs 注册表一一镜像：回执 queued / error:<reason>
            //（bad_handle/empty_key/unsupported——managed 桥经 pane 内
            // data-scroll-managed 标记识别）；状态查询 applied /
            // invalid:<reason> / unknown。Vue 轨 DOM 同步——回执返回时滚动
            // 已生效（.Init → onMounted 后 DOM 在场，无 iced 的布局 pending
            // 窗口；pane 缺席 = not_bound、key 缺席 = not_found，一次性不
            // 重试，与内容代际语义同形）。__sctlToChild = 决策报告 §3 冻结
            // 公式的视觉坐标形态（scrollIntoView 会级联滚动全部可滚祖先，
            // 违反 V-04 无跨 pane 滚动——r5 修订）；同 pane 重复 key 取文档
            // 序最后一个（iced 采集器 HashMap 后写覆盖同构）。
            let (k2, key2, s2) = if self.use_typescript {
                ("k: string", "key: string", "s: string")
            } else {
                ("k", "key", "s")
            };
            let (e2, t2) = if self.use_typescript {
                ("e: Element", "t: Element")
            } else {
                ("e", "t")
            };
            let store_t = if self.use_typescript {
                ": Record<string, string>"
            } else {
                ""
            };
            let maybe_t = if self.use_typescript {
                " as string | undefined"
            } else {
                ""
            };
            // F-781-R02（T-20）：归属谓词的 TS 注解（vue-tsc --noEmit 严格面）。
            let owns_t = if self.use_typescript {
                "const owns = (el: Element): boolean => el.closest('[data-scroll-pane]') === e"
            } else {
                "var owns = function (el) { return el.closest('[data-scroll-pane]') === e }"
            };
            script.push_str(&format!(
                r#"const __sctlAnchorStates{store_t} = {{}}
{an} {{ __sctlAnchorStates[k + '\u0000' + key] = s }}
{tc} {{
  const pr = e.getBoundingClientRect()
  const tr = t.getBoundingClientRect()
  const dv = tr.top - (pr.top + e.clientTop)
  const h = tr.height
  const vh = e.clientHeight
  let next = e.scrollTop
  if (h > vh || dv < 0) next = e.scrollTop + dv
  else if (dv + h > vh) next = e.scrollTop + dv + h - vh
  e.scrollTop = Math.min(Math.max(0, e.scrollHeight - e.clientHeight), Math.max(0, next))
}}
{stc} {{
  if (!k) return 'error:bad_handle'
  if (!key) return 'error:empty_key'
  const e = __scrollCtlEl(k)
  if (!e) {{ __sctlAnchorNote(k, key, 'invalid:not_bound'); return 'queued' }}
  // F-781-R02：最近 pane 归属过滤——候选目标与 managed 标记的最近
  // [data-scroll-pane] 所有者必须是本 pane（嵌套 pane——含无 controller
  // 者，其元素同样发射标记——的目标/标记不被外层认领）。属性比对替代
  // CSS selector 拼接：任意 key/句柄字符（引号/反斜杠/冒号/CJK）安全。
  {owns_t}
  const mgd = e.querySelectorAll('[data-scroll-managed]')
  for (const el of mgd) {{ if (owns(el)) return 'error:unsupported' }}
  let target = null
  const cands = e.querySelectorAll('[data-scroll-anchor]')
  for (const el of cands) {{
    if (el.getAttribute('data-scroll-anchor') === key && owns(el)) target = el
  }}
  if (!target) {{ __sctlAnchorNote(k, key, 'invalid:not_found'); return 'queued' }}
  __sctlToChild(e, target)
  __sctlAnchorNote(k, key, 'applied')
  return 'queued'
}}
{sas} {{ return __sctlAnchorState(k, key) }}
{ast} {{
  const s = __sctlAnchorStates[k + '\u0000' + key]{maybe_t}
  return s === undefined ? 'unknown' : s
}}

"#,
                an = sig("__sctlAnchorNote", &format!("{k2}, {key2}, {s2}"), "void"),
                tc = sig("__sctlToChild", &format!("{e2}, {t2}"), "void"),
                stc = sig("scroll_to_child", &format!("{k2}, {key2}"), "string"),
                sas = sig("scroll_anchor_state", &format!("{k2}, {key2}"), "string"),
                ast = sig("__sctlAnchorState", &format!("{k2}, {key2}"), "string"),
            ));
        }

        // PLAN-493: textarea mentions backdrop 的 HTML helper —— template
        // 发射期置位，随组件一次发射。escape + @\w+ 扫描（词小写 ∈ 名单）
        // + span 包裹 + 尾随 \n（换行几何对齐）；**无 display 替换**（对齐
        // VM 段计算契约：着色 @词原文，backdrop 文本与 value 逐字一致）。
        if self.mention_helper_needed {
            let fn_sig = if self.use_typescript {
                "(text: string, names: string[], cls: string): string"
            } else {
                "(text, names, cls)"
            };
            let is_word_def = if self.use_typescript {
                "const isWord = (ch: string) => /[A-Za-z0-9_]/.test(ch)"
            } else {
                "const isWord = (ch) => /[A-Za-z0-9_]/.test(ch)"
            };
            // PLAN-493 r9（musk-073 消费）：引用 token（@plan/001、@spec/<path>
            // 等，含 . / - 的非纯词 token）高亮——token 字符集扩展扫描 +
            // 名单最长匹配；命中含非 \w 字符者 span 附加 `mention-token`
            // 语义类（宿主全局样式差异化着色；类名零宽度，backdrop 与
            // textarea 逐字对齐不受影响）。纯 \w 词命中走原类串（Agent @词
            // 口径不变）。
            let is_token_char_def = if self.use_typescript {
                "const isTokenChar = (ch: string) => /[A-Za-z0-9_./-]/.test(ch)"
            } else {
                "const isTokenChar = (ch) => /[A-Za-z0-9_./-]/.test(ch)"
            };
            script.push_str("function __autoMentionHtml");
            script.push_str(fn_sig);
            script.push_str(
                r#" {
  if (!text) return '\n'
  const known = new Set((names ?? []).map((n) => String(n).toLowerCase()))
  const escaped = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  "#,
            );
            script.push_str(is_word_def);
            script.push_str("\n  ");
            script.push_str(is_token_char_def);
            script.push_str(
                r#"
  let out = ''
  let i = 0
  while (i < escaped.length) {
    if (escaped[i] === '@') {
      let j = i + 1
      while (j < escaped.length && isTokenChar(escaped[j])) { j += 1 }
      let hitEnd = 0
      for (let k = j; k > i + 1; k--) {
        if (known.has(escaped.slice(i + 1, k).toLowerCase())) { hitEnd = k; break }
      }
      if (hitEnd > 0) {
        const tok = escaped.slice(i + 1, hitEnd)
        const isRef = /[^A-Za-z0-9_]/.test(tok)
        out += `<span class="${cls}${isRef ? ' mention-token' : ''}">@${tok}</span>`
        i = hitEnd
        continue
      }
      let w = i + 1
      let word = ''
      while (w < escaped.length && isWord(escaped[w])) { word += escaped[w]; w += 1 }
      if (word !== '' && known.has(word.toLowerCase())) {
        out += `<span class="${cls}">@${word}</span>`
        i = w
      } else { out += '@'; i += 1 }
    } else { out += escaped[i]; i += 1 }
  }
  return out + '\n'
}

"#,
            );
        }

        // Generate event handlers
        // fn-name → on-block base pattern (e.g. update_modelValue →
        // ".update:modelValue"): fn names are sanitized JS identifiers while
        // handler_params is keyed by the verbatim pattern, so keep the mapping.
        let mut handler_base_patterns: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            handler_base_patterns.insert(
                handler_name.clone(),
                Self::base_pattern(pattern).to_string(),
            );
            // Plan 053 M5/P5-6: complete-handlers get a debounced body
            // (setTimeout 80ms + seq guard); other handlers transpile normally.
            let is_debounced = self.debounced_handlers.contains(&handler_name);
            let mut body = if is_debounced {
                self.generate_debounced_handler_body(payload)?
            } else {
                self.generate_handler_body(payload)?
            };
            // Plan 444 (ash-shell-057 ④): VM-only native rewrites need the
            // shared __vmOnly stub declared in this SFC.
            if body.contains("__vmOnly(") {
                self.needs_vm_only_helper = true;
            }
            // Auto-emit events for sub-widget handlers that match emit declarations.
            // Plan 367 P0-1: Skip emit if the handler already notifies the parent
            // via a callback prop (props.on_xxx()). In that case the emit is
            // redundant — the parent's callback was already invoked.
            // Emit names are verbatim msg variant names (e.g. "update:modelValue"
            // for a v-model contract); match against the handler's fn name, which
            // is the sanitized form.
            let emit_name = if self.has_emit {
                self.emit_events
                    .iter()
                    .find(|e| e.as_str() == handler_name || Self::sanitize_ident(e) == handler_name)
                    .cloned()
            } else {
                None
            };
            // Plan 448 B1: inline-lambda handlers (`onclick: () => { … }`)
            // are widget-internal sugar — no parent can listen for the
            // synthetic `__evt_*` event, so never append the trailing emit.
            // Plan 448 C: `__bind_*` auto-sync mints likewise (and their
            // handler fn is not emitted at all).
            let emit_name = emit_name.filter(|e| {
                !e.as_str().starts_with("__evt_") && !e.as_str().starts_with("__bind_")
            });
            // Plan musk-022 callback-relay fix: rewrite any `props.on_xxx(args)`
            // calls in the body to `emit('<Pascal>', args)` for real callback
            // props. The parent binds `@Pascal` (never `:on_xxx`), so the raw
            // `props.on_xxx()` would be undefined at runtime.
            // PLAN-037 T3: also rewrite EMITTED-callback props (dropped from
            // defineProps because a matching msg variant exists) — for those,
            // `props.on_xxx(` is definitionally dangling and the `Pascal`
            // emit is definitionally declared (the variant), so the rewrite
            // closes the gap between the two mechanisms (k2 canary).
            let mut callback_relayed = false;
            for cb_snake in Self::real_callback_prop_snakes(widget)
                .into_iter()
                .chain(Self::emitted_callback_prop_snakes(widget))
            {
                let props_call = format!("props.on_{}(", cb_snake);
                if body.contains(&props_call) {
                    let pascal = Self::snake_to_pascal(&cb_snake);
                    let emit_call = format!("emit('{}', ", pascal);
                    body = body.replace(&props_call, &emit_call);
                    callback_relayed = true;
                }
                // Plan 444 (ash-shell-057 ①a): the .at also allows BARE
                // callback invocations (`on_typing("You")` — 017-chat; the
                // VM routes them via DynamicMessage, handler_codegen strips
                // them). In the SFC the prop no longer exists under any
                // spelling, so rewrite the bare call to the Pascal emit too.
                // Runs after the props. pass, which already consumed the
                // `props.on_x(` spellings.
                let bare_call = format!("on_{}(", cb_snake);
                if body.contains(&bare_call) {
                    let pascal = Self::snake_to_pascal(&cb_snake);
                    let emit_call = format!("emit('{}', ", pascal);
                    body = body.replace(&bare_call, &emit_call);
                    callback_relayed = true;
                }
            }
            if let Some(emit_name) = emit_name {
                let snake = Self::pascal_to_snake(&handler_name);
                let callback_key = format!("props.on_{}", snake);
                let already_notifies_parent = callback_relayed || body.contains(&callback_key);
                // Plan 053 M4 (P5-7 residual): internal helper handlers that are
                // only self-called (`.DoContinuation()` from OnInput/OnEnter,
                // `.DoTokenize()` from OnInput) skip the trailing emit('X') —
                // no parent listens, and they are NOT in the defineEmits type
                // (that's built from template-bound handlers *before*
                // mark_self_called_handlers runs), so emitting would be a
                // TS2769 "not assignable to parameter" (observed for DoTokenize).
                // Template-bound handlers are already in used_handlers here.
                // QUOTED variants (contractual emit names, plan 013 Phase 2)
                // are always declared in defineEmits, so they always get the
                // trailing emit — even when the handler is only self-called
                // (the computed-payload relay pattern: an exposed/view-bound
                // handler computes the payload, then self-calls the quoted
                // handler to emit it).
                if !already_notifies_parent
                    && (self.used_handlers.contains(&handler_name)
                        || self.quoted_events.contains(&emit_name))
                {
                    // Plan 367 P1-4: pass handler params to emit() so the call
                    // matches the typed defineEmits declaration.
                    // Plan 043 M5 B-3: when the handler is a loop-param handler
                    // (template calls it as OpenPath(b) inside a v-for), the
                    // generated function signature is `function OpenPath(b: any)`
                    // — so the emit() must forward `b`, NOT the on-block's
                    // declared param name (`path`), which is never bound and
                    // would produce "Cannot find name" TS2304.
                    let pattern_key = Self::base_pattern(pattern);
                    let declared = Self::get_handler_params(&widget.handler_params, pattern_key);
                    let emit_args: String =
                        if let Some(loop_var) = self.loop_param_handlers.get(&handler_name) {
                            // Loop-param handlers (template calls OpenPath(b) inside
                            // a v-for) must emit the loop var, NOT the on-block's
                            // declared param name (`path`) which is never bound.
                            // BUT a no-arg handler (.Stop) also receives the loop
                            // var in its function signature — it must still emit()
                            // WITHOUT args, matching its `Stop: []` payload.
                            if declared.map(|p| !p.is_empty()).unwrap_or(false) {
                                loop_var.clone()
                            } else {
                                String::new()
                            }
                        } else {
                            // Plan 444 (ash-shell-057 ③): when the on-block
                            // declares no params but the msg variant carries a
                            // payload, forward synthesized arg names — the
                            // defineEmits type now always carries the variant's
                            // payload arity, so an arg-less emit would TS2554.
                            let declared_args = declared
                                .map(|params| {
                                    params
                                        .iter()
                                        .map(|p| p.as_str().to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .unwrap_or_default();
                            if declared_args.is_empty() {
                                let arity = self
                                    .msg_payload_arities
                                    .get(&handler_name)
                                    .cloned()
                                    .unwrap_or(0);
                                if arity > 0 {
                                    (0..arity)
                                        .map(|i| format!("arg{}", i))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                } else {
                                    String::new()
                                }
                            } else {
                                declared_args
                            }
                        };
                    if emit_args.is_empty() {
                        body.push_str(&format!("\nemit('{}')", emit_name));
                    } else {
                        body.push_str(&format!("\nemit('{}', {})", emit_name, emit_args));
                    }
                }
            }
            // Plan 409 §8: widget-path accent support — when the widget owns an
            // `accent_color` state, the SetAccent / ToggleDarkMode handlers must
            // also call applyAccent() so the --primary CSS variable tracks the
            // VM-side palette. Mirrors the store composable path (Plan 360).
            // isDark is resolved from the DOM (.dark class) rather than a state
            // var, since gallery apps hardcode `<html class="dark">`.
            if self.has_accent_color {
                if handler_name == "SetAccent" {
                    body.push_str("; applyAccent(accent_color.value, document.documentElement.classList.contains('dark'))");
                } else if handler_name == "ToggleDarkMode" {
                    body.push_str("; applyAccent(accent_color.value, document.documentElement.classList.contains('dark'))");
                }
            }
            // Plan 132: Check if handler contains API calls (needs async).
            // Plan 053 M5/P5-6: debounced handlers move async/await into the
            // setTimeout callback, so the function itself is synchronous.
            // Plan 617 T-06: `Http.get` transpiles to `await (await fetch(..)).json()`,
            // so a handler whose body awaits must be emitted async. Same heuristic the
            // module-fn path already uses (`body.contains("await")`).
            let is_async =
                (self.handler_has_api_calls(payload) || body.contains("await")) && !is_debounced;
            self.handlers.push((handler_name.clone(), body, is_async));
        }

        // P5-7: Auto-recognize sibling-handler self-calls (`.X()` / `self.X()`)
        // inside used handlers' bodies and mark targets used, so they emit as
        // functions instead of being skipped by the used_handlers filter below
        // (which would otherwise cause TS2304 "Cannot find name 'X'"). Fixpoint
        // covers transitive chains (A -> B -> C). Replaces the manual
        // `expose { .X }` workaround for handler-to-handler calls.
        self.mark_self_called_handlers(widget);

        // Plan 444 (ash-shell-057 ④): shared throwing stub for VM-only
        // natives (`fs.*` / `File.*`) the ts_adapter rewrote to __vmOnly —
        // declared before the handlers that call it. `never` return makes it
        // usable in any expression position under strict TS. Lifecycle bodies
        // (.Init/.Destroy) transpile after this point, so they are detected
        // from the raw AST.
        if self.needs_vm_only_helper
            || widget.lifecycle.iter().any(|l| {
                matches!(&l.payload, crate::aura::LogicPayload::AstStmts(stmts) if Self::stmts_call_vm_only(stmts))
            })
        {
            self.needs_vm_only_helper = true;
            if self.use_typescript {
                script.push_str("function __vmOnly(name: string, ...args: any[]): never {\n  throw new Error('[auto-gen] VM-only primitive \"' + name + '\" has no Vue/JS build — this path only runs in VM mode')\n}\n\n");
            } else {
                script.push_str("function __vmOnly(name, ...args) {\n  throw new Error('[auto-gen] VM-only primitive \"' + name + '\" has no Vue/JS build — this path only runs in VM mode')\n}\n\n");
            }
        }

        // Output handler functions
        // Plan 100: Add return type annotation for TypeScript
        // Plan 132: Add async keyword for handlers with API calls
        // Plan 015 P1#4: emit ALL on-block handlers. Handlers invoked only
        // from extensions (no view binding) used to be silently dropped by
        // the used_handlers filter, forcing inert event decoys and duplicated
        // logic in the extensions (demo CustomScrollbar, jade panels).
        let mut generated_handlers: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (handler_name, handler_body, is_async) in &self.handlers {
            // Plan 448 C: auto-sync mints (`__bind_oninput_<n>`) are empty by
            // construction — Vue realizes the sync via the v-model fold, so
            // emitting a TODO stub for them is pure noise.
            if handler_name.starts_with("__bind_") {
                continue;
            }
            generated_handlers.insert(handler_name.clone());

            // Build params: check for loop-param handlers first, then user-defined params
            // (verbatim on-block pattern — fn names are sanitized, patterns are not)
            let pattern_key = handler_base_patterns
                .get(handler_name)
                .cloned()
                .unwrap_or_else(|| format!(".{}", handler_name));
            let params_str = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                format!("{}: any", loop_var)
            } else {
                let declared = Self::get_handler_params(&widget.handler_params, &pattern_key)
                    .map(|params| {
                        let param_names: Vec<String> =
                            params.iter().map(|p| format!("{}: any", p)).collect();
                        param_names.join(", ")
                    })
                    .unwrap_or_default();
                if declared.is_empty() {
                    // Plan 444 (ash-shell-057 ③): mirror the synthesized
                    // emit args — the bridge fn must accept (and forward)
                    // the variant's payload arity.
                    let arity = self
                        .msg_payload_arities
                        .get(handler_name)
                        .cloned()
                        .unwrap_or(0);
                    if arity > 0 {
                        (0..arity)
                            .map(|i| format!("arg{}: any", i))
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        String::new()
                    }
                } else {
                    declared
                }
            };

            let async_kw = if *is_async { "async " } else { "" };
            let return_type = if self.use_typescript {
                if *is_async {
                    ": Promise<void>"
                } else {
                    ": void"
                }
            } else {
                ""
            };

            // For loop-param handlers with empty body, auto-generate active_id assignment
            let auto_body = if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                if handler_body.is_empty() {
                    if let Some(target_var) = self.find_active_id_var(handler_name) {
                        Some(format!("{}.value = {}", target_var, loop_var))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(ref body) = auto_body {
                let indented = Self::indent_body(body, "  ");
                script.push_str(&format!(
                    "{}function {}({}){} {{\n{}\n}}\n\n",
                    async_kw, handler_name, params_str, return_type, indented
                ));
            } else if handler_body.is_empty() {
                script.push_str(&format!(
                    "{}function {}({}){} {{\n  // TODO\n}}\n\n",
                    async_kw, handler_name, params_str, return_type
                ));
            } else {
                let indented = Self::indent_body(handler_body, "  ");
                script.push_str(&format!(
                    "{}function {}({}){} {{\n{}\n}}\n\n",
                    async_kw, handler_name, params_str, return_type, indented
                ));
            }
        }

        // Generate stub functions for handlers referenced in template but not defined in on-block.
        // Plan 015 P0#3: sorted iteration — used_handlers is a HashSet
        // (RandomState), so the unsorted order churns stub emission across
        // builds (jade gap 56).
        {
            let mut sorted_handlers: Vec<&String> = self.used_handlers.iter().collect();
            sorted_handlers.sort();
            for handler_name in sorted_handlers {
                if generated_handlers.contains(handler_name) {
                    continue;
                }
                // Skip handleChildDelete — it's generated separately below
                if handler_name == "handleChildDelete" && self.needs_child_delete_handler {
                    continue;
                }
                let return_type = if self.use_typescript { ": void" } else { "" };
                // Plan 444 (ash-shell-057 ③): a template-referenced handler with
                // NO on-block entry is the pass-through idiom — the widget wants
                // the event re-emitted to the parent (e.g. Sort(int,int) called
                // from a table header → store.SortTable). Bridge it explicitly:
                // accept and forward the msg variant's payload arity instead of
                // the old arg-less TODO stub (TS2554 at the call site).
                // Loop-param handlers keep the loop-var param (the call site
                // passes it) even when bridging — arity is 0 there by
                // construction (payload variants bind $event instead).
                let emit_event = self
                    .emit_events
                    .iter()
                    .find(|e| {
                        e.as_str() == handler_name.as_str()
                            || Self::sanitize_ident(e.as_str()) == *handler_name
                    })
                    .cloned();
                let bridge_arity = emit_event.as_ref().map(|_| {
                    self.msg_payload_arities
                        .get(handler_name)
                        .cloned()
                        .unwrap_or(0)
                });
                let bridge_params =
                    match (&bridge_arity, self.loop_param_handlers.get(handler_name)) {
                        (_, Some(loop_var)) => format!("{}: any", loop_var),
                        (Some(arity), None) => (0..*arity)
                            .map(|i| format!("arg{}: any", i))
                            .collect::<Vec<_>>()
                            .join(", "),
                        (None, None) => String::new(),
                    };
                let auto_body = if let (Some(arity), Some(event)) = (&bridge_arity, &emit_event) {
                    let args = (0..*arity)
                        .map(|i| format!("arg{}", i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if args.is_empty() {
                        Some(format!("emit('{}')", event))
                    } else {
                        Some(format!("emit('{}', {})", event, args))
                    }
                } else if let Some(loop_var) = self.loop_param_handlers.get(handler_name) {
                    if let Some(target_var) = self.find_active_id_var(handler_name) {
                        Some(format!("{}.value = {}", target_var, loop_var))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(body) = auto_body {
                    script.push_str(&format!(
                        "function {}({}){} {{\n  {}\n}}\n\n",
                        handler_name, bridge_params, return_type, body
                    ));
                } else {
                    script.push_str(&format!(
                        "function {}({}){} {{\n  // TODO: handler not defined in on-block\n}}\n\n",
                        handler_name, bridge_params, return_type
                    ));
                }
            }

            // Generate handleChildDelete for parent components with array state
            // This handles the case where a sub-widget emits 'Delete' and the parent
            // needs to remove the item from its array (e.g., notes list)
            if self.needs_child_delete_handler {
                // Find the active index variable name (active_id, active_index, or active_idx)
                let active_var = self
                    .state_names
                    .iter()
                    .find(|n| n.starts_with("active"))
                    .cloned()
                    .unwrap_or_else(|| "active_id".to_string());
                script.push_str("function handleChildDelete() {\n");
                script.push_str(&format!(
                    "  if ({}.value < notes.value.length) notes.value.splice({}.value, 1)\n",
                    active_var, active_var
                ));
                script.push_str(&format!("  if (notes.value.length > 0) {{\n"));
                script.push_str(&format!("    {}.value = 0\n", active_var));
                script.push_str("  }\n");
                if self.state_names.iter().any(|n| n == "editing") {
                    script.push_str("  editing.value = false\n");
                }
                script.push_str("}\n\n");
            }
        }

        // Emit module-level plain functions attached to the widget file (e.g. `fn eval_expr(...)`)
        if !self.module_fns.is_empty() {
            let ctx = self.handler_ts_ctx();
            for mfn in &self.module_fns {
                let param_list = mfn
                    .params
                    .iter()
                    .map(|p| format!("{}: any", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_anno = if mfn.ret_ts.is_empty() {
                    String::new()
                } else {
                    format!(": {}", mfn.ret_ts)
                };
                script.push_str(&format!(
                    "function {}({}){} {{\n",
                    mfn.name, param_list, ret_anno
                ));
                let body = crate::ui_gen::ts_adapter::transpile_handler_body(&mfn.body, &ctx);
                let indented = Self::indent_body(&body, "  ");
                script.push_str(&indented);
                script.push_str("\n}\n\n");
            }
        }

        // Plan 522: use 导入的模块级 helper fn —— 按需(computed/handler 体
        // 引用)转译进 SFC script 尾部。function 声明有提升,上方 computed
        // 段的裸名调用点可解析,调用点零改写。
        self.emit_use_module_fns(widget, &mut script);

        // Generate lifecycle hooks from widget.lifecycle
        // .Init → onMounted
        if let Some(init) = widget.lifecycle.iter().find(|l| l.name == "Init") {
            // Plan 617 T-06: build the body first so an awaiting `.Init` (e.g. the
            // /api/media/scan fetch) is emitted `async` instead of producing
            // "Unexpected reserved word 'await'" from the SFC compiler.
            let body = self
                .generate_handler_body(&init.payload)
                .unwrap_or_default();
            let is_async = self.handler_has_api_calls(&init.payload) || body.contains("await");
            let async_kw = if is_async { "async " } else { "" };
            let indented = Self::indent_body(&body, "  ");
            if !widget.props.is_empty() {
                // PLAN-684 ADR-19 Vue 对等：props 重播种后重放 Init（与 VM 轨
                // 渲染期补发同构）。Init 契约为纯派生幂等（chart-components.md）。
                script.push_str(&format!(
                    "{}function __autoReplayInit() {{\n{}\n}}\n\n",
                    async_kw, indented
                ));
                script.push_str("onMounted(() => {\n  __autoReplayInit()\n})\n\n");
                script.push_str(
                    "watch(() => props, () => {\n  __autoReplayInit()\n}, { deep: true })\n\n",
                );
            } else {
                script.push_str(&format!(
                    "onMounted({}() => {{\n{}\n}})\n\n",
                    async_kw, indented
                ));
            }
        }
        // .Destroy → onUnmounted
        if let Some(destroy) = widget.lifecycle.iter().find(|l| l.name == "Destroy") {
            let body = self
                .generate_handler_body(&destroy.payload)
                .unwrap_or_default();
            // Plan 408 P12 §10.7: .Destroy 含 await/API 调用时用 async。
            let async_kw = if self.handler_has_api_calls(&destroy.payload) {
                "async "
            } else {
                ""
            };
            script.push_str(&format!(
                "onUnmounted({}() => {{\n  {}\n}})\n\n",
                async_kw, body
            ));
        }

        // Global (window/document-level) event listeners declared in the view
        // via `.window`/`.document` event modifiers, e.g.
        // `onmousemove.window: .DragMove($event)` or
        // `onwheel.document.capture.prevent: .LockWheel($event)`.
        // Registered on mount and removed on unmount (no leaks across
        // component instances).
        if !self.global_listeners.is_empty() {
            // Wrapper functions first (prevent/stop/arg adapters).
            for gl in &self.global_listeners {
                if let Some(wrapper) = &gl.wrapper {
                    script.push_str(wrapper);
                }
            }
            script.push_str("onMounted(() => {\n");
            for gl in &self.global_listeners {
                let options = Self::listener_options(gl, false);
                script.push_str(&format!(
                    "  {}.addEventListener('{}', {}{})\n",
                    gl.target, gl.event, gl.listener, options
                ));
            }
            script.push_str("})\n\n");
            script.push_str("onUnmounted(() => {\n");
            for gl in &self.global_listeners {
                // removeEventListener only matches on the capture flag.
                let options = Self::listener_options(gl, true);
                script.push_str(&format!(
                    "  {}.removeEventListener('{}', {}{})\n",
                    gl.target, gl.event, gl.listener, options
                ));
            }
            script.push_str("})\n\n");
        }

        // Plan 451 P2: `actions {}`-declared shortcuts — a window keydown
        // FALLBACK layer under element-level onkeydown bindings (VM parity:
        // the vm subscription looks the key up in action_config only after
        // the DSL bind layer). Plain keys (no Ctrl/Alt/meta) are skipped
        // while typing in a text field, mirroring the vm's focused-widget
        // capture; matched keys preventDefault so browser defaults (Ctrl+S
        // page save, Ctrl+P print) don't fire. mac Cmd maps onto the Ctrl
        // layer (web-side convenience; the vm has no meta key).
        if !self.actions_key_bindings.is_empty() {
            let keymap_ty = if self.use_typescript {
                ": Record<string, () => void>"
            } else {
                ""
            };
            script.push_str(&format!("const __autoActionsKeymap{} = {{\n", keymap_ty));
            for (key, handler) in &self.actions_key_bindings {
                script.push_str(&format!("  '{}': {},\n", key, handler));
            }
            script.push_str("}\n");
            let target_cast = if self.use_typescript {
                "const t = e.target as HTMLElement | null\n    "
            } else {
                "const t = e.target\n    "
            };
            let fn_sig = if self.use_typescript {
                "function __autoActionsKeydown(e: KeyboardEvent) {"
            } else {
                "function __autoActionsKeydown(e) {"
            };
            script.push_str(&format!(
                "{fn_sig}\n\
                 \x20 // IME composition (LAUNCHER-001 R-02 / 452 spike): Enter/Esc go to the IME first.\n\
                 \x20 if (e.isComposing || e.keyCode === 229) return\n\
                 \x20 const hasModifier = e.ctrlKey || e.altKey || e.metaKey\n\
                 \x20 if (!hasModifier) {{\n\
                 \x20   {target_cast}\
                 if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return\n\
                 \x20 }}\n\
                 \x20 let key = e.key\n\
                 \x20 if (key.length === 1) key = key.toLowerCase()\n\
                 \x20 const combo = (e.ctrlKey || e.metaKey ? 'Ctrl+' : '') + (e.altKey ? 'Alt+' : '') + key\n\
                 \x20 const fn = __autoActionsKeymap[combo]\n\
                 \x20 if (fn) {{\n\
                 \x20   e.preventDefault()\n\
                 \x20   fn()\n\
                 \x20 }}\n\
                 }}\n\n"
            ));
            script.push_str("onMounted(() => {\n  window.addEventListener('keydown', __autoActionsKeydown)\n})\n\n");
            script.push_str("onUnmounted(() => {\n  window.removeEventListener('keydown', __autoActionsKeydown)\n})\n\n");
        }

        // Plan 464 T2: `bind {}` 块 → 全局 keydown 层。键名与 vm key_str 同名
        // （ArrowUp/Enter/Escape/Tab/Backspace/"Ctrl+ "），查表命中即派发。
        // 与 actions 回退层的差异：输入框守卫只让位「会进文本」的键（单字符/
        // Backspace）——vm 单行 input 不捕获方向键/Escape/Tab，palette 键盘流
        // 依赖这组键在打字时仍可达（464 T2 验收原案 fill→↓×2→Enter）。
        // Enter 双派发（此处 keydown + input onenter 的 keyup）幂等无害。
        if !self.bind_key_bindings.is_empty() {
            let keymap_ty = if self.use_typescript {
                ": Record<string, () => void>"
            } else {
                ""
            };
            script.push_str(&format!("const __autoBindKeymap{} = {{\n", keymap_ty));
            for (key, handler) in &self.bind_key_bindings {
                script.push_str(&format!("  '{}': {},\n", key, handler));
            }
            script.push_str("}\n");
            let target_cast = if self.use_typescript {
                "const t = e.target as HTMLElement | null\n    "
            } else {
                "const t = e.target\n    "
            };
            let fn_sig = if self.use_typescript {
                "function __autoBindKeydown(e: KeyboardEvent) {"
            } else {
                "function __autoBindKeydown(e) {"
            };
            script.push_str(&format!(
                "{fn_sig}\n\
                 \x20 // IME composition (LAUNCHER-001 R-02 / 452 spike): Enter/Esc go to the IME first.\n\
                 \x20 if (e.isComposing || e.keyCode === 229) return\n\
                 \x20 const hasModifier = e.ctrlKey || e.altKey || e.metaKey\n\
                 \x20 let key = e.key\n\
                 \x20 if (!hasModifier) {{\n\
                 \x20   {target_cast}\
                 if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable) && (key.length === 1 || key === 'Backspace')) return\n\
                 \x20 }}\n\
                 \x20 if (key.length === 1) key = key.toLowerCase()\n\
                 \x20 const combo = (e.ctrlKey || e.metaKey ? 'Ctrl+' : '') + (e.altKey ? 'Alt+' : '') + key\n\
                 \x20 const fn = __autoBindKeymap[combo]\n\
                 \x20 if (fn) {{\n\
                 \x20   e.preventDefault()\n\
                 \x20   fn()\n\
                 \x20 }}\n\
                 }}\n\n"
            ));
            script.push_str("onMounted(() => {\n  window.addEventListener('keydown', __autoBindKeydown)\n})\n\n");
            script.push_str("onUnmounted(() => {\n  window.removeEventListener('keydown', __autoBindKeydown)\n})\n\n");
        }

        // Note: auto-edit-mode onMounted was previously hardcoded here (Plan 367 P0-3
        // removed it). This logic now lives in the .at source's .Init handler,
        // which is faithfully transpiled to onMounted above. Having both caused
        // a duplicate onMounted in EditorPanel.vue.

        // Generate timer/tick mechanism (setInterval + onUnmounted cleanup)
        // The timer only runs when the widget has a `running` state var set to "true"
        if let Some(interval) = widget.tick_interval {
            // Check if there's a 'running' state variable to gate the timer
            let has_running = widget.state_vars.iter().any(|s| s.name == "running");

            if self.use_typescript {
                script.push_str("const tickTimer = ref<number | null>(null)\n\n");
            } else {
                script.push_str("const tickTimer = ref(null)\n\n");
            }

            // Find the .Tick handler body
            let tick_body = widget
                .handlers
                .get(".Tick")
                .map(|payload| self.generate_handler_body(payload).unwrap_or_default())
                .unwrap_or_default();

            if has_running {
                // Timer starts/stops based on `running` state — use watch to manage interval
                script.push_str(&format!("watch(running, (val) => {{\n  if (val === 'true' && tickTimer.value === null) {{\n    tickTimer.value = setInterval(async () => {{\n      {}\n    }}, {})\n  }} else if (val !== 'true' && tickTimer.value !== null) {{\n    clearInterval(tickTimer.value)\n    tickTimer.value = null\n  }}\n}})\n\n", tick_body, interval));
            } else {
                // No running gate — start timer immediately on mount
                script.push_str(&format!("onMounted(() => {{\n  tickTimer.value = setInterval(async () => {{\n    {}\n  }}, {})\n}})\n\n", tick_body, interval));
            }

            // If the widget has both `elapsed` and `time_display`/`ms_display`,
            // add a watch to format elapsed time into display strings
            let has_elapsed = widget.state_vars.iter().any(|s| s.name == "elapsed");
            let has_time_display = widget.state_vars.iter().any(|s| s.name == "time_display");
            let has_ms_display = widget.state_vars.iter().any(|s| s.name == "ms_display");
            if has_elapsed && (has_time_display || has_ms_display) {
                if !imports.contains(&"watch") {
                    imports.push("watch");
                }
                script.push_str("watch(elapsed, (ms) => {\n");
                script.push_str("  const totalSec = Math.floor(ms / 1000)\n");
                script.push_str("  const min = Math.floor(totalSec / 60)\n");
                script.push_str("  const sec = totalSec % 60\n");
                if has_time_display {
                    script.push_str("  time_display.value = String(min).padStart(2, '0') + ':' + String(sec).padStart(2, '0')\n");
                }
                if has_ms_display {
                    script.push_str("  ms_display.value = '.' + String(Math.floor((ms % 1000) / 10)).padStart(2, '0')\n");
                }
                script.push_str("})\n\n");
            }

            script.push_str("onUnmounted(() => {\n  if (tickTimer.value !== null) {\n    clearInterval(tickTimer.value)\n  }\n})\n\n");
        }

        // Plan 051 C7: timer 块条目——onMounted setInterval / onUnmounted
        // clearInterval；`when` 门控在回调内求值（假跳过本拍，计时不停）。
        if !widget.timers.is_empty() {
            for t in &widget.timers {
                let var = format!("__timer_{}", t.event);
                let body = widget
                    .handlers
                    .get(&format!(".{}", t.event))
                    .and_then(|p| self.generate_handler_body(p).ok())
                    .unwrap_or_default();
                let guard = match t.when.as_deref() {
                    Some(c) => Self::timer_condition_to_js(c),
                    None => "true".to_string(),
                };
                script.push_str(&format!(
                    "let {var}: any = null
const __t51_on_{ev} = async () => {{ if ({guard}) {{ {body} }} }}
onMounted(() => {{ {var} = setInterval(__t51_on_{ev}, {ms}) }})
onUnmounted(() => {{ if ({var} !== null) {{ clearInterval({var}); {var} = null }} }})

",
                    var = var,
                    ev = t.event,
                    guard = guard,
                    body = body,
                    ms = t.every_ms,
                ));
            }
        }

        // Generate previewcard state variables and copyCode function
        if !self.previewcard_data.is_empty() {
            // Add copiedCode state
            if self.use_typescript {
                script.push_str("const copiedCode = ref<string>('')\n");
            } else {
                script.push_str("const copiedCode = ref('')\n");
            }

            // Helper function to convert kebab-case to PascalCase
            let to_pascal_case = |s: &str| -> String {
                s.split('-')
                    .map(|part| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect()
            };

            // Add state for each previewcard
            for pc in &self.previewcard_data {
                let id_pascal = to_pascal_case(&pc.id);
                let show_var = format!("show{}Code", id_pascal);
                let active_var = format!("active{}Tab", id_pascal);
                if self.use_typescript {
                    script.push_str(&format!("const {} = ref<boolean>(true)\n", show_var)); // expanded by default
                    script.push_str(&format!("const {} = ref<string>('auto')\n", active_var));
                } else {
                    script.push_str(&format!("const {} = ref(true)\n", show_var)); // expanded by default
                    script.push_str(&format!("const {} = ref('auto')\n", active_var));
                }
            }
            script.push('\n');

            // Add copyCode function
            if self.use_typescript {
                script.push_str("// Copy to clipboard function\n");
                script.push_str(
                    "async function copyCode(code: string, id: string): Promise<void> {\n",
                );
                script.push_str("  try {\n");
                script.push_str("    await navigator.clipboard.writeText(code)\n");
                script.push_str("    copiedCode.value = id\n");
                script.push_str("    setTimeout(() => {\n");
                script.push_str("      copiedCode.value = ''\n");
                script.push_str("    }, 2000)\n");
                script.push_str("  } catch (err) {\n");
                script.push_str("    console.error('Failed to copy:', err)\n");
                script.push_str("  }\n");
                script.push_str("}\n\n");
            } else {
                script.push_str("// Copy to clipboard function\n");
                script.push_str("async function copyCode(code, id) {\n");
                script.push_str("  try {\n");
                script.push_str("    await navigator.clipboard.writeText(code)\n");
                script.push_str("    copiedCode.value = id\n");
                script.push_str("    setTimeout(() => {\n");
                script.push_str("      copiedCode.value = ''\n");
                script.push_str("    }, 2000)\n");
                script.push_str("  } catch (err) {\n");
                script.push_str("    console.error('Failed to copy:', err)\n");
                script.push_str("  }\n");
                script.push_str("}\n\n");
            }

            // Add code sample constants
            for pc in &self.previewcard_data {
                // Convert PascalCase to camelCase (e.g., "CardBasic" -> "cardBasic")
                let id_camel: String = pc
                    .id
                    .split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let auto_var = format!("{}AutoCode", id_camel);
                let vue_var = format!("{}VueCode", id_camel);
                script.push_str(&format!(
                    "const {} = `{}`\n",
                    auto_var,
                    sanitize_embedded_code(&pc.auto_code)
                ));
                script.push_str(&format!(
                    "const {} = `{}`\n",
                    vue_var,
                    sanitize_embedded_code(&pc.vue_code)
                ));
            }

            // Add code constants for each codeblock
            for cb in &self.codeblock_data {
                // Convert kebab-case to camelCase (e.g., "install-button" -> "installButton")
                let id_camel: String = cb
                    .id
                    .split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let code_var = format!("{}Code", id_camel);
                script.push_str(&format!(
                    "const {} = `{}`\n",
                    code_var,
                    sanitize_embedded_code(&cb.code)
                ));
            }

            // Plan 106: Add watchers for syntax highlighting when tabs change
            for pc in &self.previewcard_data {
                let id_pascal = to_pascal_case(&pc.id);
                let active_var = format!("active{}Tab", id_pascal);
                script.push_str(&format!(
                    "watch({}, () => {{\n  nextTick(() => Prism.highlightAll())\n}})\n",
                    active_var
                ));
            }

            // Add onMounted hook for initial syntax highlighting
            script.push_str("onMounted(() => {\n  nextTick(() => Prism.highlightAll())\n})\n");
            script.push('\n');
        } else if !self.codeblock_data.is_empty() {
            // Codeblocks only (no previewcard)
            // Add copiedCode state
            if self.use_typescript {
                script.push_str("const copiedCode = ref<string>('')\n");
            } else {
                script.push_str("const copiedCode = ref('')\n");
            }

            // Add copyCode function
            script.push_str("\n// Copy to clipboard function\n");
            script.push_str("async function copyCode(code: string, id: string): Promise<void> {\n");
            script.push_str("  try {\n");
            script.push_str("    await navigator.clipboard.writeText(code)\n");
            script.push_str("    copiedCode.value = id\n");
            script.push_str("    setTimeout(() => {\n");
            script.push_str("      copiedCode.value = ''\n");
            script.push_str("    }, 2000)\n");
            script.push_str("  } catch (err) {\n");
            script.push_str("    console.error('Failed to copy:', err)\n");
            script.push_str("  }\n");
            script.push_str("}\n\n");

            // Add code constants for each codeblock
            for cb in &self.codeblock_data {
                // Convert kebab-case to camelCase (e.g., "install-button" -> "installButton")
                let id_camel: String = cb
                    .id
                    .split('-')
                    .enumerate()
                    .map(|(i, part)| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => {
                                if i == 0 {
                                    c.to_lowercase().collect::<String>() + chars.as_str()
                                } else {
                                    c.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        }
                    })
                    .collect();
                let code_var = format!("{}Code", id_camel);
                script.push_str(&format!(
                    "const {} = `{}`\n",
                    code_var,
                    sanitize_embedded_code(&cb.code)
                ));
            }
            script.push('\n');
        }

        // Widget-level `expose { ... }` block → defineExpose({ ... }) so a
        // parent holding a template ref on this component can call imperative
        // methods (exposed `on` handlers / imported fns) or read exposed
        // state/template refs. Vue's expose proxy unwraps refs on access, so
        // exposing the ref object directly is correct.
        if !widget.exposes.is_empty() {
            script.push_str(&format!(
                "defineExpose({{ {} }})\n",
                widget.exposes.join(", ")
            ));
        }

        // Plan 409 §8: widget-path accent color system. When the widget owns an
        // `accent_color` state, inject the same 5-color palette + applyAccent
        // helper used by the store composable path (Plan 360), plus an onMounted
        // bootstrap that restores the saved choice from localStorage. The
        // palette is aligned with auto-forge (indigo/coral/ocean/sage/amber).
        if self.has_accent_color || self.has_theme_name {
            script.push('\n');
            script.push_str(Self::ACCENT_PALETTE_JS);
            // PLAN-601 T-06: full named-theme runtime rides the same gate —
            // theme switching is the whole-set generalization of accent
            // switching (applyTheme reuses applyAccent as its overlay).
            script.push('\n');
            script.push_str(&Self::theme_runtime_js());
            script.push_str("\n// Restore saved theme (and accent when the widget owns one) on mount.\n");
            script.push_str("onMounted(() => {\n");
            // PLAN-601 T-06: restore the active theme on mount (localStorage
            // may carry a user choice or a generator-seeded declared theme).
            // Scaffold default needs no apply — the stylesheet already is it.
            script.push_str("  const __th = getActiveTheme()\n");
            script.push_str("  if (__th !== 'scaffold') applyTheme(__th, document.documentElement.classList.contains('dark'))\n");
            // PLAN-007 T-05：accent 恢复仅在 widget 声明 accent_color 时发射
            //（theme_name-only 应用无该 ref——PLAN-007 试点面）。
            if self.has_accent_color {
                script.push_str("  const saved = getSavedAccent()\n");
                // Plan 458: only override the CLI/env-seeded ref when a real
                // persisted choice exists (getSavedAccent returns '' otherwise).
                script.push_str("  if (saved) {\n");
                script.push_str("    accent_color.value = saved\n");
                script.push_str(
                    "    applyAccent(saved, document.documentElement.classList.contains('dark'))\n",
                );
                script.push_str("  }\n");
            }
            script.push_str("})\n");
            // Plan 458: re-apply the accent when the theme flips at runtime —
            // applyAccent also writes the .dark-scoped --primary, which would
            // otherwise fall back to the stylesheet's default indigo.
            // PLAN-601 T-06: the flip now re-applies the ACTIVE THEME's full
            // set (applyTheme dark pass) — accent overlay included — so mode
            // flips keep a non-default theme instead of regressing to the
            // scaffold `.dark` stylesheet block.
            if self.has_dark_mode {
                let dark_var = self
                    .dark_mode_var
                    .clone()
                    .unwrap_or_else(|| "dark_mode".to_string());
                // PLAN-007 T-05：accent_color 是 watch 第三参的可选覆盖——
                // 仅在 widget 声明它时传（theme_name-only 应用用默认
                // getSavedAccent()）。
                if self.has_accent_color {
                    script.push_str(&format!(
                        "// Plan 458: keep the accent and html dark class in sync across theme flips.\nwatch({}, (v) => {{\n  document.documentElement.classList.toggle('dark', v)\n  applyTheme(getActiveTheme(), v, accent_color.value)\n}}, {{ immediate: true }})\n",
                        dark_var
                    ));
                } else {
                    script.push_str(&format!(
                        "// Plan 458: keep the html dark class in sync across theme flips.\nwatch({}, (v) => {{\n  document.documentElement.classList.toggle('dark', v)\n  applyTheme(getActiveTheme(), v)\n}}, {{ immediate: true }})\n",
                        dark_var
                    ));
                }
            }
            script.push('\n');
        }


        // PLAN-007 T-05：theme_name → 主题热切换（applyTheme 未知名零变化 =
        // 事务保旧；theme/mode 同帧变更经两个 watch 同一 flush 批次应用，
        // C-07 同步事件原子发布）。
        if self.has_theme_name {
            script.push_str(
                "// PLAN-007 T-05: theme_name binds the active theme family (tree themes join THEME_PALETTES via __AUTO_TREE_THEMES__).\nwatch(theme_name, (n) => {\n  if (n) applyTheme(n, document.documentElement.classList.contains('dark'))\n})\n",
            );
        }
        Ok(script)
    }

    /// Generate handler function body from LogicPayload
    /// Build the AuraTsContext used for handler-body transpilation. Plan 053
    /// M5/P5-6 factored this out so the debounced variant can layer the
    /// seq-guard on top of the same base context without duplicating it.
    pub(super) fn handler_ts_ctx(&self) -> crate::ui_gen::ts_adapter::AuraTsContext {
        let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(
            self.state_names.iter().cloned().collect(),
        )
        .with_props(self.prop_names.iter().cloned().collect())
        .with_refs(self.template_refs.iter().cloned().collect())
        .with_computed(self.computed_names.iter().cloned().collect());
        if !self.project_api_functions.is_empty() {
            ctx = ctx.with_api_functions(self.project_api_functions.clone());
        }
        // Plan 012 Batch A (gap 19): proven array/string receivers for
        // the .remove/.contains method-mapping gate.
        let (arrays, strings) = self.typed_collection_names();
        if !self.store_deps.is_empty() {
            ctx = ctx.with_store_facade_from(self.store_deps[0].clone());
            // PLAN-048: deps[1..] 的 facade 变量映射(与发射的 const 配套)。
            let mut extra = std::collections::HashMap::new();
            for dep in self.store_deps.iter().skip(1) {
                extra.insert(dep.clone(), facade_var_for(dep));
            }
            ctx = ctx.with_store_facades(extra);
            // PLAN-010: 多 store widget 的裸 `store.Method()` 归属路由。
            // 仅当方法唯一归属 deps[1..] 中某 store 且首 store 未声明时才
            // 路由；歧义/首 store 自有/未知保持 `store.`（现状）。surface
            // 缺省或单 store 不建表（调用方行为零变化）。
            if self.store_deps.len() > 1 && !self.store_surface.is_empty() {
                let first = &self.store_deps[0];
                let owns = |dep: &str, m: &str| {
                    self.store_surface.get(dep).map_or(false, |s| s.contains(m))
                };
                let mut routing = std::collections::HashMap::new();
                for dep in self.store_deps.iter().skip(1) {
                    if let Some(methods) = self.store_surface.get(dep.as_str()) {
                        for raw in methods {
                            // 归一化 on-handler 键的前导点（`.Init` → `Init`）。
                            let m = raw.trim_start_matches('.');
                            if m.is_empty()
                                || owns(first, m)
                                || routing.contains_key(m)
                                || self
                                    .store_deps
                                    .iter()
                                    .skip(1)
                                    .any(|d| d != dep && owns(d, m))
                            {
                                continue;
                            }
                            routing.insert(m.to_string(), facade_var_for(dep));
                        }
                    }
                }
                if !routing.is_empty() {
                    ctx = ctx.with_bare_store_routing(routing);
                }
            }
        }
        ctx.with_typed_collections(arrays, strings)
            .with_typed_ints(self.int_names.iter().cloned().collect())
            .with_facade_names(self.facade_local_names())
            .with_facade_ref_fields(self.facade_ref_fields_map())
    }

    /// Plan 522: 把 computed/handler 体引用的 use 导入 helper fn 转译为 TS
    /// 函数发射进 SFC script 尾部(复用 module_fns 的发射形态)。要点:
    /// - 入口门控:widget 体引用的裸名必须 ∈ use items 导入名集合(与 vm
    ///   import_aliases 同口径 —— 未导入的池内 fn 不拉);
    /// - 依赖闭包:被拉 fn 体内引用的池内其他 fn 一并拉入,迭代到不动点
    ///   (对应 vm 侧全模块加载:build_month_grid → weekday_of 兄弟解析);
    /// - 去重:同名只发一份(池序稳定输出);
    /// - ext_imports 同名优先(手写逃逸口赢,静默跳过);
    /// - 命名冲突(state/computed/props/handler/facade/同文件 module fn)
    ///   → R013 警告 + 跳过发射(提示 ext_imports 兜底);
    /// - 转译边界:模式匹配/借用语义 fn 体不保证转译 → R013 警告 + 跳过
    ///   (回退现状,不阻塞生成)。
    pub(super) fn emit_use_module_fns(&self, widget: &AuraWidget, script: &mut String) {
        use std::collections::{HashMap, HashSet};

        if self.use_module_fns.is_empty() {
            return;
        }
        // 1. widget 体引用的裸名(computed 表达式 + handler 体 + lifecycle 体)。
        //    `.Init` 等 lifecycle 提取进 onMounted/onUnmounted(非 handlers
        //    表)——其体内的 helper 调用同样需要拉取(Plan 522 T4 donut 实证:
        //    dc/ds 在 .Init 里,漏扫则 TS2304)。
        let mut referenced: HashSet<String> = HashSet::new();
        for c in &widget.computed {
            collect_called_bare_names_expr(&c.expr, &mut referenced);
        }
        for payload in widget.handlers.values() {
            if let LogicPayload::AstStmts(stmts) = payload {
                collect_called_bare_names_stmts(stmts, &mut referenced);
            }
        }
        for l in &widget.lifecycle {
            if let LogicPayload::AstStmts(stmts) = &l.payload {
                collect_called_bare_names_stmts(stmts, &mut referenced);
            }
        }
        // 2. 入口门控 + 闭包拉取(池内按名,迭代到不动点)。
        let mut pool_position: HashMap<&str, usize> = HashMap::new();
        for (i, mfn) in self.use_module_fns.iter().enumerate() {
            pool_position.entry(mfn.name.as_str()).or_insert(i);
        }
        let mut pulled_idx: Vec<usize> = Vec::new();
        let mut pulled_names: HashSet<String> = HashSet::new();
        let mut frontier: Vec<String> = referenced
            .into_iter()
            .filter(|n| self.use_imported_names.contains(n.as_str()))
            .collect();
        while let Some(name) = frontier.pop() {
            if !pulled_names.insert(name.clone()) {
                continue;
            }
            let Some(&idx) = pool_position.get(name.as_str()) else {
                continue;
            };
            pulled_idx.push(idx);
            // 闭包:被拉 fn 体内引用的池内 fn 入队(池内解析,不受导入门限)。
            let mut called: HashSet<String> = HashSet::new();
            collect_called_bare_names_stmts(&self.use_module_fns[idx].body, &mut called);
            for dep in called {
                if !pulled_names.contains(&dep) && pool_position.contains_key(dep.as_str()) {
                    frontier.push(dep);
                }
            }
        }
        if pulled_idx.is_empty() {
            return;
        }
        pulled_idx.sort_unstable();
        pulled_idx.dedup();
        // 3. 发射前过滤:ext_imports 同名静默跳过;转译边界/命名冲突 →
        //    R013 警告 + 跳过(v1 回退现状,不阻塞生成)。
        let mut ext_bound: HashSet<&str> = HashSet::new();
        for (symbols, _, _) in &self.ext_import_lines {
            for s in symbols {
                ext_bound.insert(s.as_str());
            }
        }
        let mut conflict_names: HashSet<&str> = HashSet::new();
        conflict_names.extend(self.state_names.iter().map(String::as_str));
        conflict_names.extend(self.computed_names.iter().map(String::as_str));
        conflict_names.extend(self.prop_names.iter().map(String::as_str));
        conflict_names.extend(self.used_handlers.iter().map(String::as_str));
        let facade_locals = self.facade_local_names();
        conflict_names.extend(facade_locals.iter().map(String::as_str));
        conflict_names.extend(self.module_fns.iter().map(|f| f.name.as_str()));
        let ctx = self.handler_ts_ctx();
        for idx in pulled_idx {
            let mfn = &self.use_module_fns[idx];
            // ext_imports(手写 TS 逃逸口)已绑定同名符号 → 手写优先,不发射。
            if ext_bound.contains(mfn.name.as_str()) {
                continue;
            }
            if let Some(reason) = use_fn_body_unsupported(&mfn.body) {
                self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!(
                        "use-imported fn `{}` not emitted into the SFC: {} — falls back to no emission (ext_imports hand-written TS is the escape hatch)",
                        mfn.name, reason
                    ),
                );
                continue;
            }
            if conflict_names.contains(mfn.name.as_str()) {
                self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!(
                        "use-imported fn `{}` collides with an existing script symbol (state/computed/prop/handler/module fn) — skipped emission; rename it or bind a hand-written version via ext_imports",
                        mfn.name
                    ),
                );
                continue;
            }
            let param_list = mfn
                .params
                .iter()
                .map(|p| format!("{}: any", p))
                .collect::<Vec<_>>()
                .join(", ");
            let ret_anno = if mfn.ret_ts.is_empty() {
                String::new()
            } else {
                format!(": {}", mfn.ret_ts)
            };
            script.push_str(&format!(
                "function {}({}){} {{\n",
                mfn.name, param_list, ret_anno
            ));
            // Auto fn 体尾表达式即返回值(VM 侧语义)——用 Plan 448 H1 的
            // return 化转译,裸表达式体(`(a && b) || c`)不会发射成无返回
            // 语句(否则 Vue 运行时拿到 undefined,闰年判断这类谓词失效)。
            let body = crate::ui_gen::ts_adapter::transpile_body_as_return(&mfn.body, &ctx);
            let indented = Self::indent_body(&body, "  ");
            script.push_str(&indented);
            script.push_str("\n}\n\n");
        }
    }

    pub(super) fn generate_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                let ctx = self.handler_ts_ctx();
                let body = crate::ui_gen::ts_adapter::transpile_handler_body(stmts, &ctx);
                self.drain_ctx_warnings(&ctx);
                Ok(body)
            }
            LogicPayload::Bytecode(_) => Err(GenError::UnsupportedStmt(
                "Bytecode not supported in Vue generator".to_string(),
            )),
        }
    }

    /// Plan 053 M5/P5-6: generate a debounced complete-handler body. The
    /// original body is transpiled with a seq-guard context (state-ref
    /// assignments get `if (__seq === __completeSeq)` guards so a stale,
    /// slower complete result can't overwrite newer suggestions), then wrapped
    /// in a setTimeout(80ms) debounce with a sequence counter. The handler
    /// function itself stays synchronous (is_async=false) — the
    /// `async`/`await` moves into the setTimeout callback.
    pub(super) fn generate_debounced_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                let ctx = self
                    .handler_ts_ctx()
                    .with_debounce_seq_var("__seq".to_string());
                let inner = crate::ui_gen::ts_adapter::transpile_handler_body(stmts, &ctx);
                self.drain_ctx_warnings(&ctx);
                // setTimeout wrapper: clear any pending timer, bump the seq,
                // schedule the body. The async callback retains the
                // `await complete(...)`; the seq guards on `.suggestions =`
                // ensure only the newest result writes.
                Ok(format!(
                    "if (__completeTimer) clearTimeout(__completeTimer)\nconst __seq = ++__completeSeq\n__completeTimer = setTimeout(async () => {{\n{inner}\n}}, 80)"
                ))
            }
            LogicPayload::Bytecode(_) => Err(GenError::UnsupportedStmt(
                "Bytecode not supported in Vue generator".to_string(),
            )),
        }
    }

    /// Plan 012 Batch A (gap 19): split known state/prop names into proven
    /// array / string receivers for the ts_adapter method-mapping gate.
    pub(super) fn typed_collection_names(
        &self,
    ) -> (
        std::collections::HashSet<String>,
        std::collections::HashSet<String>,
    ) {
        let mut arrays = std::collections::HashSet::new();
        let mut strings = std::collections::HashSet::new();
        for (name, ty) in self.state_types.iter().chain(self.prop_types.iter()) {
            if ty.ends_with("[]") {
                arrays.insert(name.clone());
            }
            if ty == "string" {
                strings.insert(name.clone());
            }
        }
        (arrays, strings)
    }

    /// Plan 014: is this Auto type int-ish? TS type maps collapse int and
    /// float to `number`, so division lowering needs the original `ast::Type`.
    /// Only proven-int operands get `Math.trunc` on `/` and `%`.
    pub(super) fn is_int_auto_type(ty: &auto_lang_core::ast::Type) -> bool {
        use auto_lang_core::ast::Type;
        matches!(
            ty,
            Type::Int | Type::I64 | Type::Uint | Type::U64 | Type::USize
        ) || matches!(
            ty,
            Type::User(decl)
                if matches!(decl.name.as_str(), "int" | "i64" | "uint" | "u64" | "usize" | "i32" | "u32")
        )
    }

    /// Plan 012 Batch A: forward ts_adapter passthrough notes into the
    /// unified codegen warning channel (R010, advisory).
    pub(super) fn drain_ctx_warnings(&self, ctx: &crate::ui_gen::ts_adapter::AuraTsContext) {
        for w in ctx.take_warnings() {
            self.warn("R010", crate::ui_gen::validators::Severity::Info, w);
        }
    }

    /// Plan 012 Batch A (gap 19 audit): expr_to_js-side receiver classifier
    /// for the `.contains → .includes` gate, mirroring
    /// `ts_adapter::method_map_decision` but driven by the generator's TS
    /// type maps (state_types/prop_types).
    pub(super) fn method_map_decision_for_expr(
        &self,
        object: &auto_lang_core::ast::Expr,
    ) -> crate::ui_gen::ts_adapter::MethodMapDecision {
        use crate::ui_gen::ts_adapter::MethodMapDecision as D;
        let ty_of = |field: &str| {
            self.state_types
                .get(field)
                .or_else(|| self.prop_types.get(field))
        };
        // `.field` / `self.field` receiver.
        if let auto_lang_core::ast::Expr::Dot(obj, field) = object {
            if matches!(obj.as_ref(), auto_lang_core::ast::Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".")
            {
                return match ty_of(field.as_str()) {
                    Some(ty) if ty.ends_with("[]") || ty == "string" => D::Map,
                    Some(_) => D::PassWarn, // typed non-array member (facade)
                    None if self.is_ext_composable_local(field.as_str()) => D::PassWarn,
                    None => D::Map, // unknown member — legacy behavior
                };
            }
        }
        // `store.*` facade chain.
        fn chain_root(object: &auto_lang_core::ast::Expr) -> Option<&str> {
            match object {
                auto_lang_core::ast::Expr::Ident(n) => Some(n.as_str()),
                auto_lang_core::ast::Expr::Dot(obj, _) => chain_root(obj),
                _ => None,
            }
        }
        match chain_root(object) {
            Some("store") => D::PassWarn,
            Some(root) if self.is_ext_composable_local(root) => D::PassWarn,
            _ => D::Map,
        }
    }

    /// Plan 012 Batch A (gap 19): is `name` a `use { composable: ... }`
    /// local (a facade object, e.g. `recentFilesStore`)? Such receivers
    /// never get the `.remove → .splice` mapping.
    pub(super) fn is_ext_composable_local(&self, name: &str) -> bool {
        self.ext_composables
            .iter()
            .any(|(local, _, _)| local == name)
    }

    /// Plan 012 Batch A (gap 19): local names of `use { composable: ... }`
    /// imports, for the ts_adapter facade gate.
    /// Plan 426: does a setup statement reference any forbidden name (state
    /// var / computed / prop)? Flags bare `x` identifiers and self/props-based
    /// field access (`.x` / `self.x` / `props.x`); returns the first hit.
    pub(super) fn stmt_references_any(
        stmt: &auto_lang_core::ast::Stmt,
        names: &std::collections::HashSet<&str>,
    ) -> Option<String> {
        fn walk_expr(
            expr: &auto_lang_core::ast::Expr,
            names: &std::collections::HashSet<&str>,
        ) -> Option<String> {
            use auto_lang_core::ast::Expr;
            match expr {
                Expr::Ident(n) if names.contains(n.as_str()) => Some(n.as_str().to_string()),
                Expr::Dot(obj, field) => {
                    // `.x` (implicit self), `self.x`, `props.x`
                    let base_is_root = matches!(
                        obj.as_ref(),
                        Expr::Ident(n) if n.as_str() == "self" || n.as_str() == "." || n.as_str() == "props"
                    );
                    if base_is_root && names.contains(field.as_str()) {
                        return Some(field.as_str().to_string());
                    }
                    walk_expr(obj, names)
                }
                Expr::Call(call) => walk_expr(&call.name, names).or_else(|| {
                    call.args
                        .args
                        .iter()
                        .find_map(|a| walk_expr(&a.get_expr(), names))
                }),
                Expr::Bina(l, _, r) => walk_expr(l, names).or_else(|| walk_expr(r, names)),
                Expr::Unary(_, e) => walk_expr(e, names),
                Expr::Array(items) => items.iter().find_map(|e| walk_expr(e, names)),
                _ => None,
            }
        }
        match stmt {
            auto_lang_core::ast::Stmt::Expr(expr) => walk_expr(expr, names),
            auto_lang_core::ast::Stmt::Store(store) => walk_expr(&store.expr, names),
            _ => None,
        }
    }

    pub(super) fn facade_local_names(&self) -> std::collections::HashSet<String> {
        let mut names: std::collections::HashSet<String> = self
            .ext_composables
            .iter()
            .map(|(local, _, _)| local.clone())
            .collect();
        // Plan 426: setup bindings share facade-local semantics.
        names.extend(self.setup_locals.iter().cloned());
        names
    }

    /// Plan 408 P12 §10.4: ref 字段标注的 flat map（local name → field set），
    /// 传给 AuraTsContext 让 ts_adapter 在字段访问时注入 `.value`。
    pub(super) fn facade_ref_fields_map(
        &self,
    ) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
        let mut map = self.facade_ref_fields.clone();
        for (binding, fields) in &self.setup_ref_fields {
            map.entry(binding.clone())
                .or_default()
                .extend(fields.iter().cloned());
        }
        map
    }

}

impl VueGenerator {
    /// Extract API function calls from a LogicPayload and track them
    pub(super) fn extract_api_calls_from_payload(&mut self, payload: &LogicPayload) {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                // Plan 132: Extract API calls from raw AST statements
                self.extract_api_calls_from_ast_stmts(stmts);
            }
            LogicPayload::Bytecode(_) => {
                // Bytecode not supported for API call detection
            }
        }
    }

    /// Extract API function calls from raw AST statements (Plan 132)
    pub(super) fn extract_api_calls_from_ast_stmts(&mut self, stmts: &[auto_lang_core::ast::Stmt]) {
        use auto_lang_core::ast::{Expr, Stmt};

        fn walk_expr(expr: &Expr, api_fns: &[&str], used: &mut HashSet<String>) {
            match expr {
                Expr::Call(call) => {
                    let call_name = call
                        .get_name_text_safe()
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_default();
                    if !call_name.is_empty() {
                        if api_fns.contains(&call_name.as_str()) {
                            used.insert(call_name.clone());
                        }
                    } else if let Expr::Dot(obj, method) = call.name.as_ref() {
                        // PLAN-627 (R627-F1): qualified head `api.X(...)` —
                        // register X (same list gate as the bare name) so the
                        // SFC emits the `@/lib/api` import header; without
                        // this the module-form component references the
                        // client fn without importing it (TS2304).
                        if let Expr::Ident(obj_name) = obj.as_ref() {
                            if obj_name.as_str() == "api" && api_fns.contains(&method.as_str()) {
                                used.insert(method.as_str().to_string());
                            }
                        }
                    }
                    // Recurse into args
                    for arg in &call.args.args {
                        walk_expr(&arg.get_expr(), api_fns, used);
                    }
                }
                Expr::Bina(l, _, r) => {
                    walk_expr(l, api_fns, used);
                    walk_expr(r, api_fns, used);
                }
                Expr::Unary(_, e) => walk_expr(e, api_fns, used),
                Expr::Dot(obj, _) => walk_expr(obj, api_fns, used),
                Expr::Array(items) => {
                    for item in items {
                        walk_expr(item, api_fns, used);
                    }
                }
                Expr::Block(body) => {
                    for stmt in &body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                // PLAN-076: closure/lambda bodies also walk — `let run =
                // () => { ... }` in an on-handler stores the closure as the
                // Store expr; without these arms the api call inside the
                // closure body has emission but no '@/lib/api' import line
                // (TS2304; jade search_panel sink first evidence: the
                // debounced run closure's `use back.api: search_pages`
                // call — on-block direct calls green, closure-nested red).
                Expr::Closure(c) => walk_expr(&c.body, api_fns, used),
                Expr::Lambda(l) => {
                    for stmt in &l.body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                _ => {}
            }
        }

        fn walk_stmt(stmt: &Stmt, api_fns: &[&str], used: &mut HashSet<String>) {
            match stmt {
                Stmt::Expr(expr) => walk_expr(expr, api_fns, used),
                Stmt::Store(store) => walk_expr(&store.expr, api_fns, used),
                Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        walk_expr(&branch.cond, api_fns, used);
                        for stmt in &branch.body.stmts {
                            walk_stmt(stmt, api_fns, used);
                        }
                    }
                    // Plan 444 (ash-shell-057 ④): else-branch api calls must
                    // also register for the '@/lib/api' import.
                    if let Some(else_body) = &if_stmt.else_ {
                        for stmt in &else_body.stmts {
                            walk_stmt(stmt, api_fns, used);
                        }
                    }
                }
                Stmt::For(for_) => {
                    walk_expr(&for_.range, api_fns, used);
                    for stmt in &for_.body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                Stmt::Block(body) => {
                    for stmt in &body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                }
                // PLAN-074: try/catch/finally 体同样入扫描——watch 的契约
                // 调用沉入 try 块（DSL ≥ c5b5fecf）后，`_ => {}` 兜底把整段
                // 吞掉，import 行仍不发射（backlinks 首件实证的最后一环）。
                Stmt::Try(t) => {
                    for stmt in &t.body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                    for stmt in &t.catch_body.stmts {
                        walk_stmt(stmt, api_fns, used);
                    }
                    if let Some(finally_body) = &t.finally_body {
                        for stmt in &finally_body.stmts {
                            walk_stmt(stmt, api_fns, used);
                        }
                    }
                }
                _ => {}
            }
        }

        let all_fns = self.all_api_functions();
        let all_fn_refs: Vec<&str> = all_fns.iter().map(|s| s.as_str()).collect();
        for stmt in stmts {
            walk_stmt(stmt, &all_fn_refs, &mut self.api_functions_used);
        }
    }

    /// Plan 053 M5/P5-6: does this statement list call `complete(...)` (the
    /// backend completion fn)? Such handlers get debounced via setTimeout.
    /// Uses the same recursive walk as `extract_api_calls_from_ast_stmts` but
    /// matches the specific bare-call name `complete` (a `use back.api:
    /// complete` import), NOT the full api set — we don't want every
    /// API-calling handler debounced (e.g. run_command on Enter stays eager).
    pub(super) fn stmts_call_complete(stmts: &[auto_lang_core::ast::Stmt]) -> bool {
        use auto_lang_core::ast::{Expr, Stmt};
        fn walk_expr(expr: &Expr) -> bool {
            match expr {
                Expr::Call(call) => {
                    if call
                        .get_name_text_safe()
                        .map(|n| n.as_str() == "complete")
                        .unwrap_or(false)
                    {
                        return true;
                    }
                    call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
                }
                Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
                Expr::Unary(_, e) => walk_expr(e),
                Expr::Dot(obj, _) => walk_expr(obj),
                Expr::Array(items) => items.iter().any(|e| walk_expr(e)),
                Expr::Block(body) => body.stmts.iter().any(|s| walk_stmt(s)),
                _ => false,
            }
        }
        fn walk_stmt(stmt: &Stmt) -> bool {
            match stmt {
                Stmt::Expr(expr) => walk_expr(expr),
                Stmt::Store(store) => walk_expr(&store.expr),
                Stmt::If(if_stmt) => {
                    if_stmt.branches.iter().any(|b| {
                    walk_expr(&b.cond) || b.body.stmts.iter().any(|s| walk_stmt(s))
                })
                // Plan 444 (ash-shell-057 ④): else-branch bodies hide calls
                // from complete()/toast() detection — walk them too.
                || if_stmt.else_.as_ref().map(|e| e.stmts.iter().any(|s| walk_stmt(s))).unwrap_or(false)
                }
                Stmt::For(for_) => {
                    walk_expr(&for_.range) || for_.body.stmts.iter().any(|s| walk_stmt(s))
                }
                Stmt::Block(body) => body.stmts.iter().any(|s| walk_stmt(s)),
                _ => false,
            }
        }
        stmts.iter().any(|s| walk_stmt(s))
    }

    /// Plan 444 (ash-shell-057 ④): does this statement list call a VM-only
    /// native (`fs.*` / `File.*`)? Mirrors the ts_adapter rewrite so lifecycle
    /// bodies (transpiled AFTER the __vmOnly stub emission point) can flip
    /// `needs_vm_only_helper` ahead of time.
    pub(super) fn stmts_call_vm_only(stmts: &[auto_lang_core::ast::Stmt]) -> bool {
        use auto_lang_core::ast::{Expr, Stmt};
        fn walk_expr(expr: &Expr) -> bool {
            match expr {
                Expr::Call(call) => {
                    if let Expr::Dot(obj, _) = call.name.as_ref() {
                        // PLAN-671 ①/Phase 2：与 ts_adapter 对象级 VM-only
                        // 白名单同表（单源 VM_ONLY_OBJECT_NATIVES：fs/File/
                        // image + Env/Process）——lifecycle 体在 stub 发射后
                        // 才转译，须从原始 AST 预检；漏名会落 TS2304。
                        if matches!(obj.as_ref(), Expr::Ident(n) if crate::ui_gen::ts_adapter::VM_ONLY_OBJECT_NATIVES.contains(&n.as_str()))
                        {
                            return true;
                        }
                    }
                    call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
                }
                Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
                Expr::Unary(_, e) => walk_expr(e),
                Expr::Dot(obj, _) => walk_expr(obj),
                Expr::Array(items) => items.iter().any(|e| walk_expr(e)),
                Expr::Index(a, i) => walk_expr(a) || walk_expr(i),
                _ => false,
            }
        }
        fn walk_stmt(stmt: &Stmt) -> bool {
            match stmt {
                Stmt::Expr(expr) => walk_expr(expr),
                Stmt::Store(store) => walk_expr(&store.expr),
                Stmt::If(if_stmt) => {
                    if_stmt
                        .branches
                        .iter()
                        .any(|b| walk_expr(&b.cond) || b.body.stmts.iter().any(|s| walk_stmt(s)))
                        || if_stmt
                            .else_
                            .as_ref()
                            .map(|e| e.stmts.iter().any(|s| walk_stmt(s)))
                            .unwrap_or(false)
                }
                Stmt::For(for_) => {
                    walk_expr(&for_.range) || for_.body.stmts.iter().any(|s| walk_stmt(s))
                }
                Stmt::Block(body) => body.stmts.iter().any(|s| walk_stmt(s)),
                Stmt::Try(t) => {
                    t.body.stmts.iter().any(|s| walk_stmt(s))
                        || t.catch_body.stmts.iter().any(|s| walk_stmt(s))
                        || t.finally_body
                            .as_ref()
                            .map(|fb| fb.stmts.iter().any(|s| walk_stmt(s)))
                            .unwrap_or(false)
                }
                _ => false,
            }
        }
        stmts.iter().any(|s| walk_stmt(s))
    }

    /// Does this statement list call `toast(...)` (vue-sonner)? If so the
    /// generated <script setup> needs `import { toast } from 'vue-sonner'`.
    /// Same recursive walk as `stmts_call_complete`.
    pub(super) fn stmts_call_toast(stmts: &[auto_lang_core::ast::Stmt]) -> bool {
        use auto_lang_core::ast::{Expr, Stmt};
        fn walk_expr(expr: &Expr) -> bool {
            match expr {
                Expr::Call(call) => {
                    // toast() → name is Ident("toast")
                    // toast.success()/error()/info() → name is Dot(Ident("toast"), method)
                    // (get_name_text_safe returns None for Dot, so check the Expr directly)
                    let is_toast = match call.name.as_ref() {
                        Expr::Ident(n) => n.as_str() == "toast",
                        Expr::Dot(obj, _) => {
                            matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "toast")
                        }
                        _ => false,
                    };
                    if is_toast {
                        return true;
                    }
                    call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
                }
                Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
                Expr::Unary(_, e) => walk_expr(e),
                Expr::Dot(obj, _) => walk_expr(obj),
                Expr::Array(items) => items.iter().any(|e| walk_expr(e)),
                Expr::Block(body) => body.stmts.iter().any(|s| walk_stmt(s)),
                _ => false,
            }
        }
        fn walk_stmt(stmt: &Stmt) -> bool {
            match stmt {
                Stmt::Expr(expr) => walk_expr(expr),
                Stmt::Store(store) => walk_expr(&store.expr),
                Stmt::If(if_stmt) => {
                    if_stmt.branches.iter().any(|b| {
                    walk_expr(&b.cond) || b.body.stmts.iter().any(|s| walk_stmt(s))
                })
                // Plan 444 (ash-shell-057 ④): else-branch bodies hide calls
                // from complete()/toast() detection — walk them too.
                || if_stmt.else_.as_ref().map(|e| e.stmts.iter().any(|s| walk_stmt(s))).unwrap_or(false)
                }
                Stmt::For(for_) => {
                    walk_expr(&for_.range) || for_.body.stmts.iter().any(|s| walk_stmt(s))
                }
                Stmt::Block(body) => body.stmts.iter().any(|s| walk_stmt(s)),
                _ => false,
            }
        }
        stmts.iter().any(|s| walk_stmt(s))
    }

    /// Check if a handler payload contains API calls
    pub(super) fn handler_has_api_calls(&self, payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                if self.project_api_functions.is_empty() {
                    crate::ui_gen::ts_adapter::stmts_contain_api_call(stmts)
                } else {
                    crate::ui_gen::ts_adapter::stmts_contain_api_call_with(
                        stmts,
                        &self.project_api_functions,
                    )
                }
            }
            LogicPayload::Bytecode(_) => false,
        }
    }

    /// Scan handler-body statements for sibling-handler self-method calls
    /// (`.X()` / `self.X()`), returning the called method names. Mirrors the
    /// ts_adapter self-call recognition (`Expr::Dot(Ident "."/"self", method)`).
    /// Caller MUST gate results against declared handler names — `.save()` /
    /// `.push()` / `.contains()` etc. share the self-call shape but are
    /// store/builtin calls, not handlers (`mark_self_called_handlers` gates).
    pub(super) fn collect_self_handler_calls(stmts: &[auto_lang_core::ast::Stmt]) -> Vec<String> {
        use auto_lang_core::ast::{Expr, Stmt};
        fn is_self_receiver(object: &Expr) -> bool {
            matches!(object, Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self")
        }
        fn scan_expr(expr: &Expr, out: &mut Vec<String>) {
            match expr {
                Expr::Call(call) => {
                    if let Expr::Dot(object, method) = call.name.as_ref() {
                        if is_self_receiver(object) {
                            out.push(method.as_str().to_string());
                        }
                    }
                    scan_expr(&call.name, out);
                    for arg in &call.args.args {
                        scan_expr(&arg.get_expr(), out);
                    }
                }
                Expr::Dot(object, _) => scan_expr(object, out),
                Expr::Bina(l, _, r) => {
                    scan_expr(l, out);
                    scan_expr(r, out);
                }
                Expr::Unary(_, e) => scan_expr(e, out),
                Expr::Array(items) | Expr::Tuple(items) => {
                    for item in items {
                        scan_expr(item, out);
                    }
                }
                Expr::Index(arr, idx) => {
                    scan_expr(arr, out);
                    scan_expr(idx, out);
                }
                Expr::NullCoalesce(l, r) => {
                    scan_expr(l, out);
                    scan_expr(r, out);
                }
                Expr::Cast { expr, .. } | Expr::To { expr, .. } => scan_expr(expr, out),
                Expr::Block(body) => {
                    for s in &body.stmts {
                        scan_stmt(s, out);
                    }
                }
                Expr::If(if_stmt) => scan_if(if_stmt, out),
                _ => {}
            }
        }
        fn scan_if(if_stmt: &auto_lang_core::ast::If, out: &mut Vec<String>) {
            for branch in &if_stmt.branches {
                scan_expr(&branch.cond, out);
                for s in &branch.body.stmts {
                    scan_stmt(s, out);
                }
            }
            if let Some(else_body) = &if_stmt.else_ {
                for s in &else_body.stmts {
                    scan_stmt(s, out);
                }
            }
        }
        fn scan_stmt(stmt: &Stmt, out: &mut Vec<String>) {
            match stmt {
                Stmt::Expr(expr) => scan_expr(expr, out),
                Stmt::Store(store) => scan_expr(&store.expr, out),
                Stmt::If(if_stmt) => scan_if(if_stmt, out),
                Stmt::For(for_) => {
                    scan_expr(&for_.range, out);
                    for s in &for_.body.stmts {
                        scan_stmt(s, out);
                    }
                }
                Stmt::Try(t) => {
                    for s in &t.body.stmts {
                        scan_stmt(s, out);
                    }
                    for s in &t.catch_body.stmts {
                        scan_stmt(s, out);
                    }
                    if let Some(fb) = &t.finally_body {
                        for s in &fb.stmts {
                            scan_stmt(s, out);
                        }
                    }
                }
                Stmt::Block(body) => {
                    for s in &body.stmts {
                        scan_stmt(s, out);
                    }
                }
                Stmt::Return(e) | Stmt::Reply(e) => scan_expr(e, out),
                _ => {}
            }
        }
        let mut out = Vec::new();
        for stmt in stmts {
            scan_stmt(stmt, &mut out);
        }
        out
    }

    /// Mark handlers as used when invoked via `.X()` / `self.X()` from the body
    /// of another *used* handler. Runs to fixpoint so transitive chains
    /// (A -> B -> C) are fully captured. Only targets that are actually
    /// declared handlers in this widget are marked — this filters out
    /// store/builtin calls (`.save()`, `.push()`, `.new()`, ...) that share the
    /// self-call shape. Replaces the manual `expose { .X }` workaround.
    pub(super) fn mark_self_called_handlers(&mut self, widget: &AuraWidget) {
        use std::collections::{HashMap, HashSet};
        let mut calls_by_handler: HashMap<String, Vec<String>> = HashMap::new();
        let mut declared: HashSet<String> = HashSet::new();
        for (pattern, payload) in &widget.handlers {
            let fn_name = self.pattern_to_handler_name(pattern);
            let calls = match payload {
                LogicPayload::AstStmts(stmts) => Self::collect_self_handler_calls(stmts),
                LogicPayload::Bytecode(_) => Vec::new(),
            };
            declared.insert(fn_name.clone());
            calls_by_handler.insert(fn_name, calls);
        }
        let mut changed = true;
        while changed {
            changed = false;
            let snapshot: Vec<String> = self.used_handlers.iter().cloned().collect();
            for used_name in &snapshot {
                if let Some(targets) = calls_by_handler.get(used_name) {
                    for target in targets {
                        // Quoted msg variants ("update:open") arrive raw from
                        // the AST; `declared`/`used_handlers` hold the
                        // sanitized fn names (identity for plain idents).
                        let target = Self::sanitize_ident(target);
                        if declared.contains(&target) && self.used_handlers.insert(target) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    /// Convert prop value to HTML attribute value
    /// For static values: produces `"value"`
    /// For dynamic values (StateRef, FieldAccess): produces `"name"` (caller must prefix with `:`)
    pub(super) fn prop_to_attr_value(&self, value: &AuraPropValue) -> GenResult<String> {
        use auto_lang_core::ast::Expr;
        if let Some(state_ref) = self.extract_state_ref(value) {
            return Ok(format!("\"{}\"", state_ref));
        }
        match value {
            AuraPropValue::Expr(expr) => match expr {
                Expr::Ident(name) => {
                    let resolved = if name.starts_with('.') {
                        &name[1..]
                    } else {
                        name.as_str()
                    };
                    Ok(format!("\"{}\"", resolved))
                }
                Expr::Dot(object, field) => {
                    let obj_str = self.expr_to_vue_text(object)?;
                    let clean_obj = obj_str.trim().trim_matches(|c| c == '{' || c == '}').trim();
                    if clean_obj == "self" || clean_obj == "." || clean_obj.is_empty() {
                        Ok(format!("\"{}\"", field))
                    } else {
                        Ok(format!("\"{}.{}\"", clean_obj, field))
                    }
                }
                _ => Ok(format!("\"{}\"", self.expr_to_vue_text(expr)?)),
            },
            AuraPropValue::StyleBinding(_) => {
                // Class bindings are handled separately in extract_classes
                Ok("\"\"".to_string())
            }
        }
    }

    /// Convert prop value to text content (for rendering inside element)
    pub(super) fn prop_to_text_content(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => self.expr_to_vue_text(expr),
            AuraPropValue::StyleBinding(_) => Ok("".to_string()),
        }
    }

    /// Convert AuraExpr to Vue template text (handles interpolation)
    /// Convert AuraExpr to raw Vue text (no {{ }} wrapping).
    /// Used internally by expr_to_vue_text for composing nested expressions.
    pub(super) fn expr_to_vue_text_raw(&self, expr: &auto_lang_core::ast::Expr) -> GenResult<String> {
        use auto_lang_core::ast::Expr;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => {
                let vue = self.convert_template_to_vue(s.as_str());
                // PLAN-671 附a（626 vm 侧同型的 vue 残留）：strip 仅当
                // 整串恰一段插值（产物供属性绑定等裸表达式消费位复用）。
                // 多段模板（`"${a}:${b}"`）转出的 `{{ a }}:{{ b }}` 外层
                // 界符是内容本身——剥掉即发射残缺（auto-edit StatusBar
                // `store.line }}:{{ store.col` 实案）。
                let vue = vue
                    .strip_prefix("{{ ")
                    .and_then(|v| v.strip_suffix(" }}"))
                    .filter(|inner| !inner.contains("{{") && !inner.contains("}}"))
                    .map(|v| v.to_string())
                    .unwrap_or(vue);
                Ok(vue)
            }
            Expr::Int(n) => Ok(n.to_string()),
            Expr::I8(n) => Ok((*n as i32).to_string()),
            Expr::U8(n) => Ok((*n as i32).to_string()),
            Expr::Byte(n) => Ok((*n as i32).to_string()),
            Expr::Uint(n) => Ok((*n as i32).to_string()),
            Expr::Char(c) => Ok((*c as i32).to_string()),
            Expr::I64(n) => Ok(n.to_string()),
            Expr::U64(n) => Ok((*n as i64).to_string()),
            Expr::Float(f, _) | Expr::Double(f, _) => Ok(f.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') {
                    &name[1..]
                } else {
                    name.as_str()
                };
                Ok(resolved.to_string())
            }
            Expr::Dot(object, field) => {
                // Skip `self.` prefix — the view parser turns `.field` into
                // `self.field`, but Vue <script setup> has no `self`.
                if let Expr::Ident(name) = object.as_ref() {
                    if name == "self" {
                        return Ok(field.to_string());
                    }
                }
                let object_str = self.expr_to_vue_text_raw(object)?;
                // plan 013 follow-up: binop/unary receiver keeps its parens.
                let object_str = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                    format!("({})", object_str)
                } else {
                    object_str
                };
                // Plan 043: numeric field (tuple index, e.g. `field.0`) → `field[0]`
                // for valid TypeScript in Vue templates.
                if field.as_str().chars().all(|c| c.is_ascii_digit()) && !field.as_str().is_empty()
                {
                    Ok(format!("{}[{}]", object_str, field))
                } else {
                    Ok(format!("{}.{}", object_str, field))
                }
            }
            Expr::Index(target, index) => {
                let target_str = self.expr_to_vue_text_raw(target)?;
                // Plan 492 M3 (族 B): the index is a JS expression position —
                // a Str key must stay quoted (`li['name']`). Routing it through
                // expr_to_vue_text_raw stripped the quotes (its Str arm emits
                // template text), yielding the undefined bare identifier
                // `li[name]` and an empty text node. Reuse the svg-attribute
                // path's bound-value renderer for the index instead.
                let index_str = self.expr_to_vue_bound_value(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            Expr::FStr(fstr) => {
                // f-string as text content: literal parts stay raw,
                // interpolated expressions become {{ }} Vue mustaches.
                // (Previously fell through to the "value" placeholder.)
                let mut out = String::new();
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => out.push_str(s.as_str()),
                        other => {
                            let raw = self.expr_to_vue_text_raw(other)?;
                            out.push_str(&format!("{{{{ {} }}}}", raw));
                        }
                    }
                }
                Ok(out)
            }
            Expr::Bina(left, _, right) => {
                let left_str = self.expr_to_vue_text_raw(left)?;
                let right_str = self.expr_to_vue_text_raw(right)?;
                Ok(format!("{}{}", left_str, right_str))
            }
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let method = method.clone();
                    let obj_str = self.expr_to_vue_text_raw(object)?;
                    // plan 013 follow-up: binop/unary receiver keeps its parens.
                    let obj_str = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                        format!("({})", obj_str)
                    } else {
                        obj_str
                    };
                    let is_self = obj_str == "self";
                    let args: Vec<auto_lang_core::ast::Expr> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .collect();
                    match method.as_str() {
                        "to_string" => Ok(obj_str.clone()),
                        "len" => Ok(format!("{}.length", obj_str)),
                        // Plan 012 P0#13 follow-up: an unsupported arg form
                        // used to silently become `null`; keep that fallback
                        // but warn R013 instead of propagating a hard error
                        // out of a display-text position.
                        "contains" => Ok(format!(
                            "{}.includes({})",
                            obj_str,
                            args.iter()
                                .map(|a| self.bound_value_or_warn(a, "contains() call arg", "null"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                        _ => {
                            let args_str: Vec<String> = args
                                .iter()
                                .map(|a| {
                                    self.bound_value_or_warn(
                                        a,
                                        "method-call arg in text position",
                                        "null",
                                    )
                                })
                                .collect();
                            // Plan 053 P5-3: view 文本位置的字符串方法同样走
                            // method map（此前仅 to_string/len/contains 特判，
                            // 其余原样输出 `.to_float()` 等坏 JS）。
                            if let Some(mapped) =
                                self.map_method_to_js(method.as_str(), &obj_str, &args_str)
                            {
                                return Ok(mapped);
                            }
                            if is_self {
                                if args_str.is_empty() {
                                    Ok(format!("{}()", method))
                                } else {
                                    Ok(format!("{}({})", method, args_str.join(", ")))
                                }
                            } else {
                                if args_str.is_empty() {
                                    Ok(format!("{}.{}()", obj_str, method))
                                } else {
                                    Ok(format!("{}.{}({})", obj_str, method, args_str.join(", ")))
                                }
                            }
                        }
                    }
                } else {
                    let name_str = self.expr_to_vue_text_raw(&call.name)?;
                    let args_str: Vec<String> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.bound_value_or_warn(&a, "call arg in text position", "null"))
                        .collect();
                    Ok(format!("{}({})", name_str, args_str.join(", ")))
                }
            }
            // PLAN-604 T08 (KD-VM4): 文本位 as-cast 走绑定位降级(整型
            // Math.trunc),此前落 R046 产 `value` 占位符。
            Expr::Cast { .. } => self.expr_to_vue_bound_value(expr),
            // Plan 492 M3 (族 B): an unsupported expression form in text
            // content used to silently emit the `value` placeholder — the
            // rendered text showed literal "value" with zero diagnostics.
            // Keep the placeholder but raise an R046 warning naming the form.
            _ => {
                self.warn(
                    "R046",
                    crate::ui_gen::validators::Severity::Warning,
                    format!(
                        "unsupported expression in text content position: `{:?}`; emitted `value` placeholder",
                        expr
                    ),
                );
                Ok("value".to_string())
            }
        }
    }

    /// Convert AuraExpr to Vue template text with {{ }} wrapping for display.
    /// Uses expr_to_vue_text_raw internally and wraps the final result.
    pub(super) fn expr_to_vue_text(&self, expr: &auto_lang_core::ast::Expr) -> GenResult<String> {
        use auto_lang_core::ast::Expr;
        // Plan 444 (ash-shell-057 ①c): a binary chain (`text "#" + j.id`) is
        // a JS expression — emit it through the bound-value path (string
        // quoting, operators). The text-raw concat used to drop both,
        // producing `#j.id`.
        if matches!(expr, Expr::Bina(..)) {
            let js = self.expr_to_vue_bound_value(expr)?;
            return Ok(format!("{{{{ {} }}}}", js));
        }
        // For compound expressions that produce their own {{ }},
        // use the raw version and wrap at the end.
        let raw = self.expr_to_vue_text_raw(expr)?;
        // If the raw result already contains {{ (e.g., from convert_template_to_vue),
        // or is a plain literal string / f-string, return as-is.
        // Otherwise wrap in {{ }}.
        let out = if raw.starts_with("{{")
            || matches!(expr, Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
        {
            raw
        } else {
            format!("{{{{ {} }}}}", raw)
        };
        // PLAN-528 W5: 结果是不含插值的纯文本时按 HTML 转义（裸 `<` 会让
        // Vue 模板解析器报 "Element is missing end tag"，整页 500）。
        if out.contains("{{") {
            Ok(out)
        } else {
            Ok(Self::escape_html_text(&out))
        }
    }

    /// Convert AuraExpr to Vue bound attribute value (for :prop="..." bindings).
    /// Used for chart props and other complex bindings where we need JavaScript
    /// expressions in Vue templates (state refs are kept bare, no .value).
    pub(super) fn expr_to_vue_bound_value(&self, expr: &auto_lang_core::ast::Expr) -> GenResult<String> {
        use auto_lang_core::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => Ok(format!("'{}'", Self::escape_js_string(s.as_str()))),
            Expr::Int(n) => Ok(n.to_string()),
            Expr::Float(n, _) | Expr::Double(n, _) => Ok(n.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') {
                    &name[1..]
                } else {
                    name.as_str()
                };
                Ok(resolved.to_string())
            }
            Expr::Dot(object, field) => {
                // State reference: .field or self.field → just the field name
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "." || name.as_str() == "self" {
                        return Ok(field.to_string());
                    }
                }
                let obj_str = self.expr_to_vue_bound_value(object)?;
                // plan 013 follow-up: a binop/unary receiver keeps its
                // parens — `(a + b).x`, not `a + b.x`.
                let obj_str = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                    format!("({})", obj_str)
                } else {
                    obj_str
                };
                // Plan 043: numeric field (tuple index) → bracket form for valid TS.
                if field.as_str().chars().all(|c| c.is_ascii_digit()) && !field.as_str().is_empty()
                {
                    Ok(format!("{}[{}]", obj_str, field))
                } else {
                    Ok(format!("{}.{}", obj_str, field))
                }
            }
            Expr::Index(target, index) => {
                let target_str = self.expr_to_vue_bound_value(target)?;
                let index_str = self.expr_to_vue_bound_value(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            Expr::Array(elems) => {
                let elems_vue: Vec<String> = elems
                    .iter()
                    .map(|e| self.expr_to_vue_bound_value(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_vue.join(", ")))
            }
            Expr::Object(pairs) => {
                let pairs_vue: Vec<String> = pairs
                    .iter()
                    .map(|p| {
                        let v_vue = self.expr_to_vue_bound_value(&p.value)?;
                        // Plan 012 P0#13: quote keys that aren't valid JS
                        // identifiers — `class: { "line-through": .done }`
                        // used to emit `{line-through: done}`, a syntax error.
                        Ok(format!("{}: {}", Self::js_obj_key(&p.key.to_astr()), v_vue))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_vue.join(", ")))
            }
            Expr::Bina(left, op, right) => {
                let mut left_str = self.expr_to_vue_bound_value(left)?;
                let mut right_str = self.expr_to_vue_bound_value(right)?;
                // Re-insert the parens the parser dropped (probe 09).
                if crate::ui_gen::ts_adapter::bina_child_needs_parens(op, left, false) {
                    left_str = format!("({})", left_str);
                }
                if crate::ui_gen::ts_adapter::bina_child_needs_parens(op, right, true) {
                    right_str = format!("({})", right_str);
                }
                let op_str = match op {
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Gt => ">",
                    Op::Le => "<=",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    _ => "+",
                };
                Ok(format!("{} {} {}", left_str, op_str, right_str))
            }
            Expr::Unary(op, operand) => {
                let mut expr_str = self.expr_to_vue_bound_value(operand)?;
                // Keep a Bina operand's grouping under `!` / unary `-`.
                if matches!(operand.as_ref(), Expr::Bina(..)) {
                    expr_str = format!("({})", expr_str);
                }
                match op {
                    Op::Not => Ok(format!("!{}", expr_str)),
                    Op::Sub => Ok(format!("-{}", expr_str)),
                    _ => Ok(format!("!{}", expr_str)),
                }
            }
            Expr::FStr(fstr) => {
                // f-string in a bound position → JS template literal:
                // f"${.top}px" → `${top}px` (needed e.g. for style_obj values).
                let mut out = String::from("`");
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            out.push_str(
                                &s.replace("\\", "\\\\")
                                    .replace("`", "\\`")
                                    .replace("${", "\\${"),
                            );
                        }
                        other => {
                            let v = self.expr_to_vue_bound_value(other)?;
                            out.push_str(&format!("${{{}}}", v));
                        }
                    }
                }
                out.push('`');
                Ok(out)
            }
            // Plan 022: fn 调用作为绑定值（如组件 props: Comp { items: getList(.msg) }）。
            // 复用 expr_to_vue_text_raw:5550-5559 的简单调用模板。此前 Expr::Call 落到
            // _ => "null"，导致 fn 调用 prop 被丢弃（getQuestions(.msg) → null）。
            Expr::Call(call) => {
                // Plan 053 P5-3: 方法调用（view fn / 属性绑定表达式）走共享
                // method map。此前 Expr::Call 把 Dot(name) 整体当作普通函数名
                // 输出 `{object}.{method}({args})`，字符串方法原样输出坏 JS
                // （实测 BlockBody 产物 `field[1].Text.to_float()`）。
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let method = method.clone();
                    let is_self = matches!(object.as_ref(), Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self");
                    let object_js = self.expr_to_vue_bound_value(object)?;
                    // plan 013 follow-up: binop/unary receiver keeps its parens.
                    let object_js = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                        format!("({})", object_js)
                    } else {
                        object_js
                    };
                    let args_js: Vec<String> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_vue_bound_value(&a))
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(mapped) =
                        self.map_method_to_js(method.as_str(), &object_js, &args_js)
                    {
                        return Ok(mapped);
                    }
                    // 保留既有行为:自调用 `.Method()` → `Method()`(无 self 前缀);
                    // 普通方法调用原样输出。
                    if is_self {
                        return Ok(format!("{}({})", method, args_js.join(", ")));
                    }
                    return Ok(format!("{}.{}({})", object_js, method, args_js.join(", ")));
                }
                // Plain function call (e.g. component prop: getList(.msg)) →
                // emit as-is.
                let name_str = self.expr_to_vue_bound_value(&call.name)?;
                let args_str: Vec<String> =
                    call.args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_vue_bound_value(&a))
                        .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}({})", name_str, args_str.join(", ")))
            }
            // Plan 012 P0#13: a ternary `cond ? "a" : "b"` parses as Expr::If.
            // In a bound position (e.g. a `class:` array element) it used to
            // hit the catch-all below and emit literal `null` — silently
            // broken output. Emit the string ternary instead.
            Expr::If(if_stmt) => Ok(self.if_expr_to_style_ternary(if_stmt)),
            // Literal-ish forms the old catch-all happened to render
            // correctly — keep them on the Ok path so hardening the catch-all
            // doesn't turn correct output into spurious R013 warnings.
            Expr::Nil | Expr::Null | Expr::None => Ok("null".to_string()),
            Expr::Uint(n) => Ok(n.to_string()),
            Expr::I8(n) => Ok(n.to_string()),
            Expr::U8(n) => Ok(n.to_string()),
            Expr::I64(n) => Ok(n.to_string()),
            Expr::U64(n) => Ok(n.to_string()),
            Expr::Byte(n) => Ok(n.to_string()),
            Expr::Char(c) => Ok(format!("'{}'", Self::escape_js_string(&c.to_string()))),
            // PLAN-604 T08 (KD-VM4): 绑定位 as-cast——整型目标降级
            // Math.trunc 对齐 VM TYPE_CAST_I32 的 `f as i32` 截断语义
            // (此前落 catch-all 硬错误 UnsupportedExpr)。
            Expr::Cast { expr, target_type } => {
                let mut inner = self.expr_to_vue_bound_value(expr)?;
                if matches!(expr.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                    inner = format!("({})", inner);
                }
                match target_type {
                    auto_lang_core::ast::Type::Int
                    | auto_lang_core::ast::Type::I64
                    | auto_lang_core::ast::Type::Uint
                    | auto_lang_core::ast::Type::U64
                    | auto_lang_core::ast::Type::USize
                    | auto_lang_core::ast::Type::Byte => Ok(format!("Math.trunc({})", inner)),
                    _ => Ok(inner),
                }
            }
            // Plan 012 P0#13 follow-up: everything else (Lambda, Closure,
            // Range, NullCoalesce, Cast, Block, patterns, ...) used to emit
            // literal `null` with no diagnostic. Reject instead; each call
            // site decides between propagating the hard error and warning
            // R013 + falling back.
            _ => Err(GenError::UnsupportedExpr(format!(
                "bound-value position does not support expression form {:?}",
                expr
            ))),
        }
    }

    /// Plan 012 P0#13 follow-up: render a bound value, but on an unsupported
    /// expression form emit an R013 warning (with the expression's Debug
    /// shape and the calling context) and return the caller's fallback —
    /// the exact output the old silent `null` catch-all produced there.
    pub(super) fn bound_value_or_warn(
        &self,
        expr: &auto_lang_core::ast::Expr,
        context: &str,
        fallback: &str,
    ) -> String {
        match self.expr_to_vue_bound_value(expr) {
            Ok(v) => v,
            Err(e) => {
                self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!("{}: {}; emitted `{}` fallback", context, e, fallback),
                );
                fallback.to_string()
            }
        }
    }
    /// Convert template string with ${...} placeholders to Vue {{ ... }} interpolation
    pub(super) fn convert_template_to_vue(&self, template: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = template.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for ${ pattern
            if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
                // Find the closing }
                let start = i + 2;
                let mut depth = 1;
                let mut end = start;

                while end < chars.len() && depth > 0 {
                    if chars[end] == '{' {
                        depth += 1;
                    } else if chars[end] == '}' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        end += 1;
                    }
                }

                if depth == 0 {
                    // Extract the expression inside ${...}
                    let expr: String = chars[start..end].iter().collect();
                    // Convert to Vue interpolation
                    let vue_expr = self.convert_template_expr_to_vue(&expr);
                    result.push_str(&format!("{{{{ {} }}}}", vue_expr));
                    i = end + 1;
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// Convert a template expression (inside ${...}) to Vue expression
    pub(super) fn convert_template_expr_to_vue(&self, expr: &str) -> String {
        let expr = expr.trim();

        // Handle state reference: .field -> field
        if expr.starts_with('.') {
            return expr[1..].to_string();
        }

        // Handle nested field access patterns like (dot (name user).name)
        // These come from the f-string parser's debug format
        if expr.starts_with('(') {
            return self.parse_s_expr_to_vue(expr);
        }

        // Handle simple field access: user.name
        if expr.contains('.') && !expr.starts_with('.') {
            return expr.to_string();
        }

        expr.to_string()
    }

    /// Parse S-expression format from f-string parser to Vue expression
    pub(super) fn parse_s_expr_to_vue(&self, expr: &str) -> String {
        // Handle (dot (name user).field) -> user.field
        // Handle (dot (name user).id) -> user.id
        if let Some(inner) = expr.strip_prefix("(dot ") {
            // Find the object expression and the field
            // Format: (dot <object>.<field>)
            // Example: (dot (name user).id) means user.id

            // Find where the object ends and field begins
            // Look for the pattern: ).<field>)
            if let Some(dot_pos) = inner.rfind('.') {
                // Everything before the dot is the object expression
                let obj_expr = &inner[..dot_pos];
                // Everything after the dot (and before the final ')') is the field
                let field = inner[dot_pos + 1..].trim_end_matches(')').trim();

                // Parse the object expression
                let obj_name = if obj_expr.starts_with("(name ") {
                    // (name user) -> user
                    obj_expr[6..].trim().trim_end_matches(')').to_string()
                } else {
                    self.parse_s_expr_to_vue(obj_expr)
                };

                return format!("{}.{}", obj_name, field);
            }
        }

        // Handle (name user) -> user
        if let Some(inner) = expr.strip_prefix("(name ") {
            return inner.trim_end_matches(')').trim().to_string();
        }

        // Fallback: return as-is
        expr.to_string()
    }

    // ========================================================================
    // shadcn-vue Component-specific Prop Handling
    // ========================================================================

    /// Generate shadcn-vue component attributes based on element type
    /// Returns: (attributes, text_content, generated_children_html)
    /// Pass-through for arbitrary HTML attributes on layout primitives (row/col)
    /// whose `generate_shadcn_attrs` arms only handle class. Emits any prop that
    /// isn't class/style/gap/text/style_obj/show/ref/html (handled elsewhere) as a
    /// v-bind expression (`:key="value"`) for Expr values or a static literal
    /// (`key="value"`) for string literals. This lets `row { draggable: "true",
    /// ondragstart: ... }` emit `:draggable="true"` alongside the flex class.
    /// Called from within `generate_shadcn_attrs` (non-Result context), so expr
    /// conversion failures are silently skipped (matching show/style_obj above).
    /// PLAN-781 T-04: `scroll-anchor` prop → `data-scroll-anchor` 寻址锚
    ///（静态字面量 → 静态 attr；动态表达式（for 循环键等）→ `:data-scroll-
    /// anchor` v-bind 绑定）。空/类型不可解析 = R013 告警不发射——与 iced
    /// 侧 maybe_wrap_scroll_anchor 的 stderr 告警语义同形（不失效页面）。
    pub(super) fn push_scroll_anchor_attr(&self, attrs: &mut Vec<String>, value: &AuraPropValue, tag: &str) {
        let unresolvable = |this: &Self| {
            this.warn(
                "R013",
                crate::ui_gen::validators::Severity::Warning,
                format!(
                    "element `{}` prop `scroll-anchor`: empty or unresolvable key ignored",
                    tag
                ),
            );
        };
        match value {
            AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                if s.is_empty() {
                    unresolvable(self);
                } else {
                    attrs.push(format!(
                        "data-scroll-anchor=\"{}\"",
                        Self::escape_html_attr(s.as_str())
                    ));
                }
            }
            AuraPropValue::Expr(expr) => match self.expr_to_vue_bound_value(expr) {
                Ok(bound) => attrs.push(format!(":data-scroll-anchor=\"{}\"", bound)),
                Err(_) => unresolvable(self),
            },
            _ => unresolvable(self),
        }
    }

    pub(super) fn push_passthrough_attrs(
        &self,
        attrs: &mut Vec<String>,
        props: &HashMap<String, AuraPropValue>,
    ) {
        {
            let mut sorted_pr: Vec<(&String, &AuraPropValue)> = props.iter().collect();
            // Plan 015 P0#3: sorted iteration (props is a HashMap, jade gap 56)
            sorted_pr.sort_by(|a, b| a.0.cmp(b.0));
            for (key, value) in sorted_pr {
                if matches!(
                    key.as_str(),
                    "class" | "style" | "gap" | "text" | "style_obj" | "show" | "ref" | "html"
                // Plan 412 §4.3: square 的尺寸 props 已转成 h-/w- 类,且 h/w/size
                // 不是有效 HTML 属性 —— 不透传。
                | "h" | "w" | "size"
                // PLAN-781 T-04: 已在 generate_shadcn_attrs 顶部单点转发的
                // 寻址锚——透传臂不重发（同元素双属性）。
                | "scroll-anchor"
                ) {
                    continue;
                }
                // PLAN-682 F2：显式 `key:` 已在 generate_shadcn_attrs 顶部单点
                // 转发为 `:key` 绑定；layout 臂（row/col）的透传不再重发——
                // 同元素双 `:key` 属性会被 Vue 编译器 X_DUPLICATE_ATTRIBUTE
                // 拒收（bps-gallery DataTableCrud 行 div 实证）。attrs 尚无
                // `:key=` 时维持透传（兜住顶部转发失败的表情态）。
                if key == "key" && attrs.iter().any(|a| a.starts_with(":key=")) {
                    continue;
                }
                match value {
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                        attrs.push(format!(
                            "{}=\"{}\"",
                            key,
                            Self::escape_js_string(s.as_str())
                        ));
                    }
                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                        let resolved = if name.starts_with('.') {
                            &name[1..]
                        } else {
                            name.as_str()
                        };
                        attrs.push(format!(":{}=\"{}\"", key, resolved));
                    }
                    AuraPropValue::Expr(expr) => {
                        if let Ok(value_str) = self.expr_to_vue_bound_value(expr) {
                            attrs.push(format!(":{}=\"{}\"", key, value_str));
                        }
                    }
                    _ => {}
                }
            }
        } // plan-015 sorted-props block
    }

    /// Plan 057 续(富文本输入):textarea 带 highlight/ghost props 时生成叠加层
    /// 包裹 —— 原 prompt_bar.at 手写的 overlay 结构(相对定位容器 + 绝对定位
    /// spans 层 + text-transparent textarea)下沉为 codegen 职责。无新 props
    /// 或引用解析失败时返回 None(走通用发射路径)。
    pub(super) fn textarea_rich_overlay(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        attr_str: &str,
        indent: usize,
    ) -> GenResult<Option<String>> {
        if tag != "textarea" {
            return Ok(None);
        }
        let hl_ref = props
            .get("highlight")
            .and_then(|v| self.extract_state_ref(v));
        let ghost_ref = props.get("ghost").and_then(|v| self.extract_state_ref(v));
        if hl_ref.is_none() && ghost_ref.is_none() {
            return Ok(None);
        }
        let ind = "  ".repeat(indent);
        let ind1 = "  ".repeat(indent + 1);
        let ind2 = "  ".repeat(indent + 2);
        // textarea 文字透明 + 光标可见(叠加层负责显示彩色文字)。
        let ta_attrs = if attr_str.contains("class=\"") {
            attr_str.replacen("class=\"", "class=\"text-transparent caret-foreground ", 1)
        } else {
            format!("{} class=\"text-transparent caret-foreground\"", attr_str)
        };
        // kind → Tailwind class 链 — 与旧 .at 手写 overlay 完全一致(Pixel 等价)。
        let span_class = "span.kind == 'Command' ? 'text-emerald-400 font-bold' : span.kind == 'ExternalCmd' ? 'text-sky-300' : span.kind == 'String' ? 'text-amber-300' : span.kind == 'Variable' ? 'text-red-400' : span.kind == 'Operator' ? 'text-pink-400 font-bold' : span.kind == 'Redirect' ? 'text-gray-400' : span.kind == 'Comment' ? 'text-gray-400 italic' : span.kind == 'Flag' ? 'text-purple-300' : 'text-gray-200'";
        let mut html = format!("{ind}<div class=\"flex flex-row relative flex-1 items-center\">\n");
        html.push_str(&format!(
            "{ind1}<div class=\"flex flex-row pointer-events-none absolute inset-0 items-center text-sm font-mono-ash whitespace-pre overflow-hidden\">\n"
        ));
        if let Some(hl) = &hl_ref {
            html.push_str(&format!(
                "{ind2}<span v-for=\"(span, i) in {hl}\" :key=\"i\" :class=\"{span_class}\">{{{{ span.text }}}}</span>\n"
            ));
        }
        if let Some(ghost) = &ghost_ref {
            html.push_str(&format!(
                "{ind2}<span class=\"text-muted-foreground/35\">{{{{ {ghost} }}}}</span>\n"
            ));
        }
        html.push_str(&format!("{ind1}</div>\n"));
        html.push_str(&format!("{ind1}<textarea{ta_attrs}></textarea>\n"));
        html.push_str(&format!("{ind}</div>\n"));
        Ok(Some(html))
    }

    /// PLAN-493: textarea mentions 能力——backdrop div + 透明 textarea
    /// **兄弟对**（Vue 端实现技法，随 codegen 下沉）。与 musk 迁移前的
    /// 手写结构等价（backdrop 在前、同容器直接子节点）；**不包** wrapper
    /// ——absolute 子件包进无固有高度的 wrapper 会塌陷（rich_overlay 的
    /// wrapper 形状是 prompt_bar 专属），锚点 relative 容器归 .at 自理。
    /// backdrop 类自 textarea 类串推导（几何镜像：删输入面专属 token，
    /// text-transparent 位换 text-foreground，增背板结构 token）。
    /// `__autoMentionHtml` helper 随 script 一次发射（escape + @\w+ 扫描 +
    /// span 包裹 + 尾随 \n，无 display 替换——对齐 VM 段计算契约）。
    /// 无 mentions，或 value/mentions 非裸 state ref → None（走通用发射）。
    pub(super) fn textarea_mentions_overlay(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        attr_str: &str,
        indent: usize,
    ) -> GenResult<Option<String>> {
        if tag != "textarea" || !props.contains_key("mentions") {
            return Ok(None);
        }
        let names = props
            .get("mentions")
            .and_then(|v| self.extract_state_ref(v));
        let value = props.get("value").and_then(|v| self.extract_state_ref(v));
        let (names_ref, value_ref) = match (names.clone(), value.clone()) {
            (Some(n), Some(v)) => (n, v),
            _ => {
                self.warn(
                    "R013",
                    crate::ui_gen::validators::Severity::Warning,
                    format!(
                        "textarea mentions: value 与 mentions 须为裸 state ref \
                         (value={:?}, mentions={:?}); 回退普通 textarea 发射",
                        value, names
                    ),
                );
                return Ok(None);
            }
        };
        let mention_class = props
            .get("mention_class")
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or("text-primary bg-primary/10 rounded-[3px] px-[0.2rem] font-medium")
            .to_string();
        self.mention_helper_needed = true;

        let ind = "  ".repeat(indent);
        let backdrop_class = extract_class_attr(attr_str)
            .map(|cls| derive_mention_backdrop_class(&cls))
            .unwrap_or_else(|| {
                "pointer-events-none overflow-hidden whitespace-pre-wrap break-words text-foreground"
                    .to_string()
            });
        // textarea 本体注入 text-transparent（幂等；caret 色归 .at 自理）。
        let ta_attrs = match extract_class_attr(attr_str) {
            Some(cls) if cls.split_whitespace().any(|t| t == "text-transparent") => {
                attr_str.to_string()
            }
            Some(_) => attr_str.replacen("class=\"", "class=\"text-transparent ", 1),
            None => format!("{} class=\"text-transparent\"", attr_str),
        };
        // mention_class 作单引号 JS 字符串实参（类串不含引号）。
        let html = format!(
            "{ind}<div class=\"{backdrop_class}\" \
             v-html=\"__autoMentionHtml({value_ref}, {names_ref}, '{mention_class}')\"></div>\n\
             {ind}<textarea{ta_attrs}></textarea>\n"
        );
        Ok(Some(html))
    }

}
