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
} from "../../entities/exam-plan/model";
import type { ListResult } from "../../shared/types/api";

export interface InvigilationService {
  listSessionTimes(): Promise<ExamSessionTime[]>;
  upsertSessionTimes(items: ExamSessionTimeUpsert[]): Promise<{ success: boolean }>;
  listSpaceStaffRequirements(sessionId: number): Promise<SpaceStaffRequirement[]>;
  upsertSpaceStaffRequirements(sessionId: number, items: SpaceStaffRequirementUpsert[]): Promise<{ success: boolean }>;
  generateStaffPlan(): Promise<GenerateLatestExamStaffPlanResult>;
  getStaffPlanOverview(): Promise<ExamStaffPlanOverview>;
  listStaffTasks(params: ExamStaffTaskQuery): Promise<ListResult<ExamStaffTask>>;
  listTeacherDutyStats(params?: { keyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherDutyStat>>;
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
};
