<script setup lang="ts">
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    response: string;
    thinking?: string | null;
    model?: string | null;
    durationMs?: number | null;
    title?: string;
    responseLabel?: string;
    defaultThinkingOpen?: boolean;
    streaming?: boolean;
  }>(),
  {
    thinking: null,
    model: null,
    durationMs: null,
    title: "AI-Antwort",
    responseLabel: "Antwort",
    defaultThinkingOpen: false,
    streaming: false,
  },
);

const thinkingOpen = ref(props.defaultThinkingOpen);
const copied = ref<"thinking" | "response" | null>(null);

const copy = async (kind: "thinking" | "response") => {
  const text = kind === "thinking" ? props.thinking : props.response;
  if (!text) return;

  await navigator.clipboard.writeText(text);
  copied.value = kind;
  setTimeout(() => {
    if (copied.value === kind) copied.value = null;
  }, 2000);
};
</script>

<template>
  <article class="min-w-0 max-w-full overflow-hidden rounded-xl border border-brand-primary/15 bg-white/5">
    <header class="flex flex-wrap items-center justify-between gap-x-3 gap-y-2 border-b border-white/5 px-3 py-2.5 sm:px-4 sm:py-3">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
        <span class="shrink-0 text-[10px] font-black uppercase tracking-wider text-brand-primary">
          {{ props.title }}
        </span>
        <span
          v-if="props.model"
          class="min-w-0 max-w-full truncate rounded-md bg-brand-primary/10 px-2 py-0.5 font-mono text-[10px] text-brand-primary"
          :title="props.model"
        >
          {{ props.model }}
        </span>
      </div>

      <div class="flex shrink-0 items-center gap-2 text-[10px] font-mono text-brand-text-muted/60">
        <span v-if="props.streaming" class="inline-flex items-center gap-1.5">
          <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-brand-primary" />
          Generiert
        </span>
        <span v-else-if="props.durationMs !== null">
          {{ (props.durationMs / 1000).toFixed(2) }}s
        </span>
      </div>
    </header>

    <section v-if="props.thinking" class="border-b border-white/5">
      <div class="flex min-h-11 items-center justify-between gap-2 bg-amber-400/5 px-3 sm:px-4">
        <button
          type="button"
          class="flex min-h-11 min-w-0 flex-1 items-center gap-2 text-left text-[10px] font-black uppercase tracking-wider text-amber-300/80 transition-colors hover:text-amber-300"
          :aria-expanded="thinkingOpen"
          @click="thinkingOpen = !thinkingOpen"
        >
          <svg
            class="h-3 w-3 shrink-0 transition-transform"
            :class="{ '-rotate-90': !thinkingOpen }"
            viewBox="0 0 20 20"
            fill="currentColor"
            aria-hidden="true"
          >
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
          <span class="truncate">Denkprozess</span>
        </button>
        <button
          type="button"
          class="min-h-11 shrink-0 px-1 text-[10px] text-amber-300/50 transition-colors hover:text-amber-300"
          @click="copy('thinking')"
        >
          {{ copied === 'thinking' ? 'Kopiert' : 'Kopieren' }}
        </button>
      </div>

      <div
        v-show="thinkingOpen"
        class="max-h-48 overflow-auto whitespace-pre-wrap break-words px-3 py-3 font-mono text-[11px] leading-relaxed text-brand-text-muted/70 [overflow-wrap:anywhere] sm:max-h-72 sm:px-4"
      >
        <slot name="thinking" :thinking="props.thinking">
          {{ props.thinking }}
        </slot>
      </div>
    </section>

    <section class="min-w-0">
      <div class="flex min-h-11 items-center justify-between gap-2 px-3 sm:px-4">
        <span class="min-w-0 truncate text-[10px] font-black uppercase tracking-wider text-brand-primary">
          {{ props.responseLabel }}
        </span>
        <button
          v-if="props.response"
          type="button"
          class="min-h-11 shrink-0 px-1 text-[10px] text-brand-primary/60 transition-colors hover:text-brand-primary"
          @click="copy('response')"
        >
          {{ copied === 'response' ? 'Kopiert' : 'Kopieren' }}
        </button>
      </div>

      <div
        class="min-w-0 whitespace-pre-wrap break-words px-3 pb-4 text-xs leading-relaxed text-brand-text [overflow-wrap:anywhere] sm:px-4"
        :aria-busy="props.streaming"
        :aria-live="props.streaming ? 'polite' : 'off'"
      >
        <slot :response="props.response" :streaming="props.streaming">
          {{ props.response }}
        </slot>
        <span v-if="props.streaming" class="ml-0.5 inline-block h-3 w-1 animate-pulse bg-brand-primary align-middle" aria-hidden="true" />
      </div>

      <slot name="footer" />
    </section>
  </article>
</template>
