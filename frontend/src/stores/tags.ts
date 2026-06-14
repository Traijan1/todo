import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api/client";
import type { Tag } from "../api/models";

export const useTagStore = defineStore("tags", () => {
  const tags = ref<Tag[]>([]);
  const loading = ref(false);

  async function fetchTags() {
    loading.value = true;
    try {
      const res = await api.get("/tags");
      tags.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function createTag(title: string, color?: string): Promise<Tag> {
    const res = await api.post("/tags", { title, color });
    tags.value.push(res.data);
    return res.data;
  }

  async function deleteTag(pid: string) {
    await api.delete(`/tags/${pid}`);
    tags.value = tags.value.filter((t) => t.pid !== pid);
  }

  return { tags, loading, fetchTags, createTag, deleteTag };
});
