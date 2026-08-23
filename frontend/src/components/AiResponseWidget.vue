<script setup lang="ts">
import { ref } from "vue";
import type { AiTestResult } from "../api/models";

const props = defineProps<{ result: AiTestResult }>();

const thinkingOpen = ref(true);
const copied = ref<"thinking" | "response" | null>(null);

const copy = async (kind: "thinking" | "response") => {
  const text = kind === "thinking" ? props.result.thinking : props.result.response;
  if (!text) return;

  await navigator.clipboard.writeText(text);
  copied.value = kind;
  setTimeout(() => {
    if (copied.value === kind) copied.value = null;
  }, 2000);
};
</script>

<template>
  <article class="rounded-xl bg-white/5 border border-brand-primary/15 overflow-hidden">
    <header class="flex items-center justify-between gap-3 px-4 py-3 border-b border-white/5">
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-[10px] font-black uppercase tracking-wider text-brand-primary">AI-Ergebnis</span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-brand-primary/10 text-brand-primary truncate">
          {{ props.result.model }}
        </span>
      </div>
      <span v-if="props.result.duration_ms" class="text-[10px] font-mono text-brand-text-muted/60 shrink-0">
        {{ (props.result.duration_ms / 1000).toFixed(2) }}s
      </span>
    </header>

    <section v-if="props.result.thinking" class="border-b border-white/5">
      <div class="flex items-center justify-between gap-2 px-4 py-2.5 bg-amber-400/5">
        <button
          type="button"
          class="flex items-center gap-2 text-[10px] font-black uppercase tracking-wider text-amber-300/80 hover:text-amber-300 transition-colors"
          @click="thinkingOpen = !thinkingOpen"
        >
          <svg class="w-3 h-3 transition-transform" :class="{ '-rotate-90': !thinkingOpen }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
          Denkprozess
        </button>
        <button
          type="button"
          class="text-[10px] text-amber-300/50 hover:text-amber-300 transition-colors"
          @click="copy('thinking')"
        >
          {{ copied === 'thinking' ? 'Kopiert' : 'Kopieren' }}
        </button>
      </div>
      <div
        v-show="thinkingOpen"
        class="max-h-72 overflow-y-auto px-4 py-3 text-[11px] leading-relaxed whitespace-pre-wrap text-brand-text-muted/70 select-text font-mono"
      >
        {{ props.result.thinking }}
      </div>
    </section>

    <section>
      <div class="flex items-center justify-between gap-2 px-4 py-2.5">
        <span class="text-[10px] font-black uppercase tracking-wider text-brand-primary">Antwort</span>
        <button
          type="button"
          class="text-[10px] text-brand-primary/60 hover:text-brand-primary transition-colors"
          @click="copy('response')"
        >
          {{ copied === 'response' ? 'Kopiert' : 'Kopieren' }}
        </button>
      </div>
      <div class="px-4 pb-4 text-xs leading-relaxed whitespace-pre-wrap text-brand-text select-text">
        {{ props.result.response }}
      </div>
    </section>
  </article>
</template>
