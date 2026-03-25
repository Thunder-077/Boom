import { invoke } from "@tauri-apps/api/core";
import type {
  ExamSessionTime,
  ExamSessionTimeUpsert,
  SpaceStaffRequirement,
  SpaceStaffRequirementUpsert,
  GenerateLatestExamStaffPlanResult,
  ExamStaffPlanOverview,
  ExamStaffTask,
  ExamStaffTaskQuery,
  TeacherDutyStat,
  InvigilationConfig,
  ExamStaffExclusion,
  ExamStaffExclusionCreatePayload,
} from "../../entities/exam-plan/model";
import type { ListResult } from "../../shared/types/api";
import type { TeacherRow } from "../../entities/teacher/model";

export interface InvigilationService {
  listSessionTimes(): Promise<ExamSessionTime[]>;
  upsertSessionTimes(items: ExamSessionTimeUpsert[]): Promise<{ success: boolean }>;
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
}

export const invigilationService: InvigilationService = {
  listSessionTimes() {
    return invoke<ExamSessionTime[]>("list_exam_session_times");
  },
  upsertSessionTimes(items) {
    return invoke<{ success: boolean }>("upsert_exam_session_times", { items });
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
};
