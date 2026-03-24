<template>
  <div class="page-bg">
    <AppShell
      :rail-items="railItems"
      :active-rail="activeRail"
      :secondary-title="pageCopy.title"
      :secondary-description="pageCopy.description"
      :secondary-items="secondaryItems"
      :active-secondary="activeSection"
      @select-rail="onRailSelect"
      @select-secondary="onSecondarySelect"
    >
      <TopHeader :breadcrumb="pageCopy.breadcrumb" :title="pageCopy.pageTitle" />
      <TeacherListPanel v-if="activeSection === 'teachers'" />
      <ScoreManagementPanel v-else-if="activeSection === 'scores'" />
      <ClassConfigPanel v-else />
    </AppShell>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import AppShell from "../../widgets/layout/AppShell.vue";
import TopHeader from "../../widgets/layout/TopHeader.vue";
import type { ManagementSection } from "../../app/router";
import type { RailItem, SecondaryNavItem } from "../../widgets/layout/types";
import TeacherListPanel from "../../features/teachers/ui/TeacherListPanel.vue";
import ScoreManagementPanel from "../../features/scores/ui/ScoreManagementPanel.vue";
import ClassConfigPanel from "../../features/classes/ui/ClassConfigPanel.vue";

const route = useRoute();
const router = useRouter();

const activeSection = computed<ManagementSection>(() => {
  const section = route.params.section;
  if (section === "scores" || section === "classes" || section === "teachers") {
    return section;
  }
  return "teachers";
});

const pageMap: Record<ManagementSection, { title: string; description: string; breadcrumb: string; pageTitle: string }> = {
  teachers: {
    title: "教师管理",
    description: "教师资料与授课班级关系维护",
    breadcrumb: "教师管理 / 教师列表",
    pageTitle: "教师列表",
  },
  scores: {
    title: "学生管理",
    description: "教务核心模块与考试编排入口",
    breadcrumb: "学生管理 / 成绩管理",
    pageTitle: "成绩管理",
  },
  classes: {
    title: "班级管理",
    description: "班级科目与教学楼信息配置",
    breadcrumb: "班级管理 / 班级配置",
    pageTitle: "班级配置",
  },
};

const pageCopy = computed(() => pageMap[activeSection.value]);

const railItems: RailItem[] = [
  { key: "students", label: "学生模块", icon: "person" },
  { key: "teachers", label: "教师模块", icon: "badge" },
  { key: "classes", label: "班级模块", icon: "domain" },
  { key: "dashboard", label: "考试模块", icon: "event_note" },
];

const activeRail = computed(() => {
  if (activeSection.value === "scores") {
    return "students";
  }
  if (activeSection.value === "teachers") {
    return "teachers";
  }
  if (activeSection.value === "classes") {
    return "classes";
  }
  return "dashboard";
});

const secondaryItems = computed<SecondaryNavItem[]>(() => {
  if (activeSection.value === "teachers") {
    return [{ key: "teachers", label: "教师列表", icon: "badge" }];
  }
  if (activeSection.value === "scores") {
    return [{ key: "scores", label: "成绩管理", icon: "assignment" }];
  }
  return [{ key: "classes", label: "班级配置", icon: "settings" }];
});

function onRailSelect(key: string) {
  if (key === "dashboard") {
    void router.push("/dashboard");
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
  if (key === "teachers" || key === "scores" || key === "classes") {
    void router.push(`/management/${key}`);
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
