<script setup lang="ts">
import { ref, reactive } from "vue";
import { useAuthStore } from "../stores/auth";
import { useRouter } from "vue-router";

const auth = useAuthStore();
const router = useRouter();

const form = reactive({
  name: "",
  email: "",
  password: "",
  confirmPassword: "",
});

const localError = ref("");

async function handleRegister() {
  localError.value = "";

  if (form.password !== form.confirmPassword) {
    localError.value = "Passwords do not match";
    return;
  }

  try {
    await auth.register({
      name: form.name,
      email: form.email,
      password: form.password,
    });
    
    router.push("/");
  } catch (err) {
    // Error is handled in the store and exposed via auth.error
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-brand-background px-4 py-12">
    <div class="max-w-md w-full space-y-8 p-10 bg-brand-container rounded-3xl shadow-2xl border border-brand-primary/10">
      <div>
        <h2 class="mt-6 text-center text-3xl font-black text-brand-primary tracking-tight">
          Join Us
        </h2>
        <p class="mt-2 text-center text-sm text-brand-text-muted">
          Already have an account?
          <router-link to="/login" class="font-bold text-brand-secondary hover:text-brand-primary transition-colors underline decoration-brand-secondary/30 underline-offset-4">
            Sign in
          </router-link>
        </p>
      </div>
      
      <form class="mt-8 space-y-4" @submit.prevent="handleRegister">
        <div v-if="auth.error || localError" class="bg-red-950/30 border border-red-900/50 text-red-400 px-5 py-4 rounded-2xl relative text-sm" role="alert">
          <span>{{ auth.error || localError }}</span>
        </div>

        <div class="space-y-4">
          <div>
            <label for="name" class="block text-xs font-bold uppercase tracking-wider text-brand-primary/60 mb-2 ml-1">Full Name</label>
            <input v-model="form.name" id="name" name="name" type="text" required 
              class="appearance-none relative block w-full px-5 py-4 border border-brand-primary/10 placeholder-brand-text-muted/30 text-brand-text rounded-2xl focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 sm:text-sm bg-brand-background/50 transition-all" 
              placeholder="John Doe" />
          </div>
          <div>
            <label for="email-address" class="block text-xs font-bold uppercase tracking-wider text-brand-primary/60 mb-2 ml-1">Email address</label>
            <input v-model="form.email" id="email-address" name="email" type="email" autocomplete="email" required 
              class="appearance-none relative block w-full px-5 py-4 border border-brand-primary/10 placeholder-brand-text-muted/30 text-brand-text rounded-2xl focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 sm:text-sm bg-brand-background/50 transition-all" 
              placeholder="name@example.com" />
          </div>
          <div>
            <label for="password" class="block text-xs font-bold uppercase tracking-wider text-brand-primary/60 mb-2 ml-1">Password</label>
            <input v-model="form.password" id="password" name="password" type="password" autocomplete="new-password" required 
              class="appearance-none relative block w-full px-5 py-4 border border-brand-primary/10 placeholder-brand-text-muted/30 text-brand-text rounded-2xl focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 sm:text-sm bg-brand-background/50 transition-all" 
              placeholder="••••••••" />
          </div>
          <div>
            <label for="confirm-password" class="block text-xs font-bold uppercase tracking-wider text-brand-primary/60 mb-2 ml-1">Confirm Password</label>
            <input v-model="form.confirmPassword" id="confirm-password" name="confirm-password" type="password" required 
              class="appearance-none relative block w-full px-5 py-4 border border-brand-primary/10 placeholder-brand-text-muted/30 text-brand-text rounded-2xl focus:outline-none focus:ring-2 focus:ring-brand-primary/20 focus:border-brand-primary/50 sm:text-sm bg-brand-background/50 transition-all" 
              placeholder="••••••••" />
          </div>
        </div>

        <div class="pt-2">
          <button type="submit" :disabled="auth.loading" 
            class="group relative w-full flex justify-center py-2.5 px-6 border border-transparent text-base font-bold rounded-xl text-brand-container bg-brand-primary hover:bg-brand-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-brand-primary disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-brand-primary/10 transition-all active:scale-[0.98]">
            <span v-if="auth.loading" class="absolute left-0 inset-y-0 flex items-center pl-4">
              <svg class="animate-spin h-5 w-5 text-brand-container" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </span>
            {{ auth.loading ? 'Creating account...' : 'Create Account' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
