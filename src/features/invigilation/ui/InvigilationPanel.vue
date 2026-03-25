<template>
  <section class="panel">
    <ConfigCard title="监考人数配置" description="统一设置每个考场所需监考老师个数，分配时自动按此规则执行。">
      <div class="rule-row">
        <label class="metric-field narrow">
          <span class="metric-label">每个考场监考老师个数</span>
          <input
            class="fluent-input"
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
        <div class="fluent-combo" @focusin="showTeacherMenu = true" @focusout="hideTeacherMenu">
          <input class="fluent-input select-field" v-model="teacherKeyword" placeholder="选择教师" />
          <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
          <div v-if="showTeacherMenu" class="fluent-menu">
            <button
              v-for="teacher in teacherOptions"
              :key="teacher.id"
              type="button"
              class="fluent-option"
              @mousedown.prevent="pickTeacher(teacher.id, teacher.teacherName)"
            >
              {{ teacher.teacherName }}
            </button>
          </div>
        </div>

        <div class="fluent-combo" @focusin="showSessionMenu = true" @focusout="hideSessionMenu">
          <input class="fluent-input select-field" v-model="sessionKeyword" placeholder="选择考试场次" />
          <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
          <div v-if="showSessionMenu" class="fluent-menu">
            <button
              v-for="session in sessionOptions"
              :key="session.sessionId"
              type="button"
              class="fluent-option"
              @mousedown.prevent="pickSession(session.sessionId, session.label)"
            >
              {{ session.label }}
            </button>
          </div>
        </div>
      </div>
      <div class="exclude-toolbar">
        <button class="secondary-btn add-btn" type="button" @click="addExclusion">添加禁排</button>
      </div>
      <div class="exclude-list">
        <div
          v-for="item in store.viewState.staffExclusions"
          :key="`${item.teacherId}-${item.sessionId}`"
          class="exclude-item"
        >
          <span>{{ item.teacherName }}  -  不监考{{ item.sessionLabel }}</span>
          <div class="exclude-right">
            <span class="danger-pill">已禁排</span>
            <button class="icon-btn" type="button" @click="removeExclusion(item.teacherId, item.sessionId)">
              <span class="material-symbols-rounded" aria-hidden="true">delete</span>
            </button>
          </div>
        </div>
      </div>
    </ConfigCard>

    <div class="time-row">
      <ConfigCard title="全员自习时间" description="独立配置全员自习时段，不与考试场次关联。">
        <div class="self-study-grid">
          <label class="metric-field short">
            <span class="metric-label">科目</span>
            <select class="fluent-input slim-field" v-model="selfStudySubject" @change="saveConfig">
              <option v-for="item in selfStudySubjectOptions" :key="item.value" :value="item.value">
                {{ item.label }}
              </option>
            </select>
          </label>
          <label class="metric-field short">
            <span class="metric-label">开始时间</span>
            <input class="fluent-input slim-field" type="time" v-model="selfStudyStartTime" @blur="saveConfig" />
          </label>
          <label class="metric-field short">
            <span class="metric-label">结束时间</span>
            <input class="fluent-input slim-field" type="time" v-model="selfStudyEndTime" @blur="saveConfig" />
          </label>
        </div>
      </ConfigCard>

      <ConfigCard title="监考津贴" description="分别设置场内与场外监考津贴单价，系统按分钟自动结算。">
        <div class="subsidy-row">
          <label class="metric-field">
            <span class="metric-label">场内监考津贴</span>
            <div class="inline-unit">
              <input
                class="fluent-input unit-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="indoorAllowancePerMinute"
                @blur="saveConfig"
                @keyup.enter="saveConfig"
              />
              <span class="metric-value">元 / 分钟</span>
            </div>
          </label>
          <label class="metric-field">
            <span class="metric-label">场外监考津贴</span>
            <div class="inline-unit">
              <input
                class="fluent-input unit-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="outdoorAllowancePerMinute"
                @blur="saveConfig"
                @keyup.enter="saveConfig"
              />
              <span class="metric-value">元 / 分钟</span>
            </div>
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
import { Subject } from "../../../entities/score/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import { useExamAllocationStore } from "../../dashboard/store";

const store = useExamAllocationStore();

const defaultExamRoomRequiredCount = ref(1);
const indoorAllowancePerMinute = ref(0.5);
const outdoorAllowancePerMinute = ref(0.3);
const selfStudySubject = ref<Subject>(Subject.Chinese);
const selfStudyStartTime = ref("12:10");
const selfStudyEndTime = ref("13:40");

const teacherKeyword = ref("");
const sessionKeyword = ref("");
const selectedTeacherId = ref<number | null>(null);
const selectedSessionId = ref<number | null>(null);
const showTeacherMenu = ref(false);
const showSessionMenu = ref(false);

const selfStudySubjectOptions = [
  Subject.Chinese,
  Subject.Math,
  Subject.English,
  Subject.Physics,
  Subject.Chemistry,
  Subject.Biology,
  Subject.Politics,
  Subject.History,
  Subject.Geography,
].map((value) => ({ value, label: SUBJECT_LABELS[value] }));

const teacherOptions = computed(() => {
  const keyword = teacherKeyword.value.trim();
  return store.viewState.teachers.filter((item) =>
    keyword ? item.teacherName.includes(keyword) : true,
  );
});

const sessionOptions = computed(() => {
  const keyword = sessionKeyword.value.trim();
  return store.viewState.exclusionSessionOptions.filter((item) =>
    keyword ? item.label.includes(keyword) : true,
  );
});

watch(
  () => store.viewState.invigilationConfig,
  (config) => {
    defaultExamRoomRequiredCount.value = config.defaultExamRoomRequiredCount;
    indoorAllowancePerMinute.value = Number(config.indoorAllowancePerMinute || 0);
    outdoorAllowancePerMinute.value = Number(config.outdoorAllowancePerMinute || 0);
    selfStudySubject.value = config.selfStudySubject;
    selfStudyStartTime.value = config.selfStudyStartTime;
    selfStudyEndTime.value = config.selfStudyEndTime;
  },
  { immediate: true },
);

function pickTeacher(id: number, name: string) {
  selectedTeacherId.value = id;
  teacherKeyword.value = name;
  showTeacherMenu.value = false;
}

function pickSession(sessionId: number, label: string) {
  selectedSessionId.value = sessionId;
  sessionKeyword.value = label;
  showSessionMenu.value = false;
}

function hideTeacherMenu() {
  setTimeout(() => {
    showTeacherMenu.value = false;
  }, 80);
}

function hideSessionMenu() {
  setTimeout(() => {
    showSessionMenu.value = false;
  }, 80);
}

async function saveConfig() {
  await store.saveInvigilationConfig({
    defaultExamRoomRequiredCount: Math.max(1, Math.floor(defaultExamRoomRequiredCount.value || 1)),
    indoorAllowancePerMinute: Math.max(0, Number(indoorAllowancePerMinute.value || 0)),
    outdoorAllowancePerMinute: Math.max(0, Number(outdoorAllowancePerMinute.value || 0)),
    selfStudySubject: selfStudySubject.value,
    selfStudyStartTime: selfStudyStartTime.value,
    selfStudyEndTime: selfStudyEndTime.value,
  });
}

async function addExclusion() {
  if (!selectedTeacherId.value || !selectedSessionId.value) {
    return;
  }
  await store.addStaffExclusion(selectedTeacherId.value, selectedSessionId.value);
  teacherKeyword.value = "";
  sessionKeyword.value = "";
  selectedTeacherId.value = null;
  selectedSessionId.value = null;
}

async function removeExclusion(teacherId: number, sessionId: number) {
  await store.removeStaffExclusion(teacherId, sessionId);
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

.fluent-input {
  width: 100%;
  min-height: 42px;
  border: 1px solid #d3dceb;
  border-radius: 14px;
  background: #ffffff;
  padding: 0 12px;
  font-size: 14px;
}

.fluent-input:focus {
  outline: none;
  border-color: #0f6cbd;
  box-shadow: 0 0 0 2px rgba(15, 108, 189, 0.18);
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

.fluent-combo {
  width: 220px;
  position: relative;
}

.select-field {
  padding-right: 32px;
}

.combo-icon {
  position: absolute;
  right: 10px;
  top: 11px;
  font-size: 18px;
  color: #667085;
  pointer-events: none;
}

.fluent-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid #d3dceb;
  border-radius: 10px;
  background: #ffffff;
  box-shadow: 0 10px 24px rgba(35, 52, 78, 0.18);
  z-index: 10;
}

.fluent-option {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  padding: 8px 12px;
  font-size: 13px;
  cursor: pointer;
}

.fluent-option:hover {
  background: #f2f7fd;
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
  width: 180px;
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

.subsidy-row > label {
  flex: 1;
}

.inline-unit {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.unit-input {
  width: 120px;
  min-height: 34px;
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
