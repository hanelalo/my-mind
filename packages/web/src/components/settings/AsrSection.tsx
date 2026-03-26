import type { AsrConfig } from "../../lib/config";

interface Props {
  config: AsrConfig;
  onOnlineChange: <K extends keyof AsrConfig["online"]>(
    key: K,
    value: AsrConfig["online"][K]
  ) => void;
}

const inputClass =
  "w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500";
const labelClass = "block text-sm font-medium text-gray-700 mb-1";

export default function AsrSection(props: Props) {
  return (
    <section class="rounded-lg border border-gray-200 bg-white p-4">
      <h2 class="mb-3 text-sm font-semibold text-gray-900">
        ASR (Speech Recognition)
      </h2>
      <div class="grid grid-cols-1 gap-3">
        <div>
          <label class={labelClass}>API Key</label>
          <input
            type="password"
            class={inputClass}
            value={props.config.online.api_key}
            onInput={(e) =>
              props.onOnlineChange("api_key", e.currentTarget.value)
            }
            placeholder="Enter API key"
          />
        </div>
        <div>
          <label class={labelClass}>API Base URL</label>
          <input
            type="text"
            class={inputClass}
            value={props.config.online.api_base_url ?? ""}
            onInput={(e) =>
              props.onOnlineChange(
                "api_base_url",
                e.currentTarget.value || null
              )
            }
            placeholder="https://api.openai.com/v1 (optional)"
          />
        </div>
        <div>
          <label class={labelClass}>Model</label>
          <input
            type="text"
            class={inputClass}
            value={props.config.online.model ?? ""}
            onInput={(e) =>
              props.onOnlineChange("model", e.currentTarget.value || null)
            }
            placeholder="whisper-1 (optional)"
          />
        </div>
      </div>
    </section>
  );
}
