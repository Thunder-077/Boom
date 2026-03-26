import type { Subject } from "../score/model";

export interface ExamAllocationSettings {
  defaultCapacity: number;
  maxCapacity: number;
  examTitle: string;
  examNotices: string[];
  updatedAt: string | null;
}

export interface ExamPlanOverview {
  generatedAt: string | null;
  defaultCapacity: number;
  maxCapacity: number;
  gradeCount: number;
  sessionCount: number;
  examRoomCount: number;
  selfStudyRoomCount: number;
  studentAllocationCount: number;
  warningCount: number;
}

export interface GenerateLatestExamPlanResult {
  generatedAt: string;
  gradeCount: number;
  sessionCount: number;
  warningCount: number;
}

export interface ExamGenerationProgress {
  status: string;
  stage: string;
  stageLabel: string;
  percent: number;
  message: string;
  currentGrade: string | null;
  totalGrades: number;
  completedGrades: number;
  updatedAt: string;
}

export interface ExamPlanSession {
  id: number;
  gradeName: string;
  subject: Subject;
  isForeignGroup: boolean;
  foreignOrder: number | null;
  participantCount: number;
  examRoomCount: number;
  selfStudyRoomCount: number;
}

export type ExamPlanSpaceType = "exam_room" | "self_study_room";
export type ExamPlanSpaceSource = "teaching_class" | "exam_room" | "virtual_backup";
export type ExamAllocationType = "exam" | "self_study";

export interface ExamPlanSpace {
  id: number;
  sessionId: number;
  spaceType: ExamPlanSpaceType;
  spaceSource: ExamPlanSpaceSource;
  gradeName: string;
  subject: Subject;
  spaceName: string;
  originalClassName: string | null;
  building: string;
  floor: string;
  capacity: number | null;
  sortIndex: number;
}

export interface ExamPlanStudentAllocation {
  id: number;
  sessionId: number;
  admissionNo: string;
  studentName: string;
  className: string;
  allocationType: ExamAllocationType;
  spaceId: number | null;
  seatNo: number | null;
  subjectScore: number | null;
}

export interface ExamPlanStaffAssignment {
  id: number;
  sessionId: number;
  spaceId: number;
  teacherName: string;
  assignmentType: string;
  note: string | null;
}

export interface ExamPlanSessionDetail {
  session: ExamPlanSession;
  spaces: ExamPlanSpace[];
  studentAllocations: ExamPlanStudentAllocation[];
  staffAssignments: ExamPlanStaffAssignment[];
}

export interface ExamPlanSessionQuery {
  gradeName?: string;
  subject?: Subject;
  page?: number;
  pageSize?: number;
}

export type StaffRole = "exam_room_invigilator" | "self_study_supervisor" | "floor_rover";
export type TaskStatus = "assigned" | "unassigned";
export type ExamStaffTaskSource = "exam" | "exam_linked_self_study" | "full_self_study";
export type SolverEngine = "cp_sat" | "greedy";
export type OptimalityStatus = "optimal" | "feasible" | "fallback" | "infeasible" | "error";
export type FallbackReason =
  | "timeout"
  | "unknown"
  | "infeasible"
  | "error"
  | "not_better_than_baseline";
export type AssignmentTier = "primary" | "homeroom" | "fallback_pool";

export interface ExamSessionTime {
  sessionId: number;
  gradeName: string;
  subject: Subject;
  startAt: string | null;
  endAt: string | null;
}

export interface ExamSessionTimeUpsert {
  sessionId: number;
  subject: Subject;
  startAt: string;
  endAt: string;
}

export interface GenerateLatestExamStaffPlanResult {
  generatedAt: string;
  taskCount: number;
  assignedCount: number;
  unassignedCount: number;
  imbalanceMinutes: number;
  warningCount: number;
  solverEngine: SolverEngine;
  optimalityStatus: OptimalityStatus;
  solveDurationMs: number;
  fallbackReason: FallbackReason | null;
  fallbackPoolAssignments: number;
  baselineDominated: boolean;
}

export interface ExamStaffPlanOverview {
  generatedAt: string | null;
  sessionCount: number;
  taskCount: number;
  assignedCount: number;
  unassignedCount: number;
  warningCount: number;
  imbalanceMinutes: number;
  solverEngine: SolverEngine;
  optimalityStatus: OptimalityStatus;
  solveDurationMs: number;
  fallbackReason: FallbackReason | null;
  fallbackPoolAssignments: number;
  baselineDominated: boolean;
}

export interface ExamStaffTask {
  id: number;
  sessionId: number | null;
  spaceId: number | null;
  taskSource: ExamStaffTaskSource;
  role: StaffRole;
  gradeName: string;
  subject: Subject;
  spaceName: string;
  floor: string;
  startAt: string;
  endAt: string;
  durationMinutes: number;
  recommendedSubject: Subject | null;
  prioritySubjectChain: Subject[];
  assignmentTier: AssignmentTier | null;
  status: TaskStatus;
  reason: string | null;
  allowanceAmount: number;
  teacherId: number | null;
  teacherName: string | null;
}

export interface ExamStaffTaskQuery {
  sessionId?: number;
  role?: StaffRole;
  status?: TaskStatus;
  page?: number;
  pageSize?: number;
}

export interface TeacherDutyStat {
  teacherId: number;
  teacherName: string;
  indoorMinutes: number;
  outdoorMinutes: number;
  totalMinutes: number;
  taskCount: number;
  examRoomTaskCount: number;
  selfStudyTaskCount: number;
  floorRoverTaskCount: number;
  allowanceTotal: number;
  indoorAllowanceTotal: number;
  outdoorAllowanceTotal: number;
  isMiddleManager: boolean;
}

export interface InvigilationConfig {
  defaultExamRoomRequiredCount: number;
  indoorAllowancePerMinute: number;
  outdoorAllowancePerMinute: number;
  middleManagerDefaultEnabled: boolean;
  middleManagerExceptionTeacherIds: number[];
  selfStudyDate: string;
  selfStudyStartTime: string;
  selfStudyEndTime: string;
}

export interface ExamStaffExclusion {
  teacherId: number;
  teacherName: string;
  sessionId: number;
  sessionLabel: string;
}

export interface SelfStudyClassSubjectConfig {
  classId: number;
  subject: Subject | null;
}

export interface GenerateExamStaffPlanPayload {
  defaultExamRoomRequiredCount: number;
  indoorAllowancePerMinute: number;
  outdoorAllowancePerMinute: number;
  staffExclusions: Array<{ teacherId: number; sessionId: number }>;
}

export interface InvigilationExclusionSessionOption {
  sessionId: number;
  gradeName: string;
  subject: Subject;
  startAt: string;
  endAt: string;
  label: string;
}

export interface ExportLatestExamAllocationBundleResult {
  zipPath: string;
  batchDir: string;
  gradeCount: number;
  fileCount: number;
  exportedAt: string;
}
