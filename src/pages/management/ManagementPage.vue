<template>
  <div class="page-bg">
    <AppShell
      :rail-items="railItems"
      :active-rail="activeRail"
      :secondary-title="pageCopy.title"
      :secondary-description="pageCopy.description"
      :secondary-items="secondaryItems"
      :active-secondary="activeSection"
      :is-settings-active="activeSection === 'appearance' || activeSection === 'update'"
      @select-rail="onRailSelect"
      @select-secondary="onSecondarySelect"
      @open-settings="openSettings"
    >
      <TopHeader
        :breadcrumb="pageCopy.breadcrumb"
        :title="pageCopy.pageTitle"
        :summary="pageCopy.summary"
        :compact="activeSection === 'monitor-config'"
      />
      <TeacherListPanel v-if="activeSection === 'teachers'" />
      <ScoreManagementPanel v-else-if="activeSection === 'scores'" />
      <ClassConfigPanel v-else-if="activeSection === 'classes'" />
      <CourseManagementPanel v-else-if="activeSection === 'course-management'" />
      <CourseSubstitutionPanel v-else-if="activeSection === 'course-substitution'" />
      <CourseWorkloadPanel v-else-if="activeSection === 'course-workload'" />
      <ExamDashboardPanel v-else-if="activeSection === 'exam-assignment'" />
      <AppearancePanel v-else-if="activeSection === 'appearance'" />
      <UpdatePanel v-else-if="activeSection === 'update'" />
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
import CourseManagementPanel from "../../features/course-management/ui/CourseManagementPanel.vue";
import CourseSubstitutionPanel from "../../features/course-management/ui/CourseSubstitutionPanel.vue";
import CourseWorkloadPanel from "../../features/course-management/ui/CourseWorkloadPanel.vue";
import ExamDashboardPanel from "../../features/dashboard/ui/ExamDashboardPanel.vue";
import InvigilationPanel from "../../features/invigilation/ui/InvigilationPanel.vue";
import MonitorDrawPanel from "../../features/monitor-draw/ui/MonitorDrawPanel.vue";
import ScoreManagementPanel from "../../features/scores/ui/ScoreManagementPanel.vue";
import AppearancePanel from "../../features/settings/ui/SettingsPanel.vue";
import UpdatePanel from "../../features/settings/ui/UpdatePanel.vue";
import TeacherListPanel from "../../features/teachers/ui/TeacherListPanel.vue";

const route = useRoute();
const router = useRouter();

const activeSection = computed<AppSection>(() => {
  const section = route.params.section as string;
  const validSections: AppSection[] = ["teachers", "scores", "classes", "course-management", "course-substitution", "course-workload", "exam-assignment", "monitor-draw", "monitor-config", "appearance", "update"];
  if (validSections.includes(section as AppSection)) {
    return section as AppSection;
  }
  return "exam-assignment";
});

const pageMap: Record<AppSection, { title: string; description: string; breadcrumb: string; pageTitle: string; summary: string }> = {
  teachers: {
    title: "教师管理",
    description: "教师资料与授课班级关系维护",
    breadcrumb: "教师管理 / 教师列表",
    pageTitle: "教师列表",
    summary: "",
  },
  scores: {
    title: "学生管理",
    description: "教务核心模块与考试编排入口",
    breadcrumb: "学生管理 / 成绩管理",
    pageTitle: "成绩管理",
    summary: "",
  },
  classes: {
    title: "班级管理",
    description: "班级科目与教学楼信息配置",
    breadcrumb: "班级管理 / 班级配置",
    pageTitle: "班级配置",
    summary: "",
  },
  "course-management": {
    title: "教务管理",
    description: "课表导入、课务关系同步与课表查看",
    breadcrumb: "教务管理 / 课务管理",
    pageTitle: "课务管理",
    summary: "",
  },
  "course-substitution": {
    title: "教务管理",
    description: "教师请假、临时换课与代课记录维护",
    breadcrumb: "教务管理 / 调代课管理",
    pageTitle: "调代课管理",
    summary: "",
  },
  "course-workload": {
    title: "教务管理",
    description: "教师实际课时明细、分类汇总与导出",
    breadcrumb: "教务管理 / 课时统计",
    pageTitle: "课时统计",
    summary: "",
  },
  "exam-assignment": {
    title: "考试管理",
    description: "考场分配与监考安排配置",
    breadcrumb: "考试管理 / 考场分配",
    pageTitle: "考场分配",
    summary: "",
  },
  "monitor-draw": {
    title: "考试管理",
    description: "考场分配与监考安排配置",
    breadcrumb: "考试管理 / 监考抽签",
    pageTitle: "监考抽签",
    summary: "",
  },
  "monitor-config": {
    title: "考试管理",
    description: "监考配置与津贴规则设置",
    breadcrumb: "考试管理 / 监考配置",
    pageTitle: "监考配置",
    summary: "",
  },
  appearance: {
    title: "系统设置",
    description: "配色主题与版本更新",
    breadcrumb: "系统设置 / 配色主题",
    pageTitle: "配色主题",
    summary: "",
  },
  update: {
    title: "系统设置",
    description: "配色主题与版本更新",
    breadcrumb: "系统设置 / 版本与更新",
    pageTitle: "版本与更新",
    summary: "",
  },
};

const pageCopy = computed(() => pageMap[activeSection.value] || pageMap["exam-assignment"]);

const railItems: RailItem[] = [
  { key: "students", label: "学生模块", icon: "person" },
  { key: "teachers", label: "教师模块", icon: "badge" },
  { key: "classes", label: "班级模块", icon: "domain" },
  { key: "academic", label: "教务模块", icon: "school" },
  { key: "dashboard", label: "考试模块", icon: "event_note" },
];

const activeRail = computed(() => {
  if (activeSection.value === "appearance" || activeSection.value === "update") {
    return "dashboard";
  }
  if (activeSection.value === "scores") {
    return "students";
  }
  if (activeSection.value === "teachers") {
    return "teachers";
  }
  if (activeSection.value === "classes") {
    return "classes";
  }
  if (activeSection.value === "course-management" || activeSection.value === "course-substitution" || activeSection.value === "course-workload") {
    return "academic";
  }
  return "dashboard";
});

const secondaryItems = computed<SecondaryNavItem[]>(() => {
  if (activeSection.value === "appearance" || activeSection.value === "update") {
    return [
      { key: "appearance", label: "配色主题", icon: "palette" },
      { key: "update", label: "版本与更新", icon: "system_update" },
    ];
  }
  if (activeRail.value === "students") {
    return [{ key: "scores", label: "成绩管理", icon: "assignment" }];
  }
  if (activeRail.value === "teachers") {
    return [{ key: "teachers", label: "教师列表", icon: "badge" }];
  }
  if (activeRail.value === "classes") {
    return [{ key: "classes", label: "班级配置", icon: "settings" }];
  }
  if (activeRail.value === "academic") {
    return [
      { key: "course-management", label: "课务管理", icon: "calendar_month" },
      { key: "course-substitution", label: "调代课管理", icon: "published_with_changes" },
      { key: "course-workload", label: "课时统计", icon: "query_stats" },
    ];
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
  if (key === "academic") {
    void router.push("/app/course-management");
    return;
  }
  void router.push("/app/classes");
}

function onSecondarySelect(key: string) {
  void router.push(`/app/${key}`);
}

function openSettings() {
  void router.push("/app/appearance");
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
