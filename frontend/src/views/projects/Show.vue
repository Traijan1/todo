<script setup lang="ts">
import {
  computed,
  onMounted,
  onUnmounted,
  ref,
  nextTick,
  shallowRef,
  watch,
} from "vue";
import { storeToRefs } from "pinia";
import { useBoardStore } from "../../stores/boards";
import { useProjectStore } from "../../stores/projects";
import { useTodoStore } from "../../stores/todos";
import { useTagStore } from "../../stores/tags";
import SideDrawer from "../../components/SideDrawer.vue";
import TaskDrawer from "./drawers/TaskDrawer.vue";
import BoardDrawer from "./drawers/BoardDrawer.vue";
import BoardColumn from "./components/BoardColumn.vue";
import type { Todo, Board } from "../../api/models";
import { lastMutationTime } from "../../api/client";

const props = defineProps<{
  pid: string;
}>();

const projectStore = useProjectStore();
const boardStore = useBoardStore();
const todoStore = useTodoStore();
const tagStore = useTagStore();
const { boards } = storeToRefs(boardStore);

const activeDrawer = shallowRef<any>(null);
const drawerData = ref<any>({});
const selectedTodo = ref<Todo | null>(null);
const selectedBoard = ref<Board | null>(null);
const drawerRef = ref<any>(null);
const boardsRef = ref<HTMLElement | null>(null);

const activeTagFilters = ref<string[]>([]);
const searchQuery = ref("");
const showTagDropdown = ref(false);
const activeBoardIndex = ref(0);

const project = computed(() =>
  projectStore.projects.find((p) => p.pid === props.pid),
);
const isDrawerOpen = computed(() => !!activeDrawer.value);

const drawerMeta = computed(() => {
  if (activeDrawer.value === TaskDrawer) {
    return {
      title: selectedTodo.value ? "Edit Task" : "New Task",
      subtitle: selectedTodo.value
        ? "Update task details"
        : "Create a new mission",
      pid: selectedTodo.value?.pid,
    };
  }
  if (activeDrawer.value === BoardDrawer) {
    return {
      title: "Board Settings",
      subtitle: "Configure board properties",
      pid: selectedBoard.value?.pid,
    };
  }
  return { title: "", subtitle: "", pid: "" };
});

const closeDrawer = () => {
  activeDrawer.value = null;
  selectedTodo.value = null;
  selectedBoard.value = null;
  drawerData.value = {};
  showTagDropdown.value = false;
};

const openCreateTask = async (boardPid: string) => {
  selectedTodo.value = null;
  drawerData.value = { todo: null, boardPid, projectPid: props.pid };
  activeDrawer.value = TaskDrawer;
  await nextTick();
  drawerRef.value?.focus();
};

const openEditTask = async (todo: Todo) => {
  selectedTodo.value = todo;
  drawerData.value = { todo, boardPid: null, projectPid: props.pid };
  activeDrawer.value = TaskDrawer;
  await nextTick();
  drawerRef.value?.focus();
};

const openEditBoard = (board: Board) => {
  selectedBoard.value = board;
  drawerData.value = { board };
  activeDrawer.value = BoardDrawer;
};

const handleTaskSave = async (form: {
  title: string;
  description: string;
  tags?: string[];
  locked?: boolean;
  boardPid?: string;
}) => {
  try {
    if (selectedTodo.value) {
      await todoStore.updateTodo(selectedTodo.value.pid, {
        title: form.title,
        details: form.description,
        tags: form.tags,
        locked: form.locked,
        board_pid: form.boardPid,
      });
    } else {
      await todoStore.createTodo(drawerData.value.boardPid, {
        title: form.title,
        details: form.description,
        tags: form.tags,
      });
    }
    await boardStore.fetchBoards(props.pid);
    closeDrawer();
  } catch (err) {
    console.error("Save task failed", err);
  }
};

const handleBoardSave = async (form: { title: string }) => {
  if (!selectedBoard.value) return;
  try {
    await boardStore.updateBoard(selectedBoard.value.pid, {
      title: form.title,
    });
    await boardStore.fetchBoards(props.pid);
    closeDrawer();
  } catch (err) {
    console.error("Save board failed", err);
  }
};

const handleSubtaskChanged = async () => {
  await boardStore.fetchBoards(props.pid);
  if (selectedTodo.value) {
    const pid = selectedTodo.value.pid;
    let found: any = null;
    for (const board of boards.value) {
      found = board.todos.find((t) => t.pid === pid);
      if (found) break;
    }
    if (found) {
      selectedTodo.value = found;
      drawerData.value = { ...drawerData.value, todo: found };
    } else {
      closeDrawer();
    }
  }
};

const deleteTask = async (todo: Todo) => {
  if (confirm(`Delete "${todo.title}"?`)) {
    await todoStore.deleteTodo(todo.pid);
    await boardStore.fetchBoards(props.pid);
    if (selectedTodo.value?.pid === todo.pid) closeDrawer();
  }
};

const handleMove = async (event: any, boardPid: string) => {
  if (event.added) {
    await todoStore.updateTodo(event.added.element.pid, {
      board_pid: boardPid,
    });
    await boardStore.reorderTodos(boardPid);
  } else if (event.moved) {
    await boardStore.reorderTodos(boardPid);
  }
};

const toggleTagFilter = (pid: string) => {
  const idx = activeTagFilters.value.indexOf(pid);
  if (idx === -1) {
    activeTagFilters.value.push(pid);
  } else {
    activeTagFilters.value.splice(idx, 1);
  }
};

const deleteTag = async (pid: string) => {
  if (!confirm("Tag löschen?")) return;
  activeTagFilters.value = activeTagFilters.value.filter((f) => f !== pid);
  await tagStore.deleteTag(pid);
  await boardStore.fetchBoards(props.pid);
};

const ensureReadableColor = (hex: string): string => {
  let cleanHex = hex.replace("#", "");
  if (cleanHex.length === 3) {
    cleanHex = cleanHex
      .split("")
      .map((char) => char + char)
      .join("");
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

const tagChipStyle = (color?: string, active = false) => {
  const rawColor = color || "#9f75ff";
  const c = ensureReadableColor(rawColor);
  if (active) {
    return {
      backgroundColor: `${c}33`,
      color: c,
      borderColor: `${c}88`,
      boxShadow: `0 0 12px ${c}22`,
    };
  }
  return { backgroundColor: `${c}0d`, color: `${c}99`, borderColor: `${c}22` };
};

const totalVisible = computed(() => {
  if (!activeTagFilters.value.length && !searchQuery.value.trim()) return null;
  return boards.value.reduce((sum, board) => {
    return (
      sum +
      board.todos.filter((t) => {
        const tagMatch =
          !activeTagFilters.value.length ||
          t.tags?.some((tag) => activeTagFilters.value.includes(tag.pid));
        const searchMatch =
          !searchQuery.value.trim() ||
          t.title.toLowerCase().includes(searchQuery.value.toLowerCase());
        return tagMatch && searchMatch;
      }).length
    );
  }, 0);
});

const onBoardScroll = () => {
  if (!boardsRef.value) return;
  const el = boardsRef.value;
  const index = Math.round(el.scrollLeft / el.clientWidth);
  activeBoardIndex.value = Math.max(
    0,
    Math.min(index, boards.value.length - 1),
  );
};

const scrollToBoard = (index: number) => {
  if (!boardsRef.value) return;
  boardsRef.value.scrollTo({
    left: index * boardsRef.value.clientWidth,
    behavior: "smooth",
  });
  activeBoardIndex.value = index;
};

const handleGlobalClick = () => {
  showTagDropdown.value = false;
};
onMounted(() => window.addEventListener("click", handleGlobalClick));
onUnmounted(() => window.removeEventListener("click", handleGlobalClick));

// ── WebSocket for real-time updates ──────────────────────────────────────────
let ws: WebSocket | null = null;
let wsReconnectTimer: ReturnType<typeof setTimeout> | null = null;

const connectWs = () => {
  const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsUrl = `${wsProtocol}//${window.location.host}/api/ws`;

  ws = new WebSocket(wsUrl);

  ws.onmessage = async (e) => {
    try {
      const { event, project_pid } = JSON.parse(e.data);
      if (
        (event === "refresh" && project_pid === props.pid) ||
        event === "refresh_all"
      ) {
        // Skip refreshing if we just performed a mutation in the last 2.5 seconds
        if (Date.now() - lastMutationTime < 2500) {
          return;
        }
        await boardStore.fetchBoards(props.pid);
        await tagStore.fetchTags(props.pid);
      }
    } catch {}
  };

  ws.onclose = () => {
    wsReconnectTimer = setTimeout(connectWs, 3000);
  };

  ws.onerror = () => {
    ws?.close();
  };
};

onMounted(() => connectWs());
onUnmounted(() => {
  if (wsReconnectTimer) clearTimeout(wsReconnectTimer);
  if (ws) {
    ws.onclose = null;
    ws.close();
  }
});

const loadProject = async (pid: string) => {
  closeDrawer();
  activeTagFilters.value = [];
  if (!project.value) await projectStore.fetchProjects();
  await Promise.all([boardStore.fetchBoards(pid), tagStore.fetchTags(pid)]);
};

onMounted(() => loadProject(props.pid));
watch(
  () => props.pid,
  (pid) => loadProject(pid),
);
</script>

<template>
  <div class="project-view flex flex-col min-w-0 overflow-hidden" @click="closeDrawer">
    <!-- Header (desktop only — mobile uses the app header) -->
    <header class="hidden lg:flex items-center justify-between mb-3 shrink-0">
      <h2 class="text-xl font-bold text-brand-primary tracking-tight truncate">{{ project?.title || "Loading..." }}</h2>
      <RouterLink
        :to="`/projects/${pid}/settings`"
        class="p-2 rounded-xl text-brand-text-muted/30 hover:text-brand-primary hover:bg-brand-primary/10 transition-all"
        title="Projekteinstellungen"
        @click.stop
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </RouterLink>
    </header>

    <!-- Filter Bar -->
    <div class="shrink-0 mb-3" @click.stop="showTagDropdown = false">
      <div class="flex items-center gap-2">
        <!-- Search Input -->
        <div class="relative flex-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3 w-3 text-brand-text-muted/40 pointer-events-none" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search tasks..."
            class="w-full pl-8 pr-7 py-1.5 text-[11px] bg-brand-container/60 border border-brand-primary/10 rounded-xl text-brand-text placeholder:text-brand-text-muted/30 focus:outline-none focus:border-brand-primary/30 transition-colors"
          />
          <button
            v-if="searchQuery"
            @click.stop="searchQuery = ''"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-brand-text-muted/40 hover:text-brand-text-muted text-base leading-none"
          >×</button>
        </div>

        <!-- Tag Dropdown -->
        <div v-if="tagStore.tags.length" class="relative shrink-0" @click.stop>
          <button
            @click.stop="showTagDropdown = !showTagDropdown"
            class="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium rounded-xl border transition-all whitespace-nowrap"
            :class="activeTagFilters.length
              ? 'bg-brand-primary/20 text-brand-primary border-brand-primary/40'
              : 'bg-brand-container/60 text-brand-text-muted border-brand-primary/10 hover:border-brand-primary/30 hover:text-brand-text'"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
            Tags
            <span
              v-if="activeTagFilters.length"
              class="bg-brand-primary text-brand-container text-[9px] font-black px-1.5 py-0.5 rounded-full leading-none"
            >{{ activeTagFilters.length }}</span>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 transition-transform duration-200" :class="{ 'rotate-180': showTagDropdown }" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          <!-- Dropdown Panel -->
          <Transition name="dropdown">
            <div
              v-if="showTagDropdown"
              class="absolute right-0 top-full mt-1.5 z-20 bg-brand-container border border-brand-primary/10 rounded-xl shadow-xl overflow-hidden min-w-44"
            >
              <div v-if="activeTagFilters.length" class="px-3 py-2 border-b border-brand-primary/10">
                <button @click.stop="activeTagFilters = []" class="text-[10px] text-brand-primary/60 hover:text-brand-primary transition-colors font-bold uppercase tracking-widest">
                  Clear selection
                </button>
              </div>
              <div class="py-1">
                <div
                  v-for="tag in tagStore.tags"
                  :key="tag.pid"
                  class="flex items-center gap-2 px-3 py-1.5 hover:bg-white/5 group/item"
                >
                  <button @click.stop="toggleTagFilter(tag.pid)" class="flex-1 flex items-center gap-2 text-left">
                    <div
                      class="w-3.5 h-3.5 rounded border flex items-center justify-center shrink-0 transition-all"
                      :class="activeTagFilters.includes(tag.pid) ? 'bg-brand-primary border-brand-primary' : 'border-brand-primary/20'"
                    >
                      <svg v-if="activeTagFilters.includes(tag.pid)" class="h-2 w-2 text-brand-container" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                        <path d="M2 6l3 3 5-5" />
                      </svg>
                    </div>
                    <span class="w-2 h-2 rounded-full shrink-0" :style="{ backgroundColor: ensureReadableColor(tag.color || '#9f75ff') }" />
                    <span class="text-[11px] text-brand-text">{{ tag.title }}</span>
                  </button>
                  <button
                    @click.stop="deleteTag(tag.pid)"
                    class="text-red-500/30 hover:text-red-400 transition-colors text-sm leading-none opacity-0 group-hover/item:opacity-100"
                    title="Tag löschen"
                  >×</button>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Result count + clear -->
      <Transition name="fade-count">
        <div v-if="totalVisible !== null" class="flex items-center gap-2 mt-1.5">
          <span class="text-[9px] font-bold uppercase tracking-widest text-brand-text-muted/40">{{ totalVisible }} found</span>
          <button @click.stop="activeTagFilters = []; searchQuery = ''" class="text-[9px] text-brand-primary/40 hover:text-brand-primary transition-colors">
            Clear all
          </button>
        </div>
      </Transition>
    </div>

    <!-- Boards Container -->
    <div
      class="flex-1 flex flex-col lg:grid min-h-0"
      :style="{ gridTemplateColumns: isDrawerOpen ? '1fr 600px' : '1fr 0px' }"
    >
      <!-- Main Content -->
      <div class="flex flex-col min-h-0 min-w-0">
        <!-- Boards: side-by-side columns, snap on mobile / free scroll on desktop -->
        <div
          ref="boardsRef"
          class="flex-1 min-h-0 min-w-0
            flex flex-row items-stretch gap-3
            overflow-x-auto snap-x snap-mandatory scroll-smooth overscroll-x-contain
            lg:snap-none lg:pb-6
            custom-scrollbar-x"
          @scroll.passive="onBoardScroll"
        >
          <div
            v-for="board in boards"
            :key="board.pid"
            class="snap-center shrink-0 w-[88vw] sm:w-80 lg:w-72 xl:w-80
              h-full overflow-y-auto overscroll-y-contain pb-16 lg:pb-4"
          >
            <BoardColumn
              v-model:todos="board.todos"
              :board="board"
              :filter-tags="activeTagFilters"
              :filter-search="searchQuery"
              @edit-board="openEditBoard(board)"
              @create-task="openCreateTask(board.pid)"
              @edit-task="openEditTask"
              @delete-task="deleteTask"
              @change="handleMove($event, board.pid)"
            />
          </div>
        </div>

        <!-- Mobile board navigation (in-layout, always visible at bottom) -->
        <div
          v-if="boards.length > 1"
          class="shrink-0 lg:hidden flex items-center gap-3 px-4 py-2.5
            bg-brand-container/80 backdrop-blur-md border-t border-brand-primary/10"
          style="padding-bottom: max(0.625rem, env(safe-area-inset-bottom))"
        >
          <button
            class="p-2 rounded-xl transition-all"
            :class="activeBoardIndex === 0
              ? 'text-brand-primary/20 cursor-default'
              : 'text-brand-primary/60 hover:text-brand-primary hover:bg-brand-primary/10'"
            :disabled="activeBoardIndex === 0"
            @click="scrollToBoard(activeBoardIndex - 1)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M15 19l-7-7 7-7" />
            </svg>
          </button>

          <div class="flex-1 flex flex-col items-center min-w-0">
            <p class="text-sm font-black text-brand-primary truncate max-w-full">
              {{ boards[activeBoardIndex]?.title }}
            </p>
            <p class="text-[9px] font-bold uppercase tracking-widest text-brand-primary/30">
              {{ activeBoardIndex + 1 }} / {{ boards.length }}
            </p>
          </div>

            <button
              class="p-2 rounded-xl transition-all"
              :class="activeBoardIndex === boards.length - 1
                ? 'text-brand-primary/20 cursor-default'
                : 'text-brand-primary/60 hover:text-brand-primary hover:bg-brand-primary/10'"
              :disabled="activeBoardIndex === boards.length - 1"
              @click="scrollToBoard(activeBoardIndex + 1)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
              </svg>
            </button>
        </div>
      </div>

      <!-- Desktop Spacer for Drawer -->
      <div class="hidden lg:block h-full pointer-events-none"></div>
    </div>

    <!-- SideDrawer -->
    <SideDrawer :is-open="isDrawerOpen" v-bind="drawerMeta" @close="closeDrawer">
      <component
        :is="activeDrawer"
        ref="drawerRef"
        v-bind="drawerData"
        @save="activeDrawer === TaskDrawer ? handleTaskSave($event) : handleBoardSave($event)"
        @subtask-changed="handleSubtaskChanged"
      />

      <template #footer>
        <div class="flex gap-3">
          <button
            @click="activeDrawer === TaskDrawer ? handleTaskSave(drawerRef.form) : handleBoardSave(drawerRef.form)"
            class="flex-1 bg-brand-primary text-brand-container py-3 rounded-xl font-bold uppercase tracking-widest text-[10px] hover:bg-brand-primary/90 transition-all shadow-md"
          >
            Save Changes
          </button>

          <button v-if="selectedTodo" @click="deleteTask(selectedTodo)" class="w-10 h-10 rounded-xl bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white transition-all flex items-center justify-center shrink-0">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </template>
    </SideDrawer>
  </div>
</template>

<style scoped>
/* On mobile: lock height to exactly the available screen space so the
   flex-1 / h-full chain works and boards don't overflow the viewport. */
@media (max-width: 1023px) {
  .project-view {
    height: calc(100dvh - 3.75rem - env(safe-area-inset-top) - max(1rem, env(safe-area-inset-bottom)));
    flex: none;
  }
}

/* On desktop: let it grow normally inside the sidebar layout */
@media (min-width: 1024px) {
  .project-view {
    flex: 1;
  }
}

.hide-scrollbar {
  scrollbar-width: none;
}
.hide-scrollbar::-webkit-scrollbar {
  display: none;
}

.custom-scrollbar::-webkit-scrollbar {
  height: 6px;
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.1);
  border-radius: 20px;
}

.custom-scrollbar-y::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar-y::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar-y::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.1);
  border-radius: 20px;
}

.custom-scrollbar-x::-webkit-scrollbar {
  height: 4px;
}
.custom-scrollbar-x::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar-x::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.1);
  border-radius: 20px;
}

.fade-count-enter-active,
.fade-count-leave-active {
  transition: opacity 0.2s ease;
}
.fade-count-enter-from,
.fade-count-leave-to {
  opacity: 0;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
