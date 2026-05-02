<script setup lang="ts">
import { onMounted } from "vue";
import { useProjectStore } from "../../stores/projects";
import { storeToRefs } from "pinia";

const projectStore = useProjectStore();
const { projects } = storeToRefs(projectStore);

onMounted(async () => {
  await projectStore.fetchProjects();
});
</script>

<template>
  <div>
    <div v-for="project in projects" :key="project.pid" class="text-brand-primary">
      <RouterLink :to="{ name: 'project-detail', params: { pid: project.pid } }">{{ project.title }}</RouterLink>
    </div>
  </div>
</template>
