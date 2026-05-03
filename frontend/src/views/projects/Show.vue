<script setup lang="ts">
import { computed, onMounted, ref, nextTick } from "vue";
import { useBoardStore } from "../../stores/boards";
import { storeToRefs } from "pinia";
import { useProjectStore } from "../../stores/projects";
import { useTodoStore } from "../../stores/todos";
import SideDrawer from "../../components/SideDrawer.vue";
import type { Todo } from "../../api/models";
import draggable from "vuedraggable";

const props = defineProps<{
  pid: string;
}>();

const projectStore = useProjectStore();
const boardStore = useBoardStore();
const todoStore = useTodoStore();
const { boards, loading, error } = storeToRefs(boardStore);

const project = computed(() => projectStore.projects.find((project) => project.pid === props.pid));

// --- Drawer & Task Logic ---
const isDrawerOpen = ref(false);
const selectedBoardPid = ref<string | null>(null);
const selectedTodo = ref<Todo | null>(null);
const taskForm = ref({ title: "", description: "" });
const titleInputRef = ref<HTMLInputElement | null>(null);

const drawerTitle = computed(() => (selectedTodo.value ? "Edit Task" : "New Task"));
const drawerSubtitle = computed(() => {
  if (selectedTodo.value) return `Task PID: ${selectedTodo.value.pid}`;
  const board = boards.value.find((b) => b.pid === selectedBoardPid.value);
  return board ? `Adding to board: ${board.title}` : "";
});

const openCreateDrawer = async (boardPid: string) => {
  selectedTodo.value = null;
  selectedBoardPid.value = boardPid;
  taskForm.value = { title: "", description: "" };
  isDrawerOpen.value = true;
  await nextTick();
  titleInputRef.value?.focus();
};

const openEditDrawer = async (todo: Todo) => {
  selectedTodo.value = todo;
  selectedBoardPid.value = null;
  taskForm.value = {
    title: todo.title,
    description: todo.details || "",
  };
  isDrawerOpen.value = true;
  await nextTick();
  titleInputRef.value?.focus();
};

const saveTask = async () => {
  console.log("Erstellt");

  if (!taskForm.value.title.trim()) return;

  try {
    if (selectedTodo.value) {
      await todoStore.updateTodo(selectedTodo.value.pid, {
        title: taskForm.value.title,
        details: taskForm.value.description,
      });

      await boardStore.fetchBoards(props.pid);
    } else if (selectedBoardPid.value) {
      await todoStore.createTodo(selectedBoardPid.value, {
        title: taskForm.value.title,
        details: taskForm.value.description,
      });

      await boardStore.fetchBoards(props.pid);
    }
    isDrawerOpen.value = false;
  } catch (err) {
    console.error("Save failed", err);
  }
};

const deleteTask = async (todo: Todo) => {
  if (confirm(`Are you sure you want to delete "${todo.title}"?`)) {
    await todoStore.deleteTodo(todo.pid);
    await boardStore.fetchBoards(props.pid);
  }
};

const handleMove = async (event: any, targetBoardPid: string) => {
  if (event.added) {
    const todo = event.added.element;
    try {
      await todoStore.updateTodo(todo.pid, { board_pid: targetBoardPid });
      await boardStore.reorderTodos(targetBoardPid);
    } catch (err) {
      await boardStore.fetchBoards(props.pid);
    }
  } else if (event.moved) {
    const boardPid = event.moved.element.board_pid;
    await boardStore.reorderTodos(boardPid);
  }
};
// ----------------------------

onMounted(async () => {
  if (!project.value) await projectStore.fetchProjects();
  if (props.pid) {
    await boardStore.fetchBoards(props.pid);
  }
});
</script>

<template>
  <div class="p-6 relative min-h-screen">
    <header class="mb-8">
      <h2 class="text-3xl text-brand-primary font-black tracking-tight">{{ project?.title || "Loading..." }}</h2>
      <p class="text-brand-text-muted text-sm mt-1">Project Overview & Boards</p>
    </header>

    <!-- Boards List -->
    <!-- <div v-if="loading" class="animate-pulse text-brand-primary text-sm font-bold uppercase tracking-widest">Syncing boards...</div> -->
    <div class="flex flex-row gap-6 overflow-x-auto pb-6 custom-scrollbar">
      <div v-for="board in boards" :key="board.pid" class="h-min bg-brand-container/50 backdrop-blur-sm p-5 rounded-[2.5rem] w-80 shrink-0 border border-brand-primary/10 flex flex-col shadow-sm">
        <div class="flex justify-between items-center mb-6 px-2">
          <h3 class="text-lg font-bold text-brand-primary/90">{{ board.title }}</h3>
          <button @click="openCreateDrawer(board.pid)" class="w-8 h-8 flex items-center justify-center rounded-full bg-brand-primary/10 text-brand-primary hover:bg-brand-primary hover:text-brand-container transition-all">
            <span class="text-xl font-light">+</span>
          </button>
        </div>

        <!-- Todo List Area with Drag'n'Drop -->
        <draggable v-model="board.todos" group="todos" item-key="pid" @change="(e) => handleMove(e, board.pid)" class="flex-1 space-y-3 min-h-[50px] mb-4 overflow-y-auto custom-scrollbar-y pr-1" ghost-class="opacity-50" drag-class="rotate-2">
          <template #item="{ element: todo }">
            <div @click="openEditDrawer(todo)" class="relative bg-brand-background/60 p-4 rounded-2xl border border-white/5 hover:border-brand-primary/20 transition-all cursor-pointer shadow-sm group/todo">
              <div class="pr-6">
                <p class="text-sm font-bold text-brand-text group-hover/todo:text-brand-primary transition-colors">{{ todo.title }}</p>
                <p v-if="todo.details" class="text-[10px] text-brand-text-muted mt-1 line-clamp-2 leading-relaxed">{{ todo.details }}</p>
              </div>

              <!-- Quick Delete Icon -->
              <button @click.stop="deleteTask(todo)" class="absolute top-3 right-3 p-1.5 rounded-lg bg-red-500/10 text-red-500 opacity-0 group-hover/todo:opacity-100 transition-all hover:bg-red-500 hover:text-white" title="Delete Task">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </template>

          <template #footer>
            <div v-if="!board.todos?.length" class="p-4 rounded-2xl bg-brand-background/40 border border-white/5 text-[10px] text-brand-text-muted italic text-center uppercase tracking-widest font-bold py-8 opacity-50 pointer-events-none">
              No Tasks
            </div>
          </template>
        </draggable>
      </div>
    </div>

    <!-- Global Side Drawer Component -->
    <SideDrawer :is-open="isDrawerOpen" :title="drawerTitle" :subtitle="drawerSubtitle" @close="isDrawerOpen = false">
      <div class="space-y-8">
        <div class="space-y-2">
          <label class="text-[10px] font-black uppercase tracking-[0.2em] text-brand-primary/50 ml-1">Task Title</label>
          <input
            ref="titleInputRef"
            v-model="taskForm.title"
            type="text"
            placeholder="What needs to be done?"
            class="w-full bg-brand-background/50 border border-brand-primary/10 rounded-2xl px-5 py-4 text-brand-text placeholder-white/10 focus:outline-none focus:ring-2 focus:ring-brand-primary/20 transition-all text-lg font-bold"
            @keydown.enter="saveTask"
          />
        </div>

        <div class="space-y-2">
          <label class="text-[10px] font-black uppercase tracking-[0.2em] text-brand-primary/50 ml-1">Description</label>
          <textarea
            v-model="taskForm.description"
            placeholder="Add more details about this task..."
            rows="6"
            class="w-full bg-brand-background/50 border border-brand-primary/10 rounded-2xl px-5 py-4 text-brand-text placeholder-white/10 focus:outline-none focus:ring-2 focus:ring-brand-primary/20 transition-all resize-none text-sm leading-relaxed"
          ></textarea>
        </div>
      </div>

      <template #footer>
        <div class="flex gap-4">
          <!-- Delete button inside drawer (only if editing) -->
          <button
            v-if="selectedTodo"
            @click="deleteTask(selectedTodo)"
            class="px-5 rounded-2xl bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white transition-all flex items-center justify-center shadow-lg shadow-red-500/5"
            title="Delete Task"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>

          <button @click="saveTask" class="flex-1 bg-brand-primary text-brand-container py-4 rounded-2xl font-black uppercase tracking-widest text-xs hover:bg-brand-primary/90 transition-all shadow-lg shadow-brand-primary/10">
            {{ selectedTodo ? "Update Task" : "Create Task" }}
          </button>

          <button @click="isDrawerOpen = false" class="px-6 py-4 rounded-2xl font-bold text-xs text-brand-text-muted hover:bg-white/5 transition-all">Cancel</button>
        </div>
      </template>
    </SideDrawer>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  height: 8px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(var(--brand-primary-rgb), 0.05);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.1);
  border-radius: 10px;
}

.custom-scrollbar-y::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar-y::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar-y::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.05);
  border-radius: 10px;
}
</style>
