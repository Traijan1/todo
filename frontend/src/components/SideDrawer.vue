<script setup lang="ts">
interface Props {
  isOpen: boolean;
  title?: string;
  subtitle?: string;
  width?: string;
}

withDefaults(defineProps<Props>(), {
  title: 'Details',
  width: 'w-[450px]'
});

const emit = defineEmits<{
  (e: 'close'): void;
}>();
</script>

<template>
  <div class="relative">
    <!-- Overlay / Backdrop -->
    <Transition name="fade">
      <div 
        v-if="isOpen" 
        @click="emit('close')" 
        class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 transition-opacity"
      ></div>
    </Transition>

    <!-- Side Drawer Panel -->
    <Transition name="slide">
      <div 
        v-if="isOpen" 
        :class="[
          'fixed inset-y-0 right-0 z-50 bg-brand-container shadow-[-20px_0_50px_rgba(0,0,0,0.5)] border-l border-brand-primary/10 flex flex-col transition-transform',
          width
        ]"
      >
        <!-- Header Section -->
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-brand-background/20">
          <div>
            <slot name="header">
              <h3 class="text-xl font-black text-brand-primary uppercase tracking-tighter">{{ title }}</h3>
              <p v-if="subtitle" class="text-[10px] text-brand-text-muted font-bold uppercase tracking-widest mt-0.5">
                {{ subtitle }}
              </p>
            </slot>
          </div>
          <button @click="emit('close')" class="text-brand-text-muted hover:text-white transition-colors p-2 hover:bg-white/5 rounded-full">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </header>

        <!-- Body / Main Content -->
        <div class="flex-1 p-8 overflow-y-auto custom-scrollbar">
          <slot />
        </div>

        <!-- Footer Section -->
        <footer v-if="$slots.footer" class="p-6 bg-brand-background/20 border-t border-white/5">
          <slot name="footer" />
        </footer>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Slide Transition */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  will-change: transform;
}

.slide-enter-from,
.slide-leave-to {
  transform: translate3d(100%, 0, 0);
}

.slide-enter-to,
.slide-leave-from {
  transform: translate3d(0, 0, 0);
}

/* Fade Transition for Backdrop */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}


/* Scrollbar Styling within the Drawer */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(var(--brand-primary-rgb), 0.1);
  border-radius: 10px;
}
</style>
