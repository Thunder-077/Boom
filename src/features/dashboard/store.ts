import { computed, reactive, readonly } from "vue";
import { Subject } from "../../entities/score/model";
import type {
  ExamAllocationSettings,
  ExamGenerationProgress,
  ExamPlanOverview,
  ExamPlanSession,
  ExamPlanSessionDetail,
  ExamSessionTime,
  ExamStaffPlanOverview,
  ExamStaffTask,
  ExamStaffExclusion,
  InvigilationExclusionSessionOption,
  InvigilationConfig,
  TeacherDutyStat,
} from "../../entities/exam-plan/model";
import type { TeacherRow } from "../../entities/teacher/model";
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
  solverEngine: "greedy",
  optimalityStatus: "fallback",
  solveDurationMs: 0,
  fallbackReason: null,
  fallbackPoolAssignments: 0,
  baselineDominated: false,
};

const emptyGenerationProgress: ExamGenerationProgress = {
  status: "idle",
  stage: "idle",
  stageLabel: "等待开始",
  percent: 0,
  message: "等待开始分配考场",
  currentGrade: null,
  totalGrades: 0,
  completedGrades: 0,
  updatedAt: "",
};

const defaultInvigilationConfig: InvigilationConfig = {
  defaultExamRoomRequiredCount: 1,
  indoorAllowancePerMinute: 0.5,
  outdoorAllowancePerMinute: 0.3,
  middleManagerDefaultEnabled: false,
  middleManagerExceptionTeacherIds: [],
  selfStudyDate: new Date().toISOString().slice(0, 10),
  selfStudyStartTime: "12:10",
  selfStudyEndTime: "13:40",
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
    staffOverview: { ...emptyStaffOverview } as ExamStaffPlanOverview,
    staffTasks: [] as ExamStaffTask[],
    teacherDutyStats: [] as TeacherDutyStat[],
    invigilationConfig: { ...defaultInvigilationConfig } as InvigilationConfig,
    staffExclusions: [] as ExamStaffExclusion[],
    selfStudyClassSubjects: [] as Array<{ classId: number; subject: Subject | null }>,
    exclusionSessionOptions: [] as InvigilationExclusionSessionOption[],
    teachers: [] as TeacherRow[],
    lastExportZipPath: "",
    generationProgress: { ...emptyGenerationProgress } as ExamGenerationProgress,
  });
  let progressPollTimer: number | null = null;

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

  async function loadTeachers() {
    const result = await service.listTeachers({ page: 1, pageSize: 2000 });
    state.teachers = result.items;
  }

  async function loadExclusionSessionOptions() {
    state.exclusionSessionOptions =
      await service.listInvigilationExclusionSessionOptions();
  }

  async function loadPersistedInvigilationState() {
    const persisted = await service.getPersistedInvigilationState();
    state.invigilationConfig = {
      ...defaultInvigilationConfig,
      ...persisted.config,
      selfStudyDate: (persisted.config.selfStudyDate || defaultInvigilationConfig.selfStudyDate).trim(),
    };
    state.staffExclusions = persisted.exclusions
      .map((item) => ({
        teacherId: Number(item.teacherId),
        teacherName: String(item.teacherName || ""),
        sessionId: Number(item.sessionId),
        sessionLabel: String(item.sessionLabel || ""),
      }))
      .filter((item) => item.teacherId > 0 && item.sessionId > 0 && item.teacherName);
    state.selfStudyClassSubjects = persisted.selfStudyClassSubjects.map((item) => ({
      classId: Number(item.classId),
      subject: item.subject ?? null,
    }));
  }

  async function loadAll() {
    state.loading = true;
    state.errorMessage = "";
    try {
      const [settings, overview, generationProgress] = await Promise.all([
        service.getSettings(),
        service.getOverview(),
        service.getGenerationProgress(),
      ]);
      state.settings = settings;
      state.overview = overview;
      state.generationProgress = generationProgress;
      await Promise.all([loadSessions(), loadSessionTimes()]);
      await Promise.all([loadTeachers(), loadExclusionSessionOptions(), loadPersistedInvigilationState()]);

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

  async function refreshGenerationProgress() {
    state.generationProgress = await service.getGenerationProgress();
  }

  function stopProgressPolling() {
    if (progressPollTimer !== null) {
      window.clearInterval(progressPollTimer);
      progressPollTimer = null;
    }
  }

  function startProgressPolling() {
    stopProgressPolling();
    progressPollTimer = window.setInterval(() => {
      void refreshGenerationProgress();
    }, 500);
  }

  async function generate() {
    state.generating = true;
    state.errorMessage = "";
    state.lastExportZipPath = "";
    try {
      await refreshGenerationProgress();
      startProgressPolling();
      await service.startGenerate();
      while (true) {
        await refreshGenerationProgress();
        if (state.generationProgress.status === "completed") {
          break;
        }
        if (state.generationProgress.status === "error") {
          throw new Error(state.generationProgress.message || "考场分配失败");
        }
        await new Promise((resolve) => window.setTimeout(resolve, 400));
      }
      await loadAll();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      stopProgressPolling();
      state.generating = false;
    }
  }

  async function loadDetail(sessionId: number) {
    state.selectedSessionId = sessionId;
    state.errorMessage = "";
    try {
      state.detail = await service.getSessionDetail(sessionId);
      await loadStaffOutputs();
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
      state.staffTasks = [];
    }
  }

  function setSessionTimeDraft(sessionId: number, field: "startAt" | "endAt", value: string) {
    if (!state.sessionTimeDrafts[sessionId]) {
      state.sessionTimeDrafts[sessionId] = { startAt: "", endAt: "" };
    }
    state.sessionTimeDrafts[sessionId][field] = value;
  }

  async function saveSessionTimes(extraItems: Array<{ sessionId: number; subject: Subject; startAt: string; endAt: string }> = []) {
    state.savingTimes = true;
    state.errorMessage = "";
    try {
      const items = state.sessionTimes
        .map((item) => {
          const draft = state.sessionTimeDrafts[item.sessionId];
          if (!draft) {
            return null;
          }
          return {
            sessionId: item.sessionId,
            subject: item.subject,
            startAt: draft.startAt,
            endAt: draft.endAt,
          };
        })
        .concat(extraItems)
        .filter((item): item is { sessionId: number; subject: Subject; startAt: string; endAt: string } => !!item && !!item.startAt && !!item.endAt);
      await service.upsertSessionTimes(items);
      await loadSessionTimes();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.savingTimes = false;
    }
  }

  async function deleteSessionTime(subject: Subject) {
    state.savingTimes = true;
    state.errorMessage = "";
    try {
      await service.deleteSessionTime(subject);
      await loadSessionTimes();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.savingTimes = false;
    }
  }

  async function assignTeachers() {
    state.assigning = true;
    state.errorMessage = "";
    try {
      const exclusionPairs = new Map<string, { teacherId: number; sessionId: number }>();
      for (const item of state.staffExclusions) {
        const teacherId = Number(item.teacherId);
        const sessionId = Number(item.sessionId);
        if (teacherId <= 0) {
          continue;
        }
        if (sessionId > 0) {
          exclusionPairs.set(`${teacherId}-${sessionId}`, { teacherId, sessionId });
          continue;
        }
        const templateOption = state.exclusionSessionOptions.find(
          (option) => option.sessionId === sessionId,
        );
        if (!templateOption) {
          continue;
        }
        for (const session of state.sessions) {
          if (session.subject !== templateOption.subject) {
            continue;
          }
          exclusionPairs.set(`${teacherId}-${session.id}`, {
            teacherId,
            sessionId: session.id,
          });
        }
      }
      const normalizedExclusions = Array.from(exclusionPairs.values());
      const result = await service.generateStaffPlan({
        defaultExamRoomRequiredCount: Math.max(
          1,
          Math.floor(state.invigilationConfig.defaultExamRoomRequiredCount || 1),
        ),
        indoorAllowancePerMinute: Math.max(
          0,
          Number(state.invigilationConfig.indoorAllowancePerMinute || 0),
        ),
        outdoorAllowancePerMinute: Math.max(
          0,
          Number(state.invigilationConfig.outdoorAllowancePerMinute || 0),
        ),
        staffExclusions: normalizedExclusions,
      });
      await loadStaffOutputs();
      return result;
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.assigning = false;
    }
  }

  async function saveInvigilationConfig(payload?: Partial<InvigilationConfig>) {
    const next = { ...state.invigilationConfig, ...payload };
    state.invigilationConfig = {
      defaultExamRoomRequiredCount: Math.max(
        1,
        Math.floor(next.defaultExamRoomRequiredCount || 1),
      ),
      indoorAllowancePerMinute: Math.max(
        0,
        Number(next.indoorAllowancePerMinute ?? 0),
      ),
      outdoorAllowancePerMinute: Math.max(
        0,
        Number(next.outdoorAllowancePerMinute ?? 0),
      ),
      middleManagerDefaultEnabled: Boolean(next.middleManagerDefaultEnabled),
      middleManagerExceptionTeacherIds: Array.from(
        new Set((next.middleManagerExceptionTeacherIds ?? []).map((item) => Number(item)).filter((item) => item > 0)),
      ).sort((a, b) => a - b),
      selfStudyDate: (next.selfStudyDate || defaultInvigilationConfig.selfStudyDate).trim(),
      selfStudyStartTime: (next.selfStudyStartTime || "12:10").trim(),
      selfStudyEndTime: (next.selfStudyEndTime || "13:40").trim(),
    };
    await service.savePersistedInvigilationConfig(state.invigilationConfig);
  }

  async function addStaffExclusion(teacherId: number, sessionId: number) {
    const teacher = state.teachers.find((item) => item.id === teacherId);
    const session = state.exclusionSessionOptions.find(
      (item) => item.sessionId === sessionId,
    );
    if (!teacher || !session) {
      return false;
    }
    const exists = state.staffExclusions.some(
      (item) => item.teacherId === teacherId && item.sessionId === sessionId,
    );
    if (exists) {
      return false;
    }
    state.staffExclusions.unshift({
      teacherId,
      teacherName: teacher.teacherName,
      sessionId,
      sessionLabel: session.label,
    });
    await service.replacePersistedInvigilationExclusions(state.staffExclusions);
    return true;
  }

  async function removeStaffExclusion(teacherId: number, sessionId: number) {
    state.staffExclusions = state.staffExclusions.filter(
      (item) => !(item.teacherId === teacherId && item.sessionId === sessionId),
    );
    await service.replacePersistedInvigilationExclusions(state.staffExclusions);
  }

  async function saveSelfStudyClassSubjects(
    items: Array<{ classId: number; subject: Subject | null }>,
  ) {
    state.selfStudyClassSubjects = items.map((item) => ({
      classId: item.classId,
      subject: item.subject ?? null,
    }));
    await service.savePersistedSelfStudyClassSubjects(state.selfStudyClassSubjects);
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      generating: state.generating,
      saving: state.saving,
      exporting: state.exporting,
      assigning: state.assigning,
      savingTimes: state.savingTimes,
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
      staffOverview: state.staffOverview,
      staffTasks: state.staffTasks,
      teacherDutyStats: state.teacherDutyStats,
      invigilationConfig: state.invigilationConfig,
      staffExclusions: state.staffExclusions,
      selfStudyClassSubjects: state.selfStudyClassSubjects,
      exclusionSessionOptions: state.exclusionSessionOptions.map((item) => ({
        sessionId: item.sessionId,
        label: item.label,
      })),
      teachers: state.teachers,
      lastExportZipPath: state.lastExportZipPath,
      generationProgress: state.generationProgress,
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
    deleteSessionTime,
    assignTeachers,
    saveInvigilationConfig,
    addStaffExclusion,
    removeStaffExclusion,
    saveSelfStudyClassSubjects,
    refreshGenerationProgress,
    get viewState() {
      return viewState.value;
    },
  };
}

const singleton = createExamAllocationStore();

export function useExamAllocationStore() {
  return singleton;
}
