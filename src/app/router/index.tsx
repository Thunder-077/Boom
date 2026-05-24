import { lazy, Suspense } from "react";
import { Navigate, Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { AppShell, TopHeader, type RailItem, type SecondaryNavItem } from "../../widgets/layout/index.react";
import SettingsPanel from "../../features/settings/ui/SettingsPanel";
import UpdatePanel from "../../features/settings/ui/UpdatePanel";
import TeacherListPanel from "../../features/teachers/ui/TeacherListPanel";
import ClassConfigPanel from "../../features/classes/ui/ClassConfigPanel";
import ScoreManagementPanel from "../../features/scores/ui/ScoreManagementPanel";
import CourseManagementPanel from "../../features/course-management/ui/CourseManagementPanel";
import CourseSubstitutionPanel from "../../features/course-management/ui/CourseSubstitutionPanel";
import CourseWorkloadPanel from "../../features/course-management/ui/CourseWorkloadPanel";
import ExamAssignmentPanel from "../../features/dashboard/ui/ExamAssignmentPanel";
import MonitorDrawPanel from "../../features/monitor-draw/ui/MonitorDrawPanel";
import MonitorConfigPanel from "../../features/invigilation/ui/MonitorConfigPanel";

const PdfEditorPanel = lazy(() => import("../../features/pdf-editor/ui/PdfEditorPanel"));

export type AppSection =
  | "exam-assignment"
  | "monitor-draw"
  | "monitor-config"
  | "teachers"
  | "scores"
  | "classes"
  | "course-management"
  | "course-substitution"
  | "course-workload"
  | "pdf-editor"
  | "appearance"
  | "update";

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
  "pdf-editor": {
    title: "文档工具",
    description: "PDF 查看、批注与覆盖式编辑",
    breadcrumb: "文档工具 / PDF 编辑",
    pageTitle: "PDF 编辑",
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

const SECTIONS = Object.keys(pageMap) as AppSection[];
const RAIL_ITEMS: RailItem[] = [
  { key: "students", label: "学生模块", icon: "person" },
  { key: "teachers", label: "教师模块", icon: "badge" },
  { key: "classes", label: "班级模块", icon: "domain" },
  { key: "academic", label: "教务模块", icon: "school" },
  { key: "dashboard", label: "考试模块", icon: "event_note" },
  { key: "documents", label: "文档工具", icon: "edit_file" },
];

const SECONDARY_GROUPS: Record<string, { title: string; items: SecondaryNavItem[] }> = {
  students: {
    title: pageMap.scores.title,
    items: [{ key: "scores", label: "成绩管理", icon: "assignment" }],
  },
  teachers: {
    title: pageMap.teachers.title,
    items: [{ key: "teachers", label: "教师列表", icon: "badge" }],
  },
  classes: {
    title: pageMap.classes.title,
    items: [{ key: "classes", label: "班级配置", icon: "settings" }],
  },
  academic: {
    title: pageMap["course-management"].title,
    items: [
      { key: "course-management", label: "课务管理", icon: "calendar_month" },
      { key: "course-substitution", label: "调代课管理", icon: "published_with_changes" },
      { key: "course-workload", label: "课时统计", icon: "query_stats" },
    ],
  },
  dashboard: {
    title: pageMap["exam-assignment"].title,
    items: [
      { key: "exam-assignment", label: "考场分配", icon: "inventory_2" },
      { key: "monitor-draw", label: "监考抽签", icon: "shuffle" },
      { key: "monitor-config", label: "监考配置", icon: "tune" },
    ],
  },
  documents: {
    title: pageMap["pdf-editor"].title,
    items: [{ key: "pdf-editor", label: "PDF 编辑", icon: "edit_file" }],
  },
  settings: {
    title: pageMap.appearance.title,
    items: [
      { key: "appearance", label: "配色主题", icon: "palette" },
      { key: "update", label: "版本与更新", icon: "system_update" },
    ],
  },
};

const SECTION_TO_RAIL: Record<AppSection, string> = {
  teachers: "teachers",
  scores: "students",
  classes: "classes",
  "course-management": "academic",
  "course-substitution": "academic",
  "course-workload": "academic",
  "exam-assignment": "dashboard",
  "monitor-draw": "dashboard",
  "monitor-config": "dashboard",
  "pdf-editor": "documents",
  appearance: "settings",
  update: "settings",
};

function normalizeSection(rawSection: string | undefined): AppSection {
  return SECTIONS.includes(rawSection as AppSection) ? (rawSection as AppSection) : "exam-assignment";
}

function AppSectionRoute() {
  const location = useLocation();
  const navigate = useNavigate();
  const section = normalizeSection(location.pathname.replace(/^\/app\/?/, "").split("/")[0]);
  const pageCopy = pageMap[section];
  const activeRail = SECTION_TO_RAIL[section];
  const secondaryGroup = SECONDARY_GROUPS[activeRail];

  function navigateByRail(key: string) {
    if (key === "dashboard") {
      navigate("/app/exam-assignment");
      return;
    }
    if (key === "students") {
      navigate("/app/scores");
      return;
    }
    if (key === "teachers") {
      navigate("/app/teachers");
      return;
    }
    if (key === "academic") {
      navigate("/app/course-management");
      return;
    }
    if (key === "documents") {
      navigate("/app/pdf-editor");
      return;
    }
    navigate("/app/classes");
  }

  return (
    <div className="page-bg">
      <AppShell
        railItems={RAIL_ITEMS}
        activeRail={activeRail === "settings" ? "" : activeRail}
        secondaryTitle={secondaryGroup.title}
        secondaryDescription={pageCopy.description}
        secondaryItems={secondaryGroup.items}
        activeSecondary={section}
        isSettingsActive={section === "appearance" || section === "update"}
        onSelectRail={navigateByRail}
        onSelectSecondary={(key) => navigate(`/app/${key}`)}
        onOpenSettings={() => navigate("/app/appearance")}
      >
        <TopHeader
          breadcrumb={pageCopy.breadcrumb}
          title={pageCopy.pageTitle}
          summary={pageCopy.summary}
          compact={section === "monitor-config"}
        />
        {section === "teachers" ? <TeacherListPanel /> : null}
        {section === "scores" ? <ScoreManagementPanel /> : null}
        {section === "classes" ? <ClassConfigPanel /> : null}
        {section === "course-management" ? <CourseManagementPanel /> : null}
        {section === "course-substitution" ? <CourseSubstitutionPanel /> : null}
        {section === "course-workload" ? <CourseWorkloadPanel /> : null}
        {section === "exam-assignment" ? <ExamAssignmentPanel /> : null}
        {section === "monitor-draw" ? <MonitorDrawPanel /> : null}
        {section === "monitor-config" ? <MonitorConfigPanel /> : null}
        {section === "pdf-editor" ? (
          <Suspense fallback={<div className="pdf-loading-panel">正在载入 PDF 编辑器...</div>}>
            <PdfEditorPanel />
          </Suspense>
        ) : null}
        {section === "appearance" ? <SettingsPanel /> : null}
        {section === "update" ? <UpdatePanel /> : null}
      </AppShell>
    </div>
  );
}

export function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/app/exam-assignment" replace />} />
      <Route path="/dashboard/*" element={<Navigate to="/app/exam-assignment" replace />} />
      <Route path="/management/*" element={<Navigate to="/app/teachers" replace />} />
      <Route path="/app/*" element={<AppSectionRoute />} />
      <Route path="*" element={<Navigate to="/app/exam-assignment" replace />} />
    </Routes>
  );
}
