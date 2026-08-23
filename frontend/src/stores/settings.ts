import { computed, ref } from "vue";
import { defineStore } from "pinia";
import api from "../api/client";
import type { AiModel, AiSettings, AiTestResult } from "../api/models";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AiSettings>({
    default_provider: "",
    providers: [],
  });
  const activeProviderId = ref("");
  const modelsByProvider = ref<Record<string, AiModel[]>>({});
  const loading = ref(false);
  const testingConnection = ref(false);
  const testingPrompt = ref(false);
  const connectionStatus = ref<"idle" | "connected" | "error">("idle");
  const connectionError = ref<string | null>(null);

  const models = computed(() => modelsByProvider.value[activeProviderId.value] || []);

  function modelsFor(providerId: string): AiModel[] {
    return modelsByProvider.value[providerId] || [];
  }

  function activateProvider(providerId: string) {
    activeProviderId.value = providerId;
    connectionStatus.value = modelsFor(providerId).length ? "connected" : "idle";
    connectionError.value = null;
  }

  function resetConnectionState(clearModels = false) {
    connectionStatus.value = "idle";
    connectionError.value = null;
    if (clearModels && activeProviderId.value) {
      delete modelsByProvider.value[activeProviderId.value];
    }
  }

  async function fetchSettings(): Promise<AiSettings | undefined> {
    loading.value = true;
    try {
      const response = await api.get("/settings");
      settings.value = response.data;
      if (!activeProviderId.value) {
        activeProviderId.value = response.data.default_provider;
      }
      return response.data;
    } catch (err: any) {
      console.error("Failed to load AI settings:", err);
    } finally {
      loading.value = false;
    }
  }

  async function fetchModels(providerId = activeProviderId.value): Promise<AiModel[]> {
    if (!providerId) return [];
    activeProviderId.value = providerId;
    testingConnection.value = true;
    connectionError.value = null;
    try {
      const response = await api.post("/settings/ai/models", {
        provider_id: providerId,
      });
      modelsByProvider.value[providerId] = response.data.models || [];
      connectionStatus.value = "connected";
      return modelsByProvider.value[providerId];
    } catch (err: any) {
      connectionStatus.value = "error";
      connectionError.value =
        err.response?.data?.description ||
        err.response?.data?.error ||
        err.message ||
        "Verbindung zum AI-Provider fehlgeschlagen";
      modelsByProvider.value[providerId] = [];
      throw err;
    } finally {
      testingConnection.value = false;
    }
  }

  async function testPrompt(payload: {
    provider_id: string;
    prompt: string;
    model: string;
    system_prompt?: string;
  }): Promise<AiTestResult> {
    testingPrompt.value = true;
    try {
      const response = await api.post("/settings/ai/test-prompt", payload);
      return response.data;
    } finally {
      testingPrompt.value = false;
    }
  }

  return {
    settings,
    activeProviderId,
    models,
    loading,
    testingConnection,
    testingPrompt,
    connectionStatus,
    connectionError,
    modelsFor,
    activateProvider,
    resetConnectionState,
    fetchSettings,
    fetchModels,
    testPrompt,
  };
});
