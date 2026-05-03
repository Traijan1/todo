<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Board } from "../../../api/models";

const props = defineProps<{
  board: Board;
}>();

const emit = defineEmits<{
  (e: 'save', data: { title: string }): void;
}>();

const form = ref({
  title: props.board.title
});

watch(() => props.board, (newBoard) => {
  form.value = { title: newBoard.title };
}, { immediate: true });

defineExpose({ form });
</script>

<template>
  <div class="space-y-10">
    <div class="space-y-1">
      <label class="brand-label">Board Designation</label>
      <input
        v-model="form.title"
        type="text"
        class="brand-input"
        @keydown.enter="emit('save', form)"
      />
    </div>
  </div>
</template>
