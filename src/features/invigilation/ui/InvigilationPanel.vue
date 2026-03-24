<template>
  <section class="panel">
    <ConfigCard title="监考人数配置" description="统一设置每个考场所需监考老师个数，分配时自动按此规则执行。">
      <div class="rule-row">
        <label class="metric-field narrow">
          <span class="metric-label">每个考场监考老师个数</span>
          <input class="metric-input" v-model.number="examRoomRequiredCount" type="number" min="1" @blur="saveRequirements" />
        </label>
        <div class="rule-meta">当前规则将应用到考场分配、监考抽签与津贴结算。</div>
      </div>
    </ConfigCard>

    <ConfigCard title="考试禁排设置" description="选择老师不参与某场考试的监考，系统在分配时自动跳过。">
      <div class="exclude-toolbar">
        <label>
          <select class="glass-field" disabled>
            <option>选择教师</option>
          </select>
        </label>
        <label>
          <select class="glass-field" disabled>
            <option>选择考试场次</option>
          </select>
        </label>
      </div>
      <div class="exclude-list">
        <div v-for="task in previewItems" :key="task.id" class="exclude-item">
          <span>{{ formatPreview(task) }}</span>
          <span class="danger-pill">已禁排</span>
        </div>
      </div>
    </ConfigCard>

    <div class="time-row">
      <ConfigCard title="全员自习时间" description="设置无需监考时段，系统默认全体教师可安排为自习值守。">
        <div class="self-study-grid">
          <label class="metric-field short">
            <span class="metric-label">科目</span>
            <select v-model="selfStudySubject" class="glass-field slim-field">
              <option value="" disabled>请选择科目</option>
              <option v-for="item in selfStudySubjectOptions" :key="item.value" :value="item.value">{{ item.label }}</option>
            </select>
          </label>
          <div class="metric-field short">
            <span class="metric-label">时间范围</span>
            <span class="metric-value">{{ selfStudyTimeRange }}</span>
          </div>
        </div>
      </ConfigCard>

      <ConfigCard title="监考津贴" description="分别设置场内与场外监考津贴单价，系统按分钟自动结算。">
        <div class="subsidy-row">
          <div class="metric-field">
            <span class="metric-label">场内监考津贴</span>
            <span class="metric-value">0.5 元 / 分钟</span>
          </div>
          <div class="metric-field">
            <span class="metric-label">场外监考津贴</span>
            <span class="metric-value">0.3 元 / 分钟</span>
          </div>
        </div>
      </ConfigCard>
    </div>

    <ConfigCard>
      <div class="action-row">
        <p>确认规则后即可分配监考老师，并在完成后导出监考表。</p>
        <div class="action-buttons">
          <button class="primary-btn" :disabled="store.viewState.assigning" @click="assignTeachers">
            {{ store.viewState.assigning ? "分配中..." : "分配监考老师" }}
          </button>
          <button class="secondary-btn" :disabled="!store.viewState.staffOverview.generatedAt">导出监考表</button>
        </div>
      </div>
    </ConfigCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { SUBJECT_LABELS } from "../../../entities/class-config/model";
import type { Subject } from "../../../entities/score/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useExamAllocationStore } from "../../dashboard/store";

const store = useExamAllocationStore();
const examRoomRequiredCount = ref(1);
const selfStudySubject = ref<Subject | "">("");

const previewItems = computed(() => {
  const unassigned = store.viewState.staffTasks.filter((task) => task.status === "unassigned").slice(0, 2);
  if (unassigned.length > 0) {
    return unassigned;
  }
  return store.viewState.staffTasks.slice(0, 2);
});

const selfStudySubjectOptions = computed(() => {
  const keys = Array.from(new Set(store.viewState.sessionTimes.map((item) => item.subject)));
  return keys.map((value) => ({ value, label: SUBJECT_LABELS[value] }));
});

const selfStudyTimeRange = computed(() => {
  if (!selfStudySubject.value) {
    return "--";
  }
  const rows = store.viewState.sessionTimes.filter((item) => item.subject === selfStudySubject.value);
  if (rows.length === 0) {
    return "--";
  }
  const starts = rows.map((item) => store.viewState.sessionTimeDrafts[item.sessionId]?.startAt ?? item.startAt ?? "").filter(Boolean);
  const ends = rows.map((item) => store.viewState.sessionTimeDrafts[item.sessionId]?.endAt ?? item.endAt ?? "").filter(Boolean);
  if (starts.length === 0 || ends.length === 0) {
    return "--";
  }
  const start = starts.sort()[0];
  const end = ends.sort()[ends.length - 1];
  return `${start.slice(5, 10)} ${start.slice(11, 16)} - ${end.slice(11, 16)}`;
});

watch(
  () => store.viewState.requirements,
  (requirements) => {
    const firstExamRoom = requirements.find((item) => item.role === "exam_room_invigilator");
    if (firstExamRoom) {
      examRoomRequiredCount.value = firstExamRoom.requiredCount;
    }
  },
  { immediate: true },
);

watch(
  selfStudySubjectOptions,
  (options) => {
    if (options.length === 0) {
      selfStudySubject.value = "";
      return;
    }
    if (!options.some((item) => item.value === selfStudySubject.value)) {
      selfStudySubject.value = options[0].value;
    }
  },
  { immediate: true },
);

function formatPreview(task: (typeof store.viewState.staffTasks)[number]) {
  const subject = SUBJECT_LABELS[task.subject];
  const suffix = task.reason ?? `${task.startAt.slice(5, 10)} ${task.startAt.slice(11, 16)}`;
  return `${task.teacherName ?? task.spaceName}  -  不监考${subject}（${suffix}）`;
}

async function saveRequirements() {
  for (const item of store.viewState.requirements) {
    if (item.role === "exam_room_invigilator" && item.spaceId) {
      store.setRequirementCount(item.spaceId, "exam_room_invigilator", examRoomRequiredCount.value);
    }
  }
  await store.saveRequirements();
}

async function assignTeachers() {
  await store.assignTeachers();
}

onMounted(async () => {
  await store.loadAll();
});
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.rule-row {
  display: flex;
  gap: 16px;
}

.narrow {
  width: 320px;
}

.metric-input {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
}

.metric-input:focus {
  outline: none;
}

.rule-meta {
  flex: 1;
  min-height: 74px;
  border: 1px solid #dce8f8;
  border-radius: 16px;
  background: #f7fbff;
  padding: 12px 14px;
  font-size: 14px;
  line-height: 1.4;
}

.exclude-toolbar {
  display: flex;
  gap: 12px;
}

.exclude-toolbar select {
  width: 220px;
}

.exclude-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.exclude-item {
  min-height: 44px;
  border: 1px solid #dce8f8;
  border-radius: 14px;
  background: #f7fbff;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
}

.time-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.short {
  width: 220px;
}

.self-study-grid {
  display: flex;
  gap: 12px;
}

.slim-field {
  min-height: 34px;
}

.subsidy-row {
  display: flex;
  gap: 14px;
}

.subsidy-row > div {
  flex: 1;
}

.action-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
}

.action-row p {
  width: 460px;
  margin: 0;
  font-size: 14px;
  line-height: 1.4;
}

.action-buttons {
  display: flex;
  gap: 12px;
}
</style>
