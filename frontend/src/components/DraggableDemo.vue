<script setup lang="ts">
import { ref } from "vue";
import draggable from "vuedraggable";

const list1 = ref([
  { id: 1, name: "Task 1" },
  { id: 2, name: "Task 2" },
  { id: 3, name: "Task 3" },
]);

const list2 = ref([
  { id: 4, name: "Task 4" },
  { id: 5, name: "Task 5" },
]);
</script>

<template>
  <div class="drag-demo">
    <div class="bg-gray-800 text-white w-50 p-4 rounded-sm">
      <h3>Backlog</h3>
      <draggable v-model="list1" group="tasks" item-key="id" class="drag-list">
        <template #item="{ element }">
          <div class="bg-gray-700 p-2 rounded-sm my-2 text-gray-200">{{ element.name }}</div>
        </template>
      </draggable>
    </div>

    <div class="bg-gray-800 text-white w-50 p-4 rounded-sm">
      <h3>In Progress</h3>
      <draggable v-model="list2" group="tasks" item-key="id" class="drag-list">
        <template #item="{ element }">
          <div class="bg-gray-700 p-2 rounded-sm my-2 text-gray-200">{{ element.name }}</div>
        </template>
      </draggable>
    </div>

    {{ list1 }} <br />
    {{ list2 }}
  </div>
</template>

<style scoped>
.drag-demo {
  display: flex;
  gap: 20px;
  padding: 20px;
  justify-content: center;
}

.drag-item:active {
  cursor: grabbing;
}
</style>
