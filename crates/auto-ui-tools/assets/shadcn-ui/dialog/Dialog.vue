<script setup lang="ts">
import type { DialogRootEmits, DialogRootProps } from "reka-ui"
import { DialogRoot, useForwardPropsEmits } from "reka-ui"
import { computed, provide } from "vue"

const props = withDefaults(defineProps<DialogRootProps & { closeOnOutside?: boolean }>(), {
  closeOnOutside: true,
})
const emits = defineEmits<DialogRootEmits>()

provide("autoui:dialog-close-on-outside", computed(() => props.closeOnOutside))
const delegatedProps = computed(() => {
  const { closeOnOutside: _, ...delegated } = props
  return delegated
})
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DialogRoot v-bind="forwarded">
    <slot />
  </DialogRoot>
</template>
