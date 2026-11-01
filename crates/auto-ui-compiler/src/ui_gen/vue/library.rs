//! PLAN-014 T-04：library 模式责任（LIBRARY_WIDGETS 表、covers_aura_tag、
//! library_template 嵌入模板）（自 ui_gen/vue.rs 原样收编；行为零改）。
use super::*;

/// All widget names with a library template (Plan 331). Kept in sync with
/// [`library_template`]; the CLI `auto ui list` reads this.
pub const LIBRARY_WIDGETS: &[&str] = &[
    "avatar",
    "badge",
    "button",
    "card",
    "chat_message",
    "checkbox",
    "dialog",
    "input",
    "label",
    "separator",
    "switch",
    "tabs",
    "textarea",
];

impl VueGenerator {
    /// Names of all widgets with a self-contained library template (Plan 331).
    pub const LIBRARY_WIDGETS: &'static [&'static str] = LIBRARY_WIDGETS;

    /// Plan 337: Is an AURA registry tag "covered" by the library?
    ///
    /// A tag `t` is covered ⟺ some library widget `w` satisfies
    /// `t == w` or `t.starts_with("{w}-")` (composite coverage — e.g. library
    /// `card` covers AURA `card-content`, `card-header`, …).
    pub fn covers_aura_tag(tag: &str) -> bool {
        Self::LIBRARY_WIDGETS
            .iter()
            .any(|w| tag == *w || tag.starts_with(&format!("{w}-")))
    }
}

/// Plan musk-022 Phase 1: derive a unique module-level connection-guard variable
/// name from an SSE endpoint path. `/api/chats/session/{id}/stream` →
/// `__streamConnected_api_chats_session_stream`. Slashes/braces collapse to `_`
/// so the result is a valid JS identifier. Per-path guards let multi-endpoint
/// stores (forge has 3 SSE streams) each open at most one connection.
pub(crate) fn stream_guard_var(path: &str) -> String {
    let suffix: String = path
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    format!("__streamConnected_{}", suffix)
}

/// Plan 051 M2 (②-b 预置字段模拟): emit SSE dispatch for stores using the
/// `__sse_*` preset-field pattern (ash-gui convention). For these stores the
/// RunOutput/RunResult handlers are parameter-less and read `__sse_*` refs that
/// the VM renderer pre-fills; in vue we codegen the same pattern: unpack `data`
/// into the matching `__sse_*` refs, then call the parameter-less handler.
///
/// Only the legacy-fallback actions (`RunOutput`/`RunResult`, the ash-gui
/// `command_output`/`command_result` contract) are handled here. Any other
/// action falls back to the default `(data)` parameterized call.
///
/// Field mapping source: `shell_store.at` RunOutput/RunResult handler comments
/// + the ShellEvent/CommandResult contract (`api.at`). `command_result` payloads
/// arrive nested under `data.CommandResult` when the server emits a
/// tagged-enum frame; `data.CommandResult ?? data` handles both shapes.
pub(crate) fn emit_preset_dispatch(
    code: &mut String,
    kw: &str,
    disc_field: &str,
    wire_value: &str,
    action: &str,
    follow_refresh_ctx: bool,
) {
    match action {
        "RunOutput" => {
            // command_output: { event, block_id, chunk } (flat)
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __sse_block_id.value = data.block_id; __sse_chunk.value = data.chunk; RunOutput(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "RunResult" => {
            // command_result: payload nested under data.CommandResult when the
            // server emits a tagged-enum frame (ash-server does). Unwrap both
            // shapes via `data.CommandResult ?? data`. Then fill the refs and
            // call the parameter-less handler. status may be a bare string
            // ("Success"/"Cancelled") or an object ({"Failed": msg}); the .at
            // handler treats it as a string and the vue side preserves whatever
            // shape arrived — GAP-B is handled in the .at handler body.
            //
            // Plan 052: __sse_output.value = r.output ?? {} 透传整个 RenderedOutput
            // 对象(可能是 Table/Record/Code/Error/Text/"Empty"),store 的 RunResult
            // 据此赋 b.output。保留 __sse_output_text(只取 .Text)供兼容/调试。
            //
            // Plan 065:RunResult 后跟随 RefreshContext()(store 声明了该
            // handler 时)—— 对齐 VM 轨引擎侧触发,拉 git 标签/历史/ai 槽。
            let follow = if follow_refresh_ctx {
                " RefreshContext();"
            } else {
                ""
            };
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ const r = data.CommandResult ?? data; __sse_block_id.value = r.block_id; __sse_cwd.value = r.cwd; __sse_status.value = r.status; __sse_output.value = r.output ?? {{}}; __sse_output_text.value = (r.output && r.output.Text) ? r.output.Text : ''; __sse_duration_ms.value = r.duration_ms; __sse_exit_code.value = r.exit_code; RunResult();{follow} }}\n",
                kw = kw, disc = disc_field, wire = wire_value, follow = follow,
            ));
        }
        "JobStarted" => {
            // Plan 055 Phase A.3: job_started {event, job_id, block_id, cmd}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __sse_job_id.value = data.job_id; __sse_job_cmd.value = data.cmd; JobStarted(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "JobDone" => {
            // Plan 055 Phase A.3: job_done {event, job_id, exit_code, cmd}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __sse_job_id.value = data.job_id; __sse_job_exit.value = data.exit_code; JobDone(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "AiTurn" => {
            // Plan 063 T4: ai_turn {event, block_id, turn, question}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __ai_block_id.value = data.block_id; __ai_turn.value = data.turn; __ai_question.value = data.question; AiTurn(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "AiChunk" => {
            // Plan 063 T4: ai_chunk {event, block_id, turn, text}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __ai_block_id.value = data.block_id; __ai_turn.value = data.turn; __ai_text.value = data.text; AiChunk(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "AiToolCall" => {
            // Plan 063 T4: ai_tool_call {event, block_id, turn, tool, args}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __ai_block_id.value = data.block_id; __ai_turn.value = data.turn; __ai_tool.value = data.tool; __ai_args.value = data.args; AiToolCall(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "AiToolResult" => {
            // Plan 063 T4: ai_tool_result {event, block_id, turn, tool, result}(flat)。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ __ai_block_id.value = data.block_id; __ai_turn.value = data.turn; __ai_tool.value = data.tool; __ai_result.value = data.result; AiToolResult(); }}\n",
                kw = kw, disc = disc_field, wire = wire_value,
            ));
        }
        "ChatCleared" => {
            // Plan 063 T4: chat_cleared {event, block_id} —— 无载荷,纯清空信号。
            code.push_str(&format!(
                "                {kw} (data.{disc} === '{wire}') {{ ChatCleared(); }}\n",
                kw = kw,
                disc = disc_field,
                wire = wire_value,
            ));
        }
        _ => {
            // Non-legacy action in a preset-field store: fall back to
            // parameterized dispatch (preserves prior behavior).
            code.push_str(&format!(
                "                {} (data.{} === '{}') {}(data);\n",
                kw, disc_field, wire_value, action
            ));
        }
    }
}

/// The `cn` class-merge helper emitted at the registry root (`registry/utils.ts`)
/// and imported by every library widget as `../utils`. Plan 331.
pub(crate) const LIBRARY_UTILS_TS: &str = r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#;

/// The attribution comment prepended to every generated library SFC (Plan 331).
pub(crate) fn attribution_header(name: &str) -> String {
    format!(
        "<!-- Generated by AutoUI from widgets/{name}.at.\n\
         \x20    Visual layer derived from shadcn-vue (MIT). See NOTICES. -->"
    )
}

/// Look up the library template for a primitive widget name.
///
/// Phase 1.4: button / input / label. Remaining v1 widgets land in Phase 5.
pub(crate) fn library_template(name: &str) -> Option<WidgetTemplate> {
    match name {
        "chat_message" => Some(WidgetTemplate {
            script: r#"
const props = defineProps<{
  role?: string
  content?: string
  timestamp?: number
  thinking?: string
  profession_id?: string
  streaming?: boolean
}>()

const roleLabel = computed(() => {
  if (props.role === 'user') return '🧑 You'
  if (props.profession_id) return '🤖 ' + props.profession_id
  return '🤖 AI'
})

const timeLabel = computed(() => {
  if (!props.timestamp) return ''
  const d = new Date(props.timestamp * 1000)
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
})

const isUser = computed(() => props.role === 'user')"#,
            template: r#"  <div :class="['flex flex-col gap-0.5 msg-row', isUser ? 'msg-row-user' : 'msg-row-ai']">
    <div :class="['flex items-center gap-2 px-1', isUser ? 'justify-end' : '']">
      <span :class="['text-xs font-semibold', isUser ? 'text-primary' : 'text-muted-foreground']">{{ roleLabel }}</span>
      <span v-if="timeLabel" class="text-xs text-muted-foreground">{{ timeLabel }}</span>
    </div>
    <div :class="[
      'rounded-xl px-3.5 py-2.5 text-sm leading-relaxed break-words',
      isUser
        ? 'bg-primary text-primary-foreground rounded-br-sm self-end max-w-[85%]'
        : 'bg-card border border-border text-foreground rounded-bl-sm self-start max-w-full'
    ]">
      <slot>{{ content }}</slot>
    </div>
    <div v-if="thinking" class="px-1 text-xs text-muted-foreground italic opacity-70">
      💭 {{ thinking }}
    </div>
  </div>"#,
            extra_support_files: vec![],
        }),
        "button" => Some(WidgetTemplate {
            script: r#"import { Primitive } from 'reka-ui'
import { cn } from '../utils'
import { buttonVariants } from './variants'
import type { ButtonVariants } from './variants'

const props = withDefaults(defineProps<{
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: string
  as?: string
  asChild?: boolean
}>(), { variant: 'default', size: 'default', as: 'button' })"#,
            template: r#"  <Primitive :as="as" :as-child="asChild" :class="cn(buttonVariants({ variant, size }), props.class)">
    <slot />
  </Primitive>"#,
            extra_support_files: vec![(
                "variants.ts",
                r#"import { cva, type VariantProps } from 'class-variance-authority'

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        // PLAN-571: default = UA 预填等价基线（中性填充+发丝描边），与 Rust 侧
        // ui/style/variants.rs button_variant_preset("default") 互锁——改任一须同步
        // （ui_gen::vue::tests plan571 互锁测试锚定）。base 已含 rounded-md/font-medium，
        // variant 值只写差量类。
        default: 'bg-muted border border-border text-foreground hover:bg-muted/70',
        primary: 'bg-primary text-primary-foreground hover:bg-primary/90',
        submit: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
        // PLAN-671 ⑥：`text` variant 值域吸收——vm 侧 ui/style/variants.rs
        // button_variant_preset("text") == ""（chromeless，用户 class 主导）。
        // 此前联合缺该值，vm 轨合法的 variant: "text" 在 vue-tsc 面落
        // TS2322（封闭联合外）。空差量类 = 与 vm 侧零附加样式互锁等价
        //（双轨视觉零回退；改任一侧须同步）。
        text: '',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
        icon: 'h-10 w-10',
      },
    },
    // PLAN-080: 缺省无预设（preflight 等价）——无 variant/size prop 的裸
    // Button 不落任何 variant/size 类，与 Rust variants.rs 缺省 chromeless
    // 互锁；显式 variant='default'/size='default' 才得 PLAN-571 基线。
  },
)

export type ButtonVariants = VariantProps<typeof buttonVariants>
"#,
            )],
        }),
        "input" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()
const emits = defineEmits<{ 'update:modelValue': [value: string | number] }>()"#,
            template: r#"  <input
    :value="modelValue ?? defaultValue"
    @input="emits('update:modelValue', ($event.target as HTMLInputElement).value)"
    :class="cn('flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "label" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { Label, type LabelProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<LabelProps & { class?: HTMLAttributes['class'] }>()"#,
            template: r#"  <Label
    :for="props.for"
    :class="cn('text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70', props.class)"
  >
    <slot />
  </Label>"#,
            extra_support_files: vec![],
        }),
        "textarea" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()
const emits = defineEmits<{ 'update:modelValue': [value: string | number] }>()"#,
            template: r#"  <textarea
    :value="modelValue ?? defaultValue"
    @input="emits('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    :class="cn('flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "checkbox" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  CheckboxRoot,
  CheckboxIndicator,
  type CheckboxRootEmits,
  type CheckboxRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<CheckboxRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <CheckboxRoot
    v-bind="forwarded"
    :class="cn('peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground', props.class)"
  >
    <CheckboxIndicator class="flex h-full w-full items-center justify-center text-current">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" class="h-3.5 w-3.5"><polyline points="20 6 9 17 4 12" /></svg>
    </CheckboxIndicator>
  </CheckboxRoot>"#,
            extra_support_files: vec![],
        }),
        "switch" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  SwitchRoot,
  SwitchThumb,
  type SwitchRootEmits,
  type SwitchRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<SwitchRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <SwitchRoot
    v-bind="forwarded"
    :class="cn('peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input', props.class)"
  >
    <SwitchThumb class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0" />
  </SwitchRoot>"#,
            extra_support_files: vec![],
        }),
        "card" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()"#,
            template: r#"  <div :class="cn('rounded-lg border bg-card text-card-foreground shadow-sm', props.class)">
    <slot />
  </div>"#,
            extra_support_files: vec![
                ("CardHeader.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col space-y-1.5 p-6', props.class)\"><slot /></div>\n</template>\n"),
                ("CardTitle.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <h3 :class=\"cn('text-2xl font-semibold leading-none tracking-tight', props.class)\"><slot /></h3>\n</template>\n"),
                ("CardDescription.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <p :class=\"cn('text-sm text-muted-foreground', props.class)\"><slot /></p>\n</template>\n"),
                ("CardContent.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('p-6 pt-0', props.class)\"><slot /></div>\n</template>\n"),
                ("CardFooter.vue", "<!-- Generated by AutoUI from widgets/card.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex items-center p-6 pt-0', props.class)\"><slot /></div>\n</template>\n"),
            ],
        }),
        "separator" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { Separator, type SeparatorProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = withDefaults(
  defineProps<SeparatorProps & { class?: HTMLAttributes['class'] }>(),
  { orientation: 'horizontal', decorative: true },
)

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardProps(delegatedProps)"#,
            template: r#"  <Separator
    v-bind="forwarded"
    :class="cn('shrink-0 bg-border', props.orientation === 'vertical' ? 'h-full w-[1px]' : 'h-[1px] w-full', props.class)"
  />"#,
            extra_support_files: vec![],
        }),
        "badge" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { cn } from '../utils'
import { badgeVariants, type BadgeVariants } from './variants'

const props = defineProps<{
  variant?: BadgeVariants['variant']
  class?: HTMLAttributes['class']
}>()"#,
            template: r#"  <div :class="cn(badgeVariants({ variant: props.variant }), props.class)">
    <slot />
  </div>"#,
            extra_support_files: vec![(
                "variants.ts",
                r#"import { cva, type VariantProps } from 'class-variance-authority'

export const badgeVariants = cva(
  'inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground hover:bg-primary/80',
        secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive: 'border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80',
        outline: 'text-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export type BadgeVariants = VariantProps<typeof badgeVariants>
"#,
            )],
        }),
        "avatar" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import { AvatarRoot, type AvatarRootProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<AvatarRootProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardProps(delegatedProps)"#,
            template: r#"  <AvatarRoot v-bind="forwarded" :class="cn('relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full', props.class)">
    <slot />
  </AvatarRoot>"#,
            extra_support_files: vec![
                ("AvatarImage.vue", "<!-- Generated by AutoUI from widgets/avatar.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { AvatarImage, type AvatarImageProps } from 'reka-ui'\n\nconst props = defineProps<AvatarImageProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <AvatarImage v-bind=\"props\" class=\"aspect-square h-full w-full\" />\n</template>\n"),
                ("AvatarFallback.vue", "<!-- Generated by AutoUI from widgets/avatar.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { AvatarFallback, type AvatarFallbackProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<AvatarFallbackProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <AvatarFallback v-bind=\"props\" :class=\"cn('flex h-full w-full items-center justify-center rounded-full bg-muted', props.class)\"><slot /></AvatarFallback>\n</template>\n"),
            ],
        }),
        "dialog" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  DialogRoot,
  DialogTrigger,
  type DialogRootEmits,
  type DialogRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<DialogRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DialogRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)"#,
            template: r#"  <DialogRoot v-bind="forwarded">
    <DialogTrigger v-if="$slots.trigger" as-child><slot name="trigger" /></DialogTrigger>
    <slot />
  </DialogRoot>"#,
            extra_support_files: vec![
                ("DialogContent.vue", r#"<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  type DialogContentEmits,
  type DialogContentProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<DialogContentProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DialogContentEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DialogPortal>
    <DialogOverlay class="fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
    <DialogContent
      v-bind="forwarded"
      :class="cn('fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 sm:rounded-lg', props.class)"
    >
      <slot />
      <DialogTitle v-if="$slots.title" as-child><slot name="title" /></DialogTitle>
      <DialogDescription v-if="$slots.description" as-child><slot name="description" /></DialogDescription>
      <DialogClose class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
        <span class="sr-only">Close</span>
      </DialogClose>
    </DialogContent>
  </DialogPortal>
</template>
"#),
                ("DialogHeader.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col space-y-1.5 text-center sm:text-left', props.class)\"><slot /></div>\n</template>\n"),
                ("DialogFooter.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { cn } from '../utils'\n\nconst props = defineProps<{ class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <div :class=\"cn('flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2', props.class)\"><slot /></div>\n</template>\n"),
                // Plan 435 P6-2(D1):补齐 schema vue 行声明的家族导出面 ——
                // dialogtrigger/dialogclose/dialogtitle/dialogdescription 的
                // import 路径同为 @/components/ui/dialog,官方包必须真导出
                // 这四个名(与 shadcn-vue 分发面一致;独立于 slot 组合形态)。
                ("DialogTrigger.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { DialogTrigger, type DialogTriggerProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<DialogTriggerProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <DialogTrigger v-bind=\"props\" :class=\"cn(props.class)\"><slot /></DialogTrigger>\n</template>\n"),
                ("DialogClose.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { DialogClose, type DialogCloseProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<DialogCloseProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <DialogClose v-bind=\"props\" :class=\"cn(props.class)\"><slot /></DialogClose>\n</template>\n"),
                ("DialogTitle.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { DialogTitle, type DialogTitleProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<DialogTitleProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <DialogTitle v-bind=\"props\" :class=\"cn('text-lg font-semibold leading-none tracking-tight', props.class)\"><slot /></DialogTitle>\n</template>\n"),
                ("DialogDescription.vue", "<!-- Generated by AutoUI from widgets/dialog.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->\n<script setup lang=\"ts\">\nimport type { HTMLAttributes } from 'vue'\nimport { DialogDescription, type DialogDescriptionProps } from 'reka-ui'\nimport { cn } from '../utils'\n\nconst props = defineProps<DialogDescriptionProps & { class?: HTMLAttributes['class'] }>()\n</script>\n\n<template>\n  <DialogDescription v-bind=\"props\" :class=\"cn('text-sm text-muted-foreground', props.class)\"><slot /></DialogDescription>\n</template>\n"),
            ],
        }),
        "tabs" => Some(WidgetTemplate {
            script: r#"import type { HTMLAttributes } from 'vue'
import { computed, provide } from 'vue'
import {
  TabsRoot,
  type TabsRootEmits,
  type TabsRootProps,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsRootProps & { class?: HTMLAttributes['class'], variant?: 'default' | 'enclosed' }>()
const emits = defineEmits<TabsRootEmits>()

const delegatedProps = computed(() => {
  const { class: _, variant: __, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)

// PLAN-641：形态向 List/Trigger/Content 子件下发（provide/inject——跨 slot
// prop drilling 不可行）。enclosed = 激活 tab 与内容面板连通，非激活 tab 扁平。
const variant = computed(() => props.variant ?? 'default')
provide('autoTabsVariant', variant)"#,
            template: r#"    <TabsRoot v-bind="forwarded" :class="cn('relative', props.class)">
      <slot />
    </TabsRoot>"#,
            extra_support_files: vec![
                ("TabsList.vue", r#"<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsList, type TabsListProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props
  return delegated
})
</script>

<template>
  <TabsList
    v-bind="delegatedProps"
    :class="
      cn(
        variant === 'enclosed'
          ? 'inline-flex h-9 w-full items-stretch justify-start bg-muted text-muted-foreground'
          : 'inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground',
        props.class,
      )
    "
  >
    <slot />
  </TabsList>
</template>
"#),
                ("TabsTrigger.vue", r#"<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsTrigger, type TabsTriggerProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsTriggerProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => { const { class: _, ...d } = props; return d })
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <TabsTrigger
    v-bind="forwarded"
    :class="
      cn(
        variant === 'enclosed'
          ? 'inline-flex items-center justify-center whitespace-nowrap px-4 text-sm font-medium transition-all border border-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 data-[state=active]:border-b-0 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=inactive]:bg-secondary/40 data-[state=inactive]:text-muted-foreground'
          : 'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm',
        props.class,
      )
    "
  >
    <span class="truncate">
      <slot />
    </span>
  </TabsTrigger>
</template>
"#),
                ("TabsContent.vue", r#"<!-- Generated by AutoUI from widgets/tabs.at. Visual layer derived from shadcn-vue (MIT). See NOTICES. -->
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { TabsContent, type TabsContentProps, useForwardProps } from 'reka-ui'
import { cn } from '../utils'

const props = defineProps<TabsContentProps & { class?: HTMLAttributes['class'] }>()

// PLAN-641：形态由 Tabs 根 provide（缺省 default 按钮托盘）。
const variant = inject<import('vue').ComputedRef<'default' | 'enclosed'>>(
  'autoTabsVariant',
  computed(() => 'default'),
)

const delegatedProps = computed(() => { const { class: _, ...d } = props; return d })
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <TabsContent
    v-bind="forwarded"
    :class="
      cn(
        variant === 'enclosed'
          ? 'border border-t-0 border-border bg-background ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2'
          : 'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        props.class,
      )
    "
  >
    <slot />
  </TabsContent>
</template>
"#),
            ],
        }),
        _ => None,
    }
}

// ============================================================================
// Tests
