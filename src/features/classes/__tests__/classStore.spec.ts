import { describe, expect, it } from "vitest";
import { createClassConfigStore } from "../store";
import type { ClassConfigService } from "../service";
import { Subject } from "../../../entities/score/model";

const fakeService: ClassConfigService = {
  async list() {
    return {
      total: 2,
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
        {
          id: 2,
          configType: "teaching_class",
          gradeName: "高二",
          className: "高二2班",
          building: "教学楼A",
          floor: "4层",
          roomLabel: null,
          updatedAt: "2026-03-24T10:00:00Z",
        },
      ],
    };
  },
  async getById(id) {
    return {
      id,
      configType: "teaching_class",
      gradeName: "高二",
      className: "高二1班",
      building: "教学楼A",
      floor: "3层",
      roomLabel: null,
      subjects: [Subject.Chinese, Subject.Math, Subject.Physics],
      updatedAt: "2026-03-24T10:00:00Z",
    };
  },
  async create() {
    return { id: 2 };
  },
  async update() {
    return { success: true };
  },
  async remove() {
    return { success: true };
  },
  async listGradeOptions() {
    return ["高一", "高二"];
  },
};

describe("class config store", () => {
  it("loads default row on filter init", async () => {
    const store = createClassConfigStore(fakeService);
    await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
    expect(store.viewState.total).toBe(2);
    expect(store.viewState.editingId).toBe(1);
    expect(store.viewState.form.className).toBe("高二1班");
  });

  it("supports load and update flow", async () => {
    const store = createClassConfigStore(fakeService);
    await store.loadList();
    expect(store.viewState.total).toBe(2);

    await store.loadDetail(1);
    expect(store.viewState.form.className).toBe("高二1班");

    store.setFormField("building", "教学楼B");
    await store.update();
    expect(store.viewState.errorMessage).toBe("");
  });

  it("sets switch/create/rename intents from class name input", async () => {
    const store = createClassConfigStore(fakeService);
    await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });

    store.setFormField("className", "高二2班");
    expect(store.viewState.classNameIntent).toBe("switch");
    expect(store.viewState.targetMatchId).toBe(2);

    store.setFormField("className", "高二9班");
    expect(store.viewState.classNameIntent).toBe("create");

    store.startCreateFromClassName("高二9班");
    expect(store.viewState.classNameIntent).toBe("create");

    store.setFormField("className", "高二1班（重点）");
    expect(store.viewState.classNameIntent).toBe("switch");
  });

  it("marks form dirty and clears detail for create intent", async () => {
    const store = createClassConfigStore(fakeService);
    await store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
    expect(store.viewState.isDirty).toBe(false);

    store.setFormField("building", "教学楼B");
    expect(store.viewState.isDirty).toBe(true);

    store.startCreateFromClassName("高二7班");
    expect(store.viewState.editingId).toBeNull();
    expect(store.viewState.form.className).toBe("高二7班");
    expect(store.viewState.form.building).toBe("");
    expect(store.viewState.classNameIntent).toBe("create");
  });
});
