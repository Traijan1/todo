import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { Project } from "../api/models";

export const useProjectStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
  const currentProject = ref<Project | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchProjects() {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.get("/projects");
      projects.value = response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to fetch projects";
    } finally {
      loading.value = false;
    }
  }

  async function createProject(payload: { title: string; description?: string }) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.post("/projects", payload);
      projects.value.push(response.data);
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to create project";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function fetchProjectByPid(pid: string) {
    loading.value = true;
    error.value = null;
    try {
      const response = await api.get(`/projects/${pid}`);
      currentProject.value = response.data;
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Failed to fetch project";
    } finally {
      loading.value = false;
    }
  }

  return {
    projects,
    currentProject,
    loading,
    error,
    fetchProjects,
    createProject,
    fetchProjectByPid,
  };
});
