import { describe, expect, it, vi } from "vitest";
import { createClassConfigStore } from "../store";
import type { ClassConfigService } from "../service";
import { Subject } from "../../../entities/score/model";

function createFakeService(overrides?: Partial<ClassConfigService>): ClassConfigService {
  const list = vi.fn(async () => ({
    total: 2,
    items: [
      {
        id: 1,
        configType: "teaching_class" as const,
        gradeName: "高二",
        className: "高二1班",
        building: "教学楼A",
        floor: "3层",
        roomLabel: null,
        updatedAt: "2026-03-24T10:00:00Z",
      },
      {
        id: 2,
        configType: "teaching_class" as const,
        gradeName: "高二",
        className: "高二2班",
        building: "教学楼A",
        floor: "4层",
        roomLabel: null,
        updatedAt: "2026-03-24T10:00:00Z",
      },
    ],
  }));

  const getById = vi.fn(async (id: number) => ({
    id,
    configType: "teaching_class" as const,
    gradeName: "高二",
    className: id === 1 ? "高二1班" : "高二2班",
    building: id === 1 ? "教学楼A" : "教学楼B",
    floor: id === 1 ? "3层" : "4层",
    roomLabel: null,
    subjects: [Subject.Chinese, Subject.Math, Subject.Physics],
    updatedAt: "2026-03-24T10:00:00Z",
  }));

  const create = vi.fn(async () => ({ id: 2 }));
  const update = vi.fn(async () => ({ success: true }));
  const remove = vi.fn(async () => ({ success: true }));
  const listGradeOptions = vi.fn(async () => ["高一", "高二"]);

  return {
    list,
    getById,
    create,
    update,
    remove,
    listGradeOptions,
    ...overrides,
  };
}

describe("class config store", () => {
  it("loads default row on filter init", async () => {
    const store = createClassConfigStore(createFakeService());
    await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });

    expect(store.viewState.total).toBe(2);
    expect(store.viewState.mode).toBe("existing");
    expect(store.viewState.editingId).toBe(1);
    expect(store.viewState.form.className).toBe("高二1班");
  });

  it("supports load and update flow", async () => {
    const service = createFakeService();
    const store = createClassConfigStore(service);

    await store.loadInitial();
    store.setFormField("building", "教学楼C");
    await store.update();

    expect(service.update).toHaveBeenCalledWith(1, expect.objectContaining({ building: "教学楼C" }));
    expect(store.viewState.errorMessage).toBe("");
  });

  it("starts create mode with blank related fields", async () => {
    const store = createClassConfigStore(createFakeService());
    await store.loadInitial();

    store.startCreate("高二9班");

    expect(store.viewState.mode).toBe("new");
    expect(store.viewState.editingId).toBeNull();
    expect(store.viewState.form.className).toBe("高二9班");
    expect(store.viewState.form.building).toBe("");
    expect(store.viewState.form.floor).toBe("");
    expect(store.viewState.form.subjects).toEqual([]);
  });

  it("marks form dirty after editing existing config", async () => {
    const store = createClassConfigStore(createFakeService());
    await store.loadInitial();

    expect(store.viewState.isDirty).toBe(false);
    store.setFormField("building", "教学楼D");
    expect(store.viewState.isDirty).toBe(true);
  });

  it("resets to new mode when deleting the last config", async () => {
    const service = createFakeService({
      list: vi
        .fn()
        .mockResolvedValueOnce({
          total: 1,
          items: [
            {
              id: 1,
              configType: "teaching_class",
              gradeName: "高二",
              className: "高二1班",
              building: "教学楼A",
              floor: "3层",
              roomLabel: null,
              updatedAt: "2026-03-24T10:00:00Z",
            },
          ],
        })
        .mockResolvedValueOnce({ total: 0, items: [] }),
    });
    const store = createClassConfigStore(service);

    await store.loadInitial();
    await store.remove(1);

    expect(store.viewState.mode).toBe("new");
    expect(store.viewState.editingId).toBeNull();
    expect(store.viewState.form.className).toBe("");
  });
});
