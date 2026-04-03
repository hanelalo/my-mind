import { createSignal, onMount, For, Show } from "solid-js";
import {
  getHistory,
  getHistoryCount,
  deleteHistoryRecord,
  clearHistory,
  diagnosePrompt,
  checkQuality,
  applyPromptSuggestion,
  type HistoryRecord,
  type DiagnosisMessage,
} from "./lib/history";
import { getConfig, saveConfig } from "./lib/config";

const PAGE_SIZE = 20;

function formatTime(ts: number): string {
  const d = new Date(ts * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export default function HistoryApp() {
  const [records, setRecords] = createSignal<HistoryRecord[]>([]);
  const [total, setTotal] = createSignal(0);
  const [page, setPage] = createSignal(0);
  const [loading, setLoading] = createSignal(true);
  const [expanded, setExpanded] = createSignal<string | null>(null);
  
  // Diagnosis dialog state
  const [diagnosingRecord, setDiagnosingRecord] = createSignal<HistoryRecord | null>(null);
  const [diagnosisMessages, setDiagnosisMessages] = createSignal<DiagnosisMessage[]>([]);
  const [diagnosisInput, setDiagnosisInput] = createSignal("");
  const [diagnosisLoading, setDiagnosisLoading] = createSignal(false);

  // Quality check state
  const [qualityCheckRecord, setQualityCheckRecord] = createSignal<HistoryRecord | null>(null);
  const [qualityCheckReport, setQualityCheckReport] = createSignal<string | null>(null);
  const [qualityCheckLoading, setQualityCheckLoading] = createSignal(false);

  // Apply suggestion state
  const [applyLoading, setApplyLoading] = createSignal(false);
  const [previewPrompt, setPreviewPrompt] = createSignal<string | null>(null);
  const [saveLoading, setSaveLoading] = createSignal(false);
  const [saveSuccess, setSaveSuccess] = createSignal(false);

  const totalPages = () => Math.max(1, Math.ceil(total() / PAGE_SIZE));

  const load = async () => {
    setLoading(true);
    try {
      const [list, count] = await Promise.all([
        getHistory(PAGE_SIZE, page() * PAGE_SIZE),
        getHistoryCount(),
      ]);
      setRecords(list);
      setTotal(count);
    } catch (e) {
      console.error("Failed to load history:", e);
    } finally {
      setLoading(false);
    }
  };

  onMount(load);

  const handleDelete = async (id: string) => {
    try {
      await deleteHistoryRecord(id);
      await load();
    } catch (e) {
      console.error("Failed to delete:", e);
    }
  };

  const handleClear = async () => {
    if (!confirm("确认清空所有历史记录？")) return;
    try {
      await clearHistory();
      setPage(0);
      await load();
    } catch (e) {
      console.error("Failed to clear:", e);
    }
  };

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // fallback: ignore
    }
  };

  const goPage = (p: number) => {
    setPage(p);
    setExpanded(null);
    load();
  };

  const openDiagnosis = (record: HistoryRecord) => {
    setDiagnosingRecord(record);
    setDiagnosisMessages([]);
    setDiagnosisInput("");
  };

  const closeDiagnosis = () => {
    setDiagnosingRecord(null);
    setDiagnosisMessages([]);
    setDiagnosisInput("");
  };

  const sendDiagnosisMessage = async () => {
    const record = diagnosingRecord();
    const message = diagnosisInput().trim();
    if (!record || !message || diagnosisLoading()) return;

    const userMessage: DiagnosisMessage = { role: "user", content: message };
    const currentMessages = diagnosisMessages();
    setDiagnosisMessages([...currentMessages, userMessage]);
    setDiagnosisInput("");
    setDiagnosisLoading(true);

    try {
      const response = await diagnosePrompt({
        asr_text: record.asr_text,
        final_text: record.final_text,
        user_message: message,
        conversation_history: currentMessages,
      });

      const assistantMessage: DiagnosisMessage = {
        role: "assistant",
        content: response.reply,
      };
      setDiagnosisMessages([...currentMessages, userMessage, assistantMessage]);
    } catch (e) {
      console.error("Diagnosis failed:", e);
      const errorMessage: DiagnosisMessage = {
        role: "assistant",
        content: `Error: ${e instanceof Error ? e.message : "Failed to get response"}`,
      };
      setDiagnosisMessages([...currentMessages, userMessage, errorMessage]);
    } finally {
      setDiagnosisLoading(false);
    }
  };

  const runQualityCheck = async (record: HistoryRecord) => {
    if (qualityCheckLoading()) return;

    setQualityCheckRecord(record);
    setQualityCheckReport(null);
    setQualityCheckLoading(true);

    try {
      const response = await checkQuality({
        asr_text: record.asr_text,
        final_text: record.final_text,
      });
      setQualityCheckReport(response.report);
    } catch (e) {
      console.error("Quality check failed:", e);
      setQualityCheckReport(`Error: ${e instanceof Error ? e.message : "Failed to get quality check"}`);
    } finally {
      setQualityCheckLoading(false);
    }
  };

  const closeQualityCheck = () => {
    setQualityCheckRecord(null);
    setQualityCheckReport(null);
  };

  const handleApplySuggestion = async (suggestions: string) => {
    if (applyLoading()) return;
    setApplyLoading(true);
    try {
      const response = await applyPromptSuggestion({ suggestions });
      setPreviewPrompt(response.new_prompt);
    } catch (e) {
      console.error("Apply suggestion failed:", e);
      alert(`Failed to apply suggestion: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setApplyLoading(false);
    }
  };

  const handleSavePrompt = async () => {
    const newPrompt = previewPrompt();
    if (newPrompt === null || saveLoading()) return;
    setSaveLoading(true);
    setSaveSuccess(false);
    try {
      const config = await getConfig();
      config.llm.prompt = newPrompt;
      await saveConfig(config);
      setSaveSuccess(true);
      setTimeout(() => {
        setPreviewPrompt(null);
        setSaveSuccess(false);
      }, 1500);
    } catch (e) {
      console.error("Save prompt failed:", e);
    } finally {
      setSaveLoading(false);
    }
  };

  return (
    <div class="flex min-h-screen flex-col bg-gray-50">
      {/* Header */}
      <div class="flex items-center justify-between border-b border-gray-200 bg-white px-6 py-4">
        <h1 class="text-lg font-semibold text-gray-900">History</h1>
        <div class="flex items-center gap-3">
          <span class="text-xs text-gray-400">{total()} records</span>
          <button
            class="rounded-md border border-red-200 px-3 py-1 text-xs font-medium text-red-600 hover:bg-red-50 disabled:opacity-50"
            onClick={handleClear}
            disabled={total() === 0}
          >
            Clear All
          </button>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center py-20">
              <p class="text-sm text-gray-400">Loading...</p>
            </div>
          }
        >
          <Show
            when={records().length > 0}
            fallback={
              <div class="flex items-center justify-center py-20">
                <p class="text-sm text-gray-400">No history yet</p>
              </div>
            }
          >
            <ul class="divide-y divide-gray-100">
              <For each={records()}>
                {(record) => {
                  const isExpanded = () => expanded() === record.id;
                  return (
                    <li
                      class="cursor-pointer px-6 py-3 transition-colors hover:bg-gray-50"
                      onClick={() =>
                        setExpanded(isExpanded() ? null : record.id)
                      }
                    >
                      {/* Summary row */}
                      <div class="flex items-start gap-3">
                        <div class="min-w-0 flex-1">
                          <p class="truncate text-sm text-gray-800">
                            {record.final_text}
                          </p>
                          <div class="mt-1 flex items-center gap-2 text-xs text-gray-400">
                            <span>{formatTime(record.timestamp)}</span>
                            <Show when={record.target_app}>
                              <span class="rounded bg-gray-100 px-1.5 py-0.5 text-gray-500">
                                {record.target_app}
                              </span>
                            </Show>
                          </div>
                        </div>
                        <div class="flex shrink-0 items-center gap-1">
                          <button
                            class="rounded p-1 text-indigo-400 hover:bg-indigo-50 hover:text-indigo-600"
                            title="Diagnose Prompt Issue"
                            onClick={(e) => {
                              e.stopPropagation();
                              openDiagnosis(record);
                            }}
                          >
                            <svg
                              class="h-4 w-4"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
                              />
                            </svg>
                          </button>
                          <button
                            class="rounded p-1 text-green-400 hover:bg-green-50 hover:text-green-600"
                            title="Check Quality"
                            onClick={(e) => {
                              e.stopPropagation();
                              runQualityCheck(record);
                            }}
                            disabled={qualityCheckLoading() && qualityCheckRecord()?.id === record.id}
                          >
                            <Show
                              when={!(qualityCheckLoading() && qualityCheckRecord()?.id === record.id)}
                              fallback={
                                <svg
                                  class="h-4 w-4 animate-spin"
                                  fill="none"
                                  viewBox="0 0 24 24"
                                >
                                  <circle
                                    class="opacity-25"
                                    cx="12"
                                    cy="12"
                                    r="10"
                                    stroke="currentColor"
                                    stroke-width="4"
                                  />
                                  <path
                                    class="opacity-75"
                                    fill="currentColor"
                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                  />
                                </svg>
                              }
                            >
                              <svg
                                class="h-4 w-4"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                              >
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                />
                              </svg>
                            </Show>
                          </button>
                          <button
                            class="rounded p-1 text-gray-400 hover:bg-gray-200 hover:text-gray-600"
                            title="Copy"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleCopy(record.final_text);
                            }}
                          >
                            <svg
                              class="h-4 w-4"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <rect
                                x="9"
                                y="9"
                                width="13"
                                height="13"
                                rx="2"
                                stroke-width="2"
                              />
                              <path
                                d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"
                                stroke-width="2"
                              />
                            </svg>
                          </button>
                          <button
                            class="rounded p-1 text-gray-400 hover:bg-red-100 hover:text-red-500"
                            title="Delete"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDelete(record.id);
                            }}
                          >
                            <svg
                              class="h-4 w-4"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                              />
                            </svg>
                          </button>
                        </div>
                      </div>

                      {/* Expanded detail */}
                      <Show when={isExpanded()}>
                        <div class="mt-3 space-y-2 rounded-md bg-gray-50 p-3">
                          <Show when={record.asr_text}>
                            <div>
                              <p class="text-xs font-medium text-gray-500">
                                ASR Original
                              </p>
                              <p class="mt-0.5 whitespace-pre-wrap text-sm text-gray-600">
                                {record.asr_text}
                              </p>
                            </div>
                          </Show>
                          <div>
                            <p class="text-xs font-medium text-gray-500">
                              Final Text
                            </p>
                            <p class="mt-0.5 whitespace-pre-wrap text-sm text-gray-800">
                              {record.final_text}
                            </p>
                          </div>

                        </div>
                      </Show>
                    </li>
                  );
                }}
              </For>
            </ul>
          </Show>
        </Show>
      </div>

      {/* Pagination */}
      <Show when={totalPages() > 1}>
        <div class="flex items-center justify-center gap-2 border-t border-gray-200 bg-white px-6 py-3">
          <button
            class="rounded border border-gray-300 px-3 py-1 text-xs text-gray-600 hover:bg-gray-50 disabled:opacity-40"
            disabled={page() === 0}
            onClick={() => goPage(page() - 1)}
          >
            Prev
          </button>
          <span class="text-xs text-gray-500">
            {page() + 1} / {totalPages()}
          </span>
          <button
            class="rounded border border-gray-300 px-3 py-1 text-xs text-gray-600 hover:bg-gray-50 disabled:opacity-40"
            disabled={page() + 1 >= totalPages()}
            onClick={() => goPage(page() + 1)}
          >
            Next
          </button>
        </div>
      </Show>

      {/* Quality Check Dialog */}
      <Show when={qualityCheckRecord()}>
        {(record) => (
          <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
            onClick={(e) => {
              if (e.target === e.currentTarget) closeQualityCheck();
            }}
          >
            <div class="flex max-h-[80vh] w-full max-w-3xl flex-col rounded-lg bg-white shadow-xl">
              {/* Header */}
              <div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
                <div>
                  <h2 class="text-sm font-semibold text-gray-900">
                    Quality Check Report
                  </h2>
                  <p class="text-xs text-gray-500">
                    Evaluating output against prompt specifications
                  </p>
                </div>
                <button
                  class="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
                  onClick={closeQualityCheck}
                >
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>

              {/* Content */}
              <div class="flex-1 overflow-y-auto p-4">
                <Show
                  when={!qualityCheckLoading()}
                  fallback={
                    <div class="flex flex-col items-center justify-center py-12">
                      <div class="h-8 w-8 animate-spin rounded-full border-2 border-green-500 border-t-transparent" />
                      <p class="mt-3 text-sm text-gray-500">Analyzing output quality...</p>
                    </div>
                  }
                >
                  <Show
                    when={qualityCheckReport()}
                    fallback={
                      <div class="py-8 text-center">
                        <p class="text-sm text-gray-400">Click "Check Quality" to generate a report</p>
                      </div>
                    }
                  >
                    <div class="prose prose-sm max-w-none">
                      <div class="whitespace-pre-wrap text-sm text-gray-800">
                        {qualityCheckReport()}
                      </div>
                    </div>
                  </Show>
                </Show>
              </div>

              {/* Footer */}
              <div class="border-t border-gray-200 px-4 py-3 flex items-center justify-between">
                <Show
                  when={qualityCheckReport() && !qualityCheckLoading()}
                >
                  <button
                    class="rounded-md bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50 flex items-center gap-2"
                    onClick={() => handleApplySuggestion(qualityCheckReport()!)}
                    disabled={applyLoading()}
                  >
                    <Show
                      when={!applyLoading()}
                      fallback={
                        <svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                      }
                    >
                      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </Show>
                    {applyLoading() ? "生成中..." : "应用建议到提示词"}
                  </button>
                </Show>
                <Show when={!qualityCheckReport() || qualityCheckLoading()}>
                  <div />
                </Show>
                <button
                  class="rounded-md bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-200"
                  onClick={closeQualityCheck}
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* Diagnosis Dialog */}
      <Show when={diagnosingRecord()}>
        {(record) => (
          <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
            onClick={(e) => {
              if (e.target === e.currentTarget) closeDiagnosis();
            }}
          >
            <div class="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-lg bg-white shadow-xl">
              {/* Header */}
              <div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
                <div>
                  <h2 class="text-sm font-semibold text-gray-900">
                    Prompt Diagnosis
                  </h2>
                  <p class="text-xs text-gray-500">
                    Discuss issues with this transcription and get prompt improvement suggestions
                  </p>
                </div>
                <button
                  class="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
                  onClick={closeDiagnosis}
                >
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>

              {/* Context Info */}
              <div class="border-b border-gray-100 bg-gray-50 px-4 py-2">
                <div class="mb-1">
                  <span class="text-xs font-medium text-gray-500">ASR:</span>
                  <span class="ml-1 text-xs text-gray-700 line-clamp-1">
                    {record().asr_text}
                  </span>
                </div>
                <div>
                  <span class="text-xs font-medium text-gray-500">Output:</span>
                  <span class="ml-1 text-xs text-gray-700 line-clamp-1">
                    {record().final_text}
                  </span>
                </div>
              </div>

              {/* Messages */}
              <div class="flex-1 overflow-y-auto p-4">
                <Show
                  when={diagnosisMessages().length > 0}
                  fallback={
                    <div class="py-8 text-center">
                      <p class="text-sm text-gray-400">
                        Describe the issue with this transcription result.
                        <br />
                        For example: "The output removed important context" or "It incorrectly changed X to Y"
                      </p>
                    </div>
                  }
                >
                  <div class="space-y-3">
                    <For each={diagnosisMessages()}>
                      {(msg, index) => (
                        <div
                          class={`flex flex-col ${
                            msg.role === "user" ? "items-end" : "items-start"
                          }`}
                        >
                          <div
                            class={`max-w-[85%] rounded-lg px-3 py-2 text-sm ${
                              msg.role === "user"
                                ? "bg-indigo-600 text-white"
                                : "bg-gray-100 text-gray-800"
                            }`}
                          >
                            <div class="whitespace-pre-wrap">{msg.content}</div>
                          </div>
                          {/* Apply suggestion button on last assistant message */}
                          <Show
                            when={
                              msg.role === "assistant" &&
                              index() === diagnosisMessages().length - 1
                            }
                          >
                            <button
                              class="mt-1.5 flex items-center gap-1.5 rounded-md bg-indigo-50 px-3 py-1.5 text-xs font-medium text-indigo-600 hover:bg-indigo-100 disabled:opacity-50"
                              onClick={() => handleApplySuggestion(msg.content)}
                              disabled={applyLoading()}
                            >
                              <Show
                                when={!applyLoading()}
                                fallback={
                                  <svg class="h-3.5 w-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                                  </svg>
                                }
                              >
                                <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                </svg>
                              </Show>
                              {applyLoading() ? "生成新提示词..." : "应用建议到提示词"}
                            </button>
                          </Show>
                        </div>
                      )}
                    </For>
                    <Show when={diagnosisLoading()}>
                      <div class="flex justify-start">
                        <div class="rounded-lg bg-gray-100 px-3 py-2">
                          <div class="flex items-center gap-1">
                            <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-gray-400" />
                            <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-gray-400 [animation-delay:0.1s]" />
                            <div class="h-1.5 w-1.5 animate-bounce rounded-full bg-gray-400 [animation-delay:0.2s]" />
                          </div>
                        </div>
                      </div>
                    </Show>
                  </div>
                </Show>
              </div>

              {/* Input */}
              <div class="border-t border-gray-200 p-3">
                <div class="flex gap-2">
                  <textarea
                    class="flex-1 resize-none rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-indigo-500 focus:outline-none"
                    rows={2}
                    placeholder="Describe the issue or ask a question..."
                    value={diagnosisInput()}
                    onInput={(e) => setDiagnosisInput(e.currentTarget.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        sendDiagnosisMessage();
                      }
                    }}
                    disabled={diagnosisLoading()}
                  />
                  <button
                    class="rounded-md bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50"
                    onClick={sendDiagnosisMessage}
                    disabled={!diagnosisInput().trim() || diagnosisLoading()}
                  >
                    Send
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* Preview & Confirm Prompt Dialog */}
      <Show when={previewPrompt() !== null}>
        <div
          class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4"
          onClick={(e) => {
            if (e.target === e.currentTarget) setPreviewPrompt(null);
          }}
        >
          <div class="flex max-h-[85vh] w-full max-w-3xl flex-col rounded-lg bg-white shadow-2xl">
            {/* Header */}
            <div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
              <div>
                <h2 class="text-sm font-semibold text-gray-900">确认新提示词</h2>
                <p class="text-xs text-gray-500">以下是合并建议后的完整提示词，确认后将自动保存到设置</p>
              </div>
              <button
                class="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
                onClick={() => setPreviewPrompt(null)}
              >
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Prompt Content */}
            <div class="flex-1 overflow-y-auto p-4">
              <textarea
                class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-800 font-mono focus:border-indigo-500 focus:outline-none"
                style="min-height: 300px; resize: vertical;"
                value={previewPrompt() ?? ""}
                onInput={(e) => setPreviewPrompt(e.currentTarget.value)}
              />
              <p class="mt-2 text-xs text-gray-400">可以在保存前手动调整</p>
            </div>

            {/* Footer */}
            <div class="flex items-center justify-between border-t border-gray-200 px-4 py-3">
              <button
                class="rounded-md bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-200"
                onClick={() => setPreviewPrompt(null)}
              >
                取消
              </button>
              <button
                class={`rounded-md px-4 py-2 text-sm font-medium text-white disabled:opacity-50 flex items-center gap-2 ${
                  saveSuccess() ? "bg-green-600 hover:bg-green-700" : "bg-indigo-600 hover:bg-indigo-700"
                }`}
                onClick={handleSavePrompt}
                disabled={saveLoading() || saveSuccess()}
              >
                <Show
                  when={!saveLoading() && !saveSuccess()}
                  fallback={
                    <Show
                      when={saveSuccess()}
                      fallback={
                        <svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                      }
                    >
                      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                    </Show>
                  }
                >
                  <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </Show>
                {saveSuccess() ? "已保存！" : saveLoading() ? "保存中..." : "确认保存"}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
