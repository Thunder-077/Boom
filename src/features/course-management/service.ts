import { invoke } from "@tauri-apps/api/core";
import type {
  CourseClassOption,
  CourseImportBatch,
  CourseImportResult,
  CourseImportSettingsPayload,
  CourseScheduleChange,
  CoursePeriodSlot,
  CourseScheduleQuery,
  CourseScheduleView,
  CourseSubstitutionCandidate,
  CourseSubstitutionCandidateQuery,
  CourseSummary,
  SaveCourseSubstitutionsPayload,
  CourseWorkloadQuery,
  CourseWorkloadReport,
  ExportCourseWorkloadResult,
} from "../../entities/course-management/model";

export interface CourseManagementService {
  importExcel(filePath: string): Promise<CourseImportResult>;
  getSummary(): Promise<CourseSummary>;
  listImports(): Promise<CourseImportBatch[]>;
  updateImportSettings(payload: CourseImportSettingsPayload): Promise<CourseImportBatch>;
  deleteImport(importId: number): Promise<void>;
  listClasses(classType: "admin" | "foreign", importId?: number): Promise<CourseClassOption[]>;
  listTeachers(importId?: number): Promise<string[]>;
  listPeriods(importId?: number): Promise<CoursePeriodSlot[]>;
  listSubstitutionCandidates(query: CourseSubstitutionCandidateQuery): Promise<CourseSubstitutionCandidate[]>;
  saveSubstitutions(payload: SaveCourseSubstitutionsPayload): Promise<CourseScheduleChange[]>;
  listScheduleChanges(importId?: number): Promise<CourseScheduleChange[]>;
  revokeScheduleChange(changeId: number): Promise<void>;
  getWorkloadReport(query: CourseWorkloadQuery): Promise<CourseWorkloadReport>;
  exportWorkloadReport(query: CourseWorkloadQuery): Promise<ExportCourseWorkloadResult>;
  getScheduleView(query: CourseScheduleQuery): Promise<CourseScheduleView>;
}

export const courseManagementService: CourseManagementService = {
  importExcel(filePath) {
    return invoke<CourseImportResult>("import_course_schedule_from_excel", { filePath });
  },
  getSummary() {
    return invoke<CourseSummary>("get_course_schedule_summary");
  },
  listImports() {
    return invoke<CourseImportBatch[]>("list_course_schedule_imports");
  },
  updateImportSettings(payload) {
    return invoke<CourseImportBatch>("update_course_schedule_import_settings", { payload });
  },
  deleteImport(importId) {
    return invoke<void>("delete_course_schedule_import", { importId });
  },
  listClasses(classType, importId) {
    return invoke<CourseClassOption[]>("list_course_schedule_classes", { classType, importId });
  },
  listTeachers(importId) {
    return invoke<string[]>("list_course_schedule_teachers", { importId });
  },
  listPeriods(importId) {
    return invoke<CoursePeriodSlot[]>("list_course_schedule_periods", { importId });
  },
  listSubstitutionCandidates(query) {
    return invoke<CourseSubstitutionCandidate[]>("list_course_substitution_candidates", { query });
  },
  saveSubstitutions(payload) {
    return invoke<CourseScheduleChange[]>("save_course_substitutions", { payload });
  },
  listScheduleChanges(importId) {
    return invoke<CourseScheduleChange[]>("list_course_schedule_changes", { importId });
  },
  revokeScheduleChange(changeId) {
    return invoke<void>("revoke_course_schedule_change", { changeId });
  },
  getWorkloadReport(query) {
    return invoke<CourseWorkloadReport>("get_course_workload_report", { query });
  },
  exportWorkloadReport(query) {
    return invoke<ExportCourseWorkloadResult>("export_course_workload_report", { query });
  },
  getScheduleView(query) {
    return invoke<CourseScheduleView>("get_course_schedule_view", { query });
  },
};
