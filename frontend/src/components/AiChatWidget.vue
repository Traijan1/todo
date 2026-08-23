<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import {
  type ChatResult,
  type ChatToolRun,
  type ChatTransportMessage,
  streamChat,
} from "../api/chat";
import { useAiContextStore } from "../stores/aiContext";
import { useBoardStore } from "../stores/boards";
import { useProjectStore } from "../stores/projects";
import { useTagStore } from "../stores/tags";
import AiResponseWidget from "./AiResponseWidget.vue";

interface UiMessage {
  id: number;
  role: "user" | "assistant" | "error";
  content: string;
  thinking?: string;
  provider?: string;
  model?: string;
  durationMs?: number;
  tools?: ChatToolRun[];
}

const aiContext = useAiContextStore();
const boardStore = useBoardStore();
const projectStore = useProjectStore();
const tagStore = useTagStore();
const isOpen = ref(false);
const input = ref("");
const messages = ref<UiMessage[]>([]);
const loading = ref(false);
const status = ref("Denke nach …");
const chatPanel = ref<HTMLElement | null>(null);
const scrollArea = ref<HTMLElement | null>(null);
const inputArea = ref<HTMLTextAreaElement | null>(null);
let nextMessageId = 1;
let activeRequest: AbortController | null = null;

const MUTATING_TOOLS = new Set([
  "add_project",
  "update_project",
  "delete_project",
  "add_board",
  "update_board",
  "delete_board",
  "reorder_boards",
  "add_todo",
  "update_todo",
  "delete_todo",
  "reorder_todos",
  "create_tag",
  "update_tag",
  "delete_tag",
  "add_tag_to_todo",
  "remove_tag_from_todo",
  "add_comment",
  "start_timer",
  "stop_timer",
]);

const shortId = (value?: string) => (value ? value.slice(0, 8) : "");
const contextItems = computed(() =>
  [
    aiContext.projectId
      ? { label: "Projekt", value: shortId(aiContext.projectId) }
      : null,
    aiContext.boardId
      ? { label: "Board", value: shortId(aiContext.boardId) }
      : null,
    aiContext.todoId
      ? { label: "Todo", value: shortId(aiContext.todoId) }
      : null,
  ].filter((item): item is { label: string; value: string } => Boolean(item)),
);

const suggestions = computed(() => {
  if (aiContext.todoId) {
    return [
      "Fasse das ausgewählte Todo zusammen.",
      "Welche offenen Punkte erkennst du beim ausgewählten Todo?",
    ];
  }
  if (aiContext.boardId) {
    return [
      "Zeige mir die Todos im ausgewählten Board.",
      "Welche Aufgabe sollte ich in diesem Board als Nächstes bearbeiten?",
    ];
  }
  if (aiContext.projectId) {
    return [
      "Gib mir einen Überblick über dieses Projekt.",
      "Welche Todos sind in diesem Projekt noch offen?",
    ];
  }
  return [
    "Zeige mir meine Projekte.",
    "Wobei kannst du mir in der Todo-App helfen?",
  ];
});

const scrollToBottom = async () => {
  await nextTick();
  if (scrollArea.value)
    scrollArea.value.scrollTop = scrollArea.value.scrollHeight;
};

watch([messages, status], scrollToBottom, { deep: true });

const open = async () => {
  isOpen.value = true;
  await nextTick();
  inputArea.value?.focus();
  scrollToBottom();
};

const reset = () => {
  if (
    loading.value &&
    !confirm("Die laufende Antwort abbrechen und einen neuen Chat starten?")
  ) {
    return;
  }
  activeRequest?.abort();
  activeRequest = null;
  loading.value = false;
  messages.value = [];
  input.value = "";
  status.value = "Denke nach …";
};

const refreshAfterMutations = async (result: ChatResult) => {
  if (
    !result.tools.some((tool) => tool.success && MUTATING_TOOLS.has(tool.name))
  ) {
    return;
  }
  await projectStore.fetchProjects();
  if (
    result.context.project_pid &&
    !result.tools.some((tool) => tool.success && tool.name === "delete_project")
  ) {
    await Promise.all([
      boardStore.fetchBoards(result.context.project_pid),
      tagStore.fetchTags(result.context.project_pid),
    ]);
  }
};

const applyResult = (result: ChatResult) => {
  messages.value.push({
    id: nextMessageId++,
    role: "assistant",
    content: result.response || "Das Modell hat keine Textantwort geliefert.",
    thinking: result.thinking,
    provider: result.provider_id,
    model: result.model,
    durationMs: result.duration_ms,
    tools: result.tools,
  });
  refreshAfterMutations(result).catch(() => {});
};

const send = async () => {
  const content = input.value.trim();
  if (!content || loading.value) return;

  messages.value.push({ id: nextMessageId++, role: "user", content });
  input.value = "";
  loading.value = true;
  status.value = "Denke nach …";
  activeRequest = new AbortController();

  const history = messages.value
    .filter(
      (message): message is UiMessage & { role: "user" | "assistant" } =>
        message.role === "user" || message.role === "assistant",
    )
    .slice(-30)
    .map<ChatTransportMessage>((message) => ({
      role: message.role,
      content: message.content,
    }));

  try {
    await streamChat(
      history,
      { ...aiContext.requestContext },
      (event) => {
        if (event.type === "progress") {
          status.value =
            event.progress.kind === "running_tool"
              ? `Führe ${event.progress.name} aus …`
              : "Denke nach …";
        } else if (event.type === "done") {
          applyResult(event.result);
        } else {
          throw new Error(event.message);
        }
      },
      activeRequest.signal,
    );
  } catch (error) {
    if ((error as Error).name !== "AbortError") {
      messages.value.push({
        id: nextMessageId++,
        role: "error",
        content:
          (error as Error).message || "Die Chat-Anfrage ist fehlgeschlagen.",
      });
    }
  } finally {
    loading.value = false;
    activeRequest = null;
    status.value = "Denke nach …";
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    send();
  }
};

const closeOnOutsidePointer = (event: PointerEvent) => {
  if (
    isOpen.value &&
    chatPanel.value &&
    event.target instanceof Node &&
    !chatPanel.value.contains(event.target)
  ) {
    isOpen.value = false;
  }
};

onMounted(() =>
  document.addEventListener("pointerdown", closeOnOutsidePointer, true),
);
onUnmounted(() => {
  document.removeEventListener("pointerdown", closeOnOutsidePointer, true);
  activeRequest?.abort();
});
</script>

<template>
  <Transition name="chat-panel">
    <section
      v-if="isOpen"
      ref="chatPanel"
      class="fixed inset-x-2 bottom-[calc(5rem+env(safe-area-inset-bottom))] top-[calc(4rem+env(safe-area-inset-top))] z-[120] flex min-h-0 flex-col overflow-hidden rounded-2xl border border-brand-primary/20 bg-brand-container shadow-2xl shadow-black/60 sm:inset-auto sm:bottom-24 sm:left-5 sm:h-[min(680px,calc(100dvh-8rem))] sm:w-[min(430px,calc(100vw-2.5rem))] lg:left-[calc(16rem+1.75rem)]"
      aria-label="AI Chat"
    >
      <header class="shrink-0 border-b border-white/5 px-3 py-3 sm:px-4">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-brand-primary/15 text-brand-primary">
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-3 3v-3z" />
            </svg>
          </div>
          <div class="min-w-0 flex-1">
            <h2 class="truncate text-sm font-black text-brand-text">Todo Assistent</h2>
            <p class="truncate text-[10px] text-brand-text-muted/60">
              Liest und bearbeitet über gemeinsame Tools
            </p>
          </div>
          <button
            type="button"
            class="rounded-lg p-2 text-brand-text-muted/50 transition-colors hover:bg-white/5 hover:text-brand-primary"
            title="Neuer Chat"
            @click="reset"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" />
            </svg>
          </button>
          <button
            type="button"
            class="rounded-lg p-2 text-brand-text-muted/50 transition-colors hover:bg-white/5 hover:text-brand-primary"
            title="Chat schließen"
            @click="isOpen = false"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="mt-2 flex min-w-0 flex-wrap gap-1.5">
          <span
            v-if="!contextItems.length"
            class="rounded-md bg-white/5 px-2 py-1 text-[9px] font-bold uppercase tracking-wider text-brand-text-muted/50"
          >
            Kein Objekt ausgewählt
          </span>
          <span
            v-for="item in contextItems"
            :key="item.label"
            class="max-w-full truncate rounded-md bg-brand-primary/10 px-2 py-1 font-mono text-[9px] text-brand-primary/80"
          >
            {{ item.label }} · {{ item.value }}
          </span>
        </div>
      </header>

      <div ref="scrollArea" class="min-h-0 flex-1 space-y-3 overflow-y-auto px-3 py-4 sm:px-4">
        <div v-if="!messages.length" class="flex min-h-full flex-col items-center justify-center px-3 text-center">
          <div class="mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-primary/10 text-brand-primary">
            <svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M9.75 3a.75.75 0 01.75.75V5h3V3.75a.75.75 0 011.5 0V5h.25A3.75 3.75 0 0119 8.75v5.5A3.75 3.75 0 0115.25 18h-6.5A3.75 3.75 0 015 14.25v-5.5A3.75 3.75 0 018.75 5H9V3.75A.75.75 0 019.75 3zM8.5 10.5h.01m6.99 0h.01M9 14h6" />
            </svg>
          </div>
          <h3 class="text-sm font-bold text-brand-text">Was soll ich erledigen?</h3>
          <p class="mt-1 max-w-xs text-xs leading-relaxed text-brand-text-muted/60">
            Ich kann Projekte, Boards und Todos lesen oder auf deinen ausdrücklichen Wunsch ändern.
          </p>
          <div class="mt-4 grid w-full gap-2">
            <button
              v-for="suggestion in suggestions"
              :key="suggestion"
              type="button"
              class="rounded-xl border border-white/5 bg-white/[0.025] px-3 py-2.5 text-left text-xs text-brand-text-muted transition-colors hover:border-brand-primary/20 hover:bg-brand-primary/5 hover:text-brand-text"
              @click="input = suggestion; inputArea?.focus()"
            >
              {{ suggestion }}
            </button>
          </div>
        </div>

        <template v-for="message in messages" :key="message.id">
          <div v-if="message.role === 'user'" class="flex justify-end pl-10">
            <div class="max-w-full whitespace-pre-wrap break-words rounded-2xl rounded-br-md bg-brand-primary px-3.5 py-2.5 text-xs leading-relaxed text-brand-container [overflow-wrap:anywhere]">
              {{ message.content }}
            </div>
          </div>
          <AiResponseWidget
            v-else-if="message.role === 'assistant'"
            :response="message.content"
            :thinking="message.thinking"
            :provider="message.provider"
            :model="message.model"
            :duration-ms="message.durationMs"
            title="Assistent"
            response-label="Antwort"
          >
            <template v-if="message.tools?.length" #footer>
              <div class="flex flex-wrap gap-1.5 border-t border-white/5 px-3 py-2.5 sm:px-4">
                <span
                  v-for="(tool, index) in message.tools"
                  :key="`${tool.name}-${index}`"
                  class="rounded-md px-2 py-1 font-mono text-[9px]"
                  :class="tool.success ? 'bg-emerald-400/10 text-emerald-300/70' : 'bg-red-400/10 text-red-300/70'"
                >
                  {{ tool.name }}
                </span>
              </div>
            </template>
          </AiResponseWidget>
          <div v-else class="rounded-xl border border-red-400/15 bg-red-400/5 px-3 py-2.5 text-xs leading-relaxed text-red-300/80">
            {{ message.content }}
          </div>
        </template>

        <div v-if="loading" class="rounded-xl border border-brand-primary/10 bg-white/[0.025] px-3 py-3" aria-live="polite">
          <div class="flex items-center gap-2 text-xs text-brand-text-muted">
            <span class="h-2 w-2 animate-pulse rounded-full bg-brand-primary" />
            <span class="min-w-0 truncate">{{ status }}</span>
          </div>
        </div>
      </div>

      <footer class="shrink-0 border-t border-white/5 bg-brand-container px-3 py-3 sm:px-4">
        <div class="flex items-end gap-2 rounded-xl border border-white/5 bg-brand-background/50 p-1.5 focus-within:border-brand-primary/30 focus-within:ring-4 focus-within:ring-brand-primary/5">
          <textarea
            ref="inputArea"
            v-model="input"
            rows="1"
            maxlength="12000"
            class="max-h-32 min-h-10 min-w-0 flex-1 resize-none overflow-y-auto bg-transparent px-2 py-2 text-sm text-brand-text outline-none placeholder:text-brand-text-muted/30"
            placeholder="Nachricht oder Aufgabe …"
            :disabled="loading"
            @keydown="handleKeydown"
          />
          <button
            type="button"
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-brand-primary text-brand-container transition-all hover:bg-brand-primary/90 disabled:cursor-not-allowed disabled:opacity-30"
            :disabled="loading || !input.trim()"
            title="Senden"
            @click="send"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M5 12h14m-6-6l6 6-6 6" />
            </svg>
          </button>
        </div>
        <p class="mt-1.5 px-1 text-[9px] text-brand-text-muted/35">Enter sendet · Shift+Enter fügt eine Zeile ein</p>
      </footer>
    </section>
  </Transition>

  <button
    v-if="!isOpen"
    type="button"
    class="fixed bottom-[calc(1rem+env(safe-area-inset-bottom))] left-4 z-[120] flex h-13 w-13 items-center justify-center rounded-2xl bg-brand-primary text-brand-container shadow-xl shadow-brand-primary/20 transition-transform hover:scale-105 active:scale-95 sm:bottom-5 sm:left-5 lg:left-[calc(16rem+1.75rem)]"
    aria-label="AI Chat öffnen"
    title="AI Chat öffnen"
    @click="open"
  >
    <svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-3 3v-3z" />
    </svg>
  </button>
</template>

<style scoped>
.chat-panel-enter-active,
.chat-panel-leave-active {
  transition: opacity 180ms ease, transform 180ms ease;
}
.chat-panel-enter-from,
.chat-panel-leave-to {
  opacity: 0;
  transform: translateY(12px) scale(0.98);
}
</style>
