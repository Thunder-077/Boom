import type { ListResult } from "../../shared/types/api";

export type TeacherSubject =
  | "chinese"
  | "math"
  | "english"
  | "physics"
  | "chemistry"
  | "biology"
  | "politics"
  | "history"
  | "geography"
  | "russian"
  | "japanese"
  | "sports"
  | "music"
  | "general"
  | "information"
  | "fine_arts";

export const TEACHER_SUBJECT_LABELS: Record<TeacherSubject, string> = {
  chinese: "语文",
  math: "数学",
  english: "英语",
  physics: "物理",
  chemistry: "化学",
  biology: "生物",
  politics: "政治",
  history: "历史",
  geography: "地理",
  russian: "俄语",
  japanese: "日语",
  sports: "体育",
  music: "音乐",
  general: "通用",
  information: "信息",
  fine_arts: "美术",
};

export interface TeacherRow {
  id: number;
  teacherName: string;
  subjects: TeacherSubject[];
  classNames: string[];
  remark: string | null;
}

export interface TeacherQuery {
  nameKeyword?: string;
  className?: string;
  subject?: TeacherSubject | "";
}

export interface TeacherSummary {
  importedAt: string | null;
  teacherCount: number;
}

export interface TeacherImportResult {
  importedAt: string;
  rowCount: number;
  durationMs: number;
}

export type TeacherListResult = ListResult<TeacherRow>;
