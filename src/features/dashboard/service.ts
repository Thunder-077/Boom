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
  GenerateLatestExamStaffPlanResult,
  GenerateExamStaffPlanPayload,
  InvigilationExclusionSessionOption,
  InvigilationConfig,
  ExamStaffPlanOverview,
  ExamStaffTask,
  ExamStaffTaskQuery,
  ExamStaffExclusion,
  SelfStudyClassSubjectConfig,
  TeacherDutyStat,
  ExportLatestExamAllocationBundleResult,
  ExportLatestInvigilationScheduleResult,
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
  getPersistedInvigilationState(): Promise<{ config: InvigilationConfig; exclusions: ExamStaffExclusion[]; selfStudyClassSubjects: SelfStudyClassSubjectConfig[] }>;
  savePersistedInvigilationConfig(payload: InvigilationConfig): Promise<{ success: boolean }>;
  replacePersistedInvigilationExclusions(items: ExamStaffExclusion[]): Promise<{ success: boolean }>;
  savePersistedSelfStudyClassSubjects(items: SelfStudyClassSubjectConfig[]): Promise<{ success: boolean }>;
  generateStaffPlan(payload: GenerateExamStaffPlanPayload): Promise<GenerateLatestExamStaffPlanResult>;
  getStaffPlanOverview(): Promise<ExamStaffPlanOverview>;
  listStaffTasks(params: ExamStaffTaskQuery): Promise<ListResult<ExamStaffTask>>;
  listTeacherDutyStats(params?: { keyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherDutyStat>>;
  listInvigilationExclusionSessionOptions(): Promise<InvigilationExclusionSessionOption[]>;
  listTeachers(params?: { nameKeyword?: string; page?: number; pageSize?: number }): Promise<ListResult<TeacherRow>>;
  exportLatestExamAllocationBundle(): Promise<ExportLatestExamAllocationBundleResult>;
  exportLatestInvigilationSchedule(): Promise<ExportLatestInvigilationScheduleResult>;
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
  getPersistedInvigilationState() {
    return invoke<{ config: InvigilationConfig; exclusions: ExamStaffExclusion[]; selfStudyClassSubjects: SelfStudyClassSubjectConfig[] }>("get_persisted_invigilation_state");
  },
  savePersistedInvigilationConfig(payload) {
    return invoke<{ success: boolean }>("save_persisted_invigilation_config", { payload });
  },
  replacePersistedInvigilationExclusions(items) {
    return invoke<{ success: boolean }>("replace_persisted_invigilation_exclusions", { items });
  },
  savePersistedSelfStudyClassSubjects(items) {
    return invoke<{ success: boolean }>("save_persisted_self_study_class_subjects", { items });
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
  exportLatestExamAllocationBundle() {
    return invoke<ExportLatestExamAllocationBundleResult>("export_latest_exam_allocation_bundle");
  },
  exportLatestInvigilationSchedule() {
    return invoke<ExportLatestInvigilationScheduleResult>("export_latest_invigilation_schedule");
  },
};
