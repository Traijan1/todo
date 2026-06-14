<script setup lang="ts">
import { computed, onMounted, ref, nextTick, shallowRef } from "vue";
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

const activeTagFilters = ref<string[]>([]);

const project = computed(() => projectStore.projects.find((p) => p.pid === props.pid));
const isDrawerOpen = computed(() => !!activeDrawer.value);

const drawerMeta = computed(() => {
  if (activeDrawer.value === TaskDrawer) {
    return {
      title: selectedTodo.value ? "Edit Task" : "New Task",
      subtitle: selectedTodo.value ? "Update task details" : "Create a new mission",
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
};

const openCreateTask = async (boardPid: string) => {
  selectedTodo.value = null;
  drawerData.value = { todo: null, boardPid };
  activeDrawer.value = TaskDrawer;
  await nextTick();
  drawerRef.value?.focus();
};

const openEditTask = async (todo: Todo) => {
  selectedTodo.value = todo;
  drawerData.value = { todo, boardPid: null };
  activeDrawer.value = TaskDrawer;
  await nextTick();
  drawerRef.value?.focus();
};

const openEditBoard = (board: Board) => {
  selectedBoard.value = board;
  drawerData.value = { board };
  activeDrawer.value = BoardDrawer;
};

const handleTaskSave = async (form: { title: string; description: string; tags?: string[] }) => {
  try {
    if (selectedTodo.value) {
      await todoStore.updateTodo(selectedTodo.value.pid, {
        title: form.title,
        details: form.description,
        tags: form.tags,
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
    await boardStore.updateBoard(selectedBoard.value.pid, { title: form.title });
    await boardStore.fetchBoards(props.pid);
    closeDrawer();
  } catch (err) {
    console.error("Save board failed", err);
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
    await todoStore.updateTodo(event.added.element.pid, { board_pid: boardPid });
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

const tagChipStyle = (color?: string, active = false) => {
  const c = color || "#E0BBE4";
  if (active) {
    return { backgroundColor: `${c}33`, color: c, borderColor: `${c}88`, boxShadow: `0 0 12px ${c}22` };
  }
  return { backgroundColor: `${c}0d`, color: `${c}99`, borderColor: `${c}22` };
};

const totalVisible = computed(() => {
  if (!activeTagFilters.value.length) return null;
  return boards.value.reduce((sum, board) => {
    return sum + board.todos.filter((t) =>
      t.tags?.some((tag) => activeTagFilters.value.includes(tag.pid))
    ).length;
  }, 0);
});

onMounted(async () => {
  if (!project.value) await projectStore.fetchProjects();
  await Promise.all([
    boardStore.fetchBoards(props.pid),
    tagStore.fetchTags(),
  ]);
});
</script>

<template>
  <div class="flex-1 flex flex-col min-w-0 h-full overflow-hidden" @click="closeDrawer">
    <!-- Header -->
    <header class="mb-3 shrink-0">
      <h2 class="text-xl font-bold text-brand-primary tracking-tight truncate">{{ project?.title || "Loading..." }}</h2>
    </header>

    <!-- Tag Filter Bar -->
    <div v-if="tagStore.tags.length" class="shrink-0 mb-4" @click.stop>
      <div class="flex items-center gap-2 flex-wrap">
        <!-- All chip -->
        <button
          type="button"
          class="flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border transition-all duration-200"
          :class="!activeTagFilters.length
            ? 'bg-brand-primary/20 text-brand-primary border-brand-primary/40 shadow-[0_0_12px_rgba(224,187,228,0.15)]'
            : 'bg-white/5 text-brand-text-muted/50 border-white/5 hover:text-brand-text-muted hover:bg-white/10'"
          @click="activeTagFilters = []"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-2.5 w-2.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
          All
        </button>

        <!-- Divider -->
        <div class="w-px h-4 bg-brand-primary/10 shrink-0" />

        <!-- Tag chips -->
        <button
          v-for="tag in tagStore.tags"
          :key="tag.pid"
          type="button"
          class="flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider border transition-all duration-200"
          :style="tagChipStyle(tag.color, activeTagFilters.includes(tag.pid))"
          @click="toggleTagFilter(tag.pid)"
        >
          <span
            class="w-1.5 h-1.5 rounded-full shrink-0"
            :style="{ backgroundColor: tag.color || '#E0BBE4' }"
          />
          {{ tag.title }}
          <span v-if="activeTagFilters.includes(tag.pid)" class="opacity-60 -ml-0.5">×</span>
        </button>

        <!-- Result count -->
        <Transition name="fade-count">
          <span
            v-if="totalVisible !== null"
            class="ml-auto text-[9px] font-bold uppercase tracking-widest text-brand-text-muted/40"
          >
            {{ totalVisible }} found
          </span>
        </Transition>
      </div>
    </div>

    <!-- Boards Container -->
    <div
      class="flex-1 lg:grid transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)] min-h-0"
      :style="{ gridTemplateColumns: isDrawerOpen ? '1fr 600px' : '1fr 0px' }"
    >
      <!-- Main Content (Boards) -->
      <div class="overflow-y-auto overflow-x-hidden custom-scrollbar-y pr-2 min-w-0">
        <div class="flex flex-col gap-4 pb-20">
          <BoardColumn
            v-for="board in boards"
            :key="board.pid"
            v-model:todos="board.todos"
            :board="board"
            :filter-tags="activeTagFilters"
            @edit-board="openEditBoard(board)"
            @create-task="openCreateTask(board.pid)"
            @edit-task="openEditTask"
            @delete-task="deleteTask"
            @change="handleMove($event, board.pid)"
          />
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

.fade-count-enter-active,
.fade-count-leave-active {
  transition: opacity 0.2s ease;
}
.fade-count-enter-from,
.fade-count-leave-to {
  opacity: 0;
}
</style>
