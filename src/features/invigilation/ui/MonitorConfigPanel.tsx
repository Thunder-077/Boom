import { useEffect, useMemo, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ClassConfigRow } from "../../../entities/class-config/model";
import type {
  ExamStaffAssignmentProgress,
  InvigilationConfig,
  InvigilationCustomRule,
  InvigilationRuleTaskScopeType,
  InvigilationRuleTargetOption,
  InvigilationRuleTimeScopeType,
} from "../../../entities/exam-plan/model";
import { Subject } from "../../../entities/score/model";
import { revealInExplorer } from "../../../shared/utils/appLog";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import ConfigCard from "../../../widgets/common/ConfigCard";
import FluentSelect from "../../../widgets/common/FluentSelect";
import Pagination from "../../../widgets/common/composite/Pagination";
import { classConfigService } from "../../classes/service";
import { examAllocationService } from "../../dashboard/service";
import { useReactExamAllocationStore } from "../../dashboard/store";
import {
  AlertCircle,
  AlertTriangle,
  Check,
  CircleCheck,
  Info,
  List,
  Loader2,
  Minus,
  Plus,
  Search,
  Settings,
  SlidersHorizontal,
  Trash2,
  Users,
  X,
} from "lucide-react";

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

interface AssignmentNotice {
  type: "success" | "warning" | "error";
  text: string;
  linkPath?: string;
  linkLabel?: string;
}

interface DialogState {
  visible: boolean;
  kind: "confirm" | "alert";
  title: string;
  summary: string;
  details: string[];
  confirmText: string;
  cancelText: string;
}

interface DraftRuleState {
  actionType: "exclude" | "require" | "";
  teacherId: number | "";
  timeScopeType: InvigilationRuleTimeScopeType;
  timeScopeIds: number[];
  taskScopeType: InvigilationRuleTaskScopeType;
  targetScopeType: "all" | "selected_targets";
  targetIds: string[];
}

const gradeRankMap: Record<string, number> = { 高一: 1, 高二: 2, 高三: 3 };
const staffAssignmentProgressEvent = "invigilation_staff_assignment_progress";
const pageSize = 4;
const middleManagerPageSize = 3;

const defaultDialogState: DialogState = {
  visible: false,
  kind: "confirm",
  title: "",
  summary: "",
  details: [],
  confirmText: "确认",
  cancelText: "取消",
};

const subjectLabelMap: Record<Subject, string> = {
  [Subject.Chinese]: "语文",
  [Subject.Math]: "数学",
  [Subject.English]: "英语",
  [Subject.Physics]: "物理",
  [Subject.Chemistry]: "化学",
  [Subject.Biology]: "生物",
  [Subject.Politics]: "政治",
  [Subject.History]: "历史",
  [Subject.Geography]: "地理",
  [Subject.Russian]: "俄语",
  [Subject.Japanese]: "日语",
};

const selectableSubjects: Subject[] = [
  Subject.Chinese,
  Subject.Math,
  Subject.English,
  Subject.Russian,
  Subject.Japanese,
  Subject.History,
  Subject.Geography,
  Subject.Biology,
  Subject.Politics,
  Subject.Physics,
  Subject.Chemistry,
];

function createEmptyDraftRule(): DraftRuleState {
  return {
    actionType: "",
    teacherId: "",
    timeScopeType: "exam_session",
    timeScopeIds: [],
    taskScopeType: "exam_room",
    targetScopeType: "all",
    targetIds: [],
  };
}

function mapClassSortNumber(className: string) {
  const match = className.match(/(\d+)/g);
  return match && match.length > 0 ? Number(match[match.length - 1]) : Number.POSITIVE_INFINITY;
}

function compareTeachingClasses(a: SelfStudyClassRow, b: SelfStudyClassRow) {
  const gradeDiff = (gradeRankMap[a.gradeName] ?? 99) - (gradeRankMap[b.gradeName] ?? 99);
  if (gradeDiff !== 0) {
    return gradeDiff;
  }
  const classDiff = mapClassSortNumber(a.className) - mapClassSortNumber(b.className);
  if (classDiff !== 0) {
    return classDiff;
  }
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

function exportFileName(path: string) {
  const matched = path.match(/[^\\/]+$/);
  return matched?.[0] ?? path;
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
  }
  return `${startAt} - ${endAt}`;
}

function getPeriodLabel(hour: number, minute: number) {
  if (hour < 12 || (hour === 12 && minute === 0)) {
    return "上午";
  }
  if (hour < 18 || (hour === 18 && minute < 30)) {
    return "下午";
  }
  return "晚上";
}

function normalizeRuleTimeScopeSubject(subjectLabel: string) {
  if (["英语", "俄语", "日语"].includes(subjectLabel)) {
    return "外语";
  }
  return subjectLabel;
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

function buildGroupedRuleTimeScopeLabel(options: Array<{ label: string; startAt: string; endAt: string }>) {
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

function formatTargetOptionSubtitle(subtitle: string) {
  const timePattern = /(\d{2}:\d{2})-(\d{2}:\d{2})/;
  const match = subtitle.match(timePattern);
  if (!match) {
    return subtitle;
  }
  const startTime = match[1];
  const endTime = match[2];
  const [startHour, startMinute] = startTime.split(":").map(Number);
  const [endHour, endMinute] = endTime.split(":").map(Number);
  const startPeriod = getPeriodLabel(startHour, startMinute);
  const endPeriod = getPeriodLabel(endHour, endMinute);
  const replacement = startPeriod === endPeriod
    ? `${startPeriod}${startTime} — ${endTime}`
    : `${startPeriod}${startTime} — ${endPeriod}${endTime}`;
  return subtitle.replace(timePattern, replacement);
}

function cloneRule(rule: InvigilationCustomRule): InvigilationCustomRule {
  return {
    ...rule,
    timeScopeIds: [...rule.timeScopeIds],
    timeScopeLabels: [...rule.timeScopeLabels],
    targetIds: [...rule.targetIds],
    targetLabels: [...rule.targetLabels],
  };
}

export default function MonitorConfigPanel() {
  const {
    state,
    loadAll,
    assignTeachers,
    saveInvigilationConfig,
    saveCustomRules,
    saveSelfStudyClassSubjects,
    setAssignmentProgress,
    exportLatestInvigilationSchedule,
  } = useReactExamAllocationStore();

  const [defaultExamRoomRequiredCount, setDefaultExamRoomRequiredCount] = useState(1);
  const [indoorAllowancePerMinute, setIndoorAllowancePerMinute] = useState(0.5);
  const [outdoorAllowancePerMinute, setOutdoorAllowancePerMinute] = useState(0.3);
  const [selfStudyMonthDay, setSelfStudyMonthDay] = useState(new Date().toISOString().slice(5, 10));
  const [selfStudyStartTime, setSelfStudyStartTime] = useState("12:10");
  const [selfStudyEndTime, setSelfStudyEndTime] = useState("13:40");
  const [selfStudyValidationError, setSelfStudyValidationError] = useState("");
  const [selfStudyDrawerOpen, setSelfStudyDrawerOpen] = useState(false);
  const [middleManagerDrawerOpen, setMiddleManagerDrawerOpen] = useState(false);
  const [customRuleDrawerOpen, setCustomRuleDrawerOpen] = useState(false);
  const [customRuleDetailOpen, setCustomRuleDetailOpen] = useState(false);
  const [selfStudyLoading, setSelfStudyLoading] = useState(false);
  const [selfStudyLoadError, setSelfStudyLoadError] = useState("");
  const [gradeFilter, setGradeFilter] = useState("all");
  const [availableGrades, setAvailableGrades] = useState<string[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedClassIds, setSelectedClassIds] = useState<Set<number>>(new Set());
  const [bulkMenuOpen, setBulkMenuOpen] = useState(false);
  const [middleManagerDefaultEnabledDraft, setMiddleManagerDefaultEnabledDraft] = useState(false);
  const [middleManagerExceptionTeacherIdsDraft, setMiddleManagerExceptionTeacherIdsDraft] = useState<number[]>([]);
  const [middleManagerKeyword, setMiddleManagerKeyword] = useState("");
  const [middleManagerPage, setMiddleManagerPage] = useState(1);
  const [showMiddleManagerPicker, setShowMiddleManagerPicker] = useState(false);
  const [showOnlyMiddleManagerExceptions, setShowOnlyMiddleManagerExceptions] = useState(false);
  const [subjectMenu, setSubjectMenu] = useState({
    open: false,
    top: 0,
    left: 0,
    rowId: null as number | null,
    mode: "single" as "single" | "bulk",
  });
  const [selfStudyClasses, setSelfStudyClasses] = useState<SelfStudyClassRow[]>([]);
  const [assignmentNotice, setAssignmentNotice] = useState<AssignmentNotice | null>(null);
  const [dialogState, setDialogState] = useState<DialogState>(defaultDialogState);
  const [draftRuleError, setDraftRuleError] = useState("");
  const [draftRule, setDraftRule] = useState<DraftRuleState>(createEmptyDraftRule);
  const [selectedCustomRule, setSelectedCustomRule] = useState<InvigilationCustomRule | null>(null);
  const assignmentNoticeEl = useRef<HTMLDivElement | null>(null);
  const dialogResolverRef = useRef<((value: boolean) => void) | null>(null);
  const mountedRef = useRef(true);
  const currentPageSelectAllRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    setDefaultExamRoomRequiredCount(state.invigilationConfig.defaultExamRoomRequiredCount);
    setIndoorAllowancePerMinute(Number(state.invigilationConfig.indoorAllowancePerMinute || 0));
    setOutdoorAllowancePerMinute(Number(state.invigilationConfig.outdoorAllowancePerMinute || 0));
    setSelfStudyMonthDay(extractMonthDay(state.invigilationConfig.selfStudyDate));
    setSelfStudyStartTime(state.invigilationConfig.selfStudyStartTime);
    setSelfStudyEndTime(state.invigilationConfig.selfStudyEndTime);
    if (!middleManagerDrawerOpen) {
      setMiddleManagerDefaultEnabledDraft(state.invigilationConfig.middleManagerDefaultEnabled);
      setMiddleManagerExceptionTeacherIdsDraft([...state.invigilationConfig.middleManagerExceptionTeacherIds]);
    }
  }, [state.invigilationConfig, middleManagerDrawerOpen]);

  useEffect(() => {
    setSelfStudyClasses((current) =>
      current.map((item) => {
        const persisted = state.selfStudyClassSubjects.find((subjectItem) => subjectItem.classId === item.id);
        return {
          ...item,
          subject: persisted?.subject ?? null,
        };
      }),
    );
  }, [state.selfStudyClassSubjects]);

  useEffect(() => {
    setSelfStudyValidationError("");
  }, [selfStudyMonthDay, selfStudyStartTime, selfStudyEndTime]);

  const middleManagerDefaultEnabled = state.invigilationConfig.middleManagerDefaultEnabled;
  const middleManagerExceptionCount = state.invigilationConfig.middleManagerExceptionTeacherIds.length;
  const teacherSelectOptions = useMemo(
    () => [{ label: "选择教师", value: "" as const }].concat(state.teachers.map((item) => ({ label: item.teacherName, value: item.id }))),
    [state.teachers],
  );
  const middleManagerTeachers = useMemo(
    () => [...state.teachers].filter((item) => item.isMiddleManager).sort((a, b) => a.teacherName.localeCompare(b.teacherName, "zh-CN")),
    [state.teachers],
  );
  const filteredClasses = useMemo(
    () => (gradeFilter === "all" ? selfStudyClasses : selfStudyClasses.filter((item) => item.gradeName === gradeFilter)),
    [gradeFilter, selfStudyClasses],
  );
  const totalPages = useMemo(() => Math.max(1, Math.ceil(filteredClasses.length / pageSize)), [filteredClasses.length]);
  const pagedClasses = useMemo(
    () => filteredClasses.slice((currentPage - 1) * pageSize, currentPage * pageSize),
    [currentPage, filteredClasses],
  );
  const configuredClassCount = useMemo(() => selfStudyClasses.filter((item) => !!item.subject).length, [selfStudyClasses]);
  const pendingClassCount = selfStudyClasses.length - configuredClassCount;
  const selectedClassCount = selectedClassIds.size;
  const allCurrentPageSelected = pagedClasses.length > 0 && pagedClasses.every((item) => selectedClassIds.has(item.id));
  const indeterminateCurrentPageSelected = (() => {
    const count = pagedClasses.filter((item) => selectedClassIds.has(item.id)).length;
    return count > 0 && count < pagedClasses.length;
  })();
  const inferredSelfStudyYear = useMemo(() => {
    const firstSessionStart = state.sessionTimes.find((item) => item.startAt)?.startAt;
    if (firstSessionStart && /^\d{4}-\d{2}-\d{2}/.test(firstSessionStart)) {
      return firstSessionStart.slice(0, 4);
    }
    return String(new Date().getFullYear());
  }, [state.sessionTimes]);
  const normalizedSelfStudyDate = useMemo(() => {
    const value = selfStudyMonthDay.trim();
    if (!/^\d{2}-\d{2}$/.test(value)) {
      return "";
    }
    return `${inferredSelfStudyYear}-${value}`;
  }, [inferredSelfStudyYear, selfStudyMonthDay]);
  const selfStudyScopeText = state.sessionTimes.length > 0
    ? `适用范围：本次考试第 ${state.sessionTimes.length} 场结束后`
    : "适用范围：全员自习开始与结束时间默认在同一天。";
  const selfStudySummaryText = useMemo(() => {
    if (pendingClassCount === 0) {
      return "所有班级已完成科目配置。";
    }
    const pending = selfStudyClasses.filter((item) => !item.subject).map((item) => item.className);
    return `待补充：${pending.slice(0, 2).join("、")}${pending.length > 2 ? " 等" : ""}`;
  }, [pendingClassCount, selfStudyClasses]);
  const staffSolverSummary = useMemo(() => {
    const overview = state.staffOverview;
    if (!overview.generatedAt) {
      return "";
    }
    const statusLabel =
      overview.optimalityStatus === "optimal"
        ? "已证明最优"
        : overview.optimalityStatus === "feasible"
          ? "当前可行解"
          : overview.optimalityStatus === "infeasible"
            ? "模型不可行"
            : "求解失败";
    const fallbackSummary = overview.fallbackPoolAssignments > 0 ? `，其他老师补位 ${overview.fallbackPoolAssignments} 项` : "";
    return `CP-SAT，${statusLabel}，耗时 ${formatSolveDuration(overview.solveDurationMs)}${fallbackSummary}`;
  }, [state.staffOverview]);
  const filteredMiddleManagerTeachers = useMemo(() => {
    const keyword = middleManagerKeyword.trim();
    return middleManagerTeachers.filter((item) => {
      const matchedKeyword = keyword ? item.teacherName.includes(keyword) : true;
      const matchedException = showOnlyMiddleManagerExceptions ? middleManagerExceptionTeacherIdsDraft.includes(item.id) : true;
      return matchedKeyword && matchedException;
    });
  }, [middleManagerExceptionTeacherIdsDraft, middleManagerKeyword, middleManagerTeachers, showOnlyMiddleManagerExceptions]);
  const pagedMiddleManagerTeachers = useMemo(
    () => filteredMiddleManagerTeachers.slice((middleManagerPage - 1) * middleManagerPageSize, middleManagerPage * middleManagerPageSize),
    [filteredMiddleManagerTeachers, middleManagerPage],
  );
  const subjectMenuSelectedSubject = useMemo(
    () => (subjectMenu.open && subjectMenu.mode === "single" && subjectMenu.rowId !== null
      ? selfStudyClasses.find((item) => item.id === subjectMenu.rowId)?.subject ?? null
      : null),
    [selfStudyClasses, subjectMenu],
  );
  const isAssignmentProgressVisible = Boolean(state.assigning);
  const persistedInvigilationExportNotice = useMemo<AssignmentNotice | null>(() => {
    if (!state.lastInvigilationExportPath) {
      return null;
    }
    return {
      type: "success",
      text: "监考表已导出，点击下方链接打开文件所在位置。",
      linkPath: state.lastInvigilationExportPath,
      linkLabel: exportFileName(state.lastInvigilationExportPath),
    };
  }, [state.lastInvigilationExportPath]);
  const displayedAssignmentNotice = assignmentNotice ?? persistedInvigilationExportNotice;
  const assignmentProgress = state.assignmentProgress;
  const assignmentNoticeIcon = isAssignmentProgressVisible
    ? <Loader2 size={18} className="assignment-notice-icon" />
    : displayedAssignmentNotice?.type === "success"
      ? <CircleCheck size={18} className="assignment-notice-icon" />
      : displayedAssignmentNotice?.type === "warning"
        ? <AlertTriangle size={18} className="assignment-notice-icon" />
        : <Info size={18} className="assignment-notice-icon" />;
  const assignmentNoticeText = isAssignmentProgressVisible
    ? assignmentProgress?.message || "正在准备监考分配..."
    : displayedAssignmentNotice?.text || "";
  const assignmentNoticeLinkPath = displayedAssignmentNotice?.linkPath || "";
  const assignmentNoticeLinkLabel = displayedAssignmentNotice?.linkLabel || "";
  const activeDrawer = selfStudyDrawerOpen
    ? "selfStudy"
    : middleManagerDrawerOpen
      ? "middleManager"
      : customRuleDrawerOpen
        ? "customRule"
        : customRuleDetailOpen
          ? "customRuleDetail"
          : null;

  const groupedExamSessionRuleOptions = useMemo<DraftRuleTimeScopeOption[]>(() => {
    // 考试时段按开始/结束时间合并，保证同时间的多年级同科目合并展示。
    const grouped = new Map<string, DraftRuleTimeScopeOption>();
    for (const option of state.customRuleOptions.examSessionOptions) {
      const key = `${option.startAt}__${option.endAt}`;
      const existing = grouped.get(key);
      if (existing) {
        existing.sessionIds.push(option.id);
        existing.label = buildGroupedRuleTimeScopeLabel(
          existing.sessionIds
            .map((sessionId) => state.customRuleOptions.examSessionOptions.find((item) => item.id === sessionId))
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
  }, [state.customRuleOptions.examSessionOptions]);
  const fullSelfStudyRuleLabel = state.customRuleOptions.fullSelfStudyOption?.label || "全员自习时段暂未配置";
  const availableTaskScopeOptions = useMemo(() => {
    if (draftRule.timeScopeType === "full_self_study") {
      return [{ label: "全员自习看班", value: "full_self_study" as InvigilationRuleTaskScopeType }];
    }
    return [
      { label: "考试任务", value: "exam_room" as InvigilationRuleTaskScopeType },
      { label: "考试期间自习看班", value: "exam_linked_self_study" as InvigilationRuleTaskScopeType },
      { label: "流动监考", value: "floor_rover" as InvigilationRuleTaskScopeType },
    ];
  }, [draftRule.timeScopeType]);
  const availableRuleTargetOptions = useMemo(
    () =>
      state.customRuleOptions.targetOptions.filter((option) => {
        if (option.taskScopeType !== draftRule.taskScopeType) {
          return false;
        }
        if (option.timeScopeType !== draftRule.timeScopeType) {
          return false;
        }
        if (draftRule.timeScopeType === "exam_session") {
          return draftRule.timeScopeIds.includes(option.timeScopeId || -1);
        }
        return true;
      }),
    [draftRule.taskScopeType, draftRule.timeScopeIds, draftRule.timeScopeType, state.customRuleOptions.targetOptions],
  );
  const selectedRuleTargetOptions = useMemo(
    () => availableRuleTargetOptions.filter((option) => draftRule.targetIds.includes(option.id)),
    [availableRuleTargetOptions, draftRule.targetIds],
  );
  const selectedRuleTeacherName = state.teachers.find((item) => item.id === draftRule.teacherId)?.teacherName || "";
  const selectedRuleTimeLabels = useMemo(() => {
    if (draftRule.timeScopeType === "full_self_study") {
      return state.customRuleOptions.fullSelfStudyOption ? [state.customRuleOptions.fullSelfStudyOption.label] : [];
    }
    return groupedExamSessionRuleOptions
      .filter((option) => option.sessionIds.every((sessionId) => draftRule.timeScopeIds.includes(sessionId)))
      .map((option) => option.label);
  }, [draftRule.timeScopeIds, draftRule.timeScopeType, groupedExamSessionRuleOptions, state.customRuleOptions.fullSelfStudyOption]);
  const allRuleTimeScopesSelected = groupedExamSessionRuleOptions.length > 0
    && groupedExamSessionRuleOptions.every((option) => option.sessionIds.every((sessionId) => draftRule.timeScopeIds.includes(sessionId)));
  const allRuleTargetsSelected = availableRuleTargetOptions.length > 0
    && availableRuleTargetOptions.every((option) => draftRule.targetIds.includes(option.id));
  const showTaskScopeStep = draftRule.timeScopeType === "full_self_study" || draftRule.timeScopeIds.length > 0;
  const showTargetScopeStep = draftRule.timeScopeType === "full_self_study" || draftRule.timeScopeIds.length > 0;
  const ruleTargetHintText = useMemo(() => {
    if (draftRule.targetScopeType !== "selected_targets") {
      return "";
    }
    if (draftRule.timeScopeType === "exam_session" && draftRule.timeScopeIds.length === 0) {
      return "请先选择考试时段，再指定具体考场、班级或楼层任务。";
    }
    if (availableRuleTargetOptions.length === 0) {
      return "当前没有可选对象。若要指定考场或班级，请先完成一次考场/监考任务生成。";
    }
    return "";
  }, [availableRuleTargetOptions.length, draftRule.targetScopeType, draftRule.timeScopeIds.length, draftRule.timeScopeType]);
  const draftRuleSummary = useMemo(() => {
    const actionLabel = draftRule.actionType === "require" ? "指定安排" : draftRule.actionType === "exclude" ? "禁排" : "未选择动作";
    const teacherName = selectedRuleTeacherName || "某位老师";
    const timeLabel = selectedRuleTimeLabels.length > 0
      ? selectedRuleTimeLabels.join("、")
      : draftRule.timeScopeType === "full_self_study"
        ? "全员自习时段"
        : "未选择考试时段";
    const targetLabel = draftRule.targetScopeType === "all"
      ? "全部对象"
      : selectedRuleTargetOptions.length > 0
        ? selectedRuleTargetOptions.map((option) => option.label).join("、")
        : "未选择对象";
    return `${actionLabel} ${teacherName} 在 ${timeLabel} 的 ${ruleTaskScopeLabel(draftRule.taskScopeType)}（${targetLabel}）`;
  }, [draftRule, selectedRuleTargetOptions, selectedRuleTeacherName, selectedRuleTimeLabels]);

  useEffect(() => {
    setCurrentPage(1);
  }, [gradeFilter]);

  useEffect(() => {
    if (currentPage > totalPages) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages]);

  useEffect(() => {
    if (filteredMiddleManagerTeachers.length === 0) {
      setMiddleManagerPage(1);
      return;
    }
    const maxPage = Math.max(1, Math.ceil(filteredMiddleManagerTeachers.length / middleManagerPageSize));
    if (middleManagerPage > maxPage) {
      setMiddleManagerPage(maxPage);
    }
  }, [filteredMiddleManagerTeachers.length, middleManagerPage]);

  useEffect(() => {
    setMiddleManagerPage(1);
  }, [showOnlyMiddleManagerExceptions, middleManagerKeyword]);

  useEffect(() => {
    if (currentPageSelectAllRef.current) {
      currentPageSelectAllRef.current.indeterminate = indeterminateCurrentPageSelected;
    }
  }, [indeterminateCurrentPageSelected]);

  useEffect(() => {
    function handleGlobalPointerDown(event: MouseEvent) {
      if (!subjectMenu.open) {
        return;
      }
      const target = event.target as HTMLElement | null;
      if (target?.closest(".subject-menu") || target?.closest(".subject-badge") || target?.closest(".toolbar-btn.primary")) {
        return;
      }
      closeSubjectMenu();
    }

    document.addEventListener("mousedown", handleGlobalPointerDown);
    return () => {
      document.removeEventListener("mousedown", handleGlobalPointerDown);
    };
  }, [subjectMenu.open]);

  useEffect(() => {
    let removeAssignmentProgressListener: UnlistenFn | null = null;
    void (async () => {
      if (!hasDesktopRuntime()) {
        return;
      }
      removeAssignmentProgressListener = await listen<ExamStaffAssignmentProgress>(staffAssignmentProgressEvent, (event) => {
        setAssignmentProgress(event.payload);
      });
      await loadAll();
      await loadSelfStudyClassData();
    })();
    return () => {
      removeAssignmentProgressListener?.();
      removeAssignmentProgressListener = null;
    };
  // 配置页数据仅在首次挂载时加载，避免重复初始化。
  }, []);

  async function showAssignmentNoticeMessage(type: AssignmentNotice["type"], text: string, options?: Partial<AssignmentNotice>) {
    setAssignmentNotice({ type, text, ...options });
    window.setTimeout(() => {
      assignmentNoticeEl.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }, 0);
  }

  function openDialog(options: {
    kind: "confirm" | "alert";
    title: string;
    summary: string;
    details?: string[];
    confirmText?: string;
    cancelText?: string;
  }) {
    setDialogState({
      visible: true,
      kind: options.kind,
      title: options.title,
      summary: options.summary,
      details: options.details ?? [],
      confirmText: options.confirmText ?? (options.kind === "confirm" ? "确认" : "知道了"),
      cancelText: options.cancelText ?? "取消",
    });
    return new Promise<boolean>((resolve) => {
      dialogResolverRef.current = resolve;
    });
  }

  function closeDialog(result: boolean) {
    dialogResolverRef.current?.(result);
    dialogResolverRef.current = null;
    setDialogState(defaultDialogState);
  }

  async function loadSelfStudyClassData() {
    setSelfStudyLoading(true);
    setSelfStudyLoadError("");
    try {
      const [classResult, sessionResult] = await Promise.all([
        classConfigService.list({ configType: "teaching_class", gradeName: "", keyword: "" }),
        examAllocationService.listSessions({ page: 1, pageSize: 1000 }),
      ]);
      const activeGrades = new Set(sessionResult.items.map((session) => session.gradeName));
      const relevantClasses = classResult.items.filter((item) => activeGrades.has(item.gradeName));
      const rows = relevantClasses
        .map((row) => mapClassRowToSelfStudyRow(row, []))
        .sort(compareTeachingClasses);
      const grades = Array.from(new Set(relevantClasses.map((item) => item.gradeName))).sort(
        (a, b) => (gradeRankMap[a] ?? 99) - (gradeRankMap[b] ?? 99) || a.localeCompare(b, "zh-CN", { numeric: true }),
      );
      if (!mountedRef.current) {
        return;
      }
      setSelfStudyClasses(rows);
      setAvailableGrades(grades);
    } catch (error) {
      if (!mountedRef.current) {
        return;
      }
      setSelfStudyLoadError(error instanceof Error ? error.message : String(error));
    } finally {
      if (mountedRef.current) {
        setSelfStudyLoading(false);
      }
    }
  }

  async function saveConfig(extra: Partial<InvigilationConfig> = {}) {
    await saveInvigilationConfig({
      defaultExamRoomRequiredCount: Math.max(1, Math.floor(defaultExamRoomRequiredCount || 1)),
      indoorAllowancePerMinute: Math.max(0, Number(indoorAllowancePerMinute || 0)),
      outdoorAllowancePerMinute: Math.max(0, Number(outdoorAllowancePerMinute || 0)),
      selfStudyDate: normalizedSelfStudyDate || state.invigilationConfig.selfStudyDate || `${inferredSelfStudyYear}-${new Date().toISOString().slice(5, 10)}`,
      selfStudyStartTime,
      selfStudyEndTime,
      ...extra,
    });
  }

  function resetSelfStudyDraftState() {
    setSelfStudyMonthDay(extractMonthDay(state.invigilationConfig.selfStudyDate));
    setSelfStudyStartTime(state.invigilationConfig.selfStudyStartTime);
    setSelfStudyEndTime(state.invigilationConfig.selfStudyEndTime);
    setSelfStudyValidationError("");
    setGradeFilter("all");
    setCurrentPage(1);
    setSelectedClassIds(new Set());
    closeSubjectMenu();
    setSelfStudyClasses((current) =>
      current.map((item) => {
        const persisted = state.selfStudyClassSubjects.find((subjectItem) => subjectItem.classId === item.id);
        return { ...item, subject: persisted?.subject ?? null };
      }),
    );
  }

  async function setMiddleManagerDefaultEnabled(nextValue: boolean) {
    if (middleManagerDefaultEnabled === nextValue) {
      return;
    }
    await saveConfig({ middleManagerDefaultEnabled: nextValue });
  }

  function increaseCount() {
    setDefaultExamRoomRequiredCount((value) => {
      const next = value + 1;
      void saveInvigilationConfig({
        ...state.invigilationConfig,
        defaultExamRoomRequiredCount: next,
      });
      return next;
    });
  }

  function decreaseCount() {
    setDefaultExamRoomRequiredCount((value) => {
      if (value <= 1) {
        return value;
      }
      const next = value - 1;
      void saveInvigilationConfig({
        ...state.invigilationConfig,
        defaultExamRoomRequiredCount: next,
      });
      return next;
    });
  }

  function openSelfStudyDrawer() {
    setMiddleManagerDrawerOpen(false);
    setCustomRuleDrawerOpen(false);
    setCustomRuleDetailOpen(false);
    resetSelfStudyDraftState();
    setSelfStudyDrawerOpen(true);
  }

  function closeSelfStudyDrawer() {
    setSelfStudyDrawerOpen(false);
    closeSubjectMenu();
  }

  function openMiddleManagerDrawer() {
    setSelfStudyDrawerOpen(false);
    setCustomRuleDrawerOpen(false);
    setCustomRuleDetailOpen(false);
    closeSubjectMenu();
    setMiddleManagerDefaultEnabledDraft(state.invigilationConfig.middleManagerDefaultEnabled);
    setMiddleManagerExceptionTeacherIdsDraft([...state.invigilationConfig.middleManagerExceptionTeacherIds]);
    setMiddleManagerKeyword("");
    setMiddleManagerPage(1);
    setShowMiddleManagerPicker(false);
    setShowOnlyMiddleManagerExceptions(false);
    setMiddleManagerDrawerOpen(true);
  }

  function closeMiddleManagerDrawer() {
    setMiddleManagerDrawerOpen(false);
    setMiddleManagerKeyword("");
    setMiddleManagerPage(1);
    setShowMiddleManagerPicker(false);
    setShowOnlyMiddleManagerExceptions(false);
  }

  function closeCustomRuleDrawer() {
    setCustomRuleDrawerOpen(false);
  }

  function closeCustomRuleDetail() {
    setCustomRuleDetailOpen(false);
    setSelectedCustomRule(null);
  }

  function closeActiveDrawer() {
    if (selfStudyDrawerOpen) {
      closeSelfStudyDrawer();
    }
    if (middleManagerDrawerOpen) {
      closeMiddleManagerDrawer();
    }
    if (customRuleDrawerOpen) {
      closeCustomRuleDrawer();
    }
    if (customRuleDetailOpen) {
      closeCustomRuleDetail();
    }
  }

  function openCustomRuleDrawer() {
    closeActiveDrawer();
    setDraftRule(createEmptyDraftRule());
    setDraftRuleError("");
    setCustomRuleDrawerOpen(true);
  }

  function openCustomRuleDetail(rule: InvigilationCustomRule) {
    closeActiveDrawer();
    setSelectedCustomRule(cloneRule(rule));
    setCustomRuleDetailOpen(true);
  }

  async function saveDraftRule() {
    if (!draftRule.actionType) {
      setDraftRuleError("请选择规则动作");
      return;
    }
    if (!draftRule.teacherId) {
      setDraftRuleError("请选择教师");
      return;
    }
    if (draftRule.timeScopeType === "exam_session" && draftRule.timeScopeIds.length === 0) {
      setDraftRuleError("请至少选择一个考试时段");
      return;
    }
    if (draftRule.targetScopeType === "selected_targets" && draftRule.targetIds.length === 0) {
      setDraftRuleError("请选择至少一个作用对象");
      return;
    }
    const teacher = state.teachers.find((item) => item.id === draftRule.teacherId);
    if (!teacher) {
      setDraftRuleError("未找到所选教师");
      return;
    }

    const newRule: InvigilationCustomRule = {
      actionType: draftRule.actionType,
      teacherId: teacher.id,
      teacherName: teacher.teacherName,
      timeScopeType: draftRule.timeScopeType,
      timeScopeIds: [...draftRule.timeScopeIds],
      timeScopeLabels: [...selectedRuleTimeLabels],
      taskScopeType: draftRule.taskScopeType,
      targetScopeType: draftRule.targetScopeType,
      targetIds: draftRule.targetScopeType === "all" ? [] : [...draftRule.targetIds],
      targetLabels: draftRule.targetScopeType === "all" ? [] : selectedRuleTargetOptions.map((option) => option.label),
    };

    const currentRules = state.customRules.map(cloneRule);
    currentRules.unshift(newRule);
    setDraftRuleError("");
    try {
      await saveCustomRules(currentRules);
      closeCustomRuleDrawer();
    } catch (error) {
      setDraftRuleError(error instanceof Error ? error.message : String(error));
    }
  }

  async function removeCustomRule(ruleToRemove: InvigilationCustomRule) {
    const currentRules = state.customRules
      .filter((rule) => rule !== ruleToRemove)
      .map(cloneRule);
    await saveCustomRules(currentRules);
  }

  function formatRuleTimeScope(rule: InvigilationCustomRule) {
    if (rule.timeScopeLabels.length > 0) {
      return rule.timeScopeLabels.join("、");
    }
    return rule.timeScopeType === "full_self_study" ? "全员自习时段" : "未设置考试时段";
  }

  function formatRuleTimeScopeSummary(rule: InvigilationCustomRule) {
    if (rule.timeScopeType === "full_self_study") {
      return "全员自习时段";
    }
    if (rule.timeScopeLabels.length <= 1) {
      return formatRuleTimeScope(rule);
    }
    return `${rule.timeScopeLabels.length} 个考试时段`;
  }

  function formatRuleTargetScope(rule: InvigilationCustomRule) {
    if (rule.targetScopeType === "all") {
      return "全部对象";
    }
    if (rule.targetLabels.length > 0) {
      return rule.targetLabels.join("、");
    }
    return "指定对象";
  }

  function formatRuleTargetScopeSummary(rule: InvigilationCustomRule) {
    if (rule.targetScopeType === "all") {
      return "全部对象";
    }
    if (rule.targetLabels.length <= 1) {
      return formatRuleTargetScope(rule);
    }
    return `${rule.targetLabels.length} 个对象`;
  }

  function resolvedRuleTimeScopeLabels(rule: InvigilationCustomRule) {
    if (rule.timeScopeLabels.length > 0) {
      return [...rule.timeScopeLabels];
    }
    return [formatRuleTimeScope(rule)];
  }

  function selectRuleTimeScopeType(nextType: InvigilationRuleTimeScopeType) {
    setDraftRule((current) => ({
      ...current,
      timeScopeType: nextType,
      timeScopeIds: [],
      targetIds: [],
      taskScopeType: nextType === "full_self_study"
        ? "full_self_study"
        : current.taskScopeType === "full_self_study"
          ? "exam_room"
          : current.taskScopeType,
    }));
  }

  function selectRuleTaskScopeType(taskScopeType: InvigilationRuleTaskScopeType) {
    setDraftRule((current) => ({ ...current, taskScopeType, targetIds: [] }));
  }

  function selectRuleTargetScopeType(scopeType: "all" | "selected_targets") {
    setDraftRule((current) => ({
      ...current,
      targetScopeType: scopeType,
      targetIds: scopeType === "all" ? [] : current.targetIds,
    }));
  }

  function toggleRuleTimeScopeIds(sessionIds: number[]) {
    setDraftRule((current) => {
      const nextIds = new Set(current.timeScopeIds);
      const shouldSelect = !sessionIds.every((sessionId) => nextIds.has(sessionId));
      for (const sessionId of sessionIds) {
        if (shouldSelect) {
          nextIds.add(sessionId);
        } else {
          nextIds.delete(sessionId);
        }
      }
      const updatedTimeScopeIds = Array.from(nextIds).sort((left, right) => left - right);
      const updatedTargetIds = current.targetIds.filter((targetId) =>
        state.customRuleOptions.targetOptions.some((option) => {
          if (option.taskScopeType !== current.taskScopeType || option.timeScopeType !== current.timeScopeType) {
            return false;
          }
          if (current.timeScopeType === "exam_session") {
            return updatedTimeScopeIds.includes(option.timeScopeId || -1) && option.id === targetId;
          }
          return option.id === targetId;
        }),
      );
      return {
        ...current,
        timeScopeIds: updatedTimeScopeIds,
        targetIds: updatedTargetIds,
      };
    });
  }

  function toggleAllRuleTimeScopes() {
    if (allRuleTimeScopesSelected) {
      setDraftRule((current) => ({ ...current, timeScopeIds: [], targetIds: [] }));
      return;
    }
    const nextIds = new Set<number>();
    for (const option of groupedExamSessionRuleOptions) {
      for (const sessionId of option.sessionIds) {
        nextIds.add(sessionId);
      }
    }
    setDraftRule((current) => ({ ...current, timeScopeIds: Array.from(nextIds).sort((left, right) => left - right) }));
  }

  function toggleRuleTargetId(id: string) {
    setDraftRule((current) => {
      const nextIds = new Set(current.targetIds);
      if (nextIds.has(id)) {
        nextIds.delete(id);
      } else {
        nextIds.add(id);
      }
      return { ...current, targetIds: Array.from(nextIds) };
    });
  }

  function toggleAllRuleTargets() {
    if (allRuleTargetsSelected) {
      setDraftRule((current) => ({ ...current, targetIds: [] }));
      return;
    }
    setDraftRule((current) => ({ ...current, targetIds: availableRuleTargetOptions.map((option) => option.id) }));
  }

  function toggleRowSelection(id: number) {
    setSelectedClassIds((current) => {
      const next = new Set(current);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  function toggleSelectAllCurrentPage() {
    setSelectedClassIds((current) => {
      const next = new Set(current);
      if (allCurrentPageSelected) {
        pagedClasses.forEach((item) => next.delete(item.id));
      } else {
        pagedClasses.forEach((item) => next.add(item.id));
      }
      return next;
    });
  }

  function openSubjectMenu(rowId: number, event: React.MouseEvent<HTMLElement>) {
    setBulkMenuOpen(false);
    openSubjectMenuAtEvent(event.currentTarget, rowId, "single");
  }

  function toggleBulkMenu(event: React.MouseEvent<HTMLElement>) {
    if (selectedClassCount === 0) {
      return;
    }
    if (bulkMenuOpen) {
      closeSubjectMenu();
      return;
    }
    setBulkMenuOpen(true);
    openSubjectMenuAtEvent(event.currentTarget, null, "bulk");
  }

  function openSubjectMenuAtEvent(target: HTMLElement, rowId: number | null, mode: "single" | "bulk") {
    const rect = target.getBoundingClientRect();
    const menuWidth = 168;
    const menuHeight = Math.min(5 * 42 + 16, window.innerHeight - 80);
    const padding = 12;
    let top = rect.bottom + 8;
    let left = rect.left;
    if (top + menuHeight > window.innerHeight - padding) {
      top = Math.max(padding, rect.top - menuHeight - 8);
    }
    if (left + menuWidth > window.innerWidth - padding) {
      left = window.innerWidth - menuWidth - padding;
    }
    if (left < padding) {
      left = padding;
    }
    setSubjectMenu({ open: true, top, left, rowId, mode });
  }

  function closeSubjectMenu() {
    setSubjectMenu({ open: false, top: 0, left: 0, rowId: null, mode: "single" });
    setBulkMenuOpen(false);
  }

  function applySubjectSelection(subject: Subject) {
    if (subjectMenu.mode === "bulk") {
      setSelfStudyClasses((current) =>
        current.map((item) => (selectedClassIds.has(item.id) ? { ...item, subject } : item)),
      );
      closeSubjectMenu();
      return;
    }
    if (subjectMenu.rowId === null) {
      return;
    }
    const applyToSelected = selectedClassIds.size > 1 && selectedClassIds.has(subjectMenu.rowId);
    setSelfStudyClasses((current) =>
      current.map((item) => {
        if (applyToSelected) {
          return selectedClassIds.has(item.id) ? { ...item, subject } : item;
        }
        return item.id === subjectMenu.rowId ? { ...item, subject } : item;
      }),
    );
    closeSubjectMenu();
  }

  async function saveSelfStudySetup() {
    const monthDay = selfStudyMonthDay.trim();
    const startTime = selfStudyStartTime.trim();
    const endTime = selfStudyEndTime.trim();
    if (!monthDay) {
      setSelfStudyValidationError("请选择自习日期。");
      return;
    }
    if (!/^\d{2}-\d{2}$/.test(monthDay)) {
      setSelfStudyValidationError("自习日期请按月-日填写，例如 03-26。");
      return;
    }
    if (!startTime) {
      setSelfStudyValidationError("请填写开始时间。");
      return;
    }
    if (!endTime) {
      setSelfStudyValidationError("请填写结束时间。");
      return;
    }
    if (!/^\d{2}:\d{2}$/.test(startTime) || !/^\d{2}:\d{2}$/.test(endTime)) {
      setSelfStudyValidationError("开始时间和结束时间请按 HH:MM 填写，例如 12:10。");
      return;
    }
    if (!normalizedSelfStudyDate) {
      setSelfStudyValidationError("自习日期格式不正确。");
      return;
    }
    if (`${normalizedSelfStudyDate}T${endTime}` <= `${normalizedSelfStudyDate}T${startTime}`) {
      setSelfStudyValidationError("结束时间必须晚于开始时间。");
      return;
    }
    setSelfStudyValidationError("");
    await saveConfig();
    const visibleClassIds = new Set(selfStudyClasses.map((item) => item.id));
    const preservedHiddenSubjects = state.selfStudyClassSubjects.filter(
      (item) => !visibleClassIds.has(item.classId),
    );
    await saveSelfStudyClassSubjects([
      ...preservedHiddenSubjects,
      ...selfStudyClasses.map((item) => ({ classId: item.id, subject: item.subject })),
    ]);
    closeSelfStudyDrawer();
  }

  function isMiddleManagerException(teacherId: number) {
    return middleManagerExceptionTeacherIdsDraft.includes(teacherId);
  }

  function toggleMiddleManagerExceptionTeacher(teacherId: number) {
    setMiddleManagerExceptionTeacherIdsDraft((current) => {
      if (current.includes(teacherId)) {
        return current.filter((id) => id !== teacherId);
      }
      return [...current, teacherId].sort((a, b) => a - b);
    });
  }

  function getMiddleManagerStatusLabel(teacherId: number) {
    const isException = isMiddleManagerException(teacherId);
    const enabled = isException ? !middleManagerDefaultEnabledDraft : middleManagerDefaultEnabledDraft;
    return enabled ? "参与" : "不参与";
  }

  function getMiddleManagerStatusClass(teacherId: number) {
    return getMiddleManagerStatusLabel(teacherId) === "参与" ? "on" : "off";
  }

  async function saveMiddleManagerSetup() {
    await saveConfig({
      middleManagerDefaultEnabled: middleManagerDefaultEnabledDraft,
      middleManagerExceptionTeacherIds: middleManagerExceptionTeacherIdsDraft,
    });
    closeMiddleManagerDrawer();
  }

  async function handleAssignTeachers() {
    if (state.staffOverview.generatedAt) {
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

    setAssignmentNotice(null);
    setAssignmentProgress({
      status: "running",
      stage: "preparing",
      stageLabel: "准备开始",
      percent: 0,
      message: "正在准备监考分配...",
      completedSteps: 0,
      totalSteps: 13,
      updatedAt: new Date().toISOString(),
    });
    window.setTimeout(() => {
      assignmentNoticeEl.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }, 0);
    try {
      const result = await assignTeachers();
      setAssignmentProgress({
        status: "completed",
        stage: "completed",
        stageLabel: "分配完成",
        percent: 100,
        message: "监考分配完成，正在刷新结果...",
        completedSteps: 13,
        totalSteps: 13,
        updatedAt: new Date().toISOString(),
      });
      const summary = result.optimalityStatus === "optimal"
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
      const fallbackPart = result.fallbackPoolAssignments > 0 ? `，其他老师补位 ${result.fallbackPoolAssignments} 项` : "";
      const mainMessage = `${summary}：已分配 ${result.assignedCount} 项，未分配 ${result.unassignedCount} 项，${optimality}，耗时 ${formatSolveDuration(result.solveDurationMs)}${fallbackPart}。`;
      const unassignedPart = result.unassignedDetails.length > 0 ? `\n未分配的任务：${result.unassignedDetails.join("、")}。` : "";
      await showAssignmentNoticeMessage(result.unassignedCount > 0 ? "warning" : "success", `${mainMessage}${unassignedPart}`);
    } catch (error) {
      setAssignmentProgress(null);
      const message = state.errorMessage || (error instanceof Error ? error.message : String(error)) || "分配失败，请检查配置后重试。";
      await showAssignmentNoticeMessage("error", `分配失败：${message}`);
    }
  }

  async function handleExportInvigilationSchedule() {
    try {
      const result = await exportLatestInvigilationSchedule();
      await showAssignmentNoticeMessage("success", "监考表已导出，点击下方链接打开文件所在位置。", {
        linkPath: result.filePath,
        linkLabel: exportFileName(result.filePath),
      });
    } catch (error) {
      const message = state.errorMessage || (error instanceof Error ? error.message : String(error)) || "导出失败，请稍后重试。";
      await showAssignmentNoticeMessage("error", `导出失败：${message}`);
    }
  }

  async function openInvigilationExportFolder() {
    const target = assignmentNoticeLinkPath || state.lastInvigilationExportPath;
    if (!target) {
      return;
    }
    await revealInExplorer(target);
  }

  return (
    <section className="panel">
      <div className="grid-two top-grid">
        <ConfigCard className="top-card exam-count-card" title="监考人数配置">
          <div className="exam-count-content">
            <button className="count-btn" type="button" disabled={defaultExamRoomRequiredCount <= 1} onClick={decreaseCount} aria-label="减少人数">
              <Minus size={18} />
            </button>
            <div className="count-display">
              <span className="count-number">{defaultExamRoomRequiredCount}</span>
              <span className="count-unit">人</span>
            </div>
            <button className="count-btn" type="button" onClick={increaseCount} aria-label="增加人数">
              <Plus size={18} />
            </button>
          </div>
        </ConfigCard>

        <ConfigCard className="top-card middle-manager-card" title="校中层监考配置">
          <div className="card-stack">
            <div className="segment-wrap">
              <button className={`segment-btn ${middleManagerDefaultEnabled ? "active" : ""}`.trim()} type="button" onClick={() => void setMiddleManagerDefaultEnabled(true)}>参与监考</button>
              <button className={`segment-btn ${!middleManagerDefaultEnabled ? "active" : ""}`.trim()} type="button" onClick={() => void setMiddleManagerDefaultEnabled(false)}>不参与监考</button>
            </div>
            <div className="footer-row middle-footer">
              <div className="exception-tag">
                <Users size={18} /> 已设置 {middleManagerExceptionCount} 位例外人员
              </div>
              <button className="drawer-trigger exception-btn" type="button" onClick={openMiddleManagerDrawer}>
                <Settings size={18} /> 配置例外
              </button>
            </div>
          </div>
        </ConfigCard>
      </div>

      <section className="schedule-card exclude-card">
        <div className="schedule-card-header">
          <div className="schedule-header-info">
            <div className="schedule-title-bar">
              <span className="schedule-title-line"></span>
              <h2 className="schedule-card-title">自定义排班规则</h2>
            </div>
            <div className="schedule-card-subtitle">当前已配置 <strong>{state.customRules.length}</strong> 条规则</div>
          </div>
          <button className="schedule-btn-primary drawer-trigger" type="button" onClick={openCustomRuleDrawer}>
            <Plus size={18} />
            添加排班规则
          </button>
        </div>

        {state.customRules.length === 0 ? (
          <div className="schedule-empty-state">
            <div className="schedule-empty-icon">
              <List size={24} />
            </div>
            <h3 className="schedule-empty-title">暂未添加排班规则</h3>
            <p className="schedule-empty-desc">点击右上方按钮添加规则，开始管理您的排班。</p>
          </div>
        ) : (
          <div className="compact-rule-list schedule-rule-list">
            {state.customRules.map((item, index) => (
              <div key={`${item.teacherId}-${index}`} className="compact-rule-item">
                <div className="compact-rule-main">
                  <div className="compact-rule-header">
                    <span className={item.actionType === "require" ? "primary-pill" : "danger-pill"}>
                      {item.actionType === "require" ? "指定安排" : "禁排"}
                    </span>
                    <strong className="custom-rule-teacher">{item.teacherName}</strong>
                    <span className="compact-rule-task">{ruleTaskScopeLabel(item.taskScopeType)}</span>
                    <span className="rule-tag">{formatRuleTimeScopeSummary(item)}</span>
                    <span className="rule-tag">{formatRuleTargetScopeSummary(item)}</span>
                  </div>
                </div>
                <div className="compact-rule-actions">
                  <button className="text-btn" type="button" onClick={() => openCustomRuleDetail(item)}>详情</button>
                  <button className="icon-btn" type="button" onClick={() => void removeCustomRule(item)}>
                    <Trash2 size={18} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <div className="grid-two summary-grid-row">
        <section className="study-card summary-card self-study-card">
          <div className="study-card-header">
            <div className="study-title-bar">
              <span className="study-title-line"></span>
              <h2 className="study-card-title">全员自习</h2>
            </div>
          </div>
          <div className="study-data-grid">
            <div className="study-data-item">
              <div className="study-data-label">时间范围</div>
              <div className="study-data-value study-data-time">{selfStudyMonthDay} {selfStudyStartTime} - {selfStudyEndTime}</div>
            </div>
            <div className="study-data-item">
              <div className="study-data-label">已配置班级</div>
              <div className="study-data-value">{configuredClassCount} <span className="study-data-unit">个</span></div>
            </div>
            <div className="study-data-item warning">
              <div className="study-data-label">待补充</div>
              <div className="study-data-value">{pendingClassCount} <span className="study-data-unit">个班级</span></div>
            </div>
          </div>
          <div className="study-card-footer">
            <span className={`study-status-text ${pendingClassCount > 0 ? "pending" : ""}`.trim()}>
              {pendingClassCount > 0 ? <AlertCircle size={18} /> : <CircleCheck size={18} />}
              {selfStudySummaryText}
            </span>
            <button className="study-btn-primary drawer-trigger" type="button" onClick={openSelfStudyDrawer}>
              <SlidersHorizontal size={18} />
              配置班级科目
            </button>
          </div>
        </section>

        <ConfigCard className="summary-card allowance-card">
          <div className="allowance-title-bar">
            <span className="allowance-title-line"></span>
            <span className="allowance-title-text">监考津贴</span>
          </div>
          <div className="allowance-items-container">
            <div className="allowance-item allowance-item-indoor">
              <div className="allowance-item-label">
                <span className="allowance-dot allowance-dot-inside"></span>
                场内监考津贴
              </div>
              <div className="allowance-item-value">
                <input
                  className="allowance-value-input"
                  type="number"
                  min="0"
                  step="0.1"
                  value={indoorAllowancePerMinute}
                  onChange={(event) => setIndoorAllowancePerMinute(Number(event.target.value))}
                  onBlur={() => void saveConfig()}
                  onKeyUp={(event) => {
                    if (event.key === "Enter") {
                      void saveConfig();
                    }
                  }}
                />
                <span className="allowance-value-unit">元 / 分钟</span>
              </div>
            </div>
            <div className="allowance-item allowance-item-outdoor">
              <div className="allowance-item-label">
                <span className="allowance-dot allowance-dot-outside"></span>
                场外监考津贴
              </div>
              <div className="allowance-item-value">
                <input
                  className="allowance-value-input"
                  type="number"
                  min="0"
                  step="0.1"
                  value={outdoorAllowancePerMinute}
                  onChange={(event) => setOutdoorAllowancePerMinute(Number(event.target.value))}
                  onBlur={() => void saveConfig()}
                  onKeyUp={(event) => {
                    if (event.key === "Enter") {
                      void saveConfig();
                    }
                  }}
                />
                <span className="allowance-value-unit">元 / 分钟</span>
              </div>
            </div>
          </div>
        </ConfigCard>
      </div>

      <ConfigCard>
        <div className="action-row">
          <div className="action-copy">
            <p className="action-text">点击按钮为考场、自习室及楼层分配监考、看班老师 ~~~</p>
            {state.staffOverview.generatedAt ? <p className="solver-summary">{staffSolverSummary}</p> : null}
          </div>
          {displayedAssignmentNotice || isAssignmentProgressVisible ? (
            <div ref={assignmentNoticeEl} className="assignment-notice inline" role="status" aria-live="polite" tabIndex={-1}>
              {assignmentNoticeIcon}
              <div className="assignment-notice-body">
                <span className="assignment-notice-text">{assignmentNoticeText}</span>
                {assignmentNoticeLinkPath && !isAssignmentProgressVisible ? (
                  <button className="assignment-notice-link" type="button" onClick={() => void openInvigilationExportFolder()}>
                    {assignmentNoticeLinkLabel}
                  </button>
                ) : null}
                {isAssignmentProgressVisible && assignmentProgress ? (
                  <div className="assignment-progress">
                    <div className="assignment-progress-meta">
                      <span>{assignmentProgress.stageLabel}</span>
                      <span>{assignmentProgress.percent}%</span>
                    </div>
                    <div className="assignment-progress-track" aria-hidden="true">
                      <div className="assignment-progress-bar" style={{ width: `${assignmentProgress.percent}%` }} />
                    </div>
                  </div>
                ) : null}
              </div>
            </div>
          ) : null}
          <div className="action-buttons">
            <button className="primary-btn action-btn" type="button" disabled={state.assigning} onClick={() => void handleAssignTeachers()}>
              {state.assigning ? "分配中..." : "开始分配"}
            </button>
            <button className="secondary-btn action-btn" type="button" disabled={!state.staffOverview.generatedAt || state.exportingInvigilation} onClick={() => void handleExportInvigilationSchedule()}>
              {state.exportingInvigilation ? "导出中..." : "导出监考表"}
            </button>
          </div>
        </div>
      </ConfigCard>

      {activeDrawer !== null ? <div className="drawer-backdrop" onClick={closeActiveDrawer} /> : null}

      {selfStudyDrawerOpen ? (
        <aside className="config-drawer self-study-drawer">
          <div className="drawer-header">
            <div className="drawer-title-block">
              <h3>配置全员自习</h3>
            </div>
            <button className="drawer-close" type="button" onClick={closeSelfStudyDrawer}><X size={18} /></button>
          </div>

          <section className="drawer-section soft-panel">
            <div className="section-header"><h4>统一时段</h4></div>
            <div className="schedule-row">
              <label className="display-field compact-field">
                <span className="field-label">自习日期</span>
                <input className="value-input framed-input date-input" type="text" inputMode="numeric" placeholder="03-26" value={selfStudyMonthDay} onChange={(event) => setSelfStudyMonthDay(event.target.value)} />
              </label>
              <label className="display-field compact-field">
                <span className="field-label">开始时间</span>
                <input className="value-input framed-input time-input" type="text" inputMode="numeric" maxLength={5} placeholder="12:10" value={selfStudyStartTime} onChange={(event) => setSelfStudyStartTime(event.target.value)} />
              </label>
              <label className="display-field compact-field">
                <span className="field-label">结束时间</span>
                <input className="value-input framed-input time-input" type="text" inputMode="numeric" maxLength={5} placeholder="13:40" value={selfStudyEndTime} onChange={(event) => setSelfStudyEndTime(event.target.value)} />
              </label>
            </div>
            <div className="footer-row">
              <span className="field-label">{selfStudyScopeText}</span>
              <span className="info-pill">全体教师默认转为自习值守</span>
            </div>
            {selfStudyValidationError ? <div className="empty-box error-box">{selfStudyValidationError}</div> : null}
          </section>

          <section className="drawer-section class-config-section">
            <div className="section-header">
              <div><h4>班级科目配置</h4></div>
            </div>
            {selfStudyLoadError ? <div className="empty-box error-box">{selfStudyLoadError}</div> : null}
            {!selfStudyLoadError && selfStudyLoading ? <div className="empty-box">正在加载教学班列表...</div> : null}
            {!selfStudyLoadError && !selfStudyLoading && filteredClasses.length === 0 ? <div className="empty-box">本次考试暂无涉及的教学班，请先生成包含实际考生的考场安排。</div> : null}

            {!selfStudyLoading && filteredClasses.length > 0 && selectedClassCount > 0 ? <div className="selection-strip">已选 {selectedClassCount} 个班级</div> : null}

            {!selfStudyLoading && filteredClasses.length > 0 ? (
              <div className="toolbar-row">
                <div className="toolbar-left">
                  <button className="toolbar-btn primary" type="button" disabled={selectedClassCount === 0} onClick={toggleBulkMenu}>为选中班级设科目</button>
                  <div className="toolbar-filter">
                    <FluentSelect
                      modelValue={gradeFilter}
                      options={[{ label: "全部年级", value: "all" }, ...availableGrades.map((g) => ({ label: g, value: g }))]}
                      className="grade-filter-select"
                      onUpdateModelValue={(value) => setGradeFilter(String(value || "all"))}
                    />
                  </div>
                </div>
                <span className="pending-pill">{pendingClassCount} 个待处理</span>
              </div>
            ) : null}

            {!selfStudyLoading && filteredClasses.length > 0 ? (
              <div className="class-table">
                <div className="class-table-head">
                  <label className="check-cell">
                    <input ref={currentPageSelectAllRef} type="checkbox" checked={allCurrentPageSelected} onChange={toggleSelectAllCurrentPage} />
                  </label>
                  <span>班级</span>
                  <span>年级</span>
                  <span>科目</span>
                  <span>状态</span>
                </div>
                {pagedClasses.map((row) => (
                  <div key={row.id} className={`class-table-row ${selectedClassIds.has(row.id) ? "selected" : ""}`.trim()}>
                    <label className="check-cell">
                      <input type="checkbox" checked={selectedClassIds.has(row.id)} onChange={() => toggleRowSelection(row.id)} />
                    </label>
                    <span className="cell-text strong">{row.className}</span>
                    <span className="cell-text muted">{row.gradeName}</span>
                    <button className={`subject-badge ${!row.subject ? "empty" : ""}`.trim()} type="button" onClick={(event) => openSubjectMenu(row.id, event)}>
                      {row.subject ? subjectLabelMap[row.subject] : "未选"}
                    </button>
                    <span className={`status-badge ${row.subject ? "done" : "pending"}`.trim()}>{row.subject ? "已完成" : "待处理"}</span>
                  </div>
                ))}
              </div>
            ) : null}

            <Pagination currentPage={currentPage} pageSize={pageSize} total={filteredClasses.length} onChange={setCurrentPage} />
          </section>

          <div className="drawer-footer">
            <p></p>
            <div className="drawer-actions">
              <button className="secondary-btn" type="button" onClick={closeSelfStudyDrawer}>取消</button>
              <button className="primary-btn" type="button" onClick={() => void saveSelfStudySetup()}>保存配置</button>
            </div>
          </div>
        </aside>
      ) : null}

      {middleManagerDrawerOpen ? (
        <aside className="config-drawer middle-manager-drawer">
          <div className="drawer-header">
            <div className="drawer-title-block">
              <h3>中层监考例外</h3>
            </div>
            <button className="drawer-close" type="button" onClick={closeMiddleManagerDrawer}><X size={18} /></button>
          </div>

          <section className="drawer-section soft-panel">
            <div className="section-header"><h4>默认规则</h4></div>
            <div className="segment-wrap">
              <button className={`segment-btn ${middleManagerDefaultEnabledDraft ? "active" : ""}`.trim()} type="button" onClick={() => setMiddleManagerDefaultEnabledDraft(true)}>参与监考</button>
              <button className={`segment-btn ${!middleManagerDefaultEnabledDraft ? "active" : ""}`.trim()} type="button" onClick={() => setMiddleManagerDefaultEnabledDraft(false)}>不参与监考</button>
            </div>
          </section>

          <section className="drawer-section">
            <div className="section-header">
              <div className="title-stack"><h4>例外名单</h4></div>
            </div>

            <div className="middle-toolbar">
              <button className="primary-btn middle-primary-btn" type="button" onClick={() => setShowMiddleManagerPicker((value) => !value)}>
                {showMiddleManagerPicker ? "收起添加面板" : "添加例外人员"}
              </button>
              <button className={`middle-filter-btn ${showOnlyMiddleManagerExceptions ? "active" : ""}`.trim()} type="button" onClick={() => setShowOnlyMiddleManagerExceptions((value) => !value)}>
                仅看例外
              </button>
              <span className="exception-pill">{middleManagerExceptionTeacherIdsDraft.length} 位例外</span>
            </div>

            {showMiddleManagerPicker ? (
              <div className="middle-picker">
                <label className="search-bar middle-search">
                  <Search size={18} />
                  <input value={middleManagerKeyword} onChange={(event) => setMiddleManagerKeyword(event.target.value)} type="text" placeholder="输入姓名搜索中层教师" />
                </label>
              </div>
            ) : null}

            {pagedMiddleManagerTeachers.length > 0 ? (
              <div className="exclude-list">
                {pagedMiddleManagerTeachers.map((teacher) => (
                  <div key={teacher.id} className="exclude-item middle-exception-item">
                    <div className="middle-person">
                      <strong>{teacher.teacherName}</strong>
                    </div>
                    <div className="middle-actions">
                      <span className={`middle-status-pill ${getMiddleManagerStatusClass(teacher.id)}`.trim()}>
                        {getMiddleManagerStatusLabel(teacher.id)}
                      </span>
                      <button className="text-btn" type="button" onClick={() => toggleMiddleManagerExceptionTeacher(teacher.id)}>
                        {isMiddleManagerException(teacher.id) ? "取消例外" : "设为例外"}
                      </button>
                    </div>
                  </div>
                ))}
                <Pagination currentPage={middleManagerPage} pageSize={middleManagerPageSize} total={filteredMiddleManagerTeachers.length} onChange={setMiddleManagerPage} />
              </div>
            ) : (
              <div className="empty-box">{showOnlyMiddleManagerExceptions ? "当前还没有例外人员。" : "没有匹配的中层教师。"}</div>
            )}
          </section>

          <div className="drawer-footer">
            <div className="drawer-actions">
              <button className="secondary-btn" type="button" onClick={closeMiddleManagerDrawer}>取消</button>
              <button className="primary-btn" type="button" onClick={() => void saveMiddleManagerSetup()}>保存例外</button>
            </div>
          </div>
        </aside>
      ) : null}

      {subjectMenu.open ? (
        <div className="subject-menu" style={{ top: `${subjectMenu.top}px`, left: `${subjectMenu.left}px` }} onClick={(event) => event.stopPropagation()}>
          {selectableSubjects.map((subject) => (
            <button key={subject} className={`subject-menu-item ${subjectMenuSelectedSubject === subject ? "active" : ""}`.trim()} type="button" onClick={() => applySubjectSelection(subject)}>
              <span>{subjectLabelMap[subject]}</span>
              {subjectMenuSelectedSubject === subject ? <Check size={18} /> : null}
            </button>
          ))}
        </div>
      ) : null}

      {dialogState.visible ? (
        <div className="dialog-mask" onClick={(event) => {
          if (event.target === event.currentTarget) {
            closeDialog(false);
          }
        }}>
          <section className="dialog card-shell">
            <header className="dialog-head">
              <h3>{dialogState.title}</h3>
              <button className="dialog-close" type="button" onClick={() => closeDialog(false)}>×</button>
            </header>
            <p className="dialog-summary">{dialogState.summary}</p>
            {dialogState.details.length > 0 ? (
              <ul className="dialog-details">
                {dialogState.details.map((line, index) => <li key={`${line}-${index}`}>{line}</li>)}
              </ul>
            ) : null}
            <footer className="dialog-actions">
              {dialogState.kind === "confirm" ? (
                <button className="secondary-btn" type="button" onClick={() => closeDialog(false)}>
                  {dialogState.cancelText}
                </button>
              ) : null}
              <button className="primary-btn" type="button" onClick={() => closeDialog(true)}>
                {dialogState.confirmText}
              </button>
            </footer>
          </section>
        </div>
      ) : null}

      {customRuleDrawerOpen ? (
        <aside className="config-drawer custom-rule-drawer">
          <div className="drawer-header">
            <div className="drawer-title-block">
              <h3>添加排班规则</h3>
              <p>按时间范围、任务类型和作用对象配置禁排或指定安排。</p>
            </div>
            <button className="drawer-close" type="button" onClick={closeCustomRuleDrawer}><X size={18} /></button>
          </div>

          <section className="drawer-section soft-panel custom-rule-panel">
            <div className="form-group">
              <label className="field-label form-label">规则动作</label>
              <div className="segment-wrap full-width">
                <button className={`segment-btn ${draftRule.actionType === "exclude" ? "active" : ""}`.trim()} type="button" onClick={() => setDraftRule((current) => ({ ...current, actionType: "exclude" }))}>禁排</button>
                <button className={`segment-btn ${draftRule.actionType === "require" ? "active" : ""}`.trim()} type="button" onClick={() => setDraftRule((current) => ({ ...current, actionType: "require" }))}>指定安排</button>
              </div>
            </div>

            {draftRule.actionType ? (
              <div className="form-group form-group-step">
                <label className="field-label form-label">指定教师 <span className="required-mark">*</span></label>
                <FluentSelect
                  modelValue={draftRule.teacherId}
                  options={teacherSelectOptions}
                  placeholder="请选择教师"
                  searchable
                  className="teacher-select-full"
                  onUpdateModelValue={(value) => setDraftRule((current) => ({ ...current, teacherId: value === "" ? "" : Number(value) }))}
                />
              </div>
            ) : null}

            {draftRule.teacherId ? (
              <div className="form-group form-group-step">
                <label className="field-label form-label">时间范围</label>
                <div className="segment-wrap full-width">
                  <button className={`segment-btn ${draftRule.timeScopeType === "exam_session" ? "active" : ""}`.trim()} type="button" onClick={() => selectRuleTimeScopeType("exam_session")}>考试时段</button>
                  <button className={`segment-btn ${draftRule.timeScopeType === "full_self_study" ? "active" : ""}`.trim()} type="button" onClick={() => selectRuleTimeScopeType("full_self_study")}>全员自习时段</button>
                </div>

                {draftRule.timeScopeType === "exam_session" ? (
                  <>
                    {groupedExamSessionRuleOptions.length > 0 ? (
                      <div className="selection-toolbar">
                        <div className="selection-toolbar-copy">
                          <strong>已选 {selectedRuleTimeLabels.length} 个考试时段</strong>
                        </div>
                        <div className="selection-toolbar-actions">
                          <button className="toolbar-toggle-btn" type="button" disabled={groupedExamSessionRuleOptions.length === 0} onClick={toggleAllRuleTimeScopes}>
                            {allRuleTimeScopesSelected ? "取消全选" : "全选"}
                          </button>
                        </div>
                      </div>
                    ) : null}
                    <div className="selection-list compact-option-list">
                      {groupedExamSessionRuleOptions.map((option) => (
                        <label key={option.id} className="check-option compact-option">
                          <input type="checkbox" checked={option.sessionIds.every((sessionId) => draftRule.timeScopeIds.includes(sessionId))} onChange={() => toggleRuleTimeScopeIds(option.sessionIds)} />
                          <div className="target-copy time-scope-copy">
                            {option.label.split("\n").map((line, idx) => (
                              <span key={`${option.id}-${idx}`} className={idx === 0 ? "time-scope-subject" : "time-scope-datetime"}>{line}</span>
                            ))}
                          </div>
                        </label>
                      ))}
                    </div>
                  </>
                ) : (
                  <div className="scope-preview">{fullSelfStudyRuleLabel}</div>
                )}

                {draftRule.timeScopeType === "exam_session" && groupedExamSessionRuleOptions.length === 0 ? (
                  <p className="drawer-note">暂无可选考试时段，请先配置考试时间，或先完成一次考场/监考任务生成。</p>
                ) : null}
                {draftRule.timeScopeType === "exam_session" && draftRule.timeScopeIds.length === 0 ? (
                  <p className="drawer-note">请先选择一个或多个考试时段，再指定具体考场、班级或楼层任务。</p>
                ) : null}
              </div>
            ) : null}

            {showTaskScopeStep ? (
              <div className="form-group form-group-step">
                <label className="field-label form-label">任务类型</label>
                <div className="option-grid">
                  {availableTaskScopeOptions.map((option) => (
                    <label key={option.value} className={`check-option single-option ${draftRule.taskScopeType === option.value ? "active" : ""}`.trim()}>
                      <input type="radio" name="custom-rule-task-scope" checked={draftRule.taskScopeType === option.value} onChange={() => selectRuleTaskScopeType(option.value)} />
                      <span>{option.label}</span>
                    </label>
                  ))}
                </div>
              </div>
            ) : null}

            {showTargetScopeStep ? (
              <div className="form-group form-group-step">
                <label className="field-label form-label">作用对象</label>
                <div className="segment-wrap full-width">
                  <button className={`segment-btn ${draftRule.targetScopeType === "all" ? "active" : ""}`.trim()} type="button" onClick={() => selectRuleTargetScopeType("all")}>全部对象</button>
                  <button className={`segment-btn ${draftRule.targetScopeType === "selected_targets" ? "active" : ""}`.trim()} type="button" onClick={() => selectRuleTargetScopeType("selected_targets")}>指定对象</button>
                </div>
                {draftRule.targetScopeType === "all" ? <p className="drawer-note">不选具体对象时，规则默认作用于当前时间范围内的全部匹配任务。</p> : null}
                {draftRule.targetScopeType === "selected_targets" ? (
                  <>
                    {ruleTargetHintText ? <p className="drawer-note">{ruleTargetHintText}</p> : null}
                    {availableRuleTargetOptions.length > 0 ? (
                      <>
                        <div className="selection-toolbar">
                          <div className="selection-toolbar-copy">
                            <strong>已选 {draftRule.targetIds.length} 个对象</strong>
                          </div>
                          <div className="selection-toolbar-actions">
                            <button className="toolbar-toggle-btn" type="button" disabled={availableRuleTargetOptions.length === 0} onClick={toggleAllRuleTargets}>
                              {allRuleTargetsSelected ? "取消全选" : "全选"}
                            </button>
                          </div>
                        </div>
                        <div className="selection-list compact-option-list">
                          {availableRuleTargetOptions.map((option) => (
                            <label key={option.id} className="check-option target-option compact-option">
                              <input type="checkbox" checked={draftRule.targetIds.includes(option.id)} onChange={() => toggleRuleTargetId(option.id)} />
                              <div className="target-copy target-option-copy">
                                <span className="target-option-label">{option.label}</span>
                                {option.subtitle ? <small className="target-option-subtitle">{formatTargetOptionSubtitle(option.subtitle)}</small> : null}
                              </div>
                            </label>
                          ))}
                        </div>
                      </>
                    ) : null}
                  </>
                ) : null}
              </div>
            ) : null}

            <div className="custom-rule-summary-box">
              <span className="field-label form-label">规则摘要</span>
              <strong>{draftRuleSummary}</strong>
            </div>
          </section>

          <div className="drawer-footer custom-rule-footer">
            {draftRuleError ? <p className="drawer-error">{draftRuleError}</p> : <p className="drawer-note">保存时会校验冲突规则，命中冲突将直接阻止保存。</p>}
            <div className="drawer-actions">
              <button className="secondary-btn" type="button" onClick={closeCustomRuleDrawer}>取消</button>
              <button className="primary-btn" disabled={!draftRule.actionType || !draftRule.teacherId} onClick={() => void saveDraftRule()}>保存规则</button>
            </div>
          </div>
        </aside>
      ) : null}

      {customRuleDetailOpen && selectedCustomRule ? (
        <aside className="config-drawer custom-rule-detail-drawer">
          <div className="drawer-header">
            <div className="drawer-title-block">
              <h3>规则详情</h3>
              <p>{selectedCustomRule.teacherName} 的{ruleTaskScopeLabel(selectedCustomRule.taskScopeType)}规则</p>
            </div>
            <button className="drawer-close" type="button" onClick={closeCustomRuleDetail}><X size={18} /></button>
          </div>

          <section className="drawer-section soft-panel custom-rule-panel">
            <div className="detail-summary-grid">
              <div className="summary-box">
                <span className="field-label">规则动作</span>
                <strong>{selectedCustomRule.actionType === "require" ? "指定安排" : "禁排"}</strong>
              </div>
              <div className="summary-box">
                <span className="field-label">任务类型</span>
                <strong>{ruleTaskScopeLabel(selectedCustomRule.taskScopeType)}</strong>
              </div>
              <div className="summary-box">
                <span className="field-label">时间范围</span>
                <strong>{formatRuleTimeScopeSummary(selectedCustomRule)}</strong>
              </div>
              <div className="summary-box">
                <span className="field-label">作用对象</span>
                <strong>{formatRuleTargetScopeSummary(selectedCustomRule)}</strong>
              </div>
            </div>

            <div className="form-group">
              <label className="field-label form-label">完整时间范围</label>
              <div className="detail-chip-list">
                {resolvedRuleTimeScopeLabels(selectedCustomRule).map((label, index) => (
                  <span key={`${label}-${index}`} className="rule-tag detail-chip">{label}</span>
                ))}
              </div>
            </div>

            <div className="form-group">
              <label className="field-label form-label">完整作用对象</label>
              {selectedCustomRule.targetScopeType === "all" ? (
                <div className="scope-preview">全部对象</div>
              ) : (
                <div className="detail-chip-list">
                  {selectedCustomRule.targetLabels.map((label, index) => (
                    <span key={`${label}-${index}`} className="rule-tag detail-chip">{label}</span>
                  ))}
                </div>
              )}
            </div>
          </section>

          <div className="drawer-footer custom-rule-footer">
            <p className="drawer-note">详情抽屉仅用于查看规则内容，修改时请删除后重新添加。</p>
            <div className="drawer-actions">
              <button className="secondary-btn" type="button" onClick={closeCustomRuleDetail}>关闭</button>
            </div>
          </div>
        </aside>
      ) : null}
    </section>
  );
}

function mapClassRowToSelfStudyRow(
  row: ClassConfigRow,
  persistedSubjects: Array<{ classId: number; subject: Subject | null }>,
): SelfStudyClassRow {
  const persisted = persistedSubjects.find((item) => item.classId === row.id);
  return {
    id: row.id,
    className: row.className,
    gradeName: row.gradeName,
    subject: persisted?.subject ?? null,
  };
}
