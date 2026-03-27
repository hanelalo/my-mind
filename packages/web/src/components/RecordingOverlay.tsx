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
    <div class="h-screen w-screen p-1">
      <div class="flex h-full flex-col rounded-xl bg-gray-900 p-4">
        {/* Status Bar */}
        <div class="flex items-center gap-2">
          <div
            class={`h-2.5 w-2.5 rounded-full ${
              isRecording()
                ? "animate-pulse bg-red-500"
                : isProcessing()
                  ? "animate-pulse bg-yellow-500"
                  : props.state === "done"
                    ? "bg-green-500"
                    : "bg-gray-400"
            }`}
          />
          <span class="text-sm font-medium text-white/90">{stateLabel()}</span>
        </div>

        {/* Waveform placeholder */}
        <Show when={isRecording()}>
          <div class="mt-3 flex h-8 items-center justify-center gap-[2px]">
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
          <div class="mt-3 min-h-0 flex-1 overflow-y-auto">
            <p class="text-sm leading-relaxed text-white/90">{displayText()}</p>
          </div>
        </Show>

        {/* Error display */}
        <Show when={props.error}>
          <div class="mt-3 rounded-lg bg-red-500/20 px-3 py-2">
            <p class="text-sm text-red-300">{props.error}</p>
          </div>
        </Show>

        {/* Idle hint */}
        <Show when={props.state === "idle" && !displayText()}>
          <p class="mt-3 text-center text-xs text-white/40">
            Press Option+Space to start
          </p>
        </Show>

        {/* Escape hint */}
        <Show when={props.state !== "idle" && props.state !== "done"}>
          <p class="mt-2 text-center text-[10px] text-white/30">
            Esc to cancel
          </p>
        </Show>
      </div>
    </div>
  );
}

export default RecordingOverlay;
