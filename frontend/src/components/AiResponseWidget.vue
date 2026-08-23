<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import { computed, ref } from "vue";

const props = withDefaults(
  defineProps<{
    response: string;
    thinking?: string | null;
    provider?: string | null;
    model?: string | null;
    durationMs?: number | null;
    title?: string;
    responseLabel?: string;
    defaultThinkingOpen?: boolean;
    streaming?: boolean;
  }>(),
  {
    thinking: null,
    provider: null,
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

const renderedResponse = computed(() => {
  if (!props.response) return "";

  const html = marked.parse(props.response, {
    async: false,
    breaks: true,
    gfm: true,
  }) as string;

  return DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true },
    SANITIZE_NAMED_PROPS: true,
    ALLOW_DATA_ATTR: false,
    FORBID_ATTR: ["style"],
    FORBID_TAGS: [
      "audio",
      "button",
      "embed",
      "form",
      "iframe",
      "img",
      "input",
      "object",
      "option",
      "select",
      "source",
      "style",
      "textarea",
      "video",
    ],
  });
});

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
          v-if="props.provider"
          class="shrink-0 rounded-md bg-white/5 px-2 py-0.5 font-mono text-[10px] text-brand-text-muted"
        >
          {{ props.provider }}
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
        class="min-w-0 px-3 pb-4 text-xs leading-relaxed text-brand-text [overflow-wrap:anywhere] sm:px-4"
        :aria-busy="props.streaming"
        :aria-live="props.streaming ? 'polite' : 'off'"
      >
        <slot :response="props.response" :streaming="props.streaming">
          <!-- Provider-controlled Markdown must only reach v-html after sanitizing. -->
          <div v-if="props.response" class="ai-markdown" v-html="renderedResponse" />
        </slot>
        <span v-if="props.streaming" class="ml-0.5 inline-block h-3 w-1 animate-pulse bg-brand-primary align-middle" aria-hidden="true" />
      </div>

      <slot name="footer" />
    </section>
  </article>
</template>

<style scoped>
.ai-markdown {
  min-width: 0;
  overflow-wrap: anywhere;
}

.ai-markdown :deep(:first-child) {
  margin-top: 0;
}

.ai-markdown :deep(:last-child) {
  margin-bottom: 0;
}

.ai-markdown :deep(h1),
.ai-markdown :deep(h2),
.ai-markdown :deep(h3),
.ai-markdown :deep(h4) {
  margin: 1rem 0 0.45rem;
  color: var(--color-brand-text);
  font-weight: 800;
  line-height: 1.3;
}

.ai-markdown :deep(h1) {
  font-size: 1.15rem;
}

.ai-markdown :deep(h2) {
  font-size: 1rem;
}

.ai-markdown :deep(h3),
.ai-markdown :deep(h4) {
  font-size: 0.875rem;
}

.ai-markdown :deep(p),
.ai-markdown :deep(ul),
.ai-markdown :deep(ol),
.ai-markdown :deep(blockquote),
.ai-markdown :deep(pre),
.ai-markdown :deep(table) {
  margin: 0.55rem 0;
}

.ai-markdown :deep(ul),
.ai-markdown :deep(ol) {
  padding-left: 1.25rem;
}

.ai-markdown :deep(ul) {
  list-style: disc;
}

.ai-markdown :deep(ol) {
  list-style: decimal;
}

.ai-markdown :deep(li) {
  margin: 0.2rem 0;
}

.ai-markdown :deep(a) {
  color: var(--color-brand-primary);
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, var(--color-brand-primary) 45%, transparent);
  text-underline-offset: 0.18em;
}

.ai-markdown :deep(strong) {
  color: var(--color-brand-text);
  font-weight: 800;
}

.ai-markdown :deep(blockquote) {
  border-left: 3px solid color-mix(in srgb, var(--color-brand-primary) 45%, transparent);
  padding-left: 0.75rem;
  color: var(--color-brand-text-muted);
}

.ai-markdown :deep(code) {
  border-radius: 0.3rem;
  background: rgb(255 255 255 / 7%);
  padding: 0.1rem 0.3rem;
  color: #ddd6fe;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.9em;
}

.ai-markdown :deep(pre) {
  max-width: 100%;
  overflow-x: auto;
  border: 1px solid rgb(255 255 255 / 7%);
  border-radius: 0.65rem;
  background: rgb(0 0 0 / 28%);
  padding: 0.75rem;
}

.ai-markdown :deep(pre code) {
  background: transparent;
  padding: 0;
  color: var(--color-brand-text);
  font-size: 0.72rem;
  white-space: pre;
}

.ai-markdown :deep(hr) {
  margin: 0.9rem 0;
  border: 0;
  border-top: 1px solid rgb(255 255 255 / 8%);
}

.ai-markdown :deep(table) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  border-collapse: collapse;
}

.ai-markdown :deep(th),
.ai-markdown :deep(td) {
  border: 1px solid rgb(255 255 255 / 10%);
  padding: 0.4rem 0.55rem;
  text-align: left;
  white-space: nowrap;
}

.ai-markdown :deep(th) {
  background: rgb(255 255 255 / 5%);
  color: var(--color-brand-text);
}
</style>
