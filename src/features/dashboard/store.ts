import { computed, reactive, readonly } from "vue";
import { Subject } from "../../entities/score/model";
import type {
  ExamAllocationSettings,
  ExamPlanOverview,
  ExamPlanSession,
  ExamPlanSessionDetail,
  ExamSessionTime,
  ExamStaffPlanOverview,
  ExamStaffTask,
  SpaceStaffRequirement,
  TeacherDutyStat,
} from "../../entities/exam-plan/model";
import { examAllocationService, type ExamAllocationService } from "./service";

const emptyOverview: ExamPlanOverview = {
  generatedAt: null,
  defaultCapacity: 40,
  maxCapacity: 41,
  gradeCount: 0,
  sessionCount: 0,
  examRoomCount: 0,
  selfStudyRoomCount: 0,
  studentAllocationCount: 0,
  warningCount: 0,
};

const emptySettings: ExamAllocationSettings = {
  defaultCapacity: 40,
  maxCapacity: 41,
  examTitle: "",
  examNotices: [],
  updatedAt: null,
};

const emptyStaffOverview: ExamStaffPlanOverview = {
  generatedAt: null,
  sessionCount: 0,
  taskCount: 0,
  assignedCount: 0,
  unassignedCount: 0,
  warningCount: 0,
  imbalanceMinutes: 0,
};

interface SessionTimeDraft {
  startAt: string;
  endAt: string;
}

export function createExamAllocationStore(service: ExamAllocationService = examAllocationService) {
  const state = reactive({
    loading: false,
    generating: false,
    saving: false,
    exporting: false,
    assigning: false,
    savingTimes: false,
    savingRequirements: false,
    errorMessage: "",
    settings: { ...emptySettings } as ExamAllocationSettings,
    overview: { ...emptyOverview } as ExamPlanOverview,
    sessions: [] as ExamPlanSession[],
    total: 0,
    selectedSessionId: null as number | null,
    detail: null as ExamPlanSessionDetail | null,
    filters: {
      gradeName: "",
      subject: "" as Subject | "",
      page: 1,
      pageSize: 200,
    },
    sessionTimes: [] as ExamSessionTime[],
    sessionTimeDrafts: {} as Record<number, SessionTimeDraft>,
    requirements: [] as SpaceStaffRequirement[],
    staffOverview: { ...emptyStaffOverview } as ExamStaffPlanOverview,
    staffTasks: [] as ExamStaffTask[],
    teacherDutyStats: [] as TeacherDutyStat[],
    lastExportZipPath: "",
  });

  function normalizeTimeInput(value: string | null | undefined): string {
    if (!value) {
      return "";
    }
    if (value.length >= 16) {
      return value.slice(0, 16);
    }
    return value;
  }

  async function loadSessions() {
    const result = await service.listSessions({
      gradeName: state.filters.gradeName || undefined,
      subject: state.filters.subject || undefined,
      page: state.filters.page,
      pageSize: state.filters.pageSize,
    });
    state.sessions = result.items;
    state.total = result.total;
  }

  async function loadStaffOutputs() {
    const [overview, tasks, stats] = await Promise.all([
      service.getStaffPlanOverview(),
      service.listStaffTasks({
        sessionId: state.selectedSessionId ?? undefined,
        page: 1,
        pageSize: 500,
      }),
      service.listTeacherDutyStats({ page: 1, pageSize: 500 }),
    ]);
    state.staffOverview = overview;
    state.staffTasks = tasks.items;
    state.teacherDutyStats = stats.items;
  }

  async function loadSessionTimes() {
    const list = await service.listSessionTimes();
    state.sessionTimes = list;
    const nextDrafts: Record<number, SessionTimeDraft> = {};
    for (const item of list) {
      nextDrafts[item.sessionId] = {
        startAt: normalizeTimeInput(item.startAt),
        endAt: normalizeTimeInput(item.endAt),
      };
    }
    state.sessionTimeDrafts = nextDrafts;
  }

  async function loadRequirements(sessionId: number) {
    state.requirements = await service.listSpaceStaffRequirements(sessionId);
  }

  async function loadAll() {
    state.loading = true;
    state.errorMessage = "";
    try {
      const [settings, overview] = await Promise.all([service.getSettings(), service.getOverview()]);
      state.settings = settings;
      state.overview = overview;
      await Promise.all([loadSessions(), loadSessionTimes()]);

      if (state.selectedSessionId) {
        await loadDetail(state.selectedSessionId);
      } else if (state.sessions.length > 0) {
        await loadDetail(state.sessions[0].id);
      }
      await loadStaffOutputs();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      state.loading = false;
    }
  }

  async function saveSettings(defaultCapacity: number, maxCapacity: number, examTitle: string, examNotices: string[]) {
    state.saving = true;
    state.errorMessage = "";
    try {
      await service.updateSettings({ defaultCapacity, maxCapacity, examTitle, examNotices });
      state.settings = await service.getSettings();
      state.overview.defaultCapacity = state.settings.defaultCapacity;
      state.overview.maxCapacity = state.settings.maxCapacity;
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.saving = false;
    }
  }

  async function exportLatestBundle() {
    state.exporting = true;
    state.errorMessage = "";
    try {
      const result = await service.exportLatestExamAllocationBundle();
      state.lastExportZipPath = result.zipPath;
      return result;
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.exporting = false;
    }
  }

  async function generate() {
    state.generating = true;
    state.errorMessage = "";
    try {
      await service.generate();
      await loadAll();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.generating = false;
    }
  }

  async function loadDetail(sessionId: number) {
    state.selectedSessionId = sessionId;
    state.errorMessage = "";
    try {
      state.detail = await service.getSessionDetail(sessionId);
      await Promise.all([loadRequirements(sessionId), loadStaffOutputs()]);
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    }
  }

  async function setFilters(next: Partial<{ gradeName: string; subject: Subject | "" }>) {
    state.filters = {
      ...state.filters,
      ...next,
      page: 1,
    };
    await loadSessions();
    if (state.sessions.length > 0) {
      await loadDetail(state.sessions[0].id);
    } else {
      state.selectedSessionId = null;
      state.detail = null;
      state.requirements = [];
      state.staffTasks = [];
    }
  }

  function setSessionTimeDraft(sessionId: number, field: "startAt" | "endAt", value: string) {
    if (!state.sessionTimeDrafts[sessionId]) {
      state.sessionTimeDrafts[sessionId] = { startAt: "", endAt: "" };
    }
    state.sessionTimeDrafts[sessionId][field] = value;
  }

  async function saveSessionTimes() {
    state.savingTimes = true;
    state.errorMessage = "";
    try {
      const items = Object.entries(state.sessionTimeDrafts)
        .map(([sessionId, draft]) => ({
          sessionId: Number(sessionId),
          startAt: draft.startAt,
          endAt: draft.endAt,
        }))
        .filter((item) => item.startAt && item.endAt);
      await service.upsertSessionTimes(items);
      await loadSessionTimes();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.savingTimes = false;
    }
  }

  function setRequirementCount(spaceId: number, role: "exam_room_invigilator", requiredCount: number) {
    const target = state.requirements.find((item) => item.spaceId === spaceId && item.role === role);
    if (!target) {
      return;
    }
    target.requiredCount = Math.max(1, Math.floor(requiredCount || 1));
  }

  async function saveRequirements() {
    if (!state.selectedSessionId) {
      return;
    }
    state.savingRequirements = true;
    state.errorMessage = "";
    try {
      const examRoomItems = state.requirements
        .filter((item) => item.role === "exam_room_invigilator" && item.spaceId)
        .map((item) => ({
          spaceId: item.spaceId as number,
          role: item.role,
          requiredCount: Math.max(1, Math.floor(item.requiredCount || 1)),
        }));
      await service.upsertSpaceStaffRequirements(state.selectedSessionId, examRoomItems);
      await loadRequirements(state.selectedSessionId);
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.savingRequirements = false;
    }
  }

  async function assignTeachers() {
    state.assigning = true;
    state.errorMessage = "";
    try {
      await service.generateStaffPlan();
      await loadStaffOutputs();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.assigning = false;
    }
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      generating: state.generating,
      saving: state.saving,
      exporting: state.exporting,
      assigning: state.assigning,
      savingTimes: state.savingTimes,
      savingRequirements: state.savingRequirements,
      errorMessage: state.errorMessage,
      settings: state.settings,
      overview: state.overview,
      sessions: state.sessions,
      total: state.total,
      selectedSessionId: state.selectedSessionId,
      detail: state.detail,
      filters: state.filters,
      sessionTimes: state.sessionTimes,
      sessionTimeDrafts: state.sessionTimeDrafts,
      requirements: state.requirements,
      staffOverview: state.staffOverview,
      staffTasks: state.staffTasks,
      teacherDutyStats: state.teacherDutyStats,
      lastExportZipPath: state.lastExportZipPath,
    })),
  );

  return {
    loadAll,
    saveSettings,
    exportLatestBundle,
    generate,
    loadDetail,
    setFilters,
    setSessionTimeDraft,
    saveSessionTimes,
    setRequirementCount,
    saveRequirements,
    assignTeachers,
    get viewState() {
      return viewState.value;
    },
  };
}

const singleton = createExamAllocationStore();

export function useExamAllocationStore() {
  return singleton;
}
