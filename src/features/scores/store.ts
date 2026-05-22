import { createStore } from "zustand/vanilla";
import { useSyncExternalStore } from "react";
import type {
  ImportResult,
  LatestScoreSummary,
  ScoreDetail,
  ScoreQuery,
  ScoreRow,
  ScoreUpdatePayload,
} from "../../entities/score/model";
import { createVueViewState } from "../../shared/store/zustandVueBridge";
import { scoreService, type ScoreService } from "./service";

type ImportStatus = "idle" | "importing" | "success" | "error";

const defaultFilters: ScoreQuery = {
  nameKeyword: "",
  className: "",
  gradeName: "",
};

const emptySummary: LatestScoreSummary = {
  importedAt: null,
  studentCount: 0,
  classCount: 0,
  gradeCount: 0,
};

interface ScoreStoreState {
  loading: boolean;
  filters: ScoreQuery;
  rows: ScoreRow[];
  total: number;
  summary: LatestScoreSummary;
  importStatus: ImportStatus;
  importMessage: string;
  lastImportResult: ImportResult | null;
  page: number;
  pageSize: number;
  totalPages: number;
}

function withTotalPages(state: Omit<ScoreStoreState, "totalPages">): ScoreStoreState {
  return {
    ...state,
    totalPages: Math.max(1, Math.ceil(state.total / state.pageSize)),
  };
}

export function createScoreStore(service: ScoreService = scoreService) {
  const store = createStore<ScoreStoreState>(() =>
    withTotalPages({
      loading: false,
      filters: { ...defaultFilters },
      rows: [],
      total: 0,
      summary: { ...emptySummary },
      importStatus: "idle",
      importMessage: "",
      lastImportResult: null,
      page: 1,
      pageSize: 7,
    }),
  );

  const viewState = createVueViewState(store);

  async function load() {
    store.setState({ loading: true });
    try {
      const { filters, page, pageSize } = store.getState();
      const [listResult, summaryResult] = await Promise.all([
        service.list({
          ...filters,
          page,
          pageSize,
        }),
        service.getLatestSummary(),
      ]);
      store.setState((state) =>
        withTotalPages({
          ...state,
          rows: listResult.items,
          total: listResult.total,
          summary: summaryResult,
        }),
      );
    } finally {
      store.setState({ loading: false });
    }
  }

  async function setFilters(filters: Partial<ScoreQuery>) {
    store.setState((state) =>
      withTotalPages({
        ...state,
        filters: {
          ...state.filters,
          ...filters,
        },
        page: 1,
      }),
    );
    await load();
  }

  async function resetFilters() {
    store.setState((state) =>
      withTotalPages({
        ...state,
        filters: { ...defaultFilters },
        page: 1,
      }),
    );
    await load();
  }

  async function importExcel(filePath: string) {
    store.setState({
      importStatus: "importing",
      importMessage: "正在导入成绩 Excel...",
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

  async function getDetail(admissionNo: string): Promise<ScoreDetail> {
    return service.getDetail(admissionNo);
  }

  async function setPage(page: number) {
    store.setState((state) => withTotalPages({ ...state, page }));
    await load();
  }

  async function updateScore(payload: ScoreUpdatePayload) {
    await service.updateScore(payload);
    await load();
  }

  return {
    store,
    load,
    setFilters,
    resetFilters,
    setPage,
    importExcel,
    getDetail,
    updateScore,
    setImportFeedback,
    get viewState() {
      return viewState;
    },
  };
}

const scoreStoreSingleton = createScoreStore();

export function useScoreStore() {
  return scoreStoreSingleton;
}

export function useReactScoreStore() {
  const state = useSyncExternalStore(
    scoreStoreSingleton.store.subscribe,
    scoreStoreSingleton.store.getState,
    scoreStoreSingleton.store.getInitialState,
  );

  return {
    state,
    load: scoreStoreSingleton.load,
    setFilters: scoreStoreSingleton.setFilters,
    resetFilters: scoreStoreSingleton.resetFilters,
    setPage: scoreStoreSingleton.setPage,
    importExcel: scoreStoreSingleton.importExcel,
    getDetail: scoreStoreSingleton.getDetail,
    updateScore: scoreStoreSingleton.updateScore,
    setImportFeedback: scoreStoreSingleton.setImportFeedback,
  };
}
