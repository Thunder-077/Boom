import { useEffect, useMemo, useState } from "react";
import { BarChart3, Download } from "lucide-react";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import { revealInExplorer } from "../../../shared/utils/appLog";
import { FluentSelect, InfoHint, TableCard } from "../../../widgets/common/index.react";
import { useReactCourseManagementStore } from "../store";

function excelFileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || path;
}

function formatDate(value: string) {
  const date = new Date(`${value}T00:00:00`);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return `${month}月${day}日（${weekdays[date.getDay()]}）`;
}

export default function CourseWorkloadPanel() {
  const store = useReactCourseManagementStore();
  const { state } = store;
  const today = new Date().toISOString().slice(0, 10);
  const [startDate, setStartDate] = useState(today);
  const [endDate, setEndDate] = useState(today);
  const [startPeriodIndex, setStartPeriodIndex] = useState(1);
  const [endPeriodIndex, setEndPeriodIndex] = useState(12);
  const [selectedTeacher, setSelectedTeacher] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [feedbackType, setFeedbackType] = useState<"info" | "success" | "warning" | "error">("info");
  const [feedbackMessage, setFeedbackMessage] = useState("选择真实日期和节次范围后查看课时统计；导出文件包含明细和分类汇总两个 Sheet。");
  const [feedbackLinkPath, setFeedbackLinkPath] = useState("");
  const [feedbackLinkLabel, setFeedbackLinkLabel] = useState("");

  const importOptions = useMemo(
    () => state.imports.map((item) => ({ label: excelFileName(item.sourceFile), value: item.id })),
    [state.imports],
  );
  const summaries = state.workloadReport?.summaries ?? [];
  const details = state.workloadReport?.details ?? [];
  const periodOptions = useMemo(
    () => state.periods.map((period) => ({ label: period.sectionLabel ? `${period.sectionLabel} ${period.periodLabel}` : period.periodLabel, value: period.periodIndex })),
    [state.periods],
  );
  const totalLessons = summaries.reduce((sum, row) => sum + row.totalCount, 0);
  const substitutionLessons = summaries.reduce((sum, row) => sum + row.substitutionCount, 0);
  const filteredDetails = selectedTeacher ? details.filter((item) => item.teacherName === selectedTeacher) : details;

  useEffect(() => {
    if (hasDesktopRuntime()) {
      void store.loadOptions();
    }
  }, []);

  useEffect(() => {
    if (summaries.length > 0 && !summaries.some((row) => row.teacherName === selectedTeacher)) {
      setSelectedTeacher(summaries[0].teacherName);
    }
  }, [selectedTeacher, summaries]);

  function buildQuery() {
    return {
      startDate,
      endDate,
      startPeriodIndex: Math.max(1, Number(startPeriodIndex) || 1),
      endPeriodIndex: Math.max(1, Number(endPeriodIndex) || 99),
    };
  }

  async function loadReport() {
    setIsLoading(true);
    try {
      const report = await store.loadWorkloadReport(buildQuery());
      const count = report?.details.length ?? 0;
      setFeedbackType(count > 0 ? "success" : "warning");
      setFeedbackMessage(count > 0 ? `已生成 ${count} 条课时明细。` : "该范围内暂无课时数据。");
      setFeedbackLinkPath("");
      setFeedbackLinkLabel("");
    } catch (error) {
      setFeedbackType("error");
      setFeedbackMessage(error instanceof Error ? error.message : String(error));
      setFeedbackLinkPath("");
      setFeedbackLinkLabel("");
    } finally {
      setIsLoading(false);
    }
  }

  async function exportReport() {
    try {
      const result = await store.exportWorkloadReport(buildQuery());
      if (!result) return;
      setFeedbackType("success");
      setFeedbackMessage("课时统计明细已导出，点击");
      setFeedbackLinkPath(result.filePath);
      setFeedbackLinkLabel(excelFileName(result.filePath));
    } catch (error) {
      setFeedbackType("error");
      setFeedbackMessage(error instanceof Error ? error.message : String(error));
      setFeedbackLinkPath("");
      setFeedbackLinkLabel("");
    }
  }

  return (
    <section className="panel workload-panel">
      <TableCard title="课时统计" meta={`${summaries.length} 位教师，${totalLessons} 节课`}>
        <div className="workload">
          <div className="query-grid">
            <label className="control-field">
              <span>课表批次</span>
              <FluentSelect modelValue={state.selectedImportId ?? ""} options={importOptions} placeholder="未导入" onUpdateModelValue={(value) => void store.setSelectedImport(Number(value))} />
            </label>
            <label className="control-field">
              <span>开始日期</span>
              <input className="glass-input" type="date" value={startDate} onChange={(event) => setStartDate(event.target.value)} />
            </label>
            <label className="control-field">
              <span>开始节次</span>
              <FluentSelect modelValue={startPeriodIndex} options={periodOptions} onUpdateModelValue={(value) => setStartPeriodIndex(Number(value))} />
            </label>
            <label className="control-field">
              <span>结束日期</span>
              <input className="glass-input" type="date" value={endDate} onChange={(event) => setEndDate(event.target.value)} />
            </label>
            <label className="control-field">
              <span>结束节次</span>
              <FluentSelect modelValue={endPeriodIndex} options={periodOptions} onUpdateModelValue={(value) => setEndPeriodIndex(Number(value))} />
            </label>
            <button className="action-btn primary" type="button" disabled={isLoading} onClick={() => void loadReport()}>
              <BarChart3 size={18} />
              查看统计
            </button>
            <button className="action-btn secondary" type="button" disabled={state.exportingWorkload} onClick={() => void exportReport()}>
              <Download size={18} />
              导出 Excel
            </button>
          </div>

          <div className="feedback-block">
            <InfoHint
              type={feedbackType}
              text={feedbackMessage}
              linkLabel={feedbackLinkPath ? feedbackLinkLabel : ""}
              suffix={feedbackLinkPath ? "打开文件所在位置。" : ""}
              onClickLink={() => {
                if (feedbackLinkPath) {
                  void revealInExplorer(feedbackLinkPath);
                }
              }}
            />
          </div>

          <div className="stats-strip">
            <div className="stat-cell">
              <span>教师数</span>
              <strong>{summaries.length}</strong>
            </div>
            <div className="stat-cell">
              <span>总课时</span>
              <strong>{totalLessons}</strong>
            </div>
            <div className="stat-cell">
              <span>代课节数</span>
              <strong>{substitutionLessons}</strong>
            </div>
          </div>

          <div className="tables">
            <div className="summary-table-wrap">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>教师</th>
                    <th>早上</th>
                    <th>上午</th>
                    <th>下午</th>
                    <th>晚上</th>
                    <th>代课</th>
                    <th>合计</th>
                  </tr>
                </thead>
                <tbody>
                  {summaries.length === 0 ? (
                    <tr>
                      <td colSpan={7} className="empty-cell">暂无统计结果</td>
                    </tr>
                  ) : null}
                  {summaries.map((row) => (
                    <tr key={row.teacherName} className={row.teacherName === selectedTeacher ? "active" : ""} onClick={() => setSelectedTeacher(row.teacherName)}>
                      <td>{row.teacherName}</td>
                      <td>{row.morningReadingCount}</td>
                      <td>{row.morningCount}</td>
                      <td>{row.afternoonCount}</td>
                      <td>{row.eveningCount}</td>
                      <td>{row.substitutionCount}</td>
                      <td>{row.totalCount}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <div className="detail-table-wrap">
              <table className="data-table detail-table">
                <thead>
                  <tr>
                    <th>教师</th>
                    <th>日期</th>
                    <th>节次</th>
                    <th>类别</th>
                    <th>班级</th>
                    <th>科目</th>
                    <th>备注</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredDetails.length === 0 ? (
                    <tr>
                      <td colSpan={7} className="empty-cell">选择左侧教师查看课时明细</td>
                    </tr>
                  ) : null}
                  {filteredDetails.map((detail) => (
                    <tr key={`${detail.teacherName}-${detail.targetDate}-${detail.periodIndex}-${detail.className}-${detail.subject}`}>
                      <td>{detail.teacherName}</td>
                      <td>{formatDate(detail.targetDate)}</td>
                      <td>{detail.periodLabel}</td>
                      <td>{detail.category}</td>
                      <td>{detail.displayClassName}</td>
                      <td>{detail.subject}</td>
                      <td>{detail.remark || "--"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </TableCard>
    </section>
  );
}
