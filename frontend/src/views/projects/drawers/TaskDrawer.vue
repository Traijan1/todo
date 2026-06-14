<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import type { Todo } from "../../../api/models";
import TiptapEditor from "../../../components/editor/TiptapEditor.vue";
import { useTagStore } from "../../../stores/tags";

const props = defineProps<{
  todo: Todo | null;
  boardPid: string | null;
}>();

const emit = defineEmits<{
  (e: "save", data: { title: string; description: string; tags: string[] }): void;
}>();

const tagStore = useTagStore();

const TAG_COLORS = [
  "#E0BBE4",
  "#B19CD9",
  "#6EE7B7",
  "#93C5FD",
  "#FCA5A5",
  "#FCD34D",
  "#F9A8D4",
  "#86EFAC",
];

const form = ref({
  title: props.todo?.title || "",
  description: props.todo?.details || "",
  tags: props.todo?.tags?.map((t) => t.pid) || [] as string[],
});

const showTagPicker = ref(false);
const newTagTitle = ref("");
const newTagColor = ref(TAG_COLORS[0]);

const titleInputRef = ref<HTMLTextAreaElement | null>(null);

watch(
  () => props.todo,
  (newTodo) => {
    form.value = {
      title: newTodo?.title || "",
      description: newTodo?.details || "",
      tags: newTodo?.tags?.map((t) => t.pid) || [],
    };
    showTagPicker.value = false;
  },
  { immediate: true },
);

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

const selectedTagObjects = computed(() =>
  tagStore.tags.filter((t) => form.value.tags.includes(t.pid)),
);

const toggleTag = (pid: string) => {
  const idx = form.value.tags.indexOf(pid);
  if (idx === -1) {
    form.value.tags.push(pid);
  } else {
    form.value.tags.splice(idx, 1);
  }
};

const createAndAddTag = async () => {
  const title = newTagTitle.value.trim();
  if (!title) return;
  const tag = await tagStore.createTag(title, newTagColor.value);
  form.value.tags.push(tag.pid);
  newTagTitle.value = "";
  newTagColor.value = TAG_COLORS[0];
};

const tagStyle = (color?: string) => {
  const c = color || "#E0BBE4";
  return {
    backgroundColor: `${c}22`,
    color: c,
    borderColor: `${c}44`,
  };
};

defineExpose({
  form,
  focus: () => {
    titleInputRef.value?.focus();
    adjustHeight();
  },
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

    <!-- Tags -->
    <div class="space-y-2">
      <label class="brand-label">Tags</label>
      <div class="relative">
        <div class="flex flex-wrap gap-1.5 items-center min-h-[28px]">
          <button
            v-for="tag in selectedTagObjects"
            :key="tag.pid"
            type="button"
            class="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border transition-all hover:opacity-70"
            :style="tagStyle(tag.color)"
            @click.stop="toggleTag(tag.pid)"
          >
            {{ tag.title }}
            <span class="opacity-50 text-xs leading-none">×</span>
          </button>

          <button
            type="button"
            class="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-brand-primary/10 text-brand-primary/60 hover:bg-brand-primary/20 hover:text-brand-primary transition-all border border-brand-primary/10"
            @click.stop="showTagPicker = !showTagPicker"
          >
            + Tag
          </button>
        </div>

        <!-- Backdrop -->
        <div
          v-if="showTagPicker"
          class="fixed inset-0 z-10"
          @click.stop="showTagPicker = false"
        />

        <!-- Picker dropdown -->
        <div
          v-if="showTagPicker"
          class="absolute top-full left-0 right-0 mt-2 z-20 bg-brand-container border border-brand-primary/20 rounded-xl p-3 shadow-2xl space-y-3"
          @click.stop
        >
          <!-- Existing tags -->
          <div v-if="tagStore.tags.length" class="flex flex-wrap gap-1.5">
            <button
              v-for="tag in tagStore.tags"
              :key="tag.pid"
              type="button"
              class="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border transition-all"
              :class="form.tags.includes(tag.pid) ? 'ring-2 ring-white/30' : 'opacity-60 hover:opacity-100'"
              :style="tagStyle(tag.color)"
              @click="toggleTag(tag.pid)"
            >
              <span v-if="form.tags.includes(tag.pid)" class="text-xs leading-none">✓</span>
              {{ tag.title }}
            </button>
          </div>

          <!-- Divider -->
          <div class="border-t border-brand-primary/10 pt-3 space-y-2">
            <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/30">Create New</p>
            <input
              v-model="newTagTitle"
              placeholder="Tag name..."
              class="brand-input text-xs"
              @keydown.enter.stop="createAndAddTag"
            />
            <div class="flex gap-1.5 flex-wrap">
              <button
                v-for="color in TAG_COLORS"
                :key="color"
                type="button"
                class="w-5 h-5 rounded-full border-2 transition-all hover:scale-110"
                :style="{ backgroundColor: color, borderColor: newTagColor === color ? 'white' : 'transparent' }"
                @click="newTagColor = color"
              />
            </div>
            <button
              type="button"
              :disabled="!newTagTitle.trim()"
              class="w-full text-[10px] font-bold uppercase tracking-widest py-1.5 rounded-lg bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
              @click="createAndAddTag"
            >
              Create Tag
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="space-y-2">
      <label class="brand-label">Description</label>
      <TiptapEditor v-model="form.description" placeholder="Map out the steps..." />
    </div>
  </div>
</template>
