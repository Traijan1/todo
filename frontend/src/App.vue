<script setup lang="ts">
import { useAuthStore } from "./stores/auth";
import { useProjectStore } from "./stores/projects";
import { useRouter, useRoute } from "vue-router";
import { computed, ref, onMounted, watch, nextTick } from "vue";

const auth = useAuthStore();
const projectStore = useProjectStore();
const router = useRouter();
const route = useRoute();

const isAuthPage = computed(() => ["login", "register"].includes(route.name as string));
const isSidebarOpen = ref(false);
const mcpTokenCopied = ref(false);

const copyMcpToken = async () => {
  if (auth.user?.mcp_token) {
    await navigator.clipboard.writeText(auth.user.mcp_token);
    mcpTokenCopied.value = true;
    setTimeout(() => {
      mcpTokenCopied.value = false;
    }, 2000);
  }
};

const mobileTitle = computed(() => {
  if (route.name === "project-detail") {
    const project = projectStore.projects.find((p) => p.pid === route.params.pid);
    return project?.title || "Todo";
  }
  return "Todo";
});

const handleLogout = () => {
  auth.logout();
  router.push("/login");
};

const loadProjects = async () => {
  if (auth.user && !projectStore.projects.length) {
    await projectStore.fetchProjects();
  }
  if (auth.user && !auth.user.mcp_token) {
    await auth.fetchCurrentUser();
  }
};

onMounted(loadProjects);
watch(() => auth.user, loadProjects);
</script>

<template>
  <div class="flex h-[100dvh] bg-brand-background text-brand-text overflow-hidden relative">
    <!-- Safe area filler -->
    <div v-if="!isAuthPage" class="lg:hidden fixed top-0 left-0 right-0 z-[41] bg-brand-container" style="height: env(safe-area-inset-top)" />

    <!-- Mobile Header -->
    <header v-if="!isAuthPage" class="lg:hidden flex items-center justify-between px-4 py-3 bg-brand-container border-b border-brand-primary/10 fixed left-0 right-0 z-40" style="top: env(safe-area-inset-top)">
      <button @click="isSidebarOpen = !isSidebarOpen" class="p-2 text-brand-primary bg-brand-primary/10 hover:bg-brand-primary/20 rounded-xl transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path v-if="!isSidebarOpen" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
      <h1 class="text-sm font-black tracking-tight text-brand-primary truncate max-w-[55vw]">{{ mobileTitle }}</h1>
      <RouterLink to="/projects/new" @click="isSidebarOpen = false" class="p-2 text-brand-primary bg-brand-primary/10 hover:bg-brand-primary/20 rounded-xl transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
        </svg>
      </RouterLink>
    </header>

    <!-- Mobile Sidebar Backdrop -->
    <Transition name="fade-backdrop">
      <div v-if="!isAuthPage && isSidebarOpen" @click="isSidebarOpen = false" class="fixed inset-0 bg-black/60 z-40 lg:hidden backdrop-blur-sm" />
    </Transition>

    <!-- Sidebar -->
    <aside
      v-if="!isAuthPage"
      :class="['w-64 bg-brand-container flex flex-col border-r border-brand-primary/10 fixed lg:static inset-y-0 left-0 z-50 transform transition-transform duration-300', isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0']"
    >
      <!-- Logo -->
      <div class="p-5 flex items-center gap-3 border-b border-brand-primary/10 shrink-0">
        <div class="w-8 h-8 rounded-xl bg-brand-primary/20 flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-brand-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h1 class="text-base font-black tracking-tight text-brand-primary">Todo</h1>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 p-3 overflow-y-auto space-y-0.5">
        <!-- All Projects -->
        <RouterLink
          to="/"
          @click="isSidebarOpen = false"
          class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium text-brand-text-muted hover:text-brand-text hover:bg-white/5 transition-all"
          active-class="!bg-brand-primary/10 !text-brand-primary !font-bold"
          exact
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          All Projects
        </RouterLink>

        <!-- Settings & AI -->
        <RouterLink
          to="/settings"
          @click="isSidebarOpen = false"
          class="flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium text-brand-text-muted hover:text-brand-text hover:bg-white/5 transition-all"
          active-class="!bg-brand-primary/10 !text-brand-primary !font-bold"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          AI & Settings
        </RouterLink>

        <!-- Project list -->
        <div v-if="projectStore.projects.length" class="flex flex-col gap-y-2 pt-5">
          <p class="text-[9px] uppercase tracking-widest font-black text-brand-primary/30 px-3 mb-2">Projects</p>
          <RouterLink
            v-for="project in projectStore.projects"
            :key="project.pid"
            :to="`/projects/${project.pid}`"
            @click="isSidebarOpen = false"
            class="flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-brand-text-muted hover:text-brand-text hover:bg-white/5 transition-all"
            active-class="!bg-brand-primary/10 !text-brand-primary !font-semibold"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-brand-primary/40 shrink-0"></span>
            <span class="truncate">{{ project.title }}</span>
          </RouterLink>
        </div>

        <!-- New Project -->
        <div class="pt-2">
          <RouterLink to="/projects/new" @click="isSidebarOpen = false" class="flex items-center gap-2.5 px-3 py-2 rounded-xl text-sm text-brand-primary/40 hover:text-brand-primary hover:bg-brand-primary/10 transition-all">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
            </svg>
            New Project
          </RouterLink>
        </div>
      </nav>

      <!-- User section -->
      <div class="p-4 border-t border-brand-primary/10 shrink-0">
        <div v-if="auth.user" class="space-y-2">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 shrink-0 rounded-full bg-brand-primary/20 flex items-center justify-center text-brand-primary font-black text-xs">
              {{ auth.user.name.charAt(0).toUpperCase() }}
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-bold text-brand-text truncate">{{ auth.user.name }}</p>
              <p class="text-[10px] text-brand-text-muted/50 truncate">{{ auth.user.email }}</p>
            </div>
            <button @click="handleLogout" class="p-1.5 rounded-lg text-brand-text-muted/40 hover:text-red-400 hover:bg-red-500/10 transition-all shrink-0" title="Logout">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
            </button>
          </div>
          <!-- MCP Token -->
          <div v-if="auth.user.mcp_token" class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg bg-brand-primary/5 border border-brand-primary/10">
            <span class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40 shrink-0">MCP</span>
            <span class="flex-1 text-[9px] font-mono text-brand-text-muted/50 truncate">{{ auth.user.mcp_token }}</span>
            <button @click="copyMcpToken" class="shrink-0 text-brand-primary/30 hover:text-brand-primary transition-colors" :title="mcpTokenCopied ? 'Copied!' : 'Copy MCP token'">
              <svg v-if="!mcpTokenCopied" xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
              </svg>
            </button>
          </div>
        </div>
        <RouterLink v-else to="/login" @click="isSidebarOpen = false" class="w-full block px-4 py-2 text-sm font-medium hover:bg-white/5 rounded-xl transition-colors text-brand-text-muted text-center"> Login </RouterLink>
      </div>
    </aside>

    <!-- Main Content -->
    <main
      class="app-main flex-1 flex flex-col min-w-0 overflow-hidden"
      :class="isAuthPage ? '' : 'px-4 pb-4 lg:p-8'"
      :style="!isAuthPage ? 'padding-top: calc(3.75rem + env(safe-area-inset-top)); padding-bottom: max(1rem, env(safe-area-inset-bottom))' : ''"
    >
      <router-view />
    </main>
  </div>
</template>

<style>
@media (min-width: 1024px) {
  .app-main {
    padding-top: 2rem !important;
    padding-bottom: 0 !important;
  }
}

.fade-backdrop-enter-active,
.fade-backdrop-leave-active {
  transition: opacity 0.25s ease;
}
.fade-backdrop-enter-from,
.fade-backdrop-leave-to {
  opacity: 0;
}
</style>
