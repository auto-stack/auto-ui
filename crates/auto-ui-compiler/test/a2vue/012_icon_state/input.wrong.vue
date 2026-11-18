<!-- IconStateShowcase component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { Bell as LucideBell, Heart as LucideHeart, Search as LucideSearch, Zap as LucideZap } from 'lucide-vue-next'


const liked = ref<boolean>(false)


</script>

<template>
    <div class="flex flex-col gap-4">
      <LucideBell class="w-5 h-5 text-primary" :stroke-width="2.5" />
      <LucideBell class="w-5 h-5 text-muted-foreground" />
      <LucideBell class="text-red-500 w-5 h-5" :stroke-width="2.5" />
      <LucideHeart class="w-5 h-5" :class="liked ? 'text-primary' : 'text-muted-foreground'" :stroke-width="liked ? 2.5 : undefined" />
      <LucideZap class="text-primary" style="width:48px;height:48px" :stroke-width="2" />
      <LucideSearch class="w-5 h-5" />
    </div>

</template>

<style>
/* Component styles */

</style>
