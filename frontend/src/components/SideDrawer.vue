<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  isOpen: boolean;
  title?: string;
  subtitle?: string;
  width?: string;
  pid?: string;
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Details',
  width: 'w-[600px]'
});

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const copied = ref(false);

const copyPid = async () => {
  if (props.pid) {
    await navigator.clipboard.writeText(props.pid);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="slide">
      <div 
        v-if="isOpen" 
        :class="[
          'fixed inset-y-0 right-0 z-[100] bg-brand-container border-l border-brand-primary/20 flex flex-col shadow-2xl transition-all',
          width
        ]"
      >
        <!-- Enhanced PID Badge (Top Right) -->
        <div v-if="pid" class="absolute top-4 right-14 z-20">
          <button 
            @click="copyPid"
            class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-brand-primary/10 border border-brand-primary/30 hover:bg-brand-primary/20 hover:border-brand-primary/50 transition-all group/pid backdrop-blur-md"
          >
            <span class="text-[10px] font-mono font-bold text-brand-primary tracking-wider uppercase">ID: {{ pid.split('-')[0] }}...</span>
            <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-brand-primary group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2" />
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </button>
        </div>

        <!-- Close Button -->
        <button 
          @click="emit('close')" 
          class="absolute top-4 right-4 z-20 text-brand-text-muted hover:text-white transition-colors p-2 hover:bg-white/10 rounded-full"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        <!-- Header -->
        <header class="p-6 pb-4 border-b border-white/5 bg-brand-background/10">
          <slot name="header">
            <h3 class="text-lg font-bold text-brand-primary pr-20">{{ title }}</h3>
            <p v-if="subtitle" class="text-[9px] text-brand-text-muted font-bold uppercase tracking-widest mt-1">
              {{ subtitle }}
            </p>
          </slot>
        </header>

        <!-- Main Body -->
        <div class="flex-1 p-6 overflow-y-auto custom-scrollbar overflow-x-hidden">
          <slot />
        </div>

        <!-- Footer -->
        <footer v-if="$slots.footer" class="p-6 bg-brand-background/20 border-t border-white/5">
          <slot name="footer" />
        </footer>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.4s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.2);
  border-radius: 10px;
}
</style>
