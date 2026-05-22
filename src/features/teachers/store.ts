import { createStore } from "zustand/vanilla";
import { useSyncExternalStore } from "react";
import type {
  TeacherImportResult,
  TeacherQuery,
  TeacherRow,
  TeacherSubject,
  TeacherSummary,
} from "../../entities/teacher/model";
import { createVueViewState } from "../../shared/store/zustandVueBridge";
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

interface TeacherStoreState {
  loading: boolean;
  filters: TeacherQuery;
  rows: TeacherRow[];
  total: number;
  summary: TeacherSummary;
  importStatus: ImportStatus;
  importMessage: string;
  lastImportResult: TeacherImportResult | null;
}

export function createTeacherStore(service: TeacherService = teacherService) {
  const store = createStore<TeacherStoreState>(() => ({
    loading: false,
    filters: { ...defaultFilters },
    rows: [],
    total: 0,
    summary: { ...emptySummary },
    importStatus: "idle",
    importMessage: "",
    lastImportResult: null,
  }));

  const viewState = createVueViewState(store);

  async function load() {
    store.setState({ loading: true });
    try {
      const { filters } = store.getState();
      const [listResult, summary] = await Promise.all([
        service.list(filters),
        service.getSummary(),
      ]);
      store.setState({
        rows: listResult.items,
        total: listResult.total,
        summary,
      });
    } finally {
      store.setState({ loading: false });
    }
  }

  async function setFilters(filters: Partial<TeacherQuery>) {
    store.setState((state) => ({
      filters: {
        ...state.filters,
        ...filters,
      },
    }));
    await load();
  }

  async function resetFilters() {
    store.setState({ filters: { ...defaultFilters } });
    await load();
  }

  async function importExcel(filePath: string) {
    store.setState({
      importStatus: "importing",
      importMessage: "正在导入教师 Excel...",
    });
    try {
      const result = await service.importExcel(filePath);
      store.setState({
        lastImportResult: result,
        importStatus: "success",
        importMessage: `共 ${result.rowCount} 条，耗时 ${result.durationMs}ms`,
      });
      await load();
    } catch (error) {
      store.setState({
        importStatus: "error",
        importMessage: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  function setImportFeedback(status: ImportStatus, message: string) {
    store.setState({
      importStatus: status,
      importMessage: message,
    });
  }

  return {
    store,
    load,
    setFilters,
    resetFilters,
    importExcel,
    setImportFeedback,
    get viewState() {
      return viewState;
    },
  };
}

const teacherStoreSingleton = createTeacherStore();

export function useTeacherStore() {
  return teacherStoreSingleton;
}

export function useReactTeacherStore() {
  const state = useSyncExternalStore(
    teacherStoreSingleton.store.subscribe,
    teacherStoreSingleton.store.getState,
    teacherStoreSingleton.store.getInitialState,
  );
  return {
    state,
    load: teacherStoreSingleton.load,
    setFilters: teacherStoreSingleton.setFilters,
    resetFilters: teacherStoreSingleton.resetFilters,
    importExcel: teacherStoreSingleton.importExcel,
    setImportFeedback: teacherStoreSingleton.setImportFeedback,
  };
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
