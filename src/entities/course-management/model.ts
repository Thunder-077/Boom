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
