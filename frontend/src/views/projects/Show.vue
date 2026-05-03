<script setup lang="ts">
import { computed, onMounted, ref, nextTick, shallowRef } from "vue";
import { storeToRefs } from "pinia";
import { useBoardStore } from "../../stores/boards";
import { useProjectStore } from "../../stores/projects";
import { useTodoStore } from "../../stores/todos";
import SideDrawer from "../../components/SideDrawer.vue";
import TaskDrawer from "./drawers/TaskDrawer.vue";
import BoardDrawer from "./drawers/BoardDrawer.vue";
import type { Todo, Board } from "../../api/models";
import draggable from "vuedraggable";

const props = defineProps<{
  pid: string;
}>();

// --- Stores ---
const projectStore = useProjectStore();
const boardStore = useBoardStore();
const todoStore = useTodoStore();
const { boards } = storeToRefs(boardStore);

// --- State ---
const activeDrawer = shallowRef<any>(null);
const drawerData = ref<any>({});
const selectedTodo = ref<Todo | null>(null);
const selectedBoard = ref<Board | null>(null);

const drawerRef = ref<any>(null);

// --- Computed ---
const project = computed(() => projectStore.projects.find((p) => p.pid === props.pid));
const isDrawerOpen = computed(() => !!activeDrawer.value);

const drawerMeta = computed(() => {
  if (activeDrawer.value === TaskDrawer) {
    return {
      title: selectedTodo.value ? "Edit Task" : "New Task",
      subtitle: selectedTodo.value ? "Update task details" : "Create a new mission",
      pid: selectedTodo.value?.pid
    };
  }
  if (activeDrawer.value === BoardDrawer) {
    return {
      title: "Board Settings",
      subtitle: "Configure board properties",
      pid: selectedBoard.value?.pid
    };
  }
  return { title: "", subtitle: "", pid: "" };
});

// --- Actions ---
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

const handleTaskSave = async (form: { title: string; description: string }) => {
  try {
    if (selectedTodo.value) {
      await todoStore.updateTodo(selectedTodo.value.pid, { title: form.title, details: form.description });
    } else {
      await todoStore.createTodo(drawerData.value.boardPid, { title: form.title, details: form.description });
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

onMounted(async () => {
  if (!project.value) await projectStore.fetchProjects();
  await boardStore.fetchBoards(props.pid);
});
</script>

<template>
  <!-- Container is relative to allow SideDrawer absolute positioning if needed, 
       but for now we'll keep it fixed to viewport in its own component -->
  <div class="flex-1 flex flex-col min-w-0 h-full overflow-hidden" @click="closeDrawer">
    
    <!-- Header -->
    <header class="mb-6 shrink-0">
      <h2 class="text-3xl font-bold text-brand-primary tracking-tight truncate">{{ project?.title || "Loading..." }}</h2>
      <div class="flex items-center gap-2 mt-1">
        <div class="h-0.5 w-8 bg-brand-primary/30 rounded-full"></div>
        <p class="text-brand-text-muted text-[10px] font-bold uppercase tracking-widest">Board Overview</p>
      </div>
    </header>

    <!-- Boards Container -->
    <div class="flex-1 overflow-x-auto overflow-y-hidden custom-scrollbar">
      <div class="flex h-full pb-4 gap-6 items-start w-max min-w-full">
        <section 
          v-for="board in boards" 
          :key="board.pid" 
          class="h-full flex flex-col bg-brand-container/50 backdrop-blur-md p-5 rounded-3xl w-80 shrink-0 border border-brand-primary/5 shadow-xl"
          @click.stop
        >
          <!-- Board Header -->
          <div class="flex justify-between items-center mb-6 px-2 shrink-0">
            <h3 
              class="text-lg font-bold text-brand-primary/80 cursor-pointer hover:text-brand-primary transition-all truncate"
              @click.stop="openEditBoard(board)"
            >
              {{ board.title }}
            </h3>
            <button 
              @click.stop="openCreateTask(board.pid)" 
              class="w-8 h-8 flex items-center justify-center rounded-xl bg-brand-primary/10 text-brand-primary hover:bg-brand-primary hover:text-brand-container transition-all"
            >
              <span class="text-xl font-light">+</span>
            </button>
          </div>

          <!-- Tasks -->
          <draggable 
            v-model="board.todos" 
            group="todos" 
            item-key="pid" 
            @change="(e: any) => handleMove(e, board.pid)" 
            class="flex-1 space-y-3 overflow-y-auto custom-scrollbar-y pr-1 pb-4" 
            ghost-class="opacity-10" 
            drag-class="opacity-100 scale-105"
          >
            <template #item="{ element: todo }">
              <article 
                @click.stop="openEditTask(todo)" 
                class="relative bg-brand-background/40 p-4 rounded-2xl border border-white/5 hover:border-brand-primary/30 transition-all cursor-pointer shadow-md group/task"
              >
                <div class="pr-6">
                  <p class="text-sm font-bold text-brand-text group-hover/task:text-brand-primary transition-colors leading-tight">{{ todo.title }}</p>
                  <p v-if="todo.details" class="text-[10px] text-brand-text-muted mt-2 line-clamp-2 leading-relaxed">{{ todo.details }}</p>
                </div>

                <button 
                  @click.stop="deleteTask(todo)" 
                  class="absolute top-3 right-3 p-1.5 rounded-lg bg-red-500/10 text-red-500 opacity-0 group-hover/task:opacity-100 transition-all hover:bg-red-500 hover:text-white"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </article>
            </template>

            <template #footer>
              <div v-if="!board.todos?.length" class="h-32 flex items-center justify-center rounded-2xl border-2 border-dashed border-white/5 text-[10px] text-brand-text-muted uppercase tracking-widest font-bold opacity-20">
                Empty
              </div>
            </template>
          </draggable>
        </section>

        <!-- Layout Spacer: matches SideDrawer width -->
        <div 
          class="transition-all duration-500 ease-[cubic-bezier(0.16,1,0.3,1)] shrink-0"
          :class="isDrawerOpen ? 'w-[450px]' : 'w-0'"
        ></div>
      </div>
    </div>

    <!-- SideDrawer: Fixed to viewport, handles its own transition -->
    <SideDrawer 
      :is-open="isDrawerOpen" 
      v-bind="drawerMeta"
      @close="closeDrawer"
    >
      <component 
        :is="activeDrawer" 
        ref="drawerRef"
        v-bind="drawerData"
        @save="activeDrawer === TaskDrawer ? handleTaskSave($event) : handleBoardSave($event)"
      />

      <template #footer>
        <div class="flex gap-4">
          <button 
            @click="activeDrawer === TaskDrawer ? handleTaskSave(drawerRef.form) : handleBoardSave(drawerRef.form)" 
            class="flex-1 bg-brand-primary text-brand-container py-5 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-brand-primary/90 transition-all shadow-lg"
          >
            Save Changes
          </button>
          
          <button 
            v-if="selectedTodo"
            @click="deleteTask(selectedTodo)"
            class="w-16 h-16 rounded-2xl bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white transition-all flex items-center justify-center"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
</style>
