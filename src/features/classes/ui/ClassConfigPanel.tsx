import { useEffect, useMemo, useRef, useState } from "react";
import { CLASS_CONFIG_TYPE_OPTIONS, SUBJECT_OPTIONS, type ClassConfigRow, type ClassConfigType } from "../../../entities/class-config/model";
import { Subject } from "../../../entities/score/model";
import { useAppDialog } from "../../../shared/ui/appDialog";
import { Button, ConfigCard, EmptyState, Tag } from "../../../widgets/common/index.react";
import { useReactClassConfigStore } from "../store";

const subjectMap = new Map(SUBJECT_OPTIONS.map((subject) => [subject.value, subject]));

const leadingSubjects = [Subject.Chinese, Subject.Math].map((value) => subjectMap.get(value)!);
const foreignLanguageSubjects = [Subject.English, Subject.Russian, Subject.Japanese].map((value) => subjectMap.get(value)!);
const remainingSubjects = [
  Subject.Physics,
  Subject.Chemistry,
  Subject.Biology,
  Subject.Politics,
  Subject.History,
  Subject.Geography,
].map((value) => subjectMap.get(value)!);

function compactLocation(row: ClassConfigRow) {
  const building = row.building?.trim() || "未填楼号";
  const floor = row.floor?.trim() || "未填楼层";
  return `${building} / ${floor}`;
}

export default function ClassConfigPanel() {
  const store = useReactClassConfigStore();
  const { state } = store;
  const dialog = useAppDialog();
  const [searchKeyword, setSearchKeyword] = useState("");
  const [isSearchPanelOpen, setSearchPanelOpen] = useState(false);
  const [isInteractingWithSearchPanel, setIsInteractingWithSearchPanel] = useState(false);
  const searchInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    void store.setFilters({ configType: "teaching_class", gradeName: "", keyword: "" });
  }, []);

  const normalizedSearchKeyword = searchKeyword.trim().replace(/\s+/g, "").toLowerCase();
  const filteredRows = useMemo(() => {
    if (!normalizedSearchKeyword) {
      return state.rows;
    }
    return state.rows.filter((row) => {
      const text = [row.className, row.gradeName, row.roomLabel ?? "", row.building, row.floor]
        .join(" ")
        .replace(/\s+/g, "")
        .toLowerCase();
      return text.includes(normalizedSearchKeyword);
    });
  }, [normalizedSearchKeyword, state.rows]);

  const currentTypeLabel = state.filters.configType === "teaching_class" ? "教学班" : "考试教室";
  const searchPlaceholder =
    state.filters.configType === "teaching_class" ? "搜索班级名称、年级、楼层" : "搜索教室名称、标签、楼层";
  const listTitle = `${currentTypeLabel}列表`;
  const createButtonLabel = `新建${currentTypeLabel}`;
  const saveButtonLabel = state.mode === "new" ? "创建配置" : "保存修改";
  const nameLabel = state.filters.configType === "teaching_class" ? "班级名称" : "教室名称";
  const namePlaceholder = state.filters.configType === "teaching_class" ? "请输入班级名称" : "请输入教室名称";
  const editorDescription =
    state.mode === "new"
      ? `当前正在创建新的${currentTypeLabel}配置。`
      : `当前正在编辑 ${state.loadedClassName || "已选记录"}。`;
  const emptyListTitle = normalizedSearchKeyword ? "没有匹配结果" : `暂无${currentTypeLabel}配置`;
  const emptyListSummary = normalizedSearchKeyword ? "试试换个关键词，或者点击上方按钮创建新配置。" : "点击“新建”后即可开始录入。";

  async function confirmDiscardIfNeeded(nextAction: string) {
    if (!state.isDirty) {
      return true;
    }
    return dialog.confirm({
      title: "检测到未保存修改",
      summary: "继续操作前需要先放弃当前未保存的内容。",
      details: [
        `当前内容：${state.form.className || `未命名${currentTypeLabel}`}`,
        `后续操作：${nextAction}`,
      ],
      confirmText: "放弃修改",
      cancelText: "继续编辑",
    });
  }

  async function switchConfigType(configType: ClassConfigType) {
    if (state.filters.configType === configType) {
      return;
    }
    const allowSwitch = await confirmDiscardIfNeeded(`切换到${configType === "teaching_class" ? "教学班" : "考试教室"}`);
    if (!allowSwitch) {
      return;
    }
    setSearchKeyword("");
    setSearchPanelOpen(false);
    await store.setFilters({ configType, gradeName: "", keyword: "" });
  }

  async function createNewConfig() {
    const allowCreate = await confirmDiscardIfNeeded(`新建${currentTypeLabel}`);
    if (!allowCreate) {
      return;
    }
    setSearchPanelOpen(false);
    store.startCreate("");
  }

  async function selectRow(row: ClassConfigRow) {
    if (state.mode === "existing" && state.editingId === row.id) {
      setSearchPanelOpen(false);
      return;
    }
    const allowSwitch = await confirmDiscardIfNeeded(`切换到 ${row.className}`);
    if (!allowSwitch) {
      return;
    }
    await store.loadDetail(row.id);
    setSearchPanelOpen(false);
  }

  async function saveCurrent() {
    const gradeName = state.form.gradeName.trim();
    const className = state.form.className.trim();
    if (!gradeName || !className) {
      const details: string[] = [];
      if (!gradeName) details.push("请先填写年级。");
      if (!className) details.push(`请先填写${nameLabel}。`);
      await dialog.alert({
        title: "缺少必填信息",
        summary: "当前配置还不能保存。",
        details,
      });
      return;
    }

    const wasCreating = state.mode === "new";
    try {
      if (wasCreating) {
        await store.create();
      } else {
        await store.update();
      }
      await dialog.alert({
        title: "保存成功",
        summary: wasCreating ? "配置已创建。" : "配置已更新。",
        details: [`名称：${state.form.className || `未命名${currentTypeLabel}`}`],
      });
    } catch {
      // Store already exposes error state.
    }
  }

  async function deleteCurrent() {
    if (!state.editingId) {
      return;
    }
    const confirmed = await dialog.confirm({
      title: "确认删除",
      summary: "删除后当前配置无法恢复，请确认是否继续。",
      details: [`当前记录：${state.form.className || `未命名${currentTypeLabel}`}`],
      confirmText: "确认删除",
      cancelText: "取消",
    });
    if (!confirmed) {
      return;
    }
    try {
      await store.remove(state.editingId);
      await dialog.alert({
        title: "删除成功",
        summary: "当前配置已经删除。",
      });
    } catch {
      // Store already exposes error state.
    }
  }

  return (
    <section className="class-config-page">
      <section className="workspace">
        <aside className="sidebar card-shell">
          <div className="sidebar-toolbar">
            <div className="type-switch">
              {CLASS_CONFIG_TYPE_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  className={`type-btn ${state.filters.configType === option.value ? "active" : ""}`.trim()}
                  onClick={() => void switchConfigType(option.value)}
                >
                  {option.label}
                </button>
              ))}
            </div>

            <div className="search-anchor">
              <div className="search-wrap">
                <span className="material-symbols-rounded search-icon" aria-hidden="true">search</span>
                <input
                  ref={searchInputRef}
                  value={searchKeyword}
                  className="search-input"
                  placeholder={searchPlaceholder}
                  onChange={(event) => setSearchKeyword(event.target.value)}
                  onFocus={() => setSearchPanelOpen(true)}
                  onClick={() => setSearchPanelOpen(true)}
                  onBlur={() => {
                    window.setTimeout(() => {
                      if (isInteractingWithSearchPanel) {
                        setIsInteractingWithSearchPanel(false);
                        searchInputRef.current?.focus();
                        return;
                      }
                      setSearchPanelOpen(false);
                    }, 0);
                  }}
                />
              </div>

              {isSearchPanelOpen ? (
                <div className="search-panel" onMouseDown={(event) => {
                  event.preventDefault();
                  setIsInteractingWithSearchPanel(true);
                }}>
                  <div className="list-head">
                    <strong>{listTitle}</strong>
                    <span>{filteredRows.length} / {state.rows.length}</span>
                  </div>

                  <div className="config-list">
                    {filteredRows.map((row) => (
                      <button
                        key={row.id}
                        type="button"
                        className={`config-item ${row.id === state.editingId && state.mode === "existing" ? "active" : ""}`.trim()}
                        onClick={() => void selectRow(row)}
                      >
                        <div className="config-item-top">
                          <strong>{row.className}</strong>
                          <Tag variant="primary" size="sm">{row.gradeName || currentTypeLabel}</Tag>
                        </div>
                        <p className="config-item-subtitle">{compactLocation(row)}</p>
                      </button>
                    ))}

                    {filteredRows.length === 0 ? (
                      <EmptyState title={emptyListTitle} description={emptyListSummary} />
                    ) : null}
                  </div>
                </div>
              ) : null}
            </div>

            <Button variant="primary" className="create-btn" onClick={() => void createNewConfig()}>
              <span className="material-symbols-rounded" aria-hidden="true">add</span>
              {createButtonLabel}
            </Button>
          </div>
        </aside>

        <div className="editor">
          <ConfigCard title="基础信息" description={editorDescription}>
            <div className="form-grid">
              <label className="field">
                <span className="field-label">年级</span>
                <input
                  className="field-input"
                  value={state.form.gradeName}
                  placeholder="例如：高一"
                  list="class-grade-options"
                  onChange={(event) => store.setFormField("gradeName", event.target.value)}
                />
              </label>

              <label className="field field-wide">
                <span className="field-label">{nameLabel}</span>
                <input
                  className="field-input field-input-strong"
                  value={state.form.className}
                  placeholder={namePlaceholder}
                  onChange={(event) => store.setFormField("className", event.target.value)}
                />
              </label>

              <label className="field">
                <span className="field-label">教室标签</span>
                <input
                  className="field-input"
                  value={state.form.roomLabel || ""}
                  placeholder="例如：X-505"
                  onChange={(event) => store.setFormField("roomLabel", event.target.value || null)}
                />
              </label>
            </div>
            <datalist id="class-grade-options">
              {state.gradeOptions.map((grade) => (
                <option key={grade} value={grade} />
              ))}
            </datalist>
          </ConfigCard>

          {state.form.configType === "teaching_class" ? (
            <ConfigCard
              title="所学科目"
              description={`当前已选择 ${state.form.subjects.length} 门课程。`}
            >
              <div className="subject-grid">
                {leadingSubjects.map((subject) => (
                  <button
                    key={subject.value}
                    type="button"
                    className={`subject-chip ${state.form.subjects.includes(subject.value) ? "active" : ""}`.trim()}
                    onClick={() => store.toggleSubject(subject.value, !state.form.subjects.includes(subject.value))}
                  >
                    {subject.label}
                  </button>
                ))}

                <div className="subject-curve-group">
                  {foreignLanguageSubjects.map((subject) => (
                    <button
                      key={subject.value}
                      type="button"
                      className={`subject-chip ${state.form.subjects.includes(subject.value) ? "active" : ""}`.trim()}
                      onClick={() => store.toggleSubject(subject.value, !state.form.subjects.includes(subject.value))}
                    >
                      {subject.label}
                    </button>
                  ))}
                </div>

                {remainingSubjects.map((subject) => (
                  <button
                    key={subject.value}
                    type="button"
                    className={`subject-chip ${state.form.subjects.includes(subject.value) ? "active" : ""}`.trim()}
                    onClick={() => store.toggleSubject(subject.value, !state.form.subjects.includes(subject.value))}
                  >
                    {subject.label}
                  </button>
                ))}
              </div>
            </ConfigCard>
          ) : null}

          <ConfigCard title="教室位置">
            <div className="form-grid form-grid-compact">
              <label className="field">
                <span className="field-label">楼号</span>
                <input
                  className="field-input"
                  value={state.form.building}
                  placeholder="例如：向远楼"
                  onChange={(event) => store.setFormField("building", event.target.value)}
                />
              </label>

              <label className="field">
                <span className="field-label">楼层</span>
                <input
                  className="field-input"
                  value={state.form.floor}
                  placeholder="例如：3层"
                  onChange={(event) => store.setFormField("floor", event.target.value)}
                />
              </label>
            </div>

            <div className="config-footer">
              <Button
                variant="danger"
                disabled={state.mode !== "existing" || state.deleting}
                onClick={() => void deleteCurrent()}
              >
                {state.deleting ? "删除中..." : "删除教室"}
              </Button>
              <Button variant="primary" disabled={state.saving} onClick={() => void saveCurrent()}>
                {state.saving ? "保存中..." : saveButtonLabel}
              </Button>
            </div>
            {state.errorMessage ? <p className="error-text">{state.errorMessage}</p> : null}
          </ConfigCard>
        </div>
      </section>
    </section>
  );
}
