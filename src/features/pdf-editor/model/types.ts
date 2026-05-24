export type PdfEditTool = "select" | "text" | "replace-text" | "highlight" | "rect" | "note" | "image" | "signature";

export interface PdfPageSize {
  pageNumber: number;
  width: number;
  height: number;
}

export interface PdfPageEntry {
  id: string;
  sourcePageNumber: number;
  rotation: 0 | 90 | 180 | 270;
}

interface BasePdfEditObject {
  id: string;
  pageId: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PdfTextObject extends BasePdfEditObject {
  type: "text";
  text: string;
  fontSize: number;
  color: string;
}

export interface PdfReplaceTextObject extends BasePdfEditObject {
  type: "replace-text";
  originalText: string;
  text: string;
  fontSize: number;
  color: string;
}

export interface PdfHighlightObject extends BasePdfEditObject {
  type: "highlight";
  color: string;
  opacity: number;
}

export interface PdfRectObject extends BasePdfEditObject {
  type: "rect";
  strokeColor: string;
  strokeWidth: number;
}

export interface PdfNoteObject extends BasePdfEditObject {
  type: "note";
  text: string;
}

export interface PdfImageObject extends BasePdfEditObject {
  type: "image" | "signature";
  imageId: string;
  mimeType: string;
}

export type PdfEditObject = PdfTextObject | PdfReplaceTextObject | PdfHighlightObject | PdfRectObject | PdfNoteObject | PdfImageObject;
