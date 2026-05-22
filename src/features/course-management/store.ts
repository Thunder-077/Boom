import { useSyncExternalStore } from "react";
import { createStore } from "zustand/vanilla";
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
import { createVueViewState } from "../../shared/store/zustandVueBridge";
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

interface CourseManagementStoreState {
  loading: boolean;
  viewType: CourseViewType;
  target: string;
  summary: CourseSummary;
  imports: CourseImportBatch[];
  selectedImportId: number | null;
  selectedImport: CourseImportBatch | null;
  settingsDraft: {
    effectiveStartDate: string;
    effectiveEndDate: string;
    startWeek: number;
  };
  adminClasses: CourseClassOption[];
  foreignClasses: CourseClassOption[];
  teachers: string[];
  periods: CoursePeriodSlot[];
  schedule: CourseScheduleView | null;
  substitutionCandidates: CourseSubstitutionCandidate[];
  scheduleChanges: CourseScheduleChange[];
  workloadReport: CourseWorkloadReport | null;
  exportingWorkload: boolean;
  lastWorkloadExport: ExportCourseWorkloadResult | null;
  importStatus: ImportStatus;
  importMessage: string;
  lastImportResult: CourseImportResult | null;
}

function createDefaultState(): CourseManagementStoreState {
  return {
    loading: false,
    viewType: "admin_class",
    target: "",
    summary: { ...emptySummary },
    imports: [],
    selectedImportId: null,
    selectedImport: null,
    settingsDraft: {
      effectiveStartDate: "",
      effectiveEndDate: "",
      startWeek: 1,
    },
    adminClasses: [],
    foreignClasses: [],
    teachers: [],
    periods: [],
    schedule: null,
    substitutionCandidates: [],
    scheduleChanges: [],
    workloadReport: null,
    exportingWorkload: false,
    lastWorkloadExport: null,
    importStatus: "idle",
    importMessage: "",
    lastImportResult: null,
  };
}

function selectedImportFrom(imports: CourseImportBatch[], selectedImportId: number | null) {
  return imports.find((item) => item.id === selectedImportId) ?? null;
}

export function createCourseManagementStore(service: CourseManagementService = courseManagementService) {
  const store = createStore<CourseManagementStoreState>(() => createDefaultState());
  const viewState = createVueViewState(store);

  function defaultTargetFor(viewType: CourseViewType) {
    const state = store.getState();
    if (viewType === "teacher") return state.teachers[0] ?? "";
    if (viewType === "foreign_class") return state.foreignClasses[0]?.className ?? "";
    return state.adminClasses[0]?.className ?? "";
  }

  function syncSelectedImportDraft() {
    store.setState((state) => {
      const selectedImport = selectedImportFrom(state.imports, state.selectedImportId);
      return {
        selectedImport,
        settingsDraft: {
          effectiveStartDate: selectedImport?.effectiveStartDate ?? "",
          effectiveEndDate: selectedImport?.effectiveEndDate ?? "",
          startWeek: selectedImport?.startWeek ?? 1,
        },
      };
    });
  }

  async function loadClassesForSelectedImport() {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) {
      store.setState({
        adminClasses: [],
        foreignClasses: [],
        teachers: [],
      });
      return;
    }
    const [adminClasses, foreignClasses] = await Promise.all([
      service.listClasses("admin", selectedImportId),
      service.listClasses("foreign", selectedImportId),
    ]);
    const teachers = await service.listTeachers(selectedImportId);
    store.setState({
      adminClasses,
      foreignClasses,
      teachers,
    });
  }

  async function loadPeriodsForSelectedImport() {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) {
      store.setState({ periods: [] });
      return;
    }
    store.setState({ periods: await service.listPeriods(selectedImportId) });
  }

  function targetExists(viewType: CourseViewType, target: string) {
    const state = store.getState();
    if (!target) return false;
    if (viewType === "teacher") return state.teachers.includes(target);
    if (viewType === "foreign_class") return state.foreignClasses.some((item) => item.className === target);
    return state.adminClasses.some((item) => item.className === target);
  }

  async function loadSchedule() {
    const { target, selectedImportId, viewType } = store.getState();
    if (!target || !selectedImportId) {
      store.setState({ schedule: null });
      return;
    }
    store.setState({
      schedule: await service.getScheduleView({
        viewType,
        target,
        importId: selectedImportId,
      }),
    });
  }

  async function loadScheduleChanges() {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) {
      store.setState({ scheduleChanges: [] });
      return;
    }
    store.setState({ scheduleChanges: await service.listScheduleChanges(selectedImportId) });
  }

  async function loadOptions() {
    store.setState({ loading: true });
    try {
      const [summary, imports] = await Promise.all([
        service.getSummary(),
        service.listImports(),
      ]);
      const currentSelectedId = store.getState().selectedImportId;
      const selectedStillExists = imports.some((item) => item.id === currentSelectedId);
      const selectedImportId = selectedStillExists ? currentSelectedId : imports[0]?.id ?? summary.latestImportId;
      store.setState({
        summary,
        imports,
        selectedImportId,
        selectedImport: selectedImportFrom(imports, selectedImportId),
      });
      syncSelectedImportDraft();
      await loadClassesForSelectedImport();
      await loadPeriodsForSelectedImport();
      await loadScheduleChanges();
      const { viewType, target } = store.getState();
      if (!targetExists(viewType, target)) {
        store.setState({ target: defaultTargetFor(viewType) });
      }
      await loadSchedule();
    } finally {
      store.setState({ loading: false });
    }
  }

  async function setViewType(viewType: CourseViewType) {
    store.setState({
      viewType,
      target: defaultTargetFor(viewType),
    });
    await loadSchedule();
  }

  async function setTarget(target: string) {
    store.setState({ target });
    await loadSchedule();
  }

  async function setSelectedImport(importId: number) {
    const selectedImportId = Number(importId) || null;
    store.setState((state) => ({
      selectedImportId,
      selectedImport: selectedImportFrom(state.imports, selectedImportId),
    }));
    syncSelectedImportDraft();
    await loadClassesForSelectedImport();
    await loadPeriodsForSelectedImport();
    await loadScheduleChanges();
    const { viewType, target } = store.getState();
    if (!targetExists(viewType, target)) {
      store.setState({ target: defaultTargetFor(viewType) });
    }
    await loadSchedule();
  }

  function setSettingsDraft(patch: Partial<CourseManagementStoreState["settingsDraft"]>) {
    store.setState((state) => ({
      settingsDraft: {
        ...state.settingsDraft,
        ...patch,
      },
    }));
  }

  async function saveSelectedImportSettings() {
    const { selectedImportId, settingsDraft } = store.getState();
    if (!selectedImportId) return;
    const updated = await service.updateImportSettings({
      importId: selectedImportId,
      effectiveStartDate: settingsDraft.effectiveStartDate || null,
      effectiveEndDate: settingsDraft.effectiveEndDate || null,
      startWeek: Math.max(1, Number(settingsDraft.startWeek) || 1),
    });
    store.setState((state) => {
      const imports = state.imports.map((item) => (item.id === updated.id ? updated : item));
      return {
        imports,
        selectedImport: selectedImportFrom(imports, state.selectedImportId),
      };
    });
    syncSelectedImportDraft();
  }

  async function deleteSelectedImport() {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) return;
    await service.deleteImport(selectedImportId);
    store.setState({
      target: "",
      schedule: null,
      substitutionCandidates: [],
      scheduleChanges: [],
      workloadReport: null,
      selectedImportId: null,
      selectedImport: null,
    });
    await loadOptions();
  }

  async function importExcel(filePath: string) {
    store.setState({
      importStatus: "importing",
      importMessage: "正在解析并导入课表...",
    });
    try {
      const result = await service.importExcel(filePath);
      store.setState({
        lastImportResult: result,
        importStatus: "success",
        importMessage: `导入 ${result.entryCount} 节课，更新 ${result.teacherCount} 位教师，耗时 ${result.durationMs}ms`,
        target: "",
        selectedImportId: null,
        selectedImport: null,
      });
      await loadOptions();
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

  async function findSubstitutionCandidates(query: Omit<CourseSubstitutionCandidateQuery, "importId">) {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) {
      store.setState({ substitutionCandidates: [] });
      return [];
    }
    const candidates = await service.listSubstitutionCandidates({
      ...query,
      importId: selectedImportId,
    });
    store.setState({ substitutionCandidates: candidates });
    return candidates;
  }

  async function saveSubstitutions(payload: Omit<SaveCourseSubstitutionsPayload, "importId">) {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) return [];
    const scheduleChanges = await service.saveSubstitutions({
      ...payload,
      importId: selectedImportId,
    });
    store.setState({ scheduleChanges });
    return scheduleChanges;
  }

  async function revokeScheduleChange(changeId: number) {
    await service.revokeScheduleChange(changeId);
    await loadScheduleChanges();
  }

  async function loadWorkloadReport(query: Omit<CourseWorkloadQuery, "importId">) {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) {
      store.setState({ workloadReport: null });
      return null;
    }
    const report = await service.getWorkloadReport({
      ...query,
      importId: selectedImportId,
    });
    store.setState({ workloadReport: report });
    return report;
  }

  async function exportWorkloadReport(query: Omit<CourseWorkloadQuery, "importId">) {
    const { selectedImportId } = store.getState();
    if (!selectedImportId) return null;
    store.setState({ exportingWorkload: true });
    try {
      const result = await service.exportWorkloadReport({
        ...query,
        importId: selectedImportId,
      });
      store.setState({ lastWorkloadExport: result });
      return result;
    } finally {
      store.setState({ exportingWorkload: false });
    }
  }

  return {
    store,
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
      return viewState;
    },
  };
}

const courseManagementStore = createCourseManagementStore();

export function useCourseManagementStore() {
  return courseManagementStore;
}

export function useReactCourseManagementStore() {
  const state = useSyncExternalStore(
    courseManagementStore.store.subscribe,
    courseManagementStore.store.getState,
    courseManagementStore.store.getInitialState,
  );

  return {
    state,
    loadOptions: courseManagementStore.loadOptions,
    loadSchedule: courseManagementStore.loadSchedule,
    setViewType: courseManagementStore.setViewType,
    setTarget: courseManagementStore.setTarget,
    setSelectedImport: courseManagementStore.setSelectedImport,
    setSettingsDraft: courseManagementStore.setSettingsDraft,
    saveSelectedImportSettings: courseManagementStore.saveSelectedImportSettings,
    deleteSelectedImport: courseManagementStore.deleteSelectedImport,
    importExcel: courseManagementStore.importExcel,
    setImportFeedback: courseManagementStore.setImportFeedback,
    loadScheduleChanges: courseManagementStore.loadScheduleChanges,
    findSubstitutionCandidates: courseManagementStore.findSubstitutionCandidates,
    saveSubstitutions: courseManagementStore.saveSubstitutions,
    revokeScheduleChange: courseManagementStore.revokeScheduleChange,
    loadWorkloadReport: courseManagementStore.loadWorkloadReport,
    exportWorkloadReport: courseManagementStore.exportWorkloadReport,
  };
}
