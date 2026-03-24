import { invoke } from "@tauri-apps/api/core";
import type {
  TeacherImportResult,
  TeacherListResult,
  TeacherQuery,
  TeacherSummary,
} from "../../entities/teacher/model";

export interface TeacherService {
  list(params: TeacherQuery): Promise<TeacherListResult>;
  getSummary(): Promise<TeacherSummary>;
  importExcel(filePath: string): Promise<TeacherImportResult>;
}

export const teacherService: TeacherService = {
  list(params) {
    return invoke<TeacherListResult>("list_latest_teachers", { params });
  },
  getSummary() {
    return invoke<TeacherSummary>("get_latest_teacher_summary");
  },
  importExcel(filePath) {
    return invoke<TeacherImportResult>("import_teachers_from_excel", { filePath });
  },
};
