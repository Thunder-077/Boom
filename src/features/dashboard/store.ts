import { useSyncExternalStore } from "react";
import { createStore } from "zustand/vanilla";
import { Subject } from "../../entities/score/model";
import type {
  ExamAllocationSettings,
  ExamGenerationProgress,
  ExamPlanOverview,
  ExamPlanSession,
  ExamPlanSessionDetail,
  ExamStaffAssignmentProgress,
  ExamSessionTime,
  ExamStaffPlanOverview,
  ExamStaffTask,
  InvigilationCustomRule,
  InvigilationExclusionSessionOption,
  InvigilationConfig,
  InvigilationRuleOptions,
  TeacherDutyStat,
} from "../../entities/exam-plan/model";
import type { TeacherRow } from "../../entities/teacher/model";
import { createMutableZustandState } from "../../shared/store/zustandStateProxy";
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
  solverEngine: "cp_sat",
  optimalityStatus: "feasible",
  solveDurationMs: 0,
  fallbackReason: null,
  fallbackPoolAssignments: 0,
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

const FOREIGN_SUBJECTS: Subject[] = [Subject.English, Subject.Russian, Subject.Japanese];

function isForeignSubject(subject: Subject): boolean {
  return FOREIGN_SUBJECTS.includes(subject);
}

function subjectOrder(subject: Subject): number {
  const order: Subject[] = [
    Subject.Chinese,
    Subject.Math,
    Subject.English,
    Subject.Physics,
    Subject.Chemistry,
    Subject.Biology,
    Subject.Politics,
    Subject.History,
    Subject.Geography,
    Subject.Russian,
    Subject.Japanese,
  ];
  const index = order.indexOf(subject);
  return index >= 0 ? index : 99;
}

export function createExamAllocationStore(service: ExamAllocationService = examAllocationService) {
  const store = createStore(() => ({
    loading: false,
    generating: false,
    saving: false,
    exporting: false,
    exportingInvigilation: false,
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
    sessionTimeGradeOptions: [] as string[],
    selectedSessionTimeGradeName: "",
    sessionTimes: [] as ExamSessionTime[],
    sessionTimeDrafts: {} as Record<number, SessionTimeDraft>,
    foreignSessionBindings: [] as Array<{ subject: Subject; sessionId: number }>,
    staffOverview: { ...emptyStaffOverview } as ExamStaffPlanOverview,
    staffTasks: [] as ExamStaffTask[],
    teacherDutyStats: [] as TeacherDutyStat[],
    invigilationConfig: { ...defaultInvigilationConfig } as InvigilationConfig,
    customRules: [] as InvigilationCustomRule[],
    customRuleOptions: {
      examSessionOptions: [],
      fullSelfStudyOption: null,
      targetOptions: [],
    } as InvigilationRuleOptions,
    selfStudyClassSubjects: [] as Array<{ classId: number; subject: Subject | null }>,
    exclusionSessionOptions: [] as InvigilationExclusionSessionOption[],
    teachers: [] as TeacherRow[],
    lastExportFolderPath: "",
    lastInvigilationExportPath: "",
    generationProgress: { ...emptyGenerationProgress } as ExamGenerationProgress,
    assignmentProgress: null as ExamStaffAssignmentProgress | null,
  }));
  const state = createMutableZustandState(store);
  const snapshot = () => store.getState();
  let progressPollTimer: number | null = null;
  const isNoRowsError = (error: unknown) => String(error).includes("Query returned no rows");

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

  async function loadSessionTimeGradeOptions() {
    const options = await service.listSessionTimeGradeOptions();
    state.sessionTimeGradeOptions = options;
    if (!state.selectedSessionTimeGradeName || !options.includes(state.selectedSessionTimeGradeName)) {
      state.selectedSessionTimeGradeName = options[0] ?? "";
    }
  }

  async function loadSessionTimes() {
    const list = await service.listSessionTimes(
      state.selectedSessionTimeGradeName
        ? { gradeName: state.selectedSessionTimeGradeName }
        : undefined,
    );
    if (!state.selectedSessionTimeGradeName && list.length > 0) {
      state.selectedSessionTimeGradeName = list[0].gradeName;
      if (!state.sessionTimeGradeOptions.includes(list[0].gradeName)) {
        state.sessionTimeGradeOptions = [...state.sessionTimeGradeOptions, list[0].gradeName];
      }
    }
    if (list.length === 0) {
      state.sessionTimes = [];
      state.sessionTimeDrafts = {};
      state.foreignSessionBindings = [];
      return;
    }
    const foreignRows = list
      .filter((item) => isForeignSubject(item.subject))
      .sort((a, b) => subjectOrder(a.subject) - subjectOrder(b.subject));
    state.foreignSessionBindings = foreignRows.map((item) => ({
      subject: item.subject,
      sessionId: item.sessionId,
    }));
    const mergedForeign = foreignRows.find((item) => item.startAt || item.endAt) ?? foreignRows[0];
    const nextSessionTimes = list.filter((item) => !isForeignSubject(item.subject));
    if (mergedForeign) {
      nextSessionTimes.push({
        ...mergedForeign,
        subject: Subject.English,
      });
    }
    nextSessionTimes.sort((a, b) => {
      const aTime = normalizeTimeInput(a.startAt);
      const bTime = normalizeTimeInput(b.startAt);
      if (aTime && bTime) {
        const compared = aTime.localeCompare(bTime);
        if (compared !== 0) {
          return compared;
        }
      }
      return subjectOrder(a.subject) - subjectOrder(b.subject);
    });
    state.sessionTimes = nextSessionTimes;
    const nextDrafts: Record<number, SessionTimeDraft> = {};
    for (const item of nextSessionTimes) {
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
    state.customRules = persisted.customRules
      .map((item) => ({
        actionType: item.actionType,
        teacherId: Number(item.teacherId),
        teacherName: String(item.teacherName || ""),
        timeScopeType: item.timeScopeType,
        timeScopeIds: Array.isArray(item.timeScopeIds)
          ? item.timeScopeIds.map((value) => Number(value)).filter((value) => value > 0)
          : [],
        timeScopeLabels: Array.isArray(item.timeScopeLabels)
          ? item.timeScopeLabels.map((value) => String(value))
          : [],
        taskScopeType: item.taskScopeType,
        targetScopeType: item.targetScopeType,
        targetIds: Array.isArray(item.targetIds)
          ? item.targetIds.map((value) => String(value)).filter((value) => value.trim())
          : [],
        targetLabels: Array.isArray(item.targetLabels)
          ? item.targetLabels.map((value) => String(value))
          : [],
      }))
      .filter((item) => item.teacherId > 0 && item.teacherName);
    state.selfStudyClassSubjects = persisted.selfStudyClassSubjects.map((item) => ({
      classId: Number(item.classId),
      subject: item.subject ?? null,
    }));
  }

  async function loadCustomRuleOptions() {
    state.customRuleOptions = await service.listInvigilationCustomRuleOptions();
  }

  async function loadAll() {
    state.loading = true;
    state.errorMessage = "";
    try {
      // Load core cards independently so one endpoint failure won't blank all fields.
      const coreResults = await Promise.allSettled([
        service.getSettings(),
        service.getOverview(),
        service.getGenerationProgress(),
      ]);
      const coreErrors: string[] = [];
      if (coreResults[0].status === "fulfilled") {
        state.settings = coreResults[0].value;
      } else {
        coreErrors.push(`考试配置读取失败：${String(coreResults[0].reason)}`);
      }
      if (coreResults[1].status === "fulfilled") {
        state.overview = coreResults[1].value;
      } else {
        coreErrors.push(`分配总览读取失败：${String(coreResults[1].reason)}`);
      }
      if (coreResults[2].status === "fulfilled") {
        state.generationProgress = coreResults[2].value;
      } else {
        coreErrors.push(`进度状态读取失败：${String(coreResults[2].reason)}`);
      }
      try {
        await loadSessionTimeGradeOptions();
      } catch {
        // Degrade gracefully: session times can still be fetched without explicit grade param.
        state.sessionTimeGradeOptions = [];
        state.selectedSessionTimeGradeName = "";
      }
      const firstStage = await Promise.allSettled([loadSessions(), loadSessionTimes()]);
      if (firstStage[0].status === "rejected") {
        if (!isNoRowsError(firstStage[0].reason)) {
          coreErrors.push(`场次列表读取失败：${String(firstStage[0].reason)}`);
        }
      }
      if (firstStage[1].status === "rejected") {
        if (!isNoRowsError(firstStage[1].reason)) {
          coreErrors.push(`考试时间读取失败：${String(firstStage[1].reason)}`);
        }
      }
      const secondStage = await Promise.allSettled([
        loadTeachers(),
        loadExclusionSessionOptions(),
        loadPersistedInvigilationState(),
        loadCustomRuleOptions(),
      ]);
      if (secondStage[0].status === "rejected") {
        if (!isNoRowsError(secondStage[0].reason)) {
          coreErrors.push(`教师列表读取失败：${String(secondStage[0].reason)}`);
        }
      }
      if (secondStage[1].status === "rejected") {
        if (!isNoRowsError(secondStage[1].reason)) {
          coreErrors.push(`排除场次读取失败：${String(secondStage[1].reason)}`);
        }
      }
      if (secondStage[2].status === "rejected") {
        if (!isNoRowsError(secondStage[2].reason)) {
          coreErrors.push(`监考配置读取失败：${String(secondStage[2].reason)}`);
        }
      }
      if (secondStage[3].status === "rejected") {
        if (!isNoRowsError(secondStage[3].reason)) {
          coreErrors.push(`排班规则选项读取失败：${String(secondStage[3].reason)}`);
        }
      }

      const validSessionIds = new Set(state.sessions.map((item) => item.id));
      let targetSessionId: number | null = null;
      if (
        state.selectedSessionId !== null
        && validSessionIds.has(state.selectedSessionId)
      ) {
        targetSessionId = state.selectedSessionId;
      } else if (state.sessions.length > 0) {
        targetSessionId = state.sessions[0].id;
      } else {
        state.selectedSessionId = null;
        state.detail = null;
      }
      if (targetSessionId !== null) {
        try {
          await loadDetail(targetSessionId);
        } catch (error) {
          if (!isNoRowsError(error)) {
            coreErrors.push(`场次详情读取失败：${String(error)}`);
          }
        }
      } else {
        try {
          await loadStaffOutputs();
        } catch (error) {
          if (!isNoRowsError(error)) {
            coreErrors.push(`监考结果读取失败：${String(error)}`);
          }
        }
      }
      if (coreErrors.length > 0) {
        state.errorMessage = coreErrors.join("；");
      }
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
      state.lastExportFolderPath = result.folderPath;
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
    state.lastExportFolderPath = "";
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
      if (state.generationProgress.status !== "error") {
        state.generationProgress = {
          ...state.generationProgress,
          status: "error",
          stage: "error",
          stageLabel: "执行失败",
          message: state.errorMessage,
          percent: 0,
        };
      }
      // Keep stale overview from masking the current failed run in the UI.
      state.overview.generatedAt = null;
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

  async function exportLatestInvigilationSchedule() {
    state.exportingInvigilation = true;
    state.errorMessage = "";
    try {
      const result = await service.exportLatestInvigilationSchedule();
      state.lastInvigilationExportPath = result.filePath;
      return result;
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.exportingInvigilation = false;
    }
  }

  function setSessionTimeDraft(sessionId: number, field: "startAt" | "endAt", value: string) {
    if (!state.sessionTimeDrafts[sessionId]) {
      state.sessionTimeDrafts[sessionId] = { startAt: "", endAt: "" };
    }
    state.sessionTimeDrafts[sessionId][field] = value;
  }

  async function saveSessionTimes(
    extraItems: Array<{ sessionId: number; gradeName: string; subject: Subject; startAt: string; endAt: string }> = [],
  ) {
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
            gradeName: state.selectedSessionTimeGradeName,
            subject: item.subject,
            startAt: draft.startAt,
            endAt: draft.endAt,
          };
        })
        .concat(extraItems)
        .filter(
          (
            item,
          ): item is {
            sessionId: number;
            gradeName: string;
            subject: Subject;
            startAt: string;
            endAt: string;
          } =>
            !!item &&
            !!item.gradeName &&
            !!item.startAt &&
            !!item.endAt,
        );
      const foreignDisplayRow = state.sessionTimes.find((item) => item.subject === Subject.English);
      const foreignDraft = foreignDisplayRow
        ? state.sessionTimeDrafts[foreignDisplayRow.sessionId]
        : undefined;
      if (foreignDraft?.startAt && foreignDraft?.endAt) {
        const bindingBySubject = new Map(
          state.foreignSessionBindings.map((item) => [item.subject, item.sessionId]),
        );
        for (const subject of FOREIGN_SUBJECTS) {
          const existingIndex = items.findIndex((item) => item.subject === subject);
          const syncedItem = {
            sessionId: bindingBySubject.get(subject) ?? foreignDisplayRow?.sessionId ?? -999,
            gradeName: state.selectedSessionTimeGradeName,
            subject,
            startAt: foreignDraft.startAt,
            endAt: foreignDraft.endAt,
          };
          if (existingIndex >= 0) {
            items[existingIndex] = syncedItem;
          } else {
            items.push(syncedItem);
          }
        }
      }
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
      if (!state.selectedSessionTimeGradeName) {
        return;
      }
      const subjectsToDelete = isForeignSubject(subject) ? FOREIGN_SUBJECTS : [subject];
      await Promise.all(
        subjectsToDelete.map((item) =>
          service.deleteSessionTime(state.selectedSessionTimeGradeName, item),
        ),
      );
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
        customRules: snapshot().customRules,
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

  function setAssignmentProgress(progress: ExamStaffAssignmentProgress | null) {
    state.assignmentProgress = progress;
  }

  async function saveInvigilationConfig(payload?: Partial<InvigilationConfig>) {
    const next = { ...snapshot().invigilationConfig, ...payload };
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
    await service.savePersistedInvigilationConfig(snapshot().invigilationConfig);
  }

  async function saveCustomRules(rules: InvigilationCustomRule[]) {
    state.customRules = rules;
    await service.replacePersistedInvigilationCustomRules(snapshot().customRules);
    await loadCustomRuleOptions();
  }

  async function saveSelfStudyClassSubjects(
    items: Array<{ classId: number; subject: Subject | null }>,
  ) {
    state.selfStudyClassSubjects = items.map((item) => ({
      classId: item.classId,
      subject: item.subject ?? null,
    }));
    await service.savePersistedSelfStudyClassSubjects(snapshot().selfStudyClassSubjects);
  }

  async function setSessionTimeGrade(gradeName: string) {
    if (!gradeName || gradeName === state.selectedSessionTimeGradeName) {
      return;
    }
    state.selectedSessionTimeGradeName = gradeName;
    await loadSessionTimes();
  }

  return {
    store,
    loadAll,
    saveSettings,
    exportLatestBundle,
    exportLatestInvigilationSchedule,
    generate,
    loadDetail,
    setFilters,
    setSessionTimeGrade,
    setSessionTimeDraft,
    saveSessionTimes,
    deleteSessionTime,
    assignTeachers,
    saveInvigilationConfig,
    saveCustomRules,
    saveSelfStudyClassSubjects,
    refreshGenerationProgress,
    setAssignmentProgress,
    get viewState() {
      return store.getState();
    },
  };
}

const singleton = createExamAllocationStore();

export function useReactExamAllocationStore() {
  const state = useSyncExternalStore(
    singleton.store.subscribe,
    singleton.store.getState,
    singleton.store.getInitialState,
  );

  return {
    state,
    loadAll: singleton.loadAll,
    saveSettings: singleton.saveSettings,
    exportLatestBundle: singleton.exportLatestBundle,
    exportLatestInvigilationSchedule: singleton.exportLatestInvigilationSchedule,
    generate: singleton.generate,
    loadDetail: singleton.loadDetail,
    setFilters: singleton.setFilters,
    setSessionTimeGrade: singleton.setSessionTimeGrade,
    setSessionTimeDraft: singleton.setSessionTimeDraft,
    saveSessionTimes: singleton.saveSessionTimes,
    deleteSessionTime: singleton.deleteSessionTime,
    assignTeachers: singleton.assignTeachers,
    saveInvigilationConfig: singleton.saveInvigilationConfig,
    saveCustomRules: singleton.saveCustomRules,
    saveSelfStudyClassSubjects: singleton.saveSelfStudyClassSubjects,
    refreshGenerationProgress: singleton.refreshGenerationProgress,
    setAssignmentProgress: singleton.setAssignmentProgress,
  };
}
