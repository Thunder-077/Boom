import { invoke } from "@tauri-apps/api/core";
import type {
  ExamAllocationSettings,
  ExamGenerationProgress,
  ExamSessionTime,
  ExamSessionTimeUpsert,
  ExamPlanOverview,
  ExamPlanSession,
  ExamPlanSessionDetail,
  ExamPlanSessionQuery,
  SpaceStaffRequirement,
  SpaceStaffRequirementUpsert,
  GenerateLatestExamStaffPlanResult,
  ExamStaffPlanOverview,
  ExamStaffTask,
  ExamStaffTaskQuery,
  TeacherDutyStat,
  ExportLatestExamAllocationBundleResult,
  InvigilationConfig,
  ExamStaffExclusion,
  ExamStaffExclusionCreatePayload,
} from "../../entities/exam-plan/model";
import type { TeacherRow } from "../../entities/teacher/model";
import type { ListResult } from "../../shared/types/api";

export interface ExamAllocationService {
  getSettings(): Promise<ExamAllocationSettings>;
  updateSettings(payload: {
    defaultCapacity: number;
    maxCapacity: number;
    examTitle: string;
    examNotices: string[];
  }): Promise<{ success: boolean }>;
  startGenerate(payload?: { defaultCapacity?: number; maxCapacity?: number }): Promise<{ success: boolean }>;
  getGenerationProgress(): Promise<ExamGenerationProgress>;
  getOverview(): Promise<ExamPlanOverview>;
  listSessions(params: ExamPlanSessionQuery): Promise<ListResult<ExamPlanSession>>;
  getSessionDetail(sessionId: number): Promise<ExamPlanSessionDetail>;
  listSessionTimes(): Promise<ExamSessionTime[]>;
  upsertSessionTimes(items: ExamSessionTimeUpsert[]): Promise<{ success: boolean }>;
  deleteSessionTime(subject: ExamSessionTime["subject"]): Promise<{ success: boolean }>;
  listSpaceStaffRequirements(sessionId: number): Promise<SpaceStaffRequirement[]>;
  upsertSpaceStaffRequirements(sessionId: number, items: SpaceStaffRequirementUpsert[]): Promise<{ success: boolean }>;
  generateStaffPlan(): Promise<GenerateLatestExamStaffPlanResult>;
  getStaffPlanOverview(): Promise<ExamStaffPlanOverview>;
  listStaffTasks(params: ExamStaffTaskQuery): Promise<ListResult<ExamStaffTask>>;
  listTeacherDutyStats(params?: { keyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherDutyStat>>;
  getInvigilationConfig(): Promise<InvigilationConfig>;
  updateInvigilationConfig(payload: {
    defaultExamRoomRequiredCount: number;
    indoorAllowancePerMinute: number;
    outdoorAllowancePerMinute: number;
  }): Promise<{ success: boolean }>;
  listExamStaffExclusions(): Promise<ExamStaffExclusion[]>;
  createExamStaffExclusion(payload: ExamStaffExclusionCreatePayload): Promise<{ success: boolean }>;
  deleteExamStaffExclusion(id: number): Promise<{ success: boolean }>;
  listTeachers(params?: { nameKeyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherRow>>;
  exportLatestExamAllocationBundle(): Promise<ExportLatestExamAllocationBundleResult>;
}

export const examAllocationService: ExamAllocationService = {
  getSettings() {
    return invoke<ExamAllocationSettings>("get_exam_allocation_settings");
  },
  updateSettings(payload) {
    return invoke<{ success: boolean }>("update_exam_allocation_settings", { payload });
  },
  startGenerate(payload) {
    return invoke<{ success: boolean }>("start_generate_latest_exam_plan", { payload });
  },
  getGenerationProgress() {
    return invoke<ExamGenerationProgress>("get_exam_generation_progress");
  },
  getOverview() {
    return invoke<ExamPlanOverview>("get_latest_exam_plan_overview");
  },
  listSessions(params) {
    return invoke<ListResult<ExamPlanSession>>("list_latest_exam_plan_sessions", { params });
  },
  getSessionDetail(sessionId) {
    return invoke<ExamPlanSessionDetail>("get_latest_exam_plan_session_detail", { sessionId });
  },
  listSessionTimes() {
    return invoke<ExamSessionTime[]>("list_exam_session_times");
  },
  upsertSessionTimes(items) {
    return invoke<{ success: boolean }>("upsert_exam_session_times", { items });
  },
  deleteSessionTime(subject) {
    return invoke<{ success: boolean }>("delete_exam_session_time", { subject });
  },
  listSpaceStaffRequirements(sessionId) {
    return invoke<SpaceStaffRequirement[]>("list_exam_space_staff_requirements", { sessionId });
  },
  upsertSpaceStaffRequirements(sessionId, items) {
    return invoke<{ success: boolean }>("upsert_exam_space_staff_requirements", { sessionId, items });
  },
  generateStaffPlan() {
    return invoke<GenerateLatestExamStaffPlanResult>("generate_latest_exam_staff_plan");
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
  getInvigilationConfig() {
    return invoke<InvigilationConfig>("get_invigilation_config");
  },
  updateInvigilationConfig(payload) {
    return invoke<{ success: boolean }>("update_invigilation_config", { payload });
  },
  listExamStaffExclusions() {
    return invoke<ExamStaffExclusion[]>("list_exam_staff_exclusions");
  },
  createExamStaffExclusion(payload) {
    return invoke<{ success: boolean }>("create_exam_staff_exclusion", { payload });
  },
  deleteExamStaffExclusion(id) {
    return invoke<{ success: boolean }>("delete_exam_staff_exclusion", { id });
  },
  listTeachers(params = {}) {
    return invoke<ListResult<TeacherRow>>("list_latest_teachers", { params });
  },
  exportLatestExamAllocationBundle() {
    return invoke<ExportLatestExamAllocationBundleResult>("export_latest_exam_allocation_bundle");
  },
};
