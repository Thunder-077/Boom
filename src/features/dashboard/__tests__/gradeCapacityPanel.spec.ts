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

async function renderPanel() {
  vi.useFakeTimers();
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const store = createExamAllocationStore();
  pageStore = {
    ...store,
    state: { ...store.viewState, sessionTimeGradeOptions: ["高一", "高二"] },
    saveSettings: vi.fn(async (defaultCapacity, maxCapacity, examTitle, examNotices, gradeCapacities) => {
      // 模拟真实保存后的设置回读，验证页面同步不会清除未保存输入。
      pageStore.state = { ...pageStore.state, settings: {
        ...pageStore.state.settings, defaultCapacity, maxCapacity, examTitle, examNotices,
        gradeCapacities: gradeCapacities ?? [],
      } };
      root.render(createElement(ExamAssignmentPanel));
    }),
    saveSessionTimes: vi.fn(async () => {}),
  };
  container = document.createElement("div");
  document.body.append(container);
  root = createRoot(container);
  await act(async () => root.render(createElement(ExamAssignmentPanel)));
}

function input(label: string) {
  return container.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
}

async function editCapacity(grade: string, normal: string, maximum: string) {
  await act(async () => {
    for (const [label, value] of [[`${grade}默认人数`, normal], [`${grade}最大人数`, maximum]]) {
      const element = input(label);
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(element, value);
      element.dispatchEvent(new Event("input", { bubbles: true }));
    }
  });
}

async function selectGrade(grade: string) {
  await act(async () => container.querySelector(".grade-capacity-selector .fluent-trigger")!
    .dispatchEvent(new MouseEvent("mousedown", { bubbles: true })));
  const option = [...document.querySelectorAll<HTMLButtonElement>(".teleported-fluent-menu .fluent-option")]
    .find((item) => item.textContent === grade)!;
  await act(async () => option.click());
}

async function saveCapacity() {
  await act(async () => container.querySelector<HTMLButtonElement>(".grade-capacity-heading button")!.click());
}

describe("grade capacity panel", () => {
  it("saves on click or grade switch, without timed saves or mode controls", async () => {
    await renderPanel();
    expect(container.querySelectorAll(".grade-capacity-row")).toHaveLength(1);
    expect(input("高一默认人数").disabled).toBe(false);
    expect(input("高一默认人数").value).toBe("40");
    expect(container.textContent).not.toMatch(/使用全局默认|单独配置|恢复默认|全局默认：/);
    await editCapacity("高一", "30", "30");
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(pageStore.saveSettings).not.toHaveBeenCalled();
    await saveCapacity();
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "", [], [
      { gradeName: "高一", defaultCapacity: 30, maxCapacity: 30 },
    ]);
    await selectGrade("高二");
    expect(input("高一默认人数")).toBeNull();
    expect(input("高二默认人数").value).toBe("40");
    await editCapacity("高二", "35", "36");
    await selectGrade("高一");
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "", [], [
      { gradeName: "高一", defaultCapacity: 30, maxCapacity: 30 },
      { gradeName: "高二", defaultCapacity: 35, maxCapacity: 36 },
    ]);
    expect(input("高一默认人数").value).toBe("30");
  });

  it("keeps invalid or failed drafts in the current grade, allowing correction and retry", async () => {
    await renderPanel();
    await editCapacity("高一", "201", "30");
    await selectGrade("高二");
    expect(pageStore.saveSettings).not.toHaveBeenCalled();
    expect(input("高一默认人数").value).toBe("201");
    expect(container.querySelector('[role="status"]')?.textContent).toContain("人数须为整数");
    await editCapacity("高一", "30", "30");
    vi.mocked(pageStore.saveSettings).mockRejectedValueOnce(new Error("保存失败"));
    await selectGrade("高二");
    expect(input("高一默认人数").value).toBe("30");
    expect(input("高二默认人数")).toBeNull();
    expect(container.querySelector('[role="status"]')?.textContent).toContain("保存失败");
    await selectGrade("高二");
    expect(input("高二默认人数").value).toBe("40");
    await selectGrade("高一");
    expect(input("高一默认人数").value).toBe("30");
  });

  it("keeps capacity drafts separate from title autosave and preserves them after settings reload", async () => {
    await renderPanel();
    await editCapacity("高一", "30", "30");
    await saveCapacity();
    await editCapacity("高一", "201", "30");
    await act(async () => {
      const title = container.querySelector<HTMLInputElement>('input[placeholder="2026 学年春季期末统一考试"]')!;
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(title, "测试考试");
      title.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "测试考试", [], [
      { gradeName: "高一", defaultCapacity: 30, maxCapacity: 30 },
    ]);
    expect(input("高一默认人数").value).toBe("201");
    await editCapacity("高一", "32", "33");
    await selectGrade("高二");
    expect(pageStore.saveSettings).toHaveBeenLastCalledWith(40, 41, "测试考试", [], [
      { gradeName: "高一", defaultCapacity: 32, maxCapacity: 33 },
    ]);
  });
});
