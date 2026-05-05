import { defineStore } from "pinia";
import { ref, computed } from "vue";
import axios from "axios";

interface User {
  pid: string;
  name: string;
  email?: string;
  is_verified?: boolean;
}

export const useAuthStore = defineStore("auth", () => {
  const token = ref<string | null>(localStorage.getItem("token"));
  const user = ref<User | null>(JSON.parse(localStorage.getItem("user") || "null"));
  const loading = ref(false);
  const error = ref<string | null>(null);

  const isAuthenticated = computed(() => !!token.value);

  function setAuth(newToken: string, userData: User) {
    token.value = newToken;
    user.value = userData;
    localStorage.setItem("token", newToken);
    localStorage.setItem("user", JSON.stringify(userData));
  }

  function logout() {
    token.value = null;
    user.value = null;
    localStorage.removeItem("token");
    localStorage.removeItem("user");
  }

  async function register(payload: any) {
    loading.value = true;
    error.value = null;
    try {
      const response = await axios.post("/api/auth/register", payload);
      // Loco usually returns the new user directly or a token
      if (response.data.token) {
        const { token: newToken, ...userData } = response.data;
        setAuth(newToken, userData);
      }
      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Registration failed";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function login(payload: any) {
    loading.value = true;
    error.value = null;

    try {
      const response = await axios.post("/api/auth/login", payload);

      const { token: newToken, ...userData } = response.data;
      setAuth(newToken, userData as User);

      return response.data;
    } catch (err: any) {
      error.value = err.response?.data?.description || "Login failed";
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    token,
    user,
    isAuthenticated,
    loading,
    error,
    setAuth,
    logout,
    register,
    login,
  };
});
