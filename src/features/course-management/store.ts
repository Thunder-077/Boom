import { computed, reactive, readonly } from "vue";
import type {
  CourseClassOption,
  CourseImportBatch,
  CourseImportResult,
  CourseScheduleChange,
  CoursePeriodSlot,
  CourseSubstitutionCandidate,
  CourseSubstitutionCandidateQuery,
  CourseScheduleView,
  CourseSummary,
  CourseViewType,
  SaveCourseSubstitutionsPayload,
  CourseWorkloadQuery,
  CourseWorkloadReport,
  ExportCourseWorkloadResult,
} from "../../entities/course-management/model";
import { courseManagementService, type CourseManagementService } from "./service";

type ImportStatus = "idle" | "importing" | "success" | "error";

const emptySummary: CourseSummary = {
  latestImportId: null,
  importedAt: null,
  entryCount: 0,
  teacherCount: 0,
  adminClassCount: 0,
  foreignClassCount: 0,
  effectiveStartDate: null,
  effectiveEndDate: null,
  startWeek: 1,
};

export function createCourseManagementStore(service: CourseManagementService = courseManagementService) {
  const state = reactive({
    loading: false,
    viewType: "admin_class" as CourseViewType,
    target: "",
    summary: { ...emptySummary } as CourseSummary,
    imports: [] as CourseImportBatch[],
    selectedImportId: null as number | null,
    settingsDraft: {
      effectiveStartDate: "",
      effectiveEndDate: "",
      startWeek: 1,
    },
    adminClasses: [] as CourseClassOption[],
    foreignClasses: [] as CourseClassOption[],
    teachers: [] as string[],
    periods: [] as CoursePeriodSlot[],
    schedule: null as CourseScheduleView | null,
    substitutionCandidates: [] as CourseSubstitutionCandidate[],
    scheduleChanges: [] as CourseScheduleChange[],
    workloadReport: null as CourseWorkloadReport | null,
    exportingWorkload: false,
    lastWorkloadExport: null as ExportCourseWorkloadResult | null,
    importStatus: "idle" as ImportStatus,
    importMessage: "",
    lastImportResult: null as CourseImportResult | null,
  });

  async function loadOptions() {
    state.loading = true;
    try {
      const [summary, imports] = await Promise.all([
        service.getSummary(),
        service.listImports(),
      ]);
      state.summary = summary;
      state.imports = imports;
      const selectedStillExists = imports.some((item) => item.id === state.selectedImportId);
      state.selectedImportId = selectedStillExists ? state.selectedImportId : imports[0]?.id ?? summary.latestImportId;
      syncSettingsDraft();
      await loadClassesForSelectedImport();
      await loadPeriodsForSelectedImport();
      await loadScheduleChanges();
      if (!targetExists(state.viewType, state.target)) {
        state.target = defaultTargetFor(state.viewType);
      }
      await loadSchedule();
    } finally {
      state.loading = false;
    }
  }

  function defaultTargetFor(viewType: CourseViewType) {
    if (viewType === "teacher") return state.teachers[0] ?? "";
    if (viewType === "foreign_class") return state.foreignClasses[0]?.className ?? "";
    return state.adminClasses[0]?.className ?? "";
  }

  function selectedImport() {
    return state.imports.find((item) => item.id === state.selectedImportId) ?? null;
  }

  function syncSettingsDraft() {
    const batch = selectedImport();
    state.settingsDraft.effectiveStartDate = batch?.effectiveStartDate ?? "";
    state.settingsDraft.effectiveEndDate = batch?.effectiveEndDate ?? "";
    state.settingsDraft.startWeek = batch?.startWeek ?? 1;
  }

  async function loadClassesForSelectedImport() {
    if (!state.selectedImportId) {
      state.adminClasses = [];
      state.foreignClasses = [];
      state.teachers = [];
      return;
    }
    const [adminClasses, foreignClasses] = await Promise.all([
      service.listClasses("admin", state.selectedImportId),
      service.listClasses("foreign", state.selectedImportId),
    ]);
    state.adminClasses = adminClasses;
    state.foreignClasses = foreignClasses;
    state.teachers = await service.listTeachers(state.selectedImportId);
  }

  async function loadPeriodsForSelectedImport() {
    if (!state.selectedImportId) {
      state.periods = [];
      return;
    }
    state.periods = await service.listPeriods(state.selectedImportId);
  }

  function targetExists(viewType: CourseViewType, target: string) {
    if (!target) return false;
    if (viewType === "teacher") return state.teachers.includes(target);
    if (viewType === "foreign_class") return state.foreignClasses.some((item) => item.className === target);
    return state.adminClasses.some((item) => item.className === target);
  }

  async function loadSchedule() {
    if (!state.target || !state.selectedImportId) {
      state.schedule = null;
      return;
    }
    state.schedule = await service.getScheduleView({
      viewType: state.viewType,
      target: state.target,
      importId: state.selectedImportId,
    });
  }

  async function setViewType(viewType: CourseViewType) {
    state.viewType = viewType;
    state.target = defaultTargetFor(viewType);
    await loadSchedule();
  }

  async function setTarget(target: string) {
    state.target = target;
    await loadSchedule();
  }

  async function setSelectedImport(importId: number) {
    state.selectedImportId = Number(importId) || null;
    syncSettingsDraft();
    await loadClassesForSelectedImport();
    await loadPeriodsForSelectedImport();
    await loadScheduleChanges();
    if (!targetExists(state.viewType, state.target)) {
      state.target = defaultTargetFor(state.viewType);
    }
    await loadSchedule();
  }

  function setSettingsDraft(patch: Partial<typeof state.settingsDraft>) {
    Object.assign(state.settingsDraft, patch);
  }

  async function saveSelectedImportSettings() {
    if (!state.selectedImportId) return;
    const updated = await service.updateImportSettings({
      importId: state.selectedImportId,
      effectiveStartDate: state.settingsDraft.effectiveStartDate || null,
      effectiveEndDate: state.settingsDraft.effectiveEndDate || null,
      startWeek: Math.max(1, Number(state.settingsDraft.startWeek) || 1),
    });
    state.imports = state.imports.map((item) => (item.id === updated.id ? updated : item));
    syncSettingsDraft();
  }

  async function deleteSelectedImport() {
    if (!state.selectedImportId) return;
    await service.deleteImport(state.selectedImportId);
    state.target = "";
    state.schedule = null;
    state.substitutionCandidates = [];
    state.scheduleChanges = [];
    state.workloadReport = null;
    state.selectedImportId = null;
    await loadOptions();
  }

  async function importExcel(filePath: string) {
    state.importStatus = "importing";
    state.importMessage = "正在解析并导入课表...";
    try {
      const result = await service.importExcel(filePath);
      state.lastImportResult = result;
      state.importStatus = "success";
      state.importMessage = `导入 ${result.entryCount} 节课，更新 ${result.teacherCount} 位教师，耗时 ${result.durationMs}ms`;
      state.target = "";
      state.selectedImportId = null;
      await loadOptions();
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

  async function loadScheduleChanges() {
    if (!state.selectedImportId) {
      state.scheduleChanges = [];
      return;
    }
    state.scheduleChanges = await service.listScheduleChanges(state.selectedImportId);
  }

  async function findSubstitutionCandidates(query: Omit<CourseSubstitutionCandidateQuery, "importId">) {
    if (!state.selectedImportId) {
      state.substitutionCandidates = [];
      return [];
    }
    const candidates = await service.listSubstitutionCandidates({
      ...query,
      importId: state.selectedImportId,
    });
    state.substitutionCandidates = candidates;
    return candidates;
  }

  async function saveSubstitutions(payload: Omit<SaveCourseSubstitutionsPayload, "importId">) {
    if (!state.selectedImportId) return [];
    state.scheduleChanges = await service.saveSubstitutions({
      ...payload,
      importId: state.selectedImportId,
    });
    return state.scheduleChanges;
  }

  async function revokeScheduleChange(changeId: number) {
    await service.revokeScheduleChange(changeId);
    await loadScheduleChanges();
  }

  async function loadWorkloadReport(query: Omit<CourseWorkloadQuery, "importId">) {
    if (!state.selectedImportId) {
      state.workloadReport = null;
      return null;
    }
    const report = await service.getWorkloadReport({
      ...query,
      importId: state.selectedImportId,
    });
    state.workloadReport = report;
    return report;
  }

  async function exportWorkloadReport(query: Omit<CourseWorkloadQuery, "importId">) {
    if (!state.selectedImportId) return null;
    state.exportingWorkload = true;
    try {
      const result = await service.exportWorkloadReport({
        ...query,
        importId: state.selectedImportId,
      });
      state.lastWorkloadExport = result;
      return result;
    } finally {
      state.exportingWorkload = false;
    }
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      viewType: state.viewType,
      target: state.target,
      summary: state.summary,
      imports: state.imports,
      selectedImportId: state.selectedImportId,
      settingsDraft: state.settingsDraft,
      selectedImport: selectedImport(),
      adminClasses: state.adminClasses,
      foreignClasses: state.foreignClasses,
      teachers: state.teachers,
      periods: state.periods,
      schedule: state.schedule,
      substitutionCandidates: state.substitutionCandidates,
      scheduleChanges: state.scheduleChanges,
      workloadReport: state.workloadReport,
      exportingWorkload: state.exportingWorkload,
      lastWorkloadExport: state.lastWorkloadExport,
      importStatus: state.importStatus,
      importMessage: state.importMessage,
      lastImportResult: state.lastImportResult,
    })),
  );

  return {
    loadOptions,
    loadSchedule,
    setViewType,
    setTarget,
    setSelectedImport,
    setSettingsDraft,
    saveSelectedImportSettings,
    deleteSelectedImport,
    importExcel,
    setImportFeedback,
    loadScheduleChanges,
    findSubstitutionCandidates,
    saveSubstitutions,
    revokeScheduleChange,
    loadWorkloadReport,
    exportWorkloadReport,
    get viewState() {
      return viewState.value;
    },
  };
}

const courseManagementStore = createCourseManagementStore();

export function useCourseManagementStore() {
  return courseManagementStore;
}
