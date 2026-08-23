<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useSettingsStore } from "../stores/settings";
import type { AiTestResult } from "../api/models";
import AiResponseWidget from "../components/AiResponseWidget.vue";

const settingsStore = useSettingsStore();
const selectedProviderId = ref("");
const testPrompt = ref("Hallo! Bitte stelle dich kurz vor und nenne drei Tipps für effektives Aufgaben-Management.");
const testModel = ref("");
const testSystemPrompt = ref("Du bist ein hilfreicher KI-Assistent für ein Todo- und Projektmanagement-System.");
const testResult = ref<AiTestResult | null>(null);
const testError = ref("");

const selectedProvider = computed(() =>
  settingsStore.settings.providers.find((provider) => provider.id === selectedProviderId.value),
);

onMounted(async () => {
  settingsStore.resetConnectionState();
  const data = await settingsStore.fetchSettings();
  if (data) {
    selectedProviderId.value = data.default_provider;
    settingsStore.activateProvider(data.default_provider);
    testModel.value = selectedProvider.value?.default_model || "";
  }
});

const selectProvider = () => {
  settingsStore.activateProvider(selectedProviderId.value);
  testModel.value = selectedProvider.value?.default_model || "";
  testResult.value = null;
  testError.value = "";
};

const testConnection = async () => {
  testError.value = "";
  try {
    const models = await settingsStore.fetchModels(selectedProviderId.value);
    if (!models.some((model) => model.id === testModel.value)) {
      testModel.value = models[0]?.id || "";
    }
  } catch {
    // The store exposes the provider error next to its status.
  }
};

const runTestPrompt = async () => {
  if (!testPrompt.value.trim() || !testModel.value || !selectedProviderId.value) return;
  testError.value = "";
  testResult.value = null;
  try {
    testResult.value = await settingsStore.testPrompt({
      provider_id: selectedProviderId.value,
      prompt: testPrompt.value.trim(),
      model: testModel.value,
      system_prompt: testSystemPrompt.value.trim() || undefined,
    });
  } catch (err: any) {
    testError.value = err.response?.status === 504
      ? "Der AI-Provider hat nicht rechtzeitig geantwortet. Ein großes Modell muss beim ersten Aufruf eventuell zunächst geladen werden."
      : err.response?.data?.description ||
        err.response?.data?.error ||
        err.message ||
        "Test-Prompt fehlgeschlagen";
  }
};

const formatBytes = (bytes?: number) => {
  if (!bytes) return "";
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
};
</script>

<template>
  <div class="mx-auto h-full min-h-0 w-full max-w-3xl space-y-8 overflow-y-auto pb-12 pr-1 text-brand-text">
    <div>
      <span class="text-[10px] font-black uppercase tracking-widest text-brand-primary/40">System-Einstellungen</span>
      <h1 class="text-2xl font-black tracking-tight text-brand-text lg:text-3xl">AI-Provider</h1>
      <p class="mt-1 text-xs text-brand-text-muted/60">
        Die Provider werden zentral für die gesamte Todo-Instanz konfiguriert und stehen allen Benutzern zur Verfügung.
      </p>
    </div>

    <section class="space-y-5 rounded-2xl border border-brand-primary/10 bg-brand-container p-4 sm:p-6">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0">
          <h2 class="text-sm font-black uppercase tracking-wider text-brand-primary">Provider-Katalog</h2>
          <p class="mt-0.5 text-xs leading-relaxed text-brand-text-muted/50">
            Endpoints und Standardmodelle werden serverseitig über die YAML-Konfiguration verwaltet.
          </p>
        </div>
        <div
          class="flex shrink-0 items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-bold"
          :class="{
            'border-emerald-500/20 bg-emerald-500/10 text-emerald-400': settingsStore.connectionStatus === 'connected',
            'border-red-500/20 bg-red-500/10 text-red-400': settingsStore.connectionStatus === 'error',
            'border-white/10 bg-white/5 text-brand-text-muted': settingsStore.connectionStatus === 'idle',
          }"
        >
          <span
            class="h-2 w-2 rounded-full"
            :class="{
              'animate-pulse bg-emerald-400': settingsStore.connectionStatus === 'connected',
              'bg-red-400': settingsStore.connectionStatus === 'error',
              'bg-brand-text-muted/40': settingsStore.connectionStatus === 'idle',
            }"
          />
          {{ settingsStore.connectionStatus === 'connected' ? 'Verbunden' : settingsStore.connectionStatus === 'error' ? 'Nicht erreichbar' : 'Ungeprüft' }}
        </div>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
        <div class="min-w-0 space-y-1.5">
          <label class="brand-label">AI-Provider</label>
          <select v-model="selectedProviderId" class="brand-input text-xs" @change="selectProvider">
            <option v-for="provider in settingsStore.settings.providers" :key="provider.id" :value="provider.id">
              {{ provider.name }} · {{ provider.kind }}
            </option>
          </select>
        </div>
        <button
          type="button"
          :disabled="settingsStore.testingConnection || !selectedProviderId"
          class="flex min-h-10 items-center justify-center gap-2 rounded-xl bg-brand-primary/10 px-4 py-2 text-xs font-bold uppercase tracking-wider text-brand-primary transition-all hover:bg-brand-primary/20 disabled:opacity-40"
          @click="testConnection"
        >
          <svg class="h-3.5 w-3.5" :class="{ 'animate-spin': settingsStore.testingConnection }" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ settingsStore.testingConnection ? 'Prüfe…' : 'Modelle laden' }}
        </button>
      </div>

      <div v-if="selectedProvider" class="grid grid-cols-1 gap-2 rounded-xl border border-white/5 bg-white/[0.03] p-3 text-xs sm:grid-cols-2">
        <div class="min-w-0">
          <span class="text-brand-text-muted/50">Provider-ID</span>
          <p class="truncate font-mono text-brand-text">{{ selectedProvider.id }}</p>
        </div>
        <div class="min-w-0">
          <span class="text-brand-text-muted/50">Globales Standardmodell</span>
          <p class="truncate font-mono text-brand-text">{{ selectedProvider.default_model || 'Nicht konfiguriert' }}</p>
        </div>
      </div>

      <div v-if="settingsStore.connectionError" class="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-xs text-red-400">
        <p class="font-bold">Verbindungsfehler</p>
        <p class="mt-0.5 break-words font-mono text-[11px]">{{ settingsStore.connectionError }}</p>
      </div>

      <div v-if="settingsStore.models.length" class="space-y-2 border-t border-brand-primary/10 pt-3">
        <label class="brand-label">Verfügbare Modelle ({{ settingsStore.models.length }})</label>
        <div class="flex max-h-40 flex-wrap gap-2 overflow-y-auto rounded-xl border border-brand-primary/5 bg-white/5 p-2">
          <button
            v-for="model in settingsStore.models"
            :key="model.id"
            type="button"
            class="max-w-full rounded-lg px-2.5 py-1 font-mono text-xs transition-all"
            :class="testModel === model.id ? 'bg-brand-primary font-bold text-brand-container shadow' : 'bg-white/5 text-brand-text-muted hover:bg-white/10 hover:text-brand-text'"
            @click="testModel = model.id"
          >
            <span class="break-all">{{ model.name }}</span>
            <span v-if="model.size" class="ml-1 text-[9px] opacity-60">({{ formatBytes(model.size) }})</span>
          </button>
        </div>
      </div>

      <p class="text-[11px] leading-relaxed text-brand-text-muted/40">
        Änderungen an Hosts, Zugangsdaten oder Standardmodellen erfolgen in <code class="font-mono text-brand-primary/70">config/production.yaml</code> und werden nach einem App-Neustart aktiv.
      </p>
    </section>

    <section class="space-y-5 rounded-2xl border border-brand-primary/10 bg-brand-container p-4 sm:p-6">
      <div>
        <h2 class="text-sm font-black uppercase tracking-wider text-brand-primary">Provider testen</h2>
        <p class="mt-0.5 text-xs text-brand-text-muted/50">Prüfe Modell, Systemprompt und getrennte Thinking-Ausgabe.</p>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <label class="brand-label">Zu testendes Modell</label>
          <select v-model="testModel" class="brand-input text-xs font-mono" :disabled="!settingsStore.models.length">
            <option value="" disabled>{{ settingsStore.models.length ? '-- Modell auswählen --' : '-- Zuerst Modelle laden --' }}</option>
            <option v-for="model in settingsStore.models" :key="model.id" :value="model.id">{{ model.name }}</option>
          </select>
        </div>
        <div class="space-y-1.5">
          <label class="brand-label">System-Prompt (optional)</label>
          <textarea v-model="testSystemPrompt" class="brand-textarea max-h-40 resize-y overflow-y-auto text-xs" rows="3" placeholder="Rolle / Instruktion für das Modell" />
        </div>
      </div>

      <div class="space-y-1.5">
        <label class="brand-label">Test-Prompt</label>
        <textarea v-model="testPrompt" class="brand-textarea resize-y text-xs font-normal" rows="3" placeholder="Schreibe einen Test-Prompt…" />
      </div>

      <button
        type="button"
        :disabled="settingsStore.testingPrompt || !testPrompt.trim() || !testModel || !selectedProviderId"
        class="flex w-full items-center justify-center gap-2 rounded-xl bg-brand-primary/20 py-2.5 text-xs font-bold uppercase tracking-widest text-brand-primary transition-all hover:bg-brand-primary/30 disabled:opacity-40"
        @click="runTestPrompt"
      >
        <svg v-if="settingsStore.testingPrompt" class="h-3.5 w-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        {{ settingsStore.testingPrompt ? 'Generiere Antwort…' : 'Prompt testen' }}
      </button>

      <div v-if="testError" class="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-xs text-red-400">
        <p class="font-bold">Fehler bei der Generierung:</p>
        <p class="mt-0.5 break-words font-mono text-[11px]">{{ testError }}</p>
      </div>

      <AiResponseWidget
        v-if="testResult"
        :response="testResult.response"
        :thinking="testResult.thinking"
        :provider="testResult.provider_id"
        :model="testResult.model"
        :duration-ms="testResult.duration_ms"
        title="AI-Testergebnis"
      />
    </section>
  </div>
</template>
