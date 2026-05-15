import { invoke } from "@tauri-apps/api/core";
import type {
  CourseClassOption,
  CourseImportBatch,
  CourseImportResult,
  CourseImportSettingsPayload,
  CourseScheduleQuery,
  CourseScheduleView,
  CourseSummary,
} from "../../entities/course-management/model";

export interface CourseManagementService {
  importExcel(filePath: string): Promise<CourseImportResult>;
  getSummary(): Promise<CourseSummary>;
  listImports(): Promise<CourseImportBatch[]>;
  updateImportSettings(payload: CourseImportSettingsPayload): Promise<CourseImportBatch>;
  deleteImport(importId: number): Promise<void>;
  listClasses(classType: "admin" | "foreign", importId?: number): Promise<CourseClassOption[]>;
  listTeachers(importId?: number): Promise<string[]>;
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
  getScheduleView(query) {
    return invoke<CourseScheduleView>("get_course_schedule_view", { query });
  },
};
