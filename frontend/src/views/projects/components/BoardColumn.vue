<script setup lang="ts">
import { computed } from "vue";
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

const localTodos = computed({
  get: () => props.board.todos,
  set: (val) => emit('update:todos', val)
});
</script>

<template>
  <section 
    class="h-full flex flex-col bg-brand-container/50 backdrop-blur-md p-5 rounded-3xl w-80 shrink-0 border border-brand-primary/5 shadow-xl"
    @click.stop
  >
    <!-- Board Header -->
    <div class="flex justify-between items-center mb-6 px-2 shrink-0">
      <h3 
        class="text-lg font-bold text-brand-primary/80 cursor-pointer hover:text-brand-primary transition-all truncate"
        @click.stop="emit('edit-board')"
      >
        {{ board.title }}
      </h3>
      <button 
        @click.stop="emit('create-task')" 
        class="w-8 h-8 flex items-center justify-center rounded-xl bg-brand-primary/10 text-brand-primary hover:bg-brand-primary hover:text-brand-container transition-all"
      >
        <span class="text-xl font-light">+</span>
      </button>
    </div>

    <!-- Tasks -->
    <draggable 
      v-model="localTodos"
      group="todos" 
      item-key="pid" 
      @change="emit('change', $event)" 
      class="flex-1 space-y-3 overflow-y-auto custom-scrollbar-y pr-1 pb-4" 
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
        <div v-if="!board.todos?.length" class="h-32 flex items-center justify-center rounded-2xl border-2 border-dashed border-white/5 text-[10px] text-brand-text-muted uppercase tracking-widest font-bold opacity-20">
          Empty
        </div>
      </template>
    </draggable>
  </section>
</template>
