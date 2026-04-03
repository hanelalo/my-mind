import { invoke } from "@tauri-apps/api/core";

export interface HistoryRecord {
  id: string;
  timestamp: number;
  asr_text: string;
  final_text: string;
  target_app: string | null;
}

export interface DiagnosisMessage {
  role: "user" | "assistant";
  content: string;
}

export interface DiagnosisRequest {
  asr_text: string;
  final_text: string;
  user_message: string;
  conversation_history: DiagnosisMessage[];
}

export interface DiagnosisResponse {
  reply: string;
}

export interface QualityCheckRequest {
  asr_text: string;
  final_text: string;
}

export interface QualityCheckResponse {
  report: string;
}

export interface ApplyPromptSuggestionRequest {
  suggestions: string;
}

export interface ApplyPromptSuggestionResponse {
  new_prompt: string;
}

export async function getHistory(
  limit: number,
  offset: number
): Promise<HistoryRecord[]> {
  return invoke("get_history", { limit, offset });
}

export async function getHistoryCount(): Promise<number> {
  return invoke("get_history_count");
}

export async function deleteHistoryRecord(id: string): Promise<void> {
  return invoke("delete_history_record", { id });
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export async function diagnosePrompt(
  request: DiagnosisRequest
): Promise<DiagnosisResponse> {
  return invoke("diagnose_prompt", { request });
}

export async function checkQuality(
  request: QualityCheckRequest
): Promise<QualityCheckResponse> {
  return invoke("check_quality", { request });
}

export async function applyPromptSuggestion(
  request: ApplyPromptSuggestionRequest
): Promise<ApplyPromptSuggestionResponse> {
  return invoke("apply_prompt_suggestion", { request });
}
