import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import fontkit from "@pdf-lib/fontkit";
import { degrees, type PDFFont, PDFDocument, rgb, StandardFonts } from "pdf-lib";
import * as pdfjsLib from "pdfjs-dist";
import pdfjsWorker from "pdfjs-dist/build/pdf.worker.mjs?url";
import {
  Download,
  Copy,
  Edit3,
  FileUp,
  Highlighter,
  Image as ImageIcon,
  MousePointer2,
  Plus,
  RotateCcw,
  RotateCw,
  Square,
  StickyNote,
  PenLine,
  PanelLeftClose,
  PanelLeftOpen,
  Trash2,
  Type,
  ArrowUp,
  ArrowDown,
  ZoomIn,
  ZoomOut,
  Undo2,
  Redo2,
  ScanText,
} from "lucide-react";
import { Button, Tag } from "../../../widgets/common/index.react";
import { hasDesktopRuntime } from "../../../shared/utils/desktopRuntime";
import type { PdfEditObject, PdfEditTool, PdfPageEntry, PdfPageSize } from "../model/types";

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker;

type PdfDocumentProxy = pdfjsLib.PDFDocumentProxy;
type PdfPageProxy = pdfjsLib.PDFPageProxy;

const TOOL_LABELS: Record<PdfEditTool, string> = {
  select: "选择",
  text: "文字",
  "replace-text": "原文替换",
  highlight: "高亮",
  rect: "矩形",
  note: "便签",
  image: "图片",
  signature: "签名",
};

const TOOL_ICONS = {
  select: MousePointer2,
  text: Type,
  "replace-text": Edit3,
  highlight: Highlighter,
  rect: Square,
  note: StickyNote,
  image: ImageIcon,
  signature: PenLine,
};

const DEFAULT_TEXT_OBJECT_TEXT = "双击编辑文字";

interface PdfTextHitBox {
  id: string;
  pageId: string;
  sourcePageNumber: number;
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  fontSize: number;
}

interface PdfImageAsset {
  id: string;
  bytes: Uint8Array;
  dataUrl: string;
  mimeType: string;
  width: number;
  height: number;
}

interface PdfEditorSnapshot {
  pageEntries: PdfPageEntry[];
  objects: PdfEditObject[];
  textHitBoxes: PdfTextHitBox[];
  imageAssets: PdfImageAsset[];
  selectedPageId: string | null;
  selectedId: string | null;
  tool: PdfEditTool;
}

interface PdfNativeTextFragment {
  text: string;
  editability: "native-candidate" | "overlay-recommended" | string;
  reason: string;
}

interface PdfNativeTextLine {
  text: string;
  fragments: PdfNativeTextFragment[];
}

interface PdfNativeTextPage {
  pageNumber: number;
  width: number;
  height: number;
  lines: PdfNativeTextLine[];
}

interface PdfNativeEditAnalysis {
  engine: string;
  pages: PdfNativeTextPage[];
  warnings: string[];
}

type ResizeAnchor = "se";

function makeObjectId() {
  return `pdf-edit-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function makePageId(pageNumber: number) {
  return `pdf-page-${pageNumber}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function makeImageId() {
  return `pdf-image-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function hexToRgb(color: string) {
  const normalized = color.replace("#", "");
  const value = Number.parseInt(normalized, 16);
  return rgb(((value >> 16) & 255) / 255, ((value >> 8) & 255) / 255, (value & 255) / 255);
}

function downloadPdfInBrowser(bytes: Uint8Array, downloadName: string) {
  const blob = new Blob([bytes], { type: "application/pdf" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = downloadName;
  link.click();
  URL.revokeObjectURL(url);
}

async function embedExportFont(outputDoc: PDFDocument): Promise<PDFFont> {
  if (!hasDesktopRuntime()) {
    return outputDoc.embedFont(StandardFonts.Helvetica);
  }
  outputDoc.registerFontkit(fontkit);
  const fontBytes = await invoke<number[]>("get_pdf_editor_font");
  return outputDoc.embedFont(new Uint8Array(fontBytes), { subset: true });
}

function formatExportError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function readImageSize(dataUrl: string) {
  return new Promise<{ width: number; height: number }>((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
    image.onerror = () => reject(new Error("读取图片尺寸失败"));
    image.src = dataUrl;
  });
}

function createObject(tool: PdfEditTool, pageId: string, x: number, y: number, pendingImageAsset?: PdfImageAsset | null): PdfEditObject | null {
  if (tool === "select" || tool === "replace-text") {
    return null;
  }
  if ((tool === "image" || tool === "signature") && pendingImageAsset) {
    const width = Math.min(260, pendingImageAsset.width);
    const height = Math.max(32, width * (pendingImageAsset.height / pendingImageAsset.width));
    return {
      id: makeObjectId(),
      type: tool,
      pageId,
      x,
      y,
      width,
      height,
      imageId: pendingImageAsset.id,
      mimeType: pendingImageAsset.mimeType,
    };
  }
  if (tool === "image" || tool === "signature") {
    return null;
  }
  if (tool === "text") {
    return {
      id: makeObjectId(),
      type: "text",
      pageId,
      x,
      y,
      width: 220,
      height: 54,
      text: DEFAULT_TEXT_OBJECT_TEXT,
      fontSize: 16,
      color: "#111827",
    };
  }
  if (tool === "highlight") {
    return {
      id: makeObjectId(),
      type: "highlight",
      pageId,
      x,
      y,
      width: 220,
      height: 28,
      color: "#fde047",
      opacity: 0.42,
    };
  }
  if (tool === "rect") {
    return {
      id: makeObjectId(),
      type: "rect",
      pageId,
      x,
      y,
      width: 180,
      height: 90,
      strokeColor: "#2563eb",
      strokeWidth: 2,
    };
  }
  return {
    id: makeObjectId(),
    type: "note",
    pageId,
    x,
    y,
    width: 180,
    height: 92,
    text: "批注",
  };
}

function createReplaceTextObject(hitBox: PdfTextHitBox): PdfEditObject {
  return {
    id: makeObjectId(),
    type: "replace-text",
    pageId: hitBox.pageId,
    x: Math.max(0, hitBox.x - 4),
    y: Math.max(0, hitBox.y - 3),
    width: Math.max(hitBox.width + 16, hitBox.text.length * hitBox.fontSize * 0.62),
    height: hitBox.height + 4,
    originalText: hitBox.text,
    text: hitBox.text,
    fontSize: Math.max(8, hitBox.fontSize),
    color: "#111827",
  };
}

function PdfPageCanvas({ page, scale, rotation }: { page: PdfPageProxy; scale: number; rotation: number }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const viewport = page.getViewport({ scale, rotation: (page.rotate + rotation) % 360 });
    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }
    canvas.width = Math.floor(viewport.width);
    canvas.height = Math.floor(viewport.height);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    context.clearRect(0, 0, canvas.width, canvas.height);
    const task = page.render({ canvasContext: context, viewport });
    return () => {
      task.cancel();
    };
  }, [page, rotation, scale]);

  return <canvas ref={canvasRef} className="pdf-page-canvas" />;
}

function PdfPageThumbnail({
  page,
  rotation,
  selected,
  targetWidth,
}: {
  page: PdfPageProxy;
  rotation: number;
  selected: boolean;
  targetWidth: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const baseViewport = page.getViewport({ scale: 1, rotation: (page.rotate + rotation) % 360 });
    const scale = Math.min(1, targetWidth / baseViewport.width);
    const viewport = page.getViewport({ scale, rotation: (page.rotate + rotation) % 360 });
    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }
    canvas.width = Math.floor(viewport.width);
    canvas.height = Math.floor(viewport.height);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    context.clearRect(0, 0, canvas.width, canvas.height);
    const task = page.render({ canvasContext: context, viewport });
    return () => {
      task.cancel();
    };
  }, [page, rotation, targetWidth]);

  return (
    <span className={`pdf-thumb-canvas-wrap ${selected ? "active" : ""}`}>
      <canvas ref={canvasRef} className="pdf-thumb-canvas" />
    </span>
  );
}

function PdfEditOverlay({
  object,
  imageAsset,
  selected,
  scale,
  onSelect,
  onMove,
  onResize,
  onChangeStart,
  onTextChange,
  onTextBlur,
}: {
  object: PdfEditObject;
  imageAsset?: PdfImageAsset;
  selected: boolean;
  scale: number;
  onSelect: (id: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  onResize: (id: string, width: number, height: number, x?: number, y?: number) => void;
  onChangeStart: () => void;
  onTextChange: (id: string, text: string) => void;
  onTextBlur: (id: string) => void;
}) {
  const dragRef = useRef<{ startX: number; startY: number; originX: number; originY: number } | null>(null);
  const resizeRef = useRef<{ startX: number; startY: number; originWidth: number; originHeight: number; anchor: ResizeAnchor } | null>(null);
  const textRef = useRef<HTMLTextAreaElement | null>(null);
  const commonStyle = {
    left: object.x * scale,
    top: object.y * scale,
    width: object.width * scale,
    height: object.height * scale,
  };

  function startDrag(event: React.PointerEvent<HTMLDivElement>) {
    event.stopPropagation();
    onChangeStart();
    onSelect(object.id);
    dragRef.current = {
      startX: event.clientX,
      startY: event.clientY,
      originX: object.x,
      originY: object.y,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function move(event: React.PointerEvent<HTMLDivElement>) {
    if (!dragRef.current) {
      return;
    }
    const nextX = dragRef.current.originX + (event.clientX - dragRef.current.startX) / scale;
    const nextY = dragRef.current.originY + (event.clientY - dragRef.current.startY) / scale;
    onMove(object.id, Math.max(0, nextX), Math.max(0, nextY));
  }

  function endDrag() {
    dragRef.current = null;
  }

  function startResize(event: React.PointerEvent<HTMLButtonElement>, anchor: ResizeAnchor) {
    event.stopPropagation();
    onChangeStart();
    onSelect(object.id);
    resizeRef.current = {
      startX: event.clientX,
      startY: event.clientY,
      originWidth: object.width,
      originHeight: object.height,
      anchor,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function resize(event: React.PointerEvent<HTMLButtonElement>) {
    if (!resizeRef.current) {
      return;
    }
    const nextWidth = resizeRef.current.originWidth + (event.clientX - resizeRef.current.startX) / scale;
    const nextHeight = resizeRef.current.originHeight + (event.clientY - resizeRef.current.startY) / scale;
    onResize(object.id, Math.max(24, nextWidth), Math.max(18, nextHeight));
  }

  function endResize() {
    resizeRef.current = null;
  }

  function syncTextSizeFromDom() {
    const rect = textRef.current?.getBoundingClientRect();
    if (!rect) {
      return;
    }
    const nextWidth = rect.width / scale;
    const nextHeight = rect.height / scale;
    if (Math.abs(nextWidth - object.width) > 1 || Math.abs(nextHeight - object.height) > 1) {
      onResize(object.id, Math.max(24, nextWidth), Math.max(18, nextHeight));
    }
  }

  if (object.type === "replace-text" && !selected) {
    return (
      <div
        className="pdf-edit-object pdf-replace-text-object pdf-replace-text-preview"
        style={{
          ...commonStyle,
          color: object.color,
          fontSize: object.fontSize * scale,
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
          onChangeStart();
          onSelect(object.id);
        }}
      >
        {object.text}
      </div>
    );
  }

  if (object.type === "text" || object.type === "replace-text") {
    return (
      <textarea
        ref={textRef}
        className={`pdf-edit-object pdf-text-object ${object.type === "replace-text" ? "pdf-replace-text-object" : ""} ${selected ? "selected" : ""}`}
        style={{
          ...commonStyle,
          color: object.color,
          fontSize: object.fontSize * scale,
        }}
        value={object.text}
        onChange={(event) => onTextChange(object.id, event.target.value)}
        onBlur={() => {
          syncTextSizeFromDom();
          onTextBlur(object.id);
        }}
        onMouseUp={syncTextSizeFromDom}
        onPointerDown={(event) => {
          event.stopPropagation();
          onChangeStart();
          onSelect(object.id);
        }}
      />
    );
  }

  if ((object.type === "image" || object.type === "signature") && imageAsset) {
    return (
      <div
        className={`pdf-edit-object pdf-image-object ${object.type === "signature" ? "signature" : ""} ${selected ? "selected" : ""}`}
        style={commonStyle}
        onPointerDown={startDrag}
        onPointerMove={move}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
      >
        <img src={imageAsset.dataUrl} alt={object.type === "signature" ? "签名" : "插入图片"} draggable={false} />
        {selected ? (
          <button
            type="button"
            className="pdf-resize-handle se"
            aria-label="调整大小"
            onPointerDown={(event) => startResize(event, "se")}
            onPointerMove={resize}
            onPointerUp={endResize}
            onPointerCancel={endResize}
          />
        ) : null}
      </div>
    );
  }

  const objectStyle =
    object.type === "highlight"
      ? { ...commonStyle, background: object.color, opacity: object.opacity }
      : object.type === "rect"
        ? { ...commonStyle, borderColor: object.strokeColor, borderWidth: object.strokeWidth * scale }
        : { ...commonStyle };

  return (
    <div
      className={`pdf-edit-object pdf-${object.type}-object ${selected ? "selected" : ""}`}
      style={objectStyle}
      onPointerDown={startDrag}
      onPointerMove={move}
      onPointerUp={endDrag}
      onPointerCancel={endDrag}
    >
      {object.type === "note" ? (
        <textarea
          value={object.text}
          onChange={(event) => onTextChange(object.id, event.target.value)}
          onBlur={() => onTextBlur(object.id)}
          onPointerDown={(event) => event.stopPropagation()}
        />
      ) : null}
      {selected ? (
        <button
          type="button"
          className="pdf-resize-handle se"
          aria-label="调整大小"
          onPointerDown={(event) => startResize(event, "se")}
          onPointerMove={resize}
          onPointerUp={endResize}
          onPointerCancel={endResize}
        />
      ) : null}
    </div>
  );
}

export default function PdfEditorPanel() {
  const [fileName, setFileName] = useState("");
  const [pdfBytes, setPdfBytes] = useState<Uint8Array | null>(null);
  const [pdfDoc, setPdfDoc] = useState<PdfDocumentProxy | null>(null);
  const [pages, setPages] = useState<PdfPageProxy[]>([]);
  const [pageSizes, setPageSizes] = useState<PdfPageSize[]>([]);
  const [pageEntries, setPageEntries] = useState<PdfPageEntry[]>([]);
  const [selectedPageId, setSelectedPageId] = useState<string | null>(null);
  const [textHitBoxes, setTextHitBoxes] = useState<PdfTextHitBox[]>([]);
  const [tool, setTool] = useState<PdfEditTool>("select");
  const [scale, setScale] = useState(1.2);
  const [thumbnailScale, setThumbnailScale] = useState(112);
  const [isThumbnailPanelOpen, setIsThumbnailPanelOpen] = useState(true);
  const [objects, setObjects] = useState<PdfEditObject[]>([]);
  const [imageAssets, setImageAssets] = useState<PdfImageAsset[]>([]);
  const [pendingImageAssetId, setPendingImageAssetId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [status, setStatus] = useState("打开一个 PDF 后开始编辑");
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [undoStack, setUndoStack] = useState<PdfEditorSnapshot[]>([]);
  const [redoStack, setRedoStack] = useState<PdfEditorSnapshot[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [nativeAnalysis, setNativeAnalysis] = useState<PdfNativeEditAnalysis | null>(null);
  const [isAnalyzingNative, setIsAnalyzingNative] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const imageInputRef = useRef<HTMLInputElement | null>(null);
  const signatureInputRef = useRef<HTMLInputElement | null>(null);
  const pageFrameRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const thumbnailRefs = useRef<Record<string, HTMLButtonElement | null>>({});
  const pageListRef = useRef<HTMLDivElement | null>(null);

  const selectedObject = useMemo(() => objects.find((object) => object.id === selectedId) ?? null, [objects, selectedId]);
  const selectedPage = useMemo(
    () => pageEntries.find((entry) => entry.id === selectedPageId) ?? pageEntries[0] ?? null,
    [pageEntries, selectedPageId],
  );
  const selectedPageIndex = useMemo(
    () => pageEntries.findIndex((entry) => entry.id === selectedPage?.id),
    [pageEntries, selectedPage],
  );
  const pageLookup = useMemo(() => new Map(pages.map((page) => [page.pageNumber, page])), [pages]);
  const pageSizeLookup = useMemo(() => new Map(pageSizes.map((size) => [size.pageNumber, size])), [pageSizes]);
  const imageAssetLookup = useMemo(() => new Map(imageAssets.map((asset) => [asset.id, asset])), [imageAssets]);
  const pendingImageAsset = useMemo(
    () => imageAssets.find((asset) => asset.id === pendingImageAssetId) ?? null,
    [imageAssets, pendingImageAssetId],
  );
  const selectedNativePage = useMemo(() => {
    if (!nativeAnalysis || !selectedPage) {
      return null;
    }
    return nativeAnalysis.pages.find((page) => page.pageNumber === selectedPage.sourcePageNumber) ?? null;
  }, [nativeAnalysis, selectedPage]);
  const nativeAnalysisSummary = useMemo(() => {
    if (!nativeAnalysis) {
      return null;
    }
    let candidateCount = 0;
    let overlayCount = 0;
    nativeAnalysis.pages.forEach((page) => {
      page.lines.forEach((line) => {
        line.fragments.forEach((fragment) => {
          if (fragment.editability === "native-candidate") {
            candidateCount += 1;
          } else {
            overlayCount += 1;
          }
        });
      });
    });
    return { candidateCount, overlayCount };
  }, [nativeAnalysis]);

  function currentSnapshot(): PdfEditorSnapshot {
    return {
      pageEntries,
      objects,
      textHitBoxes,
      imageAssets,
      selectedPageId,
      selectedId,
      tool,
    };
  }

  function restoreSnapshot(snapshot: PdfEditorSnapshot) {
    setPageEntries(snapshot.pageEntries);
    setObjects(snapshot.objects);
    setTextHitBoxes(snapshot.textHitBoxes);
    setImageAssets(snapshot.imageAssets);
    setSelectedPageId(snapshot.selectedPageId);
    setSelectedId(snapshot.selectedId);
    setTool(snapshot.tool);
    setHasUnsavedChanges(true);
  }

  function rememberHistory() {
    const snapshot = currentSnapshot();
    setUndoStack((current) => {
      const last = current[current.length - 1];
      if (last && JSON.stringify(last) === JSON.stringify(snapshot)) {
        return current;
      }
      return [...current.slice(-49), snapshot];
    });
    setRedoStack([]);
    setHasUnsavedChanges(true);
  }

  function undoEdit() {
    setUndoStack((current) => {
      const previous = current[current.length - 1];
      if (!previous) {
        return current;
      }
      setRedoStack((redo) => [...redo, currentSnapshot()]);
      restoreSnapshot(previous);
      setStatus("已撤销上一步编辑");
      return current.slice(0, -1);
    });
  }

  function redoEdit() {
    setRedoStack((current) => {
      const next = current[current.length - 1];
      if (!next) {
        return current;
      }
      setUndoStack((undo) => [...undo, currentSnapshot()]);
      restoreSnapshot(next);
      setStatus("已重做编辑");
      return current.slice(0, -1);
    });
  }

  function updateObject(id: string, patch: Partial<PdfEditObject>) {
    rememberHistory();
    setObjects((current) => current.map((object) => (object.id === id ? ({ ...object, ...patch } as PdfEditObject) : object)));
  }

  useEffect(() => {
    if (!selectedPageId || !isThumbnailPanelOpen) {
      return;
    }
    thumbnailRefs.current[selectedPageId]?.scrollIntoView({
      block: "nearest",
      inline: "nearest",
    });
  }, [isThumbnailPanelOpen, selectedPageId]);

  function selectPage(entryId: string) {
    setSelectedPageId(entryId);
    clearEmptySelectedObject();
    pageFrameRefs.current[entryId]?.scrollIntoView({ behavior: "smooth", block: "start", inline: "nearest" });
  }

  function syncCurrentPageFromScroll() {
    const list = pageListRef.current;
    if (!list || pageEntries.length === 0) {
      return;
    }
    const listRect = list.getBoundingClientRect();
    const anchorY = listRect.top + Math.min(220, listRect.height * 0.28);
    let bestEntryId = pageEntries[0].id;
    let bestDistance = Number.POSITIVE_INFINITY;
    for (const entry of pageEntries) {
      const frame = pageFrameRefs.current[entry.id];
      if (!frame) {
        continue;
      }
      const rect = frame.getBoundingClientRect();
      const distance = Math.abs(rect.top - anchorY);
      if (distance < bestDistance) {
        bestDistance = distance;
        bestEntryId = entry.id;
      }
    }
    setSelectedPageId((current) => (current === bestEntryId ? current : bestEntryId));
  }

  const loadPdf = useCallback(async (bytes: Uint8Array, nextFileName: string) => {
    const loadingTask = pdfjsLib.getDocument({ data: bytes.slice() });
    const loadedPdf = await loadingTask.promise;
    const loadedPages: PdfPageProxy[] = [];
    const sizes: PdfPageSize[] = [];
    for (let pageNumber = 1; pageNumber <= loadedPdf.numPages; pageNumber += 1) {
      const page = await loadedPdf.getPage(pageNumber);
      const viewport = page.getViewport({ scale: 1 });
      loadedPages.push(page);
      sizes.push({ pageNumber, width: viewport.width, height: viewport.height });
    }
    const entries = loadedPages.map((page) => ({
      id: makePageId(page.pageNumber),
      sourcePageNumber: page.pageNumber,
      rotation: 0 as const,
    }));
    setPdfBytes(bytes);
    setPdfDoc(loadedPdf);
    setPages(loadedPages);
    setPageSizes(sizes);
    setPageEntries(entries);
    setSelectedPageId(entries[0]?.id ?? null);
    const nextTextHitBoxes: PdfTextHitBox[] = [];
    for (const entry of entries) {
      const page = loadedPages[entry.sourcePageNumber - 1];
      const textContent = await page.getTextContent();
      const viewport = page.getViewport({ scale: 1 });
      textContent.items.forEach((item, itemIndex) => {
        if (!("str" in item) || item.str.trim() === "") {
          return;
        }
        const transform = pdfjsLib.Util.transform(viewport.transform, item.transform);
        const fontHeight = Math.hypot(transform[2], transform[3]) || Math.abs(transform[3]) || 12;
        const width = Math.max(item.width + 10, item.str.length * fontHeight * 0.56);
        nextTextHitBoxes.push({
          id: `${entry.id}-${itemIndex}`,
          pageId: entry.id,
          sourcePageNumber: entry.sourcePageNumber,
          text: item.str,
          x: transform[4],
          y: transform[5] - fontHeight,
          width,
          height: fontHeight * 1.25,
          fontSize: fontHeight,
        });
      });
    }
    setTextHitBoxes(nextTextHitBoxes);
    setFileName(nextFileName);
    setObjects([]);
    setImageAssets([]);
    setPendingImageAssetId(null);
    setSelectedId(null);
    setUndoStack([]);
    setRedoStack([]);
    setNativeAnalysis(null);
    setHasUnsavedChanges(false);
    setStatus(`已载入 ${loadedPdf.numPages} 页`);
  }, []);

  async function analyzeNativeContent() {
    if (!pdfBytes) {
      return;
    }
    if (!hasDesktopRuntime()) {
      setStatus("MuPDF 内容流分析仅在桌面端可用");
      return;
    }
    setIsAnalyzingNative(true);
    setStatus("正在用 MuPDF 分析 PDF 内容流...");
    try {
      const analysis = await invoke<PdfNativeEditAnalysis>("analyze_pdf_native_editability", { bytes: Array.from(pdfBytes) });
      setNativeAnalysis(analysis);
      const candidateCount = analysis.pages.reduce(
        (total, page) =>
          total +
          page.lines.reduce(
            (lineTotal, line) => lineTotal + line.fragments.filter((fragment) => fragment.editability === "native-candidate").length,
            0,
          ),
        0,
      );
      setStatus(`MuPDF 分析完成：${analysis.pages.length} 页，${candidateCount} 个文本片段可优先尝试内容流编辑`);
    } catch (error) {
      setStatus(`MuPDF 分析失败：${formatExportError(error)}`);
    } finally {
      setIsAnalyzingNative(false);
    }
  }

  async function handleOpenFile(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    const bytes = new Uint8Array(await file.arrayBuffer());
    await loadPdf(bytes, file.name);
    event.target.value = "";
  }

  async function handleImageFile(event: React.ChangeEvent<HTMLInputElement>, nextTool: "image" | "signature") {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    if (!["image/png", "image/jpeg"].includes(file.type)) {
      setStatus("仅支持插入 PNG 或 JPEG 图片");
      event.target.value = "";
      return;
    }
    const bytes = new Uint8Array(await file.arrayBuffer());
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject(new Error("读取图片失败"));
      reader.readAsDataURL(file);
    });
    const size = await readImageSize(dataUrl);
    const asset: PdfImageAsset = {
      id: makeImageId(),
      bytes,
      dataUrl,
      mimeType: file.type,
      width: size.width,
      height: size.height,
    };
    rememberHistory();
    setImageAssets((current) => [...current, asset]);
    setPendingImageAssetId(asset.id);
    setTool(nextTool);
    setStatus(nextTool === "signature" ? "点击页面放置签名图片" : "点击页面放置图片");
    event.target.value = "";
  }

  function addObject(event: React.PointerEvent<HTMLDivElement>, pageEntry: PdfPageEntry) {
    if (tool === "select") {
      clearEmptySelectedObject();
      setSelectedPageId(pageEntry.id);
      return;
    }
    clearEmptySelectedObject();
    if (pageEntry.rotation !== 0) {
      setStatus("旋转后的页面请先转回 0 度再添加覆盖对象");
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const nextObject = createObject(tool, pageEntry.id, (event.clientX - rect.left) / scale, (event.clientY - rect.top) / scale, pendingImageAsset);
    if (!nextObject) {
      return;
    }
    rememberHistory();
    setObjects((current) => [...current, nextObject]);
    setSelectedId(nextObject.id);
    setSelectedPageId(pageEntry.id);
    if (tool === "image" || tool === "signature") {
      setPendingImageAssetId(null);
    }
    setTool("select");
  }

  function updateObjectPosition(id: string, x: number, y: number) {
    setObjects((current) => current.map((object) => (object.id === id ? { ...object, x, y } : object)));
    setHasUnsavedChanges(true);
  }

  function updateObjectText(id: string, text: string) {
    setObjects((current) => current.map((object) => (object.id === id && "text" in object ? { ...object, text } : object)));
    setHasUnsavedChanges(true);
  }

  function updateObjectSize(id: string, width: number, height: number, x?: number, y?: number) {
    setObjects((current) =>
      current.map((object) => (object.id === id ? { ...object, width, height, x: x ?? object.x, y: y ?? object.y } : object)),
    );
    setHasUnsavedChanges(true);
  }

  function isEmptyDraftObject(object: PdfEditObject) {
    return (
      (object.type === "text" && (object.text.trim() === "" || object.text === DEFAULT_TEXT_OBJECT_TEXT)) ||
      (object.type === "replace-text" && object.text === object.originalText)
    );
  }

  function replaceOriginalText(hitBox: PdfTextHitBox) {
    const replacement = createReplaceTextObject(hitBox);
    clearEmptySelectedObject();
    rememberHistory();
    setObjects((current) => [...current, replacement]);
    setSelectedId(replacement.id);
    setSelectedPageId(hitBox.pageId);
    setTool("select");
    setStatus("已创建原文替换框，可直接输入新文字；留空导出时会删除该段文字");
  }

  function removeEmptyDraftObject(id: string) {
    setObjects((current) => current.filter((object) => object.id !== id || !isEmptyDraftObject(object)));
    setSelectedId((current) => (current === id ? null : current));
  }

  function finishTextEditing(id: string) {
    const object = objects.find((item) => item.id === id);
    if (!object) {
      return;
    }
    if (isEmptyDraftObject(object)) {
      removeEmptyDraftObject(id);
      if (object.type === "replace-text") {
        setTool("replace-text");
        setStatus("可继续点击页面中的文字进行原文替换");
      }
      return;
    }
    setSelectedId((current) => (current === id ? null : current));
    if (object.type === "replace-text") {
      setTool("replace-text");
      setStatus("替换内容已暂存，导出后会写入新 PDF");
    }
  }

  function clearEmptySelectedObject() {
    if (!selectedId) {
      return;
    }
    removeEmptyDraftObject(selectedId);
  }

  function deleteSelected() {
    if (!selectedId) {
      return;
    }
    rememberHistory();
    setObjects((current) => current.filter((object) => object.id !== selectedId));
    setSelectedId(null);
  }

  function moveSelectedPage(direction: -1 | 1) {
    if (!selectedPageId) {
      return;
    }
    rememberHistory();
    setPageEntries((current) => {
      const index = current.findIndex((entry) => entry.id === selectedPageId);
      const target = index + direction;
      if (index < 0 || target < 0 || target >= current.length) {
        return current;
      }
      const next = [...current];
      [next[index], next[target]] = [next[target], next[index]];
      return next;
    });
  }

  function rotateSelectedPage(direction: -1 | 1) {
    if (!selectedPageId) {
      return;
    }
    rememberHistory();
    setPageEntries((current) =>
      current.map((entry) =>
        entry.id === selectedPageId
          ? { ...entry, rotation: (((entry.rotation + direction * 90 + 360) % 360) as PdfPageEntry["rotation"]) }
          : entry,
      ),
    );
  }

  function duplicateSelectedPage() {
    if (!selectedPage) {
      return;
    }
    rememberHistory();
    const duplicateId = makePageId(selectedPage.sourcePageNumber);
    setPageEntries((current) => {
      const index = current.findIndex((entry) => entry.id === selectedPage.id);
      const next = [...current];
      next.splice(index + 1, 0, { ...selectedPage, id: duplicateId });
      return next;
    });
    setObjects((current) => [
      ...current,
      ...current
        .filter((object) => object.pageId === selectedPage.id)
        .map((object) => ({ ...object, id: makeObjectId(), pageId: duplicateId })),
    ]);
    setTextHitBoxes((current) => [
      ...current,
      ...current
        .filter((hitBox) => hitBox.pageId === selectedPage.id)
        .map((hitBox, index) => ({ ...hitBox, id: `${duplicateId}-${index}`, pageId: duplicateId })),
    ]);
    setSelectedPageId(duplicateId);
    setSelectedId(null);
  }

  function deleteSelectedPage() {
    if (!selectedPageId || pageEntries.length <= 1) {
      return;
    }
    rememberHistory();
    setPageEntries((current) => {
      const next = current.filter((entry) => entry.id !== selectedPageId);
      setSelectedPageId(next[0]?.id ?? null);
      return next;
    });
    setObjects((current) => current.filter((object) => object.pageId !== selectedPageId));
    setTextHitBoxes((current) => current.filter((hitBox) => hitBox.pageId !== selectedPageId));
    setSelectedId(null);
  }

  async function exportPdf() {
    if (!pdfBytes || pageEntries.length === 0) {
      return;
    }
    setIsExporting(true);
    setStatus("正在生成导出 PDF...");
    try {
      const sourceDoc = await PDFDocument.load(pdfBytes);
      const outputDoc = await PDFDocument.create();
      const exportFont = await embedExportFont(outputDoc);
      const copiedPages = await outputDoc.copyPages(
        sourceDoc,
        pageEntries.map((entry) => entry.sourcePageNumber - 1),
      );
      for (const [index, page] of copiedPages.entries()) {
        const entry = pageEntries[index];
        const pageObjects = objects.filter((object) => object.pageId === entry.id);
        const pageHeight = page.getHeight();
        for (const object of pageObjects) {
          if (object.type === "text") {
            page.drawText(object.text, {
              x: object.x,
              y: pageHeight - object.y - object.fontSize,
              size: object.fontSize,
              font: exportFont,
              color: hexToRgb(object.color),
              lineHeight: object.fontSize * 1.25,
              maxWidth: object.width,
            });
          }
          if (object.type === "replace-text") {
            page.drawRectangle({
              x: object.x,
              y: pageHeight - object.y - object.height,
              width: object.width,
              height: object.height,
              color: rgb(1, 1, 1),
            });
            if (object.text.trim()) {
              page.drawText(object.text, {
                x: object.x + 2,
                y: pageHeight - object.y - object.height + Math.max(2, (object.height - object.fontSize) / 2),
                size: object.fontSize,
                font: exportFont,
                color: hexToRgb(object.color),
                lineHeight: object.fontSize * 1.2,
                maxWidth: object.width - 4,
              });
            }
          }
          if (object.type === "highlight") {
            page.drawRectangle({
              x: object.x,
              y: pageHeight - object.y - object.height,
              width: object.width,
              height: object.height,
              color: hexToRgb(object.color),
              opacity: object.opacity,
            });
          }
          if (object.type === "rect") {
            page.drawRectangle({
              x: object.x,
              y: pageHeight - object.y - object.height,
              width: object.width,
              height: object.height,
              borderColor: hexToRgb(object.strokeColor),
              borderWidth: object.strokeWidth,
            });
          }
          if (object.type === "image" || object.type === "signature") {
            const asset = imageAssets.find((item) => item.id === object.imageId);
            if (asset) {
              const embeddedImage =
                asset.mimeType === "image/png" ? await outputDoc.embedPng(asset.bytes) : await outputDoc.embedJpg(asset.bytes);
              page.drawImage(embeddedImage, {
                x: object.x,
                y: pageHeight - object.y - object.height,
                width: object.width,
                height: object.height,
              });
            }
          }
          if (object.type === "note") {
            page.drawRectangle({
              x: object.x,
              y: pageHeight - object.y - object.height,
              width: object.width,
              height: object.height,
              color: rgb(1, 0.96, 0.62),
              borderColor: rgb(0.88, 0.68, 0.16),
              borderWidth: 1,
              opacity: 0.92,
            });
            page.drawText(object.text, {
              x: object.x + 10,
            y: pageHeight - object.y - 22,
            size: 12,
            font: exportFont,
              color: rgb(0.25, 0.21, 0.12),
              lineHeight: 15,
              maxWidth: object.width - 20,
            });
          }
        }
        const currentRotation = page.getRotation().angle;
        page.setRotation(degrees((currentRotation + entry.rotation) % 360));
        outputDoc.addPage(page);
      }
      const editedBytes = await outputDoc.save();
      const defaultFileName = fileName.replace(/\.pdf$/i, "") + "-edited.pdf";
      if (hasDesktopRuntime()) {
        const targetPath = await save({
          defaultPath: defaultFileName,
          filters: [{ name: "PDF 文件", extensions: ["pdf"] }],
        });
        if (!targetPath) {
          setStatus("已取消导出");
          return;
        }
        await invoke("save_pdf_file", { path: targetPath, bytes: Array.from(editedBytes) });
        setHasUnsavedChanges(false);
        setStatus(`已导出到 ${targetPath}`);
      } else {
        downloadPdfInBrowser(editedBytes, defaultFileName);
        setHasUnsavedChanges(false);
        setStatus(`已导出 ${pageEntries.length} 页、${objects.length} 个覆盖编辑对象`);
      }
    } catch (error) {
      setStatus(`导出失败：${formatExportError(error)}`);
    } finally {
      setIsExporting(false);
    }
  }

  return (
    <section className="pdf-editor-shell">
      <input ref={fileInputRef} type="file" accept="application/pdf,.pdf" className="sr-only" onChange={handleOpenFile} />
      <input ref={imageInputRef} type="file" accept="image/png,image/jpeg" className="sr-only" onChange={(event) => handleImageFile(event, "image")} />
      <input
        ref={signatureInputRef}
        type="file"
        accept="image/png,image/jpeg"
        className="sr-only"
        onChange={(event) => handleImageFile(event, "signature")}
      />
      <div className="pdf-toolbar">
        <div className="pdf-toolbar-group">
          <Button variant="primary" onClick={() => fileInputRef.current?.click()}>
            <FileUp size={16} />
            打开 PDF
          </Button>
          <Button variant="secondary" disabled={!pdfBytes || pageEntries.length === 0} loading={isExporting} onClick={exportPdf}>
            <Download size={16} />
            导出
          </Button>
          <Button variant="secondary" disabled={!pdfBytes || isAnalyzingNative} loading={isAnalyzingNative} onClick={analyzeNativeContent}>
            <ScanText size={16} />
            内容分析
          </Button>
          <Button variant="secondary" disabled={!pdfDoc} onClick={() => imageInputRef.current?.click()}>
            <ImageIcon size={16} />
            插入图片
          </Button>
          <Button variant="secondary" disabled={!pdfDoc} onClick={() => signatureInputRef.current?.click()}>
            <PenLine size={16} />
            插入签名
          </Button>
        </div>
        <div className="pdf-toolbar-group pdf-tool-group" role="toolbar" aria-label="PDF 编辑工具">
          {(Object.keys(TOOL_LABELS) as PdfEditTool[]).map((item) => {
            const Icon = TOOL_ICONS[item];
            return (
              <button
                key={item}
                type="button"
                className={`pdf-tool-btn ${tool === item ? "active" : ""}`}
                aria-label={TOOL_LABELS[item]}
                data-tooltip={TOOL_LABELS[item]}
                onClick={() => setTool(item)}
              >
                <Icon size={17} />
              </button>
            );
          })}
        </div>
        <div className="pdf-toolbar-group">
          <button type="button" className="pdf-icon-btn" aria-label="撤销" data-tooltip="撤销" disabled={undoStack.length === 0} onClick={undoEdit}>
            <Undo2 size={17} />
          </button>
          <button type="button" className="pdf-icon-btn" aria-label="重做" data-tooltip="重做" disabled={redoStack.length === 0} onClick={redoEdit}>
            <Redo2 size={17} />
          </button>
          <button type="button" className="pdf-icon-btn" aria-label="缩小" onClick={() => setScale((current) => Math.max(0.7, current - 0.1))}>
            <ZoomOut size={17} />
          </button>
          <span className="pdf-zoom-label">{Math.round(scale * 100)}%</span>
          <button type="button" className="pdf-icon-btn" aria-label="放大" onClick={() => setScale((current) => Math.min(2, current + 0.1))}>
            <ZoomIn size={17} />
          </button>
          <button
            type="button"
            className="pdf-labeled-btn danger"
            aria-label="删除选中对象"
            data-tooltip="删除当前选中的文字、矩形、高亮或便签"
            disabled={!selectedObject}
            onClick={deleteSelected}
          >
            <Trash2 size={17} />
            <span>删除对象</span>
          </button>
        </div>
        <div className="pdf-toolbar-group">
          <button
            type="button"
            className="pdf-icon-btn"
            aria-label="上移页面"
            data-tooltip="上移页面"
            disabled={!selectedPage || selectedPageIndex <= 0}
            onClick={() => moveSelectedPage(-1)}
          >
            <ArrowUp size={17} />
          </button>
          <button
            type="button"
            className="pdf-icon-btn"
            aria-label="下移页面"
            data-tooltip="下移页面"
            disabled={!selectedPage || selectedPageIndex < 0 || selectedPageIndex >= pageEntries.length - 1}
            onClick={() => moveSelectedPage(1)}
          >
            <ArrowDown size={17} />
          </button>
          <button
            type="button"
            className="pdf-icon-btn"
            aria-label="逆时针旋转页面"
            data-tooltip="逆时针旋转页面"
            disabled={!selectedPage}
            onClick={() => rotateSelectedPage(-1)}
          >
            <RotateCcw size={17} />
          </button>
          <button
            type="button"
            className="pdf-icon-btn"
            aria-label="顺时针旋转页面"
            data-tooltip="顺时针旋转页面"
            disabled={!selectedPage}
            onClick={() => rotateSelectedPage(1)}
          >
            <RotateCw size={17} />
          </button>
          <button type="button" className="pdf-icon-btn" aria-label="复制页面" data-tooltip="复制页面" disabled={!selectedPage} onClick={duplicateSelectedPage}>
            <Copy size={17} />
          </button>
          <button
            type="button"
            className="pdf-labeled-btn danger"
            aria-label="删除页面"
            data-tooltip="删除当前选中的整页"
            disabled={!selectedPage || pageEntries.length <= 1}
            onClick={deleteSelectedPage}
          >
            <Trash2 size={17} />
            <span>删除页面</span>
          </button>
        </div>
      </div>


      {nativeAnalysis ? (
        <div className="pdf-native-panel">
          <div className="pdf-native-panel-title">
            <strong>内容流分析</strong>
            <span>
              {selectedNativePage
                ? `当前页 ${selectedNativePage.lines.length} 行 / ${
                    selectedNativePage.lines.flatMap((line) => line.fragments).filter((fragment) => fragment.editability === "native-candidate").length
                  } 个候选片段`
                : `${nativeAnalysis.engine} 已分析 ${nativeAnalysis.pages.length} 页`}
            </span>
          </div>
          <div className="pdf-native-panel-body">
            {selectedNativePage ? (
              selectedNativePage.lines.slice(0, 4).map((line, index) => (
                <span key={`${selectedNativePage.pageNumber}-${index}`}>
                  {line.text || "空文本行"}
                </span>
              ))
            ) : (
              <span>选择页面后查看该页文本结构。</span>
            )}
            {nativeAnalysis.warnings[0] ? <small>{nativeAnalysis.warnings[0]}</small> : null}
          </div>
        </div>
      ) : null}

      {selectedObject ? (
        <div className="pdf-properties-panel">
          <div className="pdf-properties-title">
            <strong>对象属性</strong>
            <span>{TOOL_LABELS[selectedObject.type === "replace-text" ? "replace-text" : selectedObject.type]}</span>
          </div>
          {"text" in selectedObject ? (
            <label>
              内容
              <textarea
                value={selectedObject.text}
                onChange={(event) => updateObject(selectedObject.id, { text: event.target.value } as Partial<PdfEditObject>)}
              />
            </label>
          ) : null}
          {(selectedObject.type === "image" || selectedObject.type === "signature") ? (
            <label>
              类型
              <input value={selectedObject.type === "signature" ? "签名图片" : "图片"} readOnly />
            </label>
          ) : null}
          {(selectedObject.type === "text" || selectedObject.type === "replace-text") ? (
            <label>
              字号
              <input
                type="number"
                min="8"
                max="96"
                value={Math.round(selectedObject.fontSize)}
                onChange={(event) => updateObject(selectedObject.id, { fontSize: Number(event.target.value) || 12 } as Partial<PdfEditObject>)}
              />
            </label>
          ) : null}
          {"color" in selectedObject ? (
            <label>
              颜色
              <input
                type="color"
                value={selectedObject.color}
                onChange={(event) => updateObject(selectedObject.id, { color: event.target.value } as Partial<PdfEditObject>)}
              />
            </label>
          ) : null}
          {selectedObject.type === "highlight" ? (
            <label>
              透明度
              <input
                type="range"
                min="0.1"
                max="0.8"
                step="0.05"
                value={selectedObject.opacity}
                onChange={(event) => updateObject(selectedObject.id, { opacity: Number(event.target.value) } as Partial<PdfEditObject>)}
              />
            </label>
          ) : null}
          {selectedObject.type === "rect" ? (
            <>
              <label>
                描边颜色
                <input
                  type="color"
                  value={selectedObject.strokeColor}
                  onChange={(event) => updateObject(selectedObject.id, { strokeColor: event.target.value } as Partial<PdfEditObject>)}
                />
              </label>
              <label>
                线宽
                <input
                  type="number"
                  min="1"
                  max="12"
                  value={selectedObject.strokeWidth}
                  onChange={(event) => updateObject(selectedObject.id, { strokeWidth: Number(event.target.value) || 1 } as Partial<PdfEditObject>)}
                />
              </label>
            </>
          ) : null}
          <label>
            宽
            <input
              type="number"
              min="20"
              value={Math.round(selectedObject.width)}
              onChange={(event) => updateObject(selectedObject.id, { width: Number(event.target.value) || selectedObject.width } as Partial<PdfEditObject>)}
            />
          </label>
          <label>
            高
            <input
              type="number"
              min="18"
              value={Math.round(selectedObject.height)}
              onChange={(event) => updateObject(selectedObject.id, { height: Number(event.target.value) || selectedObject.height } as Partial<PdfEditObject>)}
            />
          </label>
        </div>
      ) : null}

      <div className="pdf-workspace">
        {!pdfDoc ? (
          <div className="pdf-empty-state">
            <Plus size={32} />
            <strong>选择 PDF 文件</strong>
            <span>打开后可添加文字、高亮、矩形和便签覆盖编辑，并导出为新的 PDF。</span>
          </div>
        ) : (
          <div className={`pdf-editor-grid ${isThumbnailPanelOpen ? "" : "thumb-collapsed"}`}>
            <aside className="pdf-page-sidebar" aria-label="页面列表">
              <div className="pdf-thumb-panel-head">
                <strong>{isThumbnailPanelOpen ? "缩略图" : ""}</strong>
                <button
                  type="button"
                  className="pdf-panel-toggle"
                  aria-label={isThumbnailPanelOpen ? "收起缩略图" : "展开缩略图"}
                  onClick={() => setIsThumbnailPanelOpen((current) => !current)}
                >
                  {isThumbnailPanelOpen ? <PanelLeftClose size={17} /> : <PanelLeftOpen size={17} />}
                </button>
              </div>
              {isThumbnailPanelOpen ? (
                <>
                  <div className="pdf-thumb-size-control">
                    <ImageIcon size={14} />
                    <input
                      type="range"
                      min="56"
                      max="220"
                      step="2"
                      value={thumbnailScale}
                      onChange={(event) => setThumbnailScale(Number(event.target.value))}
                      aria-label="缩略图大小"
                    />
                    <ImageIcon size={18} />
                  </div>
                  <div className="pdf-thumb-list">
                    {pageEntries.map((entry, index) => {
                      const thumbPage = pageLookup.get(entry.sourcePageNumber);
                      return (
                        <button
                          key={entry.id}
                          ref={(node) => {
                            thumbnailRefs.current[entry.id] = node;
                          }}
                          type="button"
                          className={`pdf-page-thumb ${entry.id === selectedPageId ? "active" : ""}`}
                          onClick={() => selectPage(entry.id)}
                        >
                          {thumbPage ? (
                            <PdfPageThumbnail
                              page={thumbPage}
                              rotation={entry.rotation}
                              selected={entry.id === selectedPageId}
                              targetWidth={thumbnailScale}
                            />
                          ) : null}
                          <span>第{index + 1}页</span>
                        </button>
                      );
                    })}
                  </div>
                </>
              ) : null}
            </aside>
            <div className="pdf-page-list" ref={pageListRef} onScroll={syncCurrentPageFromScroll}>
            {pageEntries.map((entry, index) => {
              const page = pageLookup.get(entry.sourcePageNumber);
              const size = pageSizeLookup.get(entry.sourcePageNumber);
              if (!page || !size) {
                return null;
              }
              const isSideways = entry.rotation === 90 || entry.rotation === 270;
              return (
                <div
                  key={entry.id}
                  ref={(node) => {
                    pageFrameRefs.current[entry.id] = node;
                  }}
                  className={`pdf-page-frame ${entry.id === selectedPageId ? "active" : ""}`}
                >
                  <div className="pdf-page-label">
                    第 {index + 1} 页
                    <span>源第 {entry.sourcePageNumber} 页{entry.rotation ? ` / ${entry.rotation} 度` : ""}</span>
                  </div>
                  <div
                    className={`pdf-page-stage ${tool !== "select" ? "placing" : ""}`}
                    style={{
                      width: (isSideways ? size.height : size.width) * scale,
                      height: (isSideways ? size.width : size.height) * scale,
                    }}
                    onPointerDown={(event) => addObject(event, entry)}
                    >
                    <PdfPageCanvas page={page} scale={scale} rotation={entry.rotation} />
                    {tool === "replace-text" && entry.rotation === 0
                      ? textHitBoxes
                          .filter((hitBox) => hitBox.pageId === entry.id)
                          .map((hitBox) => (
                            <button
                              key={hitBox.id}
                              type="button"
                              className="pdf-text-hitbox"
                              style={{
                                left: hitBox.x * scale,
                                top: hitBox.y * scale,
                                width: hitBox.width * scale,
                                height: hitBox.height * scale,
                              }}
                              title={`替换：${hitBox.text}`}
                              onPointerDown={(event) => event.stopPropagation()}
                              onClick={(event) => {
                                event.stopPropagation();
                                replaceOriginalText(hitBox);
                              }}
                            />
                          ))
                      : null}
                    {objects
                      .filter((object) => object.pageId === entry.id && entry.rotation === 0)
                      .map((object) => (
                        <PdfEditOverlay
                          key={object.id}
                          object={object}
                          imageAsset={"imageId" in object ? imageAssetLookup.get(object.imageId) : undefined}
                          selected={object.id === selectedId}
                          scale={scale}
                          onSelect={setSelectedId}
                          onMove={updateObjectPosition}
                          onResize={updateObjectSize}
                          onChangeStart={rememberHistory}
                          onTextChange={updateObjectText}
                          onTextBlur={finishTextEditing}
                        />
                      ))}
                  </div>
                </div>
              );
            })}
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
