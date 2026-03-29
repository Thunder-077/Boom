import { invoke } from "@tauri-apps/api/core";
import type {
  ExamSessionTime,
  ExamSessionTimeUpsert,
  GenerateLatestExamStaffPlanResult,
  GenerateExamStaffPlanPayload,
  InvigilationExclusionSessionOption,
  ExamStaffPlanOverview,
  ExamStaffTask,
  ExamStaffTaskQuery,
  TeacherDutyStat,
  ExportLatestInvigilationScheduleResult,
  MonitorDrawImportResult,
} from "../../entities/exam-plan/model";
import type { ListResult } from "../../shared/types/api";
import type { TeacherRow } from "../../entities/teacher/model";

export interface InvigilationService {
  listSessionTimes(): Promise<ExamSessionTime[]>;
  upsertSessionTimes(items: ExamSessionTimeUpsert[]): Promise<{ success: boolean }>;
  generateStaffPlan(payload: GenerateExamStaffPlanPayload): Promise<GenerateLatestExamStaffPlanResult>;
  getStaffPlanOverview(): Promise<ExamStaffPlanOverview>;
  listStaffTasks(params: ExamStaffTaskQuery): Promise<ListResult<ExamStaffTask>>;
  listTeacherDutyStats(params?: { keyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherDutyStat>>;
  listInvigilationExclusionSessionOptions(): Promise<InvigilationExclusionSessionOption[]>;
  listTeachers(params?: { nameKeyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherRow>>;
  exportLatestInvigilationSchedule(): Promise<ExportLatestInvigilationScheduleResult>;
  importMonitorDrawPairsExcel(filePath: string): Promise<MonitorDrawImportResult>;
}

export const invigilationService: InvigilationService = {
  listSessionTimes() {
    return invoke<ExamSessionTime[]>("list_exam_session_times");
  },
  upsertSessionTimes(items) {
    return invoke<{ success: boolean }>("upsert_exam_session_times", { items });
  },
  generateStaffPlan(payload) {
    return invoke<GenerateLatestExamStaffPlanResult>("generate_latest_exam_staff_plan", { payload });
  },
  getStaffPlanOverview() {
    return invoke<ExamStaffPlanOverview>("get_latest_exam_staff_plan_overview");
  },
  listStaffTasks(params) {
    return invoke<ListResult<ExamStaffTask>>("list_latest_exam_staff_tasks", { params });
  },
  listTeacherDutyStats(params = {}) {
    return invoke<ListResult<TeacherDutyStat>>("list_latest_teacher_duty_stats", { params });
  },
  listInvigilationExclusionSessionOptions() {
    return invoke<InvigilationExclusionSessionOption[]>("list_invigilation_exclusion_session_options");
  },
  listTeachers(params = {}) {
    return invoke<ListResult<TeacherRow>>("list_latest_teachers", { params });
  },
  exportLatestInvigilationSchedule() {
    return invoke<ExportLatestInvigilationScheduleResult>("export_latest_invigilation_schedule");
  },
  importMonitorDrawPairsExcel(filePath) {
    return invoke<MonitorDrawImportResult>("import_monitor_draw_pairs_from_excel", { filePath });
  },
};
