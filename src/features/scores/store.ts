import { computed, reactive, readonly } from "vue";
import type { ImportResult, LatestScoreSummary, ScoreDetail, ScoreQuery, ScoreRow, ScoreUpdatePayload } from "../../entities/score/model";
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

export function createScoreStore(service: ScoreService = scoreService) {
  const state = reactive({
    loading: false,
    filters: { ...defaultFilters },
    rows: [] as ScoreRow[],
    total: 0,
    summary: { ...emptySummary } as LatestScoreSummary,
    importStatus: "idle" as ImportStatus,
    importMessage: "",
    lastImportResult: null as ImportResult | null,
    page: 1,
    pageSize: 50,
  });

  async function load() {
    state.loading = true;
    try {
      const [listResult, summaryResult] = await Promise.all([
        service.list({
          ...state.filters,
          page: state.page,
          pageSize: state.pageSize,
        }),
        service.getLatestSummary(),
      ]);
      state.rows = listResult.items;
      state.total = listResult.total;
      state.summary = summaryResult;
    } finally {
      state.loading = false;
    }
  }

  async function setFilters(filters: Partial<ScoreQuery>) {
    state.filters = {
      ...state.filters,
      ...filters,
    };
    state.page = 1;
    await load();
  }

  async function resetFilters() {
    state.filters = { ...defaultFilters };
    state.page = 1;
    await load();
  }

  async function importExcel(filePath: string) {
    state.importStatus = "importing";
    state.importMessage = "正在导入成绩 Excel...";
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

  async function getDetail(admissionNo: string): Promise<ScoreDetail> {
    return service.getDetail(admissionNo);
  }

  async function setPage(page: number) {
    state.page = page;
    await load();
  }

  async function updateScore(payload: ScoreUpdatePayload) {
    await service.updateScore(payload);
    await load();
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      filters: state.filters,
      rows: state.rows,
      total: state.total,
      page: state.page,
      pageSize: state.pageSize,
      totalPages: Math.max(1, Math.ceil(state.total / state.pageSize)),
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
    setPage,
    importExcel,
    getDetail,
    updateScore,
    setImportFeedback,
    get viewState() {
      return viewState.value;
    },
  };
}

const scoreStoreSingleton = createScoreStore();

export function useScoreStore() {
  return scoreStoreSingleton;
}
