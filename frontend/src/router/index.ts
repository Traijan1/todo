import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import RegisterView from "../views/RegisterView.vue";
import LoginView from "../views/LoginView.vue";
import NewProjectView from "../views/projects/NewProjectView.vue";
import ProjectIndexView from "../views/projects/ProjectIndexView.vue";
import SingleProjectView from "../views/projects/SingleProjectView.vue";

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
      component: ProjectIndexView,
    },
    {
      path: "/projects/new",
      name: "project-new",
      component: NewProjectView,
    },
    {
      path: "/projects/:pid",
      name: "project-detail",
      component: SingleProjectView,
      props: true,
    },
  ],
});

export default router;
