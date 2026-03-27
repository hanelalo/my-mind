import type { LlmConfig } from "../../lib/config";

interface Props {
  config: LlmConfig;
  onChange: <K extends keyof LlmConfig>(key: K, value: LlmConfig[K]) => void;
}

const selectClass =
  "w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-900 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:bg-gray-100 disabled:text-gray-400";
const inputClass =
  "w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:bg-gray-100 disabled:text-gray-400";
const labelClass = "block text-sm font-medium text-gray-700 mb-1";

export default function LlmSection(props: Props) {
  const disabled = () => !props.config.enabled;

  return (
    <section class="rounded-lg border border-gray-200 bg-white p-4">
      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-900">
          LLM (Post-processing)
        </h2>
        <label class="flex items-center gap-2 text-sm text-gray-600">
          <input
            type="checkbox"
            class="rounded border-gray-300"
            checked={props.config.enabled}
            onChange={(e) => props.onChange("enabled", e.currentTarget.checked)}
          />
          Enable
        </label>
      </div>
      <div
        class="grid grid-cols-2 gap-3"
        classList={{ "opacity-50": disabled() }}
      >
        <div>
          <label class={labelClass}>Provider</label>
          <select
            class={selectClass}
            value={props.config.provider}
            onChange={(e) => props.onChange("provider", e.currentTarget.value)}
            disabled={disabled()}
          >
            <option value="openai">OpenAI</option>
            <option value="anthropic">Anthropic</option>
            <option value="ollama">Ollama</option>
            <option value="custom">Custom</option>
          </select>
        </div>
        <div>
          <label class={labelClass}>Model</label>
          <input
            type="text"
            class={inputClass}
            value={props.config.model}
            onInput={(e) => props.onChange("model", e.currentTarget.value)}
            disabled={disabled()}
            placeholder="gpt-4o-mini"
          />
        </div>
        <div class="col-span-2">
          <label class={labelClass}>API Key</label>
          <input
            type="password"
            class={inputClass}
            value={props.config.api_key}
            onInput={(e) => props.onChange("api_key", e.currentTarget.value)}
            disabled={disabled()}
            placeholder="Enter API key"
          />
        </div>
        <div class="col-span-2">
          <label class={labelClass}>API Base URL</label>
          <input
            type="text"
            class={inputClass}
            value={props.config.api_base_url ?? ""}
            onInput={(e) =>
              props.onChange("api_base_url", e.currentTarget.value || null)
            }
            disabled={disabled()}
            placeholder="https://api.openai.com/v1 (optional)"
          />
        </div>
        <div>
          <label class={labelClass}>Temperature</label>
          <input
            type="number"
            class={inputClass}
            value={props.config.temperature}
            onInput={(e) =>
              props.onChange("temperature", parseFloat(e.currentTarget.value) || 0)
            }
            disabled={disabled()}
            step="0.1"
            min="0"
            max="2"
          />
        </div>
        <div>
          <label class={labelClass}>Max Tokens</label>
          <input
            type="number"
            class={inputClass}
            value={props.config.max_tokens}
            onInput={(e) =>
              props.onChange(
                "max_tokens",
                parseInt(e.currentTarget.value) || 2048
              )
            }
            disabled={disabled()}
            min="1"
            max="65535"
          />
        </div>
        <div class="col-span-2">
          <label class={labelClass}>System Prompt</label>
          <textarea
            class={inputClass + " min-h-[80px] resize-y"}
            value={props.config.prompt}
            onInput={(e) => props.onChange("prompt", e.currentTarget.value)}
            disabled={disabled()}
            placeholder="Leave empty to use built-in default prompt"
            rows={4}
          />
          <p class="mt-1 text-xs text-gray-400">
            Custom system prompt for post-processing. Leave empty to use the
            built-in default.
          </p>
        </div>
      </div>
    </section>
  );
}
