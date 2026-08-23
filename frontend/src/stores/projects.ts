import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { Project, Member } from "../api/models";

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

  async function updateProject(pid: string, payload: {
    title?: string;
    description?: string;
    mcp_expose_comments?: boolean;
    ai_provider?: string;
    ai_model?: string;
    ai_system_prompt?: string;
  }) {
    const response = await api.put(`/projects/${pid}`, payload);
    const idx = projects.value.findIndex((p) => p.pid === pid);
    if (idx !== -1) projects.value[idx] = response.data;
    if (currentProject.value?.pid === pid) currentProject.value = response.data;
    return response.data;
  }

  async function testProjectAi(pid: string, payload: { prompt: string; model?: string; system_prompt?: string }) {
    const response = await api.post(`/projects/${pid}/test-ai`, payload);
    return response.data;
  }

  async function deleteProject(pid: string) {
    await api.delete(`/projects/${pid}`);
    projects.value = projects.value.filter((p) => p.pid !== pid);
  }

  async function fetchMembers(projectPid: string): Promise<Member[]> {
    const { data } = await api.get(`/projects/${projectPid}/members`);
    return data;
  }

  async function addMember(projectPid: string, email: string): Promise<Member> {
    const { data } = await api.post(`/projects/${projectPid}/members`, { email });
    return data;
  }

  async function removeMember(projectPid: string, userPid: string): Promise<void> {
    await api.delete(`/projects/${projectPid}/members/${userPid}`);
  }

  return {
    projects,
    currentProject,
    loading,
    error,
    fetchProjects,
    createProject,
    fetchProjectByPid,
    updateProject,
    testProjectAi,
    deleteProject,
    fetchMembers,
    addMember,
    removeMember,
  };
});
