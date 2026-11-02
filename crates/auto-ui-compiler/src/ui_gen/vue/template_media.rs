//! PLAN-014 T-04：template/style/媒体交互责任（<template>/<style>、canvas/video/进度、image-surface、actions 合成、previewcard/codeblock）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    /// Generate <template> content from view tree
    pub(super) fn generate_template(&mut self, root: &AuraNode) -> GenResult<String> {
        let mut template = String::new();
        // 每模板重置：ref 只属于本次生成的根元素（同一 generator 复用时
        // 不把上一个 widget 的容器标记漏给下一个）。
        self.theme_root_ref = false;

        let mut root_html = self.node_to_html(root, 2)?;

        // PLAN-031 T-03：带主题/强调色运行时的 widget，根元素挂容器 ref；
        // applyTheme/applyAccent 把 CSS vars 写到该容器子树（同文档多 App
        // 各自独立），无根 <div> 的模板保持 document legacy（具名边界）。
        if (self.has_theme_name || self.has_accent_color)
            && !self.theme_root_ref
            && root_html.contains("<div ")
        {
            root_html = root_html.replacen("<div ", "<div ref=\"__autoThemeRoot\" ", 1);
            self.theme_root_ref = true;
        }

        // Check for dark mode: either explicit state var, view tree reference,
        // or a ToggleDarkMode handler used in the template
        let has_toggle_dark = self.used_handlers.contains("ToggleDarkMode");
        if self.has_dark_mode || has_toggle_dark {
            let dark_expr = match &self.dark_mode_var {
                // Plan 458: bind whichever instance actually exists — the
                // widget's own ref (root-App pattern, 006-hero-section) or
                // the store convention (015-notes). Binding store.dark_mode
                // for a widget-local var crashes the render (`store` is not
                // defined on the instance → blank page).
                Some(var) if var == "dark_mode" => {
                    if self.dark_mode_on_widget {
                        "dark_mode"
                    } else {
                        "store.dark_mode"
                    }
                }
                _ => "isDark",
            };
            // If dark_expr is store.dark_mode but var wasn't set, use store.dark_mode
            let dark_expr = if has_toggle_dark && self.dark_mode_var.is_none() {
                "store.dark_mode"
            } else {
                dark_expr
            };
            let html = if let Some(pos) = root_html.find(":class=\"") {
                // Plan 458: the root already carries a dynamic :class (e.g. a
                // `style: if` conditional merged by the layout path). Vue
                // rejects a second :class attribute ("Duplicate attribute"),
                // so merge both into one array binding:
                //   :class="EXISTING"  →  :class="[{ dark: dark_mode }, EXISTING]"
                let val_start = pos + ":class=\"".len();
                match root_html[val_start..].find('"') {
                    Some(rel) => {
                        let val_end = val_start + rel;
                        let mut merged =
                            String::with_capacity(root_html.len() + dark_expr.len() + 8);
                        merged.push_str(&root_html[..val_start]);
                        merged.push_str(&format!("[{{ dark: {} }}, ", dark_expr));
                        merged.push_str(&root_html[val_start..val_end]);
                        merged.push(']');
                        merged.push_str(&root_html[val_end..]);
                        merged
                    }
                    None => root_html.replacen(
                        "<div ",
                        &format!("<div :class=\"{{ dark: {} }}\" ", dark_expr),
                        1,
                    ),
                }
            } else {
                root_html.replacen(
                    "<div ",
                    &format!("<div :class=\"{{ dark: {} }}\" ", dark_expr),
                    1,
                )
            };
            template.push_str(&html);
        } else {
            template.push_str(&root_html);
        }

        // Plan 548 T7: SidebarProvider 自动包裹 —— SidebarMenuButton/
        // SidebarRail 等资产注入 useSidebar() context(utils.ts),缺
        // Provider 祖先会白屏(同 TooltipProvider 教训)。view 树含 sidebar
        // 且无 sidebar_provider 时模板根自动包一层,并登记 import;用户
        // 显式写了 sidebar_provider 则不重复包裹。仅 shadcn 模式。
        if self.is_shadcn()
            && view_tree_has_tag(root, "sidebar")
            && !view_tree_has_tag(root, "sidebar_provider")
        {
            self.shadcn_components_used
                .insert("SidebarProvider".to_string());
            let ind = "  ".repeat(2);
            template = format!("{ind}<SidebarProvider>\n{template}{ind}</SidebarProvider>\n");
        }

        Ok(template)
    }

    /// Generate <style> content
    pub(super) fn generate_style(&self) -> String {
        let mut style = String::new();

        // Plan 106: Override Prism.js default margin on pre elements
        if !self.previewcard_data.is_empty() || !self.codeblock_data.is_empty() {
            style.push_str("/* Override Prism.js default styles */\n");
            style.push_str("pre[class*=\"language-\"] {\n");
            style.push_str("  margin: 0;\n");
            style.push_str("}\n\n");
            // Translucent code-block scrollbars — match ScrollArea's look:
            // thin, semi-transparent thumb, no track. The codeblock <pre> uses
            // a dark (zinc-950) background in both themes, so a light thumb
            // reads on both. WebKit/Blink via ::-webkit-scrollbar, Firefox
            // via scrollbar-width / scrollbar-color.
            style.push_str("/* Code block scrollbars: translucent thumb, no track */\n");
            style.push_str("pre::-webkit-scrollbar { height: 8px; width: 8px; }\n");
            style.push_str("pre::-webkit-scrollbar-track { background: transparent; }\n");
            style.push_str("pre::-webkit-scrollbar-thumb { background-color: rgb(244 244 245 / 0.2); border-radius: 9999px; }\n");
            style.push_str("pre::-webkit-scrollbar-thumb:hover { background-color: rgb(244 244 245 / 0.3); }\n");
            style.push_str("pre { scrollbar-width: thin; scrollbar-color: rgb(244 244 245 / 0.2) transparent; }\n\n");
        }

        style.push_str("/* Component styles */\n");
        // PLAN-018: iconfile 双主题位图切换（浅表常显；宿主根 .dark 时换深表）
        if self.iconfile_theme_css {
            style.push_str("/* PLAN-018 iconfile dual-theme bitmap switch */\n");
            style.push_str(".app-icon-dark { display: none; }\n");
            style.push_str(".dark .app-icon-light { display: none; }\n");
            style.push_str(".dark .app-icon-dark { display: inline-block; }\n");
        }
        style
    }

    /// Plan 563: canvas 元素 → `<canvas ref>`(状态驱动画布,vue 原生 2D
    /// context;与 iced canvas::Program 双端独立实现,共享场景数据契约)。
    /// 模板臂只出 ref + class 并登记 canvas_specs;redraw(watch deep +
    /// onMounted 初绘 + resize 重绘)与 pen 三包装(mousedown 内联;
    /// document mousemove/mouseup 挂载——出界即收笔,T1d 双端统一语义)
    /// 在 generate_script 消费时生成。
    pub(super) fn generate_canvas_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        // scene 前缀(scene: .strokes → "strokes")。`.field` 在 parser 层
        // 解析为 Dot(Ident("self"|"."), field),legacy 裸名/点整体 Ident
        // 兼收;其余形态退化空画布。
        let prefix = match props.get("scene") {
            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name))) => {
                name.trim_start_matches('.').to_string()
            }
            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Dot(obj, field))) => {
                match obj.as_ref() {
                    auto_lang_core::ast::Expr::Ident(base)
                        if base.as_str() == "self" || base.as_str() == "." =>
                    {
                        field.to_string()
                    }
                    _ => String::new(),
                }
            }
            _ => {
                self.warn(
                    "R561",
                    crate::ui_gen::validators::Severity::Warning,
                    "canvas scene prop 应绑定状态前缀(scene: .strokes → \
                     strokes_pts/strokes_meta 双表);当前形态退化为空画布"
                        .to_string(),
                );
                String::new()
            }
        };
        let extent = props
            .get("coords")
            .and_then(|v| self.extract_string_value(v))
            .and_then(|s| {
                let s = s.trim();
                let (w, h) = s.split_once(['x', 'X'])?;
                let w: f32 = w.trim().parse().ok()?;
                let h: f32 = h.trim().parse().ok()?;
                (w > 0.0 && h > 0.0).then_some((w, h))
            });
        let clear = props
            .get("clear")
            .and_then(|v| self.extract_string_value(v))
            .map(|s| s.to_string());
        let handler_name = |key: &str| {
            events
                .get(key)
                .map(|e| self.handler_to_function_call(&e.handler))
        };
        let on_start = handler_name("onpenstart");
        let on_move = handler_name("onpenmove");
        let on_end = handler_name("onpenend");
        // PLAN-661 T-05（R-1）：onhit handler（命中元素 id 字符串载荷）。
        let on_hit = handler_name("onhit");
        if let Some(h) = &on_hit {
            self.used_handlers.insert(h.clone());
        }
        let has_hit = on_hit.is_some();
        // 图元三表声明探测（未声明 → 运行时空集，不引用未声明 ref）。
        let nodes_declared = self.state_names.contains(&format!("{prefix}_nodes"));
        let edges_declared = self.state_names.contains(&format!("{prefix}_edges"));
        let labels_declared = self.state_names.contains(&format!("{prefix}_labels"));
        let idx = self.canvas_specs.len();
        let ref_name = format!("__canvas_ref_{idx}");
        self.template_refs.push(ref_name.clone());
        let has_start = on_start.is_some();
        self.canvas_specs.push(CanvasSpec {
            ref_name: ref_name.clone(),
            prefix,
            extent,
            clear,
            on_start,
            on_move,
            on_end,
            on_hit: on_hit.clone(),
            nodes_declared,
            edges_declared,
            labels_declared,
        });
        let (class_str, _, _) = self.extract_classes("canvas", props);
        let class_attr = if class_str.is_empty() {
            String::new()
        } else {
            format!(" class=\"{class_str}\"")
        };
        // 起笔走元素内联 mousedown(走笔/收笔在 script 臂挂 document 级)。
        let down_attr = if has_start {
            format!(" @mousedown=\"__canvasDown_{idx}\"")
        } else {
            String::new()
        };
        // PLAN-661 T-05：tap 命中走 pointer 事件（与 pen 的 mouse 事件
        // 互不占位——双通道并存零属性冲突）。
        let hit_attr = if has_hit {
            format!(" @pointerdown=\"__canvasHitDown_{idx}\" @pointerup=\"__canvasHitUp_{idx}\"")
        } else {
            String::new()
        };
        Ok(format!(
            "{ind}<canvas ref=\"{ref_name}\"{class_attr}{down_attr}{hit_attr}></canvas>\n"
        ))
    }

    // ─────────────────────────────────────────────────────────────────────
    // PLAN-617 T-07: 受控媒体契约（Vue 端）
    // ─────────────────────────────────────────────────────────────────────

    /// §2.3 的受控下行字段名。**`src` 不在此列**——它是透传（浏览器原生
    /// 属性绑定即可），列进来会让一切带 `src` 的 video 都改道，破坏兼容约束。
    const VIDEO_DOWN_KEYS: [&'static str; 5] = ["paused", "position", "volume", "muted", "rate"];

    /// §2.3 的上行回灌事件名。这些键**不进模板事件表**，改由 script 臂发射的
    /// 原生监听包装调用（因为 handler 的 `$0` 需要被生成器替换成真实取值，
    /// 而 `$event` 既进不了 handler 体、`vue_event_param` 也只认 `.value`/
    /// `.checked` 两种窄化——§2.3 明确要求不改那两处）。
    const VIDEO_UP_KEYS: [&'static str; 6] = [
        "ontimeupdate",
        "onloadedmetadata",
        "onplaystatechange",
        "onended",
        "onmediaerror",
        "onaudiotrack",
    ];

    /// 受控 `video` 元素 → 模板 + 登记 [`VideoSpec`]。
    ///
    /// 返回 `Ok(None)` 表示**这个元素不受控**（未声明任一 [`Self::VIDEO_DOWN_KEYS`]），
    /// 调用方继续走通用原生元素路径。这条早退就是兼容性约束的落点：
    /// 019-video-app 之类「裸 `video { src: ... }`」的生成结果不会经过本函数
    /// 的任何一行。
    pub(super) fn try_generate_controlled_video_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        indent: usize,
    ) -> GenResult<Option<String>> {
        if !Self::VIDEO_DOWN_KEYS.iter().any(|k| props.contains_key(*k)) {
            return Ok(None);
        }
        let ind = "  ".repeat(indent);

        // 下行表达式 → **脚本域** JS（`.volume` → `volume.value`）。与 handler
        // 体走同一个转译器，保证 props/computed/state 三类绑定的取值方式一致。
        let ctx = self.handler_ts_ctx();
        let to_script = |expr: &auto_lang_core::ast::Expr| -> String {
            let mut out: Vec<u8> = Vec::new();
            crate::ui_gen::ts_adapter::transpile_expr_pub(expr, &ctx, &mut out);
            String::from_utf8_lossy(&out).to_string()
        };
        let down_of = |key: &str| -> Option<String> {
            match props.get(key) {
                Some(AuraPropValue::Expr(expr)) => Some(to_script(expr)),
                Some(AuraPropValue::StyleBinding(_)) => None,
                None => None,
            }
        };

        let idx = self.video_specs.len();
        let ref_name = format!("__video_ref_{idx}");
        if !self.template_refs.contains(&ref_name) {
            self.template_refs.push(ref_name.clone());
        }

        // 上行：契约事件键 → handler 函数名。事件键大小写不敏感（`ontimeupdate`
        // 与 `onTimeUpdate` 同义），与 auto_event_to_vue 的宽进一致。
        let mut up_call = |key: &str| -> Option<String> {
            let ev = events.iter().find(|(k, _)| k.to_ascii_lowercase() == key)?;
            let name = self.handler_to_function_call(&ev.1.handler);
            self.used_handlers.insert(name.clone());
            Some(name)
        };
        let up_time = up_call("ontimeupdate");
        let up_metadata = up_call("onloadedmetadata");
        let up_playstate = up_call("onplaystatechange");
        let up_ended = up_call("onended");
        let up_error = up_call("onmediaerror");
        let up_audiotrack = up_call("onaudiotrack");

        let down_position = down_of("position");
        let spec = VideoSpec {
            ref_name: ref_name.clone(),
            down_paused: down_of("paused"),
            down_position: down_position.clone(),
            down_volume: down_of("volume"),
            down_muted: down_of("muted"),
            down_rate: down_of("rate"),
            down_src: down_of("src"),
            up_time,
            up_metadata: up_metadata.clone(),
            up_playstate,
            up_ended,
            up_error,
            up_audiotrack,
        };

        // ── 模板：受控下行 prop 不再作为元素属性发射（`:paused` 会打到元素的
        //    只读 DOM 属性上，严格模式下直接 TypeError；`volume` 的单位也不同）。
        //    其余 prop 一律原样透传，语义与通用路径一致。
        let mut attrs: Vec<String> = vec![format!("ref=\"{ref_name}\"")];
        let mut sorted_props: Vec<(&String, &AuraPropValue)> = props.iter().collect();
        sorted_props.sort_by(|a, b| a.0.cmp(b.0));
        for (key, value) in sorted_props {
            if Self::VIDEO_DOWN_KEYS.contains(&key.as_str()) {
                continue;
            }
            match value {
                AuraPropValue::StyleBinding(bindings) => {
                    attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
                }
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                    attrs.push(format!(":{}=\"{}\"", key, name));
                }
                AuraPropValue::Expr(expr) => {
                    let value_str = self.bound_value_or_warn(
                        expr,
                        &format!("element `video` prop `{}`", key),
                        "null",
                    );
                    attrs.push(format!(":{}=\"{}\"", key, value_str));
                }
            }
        }

        // ── 事件：契约键之外的事件（如 onclick）走通用映射；契约键换成
        //    生成器发射的原生监听包装。
        let mut sorted_ev: Vec<(&String, &crate::aura::AuraEvent)> = events.iter().collect();
        sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
        for (event, aura_event) in sorted_ev {
            if Self::VIDEO_UP_KEYS.contains(&event.to_ascii_lowercase().as_str()) {
                continue;
            }
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let vue_event = self.auto_event_to_vue(event);
            let call =
                self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            self.used_handlers
                .insert(self.handler_to_function_call(&aura_event.handler));
            let call = Self::ime_guard_enter_handler(&vue_event, call);
            attrs.push(format!("{}=\"{}\"", vue_event, call));
        }
        if spec.up_time.is_some() {
            attrs.push(format!("@timeupdate=\"__videoTime_{idx}\""));
        } else if spec.up_audiotrack.is_some() {
            // 音轨探针也需要 timeupdate 节拍，但 Vue 模板**不允许同一元素出现
            // 两个 @timeupdate**（Duplicate attribute 编译错，e2e 实测抓到）：
            // 作者没声明 ontimeupdate 时探针才独占该属性；两者都声明时探针由
            // __videoTime_ 包装链尾调用（见 video_script_block）。
            attrs.push(format!("@timeupdate=\"__videoAudio_{idx}\""));
        }
        // loadedmetadata 监听器**总是**挂（不再是「作者声明了 onloadedmetadata
        // 或 position」才挂）：它是元素侧重入点 __videoSync_{i} 的触发处。
        attrs.push(format!("@loadedmetadata=\"__videoMeta_{idx}\""));
        if spec.up_playstate.is_some() {
            attrs.push(format!("@play=\"__videoState_{idx}\""));
            attrs.push(format!("@pause=\"__videoState_{idx}\""));
        }
        if spec.up_ended.is_some() {
            attrs.push(format!("@ended=\"__videoEnded_{idx}\""));
        }
        if spec.up_error.is_some() {
            attrs.push(format!("@error=\"__videoError_{idx}\""));
        }

        self.video_specs.push(spec);
        Ok(Some(format!("{ind}<video {}></video>\n", attrs.join(" "))))
    }

    /// 受控 `video` 的 script 块：下行 apply + watchEffect 下行同步 + 上行
    /// 原生监听包装。
    ///
    /// 四个与 mpv 侧（`ui/mpv/contract.rs`）刻意对齐的语义：
    /// 1. **`position` 是「目标变化才 seek」**——比对的是上一次下发的*目标*，
    ///    不是元素的 `currentTime`；后者会在播放前进时每帧重 seek，把画面钉死。
    /// 2. **换片让 seek 目标作废**（`src` 变化 →  latch 清空），对应 mpv 侧
    ///    `generation` 前进；否则上一部的 seek 会打到新片上。
    /// 3. **下行以元素自身状态为基准**（`paused !== el.paused` 等）——元素的
    ///    布尔属性是权威值，这样「用户用原生控件改变状态 → 上行回灌 → 下行
    ///    复算」能收敛，而不是互相打架。
    /// 4. **元素侧的状态变化必须能把下行重新推一次**（`__videoSync_{i}`）。
    ///    `watchEffect` 只会在**表达式值变化**时重跑，而元素自己会变：换源后
    ///    浏览器把 `paused` 复位成 `true`，而作者状态 `is_playing` 没变 ——
    ///    没有这个重入点，表现就是「换了片子但再也不播」（实测：真实 Chrome
    ///    里点队列换片后 `paused` 恒 true、`currentTime` 恒 0）。
    ///    把最近一次的下行值缓存下来，`loadedmetadata` 时原样重推一遍即可；
    ///    这条也是「用户手势被浏览器 autoplay 策略拦下后，下一次交互能恢复」
    ///    的恢复路径。
    pub(super) fn video_script_block(&self, i: usize, spec: &VideoSpec) -> String {
        let j = |v: &Option<String>| v.clone().unwrap_or_else(|| "null".to_string());
        let refn = &spec.ref_name;
        // 音轨探针（AC-16b）只在声明了 onaudiotrack 时发射声明/复位/包装，
        // 未声明的受控 video 不多出任何行。
        let audio_decl = if spec.up_audiotrack.is_some() {
            format!("let __videoAudioDone_{i} = false\n")
        } else {
            String::new()
        };
        let audio_reset = if spec.up_audiotrack.is_some() {
            format!("    __videoAudioDone_{i} = false\n")
        } else {
            String::new()
        };
        let mut s = String::new();
        s.push_str(&format!(
"// PLAN-617 T-07 受控媒体契约（下行 状态→元素 / 上行 元素→状态）。与
// crates/auto-lang/src/ui/mpv/contract.rs 同形同名同单位：volume 走作者面
// 0..100（此处除 100 → 元素 0..1），position 走秒且「目标变化才 seek」。
let __videoSrc_{i}: string | null = null
let __videoSeek_{i}: number | null = null
let __videoPending_{i}: number | null = null
{audio_decl}// 最近一次的下行值快照 —— 供 __videoSync_{i} 在元素侧状态变化时原样重推。
let __videoLast_{i}: [boolean | null, number | null, number | null, boolean | null, number | null, string | null] = [null, null, null, null, null, null]
function __videoApply_{i}(paused: boolean | null, position: number | null, volume: number | null, muted: boolean | null, rate: number | null, src: string | null) {{
  const el = {refn}.value
  if (!el) return
  if (src !== __videoSrc_{i}) {{
    __videoSrc_{i} = src
    __videoSeek_{i} = null
    __videoPending_{i} = null
{audio_reset}  }}
  if (paused !== null && paused !== el.paused) {{
    if (paused) el.pause()
    else {{ const p = el.play(); if (p && typeof p.catch === 'function') p.catch(() => {{}}) }}
  }}
  if (muted !== null && muted !== el.muted) el.muted = muted
  if (volume !== null && Number.isFinite(volume)) {{
    const v = Math.min(Math.max(volume, 0), 100) / 100
    if (Math.abs(v - el.volume) > 1e-6) el.volume = v
  }}
  if (rate !== null && rate > 0 && Math.abs(rate - el.playbackRate) > 1e-6) el.playbackRate = rate
  if (position !== null && Number.isFinite(position)) {{
    if (__videoSeek_{i} === null || Math.abs(__videoSeek_{i} - position) > 1e-6) {{
      __videoSeek_{i} = position
      if (el.readyState >= 1) el.currentTime = position
      else __videoPending_{i} = position
    }}
  }} else if (position === null) {{
    __videoSeek_{i} = null
  }}
}}
// 元素侧重入点（见函数文档第 4 条）：把最近一次下行值原样重推。
function __videoSync_{i}() {{
  __videoApply_{i}(__videoLast_{i}[0], __videoLast_{i}[1], __videoLast_{i}[2], __videoLast_{i}[3], __videoLast_{i}[4], __videoLast_{i}[5])
}}
// 下行同步挂 onMounted：`watchEffect` 会**立即**执行一次回调，而本块在
// `<script setup>` 里的位置早于 `const store = reactive(useXxxStore())`
// （生成器的 store facade 段落靠后），setup 期直接求值会命中 TDZ
// （Cannot access store before initialization）。onMounted 回调在挂载后
// 执行，此时 ref 已指向真元素、store 已初始化；依赖登记不变。
onMounted(() => {{
  watchEffect(() => {{
    __videoLast_{i} = [{paused}, {position}, {volume}, {muted}, {rate}, {src}]
    __videoApply_{i}(__videoLast_{i}[0], __videoLast_{i}[1], __videoLast_{i}[2], __videoLast_{i}[3], __videoLast_{i}[4], __videoLast_{i}[5])
  }})
}})
",
            paused = j(&spec.down_paused),
            position = j(&spec.down_position),
            volume = j(&spec.down_volume),
            muted = j(&spec.down_muted),
            rate = j(&spec.down_rate),
            src = j(&spec.down_src),
        ));
        // loadedmetadata 的包装**总是**发射（不再只在作者声明了
        // onloadedmetadata 或 position 时）：它是上面第 4 条的落点。
        let call = spec
            .up_metadata
            .as_ref()
            .map(|h| format!("  {h}(el.duration)\n"))
            .unwrap_or_default();
        s.push_str(&format!(
"function __videoMeta_{i}(e: Event) {{
  const el = e.target as HTMLVideoElement
  if (__videoPending_{i} !== null) {{ const t = __videoPending_{i}; __videoPending_{i} = null; el.currentTime = t }}
  __videoSync_{i}()
{call}}}
"));
        if spec.up_time.is_some() {
            let call = spec.up_time.as_ref().unwrap();
            // onaudiotrack 也声明时：探针作为链尾调用（模板上 timeupdate 属性
            // 只能出现一次，见 try_generate_controlled_video_html 的注释）。
            let audio_chain = spec
                .up_audiotrack
                .as_ref()
                .map(|_| format!("  __videoAudio_{i}(e)\n"))
                .unwrap_or_default();
            let mut b = String::new();
            b.push_str(&format!(
                "function __videoTime_{i}(e: Event) {{
  const el = e.target as HTMLVideoElement
  {call}(el.currentTime)
{audio_chain}}}
"
            ));
            s.push_str(&b);
        }
        if spec.up_playstate.is_some() {
            let call = spec.up_playstate.as_ref().unwrap();
            s.push_str(&format!(
                "function __videoState_{i}(e: Event) {{
  const el = e.target as HTMLVideoElement
  {call}(!el.paused)
}}
"
            ));
        }
        if spec.up_ended.is_some() {
            let call = spec.up_ended.as_ref().unwrap();
            s.push_str(&format!(
                "function __videoEnded_{i}(_e: Event) {{
  {call}()
}}
"
            ));
        }
        if spec.up_error.is_some() {
            let call = spec.up_error.as_ref().unwrap();
            s.push_str(&format!(
                "function __videoError_{i}(e: Event) {{
  const el = e.target as HTMLVideoElement
  const err = el.error
  let msg = '媒体加载失败'
  if (err) {{
    if (err.code === 1) msg = '媒体加载被中止'
    else if (err.code === 2) msg = '网络错误：媒体数据读取失败'
    else if (err.code === 3) msg = '解码失败：该文件的画面或音轨编码不受支持'
    else if (err.code === 4) msg = '媒体格式不受支持，或媒体源不可用'
    else msg = '媒体错误（代码 ' + err.code + '）'
  }}
  {call}(msg)
}}
"
            ));
        }
        // PLAN-617 T-11 / AC-16(b)：音轨可用性探针。复用 timeupdate 节拍，
        // 条件是「实际播放超过 1s」——暂停/开头 1s 内音频字节数天然为 0，
        // 不构成结论。探针（webkitAudioDecodedByteCount）是 Chromium 专有；
        // 不存在时不调用 handler（不主张任何结论）。一次性：首个有效样本
        // 之后不再报告，换片由 __videoApply 的 src 分支复位。
        if spec.up_audiotrack.is_some() {
            let call = spec.up_audiotrack.as_ref().unwrap();
            s.push_str(&format!(
"function __videoAudio_{i}(e: Event) {{
  const el = e.target as HTMLVideoElement
  if (__videoAudioDone_{i} || el.paused || el.currentTime <= 1.0) return
  __videoAudioDone_{i} = true
  const probe = (el as unknown as {{ webkitAudioDecodedByteCount?: number }}).webkitAudioDecodedByteCount
  if (typeof probe !== 'number') return
  {call}(probe > 0)
}}
"));
        }
        s
    }

    // ─────────────────────────────────────────────────────────────────────
    // PLAN-617 后续: 可拖拽进度条（`progress` + `onseek`）
    // ─────────────────────────────────────────────────────────────────────

    /// 可拖拽进度条 → 外层指针包装 + 登记脚本块。
    ///
    /// 只在**声明了 `onseek`** 时进入（否则返回 `Ok(None)` 走通用路径，
    /// 既有 `progress` 输出逐字节不变）。契约：handler 收 **0..1 的横向比例**，
    /// 与 VM 侧 `View::ProgressBar.on_seek`（`seek_area.rs`）同尺——作者写
    /// `store.SeekTo(.duration * $0)` 即可，不需要知道像素或 max。
    pub(super) fn try_generate_seekable_progress_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        indent: usize,
    ) -> GenResult<Option<String>> {
        let Some(ev) = events
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == "onseek")
            .map(|(_, v)| v)
        else {
            return Ok(None);
        };
        let handler = self.handler_to_function_call(&ev.handler);
        self.used_handlers.insert(handler.clone());
        let idx = self.progress_seek_specs.len();
        self.progress_seek_specs.push(handler);
        let ind = "  ".repeat(indent);

        // 内层就是普通 shadcn Progress（与通用 progress 臂同口径的绑定）。
        let mut inner: Vec<String> = Vec::new();
        if let Some(value) = props.get("value") {
            if let Some(model) = self.extract_state_ref(value) {
                inner.push(format!(
                    ":model-value=\"{}\"",
                    Self::escape_html_attr(&model)
                ));
            } else if let Some(i) = self.extract_int_value(value) {
                inner.push(format!(
                    ":model-value=\"{}\"",
                    Self::escape_html_attr(&i.to_string())
                ));
            } else if let AuraPropValue::Expr(expr) = value {
                if let Ok(expr_str) = self.expr_to_vue_bound_value(expr) {
                    inner.push(format!(
                        ":model-value=\"{}\"",
                        Self::escape_html_attr(&format!("Number({expr_str})"))
                    ));
                }
            }
        }
        if let Some(max) = props.get("max") {
            let m = self.extract_int_value(max).unwrap_or(100);
            inner.push(format!(
                ":max=\"{}\"",
                Self::escape_html_attr(&m.to_string())
            ));
        }
        let (classes, _dyn_c, _dyn_s) = self.extract_classes("progress", props);
        if !classes.is_empty() {
            inner.push(format!("class=\"{}\"", Self::escape_html_attr(&classes)));
        }
        self.shadcn_components_used.insert("Progress".to_string());
        Ok(Some(format!(
            "{ind}<div class=\"w-full\" @pointerdown=\"__seekDown_{idx}($event)\" @pointermove=\"__seekMove_{idx}($event)\" @pointerup=\"__seekUp_{idx}($event)\" @pointercancel=\"__seekUp_{idx}($event)\">\n{ind}  <Progress {}/>\n{ind}</div>\n",
            inner.join(" ")
        )))
    }

    /// 可拖拽进度条的 script 块：比例换算 + 按下/拖动/抬起三包装。
    ///
    /// 按下即 seek（点哪跳哪）；**按住期间**的移动才继续 seek（悬停不 scrub
    /// ——与 VM 侧 `seek_area.rs` 的 `pressed` 门控同语义）；指针捕获让拖出
    /// 条外仍跟手。
    pub(super) fn progress_seek_script_block(&self, i: usize, handler: &str) -> String {
        let mut s = String::new();
        s.push_str("// PLAN-617: 可拖拽进度条（progress + onseek）。handler 收 0..1 比例。\n");
        s.push_str(&format!("let __seekDrag_{i} = false\n"));
        s.push_str(&format!(
            "function __seekFrac_{i}(e: PointerEvent, el: HTMLElement): number {{\n  const r = el.getBoundingClientRect()\n  if (!(r.width > 0)) return 0\n  return Math.min(Math.max((e.clientX - r.left) / r.width, 0), 1)\n}}\n"
        ));
        s.push_str(&format!(
            "function __seekDown_{i}(e: PointerEvent) {{\n  const el = e.currentTarget as HTMLElement\n  __seekDrag_{i} = true\n  const anyEl = el as any\n  if (typeof anyEl.setPointerCapture === 'function') anyEl.setPointerCapture(e.pointerId)\n  {handler}(__seekFrac_{i}(e, el))\n}}\n"
        ));
        s.push_str(&format!(
            "function __seekMove_{i}(e: PointerEvent) {{\n  if (!__seekDrag_{i}) return\n  const el = e.currentTarget as HTMLElement\n  {handler}(__seekFrac_{i}(e, el))\n}}\n"
        ));
        s.push_str(&format!(
            "function __seekUp_{i}(e: PointerEvent) {{\n  __seekDrag_{i} = false\n  const anyEl = e.currentTarget as any\n  if (typeof anyEl.releasePointerCapture === 'function') {{\n    try {{ anyEl.releasePointerCapture(e.pointerId) }} catch {{}}\n  }}\n}}\n"
        ));
        s
    }

    /// Plan 563: canvas script 块 —— redraw(场景双表 → 2D 绘制,场景
    /// 数据契约的 vue 侧独立映射:逻辑→px extent 缩放/round 线帽拐角/
    /// 单点=直径线宽圆点/eraser=clear 色缺省白——与 iced CanvasPainter
    /// 同规约不共享代码)+ pen 三包装(mousedown 起笔;document
    /// mousemove buttons&1 门控 + 33ms 时间闸 + 出界即收笔;document
    /// mouseup 收笔——与 iced PenArea 同语义)+ watch deep + onMounted/
    /// resize 初绘重绘。
    pub(super) fn canvas_script_block(&mut self, i: usize, spec: &CanvasSpec) -> String {
        let refn = &spec.ref_name;
        let prefix = &spec.prefix;
        let mut s = String::new();

        // ---- redraw:场景契约 → 2D(状态 ref 引用为 <prefix>_pts/_meta)。
        let pts_ref = format!("{prefix}_pts");
        let meta_ref = format!("{prefix}_meta");
        // PLAN-661 T-05: 图元三表 ref（未声明 → 空数组字面量，不引用）。
        let nodes_ref = format!("{prefix}_nodes");
        let edges_ref = format!("{prefix}_edges");
        let labels_ref = format!("{prefix}_labels");
        let nodes_expr = if spec.nodes_declared {
            format!("{nodes_ref}.value")
        } else {
            "[]".to_string()
        };
        let edges_expr = if spec.edges_declared {
            format!("{edges_ref}.value")
        } else {
            "[]".to_string()
        };
        let labels_expr = if spec.labels_declared {
            format!("{labels_ref}.value")
        } else {
            "[]".to_string()
        };
        let scale_decl = match spec.extent {
            Some((w, h)) => format!("  const sx = r.width / {w}, sy = r.height / {h}\n"),
            None => "  const sx = 1, sy = 1\n".to_string(),
        };
        let clear_fill = match &spec.clear {
            Some(c) => {
                format!("  ctx.fillStyle = '{c}'\n  ctx.fillRect(0, 0, r.width, r.height)\n")
            }
            None => String::new(),
        };
        let eraser_color = spec.clear.clone().unwrap_or_else(|| "#ffffff".to_string());
        s.push_str(&format!(
            "function __canvasRedraw_{i}() {{
  const el = {refn}.value
  if (!el) return
  const r = el.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1
  const bw = Math.max(1, Math.round(r.width * dpr)), bh = Math.max(1, Math.round(r.height * dpr))
  if (el.width !== bw) el.width = bw
  if (el.height !== bh) el.height = bh
  const ctx = el.getContext('2d')
  if (!ctx) return
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, r.width, r.height)
{clear_fill}{scale_decl}  const pts = {pts_ref}.value, metas = {meta_ref}.value
  for (let k = 0; k < pts.length; k++) {{
    const parts = String(pts[k]).split('|')
    if (!parts[0]) continue
    const m = String(metas[k] || '').split(',')
    const color = m[0] || '#111827'
    const lw = parseFloat(m[1]) || 3
    const er = m[2] === '1'
    ctx.strokeStyle = er ? '{eraser_color}' : color
    ctx.fillStyle = ctx.strokeStyle
    ctx.lineWidth = lw
    ctx.lineCap = 'round'
    ctx.lineJoin = 'round'
    if (parts.length === 1) {{
      const c = parts[0].split(',')
      ctx.beginPath()
      ctx.arc(parseFloat(c[0]) * sx, parseFloat(c[1]) * sy, lw / 2, 0, Math.PI * 2)
      ctx.fill()
      continue
    }}
    ctx.beginPath()
    for (let j = 0; j < parts.length; j++) {{
      const p = parts[j].split(',')
      const px = parseFloat(p[0]) * sx, py = parseFloat(p[1]) * sy
      if (j === 0) ctx.moveTo(px, py)
      else ctx.lineTo(px, py)
    }}
    ctx.stroke()
  }}
  // PLAN-661 T-05: 图元三表——edges（线段）→ nodes（circle/rect）→
  // labels（文本），声明序；与 iced CanvasPainter 同规约不共享代码。
  const edges = {edges_expr}
  for (let k = 0; k < edges.length; k++) {{
    const p = String(edges[k]).split(',')
    if (p.length < 4) continue
    ctx.strokeStyle = p[4] || '#9ca3af'
    ctx.lineWidth = parseFloat(p[5]) || 2
    ctx.lineCap = 'round'
    ctx.beginPath()
    ctx.moveTo(parseFloat(p[0]) * sx, parseFloat(p[1]) * sy)
    ctx.lineTo(parseFloat(p[2]) * sx, parseFloat(p[3]) * sy)
    ctx.stroke()
  }}
  const nodes = {nodes_expr}
  for (let k = 0; k < nodes.length; k++) {{
    const p = String(nodes[k]).split(',')
    if (p.length < 5 || !p[0]) continue
    const nx = parseFloat(p[1]) * sx, ny = parseFloat(p[2]) * sy
    ctx.fillStyle = p[4] || '#3b82f6'
    if (p[3] === 'rect') {{
      const w = (parseFloat(p[5]) || 40) * sx, h = (parseFloat(p[6]) || 40) * sy
      ctx.fillRect(nx - w / 2, ny - h / 2, w, h)
    }} else {{
      ctx.beginPath()
      ctx.arc(nx, ny, (parseFloat(p[5]) || 16) * sx, 0, Math.PI * 2)
      ctx.fill()
    }}
  }}
  const labels = {labels_expr}
  for (let k = 0; k < labels.length; k++) {{
    const p = String(labels[k]).split(',')
    if (p.length < 3 || !p[2]) continue
    ctx.fillStyle = (p.length >= 4 && p[3]) ? p[3] : '#111827'
    const fs = (p.length >= 5 && parseFloat(p[4])) || 14
    ctx.font = fs + 'px sans-serif'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText(p.slice(2).join(','), parseFloat(p[0]) * sx, parseFloat(p[1]) * sy)
  }}
}}
"
        ));

        // ---- pen 三包装(仅在有任一事件时生成)。
        if spec.on_start.is_some() || spec.on_move.is_some() || spec.on_end.is_some() {
            let to_logical = match spec.extent {
                Some((w, h)) => format!(
                    "const x = (e.clientX - r.left) / r.width * {w}, y = (e.clientY - r.top) / r.height * {h}"
                ),
                None => "const x = e.clientX - r.left, y = e.clientY - r.top".to_string(),
            };
            // 出界判定/clamp 需要 W/H 字面量(raw px 模式退化为 bounds 内恒真)。
            let bounds_check = match spec.extent {
                Some((w, h)) => {
                    format!("if (x < 0 || y < 0 || x > {w} || y > {h}) return true\n  return false")
                }
                None => "return false".to_string(),
            };
            let clamp_decl = match spec.extent {
                Some((w, h)) => format!(
                    "const cx = Math.min(Math.max(x, 0), {w}), cy = Math.min(Math.max(y, 0), {h})"
                ),
                None => "const cx = x, cy = y".to_string(),
            };
            let start_invoke = spec
                .on_start
                .as_ref()
                .map(|h| format!("  {h}(x, y)\n"))
                .unwrap_or_default();
            let move_invoke = spec
                .on_move
                .as_ref()
                .map(|h| format!("  {h}(x, y)\n"))
                .unwrap_or_default();
            let end_invoke_out = spec
                .on_end
                .as_ref()
                .map(|h| format!("    {h}(cx, cy)\n"))
                .unwrap_or_default();
            let end_invoke_up = spec
                .on_end
                .as_ref()
                .map(|h| format!("  {h}(cx, cy)\n"))
                .unwrap_or_default();
            s.push_str(&format!(
                "let __canvas_down_{i} = false
let __canvas_last_{i} = 0
function __canvasDown_{i}(e: MouseEvent) {{
  const el = {refn}.value
  if (!el) return
  const r = el.getBoundingClientRect()
  {to_logical}
  __canvas_down_{i} = true
  __canvas_last_{i} = 0
{start_invoke}}}
function __canvasIsOut_{i}(e: MouseEvent, r: DOMRect): boolean {{
  {to_logical}
  {bounds_check}
}}
function __canvasMove_{i}(e: MouseEvent) {{
  if (!__canvas_down_{i}) return
  if (!(e.buttons & 1)) {{ __canvas_down_{i} = false; return }}
  const el = {refn}.value
  if (!el) return
  const r = el.getBoundingClientRect()
  if (__canvasIsOut_{i}(e, r)) {{
    // 出界即收笔(T1d 双端统一语义;坐标 clamp 到值域边界)。
    __canvas_down_{i} = false
    {to_logical}
    {clamp_decl}
{end_invoke_out}    return
  }}
  const now = performance.now()
  if (now - __canvas_last_{i} < 33) return
  __canvas_last_{i} = now
  {to_logical}
{move_invoke}}}
function __canvasUp_{i}(e: MouseEvent) {{
  if (!__canvas_down_{i}) return
  __canvas_down_{i} = false
  const el = {refn}.value
  if (!el) return
  const r = el.getBoundingClientRect()
  {to_logical}
  {clamp_decl}
{end_invoke_up}}}
"
            ));
            // document 级 move/up(移出画布仍收事件)+ resize 重绘。
            if spec.on_move.is_some() || spec.on_end.is_some() {
                if spec.on_move.is_some() {
                    self.global_listeners.push(GlobalListener {
                        target: "document".to_string(),
                        event: "mousemove".to_string(),
                        listener: format!("__canvasMove_{i}"),
                        capture: false,
                        passive: None,
                        wrapper: None,
                    });
                }
                if spec.on_end.is_some() {
                    self.global_listeners.push(GlobalListener {
                        target: "document".to_string(),
                        event: "mouseup".to_string(),
                        listener: format!("__canvasUp_{i}"),
                        capture: false,
                        passive: None,
                        wrapper: None,
                    });
                }
            }
            self.global_listeners.push(GlobalListener {
                target: "window".to_string(),
                event: "resize".to_string(),
                listener: format!("__canvasRedraw_{i}"),
                capture: false,
                passive: None,
                wrapper: None,
            });
        }

        // ---- PLAN-661 T-05（R-1）：tap 命中三包装（pointer 事件通道，
        // 与 pen 的 mouse 通道分立；位移 ≤4px 视为 tap，nodes 声明序
        // 倒序 topmost，未命中不派发——iced PenArea 同语义不共享代码）。
        if let Some(hit_handler) = &spec.on_hit {
            let to_logical_hit = match spec.extent {
                Some((w, h)) => format!(
                    "const x = (e.clientX - r.left) / r.width * {w}, y = (e.clientY - r.top) / r.height * {h}"
                ),
                None => "const x = e.clientX - r.left, y = e.clientY - r.top".to_string(),
            };
            s.push_str(&format!(
                "let __canvas_hit_start_{i}: {{ x: number, y: number }} | null = null
function __canvasHitDown_{i}(e: PointerEvent) {{
  const el = {refn}.value
  if (!el) return
  const r = el.getBoundingClientRect()
  __canvas_hit_start_{i} = {{ x: e.clientX - r.left, y: e.clientY - r.top }}
}}
function __canvasHitUp_{i}(e: PointerEvent) {{
  const start = __canvas_hit_start_{i}
  __canvas_hit_start_{i} = null
  const el = {refn}.value
  if (!el || !start) return
  const r = el.getBoundingClientRect()
  const dx = (e.clientX - r.left) - start.x, dy = (e.clientY - r.top) - start.y
  if (Math.hypot(dx, dy) > 4) return
  {to_logical_hit}
  const nodes = {nodes_expr}
  for (let k = nodes.length - 1; k >= 0; k--) {{
    const p = String(nodes[k]).split(',')
    if (p.length < 5 || !p[0]) continue
    const nx = parseFloat(p[1]), ny = parseFloat(p[2])
    let hit = false
    if (p[3] === 'rect') {{
      const w = parseFloat(p[5]) || 40, h = parseFloat(p[6]) || w
      hit = Math.abs(x - nx) <= w / 2 && Math.abs(y - ny) <= h / 2
    }} else {{
      const rr = parseFloat(p[5]) || 16
      hit = (x - nx) * (x - nx) + (y - ny) * (y - ny) <= rr * rr
    }}
    if (hit) {{
      {hit_handler}(p[0])
      return
    }}
  }}
}}
"
            ));
        }

        // ---- 状态驱动重绘:watch deep(T1b 复用先例)+ onMounted 初绘。
        // PLAN-661 T-05: watch 源扩图元三表（仅已声明者——未声明 ref 不进
        // watch 数组防 vue-tsc 编译红）。
        let mut watch_sources = format!("{pts_ref}, {meta_ref}");
        if spec.nodes_declared {
            watch_sources.push_str(&format!(", {nodes_ref}"));
        }
        if spec.edges_declared {
            watch_sources.push_str(&format!(", {edges_ref}"));
        }
        if spec.labels_declared {
            watch_sources.push_str(&format!(", {labels_ref}"));
        }
        s.push_str(&format!(
            "watch([{watch_sources}], () => {{ nextTick(__canvasRedraw_{i}) }}, {{ deep: true }})
onMounted(() => {{ nextTick(__canvasRedraw_{i}) }})
"
        ));
        s
    }

    /// Dynamic component: `dyn (.item.icon) { size: 16, class: "..." }` →
    /// `<component :is="item.icon" :size="16" class="..." />`.
    ///
    /// The `is` value is an arbitrary bound expression — a for-loop iterator
    /// field (`item.icon`), a model field, or a computed. Additional props
    /// are passed through as `:prop` bindings; `class` / `style_obj` and
    /// event listeners behave like on plain elements.
    pub(super) fn generate_dyn_component_html(
        &mut self,
        props: &std::collections::HashMap<String, AuraPropValue>,
        events: &std::collections::HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let mut attrs: Vec<String> = Vec::new();

        // :is binding (required). The value may be `any` (e.g. a component
        // constructor carried in list data), so cast for vue-tsc strict mode.
        if let Some(is_value) = props.get("is") {
            if let AuraPropValue::Expr(expr) = is_value {
                let is_str = self.expr_to_vue_bound_value(expr)?;
                attrs.push(format!(":is=\"({}) as any\"", is_str));
            }
        }

        // Class attribute (both static and dynamic)
        let (static_classes, dynamic_class, dynamic_style) = self.extract_classes("dyn", props);
        if !static_classes.is_empty() {
            attrs.push(format!("class=\"{}\"", static_classes));
        }
        if let Some(dc) = dynamic_class {
            attrs.push(format!(":class=\"{}\"", dc));
        }
        if let Some(ds) = dynamic_style {
            attrs.push(format!(":style=\"{}\"", ds));
        }

        // Remaining props as v-bind attributes.
        // Plan 015 P0#3: sorted iteration — props is a HashMap (RandomState);
        // the unsorted order churned generated attribute order (jade gap 56).
        {
            let mut sorted_bind: Vec<(&String, &AuraPropValue)> = props.iter().collect();
            sorted_bind.sort_by(|a, b| a.0.cmp(b.0));
            for (key, value) in sorted_bind {
                if key == "is" || key == "class" || key == "style" {
                    continue;
                }
                // Inline style object: style_obj: { top: expr } → :style="{...}"
                if key == "style_obj" {
                    if let AuraPropValue::StyleBinding(bindings) = value {
                        attrs.push(format!(":style=\"{}\"", self.style_obj_to_vue(bindings)));
                    }
                    continue;
                }
                // v-show visibility directive: show: .cond → v-show="cond"
                // (component stays mounted; only inline display toggles).
                if key == "show" {
                    if let AuraPropValue::Expr(expr) = value {
                        let cond = self.expr_to_vue_bound_value(expr)?;
                        attrs.push(format!("v-show=\"{}\"", cond));
                    }
                    continue;
                }
                // NOTE: `html:` is deliberately NOT intercepted as v-html here.
                // A dyn node targets an arbitrary component — v-html on a
                // component would shadow its slot AND steal the `html` prop,
                // breaking the ext functional-component pattern
                // (`HtmlDiv = (props) => h('div', { innerHTML: props.html })`,
                // jade's gap-22 escape hatch). It falls through to the generic
                // `:html` binding below. Only plain HTML elements get v-html.
                match value {
                    AuraPropValue::Expr(expr) => {
                        // Plan 012 P0#13 follow-up: an unsupported expr form here
                        // used to silently bind `null`; keep that fallback but
                        // warn R013 (a hard error would break the whole widget
                        // for one bad prop).
                        let value_str = self.bound_value_or_warn(
                            expr,
                            &format!("dynamic-component prop `{}`", key),
                            "null",
                        );
                        attrs.push(format!(":{}=\"{}\"", key, value_str));
                    }
                    AuraPropValue::StyleBinding(_) => {}
                }
            }
        }

        // Event listeners (same conventions as plain elements)
        let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
        sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
        for (event, aura_event) in sorted_ev {
            // .window/.document modifiers → global listener, no template attr
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let vue_event = self.auto_event_to_vue(event);
            let mut handler_fn =
                self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            // Inside a for-loop, pass the loop variable when the handler
            // doesn't already have explicit params.
            if let Some(ref loop_var) = self.current_loop_var {
                if aura_event.params.is_empty() {
                    // Plan 043 H2: if the handler's msg variant takes a payload,
                    // forward the DOM event's value via $event rather than the
                    // loop var (parity with the sub-widget path above).
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
            let handler_fn = Self::ime_guard_enter_handler(&vue_event, handler_fn);
            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        if children.is_empty() {
            Ok(format!("{}<component{} />\n", ind, attr_str))
        } else {
            let mut html = format!("{}<component{}>\n", ind, attr_str);
            for child in children {
                // Component children: slot(name:) elements target named slots.
                html.push_str(&self.slot_child_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</component>\n", ind));
            Ok(html)
        }
    }

    /// Generate a slot outlet: `slot` → `<slot />`,
    /// `slot(name: "header")` → `<slot name="header" />`.
    /// Children (if any) become Vue slot fallback content.
    pub(super) fn generate_slot_outlet_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let name_attr = props
            .get("name")
            .and_then(|v| self.extract_string_value(v))
            .map(|n| format!(" name=\"{}\"", n))
            .unwrap_or_default();
        if children.is_empty() {
            Ok(format!("{}<slot{} />\n", ind, name_attr))
        } else {
            let mut html = format!("{}<slot{}>\n", ind, name_attr);
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</slot>\n", ind));
            Ok(html)
        }
    }

    /// Generate Vue's built-in `<Teleport>` (plan 012 P2 gap 27):
    /// `teleport (to: "body") { ... }` → `<Teleport to="body">...</Teleport>`.
    /// - `to:` string literal → static `to="body"`; expression → `:to="expr"`.
    /// - `disabled:` bool literal → static `disabled`; expression →
    ///   `:disabled="expr"`.
    /// Children are emitted normally (recursed via node_to_html).
    pub(super) fn generate_teleport_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);
        let mut attrs: Vec<String> = Vec::new();

        // `to` target (Vue requires it; without it <Teleport> is a no-op and
        // Vue logs a runtime warning — surface that at codegen time).
        match props.get("to") {
            Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))) => {
                attrs.push(format!("to=\"{}\"", s));
            }
            Some(AuraPropValue::Expr(expr)) => {
                let target = self.expr_to_vue_bound_value(expr)?;
                attrs.push(format!(":to=\"{}\"", target));
            }
            Some(AuraPropValue::StyleBinding(_)) => {}
            None => {
                self.warn(
                    "R015",
                    crate::ui_gen::validators::Severity::Warning,
                    "teleport element without a `to` prop: Vue's <Teleport> is a \
                     no-op without a target and logs a runtime warning"
                        .to_string(),
                );
            }
        }

        // Optional `disabled` toggle.
        if let Some(value) = props.get("disabled") {
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => {
                    if *b {
                        attrs.push("disabled".to_string());
                    }
                }
                AuraPropValue::Expr(expr) => {
                    let cond = self.expr_to_vue_bound_value(expr)?;
                    attrs.push(format!(":disabled=\"{}\"", cond));
                }
                AuraPropValue::StyleBinding(_) => {}
            }
        }

        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        if children.is_empty() {
            Ok(format!("{}<Teleport{}></Teleport>\n", ind, attr_str))
        } else {
            let mut html = format!("{}<Teleport{}>\n", ind, attr_str);
            for child in children {
                html.push_str(&self.node_to_html(child, indent + 1)?);
            }
            html.push_str(&format!("{}</Teleport>\n", ind));
            Ok(html)
        }
    }

    /// Emit a child of a component instantiation (sub-widget, external
    /// `use { component }`, dyn `<component :is>`, or PascalCase component).
    /// A `slot(name: "x") { ... }` element in this position targets the
    /// child's named slot → `<template #x>...</template>`; a bare
    /// `slot { ... }` unwraps its children into the default slot.
    /// Anything else is emitted normally.
    pub(super) fn slot_child_to_html(&mut self, child: &AuraNode, indent: usize) -> GenResult<String> {
        if let AuraNode::Element {
            tag,
            props,
            children,
            ..
        } = child
        {
            if tag == "slot" || tag == "Slot" {
                let ind = "  ".repeat(indent);
                if let Some(name) = props.get("name").and_then(|v| self.extract_string_value(v)) {
                    let name = name.to_string();
                    let mut html = format!("{}<template #{}>\n", ind, name);
                    for c in children {
                        html.push_str(&self.node_to_html(c, indent + 1)?);
                    }
                    html.push_str(&format!("{}</template>\n", ind));
                    return Ok(html);
                }
                let mut html = String::new();
                for c in children {
                    html.push_str(&self.node_to_html(c, indent)?);
                }
                return Ok(html);
            }
        }
        self.node_to_html(child, indent)
    }

    /// Emit an `if`/`else if`/`else` chain as flat sibling `<template>` nodes.
    ///
    /// Plan 043 M5 #3 — the parser (`parse_view_conditional`) nests an
    /// `else if` chain as `else_body = Some([Conditional(...)])`, so a chain
    /// `if A {} else if B {} else if C {} else {D}` becomes
    /// `A → else[B → else[C → else[D]]]`. This helper walks that nesting and
    /// emits one `<template>` per arm at the *same* indent (Vue requires the
    /// chain arms to be contiguous siblings with no nodes between them).
    ///
    /// `is_continuation` distinguishes the head (`v-if`) from arms deeper in
    /// the chain (`v-else-if`); a plain `else` arm closes the chain.
    pub(super) fn emit_conditional(
        &mut self,
        node: &AuraNode,
        indent: usize,
        is_continuation: bool,
    ) -> GenResult<String> {
        let AuraNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } = node
        else {
            // Not a Conditional — fall back to ordinary rendering.
            return self.node_to_html(node, indent);
        };
        let ind = "  ".repeat(indent);

        let head_attr = if is_continuation {
            format!("v-else-if=\"{}\"", self.convert_condition(condition))
        } else {
            format!("v-if=\"{}\"", self.convert_condition(condition))
        };

        let mut then_html = String::new();
        for child in then_body {
            then_html.push_str(&self.node_to_html(child, indent + 1)?);
        }

        // Tail of the chain. A single nested Conditional is a continuation
        // (another v-else-if, or a final v-else) — recurse at the SAME indent
        // so the arm stays a sibling, not a child.
        let tail = match else_body {
            Some(nodes) if nodes.len() == 1 && matches!(nodes[0], AuraNode::Conditional { .. }) => {
                self.emit_conditional(&nodes[0], indent, true)?
            }
            Some(nodes) => {
                let mut else_html = String::new();
                for child in nodes {
                    else_html.push_str(&self.node_to_html(child, indent + 1)?);
                }
                format!(
                    "{}<template v-else>\n{}{}</template>\n",
                    ind, else_html, ind
                )
            }
            None => String::new(),
        };

        Ok(format!(
            "{}<template {}>\n{}{}</template>\n{}",
            ind, head_attr, then_html, ind, tail
        ))
    }

    /// Plan 451 P2: `menubar {}`/`toolbar {}` 占位标签可否从当前 widget 的
    /// `actions {}` 块合成（块存在且非空）。
    pub(super) fn actions_menubar_available(&self) -> bool {
        self.current_actions
            .as_ref()
            .and_then(|a| a.menubar.as_ref())
            .map(|m| !m.menus.is_empty())
            .unwrap_or(false)
    }

    pub(super) fn actions_toolbar_available(&self) -> bool {
        self.current_actions
            .as_ref()
            .and_then(|a| a.toolbar.as_ref())
            .map(|t| !t.items.is_empty())
            .unwrap_or(false)
    }

    /// 用户在占位标签上给的 class/style 串（vm 侧 convert_menubar 同款
    /// 语义：class 或 style 均作为容器样式类）。
    pub(super) fn actions_placeholder_class(&self, props: &HashMap<String, AuraPropValue>) -> String {
        props
            .get("class")
            .or_else(|| props.get("style"))
            .and_then(|v| self.extract_string_value(v))
            .unwrap_or_default()
            .to_string()
    }

    /// Plan 451 P2: `menubar {}` 占位标签 → shadcn-vue Menubar 组件树。
    ///
    /// 结构镜像 vm 侧 convert_menubar（aura_view_builder）：每菜单一个
    /// MenubarMenu（trigger 文本 + content 项列），项为勾选槽 + 标题 +
    /// 右对齐快捷键；enabled_if/checked_if 经 convert_condition 转译为
    /// 模板表达式（`.store.x > 0` → `store.x > 0`，与 view if 条件同一
    /// 翻译器）。分隔线用 MenubarSeparator（docs/schema/aura.at 的 vue 行）。
    pub(super) fn generate_actions_menubar_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        indent: usize,
    ) -> GenResult<String> {
        let block = self
            .current_actions
            .clone()
            .expect("actions_menubar_available checked");
        let menubar = block.menubar.expect("actions_menubar_available checked");

        for comp in [
            "Menubar",
            "MenubarMenu",
            "MenubarTrigger",
            "MenubarContent",
            "MenubarItem",
            "MenubarSeparator",
        ] {
            self.shadcn_components_used.insert(comp.to_string());
        }

        let user_class = self.actions_placeholder_class(props);
        let class_attr = if user_class.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", Self::escape_html_attr(&user_class))
        };

        let mut html = format!("{}<Menubar{}>\n", "  ".repeat(indent), class_attr);
        let menu_ind = "  ".repeat(indent + 1);
        let item_ind = "  ".repeat(indent + 4);

        for menu in &menubar.menus {
            html.push_str(&format!(
                "{}<MenubarMenu value=\"{}\">\n",
                menu_ind,
                Self::escape_html_attr(&menu.id)
            ));
            html.push_str(&format!(
                "{}  <MenubarTrigger>{}</MenubarTrigger>\n",
                menu_ind,
                Self::escape_html_text(&menu.title)
            ));
            html.push_str(&format!("{}  <MenubarContent>\n", menu_ind));
            for item in &menu.items {
                match item {
                    auto_ui_dialect::ast::MenuItemEntry::Sep => {
                        html.push_str(&format!("{item_ind}<MenubarSeparator />\n"));
                    }
                    auto_ui_dialect::ast::MenuItemEntry::Action(id) => {
                        let Some(a) = block.actions.iter().find(|a| a.id == *id) else {
                            continue;
                        };
                        let handler = self.pattern_to_handler_name(&a.handler);
                        let mut attrs = vec![format!("@click=\"{}\"", handler)];
                        if let Some(cond) = a.enabled_if.as_deref() {
                            attrs
                                .push(format!(":disabled=\"!({})\"", self.convert_condition(cond)));
                        }
                        // 勾选槽：与 vm 同构（16px 槽 + lucide check）。
                        let check = match a.checked_if.as_deref() {
                            Some(cond) => {
                                self.lucide_icons.insert("Check".to_string());
                                format!(
                                    "{item_ind}  <span class=\"w-4 h-3 shrink-0 inline-flex items-center\">\
                                     <Check v-if=\"{}\" class=\"h-3 w-3 text-zinc-200\" /></span>\n",
                                    self.convert_condition(cond)
                                )
                            }
                            None => String::new(),
                        };
                        let shortcut = match a.shortcut.as_deref() {
                            Some(s) if !s.is_empty() => format!(
                                "{item_ind}  <span class=\"ml-auto text-[11px] text-zinc-500\">{}</span>\n",
                                Self::escape_html_text(s)
                            ),
                            _ => String::new(),
                        };
                        html.push_str(&format!(
                            "{item_ind}<MenubarItem {}>\n{}{item_ind}  <span>{}</span>\n{}{item_ind}</MenubarItem>\n",
                            attrs.join(" "),
                            check,
                            Self::escape_html_text(a.title.as_deref().unwrap_or("")),
                            shortcut,
                        ));
                    }
                }
            }
            html.push_str(&format!("{}  </MenubarContent>\n", menu_ind));
            html.push_str(&format!("{}</MenubarMenu>\n", menu_ind));
        }
        html.push_str(&format!("{}</Menubar>\n", "  ".repeat(indent)));
        Ok(html)
    }

    /// PLAN-677 T-03: 视图声明式 menubar 族（PLAN-630）→ shadcn Menubar
    /// 组件树。视图 DSL 的 menubar-menu/trigger/content/item/separator/
    /// checkbox-item 节点在 Element 通用发射里曾因 registry 查找键 miss
    /// 退化裸 div（下拉不可用），补齐别名后也只剩 :title 裸属性——
    /// 项内 icon/标题/快捷键需要真正的子组件组装。本发射器逐 kind 组装
    /// （镜像 generate_actions_menubar_html 的项渲染：icon→lucide 子组件、
    /// title→span、shortcut→右对齐 span、enabled→:disabled 取反；
    /// checkbox 的 checked→:checked 单向绑定，视觉随 store 态刷新——
    /// 与 vm convert_menubar_component 语义同源）。
    /// 返回 None = 非 menubar 族 kind，调用方落回通用发射。
    pub(super) fn generate_menubar_view_node(
        &mut self,
        kind: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<Option<String>> {
        // Element 路径 props 形态：HashMap<String, AuraPropValue>。
        // 嵌套 fn——闭包形式在返回借用时生命周期推不动。
        fn prop_str<'a>(props: &'a HashMap<String, AuraPropValue>, name: &str) -> Option<&'a str> {
            props.get(name).and_then(|v| match v {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s2)) => Some(s2.as_str()),
                _ => None,
            })
        }
        fn prop_expr<'a>(
            props: &'a HashMap<String, AuraPropValue>,
            name: &str,
        ) -> Option<&'a auto_lang_core::ast::Expr> {
            props.get(name).and_then(|v| match v {
                AuraPropValue::Expr(e) => Some(e),
                _ => None,
            })
        }
        let norm = kind.replace('-', "_");
        if !matches!(
            norm.as_str(),
            "menubar_menu"
                | "menubar_trigger"
                | "menubar_content"
                | "menubar_item"
                | "menubar_separator"
                | "menubar_checkbox_item"
                // PLAN-695 T-03: 族补全至 shadcn 16 元素。
                | "menubar_label"
                | "menubar_shortcut"
                | "menubar_radio_group"
                | "menubar_radio_item"
                | "menubar_sub"
                | "menubar_sub_trigger"
                | "menubar_sub_content"
                | "menubar_group"
        ) {
            return Ok(None);
        }
        let ind = "  ".repeat(indent);
        let inner_ind = "  ".repeat(indent + 1);

        // onclick → @click + handler 注册（与通用 Component 臂同法）。
        let mut click_attr = String::new();
        if let Some(aura_event) = events.get("onclick") {
            let handler_fn =
                self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            self.used_handlers.insert(handler_name);
            click_attr = format!("@click=\"{}\" ", handler_fn);
        }

        let html = match norm.as_str() {
            "menubar_menu" => {
                self.shadcn_components_used
                    .insert("MenubarMenu".to_string());
                let value_attr = match prop_str(props, "value") {
                    Some(v) => format!(" value=\"{}\"", Self::escape_html_attr(v)),
                    None => String::new(),
                };
                let mut out = format!("{}<MenubarMenu{}>\n", ind, value_attr);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarMenu>\n", ind));
                out
            }
            "menubar_trigger" => {
                self.shadcn_components_used
                    .insert("MenubarTrigger".to_string());
                let inner = match prop_str(props, "text") {
                    Some(t) => Self::escape_html_text(t),
                    None => {
                        let mut s = String::new();
                        for child in children {
                            s.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        s
                    }
                };
                format!("{}<MenubarTrigger>{}</MenubarTrigger>\n", ind, inner)
            }
            "menubar_content" => {
                self.shadcn_components_used
                    .insert("MenubarContent".to_string());
                let mut out = format!("{}<MenubarContent>\n", ind);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarContent>\n", ind));
                out
            }
            "menubar_separator" => {
                self.shadcn_components_used
                    .insert("MenubarSeparator".to_string());
                format!("{}<MenubarSeparator />\n", ind)
            }
            // PLAN-695 T-03: 分组标签——muted 小字非可点（auto-edit "行尾"
            // 此前被白名单静默吞）。
            "menubar_label" => {
                self.shadcn_components_used
                    .insert("MenubarLabel".to_string());
                let inner = match prop_str(props, "text") {
                    Some(t) => Self::escape_html_text(t),
                    None => {
                        let mut s = String::new();
                        for child in children {
                            s.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        s
                    }
                };
                format!("{}<MenubarLabel>{}</MenubarLabel>\n", ind, inner)
            }
            // 独立快捷键子元素（右对齐 muted token，替代手写 zinc span）。
            "menubar_shortcut" => {
                self.shadcn_components_used
                    .insert("MenubarShortcut".to_string());
                let inner = match prop_str(props, "text") {
                    Some(t) => Self::escape_html_text(t),
                    None => {
                        let mut s = String::new();
                        for child in children {
                            if let crate::aura::AuraNode::Text(content) = child {
                                match content {
                                    crate::aura::AuraTextContent::Literal(lit) => {
                                        s.push_str(&Self::escape_html_text(lit));
                                    }
                                    crate::aura::AuraTextContent::Interpolated {
                                        template,
                                        bindings,
                                    } => {
                                        let mut vue_text = template.clone();
                                        for binding in bindings {
                                            vue_text = vue_text.replace(
                                                &format!("${{{}.{}}}", ".", binding),
                                                &format!("{{{{ {} }}}}", binding),
                                            );
                                            vue_text = vue_text.replace(
                                                &format!("${{{}}}", binding),
                                                &format!("{{{{ {} }}}}", binding),
                                            );
                                        }
                                        s.push_str(&vue_text);
                                    }
                                }
                            }
                        }
                        s
                    }
                };
                format!("{}<MenubarShortcut>{}</MenubarShortcut>\n", ind, inner)
            }
            // 单选组：value 绑定 → :model-value 单向（视觉随 store 刷新，
            // 回写走 radio-item onclick——与 checkbox :checked 单向同源）。
            "menubar_radio_group" => {
                self.shadcn_components_used
                    .insert("MenubarRadioGroup".to_string());
                let value_attr = match prop_expr(props, "value") {
                    Some(v) => {
                        let expr = self.expr_to_vue_bound_value(v)?;
                        format!(" :model-value=\"{}\"", expr)
                    }
                    None => String::new(),
                };
                let mut out = format!("{}<MenubarRadioGroup{}>\n", ind, value_attr);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarRadioGroup>\n", ind));
                out
            }
            // submenu 三件套 + 语义分组：纯容器透传（reka-ui 上下文自持
            // 开合/选择状态，发射侧零状态通道）。
            "menubar_sub" => {
                self.shadcn_components_used.insert("MenubarSub".to_string());
                let mut out = format!("{}<MenubarSub>\n", ind);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarSub>\n", ind));
                out
            }
            "menubar_sub_trigger" => {
                self.shadcn_components_used
                    .insert("MenubarSubTrigger".to_string());
                let inner = match prop_str(props, "text") {
                    Some(t) => Self::escape_html_text(t),
                    None => {
                        let mut s = String::new();
                        for child in children {
                            s.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        s
                    }
                };
                format!("{}<MenubarSubTrigger>{}</MenubarSubTrigger>\n", ind, inner)
            }
            "menubar_sub_content" => {
                self.shadcn_components_used
                    .insert("MenubarSubContent".to_string());
                let mut out = format!("{}<MenubarSubContent>\n", ind);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarSubContent>\n", ind));
                out
            }
            "menubar_group" => {
                self.shadcn_components_used
                    .insert("MenubarGroup".to_string());
                let mut out = format!("{}<MenubarGroup>\n", ind);
                for child in children {
                    out.push_str(&self.node_to_html(child, indent + 1)?);
                }
                out.push_str(&format!("{}</MenubarGroup>\n", ind));
                out
            }
            item_kind => {
                // menubar_item / menubar_checkbox_item / menubar_radio_item
                // 共用项渲染（PLAN-695 扩 radio-item）。
                let (tag, is_checkbox, is_radio) = match item_kind {
                    "menubar_checkbox_item" => ("MenubarCheckboxItem", true, false),
                    "menubar_radio_item" => ("MenubarRadioItem", false, true),
                    _ => ("MenubarItem", false, false),
                };
                self.shadcn_components_used.insert(tag.to_string());
                let mut open = format!("{}<{} {}{}", ind, tag, click_attr, {
                    let mut s = String::new();
                    if let Some(v) = prop_expr(props, "enabled") {
                        let expr = self.expr_to_vue_bound_value(v)?;
                        s.push_str(&format!(":disabled=\"!({})\" ", expr));
                    }
                    // 静态 disabled 字面量（bool true）→ disabled 属性
                    //（PLAN-695: 族内三项统一）。
                    if let Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(true))) =
                        props.get("disabled")
                    {
                        s.push_str("disabled ");
                    }
                    if is_checkbox {
                        if let Some(v) = prop_expr(props, "checked") {
                            let expr = self.expr_to_vue_bound_value(v)?;
                            s.push_str(&format!(":checked=\"{}\" ", expr));
                        }
                    }
                    if is_radio {
                        if let Some(v) = prop_str(props, "value") {
                            s.push_str(&format!("value=\"{}\" ", Self::escape_html_attr(v)));
                        }
                    }
                    s
                });
                if open.ends_with(' ') {
                    open.pop();
                }
                let mut out = format!("{}>\n", open);
                if let Some(v) = prop_str(props, "icon") {
                    let lucide = Self::lucide_component_name(v);
                    self.lucide_icons.insert(lucide.clone());
                    out.push_str(&format!(
                        "{}  <{} class=\"mr-2 h-4 w-4\" />\n",
                        inner_ind, lucide
                    ));
                }
                match prop_str(props, "title") {
                    Some(t) => out.push_str(&format!(
                        "{}  <span>{}</span>\n",
                        inner_ind,
                        Self::escape_html_text(t)
                    )),
                    None => {
                        for child in children {
                            out.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                    }
                }
                if let Some(v) = prop_str(props, "shortcut") {
                    // PLAN-695 浅色修正（Vue 侧）：硬编码 text-zinc-500 span
                    // → MenubarShortcut 组件（muted-foreground token，浅色
                    // 主题下自动适配）。
                    self.shadcn_components_used
                        .insert("MenubarShortcut".to_string());
                    out.push_str(&format!(
                        "{}  <MenubarShortcut>{}</MenubarShortcut>\n",
                        inner_ind,
                        Self::escape_html_text(v)
                    ));
                }
                out.push_str(&format!("{}</{}>\n", ind, tag));
                out
            }
        };
        Ok(Some(html))
    }

    /// Plan 451 P2: `toolbar {}` 占位标签 → 从声明合成的工具栏（镜像 vm
    /// convert_toolbar）：图标按钮（lucide 组件 + title 原生 tooltip，
    /// enabled_if → :disabled），无图标则文本按钮；sep 为细分隔线。
    pub(super) fn generate_actions_toolbar_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        indent: usize,
    ) -> GenResult<String> {
        let block = self
            .current_actions
            .clone()
            .expect("actions_toolbar_available checked");
        let toolbar = block.toolbar.expect("actions_toolbar_available checked");

        self.shadcn_components_used.insert("Button".to_string());

        let user_class = self.actions_placeholder_class(props);
        let ind = "  ".repeat(indent);
        let mut html = format!(
            "{}<div class=\"flex flex-row items-center{}\">\n",
            ind,
            if user_class.is_empty() {
                String::new()
            } else {
                format!(" {}", Self::escape_html_attr(&user_class))
            }
        );

        for item in &toolbar.items {
            match item {
                auto_ui_dialect::ast::MenuItemEntry::Sep => {
                    html.push_str(&format!(
                        "{}  <span class=\"w-px h-4 bg-zinc-500 mx-1 shrink-0\" />\n",
                        ind
                    ));
                }
                auto_ui_dialect::ast::MenuItemEntry::Action(id) => {
                    let Some(a) = block.actions.iter().find(|a| a.id == *id) else {
                        continue;
                    };
                    let handler = self.pattern_to_handler_name(&a.handler);
                    let mut attrs = vec![format!("@click=\"{}\"", handler)];
                    if let Some(cond) = a.enabled_if.as_deref() {
                        attrs.push(format!(":disabled=\"!({})\"", self.convert_condition(cond)));
                    }
                    let title = a.title.as_deref().unwrap_or("");
                    attrs.push(format!("title=\"{}\"", Self::escape_html_attr(title)));
                    let content = match a.icon.as_deref() {
                        Some(icon) if !icon.is_empty() => {
                            let lucide = Self::lucide_component_name(icon);
                            self.lucide_icons.insert(lucide.clone());
                            format!("\n{}    <{} class=\"h-4 w-4\" />\n{}  ", ind, lucide, ind)
                        }
                        _ => format!("{{{{{}}}}}", Self::escape_html_text(title)),
                    };
                    let btn_class = match a.icon.as_deref() {
                        Some(icon) if !icon.is_empty() => "h-7 w-7 px-0 py-0",
                        _ => "h-7 px-2 py-0 text-[12px]",
                    };
                    html.push_str(&format!(
                        "{}  <Button variant=\"ghost\" class=\"{}\" {}>{}\
                         </Button>\n",
                        ind,
                        btn_class,
                        attrs.join(" "),
                        content
                    ));
                }
            }
        }
        html.push_str(&format!("{}</div>\n", ind));
        Ok(html)
    }

    /// Generate the media-only Vue representation of an ImageSurface.
    ///
    /// ImageSurface is deliberately emitted as a small template primitive,
    /// rather than a generic registry component: the browser receives only a
    /// media URI and the template applies the interactive transform. File
    /// access, decoding, prefetching and caching remain backend concerns.
    pub(super) fn generate_image_surface_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        // The wrapper is the crop/viewport boundary. User class/style
        // bindings stay on it so they control the viewport without replacing
        // the image transform style below.
        let (static_classes, dynamic_class, dynamic_style) =
            self.extract_classes("imagesurface", props);
        let class_str = if static_classes.trim().is_empty() {
            "relative overflow-hidden".to_string()
        } else {
            format!("relative overflow-hidden {}", static_classes.trim())
        };
        let mut wrapper_attrs = vec![format!("class=\"{}\"", Self::escape_html_attr(&class_str))];
        if let Some(class_expr) = dynamic_class {
            wrapper_attrs.push(format!(":class=\"{}\"", class_expr));
        }
        if let Some(style_expr) = dynamic_style {
            wrapper_attrs.push(format!(":style=\"{}\"", style_expr));
        }

        // String literals are ordinary HTML attributes; all other expression
        // forms use Vue bound attribute syntax and the shared AST renderer.
        let image_attr = |key: &str, value: Option<&AuraPropValue>| -> GenResult<Option<String>> {
            let Some(value) = value else { return Ok(None) };
            match value {
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))
                | AuraPropValue::Expr(auto_lang_core::ast::Expr::CStr(s))
                    if key == "src" =>
                {
                    Ok(Some(format!(
                        ":{}=\"'{}'\"",
                        key,
                        Self::escape_js_string(s.as_str())
                    )))
                }
                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s))
                | AuraPropValue::Expr(auto_lang_core::ast::Expr::CStr(s)) => Ok(Some(format!(
                    "{}=\"{}\"",
                    key,
                    Self::escape_html_attr(s.as_str())
                ))),
                AuraPropValue::Expr(expr) => Ok(Some(format!(
                    ":{}=\"{}\"",
                    key,
                    self.bound_value_or_warn(expr, &format!("ImageSurface {} prop", key), "null")
                ))),
                AuraPropValue::StyleBinding(_) => Ok(None),
            }
        };

        let mut image_attrs = vec![
            "class=\"max-w-full max-h-full select-none\"".to_string(),
            "draggable=\"false\"".to_string(),
        ];
        for key in ["src", "alt", "width", "height", "quality"] {
            if let Some(attr) = image_attr(key, props.get(key))? {
                image_attrs.push(attr);
            }
        }

        let bound_prop = |key: &str, default_value: &str| -> String {
            props
                .get(key)
                .and_then(|value| match value {
                    AuraPropValue::Expr(expr) => Some(self.bound_value_or_warn(
                        expr,
                        &format!("ImageSurface {} prop", key),
                        default_value,
                    )),
                    AuraPropValue::StyleBinding(_) => None,
                })
                .unwrap_or_else(|| default_value.to_string())
        };
        let fit = bound_prop("fit", "'contain'");
        let zoom = bound_prop("zoom", "1");
        let offset_x = bound_prop("offset_x", "0");
        let offset_y = bound_prop("offset_y", "0");
        let rotation = bound_prop("rotation", "0");
        let filter = bound_prop("filter", "'none'");
        let transform = format!(
            "'translate(' + ({}) + 'px, ' + ({}) + 'px) rotate(' + ({}) + 'deg) scale(' + ({}) + ')'",
            offset_x, offset_y, rotation, zoom
        );
        // Plan 547: 字符串形态的 :style 绑定——对象形态(嵌套括号拼接
        // 表达式)在 vue-tsc 的模板检查下类型收窄失败(StyleValue 被解析
        // 为 undefined,TS2345 实证);CSS 声明串语义等价且类型稳定。
        image_attrs.push(format!(
            ":style=\"'transform:' + ({}) + ';object-fit:' + ({}) + ';filter:' + ({}) + ';'\"",
            transform, fit, filter
        ));

        // Normalize the five ImageSurface event aliases explicitly. This is
        // independent of the shadcn registry so all spellings produce the
        // same DOM events.
        let mut sorted_events: Vec<(&String, &AuraEvent)> = events.iter().collect();
        sorted_events.sort_by(|a, b| a.0.cmp(b.0));
        for (event, aura_event) in sorted_events {
            if self.try_register_global_listener(event, aura_event) {
                continue;
            }
            let (base, modifiers) = Self::split_event_key(event);
            // Plan 547 修复:DOM 没有 pan 事件——裸 @pan 从不触发且被
            // vue-tsc 拒绝(ImgHTMLAttributes 无 onPan)。onpan 改为
            // pointer 三绑定 + script 合成包装(buttons&1 门控,start/
            // move/end 相位,handler 收 (dx, dy, phase))。
            if matches!(base.to_ascii_lowercase().as_str(), "onpan" | "pan") {
                let handler_fn = self
                    .handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                self.used_handlers
                    .insert(self.handler_to_function_call(&aura_event.handler));
                let idx = self.surface_pans.len();
                // 按事件声明参数个数决定调用形态(空声明 -> 无参调用,
                // vue-tsc 严格匹配;声明了参数 -> (dx, dy, phase) 全参)。
                let with_args = !aura_event.params.is_empty();
                self.surface_pans.push((handler_fn.clone(), with_args));
                image_attrs.push(format!("@pointerdown=\"__surfPanD_{idx}\""));
                image_attrs.push(format!("@pointermove=\"__surfPanM_{idx}\""));
                image_attrs.push(format!("@pointerup=\"__surfPanU_{idx}\""));
                continue;
            }
            let event_name = match base.to_ascii_lowercase().as_str() {
                "onerror" | "on_error" | "error" => "@error",
                "onload" | "onloaded" | "on_loaded" | "loaded" => "@load",
                "onwheel" | "wheel" => "@wheel",
                "ondblclick" | "on_double_click" | "dblclick" | "doubleclick" => "@dblclick",
                _ => continue,
            };
            let mut vue_event = event_name.to_string();
            for modifier in modifiers {
                vue_event.push('.');
                vue_event.push_str(Self::vue_modifier(modifier));
            }
            let mut handler_fn =
                self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            let handler_name = self.handler_to_function_call(&aura_event.handler);
            if let Some(ref loop_var) = self.current_loop_var {
                if aura_event.params.is_empty() {
                    handler_fn = format!("{}({})", handler_fn, loop_var);
                    self.loop_param_handlers
                        .insert(handler_name.clone(), loop_var.clone());
                }
            }
            self.used_handlers.insert(handler_name);
            image_attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        let mut html = format!("{}<div {}>\n", ind, wrapper_attrs.join(" "));
        html.push_str(&format!("{}  <img {} />\n", ind, image_attrs.join(" ")));
        // ImageSurface normally has no children, but preserving them keeps the
        // generator total for hand-built Aura trees and mirrors other elements.
        for child in children {
            html.push_str(&self.node_to_html(child, indent + 1)?);
        }
        html.push_str(&format!("{}</div>\n", ind));
        Ok(html)
    }

    /// Convert an AuraNode to its Vue template representation.
    pub(crate) fn node_to_html(&mut self, node: &AuraNode, indent: usize) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
                span,
                debug_id,
                ..
            } => {
                // Plan 012 Batch A (gap 30): a stray comma between view children
                // parses as an element with the literal tag "," and used to fall
                // through to the unknown-tag `<div />` fallback — silently
                // emitting junk spacer divs. Skip it and warn through the
                // unified codegen warning channel.
                if tag == "," {
                    self.warn(
                        "R008",
                        crate::ui_gen::validators::Severity::Warning,
                        "Stray comma between view children ignored. Commas are not \
                         valid separators between elements in a view block; the comma \
                         used to emit a junk `<div />` spacer into the template."
                            .to_string(),
                    );
                    return Ok(String::new());
                }

                // Special handling for previewcard element (supports both previewcard and preview-card)
                if tag == "previewcard" || tag == "preview-card" {
                    return self.generate_previewcard_html(props, events, children, indent);
                }

                // Special handling for codeblock element (with copy button)
                if tag == "codeblock" || tag == "code-block" {
                    return self.generate_codeblock_html(props, events, children, indent);
                }

                // ImageSurface is a backend-neutral media primitive. Keep its
                // Vue output as a URI-bound crop container instead of routing
                // it through a generic shadcn/custom component.
                let image_surface_tag = tag.replace('-', "_");
                if image_surface_tag.eq_ignore_ascii_case("imagesurface")
                    || image_surface_tag.eq_ignore_ascii_case("image_surface")
                {
                    return self.generate_image_surface_html(props, events, children, indent);
                }

                // Special handling for icon element - render as Lucide Vue component
                if tag == "icon" || tag == "Icon" {
                    let icon_name = props
                        .get("name")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or("circle");
                    // PLAN-018：iconfile:<stem> → 双主题位图 <img> 对（浅表
                    // 常显，深表由宿主根 .dark class 切换；生成器登记
                    // iconfile_theme_css 追加 SFC 切换规则）。stem 白名单同
                    // iced icon_file::parse_field（[A-Za-z0-9_-]）。
                    if let Some(stem) = icon_name.strip_prefix("iconfile:") {
                        if !stem.is_empty()
                            && stem
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                        {
                            self.iconfile_theme_css = true;
                            let (mut static_classes, _dc, _ds) = self.extract_classes(tag, props);
                            let size_px = props
                                .get("size")
                                .and_then(|v| self.extract_int_value(v))
                                .filter(|v| *v > 0);
                            let dim = if static_classes.split_whitespace().any(|c| {
                                c.starts_with("w-") || c.starts_with("h-") || c.starts_with("size-")
                            }) {
                                String::new()
                            } else {
                                let px = size_px.unwrap_or(20);
                                format!(" width=\"{}\" height=\"{}\"", px, px)
                            };
                            let base_cls = if static_classes.is_empty() {
                                String::new()
                            } else {
                                format!("{} ", static_classes)
                            };
                            return Ok(format!(
                                "{ind}<img src=\"/icons/light/{stem}.png\" class=\"{base_cls} app-icon-light\"{dim} />\n{ind}<img src=\"/icons/dark/{stem}.png\" class=\"{base_cls} app-icon-dark\"{dim} />\n",
                                ind = ind,
                                stem = stem,
                                base_cls = base_cls.trim_end(),
                                dim = dim,
                            ));
                        }
                    }
                    let lucide_component = Self::lucide_component_name(icon_name);
                    self.lucide_icons.insert(lucide_component.clone());

                    let (mut static_classes, _dynamic_class, dynamic_style) =
                        self.extract_classes(tag, props);

                    // 尺寸口径（**与 VM 端 `with_icon_size` 同一套**）：
                    //   显式 `w-*`/`h-*` 类 > `size` prop > 默认 20px。
                    // 为什么不能像以前那样无条件塞 `w-5 h-5`：Tailwind 里
                    // `w-5` 与 `w-3` 同权重，靠样式表顺序定胜负（数字小的在前），
                    // 于是用户写的 `class: "h-3 w-3"`（041-auto-edit 就是这么写
                    // 的）会被静默盖成 20px —— 同一个 `.at` 在 VM 上 12px、
                    // 在 Vue 上 20px。显式类存在时一律不加默认。
                    // 判据取自**已解析出的类集**而不是原始 `class`/`style` prop：
                    // `icon (name: …) { style: "w-5 h-5 text-white" }` 这种**块式**
                    // 样式根本不在 props 里（desktop.at 就是这么写的），只看 props
                    // 会判成「没有显式尺寸」→ 再塞一份默认类 → Tailwind 里两份
                    // 同权重类打架（金样 test_a2vue_desktop_surface_asset 抓到过）。
                    let has_wh_class = static_classes.split_whitespace().any(|c| {
                        c.starts_with("w-") || c.starts_with("h-") || c.starts_with("size-")
                    }) || _dynamic_class.is_some();
                    let size_px = props
                        .get("size")
                        .and_then(|v| self.extract_int_value(v))
                        .filter(|v| *v > 0);
                    let size_attr = match (has_wh_class, size_px) {
                        (false, Some(px)) => {
                            // `size` 是 px 数值，Tailwind 类表达不了任意值 →
                            // 走内联样式（与 VM 端推入的 Pixels(px) 等价）。
                            let mut decl = format!("width:{}px;height:{}px", px, px);
                            if let Some(extra) = dynamic_style.as_deref() {
                                if !extra.is_empty() {
                                    decl.push(';');
                                    decl.push_str(extra);
                                }
                            }
                            format!(" style=\"{}\"", decl)
                        }
                        (false, None) => {
                            if static_classes.is_empty() {
                                static_classes.push_str("w-5 h-5");
                            } else {
                                static_classes.push_str(" w-5 h-5");
                            }
                            String::new()
                        }
                        // 显式类在，就让类说话（动态 class 同理，无从判定时不越俎代庖）。
                        (true, _) => String::new(),
                    };

                    // PLAN-621: state 契约（**与 VM `with_state_tint` 同规则**）：
                    //   on → text-primary + stroke-width 基档+0.5；off →
                    //   text-muted-foreground；显式 text-* 类在场 → 颜色交给
                    //   作者（描边照常）；未声明 → 零改动（G3）。字面量走静态
                    //   class/attr，bool 绑定走 `:class`/`:stroke-width` 三元
                    //   （undefined 使 Vue 移除 attr = 不加重）。
                    // 描边基档与 VM renderer 同式：有效盒 ≥48px → 1.5，否则
                    // 2.0。有效盒 px：size prop 直读；显式类从 w-N/h-N（N×4px，
                    // 与 VM Fixed(n) 同刻度）/size-N/w-[Npx] 取尾款，判不出
                    // → 20 默认盒。
                    let mut state_attr = String::new();
                    if let Some(state_prop) = props.get("state") {
                        let has_text_class = static_classes
                            .split_whitespace()
                            .any(|c| c.starts_with("text-"))
                            || _dynamic_class
                                .as_deref()
                                .map(|d| d.contains("text-"))
                                .unwrap_or(false);
                        let eff_px = size_px.map(|v| v as f32).unwrap_or_else(|| {
                            static_classes
                                .split_whitespace()
                                .filter_map(|c| {
                                    c.strip_prefix("w-[")
                                        .and_then(|r| r.strip_suffix("px]"))
                                        .and_then(|n| n.parse::<f32>().ok())
                                        .or_else(|| {
                                            c.strip_prefix("w-")
                                                .and_then(|n| n.parse::<u16>().ok())
                                                .map(|n| n as f32 * 4.0)
                                        })
                                        .or_else(|| {
                                            c.strip_prefix("h-[")
                                                .and_then(|r| r.strip_suffix("px]"))
                                                .and_then(|n| n.parse::<f32>().ok())
                                        })
                                        .or_else(|| {
                                            c.strip_prefix("h-")
                                                .and_then(|n| n.parse::<u16>().ok())
                                                .map(|n| n as f32 * 4.0)
                                        })
                                        .or_else(|| {
                                            c.strip_prefix("size-")
                                                .and_then(|n| n.parse::<u16>().ok())
                                                .map(|n| n as f32 * 4.0)
                                        })
                                })
                                .fold(20.0_f32, f32::max)
                        });
                        let base = if eff_px >= 48.0 { 1.5 } else { 2.0 };
                        let heavy = base + 0.5;
                        match self.extract_string_value(state_prop) {
                            Some("on") => {
                                if !has_text_class {
                                    if static_classes.is_empty() {
                                        static_classes.push_str("text-primary");
                                    } else {
                                        static_classes.push_str(" text-primary");
                                    }
                                }
                                state_attr = format!(" :stroke-width=\"{}\"", heavy);
                            }
                            Some("off") => {
                                if !has_text_class {
                                    if static_classes.is_empty() {
                                        static_classes.push_str("text-muted-foreground");
                                    } else {
                                        static_classes.push_str(" text-muted-foreground");
                                    }
                                }
                            }
                            _ => {
                                // bool/变量绑定 → 三元动态发射。
                                let js = match state_prop {
                                    AuraPropValue::Expr(expr) => {
                                        self.bound_value_or_warn(expr, "icon state", "false")
                                    }
                                    _ => "false".to_string(),
                                };
                                if !has_text_class {
                                    state_attr = format!(
                                        " :class=\"{} ? 'text-primary' : 'text-muted-foreground'\"",
                                        js
                                    );
                                }
                                state_attr.push_str(&format!(
                                    " :stroke-width=\"{} ? {} : undefined\"",
                                    js, heavy
                                ));
                            }
                        }
                    }
                    let class_str = if static_classes.is_empty() {
                        String::new()
                    } else {
                        format!(" class=\"{}\"", static_classes)
                    };

                    if children.is_empty() {
                        return Ok(format!(
                            "{}<{}{}{}{} />\n",
                            ind, lucide_component, class_str, size_attr, state_attr
                        ));
                    } else {
                        let mut html = format!(
                            "{}<{}{}{}{}>\n",
                            ind, lucide_component, class_str, size_attr, state_attr
                        );
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, lucide_component));
                        return Ok(html);
                    }
                }

                // Special handling for dyn element — dynamic component:
                // dyn (.item.icon) { size: 16, class: "..." } →
                // <component :is="item.icon" :size="16" class="..." />
                if tag == "dyn" {
                    return self.generate_dyn_component_html(props, events, children, indent);
                }

                // Plan 563: canvas 元素 —— 状态驱动画布(原生 <canvas> 2D;
                // 场景契约双表渲染,redraw/pen 包装由 script 臂消费生成)。
                if tag == "canvas" {
                    return self.generate_canvas_html(props, events, indent);
                }

                // PLAN-617 T-07: 受控 `video` 元素（§2.3 受控媒体契约）。
                // **只在声明了受控下行 prop 时**走这条臂；否则返回 None，
                // 继续走下方通用路径 —— 未声明受控 prop 的 video 生成结果
                // 与引入契约之前逐字节一致（既有示例零回归）。
                if tag == "video" || tag == "Video" {
                    if let Some(html) =
                        self.try_generate_controlled_video_html(props, events, indent)?
                    {
                        return Ok(html);
                    }
                }

                // PLAN-617 后续：可拖拽进度条（`progress` + `onseek`）。
                // 同样只在**声明了 onseek** 时进入——未声明时原样走通用路径，
                // 既有 `progress { value: … }` 的生成结果逐字节不变。
                if (tag == "progress" || tag == "Progress") && !events.is_empty() {
                    if let Some(html) =
                        self.try_generate_seekable_progress_html(props, events, indent)?
                    {
                        return Ok(html);
                    }
                }

                // Plan 451 P2: `menubar {}` / `toolbar {}` placeholder tags.
                // When the widget carries an `actions {}` block and the tag
                // has no explicit children, synthesize the component tree
                // from the declarations (VM parity: the iced view builder
                // composes the same structure from action_config on the vm
                // side). Explicit children keep the hand-written tree.
                // PLAN-070 T-05: no longer shadcn-mode-gated — a project that
                // renders `menubar {}/toolbar {}` against an actions block
                // has opted into the command-surface contract and must host
                // the menubar ui module + reka-ui (shadcn: off projects like
                // jade-garden front host exactly those; the synthesized
                // imports resolve against the consumer's own ui module).
                if children.is_empty() {
                    if tag == "menubar" && self.actions_menubar_available() {
                        return self.generate_actions_menubar_html(props, indent);
                    }
                    if tag == "toolbar" && self.actions_toolbar_available() {
                        return self.generate_actions_toolbar_html(props, indent);
                    }
                }

                // Slot outlet (Plan: slots): `slot` → <slot />,
                // `slot(name: "header")` → <slot name="header" />.
                // Children (if any) become Vue slot fallback content.
                // Must run before the unknown-tag div fallback in map_tag.
                // (Named-slot TARGETING at the parent side — slot(name:) used
                // as a component child — is handled by slot_child_to_html at
                // the component children emission sites below.)
                if tag == "slot" || tag == "Slot" {
                    return self.generate_slot_outlet_html(props, children, indent);
                }

                // Special handling for teleport element — Vue's built-in
                // <Teleport>: teleport (to: "body") { ... } →
                // <Teleport to="body">...</Teleport>. Must run before the
                // unknown-tag div fallback in map_tag (plan 012 P2 gap 27).
                if tag == "teleport" || tag == "Teleport" {
                    return self.generate_teleport_html(props, children, indent);
                }

                // Special handling for category-section element
                if tag == "category-section" || tag == "category_section" {
                    return self.generate_category_section_html(props, children, indent);
                }

                // Plan 548 T6: sidebar_menu_button/sidebar_menu_sub_button ——
                // to/active 扩展臂(as-child 路由多态 + $route 自动探测)。
                // 仅 shadcn 模式拦截;非 shadcn 走通用路径。
                if self.is_shadcn()
                    && (fold_eq(tag, "sidebar_menu_button")
                        || fold_eq(tag, "sidebar_menu_sub_button"))
                {
                    return self
                        .generate_sidebar_menu_button_html(tag, props, events, children, indent);
                }

                // Plan 482: nav 组件族 —— nav-item/nav-group 全量生成；
                // nav 仅在 search 开启时拦截（否则走通用容器路径）。
                if tag == "nav-item" || tag == "nav_item" {
                    return self.generate_nav_item_html(props, events, children, indent);
                }
                if tag == "nav-group" || tag == "nav_group" {
                    return self.generate_nav_group_html(props, events, children, indent);
                }
                // PLAN-677 T-03: 视图声明式 menubar 族（menubar-menu/
                // trigger/content/item/separator/checkbox-item）→ shadcn
                // 组件树（registry 别名补齐 + 专用项渲染;外层 `menubar`
                // 容器不受此臂影响,照走通用路径）。非族内 kind 返回 None。
                if tag.starts_with("menubar") && tag != "menubar" {
                    if let Some(html) =
                        self.generate_menubar_view_node(tag, props, events, children, indent)?
                    {
                        return Ok(html);
                    }
                }

                if tag == "nav" {
                    let search = props
                        .get("search")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(auto_lang_core::ast::Expr::Bool(b)) => Some(*b),
                            _ => None,
                        })
                        .unwrap_or_else(|| props.contains_key("search"));
                    if search {
                        return self.generate_nav_html(props, events, children, indent);
                    }
                }

                // PLAN-013 T4: terminal 最小只读视口——<pre> 等宽 + v-for 逐行,
                // 行内容经 Vue 文本插值(HTML 自动转义,引擎输出安全)。
                // 交互面(选中/回滚 UI/xterm.js 类)属 DEBTS 009 #4 留白,不在本臂。
                if tag == "terminal" || tag == "Terminal" {
                    let lines_expr = match props.get("lines") {
                        Some(AuraPropValue::Expr(expr)) => self.expr_to_vue_bound_value(expr)?,
                        _ => "[]".to_string(),
                    };
                    let mut out = String::new();
                    out.push_str("<pre class=\"terminal-viewport\" style=\"margin:0;background:#0c0c0c;color:#cccccc;font-family:'Cascadia Mono',Consolas,monospace;font-size:13px;line-height:1.35;padding:8px;white-space:pre;overflow:auto;min-height:10em\">");
                    out.push_str(&format!(
                        "<span v-for=\"(line, i) in ({lines_expr})\" :key=\"i\">{{{{ line }}}}&#10;</span>"
                    ));
                    out.push_str("</pre>");
                    return Ok(out);
                }

                // PLAN-618-1 (P618-1): Plan 422 坐标锚 popover 的 vue 臂重写。
                // 旧臂发 `<Popover v-model:open>`——shadcn-vue 的 PopoverRoot
                // 是无条件渲染 slot 的 provider,弹层内容因此常显内联堆叠在
                // 页面文档流里(027 首屏即现,非业务代码问题)。现自绘
                // overlay:v-if 门控(open 关即卸载)+ backdrop 点击 dismiss
                // (ondismiss)+ fixed 坐标锚(x/y),与 VM iced 臂(坐标锚
                // 面板 + 点击外部关闭)语义对齐。terminal 臂同款早退模式,
                // 不进 shadcn 装配路径(亦不注册 Popover import)。
                // auto-musk 修正(2026-09-14):臂加形态判别——只吃坐标锚定
                // 形态(声明 open/x/y/ondismiss 任一);shadcn 嵌套形态
                // (popover-trigger/content 子节点,无上述 props)原样落空走
                // map_tag 装配。此前无判别时该臂吞掉一切 popover,shadcn
                // 三件套被包进无 PopoverRoot 的 fixed 空壳——触发器/弹层
                // 双双不渲染(auto-musk workspace_selector 实证),且坐标
                // 缺省回退发非法 JS `left: 8px`(字面量后跟标识符)。
                if tag == "popover"
                    && (props.contains_key("open")
                        || props.contains_key("x")
                        || props.contains_key("y")
                        || events.contains_key("ondismiss"))
                {
                    let open_expr = props.get("open").and_then(|v| match v {
                        AuraPropValue::Expr(expr) => self.expr_to_vue_bound_value(expr).ok(),
                        _ => None,
                    });
                    let class_str = props
                        .get("class")
                        .and_then(|v| self.extract_string_value(v))
                        .unwrap_or_default();
                    let xy = match (props.get("x"), props.get("y")) {
                        (Some(AuraPropValue::Expr(xe)), Some(AuraPropValue::Expr(ye))) => {
                            match (
                                self.expr_to_vue_bound_value(xe),
                                self.expr_to_vue_bound_value(ye),
                            ) {
                                (Ok(x), Ok(y)) => {
                                    format!("left: {} + 'px', top: {} + 'px'", x, y)
                                }
                                _ => "left: '8px', top: '8px'".to_string(),
                            }
                        }
                        _ => "left: '8px', top: '8px'".to_string(),
                    };
                    let mut out = String::new();
                    // backdrop:仅当声明 ondismiss 时发射(点击空白处关闭)。
                    if let Some(ev) = events.get("ondismiss") {
                        let handler = self.handler_to_function_call(&ev.handler);
                        self.used_handlers.insert(handler.clone());
                        out.push_str(&format!(
                            "<div v-if=\"{}\" class=\"fixed inset-0 z-40\" @click=\"{}\"></div>\n",
                            open_expr.clone().unwrap_or_else(|| "true".to_string()),
                            handler
                        ));
                    }
                    let panel_open = open_expr.unwrap_or_else(|| "true".to_string());
                    out.push_str(&format!(
                        "<div v-if=\"{}\" class=\"fixed z-50 {}\" :style=\"{{ {} }}\">\n",
                        panel_open, class_str, xy
                    ));
                    for child in children {
                        out.push_str(&self.node_to_html(child, indent + 1)?);
                    }
                    out.push_str("</div>\n");
                    return Ok(out);
                }

                // Check if this is a known sub-widget (custom component, not shadcn)
                // Plan 435 P4:与 map_tag 同源的折叠桥接(kebab↔Pascal;
                // 内置可解析的 tag 不桥接,builtin 优先)
                let is_known_sub_widget = self.known_sub_widgets.contains(tag)
                    || (!tag_is_builtin(tag)
                        && tag_has_component_shape(tag)
                        && self.known_sub_widgets.iter().any(|w| fold_eq(w, tag)));

                // Check if this is an external component declared via the
                // widget `use { component: ... }` block — bound generically
                // like a sub-widget (all props v-bind, events as @emit).
                let is_external_component = self.ext_components.contains_key(tag);

                // Check if this is a shadcn-vue component
                // Note: We need to check both the original tag and lowercase version because registry uses lowercase keys
                let tag_lower = tag.to_lowercase();
                // If user provides a class prop on form elements, force native HTML
                // (e.g., TodoMVC needs <input type="checkbox" class="toggle"> not <Checkbox>)
                let has_user_class = props.contains_key("class") || props.contains_key("style");
                // "button" is intentionally NOT in this list: a styled button
                // should still render as shadcn <Button> (with the extra class
                // overlaid) so it keeps the primary theme color. Only form
                // elements that need native HTML for their behavior (type attr,
                // TodoMVC `class="toggle"` on a checkbox) stay here.
                let force_native_elements = ["checkbox", "input", "textarea"];
                let force_native =
                    has_user_class && force_native_elements.contains(&tag_lower.as_str());

                // Plan 502 M1: svg 子树内 text → SVG <text> 直通(diagram 标签
                // 机制)。html_tag 定为 "text"(map_tag 从不返回该值,无歧义),
                // 并在下文强制 plain 路径——shadcn span 臂与 svg 命名空间语义
                // 不兼容。子树外 text→span 行为不变。
                let is_svg_text = (tag == "text" || tag == "Text") && self.in_svg_subtree;
                // Determine HTML tag: when force_native, use plain HTML; otherwise map_tag handles shadcn
                let html_tag = if is_svg_text {
                    "text".to_string()
                } else if force_native {
                    match tag_lower.as_str() {
                        "checkbox" => "input".to_string(),
                        _ => tag_lower.clone(),
                    }
                } else {
                    self.map_tag(tag, children.is_empty())
                };
                // Plan 502 M1: svg 上下文传播——进入 svg 族元素(含 svg text)
                // 时置位,子节点递归期可判;4 个发射尾出口统一复原(见各
                // return 点)。svg 族元素不命中中途早退出口(component 路径
                // /textarea overlay 均按 tag 排除),复原面完备。
                let svg_ctx_saved = self.in_svg_subtree;
                if Self::is_svg_element(html_tag.as_str()) || is_svg_text {
                    self.in_svg_subtree = true;
                }
                let is_spacer = tag_lower == "spacer";
                let is_shadcn_component = !is_svg_text
                    && !is_spacer
                    && !is_known_sub_widget
                    && !is_external_component
                    && !force_native
                    && self.shadcn_component_name(tag).is_some();

                // For known sub-widgets and external (widget `use`) components,
                // use component-style prop passing
                if is_known_sub_widget || is_external_component {
                    let mut attrs = Vec::new();
                    // Track first prop expression for :key binding
                    let mut first_prop_expr: Option<String> = None;
                    // Pass all props as v-bind (:prop="expr" needs a JS expression, not template text).
                    // Plan 015 P0#3: sorted iteration — props is a HashMap
                    // (RandomState); sorted keeps generated attribute order
                    // stable across builds (jade gap 56).
                    {
                        let mut sorted_bind: Vec<(&String, &AuraPropValue)> =
                            props.iter().collect();
                        sorted_bind.sort_by(|a, b| a.0.cmp(b.0));
                        for (key, value) in sorted_bind {
                            // Template ref on a child component: `ref: "canvasRef"`
                            // → static `ref="canvasRef"` attribute + a `ref<any>`
                            // declaration in <script setup> (the child's
                            // defineExpose surface is unknown to the parent).
                            // Lets handlers call exposed methods via `.canvasRef`.
                            if key == "ref" {
                                let ref_name: String = match value {
                                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(name)) => {
                                        name.to_string()
                                    }
                                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                                        name.to_string()
                                    }
                                    _ => continue,
                                };
                                if !ref_name.is_empty() {
                                    if !self.template_refs.contains(&ref_name) {
                                        self.template_refs.push(ref_name.clone());
                                    }
                                    self.component_ref_names.insert(ref_name.clone());
                                    attrs.push(format!("ref=\"{}\"", ref_name));
                                }
                                continue;
                            }
                            // Inline style object on a component: style_obj → :style="{...}"
                            if key == "style_obj" {
                                if let AuraPropValue::StyleBinding(bindings) = value {
                                    attrs.push(format!(
                                        ":style=\"{}\"",
                                        self.style_obj_to_vue(bindings)
                                    ));
                                }
                                continue;
                            }
                            // v-show visibility directive on a component:
                            // show: .cond → v-show="cond" (instance stays mounted).
                            if key == "show" {
                                if let AuraPropValue::Expr(expr) = value {
                                    let cond = self.expr_to_vue_bound_value(expr)?;
                                    attrs.push(format!("v-show=\"{}\"", cond));
                                }
                                continue;
                            }
                            // PLAN-037 T5: call-site model addressing — a prop
                            // name matching the sub-widget's model var is a
                            // two-way channel (v-model), never a plain :prop.
                            if is_known_sub_widget {
                                let slot = match value {
                                    AuraPropValue::Expr(expr) => self.model_channel_slot(expr),
                                    _ => None,
                                };
                                if let Some(attr) = self.try_model_channel_attr(tag, key, slot)? {
                                    attrs.push(attr);
                                    continue;
                                }
                            }
                            // NOTE: `html:` is deliberately NOT intercepted as
                            // v-html on components (same reason as the dyn path):
                            // it must pass through as a plain `:html` prop so ext
                            // functional components (innerHTML pattern) keep
                            // receiving it. Only plain HTML elements get v-html.
                            let value_str = match value {
                                // Plan 012 P0#13 follow-up: warn R013 + keep the
                                // old `null` fallback instead of failing the
                                // whole widget for one unsupported prop expr.
                                AuraPropValue::Expr(expr) => self.bound_value_or_warn(
                                    expr,
                                    &format!("component prop `{}`", key),
                                    "null",
                                ),
                                AuraPropValue::StyleBinding(_) => "\"\"".to_string(),
                            };
                            if first_prop_expr.is_none() {
                                first_prop_expr = Some(value_str.clone());
                            }
                            attrs.push(format!(":{}=\"{}\"", key, value_str));
                        }
                    }
                    // Add :key binding for component reuse identity.
                    //
                    // We use a per-widget-instance counter so each component usage site
                    // gets a unique, STABLE key (e.g. 'AutoDownEditor-0', 'AutoDownEditor-1').
                    // This is important because:
                    //   1. Two components with the same name in different v-if branches
                    //      must NOT share a key — otherwise Vue patches in place instead
                    //      of mounting a fresh instance (breaks Tiptap editor init).
                    //   2. The key must stay stable across prop updates so components
                    //      are reused (not destroyed/recreated) when only props change
                    //      (prevents Tiptap unmount errors on note switch).
                    //   3. Inside a for-loop, items need per-item keys for correct diff.
                    //
                    // Explicit override: a `key:` prop on the instantiation
                    // (e.g. `EditorTab(key: tab.path, ...)`) is emitted above as
                    // `:key="tab.path"` and WINS — no auto-key is added (a second
                    // :key would be a duplicate attribute).
                    let has_explicit_key = props.contains_key("key");
                    self.widget_key_counter += 1;
                    if has_explicit_key {
                        // explicit :key already emitted with the other props
                    } else if let Some(ref loop_var) = self.current_loop_var {
                        if self.current_loop_var_is_index {
                            // Index var is a primitive int — `i?.id` is meaningless;
                            // use the index itself as the per-item key.
                            attrs.push(format!(
                                ":key=\"'{}-{}-' + {}\"",
                                html_tag, self.widget_key_counter, loop_var
                            ));
                        } else {
                            attrs.push(format!(
                                ":key=\"'{}-{}-' + ((({} as any)?.id ?? {}))\"",
                                html_tag, self.widget_key_counter, loop_var, loop_var
                            ));
                        }
                    } else if let Some(ref expr) = first_prop_expr {
                        // Non-loop component: if the first prop looks like an object
                        // reference (contains '[' for index access, suggesting an array
                        // element like store.notes[idx]), bind key to its .id so the
                        // component REMOUNTS when the underlying object changes.
                        // Skip for primitive props (search: str, active_id: int) — those
                        // don't have .id and would cause TS errors.
                        if expr.contains('[') {
                            attrs.push(format!(
                                ":key=\"'{}-{}-' + ({}?.id ?? 'new')\"",
                                html_tag, self.widget_key_counter, expr
                            ));
                        } else {
                            attrs.push(format!(
                                ":key=\"'{}-{}'\"",
                                html_tag, self.widget_key_counter
                            ));
                        }
                    } else {
                        attrs.push(format!(
                            ":key=\"'{}-{}'\"",
                            html_tag, self.widget_key_counter
                        ));
                    }
                    // Event handlers
                    let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
                    sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
                    for (event, aura_event) in sorted_ev {
                        // .window/.document modifiers → global listener, no template attr
                        if self.try_register_global_listener(event, aura_event) {
                            continue;
                        }
                        // Plan 444 (①b): resolve against the child's emit
                        // roster when available (pass-through fallback).
                        let vue_event =
                            self.sub_widget_callback_event_to_vue(tag, event, aura_event);
                        let mut handler_fn = self.handler_to_function_call_with_params(
                            &aura_event.handler,
                            &aura_event.params,
                        );
                        let handler_name = self.handler_to_function_call(&aura_event.handler);
                        // If inside a for-loop, pass the loop variable's .id as argument
                        // Only append if handler doesn't already have params from aura_event
                        if let Some(ref loop_var) = self.current_loop_var {
                            if aura_event.params.is_empty() {
                                // Plan 043 H2: decide what to auto-pass as the handler arg.
                                // If the parent's msg variant for this handler takes a PAYLOAD
                                // (e.g. `.Rerun(str)`), the child emits the value and the
                                // binding must forward `$event` — NOT the loop var (which would
                                // clobber the emitted command/path with the block object).
                                // If the variant takes no payload (e.g. `.Stop`, or index-based
                                // `.SelectNote(i)` where the loop var IS the intended arg), fall
                                // back to the legacy loop-var behavior.
                                let handler_takes_payload = self
                                    .msg_payload_arities
                                    .get(&handler_name)
                                    .map(|n| *n > 0)
                                    .unwrap_or(false);
                                if handler_takes_payload {
                                    handler_fn = format!("{}($event)", handler_fn);
                                } else {
                                    handler_fn = format!("{}({})", handler_fn, loop_var);
                                    // Plan 345: only register as a loop-param handler when we
                                    // actually auto-pass the loop var. A handler with explicit
                                    // args (e.g. .SelectNote(note.id)) must keep its declared
                                    // param name, not be renamed to the loop variable.
                                    self.loop_param_handlers
                                        .insert(handler_name.clone(), loop_var.clone());
                                }
                            }
                        }
                        self.used_handlers.insert(handler_name);
                        let handler_fn = Self::ime_guard_enter_handler(&vue_event, handler_fn);
                        attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                    }
                    // Auto-wire child Delete event to parent handler
                    // If parent has an array state (e.g., 'notes'), generate @delete handler
                    if self.state_names.iter().any(|n| n == "notes") {
                        attrs.push("@delete=\"handleChildDelete\"".to_string());
                        self.used_handlers.insert("handleChildDelete".to_string());
                        self.needs_child_delete_handler = true;
                    }
                    let attr_str = if attrs.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", attrs.join(" "))
                    };
                    // Component with children
                    if children.is_empty() {
                        return Ok(format!("{}<{}{} />\n", ind, html_tag, attr_str));
                    } else {
                        let mut html = format!("{}<{}{}>\n", ind, html_tag, attr_str);
                        for child in children {
                            // Component children: slot(name:) elements target named slots.
                            html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        return Ok(html);
                    }
                }

                // Build attributes
                // Plan 410: when a child Text node is hoisted into slot_content
                // below, record its index so we skip it during child emission —
                // otherwise the text would render twice (inline + as a child).
                let mut consumed_text_child_idx: Option<usize> = None;
                let (mut attrs, text_content, generated_children) = if is_shadcn_component {
                    // Use shadcn-specific attribute generation (includes event handling)
                    let (shadcn_attrs, mut slot_content, slot_children) =
                        self.generate_shadcn_attrs(tag, props, events);
                    // Plan 354: if no slot content from props, check primary positional text.
                    // The view parser puts positional text (e.g. `badge t`) in props as "text"
                    // OR as a text child node. Check both.
                    if slot_content.is_none() {
                        // Check if "text" prop exists (positional text stored as named prop)
                        if let Some(value) = props.get("text") {
                            slot_content = self.prop_to_text_content(value).ok();
                        }
                        // Also check children for a text node
                        if slot_content.is_none() {
                            for (i, child) in children.iter().enumerate() {
                                if let AuraNode::Text(content) = child {
                                    match content {
                                        AuraTextContent::Literal(s) => {
                                            // PLAN-528 W5: 提升进 slot 的文本同样
                                            // 是纯文本通道，按 HTML 转义。
                                            slot_content = Some(Self::escape_html_text(s));
                                            consumed_text_child_idx = Some(i);
                                            break;
                                        }
                                        AuraTextContent::Interpolated { template, bindings } => {
                                            // Convert to Vue text
                                            let mut vue_text = template.clone();
                                            for binding in bindings {
                                                vue_text = vue_text.replace(
                                                    &format!("${{{}.{}}}", ".", binding),
                                                    &format!("{{{{ {} }}}}", binding),
                                                );
                                                vue_text = vue_text.replace(
                                                    &format!("${{{}}}", binding),
                                                    &format!("{{{{ {} }}}}", binding),
                                                );
                                            }
                                            slot_content = Some(vue_text);
                                            consumed_text_child_idx = Some(i);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    (shadcn_attrs, slot_content, slot_children)
                } else {
                    // Use plain Tailwind attribute generation
                    let mut attrs = Vec::new();
                    let mut text_content: Option<String> = None;

                    // Class attribute (both static and dynamic)
                    let (static_classes, dynamic_class, dynamic_style) =
                        self.extract_classes(tag, props);
                    if !static_classes.is_empty() {
                        attrs.push(format!("class=\"{}\"", static_classes));
                    }
                    // Plan 408 P12 §10.1: style and class now have independent
                    // channels — no more __style__ marker / strip_prefix.
                    // PLAN-656 T-07/T-08: scroll-pane plain 模式增强——
                    // controller 寻址锚 + hidden 滚动条内联样式（现代浏览器
                    // scrollbar-width:none；滚动能力保留——§8.3 无命中目标）；
                    // scroll-test-content = logical spacer（managed bridge v1:
                    // 内联逻辑尺寸 + 条纹背景，节点数恒 1，不随逻辑 extent
                    // 膨胀——§10.4-5）。
                    if matches!(tag.as_str(), "scroll" | "scrollable" | "scroll-pane") {
                        // F-781-R02（T-20）：所有权标记——全部 scroll-pane 元素
                        // 发射（含无 controller 者）；helper 以
                        // closest('[data-scroll-pane]')===pane 判最近 pane 归属，
                        // 嵌套 pane 的目标/managed 标记不被外层认领。
                        attrs.push("data-scroll-pane=\"1\"".to_string());
                        if let Some(AuraPropValue::Expr(expr)) = props.get("controller") {
                            self.scroll_ctl_helpers_needed = true;
                            let bound = self.bound_value_or_warn(
                                expr,
                                "element `scroll-pane` prop `controller`",
                                "''",
                            );
                            attrs.push(format!(":data-scroll-ctl=\"{}\"", bound));
                        }
                        if let Some(v) = props.get("scrollbar") {
                            if self.extract_string_value(v).as_deref() == Some("hidden") {
                                attrs.push("style=\"scrollbar-width:none\"".to_string());
                            }
                        }
                    }
                    if matches!(tag.as_str(), "scroll-test-content" | "scroll_test_content") {
                        // PLAN-781 T-04: managed 桥标记——scroll_to_child 以
                        // pane 内此标记判 unsupported（与 iced 侧 managed 入队
                        // 拒绝同形的 vue 轨可观察面）。
                        attrs.push("data-scroll-managed=\"1\"".to_string());
                        let get_num = |key: &str, default: &'static str| -> String {
                            props
                                .get(key)
                                .and_then(|v| self.extract_string_value(v))
                                .unwrap_or(default)
                                .to_string()
                        };
                        let w = get_num("extent_w", "2000000");
                        let h = get_num("extent_h", "10000000");
                        attrs.push(format!(
                            "style=\"width:{w}px;height:{h}px;background:repeating-linear-gradient(to bottom, rgba(255,255,255,0.05) 0 2px, transparent 2px 40px);background-color:rgb(18,23,28)\""
                        ));
                    }
                    if let Some(dc) = dynamic_class {
                        attrs.push(format!(":class=\"{}\"", dc));
                    }
                    if let Some(ds) = dynamic_style {
                        attrs.push(format!(":style=\"{}\"", ds));
                    }

                    // Auto-add type attribute for checkbox (native HTML needs type="checkbox")
                    let tag_lower_for_type = tag.to_lowercase();
                    if tag_lower_for_type == "checkbox" {
                        attrs.push("type=\"checkbox\"".to_string());
                    }
                    // PLAN-661 T-04（R-5）：slider → 原生 <input type="range">
                    //（shadcn Slider 组件路径退役；type 静态注入，checkbox 同款）。
                    if tag_lower_for_type == "slider" {
                        attrs.push("type=\"range\"".to_string());
                    }

                    // Track value state ref for v-model optimization
                    // When input has both :value="stateRef" and @input handler,
                    // use v-model instead (native HTML two-way binding)
                    let mut value_state_ref: Option<String> = None;

                    // Props as attributes
                    {
                        let mut sorted_pr: Vec<(&String, &AuraPropValue)> = props.iter().collect();
                        // Plan 015 P0#3: sorted iteration (props is a HashMap, jade gap 56)
                        sorted_pr.sort_by(|a, b| a.0.cmp(b.0));
                        for (key, value) in sorted_pr {
                            if key == "class" || key == "style" {
                                continue; // Already handled in extract_classes
                            }
                            // PLAN-007 T-05：元素 design 对象 → 内联 :style 绑定
                            //（槽引用 tokens.* 点链 → var(--css-var) 随主题/模式
                            // 联动；直接字面量 → 具体值不受主题影响。与 VM 轨
                            // design 对象同语义、各自投影）。
                            if key == "design" {
                                if let AuraPropValue::Expr(expr) = value {
                                    if let Some(binding) = design_obj_to_vue(expr) {
                                        attrs.push(format!(
                                            ":style=\"{}\"",
                                            binding
                                        ));
                                    }
                                }
                                continue;
                            }
                            // Inline style object: style_obj: { top: f"${.y}px", "z-index": 50 }
                            // → :style="{ top: `${y}px`, 'z-index': 50 }"
                            if key == "style_obj" {
                                if let AuraPropValue::StyleBinding(bindings) = value {
                                    attrs.push(format!(
                                        ":style=\"{}\"",
                                        self.style_obj_to_vue(bindings)
                                    ));
                                }
                                continue;
                            }
                            // v-show visibility directive: show: .cond → v-show="cond"
                            // (element stays mounted; only inline display toggles).
                            if key == "show" {
                                if let AuraPropValue::Expr(expr) = value {
                                    let cond = self.expr_to_vue_bound_value(expr)?;
                                    attrs.push(format!("v-show=\"{}\"", cond));
                                }
                                continue;
                            }
                            // v-html content directive: html: .content → v-html="content"
                            // (raw HTML injection; inner content is ignored by Vue).
                            if key == "html" {
                                if let AuraPropValue::Expr(expr) = value {
                                    let content = self.expr_to_vue_bound_value(expr)?;
                                    attrs.push(format!("v-html=\"{}\"", content));
                                }
                                continue;
                            }
                            // Template ref: `ref: "menuEl"` → static `ref="menuEl"`
                            // attribute + a `const menuEl = ref<HTMLElement | null>(null)`
                            // declaration in <script setup> (emitted later from
                            // self.template_refs). Accessible in `on` handlers as
                            // `.menuEl` (→ `menuEl.value!`). When the tag is a Vue
                            // component (PascalCase), the ref is typed `any` — the
                            // child's defineExpose surface is unknown here.
                            if key == "ref" {
                                let ref_name: String = match value {
                                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(name)) => {
                                        name.to_string()
                                    }
                                    AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) => {
                                        name.to_string()
                                    }
                                    _ => continue,
                                };
                                if !ref_name.is_empty() && !self.template_refs.contains(&ref_name) {
                                    self.template_refs.push(ref_name.clone());
                                }
                                if html_tag
                                    .chars()
                                    .next()
                                    .map(|c| c.is_uppercase())
                                    .unwrap_or(false)
                                {
                                    self.component_ref_names.insert(ref_name.clone());
                                }
                                attrs.push(format!("ref=\"{}\"", ref_name));
                                continue;
                            }
                            if key == "gap" {
                                continue; // Handled in extract_classes for layout elements
                            }
                            // PLAN-781 T-04: scroll-anchor → data-scroll-anchor
                            // 寻址锚（pane 归属 = 最近 ordinary scroll-pane；
                            // helper 族由此置位——无 controller 的 pane 上
                            // 属性无操作，但 to_child 探测面需要 helper 在场）。
                            if key == "scroll-anchor" {
                                self.scroll_ctl_helpers_needed = true;
                                self.push_scroll_anchor_attr(&mut attrs, value, tag);
                                continue;
                            }
                            // PLAN-781 T-04（真实路径探针发现的既有泄漏）：
                            // scroll 族的 controller 已在上方特例消费为
                            // :data-scroll-ctl，通用 v-bind 臂不得再发孤儿
                            // :controller（Vue 会把它落成 DOM attribute）。
                            if key == "controller"
                                && matches!(tag.as_str(), "scroll" | "scrollable" | "scroll-pane")
                            {
                                continue;
                            }
                            // Plan 412 §4.3: square 的自有 props 已由 extract_classes
                            // 转成 h-/w-/bg-… 类 —— 不透传为无效绑定属性。
                            if tag == "square"
                                && matches!(key.as_str(), "color" | "size" | "h" | "w")
                            {
                                continue;
                            }
                            if key == "text" {
                                text_content = Some(self.prop_to_text_content(value)?);
                                continue;
                            }
                            // Plan 481: selectable: true → 显式 user-select: text
                            // (防应用级 none 吞掉;不 continue,保留下方通用
                            // :selectable 绑定透传作 a2vue 金样锚点)。
                            if key == "selectable" && self.extract_bool_value(value) {
                                attrs.push("style=\"user-select: text\"".to_string());
                            }
                            // Special handling for codeblock's code prop - render as content
                            if key == "code" && (tag == "codeblock" || tag == "code-block") {
                                text_content = Some(self.prop_to_text_content(value)?);
                                continue;
                            }

                            // Plan 057 续(富文本输入): textarea 的 highlight/ghost 由
                            // 叠加层包裹消费(见 textarea_rich_overlay),不透传为无效属性。
                            // PLAN-493: mentions/mention_class 同理由
                            // textarea_mentions_overlay 消费;height 为 VM-only
                            // 几何 prop(Plan 053 契约,浏览器轨道 CSS 几何自理),
                            // 一并不透传。
                            if tag == "textarea"
                                && (key == "highlight"
                                    || key == "ghost"
                                    || key == "mentions"
                                    || key == "mention_class"
                                    || key == "height")
                            {
                                continue;
                            }

                            // PLAN-661 T-04：slider 数值属性面（原生 range 语义）
                            //——value 状态引用绑定/字面量静态；min/max/step 静态
                            // 数值（缺省 0/100/1），不走通用 v-bind 透传。
                            if tag == "slider"
                                && matches!(key.as_str(), "value" | "min" | "max" | "step")
                            {
                                if key == "value" {
                                    if let Some(model) = self.extract_state_ref(value) {
                                        attrs.push(format!(":value=\"{}\"", model));
                                    } else if let Some(f) = self.extract_float_value(value) {
                                        attrs.push(format!("value=\"{}\"", f));
                                    }
                                } else {
                                    let default = match key.as_str() {
                                        "min" => 0.0,
                                        "max" => 100.0,
                                        _ => 1.0,
                                    };
                                    let v = self.extract_float_value(value).unwrap_or(default);
                                    attrs.push(format!("{}=\"{}\"", key, v));
                                }
                                continue;
                            }

                            // Checkbox: native <input type="checkbox"> uses :checked, not :model-value
                            if tag == "checkbox" && key == "checked" {
                                if let Some(model) = self.extract_state_ref(value) {
                                    attrs.push(format!(":checked=\"{}\"", model));
                                } else if let AuraPropValue::Expr(expr) = value {
                                    match self.expr_to_vue_bound_value(expr) {
                                    Ok(js_expr) => attrs.push(format!(":checked=\"{}\"", js_expr)),
                                    // Plan 012 P0#13 follow-up: was silently
                                    // dropped; warn R013 (no attr emitted).
                                    Err(e) => self.warn(
                                        "R013",
                                        crate::ui_gen::validators::Severity::Warning,
                                        format!("checkbox `checked` binding: {}; binding not emitted", e),
                                    ),
                                }
                                }
                                continue;
                            }

                            // Plan 442 A4: SVG 元素的字面量属性按静态 attribute 发射
                            // (viewBox="0 0 24 24" 而非 :viewBox="'0 0 24 24'"——
                            // 绑定形式在 svg 子树上会被 Vue 求值,静态化才保住
                            // SVG 语义;musk-038 T9 canary 的退化根因)。动态表达
                            // 式仍走下方 v-bind。
                            // Plan 502 M1: svg 子树内 text 同待遇(html_tag=="text"
                            // 只可能来自 is_svg_text 分流,x/y/fill 静态化)。
                            if Self::is_svg_element(html_tag.as_str()) || html_tag == "text" {
                                if let AuraPropValue::Expr(expr) = value {
                                    if let Some(lit) = Self::svg_static_attr_value(expr) {
                                        attrs.push(format!("{}=\"{}\"", key, lit));
                                        continue;
                                    }
                                }
                            }

                            // Use v-bind (:attr) for dynamic values, static quotes for literals
                            if let AuraPropValue::Expr(auto_lang_core::ast::Expr::Ident(name)) =
                                value
                            {
                                // Track value state ref for v-model optimization on input elements
                                if key == "value" && (tag == "input" || tag == "textarea") {
                                    value_state_ref = Some(name.to_string());
                                }
                                attrs.push(format!(":{}=\"{}\"", key, name));
                            } else if let AuraPropValue::Expr(expr) = value {
                                // Plan 351: ALL expression prop values (FieldAccess,
                                // Index, etc.) use v-bind with bound JS value (no {{ }}).
                                // Plan 012 P0#13 follow-up: warn R013 + keep the
                                // old `null` fallback rather than fail the widget.
                                let value_str = self.bound_value_or_warn(
                                    expr,
                                    &format!("element `{}` prop `{}`", tag, key),
                                    "null",
                                );
                                // Also track value ref for v-model optimization
                                if key == "value" && (tag == "input" || tag == "textarea") {
                                    // Handle both Expr::Ident(".xxx") and Expr::Dot(Ident("self"), "xxx")
                                    match expr {
                                        auto_lang_core::ast::Expr::Ident(name) => {
                                            let resolved = if name.starts_with('.') {
                                                &name[1..]
                                            } else {
                                                name.as_str()
                                            };
                                            value_state_ref = Some(resolved.to_string());
                                        }
                                        auto_lang_core::ast::Expr::Dot(obj, field) => {
                                            if let auto_lang_core::ast::Expr::Ident(obj_name) =
                                                obj.as_ref()
                                            {
                                                if obj_name == "self" {
                                                    value_state_ref = Some(field.to_string());
                                                }
                                            }
                                        }
                                        // Plan 028 F5: dynamic-key v-model —
                                        // `value: .answers[q.id]` folds to
                                        // `v-model="answers[q.id]"` (gap G8).
                                        auto_lang_core::ast::Expr::Index(target, index) => {
                                            if let Some(t) = Self::index_target_name(target) {
                                                if let Ok(idx_js) =
                                                    self.expr_to_vue_bound_value(index)
                                                {
                                                    value_state_ref =
                                                        Some(format!("{}[{}]", t, idx_js));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                attrs.push(format!(":{}=\"{}\"", key, value_str));
                            } else {
                                let value_str = self.prop_to_attr_value(value)?;
                                attrs.push(format!("{}={}", key, value_str));
                            }
                        }
                    } // plan-015 sorted-props block

                    // Event handlers
                    let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
                    sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
                    for (event, aura_event) in sorted_ev {
                        // .window/.document modifiers → global listener, no template attr
                        if self.try_register_global_listener(event, aura_event) {
                            continue;
                        }
                        // v-model optimization: when input/textarea has both :value="stateRef"
                        // and @input handler, replace with v-model (native HTML two-way binding)
                        //
                        // Plan 041a⑤(musk 041 Phase 5 T22): value 目标是 prop 时
                        // 不折 v-model——对 defineProps 只读绑定发射 v-model 是
                        // 编译红(v-model 要求可写 model 变量),保持 :value 单向。
                        let model_target_is_prop = value_state_ref
                            .as_ref()
                            .map(|r| self.prop_names.iter().any(|p| p == r))
                            .unwrap_or(false);
                        if (event == "oninput" || event == "onInput")
                            && value_state_ref.is_some()
                            && !model_target_is_prop
                        {
                            // Replace the :value binding with v-model
                            let model_ref = value_state_ref.as_ref().unwrap();
                            // Remove the existing :value attribute and add v-model instead
                            if let Some(pos) = attrs.iter().position(|a| a.starts_with(":value=\""))
                            {
                                attrs[pos] = format!("v-model=\"{}\"", model_ref);
                            } else {
                                attrs.push(format!("v-model=\"{}\"", model_ref));
                            }
                            // Plan 399 Phase 12: still emit @input so handlers with
                            // side effects (e.g. typing-signal InputChanged) run —
                            // v-model only handles the two-way value binding, not
                            // arbitrary handler logic. Vue allows v-model + @input.
                            //
                            // Plan 008 (auto-os-config dual-backend input contract):
                            // a handler DECLARING exactly one param receives the
                            // typed TEXT on the vm backend — emit
                            // $event.target.value so vue matches. 0-param handlers
                            // read bound state (v-model) and stay bare calls
                            // (DEBTS 引擎窗口 #6: arity comes from the handler's
                            // declared params, not the binding site).
                            let handler_fn = if matches!(html_tag.as_str(), "input" | "textarea")
                                && self.input_text_handler_wants_text_arg(aura_event)
                            {
                                format!(
                                    "{}(($event.target as HTMLInputElement).value)",
                                    self.handler_to_function_call(&aura_event.handler)
                                )
                            } else {
                                self.handler_to_function_call_with_params(
                                    &aura_event.handler,
                                    &aura_event.params,
                                )
                            };
                            // Plan 448 C: auto-sync mints have no side effects —
                            // v-model alone carries the sync, and registering
                            // the name would emit a stub fn + dead emits entry.
                            if !aura_event
                                .handler
                                .trim_start_matches('.')
                                .starts_with("__bind_")
                            {
                                let handler_name =
                                    self.handler_to_function_call(&aura_event.handler);
                                self.used_handlers.insert(handler_name);
                                attrs.push(format!("@input=\"{}\"", handler_fn));
                            }
                            continue;
                        }
                        let vue_event = if html_tag
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                        {
                            // PascalCase tag — a custom component rendered via
                            // map_tag's fallback (Phase 1 front files have no
                            // known_sub_widgets, so sibling sub-widgets land
                            // here). It emits its msg variant names, so `on_*`
                            // callback props must bind to `@Pascal` just like
                            // known sub-widgets (Plan 043 M5 R4). Plan 444
                            // (①b): resolve against the child's emit roster
                            // when the driver provided one (pass-through
                            // fallback for convention-broken prop names).
                            self.sub_widget_callback_event_to_vue(tag, event, aura_event)
                        } else {
                            self.auto_event_to_vue(event)
                        };
                        // Plan 408 P10 / §7.3 缺陷 4: callback prop short-circuit.
                        if let Some(attr) = self.try_callback_prop_attr(aura_event, &vue_event) {
                            attrs.push(attr);
                            continue;
                        }
                        // PLAN-656 T-07: scroll-pane onscroll → 8 位置实参
                        //（顺序 = record 字段序 offset_x..progress_y，与 VM 端
                        // aura_view_builder 双端同形契约一致）。
                        if matches!(tag.as_str(), "scroll" | "scrollable" | "scroll-pane")
                            && Self::split_event_key(event).0 == "onscroll"
                        {
                            let call = self.handler_to_function_call_with_params(
                                &aura_event.handler,
                                &aura_event.params,
                            );
                            self.used_handlers
                                .insert(self.handler_to_function_call(&aura_event.handler));
                            attrs.push(format!(
                                "@scroll=\"e => {{ const el = e.currentTarget as HTMLElement; {}(el.scrollLeft, el.scrollTop, el.clientWidth, el.clientHeight, el.scrollWidth, el.scrollHeight, Math.min(1, el.scrollLeft / Math.max(1, el.scrollWidth - el.clientWidth)), Math.min(1, el.scrollTop / Math.max(1, el.scrollHeight - el.clientHeight))) }}\"",
                                call
                            ));
                            continue;
                        }
                        // PLAN-661 T-04: slider onchange → @input 载荷派发
                        //（valueAsNumber = f32 语义，对齐 a2r/VM 轨 .SetVol(v
                        // float)；不用 v-model 双向——回写走 msg 派发链，状态
                        // 驱动 :value 重渲染闭环，input 臂 041a⑤ 同款纪律）。
                        if matches!(tag.as_str(), "slider" | "Slider")
                            && matches!(
                                Self::split_event_key(event).0.to_ascii_lowercase().as_str(),
                                "onchange" | "change"
                            )
                        {
                            let handler_name = self.handler_to_function_call(&aura_event.handler);
                            self.used_handlers.insert(handler_name.clone());
                            attrs.push(format!(
                                "@input=\"{}(($event.target as HTMLInputElement).valueAsNumber)\"",
                                handler_name
                            ));
                            continue;
                        }
                        // Plan 499 M2: mouse-area onmousemove → 内联箭头换算逻辑
                        // 坐标(引擎层=生成端完成屏幕→逻辑换算,与 iced PointerArea
                        // 同一坐标语义;coords "WxH" 声明逻辑幅面,缺省 = raw px)。
                        // 必须先于循环变量自动传参臂(该臂会给 handler 追加 (b))。
                        if (tag == "mouse-area" || tag == "mouse_area")
                            && Self::split_event_key(event).0 == "onmousemove"
                        {
                            let call = self.handler_to_function_call_with_params(
                                &aura_event.handler,
                                &aura_event.params,
                            );
                            self.used_handlers
                                .insert(self.handler_to_function_call(&aura_event.handler));
                            let extent = props.get("coords").and_then(|v| match v {
                                AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)) => {
                                    s.split_once(['x', 'X']).and_then(|(w, h)| {
                                        match (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                                            (Ok(w), Ok(h)) if w > 0.0 && h > 0.0 => Some((w, h)),
                                            _ => None,
                                        }
                                    })
                                }
                                _ => None,
                            });
                            // PLAN-043 T9: 坐标取 currentTarget 矩形相对值（经
                            // clientX/Y - rect）——e.offsetX/Y 相对事件 target，
                            // mouse-area 带子元素时（CustomScrollbar thumb）拖拽
                            // 期 target=子元素、offsetY 退化为子元素局部坐标。
                            // 无子元素的既有消费方（charts 覆盖层）两式等价。
                            // PLAN-043 T10 续: currentTarget 在 vue-tsc 模板检查
                            // 下是 EventTarget|null（getBoundingClientRect 不在
                            // 类型上）——as HTMLElement 收窄（regen 门禁此前用
                            // 主仓二进制生成旧箭头而未暴露）。
                            let arrow = match extent {
                                Some((w, h)) => format!(
                                    "e => {}((e.clientX - (e.currentTarget as HTMLElement).getBoundingClientRect().left) / (e.currentTarget as HTMLElement).clientWidth * {}, \
                                     (e.clientY - (e.currentTarget as HTMLElement).getBoundingClientRect().top) / (e.currentTarget as HTMLElement).clientHeight * {})",
                                    call, w, h
                                ),
                                None => format!(
                                    "e => {}(e.clientX - (e.currentTarget as HTMLElement).getBoundingClientRect().left, e.clientY - (e.currentTarget as HTMLElement).getBoundingClientRect().top)",
                                    call
                                ),
                            };
                            attrs.push(format!("@mousemove=\"{}\"", arrow));
                            continue;
                        }
                        let mut handler_fn = self.handler_to_function_call_with_params(
                            &aura_event.handler,
                            &aura_event.params,
                        );
                        // Track used handler (without params for matching)
                        let handler_name = self.handler_to_function_call(&aura_event.handler);
                        // If inside a for-loop and the handler doesn't already have params,
                        // pass the loop variable's .id as argument (e.g., SelectNote(note.id))
                        if let Some(ref loop_var) = self.current_loop_var {
                            if aura_event.params.is_empty() {
                                // Plan 043 H2: decide what to auto-pass as the handler arg.
                                // If the parent's msg variant for this handler takes a PAYLOAD
                                // (e.g. `.Rerun(str)`), the child emits the value and the
                                // binding must forward `$event` — NOT the loop var (which would
                                // clobber the emitted command/path with the block object).
                                // If the variant takes no payload (e.g. `.Stop`, or index-based
                                // `.SelectNote(i)` where the loop var IS the intended arg), fall
                                // back to the legacy loop-var behavior.
                                let handler_takes_payload = self
                                    .msg_payload_arities
                                    .get(&handler_name)
                                    .map(|n| *n > 0)
                                    .unwrap_or(false);
                                if handler_takes_payload {
                                    handler_fn = format!("{}($event)", handler_fn);
                                } else {
                                    handler_fn = format!("{}({})", handler_fn, loop_var);
                                    // Plan 345: only register as a loop-param handler when we
                                    // actually auto-pass the loop var. A handler with explicit
                                    // args (e.g. .SelectNote(note.id)) must keep its declared
                                    // param name, not be renamed to the loop variable.
                                    self.loop_param_handlers
                                        .insert(handler_name.clone(), loop_var.clone());
                                }
                            }
                        }
                        // Plan 008 (auto-os-config dual-backend input contract):
                        // AFTER the loop-var pass — text handlers on
                        // input/textarea whose DECLARED arity is exactly 1
                        // receive the typed TEXT on vm; forward
                        // ($event.target as ...).value on vue to match
                        // (0-param / explicit-$event / multi-arg / map bindings
                        // pass through untouched — DEBTS 引擎窗口 #6 root fix).
                        if matches!(html_tag.as_str(), "input" | "textarea")
                            && matches!(
                                event.as_str(),
                                "oninput" | "input" | "onInput" | "onchange" | "change"
                            )
                            && self.input_text_handler_wants_text_arg(aura_event)
                        {
                            handler_fn = format!(
                                "{}(($event.target as HTMLInputElement).value)",
                                self.handler_to_function_call(&aura_event.handler)
                            );
                        }
                        // Plan 448 C: an auto-sync mint on input/textarea is
                        // realized by the v-model fold below — emitting the
                        // @input attr for the empty handler would only add a
                        // no-op listener and force a stub function.
                        let is_auto_sync_mint = matches!(html_tag.as_str(), "input" | "textarea")
                            && matches!(
                                event.as_str(),
                                "oninput" | "input" | "onInput" | "onchange" | "change"
                            )
                            && aura_event
                                .handler
                                .trim_start_matches('.')
                                .starts_with("__bind_");
                        if !is_auto_sync_mint {
                            self.used_handlers.insert(handler_name);
                            let handler_fn = Self::ime_guard_enter_handler(&vue_event, handler_fn);
                            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                        }
                    }

                    // PLAN-037 T5: native input/textarea with a state-slot
                    // `value:` and NO oninput handler still folds to v-model —
                    // a bare `input { value: .x }` must be two-way (the DOM
                    // input event is the writeback), never a one-way :value.
                    // (The pair-with-handler case was already handled above.)
                    let model_target_is_prop = value_state_ref
                        .as_ref()
                        .map(|r| self.prop_names.iter().any(|p| p == r))
                        .unwrap_or(false);
                    if (tag == "input" || tag == "textarea")
                        && value_state_ref.is_some()
                        && !model_target_is_prop
                        && !attrs.iter().any(|a| a.starts_with("v-model="))
                    {
                        if let Some(pos) = attrs.iter().position(|a| a.starts_with(":value=\"")) {
                            let model_ref = value_state_ref.as_ref().unwrap().clone();
                            attrs[pos] = format!("v-model=\"{}\"", model_ref);
                        }
                    }

                    // v-html overrides inner content in Vue: warn when the
                    // element also carries a `text:` prop or child nodes
                    // (both still emitted; the Vue runtime ignores them).
                    if props.contains_key("html")
                        && (text_content.is_some() || !children.is_empty())
                    {
                        self.warn(
                            "R014",
                            crate::ui_gen::validators::Severity::Warning,
                            format!("v-html on `{}`: children ignored at runtime", tag),
                        );
                    }

                    (attrs, text_content, None)
                };

                // PLAN-646: Select Anything DOM 标记注入——data-auto-*。
                // dev 运行面开启（AUTOUI_SELECT_MARKERS=1）；产物构建与
                // 既有快照测试保持逐字节不变。组件壳路径（known_sub_widget/
                // 外部组件）在上方早退不经此处——标记面由内层通用元素携带。
                if self.select_markers {
                    attrs.push(format!("data-auto-tag=\"{}\"", tag));
                    attrs.push(format!("data-auto-src=\"{}\"", self.source_stem));
                    if let Some(id) = debug_id {
                        attrs.push(format!("data-auto-id=\"aura_{}\"", id.0));
                    }
                    if let Some((off, len)) = span {
                        attrs.push(format!("data-auto-span=\"{}:{}\"", off, len));
                    }
                }

                // reka-ui TooltipTrigger renders its own <button>; as-child
                // avoids nesting when it wraps a <Button> (invalid HTML,
                // white-screen). Applied at the unified attr-assembly point so
                // it lands regardless of which tag-resolution path was taken.
                if tag == "tooltip-trigger" {
                    attrs.push("as-child".to_string());
                }
                // PLAN-692: utility 滚动路径统一滚动条——带 overflow-*-auto 语义类
                // 的普通容器自动挂 .ash-scroll（与 Reka ScrollArea 同一视觉参数表：
                // thumb --border 药丸 / hover 加宽加深 / active 主色）。语义 scroll
                // 元素走 ScrollArea 组件（shadcn 分支），不经此 plain 装配点。
                {
                    const OVERFLOW_UTILITIES: [&str; 3] =
                        ["overflow-y-auto", "overflow-x-auto", "overflow-auto"];
                    let has_overflow_utility = attrs.iter().any(|a| {
                        a.starts_with("class=\"")
                            && OVERFLOW_UTILITIES.iter().any(|c| a.contains(c))
                    });
                    if has_overflow_utility {
                        for a in attrs.iter_mut() {
                            if a.starts_with("class=\"") {
                                *a = a.replacen("class=\"", "class=\"ash-scroll ", 1);
                            }
                        }
                    }
                }
                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                // Plan 360: For custom Vue components (PascalCase tag like AutoDownEditor,
                // NavTree, EditorPanel, etc.), add a stable unique :key so that two
                // components with the same name in different v-if branches don't collide.
                // Without this, Vue patches in place and components that rely on fresh
                // mount semantics (e.g. Tiptap editor) fail to initialize.
                // The key is stable across re-renders (counter-based, reset per SFC) so
                // components are reused when only props change.
                let is_vue_component = html_tag
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let attr_str = if is_vue_component && !attr_str.contains(":key=") {
                    self.widget_key_counter += 1;
                    if let Some(ref loop_var) = self.current_loop_var {
                        if self.current_loop_var_is_index {
                            // Index var is a primitive int — `i?.id` is meaningless;
                            // use the index itself as the per-item key.
                            format!(
                                "{} :key=\"'{}-{}-' + {}\"",
                                attr_str, html_tag, self.widget_key_counter, loop_var
                            )
                        } else {
                            format!(
                                "{} :key=\"'{}-{}-' + ((({} as any)?.id ?? {}))\"",
                                attr_str, html_tag, self.widget_key_counter, loop_var, loop_var
                            )
                        }
                    } else {
                        format!(
                            "{} :key=\"'{}-{}'\"",
                            attr_str, html_tag, self.widget_key_counter
                        )
                    }
                } else {
                    attr_str
                };

                // PLAN-493: textarea 带 mentions prop 时,整体替换为
                // backdrop+透明 textarea 兄弟对(mention 高亮技法随 codegen
                // 下沉,声明层不再出现叠加结构)。mentions 优先于 highlight/
                // ghost 组合(与 VM 轨 convert_textarea 的优先级一致)。
                if let Some(overlay) =
                    self.textarea_mentions_overlay(tag, props, &attr_str, indent)?
                {
                    return Ok(overlay);
                }

                // Plan 057 续(富文本输入): textarea 带 highlight/ghost props 时,
                // 整体替换为叠加层包裹(overlay 手法下沉为 codegen 职责)。
                if let Some(overlay) = self.textarea_rich_overlay(tag, props, &attr_str, indent)? {
                    return Ok(overlay);
                }

                // Plan 053 后续: capture the scroll auto_scroll sentinel (set in
                // the scroll arm of generate_shadcn_attrs above) before any child
                // recursion below, so nested scrolls can't clobber the hand-off.
                let scroll_sentinel: Option<String> = self.pending_scroll_sentinel.take();

                // Check if we have text content (render as inline content)
                if let Some(text) = &text_content {
                    // Plan 410: a child Text node may have been hoisted into
                    // `text_content`; count only the remaining children so we
                    // don't render that text twice.
                    let has_other_children = children
                        .iter()
                        .enumerate()
                        .any(|(i, _)| Some(i) != consumed_text_child_idx);
                    if !has_other_children {
                        // <button @click="handler">text</button>
                        self.in_svg_subtree = svg_ctx_saved;
                        Ok(format!(
                            "{}<{}{}>{}</{}>\n",
                            ind, html_tag, attr_str, text, html_tag
                        ))
                    } else {
                        // Has both text and children - unusual but handle it
                        let mut html = format!("{}<{}{}>{}\n", ind, html_tag, attr_str, text);
                        for (i, child) in children.iter().enumerate() {
                            // Skip the child Text node already hoisted into `text`.
                            if Some(i) == consumed_text_child_idx {
                                continue;
                            }
                            // Component children: slot(name:) targets named slots.
                            if is_vue_component {
                                html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                            } else {
                                html.push_str(&self.node_to_html(child, indent + 1)?);
                            }
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        self.in_svg_subtree = svg_ctx_saved;
                        Ok(html)
                    }
                } else if children.is_empty() && generated_children.is_none() {
                    // No children and no generated children - self-closing tag
                    self.in_svg_subtree = svg_ctx_saved;
                    Ok(format!("{}<{}{} />\n", ind, html_tag, attr_str))
                } else {
                    // Has children (from source or generated)
                    let mut html = format!("{}<{}{}>\n", ind, html_tag, attr_str);

                    // Add generated children first (e.g., AvatarImage, AvatarFallback)
                    if let Some(gen_children) = &generated_children {
                        html.push_str(&format!("{}{}", "  ".repeat(indent + 1), gen_children));
                    }

                    // Add source children
                    for child in children {
                        // Component children: slot(name:) targets named slots.
                        if is_vue_component {
                            html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                        } else {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                    }
                    // Plan 053 后续: scroll auto_scroll — inject the trailing
                    // sentinel div whose ref drives the watch(()=>props.<src>,
                    // nextTick(scrollIntoView({block:'end'}))) emitted in
                    // generate_script. Placed last so it pins the view to bottom.
                    if let Some(ref_name) = &scroll_sentinel {
                        html.push_str(&format!(
                            "{}<div ref=\"{}\"></div>\n",
                            "  ".repeat(indent + 1),
                            ref_name
                        ));
                    }
                    html.push_str(&format!("{}</{}>\n", ind, html_tag));
                    self.in_svg_subtree = svg_ctx_saved;
                    Ok(html)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        // PLAN-528 W5: 模板文本是纯文本通道——裸 `<`（如文档表格
                        // 里的泛型 `List<str>`）会被 Vue 模板解析器当成未闭合
                        // 元素，整个页面 SFC 编译 500。字面量按 HTML 转义。
                        Ok(format!("{}{}\n", ind, Self::escape_html_text(s)))
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Convert template to Vue interpolation
                        let mut vue_text = template.clone();
                        for binding in bindings {
                            // Replace ${.binding} with {{ binding }} (state reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                &format!("{{{{ {} }}}}", binding),
                            );
                            // Replace ${binding} with {{ binding }} (variable reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}}}", binding),
                                &format!("{{{{ {} }}}}", binding),
                            );
                            // Also handle $binding format (without braces)
                            vue_text = vue_text.replace(
                                &format!("${}", binding),
                                &format!("{{{{ {} }}}}", binding),
                            );
                        }
                        // Plan 351: strip `self.` prefix from Vue interpolations.
                        // The view parser emits `self.xxx` for implicit-dot field access,
                        // but Vue <script setup> has no `self` — props/state are bare names.
                        vue_text = vue_text.replace("{{ self.", "{{ ");
                        vue_text = vue_text.replace(".self.", ".");
                        Ok(format!("{}{}\n", ind, vue_text))
                    }
                }
            }

            AuraNode::ForLoop {
                var,
                index,
                iterable,
                body,
                ..
            } => {
                // Generate v-for directive
                // Auto syntax: for idx, item in list (index first, value second)
                // Vue syntax: v-for="(item, index) in list" (value first, index second)
                // So we need to swap the order for Vue
                let iterable_name = iterable.trim_start_matches('.');
                // R012 (auto-down debt #2, 2026-08-29): a single-segment root
                // member as the v-for source MUST resolve to a declared
                // prop/state/computed — a name mismatch (`.header_cells` vs
                // prop `headerCells`) otherwise emits `v-for="... in
                // header_cells"` over `undefined`: a SILENT empty render with
                // no runtime error (found live in autodown plan-023's
                // TableEditorBlock).
                if iterable.starts_with('.')
                    && !iterable_name.contains('.')
                    && iterable_name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !iterable_name.is_empty()
                    && !self.prop_names.iter().any(|n| n == iterable_name)
                    && !self.state_names.iter().any(|n| n == iterable_name)
                    && !self.computed_names.contains(iterable_name)
                {
                    self.warn(
                        "R012",
                        crate::ui_gen::validators::Severity::Error,
                        format!(
                            "v-for source `.{}` matches no declared prop/state/computed — likely a name mismatch; the generated v-for iterates undefined and renders nothing silently",
                            iterable_name
                        ),
                    );
                }
                // Auto-add search filter when widget has a 'search' state and iterates over an array
                let v_for_iterable = if self.state_names.iter().any(|n| n == "search")
                    && self.state_names.iter().any(|n| n == iterable_name)
                {
                    format!("{}.filter((n: any) => !search || n.title?.toLowerCase().includes(search.toLowerCase()))", iterable_name)
                } else {
                    iterable_name.to_string()
                };
                let v_for = if let Some(idx) = index {
                    format!("v-for=\"({}, {}) in {}\"", var, idx, v_for_iterable)
                } else {
                    format!("v-for=\"{} in {}\"", var, v_for_iterable)
                };

                // Set loop variable context so child events can pass it as arg.
                // Plan 346: When the loop has an index (for i, note in ...), use
                // the INDEX variable (i) as the loop_var — handlers like
                // SelectNote(i) pass the index, not the value.
                let prev_loop_var = self.current_loop_var.clone();
                let prev_loop_var_is_index = self.current_loop_var_is_index;
                self.current_loop_var = Some(index.clone().unwrap_or_else(|| var.clone()));
                self.current_loop_var_is_index = index.is_some();

                // If body has a single Element or Component, put v-for directly on it
                // to avoid <template> scoping issues with vue-tsc
                let result = if body.len() == 1 {
                    match &body[0] {
                        AuraNode::Element { .. } | AuraNode::Component { .. } => {
                            let child_html = self.node_to_html(&body[0], indent)?;
                            if let Some(gt_pos) = find_tag_close(&child_html) {
                                let mut result = child_html;
                                // Self-closing tag (<Foo />): insert before the
                                // '/', not between '/' and '>'.
                                let insert_pos =
                                    if gt_pos > 0 && result.as_bytes()[gt_pos - 1] == b'/' {
                                        gt_pos - 1
                                    } else {
                                        gt_pos
                                    };
                                result.insert_str(insert_pos, &format!(" {}", v_for));
                                // Plan 041a(musk 041 Phase 5 R006 收口): 快路径
                                // (单元素直挂 v-for)无 :key 时补 id 回退键——
                                // strict 模式 R006 拦截;显式 key 已在则不动。
                                let tag_end = find_tag_close(&result).unwrap();
                                let head = &result[..tag_end];
                                if !head.contains(":key=") {
                                    let key_expr =
                                        format!(" :key=\"((({} as any)?.id ?? {}))\"", var, var);
                                    let insert_pos =
                                        if tag_end > 0 && result.as_bytes()[tag_end - 1] == b'/' {
                                            tag_end - 1
                                        } else {
                                            tag_end
                                        };
                                    result.insert_str(insert_pos, &key_expr);
                                }
                                Some(Ok(result))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                if let Some(r) = result {
                    self.current_loop_var = prev_loop_var;
                    self.current_loop_var_is_index = prev_loop_var_is_index;
                    return r;
                }

                // Fallback: wrap body in a div with v-for
                // Keep current_loop_var set while processing children so
                // event handlers inside the loop get proper loop-param tracking
                let mut body_html = String::new();
                for child in body {
                    body_html.push_str(&self.node_to_html(child, indent + 1)?);
                }
                self.current_loop_var = prev_loop_var;
                self.current_loop_var_is_index = prev_loop_var_is_index;
                // Plan 008 (auto-os-config batch 4): hoist a declared child key
                // onto the v-for wrapper. Mixed-if loop bodies carry keys on
                // branch-first texts (the vm-safe position — container keys
                // kill the vm subtree) and R006 requires a keyed wrapper.
                //
                // Plan 041a(musk 041 Phase 5 R006 收口): 提升失败时回退索引
                // key——if-in-for 体(分支内 button 等经 shadcn 组件路径的自
                // 键)无可提升形态,v-for 包装缺 :key 触发 R006(strict 拦截)。
                // 索引 key 是位置身份,弱于 id key 但为合法 Vue 绑定。
                let (v_for, key_attr) = match find_loop_child_key(body) {
                    Some(k) => (v_for, format!(" :key=\"{}\"", k)),
                    None => {
                        let idx_var = "__for_idx";
                        let with_idx = v_for.replace(
                            &format!("v-for=\"{} in", var),
                            &format!("v-for=\"({}, {}) in", var, idx_var),
                        );
                        if with_idx == v_for {
                            // 带 index 形态(for i, x)已含两参——用既有 index 名
                            (
                                v_for.clone(),
                                format!(
                                    " :key=\"{}\"",
                                    index.clone().unwrap_or_else(|| "__for_idx".into())
                                ),
                            )
                        } else {
                            (with_idx, format!(" :key=\"{}\"", idx_var))
                        }
                    }
                };
                // Plan 502 M6: svg 子树内循环包装用 <template v-for>——
                // <div> 会破坏 SVG 命名空间(rect/text 等 svg 子元素在
                // div 容器内不渲染;diagram 节点循环实测)。<template>
                // 是 Vue 无 DOM 包装正解,HTML 上下文行为等价。
                // PLAN-677 T-01: 单 Conditional 体循环同样透明化——
                // div 包装自身是 flex item,内容被隔在包装盒内,flex-1/
                // h-full 够不着祖父弹性容器(消费方 auto-edit 编辑器
                // `for i, t in .tabs { if t.key == .active_key { code_editor } }`
                // 高度塌缩实测);单 if 的 v-if 已在子 <template> 上,
                // 包装层无盒化不改作用域。多语句体维持 div 包装(多子
                // 迭代依赖包装盒的块级布局语义,防既有消费方回归)。
                let transparent_wrap =
                    body.len() == 1 && matches!(body[0], AuraNode::Conditional { .. });
                let (wrap_open, wrap_close) = if self.in_svg_subtree || transparent_wrap {
                    (format!("<template {}{}>", v_for, key_attr), "</template>")
                } else {
                    (format!("<div {}{}>", v_for, key_attr), "</div>")
                };
                Ok(format!(
                    "{}{}\n{}{}{}\n",
                    ind, wrap_open, body_html, ind, wrap_close
                ))
            }

            AuraNode::Conditional { .. } => {
                // Plan 043 M5 #3: delegate to the recursive helper that flattens
                // if/else-if/else chains into sibling <template> nodes (v-if /
                // v-else-if / v-else). Previously this branch re-entered itself
                // via node_to_html for chain continuations, which unconditionally
                // re-emitted a top-level <template v-if> and produced wrongly
                // nested <template v-else><template v-if>.
                self.emit_conditional(node, indent, false)
            }

            AuraNode::Component {
                name,
                props,
                events,
                children,
                ..
            } => {
                // Build props as bindings
                let mut attrs = Vec::new();
                for (key, value) in props {
                    // Template ref escape hatch on a child component:
                    // `ref: "canvasRef"` → static `ref="canvasRef"` attribute
                    // + a script-setup ref declaration (typed `any` — the
                    // child's defineExpose surface is unknown here). Lets
                    // handlers call exposed methods via `.canvasRef`.
                    if key == "ref" {
                        let ref_name = match value {
                            auto_lang_core::ast::Expr::Str(n)
                            | auto_lang_core::ast::Expr::Ident(n) => n.to_string(),
                            _ => String::new(),
                        };
                        if !ref_name.is_empty() {
                            if !self.template_refs.contains(&ref_name) {
                                self.template_refs.push(ref_name.clone());
                            }
                            self.component_ref_names.insert(ref_name.clone());
                            attrs.push(format!("ref=\"{}\"", ref_name));
                        }
                        continue;
                    }
                    // PLAN-037 T5: call-site model addressing for component
                    // fn children (same contract as the sub-widget path).
                    {
                        let slot = self.model_channel_slot(value);
                        if let Some(attr) = self.try_model_channel_attr(name, key, slot)? {
                            attrs.push(attr);
                            continue;
                        }
                    }
                    // Plan 408 P6 / §7.1 缺陷 1+2: use the binding-mode
                    // translator (expr_to_vue_bound_value) so literal props
                    // render correctly — bool→true, str→'second', Dot→self
                    // stripped. The previous prop_to_attr_value path used the
                    // text-mode translator, leaking mustaches (`{{ true }}`),
                    // dropping quotes (`second`), and keeping `self.` prefixes.
                    let value_str = self.expr_to_vue_bound_value(value)?;
                    attrs.push(format!(":{}=\"{}\"", key, value_str));
                }

                // Event handlers
                let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
                sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
                for (event, aura_event) in sorted_ev {
                    let vue_event = self.auto_event_to_vue(event);
                    // Plan 408 P10: callback-prop short-circuit (consistency
                    // with the element/shadcn event paths).
                    if let Some(attr) = self.try_callback_prop_attr(aura_event, &vue_event) {
                        attrs.push(attr);
                        continue;
                    }
                    let handler_fn = self.handler_to_function_call_with_params(
                        &aura_event.handler,
                        &aura_event.params,
                    );
                    // Track used handler (without params for matching)
                    let handler_name = self.handler_to_function_call(&aura_event.handler);
                    self.used_handlers.insert(handler_name);
                    let handler_fn = Self::ime_guard_enter_handler(&vue_event, handler_fn);
                    attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                }

                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                self.component_refs.push(name.clone());
                // Plan 408 P11: render slot children when present. A child
                // `slot(name:"x")` becomes `<template #x>…</template>`; plain
                // children go to the default slot. Mirrors the sub-widget path
                // (is_known_sub_widget children handling, ~:3433).
                if children.is_empty() {
                    Ok(format!("{}<{}{} />\n", ind, name, attr_str))
                } else {
                    let mut html = format!("{}<{}{}>\n", ind, name, attr_str);
                    for child in children {
                        html.push_str(&self.slot_child_to_html(child, indent + 1)?);
                    }
                    html.push_str(&format!("{}</{}>\n", ind, name));
                    Ok(html)
                }
            }

            // Plan 105: Router outlet and link
            AuraNode::Outlet { .. } => {
                // Vue Router outlet: <router-view />
                self.needs_router = true;
                Ok(format!("{}<router-view />\n", ind))
            }

            // PLAN-046: memo is a VM-track cache boundary — Vue diffing is
            // incremental on its own; emit the block's children inline.
            AuraNode::MemoBlock { body, .. } => {
                let mut out = String::new();
                for c in body {
                    out.push_str(&self.node_to_html(c, indent)?);
                }
                Ok(out)
            }

            AuraNode::Link {
                to,
                text,
                href,
                children,
                ..
            } => {
                // Handle different link types:
                // 1. External link with href: <a href="...">
                // 2. Router link with to: <router-link to="...">
                if !href.is_empty() {
                    // External link
                    let text_content = if text.is_empty() {
                        let mut children_html = String::new();
                        for child in children {
                            children_html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        children_html
                    } else {
                        text.clone()
                    };
                    Ok(format!(
                        "{}<a href=\"{}\">{}</a>\n",
                        ind,
                        href,
                        text_content.trim()
                    ))
                } else {
                    // Vue Router link
                    self.needs_router = true;
                    let children_html = if text.is_empty() {
                        let mut html = String::new();
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html
                    } else {
                        text.clone()
                    };
                    // Vue Router link. Plan 409 §8: links use the theme color
                    // (text-primary) — matches the VM link rendering.
                    Ok(format!("{}<router-link to=\"{}\" class=\"group block text-primary\" active-class=\"\" exact-active-class=\"router-link-exact-active\">\n{}{}</router-link>\n", ind, to, children_html, ind))
                }
            }
        }
    }

    /// Generate HTML for interactive previewcard element
    /// Plan 409 §10 续 19: 给 VM preview-card 用 —— 从 children 生成 Auto/Vue 代码文本。
    /// 复用 generate_previewcard_html 的代码生成逻辑(本文件 4361-4395)。VM builder
    /// 用 throwaway VueGenerator 实例调用,副作用不外泄。
    pub fn gen_previewcard_code(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
    ) -> (String, String) {
        let auto_code = if let Some(value) = props.get("auto") {
            self.extract_string_value(value)
                .unwrap_or_default()
                .to_string()
        } else {
            let mut parts = Vec::new();
            for child in children {
                parts.push(self.node_to_auto_code(child, 0));
            }
            let g = parts.join("\n");
            if g.is_empty() {
                "// Auto code not provided".to_string()
            } else {
                g
            }
        };
        let vue_code = if let Some(value) = props.get("vue") {
            self.extract_string_value(value)
                .unwrap_or_default()
                .to_string()
        } else {
            let mut parts = Vec::new();
            for child in children {
                match self.node_to_html(child, 0) {
                    Ok(h) => parts.push(h),
                    Err(_) => parts.push("<!-- Error generating code -->".to_string()),
                }
            }
            let g = parts.join("\n");
            if g.is_empty() {
                "// Vue code not provided".to_string()
            } else {
                g
            }
        };
        (auto_code, vue_code)
    }

    pub(super) fn generate_previewcard_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        // Extract props
        let id = if let Some(value) = props.get("id") {
            self.extract_string_value(value)
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    self.previewcard_counter += 1;
                    format!("preview{}", self.previewcard_counter)
                })
        } else {
            self.previewcard_counter += 1;
            format!("preview{}", self.previewcard_counter)
        };

        // Capitalize first letter for variable names
        // Convert kebab-case to PascalCase (e.g., "card-basic" -> "CardBasic")
        let id_cap = id
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        // Also create a lowercase version for code variable names (still camelCase)
        let id_lower = id
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
            .collect::<String>();

        // Generate Auto code from children if not provided
        let auto_code = if let Some(value) = props.get("auto") {
            self.extract_string_value(value)
                .unwrap_or_default()
                .to_string()
        } else {
            // Auto-generate Auto code from children
            let mut auto_code_parts = Vec::new();
            for child in children {
                auto_code_parts.push(self.node_to_auto_code(child, 0));
            }
            let generated = auto_code_parts.join("\n");
            if generated.is_empty() {
                "// Auto code not provided".to_string()
            } else {
                generated
            }
        };

        // Generate Vue code from children if not provided
        let vue_code = if let Some(value) = props.get("vue") {
            self.extract_string_value(value)
                .unwrap_or_default()
                .to_string()
        } else {
            // Auto-generate Vue code from children
            let mut vue_code_parts = Vec::new();
            for child in children {
                match self.node_to_html(child, 0) {
                    Ok(html) => vue_code_parts.push(html),
                    Err(_) => vue_code_parts.push("<!-- Error generating code -->".to_string()),
                }
            }
            let generated = vue_code_parts.join("\n");
            if generated.is_empty() {
                "// Vue code not provided".to_string()
            } else {
                generated
            }
        };

        // Store previewcard data for script generation
        self.previewcard_data.push(PreviewCardData {
            id: id_cap.clone(),
            auto_code: auto_code.clone(),
            vue_code: vue_code.clone(),
        });
        self.needs_copy_code = true;

        // Generate children HTML for preview area
        let mut children_html = String::new();
        for child in children {
            children_html.push_str(&self.node_to_html(child, indent + 3)?);
        }

        // Generate the full previewcard HTML
        let html = format!(
            r#"{ind}<!-- Merged {id_cap} Component -->
{ind}<div class="rounded-lg border overflow-hidden">
{ind}  <!-- Preview Area -->
{ind}  <div class="flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900">
{ind}    {children_html}{ind}  </div>
{ind}  <!-- Toolbar: language tabs left, copy + toggle right (Plan 411 merged row) -->
{ind}  <div class="flex items-center justify-between border-t bg-zinc-100 dark:bg-zinc-800">
{ind}    <div class="flex">
{ind}      <button
{ind}        @click="active{id_cap}Tab = 'auto'"
{ind}        :class="active{id_cap}Tab === 'auto' ? 'bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px' : 'text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent'"
{ind}        class="px-4 py-2 text-xs font-medium transition-colors"
{ind}      >
{ind}        Auto
{ind}      </button>
{ind}      <button
{ind}        @click="active{id_cap}Tab = 'vue'"
{ind}        :class="active{id_cap}Tab === 'vue' ? 'bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px' : 'text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent'"
{ind}        class="px-4 py-2 text-xs font-medium transition-colors"
{ind}      >
{ind}        Vue
{ind}      </button>
{ind}    </div>
{ind}    <div class="flex items-center gap-1 pr-2">
{ind}      <button
{ind}        @click="copyCode(active{id_cap}Tab === 'auto' ? {id_lower}AutoCode : {id_lower}VueCode, '{id}')"
{ind}        :title="copiedCode === '{id}' ? 'Copied!' : 'Copy code'"
{ind}        class="inline-flex items-center rounded-md px-2 py-1.5 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
{ind}      >
{ind}        <svg v-if="copiedCode !== '{id}'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
{ind}        <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
{ind}      </button>
{ind}      <button
{ind}        @click="show{id_cap}Code = !show{id_cap}Code"
{ind}        :title="show{id_cap}Code ? 'Hide code' : 'Show code'"
{ind}        class="inline-flex items-center rounded-md px-2 py-1.5 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
{ind}      >
{ind}        <svg
{ind}          :class="show{id_cap}Code ? 'rotate-180' : ''"
{ind}          xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
{ind}          class="transition-transform duration-200"
{ind}        >
{ind}          <path d="m6 9 6 6 6-6"/>
{ind}        </svg>
{ind}      </button>
{ind}    </div>
{ind}  </div>
{ind}  <!-- Expandable Code Block -->
{ind}  <div v-if="show{id_cap}Code" class="border-t">
{ind}    <!-- Code content with syntax highlighting -->
{ind}    <pre class="overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"><code :class="'block font-mono !p-0 language-' + (active{id_cap}Tab === 'auto' ? 'auto' : 'html')">{{{{ active{id_cap}Tab === 'auto' ? {id_lower}AutoCode : {id_lower}VueCode }}}}</code></pre>
{ind}  </div>
{ind}</div>
"#,
            ind = ind,
            id = id,
            id_cap = id_cap,
            id_lower = id_lower,
            children_html = children_html
        );

        Ok(html)
    }

    /// Generate HTML for codeblock element with copy button
    pub(super) fn generate_codeblock_html(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        // Extract id prop or generate one
        let id = if let Some(value) = props.get("id") {
            self.extract_string_value(value)
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    self.codeblock_counter += 1;
                    format!("codeblock{}", self.codeblock_counter)
                })
        } else {
            self.codeblock_counter += 1;
            format!("codeblock{}", self.codeblock_counter)
        };

        // Extract lang prop (default: "text")
        let lang = if let Some(value) = props.get("lang") {
            self.extract_string_value(value)
                .unwrap_or("text")
                .to_string()
        } else {
            "text".to_string()
        };

        // Extract code content from props or children
        let code = if let Some(value) = props.get("code") {
            self.prop_to_text_content(value).unwrap_or_default()
        } else if let Some(value) = props.get("text") {
            self.prop_to_text_content(value).unwrap_or_default()
        } else {
            // Get text from children
            let mut code_parts = Vec::new();
            for child in children {
                if let AuraNode::Text(content) = child {
                    match content {
                        AuraTextContent::Literal(s) => code_parts.push(s.clone()),
                        AuraTextContent::Interpolated { template, .. } => {
                            code_parts.push(template.clone())
                        }
                    }
                }
            }
            code_parts.join("\n")
        };

        // Convert kebab-case to camelCase for variable names (e.g., "install-button" -> "installButton")
        let id_camel = id
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
            .collect::<String>();

        // Store codeblock data for script generation
        self.codeblock_data.push(CodeBlockData {
            id: id.clone(),
            code: code.clone(),
            lang: lang.clone(),
        });
        self.needs_copy_code = true;

        // Generate the codeblock HTML with copy button (gray title bar, dark code content)
        let html = format!(
            r#"{ind}<div class="relative rounded-lg border overflow-hidden">
{ind}  <div class="flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b">
{ind}    <span class="text-xs text-zinc-600 dark:text-zinc-400 font-medium">{lang}</span>
{ind}    <button
{ind}      @click="copyCode({id_camel}Code, '{id}')"
{ind}      class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
{ind}    >
{ind}      <svg v-if="copiedCode !== '{id}'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
{ind}      <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
{ind}      {{{{ copiedCode === '{id}' ? 'Copied!' : 'Copy' }}}}
{ind}    </button>
{ind}  </div>
{ind}  <pre class="p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"><code class="block font-mono !p-0 language-{lang}">{{{{ {id_camel}Code }}}}</code></pre>
{ind}</div>
"#,
            ind = ind,
            id = id,
            id_camel = id_camel,
            lang = lang
        );

        Ok(html)
    }

    /// Convert AuraNode back to Auto source code string
    /// This is used to generate the Auto code for previewcard components
    pub(crate) fn node_to_auto_code(&self, node: &AuraNode, indent: usize) -> String {
        let ind = "    ".repeat(indent);

        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
                ..
            } => {
                let mut result = String::new();

                // Plan 412 续: primary-prop 位置参数展示 —— text "X" (style: …) {}
                // 与源码简化写法 / parser 的 get_primary_prop 语法一致,让
                // preview-card 的 Auto tab 与页面源码风格统一。仅对字符串
                // 字面量且不含引号/换行的 text prop 生效,其余回退 prop 形式。
                const PRIMARY_TEXT_TAGS: &[&str] =
                    &["text", "h1", "h2", "h3", "p", "span", "label"];
                let primary_literal: Option<String> = if PRIMARY_TEXT_TAGS.contains(&tag.as_str()) {
                    match props.get("text") {
                        Some(AuraPropValue::Expr(auto_lang_core::ast::Expr::Str(s)))
                            if !s.contains('"') && !s.contains('\n') =>
                        {
                            Some(format!(" \"{}\"", s))
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let has_primary = primary_literal.is_some();

                // Build props string
                let mut props_parts = Vec::new();
                {
                    let mut sorted_pr: Vec<(&String, &AuraPropValue)> = props.iter().collect();
                    // Plan 015 P0#3: sorted iteration (props is a HashMap, jade gap 56)
                    sorted_pr.sort_by(|a, b| a.0.cmp(b.0));
                    for (key, value) in sorted_pr {
                        if has_primary && key == "text" {
                            continue; // hoisted as the positional primary prop above
                        }
                        let value_str = match value {
                            AuraPropValue::Expr(expr) => self.expr_to_auto_string(expr),
                            AuraPropValue::StyleBinding(bindings) => {
                                let binding_strs: Vec<String> = bindings
                                    .iter()
                                    .map(|b| {
                                        format!(
                                            "{}: {}",
                                            b.style_name,
                                            self.expr_to_auto_string(&b.condition)
                                        )
                                    })
                                    .collect();
                                format!("{{{}}}", binding_strs.join(", "))
                            }
                        };
                        props_parts.push(format!("{}: {}", key, value_str));
                    }
                } // plan-015 sorted-props block

                // Build events string
                let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
                sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
                for (event_name, event) in sorted_ev {
                    let _params_str = if event.params.is_empty() {
                        String::new()
                    } else {
                        format!("({})", event.params.join(", "))
                    };
                    // PLAN-528 W4: handler 内部约定已含前导点（parse_event_handler
                    // 返回 ".Name"），仅缺失时补点，避免打印出 `..Name`。
                    let handler_disp = if event.handler.starts_with('.') {
                        event.handler.clone()
                    } else {
                        format!(".{}", event.handler)
                    };
                    props_parts.push(format!("{}: {}", event_name, handler_disp));
                }

                let props_str = if props_parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", props_parts.join(", "))
                };

                // Handle self-closing vs with children
                if children.is_empty() {
                    result.push_str(&format!(
                        "{}{}{}{} {{}}\n",
                        ind,
                        tag,
                        primary_literal.unwrap_or_default(),
                        props_str
                    ));
                } else {
                    result.push_str(&format!(
                        "{}{}{}{} {{\n",
                        ind,
                        tag,
                        primary_literal.unwrap_or_default(),
                        props_str
                    ));
                    for child in children {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }

                result
            }

            AuraNode::Text(text_content) => {
                match text_content {
                    AuraTextContent::Literal(s) => {
                        format!("{}\"{}\"\n", ind, s)
                    }
                    AuraTextContent::Interpolated {
                        template,
                        bindings: _,
                    } => {
                        // Show the template with bindings
                        format!("{}\"{}\"\n", ind, template)
                    }
                }
            }

            AuraNode::Conditional {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let mut result = String::new();
                result.push_str(&format!("{}if {} {{\n", ind, condition));
                for child in then_body {
                    result.push_str(&self.node_to_auto_code(child, indent + 1));
                }
                result.push_str(&format!("{}}}\n", ind));
                if let Some(else_nodes) = else_body {
                    result.push_str(&format!("{}else {{\n", ind));
                    for child in else_nodes {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }
                result
            }

            AuraNode::ForLoop {
                var,
                index,
                iterable,
                body,
                ..
            } => {
                let mut result = String::new();
                let loop_header = if let Some(idx) = index {
                    format!("for ({}, {}) in {}", var, idx, iterable)
                } else {
                    format!("for {} in {}", var, iterable)
                };
                result.push_str(&format!("{}{} {{\n", ind, loop_header));
                for child in body {
                    result.push_str(&self.node_to_auto_code(child, indent + 1));
                }
                result.push_str(&format!("{}}}\n", ind));
                result
            }

            AuraNode::Component {
                name,
                props,
                events,
                ..
            } => {
                let mut result = String::new();

                let mut props_parts = Vec::new();
                // props here is Vec<(String, Expr)> — insertion order, no sort needed
                for (key, value) in props {
                    props_parts.push(format!("{}: {}", key, self.expr_to_auto_string(value)));
                }

                let mut sorted_ev: Vec<(&String, &AuraEvent)> = events.iter().collect();
                sorted_ev.sort_by(|a, b| a.0.cmp(b.0));
                for (event_name, event) in sorted_ev {
                    // PLAN-528 W4: 同 node_to_auto_code——handler 已含前导点时不再补。
                    let handler_disp = if event.handler.starts_with('.') {
                        event.handler.clone()
                    } else {
                        format!(".{}", event.handler)
                    };
                    props_parts.push(format!("{}: {}", event_name, handler_disp));
                }

                let props_str = if props_parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", props_parts.join(", "))
                };

                result.push_str(&format!("{}{}{} {{}}\n", ind, name, props_str));
                result
            }

            // Plan 105: Router outlet and link
            AuraNode::Outlet { .. } => {
                format!("{}outlet\n", ind)
            }

            // PLAN-046: memo block — emit children (boundary is VM-track only).
            AuraNode::MemoBlock { body, .. } => {
                let mut result = String::new();
                for c in body {
                    result.push_str(&self.node_to_auto_code(c, indent));
                }
                result
            }

            AuraNode::Link {
                to,
                text,
                href,
                children,
                ..
            } => {
                let mut result = String::new();
                // Generate appropriate link syntax based on which props are provided
                if !href.is_empty() {
                    // External link with href
                    if text.is_empty() {
                        result.push_str(&format!("{}link (href: \"{}\") {{\n", ind, href));
                        for child in children {
                            result.push_str(&self.node_to_auto_code(child, indent + 1));
                        }
                        result.push_str(&format!("{}}}\n", ind));
                    } else {
                        result.push_str(&format!(
                            "{}link (text: \"{}\", href: \"{}\") {{}}\n",
                            ind, text, href
                        ));
                    }
                } else if !text.is_empty() && children.is_empty() {
                    // Shorthand form with just text and to
                    result.push_str(&format!(
                        "{}link (to: \"{}\", text: \"{}\") {{}}\n",
                        ind, to, text
                    ));
                } else {
                    // Standard form with children
                    result.push_str(&format!("{}link (to: \"{}\") {{\n", ind, to));
                    for child in children {
                        result.push_str(&self.node_to_auto_code(child, indent + 1));
                    }
                    result.push_str(&format!("{}}}\n", ind));
                }
                result
            }
        }
    }

}

/// PLAN-007 T-05：design 对象 → Vue `:style` 绑定体。
/// `radius` → borderRadius；`background` → backgroundColor；
/// 槽引用（tokens.* 点链）→ `var(--<css-var>)`；字面量 → 具体值
///（radius 值 dp→px）。未知槽跳过（试点支持面）。
pub(crate) fn design_obj_to_vue(expr: &auto_parser_api::ast::Expr) -> Option<String> {
    use auto_parser_api::ast::{Expr, Key};
    let pairs = match expr {
        Expr::Object(pairs) => pairs,
        _ => return None,
    };
    let mut parts: Vec<String> = Vec::new();
    for pair in pairs {
        let key = match &pair.key {
            Key::NamedKey(n) => n.to_string(),
            Key::StrKey(s) => s.to_string(),
            _ => continue,
        };
        let css_prop = match key.as_str() {
            "radius" => "borderRadius",
            "background" => "backgroundColor",
            _ => continue,
        };
        let val_js: String = match &*pair.value {
            Expr::Str(s) => format!("'{}'", design_literal(css_prop, s)),
            Expr::Int(i) => format!("'{}px'", i),
            Expr::Float(v, _) | Expr::Double(v, _) => format!("'{}px'", v),
            Expr::Dot(_, _) | Expr::Ident(_) => {
                let Some(segs) = flatten_design_ref(&*pair.value) else { continue };
                let Some(var) = css_var_for_ref_path(&segs) else { continue };
                format!("'var(--{})'", var)
            }
            _ => continue,
        };
        parts.push(format!("{}: {}", css_prop, val_js));
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("{{ {} }}", parts.join(", ")))
    }
}

fn design_literal(css_prop: &str, s: &str) -> String {
    if css_prop == "borderRadius" {
        // dp/px 长度 → px（CSS 侧 1dp=1px 与 VM 轨同刻度）
        let n = s.trim_end_matches("dp").trim_end_matches("px").trim();
        if let Ok(v) = n.parse::<f64>() {
            return format!("{}px", v);
        }
    }
    s.to_string()
}

fn flatten_design_ref(expr: &auto_parser_api::ast::Expr) -> Option<Vec<String>> {
    use auto_parser_api::ast::Expr;
    match expr {
        Expr::Ident(n) => Some(vec![n.to_string()]),
        Expr::Dot(root, field) => {
            let mut segs = flatten_design_ref(root)?;
            segs.push(field.to_string());
            Some(segs)
        }
        _ => None,
    }
}

/// `tokens.colors.<31键>` → `<31键>`；`tokens.radius.<k>` → `radius-<k>`；
/// `tokens.colors.<ns>.<k>` → `<ns>-<k>`。非 tokens.* 路径 → None。
fn css_var_for_ref_path(segs: &[String]) -> Option<String> {
    if segs.first().map(|s| s.as_str()) != Some("tokens") {
        return None;
    }
    match segs.get(1).map(|s| s.as_str()) {
        Some("colors") => {
            let rest = &segs[2..];
            if rest.len() == 1 {
                Some(rest[0].clone())
            } else {
                Some(rest.join("-"))
            }
        }
        Some("radius") => {
            let rest = &segs[2..];
            if rest.len() == 1 {
                Some(format!("radius-{}", rest[0]))
            } else {
                None
            }
        }
        _ => None,
    }
}
