<template>
  <section class="panel">
    <div class="grid-two top-grid">
      <ConfigCard class="top-card exam-count-card" title="监考人数配置">
        <div class="exam-count-content">
          <button class="count-btn" type="button" :disabled="defaultExamRoomRequiredCount <= 1" @click="decreaseCount" aria-label="减少人数">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
          
          <div class="count-display">
            <span class="count-number">{{ defaultExamRoomRequiredCount }}</span>
            <span class="count-unit">人</span>
          </div>
          
          <button class="count-btn" type="button" @click="increaseCount" aria-label="增加人数">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
        </div>
      </ConfigCard>

      <ConfigCard class="top-card middle-manager-card" title="校中层监考配置">
        <div class="card-stack">
          <div class="segment-wrap">
            <button class="segment-btn" :class="{ active: middleManagerDefaultEnabled }" type="button" @click="setMiddleManagerDefaultEnabled(true)">参与监考</button>
            <button class="segment-btn" :class="{ active: !middleManagerDefaultEnabled }" type="button" @click="setMiddleManagerDefaultEnabled(false)">不参与监考</button>
          </div>
          <div class="footer-row middle-footer">
            <div class="exception-tag">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                <circle cx="9" cy="7" r="4"></circle>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
              </svg>
              已设置 {{ middleManagerExceptionCount }} 位例外人员
            </div>
            <button class="drawer-trigger exception-btn" type="button" @click="openMiddleManagerDrawer">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
              配置例外
            </button>
          </div>
        </div>
      </ConfigCard>
    </div>

    <section class="schedule-card exclude-card">
      <div class="schedule-card-header">
        <div class="schedule-header-info">
          <div class="schedule-title-bar">
            <span class="schedule-title-line"></span>
            <h2 class="schedule-card-title">自定义排班规则</h2>
          </div>
          <div class="schedule-card-subtitle">当前已配置 <strong>{{ store.viewState.customRules.length }}</strong> 条规则</div>
        </div>
        <button class="schedule-btn-primary drawer-trigger" type="button" @click="openCustomRuleDrawer">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          添加排班规则
        </button>
      </div>

      <div v-if="store.viewState.customRules.length === 0" class="schedule-empty-state">
        <div class="schedule-empty-icon">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="8" y1="6" x2="21" y2="6"></line>
            <line x1="8" y1="12" x2="21" y2="12"></line>
            <line x1="8" y1="18" x2="21" y2="18"></line>
            <line x1="3" y1="6" x2="3.01" y2="6"></line>
            <line x1="3" y1="12" x2="3.01" y2="12"></line>
            <line x1="3" y1="18" x2="3.01" y2="18"></line>
          </svg>
        </div>
        <h3 class="schedule-empty-title">暂未添加排班规则</h3>
        <p class="schedule-empty-desc">点击右上方按钮添加规则，开始管理您的排班。</p>
      </div>
      <div v-else class="compact-rule-list schedule-rule-list">
        <div v-for="(item, index) in store.viewState.customRules" :key="index" class="compact-rule-item">
          <div class="compact-rule-main">
            <div class="compact-rule-header">
              <span :class="item.actionType === 'require' ? 'primary-pill' : 'danger-pill'">
                {{ item.actionType === 'require' ? '指定安排' : '禁排' }}
              </span>
              <strong class="custom-rule-teacher">{{ item.teacherName }}</strong>
              <span class="compact-rule-task">{{ ruleTaskScopeLabel(item.taskScopeType) }}</span>
              <span class="rule-tag">{{ formatRuleTimeScopeSummary(item) }}</span>
              <span class="rule-tag">{{ formatRuleTargetScopeSummary(item) }}</span>
            </div>
          </div>
          <div class="compact-rule-actions">
            <button class="text-btn" type="button" @click="openCustomRuleDetail(item)">详情</button>
            <button class="icon-btn" type="button" @click="removeCustomRule(item)">
              <span class="material-symbols-rounded">delete</span>
            </button>
          </div>
        </div>
      </div>
    </section>

    <div class="grid-two summary-grid-row">
      <section class="study-card summary-card self-study-card">
        <div class="study-card-header">
          <div class="study-title-bar">
            <span class="study-title-line"></span>
            <h2 class="study-card-title">全员自习</h2>
          </div>
        </div>
        <div class="study-data-grid">
          <div class="study-data-item">
            <div class="study-data-label">时间范围</div>
            <div class="study-data-value study-data-time">{{ selfStudyMonthDay }} {{ selfStudyStartTime }} - {{ selfStudyEndTime }}</div>
          </div>
          <div class="study-data-item">
            <div class="study-data-label">已配置班级</div>
            <div class="study-data-value">{{ configuredClassCount }} <span class="study-data-unit">个</span></div>
          </div>
          <div class="study-data-item warning">
            <div class="study-data-label">待补充</div>
            <div class="study-data-value">{{ pendingClassCount }} <span class="study-data-unit">个班级</span></div>
          </div>
        </div>
        <div class="study-card-footer">
          <span class="study-status-text" :class="{ pending: pendingClassCount > 0 }">
            <span class="material-symbols-rounded study-status-icon" aria-hidden="true">{{ pendingClassCount > 0 ? "error" : "check_circle" }}</span>
            {{ selfStudySummaryText }}
          </span>
          <button class="study-btn-primary drawer-trigger" type="button" @click="openSelfStudyDrawer">
            <span class="material-symbols-rounded" aria-hidden="true">tune</span>
            配置班级科目
          </button>
        </div>
      </section>

      <ConfigCard class="summary-card allowance-card">
        <div class="allowance-title-bar">
          <span class="allowance-title-line"></span>
          <span class="allowance-title-text">监考津贴</span>
        </div>
        <div class="allowance-items-container">
          <div class="allowance-item allowance-item-indoor">
            <div class="allowance-item-label">
              <span class="allowance-dot allowance-dot-inside"></span>
              场内监考津贴
            </div>
            <div class="allowance-item-value">
              <input
                class="allowance-value-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="indoorAllowancePerMinute"
                @blur="handleSaveConfig"
                @keyup.enter="handleSaveConfig"
              />
              <span class="allowance-value-unit">元 / 分钟</span>
            </div>
          </div>
          <div class="allowance-item allowance-item-outdoor">
            <div class="allowance-item-label">
              <span class="allowance-dot allowance-dot-outside"></span>
              场外监考津贴
            </div>
            <div class="allowance-item-value">
              <input
                class="allowance-value-input"
                type="number"
                min="0"
                step="0.1"
                v-model.number="outdoorAllowancePerMinute"
                @blur="handleSaveConfig"
                @keyup.enter="handleSaveConfig"
              />
              <span class="allowance-value-unit">元 / 分钟</span>
            </div>
          </div>
        </div>
      </ConfigCard>
    </div>

    <ConfigCard>
      <div class="action-row">
        <div class="action-copy">
          <p class="action-text">点击按钮为考场、自习室及楼层分配监考、看班老师 ~~~</p>
          <p v-if="store.viewState.staffOverview.generatedAt" class="solver-summary">{{ staffSolverSummary }}</p>
        </div>
        <div
          v-if="displayedAssignmentNotice || isAssignmentProgressVisible"
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
            <button
              v-if="assignmentNoticeLinkPath && !isAssignmentProgressVisible"
              class="assignment-notice-link"
              type="button"
              @click="openInvigilationExportFolder"
            >
              {{ assignmentNoticeLinkLabel }}
            </button>
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
          <button class="primary-btn action-btn" type="button" :disabled="store.viewState.assigning" @click="assignTeachers">{{ store.viewState.assigning ? "分配中..." : "开始分配" }}</button>
          <button
            class="secondary-btn action-btn"
            type="button"
            :disabled="!store.viewState.staffOverview.generatedAt || store.viewState.exportingInvigilation"
            @click="exportInvigilationSchedule"
          >
            {{ store.viewState.exportingInvigilation ? "导出中..." : "导出监考表" }}
          </button>
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
            <span class="pending-pill">{{ pendingClassCount }} 个待处理</span>
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
                  class="grade-filter-select"
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
            <span class="exception-pill">{{ middleManagerExceptionTeacherIdsDraft.length }} 位例外</span>
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

    <div v-if="dialogState.visible" class="dialog-mask" @click.self="closeDialog(false)">
      <section class="dialog card-shell">
        <header class="dialog-head">
          <h3>{{ dialogState.title }}</h3>
          <button class="dialog-close" type="button" @click="closeDialog(false)">×</button>
        </header>
        <p class="dialog-summary">{{ dialogState.summary }}</p>
        <ul v-if="dialogState.details.length > 0" class="dialog-details">
          <li v-for="(line, index) in dialogState.details" :key="index">{{ line }}</li>
        </ul>
        <footer class="dialog-actions">
          <button v-if="dialogState.kind === 'confirm'" class="secondary-btn" type="button" @click="closeDialog(false)">
            {{ dialogState.cancelText }}
          </button>
          <button class="primary-btn" type="button" @click="closeDialog(true)">
            {{ dialogState.confirmText }}
          </button>
        </footer>
      </section>
    </div>
    <transition name="drawer-slide">
      <aside v-if="customRuleDrawerOpen" class="config-drawer custom-rule-drawer">
        <div class="drawer-header">
          <div class="drawer-title-block">
            <h3>添加排班规则</h3>
            <p>按时间范围、任务类型和作用对象配置禁排或指定安排。</p>
          </div>
          <button class="drawer-close" type="button" @click="closeCustomRuleDrawer"><span class="material-symbols-rounded">close</span></button>
        </div>

        <section class="drawer-section soft-panel custom-rule-panel">
          <div class="form-group">
            <label class="field-label form-label">规则动作</label>
            <div class="segment-wrap full-width">
              <button class="segment-btn" :class="{ active: draftRule.actionType === 'exclude' }" type="button" @click="draftRule.actionType = 'exclude'">禁排</button>
              <button class="segment-btn" :class="{ active: draftRule.actionType === 'require' }" type="button" @click="draftRule.actionType = 'require'">指定安排</button>
            </div>
          </div>

          <div v-if="draftRule.actionType" class="form-group form-group-step">
            <label class="field-label form-label">指定教师 <span class="required-mark">*</span></label>
            <FluentSelect
              v-model="draftRule.teacherId"
              :options="teacherSelectOptions as any"
              placeholder="请选择教师"
              searchable
              class="teacher-select-full"
            />
          </div>

          <div v-if="draftRule.teacherId" class="form-group form-group-step">
            <label class="field-label form-label">时间范围</label>
            <div class="segment-wrap full-width">
              <button class="segment-btn" :class="{ active: draftRule.timeScopeType === 'exam_session' }" type="button" @click="selectRuleTimeScopeType('exam_session')">考试时段</button>
              <button class="segment-btn" :class="{ active: draftRule.timeScopeType === 'full_self_study' }" type="button" @click="selectRuleTimeScopeType('full_self_study')">全员自习时段</button>
            </div>
            <template v-if="draftRule.timeScopeType === 'exam_session'">
              <div v-if="groupedExamSessionRuleOptions.length > 0" class="selection-toolbar">
                <div class="selection-toolbar-copy">
                  <strong>已选 {{ selectedRuleTimeLabels.length }} 个考试时段</strong>
                </div>
                <div class="selection-toolbar-actions">
                  <button
                    class="toolbar-toggle-btn"
                    type="button"
                    :disabled="groupedExamSessionRuleOptions.length === 0"
                    @click="toggleAllRuleTimeScopes"
                  >
                    {{ allRuleTimeScopesSelected ? '取消全选' : '全选' }}
                  </button>
                </div>
              </div>
              <div class="selection-list compact-option-list">
                <label v-for="option in groupedExamSessionRuleOptions" :key="option.id" class="check-option compact-option">
                  <input
                    type="checkbox"
                    :checked="isRuleTimeScopeSelected(option.sessionIds)"
                    @change="toggleRuleTimeScopeIds(option.sessionIds)"
                  />
                  <div class="target-copy time-scope-copy">
                    <template v-for="(line, idx) in option.label.split('\n')" :key="idx">
                      <span :class="idx === 0 ? 'time-scope-subject' : 'time-scope-datetime'">{{ line }}</span>
                    </template>
                  </div>
                </label>
              </div>
            </template>
            <div v-else class="scope-preview">
              {{ fullSelfStudyRuleLabel }}
            </div>
            <p v-if="draftRule.timeScopeType === 'exam_session' && groupedExamSessionRuleOptions.length === 0" class="drawer-note">
              暂无可选考试时段，请先配置考试时间，或先完成一次考场/监考任务生成。
            </p>
            <p v-else-if="draftRule.timeScopeType === 'exam_session' && draftRule.timeScopeIds.length === 0" class="drawer-note">
              请先选择一个或多个考试时段，再指定具体考场、班级或楼层任务。
            </p>
          </div>

          <div v-if="showTaskScopeStep" class="form-group form-group-step">
            <label class="field-label form-label">任务类型</label>
            <div class="option-grid">
              <label v-for="option in availableTaskScopeOptions" :key="option.value" class="check-option single-option" :class="{ active: draftRule.taskScopeType === option.value }">
                <input
                  type="radio"
                  name="custom-rule-task-scope"
                  :checked="draftRule.taskScopeType === option.value"
                  @change="selectRuleTaskScopeType(option.value)"
                />
                <span>{{ option.label }}</span>
              </label>
            </div>
          </div>

          <div v-if="showTargetScopeStep" class="form-group form-group-step">
            <label class="field-label form-label">作用对象</label>
            <div class="segment-wrap full-width">
              <button class="segment-btn" :class="{ active: draftRule.targetScopeType === 'all' }" type="button" @click="selectRuleTargetScopeType('all')">全部对象</button>
              <button class="segment-btn" :class="{ active: draftRule.targetScopeType === 'selected_targets' }" type="button" @click="selectRuleTargetScopeType('selected_targets')">指定对象</button>
            </div>
            <p class="drawer-note" v-if="draftRule.targetScopeType === 'all'">不选具体对象时，规则默认作用于当前时间范围内的全部匹配任务。</p>
            <template v-else>
              <p v-if="ruleTargetHintText" class="drawer-note">{{ ruleTargetHintText }}</p>
              <template v-if="availableRuleTargetOptions.length > 0">
                <div class="selection-toolbar">
                  <div class="selection-toolbar-copy">
                    <strong>已选 {{ draftRule.targetIds.length }} 个对象</strong>
                  </div>
                  <div class="selection-toolbar-actions">
                    <button
                      class="toolbar-toggle-btn"
                      type="button"
                      :disabled="availableRuleTargetOptions.length === 0"
                      @click="toggleAllRuleTargets"
                    >
                      {{ allRuleTargetsSelected ? '取消全选' : '全选' }}
                    </button>
                  </div>
                </div>
                <div class="selection-list compact-option-list">
                  <label v-for="option in availableRuleTargetOptions" :key="option.id" class="check-option target-option compact-option">
                    <input
                      type="checkbox"
                      :checked="draftRule.targetIds.includes(option.id)"
                      @change="toggleRuleTargetId(option.id)"
                    />
                    <div class="target-copy target-option-copy">
                      <span class="target-option-label">{{ option.label }}</span>
                      <small v-if="option.subtitle" class="target-option-subtitle">{{ formatTargetOptionSubtitle(option.subtitle) }}</small>
                    </div>
                  </label>
                </div>
              </template>
            </template>
          </div>

          <div class="custom-rule-summary-box">
            <span class="field-label form-label">规则摘要</span>
            <strong>{{ draftRuleSummary }}</strong>
          </div>
        </section>

        <div class="drawer-footer custom-rule-footer">
          <p v-if="draftRuleError" class="drawer-error">{{ draftRuleError }}</p>
          <p v-else class="drawer-note">保存时会校验冲突规则，命中冲突将直接阻止保存。</p>
          <div class="drawer-actions">
            <button class="secondary-btn" type="button" @click="closeCustomRuleDrawer">取消</button>
            <button class="primary-btn" :disabled="!draftRule.actionType || !draftRule.teacherId" @click="saveDraftRule">保存规则</button>
          </div>
        </div>
      </aside>
    </transition>

    <transition name="drawer-slide">
      <aside v-if="customRuleDetailOpen && selectedCustomRule" class="config-drawer custom-rule-detail-drawer">
        <div class="drawer-header">
          <div class="drawer-title-block">
            <h3>规则详情</h3>
            <p>{{ selectedCustomRule.teacherName }} 的{{ ruleTaskScopeLabel(selectedCustomRule.taskScopeType) }}规则</p>
          </div>
          <button class="drawer-close" type="button" @click="closeCustomRuleDetail"><span class="material-symbols-rounded">close</span></button>
        </div>

        <section class="drawer-section soft-panel custom-rule-panel">
          <div class="detail-summary-grid">
            <div class="summary-box">
              <span class="field-label">规则动作</span>
              <strong>{{ selectedCustomRule.actionType === "require" ? "指定安排" : "禁排" }}</strong>
            </div>
            <div class="summary-box">
              <span class="field-label">任务类型</span>
              <strong>{{ ruleTaskScopeLabel(selectedCustomRule.taskScopeType) }}</strong>
            </div>
            <div class="summary-box">
              <span class="field-label">时间范围</span>
              <strong>{{ formatRuleTimeScopeSummary(selectedCustomRule) }}</strong>
            </div>
            <div class="summary-box">
              <span class="field-label">作用对象</span>
              <strong>{{ formatRuleTargetScopeSummary(selectedCustomRule) }}</strong>
            </div>
          </div>

          <div class="form-group">
            <label class="field-label form-label">完整时间范围</label>
            <div class="detail-chip-list">
              <span
                v-for="(label, index) in resolvedRuleTimeScopeLabels(selectedCustomRule)"
                :key="`${label}-${index}`"
                class="rule-tag detail-chip"
              >
                {{ label }}
              </span>
            </div>
          </div>

          <div class="form-group">
            <label class="field-label form-label">完整作用对象</label>
            <div v-if="selectedCustomRule.targetScopeType === 'all'" class="scope-preview">
              全部对象
            </div>
            <div v-else class="detail-chip-list">
              <span
                v-for="(label, index) in selectedCustomRule.targetLabels"
                :key="`${label}-${index}`"
                class="rule-tag detail-chip"
              >
                {{ label }}
              </span>
            </div>
          </div>
        </section>

        <div class="drawer-footer custom-rule-footer">
          <p class="drawer-note">详情抽屉仅用于查看规则内容，修改时请删除后重新添加。</p>
          <div class="drawer-actions">
            <button class="secondary-btn" type="button" @click="closeCustomRuleDetail">关闭</button>
          </div>
        </div>
      </aside>
    </transition>

  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import type { ClassConfigRow } from "../../../entities/class-config/model";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ExamStaffAssignmentProgress,
  InvigilationConfig,
  InvigilationCustomRule,
  InvigilationRuleTaskScopeType,
  InvigilationRuleTimeScopeType,
} from "../../../entities/exam-plan/model";
import type { Subject } from "../../../entities/score/model";
import { Subject as SubjectEnum } from "../../../entities/score/model";
import { revealInExplorer } from "../../../shared/utils/appLog";
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

interface DraftRuleTimeScopeOption {
  id: string;
  label: string;
  sessionIds: number[];
  startAt: string;
  endAt: string;
}

interface RuleTimeScopeLabelPart {
  gradeName: string;
  subjectLabel: string;
}

type ReadonlyInvigilationCustomRule = Omit<InvigilationCustomRule, "timeScopeIds" | "timeScopeLabels" | "targetIds" | "targetLabels"> & {
  readonly timeScopeIds: readonly number[];
  readonly timeScopeLabels: readonly string[];
  readonly targetIds: readonly string[];
  readonly targetLabels: readonly string[];
};

interface AssignmentNotice {
  type: "success" | "warning" | "error";
  text: string;
  linkPath?: string;
  linkLabel?: string;
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
let removeAssignmentProgressListener: UnlistenFn | null = null;

let dialogResolver: ((value: boolean) => void) | null = null;
const dialogState = reactive({
  visible: false,
  kind: "confirm" as "confirm" | "alert",
  title: "",
  summary: "",
  details: [] as string[],
  confirmText: "确认",
  cancelText: "取消",
});

function openDialog(options: {
  kind: "confirm" | "alert";
  title: string;
  summary: string;
  details?: string[];
  confirmText?: string;
  cancelText?: string;
}) {
  dialogState.visible = true;
  dialogState.kind = options.kind;
  dialogState.title = options.title;
  dialogState.summary = options.summary;
  dialogState.details = options.details ?? [];
  dialogState.confirmText = options.confirmText ?? (options.kind === "confirm" ? "确认" : "知道了");
  dialogState.cancelText = options.cancelText ?? "取消";
  return new Promise<boolean>((resolve) => {
    dialogResolver = resolve;
  });
}

function closeDialog(result: boolean) {
  if (dialogResolver) {
    dialogResolver(result);
    dialogResolver = null;
  }
  dialogState.visible = false;
}

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

const activeDrawer = computed(() => (
  selfStudyDrawerOpen.value
    ? "selfStudy"
    : middleManagerDrawerOpen.value
      ? "middleManager"
      : customRuleDrawerOpen.value
        ? "customRule"
        : customRuleDetailOpen.value
          ? "customRuleDetail"
          : null
));
const middleManagerDefaultEnabled = computed(() => store.viewState.invigilationConfig.middleManagerDefaultEnabled);
const middleManagerExceptionCount = computed(() => store.viewState.invigilationConfig.middleManagerExceptionTeacherIds.length);
const teacherSelectOptions = computed(() => [{ label: "选择教师", value: "" }, ...store.viewState.teachers.map((item) => ({ label: item.teacherName, value: item.id }))]);
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
      ? `，其他老师补位 ${overview.fallbackPoolAssignments} 项`
      : "";
  return `CP-SAT，${statusLabel}，耗时 ${formatSolveDuration(overview.solveDurationMs)}${fallbackSummary}`;
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
const persistedInvigilationExportNotice = computed<AssignmentNotice | null>(() => {
  const path = store.viewState.lastInvigilationExportPath;
  if (!path) {
    return null;
  }
  return {
    type: "success",
    text: "监考表已导出，点击下方链接打开文件所在位置。",
    linkPath: path,
    linkLabel: exportFileName(path),
  };
});
const displayedAssignmentNotice = computed(() => assignmentNotice.value ?? persistedInvigilationExportNotice.value);
const assignmentProgress = computed(() => store.viewState.assignmentProgress);
const assignmentNoticeIcon = computed(() => {
  if (isAssignmentProgressVisible.value) return "hourglass_top";
  const noticeType = displayedAssignmentNotice.value?.type;
  return noticeType === "success" ? "check_circle" : noticeType === "warning" ? "warning" : "info";
});
const assignmentNoticeText = computed(() => {
  if (isAssignmentProgressVisible.value) {
    return assignmentProgress.value?.message || "正在准备监考分配...";
  }
  return displayedAssignmentNotice.value?.text || "";
});
const assignmentNoticeLinkPath = computed(() => displayedAssignmentNotice.value?.linkPath || "");
const assignmentNoticeLinkLabel = computed(() => displayedAssignmentNotice.value?.linkLabel || "");

const customRuleDrawerOpen = ref(false);
const customRuleDetailOpen = ref(false);
const selectedCustomRule = ref<ReadonlyInvigilationCustomRule | null>(null);
const draftRuleError = ref("");
const draftRule = ref<{
  actionType: "exclude" | "require" | "";
  teacherId: number | "";
  timeScopeType: InvigilationRuleTimeScopeType;
  timeScopeIds: number[];
  taskScopeType: InvigilationRuleTaskScopeType;
  targetScopeType: "all" | "selected_targets";
  targetIds: string[];
}>({
  actionType: "",
  teacherId: "",
  timeScopeType: "exam_session",
  timeScopeIds: [],
  taskScopeType: "exam_room",
  targetScopeType: "all",
  targetIds: [],
});

const examSessionRuleOptions = computed(() => store.viewState.customRuleOptions.examSessionOptions);
const groupedExamSessionRuleOptions = computed<DraftRuleTimeScopeOption[]>(() => {
  const grouped = new Map<string, DraftRuleTimeScopeOption>();
  for (const option of examSessionRuleOptions.value) {
    const key = `${option.startAt}__${option.endAt}`;
    const existing = grouped.get(key);
    if (existing) {
      existing.sessionIds.push(option.id);
      existing.label = buildGroupedRuleTimeScopeLabel(
        existing.sessionIds
          .map((sessionId) => examSessionRuleOptions.value.find((item) => item.id === sessionId))
          .filter((item): item is NonNullable<typeof item> => Boolean(item)),
      );
      continue;
    }
    grouped.set(key, {
      id: key,
      label: buildGroupedRuleTimeScopeLabel([option]),
      sessionIds: [option.id],
      startAt: option.startAt,
      endAt: option.endAt,
    });
  }
  return Array.from(grouped.values()).sort((left, right) =>
    left.startAt.localeCompare(right.startAt) || left.label.localeCompare(right.label, "zh-CN"),
  );
});
const fullSelfStudyRuleLabel = computed(
  () => store.viewState.customRuleOptions.fullSelfStudyOption?.label || "全员自习时段暂未配置",
);
const availableTaskScopeOptions = computed(() => {
  if (draftRule.value.timeScopeType === "full_self_study") {
    return [{ label: "全员自习看班", value: "full_self_study" as InvigilationRuleTaskScopeType }];
  }
  return [
    { label: "考试任务", value: "exam_room" as InvigilationRuleTaskScopeType },
    { label: "考试期间自习看班", value: "exam_linked_self_study" as InvigilationRuleTaskScopeType },
    { label: "流动监考", value: "floor_rover" as InvigilationRuleTaskScopeType },
  ];
});
const availableRuleTargetOptions = computed(() => {
  const taskScopeType = draftRule.value.taskScopeType;
  const timeScopeType = draftRule.value.timeScopeType;
  return store.viewState.customRuleOptions.targetOptions.filter((option) => {
    if (option.taskScopeType !== taskScopeType) {
      return false;
    }
    if (option.timeScopeType !== timeScopeType) {
      return false;
    }
    if (timeScopeType === "exam_session") {
      return draftRule.value.timeScopeIds.includes(option.timeScopeId || -1);
    }
    return true;
  });
});
const selectedRuleTargetOptions = computed(() =>
  availableRuleTargetOptions.value.filter((option) => draftRule.value.targetIds.includes(option.id)),
);
const excludeCustomRuleCount = computed(() =>
  store.viewState.customRules.filter((rule) => rule.actionType === "exclude").length,
);
const requireCustomRuleCount = computed(() =>
  store.viewState.customRules.filter((rule) => rule.actionType === "require").length,
);
const selectedRuleTeacherName = computed(
  () => store.viewState.teachers.find((item) => item.id === draftRule.value.teacherId)?.teacherName || "",
);
const selectedRuleTimeLabels = computed(() => {
  if (draftRule.value.timeScopeType === "full_self_study") {
    return store.viewState.customRuleOptions.fullSelfStudyOption
      ? [store.viewState.customRuleOptions.fullSelfStudyOption.label]
      : [];
  }
  return groupedExamSessionRuleOptions.value
    .filter((option) => option.sessionIds.every((sessionId) => draftRule.value.timeScopeIds.includes(sessionId)))
    .map((option) => option.label);
});
const allRuleTimeScopesSelected = computed(() =>
  groupedExamSessionRuleOptions.value.length > 0 &&
  groupedExamSessionRuleOptions.value.every((option) => isRuleTimeScopeSelected(option.sessionIds)),
);
const allRuleTargetsSelected = computed(() =>
  availableRuleTargetOptions.value.length > 0 &&
  availableRuleTargetOptions.value.every((option) => draftRule.value.targetIds.includes(option.id)),
);
const showTaskScopeStep = computed(() => {
  if (draftRule.value.timeScopeType === "full_self_study") {
    return true;
  }
  return draftRule.value.timeScopeIds.length > 0;
});
const showTargetScopeStep = computed(() => {
  if (draftRule.value.timeScopeType === "full_self_study") {
    return true;
  }
  return draftRule.value.timeScopeIds.length > 0;
});
const ruleTargetHintText = computed(() => {
  if (draftRule.value.targetScopeType !== "selected_targets") {
    return "";
  }
  if (draftRule.value.timeScopeType === "exam_session" && draftRule.value.timeScopeIds.length === 0) {
    return "请先选择考试时段，再指定具体考场、班级或楼层任务。";
  }
  if (availableRuleTargetOptions.value.length === 0) {
    return "当前没有可选对象。若要指定考场或班级，请先完成一次考场/监考任务生成。";
  }
  return "";
});
const draftRuleSummary = computed(() => {
  const actionLabel = draftRule.value.actionType === "require" ? "指定安排" : draftRule.value.actionType === "exclude" ? "禁排" : "未选择动作";
  const teacherName = selectedRuleTeacherName.value || "某位老师";
  const timeLabel = selectedRuleTimeLabels.value.length > 0
    ? selectedRuleTimeLabels.value.join("、")
    : draftRule.value.timeScopeType === "full_self_study"
      ? "全员自习时段"
      : "未选择考试时段";
  const taskLabel = ruleTaskScopeLabel(draftRule.value.taskScopeType);
  const targetLabel = draftRule.value.targetScopeType === "all"
    ? "全部对象"
    : selectedRuleTargetOptions.value.length > 0
      ? selectedRuleTargetOptions.value.map((option) => option.label).join("、")
      : "未选择对象";
  return `${actionLabel} ${teacherName} 在 ${timeLabel} 的 ${taskLabel}（${targetLabel}）`;
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

function increaseCount() {
  defaultExamRoomRequiredCount.value += 1;
  void saveConfig();
}

function decreaseCount() {
  if (defaultExamRoomRequiredCount.value > 1) {
    defaultExamRoomRequiredCount.value -= 1;
    void saveConfig();
  }
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
  if (customRuleDrawerOpen.value) closeCustomRuleDrawer();
  if (customRuleDetailOpen.value) closeCustomRuleDetail();
}

function openCustomRuleDrawer() {
  closeActiveDrawer();
  draftRule.value = {
    actionType: "",
    teacherId: "",
    timeScopeType: "exam_session",
    timeScopeIds: [],
    taskScopeType: "exam_room",
    targetScopeType: "all",
    targetIds: [],
  };
  draftRuleError.value = "";
  customRuleDrawerOpen.value = true;
}

function closeCustomRuleDrawer() {
  customRuleDrawerOpen.value = false;
}

function openCustomRuleDetail(rule: ReadonlyInvigilationCustomRule) {
  closeActiveDrawer();
  selectedCustomRule.value = rule;
  customRuleDetailOpen.value = true;
}

function closeCustomRuleDetail() {
  customRuleDetailOpen.value = false;
  selectedCustomRule.value = null;
}

async function saveDraftRule() {
  if (!draftRule.value.actionType) {
    draftRuleError.value = "请选择规则动作";
    return;
  }
  if (!draftRule.value.teacherId) {
    draftRuleError.value = "请选择教师";
    return;
  }
  if (draftRule.value.timeScopeType === "exam_session" && draftRule.value.timeScopeIds.length === 0) {
    draftRuleError.value = "请至少选择一个考试时段";
    return;
  }
  if (draftRule.value.targetScopeType === "selected_targets" && draftRule.value.targetIds.length === 0) {
    draftRuleError.value = "请选择至少一个作用对象";
    return;
  }
  const teacher = store.viewState.teachers.find((item) => item.id === draftRule.value.teacherId);
  if (!teacher) {
    draftRuleError.value = "未找到所选教师";
    return;
  }

  const newRule: InvigilationCustomRule = {
    actionType: draftRule.value.actionType,
    teacherId: teacher.id,
    teacherName: teacher.teacherName,
    timeScopeType: draftRule.value.timeScopeType,
    timeScopeIds: [...draftRule.value.timeScopeIds],
    timeScopeLabels: [...selectedRuleTimeLabels.value],
    taskScopeType: draftRule.value.taskScopeType,
    targetScopeType: draftRule.value.targetScopeType,
    targetIds: draftRule.value.targetScopeType === "all" ? [] : [...draftRule.value.targetIds],
    targetLabels: draftRule.value.targetScopeType === "all"
      ? []
      : selectedRuleTargetOptions.value.map((option) => option.label),
  };

  const currentRules: InvigilationCustomRule[] = store.viewState.customRules.map((rule) => ({
    ...rule,
    timeScopeIds: [...rule.timeScopeIds],
    timeScopeLabels: [...rule.timeScopeLabels],
    targetIds: [...rule.targetIds],
    targetLabels: [...rule.targetLabels],
  }));
  currentRules.unshift(newRule);
  draftRuleError.value = "";
  try {
    await store.saveCustomRules(currentRules);
    closeCustomRuleDrawer();
  } catch (error) {
    draftRuleError.value = error instanceof Error ? error.message : String(error);
  }
}

async function removeCustomRule(ruleToRemove: ReadonlyInvigilationCustomRule) {
  const currentRules: InvigilationCustomRule[] = store.viewState.customRules
    .filter((rule) => rule !== ruleToRemove)
    .map((rule) => ({
      ...rule,
      timeScopeIds: [...rule.timeScopeIds],
      timeScopeLabels: [...rule.timeScopeLabels],
      targetIds: [...rule.targetIds],
      targetLabels: [...rule.targetLabels],
    }));
  await store.saveCustomRules(currentRules);
}

function ruleTaskScopeLabel(taskScopeType: InvigilationRuleTaskScopeType) {
  const labelMap: Record<InvigilationRuleTaskScopeType, string> = {
    exam_room: "考试任务",
    exam_linked_self_study: "考试期间自习看班",
    full_self_study: "全员自习看班",
    floor_rover: "流动监考",
  };
  return labelMap[taskScopeType];
}

function formatRuleTimeScope(rule: ReadonlyInvigilationCustomRule) {
  if (rule.timeScopeLabels.length > 0) {
    return rule.timeScopeLabels.join("、");
  }
  return rule.timeScopeType === "full_self_study" ? "全员自习时段" : "未设置考试时段";
}

function formatRuleTimeScopeSummary(rule: ReadonlyInvigilationCustomRule) {
  if (rule.timeScopeType === "full_self_study") {
    return "全员自习时段";
  }
  if (rule.timeScopeLabels.length <= 1) {
    return formatRuleTimeScope(rule);
  }
  return `${rule.timeScopeLabels.length} 个考试时段`;
}

function formatRuleTargetScope(rule: ReadonlyInvigilationCustomRule) {
  if (rule.targetScopeType === "all") {
    return "全部对象";
  }
  if (rule.targetLabels.length > 0) {
    return rule.targetLabels.join("、");
  }
  return "指定对象";
}

function formatRuleTargetScopeSummary(rule: ReadonlyInvigilationCustomRule) {
  if (rule.targetScopeType === "all") {
    return "全部对象";
  }
  if (rule.targetLabels.length <= 1) {
    return formatRuleTargetScope(rule);
  }
  return `${rule.targetLabels.length} 个对象`;
}

function resolvedRuleTimeScopeLabels(rule: ReadonlyInvigilationCustomRule) {
  if (rule.timeScopeLabels.length > 0) {
    return [...rule.timeScopeLabels];
  }
  return [formatRuleTimeScope(rule)];
}

function selectRuleTimeScopeType(nextType: InvigilationRuleTimeScopeType) {
  draftRule.value.timeScopeType = nextType;
  draftRule.value.timeScopeIds = [];
  draftRule.value.targetIds = [];
  if (nextType === "full_self_study") {
    draftRule.value.taskScopeType = "full_self_study";
  } else if (draftRule.value.taskScopeType === "full_self_study") {
    draftRule.value.taskScopeType = "exam_room";
  }
}

function selectRuleTaskScopeType(taskScopeType: InvigilationRuleTaskScopeType) {
  draftRule.value.taskScopeType = taskScopeType;
  draftRule.value.targetIds = [];
}

function selectRuleTargetScopeType(scopeType: "all" | "selected_targets") {
  draftRule.value.targetScopeType = scopeType;
  if (scopeType === "all") {
    draftRule.value.targetIds = [];
  }
}

function isRuleTimeScopeSelected(sessionIds: number[]) {
  return sessionIds.every((sessionId) => draftRule.value.timeScopeIds.includes(sessionId));
}

function toggleRuleTimeScopeIds(sessionIds: number[]) {
  const nextIds = new Set(draftRule.value.timeScopeIds);
  const shouldSelect = !sessionIds.every((sessionId) => nextIds.has(sessionId));
  for (const sessionId of sessionIds) {
    if (shouldSelect) {
      nextIds.add(sessionId);
    } else {
      nextIds.delete(sessionId);
    }
  }
  draftRule.value.timeScopeIds = Array.from(nextIds).sort((left, right) => left - right);
  draftRule.value.targetIds = draftRule.value.targetIds.filter((targetId) =>
    availableRuleTargetOptions.value.some((option) => option.id === targetId),
  );
}

function toggleAllRuleTimeScopes() {
  if (allRuleTimeScopesSelected.value) {
    draftRule.value.timeScopeIds = [];
    draftRule.value.targetIds = [];
    return;
  }
  const nextIds = new Set<number>();
  for (const option of groupedExamSessionRuleOptions.value) {
    for (const sessionId of option.sessionIds) {
      nextIds.add(sessionId);
    }
  }
  draftRule.value.timeScopeIds = Array.from(nextIds).sort((left, right) => left - right);
}

function buildGroupedRuleTimeScopeLabel(
  options: Array<{ label: string; startAt: string; endAt: string }>,
) {
  if (options.length === 0) {
    return "";
  }
  const dateTimeLabel = formatRuleTimeRange(options[0].startAt, options[0].endAt);
  const parts = options
    .map((option) => parseRuleTimeScopeLabelPart(option.label))
    .filter((part): part is RuleTimeScopeLabelPart => Boolean(part));
  if (parts.length === 0) {
    return `${options[0].label}`.trim();
  }

  const normalizedSubjectSet = new Set(parts.map((part) => part.subjectLabel));
  const topicLabel = normalizedSubjectSet.size === 1
    ? parts[0].subjectLabel
    : Array.from(new Set(parts.map((part) => `${part.gradeName}${part.subjectLabel}`))).join("、");
  return `${topicLabel}\n${dateTimeLabel}`;
}

function parseRuleTimeScopeLabelPart(label: string): RuleTimeScopeLabelPart | null {
  const tokens = label.trim().split(/\s+/);
  if (tokens.length < 2) {
    return null;
  }
  return {
    gradeName: tokens[0],
    subjectLabel: normalizeRuleTimeScopeSubject(tokens[1]),
  };
}

function normalizeRuleTimeScopeSubject(subjectLabel: string) {
  if (["英语", "俄语", "日语"].includes(subjectLabel)) {
    return "外语";
  }
  return subjectLabel;
}

function formatRuleTimeRange(startAt: string, endAt: string) {
  if (startAt.length >= 16 && endAt.length >= 16) {
    const datePart = startAt.slice(5, 10);
    const startTime = startAt.slice(11, 16);
    const endTime = endAt.slice(11, 16);
    const [startHour, startMinute] = startTime.split(":").map(Number);
    const [endHour, endMinute] = endTime.split(":").map(Number);
    const startPeriod = getPeriodLabel(startHour, startMinute);
    const endPeriod = getPeriodLabel(endHour, endMinute);
    if (startPeriod === endPeriod) {
      return `${datePart} ${startPeriod}${startTime} — ${endTime}`;
    }
    return `${datePart} ${startPeriod}${startTime} — ${endPeriod}${endTime}`;
  } else {
    return `${startAt} - ${endAt}`;
  }
}

function getPeriodLabel(hour: number, minute: number) {
  if (hour < 12 || (hour === 12 && minute === 0)) {
    return "上午";
  } else if (hour < 18 || (hour === 18 && minute < 30)) {
    return "下午";
  }
  return "晚上";
}

function formatTargetOptionSubtitle(subtitle: string) {
  const timePattern = /(\d{2}:\d{2})-(\d{2}:\d{2})/;
  const match = subtitle.match(timePattern);
  if (!match) return subtitle;
  const startTime = match[1];
  const endTime = match[2];
  const [startHour, startMinute] = startTime.split(":").map(Number);
  const [endHour, endMinute] = endTime.split(":").map(Number);
  const startPeriod = getPeriodLabel(startHour, startMinute);
  const endPeriod = getPeriodLabel(endHour, endMinute);
  let replacement: string;
  if (startPeriod === endPeriod) {
    replacement = `${startPeriod}${startTime} — ${endTime}`;
  } else {
    replacement = `${startPeriod}${startTime} — ${endPeriod}${endTime}`;
  }
  return subtitle.replace(timePattern, replacement);
}

function toggleRuleTargetId(id: string) {
  const nextIds = new Set(draftRule.value.targetIds);
  if (nextIds.has(id)) {
    nextIds.delete(id);
  } else {
    nextIds.add(id);
  }
  draftRule.value.targetIds = Array.from(nextIds);
}

function toggleAllRuleTargets() {
  if (allRuleTargetsSelected.value) {
    draftRule.value.targetIds = [];
    return;
  }
  const nextIds = new Set<string>();
  for (const option of availableRuleTargetOptions.value) {
    nextIds.add(option.id);
  }
  draftRule.value.targetIds = Array.from(nextIds);
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


async function showAssignmentNotice(type: AssignmentNotice["type"], text: string, options?: Partial<AssignmentNotice>) {
  assignmentNotice.value = { type, text, ...options };
  await nextTick();
  assignmentNoticeEl.value?.scrollIntoView({
    behavior: "smooth",
    block: "nearest",
  });
}

function exportFileName(path: string) {
  const matched = path.match(/[^\\/]+$/);
  return matched?.[0] ?? path;
}

function formatSolveDuration(durationMs: number) {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  if (totalSeconds < 60) {
    return `${totalSeconds} 秒`;
  }
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (seconds === 0) {
    return `${minutes} 分钟`;
  }
  return `${minutes} 分 ${seconds} 秒`;
}

async function assignTeachers() {
  if (store.viewState.staffOverview.generatedAt) {
    const confirmed = await openDialog({
      kind: "confirm",
      title: "系统已存在分配数据",
      summary: "重新分配耗时较长，且将覆盖当前生效的监考排班。",
      details: ["是否确认重新进行分配？"],
      confirmText: "确认",
      cancelText: "取消",
    });
    if (!confirmed) {
      return;
    }
  }

  assignmentNotice.value = null;
  store.setAssignmentProgress({
    status: "running",
    stage: "preparing",
    stageLabel: "准备开始",
    percent: 0,
    message: "正在准备监考分配...",
    completedSteps: 0,
    totalSteps: 13,
    updatedAt: new Date().toISOString(),
  });
  await nextTick();
  assignmentNoticeEl.value?.scrollIntoView({
    behavior: "smooth",
    block: "nearest",
  });
  try {
    const result = await store.assignTeachers();
    store.setAssignmentProgress({
      status: "completed",
      stage: "completed",
      stageLabel: "分配完成",
      percent: 100,
      message: "监考分配完成，正在刷新结果...",
      completedSteps: 13,
      totalSteps: 13,
      updatedAt: new Date().toISOString(),
    });
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
        ? `，其他老师补位 ${result.fallbackPoolAssignments} 项`
        : "";
    const mainMessage = `${summary}：已分配 ${result.assignedCount} 项，未分配 ${result.unassignedCount} 项，${optimality}，耗时 ${formatSolveDuration(result.solveDurationMs)}${fallbackPart}。`;
    const unassignedPart =
      result.unassignedDetails.length > 0
        ? `\n未分配的任务：${result.unassignedDetails.join("、")}。`
        : "";
    await showAssignmentNotice(
      result.unassignedCount > 0 ? "warning" : "success",
      `${mainMessage}${unassignedPart}`,
    );
  } catch (error) {
    store.setAssignmentProgress(null);
    const message =
      store.viewState.errorMessage ||
      (error instanceof Error ? error.message : String(error)) ||
      "分配失败，请检查配置后重试。";
    await showAssignmentNotice("error", `分配失败：${message}`);
  }
}

async function exportInvigilationSchedule() {
  try {
    const result = await store.exportLatestInvigilationSchedule();
    await showAssignmentNotice("success", "监考表已导出，点击下方链接打开文件所在位置。", {
      linkPath: result.filePath,
      linkLabel: exportFileName(result.filePath),
    });
  } catch (error) {
    const message =
      store.viewState.errorMessage ||
      (error instanceof Error ? error.message : String(error)) ||
      "导出失败，请稍后重试。";
    await showAssignmentNotice("error", `导出失败：${message}`);
  }
}

async function openInvigilationExportFolder() {
  const target = assignmentNoticeLinkPath.value || store.viewState.lastInvigilationExportPath;
  if (!target) {
    return;
  }
  await revealInExplorer(target);
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
    store.setAssignmentProgress(event.payload);
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
.dialog-mask {
  position: fixed;
  inset: 0;
  background: var(--surface-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 600;
}
.dialog {
  width: 480px;
  max-width: calc(100vw - 32px);
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}
.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-lg);
}
.dialog-head h3 {
  margin: 0;
  font-size: var(--font-size-2xl);
  font-weight: 700;
}
.dialog-close {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: var(--radius-xs);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: var(--font-size-2xl);
}
.dialog-summary {
  margin: 0;
  color: var(--text-primary);
  font-size: var(--font-size-base);
  line-height: 1.55;
}
.dialog-details {
  margin: 0;
  padding-left: 18px;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  display: grid;
  gap: var(--space-xs);
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-md);
}

.panel {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: var(--space-2xl);
  isolation: isolate;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  min-height: 0;
}

.section-kicker {
  color: var(--text-tertiary);
  font-size: var(--font-size-xs);
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}


.grid-two {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-lg);
  align-items: start;
}

.top-grid {
  align-items: stretch;
}

.summary-grid-row {
  align-items: stretch;
  grid-template-columns: minmax(0, 1fr) minmax(300px, 420px);
}

@media (max-width: 1100px) {
  .summary-grid-row {
    grid-template-columns: 1fr;
  }
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
.allowance-card :deep(.body) {
  height: 100%;
}

.exam-count-card .card-stack,
.middle-manager-card .card-stack,
.allowance-card .card-stack {
  height: 100%;
}

.exam-count-card :deep(.config-card) {
  gap: var(--space-md);
}

.exam-count-card :deep(.body) {
  gap: var(--space-md);
  height: 100%;
}

.exam-count-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 40px;
  height: 100%;
  min-height: 120px;
  padding: var(--space-md) 0;
}

.count-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-width: 80px;
}

.count-number {
  font-size: 56px;
  font-weight: 700;
  line-height: 1;
  color: var(--text-primary);
  letter-spacing: -1px;
}

.count-unit {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-top: 8px;
}

.count-btn {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background-color: var(--surface-panel);
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.02);
  transition: all 0.2s ease;
}

.count-btn svg {
  transition: transform 0.2s;
}

.count-btn:hover:not(:disabled) {
  border-color: var(--accent-border-strong);
  background-color: color-mix(in srgb, var(--surface-panel) 82%, white);
  color: var(--text-primary);
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.count-btn:active:not(:disabled) {
  transform: translateY(0);
  background-color: var(--surface-elevated);
  box-shadow: none;
}

.count-btn:active:not(:disabled) svg {
  transform: scale(0.9);
}

.count-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  box-shadow: none;
}

.middle-manager-card :deep(.config-card) {
  gap: var(--space-md);
}

.middle-manager-card :deep(.body) {
  gap: var(--space-md);
}

.middle-manager-card .card-stack {
  gap: 28px;
  justify-content: space-between;
}

.study-card {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e5e7eb;
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.05),
    0 2px 4px -1px rgba(0, 0, 0, 0.03);
  overflow: hidden;
}

.study-card-header {
  padding: 24px 24px 16px;
}

.study-title-bar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.study-title-line {
  display: inline-block;
  width: 4px;
  height: 16px;
  background: var(--accent-primary);
  border-radius: 2px;
}

.study-card-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #111827;
}

.study-data-grid {
  display: flex;
  gap: 16px;
  padding: 0 24px 24px;
}

.study-data-item {
  flex: 1;
  background: #f9fafb;
  border: 1px solid #f3f4f6;
  border-radius: 8px;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.study-data-item.warning {
  background: #fffbeb;
  border-color: #fde68a;
}

.study-data-item.warning .study-data-label {
  color: #d97706;
}

.study-data-item.warning .study-data-value {
  color: #b45309;
}

.study-data-item.warning .study-data-unit {
  color: #d97706;
}

.study-data-label {
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 8px;
  font-weight: 500;
}

.study-data-value {
  font-size: 24px;
  font-weight: 700;
  color: #1f2937;
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.study-data-time {
  font-size: 18px;
  letter-spacing: 0.5px;
}

.study-data-unit {
  font-size: 14px;
  font-weight: 500;
  color: #9ca3af;
}

.study-card-footer {
  margin-top: auto;
  padding: 16px 24px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
  border-radius: 0 0 12px 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.study-status-text {
  font-size: 14px;
  color: #059669;
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  min-width: 0;
}

.study-status-text.pending {
  color: #d97706;
}

.study-status-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.study-btn-primary {
  flex-shrink: 0;
  background-color: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: background-color 0.2s, transform 0.1s;
}

.study-btn-primary:hover {
  background-color: #1d4ed8;
}

.study-btn-primary:active {
  transform: scale(0.98);
}

.study-btn-primary .material-symbols-rounded {
  font-size: 16px;
}

.exclude-card {
  position: relative;
  z-index: 80;
}

.schedule-card {
  width: 100%;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
  border: 1px solid #edf2f7;
  box-sizing: border-box;
  overflow: hidden;
}

.schedule-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 24px 32px;
  border-bottom: 1px solid #f1f5f9;
}

.schedule-header-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.schedule-title-bar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.schedule-title-line {
  display: inline-block;
  width: 4px;
  height: 16px;
  background: var(--accent-primary);
  border-radius: 2px;
}

.schedule-card-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1a202c;
}

.schedule-card-subtitle {
  margin: 0;
  font-size: 13px;
  color: #718096;
}

.schedule-card-subtitle strong {
  color: #e53e3e;
  font-weight: 600;
  margin: 0 4px;
}

.schedule-btn-primary {
  flex-shrink: 0;
  background-color: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: background-color 0.2s, transform 0.1s;
}

.schedule-btn-primary:hover {
  background-color: #1d4ed8;
}

.schedule-btn-primary:active {
  transform: scale(0.98);
}

.schedule-btn-primary svg {
  color: inherit;
}

.schedule-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl) var(--space-lg);
  background-color: #f8fafc;
  border-radius: 0 0 12px 12px;
}

.schedule-empty-icon {
  width: 64px;
  height: 64px;
  background-color: #ebf8ff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
}

.schedule-empty-icon svg {
  color: #3182ce;
}

.schedule-empty-title {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 600;
  color: #2d3748;
}

.schedule-empty-desc {
  margin: 0;
  font-size: 14px;
  color: #a0aec0;
}

.schedule-rule-list {
  padding: 20px 32px 24px;
}

.allowance-card {
  width: 100%;
  max-width: 100%;
  min-width: 0;
  justify-self: stretch;
  box-sizing: border-box;
}

.allowance-card :deep(.config-card) {
  gap: 0;
  min-width: 0;
  max-width: 100%;
}

.allowance-card :deep(.config-card) h3 {
  display: none;
}

.allowance-card :deep(.body) {
  display: block !important;
  flex-direction: unset !important;
  gap: 0 !important;
}

.allowance-title-bar {
  display: flex;
  align-items: center;
  margin-bottom: 20px;
}

.allowance-title-line {
  display: inline-block;
  width: 4px;
  height: 16px;
  background: var(--accent-primary);
  border-radius: 2px;
  margin-right: 10px;
}

.allowance-title-text {
  font-size: var(--font-size-title-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.allowance-items-container {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.allowance-item {
  min-width: 0;
  background: var(--surface-page-accent);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 14px 16px;
  transition: all 0.2s ease;
}

.allowance-item:hover {
  border-color: var(--border-strong);
  background: var(--surface-panel);
  box-shadow: var(--shadow-medium);
  transform: translateY(-1px);
}

.allowance-item-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  line-height: 1.35;
}

.allowance-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.allowance-dot-inside {
  background-color: var(--color-success);
}

.allowance-dot-outside {
  background-color: var(--color-warning);
}

.allowance-item-value {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
}

.allowance-value-input {
  font-size: clamp(22px, 5vw, 28px);
  font-weight: 700;
  color: var(--text-primary);
  border: none;
  background: transparent;
  padding: 0;
  width: auto;
  min-width: 2.5ch;
  max-width: calc(100% - 4.8em);
  flex: 0 1 auto;
  font-family: inherit;
}

.allowance-value-input::-webkit-outer-spin-button,
.allowance-value-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.allowance-value-input[type="number"] {
  -moz-appearance: textfield;
  appearance: textfield;
}

.allowance-value-input:focus {
  outline: none;
}

.allowance-value-unit {
  font-size: var(--font-size-body);
  font-weight: 500;
  color: var(--text-tertiary);
  white-space: nowrap;
}

.display-field,
.summary-chip,
.summary-box {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--surface-panel) 84%, white);
  padding: var(--space-md) var(--space-md);
  box-shadow: var(--shadow-soft);
}

.display-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
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
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}
.warning-text {
  color: var(--color-warning);
}

.pending-text {
  color: color-mix(in srgb, var(--color-warning) 78%, var(--text-primary));
}

.field-value-row {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  line-height: 1.2;
}

.field-value-text,
.summary-value {
  color: var(--color-text);
  font-size: var(--font-size-xl);
  font-weight: 600;
}

.value-input {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-text);
  font-size: var(--font-size-xl);
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
  width: 34px;
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
  padding: 0 var(--space-md);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-panel);
}

.segment-wrap {
  display: flex;
  background-color: #f1f5f9;
  padding: 4px;
  border-radius: 8px;
}

.segment-btn {
  flex: 1;
  padding: 10px 0;
  text-align: center;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.segment-btn:hover:not(.active) {
  color: var(--text-primary);
}

.segment-btn.active {
  background-color: var(--surface-panel);
  color: var(--accent-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06);
  font-weight: 600;
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
  gap: var(--space-md);
}

.drawer-trigger,
.page-btn,
.action-btn,
.toolbar-btn {
  white-space: nowrap;
}

.info-pill,
.exception-pill,
.pending-pill,
.warning-pill,
.danger-pill,
.status-badge,
.subject-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 34px;
  padding: 0 var(--space-md);
  border-radius: var(--radius-pill);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.info-pill {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.exception-pill {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  border: 1px solid var(--accent-border-soft);
}

.exception-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background-color: #eff6ff;
  color: #3b82f6;
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  border: 1px solid #bfdbfe;
}

.exception-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background-color: #ffffff;
  color: #475569;
  border: 1px solid #cbd5e1;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.exception-btn:hover {
  border-color: #94a3b8;
  color: #0f172a;
  background-color: #f8fafc;
}

.exception-btn:active {
  transform: scale(0.98);
}

.pending-pill {
  background: color-mix(in srgb, var(--color-warning-soft) 74%, white);
  color: color-mix(in srgb, var(--color-warning) 76%, var(--text-primary));
  border: 1px solid rgba(154, 111, 43, 0.18);
}

.warning-pill {
  background: var(--color-warning-soft);
  color: var(--color-warning);
  border: 1px solid rgba(154, 111, 43, 0.28);
}

.primary-pill {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  border: 1px solid var(--accent-border-soft);
}

.danger-pill {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}

.required-mark {
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
  gap: var(--space-md);
}

.exclude-toolbar {
  flex-direction: row;
  flex-wrap: wrap;
  position: relative;
  z-index: 90;
}

.custom-rule-drawer {
  width: 560px;
}

.custom-rule-panel {
  padding: var(--space-xl);
  overflow-y: auto;
}

.custom-rule-detail-drawer {
  width: 520px;
}

.custom-rule-overview {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.custom-rule-overview-tags,
.compact-rule-meta,
.detail-chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.form-group-step {
  animation: step-in 0.25s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
  transform-origin: top;
}

@keyframes step-in {
  from {
    opacity: 0;
    transform: translateY(-8px) scaleY(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scaleY(1);
  }
}

.form-label {
  display: block;
}

.full-width {
  width: 100%;
}

.custom-rule-row {
  width: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-md);
}

.custom-rule-main {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.custom-rule-item {
  flex-direction: column;
  align-items: flex-start;
}

.custom-rule-teacher {
  font-size: var(--font-size-base);
}

.custom-rule-summary {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.compact-rule-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.compact-rule-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
  padding: var(--space-lg) 16px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
  background: color-mix(in srgb, var(--surface-panel) 84%, white);
}

.compact-rule-main {
  display: flex;
  min-width: 0;
}

.compact-rule-header {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.compact-rule-task {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.compact-rule-actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.detail-summary-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-md);
}

.detail-chip {
  min-height: 34px;
  align-items: center;
}

.rule-tag {
  display: inline-flex;
  align-items: center;
  min-height: 30px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-pill);
  font-size: var(--font-size-xs);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  color: var(--text-secondary);
}

.option-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-md);
}

.selection-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-md);
  padding: var(--space-lg) 16px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--surface-panel) 82%, var(--accent-surface-soft));
}

.selection-toolbar-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  min-width: 0;
}

.selection-toolbar-copy strong {
  color: var(--color-text);
  font-size: var(--font-size-base);
}

.selection-toolbar-copy span {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.5;
}

.selection-toolbar-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.toolbar-toggle-btn {
  min-height: 36px;
  padding: 0 14px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--surface-panel) 78%, white);
  color: var(--accent-text);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
}

.toolbar-toggle-btn:hover:not(:disabled) {
  border-color: var(--accent-border-strong);
  background: color-mix(in srgb, var(--accent-surface-soft) 72%, white);
}

.toolbar-toggle-btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.toolbar-selection-state {
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  white-space: nowrap;
}

.toolbar-inline-btn {
  min-height: 32px;
  padding: 0 var(--space-xs);
  border: 0;
  background: transparent;
  color: var(--accent-text);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
}

.toolbar-inline-btn:hover {
  color: var(--accent-text-strong, var(--accent-text));
  text-decoration: underline;
}

.selection-search {
  min-width: 220px;
  min-height: 38px;
  padding: 0 var(--space-md);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-panel);
  color: var(--color-text);
}

.selection-search:focus {
  outline: none;
  border-color: var(--accent-border-strong);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-surface-soft) 72%, transparent);
}

.selection-list {
  max-height: 320px;
  overflow: auto;
  padding-right: 4px;
}

.compact-option-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.check-option {
  display: flex;
  align-items: flex-start;
  gap: var(--space-md);
  min-height: 48px;
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
}

.single-option.active {
  border-color: var(--accent-border-strong);
  background: color-mix(in srgb, var(--surface-panel) 82%, var(--accent-surface-soft));
}

.check-option input {
  margin-top: 2px;
}

.compact-option {
  min-height: 64px;
  padding-top: var(--space-sm);
  padding-bottom: var(--space-sm);
}

.target-option {
  align-items: flex-start;
}

.target-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  min-width: 0;
  line-height: 1.35;
}

.target-copy > span {
  display: block;
  color: var(--color-text);
  font-size: var(--font-size-base);
}

.target-copy small {
  display: block;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.45;
  white-space: normal;
}

.time-scope-copy {
  gap: var(--space-sm);
}

.time-scope-subject {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text);
}

.time-scope-datetime {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.target-option-copy {
  gap: var(--space-xs);
}

.target-option-label {
  font-size: var(--font-size-lg);
  font-weight: 600;
  color: var(--color-text);
}

.target-option-subtitle {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.45;
  white-space: normal;
}

.scope-preview,
.custom-rule-summary-box {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
}

.custom-rule-footer {
  padding: var(--space-lg) var(--space-xl);
  border-top: 1px solid var(--border-default);
  background: var(--surface-default);
}

.drawer-error {
  color: var(--color-danger);
  font-size: var(--font-size-xs);
  margin: 0;
}

.empty-box {
  min-height: 44px;
  display: flex;
  align-items: center;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  border: 1px dashed var(--border-default);
  background: color-mix(in srgb, var(--surface-panel) 82%, white);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

@media (max-width: 900px) {
  .custom-rule-drawer {
    width: min(100vw, 560px);
  }

  .custom-rule-detail-drawer {
    width: min(100vw, 520px);
  }

  .option-grid {
    grid-template-columns: 1fr;
  }

  .compact-rule-item,
  .compact-rule-actions {
    flex-direction: column;
    align-items: flex-start;
  }

  .detail-summary-grid {
    grid-template-columns: 1fr;
  }

  .selection-toolbar {
    flex-direction: column;
  }

  .selection-toolbar-actions {
    width: 100%;
    justify-content: stretch;
  }

  .selection-search {
    width: 100%;
    min-width: 0;
  }
}

.empty-box-guide {
  min-height: 72px;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  border-style: solid;
  background: color-mix(in srgb, var(--surface-elevated) 78%, white);
}

.empty-box-icon {
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  font-size: var(--font-size-2xl);
  flex-shrink: 0;
}

.empty-box-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  min-width: 0;
}

.empty-box-copy strong {
  color: var(--text-primary);
  font-size: var(--font-size-base);
}

.empty-box-copy span {
  color: var(--text-secondary);
  line-height: 1.5;
}

.error-box {
  border-color: rgba(216, 80, 80, 0.24);
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.summary-grid,
.subsidy-row,
.time-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-md);
}

.schedule-row {
  display: grid;
  grid-template-columns: 1.1fr 1fr 1fr;
  gap: var(--space-md);
}

.summary-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-sm);
}

.subsidy-row {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  justify-content: start;
  gap: var(--space-md);
}

.summary-chip.warning,
.subject-badge.empty,
.status-badge.pending {
  background: var(--color-warning-soft);
  border-color: rgba(154, 111, 43, 0.28);
  color: var(--color-warning);
}

.summary-chip.warning {
  border: 1px solid rgba(154, 111, 43, 0.28);
}

.summary-chip.pending-chip {
  border: 1px solid rgba(154, 111, 43, 0.18);
  background: color-mix(in srgb, var(--color-warning-soft) 72%, white);
}

.summary-chip {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.time-summary-chip {
  min-width: 0;
  padding-right: 22px;
}

.time-summary-value {
  display: inline-flex;
  align-items: baseline;
  gap: var(--space-md);
  white-space: nowrap;
  font-size: var(--font-size-lg);
  letter-spacing: -0.01em;
}

.time-summary-date,
.time-summary-range {
  display: inline-block;
}

.summary-chip2 {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.fluent-combo {
  position: relative;
  width: 220px;
}

.fluent-combo.open {
  z-index: 120;
}

.fluent-input,
.toolbar-filter,
.search-bar {
  min-height: 42px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
}

.fluent-input {
  width: 100%;
  padding: 0 var(--space-md);
  font-size: var(--font-size-base);
}

.combo-icon {
  position: absolute;
  right: 10px;
  top: 11px;
  font-size: var(--font-size-xl);
  color: var(--text-secondary);
}

.select-field {
  padding-right: 32px;
}

.fluent-menu,
.subject-menu {
  position: absolute;
  padding: var(--space-xs);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--surface-input-strong);
  box-shadow: var(--shadow-strong);
  z-index: 140;
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
  border-radius: var(--radius-xs);
  background: transparent;
  text-align: left;
  padding: var(--space-sm) var(--space-md);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
}

.fluent-option.selected,
.fluent-option:hover,
.subject-menu-item.active,
.subject-menu-item:hover {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.menu-empty {
  padding: var(--space-sm) var(--space-md);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.class-table-head,
.class-table-row {
  display: grid;
  grid-template-columns: 44px 1.5fr 1fr 1fr 1fr;
  gap: var(--space-md);
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-md);
}

.class-table-head {
  background: color-mix(in srgb, var(--surface-elevated) 84%, white);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.class-table-row {
  border: 1px solid var(--border-default);
  background: color-mix(in srgb, var(--surface-panel) 86%, white);
}

.class-table-row.selected {
  border-color: var(--accent-border-strong);
  background: var(--surface-elevated);
}

.check-cell {
  display: inline-flex;
  justify-content: center;
}

.subject-badge {
  justify-self: start;
  border: 1px solid var(--border-default);
  background: var(--surface-elevated);
  color: var(--text-primary);
}

.status-badge.done {
  background: var(--color-success-soft);
  color: var(--color-success);
}

.toolbar-filter {
  position: relative;
  width: fit-content;
  min-width: 0;
  flex: 0 0 auto;
}

.grade-filter-select {
  width: 140px;
}

.teacher-select-full {
  width: 100%;
}

.search-bar input {
  border: 0;
  background: transparent;
  font-size: var(--font-size-base);
}

.toolbar-btn {
  min-height: 44px;
  padding: 0 var(--space-lg);
  border-radius: var(--radius-md);
  border: 1px solid var(--accent-border-soft);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  font-size: var(--font-size-base);
  font-weight: 700;
  cursor: pointer;
}

.toolbar-btn:disabled,
.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-chip {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  border: 1px solid var(--accent-border-soft);
  background: var(--surface-panel);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.page-btn {
  min-width: 44px;
  min-height: 40px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--accent-border-soft);
  background: var(--surface-panel);
  color: var(--text-secondary);
  font-size: var(--font-size-base);
  font-weight: 700;
  cursor: pointer;
}

.page-btn.active {
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.selection-strip {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-sm);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.14);
  z-index: 998;
}

.config-drawer {
  position: fixed;
  top: var(--space-xl);
  right: var(--space-xl);
  max-height: calc(100vh - 48px);
  overflow-y: auto;
  width: min(560px, calc(100vw - 24px));
  padding: var(--space-2xl);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  box-shadow: var(--shadow-strong);
  z-index: 999;
}

.soft-panel {
  padding: var(--space-xl);
  border-radius: var(--radius-xl, 24px);
  border: 1px solid var(--border-default);
  background: color-mix(in srgb, var(--surface-panel) 84%, white);
}

.class-config-section {
  padding-top: var(--space-sm);
  gap: var(--space-md);
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
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
}

.drawer-note {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.35;
}

.summary-card :deep(h3) {
  font-size: var(--font-size-xl);
}

.summary-card :deep(p) {
  font-size: var(--font-size-xs);
  color: var(--text-tertiary);
}

.middle-footer {
  padding-top: var(--space-xs);
}

.self-study-note {
  width: 220px;
  font-size: var(--font-size-xs);
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
  color: var(--color-danger);
}

.text-btn {
  color: var(--accent-primary);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.drawer-close {
  width: 38px;
  height: 38px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.search-bar {
  padding: 0 var(--space-md);
}

.middle-picker {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.middle-search {
  gap: var(--space-sm);
}

.middle-search input {
  flex: 1;
  min-width: 0;
}

.search-icon {
  font-size: var(--font-size-xl);
  color: var(--text-tertiary);
}

.candidate-item {
  min-height: 42px;
  justify-content: space-between;
  padding: 0 var(--space-lg);
  border-radius: var(--radius-sm);
  border: 1px dashed var(--accent-border-soft);
  background: var(--surface-elevated);
}

.candidate-action {
  color: var(--accent-primary);
  font-size: var(--font-size-sm);
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
  gap: var(--space-xs);
}

.solver-summary {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  line-height: 1.4;
}

.action-row {
  justify-content: flex-start;
}

.assignment-notice {
  display: flex;
  align-items: flex-start;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  font-size: var(--font-size-base);
  line-height: 1.6;
  color: var(--color-text);
}

.assignment-notice.inline {
  flex: 0 1 340px;
  max-width: 340px;
  min-width: 0;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  line-height: 1.45;
}

.assignment-notice-icon {
  font-size: var(--font-size-xl);
  line-height: 1.2;
  color: var(--accent-primary);
}

.assignment-notice-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.assignment-notice-text {
  min-width: 0;
  overflow-wrap: anywhere;
  white-space: pre-line;
}

.assignment-notice-link {
  align-self: flex-start;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--accent-primary);
  font: inherit;
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}

.assignment-progress {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.assignment-progress-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
}

.assignment-progress-track {
  width: 100%;
  height: 6px;
  border-radius: var(--radius-pill);
  background: rgba(var(--accent-rgb), 0.12);
  overflow: hidden;
}

.assignment-progress-bar {
  height: 100%;
  border-radius: inherit;
  background: var(--accent-progress-fill);
  transition: width var(--transition-base);
}

.title-stack {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.middle-toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.middle-primary-btn {
  padding: var(--space-sm) var(--space-lg);
  min-height: auto;
  border-radius: var(--radius-sm);
}

.middle-filter-btn {
  min-height: auto;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--surface-table-content);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 600;
  cursor: pointer;
}

.middle-filter-btn.active {
  border-color: var(--accent-border-strong);
  background: rgba(var(--accent-rgb), 0.12);
  color: var(--accent-primary);
}

.middle-exception-item {
  padding: var(--space-md);
}

.middle-pagination {
  align-items: center;
  padding-top: 2px;
}

.middle-person {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.middle-subtext {
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  font-weight: 500;
}

.middle-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.middle-status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-pill);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.middle-status-pill.off {
  background: var(--color-danger-soft);
  border: 1px solid rgba(182, 68, 68, 0.2);
  color: var(--color-danger);
}

.middle-status-pill.on {
  background: var(--color-success-soft);
  border: 1px solid rgba(68, 113, 92, 0.24);
  color: var(--color-success);
}

.subject-menu {
  position: fixed;
  width: 168px;
  max-height: calc(5 * 42px + 16px);
  overflow-y: auto;
  border-radius: var(--radius-md);
  /* 需要高于抽屉(999)和遮罩(998)，避免在抽屉中被遮挡。 */
  z-index: 1200;
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

</style>

