import { createSignal, onMount, Show } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getConfig, saveConfig } from "./lib/config";
import type { AppConfig } from "./lib/config";
import AsrSection from "./components/settings/AsrSection";
import LlmSection from "./components/settings/LlmSection";
import ShortcutSection from "./components/settings/ShortcutSection";
import OutputSection from "./components/settings/OutputSection";

const defaultConfig: AppConfig = {
  asr: {
    mode: "online",
    language: "zh",
    online: { api_key: "", api_base_url: null, model: null },
  },
  llm: {
    provider: "openai",
    model: "gpt-4o-mini",
    api_key: "",
    api_base_url: null,
    temperature: 0.3,
    max_tokens: 2048,
    enabled: true,
  },
  shortcuts: { record: "Alt+Space", mode: "push_to_talk" },
  output: { auto_paste: true },
};

export default function SettingsApp() {
  const [config, setConfig] = createStore<AppConfig>(defaultConfig);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [message, setMessage] = createSignal<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  onMount(async () => {
    try {
      const loaded = await getConfig();
      setConfig(reconcile(loaded));
    } catch (e) {
      console.error("Failed to load config:", e);
      setMessage({ type: "error", text: `Failed to load config: ${e}` });
    } finally {
      setLoading(false);
    }
  });

  const handleSave = async () => {
    setSaving(true);
    setMessage(null);
    try {
      await saveConfig(JSON.parse(JSON.stringify(config)));
      setMessage({ type: "success", text: "Settings saved" });
      setTimeout(() => {
        getCurrentWindow().close();
      }, 500);
    } catch (e) {
      console.error("Failed to save config:", e);
      setMessage({ type: "error", text: `Failed to save: ${e}` });
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    getCurrentWindow().close();
  };

  return (
    <div class="flex min-h-screen flex-col bg-gray-50">
      {/* Header */}
      <div class="border-b border-gray-200 bg-white px-6 py-4">
        <h1 class="text-lg font-semibold text-gray-900">Settings</h1>
      </div>

      {/* Content */}
      <Show
        when={!loading()}
        fallback={
          <div class="flex flex-1 items-center justify-center">
            <p class="text-sm text-gray-500">Loading...</p>
          </div>
        }
      >
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <div class="mx-auto max-w-xl space-y-4">
            <AsrSection
              config={config.asr}
              onOnlineChange={(key, value) =>
                setConfig("asr", "online", key, value as any)
              }
            />
            <LlmSection
              config={config.llm}
              onChange={(key, value) => setConfig("llm", key, value as any)}
            />
            <ShortcutSection
              config={config.shortcuts}
              onChange={(key, value) =>
                setConfig("shortcuts", key, value as any)
              }
            />
            <OutputSection
              config={config.output}
              onChange={(key, value) => setConfig("output", key, value as any)}
            />
          </div>
        </div>

        {/* Footer */}
        <div class="border-t border-gray-200 bg-white px-6 py-3">
          <div class="mx-auto flex max-w-xl items-center justify-between">
            <Show when={message()}>
              {(msg) => (
                <p
                  class="text-sm"
                  classList={{
                    "text-green-600": msg().type === "success",
                    "text-red-600": msg().type === "error",
                  }}
                >
                  {msg().text}
                </p>
              )}
            </Show>
            <div class="ml-auto flex gap-2">
              <button
                class="rounded-md border border-gray-300 bg-white px-4 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
                onClick={handleCancel}
              >
                Cancel
              </button>
              <button
                class="rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
                onClick={handleSave}
                disabled={saving()}
              >
                {saving() ? "Saving..." : "Save"}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
