import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { Board } from "../api/models";

export const useBoardStore = defineStore("boards", () => {
  const boards = ref<Board[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchBoards(projectPid: string) {
    loading.value = true;
    error.value = null;
    try {
      // Assuming nested routes or query params for project boards
      const response = await api.get(`/projects/${projectPid}/boards`);
      boards.value = response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to fetch boards";
    } finally {
      loading.value = false;
    }
  }

  async function createBoard(projectPid: string, payload: { title: string }) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.post(`/projects/${projectPid}/boards`, payload);
      boards.value.push(response.data);
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to create board";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    boards,
    loading,
    error,
    fetchBoards,
    createBoard,
  };
});
