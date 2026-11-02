//! PLAN-014 T-04：store/fn-module 责任（store composable/fn module 全量与单文件、theme runtime、router file）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;
use super::library::{emit_preset_dispatch, stream_guard_var};

impl VueGenerator {
    /// Generate a composable singleton `.ts` file for a shared store
    /// (Plan 351 / Design 18). Produces module-level `ref`s + an exported
    /// `useXxxStore()` function returning state refs and action functions.
    pub fn generate_store_composable(store: &crate::aura::AuraStore) -> String {
        Self::generate_store_composable_full(store, &[], &Default::default()).0
    }

    /// Plan 028 M1: generate a standalone fn-module `.ts` file from an .at
    /// file's top-level `fn` declarations (e.g. forge_helpers.at). Each fn
    /// becomes an exported TS function — the single source for pure helpers
    /// shared by widgets (`use { fn: … from "src/front/x.at" }`).
    pub fn generate_fn_module(fns: &[crate::aura::AuraModuleFn]) -> String {
        Self::generate_fn_module_full(fns, &[])
    }

    /// PLAN-037 Phase 5: fn module + its own `use.web` imports. Port files
    /// (ports/<域>.at) declare web bindings and expose wrapper fns; the
    /// generated TS must import the bound symbols before the wrappers use
    /// them. Plan 424: component/composable entries are forwarded via ES
    /// re-export (`export { X } from '...'`) — callers import them from the
    /// port module while the tag registry / auto-call machinery on the
    /// caller side stays identical to a direct reference.
    pub fn generate_fn_module_full(
        fns: &[crate::aura::AuraModuleFn],
        ext_imports: &[auto_ui_dialect::ast::ExtImport],
    ) -> String {
        let mut code = String::new();
        code.push_str(
            "// Auto-generated from .at fn module by AutoUI (Plan 028 M1).
",
        );
        // Plan 424 收口补强:先转译 wrapper fn 体,fn-kind 的 import 行只发射
        // wrapper 实际引用的符号——纯转发端口(零 wrapper)不再产生 unused
        // import,产出对 noUnusedLocals: true 的工程同样成立。export 行恒发
        // (转发本体)。
        let ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(Default::default());
        let mut wrapper_bodies = String::new();
        for mfn in fns {
            wrapper_bodies.push_str(&crate::ui_gen::ts_adapter::transpile_handler_body(
                &mfn.body, &ctx,
            ));
        }
        for imp in ext_imports {
            let names: Vec<&str> = imp.symbols.iter().map(|s| s.as_str()).collect();
            if names.is_empty() {
                continue;
            }
            let specifier = Self::ext_import_specifier(imp.path.as_str());
            match imp.kind {
                // Plan 435 P4:包引用无模块转发语义(包组件按需生成)。
                auto_ui_dialect::ast::ExtImportKind::Package => continue,
                auto_ui_dialect::ast::ExtImportKind::Fn
                | auto_ui_dialect::ast::ExtImportKind::ExplicitFn => {
                    // import:供本模块 wrapper fn 体内引用(仅实际使用的符号);
                    // export:纯转发(Plan 424)——把符号递给调用方。export-from
                    // 不引入局部绑定,两行共存合法。
                    let used: Vec<&str> = names
                        .iter()
                        .copied()
                        .filter(|n| wrapper_bodies.contains(n))
                        .collect();
                    if !used.is_empty() {
                        code.push_str(&format!(
                            "import {{ {} }} from '{}'
",
                            used.join(", "),
                            specifier
                        ));
                    }
                    code.push_str(&format!(
                        "export {{ {} }} from '{}'
",
                        names.join(", "),
                        specifier
                    ));
                }
                // Plan 424: ports 符号转发——component/composable 经端口再导出。
                // `.vue`/`platform:` 源是 default export(调用方直连时也是
                // default import),re-export 需 `default as X` 别名;npm/.ts
                // 源是命名导出,原样转发。
                auto_ui_dialect::ast::ExtImportKind::Component
                    if imp.path.starts_with("platform:") || imp.path.ends_with(".vue") =>
                {
                    let aliased: Vec<String> =
                        names.iter().map(|n| format!("default as {}", n)).collect();
                    code.push_str(&format!(
                        "export {{ {} }} from '{}'
",
                        aliased.join(", "),
                        specifier
                    ));
                }
                auto_ui_dialect::ast::ExtImportKind::Component
                | auto_ui_dialect::ast::ExtImportKind::Composable => {
                    code.push_str(&format!(
                        "export {{ {} }} from '{}'
",
                        names.join(", "),
                        specifier
                    ));
                }
            }
        }
        if !ext_imports.is_empty() {
            code.push('\n');
        }
        for mfn in fns {
            let param_list = mfn
                .params
                .iter()
                .map(|p| format!("{}: any", p))
                .collect::<Vec<_>>()
                .join(", ");
            let body = crate::ui_gen::ts_adapter::transpile_handler_body(&mfn.body, &ctx);
            // Plan 029 T13: fn 模块体内含 await（Http./Sse./back.api 的双
            // await 展开）时必须标 async——与 store action 的 is_async 同
            // 启发式（body.contains("await")）。返回注解相应省略，否则
            // `async fn(): any[]` 与实际 Promise<any[]> 类型冲突。
            let is_async = body.contains("await");
            let ret_anno = if is_async {
                String::new()
            } else if mfn.ret_ts.is_empty() {
                String::new()
            } else {
                format!(": {}", mfn.ret_ts)
            };
            let async_kw = if is_async { "async " } else { "" };
            code.push_str(&format!(
                "\nexport {}function {}({}){} {{\n",
                async_kw, mfn.name, param_list, ret_anno
            ));
            for line in body.lines() {
                code.push_str("    ");
                code.push_str(line);
                code.push('\n');
            }
            code.push_str("}\n");
        }
        // PLAN-055: a2ts 对 `for x in a..b` 发射 `range(a, b)`，但 fn 模块
        // 路径直拼 body、从不注入 helper（全量 typescript.rs transpiler 的
        // runtime import 在此不存在）——生成物引用未定义符号（musk
        // estimateTokens 首个消费方，vue-tsc TS2552）。检测到 range( 调用
        // 即注入最小整数区间 helper（半开区间，eq=true 含端点）。
        if code.contains("range(") {
            code.push_str("\nfunction range(start: number, end: number, eq?: boolean): number[] {\n    const out: number[] = [];\n    for (let i = start; i < end; i++) { out.push(i); }\n    if (eq === true) { out.push(end); }\n    return out;\n}\n");
        }
        code
    }

    /// Plan 012 Batch A: `generate_store_composable` + the codegen warnings
    /// raised along the way (R010 method-mapping passthrough notes), so the
    /// unified entry point can surface them through the validation channel.
    pub fn generate_store_composable_full(
        store: &crate::aura::AuraStore,
        use_module_fns: &[crate::aura::AuraModuleFn],
        use_imported_names: &std::collections::HashSet<String>,
    ) -> (String, Vec<crate::ui_gen::validators::ValidationWarning>) {
        use crate::ui_gen::ts_adapter::{transpile_handler_body, AuraTsContext};

        let mut code = String::new();
        if store.watchers.is_empty() {
            code.push_str("import { ref } from 'vue'\n");
        } else {
            code.push_str("import { ref, watch } from 'vue'\n");
        }
        // API functions used in handler bodies (from `use back.api`).
        // Streaming endpoints (~Stream<T>) are consumed via SSE in this composable,
        // not via a fetch client, so api.ts does NOT export them — exclude their
        // names from the import to avoid a dangling TS2305. (Plan 043 stream phase.)
        let stream_fn_names: std::collections::HashSet<&str> = store
            .stream_endpoints
            .iter()
            .map(|ep| ep.fn_name.as_str())
            .collect();
        let importable_fns: Vec<&String> = store
            .api_imports
            .iter()
            .filter(|f| !stream_fn_names.contains(f.as_str()))
            .collect();
        if !importable_fns.is_empty() {
            let fns = importable_fns
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            code.push_str(&format!("import {{ {} }} from '@/lib/api'\n", fns));
        }
        // Plan 559 W2: sibling store facades. Each `use <other>_store: <Name>`
        // referenced by this store's bodies gets a composable import (stores
        // sit in the same stores/ dir → relative path) + a reactive facade
        // const; the ctx's store_facades map below routes qualified call
        // heads (`Collection.Init(...)`) onto it. Self-qualification stays a
        // bare local call via store_facade_from (vm A1 contract, mirrors the
        // regen.sh deploy sed this replaces).
        for sibling in &store.sibling_stores {
            code.push_str(&format!(
                "import {{ use{}Store }} from './use{}Store'\n",
                sibling, sibling
            ));
        }
        if !store.sibling_stores.is_empty() {
            code.push_str("import { reactive } from 'vue'\n");
            for sibling in &store.sibling_stores {
                code.push_str(&format!(
                    "const {} = reactive(use{}Store())\n",
                    facade_var_for(sibling),
                    sibling
                ));
            }
        }
        code.push('\n');

        // Module-level ref declarations (singleton state).
        for sv in &store.state_vars {
            let init = Self::store_init_to_js(&sv.initial);
            // Plan 055 Phase E(a): scalar ref 类型化(str/int/bool 及其数组)。
            // User 类型回退 any(import pass E(e)后做)。
            let ty = Self::store_ref_ts_type(&sv.type_info);
            code.push_str(&format!("const {} = ref<{}>({})\n", sv.name, ty, init));
        }
        code.push('\n');
        let state_alias_insert_at = code.len();

        // Build ctx for handler transpilation (state_names → .value emission).
        let state_names: std::collections::HashSet<String> =
            store.state_vars.iter().map(|s| s.name.clone()).collect();
        // Plan 012 Batch A (gap 19): proven array/string receivers for the
        // .remove/.contains method-mapping gate.
        let mut typed_arrays: std::collections::HashSet<String> = Default::default();
        let mut typed_strings: std::collections::HashSet<String> = Default::default();
        let mut typed_ints: std::collections::HashSet<String> = Default::default();
        for sv in &store.state_vars {
            let ty = Self::auto_type_to_ts_type(&sv.type_info);
            if ty.ends_with("[]") {
                typed_arrays.insert(sv.name.clone());
            }
            if ty == "string" {
                typed_strings.insert(sv.name.clone());
            }
            if Self::is_int_auto_type(&sv.type_info) {
                typed_ints.insert(sv.name.clone());
            }
        }
        // Pass API imports so ts_adapter adds `await` to API calls.
        // Plan 559 W2: self-qualified heads emit bare (store_bare_heads —
        // a composable has no `store` const; the sibling's action is a
        // local fn); sibling store heads map onto their reactive facade
        // consts (store_facades), matching the imports emitted above.
        let mut sibling_facades: std::collections::HashMap<String, String> = Default::default();
        for sibling in &store.sibling_stores {
            sibling_facades.insert(sibling.clone(), facade_var_for(sibling));
        }
        let mut self_bare: std::collections::HashSet<String> = Default::default();
        self_bare.insert(store.name.clone());
        // PLAN-668 R-23（P666-D2）：store 自体调用兼用字面别名
        // `store.X()`（038 minesweeper_store 实案）——同自名限定走裸调
        // 发射；否则 composable 内无 `store` 绑定落 TS2552。
        self_bare.insert("store".to_string());
        let ctx = AuraTsContext::new(state_names)
            .with_state_aliases(
                store
                    .state_vars
                    .iter()
                    .map(|sv| (sv.name.clone(), format!("__auto_state${}", sv.name)))
                    .collect(),
            )
            .with_props(std::collections::HashSet::new())
            .with_api_functions(store.api_imports.clone())
            .with_store_bare_heads(self_bare)
            .with_store_facades(sibling_facades)
            .with_typed_collections(typed_arrays, typed_strings)
            .with_typed_ints(typed_ints);

        // Plan 012 Batch G (gap 12): store-level `watch { ... }` block →
        // module-level Vue watch() calls. The store refs are module-level
        // singletons, so watching them at module scope is valid (the watcher
        // lives for the app lifetime — same shape as the facade-module
        // workaround this replaces). Sources are always store state fields
        // (no props in a store), so they are watched directly; multi-source
        // entries emit the array form. Mirrors the widget-side emission.
        for watcher in &store.watchers {
            let body = match &watcher.payload {
                crate::aura::LogicPayload::AstStmts(stmts) => transpile_handler_body(stmts, &ctx),
                _ => String::new(),
            };
            let source = if watcher.sources.len() == 1 {
                watcher.sources[0].clone()
            } else {
                format!("[{}]", watcher.sources.join(", "))
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
            code.push_str(&format!(
                "watch({}, () => {{\n{}\n}}{})\n\n",
                source, indented, opts
            ));
        }

        // Plan 360: detect accent_color state so we can inject the palette
        // helpers and expose `accent_names` as a computed getter.
        let has_accent = store.state_vars.iter().any(|s| s.name == "accent_color");

        // Plan 043 stream phase: SSE wiring is now TYPE-DRIVEN. When the store
        // declares a streaming endpoint (an `#[api] fn` returning `~Stream<T>`,
        // captured in `stream_endpoints` from `back/api.at`), the composable opens
        // ONE EventSource at that endpoint's path and dispatches SSE messages into
        // the store's actions. This replaces the old name-heuristic (which keyed
        // off the literal "stream" import + RunOutput/RunResult action names).
        //
        // Dispatch policy: each SSE message is JSON-parsed and routed to the
        // store action(s) whose single parameter matches the stream's item type
        // (or, as a pragmatic fallback for externally-tagged unions whose
        // `event` discriminator names an action, by that discriminator). For the
        // ash-gui ShellEvent contract (`{"event":"command_output"|"command_result"}`),
        // the discriminator-route keeps the existing RunOutput/RunResult mapping.
        //
        // Plan musk-022 Phase 1: dispatch is now fully data-driven. Each endpoint
        // carries its own `discriminator` (JSON field name, e.g. "type" for
        // `#[serde(tag="type")]` enums) and `variants` (wire value → action name
        // map). Multiple endpoints are supported (forge has 3 SSE streams), each
        // with its own per-endpoint module-level connection guard. When
        // `variants` is empty, the legacy `command_output`/`command_result`
        // fallback is emitted for backward compatibility.
        let stream_eps = store.stream_endpoints.clone();
        let wire_sse = !stream_eps.is_empty();

        // Plan musk-022 Phase 4 fix: a streaming endpoint should only wire into
        // a store that actually imports its function (via `use back.api: <fn>`).
        // `store.stream_endpoints` is resolved project-globally from back/api.at,
        // so without this filter EVERY store would get every stream's SSE wiring
        // (e.g. AuthStore picking up chat_stream's Delta/Thinking dispatchers).
        // Filter to endpoints whose fn_name appears in this store's api_imports.
        let active_stream_eps: Vec<crate::aura::StreamEndpoint> = store
            .stream_endpoints
            .iter()
            .filter(|ep| store.api_imports.iter().any(|imp| imp == &ep.fn_name))
            .cloned()
            .collect();
        let wire_sse = wire_sse && !active_stream_eps.is_empty();

        // Export function.
        // PLAN-063 Phase B T15 (KD 061 D13): 归一命名(AuthStore→useAuthStore)。
        let fn_name = store_composable_name(&store.name);
        // Module-level guard: every widget calls reactive(useXxxStore()), so
        // without a flag each call would open its own SSE connection. One guard
        // per endpoint (keyed by path) so multi-endpoint stores don't collapse.
        if wire_sse {
            code.push_str("// Plan musk-022 multi-event SSE: one EventSource per streaming\n");
            code.push_str("// endpoint, dispatched by each variant's wire value.\n");
            for ep in &active_stream_eps {
                let guard = stream_guard_var(&ep.path);
                code.push_str(&format!("let {} = false;\n", guard));
            }
            code.push('\n');
        }
        // Plan 051 C7: store timer 单例旗标（模块级）。
        if !store.timers.is_empty() {
            code.push_str(
                "let __t51_timers_started = false
",
            );
        }
        // Plan 028 F9: `on stream sse(url[, "event"])` subscriptions get the
        // same per-URL module-level guard.
        if !store.stream_handlers.is_empty() {
            code.push_str("// Plan 028 F9: on stream sse(...) subscriptions.\n");
            for sh in &store.stream_handlers {
                let guard = stream_guard_var(&sh.url);
                code.push_str(&format!("let {} = false;\n", guard));
            }
            code.push('\n');
        }
        code.push_str(&format!("export function {}(): any {{\n", fn_name));

        // Plan 043 cat-3: declare actions as const arrow functions BEFORE the
        // return object, so sibling actions can call each other by bare name.
        // Previously actions were inlined as object properties
        // (`Action: (p) => {...}`), making them invisible to other actions'
        // bodies — a cross-call like `.RefreshGit()` emitted `RefreshGit()`
        // which has no binding in scope. Declaring `const RefreshGit = ...`
        // first makes it a closure variable visible to all sibling actions.
        let mut action_names: Vec<String> = Vec::new();
        for (pattern, payload) in &store.handlers {
            let after_dot = pattern.trim_start_matches('.');
            let action_name = match after_dot.find('(') {
                Some(paren) => after_dot[..paren].to_string(),
                None => after_dot.to_string(),
            };
            let mut body = match payload {
                crate::aura::LogicPayload::AstStmts(stmts) => transpile_handler_body(stmts, &ctx),
                _ => String::new(),
            };
            if has_accent {
                if action_name == "SetAccent" {
                    body.push_str("; applyAccent(accent_color.value, dark_mode.value)");
                } else if action_name == "ToggleDarkMode" {
                    body.push_str("; applyAccent(accent_color.value, dark_mode.value)");
                }
            }
            let is_async = body.contains("await");
            // Plan 055 Phase E(b): handler 参数类型化(scalar;User 回退 any)。
            let params = Self::handler_param_types(store, pattern)
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect::<Vec<_>>()
                .join(", ");
            let async_kw = if is_async { "async " } else { "" };
            code.push_str(&format!(
                "    const {} = {}({}) => {{ {} }}\n",
                action_name, async_kw, params, body
            ));
            action_names.push(action_name);
        }

        // Plan 051 C7: store timer 条目——模块级 started 旗标保证单例
        // composable 多次调用只建一条 interval（沿 stream_guard 先例）；
        // `when` 门控在回调内对模块级 ref 求值。
        if !store.timers.is_empty() {
            code.push_str(
                "        if (!__t51_timers_started) {
            __t51_timers_started = true
",
            );
            for t in &store.timers {
                let guard = match t.when.as_deref() {
                    Some(c) => Self::timer_condition_to_js(c),
                    None => "true".to_string(),
                };
                code.push_str(&format!(
                    "            setInterval(() => {{ if ({guard}) {{ {ev}() }} }}, {ms})
",
                    guard = guard,
                    ev = t.event,
                    ms = t.every_ms,
                ));
            }
            code.push_str(
                "        }
",
            );
        }

        // Plan 444 (ash-shell-057 ④): store actions with VM-only native
        // rewrites (`fs.*` / `File.*`) need the shared throwing stub declared
        // in this module (function declarations hoist above the actions).
        if code.contains("__vmOnly(") {
            code.push_str("function __vmOnly(name: string, ...args: any[]): never {\n  throw new Error('[auto-gen] VM-only primitive \"' + name + '\" has no Vue/JS build — this path only runs in VM mode')\n}\n\n");
        }

        // Plan 043 stream phase: wire the SSE stream into the store's
        // command-output/result handlers (see wire_sse above). Must run
        // before `return` so the connection opens on the first composable call.
        // The path comes from the streaming endpoint's `#[api(path=...)]` (no
        // longer hardcoded). Dispatch routes by the SSE event discriminator
        // (`data.<discriminator>`) to actions named after the discriminator's
        // value, matching the externally-tagged-union contract the server emits.
        //
        // Plan musk-022 Phase 1: each endpoint emits its own EventSource block
        // with a per-path guard, and the discriminator field name + variant
        // routing come from `ep.discriminator` / `ep.variants` (resolved from
        // the inner type's `#[serde(tag=...)]` declaration). When `variants`
        // is empty, fall back to the legacy `command_output`/`command_result`
        // pair so existing ash-gui stores keep working unchanged.
        if wire_sse {
            for ep in &active_stream_eps {
                let guard = stream_guard_var(&ep.path);
                code.push_str(&format!("    if (!{}) {{\n", guard));
                code.push_str(&format!("        {} = true;\n", guard));
                code.push_str(&format!(
                    "        const es = new EventSource('{}');\n",
                    ep.path
                ));
                code.push_str("        es.onmessage = (ev) => {\n");
                code.push_str("            try {\n");
                code.push_str("                const data = JSON.parse(ev.data);\n");
                // Build the dispatch if-chain from ep.variants (data-driven), or
                // fall back to the legacy two-variant ash-gui contract. Each
                // entry is (discriminator_field, wire_value, action_name).
                let disc = &ep.discriminator;
                let chain: Vec<(String, String, String)> = if ep.variants.is_empty() {
                    // Legacy fallback: data.event discriminator + RunOutput/RunResult.
                    vec![
                        (
                            "event".to_string(),
                            "command_output".to_string(),
                            "RunOutput".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "command_result".to_string(),
                            "RunResult".to_string(),
                        ),
                        // Plan 055 Phase A.3: 作业控制事件(实时 jobs dispatch)。
                        (
                            "event".to_string(),
                            "job_started".to_string(),
                            "JobStarted".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "job_done".to_string(),
                            "JobDone".to_string(),
                        ),
                        // Plan 063 T4/T5: AI chat 抽屉事件族(无参 handler 读
                        // __ai_* 预置字段,VM 轨由 renderer 直写不经 handler)。
                        (
                            "event".to_string(),
                            "ai_turn".to_string(),
                            "AiTurn".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "ai_chunk".to_string(),
                            "AiChunk".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "ai_tool_call".to_string(),
                            "AiToolCall".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "ai_tool_result".to_string(),
                            "AiToolResult".to_string(),
                        ),
                        (
                            "event".to_string(),
                            "chat_cleared".to_string(),
                            "ChatCleared".to_string(),
                        ),
                    ]
                } else {
                    ep.variants
                        .iter()
                        .map(|(wire, action)| (disc.clone(), wire.clone(), action.clone()))
                        .collect()
                };
                // Plan 051 M2 (②-b 预置字段模拟): ash-gui 的 RunOutput/RunResult
                // 是无参 handler,通过 __sse_* 预置字段读 SSE 数据(VM 限制:不能收
                // struct 参)。VM 模式下由 renderer 填字段;vue 模式下由这段 codegen
                // 模拟同样的模式:先把 data 拆填进 __sse_* ref,再调无参 handler。
                // 检测信号:store 的 state_vars 含 __sse_ 前缀字段(__sse_ 是保留前缀)。
                // 仅对 legacy fallback 的 RunOutput/RunResult 生效;带 variants 的
                // 数据驱动流(如 forge)走原带参透传路径,不受影响。
                let preset_mode = store
                    .state_vars
                    .iter()
                    .any(|s| s.name.starts_with("__sse_"));
                // Plan 065:command_result 后跟随 RefreshContext —— VM 轨由
                // 引擎侧触发(renderer.rs command_result 分支),vue 轨此前无人
                // 补位,store 的 ai_pending/ai_next/ai_steps 拉取链整条不跑
                // (chips/分步行全灭)。仅当 store 声明了 RefreshContext handler
                // 时跟随(无参语义,与引擎触发一致)。
                let follow_refresh_ctx = store.handlers.iter().any(|(p, _)| {
                    p.trim_start_matches('.')
                        .split('(')
                        .next()
                        .map_or(false, |n| n == "RefreshContext")
                });
                for (i, (disc_field, wire_value, action)) in chain.iter().enumerate() {
                    let kw = if i == 0 { "if" } else { "else if" };
                    if preset_mode {
                        emit_preset_dispatch(
                            &mut code,
                            kw,
                            disc_field,
                            wire_value,
                            action,
                            follow_refresh_ctx,
                        );
                    } else {
                        code.push_str(&format!(
                            "                {} (data.{} === '{}') {}(data);\n",
                            kw, disc_field, wire_value, action
                        ));
                    }
                }
                code.push_str("            } catch { }\n");
                code.push_str("        };\n");
                // Plan 054 M4: SSE onerror 自动重连(退避 2s)。后端重启或网络
                // 断开时 readyState 变 CLOSED,重置 per-path guard 并延迟重调
                // composable —— composable 内 `if (!guard)` 会重新 new EventSource。
                // 此前无 onerror,后端一重启前端就永久失联(需手动刷新页面)。
                code.push_str("        es.onerror = () => {\n");
                code.push_str(&format!(
                    "            if (es.readyState === EventSource.CLOSED) {{ {} = false; setTimeout({}, 2000); }}\n",
                    guard, fn_name
                ));
                code.push_str("        };\n");
                code.push_str("    }\n");
            }
        }

        // Plan 028 F9: `on stream sse(url[, "event"]) -> { … }` — open one
        // EventSource per subscription (guarded, auto-reconnecting, same shape
        // as the type-driven wiring above). 已决③: the platform layer parses
        // the SSE data JSON and dispatches the handler with the parsed object,
        // so the .at body branches on `.ev.type` with zero parsing boilerplate.
        // Default (no event filter) → onmessage; named events → addEventListener.
        for sh in &store.stream_handlers {
            let guard = stream_guard_var(&sh.url);
            let handler_key = &sh.handler_key;
            // Action fn name: pattern "__stream_sse_x(ev)" → "__stream_sse_x".
            let after_dot = handler_key.trim_start_matches('.');
            let action_name = match after_dot.find('(') {
                Some(paren) => after_dot[..paren].to_string(),
                None => after_dot.to_string(),
            };
            code.push_str(&format!("    if (!{}) {{\n", guard));
            code.push_str(&format!("        {} = true;\n", guard));
            code.push_str(&format!(
                "        const es = new EventSource('{}');\n",
                sh.url
            ));
            match &sh.event {
                None => {
                    code.push_str(&format!(
                        "        es.onmessage = (ev) => {{\n            try {{\n                {}(JSON.parse(ev.data));\n            }} catch {{ }}\n        }};\n",
                        action_name
                    ));
                }
                Some(event_name) => {
                    code.push_str(&format!(
                        "        es.addEventListener('{}', (ev) => {{\n            try {{\n                {}(JSON.parse((ev as MessageEvent).data));\n            }} catch {{ }}\n        }});\n",
                        event_name, action_name
                    ));
                }
            }
            code.push_str("        es.onerror = () => {\n");
            code.push_str(&format!(
                "            if (es.readyState === EventSource.CLOSED) {{ {} = false; setTimeout({}, 2000); }}\n",
                guard, fn_name
            ));
            code.push_str("        };\n");
            code.push_str("    }\n");
        }

        code.push_str("    return {\n");

        // Expose state refs by name.
        for sv in &store.state_vars {
            code.push_str(&format!("        {},\n", sv.name));
        }
        // Expose actions by reference (declared as const above).
        for name in &action_names {
            code.push_str(&format!("        {},\n", name));
        }

        // Plan 360: when accent_color exists, expose the palette name list as
        // a computed so the UI can render swatch buttons via `for n in .store.accent_names`.
        if has_accent {
            code.push_str("        get accent_names() {\n");
            code.push_str("            return getAccentNames();\n");
            code.push_str("        },\n");
        }

        // Plan 367 P2-2: user-declared computed properties → JS getters.
        // Uses ts_adapter to transpile the expression, with store state_names
        // as context (so .value is appended correctly to module-level refs).
        for computed_prop in &store.computed {
            // Plan 043 store-codegen: a single-expression computed renders as
            // `get name() { return <expr>; }`. A multi-statement block body
            // (Expr::Block) already ends in its own `return`, so we emit the
            // statements directly into the getter body (no wrapping `return`).
            code.push_str(&format!("        get {}() {{\n", computed_prop.name));
            match &computed_prop.expr {
                auto_lang_core::ast::Expr::Block(body) => {
                    let body_js =
                        crate::ui_gen::ts_adapter::transpile_handler_body(&body.stmts, &ctx);
                    // indent each line of the already-transpiled body
                    for line in body_js.lines() {
                        code.push_str("            ");
                        code.push_str(line);
                        code.push('\n');
                    }
                }
                _ => {
                    let mut buf = Vec::new();
                    crate::ui_gen::ts_adapter::transpile_expr_pub(
                        &computed_prop.expr,
                        &ctx,
                        &mut buf,
                    );
                    let expr_js = String::from_utf8_lossy(&buf);
                    code.push_str(&format!("            return {};\n", expr_js.trim()));
                }
            }
            code.push_str("        },\n");
        }

        code.push_str("    }\n");
        code.push_str("}\n");

        // Plan 367 P2-4: emit module-level helper functions from the store file
        // (e.g. `fn format_git_label(...)` declared OUTSIDE the store block).
        // Handlers call them by bare name; without this the generated actions
        // would throw `ReferenceError: <fn> is not defined` at runtime.
        if !store.module_fns.is_empty() {
            for mfn in &store.module_fns {
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
                code.push_str(&format!(
                    "\nfunction {}({}){} {{\n",
                    mfn.name, param_list, ret_anno
                ));
                let body = crate::ui_gen::ts_adapter::transpile_handler_body(&mfn.body, &ctx);
                // Indent every body line for readability (cosmetic only).
                for line in body.lines() {
                    code.push_str("    ");
                    code.push_str(line);
                    code.push('\n');
                }
                code.push_str("}\n");
            }
        }

        // Plan 360: accent color system. When the store declares an
        // `accent_color` state var, inject the 5-color palette + applyAccent
        // function + onMounted bootstrap. The palette is aligned with
        // auto-forge (indigo/coral/ocean/sage/amber).
        if has_accent {
            code.push_str("\n");
            code.push_str(Self::ACCENT_PALETTE_JS);
            // PLAN-601 T-06: full named-theme runtime rides the same gate.
            code.push('\n');
            code.push_str(&Self::theme_runtime_js());
            // Module-level bootstrap: apply saved accent on first import.
            // Also sync the accent_color ref so the store reflects the
            // persisted choice (localStorage may differ from the .at default).
            code.push_str("\n// Restore saved accent on module load.\n");
            // PLAN-601 T-06: restore the active theme first (generator-seeded
            // declared themes / user choice); scaffold default needs no apply.
            code.push_str(
                "(function bootstrapAccent() {\n\
                 \x20 const __th = getActiveTheme()\n\
                 \x20 if (__th !== 'scaffold') applyTheme(__th, document.documentElement.classList.contains('dark'))\n\
                 \x20 const saved = getSavedAccent()\n\
                 \x20 accent_color.value = saved || 'indigo'\n\
                 \x20 const isDark = document.documentElement.classList.contains('dark')\n\
                 \x20 applyAccent(saved || 'indigo', isDark)\n\
                 })()\n",
            );
        }

        // PLAN-671 ③：use 导入 fn 池拉取内联——组件 SFC 的
        // `emit_use_module_fns` 同机制覆盖 store composable。store 体
        //（handler/watcher/computed/module_fn）引用的显式导入裸名按池
        // 闭包拉取，模块级 function 声明（提升语义，位置在文件尾无碍）。
        // 此前 bp helper 只在组件文件内联，store 文件残留裸调（TS2304）。
        let mut pull_warnings: Vec<crate::ui_gen::validators::ValidationWarning> = Vec::new();
        if !use_module_fns.is_empty() {
            use std::collections::{HashMap, HashSet};
            let mut referenced: HashSet<String> = HashSet::new();
            for payload in store.handlers.values() {
                if let crate::aura::LogicPayload::AstStmts(stmts) = payload {
                    collect_called_bare_names_stmts(stmts, &mut referenced);
                }
            }
            for w in &store.watchers {
                if let crate::aura::LogicPayload::AstStmts(stmts) = &w.payload {
                    collect_called_bare_names_stmts(stmts, &mut referenced);
                }
            }
            for c in &store.computed {
                collect_called_bare_names_expr(&c.expr, &mut referenced);
            }
            for mfn in &store.module_fns {
                collect_called_bare_names_stmts(&mfn.body, &mut referenced);
            }
            // 池内按名定位 + 入口门控（仅显式导入名）+ 闭包拉取到不动点。
            let mut pool_position: HashMap<&str, usize> = HashMap::new();
            for (i, mfn) in use_module_fns.iter().enumerate() {
                pool_position.entry(mfn.name.as_str()).or_insert(i);
            }
            let mut pulled_idx: Vec<usize> = Vec::new();
            let mut pulled_names: HashSet<String> = HashSet::new();
            let mut frontier: Vec<String> = referenced
                .into_iter()
                .filter(|n| use_imported_names.contains(n.as_str()))
                .collect();
            while let Some(name) = frontier.pop() {
                if !pulled_names.insert(name.clone()) {
                    continue;
                }
                let Some(&idx) = pool_position.get(name.as_str()) else {
                    continue;
                };
                pulled_idx.push(idx);
                let mut called: HashSet<String> = HashSet::new();
                collect_called_bare_names_stmts(&use_module_fns[idx].body, &mut called);
                for dep in called {
                    if !pulled_names.contains(&dep) && pool_position.contains_key(dep.as_str()) {
                        frontier.push(dep);
                    }
                }
            }
            // 冲突面：本文件已有同名绑定（state ref / action / module fn）
            // 的跳过发射，R013 同款警告形态。
            let mut conflict_names: HashSet<&str> = HashSet::new();
            conflict_names.extend(store.state_vars.iter().map(|s| s.name.as_str()));
            conflict_names.extend(store.handlers.keys().map(|p| {
                let after_dot = p.trim_start_matches('.');
                match after_dot.find('(') {
                    Some(paren) => &after_dot[..paren],
                    None => after_dot,
                }
            }));
            conflict_names.extend(store.module_fns.iter().map(|f| f.name.as_str()));
            pulled_idx.sort_unstable();
            pulled_idx.dedup();
            for idx in pulled_idx {
                let mfn = &use_module_fns[idx];
                if let Some(reason) = use_fn_body_unsupported(&mfn.body) {
                    pull_warnings.push(
                        crate::ui_gen::validators::ValidationWarning::new(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            &store.name,
                            &format!(
                                "use-imported fn `{}` not emitted into the store composable: {} — falls back to no emission",
                                mfn.name, reason
                            ),
                        ),
                    );
                    continue;
                }
                if conflict_names.contains(mfn.name.as_str()) {
                    pull_warnings.push(
                        crate::ui_gen::validators::ValidationWarning::new(
                            "R013",
                            crate::ui_gen::validators::Severity::Warning,
                            &store.name,
                            &format!(
                                "use-imported fn `{}` collides with an existing store symbol (state/action/module fn) — skipped emission",
                                mfn.name
                            ),
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
                code.push_str(&format!(
                    "\n// PLAN-671 ③: use-imported fn inlined (module scope).\nfunction {}({}){} {{\n",
                    mfn.name, param_list, ret_anno
                ));
                // Auto fn 体尾表达式即返回值（VM 侧语义）——与组件路径
                // emit_use_module_fns 同款 return 化转译。
                let body = crate::ui_gen::ts_adapter::transpile_body_as_return(&mfn.body, &ctx);
                for line in body.lines() {
                    code.push_str("    ");
                    code.push_str(line);
                    code.push('\n');
                }
                code.push_str("}\n");
            }
        }

        // PLAN-093 G-12: store composable 路径同样直拼 body、从不注入
        // range helper——本仓 fn 模块路径已有同款注入（PLAN-055），但
        // store 本体动作与 use 导入 fn 内联（PLAN-671 ③）两条路经此发射
        // 时，`for x in a..b` 产出的 range( 调用引用未定义符号（musk
        // canvasSourceRows/canvasTreeVisible，vue-tsc TS2552 实证）。检测
        // 到 range( 即注入与 fn 模块路径同形的最小整数区间 helper。
        if code.contains("range(") {
            code.push_str("\nfunction range(start: number, end: number, eq?: boolean): number[] {\n    const out: number[] = [];\n    for (let i = start; i < end; i++) { out.push(i); }\n    if (eq === true) { out.push(end); }\n    return out;\n}\n");
        }

        // Plan 012 Batch A: drain ts_adapter passthrough notes into the
        // unified validation warning channel (R010, advisory).
        let mut warnings: Vec<crate::ui_gen::validators::ValidationWarning> = ctx
            .take_warnings()
            .into_iter()
            .map(|msg| {
                crate::ui_gen::validators::ValidationWarning::new(
                    "R010",
                    crate::ui_gen::validators::Severity::Info,
                    &store.name,
                    msg,
                )
            })
            .collect();
        warnings.extend(pull_warnings);

        // PLAN-890: only shadowed state actually referenced by handler bodies
        // needs an alias. User locals (including delegated lambda captures)
        // keep their names; ordinary store output and public getters stay put.
        let used_aliases = ctx.used_state_aliases();
        let mut aliases = String::new();
        for sv in &store.state_vars {
            if used_aliases.contains(&sv.name) {
                aliases.push_str(&format!("const __auto_state${} = {}\n", sv.name, sv.name));
            }
        }
        code.insert_str(state_alias_insert_at, &aliases);
        (code, warnings)
    }

    /// Plan 360: JS code injected into the store composable when `accent_color`
    /// state is declared. Defines the 5-color palette, an apply function that
    /// writes the HSL triplet to the `--primary` CSS variable (and persists to
    /// localStorage), and an onMounted bootstrap that restores the saved choice.
    ///
    /// Palette values are aligned with auto-forge's useAccentColor.ts so the
    /// two products share the same visual language.
    /// PLAN-593：5 预设 HSL 值的事实源在 `ui::style::theme::registry::accent_hsl`
    ///（E4/E5 互锁——改值须同步本 JS 文本；dark 提亮双端已归一 +10
    /// 〔PLAN-601 D2/T-07，P593-D2 关债〕）。
    pub(crate) const ACCENT_PALETTE_JS: &str = r#"
// Plan 360: Accent color palette (aligned with auto-forge).
// Each entry maps a name → shadcn --primary HSL triplet (space-separated).
const ACCENT_PALETTES: Record<string, string> = {
  indigo: '239 84% 67%',
  // Plan 503: coral 校准至 stella-os 玫瑰粉 light #c4706a(dark +4 由 applyAccent 处理)。
  coral:  '4 43% 59%',
  ocean:  '217 91% 60%',
  sage:   '160 84% 39%',
  amber:  '38 92% 50%',
}
const ACCENT_NAMES = Object.keys(ACCENT_PALETTES)
const ACCENT_STORAGE_KEY = 'notes-accent-color'

/** Apply the named accent by writing the --primary CSS variable.
 *  Also adjusts lightness up slightly in dark mode for readability.
 *  HSL values are stored as "H S% L%" (shadcn format); the % is preserved
 *  so we use parseFloat to read the numeric part for the lightness tweak.
 *
 *  The variable is written to BOTH <html> AND any element carrying the
 *  `.dark` class. This is necessary because the generated dark-mode CSS
 *  puts `.dark { --primary: ... }` on a root wrapper div, which would
 *  otherwise shadow the value inherited from <html>. We use a microtask
 *  (setTimeout 0) for the .dark pass so Vue has flushed the :class change. */
function applyAccent(name: string, isDark = false): void {
  const hsl = ACCENT_PALETTES[name]
  if (!hsl) return
  let finalHsl = hsl
  // Dark mode: boost lightness for contrast against dark backgrounds.
  // PLAN-601 D2 归一：+4 → +10（与 Rust/VM 侧 accent_primary_hsl 统一，
  // coral 校准/stella 对齐实证为准；vue 暗色 accent 视觉微调在案）。
  if (isDark) {
    const match = hsl.match(/^(\d+\s+[\d.]+%)\s+([\d.]+)%$/)
    if (match) {
      const boosted = Math.min(85, parseFloat(match[2]) + 10)
      finalHsl = match[1] + ' ' + boosted + '%'
    }
  }
  const root = document.documentElement
  root.style.setProperty('--primary', finalHsl)
  // Also set on any .dark element so it overrides the .dark { --primary }
  // rule defined in index.css (which lives on a different element than <html>).
  // Done synchronously AND on next tick (covers both: dark already applied,
  // and dark just toggled — Vue flushes :class after this call returns).
  // CRITICAL: in light mode (no .dark elements) we must also REMOVE any
  // stale inline --primary left over from a previous dark-mode apply, otherwise
  // the old value shadows the new <html>-level value via CSS inheritance.
  function applyToDark() {
    if (isDark) {
      document.querySelectorAll('.dark').forEach(function (el) {
        ;(el as HTMLElement).style.setProperty('--primary', finalHsl)
      })
    } else {
      // Light mode: clear any stale inline --primary on elements that
      // previously carried .dark (the wrapper still exists, just without .dark).
      // IMPORTANT: skip documentElement (html) — that's where we just set the
      // current value. Only clear child elements with a stale inline override.
      document.querySelectorAll('[style*="--primary"]').forEach(function (el) {
        if (el !== document.documentElement) {
          ;(el as HTMLElement).style.removeProperty('--primary')
        }
      })
    }
  }
  applyToDark()
  setTimeout(applyToDark, 0)
  try { localStorage.setItem(ACCENT_STORAGE_KEY, name) } catch {}
}

/** Read the saved accent from localStorage, or '' when nothing was persisted
 * (callers apply their own fallback — store default 'indigo', or the Plan 458
 * CLI/env-seeded value which must NOT be clobbered). */
function getSavedAccent(): string {
  try {
    const saved = localStorage.getItem(ACCENT_STORAGE_KEY)
    if (saved && ACCENT_PALETTES[saved]) return saved
  } catch {}
  return ''
}

/** List of accent names for UI rendering (swatch buttons). */
function getAccentNames(): string[] {
  return ACCENT_NAMES
}
"#;

    /// PLAN-601 T-06：主题热切换运行时——applyAccent 的整套泛化。
    /// THEME_PALETTES 值源 = design_tokens registry 五内置双面（T-06 值源
    /// 发射器，与 CSS 面同源）；pac.at theme{} 声明合成体经 auto-man 注入
    /// `window.__AUTO_COMPOSED_THEME__` 于运行时并入。注入门控与 accent
    /// 系统同臂（has_accent_color）——主题切换是 accent 切换的整套泛化，
    /// 零 accent 面 app 维持零注入（零变化优先裁定）。storage 键为全局
    /// appearance 语义（与桌面端 appearance.theme 同型）：同源 app 间共享
    /// 用户主题偏好，属预期行为。
    pub(super) fn theme_runtime_js() -> String {
        let palettes = crate::design_tokens::registry::render_theme_palettes_js();
        format!(
            r#"// PLAN-601 T-06: Named theme hot-switch runtime (applyAccent generalized
// to the full token set). Values mirror the generated index.css registry
// source (dual-face single source of truth).
const THEME_PALETTES: Record<string, {{ light: Record<string, string>, dark: Record<string, string> }}> = {palettes}
// pac.at theme{{}} declared app theme (generator-injected) joins the set.
if ((window as any).__AUTO_COMPOSED_THEME__) {{
  const ct = (window as any).__AUTO_COMPOSED_THEME__
  if (ct && ct.name && ct.light && ct.dark) THEME_PALETTES[ct.name] = ct
}}
// PLAN-007 T-05: 树状官方主题族（generator 注入 __AUTO_TREE_THEMES__：
// name -> {{ light, dark }}，值含 app 命名空间扩展色）并入可选面。
if ((window as any).__AUTO_TREE_THEMES__) {{
  const tts = (window as any).__AUTO_TREE_THEMES__
  for (const k in tts) if (tts[k] && tts[k].light && tts[k].dark) THEME_PALETTES[k] = tts[k]
}}
const THEME_STORAGE_KEY = 'auto-theme'

/** Active theme: last applied choice persisted in localStorage, else the
 *  scaffold default (matching the generated index.css :root block). */
function getActiveTheme(): string {{
  try {{
    const saved = localStorage.getItem(THEME_STORAGE_KEY)
    if (saved && THEME_PALETTES[saved]) return saved
  }} catch {{}}
  return 'scaffold'
}}

/** Apply a named theme by writing its FULL variable set. Light values go on
 *  <html> inline (beats every stylesheet); when dark, the dark set is also
 *  written on every `.dark` element (declarations beat inheritance), so a
 *  mode flip keeps the active theme instead of falling back to the scaffold
 *  `.dark` block. Light mode clears stale inline vars from a previous dark
 *  write. The accent overlay runs LAST so a named --primary rides on top of
 *  any theme (+10 dark boost inside applyAccent). Unknown names are ignored
 *  (closed vocabulary). */
function applyTheme(name: string, isDark = document.documentElement.classList.contains('dark'), accentName = getSavedAccent()): void {{
  const pal = THEME_PALETTES[name]
  if (!pal) return
  const root = document.documentElement
  function writeVars(el: HTMLElement, vars: Record<string, string>) {{
    for (const k in vars) el.style.setProperty('--' + k, vars[k])
  }}
  writeVars(root, pal.light)
  function applyDarkPass() {{
    if (isDark) {{
      document.querySelectorAll('.dark').forEach(function (el) {{ writeVars(el as HTMLElement, pal!.dark) }})
    }} else {{
      // Any previous full write carries --background — use it as the marker
      // for stale inline overrides (documentElement keeps its own value).
      document.querySelectorAll('.dark, [style*="--background"]').forEach(function (el) {{
        if (el !== root) for (const k in pal!.light) (el as HTMLElement).style.removeProperty('--' + k)
      }})
    }}
  }}
  applyDarkPass()
  setTimeout(applyDarkPass, 0)
  try {{ localStorage.setItem(THEME_STORAGE_KEY, name) }} catch {{}}
  if (accentName) applyAccent(accentName, isDark)
}}
"#
        )
    }

    /// Convert an initial-value AuraExpr to a JS literal (v1: simple cases).
    pub(super) fn store_init_to_js(expr: &auto_lang_core::ast::Expr) -> String {
        use auto_lang_core::ast::Expr;
        match expr {
            Expr::Int(n) => n.to_string(),
            // PLAN-668 R-23（013 随检）：负标量字面量 parse 为
            // Unary(Sub, 字面量)——此前缺臂落 `_ => null`，typed int store
            // 变量变 ref<number>(null)，vue-tsc TS2345。
            Expr::Unary(op, inner) if matches!(*op, auto_val::Op::Sub | auto_val::Op::Add) => {
                format!(
                    "{}{}",
                    if matches!(*op, auto_val::Op::Sub) {
                        "-"
                    } else {
                        "+"
                    },
                    Self::store_init_to_js(inner)
                )
            }
            Expr::Float(f, _) | Expr::Double(f, _) => f.to_string(),
            Expr::Str(s) | Expr::CStr(s) => format!("'{}'", Self::escape_js_string(s.as_str())),
            Expr::Bool(b) => b.to_string(),
            Expr::Array(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::store_init_to_js).collect();
                format!("[{}]", parts.join(", "))
            }
            // Plan 012 Batch G (gap 15): map literal initializers
            // (`var x map = {}` / `{ k: v, ... }`) parse as Expr::Object.
            // Previously they fell into the `_ => null` arm, so a store var
            // declared with a map literal started life as `ref(null)` while
            // the same initializer in a widget model kept the literal —
            // divergent behavior that null-deref'd any early `.x` access.
            // Emit the object literal verbatim (same shape as the widget
            // path via ts_adapter's Expr::Object arm).
            Expr::Object(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|pair| {
                        let key = match &pair.key {
                            auto_lang_core::ast::Key::NamedKey(name) => name.as_str().to_string(),
                            auto_lang_core::ast::Key::IntKey(n) => n.to_string(),
                            auto_lang_core::ast::Key::BoolKey(b) => b.to_string(),
                            auto_lang_core::ast::Key::StrKey(s) => {
                                format!("\"{}\"", s.as_str().replace('"', "\\\""))
                            }
                        };
                        format!("{}: {}", key, Self::store_init_to_js(&pair.value))
                    })
                    .collect();
                if parts.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", parts.join(", "))
                }
            }
            // Plan 043 store-codegen: `List<T>.new([])` and similar container
            // constructors parse as a call whose callee ends in `.new`. Treat
            // any `*.new(...)` initializer as an empty array — the common case
            // is `List<Block>.new([])`. Struct-literal `Type{...}` (Expr::Node)
            // and None/nil fall back to null.
            Expr::Call(call) => {
                let callee = match call.name.as_ref() {
                    Expr::Dot(_, method) => method.as_str(),
                    _ => "",
                };
                if callee == "new" {
                    "[]".to_string()
                } else {
                    "null".to_string()
                }
            }
            Expr::Nil | Expr::Null => "null".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Generate Vue Router configuration file (Plan 105)
    ///
    /// Creates a `router/index.ts` file with route definitions.
    pub fn generate_router_file(routes: &[crate::aura::AuraRoute]) -> String {
        let mut route_defs = Vec::new();

        for route in routes {
            // Generate route definition with lazy loading (Plan 106)
            let module = &route.module;
            let path = &route.path;

            // Create route object with lazy loading
            if route.params.is_empty() {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue') }}",
                    path, module, module
                ));
            } else {
                // Route with params - add props: true for dynamic segments
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue'), props: true }}",
                    path,
                    module,
                    module
                ));
            }
        }

        // No static imports needed - using lazy loading

        format!(
            r#"import {{ createRouter, createWebHashHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

const router = createRouter({{
  history: createWebHashHistory(),
  routes,
}})

export default router
"#,
            route_defs.join(",\n")
        )
    }
}

