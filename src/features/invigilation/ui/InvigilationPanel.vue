<template>
  <section class="panel">
    <ConfigCard title="监考人数配置" description="统一设置每个考场所需监考老师个数，分配时自动按此规则执行。">
      <label class="display-field count-field" for="exam-room-required-count">
        <span class="field-label">每个考场监考老师个数</span>
        <div class="field-value-row">
          <input
            id="exam-room-required-count"
            class="value-input count-input"
            v-model.number="defaultExamRoomRequiredCount"
            type="number"
            min="1"
            @blur="saveConfig"
            @keyup.enter="saveConfig"
          />
          <strong class="field-value-text">人</strong>
        </div>
      </label>
    </ConfigCard>

    <ConfigCard class="exclude-card-shell" title="考试禁排设置" description="选择老师不参与某场考试的监考，系统在分配时自动跳过。">
      <div class="exclude-toolbar">
        <div class="fluent-combo" :class="{ open: showTeacherMenu }" @focusin="showTeacherMenu = true" @focusout="hideTeacherMenu">
          <input class="fluent-input select-field" v-model="teacherKeyword" placeholder="选择教师" />
          <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
          <div v-if="showTeacherMenu" class="fluent-menu">
            <button
              v-for="teacher in teacherOptions"
              :key="teacher.id"
              type="button"
              class="fluent-option"
              :class="{ selected: teacher.id === selectedTeacherId }"
              @mousedown.prevent="pickTeacher(teacher.id, teacher.teacherName)"
            >
              {{ teacher.teacherName }}
            </button>
            <div v-if="teacherOptions.length === 0" class="menu-empty">未找到匹配教师</div>
          </div>
        </div>

        <div class="fluent-combo" :class="{ open: showSessionMenu }" @focusin="showSessionMenu = true" @focusout="hideSessionMenu">
          <input class="fluent-input select-field" v-model="sessionKeyword" placeholder="选择考试场次" />
          <span class="material-symbols-rounded combo-icon">keyboard_arrow_down</span>
          <div v-if="showSessionMenu" class="fluent-menu">
            <button
              v-for="session in sessionOptions"
              :key="session.sessionId"
              type="button"
              class="fluent-option"
              :class="{ selected: session.sessionId === selectedSessionId }"
              @mousedown.prevent="pickSession(session.sessionId, session.label)"
            >
              {{ session.label }}
            </button>
            <div v-if="sessionOptions.length === 0" class="menu-empty">未找到匹配场次</div>
          </div>
        </div>
      </div>

      <div v-if="store.viewState.staffExclusions.length === 0" class="exclude-empty">
        <span>选择教师与考试场次后会自动加入禁排列表。</span>
      </div>

      <div v-else class="exclude-list">
        <div
          v-for="item in store.viewState.staffExclusions"
          :key="`${item.teacherId}-${item.sessionId}`"
          class="exclude-item"
        >
          <span class="exclude-text">{{ item.teacherName }}  -  不监考 {{ item.sessionLabel }}</span>
          <div class="exclude-right">
            <span class="danger-pill">已禁排</span>
            <button class="icon-btn" type="button" title="删除禁排" @click="removeExclusion(item.teacherId, item.sessionId)">
              <span class="material-symbols-rounded" aria-hidden="true">delete</span>
            </button>
          </div>
        </div>
      </div>
    </ConfigCard>

    <div class="split-row">
      <ConfigCard title="全员自习时间" description="设置无需监考时段，系统默认全体教师可安排为自习值守。">
        <label class="display-field self-study-field" for="self-study-start-time">
          <span class="field-label">时间范围</span>
          <div class="field-value-row">
            <input
              id="self-study-start-time"
              class="value-input time-input"
              type="text"
              inputmode="numeric"
              placeholder="12:10"
              v-model="selfStudyStartTime"
              @blur="saveConfig"
              @keyup.enter="saveConfig"
            />
            <span class="field-value-text">-</span>
            <input
              class="value-input time-input"
              type="text"
              inputmode="numeric"
              placeholder="13:40"
              v-model="selfStudyEndTime"
              @blur="saveConfig"
              @keyup.enter="saveConfig"
            />
          </div>
        </label>
      </ConfigCard>

      <ConfigCard title="监考津贴" description="分别设置场内与场外监考津贴单价，系统按分钟自动结算。">
        <div class="subsidy-row">
          <label class="display-field subsidy-field">
            <span class="field-label">场内监考津贴</span>
            <div class="field-value-row">
              <input
                class="value-input subsidy-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="indoorAllowancePerMinute"
                @blur="saveConfig"
                @keyup.enter="saveConfig"
              />
              <strong class="field-value-text">元 / 分钟</strong>
            </div>
          </label>

          <label class="display-field subsidy-field">
            <span class="field-label">场外监考津贴</span>
            <div class="field-value-row">
              <input
                class="value-input subsidy-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="outdoorAllowancePerMinute"
                @blur="saveConfig"
                @keyup.enter="saveConfig"
              />
              <strong class="field-value-text">元 / 分钟</strong>
            </div>
          </label>
        </div>
      </ConfigCard>
    </div>

    <ConfigCard>
      <div class="action-row">
        <p class="action-text">确认规则后即可分配监考老师，并在完成后导出监考表。</p>
        <div class="action-buttons">
          <button class="primary-btn action-btn" :disabled="store.viewState.assigning" @click="assignTeachers">
            {{ store.viewState.assigning ? "分配中..." : "分配监考老师" }}
          </button>
          <button class="secondary-btn action-btn" :disabled="!store.viewState.staffOverview.generatedAt">导出监考表</button>
        </div>
      </div>
    </ConfigCard>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
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
  void maybeAddExclusion();
}

function pickSession(sessionId: number, label: string) {
  selectedSessionId.value = sessionId;
  sessionKeyword.value = label;
  showSessionMenu.value = false;
  void maybeAddExclusion();
}

async function maybeAddExclusion() {
  if (!selectedTeacherId.value || !selectedSessionId.value) {
    return;
  }
  const added = await store.addStaffExclusion(selectedTeacherId.value, selectedSessionId.value);
  if (!added) {
    return;
  }
  teacherKeyword.value = "";
  sessionKeyword.value = "";
  selectedTeacherId.value = null;
  selectedSessionId.value = null;
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
  position: relative;
  isolation: isolate;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.display-field {
  min-height: 74px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border: 1px solid var(--color-border-soft);
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.6);
  padding: 12px 14px;
}

.count-field {
  width: 320px;
}

.self-study-field {
  width: 220px;
}

.field-label {
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 600;
}

.field-value-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.field-value-text {
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
  white-space: nowrap;
}

.value-input {
  min-width: 0;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
  line-height: 1.2;
}

.value-input::-webkit-outer-spin-button,
.value-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.value-input[type="number"] {
  -moz-appearance: textfield;
  appearance: textfield;
}

.value-input:focus {
  outline: none;
}

.count-input {
  width: 18px;
}

.time-input {
  width: 56px;
}

.subsidy-input {
  width: 32px;
}

.exclude-card-shell {
  position: relative;
  z-index: 6;
}

.exclude-toolbar {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.fluent-combo {
  width: 220px;
  position: relative;
}

.fluent-combo.open {
  z-index: 20;
}

.fluent-input {
  width: 100%;
  min-height: 42px;
  border: 1px solid #d8e4f2;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.92);
  padding: 0 12px;
  font-size: 14px;
  color: var(--color-text);
  transition:
    border-color 0.18s ease,
    box-shadow 0.18s ease,
    background-color 0.18s ease;
}

.fluent-combo.open .fluent-input,
.fluent-input:focus {
  outline: none;
  border-color: #b9d6ff;
  box-shadow: 0 0 0 3px rgba(185, 214, 255, 0.32);
  background: #ffffff;
}

.select-field {
  padding-right: 32px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  padding: 6px;
  border: 1px solid #d8e4f2;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.98);
  box-shadow: 0 18px 40px rgba(35, 52, 78, 0.16);
  backdrop-filter: blur(18px);
  z-index: 24;
}

.fluent-option {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  min-height: 38px;
  padding: 8px 12px;
  border-radius: 10px;
  font-size: 13px;
  color: #334155;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background-color 0.18s ease, color 0.18s ease;
}

.fluent-option:hover {
  background: #eef5ff;
  color: #0f6cbd;
}

.fluent-option.selected {
  background: #eaf3ff;
  color: #0f6cbd;
}

.menu-empty {
  padding: 10px 12px;
  color: var(--color-text-muted);
  font-size: 13px;
}

.exclude-empty {
  min-height: 42px;
  display: flex;
  align-items: center;
  border: 1px dashed #dce8f8;
  border-radius: 14px;
  background: rgba(247, 251, 255, 0.7);
  padding: 10px 12px;
  color: var(--color-text-muted);
  font-size: 13px;
}

.exclude-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.exclude-item {
  min-height: 44px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border: 1px solid #dce8f8;
  border-radius: 14px;
  background: #f7fbff;
  padding: 10px 12px;
}

.exclude-text {
  min-width: 0;
  color: var(--color-text);
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

.split-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.subsidy-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.action-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
}

.action-text {
  width: 460px;
  margin: 0;
  color: var(--color-text);
  font-size: 14px;
  line-height: 1.4;
}

.action-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.action-btn {
  min-width: 148px;
}

.material-symbols-rounded {
  font-family: "Material Symbols Rounded";
}

@media (max-width: 1100px) {
  .split-row,
  .subsidy-row {
    grid-template-columns: 1fr;
  }

  .action-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .action-text {
    width: auto;
  }
}

@media (max-width: 720px) {
  .count-field,
  .self-study-field,
  .fluent-combo {
    width: 100%;
  }

  .exclude-item {
    flex-direction: column;
    align-items: flex-start;
  }

  .exclude-right {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
