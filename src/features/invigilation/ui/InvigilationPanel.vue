<template>
  <section class="panel">
    <ConfigCard title="监考人数配置" description="统一设置每个考场所需监考老师个数，分配时自动按此规则执行。">
      <div class="rule-row">
        <label class="metric-field narrow">
          <span class="metric-label">每个考场监考老师个数</span>
          <input
            class="metric-input"
            v-model.number="defaultExamRoomRequiredCount"
            type="number"
            min="1"
            @blur="saveConfig"
            @keyup.enter="saveConfig"
          />
        </label>
        <div class="rule-meta">当前规则将应用到考场分配、监考抽签与津贴结算。</div>
      </div>
    </ConfigCard>

    <ConfigCard title="考试禁排设置" description="选择老师不参与某场考试的监考，系统在分配时自动跳过。">
      <div class="exclude-toolbar">
        <label>
          <input
            class="glass-field auto-field"
            list="teacher-options"
            v-model="teacherKeyword"
            placeholder="选择教师"
          />
          <datalist id="teacher-options">
            <option v-for="item in teacherOptions" :key="item.id" :value="item.teacherName" />
          </datalist>
        </label>
        <label>
          <input
            class="glass-field auto-field"
            list="session-options"
            v-model="sessionKeyword"
            placeholder="选择考试场次"
          />
          <datalist id="session-options">
            <option v-for="item in sessionOptions" :key="item.sessionId" :value="item.label" />
          </datalist>
        </label>
        <button class="secondary-btn add-btn" type="button" @click="addExclusion">添加禁排</button>
      </div>
      <div class="exclude-list">
        <div v-for="item in store.viewState.staffExclusions" :key="item.id" class="exclude-item">
          <span>{{ item.teacherName }} - 不监考{{ item.sessionLabel }}</span>
          <div class="exclude-right">
            <span class="danger-pill">已禁排</span>
            <button class="icon-btn" type="button" @click="removeExclusion(item.id)">
              <span class="material-symbols-rounded" aria-hidden="true">delete</span>
            </button>
          </div>
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
          <label class="metric-field">
            <span class="metric-label">场内监考津贴</span>
            <input
              class="metric-input"
              type="number"
              min="0"
              step="0.1"
              v-model.number="indoorAllowancePerMinute"
              @blur="saveConfig"
              @keyup.enter="saveConfig"
            />
            <span class="metric-value">元 / 分钟</span>
          </label>
          <label class="metric-field">
            <span class="metric-label">场外监考津贴</span>
            <input
              class="metric-input"
              type="number"
              min="0"
              step="0.1"
              v-model.number="outdoorAllowancePerMinute"
              @blur="saveConfig"
              @keyup.enter="saveConfig"
            />
            <span class="metric-value">元 / 分钟</span>
          </label>
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
const selfStudySubject = ref<Subject | "">("");
const defaultExamRoomRequiredCount = ref(1);
const indoorAllowancePerMinute = ref(0.5);
const outdoorAllowancePerMinute = ref(0.3);
const teacherKeyword = ref("");
const sessionKeyword = ref("");

const teacherOptions = computed(() =>
  store.viewState.teachers.filter((item) => {
    const keyword = teacherKeyword.value.trim();
    if (!keyword) {
      return true;
    }
    return item.teacherName.includes(keyword);
  }),
);

const sessionOptions = computed(() => {
  const rows = store.viewState.sessionTimes.map((item) => {
    const start = item.startAt ?? "";
    const end = item.endAt ?? "";
    const date = start.length >= 10 ? start.slice(5, 10) : "--";
    const startTime = start.length >= 16 ? start.slice(11, 16) : "--:--";
    const endTime = end.length >= 16 ? end.slice(11, 16) : "--:--";
    return {
      sessionId: item.sessionId,
      label: `${item.gradeName} ${SUBJECT_LABELS[item.subject]} ${date} ${startTime}-${endTime}`,
    };
  });
  const keyword = sessionKeyword.value.trim();
  if (!keyword) {
    return rows;
  }
  return rows.filter((item) => item.label.includes(keyword));
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
  () => store.viewState.invigilationConfig,
  (config) => {
    defaultExamRoomRequiredCount.value = config.defaultExamRoomRequiredCount;
    indoorAllowancePerMinute.value = Number(config.indoorAllowancePerMinute || 0);
    outdoorAllowancePerMinute.value = Number(config.outdoorAllowancePerMinute || 0);
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

async function saveConfig() {
  await store.saveInvigilationConfig({
    defaultExamRoomRequiredCount: Math.max(1, Math.floor(defaultExamRoomRequiredCount.value || 1)),
    indoorAllowancePerMinute: Math.max(0, Number(indoorAllowancePerMinute.value || 0)),
    outdoorAllowancePerMinute: Math.max(0, Number(outdoorAllowancePerMinute.value || 0)),
  });
}

async function addExclusion() {
  const teacher = store.viewState.teachers.find((item) => item.teacherName === teacherKeyword.value.trim());
  const session = sessionOptions.value.find((item) => item.label === sessionKeyword.value.trim());
  if (!teacher || !session) {
    return;
  }
  await store.addStaffExclusion(teacher.id, session.sessionId);
  teacherKeyword.value = "";
  sessionKeyword.value = "";
}

async function removeExclusion(id: number) {
  await store.removeStaffExclusion(id);
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
  align-items: center;
}

.auto-field {
  width: 220px;
  min-height: 42px;
  border-radius: 14px;
}

.add-btn {
  height: 42px;
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

.exclude-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon-btn {
  border: 0;
  background: transparent;
  color: #c26868;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.icon-btn .material-symbols-rounded {
  font-family: "Material Symbols Rounded";
  font-size: 18px;
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

.subsidy-row > div,
.subsidy-row > label {
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
