import type { Subject } from "../score/model";

export type ClassConfigType = "teaching_class" | "exam_room";

export const CLASS_CONFIG_TYPE_OPTIONS: Array<{ value: ClassConfigType; label: string }> = [
  { value: "teaching_class", label: "教学班" },
  { value: "exam_room", label: "考试教室" },
];

export const SUBJECT_LABELS: Record<Subject, string> = {
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
};

export const SUBJECT_OPTIONS = Object.entries(SUBJECT_LABELS).map(([value, label]) => ({
  value: value as Subject,
  label,
}));

export interface ClassConfigRow {
  id: number;
  configType: ClassConfigType;
  gradeName: string;
  className: string;
  building: string;
  floor: string;
  roomLabel: string | null;
  updatedAt: string;
}

export interface ClassConfigDetail extends ClassConfigRow {
  subjects: Subject[];
}

export interface ClassConfigUpsertPayload {
  configType: ClassConfigType;
  gradeName: string;
  className: string;
  building: string;
  floor: string;
  roomLabel: string | null;
  subjects: Subject[];
}

export interface ClassConfigFilters {
  configType: ClassConfigType;
  gradeName: string;
  keyword: string;
}
