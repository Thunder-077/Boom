import { describe, expect, it, vi } from "vitest";
import { Subject } from "../../../entities/score/model";
import { createExamAllocationStore } from "../store";
import type { ExamAllocationService } from "../service";

const fakeService: ExamAllocationService = {
  async getSettings() {
    return {
      defaultCapacity: 40,
      maxCapacity: 41,
      examTitle: "测试考试",
      examNotices: ["考试前 10 分钟入场"],
      updatedAt: "2026-03-24T10:00:00Z",
    };
  },
  async updateSettings() {
    return { success: true };
  },
  async startGenerate() {
    return { success: true };
  },
  async getGenerationProgress() {
    return {
      status: "idle",
      stage: "idle",
      stageLabel: "等待开始",
      percent: 0,
      message: "等待开始分配考场",
      currentGrade: null,
      totalGrades: 0,
      completedGrades: 0,
      updatedAt: "2026-03-24T10:00:00Z",
    };
  },
  async getOverview() {
    return {
      generatedAt: "2026-03-24T10:00:00Z",
      defaultCapacity: 40,
      maxCapacity: 41,
      gradeCount: 1,
      sessionCount: 1,
      examRoomCount: 3,
      selfStudyRoomCount: 1,
      studentAllocationCount: 120,
      warningCount: 0,
    };
  },
  async listSessions() {
    return {
      total: 1,
      items: [
        {
          id: 1,
          gradeName: "高一",
          subject: Subject.English,
          isForeignGroup: true,
          foreignOrder: 1,
          participantCount: 120,
          examRoomCount: 3,
          selfStudyRoomCount: 1,
        },
      ],
    };
  },
  async getSessionDetail() {
    return {
      session: {
        id: 1,
        gradeName: "高一",
        subject: Subject.English,
        isForeignGroup: true,
        foreignOrder: 1,
        participantCount: 120,
        examRoomCount: 3,
        selfStudyRoomCount: 1,
      },
      spaces: [],
      studentAllocations: [],
      staffAssignments: [],
    };
  },
  async listSessionTimes() {
    return [
      {
        sessionId: 1,
        gradeName: "高一",
        subject: Subject.English,
        startAt: "2026-03-24T08:00",
        endAt: "2026-03-24T10:00",
        sourceGradeName: "高一",
        isInherited: false,
      },
    ];
  },
  async listSessionTimeGradeOptions() {
    return ["高一", "高二"];
  },
  async upsertSessionTimes() {
    return { success: true };
  },
  async deleteSessionTime() {
    return { success: true };
  },
  async getPersistedInvigilationState() {
    return {
      config: {
        defaultExamRoomRequiredCount: 1,
        indoorAllowancePerMinute: 0.5,
        outdoorAllowancePerMinute: 0.3,
        middleManagerDefaultEnabled: false,
        middleManagerExceptionTeacherIds: [],
        selfStudyDate: "2026-03-24",
        selfStudyStartTime: "12:10",
        selfStudyEndTime: "13:40",
      },
      customRules: [],
      selfStudyClassSubjects: [],
    };
  },
  async listInvigilationCustomRuleOptions() {
    return {
      examSessionOptions: [
        {
          id: 1,
          label: "高一外语 03-24 08:00-10:00",
          startAt: "2026-03-24T08:00",
          endAt: "2026-03-24T10:00",
        },
      ],
      fullSelfStudyOption: {
        label: "全员自习 03-24 12:10-13:40",
        startAt: "2026-03-24T12:10",
        endAt: "2026-03-24T13:40",
      },
      targetOptions: [
        {
          id: "space:1",
          label: "高一1场",
          subtitle: "高一外语 03-24 08:00-10:00",
          timeScopeType: "exam_session",
          timeScopeId: 1,
          taskScopeType: "exam_room",
        },
      ],
    };
  },
  async savePersistedInvigilationConfig() {
    return { success: true };
  },
  async replacePersistedInvigilationCustomRules() {
    return { success: true };
  },
  async savePersistedSelfStudyClassSubjects() {
    return { success: true };
  },
  async generateStaffPlan(_payload) {
    return {
      generatedAt: "2026-03-24T10:00:00Z",
      taskCount: 10,
      assignedCount: 9,
      unassignedCount: 1,
      imbalanceMinutes: 80,
      warningCount: 1,
      solverEngine: "cp_sat" as const,
      optimalityStatus: "feasible" as const,
      solveDurationMs: 1234,
      fallbackReason: null,
      fallbackPoolAssignments: 0,
      unassignedDetails: [],
    };
  },
  async getStaffPlanOverview() {
    return {
      generatedAt: "2026-03-24T10:00:00Z",
      sessionCount: 1,
      taskCount: 10,
      assignedCount: 9,
      unassignedCount: 1,
      warningCount: 1,
      imbalanceMinutes: 80,
      solverEngine: "cp_sat" as const,
      optimalityStatus: "feasible" as const,
      solveDurationMs: 1234,
      fallbackReason: null,
      fallbackPoolAssignments: 0,
    };
  },
  async listStaffTasks() {
    return {
      total: 0,
      items: [],
    };
  },
  async listTeacherDutyStats() {
    return {
      total: 0,
      items: [],
    };
  },
  async listInvigilationExclusionSessionOptions() {
    return [];
  },
  async listTeachers() {
    return {
      total: 1,
      items: [
        {
          id: 101,
          teacherName: "张老师",
          subjects: [Subject.English],
          classNames: ["高一1班"],
          remark: "",
          isMiddleManager: false,
        },
      ],
    };
  },
  async exportLatestExamAllocationBundle() {
    return {
      folderPath: "D:/exports/测试考试考场安排",
      gradeCount: 1,
      fileCount: 8,
      exportedAt: "2026-03-24T10:00:00Z",
    };
  },
  async exportLatestInvigilationSchedule() {
    return {
      filePath: "D:/exports/监考表-20260324-100000.xlsx",
      exportedAt: "2026-03-24T10:00:00Z",
    };
  },
};

describe("exam allocation store", () => {
  it("loads overview and sessions", async () => {
    const store = createExamAllocationStore(fakeService);
    await store.loadAll();
    expect(store.viewState.overview.gradeCount).toBe(1);
    expect(store.viewState.sessions.length).toBe(1);
    expect(store.viewState.detail?.session.subject).toBe(Subject.English);
  });

  it("passes custom rules through when assigning teachers", async () => {
    const generateStaffPlan = vi.fn(fakeService.generateStaffPlan);
    const store = createExamAllocationStore({
      ...fakeService,
      generateStaffPlan,
    });
    await store.loadAll();

    await store.saveCustomRules([
      {
        actionType: "exclude",
        teacherId: 101,
        teacherName: "老师",
        timeScopeType: "exam_session",
        timeScopeIds: [1],
        timeScopeLabels: ["高一外语 03-24 08:00-10:00"],
        taskScopeType: "exam_room",
        targetScopeType: "selected_targets",
        targetIds: ["space:1"],
        targetLabels: ["高一1场"],
      },
    ]);

    await store.assignTeachers();

    expect(generateStaffPlan).toHaveBeenCalledWith(
      expect.objectContaining({
        customRules: [
          expect.objectContaining({
            teacherId: 101,
            timeScopeIds: [1],
            taskScopeType: "exam_room",
            targetIds: ["space:1"],
          }),
        ],
      }),
    );
  });
});
