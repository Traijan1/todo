<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import type { Todo } from "../../../api/models";

const props = defineProps<{
  todo: Todo | null;
  boardPid: string | null;
}>();

const emit = defineEmits<{
  (e: "save", data: { title: string; description: string }): void;
}>();

const form = ref({
  title: props.todo?.title || "",
  description: props.todo?.details || "",
});

watch(
  () => props.todo,
  (newTodo) => {
    form.value = {
      title: newTodo?.title || "",
      description: newTodo?.details || "",
    };
  },
  { immediate: true },
);

const titleInputRef = ref<HTMLTextAreaElement | null>(null);

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

defineExpose({ 
  form, 
  focus: () => {
    titleInputRef.value?.focus();
    adjustHeight();
  } 
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

    <div class="space-y-2">
      <label class="brand-label">Description</label>
      <textarea v-model="form.description" placeholder="Map out the steps..." rows="12" class="brand-textarea h-auto min-h-[200px]"></textarea>
    </div>
  </div>
</template>
