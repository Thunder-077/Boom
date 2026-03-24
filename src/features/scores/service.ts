import { invoke } from "@tauri-apps/api/core";
import type {
  ScoreDetail,
  ImportResult,
  LatestScoreSummary,
  ScoreQuery,
  ScoreRow,
  ScoreUpdatePayload,
} from "../../entities/score/model";
import type { ListResult, PageQuery } from "../../shared/types/api";

export interface ScoreService {
  list(params: ScoreQuery & PageQuery): Promise<ListResult<ScoreRow>>;
  getLatestSummary(): Promise<LatestScoreSummary>;
  importExcel(filePath: string): Promise<ImportResult>;
  getDetail(admissionNo: string): Promise<ScoreDetail>;
  updateScore(payload: ScoreUpdatePayload): Promise<{ success: boolean }>;
}

export const scoreService: ScoreService = {
  list(params) {
    return invoke<ListResult<ScoreRow>>("list_latest_score_rows", { params });
  },
  getLatestSummary() {
    return invoke<LatestScoreSummary>("get_latest_summary");
  },
  importExcel(filePath) {
    return invoke<ImportResult>("import_scores_from_excel", { filePath });
  },
  getDetail(admissionNo) {
    return invoke<ScoreDetail>("get_score_detail", { admissionNo });
  },
  updateScore(payload) {
    return invoke<{ success: boolean }>("update_score_row", { payload });
  },
};
