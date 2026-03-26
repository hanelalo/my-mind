import type { OutputConfig } from "../../lib/config";

interface Props {
  config: OutputConfig;
  onChange: <K extends keyof OutputConfig>(
    key: K,
    value: OutputConfig[K]
  ) => void;
}

export default function OutputSection(props: Props) {
  return (
    <section class="rounded-lg border border-gray-200 bg-white p-4">
      <h2 class="mb-3 text-sm font-semibold text-gray-900">Output</h2>
      <label class="flex items-center gap-2 text-sm text-gray-700">
        <input
          type="checkbox"
          class="rounded border-gray-300"
          checked={props.config.auto_paste}
          onChange={(e) =>
            props.onChange("auto_paste", e.currentTarget.checked)
          }
        />
        Auto-paste after processing
      </label>
    </section>
  );
}
