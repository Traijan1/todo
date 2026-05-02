<script setup lang="ts">
import { useProjectStore } from "../../stores/projects";
import { useRouter } from "vue-router";
import { useForm } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import * as zod from "zod";

const projectStore = useProjectStore();
const router = useRouter();

const schema = toTypedSchema(
  zod.object({
    title: zod
      .string()
      .min(3, "Title must be at least 3 characters")
      .max(50, "Title is too long"),
    description: zod
      .string()
      .max(500, "Description must be under 500 characters")
      .optional()
      .or(zod.literal("")),
  })
);

const { defineField, handleSubmit, errors, isSubmitting } = useForm({
  validationSchema: schema,
  initialValues: {
    title: "",
    description: "",
  },
});

const [title, titleProps] = defineField("title");
const [description, descriptionProps] = defineField("description");

const onSubmit = handleSubmit(async (values) => {
  try {
    await projectStore.createProject({
      title: values.title,
      description: values.description || undefined,
    });
    router.push("/");
  } catch (err) {
    // Error is handled in the store
  }
});
</script>

<template>
  <div class="max-w-2xl mx-auto w-full py-8">
    <header class="mb-10 text-center">
      <h2 class="text-4xl font-black text-brand-primary tracking-tight mb-2">New Project</h2>
      <p class="text-brand-text-muted">Set up a new workspace for your tasks</p>
    </header>

    <div class="bg-brand-container p-10 rounded-[2.5rem] shadow-2xl border border-brand-primary/10">
      <form @submit="onSubmit" class="space-y-8">
        <div v-if="projectStore.error" class="bg-red-950/30 border border-red-900/50 text-red-400 px-6 py-4 rounded-2xl text-sm font-bold">
          {{ projectStore.error }}
        </div>

        <div class="space-y-6">
          <div>
            <label for="title" class="block text-xs font-bold uppercase tracking-widest text-brand-primary/60 mb-3 ml-1">
              Project Title
            </label>
            <input 
              v-bind="titleProps"
              v-model="title"
              id="title"
              type="text" 
              placeholder="e.g. My Awesome App" 
              class="w-full px-6 py-4 bg-brand-background/50 border rounded-2xl text-brand-text placeholder-brand-text-muted/30 focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 transition-all text-lg"
              :class="errors.title ? 'border-red-500/50' : 'border-brand-primary/10'"
            />
            <p v-if="errors.title" class="mt-2 ml-1 text-sm font-bold text-red-400/80">{{ errors.title }}</p>
          </div>

          <!-- Description Input -->
          <div>
            <label for="description" class="block text-xs font-bold uppercase tracking-widest text-brand-primary/60 mb-3 ml-1">
              Description (Optional)
            </label>
            <textarea 
              v-bind="descriptionProps"
              v-model="description"
              id="description"
              placeholder="What is this project about?" 
              rows="4"
              class="w-full px-6 py-4 bg-brand-background/50 border rounded-2xl text-brand-text placeholder-brand-text-muted/30 focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 transition-all resize-none"
              :class="errors.description ? 'border-red-500/50' : 'border-brand-primary/10'"
            ></textarea>
            <p v-if="errors.description" class="mt-2 ml-1 text-sm font-bold text-red-400/80">{{ errors.description }}</p>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-4 pt-4">
          <button 
            type="button"
            @click="router.back()"
            class="flex-1 py-2.5 px-6 rounded-xl font-bold text-brand-text-muted hover:bg-brand-background/50 transition-all"
          >
            Cancel
          </button>
          
          <button 
            type="submit"
            :disabled="isSubmitting || projectStore.loading"
            class="flex-[2] py-2.5 px-6 bg-brand-primary text-brand-container rounded-xl font-black text-base hover:bg-brand-primary/90 shadow-lg shadow-brand-primary/10 transition-all active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span v-if="isSubmitting || projectStore.loading">Creating...</span>
            <span v-else>Create Project</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
