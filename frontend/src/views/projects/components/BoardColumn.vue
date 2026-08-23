<script setup lang="ts">
import { computed, ref } from "vue";
import type { Board, Todo } from "../../../api/models";
import draggable from "vuedraggable";
import TaskCard from "./TaskCard.vue";

const props = defineProps<{
  board: Board;
  filterTags?: string[];
  filterSearch?: string;
  open?: boolean;
}>();

const emit = defineEmits<{
  "update:todos": [value: Todo[]];
  "edit-board": [];
  "toggle": [];
  "create-task": [];
  "edit-task": [todo: Todo];
  "delete-task": [todo: Todo];
  change: [event: any];
}>();

const internalOpen = ref(true);
const isOpen = computed({
  get: () => props.open ?? internalOpen.value,
  set: (v) => { internalOpen.value = v; },
});

const localTodos = computed({
  get: () => props.board.todos,
  set: (val) => emit("update:todos", val),
});

const isVisible = (todo: Todo) => {
  const tagMatch = !props.filterTags?.length || (todo.tags?.some((t) => props.filterTags!.includes(t.pid)) ?? false);
  const searchMatch = !props.filterSearch?.trim() || todo.title.toLowerCase().includes(props.filterSearch.toLowerCase());
  return tagMatch && searchMatch;
};

const visibleCount = computed(() =>
  props.board.todos?.filter((t) => isVisible(t)).length || 0
);

const isFiltering = computed(() =>
  !!(props.filterTags?.length || props.filterSearch?.trim())
);
</script>

<template>
  <section class="flex flex-col bg-brand-container/40 backdrop-blur-md rounded-2xl w-full border border-brand-primary/10 shadow-sm overflow-hidden transition-all duration-300" :class="{ 'pb-4': isOpen }" @click.stop>
    <!-- Board Header -->
    <div class="flex mb-4 justify-between items-center p-3 px-5 shrink-0 cursor-pointer hover:bg-white/5 transition-colors" @click="emit('toggle'); isOpen = !isOpen">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 flex items-center justify-center rounded-lg bg-brand-primary/10 text-brand-primary transition-transform duration-300" :class="{ 'rotate-180': !isOpen }">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>

        <div class="flex items-center gap-3">
          <h3 class="text-sm font-bold text-brand-primary/90 hover:text-brand-primary transition-all truncate" @click.stop="emit('edit-board')">
            {{ board.title }}
          </h3>
          <span class="text-[9px] bg-brand-primary/10 text-brand-primary px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
            {{ visibleCount }}
          </span>
        </div>
      </div>

      <button @click.stop="emit('create-task')" class="group flex items-center gap-2 pl-1 pr-3 py-1 rounded-xl bg-brand-primary/10 text-brand-primary hover:bg-brand-primary hover:text-brand-container transition-all">
        <span class="w-6 h-6 flex items-center justify-center rounded-lg bg-brand-primary/20 group-hover:bg-white/20 transition-colors text-lg font-light">+</span>
        <span class="text-[9px] font-bold uppercase tracking-widest">New</span>
      </button>
    </div>

    <!-- Tasks -->
    <div v-show="isOpen" class="px-5">
      <draggable v-model="localTodos" group="todos" item-key="pid" :delay="150" :delay-on-touch-only="true" @change="emit('change', $event)" class="flex flex-col gap-2" ghost-class="opacity-50">
        <template #item="{ element: todo }">
          <div v-show="isVisible(todo)">
            <TaskCard :todo="todo" @click.stop="emit('edit-task', todo)" @delete="emit('delete-task', todo)" />
          </div>
        </template>

        <template #footer>
          <!-- Board has no todos at all -->
          <div v-if="!board.todos?.length" class="flex flex-col items-center justify-center py-8 gap-3">
            <div class="w-10 h-10 rounded-2xl bg-brand-primary/8 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-brand-primary/30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
            </div>
            <p class="text-[10px] font-bold uppercase tracking-widest text-brand-text-muted/25">No tasks yet</p>
            <button
              @click.stop="emit('create-task')"
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-brand-primary/10 text-brand-primary/60 hover:bg-brand-primary/20 hover:text-brand-primary transition-all text-[10px] font-bold uppercase tracking-widest"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
              </svg>
              Add Task
            </button>
          </div>
          <!-- Has todos but all filtered out -->
          <div v-else-if="isFiltering && visibleCount === 0" class="flex flex-col items-center justify-center py-8 gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-brand-text-muted/20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <p class="text-[10px] font-bold uppercase tracking-widest text-brand-text-muted/20">No matches</p>
          </div>
        </template>
      </draggable>
    </div>
  </section>
</template>
