<script setup lang="ts">
const { title } = defineProps<{
  title: string;
}>();
</script>
<template>
  <section class="bg-brand-container p-8 rounded-3xl shadow-xl border border-purple-900/20">
    <h3 class="text-xl font-bold text-brand-primary mb-6 border-b border-brand-primary/10 pb-4">{{ title }}</h3>
    <div class="bg-brand-background/30 p-6 rounded-2xl border border-brand-primary/5">
      <slot></slot>
    </div>
  </section>
</template>
