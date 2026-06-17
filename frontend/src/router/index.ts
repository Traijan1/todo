import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import RegisterView from "../views/RegisterView.vue";
import LoginView from "../views/LoginView.vue";
import ProjectCreate from "../views/projects/Create.vue";
import ProjectIndex from "../views/projects/Index.vue";
import ProjectShow from "../views/projects/Show.vue";
import ProjectSettings from "../views/projects/Settings.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },
    {
      path: "/register",
      name: "register",
      component: RegisterView,
    },
    {
      path: "/login",
      name: "login",
      component: LoginView,
    },
    {
      path: "/projects",
      name: "projects",
      component: ProjectIndex,
    },
    {
      path: "/projects/new",
      name: "project-new",
      component: ProjectCreate,
    },
    {
      path: "/projects/:pid",
      name: "project-detail",
      component: ProjectShow,
      props: true,
    },
    {
      path: "/projects/:pid/settings",
      name: "project-settings",
      component: ProjectSettings,
      props: true,
    },
  ],
});

export default router;
