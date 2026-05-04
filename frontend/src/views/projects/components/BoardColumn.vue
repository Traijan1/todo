<script setup lang="ts">
import { computed, ref } from "vue";
import type { Board, Todo } from "../../../api/models";
import draggable from "vuedraggable";
import TaskCard from "./TaskCard.vue";

const props = defineProps<{
  board: Board;
}>();

const emit = defineEmits<{
  'update:todos': [value: Todo[]];
  'edit-board': [];
  'create-task': [];
  'edit-task': [todo: Todo];
  'delete-task': [todo: Todo];
  'change': [event: any];
}>();

const isOpen = ref(true);

const localTodos = computed({
  get: () => props.board.todos,
  set: (val) => emit('update:todos', val)
});
</script>

<template>
  <section 
    class="flex flex-col bg-brand-container/40 backdrop-blur-md rounded-2xl w-full border border-brand-primary/10 shadow-sm overflow-hidden transition-all duration-300"
    :class="{ 'pb-4': isOpen }"
    @click.stop
  >
    <!-- Board Header -->
    <div 
      class="flex justify-between items-center p-3 px-5 shrink-0 cursor-pointer hover:bg-white/5 transition-colors"
      @click="isOpen = !isOpen"
    >
      <div class="flex items-center gap-3">
        <div 
          class="w-6 h-6 flex items-center justify-center rounded-lg bg-brand-primary/10 text-brand-primary transition-transform duration-300"
          :class="{ 'rotate-180': !isOpen }"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>
        
        <div class="flex items-center gap-3">
          <h3 
            class="text-sm font-bold text-brand-primary/90 hover:text-brand-primary transition-all truncate"
            @click.stop="emit('edit-board')"
          >
            {{ board.title }}
          </h3>
          <span class="text-[9px] bg-brand-primary/10 text-brand-primary px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
            {{ board.todos?.length || 0 }}
          </span>
        </div>
      </div>

      <button 
        @click.stop="emit('create-task')" 
        class="group flex items-center gap-2 pl-1 pr-3 py-1 rounded-xl bg-brand-primary/10 text-brand-primary hover:bg-brand-primary hover:text-brand-container transition-all"
      >
        <span class="w-6 h-6 flex items-center justify-center rounded-lg bg-brand-primary/20 group-hover:bg-white/20 transition-colors text-lg font-light">+</span>
        <span class="text-[9px] font-bold uppercase tracking-widest">New</span>
      </button>
    </div>

    <!-- Tasks -->
    <div v-show="isOpen" class="px-5">
      <draggable 
        v-model="localTodos"
        group="todos" 
        item-key="pid" 
        @change="emit('change', $event)" 
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-3" 
        ghost-class="opacity-50"
      >
        <template #item="{ element: todo }">
          <TaskCard 
            :todo="todo" 
            @click.stop="emit('edit-task', todo)" 
            @delete="emit('delete-task', todo)" 
          />
        </template>

        <template #footer>
          <div v-if="!board.todos?.length" class="col-span-full h-24 flex items-center justify-center rounded-3xl border-2 border-dashed border-white/5 text-[10px] text-brand-text-muted uppercase tracking-widest font-bold opacity-20">
            Empty Board
          </div>
        </template>
      </draggable>
    </div>
  </section>
</template>
