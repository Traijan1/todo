<script setup lang="ts">
import { useAuthStore } from "./stores/auth";
import { useRouter, useRoute } from "vue-router";
import { computed } from "vue";

const auth = useAuthStore();
const router = useRouter();
const route = useRoute();

const isAuthPage = computed(() => ['login', 'register'].includes(route.name as string));

const handleLogout = () => {
  auth.logout();
  router.push("/login");
};
</script>

<template>
  <div class="flex min-h-screen bg-brand-background text-brand-text">
    <!-- Sidebar - only show if not on auth pages -->
    <aside v-if="!isAuthPage" class="w-64 bg-brand-container text-purple-200 flex flex-col shadow-2xl border-r border-purple-900/30">
      <div class="p-6">
        <h1 class="text-2xl font-bold tracking-tight text-brand-primary">Loco Todo</h1>
      </div>
      
      <nav class="flex-1 px-4 space-y-2 mt-4">
        <router-link to="/" 
          class="flex items-center px-4 py-3 rounded-xl hover:bg-white/5 transition-colors"
          active-class="bg-brand-primary/10 text-brand-primary font-semibold border border-brand-primary/20 shadow-[0_0_15px_rgba(224,187,228,0.1)]">
          <span>Home</span>
        </router-link>
        <router-link to="/about" 
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
          class="w-full block px-4 py-2 text-sm font-medium hover:bg-white/5 rounded-xl transition-colors text-purple-300 text-center">
          Login
        </router-link>
      </div>
    </aside>

    <!-- Main Content -->
    <main :class="['flex-1 flex flex-col', isAuthPage ? '' : 'p-8']">
      <router-view />
    </main>
  </div>
</template>

<style>
/* Transitions and other global styles */
.page-enter-active, .page-leave-active {
  transition: opacity 0.3s ease;
}
.page-enter-from, .page-leave-to {
  opacity: 0;
}
</style>
