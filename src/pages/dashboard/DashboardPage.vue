<template>
  <div class="page-bg">
    <AppShell
      :rail-items="railItems"
      :active-rail="'dashboard'"
      :secondary-title="pageCopy.secondaryTitle"
      :secondary-description="pageCopy.secondaryDescription"
      :secondary-items="secondaryItems"
      :active-secondary="activeSecondary"
      @select-rail="onRailSelect"
      @select-secondary="onSecondarySelect"
    >
      <TopHeader :breadcrumb="pageCopy.breadcrumb" :title="pageCopy.pageTitle" :compact="activeSecondary === 'monitor-config'" />
      <ExamDashboardPanel v-if="activeSecondary === 'exam-assignment'" />
      <InvigilationPanel v-else />
    </AppShell>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import type { DashboardSection } from "../../app/router";
import ExamDashboardPanel from "../../features/dashboard/ui/ExamDashboardPanel.vue";
import InvigilationPanel from "../../features/invigilation/ui/InvigilationPanel.vue";
import AppShell from "../../widgets/layout/AppShell.vue";
import TopHeader from "../../widgets/layout/TopHeader.vue";
import type { RailItem, SecondaryNavItem } from "../../widgets/layout/types";

const props = withDefaults(
  defineProps<{
    section?: DashboardSection;
  }>(),
  {
    section: "exam-assignment",
  },
);

const router = useRouter();

const railItems: RailItem[] = [
  { key: "students", label: "学生模块", icon: "person" },
  { key: "teachers", label: "教师模块", icon: "badge" },
  { key: "classes", label: "班级模块", icon: "domain" },
  { key: "dashboard", label: "考试模块", icon: "event_note" },
];

const secondaryItems: SecondaryNavItem[] = [
  { key: "exam-assignment", label: "月考考场", icon: "inventory_2" },
  { key: "monitor-draw", label: "监考抽签", icon: "shuffle" },
  { key: "monitor-config", label: "监考配置", icon: "tune" },
];

const activeSecondary = computed(() => (props.section === "monitor-config" ? "monitor-config" : "exam-assignment"));

const pageCopy = computed(() => {
  if (activeSecondary.value === "monitor-config") {
    return {
      secondaryTitle: "考试管理",
      secondaryDescription: "监考配置与津贴规则设置",
      breadcrumb: "考试管理 / 监考配置",
      pageTitle: "监考配置",
    };
  }
  return {
    secondaryTitle: "考试管理",
    secondaryDescription: "月考考场与监考安排配置",
    breadcrumb: "考试管理 / 月考考场",
    pageTitle: "月考考场",
  };
});

function onRailSelect(key: string) {
  if (key === "dashboard") {
    return;
  }
  if (key === "students") {
    void router.push("/management/scores");
    return;
  }
  if (key === "teachers") {
    void router.push("/management/teachers");
    return;
  }
  void router.push("/management/classes");
}

function onSecondarySelect(key: string) {
  if (key === "exam-assignment") {
    void router.push("/dashboard");
    return;
  }
  if (key === "monitor-config") {
    void router.push("/dashboard/monitor-config");
  }
}
</script>

<style scoped>
.page-bg {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: flex-start;
}
</style>
