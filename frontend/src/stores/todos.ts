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

  async function createTodo(boardPid: string, payload: { title: string; details?: string; tags?: string[]; parent_pid?: string }) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.post(`/boards/${boardPid}/todos`, payload);
      console.log(response);
      todos.value.push(response.data);
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to create todo";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function updateTodo(pid: string, payload: { title?: string; details?: string | null; board_pid?: string; tags?: string[]; locked?: boolean; parent_pid?: string }) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.patch(`/todos/${pid}`, payload);
      const index = todos.value.findIndex((t) => t.pid === pid);
      if (index !== -1) {
        todos.value[index] = response.data;
      }
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to update todo";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function deleteTodo(pid: string) {
    loading.value = true;
    error.value = null;

    try {
      const response = await api.delete(`/todos/${pid}`);
      todos.value = todos.value.filter((todo) => todo.pid !== pid);

      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to delete todo";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    todos,
    loading,
    error,
    fetchTodos,
    createTodo,
    updateTodo,
    deleteTodo,
  };
});
