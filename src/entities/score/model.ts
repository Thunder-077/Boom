export enum Subject {
  Chinese = "chinese",
  Math = "math",
  English = "english",
  Physics = "physics",
  Chemistry = "chemistry",
  Biology = "biology",
  Politics = "politics",
  History = "history",
  Geography = "geography",
  Russian = "russian",
  Japanese = "japanese",
}

export type ScoreCellState = "scored" | "not_selected" | "absent";

export interface ScoreRow {
  admissionNo: string;
  className: string;
  gradeName: string;
  studentName: string;
  subjectCombination: string;
  language: string;
  totalScore: number;
  classRank: number;
  gradeRank: number;
  selectedSubjectCount: number;
}

export interface ScoreSubjectItem {
  subject: Subject;
  score: number | null;
  state: ScoreCellState;
}

export interface ScoreDetail extends ScoreRow {
  subjects: ScoreSubjectItem[];
}

export interface ScoreUpdatePayload {
  admissionNo: string;
  className: string;
  studentName: string;
  subjects: ScoreSubjectItem[];
}

export interface ScoreQuery {
  nameKeyword?: string;
  className?: string;
  gradeName?: string;
}

export interface LatestScoreSummary {
  importedAt: string | null;
  studentCount: number;
  classCount: number;
  gradeCount: number;
}

export interface ImportResult {
  importedAt: string;
  rowCount: number;
  warningCount: number;
  durationMs: number;
}
