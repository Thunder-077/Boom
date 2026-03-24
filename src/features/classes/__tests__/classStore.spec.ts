import { describe, expect, it } from "vitest";
import { createClassConfigStore } from "../store";
import type { ClassConfigService } from "../service";
import { Subject } from "../../../entities/score/model";

const fakeService: ClassConfigService = {
  async list() {
    return {
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
  it("supports load and update flow", async () => {
    const store = createClassConfigStore(fakeService);
    await store.loadList();
    expect(store.viewState.total).toBe(1);

    await store.loadDetail(1);
    expect(store.viewState.form.className).toBe("高二1班");

    store.setFormField("building", "教学楼B");
    await store.update();
    expect(store.viewState.errorMessage).toBe("");
  });
});
