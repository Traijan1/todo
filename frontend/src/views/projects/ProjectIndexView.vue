import { useAuthStore } from '../../stores/auth';
<script setup lang="ts">
import { onMounted, Ref, ref } from "vue";
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
    <div v-for="project in projects" class="text-brand-primary">{{ project.title }}</div>
  </div>
</template>
