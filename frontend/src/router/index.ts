import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import RegisterView from "../views/RegisterView.vue";
import LoginView from "../views/LoginView.vue";
import ProjectCreate from "../views/projects/Create.vue";
import ProjectIndex from "../views/projects/Index.vue";
import ProjectShow from "../views/projects/Show.vue";

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
      path: "/about",
      name: "about",
      component: () => import("../views/AboutView.vue"),
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
  ],
});

export default router;
