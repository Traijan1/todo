<script setup lang="ts">
import { useAuthStore } from "../stores/auth";
import ProjectIndex from "./projects/Index.vue";
import { ref, onMounted, computed } from "vue";
import api from "../api/client";

const auth = useAuthStore();

interface BoardCount { pid: string; title: string; count: number }
interface ProjectStats { pid: string; title: string; board_counts: BoardCount[]; total: number }
interface RecentTodo { pid: string; title: string; board_title: string; project_title: string; updated_at: string }
interface Stats { total: number; by_project: ProjectStats[]; recently_updated: RecentTodo[] }

const stats = ref<Stats | null>(null);

const formatRelative = (dateStr: string) => {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(dateStr).toLocaleDateString(undefined, { month: "short", day: "numeric" });
};

onMounted(async () => {
  try {
    const { data } = await api.get("/projects/stats");
    stats.value = data;
  } catch {}
});
</script>

<template>
  <div class="max-w-5xl mx-auto w-full text-brand-text">
    <header class="hidden lg:flex items-end justify-between mb-8">
      <div>
        <p class="text-[10px] font-black uppercase tracking-widest text-brand-primary/40 mb-1">Dashboard</p>
        <h2 class="text-2xl font-black text-brand-text tracking-tight">
          Hey, {{ auth.user?.name?.split(" ")[0] || "there" }}
        </h2>
      </div>
      <RouterLink
        to="/projects/new"
        class="flex items-center gap-2 bg-brand-primary text-brand-container px-5 py-2.5 rounded-xl font-bold text-sm hover:bg-brand-primary/90 transition-all shadow-lg shadow-brand-primary/10 active:scale-95"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
        </svg>
        New Project
      </RouterLink>
    </header>

    <!-- Stats dashboard -->
    <div v-if="stats && stats.total > 0" class="mb-8 grid grid-cols-1 lg:grid-cols-3 gap-4">
      <!-- Total tasks card -->
      <div class="lg:col-span-1 p-5 rounded-2xl bg-brand-container border border-brand-primary/10 flex flex-col justify-between gap-4">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40">Overview</p>
        <div>
          <p class="text-4xl font-black text-brand-text tracking-tight">{{ stats.total }}</p>
          <p class="text-xs text-brand-text-muted/50 font-medium mt-1">tasks across all projects</p>
        </div>
        <!-- Per-project mini bars -->
        <div class="space-y-2">
          <div v-for="proj in stats.by_project" :key="proj.pid" class="space-y-1">
            <div class="flex justify-between items-center">
              <span class="text-[10px] font-bold text-brand-text/60 truncate max-w-[130px]">{{ proj.title }}</span>
              <span class="text-[10px] font-black text-brand-primary/50">{{ proj.total }}</span>
            </div>
            <div class="flex gap-0.5 h-1.5 rounded-full overflow-hidden">
              <div
                v-for="board in proj.board_counts"
                :key="board.pid"
                :style="{ flex: board.count || 0 }"
                class="h-full bg-brand-primary/20 first:rounded-l-full last:rounded-r-full transition-all"
                :title="`${board.title}: ${board.count}`"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Recently updated -->
      <div class="lg:col-span-2 p-5 rounded-2xl bg-brand-container border border-brand-primary/10">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40 mb-4">Recently Updated</p>
        <div class="space-y-2">
          <RouterLink
            v-for="todo in stats.recently_updated"
            :key="todo.pid"
            :to="`/projects/${stats.by_project.find(p => p.title === todo.project_title)?.pid || ''}`"
            class="flex items-center gap-3 p-2.5 rounded-xl hover:bg-white/4 transition-colors group"
          >
            <div class="w-1.5 h-1.5 rounded-full bg-brand-primary/30 shrink-0 group-hover:bg-brand-primary/60 transition-colors" />
            <span class="flex-1 text-xs font-medium text-brand-text/80 truncate">{{ todo.title }}</span>
            <span class="text-[9px] font-bold text-brand-text-muted/30 shrink-0 px-2 py-0.5 rounded-lg bg-white/3">{{ todo.board_title }}</span>
            <span class="text-[9px] text-brand-text-muted/25 shrink-0 w-14 text-right">{{ formatRelative(todo.updated_at) }}</span>
          </RouterLink>
        </div>
      </div>
    </div>

    <ProjectIndex />
  </div>
</template>
