<script setup lang="ts">
import type { Todo } from "../../../api/models";

defineProps<{
  todo: Todo;
  filterTags?: string[];
}>();

defineEmits<{
  (e: "click", event: MouseEvent): void;
  (e: "delete"): void;
}>();

const ensureReadableColor = (hex: string): string => {
  let cleanHex = hex.replace("#", "");
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split("").map((char) => char + char).join("");
  }
  const num = parseInt(cleanHex, 16);
  const R = num >> 16;
  const G = (num >> 8) & 0x00ff;
  const B = num & 0x0000ff;
  const luminance = 0.299 * R + 0.587 * G + 0.114 * B;
  if (luminance < 140) {
    const percent = Math.round((140 - luminance) * 0.4) + 15;
    const amt = Math.round(2.55 * percent);
    const newR = Math.min(255, R + amt);
    const newG = Math.min(255, G + amt);
    const newB = Math.min(255, B + amt);
    return `#${(0x1000000 + newR * 0x10000 + newG * 0x100 + newB).toString(16).slice(1)}`;
  }
  return hex;
};

const tagStyle = (color?: string) => {
  const rawColor = color || "#9f75ff";
  const c = ensureReadableColor(rawColor);
  return {
    backgroundColor: `${c}22`,
    color: c,
    borderColor: `${c}44`,
  };
};
</script>

<template>
  <div
    @click="$emit('click', $event)"
    class="relative bg-brand-background/60 p-2.5 px-4 rounded-xl border transition-all cursor-pointer shadow-sm group/todo flex items-center gap-4"
    :class="todo.locked
      ? 'border-amber-500/20 hover:border-amber-500/40'
      : 'border-white/5 hover:border-brand-primary/20'"
  >
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <p class="text-sm font-bold text-brand-text group-hover/todo:text-brand-primary transition-colors leading-tight truncate">{{ todo.title }}</p>
        <svg v-if="todo.locked" xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-amber-400/70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      <p v-if="todo.details" class="text-[10px] text-brand-text-muted truncate font-medium opacity-60 mt-0.5 whitespace-nowrap overflow-hidden text-ellipsis">
        {{ todo.details.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim() }}
      </p>
      <div v-if="todo.tags?.length" class="flex flex-wrap gap-1 mt-1.5">
        <span
          v-for="tag in todo.tags"
          :key="tag.pid"
          class="px-1.5 py-px rounded-full text-[9px] font-bold uppercase tracking-wider border"
          :style="tagStyle(tag.color)"
        >
          {{ tag.title }}
        </span>
      </div>
      <div v-if="todo.subtasks?.length" class="flex items-center gap-1 mt-1.5">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-2.5 w-2.5 text-brand-primary/40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <span class="text-[9px] font-bold text-brand-primary/40 uppercase tracking-wider">
          {{ todo.subtasks.length }} subtask{{ todo.subtasks.length !== 1 ? 's' : '' }}
        </span>
      </div>
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
