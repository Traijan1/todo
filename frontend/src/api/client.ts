import axios from "axios";

export let lastMutationTime = 0;

const api = axios.create({
  baseURL: "/api",
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem("token");
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  
  if (config.method && ["post", "put", "delete", "patch"].includes(config.method.toLowerCase())) {
    lastMutationTime = Date.now();
  }
  
  return config;
});

export default api;
