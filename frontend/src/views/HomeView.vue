<script setup lang="ts">
import LocoSplash from "../LocoSplash.vue";
import Card from "../components/Card.vue";
import DraggableDemo from "../components/DraggableDemo.vue";
import { useAuthStore } from "../stores/auth";
import ProjectIndex from "./projects/Index.vue";

const auth = useAuthStore();
</script>

<template>
  <div class="max-w-6xl mx-auto w-full text-brand-text">
    <header class="mb-8 flex justify-between items-center">
      <div>
        <h2 class="text-3xl font-bold text-brand-primary">Dashboard</h2>
        <p class="text-brand-text-muted">Welcome back, {{ auth.user?.name || "Guest" }}!</p>
      </div>
      <div>
        <RouterLink to="/projects/new" class="bg-brand-primary text-brand-container px-6 py-2.5 rounded-xl font-bold hover:bg-brand-primary/90 transition-all shadow-lg shadow-brand-primary/10 active:scale-95"> New Project </RouterLink>
      </div>
    </header>

    <div class="gap-8">
      <Card title="Projects" :no-background="true">
        <ProjectIndex />
      </Card>
    </div>
  </div>
</template>
