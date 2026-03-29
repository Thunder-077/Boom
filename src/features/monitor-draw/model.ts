export type PairMode = "fixed" | "random";

export type StepKey =
  | "home"
  | "rooms"
  | "import"
  | "mode"
  | "draw"
  | "result";

export type AnimationPhase =
  | "idle"
  | "pair_rolling"
  | "room_fast"
  | "room_slow"
  | "room_hit"
  | "card_slide"
  | "completed";

export interface ImportedInvigilatorRow {
  groupNo: string;
  invigilatorAName: string;
  invigilatorBName: string;
}

export interface InvigilatorPair {
  id: string;
  groupNo: string;
  invigilatorA: string;
  invigilatorB: string;
  mode: PairMode;
}

export interface RoomItem {
  id: string;
  roomNo: string;
}

export interface DrawAssignment {
  id: string;
  roomNo: string;
  invigilatorA: string;
  invigilatorB: string;
  pairMode: PairMode;
  statusTag: "待分配" | "正在抽取" | "已落位";
  assigned: boolean;
}

export interface DrawStatus {
  phase: AnimationPhase;
  isDrawing: boolean;
  progress: number;
  currentPairIndex: number;
  currentHighlightRoomIndex: number;
}
