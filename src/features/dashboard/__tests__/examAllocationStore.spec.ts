import { describe, expect, it } from "vitest";
import { vi } from "vitest";
import { Subject } from "../../../entities/score/model";
import { createExamAllocationStore } from "../store";
import type { ExamAllocationService } from "../service";

const fakeService: ExamAllocationService = {
  async getSettings() {
    return {
      defaultCapacity: 40,
      maxCapacity: 41,
      examTitle: "江河25年秋10月月考质量检测",
      examNotices: ["考试前20分钟入场"],
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
      },
    ];
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
      exclusions: [],
      selfStudyClassSubjects: [],
    };
  },
  async savePersistedInvigilationConfig() {
    return { success: true };
  },
  async replacePersistedInvigilationExclusions() {
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
    return [
      {
        sessionId: -3,
        gradeName: "全局",
        subject: Subject.English,
        startAt: "2026-03-24T08:00",
        endAt: "2026-03-24T10:00",
        label: "英语 03-24 08:00-10:00",
      },
    ];
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
      zipPath: "D:/exports/考场安排.zip",
      batchDir: "D:/exports/考场安排",
      gradeCount: 1,
      fileCount: 8,
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

  it("expands template exclusions to actual session ids when assigning teachers", async () => {
    const generateStaffPlan = vi.fn(fakeService.generateStaffPlan);
    const store = createExamAllocationStore({
      ...fakeService,
      generateStaffPlan,
    });
    await store.loadAll();

    const added = await store.addStaffExclusion(101, -3);
    expect(added).toBe(true);

    await store.assignTeachers();

    expect(generateStaffPlan).toHaveBeenCalledWith(
      expect.objectContaining({
        staffExclusions: [{ teacherId: 101, sessionId: 1 }],
      }),
    );
  });
});
