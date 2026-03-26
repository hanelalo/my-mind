import type { ShortcutConfig } from "../../lib/config";

interface Props {
  config: ShortcutConfig;
  onChange: <K extends keyof ShortcutConfig>(
    key: K,
    value: ShortcutConfig[K]
  ) => void;
}

const selectClass =
  "w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-900 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500";
const inputClass =
  "w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500";
const labelClass = "block text-sm font-medium text-gray-700 mb-1";

export default function ShortcutSection(props: Props) {
  return (
    <section class="rounded-lg border border-gray-200 bg-white p-4">
      <h2 class="mb-3 text-sm font-semibold text-gray-900">Shortcuts</h2>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class={labelClass}>Record Shortcut</label>
          <input
            type="text"
            class={inputClass}
            value={props.config.record}
            onInput={(e) => props.onChange("record", e.currentTarget.value)}
            placeholder="Alt+Space"
          />
          <p class="mt-1 text-xs text-amber-600">
            Changes take effect after restart
          </p>
        </div>
        <div>
          <label class={labelClass}>Mode</label>
          <select
            class={selectClass}
            value={props.config.mode}
            onChange={(e) => props.onChange("mode", e.currentTarget.value)}
          >
            <option value="push_to_talk">Push to Talk</option>
            <option value="toggle">Toggle</option>
          </select>
        </div>
      </div>
    </section>
  );
}
