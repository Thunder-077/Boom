import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { expect, it, vi } from "vitest";
import { createExamAllocationStore } from "../../dashboard/store";
import MonitorConfigPanel from "../ui/MonitorConfigPanel";

vi.mock("../../../shared/utils/desktopRuntime", () => ({ hasDesktopRuntime: () => true }));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("../../classes/service", () => ({ classConfigService: { list: async () => ({ items: [
  { id: 1, className: "高三1班", gradeName: "高三" },
  { id: 10, className: "高三10班", gradeName: "高三" },
  { id: 2, className: "高三2班", gradeName: "高三" },
  { id: 66, className: "综评1班", gradeName: "高三" },
  { id: 67, className: "综评2班", gradeName: "高三" },
] }) } }));
vi.mock("../../dashboard/service", () => ({ listPopulatedTeachingClassIds: async () => [1, 2, 10, 66, 67], examAllocationService: { listSessions: async () => ({ items: [{ gradeName: "高三" }] }) } }));
let pageStore: ReturnType<typeof import("../../dashboard/store").useReactExamAllocationStore>;
vi.mock("../../dashboard/store", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../dashboard/store")>();
  return { ...actual, useReactExamAllocationStore: () => pageStore };
});

it("按综评筛选并为综评班设置自习科目", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const store = createExamAllocationStore();
  pageStore = { ...store, state: store.viewState, loadAll: vi.fn(async () => []) };
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  try {
    await act(async () => root.render(createElement(MonitorConfigPanel)));
    const open = [...container.querySelectorAll("button")].find((button) => button.textContent?.includes("配置班级科目"))!;
    await act(async () => open.click());
    const initialRows = [...container.querySelectorAll(".class-table-row")].map((row) => row.textContent);
    expect(initialRows).toEqual([expect.stringContaining("高三1班"), expect.stringContaining("高三2班"), expect.stringContaining("高三10班"), expect.stringContaining("综评1班")]);
    await act(async () => container.querySelector(".grade-filter-select .fluent-trigger")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })));
    const option = [...document.querySelectorAll<HTMLButtonElement>(".teleported-fluent-menu .fluent-option")].find((item) => item.textContent?.includes("综评班"))!;
    await act(async () => option.click());
    const rows = [...container.querySelectorAll(".class-table-row")];
    expect(rows.map((row) => row.textContent)).toEqual([expect.stringContaining("综评1班"), expect.stringContaining("综评2班")]);
    await act(async () => rows[0].querySelector<HTMLButtonElement>(".subject-badge")!.click());
    const subject = [...container.querySelectorAll<HTMLButtonElement>(".subject-menu-item")].find((item) => item.textContent?.includes("数学"))!;
    await act(async () => subject.click());
    expect(rows[0].textContent).toContain("数学");
  } finally {
    act(() => root.unmount());
    container.remove();
  }
});
