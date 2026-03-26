import { invoke } from "@tauri-apps/api/core";

export interface AsrOnlineConfig {
  api_key: string;
  api_base_url: string | null;
  model: string | null;
}

export interface AsrConfig {
  mode: string;
  language: string;
  online: AsrOnlineConfig;
}

export interface LlmConfig {
  provider: string;
  model: string;
  api_key: string;
  api_base_url: string | null;
  temperature: number;
  max_tokens: number;
  enabled: boolean;
}

export interface ShortcutConfig {
  record: string;
  mode: string;
}

export interface OutputConfig {
  auto_paste: boolean;
}

export interface AppConfig {
  asr: AsrConfig;
  llm: LlmConfig;
  shortcuts: ShortcutConfig;
  output: OutputConfig;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}
