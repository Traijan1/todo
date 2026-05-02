<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useBoardStore } from "../../stores/boards";
import { storeToRefs } from "pinia";
import { useProjectStore } from "../../stores/projects";

const props = defineProps<{
  pid: string;
}>();

const projectStore = useProjectStore();
const boardStore = useBoardStore();
const { boards, loading, error } = storeToRefs(boardStore);

const project = computed(() => projectStore.projects.find((project) => project.pid === props.pid));

onMounted(async () => {
  console.log("SingleProjectView mounted with pid:", props.pid);
  if(!project.value) await projectStore.fetchProjects();

  if (props.pid) {
    await boardStore.fetchBoards(props.pid);
  } else {
    console.error("No pid provided to SingleProjectView");
  }
});
</script>

<template>
  <div class="p-6">
    <h2 class="text-3xl text-brand-primary font-bold mb-4">{{ project?.title }}</h2>

    <div v-if="loading" class="text-purple-400">Loading boards...</div>
    <div v-else-if="error" class="text-red-400">{{ error }}</div>
    <div v-else>
      <div class="flex flex-row gap-x-8">
        <div v-for="board in boards" v-key="board.pid" class="bg-purple-900/20 p-4 rounded-md w-100 p-4 text-purple-200 border border-purple-800/30">
          <h2 class="text-xl">{{ board.title }}</h2>
          <hr class="border-brand-muted mt-2" />
        </div>
      </div>
    </div>
  </div>
</template>
