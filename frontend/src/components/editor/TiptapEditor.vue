<script setup lang="ts">
import { useEditor, EditorContent } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import { watch, onBeforeUnmount } from "vue";
import { Markdown } from "@tiptap/markdown";

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
}>();

const emit = defineEmits(["update:modelValue"]);

const editor = useEditor({
  content: props.modelValue,
  extensions: [
    StarterKit.configure({
      heading: {
        levels: [1, 2, 3],
      },
    }),
    Markdown,
  ],
  editorProps: {
    attributes: {
      class: "prose prose-invert prose-sm max-w-none focus:outline-none min-h-[200px] text-brand-text font-medium p-4",
    },
  },
  onUpdate: ({ editor }) => {
    emit("update:modelValue", editor.getHTML());
  },
});

watch(
  () => props.modelValue,
  (value) => {
    const isSame = editor.value?.getHTML() === value;
    if (isSame) return;
    editor.value?.commands.setContent(value, false);
  },
);

onBeforeUnmount(() => {
  editor.value?.destroy();
});
</script>

<template>
  <div v-if="editor" class="brand-textarea p-0 overflow-hidden flex flex-col">
    <div class="flex flex-wrap gap-1 p-2 border-b border-white/5 bg-white/5">
      <button
        @click="editor.chain().focus().toggleBold().run()"
        :disabled="!editor.can().chain().focus().toggleBold().run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('bold') }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted"
        title="Bold"
      >
        <span class="font-bold">B</span>
      </button>
      <button
        @click="editor.chain().focus().toggleItalic().run()"
        :disabled="!editor.can().chain().focus().toggleItalic().run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('italic') }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted"
        title="Italic"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="19" y1="4" x2="10" y2="4" />
          <line x1="14" y1="20" x2="5" y2="20" />
          <line x1="15" y1="4" x2="9" y2="20" />
        </svg>
      </button>

      <div class="w-px h-4 bg-white/10 mx-1 self-center"></div>

      <button
        @click="editor.chain().focus().toggleHeading({ level: 1 }).run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('heading', { level: 1 }) }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted font-bold text-xs"
        title="Header 1"
      >
        H1
      </button>

      <button
        @click="editor.chain().focus().toggleHeading({ level: 2 }).run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('heading', { level: 2 }) }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted font-bold text-xs"
        title="Header 2"
      >
        H2
      </button>

      <button
        @click="editor.chain().focus().toggleHeading({ level: 3 }).run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('heading', { level: 3 }) }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted font-bold text-xs"
        title="Header 3"
      >
        H3
      </button>

      <div class="w-px h-4 bg-white/10 mx-1 self-center"></div>

      <button
        @click="editor.chain().focus().toggleBulletList().run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('bulletList') }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted"
        title="Bullet List"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="8" y1="6" x2="21" y2="6" />
          <line x1="8" y1="12" x2="21" y2="12" />
          <line x1="8" y1="18" x2="21" y2="18" />
          <line x1="3" y1="6" x2="3.01" y2="6" />
          <line x1="3" y1="12" x2="3.01" y2="12" />
          <line x1="3" y1="18" x2="3.01" y2="18" />
        </svg>
      </button>
      <button
        @click="editor.chain().focus().toggleOrderedList().run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('orderedList') }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted"
        title="Ordered List"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="10" y1="6" x2="21" y2="6" />
          <line x1="10" y1="12" x2="21" y2="12" />
          <line x1="10" y1="18" x2="21" y2="18" />
          <path d="M4 6h1v4" />
          <path d="M4 10h2" />
          <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" />
        </svg>
      </button>
      <div class="w-px h-4 bg-white/10 mx-1 self-center"></div>
      <button
        @click="editor.chain().focus().toggleCodeBlock().run()"
        :class="{ 'bg-brand-primary/20 text-brand-primary': editor.isActive('codeBlock') }"
        class="p-1.5 rounded hover:bg-white/10 transition-colors text-brand-text-muted"
        title="Code Block"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="16 18 22 12 16 6" />
          <polyline points="8 6 2 12 8 18" />
        </svg>
      </button>
    </div>

    <editor-content :editor="editor" />
  </div>
</template>

<style>
/* Tiptap styles */
.prose p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  float: left;
  color: #adb5bd;
  pointer-events: none;
  height: 0;
}

.prose :where(h1, h2, h3, h4, h5, h6, strong):not(:where([class~="not-prose"], [class~="not-prose"] *)) {
  color: var(--color-brand-text) !important;
  font-weight: 800 !important;
}

.prose ul {
  list-style-type: disc;
  padding-left: 1.5rem;
}

.prose ol {
  list-style-type: decimal;
  padding-left: 1.5rem;
}

.prose code {
  background: rgba(255, 255, 255, 0.1);
  padding: 0.2rem 0.4rem;
  border-radius: 0.25rem;
}

.prose pre {
  background: #1a1a1a;
  padding: 1rem;
  border-radius: 0.5rem;
  margin: 1rem 0;
}
</style>
