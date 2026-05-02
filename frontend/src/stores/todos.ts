import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { Todo } from "../api/models";

export const useTodoStore = defineStore("todos", () => {
  const todos = ref<Todo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchTodos(boardPid: string) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.get(`/boards/${boardPid}/todos`);
      todos.value = response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to fetch todos";
    } finally {
      loading.value = false;
    }
  }

  async function createTodo(boardPid: string, payload: { title: string; content?: string }) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.post(`/boards/${boardPid}/todos`, payload);
      todos.value.push(response.data);
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to create todo";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function toggleTodo(todoPid: string) {
    try {
      const todo = todos.value.find((t) => t.pid === todoPid);
      if (todo) {
        const response = await api.patch(`/todos/${todoPid}/toggle`);
        todo.done = response.data.done;
      }
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to toggle todo";
    }
  }

  return {
    todos,
    loading,
    error,
    fetchTodos,
    createTodo,
    toggleTodo,
  };
});
