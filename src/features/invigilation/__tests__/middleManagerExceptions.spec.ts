import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { createExamAllocationStore } from "../../dashboard/store";
import MonitorConfigPanel from "../ui/MonitorConfigPanel";

vi.mock("../../../shared/utils/desktopRuntime", () => ({ hasDesktopRuntime: () => true }));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("../../classes/service", () => ({ classConfigService: { list: async () => ({ items: [] }) } }));
vi.mock("../../dashboard/service", () => ({
  listPopulatedTeachingClassIds: async () => [],
  examAllocationService: { listSessions: async () => ({ items: [] }) },
}));

let pageStore: ReturnType<typeof import("../../dashboard/store").useReactExamAllocationStore>;
vi.mock("../../dashboard/store", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../dashboard/store")>();
  return { ...actual, useReactExamAllocationStore: () => pageStore };
});

describe("中层监考例外展示", () => {
  it("切换默认规则时保持例外人员的参与身份不变", async () => {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const store = createExamAllocationStore();
    pageStore = {
      ...store,
      state: {
        ...store.viewState,
        teachers: [
          { id: 1, teacherName: "例外甲", isMiddleManager: true, invigilationGradeName: "高一", subjects: [], classNames: [], remark: null },
          { id: 2, teacherName: "普通乙", isMiddleManager: true, invigilationGradeName: "高二", subjects: [], classNames: [], remark: null },
        ],
        invigilationConfig: {
          ...store.viewState.invigilationConfig,
          middleManagerDefaultEnabled: false,
          middleManagerExceptionTeacherIds: [1],
        },
      },
      loadAll: vi.fn(async () => []),
    };
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    try {
      await act(async () => root.render(createElement(MonitorConfigPanel)));
      const configure = [...container.querySelectorAll("button")]
        .find((button) => button.textContent?.includes("配置例外"))!;
      await act(async () => configure.click());
      const statusLabels = () => [...container.querySelectorAll(".middle-status-pill")]
        .map((item) => item.textContent);
      expect(statusLabels()).toEqual(["设为参与", "不参与"]);
      const defaultEnabled = container.querySelector<HTMLButtonElement>(".middle-manager-drawer .soft-panel .segment-btn")!;
      await act(async () => defaultEnabled.click());
      expect(statusLabels()).toEqual(["设为参与", "不参与"]);
    } finally {
      act(() => root.unmount());
      container.remove();
    }
  });
});
