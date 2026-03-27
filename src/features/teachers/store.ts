import { computed, reactive, readonly } from "vue";
import type {
  TeacherImportResult,
  TeacherQuery,
  TeacherRow,
  TeacherSubject,
  TeacherSummary,
} from "../../entities/teacher/model";
import { teacherService, type TeacherService } from "./service";

const defaultFilters: TeacherQuery = {
  nameKeyword: "",
  className: "",
  subject: "",
};

const emptySummary: TeacherSummary = {
  importedAt: null,
  teacherCount: 0,
};

type ImportStatus = "idle" | "importing" | "success" | "error";

export function createTeacherStore(service: TeacherService = teacherService) {
  const state = reactive({
    loading: false,
    filters: { ...defaultFilters },
    rows: [] as TeacherRow[],
    total: 0,
    summary: { ...emptySummary } as TeacherSummary,
    importStatus: "idle" as ImportStatus,
    importMessage: "",
    lastImportResult: null as TeacherImportResult | null,
  });

  async function load() {
    state.loading = true;
    try {
      const [listResult, summary] = await Promise.all([
        service.list(state.filters),
        service.getSummary(),
      ]);
      state.rows = listResult.items;
      state.total = listResult.total;
      state.summary = summary;
    } finally {
      state.loading = false;
    }
  }

  async function setFilters(filters: Partial<TeacherQuery>) {
    state.filters = {
      ...state.filters,
      ...filters,
    };
    await load();
  }

  async function resetFilters() {
    state.filters = { ...defaultFilters };
    await load();
  }

  async function importExcel(filePath: string) {
    state.importStatus = "importing";
    state.importMessage = "正在导入教师 Excel...";
    try {
      const result = await service.importExcel(filePath);
      state.lastImportResult = result;
      state.importStatus = "success";
      state.importMessage = `共 ${result.rowCount} 条，耗时 ${result.durationMs}ms`;
      await load();
    } catch (error) {
      state.importStatus = "error";
      state.importMessage = error instanceof Error ? error.message : String(error);
      throw error;
    }
  }

  function setImportFeedback(status: ImportStatus, message: string) {
    state.importStatus = status;
    state.importMessage = message;
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      filters: state.filters,
      rows: state.rows,
      total: state.total,
      summary: state.summary,
      importStatus: state.importStatus,
      importMessage: state.importMessage,
      lastImportResult: state.lastImportResult,
    })),
  );

  return {
    load,
    setFilters,
    resetFilters,
    importExcel,
    setImportFeedback,
    get viewState() {
      return viewState.value;
    },
  };
}

const teacherStoreSingleton = createTeacherStore();

export function useTeacherStore() {
  return teacherStoreSingleton;
}

export const TEACHER_SUBJECT_OPTIONS: Array<{ value: TeacherSubject | ""; label: string }> = [
  { value: "", label: "全部科目" },
  { value: "chinese", label: "语文" },
  { value: "math", label: "数学" },
  { value: "english", label: "英语" },
  { value: "physics", label: "物理" },
  { value: "chemistry", label: "化学" },
  { value: "biology", label: "生物" },
  { value: "politics", label: "政治" },
  { value: "history", label: "历史" },
  { value: "geography", label: "地理" },
  { value: "russian", label: "俄语" },
  { value: "japanese", label: "日语" },
  { value: "sports", label: "体育" },
  { value: "music", label: "音乐" },
  { value: "general", label: "通用" },
  { value: "information", label: "信息" },
  { value: "fine_arts", label: "美术" },
];
