<script setup lang="ts">
import { ref, watch, nextTick, computed, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import type { Todo, SubtaskItem, Comment } from "../../../api/models";
import TiptapEditor from "../../../components/editor/TiptapEditor.vue";
import { useTagStore } from "../../../stores/tags";
import { useBoardStore } from "../../../stores/boards";
import { useTodoStore } from "../../../stores/todos";
import { useAuthStore } from "../../../stores/auth";
import api from "../../../api/client";

const props = defineProps<{
  todo: Todo | null;
  boardPid: string | null;
  projectPid: string;
}>();

const emit = defineEmits<{
  (
    e: "save",
    data: {
      title: string;
      description: string;
      tags: string[];
      locked: boolean;
      boardPid?: string;
    },
  ): void;
  (e: "subtask-changed"): void;
}>();

const tagStore = useTagStore();
const boardStore = useBoardStore();
const todoStore = useTodoStore();

const TAG_COLORS = [
  "#9f75ff", // Violet
  "#6366f1", // Indigo
  "#3b82f6", // Blue
  "#06b6d4", // Cyan
  "#10b981", // Emerald
  "#eab308", // Amber
  "#f43f5e", // Rose
  "#ec4899", // Pink
];

const form = ref({
  title: props.todo?.title || "",
  description: props.todo?.details || "",
  tags: props.todo?.tags?.map((t) => t.pid) || ([] as string[]),
  locked: props.todo?.locked ?? false,
  boardPid: props.todo?.board_pid ?? props.boardPid ?? "",
});

const showTagPicker = ref(false);
const newTagTitle = ref("");
const newTagColor = ref(TAG_COLORS[0]);

const titleInputRef = ref<HTMLTextAreaElement | null>(null);

watch(
  () => props.todo,
  (newTodo) => {
    form.value = {
      title: newTodo?.title || "",
      description: newTodo?.details || "",
      tags: newTodo?.tags?.map((t) => t.pid) || [],
      locked: newTodo?.locked ?? false,
      boardPid: newTodo?.board_pid ?? props.boardPid ?? "",
    };
    showTagPicker.value = false;
    showParentPicker.value = false;
    parentSearch.value = "";
  },
  { immediate: true },
);

const adjustHeight = () => {
  const el = titleInputRef.value;
  if (el) {
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  }
};

watch(
  () => form.value.title,
  () => {
    nextTick(adjustHeight);
  },
);

const selectedTagObjects = computed(() =>
  tagStore.tags.filter((t) => form.value.tags.includes(t.pid)),
);

const toggleTag = (pid: string) => {
  const idx = form.value.tags.indexOf(pid);
  if (idx === -1) {
    form.value.tags.push(pid);
  } else {
    form.value.tags.splice(idx, 1);
  }
};

const createAndAddTag = async () => {
  const title = newTagTitle.value.trim();
  if (!title) return;
  const tag = await tagStore.createTag(
    props.projectPid,
    title,
    newTagColor.value,
  );
  form.value.tags.push(tag.pid);
  newTagTitle.value = "";
  newTagColor.value = TAG_COLORS[0];
};

const deleteTag = async (pid: string) => {
  await tagStore.deleteTag(pid);
  form.value.tags = form.value.tags.filter((t) => t !== pid);
};

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

const parentSearch = ref("");
const showParentPicker = ref(false);

const allProjectTodos = computed(() =>
  boardStore.boards
    .flatMap((b) => b.todos)
    .filter((t) => t.pid !== props.todo?.pid),
);

const filteredParentTodos = computed(() => {
  const q = parentSearch.value.trim().toLowerCase();
  if (!q) return [];
  return allProjectTodos.value
    .filter((t) => t.title.toLowerCase().includes(q))
    .slice(0, 8);
});

const setParent = async (parent: { pid: string; title: string }) => {
  if (!props.todo) return;
  showParentPicker.value = false;
  parentSearch.value = "";
  try {
    await todoStore.updateTodo(props.todo.pid, { parent_pid: parent.pid });
    emit("subtask-changed");
  } catch (e) {
    console.error("Failed to set parent", e);
  }
};

const localSubtasks = ref<SubtaskItem[]>([]);
const newSubtaskTitle = ref("");

watch(
  () => props.todo,
  (t) => {
    localSubtasks.value = t?.subtasks ? [...t.subtasks] : [];
  },
  { immediate: true },
);

const addSubtask = async () => {
  const title = newSubtaskTitle.value.trim();
  if (!title || !props.todo) return;
  try {
    await todoStore.createTodo(form.value.boardPid, {
      title,
      parent_pid: props.todo.pid,
    });
    newSubtaskTitle.value = "";
    emit("subtask-changed");
  } catch (e) {
    console.error("Failed to add subtask", e);
  }
};

const removeSubtask = async (pid: string) => {
  try {
    await todoStore.deleteTodo(pid);
    localSubtasks.value = localSubtasks.value.filter((s) => s.pid !== pid);
    emit("subtask-changed");
  } catch (e) {
    console.error("Failed to delete subtask", e);
  }
};

// ── Comments ──────────────────────────────────────────────────────────────────
const comments = ref<Comment[]>([]);
const newCommentContent = ref("");
const commentSubmitting = ref(false);
const editingCommentPid = ref<string | null>(null);
const editingContent = ref("");
const editSaving = ref(false);

const { user } = storeToRefs(useAuthStore());

const startEdit = (comment: Comment) => {
  editingCommentPid.value = comment.pid;
  editingContent.value = comment.content;
};

const cancelEdit = () => {
  editingCommentPid.value = null;
  editingContent.value = "";
};

const saveEdit = async (comment: Comment) => {
  if (!editingContent.value.trim() || !props.todo || editSaving.value) return;
  editSaving.value = true;
  try {
    const { data } = await api.patch(`/todos/${props.todo.pid}/comments/${comment.pid}`, {
      content: editingContent.value.trim(),
    });
    const idx = comments.value.findIndex((c) => c.pid === comment.pid);
    if (idx !== -1) comments.value[idx] = data;
    cancelEdit();
  } finally {
    editSaving.value = false;
  }
};

const deleteComment = async (comment: Comment) => {
  if (!props.todo) return;
  await api.delete(`/todos/${props.todo.pid}/comments/${comment.pid}`);
  comments.value = comments.value.filter((c) => c.pid !== comment.pid);
};

const formatCommentTime = (dateStr: string) => {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return new Date(dateStr).toLocaleDateString(undefined, { month: "short", day: "numeric" });
};

const loadComments = async (todoPid: string) => {
  try {
    const { data } = await api.get(`/todos/${todoPid}/comments`);
    comments.value = data;
  } catch {
    comments.value = [];
  }
};

const submitComment = async () => {
  const content = newCommentContent.value.trim();
  if (!content || !props.todo || commentSubmitting.value) return;
  commentSubmitting.value = true;
  try {
    const { data } = await api.post(`/todos/${props.todo.pid}/comments`, { content });
    comments.value.push(data);
    newCommentContent.value = "";
  } finally {
    commentSubmitting.value = false;
  }
};

// ── Timer ─────────────────────────────────────────────────────────────────────
interface TimerState {
  running: boolean;
  started_at: string | null;
  total_seconds: number;
  total_formatted: string;
}

const timer = ref<TimerState | null>(null);
const timerActing = ref(false);
const timerElapsed = ref(0); // live seconds since start
let tickInterval: ReturnType<typeof setInterval> | null = null;

const startTick = (startedAt: string) => {
  if (tickInterval) clearInterval(tickInterval);
  const base = new Date(startedAt).getTime();
  const tick = () => {
    timerElapsed.value = Math.floor((Date.now() - base) / 1000);
  };
  tick();
  tickInterval = setInterval(tick, 1000);
};

const stopTick = () => {
  if (tickInterval) { clearInterval(tickInterval); tickInterval = null; }
};

onUnmounted(stopTick);

const formatSecs = (s: number) => {
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const h = Math.floor(m / 60);
  return h > 0 ? `${h}h ${m % 60}m` : `${m}m`;
};

const loadTimer = async (todoPid: string) => {
  try {
    const { data } = await api.get(`/todos/${todoPid}/timer`);
    timer.value = data;
    if (data.running && data.started_at) startTick(data.started_at);
    else stopTick();
  } catch { timer.value = null; }
};

const startTimer = async () => {
  if (!props.todo || timerActing.value) return;
  timerActing.value = true;
  try {
    await api.post(`/todos/${props.todo.pid}/timer/start`);
    await loadTimer(props.todo.pid);
  } finally { timerActing.value = false; }
};

const stopTimer = async () => {
  if (!props.todo || timerActing.value) return;
  timerActing.value = true;
  try {
    await api.post(`/todos/${props.todo.pid}/timer/stop`);
    stopTick();
    await loadTimer(props.todo.pid);
  } finally { timerActing.value = false; }
};

watch(
  () => props.todo?.pid,
  (pid) => {
    comments.value = [];
    stopTick();
    timer.value = null;
    if (pid) {
      loadComments(pid);
      loadTimer(pid);
    }
  },
  { immediate: true },
);

defineExpose({
  form,
  focus: () => {
    titleInputRef.value?.focus();
    adjustHeight();
  },
});
</script>

<template>
  <div class="space-y-8">
    <div class="space-y-2">
      <label class="brand-label">Title</label>
      <textarea
        ref="titleInputRef"
        v-model="form.title"
        placeholder="Name your mission..."
        class="brand-textarea min-h-[42px] text-lg font-bold leading-tight overflow-hidden resize-none"
        rows="1"
        @input="adjustHeight"
      ></textarea>
    </div>

    <!-- Board picker (only when editing an existing todo) -->
    <div v-if="todo && boardStore.boards.length > 1" class="space-y-2">
      <label class="brand-label">Board</label>
      <div class="flex gap-2 overflow-x-auto pb-1 hide-scrollbar">
        <button
          v-for="board in boardStore.boards"
          :key="board.pid"
          type="button"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold border transition-all shrink-0"
          :class="form.boardPid === board.pid
            ? 'bg-brand-primary/20 text-brand-primary border-brand-primary/40'
            : 'bg-white/5 text-brand-text-muted border-white/10 hover:border-brand-primary/20 hover:text-brand-text'"
          @click="form.boardPid = board.pid"
        >
          <span
            class="w-1.5 h-1.5 rounded-full shrink-0"
            :class="form.boardPid === board.pid ? 'bg-brand-primary' : 'bg-brand-text-muted/40'"
          />
          {{ board.title }}
        </button>
      </div>
    </div>

    <!-- Tags -->
    <div class="space-y-2">
      <label class="brand-label">Tags</label>
      <div class="relative">
        <div class="flex flex-wrap gap-1.5 items-center min-h-[28px]">
          <button
            v-for="tag in selectedTagObjects"
            :key="tag.pid"
            type="button"
            class="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border transition-all hover:opacity-70"
            :style="tagStyle(tag.color)"
            @click.stop="toggleTag(tag.pid)"
          >
            {{ tag.title }}
            <span class="opacity-50 text-xs leading-none">×</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-brand-primary/10 text-brand-primary/60 hover:bg-brand-primary/20 hover:text-brand-primary transition-all border border-brand-primary/10"
            @click.stop="showTagPicker = !showTagPicker"
          >
            + Tag
          </button>
        </div>

        <!-- Backdrop -->
        <div
          v-if="showTagPicker"
          class="fixed inset-0 z-10"
          @click.stop="showTagPicker = false"
        />

        <!-- Picker dropdown -->
        <div
          v-if="showTagPicker"
          class="absolute top-full left-0 right-0 mt-2 z-20 bg-brand-container border border-brand-primary/20 rounded-xl p-3 shadow-2xl space-y-3"
          @click.stop
        >
          <!-- Existing tags -->
          <div v-if="tagStore.tags.length" class="flex flex-wrap gap-1.5">
            <div
              v-for="tag in tagStore.tags"
              :key="tag.pid"
              class="flex items-center rounded-full border transition-all"
              :class="form.tags.includes(tag.pid) ? 'ring-2 ring-white/30' : 'opacity-60 hover:opacity-100'"
              :style="tagStyle(tag.color)"
            >
              <button
                type="button"
                class="flex items-center gap-1 pl-2.5 pr-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wider"
                @click="toggleTag(tag.pid)"
              >
                <span v-if="form.tags.includes(tag.pid)" class="text-xs leading-none">✓</span>
                {{ tag.title }}
              </button>
              <button
                type="button"
                class="pr-2 py-0.5 text-[10px] opacity-50 hover:opacity-100 transition-opacity leading-none"
                title="Delete tag"
                @click.stop="deleteTag(tag.pid)"
              >×</button>
            </div>
          </div>

          <!-- Divider -->
          <div class="border-t border-brand-primary/10 pt-3 space-y-2">
            <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/30">Create New</p>
            <input
              v-model="newTagTitle"
              placeholder="Tag name..."
              class="brand-input text-xs"
              @keydown.enter.stop="createAndAddTag"
            />
            <div class="flex gap-1.5 flex-wrap">
              <button
                v-for="color in TAG_COLORS"
                :key="color"
                type="button"
                class="w-5 h-5 rounded-full border-2 transition-all hover:scale-110"
                :style="{ backgroundColor: color, borderColor: newTagColor === color ? 'white' : 'transparent' }"
                @click="newTagColor = color"
              />
            </div>
            <button
              type="button"
              :disabled="!newTagTitle.trim()"
              class="w-full text-[10px] font-bold uppercase tracking-widest py-1.5 rounded-lg bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
              @click="createAndAddTag"
            >
              Create Tag
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Parent Task picker (only for existing top-level todos) -->
    <div v-if="todo && !todo.parent_pid" class="space-y-2">
      <label class="brand-label">Als Subtask setzen</label>
      <div class="relative">
        <input
          v-model="parentSearch"
          placeholder="Titel tippen um zu suchen..."
          class="brand-input text-xs"
          @focus="showParentPicker = true"
          @blur="showParentPicker = false"
        />
        <Transition name="picker-fade">
          <div
            v-if="showParentPicker && filteredParentTodos.length"
            class="absolute top-full left-0 right-0 mt-1 z-20 bg-brand-container border border-brand-primary/20 rounded-xl overflow-hidden shadow-2xl"
          >
            <button
              v-for="t in filteredParentTodos"
              :key="t.pid"
              type="button"
              class="w-full text-left px-3 py-2.5 text-xs text-brand-text hover:bg-brand-primary/10 transition-colors flex items-center gap-2"
              @mousedown.prevent
              @click="setParent(t)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-brand-primary/30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
              <span class="truncate">{{ t.title }}</span>
            </button>
          </div>
        </Transition>
      </div>
    </div>

    <!-- Subtasks (only when editing an existing todo) -->
    <div v-if="todo" class="space-y-2">
      <label class="brand-label">Subtasks</label>
      <div class="space-y-1.5">
        <div
          v-for="subtask in localSubtasks"
          :key="subtask.pid"
          class="flex items-center gap-2 px-3 py-2 rounded-xl bg-brand-background/40 border border-white/5 group/sub"
        >
          <svg v-if="subtask.locked" xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0 text-amber-400/60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          <span class="flex-1 text-xs text-brand-text truncate">{{ subtask.title }}</span>
          <button
            type="button"
            class="opacity-0 group-hover/sub:opacity-100 text-red-400 hover:text-red-300 transition-all p-0.5 rounded"
            @click="removeSubtask(subtask.pid)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="flex gap-2">
          <input
            v-model="newSubtaskTitle"
            placeholder="Add subtask..."
            class="brand-input text-xs flex-1"
            @keydown.enter.stop="addSubtask"
          />
          <button
            type="button"
            :disabled="!newSubtaskTitle.trim()"
            class="px-3 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-widest bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30 disabled:cursor-not-allowed shrink-0"
            @click="addSubtask"
          >
            Add
          </button>
        </div>
      </div>
    </div>

    <!-- AI Lock toggle -->
    <div class="flex items-center justify-between py-3 px-4 rounded-xl border border-brand-primary/10 bg-brand-background/30">
      <div class="flex items-center gap-3">
        <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0"
          :class="form.locked ? 'bg-amber-500/20' : 'bg-brand-primary/10'">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"
            :class="form.locked ? 'text-amber-400' : 'text-brand-primary/40'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <div>
          <p class="text-xs font-bold text-brand-text" :class="form.locked ? 'text-amber-300' : ''">Unsichtbar für AI</p>
          <p class="text-[9px] text-brand-text-muted/50 uppercase tracking-wider font-bold">
            {{ form.locked ? 'AI kann dieses Todo nicht lesen oder ändern' : 'AI hat vollen Zugriff' }}
          </p>
        </div>
      </div>
      <button
        type="button"
        class="relative w-10 h-5 rounded-full transition-all duration-200 shrink-0"
        :class="form.locked ? 'bg-amber-500' : 'bg-brand-primary/20'"
        @click="form.locked = !form.locked"
      >
        <span
          class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all duration-200"
          :class="form.locked ? 'left-5' : 'left-0.5'"
        />
      </button>
    </div>

    <div class="space-y-2">
      <label class="brand-label">Description</label>
      <TiptapEditor v-model="form.description" placeholder="Map out the steps..." />
    </div>

    <!-- Timer (only for existing todos) -->
    <div v-if="todo && timer !== null" class="flex items-center gap-3 py-2.5 px-3 rounded-xl bg-white/5">
      <!-- Time display -->
      <div class="flex-1 min-w-0">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40 mb-0.5">Zeit</p>
        <p class="text-sm font-black tabular-nums text-brand-text">
          <span v-if="timer.running" class="text-brand-primary">
            {{ formatSecs(timer.total_seconds + timerElapsed) }}
          </span>
          <span v-else>{{ timer.total_formatted || '0m' }}</span>
        </p>
      </div>
      <!-- Running indicator -->
      <div v-if="timer.running" class="flex items-center gap-1.5 text-brand-primary">
        <span class="w-1.5 h-1.5 rounded-full bg-brand-primary animate-pulse"/>
        <span class="text-[9px] font-black uppercase tracking-widest">Läuft</span>
      </div>
      <!-- Start / Stop button -->
      <button
        type="button"
        :disabled="timerActing"
        class="shrink-0 px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-widest transition-all disabled:opacity-40"
        :class="timer.running
          ? 'bg-red-500/15 text-red-400 hover:bg-red-500/25'
          : 'bg-brand-primary/15 text-brand-primary hover:bg-brand-primary/25'"
        @click="timer.running ? stopTimer() : startTimer()"
      >
        {{ timerActing ? '...' : timer.running ? 'Stop' : 'Start' }}
      </button>
    </div>

    <!-- Comments (only for existing todos) -->
    <div v-if="todo" class="space-y-3">
      <label class="brand-label">Comments</label>

      <!-- Existing comments -->
      <div v-if="comments.length" class="space-y-2">
        <div
          v-for="comment in comments"
          :key="comment.pid"
          class="flex gap-2.5 group/comment"
        >
          <!-- Avatar -->
          <div
            class="w-6 h-6 rounded-full shrink-0 flex items-center justify-center text-[9px] font-black mt-0.5"
            :class="comment.is_ai ? 'bg-brand-primary/20 text-brand-primary' : 'bg-white/10 text-brand-text-muted'"
          >
            {{ comment.is_ai ? 'AI' : comment.author[0]?.toUpperCase() }}
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-baseline gap-2 mb-0.5">
              <span class="text-[10px] font-bold text-brand-text/70">{{ comment.author }}</span>
              <span v-if="comment.is_ai" class="text-[8px] font-black uppercase tracking-widest text-brand-primary/40 bg-brand-primary/10 px-1.5 py-0.5 rounded-full">AI</span>
              <span class="text-[9px] text-brand-text-muted/30 ml-auto">{{ formatCommentTime(comment.created_at) }}</span>
              <!-- Edit + Delete — only own, non-AI comments -->
              <template v-if="!comment.is_ai && comment.author === user?.name && editingCommentPid !== comment.pid">
                <button
                  class="opacity-0 group-hover/comment:opacity-100 text-brand-primary/30 hover:text-brand-primary transition-all"
                  title="Bearbeiten"
                  @click="startEdit(comment)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button
                  class="opacity-0 group-hover/comment:opacity-100 text-red-400/30 hover:text-red-400 transition-all"
                  title="Löschen"
                  @click="deleteComment(comment)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </template>
            </div>
            <!-- Edit mode -->
            <div v-if="editingCommentPid === comment.pid" class="space-y-1.5">
              <textarea
                v-model="editingContent"
                class="brand-textarea text-xs w-full resize-none min-h-[60px]"
                @keydown.enter.exact.prevent="saveEdit(comment)"
                @keydown.escape="cancelEdit"
                autofocus
              />
              <div class="flex gap-2">
                <button
                  type="button"
                  :disabled="editSaving || !editingContent.trim()"
                  class="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30"
                  @click="saveEdit(comment)"
                >{{ editSaving ? '...' : 'Speichern' }}</button>
                <button
                  type="button"
                  class="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-white/5 text-brand-text-muted hover:bg-white/10 transition-all"
                  @click="cancelEdit"
                >Abbrechen</button>
              </div>
            </div>
            <p v-else class="text-xs text-brand-text/70 leading-relaxed whitespace-pre-wrap break-words">{{ comment.content }}</p>
          </div>
        </div>
      </div>

      <div v-else class="text-[10px] text-brand-text-muted/30 font-medium text-center py-2">
        Noch keine Kommentare
      </div>

      <!-- New comment input -->
      <div class="flex gap-2 items-end">
        <textarea
          v-model="newCommentContent"
          placeholder="Kommentar schreiben..."
          rows="1"
          class="brand-textarea flex-1 text-xs resize-none min-h-[36px] max-h-32"
          style="overflow-y: auto"
          @keydown.enter.exact.prevent="submitComment"
          @input="($event.target as HTMLTextAreaElement).style.height = 'auto'; ($event.target as HTMLTextAreaElement).style.height = ($event.target as HTMLTextAreaElement).scrollHeight + 'px'"
        />
        <button
          type="button"
          :disabled="!newCommentContent.trim() || commentSubmitting"
          class="px-3 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30 disabled:cursor-not-allowed shrink-0"
          @click="submitComment"
        >
          <svg v-if="commentSubmitting" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hide-scrollbar {
  scrollbar-width: none;
}
.hide-scrollbar::-webkit-scrollbar {
  display: none;
}
.picker-fade-enter-active,
.picker-fade-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}
.picker-fade-enter-from,
.picker-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
