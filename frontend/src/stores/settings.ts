import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { UserSettings, OllamaModel, AiTestResult } from "../api/models";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<UserSettings>({
    ollama_url: "",
    default_model: undefined,
  });

  const models = ref<OllamaModel[]>([]);
  const loading = ref(false);
  const saving = ref(false);
  const testingConnection = ref(false);
  const testingPrompt = ref(false);
  const connectionStatus = ref<"idle" | "connected" | "error">("idle");
  const connectionError = ref<string | null>(null);

  function resetConnectionState(clearModels = false) {
    connectionStatus.value = "idle";
    connectionError.value = null;
    if (clearModels) models.value = [];
  }

  async function fetchSettings() {
    loading.value = true;
    try {
      const response = await api.get("/settings");
      settings.value = response.data;
      return response.data;
    } catch (err: any) {
      console.error("Failed to load settings:", err);
    } finally {
      loading.value = false;
    }
  }

  async function updateSettings(payload: { ollama_url: string; default_model?: string }) {
    saving.value = true;
    try {
      const response = await api.put("/settings", payload);
      settings.value = response.data;
      return response.data;
    } finally {
      saving.value = false;
    }
  }

  async function fetchOllamaModels(ollama_url?: string): Promise<OllamaModel[]> {
    testingConnection.value = true;
    connectionError.value = null;
    try {
      const targetUrl = ollama_url || settings.value.ollama_url;
      const response = await api.post("/settings/ollama/models", {
        ollama_url: targetUrl,
      });
      models.value = response.data.models || [];
      connectionStatus.value = "connected";
      return models.value;
    } catch (err: any) {
      connectionStatus.value = "error";
      connectionError.value =
        err.response?.data?.description ||
        err.response?.data?.error ||
        err.message ||
        "Verbindung zu Ollama fehlgeschlagen";
      models.value = [];
      throw err;
    } finally {
      testingConnection.value = false;
    }
  }

  async function testOllamaPrompt(payload: {
    prompt: string;
    model: string;
    system_prompt?: string;
    ollama_url?: string;
  }): Promise<AiTestResult> {
    testingPrompt.value = true;
    try {
      const response = await api.post("/settings/ollama/test-prompt", payload);
      return response.data;
    } finally {
      testingPrompt.value = false;
    }
  }

  return {
    settings,
    models,
    loading,
    saving,
    testingConnection,
    testingPrompt,
    connectionStatus,
    connectionError,
    resetConnectionState,
    fetchSettings,
    updateSettings,
    fetchOllamaModels,
    testOllamaPrompt,
  };
});
