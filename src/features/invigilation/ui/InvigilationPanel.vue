<template>
  <section class="panel">
    <div class="grid-two top-grid">
      <ConfigCard class="top-card exam-count-card" title="监考人数配置" description="统一设置每个考场所需监考老师个数，分配时自动按此规则执行。">
        <div class="card-stack">
          <label class="display-field count-field" for="exam-room-required-count">
            <span class="field-label">每个考场监考老师人数</span>
            <div class="field-value-row">
              <input
                id="exam-room-required-count"
                class="value-input count-input"
                v-model.number="defaultExamRoomRequiredCount"
                type="number"
                min="1"
                @blur="handleSaveConfig"
                @keyup.enter="handleSaveConfig"
              />
              <strong class="field-value-text">人</strong>
            </div>
          </label>
        </div>
      </ConfigCard>

      <ConfigCard class="top-card middle-manager-card" title="中层是否监考" description="设置中层干部默认是否参与监考，并保留个别人例外入口。">
        <div class="card-stack">
          <div class="segment-wrap">
            <button class="segment-btn" :class="{ active: middleManagerDefaultEnabled }" type="button" @click="setMiddleManagerDefaultEnabled(true)">参与监考</button>
            <button class="segment-btn" :class="{ active: !middleManagerDefaultEnabled }" type="button" @click="setMiddleManagerDefaultEnabled(false)">不参与监考</button>
          </div>
          <div class="footer-row middle-footer">
            <span class="warning-pill">已设置 {{ middleManagerExceptionCount }} 位例外人员</span>
            <button class="secondary-btn drawer-trigger" type="button" @click="openMiddleManagerDrawer">配置例外</button>
          </div>
        </div>
      </ConfigCard>
    </div>

    <ConfigCard title="考试禁排设置" description="选择老师不参与某场考试的监考，系统在分配时自动跳过。">
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

      <div v-if="store.viewState.staffExclusions.length === 0" class="empty-box">
        选择教师与考试场次后会自动加入禁排列表。
      </div>
      <div v-else class="exclude-list">
        <div v-for="item in store.viewState.staffExclusions" :key="`${item.teacherId}-${item.sessionId}`" class="exclude-item">
          <span class="exclude-text">{{ item.teacherName }} - 不监考 {{ item.sessionLabel }}</span>
          <div class="exclude-right">
            <span class="danger-pill">已禁排</span>
            <button class="icon-btn" type="button" @click="removeExclusion(item.teacherId, item.sessionId)">
              <span class="material-symbols-rounded">delete</span>
            </button>
          </div>
        </div>
      </div>
    </ConfigCard>

    <div class="grid-two summary-grid-row">
      <ConfigCard class="summary-card self-study-card" title="全员自习" description="统一设置自习时段与班级科目摘要。">
        <div class="card-stack">
          <div class="summary-grid">
            <div class="summary-chip">
              <span class="field-label">时间范围</span>
              <strong class="summary-value" v-html="selfStudyScheduleText"></strong>
            </div>
            <div class="summary-chip">
              <span class="field-label">已配置班级</span>
              <strong class="summary-value">{{ configuredClassCount }} 个</strong>
            </div>
            <div class="summary-chip warning">
              <span class="field-label warning-text">待补充</span>
              <strong class="summary-value warning-text">{{ pendingClassCount }} 个班级</strong>
            </div>
          </div>
          <div class="footer-row self-study-footer">
            <p class="card-note self-study-note">{{ selfStudySummaryText }}</p>
            <button class="primary-btn drawer-trigger" type="button" @click="openSelfStudyDrawer">配置班级科目</button>
          </div>
        </div>
      </ConfigCard>

      <ConfigCard class="summary-card allowance-card" title="监考津贴" description="分别设置场内与场外监考津贴单价，系统按分钟自动结算。">
        <div class="card-stack">
          <div class="subsidy-row">
            <label class="display-field">
              <span class="field-label">场内监考津贴</span>
              <div class="field-value-row">
                <input class="value-input subsidy-input" type="number" min="0" step="0.1" v-model.number="indoorAllowancePerMinute" @blur="handleSaveConfig" @keyup.enter="handleSaveConfig" />
                <strong class="field-value-text">元 / 分钟</strong>
              </div>
            </label>
            <label class="display-field">
              <span class="field-label">场外监考津贴</span>
              <div class="field-value-row">
                <input class="value-input subsidy-input" type="number" min="0" step="0.1" v-model.number="outdoorAllowancePerMinute" @blur="handleSaveConfig" @keyup.enter="handleSaveConfig" />
                <strong class="field-value-text">元 / 分钟</strong>
              </div>
            </label>
          </div>
        </div>
      </ConfigCard>
    </div>

    <ConfigCard>
      <div class="action-row">
        <div class="action-copy">
          <p class="action-text">点击即可分配监考老师，并在完成后导出监考表。</p>
          <p v-if="store.viewState.staffOverview.generatedAt" class="solver-summary">{{ staffSolverSummary }}</p>
        </div>
        <div
          v-if="assignmentNotice || isAssignmentProgressVisible"
          ref="assignmentNoticeEl"
          class="assignment-notice inline"
          role="status"
          aria-live="polite"
          tabindex="-1"
        >
          <span class="material-symbols-rounded assignment-notice-icon">
            {{ assignmentNoticeIcon }}
          </span>
          <div class="assignment-notice-body">
            <span class="assignment-notice-text">{{ assignmentNoticeText }}</span>
            <div v-if="isAssignmentProgressVisible && assignmentProgress" class="assignment-progress">
              <div class="assignment-progress-meta">
                <span>{{ assignmentProgress.stageLabel }}</span>
                <span>{{ assignmentProgress.percent }}%</span>
              </div>
              <div class="assignment-progress-track" aria-hidden="true">
                <div class="assignment-progress-bar" :style="{ width: `${assignmentProgress.percent}%` }" />
              </div>
            </div>
          </div>
        </div>
        <div class="action-buttons">
          <button class="primary-btn action-btn" type="button" :disabled="store.viewState.assigning" @click="assignTeachers">{{ store.viewState.assigning ? "分配中..." : "分配监考老师" }}</button>
          <button class="secondary-btn action-btn" type="button" :disabled="!store.viewState.staffOverview.generatedAt">导出监考表</button>
        </div>
      </div>
    </ConfigCard>

    <transition name="drawer-fade">
      <div v-if="activeDrawer !== null" class="drawer-backdrop" @click="closeActiveDrawer" />
    </transition>

    <transition name="drawer-slide">
      <aside v-if="selfStudyDrawerOpen" class="config-drawer self-study-drawer">
        <div class="drawer-header">
          <div class="drawer-title-block">
            <h3>配置全员自习</h3>

          </div>
          <button class="drawer-close" type="button" @click="closeSelfStudyDrawer"><span class="material-symbols-rounded">close</span></button>
        </div>

        <section class="drawer-section soft-panel">
          <div class="section-header"><h4>统一时段</h4></div>
          <div class="schedule-row">
            <label class="display-field compact-field">
              <span class="field-label">自习日期</span>
              <input class="value-input framed-input date-input" type="text" inputmode="numeric" placeholder="03-26" v-model="selfStudyMonthDay" />
            </label>
            <label class="display-field compact-field">
              <span class="field-label">开始时间</span>
              <input class="value-input framed-input time-input" type="text" inputmode="numeric" maxlength="5" placeholder="12:10" v-model="selfStudyStartTime" />
            </label>
            <label class="display-field compact-field">
              <span class="field-label">结束时间</span>
              <input class="value-input framed-input time-input" type="text" inputmode="numeric" maxlength="5" placeholder="13:40" v-model="selfStudyEndTime" />
            </label>
          </div>
          <div class="footer-row">
            <span class="field-label">{{ selfStudyScopeText }}</span>
            <span class="info-pill">全体教师默认转为自习值守</span>
          </div>
          <div v-if="selfStudyValidationError" class="empty-box error-box">{{ selfStudyValidationError }}</div>
        </section>

        <section class="drawer-section class-config-section">
          <div class="section-header">
            <div>
              <h4>班级科目配置</h4>
            </div>
            <span class="warning-pill">{{ pendingClassCount }} 个待处理</span>
          </div>

          <div v-if="selfStudyLoadError" class="empty-box error-box">{{ selfStudyLoadError }}</div>
          <div v-else-if="selfStudyLoading" class="empty-box">正在加载教学班列表...</div>
          <div v-else-if="filteredClasses.length === 0" class="empty-box">暂无教学班数据，请先在班级配置中维护教学班。</div>

          <div v-if="!selfStudyLoading && filteredClasses.length > 0 && selectedClassCount > 0" class="selection-strip">已选 {{ selectedClassCount }} 个班级</div>

          <div v-if="!selfStudyLoading && filteredClasses.length > 0" class="toolbar-row">
            <div class="toolbar-left">
              <button class="toolbar-btn primary" type="button" :disabled="selectedClassCount === 0" @click="toggleBulkMenu">为选中班级设科目</button>
              <div class="toolbar-filter">
                <FluentSelect
                  v-model="gradeFilter"
                  :options="[{ label: '全部年级', value: 'all' }, ...availableGrades.map(g => ({ label: g, value: g }))]"
                  style="width: 140px;"
                />
              </div>
            </div>
            <div class="page-chip">第 {{ currentPage }} / {{ totalPages }} 页</div>
          </div>

          <div v-if="!selfStudyLoading && filteredClasses.length > 0" class="class-table">
            <div class="class-table-head">
              <label class="check-cell">
                <input type="checkbox" :checked="allCurrentPageSelected" :indeterminate.prop="indeterminateCurrentPageSelected" @change="toggleSelectAllCurrentPage" />
              </label>
              <span>班级</span>
              <span>年级</span>
              <span>科目</span>
              <span>状态</span>
            </div>
            <div v-for="row in pagedClasses" :key="row.id" class="class-table-row" :class="{ selected: selectedClassIds.has(row.id) }">
              <label class="check-cell">
                <input type="checkbox" :checked="selectedClassIds.has(row.id)" @change="toggleRowSelection(row.id)" />
              </label>
              <span class="cell-text strong">{{ row.className }}</span>
              <span class="cell-text muted">{{ row.gradeName }}</span>
              <button class="subject-badge" :class="{ empty: !row.subject }" type="button" @click="openSubjectMenu(row.id, $event)">
                {{ row.subject ? subjectLabelMap[row.subject] : "未选" }}
              </button>
              <span class="status-badge" :class="row.subject ? 'done' : 'pending'">{{ row.subject ? "已完成" : "待处理" }}</span>
            </div>
          </div>

          <div v-if="!selfStudyLoading && filteredClasses.length > 0" class="pagination-row">
            <span class="page-meta">共 {{ filteredClasses.length }} 个班级，本页 {{ pageStart }} - {{ pageEnd }}</span>
            <div class="pagination-actions">
              <button class="page-btn" type="button" :disabled="currentPage === 1" @click="goToPrevPage">上一页</button>
              <button v-for="page in visiblePages" :key="page" class="page-btn" :class="{ active: page === currentPage }" type="button" @click="goToPage(page)">{{ page }}</button>
              <button class="page-btn" type="button" :disabled="currentPage === totalPages" @click="goToNextPage">下一页</button>
            </div>
          </div>
        </section>

        <div class="drawer-footer">
          <p></p>
          <div class="drawer-actions">
            <button class="secondary-btn" type="button" @click="closeSelfStudyDrawer">取消</button>
            <button class="primary-btn" type="button" @click="saveSelfStudySetup">保存配置</button>
          </div>
        </div>
      </aside>
    </transition>

    <transition name="drawer-slide">
      <aside v-if="middleManagerDrawerOpen" class="config-drawer middle-manager-drawer">
        <div class="drawer-header">
          <div class="drawer-title-block">
            <h3>中层监考例外</h3>
            <p>例外名单用于覆盖默认规则，仅影响中层教师是否进入监考候选池。</p>
          </div>
          <button class="drawer-close" type="button" @click="closeMiddleManagerDrawer"><span class="material-symbols-rounded">close</span></button>
        </div>

        <section class="drawer-section soft-panel">
          <div class="section-header"><h4>默认规则</h4></div>
          <div class="segment-wrap">
            <button class="segment-btn" :class="{ active: middleManagerDefaultEnabledDraft }" type="button" @click="middleManagerDefaultEnabledDraft = true">参与监考</button>
            <button class="segment-btn" :class="{ active: !middleManagerDefaultEnabledDraft }" type="button" @click="middleManagerDefaultEnabledDraft = false">不参与监考</button>
          </div>
          <p class="drawer-note">{{ middleManagerDefaultEnabledDraft ? "当前默认策略：中层干部参与监考。例外名单中的人员将覆盖默认规则。" : "当前默认策略：中层干部不参与监考。例外名单中的人员将覆盖默认规则。" }}</p>
        </section>

        <section class="drawer-section">
          <div class="section-header">
            <div class="title-stack">
              <h4>例外名单</h4>
              <p>按人设置与默认规则相反的监考状态。</p>
            </div>
            <span class="warning-pill">{{ middleManagerExceptionTeacherIdsDraft.length }} 位例外</span>
          </div>

          <div class="middle-toolbar">
            <button class="primary-btn middle-primary-btn" type="button" @click="showMiddleManagerPicker = !showMiddleManagerPicker">
              {{ showMiddleManagerPicker ? "收起添加面板" : "添加例外人员" }}
            </button>
            <button class="middle-filter-btn" type="button" :class="{ active: showOnlyMiddleManagerExceptions }" @click="showOnlyMiddleManagerExceptions = !showOnlyMiddleManagerExceptions">
              仅看例外
            </button>
          </div>

          <div v-if="showMiddleManagerPicker" class="middle-picker">
            <label class="search-bar middle-search">
              <span class="material-symbols-rounded search-icon">search</span>
              <input v-model="middleManagerKeyword" type="text" placeholder="输入姓名搜索中层教师" />
            </label>
          </div>

          <div v-if="pagedMiddleManagerTeachers.length > 0" class="exclude-list">
            <div v-for="teacher in pagedMiddleManagerTeachers" :key="teacher.id" class="exclude-item middle-exception-item">
              <div class="middle-person">
                <strong>{{ teacher.teacherName }}</strong>
                <span class="middle-subtext">
                  {{
                    isMiddleManagerException(teacher.id)
                      ? middleManagerDefaultEnabledDraft
                        ? "已设为例外，当前不参与监考"
                        : "已设为例外，当前参与监考"
                      : middleManagerDefaultEnabledDraft
                        ? "跟随默认规则，当前参与监考"
                        : "跟随默认规则，当前不参与监考"
                  }}
                </span>
              </div>
              <div class="middle-actions">
                <span class="middle-status-pill" :class="getMiddleManagerStatusClass(teacher.id)">
                  {{ getMiddleManagerStatusLabel(teacher.id) }}
                </span>
                <button class="text-btn" type="button" @click="toggleMiddleManagerExceptionTeacher(teacher.id)">
                  {{ isMiddleManagerException(teacher.id) ? "取消例外" : "设为例外" }}
                </button>
              </div>
            </div>
            <div v-if="middleManagerTotalPages > 1" class="pagination-row middle-pagination">
              <span class="page-meta">共 {{ filteredMiddleManagerTeachers.length }} 位{{ showOnlyMiddleManagerExceptions ? "例外" : "中层" }}，本页 {{ middleManagerPageStart }} - {{ middleManagerPageEnd }}</span>
              <div class="pagination-actions">
                <button class="page-btn" type="button" :disabled="middleManagerPage === 1" @click="goToPrevMiddleManagerPage">上一页</button>
                <button v-for="page in middleManagerVisiblePages" :key="page" class="page-btn" :class="{ active: page === middleManagerPage }" type="button" @click="goToMiddleManagerPage(page)">{{ page }}</button>
                <button class="page-btn" type="button" :disabled="middleManagerPage === middleManagerTotalPages" @click="goToNextMiddleManagerPage">下一页</button>
              </div>
            </div>
          </div>
          <div v-else class="empty-box">{{ showOnlyMiddleManagerExceptions ? "当前还没有例外人员。" : "没有匹配的中层教师。" }}</div>
        </section>

        <div class="drawer-footer">
          <p>保存后将更新中层监考规则摘要与例外人数。</p>
          <div class="drawer-actions">
            <button class="secondary-btn" type="button" @click="closeMiddleManagerDrawer">取消</button>
            <button class="primary-btn" type="button" @click="saveMiddleManagerSetup">保存例外</button>
          </div>
        </div>
      </aside>
    </transition>

    <div v-if="subjectMenu.open" class="subject-menu" :style="{ top: `${subjectMenu.top}px`, left: `${subjectMenu.left}px` }" @click.stop>
      <button v-for="subject in selectableSubjects" :key="subject" class="subject-menu-item" :class="{ active: subjectMenuSelectedSubject === subject }" type="button" @click="applySubjectSelection(subject)">
        <span>{{ subjectLabelMap[subject] }}</span>
        <span v-if="subjectMenuSelectedSubject === subject" class="material-symbols-rounded">check</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ClassConfigRow } from "../../../entities/class-config/model";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ExamStaffAssignmentProgress, InvigilationConfig } from "../../../entities/exam-plan/model";
import type { Subject } from "../../../entities/score/model";
import { Subject as SubjectEnum } from "../../../entities/score/model";
import ConfigCard from "../../../widgets/common/ConfigCard.vue";
import FluentSelect from "../../../widgets/common/FluentSelect.vue";
import { classConfigService } from "../../classes/service";
import { useExamAllocationStore } from "../../dashboard/store";

interface SelfStudyClassRow {
  id: number;
  className: string;
  gradeName: string;
  subject: Subject | null;
}

interface AssignmentNotice {
  type: "success" | "error";
  text: string;
}

const gradeRankMap: Record<string, number> = { 高一: 1, 高二: 2, 高三: 3 };
const staffAssignmentProgressEvent = "invigilation_staff_assignment_progress";
const store = useExamAllocationStore();

const defaultExamRoomRequiredCount = ref(1);
const indoorAllowancePerMinute = ref(0.5);
const outdoorAllowancePerMinute = ref(0.3);
const selfStudyMonthDay = ref(new Date().toISOString().slice(5, 10));
const selfStudyStartTime = ref("12:10");
const selfStudyEndTime = ref("13:40");
const selfStudyValidationError = ref("");
const teacherKeyword = ref("");
const sessionKeyword = ref("");
const selectedTeacherId = ref<number | null>(null);
const selectedSessionId = ref<number | null>(null);
const showTeacherMenu = ref(false);
const showSessionMenu = ref(false);
const selfStudyDrawerOpen = ref(false);
const middleManagerDrawerOpen = ref(false);
const selfStudyLoading = ref(false);
const selfStudyLoadError = ref("");
const gradeFilter = ref("all");
const availableGrades = ref<string[]>([]);
const currentPage = ref(1);
const pageSize = 4;
const selectedClassIds = ref(new Set<number>());
const bulkMenuOpen = ref(false);
const middleManagerDefaultEnabledDraft = ref(false);
const middleManagerExceptionTeacherIdsDraft = ref<number[]>([]);
const middleManagerKeyword = ref("");
const middleManagerPage = ref(1);
const showMiddleManagerPicker = ref(false);
const showOnlyMiddleManagerExceptions = ref(false);
const subjectMenu = ref({ open: false, top: 0, left: 0, rowId: null as number | null, mode: "single" as "single" | "bulk" });
const selfStudyClasses = ref<SelfStudyClassRow[]>([]);
const middleManagerPageSize = 3;
const assignmentNotice = ref<AssignmentNotice | null>(null);
const assignmentNoticeEl = ref<HTMLElement | null>(null);
const assignmentProgress = ref<ExamStaffAssignmentProgress | null>(null);
let removeAssignmentProgressListener: UnlistenFn | null = null;

const subjectLabelMap: Record<Subject, string> = {
  [SubjectEnum.Chinese]: "语文",
  [SubjectEnum.Math]: "数学",
  [SubjectEnum.English]: "英语",
  [SubjectEnum.Physics]: "物理",
  [SubjectEnum.Chemistry]: "化学",
  [SubjectEnum.Biology]: "生物",
  [SubjectEnum.Politics]: "政治",
  [SubjectEnum.History]: "历史",
  [SubjectEnum.Geography]: "地理",
  [SubjectEnum.Russian]: "俄语",
  [SubjectEnum.Japanese]: "日语",
};

const selectableSubjects: Subject[] = [
  SubjectEnum.Chinese,
  SubjectEnum.Math,
  SubjectEnum.English,
  SubjectEnum.Russian,
  SubjectEnum.Japanese,
  SubjectEnum.History,
  SubjectEnum.Geography,
  SubjectEnum.Biology,
  SubjectEnum.Politics,
  SubjectEnum.Physics,
  SubjectEnum.Chemistry,
];

const activeDrawer = computed(() => (selfStudyDrawerOpen.value ? "selfStudy" : middleManagerDrawerOpen.value ? "middleManager" : null));
const middleManagerDefaultEnabled = computed(() => store.viewState.invigilationConfig.middleManagerDefaultEnabled);
const middleManagerExceptionCount = computed(() => store.viewState.invigilationConfig.middleManagerExceptionTeacherIds.length);
const teacherOptions = computed(() => store.viewState.teachers.filter((item) => (teacherKeyword.value.trim() ? item.teacherName.includes(teacherKeyword.value.trim()) : true)));
const sessionOptions = computed(() => store.viewState.exclusionSessionOptions.filter((item) => (sessionKeyword.value.trim() ? item.label.includes(sessionKeyword.value.trim()) : true)));
const middleManagerTeachers = computed(() => [...store.viewState.teachers].filter((item) => item.isMiddleManager).sort((a, b) => a.teacherName.localeCompare(b.teacherName, "zh-CN")));
const filteredClasses = computed(() => (gradeFilter.value === "all" ? selfStudyClasses.value : selfStudyClasses.value.filter((item) => item.gradeName === gradeFilter.value)));
const totalPages = computed(() => Math.max(1, Math.ceil(filteredClasses.value.length / pageSize)));
const pagedClasses = computed(() => filteredClasses.value.slice((currentPage.value - 1) * pageSize, currentPage.value * pageSize));
const pageStart = computed(() => (filteredClasses.value.length === 0 ? 0 : (currentPage.value - 1) * pageSize + 1));
const pageEnd = computed(() => Math.min(currentPage.value * pageSize, filteredClasses.value.length));
const visiblePages = computed(() => Array.from(new Set([1, Math.max(1, currentPage.value - 1), currentPage.value, Math.min(totalPages.value, currentPage.value + 1), totalPages.value])).filter((page) => page >= 1 && page <= totalPages.value));
const configuredClassCount = computed(() => selfStudyClasses.value.filter((item) => !!item.subject).length);
const pendingClassCount = computed(() => selfStudyClasses.value.length - configuredClassCount.value);
const allCurrentPageSelected = computed(() => pagedClasses.value.length > 0 && pagedClasses.value.every((item) => selectedClassIds.value.has(item.id)));
const indeterminateCurrentPageSelected = computed(() => {
  const count = pagedClasses.value.filter((item) => selectedClassIds.value.has(item.id)).length;
  return count > 0 && count < pagedClasses.value.length;
});
const selectedClassCount = computed(() => selectedClassIds.value.size);
const inferredSelfStudyYear = computed(() => {
  const firstSessionStart = store.viewState.sessionTimes.find((item) => item.startAt)?.startAt;
  if (firstSessionStart && /^\d{4}-\d{2}-\d{2}/.test(firstSessionStart)) {
    return firstSessionStart.slice(0, 4);
  }
  return String(new Date().getFullYear());
});
const normalizedSelfStudyDate = computed(() => {
  const value = selfStudyMonthDay.value.trim();
  if (!/^\d{2}-\d{2}$/.test(value)) return "";
  return `${inferredSelfStudyYear.value}-${value}`;
});
const selfStudyScheduleText = computed(() => {
  if (!selfStudyMonthDay.value || !selfStudyStartTime.value || !selfStudyEndTime.value) {
    return "未设置";
  }
  return `${selfStudyMonthDay.value}<br>${selfStudyStartTime.value} - ${selfStudyEndTime.value}`;
});
const selfStudyScopeText = computed(() => {
  const sessionCount = store.viewState.sessionTimes.length;
  if (sessionCount > 0) {
    return `适用范围：本次考试第 ${sessionCount} 场结束后`;
  }
  return "适用范围：全员自习开始与结束时间默认在同一天。";
});
const selfStudySummaryText = computed(() => {
  if (pendingClassCount.value === 0) return "所有班级已完成科目配置。";
  const pending = selfStudyClasses.value.filter((item) => !item.subject).map((item) => item.className);
  return `待补充：${pending.slice(0, 2).join("、")}${pending.length > 2 ? " 等" : ""}`;
});
const staffSolverSummary = computed(() => {
  const overview = store.viewState.staffOverview;
  if (!overview.generatedAt) return "";
  const statusLabel =
    overview.optimalityStatus === "optimal"
      ? "已证明最优"
      : overview.optimalityStatus === "feasible"
        ? "当前可行解"
        : overview.optimalityStatus === "infeasible"
          ? "模型不可行"
          : "求解失败";
  const fallbackSummary =
    overview.fallbackPoolAssignments > 0
      ? `，fallback_pool ${overview.fallbackPoolAssignments} 项`
      : "";
  return `CP-SAT，${statusLabel}，耗时 ${overview.solveDurationMs} ms${fallbackSummary}`;
});
const filteredMiddleManagerTeachers = computed(() => {
  const keyword = middleManagerKeyword.value.trim();
  return middleManagerTeachers.value.filter((item) => {
    const matchedKeyword = keyword ? item.teacherName.includes(keyword) : true;
    const matchedException = showOnlyMiddleManagerExceptions.value ? middleManagerExceptionTeacherIdsDraft.value.includes(item.id) : true;
    return matchedKeyword && matchedException;
  });
});
const middleManagerTotalPages = computed(() => Math.max(1, Math.ceil(filteredMiddleManagerTeachers.value.length / middleManagerPageSize)));
const pagedMiddleManagerTeachers = computed(() => filteredMiddleManagerTeachers.value.slice((middleManagerPage.value - 1) * middleManagerPageSize, middleManagerPage.value * middleManagerPageSize));
const middleManagerPageStart = computed(() => (filteredMiddleManagerTeachers.value.length === 0 ? 0 : (middleManagerPage.value - 1) * middleManagerPageSize + 1));
const middleManagerPageEnd = computed(() => Math.min(middleManagerPage.value * middleManagerPageSize, filteredMiddleManagerTeachers.value.length));
const middleManagerVisiblePages = computed(() =>
  Array.from(
    new Set([1, Math.max(1, middleManagerPage.value - 1), middleManagerPage.value, Math.min(middleManagerTotalPages.value, middleManagerPage.value + 1), middleManagerTotalPages.value]),
  ).filter((page) => page >= 1 && page <= middleManagerTotalPages.value),
);
const subjectMenuSelectedSubject = computed(() => (subjectMenu.value.open && subjectMenu.value.mode === "single" && subjectMenu.value.rowId !== null ? selfStudyClasses.value.find((item) => item.id === subjectMenu.value.rowId)?.subject ?? null : null));
const isAssignmentProgressVisible = computed(() => Boolean(store.viewState.assigning));
const assignmentNoticeIcon = computed(() => {
  if (isAssignmentProgressVisible.value) return "hourglass_top";
  return assignmentNotice.value?.type === "success" ? "check_circle" : "info";
});
const assignmentNoticeText = computed(() => {
  if (isAssignmentProgressVisible.value) {
    return assignmentProgress.value?.message || "正在准备监考分配...";
  }
  return assignmentNotice.value?.text || "";
});

watch(
  () => store.viewState.invigilationConfig,
  (config) => {
    defaultExamRoomRequiredCount.value = config.defaultExamRoomRequiredCount;
    indoorAllowancePerMinute.value = Number(config.indoorAllowancePerMinute || 0);
    outdoorAllowancePerMinute.value = Number(config.outdoorAllowancePerMinute || 0);
    selfStudyMonthDay.value = extractMonthDay(config.selfStudyDate);
    selfStudyStartTime.value = config.selfStudyStartTime;
    selfStudyEndTime.value = config.selfStudyEndTime;
    if (!middleManagerDrawerOpen.value) {
      middleManagerDefaultEnabledDraft.value = config.middleManagerDefaultEnabled;
      middleManagerExceptionTeacherIdsDraft.value = [...config.middleManagerExceptionTeacherIds];
    }
  },
  { immediate: true, deep: true },
);

watch(gradeFilter, () => {
  currentPage.value = 1;
});

watch(totalPages, (value) => {
  if (currentPage.value > value) currentPage.value = value;
});

watch([selfStudyMonthDay, selfStudyStartTime, selfStudyEndTime], () => {
  selfStudyValidationError.value = "";
});

watch(
  () => filteredMiddleManagerTeachers.value.length,
  (value) => {
    if (value === 0) {
      middleManagerPage.value = 1;
      return;
    }
    if (middleManagerPage.value > middleManagerTotalPages.value) {
      middleManagerPage.value = middleManagerTotalPages.value;
    }
  },
);

watch(showOnlyMiddleManagerExceptions, () => {
  middleManagerPage.value = 1;
});

watch(middleManagerKeyword, () => {
  middleManagerPage.value = 1;
});

function hideTeacherMenu() {
  window.setTimeout(() => {
    showTeacherMenu.value = false;
  }, 80);
}

function hideSessionMenu() {
  window.setTimeout(() => {
    showSessionMenu.value = false;
  }, 80);
}

function mapClassRowToSelfStudyRow(row: ClassConfigRow): SelfStudyClassRow {
  const persisted = store.viewState.selfStudyClassSubjects.find((item) => item.classId === row.id);
  return { id: row.id, className: row.className, gradeName: row.gradeName, subject: persisted?.subject ?? null };
}

function extractClassSortNumber(className: string) {
  const match = className.match(/(\d+)/g);
  return match && match.length > 0 ? Number(match[match.length - 1]) : Number.POSITIVE_INFINITY;
}

function compareTeachingClasses(a: SelfStudyClassRow, b: SelfStudyClassRow) {
  const gradeDiff = (gradeRankMap[a.gradeName] ?? 99) - (gradeRankMap[b.gradeName] ?? 99);
  if (gradeDiff !== 0) return gradeDiff;
  const classDiff = extractClassSortNumber(a.className) - extractClassSortNumber(b.className);
  if (classDiff !== 0) return classDiff;
  return a.className.localeCompare(b.className, "zh-CN", { numeric: true });
}

function extractMonthDay(dateText: string) {
  const value = (dateText || "").trim();
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return value.slice(5, 10);
  }
  if (/^\d{2}-\d{2}$/.test(value)) {
    return value;
  }
  return new Date().toISOString().slice(5, 10);
}

function resolvePersistedSelfStudyDate() {
  return normalizedSelfStudyDate.value || store.viewState.invigilationConfig.selfStudyDate || `${inferredSelfStudyYear.value}-${new Date().toISOString().slice(5, 10)}`;
}

function resetSelfStudyDraftState() {
  const config = store.viewState.invigilationConfig;
  selfStudyMonthDay.value = extractMonthDay(config.selfStudyDate);
  selfStudyStartTime.value = config.selfStudyStartTime;
  selfStudyEndTime.value = config.selfStudyEndTime;
  selfStudyValidationError.value = "";
  gradeFilter.value = "all";
  currentPage.value = 1;
  selectedClassIds.value = new Set();
  closeSubjectMenu();
  selfStudyClasses.value = selfStudyClasses.value.map((item) => {
    const persisted = store.viewState.selfStudyClassSubjects.find((subjectItem) => subjectItem.classId === item.id);
    return {
      ...item,
      subject: persisted?.subject ?? null,
    };
  });
}

async function loadSelfStudyClassData() {
  selfStudyLoading.value = true;
  selfStudyLoadError.value = "";
  try {
    const classResult = await classConfigService.list({ configType: "teaching_class", gradeName: "", keyword: "" });
    selfStudyClasses.value = classResult.items.map(mapClassRowToSelfStudyRow).sort(compareTeachingClasses);
    availableGrades.value = Array.from(new Set(classResult.items.map((item) => item.gradeName))).sort(
      (a, b) => (gradeRankMap[a] ?? 99) - (gradeRankMap[b] ?? 99) || a.localeCompare(b, "zh-CN", { numeric: true }),
    );
  } catch (error) {
    selfStudyLoadError.value = error instanceof Error ? error.message : String(error);
  } finally {
    selfStudyLoading.value = false;
  }
}

async function saveConfig(extra: Partial<InvigilationConfig> = {}) {
  await store.saveInvigilationConfig({
    defaultExamRoomRequiredCount: Math.max(1, Math.floor(defaultExamRoomRequiredCount.value || 1)),
    indoorAllowancePerMinute: Math.max(0, Number(indoorAllowancePerMinute.value || 0)),
    outdoorAllowancePerMinute: Math.max(0, Number(outdoorAllowancePerMinute.value || 0)),
    selfStudyDate: resolvePersistedSelfStudyDate(),
    selfStudyStartTime: selfStudyStartTime.value,
    selfStudyEndTime: selfStudyEndTime.value,
    ...extra,
  });
}

async function setMiddleManagerDefaultEnabled(value: boolean) {
  if (middleManagerDefaultEnabled.value === value) return;
  await saveConfig({ middleManagerDefaultEnabled: value });
}

function handleSaveConfig() {
  void saveConfig();
}

function openSelfStudyDrawer() {
  middleManagerDrawerOpen.value = false;
  resetSelfStudyDraftState();
  selfStudyDrawerOpen.value = true;
}

function closeSelfStudyDrawer() {
  selfStudyDrawerOpen.value = false;
  closeSubjectMenu();
}

function openMiddleManagerDrawer() {
  selfStudyDrawerOpen.value = false;
  closeSubjectMenu();
  middleManagerDefaultEnabledDraft.value = store.viewState.invigilationConfig.middleManagerDefaultEnabled;
  middleManagerExceptionTeacherIdsDraft.value = [...store.viewState.invigilationConfig.middleManagerExceptionTeacherIds];
  middleManagerKeyword.value = "";
  middleManagerPage.value = 1;
  showMiddleManagerPicker.value = false;
  showOnlyMiddleManagerExceptions.value = false;
  middleManagerDrawerOpen.value = true;
}

function closeMiddleManagerDrawer() {
  middleManagerDrawerOpen.value = false;
  middleManagerKeyword.value = "";
  middleManagerPage.value = 1;
  showMiddleManagerPicker.value = false;
  showOnlyMiddleManagerExceptions.value = false;
}

function closeActiveDrawer() {
  if (selfStudyDrawerOpen.value) closeSelfStudyDrawer();
  if (middleManagerDrawerOpen.value) closeMiddleManagerDrawer();
}

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
  if (!selectedTeacherId.value || !selectedSessionId.value) return;
  const added = await store.addStaffExclusion(selectedTeacherId.value, selectedSessionId.value);
  if (!added) return;
  teacherKeyword.value = "";
  sessionKeyword.value = "";
  selectedTeacherId.value = null;
  selectedSessionId.value = null;
}

function toggleRowSelection(id: number) {
  const next = new Set(selectedClassIds.value);
  next.has(id) ? next.delete(id) : next.add(id);
  selectedClassIds.value = next;
}

function toggleSelectAllCurrentPage() {
  const next = new Set(selectedClassIds.value);
  if (allCurrentPageSelected.value) {
    pagedClasses.value.forEach((item) => next.delete(item.id));
  } else {
    pagedClasses.value.forEach((item) => next.add(item.id));
  }
  selectedClassIds.value = next;
}

function goToPage(page: number) {
  currentPage.value = page;
  closeSubjectMenu();
}

function goToPrevPage() {
  if (currentPage.value > 1) goToPage(currentPage.value - 1);
}

function goToNextPage() {
  if (currentPage.value < totalPages.value) goToPage(currentPage.value + 1);
}

function openSubjectMenu(rowId: number, event: MouseEvent) {
  bulkMenuOpen.value = false;
  openSubjectMenuAtEvent(event, rowId, "single");
}

function toggleBulkMenu(event: MouseEvent) {
  if (selectedClassCount.value === 0) return;
  if (bulkMenuOpen.value) return closeSubjectMenu();
  bulkMenuOpen.value = true;
  openSubjectMenuAtEvent(event, null, "bulk");
}

function openSubjectMenuAtEvent(event: MouseEvent, rowId: number | null, mode: "single" | "bulk") {
  const target = event.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  const menuWidth = 168;
  const menuHeight = Math.min(5 * 42 + 16, window.innerHeight - 80);
  const padding = 12;
  let top = rect.bottom + 8;
  let left = rect.left;
  if (top + menuHeight > window.innerHeight - padding) top = Math.max(padding, rect.top - menuHeight - 8);
  if (left + menuWidth > window.innerWidth - padding) left = window.innerWidth - menuWidth - padding;
  if (left < padding) left = padding;
  subjectMenu.value = { open: true, top, left, rowId, mode };
}

function closeSubjectMenu() {
  subjectMenu.value = { open: false, top: 0, left: 0, rowId: null, mode: "single" };
  bulkMenuOpen.value = false;
}

function applySubjectSelection(subject: Subject) {
  if (subjectMenu.value.mode === "bulk") {
    selfStudyClasses.value = selfStudyClasses.value.map((item) => (selectedClassIds.value.has(item.id) ? { ...item, subject } : item));
    return closeSubjectMenu();
  }
  if (subjectMenu.value.rowId === null) return;
  const applyToSelected = selectedClassIds.value.size > 1 && selectedClassIds.value.has(subjectMenu.value.rowId);
  selfStudyClasses.value = selfStudyClasses.value.map((item) => {
    if (applyToSelected) return selectedClassIds.value.has(item.id) ? { ...item, subject } : item;
    return item.id === subjectMenu.value.rowId ? { ...item, subject } : item;
  });
  closeSubjectMenu();
}

async function saveSelfStudySetup() {
  const monthDay = selfStudyMonthDay.value.trim();
  const startTime = selfStudyStartTime.value.trim();
  const endTime = selfStudyEndTime.value.trim();
  if (!monthDay) {
    selfStudyValidationError.value = "请选择自习日期。";
    return;
  }
  if (!/^\d{2}-\d{2}$/.test(monthDay)) {
    selfStudyValidationError.value = "自习日期请按月-日填写，例如 03-26。";
    return;
  }
  if (!startTime) {
    selfStudyValidationError.value = "请填写开始时间。";
    return;
  }
  if (!endTime) {
    selfStudyValidationError.value = "请填写结束时间。";
    return;
  }
  if (!/^\d{2}:\d{2}$/.test(startTime) || !/^\d{2}:\d{2}$/.test(endTime)) {
    selfStudyValidationError.value = "开始时间和结束时间请按 HH:MM 填写，例如 12:10。";
    return;
  }
  const fullDate = normalizedSelfStudyDate.value;
  if (!fullDate) {
    selfStudyValidationError.value = "自习日期格式不正确。";
    return;
  }
  if (`${fullDate}T${endTime}` <= `${fullDate}T${startTime}`) {
    selfStudyValidationError.value = "结束时间必须晚于开始时间。";
    return;
  }
  selfStudyValidationError.value = "";
  await saveConfig();
  await store.saveSelfStudyClassSubjects(selfStudyClasses.value.map((item) => ({ classId: item.id, subject: item.subject })));
  closeSelfStudyDrawer();
}

function isMiddleManagerException(teacherId: number) {
  return middleManagerExceptionTeacherIdsDraft.value.includes(teacherId);
}

function toggleMiddleManagerExceptionTeacher(teacherId: number) {
  if (isMiddleManagerException(teacherId)) {
    middleManagerExceptionTeacherIdsDraft.value = middleManagerExceptionTeacherIdsDraft.value.filter((id) => id !== teacherId);
    return;
  }
  middleManagerExceptionTeacherIdsDraft.value = [...middleManagerExceptionTeacherIdsDraft.value, teacherId].sort((a, b) => a - b);
}

function getMiddleManagerStatusLabel(teacherId: number) {
  const isException = isMiddleManagerException(teacherId);
  const enabled = isException ? !middleManagerDefaultEnabledDraft.value : middleManagerDefaultEnabledDraft.value;
  return enabled ? "参与" : "不参与";
}

function getMiddleManagerStatusClass(teacherId: number) {
  return getMiddleManagerStatusLabel(teacherId) === "参与" ? "on" : "off";
}

function goToMiddleManagerPage(page: number) {
  middleManagerPage.value = page;
}

function goToPrevMiddleManagerPage() {
  if (middleManagerPage.value > 1) goToMiddleManagerPage(middleManagerPage.value - 1);
}

function goToNextMiddleManagerPage() {
  if (middleManagerPage.value < middleManagerTotalPages.value) goToMiddleManagerPage(middleManagerPage.value + 1);
}

async function saveMiddleManagerSetup() {
  await saveConfig({
    middleManagerDefaultEnabled: middleManagerDefaultEnabledDraft.value,
    middleManagerExceptionTeacherIds: middleManagerExceptionTeacherIdsDraft.value,
  });
  closeMiddleManagerDrawer();
}

async function removeExclusion(teacherId: number, sessionId: number) {
  await store.removeStaffExclusion(teacherId, sessionId);
}

async function showAssignmentNotice(type: AssignmentNotice["type"], text: string) {
  assignmentNotice.value = { type, text };
  await nextTick();
  assignmentNoticeEl.value?.scrollIntoView({
    behavior: "smooth",
    block: "nearest",
  });
}

async function assignTeachers() {
  assignmentNotice.value = null;
  assignmentProgress.value = {
    status: "running",
    stage: "preparing",
    stageLabel: "准备开始",
    percent: 0,
    message: "正在准备监考分配...",
    completedSteps: 0,
    totalSteps: 13,
    updatedAt: new Date().toISOString(),
  };
  await nextTick();
  assignmentNoticeEl.value?.scrollIntoView({
    behavior: "smooth",
    block: "nearest",
  });
  try {
    const result = await store.assignTeachers();
    assignmentProgress.value = {
      status: "completed",
      stage: "completed",
      stageLabel: "分配完成",
      percent: 100,
      message: "监考分配完成，正在刷新结果...",
      completedSteps: 13,
      totalSteps: 13,
      updatedAt: new Date().toISOString(),
    };
    const summary =
      result.optimalityStatus === "optimal"
        ? "CP-SAT 求解完成，已证明最优"
        : result.fallbackReason
          ? "CP-SAT 提前结束，已保留当前最好可行解"
          : "CP-SAT 求解完成，已生成可行解";
    const optimality =
      result.optimalityStatus === "optimal"
        ? "已证明最优"
        : result.optimalityStatus === "feasible"
          ? "当前可行解"
          : result.optimalityStatus === "infeasible"
            ? "模型不可行"
            : "求解失败";
    const fallbackPart =
      result.fallbackPoolAssignments > 0
        ? `，fallback_pool ${result.fallbackPoolAssignments} 项`
        : "";
    await showAssignmentNotice(
      "success",
      `${summary}：已分配 ${result.assignedCount} 项，未分配 ${result.unassignedCount} 项，${optimality}，耗时 ${result.solveDurationMs} ms${fallbackPart}。`,
    );
  } catch (error) {
    assignmentProgress.value = null;
    const message =
      store.viewState.errorMessage ||
      (error instanceof Error ? error.message : String(error)) ||
      "分配失败，请检查配置后重试。";
    await showAssignmentNotice("error", `分配失败：${message}`);
  }
}

function handleGlobalPointerDown(event: MouseEvent) {
  if (!subjectMenu.value.open) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest(".subject-menu") || target?.closest(".subject-badge") || target?.closest(".toolbar-btn.primary")) return;
  closeSubjectMenu();
}

onMounted(async () => {
  document.addEventListener("mousedown", handleGlobalPointerDown);
  removeAssignmentProgressListener = await listen<ExamStaffAssignmentProgress>(staffAssignmentProgressEvent, (event) => {
    assignmentProgress.value = event.payload;
  });
  await store.loadAll();
  await loadSelfStudyClassData();
  await nextTick();
});

onBeforeUnmount(() => {
  document.removeEventListener("mousedown", handleGlobalPointerDown);
  removeAssignmentProgressListener?.();
  removeAssignmentProgressListener = null;
});
</script>

<style scoped>
.panel {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 20px;
  isolation: isolate;
}

.grid-two {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
  align-items: start;
}

.top-grid {
  align-items: stretch;
}

.summary-grid-row {
  align-items: stretch;
}

.top-card {
  height: 100%;
}

.summary-card {
  height: 100%;
}

.card-stack {
  display: flex;
  flex-direction: column;
}

.top-card :deep(h3),
.summary-card :deep(h3) {
  line-height: 1.2;
}

.top-card :deep(p),
.summary-card :deep(p) {
  line-height: 1.35;
}

.exam-count-card :deep(.body),
.middle-manager-card :deep(.body),
.self-study-card :deep(.body),
.allowance-card :deep(.body) {
  height: 100%;
}

.exam-count-card .card-stack,
.middle-manager-card .card-stack,
.self-study-card .card-stack,
.allowance-card .card-stack {
  height: 100%;
}

.exam-count-card :deep(.config-card) {
  gap: 12px;
}

.exam-count-card :deep(.body) {
  gap: 12px;
}

.exam-count-card .card-stack {
  gap: 12px;
  justify-content: space-between;
}

.middle-manager-card :deep(.config-card) {
  gap: 12px;
}

.middle-manager-card :deep(.body) {
  gap: 12px;
}

.middle-manager-card .card-stack {
  gap: 12px;
  justify-content: space-between;
}

.self-study-card :deep(.config-card) {
  gap: 12px;
}

.self-study-card :deep(.body) {
  gap: 12px;
}

.self-study-card .card-stack {
  gap: 12px;
  justify-content: space-between;
}

.allowance-card :deep(.config-card) {
  gap: 12px;
}

.allowance-card :deep(.body) {
  gap: 12px;
}

.allowance-card .card-stack {
  gap: 12px;
  justify-content: space-between;
}

.display-field,
.summary-chip,
.summary-box {
  border: 1px solid #e9f0f8;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.6);
  padding: 12px 14px;
}

.display-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 82px;
}

.compact-field {
  min-height: 74px;
}

.date-field {
  min-height: 74px;
}

.count-field {
  width: 320px;
}

.field-label,
.summary-label,
.card-note,
.page-meta {
  color: var(--color-text-muted);
  font-size: 13px;
}
.warning-text {
  color: var(--color-warning);
}

.field-value-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  line-height: 1.2;
}

.field-value-text,
.summary-value {
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
}

.value-input {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-text);
  font-size: 18px;
  font-weight: 600;
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

.value-input:focus,
.toolbar-filter select:focus,
.search-bar input:focus,
.fluent-input:focus {
  outline: none;
}

.count-input {
  width: 1.5ch;
}

.subsidy-input {
  width: 40px;
}

.time-input {
  width: 86px;
  letter-spacing: 0.02em;
}

.date-input {
  width: 86px;
  letter-spacing: 0.02em;
}

.framed-input {
  min-height: 40px;
  padding: 0 12px;
  border: 1px solid #dce6f3;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.96);
}

.segment-wrap {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  padding: 6px;
  background: #f6f9fc;
  border-radius: 18px;
  border: 1px solid #e2ebf5;
}

.segment-btn {
  min-height: 44px;
  border: 0;
  border-radius: 14px;
  background: transparent;
  color: #52657f;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
}

.segment-btn.active {
  background: #fff;
  color: #0f6cbd;
  box-shadow: 0 8px 18px rgba(15, 108, 189, 0.12);
}

.summary-box,
.footer-row,
.toolbar-row,
.action-row,
.drawer-header,
.section-header,
.drawer-actions,
.pagination-row,
.exclude-item,
.exclude-right,
.action-buttons,
.time-row,
.subsidy-row,
.toolbar-left,
.candidate-item,
.search-bar {
  display: flex;
  align-items: center;
}

.summary-box,
.exclude-item {
  justify-content: space-between;
}

.footer-row,
.toolbar-row,
.action-row,
.drawer-header,
.section-header,
.pagination-row {
  justify-content: space-between;
}

.footer-row,
.toolbar-row,
.action-row,
.drawer-header,
.section-header,
.drawer-actions,
.pagination-row,
.toolbar-left,
.action-buttons,
.exclude-right,
.candidate-item,
.search-bar {
  gap: 12px;
}

.drawer-trigger,
.page-btn,
.action-btn,
.toolbar-btn {
  white-space: nowrap;
}

.info-pill,
.warning-pill,
.danger-pill,
.status-badge,
.subject-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 34px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.info-pill {
  background: #eaf3ff;
  color: #0f6cbd;
}

.warning-pill {
  background: #fff7ed;
  color: #c96a10;
  border: 1px solid #f3d2a4;
}

.danger-pill {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}

.exclude-toolbar,
.exclude-list,
.class-table,
.drawer-section,
.candidate-list,
.drawer-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.exclude-toolbar {
  flex-direction: row;
  flex-wrap: wrap;
}

.empty-box {
  min-height: 44px;
  display: flex;
  align-items: center;
  padding: 10px 12px;
  border-radius: 14px;
  border: 1px dashed #dce8f8;
  background: rgba(247, 251, 255, 0.76);
  color: var(--color-text-muted);
  font-size: 13px;
}

.error-box {
  border-color: rgba(216, 80, 80, 0.24);
  color: #b63d3d;
  background: rgba(255, 245, 245, 0.9);
}

.summary-grid,
.subsidy-row,
.time-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.schedule-row {
  display: grid;
  grid-template-columns: 1.1fr 1fr 1fr;
  gap: 14px;
}

.summary-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.summary-chip.warning,
.subject-badge.empty,
.status-badge.pending {
  background: #fff8f1;
  border-color: #f7d6ae;
  color: #c96a12;
}

.summary-chip.warning {
  border: 1px solid #f7d6ae;
}

.summary-chip {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.summary-chip2 {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.fluent-combo {
  position: relative;
  width: 220px;
}

.fluent-input,
.toolbar-filter,
.search-bar {
  min-height: 42px;
  border-radius: 14px;
  border: 1px solid #d8e4f2;
  background: rgba(255, 255, 255, 0.95);
}

.fluent-input {
  width: 100%;
  padding: 0 12px;
  font-size: 14px;
}

.combo-icon {
  position: absolute;
  right: 10px;
  top: 11px;
  font-size: 18px;
  color: #667085;
}

.select-field {
  padding-right: 32px;
}

.fluent-menu,
.subject-menu {
  position: absolute;
  padding: 6px;
  border: 1px solid #d8e4f2;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.98);
  box-shadow: 0 18px 40px rgba(35, 52, 78, 0.16);
  z-index: 24;
}

.fluent-menu {
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  max-height: 240px;
  overflow-y: auto;
}

.fluent-option,
.subject-menu-item {
  width: 100%;
  min-height: 38px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  text-align: left;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  color: #334155;
}

.fluent-option.selected,
.fluent-option:hover,
.subject-menu-item.active,
.subject-menu-item:hover {
  background: #eef5ff;
  color: #0f6cbd;
}

.menu-empty {
  padding: 10px 12px;
  color: var(--color-text-muted);
  font-size: 13px;
}

.class-table-head,
.class-table-row {
  display: grid;
  grid-template-columns: 44px 1.5fr 1fr 1fr 1fr;
  gap: 12px;
  align-items: center;
  padding: 14px 16px;
  border-radius: 18px;
}

.class-table-head {
  background: #f3f7fb;
  color: #5f708a;
  font-size: 13px;
  font-weight: 700;
}

.class-table-row {
  border: 1px solid #dce8f8;
  background: #fff;
}

.class-table-row.selected {
  border-color: #bad7ff;
  background: #f8fbff;
}

.check-cell {
  display: inline-flex;
  justify-content: center;
}

.subject-badge {
  justify-self: start;
  border: 1px solid #dbe5f3;
  background: #f7fbff;
  color: #334155;
}

.status-badge.done {
  background: #e9f9ef;
  color: #0f8b57;
}

.toolbar-filter {
  position: relative;
  width: fit-content;
  min-width: 0;
  flex: 0 0 auto;
}

.search-bar input {
  border: 0;
  background: transparent;
  font-size: 14px;
}

.toolbar-btn {
  min-height: 44px;
  padding: 0 16px;
  border-radius: 16px;
  border: 1px solid #d6e5fa;
  background: #eef5ff;
  color: #0f6cbd;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
}

.toolbar-btn:disabled,
.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-chip {
  padding: 10px 14px;
  border-radius: 16px;
  border: 1px solid #d6e5fa;
  background: #f7fbff;
  color: #4a5f80;
  font-size: 13px;
  font-weight: 700;
}

.page-btn {
  min-width: 44px;
  min-height: 40px;
  border-radius: 14px;
  border: 1px solid #d6e5fa;
  background: #fff;
  color: #4a5f80;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
}

.page-btn.active {
  background: #eef5ff;
  color: #0f6cbd;
}

.selection-strip {
  padding: 8px 12px;
  border-radius: 14px;
  background: #eef6ff;
  color: #0f6cbd;
  font-size: 13px;
  font-weight: 700;
}

.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.1);
  z-index: 40;
}

.config-drawer {
  position: fixed;
  top: 20px;
  right: 20px;
  max-height: calc(100vh - 40px);
  overflow-y: auto;
  width: min(560px, calc(100vw - 24px));
  padding: 24px;
  border-radius: 24px;
  border: 1px solid rgba(220, 230, 243, 0.92);
  background: rgba(255, 255, 255, 0.97);
  box-shadow: 0 22px 44px rgba(107, 124, 147, 0.2);
  z-index: 50;
}

.soft-panel {
  padding: 20px;
  border-radius: 22px;
  border: 1px solid #dce8f8;
  background: #f7fbff;
}

.class-config-section {
  padding-top: 8px;
  gap: 14px;
}

.middle-manager-drawer {
  width: min(500px, calc(100vw - 24px));
}

.drawer-title-block h3,
.section-header h4 {
  margin: 0;
}

.drawer-title-block p,
.section-header p,
.drawer-footer p {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 12px;
}

.drawer-note {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.35;
}

.summary-card :deep(h3) {
  font-size: 18px;
}

.summary-card :deep(p) {
  font-size: 12px;
  color: #7a879a;
}

.middle-footer,
.self-study-footer {
  padding-top: 4px;
}

.self-study-note {
  width: 220px;
  font-size: 11px;
  line-height: 1.35;
}

.drawer-close,
.icon-btn,
.text-btn {
  border: 0;
  background: transparent;
  cursor: pointer;
}

.icon-btn {
  color: #c26868;
}

.text-btn {
  color: #0f6cbd;
  font-size: 13px;
  font-weight: 700;
}

.drawer-close {
  width: 38px;
  height: 38px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
}

.search-bar {
  padding: 0 12px;
}

.middle-picker {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.middle-search {
  gap: 8px;
}

.middle-search input {
  flex: 1;
  min-width: 0;
}

.search-icon {
  font-size: 18px;
  color: #7a879a;
}

.candidate-item {
  min-height: 42px;
  justify-content: space-between;
  padding: 0 14px;
  border-radius: 14px;
  border: 1px dashed #c8dbf6;
  background: #f8fbff;
}

.candidate-action {
  color: #0f6cbd;
  font-size: 13px;
  font-weight: 700;
}

.drawer-actions {
  justify-content: flex-end;
}

.action-text {
  margin: 0;
  flex: 1 1 auto;
  min-width: 0;
  line-height: 1.5;
}

.action-copy {
  display: flex;
  flex: 1 1 auto;
  min-width: 0;
  flex-direction: column;
  gap: 4px;
}

.solver-summary {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.4;
}

.action-row {
  justify-content: flex-start;
}

.assignment-notice {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid #dce6f3;
  background: rgba(247, 251, 255, 0.82);
  font-size: 14px;
  line-height: 1.6;
  color: var(--color-text);
}

.assignment-notice.inline {
  flex: 0 1 340px;
  max-width: 340px;
  min-width: 0;
  padding: 10px 12px;
  border-radius: 14px;
  font-size: 13px;
  line-height: 1.45;
}

.assignment-notice-icon {
  font-size: 18px;
  line-height: 1.2;
  color: #0f6cbd;
}

.assignment-notice-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.assignment-notice-text {
  min-width: 0;
  overflow-wrap: anywhere;
}

.assignment-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.assignment-progress-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--color-text-muted);
  font-size: 12px;
}

.assignment-progress-track {
  width: 100%;
  height: 6px;
  border-radius: 999px;
  background: rgba(15, 108, 189, 0.12);
  overflow: hidden;
}

.assignment-progress-bar {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #0f6cbd 0%, #4aa3ff 100%);
  transition: width 220ms ease;
}

.title-stack {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.middle-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.middle-primary-btn {
  padding: 10px 14px;
  min-height: auto;
  border-radius: 14px;
}

.middle-filter-btn {
  min-height: auto;
  padding: 10px 12px;
  border-radius: 14px;
  border: 1px solid #dce6f3;
  background: #fff;
  color: #667085;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.middle-filter-btn.active {
  border-color: #c5dcff;
  background: #eaf3ff;
  color: #0f6cbd;
}

.middle-exception-item {
  padding: 12px;
}

.middle-pagination {
  align-items: center;
  padding-top: 2px;
}

.middle-person {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.middle-subtext {
  color: #667085;
  font-size: 12px;
  font-weight: 500;
}

.middle-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.middle-status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.middle-status-pill.off {
  background: #fff1f2;
  border: 1px solid #fbcdd2;
  color: #d1435b;
}

.middle-status-pill.on {
  background: #ecfdf3;
  border: 1px solid #b7e4c7;
  color: #15803d;
}

.subject-menu {
  position: fixed;
  width: 168px;
  max-height: calc(5 * 42px + 16px);
  overflow-y: auto;
  border-radius: 16px;
  z-index: 60;
}

.subject-menu-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.drawer-fade-enter-active,
.drawer-fade-leave-active,
.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: all 0.2s ease;
}

.drawer-fade-enter-from,
.drawer-fade-leave-to {
  opacity: 0;
}

.drawer-slide-enter-from,
.drawer-slide-leave-to {
  opacity: 0;
  transform: translateX(18px);
}

@media (max-width: 1200px) {
  .grid-two,
  .summary-grid,
  .subsidy-row,
  .time-row,
  .schedule-row {
    grid-template-columns: 1fr;
  }

  .footer-row,
  .toolbar-row,
  .action-row,
  .pagination-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
