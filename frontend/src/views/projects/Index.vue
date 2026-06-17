<script setup lang="ts">
import { onMounted } from "vue";
import { useProjectStore } from "../../stores/projects";
import { storeToRefs } from "pinia";

const projectStore = useProjectStore();
const { projects } = storeToRefs(projectStore);

const formatDate = (dateStr: string) =>
  new Date(dateStr).toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });

onMounted(async () => {
  await projectStore.fetchProjects();
});
</script>

<template>
  <!-- Skeleton -->
  <div v-if="projectStore.loading && !projects.length" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
    <div v-for="i in 3" :key="i" class="h-36 rounded-2xl bg-brand-primary/5 animate-pulse" />
  </div>

  <!-- Empty state -->
  <div v-else-if="!projects.length" class="flex flex-col items-center justify-center py-24 text-center">
    <div class="w-14 h-14 rounded-2xl bg-brand-primary/10 flex items-center justify-center mb-5">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-brand-primary/50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
    </div>
    <h3 class="text-base font-black text-brand-text tracking-tight mb-1">No projects yet</h3>
    <p class="text-sm text-brand-text-muted mb-6">Create your first project to get started</p>
    <RouterLink
      to="/projects/new"
      class="flex items-center gap-2 bg-brand-primary text-brand-container px-5 py-2.5 rounded-xl font-bold text-sm hover:bg-brand-primary/90 transition-all shadow-lg shadow-brand-primary/10 active:scale-95"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
      </svg>
      New Project
    </RouterLink>
  </div>

  <!-- Project grid -->
  <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
    <RouterLink
      v-for="project in projects"
      :key="project.pid"
      :to="{ name: 'project-detail', params: { pid: project.pid } }"
      class="group flex flex-col p-5 rounded-2xl bg-brand-container border border-brand-primary/10 hover:border-brand-primary/25 transition-all duration-200 hover:shadow-lg hover:shadow-brand-primary/5 active:scale-[0.98]"
    >
      <!-- Top row: icon + date -->
      <div class="flex items-start justify-between mb-4">
        <div class="w-9 h-9 rounded-xl bg-brand-primary/15 flex items-center justify-center shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-brand-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <span class="text-[9px] font-bold uppercase tracking-widest text-brand-primary/30 mt-1">
          {{ formatDate(project.created_at) }}
        </span>
      </div>

      <!-- Title -->
      <h3 class="text-sm font-black text-brand-text tracking-tight mb-1 line-clamp-1">{{ project.title }}</h3>

      <!-- Description -->
      <p v-if="project.description" class="text-xs text-brand-text-muted line-clamp-2 flex-1 leading-relaxed">
        {{ project.description }}
      </p>
      <div v-else class="flex-1" />

      <!-- Footer CTA -->
      <div class="flex items-center gap-1 mt-4 text-[10px] font-bold uppercase tracking-widest text-brand-primary/40 group-hover:text-brand-primary transition-colors duration-200">
        Open
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 translate-x-0 group-hover:translate-x-0.5 transition-transform duration-200" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </RouterLink>
  </div>
</template>
