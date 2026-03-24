import { describe, expect, it } from "vitest";
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
  async generate() {
    return { generatedAt: "2026-03-24T10:00:00Z", gradeCount: 1, sessionCount: 1, warningCount: 0 };
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
  async listSpaceStaffRequirements() {
    return [];
  },
  async upsertSpaceStaffRequirements() {
    return { success: true };
  },
  async generateStaffPlan() {
    return {
      generatedAt: "2026-03-24T10:00:00Z",
      taskCount: 10,
      assignedCount: 9,
      unassignedCount: 1,
      imbalanceMinutes: 80,
      warningCount: 1,
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
  async exportLatestExamAllocationBundle() {
    return {
      zipPath: "D:/exports/exam_export_20260324_140000.zip",
      batchDir: "D:/exports/exam_export_20260324_140000",
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
});
