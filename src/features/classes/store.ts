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

export type ClassNameIntent = "idle" | "switch" | "create" | "rename";

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
  function cloneForm(form: ClassConfigUpsertPayload): ClassConfigUpsertPayload {
    return {
      configType: form.configType,
      gradeName: form.gradeName,
      className: form.className,
      building: form.building,
      floor: form.floor,
      roomLabel: form.roomLabel,
      subjects: [...form.subjects],
    };
  }

  function normalizedForm(form: ClassConfigUpsertPayload, includeClassName = true) {
    return {
      configType: form.configType,
      gradeName: form.gradeName.trim(),
      className: includeClassName ? form.className.trim() : "",
      building: form.building.trim(),
      floor: form.floor.trim(),
      roomLabel: (form.roomLabel ?? "").trim(),
      subjects: [...form.subjects].sort(),
    };
  }

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
    baselineForm: { ...defaultForm } as ClassConfigUpsertPayload,
    classNameIntent: "idle" as ClassNameIntent,
    targetMatchId: null as number | null,
    originalClassName: "",
    isDirty: false,
    isDirtyExceptClassName: false,
    errorMessage: "",
  });

  function recalculateIntentAndDirty() {
    const name = state.form.className.trim();
    const normalizedName = name.replace(/\s+/g, "");
    const candidates = state.rows.filter((row) => row.configType === state.form.configType);
    const exactMatch = candidates.find((row) => row.className.trim().replace(/\s+/g, "") === normalizedName);
    const fuzzyMatches = candidates.filter((row) => {
      const rowName = row.className.trim().replace(/\s+/g, "");
      return normalizedName.length >= 2 && (rowName.includes(normalizedName) || normalizedName.includes(rowName));
    });
    const match = exactMatch ?? (fuzzyMatches.length === 1 ? fuzzyMatches[0] : undefined);
    const originalNormalized = state.originalClassName.replace(/\s+/g, "");
    const isEditingCurrentName = !!state.editingId && !!originalNormalized && normalizedName !== originalNormalized;
    const looksLikeRename =
      isEditingCurrentName &&
      (normalizedName.startsWith(originalNormalized) || originalNormalized.startsWith(normalizedName));

    if (match && state.editingId && match.id === state.editingId && isEditingCurrentName && looksLikeRename) {
      state.classNameIntent = "rename";
      state.targetMatchId = null;
    } else if (match && match.id !== state.editingId) {
      state.classNameIntent = "switch";
      state.targetMatchId = match.id;
    } else if (!normalizedName) {
      state.classNameIntent = "idle";
      state.targetMatchId = null;
    } else if (looksLikeRename) {
      state.classNameIntent = "rename";
      state.targetMatchId = null;
    } else {
      state.classNameIntent = "create";
      state.targetMatchId = null;
    }
    state.isDirty = JSON.stringify(normalizedForm(state.form)) !== JSON.stringify(normalizedForm(state.baselineForm));
    state.isDirtyExceptClassName =
      JSON.stringify(normalizedForm(state.form, false)) !== JSON.stringify(normalizedForm(state.baselineForm, false));
  }

  function resetForm(type: ClassConfigType = state.filters.configType) {
    state.form = {
      ...defaultForm,
      configType: type,
    };
    state.baselineForm = cloneForm(state.form);
    state.selectedId = null;
    state.detail = null;
    state.editingId = null;
    state.originalClassName = "";
    recalculateIntentAndDirty();
  }

  function setFormType(configType: ClassConfigType) {
    state.form.configType = configType;
    if (configType === "exam_room") {
      state.form.subjects = [];
    }
    recalculateIntentAndDirty();
  }

  function setFormField(
    field: "gradeName" | "className" | "building" | "floor" | "roomLabel",
    value: string | null,
  ) {
    if (field === "roomLabel") {
      state.form.roomLabel = value;
      recalculateIntentAndDirty();
      return;
    }
    state.form[field] = value ?? "";
    recalculateIntentAndDirty();
  }

  function toggleSubject(subject: Subject, checked: boolean) {
    if (checked) {
      if (!state.form.subjects.includes(subject)) {
        state.form.subjects = [...state.form.subjects, subject];
      }
      recalculateIntentAndDirty();
      return;
    }
    state.form.subjects = state.form.subjects.filter((item) => item !== subject);
    recalculateIntentAndDirty();
  }

  function startCreateFromClassName(className: string) {
    const nextName = className.trim();
    const configType = state.form.configType;
    const gradeName = state.form.gradeName;
    state.selectedId = null;
    state.detail = null;
    state.editingId = null;
    state.originalClassName = "";
    state.targetMatchId = null;
    state.form = {
      ...defaultForm,
      configType,
      gradeName,
      className: nextName,
    };
    state.baselineForm = cloneForm({
      ...defaultForm,
      configType,
      gradeName,
      className: nextName,
    });
    recalculateIntentAndDirty();
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
      state.baselineForm = cloneForm(state.form);
      state.originalClassName = detail.className.trim();
      recalculateIntentAndDirty();
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
      if (state.rows.length > 0) {
        await loadDetail(state.rows[0].id);
      }
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
    if (state.rows.length > 0) {
      await loadDetail(state.rows[0].id);
    } else {
      recalculateIntentAndDirty();
    }
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
      classNameIntent: state.classNameIntent,
      targetMatchId: state.targetMatchId,
      originalClassName: state.originalClassName,
      isDirty: state.isDirty,
      isDirtyExceptClassName: state.isDirtyExceptClassName,
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
    startCreateFromClassName,
    get viewState() {
      return viewState.value;
    },
  };
}

const classConfigStoreSingleton = createClassConfigStore();

export function useClassConfigStore() {
  return classConfigStoreSingleton;
}
