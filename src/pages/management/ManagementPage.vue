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
      <TopHeader :breadcrumb="pageCopy.breadcrumb" :title="pageCopy.pageTitle" :compact="activeSection === 'monitor-config'" />
      <TeacherListPanel v-if="activeSection === 'teachers'" />
      <ScoreManagementPanel v-else-if="activeSection === 'scores'" />
      <ClassConfigPanel v-else-if="activeSection === 'classes'" />
      <ExamDashboardPanel v-else-if="activeSection === 'exam-assignment'" />
      <KeepAlive>
        <MonitorDrawPanel v-if="activeSection === 'monitor-draw'" />
      </KeepAlive>
      <InvigilationPanel v-if="activeSection === 'monitor-config'" />
    </AppShell>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import AppShell from "../../widgets/layout/AppShell.vue";
import TopHeader from "../../widgets/layout/TopHeader.vue";
import type { AppSection } from "../../app/router";
import type { RailItem, SecondaryNavItem } from "../../widgets/layout/types";
import ClassConfigPanel from "../../features/classes/ui/ClassConfigPanel.vue";
import ExamDashboardPanel from "../../features/dashboard/ui/ExamDashboardPanel.vue";
import InvigilationPanel from "../../features/invigilation/ui/InvigilationPanel.vue";
import MonitorDrawPanel from "../../features/monitor-draw/ui/MonitorDrawPanel.vue";
import ScoreManagementPanel from "../../features/scores/ui/ScoreManagementPanel.vue";
import TeacherListPanel from "../../features/teachers/ui/TeacherListPanel.vue";

const route = useRoute();
const router = useRouter();

const activeSection = computed<AppSection>(() => {
  const section = route.params.section as string;
  const validSections: AppSection[] = ["teachers", "scores", "classes", "exam-assignment", "monitor-draw", "monitor-config"];
  if (validSections.includes(section as AppSection)) {
    return section as AppSection;
  }
  return "exam-assignment";
});

const pageMap: Record<AppSection, { title: string; description: string; breadcrumb: string; pageTitle: string }> = {
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
  "exam-assignment": {
    title: "考试管理",
    description: "考场分配与监考安排配置",
    breadcrumb: "考试管理 / 考场分配",
    pageTitle: "考场分配",
  },
  "monitor-draw": {
    title: "考试管理",
    description: "考场分配与监考安排配置",
    breadcrumb: "考试管理 / 监考抽签",
    pageTitle: "监考抽签",
  },
  "monitor-config": {
    title: "考试管理",
    description: "监考配置与津贴规则设置",
    breadcrumb: "考试管理 / 监考配置",
    pageTitle: "监考配置",
  },
};

const pageCopy = computed(() => pageMap[activeSection.value] || pageMap["exam-assignment"]);

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
  if (activeRail.value === "students") {
    return [{ key: "scores", label: "成绩管理", icon: "assignment" }];
  }
  if (activeRail.value === "teachers") {
    return [{ key: "teachers", label: "教师列表", icon: "badge" }];
  }
  if (activeRail.value === "classes") {
    return [{ key: "classes", label: "班级配置", icon: "settings" }];
  }
  return [
    { key: "exam-assignment", label: "考场分配", icon: "inventory_2" },
    { key: "monitor-draw", label: "监考抽签", icon: "shuffle" },
    { key: "monitor-config", label: "监考配置", icon: "tune" },
  ];
});

function onRailSelect(key: string) {
  if (key === "dashboard") {
    void router.push("/app/exam-assignment");
    return;
  }
  if (key === "students") {
    void router.push("/app/scores");
    return;
  }
  if (key === "teachers") {
    void router.push("/app/teachers");
    return;
  }
  void router.push("/app/classes");
}

function onSecondarySelect(key: string) {
  void router.push(`/app/${key}`);
}
</script>

<style scoped>
.page-bg {
  min-height: 100%;
  width: 100%;
  display: flex;
  justify-content: flex-start;
  align-items: flex-start;
}
</style>
