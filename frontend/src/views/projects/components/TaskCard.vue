<script setup lang="ts">
import type { Todo } from "../../../api/models";

defineProps<{
  todo: Todo;
}>();

defineEmits<{
  (e: 'click', event: MouseEvent): void;
  (e: 'delete'): void;
}>();
</script>

<template>
  <div 
    @click="$emit('click', $event)" 
    class="relative bg-brand-background/60 p-2.5 px-4 rounded-xl border border-white/5 hover:border-brand-primary/20 transition-all cursor-pointer shadow-sm group/todo flex items-center gap-4"
  >
    <div class="flex-1 min-w-0">
      <p class="text-sm font-bold text-brand-text group-hover/todo:text-brand-primary transition-colors leading-tight truncate">{{ todo.title }}</p>
      <p v-if="todo.details" class="text-[10px] text-brand-text-muted truncate font-medium opacity-60 mt-0.5 whitespace-nowrap overflow-hidden text-ellipsis">
        {{ todo.details.replace(/[#*`]/g, '') }}
      </p>
    </div>

    <div class="flex items-center gap-3 shrink-0">
      <button 
        @click.stop="$emit('delete')" 
        class="p-1.5 rounded-lg bg-red-500/10 text-red-500 opacity-0 group-hover/todo:opacity-100 transition-all hover:bg-red-500 hover:text-white"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>
</template>
