export type CourseViewType = "teacher" | "admin_class" | "foreign_class";

export interface CourseImportResult {
  importedAt: string;
  entryCount: number;
  teacherCount: number;
  adminClassCount: number;
  foreignClassCount: number;
  durationMs: number;
}

export interface CourseSummary {
  latestImportId: number | null;
  importedAt: string | null;
  entryCount: number;
  teacherCount: number;
  adminClassCount: number;
  foreignClassCount: number;
  effectiveStartDate: string | null;
  effectiveEndDate: string | null;
  startWeek: number;
}

export interface CourseImportBatch {
  id: number;
  importedAt: string;
  sourceFile: string;
  entryCount: number;
  teacherCount: number;
  adminClassCount: number;
  foreignClassCount: number;
  effectiveStartDate: string | null;
  effectiveEndDate: string | null;
  startWeek: number;
}

export interface CourseImportSettingsPayload {
  importId: number;
  effectiveStartDate: string | null;
  effectiveEndDate: string | null;
  startWeek: number;
}

export interface CourseClassOption {
  className: string;
  displayName: string;
  classType: "admin" | "foreign";
}

export interface CourseScheduleEntry {
  weekIndex: number;
  dayOfWeek: number;
  dayLabel: string;
  periodIndex: number;
  periodLabel: string;
  sectionLabel: string;
  subject: string;
  teacherNames: readonly string[];
  className: string;
  displayClassName: string;
  classType: string;
}

export interface CoursePeriodSlot {
  weekIndex: number;
  dayOfWeek: number;
  dayLabel: string;
  periodIndex: number;
  periodLabel: string;
  sectionLabel: string;
}

export interface CourseScheduleQuery {
  viewType: CourseViewType;
  target: string;
  importId?: number;
}

export interface CourseScheduleView {
  importId: number;
  target: string;
  viewType: CourseViewType;
  entries: CourseScheduleEntry[];
  periods: CoursePeriodSlot[];
}

export interface CourseScheduleChange {
  id: number;
  importId: number;
  sourceEntryId: number;
  changeType: "substitute";
  status: "active" | "revoked";
  targetDate: string;
  sourceTeacherName: string;
  actualTeacherName: string;
  reason: string;
  remark: string;
  createdAt: string;
  updatedAt: string;
  revokedAt: string | null;
  weekIndex: number;
  dayOfWeek: number;
  dayLabel: string;
  periodIndex: number;
  periodLabel: string;
  sectionLabel: string;
  subject: string;
  className: string;
  displayClassName: string;
  classType: string;
}

export interface CourseSubstitutionCandidate {
  sourceEntryId: number;
  importId: number;
  targetDate: string;
  weekIndex: number;
  dayOfWeek: number;
  dayLabel: string;
  periodIndex: number;
  periodLabel: string;
  sectionLabel: string;
  subject: string;
  teacherNames: readonly string[];
  sourceTeacherName: string;
  className: string;
  displayClassName: string;
  classType: string;
  existingChange: CourseScheduleChange | null;
}

export interface CourseSubstitutionCandidateQuery {
  importId: number;
  teacherName: string;
  startDate: string;
  endDate: string;
  periodIndexes?: number[];
  startPeriodIndex?: number;
  endPeriodIndex?: number;
}

export interface SaveCourseSubstitutionItem {
  sourceEntryId: number;
  targetDate: string;
  sourceTeacherName: string;
  actualTeacherName: string;
  remark?: string;
}

export interface SaveCourseSubstitutionsPayload {
  importId: number;
  reason: string;
  remark: string;
  items: SaveCourseSubstitutionItem[];
}

export interface CourseWorkloadQuery {
  importId: number;
  startDate: string;
  endDate: string;
  startPeriodIndex?: number;
  endPeriodIndex?: number;
}

export interface CourseWorkloadDetail {
  teacherName: string;
  targetDate: string;
  dayLabel: string;
  periodIndex: number;
  periodLabel: string;
  sectionLabel: string;
  category: "早上" | "上午" | "下午" | "晚上" | string;
  subject: string;
  className: string;
  displayClassName: string;
  originalTeacherName: string;
  actualTeacherName: string;
  isSubstitution: boolean;
  remark: string;
}

export interface CourseWorkloadSummary {
  teacherName: string;
  morningReadingCount: number;
  morningCount: number;
  afternoonCount: number;
  eveningCount: number;
  substitutionCount: number;
  totalCount: number;
}

export interface CourseWorkloadReport {
  importId: number;
  startDate: string;
  endDate: string;
  details: CourseWorkloadDetail[];
  summaries: CourseWorkloadSummary[];
}

export interface ExportCourseWorkloadResult {
  filePath: string;
  exportedAt: string;
}
