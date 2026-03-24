import { describe, expect, it } from "vitest";
import { createTeacherStore } from "../store";
import type { TeacherService } from "../service";
import type { TeacherSubject } from "../../../entities/teacher/model";

const fakeService: TeacherService = {
  async list(params) {
    const rows = [
      {
        id: 1,
        teacherName: "王青",
        subjects: ["chinese"] as TeacherSubject[],
        classNames: ["高一1班"],
        remark: null,
      },
      {
        id: 2,
        teacherName: "李航",
        subjects: ["math", "information"] as TeacherSubject[],
        classNames: ["高二3班"],
        remark: "组长",
      },
    ];

    const items = rows.filter((item) => {
      if (params.nameKeyword && !item.teacherName.includes(params.nameKeyword)) {
        return false;
      }
      return true;
    });

    return {
      items,
      total: items.length,
    };
  },
  async getSummary() {
    return {
      importedAt: "2026-03-24T10:00:00Z",
      teacherCount: 2,
    };
  },
  async importExcel() {
    return {
      importedAt: "2026-03-24T10:00:00Z",
      rowCount: 2,
      durationMs: 20,
    };
  },
};

describe("teacher store", () => {
  it("loads data and applies filters", async () => {
    const store = createTeacherStore(fakeService);
    await store.load();
    expect(store.viewState.total).toBe(2);

    await store.setFilters({ nameKeyword: "王" });
    expect(store.viewState.total).toBe(1);
    expect(store.viewState.rows[0]?.teacherName).toBe("王青");

    await store.resetFilters();
    expect(store.viewState.total).toBe(2);
  });
});
