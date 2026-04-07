import { createRouter, createWebHistory } from "vue-router";
import ManagementPage from "../../pages/management/ManagementPage.vue";

export type AppSection = "exam-assignment" | "monitor-draw" | "monitor-config" | "teachers" | "scores" | "classes" | "settings";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/app/exam-assignment",
    },
    {
      path: "/dashboard/:pathMatch(.*)*",
      redirect: (to) => {
        if (to.path.includes("monitor-config")) return "/app/monitor-config";
        return "/app/exam-assignment";
      },
    },
    {
      path: "/management/:pathMatch(.*)*",
      redirect: (to) => {
        if (to.path.includes("scores")) return "/app/scores";
        if (to.path.includes("classes")) return "/app/classes";
        return "/app/teachers";
      },
    },
    {
      path: "/app/:section(.*)",
      name: "app-layout",
      component: ManagementPage,
      props: true,
    },
  ],
});
