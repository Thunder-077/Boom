import { invoke } from "@tauri-apps/api/core";
import type {
  ClassConfigDetail,
  ClassConfigFilters,
  ClassConfigRow,
  ClassConfigUpsertPayload,
} from "../../entities/class-config/model";
import type { ListResult } from "../../shared/types/api";

export interface ClassConfigService {
  list(params: ClassConfigFilters): Promise<ListResult<ClassConfigRow>>;
  getById(id: number): Promise<ClassConfigDetail>;
  create(payload: ClassConfigUpsertPayload): Promise<{ id: number }>;
  update(id: number, payload: ClassConfigUpsertPayload): Promise<{ success: boolean }>;
  remove(id: number): Promise<{ success: boolean }>;
  listGradeOptions(): Promise<string[]>;
}

export const classConfigService: ClassConfigService = {
  list(params) {
    return invoke<ListResult<ClassConfigRow>>("list_class_configs", { params });
  },
  getById(id) {
    return invoke<ClassConfigDetail>("get_class_config_detail", { id });
  },
  create(payload) {
    return invoke<{ id: number }>("create_class_config", { payload });
  },
  update(id, payload) {
    return invoke<{ success: boolean }>("update_class_config", { id, payload });
  },
  remove(id) {
    return invoke<{ success: boolean }>("delete_class_config", { id });
  },
  listGradeOptions() {
    return invoke<string[]>("list_grade_options");
  },
};
