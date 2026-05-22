import { createStore } from "zustand/vanilla";
import { useSyncExternalStore } from "react";
import type {
  ClassConfigDetail,
  ClassConfigFilters,
  ClassConfigRow,
  ClassConfigType,
  ClassConfigUpsertPayload,
} from "../../entities/class-config/model";
import type { Subject } from "../../entities/score/model";
import { createVueViewState } from "../../shared/store/zustandVueBridge";
import { classConfigService, type ClassConfigService } from "./service";

export type ClassConfigMode = "existing" | "new";

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

interface ClassConfigStoreState {
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  filters: ClassConfigFilters;
  rows: ClassConfigRow[];
  total: number;
  gradeOptions: string[];
  selectedId: number | null;
  detail: ClassConfigDetail | null;
  editingId: number | null;
  mode: ClassConfigMode;
  form: ClassConfigUpsertPayload;
  baselineForm: ClassConfigUpsertPayload;
  loadedClassName: string;
  isDirty: boolean;
  errorMessage: string;
}

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

function normalizedForm(form: ClassConfigUpsertPayload) {
  return {
    configType: form.configType,
    gradeName: form.gradeName.trim(),
    className: form.className.trim(),
    building: form.building.trim(),
    floor: form.floor.trim(),
    roomLabel: (form.roomLabel ?? "").trim(),
    subjects: [...form.subjects].sort(),
  };
}

function isDirty(form: ClassConfigUpsertPayload, baselineForm: ClassConfigUpsertPayload) {
  return JSON.stringify(normalizedForm(form)) !== JSON.stringify(normalizedForm(baselineForm));
}

function createDefaultState(): ClassConfigStoreState {
  const form = cloneForm(defaultForm);
  return {
    loading: false,
    saving: false,
    deleting: false,
    filters: { ...defaultFilters },
    rows: [],
    total: 0,
    gradeOptions: [],
    selectedId: null,
    detail: null,
    editingId: null,
    mode: "new",
    form,
    baselineForm: cloneForm(form),
    loadedClassName: "",
    isDirty: false,
    errorMessage: "",
  };
}

export function createClassConfigStore(service: ClassConfigService = classConfigService) {
  const store = createStore<ClassConfigStoreState>(() => createDefaultState());
  const viewState = createVueViewState(store);

  function resetForm(type: ClassConfigType = store.getState().filters.configType) {
    const form = {
      ...cloneForm(defaultForm),
      configType: type,
    };
    const baselineForm = cloneForm(form);
    store.setState({
      form,
      baselineForm,
      selectedId: null,
      detail: null,
      editingId: null,
      mode: "new",
      loadedClassName: "",
      isDirty: isDirty(form, baselineForm),
    });
  }

  function setFormType(configType: ClassConfigType) {
    store.setState((state) => {
      const form = {
        ...state.form,
        configType,
        subjects: configType === "exam_room" ? [] : [...state.form.subjects],
      };
      return {
        form,
        isDirty: isDirty(form, state.baselineForm),
      };
    });
  }

  function setFormField(
    field: "gradeName" | "className" | "building" | "floor" | "roomLabel",
    value: string | null,
  ) {
    store.setState((state) => {
      const form = {
        ...state.form,
        [field]: field === "roomLabel" ? value : value ?? "",
      };
      return {
        form,
        isDirty: isDirty(form, state.baselineForm),
      };
    });
  }

  function toggleSubject(subject: Subject, checked: boolean) {
    store.setState((state) => {
      const subjects = checked
        ? state.form.subjects.includes(subject)
          ? state.form.subjects
          : [...state.form.subjects, subject]
        : state.form.subjects.filter((item) => item !== subject);
      const form = {
        ...state.form,
        subjects,
      };
      return {
        form,
        isDirty: isDirty(form, state.baselineForm),
      };
    });
  }

  function startCreate(className: string) {
    const nextName = className.trim();
    const configType = store.getState().form.configType;
    const form = {
      ...cloneForm(defaultForm),
      configType,
      className: nextName,
    };
    const baselineForm = cloneForm(form);
    store.setState({
      selectedId: null,
      detail: null,
      editingId: null,
      mode: "new",
      loadedClassName: "",
      form,
      baselineForm,
      isDirty: isDirty(form, baselineForm),
    });
  }

  function discardChanges() {
    store.setState((state) => {
      const form = cloneForm(state.baselineForm);
      return {
        form,
        isDirty: isDirty(form, state.baselineForm),
      };
    });
  }

  async function loadList() {
    store.setState({ loading: true, errorMessage: "" });
    try {
      const { filters } = store.getState();
      const [listResult, grades] = await Promise.all([service.list(filters), service.listGradeOptions()]);
      store.setState({
        rows: listResult.items,
        total: listResult.total,
        gradeOptions: grades,
      });
    } catch (error) {
      store.setState({ errorMessage: error instanceof Error ? error.message : String(error) });
    } finally {
      store.setState({ loading: false });
    }
  }

  async function loadDetail(id: number) {
    store.setState({ selectedId: id, errorMessage: "" });
    try {
      const detail = await service.getById(id);
      const form = {
        configType: detail.configType,
        gradeName: detail.gradeName,
        className: detail.className,
        building: detail.building,
        floor: detail.floor,
        roomLabel: detail.roomLabel,
        subjects: [...detail.subjects],
      };
      const baselineForm = cloneForm(form);
      store.setState({
        detail,
        editingId: id,
        mode: "existing",
        form,
        baselineForm,
        loadedClassName: detail.className.trim(),
        isDirty: isDirty(form, baselineForm),
      });
    } catch (error) {
      store.setState({ errorMessage: error instanceof Error ? error.message : String(error) });
    }
  }

  async function loadInitial() {
    resetForm(store.getState().filters.configType);
    await loadList();
    const { rows } = store.getState();
    if (rows.length > 0) {
      await loadDetail(rows[0].id);
    } else {
      store.setState((state) => ({ isDirty: isDirty(state.form, state.baselineForm) }));
    }
  }

  async function create() {
    store.setState({ saving: true, errorMessage: "" });
    try {
      const { form } = store.getState();
      const { id } = await service.create(form);
      await loadList();
      await loadDetail(id);
    } catch (error) {
      store.setState({ errorMessage: error instanceof Error ? error.message : String(error) });
      throw error;
    } finally {
      store.setState({ saving: false });
    }
  }

  async function update() {
    const { editingId, form } = store.getState();
    if (!editingId) {
      return;
    }
    store.setState({ saving: true, errorMessage: "" });
    try {
      await service.update(editingId, form);
      await loadList();
      await loadDetail(editingId);
    } catch (error) {
      store.setState({ errorMessage: error instanceof Error ? error.message : String(error) });
      throw error;
    } finally {
      store.setState({ saving: false });
    }
  }

  async function remove(id: number) {
    store.setState({ deleting: true, errorMessage: "" });
    try {
      await service.remove(id);
      await loadList();
      const { rows, filters } = store.getState();
      if (rows.length > 0) {
        await loadDetail(rows[0].id);
      } else {
        resetForm(filters.configType);
      }
    } catch (error) {
      store.setState({ errorMessage: error instanceof Error ? error.message : String(error) });
      throw error;
    } finally {
      store.setState({ deleting: false });
    }
  }

  async function setFilters(next: Partial<ClassConfigFilters>) {
    store.setState((state) => {
      const filters = {
        ...state.filters,
        ...next,
      };
      if (filters.configType === "exam_room") {
        filters.gradeName = "";
      }
      return { filters };
    });
    await loadInitial();
  }

  return {
    store,
    loadInitial,
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
    startCreate,
    discardChanges,
    get viewState() {
      return viewState;
    },
  };
}

const classConfigStoreSingleton = createClassConfigStore();

export function useClassConfigStore() {
  return classConfigStoreSingleton;
}

export function useReactClassConfigStore() {
  const state = useSyncExternalStore(
    classConfigStoreSingleton.store.subscribe,
    classConfigStoreSingleton.store.getState,
    classConfigStoreSingleton.store.getInitialState,
  );

  return {
    state,
    loadInitial: classConfigStoreSingleton.loadInitial,
    loadList: classConfigStoreSingleton.loadList,
    loadDetail: classConfigStoreSingleton.loadDetail,
    create: classConfigStoreSingleton.create,
    update: classConfigStoreSingleton.update,
    remove: classConfigStoreSingleton.remove,
    setFilters: classConfigStoreSingleton.setFilters,
    resetForm: classConfigStoreSingleton.resetForm,
    setFormType: classConfigStoreSingleton.setFormType,
    setFormField: classConfigStoreSingleton.setFormField,
    toggleSubject: classConfigStoreSingleton.toggleSubject,
    startCreate: classConfigStoreSingleton.startCreate,
    discardChanges: classConfigStoreSingleton.discardChanges,
  };
}
