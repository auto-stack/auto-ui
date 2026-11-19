<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { Check } from "lucide-vue-next"
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<CheckboxRootProps & { class?: HTMLAttributes["class"] }>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, "class")

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- PLAN-092 T-01 B-4：`checkbox { text: … }` 的文本槽此前落在
       CheckboxIndicator 内（顶替勾选图标），标签在页面上不可见。
       现将槽内容移出指示器，作为复选框旁的可见文本；外层用 div 而非
       label（label 会把点击转发给控件造成双派发，一次点击两条业务
       请求违背一次提交合同）。 -->
  <div class="flex items-center gap-2">
    <CheckboxRoot
      v-bind="forwarded"
      :class="
        cn('grid place-content-center peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground',
           props.class)"
    >
      <CheckboxIndicator class="grid place-content-center text-current">
        <Check class="h-4 w-4" />
      </CheckboxIndicator>
    </CheckboxRoot>
    <slot />
  </div>
</template>
