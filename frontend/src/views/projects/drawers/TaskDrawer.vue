<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Todo } from "../../../api/models";

const props = defineProps<{
  todo: Todo | null;
  boardPid: string | null;
}>();

const emit = defineEmits<{
  (e: 'save', data: { title: string; description: string }): void;
}>();

const form = ref({
  title: props.todo?.title || "",
  description: props.todo?.details || ""
});

watch(() => props.todo, (newTodo) => {
  form.value = {
    title: newTodo?.title || "",
    description: newTodo?.details || ""
  };
}, { immediate: true });

const titleInputRef = ref<HTMLInputElement | null>(null);
defineExpose({ form, focus: () => titleInputRef.value?.focus() });
</script>

<template>
  <div class="space-y-10">
    <div class="space-y-1">
      <label class="brand-label">Task Identification</label>
      <input
        ref="titleInputRef"
        v-model="form.title"
        type="text"
        placeholder="Name your mission..."
        class="brand-input"
        @keydown.enter="emit('save', form)"
      />
    </div>

    <div class="space-y-1">
      <label class="brand-label">Strategic Details</label>
      <textarea
        v-model="form.description"
        placeholder="Map out the steps..."
        rows="8"
        class="brand-textarea"
      ></textarea>
    </div>
  </div>
</template>
