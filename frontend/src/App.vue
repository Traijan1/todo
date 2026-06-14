<script setup lang="ts">
import { useAuthStore } from "./stores/auth";
import { useRouter, useRoute } from "vue-router";
import { computed, ref } from "vue";

const auth = useAuthStore();
const router = useRouter();
const route = useRoute();

const isAuthPage = computed(() => ['login', 'register'].includes(route.name as string));
const isSidebarOpen = ref(false);

const handleLogout = () => {
  auth.logout();
  router.push("/login");
};
</script>

<template>
  <div class="flex min-h-[100dvh] bg-brand-background text-brand-text overflow-x-hidden relative">

    <!-- Safe area filler behind notch (extends header bg) -->
    <div
      v-if="!isAuthPage"
      class="lg:hidden fixed top-0 left-0 right-0 z-[41] bg-brand-container"
      style="height: env(safe-area-inset-top)"
    />

    <!-- Mobile Header -->
    <header
      v-if="!isAuthPage"
      class="lg:hidden flex items-center justify-between px-4 py-3.5 bg-brand-container border-b border-brand-primary/10 fixed left-0 right-0 z-40"
      style="top: env(safe-area-inset-top)"
    >
      <h1 class="text-lg font-bold tracking-tight text-brand-primary">Todo</h1>
      <button @click="isSidebarOpen = !isSidebarOpen" class="p-2 text-brand-primary bg-brand-primary/10 hover:bg-brand-primary/20 rounded-lg transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path v-if="!isSidebarOpen" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </header>

    <!-- Mobile Sidebar Backdrop -->
    <div
      v-if="!isAuthPage && isSidebarOpen"
      @click="isSidebarOpen = false"
      class="fixed inset-0 bg-black/50 z-40 lg:hidden backdrop-blur-sm transition-opacity"
    ></div>

    <!-- Sidebar -->
    <aside
      v-if="!isAuthPage"
      :class="[
        'w-64 bg-brand-container text-purple-200 flex flex-col shadow-2xl border-r border-purple-900/30 fixed lg:static inset-y-0 left-0 z-50 transform transition-transform duration-300',
        isSidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'
      ]"
    >
      <div class="p-6 hidden lg:block">
        <h1 class="text-2xl font-bold tracking-tight text-brand-primary">Todo</h1>
      </div>

      <nav class="flex-1 px-4 space-y-2 mt-4 lg:mt-0">
        <router-link to="/"
          @click="isSidebarOpen = false"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-white/5 transition-colors"
          active-class="bg-brand-primary/10 text-brand-primary font-semibold border border-brand-primary/20 shadow-[0_0_15px_rgba(224,187,228,0.1)]">
          <span>Home</span>
        </router-link>
        <router-link to="/about"
          @click="isSidebarOpen = false"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-white/5 transition-colors"
          active-class="bg-brand-primary/10 text-brand-primary font-semibold border border-brand-primary/20 shadow-[0_0_15px_rgba(224,187,228,0.1)]">
          <span>About</span>
        </router-link>
      </nav>

      <div class="p-4 border-t border-purple-900/30">
        <div v-if="auth.user" class="mb-4 px-4">
          <p class="text-xs uppercase font-bold text-purple-500/70">User</p>
          <p class="truncate font-medium text-purple-100">{{ auth.user.name }}</p>
        </div>
        <button
          v-if="auth.user"
          @click="handleLogout"
          class="w-full text-left px-4 py-2 text-sm font-medium hover:bg-white/5 rounded-xl transition-colors text-purple-300">
          Logout
        </button>
        <router-link v-else to="/login"
          @click="isSidebarOpen = false"
          class="w-full block px-4 py-2 text-sm font-medium hover:bg-white/5 rounded-xl transition-colors text-purple-300 text-center">
          Login
        </router-link>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="app-main flex-1 flex flex-col" :class="isAuthPage ? '' : 'px-4 pb-4 lg:p-8 lg:pt-8'" :style="!isAuthPage ? 'padding-top: calc(3.75rem + env(safe-area-inset-top)); padding-bottom: max(1rem, env(safe-area-inset-bottom))' : ''">
      <router-view />
    </main>
  </div>
</template>

<style>
/* Desktop overrides for safe-area padding */
@media (min-width: 1024px) {
  .app-main {
    padding-top: 2rem !important;
    padding-bottom: 0 !important;
  }
}

.page-enter-active, .page-leave-active {
  transition: opacity 0.3s ease;
}
.page-enter-from, .page-leave-to {
  opacity: 0;
}
</style>
