import { Show } from "solid-js";

interface Props {
  state: string;
  asrText: string;
  llmText: string;
  finalText: string;
  error: string;
}

function RecordingOverlay(props: Props) {
  const stateLabel = () => {
    switch (props.state) {
      case "recording":
        return "Recording...";
      case "processing":
      case "transcribing":
        return "Transcribing...";
      case "post_processing":
        return "Polishing...";
      case "done":
        return "Done";
      default:
        return "Ready";
    }
  };

  const displayText = () => {
    if (props.finalText) return props.finalText;
    if (props.llmText) return props.llmText;
    if (props.asrText) return props.asrText;
    return "";
  };

  const isRecording = () => props.state === "recording";
  const isProcessing = () =>
    ["processing", "transcribing", "post_processing"].includes(props.state);

  return (
    <div class="flex h-full items-center justify-center p-2">
      <div class="w-full rounded-xl p-3">
        {/* Status Bar */}
        <div class="mb-2 flex items-center gap-2">
          <div
            class={`h-2 w-2 rounded-full ${
              isRecording()
                ? "animate-pulse bg-red-500"
                : isProcessing()
                  ? "animate-pulse bg-yellow-500"
                  : props.state === "done"
                    ? "bg-green-500"
                    : "bg-gray-400"
            }`}
          />
          <span class="text-xs font-medium text-white/80">{stateLabel()}</span>
        </div>

        {/* Waveform placeholder */}
        <Show when={isRecording()}>
          <div class="mb-2 flex h-8 items-center justify-center gap-[2px]">
            {Array.from({ length: 20 }).map((_, i) => (
              <div
                class="w-[3px] animate-pulse rounded-full bg-white/50"
                style={{
                  height: `${4 + Math.random() * 20}px`,
                  "animation-delay": `${i * 50}ms`,
                  "animation-duration": `${600 + Math.random() * 400}ms`,
                }}
              />
            ))}
          </div>
        </Show>

        {/* Text display */}
        <Show when={displayText()}>
          <div class="max-h-16 overflow-y-auto rounded-lg bg-white/10 px-2 py-1.5">
            <p class="text-xs leading-relaxed text-white/90">{displayText()}</p>
          </div>
        </Show>

        {/* Error display */}
        <Show when={props.error}>
          <div class="mt-1 rounded-lg bg-red-500/20 px-2 py-1">
            <p class="text-xs text-red-300">{props.error}</p>
          </div>
        </Show>

        {/* Idle hint */}
        <Show when={props.state === "idle" && !displayText()}>
          <p class="text-center text-xs text-white/40">
            Press Option+Space to start
          </p>
        </Show>

        {/* Escape hint */}
        <Show when={props.state !== "idle" && props.state !== "done"}>
          <p class="mt-1 text-center text-[10px] text-white/30">
            Esc to cancel
          </p>
        </Show>
      </div>
    </div>
  );
}

export default RecordingOverlay;
