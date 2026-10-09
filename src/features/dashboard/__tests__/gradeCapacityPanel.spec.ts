import { act, createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createExamAllocationStore, useReactExamAllocationStore } from "../store";
import ExamAssignmentPanel from "../ui/ExamAssignmentPanel";

let pageStore: ReturnType<typeof useReactExamAllocationStore>;
vi.mock("../store", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../store")>();
  return { ...actual, useReactExamAllocationStore: () => pageStore };
});

let root: Root;
let container: HTMLDivElement;
afterEach(() => {
  act(() => root?.unmount());
  container?.remove();
  vi.useRealTimers();
});

describe("grade capacity panel", () => {
  it("inherits global values, saves independent edits, and restores inheritance", async () => {
    vi.useFakeTimers();
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const store = createExamAllocationStore();
    pageStore = {
      ...store,
      state: { ...store.viewState, sessionTimeGradeOptions: ["高一", "高二"] },
      saveSettings: vi.fn(async () => {}),
      saveSessionTimes: vi.fn(async () => {}),
    };
    container = document.createElement("div");
    document.body.append(container);
    root = createRoot(container);
    await act(async () => root.render(createElement(ExamAssignmentPanel)));
    const input = (label: string) => container.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
    const row = input("高一默认人数").closest(".grade-capacity-row")!;
    expect(input("高一默认人数").disabled).toBe(true);
    expect(input("高一默认人数").value).toBe("40");
    await act(async () => (row.querySelector("button") as HTMLButtonElement).click());
    expect(input("高一默认人数").disabled).toBe(false);
    await act(async () => {
      for (const label of ["高一默认人数", "高一最大人数"]) {
        const element = input(label);
        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(element, "30");
        element.dispatchEvent(new Event("input", { bubbles: true }));
      }
    });
    await act(async () => vi.advanceTimersByTimeAsync(800));
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "", [], [
      { gradeName: "高一", defaultCapacity: 30, maxCapacity: 30 },
    ]);
    expect(input("高二默认人数").value).toBe("40");
    await act(async () => (row.querySelector("button") as HTMLButtonElement).click());
    await act(async () => vi.advanceTimersByTimeAsync(800));
    expect(input("高一默认人数").disabled).toBe(true);
    expect(input("高一默认人数").value).toBe("40");
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "", [], []);

    await act(async () => (row.querySelector("button") as HTMLButtonElement).click());
    await act(async () => {
      const element = input("高一默认人数");
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(element, "201");
      element.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const savedCount = vi.mocked(pageStore.saveSettings).mock.calls.length;
    await act(async () => vi.advanceTimersByTimeAsync(800));
    expect(pageStore.saveSettings).toHaveBeenCalledTimes(savedCount);
    expect(row.textContent).toContain("人数须为整数");
    expect(container.querySelector('[role="status"]')?.textContent).toContain("高一");
    await act(async () => (row.querySelector("button") as HTMLButtonElement).click());
    await act(async () => vi.advanceTimersByTimeAsync(800));
    expect(pageStore.saveSettings).toHaveBeenCalledTimes(savedCount + 1);
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "", [], []);
  });
});
