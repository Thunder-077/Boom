import { computed, reactive, readonly } from "vue";
import type {
  ClassConfigDetail,
  ClassConfigFilters,
  ClassConfigRow,
  ClassConfigType,
  ClassConfigUpsertPayload,
} from "../../entities/class-config/model";
import type { Subject } from "../../entities/score/model";
import { classConfigService, type ClassConfigService } from "./service";

const defaultFilters: ClassConfigFilters = {
  configType: "teaching_class",
  gradeName: "",
  keyword: "",
};

const defaultForm: ClassConfigUpsertPayload = {
  configType: "teaching_class",
  gradeName: "",
  className: "",
  building: "",
  floor: "",
  roomLabel: null,
  subjects: [],
};

export function createClassConfigStore(service: ClassConfigService = classConfigService) {
  const state = reactive({
    loading: false,
    saving: false,
    deleting: false,
    filters: { ...defaultFilters },
    rows: [] as ClassConfigRow[],
    total: 0,
    gradeOptions: [] as string[],
    selectedId: null as number | null,
    detail: null as ClassConfigDetail | null,
    editingId: null as number | null,
    form: { ...defaultForm } as ClassConfigUpsertPayload,
    errorMessage: "",
  });

  function resetForm(type: ClassConfigType = state.filters.configType) {
    state.form = {
      ...defaultForm,
      configType: type,
    };
    state.editingId = null;
  }

  function setFormType(configType: ClassConfigType) {
    state.form.configType = configType;
    if (configType === "exam_room") {
      state.form.subjects = [];
    }
  }

  function setFormField(
    field: "gradeName" | "className" | "building" | "floor" | "roomLabel",
    value: string | null,
  ) {
    if (field === "roomLabel") {
      state.form.roomLabel = value;
      return;
    }
    state.form[field] = value ?? "";
  }

  function toggleSubject(subject: Subject, checked: boolean) {
    if (checked) {
      if (!state.form.subjects.includes(subject)) {
        state.form.subjects = [...state.form.subjects, subject];
      }
      return;
    }
    state.form.subjects = state.form.subjects.filter((item) => item !== subject);
  }

  async function loadList() {
    state.loading = true;
    state.errorMessage = "";
    try {
      const [listResult, grades] = await Promise.all([service.list(state.filters), service.listGradeOptions()]);
      state.rows = listResult.items;
      state.total = listResult.total;
      state.gradeOptions = grades;
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      state.loading = false;
    }
  }

  async function loadDetail(id: number) {
    state.selectedId = id;
    state.errorMessage = "";
    try {
      const detail = await service.getById(id);
      state.detail = detail;
      state.editingId = id;
      state.form = {
        configType: detail.configType,
        gradeName: detail.gradeName,
        className: detail.className,
        building: detail.building,
        floor: detail.floor,
        roomLabel: detail.roomLabel,
        subjects: [...detail.subjects],
      };
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
    }
  }

  async function create() {
    state.saving = true;
    state.errorMessage = "";
    try {
      const { id } = await service.create(state.form);
      await loadList();
      await loadDetail(id);
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.saving = false;
    }
  }

  async function update() {
    if (!state.editingId) {
      return;
    }
    state.saving = true;
    state.errorMessage = "";
    try {
      await service.update(state.editingId, state.form);
      await loadList();
      await loadDetail(state.editingId);
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.saving = false;
    }
  }

  async function remove(id: number) {
    state.deleting = true;
    state.errorMessage = "";
    try {
      await service.remove(id);
      if (state.selectedId === id) {
        state.selectedId = null;
        state.detail = null;
        resetForm(state.filters.configType);
      }
      await loadList();
    } catch (error) {
      state.errorMessage = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.deleting = false;
    }
  }

  async function setFilters(next: Partial<ClassConfigFilters>) {
    state.filters = {
      ...state.filters,
      ...next,
    };
    if (state.filters.configType === "exam_room") {
      state.filters.gradeName = "";
    }
    resetForm(state.filters.configType);
    await loadList();
  }

  const viewState = readonly(
    computed(() => ({
      loading: state.loading,
      saving: state.saving,
      deleting: state.deleting,
      filters: state.filters,
      rows: state.rows,
      total: state.total,
      gradeOptions: state.gradeOptions,
      selectedId: state.selectedId,
      detail: state.detail,
      editingId: state.editingId,
      form: state.form,
      errorMessage: state.errorMessage,
    })),
  );

  return {
    loadList,
    loadDetail,
    create,
    update,
    remove,
    setFilters,
    resetForm,
    setFormType,
    setFormField,
    toggleSubject,
    get viewState() {
      return viewState.value;
    },
  };
}

const classConfigStoreSingleton = createClassConfigStore();

export function useClassConfigStore() {
  return classConfigStoreSingleton;
}
