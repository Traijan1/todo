<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "../stores/settings";
import type { AiTestResult } from "../api/models";

const settingsStore = useSettingsStore();

const form = ref({
  ollama_url: "",
  default_model: "",
});

const saveSuccess = ref(false);
const saveError = ref("");

// Test prompt state
const testPrompt = ref("Hallo! Bitte stelle dich kurz vor und nenne drei Tipps für effektives Aufgaben-Management.");
const testModel = ref("");
const testSystemPrompt = ref("Du bist ein hilfreicher KI-Assistent für ein Todo- und Projektmanagement-System.");
const testResult = ref<AiTestResult | null>(null);
const testError = ref("");
const copiedResponse = ref(false);

onMounted(async () => {
  settingsStore.resetConnectionState(true);
  const data = await settingsStore.fetchSettings();
  if (data) {
    form.value.ollama_url = data.ollama_url || "";
    form.value.default_model = data.default_model || "";
  }
});

const testConnection = async () => {
  testError.value = "";
  try {
    const models = await settingsStore.fetchOllamaModels(form.value.ollama_url);
    if (models.length > 0 && !testModel.value) {
      testModel.value = models[0].name;
    }
  } catch (err: any) {
    // handled by store
  }
};

const handleSave = async () => {
  saveSuccess.value = false;
  saveError.value = "";
  try {
    await settingsStore.updateSettings({
      ollama_url: form.value.ollama_url.trim(),
      default_model: form.value.default_model.trim() || undefined,
    });
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 3000);
  } catch (err: any) {
    saveError.value =
      err.response?.data?.description ||
      err.response?.data?.error ||
      "Fehler beim Speichern der Einstellungen";
  }
};

const runTestPrompt = async () => {
  if (!testPrompt.value.trim() || !testModel.value) return;
  testError.value = "";
  testResult.value = null;
  try {
    const res = await settingsStore.testOllamaPrompt({
      prompt: testPrompt.value.trim(),
      model: testModel.value,
      system_prompt: testSystemPrompt.value.trim() || undefined,
      ollama_url: form.value.ollama_url.trim(),
    });
    testResult.value = res;
  } catch (err: any) {
    testError.value = err.response?.status === 504
      ? "Ollama hat nicht rechtzeitig geantwortet. Beim ersten Start muss ein großes Modell eventuell zunächst geladen werden. Bitte versuche es erneut."
      : err.response?.data?.description ||
        err.response?.data?.error ||
        err.message ||
        "Test-Prompt fehlgeschlagen";
  }
};

const copyResponse = async () => {
  if (testResult.value?.response) {
    await navigator.clipboard.writeText(testResult.value.response);
    copiedResponse.value = true;
    setTimeout(() => {
      copiedResponse.value = false;
    }, 2000);
  }
};

const formatBytes = (bytes?: number) => {
  if (!bytes) return "";
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
};
</script>

<template>
  <div class="max-w-3xl mx-auto w-full h-full min-h-0 overflow-y-auto text-brand-text space-y-8 pb-12 pr-1">
    <!-- Header -->
    <div>
      <div class="flex items-center gap-2 mb-1">
        <span class="text-[10px] font-black uppercase tracking-widest text-brand-primary/40">System-Einstellungen</span>
      </div>
      <h1 class="text-2xl lg:text-3xl font-black tracking-tight text-brand-text">AI & Ollama Konfiguration</h1>
      <p class="text-xs text-brand-text-muted/60 mt-1">
        Verwalte deine lokale oder netzwerkweite Ollama-Verbindung und teste Modelle mit Prompts.
      </p>
    </div>

    <!-- Ollama Connection Settings -->
    <section class="p-6 rounded-2xl bg-brand-container border border-brand-primary/10 space-y-5">
      <div class="flex items-center justify-between gap-4">
        <div>
          <h2 class="text-sm font-black uppercase tracking-wider text-brand-primary">Ollama Server</h2>
          <p class="text-xs text-brand-text-muted/50 mt-0.5">
            Host-URL für lokale oder im Netzwerk erreichbare Ollama-Instanzen.
          </p>
        </div>
        <!-- Status badge -->
        <div class="shrink-0 flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold"
             :class="{
               'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20': settingsStore.connectionStatus === 'connected',
               'bg-red-500/10 text-red-400 border border-red-500/20': settingsStore.connectionStatus === 'error',
               'bg-white/5 text-brand-text-muted border border-white/10': settingsStore.connectionStatus === 'idle',
             }">
          <span class="w-2 h-2 rounded-full"
                :class="{
                  'bg-emerald-400 animate-pulse': settingsStore.connectionStatus === 'connected',
                  'bg-red-400': settingsStore.connectionStatus === 'error',
                  'bg-brand-text-muted/40': settingsStore.connectionStatus === 'idle',
                }"></span>
          {{ settingsStore.connectionStatus === 'connected' ? 'Verbunden' : settingsStore.connectionStatus === 'error' ? 'Nicht erreichbar' : 'Ungeprüft' }}
        </div>
      </div>

      <div class="space-y-4">
        <div class="space-y-1.5">
          <label class="brand-label">Ollama Host-URL</label>
          <div class="flex gap-2">
            <input
              v-model="form.ollama_url"
              type="text"
              class="brand-input flex-1 font-mono text-xs"
              @input="settingsStore.resetConnectionState(true); testModel = ''"
              @keydown.enter="testConnection"
            />
            <button
              type="button"
              :disabled="settingsStore.testingConnection || !form.ollama_url.trim()"
              class="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider bg-brand-primary/10 text-brand-primary hover:bg-brand-primary/20 transition-all disabled:opacity-40 flex items-center gap-2 shrink-0"
              @click="testConnection"
            >
              <svg v-if="settingsStore.testingConnection" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              {{ settingsStore.testingConnection ? 'Prüfe...' : 'Verbindung testen' }}
            </button>
          </div>
          <p class="text-[11px] text-brand-text-muted/40">
            Für Ollama auf diesem Server: <code class="font-mono text-brand-primary/60">http://host.docker.internal:11434</code>. Die Verbindung wird erst beim Klick auf „Verbindung testen“ geprüft.
          </p>
        </div>

        <!-- Connection Error Display -->
        <div v-if="settingsStore.connectionError" class="p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-xs">
          <p class="font-bold flex items-center gap-1.5 mb-0.5">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            Verbindungsfehler:
          </p>
          <p class="font-mono text-[11px] break-all opacity-90">{{ settingsStore.connectionError }}</p>
          <p class="text-[10px] text-red-300/70 mt-1">
            Hinweis: Stelle sicher, dass Ollama läuft (<code class="font-mono">ollama serve</code>) und nicht ausschließlich an <code class="font-mono">127.0.0.1</code> gebunden ist.
          </p>
        </div>

        <!-- Available Models list -->
        <div v-if="settingsStore.models.length > 0" class="space-y-2 pt-2 border-t border-brand-primary/10">
          <div class="flex items-center justify-between">
            <label class="brand-label !mb-0">Verfügbare Modelle ({{ settingsStore.models.length }})</label>
            <span class="text-[10px] text-brand-primary/40 font-mono">Gefunden via /api/tags</span>
          </div>
          <div class="flex flex-wrap gap-2 max-h-36 overflow-y-auto p-2 rounded-xl bg-white/5 border border-brand-primary/5">
            <button
              v-for="m in settingsStore.models"
              :key="m.name"
              type="button"
              class="px-2.5 py-1 rounded-lg text-xs font-mono transition-all flex items-center gap-1.5"
              :class="form.default_model === m.name ? 'bg-brand-primary text-brand-container font-bold shadow' : 'bg-white/5 hover:bg-white/10 text-brand-text-muted hover:text-brand-text'"
              @click="form.default_model = m.name; testModel = m.name"
            >
              <span>{{ m.name }}</span>
              <span v-if="m.size" class="text-[9px] opacity-60">({{ formatBytes(m.size) }})</span>
            </button>
          </div>
        </div>

        <!-- Default Model Input/Select -->
        <div class="space-y-1.5">
          <label class="brand-label">Standard-Modell (Globaler Fallback)</label>
          <select
            v-model="form.default_model"
            class="brand-input text-xs font-mono"
            :disabled="settingsStore.models.length === 0"
          >
            <option value="">
              {{ settingsStore.models.length ? '-- Kein Standard-Modell ausgewählt --' : '-- Zuerst Verbindung testen --' }}
            </option>
            <option
              v-if="form.default_model && !settingsStore.models.some((m) => m.name === form.default_model)"
              :value="form.default_model"
            >
              {{ form.default_model }} (nicht auf dem Server gefunden)
            </option>
            <option v-for="m in settingsStore.models" :key="m.name" :value="m.name">
              {{ m.name }} {{ m.details?.parameter_size ? `(${m.details.parameter_size})` : '' }}
            </option>
          </select>
        </div>
      </div>

      <!-- Save Button -->
      <div class="pt-2">
        <button
          type="button"
          :disabled="settingsStore.saving || !form.ollama_url.trim()"
          class="w-full py-3 rounded-xl font-bold text-xs uppercase tracking-widest transition-all disabled:opacity-40 flex items-center justify-center gap-2"
          :class="saveSuccess ? 'bg-emerald-500/20 text-emerald-400' : 'bg-brand-primary text-brand-container hover:bg-brand-primary/90 shadow-lg shadow-brand-primary/10'"
          @click="handleSave"
        >
          <svg v-if="settingsStore.saving" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          <svg v-else-if="saveSuccess" xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
          </svg>
          {{ saveSuccess ? 'Einstellungen gespeichert!' : settingsStore.saving ? 'Speichern...' : 'Einstellungen speichern' }}
        </button>
        <p v-if="saveError" class="text-xs text-red-400 mt-2 text-center">{{ saveError }}</p>
      </div>
    </section>

    <!-- Test Prompt Section -->
    <section class="p-6 rounded-2xl bg-brand-container border border-brand-primary/10 space-y-5">
      <div class="flex items-center justify-between gap-4">
        <div>
          <h2 class="text-sm font-black uppercase tracking-wider text-brand-primary">Ollama Prompt-Tester</h2>
          <p class="text-xs text-brand-text-muted/50 mt-0.5">
            Sende Test-Prompts direkt an deine konfigurierte Ollama-Instanz und überprüfe die Antwortqualität.
          </p>
        </div>
      </div>

      <div class="space-y-4">
        <!-- Model to test -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="brand-label">Zu testendes Modell</label>
            <select
              v-model="testModel"
              class="brand-input text-xs font-mono"
              :disabled="settingsStore.models.length === 0"
            >
              <option value="" disabled>
                {{ settingsStore.models.length ? '-- Modell auswählen --' : '-- Zuerst Verbindung testen --' }}
              </option>
              <option v-for="m in settingsStore.models" :key="m.name" :value="m.name">
                {{ m.name }}
              </option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="brand-label">System-Prompt (optional)</label>
            <textarea
              v-model="testSystemPrompt"
              class="brand-textarea text-xs max-h-40 overflow-y-auto resize-y"
              rows="3"
              placeholder="Rolle / Instruktion für das Modell"
            />
          </div>
        </div>

        <!-- Prompt input -->
        <div class="space-y-1.5">
          <label class="brand-label">Test-Prompt</label>
          <textarea
            v-model="testPrompt"
            class="brand-textarea text-xs font-normal"
            rows="3"
            placeholder="Schreibe einen Test-Prompt..."
          />
        </div>

        <!-- Action Button -->
        <button
          type="button"
          :disabled="settingsStore.testingPrompt || !testPrompt.trim() || !testModel"
          class="w-full py-2.5 rounded-xl font-bold text-xs uppercase tracking-widest bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-40 flex items-center justify-center gap-2"
          @click="runTestPrompt"
        >
          <svg v-if="settingsStore.testingPrompt" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          {{ settingsStore.testingPrompt ? 'Generiere Antwort via Ollama...' : 'Prompt testen' }}
        </button>

        <!-- Test Error -->
        <div v-if="testError" class="p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-xs">
          <p class="font-bold mb-0.5">Fehler bei der Generierung:</p>
          <p class="font-mono text-[11px] break-all">{{ testError }}</p>
        </div>

        <!-- Test Result Output -->
        <div v-if="testResult" class="p-4 rounded-xl bg-white/5 border border-brand-primary/15 space-y-3">
          <div class="flex items-center justify-between gap-2 pb-2 border-b border-white/5">
            <div class="flex items-center gap-2">
              <span class="text-[10px] font-black uppercase tracking-wider text-brand-primary">Antwort</span>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-brand-primary/10 text-brand-primary">
                {{ testResult.model }}
              </span>
            </div>
            <div class="flex items-center gap-3">
              <span v-if="testResult.duration_ms" class="text-[10px] font-mono text-brand-text-muted/60">
                ⏱ {{ (testResult.duration_ms / 1000).toFixed(2) }}s
              </span>
              <button
                type="button"
                class="text-xs text-brand-primary/60 hover:text-brand-primary transition-colors flex items-center gap-1"
                @click="copyResponse"
                title="Antwort kopieren"
              >
                <svg v-if="!copiedResponse" xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                {{ copiedResponse ? 'Kopiert!' : 'Kopieren' }}
              </button>
            </div>
          </div>
          <div class="text-xs leading-relaxed whitespace-pre-wrap text-brand-text select-text font-normal">
            {{ testResult.response }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
