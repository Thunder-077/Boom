import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { Subject } from "../../../entities/score/model";
import { createExamAllocationStore } from "../../dashboard/store";
import MonitorConfigPanel from "../ui/MonitorConfigPanel";

vi.mock("../../../shared/utils/desktopRuntime", () => ({ hasDesktopRuntime: () => true }));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("../../classes/service", () => ({ classConfigService: { list: async () => ({ items: [
  { id: 1, className: "高一1班", gradeName: "高一" },
  { id: 2, className: "高一2班", gradeName: "高一" },
  { id: 3, className: "高一9班", gradeName: "高一" },
] }) } }));
vi.mock("../../dashboard/service", () => ({ listPopulatedTeachingClassIds: async () => [1, 2], examAllocationService: { listSessions: async () => ({ items: [{ gradeName: "高一" }] }) } }));

let pageStore: ReturnType<typeof import("../../dashboard/store").useReactExamAllocationStore>;
vi.mock("../../dashboard/store", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../dashboard/store")>();
  return { ...actual, useReactExamAllocationStore: () => pageStore };
});

describe("全员自习首次加载", () => {
  it("loads saved subjects even when they arrive before the class list", async () => {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const store = createExamAllocationStore();
    const subjects = [{ classId: 1, subject: Subject.Math }, { classId: 2, subject: Subject.Chinese }];
    pageStore = { ...store, state: { ...store.viewState, selfStudyClassSubjects: subjects },
      loadAll: vi.fn(async () => subjects) };
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    try {
      await act(async () => { root.render(createElement(MonitorConfigPanel)); });
      expect(container.textContent).toContain("所有班级已完成科目配置");
      expect(container.textContent).not.toContain("待补充：高一1班");
      expect(container.textContent).not.toContain("高一9班");
      expect(container.querySelector(".study-data-grid")?.textContent).toContain("2 个");
    } finally {
      act(() => root.unmount());
      container.remove();
    }
  });

  it("offers a rover switch and saves the selected state", async () => {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const store = createExamAllocationStore();
    const save = vi.fn(async () => {});
    pageStore = { ...store, state: store.viewState, loadAll: vi.fn(async () => []), saveInvigilationConfig: save };
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    try {
      await act(async () => { root.render(createElement(MonitorConfigPanel)); });
      const toggle = container.querySelector<HTMLInputElement>('input[role="switch"]')!;
      expect(toggle.checked).toBe(true);
      await act(async () => { toggle.click(); });
      expect(save).toHaveBeenCalledWith({ floorRoverEnabled: false });
    } finally {
      act(() => root.unmount());
      container.remove();
    }
  });
});
