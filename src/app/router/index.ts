import { createRouter, createWebHistory } from "vue-router";
import DashboardPage from "../../pages/dashboard/DashboardPage.vue";
import ManagementPage from "../../pages/management/ManagementPage.vue";

export type DashboardSection = "exam-assignment" | "monitor-config";
export type ManagementSection = "teachers" | "scores" | "classes";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/dashboard",
    },
    {
      path: "/dashboard",
      name: "dashboard",
      component: DashboardPage,
      props: {
        section: "exam-assignment",
      },
    },
    {
      path: "/dashboard/monitor-config",
      name: "monitor-config",
      component: DashboardPage,
      props: {
        section: "monitor-config",
      },
    },
    {
      path: "/management",
      redirect: "/management/teachers",
    },
    {
      path: "/management/:section(teachers|scores|classes)",
      name: "management",
      component: ManagementPage,
      props: true,
    },
  ],
});
