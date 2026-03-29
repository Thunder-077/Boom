<template>
  <section class="draw-panel" :class="{ dragging: isDragging && step === 'import' }">
    <div class="steps-rail card-shell">
      <div
        v-for="(item, index) in flowSteps"
        :key="item.key"
        class="flow-step"
        :class="{ active: flowActiveIndex === index, done: flowActiveIndex > index }"
      >
        <span class="step-index">{{ index + 1 }}</span>
        <span class="step-label">{{ item.label }}</span>
      </div>
    </div>

    <div v-if="step === 'home'" class="card-shell page-card">
      <h3 class="page-title">监考抽签</h3>
      <p class="lead">为每个考场一次性分配监考员甲/乙，支持固定结对和随机结对。</p>
      <p class="desc">流程：录入考场 → 导入监考员 → 选择方式 → 批量抽签 → 查看结果</p>
      <div class="prepare-list">
        <span class="pill">准备考场</span>
        <span class="pill">准备监考员Excel</span>
        <span class="pill">确认结对方式（固定或随机）</span>
      </div>
      <div class="actions"><button class="primary-btn" type="button" @click="step = 'rooms'">开始抽签</button></div>
    </div>

    <div v-else-if="step === 'rooms'" class="grid-two">
      <div class="card-shell page-card">
        <h3 class="section-title">录入考场</h3>
        <p class="desc">每行一个考场号，例如 A101。</p>
        <textarea v-model="roomsInput" class="glass-area room-input" placeholder="A101&#10;A102&#10;A103" />
        <p class="desc danger" v-if="roomError">{{ roomError }}</p>
        <div class="actions">
          <button class="secondary-btn" type="button" @click="step = 'home'">上一步</button>
          <button class="primary-btn" type="button" @click="confirmRooms">确认考场并继续</button>
        </div>
      </div>
      <div class="card-shell page-card">
        <h3 class="section-title">实时预览</h3>
        <div class="room-stats-grid compact">
          <div class="stat-cell"><span>有效考场</span><strong>{{ roomPreviewStats.validCount }}</strong></div>
          <div class="stat-cell"><span>重复条目</span><strong>{{ roomPreviewStats.duplicateCount }}</strong></div>
          <div class="stat-cell"><span>输入条目</span><strong>{{ roomPreviewStats.inputCount }}</strong></div>
          <div class="stat-cell"><span>空白条目</span><strong>{{ roomPreviewStats.emptyCount }}</strong></div>
        </div>
        <div class="row-list"><span v-for="room in parsedRoomsPreview" :key="room.id" class="pill">{{ room.roomNo }}</span></div>
      </div>
    </div>

    <div v-else-if="step === 'import'" class="grid-two">
      <div class="card-shell page-card">
        <h3 class="section-title">导入监考名单</h3>
        <div class="drop-zone" :class="{ active: isDragging }" @click="triggerFilePicker">
          <strong>{{ isDragging ? "松开鼠标即可导入 Excel" : "拖拽 Excel 到此处以导入监考人员名单" }}</strong>
          <span class="drop-hint">{{ isDragging ? "已检测到文件，释放后开始解析" : "或点击此区域选择文件（.xlsx / .xls）" }}</span>
        </div>
        <input ref="fileInputRef" class="hidden-file" type="file" accept=".xlsx,.xls" @change="onFilePicked" />
        <p class="desc" :class="{ danger: importStatus === 'error' }">{{ importMessage }}</p>
        <div class="actions">
          <button class="secondary-btn" type="button" @click="step = 'rooms'">上一步</button>
          <button class="primary-btn" type="button" :disabled="importedRows.length === 0" @click="step = 'mode'">进入方式选择</button>
        </div>
      </div>
      <div class="card-shell page-card">
        <h3 class="section-title">导入预览（{{ importedRows.length }}）</h3>
        <div class="room-stats-grid compact">
          <div class="stat-cell"><span>导入组数</span><strong>{{ importPreviewStats.totalRows }}</strong></div>
          <div class="stat-cell"><span>可覆盖考场</span><strong>{{ importPreviewStats.coverableRooms }}</strong></div>
          <div class="stat-cell"><span>待补数量</span><strong>{{ importPreviewStats.shortfall }}</strong></div>
          <div class="stat-cell"><span>当前状态</span><strong>{{ importPreviewStats.statusText }}</strong></div>
        </div>
        <div class="table-wrap small">
          <table class="table">
            <thead><tr><th>组号</th><th>监考员甲</th><th>监考员乙</th></tr></thead>
            <tbody><tr v-for="row in importedRows" :key="row.groupNo"><td>{{ row.groupNo }}</td><td>{{ row.invigilatorAName }}</td><td>{{ row.invigilatorBName }}</td></tr></tbody>
          </table>
        </div>
      </div>
    </div>

    <div v-else-if="step === 'mode'" class="card-shell page-card">
      <h3 class="section-title">选择结对方式</h3>
      <p class="desc">当前选择会影响抽签过程与结果展示。</p>
      <div class="mode-grid">
        <button class="mode-card" :class="{ active: selectedMode === 'fixed' }" @click="selectedMode='fixed'">
          <strong>固定结对（按导入）</strong>
          <span>保持导入的甲乙关系，仅对考场进行随机分配。</span>
        </button>
        <button class="mode-card" :class="{ active: selectedMode === 'random' }" @click="selectedMode='random'">
          <strong>随机结对（甲乙重组）</strong>
          <span>从甲池和乙池分别抽取，随机组成新结对后分配考场。</span>
        </button>
      </div>
      <p v-if="modeError" class="error-text">{{ modeError }}</p>
      <div class="actions">
        <button class="secondary-btn" type="button" @click="step='import'">上一步</button>
        <button class="primary-btn" type="button" @click="goNextFromMode">开始批量抽签</button>
      </div>
    </div>

    <div v-else-if="step === 'draw'" class="grid-two">
      <div class="card-shell page-card draw-main-card">
        <h3 class="section-title">{{ selectedMode === 'fixed' ? '固定结对抽签中' : '随机结对抽签中' }}</h3>
        <p class="desc">当前阶段：{{ phaseLabel }} · 已完成 {{ drawStatus.progress }}%</p>
        <div class="phase-track">
          <div
            v-for="(item, index) in drawPhases"
            :key="item.key"
            class="phase-step"
            :class="{ active: drawPhaseIndex === index, done: drawPhaseIndex > index }"
          >
            {{ item.label }}
          </div>
        </div>
        <p class="draw-counter">当前进度：第 {{ Math.max(drawStatus.currentPairIndex + 1, 0) }} / {{ Math.max(drawQueue.length, 0) }} 对</p>
        <div v-if="selectedMode === 'random'" class="rolling-pair">
          <div class="flip-card" :class="{ flipped: isFlipA }">
            <div class="flip-inner">
              <div class="flip-face flip-front">监考员甲</div>
              <div class="flip-face flip-back">{{ rollingNameA }}</div>
            </div>
          </div>
          <span class="pair-multiplier">×</span>
          <div class="flip-card" :class="{ flipped: isFlipB }">
            <div class="flip-inner">
              <div class="flip-face flip-front">监考员乙</div>
              <div class="flip-face flip-back">{{ rollingNameB }}</div>
            </div>
          </div>
        </div>
        <div class="current-pair">{{ currentPairText }}</div>
        <div class="actions">
          <button class="secondary-btn" :disabled="drawStatus.isDrawing" @click="step='mode'">返回</button>
          <button class="secondary-btn" :disabled="!drawStatus.isDrawing" @click="togglePauseDraw">{{ isDrawPaused ? '继续' : '暂停' }}</button>
          <button class="secondary-btn" :disabled="!drawStatus.isDrawing" @click="cancelDraw">取消</button>
          <button class="primary-btn" :disabled="drawStatus.isDrawing" @click="startDraw">开始抽签</button>
        </div>
      </div>

      <div class="card-shell page-card draw-rooms-card">
        <h3 class="section-title">当前抽签考场</h3>
        <div class="draw-visual-layout">
          <section class="focus-panel">
            <div class="focus-window">
              <div
                v-for="item in focusRooms"
                :key="item.room.id"
                class="room-row"
                :class="{
                  highlight: drawStatus.currentHighlightRoomIndex === item.index,
                  hit: assignedRoomNos.has(item.room.roomNo),
                  centered: drawStatus.currentHighlightRoomIndex === item.index,
                }"
              >
                <span>{{ item.room.roomNo }}</span>
                <span v-if="resultByRoom.get(item.room.roomNo)">
                  {{ resultByRoom.get(item.room.roomNo)?.invigilatorA }} × {{ resultByRoom.get(item.room.roomNo)?.invigilatorB }}
                </span>
              </div>
            </div>
          </section>

          <section class="all-rooms-panel">
            <div class="room-stats-grid">
              <div class="stat-cell"><span>总考场</span><strong>{{ totalRoomCount }}</strong></div>
              <div class="stat-cell"><span>已分配</span><strong>{{ assignedRoomCount }}</strong></div>
              <div class="stat-cell"><span>未分配</span><strong>{{ pendingRoomCount }}</strong></div>
              <div class="stat-cell"><span>当前命中</span><strong>{{ currentHighlightRoomNo || "—" }}</strong></div>
            </div>
            <button class="secondary-btn toggle-btn" type="button" @click="isAllRoomsExpanded = !isAllRoomsExpanded">
              {{ isAllRoomsExpanded ? "收起考场总览" : "展开考场总览" }}
            </button>
            <div v-if="isAllRoomsExpanded" class="all-room-list">
              <div
                v-for="(room, index) in validatedRooms"
                :key="room.id"
                class="room-row compact"
                :class="{ highlight: drawStatus.currentHighlightRoomIndex === index, hit: assignedRoomNos.has(room.roomNo) }"
              >
                <span>{{ room.roomNo }}</span>
                <span>{{ roomStatusLabel(room.roomNo, index) }}</span>
              </div>
            </div>
          </section>
        </div>
        <p class="status-legend">状态说明：高亮=当前抽签，描边=已完成分配。</p>
      </div>
    </div>

    <div v-else class="card-shell page-card">
      <h3 class="section-title">抽签结果</h3>
      <div class="result-summary">
        <div class="stat-cell"><span>总考场</span><strong>{{ resultSummary.total }}</strong></div>
        <div class="stat-cell"><span>固定结对</span><strong>{{ resultSummary.fixedCount }}</strong></div>
        <div class="stat-cell"><span>随机结对</span><strong>{{ resultSummary.randomCount }}</strong></div>
        <div class="stat-cell"><span>抽签耗时</span><strong>{{ resultSummary.durationText }}</strong></div>
      </div>
      <div class="table-wrap">
        <table class="table">
          <thead><tr><th>考场号</th><th>监考员甲</th><th>监考员乙</th><th>结对方式</th><th>操作</th></tr></thead>
          <tbody>
            <tr v-for="(item, index) in finalAssignments" :key="item.id" :class="{ 'redraw-row': recentRedrawId === item.id }">
              <td>{{ item.roomNo }}</td><td>{{ item.invigilatorA }}</td><td>{{ item.invigilatorB }}</td><td>{{ item.pairMode === 'fixed' ? '固定结对' : '随机结对' }}</td>
              <td><button class="mini-btn" type="button" @click="redrawOne(index)">单场重抽</button></td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="actions">
        <button class="secondary-btn" type="button" @click="redrawAll">全部重抽</button>
        <button class="secondary-btn" type="button" @click="restartFlow">重新开始</button>
        <button class="primary-btn" type="button" @click="exportCsv">导出结果</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invigilationService } from "../../invigilation/service";
import type { AnimationPhase, DrawAssignment, ImportedInvigilatorRow, InvigilatorPair, PairMode, RoomItem, StepKey } from "../model";

const step = ref<StepKey>("home");
const selectedMode = ref<PairMode | null>(null);
const roomsInput = ref("");
const validatedRooms = ref<RoomItem[]>([]);
const importedRows = ref<ImportedInvigilatorRow[]>([]);
const roomError = ref("");
const modeError = ref("");
const importStatus = ref<"idle" | "importing" | "success" | "error">("idle");
const importMessage = ref("请拖拽导入 Excel。");
const isDragging = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
let unlistenDragDrop: (() => void) | null = null;

const drawQueue = ref<InvigilatorPair[]>([]);
const finalAssignments = ref<DrawAssignment[]>([]);
const isDrawPaused = ref(false);
const drawCancelRequested = ref(false);
const isAllRoomsExpanded = ref(false);
const rollingNameA = ref("—");
const rollingNameB = ref("—");
const isFlipA = ref(false);
const isFlipB = ref(false);
const drawStartedAt = ref(0);
const drawEndedAt = ref(0);
const recentRedrawId = ref("");
const focusWindowSize = 7;
const drawStatus = reactive({
  phase: "idle" as AnimationPhase,
  isDrawing: false,
  progress: 0,
  currentPairIndex: -1,
  currentHighlightRoomIndex: -1,
});

const parsedRoomsPreview = computed(() =>
  Array.from(new Set(roomsInput.value.split(/[\n,，]+/g).map((s) => s.trim()).filter(Boolean))).map((roomNo) => ({
    id: `room-${roomNo}`,
    roomNo,
  })),
);
const roomPreviewStats = computed(() => {
  const raw = roomsInput.value.split(/[\n,，]+/g).map((s) => s.trim());
  const inputCount = raw.length;
  const validList = raw.filter(Boolean);
  const validCount = new Set(validList).size;
  const duplicateCount = Math.max(validList.length - validCount, 0);
  const emptyCount = Math.max(inputCount - validList.length, 0);
  return { inputCount, validCount, duplicateCount, emptyCount };
});
const assignedRoomNos = computed(() => new Set(finalAssignments.value.filter((x) => x.assigned).map((x) => x.roomNo)));
const resultByRoom = computed(() => new Map(finalAssignments.value.filter((x) => x.assigned).map((x) => [x.roomNo, x])));
const currentPair = computed(() => drawQueue.value[drawStatus.currentPairIndex] ?? null);
const currentPairText = computed(() => (currentPair.value ? `${currentPair.value.invigilatorA} × ${currentPair.value.invigilatorB}` : "等待开始"));
const phaseLabel = computed(
  () =>
    ({
      idle: "待机",
      pair_rolling: "随机结对",
      room_fast: "快转",
      room_slow: "减速",
      room_hit: "命中",
      card_slide: "落位",
      completed: "完成",
    })[drawStatus.phase],
);
const focusStartIndex = computed(() => {
  const anchor = drawStatus.currentHighlightRoomIndex < 0 ? 0 : drawStatus.currentHighlightRoomIndex;
  const start = anchor - Math.floor(focusWindowSize / 2);
  return Math.max(0, Math.min(start, Math.max(0, validatedRooms.value.length - focusWindowSize)));
});
const focusRooms = computed(() =>
  validatedRooms.value.slice(focusStartIndex.value, focusStartIndex.value + focusWindowSize).map((room, i) => ({ room, index: focusStartIndex.value + i })),
);
const invigilatorAList = computed(() => importedRows.value.map((x) => x.invigilatorAName));
const invigilatorBList = computed(() => importedRows.value.map((x) => x.invigilatorBName));
const totalRoomCount = computed(() => validatedRooms.value.length);
const assignedRoomCount = computed(() => finalAssignments.value.filter((x) => x.assigned).length);
const pendingRoomCount = computed(() => Math.max(totalRoomCount.value - assignedRoomCount.value, 0));
const currentHighlightRoomNo = computed(() => (drawStatus.currentHighlightRoomIndex < 0 ? "" : validatedRooms.value[drawStatus.currentHighlightRoomIndex]?.roomNo ?? ""));
const importPreviewStats = computed(() => {
  const totalRows = importedRows.value.length;
  const coverableRooms = Math.min(totalRows, validatedRooms.value.length);
  const shortfall = Math.max(validatedRooms.value.length - totalRows, 0);
  const statusText =
    importStatus.value === "error" ? "导入失败" : importStatus.value === "success" ? "导入成功" : importStatus.value === "importing" ? "导入中" : "等待导入";
  return { totalRows, coverableRooms, shortfall, statusText };
});
const flowSteps = [
  { key: "rooms", label: "录入考场" },
  { key: "import", label: "导入监考员" },
  { key: "mode", label: "选择方式" },
  { key: "draw", label: "抽签中" },
  { key: "result", label: "结果" },
] as const;
const flowActiveIndex = computed(() => {
  if (step.value === "home") return 0;
  const idx = flowSteps.findIndex((x) => x.key === step.value);
  return idx < 0 ? 0 : idx;
});
const drawPhases = [
  { key: "pair_rolling", label: "结对翻牌" },
  { key: "room_fast", label: "考场快转" },
  { key: "room_slow", label: "减速定位" },
  { key: "room_hit", label: "命中确认" },
  { key: "card_slide", label: "结果落位" },
] as const;
const drawPhaseRank: Record<AnimationPhase, number> = {
  idle: -1,
  pair_rolling: 0,
  room_fast: 1,
  room_slow: 2,
  room_hit: 3,
  card_slide: 4,
  completed: 4,
};
const drawPhaseIndex = computed(() => drawPhaseRank[drawStatus.phase]);
const resultSummary = computed(() => {
  const total = finalAssignments.value.length;
  const fixedCount = finalAssignments.value.filter((x) => x.pairMode === "fixed").length;
  const randomCount = Math.max(total - fixedCount, 0);
  const durationMs = drawEndedAt.value > drawStartedAt.value ? drawEndedAt.value - drawStartedAt.value : 0;
  const durationText = durationMs > 0 ? `${(durationMs / 1000).toFixed(1)}s` : "—";
  return { total, fixedCount, randomCount, durationText };
});

const sleep = (ms: number) => new Promise((r) => window.setTimeout(r, ms));
const shuffle = <T,>(list: T[]) => {
  const n = [...list];
  for (let i = n.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [n[i], n[j]] = [n[j], n[i]];
  }
  return n;
};

function confirmRooms() {
  roomError.value = "";
  if (parsedRoomsPreview.value.length === 0) {
    roomError.value = "请至少录入 1 个考场号。";
    return;
  }
  validatedRooms.value = parsedRoomsPreview.value;
  step.value = "import";
}

function goNextFromMode() {
  modeError.value = "";
  if (!selectedMode.value) {
    modeError.value = "请先选择结对方式。";
    return;
  }
  const need = validatedRooms.value.length;
  if (selectedMode.value === "fixed" && importedRows.value.length < need) {
    modeError.value = `固定结对不足：需要 ${need} 对。`;
    return;
  }
  if (selectedMode.value === "random" && (invigilatorAList.value.length < need || invigilatorBList.value.length < need)) {
    modeError.value = `随机分配人数不足：甲乙各需 ${need} 人。`;
    return;
  }
  step.value = "draw";
}

function buildQueue(): InvigilatorPair[] {
  const count = validatedRooms.value.length;
  if (selectedMode.value === "fixed") {
    return shuffle(importedRows.value)
      .slice(0, count)
      .map((r, i) => ({
        id: `f-${i}`,
        groupNo: r.groupNo,
        invigilatorA: r.invigilatorAName,
        invigilatorB: r.invigilatorBName,
        mode: "fixed" as PairMode,
      }));
  }
  const a = shuffle(invigilatorAList.value);
  const b = shuffle(invigilatorBList.value);
  return Array.from({ length: count }).map((_, i) => ({
    id: `r-${i}`,
    groupNo: `R${i + 1}`,
    invigilatorA: a[i],
    invigilatorB: b[i],
    mode: "random" as PairMode,
  }));
}

function initAssignments() {
  drawQueue.value = buildQueue();
  const rooms = shuffle(validatedRooms.value);
  finalAssignments.value = drawQueue.value.map((p, i) => ({
    id: `a-${i}`,
    roomNo: rooms[i].roomNo,
    invigilatorA: p.invigilatorA,
    invigilatorB: p.invigilatorB,
    pairMode: p.mode,
    statusTag: "待分配",
    assigned: false,
  }));
}

async function waitIfPaused() {
  while (isDrawPaused.value && !drawCancelRequested.value) {
    await sleep(90);
  }
}

async function marqueeTo(target: number, speeds: number[]) {
  const total = validatedRooms.value.length;
  let idx = Math.max(0, drawStatus.currentHighlightRoomIndex);
  for (const sp of speeds) {
    if (drawCancelRequested.value) return false;
    await waitIfPaused();
    idx = (idx + 1) % total;
    drawStatus.currentHighlightRoomIndex = idx;
    await sleep(sp);
  }
  while (idx !== target) {
    if (drawCancelRequested.value) return false;
    await waitIfPaused();
    idx = (idx + 1) % total;
    drawStatus.currentHighlightRoomIndex = idx;
    await sleep(speeds[speeds.length - 1]);
  }
  return true;
}

async function playRandomRolling(pair: InvigilatorPair) {
  drawStatus.phase = "pair_rolling";
  const a = shuffle(invigilatorAList.value);
  const b = shuffle(invigilatorBList.value);
  isFlipA.value = false;
  isFlipB.value = false;
  for (let i = 0; i < 5; i += 1) {
    if (drawCancelRequested.value) return false;
    await waitIfPaused();
    rollingNameA.value = a[i % a.length] ?? pair.invigilatorA;
    rollingNameB.value = b[i % b.length] ?? pair.invigilatorB;
    isFlipA.value = !isFlipA.value;
    isFlipB.value = !isFlipB.value;
    await sleep(55);
  }
  rollingNameA.value = pair.invigilatorA;
  rollingNameB.value = pair.invigilatorB;
  isFlipA.value = true;
  isFlipB.value = true;
  await sleep(90);
  return !drawCancelRequested.value;
}

async function startDraw() {
  if (drawStatus.isDrawing) return;
  initAssignments();
  drawStartedAt.value = Date.now();
  drawEndedAt.value = 0;
  drawStatus.isDrawing = true;
  drawCancelRequested.value = false;
  drawStatus.currentHighlightRoomIndex = -1;
  drawStatus.currentPairIndex = -1;
  drawStatus.progress = 0;

  for (let i = 0; i < finalAssignments.value.length; i += 1) {
    if (drawCancelRequested.value) return cancelDrawNow();
    drawStatus.currentPairIndex = i;
    const pair = drawQueue.value[i];
    if (selectedMode.value === "random") {
      const done = await playRandomRolling(pair);
      if (!done) return cancelDrawNow();
    } else {
      rollingNameA.value = pair.invigilatorA;
      rollingNameB.value = pair.invigilatorB;
    }

    const target = validatedRooms.value.findIndex((r) => r.roomNo === finalAssignments.value[i].roomNo);
    drawStatus.phase = "room_fast";
    if (!(await marqueeTo(target, [35, 35, 40, 45, 50, 55]))) return cancelDrawNow();
    drawStatus.phase = "room_slow";
    if (!(await marqueeTo(target, [80, 95, 110, 130]))) return cancelDrawNow();
    drawStatus.phase = "room_hit";
    finalAssignments.value[i].statusTag = "正在抽取";
    await sleep(140);
    if (drawCancelRequested.value) return cancelDrawNow();
    drawStatus.phase = "card_slide";
    finalAssignments.value[i].assigned = true;
    finalAssignments.value[i].statusTag = "已落位";
    drawStatus.progress = Math.round(((i + 1) / finalAssignments.value.length) * 100);
    await sleep(180);
  }
  drawStatus.phase = "completed";
  drawStatus.isDrawing = false;
  drawEndedAt.value = Date.now();
  await sleep(120);
  step.value = "result";
}

function cancelDrawNow() {
  drawStatus.isDrawing = false;
  drawStatus.phase = "idle";
  drawStatus.progress = 0;
  drawStatus.currentPairIndex = -1;
  drawStatus.currentHighlightRoomIndex = -1;
  isDrawPaused.value = false;
  drawQueue.value = [];
  finalAssignments.value = [];
  drawCancelRequested.value = false;
  isFlipA.value = false;
  isFlipB.value = false;
  drawStartedAt.value = 0;
  drawEndedAt.value = 0;
}

function togglePauseDraw() {
  if (drawStatus.isDrawing) isDrawPaused.value = !isDrawPaused.value;
}

function cancelDraw() {
  if (drawStatus.isDrawing) drawCancelRequested.value = true;
}

function redrawOne(index: number) {
  if (finalAssignments.value.length < 2) return;
  const candidates = finalAssignments.value.map((_, i) => i).filter((i) => i !== index);
  const j = candidates[Math.floor(Math.random() * candidates.length)];
  const next = [...finalAssignments.value];
  const a = next[index];
  const b = next[j];
  next[index] = { ...a, invigilatorA: b.invigilatorA, invigilatorB: b.invigilatorB, pairMode: b.pairMode, assigned: true, statusTag: "已落位" };
  next[j] = { ...b, invigilatorA: a.invigilatorA, invigilatorB: a.invigilatorB, pairMode: a.pairMode, assigned: true, statusTag: "已落位" };
  finalAssignments.value = next;
  recentRedrawId.value = next[index].id;
  window.setTimeout(() => {
    if (recentRedrawId.value === next[index].id) recentRedrawId.value = "";
  }, 1400);
}

async function redrawAll() {
  step.value = "draw";
  await sleep(0);
  await startDraw();
}

function restartFlow() {
  step.value = "home";
  selectedMode.value = null;
  importedRows.value = [];
  validatedRooms.value = [];
  roomsInput.value = "";
  modeError.value = "";
  roomError.value = "";
  importStatus.value = "idle";
  importMessage.value = "请拖拽导入 Excel。";
  isAllRoomsExpanded.value = false;
  cancelDrawNow();
}

function exportCsv() {
  const rows = [
    ["考场号", "监考员甲", "监考员乙", "结对方式"],
    ...finalAssignments.value.map((x) => [x.roomNo, x.invigilatorA, x.invigilatorB, x.pairMode === "fixed" ? "固定结对" : "随机分配"]),
  ];
  const csv = rows.map((r) => r.map((c) => `"${String(c).split("\"").join("\"\"")}"`).join(",")).join("\n");
  const blob = new Blob([`\uFEFF${csv}`], { type: "text/csv;charset=utf-8;" });
  const a = document.createElement("a");
  const url = URL.createObjectURL(blob);
  a.href = url;
  a.download = `监考抽签结果-${new Date().toISOString().split(":").join("-")}.csv`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function roomStatusLabel(roomNo: string, roomIndex: number) {
  if (assignedRoomNos.value.has(roomNo)) return "已落位";
  if (drawStatus.currentHighlightRoomIndex === roomIndex) return "进行中";
  return "未分配";
}

function triggerFilePicker() {
  fileInputRef.value?.click();
}

function onFilePicked(event: Event) {
  const target = event.target as HTMLInputElement | null;
  const file = target?.files?.[0];
  if (!file) return;
  const fullPath = (file as File & { path?: string }).path || "";
  if (fullPath && (fullPath.toLowerCase().endsWith(".xlsx") || fullPath.toLowerCase().endsWith(".xls"))) {
    void handleImport(fullPath);
  } else {
    importStatus.value = "error";
    importMessage.value = "当前环境未获取到本地路径，请优先使用拖拽导入。";
  }
  if (target) target.value = "";
}

function normalizeDroppedPath(raw: string) {
  const t = raw.trim();
  if (!t.startsWith("file://")) return t;
  try {
    const u = new URL(t);
    return decodeURIComponent(u.pathname).replace(/^\/([A-Za-z]:\/)/, "$1").replace(/\//g, "\\");
  } catch {
    return decodeURIComponent(t.replace(/^file:\/\//i, "")).replace(/^\/([A-Za-z]:\/)/, "$1").replace(/\//g, "\\");
  }
}

function pickExcelPath(paths: string[]) {
  for (const path of paths) {
    const p = normalizeDroppedPath(path);
    const l = p.toLowerCase();
    if (l.endsWith(".xlsx") || l.endsWith(".xls")) return p;
  }
  return undefined;
}

async function handleImport(filePath: string) {
  importStatus.value = "importing";
  importMessage.value = "正在导入...";
  try {
    const result = await invigilationService.importMonitorDrawPairsExcel(filePath);
    importedRows.value = result.rows;
    importStatus.value = "success";
    importMessage.value = `导入成功：${result.rowCount} 组，耗时 ${result.durationMs}ms`;
  } catch (e) {
    importedRows.value = [];
    importStatus.value = "error";
    importMessage.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow();
  unlistenDragDrop = await appWindow.onDragDropEvent((event) => {
    if (step.value !== "import") return;
    if (event.payload.type === "enter" || event.payload.type === "over") {
      isDragging.value = true;
      return;
    }
    if (event.payload.type === "leave") {
      isDragging.value = false;
      return;
    }
    if (event.payload.type === "drop") {
      isDragging.value = false;
      const excel = pickExcelPath(event.payload.paths);
      if (excel) {
        void handleImport(excel);
      } else {
        importStatus.value = "error";
        importMessage.value = "未识别到 Excel 文件路径。";
      }
    }
  });
});
onUnmounted(() => {
  if (unlistenDragDrop) {
    unlistenDragDrop();
    unlistenDragDrop = null;
  }
});
</script>

<style scoped>
.draw-panel { display: flex; flex-direction: column; gap: 16px; position: relative; }
.page-card { padding: 20px; }
.steps-rail {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
  padding: 10px;
}
.flow-step {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 12px;
  border: 1px solid #dce8f8;
  background: #f8fbff;
  color: #5e7088;
}
.flow-step.active { border-color: #0f6cbd; color: #0f6cbd; background: #eaf3ff; }
.flow-step.done { border-color: #97c0ee; color: #2f5c8c; }
.step-index {
  width: 22px;
  height: 22px;
  border-radius: 999px;
  border: 1px solid currentColor;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
}
.step-label { font-size: 13px; font-weight: 650; }
.page-title { font-size: 34px; line-height: 1.2; margin: 0; }
.section-title { font-size: 22px; line-height: 1.3; margin: 0; color: #16253a; }
.lead { margin-top: 10px; font-size: 15px; color: #344861; line-height: 1.5; }
.desc { margin-top: 8px; color: var(--color-text-muted); font-size: 13px; line-height: 1.45; }
.danger, .error-text { color: var(--color-danger); }
.actions { margin-top: 14px; display: flex; gap: 10px; flex-wrap: wrap; }
.grid-two { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.room-input { min-height: 260px; }
.row-list { margin-top: 12px; display: flex; flex-wrap: wrap; gap: 8px; }
.pill { padding: 6px 10px; border-radius: 999px; border: 1px solid #dbe8f8; background: #f8fbff; }
.prepare-list { margin-top: 14px; display: flex; flex-wrap: wrap; gap: 8px; }
.drop-zone {
  margin-top: 12px;
  min-height: 180px;
  border: 2px dashed #9bc1ef;
  border-radius: 14px;
  background: #f8fbff;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
  transition: border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease, transform 0.2s ease;
}
.drop-zone:hover { border-color: #6ba6e7; background: #f1f8ff; }
.drop-zone.active {
  border-color: #0f6cbd;
  border-style: solid;
  background: linear-gradient(180deg, #eff7ff, #e5f1ff);
  box-shadow: 0 14px 28px rgba(15, 108, 189, 0.2);
  transform: translateY(-1px) scale(1.01);
}
.drop-zone.active strong { color: #0f4f8c; }
.drop-zone.active .drop-hint { color: #29689e; }
.drop-hint { color: #567394; font-size: 13px; }
.hidden-file { display: none; }
.mode-grid { margin-top: 12px; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.mode-card { min-height: 110px; border: 1px solid #dce8f8; border-radius: 16px; background: #fff; padding: 14px; text-align: left; display: flex; flex-direction: column; gap: 8px; cursor: pointer; }
.mode-card.active { border-color: #7fb0ea; background: #eaf3ffcc; box-shadow: inset 3px 0 0 #0f6cbd; }
.draw-main-card { min-height: 360px; }
.draw-rooms-card { min-height: 520px; }
.phase-track { margin-top: 8px; display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 8px; }
.phase-step {
  border-radius: 10px;
  border: 1px solid #dce8f8;
  background: #f8fbff;
  text-align: center;
  font-size: 12px;
  font-weight: 650;
  color: #6b7f99;
  padding: 7px 8px;
}
.phase-step.active { border-color: #0f6cbd; color: #0f6cbd; background: #eaf3ff; }
.phase-step.done { border-color: #90c495; color: #3d7750; background: #effaf1; }
.draw-counter { margin: 8px 0 0; font-size: 13px; color: #35506f; }
.rolling-pair {
  margin-top: 10px;
  min-height: 92px;
  border-radius: 12px;
  border: 1px solid #dce8f8;
  background: #f8fbff;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 10px;
}
.flip-card {
  width: 170px;
  height: 68px;
  perspective: 800px;
  flex: 0 0 auto;
}
.flip-inner {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 12px;
  transform-style: preserve-3d;
  transition: transform 200ms ease;
}
.flip-card.flipped .flip-inner { transform: rotateY(180deg); }
.flip-face {
  position: absolute;
  inset: 0;
  border-radius: 12px;
  border: 1px solid #dce8f8;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 10px;
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
  backface-visibility: hidden;
  -webkit-backface-visibility: hidden;
}
.flip-front {
  color: #607088;
  background: linear-gradient(180deg, #ffffff, #f5f9ff);
}
.flip-back {
  color: #0f172a;
  background: linear-gradient(180deg, #edf5ff, #e2efff);
  transform: rotateY(180deg);
}
.pair-multiplier {
  color: #0f6cbd;
  font-size: 18px;
  font-weight: 800;
  user-select: none;
}
.current-pair { margin-top: 10px; min-height: 56px; border-radius: 12px; border: 1px solid #dce8f8; background: #f8fbff; display: flex; align-items: center; justify-content: center; font-weight: 700; }
.draw-visual-layout { margin-top: 10px; display: grid; grid-template-columns: minmax(0, 1.7fr) minmax(220px, 1fr); gap: 12px; align-items: start; }
.focus-window { position: relative; border-radius: 14px; border: 1px solid #dce8f8; background: #fbfdff; padding: 10px; height: 404px; display: flex; flex-direction: column; justify-content: center; gap: 8px; overflow: hidden; }
.focus-window::before, .focus-window::after { content: ""; position: absolute; left: 0; right: 0; height: 48px; pointer-events: none; z-index: 2; }
.focus-window::before { top: 0; background: linear-gradient(180deg, #fbfdff 10%, rgba(251, 253, 255, 0)); }
.focus-window::after { bottom: 0; background: linear-gradient(0deg, #fbfdff 10%, rgba(251, 253, 255, 0)); }
.all-rooms-panel { border: 1px solid #dce8f8; border-radius: 14px; background: #fbfdff; padding: 10px; }
.room-stats-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.stat-cell { border: 1px solid #dce8f8; border-radius: 10px; background: #f5f9ff; padding: 8px 9px; display: flex; flex-direction: column; gap: 4px; }
.stat-cell span { color: #607088; font-size: 12px; }
.stat-cell strong { color: #0f172a; font-size: 14px; }
.room-stats-grid.compact .stat-cell { padding: 7px 8px; }
.toggle-btn { width: 100%; margin-top: 10px; justify-content: center; }
.all-room-list { margin-top: 10px; max-height: 360px; overflow: auto; display: flex; flex-direction: column; gap: 6px; padding-right: 2px; }
.table-wrap { margin-top: 12px; overflow: auto; border: 1px solid #e4edf8; border-radius: 12px; background: #fff; }
.table-wrap.small { max-height: 320px; }
.room-row { min-height: 44px; border-radius: 10px; border: 1px solid #dce8f8; background: #f8fbff; display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 0 10px; min-width: 100%; transition: border-color .2s ease, box-shadow .2s ease, transform .2s ease; }
.room-row.highlight { border-color: #0f6cbd; background: #eaf3ff; box-shadow: 0 8px 18px rgba(15,108,189,.16); }
.room-row.hit { border-color: #8cb8f0; }
.room-row.centered { transform: scale(1.02); }
.room-row.compact { min-height: 38px; font-size: 12px; }
.status-legend { margin: 10px 0 0; color: #607088; font-size: 12px; }
.mini-btn { border: 1px solid #b8d3f5; background: #eef5ff; color: #0f6cbd; border-radius: 8px; padding: 6px 12px; font-size: 12px; font-weight: 700; cursor: pointer; }
.mini-btn:hover { background: #e1efff; border-color: #8fb8eb; }
.result-summary {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
.redraw-row { animation: redrawFlash 1.2s ease; }
@keyframes redrawFlash {
  0% { background: rgba(15, 108, 189, 0.16); }
  100% { background: transparent; }
}
.drag-overlay { position: absolute; inset: 0; z-index: 10; border-radius: 20px; background: rgba(15,108,189,.08); border: 2px dashed #7fb1ea; display: flex; align-items: center; justify-content: center; }
.drag-card { padding: 20px 24px; border-radius: 16px; background: rgba(255,255,255,.95); border: 1px solid rgba(15,108,189,.2); }
@media (max-width: 1200px) {
  .grid-two, .mode-grid, .steps-rail, .phase-track, .result-summary { grid-template-columns: 1fr; }
  .draw-visual-layout { grid-template-columns: 1fr; }
  .focus-window { height: 350px; }
}
</style>
