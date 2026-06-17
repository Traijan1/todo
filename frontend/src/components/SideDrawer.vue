<script setup lang="ts">
import { ref } from "vue";

interface Props {
  isOpen: boolean;
  title?: string;
  subtitle?: string;
  width?: string;
  pid?: string;
}

const props = withDefaults(defineProps<Props>(), {
  title: "Details",
  width: "lg:w-[600px]",
});

const emit = defineEmits<{
  (e: "close"): void;
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
    <!-- Backdrop -->
    <Transition name="fade">
      <div
        v-if="isOpen"
        @click="emit('close')"
        class="fixed inset-0 bg-black/50 z-[90] backdrop-blur-sm"
      />
    </Transition>

    <!-- Panel: bottom sheet on mobile, right panel on desktop -->
    <Transition name="slide">
      <div
        v-if="isOpen"
        :class="[
          'fixed z-[100] bg-brand-container flex flex-col shadow-2xl',
          'bottom-0 left-0 right-0 max-h-[90dvh] rounded-t-3xl',
          'lg:inset-y-0 lg:right-0 lg:left-auto lg:max-h-none lg:rounded-none',
          width,
        ]"
      >
        <!-- Drag handle (mobile only) -->
        <div class="lg:hidden flex justify-center pt-3 pb-1 shrink-0">
          <div class="w-10 h-1 rounded-full bg-brand-primary/20" />
        </div>

        <!-- PID Badge -->
        <div v-if="pid" class="absolute top-4 right-14 z-20 lg:top-4">
          <button
            @click="copyPid"
            class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-brand-primary/10 border border-brand-primary/30 hover:bg-brand-primary/20 hover:border-brand-primary/50 transition-all group/pid backdrop-blur-md"
          >
            <span class="text-[10px] font-mono font-bold text-brand-primary tracking-wider uppercase">
              ID: {{ pid.split("-")[0] }}...
            </span>
            <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-brand-primary group-hover/pid:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
        <header class="px-6 pt-4 pb-4 border-b border-white/5 bg-brand-background/10 shrink-0">
          <slot name="header">
            <h3 class="text-lg font-bold text-brand-primary pr-20">{{ title }}</h3>
            <p v-if="subtitle" class="text-[9px] text-brand-text-muted font-bold uppercase tracking-widest mt-1">
              {{ subtitle }}
            </p>
          </slot>
        </header>

        <!-- Body -->
        <div class="flex-1 p-6 overflow-y-auto custom-scrollbar overflow-x-hidden min-h-0">
          <slot />
        </div>

        <!-- Footer -->
        <footer
          v-if="$slots.footer"
          class="px-6 py-5 bg-brand-background/20 border-t border-white/5 shrink-0"
          style="padding-bottom: max(1.25rem, env(safe-area-inset-bottom))"
        >
          <slot name="footer" />
        </footer>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Mobile: slide up from bottom */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease;
}
.slide-enter-from,
.slide-leave-to {
  transform: translateY(100%);
  opacity: 0.6;
}

/* Desktop: slide in from right */
@media (min-width: 1024px) {
  .slide-enter-from,
  .slide-leave-to {
    transform: translateX(100%);
    opacity: 0;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
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
