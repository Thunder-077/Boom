import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invigilationService } from "../../invigilation/service";
import type {
  AnimationPhase,
  DrawAssignment,
  DrawStatus,
  ImportedInvigilatorRow,
  InvigilatorPair,
  PairMode,
  RoomItem,
  StepKey,
} from "../model";

const FLOW_STEPS = [
  { key: "rooms", label: "录入考场" },
  { key: "import", label: "导入监考员" },
  { key: "mode", label: "选择方式" },
  { key: "draw", label: "抽签中" },
  { key: "result", label: "结果" },
] as const;

const DRAW_PHASES = [
  { key: "pair_rolling", label: "结对翻牌" },
  { key: "room_fast", label: "考场快转" },
  { key: "room_slow", label: "减速定位" },
  { key: "room_hit", label: "命中确认" },
  { key: "card_slide", label: "结果落位" },
] as const;

const DRAW_PHASE_RANK: Record<AnimationPhase, number> = {
  idle: -1,
  pair_rolling: 0,
  room_fast: 1,
  room_slow: 2,
  room_hit: 3,
  card_slide: 4,
  completed: 4,
};

function sleep(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function shuffle<T>(list: T[]) {
  const next = [...list];
  for (let index = next.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [next[index], next[swapIndex]] = [next[swapIndex], next[index]];
  }
  return next;
}

function normalizeDroppedPath(raw: string) {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("file://")) return trimmed;
  try {
    const url = new URL(trimmed);
    return decodeURIComponent(url.pathname)
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  } catch {
    return decodeURIComponent(trimmed.replace(/^file:\/\//i, ""))
      .replace(/^\/([A-Za-z]:\/)/, "$1")
      .replace(/\//g, "\\");
  }
}

function pickExcelPath(paths: string[]) {
  for (const path of paths) {
    const normalized = normalizeDroppedPath(path);
    const lowerPath = normalized.toLowerCase();
    if (lowerPath.endsWith(".xlsx") || lowerPath.endsWith(".xls")) {
      return normalized;
    }
  }
  return undefined;
}

export default function MonitorDrawPanel() {
  const [step, setStep] = useState<StepKey>("home");
  const [selectedMode, setSelectedMode] = useState<PairMode | null>(null);
  const [roomsInput, setRoomsInput] = useState("");
  const [validatedRooms, setValidatedRooms] = useState<RoomItem[]>([]);
  const [importedRows, setImportedRows] = useState<ImportedInvigilatorRow[]>([]);
  const [roomError, setRoomError] = useState("");
  const [modeError, setModeError] = useState("");
  const [importStatus, setImportStatus] = useState<"idle" | "importing" | "success" | "error">("idle");
  const [importMessage, setImportMessage] = useState("请拖拽导入 Excel。");
  const [isDragging, setIsDragging] = useState(false);
  const [drawQueue, setDrawQueue] = useState<InvigilatorPair[]>([]);
  const [finalAssignments, setFinalAssignments] = useState<DrawAssignment[]>([]);
  const [isDrawPaused, setIsDrawPaused] = useState(false);
  const [drawCancelRequested, setDrawCancelRequested] = useState(false);
  const [isAllRoomsExpanded, setIsAllRoomsExpanded] = useState(false);
  const [rollingNameA, setRollingNameA] = useState("—");
  const [rollingNameB, setRollingNameB] = useState("—");
  const [isFlipA, setIsFlipA] = useState(false);
  const [isFlipB, setIsFlipB] = useState(false);
  const [drawStartedAt, setDrawStartedAt] = useState(0);
  const [drawEndedAt, setDrawEndedAt] = useState(0);
  const [recentRedrawId, setRecentRedrawId] = useState("");
  const [drawStatus, setDrawStatus] = useState<DrawStatus>({
    phase: "idle",
    isDrawing: false,
    progress: 0,
    currentPairIndex: -1,
    currentHighlightRoomIndex: -1,
  });
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const drawStatusRef = useRef(drawStatus);
  const isDrawPausedRef = useRef(isDrawPaused);
  const drawCancelRequestedRef = useRef(drawCancelRequested);

  useEffect(() => {
    drawStatusRef.current = drawStatus;
  }, [drawStatus]);

  useEffect(() => {
    isDrawPausedRef.current = isDrawPaused;
  }, [isDrawPaused]);

  useEffect(() => {
    drawCancelRequestedRef.current = drawCancelRequested;
  }, [drawCancelRequested]);

  const parsedRoomsPreview = useMemo(
    () =>
      Array.from(
        new Set(
          roomsInput
            .split(/[\n,，]+/g)
            .map((item) => item.trim())
            .filter(Boolean),
        ),
      ).map((roomNo) => ({
        id: `room-${roomNo}`,
        roomNo,
      })),
    [roomsInput],
  );

  const roomPreviewStats = useMemo(() => {
    const raw = roomsInput.split(/[\n,，]+/g).map((item) => item.trim());
    const inputCount = raw.length;
    const validList = raw.filter(Boolean);
    const validCount = new Set(validList).size;
    const duplicateCount = Math.max(validList.length - validCount, 0);
    const emptyCount = Math.max(inputCount - validList.length, 0);
    return { inputCount, validCount, duplicateCount, emptyCount };
  }, [roomsInput]);

  const assignedRoomNos = useMemo(
    () => new Set(finalAssignments.filter((item) => item.assigned).map((item) => item.roomNo)),
    [finalAssignments],
  );
  const resultByRoom = useMemo(
    () => new Map(finalAssignments.filter((item) => item.assigned).map((item) => [item.roomNo, item])),
    [finalAssignments],
  );
  const currentPair = drawQueue[drawStatus.currentPairIndex] ?? null;
  const currentPairText = currentPair ? `${currentPair.invigilatorA} × ${currentPair.invigilatorB}` : "等待开始";
  const phaseLabel =
    {
      idle: "待机",
      pair_rolling: "随机结对",
      room_fast: "快转",
      room_slow: "减速",
      room_hit: "命中",
      card_slide: "落位",
      completed: "完成",
    }[drawStatus.phase];
  const focusWindowSize = 7;
  const focusStartIndex = Math.max(
    0,
    Math.min(
      (drawStatus.currentHighlightRoomIndex < 0 ? 0 : drawStatus.currentHighlightRoomIndex) - Math.floor(focusWindowSize / 2),
      Math.max(0, validatedRooms.length - focusWindowSize),
    ),
  );
  const focusRooms = validatedRooms
    .slice(focusStartIndex, focusStartIndex + focusWindowSize)
    .map((room, index) => ({ room, index: focusStartIndex + index }));
  const invigilatorAList = importedRows.map((item) => item.invigilatorAName);
  const invigilatorBList = importedRows.map((item) => item.invigilatorBName);
  const totalRoomCount = validatedRooms.length;
  const assignedRoomCount = finalAssignments.filter((item) => item.assigned).length;
  const pendingRoomCount = Math.max(totalRoomCount - assignedRoomCount, 0);
  const currentHighlightRoomNo =
    drawStatus.currentHighlightRoomIndex < 0 ? "" : validatedRooms[drawStatus.currentHighlightRoomIndex]?.roomNo ?? "";
  const importPreviewStats = {
    totalRows: importedRows.length,
    coverableRooms: Math.min(importedRows.length, validatedRooms.length),
    shortfall: Math.max(validatedRooms.length - importedRows.length, 0),
    statusText:
      importStatus === "error"
        ? "导入失败"
        : importStatus === "success"
          ? "导入成功"
          : importStatus === "importing"
            ? "导入中"
            : "等待导入",
  };
  const flowActiveIndex =
    step === "home" ? 0 : Math.max(0, FLOW_STEPS.findIndex((item) => item.key === step));
  const drawPhaseIndex = DRAW_PHASE_RANK[drawStatus.phase];
  const resultSummary = {
    total: finalAssignments.length,
    fixedCount: finalAssignments.filter((item) => item.pairMode === "fixed").length,
    randomCount: Math.max(finalAssignments.length - finalAssignments.filter((item) => item.pairMode === "fixed").length, 0),
    durationText: drawEndedAt > drawStartedAt ? `${((drawEndedAt - drawStartedAt) / 1000).toFixed(1)}s` : "—",
  };

  async function waitIfPaused() {
    while (isDrawPausedRef.current && !drawCancelRequestedRef.current) {
      await sleep(90);
    }
  }

  function buildQueue() {
    const count = validatedRooms.length;
    if (selectedMode === "fixed") {
      return shuffle(importedRows)
        .slice(0, count)
        .map((row, index) => ({
          id: `f-${index}`,
          groupNo: row.groupNo,
          invigilatorA: row.invigilatorAName,
          invigilatorB: row.invigilatorBName,
          mode: "fixed" as PairMode,
        }));
    }
    const aList = shuffle(invigilatorAList);
    const bList = shuffle(invigilatorBList);
    return Array.from({ length: count }).map((_, index) => ({
      id: `r-${index}`,
      groupNo: `R${index + 1}`,
      invigilatorA: aList[index],
      invigilatorB: bList[index],
      mode: "random" as PairMode,
    }));
  }

  function initAssignments(queue: InvigilatorPair[]) {
    const rooms = shuffle(validatedRooms);
    return queue.map((pair, index) => ({
      id: `a-${index}`,
      roomNo: rooms[index].roomNo,
      invigilatorA: pair.invigilatorA,
      invigilatorB: pair.invigilatorB,
      pairMode: pair.mode,
      statusTag: "待分配" as const,
      assigned: false,
    }));
  }

  async function marqueeTo(target: number, speeds: number[]) {
    const total = validatedRooms.length;
    let currentIndex = Math.max(0, drawStatusRef.current.currentHighlightRoomIndex);
    for (const speed of speeds) {
      if (drawCancelRequestedRef.current) return false;
      await waitIfPaused();
      currentIndex = (currentIndex + 1) % total;
      setDrawStatus((current) => ({ ...current, currentHighlightRoomIndex: currentIndex }));
      await sleep(speed);
    }
    while (currentIndex !== target) {
      if (drawCancelRequestedRef.current) return false;
      await waitIfPaused();
      currentIndex = (currentIndex + 1) % total;
      setDrawStatus((current) => ({ ...current, currentHighlightRoomIndex: currentIndex }));
      await sleep(speeds[speeds.length - 1]);
    }
    return true;
  }

  async function playRandomRolling(pair: InvigilatorPair) {
    setDrawStatus((current) => ({ ...current, phase: "pair_rolling" }));
    const aList = shuffle(invigilatorAList);
    const bList = shuffle(invigilatorBList);
    setIsFlipA(false);
    setIsFlipB(false);
    for (let index = 0; index < 5; index += 1) {
      if (drawCancelRequestedRef.current) return false;
      await waitIfPaused();
      setRollingNameA(aList[index % aList.length] ?? pair.invigilatorA);
      setRollingNameB(bList[index % bList.length] ?? pair.invigilatorB);
      setIsFlipA((current) => !current);
      setIsFlipB((current) => !current);
      await sleep(55);
    }
    setRollingNameA(pair.invigilatorA);
    setRollingNameB(pair.invigilatorB);
    setIsFlipA(true);
    setIsFlipB(true);
    await sleep(90);
    return !drawCancelRequestedRef.current;
  }

  function cancelDrawNow() {
    setDrawStatus({
      phase: "idle",
      isDrawing: false,
      progress: 0,
      currentPairIndex: -1,
      currentHighlightRoomIndex: -1,
    });
    setIsDrawPaused(false);
    setDrawQueue([]);
    setFinalAssignments([]);
    setDrawCancelRequested(false);
    setIsFlipA(false);
    setIsFlipB(false);
    setDrawStartedAt(0);
    setDrawEndedAt(0);
  }

  async function startDraw() {
    if (drawStatusRef.current.isDrawing) return;
    const queue = buildQueue();
    const assignments = initAssignments(queue);
    setDrawQueue(queue);
    setFinalAssignments(assignments);
    setDrawStartedAt(Date.now());
    setDrawEndedAt(0);
    setDrawCancelRequested(false);
    setDrawStatus({
      phase: "idle",
      isDrawing: true,
      progress: 0,
      currentPairIndex: -1,
      currentHighlightRoomIndex: -1,
    });

    for (let index = 0; index < assignments.length; index += 1) {
      if (drawCancelRequestedRef.current) {
        cancelDrawNow();
        return;
      }
      setDrawStatus((current) => ({ ...current, currentPairIndex: index }));
      const pair = queue[index];
      if (selectedMode === "random") {
        const done = await playRandomRolling(pair);
        if (!done) {
          cancelDrawNow();
          return;
        }
      } else {
        setRollingNameA(pair.invigilatorA);
        setRollingNameB(pair.invigilatorB);
      }

      const targetIndex = validatedRooms.findIndex((room) => room.roomNo === assignments[index].roomNo);
      setDrawStatus((current) => ({ ...current, phase: "room_fast" }));
      if (!(await marqueeTo(targetIndex, [35, 35, 40, 45, 50, 55]))) {
        cancelDrawNow();
        return;
      }
      setDrawStatus((current) => ({ ...current, phase: "room_slow" }));
      if (!(await marqueeTo(targetIndex, [80, 95, 110, 130]))) {
        cancelDrawNow();
        return;
      }
      setDrawStatus((current) => ({ ...current, phase: "room_hit" }));
      setFinalAssignments((current) => current.map((item, currentIndex) => currentIndex === index ? { ...item, statusTag: "正在抽取" } : item));
      await sleep(140);
      if (drawCancelRequestedRef.current) {
        cancelDrawNow();
        return;
      }
      setDrawStatus((current) => ({
        ...current,
        phase: "card_slide",
        progress: Math.round(((index + 1) / assignments.length) * 100),
      }));
      setFinalAssignments((current) => current.map((item, currentIndex) => currentIndex === index ? { ...item, assigned: true, statusTag: "已落位" } : item));
      await sleep(180);
    }

    setDrawStatus((current) => ({ ...current, phase: "completed", isDrawing: false }));
    setDrawEndedAt(Date.now());
    await sleep(120);
    setStep("result");
  }

  function confirmRooms() {
    setRoomError("");
    if (parsedRoomsPreview.length === 0) {
      setRoomError("请至少录入 1 个考场号。");
      return;
    }
    setValidatedRooms(parsedRoomsPreview);
    setStep("import");
  }

  function goNextFromMode() {
    setModeError("");
    const need = validatedRooms.length;
    if (!selectedMode) {
      setModeError("请先选择结对方式。");
      return;
    }
    if (selectedMode === "fixed" && importedRows.length < need) {
      setModeError(`固定结对不足：需要 ${need} 对。`);
      return;
    }
    if (selectedMode === "random" && (invigilatorAList.length < need || invigilatorBList.length < need)) {
      setModeError(`随机分配人数不足：甲乙各需 ${need} 人。`);
      return;
    }
    setStep("draw");
  }

  function redrawOne(index: number) {
    if (finalAssignments.length < 2) return;
    const candidates = finalAssignments.map((_, currentIndex) => currentIndex).filter((currentIndex) => currentIndex !== index);
    const swapIndex = candidates[Math.floor(Math.random() * candidates.length)];
    const next = [...finalAssignments];
    const a = next[index];
    const b = next[swapIndex];
    next[index] = { ...a, invigilatorA: b.invigilatorA, invigilatorB: b.invigilatorB, pairMode: b.pairMode, assigned: true, statusTag: "已落位" };
    next[swapIndex] = { ...b, invigilatorA: a.invigilatorA, invigilatorB: a.invigilatorB, pairMode: a.pairMode, assigned: true, statusTag: "已落位" };
    setFinalAssignments(next);
    setRecentRedrawId(next[index].id);
    window.setTimeout(() => {
      setRecentRedrawId((current) => (current === next[index].id ? "" : current));
    }, 1400);
  }

  async function redrawAll() {
    setStep("draw");
    await sleep(0);
    await startDraw();
  }

  function restartFlow() {
    setStep("home");
    setSelectedMode(null);
    setImportedRows([]);
    setValidatedRooms([]);
    setRoomsInput("");
    setModeError("");
    setRoomError("");
    setImportStatus("idle");
    setImportMessage("请拖拽导入 Excel。");
    setIsAllRoomsExpanded(false);
    cancelDrawNow();
  }

  function exportCsv() {
    const rows = [
      ["考场号", "监考员甲", "监考员乙", "结对方式"],
      ...finalAssignments.map((item) => [
        item.roomNo,
        item.invigilatorA,
        item.invigilatorB,
        item.pairMode === "fixed" ? "固定结对" : "随机分配",
      ]),
    ];
    const csv = rows.map((row) => row.map((cell) => `"${String(cell).split("\"").join("\"\"")}"`).join(",")).join("\n");
    const blob = new Blob([`\uFEFF${csv}`], { type: "text/csv;charset=utf-8;" });
    const anchor = document.createElement("a");
    const url = URL.createObjectURL(blob);
    anchor.href = url;
    anchor.download = `监考抽签结果-${new Date().toISOString().split(":").join("-")}.csv`;
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  }

  function roomStatusLabel(roomNo: string, roomIndex: number) {
    if (assignedRoomNos.has(roomNo)) return "已落位";
    if (drawStatus.currentHighlightRoomIndex === roomIndex) return "进行中";
    return "未分配";
  }

  async function handleImport(filePath: string) {
    setImportStatus("importing");
    setImportMessage("正在导入...");
    try {
      const result = await invigilationService.importMonitorDrawPairsExcel(filePath);
      setImportedRows(result.rows);
      setImportStatus("success");
      setImportMessage(`导入成功：${result.rowCount} 组，耗时 ${result.durationMs}ms`);
    } catch (error) {
      setImportedRows([]);
      setImportStatus("error");
      setImportMessage(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    let unlistenDragDrop: (() => void) | null = null;

    async function bindWindowEvents() {
      const appWindow = getCurrentWebviewWindow();
      unlistenDragDrop = await appWindow.onDragDropEvent((event) => {
        if (step !== "import") return;
        if (event.payload.type === "enter" || event.payload.type === "over") {
          setIsDragging(true);
          return;
        }
        if (event.payload.type === "leave") {
          setIsDragging(false);
          return;
        }
        if (event.payload.type === "drop") {
          setIsDragging(false);
          const excelPath = pickExcelPath(event.payload.paths);
          if (excelPath) {
            void handleImport(excelPath);
            return;
          }
          setImportStatus("error");
          setImportMessage("未识别到 Excel 文件路径。");
        }
      });
    }

    void bindWindowEvents();
    return () => {
      unlistenDragDrop?.();
    };
  }, [step]);

  return (
    <section className={`draw-panel ${isDragging && step === "import" ? "dragging" : ""}`}>
      <div className="steps-rail card-shell">
        {FLOW_STEPS.map((item, index) => (
          <div key={item.key} className={`flow-step ${flowActiveIndex === index ? "active" : ""} ${flowActiveIndex > index ? "done" : ""}`.trim()}>
            <span className="step-index">{index + 1}</span>
            <span className="step-label">{item.label}</span>
          </div>
        ))}
      </div>

      {step === "home" ? (
        <div className="card-shell page-card home-card">
          <span className="section-kicker">开始前</span>
          <h3 className="page-title">监考抽签</h3>
          <p className="lead">为每个考场一次性分配监考员甲/乙，支持固定结对和随机结对。</p>
          <p className="desc">流程：录入考场 → 导入监考员 → 选择方式 → 批量抽签 → 查看结果</p>
          <div className="prepare-list">
            <span className="pill">准备考场</span>
            <span className="pill">准备监考员Excel</span>
            <span className="pill">确认结对方式（固定或随机）</span>
          </div>
          <div className="actions">
            <button className="primary-btn" type="button" onClick={() => setStep("rooms")}>开始抽签</button>
          </div>
        </div>
      ) : null}

      {step === "rooms" ? (
        <div className="grid-two">
          <div className="card-shell page-card">
            <span className="section-kicker">步骤一</span>
            <h3 className="section-title">录入考场</h3>
            <p className="desc">每行一个考场号，例如 A101。</p>
            <textarea value={roomsInput} className="glass-area room-input" placeholder={"A101\nA102\nA103"} onChange={(event) => setRoomsInput(event.target.value)} />
            {roomError ? <p className="desc danger">{roomError}</p> : null}
            <div className="actions">
              <button className="secondary-btn" type="button" onClick={() => setStep("home")}>上一步</button>
              <button className="primary-btn" type="button" onClick={confirmRooms}>确认考场并继续</button>
            </div>
          </div>
          <div className="card-shell page-card">
            <span className="section-kicker">预览</span>
            <h3 className="section-title">实时预览</h3>
            <div className="room-stats-grid compact">
              <div className="stat-cell"><span>有效考场</span><strong>{roomPreviewStats.validCount}</strong></div>
              <div className="stat-cell"><span>重复条目</span><strong>{roomPreviewStats.duplicateCount}</strong></div>
              <div className="stat-cell"><span>输入条目</span><strong>{roomPreviewStats.inputCount}</strong></div>
              <div className="stat-cell"><span>空白条目</span><strong>{roomPreviewStats.emptyCount}</strong></div>
            </div>
            <div className="row-list">
              {parsedRoomsPreview.map((room) => <span key={room.id} className="pill">{room.roomNo}</span>)}
            </div>
          </div>
        </div>
      ) : null}

      {step === "import" ? (
        <div className="grid-two">
          <div className="card-shell page-card">
            <span className="section-kicker">步骤二</span>
            <h3 className="section-title">导入监考名单</h3>
            <div className={`drop-zone ${isDragging ? "active" : ""}`} onClick={() => fileInputRef.current?.click()}>
              <strong>{isDragging ? "松开鼠标即可导入 Excel" : "拖拽 Excel 到此处以导入监考人员名单"}</strong>
              <span className="drop-hint">{isDragging ? "已检测到文件，释放后开始解析" : "或点击此区域选择文件（.xlsx / .xls）"}</span>
            </div>
            <input
              ref={fileInputRef}
              className="hidden-file"
              type="file"
              accept=".xlsx,.xls"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (!file) return;
                const fullPath = (file as File & { path?: string }).path || "";
                if (fullPath && (fullPath.toLowerCase().endsWith(".xlsx") || fullPath.toLowerCase().endsWith(".xls"))) {
                  void handleImport(fullPath);
                } else {
                  setImportStatus("error");
                  setImportMessage("当前环境未获取到本地路径，请优先使用拖拽导入。");
                }
                event.target.value = "";
              }}
            />
            <p className={`desc ${importStatus === "error" ? "danger" : ""}`.trim()}>{importMessage}</p>
            <div className="actions">
              <button className="secondary-btn" type="button" onClick={() => setStep("rooms")}>上一步</button>
              <button className="primary-btn" type="button" disabled={importedRows.length === 0} onClick={() => setStep("mode")}>进入方式选择</button>
            </div>
          </div>
          <div className="card-shell page-card">
            <span className="section-kicker">导入结果</span>
            <h3 className="section-title">导入预览（{importedRows.length}）</h3>
            <div className="room-stats-grid compact">
              <div className="stat-cell"><span>导入组数</span><strong>{importPreviewStats.totalRows}</strong></div>
              <div className="stat-cell"><span>可覆盖考场</span><strong>{importPreviewStats.coverableRooms}</strong></div>
              <div className="stat-cell"><span>待补数量</span><strong>{importPreviewStats.shortfall}</strong></div>
              <div className="stat-cell"><span>当前状态</span><strong>{importPreviewStats.statusText}</strong></div>
            </div>
            <div className="table-wrap small">
              <table className="table">
                <thead><tr><th>组号</th><th>监考员甲</th><th>监考员乙</th></tr></thead>
                <tbody>
                  {importedRows.map((row) => (
                    <tr key={row.groupNo}><td>{row.groupNo}</td><td>{row.invigilatorAName}</td><td>{row.invigilatorBName}</td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      ) : null}

      {step === "mode" ? (
        <div className="card-shell page-card">
          <span className="section-kicker">步骤三</span>
          <h3 className="section-title">选择结对方式</h3>
          <p className="desc">当前选择会影响抽签过程与结果展示。</p>
          <div className="mode-grid">
            <button className={`mode-card ${selectedMode === "fixed" ? "active" : ""}`.trim()} onClick={() => setSelectedMode("fixed")}>
              <strong>固定结对（按导入）</strong>
              <span>保持导入的甲乙关系，仅对考场进行随机分配。</span>
            </button>
            <button className={`mode-card ${selectedMode === "random" ? "active" : ""}`.trim()} onClick={() => setSelectedMode("random")}>
              <strong>随机结对（甲乙重组）</strong>
              <span>从甲池和乙池分别抽取，随机组成新结对后分配考场。</span>
            </button>
          </div>
          {modeError ? <p className="error-text">{modeError}</p> : null}
          <div className="actions">
            <button className="secondary-btn" type="button" onClick={() => setStep("import")}>上一步</button>
            <button className="primary-btn" type="button" onClick={goNextFromMode}>开始批量抽签</button>
          </div>
        </div>
      ) : null}

      {step === "draw" ? (
        <div className="grid-two">
          <div className="card-shell page-card draw-main-card">
            <span className="section-kicker">步骤四</span>
            <h3 className="section-title">{selectedMode === "fixed" ? "固定结对抽签中" : "随机结对抽签中"}</h3>
            <p className="desc">当前阶段：{phaseLabel} · 已完成 {drawStatus.progress}%</p>
            <div className="phase-track">
              {DRAW_PHASES.map((item, index) => (
                <div key={item.key} className={`phase-step ${drawPhaseIndex === index ? "active" : ""} ${drawPhaseIndex > index ? "done" : ""}`.trim()}>
                  {item.label}
                </div>
              ))}
            </div>
            <p className="draw-counter">当前进度：第 {Math.max(drawStatus.currentPairIndex + 1, 0)} / {Math.max(drawQueue.length, 0)} 对</p>
            {selectedMode === "random" ? (
              <div className="rolling-pair">
                <div className={`flip-card ${isFlipA ? "flipped" : ""}`.trim()}>
                  <div className="flip-inner">
                    <div className="flip-face flip-front">监考员甲</div>
                    <div className="flip-face flip-back">{rollingNameA}</div>
                  </div>
                </div>
                <span className="pair-multiplier">×</span>
                <div className={`flip-card ${isFlipB ? "flipped" : ""}`.trim()}>
                  <div className="flip-inner">
                    <div className="flip-face flip-front">监考员乙</div>
                    <div className="flip-face flip-back">{rollingNameB}</div>
                  </div>
                </div>
              </div>
            ) : null}
            <div className="current-pair">{currentPairText}</div>
            <div className="actions">
              <button className="secondary-btn" disabled={drawStatus.isDrawing} onClick={() => setStep("mode")}>返回</button>
              <button className="secondary-btn" disabled={!drawStatus.isDrawing} onClick={() => setIsDrawPaused((current) => !current)}>{isDrawPaused ? "继续" : "暂停"}</button>
              <button className="secondary-btn" disabled={!drawStatus.isDrawing} onClick={() => setDrawCancelRequested(true)}>取消</button>
              <button className="primary-btn" disabled={drawStatus.isDrawing} onClick={() => void startDraw()}>开始抽签</button>
            </div>
          </div>

          <div className="card-shell page-card draw-rooms-card">
            <span className="section-kicker">当前考场</span>
            <h3 className="section-title">当前抽签考场</h3>
            <div className="draw-visual-layout">
              <section className="focus-panel">
                <div className="focus-window">
                  {focusRooms.map((item) => (
                    <div
                      key={item.room.id}
                      className={`room-row ${drawStatus.currentHighlightRoomIndex === item.index ? "highlight centered" : ""} ${assignedRoomNos.has(item.room.roomNo) ? "hit" : ""}`.trim()}
                    >
                      <span>{item.room.roomNo}</span>
                      {resultByRoom.get(item.room.roomNo) ? (
                        <span>{resultByRoom.get(item.room.roomNo)?.invigilatorA} × {resultByRoom.get(item.room.roomNo)?.invigilatorB}</span>
                      ) : null}
                    </div>
                  ))}
                </div>
              </section>

              <section className="all-rooms-panel">
                <div className="room-stats-grid">
                  <div className="stat-cell"><span>总考场</span><strong>{totalRoomCount}</strong></div>
                  <div className="stat-cell"><span>已分配</span><strong>{assignedRoomCount}</strong></div>
                  <div className="stat-cell"><span>未分配</span><strong>{pendingRoomCount}</strong></div>
                  <div className="stat-cell"><span>当前命中</span><strong>{currentHighlightRoomNo || "—"}</strong></div>
                </div>
                <button className="secondary-btn toggle-btn" type="button" onClick={() => setIsAllRoomsExpanded((current) => !current)}>
                  {isAllRoomsExpanded ? "收起考场列表" : "展开考场列表"}
                </button>
                {isAllRoomsExpanded ? (
                  <div className="all-room-list">
                    {validatedRooms.map((room, index) => (
                      <div
                        key={room.id}
                        className={`room-row compact ${drawStatus.currentHighlightRoomIndex === index ? "highlight" : ""} ${assignedRoomNos.has(room.roomNo) ? "hit" : ""}`.trim()}
                      >
                        <span>{room.roomNo}</span>
                        <span>{roomStatusLabel(room.roomNo, index)}</span>
                      </div>
                    ))}
                  </div>
                ) : null}
              </section>
            </div>
            <p className="status-legend">状态说明：高亮=当前抽签，描边=已完成分配。</p>
          </div>
        </div>
      ) : null}

      {step === "result" ? (
        <div className="card-shell page-card">
          <span className="section-kicker">步骤五</span>
          <h3 className="section-title">抽签结果</h3>
          <div className="result-summary">
            <div className="stat-cell"><span>总考场</span><strong>{resultSummary.total}</strong></div>
            <div className="stat-cell"><span>固定结对</span><strong>{resultSummary.fixedCount}</strong></div>
            <div className="stat-cell"><span>随机结对</span><strong>{resultSummary.randomCount}</strong></div>
            <div className="stat-cell"><span>抽签耗时</span><strong>{resultSummary.durationText}</strong></div>
          </div>
          <div className="table-wrap">
            <table className="table">
              <thead><tr><th>考场号</th><th>监考员甲</th><th>监考员乙</th><th>结对方式</th><th>操作</th></tr></thead>
              <tbody>
                {finalAssignments.map((item, index) => (
                  <tr key={item.id} className={recentRedrawId === item.id ? "redraw-row" : ""}>
                    <td>{item.roomNo}</td>
                    <td>{item.invigilatorA}</td>
                    <td>{item.invigilatorB}</td>
                    <td>{item.pairMode === "fixed" ? "固定结对" : "随机结对"}</td>
                    <td><button className="mini-btn" type="button" onClick={() => redrawOne(index)}>单场重抽</button></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="actions">
            <button className="secondary-btn" type="button" onClick={() => void redrawAll()}>全部重抽</button>
            <button className="secondary-btn" type="button" onClick={restartFlow}>重新开始</button>
            <button className="primary-btn" type="button" onClick={exportCsv}>导出结果</button>
          </div>
        </div>
      ) : null}
    </section>
  );
}
