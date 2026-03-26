import { createSignal, onMount, onCleanup, Show } from "solid-js";
import RecordingOverlay from "./components/RecordingOverlay";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type PipelineState =
  | "idle"
  | "recording"
  | "processing"
  | "transcribing"
  | "post_processing"
  | "done";

function App() {
  const [state, setState] = createSignal<PipelineState>("idle");
  const [asrText, setAsrText] = createSignal("");
  const [llmText, setLlmText] = createSignal("");
  const [finalText, setFinalText] = createSignal("");
  const [error, setError] = createSignal("");

  onMount(async () => {
    // Listen for Escape key to cancel recording and hide overlay
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        invoke("cancel_recording").catch((err) =>
          console.error("Failed to cancel recording:", err)
        );
        // Reset local UI state
        setState("idle");
        setAsrText("");
        setLlmText("");
        setFinalText("");
        setError("");
      }
    };
    document.addEventListener("keydown", handleKeyDown);

    const unlistenState = await listen<string>("pipeline:state", (e) => {
      setState(e.payload as PipelineState);
    });

    const unlistenAsr = await listen<string>("asr:result", (e) => {
      setAsrText(e.payload);
    });

    const unlistenLlm = await listen<string>("llm:result", (e) => {
      setLlmText(e.payload);
    });

    const unlistenDone = await listen<string>("pipeline:done", (e) => {
      setFinalText(e.payload);
      setState("done");
    });

    const unlistenError = await listen<string>("pipeline:error", (e) => {
      setError(e.payload);
    });

    onCleanup(() => {
      document.removeEventListener("keydown", handleKeyDown);
      unlistenState();
      unlistenAsr();
      unlistenLlm();
      unlistenDone();
      unlistenError();
    });
  });

  return (
    <div class="h-screen w-screen select-none" style="background: transparent;">
      <RecordingOverlay
        state={state()}
        asrText={asrText()}
        llmText={llmText()}
        finalText={finalText()}
        error={error()}
      />
    </div>
  );
}

export default App;
